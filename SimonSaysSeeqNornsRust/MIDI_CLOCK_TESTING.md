# MIDI Clock Generator Test Script Support

The MIDI Clock Generator now supports automated test scripts, allowing you to create reproducible test sequences for your sequencer without manual interaction.

## Overview

Both `midi_clock_generator` and `automated_test_clock` now share the same clock generation code and test script format. This ensures consistent timing and behavior across both tools.

## Quick Start

### Interactive Mode (Default)

```bash
cargo run --bin midi_clock_generator
```

This starts the clock generator in interactive mode where you can manually control it with keyboard commands.

### Test Script Mode

```bash
cargo run --bin midi_clock_generator -- --script test_script_example.json
```

This loads and executes a test script automatically.

### Set Initial BPM

```bash
cargo run --bin midi_clock_generator -- --bpm 140.0
cargo run --bin midi_clock_generator -- --script my_test.json --bpm 140.0
```

## Test Script Format

Test scripts are JSON files with the following structure:

```json
{
  "name": "Test Name",
  "description": "What this test does",
  "initial_bpm": 120.0,
  "commands": [
    {
      "command": "Start"
    },
    {
      "command": "Wait",
      "ms": 2000
    },
    {
      "command": "SetBpm",
      "bpm": 140.0
    },
    {
      "command": "Stop"
    }
  ]
}
```

## Available Commands

### Clock Control

#### Start
Starts the MIDI clock.

```json
{
  "command": "Start"
}
```

#### Stop
Stops the MIDI clock.

```json
{
  "command": "Stop"
}
```

#### SetBpm
Changes the clock tempo.

```json
{
  "command": "SetBpm",
  "bpm": 140.0
}
```

### Timing

#### Wait
Pauses execution for specified milliseconds.

```json
{
  "command": "Wait",
  "ms": 2000
}
```

#### WaitSteps
Waits for a specific number of sequencer steps (calculated from BPM).

```json
{
  "command": "WaitSteps",
  "count": 16
}
```

### Logging

#### LogMilestone
Prints a milestone message during test execution.

```json
{
  "command": "LogMilestone",
  "message": "Starting phase 2 of test"
}
```

### Pattern Management

#### LoadPattern
Loads a test pattern file (copies it to `current_pattern.json`).

```json
{
  "command": "LoadPattern",
  "file": "test_pattern_1.json"
}
```

### Button Injection (automated_test_clock only)

These commands are only supported in `automated_test_clock` which integrates with the sequencer's button injection system:

#### ButtonPress / ButtonRelease
```json
{
  "command": "ButtonPress",
  "file": "button_a.txt",
  "grid_id": "grid_one",
  "x": 5,
  "y": 2
}
```

#### SimpleButton
```json
{
  "command": "SimpleButton",
  "file": "button_a.txt",
  "x": 5,
  "y": 2
}
```

#### ArmAction
```json
{
  "command": "ArmAction",
  "arm_column": 31,
  "target_row": 2,
  "target_column": 8
}
```

## Example Test Scripts

### Basic BPM Test

```json
{
  "name": "BPM Range Test",
  "description": "Test clock at various tempos",
  "initial_bpm": 120.0,
  "commands": [
    {
      "command": "LogMilestone",
      "message": "Testing 120 BPM"
    },
    {
      "command": "Start"
    },
    {
      "command": "WaitSteps",
      "count": 16
    },
    {
      "command": "LogMilestone",
      "message": "Testing 140 BPM"
    },
    {
      "command": "SetBpm",
      "bpm": 140.0
    },
    {
      "command": "WaitSteps",
      "count": 16
    },
    {
      "command": "LogMilestone",
      "message": "Testing 100 BPM"
    },
    {
      "command": "SetBpm",
      "bpm": 100.0
    },
    {
      "command": "WaitSteps",
      "count": 16
    },
    {
      "command": "Stop"
    },
    {
      "command": "LogMilestone",
      "message": "Test complete"
    }
  ]
}
```

### Start/Stop Test

```json
{
  "name": "Start/Stop Stress Test",
  "description": "Rapidly start and stop the clock",
  "initial_bpm": 120.0,
  "commands": [
    {
      "command": "Start"
    },
    {
      "command": "Wait",
      "ms": 1000
    },
    {
      "command": "Stop"
    },
    {
      "command": "Wait",
      "ms": 500
    },
    {
      "command": "Start"
    },
    {
      "command": "Wait",
      "ms": 1000
    },
    {
      "command": "Stop"
    },
    {
      "command": "Wait",
      "ms": 500
    },
    {
      "command": "Start"
    },
    {
      "command": "Wait",
      "ms": 2000
    },
    {
      "command": "Stop"
    }
  ]
}
```

## Command Line Options

### midi_clock_generator

```
USAGE:
    midi_clock_generator [OPTIONS]

OPTIONS:
    -s, --script <FILE>    Execute test script from JSON file
    -b, --bpm <BPM>        Initial BPM (default: 120.0)
    -h, --help             Print help information
```

### Examples

```bash
# Interactive mode at 120 BPM
cargo run --bin midi_clock_generator

# Interactive mode at 140 BPM
cargo run --bin midi_clock_generator -- --bpm 140.0

# Run test script
cargo run --bin midi_clock_generator -- --script my_test.json

# Run test script with custom initial BPM
cargo run --bin midi_clock_generator -- --script my_test.json --bpm 130.0
```

## Differences Between Tools

### midi_clock_generator
- Generates MIDI clock signals only
- Supports basic test commands (Start, Stop, SetBpm, Wait, WaitSteps, LogMilestone)
- Does NOT support button injection commands
- Suitable for testing MIDI clock sync with external sequencers

### automated_test_clock
- Generates MIDI clock signals
- Integrates with SimonSaysSeeq's button injection system
- Supports ALL test commands including button presses
- Logs to `formal_state.log`
- Suitable for automated testing of the entire SimonSaysSeeq system

## Best Practices

1. **Start Simple**: Begin with basic Start/Stop/Wait commands before adding complexity

2. **Use Milestones**: Add `LogMilestone` commands to track progress through your test

3. **Add Delays**: Include small `Wait` commands between operations to allow systems to settle

4. **Test Edge Cases**: Create scripts that test boundary conditions (very fast/slow BPM, rapid start/stop, etc.)

5. **Version Control**: Keep your test scripts in version control alongside your code

6. **Naming Convention**: Use descriptive names like `test_bpm_120_to_140.json` or `test_rapid_start_stop.json`

## Troubleshooting

### "No MIDI output ports available"
- Check that your MIDI interface is connected
- Verify MIDI drivers are installed
- Try `aconnect -l` to list MIDI ports

### Script commands not executing
- Verify JSON syntax is correct
- Check that file paths in `LoadPattern` commands exist
- Ensure BPM values are between 20.0 and 300.0

### Button injection not working
- Button commands only work with `automated_test_clock`
- Verify button injection files (`button_a.txt`, `button_b.txt`) exist
- Ensure sequencer is running and configured for button injection

## See Also

- `test_script_example.json` - Simple example test script
- `MIDI_CLOCK_DETECTION.md` - MIDI clock detection documentation
- `TESTING_README.md` - General testing guide