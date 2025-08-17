//! Simplified test to show final results of the tick-based MIDI tempo detection algorithm

use log::{info, warn, LevelFilter};
use std::time::{Duration, Instant};
use std::thread;

// Import our MIDI types
use simon_says_seeq_rust::midi::TickWindow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with minimal output
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Warn)
        .init();

    println!("MIDI Tempo Detection Algorithm Test Results");
    println!("==========================================");
    println!("Strategy: 4-second overlapping windows, new window every 5 ticks");
    println!("Formula: BPM = ticks_in_4_seconds * 0.625");
    println!();

    // Test different tempos
    let test_tempos = vec![60.0, 90.0, 120.0, 140.0, 160.0, 180.0];
    let mut results = Vec::new();
    
    for tempo in test_tempos {
        let result = test_tempo_detection(tempo)?;
        results.push((tempo, result));
        thread::sleep(Duration::from_millis(100)); // Brief pause between tests
    }

    // Print summary table
    println!("\n📊 ALGORITHM PERFORMANCE SUMMARY");
    println!("┌─────────────┬─────────────┬─────────────┬─────────────┬─────────────┐");
    println!("│ Target BPM  │ Detected    │ Error (BPM) │ Error (%)   │ Status      │");
    println!("├─────────────┼─────────────┼─────────────┼─────────────┼─────────────┤");
    
    let mut total_error = 0.0;
    let mut pass_count = 0;
    
    for (target, detected) in &results {
        let error_bpm = (detected - target).abs();
        let error_percent = (error_bpm / target) * 100.0;
        let status = if error_percent < 2.0 { "✅ PASS" } else { "❌ FAIL" };
        
        if error_percent < 2.0 {
            pass_count += 1;
        }
        total_error += error_percent;
        
        println!("│ {:>9.1}   │ {:>9.1}   │ {:>9.1}   │ {:>8.1}%   │ {}    │", 
                 target, detected, error_bpm, error_percent, status);
    }
    
    println!("└─────────────┴─────────────┴─────────────┴─────────────┴─────────────┘");
    
    let avg_error = total_error / results.len() as f32;
    let pass_rate = (pass_count as f32 / results.len() as f32) * 100.0;
    
    println!("\n📈 ALGORITHM STATISTICS:");
    println!("• Tests passed: {}/{} ({:.1}%)", pass_count, results.len(), pass_rate);
    println!("• Average error: {:.2}%", avg_error);
    println!("• Max acceptable error: 2.0%");
    
    if pass_rate >= 80.0 && avg_error < 3.0 {
        println!("🎉 OVERALL: Algorithm performs well!");
    } else {
        println!("⚠️  OVERALL: Algorithm needs improvement");
    }
    
    println!("\n🔬 TECHNICAL DETAILS:");
    println!("• Window duration: 4.0 seconds");
    println!("• New window frequency: Every 5 ticks");
    println!("• Active windows: ~20-40 (depending on tempo)");
    println!("• Calculation windows: 1-3 (aged 4.0-4.1 seconds)");
    println!("• Update frequency: ~100ms");

    Ok(())
}

fn test_tempo_detection(target_bpm: f32) -> Result<f32, Box<dyn std::error::Error>> {
    // Calculate tick interval for the target BPM
    let ticks_per_second = (target_bpm / 60.0) * 24.0;
    let tick_interval = Duration::from_secs_f32(1.0 / ticks_per_second);
    
    print!("Testing {:>5.1} BPM... ", target_bpm);
    
    // Simulate the tick windows
    let mut tick_windows: Vec<TickWindow> = Vec::new();
    let mut clock_ticks = 0u32;
    let mut final_tempo: Option<f32> = None;
    
    let start_time = Instant::now();
    let test_duration = Duration::from_secs(5); // Test for 5 seconds
    
    // Simulate MIDI clock ticks
    while start_time.elapsed() < test_duration {
        let now = Instant::now();
        clock_ticks += 1;
        
        // Start a new window every 5 ticks for smooth averaging
        if tick_windows.is_empty() || clock_ticks % 5 == 0 {
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
                let bpm = window.tick_count as f32 * 0.625;
                if bpm >= 20.0 && bpm <= 300.0 {
                    tempo_readings.push(bpm);
                }
            }
        }
        
        // Average the tempo readings
        if !tempo_readings.is_empty() {
            let average_bpm = tempo_readings.iter().sum::<f32>() / tempo_readings.len() as f32;
            final_tempo = Some(average_bpm);
        }
        
        // Wait for the next tick
        thread::sleep(tick_interval);
    }
    
    match final_tempo {
        Some(detected) => {
            let error = (detected - target_bpm).abs();
            let error_percent = (error / target_bpm) * 100.0;
            println!("detected {:>5.1} BPM (error: {:>4.1}%, {:>4.1} BPM)", 
                     detected, error_percent, error);
            Ok(detected)
        },
        None => {
            println!("❌ NO DETECTION");
            Err("No tempo detected".into())
        }
    }
}