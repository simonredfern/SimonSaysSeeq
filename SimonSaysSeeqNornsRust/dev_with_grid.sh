#!/bin/bash

# Development script for testing SimonSaysSeeq with Grid on laptop
# This allows rapid iteration with real grid hardware before deploying to Norns

set -e

# Configuration
FEATURES="desktop"
LOG_LEVEL="${LOG_LEVEL:-info}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "=================================================="
    echo "  SimonSaysSeeq Grid Development Mode"
    echo "=================================================="
    echo -e "${NC}"
}

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

# Check for Grid connection
check_grid() {
    print_status "Checking for connected Grid devices..."
    
    # Check for USB HID devices that might be a Grid
    if command -v lsusb >/dev/null 2>&1; then
        local grid_devices=$(lsusb | grep -i "monome\|grid" || true)
        if [ -n "$grid_devices" ]; then
            print_success "Found potential Grid device(s):"
            echo "$grid_devices"
        else
            print_warning "No obvious Grid devices found via lsusb"
            print_status "Grid may still work if connected via USB"
        fi
    fi
    
    # Check for HID devices
    if [ -d "/dev/hidraw0" ] || [ -d "/dev/input" ]; then
        local hid_count=$(ls /dev/hidraw* 2>/dev/null | wc -l || echo "0")
        print_status "Found $hid_count HID devices in /dev/"
    fi
}

# Check system dependencies
check_dependencies() {
    print_status "Checking development dependencies..."
    
    # Check for Rust
    if ! command -v cargo >/dev/null 2>&1; then
        print_error "Rust/Cargo not found. Please install Rust:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # Check for required system libraries
    local missing_libs=()
    
    if ! pkg-config --exists libudev; then
        missing_libs+=("libudev-dev")
    fi
    
    if ! pkg-config --exists hidapi-libusb; then
        missing_libs+=("libhidapi-dev")
    fi
    
    if [ ${#missing_libs[@]} -gt 0 ]; then
        print_error "Missing system libraries. Please install:"
        echo "  Ubuntu/Debian: sudo apt install ${missing_libs[*]}"
        echo "  macOS: brew install hidapi"
        echo "  Arch: sudo pacman -S hidapi"
        exit 1
    fi
    
    print_success "All dependencies available"
}

# Build for development
build_dev() {
    print_status "Building for desktop development..."
    
    export RUST_LOG="$LOG_LEVEL"
    
    cargo build --features "$FEATURES"
    
    if [ $? -eq 0 ]; then
        print_success "Build completed successfully"
    else
        print_error "Build failed"
        exit 1
    fi
}

# Run the application
run_dev() {
    print_status "Starting SimonSaysSeeq in development mode..."
    print_status "Features enabled: $FEATURES"
    print_status "Log level: $LOG_LEVEL"
    echo ""
    print_warning "Press Ctrl+C to stop"
    echo ""
    
    export RUST_LOG="$LOG_LEVEL"
    
    # Run with development features
    cargo run --features "$FEATURES"
}

# Watch for changes and auto-rebuild
watch_dev() {
    print_status "Starting file watcher for auto-rebuild..."
    
    if ! command -v cargo-watch >/dev/null 2>&1; then
        print_status "Installing cargo-watch..."
        cargo install cargo-watch
    fi
    
    export RUST_LOG="$LOG_LEVEL"
    
    cargo watch -x "build --features $FEATURES" -x "run --features $FEATURES"
}

# Show development help
show_help() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  build       Build for development"
    echo "  run         Build and run with grid support"
    echo "  watch       Watch files and auto-rebuild/run"
    echo "  check       Check dependencies and grid connection"
    echo "  test        Run tests"
    echo "  help        Show this help"
    echo ""
    echo "Environment variables:"
    echo "  LOG_LEVEL   Set log level (debug, info, warn, error) [default: info]"
    echo ""
    echo "Examples:"
    echo "  $0 run                    # Build and run normally"
    echo "  LOG_LEVEL=debug $0 run    # Run with debug logging"
    echo "  $0 watch                  # Auto-rebuild on file changes"
    echo ""
    echo "Development workflow:"
    echo "  1. Connect your Grid via USB"
    echo "  2. Run: $0 check"
    echo "  3. Run: $0 run"
    echo "  4. Edit code and use: $0 watch for auto-reload"
    echo "  5. When ready: ./deploy_to_norns.sh --native"
    echo ""
    echo "This mode gives you:"
    echo "  ✓ Real Grid connectivity and feedback"
    echo "  ✓ Real MIDI output to your laptop"
    echo "  ✓ Fast rebuild/test cycles (seconds, not minutes)"
    echo "  ✓ Full debugging capabilities"
    echo "  ✗ No Norns hardware (encoders/buttons) - simulate these"
}

# Show development status
show_status() {
    print_status "Development environment status:"
    echo ""
    
    # Rust version
    if command -v cargo >/dev/null 2>&1; then
        echo "  Rust: $(rustc --version)"
    else
        echo "  Rust: ❌ Not installed"
    fi
    
    # Features that will be compiled
    echo "  Features: $FEATURES"
    echo "  Log level: $LOG_LEVEL"
    
    # Grid connection
    check_grid
    
    echo ""
    echo "  Ready for development: ✓"
}

# Main function
main() {
    print_banner
    
    case "${1:-run}" in
        "build")
            check_dependencies
            build_dev
            ;;
        "run")
            check_dependencies
            build_dev
            run_dev
            ;;
        "watch")
            check_dependencies
            watch_dev
            ;;
        "check")
            check_dependencies
            check_grid
            show_status
            ;;
        "test")
            print_status "Running tests..."
            cargo test --features "$FEATURES"
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        "status")
            show_status
            ;;
        *)
            print_error "Unknown command: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# Make sure we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the SimonSaysSeeqNornsRust directory"
    exit 1
fi

# Run main function
main "$@"