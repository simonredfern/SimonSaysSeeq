# PROBLEM 1: MIDI Silence When max_step < 31

## Symptom

When setting max_step < 31 for a row, that row's **MIDI output is broken** (not working correctly).

### Example
- Set row 1 max_step = 15 (16 steps: 0-15)
- Row 1 MIDI output only plays **half the time**
- Expected: MIDI triggers on programmed steps continuously
- Actual: MIDI triggers for some period, then silent for equal period
- When max_step = 31, MIDI works fine

## Current Behavior
- When max_step = 31: MIDI works correctly
- When max_step < 31: MIDI is broken (does not work)

## Suspected Issue

Looking at `src/sequencer.rs` line 590:
```rust
let grid_value = state.sequencer_a_grid[row_step][row_idx];
```

When a row has max_step = 15:
- Row cycles through steps: 0, 1, 2, ..., 14, 15, 0, 1, ...
- The `row_step` value should wrap correctly
- But MIDI is not working

Possible causes:
1. Row step counter not wrapping correctly when max_step < 31
2. Grid data not being read from correct location
3. Synchronization issue between master step and row step
4. LED display may or may not be related to the sequencer pattern data

## Specific "Half the Time" Pattern Analysis

Row 3 has:
- max_step = 15 (16-step cycle: 0→15→0)
- Pattern: Note on step 0 only
- Expected: Trigger every 16 master steps (at master steps 0, 16, 32, 48...)

"Half the time" symptom suggests:
- Expected: Note triggers at master 0, 16, 32, 48, 64, 80...
- Actual: Note triggers only at master 0, 32, 64, 96... (every 32 steps, not 16)
- This means the row is somehow cycling through 32 steps instead of 16
- Or the row is only playing during certain phases of the master cycle

## Hypothesis: Pattern Loading Desynchronization

When loading a pattern from file (`load_current_pattern_from_file`), the code does:
```rust
*state = loaded_state;  // Replaces entire state, including row steps
state.sequencer_a_current_master_step = current_step;  // Restores master step
```

This creates a desynchronization:
- **Row steps** are reset to 0 (from saved pattern)
- **Master step** remains at current position (e.g., 20)

For a row with max_step = 15:
- Row cycles: 0→1→2→...→15→0→1→... (16 steps)
- Master cycles: 0→1→2→...→31→0→1→... (32 steps)
- If row starts at 0 but master is at 20, they're out of phase
- Result: Row plays steps 0-15 once per 32 master steps = **MIDI output only half the time**

This would explain:
- Why max_step = 31 works (row and master both have 32-step cycles, stay synchronized)
- Why max_step < 31 fails (different cycle lengths create phasing issue)
- Why it's silent "half the time" (16 active steps out of 32 master steps)

BUT: This hypothesis has a flaw - rows have independent step counters that wrap at their own max_step, so they should NOT be affected by master step position. The row should play its pattern continuously regardless of where the master is.

## Key Question

**Is there code that checks master step position before sending MIDI?**

The row step counter advances correctly (verified by test4.json), but MIDI output is broken. This suggests:
1. Grid values are being read from the wrong location
2. There's a conditional check preventing MIDI output based on master step
3. The `process_step` function is not being called consistently for all rows

## Investigation Needed

1. Check if `process_step` is called every step for all rows
2. Verify `row_step` value is actually being used to read from grid
3. Check if there's any code that skips MIDI based on master step position
4. Verify grid data is correct for rows with max_step < 31

## Related Code

- `src/sequencer.rs:506-555` - `advance_step()` function
- `src/sequencer.rs:560-625` - `process_step()` function (MIDI triggering)
- `src/main.rs:733-765` - `SetMaxStepForRow` ARM action