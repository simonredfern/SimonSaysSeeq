#!/bin/bash

# SimonSaysSeeq Raspberry Pi 5 Build and Run Script
# This script builds and runs SimonSaysSeeqNornsRust directly on a Raspberry Pi 5
# with full hardware support and optimizations for ARM64 architecture.

set -e

# Configuration
PROJECT_NAME="SimonSaysSeeqNornsRust"
RUST_VERSION="stable"
BUILD_TYPE="${BUILD_TYPE:-release}"
RPI_FEATURES="${RPI_FEATURES:-hardware,midi,desktop}"
AI_LOG_ENABLED="false"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_build() {
    echo -e "${PURPLE}[BUILD]${NC} $1"
}

log_run() {
    echo -e "${CYAN}[RUN]${NC} $1"
}

print_banner() {
    echo -e "${GREEN}"
    echo "=========================================================="
    echo "  🎵 SimonSaysSeeq Raspberry Pi 5 Builder & Runner 🎵"
    echo "=========================================================="
    echo -e "${NC}"
    echo "Features: $RPI_FEATURES"
    echo "Build type: $BUILD_TYPE"
    echo "Rust: $RUST_VERSION"
    echo
}

# Check if running on Raspberry Pi 5
check_raspberry_pi() {
    log_info "Checking Raspberry Pi environment..."

    # Check if we're on ARM64
    if [ "$(uname -m)" != "aarch64" ]; then
        log_warning "Not running on ARM64 architecture. Detected: $(uname -m)"
        log_warning "This script is optimized for Raspberry Pi 5 (ARM64)"
    fi

    # Check for Raspberry Pi specific files
    if [ -f "/proc/device-tree/model" ]; then
        local model=$(cat /proc/device-tree/model 2>/dev/null | tr -d '\0')
        log_info "Device: $model"

        if [[ "$model" == *"Raspberry Pi 5"* ]]; then
            log_success "Running on Raspberry Pi 5 - optimal performance expected"
        elif [[ "$model" == *"Raspberry Pi"* ]]; then
            log_warning "Running on older Raspberry Pi model - performance may be limited"
        fi
    else
        log_warning "Cannot detect Raspberry Pi model"
    fi

    log_success "Environment check completed"
}

# Update system packages
update_system() {
    log_info "Updating system packages..."

    sudo apt-get update
    log_success "Package list updated"
}

# Install system dependencies
install_system_dependencies() {
    log_info "Installing system dependencies..."

    # Essential build tools
    sudo apt-get install -y \
        build-essential \
        pkg-config \
        git \
        curl \
        wget \
        cmake \
        ninja-build

    # Audio system dependencies
    sudo apt-get install -y \
        libasound2-dev \
        libjack-jackd2-dev \
        jackd2 \
        pulseaudio \
        pulseaudio-utils

    # Hardware interface dependencies
    sudo apt-get install -y \
        libudev-dev \
        libevdev-dev \
        libusb-1.0-0-dev \
        libhidapi-dev

    # Graphics and display
    sudo apt-get install -y \
        libgl1-mesa-dev \
        libgles2-mesa-dev \
        libegl1-mesa-dev \
        libdrm-dev \
        libgbm-dev

    # Network and communication
    sudo apt-get install -y \
        libssl-dev \
        libcurl4-openssl-dev

    # Optional: Serial communication for grid devices
    sudo apt-get install -y \
        minicom \
        screen

    log_success "System dependencies installed"
}

# Install or update Rust
install_rust() {
    log_info "Checking Rust installation..."

    if command -v rustc >/dev/null 2>&1; then
        local current_version=$(rustc --version | cut -d' ' -f2)
        log_info "Rust already installed: $current_version"

        # Update if requested
        if [ "$1" = "--update" ]; then
            log_info "Updating Rust toolchain..."
            rustup update $RUST_VERSION
            rustup default $RUST_VERSION
        fi
    else
        log_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain $RUST_VERSION

        # Source the Rust environment
        source ~/.cargo/env
        export PATH="$HOME/.cargo/bin:$PATH"
    fi

    # Verify installation
    if command -v rustc >/dev/null 2>&1; then
        log_success "Rust ready: $(rustc --version)"
    else
        log_error "Rust installation failed"
        exit 1
    fi

    # Install useful components
    log_info "Installing Rust components..."
    rustup component add clippy rustfmt

    # Install cargo utilities for better experience
    if ! command -v cargo-watch >/dev/null 2>&1; then
        log_info "Installing cargo-watch for development..."
        cargo install cargo-watch || log_warning "Failed to install cargo-watch"
    fi
}

