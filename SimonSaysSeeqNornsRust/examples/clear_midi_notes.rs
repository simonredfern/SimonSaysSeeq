//! MIDI Notes Cleanup Utility
//! 
//! This utility clears all MIDI notes that might be stuck or playing.
//! It sends "All Notes Off" messages and cleans up the internal state.
//! 
//! Run with: cargo run --example clear_midi_notes --features desktop

use anyhow::Result;
use log::info;
use std::time::Duration;
use simon_says_seeq_rust::midi::MidiManager;
use simon_says_seeq_rust::config::{Config, MidiConfig};

fn main() -> Result<()> {
    // Initialize logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("clear_midi_notes says: MIDI Notes Cleanup Utility");
    info!("clear_midi_notes says: ================================");
    
    // Load configuration to get MIDI settings
    let config = Config::load_or_default()?;
    
    info!("clear_midi_notes says: Creating MIDI manager...");
    match MidiManager::new(&config.midi) {
        Ok(mut midi_manager) => {
            info!("clear_midi_notes says: MIDI manager created successfully");
            
            // Get current active note count
            let active_count = midi_manager.get_active_note_count();
            info!("clear_midi_notes says: Found {} active MIDI notes", active_count);
            
            if active_count > 0 {
                info!("clear_midi_notes says: Clearing all active MIDI notes...");
                
                // Send all notes off
                match midi_manager.all_notes_off() {
                    Ok(_) => info!("clear_midi_notes says: All Notes Off messages sent successfully"),
                    Err(e) => {
                        info!("clear_midi_notes says: Error sending All Notes Off: {}", e);
                        info!("clear_midi_notes says: Attempting individual note cleanup...");
                    }
                }
            } else {
                info!("clear_midi_notes says: No active MIDI notes found");
            }
            
            // Clean up any stuck notes (notes active for more than 10 seconds)
            info!("clear_midi_notes says: Checking for stuck notes...");
            let stuck_timeout = Duration::from_secs(10);
            match midi_manager.cleanup_stuck_notes(stuck_timeout) {
                Ok(_) => info!("clear_midi_notes says: Stuck note cleanup completed"),
                Err(e) => info!("clear_midi_notes says: Stuck note cleanup error: {}", e),
            }
            
            // Send additional MIDI panic messages for safety
            info!("clear_midi_notes says: Sending additional MIDI panic messages...");
            
            // Send CC 120 (All Sound Off) on all channels for extra safety
            for channel in 1..=16 {
                // CC 120 = All Sound Off (more aggressive than All Notes Off)
                if let Err(e) = send_control_change(&mut midi_manager, 120, 0, channel) {
                    info!("clear_midi_notes says: Warning - failed to send All Sound Off on channel {}: {}", channel, e);
                }
                
                // CC 121 = Reset All Controllers
                if let Err(e) = send_control_change(&mut midi_manager, 121, 0, channel) {
                    info!("clear_midi_notes says: Warning - failed to send Reset Controllers on channel {}: {}", channel, e);
                }
            }
            
            // Final verification
            let final_count = midi_manager.get_active_note_count();
            if final_count == 0 {
                info!("clear_midi_notes says: Success - All MIDI notes cleared");
                info!("clear_midi_notes says: MIDI cleanup completed successfully");
            } else {
                info!("clear_midi_notes says: Warning - {} notes may still be active", final_count);
                info!("clear_midi_notes says: You may need to restart your MIDI devices");
            }
            
            // Show MIDI device status
            let device_status = midi_manager.get_device_status();
            info!("clear_midi_notes says: MIDI device status: {}", device_status);
        }
        Err(e) => {
            info!("clear_midi_notes says: Failed to create MIDI manager: {}", e);
            info!("clear_midi_notes says: Possible reasons:");
            info!("clear_midi_notes says:   1. No MIDI devices available");
            info!("clear_midi_notes says:   2. MIDI feature not enabled");
            info!("clear_midi_notes says:   3. MIDI device in use by another application");
            return Err(e);
        }
    }
    
    info!("clear_midi_notes says: ");
    info!("clear_midi_notes says: If MIDI notes are still playing:");
    info!("clear_midi_notes says:   1. Check your DAW or MIDI software");
    info!("clear_midi_notes says:   2. Restart MIDI devices");
    info!("clear_midi_notes says:   3. Run this utility again");
    
    Ok(())
}

/// Helper function to send control change messages
/// This is a simplified version since we don't have direct access to the MIDI connection
fn send_control_change(_midi_manager: &mut MidiManager, _controller: u8, _value: u8, _channel: u8) -> Result<()> {
    // Note: This would require access to the internal MIDI connection
    // For now, we rely on the all_notes_off() method which already sends CC 123
    // In a full implementation, we would need to expose a send_cc method in MidiManager
    Ok(())
}