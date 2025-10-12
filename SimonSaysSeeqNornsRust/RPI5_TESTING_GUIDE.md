# RPi5 Testing Guide

## Overview

This guide explains how to run clock-driven tests on a Raspberry Pi 5 with physical grids connected to verify that MIDI output works correctly when `max_step < 31`.

## Prerequisites

- Raspberry Pi 5 with grids connected
- `serialosc` installed and running
- MIDI ports available (Midi Through or hardware)
- Grids powered on and detected

## Setup

### 1. Verify Grid Connection

On the RPi5, check that grids are detected:

```bash
serialosc-detector
```

You should see your connected grids listed.

### 2. Build the Sequencer and Test Binary

```bash
./rpi5_build_and_run.sh build
```

This builds:
- `simon_says_seeq` (the sequencer)
- `clock_driven_test` (the test runner)

## Running Tests with Physical Hardware

### Terminal 1: Start Sequencer in Test Mode

```bash
./rpi5_build_and_run.sh --test-mode run
```

This will:
- Start the sequencer with `--test-mode` flag
- Auto-load `test_pattern_1.json` at startup
- Disable auto-save to protect test patterns
- Connect to physical grids
- Display startup logs

**Look for these log messages:**
```
🧪 TEST MODE ENABLED - auto-load test_pattern_1.json, no auto-save
🧪 Test mode: Loading test_pattern_1.json
✅ Test pattern loaded successfully
```

**Grid connection messages:**
```
🎛️  GRID ASSIGNMENT:
   GRID_ONE: <grid_id>
   GRID_TWO: <grid_id>
✅ Two real grids ready for operation
```

### Terminal 2: Run Clock-Driven Test

Open a second SSH session or use `tmux`/`screen`:

```bash
cargo run --release --bin clock_driven_test -- --script test3.json
```

The test will:
1. Query sequencer test mode via SysEx
2. Verify test mode is active
3. Send MIDI clock and verify MIDI note output
4. Check that rows with `max_step < 31` trigger correctly

## Expected Test Output

### Test Mode Verification

```
🔧 Initializing sequencer...
  🧪 Querying sequencer test mode...
  ✅ Sequencer is in test mode
  ⏹️  Sending MIDI Stop
  ✅ Test pattern already loaded (test mode auto-loads test_pattern_1.json)
```

### MIDI Output Verification

For Row 3 (max_step=15, notes on steps 0-3):

```
✅ MIDI NoteOn note=51 velocity=100 (any) channel=1 at step=0
✅ MIDI NoteOn note=51 velocity=100 (any) channel=1 at step=1
✅ MIDI NoteOn note=51 velocity=100 (any) channel=1 at step=2
✅ MIDI NoteOn note=51 velocity=100 (any) channel=1 at step=3
...
✅ Row 3 position: 0 (expected 0)  ← Row wraps at step 16
✅ MIDI NoteOn note=51 velocity=100 (any) channel=1 at step=16
✅ MIDI NoteOn note=51 velocity=100 (any) channel=1 at step=17
```

## Verifying the Bug Fix

The original bug (PROBLEM_1.md) was:
> "When setting max_step < 31 for a row, that row's MIDI output is broken (only plays half the time)"

### What to Check

1. **Row 3 triggers continuously**: Should see note 51 at master steps 0,1,2,3,16,17,18,19,32,33...
2. **No "half the time" silence**: Notes should trigger every cycle, not skip cycles
3. **Correct wrapping**: Row should wrap at step 16 (after steps 0-15)

### Visual Verification on Grids

While the test runs, watch the grids:

- **Row 3 LED**: Should light up at steps 0,1,2,3 then wrap
- **Playhead**: Should show continuous movement
- **No gaps**: LEDs should update smoothly, no frozen patterns

## Common Issues

### Test Fails: "Sequencer not in test mode"

**Problem:** Sequencer not started with `--test-mode`

**Solution:**
```bash
# Terminal 1
./rpi5_build_and_run.sh --test-mode run
```

### No Grids Connected Warning

**Problem:** Grids not detected

**Solution:**
1. Check physical connections
2. Verify serialosc is running: `systemctl status serialosc`
3. Restart serialosc: `sudo systemctl restart serialosc`
4. Check grid power

### MIDI Timeout

**Problem:** Test times out waiting for MIDI

**Solution:**
1. Verify MIDI ports: `aconnect -l`
2. Check Midi Through is available
3. Verify both processes are using same MIDI port

### Pattern Not Loading

**Problem:** Sequencer shows wrong pattern

**Solution:**
1. Check `test_pattern_1.json` exists in project root
2. Verify file permissions: `ls -l test_pattern_1.json`
3. Check sequencer logs for error messages

## Test Pattern Details

`test_pattern_1.json` contains:

- **Row 0**: max_step=31 (32 steps), notes at steps 0,4,8,12,16,20,24,28
- **Row 3**: max_step=15 (16 steps), notes at steps 0,1,2,3
- **Row 4**: max_step=14 (15 steps), notes at steps 0,1
- **Row 6**: max_step=4 (5 steps), notes at steps 3,4

This tests various row lengths to verify wrapping behavior.

## Manual Verification (Interactive Testing)

After automated tests pass, you can manually verify on the grids:

1. **Start sequencer in test mode** (Terminal 1)
2. **Don't run automated test** - just observe the sequencer
3. **Press ARM button** (row 7, column 8) on GRID_ONE
4. **Press step column** on any sequence row to set max_step
5. **Watch LEDs update** as the sequence plays
6. **Listen for MIDI** output from connected synthesizers

### Expected Behavior

- Pressing column 15 sets max_step to 15 (16 steps)
- Row should wrap every 16 master steps
- MIDI should trigger continuously (no silence)
- LEDs should show wrapping pattern clearly

## Debugging

### Enable Verbose Logging

```bash
RUST_LOG=debug ./rpi5_build_and_run.sh --test-mode run
```

### Check Sequencer Logs

Look for these specific log messages:

```
🔍 Row 3: master_step=X, row_step=Y, max_step=15, grid[Y][3]=1
```

This shows the sequencer is reading grid values correctly.

### Verify MIDI Output

In the sequencer terminal, you should NOT see:
```
🧪 Test mode: Skipping auto-save on stop (pattern protection enabled)
```
This confirms test mode is protecting patterns.

## Exit Test Mode

To return to normal operation:

1. Stop both processes (Ctrl+C)
2. Restart sequencer WITHOUT `--test-mode`:

```bash
./rpi5_build_and_run.sh run
```

This will:
- Load `current_pattern.json` (your working pattern)
- Enable auto-save on stop
- Return to normal operation

## Related Documentation

- `TEST_MODE_GUIDE.md` - Detailed test mode documentation
- `TESTING_GUIDE.md` - General testing overview
- `PROBLEM_1.md` - Original bug description
- `MIGRATION_TO_CLOCK_DRIVEN_TEST.md` - Test framework details

## Summary

Test mode on RPi5 with grids provides:
- ✅ Real hardware verification
- ✅ Visual LED feedback on grids
- ✅ Protected test patterns (no auto-save)
- ✅ Automated MIDI verification
- ✅ Interactive testing capability

This is the definitive way to verify the max_step bug fix with actual hardware.