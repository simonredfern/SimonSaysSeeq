//! MIDI module - Handle MIDI output and note management
//! 
//! Provides MIDI output capabilities with note tracking and device management.

use anyhow::{Result, anyhow};
use log::{info, debug, warn, error};
#[cfg(feature = "midi")]
use midir::{MidiInput, MidiOutput, MidiInputConnection, MidiOutputConnection, MidiInputPort, MidiOutputPort};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crossbeam_channel::{Sender, Receiver, unbounded};

use crate::config::MidiConfig;

/// MIDI note state for tracking active notes
#[derive(Debug, Clone)]
struct ActiveNote {
    note: u8,
    velocity: u8,
    channel: u8,
    timestamp: Instant,
}

/// MIDI input events
#[derive(Debug, Clone)]
pub enum MidiInputEvent {
    NoteOn { note: u8, velocity: u8, channel: u8 },
    NoteOff { note: u8, channel: u8 },
    ControlChange { controller: u8, value: u8, channel: u8 },
    ClockTick,
    ClockStart,
    ClockStop,
    ClockContinue,
}

/// External clock sync state
#[derive(Debug, Clone)]
pub enum ClockSource {
    Internal,
    MidiExternal,
}

/// Clock sync state
#[derive(Debug, Clone)]
pub struct ClockState {
    pub source: ClockSource,
    pub external_tempo: Option<f32>,
    pub clock_ticks: u32,
    pub last_clock_time: Option<Instant>,
    pub running: bool,
}

impl Default for ClockState {
    fn default() -> Self {
        Self {
            source: ClockSource::Internal,
            external_tempo: None,
            clock_ticks: 0,
            last_clock_time: None,
            running: false,
        }
    }
}

/// MIDI manager handles all MIDI I/O operations
pub struct MidiManager {
    #[cfg(feature = "midi")]
    output_connection: Option<MidiOutputConnection>,
    #[cfg(feature = "midi")]
    input_connection: Option<MidiInputConnection<()>>,
    active_notes: Arc<Mutex<HashMap<(u8, u8), ActiveNote>>>, // (note, channel) -> ActiveNote
    last_note_sent: Arc<Mutex<Option<String>>>,
    device_name: String,
    input_device_name: String,
    /// MIDI input event channel
    input_sender: Option<Sender<MidiInputEvent>>,
    input_receiver: Option<Receiver<MidiInputEvent>>,
    /// Clock synchronization
    clock_state: Arc<Mutex<ClockState>>,
}

impl MidiManager {
    /// Create a new MIDI manager
    pub fn new(config: &MidiConfig) -> Result<Self> {
        let (input_sender, input_receiver) = unbounded();
        
        let mut manager = Self {
            #[cfg(feature = "midi")]
            output_connection: None,
            #[cfg(feature = "midi")]
            input_connection: None,
            active_notes: Arc::new(Mutex::new(HashMap::new())),
            last_note_sent: Arc::new(Mutex::new(None)),
            device_name: config.device.clone(),
            input_device_name: config.device.clone(), // Use same device for input by default
            input_sender: Some(input_sender),
            input_receiver: Some(input_receiver),
            clock_state: Arc::new(Mutex::new(ClockState::default())),
        };
        
        #[cfg(feature = "midi")]
        {
            manager.initialize_output()?;
            manager.initialize_input()?;
        }
        #[cfg(not(feature = "midi"))]
        info!("new says: MIDI simulation mode - no actual MIDI I/O");
        
        Ok(manager)
    }
    
