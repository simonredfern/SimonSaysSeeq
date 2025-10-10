//! MIDI module - Handle MIDI output and note management
//! 
//! Provides MIDI output capabilities with note tracking and device management.

use anyhow::{Result, anyhow};
use log::{info, debug, warn, error, trace};
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
    ClockBeat,
    ClockStart,
    ClockStop,
    ClockContinue,
    ExternalClockTimeout,
}

/// External clock sync state
#[derive(Debug, Clone, PartialEq)]
pub enum ClockSource {
    None,
    MidiExternal,
}

/// Multi-scale tick windows for tempo calculation
#[derive(Debug, Clone)]
pub struct MultiScaleTickWindows {
    pub tick_timestamps: Vec<Instant>, // Store all tick timestamps
    pub window_durations: Vec<f32>, // 1, 2, 4, 8, 16, 32 seconds
}

impl MultiScaleTickWindows {
    pub fn new() -> Self {
        Self {
            tick_timestamps: Vec::new(),
            window_durations: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
        }
    }
    
    pub fn add_tick(&mut self, current_time: Instant) {
        self.tick_timestamps.push(current_time);
        
        // Remove ticks older than 12 seconds to prevent unbounded growth
        let cutoff_time = current_time - std::time::Duration::from_secs(12);
        self.tick_timestamps.retain(|&timestamp| timestamp >= cutoff_time);
    }
    
    pub fn calculate_weighted_bpm(&self, current_time: Instant) -> Option<f32> {
        if self.tick_timestamps.is_empty() {
            return None;
        }
        
        let earliest_tick = self.tick_timestamps[0];
        let elapsed_since_start = current_time.duration_since(earliest_tick).as_secs_f32();
        
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        
        for &duration in &self.window_durations {
            // Only use windows that are "full" - where we have been collecting data for the full duration
            if elapsed_since_start >= duration {
                let window_start = current_time - std::time::Duration::from_secs_f32(duration);
                
                // Count ticks within this specific window
                let ticks_in_window = self.tick_timestamps.iter()
                    .filter(|&&timestamp| timestamp >= window_start && timestamp <= current_time)
                    .count() as f32;
                
                if ticks_in_window > 0.0 {
                    // BPM = (ticks / duration_seconds) / 24_ticks_per_beat * 60_seconds_per_minute
                    // Simplified: BPM = ticks * (60 / (duration * 24)) = ticks * (2.5 / duration)
                    let bpm = ticks_in_window * (2.5 / duration);
                    if bpm >= 20.0 && bpm <= 300.0 {
                        let weight = duration; // Use duration as weight (longer windows = more weight)
                        weighted_sum += bpm * weight;
                        total_weight += weight;
                    }
                }
            }
        }
        
        if total_weight > 0.0 {
            Some(weighted_sum / total_weight)
        } else {
            None
        }
    }
}

/// Clock sync state
#[derive(Debug, Clone)]
pub struct ClockState {
    pub source: ClockSource,
    pub clock_ticks: u32,
    pub last_clock_time: Option<Instant>,
    pub last_beat_time: Option<Instant>,
    pub multi_scale_window: Option<MultiScaleTickWindows>,
    pub running: bool,
    pub last_external_activity: Option<Instant>,
}

impl Default for ClockState {
    fn default() -> Self {
        Self {
            source: ClockSource::None,
            clock_ticks: 0,
            last_clock_time: None,
            last_beat_time: None,
            multi_scale_window: None,
            running: false,
            last_external_activity: None,
        }
    }
}

/// MIDI manager handles all MIDI I/O operations
pub struct MidiManager {
    #[cfg(feature = "midi")]
    sequencer_a_output_connection: Option<MidiOutputConnection>,
    #[cfg(feature = "midi")]
    input_connection: Option<MidiInputConnection<()>>,
    #[cfg(feature = "midi")]
    keyboard_input_connection: Option<MidiInputConnection<()>>,
    active_notes: Arc<Mutex<HashMap<(u8, u8), ActiveNote>>>, // (note, channel) -> ActiveNote
    last_note_sent: Arc<Mutex<Option<String>>>,
    device_name: String,
    port_a_midi_clock_in_and_gates_out: String,
    port_b_midi_keyboard_in_and_out: String,
    /// MIDI input event channel
    input_sender: Option<Sender<MidiInputEvent>>,
    input_receiver: Option<Receiver<MidiInputEvent>>,
    /// Clock synchronization
    clock_state: Arc<Mutex<ClockState>>,
    /// Previous clock source to detect transitions
    previous_clock_source: Arc<Mutex<ClockSource>>,
    /// Auto-detect MIDI clock sources
    auto_detect_clock: bool,
    /// Last successful auto-detection time
    last_detection_time: Arc<Mutex<Option<Instant>>>,
    /// Auto-detection retry interval (seconds)
    detection_retry_interval: u64,
    /// Configuration reference for saving detected devices
    config: Arc<Mutex<MidiConfig>>,
}

