# SimonSaysSeeq Rust

A high-performance sequencer for Norns hardware, written in Rust.

## Quick Start

### Simulation Mode (Development/Testing)

For development and testing without hardware:

```bash
# Build and run in simulation mode
cargo build --no-default-features --features="simulation"
./target/debug/simon_says_seeq

# Or use the provided test script
./test_local.sh --simulation
```

### Framework RGB Macropad Mode (Hardware)

For use with Framework Laptop 16 RGB Macropad:

```bash
# Build with hardware support
cargo build --features="hardware"

# Test RGB Macropad connection and rainbow flash
./target/debug/simon_says_seeq --test-macropad

# Run with RGB Macropad support
./target/debug/simon_says_seeq
```

In simulation mode, you'll see sequencer activity in the console:

```
🎵 Sequencer: STOPPED | Tempo: 120.0 BPM | Step: 1 | Bar: 1
space
🌈 Starting Framework RGB Macropad flash sequence!
💡 Flash button 1 (0,0) - Step 1/16
💡 Flash button 2 (1,0) - Step 2/16
... (flash sequence continues)
✨ Macropad flash sequence complete!
🎵 Sequencer: RUNNING | Tempo: 120.0 BPM | Step: 1 | Bar: 1
🎹 MIDI Note ON: 60 vel:100 ch:1
🔥 Macropad button Q PRESSED: (0,1)
🎵 Sequencer: RUNNING | Tempo: 120.0 BPM | Step: 2 | Bar: 1
space
🎵 Sequencer: STOPPED | Tempo: 120.0 BPM | Step: 1 | Bar: 1
```

### Hardware Mode (Norns)

For deployment on actual Norns hardware:

```bash
# Build for hardware
cargo build --release --features="hardware,midi"

# Or deploy directly to Norns
./deploy_to_norns.sh
```

## Features

- **High-Performance Sequencing**: Written in Rust for maximum performance
- **Grid Support**: Monome grid integration for tactile control
- **Framework RGB Macropad**: Native support for Framework Laptop 16 RGB Macropad
- **MIDI I/O**: Full MIDI input/output support
- **Real-time Display**: Visual feedback on Norns screen
- **RGB Flash Sequences**: Dynamic LED animations on sequencer start
- **Simulation Mode**: Test without hardware using console output
- **Pattern Management**: Save/load and chain patterns
- **CO2 Integration**: Environmental data integration for tempo modulation

## Development

### Building

Different feature sets are available:

- `simulation`: Console-only mode for development
- `hardware`: Full hardware support (encoders, buttons, screen, Framework RGB Macropad)
- `midi`: MIDI input/output support
- `desktop`: Development with grid on laptop

### Testing

```bash
# Run tests
cargo test --features="simulation"

# Test with hardware features (requires actual hardware)
cargo test --features="hardware,midi"
```

### Simulation Mode Details

When running in simulation mode, the application provides rich console feedback:

- **🎵 Sequencer Status**: Shows running state, tempo, step, and bar
- **🎹 MIDI Events**: Note on/off events with velocity and channel
- **💡 Grid LEDs**: Visual representation of grid button states
- **🕐 Transport**: Clock start/stop events

The Framework RGB Macropad simulation displays a 4x4 grid representation:

```

## Framework RGB Macropad Support

### Hardware Detection

The application automatically detects Framework RGB Macropad devices:

```bash
# Test macropad detection and RGB functionality
./target/debug/simon_says_seeq --test-macropad

