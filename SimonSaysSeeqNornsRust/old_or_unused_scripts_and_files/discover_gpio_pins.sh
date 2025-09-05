#!/bin/bash

# GPIO Pin Discovery and Testing Script for Norns
# This script helps you find the correct GPIO pin mappings for your Norns buttons
# Run this to determine the correct K2_GPIO and K3_GPIO values before deploying

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Common GPIO pins to test (typical Raspberry Pi pins)
COMMON_GPIO_PINS="2 3 4 17 18 22 23 24 25 27"

# GPIO pins that are exported for monitoring
EXPORTED_PINS=""

print_banner() {
    echo -e "${BLUE}"
    echo "================================================"
    echo "  Norns GPIO Pin Discovery & Testing Tool"
    echo "================================================"
    echo -e "${NC}"
    echo "This script helps you find the correct GPIO pins"
    echo "for your Norns K2 and K3 buttons."
    echo ""
}

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${CYAN}[STEP]${NC} $1"
}

# Clean up exported GPIO pins
cleanup_gpio() {
    if [ -n "$EXPORTED_PINS" ]; then
        log_info "Cleaning up exported GPIO pins..."
        for pin in $EXPORTED_PINS; do
            if [ -d "/sys/class/gpio/gpio$pin" ]; then
                echo $pin > /sys/class/gpio/unexport 2>/dev/null || true
            fi
        done
        EXPORTED_PINS=""
    fi
}

# Set up signal handlers
trap cleanup_gpio EXIT INT TERM

# Check if GPIO sysfs is available
check_gpio_availability() {
    if [ ! -d "/sys/class/gpio" ]; then
        log_error "GPIO sysfs interface not available at /sys/class/gpio"
        log_error "This script requires GPIO sysfs support"
        exit 1
    fi
    
    if [ ! -w "/sys/class/gpio/export" ]; then
        log_error "Cannot write to /sys/class/gpio/export"
        log_error "Please run this script as root: sudo $0"
        exit 1
    fi
    
    log_info "GPIO sysfs interface is available"
}

# Export a GPIO pin for testing
export_gpio_pin() {
    local pin=$1
    
    # Skip if already exported
    if [ -d "/sys/class/gpio/gpio$pin" ]; then
        return 0
    fi
    
    if echo $pin > /sys/class/gpio/export 2>/dev/null; then
        EXPORTED_PINS="$EXPORTED_PINS $pin"
        sleep 0.1
        
        # Set as input with pull-up
        echo "in" > /sys/class/gpio/gpio$pin/direction 2>/dev/null || return 1
        sleep 0.1
        return 0
    else
        return 1
    fi
}

# Read GPIO pin value
read_gpio_value() {
    local pin=$1
    if [ -f "/sys/class/gpio/gpio$pin/value" ]; then
        cat "/sys/class/gpio/gpio$pin/value" 2>/dev/null || echo "ERROR"
    else
        echo "ERROR"
    fi
}

# Test a single GPIO pin by monitoring for changes
test_gpio_pin() {
    local pin=$1
    local duration=${2:-3}
    
    if ! export_gpio_pin $pin; then
        return 1
    fi
    
    # Get initial value
    local initial_value=$(read_gpio_value $pin)
    if [ "$initial_value" = "ERROR" ]; then
        return 1
    fi
    
    echo -n "  Pin $pin (initial: $initial_value) - "
    
    # Monitor for changes
    local start_time=$(date +%s)
    local end_time=$((start_time + duration))
    local changes=0
    local last_value=$initial_value
    
    while [ $(date +%s) -lt $end_time ]; do
        local current_value=$(read_gpio_value $pin)
        if [ "$current_value" != "$last_value" ] && [ "$current_value" != "ERROR" ]; then
            changes=$((changes + 1))
            echo -n "[$last_value→$current_value] "
            last_value=$current_value
        fi
        sleep 0.1
    done
    
    local final_value=$(read_gpio_value $pin)
    echo "final: $final_value, changes: $changes"
    
    return $changes
}

# Discover which pins are available
discover_available_pins() {
    log_step "Discovering available GPIO pins..."
    
    local available_pins=""
    
    for pin in $COMMON_GPIO_PINS; do
        if export_gpio_pin $pin; then
            available_pins="$available_pins $pin"
            echo "  ✓ GPIO $pin - Available"
        else
            echo "  ✗ GPIO $pin - Not available or in use"
        fi
    done
    
    if [ -z "$available_pins" ]; then
        log_error "No GPIO pins are available for testing!"
        exit 1
    fi
    
    echo ""
    log_info "Available GPIO pins:$available_pins"
    return 0
}

