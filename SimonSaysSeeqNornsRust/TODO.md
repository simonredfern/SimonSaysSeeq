# TODO - SimonSaysSeeq Rust

## High Priority

### MIDI System
- [ ] Fix MIDI port logging - user still doesn't see clock/keyboard port assignments in logs
- [ ] Implement MIDI keyboard note removal when pressing grid buttons in Mozart mode
- [ ] Add precise tick offset calculation for MIDI note recording timing
- [ ] **Phase Correction Investigation**: Check how often check_phase_correction is called. We often seem to have a large drift.
  - Investigate if phase correction is running too frequently or infrequently
  - Analyze drift patterns and timing accuracy
  - May need to adjust correction frequency or algorithm
- [ ] **Scrolling Brightness Issue**: Scrolling doesn't work fully as expected - the scroll should add a brightness level for the current column for rows 0-6 irrespective if the note is active or not.
  - Active notes should be brighter and non-active notes should also be brighter
  - This should also be the case in Mozart mode otherwise we can't see the scrolling
  - Current implementation only shows brightness for active notes, making scroll position invisible on empty steps

### Sequencer Improvements
- [ ] **Tempo Update Optimization**: If we Stop and Start the sequencer we can quickly go to the new tempo
  - Currently tempo changes during playback can cause timing issues
  - Stopping and starting allows clean tempo transition without audio glitches
  - Should implement quick stop/start cycle when tempo change is detected

### Mozart Mode
- [ ] Add visual feedback when notes are recorded vs when Mozart mode is displaying them
- [ ] Implement note editing (velocity, timing) via grid interaction in Mozart mode
- [ ] Add clear/reset function for recorded MIDI sequences

## 2025-09-05 - Scrolling Brightness Issue

### Problem: Scrolling Position Not Visible on Empty Steps
- Current scrolling implementation only shows brightness for active notes
- This makes the current column position invisible when there are no active notes on rows 0-6
- Scrolling should add a brightness level for the current column irrespective of whether notes are active

### Expected Behavior:
- **Active notes in current column**: Should be brighter (existing brightness + scroll brightness)
- **Non-active notes in current column**: Should also be brighter (base brightness for scroll position)
- **This should work in both normal sequencer mode AND Mozart mode**
- Without this, users can't see where the sequencer is scrolling to on empty steps

### Technical Details:
- Affects rows 0-6 (sequencer rows)
- Current column should have additional brightness overlay
- Should be visible in Mozart mode as well for navigation feedback
- Possibly needs modification to grid LED update logic

### Status: NEEDS IMPLEMENTATION
User reports scrolling position is not visible on empty grid positions, making navigation difficult.



## Medium Priority

### User Interface
- [ ] Add tempo change confirmation/feedback to user
- [ ] Improve ARM button LED feedback timing
- [ ] Add visual indicators for active MIDI recording

### Performance
- [ ] Optimize grid display updates to prevent sequencer timing disruption
- [ ] Reduce memory allocation in MIDI processing hot paths
- [ ] Profile and optimize main loop timing

### Configuration
- [ ] Add configuration option for default MIDI recording lane
- [ ] Add tempo change behavior preferences (immediate vs stop/start)
- [ ] Add MIDI channel mapping configuration

## Low Priority

### Features
- [ ] Multiple MIDI recording lanes support
- [ ] MIDI CC recording and playback
- [ ] Pattern-specific MIDI recordings
- [ ] Export/import MIDI sequences

### Code Quality
- [ ] Reduce compiler warnings
- [ ] Add more comprehensive error handling
- [ ] Improve logging consistency
- [ ] Add unit tests for MIDI recording functionality

## Completed
- [x] Fixed auto-detection logic to respect config.auto_detect_clock setting
- [x] Removed emoji logging that was causing display issues
- [x] Added MIDI keyboard input logging
- [x] Fixed initialization order (clock detection before output port assignment)
- [x] Implemented MIDI note recording into keyboard_midi_note_events structure
- [x] Optimized Mozart ARM button to only update display on state changes
- [x] Organized unused scripts into old_or_unused_scripts_and_files folder

## Notes
- MIDI keyboard input is now continuously recorded regardless of ARM button state
- Mozart ARM button only controls display of recorded MIDI data
- All MIDI notes are currently recorded to lane 1, step position matches sequencer position
