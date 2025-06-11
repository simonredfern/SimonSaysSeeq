#!/bin/bash

# SimonSaysSeeq Hardware Boot Selector (Shell Script)
# Checks Norns hardware buttons during boot to determine startup mode
# K2 = Direct Rust App, K3 = Normal Menu, No input = Use saved setting

set -e

# Configuration
CONFIG_FILE="/home/we/.config/simonsaysseeq/startup_mode"
CONFIG_DIR="$(dirname "$CONFIG_FILE")"
TIMEOUT_SECONDS=5
CHECK_INTERVAL=0.1
SERVICE_NAME="simonsaysseeq-rust"
LOG_FILE="/tmp/simonsaysseeq_boot_selector.log"

# Norns GPIO pin mappings (adjust these based on your specific Norns model)
# These are typical mappings - may need adjustment for your hardware
K1_GPIO=17  # Key 1 (not used in this selector)
K2_GPIO=27  # Key 2 - Select Rust app
K3_GPIO=22  # Key 3 - Select Normal menu

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log_message() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${BLUE}[$(basename $0)]${NC} $message"
    echo "$timestamp - $message" >> "$LOG_FILE"
}

# Setup GPIO pins for input
setup_gpio() {
    log_message "Setting up GPIO pins for button detection..."
    
    # Export GPIO pins if not already exported
    for pin in $K2_GPIO $K3_GPIO; do
        if [ ! -d "/sys/class/gpio/gpio$pin" ]; then
            echo $pin > /sys/class/gpio/export 2>/dev/null || true
        fi
        # Set as input with pull-up
        echo "in" > /sys/class/gpio/gpio$pin/direction 2>/dev/null || true
        echo "both" > /sys/class/gpio/gpio$pin/edge 2>/dev/null || true
    done
    
    # Wait a moment for GPIO to stabilize
    sleep 0.2
}

# Clean up GPIO on exit
cleanup_gpio() {
    log_message "Cleaning up GPIO pins..."
    for pin in $K2_GPIO $K3_GPIO; do
        if [ -d "/sys/class/gpio/gpio$pin" ]; then
            echo $pin > /sys/class/gpio/unexport 2>/dev/null || true
        fi
    done
}

# Read GPIO pin state (0 = pressed, 1 = released on Norns)
read_gpio() {
    local pin=$1
    if [ -f "/sys/class/gpio/gpio$pin/value" ]; then
        cat "/sys/class/gpio/gpio$pin/value" 2>/dev/null || echo "1"
    else
        echo "1" # Default to not pressed
    fi
}

# Alternative method using /dev/input if GPIO doesn't work
check_input_devices() {
    # Look for Norns input devices
    local input_devices=$(find /dev/input -name "event*" 2>/dev/null | head -3)
    
    if [ -n "$input_devices" ]; then
        log_message "Found input devices: $input_devices"
        # Use evtest or similar tool if available
        # This is a fallback method
        return 0
    else
        log_message "No input devices found"
        return 1
    fi
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
    mkdir -p "$CONFIG_DIR"
    echo "$mode" > "$CONFIG_FILE"
    log_message "Saved configuration: $mode"
}

# Check if systemd service exists
service_exists() {
    systemctl list-unit-files | grep -q "^$SERVICE_NAME.service" 2>/dev/null
}

# Setup Rust mode
setup_rust_mode() {
    log_message "Setting up Direct Rust App mode..."
    
    if ! service_exists; then
        log_message "ERROR: systemd service $SERVICE_NAME not found!"
        log_message "Please run ./deploy_to_norns.sh first"
        return 1
    fi
    
    # Enable and configure service
    systemctl enable "$SERVICE_NAME" 2>/dev/null || {
        log_message "Failed to enable service (may need sudo)"
        return 1
    }
    
    # Save config
    write_config "rust"
    
    # Start service now
    systemctl start "$SERVICE_NAME" 2>/dev/null || {
        log_message "Failed to start service immediately"
        log_message "Service will start on next boot"
    }
    
    log_message "✓ Rust app mode configured"
    return 0
}

