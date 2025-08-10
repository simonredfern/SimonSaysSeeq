use simon_says_seeq_rust::grid_osc::GridManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🔍 Row Comparison Test - Rows 1-5 vs Rows 6-7");
    println!("Isolating the difference between working and broken rows");
    
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
    
    println!("\nTest 1: Row 1 (should work correctly)");
    // Set Row 1 pattern
    for seq_x in [1, 5, 9, 13] {
        println!("  Setting Row 1 Col {} -> grid({},{})", seq_x, seq_x - 1, 0);
        grid_manager.set_led(main_grid_id, seq_x - 1, 0, 15)?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(3000));
    
    // Clear
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0)?;
        }
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("\nTest 2: Row 6 (reported as different)");
    // Set Row 6 pattern  
    for seq_x in [1, 5, 9, 13] {
        println!("  Setting Row 6 Col {} -> grid({},{})", seq_x, seq_x - 1, 5);
        grid_manager.set_led(main_grid_id, seq_x - 1, 5, 15)?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(3000));
    
    // Clear
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0)?;
        }
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("✅ Test complete - Row 1 vs Row 6 comparison done");
    println!("Did Row 1 and Row 6 show the same pattern?");
    println!("If different, we've confirmed the row-specific bug");
    Ok(())
}