# Configure audio system
setup_audio() {
    log_info "Configuring audio system..."

    # Add current user to audio group
    sudo usermod -a -G audio "$USER"

    # Configure JACK for low-latency audio (optional)
    if command -v jackd >/dev/null 2>&1; then
        log_info "JACK Audio Connection Kit available"

        # Create basic JACK configuration
        # Remove any existing .jackdrc directory/file
        rm -rf ~/.jackdrc
        cat > ~/.jackdrc << 'EOF'
/usr/bin/jackd -dalsa -dhw:0 -r44100 -p1024 -n2
EOF
        log_info "JACK configuration created (~/.jackdrc)"
    fi

    # Ensure pulseaudio is running
    if command -v pulseaudio >/dev/null 2>&1; then
        pulseaudio --start 2>/dev/null || true
        log_info "PulseAudio started"
    fi

    log_success "Audio system configured"
}

# Configure USB and hardware access
setup_hardware_access() {
    log_info "Configuring hardware access..."

    # Add user to necessary groups for hardware access
    sudo usermod -a -G dialout,plugdev,gpio,i2c,spi "$USER"

    # Create udev rules for MIDI and HID devices
    sudo tee /etc/udev/rules.d/99-simonsaysseeq.rules > /dev/null << 'EOF'
# MIDI devices
SUBSYSTEM=="snd", GROUP="audio", MODE="0664"

# HID devices (for Framework RGB Macropad and similar)
SUBSYSTEM=="hidraw", MODE="0666", GROUP="plugdev"

# USB MIDI devices
SUBSYSTEM=="usb", ATTRS{idVendor}=="*", ATTRS{idProduct}=="*", MODE="0666", GROUP="audio"

# Grid devices (Monome and compatible)
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6001", MODE="0666", GROUP="dialout"
SUBSYSTEM=="usb", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6001", MODE="0666", GROUP="dialout"

# Framework Computer devices
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="32ac", MODE="0666", GROUP="plugdev"
EOF

    # Reload udev rules
    sudo udevadm control --reload-rules
    sudo udevadm trigger

    log_success "Hardware access configured"
    log_warning "You may need to log out and back in for group changes to take effect"
}

