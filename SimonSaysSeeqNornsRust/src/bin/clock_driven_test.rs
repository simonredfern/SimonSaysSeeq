//! Direct Test Runner - Tick-synchronized testing

use std::fs;
use std::error::Error;
use std::thread;
use std::time::Duration;


use clap::Parser;

use simon_says_seeq_rust::clock_generator::{ClockGenerator, ClockConfig};
use simon_says_seeq_rust::test_script::DirectTestScript;

#[derive(Parser, Debug)]
#[clap(name = "clock_driven_test")]
#[clap(about = "Run clock-driven tick-synchronized tests")]
struct Args {
    #[arg(long)]
    script: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("🧪 Clock-Driven Test Runner");
    println!("═══════════════════════════════════════════════════════");
    println!();

    // Load test script
    println!("📜 Loading test script: {}", args.script);
    let script_content = fs::read_to_string(&args.script)?;
    let script: DirectTestScript = serde_json::from_str(&script_content)?;
    
    let test_name = script.name.clone();
    let script_file = args.script.clone();
    
    println!("🧪 Test: {}", test_name);
    println!("📋 Description: {}", script.description);
    println!("🎵 BPM: {}", script.bpm);
    println!("📊 Commands: {}", script.commands.len());
    println!("═══════════════════════════════════════════════════════");
    println!();
    
    println!("⚠️  IMPORTANT: The main sequencer application must be running in test mode!");
    println!("   Start it in another terminal: cargo run --release --bin simon_says_seeq -- --test-mode");
    println!("   The sequencer receives MIDI clock/SysEx and writes formal_state.log");
    println!();

    // Truncate formal_state.log
    fs::write("formal_state.log", "")?;
    println!("🗑️  Truncated formal_state.log for clean test run");
    println!();

    // Create clock generator
    let config = ClockConfig {
        enable_tick_counting: true,
        enable_test_mode: false,
    };
    let mut generator = ClockGenerator::new_with_config(script.bpm, config);
    
    // Calculate max tick from script for auto-exit
    let max_tick = script.commands.iter().map(|cmd| cmd.at_tick).max().unwrap_or(0);
    let grace_ticks = 12; // Wait 2 extra steps after last command
    let exit_tick = max_tick + grace_ticks;
    
    generator.set_direct_test_script(script);

    // Connect to MIDI output (for clock)
    let connection = generator.connect_midi_output()?;
    println!("✅ MIDI Clock initialized at {:.1} BPM", generator.get_bpm());
    println!();

    // Connect to MIDI input (for listening to notes from sequencer)
    println!("🎵 Connecting MIDI input to listen for sequencer notes...");
    let _midi_input = generator.connect_midi_input()?;
    println!("✅ MIDI Input connected");
    println!();

    // Start the clock thread (needed before sending MIDI messages)
    let _clock_thread = generator.spawn_clock_thread(connection);
    thread::sleep(Duration::from_millis(100)); // Give thread time to initialize
    
    // Initialize sequencer: Verify Test Mode, Stop
    println!("🔧 Initializing sequencer...");
    
    // Verify sequencer is in test mode FIRST
    println!("  🧪 Querying sequencer test mode...");
    let test_mode_query = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, 0x10, 0xF7];
    generator.send_raw_midi(&test_mode_query)?;
    thread::sleep(Duration::from_millis(500)); // Wait for response
    
    // Check if we received test mode confirmation (0x11) or rejection (0x12)
    // The response will be in the MIDI input buffer
    let test_mode_verified = generator.check_test_mode_response();
    if !test_mode_verified {
        println!();
        println!("❌ TEST FAILED: Sequencer is NOT in test mode!");
        println!("   Please restart the sequencer with: cargo run --release --bin simon_says_seeq -- --test-mode");
        println!();
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Sequencer not in test mode"
        )));
    }
    println!("  ✅ Sequencer is in test mode");
    
    // Send MIDI Stop (in test mode, this will NOT auto-save)
    println!("  ⏹️  Sending MIDI Stop");
    generator.send_raw_midi(&[0xFC])?;
    thread::sleep(Duration::from_millis(500));
    
    // In test mode, the sequencer already loaded test_pattern_1.json at startup
    // No need to copy or reload - just proceed with test
    println!("  ✅ Test pattern already loaded (test mode auto-loads test_pattern_1.json)");
    
    println!();
    println!("🚀 Starting clock and test execution...");
    println!();
    
    // Start the clock
    generator.start()?;
    
    println!("Press Ctrl+C to stop test");
    println!();
    println!("═══════════════════════════════════════════════════════");
    println!("🧪 Running Test: {}", test_name);
    println!("📄 Script File: {}", script_file);
    println!("═══════════════════════════════════════════════════════");
    println!();
    
    // Monitor test progress and wait for completion
    let mut last_tick = 0;
    let mut no_progress_count = 0;
    
    loop {
        thread::sleep(Duration::from_millis(500));
        
        let current_tick = generator.get_tick_count();
        
        // Check if we've reached the exit tick
        if current_tick >= exit_tick {
            println!();
            println!("═══════════════════════════════════════════════════════");
            println!("📊 Test Complete - Reached tick {} (exit at {})", current_tick, exit_tick);
            break;
        }
        
        // Detect if test is stalled (no progress for 10 seconds)
        if current_tick == last_tick {
            no_progress_count += 1;
            if no_progress_count >= 20 { // 20 * 500ms = 10 seconds
                println!();
                println!("⚠️  Warning: No progress detected for 10 seconds at tick {}", current_tick);
                println!("   Test may have stalled. Exiting...");
                break;
            }
        } else {
            no_progress_count = 0;
            last_tick = current_tick;
        }
    }
    
    // Get test statistics
    let (passed, failed) = generator.get_test_stats();
    let total = passed + failed;
    
    println!();
    println!("═══════════════════════════════════════════════════════");
    println!("📊 Test Results for: {}", test_name);
    println!("═══════════════════════════════════════════════════════");
    println!("Total Verifications: {}", total);
    println!("✅ Passed: {}", passed);
    println!("❌ Failed: {}", failed);
    
    if failed == 0 && total > 0 {
        println!();
        println!("🎉 ALL TESTS PASSED!");
        println!("═══════════════════════════════════════════════════════");
        Ok(())
    } else if total == 0 {
        println!();
        println!("⚠️  No verifications were run");
        println!("═══════════════════════════════════════════════════════");
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No verifications run"
        )))
    } else {
        println!();
        println!("❌ TEST FAILED - {} verification(s) failed", failed);
        println!("═══════════════════════════════════════════════════════");
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{} test(s) failed", failed)
        )))
    }
}