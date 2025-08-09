//! Grid module - Handle monome grid communication and LED control
//! 
//! Provides interface to monome grid devices via USB HID communication.

use anyhow::{Result, anyhow};
#[cfg(feature = "hardware")]
use hidapi::{HidApi, HidDevice};
use log::{info, debug, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};


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
    device: Arc<Mutex<HidDevice>>,
    rows: usize,
    cols: usize,
    name: String,
    is_varibright: bool,
}

/// Grid manager handles multiple grid devices
pub struct GridManager {
    devices: HashMap<usize, GridDevice>,
    led_states: HashMap<(usize, usize, usize), u8>, // (grid_id, x, y) -> brightness
    #[cfg(feature = "hardware")]
    hid_api: HidApi,
}

impl GridManager {
    /// Create a new grid manager
    pub fn new() -> Result<Self> {
        #[cfg(feature = "hardware")]
        {
            let mut manager = Self {
                devices: HashMap::new(),
                led_states: HashMap::new(),
                hid_api: HidApi::new()?,
            };
            manager.discover_grids()?;
            Ok(manager)
        }
        
        #[cfg(not(feature = "hardware"))]
        {
            info!("Grid simulation mode - no actual grid devices");
            Ok(Self {
                devices: HashMap::new(),
                led_states: HashMap::new(),
            })
        }
    }
    
    /// Discover and connect to grid devices
    #[cfg(feature = "hardware")]
    fn discover_grids(&mut self) -> Result<()> {
        info!("Discovering monome grid devices...");
        
        // First, collect all the grid device info we need
        let mut grid_devices_to_add = Vec::new();
        
        for device_info in self.hid_api.device_list() {
            let vid = device_info.vendor_id();
            let pid = device_info.product_id();
            
            if self.is_monome_device(vid, pid) {
                match device_info.open_device(&self.hid_api) {
                    Ok(device) => {
                        let (cols, rows, is_varibright) = Self::detect_grid_specs_static(pid);
                        
                        let grid_device = GridDevice {
                            device: Arc::new(Mutex::new(device)),
                            rows,
                            cols,
                            name: device_info.product_string().unwrap_or("Unknown Grid").to_string(),
                            is_varibright,
                        };
                        
                        grid_devices_to_add.push((grid_device, cols, rows, device_info.product_string().unwrap_or("Unknown").to_string()));
                    }
                    Err(e) => {
                        warn!("Failed to open grid device: {}", e);
                    }
                }
            }
        }
        
        // Now add all the devices we found
        let mut grid_count = 0;
        for (grid_device, cols, rows, name) in grid_devices_to_add {
            let grid_id = grid_count;
            self.devices.insert(grid_id, grid_device);
            
            // Initialize LED state tracking
            for x in 0..cols {
                for y in 0..rows {
                    self.led_states.insert((grid_id, x, y), 0);
                }
            }
            
            self.initialize_grid(grid_id)?;
            
            info!("Connected to grid {}: {} ({}x{}, varibright: {})", 
                  grid_id, name, cols, rows, self.devices[&grid_id].is_varibright);
            
            grid_count += 1;
        }
        
        info!("Found {} grid device(s)", grid_count);
        Ok(())
    }
    
    /// Check if a device is a monome grid
    #[cfg(feature = "hardware")]
    fn is_monome_device(&self, vid: u16, pid: u16) -> bool {
        // Monome vendor ID
        const MONOME_VID: u16 = 0x0A6A;
        
        // Known monome product IDs
        const MONOME_PIDS: &[u16] = &[
            0x0001, // 40h
            0x0002, // 64
            0x0003, // 128
            0x0004, // 256
            0x0011, // mk series 64
            0x0012, // mk series 128
            0x0013, // mk series 256
        ];
        
        vid == MONOME_VID && MONOME_PIDS.contains(&pid)
    }
    
    /// Detect grid specifications based on product ID
    #[cfg(feature = "hardware")]
    fn detect_grid_specs(&self, pid: u16) -> (usize, usize, bool) {
        Self::detect_grid_specs_static(pid)
    }
    
