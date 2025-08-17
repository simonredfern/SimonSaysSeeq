//! Test to demonstrate progressive window activation in multi-scale tempo detection
//! Shows how windows become active one by one as time progresses

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

    println!("🕰️  PROGRESSIVE WINDOW ACTIVATION TEST");
    println!("=====================================");
    println!("Demonstrates how windows become active over time");
    println!("Only 'full' windows (with complete duration of data) are used\n");

    test_progressive_activation(120.0)?;

    Ok(())
}

fn test_progressive_activation(target_bpm: f32) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing progressive activation at {:.1} BPM", target_bpm);
    println!("Windows: 1s, 2s, 3s, 4s, 5s");
    println!("Expected progression:");
    println!("• t=1s:  1 window active (1s)");
    println!("• t=2s:  2 windows active (1s, 2s)");
    println!("• t=3s:  3 windows active (1s, 2s, 3s)");
    println!("• t=4s:  4 windows active (1s, 2s, 3s, 4s)");
    println!("• t=5s:  5 windows active (ALL)\n");

    // Calculate tick interval for the target BPM
    let ticks_per_second = (target_bpm / 60.0) * 24.0;
    let tick_interval = Duration::from_secs_f32(1.0 / ticks_per_second);
    
    // Create multi-scale window
    let mut window = MultiScaleTickWindows::new();
    
    let start_time = Instant::now();
    let test_duration = Duration::from_secs(7); // Test for 7 seconds
    
    // Track when we've reported each stage
    let mut reported_stages = std::collections::HashSet::new();
    
    println!("📊 REAL-TIME WINDOW ACTIVATION:");
    println!("Time | Active Windows | Detected BPM | Status");
    println!("-----|----------------|--------------|--------");
    
    // Simulate MIDI clock ticks
    while start_time.elapsed() < test_duration {
        let now = Instant::now();
        window.add_tick(now);
        
        let elapsed = start_time.elapsed().as_secs_f32();
        
        // Get weighted BPM from all mature windows
        if let Some(detected_bpm) = window.calculate_weighted_bpm(now) {
            // Count active windows
            let earliest_tick = window.tick_timestamps.first().map(|&t| t).unwrap_or(now);
            let elapsed_since_first = now.duration_since(earliest_tick).as_secs_f32();
            let active_windows: Vec<f32> = window.window_durations.iter()
                .filter(|&&duration| elapsed_since_first >= duration)
                .cloned()
                .collect();
            
            let stage_key = active_windows.len();
            
            // Report key milestones
            if !reported_stages.contains(&stage_key) && active_windows.len() > 0 {
                reported_stages.insert(stage_key);
                
                let error = (detected_bpm - target_bpm).abs();
                let error_percent = (error / target_bpm) * 100.0;
                let status = if error_percent < 1.0 { "🏆 Excellent" } else if error_percent < 3.0 { "✅ Good" } else { "⚠️  Warming up" };
                
                let windows_str = active_windows.iter()
                    .map(|&w| if w == 1.0 { "1s".to_string() } else { format!("{}s", w as u8) })
                    .collect::<Vec<_>>()
                    .join(",");
                
                println!("{:4.1}s| {} ({})      | {:>6.1} BPM   | {}", 
                         elapsed, 
                         active_windows.len(),
                         windows_str,
                         detected_bpm, 
                         status);
            }
        }
        
        // Wait for the next tick
        thread::sleep(tick_interval);
    }
    
    // Final summary
    println!("\n📈 FINAL ANALYSIS:");
    if let Some(final_bpm) = window.calculate_weighted_bpm(Instant::now()) {
        let final_error = (final_bpm - target_bpm).abs();
        let final_error_percent = (final_error / target_bpm) * 100.0;
        
        println!("• Final BPM: {:.1} (target: {:.1})", final_bpm, target_bpm);
        println!("• Final error: {:.1} BPM ({:.2}%)", final_error, final_error_percent);
        println!("• All 5 windows active after 5 seconds");
        
        if final_error_percent < 1.0 {
            println!("• 🏆 EXCELLENT: Sub-1% accuracy achieved");
        } else if final_error_percent < 3.0 {
            println!("• ✅ GOOD: Sub-3% accuracy achieved");
        } else {
            println!("• ⚠️  NEEDS IMPROVEMENT: Error exceeds 3%");
        }
    }
    
    println!("\n🔍 KEY INSIGHTS:");
    println!("• Windows activate progressively as they become 'full'");
    println!("• Early readings use fewer, shorter windows (higher error)");
    println!("• Accuracy improves as more windows become active");
    println!("• Full accuracy achieved when all 5 windows are active");
    println!("• Prevents artificially low BPM readings at startup");
    
    Ok(())
}