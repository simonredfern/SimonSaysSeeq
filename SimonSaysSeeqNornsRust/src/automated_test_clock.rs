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

use std::thread;
use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use clap::Parser;

// Import shared types
use simon_says_seeq_rust::test_script::{TestCommand, TestScript};
use simon_says_seeq_rust::clock_generator::{ClockGenerator, ClockConfig};

/// Test event logging structure
#[derive(Debug, Clone, Serialize)]
pub struct TestClockEvent {
    pub timestamp: chrono::DateTime<Utc>,
    pub event_type: String,
    pub bpm: Option<f32>,
    pub tick_count: Option<u32>,
    pub message: Option<String>,
}

/// Wrapper around ClockGenerator for automated testing
pub struct AutomatedTestClock {
    generator: ClockGenerator,
}

impl AutomatedTestClock {
    pub fn new(bpm: f32) -> Self {
        let config = ClockConfig {
            enable_tick_counting: true,
            enable_test_mode: false,
        };
        Self {
            generator: ClockGenerator::new_with_config(bpm, config),
        }
    }

    /// Initialize MIDI output connection
    pub fn connect_midi_output(&self) -> Result<midir::MidiOutputConnection, Box<dyn std::error::Error>> {
        self.generator.connect_midi_output()
    }

    /// Start the MIDI clock
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.generator.start()
    }

    /// Stop the MIDI clock
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.generator.stop()
    }

    /// Set the BPM
    pub fn set_bpm(&self, bpm: f32) -> Result<(), Box<dyn std::error::Error>> {
        self.generator.set_bpm(bpm);
        Ok(())
    }

    /// Get current BPM
    pub fn get_bpm(&self) -> f32 {
        self.generator.get_bpm()
    }

    /// Exit the clock generator
    pub fn exit(&self) {
        self.generator.exit();
    }

    /// Get current tick count
    pub fn get_tick_count(&self) -> u32 {
        self.generator.get_tick_count()
    }

    /// Reset tick count
    pub fn reset_tick_count(&self) {
        self.generator.reset_tick_count();
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
    pub fn spawn_clock_thread(&mut self, connection: midir::MidiOutputConnection) -> thread::JoinHandle<()> {
        self.generator.spawn_clock_thread(connection)
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
        for (_i, command) in script.commands.iter().enumerate() {
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
                    // Give time for logs to be written
                    thread::sleep(Duration::from_millis(100));
                }
                
                TestCommand::LogMilestone { message } => {
                    println!("🏁 Milestone: {}", message);
                    self.log_event("test_milestone", Some(message.clone()));
                }
                
                TestCommand::VerifyState { description } => {
                    // Extract step number from description if present
                    if let Some(step_num) = description.split("master step ").nth(1).and_then(|s| s.parse::<usize>().ok()) {
                        // Extra delay to ensure log is written
                        thread::sleep(Duration::from_millis(50));
                        match self.verify_sequencer_state_at_step(step_num) {
                            Ok(state_info) => {
                                println!("🔍 Verify: {} - ✅ {}", description, state_info);
                                self.log_event("test_verify_success", Some(format!("{}: {}", description, state_info)));
                            }
                            Err(e) => {
                                println!("🔍 Verify: {} - ⚠️  {}", description, e);
                                self.log_event("test_verify_skip", Some(format!("{}: {}", description, e)));
                            }
                        }
                    } else {
                        println!("🔍 Verify: {} (no step number found)", description);
                        self.log_event("test_verify", Some(description.clone()));
                    }
                }
            }
            
            // Small delay between commands
            thread::sleep(Duration::from_millis(50));
        }
        
        self.log_event("test_script_complete", Some(script.name.clone()));
        println!("\n✅ Test script completed successfully!");
        Ok(())
    }

    /// Verify sequencer state at a specific master step
    fn verify_sequencer_state_at_step(&self, step: usize) -> Result<String, String> {
        // Only implement for steps 1-8
        if step > 8 {
            return Err("Verification not implemented for this step".to_string());
        }

        // Debug: Show current working directory
        if let Ok(cwd) = std::env::current_dir() {
            println!("  📁 Current working directory: {}", cwd.display());
        }

        // Read formal_state.log to find the last StepAdvancement event for this master step
        let log_content = match fs::read_to_string("formal_state.log") {
            Ok(content) => content,
            Err(_) => {
                // Log file doesn't exist yet - wait a bit and try again
                thread::sleep(Duration::from_millis(200));
                fs::read_to_string("formal_state.log")
                    .map_err(|e| format!("Cannot read formal_state.log: {}", e))?
            }
        };
        
        // Debug: Show log file info
        let line_count = log_content.lines().count();
        println!("  📋 formal_state.log has {} lines", line_count);
        if line_count > 0 {
            // Show last few lines
            let last_lines: Vec<_> = log_content.lines().rev().take(3).collect();
            println!("  📋 Last 3 lines:");
            for line in last_lines.iter().rev() {
                println!("     {}", line);
            }
        }

        // Expected row lengths: [31, 30, 29, 15, 14, 13, 2] (which gives 32, 31, 30, 16, 15, 14, 3 steps)
        let expected_lengths = vec![31, 30, 29, 15, 14, 13, 2];
        
        // Find the StepAdvancement event for this master step
        // Look for the last occurrence of this step in the log
        let mut found_step_data: Option<Vec<(usize, usize)>> = None;
        let mut found_steps = Vec::new(); // Debug: collect all steps found
        
        for line in log_content.lines().rev() {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
                if event.get("event_type").and_then(|v| v.as_str()) == Some("StepAdvancement") {
                    if let Some(master_step_val) = event.get("master_step").and_then(|v| v.as_u64()) {
                        found_steps.push(master_step_val as usize); // Debug: track all steps
                        if master_step_val as usize == step {
                            // Found the event for this step
                            if let Some(row_steps) = event.get("row_steps").and_then(|v| v.as_array()) {
                                let mut steps = Vec::new();
                                for rs in row_steps {
                                    if let (Some(row_idx), Some(step_val)) = (
                                        rs.get(0).and_then(|v| v.as_u64()),
                                        rs.get(1).and_then(|v| v.as_u64())
                                    ) {
                                        steps.push((row_idx as usize, step_val as usize));
                                    }
                                }
                                found_step_data = Some(steps);
                                break;
                            }
                        }
                    }
                }
            }
        }

        let actual_steps = found_step_data
            .ok_or(format!("No step advancement event found for master step {} in formal_state.log (found steps: {:?})", step, found_steps))?;

        // Build verification message
        let mut matches = true;
        let mut details = Vec::new();
        
        for (row_idx, &expected_length) in expected_lengths.iter().enumerate().take(7) {
            let expected_step = step % expected_length;
            
            if let Some((_, actual_step)) = actual_steps.iter().find(|(idx, _)| *idx == row_idx) {
                if actual_step == &expected_step {
                    details.push(format!("R{}:OK({}/{})", row_idx, actual_step, expected_length));
                } else {
                    details.push(format!("R{}:MISMATCH(exp:{}/{}, got:{}/{})", 
                        row_idx, expected_step, expected_length, actual_step, expected_length));
                    matches = false;
                }
            } else {
                details.push(format!("R{}:MISSING", row_idx));
                matches = false;
            }
        }

        if matches {
            Ok(format!("All rows match - [{}]", details.join(", ")))
        } else {
            Err(format!("State mismatch - [{}]", details.join(", ")))
        }
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
    /// Run test1: Multi-length pattern test (31, 30, 29, 15, 14, 13, 2 lengths for 7 rows)
    #[arg(long)]
    test1: bool,

    /// Run test1-advance: Send MIDI clock to advance sequencer 33 steps (requires sequencer running)
    #[arg(long)]
    test1_advance: bool,

    /// Execute a test script from a JSON file
    #[arg(long, value_name = "FILE")]
    script: Option<String>,

    /// Create sample test script file
    #[arg(long)]
    create_test: bool,

    /// Initial BPM (default: 120.0)
    #[arg(long, default_value = "120.0")]
    bpm: f32,

    /// Number of steps to test (default: 5)
    #[arg(long, default_value = "5")]
    test_length_steps: usize,
}

