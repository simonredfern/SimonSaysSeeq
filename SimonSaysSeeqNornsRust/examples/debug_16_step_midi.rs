use anyhow::Result;
use log::info;
use simon_says_seeq_rust::sequencer::{Sequencer, SequencerEvent};
use crossbeam_channel::unbounded;

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🔧 Debug 16-Step MIDI Issue");
    info!("Reproducing the exact issue: MIDI silent on second 16 steps");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut sequencer = Sequencer::new();
    let (seq_tx, seq_rx) = unbounded();
    
    // Set up a simple pattern in row 0: notes at steps 0, 4, 8, 12, 16, 20, 24, 28
    info!("📋 Setting up test pattern in row 0 (every 4th step)...");
    for step in (0..32).step_by(4) {
        sequencer.set_grid_value(step, 0, 15); // Max brightness
        info!("   Set step {} active", step);
    }
    
    info!("\n🧪 First test: Full 32-step sequence (default behavior)");
    info!("   Expected MIDI at steps: 0, 4, 8, 12, 16, 20, 24, 28");
    
    let mut midi_notes = Vec::new();
    sequencer.start();
    
    // Run through one complete cycle (33 steps to see wraparound)
    for _step_num in 0..33 {
        sequencer.external_advance_step(&seq_tx)?;
        
        // Process any MIDI events
        while let Ok(event) = seq_rx.try_recv() {
            if let SequencerEvent::MidiEvent(midi_event) = event {
                if midi_event.note_on && midi_event.channel == 1 { // Row 0 is channel 1
                    midi_notes.push(midi_event.step);
                    info!("   ✅ MIDI at step {}", midi_event.step);
                }
            }
        }
    }
    
    sequencer.stop();
    info!("   📊 Full sequence MIDI notes: {:?}", midi_notes);
    
    // Clear events
    while seq_rx.try_recv().is_ok() {}
    
    info!("\n🧪 Second test: Set row 0 to 16-step sequence");
    info!("   Using SetSeqALength logic: euclidean_length = 15 (16 steps: 0-15)");
    
    // Set row 0 to 16 steps using the same logic as SetSeqALength
    if let Some(mut row_state) = sequencer.get_row_states(0) {
        let length = 16;
        let last_step = length - 1; // 15
        row_state.sequencer_a_euclidean_length = last_step;
        
        // Reset current step position if it's beyond the new length
        let current_step = if row_state.sequencer_a_current_step > last_step {
            row_state.sequencer_a_current_step = row_state.sequencer_a_first_step;
            row_state.sequencer_a_current_step
        } else {
            row_state.sequencer_a_current_step
        };
        
        sequencer.set_row_states(0, row_state);
        info!("   ✅ Row 0 set to {} steps (euclidean_length={})", length, last_step);
        info!("   ✅ Row 0 current_step reset to: {}", current_step);
    }
    
    info!("   Expected behavior: MIDI should trigger at steps 0, 4, 8, 12 only");
    info!("   Problem: MIDI should be silent when master reaches steps 16, 20, 24, 28");
    
    let mut midi_notes_16 = Vec::new();
    sequencer.start();
    
    // Run through more than one cycle to see the issue
    for step_num in 0..40 {
        sequencer.external_advance_step(&seq_tx)?;
        
        // Process any MIDI events
        while let Ok(event) = seq_rx.try_recv() {
            if let SequencerEvent::MidiEvent(midi_event) = event {
                if midi_event.note_on && midi_event.channel == 1 { // Row 0 is channel 1
                    midi_notes_16.push(midi_event.step);
                    info!("   ✅ MIDI at step {}", midi_event.step);
                }
            }
        }
        
        // Show master vs row step relationship every 4 steps
        if step_num % 4 == 0 {
            let (master_step, _) = sequencer.get_current_position();
            if let Some(row_state) = sequencer.get_row_states(0) {
                info!("   📍 Step {}: Master={}, Row0={} (length=0-{})", 
                      step_num, master_step, row_state.sequencer_a_current_step, row_state.sequencer_a_euclidean_length);
            }
        }
    }
    
    sequencer.stop();
    info!("   📊 16-step sequence MIDI notes: {:?}", midi_notes_16);
    
    info!("\n🔍 Analysis:");
    info!("   If MIDI is working correctly, we should see notes only when row step matches active grid positions");
    info!("   If MIDI is broken, we'll see silence during certain master step ranges");
    info!("   The debug output above should show exactly what's happening in the MIDI logic");
    
    Ok(())
}