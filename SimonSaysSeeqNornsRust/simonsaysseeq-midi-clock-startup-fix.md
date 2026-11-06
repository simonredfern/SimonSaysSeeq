# SimonSaysSeeq: no gates and frozen playhead when the clock starts after the sequencer

Notes from 5 November 2026, written against commit `dbc258f` ("concert mode") of `simonredfern/simonsaysseeq`, folder `SimonSaysSeeqNornsRust`. Sections 1 to 7 are diagnosis from reading the code at that commit. It has not been reproduced on the Pi. Section 8 describes the fix now in the working tree.

## 1. Symptom

- Start the sequencer, then start the clock: no gates come out, and the playhead doesn't move on the grids.
- The grids work: plugging one in shows the pattern.
- The problem is intermittent. Last week it eventually worked, possibly after booting with the clock running. Last night it didn't.
- Booting the Pi with the clock already running, on the original code, also failed. The cause in section 2 doesn't explain that: with a clock running at boot, the old scan should find it and open the output on that device. So there may be a second cause that hasn't been found yet (see section 3).

## 2. Cause

At startup, `MidiManager::new` (`src/midi.rs`) chooses the MIDI ports **once**, and nothing revisits that choice.

1. It tries the last known clock device saved in the config (`last_detected_device`). It listens for 500 milliseconds and keeps the device only if a clock is arriving.
2. Otherwise it runs a one-off timed scan of all inputs (`scan_for_midi_clock` in `src/midi_scanner/mod.rs`).
3. If no clock is heard, it falls back. What it falls back to depends on whether a device was saved:
   - **No saved device:** input goes to the first hardware MIDI port in the list (`initialize_input`), which is not necessarily the clock device. The output is not opened at all: `initialize_output` only connects when a port name is set, otherwise it logs "PORT_A not configured".
   - **Saved device that didn't send clock within 500 ms:** the saved name stays set (`midi.rs` around line 1021), so the fallback opens both the input and the output on the saved device, not the first hardware port. Gates then go to the old device, which may be the wrong interface.
   - **Saved device whose name no longer exists:** port lookup is a substring match on the full saved name, including the ALSA numbers (`find_input_port_by_name`, `find_port_by_name`). If the numbers changed, the fallback's `initialize_input()?` fails with "port not found". The error goes up through `MidiManager::new(...)?` and **the process exits**. The service has `Restart=on-failure`, so it restarts every 5 seconds and fails the same way until a clock is running at boot.

Effects when the clock starts later:
- If the input is on the wrong device, no clock ticks arrive. Steps only advance on a received clock tick (`MidiInputEvent::ClockTick` in `src/main.rs`), so **the playhead stays frozen**.
- With no output connection, every note is dropped with a "MIDI Note ON (no device)" warning, so **there are no gates**.
- `retry_detection()` exists in `src/midi.rs` but nothing calls it, so nothing recovers.
- The `ExternalClockTimeout` event never fires. It is only sent when the clock source changes from external to none (`check_clock_source_transition`). The 5-second timeout check only sets `running = false` and never sets the source to none. Only `force_redetection` does that.

Why it seems random:
- **Clock running or not at boot** decides whether the scan finds it.
- **USB enumeration order** decides what "first hardware port" means in the fallback. It changes with which Pi USB socket the hub is in, which hub socket each interface uses, and the order devices come up.
- **The saved device name includes ALSA (Advanced Linux Sound Architecture) client and port numbers**, the "20:0" part of e.g. `Midi Through:Midi Through Port-0 14:0`. Those numbers can change when devices are plugged in differently, so the "last known device" shortcut can miss after recabling.

## 3. Check before changing code

The service runs with `--concert`, which pins the log filter to warn and ignores `RUST_LOG`, so info-level lines such as "FALLBACK" and "Clock scan completed" never reach the journal. Grep for the warn-level ones:

```
sudo journalctl -u simonsaysseeq-rpi -b | grep -E "No reliable MIDI clock sources found|PORT_A not configured|Note ON \(no device\)|Failed to connect to last known device|MIDI input port .* not found"
```

Repeated service starts in the same boot point to the crash case in section 2:

```
sudo journalctl -u simonsaysseeq-rpi -b | grep -c "Started simonsaysseeq"
```

