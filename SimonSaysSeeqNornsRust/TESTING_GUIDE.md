# Testing Guide: Direct Test Framework

## Overview

The `direct_test` framework provides tick-synchronized testing of the SimonSaysSeeq sequencer using MIDI clock and SysEx injection.

## Prerequisites

⚠️ **CRITICAL**: The main sequencer application MUST be running for tests to work!

The test framework works by:
1. `direct_test` sends MIDI clock and SysEx commands
2. `simon_says_seeq` receives these commands and updates state
3. `simon_says_seeq` writes state changes to `formal_state.log`
4. `direct_test` reads `formal_state.log` to verify state

## Running Tests

### Step 1: Start the Sequencer

In terminal 1:
```bash
cd SimonSaysSeeqNornsRust
cargo run --release --bin simon_says_seeq
```

Wait for it to fully start (you'll see grid connection messages, etc.)

### Step 2: Run the Test

In terminal 2:
```bash
cd SimonSaysSeeqNornsRust
cargo run --release --bin direct_test -- --script test1.json
```

When prompted, select the same MIDI port for both applications (usually "Midi Through").

## Available Tests

### test1.json - Multi-Row Verification
- Tests 7 rows with different max_step values [31, 30, 29, 15, 14, 13, 2]
- Verifies row positions at steps 1, 2, 3, 4, 5
- Tests wrapping behavior at step 32
- Verifies all rows wrap correctly

**Expected Result**: All ✅ (21 verifications pass)

### test2.json - Multi-Row SysEx Configuration Changes
- Changes Row 0: max_step 31→7 (at step 3)
- Changes Row 1: max_step 30→9 (at step 4)
- Changes Row 3: max_step 15→4 (at step 5)
- Button events spread across 3-4 ticks for realistic timing
- Verifies wrapping occurs at new max_step values

**Expected Result**: Multiple verifications showing configuration changes take effect

## Understanding Output

### Successful Verification
```
📍 Tick 6: Verify step=1 row=0 expected=Some(1)
  ✅ Row 0 position: 1 (expected 1)
```

### Failed Verification
```
📍 Tick 192: Verify step=32 row=4 expected=Some(1)
  ❌ Row 4 position: 2 (expected 1)
```

### Sequencer Not Running
```
📍 Tick 6: Verify step=1 row=0 expected=Some(1)
  ⚠️  Step 1 not found in log yet (only 0 steps logged)
```

This means `simon_says_seeq` is not running or not writing to `formal_state.log`.

### SysEx Button Injection
```
📍 Tick 18: SysEx PRESS row=7 col=8
📍 Tick 19: SysEx PRESS row=0 col=7
📍 Tick 20: SysEx RELEASE row=0 col=7
📍 Tick 21: SysEx RELEASE row=7 col=8
```

Shows button press/release sequence spread over 4 ticks (realistic timing).

## Troubleshooting

### "only 0 steps logged"

**Problem**: The sequencer isn't running or isn't writing to the log.

**Solutions**:
1. Start `simon_says_seeq` in another terminal
2. Make sure both apps use the same MIDI port
3. Check that `formal_state.log` is being written to:
   ```bash
   tail -f formal_state.log
   ```

### "Pattern not loading"

**Problem**: `test_pattern_1.json` doesn't exist or is corrupted.

**Solutions**:
1. Verify file exists: `ls -l test_pattern_1.json`
2. Check JSON is valid: `cat test_pattern_1.json | head -20`

### Tests pass individually but fail in sequence

**Problem**: State pollution between tests.

**Solution**: The framework now copies `test_pattern_1.json` to `current_pattern.json` before each test. If issues persist, manually copy:
```bash
cp test_pattern_1.json current_pattern.json
```

## Creating New Tests

### Test Structure

```json
{
  "name": "My Test",
  "description": "What this test does",
  "bpm": 30.0,
  "commands": [
    {
      "at_tick": 6,
      "action": {
        "type": "VerifyState",
        "step": 1,
        "row": 0,
        "expected": 1
      }
    }
  ]
}
```

### Available Actions

**LogMessage**: Print a message
```json
{
  "at_tick": 0,
  "action": {
    "type": "LogMessage",
    "message": "Test starting..."
  }
}
```

**VerifyState**: Check row position
```json
{
  "at_tick": 6,
  "action": {
    "type": "VerifyState",
    "step": 1,
    "row": 0,
    "expected": 1
  }
}
```

**SysExButton**: Press/release button
```json
{
  "at_tick": 18,
  "action": {
    "type": "SysExButton",
    "row": 7,
    "col": 8,
    "press": true
  }
}
```

### Tick Calculation

- 1 step = 6 MIDI clock ticks (at 24 PPQN, 16th notes)
- Step N starts at tick (N * 6)
- Examples:
  - Step 1: tick 6
  - Step 2: tick 12
  - Step 3: tick 18
  - Step 32: tick 192

### Button Press Timing

Spread button events across 3-4 ticks for realistic timing:

```json
// Step 3 (tick 18-23): Change row 0 max_step to 7
{ "at_tick": 18, "action": { "type": "SysExButton", "row": 7, "col": 8, "press": true }},
{ "at_tick": 19, "action": { "type": "SysExButton", "row": 0, "col": 7, "press": true }},
{ "at_tick": 20, "action": { "type": "SysExButton", "row": 0, "col": 7, "press": false }},
{ "at_tick": 21, "action": { "type": "SysExButton", "row": 7, "col": 8, "press": false }}
```

## Best Practices

1. **Always start sequencer first** - Tests won't work without it
2. **Use same MIDI port** - Both apps must communicate
3. **Spread button events** - Don't send press/release in same tick
4. **Verify at known points** - After button presses, at wrap points
5. **Add log messages** - Makes test output easier to understand
6. **Test one thing at a time** - Easier to debug failures

## See Also

- `MIGRATION_TO_DIRECT_TEST.md` - Framework design and migration guide
- `CLEANUP_SUMMARY.md` - What was removed and why
- `test1.json` - Working example of multi-row verification
- `test2.json` - Working example of SysEx configuration changes

## Important Timing Consideration

### Verification Timing

**Problem**: The sequencer writes to `formal_state.log` **during** each step, not at the start.

**Solution**: Verify state in the **middle** of each step, not at the start.

```
Step N timing:
  Tick (N*6)   : Step starts
  Tick (N*6+3) : Middle of step ← Verify here!
  Tick (N*6+5) : End of step
```

**Example**:
```json
// WRONG: Verify at start of step (tick 6)
{"at_tick": 6, "action": {"type": "VerifyState", "step": 1, ...}}
// ⚠️ Step 1 not found in log yet (only 0 steps logged)

// RIGHT: Verify in middle of step (tick 9)
{"at_tick": 9, "action": {"type": "VerifyState", "step": 1, ...}}
// ✅ Row 0 position: 1 (expected 1)
```

### Tick Calculation (Updated)

- Step starts: tick = step × 6
- **Verify at**: tick = (step × 6) + 3
- Examples:
  - Step 1: verify at tick 9 (not 6)
  - Step 2: verify at tick 15 (not 12)
  - Step 3: verify at tick 21 (not 18)
  - Step 32: verify at tick 195 (not 192)

This ensures the sequencer has time to write the step advancement to the log before we try to read it.

### SysEx Button Timing

**Best Practice**: Send SysEx button sequences **between** step boundaries, not at them.

**Why**: Step boundaries are when the sequencer is advancing state and writing to the log. Sending configuration changes at the same time can cause timing conflicts.

**Good Timing** (buttons between steps):
```json
// Step 3 ends at tick 23, Step 4 starts at tick 24
{"at_tick": 22, "action": {"type": "SysExButton", "row": 7, "col": 8, "press": true}},
{"at_tick": 23, "action": {"type": "SysExButton", "row": 0, "col": 7, "press": true}},
{"at_tick": 24, "action": {"type": "SysExButton", "row": 0, "col": 7, "press": false}},
{"at_tick": 25, "action": {"type": "SysExButton", "row": 7, "col": 8, "press": false}}
```

**Bad Timing** (buttons at step boundary):
```json
// Conflicts with step 4 start at tick 24!
{"at_tick": 24, "action": {"type": "SysExButton", "row": 7, "col": 8, "press": true}},
{"at_tick": 25, "action": {"type": "SysExButton", "row": 0, "col": 7, "press": true}},
```

### Summary: Timing Best Practices

1. **Verify in mid-step**: tick = (step × 6) + 3
2. **SysEx between steps**: Start at tick = (step × 6) - 2
3. **Avoid step boundaries** (multiples of 6) for SysEx
4. **Spread button events** across 3-4 ticks
