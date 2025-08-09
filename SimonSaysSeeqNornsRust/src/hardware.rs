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
    GridPress { grid_id: usize, x: usize, y: usize, pressed: bool },
    /// Start/Stop toggle with flash
    StartStopToggle,
    /// Shutdown signal
    Shutdown,
}

/// Norns hardware interface
#[derive(Clone)]
pub struct NornsHardware {
    #[cfg(feature = "hardware")]
    devices: Arc<std::sync::Mutex<std::collections::HashMap<String, Device>>>,
    #[cfg(not(feature = "hardware"))]
    _phantom: std::marker::PhantomData<()>,
}

impl NornsHardware {
    pub fn new() -> Result<Self> {
        #[cfg(feature = "hardware")]
        {
            let hardware = Self {
                devices: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            };
            hardware.discover_devices()?;
            Ok(hardware)
        }
        
        #[cfg(not(feature = "hardware"))]
        {
            info!("Hardware simulation mode - no actual device access");
            Ok(Self {
                _phantom: std::marker::PhantomData,
            })
        }
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
        
        // Set up signal handler for graceful shutdown
        let running_signal = running.clone();
        ctrlc::set_handler(move || {
            info!("Shutdown signal received");
            running_signal.store(false, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");
        
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
        
        #[cfg(not(feature = "hardware"))]
        {
            // Simulation mode with keyboard input
            info!("Framework RGB Macropad simulation mode active!");
            info!("Use keys 1-9, a-g to simulate 4x4 macropad grid presses:");
            info!("  1 2 3 4  ->  (0,0) (1,0) (2,0) (3,0)");
            info!("  q w e r  ->  (0,1) (1,1) (2,1) (3,1)");
            info!("  a s d f  ->  (0,2) (1,2) (2,2) (3,2)");
            info!("  z x c v  ->  (0,3) (1,3) (2,3) (3,3)");
            info!("Press 'space' then Enter to run/stop sequencer");
            info!("Press 'p' then Enter to quit");
            info!("Press any macropad key then Enter to simulate button press");
            
            // Spawn keyboard input thread
            let sender_clone = sender.clone();
            let running_clone = running.clone();
            
            thread::spawn(move || {
                use std::io::{self, BufRead};
                let stdin = io::stdin();
                
                for line in stdin.lock().lines() {
                    if !running_clone.load(Ordering::SeqCst) {
                        break;
                    }
                    
                    if let Ok(input) = line {
                        let input = input.trim();
                        if input == "p" {
                            info!("Quit requested from keyboard");
                            let _ = sender_clone.send(HardwareEvent::Shutdown);
                            break;
                        }
                        
                        if input == " " {
                            info!("Run/stop toggle requested from keyboard");
                            let _ = sender_clone.send(HardwareEvent::StartStopToggle);
                            continue;
                        }
                        
                        if let Some(key_char) = input.chars().next() {
                            if let Some((x, y)) = Self::macropad_to_grid_coords(key_char) {
                                info!("🔥 Macropad button press: ({}, {}) - Button {}", x, y, Self::coords_to_button_name(x, y));
                                
                                // Send press event
                                let _ = sender_clone.send(HardwareEvent::GridPress {
                                    grid_id: 0,
                                    x,
                                    y,
                                    pressed: true,
                                });
                                
                                // Small delay, then send release event
                                thread::sleep(Duration::from_millis(50));
                                let _ = sender_clone.send(HardwareEvent::GridPress {
                                    grid_id: 0,
                                    x,
                                    y,
                                    pressed: false,
                                });
                            } else {
                                warn!("Unknown key '{}'. Use macropad keys 1-4/qwer/asdf/zxcv, 'space' to run/stop, or 'p' to quit", key_char);
                            }
                        }
                    }
                }
            });
            
            // Main simulation loop
            while running.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));
            }
        }
        
        // Send shutdown event
        let _ = sender.send(HardwareEvent::Shutdown);
        info!("Hardware input loop terminated");
        Ok(())
    }

    /// Map Framework RGB Macropad keys to grid coordinates for simulation
    #[cfg(not(feature = "hardware"))]
    fn macropad_to_grid_coords(key: char) -> Option<(usize, usize)> {
        match key {
            // Top row: 1 2 3 4
            '1' => Some((0, 0)), '2' => Some((1, 0)), '3' => Some((2, 0)), '4' => Some((3, 0)),
            // Second row: q w e r
            'q' => Some((0, 1)), 'w' => Some((1, 1)), 'e' => Some((2, 1)), 'r' => Some((3, 1)),
            // Third row: a s d f
            'a' => Some((0, 2)), 's' => Some((1, 2)), 'd' => Some((2, 2)), 'f' => Some((3, 2)),
            // Bottom row: z x c v
            'z' => Some((0, 3)), 'x' => Some((1, 3)), 'c' => Some((2, 3)), 'v' => Some((3, 3)),
            _ => None,
        }
    }

    /// Convert grid coordinates to button name for display
    #[cfg(not(feature = "hardware"))]
    fn coords_to_button_name(x: usize, y: usize) -> String {
        match (x, y) {
            (0, 0) => "1".to_string(), (1, 0) => "2".to_string(), (2, 0) => "3".to_string(), (3, 0) => "4".to_string(),
            (0, 1) => "Q".to_string(), (1, 1) => "W".to_string(), (2, 1) => "E".to_string(), (3, 1) => "R".to_string(),
            (0, 2) => "A".to_string(), (1, 2) => "S".to_string(), (2, 2) => "D".to_string(), (3, 2) => "F".to_string(),
            (0, 3) => "Z".to_string(), (1, 3) => "X".to_string(), (2, 3) => "C".to_string(), (3, 3) => "V".to_string(),
            _ => format!("({},{})", x, y),
        }
    }

    /// Execute Framework RGB Macropad flash sequence
    #[cfg(not(feature = "hardware"))]
    pub fn execute_flash_sequence(sender: &Sender<HardwareEvent>) {
        use std::thread;
        use std::time::Duration;
        
        // Spawn thread to avoid blocking main loop
        let sender_clone = sender.clone();
        thread::spawn(move || {
            info!("🌈 Starting Framework RGB Macropad flash sequence!");
            
            // Flash all 16 buttons in order: 1,2,3,4,Q,W,E,R,A,S,D,F,Z,X,C,V
            let sequence = [
                (0, 0), (1, 0), (2, 0), (3, 0),  // 1 2 3 4
                (0, 1), (1, 1), (2, 1), (3, 1),  // Q W E R
                (0, 2), (1, 2), (2, 2), (3, 2),  // A S D F  
                (0, 3), (1, 3), (2, 3), (3, 3),  // Z X C V
            ];

            for (i, &(x, y)) in sequence.iter().enumerate() {
                let button_name = Self::coords_to_button_name(x, y);
                info!("💡 Flash button {} ({},{}) - Step {}/16", button_name, x, y, i + 1);
                
                // Simulate button press for visual feedback
                let _ = sender_clone.send(HardwareEvent::GridPress {
                    grid_id: 0,
                    x,
                    y,
                    pressed: true,
                });
                
                // Hold the flash for 125ms (2000ms / 16 buttons = 125ms each)
                thread::sleep(Duration::from_millis(125));
                
                // Release button
                let _ = sender_clone.send(HardwareEvent::GridPress {
                    grid_id: 0,
                    x,
                    y,
                    pressed: false,
                });
            }
            
            info!("✨ Macropad flash sequence complete!");
        });
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