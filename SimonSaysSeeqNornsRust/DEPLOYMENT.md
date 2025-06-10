# SimonSaysSeeq Rust - Norns Deployment Guide

This guide will help you build, test, and deploy the Rust implementation of SimonSaysSeeq to your Norns device.

## Quick Start

1. **Setup development environment:**
   ```bash
   ./setup_dev_environment.sh
   ```

2. **Test locally (simulation mode):**
   ```bash
   ./test_local.sh
   ```

3. **Deploy to Norns:**
   ```bash
   ./deploy_to_norns.sh
   ```

## Prerequisites

### Development Machine
- **Rust toolchain** (installed automatically by setup script)
- **ARM cross-compiler** (arm-linux-gnueabihf-gcc)
- **System libraries** for audio, HID, and input devices
- **SSH access** to your Norns device

### Norns Device
- **Norns hardware** (any generation) with recent software
- **SSH enabled** (SYSTEM > WIFI > ADD/EDIT > SSH ON)
- **Network connectivity** (WiFi or Ethernet)
- **Optional:** Grid controller for full functionality

## Step-by-Step Setup

### 1. Initial Environment Setup

Run the setup script to install all dependencies:

```bash
./setup_dev_environment.sh
```

This script will:
- Install Rust if not present
- Add ARM cross-compilation targets
- Install system libraries (ALSA, HID, etc.)
- Set up USB permissions (Linux)
- Install development tools

### 2. Local Testing

Before deploying to Norns, test the application locally:

#### Simulation Mode (No Hardware Required)
```bash
./test_local.sh --simulation
```

#### Hardware Mode (Requires Compatible Hardware)
```bash
./test_local.sh --hardware
```

#### Advanced Testing Options
```bash
# Verbose output
./test_local.sh --verbose

# Release build (optimized)
./test_local.sh --release

# Clean build
./test_local.sh --clean
```

### 3. Deploy to Norns

#### Basic Deployment
```bash
./deploy_to_norns.sh
```

#### Custom Norns IP
```bash
NORNS_IP=192.168.1.100 ./deploy_to_norns.sh
```

#### Advanced Environment Variables
```bash
# Custom settings
export NORNS_IP=your.norns.ip
export NORNS_USER=we
export RUST_LOG=debug

./deploy_to_norns.sh
```

## What the Deployment Does

The deployment script performs these steps:

1. **Cross-compiles** the Rust project for ARM Linux
2. **Creates** a deployment package with:
   - Compiled binary (`simon_says_seeq`)
   - Norns Lua wrapper script (`SimonSaysSeeqRust.lua`)
   - Installation script (`install.sh`)
   - Documentation (`README.md`)
3. **Transfers** files to Norns via SSH/SCP
4. **Installs** the application in `/home/we/dust/code/SimonSaysSeeqRust/`
5. **Configures** systemd service for auto-start (optional)

## Using on Norns

### Starting the Application

#### Method 1: From Norns Menu
1. Navigate to **SELECT**
2. Choose **SimonSaysSeeqRust**
3. The Rust process will start automatically

#### Method 2: Manual Start
```bash
# SSH to Norns
ssh we@norns.local

# Navigate to application directory
cd /home/we/dust/code/SimonSaysSeeqRust

# Run directly
./simon_says_seeq
```

### Controls

#### Norns Hardware
- **Key 1:** Restart Rust process
- **Key 3:** Stop/Start toggle
- **Encoders:** Tempo and swing control (handled by Rust)

#### Grid Controller
- **Main Grid:** Step sequencer interface
- **Hold + Press:** Advanced operations (copy, paste, randomize)
- **Control Row (Row 8):** Transport and special functions

### Monitoring and Troubleshooting

#### View Logs
```bash
# SSH to Norns
ssh we@norns.local

# View live logs
journalctl -u simonsaysseeq-rust -f

# View recent logs
journalctl -u simonsaysseeq-rust --since "1 hour ago"
```

