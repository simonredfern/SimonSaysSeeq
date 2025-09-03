//! MIDI Clock Detector Utility
//! 
//! A standalone utility for the Pi5 sequencer that scans all USB MIDI ports
//! and automatically detects and locks onto the first reliable MIDI clock source.
//! 
//! Usage:
//!   cargo run --bin midi_clock_detector
//!   cargo run --bin midi_clock_detector -- --scan-only
//!   cargo run --bin midi_clock_detector -- --port "USB MIDI"
//!   cargo run --bin midi_clock_detector -- --help

use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use log::{error, info, warn};
use midir::{MidiInput, MidiOutput};
use std::io::{self, Write};
use std::time::Duration;
use std::thread;

mod midi_scanner {
    include!("../midi_scanner/mod.rs");
}

use midi_scanner::{scan_for_midi_clock, MidiClockScanner};

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .init();

    let matches = Command::new("MIDI Clock Detector")
        .version("1.0")
        .about("Scans USB MIDI ports for clock signals and locks onto the first reliable source")
        .arg(
            Arg::new("scan-only")
                .long("scan-only")
                .help("Only scan and report results, don't monitor continuously")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("PORT_NAME")
                .help("Manually specify a MIDI port to monitor")
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .value_name("SECONDS")
                .help("Scan timeout in seconds (default: 10)")
                .default_value("10")
        )
        .arg(
            Arg::new("list-ports")
                .long("list-ports")
                .help("List all available MIDI ports and exit")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("Interactive mode for port selection")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("status")
                .long("status")
                .help("Show current detection status from saved configuration")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("force-rescan")
                .long("force-rescan")
                .help("Force re-scan even if clock is currently working")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // List ports and exit if requested
    if matches.get_flag("list-ports") {
        list_midi_ports()?;
        return Ok(());
    }

    // Show status and exit if requested
    if matches.get_flag("status") {
        show_detection_status()?;
        return Ok(());
    }

    // Force rescan if requested
    if matches.get_flag("force-rescan") {
        return force_rescan_and_monitor(timeout_secs);
    }

    // Interactive mode
    if matches.get_flag("interactive") {
        return run_interactive_mode();
    }

    // Get timeout
    let timeout_secs: u64 = matches.get_one::<String>("timeout")
        .unwrap()
        .parse()
        .map_err(|_| anyhow!("Invalid timeout value"))?;

    // Check if specific port was requested
    if let Some(port_name) = matches.get_one::<String>("port") {
        return monitor_specific_port(port_name, timeout_secs);
    }

    // Perform scan
    if matches.get_flag("scan-only") {
        scan_and_report()?;
    } else {
        scan_and_monitor(timeout_secs)?;
    }

    Ok(())
}

/// List all available MIDI input and output ports
fn list_midi_ports() -> Result<()> {
    println!("🎹 Available MIDI Ports:");
    println!();

    // List input ports
    let midi_in = MidiInput::new("Port Lister")?;
    let in_ports = midi_in.ports();
    
    println!("📥 INPUT PORTS ({}):", in_ports.len());
    if in_ports.is_empty() {
        println!("  (none)");
    } else {
        for (i, port) in in_ports.iter().enumerate() {
            let name = midi_in.port_name(port)
                .unwrap_or_else(|_| format!("Port {}", i));
            println!("  {}: {}", i, name);
        }
    }

    println!();

    // List output ports
    let midi_out = MidiOutput::new("Port Lister")?;
    let out_ports = midi_out.ports();
    
    println!("📤 OUTPUT PORTS ({}):", out_ports.len());
    if out_ports.is_empty() {
        println!("  (none)");
    } else {
        for (i, port) in out_ports.iter().enumerate() {
            let name = midi_out.port_name(port)
                .unwrap_or_else(|_| format!("Port {}", i));
            println!("  {}: {}", i, name);
        }
    }

    Ok(())
}

