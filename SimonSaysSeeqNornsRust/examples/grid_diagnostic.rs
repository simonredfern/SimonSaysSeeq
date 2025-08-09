//! Detailed Grid Diagnostic Tool
//! 
//! This tool analyzes the differences between multiple connected grids
//! to understand why one might be working while another isn't.
//! Run with: cargo run --example grid_diagnostic --features desktop

use simon_says_seeq_rust::grid_osc::GridManager;
use log::{info, warn, error, debug};
use env_logger;
use std::time::{Duration, Instant};
use std::thread;
use std::io::{self, Write};
use std::net::UdpSocket;
use rosc::{OscPacket, OscMessage, OscType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with debug level for maximum detail
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    info!("🔬 Comprehensive Grid Diagnostic Tool");
    info!("=====================================");
    info!("");
    info!("This tool will analyze each grid individually to understand");
    info!("why one might work while another doesn't.");
    info!("");
    
    let start_time = Instant::now();
    
    // Step 1: Direct serialosc discovery
    info!("🔍 Step 1: Direct serialosc device discovery...");
    let devices = discover_serialosc_devices()?;
    
    if devices.is_empty() {
        error!("❌ No serialosc devices found");
        print_serialosc_troubleshooting();
        return Ok(());
    }
    
    info!("✅ Found {} device(s) via serialosc:", devices.len());
    for (i, (id, device_type, port)) in devices.iter().enumerate() {
        info!("   {}. {} - {} (port {})", i + 1, id, device_type, port);
    }
    info!("");
    
    // Step 2: Individual device analysis
    for (device_id, device_type, device_port) in &devices {
        info!("🔬 Analyzing device: {} ({})", device_id, device_type);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        match analyze_individual_grid(device_id, device_type, *device_port) {
            Ok(analysis) => {
                print_device_analysis(device_id, &analysis);
                
                // Interactive test
                if prompt_user(&format!("Test grid {} interactively?", device_id)) {
                    interactive_grid_test(device_id, *device_port, &analysis)?;
                }
            }
            Err(e) => {
                error!("❌ Failed to analyze device {}: {}", device_id, e);
            }
        }
        
        info!("");
    }
    
    // Step 3: Comparative analysis
    if devices.len() > 1 {
        info!("🔍 Step 3: Comparative Analysis");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        info!("Comparing {} devices to identify differences...", devices.len());
        
        // Test with GridManager to see which ones work
        match GridManager::new() {
            Ok(mut manager) => {
                let connected_grids = manager.get_connected_grids();
                info!("GridManager successfully connected to {} grid(s)", connected_grids.len());
                
                for grid_id in &connected_grids {
                    if let Some((cols, rows)) = manager.get_dimensions(grid_id) {
                        let name = manager.get_name(grid_id).unwrap_or("Unknown".to_string());
                        info!("  ✅ Working: {} - {} ({}x{})", grid_id, name, cols, rows);
                    }
                }
                
                // Identify non-working grids
                for (device_id, device_type, _) in &devices {
                    if !connected_grids.contains(device_id) {
                        warn!("  ❌ Not working with GridManager: {} - {}", device_id, device_type);
                    }
                }
            }
            Err(e) => {
                error!("Failed to create GridManager for comparison: {}", e);
            }
        }
    }
    
    let total_time = start_time.elapsed();
    info!("");
    info!("🏁 Diagnostic completed in {:.2}s", total_time.as_secs_f32());
    
    Ok(())
}

#[derive(Debug)]
struct DeviceAnalysis {
    responds_to_info: bool,
    size: Option<(usize, usize)>,
    prefix: Option<String>,
    rotation: Option<i32>,
    responds_to_led_commands: bool,
    responds_to_level_commands: bool,
    responds_to_clear: bool,
    communication_delay: Duration,
}

