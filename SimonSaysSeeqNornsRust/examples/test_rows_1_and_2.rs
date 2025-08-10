//! Test Rows 1 and 2 Together - Combined Scrolling Test
//! 
//! This test runs rows 1 and 2 simultaneously with scrolling to see if 
//! interference between rows causes the scrolling issues observed in the main sequencer.
//! 
//! Run with: cargo run --example test_rows_1_and_2 --features desktop

use simon_says_seeq_rust::grid_osc::GridManager;
use simon_says_seeq_rust::config::Config;
use std::time::Duration;
use std::thread;
use log::{info, warn, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🎯 Rows 1 and 2 Together - Combined Scrolling Test");
    info!("==================================================");
    info!("");
    info!("This test runs both row 1 and row 2 simultaneously");
    info!("to check for interference between rows during scrolling.");
    info!("");
    
    // Load config or use default
    let _config = Config::default();
    
    // Initialize grid
    info!("🔍 Initializing grid connection...");
    let mut grid = match GridManager::new() {
        Ok(grid) => {
            info!("✅ Grid connected successfully");
            grid
        }
        Err(e) => {
            error!("❌ Failed to connect to grid: {}", e);
            return Ok(());
        }
    };
    
    // Wait for grid connection
    info!("⏳ Waiting for grid connection...");
    let mut connected_grids = grid.get_connected_grids();
    let mut wait_count = 0;
    while connected_grids.is_empty() && wait_count < 30 {
        thread::sleep(Duration::from_millis(100));
        connected_grids = grid.get_connected_grids();
        wait_count += 1;
    }
    
    let grid_id = match connected_grids.first() {
        Some(id) => {
            info!("📱 Using grid: {}", id);
            id.as_str()
        }
        None => {
            error!("❌ No grid available after 3 seconds");
            return Ok(());
        }
    };
    
    // Clear the entire grid
    info!("🧹 Clearing grid...");
    for y in 0..8 {
        for x in 0..16 {
            grid.set_led(&grid_id, x, y, 0)?;
        }
    }
    thread::sleep(Duration::from_millis(100));
    
    // Set up patterns for both rows
    info!("🎨 Setting up patterns for rows 1 and 2...");
    let row_1_y = 0; // Row 1 (top row)
    let row_2_y = 1; // Row 2 (second row)
    let pattern_positions = [0, 4, 8, 12]; // Steps 1, 5, 9, 13 (0-based: 0, 4, 8, 12)
    let pattern_brightness = 10;
    
    // Set patterns on both rows
    for &x in &pattern_positions {
        // Row 1 pattern
        grid.set_led(&grid_id, x, row_1_y, pattern_brightness)?;
        // Row 2 pattern
        grid.set_led(&grid_id, x, row_2_y, pattern_brightness)?;
    }
    
    info!("✅ Row 1 pattern set at positions: {:?}", pattern_positions.iter().map(|&x| x + 1).collect::<Vec<_>>());
    info!("✅ Row 2 pattern set at positions: {:?}", pattern_positions.iter().map(|&x| x + 1).collect::<Vec<_>>());
    
    // Wait for user confirmation
    println!();
    println!("🔍 Check the grid - you should see 4 LEDs lit on BOTH row 1 (top) and row 2 (second from top)");
    println!("Press Enter to start combined scrolling test...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    info!("");
    info!("🎬 Starting Rows 1 and 2 Combined Scrolling Test");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("Both rows will now scroll through all 16 positions SIMULTANEOUSLY.");
    info!("Watch for smooth position advancement on BOTH rows from left to right.");
    info!("Each position should light up for about 500ms.");
    info!("");
    
    let position_brightness = 6;
    let combined_brightness = 14; // Pattern + position
    let step_duration = Duration::from_millis(500);
    
    // Scroll through all 16 positions on both rows simultaneously
    for cycle in 0..3 {
        info!("🔄 Cycle {} of 3", cycle + 1);
        
        for current_step in 0..16 {
            let step_number = current_step + 1;
            info!("  Both Rows Step {}/16", step_number);
            
            // Clear previous position on both rows (if not first step)
            if current_step > 0 {
                let prev_step = current_step - 1;
                let prev_brightness = if pattern_positions.contains(&prev_step) { 
                    pattern_brightness 
                } else { 
                    0 
                };
                // Clear previous on row 1
                grid.set_led(&grid_id, prev_step, row_1_y, prev_brightness)?;
                // Clear previous on row 2  
                grid.set_led(&grid_id, prev_step, row_2_y, prev_brightness)?;
            } else if cycle > 0 {
                // Clear position 16 from previous cycle on both rows
                let prev_brightness = if pattern_positions.contains(&15) { 
                    pattern_brightness 
                } else { 
                    0 
                };
                grid.set_led(&grid_id, 15, row_1_y, prev_brightness)?;
                grid.set_led(&grid_id, 15, row_2_y, prev_brightness)?;
            }
            
            // Set current position on both rows
            let current_brightness = if pattern_positions.contains(&current_step) {
                combined_brightness // Both pattern and position
            } else {
                position_brightness // Position only
            };
            
            // Set current position on row 1
            grid.set_led(&grid_id, current_step, row_1_y, current_brightness)?;
            // Set current position on row 2
            grid.set_led(&grid_id, current_step, row_2_y, current_brightness)?;
            
            thread::sleep(step_duration);
        }
    }
    
    // Clear the position indicators on both rows, leave patterns
    info!("🧹 Clearing position indicators...");
    let final_brightness = if pattern_positions.contains(&15) { 
        pattern_brightness 
    } else { 
        0 
    };
    grid.set_led(&grid_id, 15, row_1_y, final_brightness)?;
    grid.set_led(&grid_id, 15, row_2_y, final_brightness)?;
    
    info!("");
    info!("✅ Rows 1 and 2 combined scrolling test completed!");
    info!("");
    info!("📊 Results Analysis:");
    info!("━━━━━━━━━━━━━━━━━━━━");
    
    println!();
    println!("Please answer the following questions about what you observed:");
    println!();
    
    print!("1. Did Row 1 position indicator move smoothly from left to right? (y/n): ");
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    let row1_smooth = input.trim().to_lowercase().starts_with('y');
    
    print!("2. Did Row 2 position indicator move smoothly from left to right? (y/n): ");
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    let row2_smooth = input.trim().to_lowercase().starts_with('y');
    
    print!("3. Did both rows scroll identically and in sync? (y/n): ");
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    let rows_in_sync = input.trim().to_lowercase().starts_with('y');
    
    print!("4. Did you see any interference between the rows (one affecting the other)? (y/n): ");
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    let saw_interference = input.trim().to_lowercase().starts_with('y');
    
    print!("5. Did both rows complete all 3 full cycles (1→16, 3 times each)? (y/n): ");
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    let completed_cycles = input.trim().to_lowercase().starts_with('y');
    
    info!("");
    info!("📈 TEST RESULTS:");
    info!("  Row 1 smooth scrolling:     {}", if row1_smooth { "✅ YES" } else { "❌ NO" });
    info!("  Row 2 smooth scrolling:     {}", if row2_smooth { "✅ YES" } else { "❌ NO" });
    info!("  Rows scrolled in sync:      {}", if rows_in_sync { "✅ YES" } else { "❌ NO" });
    info!("  Interference observed:      {}", if saw_interference { "❌ YES" } else { "✅ NO" });
    info!("  Completed 3 cycles:         {}", if completed_cycles { "✅ YES" } else { "❌ NO" });
    
    info!("");
    info!("💡 ANALYSIS:");
    
    if row1_smooth && row2_smooth && rows_in_sync && !saw_interference && completed_cycles {
        info!("🎉 EXCELLENT: Both rows scroll perfectly together!");
        info!("   This suggests no interference between rows 1 and 2.");
        info!("   The main sequencer issue may be:");
        info!("   - Related to more than 2 rows simultaneously");
        info!("   - Timing issues with sequencer thread coordination");
        info!("   - Grid update frequency problems with full sequencer");
    } else if row1_smooth && !row2_smooth {
        warn!("⚠️  Row 1 works but Row 2 fails - consistent with main sequencer issue");
        warn!("   The problem persists even with just 2 rows");
        warn!("   This suggests fundamental Row 2 issue that isolated test missed");
    } else if !row1_smooth && !row2_smooth {
        warn!("⚠️  Both rows fail when run together");
        warn!("   This suggests interference or resource contention between rows");
        warn!("   The issue appears when multiple rows are active simultaneously");
    } else if saw_interference {
        warn!("⚠️  Interference detected between rows");
        warn!("   This could be:");
        warn!("   - OSC message timing conflicts");
        warn!("   - Grid update coordination issues");
        warn!("   - Race conditions in LED state management");
    } else {
        info!("🤔 Mixed results - partial functionality");
        info!("   Some aspects work while others don't");
        info!("   This suggests specific timing or coordination issues");
    }
    
    // Clean up
    info!("");
    info!("🧹 Final cleanup...");
    for x in 0..16 {
        grid.set_led(&grid_id, x, row_1_y, 0)?;
        grid.set_led(&grid_id, x, row_2_y, 0)?;
    }
    
    info!("");
    info!("Test completed. Rows 1 and 2 combined scrolling analysis complete.");
    
    Ok(())
}

use std::io::Write;