/// Interactive mode for port selection and monitoring
fn run_interactive_mode() -> Result<()> {
    println!("🎯 MIDI Clock Detector - Interactive Mode");
    println!();

    list_midi_ports()?;
    println!();

    // Get available input ports for selection
    let midi_in = MidiInput::new("Interactive Scanner")?;
    let ports = midi_in.ports();

    if ports.is_empty() {
        println!("❌ No MIDI input ports available!");
        return Ok(());
    }

    println!("Choose an option:");
    println!("  0: Auto-scan all ports for clock signals");
    for (i, port) in ports.iter().enumerate() {
        let name = midi_in.port_name(port)
            .unwrap_or_else(|_| format!("Port {}", i));
        println!("  {}: Monitor specific port '{}'", i + 1, name);
    }

    print!("Enter selection (0-{}): ", ports.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection: usize = input.trim().parse().unwrap_or(0);

    if selection == 0 {
        println!("\n🔍 Auto-scanning all ports...");
        scan_and_monitor(10)?;
    } else if selection >= 1 && selection <= ports.len() {
        let port_idx = selection - 1;
        let port_name = midi_in.port_name(&ports[port_idx])
            .unwrap_or_else(|_| format!("Port {}", port_idx));
        println!("\n🎯 Monitoring port '{}'...", port_name);
        monitor_specific_port(&port_name, 10)?;
    } else {
        println!("❌ Invalid selection");
    }

    Ok(())
}

/// Scan all ports and report results without continuous monitoring
fn scan_and_report() -> Result<()> {
    println!("🔍 Scanning all MIDI ports for clock signals...");
    println!();

    let summary = scan_for_midi_clock()?;

    println!("📊 Scan Results:");
    println!("  Ports scanned: {}", summary.total_ports_scanned);
    println!("  Ports with clock activity: {}", summary.ports_with_clock);
    println!("  Reliable clock sources: {}", summary.reliable_sources.len());

    if summary.reliable_sources.is_empty() {
        println!("❌ No reliable MIDI clock sources found");
    } else {
        println!("✅ Reliable clock sources found:");
        for source in &summary.reliable_sources {
            println!("  - {}", source);
        }
        
        if let Some(ref selected) = summary.selected_source {
            println!("🎯 Auto-selected: {}", selected);
        }
    }

    Ok(())
}

/// Scan all ports and monitor the selected one continuously
fn scan_and_monitor(timeout_secs: u64) -> Result<()> {
    println!("🔍 Scanning all MIDI ports for clock signals...");
    
    let scanner = MidiClockScanner::new();
    scanner.start_scan()?;

    // Monitor scan progress
    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    
    print!("Scanning");
    io::stdout().flush()?;

    while scanner.is_scanning() && start_time.elapsed() < timeout {
        scanner.process_events();
        print!(".");
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(500));
    }
    println!();

    scanner.stop_scan();
    let summary = scanner.get_scan_summary();

    println!("📊 Scan completed:");
    println!("  Ports scanned: {}", summary.total_ports_scanned);
    println!("  Ports with clock: {}", summary.ports_with_clock);
    println!("  Reliable sources: {}", summary.reliable_sources.len());

    if summary.reliable_sources.is_empty() {
        println!("❌ No reliable MIDI clock sources found");
        return Ok(());
    }

    println!("✅ Reliable clock sources:");
    for source in &summary.reliable_sources {
        println!("  - {}", source);
    }

    if let Some(selected_source) = summary.selected_source {
        println!("🎯 Auto-selected source: {}", selected_source);
        println!();
        println!("🎵 Starting continuous monitoring...");
        println!("Press Ctrl+C to stop");
        
        // TODO: Here you would start continuous monitoring of the selected source
        // For now, we'll just simulate it
        monitor_clock_source(&selected_source)?;
    } else {
        println!("⚠️  No source was auto-selected");
    }

    Ok(())
}

/// Monitor a specific MIDI port for clock signals
fn monitor_specific_port(port_name: &str, timeout_secs: u64) -> Result<()> {
    println!("🎯 Monitoring MIDI port '{}' for clock signals...", port_name);
    
    // Find the specified port
    let midi_in = MidiInput::new("Specific Port Monitor")?;
    let ports = midi_in.ports();
    
    let target_port = ports.iter().find(|port| {
        midi_in.port_name(port)
            .map(|name| name.to_lowercase().contains(&port_name.to_lowercase()))
            .unwrap_or(false)
    });

    let selected_port = match target_port {
        Some(port) => port,
        None => {
            error!("MIDI port '{}' not found", port_name);
            println!("Available ports:");
            for (i, port) in ports.iter().enumerate() {
                let name = midi_in.port_name(port)
                    .unwrap_or_else(|_| format!("Port {}", i));
                println!("  {}", name);
            }
            return Err(anyhow!("Port not found"));
        }
    };

    let actual_port_name = midi_in.port_name(selected_port)?;
    println!("🔌 Connected to: {}", actual_port_name);
    
    monitor_clock_source(&actual_port_name)?;
    
    Ok(())
}

