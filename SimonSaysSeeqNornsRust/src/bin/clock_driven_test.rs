//! Direct Test Runner - Tick-synchronized testing

use std::fs;
use std::error::Error;
use std::thread;
use std::time::Duration;
use std::io::{self, Write};

use clap::Parser;

use simon_says_seeq_rust::clock_generator::{ClockGenerator, ClockConfig};
use simon_says_seeq_rust::test_script::DirectTestScript;

/// Load saved MIDI port selections from file
fn load_saved_ports() -> Option<(usize, usize)> {
    if let Ok(content) = fs::read_to_string("midi_ports.conf") {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() >= 2 {
            if let (Ok(out_port), Ok(in_port)) = (lines[0].parse(), lines[1].parse()) {
                println!("📁 Loaded saved MIDI ports: output={}, input={}", out_port, in_port);
                return Some((out_port, in_port));
            }
        }
    }
    None
}

/// Save MIDI port selections to file
fn save_ports(out_port: usize, in_port: usize) -> Result<(), Box<dyn Error>> {
    let content = format!("{}\n{}\n", out_port, in_port);
    fs::write("midi_ports.conf", content)?;
    println!("💾 Saved MIDI port selections to midi_ports.conf (will be used for --all runs)");
    Ok(())
}

/// Prompt user to select output port and save selection
fn prompt_and_save_output_port() -> Result<usize, Box<dyn Error>> {
    use midir::MidiOutput;
    
    let midi_out = MidiOutput::new("MIDI Clock Generator")?;
    let out_ports = midi_out.ports();
    
    if out_ports.is_empty() {
        return Err("No MIDI output ports available".into());
    }
    
    println!("\nAvailable MIDI output ports:");
    for (i, port) in out_ports.iter().enumerate() {
        let port_name = midi_out.port_name(port).unwrap_or_else(|_| "Unknown".to_string());
        println!("  {}: {}", i, port_name);
    }
    
    print!("Select MIDI output port (0-{}): ", out_ports.len() - 1);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selected = input.trim().parse::<usize>().unwrap_or(0);
    
    if selected >= out_ports.len() {
        return Err(format!("Invalid port selection: {}", selected).into());
    }
    
    Ok(selected)
}

/// Prompt user to select input port and save both selections
fn prompt_and_save_input_port(out_port: usize) -> Result<usize, Box<dyn Error>> {
    use midir::MidiInput;
    
    let midi_in = MidiInput::new("MIDI Input")?;
    let in_ports = midi_in.ports();
    
    if in_ports.is_empty() {
        return Err("No MIDI input ports available".into());
    }
    
    println!("\nAvailable MIDI input ports:");
    for (i, port) in in_ports.iter().enumerate() {
        let port_name = midi_in.port_name(port).unwrap_or_else(|_| "Unknown".to_string());
        println!("  {}: {}", i, port_name);
    }
    
    print!("Select MIDI input port (0-{}): ", in_ports.len() - 1);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selected = input.trim().parse::<usize>().unwrap_or(0);
    
    if selected >= in_ports.len() {
        return Err(format!("Invalid port selection: {}", selected).into());
    }
    
    // Save both ports
    save_ports(out_port, selected)?;
    
    Ok(selected)
}

#[derive(Parser, Debug)]
#[clap(name = "clock_driven_test")]
#[clap(about = "Run clock-driven tick-synchronized tests")]
struct Args {
    #[arg(long)]
    script: Option<String>,
    
    #[arg(long)]
    all: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.all {
        return run_all_tests();
    }

    let script_path = args.script.as_ref().ok_or("--script is required when not using --all")?;

    println!("🧪 Clock-Driven Test Runner");
    println!("═══════════════════════════════════════════════════════");
    println!();

    // Load test script
    println!("📜 Loading test script: {}", script_path);
    let script_content = fs::read_to_string(script_path)?;
    let script: DirectTestScript = serde_json::from_str(&script_content)?;
    
    let test_name = script.name.clone();
    let script_file = script_path.clone();
    
    println!("🧪 Test: {}", test_name);
    println!("📋 Description: {}", script.description);
    println!("🎵 BPM: {}", script.bpm);
    println!("📊 Commands: {}", script.commands.len());
    println!("═══════════════════════════════════════════════════════");
    println!();
    