    /// Initialize MIDI output connection
    #[cfg(feature = "midi")]
    fn initialize_output(&mut self) -> Result<()> {
        let midi_out = MidiOutput::new("SimonSaysSeeq")?;
        let out_ports = midi_out.ports();
        
        info!("initialize_output says: Available MIDI output ports:");
        for (i, port) in out_ports.iter().enumerate() {
            if let Ok(name) = midi_out.port_name(port) {
                info!("initialize_output says:   {}: {}", i, name);
            }
        }
        
        // Try to find the configured device, or use the first available
        let selected_port = if !self.device_name.is_empty() {
            self.find_port_by_name(&midi_out, &out_ports, &self.device_name)?
        } else if !out_ports.is_empty() {
            out_ports[0].clone()
        } else {
            warn!("initialize_output says: No MIDI output ports available - MIDI will be disabled");
            return Ok(());
        };
        
        let port_name = midi_out.port_name(&selected_port)
            .unwrap_or_else(|_| "Unknown".to_string());
        
        match midi_out.connect(&selected_port, "SimonSaysSeeq Output") {
            Ok(connection) => {
                info!("initialize_output says: Connected to MIDI output: {}", port_name);
                self.output_connection = Some(connection);
            }
            Err(e) => {
                error!("initialize_output says: Failed to connect to MIDI port {}: {}", port_name, e);
                return Err(anyhow!("MIDI connection failed: {}", e));
            }
        }
        
        Ok(())
    }
    
    /// Find a MIDI port by name (case-insensitive substring match)
    #[cfg(feature = "midi")]
    fn find_port_by_name(&self, midi_out: &MidiOutput, ports: &[MidiOutputPort], target_name: &str) -> Result<MidiOutputPort> {
        let target_lower = target_name.to_lowercase();
        
        for port in ports {
            if let Ok(name) = midi_out.port_name(port) {
                if name.to_lowercase().contains(&target_lower) {
                    return Ok(port.clone());
                }
            }
        }
        
        Err(anyhow!("MIDI port '{}' not found", target_name))
    }
    
    /// Send a MIDI note on message
    pub fn note_on(&mut self, note: u8, velocity: u8, channel: u8) -> Result<()> {
        let channel = channel.saturating_sub(1).min(15); // Convert 1-16 to 0-15
        
        #[cfg(feature = "midi")]
        {
            let msg = [0x90 | channel, note.min(127), velocity.min(127)];
            
            if let Some(ref mut connection) = self.output_connection {
                connection.send(&msg)?;
                
                // Track the active note
                let active_note = ActiveNote {
                    note,
                    velocity,
                    channel: channel + 1, // Store as 1-16
                    timestamp: Instant::now(),
                };
                
                let mut active_notes = self.active_notes.lock().unwrap();
                active_notes.insert((note, channel + 1), active_note);
                
                // Update last note display
                let note_name = midi_note_to_name(note);
                let mut last_note = self.last_note_sent.lock().unwrap();
                *last_note = Some(format!("{} ON vel:{}", note_name, velocity));
                
                debug!("note_on says: MIDI Note ON: {} ({}), vel: {}, ch: {}", note, note_name, velocity, channel + 1);
            } else {
                debug!("note_on says: MIDI Note ON (no device): {} vel: {} ch: {}", note, velocity, channel + 1);
            }
        }
        
        #[cfg(not(feature = "midi"))]
        {
            // Simulation mode - just track and log
            let active_note = ActiveNote {
                note,
                velocity,
                channel: channel + 1,
                timestamp: Instant::now(),
            };
            
            let mut active_notes = self.active_notes.lock().unwrap();
            active_notes.insert((note, channel + 1), active_note);
            
            let note_name = midi_note_to_name(note);
            let mut last_note = self.last_note_sent.lock().unwrap();
            *last_note = Some(format!("{} ON vel:{} (sim)", note_name, velocity));
            
            info!("note_on says: MIDI Note ON (simulation): {} ({}), vel: {}, ch: {}", note, note_name, velocity, channel + 1);
        }
        
        Ok(())
    }
    
