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

**Risk.** This rewrites the code where rows advance, which is also where the phantom-pattern bug lives. The test scripts assume "step × 6" timing (`TIMING_RULES_CHEATSHEET.md`), so add triplet test cases first.

**Dropped:** NONO Major Tom-style triplet slots between steps. They would need a hidden second layer, which breaks the at-a-glance principle.

## 4. Controls used live

The controls used during a performance, existing and new. All other buttons keep working as they do now, with or without `--concert`. Concert mode does not disable any buttons.

| Control | Button | Behaviour | Status |
|---|---|---|---|
| Undo | grid one, row 7, column 0 | as now | exists |
| Redo | grid one, row 7, column 1 | as now | exists |
| Last step, straight | grid one, row 7, column 8 (hold) + step | sets that row's last step and makes the row straight; while held, each row's first light shows bright for triplet, dim for straight | in working tree |
| Last step, triplet | grid one, row 7, column 9 (hold) + step | sets that row's last step and makes the row triplet; no toggle, so pressing again keeps it triplet; latched and saved, not undoable (like last step); same rate lights as column 8 | in working tree |
| Hold for triplets | grid two, row 7, column 2 | every row plays triplets while held; not saved, not undoable | new |
| Back to straight | grid two, row 7, double-tap column 2 | every row back to straight | new |
| MIDI clock status light | grid two, row 7, column 0 | blinks while waiting for a MIDI clock, steady dim once bound; display only | in working tree |
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
