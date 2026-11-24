# SysEx Button Command Reference

This document describes how to use SysEx button commands in test scripts.

## Command Format

```
F0 7D 53 53 51 02 <row> <col> <press> F7
```

- `F0` = SysEx start
- `7D` = Educational/Development use (non-commercial manufacturer ID)
- `53 53 51` = "SSQ" in ASCII (SimonSaysSeeQ)
- `02` = Button command
- `<row>` = Row number (0-7)
- `<col>` = Column number (0-31)
- `<press>` = 1 for press, 0 for release
- `F7` = SysEx end

## Grid Layout

### Rows
- **Rows 0-6**: Sequencer rows (musical patterns)
- **Row 7**: Control row (ARM buttons, special functions)

### Columns
- **Columns 0-15**: Grid One (left grid)
- **Columns 16-31**: Grid Two (right grid)

## ARM Actions (Row 7)

| Column | Action | Constant | Description |
|--------|--------|----------|-------------|
| 0 | Undo | `ARM_UNDO` | Undo last action |
| 1 | Redo | `ARM_REDO` | Redo last undone action |
| 2 | Temporary step | | Hold + press a step: an empty step plays for the next 2 passes of its row, a lit step is muted for 2 passes; press again to cancel. Not saved, not undoable, cleared on stop |
| 3 | Preset Grid | `ARM_PRESET_GRID` | Hold + press a step: apply that row's preset (each row has its own bank: kick, open hat, closed hat, clap, accent, synth lines). Step 1 clears the row, step 2 is the standard techno pattern, then denser and more unusual; most presets on grid two are two-bar patterns with a fill. A row at 8th-note triplets gets 12-step triplet presets from a separate bank (last step set to 12) |
| 4 | Euclidean Events | `ARM_EUCLIDEAN_EVENTS` | Set number of events in euclidean pattern |
| 5 | Euclidean Length | `ARM_EUCLIDEAN_LENGTH` | Set length of euclidean pattern. The row then plays from where it would be had it played that length since MIDI Start, so it stays in time |
| 6 | Euclidean Rotation | `ARM_EUCLIDEAN_ROTATION` | Rotate euclidean pattern |
| 7 | Ratchet | `ARM_RATCHET` | Hold + press a step: off or normal -> style 1 (2 hits) -> style 2 (3 hits) -> style 3 (4 hits) -> normal step. Saved and undoable |
| 8 | Set Max Step | `ARM_SET_MAX_STEP` | Set row max_step (0-indexed), make the row straight, and clear its Jump Loop and row reset. The row then plays from where it would be had it played that length since MIDI Start (its own row reset included), so it stays in time. Undoable as one step |
| 9 | (none) | | Not used. The 8th-note triplet last-step button moved to grid two, row 7, column 15 (see below) |
| Grid two 15 | Set Max Step, 8th-note triplets | | Hold + press a step: set row max_step, make the row 8th-note triplets (8 clocks per step, 12 per bar), and clear its Jump Loop and row reset (for triplet Jump Loops, set this first, then add the segments). Undoable as one step |
| 10 | Row reset every 64 | | Hold + press a step: every 64 steps (counted from MIDI Start) the row goes to that step, even with a Jump Loop; the loop carries on in the first segment that has that step, or from the first segment if none has it. Replaces the row's 64 reset (normal or forced); the same button and step again keeps it. Undo or grid one column 8 or grid two column 15 on the row removes it; a Jump Loop segment doesn't. While held, each row's 64 reset is the only bright step. The button lights for one beat at each 64-step point while the clock runs. Saved and undoable |
| 11 | Row reset every 128 | | Same, every 128 steps, as a second reset beside the 64 one: with both, the row goes to the 128 step on the 128 points and to the 64 step on the 64 points in between. The button lights for one beat at each 128-step point |
| 12 | Jump Loop 1X | | Hold + press a start step, then an end step, on one row: add a segment played once (end before start plays backwards). A row holds up to 5 segments; a 6th drops the oldest, so entering the same segment 5 times replaces the whole loop |
| 13 | Jump Loop 2X | | Same, segment played twice |
| 14 | Jump Loop 3X | | Same, segment played three times |
| 15 | Row reset every 64, forced | | Like column 10, and replaces the row's 64 reset. On a Jump Loop row with the step outside every segment, the row plays forward from that step one step at a time, ignoring its loop, until it reaches a step inside a segment, and the loop carries on from there. It wraps at the last step; after a whole lap without reaching a segment (all beyond the last step) it joins the first segment. Lights at each 64-step point like column 10. To clear a row's Jump Loop, use grid one column 8 or grid two column 15 + a step on that row |

Grid two, row 7, columns 0 and 1 held together: MIDI clock re-detect and one quick Crow search; both lights full brightness if Crow is found, dark if not.

Grid two, row 7, column 3: Jump Loop from step 1. Hold + press step N on a row: add a segment from step 1 to step N, played once (step 32 = a 32-step segment). Undoable.

Grid two, row 7, column 4: Jump Loop remove newest. Hold + press any step on a row: remove that row's newest segment; when the last one goes, the row plays normally again, in time. Undoable.

