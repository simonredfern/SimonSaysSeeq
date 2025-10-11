//! Shared MIDI Clock Generator
//! 
//! This module provides a precise MIDI clock generator that can be used by
//! both the interactive midi_clock_generator and direct_test binaries.
//! 
//! Features:
//! - Drift-corrected timing for accurate clock generation
//! - Tick counting for tick-synchronized tests
//! - Optional test mode with tempo changes
//! - Thread-safe BPM control

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use midir::{MidiOutput, MidiOutputConnection};

use crate::test_script::{DirectTestScript, DirectTestAction};

/// Clock command for internal communication
#[derive(Debug, Clone)]
pub enum ClockCommand {
    Start,
    Stop,
    SetBpm(f32),
    ToggleTestMode,
    SendRawMidi(Vec<u8>),
    Exit,
}

/// Configuration options for the clock generator
#[derive(Debug, Clone)]
pub struct ClockConfig {
    /// Enable tick counting (needed for automated tests)
    pub enable_tick_counting: bool,
    /// Enable test mode with automatic tempo changes
    pub enable_test_mode: bool,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            enable_tick_counting: false,
            enable_test_mode: false,
        }
    }
}

/// Main clock generator structure
pub struct ClockGenerator {
    bpm: Arc<Mutex<f32>>,
    is_running: Arc<AtomicBool>,
    should_exit: Arc<AtomicBool>,
    command_sender: Option<Sender<ClockCommand>>,
    test_mode: Arc<AtomicBool>,
    tick_count: Arc<Mutex<u32>>,
    config: ClockConfig,
    direct_test_script: Option<Arc<DirectTestScript>>,
}

impl Clone for ClockGenerator {
    fn clone(&self) -> Self {
        Self {
            bpm: self.bpm.clone(),
            is_running: self.is_running.clone(),
            should_exit: self.should_exit.clone(),
            command_sender: self.command_sender.clone(),
            test_mode: self.test_mode.clone(),
            tick_count: self.tick_count.clone(),
            config: self.config.clone(),
            direct_test_script: self.direct_test_script.clone(),
        }
    }
}

impl ClockGenerator {
    /// Create a new clock generator with default configuration
    pub fn new(bpm: f32) -> Self {
        Self::new_with_config(bpm, ClockConfig::default())
    }

    /// Create a new clock generator with custom configuration
    pub fn new_with_config(bpm: f32, config: ClockConfig) -> Self {
        Self {
            bpm: Arc::new(Mutex::new(bpm.clamp(20.0, 300.0))),
            is_running: Arc::new(AtomicBool::new(false)),
            should_exit: Arc::new(AtomicBool::new(false)),
            command_sender: None,
            test_mode: Arc::new(AtomicBool::new(false)),
            tick_count: Arc::new(Mutex::new(0)),
            config,
            direct_test_script: None,
        }
    }
    
    /// Set a direct test script to be executed during clock generation
    pub fn set_direct_test_script(&mut self, script: DirectTestScript) {
        self.direct_test_script = Some(Arc::new(script));
    }

    /// Initialize MIDI output connection
    pub fn connect_midi_output(&self) -> Result<MidiOutputConnection, Box<dyn std::error::Error>> {
        let midi_out = MidiOutput::new("MIDI Clock Generator")?;
        let out_ports = midi_out.ports();

        if out_ports.is_empty() {
            return Err("No MIDI output ports available".into());
        }

        println!("Available MIDI output ports:");
        for (i, port) in out_ports.iter().enumerate() {
            if let Ok(port_name) = midi_out.port_name(port) {
                println!("  {}: {}", i, port_name);
            }
        }

        let selected_port = if out_ports.len() == 1 {
            println!("Auto-selecting the only available port");
            0
        } else {
            println!("Select MIDI output port (0-{}): ", out_ports.len() - 1);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().parse::<usize>().unwrap_or(0)
        };

        if selected_port >= out_ports.len() {
            return Err("Invalid port selection".into());
        }

        let port = &out_ports[selected_port];
        let port_name = midi_out.port_name(port).unwrap_or_else(|_| "Unknown".to_string());
        println!("Connecting to: {}", port_name);

        let connection = midi_out.connect(port, "MIDI Clock")?;
        Ok(connection)
    }

