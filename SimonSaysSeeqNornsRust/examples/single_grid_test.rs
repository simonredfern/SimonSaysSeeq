//! Single Grid Test - Test OSC implementation with one working grid
//! 
//! This test focuses on just the working grid to verify our OSC implementation
//! works correctly, then provides guidance for the non-working grid.
//! Run with: cargo run --example single_grid_test --features desktop

use simon_says_seeq_rust::grid_osc::GridManager;
use log::{info, warn, error};
use env_logger;
use std::time::{Duration, Instant};
use std::thread;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🎹 Single Grid Test (OSC Implementation Verification)");
    info!("===================================================");
    info!("");
    info!("This test focuses on the working grid to verify our OSC");
    info!("implementation is correct, then addresses the non-working grid.");
    info!("");
    
    let start_time = Instant::now();
    
    match GridManager::new() {
        Ok(mut manager) => {
            let connected_grids = manager.get_connected_grids();
            
            if connected_grids.is_empty() {
                error!("❌ No grids detected via OSC");
                return Ok(());
            }
            
            info!("✅ Found {} grid(s):", connected_grids.len());
            for (i, grid_id) in connected_grids.iter().enumerate() {
                if let Some((cols, rows)) = manager.get_dimensions(grid_id) {
                    let name = manager.get_name(grid_id).unwrap_or("Unknown".to_string());
                    info!("   {}. {}: {} ({}x{})", i + 1, grid_id, name, cols, rows);
                }
            }
            info!("");
            
            // Test each grid individually to identify which one works
            let mut working_grids = Vec::new();
            let mut non_working_grids = Vec::new();
            
            for grid_id in &connected_grids {
                info!("🧪 Testing grid: {}", grid_id);
                let name = manager.get_name(grid_id).unwrap_or("Unknown".to_string());
                info!("   Device: {}", name);
                
                // Test 1: Simple clear
                info!("   Step 1: Initial clear...");
                if let Err(e) = manager.clear_all(grid_id) {
                    warn!("   Clear failed: {}", e);
                }
                thread::sleep(Duration::from_millis(200));
                
                // Test 2: Single LED
                info!("   Step 2: Testing single LED at (0,0)...");
                if let Err(e) = manager.set_led(grid_id, 0, 0, 15) {
                    warn!("   LED command failed: {}", e);
                    continue;
                }
                thread::sleep(Duration::from_millis(700));
                
                print!("   👀 Did you see LED at (0,0) turn ON for grid {}? (y/n): ", grid_id);
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let led_visible = input.trim().to_lowercase().starts_with('y');
                
                // Turn off the LED
                manager.set_led(grid_id, 0, 0, 0)?;
                thread::sleep(Duration::from_millis(300));
                
                if led_visible {
                    info!("   ✅ Grid {} is WORKING with OSC", grid_id);
                    working_grids.push(grid_id.clone());
                } else {
                    warn!("   ❌ Grid {} is NOT responding to OSC commands", grid_id);
                    non_working_grids.push(grid_id.clone());
                }
                
                // Clear grid
                manager.clear_all(grid_id).ok();
                info!("");
            }
            
            // Summary and analysis
            info!("📊 GRID ANALYSIS SUMMARY");
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━");
            info!("");
            
            if !working_grids.is_empty() {
                info!("✅ WORKING GRIDS ({}):", working_grids.len());
                for grid_id in &working_grids {
                    let name = manager.get_name(grid_id).unwrap_or("Unknown".to_string());
                    info!("   - {}: {}", grid_id, name);
                }
                info!("");
                
                // Detailed test on working grid
                info!("🎯 Detailed Test on Working Grid: {}", working_grids[0]);
                detailed_test(&mut manager, &working_grids[0])?;
            }
            
            if !non_working_grids.is_empty() {
                warn!("❌ NON-WORKING GRIDS ({}):", non_working_grids.len());
                for grid_id in &non_working_grids {
                    let name = manager.get_name(grid_id).unwrap_or("Unknown".to_string());
                    warn!("   - {}: {}", grid_id, name);
                }
                info!("");
                
                print_non_working_grid_analysis(&non_working_grids);
            }
            
            if working_grids.len() == connected_grids.len() {
                info!("🎉 ALL GRIDS WORKING!");
                info!("The OSC implementation is working correctly for all connected grids.");
            } else if working_grids.is_empty() {
                error!("💥 NO GRIDS WORKING!");
                error!("None of the grids are responding to OSC commands.");
                error!("This suggests a fundamental OSC communication issue.");
            } else {
                warn!("⚖️  MIXED RESULTS!");
                warn!("{} grid(s) working, {} grid(s) not working", 
                      working_grids.len(), non_working_grids.len());
                warn!("This suggests different grid models with different requirements.");
            }
            
        }
        Err(e) => {
            error!("❌ Failed to create GridManager: {}", e);
            return Err(e.into());
        }
    }
    
    let total_time = start_time.elapsed();
    info!("");
    info!("🏁 Single grid test completed in {:.2}s", total_time.as_secs_f32());
    
    Ok(())
}

