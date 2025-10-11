# Test Framework - Simple and Honest

## What the Test Does

1. **Sends SysEx** button commands at step N
2. **Records** that we sent it at step N
3. **Assumes** the sequencer processes it and applies the change at step N+1
4. **Calculates** expected position based on this assumption
5. **Compares** with actual position from log
6. **FAILS** if they don't match

## No More Complex Detection

❌ **Removed**: Pattern detection (reset detection, wrap detection)
❌ **Removed**: Searching through logs to find when change happened
❌ **Removed**: Peeking at actual position to decide formula

✅ **Simple rule**: SysEx sent at step N takes effect at step N+1

## What Failures Mean

If verification FAILS, it means:
- **The sequencer didn't apply the change at step N+1**
- Could be a timing issue
- Could be a bug
- Could be our assumption is wrong

All of these are VALUABLE to know!

## Example: Row 0

```
Step 10: Send SysEx to change max_step to 7
Step 10: RecordRowConfig(row=0, max_step=7)
Step 11: Assume change is now active

At step 18 verification:
- Steps 1-10: Used old max_step=31
- Position at step 10: 10
- Step 11: Change active, 10 > 7 so reset to 0
- Steps 11-18: Advance with new max_step=7
- Expected position: (0 + 7) % 8 = 7
- Actual from log: should be 7
- If not 7: TEST FAILS (something is wrong!)
```

## Test Configuration

- **BPM**: 30 (slow, deterministic)
- **Assumption**: SysEx takes effect at step N+1
- **No guessing**: Clear expectation, clear failure mode

## Row 2 at Step 40 - The Test Case

Previous run at 120 BPM:
- SysEx sent at step 30
- Position at step 40 was 10 (should have been ~4 with new max_step=5)
- Change took >10 steps to take effect

At 30 BPM:
- If change takes effect at step 31 (as expected):
  - Position at step 30: 0 (wrapped from 29)
  - Step 31: 0 (already valid, no reset)
  - Steps 31-40: Advance with max_step=5
  - Expected: (0 + 9) % 6 = 3
- If actual ≠ 3: **FAIL** = reveals the issue!

## Ready to Run

```bash
cargo run --release --bin automated_test_clock -- --script test2.json
```

This is now a REAL test that can reveal bugs!
