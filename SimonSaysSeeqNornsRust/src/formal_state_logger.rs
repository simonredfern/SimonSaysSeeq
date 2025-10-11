//! Formal State Logger - Comprehensive logging system for debugging and testing
//!
//! This module provides structured logging of all critical system events including:
//! - Initial state capture
//! - Button presses and releases
//! - LED changes
//! - MIDI events (note on/off)
//! - State transitions
//!
//! The logs are written to formal_state.log in a structured format that can be
//! analyzed for debugging sequencer issues and building automated tests.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

/// Global formal state logger instance
static FORMAL_LOGGER: Mutex<Option<FormalStateLogger>> = Mutex::new(None);

/// Initialize the formal state logger
pub fn init_formal_logger() -> Result<()> {
    let logger = FormalStateLogger::new()?;
    let mut global_logger = FORMAL_LOGGER.lock().unwrap();
    *global_logger = Some(logger);
    Ok(())
}

/// Log a formal state event
pub fn log_event(event: FormalStateEvent) {
    if let Ok(mut logger_guard) = FORMAL_LOGGER.lock() {
        if let Some(logger) = logger_guard.as_mut() {
            if let Err(e) = logger.log_event(event) {
                eprintln!("Failed to log formal state event: {}", e);
            }
        }
    }
}

/// Structured event types for formal state logging
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum FormalStateEvent {
    /// System initialization with current pattern state
    SystemInit {
        timestamp: DateTime<Utc>,
        pattern_data: String, // JSON serialized pattern state
    },
    
    /// Grid button press
    ButtonPress {
        timestamp: DateTime<Utc>,
        grid_id: String,
        x: usize,
        y: usize,
        source: ButtonSource,
    },
    
    /// Grid button release
    ButtonRelease {
        timestamp: DateTime<Utc>,
        grid_id: String,
        x: usize,
        y: usize,
        source: ButtonSource,
    },
    
    /// LED state change
    LedChange {
        timestamp: DateTime<Utc>,
        grid_id: String,
        x: usize,
        y: usize,
        brightness: u8,
        reason: String, // Description of why LED changed
    },
    
    /// MIDI note on event
    MidiNoteOn {
        timestamp: DateTime<Utc>,
        note: u8,
        velocity: u8,
        channel: u8,
        source_row: usize,
        source_step: usize,
    },
    
    /// MIDI note off event
    MidiNoteOff {
        timestamp: DateTime<Utc>,
        note: u8,
        channel: u8,
        source_row: usize,
    },
    
    /// Sequencer state change
    SequencerStateChange {
        timestamp: DateTime<Utc>,
        change_type: String,
        old_value: String,
        new_value: String,
        affected_row: Option<usize>,
    },
    
    /// ARM action activation
    ArmActionActivated {
        timestamp: DateTime<Utc>,
        action: String,
        column: usize,
    },
    
    /// ARM action executed
    ArmActionExecuted {
        timestamp: DateTime<Utc>,
        action: String,
        target_row: usize,
        target_column: usize,
        result: String,
    },
    
    /// Step advancement
    StepAdvancement {
        timestamp: DateTime<Utc>,
        master_step: usize,
        master_bar: usize,
        row_steps: Vec<(usize, usize)>, // (row_index, current_step)
    },
    
    /// Test mode injection
    TestInjection {
        timestamp: DateTime<Utc>,
        injection_type: String,
        data: String,
    },
}

/// Source of button event (real hardware vs test injection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ButtonSource {
    Hardware,
    TestInjectionA,
    TestInjectionB,
}

/// Formal state logger implementation
pub struct FormalStateLogger {
    file: std::fs::File,
    session_start: DateTime<Utc>,
}

impl FormalStateLogger {
    /// Create a new formal state logger
    pub fn new() -> Result<Self> {
        let session_start = Utc::now();
        
        // Create or append to formal_state.log
        let log_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("formal_state.log");
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        
        // Log the actual file path
        eprintln!("📝 Formal state log file: {}", log_path.display());
        
        let mut logger = Self {
            file,
            session_start,
        };
        
        // Write session header
        logger.write_session_header()?;
        
        Ok(logger)
    }
    
