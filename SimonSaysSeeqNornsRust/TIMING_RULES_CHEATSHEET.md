# Timing Rules Cheat Sheet

## Quick Reference for Test Creation

### The Golden Rules

| Rule | Formula | Example | Why |
|------|---------|---------|-----|
| **Verify** | `(step × 6) + 3` | Step 5 → tick 33 | Log written during step |
| **SysEx Start** | `(step × 6) - 2` | Step 4 → tick 22 | Avoid step boundary |
| **Button Spread** | 3-4 ticks | 22, 23, 24, 25 | Realistic timing |
| **Avoid** | Multiples of 6 | 6, 12, 18, 24... | Step boundaries busy |

### Step Boundaries (AVOID for SysEx!)

```
Tick 0:   Step 0 boundary ⚠️
Tick 6:   Step 1 boundary ⚠️
Tick 12:  Step 2 boundary ⚠️
Tick 18:  Step 3 boundary ⚠️
Tick 24:  Step 4 boundary ⚠️
Tick 30:  Step 5 boundary ⚠️
...
```

### Safe Zones for Actions

```
Step N timeline:
├─ Tick N×6     : Boundary START ⚠️ BUSY!
├─ Tick N×6+1   : ✅ Safe for SysEx
├─ Tick N×6+2   : ✅ Safe for SysEx
├─ Tick N×6+3   : ✅ BEST for Verify
├─ Tick N×6+4   : ✅ Safe for SysEx
├─ Tick N×6+5   : ✅ Safe
└─ Tick N×6+6   : Next boundary ⚠️ BUSY!
```

## Example: Changing Row 0 max_step at Step 3

### ❌ BAD Timing (Conflicts!)
```json
{"at_tick": 18, "action": {"type": "SysExButton", ...}}  // Step 3 boundary!
{"at_tick": 24, "action": {"type": "VerifyState", ...}}  // Step 4 boundary!
```

### ✅ GOOD Timing (No Conflicts!)
```json
{"at_tick": 21, "action": {"type": "VerifyState", "step": 3, ...}},
{"at_tick": 22, "action": {"type": "SysExButton", "row": 7, "col": 8, "press": true}},
{"at_tick": 23, "action": {"type": "SysExButton", "row": 0, "col": 7, "press": true}},
{"at_tick": 24, "action": {"type": "SysExButton", "row": 0, "col": 7, "press": false}},
{"at_tick": 25, "action": {"type": "SysExButton", "row": 7, "col": 8, "press": false}},
{"at_tick": 27, "action": {"type": "VerifyState", "step": 4, ...}}
```

## Common Steps → Ticks

| Step | Boundary (AVOID) | Verify Here | SysEx Start |
|------|------------------|-------------|-------------|
| 1    | 6                | 9           | 4           |
| 2    | 12               | 15          | 10          |
| 3    | 18               | 21          | 16          |
| 4    | 24               | 27          | 22          |
| 5    | 30               | 33          | 28          |
| 10   | 60               | 63          | 58          |
| 32   | 192              | 195         | 190         |

## Remember!

✅ **DO**: Verify at mid-step (+3)  
✅ **DO**: SysEx between steps  
✅ **DO**: Spread buttons over 3-4 ticks  
❌ **DON'T**: Action at multiples of 6  
❌ **DON'T**: Press/release in same tick  
❌ **DON'T**: Expect position N when max_step=N-1  

## Quick Check

Before committing test:
- [ ] All verifications at `(step × 6) + 3`?
- [ ] No SysEx at step boundaries?
- [ ] Button events spread 3-4 ticks?
- [ ] Expectations account for wrapping?

---

*Keep this handy when creating new tests!*