fn detailed_test(manager: &mut GridManager, grid_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let name = manager.get_name(grid_id).unwrap_or("Unknown".to_string());
    info!("   Running detailed test on {} ({})", grid_id, name);
    
    // Test 1: Corner pattern
    info!("   🔸 Test 1: Corner pattern");
    let corners = [(0, 0), (15, 0), (0, 7), (15, 7)];
    
    for &(x, y) in &corners {
        manager.set_led(grid_id, x, y, 12)?;
        thread::sleep(Duration::from_millis(150));
    }
    
    thread::sleep(Duration::from_millis(500));
    
    if prompt_user("Do you see 4 corner LEDs lit up?") {
        info!("   ✅ Corner pattern working");
    } else {
        warn!("   ❌ Corner pattern not working");
    }
    
    // Clear corners
    for &(x, y) in &corners {
        manager.set_led(grid_id, x, y, 0)?;
        thread::sleep(Duration::from_millis(100));
    }
    
    // Test 2: Brightness levels
    info!("   🔸 Test 2: Brightness levels (center LED)");
    let center_x = 8;
    let center_y = 4;
    
    for brightness in [2, 5, 8, 11, 15] {
        info!("     Setting brightness {}...", brightness);
        manager.set_led(grid_id, center_x, center_y, brightness)?;
        thread::sleep(Duration::from_millis(400));
    }
    
    manager.set_led(grid_id, center_x, center_y, 0)?;
    
    if prompt_user("Did you see the center LED change brightness?") {
        info!("   ✅ Brightness control working");
    } else {
        warn!("   ❌ Brightness control not working");
    }
    
    // Test 3: Clear all
    info!("   🔸 Test 3: Clear all command");
    
    // Light up several LEDs
    for x in 5..11 {
        for y in 2..6 {
            manager.set_led(grid_id, x, y, 8)?;
        }
    }
    thread::sleep(Duration::from_millis(500));
    
    if prompt_user("Do you see a rectangular block of LEDs?") {
        manager.clear_all(grid_id)?;
        thread::sleep(Duration::from_millis(300));
        
        if prompt_user("Did the clear all command turn off all LEDs?") {
            info!("   ✅ Clear all command working");
        } else {
            warn!("   ❌ Clear all command not working properly");
        }
    } else {
        warn!("   ❌ Block pattern setup failed");
    }
    
    // Final cleanup
    manager.clear_all(grid_id)?;
    
    info!("   ✅ Detailed test completed for {}", grid_id);
    
    Ok(())
}

fn prompt_user(question: &str) -> bool {
    print!("     👀 {}: (y/n): ", question);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase().starts_with('y')
}

fn print_non_working_grid_analysis(non_working_grids: &[String]) {
    warn!("🔍 NON-WORKING GRID ANALYSIS");
    warn!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    warn!("");
    
    for grid_id in non_working_grids {
        warn!("Grid: {}", grid_id);
        
        if grid_id.contains("m2949672") {
            warn!("   Model: monome 128");
            warn!("   Likely issues:");
            warn!("     - Older grid model (~2012-2014 era)");
            warn!("     - May require different OSC address format");
            warn!("     - Might need /sys/prefix configuration");
            warn!("     - Could be in legacy serial mode instead of OSC mode");
            warn!("     - May require different baud rate or protocol");
        } else {
            warn!("   Model: Unknown");
            warn!("   Likely issues:");
            warn!("     - Non-standard configuration");
            warn!("     - Different communication protocol");
            warn!("     - Firmware incompatibility");
        }
        warn!("");
    }
    
    warn!("🛠️  RECOMMENDED FIXES:");
    warn!("");
    warn!("1. Try manual OSC commands to test each grid:");
    for grid_id in non_working_grids {
        if grid_id.contains("m2949672") {
            warn!("   # Test grid {} (port 13832):", grid_id);
            warn!("   oscsend 127.0.0.1 13832 /monome/grid/led/set iii 0 0 1");
            warn!("   sleep 1");
            warn!("   oscsend 127.0.0.1 13832 /monome/grid/led/all i 0");
        } else if grid_id.contains("m5032747") {
            warn!("   # Test grid {} (port 15335):", grid_id);
            warn!("   oscsend 127.0.0.1 15335 /monome/grid/led/set iii 0 0 1");
            warn!("   sleep 1");  
            warn!("   oscsend 127.0.0.1 15335 /monome/grid/led/all i 0");
        }
    }
    warn!("");
    
    warn!("2. Check OSC prefix configuration:");
    warn!("   oscsend 127.0.0.1 13832 /sys/info s localhost");
    warn!("   oscsend 127.0.0.1 15335 /sys/info s localhost");
    warn!("");
    
    warn!("3. Try different OSC prefixes:");
    warn!("   oscsend 127.0.0.1 13832 /grid/led/set iii 0 0 1");
    warn!("   oscsend 127.0.0.1 13832 /m128/grid/led/set iii 0 0 1");
    warn!("");
    
    warn!("4. For older monome 128 grids, try:");
    warn!("   - Different serialosc configuration");
    warn!("   - Legacy serial communication instead of OSC");
    warn!("   - Firmware updates if available");
    warn!("");
    
    warn!("5. Isolate the problem:");
    warn!("   - Disconnect the working grid, test just the non-working one");
    warn!("   - Test with other monome software (Max/MSP, norns, etc.)");
    warn!("   - Check monome community forums for model-specific issues");
}