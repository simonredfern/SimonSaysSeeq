use simon_says_seeq_rust::sequencer::Sequencer;
use simon_says_seeq_rust::grid_osc::GridManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🔍 Main Sequencer Scrolling Monitor");
    println!("Monitoring actual main sequencer row states and display");
    
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
            grid_manager.set_led(main_grid_id, x, y, 0, "test_main_sequencer_scrolling", "example_caller")?;
        }
    }
    grid_manager.refresh()?;
    
    // Set test patterns for all rows like main sequencer
    println!("\nSetting test patterns for all rows:");
    for row in 1..=7 {
        sequencer.set_grid_value(1, row, 1);   // Step 1
        sequencer.set_grid_value(5, row, 1);   // Step 5  
        sequencer.set_grid_value(9, row, 1);   // Step 9
        sequencer.set_grid_value(13, row, 1);  // Step 13
        println!("  Set Row {} patterns at columns 1,5,9,13", row);
    }
    
    // Start the sequencer (this starts the clock thread)
    println!("\nStarting main sequencer...");
    sequencer.start();
    
    // Monitor for 20 seconds and manually update display
    println!("Monitoring sequencer for 20 seconds...");
    println!("OBSERVE: Does Row 1 scroll smoothly? Do Rows 2-7 flicker/behave differently?");
    
    let start_time = std::time::Instant::now();
    let mut frame_count = 0;
    
    while start_time.elapsed().as_secs() < 20 {
        // Update grid display using exact main sequencer logic
        update_main_grid_display(&sequencer, &mut grid_manager, main_grid_id)?;
        
        // Log row states periodically
        frame_count += 1;
        if frame_count % 30 == 0 { // Every 30 frames (~1 second)
            println!("\n--- Frame {} ---", frame_count);
            for row in 0..=2 { // Monitor first 3 rows (0-indexed)
                if let Some(row_state) = sequencer.get_row_states(row) {
                    println!("  Row {} current_step = {} [display: row {}]", row, row_state.current_step, row + 1);
                }
            }
        }
        
        thread::sleep(Duration::from_millis(33)); // ~30 FPS display updates
    }
    
    sequencer.stop();
    
    // Clear grid
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0, "test_main_sequencer_scrolling", "example_caller")?;
        }
    }
    grid_manager.refresh()?;
    
    println!("\n✅ Main sequencer monitoring complete");
    println!("Did you observe the Row 1 vs Rows 2-7 scrolling difference?");
    println!("This test uses the exact same logic as the main application");
    
    Ok(())
}

/// Exact copy of main sequencer's grid display logic
fn update_main_grid_display(
    sequencer: &Sequencer, 
    grid_manager: &mut GridManager, 
    grid_id: &str
) -> Result<(), Box<dyn std::error::Error>> {
    
    // Grid display with position scrolling - 4 brightness levels (EXACT main sequencer logic)
    for seq_y in 0..=6 {
        let row_states = sequencer.get_row_states(seq_y);
        if let Some(row_state) = row_states {
            for seq_x in 0..=15 {
                let pattern_value = sequencer.get_grid_value(seq_x, seq_y);
                let is_current_step = seq_x == row_state.current_step;
                
                // 4 brightness levels based on pattern and position (EXACT main sequencer logic):
                let brightness = match (pattern_value > 0, is_current_step) {
                    (false, false) => 0,     // No pattern, not current position
                    (false, true) => 6,      // Position only - 40%
                    (true, false) => 10,     // Pattern only - 65%
                    (true, true) => 14,      // Pattern + position - 90%
                };
                
                // Use native 0-based coordinates directly (EXACT main sequencer logic)
                grid_manager.set_led(grid_id, seq_x, seq_y, brightness, "update_main_grid_display", "example_caller")?;
            }
        }
    }
    
    grid_manager.refresh()?;
    Ok(())
}