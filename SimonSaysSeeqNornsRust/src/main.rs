//! SimonSaysSeeq Pure Rust Implementation for Norns
//!
//! A high-performance sequencer application that runs directly on Norns hardware
//! without requiring the Norns Lua environment.

use anyhow::Result;
use log::{info, warn, debug, error, trace};
use anyhow::anyhow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use chrono;
use crossbeam_channel::Sender;

use simon_says_seeq_rust::formal_state_logger::{self, log_button_press, log_button_release, log_midi_note_on, log_midi_note_off, log_arm_action_activated, log_arm_action_executed, ButtonSource};

mod hardware;
mod sequencer;
mod midi;
mod midi_scanner;

mod screen;
mod config;
mod co2;
mod version;

// LED brightness constants
const LED_OFF: u8 = 0;      // Empty step, no playhead (LED off)
const LED_DIM: u8 = 6;      // Empty step, playhead present (position only)
const LED_DIM_PLUS: u8 = 8; // Empty step, playhead present (enhanced visibility)
const LED_BRIGHT: u8 = 10;  // Pattern exists, no playhead (pattern only)
const LED_MAX: u8 = 14;     // Pattern exists, playhead present (pattern + position)
const LED_TURBO: u8 = 15;     // Maximum brightness (ARM buttons, flashing, etc.)

/// External MIDI clock tick counter for step synchronization
static EXTERNAL_CLOCK_TICK_COUNTER: AtomicU32 = AtomicU32::new(0);

/// ARM actions that can be triggered from row 7 (control row) of the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArmAction {
    Undo,              // Column 0
    Redo,              // Column 1
    EuclidianEvents,   // Column 4
    EuclidianLength,   // Column 5
    EuclidianRotation, // Column 6
    Ratchet,           // Column 7
    SetMaxStepForRow,  // Column 8
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
            8 => Some(ArmAction::SetMaxStepForRow),
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
            ArmAction::SetMaxStepForRow => 8,
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
    co2: Option<Co2Manager>,
    crow: simon_says_seeq_rust::crow::Crow,
    config: Config,
    running: Arc<AtomicBool>,
    // Active ARM action for row 7 (control row) - only one can be active at a time
    active_arm_action: Option<ArmAction>,
    // Beat LED flashing state for external MIDI clock
    beat_led_flash_until: Option<Instant>,
    // Crow CV1 mute state (true when MUTE_CROW_1 button is latched ON)
    crow_cv1_muted: bool,
    // Crow CV2 mute state (true when MUTE_CROW_2 button is latched ON)
    crow_cv2_muted: bool,
    // Crow CV3 mute state (true when MUTE_CROW_3 button is latched ON)
    crow_cv3_muted: bool,
    // Crow CV4 mute state (true when MUTE_CROW_4 button is latched ON)
    crow_cv4_muted: bool,
    // GRID_TWO button state tracking for MIDI detection
    grid_two_button_0_pressed: bool,
    grid_two_button_1_pressed: bool,
    // Test mode flag (disables auto-save, auto-loads test_pattern_1.json)
    test_mode: bool,
}

impl SimonSaysSeeq {
    pub fn new() -> Result<Self> {
        Self::new_with_test_mode(false)
    }