    println!("⚠️  IMPORTANT: The main sequencer application must be running in test mode!");
    println!("   Start it in another terminal: cargo run --release --bin simon_says_seeq -- --test-mode");
    println!("   The sequencer receives MIDI clock/SysEx and writes formal_state.log");
    println!();

    // Truncate formal_state.log
    fs::write("formal_state.log", "")?;
    println!("🗑️  Truncated formal_state.log for clean test run");
    println!();

    // Create clock generator
    let config = ClockConfig {
        enable_tick_counting: true,
        enable_test_mode: false,
    };
    let mut generator = ClockGenerator::new_with_config(script.bpm, config);
    
    // Calculate max tick from script for auto-exit
    let max_tick = script.commands.iter().map(|cmd| cmd.at_tick).max().unwrap_or(0);
    let grace_ticks = 12; // Wait 2 extra steps after last command
    let exit_tick = max_tick + grace_ticks;
    
    generator.set_direct_test_script(script);

    // Connect to MIDI - use saved ports or prompt and save
    let (connection, midi_input) = if let Some((out_port, in_port)) = load_saved_ports() {
        // Use saved ports
        let conn = connect_midi_output_by_index(&generator, out_port)?;
        let input = connect_midi_input_by_index(&generator, in_port)?;
        (conn, input)
    } else {
        // First run - prompt for both ports and save
        println!("No saved MIDI ports found. Please select ports (will be saved for future runs):");
        let out_port = prompt_and_save_output_port()?;
        let conn = connect_midi_output_by_index(&generator, out_port)?;
        let in_port = prompt_and_save_input_port(out_port)?;
        let input = connect_midi_input_by_index(&generator, in_port)?;
        (conn, input)
    };
    
    let _midi_input = midi_input;
    println!("✅ MIDI Clock initialized at {:.1} BPM", generator.get_bpm());
    println!("✅ MIDI Input connected");
    println!();

    // Start the clock thread (needed before sending MIDI messages)
    let _clock_thread = generator.spawn_clock_thread(connection);
    thread::sleep(Duration::from_millis(100)); // Give thread time to initialize
    
    // Initialize sequencer: Verify Test Mode, Stop
    println!("🔧 Initializing sequencer...");
    
    // Verify sequencer is in test mode FIRST
    println!("  🧪 Querying sequencer test mode...");
    let test_mode_query = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, 0x10, 0xF7];
    generator.send_raw_midi(&test_mode_query)?;
    thread::sleep(Duration::from_millis(500)); // Wait for response
    
    // Check if we received test mode confirmation (0x11) or rejection (0x12)
    // The response will be in the MIDI input buffer
    let test_mode_verified = generator.check_test_mode_response();
    if !test_mode_verified {
        println!();
        println!("❌ TEST FAILED: Sequencer is NOT in test mode!");
        println!("   Please restart the sequencer with: cargo run --release --bin simon_says_seeq -- --test-mode");
        println!();
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Sequencer not in test mode"
        )));
    }
    println!("  ✅ Sequencer is in test mode");
    
    // Send MIDI Stop (in test mode, this will NOT auto-save)
    println!("  ⏹️  Sending MIDI Stop");
    generator.send_raw_midi(&[0xFC])?;
    thread::sleep(Duration::from_millis(500));
    
    // In test mode, the sequencer already loaded test_pattern_1.json at startup
    // No need to copy or reload - just proceed with test
    println!("  ✅ Test pattern already loaded (test mode auto-loads test_pattern_1.json)");
    
    println!();
    println!("🚀 Starting clock and test execution...");
    println!();
    
    // Start the clock
    generator.start()?;
    
    println!("Press Ctrl+C to stop test");
    println!();
    println!("═══════════════════════════════════════════════════════");
    println!("🧪 Running Test: {}", test_name);
    println!("📄 Script File: {}", script_file);
    println!("═══════════════════════════════════════════════════════");
    println!();
    
    // Monitor test progress and wait for completion
    let mut last_tick = 0;
    let mut no_progress_count = 0;
    
    loop {
        thread::sleep(Duration::from_millis(500));
        
        let current_tick = generator.get_tick_count();
        
        // Check if we've reached the exit tick
        if current_tick >= exit_tick {
            println!();
            println!("═══════════════════════════════════════════════════════");
            println!("📊 Test Complete - Reached tick {} (exit at {})", current_tick, exit_tick);
            break;
        }
        
        // Detect if test is stalled (no progress for 10 seconds)
        if current_tick == last_tick {
            no_progress_count += 1;
            if no_progress_count >= 20 { // 20 * 500ms = 10 seconds
                println!();
                println!("⚠️  Warning: No progress detected for 10 seconds at tick {}", current_tick);
                println!("   Test may have stalled. Exiting...");
                break;
            }
        } else {
            no_progress_count = 0;
            last_tick = current_tick;
        }
    }
    
    // Get test statistics
    let (passed, failed) = generator.get_test_stats();
    let total = passed + failed;
    
    println!();
    println!("═══════════════════════════════════════════════════════");
    println!("📊 Test Results for: {}", test_name);
    println!("═══════════════════════════════════════════════════════");
    println!("Total Verifications: {}", total);
    println!("✅ Passed: {}", passed);
    println!("❌ Failed: {}", failed);
    
    if failed == 0 && total > 0 {
        println!();
        println!("🎉 ALL TESTS PASSED!");
        println!("═══════════════════════════════════════════════════════");
        Ok(())
    } else if total == 0 {
        println!();
        println!("⚠️  No verifications were run");
        println!("═══════════════════════════════════════════════════════");
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No verifications run"
        )))
    } else {
        println!();
        println!("❌ TEST FAILED - {} verification(s) failed", failed);
        println!("═══════════════════════════════════════════════════════");
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{} test(s) failed", failed)
        )))
    }
}

