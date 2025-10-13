//! Sequencer module - Core timing and pattern logic
//!
//! Handles the main sequencing engine, timing, steps, bars, and pattern storage.

use anyhow::Result;
use crossbeam_channel::Sender;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use crate::formal_state_logger::{log_system_init, log_sequencer_state_change, log_step_advancement};

/// MIDI gate base note - matches Lua version
/// This works well with Flame MGTV factory default settings.
/// http://flame.fortschritt-musik.de/pdf/Manual_Flame_MGTV_module_v100_eng.pdf
const LOWEST_MIDI_NOTE_NUMBER_FOR_GATE: u8 = 47;

/// Clock division reset outputs - MIDI notes for clock-synced reset signals
const RESET_1_NOTE: u8 = 55;      // Triggers every 1 step
const RESET_16_NOTE: u8 = 56;     // Triggers every 16 steps
const RESET_32_NOTE: u8 = 57;     // Triggers every 32 steps
const RESET_64_NOTE: u8 = 58;     // Triggers every 64 steps
const RESET_128_NOTE: u8 = 59;    // Triggers every 128 steps
const RESET_CHANNEL: u8 = 1;      // MIDI channel for reset outputs
const RESET_VELOCITY: u8 = 100;   // Velocity for reset note-ons

/// MIDI event for hardware output
#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub note: u8,
    pub velocity: u8,
    pub channel: u8,
    pub note_on: bool,
    pub step: usize,
    pub sequencer_source: char, // 'A' for sequencer_a
}

/// Pending note-off event scheduled for a specific tick
#[derive(Debug, Clone)]
pub struct PendingNoteOff {
    pub note: u8,
    pub channel: u8,
    pub row: usize,
    pub target_tick: u64,
}

/// Events that the sequencer can send to the main application
#[derive(Debug, Clone)]
pub enum SequencerEvent {
    /// A step has been reached
    Step { step: usize },
    /// A beat has been reached (for visual indicators)
    Beat { beat: usize },
    /// MIDI event to be sent to hardware
    MidiEvent(MidiEvent),
    /// Grid update needed for specific row position change
    GridUpdate { row: usize, old_step: usize, new_step: usize },
}

/// A MIDI note event at a specific timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteEvent {
    pub note: u8,
    pub velocity: u8,
    pub channel: u8,
    pub note_on: bool,
    pub tick_offset: u32, // Tick within the step (0-11 for 12 ticks per step)
}

/// Row settings for each sequence row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerARowStates {
    pub current_row_step: usize,
    pub first_step: usize,
    pub max_step: usize,
    pub euclidean_events: usize,
    pub euclidean_rotation: usize,
    pub previous_row_step: usize,
    pub midi_note: u8,
    pub midi_velocity: u8,
    pub midi_channel: u8,
    pub ratchet_count: u8,
}

/// MIDI note event for recording and playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiNoteEvent {
    pub velocity: u8,
    pub tick_count_since_step: u32,
    pub is_active: bool,
}

impl Default for MidiNoteEvent {
    fn default() -> Self {
        Self {
            velocity: 0,
            tick_count_since_step: 0,
            is_active: false,
        }
    }
}

impl Default for SequencerARowStates {
    fn default() -> Self {
        Self {
            current_row_step: 0,
            first_step: 0,
            max_step: 31,
            euclidean_events: 5,
            euclidean_rotation: 0,
            previous_row_step: 31,
            midi_note: LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + 1, // Default to first gate note (48)
            midi_velocity: 100,
            midi_channel: 1,
            ratchet_count: 1,
        }
    }
}



/// Undo/Redo state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub grid: Vec<Vec<u8>>,
    pub mozart: Vec<Vec<u8>>,
    pub row_states: Vec<SequencerARowStates>,
    pub timestamp: std::time::SystemTime,
    pub description: String,
}

/// Slide state for smooth parameter transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideState {
    pub slide_grid: Vec<Vec<f32>>, // Slide amount per grid position
    pub scroll_offset: (i32, i32), // X, Y scroll offset for patterns
}

/// Main sequencer state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerState {
    /// Sequencer A Grid state: grid[column][row] = value (0=off, 1=on, 2+=ratchet)
    pub sequencer_a_grid: Vec<Vec<u8>>,
    /// Sequencer A Mozart state (second grid/performance controls) - MIDI note values
    pub sequencer_a_mozart: Vec<Vec<u8>>,

    /// Slide state for parameter transitions
    pub slide: SlideState,
    /// Held state for grid button combinations
    pub held: Vec<Vec<u8>>,
    /// Sequencer A Row states for each sequence row
    pub sequencer_a_row_states: Vec<SequencerARowStates>,
    /// Sequencer A Current global position
    pub sequencer_a_current_master_step: usize,
    pub sequencer_a_current_lane: usize,
    /// Transport state
    pub is_running: bool,
    /// Core timing variables
    pub swing_amount: f32,
    pub ticks_per_step: u32,
    pub steps_per_bar: usize,
    pub tick_count_since_midi_clock_start: u64,
    pub tick_in_step: u32,
    /// Clock division reset tracking
    pub global_reset_step_counter: usize,
    pub reset_1_active: bool,
    pub reset_16_active: bool,
    pub reset_32_active: bool,
    pub reset_64_active: bool,
    pub reset_128_active: bool,
    pub first_step: usize,
    pub last_step: usize,
    pub midi_first_step: usize,
    pub midi_last_step: usize,
    /// Advanced features
    pub swing_mode: u8,
    pub global_transpose: i8,
    pub global_velocity_scale: f32,
    /// CO2 integration
    pub total_step_co2_count: usize,
    pub total_tick_co2_count: usize,
    /// Sequencer constants
    pub total_sequence_rows: usize,
    pub cols: usize,
    pub rows: usize,
    pub min_lane: usize,
    pub max_lane: usize,
    pub max_step: usize,
    /// Flag to indicate tick counter should be reset on next clock loop iteration
    pub reset_tick_counter: bool,
}

impl Default for SequencerState {
    fn default() -> Self {
        const COLS: usize = 32;
        const ROWS: usize = 8;
        const MIN_LANE: usize = 1;
        const MAX_LANE: usize = 2;
        const MAX_STEP: usize = 31;
        const TOTAL_STEPS: usize = 32;
        const TOTAL_SEQUENCE_ROWS: usize = 7;

        let grid = vec![vec![0u8; ROWS]; COLS];
        let mozart = vec![vec![60u8; ROWS]; COLS]; // Initialize to middle C
        let slide = SlideState {
            slide_grid: vec![vec![0.0; ROWS]; COLS],
            scroll_offset: (0, 0),
        };
        let held = vec![vec![0u8; ROWS]; COLS];

        let mut row_states = Vec::new();
        for i in 0..ROWS {
            let mut seq_a_row_state = SequencerARowStates::default();
            // Match Lua version: LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + sequence_row (1-based)
            // Lua uses sequence_row 1-7, Rust uses i 0-6, so add 1 to convert
            seq_a_row_state.midi_note = LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + (i + 1) as u8;
            seq_a_row_state.midi_channel = 1; // All rows use MIDI channel 1
            row_states.push(seq_a_row_state);
        }



        Self {
            sequencer_a_grid: grid,
            sequencer_a_mozart: mozart,
            slide,
            held,
            sequencer_a_row_states: row_states,
            sequencer_a_current_master_step: 0,
            sequencer_a_current_lane: 1,
            is_running: false,
            swing_amount: 0.0,
            ticks_per_step: 12,
            first_step: 0,
            last_step: 31,
            steps_per_bar: 16,
            tick_count_since_midi_clock_start: 0,
            tick_in_step: 0,
            midi_first_step: 0,
            midi_last_step: 31,
            swing_mode: 1,
            global_transpose: 0,
            global_velocity_scale: 1.0,
            total_step_co2_count: 1,
            total_tick_co2_count: 1,
            total_sequence_rows: TOTAL_SEQUENCE_ROWS,
            cols: COLS,
            rows: ROWS,
            min_lane: MIN_LANE,
            max_lane: MAX_LANE,
            max_step: MAX_STEP,
            reset_tick_counter: false,
            global_reset_step_counter: 0,
            reset_1_active: false,
            reset_16_active: false,
            reset_32_active: false,
            reset_64_active: false,
            reset_128_active: false,
        }
    }
}

