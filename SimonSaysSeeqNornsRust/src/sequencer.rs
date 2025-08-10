//! Sequencer module - Core timing and pattern logic
//!
//! Handles the main sequencing engine, timing, steps, bars, and pattern storage.

use anyhow::Result;
use crossbeam_channel::Sender;
use log::{info, debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// MIDI event for hardware output
#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub note: u8,
    pub velocity: u8,
    pub channel: u8,
    pub note_on: bool,
    pub step: usize,
    pub bar: usize,
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
pub struct MainRowStates {
    pub current_step: usize,
    pub first_step: usize,
    pub last_step: usize,
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

impl Default for MainRowStates {
    fn default() -> Self {
        Self {
            current_step: 0,
            first_step: 0,
            last_step: 15,
            midi_note: 60, // Middle C
            midi_velocity: 100,
            midi_channel: 1,
            ratchet_count: 1,
        }
    }
}

/// Pattern chain entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternChainEntry {
    pub pattern_id: usize,
    pub repeat_count: usize,
    pub transpose: i8,
    pub velocity_offset: i8,
}

/// Undo/Redo state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub grid: Vec<Vec<u8>>,
    pub mozart: Vec<Vec<u8>>,
    pub row_states: Vec<MainRowStates>,
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
    /// Grid state: grid[column][row] = value (0=off, 1=on, 2+=ratchet)
    pub grid: Vec<Vec<u8>>,
    /// Mozart state (second grid/performance controls) - MIDI note values
    pub mozart: Vec<Vec<u8>>,
    /// Mozart grid for MIDI note assignments
    pub mozart_grid: Vec<Vec<u8>>,
    /// Slide state for parameter transitions
    pub slide: SlideState,
    /// Held state for grid button combinations
    pub held: Vec<Vec<u8>>,
    /// Row states for each sequence row
    pub row_states: Vec<MainRowStates>,
    /// Pattern chains for song mode
    pub pattern_chains: Vec<PatternChainEntry>,
    pub current_chain_position: usize,
    pub chain_mode_enabled: bool,
    pub chain_repeat_current: usize,
    /// Current global position
    pub current_step: usize,
    pub current_bar: usize,
    pub current_lane: usize,
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
    pub midi_step_count: usize,
    pub midi_bar_count: usize,
    pub first_step: usize,
    pub last_step: usize,
    pub midi_first_step: usize,
    pub midi_last_step: usize,
    /// MIDI recording system - [lane][bar][step][note][on_off] = MidiNoteEvent
    pub keyboard_midi_note_events: Vec<Vec<Vec<Vec<Vec<MidiNoteEvent>>>>>,
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
}

