//! Debug OSC Commands Test
//! 
//! This minimal test shows the exact OSC commands being sent to help debug
//! why the grids aren't responding to LED commands.
//! Run with: RUST_LOG=debug cargo run --example debug_osc_commands --features desktop

use simon_says_seeq_rust::grid_osc::GridManager;
use log::{info, debug};
use env_logger;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    info!("🔍 Debug OSC Commands Test");
    info!("==========================");
    info!("");
    
    match GridManager::new() {
        Ok(mut manager) => {
            let connected_grids = manager.get_connected_grids();
            
            if connected_grids.is_empty() {
                info!("No grids found");
                return Ok(());
            }
            
            let grid_id = &connected_grids[0];
            info!("Testing with grid: {}", grid_id);
            
            if let Some((cols, rows)) = manager.get_dimensions(grid_id) {
                let name = manager.get_name(grid_id).unwrap_or("Unknown".to_string());
                info!("Grid: {} ({}x{})", name, cols, rows);
            }
            
            info!("");
            info!("🧪 Test 1: Single LED command");
            debug!("About to send LED command for position (0,0)...");
            
            match manager.set_led(grid_id, 0, 0, 15, "example_caller") {
                Ok(_) => info!("✅ LED command sent successfully"),
                Err(e) => info!("❌ LED command failed: {}", e),
            }
            
            thread::sleep(Duration::from_millis(500));
            
            info!("");
            info!("🧪 Test 2: Clear command");
            debug!("About to send clear command...");
            
            match manager.clear_all(grid_id) {
                Ok(_) => info!("✅ Clear command sent successfully"),
                Err(e) => info!("❌ Clear command failed: {}", e),
            }
            
            thread::sleep(Duration::from_millis(500));
            
        }
        Err(e) => {
            info!("❌ Failed to create manager: {}", e);
        }
    }
    
    info!("");
    info!("🏁 Debug test complete");
    info!("Check the debug output above to see the exact OSC commands being sent");
    
    Ok(())
}