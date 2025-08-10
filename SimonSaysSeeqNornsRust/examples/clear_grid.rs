//! Clear all LEDs on connected monome grids
//! 
//! This utility connects to all available grids and ensures all LEDs are turned off.
//! Useful for cleaning up after tests or crashes that leave LEDs on.
//! Run with: cargo run --example clear_grid --features desktop

use simon_says_seeq_rust::grid::GridManager;
use log::{info, warn, error};
use env_logger;
use std::time::{Duration, Instant};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with info level
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🧹 Grid LED Cleanup Utility");
    info!("==========================");
    info!("");
    
    let start_time = Instant::now();
    
    info!("🔍 Discovering connected grids...");
    
    match GridManager::new() {
        Ok(mut manager) => {
            let connected_grids = manager.get_connected_grids();
            
            let discovery_time = start_time.elapsed();
            info!("✅ Discovery completed in {:.2}s", discovery_time.as_secs_f32());
            
            if connected_grids.is_empty() {
                info!("📭 No grids detected - nothing to clear");
                info!("💡 Make sure your monome grid is connected via USB");
            } else {
                info!("🎹 Found {} grid(s) to clear:", connected_grids.len());
                
                for &grid_id in &connected_grids {
                    if let Some((cols, rows)) = manager.get_dimensions(grid_id) {
                        let name = manager.get_name(grid_id).unwrap_or("Unknown");
                        let total_leds = cols * rows;
                        
                        info!("   Grid {}: {} ({}x{} = {} LEDs)", 
                              grid_id, name, cols, rows, total_leds);
                        
                        // Clear this grid using multiple methods for thoroughness
                        info!("🧹 Clearing grid {}...", grid_id);
                        
                        // Method 1: Use the built-in clear_all command
                        info!("   Method 1: Hardware clear command...");
                        match manager.clear_all(grid_id) {
                            Ok(_) => info!("   ✅ Hardware clear successful"),
                            Err(e) => warn!("   ⚠️  Hardware clear failed: {}", e),
                        }
                        thread::sleep(Duration::from_millis(100));
                        
                        // Method 2: Manually set each LED to 0 (more reliable)
                        info!("   Method 2: Manual LED clearing...");
                        let mut cleared_count = 0;
                        let mut failed_count = 0;
                        
                        for x in 0..cols {
                            for y in 0..rows {
                                match manager.set_led(grid_id, x, y, 0, "example_caller", "example_caller") {
                                    Ok(_) => cleared_count += 1,
                                    Err(e) => {
                                        failed_count += 1;
                                        if failed_count <= 3 {  // Only log first few errors
                                            warn!("   Failed to clear LED ({}, {}): {}", x, y, e);
                                        }
                                    }
                                }
                                
                                // Add small delay every 8 LEDs to prevent overwhelming
                                if (x * rows + y) % 8 == 0 {
                                    thread::sleep(Duration::from_millis(5));
                                }
                            }
                        }
                        
                        if failed_count > 3 {
                            warn!("   ⚠️  {} additional LED clear failures (not shown)", failed_count - 3);
                        }
                        
                        info!("   ✅ Manually cleared {}/{} LEDs", cleared_count, total_leds);
                        
                        // Method 3: Final verification clear
                        info!("   Method 3: Final verification clear...");
                        thread::sleep(Duration::from_millis(100));
                        match manager.clear_all(grid_id) {
                            Ok(_) => info!("   ✅ Final clear successful"),
                            Err(e) => warn!("   ⚠️  Final clear failed: {}", e),
                        }
                        
                        // Give the grid time to process all commands
                        thread::sleep(Duration::from_millis(200));
                        
                        info!("✅ Grid {} cleanup completed", grid_id);
                        info!("   🔍 Check your grid - ALL LEDs should now be OFF");
                    }
                }
                
                let total_time = start_time.elapsed();
                info!("");
                info!("🏁 Cleanup completed for {} grid(s) in {:.2}s", 
                      connected_grids.len(), total_time.as_secs_f32());
                info!("   All LEDs should now be completely OFF");
                
                if total_time > Duration::from_secs(5) {
                    warn!("⚠️  Cleanup took longer than expected");
                    warn!("   This might indicate communication issues with the grid");
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to create grid manager: {}", e);
            error!("   Cannot clear grids without successful connection");
            error!("   Try reconnecting your grid and running again");
        }
    }
    
    info!("");
    info!("💡 If LEDs are still on after this cleanup:");
    info!("   1. Try disconnecting and reconnecting the grid USB cable");
    info!("   2. Check for other software using the grid");
    info!("   3. Run this utility again");
    info!("   4. Restart the grid device if possible");
    
    Ok(())
}