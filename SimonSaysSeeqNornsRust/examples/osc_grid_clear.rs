//! OSC Grid Clear Utility using serialosc
//! 
//! This utility uses proper OSC communication through serialosc to clear the grid.
//! This is the correct way to communicate with modern monome grids.
//! Run with: cargo run --example osc_grid_clear --features desktop

use std::net::UdpSocket;
use std::time::{Duration, Instant};
use std::thread;
use log::{info, warn, error};
use rosc::{OscPacket, OscMessage, OscType};
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("🎹 OSC Grid Clear Utility (using serialosc)");
    info!("===========================================");
    info!("");
    
    // Create UDP socket for OSC communication
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let local_port = socket.local_addr()?.port();
    socket.set_read_timeout(Some(Duration::from_millis(1000)))?;
    
    info!("📡 Created OSC socket on port {}", local_port);
    
    // Discover serialosc devices
    info!("🔍 Discovering serialosc devices...");
    
    let list_msg = OscMessage {
        addr: "/serialosc/list".to_string(),
        args: vec![
            OscType::String("127.0.0.1".to_string()),
            OscType::Int(local_port as i32),
        ],
    };
    
    let packet = OscPacket::Message(list_msg);
    let msg_buf = rosc::encoder::encode(&packet)?;
    
    // Send list request to serialosc server
    socket.send_to(&msg_buf, "127.0.0.1:12002")?;
    info!("📤 Sent device discovery request to serialosc");
    
    // Wait for device responses
    let mut devices = Vec::new();
    let discovery_timeout = Duration::from_secs(2);
    let discovery_start = Instant::now();
    
    while discovery_start.elapsed() < discovery_timeout {
        let mut buf = [0u8; rosc::decoder::MTU];
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                    if let OscPacket::Message(msg) = packet {
                        if msg.addr == "/serialosc/device" && msg.args.len() >= 3 {
                            if let (Some(OscType::String(id)), Some(OscType::String(device_type)), Some(OscType::Int(port))) = 
                                (msg.args.get(0), msg.args.get(1), msg.args.get(2)) {
                                info!("✅ Found device: {} (type: {}) on port {}", id, device_type, port);
                                devices.push((id.clone(), device_type.clone(), *port as u16));
                            }
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // Timeout, continue waiting
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                warn!("Error receiving OSC message: {}", e);
            }
        }
    }
    
    if devices.is_empty() {
        error!("❌ No serialosc devices found!");
        error!("💡 Make sure:");
        error!("   1. Your grid is connected via USB");
        error!("   2. serialosc is running: sudo systemctl start serialosc");
        error!("   3. You're in the dialout group");
        return Ok(());
    }
    
    info!("");
    info!("🎹 Found {} grid device(s)", devices.len());
    
    // Clear each device
    for (device_id, device_type, device_port) in devices {
        info!("");
        info!("🧹 Clearing device: {} ({})", device_id, device_type);
        
        match clear_grid_via_osc(&socket, device_port, &device_id) {
            Ok(_) => info!("✅ Successfully cleared grid: {}", device_id),
            Err(e) => error!("❌ Failed to clear grid {}: {}", device_id, e),
        }
    }
    
    info!("");
    info!("🏁 OSC grid clear completed");
    info!("💡 All LEDs should now be OFF using proper OSC communication");
    
    Ok(())
}

fn clear_grid_via_osc(socket: &UdpSocket, device_port: u16, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let device_addr = format!("127.0.0.1:{}", device_port);
    info!("   Connecting to device at {}", device_addr);
    
    // Method 1: Use /grid/led/all to clear all LEDs at once
    info!("   Method 1: Clearing all LEDs with /grid/led/all");
    
    let clear_all_msg = OscMessage {
        addr: "/grid/led/all".to_string(),
        args: vec![OscType::Int(0)], // 0 = off
    };
    
    let packet = OscPacket::Message(clear_all_msg);
    let msg_buf = rosc::encoder::encode(&packet)?;
    socket.send_to(&msg_buf, &device_addr)?;
    
    thread::sleep(Duration::from_millis(200));
    
    // Method 2: Use /grid/led/level/all for variable brightness grids
    info!("   Method 2: Clearing with brightness level /grid/led/level/all");
    
    let clear_level_msg = OscMessage {
        addr: "/grid/led/level/all".to_string(),
        args: vec![OscType::Int(0)], // 0 = off
    };
    
    let packet = OscPacket::Message(clear_level_msg);
    let msg_buf = rosc::encoder::encode(&packet)?;
    socket.send_to(&msg_buf, &device_addr)?;
    
    thread::sleep(Duration::from_millis(200));
    
    // Method 3: Individual LED clear for first few positions (safety check)
    info!("   Method 3: Clearing individual LEDs (first 16 positions)");
    
    for i in 0..16 {
        let x = i % 16;
        let y = i / 16;
        
        // Clear with basic LED command
        let led_msg = OscMessage {
            addr: "/grid/led/set".to_string(),
            args: vec![
                OscType::Int(x as i32),
                OscType::Int(y as i32),
                OscType::Int(0), // 0 = off
            ],
        };
        
        let packet = OscPacket::Message(led_msg);
        let msg_buf = rosc::encoder::encode(&packet)?;
        socket.send_to(&msg_buf, &device_addr)?;
        
        // Also try level command
        let level_msg = OscMessage {
            addr: "/grid/led/level/set".to_string(),
            args: vec![
                OscType::Int(x as i32),
                OscType::Int(y as i32),
                OscType::Int(0), // 0 = off
            ],
        };
        
        let packet = OscPacket::Message(level_msg);
        let msg_buf = rosc::encoder::encode(&packet)?;
        socket.send_to(&msg_buf, &device_addr)?;
        
        if i % 4 == 0 {
            thread::sleep(Duration::from_millis(10));
        }
    }
    
    // Method 4: Final comprehensive clear
    info!("   Method 4: Final comprehensive clear");
    
    // Send multiple clear commands
    for _ in 0..3 {
        let clear_all_msg = OscMessage {
            addr: "/grid/led/all".to_string(),
            args: vec![OscType::Int(0)],
        };
        
        let packet = OscPacket::Message(clear_all_msg);
        let msg_buf = rosc::encoder::encode(&packet)?;
        socket.send_to(&msg_buf, &device_addr)?;
        
        thread::sleep(Duration::from_millis(100));
        
        let clear_level_msg = OscMessage {
            addr: "/grid/led/level/all".to_string(),
            args: vec![OscType::Int(0)],
        };
        
        let packet = OscPacket::Message(clear_level_msg);
        let msg_buf = rosc::encoder::encode(&packet)?;
        socket.send_to(&msg_buf, &device_addr)?;
        
        thread::sleep(Duration::from_millis(100));
    }
    
    info!("   ✅ OSC clear sequence completed for {}", device_id);
    
    Ok(())
}