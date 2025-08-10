use simon_says_seeq_rust::grid_osc::GridManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🔍 Basic Coordinate System Test");
    println!("Verifying grid coordinate system assumptions");
    
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
    
    println!("Test 1: Single LED at grid coordinate (0,0) - should be TOP-LEFT");
    grid_manager.set_led(main_grid_id, 0, 0, 15)?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(3000));
    
    // Clear
    grid_manager.set_led(main_grid_id, 0, 0, 0)?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("Test 2: Single LED at grid coordinate (15,0) - should be TOP-RIGHT");
    grid_manager.set_led(main_grid_id, 15, 0, 15)?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(3000));
    
    // Clear
    grid_manager.set_led(main_grid_id, 15, 0, 0)?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("Test 3: Single LED at grid coordinate (0,7) - should be BOTTOM-LEFT");
    grid_manager.set_led(main_grid_id, 0, 7, 15)?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(3000));
    
    // Clear
    grid_manager.set_led(main_grid_id, 0, 7, 0)?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("Test 4: Single LED at grid coordinate (15,7) - should be BOTTOM-RIGHT");
    grid_manager.set_led(main_grid_id, 15, 7, 15)?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(3000));
    
    // Clear
    grid_manager.set_led(main_grid_id, 15, 7, 0)?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("Test 5: Light up entire top row (row 0) - should be TOP ROW");
    for x in 0..16 {
        grid_manager.set_led(main_grid_id, x, 0, 12)?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(3000));
    
    // Clear
    for x in 0..16 {
        grid_manager.set_led(main_grid_id, x, 0, 0)?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("Test 6: Light up entire bottom row (row 7) - should be BOTTOM ROW");
    for x in 0..16 {
        grid_manager.set_led(main_grid_id, x, 7, 12)?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(3000));
    
    // Clear
    for x in 0..16 {
        grid_manager.set_led(main_grid_id, x, 7, 0)?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("Test 7: Light up entire left column (column 0) - should be LEFT COLUMN");
    for y in 0..8 {
        grid_manager.set_led(main_grid_id, 0, y, 12)?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(3000));
    
    // Clear
    for y in 0..8 {
        grid_manager.set_led(main_grid_id, 0, y, 0)?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("Test 8: Light up entire right column (column 15) - should be RIGHT COLUMN");
    for y in 0..8 {
        grid_manager.set_led(main_grid_id, 15, y, 12)?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(3000));
    
    // Clear
    for y in 0..8 {
        grid_manager.set_led(main_grid_id, 15, y, 0)?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    println!("Final Test: Expected sequencer pattern");
    println!("Setting what SHOULD be Row 1 Column 1 using our conversion logic");
    
    // Test our exact conversion logic
    let seq_x = 1;  // Sequencer column 1
    let seq_y = 1;  // Sequencer row 1
    let grid_x = seq_x - 1;  // Should be 0
    let grid_y = seq_y - 1;  // Should be 0
    
    println!("Sequencer pos ({},{}) -> Grid pos ({},{}) - should be TOP-LEFT", 
             seq_x, seq_y, grid_x, grid_y);
    
    grid_manager.set_led(main_grid_id, grid_x, grid_y, 15)?;
    grid_manager.refresh()?;
    
    println!("✅ Test complete - LEDs left on");
    println!("If TOP-LEFT LED is lit, coordinate conversion is correct");
    println!("If different LED is lit, we found the coordinate bug");
    println!("Press Ctrl+C when done inspecting...");
    
    // Keep LEDs on for inspection
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}