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

/// Row configuration change event
#[derive(Debug, Clone)]
struct RowConfigChange {
    cumulative_step: usize,
    row: usize,
    new_max_step: usize,
}

/// Wrapper around ClockGenerator for automated testing
pub struct AutomatedTestClock {
    generator: ClockGenerator,
    row_config_changes: Vec<RowConfigChange>,
    cumulative_steps: usize,
}

impl AutomatedTestClock {
    pub fn new(bpm: f32) -> Self {
        let config = ClockConfig {
            enable_tick_counting: true,
            enable_test_mode: false,
        };
        Self {
            generator: ClockGenerator::new_with_config(bpm, config),
            row_config_changes: Vec::new(),
            cumulative_steps: 0,
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

    /// Send SysEx command to reload pattern from file
    /// Format: F0 7D 53 53 51 01 F7
    /// - F0 = SysEx start
    /// - 7D = Educational/Development use (non-commercial)
    /// - 53 53 51 = "SSQ" in ASCII (SimonSaysSeeQ)
    /// - 01 = Command (reload pattern)
    /// - F7 = SysEx end
    fn send_reload_pattern_sysex(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sysex_message = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, 0x01, 0xF7];
        self.generator.send_raw_midi(&sysex_message)?;
        Ok(())
    }

    /// Send SysEx command to press/release a button
    /// Format: F0 7D 53 53 51 02 <row> <col> <press> F7
    /// - F0 = SysEx start
    /// - 7D = Educational/Development use (non-commercial)
    /// - 53 53 51 = "SSQ" in ASCII (SimonSaysSeeQ)
    /// - 02 = Command (button press/release)
    /// - row = Row number (0-7)
    /// - col = Column number (0-31)
    /// - press = 1 for press, 0 for release
    /// - F7 = SysEx end
    fn send_button_sysex(&self, row: u8, col: u8, press: bool) -> Result<(), Box<dyn std::error::Error>> {
        let press_byte = if press { 0x01 } else { 0x00 };
        let sysex_message = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, 0x02, row, col, press_byte, 0xF7];
        self.generator.send_raw_midi(&sysex_message)?;
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
        
        // Truncate formal_state.log at the start of the test to ensure clean state
        if let Err(e) = fs::write("formal_state.log", "") {
            println!("⚠️  Warning: Could not truncate formal_state.log: {}", e);
        } else {
            println!("🗑️  Truncated formal_state.log for clean test run");
        }
        
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
                    // Track cumulative steps for verification
                    self.cumulative_steps += *count as usize;
                    // Give time for logs to be written
                    thread::sleep(Duration::from_millis(100));
                }
                
                TestCommand::LogMilestone { message } => {
                    println!("🏁 Milestone: {}", message);
                    self.log_event("test_milestone", Some(message.clone()));
                }
                
                TestCommand::VerifyState { description } => {
                    // Extract cumulative step number from description if present
                    let step_num = description.split("cumulative step ")
                        .nth(1)
                        .and_then(|s| s.split_whitespace().next())
                        .and_then(|s| s.parse::<usize>().ok());
                    
                    if let Some(step_num) = step_num {
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
                
                TestCommand::ReloadPattern => {
                    println!("🔄 Sending SysEx reload pattern command...");
                    self.send_reload_pattern_sysex()?;
                    println!("✅ Reload pattern command sent");
                    self.log_event("test_reload_pattern", Some("SysEx command sent".to_string()));
                }
                
                TestCommand::SysExButton { row, col, press } => {
                    let action = if *press { "Press" } else { "Release" };
                    println!("🔘 Sending SysEx button {}: row={}, col={}", action, row, col);
                    self.send_button_sysex(*row, *col, *press)?;
                    println!("✅ Button {} command sent", action.to_lowercase());
                    self.log_event("test_sysex_button", Some(format!("{}:R{}C{}", action, row, col)));
                }
                
                TestCommand::RecordRowConfig { row, max_step } => {
                    // Record that this row's max_step changed at this cumulative step
                    self.row_config_changes.push(RowConfigChange {
                        cumulative_step: self.cumulative_steps,
                        row: *row,
                        new_max_step: *max_step,
                    });
                    println!("📝 Recorded: Row {} max_step changed to {} at cumulative step {}", 
                             row, max_step, self.cumulative_steps);
                    self.log_event("test_record_config", Some(format!("R{}:max_step={}", row, max_step)));
                }
            }
            
            // Small delay between commands
            thread::sleep(Duration::from_millis(50));
        }
        