impl MidiManager {
    /// Create a new MIDI manager
    pub fn new(config: &MidiConfig) -> Result<Self> {
        let (input_sender, input_receiver) = unbounded();
        
        let mut manager = Self {
            #[cfg(feature = "midi")]
            sequencer_a_output_connection: None,
            #[cfg(feature = "midi")]
            input_connection: None,
            #[cfg(feature = "midi")]
            keyboard_input_connection: None,
            active_notes: Arc::new(Mutex::new(HashMap::new())),
            last_note_sent: Arc::new(Mutex::new(None)),
            device_name: config.device.clone(),
            port_a_midi_clock_in_and_gates_out: config.device.clone(), // Use same device for clock by default
            port_b_midi_keyboard_in_and_out: config.device.clone(), // Use same device for keyboard by default
            input_sender: Some(input_sender),
            input_receiver: Some(input_receiver),
            clock_state: Arc::new(Mutex::new(ClockState::default())),
            previous_clock_source: Arc::new(Mutex::new(ClockSource::None)),
            auto_detect_clock: config.auto_detect_clock,
            last_detection_time: Arc::new(Mutex::new(None)),
            detection_retry_interval: config.detection_retry_interval,
            config: Arc::new(Mutex::new(config.clone())),
        };
        
        #[cfg(feature = "midi")]
        {
            info!("MIDI Manager initialized - auto_detect_clock: {}, device: '{}'", manager.auto_detect_clock, manager.device_name);
            let sequencer_a_port = if manager.auto_detect_clock {
                info!("Starting MIDI auto-detection flow...");
                manager.auto_detect_and_connect()?;
                manager.initialize_output()?
            } else {
                info!("Using manual MIDI configuration (auto-detect disabled)...");
                let port = manager.initialize_output()?;
                manager.initialize_input()?;
                manager.initialize_keyboard_input()?;
                port
            };
            info!("MIDI Manager connected: PORT_A={}", sequencer_a_port);
            
            // Always print final port assignments regardless of initialization mode
            info!("════════════════════════════════════════════════════════");
            info!("FINAL MIDI PORT ASSIGNMENTS:");
            info!("   PORT_A_MIDI_CLOCK_IN_AND_GATES_OUT:  {}", manager.port_a_midi_clock_in_and_gates_out);
            info!("   PORT_B_MIDI_KEYBOARD_IN_AND_OUT:     {}", manager.port_b_midi_keyboard_in_and_out);
            info!("════════════════════════════════════════════════════════");
        }
        #[cfg(not(feature = "midi"))]
        info!("new says: MIDI simulation mode - no actual MIDI I/O");
        
        Ok(manager)
    }
    
    /// Initialize dual MIDI output connections for sequencer A and B
    #[cfg(feature = "midi")]
    fn initialize_output(&mut self) -> Result<String> {
        let midi_out_a = MidiOutput::new("SimonSaysSeeq Sequencer A")?;
        let out_ports_a = midi_out_a.ports();
        
        info!("initialize_output says: Setting up MIDI output connection");
        info!("PORT_A (Sequencer A): {}", self.port_a_midi_clock_in_and_gates_out);
        
        // Initialize Sequencer A output (PORT_A)
        let sequencer_a_port_name = if !self.port_a_midi_clock_in_and_gates_out.is_empty() {
            let port_a = self.find_port_by_name(&midi_out_a, &out_ports_a, &self.port_a_midi_clock_in_and_gates_out)?;
            match midi_out_a.connect(&port_a, "SimonSaysSeeq Sequencer A") {
                Ok(connection) => {
                    info!("Sequencer A connected to PORT_A: {}", self.port_a_midi_clock_in_and_gates_out);
                    self.sequencer_a_output_connection = Some(connection);
                    self.port_a_midi_clock_in_and_gates_out.clone()
                }
                Err(e) => {
                    error!("Failed to connect Sequencer A to PORT_A {}: {}", self.port_a_midi_clock_in_and_gates_out, e);
                    return Err(anyhow!("Sequencer A MIDI connection failed: {}", e));
                }
            }
        } else {
            warn!("PORT_A not configured for Sequencer A");
            "No PORT_A configured".to_string()
        };
        
        info!("MIDI output connection established: Sequencer A → {}", sequencer_a_port_name);
        
        Ok(sequencer_a_port_name)
    }
    
    /// Check if a MIDI port is a system or virtual port (should be filtered out)
    #[cfg(feature = "midi")]
    fn is_system_or_virtual_port(&self, port_name: &str) -> bool {
        let system_patterns = [
            "Midi Through",           // Linux ALSA system through port
            "SimonSaysSeeq Output",   // Our own virtual output
            "SimonSaysSeeq:",         // Any SimonSaysSeeq virtual ports
            "Through:",               // Generic through ports
            "Virtual",                // Virtual devices
            "Client-",                // ALSA client ports
            "System:",                // System ports
        ];
        
        for pattern in &system_patterns {
            if port_name.contains(pattern) {
                return true;
            }
        }
        
        false
    }

