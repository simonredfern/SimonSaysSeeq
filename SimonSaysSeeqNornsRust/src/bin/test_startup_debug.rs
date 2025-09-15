//! Test program to demonstrate startup debug output
//! Shows config file location and CO2 file path during initialization

use anyhow::Result;
use env_logger;
use log::info;
use simon_says_seeq_rust::config::Config;
use simon_says_seeq_rust::co2::Co2Manager;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Initialize logging with INFO level to see our debug prints
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("=== SimonSaysSeeq Startup Debug Test ===\n");

    // 1. Show config file location (same as in SimonSaysSeeq::new())
    let config_path = Config::get_config_path();
    info!("📁 Config file location: {:?}", config_path);
    
    // 2. Load config (this will create default if none exists)
    let config = Config::load_or_default()?;
    
    // 3. Show CO2 file path (same as in main.rs)
    let co2_file_path = PathBuf::from(&config.co2.data_dir).join("simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_all_daily.csv");
    info!("📊 CO2 file path: {:?}", co2_file_path);
    
    // 4. Try to initialize CO2 manager to see its debug output
    println!("\n--- CO2 Manager Initialization ---");
    match Co2Manager::new(config.co2.clone()) {
        Ok(_manager) => {
            info!("📊 CO2 manager initialized successfully");
        }
        Err(e) => {
            info!("📊 CO2 manager initialization failed: {}", e);
            info!("📊 This is normal if CO2 data file doesn't exist");
        }
    }
    
    println!("\n--- Configuration Summary ---");
    println!("Config file: {:?}", config_path);
    println!("Config exists: {}", config_path.exists());
    println!("CO2 enabled: {}", config.co2.enabled);
    println!("CO2 data dir: {}", config.co2.data_dir);
    println!("CO2 file: {:?}", co2_file_path);
    println!("CO2 file exists: {}", co2_file_path.exists());
    println!("Hardware simulation mode: {}", config.hardware.simulation_mode);
    println!("MIDI auto-detect: {}", config.midi.auto_detect_clock);
    
    println!("\n=== Test Complete ===");
    Ok(())
}