# Resilient Grid Startup - Implementation Documentation

## Overview

SimonSaysSeeq now supports resilient startup when grids are not fully connected. The sequencer will start even if 0 or 1 grids are detected, using the last known configuration from when 2 grids were present.

**Grid Rediscovery Trigger**: Grid rediscovery is triggered **only on MIDI stop events**. This ensures minimal performance impact during playback while providing a natural opportunity to detect newly connected grids when the sequencer stops.

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

#### `refresh()`
- Returns immediately (no-op for OSC)
- Does NOT perform grid rediscovery (see `rediscover_grids()` below)
- Called frequently by the main loop without performance impact

#### `rediscover_grids()`
- Performs actual grid rediscovery
- **Triggered only on MIDI stop events**
- Detects newly connected grids
- Auto-saves configuration when 2 grids become available
- Logs grid count changes
- Natural pause point that won't disrupt playback

### 6. Updated Main Application

**File**: `src/main.rs`

#### Modified `SimonSaysSeeq::run()`
- Handles 0, 1, 2, or 3+ grids gracefully
- Provides detailed status for each case:
  - Shows which grids are connected/missing
  - Indicates when using last known configuration
  - Clear warnings about operational limitations

## Behavior by Grid Count

### 0 Grids Connected
```
⚠️  No grids detected at startup
   Will use last known configuration: GRID_ONE=..., GRID_TWO=...
🎛️  GRID ASSIGNMENT: No grids connected
   Using last known configuration:
   GRID_ONE: ... (not connected)
   GRID_TWO: ... (not connected)
⚠️  Running without hardware - button presses and LEDs will be ignored
```

### 1 Grid Connected
```
⚠️  Only 1 grid detected at startup (expected 2)
   Connected: ...
   Will use last known configuration: GRID_ONE=..., GRID_TWO=...
   Missing: GRID_TWO (...)
🎛️  GRID ASSIGNMENT: Only 1 grid connected (expected 2)
   Connected: ...
   Using last known configuration:
   GRID_ONE: ... ✓
   GRID_TWO: ... (not connected)
⚠️  Partial operation - some button presses and LEDs may not work
```

### 2 Grids Connected (Normal)
```
✅ Successfully connected to 2 grids
💾 Saved grid configuration to ~/.config/simonsaysseeq/grid_config.json
🎛️  GRID ASSIGNMENT:
   GRID_ONE: ...
   GRID_TWO: ...
✅ Two real grids ready for operation
```

## Benefits

1. **Loose Connection Tolerance**: Sequencer starts even with loose connections at startup
2. **Hot Reconnection**: Use `refresh()` to detect newly connected grids without restarting
3. **Consistent Grid Assignment**: Last known configuration ensures consistent GRID_ONE/GRID_TWO assignment
4. **Clear User Feedback**: Detailed warnings show exactly which grids are missing
5. **No Crashes**: LED operations gracefully handle missing grids without panicking

## Limitations

- Button presses from disconnected grids won't be registered
- LED updates to disconnected grids are silently ignored
- If no grids have ever been connected, configuration will be unavailable

## Performance Considerations

- `refresh()` is called very frequently (multiple times per second after LED updates)
- Grid rediscovery is **only triggered on MIDI stop** to avoid:
  - Excessive OSC communication overhead
  - Potential disruption to ongoing grid operations
  - Performance degradation during playback
- MIDI stop is a natural pause point where rediscovery won't interrupt sequencing
- Users can trigger rediscovery by stopping/starting MIDI clock

## Future Enhancements

Potential improvements:
- Additional rediscovery triggers (e.g., on transport button press)
- User notification when grids reconnect during operation
- Manual grid refresh command via button combination
- Configuration UI for manual grid assignment
- Background thread for periodic checking (optional)

## Testing

To test resilient startup:

1. **With 2 grids**: Normal operation, configuration saved
2. **Disconnect 1 grid and restart**: Should use last config, show warnings
3. **Disconnect both grids and restart**: Should use last config, show warnings
4. **Reconnect grids while running**: Send MIDI stop to trigger rediscovery
   - Grid rediscovery happens when MIDI clock stops
   - Simply stop and restart MIDI clock to detect newly connected grids

## Technical Notes

- Configuration is JSON for easy manual editing if needed
- `serde_json` handles serialization/deserialization
- File writes are atomic (temporary file + rename pattern via `fs::write`)
- Grid ID ordering is alphabetical for consistency
- No virtual/mock grids are created - only real hardware or degraded operation