    /// Extract USB interface name from MIDI port name (e.g., "28:0" from "USB MIDI Interface 28:0")
    #[cfg(feature = "midi")]
    fn extract_usb_interface_name(&self, port_name: &str) -> String {
        // Look for pattern like "USB MIDI Interface 28:0" or "Interface 24:0"
        if let Some(interface_pos) = port_name.find("Interface ") {
            let after_interface = &port_name[interface_pos + "Interface ".len()..];
            // Extract the part that looks like "XX:Y" (digits:digits)
            for word in after_interface.split_whitespace() {
                if word.contains(':') && word.chars().all(|c| c.is_ascii_digit() || c == ':') {
                    return word.to_string();
                }
            }
        }
        
        // Fallback: return the full port name if pattern not found
        port_name.to_string()
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
    
    /// Send a MIDI note on message for sequencer A
    pub fn sequencer_a_note_on(&mut self, note: u8, velocity: u8, channel: u8) -> Result<()> {
        let channel = channel.saturating_sub(1).min(15); // Convert 1-16 to 0-15
        
        #[cfg(feature = "midi")]
        {
            let msg = [0x90 | channel, note.min(127), velocity.min(127)];
            
            if let Some(ref mut connection) = self.sequencer_a_output_connection {
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
                
                let note_name = midi_note_to_name(note);
                let mut last_note = self.last_note_sent.lock().unwrap();
                *last_note = Some(format!("{} ON vel:{}", note_name, velocity));
                
                trace!("MIDI Note ON: {} ({}), vel: {}, ch: {}", note, note_name, velocity, channel + 1);
            } else {
                trace!("MIDI Note ON (no device): {} vel: {} ch: {}", note, velocity, channel + 1);
            }
        }
        
        #[cfg(not(feature = "midi"))]
        {
            // Track the active note even without MIDI
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
            
            trace!("MIDI Note ON (no device): {} ({}), vel: {}, ch: {}", note, note_name, velocity, channel + 1);
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
                let interface_name = self.extract_usb_interface_name(&name);
                info!("initialize_input says:   {}: {} → interface: '{}'", i, name, interface_name);
            }
        }
        
        // Filter out system and virtual ports to get only real hardware devices
        let hardware_ports: Vec<_> = in_ports.iter().filter(|port| {
            if let Ok(name) = midi_in.port_name(port) {
                !self.is_system_or_virtual_port(&name)
            } else {
                false
            }
        }).collect();
        
        // Try to find the configured device, or use the first available hardware device
        let selected_port = if !self.port_a_midi_clock_in_and_gates_out.is_empty() {
            // When port_a_midi_clock_in_and_gates_out is set (from clock detection), use it directly
            self.find_input_port_by_name(&midi_in, &in_ports, &self.port_a_midi_clock_in_and_gates_out)?
        } else if !hardware_ports.is_empty() {
            hardware_ports[0].clone()
        } else {
            // No hardware ports - try Midi Through as fallback for testing with automated_test_clock
            info!("initialize_input says: No hardware MIDI input ports found - checking for Midi Through port for testing");
            match self.find_input_port_by_name(&midi_in, &in_ports, "Midi Through") {
                Ok(port) => {
                    info!("initialize_input says: Using Midi Through port for testing/automated test clock");
                    port
                }
                Err(_) => {
                    warn!("initialize_input says: No hardware MIDI input ports or Midi Through available - MIDI input will be disabled");
                    return Ok(());
                }
            }
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
                if self.auto_detect_clock && !self.port_a_midi_clock_in_and_gates_out.is_empty() {
                    info!("PORT_A_MIDI_CLOCK_IN_AND_GATES_OUT: {} (for tempo sync)", port_name);
                } else {
                    info!("PORT_B_MIDI_KEYBOARD_IN_AND_OUT: {} (for note input)", port_name);
                }
                self.input_connection = Some(connection);
            }
            Err(e) => {
                error!("initialize_input says: Failed to connect to MIDI input port {}: {}", port_name, e);
                return Err(anyhow!("MIDI input connection failed: {}", e));
            }
        }
        
