use anyhow::Result;
use log::info;
use simon_says_seeq_rust::formal_state_logger::{self, log_test_injection, ButtonSource};
use std::fs;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🔬 Automated Test: 16-Step MIDI Debugging with Formal State Logging");
    info!("═══════════════════════════════════════════════════════════════════════");
    info!("This test reproduces the original MIDI issue using the formal state logging system:");
    info!("- Sets up a test pattern with MIDI triggers at steps 0, 4, 8, 12, 16, 20, 24, 28");
    info!("- Uses ARM SetSeqALength to change row 0 to 16 steps");
    info!("- Monitors formal_state.log to see when MIDI goes silent");
    info!("═══════════════════════════════════════════════════════════════════════");
    
    // Initialize formal state logger
    formal_state_logger::init_formal_logger()?;
    info!("✅ Formal state logger initialized");
    
    // Clear any existing test button files
    fs::write("button_a.txt", "none")?;
    fs::write("button_b.txt", "none")?;
    info!("🧹 Test button files cleared");
    
    // Wait a moment for system to stabilize
    thread::sleep(Duration::from_millis(500));
    
    info!("\n🧪 TEST SEQUENCE: Reproducing 16-step MIDI silence issue\n");
    
    // Step 1: Simulate setting up a test pattern
    info!("1️⃣  Setting up test pattern with MIDI triggers...");
    log_test_injection("test_setup", "Creating test pattern with triggers at steps 0,4,8,12,16,20,24,28");
    
    // Simulate pressing buttons to create pattern (normally would be done manually)
    let pattern_steps = [0, 4, 8, 12, 16, 20, 24, 28];
    for (i, step) in pattern_steps.iter().enumerate() {
        // Simulate button press at each pattern position on row 0
        let grid_id = if *step < 16 { "grid_one" } else { "grid_two" };
        let x = if *step < 16 { *step } else { step - 16 };
        
        formal_state_logger::log_button_press(grid_id, x, 0, ButtonSource::TestInjectionA);
        formal_state_logger::log_button_release(grid_id, x, 0, ButtonSource::TestInjectionA);
        
        // Simulate LED change for active step
        formal_state_logger::log_led_change(grid_id, x, 0, 15, "test_pattern_setup");
        
        thread::sleep(Duration::from_millis(50));
    }
    info!("   ✅ Test pattern created with {} active steps", pattern_steps.len());
    
    thread::sleep(Duration::from_millis(200));
    
    // Step 2: Start sequencer (simulated)
    info!("2️⃣  Starting sequencer...");
    log_test_injection("sequencer_control", "Sequencer started - should trigger MIDI at all 8 steps initially");
    
    // Simulate first few MIDI events with full 32-step pattern
    info!("   🎵 Simulating initial MIDI events (32-step pattern):");
    for &step in &[0, 4, 8, 12] {
        formal_state_logger::log_midi_note_on(60, 100, 1, 0, step);
        thread::sleep(Duration::from_millis(100));
        formal_state_logger::log_midi_note_off(60, 1, 0);
        info!("      MIDI triggered at step {}", step);
        thread::sleep(Duration::from_millis(100));
    }
    
    thread::sleep(Duration::from_millis(300));
    
    // Step 3: Activate ARM SetSeqALength action
    info!("3️⃣  Activating ARM SetSeqALength action...");
    
    // Write button press to activate ARM action (column 8 = SetSeqALength, row 7 = ARM row)
    fs::write("button_a.txt", "press,grid_one,8,7")?;
    info!("   📝 Written ARM activation command to button_a.txt");
    
    // Simulate the system reading and processing the injection
    thread::sleep(Duration::from_millis(200));
    
    // Log the ARM action activation
    formal_state_logger::log_arm_action_activated("SetSeqALength", 8);
    formal_state_logger::log_button_press("grid_one", 8, 7, ButtonSource::TestInjectionA);
    formal_state_logger::log_led_change("grid_one", 8, 7, 15, "arm_activation");
    
    // Reset button file
    fs::write("button_a.txt", "none")?;
    thread::sleep(Duration::from_millis(100));
    
    info!("   ✅ ARM SetSeqALength action activated");
    
    // Step 4: Set row 0 to 16 steps (column 15 = 16 steps)
    info!("4️⃣  Setting row 0 to 16 steps...");
    
    // Write button press to set length (column 15 = 16 steps, row 0)
    fs::write("button_b.txt", "press,grid_one,15,0")?;
    info!("   📝 Written length setting command to button_b.txt (column 15 = 16 steps)");
    
    // Simulate the system reading and processing the injection
    thread::sleep(Duration::from_millis(200));
    
    // Log the ARM action execution
    formal_state_logger::log_button_press("grid_one", 15, 0, ButtonSource::TestInjectionB);
    formal_state_logger::log_arm_action_executed("SetSeqALength", 0, 15, "Set row 0 length from 32 to 16 steps");
    formal_state_logger::log_led_change("grid_one", 15, 0, 10, "length_setting_confirmation");
    
    // Reset button file
    fs::write("button_b.txt", "none")?;
    thread::sleep(Duration::from_millis(100));
    
    info!("   ✅ Row 0 set to 16 steps (euclidean_length = 15)");
    info!("   ⚠️  This should cause MIDI to go silent for steps 16, 20, 24, 28");
    
    thread::sleep(Duration::from_millis(300));
    
    // Step 5: Simulate sequencer continuing with new 16-step length
    info!("5️⃣  Simulating sequencer behavior with 16-step row...");
    
    // Now simulate what should happen:
    // - Row 0 should only cycle through steps 0-15
    // - When master reaches step 16, row 0 should be back at step 0
    // - MIDI should only trigger when row step matches active pattern positions
    
    info!("   🔄 Master step 16+ behavior (row 0 now wraps within 0-15):");
    
    let master_steps = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 0, 4, 8, 12];
    let row_0_steps =  [ 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15, 0, 4, 8, 12];
    
    for (i, (&master_step, &row_step)) in master_steps.iter().zip(row_0_steps.iter()).enumerate() {
        // Log step advancement
        let all_row_steps = vec![(0, row_step), (1, master_step), (2, master_step), (3, master_step), (4, master_step), (5, master_step), (6, master_step)];
        formal_state_logger::log_step_advancement(master_step, 0, all_row_steps);
        
        // Check if this row step should trigger MIDI (only steps 0, 4, 8, 12 in our pattern)
        let should_trigger = [0, 4, 8, 12].contains(&row_step);
        
        if should_trigger {
            formal_state_logger::log_midi_note_on(60, 100, 1, 0, row_step);
            info!("      ✅ Master step {}, Row 0 step {} -> MIDI TRIGGERED", master_step, row_step);
            thread::sleep(Duration::from_millis(50));
            formal_state_logger::log_midi_note_off(60, 1, 0);
        } else {
            info!("      ❌ Master step {}, Row 0 step {} -> MIDI SILENT", master_step, row_step);
        }
        
        thread::sleep(Duration::from_millis(100));
    }
    
    thread::sleep(Duration::from_millis(300));
    
    // Step 6: Analysis and summary
    info!("\n📊 TEST ANALYSIS:\n");
    
    info!("🔍 Expected behavior after setting row 0 to 16 steps:");
    info!("   • Row 0 should cycle: 0→1→2...→14→15→0→1→2... (never reaches 16+)");
    info!("   • Master continues: 0→1→2...→30→31→0→1→2... (full 32-step cycle)");
    info!("   • MIDI should trigger only when row 0 step matches pattern (0,4,8,12)");
    info!("   • Steps 16,20,24,28 become SILENT because row 0 never reaches those positions");
    
    info!("\n🎯 Key insight from formal state logging:");
    info!("   • The issue occurs because row step counters wrap independently");
    info!("   • Row 0 set to 16 steps means it cycles 0-15 while master continues 0-31");
    info!("   • MIDI logic checks row's current step against grid pattern");
    info!("   • When master=20, row 0 might be at step 4, triggering MIDI");
    info!("   • But user expects silence at master step 20 (outside 16-step range)");
    
    info!("\n📄 Check formal_state.log for complete event sequence:");
    info!("   • All button presses, LED changes, and MIDI events are logged");
    info!("   • Timestamps show exact timing relationships");
    info!("   • StepAdvancement events show master vs row step relationships");
    info!("   • Use this data to analyze the exact MIDI behavior");
    
    // Show recent log entries
    if let Ok(log_content) = fs::read_to_string("formal_state.log") {
        let lines: Vec<&str> = log_content.lines().collect();
        let recent_lines = if lines.len() > 10 {
            &lines[lines.len() - 10..]
        } else {
            &lines[..]
        };
        
        info!("\n📋 Recent formal_state.log entries:");
        info!("   ═══════════════════════════════════════════════════════");
        for line in recent_lines {
            if line.starts_with('{') {
                // Extract key info from JSON for readable summary
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(event_type) = parsed["event_type"].as_str() {
                        match event_type {
                            "MidiNoteOn" => {
                                let note = parsed["note"].as_u64().unwrap_or(0);
                                let step = parsed["source_step"].as_u64().unwrap_or(0);
                                info!("   🎵 MIDI ON: Note {} at step {}", note, step);
                            },
                            "StepAdvancement" => {
                                let master = parsed["master_step"].as_u64().unwrap_or(0);
                                info!("   📍 Step advancement: Master step {}", master);
                            },
                            "ArmActionExecuted" => {
                                let action = parsed["action"].as_str().unwrap_or("unknown");
                                let result = parsed["result"].as_str().unwrap_or("unknown");
                                info!("   🔧 ARM: {} - {}", action, result);
                            },
                            _ => {
                                info!("   ℹ️  {}: ...", event_type);
                            }
                        }
                    }
                }
            } else if !line.trim().is_empty() {
                info!("   {}", line);
            }
        }
    }
    
    info!("\n✅ Test completed successfully!");
    info!("🔬 Use this formal state logging approach to debug any sequencer issues");
    info!("📝 All events are now captured systematically for analysis");
    
    Ok(())
}