fn discover_serialosc_devices() -> Result<Vec<(String, String, u16)>, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let local_port = socket.local_addr()?.port();
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;
    
    info!("   Created discovery socket on port {}", local_port);
    
    // Send list request
    let list_msg = OscMessage {
        addr: "/serialosc/list".to_string(),
        args: vec![
            OscType::String("127.0.0.1".to_string()),
            OscType::Int(local_port as i32),
        ],
    };
    
    let packet = OscPacket::Message(list_msg);
    let msg_buf = rosc::encoder::encode(&packet)?;
    socket.send_to(&msg_buf, "127.0.0.1:12002")?;
    
    info!("   Sent discovery request to serialosc:12002");
    
    let mut devices = Vec::new();
    let discovery_timeout = Duration::from_secs(3);
    let discovery_start = Instant::now();
    
    while discovery_start.elapsed() < discovery_timeout {
        let mut buf = [0u8; rosc::decoder::MTU];
        match socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                debug!("   Received {} bytes from {}", size, addr);
                if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                    if let OscPacket::Message(msg) = packet {
                        debug!("   OSC message: {} with {} args", msg.addr, msg.args.len());
                        if msg.addr == "/serialosc/device" && msg.args.len() >= 3 {
                            if let (Some(OscType::String(id)), Some(OscType::String(device_type)), Some(OscType::Int(port))) = 
                                (msg.args.get(0), msg.args.get(1), msg.args.get(2)) {
                                info!("   📱 Device: {} ({}) on port {}", id, device_type, port);
                                devices.push((id.clone(), device_type.clone(), *port as u16));
                            }
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                warn!("   Error receiving discovery response: {}", e);
            }
        }
    }
    
    Ok(devices)
}