/// Main sequencer engine
#[derive(Clone)]
pub struct Sequencer {
    state: Arc<std::sync::Mutex<SequencerState>>,
    note_events: Arc<std::sync::Mutex<HashMap<(usize, usize, usize, u8), NoteEvent>>>, // (lane, step, note) -> event (bar removed)
    /// Undo/Redo system
    undo_stack: Arc<std::sync::Mutex<VecDeque<StateSnapshot>>>,
    redo_stack: Arc<std::sync::Mutex<VecDeque<StateSnapshot>>>,
    max_undo_history: usize,
    /// Pattern storage (multiple patterns)
    patterns: Arc<std::sync::Mutex<HashMap<usize, SequencerState>>>,
    current_pattern: usize,
    /// Test mode flag (disables auto-save, auto-loads test_pattern_1.json)
    test_mode: bool,
    /// Pending note-off events scheduled by tick
    pending_note_offs: Arc<Mutex<Vec<PendingNoteOff>>>,
}

impl Sequencer {
    pub fn new() -> Self {
        Self::new_with_test_mode(false).expect("Failed to create sequencer")
    }

    pub fn new_with_test_mode(test_mode: bool) -> Result<Self> {
        let sequencer = Self {
            state: Arc::new(std::sync::Mutex::new(SequencerState::default())),
            note_events: Arc::new(std::sync::Mutex::new(HashMap::new())),
            undo_stack: Arc::new(std::sync::Mutex::new(VecDeque::new())),
            redo_stack: Arc::new(std::sync::Mutex::new(VecDeque::new())),
            max_undo_history: 50,
            patterns: Arc::new(std::sync::Mutex::new(HashMap::new())),
            current_pattern: 0,
            test_mode,
            pending_note_offs: Arc::new(Mutex::new(Vec::new())),
        };

        // In test mode, load test_pattern_1.json
        if test_mode {
            info!("🧪 Test mode: Loading test_pattern_1.json");
            match sequencer.load_pattern_from_file("test_pattern_1.json") {
                Ok(()) => {
                    info!("✅ Test pattern loaded successfully");
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to load test_pattern_1.json: {}", e));
                }
            }
        } else {
            // Normal mode: Try to load saved pattern, or create default sparse pattern if none exists
            match sequencer.load_current_pattern_from_file() {
                Ok(()) => {
                    // info!("Loaded saved pattern on startup");
                }
                Err(_) => {
                    // info!("No saved pattern found, creating default sparse pattern");
                    sequencer.create_default_sparse_pattern();
                }
            }
        }

        // Initialize with a snapshot for undo/redo
        sequencer.push_undo_snapshot("Initial state".to_string());

        Ok(sequencer)
    }

    /// Start the sequencer
    pub fn start(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.is_running {
            state.is_running = true;
            state.reset_tick_counter = true;


            // Reset to beginning
            state.sequencer_a_current_master_step = 0;
            for seq_a_row_state in &mut state.sequencer_a_row_states {
                seq_a_row_state.current_row_step = 0;
            }

            // Reset clock division counters
            state.global_reset_step_counter = 0;
            state.reset_1_active = false;
            state.reset_16_active = false;
            state.reset_32_active = false;
            state.reset_64_active = false;
            state.reset_128_active = false;

            // info!("Sequencer started");
        }
    }

    /// Stop the sequencer
    pub fn stop(&self) {
        let mut state = self.state.lock().unwrap();
        if state.is_running {
            state.is_running = false;
            // Reset to beginning
            state.sequencer_a_current_master_step = 0;
            for seq_a_row_state in &mut state.sequencer_a_row_states {
                seq_a_row_state.current_row_step = 0;
            }
            // Log the stack trace to identify what triggered the stop
            let trace = std::backtrace::Backtrace::capture();
            // info!("Sequencer stopped and reset - Stack trace: {}", trace);

            // Save current pattern when stopping (unless in test mode)
            drop(state); // Release the lock before calling save method
            if !self.test_mode {
                if let Err(e) = self.save_current_pattern_to_file() {
                    // warn!("Failed to save current pattern on stop: {}", e);
                }
            } else {
                info!("🧪 Test mode: Skipping auto-save on stop (pattern protection enabled)");
            }
        }
    }

    /// Check if sequencer is running
    pub fn is_running(&self) -> bool {
        self.state.lock().unwrap().is_running
    }



    /// Get current position
    /// Get the current playback position (step only)
    pub fn get_position(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.sequencer_a_current_master_step
    }



    /// Set grid value at position with automatic undo snapshot
    pub fn set_grid_value(&self, x: usize, y: usize, value: u8) {
        if x < 32 && y < 8 {
            self.push_undo_snapshot(format!(
                "Set grid[{}][{}] = {} (display: step {}, row {})",
                x,
                y,
                value,
                x + 1,
                y + 1
            ));
            let mut state = self.state.lock().unwrap();
            state.sequencer_a_grid[x][y] = value;
        }
    }

