//! Test serial port discovery for monome grids
//! 
//! This example demonstrates how the grid discovery works with serial communication.
//! Run with: cargo run --example test_grid_discovery --features desktop

use simon_says_seeq_rust::grid_osc::GridManager;
use log::{info, warn, error};
use env_logger;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with debug level
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    info!("Testing OSC-based grid discovery via serialosc...");
    info!("This uses proper OSC communication instead of raw serial...");
    
    let start_time = Instant::now();
    
    // Create OSC grid manager - this will discover devices via serialosc
    info!("Creating OSC grid manager...");
    match GridManager::new() {
        Ok(mut manager) => {
            let connected_grids = manager.get_connected_grids();
            
            let discovery_time = start_time.elapsed();
            info!("OSC grid discovery completed in {:.2}s", discovery_time.as_secs_f32());
            
            if connected_grids.is_empty() {
                info!("No grids detected via OSC/serialosc. This could mean:");
                info!("  1. No monome grids are connected via USB");
                info!("  2. serialosc is not running: serialoscd");
                info!("  3. serialosc is not installed: sudo apt install serialosc");
                info!("  4. Permission issues: sudo usermod -a -G dialout $USER");
                info!("  5. Grid not recognized by serialosc");
                info!("Check with: ps aux | grep serialosc");
            } else {
                info!("Found {} grid(s) via OSC!", connected_grids.len());
                
                for grid_id in &connected_grids {
                    if let Some((cols, rows)) = manager.get_dimensions(grid_id) {
                        let name = manager.get_name(grid_id).unwrap_or("Unknown".to_string());
                        let varibright = manager.is_varibright(grid_id);
                        
                        info!("Grid {}: {} ({}x{}, varibright: {})", 
                              grid_id, name, cols, rows, varibright);
                        
                        // Test the grid with OSC commands
                        info!("Testing grid {} with OSC commands...", grid_id);
                        if let Err(e) = manager.test_grid(grid_id) {
                            warn!("Failed to test grid {}: {}", grid_id, e);
                        } else {
                            info!("Grid {} OSC test completed successfully", grid_id);
                            info!("  You should have seen LEDs light up and turn off cleanly");
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to create OSC grid manager: {}", e);
            error!("Error details: {:?}", e);
            error!("This is likely due to:");
            error!("  - serialosc not running or not installed");
            error!("  - No OSC communication available");
            error!("  - Network/firewall blocking localhost OSC");
            error!("  - Grid not recognized by serialosc");
            
            // OSC-specific debugging information
            info!("OSC debugging tips:");
            info!("  - Check if serialosc is running: ps aux | grep serialosc");
            info!("  - Start serialosc manually: serialoscd");
            info!("  - Install serialosc: sudo apt install serialosc");
            info!("  - Check OSC server: netstat -ln | grep 12002");
            info!("  - Check USB devices: lsusb | grep -i cafe");
        }
    }
    
    let total_time = start_time.elapsed();
    info!("OSC grid discovery test completed in {:.2}s", total_time.as_secs_f32());
    info!("");
    info!("💡 This test now uses proper OSC communication via serialosc");
    info!("   instead of raw serial commands. This prevents system hangs");
    info!("   and provides reliable grid communication.");
    Ok(())
}