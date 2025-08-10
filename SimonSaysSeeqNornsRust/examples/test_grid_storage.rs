//! Minimal grid storage test to isolate the bug
//! 
//! This test will directly verify grid storage and retrieval
//! to identify why patterns aren't being stored correctly for rows 2-7

use anyhow::Result;
use log::info;
use simon_says_seeq_rust::sequencer::Sequencer;

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🔧 Grid Storage Bug Test");
    info!("Testing pattern storage and retrieval for all rows");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let sequencer = Sequencer::new();
    
    // Test 1: Set one pattern per row at step 1
    info!("\n📝 Setting patterns: Row X, Step 1 = X");
    for row in 1..=7 {
        sequencer.set_grid_value(1, row, row as u8); // Step 1, Row X = value X
        info!("   Set: grid[1][{}] = {}", row, row);
    }
    
    // Test 2: Retrieve and verify patterns
    info!("\n📖 Reading back patterns:");
    for row in 1..=7 {
        let value = sequencer.get_grid_value(1, row);
        let expected = row as u8;
        let status = if value == expected { "✅" } else { "❌" };
        info!("   {} Read: grid[1][{}] = {} (expected {})", status, row, value, expected);
        
        if value != expected {
            info!("   🚨 STORAGE BUG: Row {} not storing correctly!", row);
        }
    }
    
    // Test 3: Set multiple patterns per row
    info!("\n📝 Setting multiple patterns per row:");
    for row in 1..=7 {
        for step in &[5, 9, 13] {
            let value = row as u8 + 10; // Different value to distinguish from test 1
            sequencer.set_grid_value(*step, row, value);
            info!("   Set: grid[{}][{}] = {}", step, row, value);
        }
    }
    
    // Test 4: Verify multiple patterns
    info!("\n📖 Reading back multiple patterns:");
    for row in 1..=7 {
        let mut row_ok = true;
        let mut pattern_info = Vec::new();
        
        for step in &[5, 9, 13] {
            let value = sequencer.get_grid_value(*step, row);
            let expected = row as u8 + 10;
            let status = if value == expected { "✅" } else { "❌" };
            pattern_info.push(format!("S{}:{}{}", step, status, value));
            
            if value != expected {
                row_ok = false;
            }
        }
        
        let row_status = if row_ok { "✅" } else { "❌" };
        info!("   {} Row {}: {}", row_status, row, pattern_info.join(" "));
    }
    
    // Test 5: Boundary testing
    info!("\n🔬 Boundary Testing:");
    
    // Test corners of grid
    let test_coords = [
        (1, 1, "top-left"),
        (16, 1, "top-right"), 
        (1, 7, "bottom-left"),
        (16, 7, "bottom-right"),
        (8, 4, "center"),
    ];
    
    for (step, row, desc) in &test_coords {
        let test_value = 99u8;
        sequencer.set_grid_value(*step, *row, test_value);
        let read_value = sequencer.get_grid_value(*step, *row);
        let status = if read_value == test_value { "✅" } else { "❌" };
        info!("   {} {} grid[{}][{}]: wrote {}, read {}", 
              status, desc, step, row, test_value, read_value);
    }
    
    // Test 6: Invalid coordinates (should be ignored)
    info!("\n⚠️  Invalid Coordinate Testing:");
    sequencer.set_grid_value(0, 1, 88); // x=0 invalid
    sequencer.set_grid_value(1, 0, 88); // y=0 invalid  
    sequencer.set_grid_value(17, 1, 88); // x=17 invalid
    sequencer.set_grid_value(1, 9, 88); // y=9 invalid (row 8 is valid but y=9 is not)
    
    let invalid_reads = [
        (0, 1), (1, 0), (17, 1), (1, 9)
    ];
    
    for (x, y) in &invalid_reads {
        let value = sequencer.get_grid_value(*x, *y);
        info!("   Invalid coord ({},{}): read {} (should be 0)", x, y, value);
    }
    
    info!("\n🎯 Test Complete!");
    info!("If rows 2-7 show ❌, that confirms the storage bug.");
    info!("All rows should behave identically for the bug to be fixed.");
    
    Ok(())
}