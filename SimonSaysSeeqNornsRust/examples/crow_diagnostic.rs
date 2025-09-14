//! Enhanced Crow diagnostic tool with comprehensive testing
//! 
//! This utility performs deep testing of Crow communication including:
//! - USB device identification
//! - Crow-specific protocol commands
//! - CV output verification prompts
//! - Firmware version detection

use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use std::io::{Read, Write};
use std::process::Command;
use std::time::Duration;
use std::thread;

#[cfg(feature = "hardware")]
use serialport::SerialPort;

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🔍 Enhanced Crow Diagnostic Tool");
    info!("================================");
    
    // Step 1: Identify all potential devices
    list_all_serial_devices()?;
    
    // Step 2: Test each potential Crow device
    test_all_potential_crow_devices()?;
    
    // Step 3: Manual testing guide
    show_manual_testing_guide()?;
    
    Ok(())
}

fn list_all_serial_devices() -> Result<()> {
    info!("\n📋 USB Serial Device Analysis:");
    info!("==============================");
    
    let output = Command::new("sh")
        .arg("-c")
        .arg("ls /dev/ttyACM* /dev/ttyUSB* 2>/dev/null || echo 'No serial devices found'")
        .output()?;
    
    let devices_str = String::from_utf8_lossy(&output.stdout);
    let devices: Vec<&str> = devices_str.trim().lines().collect();
    
    if devices.is_empty() || devices[0] == "No serial devices found" {
        warn!("❌ No serial devices found");
        return Ok(());
    }
    
    for device in devices {
        analyze_device(device)?;
    }
    
    Ok(())
}

fn analyze_device(device: &str) -> Result<()> {
    info!("\n🔌 Device: {}", device);
    
    // Get detailed USB info
    let udev_output = Command::new("udevadm")
        .args(&["info", "--query=all", &format!("--name={}", device)])
        .output()?;
    
    let udev_str = String::from_utf8_lossy(&udev_output.stdout);
    
    let mut vendor_id = String::new();
    let mut model_id = String::new();
    let mut vendor_name = String::new();
    let mut model_name = String::new();
    let mut serial_number = String::new();
    
    for line in udev_str.lines() {
        if line.contains("ID_VENDOR_ID=") {
            vendor_id = line.split('=').nth(1).unwrap_or("").to_string();
        } else if line.contains("ID_MODEL_ID=") {
            model_id = line.split('=').nth(1).unwrap_or("").to_string();
        } else if line.contains("ID_VENDOR=") && !line.contains("ID_VENDOR_ID") {
            vendor_name = line.split('=').nth(1).unwrap_or("").to_string();
        } else if line.contains("ID_MODEL=") && !line.contains("ID_MODEL_ID") {
            model_name = line.split('=').nth(1).unwrap_or("").to_string();
        } else if line.contains("ID_SERIAL_SHORT=") {
            serial_number = line.split('=').nth(1).unwrap_or("").to_string();
        }
    }
    
    info!("   📊 USB Info:");
    info!("      Vendor: {} ({})", vendor_name, vendor_id);
    info!("      Model:  {} ({})", model_name, model_id);
    if !serial_number.is_empty() {
        info!("      Serial: {}", serial_number);
    }
    
    // Identify device type
    let device_type = identify_device_type(&vendor_id, &model_id, &vendor_name, &model_name);
    info!("   🏷️  Type: {}", device_type);
    
    Ok(())
}

fn identify_device_type(vendor_id: &str, model_id: &str, vendor_name: &str, model_name: &str) -> String {
    // Known device patterns
    if vendor_id == "cafe" {
        return format!("🎹 monome grid (VID: {})", vendor_id);
    } else if vendor_id == "0483" {
        if model_id == "5740" {
            return "🤔 STMicroelectronics Virtual COM Port (possible Crow)".to_string();
        } else {
            return format!("🔧 STMicroelectronics device (PID: {})", model_id);
        }
    } else if vendor_name.to_lowercase().contains("monome") {
        return "🐦 Potential monome device".to_string();
    } else if vendor_id == "fc02" {
        return "🎵 USB MIDI Interface".to_string();
    } else {
        return format!("❓ Unknown device ({}/{})", vendor_id, model_id);
    }
}

