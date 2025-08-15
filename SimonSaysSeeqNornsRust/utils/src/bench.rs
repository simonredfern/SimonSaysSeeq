//! Timing accuracy benchmark for MIDI clock generator
//! 
//! This module provides benchmarking utilities to measure the timing accuracy
//! of the MIDI clock generation system.

use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::io::{self, Write};

/// Statistics for timing measurements
#[derive(Debug, Clone)]
pub struct TimingStats {
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub stddev: f64,
    pub jitter: f64,
    pub sample_count: usize,
}

impl TimingStats {
    /// Calculate statistics from a collection of timing measurements
    pub fn from_measurements(measurements: &[Duration]) -> Self {
        if measurements.is_empty() {
            return Self {
                mean: 0.0,
                min: 0.0,
                max: 0.0,
                stddev: 0.0,
                jitter: 0.0,
                sample_count: 0,
            };
        }

        let values: Vec<f64> = measurements.iter()
            .map(|d| d.as_secs_f64() * 1000.0) // Convert to milliseconds
            .collect();

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let stddev = variance.sqrt();

        // Jitter is the standard deviation of the differences between consecutive measurements
        let diffs: Vec<f64> = values.windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .collect();
        
        let jitter = if diffs.len() > 1 {
            let diff_mean = diffs.iter().sum::<f64>() / diffs.len() as f64;
            let diff_variance = diffs.iter()
                .map(|&x| (x - diff_mean).powi(2))
                .sum::<f64>() / diffs.len() as f64;
            diff_variance.sqrt()
        } else {
            0.0
        };

        Self {
            mean,
            min,
            max,
            stddev,
            jitter,
            sample_count: measurements.len(),
        }
    }

    /// Print formatted statistics
    pub fn print(&self) {
        println!("Timing Statistics:");
        println!("  Sample Count: {}", self.sample_count);
        println!("  Mean:         {:.3} ms", self.mean);
        println!("  Min:          {:.3} ms", self.min);
        println!("  Max:          {:.3} ms", self.max);
        println!("  Std Dev:      {:.3} ms", self.stddev);
        println!("  Jitter:       {:.3} ms", self.jitter);
        
        // Quality assessment
        if self.jitter < 0.1 {
            println!("  Quality:      ✅ Excellent (jitter < 0.1ms)");
        } else if self.jitter < 0.5 {
            println!("  Quality:      👍 Good (jitter < 0.5ms)");
        } else if self.jitter < 1.0 {
            println!("  Quality:      ⚠️  Fair (jitter < 1.0ms)");
        } else {
            println!("  Quality:      ❌ Poor (jitter >= 1.0ms)");
        }
    }
}

/// Clock timing benchmark
pub struct ClockBenchmark {
    target_bpm: f32,
    expected_interval: Duration,
    measurements: VecDeque<Duration>,
    last_tick: Option<Instant>,
    max_samples: usize,
}

impl ClockBenchmark {
    /// Create a new benchmark for the given BPM
    pub fn new(bpm: f32, max_samples: usize) -> Self {
        // MIDI clock sends 24 ticks per quarter note
        let ticks_per_second = (bpm * 24.0) / 60.0;
        let expected_interval = Duration::from_secs_f32(1.0 / ticks_per_second);

        Self {
            target_bpm: bpm,
            expected_interval,
            measurements: VecDeque::with_capacity(max_samples),
            last_tick: None,
            max_samples,
        }
    }

    /// Record a clock tick
    pub fn tick(&mut self) {
        let now = Instant::now();
        
        if let Some(last) = self.last_tick {
            let interval = now.duration_since(last);
            
            // Add measurement
            if self.measurements.len() >= self.max_samples {
                self.measurements.pop_front();
            }
            self.measurements.push_back(interval);
        }
        
        self.last_tick = Some(now);
    }

    /// Get current statistics
    pub fn get_stats(&self) -> TimingStats {
        let measurements: Vec<Duration> = self.measurements.iter().cloned().collect();
        TimingStats::from_measurements(&measurements)
    }

    /// Check if we have enough samples for meaningful statistics
    pub fn has_sufficient_samples(&self) -> bool {
        self.measurements.len() >= 100 // At least 100 samples
    }

    /// Get the target interval for this BPM
    pub fn target_interval_ms(&self) -> f64 {
        self.expected_interval.as_secs_f64() * 1000.0
    }

    /// Reset measurements
    pub fn reset(&mut self) {
        self.measurements.clear();
        self.last_tick = None;
    }

    /// Run a timing accuracy test
    pub fn run_accuracy_test(&mut self, duration_secs: u64) -> TimingStats {
        println!("Running timing accuracy test for {} BPM over {} seconds...", 
                 self.target_bpm, duration_secs);
        println!("Expected interval: {:.3} ms", self.target_interval_ms());
        
        self.reset();
        
        let start_time = Instant::now();
        let test_duration = Duration::from_secs(duration_secs);
        let mut tick_count = 0u32;
        
        // Simulate clock generation
        let mut last_tick = Instant::now();
        
        while start_time.elapsed() < test_duration {
            let now = Instant::now();
            if now.duration_since(last_tick) >= self.expected_interval {
                self.tick();
                last_tick = now;
                tick_count += 1;
                
                // Progress indicator
                if tick_count % 240 == 0 { // Every 10 beats at 24 PPQ
                    let elapsed = start_time.elapsed().as_secs();
                    print!(".");
                    if elapsed % 10 == 0 {
                        println!(" {}s", elapsed);
                    }
                }
            }
            
            // Small sleep to prevent busy waiting
            std::thread::sleep(Duration::from_micros(100));
        }
        
        println!("\nTest completed. {} ticks recorded.", tick_count);
        
        let stats = self.get_stats();
        stats.print();
        
        stats
    }
}

