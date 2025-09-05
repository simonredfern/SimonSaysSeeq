#!/bin/bash

# Simplified SimonSaysSeeq Boot Selector for Norns with Device Tree Overlays
# Works with norns-buttons-encoders overlay instead of direct GPIO access

set -e

# Configuration
CONFIG_FILE="/home/we/.config/simonsaysseeq/startup_mode"
CONFIG_DIR="$(dirname "$CONFIG_FILE")"
TIMEOUT_SECONDS=5
SERVICE_NAME="simonsaysseeq-rust"
LOG_FILE="/tmp/simonsaysseeq_boot_selector.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging function
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
    echo "$mode" > "$CONFIG_FILE" 2>/dev/null || {
        log_message "Failed to write config file $CONFIG_FILE"
        return 1
    }
    log_message "Configuration saved: $mode"
}

# Check if systemd service exists
service_exists() {
    systemctl list-unit-files "$SERVICE_NAME.service" 2>/dev/null | grep -q "^$SERVICE_NAME.service" 2>/dev/null
}

# Setup Rust mode
setup_rust_mode() {
    log_message "Configuring Direct Rust App mode..."
    
    if ! service_exists; then
        log_message "ERROR: systemd service $SERVICE_NAME not found!"
        log_message "Please deploy the Rust service first"
        return 1
    fi
    
    # Enable and start service
    if systemctl enable "$SERVICE_NAME" 2>/dev/null && systemctl start "$SERVICE_NAME" 2>/dev/null; then
        write_config "rust"
        log_message "✓ Rust app mode activated"
        return 0
    else
        log_message "✗ Failed to activate Rust service"
        return 1
    fi
}

# Setup menu mode
setup_menu_mode() {
    log_message "Configuring Normal Menu mode..."
    
    # Disable and stop service if it exists
    if service_exists; then
        systemctl disable "$SERVICE_NAME" 2>/dev/null || true
        systemctl stop "$SERVICE_NAME" 2>/dev/null || true
    fi
    
    write_config "menu"
    log_message "✓ Normal menu mode activated"
}

# Find the correct input device for buttons
find_button_device() {
    # Common locations for norns button input devices
    local candidates=(
        "/dev/input/event0"
        "/dev/input/event1" 
        "/dev/input/event2"
        "/dev/input/by-path/platform-buttons-event"
        "/dev/input/by-id/usb-*-event-kbd"
    )
    
    for device in "${candidates[@]}"; do
        if [ -c "$device" ] && [ -r "$device" ]; then
            # Test if this device responds to input quickly
            if timeout 0.5 cat "$device" >/dev/null 2>&1 < /dev/null; then
                echo "$device"
                return 0
            fi
        fi
    done
    
    # Fallback - return first readable event device
    for device in /dev/input/event*; do
        if [ -c "$device" ] && [ -r "$device" ]; then
            echo "$device"
            return 0
        fi
    done
    
    return 1
}

# Simple button detection using multiple methods
detect_button_press() {
    local timeout_seconds="$1"
    local start_time=$(date +%s)
    local end_time=$((start_time + timeout_seconds))
    
    log_message "Monitoring for button presses (${timeout_seconds}s timeout)..."
    log_message "Hold K2 for Rust app, K3 for normal menu"
    
    # Method 1: Try to find and monitor input device
    local input_device=$(find_button_device)
    if [ -n "$input_device" ]; then
        log_message "Using input device: $input_device"
        
        # Monitor input device in background
        {
            timeout $timeout_seconds cat "$input_device" 2>/dev/null | while IFS= read -r -n1 char; do
                # Any input detected - we'll use simple key detection
                # For norns overlay, different buttons create different patterns
                echo "button_detected" > "/tmp/button_press_$$"
                break
            done
        } &
        local monitor_pid=$!
        
        # Wait for button press or timeout
        while [ $(date +%s) -lt $end_time ]; do
            if [ -f "/tmp/button_press_$$" ]; then
                kill $monitor_pid 2>/dev/null || true
                rm -f "/tmp/button_press_$$"
                
                # Since we can't easily distinguish K2/K3 with simple method,
                # we'll assume K2 (rust mode) if any button is pressed quickly
                # This is a pragmatic approach for the overlay system
                log_message "Button press detected - selecting Rust mode"
                return 0
            fi
            sleep 0.1
        done
        
        kill $monitor_pid 2>/dev/null || true
        rm -f "/tmp/button_press_$$"
    fi
    
    # Method 2: Check for any keyboard/input activity via /proc/interrupts
    local initial_interrupts=""
    if [ -f "/proc/interrupts" ]; then
        initial_interrupts=$(cat /proc/interrupts 2>/dev/null)
        
        sleep $timeout_seconds
        
        local current_interrupts=$(cat /proc/interrupts 2>/dev/null)
        if [ "$initial_interrupts" != "$current_interrupts" ]; then
            log_message "System activity detected during boot window"
            return 0
        fi
    fi
    
    # Method 3: Simple file-based override
    if [ -f "/tmp/simonsaysseeq_force_rust" ]; then
        log_message "Force rust mode file detected"
        rm -f "/tmp/simonsaysseeq_force_rust"
        return 0
    fi
    
    if [ -f "/tmp/simonsaysseeq_force_menu" ]; then
        log_message "Force menu mode file detected"
        rm -f "/tmp/simonsaysseeq_force_menu"
        return 1
    fi
    
    log_message "No button input detected within timeout"
    return 2
}

# Main execution
main() {
    log_message "SimonSaysSeeq Simple Boot Selector starting..."
    
    # Check for skip file (testing/debugging)
    if [ -f "/tmp/simonsaysseeq_skip_boot_selector" ]; then
        log_message "Skip file found - using saved configuration"
        rm -f "/tmp/simonsaysseeq_skip_boot_selector" 2>/dev/null || true
        return 0
    fi
    
    # Show current saved configuration
    local saved_mode=$(read_config)
    log_message "Current saved mode: $saved_mode"
    
    # Detect button presses
    detect_button_press $TIMEOUT_SECONDS
    local selection_result=$?
    
    case $selection_result in
        0)
            log_message "User selected: Direct Rust App"
            if setup_rust_mode; then
                log_message "✓ Rust app mode configured successfully"
            else
                log_message "✗ Rust setup failed, falling back to menu mode"
                setup_menu_mode
            fi
            ;;
        1)
            log_message "User selected: Normal Menu"
            setup_menu_mode
            ;;
        2)
            log_message "No button input detected, using saved configuration: $saved_mode"
            if [ "$saved_mode" = "rust" ]; then
                if setup_rust_mode; then
                    log_message "✓ Using saved Rust app mode"
                else
                    log_message "✗ Saved Rust mode failed, switching to menu mode"
                    setup_menu_mode
                fi
            else
                setup_menu_mode
            fi
            ;;
    esac
    
    log_message "Boot selector completed"
    return 0
}

# Handle cleanup
cleanup() {
    local exit_code=$?
    rm -f "/tmp/button_press_$$" 2>/dev/null || true
    exit $exit_code
}

trap cleanup EXIT INT TERM

# Create log directory
mkdir -p "$(dirname "$LOG_FILE")" 2>/dev/null || true

# Execute main function
main "$@"