//! Pattern Save/Load Location Demonstration
//! 
//! This script shows where patterns are saved and how the system works
//! without actually running the full sequencer (for safety in demonstrations).

use std::path::PathBuf;

fn main() {
    println!("🎼 SimonSaysSeeq Pattern Save/Load System");
    println!("========================================");
    println!();
    
    // Show where patterns are saved
    let pattern_file = get_pattern_file_path();
    println!("📁 Pattern Save Location:");
    println!("   File: {:?}", pattern_file);
    
    if let Some(parent) = pattern_file.parent() {
        println!("   Directory: {:?}", parent);
    }
    
    // Check if pattern file exists
    if pattern_file.exists() {
        match std::fs::metadata(&pattern_file) {
            Ok(metadata) => {
                let size = metadata.len();
                let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                println!("   Status: ✅ Pattern file exists");
                println!("   Size: {} bytes", size);
                println!("   Last modified: {:?}", modified);
                
                // Try to peek at the file structure (first few lines)
                if let Ok(content) = std::fs::read_to_string(&pattern_file) {
                    let lines: Vec<&str> = content.lines().take(10).collect();
                    println!("   Content preview (first 10 lines):");
                    for (i, line) in lines.iter().enumerate() {
                        println!("     {}: {}", i + 1, line);
                    }
                    if content.lines().count() > 10 {
                        println!("     ... ({} total lines)", content.lines().count());
                    }
                }
            }
            Err(e) => {
                println!("   Status: ❌ Error reading file: {}", e);
            }
        }
    } else {
        println!("   Status: 📭 No saved pattern found");
        println!("   Info: First run will create a default sparse pattern");
    }
    
    println!();
    println!("🔄 How the System Works:");
    println!("=======================");
    println!();
    println!("📱 When the sequencer starts up:");
    println!("   1. Looks for saved pattern file");
    println!("   2. If found: Loads the pattern data");
    println!("   3. If not found: Creates a sparse default pattern");
    println!("   4. Default pattern has:");
    println!("      • Main grid: Kick on 1,5,9,13 - Hi-hat on 3,7,11,15 - Snare on 5,13");
    println!("      • Mozart grid: Simple melody C4→E4→G4→C5");
    println!();
    println!("🛑 When the sequencer stops:");
    println!("   • Via MIDI transport stop message (0xFC)");
    println!("   • Via manual stop button (Grid Two, column 12)");
    println!("   • Pattern is automatically saved to JSON file");
    println!("   • Transport state is NOT saved (position, running status)");
    println!("   • Only the pattern data is preserved");
    println!();
    println!("💾 What gets saved:");
    println!("   • Main sequencer grid (32x8 steps/rows)");
    println!("   • Mozart performance grid (32x8 MIDI notes)");  
    println!("   • Row states (MIDI notes, velocities, channels, euclidean settings)");
    println!("   • Pattern chains and advanced settings");
    println!("   • Slide states and parameter transitions");
    println!();
    println!("🚫 What does NOT get saved:");
    println!("   • Current playback position");
    println!("   • Running/stopped state");
    println!("   • Real-time timing counters");
    println!("   • Temporary analysis data");
    println!();
    println!("🎯 Benefits:");
    println!("   • No loss of work when stopping sequencer");
    println!("   • Consistent patterns across sessions");
    println!("   • Automatic - no user intervention needed");
    println!("   • Works with both MIDI transport and manual controls");
    println!();
    
    // Show config directory creation
    if let Some(config_dir) = dirs::config_dir() {
        let simon_config_dir = config_dir.join("simon-says-seeq");
        println!("📂 Configuration Directory:");
        println!("   Base config: {:?}", config_dir);
        println!("   SimonSaysSeeq: {:?}", simon_config_dir);
        
        if simon_config_dir.exists() {
            println!("   Status: ✅ Directory exists");
        } else {
            println!("   Status: 📁 Will be created on first save");
        }
    } else {
        println!("📂 Fallback Location:");
        println!("   Will save to current working directory");
        println!("   File: simon_says_seeq_current_pattern.json");
    }
}

/// Get the pattern file path (mirrors the sequencer internal function)
fn get_pattern_file_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        let dir = config_dir.join("simon-says-seeq");
        dir.join("current_pattern.json")
    } else {
        PathBuf::from("simon_says_seeq_current_pattern.json")
    }
}