    /// Static version of detect_grid_specs for use during discovery
    #[cfg(feature = "hardware")]
    fn detect_grid_specs_static(pid: u16) -> (usize, usize, bool) {
        match pid {
            0x0001 => (8, 8, false),   // 40h - 8x8, no varibright
            0x0002 => (8, 8, false),   // 64 - 8x8, no varibright
            0x0003 => (16, 8, false),  // 128 - 16x8, no varibright
            0x0004 => (16, 16, false), // 256 - 16x16, no varibright
            0x0011 => (8, 8, true),    // mk series - 8x8, varibright
            0x0012 => (16, 8, true),   // mk series - 16x8, varibright
            0x0013 => (16, 16, true),  // mk series - 16x16, varibright
            _ => (16, 8, true),        // Default to 128 mk specs
        }
    }
    
    /// Initialize a grid device
    #[cfg(feature = "hardware")]
    fn initialize_grid(&mut self, grid_id: usize) -> Result<()> {
        if let Some(device) = self.devices.get(&grid_id) {
            let device_lock = device.device.lock().unwrap();
            
            // Send initialization command (clear all LEDs)
            let clear_cmd = [0x1A, 0x00, 0x00]; // Clear all command
            if let Err(e) = device_lock.write(&clear_cmd) {
                warn!("Failed to send clear command to grid {}: {}", grid_id, e);
            } else {
                debug!("Grid {} initialized and cleared", grid_id);
            }
        }
        
        Ok(())
    }
    
    /// Set LED brightness at specific coordinates
    pub fn set_led(&mut self, grid_id: usize, x: usize, y: usize, brightness: u8) -> Result<()> {
        // Update our state tracking
        self.led_states.insert((grid_id, x, y), brightness);
        
        #[cfg(feature = "hardware")]
        {
            if let Some(device) = self.devices.get(&grid_id) {
                // Validate coordinates
                if x >= device.cols || y >= device.rows {
                    return Err(anyhow!("Grid coordinates out of bounds: ({}, {}) for grid {}", x, y, grid_id));
                }
                
                // Clamp brightness based on device capabilities
                let max_brightness = if device.is_varibright { 15 } else { 1 };
                let clamped_brightness = brightness.min(max_brightness);
                
                // Send LED command
                let device_lock = device.device.lock().unwrap();
                
                if device.is_varibright {
                    // Varibright command: [0x11, x, y, brightness]
                    let cmd = [0x11, x as u8, y as u8, clamped_brightness];
                    device_lock.write(&cmd).map_err(|e| anyhow!("Failed to set LED: {}", e))?;
                } else {
                    // Binary LED command: [0x10, x, y, on/off]
                    let on_off = if clamped_brightness > 0 { 1 } else { 0 };
                    let cmd = [0x10, x as u8, y as u8, on_off];
                    device_lock.write(&cmd).map_err(|e| anyhow!("Failed to set LED: {}", e))?;
                }
                
                debug!("Set LED grid:{} ({}, {}) = {}", grid_id, x, y, clamped_brightness);
            } else {
                return Err(anyhow!("Grid {} not found", grid_id));
            }
        }
        
        #[cfg(not(feature = "hardware"))]
        {
            debug!("Set LED (simulation) grid:{} ({}, {}) = {}", grid_id, x, y, brightness);
            self.print_grid_state();
        }
        
        Ok(())
    }
    
    /// Get LED brightness at specific coordinates
    pub fn get_led(&self, grid_id: usize, x: usize, y: usize) -> Option<u8> {
        self.led_states.get(&(grid_id, x, y)).copied()
    }
    