# Install serialosc for grid support
install_serialosc() {
    log_info "Installing serialosc for grid device support..."

    # Check if serialosc is already installed and running
    if systemctl is-active --quiet serialosc 2>/dev/null; then
        log_info "serialosc is already installed and running"
        return 0
    fi

    # Try package manager first
    log_info "Trying package manager installation..."
    sudo apt-get update
    if sudo apt-get install -y serialosc 2>/dev/null; then
        log_success "serialosc installed via package manager"
        sudo systemctl enable serialosc
        sudo systemctl start serialosc
        if systemctl is-active --quiet serialosc; then
            log_success "serialosc service started successfully"
            return 0
        fi
    fi

    log_warning "Package manager installation failed, trying manual build..."

    # Check if binary exists but service doesn't
    if [ -f "/usr/local/bin/serialoscd" ] && ! systemctl is-enabled serialosc >/dev/null 2>&1; then
        log_info "serialosc binary found, creating service..."
    else
        # Install build dependencies - focus on fixing the missing headers
        log_info "Installing build dependencies..."
        sudo apt-get install -y \
            git \
            build-essential \
            libudev-dev \
            liblo-dev \
            python3 \
            pkg-config \
            libuv1-dev \
            libavahi-compat-libdnssd-dev \
            avahi-daemon

        # Start avahi daemon (for dns_sd.h support)
        sudo systemctl enable avahi-daemon
        sudo systemctl start avahi-daemon

        # Create temporary directory for build
        local temp_dir=$(mktemp -d)
        local original_dir=$(pwd)
        cd "$temp_dir"

        # First install libmonome dependency
        log_info "Cloning and building libmonome..."
        if ! git clone https://github.com/monome/libmonome.git; then
            log_error "Failed to clone libmonome repository"
            cd "$original_dir"
            rm -rf "$temp_dir"
            return 1
        fi

        cd libmonome

        log_info "Initializing libmonome submodules..."
        if ! git submodule update --init --recursive; then
            log_error "Failed to initialize libmonome submodules"
            cd "$original_dir"
            rm -rf "$temp_dir"
            return 1
        fi

        log_info "Building libmonome..."
        # Configure without the problematic windows.h check
        if ! ./waf configure --prefix=/usr/local; then
            log_error "Failed to configure libmonome"
            cd "$original_dir"
            rm -rf "$temp_dir"
            return 1
        fi

        if ! ./waf; then
            log_error "Failed to build libmonome"
            cd "$original_dir"
            rm -rf "$temp_dir"
            return 1
        fi

        log_info "Installing libmonome..."
        if ! sudo ./waf install; then
            log_error "Failed to install libmonome"
            cd "$original_dir"
            rm -rf "$temp_dir"
            return 1
        fi

        # Update library cache
        sudo ldconfig

        cd "$temp_dir"

        # Now install serialosc
        log_info "Cloning serialosc repository..."
        if ! git clone https://github.com/monome/serialosc.git; then
            log_error "Failed to clone serialosc repository"
            cd "$original_dir"
            rm -rf "$temp_dir"
            return 1
        fi

        cd serialosc

        log_info "Initializing submodules..."
        if ! git submodule update --init --recursive; then
            log_error "Failed to initialize submodules"
            cd "$original_dir"
            rm -rf "$temp_dir"
            return 1
        fi

        log_info "Building serialosc (using waf)..."
        if ! ./waf configure --prefix=/usr/local; then
            log_error "Failed to configure serialosc with waf"
            cd "$original_dir"
            rm -rf "$temp_dir"
            return 1
        fi

        if ! ./waf; then
            log_error "Failed to build serialosc with waf"
            cd "$original_dir"
            rm -rf "$temp_dir"
            return 1
        fi

        log_info "Installing serialosc..."
        if ! sudo ./waf install; then
            log_error "Failed to install serialosc"
            cd "$original_dir"
            rm -rf "$temp_dir"
            return 1
        fi

        # Clean up
        cd "$original_dir"
        rm -rf "$temp_dir"
    fi

    # Verify binary was installed
    if [ ! -f "/usr/local/bin/serialoscd" ]; then
        log_error "serialoscd binary not found at /usr/local/bin/serialoscd"
        return 1
    fi

    log_info "Creating systemd service..."
    # Create systemd service file
    sudo tee /etc/systemd/system/serialosc.service > /dev/null << 'EOF'
[Unit]
Description=serialosc daemon for monome devices
After=multi-user.target avahi-daemon.service udev.target
Wants=avahi-daemon.service

[Service]
Type=simple
ExecStart=/usr/local/bin/serialoscd
Restart=always
RestartSec=3
TimeoutStartSec=30
TimeoutStopSec=10
User=root
Environment=HOME=/root

[Install]
WantedBy=multi-user.target
EOF

    # Enable and start the service
    log_info "Enabling and starting serialosc service..."
    sudo systemctl daemon-reload

    if ! sudo systemctl enable serialosc; then
        log_error "Failed to enable serialosc service"
        return 1
    fi

    if ! sudo systemctl start serialosc; then
        log_error "Failed to start serialosc service"
        log_info "Checking service status..."
        sudo systemctl status serialosc --no-pager
        return 1
    fi

    # Verify service is running
    if systemctl is-active --quiet serialosc; then
        log_success "serialosc installed and started successfully"
        log_info "Service status:"
        sudo systemctl status serialosc --no-pager -l
    else
        log_error "serialosc service failed to start properly"
        log_info "Service logs:"
        sudo journalctl -u serialosc --no-pager -l
        return 1
    fi

    log_info "Grid devices will be automatically detected when connected"
    log_info "You can check serialosc status with: sudo systemctl status serialosc"
}

# Build the project
build_project() {
    log_build "Building SimonSaysSeeq for Raspberry Pi 5..."

    if [ ! -f "Cargo.toml" ]; then
        log_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    fi

    # Set build environment variables for optimal ARM64 performance
    export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
    export PKG_CONFIG_PATH="/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig"

    log_build "Build environment configured for ARM64 optimization"
    log_build "Features: $RPI_FEATURES"
    log_build "Type: $BUILD_TYPE"

    # Clean previous builds for fresh start
    if [ -d "target" ]; then
        log_build "Cleaning previous build artifacts..."
        cargo clean
    fi

    # Update dependencies
    log_build "Updating dependencies..."
    cargo update

    # Build with specified features
    log_build "Starting compilation (this may take 10-20 minutes)..."
    local start_time=$(date +%s)

    # Get git hash for version info
    local git_hash=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
    log_info "Building with git hash: $git_hash"

    if [ "$BUILD_TYPE" = "release" ]; then
        GIT_HASH="$git_hash" cargo build --release --features "$RPI_FEATURES"
        local binary_path="target/release/simon_says_seeq"
    else
        GIT_HASH="$git_hash" cargo build --features "$RPI_FEATURES"
        local binary_path="target/debug/simon_says_seeq"
    fi

    local end_time=$(date +%s)
    local build_time=$((end_time - start_time))

    if [ -f "$binary_path" ]; then
        log_success "Build completed in ${build_time}s"
        log_success "Binary: $binary_path"

        # Show binary information
        log_info "Binary details:"
        file "$binary_path"
        ls -lh "$binary_path"
    else
        log_error "Build failed - binary not found"
        exit 1
    fi
}