/// Comprehensive benchmark suite
pub struct BenchmarkSuite;

impl BenchmarkSuite {
    /// Run benchmarks for multiple BPM values
    pub fn run_bpm_sweep() {
        println!("🎯 MIDI Clock Timing Benchmark Suite");
        println!("=====================================");
        
        let test_bpms = vec![60.0, 100.0, 120.0, 140.0, 180.0, 200.0];
        let test_duration = 10; // seconds per test
        
        for bpm in test_bpms {
            println!("\n📊 Testing {} BPM", bpm);
            println!("{}", "-".repeat(40));
            
            let mut benchmark = ClockBenchmark::new(bpm, 1000);
            let stats = benchmark.run_accuracy_test(test_duration);
            
            // Validate against acceptable thresholds
            if stats.jitter > 1.0 {
                println!("⚠️  WARNING: High jitter detected for {} BPM", bpm);
            }
            
            if (stats.mean - benchmark.target_interval_ms()).abs() > 0.5 {
                println!("⚠️  WARNING: Mean timing error > 0.5ms for {} BPM", bpm);
            }
        }
        
        println!("\n✅ Benchmark suite completed");
    }
    
    /// Run a stress test with rapid BPM changes
    pub fn run_stress_test() {
        println!("🔥 Stress Test - Rapid BPM Changes");
        println!("==================================");
        
        let bpm_sequence = vec![
            120.0, 140.0, 100.0, 180.0, 80.0, 160.0, 110.0, 200.0
        ];
        
        for (i, bpm) in bpm_sequence.iter().enumerate() {
            println!("Phase {}: {} BPM (5 seconds)", i + 1, bpm);
            
            let mut benchmark = ClockBenchmark::new(*bpm, 500);
            let _stats = benchmark.run_accuracy_test(5);
            
            std::thread::sleep(Duration::from_millis(100));
        }
        
        println!("✅ Stress test completed");
    }
    
    /// Run a long duration stability test
    pub fn run_stability_test(duration_minutes: u64) {
        println!("⏱️  Long Duration Stability Test");
        println!("Duration: {} minutes at 120 BPM", duration_minutes);
        println!("{}", "=".repeat(40));
        
        let mut benchmark = ClockBenchmark::new(120.0, 10000);
        let stats = benchmark.run_accuracy_test(duration_minutes * 60);
        
        // Additional stability checks
        if stats.max - stats.min > 5.0 {
            println!("⚠️  WARNING: Large timing variance detected");
        }
        
        if stats.sample_count < (duration_minutes * 60 * 48) as usize {
            println!("⚠️  WARNING: Fewer ticks than expected");
        }
        
        println!("✅ Stability test completed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_stats_calculation() {
        let measurements = vec![
            Duration::from_millis(10),
            Duration::from_millis(11),
            Duration::from_millis(9),
            Duration::from_millis(10),
            Duration::from_millis(12),
        ];
        
        let stats = TimingStats::from_measurements(&measurements);
        
        assert_eq!(stats.sample_count, 5);
        assert!((stats.mean - 10.4).abs() < 0.1); // Should be around 10.4ms
        assert_eq!(stats.min, 9.0);
        assert_eq!(stats.max, 12.0);
    }

    #[test]
    fn test_clock_benchmark_creation() {
        let benchmark = ClockBenchmark::new(120.0, 1000);
        
        // At 120 BPM, 24 PPQ = 48 ticks per second
        // Expected interval = 1000ms / 48 ≈ 20.83ms
        let expected = 1000.0 / 48.0;
        assert!((benchmark.target_interval_ms() - expected).abs() < 0.1);
    }

    #[test]
    fn test_benchmark_tick_recording() {
        let mut benchmark = ClockBenchmark::new(120.0, 10);
        
        assert!(!benchmark.has_sufficient_samples());
        
        // Simulate some ticks
        for _ in 0..5 {
            std::thread::sleep(Duration::from_millis(20));
            benchmark.tick();
        }
        
        assert!(benchmark.measurements.len() == 4); // First tick has no interval
        assert!(!benchmark.has_sufficient_samples());
    }
}

fn main() {
    println!("🎯 MIDI Clock Generator Benchmark");
    println!("=================================");
    println!();
    println!("Choose benchmark type:");
    println!("1. BPM Sweep Test (multiple BPM values)");
    println!("2. Stress Test (rapid BPM changes)");
    println!("3. Stability Test (long duration)");
    println!("4. All tests");
    
    print!("Enter choice (1-4): ");
    io::stdout().flush().ok();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    
    match input.trim() {
        "1" => {
            BenchmarkSuite::run_bpm_sweep();
        }
        "2" => {
            BenchmarkSuite::run_stress_test();
        }
        "3" => {
            print!("Enter duration in minutes (default 5): ");
            io::stdout().flush().ok();
            let mut duration_input = String::new();
            io::stdin().read_line(&mut duration_input).expect("Failed to read input");
            let duration = duration_input.trim().parse::<u64>().unwrap_or(5);
            BenchmarkSuite::run_stability_test(duration);
        }
        "4" => {
            BenchmarkSuite::run_bpm_sweep();
            println!("\n{}\n", "=".repeat(50));
            BenchmarkSuite::run_stress_test();
            println!("\n{}\n", "=".repeat(50));
            BenchmarkSuite::run_stability_test(2); // Short stability test
        }
        _ => {
            println!("Invalid choice, running BPM sweep test...");
            BenchmarkSuite::run_bpm_sweep();
        }
    }
    
    println!("\n🎉 Benchmark completed!");
}