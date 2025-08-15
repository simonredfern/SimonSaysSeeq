# MIDI Clock Generator Testing Guide

This guide shows how to use the MIDI Clock Generator to test your sequencer and other MIDI devices.

## Quick Start Testing

### 1. Basic Clock Test

```bash
# Start the clock generator
cd utils
cargo run --release --bin midi_clock_generator

# Follow prompts:
# Enter BPM: 120
# Select MIDI device: 0 (or your preferred device)

# Commands to try:
s        # Start the clock
status   # Check if it's running
140      # Change to 140 BPM
+        # Increase by 5 BPM (now 145)
--       # Decrease by 1 BPM (now 144)
s        # Stop the clock
q        # Quit
```

### 2. Testing with SimonSaysSeeq

1. **Start the clock generator** in one terminal:
   ```bash
   cd utils
   cargo run --release --bin midi_clock_generator
   # Set BPM to 120, start the clock
   ```

2. **Run your sequencer** in another terminal:
   ```bash
   cd ..  # Back to main project
   ./test_local.sh
   ```

3. **Configure sequencer** to receive external clock:
   - The sequencer should detect the external MIDI clock automatically
   - Look for log messages like "External MIDI clock detected"
   - Tempo controls on the grid should be disabled when external clock is active

## Testing Scenarios

### Scenario 1: Basic Clock Sync

**Purpose**: Verify sequencer follows external clock

**Steps**:
1. Start clock generator at 100 BPM
2. Start sequencer
3. Create a simple pattern in the sequencer
4. Start both clock and sequencer
5. Change clock BPM to 150
6. Verify sequencer speed changes accordingly

**Expected Result**: Sequencer playback speed matches clock generator BPM

### Scenario 2: Start/Stop Sync

**Purpose**: Test transport control sync

**Steps**:
1. Start clock generator
2. Start sequencer with a pattern
3. Use clock generator `s` command to start/stop
4. Verify sequencer starts/stops in sync

**Expected Result**: Sequencer transport follows clock start/stop commands

### Scenario 3: Tempo Range Testing

**Purpose**: Test extreme tempo ranges

**Steps**:
1. Test minimum BPM (20): `20`
2. Test maximum BPM (300): `300`
3. Test rapid tempo changes: `++`, `--`, `+`, `-`

**Expected Result**: Clock remains stable at all tempos, sequencer follows

### Scenario 4: Real-time Tempo Changes

**Purpose**: Test smooth tempo transitions during playback

**Steps**:
1. Start at 120 BPM with pattern playing
2. Gradually increase: `+`, `+`, `+` (135 BPM)
3. Make fine adjustments: `++`, `++`, `--` (136 BPM)
4. Jump to different tempo: `100`

**Expected Result**: No audio dropouts, smooth tempo transitions

## Integration Testing

### With Hardware MIDI Devices

```bash
# Connect USB MIDI interface to hardware
# Start clock generator
cargo run --release --bin midi_clock_generator

# Select your USB MIDI interface
# Hardware device should sync to the clock
```

### With DAW Software

1. **Setup**: Connect virtual MIDI cable (e.g., loopMIDI on Windows, IAC on Mac)
2. **Clock Generator**: Select virtual MIDI port
3. **DAW**: Set to receive external MIDI clock from same virtual port
4. **Test**: Start clock, verify DAW follows tempo

### Multiple Device Testing

Test with multiple devices receiving the same clock:

```
Clock Generator → USB MIDI Hub → Device 1
                               → Device 2  
                               → Device 3
```

All devices should stay in perfect sync.

## Performance Testing

### Timing Accuracy Test

Use an oscilloscope or audio interface to measure:
- Clock jitter (should be < 1ms)
- Start/stop latency
- Tempo change response time

### CPU Usage Test

```bash
# Monitor CPU while running
top -p $(pgrep midi_clock_generator)

# Test with different BPMs:
# - 60 BPM (low frequency)
# - 180 BPM (high frequency)  
# - 300 BPM (maximum)
```

