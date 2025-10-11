# Quick Start: Direct Test Framework

## Run Tests (2 terminals required)

### Terminal 1: Start Sequencer
```bash
cd SimonSaysSeeqNornsRust
cargo run --release --bin simon_says_seeq
```

### Terminal 2: Run Test
```bash
cd SimonSaysSeeqNornsRust
cargo run --release --bin direct_test -- --script test1.json
# or
cargo run --release --bin direct_test -- --script test2.json
```

**Important**: Both must use the same MIDI port (usually "Midi Through:Midi Through Port-0")

## What Each Test Does

**test1.json**: Verifies 7 rows with different max_step values  
**test2.json**: Changes max_step via SysEx and verifies wrapping

## Troubleshooting

**"only 0 steps logged"** → Sequencer not running in Terminal 1  
**Tests fail in sequence** → Already fixed (pattern is reloaded each test)  
**Different MIDI ports** → Both apps must use same port

## More Info

- `TESTING_GUIDE.md` - Complete testing guide
- `SESSION_SUMMARY.md` - What was accomplished
- `MIGRATION_TO_DIRECT_TEST.md` - Technical details
