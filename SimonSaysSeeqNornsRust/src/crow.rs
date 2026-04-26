//! USB Serial Crow module for CV output
//!
//! Writes are queued onto a bounded crossbeam channel and drained by a
//! dedicated `crow-serial` worker thread. Callers (the audio/main thread)
//! never block on USB serial I/O, so a slow or disconnected Crow can no
//! longer stall the sequencer.
//!
//! Backpressure policy is "drop oldest": if the queue is full when a new
//! command arrives, the oldest queued command is discarded to make room.
//! CV is a continuous signal, so dropping a stale value in favour of the
//! newest is the right trade-off for live performance.

use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::process::Command;
use std::time::Duration;

#[cfg(feature = "hardware")]
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
#[cfg(feature = "hardware")]
use serialport::SerialPort;
#[cfg(feature = "hardware")]
use std::io::Write;
#[cfg(feature = "hardware")]
use std::thread::{self, JoinHandle};

/// At ~80 commands/sec under normal play (24 ticks + 4×step writes), 64
/// slots is roughly 0.8 s of buffering before drop-oldest engages.
#[cfg(feature = "hardware")]
const CROW_CHANNEL_CAPACITY: usize = 64;

pub struct Crow {
    enabled: bool,
    #[cfg(feature = "hardware")]
    sender: Option<Sender<String>>,
    #[cfg(feature = "hardware")]
    drain_rx: Option<Receiver<String>>,
    #[cfg(feature = "hardware")]
    worker: Option<JoinHandle<()>>,
}

impl Crow {
    /// Create a new Crow interface
    pub fn new() -> Result<Self> {
        Ok(Self {
            enabled: false,
            #[cfg(feature = "hardware")]
            sender: None,
            #[cfg(feature = "hardware")]
            drain_rx: None,
            #[cfg(feature = "hardware")]
            worker: None,
        })
    }

    /// Initialize Crow by finding and opening the USB serial port
    pub fn initialize(&mut self) -> Result<()> {
        #[cfg(feature = "hardware")]
        {
            for attempt in 1..=3 {
                if attempt > 1 {
                    thread::sleep(Duration::from_secs(attempt as u64));
                }

                if self.try_initialize_once().is_ok() {
                    return Ok(());
                }
            }

            return Err(anyhow!("Failed to find Crow USB serial device after retries"));
        }

        #[cfg(not(feature = "hardware"))]
        {
            return Ok(());
        }
    }

