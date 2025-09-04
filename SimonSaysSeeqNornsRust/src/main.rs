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

mod hardware;
mod sequencer;
mod midi;
mod midi_scanner;

mod screen;
mod config;
mod co2;

/// ARM actions that can be triggered from row 7 (control row) of the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArmAction {
    Undo,              // Column 0
    Redo,              // Column 1
    EuclidianEvents,   // Column 4
    EuclidianLength,   // Column 5
    EuclidianRotation, // Column 6
    Ratchet,           // Column 7
    PresetGrid,        // Column 10
    Mozart,            // Column 15
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
            10 => Some(ArmAction::PresetGrid),
            15 => Some(ArmAction::Mozart),
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
            ArmAction::PresetGrid => 10,
            ArmAction::Mozart => 15,
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
    main_grid_preference: Option<String>,
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
            main_grid_preference: None, // GRID_ONE will be auto-selected as lowest ID
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

        // Flash all connected grids for visual feedback
        if let Err(e) = self.grid.flash_all_grids() {
            // warn!("Failed to flash grids on sequencer start: {}", e);
        }

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
                            // info!("Stop pressed");
                            self.sequencer.stop();
                            #[cfg(feature = "midi")]
                            self.midi.all_notes_off()?;
                        }
                        3 => {
                            // Right key - Start/Stop toggle
                            if self.sequencer.is_running() {
                                // info!("Stop pressed");
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
                    // info!("Stop pressed");
                    self.sequencer.stop();
                    #[cfg(feature = "midi")]
                    self.midi.all_notes_off()?;
                } else {
                    // info!("Start pressed");
                    self.sequencer.start();

                    // Flash all connected grids for visual feedback
                    if let Err(e) = self.grid.flash_all_grids() {
                        // warn!("Failed to flash grids on sequencer start: {}", e);
                    }
                }
            }

            HardwareEvent::GridPress { grid_id, x, y, pressed } => {
                self.handle_grid_press(&grid_id, x, y, pressed)?;
            }

            HardwareEvent::Shutdown => {
                info!("🛑 Shutdown requested - initiating immediate exit");
                self.running.store(false, Ordering::SeqCst);
                // More aggressive force exit
                thread::spawn(|| {
                    thread::sleep(Duration::from_millis(500));
                    warn!("🛑 Forcing immediate exit");
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
                                    self.midi.note_on(note_event.note, note_event.velocity, note_event.channel)?;
                                } else {
                                    self.midi.note_off(note_event.note, note_event.channel)?;
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
                // Handle MIDI events from sequencer
                #[cfg(feature = "midi")]
                {
                    if midi_event.note_on {
                        self.midi.note_on(midi_event.note, midi_event.velocity, midi_event.channel)?;
                        // info!("MIDI Note ON: {} vel:{} ch:{} step:{}",
                        //       midi_event.note, midi_event.velocity, midi_event.channel, midi_event.step);
                    } else {
                        self.midi.note_off(midi_event.note, midi_event.channel)?;
                        // info!("MIDI Note OFF: {} ch:{} step:{}",
                        //       midi_event.note, midi_event.channel, midi_event.step);
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
        // Check if Mozart mode is active
        if let Some(arm_action) = self.active_arm_action {
            if matches!(arm_action, ArmAction::Mozart) {
                // Mozart mode: Handle MIDI note input on both grids
                if pressed && y <= 7 {
                    self.handle_mozart_grid_press(x, y)?;
                }
                return Ok(());
            }
        }

        // Normal mode: Handle 32-step sequence input
        let connected_grids = self.grid.get_connected_grids();
        let main_grid = self.get_main_grid_id(&connected_grids);
        
        // Calculate actual sequence step (0-31)
        let seq_x = if connected_grids.len() >= 2 {
            let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
            if Some(grid_id) == grid_one.as_ref().map(|x| x.as_str()) {
                // GRID_ONE (lowest ID): steps 0-15
                x
            } else if Some(grid_id) == grid_two.as_ref().map(|x| x.as_str()) {
                // GRID_TWO (second lowest ID): steps 16-31 (map from grid coordinates 0-15)
                x + 16
            } else {
                x // Fallback
            }
        } else {
            x // Single grid fallback
        };
        let seq_y = y;
        
        info!("GRID DEBUG: Grid press on {} at grid({},{}) -> seq({},{}) pressed={}", 
              grid_id, x, y, seq_x, seq_y, pressed);
        
        // Debug grid ID mapping
        let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
        info!("GRID DEBUG: Available grids - GRID_ONE: {:?}, GRID_TWO: {:?}", grid_one, grid_two);
        info!("GRID DEBUG: This press came from: {}", grid_id);

        // Handle sequence rows and ARM controls
        if (connected_grids.len() >= 2) || main_grid.as_ref().map(|id| id == grid_id).unwrap_or(false) {
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
                                let current_length = row_state.euclidean_length + 1; // Convert from 0-based to step count
                                let current_rotation = row_state.euclidean_rotation;
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
                                let current_events = row_state.euclidean_events;
                                let current_rotation = row_state.euclidean_rotation;
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
                                let current_events = row_state.euclidean_events;
                                let current_length = row_state.euclidean_length + 1; // Convert from 0-based to step count
                                self.sequencer.generate_euclidean_rhythm(seq_y, current_events, current_length, rotation);
                            } else {
                                // Fallback if row_state is not available
                                warn!("EuclidianRotation: Could not get row_state for row {}, using fallback defaults", seq_y);
                                self.sequencer.generate_euclidean_rhythm(seq_y, 5, 32, rotation);
                            }
                            
                            info!("ARM EUCLIDIAN_ROTATION: Successfully generated rotation {} on row {}", rotation, seq_y);
                            self.refresh_all_row_leds(seq_y)?;
                        },
                        _ => {
                            // No active Euclidean ARM action - handle normal grid operation
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
                        // GRID_TWO tempo controls (columns 10, 12-15)
                        else if x == 10 || (x >= 12 && x <= 15) {
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
                                        if self.snap_to_whole_tempo { 8 } else { 0 }
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
                
                // Regular ARM control handling for main grid or other buttons
                if connected_grids.is_empty() || Some(grid_id) == self.get_main_grid_id(&connected_grids).as_ref().map(|x| x.as_str()) {
                    info!("ARM CONTROL: Row 7 button {} {}", seq_x, if pressed { "PRESSED" } else { "RELEASED" });

                    // Check if this column corresponds to a valid ARM action
                    if let Some(arm_action) = ArmAction::from_column(seq_x) {
                    if pressed {
                        // Turn off previous ARM button if any
                        if let Some(prev_action) = self.active_arm_action {
                            let prev_column = prev_action.to_column();
                            info!("ARM CONTROL: Deactivating previous ARM action {:?} (column {})", prev_action, prev_column);
                            self.grid.set_led(grid_id, prev_column, seq_y, 0, "arm_action_deactivate")?;
                        }

                        // Set new active ARM action and light it up
                        self.active_arm_action = Some(arm_action);
                        info!("ARM CONTROL: Activated ARM action {:?} (column {}) - waiting for sequence row press", arm_action, seq_x);
                        self.grid.set_led(grid_id, seq_x, seq_y, 10, "arm_action_press")?;
                        self.grid.refresh()?;

                        // Handle ARM action function
                        self.handle_arm_action(arm_action)?;
                    } else {
                        // Clear active ARM action and turn off LED
                        info!("ARM CONTROL: Release detected for column {}, current active: {:?}", seq_x, self.active_arm_action);
                        if let Some(current_action) = self.active_arm_action {
                            if current_action.to_column() == seq_x {
                                self.active_arm_action = None;
                                info!("ARM CONTROL: Deactivated ARM action {:?} - returned to normal mode", current_action);
                                #[cfg(feature = "hardware")]
                                {
                                    self.grid.set_led(grid_id, seq_x, seq_y, 0, "arm_action_release")?;
                                    self.grid.refresh()?;
                                }
                            } else {
                                info!("ARM CONTROL: Release ignored - column {} is not the active ARM action", seq_x);
                            }
                        }
                    }
                    } else {
                        // info!("ARM CONTROL: ROW 7 column {} is not a valid ARM action", seq_x);
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
        // Handle ARM actions based on enum
        match action {
            ArmAction::Undo => {
                // Undo last action
                match self.sequencer.undo() {
                    Ok(description) => info!("ARM UNDO: Successfully undid: {}", description),
                    Err(e) => warn!("ARM UNDO: Cannot undo: {}", e),
                }
                #[cfg(feature = "hardware")]
                {
                    // Refresh all pattern LEDs after undo - pattern state may have changed
                    self.refresh_all_pattern_leds()?;
                }
            }
            ArmAction::Redo => {
                // Redo last undone action
                match self.sequencer.redo() {
                    Ok(description) => info!("ARM REDO: Successfully redid: {}", description),
                    Err(e) => warn!("ARM REDO: Cannot redo: {}", e),
                }
                #[cfg(feature = "hardware")]
                {
                    // Refresh all pattern LEDs after redo - pattern state may have changed
                    self.refresh_all_pattern_leds()?;
                }
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
            ArmAction::PresetGrid => {
                // Preset grid functionality - placeholder
                info!("ARM PRESET_GRID: ARM button activated - not yet implemented");
            }
            ArmAction::Mozart => {
                // Mozart mode - show keyboard MIDI notes on both grids
                info!("ARM MOZART: ARM button activated - showing 32-step keyboard MIDI sequence");
                #[cfg(feature = "hardware")]
                {
                    self.update_mozart_display()?;
                }
            }
        }
        Ok(())
    }



    #[cfg(feature = "hardware")]
    fn handle_mozart_grid_press(&mut self, x: usize, y: usize) -> Result<()> {
        // Grid two - Mozart interface for MIDI note control
        info!("Mozart grid press at ({}, {}) - MIDI note control", x, y);

        // Convert grid position to MIDI note value
        let base_note = 60; // Middle C
        let note = base_note + (7 - y) * 5 + x; // Pentatonic-ish mapping

        if note <= 127 {
            // Update Mozart state
            self.sequencer.set_mozart_value(x + 1, y + 1, note as u8);

            // Send test note
            #[cfg(feature = "midi")]
            self.midi.note_on(note as u8, 100, 1)?;
            #[cfg(not(feature = "midi"))]
            info!("Simulated MIDI note: {} vel:100 ch:1", note);

            // Update LED to show note value (brightness = note % 16)
            let brightness = ((note % 15) + 1) as u8;
            let connected_grids = self.grid.get_connected_grids();
            // Mozart LED updates disabled for debugging
            // if let Some(grid_id) = connected_grids.get(1).or_else(|| connected_grids.first()) {
            //     self.grid.set_led(grid_id, x, y, brightness, "handle_mozart_grid_press")?;
            // }

            info!("Set Mozart[{}][{}] = note {}", x + 1, y + 1, note);
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
    fn handle_grid_update(&mut self, row: usize, old_step: usize, new_step: usize) -> Result<()> {
        let connected_grids = self.grid.get_connected_grids();
        
        if connected_grids.len() >= 2 {
            // 32-step mode: Update the correct grid based on step position
            let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
            let grid_one_id = grid_one.as_ref().unwrap();
            let grid_two_id = grid_two.as_ref().unwrap();
            
            if let Some(row_state) = self.sequencer.get_row_states(row) {
                // Update old position LED
                let old_pattern_value = self.sequencer.get_grid_value(old_step, row);
                let old_brightness = if old_pattern_value > 0 { 10 } else { 0 };
                
                if old_step <= 15 {
                    self.grid.set_led(grid_one_id, old_step, row, old_brightness, "grid_update_old_1")?;
                } else if old_step <= 31 {
                    let grid_x = old_step - 16;
                    self.grid.set_led(grid_two_id, grid_x, row, old_brightness, "grid_update_old_2")?;
                }
                
                // Update new position LED
                let new_pattern_value = self.sequencer.get_grid_value(new_step, row);
                let new_brightness = if new_pattern_value > 0 { 14 } else { 6 };
                
                if new_step <= 15 {
                    self.grid.set_led(grid_one_id, new_step, row, new_brightness, "grid_update_new_1")?;
                } else if new_step <= 31 {
                    let grid_x = new_step - 16;
                    self.grid.set_led(grid_two_id, grid_x, row, new_brightness, "grid_update_new_2")?;
                }
            }
        } else if let Some(main_grid_id) = self.get_main_grid_id(&connected_grids) {
            // Single grid fallback
            if let Some(row_state) = self.sequencer.get_row_states(row) {
                let old_pattern_value = self.sequencer.get_grid_value(old_step, row);
                let old_brightness = if old_pattern_value > 0 { 10 } else { 0 };
                self.grid.set_led(&main_grid_id, old_step, row, old_brightness, "grid_update_old")?;

                let new_pattern_value = self.sequencer.get_grid_value(new_step, row);
                let new_brightness = if new_pattern_value > 0 { 14 } else { 6 };
                self.grid.set_led(&main_grid_id, new_step, row, new_brightness, "grid_update_new")?;
            }
        }
        
        // Update beat LEDs before final refresh
        self.update_beat_leds()?;
        
        self.grid.refresh()?;
        Ok(())
    }

    /// Full grid display update (only used for initialization)
    #[cfg(feature = "hardware")]
    fn update_grid_display(&mut self) -> Result<()> {
        let connected_grids = self.grid.get_connected_grids();
        info!("DEBUG: update_grid_display called - found {} connected grids: {:?}", connected_grids.len(), connected_grids);

        // Check if Mozart mode is active
        if let Some(arm_action) = self.active_arm_action {
            if matches!(arm_action, ArmAction::Mozart) {
                // Mozart mode: Show keyboard MIDI notes on both grids
                info!("DEBUG: Mozart mode active - showing keyboard MIDI notes");
                self.update_mozart_display()?;
                return Ok(());
            }
        }

        // Normal mode: Show 32-step sequence across both grids
        if connected_grids.len() >= 2 {
            // Use sorted grid IDs for consistency
            let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
            let grid_one_id = grid_one.as_ref().unwrap();
            let grid_two_id = grid_two.as_ref().unwrap();
            
            info!("GRID DEBUG: update_grid_display() using 32-step mode - GRID_ONE: {}, GRID_TWO: {}", grid_one_id, grid_two_id);
            self.update_32step_display(grid_one_id, grid_two_id)?;
        } else if let Some(main_grid_id) = self.get_main_grid_id(&connected_grids) {
            // Fallback: Single grid showing 16 steps
            info!("DEBUG: Using single grid fallback mode - grid: {}", main_grid_id);
            self.update_main_grid_display(&main_grid_id)?;
        } else {
            info!("DEBUG: No grids available for display");
        }

        // Update beat LEDs before final refresh
        self.update_beat_leds()?;
        
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
                    let is_current_step = seq_x == row_state.current_step;

                    // Calculate brightness based on pattern and current position
                    let brightness = match (pattern_value > 0, is_current_step) {
                        (false, false) => 0,     // No pattern, not current position
                        (false, true) => 6,      // No pattern, but current position  
                        (true, false) => 10,     // Has pattern, not current position
                        (true, true) => 14,      // Has pattern AND current position
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

    #[cfg(feature = "hardware")]
    fn update_32step_display(&mut self, grid_one: &str, grid_two: &str) -> Result<()> {
        // 32-step mode: Current step flows between grids
        // Steps 0-15: Show on GRID_ONE, Steps 16-31: Show on GRID_TWO
        info!("GRID DEBUG: update_32step_display called - grid_one: {}, grid_two: {}", grid_one, grid_two);
        
        for seq_y in 0..=6 {
            let row_states = self.sequencer.get_row_states(seq_y);
            if let Some(row_state) = row_states {
                let current_step = row_state.current_step;
                let euclidean_length = row_state.euclidean_length;
                
                if seq_y == 0 { // Only log for first row to avoid spam
                    info!("DEBUG: Row {} - current_step: {}, euclidean_length: {}", seq_y, current_step, euclidean_length);
                }
                
                // Debug: Check for patterns in steps 16-31
                let mut patterns_16_31 = Vec::new();
                for step in 16..=31 {
                    let pattern = self.sequencer.get_grid_value(step, seq_y);
                    if pattern > 0 {
                        patterns_16_31.push((step, pattern));
                    }
                }
                if seq_y == 0 && !patterns_16_31.is_empty() {
                    info!("DEBUG: Row {} patterns in steps 16-31: {:?}", seq_y, patterns_16_31);
                }
                
                // Clear both grids for this row first
                for grid_x in 0..=15 {
                    self.grid.set_led(grid_one, grid_x, seq_y, 0, "clear_grid1")?;
                    self.grid.set_led(grid_two, grid_x, seq_y, 0, "clear_grid2")?;
                }
                
                // Show all patterns on both grids
                for seq_x in 0..=31 {
                    let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                    if pattern_value > 0 {
                        let brightness = 10; // Pattern exists but not current
                        
                        if seq_x <= 15 {
                            // Show pattern on GRID_ONE (steps 0-15)
                            if seq_y == 0 { // Only log for first row to avoid spam
                                info!("DEBUG: Setting GRID_ONE pattern LED - step: {}, grid_x: {}, brightness: {}, pattern_value: {}", seq_x, seq_x, brightness, pattern_value);
                            }
                            self.grid.set_led(grid_one, seq_x, seq_y, brightness, "pattern_grid1")?;
                        } else {
                            // Show pattern on GRID_TWO (steps 16-31, mapped to 0-15)
                            let grid_x = seq_x - 16;
                            if seq_y == 0 { // Only log for first row to avoid spam
                                info!("DEBUG: Setting GRID_TWO pattern LED - step: {}, grid_x: {}, brightness: {}, pattern_value: {}", seq_x, grid_x, brightness, pattern_value);
                            }
                            self.grid.set_led(grid_two, grid_x, seq_y, brightness, "pattern_grid2")?;
                        }
                    }
                }
                
                // Show current step position with bright LED
                if current_step <= 15 {
                    // Current step is on GRID_ONE
                    let pattern_value = self.sequencer.get_grid_value(current_step, seq_y);
                    let brightness = if pattern_value > 0 { 14 } else { 6 }; // Bright if pattern, medium if just position
                    self.grid.set_led(grid_one, current_step, seq_y, brightness, "current_step_grid1")?;
                } else if current_step <= 31 {
                    // Current step is on GRID_TWO
                    let grid_x = current_step - 16;
                    let pattern_value = self.sequencer.get_grid_value(current_step, seq_y);
                    let brightness = if pattern_value > 0 { 14 } else { 6 }; // Bright if pattern, medium if just position
                    
                    if seq_y == 0 { // Only log for first row
                        info!("DEBUG: Setting GRID_TWO LED - step: {}, grid_x: {}, brightness: {}, pattern_value: {}", current_step, grid_x, brightness, pattern_value);
                    }
                    
                    self.grid.set_led(grid_two, grid_x, seq_y, brightness, "current_step_grid2")?;
                } else {
                    if seq_y == 0 {
                        info!("DEBUG: current_step {} is outside valid range (0-31)", current_step);
                    }
                }
            } else {
                if seq_y == 0 {
                    info!("DEBUG: No row state found for row {}", seq_y);
                }
            }
        }
        
        // Update beat LEDs before final refresh
        self.update_beat_leds()?;
        
        // Add a final debug to confirm grid refresh
        info!("DEBUG: Calling grid.refresh() for both grids");
        self.grid.refresh()?;
        
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
                let is_current_step = seq_x == row_state.current_step;

                let brightness = match (pattern_value > 0, is_current_step) {
                    (false, false) => 0,
                    (false, true) => 6,
                    (true, false) => 10,
                    (true, true) => 14,
                };

                // Map sequence coordinates to correct grid
                if seq_x <= 15 {
                    self.grid.set_led(grid_one_id, seq_x, seq_y, brightness, "update_single_led_grid1")?;
                } else if seq_x <= 31 {
                    let grid_x = seq_x - 16;
                    self.grid.set_led(grid_two_id, grid_x, seq_y, brightness, "update_single_led_grid2")?;
                }
            }
        } else if let Some(main_grid_id) = self.get_main_grid_id(&connected_grids) {
            // Single grid fallback - only handle steps 0-15
            if seq_x <= 15 {
                if let Some(row_state) = self.sequencer.get_row_states(seq_y) {
                    let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                    let is_current_step = seq_x == row_state.current_step;

                    let brightness = match (pattern_value > 0, is_current_step) {
                        (false, false) => 0,
                        (false, true) => 6,
                        (true, false) => 10,
                        (true, true) => 14,
                    };

                    self.grid.set_led(&main_grid_id, seq_x, seq_y, brightness, "update_single_led_single")?;
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
                let is_current_step = seq_x == row_state.current_step;
                
                let brightness = match (pattern_value > 0, is_current_step) {
                    (false, false) => 0,     // No pattern, not current position
                    (false, true) => 6,      // No pattern, but current position
                    (true, false) => 10,     // Has pattern, not current position
                    (true, true) => 14,      // Has pattern AND current position
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
                
                // Update beat LEDs before refresh
                self.update_beat_leds()?;
                
                self.grid.refresh()?;
            }
        } else if let Some(main_grid_id) = self.get_main_grid_id(&connected_grids) {
            // Single grid fallback
            if seq_x <= 15 {
                if let Some(row_state) = self.sequencer.get_row_states(seq_y) {
                    let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                    let is_current_step = seq_x == row_state.current_step;
                    
                    let brightness = match (pattern_value > 0, is_current_step) {
                        (false, false) => 0,
                        (false, true) => 6,
                        (true, false) => 10,
                        (true, true) => 14,
                    };
                    
                    info!("GRID DEBUG: Setting single LED (single grid) seq_x={}, seq_y={}, brightness={}", seq_x, seq_y, brightness);
                    self.grid.set_led(&main_grid_id, seq_x, seq_y, brightness, "single_button_single")?;
                    
                    // Update beat LEDs before refresh
                    self.update_beat_leds()?;
                    
                    self.grid.refresh()?;
                }
            }
        }
        
        Ok(())
    }



    /// Set GRID_ONE preference (optional - defaults to lowest ID)
    pub fn set_main_grid_preference(&mut self, grid_id: String) {
        info!("Setting GRID_ONE preference to: {}", grid_id);
        self.main_grid_preference = Some(grid_id);
    }

    /// Get sorted grid IDs - returns (GRID_ONE, GRID_TWO)
    fn get_sorted_grid_ids(&self, connected_grids: &[String]) -> (Option<String>, Option<String>) {
        // info!("ARM DEBUG: get_sorted_grid_ids called with {} grids: {:?}", connected_grids.len(), connected_grids);
        
        // Always sort grids by ID for consistency
        let mut sorted_grids = connected_grids.to_vec();
        sorted_grids.sort();
        
        let grid_one = sorted_grids.first().cloned();
        let grid_two = if sorted_grids.len() > 1 { sorted_grids.get(1).cloned() } else { None };
        
        // info!("ARM DEBUG: GRID_ONE (lowest ID): {:?}", grid_one);
        // info!("ARM DEBUG: GRID_TWO (second lowest ID): {:?}", grid_two);
        
        (grid_one, grid_two)
    }

    /// Get GRID_ONE ID (lowest ID) - for compatibility
    fn get_main_grid_id(&self, connected_grids: &[String]) -> Option<String> {
        self.get_sorted_grid_ids(connected_grids).0
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
                        let is_current_step = seq_x == row_state.current_step;
                        
                        let brightness = match (pattern_value > 0, is_current_step) {
                            (false, false) => 0,     // No pattern, not current position
                            (false, true) => 6,      // No pattern, but current position  
                            (true, false) => 10,     // Has pattern, not current position
                            (true, true) => 14,      // Has pattern AND current position
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
        } else if let Some(main_grid_id) = self.get_main_grid_id(&connected_grids) {
            info!("ARM DEBUG: Using single-grid fallback mode - grid: {}", main_grid_id);
            // Single grid fallback - only show first 16 steps
            for seq_y in 0..=6 {
                if let Some(row_state) = self.sequencer.get_row_states(seq_y) {
                    for seq_x in 0..=15 {
                        let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                        let is_current_step = seq_x == row_state.current_step;
                        
                        let brightness = match (pattern_value > 0, is_current_step) {
                            (false, false) => 0,
                            (false, true) => 6,
                            (true, false) => 10,
                            (true, true) => 14,
                        };
                        
                        self.grid.set_led(&main_grid_id, seq_x, seq_y, brightness, "refresh_pattern_single")?;
                    }
                }
            }
        } else {
            info!("ARM DEBUG: No grids found!");
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
                info!("ARM DEBUG: Got row state for row {}: current_step={}", row, row_state.current_step);
                
                for seq_x in 0..=31 {
                    let pattern_value = self.sequencer.get_grid_value(seq_x, row);
                    let is_current_step = seq_x == row_state.current_step;
                    
                    let brightness = match (pattern_value > 0, is_current_step) {
                        (false, false) => 0,
                        (false, true) => 6,
                        (true, false) => 10,
                        (true, true) => 14,
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
        } else if let Some(main_grid_id) = self.get_main_grid_id(&connected_grids) {
            // Single grid fallback
            info!("ARM DEBUG: refresh_all_row_leds() called for row {} on single grid", row);
            
            if let Some(row_state) = self.sequencer.get_row_states(row) {
                for seq_x in 0..=15 {
                    let pattern_value = self.sequencer.get_grid_value(seq_x, row);
                    let is_current_step = seq_x == row_state.current_step;
                    
                    let brightness = match (pattern_value > 0, is_current_step) {
                        (false, false) => 0,
                        (false, true) => 6,
                        (true, false) => 10,
                        (true, true) => 14,
                    };
                    
                    self.grid.set_led(&main_grid_id, seq_x, row, brightness, "refresh_all_row_leds")?;
                }
                self.grid.refresh()?;
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



    /// Handle CO2-influenced CV output
    fn update_mozart_display(&mut self) -> Result<()> {
        let connected_grids = self.grid.get_connected_grids();
        
        // Display keyboard MIDI note events on both grids
        if connected_grids.len() >= 1 {
            let (grid_one, _) = self.get_sorted_grid_ids(&connected_grids);
            let grid_one_id = grid_one.as_ref().unwrap();
            self.update_keyboard_midi_display(grid_one_id, 0)?; // Show first 16 steps
        }
        if connected_grids.len() >= 2 {
            let (_, grid_two) = self.get_sorted_grid_ids(&connected_grids);
            let grid_two_id = grid_two.as_ref().unwrap();
            self.update_keyboard_midi_display(grid_two_id, 16)?; // Show steps 16-31
        }
        Ok(())
    }

    fn update_keyboard_midi_display(&mut self, grid_id: &str, step_offset: usize) -> Result<()> {
        // Display keyboard MIDI note events for this grid
        // Clear grid first
        for x in 0..16 {
            for y in 0..8 {
                self.grid.set_led(grid_id, x, y, 0, "clear_keyboard_midi")?;
            }
        }
        
        // Get keyboard MIDI note events from sequencer
        let keyboard_events = self.sequencer.get_keyboard_midi_events();
        
        // Display MIDI events - using current lane (0) and bar (0) for now
        let lane = 0;  // First lane
        let bar = 0;   // First bar
        
        if lane < keyboard_events.len() && bar < keyboard_events[lane].len() {
            for x in 0..16 {
                let step = step_offset + x;
                if step < keyboard_events[lane][bar].len() {
                    // Check for active MIDI notes at this step
                    for note in 36..96 { // Show notes C2 to C6
                        if note < keyboard_events[lane][bar][step].len() {
                            let note_on_event = &keyboard_events[lane][bar][step][note][1]; // Note ON events
                            if note_on_event.is_active {
                                // Map MIDI note to grid Y position (notes 36-96 -> rows 0-7)
                                let grid_y = ((note - 36) / 8).min(7);
                                let brightness = (note_on_event.velocity / 8).max(1).min(15) as u8;
                                self.grid.set_led(grid_id, x, grid_y, brightness, "keyboard_midi_note")?;
                            }
                        }
                    }
                }
            }
        }
        
        // Update beat LEDs before refresh
        self.update_beat_leds()?;
        
        self.grid.refresh()?;
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
                // Flash LEDs 14 and 15 on GRID_TWO for 150ms
                self.beat_led_flash_until = Some(Instant::now() + Duration::from_millis(150));
                
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
                info!("handle_midi_input_event says: MIDI Clock Stop received - stopping sequencer");
                self.sequencer.stop();
                // Clear beat LEDs when clock stops
                self.beat_led_flash_until = None;
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
            MidiInputEvent::NoteOn { .. } => {
                // Note On events - currently not handled in main app
                // Could be used for MIDI input recording in future
            }
            MidiInputEvent::NoteOff { .. } => {
                // Note Off events - currently not handled in main app
                // Could be used for MIDI input recording in future
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
        let connected_grids = self.grid.get_connected_grids();
        if connected_grids.len() >= 2 {
            let (_, grid_two) = self.get_sorted_grid_ids(&connected_grids);
            if let Some(grid_two_id) = grid_two {
                let brightness = if let Some(flash_until) = self.beat_led_flash_until {
                    if Instant::now() < flash_until {
                        8 // Dimmer flash
                    } else {
                        self.beat_led_flash_until = None;
                        0  // Turn off
                    }
                } else {
                    0  // Off by default
                };
                
                #[cfg(feature = "hardware")]
                {
                    self.grid.set_led(&grid_two_id, 14, 7, brightness, "beat_led_flash")?;
                    self.grid.set_led(&grid_two_id, 15, 7, brightness, "beat_led_flash")?;
                    
                    // Update snap tempo button LED (column 10, row 7)
                    let snap_brightness = if self.snap_to_whole_tempo { 8 } else { 0 };
                    self.grid.set_led(&grid_two_id, 10, 7, snap_brightness, "snap_tempo_button")?;
                    
                    // Update drift indicator LED (column 11, row 7)
                    let drift_brightness = self.calculate_drift_brightness();
                    self.grid.set_led(&grid_two_id, 11, 7, drift_brightness, "drift_indicator")?;
                }
            }
        }
        
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
    let mut main_grid_id: Option<String> = None;

    // Parse grid selection argument
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--main-grid" => {
                if i + 1 < args.len() {
                    main_grid_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --main-grid requires a grid ID argument");
                    std::process::exit(1);
                }
            }
            _ => i += 1,
        }
    }

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
        println!("    --main-grid <ID>     Specify which grid to use as main sequencer (default: m2949672)");

        return Ok(());
    }

    // Handle version flag
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("SimonSaysSeeq Rust v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }



    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("SimonSaysSeeq Rust v{} (build-with-ctrlc-fix-2025-01-02)", env!("CARGO_PKG_VERSION"));
    info!("Build info: Ctrl+C handler fix applied, startup output capture enabled");

    // Create and run application
    let mut app = SimonSaysSeeq::new()?;

    // Set GRID_ONE preference if specified
    if let Some(grid_id) = main_grid_id {
        app.set_main_grid_preference(grid_id);
    }

    app.run()?;

    Ok(())
}
