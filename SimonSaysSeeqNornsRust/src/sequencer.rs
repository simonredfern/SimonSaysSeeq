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
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// MIDI gate base note - matches Lua version
/// This works well with Flame MGTV factory default settings.
/// http://flame.fortschritt-musik.de/pdf/Manual_Flame_MGTV_module_v100_eng.pdf
const LOWEST_MIDI_NOTE_NUMBER_FOR_GATE: u8 = 47;

/// MIDI event for hardware output
#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub note: u8,
    pub velocity: u8,
    pub channel: u8,
    pub note_on: bool,
    pub step: usize,
    pub bar: usize,
    pub sequencer_source: char, // 'A' for sequencer_a
}

/// Events that the sequencer can send to the main application
#[derive(Debug, Clone)]
pub enum SequencerEvent {
    /// A step has been reached
    Step { step: usize, bar: usize },
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
    pub sequencer_a_current_step: usize,
    pub sequencer_a_first_step: usize,
    pub sequencer_a_euclidean_length: usize,
    pub sequencer_a_euclidean_events: usize,
    pub sequencer_a_euclidean_rotation: usize,
    pub sequencer_a_previous_step: usize,
    pub sequencer_a_midi_note: u8,
    pub sequencer_a_midi_velocity: u8,
    pub sequencer_a_midi_channel: u8,
    pub sequencer_a_ratchet_count: u8,
}

/// MIDI note event for recording and playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiNoteEvent {
    pub velocity: u8,
    pub tick_count_since_step: u32,
    pub is_active: bool,
    pub tick_count_since_start: u64,
}

impl Default for MidiNoteEvent {
    fn default() -> Self {
        Self {
            velocity: 0,
            tick_count_since_step: 0,
            is_active: false,
            tick_count_since_start: 0,
        }
    }
}

/// Core timing and tempo analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoAnalysis {
    // Wow (large tempo instability)
    pub wow_window_size: usize,
    pub wow_window_tick_position: usize,
    pub wow_tempo_sum: f32,
    pub total_wow_tempo_ticks: u32,
    pub wow_tempo_episodes: u32,
    pub wow_average_tempo: f32,
    pub tempo_wow_is_good: bool,
    pub wow_threshold: f32,

    // Flutter (small tempo instability)
    pub flutter_window_size: usize,
    pub flutter_window_tick_position: usize,
    pub flutter_tempo_sum: f32,
    pub total_flutter_tempo_ticks: u32,
    pub flutter_tempo_episodes: u32,
    pub flutter_average_tempo: f32,
    pub tempo_flutter_is_good: bool,
    pub flutter_threshold: f32,
}

impl Default for TempoAnalysis {
    fn default() -> Self {
        Self {
            wow_window_size: 192,
            wow_window_tick_position: 0,
            wow_tempo_sum: 0.0,
            total_wow_tempo_ticks: 0,
            wow_tempo_episodes: 0,
            wow_average_tempo: 0.0,
            tempo_wow_is_good: true,
            wow_threshold: 3.0,

            flutter_window_size: 192,
            flutter_window_tick_position: 0,
            flutter_tempo_sum: 0.0,
            total_flutter_tempo_ticks: 0,
            flutter_tempo_episodes: 0,
            flutter_average_tempo: 0.0,
            tempo_flutter_is_good: true,
            flutter_threshold: 0.25,
        }
    }
}

