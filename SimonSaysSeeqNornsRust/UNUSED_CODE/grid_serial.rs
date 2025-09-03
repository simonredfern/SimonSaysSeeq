//! ⚠️ DEPRECATED: Direct Serial Grid Communication Module
//!
//! ⚠️ WARNING: This code is UNUSED and DEPRECATED!
//! ⚠️ DO NOT USE - The project switched back to OSC/serialosc communication
//! ⚠️ This file is kept for historical reference only
//!
//! Original description:
//! Bypasses serialosc entirely and communicates directly with Monome grids
//! via serial/USB FTDI devices. This approach was found to be problematic
//! and was replaced with OSC-based communication via serialosc daemon.

use anyhow::{Result, anyhow};
use log::{info, debug, warn, error};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};

#[cfg(feature = "serialport")]
use serialport::{SerialPort, SerialPortType};

/// Grid button event
#[derive(Debug, Clone)]
pub struct GridButtonEvent {
    pub grid_id: String,
    pub x: usize,
    pub y: usize,
    pub pressed: bool,
}

/// Information about a connected grid device
#[derive(Debug)]
struct GridDevice {
    id: String,
    device_path: String,
    device_type: String,
    cols: usize,
    rows: usize,
    is_varibright: bool,
    #[cfg(feature = "serialport")]
    port: Option<Box<dyn SerialPort>>,
}

/// Direct Serial Grid Manager
pub struct GridManager {
    devices: HashMap<String, GridDevice>,
    assumed_led_states: HashMap<(String, usize, usize), u8>,
    last_discovery: Instant,
    discovery_interval: Duration,
}

impl GridManager {
    /// Create new grid manager with direct serial communication
    pub fn new() -> Result<Self> {
        info!("Creating Direct Serial Grid Manager...");
        
        Ok(GridManager {
            devices: HashMap::new(),
            assumed_led_states: HashMap::new(),
            last_discovery: Instant::now() - Duration::from_secs(60), // Force initial discovery
            discovery_interval: Duration::from_secs(5),
        })
    }

