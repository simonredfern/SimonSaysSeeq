//! MIDI Clock Generator Utility
//! 
//! A standalone utility program that generates MIDI clock signals and sends them over USB.
//! This can be used to test sequencers and other MIDI devices that need external clock sync.

use std::io::{self, Write};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;
use midir::{MidiOutput, MidiOutputConnection};

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
        println!("♩ BPM set to: {:.1}", clamped_bpm);
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
            let mut beat_count = 0u32;
            let mut last_measure_time: Option<chrono::DateTime<chrono::Utc>> = None;
            let start_time = Instant::now();
            let mut test_start_time = Instant::now();

            println!("🎵 Clock generation thread started");

            while !should_exit.load(Ordering::Relaxed) {
                // Handle commands from main thread
                if let Ok(command) = receiver.try_recv() {
                    match command {
                        ClockCommand::Start => {
                            is_running.store(true, Ordering::Relaxed);
                            if let Err(e) = connection.send(&[0xFA]) {
                                eprintln!("Error sending MIDI Start: {}", e);
                            } else {
                                println!("♪ MIDI Clock Started");
                            }
                        }
                        ClockCommand::Stop => {
                            is_running.store(false, Ordering::Relaxed);
                            if let Err(e) = connection.send(&[0xFC]) {
                                eprintln!("Error sending MIDI Stop: {}", e);
                            } else {
                                println!("⏹ MIDI Clock Stopped");
                            }
                        }
                        ClockCommand::SetBpm(_) => {
                            // BPM is already updated in the shared state
                        }
                        ClockCommand::ToggleTestMode => {
                            let new_test_mode = !test_mode.load(Ordering::Relaxed);
                            test_mode.store(new_test_mode, Ordering::Relaxed);
                            if new_test_mode {
                                println!("🧪 Test mode enabled: Stepped tempo changes with stop/start cycles (120→125→121→140→130→122→110, 120s cycle)");
                                println!("🧪 Auto-starting clock for test mode");
                                is_running.store(true, Ordering::Relaxed);
                                test_start_time = Instant::now();
                            } else {
                                println!("🧪 Test mode disabled");
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
                                println!("🧪 Test mode: Stopping clock for 10 seconds");
                            }
                            false
                        } else if cycle_position >= 110.0 && cycle_position < 120.0 {
                            // 110-120s: Restart clock for 10 seconds before next cycle
                            if !is_running.load(Ordering::Relaxed) {
                                is_running.store(true, Ordering::Relaxed);
                                println!("🧪 Test mode: Restarting clock for next cycle");
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

                        // Print visual beat indicator every 24 ticks (1 beat at 24 PPQ)
                        if tick_count % 24 == 0 {
                            beat_count = beat_count.wrapping_add(1);
                            match beat_count % 4 {
                                1 => print!("\r1"), // Beat 1 (downbeat)
                                2 => print!("2"), // Beat 2
                                3 => print!("3"), // Beat 3
                                0 => print!("4"), // Beat 4
                                _ => print!("·"),
                            }
                            
                            // New line every 4 beats (1 measure)
                            if beat_count % 4 == 0 {
                                let now = chrono::Utc::now();
                                let measure_time = now;
                                
                                // Calculate actual timing accuracy with drift correction
                                if beat_count >= 8 {
                                    let expected_measure_duration = 240.0 / current_bpm; // 4 beats in seconds
                                    let actual_duration = measure_time.signed_duration_since(last_measure_time.unwrap_or(measure_time)).num_milliseconds() as f32 / 1000.0;
                                    let timing_error = actual_duration - expected_measure_duration;
                                    
                                    // Calculate drift correction effectiveness
                                    let ticks_per_second_calc = (current_bpm * 24.0) / 60.0;
                                    let tick_interval_secs_calc = 1.0 / ticks_per_second_calc;
                                    let expected_tick_time = start_time + Duration::from_secs_f32(tick_count as f32 * tick_interval_secs_calc);
                                    let actual_now = Instant::now();
                                    let drift_correction = if actual_now > expected_tick_time {
                                        actual_now.duration_since(expected_tick_time).as_millis() as f32 / 1000.0
                                    } else {
                                        -(expected_tick_time.duration_since(actual_now).as_millis() as f32 / 1000.0)
                                    };
                                    
                                    let test_suffix = if test_mode.load(Ordering::Relaxed) { " [TEST MODE]" } else { "" };
                                    println!(" | {} {:.1} BPM{} (timing: expected {:.3}s, actual {:.3}s, error {:.3}s, drift {:.3}s)", 
                                             now.format("%Y-%m-%dT%H:%M:%S%.3fZ"), current_bpm, test_suffix,
                                             expected_measure_duration, actual_duration, timing_error, drift_correction);
                                    last_measure_time = Some(measure_time);
                                } else {
                                    let test_suffix = if test_mode.load(Ordering::Relaxed) { " [TEST MODE]" } else { "" };
                                    println!(" | {} {:.1} BPM{}", now.format("%Y-%m-%dT%H:%M:%S%.3fZ"), current_bpm, test_suffix);
                                    last_measure_time = Some(measure_time);
                                }
                            }
                            
                            io::stdout().flush().ok();
                        }
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
                println!("\n⏹ Clock stopped on exit");
            }

            println!("🎵 Clock generation thread ended");
        })
    }
}

