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
| 4 | Euclidean Events | `ARM_EUCLIDEAN_EVENTS` | Set number of events in euclidean pattern |
| 5 | Euclidean Length | `ARM_EUCLIDEAN_LENGTH` | Set length of euclidean pattern |
| 6 | Euclidean Rotation | `ARM_EUCLIDEAN_ROTATION` | Rotate euclidean pattern |
| 7 | Ratchet | `ARM_RATCHET` | Set ratchet count (note repeats) |
| 8 | Set Length | `ARM_SET_LENGTH` | Set row length (max_step) |
| 10 | Preset Grid | `ARM_PRESET_GRID` | Apply preset pattern to row |

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

### Setting Row Length

To set a row to 16 steps (max_step = 15):

```json
{
  "commands": [
    {
      "command": "LogMilestone",
      "message": "Setting row 0 to 16 steps"
    },
    {
      "command": "SysExButton",
      "row": 7,
      "col": 8,
      "press": true,
      "comment": "Press ARM Set Length button"
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
      "comment": "Press step 15 on row 0 (sets max_step to 15 = 16 steps)"
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

- **Length/Events**: Column N = Value N+1
  - Column 0 = 1 step/event
  - Column 7 = 8 steps/events
  - Column 15 = 16 steps/events
  - Column 31 = 32 steps/events (use Grid Two, column 15)

- **Rotation**: Column N = Rotation N
  - Column 0 = no rotation
  - Column 15 = rotate 15 steps
  - Column 31 = rotate 31 steps (use Grid Two, column 15)

## Tips

1. **Timing**: Add `Wait` commands between press and release (minimum 50ms recommended)
2. **ARM Actions**: Always release ARM buttons after use to exit the mode
3. **Two Grids**: Columns 0-15 are on Grid One, 16-31 on Grid Two
4. **Max Step**: Setting length to 16 means max_step = 15 (0-15 = 16 steps total)
5. **Verification**: Use `VerifyState` after setting parameters to confirm changes

## Example: Complete Test2

See `test2.json` for a complete example that:
1. Loads a pattern
2. Uses SysEx commands to configure row lengths
3. Verifies each row wraps at the correct length
4. Tests multiple bar cycles