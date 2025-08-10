use anyhow::Result;
use log::info;
use simon_says_seeq_rust::midi::MidiManager;
use simon_says_seeq_rust::config::{Config, MidiConfig};

fn main() -> Result<()> {
    // Initialize logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("list_midi_notes says: Starting MIDI note listing utility");
    
    // Load configuration to get MIDI settings
    let config = Config::load_or_default()?;
    
    // Create MIDI manager
    match MidiManager::new(&config.midi) {
        Err(e) => {
            info!("list_midi_notes says: Failed to create MIDI manager: {}", e);
            return Err(e);
        }
        Ok(midi_manager) => {
            info!("list_midi_notes says: MIDI manager created successfully");
            
            // Get active note count
            let active_count = midi_manager.get_active_note_count();
            info!("list_midi_notes says: Found {} active MIDI notes", active_count);
            
            if active_count > 0 {
                // Get detailed active notes
                let active_notes = midi_manager.get_active_notes();
                
                info!("list_midi_notes says: Active MIDI Notes:");
                info!("list_midi_notes says: ========================");
                info!("list_midi_notes says: Note | Velocity | Channel");
                info!("list_midi_notes says: -----|----------|--------");
                
                for (note, velocity, channel) in active_notes {
                    let note_name = note_to_name(note);
                    info!("list_midi_notes says: {:4} | {:8} | {:7}", 
                          format!("{} ({})", note_name, note), velocity, channel);
                }
                
                info!("list_midi_notes says: ========================");
                info!("list_midi_notes says: Total: {} active notes", active_count);
            } else {
                info!("list_midi_notes says: No active MIDI notes found");
                info!("list_midi_notes says: MIDI system is clean");
            }
        }
    }
    
    Ok(())
}

/// Convert MIDI note number to note name
fn note_to_name(note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note as i32 / 12) - 1; // MIDI note 60 = C4
    let note_index = note as usize % 12;
    format!("{}{}", note_names[note_index], octave)
}