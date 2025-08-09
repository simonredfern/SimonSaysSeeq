//! SimonSaysSeeq Pure Rust Implementation for Norns
//! 
//! A high-performance sequencer application that runs directly on Norns hardware
//! without requiring the Norns Lua environment.

use anyhow::Result;
use crossbeam_channel::Receiver;
use log::{info, warn, error, debug};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

mod hardware;
mod sequencer;
mod midi;
mod grid;
mod screen;
mod config;
mod co2;

use hardware::{NornsHardware, HardwareEvent};
use sequencer::{Sequencer, SequencerEvent};
#[cfg(feature = "midi")]
use midi::MidiManager;
#[cfg(feature = "hardware")]
use grid::GridManager;
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
}

impl SimonSaysSeeq {
    pub fn new() -> Result<Self> {
        let config = Config::load_or_default()?;
        
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
            tempo: 120.0,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting SimonSaysSeeq Rust application");
        
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
        
        // Main event loop
        self.main_loop(hw_rx, seq_rx)?;
        
        // Cleanup
        self.running.store(false, Ordering::SeqCst);
        let _ = hw_thread.join().map_err(|_| anyhow::anyhow!("Hardware thread panicked"))?;
        let _ = seq_thread.join().map_err(|_| anyhow::anyhow!("Sequencer thread panicked"))?;
        
        info!("SimonSaysSeeq shut down successfully");
        Ok(())
    }
    
    fn main_loop(&mut self, hw_rx: Receiver<HardwareEvent>, seq_rx: Receiver<SequencerEvent>) -> Result<()> {
        let mut last_screen_update = Instant::now();
        let screen_update_interval = Duration::from_millis(33); // ~30 FPS
        
        loop {
            // Handle hardware events (non-blocking)
            while let Ok(event) = hw_rx.try_recv() {
                if let Err(e) = self.handle_hardware_event(event, &hw_tx) {
                    error!("Error handling hardware event: {}", e);
                }
            }
            
            // Handle sequencer events (non-blocking)
            while let Ok(event) = seq_rx.try_recv() {
                if let Err(e) = self.handle_sequencer_event(event) {
                    error!("Error handling sequencer event: {}", e);
                }
            }
            
            // Update screen at regular intervals
            if last_screen_update.elapsed() >= screen_update_interval {
                self.update_screen()?;
                last_screen_update = Instant::now();
            }
            
            // Check for shutdown
            if !self.running.load(Ordering::SeqCst) {
                break;
            }
            
            // Small sleep to prevent busy waiting
            thread::sleep(Duration::from_millis(1));
        }
        
        Ok(())
    }
    
    fn handle_hardware_event(&mut self, event: HardwareEvent, hw_sender: &Sender<HardwareEvent>) -> Result<()> {
        match event {
            HardwareEvent::EncoderTurn { encoder, delta } => {
                match encoder {
                    1 => {
                        // Left encoder controls swing
                        let current_swing = self.sequencer.get_swing_amount();
                        let new_swing = (current_swing + delta as f32 * 0.01).clamp(0.0, 0.5);
                        self.sequencer.set_swing(new_swing);
                        info!("Swing changed to: {:.2}", new_swing);
                    }
                    2 => {
                        // Middle encoder controls global transpose
                        let current_transpose = self.sequencer.get_global_transpose();
                        let new_transpose = (current_transpose as i32 + delta).clamp(-24, 24);
                        self.sequencer.set_global_transpose(new_transpose as i8);
                        info!("Global transpose changed to: {} semitones", new_transpose);
                    }
                    3 => {
                        // Right encoder controls tempo
                        self.tempo = (self.tempo + delta as f32).clamp(60.0, 200.0);
                        self.sequencer.set_tempo(self.tempo);
                        info!("Tempo changed to: {:.1} BPM", self.tempo);
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
                                Ok(description) => info!("Undid: {}", description),
                                Err(e) => warn!("Cannot undo: {}", e),
                            }
                        }
                        2 => {
                            // Left key - Stop
                            info!("Stop pressed");
                            self.sequencer.stop();
                            #[cfg(feature = "midi")]
                            self.midi.all_notes_off()?;
                        }
                        3 => {
                            // Right key - Start/Stop toggle
                            if self.sequencer.is_running() {
                                info!("Stop pressed");
                                self.sequencer.stop();
                                #[cfg(feature = "midi")]
                                self.midi.all_notes_off()?;
                            } else {
                                info!("Start pressed");
                                self.sequencer.start();
                            }
                        }
                        _ => {}
                    }
                }
            }
            
