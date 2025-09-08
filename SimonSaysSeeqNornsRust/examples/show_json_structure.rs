//! JSON Structure Example
//! 
//! This example creates a sequencer with some pattern data and shows the JSON structure
//! that gets saved to disk. This helps understand what data is preserved.

use simon_says_seeq_rust::sequencer::Sequencer;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    println!("🎼 JSON Pattern Structure Example");
    println!("=================================");
    println!();
    
    // Clean up any existing pattern file
    let pattern_file = get_pattern_file_path();
    if pattern_file.exists() {
        fs::remove_file(&pattern_file)?;
    }
    
    // Create a sequencer and modify it to show various data types
    {
        let sequencer = Sequencer::new();
        
        println!("📝 Creating example pattern with various data:");
        
        // Add some main grid patterns
        sequencer.set_grid_value(0, 0, 1);   // Kick on step 1
        sequencer.set_grid_value(4, 0, 1);   // Kick on step 5
        sequencer.set_grid_value(8, 0, 1);   // Kick on step 9
        sequencer.set_grid_value(12, 0, 1);  // Kick on step 13
        
        // Add some ratcheted notes
        sequencer.set_grid_value(2, 1, 2);   // Ratcheted hi-hat
        sequencer.set_grid_value(6, 1, 3);   // Triple ratchet
        sequencer.set_grid_value(14, 1, 4);  // Quad ratchet
        
        // Add some snare hits
        sequencer.set_grid_value(4, 2, 1);   // Snare on step 5
        sequencer.set_grid_value(12, 2, 1);  // Snare on step 13
        
        // Add some melody to Mozart grid (these are MIDI note values)
        sequencer.set_sequence_b_value(0, 0, 60);   // C4
        sequencer.set_sequence_b_value(2, 0, 62);   // D4
        sequencer.set_sequence_b_value(4, 0, 64);   // E4
        sequencer.set_sequence_b_value(6, 0, 65);   // F4
        sequencer.set_sequence_b_value(8, 0, 67);   // G4
        sequencer.set_sequence_b_value(12, 0, 72);  // C5
        
        // Add some bass notes
        sequencer.set_sequence_b_value(0, 1, 36);   // C2
        sequencer.set_sequence_b_value(8, 1, 43);   // G2
        
        println!("   ✅ Main grid: Kick pattern with ratcheted hi-hats and snare");
        println!("   ✅ Mozart grid: Melody (C4-D4-E4-F4-G4-C5) + bass (C2-G2)");
        
        // Save by stopping the sequencer
        sequencer.start();
        sequencer.stop();
        
        println!("   ✅ Pattern saved via sequencer stop");
    }
    
    // Read and display the JSON structure
    if pattern_file.exists() {
        let json_content = fs::read_to_string(&pattern_file)?;
        let file_size = json_content.len();
        
        println!("\n📄 Saved JSON file:");
        println!("   Location: {:?}", pattern_file);
        println!("   Size: {} bytes", file_size);
        println!("   Lines: {}", json_content.lines().count());
        
        // Parse JSON to pretty-print structure overview
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_content) {
            println!("\n🏗️  JSON Structure Overview:");
            print_json_structure(&parsed, 0);
        }
        
        println!("\n📋 First 50 lines of actual JSON:");
        println!("   (showing structure with some actual values)");
        println!();
        for (i, line) in json_content.lines().take(50).enumerate() {
            println!("{:3}: {}", i + 1, line);
        }
        
        if json_content.lines().count() > 50 {
            println!("   ... ({} more lines)", json_content.lines().count() - 50);
        }
        
        // Show some key sections in detail
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_content) {
            println!("\n🔍 Key Data Sections:");
            
            // Show main grid sample
            if let Some(main_grid) = parsed.get("sequencer_a_grid") {
                if let Some(first_column) = main_grid.as_array().and_then(|arr| arr.get(0)) {
                    println!("\n   Main Grid Column 0 (steps 1-8):");
                    if let Some(values) = first_column.as_array() {
                        for (i, val) in values.iter().enumerate() {
                            if let Some(num) = val.as_u64() {
                                if num > 0 {
                                    println!("     Step {}: {} ({})", 
                                        i + 1, num, 
                                        match num {
                                            1 => "normal hit",
                                            2 => "ratchet x2",
                                            3 => "ratchet x3", 
                                            4 => "ratchet x4",
                                            _ => "high ratchet"
                                        });
                                }
                            }
                        }
                    }
                }
            }
            
            // Show Mozart grid sample
            if let Some(mozart_grid) = parsed.get("sequencer_a_mozart") {
                if let Some(first_column) = mozart_grid.as_array().and_then(|arr| arr.get(0)) {
                    println!("\n   Mozart Grid Column 0 (MIDI notes):");
                    if let Some(values) = first_column.as_array() {
                        for (i, val) in values.iter().enumerate() {
                            if let Some(num) = val.as_u64() {
                                if num > 0 && num <= 127 {
                                    let note_name = midi_note_to_name(num as u8);
                                    println!("     Step {}: {} ({})", i + 1, num, note_name);
                                }
                            }
                        }
                    }
                }
            }
            
            // Show row states sample
            if let Some(row_states) = parsed.get("sequencer_a_row_states") {
                if let Some(first_row) = row_states.as_array().and_then(|arr| arr.get(0)) {
                    println!("\n   Row 0 Settings:");
                    if let Some(midi_note) = first_row.get("sequencer_a_midi_note") {
                        println!("     MIDI Note: {}", midi_note);
                    }
                    if let Some(velocity) = first_row.get("sequencer_a_midi_velocity") {
                        println!("     Velocity: {}", velocity);
                    }
                    if let Some(channel) = first_row.get("sequencer_a_midi_channel") {
                        println!("     MIDI Channel: {}", channel);
                    }
                    if let Some(euclidean_events) = first_row.get("sequencer_a_euclidean_events") {
                        println!("     Euclidean Events: {}", euclidean_events);
                    }
                    if let Some(euclidean_length) = first_row.get("sequencer_a_euclidean_length") {
                        println!("     Euclidean Length: {}", euclidean_length);
                    }
                }
            }
            
            // Show timing/transport settings
            println!("\n   Transport/Timing Settings:");
            if let Some(tempo) = parsed.get("tempo") {
                println!("     Tempo: {} BPM", tempo);
            }
            if let Some(swing) = parsed.get("swing_amount") {
                println!("     Swing: {}", swing);
            }
            if let Some(steps_per_bar) = parsed.get("steps_per_bar") {
                println!("     Steps per bar: {}", steps_per_bar);
            }
            if let Some(first_step) = parsed.get("first_step") {
                println!("     First step: {}", first_step);
            }
            if let Some(last_step) = parsed.get("last_step") {
                println!("     Last step: {}", last_step);
            }
        }
        
    } else {
        println!("❌ Pattern file was not created");
        return Err("Pattern file not found".into());
    }
    
    // Clean up
    if pattern_file.exists() {
        fs::remove_file(&pattern_file)?;
        println!("\n🗑️  Cleaned up example file");
    }
    
    println!("\n✨ JSON structure example complete!");
    println!("\nThe JSON file contains all sequencer state needed to recreate");
    println!("the exact pattern, including grid data, MIDI settings, and");
    println!("advanced features like euclidean rhythms and pattern chains.");
    
    Ok(())
}

