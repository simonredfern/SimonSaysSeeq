//! Test Script Types
//! 
//! Shared types for automated test scripts that can be executed by
//! both the automated test clock and the MIDI clock generator.

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
}

/// Test script structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestScript {
    pub name: String,
    pub description: String,
    pub initial_bpm: Option<f32>,
    pub commands: Vec<TestCommand>,
}