impl Default for SequencerState {
    fn default() -> Self {
        const COLS: usize = 16;
        const ROWS: usize = 8;
        const MIN_BAR: usize = 1;
        const MAX_BAR: usize = 4;
        const MIN_LANE: usize = 1;
        const MAX_LANE: usize = 2;
        const MAX_STEP: usize = 16;
        const TOTAL_SEQUENCE_ROWS: usize = 7;

        let grid = vec![vec![0u8; ROWS]; COLS];
        let mozart = vec![vec![60u8; ROWS]; COLS]; // Initialize to middle C
        let slide = SlideState {
            slide_grid: vec![vec![0.0; ROWS]; COLS],
            scroll_offset: (0, 0),
        };
        let held = vec![vec![0u8; ROWS]; COLS];

        let mut row_states = Vec::new();
        for _ in 0..ROWS {
            row_states.push(MainRowStates::default());
        }

        // Initialize the 5D MIDI note events table: [lane][bar][step][note][on_off]
        let mut keyboard_midi_note_events = Vec::new();
        for _lane in MIN_LANE..=MAX_LANE {
            let mut bars = Vec::new();
            for _bar in MIN_BAR..=MAX_BAR {
                let mut steps = Vec::new();
                for _step in 1..=MAX_STEP {
                    let mut notes = Vec::new();
                    for _note in 0..=127 {
                        let mut on_off = Vec::new();
                        on_off.push(MidiNoteEvent::default()); // note off (0)
                        on_off.push(MidiNoteEvent::default()); // note on (1)
                        notes.push(on_off);
                    }
                    steps.push(notes);
                }
                bars.push(steps);
            }
            keyboard_midi_note_events.push(bars);
        }

        // Initialize mozart_grid with default MIDI note values (60 = middle C)
        let mozart_grid = vec![vec![60; ROWS]; COLS];

        Self {
            grid,
            mozart,
            mozart_grid,
            slide,
            held,
            row_states,
            pattern_chains: Vec::new(),
            current_chain_position: 0,
            chain_mode_enabled: false,
            chain_repeat_current: 1,
            current_step: 1,
            current_bar: 1,
            current_lane: 1,
            is_running: false,
            tempo: 30.0,
            swing_amount: 0.0,
            ticks_per_step: 12,
            first_step: 1,
            last_step: 16,
            steps_per_bar: 16,
            tick_count: 0,
            the_current_tick_count_since_step: 0,
            the_current_tick_count_since_start: 0,
            midi_step_count: 1,
            midi_bar_count: 1,
            midi_first_step: 1,
            midi_last_step: 16,
            keyboard_midi_note_events,
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

        // Initialize with a snapshot for undo/redo
        sequencer.push_undo_snapshot("Initial state".to_string());

        sequencer
    }

    /// Start the sequencer
    pub fn start(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.is_running {
            state.is_running = true;
            info!("Sequencer started");
        }
    }

    /// Stop the sequencer
    pub fn stop(&self) {
        let mut state = self.state.lock().unwrap();
        if state.is_running {
            state.is_running = false;
            // Reset to beginning
            state.current_step = 0;
            state.current_bar = 1;
            for row_state in &mut state.row_states {
                row_state.current_step = 0;
            }
            info!("Sequencer stopped and reset");
        }
    }

    /// Check if sequencer is running
    pub fn is_running(&self) -> bool {
        self.state.lock().unwrap().is_running
    }

    /// Set tempo
    pub fn set_tempo(&self, tempo: f32) {
        let mut state = self.state.lock().unwrap();
        state.tempo = tempo.clamp(20.0, 200.0);
        debug!("set_tempo says: Tempo set to: {:.1} BPM", state.tempo);
    }

    /// Get current position
    pub fn get_position(&self) -> (usize, usize) {
        let state = self.state.lock().unwrap();
        (state.current_step, state.current_bar)
    }

    /// Set grid value at position with automatic undo snapshot
    pub fn set_grid_value(&self, x: usize, y: usize, value: u8) {
        if x < 16 && y < 8 {
            self.push_undo_snapshot(format!("Set grid[{}][{}] = {} (display: step {}, row {})", x, y, value, x + 1, y + 1));
            let mut state = self.state.lock().unwrap();
            state.grid[x][y] = value;
        }
    }

    /// Get grid value at position
    pub fn get_grid_value(&self, x: usize, y: usize) -> u8 {
        let state = self.state.lock().unwrap();
        if x < 16 && y < 8 {
            state.grid[x][y]
        } else {
            0
        }
    }

    /// Set Mozart grid value (MIDI note number)
    pub fn set_mozart_value(&self, x: usize, y: usize, note: u8) {
        if x > 0 && x <= 16 && y > 0 && y <= 8 {
            self.push_undo_snapshot(format!("Set mozart[{}][{}] = {}", x, y, note));

            let mut state = self.state.lock().unwrap();
            let clamped_note = note.min(127);
            state.mozart[x - 1][y - 1] = clamped_note;
            // Clear slide for immediate response
            state.slide.slide_grid[x - 1][y - 1] = 0.0;
            debug!("Set mozart[{}][{}] = {}", x, y, clamped_note);
        }
    }

    /// Get Mozart grid value (MIDI note number)
    pub fn get_mozart_value(&self, x: usize, y: usize) -> u8 {
        let state = self.state.lock().unwrap();
        if x > 0 && x <= 16 && y > 0 && y <= 8 {
            state.mozart_grid[x - 1][y - 1]
        } else {
            60 // Default to middle C
        }
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
        (state.midi_step_count, state.midi_bar_count)
    }

    /// Randomize a specific section of the grid
    pub fn randomize_section(&self, x: usize, y: usize) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut state = self.state.lock().unwrap();

        if y <= 7 && x <= 16 {
            // Randomize based on position - left side creates denser patterns
            let density = (x as f32) / 16.0;
            let new_value = if rng.gen::<f32>() < density {
                rng.gen_range(1..=4) // Random ratchet value
            } else {
                0
            };
            state.grid[x - 1][y - 1] = new_value;
        }
    }

    /// Clear a specific section of the grid
    pub fn clear_section(&self, start_x: usize, start_y: usize, width: usize, height: usize) {
        let mut state = self.state.lock().unwrap();

        for x in start_x..=(start_x + width - 1).min(16) {
            for y in start_y..=(start_y + height - 1).min(8) {
                if x > 0 && y > 0 && x <= 16 && y <= 8 {
                    state.grid[x - 1][y - 1] = 0;
                }
            }
        }
    }

    /// Copy a section of the grid
    pub fn copy_grid_section(&self, src_x: usize, src_y: usize, dst_x: usize, dst_y: usize, width: usize, height: usize) {
        let mut state = self.state.lock().unwrap();
        let mut copy_buffer = Vec::new();

        // First copy the source section
        for y in 0..height {
            let mut row = Vec::new();
            for x in 0..width {
                let sx = src_x + x;
                let sy = src_y + y;
                if sx > 0 && sy > 0 && sx <= 16 && sy <= 8 {
                    row.push(state.grid[sx - 1][sy - 1]);
                } else {
                    row.push(0);
                }
            }
            copy_buffer.push(row);
        }

        // Then paste to destination
        for y in 0..height {
            for x in 0..width {
                let dx = dst_x + x;
                let dy = dst_y + y;
                if dx > 0 && dy > 0 && dx <= 16 && dy <= 8 {
                    state.grid[dx - 1][dy - 1] = copy_buffer[y][x];
                }
            }
        }
    }

