#!/bin/bash

# Fixed SimonSaysSeeq Boot Selector for Norns
# Simplified version with better error handling for overlay systems

# Configuration
CONFIG_FILE="/home/we/.config/simonsaysseeq/startup_mode"
CONFIG_DIR="$(dirname "$CONFIG_FILE")"
TIMEOUT_SECONDS=5
SERVICE_NAME="simonsaysseeq-rust"
LOG_FILE="/tmp/simonsaysseeq_boot_selector.log"

# Input device for buttons (confirmed from your system)
INPUT_DEVICE="/dev/input/event0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging function with timestamp
log_message() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${BLUE}[BOOT-SELECTOR]${NC} $message"
    echo "$timestamp - $message" >> "$LOG_FILE" 2>/dev/null || true
}

# Read current configuration
read_config() {
    if [ -f "$CONFIG_FILE" ]; then
        local mode=$(cat "$CONFIG_FILE" 2>/dev/null | tr -d '\n\r ')
        if [ "$mode" = "rust" ] || [ "$mode" = "menu" ]; then
            echo "$mode"
        else
            echo "menu"
        fi
    else
        echo "menu"
    fi
}

# Write configuration
write_config() {
    local mode="$1"
    mkdir -p "$CONFIG_DIR" 2>/dev/null || true
    echo "$mode" > "$CONFIG_FILE" 2>/dev/null || true
    log_message "Configuration saved: $mode"
}

# Check if systemd service exists
service_exists() {
    systemctl list-unit-files "$SERVICE_NAME.service" >/dev/null 2>&1
}

# Setup Rust mode
setup_rust_mode() {
    log_message "Configuring Direct Rust App mode..."
    
    if ! service_exists; then
        log_message "Warning: systemd service $SERVICE_NAME not found, skipping service setup"
        write_config "rust"
        return 0
    fi
    
    # Enable and start service
    if systemctl enable "$SERVICE_NAME" >/dev/null 2>&1; then
        log_message "✓ Service enabled for auto-start"
        systemctl start "$SERVICE_NAME" >/dev/null 2>&1 || log_message "Service will start on next boot"
        write_config "rust"
        log_message "✓ Rust app mode configured successfully"
        return 0
    else
        log_message "Warning: Failed to enable systemd service, but continuing"
        write_config "rust"
        return 0
    fi
}

# Setup menu mode
setup_menu_mode() {
    log_message "Configuring Normal Menu mode..."
    
    # Disable and stop service if it exists (don't fail if it doesn't work)
    if service_exists; then
        systemctl disable "$SERVICE_NAME" >/dev/null 2>&1 || true
        systemctl stop "$SERVICE_NAME" >/dev/null 2>&1 || true
        log_message "✓ Service disabled"
    fi
    
    write_config "menu"
    log_message "✓ Normal menu mode configured successfully"
    return 0
}

# Simple button detection with timeout
detect_button_press() {
    local timeout_seconds="$1"
    
    log_message "Monitoring $INPUT_DEVICE for button presses (${timeout_seconds}s timeout)..."
    log_message "Hold any button for Rust app, or wait for normal menu"
    
    # Check if input device exists and is readable
    if [ ! -c "$INPUT_DEVICE" ] || [ ! -r "$INPUT_DEVICE" ]; then
        log_message "Warning: Cannot read input device $INPUT_DEVICE, using saved configuration"
        return 2
    fi
    
    # Simple detection - any input activity during timeout = rust mode
    local temp_file="/tmp/button_detect_$$"
    
    # Monitor input device in background
    (
        timeout $timeout_seconds cat "$INPUT_DEVICE" >/dev/null 2>&1
        echo $? > "$temp_file"
    ) &
    
    local monitor_pid=$!
    
    # Wait for timeout or activity
    sleep $timeout_seconds
    
    # Kill background process if still running
    kill $monitor_pid 2>/dev/null || true
    wait $monitor_pid 2>/dev/null || true
    
    # Check result
    local result=2
    if [ -f "$temp_file" ]; then
        local exit_code=$(cat "$temp_file" 2>/dev/null)
        # timeout command returns 124 on timeout, 0 on activity
        if [ "$exit_code" = "0" ]; then
            result=0  # Activity detected = rust mode
            log_message "Input activity detected - selecting Rust mode"
        else
            result=2  # Timeout = use saved config
            log_message "No input detected - using saved configuration"
        fi
        rm -f "$temp_file" 2>/dev/null || true
    else
        log_message "Input detection failed - using saved configuration"
    fi
    
    return $result
}

# Main execution function
main() {
    log_message "SimonSaysSeeq Boot Selector starting (fixed version)..."
    
    # Create log directory
    mkdir -p "$(dirname "$LOG_FILE")" 2>/dev/null || true
    
    # Check for skip file (testing/debugging)
    if [ -f "/tmp/simonsaysseeq_skip_boot_selector" ]; then
        log_message "Skip file found - using saved configuration only"
        rm -f "/tmp/simonsaysseeq_skip_boot_selector" 2>/dev/null || true
        local saved_mode=$(read_config)
        if [ "$saved_mode" = "rust" ]; then
            setup_rust_mode
        else
            setup_menu_mode
        fi
        return 0
    fi
    
    # Show current saved configuration
    local saved_mode=$(read_config)
    log_message "Current saved mode: $saved_mode"
    
    # Detect button presses
    detect_button_press $TIMEOUT_SECONDS
    local selection_result=$?
    
    # Process the selection
    case $selection_result in
        0)
            log_message "User selected: Direct Rust App"
            setup_rust_mode
            ;;
        1)
            log_message "User selected: Normal Menu"
            setup_menu_mode
            ;;
        2|*)
            log_message "Using saved configuration: $saved_mode"
            if [ "$saved_mode" = "rust" ]; then
                setup_rust_mode
            else
                setup_menu_mode
            fi
            ;;
    esac
    
    log_message "Boot selector completed successfully"
    return 0
}

# Handle cleanup on exit
cleanup() {
    # Clean up any temporary files
    rm -f "/tmp/button_detect_$$" 2>/dev/null || true
    
    # Always exit successfully - don't fail the boot process
    exit 0
}

# Set up signal traps
trap cleanup EXIT INT TERM

# Execute main function
main "$@"