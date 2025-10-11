//! MIDI Clock Generator Utility
//! 
//! A standalone utility program that generates MIDI clock signals and sends them over USB.
//! This can be used to test sequencers and other MIDI devices that need external clock sync.

use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use std::fs;

use clap::Parser;

use simon_says_seeq_rust::test_script::{TestScript, TestCommand};
use simon_says_seeq_rust::clock_generator::{ClockGenerator, ClockConfig, ClockCommand};

/// Display help information
fn show_help() {
    println!("Commands:");
    println!("  s       - Start/stop clock");
    println!("  u       - Increase BPM by 1 (uu=+10, uuu=+50)");
    println!("  d       - Decrease BPM by 1 (dd=-10, ddd=-50)");
    println!("  t       - Toggle test mode (stepped tempo + stop/start: 120->125->121->140->130->122->110)");
    println!("  q       - Quit program");
    println!("  h       - Show this help");
}

/// Handle user input commands
fn handle_command(generator: &ClockGenerator, command: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match command {
        "q" | "quit" | "exit" => {
            println!("Exiting...");
            return Ok(true);
        }
        "s" | "start" => {
            if generator.is_running() {
                generator.stop()?;
            } else {
                generator.start()?;
            }
        }

        "status" => {
            let status = if generator.is_running() {
                "Running"
            } else {
                "Stopped"
            };
            println!("Status: {} | BPM: {:.1}", status, generator.get_bpm());
        }
        "h" | "help" => {
            show_help();
        }
        cmd => {
            // Check for "u" pattern (increase BPM)
            if cmd.chars().all(|c| c == 'u') && !cmd.is_empty() && cmd.len() <= 3 {
                let increment = match cmd.len() {
                    1 => 1.0,    // u = +1
                    2 => 10.0,   // uu = +10
                    3 => 50.0,   // uuu = +50
                    _ => 1.0,
                };
                let new_bpm = generator.get_bpm() + increment;
                generator.set_bpm(new_bpm);
                println!("BPM increased by {:.0} to {:.1}", increment, new_bpm);
            }
            // Check for "d" pattern (decrease BPM)
            else if cmd.chars().all(|c| c == 'd') && !cmd.is_empty() && cmd.len() <= 3 {
                let decrement = match cmd.len() {
                    1 => 1.0,    // d = -1
                    2 => 10.0,   // dd = -10
                    3 => 50.0,   // ddd = -50
                    _ => 1.0,
                };
                let new_bpm = generator.get_bpm() - decrement;
                generator.set_bpm(new_bpm);
                println!("BPM decreased by {:.0} to {:.1}", decrement, new_bpm);
            }
            // Try to parse as BPM value
            else if let Ok(bpm) = cmd.parse::<f32>() {
                generator.set_bpm(bpm);
            } else {
                println!("Unknown command: '{}'. Type 'h' for help.", cmd);
            }
        }
    }

    Ok(false)
}

/// Execute a test script with the clock generator
fn execute_test_script(generator: &mut ClockGenerator, script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 Loading test script: {}", script_path);
    
    // Load the test script
    let content = fs::read_to_string(script_path)?;
    let script: TestScript = serde_json::from_str(&content)?;
    
    println!("📋 Test: {}", script.name);
    println!("📝 Description: {}", script.description);
    println!("═══════════════════════════════════════════════════════════════════");
    
    // Set initial BPM if specified
    if let Some(bpm) = script.initial_bpm {
        generator.set_bpm(bpm);
        println!("🎚️  Initial BPM set to {:.1}", bpm);
    }
    
    // Execute each command in sequence
    for command in script.commands.iter() {
        match command {
            TestCommand::Wait { ms } => {
                println!("⏱️  Waiting {} ms...", ms);
                thread::sleep(Duration::from_millis(*ms));
            }
            
            TestCommand::Start => {
                generator.start()?;
                println!("▶️  MIDI clock started");
            }
            
            TestCommand::Stop => {
                generator.stop()?;
                println!("⏹️  MIDI clock stopped");
            }
            
            TestCommand::SetBpm { bpm } => {
                generator.set_bpm(*bpm);
                println!("🎚️  BPM set to {:.1}", bpm);
            }
            
            TestCommand::LoadPattern { file } => {
                println!("📂 Loading test pattern: {}", file);
                let test_pattern_content = fs::read_to_string(file)?;
                fs::write("current_pattern.json", test_pattern_content)?;
                println!("✅ Pattern loaded - sequencer will use it on next start");
            }
            
            TestCommand::ButtonPress { file, grid_id, x, y } => {
                println!("🔘 Button press: {} at ({}, {}) via {}", grid_id, x, y, file);
                // Note: Button injection not supported in MIDI clock generator
                println!("⚠️  Button injection only works with automated_test_clock");
            }
            
            TestCommand::ButtonRelease { file, grid_id, x, y } => {
                println!("🔘 Button release: {} at ({}, {}) via {}", grid_id, x, y, file);
                println!("⚠️  Button injection only works with automated_test_clock");
            }
            
            TestCommand::SimpleButton { file, x, y } => {
                println!("🔘 Simple button: ({}, {}) via {}", x, y, file);
                println!("⚠️  Button injection only works with automated_test_clock");
            }
            
            TestCommand::ArmAction { arm_column, target_row, target_column } => {
                println!("🎯 ARM action: arm={}, target=({},{})", arm_column, target_row, target_column);
                println!("⚠️  Button injection only works with automated_test_clock");
            }
            
            TestCommand::WaitSteps { count } => {
                println!("⏳ Waiting for {} steps...", count);
                // Calculate approximate time based on BPM
                let bpm = generator.get_bpm();
                let ms_per_step = (60_000.0 / bpm) / 4.0; // 4 steps per beat at 24 PPQ
                let total_ms = ms_per_step * (*count as f32);
                thread::sleep(Duration::from_millis(total_ms as u64));
            }
            
            TestCommand::LogMilestone { message } => {
                println!("🏁 Milestone: {}", message);
            }
            
            TestCommand::VerifyState { description } => {
                println!("🔍 Verify: {} (placeholder - not implemented)", description);
            }
            
            TestCommand::ReloadPattern => {
                println!("🔄 Sending SysEx reload pattern command...");
                let sysex_message = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, 0x01, 0xF7];
                if let Err(e) = generator.send_raw_midi(&sysex_message) {
                    eprintln!("❌ Error sending SysEx: {}", e);
                } else {
                    println!("✅ Reload pattern command sent");
                }
            }
            
            TestCommand::SysExButton { row, col, press } => {
                let action = if *press { "Press" } else { "Release" };
                println!("🔘 Sending SysEx button {}: row={}, col={}", action, row, col);
                let press_byte = if *press { 0x01 } else { 0x00 };
                let sysex_message = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, 0x02, *row, *col, press_byte, 0xF7];
                if let Err(e) = generator.send_raw_midi(&sysex_message) {
                    eprintln!("❌ Error sending SysEx: {}", e);
                } else {
                    println!("✅ Button {} command sent", action.to_lowercase());
                }
            }
            
            TestCommand::RecordRowConfig { row, max_step } => {
                println!("📝 RecordRowConfig: row={}, max_step={} (not applicable to midi_clock_generator)", row, max_step);
            }
        }
    }
    
    println!("\n✅ Test script completed successfully!");
    println!("Press Ctrl+C to exit");
    
    // Keep the clock running until user exits
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(name = "MIDI Clock Generator")]
#[command(about = "Generate MIDI clock signals for testing sequencers", long_about = None)]
struct Args {
    /// Optional test script file to execute
    #[arg(short, long)]
    script: Option<String>,
    
