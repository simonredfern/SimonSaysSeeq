//! Test Script Types
//! 
//! Shared types for automated test scripts that can be executed by
//! both the automated test clock and the MIDI clock generator.
//!
//! ## SysEx Commands
//!
//! SysEx command format: `F0 7D 53 53 51 <cmd> [params...] F7`
//! - `F0` = SysEx start
//! - `7D` = Educational/Development use (non-commercial manufacturer ID)
//! - `53 53 51` = "SSQ" in ASCII (SimonSaysSeeQ signature)
//! - `<cmd>` = Command code:
//!   - `01` = Reload pattern from current_pattern.json
//!   - `02` = Button press/release: `02 <row> <col> <press>`
//! - `F7` = SysEx end
//!

//! Note: The main sequencer application must be running to receive commands.

use serde::{Deserialize, Serialize};

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
    /// Send SysEx command to reload pattern from current_pattern.json
    /// Note: Requires the main sequencer application to be running to receive the command
    ReloadPattern,
    /// Send SysEx command to press/release a button
    /// Format: F0 7D 53 53 51 02 <row> <col> <press> F7
    /// row: 0-7, col: 0-31, press: 1=press, 0=release
    SysExButton { row: u8, col: u8, press: bool },
    /// Record a row configuration change for verification tracking
    /// This tells the test framework that a row's max_step changed
    /// Used to predict expected sequencer state in subsequent verification steps
    RecordRowConfig { row: usize, max_step: usize },
}

/// Test script structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestScript {
    pub name: String,
    pub description: String,
    pub initial_bpm: Option<f32>,
    pub commands: Vec<TestCommand>,
}

/// Direct test action - executed at specific tick counts
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum DirectTestAction {
    /// Log a message
    LogMessage { message: String },
    /// Send SysEx button press/release
    SysExButton { row: u8, col: u8, press: bool },
    /// Verify sequencer state at this tick
    VerifyState { 
        step: u32,           // Expected step number (tick / 6)
        row: usize,          // Which row to verify
        expected: Option<usize>, // Expected position (None = just observe)
    },
}

/// Direct test command - action at specific tick
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirectTestCommand {
    pub at_tick: u32,
    pub action: DirectTestAction,
}

/// Direct test script - tick-synchronized testing
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirectTestScript {
    pub name: String,
    pub description: String,
    pub bpm: f32,
    pub commands: Vec<DirectTestCommand>,
}