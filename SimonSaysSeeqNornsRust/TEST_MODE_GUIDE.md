# Test Mode Guide (Clock-Driven Tests)

## Overview

Test mode is a special operating mode for the SimonSaysSeeq sequencer that:
1. Automatically loads `test_pattern_1.json` on startup
2. Disables auto-save on MIDI Stop (prevents test patterns from being overwritten)
3. Responds to SysEx test mode queries to verify configuration

This ensures tests run with known, reproducible pattern data without corrupting user patterns.

## Starting the Sequencer in Test Mode

```bash
cargo run --release --bin simon_says_seeq -- --test-mode
```

You should see this in the startup logs:
```
🧪 TEST MODE ENABLED - auto-load test_pattern_1.json, no auto-save
```

## Test Mode Verification Protocol

The test framework verifies the sequencer is in test mode before running tests.

### SysEx Query Command

**Test → Sequencer:**
```
F0 7D 53 53 51 10 F7
```
- `F0` = SysEx Start
- `7D 53 53 51` = SimonSaysSeeQ manufacturer ID
- `10` = "Query test mode" command
- `F7` = SysEx End

**Sequencer → Test (Response):**

If test mode is **ACTIVE**:
```
F0 7D 53 53 51 11 F7
```
- `11` = "Test mode is ACTIVE"

If test mode is **NOT ACTIVE**:
```
F0 7D 53 53 51 12 F7
```
- `12` = "Test mode is NOT ACTIVE"

### Test Behavior

If the test receives response `0x12` (not in test mode) or no response within 500ms, the test **fails immediately** with:
```
❌ TEST FAILED: Sequencer is NOT in test mode!
   Please restart the sequencer with: cargo run --release --bin simon_says_seeq -- --test-mode
```

This prevents tests from running with incorrect configuration and potentially corrupting user data.

## Test Mode Differences from Normal Mode

| Feature | Normal Mode | Test Mode |
|---------|-------------|-----------|
| **Pattern Loading** | Loads `current_pattern.json` | Loads `test_pattern_1.json` |
| **Auto-save on Stop** | ✅ Saves to `current_pattern.json` | ❌ No auto-save |
| **User Pattern Protection** | N/A | ✅ User patterns never touched |
| **Test Verification** | N/A | ✅ SysEx query support |

## Running Tests

### Terminal 1: Start Sequencer in Test Mode
```bash
cd SimonSaysSeeqNornsRust
cargo run --release --bin simon_says_seeq -- --test-mode
```

### Terminal 2: Run Clock-Driven Test
```bash
cd SimonSaysSeeqNornsRust
cargo run --release --bin clock_driven_test -- --script test3.json
```

The clock-driven test will automatically:
1. Send SysEx query `0x10` to verify test mode
2. Fail immediately if sequencer is not in test mode
3. Proceed with test execution if verified

## Test Pattern Requirements

Tests assume `test_pattern_1.json` exists in the project root directory with a known pattern structure:

- **Row 0**: Notes at steps 0, 4, 8, 12, 16, 20, 24, 28 (MIDI note 48)
- **Row 3**: Notes at steps 0, 1, 2, 3 with `max_step=15` (MIDI note 51)
- **Row 6**: Different `max_step` for polyrhythm testing (MIDI note 54)

See `test_pattern_1.json` for the complete pattern definition.

## Implementation Details

### Sequencer Code

**New Field in `Sequencer` struct:**
```rust
test_mode: bool,
```

**New Constructor:**
```rust
pub fn new_with_test_mode(test_mode: bool) -> Result<Self>
```

**Modified `stop()` method:**
```rust
// Save current pattern when stopping (unless in test mode)
if !self.test_mode {
    self.save_current_pattern_to_file()?;
}
```

**New Method:**
```rust
pub fn is_test_mode(&self) -> bool {
    self.test_mode
}
```

### Main Application

**Command Line Parsing:**
```rust
let test_mode = args.iter().any(|arg| arg == "--test-mode");
let mut app = SimonSaysSeeq::new_with_test_mode(test_mode)?;
```

**SysEx Handler (0x10 query):**
```rust
0x10 => {
    // Test mode query
    let is_test_mode = self.sequencer.is_test_mode();
    let response_command = if is_test_mode { 0x11 } else { 0x12 };
    let response = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, response_command, 0xF7];
    self.midi.send_sysex(&response)?;
}
```

### Clock-Driven Test Framework

**Test Initialization:**
```rust
// 1. Query test mode
let test_mode_query = vec![0xF0, 0x7D, 0x53, 0x53, 0x51, 0x10, 0xF7];
generator.send_raw_midi(&test_mode_query)?;
thread::sleep(Duration::from_millis(500));

// 2. Verify response
if !generator.check_test_mode_response() {
    return Err("Sequencer not in test mode");
}

// 3. Proceed with test
```

## Troubleshooting

### Test fails with "Sequencer not in test mode"

**Solution:** Restart the sequencer with the `--test-mode` flag:
```bash
cargo run --release --bin simon_says_seeq -- --test-mode
```

### Test pattern not loading correctly

**Verify file exists:**
```bash
ls -l test_pattern_1.json
```

**Check sequencer logs for:**
```
🧪 Test mode: Loading test_pattern_1.json
✅ Test pattern loaded successfully
```

### SysEx response not received

**Check MIDI connections:**
- Sequencer should be connected to "Midi Through" port
- Test should be listening on the same port
- Verify with `aconnect -l` (Linux) or equivalent

## Future Enhancements

Potential improvements to test mode:
1. Support loading arbitrary test pattern files (e.g., `--test-pattern <file>`)
2. Add test mode indicator on screen/LEDs
3. Support multiple test pattern presets
4. Add test mode timeout (auto-exit after N minutes)
5. Record/replay test sequences for regression testing

## Related Documentation

- `TESTING_GUIDE.md` - Overview of testing framework
- `MIGRATION_TO_CLOCK_DRIVEN_TEST.md` - Technical details of clock_driven_test
- `SYSEX_BUTTON_REFERENCE.md` - SysEx command reference
- `test3.json` - Example test script using test mode with clock-driven tests