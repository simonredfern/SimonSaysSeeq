//! Automated Test MIDI Clock Generator
//! 
//! An enhanced MIDI clock generator specifically designed for automated testing
//! of the SimonSaysSeeq sequencer. This tool integrates with the formal state
//! logging system and can execute automated test scenarios.
//!
//! Features:
//! - Generates MIDI clock signals at precise timing
//! - Reads test script files for automated control
//! - Coordinates with button injection system (button_a.txt, button_b.txt)
//! - Logs all MIDI clock events to formal_state.log
//! - Supports step-by-step sequencer advancement
//! - Enables reproducible test scenarios

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

use midir::{MidiOutput, MidiOutputConnection};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use clap::Parser;

/// Test script command types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "command")]
pub enum TestCommand {
    /// Wait for a specified number of milliseconds
    Wait { ms: u64 },
    /// Start MIDI clock
    Start,
    /// Stop MIDI clock
    Stop,
    /// Change BPM
    SetBpm { bpm: f32 },
    /// Send specific number of MIDI clock ticks
    SendTicks { count: u32 },
    /// Load a test pattern file into the sequencer
    LoadPattern { file: String },
    /// Inject button press via button file
    ButtonPress { file: String, grid_id: String, x: usize, y: usize },
    /// Inject button release via button file
    ButtonRelease { file: String, grid_id: String, x: usize, y: usize },
    /// Simple button press (just coordinates, assumes grid_one press)
    SimpleButton { file: String, x: usize, y: usize },
    /// ARM action sequence (press ARM button, then target)
    ArmAction { arm_column: usize, target_row: usize, target_column: usize },
    /// Wait for sequencer to complete N steps
    WaitSteps { count: u32 },
    /// Log a test milestone
    LogMilestone { message: String },
    /// Verify expected state (placeholder for future implementation)
    VerifyState { description: String },
}

/// Test script structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestScript {
    pub name: String,
    pub description: String,
    pub initial_bpm: Option<f32>,
    pub commands: Vec<TestCommand>,
}

/// Clock command for internal communication
#[derive(Debug, Clone)]
pub enum ClockCommand {
    Start,
    Stop,
    SetBpm(f32),
    SendTicks(u32),
    Exit,
}

/// Test event logging structure
#[derive(Debug, Clone, Serialize)]
pub struct TestClockEvent {
    pub timestamp: chrono::DateTime<Utc>,
    pub event_type: String,
    pub bpm: Option<f32>,
    pub tick_count: Option<u32>,
    pub message: Option<String>,
}

/// Main automated test clock generator
pub struct AutomatedTestClock {
    bpm: Arc<Mutex<f32>>,
    is_running: Arc<AtomicBool>,
    should_exit: Arc<AtomicBool>,
    command_sender: Option<Sender<ClockCommand>>,
    tick_count: Arc<Mutex<u32>>,
    start_time: Arc<Mutex<Option<Instant>>>,
}

impl AutomatedTestClock {
    pub fn new(bpm: f32) -> Self {
        Self {
            bpm: Arc::new(Mutex::new(bpm.clamp(20.0, 300.0))),
            is_running: Arc::new(AtomicBool::new(false)),
            should_exit: Arc::new(AtomicBool::new(false)),
            command_sender: None,
            tick_count: Arc::new(Mutex::new(0)),
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    /// Connect to MIDI output
    pub fn connect_midi_output(&self) -> Result<MidiOutputConnection, Box<dyn std::error::Error>> {
        let midi_out = MidiOutput::new("Automated Test Clock")?;
        let out_ports = midi_out.ports();

        if out_ports.is_empty() {
            return Err("No MIDI output ports available".into());
        }

        // Try to find a port that looks like it might be the target
        let mut selected_port = 0;
        for (i, port) in out_ports.iter().enumerate() {
            let port_name = midi_out.port_name(port).unwrap_or_default();
            println!("MIDI Output Port {}: {}", i, port_name);
            
            // Prefer ports that might be our target sequencer
            if port_name.to_lowercase().contains("usb") || 
               port_name.to_lowercase().contains("midi") ||
               port_name.to_lowercase().contains("norns") {
                selected_port = i;
            }
        }

        println!("Selected MIDI output port: {}", selected_port);
        let port = &out_ports[selected_port];
        let connection = midi_out.connect(port, "AutoTest")?;
        
        Ok(connection)
    }

    /// Start the clock
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref sender) = self.command_sender {
            sender.send(ClockCommand::Start)?;
        }
        Ok(())
    }

