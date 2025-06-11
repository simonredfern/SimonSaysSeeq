#!/bin/bash

# Debug script to detect Norns input devices and button codes
# This will help identify the correct device paths and button mappings for the boot selector

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_banner() {
    echo -e "${BLUE}"
    echo "=============================================="
    echo "  Norns Input Device Detection & Debug Tool  "
    echo "=============================================="
    echo -e "${NC}"
}

# Function to check input devices
check_input_devices() {
    log_info "Scanning available input devices..."
    
    if [ ! -d "/dev/input" ]; then
        log_error "/dev/input directory not found"
        return 1
    fi
    
    echo -e "\n${BLUE}Available input devices:${NC}"
    ls -la /dev/input/
    
    echo -e "\n${BLUE}Input device details from /proc/bus/input/devices:${NC}"
    if [ -f "/proc/bus/input/devices" ]; then
        cat /proc/bus/input/devices
    else
        log_warn "/proc/bus/input/devices not found"
    fi
    
    # Check each event device
    echo -e "\n${BLUE}Testing each event device:${NC}"
    for device in /dev/input/event*; do
        if [ -c "$device" ]; then
            echo -e "\n${YELLOW}Device: $device${NC}"
            
            # Check if we can read from it
            if [ -r "$device" ]; then
                echo "  ✓ Readable"
                
                # Try to get device info using evtest if available
                if command -v evtest >/dev/null 2>&1; then
                    echo "  Device info (evtest):"
                    timeout 1 evtest "$device" 2>/dev/null | head -10 || echo "  Could not get evtest info"
                fi
            else
                echo "  ✗ Not readable (permissions issue)"
                ls -la "$device"
            fi
        fi
    done
}

# Function to monitor input for button presses
monitor_button_presses() {
    local device="$1"
    local timeout_seconds="${2:-10}"
    
    if [ ! -c "$device" ]; then
        log_error "Device $device not found"
        return 1
    fi
    
    if [ ! -r "$device" ]; then
        log_error "Cannot read from device $device (check permissions)"
        return 1
    fi
    
    log_info "Monitoring $device for button presses..."
    log_info "Press K1, K2, K3 buttons during the next ${timeout_seconds} seconds"
    log_info "Press Ctrl+C to stop early"
    
    echo -e "\n${BLUE}Raw hex data (timestamp + type + code + value):${NC}"
    
    # Monitor with hexdump and parse the data
    timeout ${timeout_seconds} hexdump -C "$device" 2>/dev/null | while read line; do
        # Skip empty lines
        [ -z "$line" ] && continue
        
        echo "$line"
        
        # Parse the hex data to extract button information
        local hex_data=$(echo "$line" | sed 's/^[0-9a-f]*\s*//' | sed 's/\s*|.*$//' | tr -d ' ')
        
        # Need at least 32 hex chars (16 bytes) for a complete input event
        if [ ${#hex_data} -ge 32 ]; then
            # Extract type (bytes 16-17)
            local type=$(echo "$hex_data" | cut -c17-18)
            # Extract code (bytes 20-21)  
            local code=$(echo "$hex_data" | cut -c21-22)
            # Extract value (bytes 24-25)
            local value=$(echo "$hex_data" | cut -c25-26)
            
            # Decode event type
            local event_type="unknown"
            case "$type" in
                "01") event_type="EV_KEY (button)" ;;
                "02") event_type="EV_REL (relative)" ;;
                "03") event_type="EV_ABS (absolute)" ;;
                "00") event_type="EV_SYN (sync)" ;;
            esac
            
            # Decode value
            local action="unknown"
            case "$value" in
                "00") action="RELEASE" ;;
                "01") action="PRESS" ;;
                "02") action="REPEAT" ;;
            esac
            
            # Only show key events
            if [ "$type" = "01" ]; then
                echo -e "  ${GREEN}→ Button Event: Type=$type ($event_type), Code=$code, Value=$value ($action)${NC}"
                
                # Map to likely button names based on common codes
                local button_name="unknown"
                case "$code" in
                    "01") button_name="K1" ;;
                    "02") button_name="K2" ;;
                    "03") button_name="K3" ;;
                    "1c") button_name="ENTER" ;;
                    "39") button_name="SPACE" ;;
                esac
                
                if [ "$button_name" != "unknown" ]; then
                    echo -e "  ${YELLOW}→ Likely button: $button_name${NC}"
                fi
            fi
        fi
    done
    
    echo -e "\n${GREEN}Monitoring completed${NC}"
}

