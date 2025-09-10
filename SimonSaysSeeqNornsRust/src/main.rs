//! SimonSaysSeeq Pure Rust Implementation for Norns
//!
//! A high-performance sequencer application that runs directly on Norns hardware
//! without requiring the Norns Lua environment.

use anyhow::Result;
use log::{info, warn, debug, error};
use anyhow::anyhow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use chrono;

mod hardware;
mod sequencer;
mod midi;
mod midi_scanner;

mod screen;
mod config;
mod co2;

// LED brightness constants
const LED_OFF: u8 = 0;      // Empty step, no playhead (LED off)
const LED_DIM: u8 = 6;      // Empty step, playhead present (position only)
const LED_DIM_PLUS: u8 = 8; // Empty step, playhead present (enhanced visibility)
const LED_BRIGHT: u8 = 10;  // Pattern exists, no playhead (pattern only)
const LED_MAX: u8 = 14;     // Pattern exists, playhead present (pattern + position)
const LED_TURBO: u8 = 15;   // Maximum brightness (ARM buttons, flashing, etc.)

/// ARM actions that can be triggered from row 7 (control row) of the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArmAction {
    Undo,              // Column 0
    Redo,              // Column 1
    EuclidianEvents,   // Column 4
    EuclidianLength,   // Column 5
    EuclidianRotation, // Column 6
    Ratchet,           // Column 7
    SetSeqALength,     // Column 8
    PresetGrid,        // Column 10

}

impl ArmAction {
    /// Convert grid column to ARM action
    fn from_column(column: usize) -> Option<Self> {
        match column {
            0 => Some(ArmAction::Undo),
            1 => Some(ArmAction::Redo),
            4 => Some(ArmAction::EuclidianEvents),
            5 => Some(ArmAction::EuclidianLength),
            6 => Some(ArmAction::EuclidianRotation),
            7 => Some(ArmAction::Ratchet),
            8 => Some(ArmAction::SetSeqALength),
            10 => Some(ArmAction::PresetGrid),

            _ => None,
        }
    }

    /// Convert ARM action to grid column
    fn to_column(&self) -> usize {
        match self {
            ArmAction::Undo => 0,
            ArmAction::Redo => 1,
            ArmAction::EuclidianEvents => 4,
            ArmAction::EuclidianLength => 5,
            ArmAction::EuclidianRotation => 6,
            ArmAction::Ratchet => 7,
            ArmAction::SetSeqALength => 8,
            ArmAction::PresetGrid => 10,

        }
    }
}

use crossbeam_channel::Receiver;

use hardware::{NornsHardware, HardwareEvent};
use sequencer::{Sequencer, SequencerEvent};
#[cfg(feature = "midi")]
use midi::MidiManager;
use simon_says_seeq_rust::grid_osc::GridManager;
use screen::ScreenManager;
use config::Config;
use co2::Co2Manager;

/// Main application state
pub struct SimonSaysSeeq {
    hardware: NornsHardware,
    sequencer: Sequencer,
    #[cfg(feature = "midi")]
    midi: MidiManager,
    grid: GridManager,
    screen: ScreenManager,
    co2: Co2Manager,
    config: Config,
    running: Arc<AtomicBool>,
    tempo: f32,
    // Active ARM action for row 7 (control row) - only one can be active at a time
    active_arm_action: Option<ArmAction>,
    // Beat LED flashing state for external MIDI clock
    beat_led_flash_until: Option<Instant>,
    // Phase correction state for MIDI clock sync
    last_midi_clock_count: u32,
    last_sync_check: Instant,
    current_drift_ticks: i32,
    // Tempo stability tracking for phase correction
    tempo_stable_since: Option<Instant>,
    last_stable_tempo: Option<f32>,
    // Snap to whole number tempo setting (default ON)
    snap_to_whole_tempo: bool,
    // GRID_TWO button state tracking for MIDI detection
    grid_two_button_0_pressed: bool,
    grid_two_button_1_pressed: bool,
}

