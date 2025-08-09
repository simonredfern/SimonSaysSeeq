//! Grid module - Handle monome grid communication and LED control
//! 
//! Provides interface to monome grid devices via serial communication (CDC-ACM).

use anyhow::{Result, anyhow};
#[cfg(feature = "hardware")]
use serialport::{SerialPort, available_ports, SerialPortType};
use log::{info, debug, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::io::{Read, Write};

/// Grid button event
#[derive(Debug, Clone)]
pub struct GridButtonEvent {
    pub grid_id: usize,
    pub x: usize,
    pub y: usize,
    pub pressed: bool,
}

/// Grid device information
#[derive(Debug)]
struct GridDevice {
    #[cfg(feature = "hardware")]
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    rows: usize,
    cols: usize,
    name: String,
    is_varibright: bool,
    port_name: String,
}

/// Grid manager handles multiple grid devices
pub struct GridManager {
    devices: HashMap<usize, GridDevice>,
    led_states: HashMap<(usize, usize, usize), u8>, // (grid_id, x, y) -> brightness
}

impl GridManager {
    /// Create a new grid manager
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            devices: HashMap::new(),
            led_states: HashMap::new(),
        };
        
        #[cfg(feature = "hardware")]
        {
            manager.discover_grids()?;
        }
        
        #[cfg(not(feature = "hardware"))]
        {
            info!("Grid simulation mode - no actual grid devices");
        }
        
        Ok(manager)
    }
    
    /// Discover and connect to grid devices via serial ports
    #[cfg(feature = "hardware")]
    fn discover_grids(&mut self) -> Result<()> {
        info!("Discovering grid devices via serial ports...");
        
        debug!("Enumerating available serial ports...");
        let ports = match available_ports() {
            Ok(ports) => ports,
            Err(e) => {
                warn!("Failed to enumerate serial ports: {}", e);
                return Ok(()); // Don't fail completely, just no grids available
            }
        };
        
        let mut monome_count = 0;
        
        // Log all available serial ports
        info!("Found {} serial port(s):", ports.len());
        for port in &ports {
            debug!("  - {}: {:?}", port.port_name, port.port_type);
            
            if let SerialPortType::UsbPort(usb_info) = &port.port_type {
                let vid = usb_info.vid;
                let pid = usb_info.pid;
                debug!("    USB VID:PID {:04X}:{:04X}", vid, pid);
                debug!("    Manufacturer: {:?}", usb_info.manufacturer);
                debug!("    Product: {:?}", usb_info.product);
                
                if self.is_monome_device(vid, pid) {
                    info!("    ✅ MONOME GRID DETECTED: {} (VID:{:04X} PID:{:04X})", 
                          port.port_name, vid, pid);
                    monome_count += 1;
                }
            }
        }
        
        if monome_count == 0 {
            info!("💡 No monome devices found. Expected combinations:");
            info!("  - VID 0A6A (original monome)");
            info!("  - VID CAFE (newer monome devices, like grid)");
            info!("  - Common PIDs: 4001, 0001, 0002, 0003, etc.");
        }
        
        // Connect to monome devices
        info!("Attempting to connect to {} detected monome device(s)...", monome_count);
        let mut grid_id = 0;
        for port_info in &ports {
            if let SerialPortType::UsbPort(usb_info) = &port_info.port_type {
                if self.is_monome_device(usb_info.vid, usb_info.pid) {
                    info!("Connecting to monome grid on {}...", port_info.port_name);
                    match self.connect_to_grid(&port_info.port_name, usb_info.vid, usb_info.pid, grid_id) {
                        Ok(()) => {
                            info!("✅ Successfully connected to grid {} on {}", grid_id, port_info.port_name);
                            grid_id += 1;
                        }
                        Err(e) => {
                            warn!("❌ Failed to connect to grid on {}: {}", port_info.port_name, e);
                        }
                    }
                }
            }
        }
        
        info!("Connected to {} grid device(s)", self.devices.len());
        Ok(())
    }
    
    /// Connect to a specific grid device
    #[cfg(feature = "hardware")]
    fn connect_to_grid(&mut self, port_name: &str, vid: u16, pid: u16, grid_id: usize) -> Result<()> {
        debug!("Opening serial port {} with 115200 baud...", port_name);
        
        // Open serial port with appropriate settings for monome
        let mut port = serialport::new(port_name, 115200)
            .timeout(Duration::from_millis(500)) // Increased timeout for initial connection
            .data_bits(serialport::DataBits::Eight)
            .flow_control(serialport::FlowControl::None)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .open()
            .map_err(|e| anyhow!("Failed to open serial port {}: {}", port_name, e))?;
        
        debug!("Serial port {} opened successfully", port_name);
        
        // Detect grid specifications
        let (cols, rows, is_varibright) = self.detect_grid_specs(pid);
        let product_name = format!("Monome Grid ({:04X}:{:04X})", vid, pid);
        
        // Test communication by sending a query command
        debug!("Testing communication with grid on {}...", port_name);
        if let Err(e) = self.test_grid_communication(&mut port) {
            warn!("Communication test failed for {}: {}", port_name, e);
            // Don't fail completely, some grids might not respond to test commands
        } else {
            debug!("Communication test successful for {}", port_name);
        }
        
        let grid_device = GridDevice {
            port: Arc::new(Mutex::new(port)),
            rows,
            cols,
            name: product_name,
            is_varibright,
            port_name: port_name.to_string(),
        };
        
        // Initialize LED state tracking
        for x in 0..cols {
            for y in 0..rows {
                self.led_states.insert((grid_id, x, y), 0);
            }
        }
        
        self.devices.insert(grid_id, grid_device);
        
        // Initialize the grid (clear all LEDs)
        self.initialize_grid(grid_id)?;
        
        info!("Grid {} connected: {} ({}x{}, varibright: {})", 
              grid_id, self.devices[&grid_id].name, cols, rows, is_varibright);
        
        Ok(())
    }
    
    /// Test grid communication
    #[cfg(feature = "hardware")]
    fn test_grid_communication(&self, port: &mut Box<dyn SerialPort>) -> Result<()> {
        debug!("Sending test command to grid...");
        
        // Send a simple LED command to test communication
        let test_cmd = [0x1A, 0x00, 0x00]; // Clear all LEDs command
        port.write_all(&test_cmd)
            .map_err(|e| anyhow!("Failed to write test command: {}", e))?;
        
        port.flush()
            .map_err(|e| anyhow!("Failed to flush serial port: {}", e))?;
        
        // Small delay to ensure command is processed
        std::thread::sleep(Duration::from_millis(50));
        
        debug!("Grid communication test completed");
        Ok(())
    }
    
    /// Check if a device is a monome grid
    fn is_monome_device(&self, vid: u16, pid: u16) -> bool {
        // Monome vendor IDs
        const MONOME_VID: u16 = 0x0A6A;       // Original monome VID
        const MONOME_ALT_VID: u16 = 0xCAFE;   // Alternate VID for newer devices
        
        // Known monome product IDs
        const MONOME_PIDS: &[u16] = &[
            0x0001, // 40h
            0x0002, // 64
            0x0003, // 128
            0x0004, // 256
            0x0011, // mk series 64
            0x0012, // mk series 128
            0x0013, // mk series 256
            0x4001, // newer grid (commonly seen)
            0x4002, // potential additional grid
            0x4003, // potential additional grid
        ];
        
        let is_monome_vid = vid == MONOME_VID || vid == MONOME_ALT_VID;
        let is_known_pid = MONOME_PIDS.contains(&pid);
        
        // Be more permissive with PIDs for monome VIDs
        if is_monome_vid {
            debug!("Monome VID detected: {:04X}, PID: {:04X}, known PID: {}", vid, pid, is_known_pid);
            return true; // Accept any PID for known monome VIDs
        }
        
        false
    }
    
    /// Detect grid specifications based on product ID
    fn detect_grid_specs(&self, pid: u16) -> (usize, usize, bool) {
        match pid {
            0x0001 => (8, 8, false),   // 40h - 8x8, no varibright
            0x0002 => (8, 8, false),   // 64 - 8x8, no varibright  
            0x0003 => (16, 8, false),  // 128 - 16x8, no varibright
            0x0004 => (16, 16, false), // 256 - 16x16, no varibright
            0x0011 => (8, 8, true),    // mk series - 8x8, varibright
            0x0012 => (16, 8, true),   // mk series - 16x8, varibright
            0x0013 => (16, 16, true),  // mk series - 16x16, varibright
            0x4001 => (16, 8, true),   // Assume modern grid 128
            _ => (16, 8, true),        // Default to 128 mk specs for unknown PIDs
        }
    }
    
    /// Initialize a grid device (clear all LEDs)
    #[cfg(feature = "hardware")]
    fn initialize_grid(&mut self, grid_id: usize) -> Result<()> {
        if let Some(device) = self.devices.get(&grid_id) {
            let mut port = device.port.lock().unwrap();
            
            // Send clear all LEDs command
            let clear_cmd = [0x1A, 0x00, 0x00];
            port.write_all(&clear_cmd)?;
            port.flush()?;
            
            debug!("Grid {} initialized and cleared", grid_id);
        }
        
        Ok(())
    }
    
    /// Set LED brightness for a specific position
    pub fn set_led(&mut self, grid_id: usize, x: usize, y: usize, brightness: u8) -> Result<()> {
        if let Some(device) = self.devices.get(&grid_id) {
            // Validate coordinates
            if x >= device.cols || y >= device.rows {
                return Err(anyhow!("LED coordinates ({}, {}) out of bounds for grid {} ({}x{})", 
                                 x, y, grid_id, device.cols, device.rows));
            }
            
            // Clamp brightness
            let brightness = if device.is_varibright { 
                brightness.min(15) 
            } else { 
                if brightness > 0 { 15 } else { 0 }
            };
            
            // Update internal state
            self.led_states.insert((grid_id, x, y), brightness);
            
            #[cfg(feature = "hardware")]
            {
                let mut port = device.port.lock().unwrap();
                
                // Send LED command: [0x1B, x, y, brightness]
                let led_cmd = [0x1B, x as u8, y as u8, brightness];
                port.write_all(&led_cmd)?;
                port.flush()?;
            }
            
            debug!("Set LED grid {} ({}, {}) = {}", grid_id, x, y, brightness);
        } else {
            return Err(anyhow!("Grid {} not found", grid_id));
        }
        
        Ok(())
    }
    
    /// Get LED brightness for a specific position
    pub fn get_led(&self, grid_id: usize, x: usize, y: usize) -> u8 {
        self.led_states.get(&(grid_id, x, y)).copied().unwrap_or(0)
    }
    
    /// Set multiple LEDs from a 2D brightness map
    pub fn set_led_map(&mut self, grid_id: usize, led_map: &[Vec<u8>]) -> Result<()> {
        if let Some(device) = self.devices.get(&grid_id) {
            let rows = led_map.len().min(device.rows);
            let cols = if rows > 0 { led_map[0].len().min(device.cols) } else { 0 };
            
            #[cfg(feature = "hardware")]
            {
                let mut port = device.port.lock().unwrap();
                
                // Send LED map in chunks to avoid overwhelming the device
                for y in 0..rows {
                    for x in 0..cols {
                        let brightness = if device.is_varibright { 
                            led_map[y][x].min(15) 
                        } else { 
                            if led_map[y][x] > 0 { 15 } else { 0 }
                        };
                        
                        // Update internal state
                        self.led_states.insert((grid_id, x, y), brightness);
                        
                        // Send LED command
                        let led_cmd = [0x1B, x as u8, y as u8, brightness];
                        port.write_all(&led_cmd)?;
                        
                        // Small delay to prevent overwhelming
                        if (x + y * cols) % 8 == 0 {
                            port.flush()?;
                            std::thread::sleep(Duration::from_micros(100));
                        }
                    }
                }
                
                port.flush()?;
            }
            
            debug!("Updated LED map for grid {} ({}x{})", grid_id, cols, rows);
        } else {
            return Err(anyhow!("Grid {} not found", grid_id));
        }
        
        Ok(())
    }
    
    /// Clear all LEDs on a grid
    pub fn clear_all(&mut self, grid_id: usize) -> Result<()> {
        if let Some(device) = self.devices.get(&grid_id) {
            // Clear internal state
            for x in 0..device.cols {
                for y in 0..device.rows {
                    self.led_states.insert((grid_id, x, y), 0);
                }
            }
            
            #[cfg(feature = "hardware")]
            {
                let mut port = device.port.lock().unwrap();
                
                // Send clear all command
                let clear_cmd = [0x1A, 0x00, 0x00];
                port.write_all(&clear_cmd)?;
                port.flush()?;
            }
            
            debug!("Cleared all LEDs on grid {}", grid_id);
        } else {
            return Err(anyhow!("Grid {} not found", grid_id));
        }
        
        Ok(())
    }
    
    /// Refresh all LEDs (resend current state)
    pub fn refresh(&mut self) -> Result<()> {
        let grid_ids: Vec<usize> = self.devices.keys().copied().collect();
        for grid_id in grid_ids {
            self.refresh_grid(grid_id)?;
        }
        Ok(())
    }
    
    /// Refresh a specific grid
    pub fn refresh_grid(&mut self, grid_id: usize) -> Result<()> {
        if let Some(device) = self.devices.get(&grid_id) {
            #[cfg(feature = "hardware")]
            {
                let mut port = device.port.lock().unwrap();
                
                for x in 0..device.cols {
                    for y in 0..device.rows {
                        let brightness = self.led_states.get(&(grid_id, x, y)).copied().unwrap_or(0);
                        let led_cmd = [0x1B, x as u8, y as u8, brightness];
                        port.write_all(&led_cmd)?;
                    }
                }
                
                port.flush()?;
            }
            
            debug!("Refreshed grid {}", grid_id);
        }
        
        Ok(())
    }
    
    /// Get grid dimensions
    pub fn get_dimensions(&self, grid_id: usize) -> Option<(usize, usize)> {
        self.devices.get(&grid_id).map(|d| (d.cols, d.rows))
    }
    
    /// Get grid name
    pub fn get_name(&self, grid_id: usize) -> Option<&str> {
        self.devices.get(&grid_id).map(|d| d.name.as_str())
    }
    
    /// Check if grid supports variable brightness
    pub fn is_varibright(&self, grid_id: usize) -> bool {
        self.devices.get(&grid_id).map(|d| d.is_varibright).unwrap_or(false)
    }
    
    /// Get list of connected grid IDs
    pub fn get_connected_grids(&self) -> Vec<usize> {
        self.devices.keys().copied().collect()
    }
    
    /// Read button events from all grids
    pub fn read_button_events(&mut self) -> Result<Vec<GridButtonEvent>> {
        let mut events = Vec::new();
        
        #[cfg(feature = "hardware")]
        {
            for (&grid_id, device) in &self.devices {
                let mut port = device.port.lock().unwrap();
                
                // Try to read from the serial port (non-blocking)
                let mut buf = [0u8; 64];
                match port.read(&mut buf) {
                    Ok(bytes_read) if bytes_read > 0 => {
                        // Parse button events from the buffer
                        // Monome serial format: [0x00, x, y, pressed] or similar
                        let mut i = 0;
                        while i + 3 < bytes_read {
                            if buf[i] == 0x00 {  // Button event marker
                                let x = buf[i + 1] as usize;
                                let y = buf[i + 2] as usize;
                                let pressed = buf[i + 3] != 0;
                                
                                // Validate coordinates
                                if x < device.cols && y < device.rows {
                                    events.push(GridButtonEvent {
                                        grid_id,
                                        x,
                                        y,
                                        pressed,
                                    });
                                    
                                    debug!("Grid {} button ({}, {}) {}", 
                                           grid_id, x, y, 
                                           if pressed { "pressed" } else { "released" });
                                }
                                
                                i += 4;
                            } else {
                                i += 1;
                            }
                        }
                    }
                    Ok(_) => {
                        // No data available
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::TimedOut {
                            warn!("Error reading from grid {}: {}", grid_id, e);
                        }
                    }
                }
            }
        }
        
        Ok(events)
    }
    
    /// Update step cursor display on all grids
    pub fn update_step_cursor(&mut self, current_step: usize, steps_per_row: usize) -> Result<()> {
        let grid_ids: Vec<usize> = self.devices.keys().copied().collect();
        for grid_id in grid_ids {
            if let Some((cols, rows)) = self.get_dimensions(grid_id) {
                // Calculate cursor position
                let cursor_x = current_step % steps_per_row;
                let cursor_y = current_step / steps_per_row;
                
                if cursor_x < cols && cursor_y < rows {
                    // Flash the current step
                    self.set_led(grid_id, cursor_x, cursor_y, 15)?;
                    
                    // Dim the previous step (simple approach)
                    if current_step > 0 {
                        let prev_x = (current_step - 1) % steps_per_row;
                        let prev_y = (current_step - 1) / steps_per_row;
                        if prev_x < cols && prev_y < rows {
                            self.set_led(grid_id, prev_x, prev_y, 3)?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Test grid functionality
    pub fn test_grid(&mut self, grid_id: usize) -> Result<()> {
        info!("Testing grid {} functionality...", grid_id);
        
        if let Some((cols, rows)) = self.get_dimensions(grid_id) {
            // Flash all LEDs
            for brightness in [15, 0, 15, 0] {
                for x in 0..cols {
                    for y in 0..rows {
                        self.set_led(grid_id, x, y, brightness)?;
                    }
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            
            // Clear
            self.clear_all(grid_id)?;
            info!("Grid {} test completed", grid_id);
        }
        
        Ok(())
    }
    
    /// Flash all connected grids
    pub fn flash_all_grids(&mut self) -> Result<()> {
        let grid_ids: Vec<usize> = self.devices.keys().copied().collect();
        
        for &grid_id in &grid_ids {
            self.flash_grid(grid_id)?;
        }
        
        Ok(())
    }
    
    /// Flash a specific grid
    pub fn flash_grid(&mut self, grid_id: usize) -> Result<()> {
        if let Some((cols, rows)) = self.get_dimensions(grid_id) {
            info!("Flashing grid {} ({}x{})...", grid_id, cols, rows);
            
            // Flash sequence
            for _ in 0..3 {
                // All on
                for x in 0..cols {
                    for y in 0..rows {
                        self.set_led(grid_id, x, y, 15)?;
                    }
                }
                std::thread::sleep(Duration::from_millis(150));
                
                // All off
                self.clear_all(grid_id)?;
                std::thread::sleep(Duration::from_millis(150));
            }
            
            info!("Grid {} flash completed", grid_id);
        } else {
            warn!("Grid {} not found for flashing", grid_id);
        }
        
        Ok(())
    }
    
    /// Print current grid state (for debugging)
    fn print_grid_state(&self, grid_id: usize) {
        if let Some(device) = self.devices.get(&grid_id) {
            info!("Grid {} state ({}x{}):", grid_id, device.cols, device.rows);
            for y in 0..device.rows {
                let mut row = String::new();
                for x in 0..device.cols {
                    let brightness = self.led_states.get(&(grid_id, x, y)).copied().unwrap_or(0);
                    row.push_str(&format!("{:2} ", brightness));
                }
                info!("  {}", row);
            }
        }
    }
}

impl Drop for GridManager {
    fn drop(&mut self) {
        info!("Shutting down grid manager...");
        
        // Clear all grids before shutdown
        let grid_ids: Vec<usize> = self.devices.keys().copied().collect();
        for grid_id in grid_ids {
            if let Err(e) = self.clear_all(grid_id) {
                warn!("Failed to clear grid {} on shutdown: {}", grid_id, e);
            }
        }
        
        info!("Grid manager shutdown complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_grid_manager_creation() {
        let manager = GridManager::new();
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_monome_device_detection() {
        let manager = GridManager::new().unwrap();
        assert!(manager.is_monome_device(0x0A6A, 0x4001)); // Original VID with modern PID
        assert!(manager.is_monome_device(0xCAFE, 0x4001)); // Alt VID with modern PID
        assert!(!manager.is_monome_device(0x1234, 0x5678)); // Random VID/PID
    }
    
    #[test]
    fn test_grid_specs_detection() {
        let manager = GridManager::new().unwrap();
        let (cols, rows, varibright) = manager.detect_grid_specs(0x4001);
        assert_eq!(cols, 16);
        assert_eq!(rows, 8);
        assert_eq!(varibright, true);
    }
    
    #[test]
    fn test_led_operations_without_device() {
        let mut manager = GridManager::new().unwrap();
        // These should fail gracefully when no device is connected
        assert!(manager.set_led(0, 0, 0, 15).is_err());
        assert_eq!(manager.get_led(0, 0, 0), 0);
    }
}