use simon_says_seeq_rust::sequencer::Sequencer;
use simon_says_seeq_rust::grid_osc::GridManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🔍 Pattern Storage Verification Test");
    println!("Testing if pattern storage works correctly for all rows");
    
    // Initialize sequencer and grid
    let sequencer = Sequencer::new();
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
    
    // Clear grid
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0, "example_caller")?;
        }
    }
    grid_manager.refresh()?;
    
    println!("\nTesting pattern storage for each row...");
    
    // Test pattern storage for each row individually
    for test_row in 1..=7 {
        println!("\n--- Testing Row {} Pattern Storage ---", test_row);
        
        // Clear all patterns first
        for row in 1..=7 {
            for col in 1..=16 {
                sequencer.set_grid_value(col, row, 0);
            }
        }
        
        // Set specific pattern ONLY for this test row
        let pattern_columns = [1, 5, 9, 13];
        for &col in &pattern_columns {
            sequencer.set_grid_value(col, test_row, 1);
            println!("  Set pattern: sequencer({}, {}) = 1", col, test_row);
        }
        
        // Verify storage by reading back
        println!("  Reading back stored values:");
        for col in 1..=16 {
            let value = sequencer.get_grid_value(col, test_row);
            if value > 0 {
                println!("    sequencer({}, {}) = {}", col, test_row, value);
            }
        }
        
        // Display on grid using exact same logic as main sequencer
        for seq_y in 1..=7 {
            for seq_x in 1..=16 {
                let pattern_value = sequencer.get_grid_value(seq_x, seq_y);
                let brightness = if pattern_value > 0 { 12 } else { 0 };
                
                // Use main sequencer coordinate conversion
                grid_manager.set_led(main_grid_id, seq_x - 1, seq_y - 1, brightness, "example_caller")?;
            }
        }
        
        grid_manager.refresh()?;
        println!("  Grid should show Row {} with LEDs at columns 1,5,9,13 only", test_row);
        thread::sleep(Duration::from_millis(2000));
    }
    
    println!("\n--- Final Test: All Rows Together ---");
    
    // Set patterns for ALL rows
    for row in 1..=7 {
        for &col in &[1, 5, 9, 13] {
            sequencer.set_grid_value(col, row, 1);
        }
    }
    
    // Display all rows together
    for seq_y in 1..=7 {
        for seq_x in 1..=16 {
            let pattern_value = sequencer.get_grid_value(seq_x, seq_y);
            let brightness = if pattern_value > 0 { 12 } else { 0 };
            grid_manager.set_led(main_grid_id, seq_x - 1, seq_y - 1, brightness, "example_caller")?;
        }
    }
    
    grid_manager.refresh()?;
    
    println!("✅ Pattern storage test complete");
    println!("All 7 rows should show identical patterns at columns 1,5,9,13");
    println!("If any row is different, pattern storage has row-specific bugs");
    println!("Press Ctrl+C when done inspecting...");
    
    // Keep LEDs on for inspection
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}