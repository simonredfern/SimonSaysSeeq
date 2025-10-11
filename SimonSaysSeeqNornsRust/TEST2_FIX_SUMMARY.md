# Test2 Reliability Fix Summary

## Problems Found and Fixed

### 1. Verification Timing (Same as Test1)
**Problem**: Verifications at step start (tick N×6) but log written during step
**Solution**: Move verifications to mid-step (tick N×6+3)

### 2. SysEx Button Timing Conflicts
**Problem**: SysEx buttons sent at step boundaries (multiples of 6) conflicted with step advancement
**Solution**: Move SysEx sequences between steps (ticks 22-25, 28-31, 34-37)

### 3. Incorrect Expectation at Step 10
**Problem**: Expected Row 1 position 10, but max_step=9 means it wraps at position 9
**Solution**: Correct expectation to position 0 (wrapped)

## Detailed Changes

### Verification Timing
```diff
- "at_tick": 6   → "at_tick": 9   (step 1)
- "at_tick": 12  → "at_tick": 15  (step 2)
- "at_tick": 18  → "at_tick": 21  (step 3)
- "at_tick": 63  (step 10, etc.)
```

### SysEx Button Timing
```diff
Row 0 change (31→7):
- "at_tick": 18-21  → "at_tick": 22-25  (moved away from step 4 boundary)

Row 1 change (30→9):
- "at_tick": 24-27  → "at_tick": 28-31  (moved away from step 5 boundary)

Row 3 change (15→4):
- "at_tick": 30-33  → "at_tick": 34-37  (moved away from step 6 boundary)
```

### Expectation Corrections
```diff
Step 10, Row 1:
- "expected": 10  → "expected": 0  (wrapped!)
```

## Test Results After Fix

✅ **All 19 verifications pass!**

**Row 0 (31→7):**
- Steps 1-7: Positions 1-7 ✅
- Step 8: Position 0 (wrapped) ✅
- Steps 9-10: Positions 1-2 ✅

**Row 1 (30→9):**
- Steps 1-9: Positions 1-9 ✅
- Step 10: Position 0 (wrapped) ✅
- Steps 11-12: Positions 1-2 ✅

**Row 3 (15→4):**
- Step 6: Position 1 ✅
- Step 8: Position 3 ✅
- Step 9: Position 4 ✅
- Step 10: Position 0 (wrapped) ✅
- Step 11: Position 1 ✅

## Key Timing Rules

1. **Verify at mid-step**: `(step × 6) + 3`
2. **SysEx between steps**: Start at `(step × 6) - 2`
3. **Avoid multiples of 6**: Step boundaries are busy
4. **Spread buttons**: 3-4 ticks per sequence

## Why Test2 Was Unreliable

The combination of:
- Verifying at step start (log not ready)
- Sending SysEx at step boundaries (conflicts with advancement)
- Wrong expectations (didn't account for wrapping)

Created race conditions where timing varied slightly between runs.

## Now Reliable Because

✅ Verifications happen when log is ready (mid-step)
✅ SysEx happens between step boundaries (no conflicts)
✅ Expectations match actual wrap behavior
✅ No more timing-dependent behavior

## Test2 Status: ✅ RELIABLE AND PASSING
