//! Row 1 Scrolling Monitor
//! 
//! This utility monitors row 1 step advancement in real-time to detect skips
//! or timing issues in the position scrolling LED. It tracks:
//! - Sequential step advancement (1→2→3→4...)
//! - Missing steps (1→3, skipping 2)
//! - Display update timing
//! - Step advancement vs display synchronization
//! 
//! Run with: cargo run --example monitor_row1_scrolling --features desktop

use anyhow::Result;
use log::info;
use std::time::{Duration, Instant};
use std::thread;
use simon_says_seeq_rust::sequencer::Sequencer;
use simon_says_seeq_rust::grid_osc::GridManager;
use std::collections::VecDeque;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🔍 Row 1 Scrolling Monitor");
    info!("=========================");
    info!("Monitoring for step advancement skips and timing issues...");
    
    // Initialize sequencer and grid
    let sequencer = Sequencer::new();
    let mut grid_manager = GridManager::new()?;
    let connected_grids = grid_manager.get_connected_grids();
    
    if connected_grids.is_empty() {
        info!("❌ No grids connected");
        return Ok(());
    }
    
    let main_grid_id = connected_grids[0].clone();
    info!("📱 Using grid: {}", main_grid_id);
    
    // Step tracking variables
    let mut last_step = 0usize;
    let mut step_history: VecDeque<(usize, Instant)> = VecDeque::new();
    let mut skip_count = 0u32;
    let mut total_steps = 0u32;
    let mut last_display_update = Instant::now();
    let monitor_duration = Duration::from_secs(30); // Monitor for 30 seconds
    let start_time = Instant::now();
    
    info!("🎯 Monitoring Row 1 for {} seconds...", monitor_duration.as_secs());
    info!("Expected: Sequential advancement 1→2→3→4→5→6→7→8→9→10→11→12→13→14→15→16→1→2...");
    info!("");
    
    while start_time.elapsed() < monitor_duration {
        // Get current row 0 state (0-indexed)
        if let Some(row_state) = sequencer.get_row_states(0) {
            let current_step = row_state.current_step;
            let current_time = Instant::now();
            
            // Check if step has changed
            if current_step != last_step {
                total_steps += 1;
                
                // Calculate expected next step
                let expected_step = if last_step == 0 {
                    current_step // First observation
                } else if last_step == 16 {
                    1 // Wrap around
                } else {
                    last_step + 1 // Normal increment
                };
                
                // Check for skip
                let is_skip = last_step != 0 && current_step != expected_step;
                
                if is_skip {
                    skip_count += 1;
                    info!("⚠️  SKIP DETECTED: Step {} → {} (expected {})", 
                          last_step, current_step, expected_step);
                    
                    // Check if it's a multi-step skip
                    let steps_skipped = if current_step > expected_step {
                        current_step - expected_step
                    } else if current_step < expected_step && last_step == 16 {
                        current_step // Wrapped around
                    } else if current_step < expected_step {
                        (16 - expected_step) + current_step + 1
                    } else {
                        1
                    };
                    
                    if steps_skipped > 1 {
                        info!("🚨 MULTI-STEP SKIP: Skipped {} steps", steps_skipped);
                    }
                } else if last_step != 0 {
                    // Normal advancement - log occasionally
                    if total_steps % 16 == 0 {
                        info!("✅ Step {} → {} (cycle {})", last_step, current_step, total_steps / 16 + 1);
                    }
                }
                
                // Record step with timestamp
                step_history.push_back((current_step, current_time));
                
                // Keep only recent history (last 20 steps)
                while step_history.len() > 20 {
                    step_history.pop_front();
                }
                
                last_step = current_step;
            }
        }
        
        // Update display periodically to monitor display timing
        if last_display_update.elapsed().as_millis() > 33 { // ~30 FPS
            update_row1_display(&sequencer, &mut grid_manager, &main_grid_id)?;
            last_display_update = Instant::now();
        }
        
        // Small sleep to prevent excessive CPU usage
        thread::sleep(Duration::from_millis(10));
    }
    
    // Final report
    info!("");
    info!("📊 MONITORING RESULTS");
    info!("====================");
    info!("Duration: {} seconds", monitor_duration.as_secs());
    info!("Total steps observed: {}", total_steps);
    info!("Skip count: {}", skip_count);
    
    if total_steps > 0 {
        let skip_percentage = (skip_count as f32 / total_steps as f32) * 100.0;
        info!("Skip rate: {:.2}%", skip_percentage);
        
        if skip_count == 0 {
            info!("✅ PERFECT: No skips detected - scrolling working correctly");
        } else if skip_percentage < 5.0 {
            info!("⚠️  MINOR ISSUE: Low skip rate - occasional timing issue");
        } else {
            info!("🚨 MAJOR ISSUE: High skip rate - significant scrolling problem");
        }
    } else {
        info!("❌ NO DATA: No step advancement detected - sequencer may not be running");
    }
    
    // Show recent step history
    info!("");
    info!("📈 Recent Step History:");
    for (step, timestamp) in &step_history {
        let time_since_start = timestamp.duration_since(start_time).as_secs_f32();
        info!("  Step {} at {:.2}s", step, time_since_start);
    }
    
    // Analyze timing patterns
    if step_history.len() > 1 {
        info!("");
        info!("⏱️  Timing Analysis:");
        let mut intervals: Vec<f32> = Vec::new();
        
        for i in 1..step_history.len() {
            let interval = step_history[i].1.duration_since(step_history[i-1].1).as_secs_f32();
            intervals.push(interval);
        }
        
        if !intervals.is_empty() {
            let avg_interval = intervals.iter().sum::<f32>() / intervals.len() as f32;
            let min_interval = intervals.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_interval = intervals.iter().cloned().fold(0.0f32, f32::max);
            
            info!("  Average step interval: {:.3}s", avg_interval);
            info!("  Min interval: {:.3}s", min_interval);
            info!("  Max interval: {:.3}s", max_interval);
            
            // Check for irregular timing
            let timing_variance = max_interval - min_interval;
            if timing_variance > avg_interval * 0.5 {
                info!("⚠️  HIGH TIMING VARIANCE: Step intervals vary significantly");
                info!("    This could cause visual scrolling irregularities");
            } else {
                info!("✅ CONSISTENT TIMING: Step intervals are regular");
            }
        }
    }
    
    Ok(())
}

/// Update Row 1 display (simplified version of main sequencer logic)
fn update_row1_display(
    sequencer: &Sequencer,
    grid_manager: &mut GridManager,
    grid_id: &str
) -> Result<()> {
    if let Some(row_state) = sequencer.get_row_states(0) {
        for seq_x in 0..=15 {
            let pattern_value = sequencer.get_grid_value(seq_x, 0);
            let is_current_step = seq_x == row_state.current_step;
            
            // 4 brightness levels based on pattern and position
            let brightness = match (pattern_value > 0, is_current_step) {
                (false, false) => 0,     // No pattern, not current position
                (false, true) => 6,      // No pattern, but current position  
                (true, false) => 10,     // Has pattern, not current position
                (true, true) => 14,      // Has pattern AND current position
            };
            
            // Use native 0-based coordinates directly
            grid_manager.set_led(grid_id, seq_x, 0, brightness, "monitor_row1_scrolling")?;
        }
    }
    
    grid_manager.refresh()?;
    Ok(())
}