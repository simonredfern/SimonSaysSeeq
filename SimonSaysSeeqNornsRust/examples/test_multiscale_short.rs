//! Short test of the multi-scale MIDI tempo detection algorithm
//! Uses windows of 1, 2, 4, 8 seconds and averages them (shortened test)

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

    println!("🎵 MULTI-SCALE TEMPO DETECTION (SHORT TEST)");
    println!("===========================================");
    println!("Strategy: Multiple windows (1,2,3,4,5s) with weighted averaging");
    println!("Formula: BPM = weighted_sum / total_weight (longer windows = more weight)");
    println!("Test duration: 7 seconds per tempo\n");

    // Test different tempos
    let test_tempos = vec![60.0, 90.0, 120.0, 150.0, 180.0];
    let mut results = Vec::new();
    
    for tempo in test_tempos {
        let result = test_tempo_detection(tempo)?;
        results.push((tempo, result));
        thread::sleep(Duration::from_millis(100));
    }

    // Print results table
    println!("\n📊 MULTI-SCALE ALGORITHM RESULTS");
    println!("┌─────────────┬─────────────┬─────────────┬─────────────┬─────────────┐");
    println!("│ Target BPM  │ Detected    │ Error (BPM) │ Error (%)   │ Status      │");
    println!("├─────────────┼─────────────┼─────────────┼─────────────┼─────────────┤");
    
    let mut total_error = 0.0;
    let mut excellent_count = 0;
    let mut good_count = 0;
    
    for (target, (detected, _)) in &results {
        let error_bpm = (detected - target).abs();
        let error_percent = (error_bpm / target) * 100.0;
        let status = if error_percent < 1.0 { 
            excellent_count += 1;
            "🏆 EXCELLENT"
        } else if error_percent < 2.0 { 
            good_count += 1;
            "✅ VERY GOOD" 
        } else if error_percent < 4.0 { 
            "✅ GOOD" 
        } else { 
            "❌ NEEDS WORK" 
        };
        
        total_error += error_percent;
        
        println!("│ {:>9.1}   │ {:>9.1}   │ {:>9.1}   │ {:>8.1}%   │ {}   │", 
                 target, detected, error_bpm, error_percent, status);
    }
    
    println!("└─────────────┴─────────────┴─────────────┴─────────────┴─────────────┘");
    
    let avg_error = total_error / results.len() as f32;
    let excellent_rate = (excellent_count as f32 / results.len() as f32) * 100.0;
    let good_rate = ((excellent_count + good_count) as f32 / results.len() as f32) * 100.0;
    
    println!("\n📈 PERFORMANCE SUMMARY:");
    println!("• Average error: {:.2}%", avg_error);
    println!("• Excellent (<1%): {}/{} ({:.1}%)", excellent_count, results.len(), excellent_rate);
    println!("• Very good (<2%): {}/{} ({:.1}%)", excellent_count + good_count, results.len(), good_rate);
    
    if avg_error < 1.5 && excellent_rate >= 60.0 {
        println!("🏆 OUTSTANDING: Multi-scale algorithm is excellent!");
    } else if avg_error < 3.0 && good_rate >= 80.0 {
        println!("🎉 GREAT: Multi-scale algorithm performs very well!");
    } else {
        println!("⚠️  ACCEPTABLE: Algorithm works but could be improved");
    }
    
    println!("🔬 WINDOW ANALYSIS:");
    println!("• 1s window:  Very fast response, weight=1");
    println!("• 2s window:  Fast response, weight=2");
    println!("• 3s window:  Good balance, weight=3"); 
    println!("• 4s window:  Better stability, weight=4");
    println!("• 5s window:  Maximum stability, weight=5");
    println!("• Final reading: Weighted average favoring longer windows");
    
    println!("⏱️  MATURATION TIMELINE:");
    println!("• t=1s:  1 window active (1s, weight=1)");
    println!("• t=2s:  2 windows active (1s,2s, total_weight=3)");
    println!("• t=3s:  3 windows active (1s,2s,3s, total_weight=6)");
    println!("• t=4s:  4 windows active (1s,2s,3s,4s, total_weight=10)");
    println!("• t=5s:  5 windows active (total_weight=15, full accuracy)");

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
    let test_duration = Duration::from_secs(7); // 7 second test to mature all windows
    
    // Simulate MIDI clock ticks
    while start_time.elapsed() < test_duration {
        let now = Instant::now();
        window.add_tick(now);
        
        // Get weighted BPM from all mature windows
        if let Some(weighted_bpm) = window.calculate_weighted_bpm(now) {
            final_tempo = Some(weighted_bpm);
            // Count how many windows contributed (approximate)
            final_window_count = window.window_durations.iter()
                .filter(|&&duration| {
                    let elapsed = now.duration_since(start_time).as_secs_f32();
                    elapsed >= duration
                })
                .count();
        }
        
        // Wait for the next tick
        thread::sleep(tick_interval);
    }
    
    match final_tempo {
        Some(detected) => {
            let error = (detected - target_bpm).abs();
            let error_percent = (error / target_bpm) * 100.0;
            println!("detected {:>6.1} BPM ({:.1}% error, {} windows)", 
                     detected, error_percent, final_window_count);
            Ok((detected, final_window_count))
        },
        None => {
            println!("❌ NO DETECTION");
            Err("No tempo detected".into())
        }
    }
}