    /// Single initialization attempt
    #[cfg(feature = "hardware")]
    fn try_initialize_once(&mut self) -> Result<()> {
        // Log available USB devices for debugging
        info!("Available USB serial devices:");
        if let Ok(entries) = std::fs::read_dir("/dev") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if let Some(name_str) = name.to_str() {
                    if name_str.starts_with("ttyACM") || name_str.starts_with("ttyUSB") {
                        info!("  Found: /dev/{}", name_str);
                    }
                }
            }
        }

        // Identify Crow strictly by its USB vendor/product ID (0483:5740).
        // We deliberately do NOT fall back to "open whatever ttyACM*/ttyUSB*
        // happens to be available" — on a Pi with monome grids (or any other
        // USB-CDC peripheral) that fallback would grab the wrong port,
        // happily report success, then spam `Crow serial write failed`
        // warnings at every CO2 CV tick (especially after a hot-unplug
        // invalidates the fd).
        let crow_path = match self.find_crow_device()? {
            Some(p) => p,
            None => return Err(anyhow!("No Crow device found by USB ID (0483:5740)")),
        };

        let port = serialport::new(&crow_path, 115_200)
            .timeout(Duration::from_millis(1000))
            .open()
            .map_err(|e| anyhow!("Failed to open identified Crow device {}: {}", crow_path, e))?;

        self.enable_with_port(port);
        Ok(())
    }

    /// Take ownership of an opened port, spawn the writer thread, and queue
    /// the initial 0 V outputs.
    #[cfg(feature = "hardware")]
    fn enable_with_port(&mut self, port: Box<dyn SerialPort>) {
        self.spawn_worker(port);
        self.enabled = true;

        // Initialize all outputs to 0V (fire-and-forget through the worker)
        let _ = self.send_command("output[1].volts = 0");
        let _ = self.send_command("output[2].volts = 0");
        let _ = self.send_command("output[3].volts = 0");
        let _ = self.send_command("output[4].volts = 0");
    }

    /// Spawn the dedicated serial-writer thread.
    #[cfg(feature = "hardware")]
    fn spawn_worker(&mut self, mut port: Box<dyn SerialPort>) {
        let (tx, rx) = bounded::<String>(CROW_CHANNEL_CAPACITY);
        let drain_rx = rx.clone();
        let worker = thread::Builder::new()
            .name("crow-serial".into())
            .spawn(move || {
                // Rate-limit serial-I/O failure warnings. Without throttling,
                // a real disconnect mid-show would emit one warning per tick
                // (~48/s at 120 BPM × 24 PPQ) and flood journald. We log
                // first failures immediately so the operator sees them, then
                // suppress and summarise inside a 1 s window.
                let warn_throttle = Duration::from_secs(1);
                let mut last_warn: Option<std::time::Instant> = None;
                let mut suppressed_since_warn: u32 = 0;

                while let Ok(cmd) = rx.recv() {
                    let result = port
                        .write_all(cmd.as_bytes())
                        .and_then(|_| port.flush());
                    match result {
                        Ok(_) => {
                            // Reset counter so the next failure logs immediately.
                            suppressed_since_warn = 0;
                        }
                        Err(e) => {
                            let now = std::time::Instant::now();
                            let should_log = match last_warn {
                                None => true,
                                Some(t) => now.duration_since(t) >= warn_throttle,
                            };
                            if should_log {
                                if suppressed_since_warn > 0 {
                                    warn!(
                                        "Crow serial I/O failed: {} ({} more suppressed in the last second)",
                                        e, suppressed_since_warn
                                    );
                                } else {
                                    warn!("Crow serial I/O failed: {}", e);
                                }
                                last_warn = Some(now);
                                suppressed_since_warn = 0;
                            } else {
                                suppressed_since_warn += 1;
                            }
                        }
                    }
                }
                debug!("crow-serial worker exiting (channel disconnected)");
            })
            .expect("spawn crow-serial worker thread");

        self.sender = Some(tx);
        self.drain_rx = Some(drain_rx);
        self.worker = Some(worker);
    }

    /// Check if Crow is enabled and ready
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set all 4 CV outputs to specified voltages
    pub fn set_all_outputs(&mut self, v1: f32, v2: f32, v3: f32, v4: f32) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Clamp voltages to Crow's safe range (-5V to +10V)
        let v1 = v1.clamp(-5.0, 10.0);
        let v2 = v2.clamp(-5.0, 10.0);
        let v3 = v3.clamp(-5.0, 10.0);
        let v4 = v4.clamp(-5.0, 10.0);

        self.send_command(&format!("output[1].volts = {:.6}", v1))?;
        self.send_command(&format!("output[2].volts = {:.6}", v2))?;
        self.send_command(&format!("output[3].volts = {:.6}", v3))?;
        self.send_command(&format!("output[4].volts = {:.6}", v4))?;

        Ok(())
    }

    /// Queue a raw Lua command for the Crow. Non-blocking; if the queue is
    /// full, the oldest pending command is dropped to make room.
    pub fn send_command(&mut self, lua_code: &str) -> Result<()> {
        #[cfg(feature = "hardware")]
        {
            let Some(sender) = self.sender.as_ref() else {
                return Ok(());
            };
            let drain_rx = self.drain_rx.as_ref();

            let mut payload = format!("{}\n", lua_code);
            loop {
                match sender.try_send(payload) {
                    Ok(()) => return Ok(()),
                    Err(TrySendError::Full(returned)) => {
                        payload = returned;
                        // Drop oldest queued command to make room for the new one.
                        // The worker may also be popping concurrently; in the worst
                        // case we discard one extra command — acceptable for CV.
                        match drain_rx {
                            Some(rx) => {
                                let _ = rx.try_recv();
                            }
                            None => return Ok(()),
                        }
                    }
                    Err(TrySendError::Disconnected(_)) => return Ok(()),
                }
            }
        }

        #[cfg(not(feature = "hardware"))]
        {
            let _ = lua_code;
            Ok(())
        }
    }

    /// Find Crow device by USB vendor/product ID
    #[cfg(feature = "hardware")]
    fn find_crow_device(&self) -> Result<Option<String>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg("ls /dev/ttyACM* /dev/ttyUSB* 2>/dev/null || true")
            .output()?;

        let devices_str = String::from_utf8_lossy(&output.stdout);
        let devices: Vec<&str> = devices_str.trim().lines().filter(|s| !s.is_empty()).collect();

        for device in devices {
            // Check if this device has the Crow USB IDs (STMicroelectronics Virtual COM Port)
            let udev_output = Command::new("udevadm")
                .args(&["info", "--query=all", &format!("--name={}", device)])
                .output()?;

            let udev_str = String::from_utf8_lossy(&udev_output.stdout);

            // Look for STMicroelectronics Virtual COM Port (0483:5740)
            if udev_str.contains("ID_VENDOR_ID=0483") && udev_str.contains("ID_MODEL_ID=5740") {
                return Ok(Some(device.to_string()));
            }
        }

        Ok(None)
    }

}

impl Drop for Crow {
    fn drop(&mut self) {
        // Best-effort: zero outputs on shutdown. These go through the worker
        // queue; if the process is exiting the worker may not drain them,
        // but Crow also zeros itself when its USB host disconnects.
        if self.enabled {
            let _ = self.send_command("output[1].volts = 0");
            let _ = self.send_command("output[2].volts = 0");
            let _ = self.send_command("output[3].volts = 0");
            let _ = self.send_command("output[4].volts = 0");
        }
        // Default field drop order is fine: dropping `sender` makes the
        // worker's `recv` return Disconnected, ending the worker loop.
    }
}