    /// Initialize MIDI input connection
    #[cfg(feature = "midi")]
    fn initialize_input(&mut self) -> Result<()> {
        let midi_in = MidiInput::new("SimonSaysSeeq Input")?;
        let in_ports = midi_in.ports();
        
        info!("initialize_input says: Available MIDI input ports:");
        for (i, port) in in_ports.iter().enumerate() {
            if let Ok(name) = midi_in.port_name(port) {
                info!("initialize_input says:   {}: {}", i, name);
            }
        }
        
        // Try to find the configured device, or use the first available
        let selected_port = if !self.input_device_name.is_empty() {
            self.find_input_port_by_name(&midi_in, &in_ports, &self.input_device_name)?
        } else if !in_ports.is_empty() {
            in_ports[0].clone()
        } else {
            warn!("initialize_input says: No MIDI input ports available - MIDI input will be disabled");
            return Ok(());
        };
        
        let port_name = midi_in.port_name(&selected_port)
            .unwrap_or_else(|_| "Unknown".to_string());
        
        // Set up input callback
        let sender = self.input_sender.as_ref().unwrap().clone();
        let clock_state = self.clock_state.clone();
        
        match midi_in.connect(&selected_port, "SimonSaysSeeq Input", move |timestamp, message, _| {
            Self::handle_midi_input_message(timestamp, message, &sender, &clock_state);
        }, ()) {
            Ok(connection) => {
                info!("initialize_input says: Connected to MIDI input: {}", port_name);
                self.input_connection = Some(connection);
            }
            Err(e) => {
                error!("initialize_input says: Failed to connect to MIDI input port {}: {}", port_name, e);
                return Err(anyhow!("MIDI input connection failed: {}", e));
            }
        }
        
        Ok(())
    }
    
    /// Find a MIDI input port by name (case-insensitive substring match)
    #[cfg(feature = "midi")]
    fn find_input_port_by_name(&self, midi_in: &MidiInput, ports: &[MidiInputPort], target_name: &str) -> Result<MidiInputPort> {
        let target_lower = target_name.to_lowercase();
        
        for port in ports {
            if let Ok(name) = midi_in.port_name(port) {
                if name.to_lowercase().contains(&target_lower) {
                    return Ok(port.clone());
                }
            }
        }
        
        Err(anyhow!("MIDI input port '{}' not found", target_name))
    }
    
    /// Handle incoming MIDI message
    #[cfg(feature = "midi")]
    fn handle_midi_input_message(_timestamp: u64, message: &[u8], sender: &Sender<MidiInputEvent>, clock_state: &Arc<Mutex<ClockState>>) {
        if message.is_empty() {
            return;
        }
        
        match message[0] {
            // Note On (0x90-0x9F)
            0x90..=0x9F if message.len() >= 3 => {
                let channel = (message[0] & 0x0F) + 1; // Convert to 1-16
                let note = message[1];
                let velocity = message[2];
                
                if velocity > 0 {
                    let _ = sender.send(MidiInputEvent::NoteOn { note, velocity, channel });
                } else {
                    // Velocity 0 note-on is equivalent to note-off
                    let _ = sender.send(MidiInputEvent::NoteOff { note, channel });
                }
            }
            // Note Off (0x80-0x8F)
            0x80..=0x8F if message.len() >= 3 => {
                let channel = (message[0] & 0x0F) + 1; // Convert to 1-16
                let note = message[1];
                let _ = sender.send(MidiInputEvent::NoteOff { note, channel });
            }
            // Control Change (0xB0-0xBF)
            0xB0..=0xBF if message.len() >= 3 => {
                let channel = (message[0] & 0x0F) + 1; // Convert to 1-16
                let controller = message[1];
                let value = message[2];
                let _ = sender.send(MidiInputEvent::ControlChange { controller, value, channel });
            }
            // System Real-Time Messages
            0xF8 => {
                // MIDI Clock
                let mut clock = clock_state.lock().unwrap();
                clock.clock_ticks = clock.clock_ticks.wrapping_add(1);
                
                // Calculate tempo from clock ticks
                if let Some(last_time) = clock.last_clock_time {
                    let now = Instant::now();
                    let time_diff = now.duration_since(last_time).as_secs_f32();
                    
                    // MIDI clock sends 24 ticks per quarter note
                    // BPM = 60 / (time_per_tick * 24)
                    if time_diff > 0.0 {
                        let ticks_per_minute = 60.0 / time_diff;
                        let bpm = ticks_per_minute / 24.0;
                        
                        // Filter out unreasonable tempos
                        if bpm >= 60.0 && bpm <= 200.0 {
                            clock.external_tempo = Some(bpm);
                        }
                    }
                }
                clock.last_clock_time = Some(Instant::now());
                
                let _ = sender.send(MidiInputEvent::ClockTick);
            }
            0xFA => {
                // MIDI Start
                let mut clock = clock_state.lock().unwrap();
                clock.running = true;
                clock.clock_ticks = 0;
                let _ = sender.send(MidiInputEvent::ClockStart);
            }
            0xFB => {
                // MIDI Continue
                let mut clock = clock_state.lock().unwrap();
                clock.running = true;
                let _ = sender.send(MidiInputEvent::ClockContinue);
            }
            0xFC => {
                // MIDI Stop
                let mut clock = clock_state.lock().unwrap();
                clock.running = false;
                let _ = sender.send(MidiInputEvent::ClockStop);
            }
            _ => {
                // Ignore other messages
                debug!("handle_midi_input_message says: Unhandled MIDI message: {:02X?}", message);
            }
        }
    }
    
