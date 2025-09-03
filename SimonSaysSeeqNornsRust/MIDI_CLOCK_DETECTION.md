# MIDI Clock Detection for Raspberry Pi 5 Sequencer

This document describes the MIDI clock detection system for the SimonSaysSeeq sequencer running on Raspberry Pi 5.

## Overview

The MIDI clock detection system automatically scans all available USB MIDI ports and locks onto the first reliable MIDI clock source it finds. This solves the common problem where the sequencer doesn't pick up MIDI clock because it's connected to the wrong port or the port configuration has changed.

## Features

- **Auto-Detection**: Automatically scans all USB MIDI ports for clock signals
- **Multi-Port Support**: Can detect clocks from multiple devices simultaneously
- **Reliability Testing**: Only locks onto sources that provide consistent clock signals
- **Real-Time Monitoring**: Provides live feedback during scanning and monitoring
- **Manual Override**: Allows manual selection of specific ports when needed
- **Pi5 Optimized**: Specifically designed for Raspberry Pi 5 hardware

## Quick Start

### 1. Build and Run the Clock Detector

```bash
cd SimonSaysSeeq/SimonSaysSeeq/SimonSaysSeeqNornsRust
./run_midi_clock_detector.sh
```

This will:
- Build the MIDI clock detector utility
- Scan all available MIDI ports
- Auto-select the first reliable clock source
- Start continuous monitoring

### 2. Interactive Mode (Recommended for First Use)

```bash
./run_midi_clock_detector.sh -i
```

This provides a menu-driven interface where you can:
- See all available MIDI ports
- Choose to auto-scan or manually select a port
- Get real-time feedback during detection

### 3. List Available Ports

```bash
./run_midi_clock_detector.sh -l
```

Shows all MIDI input/output ports detected by the system.

## Command Line Options

```bash
./run_midi_clock_detector.sh [OPTIONS]

Options:
  -h, --help           Show help message
  -l, --list           List MIDI ports and exit
  -i, --interactive    Run in interactive mode
  -s, --scan-only      Only scan, don't monitor continuously
  -p, --port PORT      Monitor specific port by name
  -t, --timeout SECS   Scan timeout in seconds (default: 10)

Examples:
  ./run_midi_clock_detector.sh                    # Auto-scan and monitor
  ./run_midi_clock_detector.sh -i                 # Interactive mode
  ./run_midi_clock_detector.sh -p "USB MIDI"      # Monitor specific port
  ./run_midi_clock_detector.sh -s                 # Scan only
```

## How It Works

### 1. Port Discovery
The system discovers all available MIDI input ports using ALSA on Linux.

### 2. Simultaneous Scanning
It creates temporary connections to all ports simultaneously and listens for MIDI clock messages (0xF8).

### 3. Reliability Assessment
For each port, it tracks:
- Total clock ticks received
- Timing consistency
- Estimated BPM
- Signal stability

### 4. Auto-Selection
The first port that meets reliability criteria (24+ consistent clock ticks) is automatically selected.

### 5. Continuous Monitoring
Once locked onto a source, it provides real-time monitoring with:
- Beat indicators (♪ = quarter note, . = 16th note)
- BPM estimation
- Signal health monitoring

## Understanding the Output

### Scanning Phase
```
🔍 Scanning all MIDI ports for clock signals...
Scanning...........
📊 Scan completed:
  Ports scanned: 3
  Ports with clock: 1
  Reliable sources: 1
✅ Reliable clock sources:
  - USB MIDI Interface MIDI 1
🎯 Auto-selected source: USB MIDI Interface MIDI 1
```

### Monitoring Phase
```
🎵 Listening for MIDI clock (♪ = beat, . = 16th note)...
Press Enter to stop monitoring

♪...♪...♪...♪...♪ [START] ♪...♪...♪...♪...

📊 Stats: 192 ticks, 120.0 BPM, last tick 45ms ago
♪...♪...♪...♪... [STOP]

🛑 Monitoring stopped
📈 Final stats: 384 total ticks, 119.8 average BPM over 32.1s
```

## Integration with Main Sequencer

### Automatic Mode
When the main sequencer starts with an empty MIDI device configuration, it will automatically run the clock detection:

```toml
# In your config.toml
[midi]
device = ""  # Empty string triggers auto-detection
```

### Manual Configuration
After finding your clock source, you can configure it manually:

```toml
[midi]
device = "USB MIDI Interface MIDI 1"  # Exact name from detection
```

## Troubleshooting

### No MIDI Ports Found
```bash
# Check USB devices
lsusb | grep -i midi

# Check ALSA MIDI connections
aconnect -l

# Install ALSA utilities if missing
sudo apt install alsa-utils
```

### Clock Not Detected
1. **Verify Clock Source**: Make sure your device is actually sending MIDI clock
2. **Check Connections**: Verify USB cables and connections
3. **Restart Detection**: USB MIDI devices sometimes need to be reconnected
4. **Manual Selection**: Use `-i` interactive mode to manually select ports

### Permission Issues
```bash
# Add user to audio group
sudo usermod -a -G audio $USER

# Logout and login again, or:
newgrp audio
```

### Build Errors
```bash
# Install required development packages
sudo apt update
sudo apt install build-essential libasound2-dev pkg-config

# Install Rust if missing
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## Technical Details

### MIDI Clock Requirements
- **Signal Type**: MIDI Real-Time Clock (0xF8)
- **Frequency**: 24 pulses per quarter note (24 PPQ)
- **Reliability**: Minimum 24 consecutive ticks for auto-selection
- **Timeout**: 2 seconds without ticks = source considered dead

### Performance
- **Scan Duration**: 3 seconds per port (configurable)
- **CPU Usage**: Minimal during monitoring
- **Memory**: ~1MB for the detector utility
- **Latency**: Sub-millisecond clock detection

### Supported Devices
The scanner works with any USB MIDI device that sends standard MIDI clock, including:
- Arturia BeatStep Pro
- Elektron devices (Digitakt, Octatrack, etc.)
- Roland TR-series
- Novation Circuit/Launchpad
- Generic USB MIDI interfaces

## Files and Components

- `src/midi_scanner/mod.rs` - Core scanning logic
- `src/bin/midi_clock_detector.rs` - Standalone utility
- `run_midi_clock_detector.sh` - Build and run script
- `src/midi.rs` - Integration with main sequencer

## Development

### Building from Source
```bash
cargo build --release --bin midi_clock_detector --features midi
```

### Running Tests
```bash
cargo test midi_scanner
```

### Debug Mode
```bash
RUST_LOG=debug ./target/release/midi_clock_detector
```

## Contributing

When contributing to the MIDI clock detection system:

1. **Test with Real Hardware**: Use actual MIDI devices for testing
2. **Handle Edge Cases**: Consider device disconnection, driver issues, etc.
3. **Maintain Compatibility**: Ensure changes work across different Pi models
4. **Update Documentation**: Keep this README current with changes

## License

This MIDI clock detection system is part of SimonSaysSeeq and is licensed under AGPL-3.0.