# Interactive button testing
interactive_button_test() {
    log_step "Interactive button testing..."
    echo ""
    echo "This test will monitor GPIO pins while you press buttons."
    echo "Press buttons when prompted to see which GPIO pins respond."
    echo ""
    
    # Get list of available pins
    local test_pins=""
    for pin in $COMMON_GPIO_PINS; do
        if [ -d "/sys/class/gpio/gpio$pin" ]; then
            test_pins="$test_pins $pin"
        fi
    done
    
    if [ -z "$test_pins" ]; then
        log_error "No GPIO pins available for testing"
        return 1
    fi
    
    echo -e "${YELLOW}Test 1: Press and hold K2 button NOW${NC}"
    echo "Monitoring for 5 seconds..."
    echo ""
    
    local k2_candidates=""
    for pin in $test_pins; do
        test_gpio_pin $pin 5
        local changes=$?
        if [ $changes -gt 0 ]; then
            k2_candidates="$k2_candidates $pin"
        fi
    done
    
    echo ""
    echo -e "${YELLOW}Test 2: Press and hold K3 button NOW${NC}"
    echo "Monitoring for 5 seconds..."
    echo ""
    
    local k3_candidates=""
    for pin in $test_pins; do
        test_gpio_pin $pin 5
        local changes=$?
        if [ $changes -gt 0 ]; then
            k3_candidates="$k3_candidates $pin"
        fi
    done
    
    echo ""
    log_info "Results:"
    echo "  K2 button candidates:$k2_candidates"
    echo "  K3 button candidates:$k3_candidates"
    
    # Suggest pin mappings
    if [ -n "$k2_candidates" ] || [ -n "$k3_candidates" ]; then
        echo ""
        echo -e "${GREEN}Suggested GPIO pin mappings:${NC}"
        
        # Pick first candidate from each
        local suggested_k2=$(echo $k2_candidates | awk '{print $1}')
        local suggested_k3=$(echo $k3_candidates | awk '{print $1}')
        
        if [ -n "$suggested_k2" ]; then
            echo "K2_GPIO=$suggested_k2  # Key 2 button"
        fi
        
        if [ -n "$suggested_k3" ]; then
            echo "K3_GPIO=$suggested_k3  # Key 3 button"
        fi
    else
        log_warn "No button activity detected on any GPIO pins"
        log_warn "Your Norns may use different GPIO pins or input methods"
    fi
}

# Check existing Norns configuration files
check_norns_config() {
    log_step "Checking for existing Norns configuration..."
    
    local config_files=(
        "/home/we/norns/lua/core/hardware.lua"
        "/home/we/norns/lua/core/input.lua" 
        "/home/we/dust/code/norns/lua/core/hardware.lua"
        "/etc/norns.conf"
        "/boot/config.txt"
    )
    
    for config_file in "${config_files[@]}"; do
        if [ -f "$config_file" ]; then
            echo "Checking $config_file..."
            
            # Look for GPIO or button configuration
            local gpio_lines=$(grep -i -E "(gpio|button|key)" "$config_file" 2>/dev/null | head -5)
            if [ -n "$gpio_lines" ]; then
                echo "  Found GPIO/button configuration:"
                echo "$gpio_lines" | sed 's/^/    /'
            fi
        fi
    done
    
    # Check device tree overlays
    if [ -f "/boot/config.txt" ]; then
        echo ""
        echo "Device tree overlays in /boot/config.txt:"
        grep -E "^dtoverlay=" /boot/config.txt 2>/dev/null | sed 's/^/  /' || echo "  None found"
    fi
}

# Check input devices
check_input_devices() {
    log_step "Checking input devices..."
    
    echo "Available input devices:"
    if [ -d "/dev/input" ]; then
        ls -la /dev/input/ | sed 's/^/  /'
        echo ""
        
        echo "Input device information:"
        for device in /dev/input/event*; do
            if [ -c "$device" ]; then
                echo "  $device:"
                # Try to get device info (requires root)
                if command -v udevadm >/dev/null 2>&1; then
                    udevadm info --name="$device" 2>/dev/null | grep -E "(ID_INPUT|ID_VENDOR|NAME)" | sed 's/^/    /' || echo "    No info available"
                fi
            fi
        done
    else
        echo "  /dev/input directory not found"
    fi
}

# Monitor all GPIO pins simultaneously
monitor_all_pins() {
    log_step "Monitoring all available GPIO pins simultaneously..."
    echo ""
    echo "Press any button on your Norns to see which GPIO pins change."
    echo "Press Ctrl+C to stop monitoring."
    echo ""
    
    # Get baseline values
    local baseline_file="/tmp/gpio_baseline"
    > "$baseline_file"
    
    for pin in $COMMON_GPIO_PINS; do
        if [ -d "/sys/class/gpio/gpio$pin" ]; then
            local value=$(read_gpio_value $pin)
            echo "$pin=$value" >> "$baseline_file"
        fi
    done
    
    echo "Baseline GPIO values:"
    cat "$baseline_file" | sed 's/^/  /'
    echo ""
    echo "Monitoring for changes (press any Norns button)..."
    
    local iteration=0
    while true; do
        sleep 0.2
        iteration=$((iteration + 1))
        
        local changes_detected=false
        for pin in $COMMON_GPIO_PINS; do
            if [ -d "/sys/class/gpio/gpio$pin" ]; then
                local current_value=$(read_gpio_value $pin)
                local baseline_value=$(grep "^$pin=" "$baseline_file" | cut -d'=' -f2)
                
                if [ "$current_value" != "$baseline_value" ] && [ "$current_value" != "ERROR" ]; then
                    echo "  GPIO $pin: $baseline_value → $current_value"
                    changes_detected=true
                    # Update baseline
                    sed -i "s/^$pin=.*/$pin=$current_value/" "$baseline_file"
                fi
            fi
        done
        
        # Show periodic status
        if [ $((iteration % 25)) -eq 0 ]; then
            echo "  [$(date +%H:%M:%S)] Still monitoring... (Ctrl+C to stop)"
        fi
    done
}

