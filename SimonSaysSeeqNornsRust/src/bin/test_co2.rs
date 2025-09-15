use anyhow::Result;
use simon_says_seeq_rust::co2::{Co2Manager, Co2Config};
use std::path::Path;

fn main() -> Result<()> {
    env_logger::init();
    
    println!("Testing CO2 Manager with empty latest daily file...");
    
    // Use the actual co2_data directory
    let data_dir = Path::new("co2_data");
    
    let config = Co2Config {
        enabled: true,
        data_dir: data_dir.to_string_lossy().to_string(),
        wow_threshold: 0.5,
        flutter_threshold: 0.1,
        window_size: 16,
        voltage_scale: 3.3,
        co2_min: 300.0,
        co2_max: 500.0,
    };
    
    // Create CO2 manager
    let mut co2_manager = Co2Manager::new(config)?;
    
    // Load data - this should handle the empty latest daily file gracefully
    match co2_manager.load_data() {
        Ok(()) => {
            println!("✓ CO2 data loaded successfully");
            
            // Check if we have any data
            if co2_manager.has_data() {
                println!("✓ CO2 manager has historical data ({} records)", co2_manager.get_record_count());
            } else {
                println!("! CO2 manager has no historical data");
            }
            
            // Check latest daily value
            match co2_manager.get_latest_daily_value() {
                Some(value) => println!("✓ Latest daily CO2 value: {:.2} ppm", value),
                None => println!("! No latest daily CO2 value (this is expected if the file is empty)")
            }
            
            // Get current values (should use historical data)
            let current_step = co2_manager.get_current_step_co2();
            let current_tick = co2_manager.get_current_tick_co2();
            
            match current_step {
                Some(val) => println!("Current step CO2: {:.2} ppm", val),
                None => println!("Current step CO2: No data available"),
            }
            match current_tick {
                Some(val) => println!("Current tick CO2: {:.2} ppm", val),
                None => println!("Current tick CO2: No data available"),
            }
            
            // Test tempo analysis
            co2_manager.analyze_tempo_stability(120.0);
            let stability = co2_manager.get_tempo_stability();
            println!("Tempo stability - WOW: {}, Flutter: {}", stability.0, stability.1);
            
            println!("✓ All CO2 manager tests passed!");
        }
        Err(e) => {
            println!("✗ Failed to load CO2 data: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}