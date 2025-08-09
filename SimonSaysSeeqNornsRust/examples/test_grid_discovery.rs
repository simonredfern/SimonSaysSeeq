//! Test serial port discovery for monome grids
//! 
//! This example demonstrates how the grid discovery works with serial communication.
//! Run with: cargo run --example test_grid_discovery --features desktop

use simon_says_seeq_rust::grid::GridManager;
use log::{info, warn, error};
use env_logger;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with debug level
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    info!("Testing serial port discovery for monome grids...");
    info!("This may take a moment to scan serial ports...");
    
    let start_time = Instant::now();
    
    // Create grid manager - this will automatically discover devices
    info!("Creating grid manager...");
    match GridManager::new() {
        Ok(mut manager) => {
            let connected_grids = manager.get_connected_grids();
            
            let discovery_time = start_time.elapsed();
            info!("Grid discovery completed in {:.2}s", discovery_time.as_secs_f32());
            
            if connected_grids.is_empty() {
                info!("No grids detected. This is normal if no monome grids are connected.");
                info!("To test with actual hardware:");
                info!("  1. Connect your monome grid via USB");
                info!("  2. Ensure you're in the 'dialout' group: sudo usermod -a -G dialout $USER");
                info!("  3. Log out and back in");
                info!("  4. Run this example again");
                info!("  5. Check 'lsusb' output for devices with VID CAFE or 0A6A");
            } else {
                info!("Found {} grid(s)!", connected_grids.len());
                
                for &grid_id in &connected_grids {
                    if let Some((cols, rows)) = manager.get_dimensions(grid_id) {
                        let name = manager.get_name(grid_id).unwrap_or("Unknown");
                        let varibright = manager.is_varibright(grid_id);
                        
                        info!("Grid {}: {} ({}x{}, varibright: {})", 
                              grid_id, name, cols, rows, varibright);
                        
                        // Test the grid
                        info!("Testing grid {}...", grid_id);
                        if let Err(e) = manager.test_grid(grid_id) {
                            warn!("Failed to test grid {}: {}", grid_id, e);
                        } else {
                            info!("Grid {} test completed successfully", grid_id);
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to create grid manager: {}", e);
            error!("Error details: {:?}", e);
            info!("This might be due to:");
            info!("  - No serial ports available");
            info!("  - Permission issues (not in dialout group)");
            info!("  - No monome grids connected");
            info!("  - Serial port enumeration failure");
            
            // Additional debugging information
            info!("Debug tips:");
            info!("  - Run 'lsusb' to see USB devices");
            info!("  - Run 'ls /dev/ttyACM* /dev/ttyUSB*' to see serial devices");
            info!("  - Check 'dmesg | tail' for recent USB connection messages");
        }
    }
    
    let total_time = start_time.elapsed();
    info!("Grid discovery test completed in {:.2}s", total_time.as_secs_f32());
    Ok(())
}