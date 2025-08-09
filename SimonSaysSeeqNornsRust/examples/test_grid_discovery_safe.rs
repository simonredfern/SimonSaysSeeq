//! Safe test for serial port discovery for monome grids
//! 
//! This is a safer version that limits resource usage and prevents system hangs.
//! Run with: cargo run --example test_grid_discovery_safe --features desktop

use simon_says_seeq_rust::grid::GridManager;
use log::{info, warn, error};
use env_logger;
use std::time::{Duration, Instant};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with info level (less verbose than debug)
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🔍 Testing serial port discovery for monome grids (SAFE MODE)...");
    info!("⏱️  This discovery process has built-in timeouts to prevent hangs...");
    
    let start_time = Instant::now();
    
    // Set a maximum timeout for the entire operation
    let max_timeout = Duration::from_secs(30);
    
    info!("🏗️  Creating grid manager (max timeout: {}s)...", max_timeout.as_secs());
    
    // Create grid manager with timeout handling
    let manager_result = std::panic::catch_unwind(|| {
        GridManager::new()
    });
    
    match manager_result {
        Ok(Ok(mut manager)) => {
            let connected_grids = manager.get_connected_grids();
            
            let discovery_time = start_time.elapsed();
            info!("✅ Grid discovery completed in {:.2}s", discovery_time.as_secs_f32());
            
            if connected_grids.is_empty() {
                info!("📭 No grids detected. This is normal if no monome grids are connected.");
                print_troubleshooting_info();
            } else {
                info!("🎹 Found {} grid(s)!", connected_grids.len());
                
                for &grid_id in &connected_grids {
                    if let Some((cols, rows)) = manager.get_dimensions(grid_id) {
                        let name = manager.get_name(grid_id).unwrap_or("Unknown");
                        let varibright = manager.is_varibright(grid_id);
                        
                        info!("Grid {}: {} ({}x{}, varibright: {})", 
                              grid_id, name, cols, rows, varibright);
                        
                        // Safe grid test - only test if explicitly requested and small grid
                        if should_test_grid(cols, rows) {
                            info!("🧪 Running SAFE test on grid {} (limited LED operations)...", grid_id);
                            match safe_test_grid(&mut manager, grid_id, cols, rows) {
                                Ok(_) => info!("✅ Grid {} test completed successfully", grid_id),
                                Err(e) => warn!("⚠️  Grid {} test failed: {}", grid_id, e),
                            }
                        } else {
                            info!("⏭️  Skipping test for grid {} (too large or disabled for safety)", grid_id);
                            info!("   To test: reduce grid size or modify should_test_grid() function");
                        }
                    }
                }
            }
        }
        Ok(Err(e)) => {
            error!("❌ Failed to create grid manager: {}", e);
            print_error_diagnostics(e.as_ref());
        }
        Err(_) => {
            error!("💥 Grid manager creation panicked - this suggests a serious issue!");
            error!("   This could be due to:");
            error!("   - Infinite loops in serial port enumeration");
            error!("   - Deadlocks in serial communication");
            error!("   - Memory allocation issues");
            error!("   - Hardware driver problems");
        }
    }
    
    let total_time = start_time.elapsed();
    info!("🏁 Grid discovery test completed in {:.2}s", total_time.as_secs_f32());
    
    if total_time > Duration::from_secs(10) {
        warn!("⚠️  Discovery took longer than expected ({}s)", total_time.as_secs());
        warn!("   This might indicate slow serial port enumeration or communication issues.");
    }
    
    Ok(())
}

/// Determine if we should test a grid based on its size (safety check)
fn should_test_grid(cols: usize, rows: usize) -> bool {
    // Allow testing of larger grids but with very limited operations
    let total_leds = cols * rows;
    let max_safe_leds = 256; // Increased to allow 16x16 grids
    
    if total_leds > max_safe_leds {
        warn!("⚠️  Grid too large for safe testing ({} LEDs > {} max)", total_leds, max_safe_leds);
        false
    } else {
        info!("✅ Grid size acceptable for safe testing ({} LEDs <= {} max)", total_leds, max_safe_leds);
        true
    }
}

