#!/bin/bash

# SimonSaysSeeq Shell Boot Selector Installation Script
# Installs and configures the hardware boot selector for Norns using shell script only

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE_FILE="simonsaysseeq-boot-selector.service"
SERVICE_PATH="/etc/systemd/system/$SERVICE_FILE"
SHELL_SELECTOR="$SCRIPT_DIR/hardware_boot_selector.sh"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "=============================================="
    echo "  SimonSaysSeeq Shell Boot Selector Install"
    echo "=============================================="
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

check_requirements() {
    log_info "Checking system requirements..."
    
    # Check if running as root or with sudo
    if [ "$EUID" -ne 0 ]; then
        log_error "This script must be run as root or with sudo"
        exit 1
    fi
    
    # Check if systemd is available
    if ! command -v systemctl >/dev/null 2>&1; then
        log_error "systemd is required but not found"
        exit 1
    fi
    
    # Check if we're on a Norns-compatible system
    if [ ! -d "/home/we" ]; then
        log_warn "This doesn't appear to be a standard Norns system"
        log_warn "Continuing anyway, but you may need to adjust paths"
    fi
    
    # Check for shell script dependencies
    if ! command -v bc >/dev/null 2>&1; then
        log_error "bc (calculator) is required but not found"
        log_error "Please install: apt-get install bc"
        exit 1
    fi
    
    # Check if shell selector exists
    if [ ! -f "$SHELL_SELECTOR" ]; then
        log_error "Shell boot selector not found at: $SHELL_SELECTOR"
        exit 1
    fi
    
    log_info "✓ System requirements check passed"
}

setup_gpio_permissions() {
    log_info "Setting up GPIO permissions..."
    
    # Add we user to gpio group if it exists
    if getent group gpio >/dev/null 2>&1; then
        usermod -a -G gpio we 2>/dev/null || true
        log_info "✓ Added user 'we' to gpio group"
    else
        log_warn "GPIO group not found, creating it"
        groupadd gpio 2>/dev/null || true
        usermod -a -G gpio we 2>/dev/null || true
    fi
    
    # Add we user to input group for input device access
    if getent group input >/dev/null 2>&1; then
        usermod -a -G input we 2>/dev/null || true
        log_info "✓ Added user 'we' to input group"
    fi
    
    # Set up udev rules for GPIO access
    cat > /etc/udev/rules.d/99-simonsaysseeq-gpio.rules << 'EOF'
# SimonSaysSeeq GPIO access rules
KERNEL=="gpiochip*", GROUP="gpio", MODE="0664"
SUBSYSTEM=="gpio", GROUP="gpio", MODE="0664"
SUBSYSTEM=="input", GROUP="input", MODE="0664"
# Allow gpio export/unexport
SUBSYSTEM=="gpio", KERNEL=="export", GROUP="gpio", MODE="0220"
SUBSYSTEM=="gpio", KERNEL=="unexport", GROUP="gpio", MODE="0220"
EOF
    
    # Reload udev rules
    udevadm control --reload-rules 2>/dev/null || true
    udevadm trigger 2>/dev/null || true
    
    log_info "✓ GPIO permissions configured"
}

install_service() {
    log_info "Installing systemd service..."
    
    # Copy service file
    cp "$SCRIPT_DIR/$SERVICE_FILE" "$SERVICE_PATH"
    
    # Make sure the selector script is executable
    chmod +x "$SHELL_SELECTOR"
    log_info "✓ Made shell selector executable"
    
    # Reload systemd
    systemctl daemon-reload
    
    log_info "✓ Systemd service installed"
}

configure_service() {
    log_info "Configuring boot selector service..."
    
    # Enable the service
    systemctl enable "$SERVICE_FILE"
    log_info "✓ Boot selector service enabled"
    
    # Create config directory
    mkdir -p /home/we/.config/simonsaysseeq
    chown -R we:we /home/we/.config/simonsaysseeq
    
    # Set default configuration if none exists
    if [ ! -f "/home/we/.config/simonsaysseeq/startup_mode" ]; then
        echo "menu" > /home/we/.config/simonsaysseeq/startup_mode
        chown we:we /home/we/.config/simonsaysseeq/startup_mode
        log_info "✓ Default configuration set to menu mode"
    fi
    
    log_info "✓ Boot selector service configured"
}

test_installation() {
    log_info "Testing installation..."
    
    # Check service status
    if systemctl is-enabled "$SERVICE_FILE" >/dev/null 2>&1; then
        log_info "✓ Service is enabled"
    else
        log_error "✗ Service is not enabled"
        return 1
    fi
    
    # Test script syntax
    if bash -n "$SHELL_SELECTOR"; then
        log_info "✓ Shell selector script syntax is valid"
    else
        log_error "✗ Shell selector script has syntax errors"
        return 1
    fi
    
    # Test GPIO sysfs access
    if [ -d "/sys/class/gpio" ]; then
        log_info "✓ GPIO sysfs interface is available"
    else
        log_warn "GPIO sysfs interface not found - hardware detection may not work"
    fi
    
    # Test bc availability
    if echo "1 + 1" | bc >/dev/null 2>&1; then
        log_info "✓ Calculator (bc) is working"
    else
        log_error "✗ Calculator (bc) is not working properly"
        return 1
    fi
    
    log_info "✓ Installation test completed"
}