    /// Stop the clock
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref sender) = self.command_sender {
            sender.send(ClockCommand::Stop)?;
        }
        Ok(())
    }

    /// Set BPM
    pub fn set_bpm(&self, bpm: f32) -> Result<(), Box<dyn std::error::Error>> {
        let clamped_bpm = bpm.clamp(20.0, 300.0);
        *self.bpm.lock().unwrap() = clamped_bpm;
        if let Some(ref sender) = self.command_sender {
            sender.send(ClockCommand::SetBpm(clamped_bpm))?;
        }
        Ok(())
    }

    /// Get current BPM
    pub fn get_bpm(&self) -> f32 {
        *self.bpm.lock().unwrap()
    }

    /// Send specific number of MIDI clock ticks
    pub fn send_ticks(&self, count: u32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref sender) = self.command_sender {
            sender.send(ClockCommand::SendTicks(count))?;
        }
        Ok(())
    }

    /// Exit the clock generator
    pub fn exit(&self) {
        self.should_exit.store(true, Ordering::Relaxed);
        if let Some(ref sender) = self.command_sender {
            let _ = sender.send(ClockCommand::Exit);
        }
    }

    /// Get current tick count
    pub fn get_tick_count(&self) -> u32 {
        *self.tick_count.lock().unwrap()
    }

    /// Reset tick count
    pub fn reset_tick_count(&self) {
        *self.tick_count.lock().unwrap() = 0;
    }

    /// Log a test clock event
    fn log_event(&self, event_type: &str, message: Option<String>) {
        let event = TestClockEvent {
            timestamp: Utc::now(),
            event_type: event_type.to_string(),
            bpm: Some(self.get_bpm()),
            tick_count: Some(self.get_tick_count()),
            message,
        };

        // Write to test_clock.log 
        let log_file = "test_clock.log";

        if let Ok(json_line) = serde_json::to_string(&event) {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file) 
            {
                let _ = writeln!(file, "{}", json_line);
            }
        }
    }

    /// Spawn the clock generation thread
    pub fn spawn_clock_thread(&mut self, mut connection: MidiOutputConnection) -> thread::JoinHandle<()> {
        let (sender, receiver) = mpsc::channel();
        self.command_sender = Some(sender);
        
        let bpm = self.bpm.clone();
        let is_running = self.is_running.clone();
        let should_exit = self.should_exit.clone();
        let tick_count = self.tick_count.clone();
        let start_time = self.start_time.clone();

        thread::spawn(move || {
            println!("Automated test clock thread started");

            while !should_exit.load(Ordering::Relaxed) {
                // Handle commands from main thread
                if let Ok(command) = receiver.try_recv() {
                    match command {
                        ClockCommand::Start => {
                            is_running.store(true, Ordering::Relaxed);
                            *start_time.lock().unwrap() = Some(Instant::now());
                            *tick_count.lock().unwrap() = 0;
                            
                            if let Err(e) = connection.send(&[0xFA]) { // MIDI Start
                                eprintln!("Error sending MIDI Start: {}", e);
                            } else {
                                println!("🎵 MIDI Clock Started");
                            }
                        }
                        ClockCommand::Stop => {
                            is_running.store(false, Ordering::Relaxed);
                            if let Err(e) = connection.send(&[0xFC]) { // MIDI Stop
                                eprintln!("Error sending MIDI Stop: {}", e);
                            } else {
                                println!("⏹️  MIDI Clock Stopped");
                            }
                        }
                        ClockCommand::SetBpm(new_bpm) => {
                            println!("🎚️  BPM changed to {:.1}", new_bpm);
                        }
                        ClockCommand::SendTicks(count) => {
                            println!("⚡ Sending {} MIDI clock ticks", count);
                            for _ in 0..count {
                                if let Err(e) = connection.send(&[0xF8]) { // MIDI Clock
                                    eprintln!("Error sending MIDI Clock: {}", e);
                                    break;
                                }
                                *tick_count.lock().unwrap() += 1;
                                thread::sleep(Duration::from_millis(1)); // Small delay between ticks
                            }
                        }
                        ClockCommand::Exit => {
                            should_exit.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                }

                // Regular clock generation when running
                if is_running.load(Ordering::Relaxed) {
                    let current_bpm = *bpm.lock().unwrap();
                    
                    // Calculate tick interval: 24 PPQN (pulses per quarter note)
                    // At 120 BPM: 120 beats/min * 24 ticks/beat = 2880 ticks/min = 48 ticks/sec
                    let ticks_per_second = (current_bpm * 24.0) / 60.0;
                    let tick_interval = Duration::from_secs_f64((1.0 / ticks_per_second) as f64);
                    
                    if let Err(e) = connection.send(&[0xF8]) { // MIDI Clock
                        eprintln!("Error sending MIDI Clock: {}", e);
                    } else {
                        *tick_count.lock().unwrap() += 1;
                    }
                    
                    thread::sleep(tick_interval);
                } else {
                    // Not running, check less frequently
                    thread::sleep(Duration::from_millis(10));
                }
            }
            
            println!("Clock generation thread stopped");
        })
    }

    /// Execute a button injection command
    fn execute_button_command(&self, file: &str, grid_id: &str, x: usize, y: usize, is_press: bool) -> Result<(), Box<dyn std::error::Error>> {
        let action = if is_press { "press" } else { "release" };
        let content = format!("{},{},{},{}", action, grid_id, x, y);
        fs::write(file, &content)?;
        
        println!("📝 Injected: {} {} at ({},{}) via {}", action, grid_id, x, y, file);
        self.log_event("button_injection", Some(format!("{}: {}", file, content)));
        
        // Brief delay to allow processing
        thread::sleep(Duration::from_millis(50));
        
        // Reset file
        fs::write(file, "none")?;
        Ok(())
    }

    /// Execute ARM action (press ARM button, then target)
    fn execute_arm_action(&self, arm_column: usize, target_row: usize, target_column: usize) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 ARM Action: Column {} -> Row {} Column {}", arm_column, target_row, target_column);
        
        // Step 1: Press ARM button (row 7)
        self.execute_button_command("button_a.txt", "grid_one", arm_column, 7, true)?;
        thread::sleep(Duration::from_millis(100));
        
        // Step 2: Press target position
        let grid_id = if target_column < 16 { "grid_one" } else { "grid_two" };
        let adjusted_column = if target_column < 16 { target_column } else { target_column - 16 };
        self.execute_button_command("button_b.txt", grid_id, adjusted_column, target_row, true)?;
        thread::sleep(Duration::from_millis(100));
        
        self.log_event("arm_action", Some(format!("ARM{} -> R{}C{}", arm_column, target_row, target_column)));
        Ok(())
    }

    /// Wait for sequencer to advance a specified number of steps
    fn wait_for_steps(&self, step_count: u32) -> Result<(), Box<dyn std::error::Error>> {
        // Each step requires 6 MIDI clock ticks (24 PPQN ÷ 4 = 6 ticks per 16th note)
        let required_ticks = step_count * 6;
        let start_ticks = self.get_tick_count();
        let mut last_printed_tick = start_ticks;
        
        while (self.get_tick_count() - start_ticks) < required_ticks {
            let current_tick = self.get_tick_count();
            // Print a dot for each new tick
            while last_printed_tick < current_tick {
                print!(".");
                std::io::stdout().flush().ok();
                last_printed_tick += 1;
            }
            thread::sleep(Duration::from_millis(10));
        }
        
        println!("\n✅ Completed {} steps ({} ticks)", step_count, required_ticks);
        Ok(())
    }

    /// Execute a test script
    pub fn execute_test_script(&mut self, script: TestScript) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🧪 Executing Test Script: {}", script.name);
        println!("📋 Description: {}", script.description);
        println!("═══════════════════════════════════════════════════════════════════");
        
        self.log_event("test_script_start", Some(format!("{}: {}", script.name, script.description)));
        
        // Set initial BPM if specified
        if let Some(bpm) = script.initial_bpm {
            self.set_bpm(bpm)?;
            println!("🎚️  Initial BPM set to {:.1}", bpm);
        }
        
        // Execute each command in sequence
        for (i, command) in script.commands.iter().enumerate() {
            println!("\n{}. Executing: {:?}", i + 1, command);
            
            match command {
                TestCommand::Wait { ms } => {
                    println!("⏱️  Waiting {} ms...", ms);
                    thread::sleep(Duration::from_millis(*ms));
                }
                
                TestCommand::Start => {
                    self.start()?;
                    self.log_event("test_start_clock", None);
                }
                
                TestCommand::Stop => {
                    self.stop()?;
                    self.log_event("test_stop_clock", None);
                }
                
                TestCommand::SetBpm { bpm } => {
                    self.set_bpm(*bpm)?;
                    self.log_event("test_set_bpm", Some(format!("BPM: {}", bpm)));
                }
                
                TestCommand::SendTicks { count } => {
                    self.send_ticks(*count)?;
                    self.log_event("test_send_ticks", Some(format!("Count: {}", count)));
                }
                
                TestCommand::LoadPattern { file } => {
                    println!("📂 Loading test pattern: {}", file);
                    // Copy test pattern to current_pattern.json so sequencer loads it
                    let test_pattern_content = fs::read_to_string(file)?;
                    fs::write("current_pattern.json", test_pattern_content)?;
                    println!("✅ Pattern loaded - sequencer will use it on next start");
                    self.log_event("test_load_pattern", Some(format!("File: {}", file)));
                }
                
                TestCommand::ButtonPress { file, grid_id, x, y } => {
                    self.execute_button_command(file, grid_id, *x, *y, true)?;
                }
                
                TestCommand::ButtonRelease { file, grid_id, x, y } => {
                    self.execute_button_command(file, grid_id, *x, *y, false)?;
                }
                
                TestCommand::SimpleButton { file, x, y } => {
                    self.execute_button_command(file, "grid_one", *x, *y, true)?;
                }
                
                TestCommand::ArmAction { arm_column, target_row, target_column } => {
                    self.execute_arm_action(*arm_column, *target_row, *target_column)?;
                }
                
                TestCommand::WaitSteps { count } => {
                    self.wait_for_steps(*count)?;
                }
                
                TestCommand::LogMilestone { message } => {
                    println!("🏁 Milestone: {}", message);
                    self.log_event("test_milestone", Some(message.clone()));
                }
                
                TestCommand::VerifyState { description } => {
                    println!("🔍 Verify: {} (placeholder - not implemented)", description);
                    self.log_event("test_verify", Some(description.clone()));
                }
            }
            
            // Small delay between commands
            thread::sleep(Duration::from_millis(50));
        }
        
        self.log_event("test_script_complete", Some(script.name.clone()));
        println!("\n✅ Test script completed successfully!");
        Ok(())
    }

    /// Load test script from JSON file
    pub fn load_test_script(filename: &str) -> Result<TestScript, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(filename)?;
        let script: TestScript = serde_json::from_str(&content)?;
        Ok(script)
    }

    /// Create a sample test script for the 16-step MIDI issue
    pub fn create_16_step_midi_test_script() -> TestScript {
        TestScript {
            name: "16-Step MIDI Debug Test".to_string(),
            description: "Reproduces the 16-step MIDI silence issue with formal state logging".to_string(),
            initial_bpm: Some(120.0),
            commands: vec![
                TestCommand::LogMilestone { 
                    message: "Starting 16-step MIDI debug test".to_string() 
                },
                
                // Start the sequencer
                TestCommand::Start,
                TestCommand::Wait { ms: 500 },
                
                // Create test pattern (every 4th step)
                TestCommand::LogMilestone { 
                    message: "Creating test pattern at steps 0,4,8,12,16,20,24,28".to_string() 
                },
                TestCommand::SimpleButton { file: "button_a.txt".to_string(), x: 0, y: 0 },
                TestCommand::SimpleButton { file: "button_a.txt".to_string(), x: 4, y: 0 },
                TestCommand::SimpleButton { file: "button_a.txt".to_string(), x: 8, y: 0 },
                TestCommand::SimpleButton { file: "button_a.txt".to_string(), x: 12, y: 0 },
                TestCommand::SimpleButton { file: "button_b.txt".to_string(), x: 0, y: 0 }, // Step 16
                TestCommand::SimpleButton { file: "button_b.txt".to_string(), x: 4, y: 0 }, // Step 20
                TestCommand::SimpleButton { file: "button_b.txt".to_string(), x: 8, y: 0 }, // Step 24
                TestCommand::SimpleButton { file: "button_b.txt".to_string(), x: 12, y: 0 }, // Step 28
                
                // Let it run for a few cycles
                TestCommand::LogMilestone { 
                    message: "Running full 32-step pattern for observation".to_string() 
                },
                TestCommand::WaitSteps { count: 64 }, // 2 full 32-step cycles
                
                // Now use ARM SetSeqALength to change row 0 to 16 steps
                TestCommand::LogMilestone { 
                    message: "Setting row 0 to 16 steps using ARM SetSeqALength".to_string() 
                },
                TestCommand::ArmAction { 
                    arm_column: 8,      // SetSeqALength ARM button
                    target_row: 0,      // Row 0
                    target_column: 15   // Column 15 = 16 steps
                },
                
                TestCommand::Wait { ms: 1000 },
                
                // Now observe the behavior - MIDI should be different
                TestCommand::LogMilestone { 
                    message: "Observing 16-step behavior - MIDI should be silent for steps 16,20,24,28".to_string() 
                },
                TestCommand::WaitSteps { count: 64 }, // 2 full 32-step master cycles
                
                TestCommand::LogMilestone { 
                    message: "Test completed - check formal_state.log for detailed analysis".to_string() 
                },
                
                TestCommand::Stop,
            ],
        }
    }
}

