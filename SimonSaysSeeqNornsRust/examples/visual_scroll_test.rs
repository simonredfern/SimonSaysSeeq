//! Visual Scroll Pattern Test
//! 
//! This utility creates a simple visual test to help manually observe
//! scrolling patterns and identify skips. It displays a clear pattern
//! that makes it easy to spot when steps are skipped.
//! 
//! Run with: cargo run --example visual_scroll_test --features desktop

use anyhow::Result;
use log::info;
use std::time::{Duration, Instant};
use std::thread;
use simon_says_seeq_rust::grid_osc::GridManager;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("👀 Visual Scroll Pattern Test");
    info!("=============================");
    info!("Creates a visual pattern to help identify scrolling skips");
    
    // Initialize grid
    let mut grid_manager = GridManager::new()?;
    let connected_grids = grid_manager.get_connected_grids();
    
    if connected_grids.is_empty() {
        info!("❌ No grids connected");
        return Ok(());
    }
    
    let main_grid_id = connected_grids[0].clone();
    info!("📱 Using grid: {}", main_grid_id);
    
    // Clear grid first
    clear_grid(&mut grid_manager, &main_grid_id)?;
    
    info!("");
    info!("🎯 VISUAL SCROLL TEST");
    info!("Row 1: Moving bright LED (position indicator)");  
    info!("Row 2: Static pattern (4 dim LEDs for reference)");
    info!("Row 3: Step counter pattern (shows current step number in binary)");
    info!("");
    info!("👁️  WATCH FOR:");
    info!("   - Row 1 bright LED should move smoothly 1→2→3→4...→16→1");
    info!("   - If LED jumps positions (1→3, skipping 2), that's a SKIP");
    info!("   - Row 2 reference pattern should stay constant");
    info!("   - Row 3 binary counter should increment steadily");
    info!("");
    
    // Set up reference pattern in row 2
    for &pos in &[0, 4, 8, 12] { // Positions 1, 5, 9, 13 (0-based: 0, 4, 8, 12)
        grid_manager.set_led(&main_grid_id, pos, 1, 4, "visual_scroll_test")?;
    }
    
    info!("🚀 Starting visual scroll test (60 seconds)...");
    info!("Press Ctrl+C to stop early");
    
    let test_duration = Duration::from_secs(60);
    let start_time = Instant::now();
    let step_duration = Duration::from_millis(500); // 0.5 seconds per step (slow for observation)
    
    let mut current_step = 0usize;
    let mut last_step_time = Instant::now();
    
    while start_time.elapsed() < test_duration {
        // Check if it's time to advance step
        if last_step_time.elapsed() >= step_duration {
            // Clear previous position in row 1
            if current_step > 0 {
                grid_manager.set_led(&main_grid_id, current_step - 1, 0, 0, "visual_scroll_test")?;
            } else {
                grid_manager.set_led(&main_grid_id, 15, 0, 0, "visual_scroll_test")?; // Clear position 16 (0-based: 15)
            }
            
            // Set new position in row 1 (bright)
            grid_manager.set_led(&main_grid_id, current_step, 0, 15, "visual_scroll_test")?;
            
            // Update binary counter in row 3 (show step number in binary)
            update_binary_counter(&mut grid_manager, &main_grid_id, current_step + 1)?;
            
            // Log every 4th step to track progress
            if (current_step + 1) % 4 == 0 {
                info!("Step {} → {} (cycle {})", 
                      current_step + 1, 
                      if current_step == 15 { 1 } else { current_step + 2 },
                      (current_step + 1) / 16 + 1);
            }
            
            // Advance to next step
            current_step = (current_step + 1) % 16;
            last_step_time = Instant::now();
        }
        
        // Refresh grid display
        grid_manager.refresh()?;
        
        // Small sleep to prevent excessive CPU usage
        thread::sleep(Duration::from_millis(10));
    }
    
    info!("");
    info!("🏁 Visual scroll test completed");
    info!("   Did you observe any skips in Row 1?");
    info!("   Did Row 2 reference pattern stay constant?");
    info!("   Did Row 3 binary counter increment smoothly?");
    
    // Clean up - clear grid
    clear_grid(&mut grid_manager, &main_grid_id)?;
    
    Ok(())
}

/// Update binary counter display in row 3 to show step number
fn update_binary_counter(
    grid_manager: &mut GridManager,
    grid_id: &str,
    step_number: usize
) -> Result<()> {
    // Clear row 3 first
    for x in 0..16 {
        grid_manager.set_led(grid_id, x, 2, 0, "visual_scroll_test")?;
    }
    
    // Show step number in binary (first 4 bits)
    // Step 1 = 0001, Step 2 = 0010, Step 3 = 0011, etc.
    let binary_value = step_number;
    
    // Show 4-bit binary representation
    for bit in 0..4 {
        let is_set = (binary_value >> bit) & 1 == 1;
        let brightness = if is_set { 8 } else { 1 }; // Bright for 1, dim for 0
        grid_manager.set_led(grid_id, bit, 2, brightness, "visual_scroll_test")?;
    }
    
    // Also show step number directly in positions 8-15 (one LED per step)
    if step_number <= 8 {
        grid_manager.set_led(grid_id, 7 + step_number.min(8), 2, 6, "visual_scroll_test")?;
    }
    
    Ok(())
}

/// Clear all LEDs on the grid
fn clear_grid(grid_manager: &mut GridManager, grid_id: &str) -> Result<()> {
    info!("🧹 Clearing grid...");
    
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(grid_id, x, y, 0, "visual_scroll_test")?;
        }
    }
    
    grid_manager.refresh()?;
    Ok(())
}