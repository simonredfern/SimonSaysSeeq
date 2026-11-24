# SimonSaysSeeq Rust

A high-performance sequencer for Norns hardware, written in Rust.

Works ok using external clock and timing.

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
- **CO2 Environmental Data Sonification**: Four CV outputs driven by atmospheric CO2 data

## Grid buttons

SimonSaysSeeq uses two 16x8 grids side by side. On both grids, rows 1 to 7 are the seven sequence rows: steps 1-16 on grid one and steps 17-32 on grid two. Row 8, at the bottom, is the control row. Columns are counted from 1 on the left. "Hold" means hold the control button and press a step on a sequence row; step presses work on either grid.

Positions are counted from MIDI Start. "In time" means a row plays from where it would be had it played its loop since MIDI Start, as after a stop and start.

### Grid one, control row

| Column | Button | What it does |
|---|---|---|
| 1 | Undo | Undo the last change |
| 2 | Redo | Redo the last undone change |
| 3 | Temporary step (hold) | An empty step plays, or a lit step is muted, for the next 2 passes of its row. Press again to cancel. Not saved, not undoable, cleared on stop |
| 4 | Preset pattern (hold) | Apply a preset from the row's own bank. Step 1 clears the row, step 2 is the standard pattern, later steps are denser. A row at 8th-note triplets gets 12-step triplet presets |
| 5 | Euclidean events (hold) | Set the number of events in the row's Euclidean pattern |
| 6 | Euclidean length (hold) | Set the length of the row's Euclidean pattern. The row stays in time |
| 7 | Euclidean rotation (hold) | Rotate the row's Euclidean pattern |
| 8 | Ratchet (hold) | Cycle a step: normal, 2 hits, 3 hits, 4 hits, normal |
| 9 | Last step, straight (hold) | Set the row's last step and make it straight. Clears the row's Jump Loop and row resets. The row stays in time. One undo takes back the whole press |
| 10 | Free | |
| 11 | Row reset every 64 (hold) | Every 64 steps the row goes to the pressed step. With a Jump Loop, the loop carries on in the segment that has that step, or from the first segment. Replaces the row's 64 reset (normal or forced). The button flashes for one beat at each 64-step point |
| 12 | Row reset every 128 (hold) | Every 128 steps the row goes to the pressed step. A row can have a 64 reset and a 128 reset; where they fall together, the 128 one wins. The button flashes for one beat at each 128-step point |
| 13 | Jump Loop 1X (hold) | Press a start step, then an end step, on one row: add a segment played once. End before start plays backwards. Up to 5 segments per row; a 6th drops the oldest |
| 14 | Jump Loop 2X (hold) | Same, segment played twice |
| 15 | Jump Loop 3X (hold) | Same, segment played three times |
| 16 | Row reset every 64, forced (hold) | Like column 11, and replaces the row's 64 reset. On a Jump Loop row with the step outside every segment, the row plays forward from that step until it reaches a step inside a segment, then the loop carries on from there. Flashes at each 64-step point |

Row resets stay until undone or until column 9 (grid one) or column 16 (grid two) is pressed on the row. Pressing the same reset step again keeps it. While a row reset button is held, each row's reset step of that length is the only bright step.

### Grid two, control row

| Column | Button | What it does |
|---|---|---|
| 1 + 2 held | MIDI clock re-detect and Crow search | Release the MIDI clock binding (the next port to send clock is bound) and search once for Crow. Both lights go full brightness if Crow is found, dark if not. Column 1 is also the MIDI clock light: it flashes on each beat while clock is coming in |
| 3 | Jump Loop sync | Each press: off, every 64 steps (dim), every 128 steps (bright), off. At each sync point, rows with a Jump Loop start again at their first segment. Saved and undoable |
| 4 | Jump Loop from step 1 (hold) | Press step N on a row: add a segment from step 1 to step N, played once (step 32 gives a 32-step segment). Undoable |
| 5 | Jump Loop remove newest (hold) | Press any step on a row: remove its newest segment. When the last one goes, the row plays normally again, in time. Undoable |
| 6, 7 | Free | |
| 8-11 | Crow mute 1-4 | On/off: mute Crow output 1, 2, 3 or 4. Lit while muted |
| 12 | Free | |
| 13 | Stop | Stop the sequencer |
| 14 | Start | Start the sequencer from step 1 |
| 15 | Tempo | No effect: the sequencer follows external MIDI clock |
| 16 | Last step, 8th-note triplets (hold) | Set the row's last step and make it 8th-note triplets (12 steps per bar). Clears the row's Jump Loop and row resets, so for triplet Jump Loops set this first, then add the segments. One undo takes back the whole press. On grid two, out of the way of mistaken presses |

While any Jump Loop button is held, steps outside the Jump Loop segments are dimmed. While a last-step button is held, rows at that button's rate look normal and the others are dimmed.

## CO2 CV Outputs

SimonSaysSeeq includes a unique environmental data sonification system that converts atmospheric CO2 measurements into musical control voltages. This system uses real NOAA CO2 data from 1958 to present, creating a powerful tool for climate-aware electronic music.

### CV Output Configuration