# Generate configuration for hardware_boot_selector.sh
generate_config() {
    echo ""
    log_step "Configuration Generator"
    echo ""
    echo "Based on your testing, enter the GPIO pin numbers:"
    echo ""
    
    read -p "K2 GPIO pin number (or press Enter to skip): " k2_pin
    read -p "K3 GPIO pin number (or press Enter to skip): " k3_pin
    
    if [ -n "$k2_pin" ] || [ -n "$k3_pin" ]; then
        local config_file="gpio_config.sh"
        echo "# GPIO Configuration for SimonSaysSeeq Hardware Boot Selector" > "$config_file"
        echo "# Generated on $(date)" >> "$config_file"
        echo "" >> "$config_file"
        
        if [ -n "$k2_pin" ]; then
            echo "K2_GPIO=$k2_pin  # Key 2 - Select Rust app" >> "$config_file"
        else
            echo "# K2_GPIO=27  # Key 2 - Select Rust app (not configured)" >> "$config_file"
        fi
        
        if [ -n "$k3_pin" ]; then
            echo "K3_GPIO=$k3_pin  # Key 3 - Select Normal menu" >> "$config_file"
        else
            echo "# K3_GPIO=22  # Key 3 - Select Normal menu (not configured)" >> "$config_file"
        fi
        
        echo "" >> "$config_file"
        echo "# To use this configuration, edit hardware_boot_selector.sh and update the pin values" >> "$config_file"
        
        log_info "Configuration saved to $config_file"
        echo ""
        echo "To apply this configuration:"
        echo "1. Edit hardware_boot_selector.sh"
        echo "2. Update the K2_GPIO and K3_GPIO values at the top of the file"
        echo "3. Test with: sudo ./hardware_boot_selector.sh"
    fi
}

# Main menu
show_menu() {
    echo ""
    echo -e "${CYAN}What would you like to do?${NC}"
    echo "1. Discover available GPIO pins"
    echo "2. Interactive button testing"
    echo "3. Monitor all pins simultaneously"
    echo "4. Check Norns configuration files"
    echo "5. Check input devices"
    echo "6. Generate configuration file"
    echo "7. Quick test (all of the above)"
    echo "8. Exit"
    echo ""
    read -p "Enter choice [1-8]: " choice
}

# Quick test - runs all discovery methods
quick_test() {
    log_info "Running complete GPIO discovery test..."
    echo ""
    
    discover_available_pins
    echo ""
    
    check_norns_config
    echo ""
    
    check_input_devices
    echo ""
    
    interactive_button_test
    echo ""
    
    generate_config
}

# Main execution
main() {
    print_banner
    
    # Check requirements
    check_gpio_availability
    
    # Handle command line arguments
    case "${1:-}" in
        "discover"|"--discover")
            discover_available_pins
            exit 0
            ;;
        "test"|"--test")
            discover_available_pins
            interactive_button_test
            exit 0
            ;;
        "monitor"|"--monitor")
            discover_available_pins
            monitor_all_pins
            exit 0
            ;;
        "quick"|"--quick")
            quick_test
            exit 0
            ;;
        "help"|"--help"|"-h")
            echo "Usage: $0 [option]"
            echo ""
            echo "Options:"
            echo "  discover   Discover available GPIO pins"
            echo "  test       Interactive button testing"
            echo "  monitor    Monitor all pins simultaneously"
            echo "  quick      Run complete discovery test"
            echo "  help       Show this help"
            echo ""
            echo "Interactive mode: Run without arguments"
            exit 0
            ;;
    esac
    
    # Interactive mode
    while true; do
        show_menu
        
        case $choice in
            1)
                discover_available_pins
                ;;
            2)
                discover_available_pins
                interactive_button_test
                ;;
            3)
                discover_available_pins
                monitor_all_pins
                ;;
            4)
                check_norns_config
                ;;
            5)
                check_input_devices
                ;;
            6)
                generate_config
                ;;
            7)
                quick_test
                ;;
            8)
                log_info "Goodbye!"
                exit 0
                ;;
            *)
                log_error "Invalid option. Please choose 1-8."
                ;;
        esac
        
        echo ""
        read -p "Press Enter to continue..."
    done
}

# Execute main function
main "$@"