            HardwareEvent::StartStopToggle => {
                // Handle start/stop with RGB flash for macropad
                if self.sequencer.is_running() {
                    info!("Stop pressed via macropad");
                    self.sequencer.stop();
                    #[cfg(feature = "midi")]
                    self.midi.all_notes_off()?;
                } else {
                    info!("Start pressed via macropad");
                    self.sequencer.start();
                    
                    // Trigger RGB flash sequence in simulation mode
                    #[cfg(not(feature = "hardware"))]
                    {
                        use crate::hardware::NornsHardware;
                        NornsHardware::execute_flash_sequence(hw_sender);
                    }
                }
            }

            HardwareEvent::GridPress { grid_id, x, y, pressed } => {
                #[cfg(feature = "hardware")]
                self.handle_grid_press(grid_id, x, y, pressed)?;
                #[cfg(not(feature = "hardware"))]
                {
                    let button_name = match (x, y) {
                        (0, 0) => "1", (1, 0) => "2", (2, 0) => "3", (3, 0) => "4",
                        (0, 1) => "Q", (1, 1) => "W", (2, 1) => "E", (3, 1) => "R",
                        (0, 2) => "A", (1, 2) => "S", (2, 2) => "D", (3, 2) => "F",
                        (0, 3) => "Z", (1, 3) => "X", (2, 3) => "C", (3, 3) => "V",
                        _ => "?",
                    };
                    if pressed {
                        info!("🔥 Macropad button {} PRESSED: ({}, {})", button_name, x, y);
                    } else {
                        info!("💨 Macropad button {} released: ({}, {})", button_name, x, y);
                    }
                    
                    // Update grid display
                    if pressed {
                        self.grid.set_led(grid_id, x, y, 15)?;
                    } else {
                        self.grid.set_led(grid_id, x, y, 0)?;
                    }
                }
            }
            
