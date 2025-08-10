use simon_says_seeq_rust::grid_osc::GridManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🔍 LED Interference Test");
    println!("Testing if setting LEDs in one row affects other rows");
    
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
            grid_manager.set_led(main_grid_id, x, y, 0)?;
        }
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(1000));
    
    println!("\nStep 1: Set Row 1 pattern and observe");
    // Set Row 1 pattern (top row)
    for seq_x in [1, 5, 9, 13] {
        println!("  Setting Row 1 Col {} -> grid({},{})", seq_x, seq_x - 1, 0);
        grid_manager.set_led(main_grid_id, seq_x - 1, 0, 15)?;
    }
    grid_manager.refresh()?;
    
    println!("Row 1 should now show 4 LEDs at columns 1,5,9,13");
    println!("OBSERVE: Are all 4 LEDs lit in Row 1? (Y/N)");
    thread::sleep(Duration::from_millis(5000));
    
    println!("\nStep 2: Set Row 6 pattern WITHOUT clearing Row 1");
    // Set Row 6 pattern while Row 1 is still lit
    for seq_x in [1, 5, 9, 13] {
        println!("  Setting Row 6 Col {} -> grid({},{})", seq_x, seq_x - 1, 5);
        grid_manager.set_led(main_grid_id, seq_x - 1, 5, 15)?;
    }
    grid_manager.refresh()?;
    
    println!("Row 6 should now ALSO show 4 LEDs at columns 1,5,9,13");
    println!("CRITICAL: Are Row 1's LEDs still all lit? (Y/N)");
    println!("If any Row 1 LEDs turned off, we found the interference bug");
    thread::sleep(Duration::from_millis(5000));
    
    println!("\nStep 3: Set Row 7 pattern");
    // Set Row 7 pattern while Rows 1 and 6 are still lit
    for seq_x in [1, 5, 9, 13] {
        println!("  Setting Row 7 Col {} -> grid({},{})", seq_x, seq_x - 1, 6);
        grid_manager.set_led(main_grid_id, seq_x - 1, 6, 15)?;
    }
    grid_manager.refresh()?;
    
    println!("Now all 3 rows should show LEDs");
    println!("CRITICAL: Are Row 1's LEDs STILL all lit? (Y/N)");
    println!("CRITICAL: Are Row 6's LEDs STILL all lit? (Y/N)");
    thread::sleep(Duration::from_millis(5000));
    
    println!("\nStep 4: Individual LED verification");
    // Check each LED individually to see which ones are actually on
    println!("Final verification - checking each expected LED individually:");
    
    // Re-set each LED individually with confirmation
    let test_positions = [
        (1, 1), (5, 1), (9, 1), (13, 1),  // Row 1
        (1, 6), (5, 6), (9, 6), (13, 6),  // Row 6  
        (1, 7), (5, 7), (9, 7), (13, 7),  // Row 7
    ];
    
    for (seq_x, seq_y) in test_positions {
        println!("  Confirming Row {} Col {} -> grid({},{})", 
                seq_y, seq_x, seq_x - 1, seq_y - 1);
        grid_manager.set_led(main_grid_id, seq_x - 1, seq_y - 1, 15)?;
    }
    grid_manager.refresh()?;
    
    println!("✅ Interference test complete");
    println!("Expected: 3 rows (1,6,7) each with 4 LEDs at columns 1,5,9,13");
    println!("If any LEDs are missing, interference bug confirmed");
    println!("Press Ctrl+C when done inspecting...");
    
    // Keep LEDs on for inspection
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}