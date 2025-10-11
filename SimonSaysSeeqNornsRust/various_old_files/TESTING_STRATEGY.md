# SimonSaysSeeq Testing Strategy

## Overview

This document describes the comprehensive testing strategy for the SimonSaysSeeq sequencer, focusing on automated, reproducible tests that verify sequencer behavior through pattern loading and state verification.

## Testing Philosophy

Our testing approach consists of two complementary strategies:

1. **Pattern-Based Tests** - Load pre-configured patterns, advance the clock, verify state
2. **Button-Injection Tests** - Simulate user interactions through button file injection

## Pattern-Based Testing

### Concept

Pattern-based tests follow a simple three-step process:

1. **Arrange**: Load a pre-configured test pattern file (`test_pattern_1.json`, `test_pattern_2.json`, etc.)
2. **Act**: Advance the MIDI clock by a specific number of steps
3. **Assert**: Record and verify the actual sequencer state matches the expected state

### Advantages

- **Fast**: No button simulation overhead
- **Deterministic**: Same pattern + same clock advancement = same result
- **Isolated**: Tests specific sequencer behavior without UI complexity
- **Reproducible**: Exact same conditions every time
- **Version Control Friendly**: Test patterns can be committed to git

### Test Pattern File Structure

Test patterns are JSON files containing a specific sequencer configuration:

```json
{
  "sequencer_a_grid": [[0,0,0,...], [0,0,0,...], ...],
  "sequencer_a_mozart": [[0,0,0,...], [0,0,0,...], ...],
  "slide": [[0,0,0,...], [0,0,0,...], ...],
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
    },
    // ... more rows
  ],
  // ... other state fields
}
```

### Example Test Pattern Library

#### `test_pattern_1.json` - Multi-Length Pattern
- **Purpose**: Test modulo behavior with various row lengths
- **Configuration**:
  - Row 0: 32 steps
  - Row 1: 31 steps
  - Row 2: 30 steps
  - Row 3: 16 steps
  - Row 4: 15 steps
  - Row 5: 14 steps
- **Test Cases**:
  - Advance 33 steps, verify each row at correct position
  - Advance 64 steps, verify wrap-around behavior
  - Advance 420 steps (LCM of all lengths), verify all rows return to step 0

#### `test_pattern_2.json` - MIDI Trigger Pattern
- **Purpose**: Test MIDI note generation timing
- **Configuration**:
  - Row 0: Length 16, triggers at steps 0, 4, 8, 12
  - All other rows: Empty (no triggers)
- **Test Cases**:
  - Advance 16 steps, verify 4 MIDI note events at correct master steps
  - Advance 32 steps, verify 8 total MIDI events (2 cycles)

#### `test_pattern_3.json` - Euclidean Pattern
- **Purpose**: Test euclidean rhythm generation
- **Configuration**:
  - Row 0: Length 16, 5 events, rotation 0
  - Row 1: Length 16, 7 events, rotation 0
  - Row 2: Length 16, 3 events, rotation 4
- **Test Cases**:
  - Advance 16 steps, verify euclidean triggers match Bjorklund algorithm
  - Verify rotation offsets work correctly

#### `test_pattern_4.json` - Slide Pattern
- **Purpose**: Test slide/legato behavior
- **Configuration**:
  - Row 0: MIDI triggers with slide values
  - Various slide durations
- **Test Cases**:
  - Verify slide affects note duration
  - Verify slide doesn't affect non-adjacent notes

#### `test_pattern_5.json` - Mozart (Probability) Pattern
- **Purpose**: Test probability system
- **Configuration**:
  - Row 0: Various probability values (0, 25, 50, 75, 100)
- **Test Cases**:
  - Run 1000 cycles, verify probability distribution
  - Verify 0% never triggers
  - Verify 100% always triggers

#### `test_pattern_6.json` - Ratchet Pattern
- **Purpose**: Test ratcheting/note repetition
- **Configuration**:
  - Row 0: Various ratchet counts (1, 2, 3, 4)
- **Test Cases**:
  - Verify correct number of MIDI notes per step
  - Verify ratchet timing is correct

## Button-Injection Testing

### Concept

Button-injection tests simulate user interactions by writing to `button_a.txt` and `button_b.txt` files that the sequencer monitors. This allows testing of:

- ARM actions (SetSeqALength, SetSeqAEvents, etc.)
- Pattern editing during playback
- Complex user interaction sequences

### Advantages

- **Integration Testing**: Tests the complete button → action → state pipeline
- **Real-World Scenarios**: Simulates actual user behavior
- **ARM Function Testing**: Tests ARM button combinations

### Example Test Cases

- Change row length during playback, verify MIDI timing adjusts
- Set euclidean events, verify rhythm changes
- Rotate euclidean pattern, verify phase shift

## State Verification

### What to Record

