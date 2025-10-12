#!/bin/bash

# run_and_test.sh
# Automated test runner for SimonSaysSeeq
# Runs sequencer in test mode and executes all clock-driven tests

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  SimonSaysSeeq Automated Test Runner${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo

# Check if sequencer binary exists
if [ ! -f "target/release/simon_says_seeq" ]; then
    echo -e "${RED}❌ Sequencer binary not found${NC}"
    echo "Building sequencer..."
    cargo build --release --bin simon_says_seeq
fi

# Check if test binary exists
if [ ! -f "target/release/clock_driven_test" ]; then
    echo -e "${RED}❌ Test binary not found${NC}"
    echo "Building test runner..."
    cargo build --release --bin clock_driven_test
fi

# Check if test files exist
TESTS=(test1.json test2.json test3.json test4.json)
for test in "${TESTS[@]}"; do
    if [ ! -f "$test" ]; then
        echo -e "${YELLOW}⚠️  Warning: Test file not found: $test${NC}"
    fi
done

echo -e "${GREEN}✅ All binaries ready${NC}"
echo

# Create temp directory for logs
LOGDIR="test_logs_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$LOGDIR"

echo -e "${BLUE}📂 Logs will be saved to: $LOGDIR${NC}"
echo

# Start sequencer in background
echo -e "${BLUE}🎵 Starting sequencer in test mode...${NC}"
RUST_LOG=info target/release/simon_says_seeq --test-mode > "$LOGDIR/sequencer.log" 2>&1 &
SEQUENCER_PID=$!

echo -e "${GREEN}✅ Sequencer started (PID: $SEQUENCER_PID)${NC}"
echo

# Wait for sequencer to initialize
echo -e "${YELLOW}⏳ Waiting 60 seconds for sequencer initialization...${NC}"
sleep 60

# Function to run a test
run_test() {
    local test_file=$1
    local test_name=$(basename "$test_file" .json)
    
    echo -e "${BLUE}════════════════════════════════════════════${NC}"
    echo -e "${BLUE}🧪 Running: $test_name${NC}"
    echo -e "${BLUE}════════════════════════════════════════════${NC}"
    
    if target/release/clock_driven_test --script "$test_file" > "$LOGDIR/${test_name}.log" 2>&1; then
        echo -e "${GREEN}✅ $test_name PASSED${NC}"
        return 0
    else
        echo -e "${RED}❌ $test_name FAILED${NC}"
        echo -e "${YELLOW}   See log: $LOGDIR/${test_name}.log${NC}"
        return 1
    fi
}

# Run all tests
PASSED=0
FAILED=0
FAILED_TESTS=()

for test in "${TESTS[@]}"; do
    if [ -f "$test" ]; then
        if run_test "$test"; then
            ((PASSED++))
        else
            ((FAILED++))
            FAILED_TESTS+=("$test")
        fi
        echo
        
        # Wait between tests
        sleep 1
    fi
done

# Send MIDI stop to sequencer
echo -e "${BLUE}⏹️  Sending MIDI Stop to sequencer...${NC}"
# Using Python to send MIDI stop
python3 -c "
import mido
try:
    outport = mido.open_output('Midi Through:Midi Through Port-0 14:0')
    outport.send(mido.Message('stop'))
    print('MIDI Stop sent')
    outport.close()
except Exception as e:
    print(f'Could not send MIDI stop: {e}')
" 2>/dev/null || echo -e "${YELLOW}⚠️  Could not send MIDI stop (mido not installed)${NC}"

sleep 1

# Stop sequencer
echo -e "${BLUE}🛑 Stopping sequencer...${NC}"
kill $SEQUENCER_PID 2>/dev/null || true
wait $SEQUENCER_PID 2>/dev/null || true
echo -e "${GREEN}✅ Sequencer stopped${NC}"
echo

# Summary
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Test Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed tests:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "${RED}  - $test${NC}"
    done
    echo
fi

echo -e "${BLUE}📂 Logs saved to: $LOGDIR${NC}"
echo

# Exit with appropriate code
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
else
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
fi