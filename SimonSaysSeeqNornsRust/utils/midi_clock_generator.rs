//! MIDI Clock Generator Utility
//! 
//! A standalone utility program that generates MIDI clock signals and sends them over USB.
//! This can be used to test sequencers and other MIDI devices that need external clock sync.

use std::io::{self, Write};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};
use midir::{MidiOutput, MidiOutputConnection};

#[derive(Debug, Clone)]
pub enum ClockCommand {
    Start,
    Stop,
    SetBpm(f32),
    Exit,
}

#[derive(Debug, Clone)]
pub struct ClockGenerator {
    bpm: Arc<Mutex<f32>>,
    is_running: Arc<AtomicBool>,
    should_exit: Arc<AtomicBool>,
    command_sender: Option<Sender<ClockCommand>>,
}

impl ClockGenerator {
    pub fn new(bpm: f32) -> Self {
        Self {
            bpm: Arc::new(Mutex::new(bpm.clamp(20.0, 300.0))),
            is_running: Arc::new(AtomicBool::new(false)),
            should_exit: Arc::new(AtomicBool::new(false)),
            command_sender: None,
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

        // Auto-select first port or let user choose
        let selected_port = if out_ports.len() == 1 {
            println!("Auto-selecting port 0");
            &out_ports[0]
        } else {
            print!("Select MIDI output port (0-{}): ", out_ports.len() - 1);
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let port_idx: usize = input.trim().parse().unwrap_or(0);
            
            if port_idx >= out_ports.len() {
                return Err("Invalid port selection".into());
            }
            
            &out_ports[port_idx]
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

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            let mut tick_count = 0u32;
            let mut beat_count = 0u32;

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
                        ClockCommand::Exit => {
                            should_exit.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                }
                if is_running.load(Ordering::Relaxed) {
                    let current_bpm = *bpm.lock().unwrap();
                    
                    // Calculate time between MIDI clock ticks
                    // MIDI clock sends 24 ticks per quarter note (24 PPQ)
                    let ticks_per_second = (current_bpm * 24.0) / 60.0;
                    let tick_interval = Duration::from_secs_f32(1.0 / ticks_per_second);

                    let now = Instant::now();
                    if now.duration_since(last_tick) >= tick_interval {
                        // Send MIDI clock tick
                        if let Err(e) = connection.send(&[0xF8]) {
                            eprintln!("Error sending MIDI clock: {}", e);
                            break;
                        }

                        tick_count = tick_count.wrapping_add(1);
                        last_tick = now;

                        // Print visual beat indicator every 24 ticks (1 beat at 24 PPQ)
                        if tick_count % 24 == 0 {
                            beat_count = beat_count.wrapping_add(1);
                            match beat_count % 4 {
                                1 => print!("♩"), // Beat 1 (downbeat)
                                2 => print!("♪"), // Beat 2
                                3 => print!("♫"), // Beat 3
                                0 => print!("♬"), // Beat 4
                                _ => print!("·"),
                            }
                            
                            // New line every 4 beats (1 measure)
                            if beat_count % 4 == 0 {
                                println!(" | {:.1} BPM", current_bpm);
                            }
                            
                            io::stdout().flush().ok();
                        }
                    }
                }

                // Small sleep to prevent busy waiting
                thread::sleep(Duration::from_millis(1));
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
    println!("  s, start      - Start/stop clock");
    println!("  +             - Increase BPM by 5");
    println!("  -             - Decrease BPM by 5");
    println!("  ++            - Increase BPM by 1");
    println!("  --            - Decrease BPM by 1");
    println!("  <number>      - Set specific BPM (e.g., '140')");
    println!("  status        - Show current status");
    println!("  q, quit, exit - Quit program");
    println!("  h, help       - Show this help");
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

    // Get initial BPM from user or use default
    print!("Enter BPM (default 120): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let bpm = input.trim().parse::<f32>().unwrap_or(120.0);

    let mut generator = ClockGenerator::new(bpm);
    
    // Connect to MIDI output
    let connection = generator.connect_midi_output()?;
    
    println!("✅ MIDI Clock Generator initialized at {:.1} BPM", generator.get_bpm());
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

    // Main command loop
    loop {
        print!("🎛 Command: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let command = input.trim().to_lowercase();
                if command.is_empty() {
                    continue;
                }

                match handle_command(&generator, &command) {
                    Ok(true) => break, // User wants to quit
                    Ok(false) => continue,
                    Err(e) => {
                        eprintln!("❌ Error: {}", e);
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Input error: {}", e);
                break;
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