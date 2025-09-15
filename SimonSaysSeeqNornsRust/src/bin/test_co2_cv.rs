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
        if let Some(co2_step_value) = co2_manager.advance_step() {
            // Convert CO2 value to voltages (same logic as main application)
            let co2_step_voltage = co2_manager.get_co2_voltage_offset(co2_step_value);
            
            // Get tick-based CO2 value for output 4 (advances on each tick)
            let co2_tick_value = if let Some(tick_value) = co2_manager.advance_tick() {
                tick_value
            } else {
                co2_step_value // Fall back to step value if tick data not available
            };
            
            let step_delta_voltage = co2_manager.get_step_delta_voltage();
            let tick_delta_voltage = co2_manager.get_tick_delta_voltage();
            
            let voltages = [
                co2_step_voltage,                    // Output 1: CO2 voltage (0-10V range)
                (co2_tick_value - 318.0) / 482.0 * 10.0, // Output 2: Tick-based CO2 as unipolar voltage (318-800ppm → 0-10V)
                step_delta_voltage,                  // Output 3: Step delta (bipolar -5V to +5V)
                tick_delta_voltage,                  // Output 4: Tick delta (bipolar -5V to +5V)
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
                        info!("🎛️  Step#{} Tick#{}: Step:{:.2}ppm Tick:{:.2}ppm Δ:{:.3}ppm/{:.3}ppm -> [Step:{:.3}V, Tick:{:.3}V, StepΔ:{:.3}V, TickΔ:{:.3}V] ✅", 
                              co2_manager.get_step_counter(), co2_manager.get_tick_counter(), co2_step_value, co2_tick_value,
                              co2_manager.get_step_delta(), co2_manager.get_tick_delta(),
                              clamped_voltages[0], clamped_voltages[1], 
                              clamped_voltages[2], clamped_voltages[3]);
                    }
                    Err(e) => {
                        info!("🎛️  Step#{} Tick#{}: Step:{:.2}ppm Tick:{:.2}ppm Δ:{:.3}ppm/{:.3}ppm -> [Step:{:.3}V, Tick:{:.3}V, StepΔ:{:.3}V, TickΔ:{:.3}V] ❌ ({})", 
                              co2_manager.get_step_counter(), co2_manager.get_tick_counter(), co2_step_value, co2_tick_value,
                              co2_manager.get_step_delta(), co2_manager.get_tick_delta(),
                              clamped_voltages[0], clamped_voltages[1], 
                              clamped_voltages[2], clamped_voltages[3], e);
                    }
                }
            } else {
                info!("🎛️  Step {}: Step:{:.2}ppm Tick:{:.2}ppm -> [CO2:{:.3}V, Half:{:.3}V, Dev:{:.3}V, TickUni:{:.3}V] (simulated)", 
                      step, co2_step_value, co2_tick_value,
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
    
    // Create fake CO2 values for demonstration (step-based)
    let fake_co2_values = [
        410.5, 411.2, 409.8, 412.1, 410.9, 
        413.3, 411.7, 410.2, 412.8, 411.4,
        409.6, 413.1, 412.5, 410.8, 411.9,
        409.3, 412.7, 411.1, 410.4, 412.2
    ];
    
    // Create fake CO2 values for tick-based advancement (slightly different values)
    let fake_tick_co2_values = [
        410.3, 411.0, 409.6, 411.9, 410.7, 
        413.1, 411.5, 410.0, 412.6, 411.2,
        409.4, 412.9, 412.3, 410.6, 411.7,
        409.1, 412.5, 410.9, 410.2, 412.0
    ];

    let mut crow = Crow::new()?;
    match crow.initialize() {
        Ok(()) => info!("🎛️  Crow CV output initialized"),
        Err(e) => info!("🎛️  Crow not available: {} - will simulate", e),
    }

    for (step, &co2_step_value) in fake_co2_values.iter().enumerate() {
        let step = step + 1;
        
        // Use the same voltage calculation logic
        let co2_step_voltage = ((co2_step_value - 280.0_f32) / (450.0_f32 - 280.0_f32)).clamp(0.0_f32, 1.0_f32) * 10.0_f32;
        
        // Get corresponding tick-based value
        let co2_tick_value = fake_tick_co2_values[(step - 1) % fake_tick_co2_values.len()];
        
        // Simulate delta values when no real co2_manager is available
        let step_delta_voltage = 0.0; // Simulated step delta
        let tick_delta_voltage = 0.0; // Simulated tick delta
        
        let voltages = [
            co2_step_voltage,                    // Output 1: CO2 voltage (0-10V range)
            (co2_tick_value - 318.0) / 482.0 * 10.0, // Output 2: Tick-based CO2 as unipolar voltage (318-800ppm → 0-10V)
            step_delta_voltage,                  // Output 3: Step delta (bipolar -5V to +5V)
            tick_delta_voltage,                  // Output 4: Tick delta (bipolar -5V to +5V)
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

        info!("🎛️  Step {}: Step:{:.2}ppm Tick:{:.2}ppm -> [CO2:{:.3}V, Half:{:.3}V, Dev:{:.3}V, TickUni:{:.3}V] (simulated)", 
              step, co2_step_value, co2_tick_value,
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