    /// Set multiple LEDs efficiently
    pub fn set_led_map(&mut self, grid_id: usize, led_map: &[u8]) -> Result<()> {
        #[cfg(feature = "hardware")]
        {
            if let Some(device) = self.devices.get(&grid_id) {
                let expected_size = device.cols * device.rows;
                if led_map.len() != expected_size {
                    return Err(anyhow!("LED map size mismatch: expected {}, got {}", expected_size, led_map.len()));
                }
                
                let device_lock = device.device.lock().unwrap();
                
                if device.is_varibright {
                    // Send varibright map command
                    let mut cmd = vec![0x1A, 0x00, 0x00]; // Map command header
                    cmd.extend_from_slice(led_map);
                    device_lock.write(&cmd).map_err(|e| anyhow!("Failed to set LED map: {}", e))?;
                } else {
                    // For binary devices, convert to packed format
                    let mut packed_data = Vec::new();
                    for chunk in led_map.chunks(8) {
                        let mut byte = 0u8;
                        for (i, &brightness) in chunk.iter().enumerate() {
                            if brightness > 0 {
                                byte |= 1 << i;
                            }
                        }
                        packed_data.push(byte);
                    }
                    
                    let mut cmd = vec![0x1A, 0x00, 0x00]; // Map command header
                    cmd.extend_from_slice(&packed_data);
                    device_lock.write(&cmd).map_err(|e| anyhow!("Failed to set LED map: {}", e))?;
                }
                
                debug!("Set LED map for grid {}", grid_id);
            } else {
                return Err(anyhow!("Grid {} not found", grid_id));
            }
        }
        
        #[cfg(not(feature = "hardware"))]
        {
            debug!("Set LED map (simulation) for grid {} with {} LEDs", grid_id, led_map.len());
        }
        
        Ok(())
    }
    
    /// Clear all LEDs on a grid
    pub fn clear_all(&mut self, grid_id: usize) -> Result<()> {
        #[cfg(feature = "hardware")]
        {
            if let Some(device) = self.devices.get(&grid_id) {
                let device_lock = device.device.lock().unwrap();
                
                // Send clear all command
                let clear_cmd = [0x1A, 0x00, 0x00]; // Clear all command
                device_lock.write(&clear_cmd).map_err(|e| anyhow!("Failed to clear grid: {}", e))?;
                
                // Update our state tracking
                for x in 0..device.cols {
                    for y in 0..device.rows {
                        self.led_states.insert((grid_id, x, y), 0);
                    }
                }
                
                debug!("Cleared all LEDs on grid {}", grid_id);
            } else {
                return Err(anyhow!("Grid {} not found", grid_id));
            }
        }
        
        #[cfg(not(feature = "hardware"))]
        {
            debug!("Clear all LEDs (simulation) on grid {}", grid_id);
            // Clear our state tracking
            self.led_states.retain(|(gid, _, _), _| *gid != grid_id);
        }
        
        Ok(())
    }
    
    /// Refresh grid display (no-op for most grids, but useful for some)
    pub fn refresh(&mut self, grid_id: usize) -> Result<()> {
        debug!("Refresh grid {}", grid_id);
        Ok(())
    }
    
    /// Get grid dimensions
    pub fn get_dimensions(&self, grid_id: usize) -> Option<(usize, usize)> {
        self.devices.get(&grid_id).map(|device| (device.cols, device.rows))
    }
    
    /// Get grid name
    pub fn get_name(&self, grid_id: usize) -> Option<&str> {
        self.devices.get(&grid_id).map(|device| device.name.as_str())
    }
    
    /// Check if grid supports varibright
    pub fn is_varibright(&self, grid_id: usize) -> bool {
        self.devices.get(&grid_id).map(|device| device.is_varibright).unwrap_or(false)
    }
    
    /// Get list of connected grid IDs
    pub fn get_connected_grids(&self) -> Vec<usize> {
        self.devices.keys().cloned().collect()
    }
    
    /// Read button events from all grids (blocking)
    pub fn read_button_events(&mut self) -> Result<Vec<GridButtonEvent>> {
        let mut events = Vec::new();
        
        #[cfg(feature = "hardware")]
        {
            for (&grid_id, device) in &self.devices {
                let device_lock = device.device.lock().unwrap();
                
                // Try to read from the device (non-blocking)
                let mut buf = [0u8; 64];
                match device_lock.read_timeout(&mut buf, 0) {
                    Ok(size) if size > 0 => {
                        // Parse button events from the buffer
                        // Format: [0x00, x, y, pressed]
                        if size >= 4 && buf[0] == 0x00 {
                            let x = buf[1] as usize;
                            let y = buf[2] as usize;
                            let pressed = buf[3] != 0;
                            
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
                        }
                    }
                    Ok(_) => {
                        // No data available
                    }
                    Err(e) => {
                        warn!("Error reading from grid {}: {}", grid_id, e);
                    }
                }
            }
        }
        
        Ok(events)
    }
    
