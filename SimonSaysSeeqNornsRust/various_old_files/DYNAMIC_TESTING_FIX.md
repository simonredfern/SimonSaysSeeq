# Dynamic Max Step Testing - Fix Documentation

## Problem Description

The Test2 dynamic testing framework was experiencing verification failures when checking sequencer state after runtime max_step changes. The test would report mismatches like:

```
🔍 Verify: Check sequencer state at cumulative step 30 - ⚠️  Mismatch - [R0:exp3act4, ...]
```

### Root Causes Identified

1. **Timing Uncertainty**: When SysEx commands are sent to change max_step values, there's an unpredictable delay (0-2 steps) before the sequencer processes the command due to:
   - Network/IPC latency between test framework and sequencer
   - MIDI SysEx processing time
   - Sequencer event loop timing

2. **Recording vs. Effect Timing**: The `RecordRowConfig` command was being called AFTER waiting for steps, recording the change at the verification step rather than when the SysEx was actually sent.

3. **Prediction vs. Reality**: The verification logic tried to PREDICT when changes would take effect based on when `RecordRowConfig` was called, but this didn't match when the sequencer actually processed the SysEx commands.

## Solution Implemented

### Detection-Based Verification

Instead of predicting when max_step changes take effect, the verification logic now **detects** when changes actually happened by analyzing the `formal_state.log`:

1. **Parse All Step Events**: Read all StepAdvancement events from the log
2. **Detect Change Points**: Look for telltale signs of max_step changes:
   - Position resets (e.g., position jumps from 18 to 1)
   - New wrapping behavior (wrapping at new max_step instead of old)
3. **Calculate from Actuals**: Use the detected change point and actual position at that point to predict subsequent positions

### Detection Algorithm

For each row with a recorded config change:

```rust
// Look for position reset after max_step change
if prev_pos > new_max_step && curr_pos <= new_max_step {
    // Reset detected at this step
    detected_changes.insert(row_idx, (step_num, new_max_step));
}

// Look for wrapping at new max_step
if prev_pos == new_max_step && curr_pos == 0 {
    // New wrapping behavior detected
    detected_changes.insert(row_idx, (step_num, new_max_step));
}
```

### Test Script Structure

The test script now places `RecordRowConfig` immediately after sending SysEx commands:

```json
{
  "command": "SysExButton",
  "row": 7,
  "col": 8,
  "press": false
},
{
  "command": "RecordRowConfig",
  "row": 0,
  "max_step": 7
},
{
  "command": "WaitSteps",
  "count": 8
},
{
  "command": "VerifyState",
  "description": "Check sequencer state at cumulative step 18"
}
```

This records the INTENT to change max_step at approximately the right step, then verification detects when it actually took effect.

## How Verification Works Now

### Step 1: Detect Change Points
```
Analyzing log for Row 0:
  Step 18: position 18
  Step 19: position 1  <- RESET DETECTED! Change took effect here
```

### Step 2: Calculate Expected Position
```
Change took effect at step 19
Current verification at step 30
Position at step 19: 1
Steps since change: 30 - 19 = 11
Expected position: (1 + 11) % 8 = 12 % 8 = 4
```

### Step 3: Compare with Actual
```
Actual position at step 30: 4
Expected: 4
✅ Match!
```

## Benefits

1. **Robust to Timing**: Works regardless of SysEx processing delay
2. **Reality-Based**: Verifies based on what actually happened, not predictions
3. **Informative**: Can detect if max_step changes don't take effect at all
4. **Debuggable**: Detailed logging shows exactly when changes were detected

## Testing Guidelines

### When Writing New Tests

1. **Record Early**: Call `RecordRowConfig` immediately after SysEx commands
2. **Wait Before Verify**: Allow a few steps for the change to take effect before verification
3. **Trust Detection**: The verification will detect when the change actually happened

### Example Test Pattern

```json
[
  {"command": "WaitSteps", "count": 10},
  
  // Send SysEx to change max_step
  {"command": "SysExButton", "row": 7, "col": 8, "press": true},
  {"command": "SysExButton", "row": 0, "col": 7, "press": true},
  {"command": "SysExButton", "row": 0, "col": 7, "press": false},
  {"command": "SysExButton", "row": 7, "col": 8, "press": false},
  
  // Record the change NOW (at step 10)
  {"command": "RecordRowConfig", "row": 0, "max_step": 7},
  
  // Wait for change to take effect and some additional steps
  {"command": "WaitSteps", "count": 8},
  
  // Verify (at step 18) - detection will find the actual change point
  {"command": "VerifyState", "description": "..."}
]
```

## Known Limitations

1. **Detection Window**: The detection algorithm searches within a window around the recorded change. If the actual change is far outside this window, it may not be detected.

2. **Complex Patterns**: If row positions naturally reset or wrap in ways that mimic max_step changes, detection may produce false positives.

3. **First-Step Changes**: Changes that occur at step 0 or 1 may be harder to detect due to limited history.

## Future Improvements

1. **Explicit Change Events**: Sequencer could log explicit "MaxStepChanged" events to `formal_state.log`
2. **Bidirectional Communication**: Test framework could receive acknowledgment from sequencer when config changes are applied
3. **Deterministic Testing**: Option to pause sequencer, apply changes, verify state, then resume

## Files Modified

- `src/automated_test_clock.rs`: Updated `verify_sequencer_state_at_step()` with detection logic
- `test2.json`: Moved `RecordRowConfig` commands to immediately after SysEx commands

## Related Issues

- Off-by-one errors in step counting (resolved)
- Confusion between master_step and cumulative step count (clarified)
- Log file not being cleared between runs (already fixed - truncation on test start)