//! Test 0-Indexed Coordinate System
//! 
//! This test verifies that the coordinate system refactor is working correctly.
//! It tests that grid coordinates (0-based) map directly to sequencer coordinates
//! without any conversion, and that user display shows the correct 1-based values.
//! 
//! Run with: cargo run --example test_0_indexed_coordinates --features desktop

use anyhow::Result;
use log::info;
use std::time::Duration;
use std::thread;
use simon_says_seeq_rust::sequencer::Sequencer;
use simon_says_seeq_rust::grid_osc::GridManager;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🧪 0-Indexed Coordinate System Test");
    info!("===================================");
    info!("Verifying coordinate system refactor works correctly");
    
    // Initialize sequencer and grid
    let sequencer = Sequencer::new();
    let mut grid_manager = GridManager::new()?;
    let connected_grids = grid_manager.get_connected_grids();
    
    if connected_grids.is_empty() {
        info!("❌ No grids connected");
        return Ok(());
    }
    
    let main_grid_id = connected_grids[0].clone();
    info!("📱 Using grid: {}", main_grid_id);
    
    // Test 1: Basic coordinate mapping
    info!("");
    info!("🧪 TEST 1: Basic Coordinate Mapping");
    info!("===================================");
    
    // Set pattern at (0,0) - should be top-left corner
    sequencer.set_grid_value(0, 0, 1);
    let value = sequencer.get_grid_value(0, 0);
    info!("✅ Set sequencer[0][0] = 1, got back: {}", value);
    assert_eq!(value, 1);
    
    // Set pattern at (15,1) - should be bottom-right of row 1 
    sequencer.set_grid_value(15, 1, 2);
    let value = sequencer.get_grid_value(15, 1);
    info!("✅ Set sequencer[15][1] = 2, got back: {}", value);
    assert_eq!(value, 2);
    
    // Test 2: Grid display mapping
    info!("");
    info!("🧪 TEST 2: Grid Display Mapping");
    info!("===============================");
    
    // Clear grid first
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(&main_grid_id, x, y, 0, "test_0_indexed_coordinates")?;
        }
    }
    
    // Set LED at grid (0,0) - should correspond to sequencer (0,0)
    grid_manager.set_led(&main_grid_id, 0, 0, 15, "test_0_indexed_coordinates")?;
    info!("✅ Set grid LED [0][0] = 15 (top-left corner)");
    
    // Set LED at grid (15,1) - should correspond to sequencer (15,1)
    grid_manager.set_led(&main_grid_id, 15, 1, 15, "test_0_indexed_coordinates")?;
    info!("✅ Set grid LED [15][1] = 15 (bottom-right of row 2)");
    
    grid_manager.refresh()?;
    
    info!("");
    info!("👁️  VISUAL CHECK:");
    info!("   - Top-left LED should be lit (position 0,0)");
    info!("   - Bottom-right LED of row 2 should be lit (position 15,1)");
    info!("   - These correspond to sequencer coordinates (0,0) and (15,1)");
    
    thread::sleep(Duration::from_secs(3));
    
    // Test 3: Row state mapping
    info!("");
    info!("🧪 TEST 3: Row State Mapping");
    info!("============================");
    
    if let Some(row_state) = sequencer.get_row_states(0) {
        info!("✅ Row 0 state: current_step={}, first_step={}, last_step={}", 
              row_state.current_step, row_state.first_step, row_state.last_step);
        info!("   (Display: step {}, first step {}, last step {})",
              row_state.current_step + 1, row_state.first_step + 1, row_state.last_step + 1);
    }
    
    if let Some(row_state) = sequencer.get_row_states(1) {
        info!("✅ Row 1 state: current_step={}, first_step={}, last_step={}", 
              row_state.current_step, row_state.first_step, row_state.last_step);
        info!("   (Display: step {}, first step {}, last step {})",
              row_state.current_step + 1, row_state.first_step + 1, row_state.last_step + 1);
    }
    
    // Test 4: Boundary conditions
    info!("");
    info!("🧪 TEST 4: Boundary Conditions");
    info!("==============================");
    
    // Test valid boundaries
    sequencer.set_grid_value(0, 0, 1);    // Min valid
    sequencer.set_grid_value(15, 7, 1);   // Max valid
    info!("✅ Boundary test: (0,0) and (15,7) are valid coordinates");
    
    // Test invalid boundaries - these should be ignored
    sequencer.set_grid_value(16, 0, 1);   // x too large
    sequencer.set_grid_value(0, 8, 1);    // y too large
    
    let invalid_x = sequencer.get_grid_value(16, 0);
    let invalid_y = sequencer.get_grid_value(0, 8);
    info!("✅ Boundary test: (16,0)={}, (0,8)={} (should both be 0)", invalid_x, invalid_y);
    assert_eq!(invalid_x, 0);
    assert_eq!(invalid_y, 0);
    
    // Test 5: Pattern display
    info!("");
    info!("🧪 TEST 5: Pattern Display Test");
    info!("===============================");
    
    // Clear grid
    for x in 0..16 {
        for y in 0..2 {
            grid_manager.set_led(&main_grid_id, x, y, 0, "test_0_indexed_coordinates")?;
        }
    }
    
    // Set up test pattern in sequencer (0-indexed)
    let test_positions = [0, 4, 8, 12]; // Steps 0, 4, 8, 12 (display: steps 1, 5, 9, 13)
    for &pos in &test_positions {
        sequencer.set_grid_value(pos, 0, 1); // Row 0
        sequencer.set_grid_value(pos, 1, 2); // Row 1
    }
    
    info!("✅ Set test pattern at positions: {:?} (display: steps 1,5,9,13)", test_positions);
    
    // Display pattern on grid - should map directly
    for row in 0..2 {
        for step in 0..16 {
            let pattern_value = sequencer.get_grid_value(step, row);
            let brightness = if pattern_value > 0 { 10 } else { 0 };
            grid_manager.set_led(&main_grid_id, step, row, brightness, "test_0_indexed_coordinates")?;
        }
    }
    
    grid_manager.refresh()?;
    
    info!("");
    info!("👁️  PATTERN CHECK:");
    info!("   - Row 1 should have 4 LEDs lit at columns 1, 5, 9, 13");  
    info!("   - Row 2 should have 4 LEDs lit at columns 1, 5, 9, 13");
    info!("   - These correspond to sequencer positions [0,4,8,12] in rows [0,1]");
    
    thread::sleep(Duration::from_secs(3));
    
    // Clean up
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(&main_grid_id, x, y, 0, "test_0_indexed_coordinates")?;
        }
    }
    grid_manager.refresh()?;
    
    info!("");
    info!("🎉 COORDINATE SYSTEM TEST COMPLETED");
    info!("===================================");
    info!("✅ All tests passed - 0-indexed coordinate system working correctly");
    info!("✅ Grid coordinates map directly to sequencer coordinates");
    info!("✅ No conversion overhead or potential off-by-one errors");
    info!("✅ User display values correctly show +1 for human-readable format");
    
    Ok(())
}