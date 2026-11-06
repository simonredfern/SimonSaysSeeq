//! MIDI module - Handle MIDI output and note management
//! 
//! Provides MIDI output capabilities with note tracking and device management.

use anyhow::{Result, anyhow};
use log::{info, debug, warn, trace};
#[cfg(feature = "midi")]
use midir::{MidiInput, MidiOutput, MidiInputConnection, MidiOutputConnection, MidiOutputPort};
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
    SysEx { data: Vec<u8> },
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

/// How long the bound clock port may stay silent before we release it and listen on all inputs again.
/// Matches the existing 5 second "clock stopped" timeout.
#[cfg(feature = "midi")]
const CLOCK_LOSS_TIMEOUT: Duration = Duration::from_secs(5);

/// How often the input port list is refreshed while waiting for a clock (picks up hot-plugged
/// interfaces and an ALSA sequencer that wasn't ready at boot). Nothing is scanned once bound.
#[cfg(feature = "midi")]
const LISTEN_RESCAN_INTERVAL: Duration = Duration::from_secs(1);

/// Which input port plays which role. Written by the listener callbacks (the first port to send
/// clock claims the clock role) and by the main loop (binding, release). Port names are the full
/// ALSA names including client:port numbers, which are unambiguous within one session.
#[derive(Debug, Default)]
struct PortRoles {
    /// Port carrying the MIDI clock: clock in, and gates out on the same device
    clock: Option<String>,
    /// The other interface: keyboard notes and CCs in
    keyboard: Option<String>,
    /// Ports that sent clock but have no matching output port (e.g. a software clock generator's
    /// own source port). Never claimed again this session.
    rejected: Vec<String>,
}

/// What a listener does with a message from its port
#[derive(Debug, PartialEq)]
enum PortRole {
    Clock,
    Keyboard,
    Ignore,
}

impl PortRoles {
    /// Decide what to do with a message (status byte `status`) arriving on `port`. While no clock
    /// is bound, the first port to send clock (F8 tick, FA start, FB continue) claims the clock role.
    fn route(&mut self, port: &str, status: u8) -> PortRole {
        match self.clock.as_deref() {
            Some(clock) if clock == port => PortRole::Clock,
            None if matches!(status, 0xF8 | 0xFA | 0xFB) && !self.rejected.iter().any(|r| r == port) => {
                self.clock = Some(port.to_string());
                PortRole::Clock
            }
            _ if self.keyboard.as_deref() == Some(port) => PortRole::Keyboard,
            _ => PortRole::Ignore,
        }
    }
}

/// MIDI manager handles all MIDI I/O operations
///
/// Port selection: listen on every input until one sends clock, then bind clock in and gates out
/// to that device and use the other interface as the keyboard port. If the clock goes silent for
/// CLOCK_LOSS_TIMEOUT, release and listen on everything again. Nothing is decided once at startup
/// and nothing is saved across boots.
pub struct MidiManager {
    #[cfg(feature = "midi")]
    sequencer_a_output_connection: Option<MidiOutputConnection>,
    /// One input connection per port, keyed by full port name. Callbacks run on midir's threads.
    #[cfg(feature = "midi")]
    listeners: HashMap<String, MidiInputConnection<()>>,
    #[cfg(feature = "midi")]
    port_roles: Arc<Mutex<PortRoles>>,
    /// Port the main loop has finished binding to (output open, keyboard chosen)
    #[cfg(feature = "midi")]
    bound_port: Option<String>,
    #[cfg(feature = "midi")]
    last_listener_refresh: Option<Instant>,
    /// Last port-list problem logged, so a missing ALSA sequencer is logged once, not every second
    #[cfg(feature = "midi")]
    last_listener_error: Option<String>,
    /// Optional config.midi.device: only ports whose name contains this are considered
    #[cfg(feature = "midi")]
    device_filter: String,
    active_notes: Arc<Mutex<HashMap<(u8, u8), ActiveNote>>>, // (note, channel) -> ActiveNote
    last_note_sent: Arc<Mutex<Option<String>>>,
    port_a_midi_clock_in_and_gates_out: String,
    port_b_midi_keyboard_in_and_out: String,
    /// MIDI input event channel
    input_sender: Option<Sender<MidiInputEvent>>,
    input_receiver: Option<Receiver<MidiInputEvent>>,
    /// Clock synchronization
    clock_state: Arc<Mutex<ClockState>>,
    /// Previous clock source to detect transitions
    previous_clock_source: Arc<Mutex<ClockSource>>,
}