    /// Send a MIDI note on message
    pub fn note_off(&mut self, note: u8, channel: u8) -> Result<()> {
        let channel = channel.saturating_sub(1).min(15); // Convert 1-16 to 0-15
        
        #[cfg(feature = "midi")]
        {
            let msg = [0x80 | channel, note.min(127), 0];
            
            if let Some(ref mut connection) = self.output_connection {
                connection.send(&msg)?;
                
                // Remove from active notes
                let mut active_notes = self.active_notes.lock().unwrap();
                active_notes.remove(&(note, channel + 1));
                
                // Update last note display
                let note_name = midi_note_to_name(note);
                let mut last_note = self.last_note_sent.lock().unwrap();
                *last_note = Some(format!("{} OFF", note_name));
                
                debug!("note_off says: MIDI Note OFF: {} ({}), ch: {}", note, note_name, channel + 1);
            } else {
                debug!("note_off says: MIDI Note OFF (no device): {} ch: {}", note, channel + 1);
            }
        }
        
        #[cfg(not(feature = "midi"))]
        {
            // Simulation mode
            let mut active_notes = self.active_notes.lock().unwrap();
            active_notes.remove(&(note, channel + 1));
            
            let note_name = midi_note_to_name(note);
            let mut last_note = self.last_note_sent.lock().unwrap();
            *last_note = Some(format!("{} OFF (sim)", note_name));
            
            info!("note_off says: MIDI Note OFF (simulation): {} ({}), ch: {}", note, note_name, channel + 1);
        }
        
        Ok(())
    }
    
    /// Send all notes off (panic button)
    pub fn all_notes_off(&mut self) -> Result<()> {
        info!("all_notes_off says: Sending All Notes Off");
        
        // Send note off for all currently active notes
        let active_notes = {
            let notes = self.active_notes.lock().unwrap();
            notes.clone()
        };
        
        for ((note, channel), _) in active_notes {
            self.note_off(note, channel)?;
        }
        
        // Also send CC 123 (All Notes Off) on all channels
        #[cfg(feature = "midi")]
        {
            if let Some(ref mut connection) = self.output_connection {
                for channel in 0..16 {
                    let msg = [0xB0 | channel, 123, 0]; // CC 123 = All Notes Off
                    let _ = connection.send(&msg);
                }
            }
        }
        
        // Clear our tracking
        let mut active_notes = self.active_notes.lock().unwrap();
        active_notes.clear();
        
        Ok(())
    }
    
    /// Send MIDI clock start
    pub fn send_clock_start(&mut self) -> Result<()> {
        #[cfg(feature = "midi")]
        {
            if let Some(ref mut connection) = self.output_connection {
                let msg = [0xFA]; // MIDI Clock Start
                connection.send(&msg)?;
                debug!("send_clock_start says: MIDI Clock Start sent");
            }
        }
        #[cfg(not(feature = "midi"))]
        info!("send_clock_start says: MIDI Clock Start (simulation)");
        
        Ok(())
    }
    
