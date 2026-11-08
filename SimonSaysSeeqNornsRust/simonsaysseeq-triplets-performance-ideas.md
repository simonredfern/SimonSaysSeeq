# SimonSaysSeeq: triplets and the controls used live

Notes from 5 November 2026, written against commit `dbc258f` ("concert mode") of `simonredfern/simonsaysseeq`, folder `SimonSaysSeeqNornsRust`. Nothing here is implemented yet.

## 1. Design principles

- **See everything at a glance.** The 32 steps across the two grids show exactly what will play. Nothing new may hide events on a second page or layer.
- **Performance, not editing.** In practice patterns rarely change live. Dynamics come from the modular (mutes, filters, voltage-controlled amplifiers on the hi-hats). The sequencer controls actually used are **last step per row** (for phasing), **undo** and **redo**.
- **Triplets are a phasing tool.** A row switched to triplet timing runs 1.5 times faster over the same visible steps. It's the same kind of control as last step: a different rate instead of a different length.

## 2. Timing basis

- External MIDI clock runs at 24 pulses per quarter note.
- A straight sixteenth step is 6 clocks. Steps advance at `tick_count % 6 == 0` in `src/main.rs` (around line 1869).
- A sixteenth triplet is 4 clocks.
- **3 triplet steps = 2 straight steps = 12 clocks.** Straight and triplet rows line up again every 12 clocks, so every position falls on a whole clock and nothing drifts.
- `ticks_per_step = 12` in the code is stale and not the real timing. See `TODO_remove_ticks_per_step.md`.

```
clock:     0  1  2  3  4  5  6  7  8  9  10 11
straight:  X                 X
triplet:   X           X           X
```

## 3. Implementation: triplet rows

A whole row advances every 4 clocks instead of 6. The grid display is unchanged; only that row's playhead moves faster.

- Add `clocks_per_step: u8` (6 or 4) to `SequencerARowStates`, with `#[serde(default)]` returning 6 so old pattern files still load.
- In `advance_step()` (`src/sequencer.rs`), stop advancing every row together. Each row advances when `tick_count % clocks_per_step == 0`. The master step stays on 6.
- Make sure a triplet row's note-off lands within its 4 clocks.
- Update the playhead lights per row, not all rows at once. This is the largest change.
- **Switch only on the 12-clock boundary.** Any change between straight and triplet waits for the next 12-clock boundary, so rows stay locked and nothing flams.
- Going to triplet, the row **keeps its step index**, so the pattern carries on from where it was.
- Going back to straight, the row **jumps to where it would be had it stayed straight** (straight steps since clock start, modulo its length), the same position a stop and start gives. Without this it ran on from wherever the faster triplet run had left it, out of step with the straight rows (found in testing, 6 November 2026).
- It works with last step: for example, a 6-step triplet row against a 4-step straight row gives an even 3-against-2 cycle.
- The rate is set together with last step (section 4): column 8 + step sets the length and makes the row straight, column 9 + step sets the length and makes it triplet. No toggle. The length changes straight away; the rate waits for the next 12-clock boundary.
- The existing rule that setting last step to 31 resyncs the row to the master step only applies to straight rows; a triplet row keeps its own position.

**Done in the working tree (6 November 2026).**
- `clocks_per_step` (6 or 4) and an unsaved `pending_clocks_per_step` on `SequencerARowStates`; older pattern files load as straight.
- `Sequencer::external_clock_tick()` is called on every MIDI clock instead of `external_advance_step()` every 6th. It applies pending rate changes on 12-clock boundaries, then plays and advances each row that is due. The master step still moves every 6 clocks.
- The playhead lights already updated per row (`GridUpdate` events), so no display change was needed. Gates are one-tick pulses, so they fit inside a 4-clock triplet step.
- Undo and redo leave each row's rate alone (like its position).
- Tests in `src/sequencer.rs` (`timing_tests`): an independent model of today's straight timing, checked against both the old and the new tick path over 1,200 clocks, plus triplet cases (4-clock rows, the 12-clock boundary, keeping the step, no double notes, 3 against 2, old pattern files, undo). The old sequencer tests now use an in-memory sequencer, because `Sequencer::new()` read and `stop()` wrote `current_pattern.json`.
- Built and running on the Pi; not yet tested by ear. Backups of the previous binary and `current_pattern.json` are in `~/simonsaysseeq-backup-20261106/` (`*.before-triplets`).