impl Default for SequencerARowStates {
    fn default() -> Self {
        Self {
            sequencer_a_current_step: 0,
            sequencer_a_first_step: 0,
            sequencer_a_euclidean_length: 31,
            sequencer_a_euclidean_events: 5,
            sequencer_a_euclidean_rotation: 0,
            sequencer_a_previous_step: 31,
            sequencer_a_midi_note: LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + 1, // Default to first gate note (48)
            sequencer_a_midi_velocity: 100,
            sequencer_a_midi_channel: 1,
            sequencer_a_ratchet_count: 1,
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
    pub sequencer_a_current_master_bar: usize,
    pub sequencer_a_current_lane: usize,
    /// Transport state
    pub is_running: bool,
    /// Core timing variables
    pub tempo: f32,
    pub swing_amount: f32,
    pub ticks_per_step: u32,
    pub steps_per_bar: usize,
    pub tick_count: u64,
    pub the_current_tick_count_since_step: u32,
    pub the_current_tick_count_since_start: u64,
    pub first_step: usize,
    pub last_step: usize,
    pub midi_first_step: usize,
    pub midi_last_step: usize,
    /// Tempo analysis
    pub tempo_analysis: TempoAnalysis,
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
    pub min_bar: usize,
    pub max_bar: usize,
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
        const MIN_BAR: usize = 0;
        const MAX_BAR: usize = 3;
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
            let mut row_state = SequencerARowStates::default();
            // Match Lua version: LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + sequence_row (1-based)
            // Lua uses sequence_row 1-7, Rust uses i 0-6, so add 1 to convert
            row_state.sequencer_a_midi_note = LOWEST_MIDI_NOTE_NUMBER_FOR_GATE + (i + 1) as u8;
            row_state.sequencer_a_midi_channel = 1; // All rows use MIDI channel 1
            row_states.push(row_state);
        }



        Self {
            sequencer_a_grid: grid,
            sequencer_a_mozart: mozart,
            slide,
            held,
            sequencer_a_row_states: row_states,
            sequencer_a_current_master_step: 0,
            sequencer_a_current_master_bar: 0,
            sequencer_a_current_lane: 1,
            is_running: false,
            tempo: 30.0,
            swing_amount: 0.0,
            ticks_per_step: 12,
            first_step: 0,
            last_step: 31,
            steps_per_bar: 16,
            tick_count: 0,
            the_current_tick_count_since_step: 0,
            the_current_tick_count_since_start: 0,
            midi_first_step: 0,
            midi_last_step: 31,
            tempo_analysis: TempoAnalysis::default(),
            swing_mode: 1,
            global_transpose: 0,
            global_velocity_scale: 1.0,
            total_step_co2_count: 1,
            total_tick_co2_count: 1,
            total_sequence_rows: TOTAL_SEQUENCE_ROWS,
            cols: COLS,
            rows: ROWS,
            min_bar: MIN_BAR,
            max_bar: MAX_BAR,
            min_lane: MIN_LANE,
            max_lane: MAX_LANE,
            max_step: MAX_STEP,
            reset_tick_counter: false,
        }
    }
}

/// Main sequencer engine
#[derive(Clone)]
pub struct Sequencer {
    state: Arc<std::sync::Mutex<SequencerState>>,
    note_events: Arc<std::sync::Mutex<HashMap<(usize, usize, usize, u8), NoteEvent>>>, // (lane, bar, step, note) -> event
    /// Undo/Redo system
    undo_stack: Arc<std::sync::Mutex<VecDeque<StateSnapshot>>>,
    redo_stack: Arc<std::sync::Mutex<VecDeque<StateSnapshot>>>,
    max_undo_history: usize,
    /// Pattern storage (multiple patterns)
    patterns: Arc<std::sync::Mutex<HashMap<usize, SequencerState>>>,
    current_pattern: usize,
}

impl Sequencer {
    pub fn new() -> Self {
        let sequencer = Self {
            state: Arc::new(std::sync::Mutex::new(SequencerState::default())),
            note_events: Arc::new(std::sync::Mutex::new(HashMap::new())),
            undo_stack: Arc::new(std::sync::Mutex::new(VecDeque::new())),
            redo_stack: Arc::new(std::sync::Mutex::new(VecDeque::new())),
            max_undo_history: 50,
            patterns: Arc::new(std::sync::Mutex::new(HashMap::new())),
            current_pattern: 0,
        };

        // Try to load saved pattern, or create default sparse pattern if none exists
        match sequencer.load_current_pattern_from_file() {
            Ok(()) => {
                info!("Loaded saved pattern on startup");
            }
            Err(_) => {
                info!("No saved pattern found, creating default sparse pattern");
                sequencer.create_default_sparse_pattern();
            }
        }

        // Initialize with a snapshot for undo/redo
        sequencer.push_undo_snapshot("Initial state".to_string());

        sequencer
    }

    /// Start the sequencer
    pub fn start(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.is_running {
            state.is_running = true;
            state.reset_tick_counter = true;


            // Reset to beginning
            state.sequencer_a_current_master_step = 0;
            state.sequencer_a_current_master_bar = 0;
            for row_state in &mut state.sequencer_a_row_states {
                row_state.sequencer_a_current_step = 0;
            }


            info!("Sequencer started");
        }
    }

