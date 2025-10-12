//! Direct Test Runner - Tick-synchronized testing

use std::fs;
use std::error::Error;
use std::thread;
use std::time::Duration;


use clap::Parser;

use simon_says_seeq_rust::clock_generator::{ClockGenerator, ClockConfig};
use simon_says_seeq_rust::test_script::DirectTestScript;

#[derive(Parser, Debug)]
#[clap(name = "direct_test")]
#[clap(about = "Run direct tick-synchronized tests")]
struct Args {
    #[arg(long)]
    script: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    println!("🧪 Direct Test Runner");
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
    
    println!("⚠️  IMPORTANT: The main sequencer application must be running!");
    println!("   Start it in another terminal: cargo run --release --bin simon_says_seeq");
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
    
    // Keep running until user stops
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}