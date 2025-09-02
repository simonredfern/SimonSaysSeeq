//! Hardware abstraction layer for Norns device
//! 
//! Handles encoders, buttons, and other input devices on the Norns hardware.

use anyhow::Result;
use crossbeam_channel::Sender;
#[cfg(feature = "hardware")]
use evdev::{Device, InputEventKind};
use log::{info, warn, debug, error};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Hardware events that can be sent to the main application
#[derive(Debug, Clone)]
pub enum HardwareEvent {
    /// Encoder was turned
    EncoderTurn { encoder: u8, delta: i32 },
    /// Key was pressed or released
    KeyPress { key: u8, pressed: bool },
    /// Grid button was pressed or released
    GridPress { grid_id: String, x: usize, y: usize, pressed: bool },
    /// Start/Stop toggle with flash
    StartStopToggle,
    /// Shutdown signal
    Shutdown,
}

/// Norns hardware interface
#[derive(Clone)]
pub struct NornsHardware {
    devices: Arc<std::sync::Mutex<std::collections::HashMap<String, Device>>>,
}

impl NornsHardware {
    pub fn new() -> Result<Self> {
        let hardware = Self {
            devices: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        };
        hardware.discover_devices()?;
        Ok(hardware)
    }
    
    /// Discover and initialize input devices
    #[cfg(feature = "hardware")]
    fn discover_devices(&self) -> Result<()> {
        info!("Discovering Norns input devices...");
        
        let input_dir = std::path::Path::new("/dev/input");
        if !input_dir.exists() {
            warn!("/dev/input directory not found - running in simulation mode");
            return Ok(());
        }
        
        // Look for input devices
        for entry in std::fs::read_dir(input_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    if filename_str.starts_with("event") {
                        if let Ok(device) = Device::open(&path) {
                            let name = device.name().unwrap_or("Unknown").to_string();
                            info!("Found input device: {} at {:?}", name, path);
                            
                            // Check if this looks like a Norns device
                            if self.is_norns_device(&name) {
                                let mut devices = self.devices.lock().unwrap();
                                devices.insert(name.clone(), device);
                                info!("Added Norns device: {}", name);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if a device name indicates it's a Norns input device
    #[cfg(feature = "hardware")]
    fn is_norns_device(&self, name: &str) -> bool {
        let norns_indicators = [
            "rotary",
            "encoder", 
            "button",
            "key",
            "gpio",
            "input",
        ];
        
        let name_lower = name.to_lowercase();
        norns_indicators.iter().any(|&indicator| name_lower.contains(indicator))
    }
    
    /// Run the input event loop
    pub fn run_input_loop(&mut self, sender: Sender<HardwareEvent>, running: Arc<AtomicBool>) -> Result<()> {
        info!("Starting hardware input loop");
        
        // Note: Signal handler is already set up in main.rs
        // We'll rely on the running flag being set by the main handler
        
        #[cfg(feature = "hardware")]
        {
            while running.load(Ordering::SeqCst) {
                let mut has_events = false;
                
                // Poll all devices for events
                {
                    let mut devices = self.devices.lock().unwrap();
                    for (name, device) in devices.iter_mut() {
                        // Fetch events from this device
                        match device.fetch_events() {
                            Ok(events) => {
                                for event in events {
                                    has_events = true;
                                    if let Err(e) = self.process_input_event(name, &event, &sender) {
                                        error!("Error processing event from {}: {}", name, e);
                                    }
                                }
                            }
                            Err(e) => {
                                if e.kind() != std::io::ErrorKind::WouldBlock {
                                    error!("Error fetching events from {}: {}", name, e);
                                }
                            }
                        }
                    }
                }
                
                // If no events, sleep briefly to avoid busy waiting
                if !has_events {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        

        
        // Send shutdown event
        let _ = sender.send(HardwareEvent::Shutdown);
        info!("Hardware input loop terminated");
        Ok(())
    }






    
    /// Process a single input event
    #[cfg(feature = "hardware")]
    fn process_input_event(&self, device_name: &str, event: &evdev::InputEvent, sender: &Sender<HardwareEvent>) -> Result<()> {
        debug!("Event from {}: {:?}", device_name, event);
        
        match event.kind() {
            InputEventKind::RelAxis(axis) => {
                // Rotary encoders
                let encoder_num = self.map_rel_axis_to_encoder(axis)?;
                let delta = event.value();
                
                debug!("Encoder {} turned: {}", encoder_num, delta);
                sender.send(HardwareEvent::EncoderTurn {
                    encoder: encoder_num,
                    delta,
                })?;
            }
            
            InputEventKind::Key(key) => {
                // Buttons/keys
                let pressed = event.value() != 0;
                let key_num = self.map_key_to_norns_key(key)?;
                
                debug!("Key {} {}", key_num, if pressed { "pressed" } else { "released" });
                sender.send(HardwareEvent::KeyPress {
                    key: key_num,
                    pressed,
                })?;
            }
            
            _ => {
                // Ignore other event types for now
                debug!("Ignoring event type: {:?}", event.kind());
            }
        }
        
        Ok(())
    }
    
    /// Map evdev relative axis to Norns encoder number
    #[cfg(feature = "hardware")]
    fn map_rel_axis_to_encoder(&self, axis: evdev::RelativeAxisType) -> Result<u8> {
        use evdev::RelativeAxisType;
        
        match axis {
            RelativeAxisType::REL_X => Ok(1),      // Left encoder
            RelativeAxisType::REL_Y => Ok(2),      // Middle encoder  
            RelativeAxisType::REL_Z => Ok(3),      // Right encoder
            RelativeAxisType::REL_RX => Ok(1),     // Alternative mapping
            RelativeAxisType::REL_RY => Ok(2),     // Alternative mapping
            RelativeAxisType::REL_RZ => Ok(3),     // Alternative mapping
            _ => {
                // Default to encoder 1 for unknown types
                Ok(1)
            }
        }
    }
    
    /// Map evdev key to Norns key number
    #[cfg(feature = "hardware")]
    fn map_key_to_norns_key(&self, key: evdev::Key) -> Result<u8> {
        use evdev::Key;
        
        match key {
            Key::KEY_LEFTSHIFT | Key::BTN_0 | Key::KEY_ESC => Ok(1),     // K1 (usually not used)
            Key::KEY_LEFTCTRL | Key::BTN_1 | Key::KEY_ENTER => Ok(2),    // K2 (left/stop)
            Key::KEY_LEFTALT | Key::BTN_2 | Key::KEY_SPACE => Ok(3),     // K3 (right/start)
            _ => {
                // Default mapping for other keys
                Ok(2)
            }
        }
    }
}

/// Stub implementation for grid hardware (to be implemented with USB HID)
pub struct GridHardware {
    // TODO: Implement actual grid communication
}

impl GridHardware {
    pub fn new() -> Result<Self> {
        warn!("Grid hardware not yet implemented - using stub");
        Ok(Self {})
    }
    
    pub fn set_led(&mut self, grid_id: usize, x: usize, y: usize, brightness: u8) -> Result<()> {
        debug!("Grid {} LED ({}, {}) = {}", grid_id, x, y, brightness);
        // TODO: Implement actual LED control
        Ok(())
    }
    
    pub fn refresh(&mut self, grid_id: usize) -> Result<()> {
        debug!("Refresh grid {}", grid_id);
        // TODO: Implement actual grid refresh
        Ok(())
    }
    
    pub fn clear_all(&mut self, grid_id: usize) -> Result<()> {
        debug!("Clear all LEDs on grid {}", grid_id);
        // TODO: Implement actual clear
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encoder_mapping() {
        let hw = NornsHardware::new().unwrap();
        
        // Test basic encoder mappings
        assert_eq!(hw.map_rel_axis_to_encoder(evdev::RelativeAxisType::REL_X).unwrap(), 1);
        assert_eq!(hw.map_rel_axis_to_encoder(evdev::RelativeAxisType::REL_Y).unwrap(), 2);
        assert_eq!(hw.map_rel_axis_to_encoder(evdev::RelativeAxisType::REL_Z).unwrap(), 3);
    }
    
    #[test]
    fn test_key_mapping() {
        let hw = NornsHardware::new().unwrap();
        
        // Test basic key mappings
        assert_eq!(hw.map_key_to_norns_key(evdev::Key::KEY_LEFTCTRL).unwrap(), 2);
        assert_eq!(hw.map_key_to_norns_key(evdev::Key::KEY_LEFTALT).unwrap(), 3);
    }
    
    #[test]
    fn test_device_detection() {
        let hw = NornsHardware::new().unwrap();
        
        // Test device name detection
        assert!(hw.is_norns_device("rotary encoder"));
        assert!(hw.is_norns_device("GPIO Keys"));
        assert!(hw.is_norns_device("input device"));
        assert!(!hw.is_norns_device("USB Mouse"));
    }
}