    /// Stop the sequencer
    pub fn stop(&self) {
        let mut state = self.state.lock().unwrap();
        if state.is_running {
            state.is_running = false;
            // Reset to beginning
            state.sequencer_a_current_master_step = 0;
            state.sequencer_a_current_master_bar = 0;
            for row_state in &mut state.sequencer_a_row_states {
                row_state.sequencer_a_current_step = 0;
            }
            // Log the stack trace to identify what triggered the stop
            let trace = std::backtrace::Backtrace::capture();
            info!("Sequencer stopped and reset - Stack trace: {}", trace);
            
            // Save current pattern when stopping
            drop(state); // Release the lock before calling save method
            if let Err(e) = self.save_current_pattern_to_file() {
                warn!("Failed to save current pattern on stop: {}", e);
            }
        }
    }

    /// Check if sequencer is running
    pub fn is_running(&self) -> bool {
        self.state.lock().unwrap().is_running
    }

    /// Set tempo
    pub fn set_tempo(&self, tempo: f32) {
        let mut state = self.state.lock().unwrap();
        state.tempo = tempo.clamp(20.0, 300.0);
        debug!("set_tempo says: Tempo set to: {:.1} BPM", state.tempo);
    }

    /// Get current position
    /// Get the current playback position (step, bar)
    pub fn get_position(&self) -> (usize, usize) {
        let state = self.state.lock().unwrap();
        (state.sequencer_a_current_master_step, state.sequencer_a_current_master_bar)
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
            debug!("Set mozart[{}][{}] = {}", x, y, clamped_note);
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

    /// Get current tempo
    pub fn get_tempo(&self) -> f32 {
        let state = self.state.lock().unwrap();
        state.tempo
    }

    /// Get current step and bar
    pub fn get_current_position(&self) -> (usize, usize) {
        let state = self.state.lock().unwrap();
        (state.sequencer_a_current_master_step, state.sequencer_a_current_master_bar)
    }



    /// Clear a specific section of the grid
    pub fn clear_section(&self, start_x: usize, start_y: usize, width: usize, height: usize) {
        let mut state = self.state.lock().unwrap();

        for x in start_x..=(start_x + width - 1).min(31) {
            for y in start_y..=(start_y + height - 1).min(7) {
                if x < 32 && y < 8 {
                    state.sequencer_a_grid[x][y] = 0;
                }
            }
        }
    }



    /// Set first step for sequencer
    pub fn set_first_step(&self, step: usize) {
        let mut state = self.state.lock().unwrap();
        state.first_step = step.clamp(0, 31);
    }

    /// Set last step for sequencer
    pub fn set_last_step(&self, step: usize) {
        let mut state = self.state.lock().unwrap();
        state.last_step = step.clamp(0, 31);
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

    /// Set pattern change mode
    pub fn set_pattern_change_mode(&self, mode: bool) {
        let _state = self.state.lock().unwrap();
        // Store pattern change mode in a custom field if needed
        // For now, just log it
        info!("Pattern change mode set to: {}", mode);
    }

    /// Set held state for button combinations
    pub fn set_held_state(&self, x: usize, y: usize, held: bool) {
        if x > 0 && x <= 32 && y > 0 && y <= 8 {
            let mut state = self.state.lock().unwrap();
            state.held[x - 1][y - 1] = if held { 1 } else { 0 };
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

    /// Reset all sequences
    pub fn reset_all(&self) {
        let mut state = self.state.lock().unwrap();

        // Clear grid
        for row in &mut state.sequencer_a_grid {
            for cell in row {
                *cell = 0;
            }
        }
        
        // Reset position
        state.sequencer_a_current_master_step = 0;
        state.sequencer_a_current_master_bar = 0;
        for row_state in &mut state.sequencer_a_row_states {
            row_state.sequencer_a_current_step = 0;
            row_state.sequencer_a_previous_step = 0;
        }

        info!("All sequences reset");
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

        let row_state = &state.sequencer_a_row_states[row];

        // Check if this step is within the row's range
        if step < row_state.sequencer_a_first_step || step > row_state.sequencer_a_euclidean_length {
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
                note: row_state.sequencer_a_midi_note,
                velocity: row_state.sequencer_a_midi_velocity,
                channel: row_state.sequencer_a_midi_channel,
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
                note: row_state.sequencer_a_midi_note,
                velocity: 0,
                channel: row_state.sequencer_a_midi_channel,
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
        state: &mut SequencerState,
        sender: &Sender<SequencerEvent>,
    ) -> Result<()> {
        // Reset tick count since step
        state.the_current_tick_count_since_step = 0;

        // Store current step values BEFORE increment for MIDI synchronization
        let current_step = state.sequencer_a_current_master_step;
        let current_bar = state.sequencer_a_current_master_bar;

        // Process current step for all sequence rows
        self.process_step(&*state, sender)?;

        // Send step event with CURRENT step values (before increment) for MIDI sync
        let _ = sender.try_send(SequencerEvent::Step {
            step: current_step,
            bar: current_bar,
        });

        // Advance sequencer master step
        state.sequencer_a_current_master_step += 1;
        if state.sequencer_a_current_master_step > state.last_step {
            state.sequencer_a_current_master_step = state.first_step;

            // Advance master bar
            state.sequencer_a_current_master_bar += 1;
            if state.sequencer_a_current_master_bar > state.max_bar {
                state.sequencer_a_current_master_bar = state.min_bar;
            }
        }
        
        // Advance each row's current step based on its individual settings
        for (row_idx, row_state) in state.sequencer_a_row_states.iter_mut().enumerate() {
            let old_step = row_state.sequencer_a_current_step;
            row_state.sequencer_a_previous_step = old_step;
            row_state.sequencer_a_current_step += 1;
            if row_state.sequencer_a_current_step > row_state.sequencer_a_euclidean_length {
                row_state.sequencer_a_current_step = row_state.sequencer_a_first_step;
            }
            if row_idx <= 6 { // Debug all 7 sequencer rows (0-indexed)
                 // debug!("🎯 Row {} step advancement: {} -> {} (range: {}-{})",
                 //       row_idx, old_step, row_state.sequencer_a_current_step,
                 //       row_state.sequencer_a_first_step, row_state.sequencer_a_euclidean_length);
            }
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
        // DEBUG: Show row 0 current step
        if let Some(row_state) = state.sequencer_a_row_states.get(0) {
            info!("🎯 Row 0: step {}", row_state.sequencer_a_current_step);
        }
        
        // Process each sequence row (0-indexed)
        for row_idx in 0..state.sequencer_a_row_states.len() {
            // Only process rows 0-6 (sequencer rows, row 7 is control)
            if row_idx > 6 {
                continue;
            }

            if let Some(row_state) = state.sequencer_a_row_states.get(row_idx) {
                let current_step = row_state.sequencer_a_current_step;

                // Get grid value for this row at current step
                if current_step < state.cols {
                    let grid_value = state.sequencer_a_grid[current_step][row_idx];

                    if grid_value > 0 {
                        // This step is active - send MIDI note
                        // debug!("Trigger: row={}, step={}, value={}", row_idx, current_step, grid_value);
                        
                        // DEBUG: Track MIDI step position for row 0
                        // if row_idx == 0 {
                        //     info!("🎵 MIDI DEBUG Row 0: Sending MIDI note at step {} (grid_value={})", current_step, grid_value);
                        // }

                        // Send MIDI note ON event for this row
                        if let Some(row_state) = state.sequencer_a_row_states.get(row_idx) {
                            let midi_event = MidiEvent {
                                note: row_state.sequencer_a_midi_note,
                                velocity: 100,                // Default velocity
                                channel: (row_idx + 1) as u8, // Row-based channel 1-7
                                note_on: true,
                                step: current_step,
                                bar: state.sequencer_a_current_master_bar,
                                sequencer_source: 'A',
                            };

                            if let Err(e) = sender.try_send(SequencerEvent::MidiEvent(midi_event)) {
                                warn!("Failed to send MIDI note ON event: {}", e);
                            }
                        }
                    }
                }

                // Send selective grid update event for this row
                // DEBUG: Track LED step position for row 0
                // if row_idx == 0 {
                //     info!("💡 LED DEBUG Row 0: Sending grid update old_step={} -> new_step={}", 
                //           row_state.sequencer_a_previous_step, current_step);
                // }
                
                if let Err(e) = sender.try_send(SequencerEvent::GridUpdate {
                    row: row_idx,
                    old_step: row_state.sequencer_a_previous_step,
                    new_step: current_step,
                }) {
                    warn!("Failed to send grid update event: {}", e);
                }
            }
        }

        Ok(())
    }





    /// Toggle grid position (used for pattern editing)
    pub fn toggle_grid_position(&self, x: usize, y: usize) {
        if x > 0 && x <= 32 && y > 0 && y <= 8 {
            let mut state = self.state.lock().unwrap();
            let current_value = state.sequencer_a_grid[x - 1][y - 1];
            let new_value = if current_value == 0 { 1 } else { 0 };
            state.sequencer_a_grid[x - 1][y - 1] = new_value;
            debug!(
                "toggle_grid_position says: Toggled grid[{}][{}]: {} -> {}",
                x, y, current_value, new_value
            );
        }
    }



    /// Save state to file
    pub fn save_state(&self, path: &str) -> Result<()> {
        let state = self.state.lock().unwrap();
        let toml_string = toml::to_string(&*state)?;
        std::fs::write(path, toml_string)?;
        info!("Sequencer state saved to: {}", path);
        Ok(())
    }

    /// Load state from file
    pub fn load_state(&self, path: &str) -> Result<()> {
        if std::path::Path::new(path).exists() {
            let toml_string = std::fs::read_to_string(path)?;
            let loaded_state: SequencerState = toml::from_str(&toml_string)?;

            let mut state = self.state.lock().unwrap();
            *state = loaded_state;

            info!("Sequencer state loaded from: {}", path);
        } else {
            warn!("State file not found: {}, using defaults", path);
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
        for row_state in &mut row_states_snapshot {
            // Don't save current step positions - these should not be undone
            row_state.sequencer_a_current_step = 0;
            row_state.sequencer_a_previous_step = 0;
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

        debug!("Pushed undo snapshot: {}", description);
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
        for row_state in &mut current_row_states {
            row_state.sequencer_a_current_step = 0;
            row_state.sequencer_a_previous_step = 0;
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
            .map(|rs| (rs.sequencer_a_current_step, rs.sequencer_a_previous_step))
            .collect();
        drop(current_state);

        redo_stack.push_back(current_snapshot);

        // Pop from undo stack and apply
        if let Some(mut snapshot) = undo_stack.pop_back() {
            let mut state = self.state.lock().unwrap();
            state.sequencer_a_grid = snapshot.grid;
            state.sequencer_a_mozart = snapshot.mozart;
            
            // Restore row states but keep current positions
            for (i, row_state) in snapshot.row_states.iter_mut().enumerate() {
                if i < current_positions.len() {
                    row_state.sequencer_a_current_step = current_positions[i].0;
                    row_state.sequencer_a_previous_step = current_positions[i].1;
                }
            }
            state.sequencer_a_row_states = snapshot.row_states;

            let description = snapshot.description.clone();
            info!("Undid: {}", description);
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
        for row_state in &mut current_row_states {
            row_state.sequencer_a_current_step = 0;
            row_state.sequencer_a_previous_step = 0;
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
            .map(|rs| (rs.sequencer_a_current_step, rs.sequencer_a_previous_step))
            .collect();
        drop(current_state);

        undo_stack.push_back(current_snapshot);

        // Pop from redo stack and apply
        if let Some(mut snapshot) = redo_stack.pop_back() {
            let mut state = self.state.lock().unwrap();
            state.sequencer_a_grid = snapshot.grid;
            state.sequencer_a_mozart = snapshot.mozart;
            
            // Restore row states but keep current positions
            for (i, row_state) in snapshot.row_states.iter_mut().enumerate() {
                if i < current_positions.len() {
                    row_state.sequencer_a_current_step = current_positions[i].0;
                    row_state.sequencer_a_previous_step = current_positions[i].1;
                }
            }
            state.sequencer_a_row_states = snapshot.row_states;

            let description = snapshot.description.clone();
            info!("Redid: {}", description);
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
        pattern_state.sequencer_a_current_master_bar = 0;

        let mut patterns = self.patterns.lock().unwrap();
        patterns.insert(pattern_id, pattern_state);

        let description = name.unwrap_or_else(|| format!("Pattern {}", pattern_id));
        info!("Saved pattern {}: {}", pattern_id, description);

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

            // Keep current transport state
            let current_step = state.sequencer_a_current_master_step;
            let current_bar = state.sequencer_a_current_master_bar;
            let is_running = state.is_running;

            *state = pattern_state.clone();
            state.sequencer_a_current_master_step = current_step;
            state.sequencer_a_current_master_bar = current_bar;
            state.is_running = is_running;

            // Note: current_pattern tracking would need to be moved to state if needed
            info!("Loaded pattern {}", pattern_id);

            Ok(())
        } else {
            Err(anyhow::anyhow!("Pattern {} not found", pattern_id))
        }
    }

    /// Get the pattern file path for saving/loading current pattern
    fn get_pattern_file_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            let dir = config_dir.join("simon-says-seeq");
            std::fs::create_dir_all(&dir).ok(); // Create directory if it doesn't exist
            dir.join("current_pattern.json")
        } else {
            PathBuf::from("simon_says_seeq_current_pattern.json")
        }
    }

    /// Save current pattern to file
    pub fn save_current_pattern_to_file(&self) -> Result<()> {
        let state = self.state.lock().unwrap();
        let pattern_file = Self::get_pattern_file_path();
        
        // Create a copy of the state for saving (without transport state)
        let mut save_state = state.clone();
        save_state.is_running = false;
        save_state.sequencer_a_current_master_step = 0;
        save_state.sequencer_a_current_master_bar = 0;
        save_state.tick_count = 0;
        save_state.the_current_tick_count_since_step = 0;
        save_state.the_current_tick_count_since_start = 0;

        let json_content = serde_json::to_string_pretty(&save_state)?;
        std::fs::write(&pattern_file, json_content)?;
        
        info!("Current pattern saved to: {:?}", pattern_file);
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
        let current_step = state.sequencer_a_current_master_step;
        let current_bar = state.sequencer_a_current_master_bar;
        let is_running = state.is_running;
        let tick_count = state.tick_count;
        let tick_count_since_step = state.the_current_tick_count_since_step;
        let tick_count_since_start = state.the_current_tick_count_since_start;

        // Load the pattern data
        *state = loaded_state;
        
        // Restore transport state
        state.sequencer_a_current_master_step = current_step;
        state.sequencer_a_current_master_bar = current_bar;
        state.is_running = is_running;
        state.tick_count = tick_count;
        state.the_current_tick_count_since_step = tick_count_since_step;
        state.the_current_tick_count_since_start = tick_count_since_start;

        info!("Current pattern loaded from: {:?}", pattern_file);
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

    /// Advanced Sequencing Features

    /// Set global transpose
    pub fn set_global_transpose(&self, semitones: i8) {
        let mut state = self.state.lock().unwrap();
        state.global_transpose = semitones.clamp(-24, 24);
        debug!("Set global transpose: {} semitones", state.global_transpose);
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
        events: usize,
        length: usize,
        rotation: usize,
    ) {
        if row > 6 || events > length || length > 32 {
            return;
        }

        self.push_undo_snapshot(format!(
            "Euclidean rhythm R{}: {} events in {} length",
            row, events, length
        ));

        let mut state = self.state.lock().unwrap();
        let row_idx = row;

        // Update the row's Euclidean parameters
        state.sequencer_a_row_states[row_idx].sequencer_a_euclidean_events = events;
        state.sequencer_a_row_states[row_idx].sequencer_a_euclidean_length = length - 1; // Store as last valid step index (0-based)
        state.sequencer_a_row_states[row_idx].sequencer_a_euclidean_rotation = rotation;

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

        let (step, bar) = sequencer.get_position();
        assert_eq!(step, 0);
        assert_eq!(bar, 0);
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
        let (step, bar) = sequencer.get_position();
        assert_eq!(step, 0);
        assert_eq!(bar, 0);
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
            let state = sequencer.state.lock().unwrap();
            assert!(!state.is_running);
            assert_eq!(state.sequencer_a_current_master_step, 0);
            assert_eq!(state.sequencer_a_current_master_bar, 0);
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
            assert!(!state.is_running);
            assert_eq!(state.sequencer_a_current_master_step, 0);
            assert_eq!(state.sequencer_a_current_master_bar, 0);
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
                state.sequencer_a_row_states[0].sequencer_a_current_step = 5;
                state.sequencer_a_row_states[1].sequencer_a_current_step = 8;
                state.sequencer_a_row_states[2].sequencer_a_current_step = 12;
                state.sequencer_a_row_states[3].sequencer_a_current_step = 3;
                state.sequencer_a_row_states[4].sequencer_a_current_step = 15;
                state.sequencer_a_row_states[5].sequencer_a_current_step = 7;
                state.sequencer_a_row_states[6].sequencer_a_current_step = 20;
                state.sequencer_a_row_states[7].sequencer_a_current_step = 11;
            }
        }

        // Verify rows are at different positions
        {
            let state = sequencer.state.lock().unwrap();
            if state.sequencer_a_row_states.len() >= 8 {
                assert_eq!(state.sequencer_a_row_states[0].sequencer_a_current_step, 5);
                assert_eq!(state.sequencer_a_row_states[1].sequencer_a_current_step, 8);
                assert_eq!(state.sequencer_a_row_states[2].sequencer_a_current_step, 12);
                assert_eq!(state.sequencer_a_row_states[7].sequencer_a_current_step, 11);
            }
        }

        // Stop should reset ALL individual row counters to 0
        sequencer.stop();

        // Verify all row counters are reset
        {
            let state = sequencer.state.lock().unwrap();
            for (row_idx, row_state) in state.sequencer_a_row_states.iter().enumerate() {
                assert_eq!(row_state.sequencer_a_current_step, 0, 
                    "Row {} counter should be reset to 0 after stop", row_idx);
            }
        }

        // Starting again should still have all counters at 0
        sequencer.start();
        {
            let state = sequencer.state.lock().unwrap();
            for (row_idx, row_state) in state.sequencer_a_row_states.iter().enumerate() {
                assert_eq!(row_state.sequencer_a_current_step, 0, 
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
            for (row_idx, row_state) in state.sequencer_a_row_states.iter().enumerate() {
                assert_eq!(row_state.sequencer_a_current_step, 0, "Row {} should start at step 0", row_idx);
                assert_eq!(row_state.sequencer_a_first_step, 0, "Row {} should have first_step=0", row_idx);
                assert_eq!(row_state.sequencer_a_euclidean_length, 31, "Row {} should have length=31 by default", row_idx);
                assert_eq!(row_state.sequencer_a_previous_step, 31, "Row {} should have previous_step=31", row_idx);
            }
        }

        // Simulate step advancement to show the actual range
        sequencer.start();
        {
            let mut state = sequencer.state.lock().unwrap();
            let row = &mut state.sequencer_a_row_states[0];
            
            // Simulate advancing through the full range
            for expected_step in 0..=31 {
                assert_eq!(row.sequencer_a_current_step, expected_step, 
                    "Row step counter should be {} at position {}", expected_step, expected_step);
                
                // Advance step (simulating the advance_step logic)
                row.sequencer_a_previous_step = row.sequencer_a_current_step;
                row.sequencer_a_current_step += 1;
                if row.sequencer_a_current_step > row.sequencer_a_euclidean_length {
                    row.sequencer_a_current_step = row.sequencer_a_first_step;
                }
            }
            
            // After 32 steps (0-31), should wrap back to 0
            assert_eq!(row.sequencer_a_current_step, 0, "Should wrap back to 0 after step 31");
        }

        // Test with custom range
        {
            let mut state = sequencer.state.lock().unwrap();
            let row = &mut state.sequencer_a_row_states[1];
            
            // Set custom range: steps 4-15 (12 step loop starting at step 4)
            row.sequencer_a_first_step = 4;
            row.sequencer_a_euclidean_length = 15;
            row.sequencer_a_current_step = 4;
            
            // Advance through custom range
            for i in 0..20 { // Test more than one full cycle
                let expected = if i <= 11 { 4 + i } else { 4 + ((i - 12) % 12) };
                assert_eq!(row.sequencer_a_current_step, expected, 
                    "Custom range: step {} should be at position {}", i, expected);
                
                // Advance
                row.sequencer_a_previous_step = row.sequencer_a_current_step;
                row.sequencer_a_current_step += 1;
                if row.sequencer_a_current_step > row.sequencer_a_euclidean_length {
                    row.sequencer_a_current_step = row.sequencer_a_first_step;
                }
            }
        }
    }
}