fn run_all_tests() -> Result<(), Box<dyn Error>> {
    let test_files = vec![
        "test1.json",
        "test2.json",
        "test3.json",
        "test4.json",
        "test5.json",
    ];
    
    let mut log_file = fs::File::create("latest_tests.log")?;
    let mut all_passed = true;
    let mut results = Vec::new();
    
    writeln!(log_file, "🧪 Running All Tests")?;
    writeln!(log_file, "═══════════════════════════════════════════════════════")?;
    writeln!(log_file)?;
    
    for test_file in test_files {
        writeln!(log_file, "Running: {}", test_file)?;
        writeln!(log_file, "───────────────────────────────────────────────────────")?;
        
        // Check if file exists
        if !std::path::Path::new(test_file).exists() {
            writeln!(log_file, "⚠️  Test file not found: {}", test_file)?;
            writeln!(log_file)?;
            continue;
        }
        
        // Run the test
        match run_single_test(test_file, &mut log_file) {
            Ok(_) => {
                results.push((test_file, true));
                writeln!(log_file, "✅ PASSED: {}", test_file)?;
            }
            Err(e) => {
                all_passed = false;
                results.push((test_file, false));
                writeln!(log_file, "❌ FAILED: {} - {}", test_file, e)?;
            }
        }
        writeln!(log_file)?;
        
        // Wait between tests
        thread::sleep(Duration::from_secs(2));
    }
    
    writeln!(log_file, "═══════════════════════════════════════════════════════")?;
    writeln!(log_file, "Test Summary:")?;
    writeln!(log_file, "═══════════════════════════════════════════════════════")?;
    
    for (test, passed) in &results {
        if *passed {
            writeln!(log_file, "✅ {}", test)?;
        } else {
            writeln!(log_file, "❌ {}", test)?;
        }
    }
    
    let passed_count = results.iter().filter(|(_, p)| *p).count();
    let total_count = results.len();
    
    writeln!(log_file)?;
    writeln!(log_file, "Results: {}/{} passed", passed_count, total_count)?;
    writeln!(log_file, "═══════════════════════════════════════════════════════")?;
    
    println!("✅ All tests complete. Results written to latest_tests.log");
    
    if all_passed {
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{}/{} tests failed", total_count - passed_count, total_count)
        )))
    }
}