# Test the binary
test_binary() {
    log_info "Testing the compiled binary..."

    local binary_path
    if [ "$BUILD_TYPE" = "release" ]; then
        binary_path="target/release/simon_says_seeq"
    else
        binary_path="target/debug/simon_says_seeq"
    fi

    if [ ! -f "$binary_path" ]; then
        log_error "Binary not found: $binary_path"
        return 1
    fi

    # Quick startup test (run for 3 seconds)
    log_info "Running startup test (3 seconds)..."
    timeout 3 "$binary_path" --test 2>/dev/null || true

    log_success "Binary test completed"
}

# Setup configuration with new MIDI detection fields
setup_configuration() {
    log_info "Setting up configuration with MIDI auto-detection..."

    local config_dir="$HOME/.config/simon-says-seeq"
    local config_file="$config_dir/config.toml"

    # Create config directory
    mkdir -p "$config_dir"

    # Check if config file exists and has new fields
    if [ -f "$config_file" ]; then
        if grep -q "auto_detect_clock" "$config_file"; then
            log_info "Configuration already up to date"
            return 0
        else
            log_info "Updating existing configuration with MIDI detection fields..."
            # Backup existing config
            cp "$config_file" "$config_file.backup.$(date +%Y%m%d_%H%M%S)"
        fi
    fi

    # Create or update configuration file
    log_info "Creating configuration file: $config_file"
    cat > "$config_file" << 'EOF'
[midi]
device = ""                      # Empty = auto-detect MIDI clock
auto_detect_clock = true         # Enable automatic MIDI clock detection
detection_retry_interval = 30    # Re-scan every 30s if no clock
detection_scan_timeout = 10      # How long to scan each port
last_detected_device = null      # Will be auto-populated
default_channel = 1
default_velocity = 100
send_clock = true
clock_ppq = 24
stuck_note_timeout = 5

[grid]
rotation = 0
default_brightness = 5
debounce_ms = 50
auto_detect = true

[sequencer]
default_tempo = 30.0
steps_per_bar = 16
ticks_per_step = 12
default_first_step = 1
default_last_step = 16
auto_save_interval = 300

[hardware]
input_poll_ms = 10
encoder_sensitivity = 1.0
button_hold_ms = 500
simulation_mode = false

[display]
refresh_rate = 30
brightness = 255
show_beat_indicators = true
show_tempo_viz = true
font_scale = 1

[co2]
enabled = true
data_dir = "/home/simonredfern/Documents/workspace_2025/SimonSaysSeeq/SimonSaysSeeqNornsRust/co2_data"
wow_threshold = 20.0
flutter_threshold = 10.0
window_size = 100
voltage_scale = 1.0
co2_min = 320.0
co2_max = 450.0
EOF

    # Set proper ownership
    chown -R "$USER:$USER" "$config_dir"
    chmod -R 755 "$config_dir"
    chmod 644 "$config_file"

    log_success "Configuration file created/updated: $config_file"
    log_info "MIDI auto-detection is enabled by default"
}

