#!/bin/bash

# Deploy SimonSaysSeeq Hardware Boot Selector to Norns
# This script deploys the boot selector, installer, and related files to your Norns device

set -e

# Configuration
NORNS_IP="${NORNS_IP:-norns.local}"
NORNS_USER="${NORNS_USER:-we}"
REMOTE_DIR="/home/we/dust/code/SimonSaysSeeqRust"
LOCAL_SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Files to deploy
BOOT_SELECTOR_FILES=(
    "hardware_boot_selector.sh"
    "install_boot_selector.sh"
    "toggle_startup_mode.sh"
    "simonsaysseeq-boot-selector.service"
)

OPTIONAL_FILES=(
    "HARDWARE_BOOT_SELECTOR.md"
    "STARTUP_CONTROL.md"
    "GPIO_DISCOVERY.md"
    "README_BOOT_SELECTOR.md"
    "discover_gpio_pins.sh"
)

print_banner() {
    echo -e "${BLUE}"
    echo "=================================================="
    echo "  Deploy Hardware Boot Selector to Norns"
    echo "=================================================="
    echo -e "${NC}"
    echo "This script deploys the complete boot selector system"
    echo "to your Norns device for hardware button startup control."
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

check_local_files() {
    log_step "Checking local files..."
    
    local missing_files=0
    
    # Check required files
    for file in "${BOOT_SELECTOR_FILES[@]}"; do
        if [ ! -f "$LOCAL_SCRIPT_DIR/$file" ]; then
            log_error "Required file missing: $file"
            missing_files=$((missing_files + 1))
        else
            echo "  ✓ $file"
        fi
    done
    
    # Check optional files
    local optional_count=0
    for file in "${OPTIONAL_FILES[@]}"; do
        if [ -f "$LOCAL_SCRIPT_DIR/$file" ]; then
            echo "  ✓ $file (optional)"
            optional_count=$((optional_count + 1))
        else
            echo "  - $file (optional, not found)"
        fi
    done
    
    if [ $missing_files -gt 0 ]; then
        log_error "$missing_files required files are missing"
        exit 1
    fi
    
    log_info "✓ All required files found ($optional_count optional files available)"
}

make_scripts_executable() {
    log_step "Making scripts executable..."
    
    for file in "${BOOT_SELECTOR_FILES[@]}"; do
        if [[ "$file" == *.sh ]]; then
            chmod +x "$LOCAL_SCRIPT_DIR/$file" 2>/dev/null || true
            echo "  ✓ $file"
        fi
    done
    
    # Make optional scripts executable too
    for file in "${OPTIONAL_FILES[@]}"; do
        if [[ "$file" == *.sh ]] && [ -f "$LOCAL_SCRIPT_DIR/$file" ]; then
            chmod +x "$LOCAL_SCRIPT_DIR/$file" 2>/dev/null || true
            echo "  ✓ $file"
        fi
    done
    
    log_info "✓ Scripts made executable"
}

test_ssh_connection() {
    log_step "Testing SSH connection to Norns..."
    
    if ! command -v ssh >/dev/null 2>&1; then
        log_error "SSH client not found"
        exit 1
    fi
    
    if ! command -v scp >/dev/null 2>&1; then
        log_error "SCP not found"
        exit 1
    fi
    
    if ! ssh -o ConnectTimeout=10 -o BatchMode=yes "$NORNS_USER@$NORNS_IP" "echo 'SSH test successful'" >/dev/null 2>&1; then
        log_error "Cannot connect to Norns at $NORNS_IP"
        log_error "Please check:"
        log_error "  1. Norns is powered on and connected to network"
        log_error "  2. SSH is enabled (SYSTEM > WIFI > ADD/EDIT > SSH ON)"
        log_error "  3. IP address is correct: $NORNS_IP"
        log_error "  4. You can SSH manually: ssh $NORNS_USER@$NORNS_IP"
        echo ""
        echo "To specify a different IP:"
        echo "  NORNS_IP=192.168.1.100 $0"
        exit 1
    fi
    
    log_info "✓ SSH connection successful"
}

create_remote_directory() {
    log_step "Creating remote directory structure..."
    
    ssh "$NORNS_USER@$NORNS_IP" "mkdir -p $REMOTE_DIR" || {
        log_error "Failed to create remote directory $REMOTE_DIR"
        exit 1
    }
    
    log_info "✓ Remote directory ready: $REMOTE_DIR"
}

deploy_files() {
    log_step "Deploying boot selector files..."
    
    local deployed_count=0
    local failed_count=0
    
    # Deploy required files
    for file in "${BOOT_SELECTOR_FILES[@]}"; do
        if scp "$LOCAL_SCRIPT_DIR/$file" "$NORNS_USER@$NORNS_IP:$REMOTE_DIR/" >/dev/null 2>&1; then
            echo "  ✓ $file"
            deployed_count=$((deployed_count + 1))
        else
            echo "  ✗ $file"
            failed_count=$((failed_count + 1))
        fi
    done
    
    # Deploy optional files
    for file in "${OPTIONAL_FILES[@]}"; do
        if [ -f "$LOCAL_SCRIPT_DIR/$file" ]; then
            if scp "$LOCAL_SCRIPT_DIR/$file" "$NORNS_USER@$NORNS_IP:$REMOTE_DIR/" >/dev/null 2>&1; then
                echo "  ✓ $file (optional)"
                deployed_count=$((deployed_count + 1))
            else
                echo "  ✗ $file (optional, failed)"
            fi
        fi
    done
    
    if [ $failed_count -gt 0 ]; then
        log_error "$failed_count files failed to deploy"
        exit 1
    fi
    
    log_info "✓ $deployed_count files deployed successfully"
}

set_remote_permissions() {
    log_step "Setting permissions on remote files..."
    
    # Make scripts executable on remote
    ssh "$NORNS_USER@$NORNS_IP" "cd $REMOTE_DIR && chmod +x *.sh 2>/dev/null || true" || {
        log_warn "Failed to set some script permissions (may be normal)"
    }
    
    # Verify key files are executable
    local key_scripts="hardware_boot_selector.sh install_boot_selector.sh toggle_startup_mode.sh"
    for script in $key_scripts; do
        ssh "$NORNS_USER@$NORNS_IP" "chmod +x $REMOTE_DIR/$script" || {
            log_error "Failed to make $script executable"
            exit 1
        }
        echo "  ✓ $script"
    done
    
    log_info "✓ Remote permissions configured"
}

verify_deployment() {
    log_step "Verifying deployment..."
    
    # Check that files exist and are accessible
    for file in "${BOOT_SELECTOR_FILES[@]}"; do
        if ssh "$NORNS_USER@$NORNS_IP" "test -f $REMOTE_DIR/$file" >/dev/null 2>&1; then
            echo "  ✓ $file exists"
        else
            log_error "$file not found on remote"
            exit 1
        fi
    done
    
    # Test that key scripts are executable
    if ssh "$NORNS_USER@$NORNS_IP" "test -x $REMOTE_DIR/hardware_boot_selector.sh" >/dev/null 2>&1; then
        echo "  ✓ hardware_boot_selector.sh is executable"
    else
        log_error "hardware_boot_selector.sh is not executable"
        exit 1
    fi
    
    # Check input device availability
    if ssh "$NORNS_USER@$NORNS_IP" "test -c /dev/input/event0" >/dev/null 2>&1; then
        echo "  ✓ Input device /dev/input/event0 available"
    else
        log_warn "Input device /dev/input/event0 not found (may affect button detection)"
    fi
    
    log_info "✓ Deployment verification completed"
}

show_next_steps() {
    echo ""
    echo -e "${GREEN}=================================================="
    echo "  Deployment Completed Successfully!"
    echo -e "==================================================${NC}"
    echo ""
    echo -e "${BLUE}Boot Selector files are now on your Norns.${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo ""
    echo "1. SSH to your Norns:"
    echo "   ssh $NORNS_USER@$NORNS_IP"
    echo ""
    echo "2. Navigate to the directory:"
    echo "   cd $REMOTE_DIR"
    echo ""
    echo "3. Test the boot selector manually:"
    echo "   sudo ./hardware_boot_selector.sh"
    echo "   (Press K2 or K3 during the test to verify button detection)"
    echo ""
    echo "4. Install the boot selector for startup:"
    echo "   sudo ./install_boot_selector.sh"
    echo ""
    echo "5. Reboot to test boot-time selection:"
    echo "   sudo reboot"
    echo ""
    echo -e "${BLUE}Boot selector controls:${NC}"
    echo "   • Hold K2 during boot: Direct Rust app"
    echo "   • Hold K3 during boot: Normal Norns menu"
    echo "   • No buttons: Use saved preference"
    echo ""
    echo -e "${BLUE}Manual control (after installation):${NC}"
    echo "   ./toggle_startup_mode.sh status    # Check current mode"
    echo "   ./toggle_startup_mode.sh rust      # Set Rust app mode"
    echo "   ./toggle_startup_mode.sh menu      # Set menu mode"
    echo ""
    echo -e "${YELLOW}Files deployed:${NC}"
    for file in "${BOOT_SELECTOR_FILES[@]}"; do
        echo "   • $file"
    done
    
    echo ""
    echo -e "${CYAN}Troubleshooting:${NC}"
    echo "   tail -f /tmp/simonsaysseeq_boot_selector.log"
    echo "   systemctl status simonsaysseeq-boot-selector"
    echo "   ./install_boot_selector.sh --uninstall"
}

show_help() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Environment variables:"
    echo "  NORNS_IP      IP address of Norns (default: norns.local)"
    echo "  NORNS_USER    Username on Norns (default: we)"
    echo ""
    echo "Options:"
    echo "  --help, -h    Show this help message"
    echo "  --verify      Only verify deployment (don't deploy)"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Deploy to norns.local"
    echo "  NORNS_IP=192.168.1.100 $0           # Deploy to specific IP"
    echo "  NORNS_USER=myuser NORNS_IP=myip $0   # Custom user and IP"
    echo ""
    echo "This script deploys the complete hardware boot selector system"
    echo "to your Norns device, including:"
    echo "  • Boot selector script"
    echo "  • Installation script"
    echo "  • Manual control script"
    echo "  • systemd service file"
    echo "  • Documentation"
}

