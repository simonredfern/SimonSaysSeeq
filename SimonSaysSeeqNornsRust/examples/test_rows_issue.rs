//! Test program to isolate the rows 2-7 issue
//! 
//! This program will test button presses on all rows to identify
//! why Row 1 works but rows 2-7 have issues

use anyhow::Result;
use log::{info, warn, error, debug};
use std::thread;
use std::time::Duration;

use simon_says_seeq_rust::grid_osc::{GridManager, GridButtonEvent};

fn main() -> Result<()> {
    env_logger::init();
    
    info!("🔧 Grid Rows Issue Test");
    info!("Testing button press behavior across all rows");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Initialize grid manager
    let mut grid = GridManager::new()?;
    
    // Wait for grid connection
    thread::sleep(Duration::from_millis(500));
    
    let connected_grids = grid.get_connected_grids();
    if connected_grids.is_empty() {
        error!("❌ No grids connected!");
        return Ok(());
    }
    
    // Use first connected grid
    let main_grid_id = &connected_grids[0];
    info!("🎹 Using grid: {}", main_grid_id);
    
    // Clear the grid
    grid.clear_all(main_grid_id)?;
    thread::sleep(Duration::from_millis(100));
    
    // Set up test pattern - light up first column of each row with different brightness
    info!("🎨 Setting up test pattern...");
    for row in 0..7 { // Rows 0-6 (grid coords) = Rows 1-7 (sequencer coords)
        let brightness = (row + 1) * 2; // Brightness 2, 4, 6, 8, 10, 12, 14
        grid.set_led(main_grid_id, 0, row, brightness as u8, "example_caller")?; // Column 0 = first column
        info!("   Row {} (seq row {}): Set LED at (0,{}) to brightness {}", 
              row, row + 1, row, brightness);
    }
    
    grid.refresh()?;
    thread::sleep(Duration::from_millis(500));
    
    info!("");
    info!("📋 Test Pattern Set:");
    info!("   Grid coordinates (0-based) | Sequencer coordinates (1-based) | Brightness");
    info!("   ────────────────────────────┼────────────────────────────────┼──────────");
    for row in 0..7 {
        let brightness = (row + 1) * 2;
        info!("   (0,{})                      | ({},{})                           | {}",
              row, 1, row + 1, brightness);
    }
    
    info!("");
    info!("🎮 Interactive Test Mode");
    info!("Press buttons on different rows and observe behavior:");
    info!("- Row 1 should work normally (known working)");
    info!("- Rows 2-7 may have issues (investigate)");
    info!("- Watch for: LED response, coordinate conversion, timing");
    info!("");
    info!("Press Ctrl+C to exit when done testing...");
    
    // Track button states for each position
    let mut button_states = vec![vec![false; 16]; 7]; // 7 rows x 16 columns
    let mut press_count = 0;
    
    loop {
        // Poll for grid events
        match grid.read_button_events() {
            Ok(events) => {
                for event in events {
                    if event.grid_id == *main_grid_id {
                        press_count += 1;
                        
                        // Convert to sequencer coordinates
                        let seq_x = event.x + 1;
                        let seq_y = event.y + 1;
                        
                        if event.pressed && seq_y <= 7 {
                            info!("");
                            info!("🔥 BUTTON PRESS #{}", press_count);
                            info!("   Grid coords: ({}, {})", event.x, event.y);
                            info!("   Sequencer coords: ({}, {})", seq_x, seq_y);
                            info!("   Row: {} (seq row {})", event.y, seq_y);
                            
                            // Toggle button state
                            if event.x < 16 && event.y < 7 {
                                button_states[event.y][event.x] = !button_states[event.y][event.x];
                                let new_state = button_states[event.y][event.x];
                                
                                info!("   Button state: {} -> {}", !new_state, new_state);
                                
                                // Update LED based on new state
                                let brightness = if new_state { 15 } else { 0 };
                                
                                // Special handling for first column (test pattern)
                                if event.x == 0 {
                                    let pattern_brightness = (event.y + 1) * 2;
                                    let final_brightness = if new_state { 15 } else { pattern_brightness as u8 };
                                    grid.set_led(main_grid_id, event.x, event.y, final_brightness, "example_caller")?;
                                    info!("   LED update: Set ({},{}) to brightness {} (pattern column)", 
                                          event.x, event.y, final_brightness);
                                } else {
                                    grid.set_led(main_grid_id, event.x, event.y, brightness, "example_caller")?;
                                    info!("   LED update: Set ({},{}) to brightness {}", 
                                          event.x, event.y, brightness);
                                }
                                
                                // Analyze behavior per row
                                match seq_y {
                                    1 => info!("   🟢 Row 1: Expected to work correctly"),
                                    2..=7 => {
                                        info!("   🔍 Row {}: Investigate if LED response is immediate", seq_y);
                                        info!("       Check: Visual feedback timing, coordinate handling");
                                    },
                                    _ => warn!("   ⚠️  Unexpected row: {}", seq_y),
                                }
                            } else {
                                warn!("   ⚠️  Button coordinates out of expected range");
                            }
                        } else if !event.pressed {
                            debug!("   Button release at ({}, {}) - ignored", event.x, event.y);
                        } else {
                            info!("   Button press at row {} (control row) - ignored", seq_y);
                        }
                    }
                }
            },
            Err(e) => {
                debug!("Grid poll error (normal): {}", e);
            }
        }
        
        // Small delay to prevent excessive CPU usage
        thread::sleep(Duration::from_millis(1));
    }
}