    /// Start the MIDI clock
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_running.store(true, Ordering::Relaxed);
        if let Some(ref sender) = self.command_sender {
            sender.send(ClockCommand::Start)?;
        }
        Ok(())
    }

    /// Stop the MIDI clock
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(ref sender) = self.command_sender {
            sender.send(ClockCommand::Stop)?;
        }
        Ok(())
    }

    /// Set the BPM (clamped to 20.0 - 300.0)
    pub fn set_bpm(&self, bpm: f32) {
        let clamped_bpm = bpm.clamp(20.0, 300.0);
        *self.bpm.lock().unwrap() = clamped_bpm;
        if let Some(ref sender) = self.command_sender {
            let _ = sender.send(ClockCommand::SetBpm(clamped_bpm));
        }
        println!("BPM set to: {:.1}", clamped_bpm);
    }

    /// Get the current BPM
    pub fn get_bpm(&self) -> f32 {
        *self.bpm.lock().unwrap()
    }

    /// Check if the clock is currently running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    /// Get the current tick count (if tick counting is enabled)
    pub fn get_tick_count(&self) -> u32 {
        *self.tick_count.lock().unwrap()
    }

    /// Reset the tick count
    pub fn reset_tick_count(&self) {
        *self.tick_count.lock().unwrap() = 0;
    }
    
    /// Execute a direct test action (called from clock thread)
    fn execute_direct_test_action(action: &DirectTestAction, connection: &mut midir::MidiOutputConnection, tick: u32) {
        match action {
            DirectTestAction::LogMessage { message } => {
                println!("📍 Tick {}: {}", tick, message);
            }
            DirectTestAction::SysExButton { row, col, press } => {
                let press_byte = if *press { 0x01 } else { 0x00 };
                let sysex = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, 0x02, *row, *col, press_byte, 0xF7];
                let action_str = if *press { "PRESS" } else { "RELEASE" };
                println!("📍 Tick {}: SysEx {} row={} col={}", tick, action_str, row, col);
                if let Err(e) = connection.send(&sysex) {
                    eprintln!("Error sending SysEx: {}", e);
                }
            }
            DirectTestAction::VerifyState { step, row, expected } => {
                println!("📍 Tick {}: Verify step={} row={} expected={:?}", tick, step, row, expected);
                Self::verify_state_at_step(*step, *row, *expected);
            }
        }
    }
    
    /// Verify sequencer state by reading formal_state.log
    fn verify_state_at_step(step: u32, row: usize, expected: Option<usize>) {
        use std::fs;
        
        // Read formal_state.log
        let log_content = match fs::read_to_string("formal_state.log") {
            Ok(content) => content,
            Err(e) => {
                println!("  ❌ Cannot read formal_state.log: {}", e);
                return;
            }
        };
        
        // Find the Nth StepAdvancement event
        let mut step_count = 0;
        let mut found_position: Option<usize> = None;
        
        for line in log_content.lines() {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
                if event.get("event_type").and_then(|v| v.as_str()) == Some("StepAdvancement") {
                    step_count += 1;
                    if step_count == step {
                        // Found the step - extract row position
                        if let Some(row_steps) = event.get("row_steps").and_then(|v| v.as_array()) {
                            for rs in row_steps {
                                if let (Some(r), Some(pos)) = (
                                    rs.get(0).and_then(|v| v.as_u64()),
                                    rs.get(1).and_then(|v| v.as_u64())
                                ) {
                                    if r as usize == row {
                                        found_position = Some(pos as usize);
                                        break;
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
        
        match (found_position, expected) {
            (Some(actual), Some(exp)) => {
                if actual == exp {
                    println!("  ✅ Row {} position: {} (expected {})", row, actual, exp);
                } else {
                    println!("  ❌ Row {} position: {} (expected {})", row, actual, exp);
                }
            }
            (Some(actual), None) => {
                println!("  📊 Row {} position: {} (observation only)", row, actual);
            }
            (None, _) => {
                println!("  ⚠️  Step {} not found in log yet (only {} steps logged)", step, step_count);
            }
        }
    }

    /// Exit the clock generator
    pub fn exit(&self) {
        self.should_exit.store(true, Ordering::Relaxed);
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(ref sender) = self.command_sender {
            let _ = sender.send(ClockCommand::Exit);
        }
    }

    /// Toggle test mode (if enabled in config)
    pub fn toggle_test_mode(&self) {
        if let Some(ref sender) = self.command_sender {
            let _ = sender.send(ClockCommand::ToggleTestMode);
        }
    }

    /// Send raw MIDI message (for SysEx and other special commands)
    pub fn send_raw_midi(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref sender) = self.command_sender {
            sender.send(ClockCommand::SendRawMidi(data.to_vec()))?;
            Ok(())
        } else {
            Err("Clock thread not initialized".into())
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
        let tick_count = self.tick_count.clone();
        let enable_tick_counting = self.config.enable_tick_counting;
        let enable_test_mode = self.config.enable_test_mode;
        let direct_test_script = self.direct_test_script.clone();

        thread::spawn(move || {
            println!("Clock generation thread started");

            while !should_exit.load(Ordering::Relaxed) {
                // Handle commands from main thread
                if let Ok(command) = receiver.try_recv() {
                    match command {
                        ClockCommand::Start => {
                            is_running.store(true, Ordering::Relaxed);
                            if enable_tick_counting {
                                *tick_count.lock().unwrap() = 0;
                            }
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
                            if enable_test_mode {
                                let new_test_mode = !test_mode.load(Ordering::Relaxed);
                                test_mode.store(new_test_mode, Ordering::Relaxed);
                                if new_test_mode {
                                    println!("Test mode enabled: automatic tempo changes (120->125->121->140->130->122->110)");
                                } else {
                                    println!("Test mode disabled");
                                }
                            }
                        }
                        ClockCommand::SendRawMidi(data) => {
                            if let Err(e) = connection.send(&data) {
                                eprintln!("Error sending raw MIDI: {}", e);
                            }
                        }
                        ClockCommand::Exit => {
                            should_exit.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                }

                // Clock generation with drift correction
                if is_running.load(Ordering::Relaxed) {
                    // Timing state (persists across iterations)
                    static mut START_TIME: Option<Instant> = None;
                    static mut TICK_COUNT_LOCAL: u64 = 0;
                    static mut LAST_BPM: f32 = 0.0;
                    static mut TEST_START_TIME: Option<Instant> = None;

                    unsafe {
                        // Initialize timing on first run
                        if START_TIME.is_none() {
                            START_TIME = Some(Instant::now());
                            TICK_COUNT_LOCAL = 0;
                            LAST_BPM = *bpm.lock().unwrap();
                            TEST_START_TIME = Some(Instant::now());
                        }

                        let start_time = START_TIME.unwrap();
                        let test_start_time = TEST_START_TIME.unwrap();
                        let mut tick_count_local = TICK_COUNT_LOCAL;
                        let mut last_bpm = LAST_BPM;
                        
                        let mut current_bpm = *bpm.lock().unwrap();
                        
                        // Reset timing reference if BPM changed to prevent timing disruption
                        if (current_bpm - last_bpm).abs() > 0.1 {
                            START_TIME = Some(Instant::now());
                            TICK_COUNT_LOCAL = 0;
                            tick_count_local = 0;
                            LAST_BPM = current_bpm;
                            last_bpm = current_bpm;
                        }
                        
                        // Test mode: discrete tempo changes with stop/start cycles
                        if enable_test_mode && test_mode.load(Ordering::Relaxed) {
                            let elapsed_secs = test_start_time.elapsed().as_secs_f32();
                            
                            // Extended test sequence with stop/start cycles
                            let cycle_duration = 120.0;
                            let cycle_position = elapsed_secs % cycle_duration;
                            
                            // Check if we should be stopped
                            let should_run = if cycle_position >= 100.0 && cycle_position < 110.0 {
                                // Stop clock for 10 seconds
                                if is_running.load(Ordering::Relaxed) {
                                    is_running.store(false, Ordering::Relaxed);
                                    println!("Test mode: Stopping clock for 10 seconds");
                                }
                                false
                            } else if cycle_position >= 110.0 && cycle_position < 120.0 {
                                // Restart clock for 10 seconds
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
                                    120.0
                                } else if cycle_position < 20.0 {
                                    125.0
                                } else if cycle_position < 30.0 {
                                    121.0
                                } else if cycle_position < 40.0 {
                                    140.0
                                } else if cycle_position < 60.0 {
                                    130.0
                                } else if cycle_position < 80.0 {
                                    122.0
                                } else if cycle_position < 100.0 {
                                    110.0
                                } else {
                                    current_bpm
                                };
                                
                                *bpm.lock().unwrap() = current_bpm;
                            }
                        }
                        
                        // Drift-corrected timing
                        let ticks_per_second = (current_bpm * 24.0) / 60.0;
                        let tick_interval_secs = 1.0 / ticks_per_second;
                        
                        let target_time = start_time + Duration::from_secs_f32(tick_count_local as f32 * tick_interval_secs);
                        let now = Instant::now();
                        
                        if now >= target_time {
                            // Send MIDI clock tick
                            if let Err(e) = connection.send(&[0xF8]) {
                                eprintln!("Error sending MIDI clock: {}", e);
                                break;
                            }
                            
                            tick_count_local += 1;
                            
                            // Update shared tick count if enabled
                            if enable_tick_counting {
                                *tick_count.lock().unwrap() += 1;
                            }
                            
                            // Print a dot for each step (every 6 ticks) in direct test mode
                            if direct_test_script.is_some() && tick_count_local % 6 == 0 {
                                print!(".");
                                use std::io::Write;
                                std::io::stdout().flush().ok();
                            }
                            
                            // Execute direct test actions at this tick
                            if let Some(ref script) = direct_test_script {
                                for cmd in &script.commands {
                                    if cmd.at_tick == tick_count_local as u32 {
                                        Self::execute_direct_test_action(&cmd.action, &mut connection, tick_count_local as u32);
                                    }
                                }
                            }
                        }
                        
                        // Store state for next iteration
                        TICK_COUNT_LOCAL = tick_count_local;
                        LAST_BPM = last_bpm;
                    }
                }

                // Adaptive sleep based on time until next tick
                let current_bpm = *bpm.lock().unwrap();
                let ticks_per_second = (current_bpm * 24.0) / 60.0;
                let tick_interval = Duration::from_secs_f32(1.0 / ticks_per_second);
                let sleep_duration = tick_interval / 10;
                thread::sleep(sleep_duration.max(Duration::from_micros(100)));
            }

            println!("Clock generation thread exiting");
        })
    }
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
        assert_eq!(gen.get_tick_count(), 0);
    }

    #[test]
    fn test_tick_counting_config() {
        let config = ClockConfig {
            enable_tick_counting: true,
            enable_test_mode: false,
        };
        let gen = ClockGenerator::new_with_config(120.0, config);
        assert_eq!(gen.get_tick_count(), 0);
    }
}