use simon_says_seeq_rust::grid_osc::GridManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🔍 One LED Test - Systematic Grid Testing");
    println!("Testing one LED at a time to isolate coordinate bugs");
    
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
    thread::sleep(Duration::from_millis(1000));
    
    println!("\nPhase 1: Testing Row 1 vs Row 2 - Same Column");
    
    // Test Row 1, Column 1 (should be top-left)
    println!("Setting Row 1 Column 1 -> grid(0,0)");
    grid_manager.set_led(main_grid_id, 0, 0, 15, "example_caller")?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(2000));
    
    // Clear
    grid_manager.set_led(main_grid_id, 0, 0, 0, "example_caller")?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    // Test Row 2, Column 1 (should be second row, left)
    println!("Setting Row 2 Column 1 -> grid(0,1)");
    grid_manager.set_led(main_grid_id, 0, 1, 15, "example_caller")?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(2000));
    
    // Clear
    grid_manager.set_led(main_grid_id, 0, 1, 0, "example_caller")?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("\nPhase 2: Testing All Rows - Column 5");
    
    // Test column 5 for all rows 1-7
    for row in 1..=7 {
        let grid_x = 4;  // Column 5 -> grid coordinate 4
        let grid_y = row - 1;  // Row N -> grid coordinate N-1
        
        println!("Setting Row {} Column 5 -> grid({},{})", row, grid_x, grid_y);
        grid_manager.set_led(main_grid_id, grid_x, grid_y, 15, "example_caller")?;
        grid_manager.refresh()?;
        thread::sleep(Duration::from_millis(1500));
        
        // Clear this LED
        grid_manager.set_led(main_grid_id, grid_x, grid_y, 0, "example_caller")?;
        grid_manager.refresh()?;
        thread::sleep(Duration::from_millis(300));
    }
    
    println!("\nPhase 3: Using Sequencer Logic");
    
    // Now test using the exact same coordinate conversion as sequencer
    for seq_y in 1..=7 {
        for seq_x in [1, 5, 9, 13] {
            println!("Sequencer logic: seq({},{}) -> grid({},{})", 
                    seq_x, seq_y, seq_x - 1, seq_y - 1);
            
            grid_manager.set_led(main_grid_id, seq_x - 1, seq_y - 1, 15, "example_caller")?;
            grid_manager.refresh()?;
            thread::sleep(Duration::from_millis(800));
            
            // Clear this LED
            grid_manager.set_led(main_grid_id, seq_x - 1, seq_y - 1, 0, "example_caller")?;
            grid_manager.refresh()?;
            thread::sleep(Duration::from_millis(200));
        }
    }
    
    println!("\nPhase 4: Final Pattern - All Rows Together");
    
    // Set the expected pattern for all rows simultaneously
    for seq_y in 1..=7 {
        for seq_x in [1, 5, 9, 13] {
            grid_manager.set_led(main_grid_id, seq_x - 1, seq_y - 1, 12, "example_caller")?;
        }
    }
    grid_manager.refresh()?;
    
    println!("✅ All tests complete - final pattern displayed");
    println!("Expected: 7 rows, each with 4 LEDs at columns 1,5,9,13");
    println!("Row 8 should be completely dark");
    println!("If Row 1 looks different from others, we found the bug location");
    println!("Press Ctrl+C when done inspecting...");
    
    // Keep LEDs on for inspection
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}