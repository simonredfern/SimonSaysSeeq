use simon_says_seeq_rust::grid_osc::GridManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🔍 Simple Coordinate Mapping Test");
    
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
    thread::sleep(Duration::from_millis(500));
    
    println!("Testing coordinate mapping - one LED per row");
    
    // Test exact coordinate mapping: light up column 1 for each row
    for seq_y in 1..=7 {
        let grid_x = 0;  // Column 1 -> grid coordinate 0
        let grid_y = seq_y - 1;  // Convert seq_y to 0-based
        
        println!("Setting Row {} Column 1: seq({},{}) -> grid({},{}) brightness 15", 
                seq_y, 1, seq_y, grid_x, grid_y);
        
        grid_manager.set_led(main_grid_id, grid_x, grid_y, 15)?;
    }
    
    grid_manager.refresh()?;
    println!("\nPhase 1: Column 1 should be lit for rows 1-7 only");
    thread::sleep(Duration::from_millis(3000));
    
    // Clear and test column 16
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0)?;
        }
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("Testing column 16 for each row");
    for seq_y in 1..=7 {
        let grid_x = 15;  // Column 16 -> grid coordinate 15
        let grid_y = seq_y - 1;  // Convert seq_y to 0-based
        
        println!("Setting Row {} Column 16: seq({},{}) -> grid({},{}) brightness 15", 
                seq_y, 16, seq_y, grid_x, grid_y);
        
        grid_manager.set_led(main_grid_id, grid_x, grid_y, 15)?;
    }
    
    grid_manager.refresh()?;
    println!("\nPhase 2: Column 16 should be lit for rows 1-7 only");
    thread::sleep(Duration::from_millis(3000));
    
    // Clear and test row boundaries
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0)?;
        }
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("Testing row boundaries - lighting entire rows");
    for seq_y in 1..=7 {
        println!("Lighting entire Row {}", seq_y);
        for seq_x in 1..=16 {
            let grid_x = seq_x - 1;
            let grid_y = seq_y - 1;
            grid_manager.set_led(main_grid_id, grid_x, grid_y, 8)?;
        }
        grid_manager.refresh()?;
        thread::sleep(Duration::from_millis(1000));
        
        // Clear this row before next
        for seq_x in 1..=16 {
            let grid_x = seq_x - 1;
            let grid_y = seq_y - 1;
            grid_manager.set_led(main_grid_id, grid_x, grid_y, 0)?;
        }
        grid_manager.refresh()?;
        thread::sleep(Duration::from_millis(300));
    }
    
    println!("Final test: Light up specific coordinates");
    // Test specific coordinates to verify mapping
    let test_coords = [
        (1, 1),   // Should be top-left
        (16, 1),  // Should be top-right  
        (1, 7),   // Should be bottom-left of sequencer area
        (16, 7),  // Should be bottom-right of sequencer area
    ];
    
    for (seq_x, seq_y) in test_coords {
        let grid_x = seq_x - 1;
        let grid_y = seq_y - 1;
        println!("Setting corner: seq({},{}) -> grid({},{}) brightness 15", 
                seq_x, seq_y, grid_x, grid_y);
        grid_manager.set_led(main_grid_id, grid_x, grid_y, 15)?;
    }
    
    grid_manager.refresh()?;
    println!("\nFinal: Four corners should be lit (top-left, top-right, bottom-left, bottom-right)");
    println!("Row 8 should be completely dark");
    println!("Press Ctrl+C when done inspecting...");
    
    // Keep LEDs on for inspection
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}