/// Display help information
fn show_help() {
    println!("Commands:");
    println!("  s             - Start/stop clock");
    println!("  ↑ (Up Arrow) - Increase BPM by 1");
    println!("  ↓ (Down Arrow) - Decrease BPM by 1");
    println!("  → (Right Arrow) - Increase BPM by 5");
    println!("  ← (Left Arrow) - Decrease BPM by 5");
    println!("  t             - Toggle test mode (stepped tempo + stop/start: 120→125→121→140→130→122→110)");
    println!("  q             - Quit program");
    println!("  h             - Show this help");
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
        "+" => {
            let new_bpm = generator.get_bpm() + 5.0;
            generator.set_bpm(new_bpm);
        }
        "-" => {
            let new_bpm = generator.get_bpm() - 5.0;
            generator.set_bpm(new_bpm);
        }
        "++" => {
            let new_bpm = generator.get_bpm() + 1.0;
            generator.set_bpm(new_bpm);
        }
        "--" => {
            let new_bpm = generator.get_bpm() - 1.0;
            generator.set_bpm(new_bpm);
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
            // Try to parse as BPM value
            if let Ok(bpm) = cmd.parse::<f32>() {
                generator.set_bpm(bpm);
            } else {
                println!("Unknown command: '{}'. Type 'h' for help.", cmd);
            }
        }
    }

    Ok(false)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 MIDI Clock Generator");
    println!("======================");

    // Start with default 120 BPM - no user input needed
    let mut generator = ClockGenerator::new(120.0);
    
    // Connect to MIDI output
    let connection = generator.connect_midi_output()?;
    
    println!("✅ MIDI Clock Generator initialized at {:.1} BPM", generator.get_bpm());
    println!("🚀 Auto-starting clock");
    generator.start()?;
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

    // Enable raw mode for arrow key detection
    let _stdout = io::stdout().into_raw_mode()?;
    let stdin = io::stdin();
    
    println!("🎛 Use arrow keys for BPM, 's' start/stop, 't' test mode, 'q' quit\r");
    
    // Main input loop with arrow key support
    for key in stdin.keys() {
        match key? {
            Key::Up => {
                let new_bpm = generator.get_bpm() + 1.0;
                generator.set_bpm(new_bpm);
                println!("\r🎛 BPM: {:.1} (↑+1)", generator.get_bpm());
            }
            Key::Down => {
                let new_bpm = generator.get_bpm() - 1.0;
                generator.set_bpm(new_bpm);
                println!("\r🎛 BPM: {:.1} (↓-1)", generator.get_bpm());
            }
            Key::Right => {
                let new_bpm = generator.get_bpm() + 5.0;
                generator.set_bpm(new_bpm);
                println!("\r🎛 BPM: {:.1} (→+5)", generator.get_bpm());
            }
            Key::Left => {
                let new_bpm = generator.get_bpm() - 5.0;
                generator.set_bpm(new_bpm);
                println!("\r🎛 BPM: {:.1} (←-5)", generator.get_bpm());
            }
            Key::Char('s') | Key::Char('S') => {
                if generator.is_running.load(Ordering::Relaxed) {
                    generator.stop()?;
                    println!("\r🛑 Stopped");
                } else {
                    generator.start()?;
                    println!("\r▶️  Started");
                }
            }
            Key::Char('q') | Key::Char('Q') => {
                println!("\r\n👋 Exiting...");
                break;
            }
            Key::Char('t') | Key::Char('T') => {
                if let Some(ref sender) = generator.command_sender {
                    let _ = sender.send(ClockCommand::ToggleTestMode);
                }
            }
            Key::Char('h') | Key::Char('H') => {
                println!("\r\n");
                show_help();
                println!("🎛 Use arrow keys for BPM, 's' start/stop, 't' test mode, 'q' quit\r");
            }
            _ => {
                // Ignore other keys
            }
        }
    }

    // Clean shutdown
    generator.exit();

    // Wait for clock thread to finish
    println!("⏳ Waiting for clock thread to finish...");
    clock_thread.join().unwrap();

    println!("👋 MIDI Clock Generator stopped.");
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