    /// Update step cursor display on all grids
    pub fn update_step_cursor(&mut self, step: usize, _bar: usize) -> Result<()> {
        let grid_ids: Vec<usize> = self.devices.keys().cloned().collect();
        
        for grid_id in grid_ids {
            if let Some(device) = self.devices.get(&grid_id) {
                let cols = device.cols;
                let rows = device.rows;
                
                // Clear previous step cursor (bottom row)
                for x in 0..cols {
                    if rows > 0 {
                        self.set_led(grid_id, x, rows - 1, 0)?;
                    }
                }
                
                // Set current step cursor
                if step > 0 && step <= cols && rows > 0 {
                    self.set_led(grid_id, step - 1, rows - 1, 4)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Test grid by lighting up LEDs in a pattern
    pub fn test_grid(&mut self, grid_id: usize) -> Result<()> {
        if let Some(device) = self.devices.get(&grid_id) {
            let cols = device.cols;
            let rows = device.rows;
            let max_brightness = if device.is_varibright { 15 } else { 1 };
            
            info!("Testing grid {} ({}x{})", grid_id, cols, rows);
            
            // Light up all LEDs briefly
            for x in 0..cols.min(8) {
                for y in 0..rows.min(4) {
                    self.set_led(grid_id, x, y, max_brightness)?;
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    self.set_led(grid_id, x, y, 0)?;
                }
            }
            
            info!("Grid {} test completed", grid_id);
        } else {
            warn!("Grid {} not found for testing", grid_id);
        }
        
        Ok(())
    }

    /// Print visual representation of grid state (simulation mode only)
    #[cfg(not(feature = "hardware"))]
    fn print_grid_state(&self) {
        // Clear screen and move cursor to top
        print!("\x1B[2J\x1B[1;1H");
        println!("Grid State (brightness 0-15):");
        println!("┌─────┬─────┬─────┐");
        for y in 0..4 {
            print!("│");
            for x in 0..3 {
                let brightness = self.led_states.get(&(0, x, y)).unwrap_or(&0);
                if *brightness > 0 {
                    print!(" ■{:2} ", brightness);
                } else {
                    print!(" □ 0 ");
                }
                print!("│");
            }
            println!();
            if y < 3 { 
                println!("├─────┼─────┼─────┤"); 
            }
        }
        println!("└─────┴─────┴─────┘");
        println!("Numpad mapping:");
        println!("  7 8 9");
        println!("  4 5 6");
        println!("  1 2 3");
        println!("    0");
        println!("Press numpad key + Enter to toggle, 'q' + Enter to quit");
    }
}

impl Drop for GridManager {
    fn drop(&mut self) {
        // Clear all grids before dropping
        let grid_ids: Vec<usize> = self.devices.keys().cloned().collect();
        for grid_id in grid_ids {
            let _ = self.clear_all(grid_id);
        }
        info!("Grid manager dropped, all grids cleared");
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
    
    #[cfg(feature = "hardware")]
    #[test]
    fn test_monome_device_detection() {
        let manager = GridManager::new().unwrap();
        assert!(manager.is_monome_device(0x0A6A, 0x0012)); // monome 128
        assert!(!manager.is_monome_device(0x1234, 0x5678)); // random device
    }
    
    #[cfg(feature = "hardware")]
    #[test]
    fn test_grid_specs_detection() {
        let manager = GridManager::new().unwrap();
        let (cols, rows, varibright) = manager.detect_grid_specs(0x0012);
        assert_eq!(cols, 16);
        assert_eq!(rows, 8);
        assert!(varibright);
    }
    
    #[test]
    fn test_led_operations_without_device() {
        let mut manager = GridManager::new().unwrap();
        // These should not panic even without actual devices
        assert!(manager.set_led(0, 0, 0, 5).is_ok());
        assert!(manager.clear_all(0).is_ok());
    }
}