impl MidiManager {
    /// Create a new MIDI manager. Never fails because of MIDI: with no ports (or no ALSA sequencer
    /// yet) it starts with nothing bound and keeps listening from the main loop.
    pub fn new(config: &MidiConfig) -> Result<Self> {
        let (input_sender, input_receiver) = unbounded();

        #[cfg_attr(not(feature = "midi"), allow(unused_mut))]
        let mut manager = Self {
            #[cfg(feature = "midi")]
            sequencer_a_output_connection: None,
            #[cfg(feature = "midi")]
            listeners: HashMap::new(),
            #[cfg(feature = "midi")]
            port_roles: Arc::new(Mutex::new(PortRoles::default())),
            #[cfg(feature = "midi")]
            bound_port: None,
            #[cfg(feature = "midi")]
            last_listener_refresh: None,
            #[cfg(feature = "midi")]
            last_listener_error: None,
            #[cfg(feature = "midi")]
            device_filter: config.device.trim().to_string(),
            active_notes: Arc::new(Mutex::new(HashMap::new())),
            last_note_sent: Arc::new(Mutex::new(None)),
            port_a_midi_clock_in_and_gates_out: String::new(),
            port_b_midi_keyboard_in_and_out: String::new(),
            input_sender: Some(input_sender),
            input_receiver: Some(input_receiver),
            clock_state: Arc::new(Mutex::new(ClockState::default())),
            previous_clock_source: Arc::new(Mutex::new(ClockSource::None)),
        };

        #[cfg(feature = "midi")]
        {
            if manager.device_filter.is_empty() {
                info!("MIDI Manager initialized - listening on all inputs for a MIDI clock");
            } else {
                info!("MIDI Manager initialized - listening for a MIDI clock on inputs matching '{}'", manager.device_filter);
            }
            manager.refresh_listeners();
        }
        #[cfg(not(feature = "midi"))]
        {
            let _ = config;
            info!("new says: MIDI simulation mode - no actual MIDI I/O");
        }

        Ok(manager)
    }

    /// Open the gates output on the device with this port name (exact name first, so two identical
    /// interfaces are told apart by their ALSA numbers).
    #[cfg(feature = "midi")]
    fn open_output(&mut self, port_name: &str) -> Result<()> {
        let midi_out = MidiOutput::new("SimonSaysSeeq Sequencer A")?;
        let out_ports = midi_out.ports();
        let port = self.find_port_by_name(&midi_out, &out_ports, port_name)?;
        let connection = midi_out.connect(&port, "SimonSaysSeeq Sequencer A")
            .map_err(|e| anyhow!("Sequencer A MIDI connection to '{}' failed: {}", port_name, e))?;
        self.sequencer_a_output_connection = Some(connection);
        Ok(())
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

    /// Find a MIDI output port by name: exact match first, then case-insensitive substring match
    #[cfg(feature = "midi")]
    fn find_port_by_name(&self, midi_out: &MidiOutput, ports: &[MidiOutputPort], target_name: &str) -> Result<MidiOutputPort> {
        if let Some(port) = ports.iter().find(|p| midi_out.port_name(p).ok().as_deref() == Some(target_name)) {
            return Ok(port.clone());
        }
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
            // MIDI Note On message: [status_byte, note_byte, velocity_byte]
            // 0x90 = Note On for channel 0, OR with channel (0-15) to set actual channel
            // note.min(127) and velocity.min(127) clamp values to valid MIDI range (0-127)
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
                
                debug!("MIDI Note ON: {} ({}), vel: {}, ch: {}", note, note_name, velocity, channel + 1);
            } else {
                warn!("MIDI Note ON (no device): {} vel: {} ch: {}", note, velocity, channel + 1);
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
            
            debug!("MIDI Note ON (no device): {} ({}), vel: {}, ch: {}", note, note_name, velocity, channel + 1);
        }
        
        Ok(())
    }


    
    /// Is this port one of ours (our own output client shows up in the input list)?
    #[cfg(feature = "midi")]
    fn is_own_port(port_name: &str) -> bool {
        port_name.contains("SimonSaysSeeq")
    }