**More rates (6 November 2026).** Rows can also play 8th-note triplets (8 clocks per step) and dotted 16ths (9 clocks per step), set with columns 10 and 11. A rate change now waits for the least common multiple of the old and new rates: 12 clocks for straight and 16th triplets, 24 for 8th-note triplets, 18 for dotted 16ths, 36 for dotted and 16th triplets. Coming back to straight from any rate resyncs the row as above. Unknown values in a pattern file play straight. Tests: `eighth_triplet_and_dotted_rows_play_at_their_rates...`, `switching_to_eighth_triplets_and_dotted_waits_for_the_line_up...`, `rate_change_boundaries_are_where_the_rates_line_up`, `unsupported_rates_are_ignored`.

**Risk.** This rewrites the code where rows advance, which is also where the phantom-pattern bug lives. The test scripts assume "step × 6" timing (`TIMING_RULES_CHEATSHEET.md`), so add triplet test cases first.

**Dropped:** NONO Major Tom-style triplet slots between steps. They would need a hidden second layer, which breaks the at-a-glance principle.

## 4. Controls used live

The controls used during a performance, existing and new. All other buttons keep working as they do now, with or without `--concert`. Concert mode does not disable any buttons.

| Control | Button | Behaviour | Status |
|---|---|---|---|
| Undo | grid one, row 7, column 0 | as now | exists |
| Redo | grid one, row 7, column 1 | as now | exists |
| Temporary step | grid one, row 7, column 2 (hold) + step | an empty step plays for the next 2 passes of its row, a lit step is muted for 2 passes, then the pattern is back as it was; press the step again (still holding) to cancel; not saved, not undoable, cleared on stop; temporary steps show at brightness 6 then 3 as a countdown, muted ones blink | in working tree |
| Ratchet | grid one, row 7, column 7 (hold) + step | each press on the same step: style 1 (2 hits in the step) -> style 2 (3 hits) -> style 3 (4 hits) -> normal step; an empty step starts at style 1; hits are one-tick gates spread over the step's clocks, so ratchets follow the row's rate; saved and undoable (grid values 2-4); brightness 12 / 13 / 14 (normal beat 10, playhead on a beat 15) | in working tree |
| Last step, straight | grid one, row 7, column 8 (hold) + step | sets that row's last step and makes the row straight; while held, straight rows look normal and all others are dimmed (same picture at about a quarter brightness) | in working tree |
| Last step, 16th triplets | grid one, row 7, column 9 (hold) + step | sets that row's last step and makes the row 16th triplets (4 clocks per step); no toggle, so pressing again keeps it triplet; latched and saved, not undoable (like last step); while held, rows at this rate look normal and all others are dimmed | in working tree |
| Last step, 8th-note triplets | grid one, row 7, column 10 (hold) + step | sets the last step and makes the row 8th-note triplets (8 clocks per step, 0.75 times straight); same dimming | in working tree |
| Last step, dotted 16ths | grid one, row 7, column 11 (hold) + step | sets the last step and makes the row dotted 16ths (9 clocks per step, two-thirds of straight); same dimming | in working tree |
| Preset pattern | grid one, row 7, column 3 (hold) + step | as before; moved from column 10 | in working tree |
| Jump Loop 1X / 2X / 3X | grid one, row 7, columns 12 / 13 / 14 (hold) + start step + end step on one row | adds a segment to the row's Jump Loop, played 1, 2 or 3 times; segments play in the order entered, then repeat; end before start plays backwards; up to 8 segments; saved and undoable; while held, steps outside each row's segments are dimmed and a waiting start step is full brightness | in working tree |
| Jump Loop Reset | grid one, row 7, column 15 (hold) + any step on a row | clears the row's Jump Loop; the row plays normally again, in line with where it would be had it never had one; undoable | in working tree |
| Button press counts | (all control-row buttons, both grids) | every press is counted; on each clock stop (each jam) one line is added to `control_button_counts.jsonl` (next to `current_pattern.json`) with the date and that jam's presses, then counting starts again from zero | in working tree |
| Hold for triplets | grid two, row 7, column 2 | every row plays triplets while held; not saved, not undoable | new |
| Back to straight | grid two, row 7, double-tap column 2 | every row back to straight | new |
| MIDI clock light | grid two, row 7, column 0 | flashes on each beat (first sixteenth, brightness 3) while MIDI clock is coming in; off when no clock has arrived for 250 ms; display only | in working tree |
| Undo / redo redraw | grid one, row 7, columns 0 / 1 | all seven rows are redrawn straight after a successful undo or redo, instead of as the playhead passes; rows are sent 8 lights per OSC message (`/grid/led/level/row`), 28 messages instead of 224 | in working tree |
| MIDI clock re-detect | grid two, row 7, columns 0 and 1 held together | releases the clock binding; the next port to send clock is bound | exists, non-blocking in working tree |
| Crow search | grid two, row 7, columns 4, 5 and 6 held together | one quick search for Crow (no retries, does not stop the sequencer); the three lights go full brightness if found, dark if not | in working tree |