/// CLI arguments structure
#[derive(Parser, Debug)]
#[command(name = "automated_test_clock")]
#[command(about = "Automated Test MIDI Clock Generator", long_about = None)]
struct Args {
    /// Run test1: Multi-length pattern test (32, 31, 30, 16, 15, 14 lengths, run 33 steps)
    #[arg(long)]
    test1: bool,

    /// Run test1-advance: Send MIDI clock to advance sequencer 33 steps (requires sequencer running)
    #[arg(long)]
    test1_advance: bool,

    /// Execute a test script from a JSON file
    #[arg(long, value_name = "FILE")]
    script: Option<String>,

    /// Create the sample 16-step MIDI test script
    #[arg(long)]
    create_test: bool,

    /// Set initial BPM
    #[arg(long, default_value = "120.0")]
    bpm: f32,
}

impl AutomatedTestClock {
    /// Create test1: Multi-length pattern test
    /// Tests patterns of various lengths (32, 31, 30, 16, 15, 14) and runs for 33 steps
    fn create_test1_script() -> TestScript {
        TestScript {
            name: "Test1: Multi-Length Pattern Test".to_string(),
            description: "Test patterns with lengths 32, 31, 30, 16, 15, 14 and verify step counters after 33 steps".to_string(),
            initial_bpm: Some(120.0),
            commands: vec![
                TestCommand::LogMilestone { 
                    message: "Starting Test1: Multi-Length Pattern Test".to_string() 
                },
                
                // Load pre-configured test pattern with row lengths: 32, 31, 30, 16, 15, 14
                TestCommand::LogMilestone { 
                    message: "Loading test_pattern_1.json with row lengths [32, 31, 30, 16, 15, 14]".to_string() 
                },
                TestCommand::LoadPattern { 
                    file: "test_pattern_1.json".to_string() 
                },
                
                TestCommand::LogMilestone { 
                    message: "Pattern loaded to current_pattern.json".to_string() 
                },
                
                // Start the MIDI clock
                TestCommand::LogMilestone { 
                    message: "Starting MIDI clock".to_string() 
                },
                TestCommand::Start,
                TestCommand::Wait { ms: 500 },
                
                // Advance 33 steps
                TestCommand::LogMilestone { 
                    message: "Advancing 33 steps to verify row counters".to_string() 
                },
                TestCommand::WaitSteps { count: 33 },
                
                TestCommand::LogMilestone { 
                    message: "Expected states after 33 steps:".to_string() 
                },
                TestCommand::LogMilestone { 
                    message: "  Row 0 (len=32): step 1 (33 % 32 = 1)".to_string() 
                },
                TestCommand::LogMilestone { 
                    message: "  Row 1 (len=31): step 2 (33 % 31 = 2)".to_string() 
                },
                TestCommand::LogMilestone { 
                    message: "  Row 2 (len=30): step 3 (33 % 30 = 3)".to_string() 
                },
                TestCommand::LogMilestone { 
                    message: "  Row 3 (len=16): step 1 (33 % 16 = 1)".to_string() 
                },
                TestCommand::LogMilestone { 
                    message: "  Row 4 (len=15): step 3 (33 % 15 = 3)".to_string() 
                },
                TestCommand::LogMilestone { 
                    message: "  Row 5 (len=14): step 5 (33 % 14 = 5)".to_string() 
                },
                
                // Stop the MIDI clock
                TestCommand::LogMilestone { 
                    message: "Test complete - stopping MIDI clock".to_string() 
                },
                TestCommand::Stop,
            ],
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🤖 Automated Test MIDI Clock Generator");
    println!("═══════════════════════════════════════════════════════");
    println!("Enhanced MIDI clock with formal state logging and test automation");
    println!();

    // Handle create-test flag
    if args.create_test {
        let script = AutomatedTestClock::create_16_step_midi_test_script();
        let filename = "16_step_midi_test.json";
        match serde_json::to_string_pretty(&script) {
            Ok(json) => {
                if fs::write(filename, json).is_ok() {
                    println!("✅ Created sample test script: {}", filename);
                    println!("Run with: --script {}", filename);
                } else {
                    eprintln!("❌ Error writing test script file");
                }
            }
            Err(e) => {
                eprintln!("❌ Error serializing test script: {}", e);
            }
        }
        return Ok(());
    }

    let mut clock = AutomatedTestClock::new(args.bpm);
    let connection = clock.connect_midi_output()?;
    
    println!("✅ MIDI Clock initialized at {:.1} BPM", clock.get_bpm());
    println!();

    // Handle Ctrl+C gracefully
    let should_exit_signal = clock.should_exit.clone();
    ctrlc::set_handler(move || {
        println!("\n⚡ Received Ctrl+C, stopping...");
        should_exit_signal.store(true, Ordering::Relaxed);
    })?;

    // Start the clock generation thread
    let clock_thread = clock.spawn_clock_thread(connection);

    // Execute based on flags
    if args.test1 {
        println!("🧪 Running Test1: Multi-Length Pattern Test");
        let script = AutomatedTestClock::create_test1_script();
        clock.execute_test_script(script)?;
    } else if let Some(script_file) = args.script {
        println!("📜 Loading test script: {}", script_file);
        match AutomatedTestClock::load_test_script(&script_file) {
            Ok(script) => {
                println!("🧪 Executing: {}", script.name);
                clock.execute_test_script(script)?;
            }
            Err(e) => {
                eprintln!("❌ Error loading test script: {}", e);
                return Err(e);
            }
        }
    } else {
        println!("ℹ️  No test specified. Available options:");
        println!("  --test1              Run multi-length pattern test");
        println!("  --script <file>      Execute test script from JSON file");
        println!("  --create-test        Create sample 16-step test script");
        println!("  --bpm <value>        Set initial BPM (default: 120.0)");
        println!();
        println!("Use --help for more information");
    }

    // Clean shutdown
    clock.exit();
    println!("Waiting for clock thread to finish...");
    clock_thread.join().unwrap();
    println!("🛑 Automated Test MIDI Clock stopped.");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_creation() {
        let clock = AutomatedTestClock::new(120.0);
        assert_eq!(clock.get_bpm(), 120.0);
        assert_eq!(clock.get_tick_count(), 0);
    }

    #[test]
    fn test_bpm_clamping() {
        let clock = AutomatedTestClock::new(500.0); // Too high
        assert_eq!(clock.get_bpm(), 300.0);
        
        let clock2 = AutomatedTestClock::new(10.0); // Too low  
        assert_eq!(clock2.get_bpm(), 20.0);
    }

    #[test]
    fn test_script_creation() {
        let script = AutomatedTestClock::create_16_step_midi_test_script();
        assert_eq!(script.name, "16-Step MIDI Debug Test");
        assert!(!script.commands.is_empty());
        assert_eq!(script.initial_bpm, Some(120.0));
    }
}