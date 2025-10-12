# RPi5 Test Quick Start

## Quick Commands

### Terminal 1: Start Sequencer in Test Mode
```bash
cd ~/SimonSaysSeeqNornsRust
./rpi5_build_and_run.sh --test-mode run
```

**Look for:**
```
🧪 TEST MODE ENABLED - auto-load test_pattern_1.json, no auto-save
✅ Test pattern loaded successfully
🎛️  GRID ASSIGNMENT: [shows your grids]
```

### Terminal 2: Run Test
```bash
cd ~/SimonSaysSeeqNornsRust
cargo run --release --bin clock_driven_test -- --script test3.json
```

**Expected:**
```
✅ Sequencer is in test mode
✅ All MIDI note verifications pass
✅ Row 3 wraps correctly at step 16
```

## What This Tests

- Row 3 with max_step=15 (16 steps)
- MIDI output should trigger continuously (not "half the time")
- LEDs on grids should show correct wrapping behavior

## If Test Fails

1. Check sequencer started with `--test-mode`
2. Verify grids are connected: `serialosc-detector`
3. Check MIDI ports: `aconnect -l`

See `RPI5_TESTING_GUIDE.md` for full details.