#### Check Status
```bash
# Check if service is running
systemctl status simonsaysseeq-rust

# Check process
ps aux | grep simon_says_seeq
```

#### Manual Control
```bash
# Start service
sudo systemctl start simonsaysseeq-rust

# Stop service
sudo systemctl stop simonsaysseeq-rust

# Enable auto-start
sudo systemctl enable simonsaysseeq-rust

# Disable auto-start
sudo systemctl disable simonsaysseeq-rust
```

## Configuration

The application looks for configuration files in:
- `/home/we/.config/simonsaysseeq/config.toml`

Example configuration:
```toml
[sequencer]
default_tempo = 120.0
steps_per_bar = 16
ticks_per_step = 12

[midi]
output_device = "default"
base_channel = 1

[grid]
brightness = 15
auto_connect = true

[co2]
enabled = true
update_interval = 3600
```

## Features Available

### Core Sequencing
- ✅ Real-time step sequencing (7 lanes + control row)
- ✅ Multi-ratchet support (1x, 2x, 4x patterns)
- ✅ Pattern recording and playback
- ✅ Swing/groove control

### Hardware Integration
- ✅ Grid visual feedback with brightness levels
- ✅ Real-time MIDI output (multi-channel)
- ✅ Screen display with status information
- ✅ Encoder control for parameters

### Advanced Features
- ✅ CO2 environmental data integration
- ✅ Tempo stability analysis (wow/flutter detection)
- ✅ Pattern chains and song mode
- ✅ Undo/redo system
- ✅ State persistence

### Interactive Operations
- ✅ Hold + press for copy/paste operations
- ✅ Pattern randomization with density control
- ✅ Live editing during playback
- ✅ Column-wide operations

## Troubleshooting

### Common Issues

#### "Cannot connect to Norns"
- Verify Norns IP address: `ping norns.local`
- Check SSH is enabled on Norns
- Test SSH manually: `ssh we@norns.local`

#### "Cross-compilation failed"
- Run setup script: `./setup_dev_environment.sh`
- Check ARM compiler: `arm-linux-gnueabihf-gcc --version`
- Try clean build: `cargo clean && cargo build --target armv7-unknown-linux-gnueabihf`

#### "Binary not found" on Norns
- Check file permissions: `ls -la /home/we/dust/code/SimonSaysSeeqRust/`
- Make executable: `chmod +x simon_says_seeq`
- Verify architecture: `file simon_says_seeq`

#### No Grid Response
- Check USB connections
- Verify grid permissions (run setup script)
- Check logs for HID errors

#### No MIDI Output
- Verify MIDI connections
- Check ALSA/JACK configuration
- Test with other MIDI software

### Debug Mode

For detailed debugging:

```bash
# On development machine
RUST_LOG=trace ./test_local.sh

# On Norns
RUST_LOG=debug ./simon_says_seeq
```

### Getting Help

1. **Check logs** for error messages
2. **Test locally** in simulation mode first
3. **Verify hardware** with other applications
4. **Check network** connectivity to Norns

## Development Workflow

For ongoing development:

```bash
# Make changes to code
vim src/main.rs

# Test locally
./test_local.sh

# Deploy to Norns
./deploy_to_norns.sh

# Check logs on Norns
ssh we@norns.local journalctl -u simonsaysseeq-rust -f
```

## Performance Notes

### Optimizations
- Release builds use full optimization (`-O3`, LTO)
- Single-threaded compilation for smaller binaries
- Minimal runtime dependencies

### Resource Usage
- **Memory:** ~10-20MB typical usage
- **CPU:** <5% on Norns hardware during normal operation
- **Latency:** Sub-millisecond MIDI timing accuracy

## Security Considerations

- SSH keys recommended over passwords
- Firewall rules may be needed for network access
- USB permissions configured for current user only

---

Happy sequencing! 🎵🦀

For questions or issues, check the logs first and ensure all prerequisites are met.