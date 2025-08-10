//! Debug sequencer rows - test current_step advancement and display behavior
//! 
//! This test will simulate the sequencer's row advancement and display logic
//! to identify why Row 1 works but rows 2-7 have issues.

use anyhow::Result;
use log::{info, warn, error};
use std::thread;
use std::time::Duration;

use simon_says_seeq_rust::grid_osc::GridManager;
use simon_says_seeq_rust::sequencer::{Sequencer, MainRowStates};

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🔧 Sequencer Row Diagnostics");
    info!("Testing current_step advancement and display behavior for all rows");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Initialize sequencer and grid
    let sequencer = Sequencer::new();
    let mut grid = GridManager::new()?;
    
    thread::sleep(Duration::from_millis(500));
    
    let connected_grids = grid.get_connected_grids();
    if connected_grids.is_empty() {
        error!("❌ No grids connected!");
        return Ok(());
    }
    
    let main_grid_id = &connected_grids[0];
    info!("🎹 Using grid: {}", main_grid_id);
    
    // Clear the grid
    grid.clear_all(main_grid_id)?;
    thread::sleep(Duration::from_millis(100));
    
    // Test 1: Check row settings for all rows
    info!("\n📋 TEST 1: Row Settings Analysis");
    info!("──────────────────────────────────────");
    for row in 1..=7 {
        if let Some(row_settings) = sequencer.get_row_settings(row) {
            info!("✅ Row {}: current_step={}, first_step={}, last_step={}", 
                  row, row_settings.current_step, row_settings.first_step, row_settings.last_step);
        } else {
            warn!("❌ Row {}: No row settings found!", row);
        }
    }
    
    // Test 2: Set some test patterns
    info!("\n🎨 TEST 2: Setting Test Patterns");
    info!("─────────────────────────────────────");
    for row in 1..=7 {
        // Set pattern at steps 1, 5, 9, 13 for each row
        for step in &[1, 5, 9, 13] {
            sequencer.set_grid_value(*step, row, 1);
            info!("   Set pattern: Row {}, Step {} = 1", row, step);
        }
    }
    
    // Test 3: Simulate display logic for each row
    info!("\n🖥️  TEST 3: Display Logic Simulation");
    info!("──────────────────────────────────────");
    
    // Simulate the main grid display logic
    for seq_y in 1..=7 {
        info!("\n🔍 Testing Row {} Display Logic:", seq_y);
        
        if let Some(row_state) = sequencer.get_row_settings(seq_y) {
            info!("   Row state: current_step={}, range={}-{}", 
                  row_state.current_step, row_state.first_step, row_state.last_step);
            
            // Check each column in this row
            let mut display_info = Vec::new();
            for seq_x in 1..=16 {
                let pattern_value = sequencer.get_grid_value(seq_x, seq_y);
                let is_current_step = seq_x == row_state.current_step;
                
                // Calculate brightness using same logic as main sequencer
                let brightness = match (pattern_value > 0, is_current_step) {
                    (false, false) => 0,     // 0% - No pattern, not current position
                    (false, true) => 4,      // 25% - No pattern, but current position  
                    (true, false) => 8,      // 50% - Has pattern, not current position
                    (true, true) => 12,      // 75% - Has pattern AND current position
                };
                
                if brightness > 0 || pattern_value > 0 || is_current_step {
                    display_info.push(format!("Step{}: pat={}, pos={}, bright={}", 
                                             seq_x, pattern_value, is_current_step, brightness));
                }
                
                // Set LED on actual grid
                grid.set_led(main_grid_id, seq_x - 1, seq_y - 1, brightness)?;
            }
            
            info!("   Display: {}", 
                  if display_info.is_empty() { 
                      "All LEDs off".to_string() 
                  } else { 
                      display_info.join(", ") 
                  });
                  
        } else {
            warn!("   ❌ No row settings found for row {}", seq_y);
        }
    }
    
    grid.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    // Test 4: Manual step advancement simulation
    info!("\n⏭️  TEST 4: Manual Step Advancement Simulation");
    info!("──────────────────────────────────────────────");
    info!("Simulating 4 step advances to see row behavior differences...");
    
    for advance_count in 1..=4 {
        info!("\n🎵 Step Advance #{}", advance_count);
        
        // Read current states
        let mut current_steps = Vec::new();
        for row in 1..=7 {
            if let Some(row_settings) = sequencer.get_row_settings(row) {
                current_steps.push((row, row_settings.current_step));
            }
        }
        
        // Display current steps
        let step_display: Vec<String> = current_steps.iter()
            .map(|(row, step)| format!("R{}:S{}", row, step))
            .collect();
        info!("   Before: {}", step_display.join(", "));
        
        // Simulate step advancement (this would normally happen in sequencer)
        // For testing, let's manually advance each row's current_step
        for row in 1..=7 {
            if let Some(mut row_settings) = sequencer.get_row_settings(row) {
                let old_step = row_settings.current_step;
                row_settings.current_step += 1;
                if row_settings.current_step > row_settings.last_step {
                    row_settings.current_step = row_settings.first_step;
                }
                // Note: We can't actually update the sequencer state here without internal access
                // This is just for demonstration of the logic
                info!("   Row {}: {} -> {} (would advance)", row, old_step, row_settings.current_step);
            }
        }
        
        // Update display based on new positions (simulated)
        thread::sleep(Duration::from_millis(200));
    }
    
    // Test 5: Interactive button test
    info!("\n🎮 TEST 5: Interactive Button Press Test");
    info!("────────────────────────────────────────");
    info!("Press buttons on different rows to test pattern storage and LED response:");
    info!("- Watch for immediate LED feedback");
    info!("- Check if pattern data is stored correctly");  
    info!("- Compare response times between rows");
    info!("Press Ctrl+C when done testing...");
    
    let mut press_count = 0;
    loop {
        match grid.read_button_events() {
            Ok(events) => {
                for event in events {
                    if event.grid_id == *main_grid_id && event.pressed {
                        let seq_x = event.x + 1;
                        let seq_y = event.y + 1;
                        
                        if seq_y <= 7 {
                            press_count += 1;
                            
                            info!("\n🔥 BUTTON PRESS #{} - Row {} Analysis", press_count, seq_y);
                            info!("   Coordinates: grid({},{}) -> seq({},{})", 
                                  event.x, event.y, seq_x, seq_y);
                            
                            // Get current pattern value
                            let current_value = sequencer.get_grid_value(seq_x, seq_y);
                            let new_value = if current_value > 0 { 0 } else { 1 };
                            
                            info!("   Pattern: {} -> {} (toggle)", current_value, new_value);
                            
                            // Update pattern in sequencer
                            sequencer.set_grid_value(seq_x, seq_y, new_value);
                            
                            // Get row state for position info
                            if let Some(row_state) = sequencer.get_row_settings(seq_y) {
                                let is_current_step = seq_x == row_state.current_step;
                                
                                // Calculate brightness
                                let brightness = match (new_value > 0, is_current_step) {
                                    (false, false) => 0,
                                    (false, true) => 4,
                                    (true, false) => 8,
                                    (true, true) => 12,
                                };
                                
                                info!("   Position: current_step={}, is_current={}, brightness={}", 
                                      row_state.current_step, is_current_step, brightness);
                                
                                // Update LED immediately
                                grid.set_led(main_grid_id, event.x, event.y, brightness)?;
                                
                                // Performance analysis per row
                                match seq_y {
                                    1 => info!("   🟢 Row 1: Reference behavior (should work correctly)"),
                                    2 => info!("   🔍 Row 2: First problematic row - check for differences"),
                                    3..=7 => info!("   🔍 Row {}: Compare behavior to Row 1", seq_y),
                                    _ => {}
                                }
                                
                            } else {
                                warn!("   ❌ No row settings found for row {}", seq_y);
                            }
                        }
                    }
                }
            },
            Err(e) => {
                // Normal - no events available
            }
        }
        
        thread::sleep(Duration::from_millis(1));
    }
}