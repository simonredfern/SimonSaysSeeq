//! Comprehensive grid communication diagnostic tool
//! 
//! This utility performs detailed testing of grid LED communication to diagnose
//! why LEDs might be staying on or not responding properly.
//! Run with: cargo run --example test_grid_communication --features desktop

use simon_says_seeq_rust::grid::GridManager;
use log::{info, warn, error, debug};
use env_logger;
use std::time::{Duration, Instant};
use std::thread;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with debug level for maximum detail
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    info!("🔬 Grid Communication Diagnostic Tool");
    info!("=====================================");
    info!("");
    
    let start_time = Instant::now();
    
    info!("🔍 Step 1: Discovering grids...");
    
    match GridManager::new() {
        Ok(mut manager) => {
            let connected_grids = manager.get_connected_grids();
            
            if connected_grids.is_empty() {
                error!("❌ No grids detected - cannot run communication test");
                return Ok(());
            }
            
            info!("✅ Found {} grid(s)", connected_grids.len());
            
            for &grid_id in &connected_grids {
                if let Some((cols, rows)) = manager.get_dimensions(grid_id) {
                    let name = manager.get_name(grid_id).unwrap_or("Unknown");
                    let varibright = manager.is_varibright(grid_id);
                    let total_leds = cols * rows;
                    
                    info!("");
                    info!("🎹 Testing Grid {}: {} ({}x{} = {} LEDs, varibright: {})", 
                          grid_id, name, cols, rows, total_leds, varibright);
                    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    
                    run_comprehensive_test(&mut manager, grid_id, cols, rows, varibright)?;
                }
            }
            
            let total_time = start_time.elapsed();
            info!("");
            info!("🏁 All tests completed in {:.2}s", total_time.as_secs_f32());
            
        }
        Err(e) => {
            error!("❌ Failed to create grid manager: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

fn run_comprehensive_test(
    manager: &mut GridManager, 
    grid_id: usize, 
    cols: usize, 
    rows: usize, 
    varibright: bool
) -> Result<(), Box<dyn std::error::Error>> {
    
    // Test 1: Initial clear and verification
    info!("🧪 Test 1: Initial clear and verification");
    info!("   Clearing all LEDs...");
    manager.clear_all(grid_id)?;
    thread::sleep(Duration::from_millis(200));
    
    // Ask user to verify
    print!("   👀 Please look at your grid. Are ALL LEDs off? (y/n): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let initial_clear_ok = input.trim().to_lowercase().starts_with('y');
    
    if !initial_clear_ok {
        warn!("⚠️  Initial clear failed - some LEDs still on");
        info!("   Attempting force clear...");
        force_clear_all_leds(manager, grid_id, cols, rows)?;
        
        print!("   👀 After force clear, are ALL LEDs off now? (y/n): ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let force_clear_ok = input.trim().to_lowercase().starts_with('y');
        
        if !force_clear_ok {
            error!("❌ Even force clear failed - there may be a hardware or communication issue");
        } else {
            info!("✅ Force clear successful");
        }
    } else {
        info!("✅ Initial clear successful");
    }
    
    // Test 2: Single LED test
    info!("");
    info!("🧪 Test 2: Single LED communication test");
    let test_x = 0;
    let test_y = 0;
    
    info!("   Testing LED at position (0,0)...");
    manager.set_led(grid_id, test_x, test_y, 15)?;
    thread::sleep(Duration::from_millis(300));
    
    print!("   👀 Is the LED at top-left corner (0,0) now ON? (y/n): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let single_led_on = input.trim().to_lowercase().starts_with('y');
    
    if single_led_on {
        info!("✅ Single LED ON command working");
    } else {
        warn!("⚠️  Single LED ON command failed");
    }
    
    info!("   Turning off LED at (0,0)...");
    manager.set_led(grid_id, test_x, test_y, 0)?;
    thread::sleep(Duration::from_millis(300));
    
    print!("   👀 Is the LED at top-left corner (0,0) now OFF? (y/n): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let single_led_off = input.trim().to_lowercase().starts_with('y');
    
    if single_led_off {
        info!("✅ Single LED OFF command working");
    } else {
        warn!("⚠️  Single LED OFF command failed");
    }
    
    // Test 3: Brightness levels (if varibright)
    let brightness_ok = if varibright {
        info!("");
        info!("🧪 Test 3: Brightness level test (varibright grid)");
        let test_x = cols / 2;
        let test_y = rows / 2;
        
        info!("   Testing brightness levels at center LED ({}, {})...", test_x, test_y);
        
        for brightness in [1, 4, 8, 12, 15, 0] {
            info!("   Setting brightness to {}...", brightness);
            manager.set_led(grid_id, test_x, test_y, brightness)?;
            thread::sleep(Duration::from_millis(500));
        }
        
        print!("   👀 Did you see the center LED change brightness levels? (y/n): ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let brightness_result = input.trim().to_lowercase().starts_with('y');
        
        if brightness_result {
            info!("✅ Brightness control working");
        } else {
            warn!("⚠️  Brightness control may not be working properly");
        }
        
        brightness_result
    } else {
        info!("");
        info!("🧪 Test 3: Skipped (non-varibright grid)");
        true // Consider it passing for non-varibright grids
    };
    
    // Test 4: Pattern test to check for stuck LEDs
    info!("");
    info!("🧪 Test 4: Stuck LED detection test");
    info!("   This will light up a few LEDs, then clear them individually...");
    
    // Light up a small pattern
    let test_positions = [
        (0, 0), (cols-1, 0), (0, rows-1), (cols-1, rows-1), // Corners
        (cols/2, rows/2), // Center
    ];
    
    info!("   Lighting up test pattern (5 LEDs)...");
    for &(x, y) in &test_positions {
        if x < cols && y < rows {
            manager.set_led(grid_id, x, y, 10)?;
            thread::sleep(Duration::from_millis(100));
        }
    }
    
    thread::sleep(Duration::from_millis(1000));
    
    print!("   👀 Do you see exactly 5 LEDs lit up (4 corners + center)? (y/n): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let pattern_ok = input.trim().to_lowercase().starts_with('y');
    
    if !pattern_ok {
        warn!("⚠️  Pattern display incorrect - checking for stuck LEDs...");
        
        print!("   How many LEDs do you see lit up? (enter number): ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        if let Ok(count) = input.trim().parse::<usize>() {
            if count > 5 {
                warn!("⚠️  More LEDs lit than expected ({} vs 5) - likely stuck LEDs", count);
            } else if count < 5 {
                warn!("⚠️  Fewer LEDs lit than expected ({} vs 5) - communication issue", count);
            }
        }
    } else {
        info!("✅ Pattern display correct");
    }
    
    // Clear the pattern one LED at a time
    info!("   Clearing pattern LEDs individually...");
    for &(x, y) in &test_positions {
        if x < cols && y < rows {
            info!("   Clearing LED ({}, {})...", x, y);
            manager.set_led(grid_id, x, y, 0)?;
            thread::sleep(Duration::from_millis(200));
        }
    }
    
    thread::sleep(Duration::from_millis(500));
    
    print!("   👀 Are all pattern LEDs now OFF? (y/n): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let individual_clear_ok = input.trim().to_lowercase().starts_with('y');
    
    if !individual_clear_ok {
        warn!("⚠️  Individual LED clearing failed");
        
        print!("   How many LEDs are still on? (enter number): ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        if let Ok(count) = input.trim().parse::<usize>() {
            warn!("⚠️  {} LEDs still stuck on after individual clearing", count);
        }
    } else {
        info!("✅ Individual LED clearing working");
    }
    
    // Test 5: Final comprehensive clear
    info!("");
    info!("🧪 Test 5: Final comprehensive clear test");
    info!("   Performing multiple clearing methods...");
    
    // Method 1: Hardware clear
    info!("   Method 1: Hardware clear command...");
    manager.clear_all(grid_id)?;
    thread::sleep(Duration::from_millis(100));
    
    // Method 2: Individual LED clear with delay
    info!("   Method 2: Individual LED clear (slow and thorough)...");
    for x in 0..cols {
        for y in 0..rows {
            manager.set_led(grid_id, x, y, 0)?;
            // Longer delay to ensure each command is processed
            if (x * rows + y) % 4 == 0 {
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
    
    // Method 3: Multiple hardware clears
    info!("   Method 3: Multiple hardware clears...");
    for _ in 0..3 {
        manager.clear_all(grid_id)?;
        thread::sleep(Duration::from_millis(100));
    }
    
    // Final verification
    thread::sleep(Duration::from_millis(500));
    
    print!("   👀 FINAL CHECK: Are ALL LEDs now OFF? (y/n): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let final_clear_ok = input.trim().to_lowercase().starts_with('y');
    
    // Generate diagnostic report
    info!("");
    info!("📊 DIAGNOSTIC REPORT for Grid {}", grid_id);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("Initial clear:      {}", if initial_clear_ok { "✅ PASS" } else { "❌ FAIL" });
    info!("Single LED ON:      {}", if single_led_on { "✅ PASS" } else { "❌ FAIL" });
    info!("Single LED OFF:     {}", if single_led_off { "✅ PASS" } else { "❌ FAIL" });
    if varibright {
        info!("Brightness levels:  {}", if brightness_ok { "✅ PASS" } else { "❌ FAIL" });
    }
    info!("Pattern display:    {}", if pattern_ok { "✅ PASS" } else { "❌ FAIL" });
    info!("Individual clear:   {}", if individual_clear_ok { "✅ PASS" } else { "❌ FAIL" });
    info!("Final clear:        {}", if final_clear_ok { "✅ PASS" } else { "❌ FAIL" });
    
    if !final_clear_ok {
        error!("");
        error!("❌ GRID COMMUNICATION ISSUES DETECTED");
        error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        error!("Possible causes:");
        error!("  🔌 Hardware connection issues");
        error!("  📡 Serial communication problems");
        error!("  ⚡ Grid firmware issues");
        error!("  🔄 Another program controlling the grid");
        error!("  💾 Grid internal state corruption");
        error!("");
        error!("Recommended actions:");
        error!("  1. Disconnect and reconnect the grid USB cable");
        error!("  2. Try a different USB port");
        error!("  3. Check if other software is using the grid (max/msp, norns, etc.)");
        error!("  4. Power cycle the grid if possible");
        error!("  5. Check USB cable quality");
        
        // Try emergency reset
        info!("");
        info!("🚨 Attempting emergency reset sequence...");
        attempt_emergency_reset(manager, grid_id, cols, rows)?;
        
    } else {
        info!("");
        info!("✅ GRID COMMUNICATION WORKING CORRECTLY");
        info!("   All tests passed - the grid is responding properly");
    }
    
    Ok(())
}

/// Force clear all LEDs using individual commands
fn force_clear_all_leds(
    manager: &mut GridManager, 
    grid_id: usize, 
    cols: usize, 
    rows: usize
) -> Result<(), Box<dyn std::error::Error>> {
    info!("   🔧 Force clearing each LED individually...");
    
    let mut success_count = 0;
    let mut error_count = 0;
    
    for x in 0..cols {
        for y in 0..rows {
            match manager.set_led(grid_id, x, y, 0) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    if error_count <= 5 {  // Only log first few errors
                        debug!("   Error clearing LED ({}, {}): {}", x, y, e);
                    }
                }
            }
            
            // Add delay every 8 LEDs
            if (x * rows + y) % 8 == 0 {
                thread::sleep(Duration::from_millis(10));
            }
        }
        
        // Progress indicator
        if x % 4 == 0 {
            info!("   Progress: {}/{} columns cleared", x + 1, cols);
        }
    }
    
    if error_count > 5 {
        warn!("   ⚠️  {} additional errors not shown", error_count - 5);
    }
    
    info!("   Force clear result: {}/{} LEDs processed successfully", 
          success_count, cols * rows);
    
    Ok(())
}

/// Attempt various emergency reset techniques
fn attempt_emergency_reset(
    manager: &mut GridManager, 
    grid_id: usize, 
    cols: usize, 
    rows: usize
) -> Result<(), Box<dyn std::error::Error>> {
    
    info!("   🚨 Emergency reset technique 1: Multiple rapid clears...");
    for i in 0..5 {
        manager.clear_all(grid_id)?;
        thread::sleep(Duration::from_millis(50));
        info!("      Clear {} sent", i + 1);
    }
    
    thread::sleep(Duration::from_millis(500));
    print!("   👀 After rapid clears, any improvement? (y/n): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let rapid_clear_helped = input.trim().to_lowercase().starts_with('y');
    
    if rapid_clear_helped {
        info!("   ✅ Rapid clears helped");
    } else {
        info!("   ❌ Rapid clears didn't help");
    }
    
    info!("   🚨 Emergency reset technique 2: Full grid sweep clear...");
    for brightness in [0] {  // Only turn off, don't turn on
        for x in 0..cols {
            for y in 0..rows {
                manager.set_led(grid_id, x, y, brightness)?;
                // Very slow to ensure each command is processed
                if (x * rows + y) % 2 == 0 {
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }
    }
    
    thread::sleep(Duration::from_millis(500));
    print!("   👀 After full sweep clear, any improvement? (y/n): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let sweep_clear_helped = input.trim().to_lowercase().starts_with('y');
    
    if sweep_clear_helped {
        info!("   ✅ Full sweep clear helped");
    } else {
        info!("   ❌ Full sweep clear didn't help");
    }
    
    info!("   🚨 Emergency reset technique 3: Pattern overwrite...");
    info!("      (This will light up then clear each LED to 'reset' its state)");
    
    for x in 0..cols {
        for y in 0..rows {
            // First turn it on
            manager.set_led(grid_id, x, y, 15)?;
            thread::sleep(Duration::from_millis(10));
            // Then turn it off
            manager.set_led(grid_id, x, y, 0)?;
            thread::sleep(Duration::from_millis(10));
            
            if (x * rows + y) % 16 == 0 {
                info!("      Progress: {}/{} LEDs reset", x * rows + y + 1, cols * rows);
            }
        }
    }
    
    // Final hardware clear
    manager.clear_all(grid_id)?;
    thread::sleep(Duration::from_millis(300));
    
    print!("   👀 After pattern overwrite reset, are ALL LEDs off? (y/n): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let pattern_reset_helped = input.trim().to_lowercase().starts_with('y');
    
    if pattern_reset_helped {
        info!("   ✅ Pattern overwrite reset successful!");
    } else {
        error!("   ❌ Pattern overwrite reset failed");
        error!("      This indicates a serious communication or hardware issue");
        error!("      Recommendations:");
        error!("        - Disconnect and reconnect the grid");
        error!("        - Try a different USB cable");
        error!("        - Check if the grid works with other software");
        error!("        - Contact monome support if this is a new issue");
    }
    
    Ok(())
}