verify_only() {
    log_step "Verification mode - checking existing deployment..."
    
    test_ssh_connection
    
    # Check if files exist on remote
    local missing_files=0
    for file in "${BOOT_SELECTOR_FILES[@]}"; do
        if ssh "$NORNS_USER@$NORNS_IP" "test -f $REMOTE_DIR/$file" >/dev/null 2>&1; then
            echo "  ✓ $file exists on remote"
        else
            echo "  ✗ $file missing on remote"
            missing_files=$((missing_files + 1))
        fi
    done
    
    if [ $missing_files -eq 0 ]; then
        log_info "✓ All files are deployed"
        
        # Check if installed
        if ssh "$NORNS_USER@$NORNS_IP" "systemctl is-enabled simonsaysseeq-boot-selector >/dev/null 2>&1" >/dev/null 2>&1; then
            log_info "✓ Boot selector service is installed and enabled"
        else
            log_warn "Boot selector service is not installed (run install_boot_selector.sh)"
        fi
    else
        log_error "$missing_files files are missing from remote"
        echo "Run deployment without --verify to deploy missing files"
        exit 1
    fi
}

main() {
    # Handle command line arguments
    case "${1:-}" in
        "--help"|"-h"|"help")
            show_help
            exit 0
            ;;
        "--verify")
            print_banner
            log_info "Deploying to: $NORNS_USER@$NORNS_IP"
            verify_only
            exit 0
            ;;
        "")
            # Continue with deployment
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
    
    print_banner
    
    log_info "Deploying to: $NORNS_USER@$NORNS_IP"
    log_info "Remote directory: $REMOTE_DIR"
    echo ""
    
    # Main deployment process
    check_local_files
    make_scripts_executable
    test_ssh_connection
    create_remote_directory
    deploy_files
    set_remote_permissions
    verify_deployment
    
    show_next_steps
}

# Handle Ctrl+C gracefully
trap 'echo ""; log_warn "Deployment interrupted"; exit 130' INT

# Execute main function
main "$@"