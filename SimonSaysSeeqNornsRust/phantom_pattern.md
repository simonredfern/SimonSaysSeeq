# Phantom Pattern Bug

## Status: ✅ RESOLVED

**Fix Applied**: See `PHANTOM_PATTERN_FIX.md` for complete fix documentation.

**File Changed**: `src/main.rs` lines 545-567 removed (duplicate MIDI generation)

**Date Fixed**: 2024-01-XX

---

## Summary
A duplicate "phantom" pattern exists alongside the intended pattern at ALL times. When the row length is set to maximum, the phantom remains hidden because it stays perfectly in sync with the intended pattern. However, when row length is set to anything less than maximum, the phantom pattern becomes audible as it drifts out of sync, advancing through the sequence by (max_length - set_length) steps per cycle.

## Bug Description

### Apparently Working Case (Max Length)
- **Pattern**: Steps 1,1 are ON, all other steps are OFF
- **Row Length**: Maximum (16 steps)
- **Result**: ✅ Appears to work correctly
  - LEDs display correctly
  - MIDI output matches the visual display
  - Scrolling works perfectly
  - Pattern cycles correctly over 16 steps
  - **BUT**: The phantom pattern is actually present, just hidden because it's in perfect sync with the intended pattern

### Broken Case (Max - 1)
- **Pattern**: Steps 1,1 are ON, all other steps are OFF
- **Row Length**: 15 steps (max - 1)
- **Result**: ❌ Phantom pattern appears
  - The correct pattern cycles over 15 steps as expected
  - BUT a duplicate "phantom" pattern also plays
  - The phantom pattern is offset by 1 step to the right on each cycle
  - Both patterns are audible in MIDI output
  - Visual LEDs may show the correct pattern, but audio reveals the phantom

### Broken Case (Max - 2)
- **Pattern**: Steps 1,1 are ON, all other steps are OFF
- **Row Length**: 14 steps (max - 2)
- **Result**: ❌ Phantom pattern appears
  - The correct pattern cycles over 14 steps as expected
  - BUT a duplicate "phantom" pattern also plays
  - The phantom pattern is offset by 2 steps to the right on each cycle
  - Both patterns are audible in MIDI output

## Pattern of the Bug

The offset of the phantom pattern appears to be directly related to the difference between maximum length and the set length:

```
Offset per cycle = Max Length - Set Length
```

| Row Length | Difference from Max | Phantom Offset per Cycle | Audible? |
|------------|---------------------|--------------------------|----------|
| 16 (max)   | 0                   | 0 (stays in sync)        | No - hidden by synchronization |
| 15         | 1                   | +1 step per cycle        | Yes - phantom drifts |
| 14         | 2                   | +2 steps per cycle       | Yes - phantom drifts faster |
| 13         | 3                   | +3 steps per cycle (?)   | Yes - phantom drifts even faster |
| ...        | ...                 | ...                      | ... |</parameter>

<old_text line=79>
## Likely Root Causes

This bug pattern suggests one of the following issues:

## Symptoms

1. **Audio**: Double-triggering of notes - you hear the intended pattern AND the phantom pattern
2. **Visual**: LEDs may show correct pattern, but the audio doesn't match
3. **Scrolling**: The visual scrolling may work correctly, but phantom notes still play
4. **Phase Drift**: The phantom pattern continuously drifts out of phase with the real pattern

## Example Timeline

### With Row Length = 15 (Max - 1)

```
Cycle 1:  Real pattern at step 0-1, Phantom at step 0-1 (aligned)
Cycle 2:  Real pattern at step 0-1, Phantom at step 1-2 (offset +1)
Cycle 3:  Real pattern at step 0-1, Phantom at step 2-3 (offset +2)
Cycle 4:  Real pattern at step 0-1, Phantom at step 3-4 (offset +3)
...and so on
```

### With Row Length = 14 (Max - 2)

```
Cycle 1:  Real pattern at step 0-1, Phantom at step 0-1 (aligned)
Cycle 2:  Real pattern at step 0-1, Phantom at step 2-3 (offset +2)
Cycle 3:  Real pattern at step 0-1, Phantom at step 4-5 (offset +4)
Cycle 4:  Real pattern at step 0-1, Phantom at step 6-7 (offset +6)
...and so on
```

## Likely Root Causes

This bug pattern suggests one of the following issues:

### 1. Dual Counter System (MOST LIKELY)
- There are TWO separate step counters or indices running simultaneously
- One counter wraps at `set_length` (drives the intended pattern)
- Another counter wraps at `max_length` (drives the phantom pattern)
- When `set_length == max_length`, both counters advance in lockstep (phantom is hidden)
- When `set_length < max_length`, the counters drift apart by (max_length - set_length) per cycle
- Both counters trigger pattern evaluation, creating double notes</parameter>

<old_text line=88>
### 2. Buffer/Array Index Issue
- There may be two separate indices being used:
  - One that correctly wraps at `set_length`
  - Another that wraps at `max_length`