show_usage_instructions() {
    echo -e "\n${GREEN}=============================================="
    echo "  Installation Completed Successfully!"
    echo -e "===============================================${NC}\n"
    
    echo -e "${BLUE}How to use the boot selector:${NC}"
    echo "1. During Norns startup, you have 5 seconds to make a choice:"
    echo "   - Hold K2: Boot directly to SimonSaysSeeqRust app"
    echo "   - Hold K3: Boot to normal Norns menu"
    echo "   - Do nothing: Use previously saved preference"
    echo ""
    echo -e "${BLUE}Manual control (via SSH):${NC}"
    echo "   ./toggle_startup_mode.sh         # Interactive menu"
    echo "   ./toggle_startup_mode.sh rust    # Set to Rust app mode"
    echo "   ./toggle_startup_mode.sh menu    # Set to menu mode"
    echo "   ./toggle_startup_mode.sh status  # Check current status"
    echo ""
    echo -e "${BLUE}Service management:${NC}"
    echo "   sudo systemctl status simonsaysseeq-boot-selector"
    echo "   sudo systemctl disable simonsaysseeq-boot-selector  # Remove boot selector"
    echo "   sudo systemctl enable simonsaysseeq-boot-selector   # Re-enable boot selector"
    echo ""
    echo -e "${BLUE}Logs and debugging:${NC}"
    echo "   tail -f /tmp/simonsaysseeq_boot_selector.log"
    echo "   journalctl -u simonsaysseeq-boot-selector -f"
    echo ""
    echo -e "${BLUE}Test the selector without rebooting:${NC}"
    echo "   sudo $SHELL_SELECTOR"
    echo ""
    echo -e "${YELLOW}Note: Reboot your Norns to test the boot selector${NC}"
    echo -e "${YELLOW}The selector will activate during the boot process${NC}"
}

uninstall() {
    log_info "Uninstalling boot selector..."
    
    # Stop and disable service
    systemctl stop "$SERVICE_FILE" 2>/dev/null || true
    systemctl disable "$SERVICE_FILE" 2>/dev/null || true
    
    # Remove service file
    rm -f "$SERVICE_PATH"
    
    # Remove udev rules
    rm -f /etc/udev/rules.d/99-simonsaysseeq-gpio.rules
    
    # Reload systemd
    systemctl daemon-reload
    udevadm control --reload-rules 2>/dev/null || true
    
    log_info "✓ Boot selector uninstalled"
    echo -e "${GREEN}Boot selector has been removed. Norns will boot normally.${NC}"
}

show_help() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --uninstall, -u    Uninstall the boot selector"
    echo "  --help, -h         Show this help message"
    echo "  --test             Test the selector without installing"
    echo ""
    echo "This script installs a shell-based hardware boot selector that allows"
    echo "you to choose between normal Norns menu and direct Rust app boot by"
    echo "holding buttons during startup."
    echo ""
    echo "Requirements:"
    echo "  - Norns device with standard GPIO button layout"
    echo "  - Root/sudo access for systemd service installation"
    echo "  - bc (calculator) command for timing calculations"
    echo ""
    echo "Button mappings:"
    echo "  - K2 (GPIO 27): Select Rust app mode"
    echo "  - K3 (GPIO 22): Select normal menu mode"
}

test_selector() {
    log_info "Testing boot selector (dry run)..."
    
    if [ ! -f "$SHELL_SELECTOR" ]; then
        log_error "Shell selector not found: $SHELL_SELECTOR"
        exit 1
    fi
    
    # Test script syntax
    if ! bash -n "$SHELL_SELECTOR"; then
        log_error "Shell selector has syntax errors"
        exit 1
    fi
    
    log_info "✓ Shell selector syntax is valid"
    
    # Create a temporary skip file so the selector doesn't actually run
    echo "test-mode" > /tmp/simonsaysseeq_skip_boot_selector
    
    log_info "Running selector in test mode..."
    
    # Run the selector
    if "$SHELL_SELECTOR"; then
        log_info "✓ Boot selector test completed successfully"
    else
        log_error "✗ Boot selector test failed"
        rm -f /tmp/simonsaysseeq_skip_boot_selector
        exit 1
    fi
    
    # Cleanup
    rm -f /tmp/simonsaysseeq_skip_boot_selector
    
    log_info "Test completed - selector is ready for installation"
}

main() {
    print_banner
    
    # Parse command line arguments
    case "${1:-}" in
        "--uninstall"|"-u")
            uninstall
            exit 0
            ;;
        "--help"|"-h")
            show_help
            exit 0
            ;;
        "--test")
            test_selector
            exit 0
            ;;
        "")
            # Continue with installation
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
    
    # Main installation process
    check_requirements
    setup_gpio_permissions
    install_service
    configure_service
    test_installation
    
    show_usage_instructions
}

# Handle cleanup on exit
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        log_error "Installation failed with exit code $exit_code"
        log_info "You can try running with --test to check for issues"
    fi
    exit $exit_code
}

trap cleanup EXIT INT TERM

# Run main function
main "$@"