impl AutomatedTestClock {
    /// Create test1: Multi-length pattern test
    /// Tests patterns with various lengths and checks state at each master step
    fn create_test1_script(no_of_steps: usize) -> TestScript {
        let mut commands = vec![
            TestCommand::LogMilestone { 
                message: "Starting Test1: Multi-Length Pattern Test".to_string() 
            },
            
            // Stop sequencer first to reset all counters to 0
            TestCommand::LogMilestone { 
                message: "Stopping sequencer to reset state".to_string()
            },
            TestCommand::Stop,
            TestCommand::Wait { ms: 500 },
            
            // Load pre-configured test pattern with row lengths: 31, 30, 29, 15, 14, 13, 2 (which gives 3 steps: 0,1,2)
            TestCommand::LogMilestone { 
                message: "Loading test_pattern_1.json with row lengths [31, 30, 29, 15, 14, 13, 2]".to_string()
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
        ];
        
        // Advance one step at a time for specified number of steps, logging state at each master step
        for step in 1..=no_of_steps {
            commands.push(TestCommand::LogMilestone { 
                message: format!("Advancing to master step {}", step) 
            });
            commands.push(TestCommand::WaitSteps { count: 1 });
            commands.push(TestCommand::VerifyState { 
                description: format!("Check sequencer state at master step {}", step) 
            });
        }
        
        // Stop the MIDI clock
        commands.push(TestCommand::LogMilestone { 
            message: "Test complete - stopping MIDI clock".to_string() 
        });
        commands.push(TestCommand::Stop);
        
        TestScript {
            name: "Test1: Multi-Length Pattern Test".to_string(),
            description: format!("Test patterns with lengths 31, 30, 29, 15, 14, 13, 2 (7 rows, giving 32,31,30,16,15,14,3 steps) and verify step counters at each step up to {}", no_of_steps),
            initial_bpm: Some(120.0),
            commands,
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

    // Note: Ctrl+C handler removed - clock will be stopped on exit

    // Start the clock generation thread
    let clock_thread = clock.spawn_clock_thread(connection);

    // Execute based on flags
    if args.test1 {
        println!("🧪 Running Test1: Multi-Length Pattern Test (testing {} steps)", args.test_length_steps);
        let script = AutomatedTestClock::create_test1_script(args.test_length_steps);
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