    /// Send MIDI clock stop
    pub fn send_clock_stop(&mut self) -> Result<()> {
        #[cfg(feature = "midi")]
        {
            if let Some(ref mut connection) = self.output_connection {
                let msg = [0xFC]; // MIDI Clock Stop
                connection.send(&msg)?;
                debug!("send_clock_stop says: MIDI Clock Stop sent");
            }
        }
        #[cfg(not(feature = "midi"))]
        info!("send_clock_stop says: MIDI Clock Stop (simulation)");
        
        Ok(())
    }
    
    /// Send MIDI clock tick
    pub fn send_clock_tick(&mut self) -> Result<()> {
        #[cfg(feature = "midi")]
        {
            if let Some(ref mut connection) = self.output_connection {
                let msg = [0xF8]; // MIDI Clock Tick
                connection.send(&msg)?;
            }
        }
        Ok(())
    }
    
    /// Set clock source
    pub fn set_clock_source(&self, source: ClockSource) {
        let mut clock = self.clock_state.lock().unwrap();
        clock.source = source;
        info!("set_clock_source says: Clock source set to: {:?}", clock.source);
    }
    
    /// Get clock source
    pub fn get_clock_source(&self) -> ClockSource {
        let clock = self.clock_state.lock().unwrap();
        clock.source.clone()
    }
    
    /// Get external tempo (if available)
    pub fn get_external_tempo(&self) -> Option<f32> {
        let clock = self.clock_state.lock().unwrap();
        clock.external_tempo
    }
    
    /// Check if external clock is running
    pub fn is_external_clock_running(&self) -> bool {
        let clock = self.clock_state.lock().unwrap();
        matches!(clock.source, ClockSource::MidiExternal) && clock.running
    }
    
    /// Get MIDI input events (non-blocking)
    pub fn get_input_events(&self) -> Vec<MidiInputEvent> {
        let mut events = Vec::new();
        
        if let Some(ref receiver) = self.input_receiver {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }
        
        events
    }
    
    /// Reset clock state
    pub fn reset_clock(&self) {
        let mut clock = self.clock_state.lock().unwrap();
        clock.clock_ticks = 0;
        clock.last_clock_time = None;
        clock.external_tempo = None;
        debug!("reset_clock says: Clock state reset");
    }
    
    /// Get the last note sent (for display purposes)
    pub fn get_last_note(&self) -> Option<String> {
        let last_note = self.last_note_sent.lock().unwrap();
        last_note.clone()
    }
    
    /// Get count of active notes
    pub fn get_active_note_count(&self) -> usize {
        let active_notes = self.active_notes.lock().unwrap();
        active_notes.len()
    }
    
    /// Get active notes (for display)
    pub fn get_active_notes(&self) -> Vec<(u8, u8, u8)> { // (note, velocity, channel)
        let active_notes = self.active_notes.lock().unwrap();
        active_notes.values()
            .map(|note| (note.note, note.velocity, note.channel))
            .collect()
    }
    
    /// Clean up stuck notes (notes that have been on too long)
    pub fn cleanup_stuck_notes(&mut self, max_duration: Duration) -> Result<()> {
        let now = Instant::now();
        let stuck_notes: Vec<(u8, u8)> = {
            let active_notes = self.active_notes.lock().unwrap();
            active_notes.iter()
                .filter_map(|(&(note, channel), active_note)| {
                    if now.duration_since(active_note.timestamp) > max_duration {
                        Some((note, channel))
                    } else {
                        None
                    }
                })
                .collect()
        };
        
        for (note, channel) in stuck_notes {
            warn!("cleanup_stuck_notes says: Cleaning up stuck note: {} on channel {}", note, channel);
            self.note_off(note, channel)?;
        }
        
        Ok(())
    }
    
    /// Test MIDI output with a short note
    pub fn test_output(&mut self) -> Result<()> {
        info!("test_output says: Testing MIDI output...");
        
        // Send a middle C note for 100ms
        self.note_on(60, 100, 1)?;
        std::thread::sleep(Duration::from_millis(100));
        self.note_off(60, 1)?;
        
        info!("test_output says: MIDI test completed");
        Ok(())
    }
    
