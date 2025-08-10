use simon_says_seeq_rust::grid_osc::GridManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🔍 Coordinate Conversion Debug Test");
    println!("Testing grid coordinate conversion for all rows");
    
    // Initialize grid manager
    let mut grid_manager = GridManager::new()?;
    
    // Wait for grid connection
    println!("Waiting for grid connection...");
    let mut grids = grid_manager.get_connected_grids();
    while grids.is_empty() {
        thread::sleep(Duration::from_millis(100));
        grids = grid_manager.get_connected_grids();
    }
    
    let main_grid_id = &grids[0];
    println!("Connected to grid: {}", main_grid_id);
    
    // Clear entire grid
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0, "example_caller")?;
        }
    }
    grid_manager.refresh()?;
    
    println!("Testing coordinate conversion for each row...");
    
    // Test each row individually with step-by-step coordinate conversion
    for seq_y in 1..=7 {
        println!("\n--- Testing Row {} ---", seq_y);
        
        // Clear grid
        for x in 0..16 {
            for y in 0..8 {
                grid_manager.set_led(main_grid_id, x, y, 0, "example_caller")?;
            }
        }
        
        // Light up specific positions for this row using same conversion as main code
        let test_positions = [1, 5, 9, 13]; // Same as main sequencer initialization
        
        for &seq_x in &test_positions {
            // EXACT same coordinate conversion as main sequencer
            let grid_x = seq_x - 1;  // Convert to 0-based
            let grid_y = seq_y - 1;  // Convert to 0-based
            
            println!("  Setting LED: seq({},{}) -> grid({},{}) brightness 10", 
                    seq_x, seq_y, grid_x, grid_y);
            
            grid_manager.set_led(main_grid_id, grid_x, grid_y, 12, "example_caller")?;
        }
        
        grid_manager.refresh()?;
        thread::sleep(Duration::from_millis(1000)); // 1 second per row
        
        // Now test position scrolling simulation for this row
        println!("  Testing position scrolling for Row {}...", seq_y);
        
        for current_step in 1..=16 {
            // Clear previous position indicator
            for x in 0..16 {
                let pattern_exists = test_positions.contains(&(x + 1));
                let brightness = if pattern_exists { 12 } else { 0 }; // Pattern only - BRIGHT
                grid_manager.set_led(main_grid_id, x, seq_y - 1, brightness, "example_caller")?;
            }
            
            // Set current position with higher brightness
            let pattern_exists = test_positions.contains(&current_step);
            let brightness = if pattern_exists { 15 } else { 8 }; // Position or pattern+position - MAX or MEDIUM
            grid_manager.set_led(main_grid_id, current_step - 1, seq_y - 1, brightness, "example_caller")?;
            
            grid_manager.refresh()?;
            thread::sleep(Duration::from_millis(100)); // Fast scroll to see movement
        }
        
        thread::sleep(Duration::from_millis(500)); // Pause between rows
    }
    
    // Final test: Show all rows simultaneously like main sequencer
    println!("\n--- Final Test: All Rows Simultaneously ---");
    
    // Clear entire grid including row 8
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0, "example_caller")?;
        }
    }
    grid_manager.refresh()?;
    
    // Test ONLY rows 1-7 (sequencer rows) with exact expected pattern
    let current_step = 3;
    for seq_y in 1..=7 {
        println!("Setting Row {} with patterns [1,5,9,13] and current_step {}", seq_y, current_step);
        
        for seq_x in 1..=16 {
            let pattern_exists = [1, 5, 9, 13].contains(&seq_x);
            let is_current_step = seq_x == current_step;
            
            let brightness = match (pattern_exists, is_current_step) {
                (false, false) => 0,   // No pattern, not current position - OFF
                (false, true) => 8,    // Position only - MEDIUM
                (true, false) => 12,   // Pattern only - BRIGHT
                (true, true) => 15,    // Pattern + position - MAXIMUM
            };
            
            if brightness > 0 {
                println!("  Row {} Step {} -> brightness {} (pattern={}, current={})", 
                        seq_y, seq_x, brightness, pattern_exists, is_current_step);
            }
            
            grid_manager.set_led(main_grid_id, seq_x - 1, seq_y - 1, brightness, "example_caller")?;
        }
    }
    
    // Ensure row 8 stays completely dark
    for x in 0..16 {
        grid_manager.set_led(main_grid_id, x, 7, 0, "example_caller")?;
    }
    
    grid_manager.refresh()?;
    
    println!("✅ Coordinate conversion test complete");
    println!("LEDs left on for inspection - describe the final pattern you see");
    println!("Expected: All 7 rows should show identical patterns with current position at step 3");
    println!("Press Ctrl+C when done inspecting...");
    
    // Keep LEDs on and wait for user inspection
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}