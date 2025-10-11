# Test2 Simple - Minimal Focus Test

## What This Test Does

**6 steps total, 1 change, 3 verifications**

**Timeline**:
- Steps 1-3: Run with Row 0 max_step = 31
- Step 3: Send SysEx to change Row 0 max_step to 7
- Step 4: Verify (should take effect at step 4)
- Step 5: Verify
- Step 6: Verify

## Expected Positions (if change takes effect at step 4)

- Step 1: position 1
- Step 2: position 2
- Step 3: position 3
- **Step 4: position 0** (reset from 3 → 0 because 3 <= 7, no reset needed, but sequencer might reset anyway)
- Step 5: position 1
- Step 6: position 2

## What We'll See

The three verifications at steps 4, 5, 6 will show:
- If change takes effect immediately (step 4)
- If there's a 1-step delay (step 5)
- If there's a 2-step delay (step 6)
- Or if it takes even longer (all three fail)

## Run Test

```bash
cargo run --release --bin automated_test_clock -- --script test2_simple.json
```

Very short test - results in seconds!
