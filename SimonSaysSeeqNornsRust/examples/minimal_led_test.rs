//! Minimal LED command test to isolate grid communication issues
//! 
//! This is the most basic possible test to verify LED commands are working.
//! It bypasses most of the GridManager complexity to test raw communication.
//! Run with: cargo run --example minimal_led_test --features desktop

use serialport::{SerialPort, available_ports, SerialPortType};
use std::time::Duration;
use std::thread;
use std::io::{self, Write};
use log::{info, warn, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🔬 Minimal LED Command Test");
    info!("===========================");
    info!("");
    
    // Find monome grid
    info!("🔍 Looking for monome grid...");
    let ports = available_ports()?;
    
    let mut grid_port_name = None;
    for port in ports {
        if let SerialPortType::UsbPort(usb_info) = &port.port_type {
            let vid = usb_info.vid;
            let pid = usb_info.pid;
            
            // Check for monome VIDs
            if vid == 0xCAFE || vid == 0x0A6A {
                info!("✅ Found monome grid: {} (VID:{:04X} PID:{:04X})", 
                      port.port_name, vid, pid);
                grid_port_name = Some(port.port_name);
                break;
            }
        }
    }
    
    let port_name = match grid_port_name {
        Some(name) => name,
        None => {
            error!("❌ No monome grid found");
            return Ok(());
        }
    };
    
    // Open serial port
    info!("🔌 Opening serial port: {}", port_name);
    let mut port = serialport::new(&port_name, 115200)
        .timeout(Duration::from_millis(100))
        .data_bits(serialport::DataBits::Eight)
        .flow_control(serialport::FlowControl::None)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open()?;
    
    info!("✅ Serial port opened successfully");
    
    // Test 1: Send clear all command
    info!("");
    info!("🧪 Test 1: Sending clear all command");
    let clear_cmd = [0x1A, 0x00, 0x00];
    info!("   Command: {:02X} {:02X} {:02X}", clear_cmd[0], clear_cmd[1], clear_cmd[2]);
    
    port.write_all(&clear_cmd)?;
    port.flush()?;
    thread::sleep(Duration::from_millis(300));
    
    print!("   👀 Are ALL LEDs off after clear command? (y/n): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let clear_worked = input.trim().to_lowercase().starts_with('y');
    
    if clear_worked {
        info!("✅ Clear command working");
    } else {
        warn!("⚠️  Clear command not working properly");
    }
    
    // Test 2: Send individual LED on command
    info!("");
    info!("🧪 Test 2: Sending LED ON command");
    let led_on_cmd = [0x1B, 0x00, 0x00, 0x0F]; // Turn on LED at (0,0) with brightness 15
    info!("   Command: {:02X} {:02X} {:02X} {:02X}", 
          led_on_cmd[0], led_on_cmd[1], led_on_cmd[2], led_on_cmd[3]);
    
    port.write_all(&led_on_cmd)?;
    port.flush()?;
    thread::sleep(Duration::from_millis(300));
    
    print!("   👀 Is the top-left LED (0,0) now ON? (y/n): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let led_on_worked = input.trim().to_lowercase().starts_with('y');
    
    if led_on_worked {
        info!("✅ LED ON command working");
    } else {
        warn!("⚠️  LED ON command not working");
    }
    
    // Test 3: Send individual LED off command
    info!("");
    info!("🧪 Test 3: Sending LED OFF command");
    let led_off_cmd = [0x1B, 0x00, 0x00, 0x00]; // Turn off LED at (0,0)
    info!("   Command: {:02X} {:02X} {:02X} {:02X}", 
          led_off_cmd[0], led_off_cmd[1], led_off_cmd[2], led_off_cmd[3]);
    
    port.write_all(&led_off_cmd)?;
    port.flush()?;
    thread::sleep(Duration::from_millis(300));
    
    print!("   👀 Is the top-left LED (0,0) now OFF? (y/n): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let led_off_worked = input.trim().to_lowercase().starts_with('y');
    
    if led_off_worked {
        info!("✅ LED OFF command working");
    } else {
        warn!("⚠️  LED OFF command not working");
    }
    
    // Test 4: Test a few more positions to see if it's position-specific
    info!("");
    info!("🧪 Test 4: Testing different positions");
    
    let test_positions = [
        (1, 1, "near top-left"),
        (15, 0, "top-right corner"),
        (8, 4, "center"),
        (0, 7, "bottom-left corner"),
    ];
    
    for &(x, y, description) in &test_positions {
        info!("   Testing LED at ({}, {}) - {}", x, y, description);
        
        // Turn on
        let cmd_on = [0x1B, x, y, 0x08]; // Medium brightness
        port.write_all(&cmd_on)?;
        port.flush()?;
        thread::sleep(Duration::from_millis(200));
        
        print!("   👀 Is the LED at {} ON? (y/n): ", description);
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let pos_on = input.trim().to_lowercase().starts_with('y');
        
        // Turn off
        let cmd_off = [0x1B, x, y, 0x00];
        port.write_all(&cmd_off)?;
        port.flush()?;
        thread::sleep(Duration::from_millis(200));
        
        print!("   👀 Is the LED at {} OFF? (y/n): ", description);
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let pos_off = input.trim().to_lowercase().starts_with('y');
        
        info!("   Position ({}, {}): ON={}, OFF={}", x, y, 
              if pos_on { "✅" } else { "❌" },
              if pos_off { "✅" } else { "❌" });
    }
    
    // Final cleanup
    info!("");
    info!("🧹 Final cleanup...");
    let clear_cmd = [0x1A, 0x00, 0x00];
    port.write_all(&clear_cmd)?;
    port.flush()?;
    thread::sleep(Duration::from_millis(200));
    
    print!("   👀 FINAL: Are ALL LEDs off after final clear? (y/n): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let final_clear_worked = input.trim().to_lowercase().starts_with('y');
    
    // Summary
    info!("");
    info!("📊 MINIMAL TEST RESULTS");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("Clear all command:    {}", if clear_worked { "✅ WORKING" } else { "❌ FAILED" });
    info!("LED ON command:       {}", if led_on_worked { "✅ WORKING" } else { "❌ FAILED" });
    info!("LED OFF command:      {}", if led_off_worked { "✅ WORKING" } else { "❌ FAILED" });
    info!("Final clear:          {}", if final_clear_worked { "✅ WORKING" } else { "❌ FAILED" });
    
    if !clear_worked || !led_off_worked || !final_clear_worked {
        error!("");
        error!("❌ COMMUNICATION PROBLEM CONFIRMED");
        error!("The grid is not responding properly to LED commands.");
        error!("");
        error!("This could be:");
        error!("  🔧 Wrong command protocol for this grid model");
        error!("  📡 Serial communication timing issues");
        error!("  ⚡ Grid firmware expecting different commands");
        error!("  🔄 Another program interfering");
        error!("  💾 Grid in wrong mode/state");
        error!("");
        error!("Next steps:");
        error!("  1. Try disconnecting/reconnecting the grid");
        error!("  2. Check if the grid works with other software (max/msp, etc.)");
        error!("  3. Research the specific command protocol for PID 4001");
        error!("  4. Check monome documentation for this grid model");
    } else {
        info!("");
        info!("✅ BASIC COMMUNICATION WORKING");
        info!("The LED commands are working at the protocol level.");
        info!("The issue may be in the higher-level GridManager logic.");
    }
    
    Ok(())
}