/// Monitor a specific clock source continuously
fn monitor_clock_source(port_name: &str) -> Result<()> {
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicU32, Ordering};
    
    let midi_in = MidiInput::new(&format!("Monitor-{}", port_name))?;
    let ports = midi_in.ports();
    
    let target_port = ports.iter().find(|port| {
        midi_in.port_name(port)
            .map(|name| name == port_name)
            .unwrap_or(false)
    });

    let selected_port = match target_port {
        Some(port) => port,
        None => return Err(anyhow!("Port '{}' no longer available", port_name)),
    };

    // Statistics tracking
    let tick_count = Arc::new(AtomicU32::new(0));
    let last_tick_time = Arc::new(Mutex::new(std::time::Instant::now()));
    let start_time = std::time::Instant::now();

    let tick_count_cb = tick_count.clone();
    let last_tick_cb = last_tick_time.clone();

    // Connect with monitoring callback
    let _connection = midi_in.connect(selected_port, &format!("Monitor-{}", port_name),
        move |_timestamp, message, _| {
            if !message.is_empty() && message[0] == 0xF8 {
                // MIDI Clock tick
                let count = tick_count_cb.fetch_add(1, Ordering::Relaxed) + 1;
                *last_tick_cb.lock().unwrap() = std::time::Instant::now();
                
                if count % 24 == 0 {
                    // Every beat (24 ticks)
                    print!("♪");
                } else if count % 6 == 0 {
                    // Every 16th note (6 ticks)  
                    print!(".");
                }
                io::stdout().flush().ok();
            } else if !message.is_empty() {
                match message[0] {
                    0xFA => print!(" [START] "),
                    0xFB => print!(" [CONTINUE] "),
                    0xFC => print!(" [STOP] "),
                    _ => {}
                }
                io::stdout().flush().ok();
            }
        }, ()
    )?;

    println!("🎵 Listening for MIDI clock (♪ = beat, . = 16th note)...");
    println!("Press Enter to stop monitoring");
    println!();

    // Monitor in background and provide status updates
    let tick_count_status = tick_count.clone();
    let last_tick_status = last_tick_time.clone();
    
    let status_thread = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            
            let current_ticks = tick_count_status.load(Ordering::Relaxed);
            let last_tick = *last_tick_status.lock().unwrap();
            let elapsed = start_time.elapsed().as_secs_f32();
            
            if current_ticks > 0 {
                let bpm = (current_ticks as f32 / 24.0) / (elapsed / 60.0);
                let time_since_last = last_tick.elapsed().as_millis();
                
                println!("\n📊 Stats: {} ticks, {:.1} BPM, last tick {}ms ago", 
                         current_ticks, bpm, time_since_last);
                
                if time_since_last > 2000 {
                    println!("⚠️  Clock signal lost!");
                }
            } else {
                println!("\n⏳ Waiting for clock signal...");
            }
        }
    });

    // Wait for user input to stop
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    // Clean shutdown
    drop(status_thread);
    println!("\n🛑 Monitoring stopped");
    
    let final_ticks = tick_count.load(Ordering::Relaxed);
    let total_time = start_time.elapsed().as_secs_f32();
    
    if final_ticks > 0 {
        let final_bpm = (final_ticks as f32 / 24.0) / (total_time / 60.0);
        println!("📈 Final stats: {} total ticks, {:.1} average BPM over {:.1}s", 
                 final_ticks, final_bpm, total_time);
    }

    Ok(())
}

