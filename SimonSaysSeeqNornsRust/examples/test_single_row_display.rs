use simon_says_seeq_rust::sequencer::Sequencer;
use simon_says_seeq_rust::grid_osc::GridManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🔍 Single Row Display Test");
    println!("Testing display logic for individual rows");
    
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
    
    // Test each row individually with focused display logic
    for test_row in 1..=7 {
        println!("\n--- Testing Row {} Display ---", test_row);
        
        // Clear entire grid
        for x in 0..16 {
            for y in 0..8 {
                grid_manager.set_led(main_grid_id, x, y, 0)?;
            }
        }
        grid_manager.refresh()?;
        
        // Set pattern ONLY for this test row
        sequencer.set_grid_value(1, test_row, 1);   // Column 1
        sequencer.set_grid_value(5, test_row, 1);   // Column 5  
        sequencer.set_grid_value(9, test_row, 1);   // Column 9
        sequencer.set_grid_value(13, test_row, 1);  // Column 13
        
        // Display ONLY this test row (not all rows)
        for seq_x in 1..=16 {
            let pattern_value = sequencer.get_grid_value(seq_x, test_row);
            let brightness = if pattern_value > 0 { 15 } else { 0 };
            
            println!("  Row {} Col {}: pattern_value={}, brightness={}, grid_coord=({},{})", 
                    test_row, seq_x, pattern_value, brightness, seq_x - 1, test_row - 1);
            
            // Use exact same coordinate conversion as main sequencer
            grid_manager.set_led(main_grid_id, seq_x - 1, test_row - 1, brightness)?;
        }
        
        grid_manager.refresh()?;
        println!("  Row {} should show 4 bright LEDs at columns 1,5,9,13", test_row);
        println!("  All other rows should be completely dark");
        thread::sleep(Duration::from_millis(4000));
        
        // Clear patterns for next test
        sequencer.set_grid_value(1, test_row, 0);
        sequencer.set_grid_value(5, test_row, 0);
        sequencer.set_grid_value(9, test_row, 0);
        sequencer.set_grid_value(13, test_row, 0);
    }
    
    println!("\n--- Summary Test: Verify Each Row Individually ---");
    
    // Final verification: Light up one LED per row for easy identification
    for test_row in 1..=7 {
        // Clear grid
        for x in 0..16 {
            for y in 0..8 {
                grid_manager.set_led(main_grid_id, x, y, 0)?;
            }
        }
        
        // Light up ONLY column 8 (middle) for this row
        println!("Lighting Row {} Column 8 -> grid({},{})", test_row, 7, test_row - 1);
        grid_manager.set_led(main_grid_id, 7, test_row - 1, 15)?;
        grid_manager.refresh()?;
        
        println!("Should see ONE LED in Row {} Column 8 (middle)", test_row);
        thread::sleep(Duration::from_millis(2000));
    }
    
    // Final state: Clear grid
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0)?;
        }
    }
    grid_manager.refresh()?;
    
    println!("✅ Single row display test complete");
    println!("If Row 1 worked correctly but others didn't, display logic has row-specific bug");
    Ok(())
}