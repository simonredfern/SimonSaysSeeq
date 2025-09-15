//! Test program for CO2 CV per step functionality
//! Demonstrates how the new CO2-based CV output system works

use anyhow::Result;
use env_logger;
use log::info;
use simon_says_seeq_rust::config::Config;
use simon_says_seeq_rust::co2::Co2Manager;
use simon_says_seeq_rust::crow::Crow;

use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("=== CO2 CV Per Step Test ===\n");

    // Load configuration
    let config = Config::load_or_default()?;
    
    // Initialize CO2 manager
    let mut co2_manager = match Co2Manager::new(config.co2.clone()) {
        Ok(manager) => {
            info!("✅ CO2 manager initialized successfully");
            if manager.has_data() {
                info!("📊 {}", manager.get_data_summary());
                manager
            } else {
                println!("⚠️  No CO2 data loaded - creating test data simulation");
                return simulate_without_data();
            }
        }
        Err(e) => {
            println!("❌ CO2 manager initialization failed: {}", e);
            return simulate_without_data();
        }
    };

    // Initialize Crow (optional - will work without it)
    let mut crow = Crow::new()?;
    match crow.initialize() {
        Ok(()) => {
            info!("🎛️  Crow CV output initialized");
        }
        Err(e) => {
            info!("🎛️  Crow not available: {} - will simulate CV output", e);
        }
    }

    // Simulate stepping through CO2 data
    println!("\n--- Stepping Through CO2 Data ---");
    
    for step in 1..=20 {
        if let Some(co2_value) = co2_manager.advance_step() {
            // Convert CO2 value to voltages (same logic as main application)
            let co2_voltage = co2_manager.get_co2_voltage_offset(co2_value);
            
            let voltages = [
                co2_voltage,                    // Output 1: CO2 voltage (0-10V range)
                co2_voltage * 0.5,             // Output 2: CO2 voltage scaled down (0-5V range)
                (co2_value - 400.0) / 50.0,    // Output 3: CO2 deviation from 400ppm baseline
                (co2_value / 100.0) - 4.0,     // Output 4: CO2 as bipolar voltage (410ppm = 0.1V)
            ];

            // Clamp all voltages to Crow's safe range
            let clamped_voltages = [
                voltages[0].clamp(-5.0, 10.0),
                voltages[1].clamp(-5.0, 10.0),
                voltages[2].clamp(-5.0, 10.0),
                voltages[3].clamp(-5.0, 10.0),
            ];

            // Send to Crow if available
            if crow.is_enabled() {
                match crow.set_all_outputs(
                    clamped_voltages[0], 
                    clamped_voltages[1], 
                    clamped_voltages[2], 
                    clamped_voltages[3]
                ) {
                    Ok(()) => {
                        info!("🎛️  Step {}: {:.2} ppm -> [CO2:{:.3}V, Half:{:.3}V, Dev:{:.3}V, Bipolar:{:.3}V] ✅", 
                              step, co2_value, 
                              clamped_voltages[0], clamped_voltages[1], 
                              clamped_voltages[2], clamped_voltages[3]);
                    }
                    Err(e) => {
                        info!("🎛️  Step {}: {:.2} ppm -> [CO2:{:.3}V, Half:{:.3}V, Dev:{:.3}V, Bipolar:{:.3}V] ❌ ({})", 
                              step, co2_value, 
                              clamped_voltages[0], clamped_voltages[1], 
                              clamped_voltages[2], clamped_voltages[3], e);
                    }
                }
            } else {
                info!("🎛️  Step {}: {:.2} ppm -> [CO2:{:.3}V, Half:{:.3}V, Dev:{:.3}V, Bipolar:{:.3}V] (simulated)", 
                      step, co2_value, 
                      clamped_voltages[0], clamped_voltages[1], 
                      clamped_voltages[2], clamped_voltages[3]);
            }

            // Small delay to simulate sequencer timing
            thread::sleep(Duration::from_millis(200));
        } else {
            println!("Step {}: No CO2 data available", step);
        }
    }

    // Reset CV outputs to 0V
    if crow.is_enabled() {
        let _ = crow.set_all_outputs(0.0, 0.0, 0.0, 0.0);
        info!("🎛️  CV outputs reset to 0V");
    }

    println!("\n=== Test Complete ===");
    Ok(())
}

fn simulate_without_data() -> Result<()> {
    println!("\n--- Simulating Without Real CO2 Data ---");
    
    // Create fake CO2 values for demonstration
    let fake_co2_values = [
        410.5, 411.2, 409.8, 412.1, 410.9, 
        413.3, 411.7, 410.2, 412.8, 411.4,
        409.6, 413.1, 412.5, 410.8, 411.9,
        409.3, 412.7, 411.1, 410.4, 412.2
    ];

    let mut crow = Crow::new()?;
    match crow.initialize() {
        Ok(()) => info!("🎛️  Crow CV output initialized"),
        Err(e) => info!("🎛️  Crow not available: {} - will simulate", e),
    }

    for (step, &co2_value) in fake_co2_values.iter().enumerate() {
        let step = step + 1;
        
        // Use the same voltage calculation logic
        let co2_voltage = ((co2_value - 280.0_f32) / (450.0_f32 - 280.0_f32)).clamp(0.0_f32, 1.0_f32) * 10.0_f32;
        
        let voltages = [
            co2_voltage,                    // Output 1: CO2 voltage (0-10V range)
            co2_voltage * 0.5,             // Output 2: CO2 voltage scaled down (0-5V range)
            (co2_value - 400.0) / 50.0,    // Output 3: CO2 deviation from 400ppm baseline
            (co2_value / 100.0) - 4.0,     // Output 4: CO2 as bipolar voltage (410ppm = 0.1V)
        ];

        let clamped_voltages = [
            voltages[0].clamp(-5.0, 10.0),
            voltages[1].clamp(-5.0, 10.0),
            voltages[2].clamp(-5.0, 10.0),
            voltages[3].clamp(-5.0, 10.0),
        ];

        if crow.is_enabled() {
            let _ = crow.set_all_outputs(
                clamped_voltages[0], 
                clamped_voltages[1], 
                clamped_voltages[2], 
                clamped_voltages[3]
            );
        }

        info!("🎛️  Step {}: {:.2} ppm -> [CO2:{:.3}V, Half:{:.3}V, Dev:{:.3}V, Bipolar:{:.3}V] (simulated)", 
              step, co2_value, 
              clamped_voltages[0], clamped_voltages[1], 
              clamped_voltages[2], clamped_voltages[3]);

        thread::sleep(Duration::from_millis(200));
    }

    if crow.is_enabled() {
        let _ = crow.set_all_outputs(0.0, 0.0, 0.0, 0.0);
        info!("🎛️  CV outputs reset to 0V");
    }

    println!("\n=== Simulation Complete ===");
    Ok(())
}