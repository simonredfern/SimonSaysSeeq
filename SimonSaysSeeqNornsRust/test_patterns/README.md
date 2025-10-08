# Test Pattern Files

This directory contains pre-configured sequencer patterns for automated testing. Each pattern is designed to test specific sequencer behaviors.

## Pattern Files

### `test_pattern_1.json` - Multi-Length Pattern Test
**Purpose**: Verify modulo behavior with various row lengths

**Configuration**:
- Row 0: 32 steps (full master length)
- Row 1: 31 steps
- Row 2: 30 steps
- Row 3: 16 steps (half master length)
- Row 4: 15 steps
- Row 5: 14 steps

**Test Cases**:
```bash
# Test 1: Run 33 steps, verify modulo wrapping
# Expected after 33 steps:
# Row 0: step 1 (33 % 32 = 1)
# Row 1: step 2 (33 % 31 = 2)
# Row 2: step 3 (33 % 30 = 3)
# Row 3: step 1 (33 % 16 = 1)
# Row 4: step 3 (33 % 15 = 3)
# Row 5: step 5 (33 % 14 = 5)

# Test 2: Run 64 steps, verify longer wrap-around
# Test 3: Run 420 steps (LCM of 14,15,16,30,31,32), verify all rows at step 0
```

### `test_pattern_2.json` - MIDI Trigger Pattern
**Purpose**: Test MIDI note generation timing

**Configuration**:
- Row 0: Length 16, MIDI triggers at steps 0, 4, 8, 12
- Rows 1-5: Empty (no triggers)

**Test Cases**:
- Advance 16 steps → verify 4 MIDI events at master steps 0, 4, 8, 12
- Advance 32 steps → verify 8 MIDI events (pattern repeats)
- Verify MIDI note timing is precise

### `test_pattern_3.json` - Euclidean Rhythm Test
**Purpose**: Verify euclidean rhythm generation

**Configuration**:
- Row 0: Length 16, 5 events, rotation 0
- Row 1: Length 16, 7 events, rotation 0
- Row 2: Length 16, 3 events, rotation 4

**Test Cases**:
- Verify euclidean distribution matches Bjorklund algorithm
- Verify rotation offset works correctly
- Compare actual triggers vs mathematically computed pattern

### `test_pattern_4.json` - Slide/Legato Test
**Purpose**: Test slide behavior and note duration

**Configuration**:
- Row 0: MIDI triggers with various slide values
- Adjacent notes with slide enabled
- Non-adjacent notes without slide

**Test Cases**:
- Verify slide affects note duration
- Verify slide creates legato between notes
- Verify non-slide notes have normal gate

### `test_pattern_5.json` - Probability (Mozart) Test
**Purpose**: Test probability system

**Configuration**:
- Row 0: Triggers with probabilities 0%, 25%, 50%, 75%, 100%

**Test Cases**:
- Run 1000 cycles, measure probability distribution
- Verify 0% never triggers
- Verify 100% always triggers
- Verify 50% triggers approximately 500 times

### `test_pattern_6.json` - Ratchet Test
**Purpose**: Test note repetition/ratcheting

**Configuration**:
- Row 0: Various ratchet counts (1, 2, 3, 4) at different steps

**Test Cases**:
- Verify correct number of MIDI notes per step
- Verify ratchet timing is evenly distributed
- Verify ratchet doesn't affect adjacent steps

## File Format

Test pattern files are JSON representations of `SequencerState`:

```json
{
  "sequencer_a_grid": [/* 2D array of trigger values */],
  "sequencer_a_mozart": [/* 2D array of probability values */],
  "slide": [/* 2D array of slide values */],
  "sequencer_a_row_states": [
    {
      "sequencer_a_current_step": 0,
      "sequencer_a_first_step": 0,
      "sequencer_a_euclidean_length": 32,
      "sequencer_a_euclidean_events": 0,
      "sequencer_a_euclidean_rotation": 0,
      "sequencer_a_previous_step": 0,
      "sequencer_a_midi_note": 60,
      "sequencer_a_midi_velocity": 100,
      "sequencer_a_midi_channel": 0,
      "sequencer_a_ratchet_count": 1
    }
    /* ... 5 more rows */
  ],
  /* ... other state fields */
}
```

## Creating New Test Patterns

### Method 1: From Current Pattern
```bash
# 1. Run the sequencer and create your desired pattern
# 2. Stop the sequencer (saves to current_pattern.json)
# 3. Copy and modify:
cp current_pattern.json test_patterns/test_pattern_7.json
# 4. Edit the JSON to set desired initial state
```

### Method 2: Python Script
```python
import json

# Load template
with open('current_pattern.json', 'r') as f:
    pattern = json.load(f)

# Modify as needed
pattern['sequencer_a_row_states'][0]['sequencer_a_euclidean_length'] = 16

# Save
with open('test_patterns/my_test.json', 'w') as f:
    json.dump(pattern, f, indent=2)
```

### Method 3: Manual Editing
Edit the JSON file directly. Key fields:
- `sequencer_a_euclidean_length`: Row length (1-32)
- `sequencer_a_euclidean_events`: Number of euclidean triggers (0-32)
- `sequencer_a_euclidean_rotation`: Euclidean rotation offset (0-31)
- `sequencer_a_grid[row][col]`: Trigger value (0=off, 1-15=on with brightness)
- `sequencer_a_mozart[row][col]`: Probability (0-100)
- `slide[row][col]`: Slide amount (0-15)

## Using Test Patterns

### With automated_test_clock
```bash
# Run a test that loads a pattern
cargo run --bin automated_test_clock -- --test1

# Or create a test script:
{
  "name": "My Test",
  "commands": [
    {
      "command": "LoadPattern",
      "file": "test_patterns/test_pattern_1.json"
    },
    {
      "command": "Start"
    },
    {
      "command": "WaitSteps",
      "count": 33
    },
    {
      "command": "Stop"
    }
  ]
}
```

### Manual Testing
```bash
# Copy test pattern to current pattern
cp test_patterns/test_pattern_1.json current_pattern.json

# Start sequencer - it will load the test pattern
cargo run --release --bin simon_says_seeq
```

## Verification

After running a test, check `formal_state.log`:

```bash
# View step advancement events
jq 'select(.event_type == "StepAdvancement")' formal_state.log

# View MIDI events
jq 'select(.event_type == "MidiNoteOn")' formal_state.log

# View ARM actions
jq 'select(.event_type == "ArmActionExecuted")' formal_state.log
```

## Best Practices

1. **Reset State**: Always set `is_running: false` and zero out step counters
2. **Document Purpose**: Add comments explaining what the pattern tests
3. **Minimal Patterns**: Only include triggers/settings needed for the test
4. **Version Control**: Commit test patterns to git for reproducibility
5. **Naming**: Use descriptive names: `test_pattern_<number>_<description>.json`

## Mathematical Verification

For length-based tests, verify using modulo arithmetic:
```
After N steps, row i should be at:
  expected_step[i] = N % row_length[i]
```

For euclidean tests, use the Bjorklund algorithm to compute expected pattern.

## Contributing

When adding new test patterns:
1. Create the pattern file in `test_patterns/`
2. Update this README with the pattern description
3. Add corresponding test cases to `automated_test_clock`
4. Document expected outcomes
5. Commit both pattern and documentation