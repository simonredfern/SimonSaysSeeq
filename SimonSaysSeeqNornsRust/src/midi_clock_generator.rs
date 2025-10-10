//! MIDI Clock Generator Utility
//! 
//! A standalone utility program that generates MIDI clock signals and sends them over USB.
//! This can be used to test sequencers and other MIDI devices that need external clock sync.

use std::io::{self, Write};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};
use std::fs;

use midir::{MidiOutput, MidiOutputConnection};
use clap::Parser;

use simon_says_seeq_rust::test_script::{TestScript, TestCommand};

#[derive(Debug, Clone)]
pub enum ClockCommand {
    Start,
    Stop,
    SetBpm(f32),
    ToggleTestMode,
    Exit,
}

#[derive(Debug, Clone)]
pub struct ClockGenerator {
    bpm: Arc<Mutex<f32>>,
    is_running: Arc<AtomicBool>,
    should_exit: Arc<AtomicBool>,
    command_sender: Option<Sender<ClockCommand>>,
    test_mode: Arc<AtomicBool>,
}

impl ClockGenerator {
    pub fn new(bpm: f32) -> Self {
        Self {
            bpm: Arc::new(Mutex::new(bpm.clamp(20.0, 300.0))),
            is_running: Arc::new(AtomicBool::new(false)),
            should_exit: Arc::new(AtomicBool::new(false)),
            command_sender: None,
            test_mode: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Initialize MIDI output connection
    pub fn connect_midi_output(&self) -> Result<MidiOutputConnection, Box<dyn std::error::Error>> {
        let midi_out = MidiOutput::new("MIDI Clock Generator")?;
        let out_ports = midi_out.ports();

        if out_ports.is_empty() {
            return Err("No MIDI output ports available".into());
        }

        // List available ports
        println!("Available MIDI output ports:");
        for (i, port) in out_ports.iter().enumerate() {
            if let Ok(port_name) = midi_out.port_name(port) {
                println!("  {}: {}", i, port_name);
            }
        }

        // Port selection with default to 0
        let selected_port = if out_ports.len() == 1 {
            println!("Auto-selecting port 0");
            &out_ports[0]
        } else {
            print!("Select MIDI output port (default 0): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let port_idx: usize = if input.trim().is_empty() {
                0 // Default to port 0
            } else {
                input.trim().parse().unwrap_or(0)
            };
            
            if port_idx >= out_ports.len() {
                println!("Invalid port selection, defaulting to port 0");
                &out_ports[0]
            } else {
                &out_ports[port_idx]
            }
        };

        let port_name = midi_out.port_name(selected_port)?;
        println!("Connecting to MIDI output: {}", port_name);

        let connection = midi_out.connect(selected_port, "Clock Generator Output")?;
        Ok(connection)
    }

    /// Start the MIDI clock
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref sender) = self.command_sender {
            sender.send(ClockCommand::Start)?;
        }
        Ok(())
    }