    /// Initial BPM (default: 120.0)
    #[arg(short, long, default_value = "120.0")]
    bpm: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("MIDI Clock Generator");
    println!("======================");

    // Start with BPM from args and enable test mode
    let config = ClockConfig {
        enable_tick_counting: false,
        enable_test_mode: true,
    };
    let mut generator = ClockGenerator::new_with_config(args.bpm, config);
    
    // Connect to MIDI output
    let connection = generator.connect_midi_output()?;
    
    println!("MIDI Clock Generator initialized at {:.1} BPM", generator.get_bpm());
    
    // Check if we should execute a test script
    if let Some(script_path) = args.script {
        return execute_test_script(&mut generator, &script_path);
    }
    
    show_help();
    println!();

    // Handle Ctrl+C gracefully
    let generator_for_signal = generator.clone();
    ctrlc::set_handler(move || {
        println!("\n⚡ Received Ctrl+C, stopping...");
        generator_for_signal.exit();
    })?;

    // Start the clock generation thread
    let clock_thread = generator.spawn_clock_thread(connection);
    
    // Auto-start the clock (must be after spawn_clock_thread so command_sender exists)
    println!("Auto-starting clock");
    generator.start()?;

    println!("Enter commands: s(start/stop), u/uu/uuu(+1/+10/+50 BPM), d/dd/ddd(-1/-10/-50 BPM), t(test), q(quit), h(help)");
    
    // Simple line-based input loop
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        match input {
            "s" | "S" => {
                if generator.is_running() {
                    generator.stop()?;
                    println!("Stopped");
                } else {
                    generator.start()?;
                    println!("Started");
                }
            }

            "t" | "T" => {
                generator.toggle_test_mode();
            }
            "q" | "Q" => {
                println!("Exiting...");
                break;
            }
            "h" | "H" => {
                show_help();
            }
            _ => {
                // Check for "u" pattern (increase BPM)
                if input.chars().all(|c| c == 'u') && !input.is_empty() && input.len() <= 3 {
                    let increment = match input.len() {
                        1 => 1.0,    // u = +1
                        2 => 10.0,   // uu = +10
                        3 => 50.0,   // uuu = +50
                        _ => 1.0,
                    };
                    let new_bpm = generator.get_bpm() + increment;
                    generator.set_bpm(new_bpm);
                    println!("BPM: {:.1} (+{})", new_bpm, increment);
                }
                // Check for "d" pattern (decrease BPM)
                else if input.chars().all(|c| c == 'd') && !input.is_empty() && input.len() <= 3 {
                    let decrement = match input.len() {
                        1 => 1.0,    // d = -1
                        2 => 10.0,   // dd = -10
                        3 => 50.0,   // ddd = -50
                        _ => 1.0,
                    };
                    let new_bpm = generator.get_bpm() - decrement;
                    generator.set_bpm(new_bpm);
                    println!("BPM: {:.1} (-{})", new_bpm, decrement);
                }
                // Try to parse as BPM value
                else if let Ok(bpm) = input.parse::<f32>() {
                    generator.set_bpm(bpm);
                    println!("BPM: {:.1}", generator.get_bpm());
                }
                else if !input.trim().is_empty() {
                    println!("Unknown command. Type 'h' for help.");
                }
            }
        }
    }

    // Clean shutdown
    generator.exit();

    // Wait for clock thread to finish
    println!("Waiting for clock thread to finish...");
    clock_thread.join().unwrap();

    println!("MIDI Clock Generator stopped.");
    Ok(())
}

