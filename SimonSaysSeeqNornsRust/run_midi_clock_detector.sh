#!/bin/bash

# MIDI Clock Detector Runner Script for Raspberry Pi 5
# This script builds and runs the MIDI clock detector utility

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎹 MIDI Clock Detector for Raspberry Pi 5${NC}"
echo -e "${BLUE}===========================================${NC}"
echo

# Check if we're on Pi5 or compatible system
if [[ $(uname -m) != "aarch64" ]] && [[ $(uname -m) != "armv7l" ]]; then
    echo -e "${YELLOW}⚠️  Warning: Not running on ARM architecture. Detected: $(uname -m)${NC}"
    echo "This script is optimized for Raspberry Pi 5, but will attempt to run anyway."
fi

# Check for required dependencies
echo -e "${BLUE}📋 Checking dependencies...${NC}"

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Error: cargo not found. Please install Rust first.${NC}"
    echo "Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check for ALSA development libraries (needed for MIDI on Linux)
if ! pkg-config --exists alsa; then
    echo -e "${YELLOW}⚠️  Warning: ALSA development libraries not found.${NC}"
    echo "Install with: sudo apt install libasound2-dev"
    echo "Continuing anyway..."
fi

# Check for basic MIDI tools
if ! command -v aconnect &> /dev/null; then
    echo -e "${YELLOW}⚠️  Note: aconnect not found. Install alsa-utils for better MIDI debugging.${NC}"
    echo "Install with: sudo apt install alsa-utils"
fi

echo -e "${GREEN}✅ Basic dependencies check completed${NC}"
echo

# Build the project
echo -e "${BLUE}🔨 Building MIDI clock detector...${NC}"
if cargo build --release --bin midi_clock_detector --features midi; then
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

echo

# Show available MIDI devices before starting
echo -e "${BLUE}🎛️  Available MIDI devices:${NC}"
if command -v aconnect &> /dev/null; then
    echo "ALSA MIDI connections:"
    aconnect -l 2>/dev/null || echo "  No ALSA MIDI devices found"
else
    echo "  (aconnect not available - install alsa-utils for MIDI device listing)"
fi

if command -v lsusb &> /dev/null; then
    echo
    echo "USB devices:"
    lsusb | grep -i midi || echo "  No USB MIDI devices detected"
fi

echo

# Parse command line arguments
MODE="auto"
PORT=""
TIMEOUT="10"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo
            echo "Options:"
            echo "  -h, --help           Show this help"
            echo "  -l, --list           List MIDI ports and exit"
            echo "  -i, --interactive    Run in interactive mode"
            echo "  -s, --scan-only      Only scan, don't monitor"
            echo "  -p, --port PORT      Monitor specific port"
            echo "  -t, --timeout SECS   Scan timeout (default: 10)"
            echo "      --status         Show saved detection status"
            echo
            echo "Examples:"
            echo "  $0                   # Auto-scan and monitor first clock found"
            echo "  $0 -i                # Interactive port selection"
            echo "  $0 -l                # List available ports"
            echo "  $0 --status          # Show current detection status"
            echo "  $0 -p \"USB MIDI\"     # Monitor specific port"
            echo "  $0 -s                # Scan only, no monitoring"
            exit 0
            ;;
        -l|--list)
            MODE="list"
            shift
            ;;
        --status)
            MODE="status"
            shift
            ;;
        -i|--interactive)
            MODE="interactive"
            shift
            ;;
        -s|--scan-only)
            MODE="scan-only"
            shift
            ;;
        -p|--port)
            MODE="specific"
            PORT="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            echo "Use -h for help"
            exit 1
            ;;
    esac
done

# Build command line arguments for the detector
DETECTOR_ARGS=()

case $MODE in
    list)
        DETECTOR_ARGS+=(--list-ports)
        ;;
    status)
        DETECTOR_ARGS+=(--status)
        ;;
    interactive)
        DETECTOR_ARGS+=(--interactive)
        ;;
    scan-only)
        DETECTOR_ARGS+=(--scan-only --timeout "$TIMEOUT")
        ;;
    specific)
        if [[ -z "$PORT" ]]; then
            echo -e "${RED}❌ Error: --port requires a port name${NC}"
            exit 1
        fi
        DETECTOR_ARGS+=(--port "$PORT" --timeout "$TIMEOUT")
        ;;
    auto)
        DETECTOR_ARGS+=(--timeout "$TIMEOUT")
        ;;
esac

# Run the detector
echo -e "${BLUE}🚀 Starting MIDI clock detector...${NC}"
echo

# Set up signal handling for clean exit
trap 'echo -e "\n${YELLOW}🛑 Stopping MIDI clock detector...${NC}"; exit 0' INT TERM

# Run with proper error handling
if ./target/release/midi_clock_detector "${DETECTOR_ARGS[@]}"; then
    echo
    echo -e "${GREEN}✅ MIDI clock detector completed successfully${NC}"
else
    echo
    echo -e "${RED}❌ MIDI clock detector encountered an error${NC}"
    exit 1
fi

echo
echo -e "${BLUE}🎵 Done! Use this information to configure your sequencer's MIDI input.${NC}"