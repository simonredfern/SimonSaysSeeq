//! Test Row 2 Only - Isolated Scrolling Test
//! 
//! This test runs ONLY row 2 with scrolling to see if it works correctly
//! when there's no interference from other rows or complex sequencer logic.
//! 
//! Run with: cargo run --example test_row2_only --features desktop

use simon_says_seeq_rust::grid_osc::GridManager;
use simon_says_seeq_rust::config::Config;
use std::time::Duration;
use std::thread;
use log::{info, warn, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🎯 Row 2 Only - Isolated Scrolling Test");
    info!("=======================================");
    info!("");
    info!("This test focuses exclusively on row 2 scrolling behavior");
    info!("with no interference from other rows or complex logic.");
    info!("");
    
    // Load config or use default
    let config = Config::default();
    
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
    
    // Set up row 2 pattern (y=1 in 0-based coordinates)
    info!("🎨 Setting up row 2 pattern...");
    let row_2_y = 1;
    let pattern_positions = [1, 5, 9, 13]; // Steps 2, 6, 10, 14 (0-based: 1, 5, 9, 13)
    let pattern_brightness = 10;
    
    // Set the pattern
    for &x in &pattern_positions {
        grid.set_led(&grid_id, x, row_2_y, pattern_brightness)?;
    }
    
    info!("✅ Row 2 pattern set at positions: {:?}", pattern_positions.iter().map(|&x| x + 1).collect::<Vec<_>>());
    
    // Wait for user confirmation
    println!();
    println!("🔍 Check the grid - you should see 4 LEDs lit on row 2 (second row from top)");
    println!("Press Enter to start scrolling test...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    info!("");
    info!("🎬 Starting Row 2 Only Scrolling Test");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("Row 2 will now scroll through all 16 positions.");
    info!("Watch for smooth position advancement from left to right.");
    info!("Each position should light up for about 500ms.");
    info!("");
    
    let position_brightness = 6;
    let combined_brightness = 14; // Pattern + position
    let step_duration = Duration::from_millis(500);
    
    // Scroll through all 16 positions on row 2 only
    for cycle in 0..3 {
        info!("🔄 Cycle {} of 3", cycle + 1);
        
        for current_step in 0..16 {
            let step_number = current_step + 1;
            info!("  Row 2 Step {}/16", step_number);
            
            // Clear previous position (if not first step)
            if current_step > 0 {
                let prev_step = current_step - 1;
                let prev_brightness = if pattern_positions.contains(&prev_step) { 
                    pattern_brightness 
                } else { 
                    0 
                };
                grid.set_led(&grid_id, prev_step, row_2_y, prev_brightness)?;
            } else if cycle > 0 {
                // Clear position 16 from previous cycle
                let prev_brightness = if pattern_positions.contains(&15) { 
                    pattern_brightness 
                } else { 
                    0 
                };
                grid.set_led(&grid_id, 15, row_2_y, prev_brightness)?;
            }
            
            // Set current position
            let current_brightness = if pattern_positions.contains(&current_step) {
                combined_brightness // Both pattern and position
            } else {
                position_brightness // Position only
            };
            
            grid.set_led(&grid_id, current_step, row_2_y, current_brightness)?;
            
            thread::sleep(step_duration);
        }
    }
    
    // Clear the position indicator, leave pattern
    info!("🧹 Clearing position indicator...");
    let final_brightness = if pattern_positions.contains(&15) { 
        pattern_brightness 
    } else { 
        0 
    };
    grid.set_led(&grid_id, 15, row_2_y, final_brightness)?;
    
    info!("");
    info!("✅ Row 2 scrolling test completed!");
    info!("");
    info!("📊 Results Analysis:");
    info!("━━━━━━━━━━━━━━━━━━━━");
    
    println!();
    println!("Please answer the following questions about what you observed:");
    println!();
    
    print!("1. Did you see the position indicator (bright light) move smoothly from left to right? (y/n): ");
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    let smooth_scrolling = input.trim().to_lowercase().starts_with('y');
    
    print!("2. Did the position indicator complete all 3 full cycles (1→16, 3 times)? (y/n): ");
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    let completed_cycles = input.trim().to_lowercase().starts_with('y');
    
    print!("3. Did the position indicator correctly combine with patterns (brighter when overlapping)? (y/n): ");
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    let correct_combining = input.trim().to_lowercase().starts_with('y');
    
    print!("4. Did you see any flickering, jumping, or irregular behavior? (y/n): ");
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    let saw_irregularities = input.trim().to_lowercase().starts_with('y');
    
    info!("");
    info!("📈 TEST RESULTS:");
    info!("  Smooth scrolling:           {}", if smooth_scrolling { "✅ YES" } else { "❌ NO" });
    info!("  Completed 3 cycles:         {}", if completed_cycles { "✅ YES" } else { "❌ NO" });
    info!("  Correct pattern combining:  {}", if correct_combining { "✅ YES" } else { "❌ NO" });
    info!("  Irregularities observed:    {}", if saw_irregularities { "❌ YES" } else { "✅ NO" });
    
    info!("");
    info!("💡 ANALYSIS:");
    
    if smooth_scrolling && completed_cycles && correct_combining && !saw_irregularities {
        info!("🎉 EXCELLENT: Row 2 scrolling works perfectly in isolation!");
        info!("   This suggests the row 2 issue is caused by interference");
        info!("   from other rows or complex sequencer logic in the main app.");
        info!("");
        info!("🔍 NEXT STEPS:");
        info!("   1. The issue is likely in main sequencer coordination");
        info!("   2. Check for race conditions between rows");
        info!("   3. Investigate grid update timing in main app");
        info!("   4. Look for row state synchronization issues");
    } else if !smooth_scrolling {
        warn!("⚠️  Row 2 scrolling issue confirmed even in isolation");
        warn!("   This indicates a fundamental problem with:");
        warn!("   - Row 2 coordinate mapping");
        warn!("   - Grid hardware communication for row 2");
        warn!("   - OSC command generation for row 2");
    } else if saw_irregularities {
        warn!("⚠️  Row 2 has intermittent issues");
        warn!("   Scrolling mostly works but has occasional problems");
        warn!("   This could indicate timing or hardware issues");
    } else {
        info!("🤔 Mixed results - some aspects work, others don't");
        info!("   This suggests partial functionality with specific issues");
    }
    
    // Clean up
    info!("");
    info!("🧹 Final cleanup...");
    for x in 0..16 {
        grid.set_led(&grid_id, x, row_2_y, 0)?;
    }
    
    info!("");
    info!("Test completed. Row 2 isolated scrolling analysis complete.");
    
    Ok(())
}

use std::io::Write;