Then, with the sequencer running and the clock started, hold **row 7, columns 0 and 1 on grid two** together. This forces re-detection. If the playhead and gates come back, the cause above is confirmed. At this commit, re-detection runs inside the grid button handler: a 3-second scan plus about 1 second of cleanup, during which the main loop and grids are frozen.

Booting with the clock already running was expected to work around it, but it didn't (section 1). Collect evidence from such a boot before assuming the cause above is the whole story. Setting `RUST_LOG=info` in the unit has no effect while the wrapper passes `--concert`; to see info lines, remove `--concert` from `EXTRA_BINARY_ARGS` in `/usr/local/bin/simonsaysseeq_wrapper.sh` for that boot. Then:

```
journalctl --list-boots                     # is the journal kept across reboots?
sudo journalctl -u simonsaysseeq-rpi -b | grep -E "MIDI PORT ASSIGNMENTS|PORT_A_MIDI_CLOCK_IN_AND_GATES_OUT|Clock scan completed|Started|panicked|Error"
```

Things to check in that boot: whether the process stays up (one "Started" line, or one per wrapper restart), which port was chosen for clock and gates, and whether "MIDI Clock detected" appears at all. With the fix (section 8), the grid status light separates the cases: blinking means no clock is arriving on any port, steady dim with a frozen playhead means the clock is bound and the problem is further along, and a dark or unchanging light means the process or its main loop has stopped.

## 4. Two identical USB MIDI interfaces

Two interfaces of the same type have the same name, and the ALSA numbers are the only thing that tells them apart within one session. That is probably why the numbers ended up in the saved name. They are not stable across boots or recabling, though.

The clock port doesn't need a stable identity. The sequencer can identify it by **listening for which port is actually sending clock**. Whichever interface carries the clock becomes the clock-in and gates-out device, and the other becomes the keyboard port. That works with two identical interfaces without any naming rule.

So:
- **Within a session**, keep using the full port name including numbers. It is unambiguous while nothing is replugged.
- **Across boots**, don't rely on a saved name. If something must be saved, save the name without numbers. Because port lookup is a substring match, a name without numbers always picks the first of two identical interfaces, so it only helps with one interface. For identical interfaces, the only stable distinguishing detail is the USB physical path (the socket, from `/sys/class/sound/cardN/device`). That's more work and probably not needed if detection is continuous (see step 2 of the fix).

Using two identical interfaces can stay. With this approach, the only thing to avoid is sending clock into both at once.

## 5. Fix steps

In order of importance:

1. **Don't treat MIDI failures as fatal at startup.** `MidiManager::new` is called with `?` in `new_with_test_mode` (`src/main.rs`), so any MIDI error exits the process. That includes the ALSA sequencer not being ready at boot. Instead, log the error, start with no ports, and retry.

2. **Listen on all inputs until a clock arrives.** Replace the one-off scan with a listener that stays open on every input. It must run on its own threads so it never blocks the main loop, which matters in concert mode. "Midi Through" has to be included, because the test scripts send their clock through it.

