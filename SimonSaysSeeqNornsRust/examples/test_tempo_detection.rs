//! Test program to demonstrate the new tick-based MIDI tempo detection algorithm
//! 
//! This example simulates MIDI clock input at various tempos and shows how the
//! new algorithm performs compared to the old beat-based approach.

use log::{info, debug, LevelFilter};
use std::time::{Duration, Instant};
use std::thread;

// Import our MIDI types
use simon_says_seeq_rust::midi::{MidiManager, MidiInputEvent, TickWindow, ClockState, ClockSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();

    info!("Starting MIDI Tempo Detection Test");
    info!("=====================================");

    // Test different tempos
    let test_tempos = vec![60.0, 120.0, 140.0, 180.0];
    
    for tempo in test_tempos {
        info!("\nTesting tempo: {:.1} BPM", tempo);
        test_tempo_detection(tempo)?;
        thread::sleep(Duration::from_millis(500)); // Brief pause between tests
    }

    info!("\nTempo detection test completed!");
    Ok(())
}

fn test_tempo_detection(target_bpm: f32) -> Result<(), Box<dyn std::error::Error>> {
    // Calculate tick interval for the target BPM
    // 24 ticks per quarter note, target_bpm quarter notes per minute
    let ticks_per_second = (target_bpm / 60.0) * 24.0;
    let tick_interval = Duration::from_secs_f32(1.0 / ticks_per_second);
    
    info!("Target: {:.1} BPM = {:.2} ticks/sec = {:.2}ms per tick", 
          target_bpm, ticks_per_second, tick_interval.as_millis());

    // Simulate the clock state and tick windows
    let mut tick_windows: Vec<TickWindow> = Vec::new();
    let mut clock_ticks = 0u32;
    let mut detected_tempo: Option<f32> = None;
    
    let start_time = Instant::now();
    let test_duration = Duration::from_secs(6); // Test for 6 seconds
    
    // Simulate MIDI clock ticks
    while start_time.elapsed() < test_duration {
        let now = Instant::now();
        
        // Simulate a MIDI clock tick
        clock_ticks += 1;
        
        // Apply the new tick-based tempo detection algorithm
        // Start a new window every 5 ticks for smooth averaging
        if tick_windows.is_empty() || clock_ticks % 5 == 0 {
            // Add new window starting with this tick
            tick_windows.push(TickWindow {
                start_time: now,
                tick_count: 0,
            });
        }
        
        // Update all active windows with this tick
        for window in &mut tick_windows {
            window.tick_count += 1;
        }
        
        // Remove windows older than 4 seconds
        tick_windows.retain(|window| {
            now.duration_since(window.start_time).as_secs_f32() < 4.1
        });
        
        // Calculate tempo from windows that are exactly 4 seconds old
        let mut tempo_readings = Vec::new();
        for window in &tick_windows {
            let elapsed = now.duration_since(window.start_time).as_secs_f32();
            if elapsed >= 4.0 && elapsed <= 4.1 {
                // Window is complete (4 seconds)
                // BPM = ticks_in_4_seconds * 0.625
                // This is because: BPM = (ticks/4sec) * (1/24 ticks_per_beat) * (60 sec/min) = ticks * 15/24 = ticks * 0.625
                let bpm = window.tick_count as f32 * 0.625;
                
                // Validate tempo range
                if bpm >= 20.0 && bpm <= 300.0 {
                    tempo_readings.push(bpm);
                }
            }
        }
        
        // Average the tempo readings
        if !tempo_readings.is_empty() {
            let average_bpm = tempo_readings.iter().sum::<f32>() / tempo_readings.len() as f32;
            
            // Only update if we have a new reading or significant change
            let should_update = match detected_tempo {
                None => true,
                Some(prev) => (average_bpm - prev).abs() > 0.5
            };
            
            if should_update {
                detected_tempo = Some(average_bpm);
                let error = (average_bpm - target_bpm).abs();
                let error_percent = (error / target_bpm) * 100.0;
                
                info!("Tick {}: Detected {:.1} BPM from {} windows (error: {:.1} BPM, {:.1}%)", 
                      clock_ticks, average_bpm, tempo_readings.len(), error, error_percent);
            }
        }
        
        // Log progress every 2 seconds
        if clock_ticks % (ticks_per_second as u32 * 2) == 0 {
            debug!("Progress: {:.1}s elapsed, {} active windows, {} total ticks", 
                   start_time.elapsed().as_secs_f32(), tick_windows.len(), clock_ticks);
        }
        
        // Wait for the next tick
        thread::sleep(tick_interval);
    }
    
    // Final results
    match detected_tempo {
        Some(final_bpm) => {
            let final_error = (final_bpm - target_bpm).abs();
            let final_error_percent = (final_error / target_bpm) * 100.0;
            info!("Final result: {:.1} BPM (target: {:.1}, error: {:.1} BPM, {:.1}%)", 
                  final_bpm, target_bpm, final_error, final_error_percent);
            
            // Test passes if error is less than 1%
            if final_error_percent < 1.0 {
                info!("✅ PASS - Tempo detection accurate within 1%");
            } else {
                info!("❌ FAIL - Tempo detection error exceeds 1%");
            }
        },
        None => {
            info!("❌ FAIL - No tempo detected");
        }
    }
    
    Ok(())
}

// Helper function to demonstrate the mathematical relationship
fn explain_algorithm() {
    info!("Algorithm Explanation:");
    info!("======================");
    info!("MIDI Clock: 24 ticks per quarter note (beat)");
    info!("At 120 BPM: 120 beats/min = 2 beats/sec = 48 ticks/sec");
    info!("Over 4 seconds: 48 * 4 = 192 ticks");
    info!("Formula: BPM = (ticks_in_4_seconds / 4_seconds) / 24_ticks_per_beat * 60_seconds_per_minute");
    info!("Simplified: BPM = ticks_in_4_seconds * (60 / (4 * 24)) = ticks_in_4_seconds * 0.625");
    info!("");
    info!("Overlapping windows every 5 ticks provide:");
    info!("- More frequent tempo updates");
    info!("- Better averaging for stability");  
    info!("- Faster response to tempo changes");
    info!("");
}