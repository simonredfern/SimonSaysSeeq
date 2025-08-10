use simon_says_seeq_rust::grid_osc::GridManager;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🔍 Comprehensive Interference Test");
    println!("Testing various interference scenarios between rows and columns");
    
    // Initialize grid manager
    let mut grid_manager = GridManager::new()?;
    
    // Wait for grid connection
    println!("Waiting for grid connection...");
    let mut grids = grid_manager.get_connected_grids();
    while grids.is_empty() {
        thread::sleep(Duration::from_millis(100));
        grids = grid_manager.get_connected_grids();
    }
    
    let main_grid_id = &grids[0];
    println!("Connected to grid: {}", main_grid_id);
    
    // Clear entire grid
    for x in 0..16 {
        for y in 0..8 {
            grid_manager.set_led(main_grid_id, x, y, 0, "example_caller")?;
        }
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(1000));
    
    println!("\n=== TEST 1: Adjacent Row Interference ===");
    
    // Set Row 1 LED
    println!("Setting Row 1 Col 5 -> grid(4,0)");
    grid_manager.set_led(main_grid_id, 4, 0, 15, "example_caller")?;
    grid_manager.refresh()?;
    println!("Row 1 Col 5 should be lit - observe 5 seconds");
    thread::sleep(Duration::from_millis(5000));
    
    // Now set Row 2 LED and check if Row 1 is affected
    println!("Setting Row 2 Col 5 -> grid(4,1) - SAME COLUMN as Row 1");
    grid_manager.set_led(main_grid_id, 4, 1, 15, "example_caller")?;
    grid_manager.refresh()?;
    println!("Both Row 1 and Row 2 Col 5 should be lit");
    println!("CRITICAL: Did Row 1 Col 5 stay lit? (Y/N)");
    thread::sleep(Duration::from_millis(5000));
    
    // Clear for next test
    grid_manager.set_led(main_grid_id, 4, 0, 0, "example_caller")?;
    grid_manager.set_led(main_grid_id, 4, 1, 0, "example_caller")?;
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(1000));
    
    println!("\n=== TEST 2: Same Column Different Rows ===");
    
    // Light up column 8 for multiple rows sequentially
    println!("Setting Col 8 for Rows 1,2,3,4,5 one by one:");
    
    for test_row in 1..=5 {
        println!("  Adding Row {} Col 8 -> grid(7,{})", test_row, test_row - 1);
        grid_manager.set_led(main_grid_id, 7, test_row - 1, 15, "example_caller")?;
        grid_manager.refresh()?;
        println!("  {} LEDs should now be lit in Column 8", test_row);
        thread::sleep(Duration::from_millis(2000));
    }
    
    println!("Final check: Do all 5 LEDs remain lit in Column 8?");
    thread::sleep(Duration::from_millis(3000));
    
    // Clear for next test
    for y in 0..8 {
        grid_manager.set_led(main_grid_id, 7, y, 0, "example_caller")?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(1000));
    
    println!("\n=== TEST 3: Rapid Sequential Setting ===");
    
    // Set multiple LEDs rapidly to test for timing issues
    let test_leds = [
        (0, 0),   // Row 1 Col 1
        (4, 0),   // Row 1 Col 5
        (8, 0),   // Row 1 Col 9
        (12, 0),  // Row 1 Col 13
        (0, 5),   // Row 6 Col 1
        (4, 5),   // Row 6 Col 5
        (8, 5),   // Row 6 Col 9
        (12, 5),  // Row 6 Col 13
    ];
    
    println!("Setting 8 LEDs rapidly in sequence:");
    for (i, (x, y)) in test_leds.iter().enumerate() {
        println!("  LED {}: grid({},{}) brightness 15", i + 1, x, y);
        grid_manager.set_led(main_grid_id, *x, *y, 15, "example_caller")?;
        grid_manager.refresh()?;
        thread::sleep(Duration::from_millis(200)); // Fast setting
    }
    
    println!("All 8 LEDs should remain lit after rapid setting");
    println!("CRITICAL: Count how many LEDs are actually lit (should be 8)");
    thread::sleep(Duration::from_millis(5000));
    
    // Clear for next test
    for (x, y) in test_leds {
        grid_manager.set_led(main_grid_id, x, y, 0, "example_caller")?;
    }
    grid_manager.refresh()?;
    thread::sleep(Duration::from_millis(1000));
    
    println!("\n=== TEST 4: Batch vs Individual Setting ===");
    
    // Test setting all LEDs at once vs individually
    println!("Setting all Row 1 LEDs simultaneously:");
    grid_manager.set_led(main_grid_id, 0, 0, 15, "example_caller")?;   // Col 1
    grid_manager.set_led(main_grid_id, 4, 0, 15, "example_caller")?;   // Col 5
    grid_manager.set_led(main_grid_id, 8, 0, 15, "example_caller")?;   // Col 9
    grid_manager.set_led(main_grid_id, 12, 0, 15, "example_caller")?;  // Col 13
    grid_manager.refresh()?; // Single refresh for all
    
    println!("Row 1 should show 4 LEDs - observe for 3 seconds");
    thread::sleep(Duration::from_millis(3000));
    
    println!("✅ Interference tests complete");
    println!("If any tests showed LEDs turning off unexpectedly,");
    println!("we've found the source of the interference bug");
    println!("Press Ctrl+C when done inspecting...");
    
    // Keep final state for inspection
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}