3. **Bind on the first clock.** When clock messages arrive on port X:
   - bind clock input to X,
   - open the output (gates) to the same device X,
   - choose the keyboard port as the other interface (`find_other_usb_midi_device`),
   - ignore clock and notes from all other ports. (Keeping their listeners open but ignored is simpler than closing them, and recovery after a release is then immediate.)

   This replaces both the "first hardware port" guess and "output only opened if the clock was found at startup".

   A port that sends clock but has no output with the same name (for example a software clock generator's own source port) can't take gates. It should be skipped as a clock source for the rest of the session.

4. **Release on loss.** If the clock stops arriving for a while, go back to listening on all inputs. Then recabling or swapping the hub's USB socket mid-session recovers by itself. An `ExternalClockTimeout` event exists but never fires (section 2), so the release has to set the clock source to none itself. That also makes the event fire. Send all notes off before closing the output so no gate is left high.

5. **Remove or simplify the "last known device" shortcut** (`save_detected_device` and the start of `auto_detect_and_connect`). With continuous listening it's unnecessary. If kept, store the name without ALSA numbers.

6. **Show clock status on the grid.** For example, one control-row light blinks while waiting for a clock and goes steady once bound. On stage you can then see that the sequencer is waiting rather than broken, without reading logs. Use a button that does nothing in the concert layout.

7. **Make the logs visible in concert mode.** Logging a binding change ("clock bound to …", "clock lost, listening") at warn level means it still appears with `--concert`.

## 6. Tests on the Pi

| Case | Expected |
|---|---|
| Boot with clock running | binds immediately, gates and playhead work |
| Boot without clock, start clock later | binds on first clock, gates and playhead work |
| Move the hub to another Pi USB socket, reboot | same as above |
| Two identical interfaces, clock into one | that one becomes clock and gates, the other becomes keyboard |
| Unplug the clock interface mid-session, plug it back in | releases, listens, rebinds |
| ALSA not ready at boot (service starts early) | no exit; binds once ALSA is up |
| `--concert` | all of the above, and the grid status light still works |

## 7. Code references

Line numbers are for commit `dbc258f`, before the fix.

- `src/midi.rs`: `MidiManager::new` (around line 195), `initialize_output` (around 226), `initialize_input` (around 374), `initialize_keyboard_input`, `auto_detect_and_connect` (around 1012), `retry_detection` (around 1188, unused), `force_redetection` (around 1218), `save_detected_device` (around 1278), `find_other_usb_midi_device`.
- `src/midi_scanner/mod.rs`: `scan_for_midi_clock`.
- `src/main.rs`: `new_with_test_mode` (around 132), `MidiInputEvent::ClockTick` handling (around 1850), the grid two re-detect buttons (around 823).

## 8. Fix in the working tree (5 November 2026)

Implemented, uncommitted, in `src/midi.rs` and `src/main.rs`. It builds, and the new routing unit tests pass. On 6 November 2026 it was built and run on the Pi (see the end of this section).

How it works now:
- `MidiManager::new` never fails because of MIDI. With no ports, or no ALSA sequencer yet, it starts with nothing bound and logs one warning (fix step 1).
- One listener per input port, including Midi Through but not the sequencer's own ports. Callbacks run on midir's threads. While nothing is bound, the first port to send a clock tick, Start or Continue claims the clock role, and that first message is handled too, so no tick is lost (step 2). The port list is refreshed once a second while waiting, which picks up hot-plugged interfaces. Nothing is rescanned while bound.
- On the next main-loop pass, the gates output opens on the device with the same port name (exact name first, so two identical interfaces are told apart). The keyboard port is the other hardware input, preferring "USB MIDI Interface". Clock and notes from all other ports are ignored. A port with no matching output is rejected for the session (step 3).
- After 5 seconds without clock: all notes off, the output closes, the clock source goes to none (so `ExternalClockTimeout` now fires), and it listens on all inputs again. A clock that is stopped for more than 5 seconds between songs is therefore released and rebound on its next message (step 4).
- The saved-device shortcut is gone from `MidiManager`: nothing is read from or written to `last_detected_device` (step 5). The config field is still there, and so are the one-off scanner `src/midi_scanner/` and `midi_clock_detector`, which uses it. They can go in a cleanup.
- Grid two, row 7, column 0 shows the clock status: blinking while waiting, steady dim once bound (step 6). It is one of the two re-detect buttons. The light is checked twice a second and an LED message is only sent when it changes, plus a re-send every 5 seconds.
- Binding changes are logged at warn level, so they appear in the journal and with `--concert`: "MIDI: clock bound to …", "MIDI: clock on … lost (no clock for 5 s); listening on all inputs", "MIDI: listening for clock on N input(s): …" (step 7).
- Columns 0 and 1 on grid two (held together) now just release the current binding. They no longer block: the next port to send clock is bound straight away.

Pi test, 6 November 2026 (one USB MIDI Interface, `--concert`, root file system on the old SD card in a USB reader):
- Boot without clock, start clock later: binds, gates and playhead work.
- Stop the clock for more than 5 seconds, start again: releases, then rebinds without a restart.
- Boot with clock running: works. The log shows "MIDI: clock bound to 'USB MIDI Interface…'" about 10 seconds after the service started. This is the case that failed with the old code.
- The old binary's flood of "Event channel disconnected" errors at boot (from `midi_scanner`) is gone.
- With only one interface the log says "keyboard in: none". Notes and CCs on the clock port are still handled by the clock handler, so a keyboard on the same interface works.

Still to do from section 6: two identical interfaces, unplug and replug mid-session, moving the hub to another socket.

The Pi's previous binary and `src/` are backed up in `~/simonsaysseeq-backup-20261106`.
