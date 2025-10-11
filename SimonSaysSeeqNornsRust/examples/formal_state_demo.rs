use anyhow::Result;
use log::info;
use simon_says_seeq_rust::formal_state_logger::{self, log_test_injection, ButtonSource};
use std::fs;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🎛️  Formal State Logging System Demo");
    info!("═══════════════════════════════════════════════════════════");
    info!("This demo shows how the formal state logging system works:");
    info!("1. Logs are written to formal_state.log in structured JSON format");
    info!("2. Button presses, LED changes, MIDI events, and state changes are tracked");
    info!("3. Test mode uses SysEx injection (see direct_test binary for examples)");
    info!("═══════════════════════════════════════════════════════════");
    
    // Initialize formal state logger
    formal_state_logger::init_formal_logger()?;
    info!("✅ Formal state logger initialized");
    
    // Log initial system state
    let initial_state = r#"{"demo": "initial_state", "version": "1.0", "sequencer_rows": 7, "grid_columns": 32}"#;
    formal_state_logger::log_system_init(initial_state);
    info!("📋 Initial system state logged");
    
    info!("\n🧪 Demonstrating different types of formal state events...\n");
    
    // 1. Button press simulation
    info!("1️⃣  Simulating button presses:");
    formal_state_logger::log_button_press("grid_one", 0, 0, ButtonSource::Hardware);
    formal_state_logger::log_button_release("grid_one", 0, 0, ButtonSource::Hardware);
    info!("   ✅ Button press/release logged for grid_one at (0,0)");
    
    thread::sleep(Duration::from_millis(100));
    
    // 2. LED changes
    info!("2️⃣  Simulating LED changes:");
    formal_state_logger::log_led_change("grid_one", 0, 0, 15, "demo_bright_led");
    formal_state_logger::log_led_change("grid_one", 0, 0, 0, "demo_led_off");
    info!("   ✅ LED changes logged for grid_one at (0,0)");
    
    thread::sleep(Duration::from_millis(100));
    
    // 3. MIDI events
    info!("3️⃣  Simulating MIDI events:");
    formal_state_logger::log_midi_note_on(60, 100, 1, 0, 0); // C4, velocity 100, channel 1, row 0, step 0
    formal_state_logger::log_midi_note_off(60, 1, 0); // C4, channel 1, row 0
    info!("   ✅ MIDI note on/off logged for C4 (note 60)");
    
    thread::sleep(Duration::from_millis(100));
    
    // 4. ARM actions
    info!("4️⃣  Simulating ARM actions:");
    formal_state_logger::log_arm_action_activated("SetSeqALength", 8);
    formal_state_logger::log_arm_action_executed("SetSeqALength", 0, 15, "Set row 0 to 16 steps");
    info!("   ✅ ARM action activation and execution logged");
    
    thread::sleep(Duration::from_millis(100));
    
    // 5. Step advancement
    info!("5️⃣  Simulating step advancement:");
    let row_steps = vec![(0, 1), (1, 5), (2, 3), (3, 7), (4, 2), (5, 9), (6, 4)];
    formal_state_logger::log_step_advancement(1, 0, row_steps);
    info!("   ✅ Step advancement logged with all row positions");
    
    thread::sleep(Duration::from_millis(100));
    
    // 6. Sequencer state changes
    info!("6️⃣  Simulating sequencer state changes:");
    formal_state_logger::log_sequencer_state_change(
        "sequence_length", 
        "32", 
        "16", 
        Some(0)
    );
    info!("   ✅ Sequencer state change logged");
    
    thread::sleep(Duration::from_millis(100));
    
    info!("\n🧪 Demonstrating test mode button injection...\n");
    info!("═══════════════════════════════════════════════════════════");
    
    // 7. Test mode demonstration
    info!("7️⃣  Test mode button injection:");
    info!("   Note: Button injection now uses SysEx MIDI messages");
    info!("   Format: F0 7D 53 53 51 02 <row> <col> <press> F7");
    info!("   See direct_test binary and test1.json/test2.json for examples");
    info!("   ✅ More reliable and precise than file-based injection");
    
    thread::sleep(Duration::from_millis(100));
    
    // 8. Test injection logging
    info!("8️⃣  Direct test injection logging:");
    log_test_injection("demo_test", "Demonstration of test injection logging");
    info!("   ✅ Test injection logged");
    
    info!("\n📊 Demo complete! Check formal_state.log for all logged events.\n");
    
    // Show how to read the log file
    if let Ok(log_content) = fs::read_to_string("formal_state.log") {
        let lines: Vec<&str> = log_content.lines().collect();
        let recent_lines = if lines.len() > 20 {
            &lines[lines.len() - 20..]
        } else {
            &lines[..]
        };
        
        info!("📄 Recent entries from formal_state.log:");
        info!("   (Last {} lines shown)", recent_lines.len());
        info!("   ═══════════════════════════════════════════════════════════");
        for line in recent_lines {
            if line.starts_with('{') {
                // Try to parse and pretty-print JSON
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                        for pretty_line in pretty.lines() {
                            info!("   {}", pretty_line);
                        }
                        info!("   ───────────────────────────────────────────────────────────");
                    }
                }
            } else {
                info!("   {}", line);
            }
        }
    }
    
    info!("\n🎯 Usage Instructions:");
    info!("   • The formal_state.log file contains all system events in JSON format");
    info!("   • Each line is a complete JSON object representing one event");
    info!("   • Events include timestamps, event types, and detailed parameters");
    info!("   • Use SysEx MIDI messages to inject test button presses");
    info!("   • SysEx format: F0 7D 53 53 51 02 <row> <col> <press> F7");
    info!("   • See direct_test binary for practical examples");
    
    info!("\n🔧 Integration with main application:");
    info!("   • The main SimonSaysSeeq application now logs all critical events");
    info!("   • Button presses, LED changes, MIDI events are automatically logged");
    info!("   • SysEx injection allows precise button simulation via MIDI");
    info!("   • This enables comprehensive debugging and automated testing");
    
    Ok(())
}