#!/bin/bash

# Local Testing Script for SimonSaysSeeqNornsRust
# This script builds and runs the project locally for development and testing

set -e  # Exit on any error

# Configuration
PROJECT_NAME="simon-says-seeq-rust"
BINARY_NAME="simon_says_seeq"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🦀 SimonSaysSeeq Local Testing Script${NC}"
echo "=============================================="

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the SimonSaysSeeqNornsRust directory."
    exit 1
fi

# Parse command line arguments
MODE="simulation"
VERBOSE=""
RELEASE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--hardware)
            MODE="hardware"
            shift
            ;;
        -s|--simulation)
            MODE="simulation"
            shift
            ;;
        -v|--verbose)
            VERBOSE="--verbose"
            shift
            ;;
        -r|--release)
            RELEASE="--release"
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -h, --hardware      Build with hardware features (requires hardware)"
            echo "  -s, --simulation    Build in simulation mode (default)"
            echo "  -v, --verbose       Verbose output"
            echo "  -r, --release       Release build (optimized)"
            echo "  --help              Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                  # Simulation mode, debug build"
            echo "  $0 --hardware       # Hardware mode (for testing on actual hardware)"
            echo "  $0 --release        # Optimized build"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

print_status "Testing mode: $MODE"

# Set features based on mode
if [ "$MODE" = "hardware" ]; then
    FEATURES="hardware,midi"
    print_warning "Hardware mode requires actual hardware devices (grid, MIDI, etc.)"
else
    FEATURES="simulation"
    print_status "Simulation mode - no hardware required"
fi

# Check dependencies
print_status "Checking Rust toolchain..."
if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Show Rust version
rustc --version
cargo --version

# Clean previous builds if requested
if [[ "$*" == *"--clean"* ]]; then
    print_status "Cleaning previous builds..."
    cargo clean
fi

# Check for basic dependencies
print_status "Checking dependencies..."

if [ "$MODE" = "hardware" ]; then
    # Check for ALSA dev libraries (required for cpal)
    if ! pkg-config --exists alsa; then
        print_warning "ALSA development libraries not found."
        echo "Install with:"
        echo "  Ubuntu/Debian: sudo apt install libasound2-dev"
        echo "  Fedora: sudo dnf install alsa-lib-devel"
        echo "  Arch: sudo pacman -S alsa-lib"
    fi
    
    # Check for HID libraries
    if ! pkg-config --exists hidapi-libusb; then
        print_warning "HID API libraries not found."
        echo "Install with:"
        echo "  Ubuntu/Debian: sudo apt install libhidapi-dev"
        echo "  Fedora: sudo dnf install hidapi-devel"
        echo "  Arch: sudo pacman -S hidapi"
    fi
fi

# Run tests first
print_status "Running tests..."
if ! cargo test $VERBOSE --features="$FEATURES"; then
    print_error "Tests failed"
    exit 1
fi
print_success "All tests passed"

# Build the project
print_status "Building project with features: $FEATURES"
if ! cargo build $RELEASE $VERBOSE --features="$FEATURES"; then
    print_error "Build failed"
    exit 1
fi

print_success "Build completed successfully"

# Check binary
if [ -n "$RELEASE" ]; then
    BINARY_PATH="target/release/$BINARY_NAME"
else
    BINARY_PATH="target/debug/$BINARY_NAME"
fi

if [ ! -f "$BINARY_PATH" ]; then
    print_error "Binary not found at $BINARY_PATH"
    exit 1
fi

print_status "Binary information:"
file "$BINARY_PATH"
ls -lh "$BINARY_PATH"

# Set up environment
export RUST_LOG="${RUST_LOG:-info}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

print_status "Environment:"
echo "  RUST_LOG=$RUST_LOG"
echo "  RUST_BACKTRACE=$RUST_BACKTRACE"
echo "  Features: $FEATURES"

# Create a simple test config if it doesn't exist
CONFIG_DIR="$HOME/.config/simonsaysseeq"
mkdir -p "$CONFIG_DIR"

if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    print_status "Creating default configuration..."
    cat > "$CONFIG_DIR/config.toml" << 'EOF'
# SimonSaysSeeq Configuration

[sequencer]
default_tempo = 120.0
steps_per_bar = 16
ticks_per_step = 12

[midi]
output_device = "default"
input_device = "default"
base_channel = 1

[grid]
brightness = 15
auto_connect = true

[screen]
fps = 30
brightness = 80

[co2]
enabled = true
data_path = "~/.local/share/simonsaysseeq/co2_data.csv"
update_interval = 3600  # 1 hour
EOF
    print_success "Created default config at $CONFIG_DIR/config.toml"
fi

# Show run instructions
echo ""
print_success "Ready to run! 🚀"
echo ""
echo "Run options:"
echo "  1. Direct execution:"
echo "     ./$BINARY_PATH"
echo ""
echo "  2. With custom log level:"
echo "     RUST_LOG=debug ./$BINARY_PATH"
echo ""
echo "  3. With environment variables:"
echo "     RUST_LOG=trace RUST_BACKTRACE=full ./$BINARY_PATH"
echo ""

if [ "$MODE" = "simulation" ]; then
    echo "Simulation Mode Notes:"
    echo "  - No hardware devices required"
    echo "  - Sequencer state displayed: 🎵 RUNNING/STOPPED with tempo, step, bar"
    echo "  - MIDI events logged: 🎹 Note ON/OFF with velocity and channel"
    echo "  - Grid operations simulated: 💡 LED states and visual grid display"
    echo "  - Transport events shown: 🕐 Clock start/stop"
    echo "  - All output visible at default 'info' log level"
    echo "  - Interactive controls: 'r' + Enter to run/stop, numpad for grid, 'q' to quit"
    echo ""
elif [ "$MODE" = "hardware" ]; then
    echo "Hardware Mode Notes:"
    echo "  - Requires actual Norns hardware or compatible devices"
    echo "  - MIDI ports will be opened"
    echo "  - Grid devices will be detected"
    echo "  - Framebuffer access needed for screen"
    echo ""
    echo "Hardware Requirements:"
    echo "  - USB permissions for grid access"
    echo "  - MIDI device access"
    echo "  - Audio system access (ALSA)"
    echo ""
fi

echo "Configuration:"
echo "  Config file: $CONFIG_DIR/config.toml"
echo "  Log level: $RUST_LOG"
echo ""
echo "Controls (when running):"
echo "  Ctrl+C: Graceful shutdown"
echo "  'r' + Enter: Run/stop sequencer (simulation mode)"
echo "  Numpad 0-9 + Enter: Grid button simulation"
echo "  'q' + Enter: Quit (simulation mode)"
echo "  Grid: Interactive sequencing (hardware mode)"
echo "  Encoders: Parameter control (hardware mode)"
echo ""

# Ask if user wants to run now
read -p "Would you like to run the application now? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Starting SimonSaysSeeq..."
    echo "Press Ctrl+C to stop"
    echo "===================="
    ./"$BINARY_PATH"
else
    echo "To run later, use: ./$BINARY_PATH"
fi

print_success "Local testing script completed! 🎵"