# Logging Levels Guide

## Log Level Configuration

The sequencer uses Rust's `env_logger` with configurable log levels via the `RUST_LOG` environment variable.

## Default Levels by Component

### INFO Level (Default)
```bash
RUST_LOG=info cargo run --release --bin simon_says_seeq
```

**What you see:**
- Application startup/shutdown
- MIDI port assignments
- Pattern loading/saving
- Test mode status
- Grid connections
- Major state changes
- Row 3 grid value reads (for debugging max_step)

### DEBUG Level
```bash
RUST_LOG=debug cargo run --release --bin simon_says_seeq
```

**Additional output:**
- **MIDI Note ON/OFF messages** (all notes sent)
- Master step advancement
- Row step counters
- Clock tick information
- Detailed sequencer state

### TRACE Level
```bash
RUST_LOG=trace cargo run --release --bin simon_says_seeq
```

**Additional output:**
- **LED updates** (every step position change)
- Grid button press/release details
- Internal state transitions
- Timing details

## Component-Specific Logging

You can set different levels for different modules:

```bash
# INFO for most, DEBUG for MIDI only
RUST_LOG=info,simon_says_seeq::midi=debug cargo run --release

# INFO for most, TRACE for grid updates
RUST_LOG=info,simon_says_seeq=trace cargo run --release

# DEBUG for sequencer, INFO for everything else  
RUST_LOG=info,simon_says_seeq::sequencer=debug cargo run --release
```

## Recommended Settings

### Normal Operation
```bash
RUST_LOG=info
```
Clean output, shows important events only.

### Testing with Clock-Driven Tests
```bash
RUST_LOG=info
```
INFO level is sufficient - MIDI verification happens in the test runner.

### Debugging MIDI Issues
```bash
RUST_LOG=debug
```
Shows all MIDI Note ON/OFF messages.

### Debugging LED/Grid Issues
```bash
RUST_LOG=trace
```
Shows every LED update and grid interaction.

### Debugging max_step Bug
```bash
RUST_LOG=info
```
Row 3 grid reads are already at INFO level for debugging.

## Log Output Examples

### INFO Level (Startup)
```
🧪 TEST MODE ENABLED - auto-load test_pattern_1.json, no auto-save
✅ Test pattern loaded successfully
🎛️  GRID ASSIGNMENT:
   GRID_ONE: monome 12345
   GRID_TWO: monome 67890
🔍 Row 3: master_step=0, row_step=0, max_step=15, grid[0][3]=1
```

### DEBUG Level (MIDI Notes)
```
MIDI Note ON: 48 (C3), vel: 100, ch: 1
MIDI Note ON: 51 (D#3), vel: 100, ch: 1
MIDI Note OFF: 48 (C3), ch: 1
```

### TRACE Level (LED Updates)
```
🔥 LED HANDLER Row 0: Processing LED update old_step=14 -> new_step=15
   Connected grids: 2 grids
🔥 LED HANDLER Row 6: Processing LED update old_step=4 -> new_step=0
   Connected grids: 2 grids
```

## Performance Impact

- **INFO**: Minimal impact
- **DEBUG**: Slight impact (MIDI messages add ~1% overhead)
- **TRACE**: Moderate impact (LED updates can add ~5-10% overhead)

For production/performance testing, use INFO level.
For debugging, use DEBUG or TRACE as needed.

## RPi5 Usage

With the `rpi5_build_and_run.sh` script:

```bash
# INFO level (default)
./rpi5_build_and_run.sh --test-mode run

# DEBUG level
RUST_LOG=debug ./rpi5_build_and_run.sh --test-mode run

# TRACE level
RUST_LOG=trace ./rpi5_build_and_run.sh --test-mode run
```

## Changes Made

Recent logging level adjustments:
- **LED updates**: Moved from INFO → TRACE (reduces noise)
- **MIDI Note ON/OFF**: Standardized at DEBUG (was mixed trace/debug)
- **Row 3 grid reads**: Kept at INFO (needed for max_step debugging)

This provides cleaner default output while keeping debugging information accessible.
