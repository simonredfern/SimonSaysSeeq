# SimonSaysSeeq Pattern JSON Structure

This document describes the structure of the `current_pattern.json` file that saves and loads sequencer patterns.

## File Location
- **Linux**: `~/.config/simon-says-seeq/current_pattern.json`
- **Fallback**: `simon_says_seeq_current_pattern.json` (current directory)

## JSON Structure Overview

The JSON file contains a single object with all sequencer state needed to recreate the pattern:

```json
{
  "sequencer_a_grid": [...],           // Main sequencer grid (32x8)
  "sequencer_a_mozart": [...],         // Mozart performance grid (32x8) 
  "sequencer_a_mozart_grid": [...],    // Mozart MIDI note assignments
  "sequencer_a_row_states": [...],     // Per-row settings (8 rows)
  "slide": {...},                      // Slide/transition states
  "pattern_chains": [...],             // Pattern chain data
  "held": [...],                       // Grid hold states
  "tempo": 30.0,                       // Tempo in BPM
  "swing_amount": 0.0,                 // Swing timing
  "steps_per_bar": 16,                 // Steps per bar
  "first_step": 0,                     // Pattern start step
  "last_step": 31,                     // Pattern end step
  // ... plus timing, analysis, and config data
}
```

## Key Data Sections

### Main Sequencer Grid (`sequencer_a_grid`)
32x8 array representing the main pattern grid:
- **Dimensions**: 32 columns (steps) × 8 rows (tracks)
- **Values**: 
  - `0` = Off/empty
  - `1` = Normal hit
  - `2+` = Ratcheted hits (2=double, 3=triple, etc.)

**Example:**
```json
"sequencer_a_grid": [
  [1, 0, 0, 0, 0, 0, 0, 0],  // Column 0: Kick on row 0
  [0, 0, 0, 0, 0, 0, 0, 0],  // Column 1: Empty
  [0, 2, 0, 0, 0, 0, 0, 0],  // Column 2: Ratcheted hi-hat on row 1
  // ... 29 more columns
]
```

### Mozart Performance Grid (`sequencer_a_mozart`)
32x8 array for MIDI note values:
- **Dimensions**: 32 columns (steps) × 8 rows (tracks)
- **Values**: MIDI note numbers (0-127, where 0=off)
- **Common notes**: C4=60, C3=48, C5=72

**Example:**
```json
"sequencer_a_mozart": [
  [60, 36, 0, 0, 0, 0, 0, 0],  // Column 0: C4 melody, C2 bass
  [0, 0, 0, 0, 0, 0, 0, 0],    // Column 1: Empty
  [62, 0, 0, 0, 0, 0, 0, 0],   // Column 2: D4 melody
  // ... 29 more columns
]
```

### Row States (`sequencer_a_row_states`)
Array of 8 objects, one per track/row:

```json
"sequencer_a_row_states": [
  {
    "sequencer_a_current_step": 0,        // Current playback position
    "sequencer_a_first_step": 0,          // Row start step
    "sequencer_a_euclidean_length": 31,   // Euclidean pattern length
    "sequencer_a_euclidean_events": 5,    // Euclidean events count
    "sequencer_a_euclidean_rotation": 0,  // Euclidean rotation offset
    "sequencer_a_previous_step": 31,      // Previous step position
    "sequencer_a_midi_note": 48,          // MIDI note for this row (C3)
    "sequencer_a_midi_velocity": 100,     // Velocity (0-127)
    "sequencer_a_midi_channel": 1,        // MIDI channel (1-16)
    "sequencer_a_ratchet_count": 1        // Ratchet multiplier
  },
  // ... 7 more row objects
]
```

### Slide States (`slide`)
Parameter transitions and scrolling:

```json
"slide": {
  "slide_grid": [[0.0, 0.0, ...], ...],  // 32x8 slide amounts per position
  "scroll_offset": [0, 0]                  // X, Y scroll offset
}
```

### Transport/Timing Settings
Core sequencer parameters:

```json
"tempo": 30.0,              // BPM
"swing_amount": 0.0,        // Swing timing (0.0-1.0)
"swing_mode": 0,            // Swing mode type
"steps_per_bar": 16,        // Steps per bar
"ticks_per_step": 12,       // MIDI ticks per step
"first_step": 0,            // Pattern start boundary
"last_step": 31,            // Pattern end boundary
```

### Advanced Features
```json
"pattern_chains": [],                    // Pattern chain entries
"global_transpose": 0,                   // Global transpose (semitones)
"global_velocity_scale": 1.0,           // Global velocity scaling
"chain_mode_enabled": false,             // Pattern chain mode
"tempo_analysis": {...},                 // Tempo stability analysis
```

## What Gets Saved vs. Not Saved

### ✅ Saved
- All pattern data (grids, MIDI notes, ratchets)
- Row configurations (MIDI settings, euclidean patterns)
- Global settings (tempo, swing, transpose)
- Advanced features (chains, slides, analysis settings)

### ❌ Not Saved (Reset on Load)
- Current playback position (`is_running = false`)
- Real-time counters (`tick_count = 0`)
- Transport state (always starts stopped)

## Usage Notes

- File is automatically created/updated when sequencer stops
- File is automatically loaded when sequencer starts up  
- If file doesn't exist, creates default sparse pattern
- JSON is human-readable and can be manually edited
- File size typically 15-25KB depending on pattern complexity
- Uses pretty-printed JSON for readability

## MIDI Note Reference
Common MIDI note numbers for reference:
- C2: 36 (bass)
- C3: 48 (kick/low)  
- C4: 60 (middle C)
- C5: 72 (melody/high)
- Range: 0-127 (0 = off/silent)