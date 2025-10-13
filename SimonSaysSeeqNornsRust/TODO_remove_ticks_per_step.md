# TODO: Remove ticks_per_step

## Issue
The `ticks_per_step` variable is set to 12 throughout the codebase, but the actual MIDI clock timing uses 6 MIDI clock pulses per step (at 24 PPQN, where 6 clocks = 1 sixteenth note).

This creates confusion between:
- **Internal tick resolution**: `ticks_per_step = 12` (used for ratcheting/swing)
- **Actual MIDI clock ticks**: 6 MIDI clocks per step
- **Real tick counter**: `tick_count_since_midi_clock_start` increments once per MIDI clock

## Current Usage

### Where ticks_per_step is used:
1. **`get_step_events()` - Ratcheting logic** (sequencer.rs:504-525)
   - Calculates `ticks_between_ratchets = state.ticks_per_step / ratchet_count`
   - Used to schedule multiple note retriggering within a step
   
2. **`apply_swing_timing()` - Swing timing** (sequencer.rs:1243)
   - Calculates `swing_offset = state.swing_amount * state.ticks_per_step * 0.5`
   - Used to delay off-beat steps for groove

3. **Configuration/State storage**
   - `SequencerState.ticks_per_step` (sequencer.rs:155)
   - `SequencerConfig.ticks_per_step` (config.rs:72)
   - Pattern JSON files (current_pattern.json, test_pattern_1.json)
   - Config validation (config.rs:294)

### Actual Clock System
The sequencer runs on external MIDI clock:
- MIDI Clock arrives at 24 PPQN (pulses per quarter note)
- Step advancement happens every 6 MIDI clocks (main.rs:1838)
- `tick_count_since_midi_clock_start` increments on each MIDI clock
- `tick_in_step` resets to 0 on each step

## Problem Analysis

The `get_step_events()` function appears to be **legacy code** from an internal clock implementation:
- It's called from `handle_sequencer_event(SequencerEvent::Step)` (main.rs:508)
- But actual MIDI notes are sent directly from `process_step()` via `MidiEvent` 
- The ratcheting logic in `get_step_events()` may not be working correctly with external clock

## Removal Plan

### Phase 1: Investigate
- [ ] Verify if `get_step_events()` is actually used or dead code
- [ ] Test if ratcheting feature works with current external MIDI clock
- [ ] Test if swing feature works with current external MIDI clock
- [ ] Document the actual tick timing architecture

### Phase 2: Refactor (if features are broken)
- [ ] Reimplement ratcheting using actual MIDI clock ticks (6 per step)
- [ ] Reimplement swing using actual MIDI clock ticks
- [ ] Remove `ticks_per_step` from all calculations
- [ ] Use direct MIDI clock counts: 6 ticks per step

### Phase 3: Clean Up
- [ ] Remove `ticks_per_step` from `SequencerState`
- [ ] Remove `ticks_per_step` from `SequencerConfig`
- [ ] Remove from pattern JSON serialization
- [ ] Remove validation in config.rs
- [ ] Remove from README.md documentation
- [ ] Remove from rpi5_build_and_run.sh default config

### Phase 4: Testing
- [ ] Verify step timing unchanged
- [ ] Verify ratcheting works (if feature is retained)
- [ ] Verify swing works (if feature is retained)
- [ ] Verify pattern loading/saving
- [ ] Verify clock division resets work correctly

## Notes
- **Do NOT remove until ratcheting/swing are tested or reimplemented**
- The value of 12 may have been chosen for finer resolution (12 = LCM of 2,3,4 for ratchet divisions)
- Current external clock implementation may have made this obsolete
- Check if `SequencerEvent::Step` handler is even necessary with external clock

## Related Files
- `src/sequencer.rs` - Main usage of ticks_per_step
- `src/config.rs` - Configuration definition and validation
- `src/main.rs` - Step event handler that calls get_step_events()
- `current_pattern.json` - Serialized state with ticks_per_step
- `test_pattern_1.json` - Test pattern with ticks_per_step
- `README.md` - Documentation references
- `rpi5_build_and_run.sh` - Default config generation

## Decision Required
Before removing, we need to decide:
1. Keep ratcheting feature? If yes, reimplement with 6 ticks/step
2. Keep swing feature? If yes, reimplement with 6 ticks/step
3. Or remove both features entirely?

Current external clock architecture may not need these features at all since timing is driven by external hardware.