    /// Bring the per-port listeners in line with the current input port list: open a listener on
    /// each new port, drop listeners on ports that have gone. Cheap, and only called while waiting
    /// for a clock (and once at startup), never while bound.
    #[cfg(feature = "midi")]
    fn refresh_listeners(&mut self) {
        self.last_listener_refresh = Some(Instant::now());

        let names: Vec<String> = match MidiInput::new("SimonSaysSeeq Port List") {
            Ok(midi_in) => midi_in.ports().iter()
                .filter_map(|p| midi_in.port_name(p).ok())
                .collect(),
            Err(e) => {
                let msg = format!("MIDI input not available yet ({}), will keep trying", e);
                if self.last_listener_error.as_deref() != Some(msg.as_str()) {
                    warn!("refresh_listeners says: {}", msg);
                    self.last_listener_error = Some(msg);
                }
                return;
            }
        };
        self.last_listener_error = None;

        let filter = self.device_filter.to_lowercase();
        let candidates: Vec<String> = names.into_iter()
            .filter(|n| !Self::is_own_port(n))
            .filter(|n| filter.is_empty() || n.to_lowercase().contains(&filter))
            .collect();

        let before: Vec<String> = self.listeners.keys().cloned().collect();
        self.listeners.retain(|name, _| candidates.contains(name));

        for name in &candidates {
            if self.listeners.contains_key(name) {
                continue;
            }
            match self.open_listener(name) {
                Ok(connection) => {
                    self.listeners.insert(name.clone(), connection);
                }
                Err(e) => debug!("refresh_listeners says: could not listen on '{}': {}", name, e),
            }
        }

        let mut after: Vec<String> = self.listeners.keys().cloned().collect();
        let mut before_sorted = before;
        before_sorted.sort();
        after.sort();
        if after != before_sorted {
            if after.is_empty() {
                warn!("MIDI: no input ports found, waiting for a MIDI interface");
            } else {
                warn!("MIDI: listening for clock on {} input(s): {}", after.len(), after.join(", "));
            }
        }
    }

    /// Open one listener. Its callback routes by role: the clock port gets the full handler, the
    /// keyboard port gets notes/CCs only, and while nothing is bound the first port to send clock
    /// (F8 tick, FA start, FB continue) claims the clock role. That first message is handled too,
    /// so no tick is lost while the main loop opens the output.
    #[cfg(feature = "midi")]
    fn open_listener(&self, port_name: &str) -> Result<MidiInputConnection<()>> {
        let midi_in = MidiInput::new("SimonSaysSeeq Input")?;
        let port = midi_in.ports().into_iter()
            .find(|p| midi_in.port_name(p).ok().as_deref() == Some(port_name))
            .ok_or_else(|| anyhow!("port '{}' disappeared", port_name))?;

        let name = port_name.to_string();
        let sender = self.input_sender.as_ref().unwrap().clone();
        let clock_state = self.clock_state.clone();
        let roles = self.port_roles.clone();

        midi_in.connect(&port, "SimonSaysSeeq Input", move |timestamp, message, _| {
            if message.is_empty() {
                return;
            }
            let role = roles.lock().unwrap().route(&name, message[0]);
            match role {
                PortRole::Clock => Self::handle_midi_input_message(timestamp, message, &sender, &clock_state),
                PortRole::Keyboard => Self::handle_keyboard_input_message(message, &sender),
                PortRole::Ignore => {}
            }
        }, ()).map_err(|e| anyhow!("connect failed: {}", e))
    }