    /// Get MIDI device status
    pub fn get_device_status(&self) -> String {
        #[cfg(feature = "midi")]
        {
            if self.output_connection.is_some() {
                format!("Connected: {}", self.device_name)
            } else {
                "No MIDI device".to_string()
            }
        }
        #[cfg(not(feature = "midi"))]
        {
            "MIDI simulation mode".to_string()
        }
    }
    
    /// Get clock state information
    pub fn get_clock_info(&self) -> (ClockSource, Option<f32>, bool) {
        let clock = self.clock_state.lock().unwrap();
        (clock.source.clone(), clock.external_tempo, clock.running)
    }
}

/// Convert MIDI note number to note name
pub fn midi_note_to_name(note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i8 - 1;
    let note_index = (note % 12) as usize;
    
    format!("{}{}", note_names[note_index], octave)
}

/// Convert note name to MIDI note number
pub fn note_name_to_midi(name: &str) -> Option<u8> {
    if name.len() < 2 {
        return None;
    }
    
    let note_part = &name[..name.len()-1];
    let octave_part = &name[name.len()-1..];
    
    let note_value = match note_part {
        "C" => 0, "C#" => 1, "Db" => 1,
        "D" => 2, "D#" => 3, "Eb" => 3,
        "E" => 4,
        "F" => 5, "F#" => 6, "Gb" => 6,
        "G" => 7, "G#" => 8, "Ab" => 8,
        "A" => 9, "A#" => 10, "Bb" => 10,
        "B" => 11,
        _ => return None,
    };
    
    if let Ok(octave) = octave_part.parse::<i8>() {
        let midi_note = (octave + 1) * 12 + note_value;
        if midi_note <= 127 {
            Some(midi_note as u8)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_note_name_conversion() {
        assert_eq!(midi_note_to_name(60), "C4");
        assert_eq!(midi_note_to_name(61), "C#4");
        assert_eq!(midi_note_to_name(72), "C5");
        assert_eq!(midi_note_to_name(0), "C-1");
        assert_eq!(midi_note_to_name(127), "G9");
    }
    
    #[test]
    fn test_note_name_parsing() {
        assert_eq!(note_name_to_midi("C4"), Some(60));
        assert_eq!(note_name_to_midi("C#4"), Some(61));
        assert_eq!(note_name_to_midi("C5"), Some(72));
        assert_eq!(note_name_to_midi("A4"), Some(69));
        assert_eq!(note_name_to_midi("G9"), Some(127));
        
        // Invalid cases
        assert_eq!(note_name_to_midi("H4"), None);
        assert_eq!(note_name_to_midi("C"), None);
        assert_eq!(note_name_to_midi("C10"), None); // Out of range
    }
    
    #[test]
    fn test_channel_conversion() {
        // Test that channel 1-16 gets converted to 0-15 internally
        // This would require access to internal state, so we test indirectly
        // by ensuring no panics with various channel values
        
        let config = MidiConfig::default();
        if let Ok(mut midi) = MidiManager::new(&config) {
            // These should not panic
            let _ = midi.note_on(60, 100, 1);
            let _ = midi.note_on(60, 100, 16);
            let _ = midi.note_on(60, 100, 0); // Should clamp to 1
            let _ = midi.note_on(60, 100, 255); // Should clamp to 16
        }
    }
    
    #[test]
    fn test_clock_source() {
        let config = MidiConfig::default();
        if let Ok(midi) = MidiManager::new(&config) {
            // Test default internal clock
            assert!(matches!(midi.get_clock_source(), ClockSource::Internal));
            
            // Test setting external clock
            midi.set_clock_source(ClockSource::MidiExternal);
            assert!(matches!(midi.get_clock_source(), ClockSource::MidiExternal));
        }
    }
    
    #[test]
    fn test_input_events() {
        let config = MidiConfig::default();
        if let Ok(midi) = MidiManager::new(&config) {
            // Should start with no events
            let events = midi.get_input_events();
            assert!(events.is_empty());
        }
    }
}