        Ok(())
    }

    /// Initialize MIDI keyboard input connection (separate from clock input)
    #[cfg(feature = "midi")]
    fn initialize_keyboard_input(&mut self) -> Result<()> {
        if self.port_b_midi_keyboard_in_and_out.is_empty() {
            warn!("initialize_keyboard_input says: No keyboard port configured");
            return Ok(());
        }

        let midi_in = MidiInput::new("SimonSaysSeeq Keyboard Input")?;
        let in_ports = midi_in.ports();
        
        info!("initialize_keyboard_input says: Setting up keyboard input on: {}", self.port_b_midi_keyboard_in_and_out);
        
        let selected_port = self.find_input_port_by_name(&midi_in, &in_ports, &self.port_b_midi_keyboard_in_and_out)?;
        let port_name = midi_in.port_name(&selected_port)
            .unwrap_or_else(|_| "Unknown".to_string());
        
        // Set up keyboard input callback (only for note/CC messages, not clock)
        let sender = self.input_sender.as_ref().unwrap().clone();
        
        match midi_in.connect(&selected_port, "SimonSaysSeeq Keyboard Input", move |_timestamp, message, _| {
            Self::handle_keyboard_input_message(message, &sender);
        }, ()) {
            Ok(connection) => {
                info!("initialize_keyboard_input says: Connected to keyboard port: {}", port_name);
                self.keyboard_input_connection = Some(connection);
            }
            Err(e) => {
                error!("initialize_keyboard_input says: Failed to connect to keyboard port {}: {}", port_name, e);
                return Err(anyhow!("Keyboard MIDI input connection failed: {}", e));
            }
        }
        
        Ok(())
    }

    /// Handle keyboard-only MIDI input messages (notes, CC, but not clock)
    #[cfg(feature = "midi")]
    fn handle_keyboard_input_message(message: &[u8], sender: &Sender<MidiInputEvent>) {
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
                    trace!("PORT_B_MIDI_KEYBOARD_IN_AND_OUT: Note On - note={} velocity={} channel={}", note, velocity, channel);
                    let _ = sender.send(MidiInputEvent::NoteOn { note, velocity, channel });
                } else {
                    // Velocity 0 note-on is equivalent to note-off
                    trace!("PORT_B_MIDI_KEYBOARD_IN_AND_OUT: Note Off (vel=0) - note={} channel={}", note, channel);
                    let _ = sender.send(MidiInputEvent::NoteOff { note, channel });
                }
            }
            // Note Off (0x80-0x8F)
            0x80..=0x8F if message.len() >= 3 => {
                let channel = (message[0] & 0x0F) + 1; // Convert to 1-16
                let note = message[1];
                trace!("PORT_B_MIDI_KEYBOARD_IN_AND_OUT: Note Off - note={} channel={}", note, channel);
                let _ = sender.send(MidiInputEvent::NoteOff { note, channel });
            }
            // Control Change (0xB0-0xBF)
            0xB0..=0xBF if message.len() >= 3 => {
                let channel = (message[0] & 0x0F) + 1; // Convert to 1-16
                let controller = message[1];
                let value = message[2];
                let _ = sender.send(MidiInputEvent::ControlChange { controller, value, channel });
            }
            // Ignore clock and other system messages on keyboard input
            _ => {
                // Don't log every ignored message to avoid spam
            }
        }
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
                    trace!("PORT_B_MIDI_KEYBOARD_IN_AND_OUT: Note On - note={} velocity={} channel={}", note, velocity, channel);
                    let _ = sender.send(MidiInputEvent::NoteOn { note, velocity, channel });
                } else {
                    // Velocity 0 note-on is equivalent to note-off
                    trace!("PORT_B_MIDI_KEYBOARD_IN_AND_OUT: Note Off (vel=0) - note={} channel={}", note, channel);
                    let _ = sender.send(MidiInputEvent::NoteOff { note, channel });
                }
            }
            // Note Off (0x80-0x8F)
            0x80..=0x8F if message.len() >= 3 => {
                let channel = (message[0] & 0x0F) + 1; // Convert to 1-16
                let note = message[1];
                trace!("PORT_B_MIDI_KEYBOARD_IN_AND_OUT: Note Off - note={} channel={}", note, channel);
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
                
                // Set to external clock source on first clock tick
                if !matches!(clock.source, ClockSource::MidiExternal) {
                    clock.source = ClockSource::MidiExternal;
                    info!("handle_midi_input_message says: MIDI Clock detected - switching to external clock");
                }
                
                // Update external activity timestamp
                let now = Instant::now();
                clock.last_external_activity = Some(now);
                
                clock.clock_ticks = clock.clock_ticks.wrapping_add(1);
                clock.last_clock_time = Some(now);
                
                // Multi-scale tempo detection using 1,2,4,8,16,32 second windows
                // Initialize window if needed
                if clock.multi_scale_window.is_none() {
                    clock.multi_scale_window = Some(MultiScaleTickWindows::new());
                }
                
                // Update the window with this tick and calculate tempo
                let tempo_result = if let Some(ref mut window) = clock.multi_scale_window {
                    window.add_tick(now);
                    
                    // Calculate weighted BPM and get window info
                    if let Some(weighted_bpm) = window.calculate_weighted_bpm(now) {
                        // Count how many windows are actually active (full)
                        let earliest_tick = window.tick_timestamps.first().map(|&t| t).unwrap_or(now);
                        let elapsed_since_start = now.duration_since(earliest_tick).as_secs_f32();
                        let active_windows = window.window_durations.iter()
                            .filter(|&&duration| elapsed_since_start >= duration)
                            .count();
                        let total_windows = window.window_durations.len();
                        
                        Some((weighted_bpm, active_windows, total_windows))
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                // Process tempo result outside of window borrow
                if let Some((weighted_bpm, active_windows, total_windows)) = tempo_result {
                    let total_ticks = clock.clock_ticks;
                    
                    // Tempo calculation for logging only - not used by sequencer
                    trace!("handle_midi_input_message says: External tempo detected: {:.1} BPM (weighted from {}/{} active windows, {} total ticks)", 
                           weighted_bpm, active_windows, total_windows, total_ticks);
                }
                
                // Send beat event every 24 ticks for compatibility
                if clock.clock_ticks % 24 == 0 {
                    clock.last_beat_time = Some(now);
                    let _ = sender.send(MidiInputEvent::ClockBeat);
                }
                
                let _ = sender.send(MidiInputEvent::ClockTick);
            }
            0xFA => {
                // MIDI Start
                let mut clock = clock_state.lock().unwrap();
                clock.source = ClockSource::MidiExternal; // Set to external clock
                clock.running = true;
                clock.clock_ticks = 0;
                clock.last_external_activity = Some(Instant::now());
                info!("handle_midi_input_message says: MIDI Start - switching to external clock");
                let _ = sender.send(MidiInputEvent::ClockStart);
            }
            0xFB => {
                // MIDI Continue
                let mut clock = clock_state.lock().unwrap();
                clock.source = ClockSource::MidiExternal; // Set to external clock
                clock.running = true;
                clock.last_external_activity = Some(Instant::now());
                info!("handle_midi_input_message says: MIDI Continue - switching to external clock");
                let _ = sender.send(MidiInputEvent::ClockContinue);
            }
            0xFC => {
                // MIDI Stop
                let mut clock = clock_state.lock().unwrap();
                clock.running = false;
                clock.last_external_activity = Some(Instant::now());
                // Keep external clock source - just stop running
                info!("handle_midi_input_message says: MIDI Stop - external clock stopped");
                let _ = sender.send(MidiInputEvent::ClockStop);
            }
            _ => {
                // Ignore other messages
                debug!("handle_midi_input_message says: Unhandled MIDI message: {:02X?}", message);
            }
        }
    }
    
    /// Send a MIDI note off message for sequencer A
    pub fn sequencer_a_note_off(&mut self, note: u8, channel: u8) -> Result<()> {
        let channel = channel.saturating_sub(1).min(15); // Convert 1-16 to 0-15
        
        #[cfg(feature = "midi")]
        {
            let msg = [0x80 | channel, note.min(127), 0];
            
            if let Some(ref mut connection) = self.sequencer_a_output_connection {
                connection.send(&msg)?;
                
                // Remove from active notes
                let mut active_notes = self.active_notes.lock().unwrap();
                active_notes.remove(&(note, channel + 1));
                
                // Update last note display
                let note_name = midi_note_to_name(note);
                let mut last_note = self.last_note_sent.lock().unwrap();
                *last_note = Some(format!("{} OFF", note_name));
                
                trace!("MIDI Note OFF: {} ({}), ch: {}", note, note_name, channel + 1);
            } else {
                trace!("MIDI Note OFF (no device): {} ch: {}", note, channel + 1);
            }
        }
        
        #[cfg(not(feature = "midi"))]
        {
            // No MIDI device mode
            let mut active_notes = self.active_notes.lock().unwrap();
            active_notes.remove(&(note, channel + 1));
            
            let note_name = midi_note_to_name(note);
            let mut last_note = self.last_note_sent.lock().unwrap();
            *last_note = Some(format!("{} OFF", note_name));
            
            trace!("MIDI Note OFF: {} ({}), ch: {}", note, note_name, channel + 1);
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
            // Send note off for sequencer A
            let _ = self.sequencer_a_note_off(note, channel);
        }
        
        // Also send CC 123 (All Notes Off) on all channels
        #[cfg(feature = "midi")]
        {
            // Send all notes off to sequencer A
            if let Some(ref mut connection) = self.sequencer_a_output_connection {
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
            if let Some(ref mut connection) = self.sequencer_a_output_connection {
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
            if let Some(ref mut connection) = self.sequencer_a_output_connection {
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
            if let Some(ref mut connection) = self.sequencer_a_output_connection {
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
    

    
    /// Check if external clock is running
    pub fn is_external_clock_running(&self) -> bool {
        let clock = self.clock_state.lock().unwrap();
        matches!(clock.source, ClockSource::MidiExternal) && clock.running
    }
    
    /// Get MIDI input events (non-blocking)
    pub fn get_input_events(&self) -> Vec<MidiInputEvent> {
        let mut events = Vec::new();
        
        // Check for external clock timeout (5 seconds without activity)
        self.check_external_clock_timeout();
        
        // Check for clock source transitions
        if let Some(transition_event) = self.check_clock_source_transition() {
            events.push(transition_event);
        }
        
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
        clock.last_beat_time = None;
        clock.multi_scale_window = None;
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
            let _ = self.sequencer_a_note_off(note, channel);
        }
        
        Ok(())
    }
    
    /// Test MIDI output with a short note
    pub fn test_output(&mut self) -> Result<()> {
        info!("test_output says: Testing MIDI output...");
        
        // Send a middle C note for 100ms on sequencer A
        self.sequencer_a_note_on(60, 100, 1)?;
        std::thread::sleep(Duration::from_millis(100));
        self.sequencer_a_note_off(60, 1)?;
        
        info!("test_output says: MIDI test completed");
        Ok(())
    }
    
    /// Get MIDI device status
    pub fn get_device_status(&self) -> String {
        #[cfg(feature = "midi")]
        {
            let a_connected = self.sequencer_a_output_connection.is_some();
            if a_connected {
                format!("Connected: A:{}", self.port_a_midi_clock_in_and_gates_out)
            } else {
                "No MIDI devices".to_string()
            }
        }
        #[cfg(not(feature = "midi"))]
        {
            "MIDI simulation mode".to_string()
        }
    }
    
    /// Get clock state information
    pub fn get_clock_info(&self) -> (ClockSource, bool) {
        let clock = self.clock_state.lock().unwrap();
        (clock.source.clone(), clock.running)
    }
    
    /// Get clock state for phase correction
    pub fn get_clock_state(&self) -> Option<ClockState> {
        if let Ok(clock) = self.clock_state.try_lock() {
            Some(clock.clone())
        } else {
            None
        }
    }
    
    /// Check if external clock has timed out and stop sequencer
    fn check_external_clock_timeout(&self) {
        let mut clock = self.clock_state.lock().unwrap();
        
        if matches!(clock.source, ClockSource::MidiExternal) {
            if let Some(last_activity) = clock.last_external_activity {
                if last_activity.elapsed() > Duration::from_secs(5) {
                    // Only log if clock was running (prevents spam)
                    if clock.running {
                        info!("check_external_clock_timeout says: External MIDI clock timeout - sequencer stopped. Waiting for clock...");
                    }
                    // Don't switch to internal clock (doesn't exist) - just mark as not running
                    clock.running = false;
                }
            }
        }
    }
    
    
    /// Check for clock source transitions and generate appropriate events
    fn check_clock_source_transition(&self) -> Option<MidiInputEvent> {
        let current_source = {
            let clock = self.clock_state.lock().unwrap();
            clock.source.clone()
        };
        
        let mut previous_source = self.previous_clock_source.lock().unwrap();
        
        if *previous_source != current_source {
            // Clock source has changed
            let transition_event = match (&*previous_source, &current_source) {
                (ClockSource::MidiExternal, ClockSource::None) => {
                    // External clock timeout - just stop, no internal clock to switch to
                    Some(MidiInputEvent::ExternalClockTimeout)
                }
                _ => None
            };
            
            // Update previous source
            *previous_source = current_source;
            
            transition_event
        } else {
            None
        }
    }

    /// Auto-detect and connect to the first available MIDI clock source
    #[cfg(feature = "midi")]
    fn auto_detect_and_connect(&mut self) -> Result<()> {
        use crate::midi_scanner::{scan_for_midi_clock};
        
        info!("auto_detect_and_connect says: Starting automatic MIDI clock detection...");
        
        // First try the last known good device if available
        let last_detected = self.config.lock().unwrap().last_detected_device.clone();
        if let Some(ref last_device) = last_detected {
            info!("auto_detect_and_connect says: Trying last known device: {}", last_device);
            self.port_a_midi_clock_in_and_gates_out = last_device.clone();
            match self.initialize_input() {
                Ok(_) => {
                    // Test if this device actually provides clock
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let is_external_clock = {
                        let clock_state = self.clock_state.lock().unwrap();
                        matches!(clock_state.source, ClockSource::MidiExternal)
                    };
                    
                    if is_external_clock {
                        info!("auto_detect_and_connect says: Successfully reconnected to last known device");
                        // Set up keyboard input/output on the OTHER port
                        self.port_b_midi_keyboard_in_and_out = self.find_other_usb_midi_device(last_device)?;
                        self.initialize_keyboard_input()?;
                        let sequencer_a_port = self.initialize_output()?;
                        info!("════════════════════════════════════════════════════════");
                        info!("MIDI PORT ASSIGNMENTS COMPLETE (RECONNECTED):");
                        info!("   PORT_A_MIDI_CLOCK_IN_AND_GATES_OUT:  {} (sequencer tempo sync)", last_device);
                        info!("   PORT_A_MIDI_CLOCK_IN_AND_GATES_OUT:  {} (pattern playback)", sequencer_a_port);
                        info!("   PORT_B_MIDI_KEYBOARD_IN_AND_OUT:     {} (note input)", self.port_b_midi_keyboard_in_and_out);
                        info!("════════════════════════════════════════════════════════");
                        return Ok(());
                    } else {
                        info!("auto_detect_and_connect says: Last known device no longer provides clock, scanning for new sources");
                    }
                }
                Err(e) => {
                    warn!("auto_detect_and_connect says: Failed to connect to last known device: {}", e);
                }
            }
        }
        
        // Fall back to full scanning
        info!("Starting full MIDI clock scan...");
        match scan_for_midi_clock() {
            Ok(summary) => {
                info!("Clock scan completed - reliable sources: {:?}, selected: {:?}", 
                     summary.reliable_sources, summary.selected_source);
                if let Some(selected_source) = summary.selected_source {
                    info!("auto_detect_and_connect says: Found reliable clock source: {}", selected_source);
                    self.port_a_midi_clock_in_and_gates_out = selected_source.clone();
                    // Set keyboard input to OTHER port (not the clock port)
                    self.port_b_midi_keyboard_in_and_out = self.find_other_usb_midi_device(&selected_source)?;
                    self.initialize_input()?;
                    self.initialize_keyboard_input()?;
                    // Re-initialize output for sequencer
                    let sequencer_a_port = self.initialize_output()?;
                    self.save_detected_device(&selected_source)?;
                    info!("════════════════════════════════════════════════════════");
                    info!("MIDI PORT ASSIGNMENTS COMPLETE (AUTO-DETECTED):");
                    info!("   PORT_A_MIDI_CLOCK_IN_AND_GATES_OUT:  {} (sequencer tempo sync)", selected_source);
                    info!("   PORT_A_MIDI_CLOCK_IN_AND_GATES_OUT:  {} (pattern playback)", sequencer_a_port);
                    info!("   PORT_B_MIDI_KEYBOARD_IN_AND_OUT:     {} (note input)", self.port_b_midi_keyboard_in_and_out);
                    info!("════════════════════════════════════════════════════════");
                } else if !summary.reliable_sources.is_empty() {
                    // Use the first reliable source if none was auto-selected
                    let first_source = summary.reliable_sources[0].clone();
                    info!("auto_detect_and_connect says: Using first reliable source: {}", first_source);
                    self.port_a_midi_clock_in_and_gates_out = first_source.clone();
                    // Set keyboard input to OTHER port (not the clock port)
                    self.port_b_midi_keyboard_in_and_out = self.find_other_usb_midi_device(&first_source)?;
                    self.initialize_input()?;
                    self.initialize_keyboard_input()?;
                    // Re-initialize output for sequencer
                    let sequencer_a_port = self.initialize_output()?;
                    self.save_detected_device(&first_source)?;
                    info!("════════════════════════════════════════════════════════");
                    info!("MIDI PORT ASSIGNMENTS COMPLETE (FIRST RELIABLE):");
                    info!("   PORT_A_MIDI_CLOCK_IN_AND_GATES_OUT:  {} (sequencer tempo sync)", first_source);
                    info!("   PORT_A_MIDI_CLOCK_IN_AND_GATES_OUT:  {} (pattern playback)", sequencer_a_port);
                    info!("   PORT_B_MIDI_KEYBOARD_IN_AND_OUT:     {} (note input)", self.port_b_midi_keyboard_in_and_out);
                    info!("════════════════════════════════════════════════════════");
                } else {
                    warn!("auto_detect_and_connect says: No reliable MIDI clock sources found, falling back to first available port");
                    self.initialize_input()?;
                    self.initialize_keyboard_input()?;
                    let sequencer_a_port = self.initialize_output()?;
                    info!("════════════════════════════════════════════════════════");
                    info!("MIDI PORT ASSIGNMENTS COMPLETE (FALLBACK MODE):");
                    info!("   PORT_A_MIDI_CLOCK_IN_AND_GATES_OUT:  {} (pattern playback)", sequencer_a_port);
                    info!("   PORT_B_MIDI_KEYBOARD_IN_AND_OUT:     {} (note input - no clock detected)", self.port_b_midi_keyboard_in_and_out);
                    info!("════════════════════════════════════════════════════════");
                }
            }
            Err(e) => {
                warn!("auto_detect_and_connect says: Clock detection failed: {}, falling back to normal initialization", e);
                self.initialize_input()?;
                self.initialize_keyboard_input()?;
                let sequencer_a_port = self.initialize_output()?;
                info!("════════════════════════════════════════════════════════");
                info!("MIDI PORT ASSIGNMENTS COMPLETE (ERROR FALLBACK):");
                info!("   PORT_A_MIDI_CLOCK_IN_AND_GATES_OUT:  {} (pattern playback)", sequencer_a_port);
                info!("   PORT_B_MIDI_KEYBOARD_IN_AND_OUT:     {} (note input - clock detection failed)", self.port_b_midi_keyboard_in_and_out);
                info!("════════════════════════════════════════════════════════");
            }
        }
        
        Ok(())
    }

    /// Find the other USB MIDI device that's not the specified device
    #[cfg(feature = "midi")]
    fn find_other_usb_midi_device(&self, exclude_device: &str) -> Result<String> {
        info!("find_other_usb_midi_device: Looking for other device, excluding: '{}'", exclude_device);
        let midi_in = MidiInput::new("Device Finder")?;
        let in_ports = midi_in.ports();
        
        // Find all hardware USB MIDI devices
        let usb_devices: Vec<String> = in_ports.iter()
            .filter_map(|port| {
                if let Ok(name) = midi_in.port_name(port) {
                    if name.contains("USB MIDI Interface") && !self.is_system_or_virtual_port(&name) {
                        info!("find_other_usb_midi_device: Found USB device: '{}'", name);
                        Some(name)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        info!("find_other_usb_midi_device: Found {} USB devices total", usb_devices.len());

        // Find a device that's NOT the excluded device
        for device in &usb_devices {
            if device != exclude_device {
                info!("find_other_usb_midi_device: Selected other device: '{}'", device);
                return Ok(device.clone());
            }
        }

        // Fallback: if we can't find another device, use the same device
        warn!("find_other_usb_midi_device: Could not find other USB MIDI device, using same device for keyboard");
        Ok(exclude_device.to_string())
    }

    /// Schedule a re-detection attempt
    fn schedule_redetection(&self) {
        *self.last_detection_time.lock().unwrap() = Some(Instant::now());
    }

    /// Check if we should attempt re-detection
    pub fn should_retry_detection(&self) -> bool {
        if !self.auto_detect_clock {
            return false;
        }

        let clock = self.clock_state.lock().unwrap();
        let last_detection = self.last_detection_time.lock().unwrap();

        // Only retry if we have no clock source and enough time has passed
        if matches!(clock.source, ClockSource::None) {
            if let Some(last_time) = *last_detection {
                last_time.elapsed().as_secs() >= self.detection_retry_interval
            } else {
                true // Never attempted detection
            }
        } else {
            false // We have external clock, no need to retry
        }
    }

    /// Attempt to re-detect MIDI clock sources
    #[cfg(feature = "midi")]
    pub fn retry_detection(&mut self) -> Result<bool> {
        if !self.should_retry_detection() {
            return Ok(false);
        }

        info!("retry_detection says: Attempting to re-detect MIDI clock sources...");
        
        // Disconnect current input if any
        #[cfg(feature = "midi")]
        {
            self.input_connection = None;
        }

        // Run detection
        match self.auto_detect_and_connect() {
            Ok(_) => {
                *self.last_detection_time.lock().unwrap() = Some(Instant::now());
                info!("retry_detection says: Re-detection completed successfully");
                Ok(true)
            }
            Err(e) => {
                warn!("retry_detection says: Re-detection failed: {}", e);
                *self.last_detection_time.lock().unwrap() = Some(Instant::now());
                Ok(false)
            }
        }
    }

    /// Force re-detection even if clock is currently active
    #[cfg(feature = "midi")]
    pub fn force_redetection(&mut self) -> Result<bool> {
        info!("force_redetection says: Forcing MIDI clock re-detection (ignoring current state)");
        
        // Disconnect current input
        #[cfg(feature = "midi")]
        {
            self.input_connection = None;
        }

        // Reset clock state to internal to ensure fresh detection
        {
            let mut clock = self.clock_state.lock().unwrap();
            clock.source = ClockSource::None;
            clock.running = false;
            clock.clock_ticks = 0;
            clock.last_external_activity = None;
        }

        // Run detection
        match self.auto_detect_and_connect() {
            Ok(_) => {
                *self.last_detection_time.lock().unwrap() = Some(Instant::now());
                info!("force_redetection says: Forced re-detection completed successfully");
                Ok(true)
            }
            Err(e) => {
                warn!("force_redetection says: Forced re-detection failed: {}", e);
                *self.last_detection_time.lock().unwrap() = Some(Instant::now());
                Ok(false)
            }
        }
    }

    /// Get detection status information
    pub fn get_detection_status(&self) -> DetectionStatus {
        let clock = self.clock_state.lock().unwrap();
        let last_detection = self.last_detection_time.lock().unwrap();
        let should_retry = self.should_retry_detection();

        DetectionStatus {
            auto_detect_enabled: self.auto_detect_clock,
            current_source: clock.source.clone(),
            last_detection_time: *last_detection,
            next_retry_in: if should_retry {
                Some(0)
            } else if let Some(last_time) = *last_detection {
                let elapsed = last_time.elapsed().as_secs();
                if elapsed < self.detection_retry_interval {
                    Some(self.detection_retry_interval - elapsed)
                } else {
                    Some(0)
                }
            } else {
                None
            },
            device_name: self.port_a_midi_clock_in_and_gates_out.clone(),
        }
    }

    /// Save the detected MIDI device to configuration for future use
    fn save_detected_device(&self, device_name: &str) -> Result<()> {
        {
            let mut config = self.config.lock().unwrap();
            config.last_detected_device = Some(device_name.to_string());
        }
        
        // Save to file
        let config_path = crate::config::Config::get_config_path();
        if let Ok(mut full_config) = crate::config::Config::load_or_default() {
            full_config.midi.last_detected_device = Some(device_name.to_string());
            if let Err(e) = full_config.save(&config_path) {
                warn!("save_detected_device says: Failed to save config: {}", e);
            } else {
                info!("save_detected_device says: Saved detected device '{}' to config", device_name);
            }
        }
        
        Ok(())
    }

    /// Clear the saved detected device from configuration
    pub fn clear_detected_device(&self) -> Result<()> {
        {
            let mut config = self.config.lock().unwrap();
            config.last_detected_device = None;
        }
        
        let config_path = crate::config::Config::get_config_path();
        if let Ok(mut full_config) = crate::config::Config::load_or_default() {
            full_config.midi.last_detected_device = None;
            if let Err(e) = full_config.save(&config_path) {
                warn!("clear_detected_device says: Failed to save config: {}", e);
            } else {
                info!("clear_detected_device says: Cleared saved detected device from config");
            }
        }
        
        Ok(())
    }

    /// Get current clock connection health
    pub fn get_clock_health(&self) -> ClockHealth {
        let clock = self.clock_state.lock().unwrap();
        
        match clock.source {
            ClockSource::None => ClockHealth::NoExternalClock,
            ClockSource::MidiExternal => {
                if let Some(last_activity) = clock.last_external_activity {
                    let silence_duration = last_activity.elapsed().as_millis();
                    if silence_duration > 2000 {
                        ClockHealth::Lost
                    } else if silence_duration > 500 {
                        ClockHealth::Unstable
                    } else {
                        ClockHealth::Healthy
                    }
                } else {
                    ClockHealth::Unknown
                }
            }
        }
    }
}

/// Status information for MIDI clock detection
#[derive(Debug, Clone)]
pub struct DetectionStatus {
    pub auto_detect_enabled: bool,
    pub current_source: ClockSource,
    pub last_detection_time: Option<Instant>,
    pub next_retry_in: Option<u64>, // seconds
    pub device_name: String,
}

/// Health status of MIDI clock connection
#[derive(Debug, Clone, PartialEq)]
pub enum ClockHealth {
    /// External clock is working well
    Healthy,
    /// External clock detected but unstable
    Unstable,
    /// External clock connection lost
    Lost,
    /// No external clock source
    NoExternalClock,
    /// Clock health unknown
    Unknown,
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
            let _ = midi.sequencer_a_note_on(60, 100, 1);
            let _ = midi.sequencer_a_note_on(60, 100, 16);
            let _ = midi.sequencer_a_note_on(60, 100, 0); // Should clamp to 1
            let _ = midi.sequencer_a_note_on(60, 100, 255); // Should clamp to 16
        }
    }
    
    #[test]
    fn test_clock_source() {
        let config = MidiConfig::default();
        if let Ok(midi) = MidiManager::new(&config) {
            // Test default (no clock source)
            assert!(matches!(midi.get_clock_source(), ClockSource::None));
            
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