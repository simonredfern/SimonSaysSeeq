/// MIDI Clock Scanner Module
/// 
/// This module provides functionality to automatically scan all available MIDI input ports
/// and detect which ones are providing MIDI clock signals. It will lock onto the first
/// reliable MIDI clock source found.

use anyhow::{anyhow, Result};
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, error, info, warn};
use midir::{MidiInput, MidiInputPort};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Minimum number of clock ticks required to consider a source reliable
const MIN_CLOCK_TICKS_FOR_RELIABILITY: u32 = 24; // 1 beat at 24 PPQ

/// Maximum time to wait without clock ticks before considering a source dead
const CLOCK_TIMEOUT_MS: u64 = 2000;

/// Time to scan each port for clock signals
const SCAN_DURATION_MS: u64 = 3000;

/// Clock detection events sent from scanner threads
#[derive(Debug, Clone)]
pub enum ClockScanEvent {
    /// Clock tick detected on a specific port
    ClockTick { port_name: String, timestamp: Instant },
    /// Clock start detected on a specific port
    ClockStart { port_name: String },
    /// Clock stop detected on a specific port  
    ClockStop { port_name: String },
    /// Port scan completed (with or without finding clock)
    ScanComplete { port_name: String, had_clock: bool },
    /// Scanner error occurred
    ScanError { port_name: String, error: String },
}

/// Statistics for a MIDI clock source
#[derive(Debug, Clone)]
pub struct ClockSourceStats {
    pub port_name: String,
    pub first_tick_time: Instant,
    pub last_tick_time: Instant,
    pub total_ticks: u32,
    pub estimated_bpm: Option<f32>,
    pub is_reliable: bool,
}

impl ClockSourceStats {
    pub fn new(port_name: String) -> Self {
        let now = Instant::now();
        Self {
            port_name,
            first_tick_time: now,
            last_tick_time: now,
            total_ticks: 0,
            estimated_bpm: None,
            is_reliable: false,
        }
    }

    pub fn add_tick(&mut self, timestamp: Instant) {
        self.total_ticks += 1;
        self.last_tick_time = timestamp;

        // Calculate BPM after we have enough ticks
        if self.total_ticks >= MIN_CLOCK_TICKS_FOR_RELIABILITY {
            let elapsed = timestamp.duration_since(self.first_tick_time).as_secs_f32();
            if elapsed > 0.0 {
                // 24 MIDI clock ticks = 1 quarter note
                let beats = self.total_ticks as f32 / 24.0;
                let minutes = elapsed / 60.0;
                self.estimated_bpm = Some(beats / minutes);
                self.is_reliable = true;
            }
        }
    }

    pub fn is_alive(&self) -> bool {
        self.last_tick_time.elapsed().as_millis() < CLOCK_TIMEOUT_MS as u128
    }
}

/// MIDI Clock Scanner
pub struct MidiClockScanner {
    /// Channel for receiving scan events
    event_receiver: Receiver<ClockScanEvent>,
    /// Statistics for detected clock sources
    clock_sources: Arc<Mutex<HashMap<String, ClockSourceStats>>>,
    /// Currently selected clock source
    selected_source: Arc<Mutex<Option<String>>>,
    /// Flag to indicate if scanning is active
    is_scanning: Arc<Mutex<bool>>,
}

impl MidiClockScanner {
    pub fn new() -> Self {
        let (_sender, receiver) = unbounded();
        
        Self {
            event_receiver: receiver,
            clock_sources: Arc::new(Mutex::new(HashMap::new())),
            selected_source: Arc::new(Mutex::new(None)),
            is_scanning: Arc::new(Mutex::new(false)),
        }
    }

    /// Start scanning all available MIDI input ports for clock signals
    pub fn start_scan(&self) -> Result<()> {
        {
            let mut is_scanning = self.is_scanning.lock().unwrap();
            if *is_scanning {
                return Err(anyhow!("Scan already in progress"));
            }
            *is_scanning = true;
        }

        info!("Starting MIDI clock scan...");
        
        // Clear previous results
        self.clock_sources.lock().unwrap().clear();
        *self.selected_source.lock().unwrap() = None;

        // Get all available MIDI input ports
        let midi_in = MidiInput::new("MIDI Clock Scanner")?;
        let ports = midi_in.ports();

        if ports.is_empty() {
            warn!("No MIDI input ports available for scanning");
            *self.is_scanning.lock().unwrap() = false;
            return Ok(());
        }

        info!("Found {} MIDI input ports to scan", ports.len());

        // Start scanner threads for each port
        let (sender, _) = unbounded(); // We'll use the instance receiver
        for (i, port) in ports.iter().enumerate() {
            let port_name = midi_in.port_name(port)
                .unwrap_or_else(|_| format!("Port {}", i));
            
            info!("Starting scan thread for port: {}", port_name);
            
            let port_clone = port.clone();
            let port_name_clone = port_name.clone();
            let sender_clone = sender.clone();
            let clock_sources = self.clock_sources.clone();
            let selected_source = self.selected_source.clone();

            thread::spawn(move || {
                Self::scan_port_for_clock(
                    port_clone, 
                    port_name_clone, 
                    sender_clone,
                    clock_sources,
                    selected_source,
                );
            });
        }

        Ok(())
    }

