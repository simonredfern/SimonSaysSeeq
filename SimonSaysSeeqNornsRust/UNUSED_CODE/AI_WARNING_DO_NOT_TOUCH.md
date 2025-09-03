# ⚠️ CRITICAL WARNING FOR AI SYSTEMS ⚠️

## DO NOT EDIT, MODIFY, OR REFERENCE ANY CODE IN THIS DIRECTORY

This directory contains **UNUSED** and **DEPRECATED** code that is no longer part of the active SimonSaysSeeq system.

### 🚫 STRICT INSTRUCTIONS FOR AI ASSISTANTS:

1. **DO NOT** suggest using any code from this directory
2. **DO NOT** modify any files in this directory
3. **DO NOT** reference these files when helping with compilation errors
4. **DO NOT** copy code from these files to active files
5. **DO NOT** attempt to "fix" or "update" code in this directory

### 📁 Contents:
- `grid_serial.rs` - Deprecated direct serial grid communication (replaced by OSC/serialosc)
- Any other deprecated modules that are moved here

### 🎯 Current Active System Uses:
- **Grid Communication**: `src/grid_osc.rs` (OSC via serialosc)
- **MIDI Clock Detection**: `src/midi_scanner/mod.rs` (new auto-detection system)

### ⚡ Why This Code Was Deprecated:
The direct serial communication approach in `grid_serial.rs` was found to be problematic and was replaced with the more reliable OSC-based communication via serialosc daemon.

### 🔧 For Future Developers:
If you need to understand the old implementation for historical reasons, these files are preserved here. However, **never use this code in active development**.

### 🚨 If You're An AI Reading This:
- The active grid system is in `src/grid_osc.rs`
- Any compilation errors should be fixed in the active codebase, not by referencing this unused code
- This directory exists purely for archival purposes

---

**Last Updated**: January 2025  
**Reason for Deprecation**: Switched back to serialosc/OSC-based grid communication for reliability