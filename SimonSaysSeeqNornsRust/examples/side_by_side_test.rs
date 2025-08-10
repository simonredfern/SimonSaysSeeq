//! Side-by-side Grid Test
//! 
//! This utility tests both grids simultaneously to identify behavioral differences.
//! It performs the same operations on both grids so you can see which one responds correctly.
//! Run with: cargo run --example side_by_side_test --features desktop

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
    
    info!("🎹🎹 Side-by-Side Grid Test");
    info!("===========================");
    info!("");
    info!("This test will perform identical operations on both grids");
    info!("so you can observe which one responds correctly.");
    info!("");
    
    let start_time = Instant::now();
    
    match GridManager::new() {
        Ok(mut manager) => {
            let connected_grids = manager.get_connected_grids();
            
            if connected_grids.len() < 2 {
                error!("❌ Need at least 2 grids for side-by-side test");
                error!("   Found {} grid(s): {:?}", connected_grids.len(), connected_grids);
                return Ok(());
            }
            
            info!("✅ Found {} grids for side-by-side test:", connected_grids.len());
            for (i, grid_id) in connected_grids.iter().enumerate() {
                if let Some((cols, rows)) = manager.get_dimensions(grid_id) {
                    let name = manager.get_name(grid_id).unwrap_or("Unknown".to_string());
                    info!("   {}. {}: {} ({}x{})", i + 1, grid_id, name, cols, rows);
                }
            }
            info!("");
            
            // Use first two grids for comparison
            let grid_a = &connected_grids[0];
            let grid_b = &connected_grids[1];
            
            info!("🔬 Comparing Grid A ({}) vs Grid B ({})", grid_a, grid_b);
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            info!("");
            
            // Test 1: Initial clear
            info!("🧪 Test 1: Initial clear test");
            info!("   Clearing both grids...");
            
            let clear_start = Instant::now();
            let result_a = manager.clear_all(grid_a);
            let result_b = manager.clear_all(grid_b);
            let clear_time = clear_start.elapsed();
            
            info!("   Grid A clear: {}", format_result(&result_a));
            info!("   Grid B clear: {}", format_result(&result_b));
            info!("   Clear time: {:.0}ms", clear_time.as_millis());
            
            thread::sleep(Duration::from_millis(300));
            
            if !prompt_user("Are BOTH grids completely clear (all LEDs off)?") {
                warn!("⚠️  Initial clear issue detected");
                
                let grid_a_clear = prompt_user(&format!("Is Grid A ({}) completely clear?", grid_a));
                let grid_b_clear = prompt_user(&format!("Is Grid B ({}) completely clear?", grid_b));
                
                info!("   Grid A clear status: {}", if grid_a_clear { "✅ YES" } else { "❌ NO" });
                info!("   Grid B clear status: {}", if grid_b_clear { "✅ YES" } else { "❌ NO" });
                
                if !grid_a_clear || !grid_b_clear {
                    info!("   Attempting force clear on problematic grid(s)...");
                    if !grid_a_clear {
                        force_clear_grid(&mut manager, grid_a)?;
                    }
                    if !grid_b_clear {
                        force_clear_grid(&mut manager, grid_b)?;
                    }
                }
            } else {
                info!("✅ Both grids cleared successfully");
            }
            
            info!("");
            
            // Test 2: Single LED test
            info!("🧪 Test 2: Single LED test (position 0,0)");
            info!("   Lighting LED at (0,0) on both grids...");
            
            let led_start = Instant::now();
            let result_a = manager.set_led(grid_a, 0, 0, 15, "example_caller");
            let result_b = manager.set_led(grid_b, 0, 0, 15, "example_caller");
            let led_time = led_start.elapsed();
            
            info!("   Grid A LED on: {}", format_result(&result_a));
            info!("   Grid B LED on: {}", format_result(&result_b));
            info!("   LED command time: {:.0}ms", led_time.as_millis());
            
            thread::sleep(Duration::from_millis(500));
            
            let grid_a_on = prompt_user(&format!("Is the top-left LED ON for Grid A ({})?", grid_a));
            let grid_b_on = prompt_user(&format!("Is the top-left LED ON for Grid B ({})?", grid_b));
            
            info!("   Grid A LED visible: {}", if grid_a_on { "✅ YES" } else { "❌ NO" });
            info!("   Grid B LED visible: {}", if grid_b_on { "✅ YES" } else { "❌ NO" });
            
            // Turn off the test LED
            manager.set_led(grid_a, 0, 0, 0, "example_caller")?;
            manager.set_led(grid_b, 0, 0, 0, "example_caller")?;
            thread::sleep(Duration::from_millis(300));
            
            info!("");
            
            // Test 3: Pattern test
            info!("🧪 Test 3: Pattern test (corners)");
            info!("   Lighting corner LEDs on both grids...");
            
            let pattern_positions = [(0, 0), (15, 0), (0, 7), (15, 7)];
            
            for &(x, y) in &pattern_positions {
                let result_a = manager.set_led(grid_a, x, y, 10, "example_caller");
                let result_b = manager.set_led(grid_b, x, y, 10, "example_caller");
                
                if result_a.is_err() || result_b.is_err() {
                    warn!("   Pattern position ({}, {}): A={}, B={}", 
                          x, y, format_result(&result_a), format_result(&result_b));
                }
                
                thread::sleep(Duration::from_millis(100));
            }
            
            thread::sleep(Duration::from_millis(500));
            
            let grid_a_pattern = prompt_user(&format!("Does Grid A ({}) show 4 corner LEDs?", grid_a));
            let grid_b_pattern = prompt_user(&format!("Does Grid B ({}) show 4 corner LEDs?", grid_b));
            
            info!("   Grid A pattern: {}", if grid_a_pattern { "✅ CORRECT" } else { "❌ INCORRECT" });
            info!("   Grid B pattern: {}", if grid_b_pattern { "✅ CORRECT" } else { "❌ INCORRECT" });
            
            // Clear pattern
            for &(x, y) in &pattern_positions {
                manager.set_led(grid_a, x, y, 0, "example_caller")?;
                manager.set_led(grid_b, x, y, 0, "example_caller")?;
                thread::sleep(Duration::from_millis(50));
            }
            
            info!("");
            
            // Test 4: Brightness test (if both support varibright)
            if manager.is_varibright(grid_a) && manager.is_varibright(grid_b) {
                info!("🧪 Test 4: Brightness level test");
                info!("   Testing different brightness levels on center LED...");
                
                let center_x = 8;
                let center_y = 4;
                
                for brightness in [3, 6, 9, 12, 15, 0] {
                    info!("   Setting brightness {} on both grids...", brightness);
                    manager.set_led(grid_a, center_x, center_y, brightness, "example_caller")?;
                    manager.set_led(grid_b, center_x, center_y, brightness, "example_caller")?;
                    thread::sleep(Duration::from_millis(400));
                }
                
                let grid_a_brightness = prompt_user(&format!("Did Grid A ({}) show changing brightness levels?", grid_a));
                let grid_b_brightness = prompt_user(&format!("Did Grid B ({}) show changing brightness levels?", grid_b));
                
                info!("   Grid A brightness: {}", if grid_a_brightness { "✅ WORKING" } else { "❌ NOT WORKING" });
                info!("   Grid B brightness: {}", if grid_b_brightness { "✅ WORKING" } else { "❌ NOT WORKING" });
                
                info!("");
            }
            
            // Test 5: Clear all test
            info!("🧪 Test 5: Clear all command test");
            
            // First light up several LEDs on both grids
            info!("   Lighting up test pattern on both grids...");
            for x in 2..6 {
                for y in 2..4 {
                    manager.set_led(grid_a, x, y, 12, "example_caller")?;
                    manager.set_led(grid_b, x, y, 12, "example_caller")?;
                }
            }
            thread::sleep(Duration::from_millis(500));
            
            if prompt_user("Do you see identical rectangular patterns on both grids?") {
                info!("✅ Pattern setup successful");
                
                // Now test clear all
                info!("   Testing clear all command...");
                let clear_start = Instant::now();
                let result_a = manager.clear_all(grid_a);
                let result_b = manager.clear_all(grid_b);
                let clear_time = clear_start.elapsed();
                
                info!("   Grid A clear all: {}", format_result(&result_a));
                info!("   Grid B clear all: {}", format_result(&result_b));
                info!("   Clear all time: {:.0}ms", clear_time.as_millis());
                
                thread::sleep(Duration::from_millis(300));
                
                let grid_a_cleared = prompt_user(&format!("Is Grid A ({}) completely cleared?", grid_a));
                let grid_b_cleared = prompt_user(&format!("Is Grid B ({}) completely cleared?", grid_b));
                
                info!("   Grid A clear result: {}", if grid_a_cleared { "✅ SUCCESS" } else { "❌ FAILED" });
                info!("   Grid B clear result: {}", if grid_b_cleared { "✅ SUCCESS" } else { "❌ FAILED" });
                
            } else {
                warn!("⚠️  Pattern setup failed - skipping clear test");
            }
            
            info!("");
            
            // Final summary
            info!("📋 SIDE-BY-SIDE COMPARISON SUMMARY");
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            let grid_a_name = manager.get_name(grid_a).unwrap_or("Unknown".to_string());
            let grid_b_name = manager.get_name(grid_b).unwrap_or("Unknown".to_string());
            
            info!("Grid A ({}): {}", grid_a, grid_a_name);
            info!("Grid B ({}): {}", grid_b, grid_b_name);
            info!("");
            
            if prompt_user("Overall, do BOTH grids appear to be working correctly?") {
                info!("✅ Both grids working - the issue may have been resolved!");
            } else {
                warn!("❌ One or both grids still having issues");
                
                let which_working = if prompt_user("Is Grid A working correctly?") {
                    if prompt_user("Is Grid B working correctly?") {
                        "Both working but inconsistent"
                    } else {
                        "Only Grid A working"
                    }
                } else {
                    if prompt_user("Is Grid B working correctly?") {
                        "Only Grid B working"
                    } else {
                        "Neither working"
                    }
                };
                
                warn!("   Status: {}", which_working);
                info!("");
                print_troubleshooting_recommendations(grid_a, grid_b, &grid_a_name, &grid_b_name);
            }
            
            // Final cleanup
            info!("");
            info!("🧹 Final cleanup...");
            manager.clear_all(grid_a).ok();
            manager.clear_all(grid_b).ok();
            
        }
        Err(e) => {
            error!("❌ Failed to create GridManager: {}", e);
            return Err(e.into());
        }
    }
    
    let total_time = start_time.elapsed();
    info!("");
    info!("🏁 Side-by-side test completed in {:.2}s", total_time.as_secs_f32());
    
    Ok(())
}

