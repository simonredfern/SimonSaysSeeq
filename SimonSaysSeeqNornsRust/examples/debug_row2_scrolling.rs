//! Debug Row 2 Scrolling Issue
//! 
//! This test focuses specifically on row 2 scrolling behavior to understand
//! why it's not working while row 1 scrolling works correctly.
//! 
//! Run with: cargo run --example debug_row2_scrolling --features desktop

use serialport::{SerialPort, available_ports, SerialPortType};
use std::time::Duration;
use std::thread;
use std::io::{self, Write};
use log::{info, warn, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🔍 Row 2 Scrolling Debug Test");
    info!("============================");
    info!("");
    info!("This test will help diagnose why row 2 scrolling");
    info!("isn't working while row 1 scrolling works correctly.");
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
    
    // Clear the grid first
    info!("");
    info!("🧹 Clearing grid...");
    let clear_cmd = [0x1A, 0x00, 0x00];
    port.write_all(&clear_cmd)?;
    port.flush()?;
    thread::sleep(Duration::from_millis(500));
    
    info!("");
    info!("🧪 Row Comparison Test");
    info!("━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("Setting up identical patterns on rows 1 and 2 for comparison...");
    
    // Set identical patterns on row 1 (y=0) and row 2 (y=1)
    let pattern_positions = [1, 5, 9, 13]; // Steps 2, 6, 10, 14 (0-based)
    let pattern_brightness = 10; // Pattern-only brightness
    
    // Row 1 pattern
    for &pos in &pattern_positions {
        let cmd = [0x1B, pos, 0, pattern_brightness];
        port.write_all(&cmd)?;
        port.flush()?;
        thread::sleep(Duration::from_millis(50));
    }
    
    // Row 2 pattern
    for &pos in &pattern_positions {
        let cmd = [0x1B, pos, 1, pattern_brightness];
        port.write_all(&cmd)?;
        port.flush()?;
        thread::sleep(Duration::from_millis(50));
    }
    
    info!("✅ Identical patterns set on both rows");
    
    let mut input = String::new();
    print!("Are both rows showing identical patterns? (y/n): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let patterns_identical = input.trim().to_lowercase().starts_with('y');
    
    if !patterns_identical {
        warn!("⚠️  Patterns not identical - this indicates a basic grid communication issue");
    }
    
    info!("");
    info!("🎬 Position Scrolling Comparison");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("Now testing position scrolling on both rows simultaneously...");
    
    let position_brightness = 6; // Position-only brightness
    let combined_brightness = 14; // Pattern + position brightness
    
    // Animate position scrolling on both rows simultaneously
    for step in 0..16 {
        info!("Step {}: Setting position indicators...", step + 1);
        
        // Clear previous position on both rows
        if step > 0 {
            let prev_step = step - 1;
            
            // Row 1 - clear previous
            let prev_brightness_row1 = if pattern_positions.contains(&prev_step) { pattern_brightness } else { 0 };
            let cmd = [0x1B, prev_step, 0, prev_brightness_row1];
            port.write_all(&cmd)?;
            port.flush()?;
            
            // Row 2 - clear previous
            let prev_brightness_row2 = if pattern_positions.contains(&prev_step) { pattern_brightness } else { 0 };
            let cmd = [0x1B, prev_step, 1, prev_brightness_row2];
            port.write_all(&cmd)?;
            port.flush()?;
        }
        
        // Set current position on both rows
        // Row 1 - set current
        let current_brightness_row1 = if pattern_positions.contains(&step) { combined_brightness } else { position_brightness };
        let cmd = [0x1B, step, 0, current_brightness_row1];
        port.write_all(&cmd)?;
        port.flush()?;
        
        // Row 2 - set current  
        let current_brightness_row2 = if pattern_positions.contains(&step) { combined_brightness } else { position_brightness };
        let cmd = [0x1B, step, 1, current_brightness_row2];
        port.write_all(&cmd)?;
        port.flush()?;
        
        thread::sleep(Duration::from_millis(300));
        
        // Quick check at key positions
        if step == 3 || step == 7 || step == 11 {
            print!("  Step {}: Is position visible on BOTH rows? (y/n): ", step + 1);
            io::stdout().flush()?;
            input.clear();
            io::stdin().read_line(&mut input)?;
            let both_visible = input.trim().to_lowercase().starts_with('y');
            
            if !both_visible {
                warn!("⚠️  Position not visible on both rows at step {}", step + 1);
            }
        }
    }
    
    // Clear position indicators
    let cmd = [0x1B, 15, 0, if pattern_positions.contains(&15) { pattern_brightness } else { 0 }];
    port.write_all(&cmd)?;
    port.flush()?;
    
    let cmd = [0x1B, 15, 1, if pattern_positions.contains(&15) { pattern_brightness } else { 0 }];
    port.write_all(&cmd)?;
    port.flush()?;
    
    info!("");
    info!("📊 Row Scrolling Comparison Results");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    input.clear();
    print!("Did Row 1 position scrolling work correctly? (y/n): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let row1_works = input.trim().to_lowercase().starts_with('y');
    
    input.clear();
    print!("Did Row 2 position scrolling work correctly? (y/n): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let row2_works = input.trim().to_lowercase().starts_with('y');
    
    input.clear();
    print!("Were the scrolling behaviors identical between rows? (y/n): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let behaviors_identical = input.trim().to_lowercase().starts_with('y');
    
    info!("");
    info!("🔬 Individual LED Test");
    info!("━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("Testing individual LED control on both rows...");
    
    // Clear grid
    port.write_all(&clear_cmd)?;
    port.flush()?;
    thread::sleep(Duration::from_millis(300));
    
    // Test specific positions that might be problematic
    let test_positions = [0, 4, 8, 12, 15];
    
    for &pos in &test_positions {
        info!("Testing position {} on both rows...", pos + 1);
        
        // Light up position on both rows simultaneously
        let cmd_row1 = [0x1B, pos, 0, position_brightness];
        let cmd_row2 = [0x1B, pos, 1, position_brightness];
        
        port.write_all(&cmd_row1)?;
        port.flush()?;
        port.write_all(&cmd_row2)?;
        port.flush()?;
        
        thread::sleep(Duration::from_millis(200));
        
        print!("  Position {}: Both LEDs lit equally? (y/n): ", pos + 1);
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let both_equal = input.trim().to_lowercase().starts_with('y');
        
        if !both_equal {
            warn!("⚠️  LEDs not equal at position {}", pos + 1);
        }
        
        // Clear both LEDs
        let cmd_row1 = [0x1B, pos, 0, 0];
        let cmd_row2 = [0x1B, pos, 1, 0];
        port.write_all(&cmd_row1)?;
        port.flush()?;
        port.write_all(&cmd_row2)?;
        port.flush()?;
        
        thread::sleep(Duration::from_millis(100));
    }
    
    // Final cleanup
    info!("");
    info!("🧹 Final cleanup...");
    port.write_all(&clear_cmd)?;
    port.flush()?;
    thread::sleep(Duration::from_millis(300));
    
    // Analysis and recommendations
    info!("");
    info!("📊 DIAGNOSTIC RESULTS");
    info!("━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("Pattern Setup:");
    info!("  Patterns identical:       {}", if patterns_identical { "✅ YES" } else { "❌ NO" });
    info!("");
    info!("Scrolling Behavior:");
    info!("  Row 1 scrolling works:     {}", if row1_works { "✅ YES" } else { "❌ NO" });
    info!("  Row 2 scrolling works:     {}", if row2_works { "✅ YES" } else { "❌ NO" });
    info!("  Behaviors identical:       {}", if behaviors_identical { "✅ YES" } else { "❌ NO" });
    
    info!("");
    info!("💡 ANALYSIS");
    info!("━━━━━━━━━━━");
    
    if !patterns_identical {
        error!("❌ HARDWARE ISSUE: Basic grid communication problem");
        error!("   Row 2 is not receiving LED commands properly");
        error!("   Check grid hardware, connections, or command protocol");
    } else if row1_works && !row2_works && behaviors_identical {
        info!("🤔 INCONSISTENT RESULTS: Hardware test shows identical behavior");
        info!("   but you report Row 2 scrolling doesn't work in the main app.");
        info!("   This suggests the issue may be:");
        info!("   1. In the main application's row state management");
        info!("   2. Row 2 current_step not being updated correctly in sequencer");
        info!("   3. Grid update timing issues specific to Row 2");
    } else if row1_works && !row2_works && !behaviors_identical {
        warn!("⚠️  HARDWARE DIFFERENCE: Row 2 behaves differently than Row 1");
        warn!("   This could be:");
        warn!("   1. Hardware issue with Row 2 LEDs");
        warn!("   2. Grid coordinate mapping problem");
        warn!("   3. Command protocol issue for Row 2");
    } else if !row1_works && !row2_works {
        error!("❌ GENERAL SCROLLING ISSUE: Neither row scrolls properly");
        error!("   The problem is not row-specific but affects position scrolling generally");
    }
    
    info!("");
    info!("🔧 NEXT STEPS");
    info!("━━━━━━━━━━━━━");
    
    if row1_works && !row2_works {
        info!("1. Check sequencer row state updates for Row 2");
        info!("2. Verify get_row_settings() returns correct data for Row 2");
        info!("3. Add more detailed logging for Row 2 current_step changes");
        info!("4. Test with main sequencer running and compare debug output");
    }
    
    info!("");
    info!("Test completed. Use results above to guide next debugging steps.");
    
    Ok(())
}