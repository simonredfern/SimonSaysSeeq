//! Debug OSC Communication Test
//! 
//! This test isolates the OSC communication to help debug why our Rust code
//! isn't successfully lighting LEDs even though manual oscsend commands work.

use simon_says_seeq_rust::grid_osc::GridManager;
use log::info;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    info!("🔍 OSC Debug Test");
    info!("==================");
    
    // Create grid manager
    let mut manager = GridManager::new()?;
    
    // Wait for device discovery
    info!("Discovering devices...");
    thread::sleep(Duration::from_secs(3));
    
    let device_ids = manager.get_connected_grids();
    if device_ids.is_empty() {
        println!("❌ No devices found");
        return Ok(());
    }
    
    info!("Found {} device(s)", device_ids.len());
    for device_id in &device_ids {
        let name = manager.get_name(device_id).unwrap_or("unknown".to_string());
        let (cols, rows) = manager.get_dimensions(device_id).unwrap_or((16, 8));
        info!("  - {}: {} ({}x{})", device_id, name, cols, rows);
    }
    
    // Test each device
    for device_id in device_ids {
        info!("\n🧪 Testing device: {}", device_id);
        
        // Test 1: Try to set a single LED
        info!("Step 1: Setting LED at (0,0) with brightness 15");
        manager.set_led(&device_id, 0, 0, 15)?;
        
        println!("👀 Check grid {} - do you see LED at (0,0)? Press Enter to continue...", device_id);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        // Test 2: Try different brightness
        info!("Step 2: Setting LED at (0,1) with brightness 1");
        manager.set_led(&device_id, 0, 1, 1)?;
        
        println!("👀 Check grid {} - do you see LED at (0,1)? Press Enter to continue...", device_id);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        // Test 3: Clear all
        info!("Step 3: Clearing all LEDs");
        manager.clear_all(&device_id)?;
        
        println!("👀 Check grid {} - are all LEDs off? Press Enter to continue...", device_id);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        // Test 4: Flash test
        info!("Step 4: Flash test - 5 quick flashes");
        for i in 0..5 {
            info!("Flash {}/5", i + 1);
            manager.set_led(&device_id, 1, 1, 15)?;
            thread::sleep(Duration::from_millis(200));
            manager.set_led(&device_id, 1, 1, 0)?;
            thread::sleep(Duration::from_millis(200));
        }
        
        println!("👀 Did you see 5 flashes at (1,1) on grid {}? (y/n):", device_id);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let response = input.trim().to_lowercase();
        
        if response == "y" || response == "yes" {
            info!("✅ Grid {} is working!", device_id);
        } else {
            info!("❌ Grid {} is not responding", device_id);
        }
    }
    
    info!("\n🏁 Debug test complete");
    Ok(())
}