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

In simulation mode, you'll see sequencer activity in the console:

```
🎵 Sequencer: STOPPED | Tempo: 120.0 BPM | Step: 1 | Bar: 1
r
🎵 Sequencer: RUNNING | Tempo: 120.0 BPM | Step: 1 | Bar: 1
🎹 MIDI Note ON: 60 vel:100 ch:1
💡 Grid LED: (0,0) brightness 15
🎹 MIDI Note OFF: 60 ch:1
🎵 Sequencer: RUNNING | Tempo: 120.0 BPM | Step: 2 | Bar: 1
r
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
- **MIDI I/O**: Full MIDI input/output support
- **Real-time Display**: Visual feedback on Norns screen
- **Simulation Mode**: Test without hardware using console output
- **Pattern Management**: Save/load and chain patterns
- **CO2 Integration**: Environmental data integration for tempo modulation

## Development

### Building

Different feature sets are available:

- `simulation`: Console-only mode for development
- `hardware`: Full hardware support (encoders, buttons, screen)
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

The grid simulation displays a 3x4 grid representation:

```
Grid State (brightness 0-15):
┌─────┬─────┬─────┐
│ ■ 15│ □ 0 │ □ 0 │
│ □ 0 │ ■ 8 │ □ 0 │
│ □ 0 │ □ 0 │ ■ 15│
│ □ 0 │ □ 0 │ □ 0 │
└─────┴─────┴─────┘
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

### Grid Simulation (Simulation Mode)
- **Numpad 1-9, 0**: Simulate grid button presses
- **r + Enter**: Run/stop sequencer
- **Enter**: Execute button press
- **q + Enter**: Quit

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
- MIDI devices (optional)