    /// Set first step for sequencer
    pub fn set_first_step(&self, step: usize) {
        let mut state = self.state.lock().unwrap();
        state.first_step = step.clamp(0, 15);
    }

    /// Set last step for sequencer
    pub fn set_last_step(&self, step: usize) {
        let mut state = self.state.lock().unwrap();
        state.last_step = step.clamp(0, 15);
    }

    /// Get row data for display
    pub fn get_row_data(&self, row: usize) -> Vec<u8> {
        let state = self.state.lock().unwrap();
        if row > 0 && row <= 8 {
            (0..16).map(|x| state.grid[x][row - 1]).collect()
        } else {
            vec![0; 16]
        }
    }

    /// Set pattern change mode
    pub fn set_pattern_change_mode(&self, mode: bool) {
        let mut state = self.state.lock().unwrap();
        // Store pattern change mode in a custom field if needed
        // For now, just log it
        info!("Pattern change mode set to: {}", mode);
    }

    /// Set held state for button combinations
    pub fn set_held_state(&self, x: usize, y: usize, held: bool) {
        if x > 0 && x <= 16 && y > 0 && y <= 8 {
            let mut state = self.state.lock().unwrap();
            state.held[x - 1][y - 1] = if held { 1 } else { 0 };
        }
    }

    /// Check if position is held
    pub fn is_held(&self, x: usize, y: usize) -> bool {
        let state = self.state.lock().unwrap();
        if x > 0 && x <= 16 && y > 0 && y <= 8 {
            state.held[x - 1][y - 1] > 0
        } else {
            false
        }
    }

    /// Reset all sequences
    pub fn reset_all(&self) {
        let mut state = self.state.lock().unwrap();

        // Clear grid
        for col in &mut state.grid {
            for cell in col {
                *cell = 0;
            }
        }

        // Reset positions
        state.current_step = 0;
        state.current_bar = 1;
        for row_state in &mut state.row_states {
            row_state.current_step = 0;
        }

        info!("All sequences reset");
    }