/// Print JSON structure recursively
fn print_json_structure(value: &serde_json::Value, indent: usize) {
    let prefix = "  ".repeat(indent);
    
    match value {
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                match val {
                    serde_json::Value::Object(_) => {
                        println!("{}📁 {} (object)", prefix, key);
                        print_json_structure(val, indent + 1);
                    }
                    serde_json::Value::Array(arr) => {
                        println!("{}📊 {} (array[{}])", prefix, key, arr.len());
                        if !arr.is_empty() {
                            print_json_structure(&arr[0], indent + 1);
                        }
                    }
                    serde_json::Value::String(_) => {
                        println!("{}📝 {} (string)", prefix, key);
                    }
                    serde_json::Value::Number(_) => {
                        println!("{}🔢 {} (number)", prefix, key);
                    }
                    serde_json::Value::Bool(_) => {
                        println!("{}☑️ {} (boolean)", prefix, key);
                    }
                    serde_json::Value::Null => {
                        println!("{}❌ {} (null)", prefix, key);
                    }
                }
            }
        }
        _ => {
            println!("{}└─ array element type: {:?}", prefix, value);
        }
    }
}

/// Convert MIDI note number to note name
fn midi_note_to_name(note: u8) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i32 - 1;
    let note_index = (note % 12) as usize;
    format!("{}{}", note_names[note_index], octave)
}

/// Get the pattern file path (mirrors the sequencer internal function)
fn get_pattern_file_path() -> std::path::PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        let dir = config_dir.join("simon-says-seeq");
        std::fs::create_dir_all(&dir).ok();
        dir.join("current_pattern.json")
    } else {
        std::path::PathBuf::from("simon_says_seeq_current_pattern.json")
    }
}