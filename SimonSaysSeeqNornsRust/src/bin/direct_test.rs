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
    
    println!("🧪 Test: {}", script.name);
    println!("📋 Description: {}", script.description);
    println!("🎵 BPM: {}", script.bpm);
    println!("📊 Commands: {}", script.commands.len());
    println!("═══════════════════════════════════════════════════════");
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

    // Connect to MIDI
    let connection = generator.connect_midi_output()?;
    println!("✅ MIDI Clock initialized at {:.1} BPM", generator.get_bpm());
    println!();

    // Start the clock thread (needed before sending MIDI messages)
    let _clock_thread = generator.spawn_clock_thread(connection);
    thread::sleep(Duration::from_millis(100)); // Give thread time to initialize
    
    // Initialize sequencer: Stop, Reload Pattern
    println!("🔧 Initializing sequencer...");
    
    // Send MIDI Stop
    println!("  ⏹️  Sending MIDI Stop");
    generator.send_raw_midi(&[0xFC])?;
    thread::sleep(Duration::from_millis(500));
    
    // Send SysEx to reload pattern (resets sequencer state)
    println!("  🔄 Sending SysEx ReloadPattern");
    let reload_sysex = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, 0x01, 0xF7];
    generator.send_raw_midi(&reload_sysex)?;
    thread::sleep(Duration::from_millis(500));
    
    println!();
    println!("🚀 Starting clock and test execution...");
    println!();
    
    // Start the clock
    generator.start()?;
    
    println!("Press Ctrl+C to stop test");
    println!();
    
    // Keep running until user stops
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}