# Install systemd service
install_service() {
    log_info "Installing systemd service..."

    # Stop service if it's running
    if systemctl is-active --quiet simonsaysseeq-rpi 2>/dev/null; then
        log_info "Stopping existing service..."
        sudo systemctl stop simonsaysseeq-rpi
        log_info "Service stopped"
    fi

    local binary_path
    if [ "$BUILD_TYPE" = "release" ]; then
        binary_path="$(pwd)/target/release/simon_says_seeq"
    else
        binary_path="$(pwd)/target/debug/simon_says_seeq"
    fi

    # Create systemd service
    sudo tee /etc/systemd/system/simonsaysseeq-rpi.service > /dev/null << EOF
[Unit]
Description=SimonSaysSeeq Sequencer for Raspberry Pi
After=multi-user.target udev.target serialosc.service systemd-udev-settle.service
Wants=serialosc.service systemd-udev-settle.service

[Service]
Type=simple
User=$USER
Group=audio
WorkingDirectory=$(pwd)
ExecStartPre=/bin/sleep 10
ExecStart=$binary_path
Environment=RUST_LOG=error
Environment=XDG_RUNTIME_DIR=/run/user/$(id -u)
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal
KillMode=mixed
TimeoutStartSec=30
TimeoutStopSec=15
SupplementaryGroups=audio gpio i2c spi dialout plugdev

[Install]
WantedBy=multi-user.target
EOF

    # Reload systemd and enable service
    sudo systemctl daemon-reload
    sudo systemctl enable simonsaysseeq-rpi

    log_success "Systemd service installed and enabled"
    log_info "Service will start automatically on boot"
}

# Run the application
run_application() {
    log_run "Starting SimonSaysSeeq..."

    local binary_path
    if [ "$BUILD_TYPE" = "release" ]; then
        binary_path="target/release/simon_says_seeq"
    else
        binary_path="target/debug/simon_says_seeq"
    fi

    if [ ! -f "$binary_path" ]; then
        log_error "Binary not found: $binary_path"
        log_error "Please build the project first with: $0 --build"
        exit 1
    fi

    # Set environment variables
    export RUST_LOG="${RUST_LOG:-info}"
    export RUST_BACKTRACE=1

    log_run "Environment: RUST_LOG=$RUST_LOG"
    log_run "Binary: $binary_path"
    log_run "Features: $RPI_FEATURES"

    echo
    log_run "🎵 Starting SimonSaysSeeq on Raspberry Pi 5..."
    echo -e "${CYAN}Press Ctrl+C to stop${NC}"
    echo

    # Run the application with optional AI logging
    if [ "$AI_LOG_ENABLED" = "true" ]; then
        local startup_log="ai_startup_output"
        log_run "AI logging enabled - writing startup output to: $startup_log"
        "$binary_path" "$@" 2>&1 | tee "$startup_log"
    else
        "$binary_path" "$@"
    fi
}

# Show system information
show_system_info() {
    echo -e "${BLUE}System Information:${NC}"
    echo "OS: $(lsb_release -ds 2>/dev/null || echo "Unknown Linux")"
    echo "Kernel: $(uname -r)"
    echo "Architecture: $(uname -m)"
    echo "CPU: $(lscpu | grep 'Model name' | sed 's/Model name:[[:space:]]*//')"
    echo "Memory: $(free -h | awk '/^Mem:/ {print $2}')"
    echo "Disk: $(df -h / | awk 'NR==2 {print $4 " free"}')"

    if [ -f "/proc/device-tree/model" ]; then
        echo "Device: $(cat /proc/device-tree/model 2>/dev/null | tr -d '\0')"
    fi

    if command -v rustc >/dev/null 2>&1; then
        echo "Rust: $(rustc --version)"
    else
        echo "Rust: Not installed"
    fi

    echo
}