Expected CPU usage: < 1%

### Long Duration Test

```bash
# Run for extended period
# Start at 9 AM
cargo run --release --bin midi_clock_generator
120     # Set BPM
s       # Start clock

# Leave running until 5 PM
# Monitor for:
# - Timing drift
# - Memory leaks
# - System stability
```

## Common Issues and Solutions

### Issue: "No MIDI output ports available"

**Solutions**:
- Check USB MIDI device connection
- Verify device drivers installed
- Try different USB port
- Restart audio system: `pulseaudio --kill && pulseaudio --start`

### Issue: Clock not received by target device

**Solutions**:
- Verify MIDI connections
- Check device is set to "External Clock" mode
- Test with different MIDI device
- Use MIDI monitor to verify messages: `aseqdump -p 14:0`

### Issue: Timing instability

**Solutions**:
- Close other audio applications
- Use dedicated USB MIDI interface
- Increase audio buffer size in system settings
- Check for USB power issues

### Issue: Sequencer doesn't follow tempo changes

**Solutions**:
- Some devices only respond to tempo on clock start
- Stop and restart clock after tempo change
- Check if device supports real-time tempo changes

## Advanced Testing

### MIDI Message Validation

Use a MIDI monitor to verify correct messages:

```bash
# Linux - monitor MIDI port
aseqdump -p 14:0

# Expected messages:
# FA (Start)
# F8 F8 F8... (Clock ticks, 24 per beat)
# FC (Stop)
```

### Custom Test Patterns

Create test sequences that highlight timing issues:

1. **Metronome Pattern**: Quarter notes only
2. **Subdivision Test**: 16th note patterns  
3. **Polyrhythm Test**: 3 against 4 patterns
4. **Sparse Pattern**: Notes on beat 1 and 3 only

### Latency Measurement

```bash
# Use jack_delay or similar tools
jack_delay -I system:midi_capture_1 -O system:midi_playback_1

# Measure round-trip latency:
# Clock Generator → Target Device → Audio Output → Measurement
```

## Automation Scripts

### Automated Tempo Sweep Test

```bash
#!/bin/bash
# automated_tempo_test.sh

echo "Starting automated tempo test..."
for bpm in 60 80 100 120 140 160 180 200; do
    echo "Testing ${bpm} BPM for 10 seconds..."
    echo "$bpm" | timeout 10 cargo run --release --bin midi_clock_generator
    sleep 2
done
echo "Tempo sweep test complete"
```

### Stress Test

```bash
#!/bin/bash
# stress_test.sh

echo "Starting stress test - rapid tempo changes..."
(
    echo "120"  # Initial BPM
    echo "s"    # Start
    
    for i in {1..100}; do
        echo "++"   # Increment
        sleep 0.1
    done
    
    echo "s"    # Stop
    echo "q"    # Quit
) | cargo run --release --bin midi_clock_generator
```

## Results Documentation

Keep a test log with:

```
Date: 2024-01-15
Test: Basic Clock Sync
BPM Range: 60-180
Duration: 30 minutes
Result: ✅ PASS
Notes: Perfect sync, no dropouts

Date: 2024-01-15  
Test: Multiple Device Sync
Devices: 3x hardware synths
Result: ⚠️ PARTIAL
Notes: Device #3 occasional timing drift
```

## Integration with CI/CD

Add automated tests to your build pipeline:

```yaml
# .github/workflows/midi_test.yml
name: MIDI Clock Tests
on: [push, pull_request]

jobs:
  midi-tests:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Build Clock Generator
      run: cd utils && cargo build --release
    - name: Run Unit Tests
      run: cd utils && cargo test
    # Note: Hardware MIDI tests require special runners
```

Remember: Hardware MIDI testing requires physical devices and cannot be fully automated in CI/CD environments.