# Automated Clock Test System

A comprehensive testing framework that combines MIDI clock generation with formal state logging to enable precise, reproducible testing of the SimonSaysSeeq sequencer.

## Overview

This system solves the challenge of debugging timing-sensitive sequencer issues by providing:
- **Precise MIDI clock control** - Advance sequencer step-by-step with perfect timing
- **Automated button injection** - Simulate button presses at exact moments
- **Formal state logging** - Complete JSON logs of all system events
- **Reproducible test scenarios** - JSON-based test scripts for automated testing

## System Architecture

```
┌─────────────────┐    MIDI Clock    ┌──────────────────────┐
│ Automated Test  │ ───────────────► │ SimonSaysSeeq        │
│ Clock           │                  │ Sequencer            │
│                 │                  │                      │
│ • MIDI timing   │    Button Files  │ • Pattern logic      │
│ • Test scripts  │ ───────────────► │ • ARM actions        │
│ • Button inject │                  │ • LED control        │
└─────────────────┘                  │ • MIDI output        │
                                     └──────────────────────┘
                                               │
                                               ▼
                                     formal_state.log
                                     (Complete event history)
```

## Quick Start

### 1. Build the Components

```bash
cd SimonSaysSeeqNornsRust

# Build main sequencer
cargo build --release

# Build automated test clock
cd utils
cargo build --bin automated_test_clock
cd ..
```

### 2. Start the System

**Terminal 1: Main Sequencer**
```bash
RUST_LOG=info ./target/release/simon-says-seeq-rust
```

**Terminal 2: Automated Test Clock**
```bash
cd utils
RUST_LOG=info ./target/debug/automated_test_clock
```

### 3. Run the 16-Step MIDI Debug Test

In Terminal 2:
```
> create-test
✅ Created sample test script: 16_step_midi_test.json

> script 16_step_midi_test.json
🧪 Executing Test Script: 16-Step MIDI Debug Test
```

## How It Works

### The Button Injection System

The sequencer checks for test button injections in its main event loop:

```rust
// Real hardware events
while let Ok(event) = hw_rx.try_recv() {
    self.handle_hardware_event(event)?;
}

// Test button injections  
let test_events = self.test_injector.check_injections();
for test_event in test_events {
    self.handle_grid_press(&test_event.grid_id, test_event.x, test_event.y, test_event.is_press);
}
```

**Button injection files:**
- `button_a.txt` - Commands for test injection A
- `button_b.txt` - Commands for test injection B

**File formats:**
- Simple: `"15,7"` (column 15, row 7, assumes press on grid_one)
- Full: `"press,grid_two,8,3"` (explicit press/release, grid, coordinates)
- Reset: `"none"` (no injection)

### MIDI Clock Synchronization

The automated test clock sends precise MIDI clock signals:
- **24 PPQN** (Pulses Per Quarter Note) standard
- **6 ticks per step** (24 ÷ 4 = 6 for 16th notes)
- **Perfect timing control** - can advance exact number of steps

### Formal State Logging

Every system event is logged to `formal_state.log` as structured JSON:

```json
{"event_type":"ButtonPress","timestamp":"2025-10-03T18:30:15Z","grid_id":"grid_one","x":8,"y":7,"source":"TestInjectionA"}
{"event_type":"ArmActionExecuted","action":"SetSeqALength","target_row":0,"result":"Changed row 0 length from 32 to 16 steps"}
{"event_type":"StepAdvancement","master_step":20,"row_steps":[[0,4],[1,20],[2,20]]}
{"event_type":"MidiNoteOn","note":60,"source_row":0,"source_step":4}
```

## Test Script Format

Test scripts are JSON files with structured commands:

```json
{
  "name": "16-Step MIDI Debug Test",
  "description": "Reproduces the 16-step MIDI silence issue",
  "initial_bpm": 120.0,
  "commands": [
    {
      "command": "LogMilestone",
      "message": "Starting test"
    },
    {
      "command": "Start"
    },
    {
      "command": "SimpleButton",
      "file": "button_a.txt",
      "x": 0,
      "y": 0
    },
    {
      "command": "WaitSteps",
      "count": 16
    },
    {
      "command": "ArmAction",
      "arm_column": 8,
      "target_row": 0,
      "target_column": 15
    }
  ]
}
```

### Available Commands

| Command | Description | Parameters |
|---------|-------------|------------|
| `Wait` | Pause for milliseconds | `ms: u64` |
| `Start` | Start MIDI clock | None |
| `Stop` | Stop MIDI clock | None |
| `SetBpm` | Change clock speed | `bpm: f32` |
| `SendTicks` | Send exact tick count | `count: u32` |
| `ButtonPress` | Inject button press | `file, grid_id, x, y` |
| `ButtonRelease` | Inject button release | `file, grid_id, x, y` |
| `SimpleButton` | Simple press (assumes grid_one) | `file, x, y` |
| `ArmAction` | ARM button sequence | `arm_column, target_row, target_column` |
| `WaitSteps` | Wait for sequencer steps | `count: u32` |
| `LogMilestone` | Log test milestone | `message: String` |
| `VerifyState` | Verify expected state | `description: String` |

