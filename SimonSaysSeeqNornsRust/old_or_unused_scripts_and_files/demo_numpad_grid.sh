#!/bin/bash

# Demo script for numpad grid simulation
# This shows how to use your laptop's numeric keypad to simulate a monome grid

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎹 SimonSaysSeeq Numpad Grid Demo${NC}"
echo "=================================="
echo

echo -e "${GREEN}What this demo does:${NC}"
echo "• Uses your laptop's numpad as a 3x4 grid simulator"
echo "• Shows visual grid display with LED brightness"
echo "• Lets you create step sequences interactively"
echo "• Demonstrates the sequencer engine working"
echo

echo -e "${GREEN}Numpad Layout:${NC}"
echo "  7 8 9    ->    (0,0) (1,0) (2,0)"
echo "  4 5 6    ->    (0,1) (1,1) (2,1)" 
echo "  1 2 3    ->    (0,2) (1,2) (2,2)"
echo "    0      ->      (0,3)"
echo

echo -e "${GREEN}How to use:${NC}"
echo "1. The application will start and show a visual grid"
echo "2. Press numpad keys + Enter to toggle grid positions"
echo "3. Watch the grid display update with LED brightness"
echo "4. Listen to console logs showing sequencer activity"
echo "5. Press 'q' + Enter to quit"
echo

echo -e "${YELLOW}Example sequence to try:${NC}"
echo "• Press 7 + Enter  (top-left position)"
echo "• Press 5 + Enter  (center position)"
echo "• Press 3 + Enter  (bottom-right position)"
echo "• This creates a diagonal pattern!"
echo

echo -e "${GREEN}What you'll see:${NC}"
echo "• Visual grid with ■ for active positions and □ for inactive"
echo "• Brightness numbers (0-15) showing LED intensity"
echo "• Console logs showing grid press events"
echo "• Sequencer state changes and MIDI events"
echo

# Check if binary exists
BINARY_PATH="./target/x86_64-unknown-linux-gnu/debug/simon_says_seeq"
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${YELLOW}Binary not found. Building first...${NC}"
    cargo build --features="simulation" --no-default-features --target=x86_64-unknown-linux-gnu
    echo
fi

read -p "Press Enter to start the numpad grid simulation..."

echo -e "\n${BLUE}Starting SimonSaysSeeq with numpad simulation...${NC}"
echo -e "${YELLOW}(The grid display will appear after startup logs)${NC}"
echo

# Set log level for better visibility
export RUST_LOG=info

# Run the simulation
exec $BINARY_PATH