# Migration from automated_test_clock to direct_test

## Summary
The old `automated_test_clock` binary has been replaced with the new `direct_test` binary, which provides superior tick-synchronized testing capabilities.

## What Was Removed
- `src/automated_test_clock.rs` (820+ lines) - Old timing-based test framework
- Binary entries from `Cargo.toml` and `utils/Cargo.toml`

## Why the Change?
The old `automated_test_clock` had fundamental limitations:
- **Timing-based**: Used `Wait` and `WaitSteps` commands with timing delays
- **Race conditions**: Couldn't precisely verify state changes
- **Prediction-based**: Had to predict sequencer state rather than observe it
- **Complex**: Required separate button injection file system

## New direct_test Advantages
- **Tick-synchronized**: Actions execute at exact MIDI clock ticks
- **Real-time verification**: Reads `formal_state.log` to verify actual state
- **Precise**: No timing assumptions or race conditions
- **Integrated**: SysEx commands sent directly via MIDI
- **Visual feedback**: Prints dots for each step

## Test Format Comparison

### Old Format (automated_test_clock)
```json
{
  "commands": [
    {"command": "Start"},
    {"command": "Wait", "ms": 500},
    {"command": "WaitSteps", "count": 3},
    {"command": "Stop"}
  ]
}
```

### New Format (direct_test)
```json
{
  "bpm": 30.0,
  "commands": [
    {
      "at_tick": 18,
      "action": {
        "type": "SysExButton",
        "row": 7,
        "col": 8,
        "press": true
      }
    },
    {
      "at_tick": 24,
      "action": {
        "type": "VerifyState",
        "step": 4,
        "row": 0,
        "expected": 4
      }
    }
  ]
}
```

## Migration Guide
If you have old test scripts:
1. Convert `Wait`/`WaitSteps` to specific `at_tick` values
2. Replace button injection commands with `SysExButton` actions
3. Use `VerifyState` actions instead of `VerifyState` commands
4. Add visual `LogMessage` actions for test progress

## Running Tests
```bash
# Old way (no longer available)
cargo run --release --bin automated_test_clock -- --script test.json

# New way
cargo run --release --bin direct_test -- --script test.json
```

## Available Binaries
After cleanup, the project provides:
- `simon_says_seeq` - Main sequencer application
- `direct_test` - Tick-synchronized testing framework
- `midi_clock_generator` - Interactive MIDI clock tool
- `midi_clock_detector` - MIDI port detection utility

## State Isolation Between Tests

### Problem
Tests were failing when run in sequence because the sequencer auto-saves pattern state (including SysEx configuration changes) to `current_pattern.json` on MIDI Stop. This caused state pollution between tests.

**Example**:
1. Test2 changes max_step from 31 to 7 via SysEx
2. Test2 finishes, sequencer saves the modified pattern
3. Test1 runs and loads the polluted pattern with max_step=7
4. Test1 fails because it expects max_step=31

### Solution
Each test now copies `test_pattern_1.json` to `current_pattern.json` before initialization, ensuring a clean, known-good pattern state for every test run.

```rust
// Copy clean test pattern to current_pattern.json
fs::copy("test_pattern_1.json", "current_pattern.json")?;
```

### Result
✅ Tests can now be run in any order without state pollution
✅ Test1 → Test2 → Test1 all pass
✅ Test2 → Test1 → Test2 all pass
✅ Each test starts with a pristine pattern configuration