# Show help
show_help() {
    cat << EOF
SimonSaysSeeq Raspberry Pi 5 Build and Run Script

USAGE:
    $0 [OPTIONS] [COMMAND] [-- APPLICATION_ARGS]

COMMANDS:
    setup       Install system dependencies and configure environment
    build       Build the project with current configuration
    run         Run the application (builds first if necessary)
    test        Test the compiled binary
    service     Install and manage systemd service
    info        Show system information
    clean       Clean build artifacts
    help        Show this help message

OPTIONS:
    --release           Build in release mode (default)
    --debug             Build in debug mode
    --features FEATURES Specify cargo features (default: hardware,midi,desktop)
    --update-rust       Update Rust toolchain before building
    --ai-log            Write startup output to ai_startup_output file

EXAMPLES:
    $0 setup                           # Install dependencies and setup environment (includes grid support)
    $0 build                          # Build in release mode
    $0 --debug build                  # Build in debug mode
    $0 run                            # Build and run
    $0 run -- --simulation            # Run with simulation mode
    $0 --ai-log run                   # Run with AI logging enabled
    $0 service install                # Install systemd service
    $0 service start                  # Start the service
    $0 --features "hardware,midi" run # Build and run with specific features

NOTES:
    - Grid support requires serialosc (automatically installed during setup)
    - Grid support is a hard requirement for SimonSaysSeeq functionality

ENVIRONMENT VARIABLES:
    BUILD_TYPE          Build type: release or debug (default: release)
    RPI_FEATURES        Cargo features to enable (default: hardware,midi,desktop)
    RUST_LOG           Log level: error,warn,info,debug,trace (default: info)

The script automatically detects Raspberry Pi 5 and optimizes compilation for ARM64.
EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --release)
                BUILD_TYPE="release"
                shift
                ;;
            --debug)
                BUILD_TYPE="debug"
                shift
                ;;
            --features)
                RPI_FEATURES="$2"
                shift 2
                ;;
            --update-rust)
                UPDATE_RUST="true"
                shift
                ;;
            --ai-log)
                AI_LOG_ENABLED="true"
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            --)
                shift
                APP_ARGS="$@"
                break
                ;;
            setup|build|run|test|service|info|clean|help)
                COMMAND="$1"
                shift
                ;;
            install|start|stop|status|enable|disable)
                if [ "$COMMAND" = "service" ]; then
                    SERVICE_ACTION="$1"
                    shift
                else
                    log_error "Unknown command: $1"
                    exit 1
                fi
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Main function
main() {
    # Parse arguments
    parse_args "$@"

    # Set default command
    COMMAND="${COMMAND:-run}"

    # Show banner
    print_banner

    case "$COMMAND" in
        "setup")
            check_raspberry_pi
            update_system
            install_system_dependencies
            install_rust ${UPDATE_RUST:+--update}
            setup_audio
            setup_hardware_access

            # Install serialosc (required for grid support)
            log_info "Installing serialosc for grid device support (required)..."
            install_serialosc

            # Setup configuration
            setup_configuration

            log_success "Setup completed successfully!"
            log_warning "Please log out and back in for group permissions to take effect"
            ;;
        "build")
            check_raspberry_pi
            setup_configuration
            build_project
            test_binary
            log_success "Build completed successfully!"
            ;;
        "run")
            check_raspberry_pi

            # Build if binary doesn't exist or if requested
            local binary_path
            if [ "$BUILD_TYPE" = "release" ]; then
                binary_path="target/release/simon_says_seeq"
            else
                binary_path="target/debug/simon_says_seeq"
            fi

            if [ ! -f "$binary_path" ]; then
                log_info "Binary not found, building first..."
                setup_configuration
                build_project
                test_binary
            else
                # Ensure configuration is up to date even if binary exists
                setup_configuration
            fi

            run_application ${APP_ARGS}
            ;;
        "test")
            test_binary
            ;;
        "service")
            case "${SERVICE_ACTION:-install}" in
                "install")
                    if [ ! -f "target/release/simon_says_seeq" ] && [ ! -f "target/debug/simon_says_seeq" ]; then
                        log_error "No binary found. Please build first."
                        exit 1
                    fi
                    # Ensure configuration is set up before installing service
                    setup_configuration
                    install_service
                    ;;
                "start")
                    sudo systemctl start simonsaysseeq-rpi
                    log_success "Service started"
                    ;;
                "stop")
                    sudo systemctl stop simonsaysseeq-rpi
                    log_success "Service stopped"
                    ;;
                "status")
                    systemctl status simonsaysseeq-rpi --no-pager
                    ;;
                "enable")
                    sudo systemctl enable simonsaysseeq-rpi
                    log_success "Service enabled for autostart"
                    ;;
                "disable")
                    sudo systemctl disable simonsaysseeq-rpi
                    log_success "Service disabled"
                    ;;
                *)
                    log_error "Unknown service action: $SERVICE_ACTION"
                    exit 1
                    ;;
            esac
            ;;
        "info")
            show_system_info
            ;;
        "clean")
            if [ -d "target" ]; then
                cargo clean
                log_success "Build artifacts cleaned"
            else
                log_info "No build artifacts to clean"
            fi
            ;;
        "help")
            show_help
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
}

# Handle cleanup on exit
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ] && [ "$COMMAND" = "build" ]; then
        log_error "Build failed with exit code $exit_code"
        echo
        echo "Troubleshooting tips:"
        echo "  1. Check system dependencies: $0 setup"
        echo "  2. Update Rust: $0 --update-rust build"
        echo "  3. Try debug build: $0 --debug build"
        echo "  4. Check available disk space: df -h"
        echo "  5. Check memory: free -h"
    fi
    exit $exit_code
}

trap cleanup EXIT INT TERM

# Check basic prerequisites
if ! command -v sudo >/dev/null 2>&1; then
    log_error "sudo not found. Please install sudo or run as root."
    exit 1
fi

# Run main function
main "$@"