# Function to test all event devices for button activity
test_all_devices() {
    local timeout_seconds="${1:-5}"
    
    log_info "Testing all event devices for button activity..."
    log_info "Press buttons during the next ${timeout_seconds} seconds"
    
    for device in /dev/input/event*; do
        if [ -c "$device" ] && [ -r "$device" ]; then
            echo -e "\n${YELLOW}Testing $device...${NC}"
            
            # Quick test for activity
            local activity=$(timeout 1 hexdump -C "$device" 2>/dev/null | head -1)
            if [ -n "$activity" ]; then
                echo -e "  ${GREEN}✓ Activity detected on $device${NC}"
                echo "  Sample: $activity"
            else
                echo -e "  ${BLUE}• No immediate activity on $device${NC}"
            fi
        else
            echo -e "\n${RED}✗ Cannot read $device${NC}"
        fi
    done
}

# Function to check overlays and device tree
check_overlays() {
    log_info "Checking device tree overlays..."
    
    echo -e "\n${BLUE}Boot config overlays:${NC}"
    if [ -f "/boot/config.txt" ]; then
        grep -i overlay /boot/config.txt || echo "No overlays found in /boot/config.txt"
    else
        log_warn "/boot/config.txt not found"
    fi
    
    echo -e "\n${BLUE}Available overlays:${NC}"
    if [ -d "/boot/overlays" ]; then
        ls -la /boot/overlays/*norns* 2>/dev/null || echo "No norns overlays found"
        ls -la /boot/overlays/*button* 2>/dev/null || echo "No button overlays found"
    else
        log_warn "/boot/overlays directory not found"
    fi
    
    echo -e "\n${BLUE}Device tree information:${NC}"
    if [ -d "/proc/device-tree" ]; then
        find /proc/device-tree -name "*button*" -o -name "*key*" 2>/dev/null | head -10
    else
        log_warn "/proc/device-tree not found"
    fi
}

# Function to check permissions and groups
check_permissions() {
    log_info "Checking permissions and user groups..."
    
    echo -e "\n${BLUE}Current user groups:${NC}"
    groups
    
    echo -e "\n${BLUE}Input device permissions:${NC}"
    ls -la /dev/input/event* 2>/dev/null | head -5
    
    echo -e "\n${BLUE}Group memberships:${NC}"
    echo "input group:" $(getent group input 2>/dev/null || echo "not found")
    echo "gpio group:" $(getent group gpio 2>/dev/null || echo "not found")
    
    echo -e "\n${BLUE}Udev rules for input devices:${NC}"
    if [ -d "/etc/udev/rules.d" ]; then
        ls -la /etc/udev/rules.d/*input* /etc/udev/rules.d/*gpio* 2>/dev/null || echo "No relevant udev rules found"
    fi
}

# Main execution
main() {
    print_banner
    
    case "${1:-}" in
        "devices"|"dev")
            check_input_devices
            ;;
        "monitor"|"mon")
            local device="${2:-/dev/input/event0}"
            local timeout="${3:-10}"
            monitor_button_presses "$device" "$timeout"
            ;;
        "test"|"t")
            local timeout="${2:-5}"
            test_all_devices "$timeout"
            ;;
        "overlays"|"dt")
            check_overlays
            ;;
        "permissions"|"perm")
            check_permissions
            ;;
        "all"|"")
            check_input_devices
            echo -e "\n" && check_overlays
            echo -e "\n" && check_permissions
            echo -e "\n" && test_all_devices 3
            ;;
        "help"|"-h"|"--help")
            echo "Usage: $0 [command] [options]"
            echo ""
            echo "Commands:"
            echo "  devices, dev              List and check input devices"
            echo "  monitor, mon [device] [timeout]  Monitor specific device for button presses"
            echo "  test, t [timeout]         Test all devices for activity"
            echo "  overlays, dt              Check device tree overlays"
            echo "  permissions, perm         Check permissions and groups"
            echo "  all (default)             Run all checks"
            echo "  help                      Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                        # Run all diagnostics"
            echo "  $0 monitor /dev/input/event0 15  # Monitor event0 for 15 seconds"
            echo "  $0 test 10                # Test all devices for 10 seconds"
            echo ""
            echo "This script helps debug input device detection for the Norns boot selector."
            ;;
        *)
            log_error "Unknown command: $1"
            echo "Run '$0 help' for usage information"
            exit 1
            ;;
    esac
}

# Make sure we can access input devices
if [ "$EUID" -ne 0 ] && [ "$(id -u)" != "0" ]; then
    log_warn "Running as non-root user. Some operations may require sudo."
    log_warn "If you get permission errors, try: sudo $0 $*"
fi

main "$@"