    /// Main-loop housekeeping for port roles. Non-blocking apart from opening/closing ALSA ports,
    /// which happens only when the binding changes (or once a second while nothing is bound).
    #[cfg(feature = "midi")]
    fn service_ports(&mut self) {
        let claimed = self.port_roles.lock().unwrap().clock.clone();

        match (&self.bound_port, claimed) {
            (None, Some(port)) => self.complete_binding(port),
            (Some(bound), Some(port)) if *bound != port => {
                // Only possible if a release raced with a new claim; rebind cleanly.
                self.release_clock("moved to another port");
                self.complete_binding(port);
            }
            (Some(_), Some(_)) => {
                let silent_too_long = self.clock_state.lock().unwrap().last_external_activity
                    .map_or(true, |t| t.elapsed() > CLOCK_LOSS_TIMEOUT);
                if silent_too_long {
                    self.release_clock("lost (no clock for 5 s)");
                }
            }
            (Some(_), None) => self.release_clock("released"),
            (None, None) => {}
        }

        if self.bound_port.is_none()
            && self.last_listener_refresh.map_or(true, |t| t.elapsed() >= LISTEN_RESCAN_INTERVAL)
        {
            self.refresh_listeners();
        }
    }

    /// A listener has claimed the clock role for `port`: open the gates output on the same device
    /// and choose the keyboard port.
    #[cfg(feature = "midi")]
    fn complete_binding(&mut self, port: String) {
        if let Err(e) = self.open_output(&port) {
            warn!("MIDI: '{}' sends clock but has no usable output ({}); ignoring it as a clock source", port, e);
            {
                let mut roles = self.port_roles.lock().unwrap();
                roles.rejected.push(port.clone());
                if roles.clock.as_deref() == Some(port.as_str()) {
                    roles.clock = None;
                }
            }
            *self.clock_state.lock().unwrap() = ClockState::default();
            return;
        }

        let keyboard = self.choose_keyboard_port(&port);
        self.port_roles.lock().unwrap().keyboard = keyboard.clone();
        self.bound_port = Some(port.clone());
        self.port_a_midi_clock_in_and_gates_out = port.clone();
        self.port_b_midi_keyboard_in_and_out = keyboard.clone().unwrap_or_default();

        warn!("MIDI: clock bound to '{}' (clock in and gates out); keyboard in: {}",
              port, keyboard.as_deref().unwrap_or("none"));
    }

    /// Stop using the current clock port: silence any held gates, close the output, and go back
    /// to listening on all inputs.
    #[cfg(feature = "midi")]
    fn release_clock(&mut self, reason: &str) {
        let _ = self.all_notes_off();
        self.sequencer_a_output_connection = None;
        {
            let mut roles = self.port_roles.lock().unwrap();
            roles.clock = None;
            roles.keyboard = None;
        }
        {
            // Source None makes check_clock_source_transition emit ExternalClockTimeout
            let mut clock = self.clock_state.lock().unwrap();
            clock.source = ClockSource::None;
            clock.running = false;
            clock.clock_ticks = 0;
            clock.last_external_activity = None;
        }
        let was = self.bound_port.take().unwrap_or_default();
        self.port_a_midi_clock_in_and_gates_out.clear();
        self.port_b_midi_keyboard_in_and_out.clear();
        self.last_listener_refresh = None; // rescan straight away

        warn!("MIDI: clock on '{}' {}; listening on all inputs", was, reason);
    }

    /// Pick the keyboard port: another hardware input that isn't the clock port, preferring
    /// "USB MIDI Interface" devices. With two identical interfaces this is simply the other one.
    #[cfg(feature = "midi")]
    fn choose_keyboard_port(&self, clock_port: &str) -> Option<String> {
        let mut others: Vec<&String> = self.listeners.keys()
            .filter(|n| n.as_str() != clock_port && !self.is_system_or_virtual_port(n))
            .collect();
        others.sort();
        others.iter()
            .find(|n| n.contains("USB MIDI Interface"))
            .or_else(|| others.first())
            .map(|n| n.to_string())
    }

    /// True once a clock port is bound and the gates output is open
    pub fn is_clock_bound(&self) -> bool {
        #[cfg(feature = "midi")]
        { self.bound_port.is_some() }
        #[cfg(not(feature = "midi"))]
        { false }
    }

