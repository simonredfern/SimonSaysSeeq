//! Row Interference Diagnostic Test
//! 
//! This test isolates the interference between Row 1 and Row 2 that's causing
//! scrolling issues and Row 2 malfunction. It systematically tests:
//! - Row 1 alone (baseline)
//! - Row 2 alone (isolation)
//! - Both rows together (interference detection)
//! 
//! Run with: cargo run --example test_row_interference --features desktop

use anyhow::Result;
use log::info;
use std::time::{Duration, Instant};
use std::thread;
use simon_says_seeq_rust::grid_osc::GridManager;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🔍 Row Interference Diagnostic Test");
    info!("===================================");
    info!("Testing for interference between Row 1 and Row 2");
    
    // Initialize grid
    let mut grid_manager = GridManager::new()?;
    let connected_grids = grid_manager.get_connected_grids();
    
    if connected_grids.is_empty() {
        info!("❌ No grids connected");
        return Ok(());
    }
    
    let main_grid_id = connected_grids[0].clone();
    info!("📱 Using grid: {}", main_grid_id);
    
    // Test sequence
    clear_grid(&mut grid_manager, &main_grid_id)?;
    
    info!("");
    info!("🧪 TEST 1: Row 1 Only (Baseline)");
    info!("================================");
    test_row_1_only(&mut grid_manager, &main_grid_id)?;
    
    info!("");
    info!("🧪 TEST 2: Row 2 Only (Isolation)");
    info!("=================================");
    test_row_2_only(&mut grid_manager, &main_grid_id)?;
    
    info!("");
    info!("🧪 TEST 3: Both Rows (Interference Detection)");
    info!("=============================================");
    test_both_rows(&mut grid_manager, &main_grid_id)?;
    
    info!("");
    info!("🧪 TEST 4: Sequential Updates (Timing Test)");
    info!("===========================================");
    test_sequential_updates(&mut grid_manager, &main_grid_id)?;
    
    info!("");
    info!("🧪 TEST 5: Simultaneous Updates (Race Condition)");
    info!("================================================");
    test_simultaneous_updates(&mut grid_manager, &main_grid_id)?;
    
    // Final cleanup
    clear_grid(&mut grid_manager, &main_grid_id)?;
    
    info!("");
    info!("📊 INTERFERENCE TEST COMPLETED");
    info!("==============================");
    info!("Compare the behavior between tests:");
    info!("- Test 1: Row 1 should scroll smoothly");
    info!("- Test 2: Row 2 should scroll smoothly (if working)");
    info!("- Test 3: Both rows - look for skips or malfunction");
    info!("- Test 4: Sequential - controlled timing");
    info!("- Test 5: Simultaneous - race condition test");
    
    Ok(())
}

/// Test Row 1 scrolling alone (baseline)
fn test_row_1_only(grid_manager: &mut GridManager, grid_id: &str) -> Result<()> {
    info!("Testing Row 1 scrolling alone...");
    
    // Set reference pattern in Row 1
    for &pos in &[0, 4, 8, 12] { // Positions 1, 5, 9, 13
        grid_manager.set_led(grid_id, pos, 0, 10, "test_row_interference")?;
    }
    
    // Animate position scrolling
    for step in 0..16 {
        // Clear previous position
        if step > 0 {
            grid_manager.set_led(grid_id, step - 1, 0, 
                if [0, 4, 8, 12].contains(&(step - 1)) { 10 } else { 0 },
                "test_row_interference")?;
        }
        
        // Set new position (bright)
        grid_manager.set_led(grid_id, step, 0,
            if [0, 4, 8, 12].contains(&step) { 14 } else { 6 },
            "test_row_interference")?;
        
        grid_manager.refresh()?;
        thread::sleep(Duration::from_millis(300)); // Visible speed
        
        info!("  Row 1 step {}", step + 1);
    }
    
    info!("✅ Row 1 test completed - did it scroll smoothly?");
    thread::sleep(Duration::from_millis(1000));
    
    Ok(())
}

/// Test Row 2 scrolling alone (isolation)
fn test_row_2_only(grid_manager: &mut GridManager, grid_id: &str) -> Result<()> {
    clear_grid(grid_manager, grid_id)?;
    info!("Testing Row 2 scrolling alone...");
    
    // Set reference pattern in Row 2
    for &pos in &[0, 4, 8, 12] { // Positions 1, 5, 9, 13
        grid_manager.set_led(grid_id, pos, 1, 10, "test_row_interference")?;
    }
    
    // Animate position scrolling
    for step in 0..16 {
        // Clear previous position
        if step > 0 {
            grid_manager.set_led(grid_id, step - 1, 1,
                if [0, 4, 8, 12].contains(&(step - 1)) { 10 } else { 0 },
                "test_row_interference")?;
        }
        
        // Set new position (bright)
        grid_manager.set_led(grid_id, step, 1,
            if [0, 4, 8, 12].contains(&step) { 14 } else { 6 },
            "test_row_interference")?;
        
        grid_manager.refresh()?;
        thread::sleep(Duration::from_millis(300)); // Visible speed
        
        info!("  Row 2 step {}", step + 1);
    }
    
    info!("✅ Row 2 test completed - did it scroll smoothly?");
    thread::sleep(Duration::from_millis(1000));
    
    Ok(())
}