## Built-in Test: 16-Step MIDI Issue

The system includes a comprehensive test for the 16-step MIDI silence bug:

### Test Sequence:
1. **Pattern Creation** - Sets up MIDI triggers at steps 0,4,8,12,16,20,24,28
2. **Normal Operation** - Runs 64 steps, observing full 32-step MIDI behavior
3. **Length Change** - Uses ARM SetSeqALength to set row 0 to 16 steps
4. **Bug Observation** - Continues for 64 steps, documenting changed MIDI behavior

### Expected Results:
- **Before change:** MIDI at master steps 0,4,8,12,16,20,24,28
- **After change:** MIDI timing shifts because row 0 cycles 0-15 while master continues 0-31
- **Root cause:** When master=20, row 0 step=4, which triggers MIDI unexpectedly

## Debugging Workflow

### 1. Reproduce the Issue
```bash
> script 16_step_midi_test.json
```

### 2. Analyze the Logs
```bash
# Watch real-time events
tail -f formal_state.log | jq '.'

# Filter specific event types
jq 'select(.event_type == "MidiNoteOn")' formal_state.log
jq 'select(.event_type == "StepAdvancement")' formal_state.log
```

### 3. Identify the Pattern
Look for:
- **StepAdvancement events** showing master vs row step relationships
- **MidiNoteOn events** and their timing
- **ArmActionExecuted events** confirming state changes

### 4. Develop and Test Fixes
1. Modify sequencer MIDI logic
2. Create new test script to verify fix
3. Run automated test to confirm solution
4. Use formal state logs to validate correct behavior

## Advanced Usage

### Custom Test Scripts

Create your own test scenarios:

```json
{
  "name": "Custom Test",
  "description": "Test specific behavior",
  "commands": [
    {"command": "SetBpm", "bpm": 140.0},
    {"command": "Start"},
    {"command": "SimpleButton", "file": "button_a.txt", "x": 5, "y": 2},
    {"command": "WaitSteps", "count": 8},
    {"command": "LogMilestone", "message": "Checkpoint reached"}
  ]
}
```

### Manual Button Injection

For interactive testing:
```bash
echo "8,7" > button_a.txt     # ARM SetSeqALength
echo "15,0" > button_b.txt    # Row 0, 16 steps
echo "none" > button_a.txt    # Reset
```

### MIDI Clock Commands

In the test clock interface:
```
> start          # Start/stop MIDI clock
> 140            # Set BPM to 140
> ticks 24       # Send exactly 24 MIDI clock ticks
> script my_test.json  # Run custom test script
```

## Simulation Mode (No Hardware)

The system works perfectly without physical grids:
- Main sequencer runs in simulation mode
- LED changes logged as: `💡 SIMULATION: Grid grid_one LED (5,2) = brightness 15`
- All logic functions identically
- Perfect for development and debugging on laptops

## Benefits

### For Developers:
- **Reproducible bugs** - Same test, same results every time
- **Precise timing** - Control down to individual MIDI clock ticks
- **Complete visibility** - Every event logged with timestamps
- **Rapid iteration** - Automated test cycles instead of manual testing

### For QA Testing:
- **Regression testing** - Ensure fixes don't break existing behavior
- **Edge case testing** - Test unusual timing and button sequences
- **Documentation** - Formal state logs serve as test evidence
- **Automation** - Run test suites without human intervention

### For Bug Investigation:
- **Root cause analysis** - See exact sequence of events leading to issues
- **Timing analysis** - Understand relationships between events
- **State correlation** - Connect button presses to MIDI behavior
- **Historical debugging** - Compare logs from working vs broken states

## Troubleshooting

### No MIDI Ports Available
```
Error: No MIDI output ports available
```
**Solution:** Test clock still works for button injection and logging. MIDI timing control won't function, but formal state logging remains fully operational.

### Sequencer Grid Errors
```
HARD REQUIREMENT VIOLATION: Two real grids required
```
**Solution:** Expected in simulation mode. The sequencer will still demonstrate the logging system and button injection.

### File Permission Errors
```
Error: Permission denied writing to button_a.txt
```
**Solution:** Ensure write permissions in the working directory:
```bash
chmod 755 .
touch button_a.txt button_b.txt
chmod 666 button_a.txt button_b.txt
```

## Files Generated

- `formal_state.log` - Complete system event history (JSON lines)
- `test_clock.log` - Test clock specific events
- `button_a.txt` - Test injection file A (auto-managed)
- `button_b.txt` - Test injection file B (auto-managed)
- `16_step_midi_test.json` - Sample test script (created by `create-test`)

## Next Steps

1. **Run the built-in test** to see the system in action
2. **Analyze the formal state logs** to understand event relationships
3. **Create custom test scripts** for specific scenarios
4. **Use the system to develop fixes** for identified issues
5. **Build a test suite** for regression testing

This automated test system transforms sequencer debugging from guesswork into precise, reproducible science! 🧪✨