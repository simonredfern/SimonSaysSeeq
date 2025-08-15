# MIDI Clock Generator Utility

A standalone utility program that generates MIDI clock signals and sends them over USB. This tool is designed to test sequencers and other MIDI devices that need external clock synchronization.

## Features

- 🎵 **Precise MIDI Clock Generation**: Sends standard MIDI clock messages (24 PPQ)
- 🎛 **Interactive Controls**: Real-time tempo adjustment and start/stop control
- 📊 **Visual Feedback**: Beat indicators and tempo display
- 🔌 **USB MIDI Output**: Automatic MIDI device detection and connection
- ⚡ **Low Latency**: Optimized for accurate timing
- 🛡 **Safe Operation**: Proper MIDI start/stop/continue message handling

## Requirements

- Rust toolchain (install from [rustup.rs](https://rustup.rs/))
- USB MIDI interface or device
- Linux/macOS/Windows (cross-platform)

## Installation and Usage

### Quick Start

```bash
cd utils

# Build and run (debug)
cargo run --bin midi_clock_generator

# Build and run (optimized - recommended)
cargo run --release --bin midi_clock_generator
```

### First Run

1. Enter desired BPM (default: 120)
2. Select MIDI output device from the list
3. Use commands to control the clock

## Commands

| Command | Description |
|---------|-------------|
| `s`, `start` | Start/stop the MIDI clock |
| `+` | Increase BPM by 5 |
| `-` | Decrease BPM by 5 |
| `++` | Increase BPM by 1 |
| `--` | Decrease BPM by 1 |
| `<number>` | Set specific BPM (e.g., `140`) |
| `status` | Show current status and BPM |
| `h`, `help` | Show help |
| `q`, `quit` | Exit program |

## Visual Indicators

When the clock is running, you'll see beat indicators:
- `♩` - Beat 1 (downbeat)
- `♪` - Beat 2  
- `♫` - Beat 3
- `♬` - Beat 4

Example output:
```
♩♪♫♬ | 120.0 BPM
♩♪♫♬ | 120.0 BPM
```

## MIDI Messages

The utility sends standard MIDI clock messages:
- `0xFA` - MIDI Start
- `0xFC` - MIDI Stop  
- `0xFB` - MIDI Continue (when resuming)
- `0xF8` - MIDI Clock (24 per quarter note)

## Testing Your Sequencer

1. **Start the clock generator** with desired BPM
2. **Connect USB MIDI** to your sequencer device
3. **Configure your sequencer** to receive external MIDI clock
4. **Start the clock** with `s` command
5. **Adjust tempo** in real-time with `+`/`-` commands

## Technical Details

- **Clock Resolution**: 24 pulses per quarter note (24 PPQ)
- **BPM Range**: 20.0 - 300.0 BPM
- **Timing Accuracy**: Sub-millisecond precision
- **Threading**: Separate thread for clock generation to ensure timing accuracy

## Troubleshooting

### No MIDI Devices Found
- Check USB MIDI interface is connected
- Verify device drivers are installed
- Try unplugging and reconnecting MIDI device

### Clock Not Received by Sequencer
- Ensure sequencer is set to "External Clock" mode
- Check MIDI connections and cables
- Verify correct MIDI device is selected

### Timing Issues
- Close other audio/MIDI applications
- Use a dedicated USB MIDI interface for best results
- Check system audio settings and buffer sizes

## Development

The utility is built with:
- **midir**: Cross-platform MIDI I/O
- **ctrlc**: Graceful shutdown handling
- **std::thread**: Multi-threaded clock generation

### Building from Source

```bash
git clone <repository>
cd SimonSaysSeeqNornsRust/utils

# Build
cargo build --release

# Run
cargo run --release --bin midi_clock_generator

# Run benchmark tool
cargo run --release --bin midi_clock_benchmark
```

### Running Tests

```bash
cargo test
```

## License

This utility is part of the SimonSaysSeeq project and is licensed under AGPL-3.0.

---

**Happy sequencing!** 🎵