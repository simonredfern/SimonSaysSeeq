//! Emergency grid clear utility using direct serial commands
//! 
//! This utility sends multiple types of clear commands directly to the grid
//! to force all LEDs off, bypassing the normal GridManager logic.
//! Use this when LEDs are stuck on and normal clear methods don't work.
//! Run with: cargo run --example emergency_clear --features desktop

use serialport::{available_ports, SerialPortType};
use std::time::Duration;
use std::thread;
use log::{info, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🚨 EMERGENCY GRID CLEAR UTILITY");
    info!("================================");
    info!("⚠️  This tool attempts multiple clear methods to force LEDs off");
    info!("");
    
    // Find monome grid
    info!("🔍 Looking for monome grid...");
    let ports = available_ports()?;
    
    let mut grid_ports = Vec::new();
    for port in ports {
        if let SerialPortType::UsbPort(usb_info) = &port.port_type {
            let vid = usb_info.vid;
            let pid = usb_info.pid;
            
            // Check for monome VIDs
            if vid == 0xCAFE || vid == 0x0A6A {
                info!("✅ Found monome grid: {} (VID:{:04X} PID:{:04X})", 
                      port.port_name, vid, pid);
                grid_ports.push(port.port_name);
            }
        }
    }
    
    if grid_ports.is_empty() {
        error!("❌ No monome grids found");
        return Ok(());
    }
    
    // Clear each grid found
    for port_name in grid_ports {
        info!("");
        info!("🧹 Emergency clearing grid on: {}", port_name);
        
        match emergency_clear_grid(&port_name) {
            Ok(_) => info!("✅ Emergency clear completed for {}", port_name),
            Err(e) => error!("❌ Emergency clear failed for {}: {}", port_name, e),
        }
    }
    
    info!("");
    info!("🏁 Emergency clear procedure completed");
    info!("💡 If LEDs are still on:");
    info!("   1. Disconnect and reconnect the grid USB cable");
    info!("   2. Try a different USB port");
    info!("   3. Install serialosc for proper OSC communication");
    
    Ok(())
}

fn emergency_clear_grid(port_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("   Opening serial port: {}", port_name);
    
    let mut port = serialport::new(port_name, 115200)
        .timeout(Duration::from_millis(100))
        .data_bits(serialport::DataBits::Eight)
        .flow_control(serialport::FlowControl::None)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()?;
    
    info!("   🚨 Method 1: Standard clear commands");
    
    // Try various clear command formats
    let clear_commands_3 = [
        ([0x1A, 0x00, 0x00], "Standard clear all"),
        ([0x1A, 0xFF, 0xFF], "Clear all variant 1"),
        ([0x00, 0x00, 0x00], "Null command clear"),
    ];
    
    for (cmd, description) in &clear_commands_3 {
        info!("      Trying: {} {:02X?}", description, cmd);
        port.write_all(cmd)?;
        port.flush()?;
        thread::sleep(Duration::from_millis(100));
    }
    
    // Try 4-byte commands separately
    let clear_commands_4 = [
        ([0x1B, 0x00, 0x00, 0x00], "Individual LED clear (0,0)"),
    ];
    
    for (cmd, description) in &clear_commands_4 {
        info!("      Trying: {} {:02X?}", description, cmd);
        port.write_all(cmd)?;
        port.flush()?;
        thread::sleep(Duration::from_millis(100));
    }
    
    info!("   🚨 Method 2: Rapid fire clear commands");
    
    // Send multiple rapid clear commands
    for i in 0..10 {
        let clear_cmd = [0x1A, 0x00, 0x00];
        port.write_all(&clear_cmd)?;
        if i % 3 == 0 {
            port.flush()?;
        }
        thread::sleep(Duration::from_millis(50));
    }
    port.flush()?;
    
    info!("   🚨 Method 3: Individual LED shutdown (first 32 positions)");
    
    // Clear first 32 LEDs individually with various command formats
    for i in 0..32 {
        let x = i % 16;
        let y = i / 16;
        
        // Try different LED command formats
        let led_commands = [
            [0x1B, x, y, 0x00],        // Standard format
            [0x1C, x, y, 0x00],        // Alternative format 1
            [0x10, x, y, 0x00],        // Alternative format 2
        ];
        
        for cmd in &led_commands {
            port.write_all(cmd)?;
        }
        
        if i % 8 == 0 {
            port.flush()?;
            thread::sleep(Duration::from_millis(10));
        }
    }
    
    info!("   🚨 Method 4: Reset sequence");
    
    // Try a reset-like sequence
    let reset_commands = [
        [0xFF, 0xFF, 0xFF],        // Possible reset command
        [0x00, 0x00, 0x00],        // Null
        [0x1A, 0x00, 0x00],        // Clear
        [0x1A, 0x00, 0x00],        // Clear again
        [0x1A, 0x00, 0x00],        // Clear third time
    ];
    
    for cmd in &reset_commands {
        port.write_all(cmd)?;
        port.flush()?;
        thread::sleep(Duration::from_millis(100));
    }
    
    info!("   🚨 Method 5: Exhaustive clear (all possible LED positions)");
    
    // Clear every possible LED position with multiple command formats
    for brightness in [0x00] {  // Only turn off
        for x in 0..16 {
            for y in 0..8 {
                // Try main command format
                port.write_all(&[0x1B, x, y, brightness])?;
                
                // Flush every 16 commands to prevent buffer overflow
                if (x * 8 + y) % 16 == 0 {
                    port.flush()?;
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }
    
    port.flush()?;
    
    info!("   🚨 Method 6: Final comprehensive clear");
    
    // Multiple final clears with delays
    for i in 0..5 {
        port.write_all(&[0x1A, 0x00, 0x00])?;
        port.flush()?;
        thread::sleep(Duration::from_millis(200));
        info!("      Final clear {} sent", i + 1);
    }
    
    // Give grid time to process all commands
    thread::sleep(Duration::from_millis(500));
    
    info!("   ✅ Emergency clear sequence completed for {}", port_name);
    
    Ok(())
}