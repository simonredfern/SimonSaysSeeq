#!/bin/bash

# SimonSaysSeeq Hardware Boot Selector (Production Ready)
# Detects Norns hardware buttons during boot to determine startup mode
# K2 = Direct Rust App, K3 = Normal Menu, No input = Use saved setting

# Removed 'set -e' to allow handling of non-zero return codes from detection functions

# Version for deployment tracking
BOOT_SELECTOR_VERSION="v2.3-fixed-exit-handling-$(date +%Y%m%d-%H%M)"

# Configuration
CONFIG_FILE="/home/we/.config/simonsaysseeq/startup_mode"
CONFIG_DIR="$(dirname "$CONFIG_FILE")"
TIMEOUT_SECONDS=5
CHECK_INTERVAL=0.1
SERVICE_NAME="simonsaysseeq-rust"
LOG_FILE="/tmp/simonsaysseeq_boot_selector.log"

# Norns input device configuration (norns-buttons-encoders overlay)
INPUT_DEVICE="/dev/input/event0"  # Button input device
K2_CODE="02"  # Key 2 code - Select Rust app
K3_CODE="03"  # Key 3 code - Select Normal menu

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Background monitoring process PID
MONITOR_PID=""

# Logging function with timestamp
log_message() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${BLUE}[HW-BOOT-SELECTOR]${NC} $message"
    echo "$timestamp - $message" >> "$LOG_FILE" 2>/dev/null || true
}

# Clean up background processes on exit
cleanup_processes() {
    if [ -n "$MONITOR_PID" ]; then
        kill $MONITOR_PID 2>/dev/null || true
        wait $MONITOR_PID 2>/dev/null || true
        MONITOR_PID=""
    fi
}

# Check if input device is available
check_input_device() {
    if [ ! -c "$INPUT_DEVICE" ]; then
        log_message "Input device $INPUT_DEVICE not available"
        return 1
    fi
    
    if [ ! -r "$INPUT_DEVICE" ]; then
        log_message "Cannot read from input device $INPUT_DEVICE"
        return 1
    fi
    
    return 0
}

# Parse input event data
parse_input_event() {
    local hex_line="$1"
    
    # Extract just the hex bytes from hexdump output (remove address and ASCII)
    local hex_data=$(echo "$hex_line" | sed 's/^[0-9a-f]*\s*//' | sed 's/\s*|.*$//' | tr -d ' ')
    
    # Skip lines that don't have enough data (need at least 32 hex chars = 16 bytes)
    if [ ${#hex_data} -lt 32 ]; then
        echo ""
        return
    fi
    
    # Input event structure in hex: timestamp(16) + type(4) + code(4) + value(8)
    # Extract type (bytes 16-17, should be "01" for EV_KEY)
    local type=$(echo "$hex_data" | cut -c17-18)
    
    # Extract code (bytes 20-21, "02" for K2, "03" for K3)  
    local code=$(echo "$hex_data" | cut -c21-22)
    
    # Extract value (bytes 24-25, "01" for press, "00" for release)
    local value=$(echo "$hex_data" | cut -c25-26)
    
    # Return button code only for key press events (type=01, value=01)
    if [ "$type" = "01" ] && [ "$value" = "01" ]; then
        echo "$code"
    else
        echo ""
    fi
}

# Monitor input device for button presses
monitor_input_device() {
    local timeout=$1
    local result_file="/tmp/button_result_$$"
    
    # Check if input device is available
    if ! check_input_device; then
        return 2
    fi
    
    log_message "Monitoring input device $INPUT_DEVICE for button presses..."
    
    # Start background monitoring
    (
        timeout $timeout hexdump -C "$INPUT_DEVICE" 2>/dev/null | while read line; do
            # Parse the input event from the hexdump line
            local button_code=$(parse_input_event "$line")
            
            if [ -n "$button_code" ]; then
                case "$button_code" in
                    "$K2_CODE")
                        echo "0" > "$result_file"
                        break
                        ;;
                    "$K3_CODE")
                        echo "1" > "$result_file"
                        break
                        ;;
                esac
            fi
        done
    ) &
    
    MONITOR_PID=$!
    
    # Wait for result or timeout
    local start_time=$(date +%s)
    local end_time=$((start_time + timeout))
    
    while [ $(date +%s) -lt $end_time ]; do
        if [ -f "$result_file" ]; then
            local result=$(cat "$result_file" 2>/dev/null)
            rm -f "$result_file" 2>/dev/null || true
            
            # Clean up background process
            kill $MONITOR_PID 2>/dev/null || true
            wait $MONITOR_PID 2>/dev/null || true
            MONITOR_PID=""
            
            if [ "$result" = "0" ]; then
                log_message "K2 button press detected"
                return 0
            elif [ "$result" = "1" ]; then
                log_message "K3 button press detected"  
                return 1
            fi
        fi
        sleep $CHECK_INTERVAL
    done
    
    # Cleanup on timeout
    if [ -n "$MONITOR_PID" ]; then
        kill $MONITOR_PID 2>/dev/null || true
        wait $MONITOR_PID 2>/dev/null || true
        MONITOR_PID=""
    fi
    rm -f "$result_file" 2>/dev/null || true
    
    log_message "Input device monitoring timeout - no buttons pressed"
    return 2
}