impl SimonSaysSeeq {
    pub fn new() -> Result<Self> {
        let config = Config::load_or_default()?;
        let initial_tempo = config.sequencer.default_tempo;

        Ok(Self {
            hardware: NornsHardware::new()?,
            sequencer: Sequencer::new(),
            #[cfg(feature = "midi")]
            midi: MidiManager::new(&config.midi)?,
            grid: {
                let grid_manager = GridManager::new()?;
                // HARD REQUIREMENT: Verify exactly 2 real grids are connected
                grid_manager.verify_two_grids_requirement()
                    .map_err(|e| {
                        error!("STARTUP FAILURE: {}", e);
                        error!("SimonSaysSeeq requires exactly TWO REAL grids (GRID_ONE and GRID_TWO)");
                        error!("Application cannot start without meeting this requirement.");
                        e
                    })?;
                info!("✅ HARD REQUIREMENT MET: Two real grids verified at startup");
                grid_manager
            },
            screen: ScreenManager::new()?,
            co2: Co2Manager::new(config.co2.clone())?,
            config,
            running: Arc::new(AtomicBool::new(false)),
            tempo: initial_tempo,
            active_arm_action: None, // No ARM action initially active
            beat_led_flash_until: None,
            last_midi_clock_count: 0,
            last_sync_check: Instant::now(),
            current_drift_ticks: 0,
            tempo_stable_since: None,
            last_stable_tempo: None,
            snap_to_whole_tempo: true, // Default ON
            grid_two_button_0_pressed: false,
            grid_two_button_1_pressed: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // info!("run says: Starting SimonSaysSeeq Rust application");

        // Verify and display grid assignment
        let connected_grids = self.grid.get_connected_grids();
        if connected_grids.len() != 2 {
            error!("RUNTIME FAILURE: Expected exactly 2 grids, found {}", connected_grids.len());
            return Err(anyhow!("HARD REQUIREMENT VIOLATION: Two real grids required"));
        }
        
        let (grid_one, grid_two) = self.grid.get_grid_ids_ordered()?;
        info!("🎛️  GRID ASSIGNMENT:");
        info!("   GRID_ONE: {}", grid_one);
        info!("   GRID_TWO: {}", grid_two);
        info!("✅ Two real grids ready for operation");

        // Show control instructions
        // info!("run says: Controls:");
        // info!("run says:   Ctrl+C: Stop application");

        self.running.store(true, Ordering::SeqCst);

        // Create communication channels
        let (hw_tx, hw_rx) = crossbeam_channel::unbounded::<HardwareEvent>();
        let (seq_tx, seq_rx) = crossbeam_channel::unbounded::<SequencerEvent>();

        // Start hardware input thread
        let running_hw = self.running.clone();
        let mut hardware = self.hardware.clone();
        let hw_thread = thread::spawn(move || {
            hardware.run_input_loop(hw_tx, running_hw)
        });

        // Start sequencer thread
        let running_seq = self.running.clone();
        let mut sequencer = self.sequencer.clone();
        let seq_thread = thread::spawn(move || {
            sequencer.run_clock_loop(seq_tx, running_seq)
        });

        // Auto-start the sequencer for desktop testing (no hardware required)
        // info!("Auto-starting sequencer - tempo: {:.1} BPM", self.tempo);
        self.sequencer.set_tempo(self.tempo);
        self.sequencer.start();

        // Show grid connection status
        // let connected_grids = self.grid.get_connected_grids();
        // info!("run says: Connected grids: {:?}", connected_grids);
        // for grid_id in &connected_grids {
        //     if let Some((cols, rows)) = self.grid.get_dimensions(grid_id) {
        //         info!("run says:   Grid {}: {}x{}", grid_id, cols, rows);
        //     }
        // }

        // TEMPORARILY DISABLED FOR DEBUGGING LED DROPPING ISSUE
        // Flash all connected grids for visual feedback
        // if let Err(e) = self.grid.flash_all_grids() {
        //     // warn!("Failed to flash grids on sequencer start: {}", e);
        // }

        // Initialize main grid with some default pattern for testing
        // Set test patterns for all rows to verify display pipeline (0-indexed)
        for row in 0..=6 {
            self.sequencer.set_grid_value(0, row, 1);   // Step 0 (display: step 1)
            self.sequencer.set_grid_value(4, row, 2);   // Step 4 (display: step 5)
            self.sequencer.set_grid_value(8, row, 1);   // Step 8 (display: step 9)
            self.sequencer.set_grid_value(12, row, 2);  // Step 12 (display: step 13)
        }

        // Update main grid initially
        self.update_grid_display()?;

        // Main event loop
        self.main_loop(hw_rx, seq_rx)?;

        // Cleanup with timeout
        self.running.store(false, Ordering::SeqCst);

        // Give threads a chance to exit gracefully
        // info!("run says: Shutting down threads...");

        // Try to join with timeout
        let hw_result = std::thread::spawn(move || hw_thread.join()).join();
        let seq_result = std::thread::spawn(move || seq_thread.join()).join();

        // If threads don't exit cleanly within reasonable time, force exit
        thread::sleep(Duration::from_millis(500));

        if hw_result.is_err() || seq_result.is_err() {
            // warn!("run says: Threads did not exit cleanly, forcing shutdown");
            std::process::exit(0);
        }

        // info!("run says: SimonSaysSeeq shut down successfully");
        Ok(())
    }

    fn main_loop(&mut self, hw_rx: Receiver<HardwareEvent>, seq_rx: Receiver<SequencerEvent>) -> Result<()> {
        let mut last_screen_update = Instant::now();
        let screen_update_interval = Duration::from_millis(33); // ~30 FPS

        loop {
            // Handle hardware events (non-blocking)
            while let Ok(event) = hw_rx.try_recv() {
                if let Err(e) = self.handle_hardware_event(event) {
                    // error!("Error handling hardware event: {}", e);
                }
            }

            // Poll grid for button events
            let poll_start = Instant::now();
                match self.grid.read_button_events() {
                    Ok(grid_events) => {
                        // Process events from both grids - each should only report its own presses
                        let connected_grids = self.grid.get_connected_grids();
                        
                        for grid_event in grid_events {
                            // Check if this is from one of our expected grids
                            let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
                            let is_valid_grid = Some(&grid_event.grid_id) == grid_one.as_ref() || 
                                              Some(&grid_event.grid_id) == grid_two.as_ref();
                            
                            if is_valid_grid {
                                info!("GRID DEBUG: Processing event from {} at ({},{}) pressed={}", 
                                      grid_event.grid_id, grid_event.x, grid_event.y, grid_event.pressed);
                                
                                let hardware_event = HardwareEvent::GridPress {
                                    grid_id: grid_event.grid_id,
                                    x: grid_event.x,
                                    y: grid_event.y,
                                    pressed: grid_event.pressed,
                                };
                                if let Err(e) = self.handle_hardware_event(hardware_event) {
                                    // error!("Error handling grid event: {}", e);
                                }
                            } else {
                                info!("GRID DEBUG: Ignoring event from unknown grid: {}", grid_event.grid_id);
                            }
                        }
                    }
                    Err(e) => {
                        // Don't spam errors for no events
                        if !e.to_string().contains("No events available") && !e.to_string().contains("would block") {
                            debug!("Grid polling error: {}", e);
                        }
                    }
                }

            // Handle sequencer events (non-blocking)
            while let Ok(event) = seq_rx.try_recv() {
                if let Err(e) = self.handle_sequencer_event(event) {
                    // error!("Error handling sequencer event: {}", e);
                }
            }

            // Handle MIDI input events (non-blocking)
            #[cfg(feature = "midi")]
            {
                let midi_events = self.midi.get_input_events();
                for event in midi_events {
                    if let Err(e) = self.handle_midi_input_event(event) {
                        // error!("Error handling MIDI input event: {}", e);
                    }
                }
                

            }

            // Update screen at regular intervals
            if last_screen_update.elapsed() >= screen_update_interval {
                self.update_screen()?;
                last_screen_update = Instant::now();
            }

            // Check for shutdown
            if !self.running.load(Ordering::SeqCst) {
                // info!("main_loop says: Main loop detected shutdown signal, breaking");
                break;
            }

            // Faster polling for better button responsiveness
            thread::sleep(Duration::from_micros(100));
        }

        Ok(())
    }

    fn handle_hardware_event(&mut self, event: HardwareEvent) -> Result<()> {
        match event {
            //TODO map these to buttons instead of encoders if we neeed them.
            HardwareEvent::EncoderTurn { encoder, delta } => {
                match encoder {
                    1 => {
                        // Left encoder controls swing
                        let current_swing = self.sequencer.get_swing_amount();
                        let new_swing = (current_swing + delta as f32 * 0.01).clamp(0.0, 0.5);
                        self.sequencer.set_swing(new_swing);
                        // info!("Swing changed to: {:.2}", new_swing);
                    }
                    2 => {
                        // Middle encoder controls global transpose
                        let current_transpose = self.sequencer.get_global_transpose();
                        let new_transpose = (current_transpose as i32 + delta).clamp(-24, 24);
                        self.sequencer.set_global_transpose(new_transpose as i8);
                        // info!("Global transpose changed to: {} semitones", new_transpose);
                    }
                    3 => {
                        // Right encoder controls tempo
                        self.tempo = (self.tempo + delta as f32).clamp(20.0, 300.0);
                        self.sequencer.set_tempo(self.tempo);
                        // info!("Tempo changed to: {:.1} BPM", self.tempo);
                    }
                    _ => {}
                }
            }

            HardwareEvent::KeyPress { key, pressed } => {
                if pressed {
                    match key {
                        1 => {
                            // Key 1 - Undo/Redo (long press for redo)
                            match self.sequencer.undo() {
                                Ok(description) => {}, // info!("Undid: {}", description),
                                Err(e) => {}, // warn!("Cannot undo: {}", e),
                            }
                        }
                        2 => {
                            // Left key - Stop
                            info!("STOP TRIGGER: Key 2 (Left key) pressed - stopping sequencer");
                            self.sequencer.stop();
                            #[cfg(feature = "midi")]
                            self.midi.all_notes_off()?;
                        }
                        3 => {
                            // Right key - Start/Stop toggle
                            if self.sequencer.is_running() {
                                info!("STOP TRIGGER: Key 3 (Right key) pressed - stopping sequencer via toggle");
                                self.sequencer.stop();
                                #[cfg(feature = "midi")]
                                self.midi.all_notes_off()?;
                            } else {
                                // info!("Start pressed");
                                self.sequencer.start();
                            }
                        }
                        _ => {}
                    }
                }
            }

            HardwareEvent::StartStopToggle => {
                // Handle start/stop toggle
                if self.sequencer.is_running() {
                    info!("STOP TRIGGER: StartStopToggle hardware event - stopping sequencer");
                    self.sequencer.stop();
                    #[cfg(feature = "midi")]
                    self.midi.all_notes_off()?;
                } else {
                    // info!("Start pressed");
                    self.sequencer.start();

                    // TEMPORARILY DISABLED FOR DEBUGGING LED DROPPING ISSUE
                    // Flash all connected grids for visual feedback
                    // if let Err(e) = self.grid.flash_all_grids() {
                    //     // warn!("Failed to flash grids on sequencer start: {}", e);
                    // }
                }
            }

            HardwareEvent::GridPress { grid_id, x, y, pressed } => {
                self.handle_grid_press(&grid_id, x, y, pressed)?;
            }

            HardwareEvent::Shutdown => {
                info!("Shutdown requested - initiating immediate exit");
                self.running.store(false, Ordering::SeqCst);
                // More aggressive force exit
                thread::spawn(|| {
                    thread::sleep(Duration::from_millis(500));
                    warn!("Forcing immediate exit");
                    std::process::exit(0);
                });
            }
        }

        Ok(())
    }

    fn handle_sequencer_event(&mut self, event: SequencerEvent) -> Result<()> {
        match event {
            SequencerEvent::Step { step, bar } => {
                // Step event - display updates handled

                // Advance CO2 step counter
                let step_co2_value = self.co2.advance_step();

                // Process step for all active rows
                for row in 0..=6 { // Rows 0-6 are sequence rows
                    if let Some(note_events) = self.sequencer.get_step_events(row, bar, step) {
                        for note_event in note_events {
                            #[cfg(feature = "midi")]
                            {
                                if note_event.note_on {
                                    self.midi.sequencer_a_note_on(note_event.note, note_event.velocity, note_event.channel)?;
                                } else {
                                    self.midi.sequencer_a_note_off(note_event.note, note_event.channel)?;
                                }
                            }
                            #[cfg(not(feature = "midi"))]
                            {
                                if note_event.note_on {
                                    // info!("🎹 MIDI Note ON: {} vel:{} ch:{}", note_event.note, note_event.velocity, note_event.channel);
                                } else {
                                    // info!("🎹 MIDI Note OFF: {} ch:{}", note_event.note, note_event.channel);
                                }
                            }
                        }
                    }
                }

                // Handle CO2-influenced CV output for special rows
                if let Some(co2_value) = step_co2_value {
                    self.handle_co2_cv_output(step, 3, co2_value)?; // Row 3 uses step-based CO2
                }



                // Grid updates now handled by selective GridUpdate events
                // No need for full grid refresh on every step
            }

            SequencerEvent::Beat { beat } => {
                // Update any beat-based visual indicators
                self.screen.set_beat_indicator(beat);
            }

            SequencerEvent::GridUpdate { row, old_step, new_step } => {
                // Selective grid update - only update changed LEDs
                self.handle_grid_update(row, old_step, new_step)?;
            }

            SequencerEvent::MidiEvent(midi_event) => {
                // Handle MIDI events from sequencer - route based on source
                #[cfg(feature = "midi")]
                {
                    match midi_event.sequencer_source {
                        'A' => {
                            if midi_event.note_on {
                                self.midi.sequencer_a_note_on(midi_event.note, midi_event.velocity, midi_event.channel)?;
                                // info!("Sequencer A MIDI Note ON: {} vel:{} ch:{} step:{}",
                                //       midi_event.note, midi_event.velocity, midi_event.channel, midi_event.step);
                            } else {
                                self.midi.sequencer_a_note_off(midi_event.note, midi_event.channel)?;
                                // info!("Sequencer A MIDI Note OFF: {} ch:{} step:{}",
                                //       midi_event.note, midi_event.channel, midi_event.step);
                            }
                        }
                        'B' => {
                            // Sequencer B functionality removed
                        }
                        _ => {
                            warn!("Unknown sequencer source: {}", midi_event.sequencer_source);
                        }
                    }
                }
                #[cfg(not(feature = "midi"))]
                {
                    if midi_event.note_on {
                        // info!("MIDI Note ON: {} vel:{} ch:{} step:{}",
                        //       midi_event.note, midi_event.velocity, midi_event.channel, midi_event.step);
                    } else {
                        // info!("MIDI Note OFF: {} ch:{} step:{}",
                        //       midi_event.note, midi_event.channel, midi_event.step);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_grid_press(&mut self, grid_id: &str, x: usize, y: usize, pressed: bool) -> Result<()> {
        // DEBUG: Log ALL grid presses to trace Sequence B button issue
        info!("DEBUG Sequence B: Grid press {} at ({},{}) pressed={} - Sequence B active: {:?}", 
              grid_id, x, y, pressed, self.active_arm_action);
        
        // Handle ARM buttons first (row 7), even in Sequence B mode - BOTH press and release
        if y == 7 {
            let connected_grids = self.grid.get_connected_grids();
            let (grid_one, _) = self.get_sorted_grid_ids(&connected_grids);
            if Some(grid_id) == grid_one.as_ref().map(|x| x.as_str()) {
                if ArmAction::from_column(x).is_some() {
                    info!("DEBUG Sequence B: ARM button detected at column {} press={} - proceeding to ARM logic", x, pressed);
                    // This is an ARM button on GRID_ONE - process it directly
                    // Skip Sequence B mode check and go straight to ARM button logic
                    // (ARM button logic is later in this function)
                }
            }
        } else {

        }

        // Normal mode: Handle 32-step sequence input
        let connected_grids = self.grid.get_connected_grids();
        
        // Calculate actual sequence step (0-31)
        let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
        let seq_x = if Some(grid_id) == grid_one.as_ref().map(|x| x.as_str()) {
            // GRID_ONE (lowest ID): steps 0-15
            x
        } else if Some(grid_id) == grid_two.as_ref().map(|x| x.as_str()) {
            // GRID_TWO (second lowest ID): steps 16-31 (map from grid coordinates 0-15)
            x + 16
        } else {
            return Ok(()); // Invalid grid, ignore
        };
        let seq_y = y;
        
        info!("GRID DEBUG: Grid press on {} at grid({},{}) -> seq({},{}) pressed={}", 
              grid_id, x, y, seq_x, seq_y, pressed);
        
        // Debug grid ID mapping
        let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
        info!("GRID DEBUG: Available grids - GRID_ONE: {:?}, GRID_TWO: {:?}", grid_one, grid_two);
        info!("GRID DEBUG: This press came from: {}", grid_id);

        // Handle sequence rows and ARM controls
        if connected_grids.len() >= 2 {
            // Sequence rows 0-6 - only handle button presses, not releases
            if seq_y <= 6 && pressed {
                // Check if there's an active Euclidean ARM action
                if let Some(arm_action) = self.active_arm_action {
                    match arm_action {
                        ArmAction::EuclidianEvents => {
                            let events = (seq_x % 32) + 1; // Use full 32-step column + 1 for events (1-32)
                            info!("ARM EUCLIDIAN_EVENTS: Generating rhythm on row {} with {} events (step {})", seq_y, events, seq_x);
                            
                            // Get current euclidean parameters to preserve length and rotation
                            if let Some(row_state) = self.sequencer.get_row_states(seq_y) {
                                let current_length = row_state.sequencer_a_euclidean_length + 1; // Convert from 0-based to step count
                                let current_rotation = row_state.sequencer_a_euclidean_rotation;
                                self.sequencer.generate_euclidean_rhythm(seq_y, events, current_length, current_rotation);
                            } else {
                                // Fallback if row_state is not available
                                warn!("EuclidianEvents: Could not get row_state for row {}, using fallback defaults", seq_y);
                                self.sequencer.generate_euclidean_rhythm(seq_y, events, 32, 0);
                            }
                            
                            info!("ARM EUCLIDIAN_EVENTS: Successfully generated {} events on row {}", events, seq_y);
                            self.refresh_all_row_leds(seq_y)?;
                        },
                        ArmAction::EuclidianLength => {
                            let length = seq_x + 1; // Use full 32-step coordinate + 1 for length (1-32)
                            let length = length.clamp(1, 32); // Ensure valid range 1-32
                            info!("ARM EUCLIDIAN_LENGTH: Generating rhythm on row {} with length {} (step {})", seq_y, length, seq_x);
                            
                            // Get current euclidean parameters to preserve events and rotation
                            if let Some(row_state) = self.sequencer.get_row_states(seq_y) {
                                let current_events = row_state.sequencer_a_euclidean_events;
                                let current_rotation = row_state.sequencer_a_euclidean_rotation;
                                self.sequencer.generate_euclidean_rhythm(seq_y, current_events, length, current_rotation);
                            } else {
                                // Fallback if row_state is not available
                                warn!("EuclidianLength: Could not get row_state for row {}, using fallback defaults", seq_y);
                                self.sequencer.generate_euclidean_rhythm(seq_y, 5, length, 0);
                            }
                            
                            info!("ARM EUCLIDIAN_LENGTH: Successfully generated length {} on row {}", length, seq_y);
                            self.refresh_all_row_leds(seq_y)?;
                        },
                        ArmAction::EuclidianRotation => {
                            let rotation = seq_x % 32; // Use full 32-step coordinate for rotation (0-31)
                            info!("ARM EUCLIDIAN_ROTATION: Generating rhythm on row {} with rotation {} (step {})", seq_y, rotation, seq_x);
                            
                            // Get current euclidean parameters to preserve events and length
                            if let Some(row_state) = self.sequencer.get_row_states(seq_y) {
                                let current_events = row_state.sequencer_a_euclidean_events;
                                let current_length = row_state.sequencer_a_euclidean_length + 1; // Convert from 0-based to step count
                                self.sequencer.generate_euclidean_rhythm(seq_y, current_events, current_length, rotation);
                            } else {
                                // Fallback if row_state is not available
                                warn!("EuclidianRotation: Could not get row_state for row {}, using fallback defaults", seq_y);
                                self.sequencer.generate_euclidean_rhythm(seq_y, 5, 32, rotation);
                            }
                            
                            info!("ARM EUCLIDIAN_ROTATION: Successfully generated rotation {} on row {}", rotation, seq_y);
                            self.refresh_all_row_leds(seq_y)?;
                        },
                        ArmAction::SetSeqALength => {
                            let length = seq_x + 1; // Convert 0-based to 1-based (1-32)
                            let length = length.clamp(1, 32);
                            let last_step = length - 1; // Convert back to 0-based for internal storage (0-31)
                            
                            info!("ARM SET_SEQ_A_LENGTH: Setting row {} length to {} steps (last_step={})", seq_y, length, last_step);
                            
                            // Set the last step for this specific row
                            if let Some(mut row_state) = self.sequencer.get_row_states(seq_y) {
                                row_state.sequencer_a_euclidean_length = last_step;
                                
                                // If last_step is 31 (full 32 steps), sync this row with row 0 (master)
                                if last_step == 31 {
                                    if let Some(master_row_state) = self.sequencer.get_row_states(0) {
                                        row_state.sequencer_a_current_step = master_row_state.sequencer_a_current_step;
                                        info!("ARM SET_SEQ_A_LENGTH: Row {} synced with master row 0 (current_step={})", seq_y, row_state.sequencer_a_current_step);
                                    }
                                }
                                
                                self.sequencer.set_row_states(seq_y, row_state);
                                
                                info!("ARM SET_SEQ_A_LENGTH: Successfully set row {} length to {} steps", seq_y, length);
                                self.refresh_all_row_leds(seq_y)?;
                            }
                        },
                        _ => {
                            // Other ARM actions don't have grid-press behavior - handle normal grid operation
                            self.handle_normal_grid_operation(seq_x, seq_y)?;
                        }
                    }
                } else {
                    // No ARM action active - handle normal grid operation
                    self.handle_normal_grid_operation(seq_x, seq_y)?;
                }
            } else if seq_y == 7 {
                // Control row (7, 0-indexed) - handle both presses and releases
                
                // Check if this is GRID_TWO controls
                if connected_grids.len() >= 2 {
                    let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
                    if Some(grid_id) == grid_two.as_ref().map(|x| x.as_str()) {
                        // GRID_TWO MIDI auto-detection controls (columns 0, 1) - require both pressed
                        if x == 0 || x == 1 {
                            if pressed {
                                // Track button press state
                                if x == 0 {
                                    self.grid_two_button_0_pressed = true;
                                    info!("handle_grid_press says: GRID_TWO button 0 pressed for MIDI detection");
                                } else {
                                    self.grid_two_button_1_pressed = true;
                                    info!("handle_grid_press says: GRID_TWO button 1 pressed for MIDI detection");
                                }
                                
                                // Light up the pressed button
                                #[cfg(feature = "hardware")]
                                {
                                    self.grid.set_led(grid_id, x, seq_y, 10, "midi_detection_button_press")?;
                                    self.grid.refresh()?;
                                }
                                
                                // Check if both buttons are now pressed
                                if self.grid_two_button_0_pressed && self.grid_two_button_1_pressed {
                                    info!("handle_grid_press says: Both GRID_TWO buttons 0 and 1 pressed - triggering MIDI detection");
                                    #[cfg(feature = "midi")]
                                    {
                                        // First clear any saved device, then force detection
                                        if let Err(e) = self.midi.clear_detected_device() {
                                            warn!("handle_grid_press says: Failed to clear detected device: {}", e);
                                        }
                                        match self.midi.force_redetection() {
                                            Ok(true) => info!("handle_grid_press says: MIDI clock detection successful (both buttons)"),
                                            Ok(false) => warn!("handle_grid_press says: MIDI clock detection found no sources (both buttons)"),
                                            Err(e) => warn!("handle_grid_press says: MIDI clock detection failed: {}", e),
                                        }
                                    }
                                }
                            } else {
                                // Button release - update state and turn off LED
                                if x == 0 {
                                    self.grid_two_button_0_pressed = false;
                                    info!("handle_grid_press says: GRID_TWO button 0 released");
                                } else {
                                    self.grid_two_button_1_pressed = false;
                                    info!("handle_grid_press says: GRID_TWO button 1 released");
                                }
                                
                                #[cfg(feature = "hardware")]
                                {
                                    self.grid.set_led(grid_id, x, seq_y, 0, "midi_detection_button_release")?;
                                    self.grid.refresh()?;
                                }
                            }
                            return Ok(());
                        }

                        // GRID_TWO tempo controls (columns 10-15)
                        else if x >= 10 && x <= 15 {
                        // GRID_TWO transport and tempo controls - handle both press and release
                        if pressed {
                            // Button press - light LED and perform action
                            #[cfg(feature = "hardware")]
                            {
                                self.grid.set_led(grid_id, x, seq_y, 10, "transport_button_press")?;
                                self.grid.refresh()?;
                            }
                            
                            match x {
                                10 => {
                                    // Snap to whole tempo toggle button
                                    self.snap_to_whole_tempo = !self.snap_to_whole_tempo;
                                    #[cfg(feature = "midi")]
                                    self.midi.set_snap_to_whole_tempo(self.snap_to_whole_tempo);
                                    info!("handle_grid_press says: Snap to whole tempo {} via GRID_TWO column 10", 
                                          if self.snap_to_whole_tempo { "enabled" } else { "disabled" });
                                }
                                12 => {
                                    // Stop button - always works regardless of external clock
                                    info!("STOP TRIGGER: GRID_TWO column 12 pressed - stopping sequencer");
                                    self.sequencer.stop();
                                    info!("handle_grid_press says: Sequencer stopped via GRID_TWO column 12");
                                }
                                13 => {
                                    // Start button - always works regardless of external clock
                                    self.sequencer.start();
                                    info!("handle_grid_press says: Sequencer started via GRID_TWO column 13");
                                }
                                14 | 15 => {
                                    // Tempo controls - only work when external clock is not active
                                    if !self.midi.is_external_clock_running() {
                                        let current_tempo = self.sequencer.get_tempo();
                                        let new_tempo = if x == 14 {
                                            // Column 14: Decrease tempo
                                            (current_tempo - 1.0).clamp(20.0, 300.0)
                                        } else {
                                            // Column 15: Increase tempo
                                            (current_tempo + 1.0).clamp(20.0, 300.0)
                                        };
                                        
                                        if new_tempo != current_tempo {
                                            self.sequencer.set_tempo(new_tempo);
                                            self.tempo = new_tempo; // Keep main tempo in sync
                                            let snap_suffix = if self.snap_to_whole_tempo { " (snapped)" } else { "" };
                                            info!("handle_grid_press says: Tempo changed from {:.1} to {:.1} BPM{} via GRID_TWO column {}", 
                                                  current_tempo, new_tempo, snap_suffix, x);
                                        }
                                    } else {
                                        info!("handle_grid_press says: Tempo control ignored - external MIDI clock is active");
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            // Button release - turn off LED (except for toggle buttons)
                            #[cfg(feature = "hardware")]
                            {
                                let brightness = match x {
                                    10 => {
                                        // Snap tempo button - show state (ON/OFF)
                                        if self.snap_to_whole_tempo { LED_DIM_PLUS } else { LED_OFF }
                                    }
                                    12 | 13 => 0, // Transport buttons always turn off
                                    14 | 15 => {
                                        // Tempo LEDs stay off if external clock active (beat LEDs will handle them)
                                        if self.midi.is_external_clock_running() {
                                            return Ok(()); // Don't interfere with beat LED flashing
                                        } else {
                                            0
                                        }
                                    }
                                    _ => 0,
                                };
                                self.grid.set_led(grid_id, x, seq_y, brightness, "transport_button_release")?;
                                self.grid.refresh()?;
                            }
                        }
                        }
                        return Ok(());
                    }
                }
                
                // Regular ARM control handling for GRID_ONE buttons
                let (grid_one, _) = self.get_sorted_grid_ids(&connected_grids);
                if Some(grid_id) == grid_one.as_ref().map(|x| x.as_str()) {
                    info!("ARM CONTROL: Row 7 button {} {} - Current Sequence B: {:?}", x, if pressed { "PRESSED" } else { "RELEASED" }, self.active_arm_action);

                    // Check if this column corresponds to a valid ARM action (use original x, not seq_x)
                    if let Some(arm_action) = ArmAction::from_column(x) {
                        info!("DEBUG Sequence B: Found ARM action {:?} for column {}", arm_action, x);
                        
                        // ALL ARM buttons are now momentary (press-and-hold)
                        if pressed {
                            // ARM button pressed - activate mode
                            if !matches!(self.active_arm_action, Some(ref current) if *current == arm_action) {
                                // Turn off any other active ARM action first
                                if let Some(prev_action) = self.active_arm_action {
                                    let prev_column = prev_action.to_column();
                                    self.grid.set_led(grid_id, prev_column, seq_y, 0, "arm_action_deactivate")?;
                                }
                            }
                            self.active_arm_action = Some(arm_action);
                            info!("ARM CONTROL: {:?} PRESSED - mode ON (column {})", arm_action, x);
                            #[cfg(feature = "hardware")]
                            {
                                self.grid.set_led(grid_id, x, seq_y, LED_MAX, "arm_press")?;
                                self.grid.refresh()?;
                            }
                            self.handle_arm_action(arm_action)?;
                        } else {
                            // ARM button released - deactivate mode
                            if matches!(self.active_arm_action, Some(ref current) if *current == arm_action) {
                                self.active_arm_action = None;
                                info!("ARM CONTROL: {:?} RELEASED - mode OFF (column {})", arm_action, x);
                                #[cfg(feature = "hardware")]
                                {
                                    self.grid.set_led(grid_id, x, seq_y, 0, "arm_release")?;
                                    self.grid.refresh()?;
                                }
                            }
                        }
                    } else {
                        // info!("ARM CONTROL: ROW 7 column {} is not a valid ARM action", x);
                    }
                } else {
                    // Non-main grid, non-tempo button - ignore
                    info!("ARM CONTROL: Ignoring row 7 button {} on non-main grid {}", seq_x, grid_id);
                }
            }
        } else {
            warn!("Unexpected: Non-main grid event should have been filtered: {}", grid_id);
        }

        Ok(())
    }

    #[cfg(feature = "hardware")]
    fn handle_arm_action(&mut self, action: ArmAction) -> Result<()> {
        match action {
            ArmAction::Undo => {
                // Undo last action
                match self.sequencer.undo() {
                    Ok(description) => info!("ARM UNDO: Successfully undid: {}", description),
                    Err(e) => warn!("ARM UNDO: Cannot undo: {}", e),
                }
                // Grid updates will be handled naturally by sequencer scroll - no manual refresh needed
            }
            ArmAction::Redo => {
                // Redo last undone action
                match self.sequencer.redo() {
                    Ok(description) => info!("ARM REDO: Successfully redid: {}", description),
                    Err(e) => warn!("ARM REDO: Cannot redo: {}", e),
                }
                // Grid updates will be handled naturally by sequencer scroll - no manual refresh needed
            }
            ArmAction::EuclidianEvents => {
                // Euclidean Events ARM button activated - waiting for sequence row press
                info!("ARM EUCLIDIAN_EVENTS: ARM button activated - press sequence row at column N for N+1 events (max 32)");
                info!("ARM EUCLIDIAN_EVENTS: GRID_ONE columns 0-15 = events 1-16, GRID_TWO columns 0-15 = events 17-32");
            }
            ArmAction::EuclidianLength => {
                // Euclidean Length ARM button activated - waiting for sequence row press
                info!("ARM EUCLIDIAN_LENGTH: ARM button activated - press sequence row at column N for length N+1 (max 32)");
                info!("ARM EUCLIDIAN_LENGTH: GRID_ONE columns 0-15 = lengths 1-16, GRID_TWO columns 0-15 = lengths 17-32");
            }
            ArmAction::EuclidianRotation => {
                // Euclidean Rotation ARM button activated - waiting for sequence row press
                info!("ARM EUCLIDIAN_ROTATION: ARM button activated - press sequence row at column N for rotation N (0-31)");
                info!("ARM EUCLIDIAN_ROTATION: GRID_ONE columns 0-15 = rotation 0-15, GRID_TWO columns 0-15 = rotation 16-31");
            }
            ArmAction::Ratchet => {
                // Ratchet functionality - placeholder
                info!("ARM RATCHET: ARM button activated - not yet implemented");
            }
            ArmAction::SetSeqALength => {
                // Set Seq A Length ARM button activated - waiting for sequence row press
                info!("ARM SET_SEQ_A_LENGTH: ARM button activated - press sequence row at column N for length N+1 (max 32)");
                info!("ARM SET_SEQ_A_LENGTH: GRID_ONE columns 0-15 = lengths 1-16, GRID_TWO columns 0-15 = lengths 17-32");
                info!("ARM SET_SEQ_A_LENGTH: When set to 32 steps, row will sync with master row 0");
            }
            ArmAction::PresetGrid => {
                // Preset grid functionality - placeholder
                info!("ARM PRESET_GRID: ARM button activated - not yet implemented");
            }

        }
        Ok(())
    }









    fn update_screen(&mut self) -> Result<()> {
        self.screen.clear();

        // Display tempo
        self.screen.draw_text(1, 7, &format!("Tempo: {:.1}", self.sequencer.get_tempo()));

        // Display current step/bar
        let (step, bar) = self.sequencer.get_position();
        self.screen.draw_text(1, 21, &format!("Step: {} Bar: {}", step, bar));

        // Display transport state
        let transport_text = if self.sequencer.is_running() { "RUNNING" } else { "STOPPED" };
        self.screen.draw_text(1, 35, transport_text);

        // Display MIDI activity
        #[cfg(feature = "midi")]
        if let Some(last_note) = self.midi.get_last_note() {
            self.screen.draw_text(1, 49, &format!("MIDI: {}", last_note));
        }

        // Show beat indicators if enabled
        if self.config.display.show_beat_indicators {
            self.screen.draw_beat_indicator();
        }

        // Show tempo visualization if enabled
        if self.config.display.show_tempo_viz {
            self.screen.draw_tempo_viz(self.sequencer.get_tempo());
        }

        self.screen.update()?;

        Ok(())
    }

    /// Selective grid update - only update specific LEDs that changed
    /// 
    /// LED BRIGHTNESS SYSTEM (4 levels):
    ///   - Brightness 0:  No pattern, not current position (OFF - dark)
    ///   - Brightness 6:  No pattern, IS current position (MEDIUM - position indicator only)
    ///   - Brightness 10: Has pattern, not current position (DIM - shows programmed beats)
    ///   - Brightness 14: Has pattern, IS current position (BRIGHT - pattern + position highlight)
    ///
    /// This creates the "moving playhead" effect by updating only 2 LEDs per row:
    ///   1. OLD position: Reverts to pattern-only brightness (10 if pattern, 0 if empty)  
    ///   2. NEW position: Shows position highlight (14 if pattern+position, 6 if position-only)
    /// 
    /// This is much more efficient than refreshing the entire grid every step (32x7=224 LEDs)
    /// vs selective update (2 LEDs per active row = ~14 LEDs per step)
    fn handle_grid_update(&mut self, row: usize, old_step: usize, new_step: usize) -> Result<()> {

        
        let connected_grids = self.grid.get_connected_grids();
        
        // DEBUG: Log grid update details
        info!("GRID_DEBUG: handle_grid_update called - row: {}, old_step: {}, new_step: {}", row, old_step, new_step);
        info!("GRID_DEBUG: Connected grids: {:?}", connected_grids);

        // DEBUG: Focused tracking for row 0 LED updates
        if row == 0 {
            info!("🔥 LED HANDLER Row 0: Processing LED update old_step={} -> new_step={}", old_step, new_step);
        }
        
        if connected_grids.len() >= 2 {
            // DUAL-GRID MODE (32-step sequences):
            // When 2+ grids are connected, we use the first two as GRID_ONE and GRID_TWO
            // This creates a seamless 32-step sequence: steps 0-15 on left grid, 16-31 on right grid
            // Users can see and edit the full 32-step pattern across both grids simultaneously
            let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
            let grid_one_id = grid_one.as_ref().unwrap();
            let grid_two_id = grid_two.as_ref().unwrap();
            
            // DEBUG: Log grid ID assignments
            info!("GRID_DEBUG: Grid assignments - GRID_ONE: {}, GRID_TWO: {}", grid_one_id, grid_two_id);
            
            if let Some(row_state) = self.sequencer.get_row_states(row) {
                // UPDATE OLD POSITION LED (remove position highlight, keep pattern visibility)
                //
                // We MUST check old_pattern_value because when the playhead moves away from a step,
                // we need to know: "Was there a programmed beat at this position?"
                //   
                // If YES (pattern exists): Set brightness to 10 (dim but visible - shows the beat)
                // If NO  (empty step):     Set brightness to 0  (turn off - nothing there)
                //
                // Without this check, ALL old positions would go dark, hiding the pattern data.
                // Users need to see both: WHERE beats are programmed (dim LEDs) AND where the playhead is (bright LED)
                let old_pattern_value = self.sequencer.get_grid_value(old_step, row);
                let old_brightness = if old_pattern_value > 0 { LED_BRIGHT } else { LED_OFF };
                
                // DUAL-GRID COORDINATE MAPPING:
                // Steps 0-15  → GRID_ONE (coordinates 0-15)  
                // Steps 16-31 → GRID_TWO (coordinates 0-15, mapped via: grid_x = step - 16)
                //
                // This creates a seamless 32-step sequence across two 16-step grids
                if old_step <= 15 {
                    // OLD position is on GRID_ONE (left grid): Direct coordinate mapping
                    info!("GRID_DEBUG: Setting OLD LED on GRID_ONE: grid_id={}, x={}, y={}, brightness={}", grid_one_id, old_step, row, old_brightness);
                    self.grid.set_led(grid_one_id, old_step, row, old_brightness, "grid_update_old_1")?;
                } else if old_step > 15 && old_step <= 31 {
                    // OLD position is on GRID_TWO (right grid): Coordinate mapping required
                    // Step 16 becomes grid_x=0, step 17 becomes grid_x=1, etc.
                    let grid_x = old_step - 16;
                    info!("GRID_DEBUG: Setting OLD LED on GRID_TWO: grid_id={}, x={}, y={}, brightness={} (original_step={})", grid_two_id, grid_x, row, old_brightness, old_step);
                    self.grid.set_led(grid_two_id, grid_x, row, old_brightness, "grid_update_old_2")?;
                } else {
                    warn!("GRID_DEBUG: OLD step {} is out of bounds (valid range: 0-31), skipping LED update for row {}", old_step, row);
                }
                
                // UPDATE NEW POSITION LED (add position highlight, preserve pattern info)
                //
                // We MUST check new_pattern_value to create the correct brightness for the current position:
                //
                // If pattern EXISTS at current step: Brightness 14 (BRIGHTEST)
                //   - User sees: "There's a beat HERE and playhead is HERE" (pattern + position)
                //
                // If pattern is EMPTY at current step: Brightness 6 (MEDIUM)  
                //   - User sees: "No beat here, but playhead is HERE" (position only)
                //
                // This dual-brightness system lets users instantly distinguish:
                //   - Steps WITH beats that are playing (brightness 14)
                //   - Steps WITHOUT beats where playhead is just passing through (brightness 6)
                let new_pattern_value = self.sequencer.get_grid_value(new_step, row);
                let new_brightness = if new_pattern_value > 0 { LED_MAX } else { LED_DIM };
                
                // Same coordinate mapping logic applies to NEW position
                if new_step <= 15 {
                    // NEW position is on GRID_ONE: Direct coordinate mapping  
                    info!("GRID_DEBUG: Setting NEW LED on GRID_ONE: grid_id={}, x={}, y={}, brightness={}", grid_one_id, new_step, row, new_brightness);
                    self.grid.set_led(grid_one_id, new_step, row, new_brightness, "grid_update_new_1")?;
                } else if new_step > 15 && new_step <= 31 {
                    // NEW position is on GRID_TWO: Coordinate mapping required
                    let grid_x = new_step - 16;
                    info!("GRID_DEBUG: Setting NEW LED on GRID_TWO: grid_id={}, x={}, y={}, brightness={} (original_step={})", grid_two_id, grid_x, row, new_brightness, new_step);
                    self.grid.set_led(grid_two_id, grid_x, row, new_brightness, "grid_update_new_2")?;
                } else {
                    warn!("GRID_DEBUG: NEW step {} is out of bounds (valid range: 0-31), skipping LED update for row {}", new_step, row);
                }
            }
        }
        
        // TEMPORARILY DISABLED FOR DEBUGGING LED DROPPING ISSUE
        // Update beat LEDs and ARM button LEDs before final refresh
        // self.update_beat_leds()?;
        // self.update_arm_button_leds()?;
        
        self.grid.refresh()?;
        Ok(())
    }

    /// Full grid display update (only used for initialization)
    #[cfg(feature = "hardware")]
    fn update_grid_display(&mut self) -> Result<()> {
        let connected_grids = self.grid.get_connected_grids();
        info!("DEBUG: update_grid_display called - found {} connected grids: {:?}", connected_grids.len(), connected_grids);



        // Normal mode: Grid will be naturally painted by selective updates during playback
        // No need for full grid refresh - let the sequencer paint the display as it runs
        if connected_grids.len() >= 2 {
            info!("GRID DEBUG: Dual-grid mode active - display will be painted by selective updates");
        } else {
            error!("ERROR: Dual-grid mode required - {} grids connected", connected_grids.len());
        }

        // TEMPORARILY DISABLED FOR DEBUGGING LED DROPPING ISSUE
        // Update beat LEDs and ARM button LEDs before final refresh
        // self.update_beat_leds()?;
        // self.update_arm_button_leds()?;
        
        self.grid.refresh()?;
        Ok(())
    }

    #[cfg(feature = "hardware")]
    fn update_main_grid_display(&mut self, grid_id: &str) -> Result<()> {
        // Grid display with position scrolling - 4 brightness levels - ROWS 0-6 (0-indexed)
        for seq_y in 0..=6 {
            let row_states = self.sequencer.get_row_states(seq_y);
            if let Some(row_state) = row_states {
                // Debug row state every few updates
                static mut DEBUG_COUNTER: u32 = 0;
                unsafe {
                    DEBUG_COUNTER += 1;
                    if DEBUG_COUNTER % 20 == 0 && seq_y <= 6 { // Debug all 7 sequencer rows, every 20 updates
                        // info!("🎯 Row {} current_step = {} (first_step={}, euclidean_length={}) [display: row {}]",
                        //       seq_y, row_state.current_step, row_state.first_step, row_state.euclidean_length, seq_y + 1);
                    }
                }

                // Update this row's LEDs based on pattern values and current step
                for seq_x in 0..=15 {
                    let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                    let is_current_step = seq_x == row_state.sequencer_a_current_step;

                    // Calculate brightness based on pattern and current position
                    // Enhanced brightness for better scroll position visibility
                    let brightness = match (pattern_value > 0, is_current_step) {
                        (false, false) => LED_OFF,     // No pattern, not current position
                        (false, true) => LED_DIM_PLUS, // No pattern, but current position (enhanced visibility)
                        (true, false) => LED_BRIGHT,   // Has pattern, not current position
                        (true, true) => LED_MAX,       // Has pattern AND current position (maximum brightness)
                    };

                    // Use native 0-based grid coordinates directly
                    self.grid.set_led(grid_id, seq_x, seq_y, brightness, "update_main_grid_display")?;
                }
            } else {
                // info!("No row state for row {}", seq_y);
            }
        }
        Ok(())
    }


    /// Update single LED with current pattern and position state
    #[cfg(feature = "hardware")]
    fn update_single_led(&mut self, seq_x: usize, seq_y: usize) -> Result<()> {

        
        // Only update LEDs for rows 0-6 (0-indexed)
        if seq_y > 6 {
            return Ok(());
        }

        let connected_grids = self.grid.get_connected_grids();
        
        if connected_grids.len() >= 2 {
            // 32-step mode: Route to correct grid
            let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
            let grid_one_id = grid_one.as_ref().unwrap();
            let grid_two_id = grid_two.as_ref().unwrap();
            
            if let Some(row_state) = self.sequencer.get_row_states(seq_y) {
                let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                let is_current_step = seq_x == row_state.sequencer_a_current_step;

                let brightness = match (pattern_value > 0, is_current_step) {
                    (false, false) => LED_OFF,
                    (false, true) => LED_DIM,
                    (true, false) => LED_BRIGHT,
                    (true, true) => LED_MAX,
                };

                // Map sequence coordinates to correct grid
                if seq_x <= 15 {
                    self.grid.set_led(grid_one_id, seq_x, seq_y, brightness, "update_single_led_grid1")?;
                } else if seq_x <= 31 {
                    let grid_x = seq_x - 16;
                    self.grid.set_led(grid_two_id, grid_x, seq_y, brightness, "update_single_led_grid2")?;
                }
            }
        }

        Ok(())
    }

    /// Update single LED after button press - only updates the specific pressed button
    #[cfg(feature = "hardware")]
    fn update_single_button_led(&mut self, seq_x: usize, seq_y: usize) -> Result<()> {
        let connected_grids = self.grid.get_connected_grids();
        
        if connected_grids.len() >= 2 {
            // 32-step mode: Route to correct grid
            let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
            let grid_one_id = grid_one.as_ref().unwrap();
            let grid_two_id = grid_two.as_ref().unwrap();
            
            if let Some(row_state) = self.sequencer.get_row_states(seq_y) {
                let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                let is_current_step = seq_x == row_state.sequencer_a_current_step;
                
                let brightness = match (pattern_value > 0, is_current_step) {
                    (false, false) => LED_OFF,     // No pattern, not current position
                    (false, true) => LED_DIM,      // No pattern, but current position
                    (true, false) => LED_BRIGHT,     // Has pattern, not current position
                    (true, true) => LED_MAX,      // Has pattern AND current position
                };
                
                info!("GRID DEBUG: Setting single LED seq_x={}, seq_y={}, brightness={}, pattern_value={}, is_current={}", 
                      seq_x, seq_y, brightness, pattern_value, is_current_step);
                
                // Route to correct grid based on step position
                if seq_x <= 15 {
                    self.grid.set_led(grid_one_id, seq_x, seq_y, brightness, "single_button_grid1")?;
                } else if seq_x <= 31 {
                    let grid_x = seq_x - 16;
                    self.grid.set_led(grid_two_id, grid_x, seq_y, brightness, "single_button_grid2")?;
                }
                
                // TEMPORARILY DISABLED FOR DEBUGGING LED DROPPING ISSUE
                // Update beat LEDs and ARM button LEDs before refresh
                // self.update_beat_leds()?;
                // self.update_arm_button_leds()?;
                
                self.grid.refresh()?;
            }
        }
        
        Ok(())
    }




    /// Get sorted grid IDs - returns (GRID_ONE, GRID_TWO)
    fn get_sorted_grid_ids(&self, connected_grids: &[String]) -> (Option<String>, Option<String>) {
        // DEBUG: Track grid ID consistency
        info!("GRID_DEBUG: get_sorted_grid_ids called with {} grids: {:?}", connected_grids.len(), connected_grids);
        
        // Always sort grids by ID for consistency
        let mut sorted_grids = connected_grids.to_vec();
        sorted_grids.sort();
        
        let grid_one = sorted_grids.first().cloned();
        let grid_two = if sorted_grids.len() > 1 { sorted_grids.get(1).cloned() } else { None };
        
        info!("GRID_DEBUG: Sorted result - GRID_ONE (lowest ID): {:?}", grid_one);
        info!("GRID_DEBUG: Sorted result - GRID_TWO (second lowest ID): {:?}", grid_two);
        
        (grid_one, grid_two)
    }




    // Helper methods for advanced grid operations

    fn has_held_positions(&self) -> bool {
        // Check if any grid positions are being held
        for x in 1..=16 {
            for y in 1..=8 {
                if self.sequencer.is_held(x, y) {
                    return true;
                }
            }
        }
        false
    }

    fn get_held_rows(&self) -> Vec<usize> {
        let mut held_rows = Vec::new();
        for y in 1..=8 {
            for x in 1..=16 {
                if self.sequencer.is_held(x, y) && !held_rows.contains(&y) {
                    held_rows.push(y);
                }
            }
        }
        held_rows
    }

    fn get_first_held_row(&self) -> Option<usize> {
        for y in 0..=7 {
            for x in 0..=31 {
                if self.sequencer.is_held(x, y) {
                    return Some(y);
                }
            }
        }
        None
    }

    /// Check if any ARM action is currently active
    fn has_active_arm_action(&self) -> bool {
        self.active_arm_action.is_some()
    }

    /// Get the currently active ARM action
    fn get_active_arm_action(&self) -> Option<ArmAction> {
        self.active_arm_action
    }

    /// Check if specific ARM action is active
    fn is_arm_action_active(&self, action: ArmAction) -> bool {
        self.active_arm_action == Some(action)
    }



    /// Refresh all pattern LEDs on the grid (used after operations that change multiple positions)
    #[cfg(feature = "hardware")]
    fn refresh_all_pattern_leds(&mut self) -> Result<()> {

        
        info!("ARM DEBUG: refresh_all_pattern_leds() called");
        let connected_grids = self.grid.get_connected_grids();
        info!("ARM DEBUG: Found {} connected grids: {:?}", connected_grids.len(), connected_grids);
        
        if connected_grids.len() >= 2 {
            // 32-step mode: Update both grids
            let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
            let grid_one_id = grid_one.as_ref().unwrap();
            let grid_two_id = grid_two.as_ref().unwrap();
            
            info!("ARM DEBUG: Using dual-grid mode - GRID_ONE: {}, GRID_TWO: {}", grid_one_id, grid_two_id);
            
            for seq_y in 0..=6 {
                if let Some(row_state) = self.sequencer.get_row_states(seq_y) {
                    for seq_x in 0..=31 {
                        let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                        let is_current_step = seq_x == row_state.sequencer_a_current_step;
                        
                        let brightness = match (pattern_value > 0, is_current_step) {
                            (false, false) => LED_OFF,     // No pattern, not current position
                            (false, true) => LED_DIM,      // No pattern, but current position  
                            (true, false) => LED_BRIGHT,     // Has pattern, not current position
                            (true, true) => LED_MAX,      // Has pattern AND current position
                        };
                        
                        if brightness > 0 {
                            info!("ARM DEBUG: Setting LED seq_x={}, seq_y={}, brightness={}", seq_x, seq_y, brightness);
                        }
                        
                        // Map sequence coordinates to correct grid
                        if seq_x <= 15 {
                            self.grid.set_led(grid_one_id, seq_x, seq_y, brightness, "refresh_pattern_grid1")?;
                        } else if seq_x <= 31 {
                            let grid_x = seq_x - 16;
                            self.grid.set_led(grid_two_id, grid_x, seq_y, brightness, "refresh_pattern_grid2")?;
                        }
                    }
                }
            }
        }
        
        info!("ARM DEBUG: Calling grid.refresh()");
        self.grid.refresh()?;
        info!("ARM DEBUG: refresh_all_pattern_leds() completed successfully");
        Ok(())
    }

    /// Refresh LEDs for a specific row (used after operations that change one row)
    #[cfg(feature = "hardware")]
    fn refresh_all_row_leds(&mut self, row: usize) -> Result<()> {

        
        let connected_grids = self.grid.get_connected_grids();
        
        if connected_grids.len() >= 2 {
            // 32-step mode: Update both grids
            let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
            let grid_one_id = grid_one.as_ref().unwrap();
            let grid_two_id = grid_two.as_ref().unwrap();
            
            info!("ARM DEBUG: refresh_all_row_leds() called for row {} on both grids", row);
            
            if let Some(row_state) = self.sequencer.get_row_states(row) {
                info!("ARM DEBUG: Got row state for row {}: current_step={}", row, row_state.sequencer_a_current_step);
                
                for seq_x in 0..=31 {
                    let pattern_value = self.sequencer.get_grid_value(seq_x, row);
                    let is_current_step = seq_x == row_state.sequencer_a_current_step;
                    
                    let brightness = match (pattern_value > 0, is_current_step) {
                        (false, false) => LED_OFF,
                        (false, true) => LED_DIM,
                        (true, false) => LED_BRIGHT,
                        (true, true) => LED_MAX,
                    };
                    
                    if seq_x <= 15 {
                        self.grid.set_led(grid_one_id, seq_x, row, brightness, "refresh_row_grid1")?;
                    } else if seq_x <= 31 {
                        let grid_x = seq_x - 16;
                        self.grid.set_led(grid_two_id, grid_x, row, brightness, "refresh_row_grid2")?;
                    }
                }
                self.grid.refresh()?;
                info!("ARM DEBUG: refresh_all_row_leds() completed successfully for row {}", row);
            }
        }
        Ok(())
    }

    /// Handle normal grid operation (toggle pattern)
    #[cfg(feature = "hardware")]
    fn handle_normal_grid_operation(&mut self, seq_x: usize, seq_y: usize) -> Result<()> {
        // Check if any positions are held for advanced operations
        if self.has_held_positions() {
            self.handle_advanced_grid_operation(seq_x, seq_y)?;
        } else {
            // Normal grid operation - toggle or cycle ratchet
            let current_value = self.sequencer.get_grid_value(seq_x, seq_y);
            let new_value = if current_value > 0 { 0 } else { 1 }; // Simple on/off toggle

            info!("GRID DEBUG: Toggling seq_x={}, seq_y={} from {} to {}", seq_x, seq_y, current_value, new_value);
            self.sequencer.set_grid_value(seq_x, seq_y, new_value);
            info!("GRID DEBUG: Grid value set successfully, calling update_single_button_led()");

            // Update only the specific LED that was pressed
            self.update_single_button_led(seq_x, seq_y)?;
            info!("GRID DEBUG: update_single_button_led() completed");
        }
        Ok(())
    }

    fn handle_advanced_grid_operation(&mut self, x: usize, y: usize) -> Result<()> {
        let held_rows = self.get_held_rows();

        if held_rows.len() == 1 {
            let held_row = held_rows[0];

            match held_row {
                8 => {
                    // Control row held - special functions
                    if x <= 8 {
                        // Randomize column
                        // Only randomize rows 0 and 1
                        self.sequencer.randomize_section(x, 0);
                        self.sequencer.randomize_section(x, 1);
                        info!("Randomized column {}", x);
                    } else {
                        // Clear column
                        // Only clear rows 1 and 2
                        self.sequencer.set_grid_value(x, 1, 0);
                        self.sequencer.set_grid_value(x, 2, 0);
                        info!("Cleared column {}", x);
                    }
                }
                _ => {
                    // Sequence row held - copy/paste operations
                    if y == held_row {
                        // Same row - randomize this position
                        self.sequencer.randomize_section(x, y);
                        info!("Randomized position [{}, {}]", x, y);
                    } else {
                        // Different row - copy from held row to this row
                        self.sequencer.copy_grid_section(x, held_row, x, y, 1, 1);
                        info!("Copied from [{}, {}] to [{}, {}]", x, held_row, x, y);
                    }
                }
            }
        } else if held_rows.len() > 1 {
            // Multiple rows held - advanced operations
            let first_row = self.get_first_held_row().unwrap_or(0);

            // Copy pattern from first held row to current position
            self.sequencer.copy_grid_section(0, first_row, 0, y, 16, 1);
            info!("Copied pattern from row {} to row {}", first_row, y);
        }

        // Update grid display
        #[cfg(feature = "hardware")]
        self.update_grid_display()?;

        Ok(())
    }





    fn handle_co2_cv_output(&mut self, step: usize, row: usize, co2_value: f32) -> Result<()> {
        // Get Mozart note value for this position
        let mozart_note = self.sequencer.get_mozart_value(step, row) as f32;

        // Calculate CO2 offset
        let co2_offset = self.co2.get_step_offset(co2_value);

        // Combined voltage: CO2 offset + musical note (scaled to voltage)
        let voltage = co2_offset + (mozart_note / 12.0);

        // In a real implementation, this would send to Crow CV output
        debug!("CV Output Row {}: {:.3}V (CO2: {:.2} ppm, Note: {}, Offset: {:.3})",
               row, voltage, co2_value, mozart_note, co2_offset);

        // TODO: Add actual Crow CV output when hardware support is added

        Ok(())
    }

    /// Handle MIDI input events
    #[cfg(feature = "midi")]
    fn handle_midi_input_event(&mut self, event: crate::midi::MidiInputEvent) -> Result<()> {
        use crate::midi::MidiInputEvent;
        
        match event {
            MidiInputEvent::ClockBeat => {
                // TEMPORARILY DISABLED FOR DEBUGGING LED DROPPING ISSUE
                // Flash LEDs 14 and 15 on GRID_TWO for 150ms
                // self.beat_led_flash_until = Some(Instant::now() + Duration::from_millis(150));
                
                // Synchronize sequencer tempo with external MIDI clock (less frequent logging)
                #[cfg(feature = "midi")]
                {
                    if let Some(external_tempo) = self.midi.get_external_tempo() {
                        let current_tempo = self.sequencer.get_tempo();
                        if (external_tempo - current_tempo).abs() > 0.5 {
                            self.sequencer.set_tempo(external_tempo);
                            self.tempo = external_tempo; // Keep main tempo in sync
                            if self.snap_to_whole_tempo {
                                info!("handle_midi_input_event says: Applied snapped tempo to sequencer: {:.0} BPM", external_tempo);
                            } else {
                                info!("handle_midi_input_event says: Applied tempo to sequencer: {:.1} BPM (no snapping)", external_tempo);
                            }
                        }
                    }
                    
                    // Phase correction - check for drift every beat
                    self.check_phase_correction()?;
                }
                
                let current_tempo = self.sequencer.get_tempo();
                let snap_suffix = if self.snap_to_whole_tempo { " (snapped)" } else { "" };
                debug!("handle_midi_input_event says: MIDI Clock Beat - flashing tempo LEDs at {:.1} BPM{}", current_tempo, snap_suffix);
            }
            MidiInputEvent::ClockStart => {
                info!("handle_midi_input_event says: MIDI Clock Start received - starting sequencer");
                self.sequencer.start();
                
                // Reset phase correction state on new start
                self.reset_phase_correction();
                
                // Synchronize sequencer tempo with external MIDI clock on start
                #[cfg(feature = "midi")]
                {
                    if let Some(external_tempo) = self.midi.get_external_tempo() {
                        let current_tempo = self.sequencer.get_tempo();
                        if (external_tempo - current_tempo).abs() > 0.5 {
                            self.sequencer.set_tempo(external_tempo);
                            self.tempo = external_tempo; // Keep main tempo in sync
                            if self.snap_to_whole_tempo {
                                info!("handle_midi_input_event says: Applied snapped tempo to sequencer on start: {:.0} BPM", external_tempo);
                            } else {
                                info!("handle_midi_input_event says: Applied tempo to sequencer on start: {:.1} BPM (no snapping)", external_tempo);
                            }
                        }
                    }
                }
            }
            MidiInputEvent::ClockStop => {
                info!("STOP TRIGGER: MIDI Clock Stop received from external device - stopping sequencer");
                info!("handle_midi_input_event says: MIDI Clock Stop received - stopping sequencer");
                self.sequencer.stop();
                // TEMPORARILY DISABLED FOR DEBUGGING LED DROPPING ISSUE
                // Clear beat LEDs when clock stops
                // self.beat_led_flash_until = None;
                // Reset phase correction state
                self.reset_phase_correction();
            }
            MidiInputEvent::ClockTick => {
                // Log every 100th tick for connection diagnostics
                static mut TICK_COUNTER: u32 = 0;
                unsafe {
                    TICK_COUNTER += 1;
                    if TICK_COUNTER % 100 == 0 {
                        debug!("handle_midi_input_event says: MIDI Clock Tick #{} - connection active", TICK_COUNTER);
                    }
                }
            }
            MidiInputEvent::ExternalClockTimeout => {
                // External MIDI clock timed out - preserve the last known external tempo
                #[cfg(feature = "midi")]
                {
                    if let Some(last_external_tempo) = self.midi.get_external_tempo() {
                        self.sequencer.set_tempo(last_external_tempo);
                        self.tempo = last_external_tempo; // Keep main tempo in sync
                        if self.snap_to_whole_tempo {
                            info!("handle_midi_input_event says: External clock timeout - preserving snapped tempo: {:.0} BPM", last_external_tempo);
                        } else {
                            info!("handle_midi_input_event says: External clock timeout - preserving tempo: {:.1} BPM (no snapping)", last_external_tempo);
                        }
                    } else {
                        info!("handle_midi_input_event says: External clock timeout - no previous external tempo to preserve");
                    }
                }
            }
            MidiInputEvent::NoteOn { channel, note, velocity } => {
                // Sequencer B MIDI recording functionality removed
                info!("MIDI Note On (sequencer B removed): Channel: {}, Note: {}, Velocity: {}", channel, note, velocity);
            }
            MidiInputEvent::NoteOff { channel, note } => {
                // Sequencer B MIDI recording functionality removed  
                info!("MIDI Note Off (sequencer B removed): Channel: {}, Note: {}", channel, note);
            }
            MidiInputEvent::ControlChange { .. } => {
                // Control Change events - currently not handled in main app
                // Could be used for MIDI CC mapping in future
            }
            MidiInputEvent::ClockContinue => {
                // MIDI Clock Continue - currently not handled
                // Similar to ClockStart but resumes from current position
            }
        }
        
        Ok(())
    }

    /// Update beat LEDs on GRID_TWO to flash with external MIDI clock beats
    fn update_beat_leds(&mut self) -> Result<()> {
        // TEMPORARILY DISABLED FOR DEBUGGING LED DROPPING ISSUE
        // let connected_grids = self.grid.get_connected_grids();
        // if connected_grids.len() >= 2 {
        //     let (_, grid_two) = self.get_sorted_grid_ids(&connected_grids);
        //     if let Some(grid_two_id) = grid_two {
        //         let brightness = if let Some(flash_until) = self.beat_led_flash_until {
        //             if Instant::now() < flash_until {
        //                 LED_DIM_PLUS // Dimmer flash
        //             } else {
        //                 self.beat_led_flash_until = None;
        //                 0  // Turn off
        //             }
        //         } else {
        //             0  // Off by default
        //         };
        //         
        //         #[cfg(feature = "hardware")]
        //         {
        //             self.grid.set_led(&grid_two_id, 14, 7, brightness, "beat_led_flash")?;
        //             self.grid.set_led(&grid_two_id, 15, 7, brightness, "beat_led_flash")?;
        //             
        //             // Update snap tempo button LED (column 10, row 7)
        //             let snap_brightness = if self.snap_to_whole_tempo { LED_DIM_PLUS } else { LED_OFF };
        //             self.grid.set_led(&grid_two_id, 10, 7, snap_brightness, "snap_tempo_button")?;
        //             
        //             // Update drift indicator LED (column 11, row 7)
        //             let drift_brightness = self.calculate_drift_brightness();
        //             self.grid.set_led(&grid_two_id, 11, 7, drift_brightness, "drift_indicator")?;
        //         }
        //     }
        // }
        
        Ok(())
    }

    /// Update ARM button LEDs to maintain their state based on active ARM action
    fn update_arm_button_leds(&mut self) -> Result<()> {
        // TEMPORARILY DISABLED FOR DEBUGGING LED DROPPING ISSUE
        // let connected_grids = self.grid.get_connected_grids();
        // let (grid_one, _) = self.get_sorted_grid_ids(&connected_grids);
        // if let Some(grid_one_id) = grid_one {
        //     #[cfg(feature = "hardware")]
        //     {
        //         // Update ARM button LED based on active ARM action
        //         if let Some(active_action) = self.active_arm_action {
        //             let active_column = active_action.to_column();
        //             // Keep active ARM button lit at full brightness
        //             self.grid.set_led(&grid_one_id, active_column, 7, LED_MAX, "arm_button_maintain")?;
        //         } else {
        //             // Turn off all ARM button LEDs when no ARM action is active
        //             for column in [0, 1, 4, 5, 6, 7, 10, 15] { // All ARM action columns
        //                 self.grid.set_led(&grid_one_id, column, 7, LED_OFF, "arm_button_clear")?;
        //             }
        //         }
        //     }
        // }
        Ok(())
    }

    /// Check phase correction and apply gentle corrections to prevent drift
    #[cfg(feature = "midi")]
    fn check_phase_correction(&mut self) -> Result<()> {
        if !self.midi.is_external_clock_running() {
            return Ok(());
        }

        // Get current MIDI clock state
        let (_, _, midi_running) = self.midi.get_clock_info();
        if !midi_running {
            return Ok(());
        }

        // Check tempo stability - only apply phase correction if tempo has been stable for 10 seconds
        if let Some(external_tempo) = self.midi.get_external_tempo() {
            let now = Instant::now();
            let tempo_changed = if let Some(last_tempo) = self.last_stable_tempo {
                (external_tempo - last_tempo).abs() > 2.0 // Consider tempo stable if within 2 BPM
            } else {
                true // No previous tempo, consider it changed
            };

            if tempo_changed {
                // Tempo changed, reset stability timer
                self.tempo_stable_since = Some(now);
                self.last_stable_tempo = Some(external_tempo);
                debug!("check_phase_correction says: Tempo changed to {:.1} BPM, resetting stability timer", external_tempo);
                return Ok(());
            } else if let Some(stable_since) = self.tempo_stable_since {
                if now.duration_since(stable_since).as_secs() < 10 {
                    // Tempo hasn't been stable for 10 seconds yet
                    debug!("check_phase_correction says: Tempo stable for {:.1}s, waiting for 10s before enabling phase correction", 
                           now.duration_since(stable_since).as_secs_f32());
                    return Ok(());
                }
            }
        } else {
            // No external tempo available, can't do phase correction
            return Ok(());
        }

        // Get MIDI clock tick count
        let midi_clock_ticks = {
            let clock_state = self.midi.get_clock_state();
            clock_state.map(|state| state.clock_ticks).unwrap_or(0)
        };

        // Calculate expected sequencer position based on MIDI clock
        let midi_beats = midi_clock_ticks / 24; // 24 ticks per beat
        let expected_sequencer_step = (midi_beats % 32) as usize; // 32 steps per pattern

        // Get current sequencer position
        let (current_step, _) = self.sequencer.get_current_position();

        // Calculate drift in ticks (convert steps back to ticks for precision)
        let current_step_ticks = (current_step * 24) as i32; // 24 MIDI ticks per step
        let expected_step_ticks = (expected_sequencer_step * 24) as i32;
        let raw_drift = current_step_ticks - expected_step_ticks;
        
        // Handle wraparound (sequence boundary)
        let drift_ticks = if raw_drift > 384 { // 384 = 16 steps * 24 ticks
            raw_drift - 768 // 768 = 32 steps * 24 ticks (full sequence)
        } else if raw_drift < -384 {
            raw_drift + 768
        } else {
            raw_drift
        };

        self.current_drift_ticks = drift_ticks;

        // Apply correction if drift is significant
        if drift_ticks.abs() > 24 { // More than 1 step off
            info!("check_phase_correction says: Large drift detected: {} ticks, applying correction", drift_ticks);
            // For large drift, do a hard reset to the expected position
            // This would require extending the sequencer interface
            debug!("check_phase_correction says: Would reset sequencer to step {}", expected_sequencer_step);
        } else if drift_ticks.abs() > 6 { // More than 1/4 step off
            debug!("check_phase_correction says: Small drift detected: {} ticks, gentle correction needed", drift_ticks);
            // For small drift, apply gentle correction by slightly adjusting timing
            // This would require extending the sequencer interface for micro-timing adjustments
        }

        self.last_midi_clock_count = midi_clock_ticks;
        self.last_sync_check = Instant::now();

        Ok(())
    }

    /// Reset phase correction state
    fn reset_phase_correction(&mut self) {
        self.last_midi_clock_count = 0;
        self.last_sync_check = Instant::now();
        self.current_drift_ticks = 0;
        self.tempo_stable_since = None;
        self.last_stable_tempo = None;
        debug!("reset_phase_correction says: Phase correction state reset");
    }

    /// Calculate LED brightness based on drift amount
    fn calculate_drift_brightness(&self) -> u8 {
        let abs_drift = self.current_drift_ticks.abs();
        match abs_drift {
            0 => 0,            // Perfect sync: off
            1..=12 => 3,       // 1 tick to 1/2 step: very dim
            13..=24 => 6,      // 1/2 to 1 step: dim
            25..=48 => 10,     // 1 to 2 steps: medium
            _ => 15,           // > 2 steps: bright warning
        }
    }
}

fn main() -> Result<()> {
    // Set up direct signal handler that bypasses hardware thread
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();

    match ctrlc::set_handler(move || {
        info!("🛑 Direct Ctrl+C handler triggered - forcing exit");
        shutdown_flag_clone.store(true, Ordering::SeqCst);
        thread::spawn(|| {
            thread::sleep(Duration::from_millis(100));
            warn!("🛑 Direct force exit");
            std::process::exit(0);
        });
    }) {
        Ok(()) => {
            info!("Ctrl+C handler successfully registered");
        }
        Err(ctrlc::Error::MultipleHandlers) => {
            warn!("Ctrl+C handler already exists, skipping registration");
        }
        Err(e) => {
            error!("Failed to set Ctrl+C handler: {}", e);
            return Err(anyhow::anyhow!("Failed to set Ctrl+C handler: {}", e));
        }
    }

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Parse other arguments if needed (currently none)

    // Handle help flag
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        println!("SimonSaysSeeq Rust v{}", env!("CARGO_PKG_VERSION"));
        println!("A high-performance sequencer for Norns hardware\n");
        println!("USAGE:");
        println!("    simon_says_seeq [OPTIONS]\n");
        println!("OPTIONS:");
        println!("    -h, --help           Print help information");
        println!("    -v, --version        Print version information");
        println!("    --config <FILE>      Use custom configuration file");
        println!("    --no-hardware        Disable hardware features");
        println!("    --no-midi            Disable MIDI features");


        return Ok(());
    }

    // Handle version flag
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("SimonSaysSeeq Rust v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }



    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Also log version info to ai_startup.log file
    if let Ok(mut ai_log) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("ai_startup.log") 
    {
        use std::io::Write;
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let version = env!("CARGO_PKG_VERSION");
        let profile = if cfg!(debug_assertions) { "debug" } else { "release" };
        
        writeln!(ai_log, "\n## {} - SimonSaysSeeq Rust v{} Startup", timestamp, version).ok();
        writeln!(ai_log, "Build Profile: {}", profile).ok();
        writeln!(ai_log, "Features: MIDI={}, Hardware={}", cfg!(feature = "midi"), cfg!(feature = "hardware")).ok();
    }

    // Log startup banner with version and timestamp
    info!("════════════════════════════════════════════════════════");
    info!("SimonSaysSeeq Rust v{} Starting Up", env!("CARGO_PKG_VERSION"));
    info!("Startup Time: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    info!("Build Profile: {}", if cfg!(debug_assertions) { "debug" } else { "release" });
    info!("Features: MIDI={}, Hardware={}", cfg!(feature = "midi"), cfg!(feature = "hardware"));
    info!("════════════════════════════════════════════════════════");

    // Create and run application
    let mut app = SimonSaysSeeq::new()?;

    // Single-grid mode removed - application now requires exactly 2 grids

    app.run()?;

    Ok(())
}
