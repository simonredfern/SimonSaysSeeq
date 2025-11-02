# Grid Hotplug Detection - Test Scenarios

## Overview

This document describes test scenarios for verifying that grid hotplug detection works correctly in all situations, especially the **1→2 grid transition**.

## Test Scenarios

### Scenario 1: Start with 0 grids, plug in 1 grid

**Initial State:**
- No grids connected
- Sequencer running in headless mode

**Action:**
1. Plug in first grid (GRID_ONE)

**Expected Behavior:**
- Within 1-2 seconds: "🔌 serialosc hotplug: Grid connected!"
- "🔍 Hotplug detected - triggering automatic grid rediscovery..."
- "   Previous grid count: 0"
- "   New grid count: 1"
- "🔄 Grid count changed: 0 → 1"
- "✨ Grid [device_id] connected and flashed"
- Grid flashes 2 times quickly
- Grid immediately starts responding to button presses
- LEDs on that grid work

**What to Verify:**
- ✅ Grid detection happens automatically (no MIDI stop needed)
- ✅ Grid flashes to confirm connection
- ✅ Button presses register
- ✅ LEDs light up when pressed
- ✅ Only 1 grid is shown as connected

---

### Scenario 2: Start with 1 grid, plug in second grid ⭐ **KEY TEST**

**Initial State:**
- 1 grid connected (e.g., GRID_ONE)
- Sequencer running with partial grid control

**Action:**
1. Plug in second grid (GRID_TWO)

**Expected Behavior:**
- Within 1-2 seconds: "🔌 serialosc hotplug: Grid connected!"
- "🔍 Hotplug detected - triggering automatic grid rediscovery..."
- "   Previous grid count: 1"
- Device already connected gets skipped: "Device [existing_id] already connected, skipping reconnection"
- "   New grid count: 2"
- "🔄 Grid count changed: 1 → 2"
- "✨ Grid [new_device_id] connected and flashed"
- "💾 Saved grid configuration to ~/.config/simonsaysseeq/grid_config.json"
- "💾 Grid configuration saved: 2 grids now connected"
- "✅ Successfully connected to 2 grids"
- Second grid flashes 2 times quickly
- Both grids now fully functional

**What to Verify:**
- ✅ Second grid is detected automatically
- ✅ First grid is NOT reconnected (no double flash)
- ✅ Second grid flashes to confirm connection
- ✅ Configuration is saved (check file exists)
- ✅ Both grids respond to button presses
- ✅ LEDs work on both grids
- ✅ GRID_ONE and GRID_TWO are correctly assigned (alphabetically)
- ✅ No errors or warnings about reconnection
- ✅ Grid count shows 2

---

### Scenario 3: Start with 2 grids, unplug one grid

**Initial State:**
- 2 grids connected
- Full grid control active

**Action:**
1. Unplug GRID_TWO