fn test_all_potential_crow_devices() -> Result<()> {
    info!("\n🐦 Testing Potential Crow Devices:");
    info!("==================================");
    
    #[cfg(feature = "hardware")]
    {
        let potential_devices = find_potential_crow_devices()?;
        
        if potential_devices.is_empty() {
            warn!("❌ No potential Crow devices found");
            info!("💡 Expected: STMicroelectronics Virtual COM Port or similar");
            return Ok(());
        }
        
        for device in potential_devices {
            info!("\n🧪 Testing device: {}", device);
            match test_crow_protocol(&device) {
                Ok(is_crow) => {
                    if is_crow {
                        info!("✅ CONFIRMED: {} appears to be a working Crow!", device);
                        test_cv_outputs(&device)?;
                    } else {
                        warn!("❌ {} does not respond as a Crow", device);
                    }
                }
                Err(e) => {
                    error!("❌ Failed to test {}: {}", device, e);
                }
            }
        }
    }
    
    #[cfg(not(feature = "hardware"))]
    {
        warn!("❌ Hardware features not enabled - cannot test serial communication");
        info!("💡 Build with --features hardware to enable testing");
    }
    
    Ok(())
}

#[cfg(feature = "hardware")]
fn find_potential_crow_devices() -> Result<Vec<String>> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("ls /dev/ttyACM* /dev/ttyUSB* 2>/dev/null || true")
        .output()?;
    
    let devices_str = String::from_utf8_lossy(&output.stdout);
    let all_devices: Vec<&str> = devices_str.trim().lines().filter(|s| !s.is_empty()).collect();
    
    let mut potential_devices = Vec::new();
    
    for device in all_devices {
        // Skip obvious non-Crow devices
        if is_definitely_not_crow(device) {
            continue;
        }
        
        potential_devices.push(device.to_string());
    }
    
    Ok(potential_devices)
}

#[cfg(feature = "hardware")]
fn is_definitely_not_crow(device: &str) -> bool {
    // Check if device is definitely a monome grid
    if let Ok(output) = Command::new("udevadm")
        .args(&["info", "--query=all", &format!("--name={}", device)])
        .output()
    {
        let udev_str = String::from_utf8_lossy(&output.stdout);
        
        // Monome grids have vendor ID "cafe"
        if udev_str.contains("ID_VENDOR_ID=cafe") {
            info!("   ⏭️  Skipping {} - confirmed monome grid", device);
            return true;
        }
    }
    
    false
}

#[cfg(feature = "hardware")]
fn test_crow_protocol(device_path: &str) -> Result<bool> {
    info!("   🔗 Opening serial connection...");
    
    let mut port = serialport::new(device_path, 115_200)
        .timeout(Duration::from_millis(2000))
        .open()
        .map_err(|e| anyhow!("Failed to open {}: {}", device_path, e))?;
    
    info!("   ✅ Serial port opened successfully");
    
    // Clear any existing data in the buffer
    let mut discard_buf = [0u8; 1024];
    let _ = port.read(&mut discard_buf);
    
    // Test 1: Try to get Crow version (this is a definitive test)
    info!("   🏷️  Testing Crow version command...");
    let version_result = test_crow_version(&mut port)?;
    
    if version_result.is_some() {
        info!("   ✅ Crow version detected: {}", version_result.unwrap());
        return Ok(true);
    }
    
    // Test 2: Try to get Crow identity
    info!("   🆔 Testing Crow identity command...");
    let identity_result = test_crow_identity(&mut port)?;
    
    if identity_result.is_some() {
        info!("   ✅ Crow identity detected: {}", identity_result.unwrap());
        return Ok(true);
    }
    
    // Test 3: Try basic Lua print command and see if we get any response
    info!("   💬 Testing basic Lua communication...");
    let lua_result = test_basic_lua(&mut port)?;
    
    if lua_result {
        info!("   ✅ Device responds to Lua commands (likely Crow)");
        return Ok(true);
    }
    
    // Test 4: Try CV output command (this won't confirm response but tests protocol)
    info!("   🔊 Testing CV output command...");
    test_cv_command(&mut port)?;
    
    warn!("   ❓ Device accepts commands but no definitive Crow responses detected");
    Ok(false)
}

#[cfg(feature = "hardware")]
fn test_crow_version(port: &mut Box<dyn SerialPort>) -> Result<Option<String>> {
    // Send Crow version command
    port.write_all(b"^^version\n")?;
    port.flush()?;
    
    thread::sleep(Duration::from_millis(500));
    
    let mut response = String::new();
    let mut buf = [0u8; 256];
    
    for _ in 0..5 {  // Try reading multiple times
        match port.read(&mut buf) {
            Ok(bytes_read) if bytes_read > 0 => {
                let chunk = String::from_utf8_lossy(&buf[..bytes_read]);
                response.push_str(&chunk);
                if response.contains('\n') {
                    break;
                }
            }
            _ => thread::sleep(Duration::from_millis(100))
        }
    }
    
    if !response.is_empty() && (response.contains("crow") || response.contains("version") || response.contains('.')) {
        return Ok(Some(response.trim().to_string()));
    }
    
    Ok(None)
}