/// Test both rows together (interference detection)
fn test_both_rows(grid_manager: &mut GridManager, grid_id: &str) -> Result<()> {
    clear_grid(grid_manager, grid_id)?;
    info!("Testing both rows scrolling together...");
    
    // Set reference patterns in both rows
    for &pos in &[0, 4, 8, 12] {
        grid_manager.set_led(grid_id, pos, 0, 10, "test_row_interference")?; // Row 1
        grid_manager.set_led(grid_id, pos, 1, 10, "test_row_interference")?; // Row 2
    }
    
    // Animate both rows simultaneously
    for step in 0..16 {
        info!("  Both rows step {}", step + 1);
        
        // Update Row 1
        if step > 0 {
            grid_manager.set_led(grid_id, step - 1, 0,
                if [0, 4, 8, 12].contains(&(step - 1)) { 10 } else { 0 },
                "test_row_interference")?;
        }
        grid_manager.set_led(grid_id, step, 0,
            if [0, 4, 8, 12].contains(&step) { 14 } else { 6 },
            "test_row_interference")?;
        
        // Update Row 2 (same step)
        if step > 0 {
            grid_manager.set_led(grid_id, step - 1, 1,
                if [0, 4, 8, 12].contains(&(step - 1)) { 10 } else { 0 },
                "test_row_interference")?;
        }
        grid_manager.set_led(grid_id, step, 1,
            if [0, 4, 8, 12].contains(&step) { 14 } else { 6 },
            "test_row_interference")?;
        
        grid_manager.refresh()?;
        thread::sleep(Duration::from_millis(300));
    }
    
    info!("⚠️  Both rows test completed - compare with individual row tests");
    info!("   Did Row 1 have more skips than when alone?");
    info!("   Did Row 2 work better or worse than when alone?");
    thread::sleep(Duration::from_millis(1000));
    
    Ok(())
}

/// Test sequential updates (controlled timing)
fn test_sequential_updates(grid_manager: &mut GridManager, grid_id: &str) -> Result<()> {
    clear_grid(grid_manager, grid_id)?;
    info!("Testing sequential updates (Row 1, then Row 2, then refresh)...");
    
    for step in 0..8 { // Shorter test
        info!("  Sequential step {}", step + 1);
        
        // First update Row 1 completely
        grid_manager.set_led(grid_id, step, 0, 15, "test_row_interference")?;
        if step > 0 {
            grid_manager.set_led(grid_id, step - 1, 0, 0, "test_row_interference")?;
        }
        
        // Then update Row 2 completely  
        grid_manager.set_led(grid_id, step, 1, 15, "test_row_interference")?;
        if step > 0 {
            grid_manager.set_led(grid_id, step - 1, 1, 0, "test_row_interference")?;
        }
        
        // Single refresh for both
        grid_manager.refresh()?;
        thread::sleep(Duration::from_millis(400));
    }
    
    info!("✅ Sequential test completed - more stable?");
    thread::sleep(Duration::from_millis(1000));
    
    Ok(())
}

/// Test simultaneous updates (race condition)
fn test_simultaneous_updates(grid_manager: &mut GridManager, grid_id: &str) -> Result<()> {
    clear_grid(grid_manager, grid_id)?;
    info!("Testing simultaneous updates (interleaved LED commands)...");
    
    for step in 0..8 { // Shorter test
        info!("  Simultaneous step {}", step + 1);
        
        // Interleave Row 1 and Row 2 commands rapidly
        grid_manager.set_led(grid_id, step, 0, 15, "test_row_interference")?;
        grid_manager.set_led(grid_id, step, 1, 15, "test_row_interference")?;
        
        if step > 0 {
            grid_manager.set_led(grid_id, step - 1, 0, 0, "test_row_interference")?;
            grid_manager.set_led(grid_id, step - 1, 1, 0, "test_row_interference")?;
        }
        
        grid_manager.refresh()?;
        thread::sleep(Duration::from_millis(400));
    }
    
    info!("⚠️  Simultaneous test completed - worse interference?");
    thread::sleep(Duration::from_millis(1000));
    
    Ok(())
}

/// Clear all LEDs on the grid
fn clear_grid(grid_manager: &mut GridManager, grid_id: &str) -> Result<()> {
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(grid_id, x, y, 0, "test_row_interference")?;
        }
    }
    grid_manager.refresh()?;
    Ok(())
}