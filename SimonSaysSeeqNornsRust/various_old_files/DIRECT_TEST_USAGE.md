# Direct Test Usage

## What is Direct Testing?

Direct testing runs test actions **inside the clock generation loop**, synchronized perfectly with MIDI clock ticks. This eliminates all timing assumptions and race conditions.

## Running Direct Tests

```bash
# Make sure sequencer is running first
cargo run --release --bin simon_says_seeq

# In another terminal, run the direct test
cargo run --release --bin direct_test -- --script test2.json
```

## Test Scripts

### test1.json - Multi-Length Pattern Verification
- Verifies rows with different max_step values
- Checks wrapping behavior at step boundaries
- Tick-synchronized observations

### test2.json - Single Row Max Step Change  
- Changes Row 0 from max_step=31 to max_step=7
- SysEx sent at ticks 18-21
- Verifies at ticks 24, 30, 36, 42, 48, 54
- Will show EXACT tick when change takes effect

## Test Script Format

```json
{
  "name": "Test Name",
  "description": "What this test does",
  "bpm": 30.0,
  "commands": [
    {
      "at_tick": 0,
      "action": {
        "type": "LogMessage",
        "message": "Test starting"
      }
    },
    {
      "at_tick": 18,
      "action": {
        "type": "SysExButton",
        "row": 0,
        "col": 7,
        "press": true
      }
    },
    {
      "at_tick": 24,
      "action": {
        "type": "VerifyState",
        "step": 4,
        "row": 0,
        "expected": null
      }
    }
  ]
}
```

## Action Types

### LogMessage
```json
{
  "type": "LogMessage",
  "message": "Your message here"
}
```

### SysExButton
```json
{
  "type": "SysExButton",
  "row": 0,
  "col": 7,
  "press": true
}
```

### RecordConfig
```json
{
  "type": "RecordConfig",
  "row": 0,
  "max_step": 7
}
```

### VerifyState
```json
{
  "type": "VerifyState",
  "step": 4,
  "row": 0,
  "expected": 4
}
```
Use `"expected": null` for observation-only (no pass/fail).

## Advantages Over Old Testing

| Feature | Old Test | Direct Test |
|---------|----------|-------------|
| Synchronization | Approximate | Perfect |
| Timing | Uses `Wait` and `WaitSteps` | Uses exact tick counts |
| SysEx Timing | Uncertain | Exact tick known |
| Verification | Reads log after waiting | Reads log at exact tick |
| Reproducibility | Varies with system load | Deterministic |
| Debugging | Hard to trace timing | Exact tick sequence visible |

## Understanding Ticks

At 30 BPM:
- 24 ticks per quarter note (PPQN = 24)
- 6 ticks per 16th note step
- Each tick = ~83ms
- Each step = ~500ms

So:
- Tick 0-5 = Step 0
- Tick 6-11 = Step 1  
- Tick 12-17 = Step 2
- Tick 18-23 = Step 3
- etc.

## Example Output

```
📍 Tick 18: Step 3 reached - sending SysEx to change Row 0 max_step to 7
📍 Tick 18: SysEx PRESS row=7 col=8
📍 Tick 19: SysEx PRESS row=0 col=7
📍 Tick 20: SysEx RELEASE row=0 col=7
📍 Tick 21: SysEx RELEASE row=7 col=8
📍 Tick 22: Record row 0 max_step=7
📍 Tick 24: Verify step=4 row=0 expected=None
  📊 Row 0 position: 4 (observation only)
📍 Tick 30: Verify step=5 row=0 expected=None
  📊 Row 0 position: 5 (observation only)
📍 Tick 36: Verify step=6 row=0 expected=None
  📊 Row 0 position: 1 (observation only)
```

This shows the config change took effect between tick 30 and tick 36!

## Tips

1. **Start with observation**: Use `"expected": null` to see what positions actually occur
2. **Add expectations**: Once you know the correct behavior, add expected values
3. **Tick 0 vs Step 0**: Tick 0 is before step 1 starts
4. **One action per tick**: You can have multiple actions at the same tick
5. **Stop with Ctrl+C**: Tests run until you stop them

## Next Steps

Once you identify when config changes take effect, you can:
1. Investigate why there's a delay
2. Add expected values to catch regressions
3. Create more complex test scenarios