**Expected Behavior:**
- Within 1-2 seconds: "🔌 serialosc hotplug: Grid disconnected!"
- "🔍 Hotplug detected - triggering automatic grid rediscovery..."
- "   Previous grid count: 2"
- "   New grid count: 1"
- "🔄 Grid count changed: 2 → 1"
- "Connected to 1 grid(s) - recommended: 2"
- Remaining grid (GRID_ONE) continues to work
- No flashing (device wasn't added)

**What to Verify:**
- ✅ Disconnection is detected automatically
- ✅ Remaining grid continues to function
- ✅ No crashes or errors
- ✅ Grid count shows 1
- ✅ Application doesn't try to send LEDs to disconnected grid

---

### Scenario 4: Start with 2 grids, unplug one, then replug it

**Initial State:**
- 2 grids connected
- Full grid control active

**Action:**
1. Unplug GRID_TWO
2. Wait 2-3 seconds
3. Plug GRID_TWO back in

**Expected Behavior:**

**After unplug:**
- Disconnection detected as in Scenario 3
- Grid count: 1

**After replug:**
- "🔌 serialosc hotplug: Grid connected!"
- "🔍 Hotplug detected - triggering automatic grid rediscovery..."
- "   Previous grid count: 1"
- First grid skipped (already connected)
- "   New grid count: 2"
- "🔄 Grid count changed: 1 → 2"
- "✨ Grid [device_id] connected and flashed"
- "💾 Grid configuration saved: 2 grids now connected"
- Replugd grid flashes 2 times
- Both grids fully functional

**What to Verify:**
- ✅ Replug is detected (same as Scenario 2)
- ✅ Same device ID is recognized
- ✅ Grid assignment is consistent (same GRID_ONE/GRID_TWO as before)
- ✅ Configuration is updated/resaved
- ✅ Both grids work normally

---

### Scenario 5: Start with 0 grids, plug in 2 grids simultaneously

**Initial State:**
- No grids connected
- Sequencer running in headless mode

**Action:**
1. Plug in both grids at approximately the same time (within 1 second)

**Expected Behavior:**
- Two hotplug events received (may be rapid)
- "🔌 serialosc hotplug: Grid connected!" (x2, may overlap in logs)
- Rediscovery triggered (possibly twice, but safely)
- "   Previous grid count: 0" (or 1 if second event happens after first connection)
- "   New grid count: 1" or "   New grid count: 2"
- "🔄 Grid count changed: 0 → 1" or "1 → 2"
- Both grids flash (2 times each)
- "💾 Grid configuration saved: 2 grids now connected"
- "✅ Successfully connected to 2 grids"

**What to Verify:**
- ✅ Both grids are detected
- ✅ No crashes from rapid events
- ✅ Both grids flash
- ✅ Configuration saved
- ✅ Grid count shows 2
- ✅ Both grids fully functional

---

### Scenario 6: Start with 0 grids saved config, plug in grids in wrong order

**Initial State:**
- No grids connected
- Saved config exists: GRID_ONE=m1000456, GRID_TWO=m1000123
- Sequencer running in headless mode

**Action:**
1. Plug in m1000123 first (which was GRID_TWO in saved config)
2. Plug in m1000456 second (which was GRID_ONE in saved config)

**Expected Behavior:**
- Both grids detected and connected
- Grid assignment is **alphabetical** (not based on saved config):
  - m1000123 → GRID_ONE (alphabetically first)
  - m1000456 → GRID_TWO (alphabetically second)
- Configuration is **updated** with new assignment
- Both grids work, but roles may have swapped

**What to Verify:**
- ✅ Grid assignment is deterministic (alphabetical)
- ✅ Saved config is updated to match new assignment
- ✅ No errors about mismatched grid IDs
- ✅ Both grids fully functional in their new roles

---

## Debugging Tips

### If hotplug detection doesn't work:

1. **Check serialosc is running:**
   ```bash
   ps aux | grep serialosc
   ```

2. **Check logs for registration:**
   - Should see: "📡 Registered for serialosc hotplug notifications"
   - If missing: "Failed to register for serialosc hotplug notifications"

3. **Check if notifications are received:**
   - Enable debug logging
   - Look for: "🔌 serialosc hotplug: Grid connected/disconnected!"

4. **Manual fallback:**
   - Send MIDI stop to manually trigger rediscovery

### Common Issues:

- **Grid flashes twice every time button pressed:** Check if hotplug is triggering too often
- **Second grid not detected:** Check logs for "Device [id] already connected, skipping reconnection"
- **Grid count doesn't change:** Check if `discover_devices()` is being called
- **Config not saved:** Check for "Failed to save grid config after hotplug" warnings

---

## Expected Log Output for 1→2 Transition

```
[Previous state: 1 grid connected]
INFO  🔌 serialosc hotplug: Grid connected!
INFO  🔍 Hotplug detected - triggering automatic grid rediscovery...
INFO     Previous grid count: 1
DEBUG Device m1000123 already connected, skipping reconnection
INFO  ✨ Grid m1000456 connected and flashed
INFO  ✅ Successfully connected to 2 grids
INFO     New grid count: 2
INFO  🔄 Grid count changed: 1 → 2
INFO  💾 Saved grid configuration to /home/we/.config/simonsaysseeq/grid_config.json
INFO  💾 Grid configuration saved: 2 grids now connected
[LED flash happens on grid m1000456]
```

---

## Performance Notes

- Hotplug detection happens in the `read_button_events()` loop
- This loop runs continuously (called every frame/input cycle)
- Detection adds minimal overhead:
  - Just checking OSC message address string
  - Only triggers full rediscovery when hotplug detected
  - Rediscovery takes ~3 seconds max (includes timeouts)

---

## Files to Check After Testing

1. **Grid configuration file:**
   ```bash
   cat ~/.config/simonsaysseeq/grid_config.json
   ```
   Should contain:
   ```json
   {
     "grid_one_id": "m1000123",
     "grid_two_id": "m1000456"
   }
   ```

2. **Application logs:**
   - Check for hotplug detection messages
   - Verify no error messages
   - Confirm grid count changes are logged

---

## Success Criteria

All scenarios should pass with:
- ✅ Automatic detection (no manual trigger needed)
- ✅ Correct grid count after each action
- ✅ Visual confirmation (LED flash)
- ✅ Full functionality after connection
- ✅ No crashes or errors
- ✅ Configuration saved when reaching 2 grids
- ✅ Existing grids not reconnected unnecessarily