    pub fn new_with_test_mode(test_mode: bool) -> Result<Self> {
        let config_path = Config::get_config_path();
        // info!("📁 Config file location: {:?}", config_path);
        let config = Config::load_or_default()?;

        Ok(Self {
            hardware: NornsHardware::new()?,
            sequencer: Sequencer::new_with_test_mode(test_mode)?,
            #[cfg(feature = "midi")]
            midi: MidiManager::new(&config.midi)?,
            grid: GridManager::new()?,
            screen: ScreenManager::new()?,
            co2: {
                let data_dir = std::path::PathBuf::from(&config.co2.data_dir);
                let co2_file_path = data_dir.join("simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_all_daily.csv");
                // info!("📊 CO2 file path: {:?}", co2_file_path);
                
                match Co2Manager::new(config.co2.clone()) {
                    Ok(manager) => {
                        // info!("📊 CO2 manager initialized successfully");
                        Some(manager)
                    }
                    Err(e) => {
                        // warn!("📊 CO2 manager initialization failed: {}", e);
                        // warn!("📊 Continuing without CO2 features");
                        None
                    }
                }
            },
            crow: simon_says_seeq_rust::crow::Crow::new()?,
            config,
            running: Arc::new(AtomicBool::new(false)),
            active_arm_action: None, // No ARM action initially active
            beat_led_flash_until: None,
            crow_cv1_muted: false, // Default CV1 not muted
            crow_cv2_muted: false, // Default CV2 not muted
            crow_cv3_muted: false, // Default CV3 not muted
            crow_cv4_muted: false, // Default CV4 not muted
            grid_two_button_0_pressed: false,
            grid_two_button_1_pressed: false,
            test_mode,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // info!("run says: Starting SimonSaysSeeq Rust application");

        // Verify and display grid assignment
        let connected_grids = self.grid.get_connected_grids();
        
        if connected_grids.len() == 0 {
            warn!("🎛️  GRID ASSIGNMENT: No grids connected");
            warn!("⚠️  Running without hardware - button presses and LEDs will be ignored");
        } else if connected_grids.len() == 2 {
            let (grid_one, grid_two) = self.grid.get_grid_ids_ordered()?;
            info!("🎛️  GRID ASSIGNMENT:");
            info!("   GRID_ONE: {}", grid_one);
            info!("   GRID_TWO: {}", grid_two);
            info!("✅ Two real grids ready for operation");
        } else {
            // Should never happen due to GridManager checks, but handle it
            error!("Invalid grid configuration: {} grids connected", connected_grids.len());
            return Err(anyhow!("Either 0 or 2 grids are required, found {}", connected_grids.len()));
        }

        // Initialize Crow USB serial communication
        if let Err(e) = self.crow.initialize() {
            // warn!("Failed to initialize Crow: {}. CV output will be disabled.", e);
        } else if self.crow.is_enabled() {
            // info!("🎛️  Crow USB serial initialized and ready");
        } else {
            // info!("🎛️  Crow CV output disabled (hardware feature not enabled)");
        }

        // Display CO2 data initialization status
        if let Some(ref co2) = self.co2 {
            // info!("📊 CO2 manager successfully initialized");
            // info!("🔍 CO2 Debug: Configuration - enabled: {}, data_dir: {:?}", self.config.co2.enabled, self.config.co2.data_dir);
            // info!("🔍 CO2 Debug: Record count: {}", co2.get_record_count());

            // info!("🔍 CO2 Debug: has_data() returns: {}", co2.has_data());
            
            if co2.has_data() {
                // info!("📊 {}", co2.get_data_summary());
                // info!("📊 {}", co2.get_status_string());
                // info!("📊 CO2 data is available and ready for use");
                // info!("⏱️  CO2 CV timing: 6 ticks per step (1 tick = 1 MIDI clock pulse)");
            } else {
                // warn!("📊 CO2 manager loaded but no data available");
                // warn!("🔍 CO2 Debug: This indicates either:");
                // warn!("🔍 CO2 Debug: - Files exist but contain no valid records");
                // warn!("🔍 CO2 Debug: - Files are missing from data directory");
                // warn!("🔍 CO2 Debug: - Data parsing failed for all records");
                // warn!("🔍 CO2 Debug: Check the detailed logs above for specific issues");
            }
        } else if self.config.co2.enabled {
            // warn!("📊 CO2 features enabled but manager failed to initialize");
            // warn!("🔍 CO2 Debug: This means CO2Manager::new() returned an error");
            // warn!("🔍 CO2 Debug: Check logs above for initialization failure details");
            // warn!("🔍 CO2 Debug: Sequencer will continue without CO2 features");
        } else {
            // info!("📊 CO2 features disabled in configuration");
        }

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

        // Internal sequencer thread removed - external clock slave mode only

        // Auto-start the sequencer for desktop testing (no hardware required)
        // Sequencer is driven by external MIDI clock only
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



        // Update main grid initially
        self.update_grid_display()?;

        // Main event loop
        self.main_loop(hw_rx, seq_rx, seq_tx)?;

        // Cleanup with timeout
        self.running.store(false, Ordering::SeqCst);

        // Give hardware thread a chance to exit gracefully
        // info!("run says: Shutting down threads...");

        // Try to join with timeout
        let hw_result = std::thread::spawn(move || hw_thread.join()).join();

        // If thread doesn't exit cleanly within reasonable time, force exit
        thread::sleep(Duration::from_millis(500));

        if hw_result.is_err() {
            // warn!("run says: Thread did not exit cleanly, forcing shutdown");
            std::process::exit(0);
        }

        // info!("run says: SimonSaysSeeq shut down successfully");
        Ok(())
    }

    fn main_loop(&mut self, hw_rx: Receiver<HardwareEvent>, seq_rx: Receiver<SequencerEvent>, seq_tx: Sender<SequencerEvent>) -> Result<()> {
        let mut last_screen_update = Instant::now();
        let screen_update_interval = Duration::from_millis(33); // ~30 FPS

        loop {
            // Handle hardware events (non-blocking)
            while let Ok(event) = hw_rx.try_recv() {
                if let Err(_e) = self.handle_hardware_event(event) {
                    // error!("Error handling hardware event: {}", e);
                }
            }

            // Poll grid for button events
            let _poll_start = Instant::now();
                match self.grid.read_button_events() {
                    Ok(grid_events) => {
                        // Process events from both grids - each should only report its own presses
                        let connected_grids = self.grid.get_connected_grids();
                        
                        for grid_event in grid_events {
                            // Check if this is from one of our expected grids
                            let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
                            let is_virtual_grid = grid_event.grid_id == "grid_one" || grid_event.grid_id == "grid_two";
                            let is_valid_grid = is_virtual_grid || 
                                              Some(&grid_event.grid_id) == grid_one.as_ref() || 
                                              Some(&grid_event.grid_id) == grid_two.as_ref();
                            
                            info!("GRID EVENT: grid_id={}, x={}, y={}, pressed={}, is_virtual={}, is_valid={}", 
                                  grid_event.grid_id, grid_event.x, grid_event.y, grid_event.pressed, 
                                  is_virtual_grid, is_valid_grid);
                            
                            if is_valid_grid {
                                // info!("GRID DEBUG: Processing event from {} at ({},{}) pressed={}", 
                                //       grid_event.grid_id, grid_event.x, grid_event.y, grid_event.pressed);
                                
                                let hardware_event = HardwareEvent::GridPress {
                                    grid_id: grid_event.grid_id,
                                    x: grid_event.x,
                                    y: grid_event.y,
                                    pressed: grid_event.pressed,
                                };
                                if let Err(_e) = self.handle_hardware_event(hardware_event) {
                                    // error!("Error handling grid event: {}", e);
                                }
                            } else {
                                // info!("GRID DEBUG: Ignoring event from unknown grid: {}", grid_event.grid_id);
                            }
                        }
                    }
                    Err(e) => {
                        // Don't spam errors for no events
                        if !e.to_string().contains("No events available") && !e.to_string().contains("would block") {
                            // debug!("Grid polling error: {}", e);
                        }
                    }
                }

            // Handle sequencer events (non-blocking)
            while let Ok(event) = seq_rx.try_recv() {
                if let Err(_e) = self.handle_sequencer_event(event) {
                    // error!("Error handling sequencer event: {}", e);
                }
            }

            // Handle MIDI input events (non-blocking)
            #[cfg(feature = "midi")]
            {
                let midi_events = self.midi.get_input_events();
                for event in midi_events {
                    if let Err(_e) = self.handle_midi_input_event(event, &seq_tx) {
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
                        // Right encoder - tempo control removed (sequencer uses external clock only)
                        // self.tempo = (self.tempo + delta as f32).clamp(20.0, 300.0);
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
                                Ok(_description) => {}, // info!("Undid: {}", description),
                                Err(_e) => {}, // warn!("Cannot undo: {}", e),
                            }
                        }
                        2 => {
                            // Left key - Stop
                            // info!("STOP TRIGGER: Key 2 (Left key) pressed - stopping sequencer");
                            self.sequencer.stop();
                            #[cfg(feature = "midi")]
                            self.midi.all_notes_off()?;
                        }
                        3 => {
                            // Right key - Start/Stop toggle
                            if self.sequencer.is_running() {
                                // info!("STOP TRIGGER: Key 3 (Right key) pressed - stopping sequencer via toggle");
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
                    // info!("STOP TRIGGER: StartStopToggle hardware event - stopping sequencer");
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
                // info!("Shutdown requested - initiating immediate exit");
                self.running.store(false, Ordering::SeqCst);
                // More aggressive force exit
                thread::spawn(|| {
                    thread::sleep(Duration::from_millis(500));
                    // warn!("Forcing immediate exit");
                    std::process::exit(0);
                });
            }
        }

        Ok(())
    }

    fn handle_sequencer_event(&mut self, event: SequencerEvent) -> Result<()> {
        match event {
            SequencerEvent::Step { step } => {
                // Step event - display updates handled

                // CO2 data will be handled in handle_co2_cv_per_step function

                // Process step for all active rows
                for row in 0..=6 { // Rows 0-6 are sequence rows
                    if let Some(note_events) = self.sequencer.get_step_events(row, 0, step) {
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
                // Send CV output using CO2 data - one record per step
                self.handle_co2_cv_per_step(step)?;

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
                                // Log to formal state logger
                                log_midi_note_on(midi_event.note, midi_event.velocity, midi_event.channel, 
                                               (midi_event.channel - 1) as usize, midi_event.step);
                                // info!("Sequencer A MIDI Note ON: {} vel:{} ch:{} step:{}",
                                //       midi_event.note, midi_event.velocity, midi_event.channel, midi_event.step);
                            } else {
                                self.midi.sequencer_a_note_off(midi_event.note, midi_event.channel)?;
                                // Log to formal state logger
                                log_midi_note_off(midi_event.note, midi_event.channel, 
                                                (midi_event.channel - 1) as usize);
                                // info!("Sequencer A MIDI Note OFF: {} ch:{} step:{}",
                                //       midi_event.note, midi_event.channel, midi_event.step);
                            }
                        }
                        'B' => {
                            // Sequencer B functionality removed
                        }
                        _ => {
                            // warn!("Unknown sequencer source: {}", midi_event.sequencer_source);
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
        // Log to formal state logger
        if pressed {
            log_button_press(grid_id, x, y, ButtonSource::Hardware);
        } else {
            log_button_release(grid_id, x, y, ButtonSource::Hardware);
        }
        
        // DEBUG: Log ALL grid presses to trace Sequence B button issue
        // info!("DEBUG Sequence B: Grid press {} at ({},{}) pressed={} - Sequence B active: {:?}", 
        //       grid_id, x, y, pressed, self.active_arm_action);
        
        // Handle ARM buttons first (row 7), even in Sequence B mode - BOTH press and release
        if y == 7 {
            info!("ARM BUTTON CHECK: y=7 detected, grid_id={}, x={}, pressed={}", grid_id, x, pressed);
            let connected_grids = self.grid.get_connected_grids();
            let (grid_one, _) = self.get_sorted_grid_ids(&connected_grids);
            // Accept virtual grid IDs from SysEx button commands (when no physical grids connected)
            // or match against actual physical grid device IDs
            let is_virtual_grid = grid_id == "grid_one" || grid_id == "grid_two";
            info!("ARM BUTTON: is_virtual_grid={}, grid_one={:?}", is_virtual_grid, grid_one);
            if is_virtual_grid || Some(grid_id) == grid_one.as_ref().map(|x| x.as_str()) {
                info!("ARM BUTTON: Passed grid ID check");
                if ArmAction::from_column(x).is_some() {
                    info!("ARM BUTTON: Found ARM action for column {}", x);
                    // info!("DEBUG Sequence B: ARM button detected at column {} press={} - proceeding to ARM logic", x, pressed);
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
        // Accept virtual grid IDs from SysEx button commands
        let is_virtual_grid = grid_id == "grid_one" || grid_id == "grid_two";
        let seq_x = if is_virtual_grid {
            // Virtual grids use logical mapping: grid_one = 0-15, grid_two = 16-31
            if grid_id == "grid_one" {
                x
            } else {
                x + 16
            }
        } else if Some(grid_id) == grid_one.as_ref().map(|x| x.as_str()) {
            // GRID_ONE (lowest ID): steps 0-15
            x
        } else if Some(grid_id) == grid_two.as_ref().map(|x| x.as_str()) {
            // GRID_TWO (second lowest ID): steps 16-31 (map from grid coordinates 0-15)
            x + 16
        } else {
            return Ok(()); // Invalid grid, ignore
        };
        let seq_y = y;
        
        // info!("GRID DEBUG: Grid press on {} at grid({},{}) -> seq({},{}) pressed={}", 
        //       grid_id, x, y, seq_x, seq_y, pressed);
        
        // Debug grid ID mapping
        let (grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
        // info!("GRID DEBUG: Available grids - GRID_ONE: {:?}, GRID_TWO: {:?}", grid_one, grid_two);
        // info!("GRID DEBUG: This press came from: {}", grid_id);

        // Handle sequence rows and ARM controls
        // Accept virtual grids (from SysEx) even when no physical grids connected
        let is_virtual_grid = grid_id == "grid_one" || grid_id == "grid_two";
        if is_virtual_grid || connected_grids.len() >= 2 {
            // Sequence rows 0-6 - only handle button presses, not releases
            if seq_y <= 6 && pressed {
                // Check if there's an active Euclidean ARM action
                if let Some(arm_action) = self.active_arm_action {
                    match arm_action {
                        ArmAction::EuclidianEvents => {
                            let events = (seq_x % 32) + 1; // Use full 32-step column + 1 for events (1-32)
                            // info!("ARM EUCLIDIAN_EVENTS: Generating rhythm on row {} with {} events (step {})", seq_y, events, seq_x);
                            
                            // Get current euclidean parameters to preserve length and rotation
                            if let Some(seq_a_row_state) = self.sequencer.get_row_states(seq_y) {
                                let current_length = seq_a_row_state.max_step + 1; // Convert from 0-based max_step to step count
                                let current_rotation = seq_a_row_state.euclidean_rotation;
                                self.sequencer.generate_euclidean_rhythm(seq_y, events, current_length, current_rotation);
                            } else {
                                // Fallback if seq_a_row_state is not available
                                // warn!("EuclidianEvents: Could not get seq_a_row_state for row {}, using fallback defaults", seq_y);
                                self.sequencer.generate_euclidean_rhythm(seq_y, events, 32, 0);
                            }
                            
                            // info!("ARM EUCLIDIAN_EVENTS: Successfully generated {} events on row {}", events, seq_y);
                            self.refresh_all_row_leds(seq_y)?;
                        },
                        ArmAction::EuclidianLength => {
                            let length = seq_x + 1; // Use full 32-step coordinate + 1 for length (1-32)
                            let length = length.clamp(1, 32); // Ensure valid range 1-32
                            // info!("ARM EUCLIDIAN_LENGTH: Generating rhythm on row {} with length {} (step {})", seq_y, length, seq_x);
                            
                            // Get current euclidean parameters to preserve events and rotation
                            if let Some(mut seq_a_row_state) = self.sequencer.get_row_states(seq_y) {
                                let current_events = seq_a_row_state.euclidean_events;
                                let current_rotation = seq_a_row_state.euclidean_rotation;
                                
                                // Generate the rhythm first
                                self.sequencer.generate_euclidean_rhythm(seq_y, current_events, length, current_rotation);
                                
                                // Reset step position if it's beyond the new length
                                let last_step = length - 1; // Convert to 0-based
                                if seq_a_row_state.current_row_step > last_step {
                                    // Reset to beginning of the row's own cycle
                                    seq_a_row_state.current_row_step = seq_a_row_state.first_step;
                                    let new_step = seq_a_row_state.current_row_step;
                                    self.sequencer.set_row_states(seq_y, seq_a_row_state);
                                    // info!("ARM EUCLIDIAN_LENGTH: Row {} step position reset to {} (was beyond new length {})", 
                                    //       seq_y, new_step, last_step);
                                }
                            } else {
                                // Fallback if seq_a_row_state is not available
                                // warn!("EuclidianLength: Could not get seq_a_row_state for row {}, using fallback defaults", seq_y);
                                self.sequencer.generate_euclidean_rhythm(seq_y, 5, length, 0);
                            }
                            
                            // info!("ARM EUCLIDIAN_LENGTH: Successfully generated length {} on row {}", length, seq_y);
                            self.refresh_all_row_leds(seq_y)?;
                        },
                        ArmAction::EuclidianRotation => {
                            let rotation = seq_x % 32; // Use full 32-step coordinate for rotation (0-31)
                            // info!("ARM EUCLIDIAN_ROTATION: Generating rhythm on row {} with rotation {} (step {})", seq_y, rotation, seq_x);
                            
                            // Get current euclidean parameters to preserve events and length
                            if let Some(seq_a_row_state) = self.sequencer.get_row_states(seq_y) {
                                let current_events = seq_a_row_state.euclidean_events;
                                let current_length = seq_a_row_state.max_step + 1; // Convert from 0-based max_step to step count
                                self.sequencer.generate_euclidean_rhythm(seq_y, current_events, current_length, rotation);
                            } else {
                                // Fallback if seq_a_row_state is not available
                                // warn!("EuclidianRotation: Could not get seq_a_row_state for row {}, using fallback defaults", seq_y);
                                self.sequencer.generate_euclidean_rhythm(seq_y, 5, 32, rotation);
                            }
                            
                            // info!("ARM EUCLIDIAN_ROTATION: Successfully generated rotation {} on row {}", rotation, seq_y);
                            self.refresh_all_row_leds(seq_y)?;
                        },
                        ArmAction::SetMaxStepForRow => {
                            let max_step = seq_x; // Use column directly as max_step (0-indexed)
                            let max_step = max_step.clamp(0, 31); // Ensure valid range 0-31
                            
                            // info!("ARM SET_MAX_STEP: Setting row {} max_step to {}", seq_y, max_step);
                            
                            // Set the max_step for this specific row
                            if let Some(mut seq_a_row_state) = self.sequencer.get_row_states(seq_y) {
                                let old_max_step = seq_a_row_state.max_step;
                                seq_a_row_state.max_step = max_step;
                                
                                // If current step is beyond new max_step, reset to beginning of the row's own cycle
                                if seq_a_row_state.current_row_step > max_step {
                                    // Reset to beginning of the row's own cycle
                                    seq_a_row_state.current_row_step = seq_a_row_state.first_step;
                                    // info!("ARM SET_MAX_STEP: Row {} step position reset to {} (was beyond new max_step {})", 
                                    //       seq_y, seq_a_row_state.current_row_step, max_step);
                                } else if max_step == 31 {
                                    // If max_step is 31 (full 32 steps), sync this row with global master step counter
                                    let master_step = self.sequencer.get_current_position();
                                    seq_a_row_state.current_row_step = master_step;
                                    // info!("ARM SET_MAX_STEP: Row {} synced with global master step counter (current_step={})", seq_y, seq_a_row_state.current_row_step);
                                }
                                
                                self.sequencer.set_row_states(seq_y, seq_a_row_state);
                                
                                // Log ARM action execution to formal state logger
                                log_arm_action_executed("SetMaxStepForRow", seq_y, seq_x, 
                                    &format!("Changed row {} max_step from {} to {}", seq_y, old_max_step, max_step));
                                
                                // info!("ARM SET_MAX_STEP: Successfully set row {} max_step to {}", seq_y, max_step);
                                self.refresh_all_row_leds(seq_y)?;
                            }
                        },
                        ArmAction::PresetGrid => {
                            // Apply preset pattern to this row
                            // info!("ARM PRESET_GRID: Applying preset pattern column {} to row {}", seq_x, seq_y);
                            self.sequencer.apply_preset_pattern(seq_y, seq_x);
                            
                            // Refresh the entire row to show the new pattern
                            self.refresh_all_row_leds(seq_y)?;
                            // info!("ARM PRESET_GRID: Successfully applied preset pattern column {} to row {}", seq_x, seq_y);
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
                    let (_grid_one, grid_two) = self.get_sorted_grid_ids(&connected_grids);
                    if Some(grid_id) == grid_two.as_ref().map(|x| x.as_str()) {
                        // GRID_TWO MIDI auto-detection controls (columns 0, 1) - require both pressed
                        if x == 0 || x == 1 {
                            if pressed {
                                // Track button press state
                                if x == 0 {
                                    self.grid_two_button_0_pressed = true;
                                    // info!("handle_grid_press says: GRID_TWO button 0 pressed for MIDI detection");
                                } else {
                                    self.grid_two_button_1_pressed = true;
                                    // info!("handle_grid_press says: GRID_TWO button 1 pressed for MIDI detection");
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

                        // GRID_TWO mute and tempo controls (columns 7-15)
                        else if x >= 7 && x <= 15 {
                        // GRID_TWO transport and tempo controls - handle both press and release
                        if pressed {
                            // Button press - light LED and perform action
                            #[cfg(feature = "hardware")]
                            {
                                self.grid.set_led(grid_id, x, seq_y, 10, "transport_button_press")?;
                                self.grid.refresh()?;
                            }
                            
                            match x {
                                7 => {
                                    // MUTE_CROW_1 latching toggle button - handled outside this block
                                }
                                8 => {
                                    // MUTE_CROW_2 latching toggle button - handled outside this block
                                }
                                9 => {
                                    // MUTE_CROW_3 latching toggle button - handled outside this block
                                }
                                10 => {
                                    // MUTE_CROW_4 latching toggle button - handled outside this block
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
                                        // Tempo control removed - sequencer uses external clock only
                                        info!("handle_grid_press says: Tempo control disabled - sequencer uses external MIDI clock");
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
                                    7 => {
                                        // MUTE_CROW_1 button - show state (ON/OFF)
                                        if self.crow_cv1_muted { LED_BRIGHT } else { LED_OFF }
                                    }
                                    8 => {
                                        // MUTE_CROW_2 button - show state (ON/OFF)
                                        if self.crow_cv2_muted { LED_BRIGHT } else { LED_OFF }
                                    }
                                    9 => {
                                        // MUTE_CROW_3 button - show state (ON/OFF)
                                        if self.crow_cv3_muted { LED_BRIGHT } else { LED_OFF }
                                    }
                                    10 => {
                                        // MUTE_CROW_4 button - show state (ON/OFF)
                                        if self.crow_cv4_muted { LED_BRIGHT } else { LED_OFF }
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

                        // Handle MUTE_CROW buttons as latching toggles (only on press, ignore release)
                        if pressed {
                            match x {
                                7 => {
                                    self.crow_cv1_muted = !self.crow_cv1_muted;
                                    if self.crow_cv1_muted {
                                        // info!("handle_grid_press says: MUTE_CROW_1 toggled ON - Crow CV1 output muted");
                                    } else {
                                        // info!("handle_grid_press says: MUTE_CROW_1 toggled OFF - Crow CV1 output enabled");
                                    }
                                }
                                8 => {
                                    self.crow_cv2_muted = !self.crow_cv2_muted;
                                    if self.crow_cv2_muted {
                                        // info!("handle_grid_press says: MUTE_CROW_2 toggled ON - Crow CV2 output muted");
                                    } else {
                                        // info!("handle_grid_press says: MUTE_CROW_2 toggled OFF - Crow CV2 output enabled");
                                    }
                                }
                                9 => {
                                    self.crow_cv3_muted = !self.crow_cv3_muted;
                                    if self.crow_cv3_muted {
                                        // info!("handle_grid_press says: MUTE_CROW_3 toggled ON - Crow CV3 output muted");
                                    } else {
                                        // info!("handle_grid_press says: MUTE_CROW_3 toggled OFF - Crow CV3 output enabled");
                                    }
                                }
                                10 => {
                                    self.crow_cv4_muted = !self.crow_cv4_muted;
                                    if self.crow_cv4_muted {
                                        // info!("handle_grid_press says: MUTE_CROW_4 toggled ON - Crow CV4 output muted");
                                    } else {
                                        // info!("handle_grid_press says: MUTE_CROW_4 toggled OFF - Crow CV4 output enabled");
                                    }
                                }
                                _ => {}
                            }
                        }
                        }
                        return Ok(());
                    }
                }
                
                // Regular ARM control handling for GRID_ONE buttons
                let (grid_one, _) = self.get_sorted_grid_ids(&connected_grids);
                // Accept virtual grid IDs from SysEx button commands (when no physical grids connected)
                // or match against actual physical grid device IDs
                let is_virtual_grid = grid_id == "grid_one" || grid_id == "grid_two";
                if is_virtual_grid || Some(grid_id) == grid_one.as_ref().map(|x| x.as_str()) {
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
                            // Log ARM action activation to formal state logger
                            log_arm_action_activated(&format!("{:?}", arm_action), x);
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
                // info!("ARM EUCLIDIAN_EVENTS: ARM button activated - press sequence row at column N for N+1 events (max 32)");
                // info!("ARM EUCLIDIAN_EVENTS: GRID_ONE columns 0-15 = events 1-16, GRID_TWO columns 0-15 = events 17-32");
            }
            ArmAction::EuclidianLength => {
                // Euclidean Length ARM button activated - waiting for sequence row press
                // info!("ARM EUCLIDIAN_LENGTH: ARM button activated - press sequence row at column N for length N+1 (max 32)");
                // info!("ARM EUCLIDIAN_LENGTH: GRID_ONE columns 0-15 = lengths 1-16, GRID_TWO columns 0-15 = lengths 17-32");
            }
            ArmAction::EuclidianRotation => {
                // Euclidean Rotation ARM button activated - waiting for sequence row press
                // info!("ARM EUCLIDIAN_ROTATION: ARM button activated - press sequence row at column N for rotation N (0-31)");
                // info!("ARM EUCLIDIAN_ROTATION: GRID_ONE columns 0-15 = rotation 0-15, GRID_TWO columns 0-15 = rotation 16-31");
            }
            ArmAction::Ratchet => {
                // Ratchet functionality - placeholder
                info!("ARM RATCHET: ARM button activated - not yet implemented");
            }
            ArmAction::SetMaxStepForRow => {
                // Set Max Step ARM button activated - waiting for sequence row press
                // info!("ARM SET_MAX_STEP: ARM button activated - press sequence row at column N to set max_step to N (0-indexed)");
                // info!("ARM SET_MAX_STEP: GRID_ONE columns 0-15 = max_step 0-15 (1-16 steps), GRID_TWO columns 0-15 = max_step 16-31 (17-32 steps)");
                // info!("ARM SET_MAX_STEP: Column 31 sets max_step to 31 (32 steps: 0-31)");
            }
            ArmAction::PresetGrid => {
                // Preset Grid ARM button activated - waiting for sequence row press
                // info!("ARM PRESET_GRID: ARM button activated - press any sequence row at any column for preset patterns");
                // info!("ARM PRESET_GRID: Column 0 = Clear row, 1-3 = Basic drums, 4-15 = African rhythms, 16-23 = Salsa, 24-31 = Jazz");
            }

        }
        Ok(())
    }









    fn update_screen(&mut self) -> Result<()> {
        self.screen.clear();

        // Display tempo - removed (sequencer uses external clock)
        // self.screen.draw_text(1, 7, &format!("Tempo: {:.1}", self.sequencer.get_tempo()));

        // Display current step
        let step = self.sequencer.get_position();
        self.screen.draw_text(1, 21, &format!("Step: {}", step));

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
        // Tempo visualization removed - sequencer uses external clock only
        // if self.config.display.show_tempo_viz {
        //     self.screen.draw_tempo_viz(self.sequencer.get_tempo());
        // }

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
        
        // DEBUG: Log grid update details for rows 0 and 6
        if row == 0 || row == 6 {
            trace!("🔥 LED HANDLER Row {}: Processing LED update old_step={} -> new_step={}", row, old_step, new_step);
            trace!("   Connected grids: {} grids", connected_grids.len());
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
            // info!("GRID_DEBUG: Grid assignments - GRID_ONE: {}, GRID_TWO: {}", grid_one_id, grid_two_id);
            
            if let Some(_row_state) = self.sequencer.get_row_states(row) {
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
                    if row == 0 || row == 6 {
                        trace!("   🔴 Setting OLD LED on GRID_ONE: grid_id={}, x={}, y={}, brightness={}", grid_one_id, old_step, row, old_brightness);
                    }
                    self.grid.set_led(grid_one_id, old_step, row, old_brightness, "grid_update_old_1")?;
                } else if old_step > 15 && old_step <= 31 {
                    // OLD position is on GRID_TWO (right grid): Coordinate mapping required
                    // Step 16 becomes grid_x=0, step 17 becomes grid_x=1, etc.
                    let grid_x = old_step - 16;
                    if row == 0 || row == 6 {
                        trace!("   🔴 Setting OLD LED on GRID_TWO: grid_id={}, x={}, y={}, brightness={} (original_step={})", grid_two_id, grid_x, row, old_brightness, old_step);
                    }
                    self.grid.set_led(grid_two_id, grid_x, row, old_brightness, "grid_update_old_2")?;
                } else {
                    // warn!("GRID_DEBUG: OLD step {} is out of bounds (valid range: 0-31), skipping LED update for row {}", old_step, row);
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
                    if row == 0 || row == 6 {
                        trace!("   🟢 Setting NEW LED on GRID_ONE: grid_id={}, x={}, y={}, brightness={}", grid_one_id, new_step, row, new_brightness);
                    }
                    self.grid.set_led(grid_one_id, new_step, row, new_brightness, "grid_update_new_1")?;
                } else if new_step > 15 && new_step <= 31 {
                    // NEW position is on GRID_TWO: Coordinate mapping required
                    let grid_x = new_step - 16;
                    if row == 0 || row == 6 {
                        trace!("   🟢 Setting NEW LED on GRID_TWO: grid_id={}, x={}, y={}, brightness={} (original_step={})", grid_two_id, grid_x, row, new_brightness, new_step);
                    }
                    self.grid.set_led(grid_two_id, grid_x, row, new_brightness, "grid_update_new_2")?;
                } else {
                    // warn!("GRID_DEBUG: NEW step {} is out of bounds (valid range: 0-31), skipping LED update for row {}", new_step, row);
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
        if connected_grids.len() == 2 {
            info!("GRID DEBUG: Dual-grid mode active - display will be painted by selective updates");
        } else if connected_grids.len() == 0 {
            warn!("GRID DEBUG: No grids connected - running without hardware");
        } else {
            error!("ERROR: Invalid grid configuration - {} grids connected (need 0 or 2)", connected_grids.len());
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
            if let Some(seq_a_row_state) = row_states {
                // Debug row state every few updates
                static mut DEBUG_COUNTER: u32 = 0;
                unsafe {
                    DEBUG_COUNTER += 1;
                    if DEBUG_COUNTER % 20 == 0 && seq_y <= 6 { // Debug all 7 sequencer rows, every 20 updates
                        // info!("🎯 Row {} current_step = {} (first_step={}, euclidean_length={}) [display: row {}]",
                        //       seq_y, seq_a_row_state.current_step, seq_a_row_state.first_step, seq_a_row_state.euclidean_length, seq_y + 1);
                    }
                }

                // Update this row's LEDs based on pattern values and current step
                for seq_x in 0..=15 {
                    let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                    let is_current_step = seq_x == seq_a_row_state.current_row_step;

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
            
            if let Some(seq_a_row_state) = self.sequencer.get_row_states(seq_y) {
                let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                let is_current_step = seq_x == seq_a_row_state.current_row_step;

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
            
            if let Some(seq_a_row_state) = self.sequencer.get_row_states(seq_y) {
                let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                let is_current_step = seq_x == seq_a_row_state.current_row_step;
                
                let brightness = match (pattern_value > 0, is_current_step) {
                    (false, false) => LED_OFF,     // No pattern, not current position
                    (false, true) => LED_DIM,      // No pattern, but current position
                    (true, false) => LED_BRIGHT,     // Has pattern, not current position
                    (true, true) => LED_MAX,      // Has pattern AND current position
                };
                
                // info!("GRID DEBUG: Setting single LED seq_x={}, seq_y={}, brightness={}, pattern_value={}, is_current={}", 
                //       seq_x, seq_y, brightness, pattern_value, is_current_step);
                
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
        // info!("GRID_DEBUG: get_sorted_grid_ids called with {} grids: {:?}", connected_grids.len(), connected_grids);
        
        // Always sort grids by ID for consistency
        let mut sorted_grids = connected_grids.to_vec();
        sorted_grids.sort();
        
        let grid_one = sorted_grids.first().cloned();
        let grid_two = if sorted_grids.len() > 1 { sorted_grids.get(1).cloned() } else { None };
        
        // info!("GRID_DEBUG: Sorted result - GRID_ONE (lowest ID): {:?}", grid_one);
        // info!("GRID_DEBUG: Sorted result - GRID_TWO (second lowest ID): {:?}", grid_two);
        
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
                if let Some(seq_a_row_state) = self.sequencer.get_row_states(seq_y) {
                    for seq_x in 0..=31 {
                        let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                        let is_current_step = seq_x == seq_a_row_state.current_row_step;
                        
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
            
            if let Some(seq_a_row_state) = self.sequencer.get_row_states(row) {
                info!("ARM DEBUG: Got row state for row {}: current_step={}", row, seq_a_row_state.current_row_step);
                
                for seq_x in 0..=31 {
                    let pattern_value = self.sequencer.get_grid_value(seq_x, row);
                    let is_current_step = seq_x == seq_a_row_state.current_row_step;
                    
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
                        // Randomize column functionality removed
                        info!("Randomize column {} (functionality disabled)", x);
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
                        // Same row - randomize this position (functionality disabled)
                        info!("Randomize position [{}, {}] (functionality disabled)", x, y);
                    } else {
                        // Different row - copy from held row to this row (functionality disabled)
                        info!("Copy from [{}, {}] to [{}, {}] (functionality disabled)", x, held_row, x, y);
                    }
                }
            }
        } else if held_rows.len() > 1 {
            // Multiple rows held - advanced operations
            let first_row = self.get_first_held_row().unwrap_or(0);

            // Copy pattern from first held row to current position (functionality disabled)
            info!("Copy pattern from row {} to row {} (functionality disabled)", first_row, y);
        }

        // Update grid display
        #[cfg(feature = "hardware")]
        self.update_grid_display()?;

        Ok(())
    }





    fn handle_co2_cv_per_step(&mut self, step: usize) -> Result<()> {
        // Get CO2 manager and advance to next record
        if let Some(ref mut co2_manager) = self.co2 {
            if let Some(co2_step_value) = co2_manager.advance_step() {
                // Convert CO2 value to voltage for output 1
                let co2_step_voltage = co2_manager.get_co2_voltage_offset(co2_step_value);
                
                // Get tick-based CO2 value for output 4 (advances on each tick)
                let co2_tick_value = if let Some(tick_value) = co2_manager.advance_tick() {
                    tick_value
                } else {
                    co2_step_value // Fall back to step value if tick data not available
                };
                
                // Set all 4 outputs based on CO2 data
                let step_delta_voltage = co2_manager.get_step_delta_voltage();
                let seasonal_anomaly_voltage = co2_manager.get_seasonal_anomaly_voltage();
                
                let voltages = [
                    co2_step_voltage,                    // Output 1: CO2 voltage (step-based)
                    (co2_tick_value - 318.0) / 482.0 * 10.0, // Output 2: Tick-based CO2 as unipolar voltage (318-800ppm → 0-10V)
                    step_delta_voltage,                  // Output 3: Step delta (bipolar -5V to +5V)
                    seasonal_anomaly_voltage,            // Output 4: Seasonal anomaly delta (bipolar -5V to +5V)
                ];

                // Clamp all voltages to Crow's safe range
                let clamped_voltages = [
                    voltages[0].clamp(-5.0, 10.0),
                    voltages[1].clamp(-5.0, 10.0),
                    voltages[2].clamp(-5.0, 10.0),
                    voltages[3].clamp(-5.0, 10.0),
                ];

                // Send to Crow CV outputs (with CV1, CV2, CV3 and CV4 muting support)
                if self.crow.is_enabled() {
                    let final_cv1 = if self.crow_cv1_muted { 0.0 } else { clamped_voltages[0] };
                    let final_cv2 = if self.crow_cv2_muted { 0.0 } else { clamped_voltages[1] };
                    let final_cv3 = if self.crow_cv3_muted { 0.0 } else { clamped_voltages[2] };
                    let final_cv4 = if self.crow_cv4_muted { 0.0 } else { clamped_voltages[3] };
                    if let Err(e) = self.crow.set_all_outputs(
                        final_cv1,
                        final_cv2, 
                        final_cv3, 
                        final_cv4
                    ) {
                        // warn!("Failed to send CO2 CV to Crow: {}", e);
                    } else {
                        // Only log when at least one CV output is not muted
                        if !self.crow_cv1_muted || !self.crow_cv2_muted || !self.crow_cv3_muted || !self.crow_cv4_muted {
                            // info!("🎛️  CO2 CV Output - Step#{} Tick#{}: Step:{:.2}ppm Tick:{:.2}ppm Δ:{:.3}ppm/YoY:{:.3}ppm -> [Step:{:.3}V, Tick:{:.3}V, StepΔ:{:.3}V, SeasonΔ:{:.3}V]", 
                            //       co2_manager.get_step_counter(), co2_manager.get_tick_counter(), 
                            //       co2_step_value, co2_tick_value,
                            //       co2_manager.get_step_delta(), co2_manager.get_seasonal_anomaly_delta(),
                            //       final_cv1, final_cv2, 
                            //       final_cv3, final_cv4);
                        }
                    }
                } else {
                    // debug!("Crow disabled - CO2 CV output ignored");
                }
            } else {
                // No CO2 data available, send zero voltages
                if self.crow.is_enabled() {
                    if let Err(_e) = self.crow.set_all_outputs(0.0, 0.0, 0.0, 0.0) {
                        // warn!("Failed to send zero CV to Crow: {}", e);
                    }
                }
                trace!("No CO2 data available - CV outputs set to 0V");
            }
        } else {
            // No CO2 manager, send zero voltages
            if self.crow.is_enabled() {
                if let Err(_e) = self.crow.set_all_outputs(0.0, 0.0, 0.0, 0.0) {
                    // warn!("Failed to send zero CV to Crow: {}", e);
                }
            }
            debug!("CO2 manager not available - CV outputs set to 0V");
        }

        Ok(())
    }

    fn handle_co2_tick_advance(&mut self) -> Result<()> {
        // Advance CO2 tick counter and update CV outputs 2 and 4
        if let Some(ref mut co2_manager) = self.co2 {
            if let Some(co2_tick_value) = co2_manager.advance_tick() {
                // Update CV output 2 with tick-based CO2 data
                let tick_voltage = (co2_tick_value - 318.0) / 482.0 * 10.0; // Same formula as in handle_co2_cv_per_step
                let clamped_tick_voltage = tick_voltage.clamp(-5.0, 10.0);
                
                // Send updates to Crow for output 2 only (CV4 is now quarter note based, not tick based)
                if self.crow.is_enabled() {
                    let final_tick_voltage = if self.crow_cv2_muted { 0.0 } else { clamped_tick_voltage };
                    if let Err(_e) = self.crow.send_command(&format!("output[2].volts = {:.6}", final_tick_voltage)) {
                        // warn!("Failed to send tick-based CO2 CV to Crow output 2: {}", e);
                    } else {
                        // Only log when CV2 is not muted
                        if !self.crow_cv2_muted {
                            // info!("🎛️  CO2 Tick CV - Tick#{}: {:.2}ppm -> Output 2: {:.3}V", 
                            //       co2_manager.get_tick_counter(), co2_tick_value, final_tick_voltage);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle MIDI input events
    #[cfg(feature = "midi")]
    fn handle_midi_input_event(&mut self, event: crate::midi::MidiInputEvent, seq_tx: &Sender<SequencerEvent>) -> Result<()> {
        use crate::midi::MidiInputEvent;
        
        match event {
            MidiInputEvent::ClockBeat => {
                // TEMPORARILY DISABLED FOR DEBUGGING LED DROPPING ISSUE
                // Flash LEDs 14 and 15 on GRID_TWO for 150ms
                // self.beat_led_flash_until = Some(Instant::now() + Duration::from_millis(150));
                
                // Synchronize sequencer tempo with external MIDI clock (less frequent logging)
                #[cfg(feature = "midi")]
                {
                    // Tempo application removed - sequencer uses clock ticks only
                }
                
                // Tempo display removed - sequencer uses external clock
                trace!("handle_midi_input_event says: MIDI Clock Beat");
            }
            MidiInputEvent::ClockStart => {
                info!("handle_midi_input_event says: MIDI Clock Start received - starting sequencer");
                self.sequencer.start();

                // Reset external clock tick counter on start
                EXTERNAL_CLOCK_TICK_COUNTER.store(0, Ordering::SeqCst);

                // Reset CO2 counters on MIDI start
                if let Some(ref mut co2_manager) = self.co2 {
                    co2_manager.reset_counters();
                    debug!("🔄 CO2 counters reset on MIDI start");
                }
                
                // Synchronize sequencer tempo with external MIDI clock on start
                #[cfg(feature = "midi")]
                {
                    // Tempo application removed - sequencer uses clock ticks only
                }
            }
            MidiInputEvent::ClockStop => {
                info!("STOP TRIGGER: MIDI Clock Stop received from external device - stopping sequencer");
                info!("handle_midi_input_event says: MIDI Clock Stop received - stopping sequencer");
                self.sequencer.stop();
                
                // Reset external clock tick counter on stop
                EXTERNAL_CLOCK_TICK_COUNTER.store(0, Ordering::SeqCst);
                
                // Reset CO2 counters on MIDI stop
                if let Some(ref mut co2_manager) = self.co2 {
                    co2_manager.reset_counters();
                    debug!("🔄 CO2 counters reset on MIDI stop");
                }
                
                // TEMPORARILY DISABLED FOR DEBUGGING LED DROPPING ISSUE
                // Clear beat LEDs when clock stops
                // self.beat_led_flash_until = None;

            }
            MidiInputEvent::ClockTick => {
                // External clock slave mode: advance sequencer directly on MIDI clock
                if self.sequencer.is_running() {
                    // MIDI clock runs at 24 PPQ, we get 6 ticks per step (every clock, 6 clocks per step)
                    let tick_count = EXTERNAL_CLOCK_TICK_COUNTER.load(Ordering::SeqCst);
                    
                    // Check if we should advance step BEFORE incrementing, so step 0 happens at tick 0
                    if tick_count % 6 == 0 { // Every 6th MIDI clock = 1 step (16th note: 24÷4 = 6)
                        // Advance sequencer step based on external clock
                        if let Err(e) = self.sequencer.external_advance_step(seq_tx) {
                            warn!("External clock step advancement failed: {}", e);
                        }
                    }
                    
                    EXTERNAL_CLOCK_TICK_COUNTER.fetch_add(1, Ordering::SeqCst);
                    
                    // Advance CO2 tick counter on every MIDI clock (gives us 6 ticks per step)
                    self.handle_co2_tick_advance()?;
                }
            }
            MidiInputEvent::ExternalClockTimeout => {
                // External MIDI clock timed out - preserve the last known external tempo
                #[cfg(feature = "midi")]
                {
                    // Tempo preservation removed - sequencer uses clock ticks only
                    info!("handle_midi_input_event says: External clock timeout");
                }
            }
            MidiInputEvent::NoteOn { channel, note, velocity } => {
                // Sequencer B MIDI recording functionality removed
                trace!("MIDI Note On (sequencer B removed): Channel: {}, Note: {}, Velocity: {}", channel, note, velocity);
            }
            MidiInputEvent::NoteOff { channel, note } => {
                // Sequencer B MIDI recording functionality removed  
                trace!("MIDI Note Off (sequencer B removed): Channel: {}, Note: {}", channel, note);
            }
            MidiInputEvent::ControlChange { .. } => {
                // Control Change events - currently not handled in main app
                // Could be used for MIDI CC mapping in future
            }
            MidiInputEvent::ClockContinue => {
                // MIDI Clock Continue - currently not handled
                // Similar to ClockStart but resumes from current position
            }
            MidiInputEvent::SysEx { data } => {
                // Handle SimonSaysSeeQ SysEx commands
                // Format: F0 7D 53 53 51 <cmd> F7
                if data.len() >= 7 && data[1] == 0x7D && data[2] == 0x53 && data[3] == 0x53 && data[4] == 0x51 {
                    let command = data[5];
                    match command {
                        0x01 => {
                            // Reload pattern from current_pattern.json
                            info!("🔄 SysEx reload pattern command received");
                            match self.sequencer.load_current_pattern_from_file() {
                                Ok(()) => {
                                    info!("✅ Pattern reloaded successfully from current_pattern.json");
                                }
                                Err(e) => {
                                    warn!("⚠️ Failed to reload pattern: {}", e);
                                }
                            }
                        }
                        0x02 => {
                            // Button press/release: F0 7D 53 53 51 02 <row> <col> <press> F7
                            if data.len() >= 10 {
                                let row = data[6] as usize;
                                let col = data[7] as usize;
                                let press = data[8] != 0;
                                
                                info!("🔘 SysEx button {} received: row={}, col={}", 
                                      if press { "press" } else { "release" }, row, col);
                                
                                // Determine grid_id based on column
                                let grid_id = if col < 16 { "grid_one" } else { "grid_two" };
                                let adjusted_col = if col < 16 { col } else { col - 16 };
                                
                                // Inject as virtual button event
                                self.grid.inject_virtual_button(grid_id, adjusted_col, row, press);
                            } else {
                                warn!("⚠️ Invalid SysEx button command: insufficient data length");
                            }
                        }
                        0x10 => {
                            // Test mode query: F0 7D 53 53 51 10 F7
                            // Respond with 0x11 (test mode active) or 0x12 (test mode NOT active)
                            info!("🧪 SysEx test mode query received");
                            let is_test_mode = self.sequencer.is_test_mode();
                            let response_command = if is_test_mode { 0x11 } else { 0x12 };
                            let response = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, response_command, 0xF7];
                            
                            #[cfg(feature = "midi")]
                            if let Err(e) = self.midi.send_sysex(&response) {
                                warn!("⚠️ Failed to send test mode response: {}", e);
                            } else {
                                info!("✅ Sent test mode response: {} (0x{:02X})", 
                                      if is_test_mode { "ACTIVE" } else { "NOT ACTIVE" }, 
                                      response_command);
                            }
                        }
                        _ => {
                            debug!("Unknown SimonSaysSeeQ SysEx command: 0x{:02X}", command);
                        }
                    }
                }
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
        //             // Update MUTE_CROW_1 button LED (column 7, row 7)
        //             let mute_cv1_brightness = if self.crow_cv1_muted { LED_BRIGHT } else { LED_OFF };
        //             self.grid.set_led(&grid_two_id, 7, 7, mute_cv1_brightness, "mute_crow_1_button")?;
        //             
        //             // Update MUTE_CROW_2 button LED (column 8, row 7)
        //             let mute_cv2_brightness = if self.crow_cv2_muted { LED_BRIGHT } else { LED_OFF };
        //             self.grid.set_led(&grid_two_id, 8, 7, mute_cv2_brightness, "mute_crow_2_button")?;
        //             
        //             // Update MUTE_CROW_3 button LED (column 9, row 7)
        //             let mute_cv3_brightness = if self.crow_cv3_muted { LED_BRIGHT } else { LED_OFF };
        //             self.grid.set_led(&grid_two_id, 9, 7, mute_cv3_brightness, "mute_crow_3_button")?;
        //             
        //             // Update MUTE_CROW_4 button LED (column 10, row 7)
        //             let mute_cv4_brightness = if self.crow_cv4_muted { LED_BRIGHT } else { LED_OFF };
        //             self.grid.set_led(&grid_two_id, 10, 7, mute_cv4_brightness, "mute_crow_4_button")?;
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

    // Check for test mode flag
    let test_mode = args.iter().any(|arg| arg == "--test-mode");

    // Handle help flag
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        let git_hash = option_env!("GIT_HASH").unwrap_or("unknown");
        println!("SimonSaysSeeq Rust v{} ({})", env!("CARGO_PKG_VERSION"), git_hash);
        println!("A high-performance sequencer for Norns hardware\n");
        println!("USAGE:");
        println!("    simon_says_seeq [OPTIONS]\n");
        println!("OPTIONS:");
        println!("    -h, --help           Print help information");
        println!("    -v, --version        Print version information");
        println!("    --config <FILE>      Use custom configuration file");
        println!("    --no-hardware        Disable hardware features");
        println!("    --no-midi            Disable MIDI features");
        println!("    --test-mode          Enable test mode (auto-load test_pattern_1.json, disable auto-save)");


        return Ok(());
    }

    // Handle version flag
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        let git_hash = option_env!("GIT_HASH").unwrap_or("unknown");
        println!("SimonSaysSeeq Rust v{} ({})", env!("CARGO_PKG_VERSION"), git_hash);
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

    // Initialize formal state logger
    use simon_says_seeq_rust::formal_state_logger;
    if let Err(e) = formal_state_logger::init_formal_logger() {
        warn!("Failed to initialize formal state logger: {}", e);
    } else {
        info!("📝 Formal state logger initialized");
    }

    // Log startup banner with version and timestamp
    info!("════════════════════════════════════════════════════════");
    let git_hash = option_env!("GIT_HASH").unwrap_or("unknown");
    info!("🎵 {}", version::get_version_info());
    info!("📦 Cargo Version: v{} ({})", env!("CARGO_PKG_VERSION"), git_hash);
    info!("📅 Startup Time: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    info!("🔧 Build Profile: {}", if cfg!(debug_assertions) { "debug" } else { "release" });
    info!("⚙️  Features: MIDI={}, Hardware={}", cfg!(feature = "midi"), cfg!(feature = "hardware"));
    info!("⏰ Timing Mode: EXTERNAL CLOCK SLAVE ONLY (basic sync)");
    if test_mode {
        info!("🧪 TEST MODE ENABLED - auto-load test_pattern_1.json, no auto-save");
    }
    info!("════════════════════════════════════════════════════════");

    // Create and run application
    let mut app = SimonSaysSeeq::new_with_test_mode(test_mode)?;

    // Single-grid mode removed - application now requires exactly 2 grids

    app.run()?;

    Ok(())
}