    /// Scan a specific port for MIDI clock signals
    fn scan_port_for_clock(
        port: MidiInputPort,
        port_name: String,
        sender: Sender<ClockScanEvent>,
        clock_sources: Arc<Mutex<HashMap<String, ClockSourceStats>>>,
        selected_source: Arc<Mutex<Option<String>>>,
    ) {
        let midi_in = match MidiInput::new(&format!("Scanner-{}", port_name)) {
            Ok(midi_in) => midi_in,
            Err(e) => {
                error!("Failed to create MIDI input for {}: {}", port_name, e);
                let _ = sender.send(ClockScanEvent::ScanError {
                    port_name: port_name.clone(),
                    error: format!("Failed to create MIDI input: {}", e),
                });
                return;
            }
        };

        // Set up statistics tracking
        {
            let mut sources = clock_sources.lock().unwrap();
            sources.insert(port_name.clone(), ClockSourceStats::new(port_name.clone()));
        }

        let port_name_for_callback = port_name.clone();
        let sender_for_callback = sender.clone();
        let clock_sources_for_callback = clock_sources.clone();
        let selected_source_for_callback = selected_source.clone();

        // Connect to the port with callback
        let connection_result = midi_in.connect(&port, &format!("Scanner-{}", port_name), 
            move |timestamp, message, _| {
                Self::handle_scan_midi_message(
                    timestamp,
                    message,
                    &port_name_for_callback,
                    &sender_for_callback,
                    &clock_sources_for_callback,
                    &selected_source_for_callback,
                );
            }, ()
        );

        let _connection = match connection_result {
            Ok(connection) => {
                debug!("Connected to {} for scanning", port_name);
                connection
            }
            Err(e) => {
                error!("Failed to connect to {}: {}", port_name, e);
                let _ = sender.send(ClockScanEvent::ScanError {
                    port_name: port_name.clone(),
                    error: format!("Connection failed: {}", e),
                });
                return;
            }
        };

        // Wait for scan duration
        thread::sleep(Duration::from_millis(SCAN_DURATION_MS));

        // Check if this port had a reliable clock
        let had_clock = {
            let sources = clock_sources.lock().unwrap();
            sources.get(&port_name)
                .map(|stats| stats.is_reliable)
                .unwrap_or(false)
        };

        debug!("Scan completed for {} - had_clock: {}", port_name, had_clock);
        let _ = sender.send(ClockScanEvent::ScanComplete {
            port_name: port_name.clone(),
            had_clock,
        });

        // Connection automatically closes when it goes out of scope
    }

    /// Handle MIDI messages during port scanning
    fn handle_scan_midi_message(
        _timestamp: u64,
        message: &[u8],
        port_name: &str,
        sender: &Sender<ClockScanEvent>,
        clock_sources: &Arc<Mutex<HashMap<String, ClockSourceStats>>>,
        selected_source: &Arc<Mutex<Option<String>>>,
    ) {
        if message.is_empty() {
            return;
        }

        let now = Instant::now();

        match message[0] {
            0xF8 => {
                // MIDI Clock tick
                debug!("Clock tick detected on {}", port_name);
                
                // Update statistics
                {
                    let mut sources = clock_sources.lock().unwrap();
                    if let Some(stats) = sources.get_mut(port_name) {
                        stats.add_tick(now);
                        
                        // If this is the first reliable source we found, select it
                        if stats.is_reliable {
                            let mut selected = selected_source.lock().unwrap();
                            if selected.is_none() {
                                *selected = Some(port_name.to_string());
                                info!("Auto-selected first reliable clock source: {} (BPM: {:.1})", 
                                      port_name, stats.estimated_bpm.unwrap_or(0.0));
                            }
                        }
                    }
                }

                let _ = sender.send(ClockScanEvent::ClockTick {
                    port_name: port_name.to_string(),
                    timestamp: now,
                });
            }
            0xFA => {
                // MIDI Start
                debug!("Clock start detected on {}", port_name);
                let _ = sender.send(ClockScanEvent::ClockStart {
                    port_name: port_name.to_string(),
                });
            }
            0xFB => {
                // MIDI Continue  
                debug!("Clock continue detected on {}", port_name);
                let _ = sender.send(ClockScanEvent::ClockStart {
                    port_name: port_name.to_string(),
                });
            }
            0xFC => {
                // MIDI Stop
                debug!("Clock stop detected on {}", port_name);
                let _ = sender.send(ClockScanEvent::ClockStop {
                    port_name: port_name.to_string(),
                });
            }
            _ => {
                // Ignore other messages during scanning
            }
        }
    }

