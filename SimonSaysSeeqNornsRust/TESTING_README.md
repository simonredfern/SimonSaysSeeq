# Testing README

Simple guide for running and testing SimonSaysSeeq.

## Running the Sequencer

### Basic Run (Desktop/Development)
```bash
cargo run --bin simon_says_seeq --features desktop
```

### Run on Norns Hardware
```bash
cargo run --bin simon_says_seeq --features norns
```

### Run Without Hardware (No Grid Required)
```bash
cargo run --bin simon_says_seeq
```

### Run with Different Log Levels
```bash
# Info level (default)
RUST_LOG=info cargo run --bin simon_says_seeq

# Debug level (more verbose)
RUST_LOG=debug cargo run --bin simon_says_seeq

# Warn level (less verbose)
RUST_LOG=warn cargo run --bin simon_says_seeq

# Trace level (most verbose)
RUST_LOG=trace cargo run --bin simon_says_seeq
```

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Specific Test
```bash
cargo test test_sequencer_creation
```

### Run Tests for Specific Module
```bash
cargo test sequencer::
```

## Running Examples

### Show JSON Structure
```bash
cargo run --example show_json_structure
```

### Single Grid Test
```bash
cargo run --example single_grid_test --features desktop
```

## Running Automated Tests

**Important**: Run these from the project root directory (`SimonSaysSeeqNornsRust`), not from `utils/`

**MIDI Connection Required**: The automated test clock sends MIDI messages to the sequencer. Both programs must be running simultaneously and connected via MIDI:

**Terminal 1 - Start the sequencer:**
```bash
cd SimonSaysSeeqNornsRust
RUST_LOG=info cargo run --bin simon_says_seeq
```

**Terminal 2 - Run the test (sends MIDI clock to Terminal 1):**
```bash
cd SimonSaysSeeqNornsRust  # Make sure you're in project root
cargo run --bin automated_test_clock -- --test1
```

The test clock sends MIDI clock ticks, and the sequencer advances steps when it receives them. You should see `Steps: [R0:1, R1:2, ...]` logs in Terminal 1 as the sequencer advances.

### Run Test with test_pattern_1.json (Standalone)
```bash
cd SimonSaysSeeqNornsRust  # Make sure you're in project root
cargo run --bin automated_test_clock -- --test1
```
This only loads the pattern file but doesn't advance the sequencer.

### Run Custom Test Script
```bash
cargo run --bin automated_test_clock -- --script path/to/test.json
```

### Other Automated Test Tools
```bash
# MIDI clock detector
cargo run --bin midi_clock_detector

# CO2 tests
cargo run --bin test_co2
cargo run --bin test_co2_cv
```

## Quick Reference

| Command | Purpose |
|---------|---------|
| `cargo run --bin simon_says_seeq` | Run sequencer (no hardware) |
| `cargo run --bin simon_says_seeq --features desktop` | Run with grid support |
| `RUST_LOG=debug cargo run --bin simon_says_seeq` | Run with debug logging |
| `cargo test` | Run all tests |
| `cargo test -- --nocapture` | Run tests with debug output |
| `cargo run --bin automated_test_clock -- --test1` | Run automated test (from project root) |
| `cargo build --release --bin simon_says_seeq` | Build optimized binary |

## Notes

- Grid support requires `serialosc` running
- Desktop features enable grid OSC communication
- Tests run without requiring hardware
- Use `--nocapture` to see `println!` and debug output in tests
- Log levels: `error`, `warn`, `info`, `debug`, `trace` (from least to most verbose)
- Set log level with `RUST_LOG=level` environment variable
- Automated tests require running both sequencer and test clock simultaneously
- The test clock sends MIDI to the sequencer to advance steps