#[cfg(feature = "hardware")]
fn test_crow_identity(port: &mut Box<dyn SerialPort>) -> Result<Option<String>> {
    // Send Crow identity command
    port.write_all(b"^^identity\n")?;
    port.flush()?;
    
    thread::sleep(Duration::from_millis(500));
    
    let mut response = String::new();
    let mut buf = [0u8; 256];
    
    for _ in 0..5 {
        match port.read(&mut buf) {
            Ok(bytes_read) if bytes_read > 0 => {
                let chunk = String::from_utf8_lossy(&buf[..bytes_read]);
                response.push_str(&chunk);
                if response.len() > 10 {  // Identity should be reasonable length
                    break;
                }
            }
            _ => thread::sleep(Duration::from_millis(100))
        }
    }
    
    if !response.is_empty() {
        return Ok(Some(response.trim().to_string()));
    }
    
    Ok(None)
}

#[cfg(feature = "hardware")]
fn test_basic_lua(port: &mut Box<dyn SerialPort>) -> Result<bool> {
    // Send a basic Lua print command
    port.write_all(b"print('crow_test_12345')\n")?;
    port.flush()?;
    
    thread::sleep(Duration::from_millis(500));
    
    let mut response = String::new();
    let mut buf = [0u8; 256];
    
    for _ in 0..5 {
        match port.read(&mut buf) {
            Ok(bytes_read) if bytes_read > 0 => {
                let chunk = String::from_utf8_lossy(&buf[..bytes_read]);
                response.push_str(&chunk);
            }
            _ => thread::sleep(Duration::from_millis(100))
        }
    }
    
    // Look for our test string in the response
    Ok(response.contains("crow_test_12345"))
}

#[cfg(feature = "hardware")]
fn test_cv_command(port: &mut Box<dyn SerialPort>) -> Result<()> {
    // Send a basic CV output command
    port.write_all(b"output[1].volts = 2.5\n")?;
    port.flush()?;
    
    thread::sleep(Duration::from_millis(200));
    
    info!("   📤 CV command sent (output[1].volts = 2.5)");
    Ok(())
}

#[cfg(feature = "hardware")]
fn test_cv_outputs(device_path: &str) -> Result<()> {
    info!("\n🎵 CV Output Test Sequence");
    info!("==========================");
    
    let mut port = serialport::new(device_path, 115_200)
        .timeout(Duration::from_millis(1000))
        .open()?;
    
    info!("🔊 Starting CV output test - listen to your 1V/octave oscillator!");
    info!("   (You should hear distinct pitch changes)");
    
    let test_voltages = vec![
        (0.0, "0V - Base note"),
        (1.0, "1V - One octave up"), 
        (2.0, "2V - Two octaves up"),
        (-1.0, "-1V - One octave down"),
        (3.0, "3V - Three octaves up"),
        (0.0, "0V - Back to base"),
    ];
    
    for (voltage, description) in test_voltages {
        info!("🎵 Setting output[1] = {} ({})", voltage, description);
        
        let command = format!("output[1].volts = {}\n", voltage);
        port.write_all(command.as_bytes())?;
        port.flush()?;
        
        info!("   ⏸️  Holding for 2 seconds - do you hear the pitch change?");
        thread::sleep(Duration::from_secs(2));
    }
    
    // Reset to 0V
    port.write_all(b"output[1].volts = 0\n")?;
    port.flush()?;
    
    info!("✅ CV test sequence complete - output reset to 0V");
    info!("💡 If you heard pitch changes, your Crow is working correctly!");
    
    Ok(())
}

fn show_manual_testing_guide() -> Result<()> {
    info!("\n📖 Manual Testing Guide:");
    info!("========================");
    info!("");
    info!("🔍 To verify if your Crow is working:");
    info!("   1. Connect Crow's CV output 1 to your oscillator's 1V/oct input");
    info!("   2. Listen to the oscillator");
    info!("   3. Run this diagnostic tool");
    info!("   4. During the CV test, you should hear clear pitch changes");
    info!("");
    info!("🎵 Expected results:");
    info!("   ✅ Working Crow: Distinct pitch jumps (octaves up/down)");
    info!("   ❌ Non-working: No sound changes at all");
    info!("");
    info!("🐛 If no pitch changes occur:");
    info!("   • Device may not be a real Crow");
    info!("   • Crow may have firmware issues");
    info!("   • USB communication problems");
    info!("   • Try different USB ports");
    info!("   • Check Crow power LED");
    info!("");
    info!("🔧 Official Crow tools to try:");
    info!("   • druid (official Crow terminal)");
    info!("   • Max/MSP Crow objects");
    info!("   • norns Crow integration");
    info!("");
    info!("📞 Need help? Check: https://monome.org/docs/crow/");
    
    Ok(())
}