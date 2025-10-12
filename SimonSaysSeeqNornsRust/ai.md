# AI Instructions for SimonSaysSeeq Project

## Communication Style
- Be concise and direct
- Avoid excessive enthusiasm or chattiness
- Focus on technical facts and actionable information
- Use clear, professional language
- Do NOT create excessive documentation files unless explicitly requested
- Do NOT write lengthy summaries or guides after making simple changes
- Respond briefly - just state what was done and if it worked

## Logging Protocol
- Always update ai.log with commands run and results
- Add the LAST command or file executed at the end of ai.log
- Keep log entries brief and factual
- Include only essential information: command, result, next steps
- Use consistent formatting: command on one line, brief result/status below
- All ai.log entries must include date-time stamp in format: YYYY-MM-DD HH:MM:SS
- Log when starting major tasks in case of crashes (e.g. "Starting hardware mode test", "Beginning compilation", etc.)
- Do not use emojis in logs

## AI Log Rules
- ALWAYS use the current date/time for new ai.log entries
- Use the `now` tool to get the current datetime before creating log entries
- NEVER write over or modify existing log entries
- NEVER use past dates or estimated dates
- Always append new entries to the end of ai.log
- Format: ## YYYY-MM-DD HH:MM:SS Description

## Log Message Format Standards
- Prepend each log message with "function_name says: " where function_name is the actual function making the log
- Remove all tick boxes, emojis, and other decorative characters from log messages
- Use plain text only for clarity and consistency
- Apply this format to all info!, debug!, warn!, error! messages throughout the codebase
- Example: info!("run says: Starting sequencer for desktop testing");

## Development Workflow
1. Check ai.log first to understand current project state
2. Run necessary commands/tests
3. Always ask the user what worked when a grid is involved. Ask the user what they saw when a grid is involved.
4. Update ai.log with what was done
5. Provide concise summary to user

## Code Standards
- Prefer working solutions over perfect code
- Test changes incrementally
- Document significant findings in ai.log
- Use appropriate feature flags (--features desktop for grid testing)

## Grid Testing Notes
- serialosc must be running manually (system maybe service has permission issues)
- Both grids (m5032747, m2949672) work via OSC
- The prefered grid, original grid is m2949672
- Use examples/debug_osc.rs for isolated grid testing
- Comprehensive tests may give false negatives due to timing

## 0-Based Indexing Requirements
- ALL indexing in this codebase MUST be 0-based throughout
- Steps: 0-15 (16 total steps)
- Bars: 0-3 (4 total bars)
- Rows: 0-6 (7 sequence rows)
- Grid coordinates: 0-15 x 0-7 (16x8 grid)
- No index conversions (no "- 1" operations) should be needed
- All array access should use direct 0-based indices
- Any new functions must follow 0-based indexing consistently
- When fixing bugs, verify indexing is 0-based throughout the call chain

## Grid LED Update Constraints and Rules
- Grids have finite OSC bandwidth and will not respond correctly to excessive LED updates
- All grid LED writes must be carefully considered and targeted
- NEVER write code that spams LED updates - this will cause grid communication failures
- Use targeted LED updates based on the specific operation:
  - Single button press: Update ONLY the specific pressed LED
  - Current step change: Update ONLY old and new step positions
  - Euclidean operation: Update ONLY the affected row
  - Undo/redo operations: Update whole grids (acceptable for rare operations)
- Avoid full grid refreshes (update_grid_display) except during initialization
- Always prefer single LED updates (update_single_button_led) over full refreshes
- Each grid.refresh() call should be minimized and purposeful
- Test grid responsiveness after any LED update changes

## Mozart ARM Button Behavior
- Column 15: Mozart ARM action controls keyboard MIDI note display mode
- When Mozart ARM is pressed and held:
  - GRID_ONE displays keyboard_midi_note_events for steps 0-15
  - GRID_TWO displays keyboard_midi_note_events for steps 16-31
  - Shows recorded MIDI performance data as LED brightness
- When Mozart ARM is NOT active:
  - Both GRID_ONE and GRID_TWO show the main 32-step sequence
  - GRID_ONE shows sequence steps 0-15
  - GRID_TWO shows sequence steps 16-31
  - Current step indicator flows between grids as sequence plays

## BPM Testing Guidelines
- Focus tempo detection tests on the musical range: 60-150 BPM
- This covers the majority of musical genres and use cases:
  - 60-80 BPM: Ballads, slow songs
  - 80-100 BPM: Mid-tempo, hip-hop
  - 100-120 BPM: Pop, rock, dance
  - 120-150 BPM: Uptempo dance, electronic
- Use test cases: [60, 75, 90, 105, 120, 135, 150] BPM for comprehensive coverage
- Higher BPMs (150-300) are edge cases and less critical for musical applications
- Algorithm validation should prioritize accuracy in the 60-150 BPM range
