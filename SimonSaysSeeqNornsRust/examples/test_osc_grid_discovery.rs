//! Test OSC-based grid discovery and functionality
//! 
//! This example demonstrates the new OSC-based grid communication through serialosc.
//! This is the proper way to communicate with modern monome grids.
//! Run with: cargo run --example test_osc_grid_discovery --features desktop

use simon_says_seeq_rust::grid_osc::GridManager;
use log::{info, warn, error};
use env_logger;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with info level
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🎹 OSC Grid Discovery Test (using serialosc)");
    info!("==============================================");
    info!("");
    info!("This test uses proper OSC communication through serialosc");
    info!("instead of raw serial commands. This prevents system hangs");
    info!("and provides reliable grid communication.");
    info!("");
    
    let start_time = Instant::now();
    
    // Check if serialosc is likely running
    info!("🔍 Step 1: Creating OSC grid manager...");
    info!("(This will discover grids through serialosc)");
    
    match GridManager::new() {
        Ok(mut manager) => {
            let connected_grids = manager.get_connected_grids();
            
            let discovery_time = start_time.elapsed();
            info!("✅ OSC grid discovery completed in {:.2}s", discovery_time.as_secs_f32());
            
            if connected_grids.is_empty() {
                info!("📭 No grids detected via OSC/serialosc");
                print_osc_troubleshooting_info();
            } else {
                info!("🎹 Found {} grid(s) via OSC:", connected_grids.len());
                info!("");
                
                // Print grid information
                manager.print_grid_info();
                info!("");
                
                // Test each grid
                for grid_id in &connected_grids {
                    if let Some((cols, rows)) = manager.get_dimensions(grid_id) {
                        let name = manager.get_name(grid_id).unwrap_or("Unknown".to_string());
                        let varibright = manager.is_varibright(grid_id);
                        let total_leds = cols * rows;
                        
                        info!("🧪 Testing Grid {}: {} ({}x{} = {} LEDs, varibright: {})", 
                              grid_id, name, cols, rows, total_leds, varibright);
                        
                        // Test the grid with OSC commands
                        match manager.test_grid(grid_id) {
                            Ok(_) => {
                                info!("✅ Grid {} test completed successfully", grid_id);
                                info!("   You should have seen LEDs light up and then turn off");
                            }
                            Err(e) => {
                                warn!("⚠️  Grid {} test failed: {}", grid_id, e);
                            }
                        }
                        
                        info!("");
                    }
                }
                
                // Final cleanup - ensure all LEDs are off
                info!("🧹 Final cleanup: ensuring all LEDs are OFF...");
                for grid_id in &connected_grids {
                    if let Err(e) = manager.clear_all(grid_id) {
                        warn!("Failed to clear grid {} during final cleanup: {}", grid_id, e);
                    }
                }
                
                // Wait a moment for commands to process
                std::thread::sleep(Duration::from_millis(200));
                
                info!("✅ All grids should now have all LEDs OFF");
            }
        }
        Err(e) => {
            error!("❌ Failed to create OSC grid manager: {}", e);
            error!("This is likely because serialosc is not running or accessible");
            print_osc_error_info();
        }
    }
    
    let total_time = start_time.elapsed();
    info!("");
    info!("🏁 OSC grid discovery test completed in {:.2}s", total_time.as_secs_f32());
    
    if total_time < Duration::from_millis(500) {
        info!("⚡ Very fast completion - this suggests no grids were found");
    } else if total_time > Duration::from_secs(10) {
        warn!("⚠️  Test took longer than expected ({}s)", total_time.as_secs());
        warn!("   This might indicate slow OSC communication");
    }
    
    Ok(())
}

fn print_osc_troubleshooting_info() {
    info!("💡 OSC/serialosc troubleshooting tips:");
    info!("");
    info!("📋 Prerequisites:");
    info!("   ✅ serialosc installed: sudo apt install serialosc");
    info!("   ✅ serialosc running: serialoscd (or sudo systemctl start serialosc)");
    info!("   ✅ Grid connected via USB");
    info!("   ✅ User in dialout group: sudo usermod -a -G dialout $USER");
    info!("");
    info!("🔍 Diagnostic commands:");
    info!("   ps aux | grep serialosc     # Check if serialosc is running");
    info!("   lsusb | grep -i cafe         # Check for monome devices");
    info!("   netstat -ln | grep 12002     # Check if serialosc server is listening");
    info!("");
    info!("🔧 Manual serialosc start:");
    info!("   serialoscd                  # Run serialosc manually");
    info!("   # Then in another terminal:");
    info!("   cargo run --example test_osc_grid_discovery --features desktop");
}

fn print_osc_error_info() {
    error!("🩺 Common OSC communication issues:");
    error!("");
    error!("1. 📵 serialosc not installed:");
    error!("   sudo add-apt-repository ppa:artfwo/monome");
    error!("   sudo apt update && sudo apt install serialosc");
    error!("");
    error!("2. 🔄 serialosc not running:");
    error!("   serialoscd");
    error!("   # Or as a service:");
    error!("   sudo systemctl start serialosc");
    error!("");
    error!("3. 🔌 Grid not connected or not recognized:");
    error!("   # Check USB connection:");
    error!("   lsusb | grep -E '(cafe|0a6a)'");
    error!("   # Check serial devices:");
    error!("   ls -la /dev/ttyACM*");
    error!("");
    error!("4. 🔐 Permission issues:");
    error!("   # Add to dialout group:");
    error!("   sudo usermod -a -G dialout $USER");
    error!("   # Then log out and back in");
    error!("");
    error!("5. 🌐 Network/firewall issues:");
    error!("   # Check if serialosc server is listening:");
    error!("   netstat -ln | grep 12002");
    error!("   # Should show: udp 0 0 127.0.0.1:12002");
}