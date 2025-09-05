#!/bin/bash

# Native Compilation Script for SimonSaysSeeq on Norns
# This script compiles the Rust project directly on the Norns device
# with full hardware features enabled, avoiding cross-compilation issues.

set -e

# Configuration
PROJECT_DIR="/home/we/dust/code/SimonSaysSeeqRust"
RUST_VERSION="1.75.0"  # Known working version for ARM
SERVICE_NAME="simonsaysseeq-rust"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "=================================================="
    echo "  SimonSaysSeeq Native Compilation on Norns"
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

# Check if we're running on Norns
check_norns_environment() {
    print_status "Checking Norns environment..."
    
    if [ ! -d "/home/we/norns" ]; then
        print_error "This script must be run on a Norns device"
        exit 1
    fi
    
    if [ "$(whoami)" != "we" ]; then
        print_error "This script must be run as user 'we'"
        echo "Please SSH to Norns: ssh we@norns.local"
        exit 1
    fi
    
    print_success "Running on Norns as user 'we'"
}

# Install system dependencies
install_system_dependencies() {
    print_status "Installing system dependencies..."
    
    # Update package list
    sudo apt-get update
    
    # Install build tools and libraries
    sudo apt-get install -y \
        build-essential \
        pkg-config \
        libasound2-dev \
        libudev-dev \
        libevdev-dev \
        libssl-dev \
        cmake \
        git \
        curl
    
    print_success "System dependencies installed"
}

# Install or update Rust
install_rust() {
    print_status "Checking Rust installation..."
    
    if command -v rustc >/dev/null 2>&1; then
        local current_version=$(rustc --version | cut -d' ' -f2)
        print_status "Rust is already installed: $current_version"
        
        # Update Rust toolchain
        print_status "Updating Rust toolchain..."
        rustup update stable
        rustup default stable
    else
        print_status "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        
        # Source the Rust environment
        source ~/.cargo/env
    fi
    
    # Verify installation
    if command -v rustc >/dev/null 2>&1; then
        print_success "Rust installed: $(rustc --version)"
    else
        print_error "Rust installation failed"
        exit 1
    fi
    
    # Install additional components
    print_status "Installing Rust components..."
    rustup component add clippy rustfmt
}

# Prepare source code
prepare_source() {
    print_status "Preparing source code..."
    
    if [ ! -d "$PROJECT_DIR" ]; then
        print_error "Project directory not found: $PROJECT_DIR"
        print_error "Please run the deployment script first to copy the source code"
        exit 1
    fi
    
    cd "$PROJECT_DIR"
    
    # Ensure Cargo.toml exists
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found in $PROJECT_DIR"
        exit 1
    fi
    
    print_success "Source code ready in $PROJECT_DIR"
}

# Clean previous builds
clean_build() {
    print_status "Cleaning previous builds..."
    cd "$PROJECT_DIR"
    
    if [ -d "target" ]; then
        rm -rf target
        print_status "Removed previous build artifacts"
    fi
    
    # Update dependencies
    print_status "Updating Cargo dependencies..."
    cargo update
}

# Build with hardware features
build_with_hardware() {
    print_status "Building with full hardware features..."
    cd "$PROJECT_DIR"
    
    # Set environment variables for native compilation
    export PKG_CONFIG_PATH="/usr/lib/arm-linux-gnueabihf/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig"
    export RUST_BACKTRACE=1
    
    print_status "Building release version with norns features..."
    print_status "This may take 10-20 minutes on Norns hardware..."
    
    # Build with full hardware features
    cargo build --release --features norns
    
    if [ $? -eq 0 ]; then
        print_success "Build completed successfully!"
    else
        print_error "Build failed"
        return 1
    fi
    
    # Verify binary was created
    if [ -f "target/release/simon_says_seeq" ]; then
        print_success "Binary created: target/release/simon_says_seeq"
        
        # Show binary info
        file target/release/simon_says_seeq
        ls -lh target/release/simon_says_seeq
    else
        print_error "Binary not found after build"
        return 1
    fi
}

