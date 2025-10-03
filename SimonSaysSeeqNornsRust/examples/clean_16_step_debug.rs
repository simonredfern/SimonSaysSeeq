use anyhow::Result;
use log::info;
use simon_says_seeq_rust::sequencer::{Sequencer, SequencerEvent};
use crossbeam_channel::unbounded;

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🔧 Clean 16-Step MIDI Debug Test");
    info!("Starting with fresh sequencer to isolate the 16-step MIDI issue");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut sequencer = Sequencer::new();
    let (seq_tx, seq_rx) = unbounded();
    
    // First, reset row 0 to default 32-step settings
    if let Some(mut row_state) = sequencer.get_row_states(0) {
        row_state.sequencer_a_first_step = 0;
        row_state.sequencer_a_euclidean_length = 31; // 32 steps (0-31)
        row_state.sequencer_a_current_step = 0;
        sequencer.set_row_states(0, row_state);
        info!("✅ Reset row 0 to default: first_step=0, euclidean_length=31");
    }
    
    // Clear any existing pattern and set up fresh test pattern
    info!("📋 Setting up fresh test pattern in row 0...");
    for step in 0..32 {
        sequencer.set_grid_value(step, 0, 0); // Clear all
    }
    // Set pattern at steps 0, 4, 8, 12, 16, 20, 24, 28
    for step in (0..32).step_by(4) {
        sequencer.set_grid_value(step, 0, 15); // Max brightness
        info!("   Set step {} active", step);
    }
    
    info!("\n🧪 Test 1: Full 32-step sequence (should work correctly)");
    
    let mut midi_full = Vec::new();
    sequencer.start();
    
    for step_num in 0..33 {
        sequencer.external_advance_step(&seq_tx)?;
        
        while let Ok(event) = seq_rx.try_recv() {
            if let SequencerEvent::MidiEvent(midi_event) = event {
                if midi_event.note_on && midi_event.channel == 1 {
                    midi_full.push(midi_event.step);
                    info!("   🎵 MIDI at step {}", midi_event.step);
                }
            }
        }
    }
    
    sequencer.stop();
    info!("   📊 Full 32-step MIDI: {:?}", midi_full);
    
    // Clear events
    while seq_rx.try_recv().is_ok() {}
    
    info!("\n🧪 Test 2: Set row 0 to 16-step sequence (reproduce the bug)");
    
    // Now set row 0 to 16 steps exactly as SetSeqALength does
    if let Some(mut row_state) = sequencer.get_row_states(0) {
        let length = 16;
        let last_step = length - 1; // 15
        row_state.sequencer_a_euclidean_length = last_step;
        row_state.sequencer_a_first_step = 0; // Ensure starting from 0
        
        // Reset current step if beyond new length
        if row_state.sequencer_a_current_step > last_step {
            row_state.sequencer_a_current_step = 0;
        }
        
        sequencer.set_row_states(0, row_state);
        info!("   ✅ Set row 0 to 16 steps: first_step=0, euclidean_length=15");
        
        // Verify the settings took effect
        if let Some(check_state) = sequencer.get_row_states(0) {
            info!("   📋 Verified: first_step={}, euclidean_length={}, current_step={}", 
                  check_state.sequencer_a_first_step, 
                  check_state.sequencer_a_euclidean_length,
                  check_state.sequencer_a_current_step);
        }
    }
    
    info!("   Expected: MIDI should trigger at steps 0, 4, 8, 12 (within 16-step range)");
    info!("   Problem: MIDI should be silent when master goes to steps 16, 20, 24, 28");
    
    let mut midi_16 = Vec::new();
    sequencer.start();
    
    for step_num in 0..40 {
        sequencer.external_advance_step(&seq_tx)?;
        
        while let Ok(event) = seq_rx.try_recv() {
            if let SequencerEvent::MidiEvent(midi_event) = event {
                if midi_event.note_on && midi_event.channel == 1 {
                    midi_16.push(midi_event.step);
                    info!("   🎵 MIDI at step {}", midi_event.step);
                }
            }
        }
        
        // Show relationship every 8 steps for clarity
        if step_num % 8 == 0 {
            let (master_step, _) = sequencer.get_current_position();
            if let Some(row_state) = sequencer.get_row_states(0) {
                info!("   📍 Step {}: Master={}, Row0={} (range=0-{})", 
                      step_num, master_step, row_state.sequencer_a_current_step, 
                      row_state.sequencer_a_euclidean_length);
            }
        }
    }
    
    sequencer.stop();
    info!("   📊 16-step sequence MIDI: {:?}", midi_16);
    
    info!("\n🔍 Analysis:");
    info!("   Expected pattern for 16-step: [0, 4, 8, 12, 0, 4, 8, 12, 0, 4, 8, 12, ...]");
    info!("   If MIDI is working: we should see repeating pattern of steps 0,4,8,12");
    info!("   If MIDI is broken: we'll see silence when row step doesn't match expected grid positions");
    
    let expected_pattern = vec![0, 4, 8, 12];
    let is_correct = midi_16.iter().all(|&step| expected_pattern.contains(&step));
    
    if is_correct && !midi_16.is_empty() {
        info!("   ✅ MIDI appears to be working correctly!");
    } else {
        info!("   ❌ MIDI issue confirmed - debug output above shows the problem");
    }
    
    Ok(())
}