After advancing the clock, we record:

1. **Step Counters**:
   - Master step position
   - Each row's current step
   - Each row's previous step

2. **MIDI Events**:
   - Note On events (note, velocity, channel, timestamp)
   - Note Off events (note, timestamp)
   - Total event count

3. **Row States**:
   - Euclidean length
   - Euclidean events
   - Euclidean rotation
   - First step
   - Current MIDI note settings

4. **Timing Information**:
   - Tick count
   - Steps advanced
   - Expected vs actual timing

### State Comparison

Tests should compare:

```rust
struct ExpectedState {
    master_step: usize,
    row_steps: Vec<usize>,
    midi_events: Vec<MidiEvent>,
    row_lengths: Vec<usize>,
}

struct ActualState {
    master_step: usize,
    row_steps: Vec<usize>,
    midi_events: Vec<MidiEvent>,
    row_lengths: Vec<usize>,
}

fn verify_state(expected: &ExpectedState, actual: &ActualState) -> TestResult {
    // Compare each field
    // Return pass/fail with detailed diff
}
```

## Test Execution Flow

### Pattern-Based Test Flow

```
1. Load test pattern file → sequencer state
2. Start MIDI clock
3. Send N clock ticks (N steps × 6 ticks per step)
4. Stop MIDI clock
5. Read formal_state.log
6. Extract StepAdvancement events
7. Extract MidiNoteOn/Off events
8. Compare actual vs expected state
9. Report pass/fail with detailed diff
```

### Test Script Format

```json
{
  "name": "Test1: Multi-Length Pattern",
  "pattern_file": "test_pattern_1.json",
  "bpm": 120.0,
  "commands": [
    {
      "command": "LoadPattern",
      "file": "test_pattern_1.json"
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
    },
    {
      "command": "VerifyState",
      "expected": {
        "master_step": 33,
        "row_steps": [1, 2, 3, 1, 3, 5],
        "row_lengths": [32, 31, 30, 16, 15, 14]
      }
    }
  ]
}
```

## Implementation Roadmap

### Phase 1: Test Pattern Creation ✅
- [x] Document testing strategy
- [x] Create `test_pattern_1.json` (multi-length pattern)
- [x] Add `LoadPattern` command to automated_test_clock
- [x] Create test_patterns directory with README
- [ ] Create `test_pattern_2.json` through `test_pattern_6.json`

### Phase 2: State Recording
- [ ] Add `LoadPattern` command to TestCommand enum
- [ ] Implement state extraction from formal_state.log
- [ ] Create state comparison functions

### Phase 3: Verification
- [ ] Implement `VerifyState` command
- [ ] Add expected state to test scripts
- [ ] Generate pass/fail reports with diffs

### Phase 4: Test Suite
- [ ] Create comprehensive test suite
- [ ] Add CI/CD integration
- [ ] Generate test coverage reports

## Expected Test Output

```
🧪 Running Test1: Multi-Length Pattern
  ✅ Pattern loaded: test_pattern_1.json
  ✅ MIDI clock started
  ⏱️  Advanced 33 steps (198 ticks)
  ✅ MIDI clock stopped
  📊 Verifying state...
  
  Expected vs Actual:
  ✅ Master step: 33 = 33
  ✅ Row 0 step: 1 = 1 (33 % 32)
  ✅ Row 1 step: 2 = 2 (33 % 31)
  ✅ Row 2 step: 3 = 3 (33 % 30)
  ✅ Row 3 step: 1 = 1 (33 % 16)
  ✅ Row 4 step: 3 = 3 (33 % 15)
  ✅ Row 5 step: 5 = 5 (33 % 14)
  
  ✅ TEST PASSED
```

## Benefits

1. **Regression Testing**: Ensure bug fixes don't break existing functionality
2. **Refactoring Confidence**: Safely refactor knowing tests will catch issues
3. **Documentation**: Tests serve as executable specifications
4. **Bug Reproduction**: Convert bug reports into failing tests
5. **Continuous Integration**: Automated testing on every commit

## Mathematical Verification

For pattern-based tests, we can mathematically predict expected outcomes:

### Row Step Position After N Steps
```
row_step[i] = N % row_length[i]
```

### MIDI Events for Simple Trigger Pattern
```
If trigger at row_step S:
  Event occurs at master_step M when:
    M % row_length == S
```

### Euclidean Pattern Verification
Use Bjorklund algorithm to generate expected pattern, compare with actual triggers.

## Conclusion

This dual approach (pattern-based + button-injection) provides comprehensive test coverage:

- **Pattern-based tests** verify core sequencer logic and timing
- **Button-injection tests** verify user interaction and ARM functions
- **State verification** ensures deterministic, reproducible behavior

All tests run without hardware, making them perfect for:
- Local development
- CI/CD pipelines
- Automated regression testing
- Bug reproduction and verification