**CV1: Step-based CO2 Value (0-10V unipolar)**
- Updates once per musical step (16th notes)
- Represents absolute CO2 levels: 318-800 ppm mapped to 0-10V
- Data source: Historical NOAA atmospheric CO2 measurements
- Use for: Pitch control, filter cutoff, oscillator frequency

**CV2: Tick-based CO2 Value (0-10V unipolar)**
- Updates 6 times per musical step (every MIDI clock pulse)
- Same CO2 range as CV1 but with faster temporal resolution
- Provides rapid modulation based on CO2 progression
- Use for: Fast vibrato, tremolo, high-frequency modulation

**CV3: Daily Change Delta (±5V bipolar)**
- Shows day-to-day change in CO2 levels
- Updates once per musical step
- Positive voltage = CO2 increasing, negative = decreasing
- Auto-scaled to maximum delta found in dataset
- Use for: Dynamic expression, filter resonance, amplitude modulation

**CV4: Seasonal Anomaly Delta (±5V bipolar)**
- Compares current CO2 level vs same day last year
- **Positive voltage = Climate acceleration** (CO2 rising faster than last year)
- **Negative voltage = Climate improvement** (CO2 rising slower than last year)
- **Zero voltage = Same rate as last year**
- Use for: Harmonic tension/release, distortion amount, musical "climate alarm"

### Musical Applications

The CO2 CV system provides multiple time scales for musical expression:
- **Fast modulation** (CV2): 6x per step for rapid sonic changes
- **Melodic changes** (CV1, CV3): Once per step for harmonic/melodic progression
- **Climate narrative** (CV4): Real-time climate change acceleration feedback

### Technical Features

- **Historical Range**: 1958-2025+ NOAA CO2 data (318-800 ppm)
- **Update Timing**: Synchronized to external MIDI clock (24 PPQ)
- **Error Safe**: Graceful handling of missing data or date boundaries  
- **Voltage Scaling**: Automatically scaled to maximize musical resolution
- **Crow mute buttons**: Grid two control row, columns 8-11, mute Crow outputs 1-4 (on/off); see [Grid buttons](#grid-buttons)

### Data Source & Accuracy

- **Source**: NOAA Global Monitoring Laboratory CO2 measurements
- **Resolution**: Daily atmospheric CO2 readings
- **Span**: 65+ years of historical climate data
- **Updates**: Counters advance with musical timing, data progresses through history
- **Reset**: MIDI start/stop resets all CO2 counters to beginning of dataset

### Example Log Output

```
⏱️  CO2 CV timing: 6 ticks per step (1 tick = 1 MIDI clock pulse)
🎛️  CO2 CV Output - Step#1250 Tick#7500: Step:415.2ppm Tick:415.3ppm Δ:0.120ppm/YoY:2.1ppm -> [Step:2.013V, Tick:2.015V, StepΔ:0.245V, SeasonΔ:4.285V]
🎛️  CO2 Tick CV - Tick#7501: 415.4ppm -> Output 2: 2.017V
```

This system transforms decades of climate data into immediate musical expression, creating a unique intersection of environmental awareness and electronic music performance.

## Version Management

SimonSaysSeeq uses a simple VERSION file for manual version tracking:

### VERSION File Format

The `VERSION` file contains a version number and optional description:

```
1.0.0 - Initial CO2 Environmental Data Sonification Release
```

### Version Functions

```rust
use simon_says_seeq_rust::version;

// Get just the version number
let version = version::get_version(); // "1.0.0"

// Get just the description  
let description = version::get_version_description(); // "Initial CO2 Environmental Data..."

// Get the full version line
let full = version::get_full_version(); // "1.0.0 - Initial CO2 Environmental..."

// Get formatted version info
let info = version::get_version_info(); // "SimonSaysSeeq Rust v1.0.0 - Initial CO2..."
```

### Startup Logging

The version appears in startup logs:
```
🎵 SimonSaysSeeq Rust v1.0.0 - Initial CO2 Environmental Data Sonification Release
📦 Cargo Version: v0.1.0 (abc123)
```

### Updating Version

Simply edit the `VERSION` file:
```bash
echo "1.1.0 - Added seasonal anomaly CV4 output with proper scaling" > VERSION
```




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

### Clock-Driven Integration Tests

The `clock_driven_test` runner executes tick-synchronized tests against a running sequencer:

```bash
# Terminal 1: Start sequencer in test mode
cargo run --release --bin simon_says_seeq -- --test-mode

# Terminal 2: Run a test and save output to log file
cargo run --release --bin clock_driven_test -- --script test1.json 2>&1 | tee latest_test_results_test1.log

# Run other tests
cargo run --release --bin clock_driven_test -- --script test2.json 2>&1 | tee latest_test_results_test2.log
cargo run --release --bin clock_driven_test -- --script test3.json 2>&1 | tee latest_test_results_test3.log
```

Available test scripts:
- `test1.json` - Multi-Length Pattern Verification
- `test2.json` - Multi-Row Max Step Changes
- `test3.json` - MIDI Note Output Verification
- `test4.json` - LED Sync Verification
- `test5.json` - Clock Division Reset Verification

The `tee` command writes output to both the terminal and the log file for later review.

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