fn run_single_test(script_path: &str, log_file: &mut fs::File) -> Result<(), Box<dyn Error>> {
    // Load test script
    let script_content = fs::read_to_string(script_path)?;
    let script: DirectTestScript = serde_json::from_str(&script_content)?;
    
    writeln!(log_file, "Test: {}", script.name)?;
    writeln!(log_file, "Description: {}", script.description)?;
    let bpm = script.bpm;
    
    // Truncate formal_state.log
    fs::write("formal_state.log", "")?;
    
    // Create clock generator
    let config = ClockConfig {
        enable_tick_counting: true,
        enable_test_mode: false,
    };
    let mut generator = ClockGenerator::new_with_config(script.bpm, config);
    
    // Calculate max tick for auto-exit
    let max_tick = script.commands.iter().map(|cmd| cmd.at_tick).max().unwrap_or(0);
    let grace_ticks = 12;
    let exit_tick = max_tick + grace_ticks;
    
    generator.set_direct_test_script(script);
    
    // Connect MIDI using saved ports (or prompt and save on first run)
    let connection = if let Some((out_port, _)) = load_saved_ports() {
        // Use saved output port
        connect_midi_output_by_index(&generator, out_port)?
    } else {
        // First run - prompt user and save selection
        println!("First run - please select MIDI ports (will be saved for future runs)");
        let conn = generator.connect_midi_output()?;
        conn
    };
    
    let _midi_input = if let Some((_, in_port)) = load_saved_ports() {
        // Use saved input port
        connect_midi_input_by_index(&generator, in_port)?
    } else {
        generator.connect_midi_input()?
    };
    
    // Start clock thread
    let _clock_thread = generator.spawn_clock_thread(connection);
    thread::sleep(Duration::from_millis(100));
    
    // Verify test mode
    let test_mode_query = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, 0x10, 0xF7];
    generator.send_raw_midi(&test_mode_query)?;
    thread::sleep(Duration::from_millis(500));
    
    let test_mode_verified = generator.check_test_mode_response();
    if !test_mode_verified {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Sequencer not in test mode"
        )));
    }
    
    // Stop sequencer
    let stop_cmd = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, 0x02, 0xF7];
    generator.send_raw_midi(&stop_cmd)?;
    thread::sleep(Duration::from_millis(100));
    
    // Start clock
    generator.start()?;
    
    // Wait for test to complete
    let timeout = Duration::from_secs((exit_tick as f64 / 24.0 * 60.0 / bpm as f64 * 2.0) as u64 + 10);
    thread::sleep(timeout);
    
    // Stop clock
    generator.stop()?;
    thread::sleep(Duration::from_millis(500));
    
    // Parse results from formal_state.log
    let formal_log = fs::read_to_string("formal_state.log")?;
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    
    for line in formal_log.lines() {
        if line.contains("✅") && line.contains("Verify") {
            total += 1;
            passed += 1;
        } else if line.contains("❌") && line.contains("Verify") {
            total += 1;
            failed += 1;
        }
    }
    
    writeln!(log_file, "Verifications: {} total, {} passed, {} failed", total, passed, failed)?;
    
    if failed > 0 {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{} verifications failed", failed)
        )))
    } else if total == 0 {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No verifications run"
        )))
    } else {
        Ok(())
    }
}

/// Connect MIDI output by port index
fn connect_midi_output_by_index(generator: &ClockGenerator, port_index: usize) -> Result<midir::MidiOutputConnection, Box<dyn Error>> {
    use midir::MidiOutput;
    
    let midi_out = MidiOutput::new("MIDI Clock Generator")?;
    let out_ports = midi_out.ports();
    
    if port_index >= out_ports.len() {
        return Err(format!("Saved output port {} not available (only {} ports)", port_index, out_ports.len()).into());
    }
    
    let port = &out_ports[port_index];
    let port_name = midi_out.port_name(port).unwrap_or_else(|_| "Unknown".to_string());
    println!("Using saved MIDI output port {}: {}", port_index, port_name);
    
    let connection = midi_out.connect(port, "MIDI Clock")?;
    Ok(connection)
}

/// Connect MIDI input by port index
fn connect_midi_input_by_index(generator: &ClockGenerator, port_index: usize) -> Result<midir::MidiInputConnection<()>, Box<dyn Error>> {
    use midir::MidiInput;
    
    let midi_in = MidiInput::new("MIDI Input")?;
    let in_ports = midi_in.ports();
    
    if port_index >= in_ports.len() {
        return Err(format!("Saved input port {} not available (only {} ports)", port_index, in_ports.len()).into());
    }
    
    let port = &in_ports[port_index];
    let port_name = midi_in.port_name(port).unwrap_or_else(|_| "Unknown".to_string());
    println!("Using saved MIDI input port {}: {}", port_index, port_name);
    
    let connection = midi_in.connect(
        port,
        "MIDI Input",
        |_timestamp, _message, _| {
            // No-op: for testing we just need the connection open
        },
        (),
    )?;
    
    Ok(connection)
}