    /// Drop the current clock binding (if any) and listen on all inputs again. Non-blocking: the
    /// next port to send clock is bound from the main loop.
    #[cfg(feature = "midi")]
    pub fn force_redetection(&mut self) -> Result<bool> {
        if self.bound_port.is_some() {
            self.release_clock("released by grid re-detect");
        } else {
            self.refresh_listeners();
        }
        Ok(true)
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
            0xF0 => {
                // SysEx message
                if message.len() >= 7 && message[message.len() - 1] == 0xF7 {
                    // Check for SimonSaysSeeQ SysEx: F0 7D 53 53 51 <cmd> F7
                    if message[1] == 0x7D && message[2] == 0x53 && message[3] == 0x53 && message[4] == 0x51 {
                        debug!("handle_midi_input_message says: SimonSaysSeeQ SysEx received: {:02X?}", message);
                        let _ = sender.send(MidiInputEvent::SysEx { data: message.to_vec() });
                    }
                }
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
                
                debug!("MIDI Note OFF: {} ({}), ch: {}", note, note_name, channel + 1);
            } else {
                debug!("MIDI Note OFF (no device): {} ch: {}", note, channel + 1);
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
            
            debug!("MIDI Note OFF: {} ({}), ch: {}", note, note_name, channel + 1);
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
    
    /// Send SysEx message
    pub fn send_sysex(&mut self, data: &[u8]) -> Result<()> {
        #[cfg(feature = "midi")]
        {
            if let Some(ref mut connection) = self.sequencer_a_output_connection {
                connection.send(data)?;
                debug!("send_sysex says: SysEx sent: {:?}", data);
            }
        }
        #[cfg(not(feature = "midi"))]
        debug!("send_sysex says: SysEx (simulation): {:?}", data);
        
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
    
    /// Get MIDI input events (non-blocking). Also binds/releases the clock port, so that a newly
    /// bound port's output is open before its first clock tick is handled.
    pub fn get_input_events(&mut self) -> Vec<MidiInputEvent> {
        let mut events = Vec::new();

        #[cfg(feature = "midi")]
        self.service_ports();
        
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
    fn test_port_roles_first_clock_wins() {
        let mut roles = PortRoles::default();
        // Notes before any clock are ignored and claim nothing
        assert_eq!(roles.route("A 20:0", 0x90), PortRole::Ignore);
        assert_eq!(roles.clock, None);
        // First clock tick claims the clock role
        assert_eq!(roles.route("B 24:0", 0xF8), PortRole::Clock);
        assert_eq!(roles.clock.as_deref(), Some("B 24:0"));
        // A second clock on another port is ignored
        assert_eq!(roles.route("A 20:0", 0xF8), PortRole::Ignore);
        // Everything on the clock port goes to the full handler (sysex, notes)
        assert_eq!(roles.route("B 24:0", 0xF0), PortRole::Clock);
        // Keyboard port gets the keyboard handler once chosen
        roles.keyboard = Some("A 20:0".to_string());
        assert_eq!(roles.route("A 20:0", 0x90), PortRole::Keyboard);
    }

    #[test]
    fn test_port_roles_start_and_continue_claim() {
        let mut roles = PortRoles::default();
        assert_eq!(roles.route("B 24:0", 0xFA), PortRole::Clock);
        let mut roles = PortRoles::default();
        assert_eq!(roles.route("B 24:0", 0xFB), PortRole::Clock);
        // Stop does not claim
        let mut roles = PortRoles::default();
        assert_eq!(roles.route("B 24:0", 0xFC), PortRole::Ignore);
    }

    #[test]
    fn test_port_roles_rejected_port_never_claims() {
        let mut roles = PortRoles::default();
        roles.rejected.push("Fake Clock 128:0".to_string());
        assert_eq!(roles.route("Fake Clock 128:0", 0xF8), PortRole::Ignore);
        assert_eq!(roles.clock, None);
        assert_eq!(roles.route("Midi Through 14:0", 0xF8), PortRole::Clock);
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
        if let Ok(mut midi) = MidiManager::new(&config) {
            // Should start with no events
            let events = midi.get_input_events();
            assert!(events.is_empty());
        }
    }
}