    /// Stop the MIDI clock
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref sender) = self.command_sender {
            sender.send(ClockCommand::Stop)?;
        }
        Ok(())
    }

    /// Set new BPM
    pub fn set_bpm(&self, bpm: f32) {
        let clamped_bpm = bpm.clamp(20.0, 300.0);
        *self.bpm.lock().unwrap() = clamped_bpm;
        if let Some(ref sender) = self.command_sender {
            let _ = sender.send(ClockCommand::SetBpm(clamped_bpm));
        }
        println!("BPM set to: {:.1}", clamped_bpm);
    }

    /// Get current BPM
    pub fn get_bpm(&self) -> f32 {
        *self.bpm.lock().unwrap()
    }

    /// Signal that the generator should exit
    pub fn exit(&self) {
        self.should_exit.store(true, Ordering::Relaxed);
        if let Some(ref sender) = self.command_sender {
            let _ = sender.send(ClockCommand::Exit);
        }
    }

    /// Run the clock generation loop in a separate thread
    pub fn spawn_clock_thread(&mut self, mut connection: MidiOutputConnection) -> thread::JoinHandle<()> {
        let (sender, receiver) = mpsc::channel();
        self.command_sender = Some(sender);
        
        let bpm = self.bpm.clone();
        let is_running = self.is_running.clone();
        let should_exit = self.should_exit.clone();
        let test_mode = self.test_mode.clone();

        thread::spawn(move || {
            let mut tick_count = 0u32;
            let mut start_time = Instant::now();
            let mut last_bpm = *bpm.lock().unwrap();
            let mut test_start_time = Instant::now();

            println!("Clock generation thread started");

            while !should_exit.load(Ordering::Relaxed) {
                // Handle commands from main thread
                if let Ok(command) = receiver.try_recv() {
                    match command {
                        ClockCommand::Start => {
                            is_running.store(true, Ordering::Relaxed);
                            if let Err(e) = connection.send(&[0xFA]) {
                                eprintln!("Error sending MIDI Start: {}", e);
                            } else {
                                let current_bpm = *bpm.lock().unwrap();
                                println!("MIDI Clock Started at {:.1} BPM", current_bpm);
                            }
                        }
                        ClockCommand::Stop => {
                            is_running.store(false, Ordering::Relaxed);
                            if let Err(e) = connection.send(&[0xFC]) {
                                eprintln!("Error sending MIDI Stop: {}", e);
                            } else {
                                println!("MIDI Clock Stopped");
                            }
                        }
                        ClockCommand::SetBpm(_) => {
                            // BPM is already updated in the shared state
                        }
                        ClockCommand::ToggleTestMode => {
                            let new_test_mode = !test_mode.load(Ordering::Relaxed);
                            test_mode.store(new_test_mode, Ordering::Relaxed);
                            if new_test_mode {
                                println!("Test mode enabled: Stepped tempo changes with stop/start cycles (120->125->121->140->130->122->110, 120s cycle)");
                                println!("Auto-starting clock for test mode");
                                is_running.store(true, Ordering::Relaxed);
                                test_start_time = Instant::now();
                            } else {
                                println!("Test mode disabled");
                            }
                        }
                        ClockCommand::Exit => {
                            should_exit.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                }
                if is_running.load(Ordering::Relaxed) {
                    let mut current_bpm = *bpm.lock().unwrap();
                    
                    // Reset timing reference if BPM changed to prevent timing disruption
                    if (current_bpm - last_bpm).abs() > 0.1 {
                        start_time = Instant::now();
                        tick_count = 0;
                        last_bpm = current_bpm;
                    }
                    
                    // Test mode: discrete tempo changes with stop/start cycles
                    if test_mode.load(Ordering::Relaxed) {
                        let elapsed_secs = test_start_time.elapsed().as_secs_f32();
                        
                        // Extended test sequence with stop/start cycles
                        // Total cycle: 10+10+10+10+20+20+20+10+10 = 120 seconds
                        let cycle_duration = 120.0;
                        let cycle_position = elapsed_secs % cycle_duration;
                        
                        // Check if we should be stopped (stop for 10s at end of tempo sequence, then restart for 10s)
                        let should_run = if cycle_position >= 100.0 && cycle_position < 110.0 {
                            // 100-110s: Stop clock for 10 seconds
                            if is_running.load(Ordering::Relaxed) {
                                is_running.store(false, Ordering::Relaxed);
                                println!("Test mode: Stopping clock for 10 seconds");
                            }
                            false
                        } else if cycle_position >= 110.0 && cycle_position < 120.0 {
                            // 110-120s: Restart clock for 10 seconds before next cycle
                            if !is_running.load(Ordering::Relaxed) {
                                is_running.store(true, Ordering::Relaxed);
                                println!("Test mode: Restarting clock for next cycle");
                            }
                            true
                        } else {
                            true
                        };
                        
                        if should_run {
                            current_bpm = if cycle_position < 10.0 {
                                120.0  // 0-10s: 120 BPM
                            } else if cycle_position < 20.0 {
                                125.0  // 10-20s: 125 BPM
                            } else if cycle_position < 30.0 {
                                121.0  // 20-30s: 121 BPM
                            } else if cycle_position < 40.0 {
                                140.0  // 30-40s: 140 BPM
                            } else if cycle_position < 60.0 {
                                130.0  // 40-60s: 130 BPM
                            } else if cycle_position < 80.0 {
                                122.0  // 60-80s: 122 BPM
                            } else if cycle_position < 100.0 {
                                110.0  // 80-100s: 110 BPM
                            } else {
                                current_bpm  // During stop/restart period, maintain current BPM
                            };
                            
                            // Update the shared BPM state so it's visible in status
                            *bpm.lock().unwrap() = current_bpm;
                        }
                    }
                    
                    // Drift-corrected timing: calculate absolute target time for each tick
                    // MIDI clock sends 24 ticks per quarter note (24 PPQ)
                    let ticks_per_second = (current_bpm * 24.0) / 60.0;
                    let tick_interval_secs = 1.0 / ticks_per_second;
                    
                    // Calculate when this tick should occur (absolute time)
                    let target_time = start_time + Duration::from_secs_f32(tick_count as f32 * tick_interval_secs);
                    let now = Instant::now();
                    
                    if now >= target_time {
                        // Send MIDI clock tick
                        if let Err(e) = connection.send(&[0xF8]) {
                            eprintln!("Error sending MIDI clock: {}", e);
                            break;
                        }

                        tick_count = tick_count.wrapping_add(1);
                    }
                }

                // Adaptive sleep based on time until next tick
                let current_bpm = *bpm.lock().unwrap();
                let ticks_per_second = (current_bpm * 24.0) / 60.0;
                let tick_interval_secs = 1.0 / ticks_per_second;
                let next_tick_time = start_time + Duration::from_secs_f32((tick_count + 1) as f32 * tick_interval_secs);
                let now = Instant::now();
                
                if next_tick_time > now {
                    let sleep_duration = next_tick_time.duration_since(now);
                    // Sleep for most of the time, but wake up slightly early to avoid overshooting
                    let sleep_ms = (sleep_duration.as_millis() as f32 * 0.8).max(0.1) as u64;
                    thread::sleep(Duration::from_millis(sleep_ms.min(10))); // Cap at 10ms
                } else {
                    // We're behind, minimal sleep to yield CPU
                    thread::sleep(Duration::from_micros(100));
                }
            }

            // Send stop message when exiting
            if is_running.load(Ordering::Relaxed) {
                let _ = connection.send(&[0xFC]); // MIDI Stop
                println!("\nClock stopped on exit");
            }

            println!("Clock generation thread ended");
        })
    }
}

/// Display help information
fn show_help() {
    println!("Commands:");
    println!("  s       - Start/stop clock");
    println!("  u       - Increase BPM by 1 (uu=+10, uuu=+50)");
    println!("  d       - Decrease BPM by 1 (dd=-10, ddd=-50)");
    println!("  t       - Toggle test mode (stepped tempo + stop/start: 120->125->121->140->130->122->110)");
    println!("  q       - Quit program");
    println!("  h       - Show this help");
}

/// Handle user input commands
fn handle_command(generator: &ClockGenerator, command: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match command {
        "q" | "quit" | "exit" => {
            println!("Exiting...");
            return Ok(true);
        }
        "s" | "start" => {
            if generator.is_running.load(Ordering::Relaxed) {
                generator.stop()?;
            } else {
                generator.start()?;
            }
        }

        "status" => {
            let status = if generator.is_running.load(Ordering::Relaxed) {
                "Running"
            } else {
                "Stopped"
            };
            println!("Status: {} | BPM: {:.1}", status, generator.get_bpm());
        }
        "h" | "help" => {
            show_help();
        }
        cmd => {
            // Check for "u" pattern (increase BPM)
            if cmd.chars().all(|c| c == 'u') && !cmd.is_empty() && cmd.len() <= 3 {
                let increment = match cmd.len() {
                    1 => 1.0,    // u = +1
                    2 => 10.0,   // uu = +10
                    3 => 50.0,   // uuu = +50
                    _ => 1.0,
                };
                let new_bpm = generator.get_bpm() + increment;
                generator.set_bpm(new_bpm);
                println!("BPM increased by {:.0} to {:.1}", increment, new_bpm);
            }
            // Check for "d" pattern (decrease BPM)
            else if cmd.chars().all(|c| c == 'd') && !cmd.is_empty() && cmd.len() <= 3 {
                let decrement = match cmd.len() {
                    1 => 1.0,    // d = -1
                    2 => 10.0,   // dd = -10
                    3 => 50.0,   // ddd = -50
                    _ => 1.0,
                };
                let new_bpm = generator.get_bpm() - decrement;
                generator.set_bpm(new_bpm);
                println!("BPM decreased by {:.0} to {:.1}", decrement, new_bpm);
            }
            // Try to parse as BPM value
            else if let Ok(bpm) = cmd.parse::<f32>() {
                generator.set_bpm(bpm);
            } else {
                println!("Unknown command: '{}'. Type 'h' for help.", cmd);
            }
        }
    }

    Ok(false)
}

/// Execute a test script with the clock generator
fn execute_test_script(generator: &mut ClockGenerator, script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 Loading test script: {}", script_path);
    
    // Load the test script
    let content = fs::read_to_string(script_path)?;
    let script: TestScript = serde_json::from_str(&content)?;
    
    println!("📋 Test: {}", script.name);
    println!("📝 Description: {}", script.description);
    println!("═══════════════════════════════════════════════════════════════════");
    
    // Set initial BPM if specified
    if let Some(bpm) = script.initial_bpm {
        generator.set_bpm(bpm);
        println!("🎚️  Initial BPM set to {:.1}", bpm);
    }
    
    // Execute each command in sequence
    for command in script.commands.iter() {
        match command {
            TestCommand::Wait { ms } => {
                println!("⏱️  Waiting {} ms...", ms);
                thread::sleep(Duration::from_millis(*ms));
            }
            
            TestCommand::Start => {
                generator.start()?;
                println!("▶️  MIDI clock started");
            }
            
            TestCommand::Stop => {
                generator.stop()?;
                println!("⏹️  MIDI clock stopped");
            }
            
            TestCommand::SetBpm { bpm } => {
                generator.set_bpm(*bpm);
                println!("🎚️  BPM set to {:.1}", bpm);
            }
            
            TestCommand::LoadPattern { file } => {
                println!("📂 Loading test pattern: {}", file);
                let test_pattern_content = fs::read_to_string(file)?;
                fs::write("current_pattern.json", test_pattern_content)?;
                println!("✅ Pattern loaded - sequencer will use it on next start");
            }
            
            TestCommand::ButtonPress { file, grid_id, x, y } => {
                println!("🔘 Button press: {} at ({}, {}) via {}", grid_id, x, y, file);
                // Note: Button injection not supported in MIDI clock generator
                println!("⚠️  Button injection only works with automated_test_clock");
            }
            
            TestCommand::ButtonRelease { file, grid_id, x, y } => {
                println!("🔘 Button release: {} at ({}, {}) via {}", grid_id, x, y, file);
                println!("⚠️  Button injection only works with automated_test_clock");
            }
            
            TestCommand::SimpleButton { file, x, y } => {
                println!("🔘 Simple button: ({}, {}) via {}", x, y, file);
                println!("⚠️  Button injection only works with automated_test_clock");
            }
            
            TestCommand::ArmAction { arm_column, target_row, target_column } => {
                println!("🎯 ARM action: arm={}, target=({},{})", arm_column, target_row, target_column);
                println!("⚠️  Button injection only works with automated_test_clock");
            }
            
            TestCommand::WaitSteps { count } => {
                println!("⏳ Waiting for {} steps...", count);
                // Calculate approximate time based on BPM
                let bpm = generator.get_bpm();
                let ms_per_step = (60_000.0 / bpm) / 4.0; // 4 steps per beat at 24 PPQ
                let total_ms = ms_per_step * (*count as f32);
                thread::sleep(Duration::from_millis(total_ms as u64));
            }
            
            TestCommand::LogMilestone { message } => {
                println!("🏁 Milestone: {}", message);
            }
            
            TestCommand::VerifyState { description } => {
                println!("🔍 Verify: {} (placeholder - not implemented)", description);
            }
        }
        
        // Small delay between commands
        thread::sleep(Duration::from_millis(50));
    }
    
    println!("\n✅ Test script completed successfully!");
    println!("Press Ctrl+C to exit");
    
    // Keep the clock running until user exits
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(name = "MIDI Clock Generator")]
#[command(about = "Generate MIDI clock signals for testing sequencers", long_about = None)]
struct Args {
    /// Optional test script file to execute
    #[arg(short, long)]
    script: Option<String>,
    
    /// Initial BPM (default: 120.0)
    #[arg(short, long, default_value = "120.0")]
    bpm: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("MIDI Clock Generator");
    println!("======================");

    // Start with BPM from args
    let mut generator = ClockGenerator::new(args.bpm);
    
    // Connect to MIDI output
    let connection = generator.connect_midi_output()?;
    
    println!("MIDI Clock Generator initialized at {:.1} BPM", generator.get_bpm());
    
    // Check if we should execute a test script
    if let Some(script_path) = args.script {
        return execute_test_script(&mut generator, &script_path);
    }
    
    show_help();
    println!();

    // Handle Ctrl+C gracefully
    let generator_for_signal = generator.clone();
    ctrlc::set_handler(move || {
        println!("\n⚡ Received Ctrl+C, stopping...");
        generator_for_signal.exit();
    })?;

    // Start the clock generation thread
    let clock_thread = generator.spawn_clock_thread(connection);
    
    // Auto-start the clock (must be after spawn_clock_thread so command_sender exists)
    println!("Auto-starting clock");
    generator.start()?;

    println!("Enter commands: s(start/stop), u/uu/uuu(+1/+10/+50 BPM), d/dd/ddd(-1/-10/-50 BPM), t(test), q(quit), h(help)");
    
    // Simple line-based input loop
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        match input {
            "s" | "S" => {
                if generator.is_running.load(Ordering::Relaxed) {
                    generator.stop()?;
                    println!("Stopped");
                } else {
                    generator.start()?;
                    println!("Started");
                }
            }

            "t" | "T" => {
                if let Some(ref sender) = generator.command_sender {
                    let _ = sender.send(ClockCommand::ToggleTestMode);
                }
            }
            "q" | "Q" => {
                println!("Exiting...");
                break;
            }
            "h" | "H" => {
                show_help();
            }
            _ => {
                // Check for "u" pattern (increase BPM)
                if input.chars().all(|c| c == 'u') && !input.is_empty() && input.len() <= 3 {
                    let increment = match input.len() {
                        1 => 1.0,    // u = +1
                        2 => 10.0,   // uu = +10
                        3 => 50.0,   // uuu = +50
                        _ => 1.0,
                    };
                    let new_bpm = generator.get_bpm() + increment;
                    generator.set_bpm(new_bpm);
                    println!("BPM: {:.1} (+{})", new_bpm, increment);
                }
                // Check for "d" pattern (decrease BPM)
                else if input.chars().all(|c| c == 'd') && !input.is_empty() && input.len() <= 3 {
                    let decrement = match input.len() {
                        1 => 1.0,    // d = -1
                        2 => 10.0,   // dd = -10
                        3 => 50.0,   // ddd = -50
                        _ => 1.0,
                    };
                    let new_bpm = generator.get_bpm() - decrement;
                    generator.set_bpm(new_bpm);
                    println!("BPM: {:.1} (-{})", new_bpm, decrement);
                }
                // Try to parse as BPM value
                else if let Ok(bpm) = input.parse::<f32>() {
                    generator.set_bpm(bpm);
                    println!("BPM: {:.1}", generator.get_bpm());
                }
                else if !input.trim().is_empty() {
                    println!("Unknown command. Type 'h' for help.");
                }
            }
        }
    }

    // Clean shutdown
    generator.exit();

    // Wait for clock thread to finish
    println!("Waiting for clock thread to finish...");
    clock_thread.join().unwrap();

    println!("MIDI Clock Generator stopped.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpm_clamping() {
        let gen = ClockGenerator::new(500.0);
        assert_eq!(gen.get_bpm(), 300.0);
        
        gen.set_bpm(10.0);
        assert_eq!(gen.get_bpm(), 20.0);
        
        gen.set_bpm(150.0);
        assert_eq!(gen.get_bpm(), 150.0);
    }

    #[test]
    fn test_initial_state() {
        let gen = ClockGenerator::new(120.0);
        assert!(!gen.is_running.load(Ordering::Relaxed));
        assert!(!gen.should_exit.load(Ordering::Relaxed));
        assert_eq!(gen.get_bpm(), 120.0);
    }
}