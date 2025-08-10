use simon_says_seeq_rust::sequencer::Sequencer;
use simon_says_seeq_rust::config::Config;
use simon_says_seeq_rust::grid_osc::GridManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🔍 Row Scrolling Debug Test");
    println!("Testing Row 1 vs Rows 2-7 scrolling behavior");
    
    // Initialize sequencer
    let config = Config::default();
    let mut sequencer = Sequencer::new();
    
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
    
    // Clear grid
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0, "example_caller")?;
        }
    }
    grid_manager.refresh()?;
    
    // Set identical test patterns for rows 1-7
    println!("Setting test patterns...");
    for row in 1..=7 {
        sequencer.set_grid_value(1, row, 1);   // Step 1
        sequencer.set_grid_value(5, row, 1);   // Step 5  
        sequencer.set_grid_value(9, row, 1);   // Step 9
        sequencer.set_grid_value(13, row, 1);  // Step 13
    }
    
    println!("Starting sequencer...");
    sequencer.start();
    
    // Monitor and display for 10 seconds
    let start_time = std::time::Instant::now();
    let mut last_display_time = std::time::Instant::now();
    
    while start_time.elapsed().as_secs() < 10 {
        // Update grid display at 30 FPS to catch scrolling differences
        if last_display_time.elapsed().as_millis() > 33 {
            update_test_grid(&sequencer, &mut grid_manager, main_grid_id)?;
            last_display_time = std::time::Instant::now();
        }
        
        thread::sleep(Duration::from_millis(1));
    }
    
    sequencer.stop();
    
    // Clear grid
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0, "example_caller")?;
        }
    }
    grid_manager.refresh()?;
    
    println!("✅ Test complete - observe differences between Row 1 and other rows");
    Ok(())
}

fn update_test_grid(
    sequencer: &Sequencer, 
    grid_manager: &mut GridManager, 
    grid_id: &str
) -> Result<(), Box<dyn std::error::Error>> {
    
    static mut FRAME_COUNT: u32 = 0;
    unsafe { FRAME_COUNT += 1; }
    
    // Update all rows 1-7
    for seq_y in 1..=7 {
        if let Some(row_state) = sequencer.get_row_states(seq_y) {
            
            // Log Row 1 and Row 2 current_step every 30 frames (1 second)
            unsafe {
                if FRAME_COUNT % 30 == 0 && (seq_y == 1 || seq_y == 2) {
                    println!("🎯 Row {} current_step = {} (frame {})", 
                           seq_y, row_state.current_step, FRAME_COUNT);
                }
            }
            
            // Update all 16 steps for this row
            for seq_x in 1..=16 {
                let pattern_value = sequencer.get_grid_value(seq_x, seq_y);
                let is_current_step = seq_x == row_state.current_step;
                
                // Same brightness logic as main sequencer
                let brightness = match (pattern_value > 0, is_current_step) {
                    (false, false) => 0,     // No pattern, not current position
                    (false, true) => 6,      // Position only - 40%
                    (true, false) => 10,     // Pattern only - 65%
                    (true, true) => 14,      // Pattern + position - 90%
                };
                
                // Convert to 0-based coordinates and set LED
                grid_manager.set_led(grid_id, seq_x - 1, seq_y - 1, brightness, "example_caller")?;
            }
        } else {
            // Row state not found - should not happen for rows 1-7
            unsafe {
                if FRAME_COUNT % 30 == 0 {
                    println!("⚠️  No row state found for row {}", seq_y);
                }
            }
        }
    }
    
    grid_manager.refresh()?;
    Ok(())
}