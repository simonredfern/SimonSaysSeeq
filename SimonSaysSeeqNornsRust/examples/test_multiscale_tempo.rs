//! Test the new multi-scale MIDI tempo detection algorithm
//! Uses windows of 1, 2, 4, 8, 16, 32 seconds and averages them

use log::LevelFilter;
use std::time::{Duration, Instant};
use std::thread;

// Import our MIDI types
use simon_says_seeq_rust::midi::MultiScaleTickWindows;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with minimal output
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Warn)
        .init();

    println!("🎵 MULTI-SCALE MIDI TEMPO DETECTION TEST");
    println!("========================================");
    println!("Strategy: Multiple windows (1,2,4,8,16,32 seconds) averaged together");
    println!("Formula: For N-second window: BPM = ticks * (2.5 / N)");
    println!();

    // Test different tempos
    let test_tempos = vec![60.0, 75.0, 90.0, 120.0, 140.0, 160.0, 180.0, 200.0];
    let mut results = Vec::new();
    
    for tempo in test_tempos {
        let result = test_tempo_detection(tempo)?;
        results.push((tempo, result));
        thread::sleep(Duration::from_millis(100));
    }

    // Print detailed results table
    println!("\n📊 MULTI-SCALE ALGORITHM PERFORMANCE");
    println!("┌─────────────┬─────────────┬─────────────┬─────────────┬─────────────┬─────────────┐");
    println!("│ Target BPM  │ Detected    │ Error (BPM) │ Error (%)   │ Windows     │ Status      │");
    println!("├─────────────┼─────────────┼─────────────┼─────────────┼─────────────┼─────────────┤");
    
    let mut total_error = 0.0;
    let mut pass_count = 0;
    
    for (target, (detected, window_count)) in &results {
        let error_bpm = (detected - target).abs();
        let error_percent = (error_bpm / target) * 100.0;
        let status = if error_percent < 1.5 { "✅ EXCELLENT" } else if error_percent < 3.0 { "✅ GOOD" } else { "❌ NEEDS WORK" };
        
        if error_percent < 3.0 {
            pass_count += 1;
        }
        total_error += error_percent;
        
        println!("│ {:>9.1}   │ {:>9.1}   │ {:>9.1}   │ {:>8.1}%   │ {:>9}   │ {}  │", 
                 target, detected, error_bpm, error_percent, window_count, status);
    }
    
    println!("└─────────────┴─────────────┴─────────────┴─────────────┴─────────────┴─────────────┘");
    
    let avg_error = total_error / results.len() as f32;
    let pass_rate = (pass_count as f32 / results.len() as f32) * 100.0;
    
    println!("\n📈 ALGORITHM PERFORMANCE METRICS:");
    println!("• Tests passed: {}/{} ({:.1}%)", pass_count, results.len(), pass_rate);
    println!("• Average error: {:.2}%", avg_error);
    println!("• Excellent threshold: <1.5% error");
    println!("• Good threshold: <3.0% error");
    
    if pass_rate >= 90.0 && avg_error < 2.0 {
        println!("🏆 OUTSTANDING: Multi-scale algorithm excels!");
    } else if pass_rate >= 75.0 && avg_error < 3.0 {
        println!("🎉 EXCELLENT: Multi-scale algorithm performs very well!");
    } else {
        println!("⚠️  NEEDS IMPROVEMENT: Algorithm requires tuning");
    }
    
    println!("\n🔬 TECHNICAL ANALYSIS:");
    println!("• Window sizes: 1s, 2s, 4s, 8s, 16s, 32s");
    println!("• Responsiveness: 1-2s windows provide fast updates");
    println!("• Accuracy: 4-8s windows provide good precision");
    println!("• Stability: 16-32s windows provide maximum stability");
    println!("• Averaging: All mature windows averaged for final reading");
    
    // Show window maturation timeline
    println!("\n⏱️  WINDOW MATURATION TIMELINE:");
    println!("• After  1s: 1 window available (fast but jittery)");
    println!("• After  2s: 2 windows available (faster convergence)");
    println!("• After  4s: 3 windows available (good accuracy)");
    println!("• After  8s: 4 windows available (very stable)");
    println!("• After 16s: 5 windows available (excellent stability)");
    println!("• After 32s: 6 windows available (maximum accuracy)");

    Ok(())
}

fn test_tempo_detection(target_bpm: f32) -> Result<(f32, usize), Box<dyn std::error::Error>> {
    // Calculate tick interval for the target BPM
    let ticks_per_second = (target_bpm / 60.0) * 24.0;
    let tick_interval = Duration::from_secs_f32(1.0 / ticks_per_second);
    
    print!("Testing {:>6.1} BPM... ", target_bpm);
    
    // Create multi-scale window
    let mut window = MultiScaleTickWindows::new();
    let mut final_tempo: Option<f32> = None;
    let mut final_window_count = 0;
    
    let start_time = Instant::now();
    let test_duration = Duration::from_secs(35); // Test for 35 seconds to mature all windows
    
    // Simulate MIDI clock ticks
    while start_time.elapsed() < test_duration {
        let now = Instant::now();
        window.tick_count += 1;
        
        // Get BPM readings from all mature windows
        let tempo_readings = window.calculate_bpm_readings(now);
        
        // Average the readings if we have any
        if !tempo_readings.is_empty() {
            let average_bpm = tempo_readings.iter().sum::<f32>() / tempo_readings.len() as f32;
            final_tempo = Some(average_bpm);
            final_window_count = tempo_readings.len();
        }
        
        // Wait for the next tick
        thread::sleep(tick_interval);
    }
    
    match final_tempo {
        Some(detected) => {
            let error = (detected - target_bpm).abs();
            let error_percent = (error / target_bpm) * 100.0;
            println!("detected {:>6.1} BPM (error: {:>4.1}%, {} windows)", 
                     detected, error_percent, final_window_count);
            Ok((detected, final_window_count))
        },
        None => {
            println!("❌ NO DETECTION");
            Err("No tempo detected".into())
        }
    }
}