    /// Discover connected grid devices by scanning USB/Serial ports
    pub fn discover_devices(&mut self) -> Result<()> {
        if self.last_discovery.elapsed() < self.discovery_interval {
            return Ok(());
        }
        
        info!("Discovering grid devices via direct serial...");
        self.last_discovery = Instant::now();

        #[cfg(feature = "serialport")]
        {
            // Get available serial ports
            match serialport::available_ports() {
                Ok(ports) => {
                    for port_info in ports {
                        if self.is_potential_grid_device(&port_info) {
                            self.try_connect_device(&port_info)?;
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to enumerate serial ports: {}", e);
                }
            }
        }

        #[cfg(not(feature = "serialport"))]
        {
            warn!("Serial port support not enabled - no grids will be detected");
        }

        let device_count = self.devices.len();
        if device_count > 0 {
            info!("Found {} grid device(s)", device_count);
        } else {
            info!("No grid devices found. Make sure:");
            info!("  1. Your grid is connected via USB");
            info!("  2. You have permission to access serial devices");
            info!("  3. The grid is powered on");
        }

        Ok(())
    }

    #[cfg(feature = "serialport")]
    fn is_potential_grid_device(&self, port_info: &serialport::SerialPortInfo) -> bool {
        match &port_info.port_type {
            SerialPortType::UsbPort(usb_info) => {
                // Check for FTDI devices (common for Monome grids)
                // Monome grids typically use FTDI chips
                usb_info.vid == 0x0403 && (usb_info.pid == 0x6001 || usb_info.pid == 0x6014)
            }
            _ => false,
        }
    }

    #[cfg(feature = "serialport")]
    fn try_connect_device(&mut self, port_info: &serialport::SerialPortInfo) -> Result<()> {
        let device_path = port_info.port_name.clone();
        
        // Skip if already connected
        if self.devices.values().any(|d| d.device_path == device_path) {
            return Ok(());
        }

        debug!("Attempting to connect to potential grid at {}", device_path);

        match serialport::new(&device_path, 115200)
            .timeout(Duration::from_millis(1000))
            .open()
        {
            Ok(mut port) => {
                // Try to identify the device by sending an info command
                if let Ok(grid_info) = self.identify_grid_device(&mut port) {
                    let device_id = format!("grid_{}", self.devices.len());
                    
                    info!("Connected to grid: {} ({}x{}) at {}", 
                          grid_info.device_type, grid_info.cols, grid_info.rows, device_path);

                    let device = GridDevice {
                        id: device_id.clone(),
                        device_path: device_path.clone(),
                        device_type: grid_info.device_type,
                        cols: grid_info.cols,
                        rows: grid_info.rows,
                        is_varibright: grid_info.is_varibright,
                        port: Some(port),
                    };

                    self.devices.insert(device_id, device);
                }
            }
            Err(e) => {
                debug!("Failed to connect to {}: {}", device_path, e);
            }
        }

        Ok(())
    }

    #[cfg(feature = "serialport")]
    fn identify_grid_device(&self, port: &mut Box<dyn SerialPort>) -> Result<GridInfo> {
        // Send device info command (Monome protocol)
        let info_cmd = [0x00, 0x00]; // Device info command
        port.write_all(&info_cmd)?;
        port.flush()?;

        // Wait for response
        thread::sleep(Duration::from_millis(100));

        let mut buffer = [0u8; 64];
        let bytes_read = port.read(&mut buffer)?;
        
        if bytes_read >= 4 {
            // Parse device info response
            // This is a simplified parser - actual Monome protocol may vary
            let device_type = match buffer[0] {
                0x01 => "Grid128".to_string(),
                0x02 => "Grid64".to_string(),
                0x03 => "Grid256".to_string(),
                _ => "GridUnknown".to_string(),
            };
            
            let cols = buffer[1] as usize;
            let rows = buffer[2] as usize;
            let is_varibright = buffer[3] != 0;

            // Default to 16x8 if we can't read proper dimensions
            let (cols, rows) = if cols == 0 || rows == 0 {
                (16, 8)
            } else {
                (cols, rows)
            };

            Ok(GridInfo {
                device_type,
                cols,
                rows,
                is_varibright,
            })
        } else {
            // Fallback - assume it's a 16x8 grid
            Ok(GridInfo {
                device_type: "Grid128".to_string(),
                cols: 16,
                rows: 8,
                is_varibright: true,
            })
        }
    }

    /// Set a single LED
    pub fn set_led(&mut self, grid_id: &str, x: usize, y: usize, brightness: u8, _caller: &str) -> Result<()> {
        #[cfg(feature = "serialport")]
        {
            if let Some(device) = self.devices.get_mut(grid_id) {
                if x >= device.cols || y >= device.rows {
                    return Err(anyhow!("LED coordinates out of bounds: ({}, {}) for {}x{} grid", 
                                     x, y, device.cols, device.rows));
                }

                // Send LED set command (Monome protocol)
                // Format: [0x10, x, y, brightness]
                let cmd = [0x10, x as u8, y as u8, brightness];
                
                if let Some(ref mut port) = device.port {
                    port.write_all(&cmd)?;
                    port.flush()?;
                }

                // Update assumed state
                self.assumed_led_states.insert((grid_id.to_string(), x, y), brightness);
            }
        }

        #[cfg(not(feature = "serialport"))]
        {
            debug!("Serial support not enabled - LED command ignored");
        }

        Ok(())
    }

    /// Get current LED state (from assumed state)
    pub fn get_led(&self, grid_id: &str, x: usize, y: usize) -> u8 {
        self.assumed_led_states.get(&(grid_id.to_string(), x, y)).copied().unwrap_or(0)
    }

    /// Clear all LEDs on a grid
    pub fn clear_all(&mut self, grid_id: &str) -> Result<()> {
        if let Some(device) = self.devices.get(grid_id) {
            let rows = device.rows;
            let cols = device.cols;
            for y in 0..rows {
                for x in 0..cols {
                    self.set_led(grid_id, x, y, 0, "clear_all")?;
                }
            }
        }
        Ok(())
    }

    /// Clear only sequencer rows (0-6), preserve control row (7)
    /// Clear sequence rows (rows 0-7) on a grid
    pub fn clear_sequence_rows(&mut self, grid_id: &str) -> Result<()> {
        if let Some(device) = self.devices.get(grid_id) {
            let rows = 8.min(device.rows);
            let cols = device.cols;
            for y in 0..rows {
                for x in 0..cols {
                    self.set_led(grid_id, x, y, 0, "clear_sequence_rows")?;
                }
            }
        }
        Ok(())
    }

    /// Set LED map for efficient bulk updates
    pub fn set_led_map(&mut self, grid_id: &str, led_map: &[Vec<u8>]) -> Result<()> {
        let (rows, cols) = if let Some(device) = self.devices.get(grid_id) {
            (device.rows, device.cols)
        } else {
            return Ok(());
        };

        for (y, row) in led_map.iter().enumerate() {
            if y >= rows { break; }
            for (x, &brightness) in row.iter().enumerate() {
                if x >= cols { break; }
                self.set_led(grid_id, x, y, brightness, "set_led_map")?;
            }
        }
        Ok(())
    }

    /// Get grid dimensions
    pub fn get_dimensions(&self, grid_id: &str) -> Option<(usize, usize)> {
        self.devices.get(grid_id).map(|d| (d.cols, d.rows))
    }

    /// Get grid device name/type
    pub fn get_name(&self, grid_id: &str) -> Option<String> {
        self.devices.get(grid_id).map(|d| d.device_type.clone())
    }

    /// Check if grid supports variable brightness
    pub fn is_varibright(&self, grid_id: &str) -> bool {
        self.devices.get(grid_id).map(|d| d.is_varibright).unwrap_or(false)
    }

    /// Get list of connected grid IDs
    pub fn get_connected_grids(&self) -> Vec<String> {
        let mut grids: Vec<String> = self.devices.keys().cloned().collect();
        grids.sort(); // Ensure consistent ordering
        grids
    }

    /// Read button events from grids
    pub fn read_button_events(&mut self) -> Result<Vec<GridButtonEvent>> {
        let mut events = Vec::new();

        #[cfg(feature = "serialport")]
        {
            for (grid_id, device) in &mut self.devices {
                if let Some(ref mut port) = device.port {
                    let mut buffer = [0u8; 256];
                    
                    // Non-blocking read
                    match port.read(&mut buffer) {
                        Ok(bytes_read) if bytes_read > 0 => {
                            // Parse button events from buffer
                            let mut i = 0;
                            while i + 3 < bytes_read {
                                if buffer[i] == 0x20 { // Button event command
                                    let x = buffer[i + 1] as usize;
                                    let y = buffer[i + 2] as usize;
                                    let pressed = buffer[i + 3] != 0;
                                    
                                    if x < device.cols && y < device.rows {
                                        events.push(GridButtonEvent {
                                            grid_id: grid_id.clone(),
                                            x,
                                            y,
                                            pressed,
                                        });
                                    }
                                    
                                    i += 4;
                                } else {
                                    i += 1;
                                }
                            }
                        }
                        Ok(_) => {}, // No data available
                        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}, // Timeout is OK
                        Err(e) => {
                            warn!("Error reading from grid {}: {}", grid_id, e);
                        }
                    }
                }
            }
        }

        Ok(events)
    }

    /// Test grid functionality
    pub fn test_grid(&mut self, grid_id: &str) -> Result<()> {
        info!("Testing grid: {}", grid_id);
        
        // Flash all corners briefly
        self.test_corners_only(grid_id)?;
        
        Ok(())
    }

    /// Flash all connected grids
    pub fn flash_all_grids(&mut self) -> Result<()> {
        let connected_grids = self.get_connected_grids();
        
        for grid_id in connected_grids {
            self.flash_grid(&grid_id, 2)?;
        }
        
        Ok(())
    }

    /// Flash a specific grid
    pub fn flash_grid(&mut self, grid_id: &str, flash_count: usize) -> Result<()> {
        let (rows, cols) = if let Some(device) = self.devices.get(grid_id) {
            (device.rows, device.cols)
        } else {
            return Ok(());
        };

        for _ in 0..flash_count {
            // Turn all LEDs on
            for y in 0..rows {
                for x in 0..cols {
                    self.set_led(grid_id, x, y, 15, "flash")?;
                }
            }
            
            thread::sleep(Duration::from_millis(100));
            
            // Turn all LEDs off
            self.clear_all(grid_id)?;
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    fn test_corners_only(&mut self, grid_id: &str) -> Result<()> {
        if let Some(device) = self.devices.get(grid_id) {
            let max_x = device.cols - 1;
            let max_y = device.rows - 1;
            
            // Light up corners
            self.set_led(grid_id, 0, 0, 15, "test")?;           // Top-left
            self.set_led(grid_id, max_x, 0, 15, "test")?;       // Top-right
            self.set_led(grid_id, 0, max_y, 15, "test")?;       // Bottom-left
            self.set_led(grid_id, max_x, max_y, 15, "test")?;   // Bottom-right
            
            thread::sleep(Duration::from_millis(500));
            
            // Clear corners
            self.set_led(grid_id, 0, 0, 0, "test")?;
            self.set_led(grid_id, max_x, 0, 0, "test")?;
            self.set_led(grid_id, 0, max_y, 0, "test")?;
            self.set_led(grid_id, max_x, max_y, 0, "test")?;
        }
        
        Ok(())
    }

    /// Refresh display (no-op for direct serial - updates are immediate)
    pub fn refresh(&mut self) -> Result<()> {
        // Serial grids update immediately when LEDs are set
        // This method is provided for compatibility with other grid implementations
        Ok(())
    }

    /// Print diagnostic information about connected grids
    pub fn print_grid_info(&self) {
        info!("Connected grids via direct serial:");
        for (id, device) in &self.devices {
            info!("  {}: {} ({}x{}) at {}", 
                  id, device.device_type, device.cols, device.rows, device.device_path);
        }
        if self.devices.is_empty() {
            info!("  No grids connected");
        }
    }
}

/// Grid device information
struct GridInfo {
    device_type: String,
    cols: usize,
    rows: usize,
    is_varibright: bool,
}

impl Default for GridManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            error!("Failed to create GridManager: {}", e);
            GridManager {
                devices: HashMap::new(),
                assumed_led_states: HashMap::new(),
                last_discovery: Instant::now(),
                discovery_interval: Duration::from_secs(5),
            }
        })
    }
}

// Thread-safe wrapper if needed
pub type SharedGridManager = Arc<Mutex<GridManager>>;

/// Create a thread-safe grid manager
pub fn create_shared_manager() -> Result<SharedGridManager> {
    let manager = GridManager::new()?;
    Ok(Arc::new(Mutex::new(manager)))
}