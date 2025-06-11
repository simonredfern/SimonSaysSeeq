#!/bin/bash

# Manual Boot Mode Toggle for SimonSaysSeeq
# This script allows you to manually set the boot mode without rebooting

set -e

# Configuration
CONFIG_FILE="/home/we/.config/simonsaysseeq/startup_mode"
CONFIG_DIR="$(dirname "$CONFIG_FILE")"
SERVICE_NAME="simonsaysseeq-rust"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_banner() {
    echo -e "${BLUE}"
    echo "=========================================="
    echo "  SimonSaysSeeq Manual Boot Mode Toggle  "
    echo "=========================================="
    echo -e "${NC}"
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
        log_error "Failed to write config file $CONFIG_FILE"
        return 1
    }
    log_info "Configuration saved: $mode"
}

# Check if systemd service exists
service_exists() {
    systemctl list-unit-files "$SERVICE_NAME.service" 2>/dev/null | grep -q "^$SERVICE_NAME.service" 2>/dev/null
}

# Get service status
get_service_status() {
    if service_exists; then
        if systemctl is-active "$SERVICE_NAME" >/dev/null 2>&1; then
            echo "running"
        elif systemctl is-enabled "$SERVICE_NAME" >/dev/null 2>&1; then
            echo "enabled"
        else
            echo "disabled"
        fi
    else
        echo "not_found"
    fi
}

# Setup Rust mode
setup_rust_mode() {
    log_info "Configuring Direct Rust App mode..."
    
    if ! service_exists; then
        log_error "systemd service $SERVICE_NAME not found!"
        log_error "Please deploy the Rust service first"
        return 1
    fi
    
    # Enable and start service
    if systemctl enable "$SERVICE_NAME" 2>/dev/null; then
        log_info "✓ Service enabled for auto-start"
        
        if systemctl start "$SERVICE_NAME" 2>/dev/null; then
            log_info "✓ Service started immediately"
        else
            log_warn "Service will start on next boot"
        fi
        
        write_config "rust"
        log_info "✓ Rust app mode configured successfully"
        return 0
    else
        log_error "Failed to enable systemd service"
        return 1
    fi
}

# Setup menu mode
setup_menu_mode() {
    log_info "Configuring Normal Menu mode..."
    
    # Disable and stop service if it exists
    if service_exists; then
        if systemctl stop "$SERVICE_NAME" 2>/dev/null; then
            log_info "✓ Service stopped"
        fi
        
        if systemctl disable "$SERVICE_NAME" 2>/dev/null; then
            log_info "✓ Service disabled"
        fi
    fi
    
    write_config "menu"
    log_info "✓ Normal menu mode configured successfully"
}

# Show current status
show_status() {
    local current_mode=$(read_config)
    local service_status=$(get_service_status)
    
    echo -e "\n${BLUE}Current Status:${NC}"
    echo "  Boot Mode: $current_mode"
    echo "  Service Status: $service_status"
    
    if [ -f "$CONFIG_FILE" ]; then
        echo "  Config File: $CONFIG_FILE (exists)"
    else
        echo "  Config File: $CONFIG_FILE (missing)"
    fi
    
    if service_exists; then
        echo -e "\n${BLUE}Service Details:${NC}"
        systemctl status "$SERVICE_NAME" --no-pager -l 2>/dev/null | head -10 || true
    else
        echo -e "\n${YELLOW}Service $SERVICE_NAME not found${NC}"
    fi
}

# Interactive menu
interactive_menu() {
    while true; do
        local current_mode=$(read_config)
        local service_status=$(get_service_status)
        
        echo -e "\n${BLUE}Current mode: $current_mode (service: $service_status)${NC}"
        echo ""
        echo "Choose an option:"
        echo "  1) Set to Rust App mode"
        echo "  2) Set to Normal Menu mode"
        echo "  3) Show detailed status"
        echo "  4) Test boot selector manually"
        echo "  5) View logs"
        echo "  6) Exit"
        echo ""
        read -p "Enter choice [1-6]: " choice
        
        case $choice in
            1)
                setup_rust_mode
                ;;
            2)
                setup_menu_mode
                ;;
            3)
                show_status
                ;;
            4)
                test_boot_selector
                ;;
            5)
                view_logs
                ;;
            6)
                log_info "Goodbye!"
                exit 0
                ;;
            *)
                log_error "Invalid choice. Please enter 1-6."
                ;;
        esac
    done
}

