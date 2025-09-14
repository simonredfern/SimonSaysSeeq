//! Test utility to identify and test Crow USB serial connection
//! 
//! This utility helps identify which /dev/ttyACM* device is the Crow
//! and tests basic communication with it.

use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use std::io::{Read, Write};
use std::process::Command;
use std::time::Duration;

#[cfg(feature = "hardware")]
use serialport::SerialPort;

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🔍 Crow USB Serial Test Utility");
    info!("===============================");
    
    // First, let's identify all ttyACM devices and their USB info
    list_ttyacm_devices()?;
    
    // Try to find and test Crow specifically
    test_crow_detection()?;
    
    Ok(())
}

fn list_ttyacm_devices() -> Result<()> {
    info!("\n📋 Listing all ttyACM devices:");
    
    let output = Command::new("sh")
        .arg("-c")
        .arg("ls /dev/ttyACM* 2>/dev/null || echo 'No ttyACM devices found'")
        .output()?;
    
    let devices_str = String::from_utf8_lossy(&output.stdout);
    let devices: Vec<&str> = devices_str.trim().lines().collect();
    
    if devices.is_empty() || devices[0] == "No ttyACM devices found" {
        warn!("❌ No ttyACM devices found");
        return Ok(());
    }
    
    for device in devices {
        info!("\n🔌 Device: {}", device);
        
        // Get USB vendor/product info
        let udev_output = Command::new("udevadm")
            .args(&["info", "--query=all", &format!("--name={}", device)])
            .output()?;
        
        let udev_str = String::from_utf8_lossy(&udev_output.stdout);
        
        let mut vendor_id = String::new();
        let mut model_id = String::new();
        let mut vendor_name = String::new();
        let mut model_name = String::new();
        
        for line in udev_str.lines() {
            if line.contains("ID_VENDOR_ID=") {
                vendor_id = line.split('=').nth(1).unwrap_or("").to_string();
            } else if line.contains("ID_MODEL_ID=") {
                model_id = line.split('=').nth(1).unwrap_or("").to_string();
            } else if line.contains("ID_VENDOR=") && !line.contains("ID_VENDOR_ID") {
                vendor_name = line.split('=').nth(1).unwrap_or("").to_string();
            } else if line.contains("ID_MODEL=") && !line.contains("ID_MODEL_ID") {
                model_name = line.split('=').nth(1).unwrap_or("").to_string();
            }
        }
        
        info!("   Vendor: {} ({})", vendor_name, vendor_id);
        info!("   Model:  {} ({})", model_name, model_id);
        
        // Identify likely device type
        if vendor_id == "cafe" {
            info!("   🎹 This appears to be a monome grid");
        } else if vendor_id == "0483" && model_id == "5740" {
            info!("   🐦 This appears to be Crow (STMicroelectronics Virtual COM Port)");
        } else {
            info!("   ❓ Unknown device type");
        }
    }
    
    Ok(())
}

fn test_crow_detection() -> Result<()> {
    info!("\n🐦 Testing Crow Detection:");
    
    #[cfg(feature = "hardware")]
    {
        // Try to find Crow by USB vendor/product ID
        let crow_device = find_crow_device()?;
        
        match crow_device {
            Some(device_path) => {
                info!("✅ Found potential Crow at: {}", device_path);
                test_crow_communication(&device_path)?;
            }
            None => {
                warn!("❌ Could not identify Crow device");
                info!("💡 Make sure Crow is connected via USB");
                info!("💡 Expected: STMicroelectronics Virtual COM Port (0483:5740)");
            }
        }
    }
    
    #[cfg(not(feature = "hardware"))]
    {
        warn!("❌ Hardware features not enabled - cannot test serial communication");
        info!("💡 Build with --features hardware to enable serial testing");
    }
    
    Ok(())
}

#[cfg(feature = "hardware")]
fn find_crow_device() -> Result<Option<String>> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("ls /dev/ttyACM* 2>/dev/null || true")
        .output()?;
    
    let devices_str = String::from_utf8_lossy(&output.stdout);
    let devices: Vec<&str> = devices_str.trim().lines().filter(|s| !s.is_empty()).collect();
    
    for device in devices {
        // Check if this device has the Crow USB IDs
        let udev_output = Command::new("udevadm")
            .args(&["info", "--query=all", &format!("--name={}", device)])
            .output()?;
        
        let udev_str = String::from_utf8_lossy(&udev_output.stdout);
        
        // Look for STMicroelectronics Virtual COM Port
        if udev_str.contains("ID_VENDOR_ID=0483") && udev_str.contains("ID_MODEL_ID=5740") {
            return Ok(Some(device.to_string()));
        }
    }
    
    Ok(None)
}

#[cfg(feature = "hardware")]
fn test_crow_communication(device_path: &str) -> Result<()> {
    info!("🔗 Testing communication with Crow at {}", device_path);
    
    // Try to open the serial port
    let mut port = match serialport::new(device_path, 115_200)
        .timeout(Duration::from_millis(1000))
        .open()
    {
        Ok(port) => {
            info!("✅ Successfully opened serial port");
            port
        }
        Err(e) => {
            error!("❌ Failed to open serial port: {}", e);
            return Err(anyhow!("Failed to open {}: {}", device_path, e));
        }
    };
    
    // Test basic Crow commands
    let test_commands = vec![
        "print('hello crow')",           // Basic Lua print
        "output[1].volts = 0",          // Set output 1 to 0V
        "output[2].volts = 1",          // Set output 2 to 1V  
        "output[3].volts = -1",         // Set output 3 to -1V
        "output[4].volts = 5",          // Set output 4 to 5V
        "print('test complete')",       // Confirmation
    ];
    
    for (i, command) in test_commands.iter().enumerate() {
        info!("📤 Sending command {}: {}", i + 1, command);
        
        let full_command = format!("{}\n", command);
        
        match port.write_all(full_command.as_bytes()) {
            Ok(_) => {
                port.flush()?;
                info!("   ✅ Command sent successfully");
                
                // Give Crow time to process
                std::thread::sleep(Duration::from_millis(100));
                
                // Try to read any response (Crow might send debug output)
                let mut buf = [0u8; 256];
                match port.read(&mut buf) {
                    Ok(bytes_read) if bytes_read > 0 => {
                        let response = String::from_utf8_lossy(&buf[..bytes_read]);
                        info!("   📥 Response: {}", response.trim());
                    }
                    Ok(_) => {
                        debug!("   📭 No response (normal for Crow)");
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        debug!("   ⏱️ Read timeout (normal for Crow)");
                    }
                    Err(e) => {
                        warn!("   ⚠️ Read error: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("   ❌ Failed to send command: {}", e);
                return Err(anyhow!("Failed to send command: {}", e));
            }
        }
        
        // Small delay between commands
        std::thread::sleep(Duration::from_millis(50));
    }
    
    info!("🎉 Crow communication test completed!");
    info!("💡 If you have a multimeter, check the CV outputs:");
    info!("   Output 1: 0V");
    info!("   Output 2: 1V");
    info!("   Output 3: -1V");
    info!("   Output 4: 5V");
    
    // Reset all outputs to 0V
    info!("🔄 Resetting all outputs to 0V...");
    for i in 1..=4 {
        let reset_cmd = format!("output[{}].volts = 0\n", i);
        port.write_all(reset_cmd.as_bytes())?;
        port.flush()?;
        std::thread::sleep(Duration::from_millis(50));
    }
    info!("✅ All outputs reset to 0V");
    
    Ok(())
}