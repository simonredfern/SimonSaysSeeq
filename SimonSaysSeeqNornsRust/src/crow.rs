//! Simple USB Serial Crow module for CV output
//!
//! This module provides direct USB serial communication with Crow hardware
//! for controlling CV outputs. Crow accepts plain Lua commands over USB serial.

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::io::Write;
use std::process::Command;
use std::time::Duration;

#[cfg(feature = "hardware")]
use serialport::SerialPort;

/// Simple Crow interface for USB serial communication
pub struct Crow {
    #[cfg(feature = "hardware")]
    port: Option<Box<dyn SerialPort>>,
    enabled: bool,
}

impl Crow {
    /// Create a new Crow interface
    pub fn new() -> Result<Self> {
        Ok(Self {
            #[cfg(feature = "hardware")]
            port: None,
            enabled: false,
        })
    }

    /// Initialize Crow by finding and opening the USB serial port
    pub fn initialize(&mut self) -> Result<()> {
        #[cfg(feature = "hardware")]
        {
            // Retry USB device detection with backoff for boot reliability
            for attempt in 1..=10 {
                info!("Crow initialization attempt {}/10", attempt);
                
                if attempt > 1 {
                    std::thread::sleep(Duration::from_secs(attempt as u64));
                }
                
                if let Ok(()) = self.try_initialize_once() {
                    return Ok(());
                }
                
                warn!("Crow initialization attempt {} failed, retrying in {} seconds...", attempt, attempt + 1);
            }
            
            warn!("Failed to initialize Crow after 10 attempts");
            return Err(anyhow!("Failed to find Crow USB serial device after retries"));
        }

        #[cfg(not(feature = "hardware"))]
        {
            warn!("Crow support disabled (hardware feature not enabled)");
            return Ok(());
        }
    }

    /// Single initialization attempt
    fn try_initialize_once(&mut self) -> Result<()> {
        #[cfg(feature = "hardware")]
        {
            // Log available USB devices for debugging
            info!("Available USB serial devices:");
            if let Ok(entries) = std::fs::read_dir("/dev") {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    if let Some(name_str) = name.to_str() {
                        if name_str.starts_with("ttyACM") || name_str.starts_with("ttyUSB") {
                            info!("  Found: /dev/{}", name_str);
                        }
                    }
                }
            }
            
            // Try to find Crow by USB vendor/product ID
            match self.find_crow_device() {
                Ok(Some(crow_path)) => {
                    info!("Found Crow device at: {}", crow_path);
                    match serialport::new(&crow_path, 115_200)
                        .timeout(Duration::from_millis(1000))
                        .open()
                    {
                        Ok(port) => {
                            info!("Crow found and opened at {}", crow_path);
                            self.port = Some(port);
                            self.enabled = true;
                            
                            // Initialize all outputs to 0V
                            self.send_command("output[1].volts = 0")?;
                            self.send_command("output[2].volts = 0")?;
                            self.send_command("output[3].volts = 0")?;
                            self.send_command("output[4].volts = 0")?;
                            
                            info!("Crow initialized - all outputs set to 0V");
                            return Ok(());
                        }
                        Err(e) => {
                            warn!("Failed to open identified Crow device {}: {}", crow_path, e);
                        }
                    }
                }
                Ok(None) => {
                    debug!("No Crow device found by USB ID, trying fallback paths");
                }
                Err(e) => {
                    warn!("Error searching for Crow device: {}", e);
                }
            }

            // Fallback: Try common paths (but skip monome grids)
            let possible_paths = [
                "/dev/ttyACM0",
                "/dev/ttyACM1",
                "/dev/ttyACM2",
                "/dev/ttyACM3",
                "/dev/ttyUSB0",
                "/dev/ttyUSB1",
            ];

            for path in &possible_paths {
                // Skip devices that are clearly monome grids
                if self.is_monome_grid(path) {
                    debug!("Skipping {} - identified as monome grid", path);
                    continue;
                }

                match serialport::new(*path, 115_200)
                    .timeout(Duration::from_millis(1000))
                    .open()
                {
                    Ok(port) => {
                        info!("Crow found and opened at {} (fallback detection)", path);
                        self.port = Some(port);
                        self.enabled = true;
                        
                        // Initialize all outputs to 0V
                        self.send_command("output[1].volts = 0")?;
                        self.send_command("output[2].volts = 0")?;
                        self.send_command("output[3].volts = 0")?;
                        self.send_command("output[4].volts = 0")?;
                        
                        info!("Crow initialized - all outputs set to 0V");
                        return Ok(());
                    }
                    Err(_) => continue,
                }
            }

            return Err(anyhow!("No suitable Crow device found"));
        }
        