fn analyze_individual_grid(device_id: &str, device_type: &str, device_port: u16) -> Result<DeviceAnalysis, Box<dyn std::error::Error>> {
    info!("   Connecting to {} on port {}", device_id, device_port);
    
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let local_port = socket.local_addr()?.port();
    socket.set_read_timeout(Some(Duration::from_millis(300)))?;
    
    let device_addr = format!("127.0.0.1:{}", device_port);
    let comm_start = Instant::now();
    
    let mut analysis = DeviceAnalysis {
        responds_to_info: false,
        size: None,
        prefix: None,
        rotation: None,
        responds_to_led_commands: false,
        responds_to_level_commands: false,
        responds_to_clear: false,
        communication_delay: Duration::from_millis(0),
    };
    
    // Test 1: Info request
    info!("   Test 1: Device info request...");
    let info_msg = OscMessage {
        addr: "/sys/info".to_string(),
        args: vec![
            OscType::String("127.0.0.1".to_string()),
            OscType::Int(local_port as i32),
        ],
    };
    
    let packet = OscPacket::Message(info_msg);
    let msg_buf = rosc::encoder::encode(&packet)?;
    socket.send_to(&msg_buf, &device_addr)?;
    
    // Collect info responses
    let info_timeout = Duration::from_millis(1000);
    let info_start = Instant::now();
    
    while info_start.elapsed() < info_timeout {
        let mut buf = [0u8; rosc::decoder::MTU];
        match socket.recv_from(&mut buf) {
            Ok((size, _)) => {
                if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                    if let OscPacket::Message(msg) = packet {
                        analysis.responds_to_info = true;
                        
                        match msg.addr.as_str() {
                            "/sys/size" => {
                                if let (Some(OscType::Int(cols)), Some(OscType::Int(rows))) = 
                                    (msg.args.get(0), msg.args.get(1)) {
                                    analysis.size = Some((*cols as usize, *rows as usize));
                                    debug!("     Size: {}x{}", cols, rows);
                                }
                            }
                            "/sys/prefix" => {
                                if let Some(OscType::String(prefix)) = msg.args.get(0) {
                                    analysis.prefix = Some(prefix.clone());
                                    debug!("     Prefix: {}", prefix);
                                }
                            }
                            "/sys/rotation" => {
                                if let Some(OscType::Int(rotation)) = msg.args.get(0) {
                                    analysis.rotation = Some(*rotation);
                                    debug!("     Rotation: {}°", rotation);
                                }
                            }
                            _ => {
                                debug!("     Other info: {} {:?}", msg.addr, msg.args);
                            }
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => {
                warn!("     Error receiving info: {}", e);
                break;
            }
        }
    }
    
    analysis.communication_delay = comm_start.elapsed();
    
    // Test 2: LED command responsiveness
    info!("   Test 2: LED command tests...");
    
    let prefix = analysis.prefix.as_deref().unwrap_or("/grid");
    
    // Test basic LED command
    let led_msg = OscMessage {
        addr: format!("{}/grid/led/set", prefix),
        args: vec![OscType::Int(0), OscType::Int(0), OscType::Int(1)],
    };
    
    let packet = OscPacket::Message(led_msg);
    let msg_buf = rosc::encoder::encode(&packet)?;
    
    let led_start = Instant::now();
    socket.send_to(&msg_buf, &device_addr)?;
    
    // We can't directly verify LED response via OSC, so we assume success if no error
    analysis.responds_to_led_commands = true;
    thread::sleep(Duration::from_millis(100));
    
    // Test level command (for varibright)
    let level_msg = OscMessage {
        addr: format!("{}/grid/led/level/set", prefix),
        args: vec![OscType::Int(0), OscType::Int(0), OscType::Int(5)],
    };
    
    let packet = OscPacket::Message(level_msg);
    let msg_buf = rosc::encoder::encode(&packet)?;
    socket.send_to(&msg_buf, &device_addr)?;
    
    analysis.responds_to_level_commands = true;
    thread::sleep(Duration::from_millis(100));
    
    // Test clear command
    let clear_msg = OscMessage {
        addr: format!("{}/grid/led/all", prefix),
        args: vec![OscType::Int(0)],
    };
    
    let packet = OscPacket::Message(clear_msg);
    let msg_buf = rosc::encoder::encode(&packet)?;
    socket.send_to(&msg_buf, &device_addr)?;
    
    analysis.responds_to_clear = true;
    
    info!("   Communication tests completed in {:.0}ms", led_start.elapsed().as_millis());
    
    Ok(analysis)
}

fn print_device_analysis(device_id: &str, analysis: &DeviceAnalysis) {
    info!("📊 Analysis Results for {}:", device_id);
    info!("   Info response:        {}", if analysis.responds_to_info { "✅ YES" } else { "❌ NO" });
    
    if let Some((cols, rows)) = analysis.size {
        info!("   Grid size:            ✅ {}x{} ({} LEDs)", cols, rows, cols * rows);
    } else {
        warn!("   Grid size:            ❌ Unknown (using defaults)");
    }
    
    if let Some(prefix) = &analysis.prefix {
        info!("   OSC prefix:           ✅ '{}'", prefix);
    } else {
        warn!("   OSC prefix:           ❌ Unknown (using default '/grid')");
    }
    
    if let Some(rotation) = analysis.rotation {
        info!("   Rotation:             ✅ {}°", rotation);
    } else {
        warn!("   Rotation:             ❌ Unknown");
    }
    
    info!("   LED commands:         {}", if analysis.responds_to_led_commands { "✅ Sent" } else { "❌ Failed" });
    info!("   Level commands:       {}", if analysis.responds_to_level_commands { "✅ Sent" } else { "❌ Failed" });
    info!("   Clear commands:       {}", if analysis.responds_to_clear { "✅ Sent" } else { "❌ Failed" });
    info!("   Communication delay:  {:.0}ms", analysis.communication_delay.as_millis());
    
    // Determine likely issues
    info!("");
    if !analysis.responds_to_info {
        error!("⚠️  CRITICAL: Device not responding to info requests");
        error!("   This suggests the device is not properly connected to serialosc");
    } else if analysis.prefix.is_none() {
        warn!("⚠️  WARNING: No OSC prefix received");
        warn!("   Commands may be sent to wrong OSC address");
    } else if analysis.communication_delay > Duration::from_millis(500) {
        warn!("⚠️  WARNING: Slow communication ({:.0}ms)", analysis.communication_delay.as_millis());
        warn!("   This device may have communication issues");
    } else {
        info!("✅ Device appears to be communicating properly");
    }
}

fn interactive_grid_test(device_id: &str, device_port: u16, analysis: &DeviceAnalysis) -> Result<(), Box<dyn std::error::Error>> {
    info!("");
    info!("🎮 Interactive Test for {}", device_id);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    let device_addr = format!("127.0.0.1:{}", device_port);
    
    let prefix = analysis.prefix.as_deref().unwrap_or("/grid");
    let (cols, rows) = analysis.size.unwrap_or((16, 8));
    
    info!("Using prefix: '{}', size: {}x{}", prefix, cols, rows);
    info!("");
    
    // Test 1: Single LED test
    info!("Test 1: Single LED at (0,0)");
    send_led_command(&socket, &device_addr, prefix, 0, 0, 15)?;
    thread::sleep(Duration::from_millis(500));
    
    if prompt_user("Did you see LED at top-left corner (0,0) turn ON?") {
        info!("✅ LED ON command working for {}", device_id);
    } else {
        warn!("❌ LED ON command not working for {}", device_id);
    }
    
    send_led_command(&socket, &device_addr, prefix, 0, 0, 0)?;
    thread::sleep(Duration::from_millis(300));
    
    if prompt_user("Did the LED at (0,0) turn OFF?") {
        info!("✅ LED OFF command working for {}", device_id);
    } else {
        warn!("❌ LED OFF command not working for {}", device_id);
    }
    
    // Test 2: Corner test
    info!("");
    info!("Test 2: Four corners test");
    let corners = [(0, 0), (cols-1, 0), (0, rows-1), (cols-1, rows-1)];
    
    for &(x, y) in &corners {
        send_led_command(&socket, &device_addr, prefix, x, y, 10)?;
        thread::sleep(Duration::from_millis(150));
    }
    
    if prompt_user("Do you see exactly 4 LEDs lit up at the corners?") {
        info!("✅ Corner pattern working for {}", device_id);
    } else {
        warn!("❌ Corner pattern not working correctly for {}", device_id);
    }
    
    // Clear corners individually
    for &(x, y) in &corners {
        send_led_command(&socket, &device_addr, prefix, x, y, 0)?;
        thread::sleep(Duration::from_millis(100));
    }
    
    // Test 3: Clear all test
    info!("");
    info!("Test 3: Clear all command");
    
    // First light up some LEDs
    for i in 0..8 {
        send_led_command(&socket, &device_addr, prefix, i, 2, 8)?;
    }
    thread::sleep(Duration::from_millis(500));
    
    if prompt_user("Do you see a row of LEDs lit up?") {
        // Test clear all
        send_clear_all_command(&socket, &device_addr, prefix)?;
        thread::sleep(Duration::from_millis(300));
        
        if prompt_user("Did ALL LEDs turn off with the clear command?") {
            info!("✅ Clear all command working for {}", device_id);
        } else {
            warn!("❌ Clear all command not working for {}", device_id);
        }
    } else {
        warn!("❌ Row lighting failed for {}", device_id);
    }
    
    // Final cleanup
    send_clear_all_command(&socket, &device_addr, prefix)?;
    
    Ok(())
}

fn send_led_command(socket: &UdpSocket, device_addr: &str, prefix: &str, x: usize, y: usize, brightness: u8) -> Result<(), Box<dyn std::error::Error>> {
    // Try level command first (for varibright)
    let level_msg = OscMessage {
        addr: format!("{}/grid/led/level/set", prefix),
        args: vec![
            OscType::Int(x as i32),
            OscType::Int(y as i32),
            OscType::Int(brightness as i32),
        ],
    };
    
    let packet = OscPacket::Message(level_msg);
    let msg_buf = rosc::encoder::encode(&packet)?;
    socket.send_to(&msg_buf, device_addr)?;
    
    // Also try basic on/off command as fallback
    let basic_msg = OscMessage {
        addr: format!("{}/grid/led/set", prefix),
        args: vec![
            OscType::Int(x as i32),
            OscType::Int(y as i32),
            OscType::Int(if brightness > 0 { 1 } else { 0 }),
        ],
    };
    
    let packet = OscPacket::Message(basic_msg);
    let msg_buf = rosc::encoder::encode(&packet)?;
    socket.send_to(&msg_buf, device_addr)?;
    
    Ok(())
}

fn send_clear_all_command(socket: &UdpSocket, device_addr: &str, prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Try both clear methods
    let commands = [
        format!("{}/grid/led/all", prefix),
        format!("{}/grid/led/level/all", prefix),
    ];
    
    for addr in &commands {
        let clear_msg = OscMessage {
            addr: addr.clone(),
            args: vec![OscType::Int(0)],
        };
        
        let packet = OscPacket::Message(clear_msg);
        let msg_buf = rosc::encoder::encode(&packet)?;
        socket.send_to(&msg_buf, device_addr)?;
        
        thread::sleep(Duration::from_millis(50));
    }
    
    Ok(())
}

fn prompt_user(question: &str) -> bool {
    print!("   👀 {}: (y/n): ", question);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase().starts_with('y')
}

fn print_serialosc_troubleshooting() {
    error!("");
    error!("🩺 serialosc Troubleshooting:");
    error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    error!("");
    error!("1. Check if serialosc is running:");
    error!("   ps aux | grep serialosc");
    error!("");
    error!("2. Start serialosc if not running:");
    error!("   serialoscd");
    error!("   # Or as service:");
    error!("   sudo systemctl start serialosc");
    error!("");
    error!("3. Check if serialosc server is listening:");
    error!("   netstat -ln | grep 12002");
    error!("   # Should show: udp 127.0.0.1:12002");
    error!("");
    error!("4. Check USB devices:");
    error!("   lsusb | grep -E '(cafe|0a6a)'");
    error!("");
    error!("5. Check permissions:");
    error!("   groups | grep dialout");
    error!("   ls -la /dev/ttyACM*");
    error!("");
    error!("6. If serialosc devices exist but no discovery:");
    error!("   sudo systemctl restart serialosc");
    error!("   # Or kill and restart manually:");
    error!("   pkill serialosc");
    error!("   serialoscd");
}