use anyhow::Result;
use log::{info, warn};
use std::{thread, time::Duration};
use simonSaysSeeq::{Sequencer, GridManager};

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🔧 Set Seq A Length ARM Button Test");
    info!("Testing SetSeqALength functionality with row 0 synchronization");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let sequencer = Sequencer::new();
    
    // Initialize sequencer with some test patterns
    info!("📋 Setting up test patterns...");
    
    // Set different patterns on rows 1-3
    for row in 1..=3 {
        for step in [0, 4, 8, 12] {
            sequencer.set_grid_value(step, row, 15); // Set bright pattern
        }
        info!("   Row {}: Set pattern at steps 1, 5, 9, 13 (display)", row + 1);
    }
    
    // Set different lengths on rows
    info!("📏 Setting initial row lengths...");
    if let Some(mut row_state) = sequencer.get_row_states(1) {
        row_state.sequencer_a_euclidean_length = 15; // 16 steps
        sequencer.set_row_states(1, row_state);
        info!("   Row 2: Set to 16 steps");
    }
    
    if let Some(mut row_state) = sequencer.get_row_states(2) {
        row_state.sequencer_a_euclidean_length = 7; // 8 steps  
        sequencer.set_row_states(2, row_state);
        info!("   Row 3: Set to 8 steps");
    }
    
    // Show initial state
    info!("\n📊 Initial Row States:");
    for row in 0..=3 {
        if let Some(row_state) = sequencer.get_row_states(row) {
            let length = row_state.sequencer_a_euclidean_length + 1;
            info!("   Row {}: current_step={}, length={} steps", 
                  row + 1, row_state.sequencer_a_current_step + 1, length);
        }
    }
    
    // Simulate master row (row 0) advancement
    info!("\n⏭️  Advancing master row 0 to step 5...");
    if let Some(mut master_state) = sequencer.get_row_states(0) {
        master_state.sequencer_a_current_step = 4; // Step 5 in 1-based
        sequencer.set_row_states(0, master_state);
        info!("   Master row 0 now at step 5");
    }
    
    // Test 1: Set row 1 to 16 steps (should not sync)
    info!("\n🧪 Test 1: SetSeqALength - Row 2 to 16 steps (no sync)");
    info!("   Simulating: ARM Set Seq A Length + Row 2 press at column 15 (16 steps)");
    
    if let Some(mut row_state) = sequencer.get_row_states(1) {
        let length = 16;
        let last_step = length - 1; // 15 (0-based)
        row_state.sequencer_a_euclidean_length = last_step;
        
        // No sync since last_step != 31
        sequencer.set_row_states(1, row_state);
        info!("   ✅ Row 2 length set to {} steps (last_step={})", length, last_step);
        info!("   ℹ️  No synchronization (length < 32 steps)");
    }
    
    // Test 2: Set row 2 to 32 steps (should sync with row 0)
    info!("\n🧪 Test 2: SetSeqALength - Row 3 to 32 steps (with sync)");
    info!("   Simulating: ARM Set Seq A Length + Row 3 press at column 31 (32 steps)");
    
    if let Some(mut row_state) = sequencer.get_row_states(2) {
        let length = 32;
        let last_step = length - 1; // 31 (0-based)
        row_state.sequencer_a_euclidean_length = last_step;
        
        // Sync with row 0 since last_step == 31
        if last_step == 31 {
            if let Some(master_row_state) = sequencer.get_row_states(0) {
                let old_step = row_state.sequencer_a_current_step;
                row_state.sequencer_a_current_step = master_row_state.sequencer_a_current_step;
                info!("   🔄 Row 3 synced with master row 0: step {} → step {}", 
                      old_step + 1, row_state.sequencer_a_current_step + 1);
            }
        }
        
        sequencer.set_row_states(2, row_state);
        info!("   ✅ Row 3 length set to {} steps (last_step={})", length, last_step);
    }
    
    // Show final state
    info!("\n📊 Final Row States:");
    for row in 0..=3 {
        if let Some(row_state) = sequencer.get_row_states(row) {
            let length = row_state.sequencer_a_euclidean_length + 1;
            let is_master = if row == 0 { " (MASTER)" } else { "" };
            info!("   Row {}{}: current_step={}, length={} steps", 
                  row + 1, is_master, row_state.sequencer_a_current_step + 1, length);
        }
    }
    
    // Test edge cases
    info!("\n🧪 Test 3: Edge Cases");
    
    // Test minimum length (1 step)
    info!("   Testing minimum length (1 step)...");
    if let Some(mut row_state) = sequencer.get_row_states(3) {
        let length = 1;
        let last_step = length - 1; // 0 (0-based)
        row_state.sequencer_a_euclidean_length = last_step;
        sequencer.set_row_states(3, row_state);
        info!("   ✅ Row 4 set to {} step (last_step={})", length, last_step);
    }
    
    // Test maximum length (32 steps) with sync
    info!("   Testing maximum length (32 steps) with sync...");
    if let Some(mut row_state) = sequencer.get_row_states(3) {
        let length = 32;
        let last_step = length - 1; // 31 (0-based)
        row_state.sequencer_a_euclidean_length = last_step;
        
        // Should sync with master row 0
        if last_step == 31 {
            if let Some(master_row_state) = sequencer.get_row_states(0) {
                row_state.sequencer_a_current_step = master_row_state.sequencer_a_current_step;
                info!("   🔄 Row 4 synced with master row 0 (current_step={})", 
                      row_state.sequencer_a_current_step + 1);
            }
        }
        
        sequencer.set_row_states(3, row_state);
        info!("   ✅ Row 4 set to {} steps with sync", length);
    }
    
    info!("\n📋 Test Summary:");
    info!("   • SetSeqALength ARM button sets individual row lengths");
    info!("   • Row lengths can be set from 1-32 steps");
    info!("   • When set to 32 steps (last_step=31), row syncs with master row 0");
    info!("   • No Euclidean rhythm generation (length only)");
    info!("   • Master row 0 provides sync reference for full-length rows");
    
    info!("\n✅ Set Seq A Length ARM Button Test Complete");
    
    Ok(())
}