        self.log_event("test_script_complete", Some(script.name.clone()));
        println!("\n✅ Test script completed successfully!");
        Ok(())
    }

    /// Verify sequencer state at a specific cumulative step
    fn verify_sequencer_state_at_step(&self, cumulative_step: usize) -> Result<String, String> {
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

        // Initial max_step values: [31, 30, 29, 15, 14, 13, 2]
        let initial_max_steps = vec![31, 30, 29, 15, 14, 13, 2];
        
        // Calculate expected position for each row at this cumulative step
        // accounting for any max_step changes that happened
        let mut expected_positions = Vec::new();
        
        for row_idx in 0..7 {
            // Find the most recent config change for this row before cumulative_step
            let mut current_max_step = initial_max_steps[row_idx];
            let mut last_change_step = 0;
            let mut old_max_step = initial_max_steps[row_idx];
            
            for change in &self.row_config_changes {
                if change.row == row_idx && change.cumulative_step <= cumulative_step {
                    old_max_step = current_max_step;
                    current_max_step = change.new_max_step;
                    last_change_step = change.cumulative_step;
                }
            }
            
            let expected_step = if last_change_step == 0 {
                // No config changes - simple case
                cumulative_step % (current_max_step + 1)
            } else {
                // Config changed at last_change_step
                // Calculate position at moment of change
                let position_at_change = last_change_step % (old_max_step + 1);
                
                // Sequencer resets to 0 if position > new_max_step
                let position_after_reset = if position_at_change > current_max_step {
                    0
                } else {
                    position_at_change
                };
                
                // Advance from that position
                let steps_since_change = cumulative_step - last_change_step;
                (position_after_reset + steps_since_change) % (current_max_step + 1)
            };
            
            expected_positions.push(expected_step);
        }
        
        // Count StepAdvancement events from the start to find the Nth one
        let mut step_count = 0;
        let mut found_step_data: Option<Vec<(usize, usize)>> = None;
        let mut found_bar: Option<usize> = None;
        let mut found_master_step: Option<usize> = None;
        
        for line in log_content.lines() {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
                if event.get("event_type").and_then(|v| v.as_str()) == Some("StepAdvancement") {
                    step_count += 1;
                    if step_count == cumulative_step {
                        // Found the Nth step advancement
                        found_master_step = event.get("master_step").and_then(|v| v.as_u64()).map(|s| s as usize);
                        found_bar = event.get("master_bar").and_then(|v| v.as_u64()).map(|b| b as usize);
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

        let actual_steps = found_step_data
            .ok_or(format!("No step advancement event found for cumulative step {} in formal_state.log (found {} step events)", cumulative_step, step_count))?;

        // Build verification message - show actual vs expected row states
        let mut matches = true;
        let mut details = Vec::new();
        
        for row_idx in 0..7 {
            let expected_step = expected_positions[row_idx];
            
            if let Some((_, actual_step)) = actual_steps.iter().find(|(idx, _)| *idx == row_idx) {
                if actual_step == &expected_step {
                    details.push(format!("R{}:ok({})", row_idx, actual_step));
                } else {
                    details.push(format!("R{}:act{}exp{}", row_idx, actual_step, expected_step));
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
            Err(format!("Mismatch - [{}]", details.join(", ")))
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
    /// Run test1: Multi-length pattern test (max_step: 31, 30, 29, 15, 14, 13, 2 for 7 rows)
    #[arg(long)]
    test1: bool,

    /// Create test1.json file
    #[arg(long)]
    create_test1: bool,

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

    /// Number of steps to test (default: 33 to see all rows wrap at least once)
    #[arg(long, default_value = "33")]
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
            
            // Load pre-configured test pattern with max_step: 31, 30, 29, 15, 14, 13, 2 (giving 32, 31, 30, 16, 15, 14, 3 steps)
            TestCommand::LogMilestone { 
                message: "Loading test_pattern_1.json with max_step [31, 30, 29, 15, 14, 13, 2]".to_string()
            },
            TestCommand::LoadPattern { 
                file: "test_pattern_1.json".to_string() 
            },
            
            TestCommand::LogMilestone { 
                message: "Pattern loaded to current_pattern.json".to_string() 
            },
            
            // Send SysEx command to reload pattern into sequencer
            TestCommand::LogMilestone { 
                message: "Sending SysEx reload pattern command".to_string() 
            },
            TestCommand::ReloadPattern,
            TestCommand::Wait { ms: 200 },
            
            // Start the MIDI clock
            TestCommand::LogMilestone { 
                message: "Starting MIDI clock".to_string() 
            },
            TestCommand::Start,
            TestCommand::Wait { ms: 500 },
        ];
        
        // Advance one step at a time for specified number of steps, logging state at each cumulative step
        for cumulative_step in 1..=no_of_steps {
            commands.push(TestCommand::LogMilestone { 
                message: format!("Advancing to cumulative step {}", cumulative_step) 
            });
            commands.push(TestCommand::WaitSteps { count: 1 });
            commands.push(TestCommand::VerifyState { 
                description: format!("Check sequencer state at cumulative step {}", cumulative_step) 
            });
        }
        
        // Stop the MIDI clock
        commands.push(TestCommand::LogMilestone { 
            message: "Test complete - stopping MIDI clock".to_string() 
        });
        commands.push(TestCommand::Stop);
        
        TestScript {
            name: "Test1: Multi-Length Pattern Test".to_string(),
            description: format!("Test patterns with max_step [31, 30, 29, 15, 14, 13, 2] (7 rows, giving 32,31,30,16,15,14,3 steps) and verify step counters at each step up to {}", no_of_steps),
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

    // Handle create-test1 flag
    if args.create_test1 {
        let script = AutomatedTestClock::create_test1_script(args.test_length_steps);
        let filename = "test1.json";
        match serde_json::to_string_pretty(&script) {
            Ok(json) => {
                if fs::write(filename, json).is_ok() {
                    println!("✅ Created test1.json with {} steps", args.test_length_steps);
                    println!("Run with: --script test1.json");
                } else {
                    eprintln!("❌ Error writing test1.json file");
                }
            }
            Err(e) => {
                eprintln!("❌ Error serializing test1 script: {}", e);
            }
        }
        return Ok(());
    }

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
        println!("  --test1              Run multi-length pattern test (generates test1 on-the-fly)");
        println!("  --create-test1       Create test1.json file");
        println!("  --script <file>      Execute test script from JSON file");
        println!("  --create-test        Create sample 16-step test script");
        println!("  --bpm <value>        Set initial BPM (default: 120.0)");
        println!("  --test-length-steps <n>  Number of steps for test1 (default: 33)");
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