fn format_result<T, E: std::fmt::Display>(result: &Result<T, E>) -> &'static str {
    match result {
        Ok(_) => "✅ OK",
        Err(_) => "❌ ERROR",
    }
}

fn prompt_user(question: &str) -> bool {
    print!("👀 {}: (y/n): ", question);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase().starts_with('y')
}

fn force_clear_grid(manager: &mut GridManager, grid_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("   🚨 Force clearing grid {}...", grid_id);
    
    // Multiple clear attempts
    for i in 0..3 {
        manager.clear_all(grid_id)?;
        thread::sleep(Duration::from_millis(100));
        info!("     Clear attempt {} sent", i + 1);
    }
    
    // Individual LED clear for first row
    if let Some((cols, _rows)) = manager.get_dimensions(grid_id) {
        info!("   🔧 Individual LED clear (first row)...");
        for x in 0..cols {
            manager.set_led(grid_id, x, 0, 0, "example_caller")?;
            if x % 4 == 0 {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
    
    // Final clear
    manager.clear_all(grid_id)?;
    thread::sleep(Duration::from_millis(200));
    
    info!("   ✅ Force clear completed for {}", grid_id);
    Ok(())
}

fn print_troubleshooting_recommendations(grid_a: &str, grid_b: &str, name_a: &str, name_b: &str) {
    info!("🔧 TROUBLESHOOTING RECOMMENDATIONS");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("Grid A: {} ({})", grid_a, name_a);
    info!("Grid B: {} ({})", grid_b, name_b);
    info!("");
    
    info!("🔍 Possible causes for different behavior:");
    info!("   1. Different grid models/generations");
    info!("   2. Different firmware versions");
    info!("   3. Different USB ports/hubs causing timing differences");
    info!("   4. One grid may be in a different mode");
    info!("   5. Hardware differences between the grids");
    info!("   6. One grid may need different OSC prefix");
    info!("");
    
    info!("🛠️  Recommended actions:");
    info!("   1. Try swapping USB ports between the grids");
    info!("   2. Test each grid individually (disconnect one at a time)");
    info!("   3. Check if one grid works better with different software");
    info!("   4. Update grid firmware if possible");
    info!("   5. Try different OSC prefixes for each grid");
    info!("   6. Check monome documentation for model-specific differences");
    info!("");
    
    info!("🔧 Quick tests to try:");
    info!("   # Test just Grid A:");
    info!("   # Disconnect Grid B, then run:");
    info!("   cargo run --example test_osc_grid_discovery --features desktop");
    info!("");
    info!("   # Test just Grid B:");
    info!("   # Disconnect Grid A, then run:");
    info!("   cargo run --example test_osc_grid_discovery --features desktop");
    info!("");
    
    info!("📱 Grid model identification:");
    info!("   monome 128: Usually 16x8 grid, released ~2012+");
    info!("   monome one: Usually 16x8 grid, more recent model");
    info!("   These may have different communication timing requirements");
}