            HardwareEvent::Shutdown => {
                info!("Shutdown requested");
                self.running.store(false, Ordering::SeqCst);
            }
        }
        
        Ok(())
    }
    
    fn handle_sequencer_event(&mut self, event: SequencerEvent) -> Result<()> {
        match event {
            SequencerEvent::Step { step, bar } => {
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
                
                // Update grid display
                #[cfg(feature = "hardware")]
                self.update_grid_display()?;
            }
            
            SequencerEvent::Beat { beat } => {
                // Update any beat-based visual indicators
                #[cfg(feature = "hardware")]
                self.screen.set_beat_indicator(beat);
                #[cfg(not(feature = "hardware"))]
                debug!("Beat indicator: {}", beat);
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
                        info!("MIDI Note ON: {} vel:{} ch:{} step:{}", 
                              midi_event.note, midi_event.velocity, midi_event.channel, midi_event.step);
                    } else {
                        info!("MIDI Note OFF: {} ch:{} step:{}", 
                              midi_event.note, midi_event.channel, midi_event.step);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    #[cfg(feature = "hardware")]
    fn handle_grid_press(&mut self, grid_id: usize, x: usize, y: usize, pressed: bool) -> Result<()> {
        if !pressed {
            return Ok(()); // Only handle press, not release
        }
        
        info!("Grid {} press at ({}, {})", grid_id, x, y);
        
        match grid_id {
    0 => {
        // Grid One - main sequencer grid
        if y <= 7 {
            // Sequence rows (1-7)
            // Check if any positions are held for advanced operations
            if self.has_held_positions() {
                self.handle_advanced_grid_operation(x, y)?;
            } else {
                // Normal grid operation - toggle or cycle ratchet
                let current_value = self.sequencer.get_grid_value(x, y);
                let new_value = match current_value {
                    0 => 1,
                    1 => 2, // Ratchet
                    2 => 4, // Double ratchet
                    _ => 0, // Clear
                };
                        
                self.sequencer.set_grid_value(x, y, new_value);
                #[cfg(feature = "hardware")]
                self.grid.set_led(0, x, y, if new_value > 0 { new_value.min(15) } else { 0 })?;
                        
                info!("Set grid[{}][{}] = {}", x, y, new_value);
            }
        } else {
            // Control row (8)
            self.handle_control_button(x, y)?;
        }
    }
            
    1 => {
        // Grid Two - Mozart MIDI note control
        self.handle_mozart_grid_press(x, y)?;
    }
            
    _ => {
        warn!("Unknown grid ID: {}", grid_id);
    }
}
        
        Ok(())
    }
    
    #[cfg(feature = "hardware")]
    fn handle_control_button(&mut self, x: usize, _y: usize) -> Result<()> {
        // Control buttons on row 8 of grid one
        match x {
            1 => {
                // Reset all
                info!("Reset all sequences");
                self.sequencer.reset_all();
                #[cfg(feature = "hardware")]
                self.grid.clear_all(0)?;
            }
            2 => {
                // Randomize current row or all
                if self.has_held_positions() {
                    info!("Randomize selected rows");
                    let held_rows = self.get_held_rows();
                    self.sequencer.randomize_rows(&held_rows, 0.5, 2);
                } else {
                    info!("Randomize all rows");
                    self.sequencer.randomize_grid();
                }
                #[cfg(feature = "hardware")]
                self.update_grid_display()?;
            }
            3 => {
                // Undo
                match self.sequencer.undo() {
                    Ok(description) => info!("Undid: {}", description),
                    Err(e) => warn!("Cannot undo: {}", e),
                }
                #[cfg(feature = "hardware")]
                self.update_grid_display()?;
            }
            4 => {
                // Redo
                match self.sequencer.redo() {
                    Ok(description) => info!("Redid: {}", description),
                    Err(e) => warn!("Cannot redo: {}", e),
                }
                #[cfg(feature = "hardware")]
                self.update_grid_display()?;
            }
            5 => {
                // Generate Euclidean rhythm
                if let Some(held_row) = self.get_first_held_row() {
                    self.sequencer.generate_euclidean_rhythm(held_row, 5, 16, 0);
                    info!("Generated Euclidean rhythm for row {}", held_row);
                    #[cfg(feature = "hardware")]
                    self.update_grid_display()?;
                }
            }
            6 => {
                // Copy pattern section
                if self.has_held_positions() {
                    info!("Copy selected pattern section");
                    // Implementation would depend on held position logic
                }
            }
            7 => {
                // Scroll pattern
                self.sequencer.scroll_pattern(1, 0); // Scroll right
                info!("Scrolled pattern right");
                #[cfg(feature = "hardware")]
                self.update_grid_display()?;
            }
            8 => {
                // Chain mode toggle
                let chain_enabled = !self.sequencer.is_chain_mode_enabled();
                self.sequencer.set_chain_mode(chain_enabled);
                info!("Chain mode: {}", if chain_enabled { "enabled" } else { "disabled" });
            }
            _ => {}
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
            self.grid.set_led(1, x, y, brightness)?;
            
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
            self.screen.draw_text(1, 7, &format!("Tempo: {:.1}", self.tempo));
            
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
                self.screen.draw_tempo_viz(self.tempo);
            }
            
            self.screen.update()?;
        }
        
        #[cfg(not(feature = "hardware"))]
        {
            // Simulation mode - display info to console
            let (step, bar) = self.sequencer.get_position();
            let transport_text = if self.sequencer.is_running() { "RUNNING" } else { "STOPPED" };
            info!("🎵 Sequencer: {} | Tempo: {:.1} BPM | Step: {} | Bar: {}", transport_text, self.tempo, step, bar);
        }
        
        Ok(())
    }
    
    #[cfg(feature = "hardware")]
    fn update_grid_display(&mut self) -> Result<()> {
        // Update grid one with current sequence state
        for x in 1..=16 {
            for y in 1..=7 {
                let value = self.sequencer.get_grid_value(x, y);
                let brightness = if value > 0 { 5 } else { 0 };
                self.grid.set_led(0, x, y, brightness)?;
            }
        }
        
        self.grid.refresh(0)?;
        Ok(())
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

    fn handle_advanced_grid_operation(&mut self, x: usize, y: usize) -> Result<()> {
        let held_rows = self.get_held_rows();
        
        if held_rows.len() == 1 {
            let held_row = held_rows[0];
            
            match held_row {
                8 => {
                    // Control row held - special functions
                    if x <= 8 {
                        // Randomize column
                        for row in 1..=7 {
                            self.sequencer.randomize_section(x, row);
                        }
                        info!("Randomized column {}", x);
                    } else {
                        // Clear column
                        for row in 1..=7 {
                            self.sequencer.set_grid_value(x, row, 0);
                        }
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
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    // Handle help flag
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        println!("SimonSaysSeeq Rust v{}", env!("CARGO_PKG_VERSION"));
        println!("A high-performance sequencer for Norns hardware\n");
        println!("USAGE:");
        println!("    simon_says_seeq [OPTIONS]\n");
        println!("OPTIONS:");
        println!("    -h, --help       Print help information");
        println!("    -v, --version    Print version information");
        println!("    --config <FILE>  Use custom configuration file");
        println!("    --no-hardware    Disable hardware features (simulation mode)");
        println!("    --no-midi        Disable MIDI features");
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
    app.run()?;
    
    Ok(())
}