Grid two, row 7, column 15: Set Max Step, 8th-note triplets (in the table above; moved from grid one column 9 so it isn't pressed by mistake).

Grid two, row 7, column 2: Jump Loop sync. Press to cycle off -> every 64 steps (dim) -> every 128 steps (bright) -> off. At each sync point, rows with a Jump Loop start again at their first segment. Saved with the pattern and undoable.

## Usage in Test Scripts

### Basic Button Press/Release

```json
{
  "command": "SysExButton",
  "row": 0,
  "col": 5,
  "press": true
}
```

### Setting Row Max Step

To set a row to wrap at step 15 (16 total steps: 0-15):

```json
{
  "commands": [
    {
      "command": "LogMilestone",
      "message": "Setting row 0 max_step to 15 (16 steps total)"
    },
    {
      "command": "SysExButton",
      "row": 7,
      "col": 8,
      "press": true,
      "comment": "Press ARM Set Max Step button"
    },
    {
      "command": "Wait",
      "ms": 50
    },
    {
      "command": "SysExButton",
      "row": 0,
      "col": 15,
      "press": true,
      "comment": "Press column 15 on row 0 (sets max_step to 15)"
    },
    {
      "command": "Wait",
      "ms": 50
    },
    {
      "command": "SysExButton",
      "row": 0,
      "col": 15,
      "press": false,
      "comment": "Release button"
    },
    {
      "command": "Wait",
      "ms": 50
    },
    {
      "command": "SysExButton",
      "row": 7,
      "col": 8,
      "press": false,
      "comment": "Release ARM button"
    }
  ]
}
```

### Setting Euclidean Pattern

To set row 1 with 5 events in 8 steps:

```json
{
  "commands": [
    {
      "command": "LogMilestone",
      "message": "Setting row 1 to euclidean 5/8"
    },
    {
      "command": "SysExButton",
      "row": 7,
      "col": 5,
      "press": true,
      "comment": "Press ARM Euclidean Length"
    },
    {
      "command": "Wait",
      "ms": 50
    },
    {
      "command": "SysExButton",
      "row": 1,
      "col": 7,
      "press": true,
      "comment": "Set length to 8 steps (col 7 = step 7 = length 8)"
    },
    {
      "command": "Wait",
      "ms": 50
    },
    {
      "command": "SysExButton",
      "row": 1,
      "col": 7,
      "press": false
    },
    {
      "command": "SysExButton",
      "row": 7,
      "col": 5,
      "press": false
    },
    {
      "command": "Wait",
      "ms": 100
    },
    {
      "command": "SysExButton",
      "row": 7,
      "col": 4,
      "press": true,
      "comment": "Press ARM Euclidean Events"
    },
    {
      "command": "Wait",
      "ms": 50
    },
    {
      "command": "SysExButton",
      "row": 1,
      "col": 4,
      "press": true,
      "comment": "Set 5 events (col 4 = step 4 = 5 events)"
    },
    {
      "command": "Wait",
      "ms": 50
    },
    {
      "command": "SysExButton",
      "row": 1,
      "col": 4,
      "press": false
    },
    {
      "command": "SysExButton",
      "row": 7,
      "col": 4,
      "press": false
    }
  ]
}
```

### Toggling Steps On/Off

To toggle step 8 on row 2:

```json
{
  "commands": [
    {
      "command": "SysExButton",
      "row": 2,
      "col": 8,
      "press": true
    },
    {
      "command": "Wait",
      "ms": 50
    },
    {
      "command": "SysExButton",
      "row": 2,
      "col": 8,
      "press": false
    }
  ]
}
```

## Column to Step/Value Mapping

When using ARM actions, the column number maps to values:

- **Set Max Step (ARM column 8)**: Column N = max_step N (0-indexed)
  - Column 0 = max_step 0 (1 step: only step 0)
  - Column 7 = max_step 7 (8 steps: 0-7)
  - Column 15 = max_step 15 (16 steps: 0-15)
  - Column 31 = max_step 31 (32 steps: 0-31) (use Grid Two, column 15)

- **Euclidean Length/Events**: Column N = Value N+1 (1-indexed)
  - Column 0 = 1 step/event
  - Column 7 = 8 steps/events
  - Column 15 = 16 steps/events
  - Column 31 = 32 steps/events (use Grid Two, column 15)

- **Euclidean Rotation**: Column N = Rotation N (0-indexed)
  - Column 0 = no rotation
  - Column 15 = rotate 15 steps
  - Column 31 = rotate 31 steps (use Grid Two, column 15)

## Tips

1. **Timing**: Add `Wait` commands between press and release (minimum 50ms recommended)
2. **ARM Actions**: Always release ARM buttons after use to exit the mode
3. **Two Grids**: Columns 0-15 are on Grid One, 16-31 on Grid Two
4. **Max Step**: max_step is 0-indexed (max_step 15 = 16 steps: 0-15)
5. **Euclidean**: Euclidean length/events are 1-indexed (column 7 = 8 steps/events)
6. **Verification**: Use `VerifyState` after setting parameters to confirm changes

## Example: Complete Test2

See `test2.json` for a complete example that:
1. Loads a pattern
2. Uses SysEx commands to configure row lengths
3. Verifies each row wraps at the correct length
4. Tests multiple bar cycles