# Quick Reference: Dynamic Max Step Testing

## TL;DR
**Problem**: Verification failed due to SysEx timing uncertainty  
**Solution**: Detect when changes actually happened from log data  
**Result**: Tests now pass consistently ✅

## What Changed
```
OLD: SysEx → Wait → Record → Verify (WRONG timing)
NEW: SysEx → Record → Wait → Verify (CORRECT timing)
```

## Detection Logic
```rust
// Detects position reset
if prev > new_max && curr <= new_max { CHANGE! }

// Detects new wrapping  
if prev == new_max && curr == 0 { CHANGE! }
```

## Test Pattern
```json
[
  {"command": "SysExButton", "..."},         // 1. Send command
  {"command": "RecordRowConfig", "..."},      // 2. Record NOW
  {"command": "WaitSteps", "count": N},       // 3. Wait
  {"command": "VerifyState", "..."}           // 4. Verify
]
```

## Running Tests
```bash
# Build
cargo build --release --bin automated_test_clock

# Run sequencer (terminal 1)
cargo run --release --bin simon_says_seeq

# Run test (terminal 2)
./target/release/automated_test_clock --script test2.json
```

## Expected Output
```
✅ All rows match - [R0:ok(4), R1:ok(30), R2:ok(0), ...]
```

NOT:
```
⚠️  Mismatch - [R0:exp3act4, ...]
```

## Troubleshooting

### Still getting mismatches?
1. Check RecordRowConfig is after SysEx (not after WaitSteps)
2. Verify formal_state.log exists and has data
3. Ensure sequencer is running before test starts
4. Wait at least 2-3 steps between config change and verification

### Log analysis
```bash
# See when Row 0 position changed
grep StepAdvancement formal_state.log | grep -o '\[0,[0-9]*\]' | head -20
```

## Key Files
- `src/automated_test_clock.rs` - Detection logic
- `test2.json` - Test script
- `formal_state.log` - Event log (auto-generated)

## Read More
- `TEST_FRAMEWORK_FIX_COMPLETE.md` - Full explanation
- `DYNAMIC_TESTING_FIX.md` - Detailed implementation
- `DETECTION_EXAMPLE.txt` - Worked example

---
Last Updated: 2025-01-11