- Both indices might be triggering pattern evaluation

### 2. Buffer/Array Index Issue
- There may be two separate indices being used:
  - One that correctly wraps at `set_length`
  - Another that wraps at `max_length`
- Both indices might be triggering pattern evaluation

### 3. Clock/Trigger Split Path
- The master clock or trigger source may split into two paths
- Path A: Correctly handles set_length wrapping
- Path B: Always wraps at max_length (never updated when length changes)
- Both paths generate MIDI output, but only Path A drives the LED display
- This explains visual correctness but audio doubling</parameter>

<old_text line=98>
### 4. LED Update vs. MIDI Output Mismatch
- LED display logic correctly handles the set length
- MIDI output logic uses a different calculation that doesn't respect set length
- This would explain why visuals look correct but audio has phantoms

### 4. LED Update vs. MIDI Output Mismatch
- LED display logic correctly handles the set length
- MIDI output logic uses a different calculation that doesn't respect set length
- This would explain why visuals look correct but audio has phantoms

## Code Areas to Investigate

1. **Step Counter Increment/Wrap Logic**
   - Look for modulo operations: `step % length`
   - Verify all length calculations use `set_length` not `max_length`

2. **Pattern Playback/Trigger Logic**
   - Check where notes are triggered
   - Ensure only one trigger point exists per step
   - Verify length boundaries are respected

3. **MIDI Output Generation**
   - Compare LED update code with MIDI trigger code
   - Look for discrepancies in length handling

4. **Row Length Set/Update Functions**
   - Verify that changing row length properly updates all relevant counters
   - Check if any cached indices need to be reset when length changes

## Reproduction Steps

1. Create a new pattern
2. Set steps 0 and 1 to ON (two consecutive steps at the beginning)
3. Set all other steps to OFF
4. Set row length to maximum (16) - verify it APPEARS to work correctly
   - Note: The phantom is present but hidden because it's in sync!
5. Set row length to 15 (max - 1)
6. Play the sequence
7. Observe: You should hear TWO patterns playing, with the phantom shifting by 1 step each cycle
8. Set row length back to 16 and observe that the double pattern disappears (they re-sync)</parameter>

<old_text line=134>
## Expected Behavior

When row length is set to any value less than maximum:
- Only ONE pattern should play
- The pattern should cycle over the set length
- No phantom patterns should appear
- MIDI output should match LED display exactly

## Expected Behavior

When row length is set to any value less than maximum:
- Only ONE pattern should play
- The pattern should cycle over the set length
- No phantom patterns should appear
- MIDI output should match LED display exactly

## Actual Behavior

At ALL times (including at maximum row length):
- TWO patterns are being generated internally
- When row length equals maximum:
  - Both patterns stay synchronized (same steps at same time)
  - Only one note per step is heard (because both patterns trigger the same note simultaneously)
  - The bug is HIDDEN but still present
- When row length is less than maximum:
  - The two patterns drift out of sync
  - One pattern cycles correctly over the set length
  - A phantom pattern also plays, shifting by (max_length - set_length) steps per cycle
  - Creates confusing audio output with double-triggers that reveal the underlying dual-pattern bug

## ROOT CAUSE IDENTIFIED ✅

### The Dual Grid Reading Paths

There are **TWO separate code paths** that read the grid and generate MIDI notes:

#### Path 1: `process_step()` (Correct) - Line 656 in sequencer.rs
```rust
let row_step = seq_a_row_state.current_row_step;
let grid_value = state.sequencer_a_grid[row_step][row_idx];
```
- Uses `current_row_step` (per-row counter)
- Respects `max_step` (row length)
- Wraps correctly at the set row length
- Sends MIDI via `SequencerEvent::MidiEvent`

#### Path 2: `get_step_events()` (WRONG) - Line 508 in sequencer.rs
```rust
let grid_value = state.sequencer_a_grid[step][row];
```
- Uses `step` parameter = master step counter (always 0-31)
- **IGNORES row length** - always cycles through all 32 steps!
- Called from `handle_sequencer_event()` in main.rs line 547
- Also sends MIDI output

### Why the Phantom Appears

When row length is set to max (31):
- Path 1: Reads steps 0-31 using `current_row_step`
- Path 2: Reads steps 0-31 using master `step`
- **Both paths read the same steps** → phantom is hidden (in sync)

When row length is set to 15 (max-1):
- Path 1: Reads steps 0-15 using `current_row_step`, wraps back to 0
- Path 2: Reads steps 0-31 using master `step`, wraps back to 0
- **Paths diverge** → phantom becomes audible, drifting by 1 step per cycle

### The Math Behind the Phantom

```
Path 1 cycles every: set_length steps
Path 2 cycles every: 32 steps (always)

Drift per cycle = 32 - set_length

When set_length = 31: drift = 1 step per cycle
When set_length = 30: drift = 2 steps per cycle
When set_length = 32: drift = 0 (no phantom audible)
```