Optional hands-free control: sustain pedal (controller 64) on the MIDI keyboard port acts as "hold for triplets". Low effort, because MIDI input already arrives there.

Notes:
- `--concert` (`apply_concert_mode()` in `src/main.rs`) only turns off grid rediscovery and pins logging to warn. It does not change what any button does.
- The re-detect combination is the on-stage recovery if the clock interface is replugged, and the check in `simonsaysseeq-midi-clock-startup-fix.md` section 3 relies on it. With the MIDI fix it no longer blocks the main loop.

## 5. Remove the Crow startup fix

Crow is no longer in the case, but startup still pays for it:

- **Service wrapper** (`/usr/local/bin/simonsaysseeq_wrapper.sh`, generated by `install_service` in `rpi5_build_and_run.sh`): starts the binary, kills it after 3 seconds, waits 2 seconds, then starts it again. Without Crow this is pointless. The first instance can also still hold the MIDI ports while the second starts. Before the MIDI fix, a stale saved device name could make the binary exit at startup (`simonsaysseeq-midi-clock-startup-fix.md` section 2), and the restart sequence makes that harder to spot in the journal.
- **`Crow::initialize()`** (`src/crow.rs`) tries 3 times, sleeping 2 and 3 seconds between attempts, so about 5 seconds of startup is spent looking for a device that isn't there.

Done in the working tree (6 November 2026):
- **Crow detection was matching a grid.** `find_crow_device()` identified Crow by USB ID 0483:5740 alone. The older grid `m2949672` reports the same ID, so at startup the sequencer opened that grid's serial port (`/dev/ttyACM0`, also used by serialosc) as Crow and sent Lua commands to it. Crow is now also required to name itself "crow" in its USB product string (`ID_MODEL`); the grid reports `ID_MODEL=grid`. Unit tests in `src/crow.rs` cover both.
- **No Crow search at startup** unless `[crow] search_at_startup = true` in the config. The section can be absent; it defaults to false.
- **Crow search on demand:** grid two, row 7, columns 4, 5 and 6 held together run `Crow::search_once()`, a single attempt without sleeps. Logged at warn: "Crow: found and connected" or "Crow: not found". Not tested with a Crow, since none is connected.
- **Single-start wrapper:** `install_service` now generates a wrapper that `exec`s the binary once. `SSSEQ_CROW_BOOT_FIX=1` at install time brings back the old start-kill-start wrapper.
- Kept: the 10-second `ExecStartPre` delay. In concert mode, grids are only discovered once at startup, so serialosc and USB need to be ready by then.
- On the Pi: built and running; one process, the grid's serial port is no longer held. The old wrapper is in `~/simonsaysseeq-backup-20261106/`. The installed unit file was not regenerated, so its description still says "with Crow CV fix".

## 6. Order of work

1. **Fix gates not being output when the clock starts after the sequencer.** Ports are chosen once at startup and nothing revisits the choice (the unused `retry_detection()` in `src/midi.rs`). Without a clock at boot, the output is either not opened or opened on a stale saved device, and a renamed saved device makes the binary exit. See `simonsaysseeq-midi-clock-startup-fix.md`. Fixed in the working tree, not yet tested on the Pi. Booting with the clock already running did not work around it on the original code, so there may be a second cause (see that note, sections 1 and 3).
2. Remove the Crow startup fix (section 5). Done in the working tree.
3. Add triplet test cases.
4. Triplet rows (section 3), with the triplet toggle.
5. Hold for triplets and back to straight (section 4).
6. Sustain pedal, if the button feels awkward live.