# Simplified interrupt check (no longer used but kept for compatibility)
check_interrupt_based() {
    log_message "Interrupt monitoring skipped for reliability"
    return 2
}

# Simplified fallback check (no longer used but kept for compatibility)
check_input_devices_fallback() {
    log_message "Fallback detection skipped for reliability"
    return 2
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
    return 0
}

# Check if systemd service exists
service_exists() {
    systemctl list-unit-files "$SERVICE_NAME.service" 2>/dev/null | grep -q "^$SERVICE_NAME.service" 2>/dev/null
}

# Run system command with error handling
run_command() {
    local cmd="$1"
    local description="$2"
    
    log_message "Executing: $description"
    
    if eval "$cmd" >/dev/null 2>&1; then
        log_message "✓ $description completed successfully"
        return 0
    else
        local exit_code=$?
        log_message "✗ $description failed (exit code: $exit_code)"
        return $exit_code
    fi
}

# Setup Rust mode
setup_rust_mode() {
    log_message "Configuring Direct Rust App mode..."
    
    if ! service_exists; then
        log_message "ERROR: systemd service $SERVICE_NAME not found!"
        log_message "Please run ./deploy_to_norns.sh first to install the service"
        return 1
    fi
    
    # Enable service for auto-start
    if run_command "systemctl enable $SERVICE_NAME" "Enable systemd service"; then
        write_config "rust"
        
        # Try to start service immediately (non-critical if it fails)
        run_command "systemctl start $SERVICE_NAME" "Start service immediately" || {
            log_message "Service will start on next boot"
        }
        
        log_message "✓ Rust app mode configured successfully"
        return 0
    else
        log_message "✗ Failed to enable systemd service"
        return 1
    fi
}

# Setup menu mode
setup_menu_mode() {
    log_message "Configuring Normal Menu mode..."
    
    # Disable and stop service if it exists
    if service_exists; then
        run_command "systemctl disable $SERVICE_NAME" "Disable systemd service" || true
        run_command "systemctl stop $SERVICE_NAME" "Stop systemd service" || true
    fi
    
    write_config "menu"
    log_message "✓ Normal menu mode configured successfully"
    return 0
}

# Main execution function
main() {
    log_message "SimonSaysSeeq Hardware Boot Selector starting..."
    log_message "Version: $BOOT_SELECTOR_VERSION"
    
    # Check for skip file (testing/debugging)
    if [ -f "/tmp/simonsaysseeq_skip_boot_selector" ]; then
        log_message "Skip file found - using saved configuration"
        rm -f "/tmp/simonsaysseeq_skip_boot_selector" 2>/dev/null || true
        return 0
    fi
    
    # Show current saved configuration
    local saved_mode=$(read_config)
    log_message "Current saved mode: $saved_mode"
    log_message "Monitoring for button presses (timeout: ${TIMEOUT_SECONDS}s)"
    log_message "Hold K2 for Rust app, K3 for normal menu"
    
    # Try button detection methods in order of reliability
    local selection_result=2
    local detection_method="none"
    
    # Method 1: Input device monitoring (primary method for norns-buttons-encoders overlay)
    log_message "Attempting input device button detection..."
    monitor_input_device $TIMEOUT_SECONDS
    selection_result=$?
    
    if [ $selection_result -ne 2 ]; then
        detection_method="input-device"
    else
        # Skip all fallback methods for reliability
        log_message "Primary input device timed out, proceeding with saved configuration"
        detection_method="timeout"
    fi
    
    # Process the selection
    case $selection_result in
        0)
            log_message "User selected: Direct Rust App (via $detection_method)"
            if setup_rust_mode; then
                log_message "✓ Rust app mode activated successfully"
            else
                log_message "✗ Rust setup failed, falling back to menu mode"
                setup_menu_mode
            fi
            ;;
        1)
            log_message "User selected: Normal Menu (via $detection_method)"
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
        *)
            log_message "Unexpected selection result: $selection_result, defaulting to menu mode"
            setup_menu_mode
            ;;
    esac
    
    log_message "Hardware boot selector completed successfully"
    return 0
}

# Signal handlers for clean shutdown
cleanup_and_exit() {
    local exit_code=${1:-0}
    log_message "Cleaning up and exiting with code $exit_code"
    cleanup_processes
    exit $exit_code
}

# Set up signal traps
trap 'cleanup_and_exit 130' INT
trap 'cleanup_and_exit 143' TERM
trap 'cleanup_processes' EXIT

# Validate environment before starting
if [ ! -c "$INPUT_DEVICE" ]; then
    log_message "WARNING: Input device $INPUT_DEVICE not available"
fi

if ! command -v bc >/dev/null 2>&1; then
    log_message "WARNING: bc calculator not available, using integer arithmetic"
fi

# Create log directory
mkdir -p "$(dirname "$LOG_FILE")" 2>/dev/null || true

# Execute main function
main "$@"
exit_code=$?

# Clean up and exit
cleanup_processes
exit $exit_code