        #[cfg(not(feature = "hardware"))]
        {
            return Err(anyhow!("Hardware feature not enabled"));
        }
    }

    /// Check if Crow is enabled and ready
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set all 4 CV outputs to specified voltages
    pub fn set_all_outputs(&mut self, v1: f32, v2: f32, v3: f32, v4: f32) -> Result<()> {
        if !self.enabled {
            debug!("Crow disabled - output voltages ignored");
            return Ok(());
        }

        // Clamp voltages to Crow's safe range (-5V to +10V)
        let v1 = v1.clamp(-5.0, 10.0);
        let v2 = v2.clamp(-5.0, 10.0);
        let v3 = v3.clamp(-5.0, 10.0);
        let v4 = v4.clamp(-5.0, 10.0);

        self.send_command(&format!("output[1].volts = {:.6}", v1))?;
        self.send_command(&format!("output[2].volts = {:.6}", v2))?;
        self.send_command(&format!("output[3].volts = {:.6}", v3))?;
        self.send_command(&format!("output[4].volts = {:.6}", v4))?;

        debug!("Crow outputs: {:.3}V, {:.3}V, {:.3}V, {:.3}V", v1, v2, v3, v4);
        Ok(())
    }

    /// Send a raw Lua command to Crow
    fn send_command(&mut self, lua_code: &str) -> Result<()> {
        #[cfg(feature = "hardware")]
        {
            if let Some(ref mut port) = self.port {
                // Send the Lua command followed by newline
                let command = format!("{}\n", lua_code);
                
                info!("🐦 Sending to Crow: {}", lua_code);
                
                match port.write_all(command.as_bytes()) {
                    Ok(()) => {
                        info!("✅ Command written to serial port");
                    }
                    Err(e) => {
                        error!("❌ Failed to write to Crow serial port: {}", e);
                        return Err(anyhow!("Failed to send command to Crow: {}", e));
                    }
                }
                
                match port.flush() {
                    Ok(()) => {
                        info!("✅ Serial port flushed successfully");
                    }
                    Err(e) => {
                        error!("❌ Failed to flush Crow serial port: {}", e);
                        return Err(anyhow!("Failed to flush Crow serial port: {}", e));
                    }
                }
                
                return Ok(());
            } else {
                error!("❌ Crow serial port is None");
            }
        }
        
        error!("❌ Crow serial port not available");
        Err(anyhow!("Crow serial port not available"))
    }

    #[cfg(feature = "hardware")]
    /// Find Crow device by USB vendor/product ID
    fn find_crow_device(&self) -> Result<Option<String>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg("ls /dev/ttyACM* /dev/ttyUSB* 2>/dev/null || true")
            .output()?;
        
        let devices_str = String::from_utf8_lossy(&output.stdout);
        let devices: Vec<&str> = devices_str.trim().lines().filter(|s| !s.is_empty()).collect();
        
        for device in devices {
            // Check if this device has the Crow USB IDs (STMicroelectronics Virtual COM Port)
            let udev_output = Command::new("udevadm")
                .args(&["info", "--query=all", &format!("--name={}", device)])
                .output()?;
            
            let udev_str = String::from_utf8_lossy(&udev_output.stdout);
            
            // Look for STMicroelectronics Virtual COM Port (0483:5740)
            if udev_str.contains("ID_VENDOR_ID=0483") && udev_str.contains("ID_MODEL_ID=5740") {
                return Ok(Some(device.to_string()));
            }
        }
        
        Ok(None)
    }

    #[cfg(feature = "hardware")]
    /// Check if a device is a monome grid
    fn is_monome_grid(&self, device_path: &str) -> bool {
        match Command::new("udevadm")
            .args(&["info", "--query=all", &format!("--name={}", device_path)])
            .output()
        {
            Ok(output) => {
                let udev_str = String::from_utf8_lossy(&output.stdout);
                // Check for monome vendor ID (cafe)
                udev_str.contains("ID_VENDOR_ID=cafe")
            }
            Err(_) => false,
        }
    }
}

impl Drop for Crow {
    fn drop(&mut self) {
        // Set all outputs to 0V when dropping
        if self.enabled {
            let _ = self.send_command("output[1].volts = 0");
            let _ = self.send_command("output[2].volts = 0");
            let _ = self.send_command("output[3].volts = 0");
            let _ = self.send_command("output[4].volts = 0");
        }
    }
}