# SimonSaysSeeq Rust

A high-performance sequencer for Norns hardware, written in Rust.

## Raspberry Pi 5 Build and Run Script

For Raspberry Pi 5 users, use the optimized build script:

```bash
# Quick start - setup and run
./rpi5_build_and_run.sh setup
./rpi5_build_and_run.sh run

# Or all-in-one
./rpi5_build_and_run.sh
```

### rpi5_build_and_run.sh Usage

```
USAGE:
    ./rpi5_build_and_run.sh [OPTIONS] [COMMAND] [-- APPLICATION_ARGS]

COMMANDS:
    setup       Install system dependencies and configure environment
    build       Build the project with current configuration
    run         Run the application (builds first if necessary)
    test        Test the compiled binary
    service     Install and manage systemd service
                Actions: install, start, stop, status, enable, disable
    info        Show system information
    clean       Clean build artifacts
    help        Show this help

OPTIONS:
    --release           Build in release mode (default)
    --debug             Build in debug mode
    --features FEATURES Specify cargo features (default: hardware,midi,desktop)
    --update-rust       Update Rust toolchain before building
    -h, --help          Show help

EXAMPLES:
    ./rpi5_build_and_run.sh setup                           # Install dependencies and setup environment
    ./rpi5_build_and_run.sh build                          # Build in release mode
    ./rpi5_build_and_run.sh --debug build                  # Build in debug mode
    ./rpi5_build_and_run.sh run                            # Build and run
    ./rpi5_build_and_run.sh run -- --simulation            # Run with simulation mode
    ./rpi5_build_and_run.sh service install                # Install systemd service (stops if running)
    ./rpi5_build_and_run.sh service start                  # Start the service
    ./rpi5_build_and_run.sh service stop                   # Stop the service
    ./rpi5_build_and_run.sh service status                 # Check service status
    ./rpi5_build_and_run.sh service enable                 # Enable autostart on boot
    ./rpi5_build_and_run.sh service disable                # Disable autostart
    ./rpi5_build_and_run.sh --features "hardware,midi" run # Build and run with specific features
```



### Manual Options

#### Simulation Mode (Development/Testing)

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

#### Hardware Mode (Norns)

**Recommended: Use the getting started script**
```bash
./getting_started.sh
```

**Manual deployment options:**

```bash
# Build for hardware
cargo build --release --features="hardware,midi"

# Deploy Pure Rust (recommended - uploads source & compiles on Norns)
./deploy_pure_rust_simple.sh

# OR deploy with Lua wrapper (traditional)
./deploy_to_norns.sh

# OR advanced Pure Rust (cross-compile locally - requires ARM toolchain)
./deploy_pure_rust.sh
```

#### Deployment Options

**Pure Rust Mode - Simple** (recommended):
- Uploads source code and compiles natively on Norns
- No cross-compilation issues or ARM toolchain setup
- Boots directly into sequencer (no Norns menu)
- Eliminates Lua interpreter overhead
- Full hardware support guaranteed

**Traditional Mode**:
- Uses Norns menu system
- Select `SimonSaysSeeqRust.lua` from SELECT menu
- Lua script launches Rust binary
- Better for mixed-use Norns systems

**Pure Rust Mode - Advanced**:
- Cross-compiles locally (requires ARM toolchain setup)
- Faster deployment but complex setup
- See CROSS_COMPILATION_ISSUES.md for troubleshooting

## Features

- **7 Row step sequencer**: Written in Rust
- **Current Pattern Saved**: Current Pattern is saved on MIDI stop and loaded at seqeuncer boot.



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

### Timing Architecture

The sequencer uses **External Clock Slave Mode Only** for maximum stability and reliability:

**External Clock Slave Mode:**
```
MIDI Clock Input (24 PPQ)
         ↓
   ClockTick Event
         ↓
handle_midi_input_event()
         ↓
  Counter % 6 == 0 ?  ←── (16th note timing)
         ↓ YES
external_advance_step()
         ↓
   advance_step()
         ↓
  Trigger MIDI + LEDs
```

**Key Benefits:**
- **Basic sync** with external devices
- **Preserves external swing** timing perfectly
- **Rock solid stability** - no complex internal timing
- **Simple architecture** - fewer bugs, easier maintenance
- **Professional workflow** - matches hardware sequencer behavior

### Code Structure

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


# To check midi is being sent on linux

See the ports

$ aseqdump -l
 Port    Client name                      Port name
  0:0    System                           Timer
  0:1    System                           Announce
 14:0    Midi Through                     Midi Through Port-0
128:0    SimonSaysSeeq                    SimonSaysSeeq Output

then use the port thus:

aseqdump -p 128:0 | awk 'function nn(n, i,o,a){split("C C# D D# E F F# G G# A A# B",a," "); i=n%12; o=int(n/12)-1; return a[i+1] "" o} /Note on/ {match($0,/note[ =]*([0-9]+)/,n); match($0,/velocity[ =]*([0-9]+)/,v); if(n[1]!="") {cmd="date +%s%3N"; cmd | getline ts; close(cmd); printf "%s Note ON  %-4s Vel=%-3s\n", ts, nn(n[1]), v[1]}} /Note off/ {match($0,/note[ =]*([0-9]+)/,n); if(n[1]!="") {cmd="date +%s%3N"; cmd | getline ts; close(cmd); printf "%s Note OFF %-4s\n", ts, nn(n[1])}}'