    /// Get grid value at position
    pub fn get_grid_value(&self, x: usize, y: usize) -> u8 {
        let state = self.state.lock().unwrap();
        if x < 32 && y < 8 {
            if x < state.sequencer_a_grid.len() && y < state.sequencer_a_grid[0].len() {
                state.sequencer_a_grid[x][y]
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Set Sequence B grid value (MIDI note number)
    pub fn set_sequence_b_value(&self, x: usize, y: usize, note: u8) {
        if x > 0 && x <= 32 && y > 0 && y <= 8 {
            self.push_undo_snapshot(format!("Set sequence_b[{}][{}] = {}", x, y, note));

            let mut state = self.state.lock().unwrap();
            let clamped_note = note.min(127);
            state.sequencer_a_mozart[x - 1][y - 1] = clamped_note;
            // Clear slide for immediate response
            state.slide.slide_grid[x - 1][y - 1] = 0.0;
            // debug!("Set mozart[{}][{}] = {}", x, y, clamped_note);
        }
    }

    /// Get Mozart grid value (MIDI note number) - simplified to return default
    pub fn get_mozart_value(&self, _x: usize, _y: usize) -> u8 {
        60 // Default to middle C
    }

    /// Get swing amount
    pub fn get_swing_amount(&self) -> f32 {
        let state = self.state.lock().unwrap();
        state.swing_amount
    }

    /// Set swing amount
    pub fn set_swing(&self, amount: f32) {
        let mut state = self.state.lock().unwrap();
        state.swing_amount = amount.clamp(0.0, 0.5);
    }



    /// Get current step
    pub fn get_current_position(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.sequencer_a_current_master_step
    }









    /// Get row data for display
    pub fn get_row_data(&self, row: usize) -> Vec<u8> {
        let state = self.state.lock().unwrap();
        if row < 8 {
            (0..32).map(|x| state.sequencer_a_grid[x][row]).collect()
        } else {
            vec![0; 32]
        }
    }





    /// Check if position is held
    pub fn is_held(&self, x: usize, y: usize) -> bool {
        let state = self.state.lock().unwrap();
        if x > 0 && x <= 32 && y > 0 && y <= 8 {
            state.held[x - 1][y - 1] > 0
        } else {
            false
        }
    }



    /// Get note events for a specific step
    pub fn get_step_events(&self, row: usize, _bar: usize, step: usize) -> Option<Vec<NoteEvent>> {
        let state = self.state.lock().unwrap();
        let _note_events = self.note_events.lock().unwrap();

        if row > 6 {
            return None; // Only sequence rows 0-6
        }

        let grid_value = state.sequencer_a_grid[step][row];
        if grid_value == 0 {
            return None; // No trigger
        }

        let seq_a_row_state = &state.sequencer_a_row_states[row];

        // Check if this step is within the row's range
        if step < seq_a_row_state.first_step || step > seq_a_row_state.max_step {
            return None;
        }

        let mut events = Vec::new();

        // Handle ratcheting
        let ratchet_count = if grid_value >= 2 {
            std::cmp::min(grid_value as usize, 4)
        } else {
            1
        };

        let ticks_between_ratchets = state.ticks_per_step / ratchet_count as u32;

        for ratchet in 0..ratchet_count {
            let tick_offset = ratchet as u32 * ticks_between_ratchets;

            // Note ON event
            events.push(NoteEvent {
                note: seq_a_row_state.midi_note,
                velocity: seq_a_row_state.midi_velocity,
                channel: seq_a_row_state.midi_channel,
                note_on: true,
                tick_offset,
            });

            // Note OFF event (slightly before next ratchet or at end of step)
            let off_tick = if ratchet < ratchet_count - 1 {
                tick_offset + ticks_between_ratchets - 1
            } else {
                state.ticks_per_step - 1
            };

            events.push(NoteEvent {
                note: seq_a_row_state.midi_note,
                velocity: 0,
                channel: seq_a_row_state.midi_channel,
                note_on: false,
                tick_offset: off_tick,
            });
        }

        Some(events)
    }

    /// Main clock loop that runs in its own thread
    // Internal timing methods removed - external clock slave mode only

    // play_midi removed - not needed for external clock mode

    /// Advance to next step and process triggers
    fn advance_step(
        &self,
        state: &mut std::sync::MutexGuard<SequencerState>,
        sender: &Sender<SequencerEvent>,
    ) -> Result<()> {
        // Reset tick count since step
        state.tick_in_step = 0;

        // Process pending note-offs for current tick
        self.process_pending_note_offs(state.tick_count_since_midi_clock_start, sender)?;

        // Process current step for all sequence rows
        self.process_step(&*state, sender)?;

        // Send step event with CURRENT step values (before increment) for MIDI sync
        let _ = sender.try_send(SequencerEvent::Step {
            step: state.sequencer_a_current_master_step,
        });

        // Advance sequencer master step
        state.sequencer_a_current_master_step += 1;
        if state.sequencer_a_current_master_step > state.last_step {
            state.sequencer_a_current_master_step = state.first_step;
        }

        // Advance all row step counters and collect step info for logging
        // formal_state_row_steps will contain tuples of (row_index, new_step_position) for each row
        // e.g., [(0, 5), (1, 5), (2, 5), ..., (6, 0), ...] 
        // This is logged to formal_state.log as StepAdvancement event for debugging
        let mut formal_state_row_steps = Vec::new();
        for (row_idx, seq_a_row_state) in state.sequencer_a_row_states.iter_mut().enumerate() {
            seq_a_row_state.previous_row_step = seq_a_row_state.current_row_step;
            seq_a_row_state.current_row_step += 1;
            // Wrap step counter if it exceeds the max_step
            // max_step is the maximum valid step index (e.g., 31 for 32 steps)
            if seq_a_row_state.current_row_step > seq_a_row_state.max_step {
                seq_a_row_state.current_row_step = seq_a_row_state.first_step;
            }
            // Collect the row's new position for logging (doesn't affect sequencer logic)
            formal_state_row_steps.push((row_idx, seq_a_row_state.current_row_step));
        }

        // Log step advancement to formal state logger
        log_step_advancement(state.sequencer_a_current_master_step, 0, formal_state_row_steps);

        // Advance global reset step counter
        state.global_reset_step_counter += 1;
        if state.global_reset_step_counter > 127 {
            state.global_reset_step_counter = 0;
        }

        // Update CO2 counters if we have data
        if state.total_step_co2_count > 0 {
            // This would advance CO2 counters if CO2 data is available
            // Implementation would depend on CO2 data structure
        }

        Ok(())
    }

    /// Process triggers for the current step and handle selective grid updates
    fn process_step(&self, state: &SequencerState, sender: &Sender<SequencerEvent>) -> Result<()> {
        // DEBUG: Show current step for all rows
        debug!("Master Step: {:02} | Steps: [{}]",
            state.sequencer_a_current_master_step,
            state.sequencer_a_row_states.iter()
                .enumerate()
                .take(7) // Only show rows 0-6 (sequencer rows)
                .map(|(i, row)| format!("R{}:{:02}", i, row.current_row_step))
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Process each sequence row (0-indexed)
        for row_idx in 0..state.sequencer_a_row_states.len() {
            // Only process rows 0-6 (sequencer rows, row 7 is control)
            if row_idx > 6 {
                continue;
            }

            if let Some(seq_a_row_state) = state.sequencer_a_row_states.get(row_idx) {
                let row_step = seq_a_row_state.current_row_step;

                // Bounds check: ensure row_step is valid for grid access
                if row_step >= state.sequencer_a_grid.len() {
                    warn!("Row {} row_step {} exceeds grid bounds ({}), skipping",
                          row_idx, row_step, state.sequencer_a_grid.len());
                    continue;
                }

                // Get grid value for this row at current step
                let grid_value = state.sequencer_a_grid[row_step][row_idx];

                // DEBUG: Log grid reads for Row 3
                if row_idx == 3 {
                    info!("🔍 Row 3: master_step={}, row_step={}, max_step={}, grid[{}][{}]={}", 
                          state.sequencer_a_current_master_step, row_step, 
                          seq_a_row_state.max_step, row_step, row_idx, grid_value);
                }

                if grid_value > 0 {
                    // This step is active - send MIDI note
                    // debug!("Trigger: row={}, step={}, value={}", row_idx, current_step, grid_value);


                    // Send MIDI note ON event for this row
                    if let Some(seq_a_row_state) = state.sequencer_a_row_states.get(row_idx) {
                        let midi_event = MidiEvent {
                            note: seq_a_row_state.midi_note,
                            velocity: 100,                // Default velocity
                            channel: seq_a_row_state.midi_channel,
                            note_on: true,
                            step: row_step,
                            sequencer_source: 'A',
                        };

                        if let Err(e) = sender.try_send(SequencerEvent::MidiEvent(midi_event)) {
                            warn!("Failed to send MIDI note ON event: {}", e);
                        }

                        // Schedule note OFF for next tick
                        let note_off = PendingNoteOff {
                            note: seq_a_row_state.midi_note,
                            channel: seq_a_row_state.midi_channel,
                            row: row_idx,
                            target_tick: state.tick_count_since_midi_clock_start + 1,
                        };
                        if let Ok(mut pending) = self.pending_note_offs.lock() {
                            pending.push(note_off);
                        }
                    }
                }

                // Send selective grid update event for this row
                if let Err(e) = sender.try_send(SequencerEvent::GridUpdate {
                    row: row_idx,
                    old_step: seq_a_row_state.previous_row_step,
                    new_step: seq_a_row_state.current_row_step,
                }) {
                    // warn!("Failed to send grid update event: {}", e);
                }
            }
        }

        Ok(())
    }









    /// Save state to file
    pub fn save_state(&self, path: &str) -> Result<()> {
        let state = self.state.lock().unwrap();
        let toml_string = toml::to_string(&*state)?;
        std::fs::write(path, toml_string)?;
        // info!("Sequencer state saved to: {}", path);
        Ok(())
    }

    /// Load state from file
    pub fn load_state(&self, path: &str) -> Result<()> {
        if std::path::Path::new(path).exists() {
            let toml_string = std::fs::read_to_string(path)?;
            let loaded_state: SequencerState = toml::from_str(&toml_string)?;

            let mut state = self.state.lock().unwrap();
            *state = loaded_state;

            // info!("Sequencer state loaded from: {}", path);
        } else {
            // warn!("State file not found: {}, using defaults", path);
        }
        Ok(())
    }

    /// Get a copy of the current grid state for display
    pub fn get_grid_state(&self) -> Vec<Vec<u8>> {
        let state = self.state.lock().unwrap();
        state.sequencer_a_grid.clone()
    }

    /// Get a copy of the current Mozart state for display
    pub fn get_mozart_state(&self) -> Vec<Vec<u8>> {
        let state = self.state.lock().unwrap();
        state.sequencer_a_mozart.clone()
    }



    /// Undo/Redo System Implementation

    /// Push current state to undo stack
    fn push_undo_snapshot(&self, description: String) {
        let state = self.state.lock().unwrap();

        // Clone row states but preserve current step positions
        let mut row_states_snapshot = state.sequencer_a_row_states.clone();
        for seq_a_row_state in &mut row_states_snapshot {
            // Don't save current step positions - these should not be undone
            seq_a_row_state.current_row_step = 0;
            seq_a_row_state.previous_row_step = 0;
        }

        let snapshot = StateSnapshot {
            grid: state.sequencer_a_grid.clone(),
            mozart: state.sequencer_a_mozart.clone(),
            row_states: row_states_snapshot,
            timestamp: std::time::SystemTime::now(),
            description: description.clone(),
        };

        let mut undo_stack = self.undo_stack.lock().unwrap();
        undo_stack.push_back(snapshot);

        // Limit undo history size
        while undo_stack.len() > self.max_undo_history {
            undo_stack.pop_front();
        }

        // Clear redo stack when new action is performed
        let mut redo_stack = self.redo_stack.lock().unwrap();
        redo_stack.clear();

        // debug!("Pushed undo snapshot: {}", description);
    }

    /// Undo last action
    pub fn undo(&self) -> Result<String> {
        let mut undo_stack = self.undo_stack.lock().unwrap();
        let mut redo_stack = self.redo_stack.lock().unwrap();

        if undo_stack.len() <= 1 {
            return Err(anyhow::anyhow!("Nothing to undo"));
        }

        // Push current state to redo stack (preserve positions)
        let current_state = self.state.lock().unwrap();
        let mut current_row_states = current_state.sequencer_a_row_states.clone();
        for seq_a_row_state in &mut current_row_states {
            seq_a_row_state.current_row_step = 0;
            seq_a_row_state.previous_row_step = 0;
        }

        let current_snapshot = StateSnapshot {
            grid: current_state.sequencer_a_grid.clone(),
            mozart: current_state.sequencer_a_mozart.clone(),
            row_states: current_row_states,
            timestamp: std::time::SystemTime::now(),
            description: "Current state before undo".to_string(),
        };

        // Preserve current positions before restoring
        let current_positions: Vec<(usize, usize)> = current_state.sequencer_a_row_states.iter()
            .map(|rs| (rs.current_row_step, rs.previous_row_step))
            .collect();
        drop(current_state);

        redo_stack.push_back(current_snapshot);

        // Pop from undo stack and apply
        if let Some(mut snapshot) = undo_stack.pop_back() {
            let mut state = self.state.lock().unwrap();
            state.sequencer_a_grid = snapshot.grid;
            state.sequencer_a_mozart = snapshot.mozart;

            // Restore row states but keep current positions
            for (i, seq_a_row_state) in snapshot.row_states.iter_mut().enumerate() {
                if i < current_positions.len() {
                    seq_a_row_state.current_row_step = current_positions[i].0;
                    seq_a_row_state.previous_row_step = current_positions[i].1;
                }
            }
            state.sequencer_a_row_states = snapshot.row_states;

            let description = snapshot.description.clone();
            // info!("Undid: {}", description);
            Ok(description)
        } else {
            Err(anyhow::anyhow!("Undo stack corruption"))
        }
    }

    /// Redo last undone action
    pub fn redo(&self) -> Result<String> {
        let mut undo_stack = self.undo_stack.lock().unwrap();
        let mut redo_stack = self.redo_stack.lock().unwrap();

        if redo_stack.is_empty() {
            return Err(anyhow::anyhow!("Nothing to redo"));
        }

        // Push current state to undo stack (preserve positions)
        let current_state = self.state.lock().unwrap();
        let mut current_row_states = current_state.sequencer_a_row_states.clone();
        for seq_a_row_state in &mut current_row_states {
            seq_a_row_state.current_row_step = 0;
            seq_a_row_state.previous_row_step = 0;
        }

        let current_snapshot = StateSnapshot {
            grid: current_state.sequencer_a_grid.clone(),
            mozart: current_state.sequencer_a_mozart.clone(),
            row_states: current_row_states,
            timestamp: std::time::SystemTime::now(),
            description: "State before redo".to_string(),
        };

        // Preserve current positions before restoring
        let current_positions: Vec<(usize, usize)> = current_state.sequencer_a_row_states.iter()
            .map(|rs| (rs.current_row_step, rs.previous_row_step))
            .collect();
        drop(current_state);

        undo_stack.push_back(current_snapshot);

        // Pop from redo stack and apply
        if let Some(mut snapshot) = redo_stack.pop_back() {
            let mut state = self.state.lock().unwrap();
            state.sequencer_a_grid = snapshot.grid;
            state.sequencer_a_mozart = snapshot.mozart;

            // Restore row states but keep current positions
            for (i, seq_a_row_state) in snapshot.row_states.iter_mut().enumerate() {
                if i < current_positions.len() {
                    seq_a_row_state.current_row_step = current_positions[i].0;
                    seq_a_row_state.previous_row_step = current_positions[i].1;
                }
            }
            state.sequencer_a_row_states = snapshot.row_states;

            let description = snapshot.description.clone();
            // info!("Redid: {}", description);
            Ok(description)
        } else {
            Err(anyhow::anyhow!("Redo stack corruption"))
        }
    }

    /// Pattern Management

    /// Save current state as a pattern
    pub fn save_pattern(&self, pattern_id: usize, name: Option<String>) -> Result<()> {
        let state = self.state.lock().unwrap();
        let mut pattern_state = state.clone();

        // Reset transport state for saved patterns
        pattern_state.is_running = false;
        pattern_state.sequencer_a_current_master_step = 0;

        let mut patterns = self.patterns.lock().unwrap();
        patterns.insert(pattern_id, pattern_state);

        let description = name.unwrap_or_else(|| format!("Pattern {}", pattern_id));
        // info!("Saved pattern {}: {}", pattern_id, description);

        Ok(())
    }

    /// Load a pattern
    pub fn load_pattern(&self, pattern_id: usize) -> Result<()> {
        let patterns = self.patterns.lock().unwrap();

        if let Some(pattern_state) = patterns.get(&pattern_id) {
            self.push_undo_snapshot(format!("Load pattern {}", pattern_id));

            let mut state = self.state.lock().unwrap();
            state.sequencer_a_grid = pattern_state.sequencer_a_grid.clone();
            state.sequencer_a_mozart = pattern_state.sequencer_a_mozart.clone();
            state.slide = pattern_state.slide.clone();
            state.sequencer_a_row_states = pattern_state.sequencer_a_row_states.clone();

            // Preserve current transport state
            let current_step = state.sequencer_a_current_master_step;
            let is_running = state.is_running;

            *state = pattern_state.clone();
            state.sequencer_a_current_master_step = current_step;
            state.is_running = is_running;

            // Note: current_pattern tracking would need to be moved to state if needed
            // info!("Loaded pattern {}", pattern_id);

            Ok(())
        } else {
            Err(anyhow::anyhow!("Pattern {} not found", pattern_id))
        }
    }

    /// Get the pattern file path for saving/loading current pattern
    fn get_pattern_file_path() -> PathBuf {
        PathBuf::from("current_pattern.json")
    }

    /// Save current pattern to file
    pub fn save_current_pattern_to_file(&self) -> Result<()> {
        let state = self.state.lock().unwrap();
        let pattern_file = Self::get_pattern_file_path();

        // Create a copy of the state for saving (without transport state)
        let mut save_state = state.clone();
        save_state.is_running = false;
        save_state.sequencer_a_current_master_step = 0;
        save_state.tick_count_since_midi_clock_start = 0;
        save_state.tick_in_step = 0;

        let json_content = serde_json::to_string_pretty(&save_state)?;
        std::fs::write(&pattern_file, json_content)?;

        info!("Current pattern saved to: {:?}", pattern_file);
        Ok(())
    }

    /// Load pattern from specified file path
    pub fn load_pattern_from_file(&self, path: &str) -> Result<()> {
        let pattern_file = std::path::PathBuf::from(path);

        if !pattern_file.exists() {
            return Err(anyhow::anyhow!("Pattern file not found at: {:?}", pattern_file));
        }

        let json_content = std::fs::read_to_string(&pattern_file)?;
        let loaded_state: SequencerState = serde_json::from_str(&json_content)?;

        // Save current transport state before loading
        let mut state = self.state.lock().unwrap();
        // Preserve transport state while loading
        let current_step = state.sequencer_a_current_master_step;
        let is_running = state.is_running;
        let tick_count = state.tick_count_since_midi_clock_start;
        let tick_count_since_step = state.tick_in_step;

        // Load the pattern data
        *state = loaded_state;

        // Restore transport state
        state.sequencer_a_current_master_step = current_step;
        state.is_running = is_running;
        state.tick_count_since_midi_clock_start = tick_count;
        state.tick_in_step = tick_count_since_step;

        // DEBUG: Log Row 3 grid values after loading
        info!("Pattern loaded from: {:?}", pattern_file);
        info!("🔍 LOADED Row 3 grid values: [0]={}, [1]={}, [2]={}, [3]={}", 
              state.sequencer_a_grid[0][3], state.sequencer_a_grid[1][3], 
              state.sequencer_a_grid[2][3], state.sequencer_a_grid[3][3]);

        // Log initial state to formal state logger
        if let Ok(pattern_json) = serde_json::to_string(&*state) {
            log_system_init(&pattern_json);
        }

        Ok(())
    }

    /// Load current pattern from file
    pub fn load_current_pattern_from_file(&self) -> Result<()> {
        let pattern_file = Self::get_pattern_file_path();

        if !pattern_file.exists() {
            return Err(anyhow::anyhow!("No saved pattern found at: {:?}", pattern_file));
        }

        let json_content = std::fs::read_to_string(&pattern_file)?;
        let loaded_state: SequencerState = serde_json::from_str(&json_content)?;

        // Save current transport state before loading
        let mut state = self.state.lock().unwrap();
        // Preserve transport state while loading
        let current_step = state.sequencer_a_current_master_step;
        let is_running = state.is_running;
        let tick_count = state.tick_count_since_midi_clock_start;
        let tick_count_since_step = state.tick_in_step;

        // Load the pattern data
        *state = loaded_state;

        // Restore transport state
        state.sequencer_a_current_master_step = current_step;
        state.is_running = is_running;
        state.tick_count_since_midi_clock_start = tick_count;
        state.tick_in_step = tick_count_since_step;

        // DEBUG: Log Row 3 grid values after loading
        info!("Current pattern loaded from: {:?}", pattern_file);
        info!("🔍 LOADED Row 3 grid values: [0]={}, [1]={}, [2]={}, [3]={}", 
              state.sequencer_a_grid[0][3], state.sequencer_a_grid[1][3], 
              state.sequencer_a_grid[2][3], state.sequencer_a_grid[3][3]);

        // Log initial state to formal state logger
        if let Ok(pattern_json) = serde_json::to_string(&*state) {
            log_system_init(&pattern_json);
        }

        Ok(())
    }

    /// Create a sparse default pattern across both grids
    pub fn create_default_sparse_pattern(&self) {
        let mut state = self.state.lock().unwrap();

        // Create sparse pattern - just a few notes across both grids
        // Main grid (sequencer_a_grid) - add some sparse beats
        if state.sequencer_a_grid.len() >= 16 && state.sequencer_a_grid[0].len() >= 8 {
            // Row 0: Kick pattern - beats 1, 5, 9, 13
            state.sequencer_a_grid[0][0] = 1;  // Step 1
            state.sequencer_a_grid[4][0] = 1;  // Step 5
            state.sequencer_a_grid[8][0] = 1;  // Step 9
            state.sequencer_a_grid[12][0] = 1; // Step 13

            // Row 1: Hi-hat pattern - every 4th step offset
            state.sequencer_a_grid[2][1] = 1;  // Step 3
            state.sequencer_a_grid[6][1] = 1;  // Step 7
            state.sequencer_a_grid[10][1] = 1; // Step 11
            state.sequencer_a_grid[14][1] = 1; // Step 15

            // Row 2: Snare pattern - beats 5, 13
            state.sequencer_a_grid[4][2] = 1;  // Step 5
            state.sequencer_a_grid[12][2] = 1; // Step 13
        }

        // Mozart grid (sequencer_a_mozart) - add some melody notes
        if state.sequencer_a_mozart.len() >= 16 && state.sequencer_a_mozart[0].len() >= 8 {
            // Row 0: Simple melody
            state.sequencer_a_mozart[0][0] = 60;  // C4
            state.sequencer_a_mozart[4][0] = 64;  // E4
            state.sequencer_a_mozart[8][0] = 67;  // G4
            state.sequencer_a_mozart[12][0] = 72; // C5
        }

        info!("Created default sparse pattern across both grids");
    }

    /// Preset Rhythm Patterns

    /// Get preset rhythm pattern by column (0-31)
    pub fn get_preset_pattern(column: usize) -> Vec<u8> {
        match column {
            // Column 0: Clear row
            0 => vec![0; 32], // Clear row (32 steps of silence)
            // Basic Drum Patterns (1-3)
            1 => vec![0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,1,1,0,1,0,1,0,1,0,1], // Techno closed hi-hat with variation (32 steps)
            2 => vec![0,0,0,0,0,0,1,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,1,0,0,0,1,0,0,0,1,0], // Techno open hi-hat with accents (32 steps)
            3 => vec![0,0,0,0,1,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,1,0,0,0], // Techno clap/snare with ghost notes (32 steps)

            // African Rhythms (4-15)
            4 => vec![1,0,1,0,1,0,1,1,0,1,0,1,0,1,1,0,1,0,1,0,1,0,1,1,0,1,0,1,0,1,1,0], // Kagan with variation (32 steps)
            5 => vec![1,0,0,1,0,1,1,0,1,0,1,0,0,1,0,1,1,0,0,1,0,1,1,0,1,0,1,0,0,1,0,1], // Soukous (32 steps)
            6 => vec![1,0,1,1,0,1,0,1,1,0,1,1,0,1,0,0,1,0,1,1,0,1,0,1,1,0,1,1,0,1,0,0], // Bembe extended (32 steps)
            7 => vec![1,0,0,1,0,0,1,0,1,1,0,0,1,0,0,0,1,0,0,1,0,0,1,0,1,1,0,0,1,0,0,0], // Djembe pattern (32 steps)
            8 => vec![1,1,0,1,0,1,1,0,1,0,1,0,1,1,0,0,1,1,0,1,0,1,1,0,1,0,1,0,1,1,0,0], // West African polyrhythm extended (32 steps)
            9 => vec![1,0,1,0,0,1,1,0,1,0,0,1,0,1,1,0,1,0,1,0,0,1,1,0,1,0,0,1,0,1,1,0], // Makossa (32 steps)
            10 => vec![1,0,0,1,0,1,0,1,0,0,1,0,1,0,1,1,1,0,0,1,0,1,0,1,0,0,1,0,1,0,1,1], // Highlife extended (32 steps)
            11 => vec![1,1,0,0,1,0,1,0,1,1,0,0,1,0,0,1,1,1,0,0,1,0,1,0,1,1,0,0,1,0,0,1], // Afrobeat (32 steps)
            12 => vec![1,0,1,1,0,0,1,0,1,0,1,1,0,0,1,0,1,0,1,1,0,0,1,0,1,0,1,1,0,0,1,0], // Ashiko extended (32 steps)
            13 => vec![1,0,0,1,1,0,1,0,0,1,1,0,1,0,0,1,1,0,0,1,1,0,1,0,0,1,1,0,1,0,0,1], // Kpanlogo extended (32 steps)
            14 => vec![1,1,0,1,0,1,0,0,1,1,0,1,0,1,0,0,1,1,0,1,0,1,0,0,1,1,0,1,0,1,0,0], // Agbadza extended (32 steps)
            15 => vec![1,0,1,0,1,1,0,1,0,1,0,1,1,0,1,0,1,0,1,0,1,1,0,1,0,1,0,1,1,0,1,0], // Gahu (32 steps)

            // Salsa Rhythms (16-23)
            16 => vec![1,0,0,1,0,0,1,0,0,1,0,0,1,0,0,0,1,0,0,1,0,0,1,0,0,1,0,0,1,0,0,0], // Son Clave 3-2 (32 steps)
            17 => vec![0,0,1,0,0,1,0,0,0,1,0,0,1,0,0,1,0,0,1,0,0,1,0,0,0,1,0,0,1,0,0,1], // Son Clave 2-3 extended (32 steps)
            18 => vec![1,0,0,1,0,0,1,0,0,0,1,0,0,1,0,0,1,0,0,1,0,0,1,0,0,0,1,0,0,1,0,0], // Rumba Clave 3-2 extended (32 steps)
            19 => vec![0,0,1,0,0,1,0,0,0,0,1,0,0,1,0,0,0,0,1,0,0,1,0,0,0,0,1,0,0,1,0,0], // Rumba Clave 2-3 (32 steps)
            20 => vec![1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0], // Tumbao (32 steps)
            21 => vec![1,1,0,0,1,0,1,1,0,0,1,0,1,1,0,0,1,1,0,0,1,0,1,1,0,0,1,0,1,1,0,0], // Cascara extended (32 steps)
            22 => vec![0,1,0,1,0,0,1,0,1,0,1,0,0,1,0,1,0,1,0,1,0,0,1,0,1,0,1,0,0,1,0,1], // Mambo bell (32 steps)
            23 => vec![1,0,0,1,1,0,1,0,0,1,1,0,1,0,0,1,1,0,0,1,1,0,1,0,0,1,1,0,1,0,0,1], // Cha-cha-cha extended (32 steps)

            // Jazz Rhythms (24-31)
            24 => vec![1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,0], // Jazz ride (32 steps)
            25 => vec![1,0,0,1,1,0,0,1,1,0,0,1,1,0,0,1,1,0,0,1,1,0,0,1,1,0,0,1,1,0,0,1], // Swing shuffle (32 steps)
            26 => vec![0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0], // Jazz snare (32 steps)
            27 => vec![1,1,0,1,1,0,1,1,0,1,1,0,1,1,0,1,1,0,1,1,0,1,1,0], // Brushes pattern (24 steps)
            28 => vec![1,0,1,1,0,1,0,1,1,0,1,0,1,0,1,1,0,1,0,1,1,0,1,0,1,0,1,1,0,1,0,1], // Latin jazz (32 steps)
            29 => vec![1,0,0,0,1,0,1,0,0,0,1,0,1,0,0,0,1,0,0,0,1,0,1,0,0,0,1,0,1,0,0,0], // Bossa nova (32 steps)
            30 => vec![1,1,0,1,0,1,1,0,1,0,1,1,0,1,0,1,1,1,0,1,0,1,1,0,1,0,1,1,0,1,0,1], // Samba (32 steps)
            31 => vec![1,0,1,0,0,1,0,1,0,0,1,0,1,0,0,1,1,0,1,0,0,1,0,1,0,0,1,0,1,0,0,1], // Jazz waltz (32 steps)

            _ => vec![1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,0,0,0], // Default to 4-on-floor (32 steps)
        }
    }

    /// Apply preset pattern to a row
    pub fn apply_preset_pattern(&self, row: usize, column: usize) {
        if row > 6 {
            return;
        }

        self.push_undo_snapshot(format!("Preset pattern column {} on row {}", column, row));

        let pattern = Self::get_preset_pattern(column);
        let pattern_length = pattern.len();
        let mut state = self.state.lock().unwrap();

        // Clear the entire row first
        for col in 0..32 {
            state.sequencer_a_grid[col][row] = 0;
        }

        // Apply the preset pattern
        for (i, &value) in pattern.iter().enumerate() {
            if i < 32 {
                state.sequencer_a_grid[i][row] = value;
            }
        }

        // Set the row's euclidean length to match the pattern length
        if let Some(seq_a_row_state) = state.sequencer_a_row_states.get_mut(row) {
            seq_a_row_state.max_step = pattern_length - 1; // Store as 0-based index
            info!("Applied preset pattern column {} to row {} (pattern length: {}, max_step set to: {})",
                  column, row, pattern_length, seq_a_row_state.max_step);
        } else {
            info!("Applied preset pattern column {} to row {} (pattern length: {})", column, row, pattern_length);
        }
    }

    /// Advanced Sequencing Features

    /// Set global transpose
    pub fn set_global_transpose(&self, semitones: i8) {
        let mut state = self.state.lock().unwrap();
        state.global_transpose = semitones.clamp(-24, 24);
        debug!("Set global transpose: {} semitones", state.global_transpose);
    }

    /// Check if sequencer is in test mode
    pub fn is_test_mode(&self) -> bool {
        self.test_mode
    }

    /// Increment tick count on every MIDI clock tick
    pub fn increment_tick_count(&self) {
        let mut state = self.state.lock().unwrap();
        state.tick_count_since_midi_clock_start += 1;
        state.tick_in_step += 1;
    }

    /// Process pending note-offs on every tick (public interface)
    pub fn process_pending_note_offs_on_tick(&self, sender: &Sender<SequencerEvent>) -> Result<()> {
        let state = self.state.lock().unwrap();
        let current_tick = state.tick_count_since_midi_clock_start;
        drop(state);
        self.process_pending_note_offs(current_tick, sender)
    }

    /// Process clock division resets on every tick
    /// Tick 0: Send Note ON if reset condition met
    /// Tick 1: Send Note OFF for any active resets
    pub fn process_clock_division_resets_on_tick(&self, sender: &Sender<SequencerEvent>) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let tick_in_step = state.tick_in_step;
        let reset_step = state.global_reset_step_counter;
        
        if tick_in_step == 0 {
            // Tick 0: Check conditions and send Note ON
            
            // reset_1 always fires (every step)
            state.reset_1_active = true;
            Self::send_reset_note_on(RESET_1_NOTE, sender)?;
            
            // Nested checks for longer divisions
            if reset_step % 16 == 0 {
                state.reset_16_active = true;
                Self::send_reset_note_on(RESET_16_NOTE, sender)?;
                
                if reset_step % 32 == 0 {
                    state.reset_32_active = true;
                    Self::send_reset_note_on(RESET_32_NOTE, sender)?;
                    
                    if reset_step % 64 == 0 {
                        state.reset_64_active = true;
                        Self::send_reset_note_on(RESET_64_NOTE, sender)?;
                        
                        if reset_step == 0 {
                            state.reset_128_active = true;
                            Self::send_reset_note_on(RESET_128_NOTE, sender)?;
                        }
                    }
                }
            }
            
        } else if tick_in_step == 1 {
            // Tick 1: Send Note OFF for any active resets
            
            if state.reset_1_active {
                Self::send_reset_note_off(RESET_1_NOTE, sender)?;
                state.reset_1_active = false;
            }
            if state.reset_16_active {
                Self::send_reset_note_off(RESET_16_NOTE, sender)?;
                state.reset_16_active = false;
            }
            if state.reset_32_active {
                Self::send_reset_note_off(RESET_32_NOTE, sender)?;
                state.reset_32_active = false;
            }
            if state.reset_64_active {
                Self::send_reset_note_off(RESET_64_NOTE, sender)?;
                state.reset_64_active = false;
            }
            if state.reset_128_active {
                Self::send_reset_note_off(RESET_128_NOTE, sender)?;
                state.reset_128_active = false;
            }
        }
        
        Ok(())
    }

    /// Helper function to send reset note ON
    fn send_reset_note_on(note: u8, sender: &Sender<SequencerEvent>) -> Result<()> {
        let midi_event = MidiEvent {
            note,
            velocity: RESET_VELOCITY,
            channel: RESET_CHANNEL,
            note_on: true,
            step: 0,
            sequencer_source: 'A',
        };
        sender.try_send(SequencerEvent::MidiEvent(midi_event))?;
        Ok(())
    }

    /// Helper function to send reset note OFF
    fn send_reset_note_off(note: u8, sender: &Sender<SequencerEvent>) -> Result<()> {
        let midi_event = MidiEvent {
            note,
            velocity: 0,
            channel: RESET_CHANNEL,
            note_on: false,
            step: 0,
            sequencer_source: 'A',
        };
        sender.try_send(SequencerEvent::MidiEvent(midi_event))?;
        Ok(())
    }

    /// Process pending note-offs that should trigger at current tick
    fn process_pending_note_offs(&self, current_tick: u64, sender: &Sender<SequencerEvent>) -> Result<()> {
        let mut pending = self.pending_note_offs.lock().unwrap();
        
        // Find and send note-offs for current tick
        let mut i = 0;
        while i < pending.len() {
            if pending[i].target_tick <= current_tick {
                let note_off = pending.remove(i);
                let midi_event = MidiEvent {
                    note: note_off.note,
                    velocity: 0,
                    channel: note_off.channel,
                    note_on: false,
                    step: 0, // Not used for note-off
                    sequencer_source: 'A',
                };
                
                if let Err(e) = sender.try_send(SequencerEvent::MidiEvent(midi_event)) {
                    warn!("Failed to send MIDI note OFF event: {}", e);
                }
            } else {
                i += 1;
            }
        }
        
        Ok(())
    }

    /// Set global velocity scale
    pub fn set_global_velocity_scale(&self, scale: f32) {
        let mut state = self.state.lock().unwrap();
        state.global_velocity_scale = scale.clamp(0.1, 2.0);
        debug!(
            "Set global velocity scale: {:.2}",
            state.global_velocity_scale
        );
    }

    /// Randomize specific rows with constraints





    /// Advanced Timing Features

    /// Apply swing to timing (modifies tick timing)
    pub fn apply_swing_timing(&self, step: usize, tick: u32) -> u32 {
        let state = self.state.lock().unwrap();
        if state.swing_amount == 0.0 {
            return tick;
        }

        // Apply swing to off-beats (steps 2, 4, 6, 8, etc.)
        if step % 2 == 0 {
            let swing_offset = (state.swing_amount * state.ticks_per_step as f32 * 0.5) as u32;
            tick + swing_offset
        } else {
            tick
        }
    }

    /// Get effective note with global modifications
    pub fn get_effective_note(&self, base_note: u8, _row: usize) -> u8 {
        let state = self.state.lock().unwrap();
        let mut note = base_note as i16;

        // Apply global transpose
        note += state.global_transpose as i16;

        // Clamp to valid MIDI range
        note.clamp(0, 127) as u8
    }

    /// Get effective velocity with global modifications
    pub fn get_effective_velocity(&self, base_velocity: u8, _row: usize) -> u8 {
        let state = self.state.lock().unwrap();
        let mut velocity = base_velocity as f32;

        // Apply global velocity scale
        velocity *= state.global_velocity_scale;

        // Clamp to valid velocity range
        velocity.clamp(1.0, 127.0) as u8
    }

    /// Euclidean rhythm generation
    pub fn generate_euclidean_rhythm(
        &self,
        row: usize,
        mut events: usize,
        length: usize,
        rotation: usize,
    ) {
        if row > 6 || length > 32 {
            return;
        }

        // Auto-reduce events if they exceed the new length
        if events > length {
            events = length;
            info!("generate_euclidean_rhythm says: Auto-reduced events from {} to {} to match length {}",
                  events, length, length);
        }

        self.push_undo_snapshot(format!(
            "Euclidean rhythm R{}: {} events in {} length",
            row, events, length
        ));

        let mut state = self.state.lock().unwrap();
        let row_idx = row;

        // Update the row's Euclidean parameters
        state.sequencer_a_row_states[row_idx].euclidean_events = events;
        state.sequencer_a_row_states[row_idx].max_step = length - 1; // Store as last valid step index (0-based)
        state.sequencer_a_row_states[row_idx].euclidean_rotation = rotation;

        // Clear the row first
        for col in 0..32 {
            state.sequencer_a_grid[col][row_idx] = 0;
        }

        // Generate euclidean rhythm using Bresenham's algorithm
        let mut bucket = 0;
        for i in 0..length {
            bucket += events;
            if bucket >= length {
                bucket -= length;
                let pos = (i + rotation) % length;
                if pos < 32 {
                    state.sequencer_a_grid[pos][row_idx] = 1;
                }
            }
        }

        // Create binary pattern string for logging
        let mut pattern = String::new();
        let mut patterns_beyond_15 = Vec::new();
        for i in 0..length.min(32) {
            let has_pattern = state.sequencer_a_grid[i][row_idx] == 1;
            pattern.push(if has_pattern { '1' } else { '0' });

            // Track patterns beyond step 15
            if i > 15 && has_pattern {
                patterns_beyond_15.push(i);
            }
        }

        info!("generate_euclidean_rhythm says: Generated euclidean rhythm for row {}: {} events in {} length, rotation {}, pattern: {}",
              row, events, length, rotation, pattern);

        if !patterns_beyond_15.is_empty() {
            info!(
                "DEBUG: Euclidean patterns created beyond position 15 at positions: {:?}",
                patterns_beyond_15
            );
        }
    }

    /// Copy pattern section
    pub fn copy_section(
        &self,
        src_x: usize,
        src_y: usize,
        width: usize,
        height: usize,
        dest_x: usize,
        dest_y: usize,
    ) {
        self.push_undo_snapshot(format!(
            "Copy section {}x{} from ({},{}) to ({},{})",
            width, height, src_x, src_y, dest_x, dest_y
        ));

        let mut state = self.state.lock().unwrap();

        for x in 0..width {
            for y in 0..height {
                let src_col = src_x + x;
                let src_row = src_y + y;
                let dest_col = dest_x + x;
                let dest_row = dest_y + y;

                if src_col < 32 && src_row < 8 && dest_col < 32 && dest_row < 8 {
                    state.sequencer_a_grid[dest_col][dest_row] = state.sequencer_a_grid[src_col][src_row];
                    state.sequencer_a_mozart[dest_col][dest_row] = state.sequencer_a_mozart[src_col][src_row];
                    state.slide.slide_grid[dest_col][dest_row] =
                        state.slide.slide_grid[src_col][src_row];
                }
            }
        }

        info!(
            "Copied {}x{} section from ({},{}) to ({},{})",
            width, height, src_x, src_y, dest_x, dest_y
        );
    }

    /// Set row states
    pub fn set_row_states(&self, row: usize, states: SequencerARowStates) {
        if row < 7 {
            let mut state = self.state.lock().unwrap();
            state.sequencer_a_row_states[row] = states;
            debug!("set_row_states says: Updated states for row {}", row);
        }
    }

    /// Get row states
    pub fn get_row_states(&self, row: usize) -> Option<SequencerARowStates> {
        if row < 7 {
            let state = self.state.lock().unwrap();
            Some(state.sequencer_a_row_states[row].clone())
        } else {
            None
        }
    }

    /// Get current global transpose
    pub fn get_global_transpose(&self) -> i8 {
        let state = self.state.lock().unwrap();
        state.global_transpose
    }



    /// Check if tick counter reset flag is set (for testing/debugging)
    pub fn get_reset_tick_counter_flag(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.reset_tick_counter
    }

    /// External clock step advancement - locks state and calls advance_step
    pub fn external_advance_step(&mut self, sender: &Sender<SequencerEvent>) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        self.advance_step(&mut state, sender)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequencer_creation() {
        let sequencer = Sequencer::new();
        assert!(!sequencer.is_running());

        let step = sequencer.get_position();
        assert_eq!(step, 0);
    }

    #[test]
    fn test_grid_operations() {
        let sequencer = Sequencer::new();

        // Test setting and getting grid values
        sequencer.set_grid_value(0, 0, 1);
        assert_eq!(sequencer.get_grid_value(0, 0), 1);

        sequencer.set_grid_value(15, 6, 2);
        assert_eq!(sequencer.get_grid_value(15, 6), 2);

        // Test bounds checking
        sequencer.set_grid_value(32, 8, 1); // Should be ignored
        assert_eq!(sequencer.get_grid_value(32, 8), 0);
    }

    #[test]
    fn test_transport_control() {
        let sequencer = Sequencer::new();

        assert!(!sequencer.is_running());

        sequencer.start();
        assert!(sequencer.is_running());

        sequencer.stop();
        assert!(!sequencer.is_running());

        // Position should reset after stop
        let step = sequencer.get_position();
        assert_eq!(step, 0);
    }

    #[test]
    fn test_tempo_control() {
        let sequencer = Sequencer::new();

        sequencer.set_tempo(140.0);
        let state = sequencer.state.lock().unwrap();
        assert_eq!(state.tempo, 140.0);

        // Test clamping
        drop(state);
        sequencer.set_tempo(350.0);
        let state = sequencer.state.lock().unwrap();
        assert_eq!(state.tempo, 300.0); // Should be clamped to max
    }

    #[test]
    fn test_step_events() {
        let sequencer = Sequencer::new();

        // Set a trigger on step 0, row 0
        sequencer.set_grid_value(0, 0, 1);

        // Should get note events for this step
        let events = sequencer.get_step_events(0, 0, 0);
        assert!(events.is_some());
    }

    #[test]
    fn test_tick_counter_reset() {
        let sequencer = Sequencer::new();

        // Clear any initial state by stopping first
        sequencer.stop();

        // Verify basic state after stop
        {
            // Position should remain at beginning
            let state = sequencer.state.lock().unwrap();
            assert_eq!(state.sequencer_a_current_master_step, 0);
        }


        // Start should set the reset flag
        sequencer.start();
        {
            let state = sequencer.state.lock().unwrap();
            assert!(state.reset_tick_counter);
            assert!(state.is_running);
        }

        // Stop should reset positions and clear running flag
        sequencer.stop();
        {
            let state = sequencer.state.lock().unwrap();
            assert_eq!(state.sequencer_a_current_master_step, 0);
        }

        // Starting again should set reset flag again
        sequencer.start();
        {
            let state = sequencer.state.lock().unwrap();
            assert!(state.reset_tick_counter);
            assert!(state.is_running);
        }
    }

    #[test]
    fn test_step_events_completion() {
        let sequencer = Sequencer::new();
        sequencer.set_grid_value(0, 0, 1);
        let events = sequencer.get_step_events(0, 0, 0);
        let events = events.unwrap();
        assert!(!events.is_empty());

        // Should have note on and note off events
        let note_on_events: Vec<_> = events.iter().filter(|e| e.note_on).collect();
        let note_off_events: Vec<_> = events.iter().filter(|e| !e.note_on).collect();

        assert!(!note_on_events.is_empty());
        assert!(!note_off_events.is_empty());
    }

    #[test]
    fn test_individual_row_step_counters_reset() {
        let sequencer = Sequencer::new();

        // Simulate advancing individual row counters by starting and letting them advance
        sequencer.start();

        // Manually advance row states to simulate playback
        {
            let mut state = sequencer.state.lock().unwrap();
            // Simulate some rows having advanced to different positions
            if state.sequencer_a_row_states.len() >= 8 {
                state.sequencer_a_row_states[0].current_row_step = 5;
                state.sequencer_a_row_states[1].current_row_step = 8;
                state.sequencer_a_row_states[2].current_row_step = 12;
                state.sequencer_a_row_states[3].current_row_step = 3;
                state.sequencer_a_row_states[4].current_row_step = 15;
                state.sequencer_a_row_states[5].current_row_step = 7;
                state.sequencer_a_row_states[6].current_row_step = 20;
                state.sequencer_a_row_states[7].current_row_step = 11;
            }
        }

        // Verify rows are at different positions
        {
            let state = sequencer.state.lock().unwrap();
            if state.sequencer_a_row_states.len() >= 8 {
                assert_eq!(state.sequencer_a_row_states[0].current_row_step, 5);
                assert_eq!(state.sequencer_a_row_states[1].current_row_step, 8);
                assert_eq!(state.sequencer_a_row_states[2].current_row_step, 12);
                assert_eq!(state.sequencer_a_row_states[7].current_row_step, 11);
            }
        }

        // Stop should reset ALL individual row counters to 0
        sequencer.stop();

        // Verify all row counters are reset
        {
            let state = sequencer.state.lock().unwrap();
            for (row_idx, seq_a_row_state) in state.sequencer_a_row_states.iter().enumerate() {
                assert_eq!(seq_a_row_state.current_row_step, 0,
                    "Row {} counter should be reset to 0 after stop", row_idx);
            }
        }

        // Starting again should still have all counters at 0
        sequencer.start();
        {
            let state = sequencer.state.lock().unwrap();
            for (row_idx, seq_a_row_state) in state.sequencer_a_row_states.iter().enumerate() {
                assert_eq!(seq_a_row_state.current_row_step, 0,
                    "Row {} counter should remain at 0 after restart", row_idx);
            }
        }
    }

    #[test]
    fn test_row_step_counter_values_and_ranges() {
        let sequencer = Sequencer::new();

        // Check default values
        {
            let state = sequencer.state.lock().unwrap();
            for (row_idx, seq_a_row_state) in state.sequencer_a_row_states.iter().enumerate() {
                assert_eq!(seq_a_row_state.current_row_step, 0, "Row {} should start at step 0", row_idx);
                assert_eq!(seq_a_row_state.first_step, 0, "Row {} should have first_step=0", row_idx);
                assert_eq!(seq_a_row_state.max_step, 31, "Row {} should have max_step=31 by default", row_idx);
                assert_eq!(seq_a_row_state.previous_row_step, 31, "Row {} should have previous_step=31", row_idx);
            }
        }

        // Simulate step advancement to show the actual range
        sequencer.start();
        {
            let mut state = sequencer.state.lock().unwrap();
            let row = &mut state.sequencer_a_row_states[0];

            // Simulate advancing through the full range
            for expected_step in 0..=31 {
                assert_eq!(row.current_row_step, expected_step,
                    "Row step counter should be {} at position {}", expected_step, expected_step);

                // Advance step (simulating the advance_step logic)
                row.previous_row_step = row.current_row_step;
                row.current_row_step += 1;
                if row.current_row_step > row.max_step {
                    row.current_row_step = row.first_step;
                }
            }

            // After 32 steps (0-31), should wrap back to 0
            assert_eq!(row.current_row_step, 0, "Should wrap back to 0 after step 31");
        }

        // Test with custom range
        {
            let mut state = sequencer.state.lock().unwrap();
            let row = &mut state.sequencer_a_row_states[1];

            // Set custom range: steps 4-15 (12 step loop starting at step 4)
            row.first_step = 4;
            row.max_step = 15;
            row.current_row_step = 4;

            // Advance through custom range
            for i in 0..20 { // Test more than one full cycle
                let expected = if i <= 11 { 4 + i } else { 4 + ((i - 12) % 12) };
                assert_eq!(row.current_row_step, expected,
                    "Custom range: step {} should be at position {}", i, expected);

                // Advance
                row.previous_row_step = row.current_row_step;
                row.current_row_step += 1;
                if row.current_row_step > row.max_step {
                    row.current_row_step = row.first_step;
                }
            }
        }
    }
}