    /// Write session start header
    fn write_session_header(&mut self) -> Result<()> {
        writeln!(self.file, "=== FORMAL STATE LOG SESSION START ===")?;
        writeln!(self.file, "Session ID: {}", self.session_start.format("%Y%m%d_%H%M%S_%3f"))?;
        writeln!(self.file, "Start Time: {}", self.session_start.format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(self.file, "==========================================")?;
        writeln!(self.file)?;
        self.file.flush()?;
        Ok(())
    }
    
    /// Log a formal state event
    pub fn log_event(&mut self, event: FormalStateEvent) -> Result<()> {
        let json_line = serde_json::to_string(&event)?;
        writeln!(self.file, "{}", json_line)?;
        self.file.flush()?;
        Ok(())
    }
}

/// Convenience functions for logging common events

/// Log system initialization with current pattern
pub fn log_system_init(pattern_json: &str) {
    log_event(FormalStateEvent::SystemInit {
        timestamp: Utc::now(),
        pattern_data: pattern_json.to_string(),
    });
}

/// Log button press
pub fn log_button_press(grid_id: &str, x: usize, y: usize, source: ButtonSource) {
    log_event(FormalStateEvent::ButtonPress {
        timestamp: Utc::now(),
        grid_id: grid_id.to_string(),
        x,
        y,
        source,
    });
}

/// Log button release
pub fn log_button_release(grid_id: &str, x: usize, y: usize, source: ButtonSource) {
    log_event(FormalStateEvent::ButtonRelease {
        timestamp: Utc::now(),
        grid_id: grid_id.to_string(),
        x,
        y,
        source,
    });
}

/// Log LED change
pub fn log_led_change(grid_id: &str, x: usize, y: usize, brightness: u8, reason: &str) {
    log_event(FormalStateEvent::LedChange {
        timestamp: Utc::now(),
        grid_id: grid_id.to_string(),
        x,
        y,
        brightness,
        reason: reason.to_string(),
    });
}

/// Log MIDI note on
pub fn log_midi_note_on(note: u8, velocity: u8, channel: u8, source_row: usize, source_step: usize) {
    log_event(FormalStateEvent::MidiNoteOn {
        timestamp: Utc::now(),
        note,
        velocity,
        channel,
        source_row,
        source_step,
    });
}

/// Log MIDI note off
pub fn log_midi_note_off(note: u8, channel: u8, source_row: usize) {
    log_event(FormalStateEvent::MidiNoteOff {
        timestamp: Utc::now(),
        note,
        channel,
        source_row,
    });
}

/// Log sequencer state change
pub fn log_sequencer_state_change(change_type: &str, old_value: &str, new_value: &str, affected_row: Option<usize>) {
    log_event(FormalStateEvent::SequencerStateChange {
        timestamp: Utc::now(),
        change_type: change_type.to_string(),
        old_value: old_value.to_string(),
        new_value: new_value.to_string(),
        affected_row,
    });
}

/// Log ARM action activation
pub fn log_arm_action_activated(action: &str, column: usize) {
    log_event(FormalStateEvent::ArmActionActivated {
        timestamp: Utc::now(),
        action: action.to_string(),
        column,
    });
}

/// Log ARM action execution
pub fn log_arm_action_executed(action: &str, target_row: usize, target_column: usize, result: &str) {
    log_event(FormalStateEvent::ArmActionExecuted {
        timestamp: Utc::now(),
        action: action.to_string(),
        target_row,
        target_column,
        result: result.to_string(),
    });
}

/// Log step advancement
pub fn log_step_advancement(master_step: usize, master_bar: usize, row_steps: Vec<(usize, usize)>) {
    log_event(FormalStateEvent::StepAdvancement {
        timestamp: Utc::now(),
        master_step,
        master_bar,
        row_steps,
    });
}

/// Log test injection
pub fn log_test_injection(injection_type: &str, data: &str) {
    log_event(FormalStateEvent::TestInjection {
        timestamp: Utc::now(),
        injection_type: injection_type.to_string(),
        data: data.to_string(),
    });
}