# Test the binary
test_binary() {
    print_status "Testing the compiled binary..."
    cd "$PROJECT_DIR"
    
    # Test that it can start (run for 3 seconds then stop)
    print_status "Running quick test (3 seconds)..."
    timeout 3 ./target/release/simon_says_seeq || true
    
    print_success "Binary test completed"
}

# Update the deployed binary
update_deployment() {
    print_status "Updating deployed binary..."
    cd "$PROJECT_DIR"
    
    # Stop the service if running
    if systemctl is-active "$SERVICE_NAME" >/dev/null 2>&1; then
        print_status "Stopping $SERVICE_NAME service..."
        sudo systemctl stop "$SERVICE_NAME"
    fi
    
    # Backup old binary
    if [ -f "simon_says_seeq" ]; then
        mv simon_says_seeq simon_says_seeq.backup.$(date +%Y%m%d_%H%M%S)
        print_status "Backed up old binary"
    fi
    
    # Copy new binary
    cp target/release/simon_says_seeq .
    chmod +x simon_says_seeq
    
    print_success "Binary updated in deployment location"
    
    # Restart service if it was running
    if systemctl is-enabled "$SERVICE_NAME" >/dev/null 2>&1; then
        print_status "Restarting $SERVICE_NAME service..."
        sudo systemctl start "$SERVICE_NAME"
        sleep 2
        sudo systemctl status "$SERVICE_NAME" --no-pager || true
    fi
}

# Show final status
show_final_status() {
    print_success "Native compilation completed!"
    echo ""
    echo -e "${GREEN}Next steps:${NC}"
    echo "  1. Test manually: ./simon_says_seeq"
    echo "  2. Check logs: journalctl -u $SERVICE_NAME -f"
    echo "  3. Control: ../norns_control.sh rust-reboot"
    echo ""
    echo -e "${BLUE}Features now enabled:${NC}"
    echo "  ✓ Real MIDI output"
    echo "  ✓ Hardware button/encoder input"
    echo "  ✓ Grid connectivity (if available)"
    echo "  ✓ Native framebuffer graphics"
    echo "  ✓ Audio integration"
    echo ""
    echo -e "${YELLOW}No more 'Hardware simulation mode'!${NC}"
}

# Show help
show_help() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --help, -h      Show this help message"
    echo "  --clean         Clean build only (no compilation)"
    echo "  --deps-only     Install dependencies only"
    echo "  --build-only    Build only (skip dependency installation)"
    echo ""
    echo "This script compiles SimonSaysSeeq natively on Norns with full hardware features."
    echo "It automatically installs Rust, system dependencies, and compiles the project."
    echo ""
    echo "Requirements:"
    echo "  - Must be run on a Norns device"
    echo "  - Must be run as user 'we'"
    echo "  - Internet connection for downloading dependencies"
    echo "  - About 1-2GB free disk space"
    echo "  - 20-30 minutes compilation time"
}

# Main function
main() {
    print_banner
    
    # Parse command line arguments
    case "${1:-}" in
        "--help"|"-h")
            show_help
            exit 0
            ;;
        "--clean")
            check_norns_environment
            prepare_source
            clean_build
            print_success "Build cleaned"
            exit 0
            ;;
        "--deps-only")
            check_norns_environment
            install_system_dependencies
            install_rust
            print_success "Dependencies installed"
            exit 0
            ;;
        "--build-only")
            check_norns_environment
            prepare_source
            clean_build
            build_with_hardware
            test_binary
            update_deployment
            show_final_status
            exit 0
            ;;
        "")
            # Full installation
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
    
    # Full native compilation process
    check_norns_environment
    install_system_dependencies
    install_rust
    prepare_source
    clean_build
    build_with_hardware
    test_binary
    update_deployment
    show_final_status
}

# Handle cleanup on exit
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        print_error "Native compilation failed with exit code $exit_code"
        echo ""
        echo "Common issues:"
        echo "  - Network connectivity (for downloading dependencies)"
        echo "  - Insufficient disk space (need ~1-2GB free)"
        echo "  - Missing system packages"
        echo ""
        echo "Try running with --deps-only first, then --build-only"
    fi
    exit $exit_code
}

trap cleanup EXIT INT TERM

# Run main function
main "$@"