    /// Process scan events (call this periodically during scanning)
    pub fn process_events(&self) {
        loop {
            match self.event_receiver.try_recv() {
                Ok(event) => {
                    match event {
                        ClockScanEvent::ScanComplete { port_name, had_clock } => {
                            if had_clock {
                                info!("✓ Clock source found on: {}", port_name);
                            } else {
                                debug!("✗ No clock found on: {}", port_name);
                            }
                        }
                        ClockScanEvent::ScanError { port_name, error } => {
                            warn!("Scan error on {}: {}", port_name, error);
                        }
                        _ => {
                            // Other events are handled in the callback
                        }
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    error!("Event channel disconnected");
                    break;
                }
            }
        }
    }

    /// Get all detected clock sources
    pub fn get_detected_sources(&self) -> HashMap<String, ClockSourceStats> {
        self.clock_sources.lock().unwrap().clone()
    }

    /// Get the currently selected (locked) clock source
    pub fn get_selected_source(&self) -> Option<String> {
        self.selected_source.lock().unwrap().clone()
    }

    /// Manually select a specific clock source
    pub fn select_source(&self, port_name: &str) -> Result<()> {
        let sources = self.clock_sources.lock().unwrap();
        if sources.contains_key(port_name) {
            *self.selected_source.lock().unwrap() = Some(port_name.to_string());
            info!("Manually selected clock source: {}", port_name);
            Ok(())
        } else {
            Err(anyhow!("Clock source '{}' not found", port_name))
        }
    }

    /// Stop scanning
    pub fn stop_scan(&self) {
        *self.is_scanning.lock().unwrap() = false;
        info!("MIDI clock scan stopped");
    }

    /// Check if scanning is currently active
    pub fn is_scanning(&self) -> bool {
        *self.is_scanning.lock().unwrap()
    }

    /// Get a summary of scan results
    pub fn get_scan_summary(&self) -> ScanSummary {
        let sources = self.clock_sources.lock().unwrap();
        let selected = self.selected_source.lock().unwrap();
        
        let reliable_sources: Vec<String> = sources.iter()
            .filter_map(|(name, stats)| {
                if stats.is_reliable && stats.is_alive() {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        let total_ports_scanned = sources.len();
        let ports_with_clock = sources.iter()
            .filter(|(_, stats)| stats.total_ticks > 0)
            .count();

        ScanSummary {
            total_ports_scanned,
            ports_with_clock,
            reliable_sources,
            selected_source: selected.clone(),
            is_scanning: self.is_scanning(),
        }
    }
}

/// Summary of MIDI clock scan results
#[derive(Debug, Clone)]
pub struct ScanSummary {
    pub total_ports_scanned: usize,
    pub ports_with_clock: usize,
    pub reliable_sources: Vec<String>,
    pub selected_source: Option<String>,
    pub is_scanning: bool,
}

impl ScanSummary {
    pub fn has_reliable_sources(&self) -> bool {
        !self.reliable_sources.is_empty()
    }

    pub fn is_locked(&self) -> bool {
        self.selected_source.is_some()
    }
}

/// Convenience function to perform a complete MIDI clock scan
pub fn scan_for_midi_clock() -> Result<ScanSummary> {
    let scanner = MidiClockScanner::new();
    
    info!("Starting comprehensive MIDI clock scan...");
    scanner.start_scan()?;

    // Wait for scan to complete
    let scan_start = Instant::now();
    let timeout = Duration::from_millis(SCAN_DURATION_MS + 1000); // Extra time for cleanup

    while scanner.is_scanning() && scan_start.elapsed() < timeout {
        scanner.process_events();
        thread::sleep(Duration::from_millis(100));
    }

    scanner.stop_scan();
    
    let summary = scanner.get_scan_summary();
    
    info!("MIDI clock scan completed:");
    info!("  Ports scanned: {}", summary.total_ports_scanned);
    info!("  Ports with clock: {}", summary.ports_with_clock);
    info!("  Reliable sources: {}", summary.reliable_sources.len());
    if let Some(ref selected) = summary.selected_source {
        info!("  Auto-selected: {}", selected);
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_source_stats() {
        let mut stats = ClockSourceStats::new("Test Port".to_string());
        assert_eq!(stats.total_ticks, 0);
        assert!(!stats.is_reliable);

        // Add enough ticks to make it reliable
        let start_time = Instant::now();
        for i in 0..MIN_CLOCK_TICKS_FOR_RELIABILITY {
            let tick_time = start_time + Duration::from_millis((i * 20) as u64); // 50ms between ticks (120 BPM)
            stats.add_tick(tick_time);
        }

        assert!(stats.is_reliable);
        assert!(stats.estimated_bpm.is_some());
    }

    #[test]
    fn test_scanner_creation() {
        let scanner = MidiClockScanner::new();
        assert!(!scanner.is_scanning());
        assert!(scanner.get_selected_source().is_none());
    }

    #[test]
    fn test_scan_summary() {
        let summary = ScanSummary {
            total_ports_scanned: 3,
            ports_with_clock: 1,
            reliable_sources: vec!["USB MIDI".to_string()],
            selected_source: Some("USB MIDI".to_string()),
            is_scanning: false,
        };

        assert!(summary.has_reliable_sources());
        assert!(summary.is_locked());
    }
}