This perfectly matches the observed behavior!

### The Fix

The `get_step_events()` function must be modified to use the per-row `current_row_step` instead of the master `step` parameter. Either:

1. **Remove the call to `get_step_events()`** entirely from `handle_sequencer_event()` since `process_step()` already handles MIDI output correctly, OR

2. **Fix `get_step_events()`** to not read the grid at all, or read it using the correct row counter

Option 1 is likely correct since it eliminates the duplicate MIDI generation path entirely.

## Step Counter Analysis

After analyzing the codebase, here's what we found regarding step counters:

### Counter Architecture (As Designed)

The sequencer has the following counter structure:

1. **Master Step Counter** (`sequencer_a_current_master_step`)
   - Located in `SequencerState` 
   - Always cycles 0-31 (32 steps, zero-indexed)
   - Advances once per step in `advance_step()` function
   - Wraps at `state.last_step` (which is 31 for full 32-step cycle)
   - **Purpose**: Global sequencer position, used for synchronization

2. **Per-Row Step Counters** (`current_row_step` in `SequencerARowStates`)
   - One counter per row (8 rows total: 0-6 for sequencer, 7 for control)
   - Each can have independent length via `max_step` field
   - Advances independently in `advance_step()` function
   - Wraps at `seq_a_row_state.max_step` (which can be 0-31)
   - **Purpose**: Track playback position for each individual row

### How Grid Reading Works

In `process_step()` function (line 628+):
```
let row_step = seq_a_row_state.current_row_step;
let grid_value = state.sequencer_a_grid[row_step][row_idx];
```

**Key Finding**: The grid is read using `current_row_step` (the per-row counter), NOT the master step counter. This is correct.

### Advancement Logic

In `advance_step()` function (line 585-607), the counters advance like this:

1. Master counter advances:
```
state.sequencer_a_current_master_step += 1;
if state.sequencer_a_current_master_step > state.last_step {
    state.sequencer_a_current_master_step = state.first_step;
}
```

2. Each row counter advances independently:
```
for (row_idx, seq_a_row_state) in state.sequencer_a_row_states.iter_mut().enumerate() {
    seq_a_row_state.previous_row_step = seq_a_row_state.current_row_step;
    seq_a_row_state.current_row_step += 1;
    if seq_a_row_state.current_row_step > seq_a_row_state.max_step {
        seq_a_row_state.current_row_step = seq_a_row_state.first_step;
    }
}
```

### Why This Should Work (But Apparently Doesn't)

Based on the code analysis:
- There is only ONE place where MIDI notes are generated: in `process_step()` at line 673
- This place reads the grid using `current_row_step` (the per-row counter)
- The per-row counter correctly wraps at `max_step`
- There is NO code path that reads using the master counter

**This means the phantom pattern is NOT coming from dual counter reads in the obvious way.**

### Confirmed: Dual MIDI Generation Paths

The code analysis confirms there are TWO places generating MIDI:

1. **`process_step()` at line 673** - Sends `SequencerEvent::MidiEvent` directly
2. **`get_step_events()` called from `handle_sequencer_event()`** - Also generates MIDI via the event loop

Both paths trigger MIDI output, creating the phantom pattern.

### Call Stack for Path 2 (Phantom Source)

```
main.rs:1885 → external_advance_step()
  ↓
sequencer.rs:1520 → advance_step()
  ↓
sequencer.rs:582 → sends SequencerEvent::Step { step: master_step }
  ↓
main.rs:539 → handle_sequencer_event() receives Step event
  ↓
main.rs:547 → calls get_step_events(row, 0, step) ← MASTER STEP!
  ↓
sequencer.rs:508 → reads grid[step][row] ← WRONG COUNTER!
  ↓
main.rs:551-556 → sends MIDI note on/off
```

### The Fix

Remove lines 545-567 in `main.rs` (`handle_sequencer_event()` Step case) that call `get_step_events()` and send MIDI, since `process_step()` already handles all MIDI output correctly with the proper per-row counters.

## Critical Insight

**The phantom pattern is not created when length < max; it EXISTS ALL THE TIME.** The difference in row length simply causes the phantom to desynchronize from the intended pattern, making it audible. This means:

1. The bug exists even when everything "appears" to work at max length
2. Fixing this requires finding and eliminating the SECOND pattern generation path ✅ **DONE**
3. Simply adjusting wrapping logic won't fix it - we need to find where the duplicate playback originates ✅ **FOUND**
4. Look for duplicate clock handlers, redundant event loops, or multiple playback code paths ✅ **REMOVED**

---

## See Also

- **`PHANTOM_PATTERN_FIX.md`** - Complete documentation of the fix
- `src/main.rs` - Location of the fix (removed lines 545-567)
- `src/sequencer.rs` - Correct MIDI generation in `process_step()` function