# Setup menu mode
setup_menu_mode() {
    log_message "Setting up Normal Menu mode..."
    
    if service_exists; then
        # Disable and stop service
        systemctl disable "$SERVICE_NAME" 2>/dev/null || true
        systemctl stop "$SERVICE_NAME" 2>/dev/null || true
    fi
    
    # Save config
    write_config "menu"
    
    log_message "✓ Normal menu mode configured"
    return 0
}

# Check for button presses
check_buttons() {
    local k2_pressed=0
    local k3_pressed=0
    local elapsed=0
    
    log_message "Checking for button presses (timeout: ${TIMEOUT_SECONDS}s)..."
    log_message "Hold K2 for Rust app, K3 for normal menu"
    
    # Setup GPIO
    if ! setup_gpio; then
        log_message "GPIO setup failed, using saved configuration"
        return 2 # Use saved config
    fi
    
    # Monitor buttons for timeout period
    while [ $(echo "$elapsed < $TIMEOUT_SECONDS" | bc -l 2>/dev/null || [ $elapsed -lt $TIMEOUT_SECONDS ]) = 1 ]; do
        # Read button states
        local k2_state=$(read_gpio $K2_GPIO)
        local k3_state=$(read_gpio $K3_GPIO)
        
        # On Norns, buttons are typically active-low (0 = pressed)
        if [ "$k2_state" = "0" ]; then
            k2_pressed=1
            log_message "K2 (Rust app) button detected!"
            break
        fi
        
        if [ "$k3_state" = "0" ]; then
            k3_pressed=1
            log_message "K3 (Normal menu) button detected!"
            break
        fi
        
        sleep $CHECK_INTERVAL
        elapsed=$(echo "$elapsed + $CHECK_INTERVAL" | bc -l 2>/dev/null || echo $((elapsed + 1)))
    done
    
    cleanup_gpio
    
    # Return selection
    if [ $k2_pressed -eq 1 ]; then
        return 0 # Rust mode
    elif [ $k3_pressed -eq 1 ]; then
        return 1 # Menu mode
    else
        return 2 # Use saved config
    fi
}

# Alternative button check using input events
check_buttons_input() {
    log_message "Trying input device method..."
    
    # This is a simplified version - in practice you'd use evtest or similar
    local timeout_counter=0
    local max_timeout=$((TIMEOUT_SECONDS * 10)) # Convert to deciseconds
    
    while [ $timeout_counter -lt $max_timeout ]; do
        # Check if any input events are available
        # This is a placeholder - real implementation would read input events
        
        # For now, just timeout to saved config
        sleep 0.1
        timeout_counter=$((timeout_counter + 1))
    done
    
    return 2 # Use saved config
}

# Main execution
main() {
    log_message "SimonSaysSeeq Hardware Boot Selector starting..."
    
    # Check if we should skip (for testing/debugging)
    if [ -f "/tmp/simonsaysseeq_skip_boot_selector" ]; then
        log_message "Skip file found, using saved configuration"
        rm -f "/tmp/simonsaysseeq_skip_boot_selector"
        local saved_mode=$(read_config)
        log_message "Using saved mode: $saved_mode"
        exit 0
    fi
    
    # Show current saved configuration
    local saved_mode=$(read_config)
    log_message "Current saved mode: $saved_mode"
    
    # Check for button presses
    check_buttons
    local selection_result=$?
    
    case $selection_result in
        0)
            log_message "User selected: Direct Rust App"
            if setup_rust_mode; then
                log_message "✓ Rust app mode activated"
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
            log_message "No button pressed, using saved configuration: $saved_mode"
            if [ "$saved_mode" = "rust" ]; then
                if setup_rust_mode; then
                    log_message "✓ Using saved Rust app mode"
                else
                    log_message "✗ Saved Rust mode failed, switching to menu"
                    setup_menu_mode
                fi
            else
                setup_menu_mode
            fi
            ;;
        *)
            log_message "Unexpected result, defaulting to menu mode"
            setup_menu_mode
            ;;
    esac
    
    log_message "Hardware boot selector completed"
}

# Handle signals for cleanup
trap cleanup_gpio EXIT INT TERM

# Ensure bc is available for floating point arithmetic
if ! command -v bc >/dev/null 2>&1; then
    log_message "Warning: bc not available, using integer arithmetic"
fi

# Create log directory
mkdir -p "$(dirname "$LOG_FILE")"

# Run main function
main "$@"

exit 0