# Expected output:
🌈 Framework RGB Macropad Test Mode
✅ Framework RGB Macropad found!
🚀 Starting rainbow flash test...
💡 Flash button (0,0) with color RGB(255, 0, 0) - Step 1/16
💡 Flash button (1,0) with color RGB(255, 127, 0) - Step 2/16
...
✨ Test complete!
```

### USB Device Information

Framework RGB Macropad typically appears as:
- **Vendor ID**: `32AC` (Framework Computer Inc.)
- **Product Name**: Framework RGB Macropad
- **Layout**: 4x4 button grid with individual RGB LEDs

### RGB Features

- **Individual LED Control**: Each of the 16 buttons has independent RGB control
- **Rainbow Flash Sequence**: Automatic RGB animation on sequencer start
- **Real-time Feedback**: LEDs respond to button presses and sequencer activity
- **Color Coding**: Different colors can represent different sequencer states

### Troubleshooting

If the RGB Macropad doesn't light up:

1. **Check USB Connection**: Ensure the macropad is properly connected
2. **Verify Detection**: Run `--test-macropad` to see if device is found
3. **Check Permissions**: On Linux, you may need udev rules for HID access
4. **Build with Hardware Features**: Ensure you built with `--features="hardware"`
5. **View Device List**: Application logs show all detected USB HID devices

```
🌈 Framework RGB Macropad State (brightness 0-15):
┌─────┬─────┬─────┬─────┐
│ ■ 15│ □ 0 │ □ 0 │ ■ 8 │
│ □ 0 │ ■ 15│ □ 0 │ □ 0 │
│ □ 0 │ □ 0 │ ■ 12│ □ 0 │
│ ■ 4 │ □ 0 │ □ 0 │ □ 0 │
└─────┴─────┴─────┴─────┘
Framework RGB Macropad mapping:
  1 2 3 4
  Q W E R
  A S D F
  Z X C V
```

### Log Levels

Control output verbosity with `RUST_LOG`:

```bash
# Minimal output
RUST_LOG=warn ./target/debug/simon_says_seeq

# Default (shows sequencer and MIDI activity)
RUST_LOG=info ./target/debug/simon_says_seeq

# Verbose debugging
RUST_LOG=debug ./target/debug/simon_says_seeq

# Everything
RUST_LOG=trace ./target/debug/simon_says_seeq
```

## Controls

### Grid (Hardware Mode)
- **Grid Pads**: Toggle sequence steps
- **Hold + Grid**: Advanced operations (copy, paste, etc.)

### Framework RGB Macropad (Hardware Mode)
- **16 RGB Buttons**: 4x4 grid for step sequencing and control
- **LED Feedback**: Real-time visual feedback for active steps
- **RGB Flash**: Rainbow animation on sequencer start
- **Press Detection**: Hardware button press detection

### Framework RGB Macropad Simulation (Simulation Mode)
- **1-4, QWER, ASDF, ZXCV**: Simulate 4x4 macropad button presses
- **Space + Enter**: Run/stop sequencer (triggers RGB flash sequence on start)
- **Enter**: Execute button press
- **p + Enter**: Quit

### Encoders (Hardware Mode)
- **E1**: Tempo
- **E2**: Swing
- **E3**: Pattern selection

### Buttons (Hardware Mode)
- **K1**: Menu/shift
- **K2**: Play/stop
- **K3**: Pattern functions

## Configuration

Configuration file: `~/.config/simonsaysseeq/config.toml`

Example configuration:

```toml
[sequencer]
default_tempo = 120.0
steps_per_bar = 16
ticks_per_step = 12

[midi]
output_device = "default"
input_device = "default"
base_channel = 1

[grid]
brightness = 15
auto_connect = true

[screen]
fps = 30
brightness = 80
```

## Architecture

- **main.rs**: Application entry point and main loop
- **sequencer.rs**: Core sequencing engine
- **grid.rs**: Monome grid interface
- **midi.rs**: MIDI input/output handling
- **screen.rs**: Display management
- **hardware.rs**: Hardware abstraction layer
- **config.rs**: Configuration management
- **co2.rs**: Environmental data integration

## License

AGPL-3.0 - See LICENSE file for details

## Contributing

1. Fork the repository
2. Create a feature branch
3. Test in simulation mode: `./test_local.sh --simulation`
4. Submit a pull request

For hardware testing, ensure you have access to:
- Norns or compatible hardware
- Monome grid (optional)  
- Framework Laptop 16 RGB Macropad (optional)
- MIDI devices (optional)

### Framework RGB Macropad Setup

1. **Connect Hardware**: Plug in Framework RGB Macropad via USB
2. **Test Detection**: Run `./target/debug/simon_says_seeq --test-macropad`
3. **Check Permissions**: Ensure your user has HID device access
4. **Verify RGB**: Look for rainbow flash during test mode

### Linux udev Rules (if needed)

Create `/etc/udev/rules.d/50-framework-macropad.rules`:
```
# Framework RGB Macropad
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="32ac", MODE="0666", GROUP="plugdev"
```

Then reload udev rules:
```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```