/// Show current MIDI clock detection status from saved configuration
fn show_detection_status() -> Result<()> {
    use std::path::PathBuf;
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct MidiConfigStatus {
        device: String,
        last_detected_device: Option<String>,
        auto_detect_clock: bool,
        detection_retry_interval: u64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ConfigStatus {
        midi: MidiConfigStatus,
    }

    println!("🎛️  MIDI Clock Detection Status");
    println!("================================");
    println!();

    // Try to find and read config file
    let config_path = if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("simon-says-seeq").join("config.toml")
    } else {
        PathBuf::from("simon_says_seeq_config.toml")
    };

    if !config_path.exists() {
        println!("📄 Configuration file: Not found");
        println!("   Expected location: {:?}", config_path);
        println!("   Status: Using defaults (auto-detection enabled)");
        println!();
        return Ok(());
    }

    println!("📄 Configuration file: {:?}", config_path);

    match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            match toml::from_str::<ConfigStatus>(&content) {
                Ok(config) => {
                    println!("✅ Configuration loaded successfully");
                    println!();
                    
                    println!("⚙️  MIDI Settings:");
                    println!("   Manual device: '{}'", if config.midi.device.is_empty() { "(auto-detect)" } else { &config.midi.device });
                    println!("   Auto-detection: {}", if config.midi.auto_detect_clock { "Enabled" } else { "Disabled" });
                    println!("   Retry interval: {} seconds", config.midi.detection_retry_interval);
                    
                    println!();
                    println!("🎯 Last Detection Result:");
                    if let Some(ref detected) = config.midi.last_detected_device {
                        println!("   Last found device: '{}'", detected);
                        println!("   Status: Will try this device first on next startup");
                    } else {
                        println!("   Last found device: None");
                        println!("   Status: Will perform full scan on next startup");
                    }
                    
                    println!();
                    println!("🔄 Next Startup Behavior:");
                    if config.midi.device.is_empty() && config.midi.auto_detect_clock {
                        if config.midi.last_detected_device.is_some() {
                            println!("   1. Try last known device first");
                            println!("   2. If that fails, scan all ports");
                            println!("   3. Lock onto first reliable clock found");
                        } else {
                            println!("   1. Scan all available ports");
                            println!("   2. Lock onto first reliable clock found");
                            println!("   3. Save successful device for future use");
                        }
                    } else if !config.midi.device.is_empty() {
                        println!("   1. Connect directly to '{}'", config.midi.device);
                        println!("   2. No auto-detection (manual mode)");
                    } else {
                        println!("   1. Auto-detection is disabled");
                        println!("   2. Will use first available port");
                    }
                }
                Err(e) => {
                    println!("❌ Failed to parse configuration: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to read configuration: {}", e);
        }
    }

    Ok(())
}

/// Force a complete re-scan even if clock is currently working
fn force_rescan_and_monitor(timeout_secs: u64) -> Result<()> {
    println!("🔄 Forcing complete MIDI clock re-scan...");
    println!("   This will disconnect any current clock source and scan all ports fresh.");
    
    // Clear any saved device to force full scan
    clear_saved_device()?;
    
    println!("🔍 Starting fresh scan of all MIDI ports...");
    scan_and_monitor(timeout_secs)?;
    
    Ok(())
}

/// Clear the saved detected device from configuration
fn clear_saved_device() -> Result<()> {
    use std::path::PathBuf;
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct MidiConfigForClear {
        device: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_detected_device: Option<String>,
        #[serde(flatten)]
        other: toml::Value,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ConfigForClear {
        midi: MidiConfigForClear,
        #[serde(flatten)]
        other: toml::Value,
    }

    let config_path = if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("simon-says-seeq").join("config.toml")
    } else {
        PathBuf::from("simon_says_seeq_config.toml")
    };

    if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                if let Ok(mut config) = toml::from_str::<toml::Value>(&content) {
                    // Clear the last_detected_device field
                    if let Some(midi_table) = config.get_mut("midi").and_then(|v| v.as_table_mut()) {
                        midi_table.remove("last_detected_device");
                        
                        let updated_content = toml::to_string_pretty(&config)?;
                        std::fs::write(&config_path, updated_content)?;
                        println!("✅ Cleared saved device from configuration");
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read config file: {}", e);
            }
        }
    }
    
    Ok(())
}