//! OSC-based Grid module - Handle monome grid communication via serialosc
//!
//! This module provides proper OSC communication with monome grids through serialosc,
//! replacing the problematic raw serial communication approach.

use anyhow::{Result, anyhow};
use log::{info, debug, warn, error};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::thread;

#[cfg(feature = "rosc")]
use std::net::UdpSocket;
#[cfg(feature = "rosc")]
use std::io;
#[cfg(feature = "rosc")]
use rosc::{OscPacket, OscMessage, OscType};

/// Grid button event
#[derive(Debug, Clone)]
pub struct GridButtonEvent {
    pub grid_id: String,
    pub x: usize,
    pub y: usize,
    pub pressed: bool,
}

/// Information about a connected grid device
#[derive(Debug, Clone)]
struct GridDevice {
    id: String,
    device_type: String,
    port: u16,
    cols: usize,
    rows: usize,
    prefix: String,
    is_varibright: bool,
}

/// OSC-based Grid Manager
pub struct GridManager {
    #[cfg(feature = "rosc")]
    socket: UdpSocket,
    devices: HashMap<String, GridDevice>,
    assumed_led_states: HashMap<(String, usize, usize), u8>,
    #[cfg(feature = "rosc")]
    local_port: u16,
}

impl GridManager {
    /// Create a new OSC-based grid manager
    pub fn new() -> Result<Self> {
        #[cfg(feature = "rosc")]
        {
            info!("new says: Creating OSC-based grid manager...");

            // Create UDP socket for OSC communication
            let socket = UdpSocket::bind("127.0.0.1:0")?;
            let local_port = socket.local_addr()?.port();
            socket.set_nonblocking(true)?;

            info!("new says: Created OSC socket on port {}", local_port);

            let mut manager = Self {
                socket,
                devices: HashMap::new(),
                assumed_led_states: HashMap::new(),
                local_port,
            };

            // Discover devices via serialosc
            manager.discover_devices()?;

            Ok(manager)
        }

        #[cfg(not(feature = "rosc"))]
        {
            info!("new says: OSC grid support disabled (rosc feature not enabled)");
            Ok(Self {
                devices: HashMap::new(),
                assumed_led_states: HashMap::new(),
            })
        }
    }

    /// Discover grid devices through serialosc
    #[cfg(feature = "rosc")]
    fn discover_devices(&mut self) -> Result<()> {
        info!("discover_devices says: Discovering grid devices via serialosc...");

        // Send discovery request to serialosc server
        let list_msg = OscMessage {
            addr: "/serialosc/list".to_string(),
            args: vec![
                OscType::String("127.0.0.1".to_string()),
                OscType::Int(self.local_port as i32),
            ],
        };

        let packet = OscPacket::Message(list_msg);
        let msg_buf = rosc::encoder::encode(&packet)?;

        self.socket.send_to(&msg_buf, "127.0.0.1:12002")
            .map_err(|e| anyhow!("Failed to send discovery request to serialosc: {}", e))?;

        info!("discover_devices says: Sent device discovery request to serialosc");

        // Wait for device responses
        let discovery_timeout = Duration::from_secs(3);
        let discovery_start = Instant::now();
        let mut discovered_devices = Vec::new();

        while discovery_start.elapsed() < discovery_timeout {
            let mut buf = [0u8; rosc::decoder::MTU];
            match self.socket.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                        if let OscPacket::Message(msg) = packet {
                            if msg.addr == "/serialosc/device" && msg.args.len() >= 3 {
                                if let (Some(OscType::String(id)), Some(OscType::String(device_type)), Some(OscType::Int(port))) =
                                    (msg.args.get(0), msg.args.get(1), msg.args.get(2)) {
                                    info!("discover_devices says: Found serialosc device: {} (type: {}) on port {}", id, device_type, port);
                                    discovered_devices.push((id.clone(), device_type.clone(), *port as u16));
                                }
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Timeout, continue waiting
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    warn!("discover_devices says: Error receiving OSC discovery response: {}", e);
                }
            }
        }

        if discovered_devices.is_empty() {
            info!("discover_devices says: No serialosc devices found. Make sure:");
            info!("discover_devices says:   1. Your grid is connected via USB");
            info!("discover_devices says:   2. serialosc is running: sudo systemctl start serialosc");
            info!("discover_devices says:   3. You're in the dialout group");
            return Ok(());
        }

        // Connect to each discovered device
        for (device_id, device_type, device_port) in discovered_devices {
            if let Err(e) = self.connect_to_device(&device_id, &device_type, device_port) {
                warn!("discover_devices says: Failed to connect to device {}: {}", device_id, e);
            }
        }

        info!("discover_devices says: Connected to {} grid device(s) via OSC", self.devices.len());
        Ok(())
    }

