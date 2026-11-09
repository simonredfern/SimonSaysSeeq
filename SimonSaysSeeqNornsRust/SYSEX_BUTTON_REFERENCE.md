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
| 5 | Euclidean Length | `ARM_EUCLIDEAN_LENGTH` | Set length of euclidean pattern |
| 6 | Euclidean Rotation | `ARM_EUCLIDEAN_ROTATION` | Rotate euclidean pattern |
| 7 | Ratchet | `ARM_RATCHET` | Hold + press a step: off or normal -> style 1 (2 hits) -> style 2 (3 hits) -> style 3 (4 hits) -> normal step. Saved and undoable |
| 8 | Set Max Step | `ARM_SET_MAX_STEP` | Set row max_step (0-indexed), make the row straight, and clear its Jump Loop |
| 9 | Set Max Step, 8th-note triplets | | Set row max_step and make the row 8th-note triplets (8 clocks per step, 12 per bar) |
| 10 | Triplet beat | | Hold + press a step: that step's beat (4 steps) plays as an 8th-note triplet on a straight row (steps 1-3 on the triplet grid, step 4 silent and dimmed); press again for straight. Saved and undoable |
| 11 | Set Max Step, quarter-note triplets | | Set row max_step and make the row quarter-note triplets (16 clocks per step, 6 per bar) |
| 12 | Jump Loop 1X | | Hold + press a start step, then an end step, on one row: add a segment played once (end before start plays backwards) |
| 13 | Jump Loop 2X | | Same, segment played twice |
| 14 | Jump Loop 3X | | Same, segment played three times |
| 15 | Jump Loop Reset | | Hold + press any step on a row: clear that row's Jump Loop |

Grid two, row 7, column 2: Jump Loop sync. Press to cycle off -> every 64 steps (dim) -> every 128 steps (bright) -> off. At each sync point, rows with a Jump Loop start again at their first segment. Saved with the pattern.

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