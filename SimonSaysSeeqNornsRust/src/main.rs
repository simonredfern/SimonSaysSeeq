//! SimonSaysSeeq Pure Rust Implementation for Norns
//! 
//! A high-performance sequencer application that runs directly on Norns hardware
//! without requiring the Norns Lua environment.

use anyhow::Result;
use log::{info, warn, error, debug};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

mod hardware;
mod sequencer;
mod midi;

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
        }
    }
}

use crossbeam_channel::Receiver;

use hardware::{NornsHardware, HardwareEvent};
use sequencer::{Sequencer, SequencerEvent};
#[cfg(feature = "midi")]
use midi::MidiManager;
use simon_says_seeq_rust::grid_osc::GridManager;
#[cfg(feature = "hardware")]
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
    #[cfg(feature = "hardware")]
    screen: ScreenManager,
    co2: Co2Manager,
    config: Config,
    running: Arc<AtomicBool>,
    tempo: f32,
    main_grid_preference: Option<String>,
    // Active ARM action for row 7 (control row) - only one can be active at a time
    active_arm_action: Option<ArmAction>,
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
            grid: GridManager::new()?,
            #[cfg(feature = "hardware")]
            screen: ScreenManager::new()?,
            co2: Co2Manager::new(config.co2.clone())?,
            config,
            running: Arc::new(AtomicBool::new(false)),
            tempo: initial_tempo,
            main_grid_preference: Some("m2949672".to_string()), // Default main grid
            active_arm_action: None, // No ARM action initially active
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // info!("run says: Starting SimonSaysSeeq Rust application");
        
        // List connected devices for debugging
        #[cfg(feature = "hardware")]
        {
            let connected_grids = self.grid.get_connected_grids();
            if connected_grids.is_empty() {
                // info!("No monome grid devices found - connect grid for hardware functionality");
            } else {
                // info!("Found {} monome grid device(s): {:?}", connected_grids.len(), connected_grids);
            }
        }
        
        #[cfg(not(feature = "hardware"))]
        {
            // info!("run says: Simulation mode - no actual hardware detection");
        }
        
        // Show control instructions
        // info!("run says: Controls:");
        // info!("run says:   Ctrl+C: Stop application");
        // #[cfg(not(feature = "hardware"))]
        // info!("run says:   Space+Enter: Start/Stop sequencer (simulation mode)");
        // #[cfg(not(feature = "hardware"))]
        // info!("run says:   1-4/QWER/ASDF/ZXCV+Enter: Simulate grid press");
        
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
        #[cfg(feature = "hardware")]
        {
            // Set test patterns for all rows to verify display pipeline (0-indexed)
            for row in 0..=6 {
                self.sequencer.set_grid_value(0, row, 1);   // Step 0 (display: step 1)
                self.sequencer.set_grid_value(4, row, 2);   // Step 4 (display: step 5)
                self.sequencer.set_grid_value(8, row, 1);   // Step 8 (display: step 9)
                self.sequencer.set_grid_value(12, row, 2);  // Step 12 (display: step 13)
            }
            
            // Update main grid initially
            self.update_grid_display()?;
        }
        
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
            #[cfg(feature = "hardware")]
            {
                let poll_start = Instant::now();
                match self.grid.read_button_events() {
                    Ok(grid_events) => {
                        // Filter events to only process main grid and only button presses
                        let connected_grids = self.grid.get_connected_grids();
                        let main_grid_id = self.get_main_grid_id(&connected_grids);
                        
                        for grid_event in grid_events {
                            // Only process events from main grid (both presses and releases)
                            if Some(&grid_event.grid_id) == main_grid_id.as_ref() {
                                // Grid event detected - removed timing for performance
                                
                                let hardware_event = HardwareEvent::GridPress {
                                    grid_id: grid_event.grid_id,
                                    x: grid_event.x,
                                    y: grid_event.y,
                                    pressed: grid_event.pressed,
                                };
                                if let Err(e) = self.handle_hardware_event(hardware_event) {
                                    // error!("Error handling grid event: {}", e);
                                }
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
            }
            
            // Handle sequencer events (non-blocking)
            while let Ok(event) = seq_rx.try_recv() {
                if let Err(e) = self.handle_sequencer_event(event) {
                    // error!("Error handling sequencer event: {}", e);
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
                        self.tempo = (self.tempo + delta as f32).clamp(20.0, 200.0);
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
                #[cfg(feature = "hardware")]
                self.handle_grid_press(&grid_id, x, y, pressed)?;
                #[cfg(not(feature = "hardware"))]
                {
                    let button_name = match (x, y) {
                        (0, 0) => "1", (1, 0) => "2", (2, 0) => "3", (3, 0) => "4",
                        (0, 1) => "Q", (1, 1) => "W", (2, 1) => "E", (3, 1) => "R",
                        (0, 2) => "A", (1, 2) => "S", (2, 2) => "D", (3, 2) => "F",
                        (0, 3) => "Z", (1, 3) => "X", (2, 3) => "C", (3, 3) => "V",
                        _ => "?",
                    };
                    // if pressed {
                    //     info!("handle_hardware_event says: Grid button {} PRESSED: ({}, {})", button_name, x, y);
                    // } else {
                    //     info!("handle_hardware_event says: Grid button {} released: ({}, {})", button_name, x, y);
                    // }
                    
                    // Update grid display
                    // Use native 0-based grid coordinates directly
                    let seq_x = x;
                    let seq_y = y;
                    if pressed {
                        self.grid.set_led(grid_id, x, y, 15, "handle_hardware_event")?;
                    } else {
                        self.grid.set_led(grid_id, x, y, 0, "handle_hardware_event")?;
                    }
                }
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
                for row in 1..=7 { // Rows 1-7 are sequence rows
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
                                    info!("🎹 MIDI Note ON: {} vel:{} ch:{}", note_event.note, note_event.velocity, note_event.channel);
                                } else {
                                    info!("🎹 MIDI Note OFF: {} ch:{}", note_event.note, note_event.channel);
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
                #[cfg(feature = "hardware")]
                self.screen.set_beat_indicator(beat);
                #[cfg(not(feature = "hardware"))]
                debug!("Beat indicator: {}", beat);
            }
            
            SequencerEvent::GridUpdate { row, old_step, new_step } => {
                // Selective grid update - only update changed LEDs
                #[cfg(feature = "hardware")]
                self.handle_grid_update(row, old_step, new_step)?;
            }
            
            SequencerEvent::MidiEvent(midi_event) => {
                // Handle MIDI events from sequencer
                #[cfg(feature = "midi")]
                {
                    if midi_event.note_on {
                        self.midi.note_on(midi_event.note, midi_event.velocity, midi_event.channel)?;
                        info!("MIDI Note ON: {} vel:{} ch:{} step:{}", 
                              midi_event.note, midi_event.velocity, midi_event.channel, midi_event.step);
                    } else {
                        self.midi.note_off(midi_event.note, midi_event.channel)?;
                        info!("MIDI Note OFF: {} ch:{} step:{}", 
                              midi_event.note, midi_event.channel, midi_event.step);
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
    
    #[cfg(feature = "hardware")]
    fn handle_grid_press(&mut self, grid_id: &str, x: usize, y: usize, pressed: bool) -> Result<()> {
        // Use 0-based coordinates throughout (post-refactor)
        let seq_x = x;
        let seq_y = y;
        
        // info!("handle_grid_press says: Button {}: grid {} at ({}, {}) pressed={}", 
        //       if pressed { "PRESS" } else { "RELEASE" }, grid_id, x, y, pressed);
        
        // Should only get main grid events now due to filtering, but double-check
        let connected_grids = self.grid.get_connected_grids();
        let main_grid = self.get_main_grid_id(&connected_grids);
        let is_main_grid = main_grid.as_ref().map(|id| id == grid_id).unwrap_or(false);
        
        if is_main_grid {
            // Main sequencer grid - ROWS 0-6 (0-indexed)
            if seq_y <= 6 {
                // Sequence rows 0-6 - only handle button presses, not releases
                if pressed {
                    // Check if there's an active Euclidean ARM action
                    if let Some(arm_action) = self.active_arm_action {
                        match arm_action {
                            ArmAction::EuclidianEvents => {
                                let events = seq_x + 1; // Column + 1
                                info!("ARM EUCLIDIAN_EVENTS: Generating rhythm on row {} with {} events (column {})", seq_y, events, seq_x);
                                self.sequencer.generate_euclidean_rhythm(seq_y, events, 16, 0);
                                info!("ARM EUCLIDIAN_EVENTS: Successfully generated {} events on row {}", events, seq_y);
                                #[cfg(feature = "hardware")]
                                self.refresh_all_pattern_leds()?;
                            },
                            ArmAction::EuclidianLength => {
                                let length = seq_x + 1; // Column + 1
                                info!("ARM EUCLIDIAN_LENGTH: Generating rhythm on row {} with length {} (column {})", seq_y, length, seq_x);
                                self.sequencer.generate_euclidean_rhythm(seq_y, 5, length, 0);
                                info!("ARM EUCLIDIAN_LENGTH: Successfully generated length {} on row {}", length, seq_y);
                                #[cfg(feature = "hardware")]
                                self.refresh_all_pattern_leds()?;
                            },
                            ArmAction::EuclidianRotation => {
                                let rotation = seq_x; // Column + 0
                                info!("ARM EUCLIDIAN_ROTATION: Generating rhythm on row {} with rotation {} (column {})", seq_y, rotation, seq_x);
                                self.sequencer.generate_euclidean_rhythm(seq_y, 5, 16, rotation);
                                info!("ARM EUCLIDIAN_ROTATION: Successfully generated rotation {} on row {}", rotation, seq_y);
                                #[cfg(feature = "hardware")]
                                self.refresh_all_pattern_leds()?;
                            },
                            _ => {
                                // For other ARM actions, handle normally
                                self.handle_normal_grid_operation(grid_id, seq_x, seq_y)?;
                            }
                        }
                    } else {
                        // No ARM action active, handle normally
                        self.handle_normal_grid_operation(grid_id, seq_x, seq_y)?;
                    }
                }
            } else if seq_y == 7 {
                // Control row (7, 0-indexed) - handle both presses and releases
                info!("ARM CONTROL: Row 7 button {} {}", seq_x, if pressed { "PRESSED" } else { "RELEASED" });
                
                // Check if this column corresponds to a valid ARM action
                if let Some(arm_action) = ArmAction::from_column(seq_x) {
                    if pressed {
                        // Turn off previous ARM button if any
                        if let Some(prev_action) = self.active_arm_action {
                            let prev_column = prev_action.to_column();
                            info!("ARM CONTROL: Deactivating previous ARM action {:?} (column {})", prev_action, prev_column);
                            #[cfg(feature = "hardware")]
                            {
                                self.grid.set_led(grid_id, prev_column, seq_y, 0, "arm_action_deactivate")?;
                            }
                        }
                        
                        // Set new active ARM action and light it up
                        self.active_arm_action = Some(arm_action);
                        info!("ARM CONTROL: Activated ARM action {:?} (column {}) - waiting for sequence row press", arm_action, seq_x);
                        #[cfg(feature = "hardware")]
                        {
                            self.grid.set_led(grid_id, seq_x, seq_y, 10, "arm_action_press")?;
                            self.grid.refresh()?;
                        }
                        
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
                info!("ARM EUCLIDIAN_EVENTS: ARM button activated - press sequence row at column N for N+1 events");
            }
            ArmAction::EuclidianLength => {
                // Euclidean Length ARM button activated - waiting for sequence row press
                info!("ARM EUCLIDIAN_LENGTH: ARM button activated - press sequence row at column N for length N+1");
            }
            ArmAction::EuclidianRotation => {
                // Euclidean Rotation ARM button activated - waiting for sequence row press
                info!("ARM EUCLIDIAN_ROTATION: ARM button activated - press sequence row at column N for rotation N");
            }
            ArmAction::Ratchet => {
                // Ratchet functionality - placeholder
                info!("ARM RATCHET: ARM button activated - not yet implemented");
            }
            ArmAction::PresetGrid => {
                // Preset grid functionality - placeholder
                info!("ARM PRESET_GRID: ARM button activated - not yet implemented");
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
    
    #[cfg(not(feature = "hardware"))]
    fn handle_mozart_grid_press(&mut self, x: usize, y: usize) -> Result<()> {
        // Simulation mode
        let base_note = 60;
        let note = base_note + (7 - y) * 5 + x;
        self.sequencer.set_mozart_value(x + 1, y + 1, note as u8);
        info!("Mozart grid (sim): Set [{}][{}] = note {}", x + 1, y + 1, note);
        Ok(())
    }
    
    fn update_screen(&mut self) -> Result<()> {
        #[cfg(feature = "hardware")]
        {
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
        }
        
        #[cfg(not(feature = "hardware"))]
        {
            // Simulation mode - display info to console
            let (step, bar) = self.sequencer.get_position();
            let transport_text = if self.sequencer.is_running() { "RUNNING" } else { "STOPPED" };
            // info!("update_screen says: Sequencer: {} | Tempo: {:.1} BPM | Step: {} | Bar: {}", transport_text, self.sequencer.get_tempo(), step, bar);
        }
        
        Ok(())
    }
    
    /// Selective grid update - only update specific LEDs that changed
    #[cfg(feature = "hardware")]
    fn handle_grid_update(&mut self, row: usize, old_step: usize, new_step: usize) -> Result<()> {
        let connected_grids = self.grid.get_connected_grids();
        
        if let Some(main_grid_id) = self.get_main_grid_id(&connected_grids) {
            if let Some(row_state) = self.sequencer.get_row_states(row) {
                // Update old position LED (turn off position indicator)
                let old_pattern_value = self.sequencer.get_grid_value(old_step, row);
                let old_brightness = if old_pattern_value > 0 { 10 } else { 0 }; // Pattern only or off
                self.grid.set_led(&main_grid_id, old_step, row, old_brightness, "grid_update_old")?;
                
                // Update new position LED (turn on position indicator)
                let new_pattern_value = self.sequencer.get_grid_value(new_step, row);
                let new_brightness = if new_pattern_value > 0 { 14 } else { 6 }; // Pattern+position or position only
                self.grid.set_led(&main_grid_id, new_step, row, new_brightness, "grid_update_new")?;
            }
        }
        
        self.grid.refresh()?;
        Ok(())
    }
    
    /// Full grid display update (only used for initialization)
    #[cfg(feature = "hardware")]
    fn update_grid_display(&mut self) -> Result<()> {
        let connected_grids = self.grid.get_connected_grids();
        
        // Update only the main grid with position scrolling
        if let Some(main_grid_id) = self.get_main_grid_id(&connected_grids) {
            self.update_main_grid_display(&main_grid_id)?;
        }
        
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
                        // info!("🎯 Row {} current_step = {} (first_step={}, last_step={}) [display: row {}]", 
                        //       seq_y, row_state.current_step, row_state.first_step, row_state.last_step, seq_y + 1);
                    }
                }
                
                for seq_x in 0..=15 {
                    let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                    let is_current_step = seq_x == row_state.current_step;
                    
                    // 4 brightness levels based on pattern and position:
                    let brightness = match (pattern_value > 0, is_current_step) {
                        (false, false) => 0,     // 0% - No pattern, not current position
                        (false, true) => 6,      // 40% - No pattern, but current position  
                        (true, false) => 10,     // 65% - Has pattern, not current position
                        (true, true) => 14,      // 90% - Has pattern AND current position
                    };
                    

                    
                    // Use native 0-based grid coordinates directly
                    self.grid.set_led(grid_id, seq_x, seq_y, brightness, "update_main_grid_display")?;
                }
            } else {
                // warn!("No row settings found for row {} (display: row {})", seq_y, seq_y + 1);
            }
        }
        
        Ok(())
    }
    /// Update single LED with current pattern and position state
    #[cfg(feature = "hardware")]
    fn update_single_led(&mut self, grid_id: &str, seq_x: usize, seq_y: usize) -> Result<()> {
        // Only update LEDs for rows 0-6 (0-indexed)
        if seq_y > 6 {
            return Ok(());
        }
        
        if let Some(row_state) = self.sequencer.get_row_states(seq_y) {
            let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
            let is_current_step = seq_x == row_state.current_step;
            
            // Same brightness logic as main display
            let brightness = match (pattern_value > 0, is_current_step) {
                (false, false) => 0,     // No pattern, not current position
                (false, true) => 6,      // No pattern, but current position  
                (true, false) => 10,     // Has pattern, not current position
                (true, true) => 14,      // Has pattern AND current position
            };
            
            // Use native 0-based grid coordinates directly
            self.grid.set_led(grid_id, seq_x, seq_y, brightness, "update_single_led")?;
        }
        
        Ok(())
    }
    
    /// Set main grid preference
    pub fn set_main_grid_preference(&mut self, grid_id: String) {
        info!("Setting main grid preference to: {}", grid_id);
        self.main_grid_preference = Some(grid_id);
    }
    
    /// Get the main grid ID based on preference or discovery order
    fn get_main_grid_id(&self, connected_grids: &[String]) -> Option<String> {
        if let Some(preferred_id) = &self.main_grid_preference {
            if connected_grids.contains(preferred_id) {
                return Some(preferred_id.clone());
            } else {
                warn!("Preferred main grid {} not found, using first available", preferred_id);
            }
        }
        
        connected_grids.first().cloned()
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
        for y in 1..=8 {
            for x in 1..=16 {
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
        let connected_grids = self.grid.get_connected_grids();
        if let Some(main_grid_id) = self.get_main_grid_id(&connected_grids) {
            // Update all LEDs using the same logic as update_single_led
            for seq_y in 0..=6 {
                if let Some(row_state) = self.sequencer.get_row_states(seq_y) {
                    for seq_x in 0..=15 {
                        let pattern_value = self.sequencer.get_grid_value(seq_x, seq_y);
                        let is_current_step = seq_x == row_state.current_step;
                        
                        // Same brightness logic as update_single_led and update_main_grid_display
                        let brightness = match (pattern_value > 0, is_current_step) {
                            (false, false) => 0,     // No pattern, not current position
                            (false, true) => 6,      // No pattern, but current position  
                            (true, false) => 10,     // Has pattern, not current position
                            (true, true) => 14,      // Has pattern AND current position
                        };
                        
                        self.grid.set_led(&main_grid_id, seq_x, seq_y, brightness, "refresh_all_pattern_leds")?;
                    }
                }
            }
            self.grid.refresh()?;
        }
        Ok(())
    }

    /// Handle normal grid operation (toggle pattern)
    #[cfg(feature = "hardware")]
    fn handle_normal_grid_operation(&mut self, grid_id: &str, seq_x: usize, seq_y: usize) -> Result<()> {
        // Check if any positions are held for advanced operations
        if self.has_held_positions() {
            self.handle_advanced_grid_operation(seq_x, seq_y)?;
        } else {
            // Normal grid operation - toggle or cycle ratchet
            let current_value = self.sequencer.get_grid_value(seq_x, seq_y);
            let new_value = if current_value > 0 { 0 } else { 1 }; // Simple on/off toggle
            
            self.sequencer.set_grid_value(seq_x, seq_y, new_value);
            // info!("handle_normal_grid_operation says: Toggle: grid[{}][{}] {} -> {} (step {}, row {})", seq_x, seq_y, current_value, new_value, seq_x + 1, seq_y + 1); // +1 for user display
            
            // Update only this specific LED for immediate response
            self.update_single_led(grid_id, seq_x, seq_y)?;
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
                        // Only randomize rows 1 and 2
                        self.sequencer.randomize_section(x, 1);
                        self.sequencer.randomize_section(x, 2);
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
            let first_row = self.get_first_held_row().unwrap_or(1);
            
            // Copy pattern from first held row to current position
            self.sequencer.copy_grid_section(1, first_row, 1, y, 16, 1);
            info!("Copied pattern from row {} to row {}", first_row, y);
        }
        
        // Update grid display
        #[cfg(feature = "hardware")]
        self.update_grid_display()?;
        
        Ok(())
    }


    
    /// Handle CO2-influenced CV output
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
}

fn main() -> Result<()> {
    // Set up direct signal handler that bypasses hardware thread
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();
    
    ctrlc::set_handler(move || {
        info!("🛑 Direct Ctrl+C handler triggered - forcing exit");
        shutdown_flag_clone.store(true, Ordering::SeqCst);
        thread::spawn(|| {
            thread::sleep(Duration::from_millis(100));
            warn!("🛑 Direct force exit");
            std::process::exit(0);
        });
    }).expect("Error setting direct Ctrl-C handler");
    
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
        println!("    --no-hardware        Disable hardware features (simulation mode)");
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
    
    info!("SimonSaysSeeq Rust v{}", env!("CARGO_PKG_VERSION"));
    
    // Create and run application
    let mut app = SimonSaysSeeq::new()?;
    
    // Set main grid preference if specified
    if let Some(grid_id) = main_grid_id {
        app.set_main_grid_preference(grid_id);
    }
    
    app.run()?;
    
    Ok(())
}