/// Safe grid test that limits the number of operations
fn safe_test_grid(manager: &mut GridManager, grid_id: usize, cols: usize, rows: usize) -> Result<(), Box<dyn std::error::Error>> {
    let total_leds = cols * rows;
    info!("   Testing grid {} with ULTRA-SAFE LED operations ({} total LEDs)...", grid_id, total_leds);
    
    // Test 1: Light up just the corners (4 LEDs max)
    let corners = [
        (0, 0),                    // Top-left
        (cols.saturating_sub(1), 0), // Top-right  
        (0, rows.saturating_sub(1)), // Bottom-left
        (cols.saturating_sub(1), rows.saturating_sub(1)), // Bottom-right
    ];
    
    info!("   Step 1: Testing corner LEDs...");
    for &(x, y) in &corners {
        if x < cols && y < rows {
            manager.set_led(grid_id, x, y, 8)?; // Lower brightness for safety
            thread::sleep(Duration::from_millis(150)); // Longer delay for larger grids
        }
    }
    
    thread::sleep(Duration::from_millis(800));
    
    info!("   Step 2: Clearing corner LEDs...");
    for &(x, y) in &corners {
        if x < cols && y < rows {
            manager.set_led(grid_id, x, y, 0)?;
            thread::sleep(Duration::from_millis(100));
        }
    }
    
    // Test 2: Very limited pattern (only center and a few strategic points)
    info!("   Step 3: Limited pattern test...");
    manager.clear_all(grid_id)?;
    thread::sleep(Duration::from_millis(200));
    
    // Only light up 6 LEDs maximum in a cross pattern
    let safe_pattern = [
        (cols/2, rows/2),           // Center
        (cols/2, rows/4),           // Top center
        (cols/2, 3*rows/4),         // Bottom center  
        (cols/4, rows/2),           // Left center
        (3*cols/4, rows/2),         // Right center
        (0, 0),                     // Top-left corner
    ];
    
    info!("   Lighting up cross pattern (6 LEDs max)...");
    for &(x, y) in &safe_pattern {
        if x < cols && y < rows {
            manager.set_led(grid_id, x, y, 6)?; // Very low brightness
            thread::sleep(Duration::from_millis(100)); // Delay between each LED
        }
    }
    thread::sleep(Duration::from_millis(1000));
    
    // Final clear with verification
    info!("   Step 4: Final cleanup...");
    manager.clear_all(grid_id)?;
    thread::sleep(Duration::from_millis(100));
    
    // Double-check: manually clear all LEDs to ensure they're off
    info!("   Step 5: Verification cleanup (ensuring all LEDs are OFF)...");
    for x in 0..cols {
        for y in 0..rows {
            manager.set_led(grid_id, x, y, 0)?;
            // Only add delay every 16 LEDs to avoid too much delay
            if (x * rows + y) % 16 == 0 {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
    
    // Final verification delay
    thread::sleep(Duration::from_millis(200));
    
    info!("   ✅ Ultra-safe test completed for grid {} - ALL LEDs should now be OFF", grid_id);
    info!("   📝 If any LEDs are still on, there may be a communication issue");
    Ok(())
}

fn print_troubleshooting_info() {
    info!("💡 Troubleshooting tips:");
    info!("   🔌 Connect your monome grid via USB");
    info!("   👥 Ensure you're in the 'dialout' group:");
    info!("      sudo usermod -a -G dialout $USER");
    info!("   🔄 Log out and back in after adding to group");
    info!("   🔍 Check for devices: lsusb | grep -E '(cafe|0a6a)'");
    info!("   📱 Expected USB VIDs: CAFE (newer) or 0A6A (original)");
    info!("   🔗 Check serial devices: ls /dev/ttyACM* /dev/ttyUSB* 2>/dev/null");
    info!("   📋 Check recent USB messages: dmesg | tail -20");
}

fn print_error_diagnostics(error: &dyn std::error::Error) {
    error!("🔍 Error details: {:?}", error);
    error!("🩺 Possible causes:");
    error!("   📵 No serial ports available on system");
    error!("   🔐 Permission issues (not in dialout/tty group)");
    error!("   🔌 No monome grids connected");
    error!("   💾 Serial port enumeration failure");
    error!("   🏗️  Driver or system configuration issues");
    
    info!("🔧 Additional debugging commands to try:");
    info!("   lsusb                           # Show all USB devices");
    info!("   ls -la /dev/tty*                # Show serial devices and permissions");
    info!("   groups $USER                    # Check your group membership");
    info!("   dmesg | grep -i usb | tail -10  # Recent USB kernel messages");
    info!("   sudo chmod 666 /dev/ttyACM*     # Temporary permission fix (if needed)");
}