    /// Connect to a specific grid device and get its configuration
    #[cfg(feature = "rosc")]
    fn connect_to_device(&mut self, device_id: &str, device_type: &str, device_port: u16) -> Result<()> {
        info!("connect_to_device says: Connecting to grid device: {} ({})", device_id, device_type);

        let device_addr = format!("127.0.0.1:{}", device_port);

        // Request device info to get size and other details
        let info_msg = OscMessage {
            addr: "/sys/info".to_string(),
            args: vec![
                OscType::String("127.0.0.1".to_string()),
                OscType::Int(self.local_port as i32),
            ],
        };

        let packet = OscPacket::Message(info_msg);
        let msg_buf = rosc::encoder::encode(&packet)?;
        self.socket.send_to(&msg_buf, &device_addr)?;

        // Also explicitly request key events to be sent to our port
        let key_msg = OscMessage {
            addr: "/sys/port".to_string(),
            args: vec![OscType::Int(self.local_port as i32)],
        };

        let key_packet = OscPacket::Message(key_msg);
        let key_msg_buf = rosc::encoder::encode(&key_packet)?;
        self.socket.send_to(&key_msg_buf, &device_addr)?;

        info!("connect_to_device says: Requested grid {} to send key events to port {}", device_id, self.local_port);

        // Wait for device info response
        let mut cols = 16; // Default
        let mut rows = 8;  // Default
        let mut prefix = "/monome".to_string(); // Default to /monome (more common)
        let mut received_size = false;
        let mut received_prefix = false;

        let info_timeout = Duration::from_millis(1000);
        let info_start = Instant::now();

        while info_start.elapsed() < info_timeout && (!received_size || !received_prefix) {
            let mut buf = [0u8; rosc::decoder::MTU];
            match self.socket.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                        if let OscPacket::Message(msg) = packet {
                            match msg.addr.as_str() {
                                "/sys/size" => {
                                    if let (Some(OscType::Int(c)), Some(OscType::Int(r))) =
                                        (msg.args.get(0), msg.args.get(1)) {
                                        cols = *c as usize;
                                        rows = *r as usize;
                                        received_size = true;
                                        debug!("connect_to_device says: Device {} size: {}x{}", device_id, cols, rows);
                                    }
                                }
                                "/sys/prefix" => {
                                    if let Some(OscType::String(p)) = msg.args.get(0) {
                                        prefix = p.clone();
                                        received_prefix = true;
                                        debug!("connect_to_device says: Device {} prefix: {}", device_id, prefix);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    warn!("connect_to_device says: Error receiving device info: {}", e);
                    break;
                }
            }
        }

        if !received_size {
            warn!("connect_to_device says: Did not receive size info for device {}, using defaults", device_id);
        }

        if !received_prefix {
            warn!("connect_to_device says: Did not receive prefix info for device {}, using default: {}", device_id, prefix);
        }

        // Determine if device supports variable brightness
        let is_varibright = device_type.contains("128") || device_type.contains("256") ||
                           device_type.contains("one"); // Most modern grids support varibright

        let grid_device = GridDevice {
            id: device_id.to_string(),
            device_type: device_type.to_string(),
            port: device_port,
            cols,
            rows,
            prefix: prefix.clone(),
            is_varibright,
        };

        // Insert device first, before trying to access it
        self.devices.insert(device_id.to_string(), grid_device);

        // Initialize LED state tracking
        for x in 0..cols {
            for y in 0..rows {
                self.assumed_led_states.insert((device_id.to_string(), x, y), 0);
            }
        }

        // Initialize the grid (clear all LEDs) - now device exists
        if let Err(e) = self.clear_all(device_id) {
            warn!("connect_to_device says: Failed to clear grid {} during initialization: {}", device_id, e);
        }

        info!("connect_to_device says: Grid {} connected: {} ({}x{}, varibright: {})",
              device_id, device_type, cols, rows, is_varibright);

        Ok(())
    }

    /// Set a single LED
    pub fn set_led(&mut self, grid_id: &str, x: usize, y: usize, brightness: u8, caller: &str) -> Result<()> {
        // Use native 0-based coordinates directly
        let seq_x = x;
        let seq_y = y;

        // Only allow LED updates for rows 0 and 1 (0-indexed)
        if seq_y != 0 && seq_y != 1 {
            info!("set_led says: LED update blocked for row {} (display: row {}): {} (caller: {})", seq_y, seq_y + 1, format!("({}, {})", seq_x, seq_y), caller);
            return Ok(());
        }


        //debug!("set_led says: LED update row {}: ({}, {}) brightness={} (caller: {})", seq_y, seq_x, seq_y, brightness, caller);


        #[cfg(not(feature = "rosc"))]
        {
            return Err(anyhow!("OSC feature not enabled"));
        }

        #[cfg(feature = "rosc")]
        {
        let device = self.devices.get(grid_id)
            .ok_or_else(|| anyhow!("Grid {} not found", grid_id))?;

        // Validate coordinates
        if x >= device.cols || y >= device.rows {
            return Err(anyhow!("LED coordinates ({}, {}) out of bounds for grid {} ({}x{})",
                             x, y, grid_id, device.cols, device.rows));
        }

        // Clamp brightness
        let brightness = if device.is_varibright {
            brightness.min(15)
        } else {
            if brightness > 0 { 1 } else { 0 }
        };

        // Update internal state
        self.assumed_led_states.insert((grid_id.to_string(), x, y), brightness);

        // Send OSC command
        let device_addr = format!("127.0.0.1:{}", device.port);
        let prefix = &device.prefix;

        let osc_msg = if device.is_varibright {
            // Use level command for variable brightness
            OscMessage {
                addr: format!("{}/grid/led/level/set", prefix),
                args: vec![
                    OscType::Int(x as i32),
                    OscType::Int(y as i32),
                    OscType::Int(brightness as i32),
                ],
            }
        } else {
            // Use basic on/off command
            OscMessage {
                addr: format!("{}/grid/led/set", prefix),
                args: vec![
                    OscType::Int(x as i32),
                    OscType::Int(y as i32),
                    OscType::Int(if brightness > 0 { 1 } else { 0 }),
                ],
            }
        };

        //debug!("set_led says: Sending OSC command: {} to {}", osc_msg.addr, device_addr);
        //debug!("set_led says: OSC args: {:?}", osc_msg.args);

        let packet = OscPacket::Message(osc_msg);
        let msg_buf = rosc::encoder::encode(&packet)?;
        self.socket.send_to(&msg_buf, &device_addr)?;

        debug!("set_led says: After socket.send_to Set LED grid {} ({}, {}) = {}", grid_id, x, y, brightness);

        Ok(())
        }
    }

    /// Get current LED state
    pub fn get_led(&self, grid_id: &str, x: usize, y: usize) -> u8 {
        self.assumed_led_states.get(&(grid_id.to_string(), x, y)).copied().unwrap_or(0)
    }

    /// Clear all LEDs on a grid
    pub fn clear_all(&mut self, grid_id: &str) -> Result<()> {
        #[cfg(not(feature = "rosc"))]
        {
            return Err(anyhow!("OSC feature not enabled"));
        }

        #[cfg(feature = "rosc")]
        {
        let device = self.devices.get(grid_id)
            .ok_or_else(|| anyhow!("Grid {} not found", grid_id))?;

        // Clear internal state
        for x in 0..device.cols {
            for y in 0..device.rows {
                self.assumed_led_states.insert((grid_id.to_string(), x, y), 0);
            }
        }

        // Send OSC clear command
        let device_addr = format!("127.0.0.1:{}", device.port);
        let prefix = &device.prefix;

        // Try both basic and level clear commands for maximum compatibility
        let clear_commands = if device.is_varibright {
            vec![
                format!("{}/grid/led/level/all", prefix),
                format!("{}/grid/led/all", prefix),
            ]
        } else {
            vec![format!("{}/grid/led/all", prefix)]
        };

        for addr in clear_commands {
            let osc_msg = OscMessage {
                addr: addr.clone(),
                args: vec![OscType::Int(0)],
            };

            debug!("Sending clear command: {} to {}", addr, device_addr);
            debug!("Clear args: {:?}", osc_msg.args);

            let packet = OscPacket::Message(osc_msg);
            let msg_buf = rosc::encoder::encode(&packet)?;
            self.socket.send_to(&msg_buf, &device_addr)?;

            // Small delay between commands
            thread::sleep(Duration::from_millis(50));
        }

        debug!("Cleared all LEDs on grid {}", grid_id);
        Ok(())
        }
    }

    /// Set LED map for efficient bulk updates
    pub fn set_led_map(&mut self, grid_id: &str, led_map: &[Vec<u8>]) -> Result<()> {
        let (cols, rows) = {
            let device = self.devices.get(grid_id)
                .ok_or_else(|| anyhow!("Grid {} not found", grid_id))?;
            (device.cols, device.rows)
        };

        if led_map.len() != cols {
            return Err(anyhow!("LED map width {} doesn't match grid width {}",
                             led_map.len(), cols));
        }

        // Update internal state and send individual LED commands
        // For now, we'll use individual LED commands rather than the more complex map format
        for (x, column) in led_map.iter().enumerate() {
            if column.len() != rows {
                return Err(anyhow!("LED map column {} height {} doesn't match grid height {}",
                                 x, column.len(), rows));
            }

            for (y, &brightness) in column.iter().enumerate() {
                self.set_led(grid_id, x, y, brightness, "set_led_map")?;

                // Add small delay every 8 LEDs to prevent overwhelming
                if (x * rows + y) % 8 == 0 {
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }

        debug!("Updated LED map for grid {} ({}x{})", grid_id, cols, rows);
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
        self.devices.keys().cloned().collect()
    }

    /// Read button events from grids
    pub fn read_button_events(&mut self) -> Result<Vec<GridButtonEvent>> {
        #[cfg(not(feature = "rosc"))]
        {
            return Ok(Vec::new());
        }

        #[cfg(feature = "rosc")]
        {
        let mut events = Vec::new();

        // Check for incoming OSC messages from grids (non-blocking)
        loop {
            let mut buf = [0u8; rosc::decoder::MTU];
            match self.socket.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                        if let OscPacket::Message(msg) = packet {
                            // Parse grid key events
                            for device in self.devices.values() {
                                let key_addr = format!("{}/grid/key", device.prefix);
                                if msg.addr == key_addr && msg.args.len() >= 3 {
                                    if let (Some(OscType::Int(x)), Some(OscType::Int(y)), Some(OscType::Int(state))) =
                                        (msg.args.get(0), msg.args.get(1), msg.args.get(2)) {
                                        events.push(GridButtonEvent {
                                            grid_id: device.id.clone(),
                                            x: *x as usize,
                                            y: *y as usize,
                                            pressed: *state != 0,
                                        });
                                        debug!("Grid {} button event: ({}, {}) = {}",
                                               device.id, x, y, *state != 0);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No more messages available - this is expected with non-blocking socket
                    break;
                }
                Err(e) => {
                    return Err(anyhow!("Error reading OSC messages: {}", e));
                }
            }
        }

        Ok(events)
        }
    }

    /// Test grid functionality with safe OSC commands
    pub fn test_grid(&mut self, grid_id: &str) -> Result<()> {
        #[cfg(not(feature = "rosc"))]
        {
            return Err(anyhow!("OSC feature not enabled"));
        }

        #[cfg(feature = "rosc")]
        {
        let (cols, rows, device_port, prefix, is_varibright) = {
            let device = self.devices.get(grid_id)
                .ok_or_else(|| anyhow!("Grid {} not found", grid_id))?;
            (device.cols, device.rows, device.port, device.prefix.clone(), device.is_varibright)
        };

        info!("Testing grid {} functionality (OSC mode)...", grid_id);

        let total_leds = cols * rows;

        // Safety check - limit test to reasonable grid sizes
        if total_leds > 256 {
            warn!("Grid {} too large for full test ({} LEDs), doing corner test only", grid_id, total_leds);
            return self.test_corners_only(grid_id);
        }

        // Safe test sequence
        info!("Running safe OSC test on {} LEDs...", total_leds);

        // Test 1: Corner flash
        info!("  Step 1: Testing corners...");
        let corners = [(0, 0), (cols-1, 0), (0, rows-1), (cols-1, rows-1)];
        for &(x, y) in &corners {
            self.set_led(grid_id, x, y, 10, "test_grid_functionality")?;
            thread::sleep(Duration::from_millis(150));
        }
        thread::sleep(Duration::from_millis(500));

        // Clear corners
        for &(x, y) in &corners {
            self.set_led(grid_id, x, y, 0, "test_grid_functionality")?;
            thread::sleep(Duration::from_millis(100));
        }

        // Test 2: Brief all-LED flash (but only once)
        info!("  Step 2: Brief flash test...");

        // Use OSC bulk command for efficiency
        let device_addr = format!("127.0.0.1:{}", device_port);

        if is_varibright {
            // Flash with low brightness
            let flash_msg = OscMessage {
                addr: format!("{}/grid/led/level/all", prefix),
                args: vec![OscType::Int(5)], // Low brightness
            };
            let packet = OscPacket::Message(flash_msg);
            let msg_buf = rosc::encoder::encode(&packet)?;
            self.socket.send_to(&msg_buf, &device_addr)?;
        } else {
            // Flash on/off
            let flash_msg = OscMessage {
                addr: format!("{}/grid/led/all", prefix),
                args: vec![OscType::Int(1)], // On
            };
            debug!("Sending flash command: {} to {}", flash_msg.addr, device_addr);
            debug!("Flash args: {:?}", flash_msg.args);

            let packet = OscPacket::Message(flash_msg);
            let msg_buf = rosc::encoder::encode(&packet)?;
            self.socket.send_to(&msg_buf, &device_addr)?;
        }

        thread::sleep(Duration::from_millis(200));

        // Clear all
        info!("  Step 3: Final cleanup...");
        self.clear_all(grid_id)?;

        info!("Grid {} OSC test completed successfully", grid_id);
        Ok(())
        }
    }

    /// Test only corners for large grids
    fn test_corners_only(&mut self, grid_id: &str) -> Result<()> {
        #[cfg(not(feature = "rosc"))]
        {
            return Err(anyhow!("OSC feature not enabled"));
        }

        #[cfg(feature = "rosc")]
        {
        let (cols, rows) = {
            let device = self.devices.get(grid_id)
                .ok_or_else(|| anyhow!("Grid {} not found", grid_id))?;
            (device.cols, device.rows)
        };

        info!("Testing corners only for large grid {}...", grid_id);

        let corners = [(0, 0), (cols-1, 0), (0, rows-1), (cols-1, rows-1)];

        // Light up corners
        for &(x, y) in &corners {
            self.set_led(grid_id, x, y, 8, "comprehensive_grid_test")?;
            thread::sleep(Duration::from_millis(200));
        }

        thread::sleep(Duration::from_millis(800));

        // Clear corners
        for &(x, y) in &corners {
            self.set_led(grid_id, x, y, 0, "comprehensive_grid_test")?;
            thread::sleep(Duration::from_millis(100));
        }

        // Final clear
        self.clear_all(grid_id)?;

        info!("Corner test completed for grid {}", grid_id);
        Ok(())
        }
    }

    /// Flash all connected grids
    pub fn flash_all_grids(&mut self) -> Result<()> {
        for grid_id in self.get_connected_grids() {
            self.flash_grid(&grid_id)?;
        }
        Ok(())
    }

    /// Flash a specific grid
    pub fn flash_grid(&mut self, grid_id: &str) -> Result<()> {
        #[cfg(not(feature = "rosc"))]
        {
            return Err(anyhow!("OSC feature not enabled"));
        }

        #[cfg(feature = "rosc")]
        {
        let (device_port, prefix, is_varibright) = {
            let device = self.devices.get(grid_id)
                .ok_or_else(|| anyhow!("Grid {} not found", grid_id))?;
            (device.port, device.prefix.clone(), device.is_varibright)
        };

        let device_addr = format!("127.0.0.1:{}", device_port);

        // Flash sequence
        let brightness_levels = if is_varibright {
            vec![8, 0, 8, 0]
        } else {
            vec![1, 0, 1, 0]
        };

        for brightness in brightness_levels {
            let flash_msg = if is_varibright {
                OscMessage {
                    addr: format!("{}/grid/led/level/all", prefix),
                    args: vec![OscType::Int(brightness)],
                }
            } else {
                OscMessage {
                    addr: format!("{}/grid/led/all", prefix),
                    args: vec![OscType::Int(brightness)],
                }
            };

            debug!("Sending flash command: {} to {}", flash_msg.addr, device_addr);
            debug!("Flash args: {:?}", flash_msg.args);

            let packet = OscPacket::Message(flash_msg);
            let msg_buf = rosc::encoder::encode(&packet)?;
            self.socket.send_to(&msg_buf, &device_addr)?;

            thread::sleep(Duration::from_millis(150));
        }

        debug!("Flashed grid {}", grid_id);
        Ok(())
        }
    }

    /// Refresh/update a grid display
    pub fn refresh(&mut self) -> Result<()> {
        // For OSC, we don't need explicit refresh - commands are sent immediately
        Ok(())
    }

    /// Print diagnostic information about connected grids
    pub fn print_grid_info(&self) {
        info!("Connected grids via OSC:");
        for (id, device) in &self.devices {
            info!("  {}: {} ({}x{}, port {}, varibright: {})",
                  id, device.device_type, device.cols, device.rows,
                  device.port, device.is_varibright);
        }
    }
}

impl Drop for GridManager {
    fn drop(&mut self) {
        info!("Shutting down OSC grid manager...");

        // Clear all grids before shutdown
        for grid_id in self.get_connected_grids() {
            if let Err(e) = self.clear_all(&grid_id) {
                warn!("Failed to clear grid {} during shutdown: {}", grid_id, e);
            }
        }

        // Give time for final commands to be processed
        #[cfg(feature = "rosc")]
        thread::sleep(Duration::from_millis(100));

        info!("OSC grid manager shutdown complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osc_grid_manager_creation() {
        // This test will only pass if serialosc is running
        // In practice, this would be an integration test
    }
}
