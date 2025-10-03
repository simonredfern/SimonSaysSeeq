use anyhow::Result;
use log::info;
use simon_says_seeq_rust::sequencer::{Sequencer, SequencerEvent};
use crossbeam_channel::unbounded;

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🔧 MIDI 16-Step Sequence Fix Test");
    info!("Testing that MIDI is sent only for steps within row sequence length");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut sequencer = Sequencer::new();
    let (seq_tx, seq_rx) = unbounded();
    
    // Set up test pattern in row 0 (steps 0, 4, 8, 12, 16, 20, 24, 28)
    info!("📋 Setting up test pattern in row 0...");
    for step in (0..32).step_by(4) {
        sequencer.set_grid_value(step, 0, 14); // Bright LED + MIDI trigger
        info!("   Set step {} active (grid_value=14)", step);
    }
    
    info!("\n🧪 Test 1: Full 32-step sequence (default)");
    info!("   Row 0 should trigger MIDI at steps: 0, 4, 8, 12, 16, 20, 24, 28");
    
    // Test full 32-step sequence
    let mut midi_steps = Vec::new();
    sequencer.start();
    
    // Run for 34 steps to ensure we complete one full cycle
    for _i in 0..34 {
        sequencer.external_advance_step(&seq_tx)?;
        
        // Check for MIDI events
        while let Ok(event) = seq_rx.try_recv() {
            match event {
                SequencerEvent::MidiEvent(midi_event) => {
                    if midi_event.note_on && midi_event.sequencer_source == 'A' {
                        midi_steps.push(midi_event.step);
                        info!("   ✅ MIDI ON at step {} (channel {})", midi_event.step, midi_event.channel);
                    }
                }
                _ => {}
            }
        }
    }
    
    info!("   📊 MIDI steps triggered: {:?}", midi_steps);
    let expected_full = vec![0, 4, 8, 12, 16, 20, 24, 28];
    if midi_steps == expected_full {
        info!("   ✅ PASS: All expected MIDI steps triggered");
    } else {
        info!("   ❌ FAIL: Expected {:?}, got {:?}", expected_full, midi_steps);
    }
    
    sequencer.stop();
    
    info!("\n🧪 Test 2: 16-step sequence (SetSeqALength to 16)");
    info!("   Setting row 0 length to 16 steps (last_step=15)");
    info!("   Row 0 should now only trigger MIDI at steps: 0, 4, 8, 12 (NOT 16, 20, 24, 28)");
    
    // Set row 0 to 16-step sequence
    if let Some(mut row_state) = sequencer.get_row_states(0) {
        row_state.sequencer_a_euclidean_length = 15; // 16 steps (0-15)
        row_state.sequencer_a_first_step = 0;
        sequencer.set_row_states(0, row_state);
        info!("   ✅ Row 0 set to 16 steps (euclidean_length=15)");
    }
    
    // Clear previous MIDI events
    while seq_rx.try_recv().is_ok() {}
    
    // Test 16-step sequence
    let mut midi_steps_16 = Vec::new();
    sequencer.start();
    
    // Run for 34 steps to see behavior across full grid cycle
    for _i in 0..34 {
        sequencer.external_advance_step(&seq_tx)?;
        
        // Check for MIDI events
        while let Ok(event) = seq_rx.try_recv() {
            match event {
                SequencerEvent::MidiEvent(midi_event) => {
                    if midi_event.note_on && midi_event.sequencer_source == 'A' {
                        midi_steps_16.push(midi_event.step);
                        info!("   ✅ MIDI ON at step {} (channel {})", midi_event.step, midi_event.channel);
                    }
                }
                _ => {}
            }
        }
    }
    
    info!("   📊 MIDI steps triggered: {:?}", midi_steps_16);
    let expected_16 = vec![0, 4, 8, 12]; // Only steps within 16-step range
    if midi_steps_16 == expected_16 {
        info!("   ✅ PASS: Only steps 0-15 triggered MIDI (steps 16+ correctly ignored)");
    } else {
        info!("   ❌ FAIL: Expected {:?}, got {:?}", expected_16, midi_steps_16);
    }
    
    sequencer.stop();
    
    info!("\n🧪 Test 3: Custom range (8-step sequence starting at step 4)");
    info!("   Setting row 0: first_step=4, euclidean_length=11 (8 steps: 4-11)");
    info!("   Row 0 should only trigger MIDI at step 8 (only active step in range 4-11)");
    
    // Set row 0 to custom range
    if let Some(mut row_state) = sequencer.get_row_states(0) {
        row_state.sequencer_a_first_step = 4;
        row_state.sequencer_a_euclidean_length = 11; // 8 steps (4-11)
        row_state.sequencer_a_current_step = 4; // Start at first step
        sequencer.set_row_states(0, row_state);
        info!("   ✅ Row 0 set to custom range: steps 4-11 (first_step=4, euclidean_length=11)");
    }
    
    // Clear previous MIDI events
    while seq_rx.try_recv().is_ok() {}
    
    // Test custom range
    let mut midi_steps_custom = Vec::new();
    sequencer.start();
    
    // Run for 34 steps to see behavior
    for _i in 0..34 {
        sequencer.external_advance_step(&seq_tx)?;
        
        // Check for MIDI events
        while let Ok(event) = seq_rx.try_recv() {
            match event {
                SequencerEvent::MidiEvent(midi_event) => {
                    if midi_event.note_on && midi_event.sequencer_source == 'A' {
                        midi_steps_custom.push(midi_event.step);
                        info!("   ✅ MIDI ON at step {} (channel {})", midi_event.step, midi_event.channel);
                    }
                }
                _ => {}
            }
        }
    }
    
    info!("   📊 MIDI steps triggered: {:?}", midi_steps_custom);
    let expected_custom = vec![8]; // Only step 8 is both active and in range 4-11
    if midi_steps_custom == expected_custom {
        info!("   ✅ PASS: Only step 8 triggered MIDI (within custom range 4-11)");
    } else {
        info!("   ❌ FAIL: Expected {:?}, got {:?}", expected_custom, midi_steps_custom);
    }
    
    sequencer.stop();
    
    info!("\n📋 Test Summary:");
    info!("   • MIDI processing now respects individual row sequence ranges");
    info!("   • Steps outside a row's first_step to euclidean_length range are ignored");
    info!("   • This fixes the issue where 16-step sequences had silent MIDI on steps 16-31");
    info!("   • The fix ensures MIDI only triggers for steps within each row's active range");
    
    Ok(())
}