# Resilient Grid Startup - Implementation Documentation

## Overview

SimonSaysSeeq now supports resilient startup when grids are not fully connected. The sequencer will start even if 0 or 1 grids are detected, using the last known configuration from when 2 grids were present.

**Grid Rediscovery**: Grid rediscovery happens **automatically via USB hotplug detection**. When you plug in a grid, serialosc sends a notification and the sequencer immediately detects it. Additionally, you can manually trigger rediscovery by sending **MIDI stop**.

**Visual Feedback**: When a grid is detected (at startup or during rediscovery), it **flashes 2 times quickly** (150ms timing) to provide immediate visual confirmation of successful connection.

## Changes Made

### 1. Grid Configuration Persistence

**File**: `src/grid_osc.rs`

#### New Fields in `GridManager`
- `last_known_grid_ids: Option<(String, String)>` - Stores the last known grid IDs when 2 grids were connected
- `config_path: PathBuf` - Path to the persistent configuration file

#### New Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GridConfig {
    grid_one_id: String,
    grid_two_id: String,
}
```

#### Configuration Storage Location
- Linux/macOS: `~/.config/simonsaysseeq/grid_config.json`
- Fallback: `./grid_config.json` (current directory)

### 2. Graceful Grid Detection

#### Modified `GridManager::new()`
- Loads last known grid configuration from disk
- Logs warnings (not errors) when grids are missing
- Provides clear status messages for:
  - 0 grids: Uses last known config or warns if no config exists
  - 1 grid: Identifies which grid is connected and which is missing
  - 2 grids: Normal operation, saves configuration

#### Modified `discover_devices()`
- Removed hard requirement for exactly 2 grids
- Changed from errors to warnings:
  - 1 grid: "Found 1 grid, but 2 grids are recommended for full operation"
  - 3+ grids: "Found N grids, but only 2 are supported"

### 3. Fallback Grid ID Methods

All grid ID methods now fall back to last known configuration:

#### `get_grid_one_id()` / `get_grid_two_id()`
- Returns actual grid ID if 2 grids connected
- Falls back to last known configuration if available
- Returns error only if no grids AND no saved config

#### `get_grid_ids_ordered()`
- Returns current grid IDs if 2 grids connected
- Falls back to last known configuration if available
- Returns error only if no grids AND no saved config

#### `is_grid_two()`
- Returns `false` if not exactly 2 grids connected (no transformation)
- Prevents crashes when LED operations occur with missing grids

### 4. Configuration Auto-Save

#### `save_grid_config()`
- Automatically saves configuration when 2 grids are detected
- Updates in-memory cache (`last_known_grid_ids`)
- Called from:
  - `new()` - After initial discovery
  - `refresh()` - When grid count changes from non-2 to 2

### 5. Dynamic Grid Rediscovery

#### Automatic Hotplug Detection
- **serialosc hotplug notifications**: GridManager registers with serialosc to receive device add/remove events
- When a grid is plugged in: serialosc sends `/serialosc/add` → automatic rediscovery triggered
- When a grid is unplugged: serialosc sends `/serialosc/remove` → automatic rediscovery triggered
- Happens in `read_button_events()` loop (called frequently)
- **Zero user action required** - grids just work when plugged in

#### `refresh()`
- Returns immediately (no-op for OSC)
- Does NOT perform grid rediscovery
- Called frequently by the main loop without performance impact

#### `rediscover_grids()`
- Performs actual grid rediscovery
- **Triggered by**: USB hotplug events OR MIDI stop
- Detects newly connected grids
- Auto-saves configuration when 2 grids become available
- Logs grid count changes
- Can be manually triggered via MIDI stop as fallback

### 6. Visual Feedback on Grid Connection

#### LED Flash on Connection
- When a grid is successfully connected, it flashes **2 quick times** (150ms timing)
- Provides immediate visual confirmation that the grid was detected
- Happens during:
  - Initial startup (if grids are connected)
  - Rediscovery via MIDI stop (if new grids are found)
- Flash uses full brightness for varibright grids, on/off for non-varibright
- Non-blocking - uses `flash_grid_with_timing()` method

### 7. Updated Main Application

**File**: `src/main.rs`

#### Modified `SimonSaysSeeq::run()`
- Handles 0, 1, 2, or 3+ grids gracefully
- Provides detailed status for each case:
  - Shows which grids are connected/missing
  - Indicates when using last known configuration
  - Clear warnings about operational limitations

## Behavior by Grid Count

All scenarios include **LED flash feedback** when grids are detected.

### 0 Grids Connected
```
⚠️  No grids detected at startup
   Will use last known configuration: GRID_ONE=..., GRID_TWO=...
   📝 Sequencer will run normally and generate MIDI output
   🎛️  Grid control and LED feedback disabled
   🔌 Plug in grids - they will be detected automatically
🎛️  GRID ASSIGNMENT: No grids connected
   Using last known configuration:
   GRID_ONE: ... (not connected)
   GRID_TWO: ... (not connected)