# Test boot selector manually
test_boot_selector() {
    log_info "Testing boot selector manually..."
    
    # Check if the boot selector script exists
    local script_candidates=(
        "/home/we/SimonSaysSeeq/SimonSaysSeeq/SimonSaysSeeqNornsRust/hardware_boot_selector.sh"
        "/home/we/SimonSaysSeeq/SimonSaysSeeq/SimonSaysSeeqNornsRust/simple_boot_selector.sh"
        "/home/we/dust/code/SimonSaysSeeqRust/hardware_boot_selector.sh"
        "./hardware_boot_selector.sh"
        "./simple_boot_selector.sh"
    )
    
    local boot_script=""
    for script in "${script_candidates[@]}"; do
        if [ -f "$script" ] && [ -x "$script" ]; then
            boot_script="$script"
            break
        fi
    done
    
    if [ -z "$boot_script" ]; then
        log_error "Boot selector script not found in expected locations"
        log_info "Looked for scripts at:"
        for script in "${script_candidates[@]}"; do
            echo "  - $script"
        done
        return 1
    fi
    
    log_info "Found boot selector script: $boot_script"
    log_info "Creating skip file to prevent service conflicts..."
    
    # Create skip file so the selector runs in test mode
    echo "manual-test" > /tmp/simonsaysseeq_skip_boot_selector
    
    log_info "Running boot selector..."
    if "$boot_script"; then
        log_info "✓ Boot selector test completed successfully"
    else
        log_error "✗ Boot selector test failed"
    fi
    
    # Cleanup
    rm -f /tmp/simonsaysseeq_skip_boot_selector 2>/dev/null || true
}

# View logs
view_logs() {
    echo -e "\n${BLUE}Recent boot selector logs:${NC}"
    if [ -f "/tmp/simonsaysseeq_boot_selector.log" ]; then
        tail -20 /tmp/simonsaysseeq_boot_selector.log
    else
        log_warn "No boot selector log file found"
    fi
    
    echo -e "\n${BLUE}Recent systemd service logs:${NC}"
    if service_exists; then
        journalctl -u "$SERVICE_NAME" -n 10 --no-pager 2>/dev/null || log_warn "Could not retrieve service logs"
    else
        log_warn "Service not found"
    fi
    
    echo -e "\n${BLUE}Recent boot selector service logs:${NC}"
    journalctl -u "simonsaysseeq-boot-selector" -n 10 --no-pager 2>/dev/null || log_warn "Could not retrieve boot selector service logs"
}

# Show help
show_help() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  rust          Set boot mode to Rust app"
    echo "  menu          Set boot mode to normal menu"
    echo "  status        Show current status"
    echo "  test          Test boot selector manually"
    echo "  logs          View recent logs"
    echo "  interactive   Interactive menu (default)"
    echo "  help          Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 rust       # Set to Rust app mode"
    echo "  $0 menu       # Set to normal menu mode"
    echo "  $0 status     # Show current configuration"
    echo "  $0            # Interactive menu"
    echo ""
    echo "This script allows you to manually control the SimonSaysSeeq boot mode"
    echo "without rebooting or waiting for the boot selector timeout."
}

# Force modes for testing
force_rust_mode() {
    log_info "Creating force rust mode trigger..."
    echo "force" > /tmp/simonsaysseeq_force_rust
    log_info "✓ Next boot selector run will choose Rust mode"
}

force_menu_mode() {
    log_info "Creating force menu mode trigger..."
    echo "force" > /tmp/simonsaysseeq_force_menu
    log_info "✓ Next boot selector run will choose Menu mode"
}

# Main execution
main() {
    print_banner
    
    # Check if we need root permissions for some operations
    if [ "$EUID" -ne 0 ] && [ "$(id -u)" != "0" ]; then
        log_warn "Running as non-root user. Some operations may require sudo."
    fi
    
    case "${1:-interactive}" in
        "rust"|"r")
            setup_rust_mode
            ;;
        "menu"|"m")
            setup_menu_mode
            ;;
        "status"|"s")
            show_status
            ;;
        "test"|"t")
            test_boot_selector
            ;;
        "logs"|"l")
            view_logs
            ;;
        "interactive"|"i"|"")
            interactive_menu
            ;;
        "force-rust")
            force_rust_mode
            ;;
        "force-menu")
            force_menu_mode
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            log_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

main "$@"