    /// Get note events for a specific step
    pub fn get_step_events(&self, row: usize, _bar: usize, step: usize) -> Option<Vec<NoteEvent>> {
        let state = self.state.lock().unwrap();
        let _note_events = self.note_events.lock().unwrap();

        if row > 7 || row == 0 {
            return None; // Only sequence rows 1-7
        }

        let grid_value = state.grid[step - 1][row - 1];
        if grid_value == 0 {
            return None; // No trigger
        }

        let row_state = &state.row_states[row - 1];

        // Check if this step is within the row's range
        if step < row_state.first_step || step > row_state.last_step {
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
                note: row_state.midi_note,
                velocity: row_state.midi_velocity,
                channel: row_state.midi_channel,
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
                note: row_state.midi_note,
                velocity: 0,
                channel: row_state.midi_channel,
                note_on: false,
                tick_offset: off_tick,
            });
        }

        Some(events)
    }

    /// Main clock loop that runs in its own thread
    pub fn run_clock_loop(&mut self, sender: Sender<SequencerEvent>, running: Arc<AtomicBool>) -> Result<()> {
        info!("Starting sequencer clock loop");

        let mut last_tick = Instant::now();
        let mut tick_counter = 0u32;

        while running.load(Ordering::SeqCst) {
            let current_time = Instant::now();

            // Calculate tick interval based on tempo
            let tempo = {
                let state = self.state.lock().unwrap();
                state.tempo
            };

            // Calculate microseconds per tick
            // 60 seconds/minute * 1,000,000 microseconds/second / (tempo BPM * 4 steps/beat * 12 ticks/step)
            let micros_per_tick = (60_000_000.0 / (tempo * 4.0 * 12.0)) as u64;
            let tick_interval = Duration::from_micros(micros_per_tick);

            if current_time.duration_since(last_tick) >= tick_interval {
                last_tick = current_time;

                let is_running = {
                    let state = self.state.lock().unwrap();
                    state.is_running
                };

                if is_running {
                    self.process_tick(tick_counter, &sender)?;
                    tick_counter = (tick_counter + 1) % (16 * 12); // 16 steps per bar * 12 ticks per step
                }
            }

            // Small sleep to prevent busy waiting
            thread::sleep(Duration::from_micros(100));
        }

        info!("Sequencer clock loop terminated");
        Ok(())
    }

    /// Process a single tick - the heart of the sequencer
    fn process_tick(&mut self, tick_counter: u32, sender: &Sender<SequencerEvent>) -> Result<()> {
        // First phase: update tick counters and analyze tempo
        {
            let mut state = self.state.lock().unwrap();
            state.tick_count += 1;
            state.the_current_tick_count_since_start += 1;
            state.the_current_tick_count_since_step = tick_counter % state.ticks_per_step;

            // Analyze tempo stability
            self.analyze_tempo_stability(&mut state);
        }

        // Second phase: process MIDI playback (needs immutable access)
        {
            let state = self.state.lock().unwrap();
            self.play_midi(&state, sender)?;
        }

        // Third phase: advance step if needed (needs mutable access)
        if tick_counter % self.state.lock().unwrap().ticks_per_step == 0 {
            let mut state = self.state.lock().unwrap();
            self.advance_step(&mut state, sender)?;
        }

        Ok(())
    }

    /// Play MIDI events for current tick
    fn play_midi(&self, state: &SequencerState, sender: &Sender<SequencerEvent>) -> Result<()> {
        let current_lane = state.current_lane;
        let midi_bar_count = state.midi_bar_count;
        let midi_step_count = state.midi_step_count;
        let tick_since_step = state.the_current_tick_count_since_step;

        // Check keyboard MIDI note events for this tick
        if current_lane <= state.max_lane && midi_bar_count <= state.max_bar {
            for note in 0..=127 {
                let lane_idx = current_lane - 1;
                let bar_idx = midi_bar_count - 1;
                let step_idx = midi_step_count - 1;

                if lane_idx < state.keyboard_midi_note_events.len()
                    && bar_idx < state.keyboard_midi_note_events[lane_idx].len()
                    && step_idx < state.keyboard_midi_note_events[lane_idx][bar_idx].len() {

                    // Check note ON events
                    let note_on_event = &state.keyboard_midi_note_events[lane_idx][bar_idx][step_idx][note][1];
                    if note_on_event.is_active && note_on_event.tick_count_since_step == tick_since_step {
                        debug!("MIDI Note ON: note={}, velocity={}, step={}", note, note_on_event.velocity, midi_step_count);

                        // Create and send MIDI event
                        let midi_event = MidiEvent {
                            note: note as u8,
                            velocity: note_on_event.velocity,
                            channel: (lane_idx + 1) as u8,
                            note_on: true,
                            step: midi_step_count,
                            bar: midi_bar_count,
                        };

                        if let Err(e) = sender.try_send(SequencerEvent::MidiEvent(midi_event)) {
                            warn!("Failed to send MIDI note ON event: {}", e);
                        }
                    }

                    // Check note OFF events
                    let note_off_event = &state.keyboard_midi_note_events[lane_idx][bar_idx][step_idx][note][0];
                    if note_off_event.is_active && note_off_event.tick_count_since_step == tick_since_step {
                        debug!("MIDI Note OFF: note={}, step={}", note, midi_step_count);

                        // Create and send MIDI event
                        let midi_event = MidiEvent {
                            note: note as u8,
                            velocity: 0,
                            channel: (lane_idx + 1) as u8,
                            note_on: false,
                            step: midi_step_count,
                            bar: midi_bar_count,
                        };

                        if let Err(e) = sender.try_send(SequencerEvent::MidiEvent(midi_event)) {
                            warn!("Failed to send MIDI note OFF event: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Advance to next step and process triggers
    fn advance_step(&self, state: &mut SequencerState, sender: &Sender<SequencerEvent>) -> Result<()> {
        // Reset tick count since step
        state.the_current_tick_count_since_step = 0;

        // Process current step for all sequence rows
        self.process_step(&*state, sender)?;

        // Advance MIDI step
        state.midi_step_count += 1;
        if state.midi_step_count > state.midi_last_step {
            state.midi_step_count = state.midi_first_step;

            // Advance bar
            state.midi_bar_count += 1;
            if state.midi_bar_count > state.max_bar {
                state.midi_bar_count = state.min_bar;
            }
        }

        // Update global position counters to match MIDI counters
        state.current_step = state.midi_step_count;
        state.current_bar = state.midi_bar_count;

        // Advance each row's current step based on its individual settings
        for (row_idx, row_state) in state.row_states.iter_mut().enumerate() {
            let old_step = row_state.current_step;
            row_state.current_step += 1;
            if row_state.current_step > row_state.last_step {
                row_state.current_step = row_state.first_step;
            }
            if row_idx < 3 { // Debug first 3 rows
                //info!("🎯 Row {} step advancement: {} -> {} (range: {}-{})",
                //      row_idx + 1, old_step, row_state.current_step,
                //      row_state.first_step, row_state.last_step);
            }

        }

        // Update CO2 counters if we have data
        if state.total_step_co2_count > 0 {
            // This would advance CO2 counters if CO2 data is available
            // Implementation would depend on CO2 data structure
        }

        // Send step event
        let _ = sender.send(SequencerEvent::Step {
            step: state.midi_step_count,
            bar: state.midi_bar_count
        });

        Ok(())
    }

    /// Process triggers for the current step
    fn process_step(&self, state: &SequencerState, _sender: &Sender<SequencerEvent>) -> Result<()> {
        // Process each sequence row
        for sequence_row in 1..=state.total_sequence_rows {
            if let Some(row_state) = state.row_states.get(sequence_row - 1) {
                let current_step = row_state.current_step;

                // Get grid value for this row at current step
                if current_step < state.cols {
                    let grid_value = state.grid[current_step][sequence_row - 1];

                    if grid_value > 0 {
                        // This step is active - trigger would happen here
                        debug!("Trigger: row={}, step={}, value={}", sequence_row, current_step, grid_value);

                        // Here we would:
                        // 1. Send MIDI note based on row_settings.midi_note
                        // 2. Handle ratcheting if grid_value > 1
                        // 3. Send CV/Gate outputs via Crow
                        // 4. Update grid LEDs
                    }
                }
            }
        }

        Ok(())
    }

    /// Analyze tempo stability (wow and flutter detection)
    fn analyze_tempo_stability(&self, state: &mut SequencerState) {
        let current_tempo = state.tempo;
        let analysis = &mut state.tempo_analysis;

        // Update tempo sum for averaging windows
        analysis.wow_tempo_sum += current_tempo;
        analysis.flutter_tempo_sum += current_tempo;

        // Advance window positions
        analysis.wow_window_tick_position += 1;
        analysis.flutter_window_tick_position += 1;

        // Calculate averages when windows are full
        if analysis.wow_window_tick_position >= analysis.wow_window_size {
            analysis.wow_average_tempo = analysis.wow_tempo_sum / analysis.wow_window_size as f32;
            analysis.wow_window_tick_position = 0;
            analysis.wow_tempo_sum = 0.0;
        }

        if analysis.flutter_window_tick_position >= analysis.flutter_window_size {
            analysis.flutter_average_tempo = analysis.flutter_tempo_sum / analysis.flutter_window_size as f32;
            analysis.flutter_window_tick_position = 0;
            analysis.flutter_tempo_sum = 0.0;
        }

        // Check for wow (large tempo instability)
        let wow_diff = (analysis.wow_average_tempo - current_tempo).abs();
        if wow_diff > analysis.wow_threshold {
            analysis.total_wow_tempo_ticks += 1;
            if analysis.tempo_wow_is_good {
                analysis.wow_tempo_episodes += 1;
                analysis.tempo_wow_is_good = false;
            }
        } else {
            analysis.tempo_wow_is_good = true;
        }

        // Check for flutter (small tempo instability)
        let flutter_diff = (analysis.flutter_average_tempo - current_tempo).abs();
        if flutter_diff > analysis.flutter_threshold {
            analysis.total_flutter_tempo_ticks += 1;
            if analysis.tempo_flutter_is_good {
                analysis.flutter_tempo_episodes += 1;
                analysis.tempo_flutter_is_good = false;
            }
        } else {
            analysis.tempo_flutter_is_good = true;
        }
    }

    /// Initialize sequencer state tables
    pub fn init_state_tables(&self) {
        let mut state = self.state.lock().unwrap();

        // Reset all counters
        state.tick_count = 0;
        state.the_current_tick_count_since_start = 0;
        state.the_current_tick_count_since_step = 0;
        state.midi_step_count = state.first_step;
        state.midi_bar_count = 1;

        // Reset tempo analysis
        state.tempo_analysis = TempoAnalysis::default();

        // Initialize row settings
        for (i, row_state) in state.row_states.iter_mut().enumerate() {
            row_state.current_step = row_state.first_step;
            row_state.midi_note = 60 + i as u8; // Start from middle C
            row_state.midi_velocity = 100;
            row_state.midi_channel = (i + 1) as u8;
        }

        info!("Sequencer state tables initialized");
    }

    /// Toggle grid position (used for pattern editing)
    pub fn toggle_grid_position(&self, x: usize, y: usize) {
        if x > 0 && x <= 16 && y > 0 && y <= 8 {
            let mut state = self.state.lock().unwrap();
            let current_value = state.grid[x - 1][y - 1];
            let new_value = if current_value == 0 { 1 } else { 0 };
            state.grid[x - 1][y - 1] = new_value;
            debug!("toggle_grid_position says: Toggled grid[{}][{}]: {} -> {}", x, y, current_value, new_value);
        }
    }

    /// Set MIDI note event for keyboard recording
    pub fn set_midi_note_event(&self, lane: usize, bar: usize, step: usize, note: u8, is_on: bool, velocity: u8, tick_offset: u32) {
        let mut state = self.state.lock().unwrap();

        if lane >= state.min_lane && lane <= state.max_lane
            && bar >= state.min_bar && bar <= state.max_bar
            && step >= 1 && step <= state.max_step {

            let lane_idx = lane - 1;
            let bar_idx = bar - 1;
            let step_idx = step - 1;
            let event_idx = if is_on { 1 } else { 0 };

            if lane_idx < state.keyboard_midi_note_events.len()
                && bar_idx < state.keyboard_midi_note_events[lane_idx].len()
                && step_idx < state.keyboard_midi_note_events[lane_idx][bar_idx].len()
                && (note as usize) < state.keyboard_midi_note_events[lane_idx][bar_idx][step_idx].len() {

                let current_tick_count = state.the_current_tick_count_since_start;
                let event = &mut state.keyboard_midi_note_events[lane_idx][bar_idx][step_idx][note as usize][event_idx];
                event.velocity = velocity;
                event.tick_count_since_step = tick_offset;
                event.is_active = velocity > 0;
                event.tick_count_since_start = current_tick_count;

                debug!("Set MIDI event: lane={}, bar={}, step={}, note={}, on={}, vel={}, tick={}",
                       lane, bar, step, note, is_on, velocity, tick_offset);
            }
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
        state.grid.clone()
    }

    /// Get a copy of the current Mozart state for display
    pub fn get_mozart_state(&self) -> Vec<Vec<u8>> {
        let state = self.state.lock().unwrap();
        state.mozart.clone()
    }

    /// Undo/Redo System Implementation

    /// Push current state to undo stack
    fn push_undo_snapshot(&self, description: String) {
        let state = self.state.lock().unwrap();
        let snapshot = StateSnapshot {
            grid: state.grid.clone(),
            mozart: state.mozart.clone(),
            row_states: state.row_states.clone(),
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

        // Push current state to redo stack
        let current_state = self.state.lock().unwrap();
        let current_snapshot = StateSnapshot {
            grid: current_state.grid.clone(),
            mozart: current_state.mozart.clone(),
            row_states: current_state.row_states.clone(),
            timestamp: std::time::SystemTime::now(),
            description: "Current state before undo".to_string(),
        };
        drop(current_state);

        redo_stack.push_back(current_snapshot);

        // Pop from undo stack and apply
        if let Some(snapshot) = undo_stack.pop_back() {
            let mut state = self.state.lock().unwrap();
            state.grid = snapshot.grid;
            state.mozart = snapshot.mozart;
            state.row_states = snapshot.row_states;

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

        // Push current state to undo stack
        let current_state = self.state.lock().unwrap();
        let current_snapshot = StateSnapshot {
            grid: current_state.grid.clone(),
            mozart: current_state.mozart.clone(),
            row_states: current_state.row_states.clone(),
            timestamp: std::time::SystemTime::now(),
            description: "State before redo".to_string(),
        };
        drop(current_state);

        undo_stack.push_back(current_snapshot);

        // Pop from redo stack and apply
        if let Some(snapshot) = redo_stack.pop_back() {
            let mut state = self.state.lock().unwrap();
            state.grid = snapshot.grid;
            state.mozart = snapshot.mozart;
            state.row_states = snapshot.row_states;

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
        pattern_state.current_step = 0;
        pattern_state.current_bar = 1;

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
            state.grid = pattern_state.grid.clone();
            state.mozart = pattern_state.mozart.clone();
            state.slide = pattern_state.slide.clone();
            state.row_states = pattern_state.row_states.clone();

            // Keep current transport state
            let current_step = state.current_step;
            let current_bar = state.current_bar;
            let is_running = state.is_running;

            *state = pattern_state.clone();
            state.current_step = current_step;
            state.current_bar = current_bar;
            state.is_running = is_running;

            // Note: current_pattern tracking would need to be moved to state if needed
            info!("Loaded pattern {}", pattern_id);

            Ok(())
        } else {
            Err(anyhow::anyhow!("Pattern {} not found", pattern_id))
        }
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
        debug!("Set global velocity scale: {:.2}", state.global_velocity_scale);
    }

    /// Scroll pattern by offset
    pub fn scroll_pattern(&self, x_offset: i32, y_offset: i32) {
        self.push_undo_snapshot(format!("Scroll pattern ({}, {})", x_offset, y_offset));

        let mut state = self.state.lock().unwrap();

        // Create new grids with scrolled content
        let mut new_grid = vec![vec![0u8; 8]; 16];
        let mut new_mozart = vec![vec![60u8; 8]; 16];

        for x in 0..16 {
            for y in 0..8 {
                let src_x = ((x as i32 - x_offset) + 16) % 16;
                let src_y = ((y as i32 - y_offset) + 8) % 8;

                new_grid[x][y] = state.grid[src_x as usize][src_y as usize];
                new_mozart[x][y] = state.mozart[src_x as usize][src_y as usize];
            }
        }

        state.grid = new_grid;
        state.mozart = new_mozart;
        state.slide.scroll_offset.0 += x_offset;
        state.slide.scroll_offset.1 += y_offset;

        info!("scroll_pattern says: Scrolled pattern by ({}, {})", x_offset, y_offset);
    }

    /// Randomize specific rows with constraints


    /// Pattern Chain Management

    /// Add pattern to chain
    pub fn add_to_chain(&self, pattern_id: usize, repeat_count: usize, transpose: i8, velocity_offset: i8) {
        let mut state = self.state.lock().unwrap();
        let entry = PatternChainEntry {
            pattern_id,
            repeat_count,
            transpose,
            velocity_offset,
        };
        state.pattern_chains.push(entry);
        info!("Added pattern {} to chain (repeat: {}, transpose: {}, vel_offset: {})",
              pattern_id, repeat_count, transpose, velocity_offset);
    }

    /// Clear pattern chain
    pub fn clear_chain(&self) {
        let mut state = self.state.lock().unwrap();
        state.pattern_chains.clear();
        state.current_chain_position = 0;
        state.chain_repeat_current = 0;
        state.chain_mode_enabled = false;
        info!("Cleared pattern chain");
    }

    /// Enable/disable chain mode
    pub fn set_chain_mode(&self, enabled: bool) {
        let mut state = self.state.lock().unwrap();
        state.chain_mode_enabled = enabled;
        if enabled && !state.pattern_chains.is_empty() {
            // Load first pattern in chain
            state.current_chain_position = 0;
            state.chain_repeat_current = 0;
            info!("Chain mode enabled, starting with pattern {}",
                  state.pattern_chains[0].pattern_id);
        } else {
            info!("Chain mode disabled");
        }
    }

    /// Advance chain to next pattern (called at end of bar)
    pub fn advance_chain(&self) -> Result<()> {
        let (should_advance, next_pattern_id) = {
            let mut state = self.state.lock().unwrap();

            if !state.chain_mode_enabled || state.pattern_chains.is_empty() {
                return Ok(());
            }

            let current_entry_repeat_count = state.pattern_chains[state.current_chain_position].repeat_count;
            state.chain_repeat_current += 1;

            if state.chain_repeat_current >= current_entry_repeat_count {
                // Move to next pattern in chain
                state.chain_repeat_current = 0;
                state.current_chain_position = (state.current_chain_position + 1) % state.pattern_chains.len();

                let next_pattern_id = state.pattern_chains[state.current_chain_position].pattern_id;
                info!("Chain advanced to pattern {} (repeat {}/{})",
                      next_pattern_id,
                      state.chain_repeat_current + 1,
                      state.pattern_chains[state.current_chain_position].repeat_count);

                (true, next_pattern_id)
            } else {
                (false, 0)
            }
        };

        if should_advance {
            // Load the pattern by copying its state
            let patterns = self.patterns.lock().unwrap();
            if let Some(pattern_state) = patterns.get(&next_pattern_id) {
                let mut state = self.state.lock().unwrap();
                state.grid = pattern_state.grid.clone();
                state.mozart = pattern_state.mozart.clone();
                state.slide = pattern_state.slide.clone();
                state.row_states = pattern_state.row_states.clone();
                info!("Loaded pattern {} in chain", next_pattern_id);
            }
        }

        Ok(())
    }

    /// Slide and Interpolation Features

    /// Set slide amount for smooth parameter transitions
    pub fn set_slide(&self, x: usize, y: usize, amount: f32) {
        if x > 0 && x <= 16 && y > 0 && y <= 8 {
            let mut state = self.state.lock().unwrap();
            state.slide.slide_grid[x - 1][y - 1] = amount.clamp(0.0, 1.0);
            debug!("Set slide[{}][{}] = {:.2}", x, y, amount);
        }
    }

    /// Get interpolated value considering slide
    pub fn get_interpolated_note(&self, x: usize, y: usize, progress: f32) -> f32 {
        let state = self.state.lock().unwrap();
        if x > 0 && x <= 16 && y > 0 && y <= 8 {
            let base_note = state.mozart[x - 1][y - 1] as f32;
            let slide_amount = state.slide.slide_grid[x - 1][y - 1];

            if slide_amount > 0.0 {
                // Find next non-zero note for sliding target
                let mut target_note = base_note;
                for offset in 1..=8 {
                    let target_x = (x + offset) % 16;
                    if self.get_grid_value(target_x, y) > 0 {
                        target_note = state.mozart[target_x][y] as f32;
                        break;
                    }
                }

                // Interpolate between base and target
                base_note + (target_note - base_note) * slide_amount * progress
            } else {
                base_note
            }
        } else {
            60.0 // Default to middle C
        }
    }

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

        // Apply pattern chain transpose if in chain mode
        if state.chain_mode_enabled && !state.pattern_chains.is_empty() {
            let current_entry = &state.pattern_chains[state.current_chain_position];
            note += current_entry.transpose as i16;
        }

        // Clamp to valid MIDI range
        note.clamp(0, 127) as u8
    }

    /// Get effective velocity with global modifications
    pub fn get_effective_velocity(&self, base_velocity: u8, _row: usize) -> u8 {
        let state = self.state.lock().unwrap();
        let mut velocity = base_velocity as f32;

        // Apply global velocity scale
        velocity *= state.global_velocity_scale;

        // Apply pattern chain velocity offset if in chain mode
        if state.chain_mode_enabled && !state.pattern_chains.is_empty() {
            let current_entry = &state.pattern_chains[state.current_chain_position];
            velocity += current_entry.velocity_offset as f32;
        }

        // Clamp to valid MIDI range
        velocity.clamp(1.0, 127.0) as u8
    }

    /// Euclidean rhythm generation
    pub fn generate_euclidean_rhythm(&self, row: usize, pulses: usize, steps: usize, rotation: usize) {
        if row == 0 || row > 8 || pulses > steps || steps > 16 {
            return;
        }

        self.push_undo_snapshot(format!("Euclidean rhythm R{}: {} pulses in {} steps", row, pulses, steps));

        let mut state = self.state.lock().unwrap();
        let row_idx = row - 1;

        // Clear the row first
        for col in 0..16 {
            state.grid[col][row_idx] = 0;
        }

        // Generate euclidean rhythm using Bresenham's algorithm
        let mut bucket = 0;
        for i in 0..steps {
            bucket += pulses;
            if bucket >= steps {
                bucket -= steps;
                let pos = (i + rotation) % steps;
                if pos < 16 {
                    state.grid[pos][row_idx] = 1;
                }
            }
        }

        info!("Generated euclidean rhythm for row {}: {} pulses in {} steps, rotation {}",
              row, pulses, steps, rotation);
    }

    /// Copy pattern section
    pub fn copy_section(&self, src_x: usize, src_y: usize, width: usize, height: usize,
                       dest_x: usize, dest_y: usize) {
        self.push_undo_snapshot(format!("Copy section {}x{} from ({},{}) to ({},{})",
                                       width, height, src_x, src_y, dest_x, dest_y));

        let mut state = self.state.lock().unwrap();

        for x in 0..width {
            for y in 0..height {
                let src_col = src_x + x;
                let src_row = src_y + y;
                let dest_col = dest_x + x;
                let dest_row = dest_y + y;

                if src_col < 16 && src_row < 8 && dest_col < 16 && dest_row < 8 {
                    state.grid[dest_col][dest_row] = state.grid[src_col][src_row];
                    state.mozart[dest_col][dest_row] = state.mozart[src_col][src_row];
                    state.slide.slide_grid[dest_col][dest_row] = state.slide.slide_grid[src_col][src_row];
                }
            }
        }

        info!("Copied {}x{} section from ({},{}) to ({},{})", width, height, src_x, src_y, dest_x, dest_y);
    }

    /// Set row states
    pub fn set_row_states(&self, row: usize, states: MainRowStates) {
        if row > 0 && row <= 7 {
            let mut state = self.state.lock().unwrap();
            state.row_states[row - 1] = states;
            debug!("set_row_states says: Updated states for row {}", row);
        }
    }

    /// Get row states
    pub fn get_row_states(&self, row: usize) -> Option<MainRowStates> {
        if row > 0 && row <= 7 {
            let state = self.state.lock().unwrap();
            Some(state.row_states[row - 1].clone())
        } else {
            None
        }
    }

    /// Get current global transpose
    pub fn get_global_transpose(&self) -> i8 {
        let state = self.state.lock().unwrap();
        state.global_transpose
    }

    /// Check if chain mode is enabled
    pub fn is_chain_mode_enabled(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.chain_mode_enabled
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
        assert_eq!(step, 1);
        assert_eq!(bar, 1);
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
        sequencer.set_grid_value(16, 8, 1); // Should be ignored
        assert_eq!(sequencer.get_grid_value(16, 8), 0);
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
        assert_eq!(step, 1);
        assert_eq!(bar, 1);
    }

    #[test]
    fn test_tempo_control() {
        let sequencer = Sequencer::new();

        sequencer.set_tempo(140.0);
        let state = sequencer.state.lock().unwrap();
        assert_eq!(state.tempo, 140.0);

        // Test clamping
        drop(state);
        sequencer.set_tempo(300.0);
        let state = sequencer.state.lock().unwrap();
        assert_eq!(state.tempo, 200.0); // Should be clamped to max
    }

    #[test]
    fn test_step_events() {
        let sequencer = Sequencer::new();

        // Set a trigger on step 0, row 0
        sequencer.set_grid_value(0, 0, 1);

        // Should get note events for this step
        let events = sequencer.get_step_events(0, 1, 1);
        assert!(events.is_some());

        let events = events.unwrap();
        assert!(!events.is_empty());

        // Should have note on and note off events
        let note_on_events: Vec<_> = events.iter().filter(|e| e.note_on).collect();
        let note_off_events: Vec<_> = events.iter().filter(|e| !e.note_on).collect();

        assert!(!note_on_events.is_empty());
        assert!(!note_off_events.is_empty());
    }
}
