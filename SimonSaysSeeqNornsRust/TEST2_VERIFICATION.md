# Test2 Verification Solution

## Problem

Test2 dynamically changes row `max_step` values while the sequencer is running. The existing verification logic couldn't predict expected sequencer state after these dynamic changes, resulting in messages like:

```
🔍 Verify: Check sequencer state at cumulative step 40 - Rows 0,1,2 should be wrapping at new lengths (no step number found)
```

## Root Cause

The verification system was designed for Test1, which has **static** row configurations. It used simple modulo arithmetic:

```rust
expected_step = cumulative_step % (max_step + 1)
```

This doesn't work for Test2 because:
1. Rows start with one `max_step` value
2. Mid-run, `max_step` changes to a new value
3. The sequencer resets position if `current_step > new_max_step`
4. From that point forward, the row cycles at the new length

## Solution

### 1. New Test Command: `RecordRowConfig`

Added a new test script command to explicitly track configuration changes:

```json
{
  "command": "RecordRowConfig",
  "row": 0,
  "max_step": 7
}
```

This tells the test framework: "At this cumulative step, row 0's max_step changed to 7."

### 2. Configuration Change Tracking

The `AutomatedTestClock` now tracks all row configuration changes:

```rust
struct RowConfigChange {
    cumulative_step: usize,  // When the change happened
    row: usize,               // Which row changed
    new_max_step: usize,      // New max_step value
}
```

### 3. Cumulative Step Tracking

Track cumulative steps as the test progresses:

```rust
TestCommand::WaitSteps { count } => {
    self.wait_for_steps(*count)?;
    self.cumulative_steps += *count as usize;
}
```

### 4. Smart Position Calculation

For verification, calculate expected position accounting for config changes:

```rust
// Find most recent config change for this row
let mut current_max_step = initial_max_steps[row_idx];
let mut last_change_step = 0;
let mut old_max_step = initial_max_steps[row_idx];

for change in &self.row_config_changes {
    if change.row == row_idx && change.cumulative_step <= cumulative_step {
        old_max_step = current_max_step;
        current_max_step = change.new_max_step;
        last_change_step = change.cumulative_step;
    }
}

// Calculate expected position
if last_change_step == 0 {
    // No config changes - simple modulo
    expected_step = cumulative_step % (current_max_step + 1)
} else {
    // Calculate position at moment of change
    position_at_change = last_change_step % (old_max_step + 1);
    
    // Sequencer resets to 0 if position > new_max_step
    position_after_reset = if position_at_change > current_max_step {
        0
    } else {
        position_at_change
    };
    
    // Advance from that position
    steps_since_change = cumulative_step - last_change_step;
    expected_step = (position_after_reset + steps_since_change) % (current_max_step + 1);
}
```

## Example: Row 0 in Test2

**Initial state:**
- Row 0: max_step = 31 (32 steps: 0-31)

**Cumulative step 10:** Change to max_step = 7
- Position before change: 10 % 32 = 10
- New max_step: 7 (8 steps: 0-7)
- Position 10 > 7, so **reset to 0**
- Row 0 is now at position 0 with 8-step cycle

**Cumulative step 18:** Verify position
- Steps since change: 18 - 10 = 8
- Expected position: (0 + 8) % 8 = 0
- Row 0 should be at position 0 ✓

**Cumulative step 19:** Next step
- Steps since change: 19 - 10 = 9
- Expected position: (0 + 9) % 8 = 1
- Row 0 should be at position 1 ✓

## Updated Test2 Structure

Each max_step change now has a `RecordRowConfig` command:

```json
{
  "commands": [
    {"command": "WaitSteps", "count": 10},
    {"command": "SysExButton", "row": 7, "col": 8, "press": true},
    {"command": "SysExButton", "row": 0, "col": 7, "press": true},
    {"command": "SysExButton", "row": 0, "col": 7, "press": false},
    {"command": "SysExButton", "row": 7, "col": 8, "press": false},
    {"command": "RecordRowConfig", "row": 0, "max_step": 7},
    {"command": "WaitSteps", "count": 8},
    {"command": "VerifyState", "description": "Check sequencer state at cumulative step 18"}
  ]
}
```

## Verification Output

With this solution, verification now shows actual vs expected:

```
🔍 Verify: Check sequencer state at cumulative step 18 - ✅ All rows match - [R0:✓0, R1:✓18, R2:✓18, R3:✓2, R4:✓3, R5:✓4, R6:✓2]
```

Or if there's a mismatch:

```
🔍 Verify: Check sequencer state at cumulative step 18 - ⚠️ Mismatch - [R0:10≠0, R1:✓18, R2:✓18, R3:✓2, R4:✓3, R5:✓4, R6:✓2]
```

Format:
- `R0:✓0` = Row 0 at position 0 (matches expected)
- `R0:10≠0` = Row 0 at position 10 (expected 0, mismatch!)

## Key Insight

> "If we can predict the future for a static pattern and we change a pattern to a new static pattern, we should be able to predict the future for that as well. We just have multiple static patterns and different starting points."

The solution recognizes that Test2 has **piecewise static behavior**:
- Segment 1: Steps 0-10 with initial config
- Segment 2: Steps 10-20 with row 0 changed
- Segment 3: Steps 20-30 with rows 0,1 changed
- Segment 4: Steps 30-40 with rows 0,1,2 changed

Each segment is predictable. We just need to track the transition points.

## Files Modified

1. `src/test_script.rs` - Added `RecordRowConfig` command type
2. `src/automated_test_clock.rs` - Added tracking and smart calculation
3. `src/midi_clock_generator.rs` - Added placeholder handler
4. `test2.json` - Added `RecordRowConfig` commands

## Testing

To verify this works:

```bash
# Terminal 1
cargo run --bin simon_says_seeq

# Terminal 2
cargo run --bin automated_test_clock -- --script test2.json
```

Look for verification messages showing all rows match expected positions.