📝 Sequencer WILL run and generate MIDI output normally
🎛️  Grid control disabled - no button input or LED feedback
🔌 Plug in grids - they will be detected automatically
```
</parameter>

**Important**: When you plug in a grid:
- ✅ **Automatically detected** within seconds via serialosc hotplug
- ✅ Grid flashes 2 times to confirm connection
- ✅ Immediately starts working (buttons and LEDs)
- ✅ No MIDI stop required
No LED flashes (no grids to flash).

**Important**: The sequencer continues to run normally:
- ✅ MIDI notes are generated and sent
- ✅ MIDI clock sync works
- ✅ Patterns play back
- ✅ Crow CV output works (if enabled)
- ❌ No grid button input
- ❌ No grid LED feedback

### 1 Grid Connected
```
⚠️  Only 1 grid detected at startup (expected 2)
   Connected: ...
   ✨ Grid ... connected and flashed
   Will use last known configuration: GRID_ONE=..., GRID_TWO=...
   Missing: GRID_TWO (...)
   📝 Sequencer will run with partial grid control
   🔌 Plug in second grid - it will be detected automatically
🎛️  GRID ASSIGNMENT: Only 1 grid connected (expected 2)
   Connected: ...
   Using last known configuration:
   GRID_ONE: ... ✓
   GRID_TWO: ... (not connected)
📝 Sequencer WILL run and generate MIDI output normally
⚠️  Partial grid control - only connected grid will respond
🔌 Plug in second grid - it will be detected automatically
```
The connected grid will flash 2 times quickly.

**Important**: When you plug in the second grid:
1. Plug in the missing grid
2. **Automatic detection** happens within seconds
3. Grid flashes 2 times to confirm
4. Grid immediately starts working - no further action needed

### 2 Grids Connected (Normal)
```
✨ Grid ... connected and flashed
✨ Grid ... connected and flashed
✅ Successfully connected to 2 grids
💾 Saved grid configuration to ~/.config/simonsaysseeq/grid_config.json
🎛️  GRID ASSIGNMENT:
   GRID_ONE: ...
   GRID_TWO: ...
✅ Two real grids ready for operation
```
Both grids will flash 2 times quickly in sequence.

## Benefits

1. **Loose Connection Tolerance**: Sequencer starts even with loose connections at startup
2. **Automatic Hotplug Detection**: Grids are detected instantly when plugged in - no manual trigger needed
3. **Hot Reconnection**: Plug and play - grids work immediately without restarting
4. **Consistent Grid Assignment**: Last known configuration ensures consistent GRID_ONE/GRID_TWO assignment
5. **Clear User Feedback**: Detailed warnings show exactly which grids are missing
6. **Visual Confirmation**: Grids flash when detected, providing immediate feedback
7. **No Crashes**: LED operations gracefully handle missing grids without panicking
8. **Manual Fallback**: Can manually trigger detection via MIDI stop if needed

## Limitations

- Button presses from disconnected grids won't be registered
- LED updates to disconnected grids are silently ignored
- If no grids have ever been connected, configuration will be unavailable
- Grid flashing during connection takes ~600ms (2 flashes × 150ms on + 150ms off)
- Hotplug detection requires serialosc daemon to be running (standard on Norns)

## Performance Considerations

- `refresh()` is called very frequently (multiple times per second after LED updates) - no impact
- Grid rediscovery is **event-driven via serialosc notifications**:
  - No polling overhead
  - Instant detection when grids are plugged in
  - Minimal OSC communication (only when devices change)
- `read_button_events()` checks for hotplug messages in the same loop as button events
- **Automatic detection** happens naturally without disrupting sequencing
- MIDI stop can still be used as manual fallback trigger

## Future Enhancements

Potential improvements:
- Visual notification on screen when grids reconnect during operation
- Manual grid refresh command via button combination (as alternative to MIDI stop)
- Configuration UI for manual grid assignment
- Grid connection history/logging

## Testing

To test resilient startup:

1. **With 2 grids**: Normal operation, configuration saved, both grids flash on startup
2. **Disconnect 1 grid and restart**: Should use last config, show warnings, connected grid flashes
3. **Disconnect both grids and restart**: Should use last config, show warnings, no flashes
4. **Hot reconnect while running**: 
   - Plug in the grid(s)
   - **Automatic detection** happens within 1-2 seconds
   - **Watch for the LED flash** - this confirms the grid was detected
   - Grid immediately starts working - buttons and LEDs active
   - No interruption to sequencing or MIDI output
   - **Fallback**: If detection doesn't happen, send MIDI stop to trigger manually

## Technical Notes

- Configuration is JSON for easy manual editing if needed
- `serde_json` handles serialization/deserialization
- File writes are atomic (temporary file + rename pattern via `fs::write`)
- Grid ID ordering is alphabetical for consistency
- No virtual/mock grids are created - only real hardware or degraded operation
- Uses serialosc `/serialosc/notify` protocol for hotplug notifications
- Hotplug messages (`/serialosc/add` and `/serialosc/remove`) processed in button event loop
- MIDI stop available as manual fallback if hotplug detection fails