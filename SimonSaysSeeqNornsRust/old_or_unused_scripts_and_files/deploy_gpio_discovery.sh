#!/bin/bash

# Deploy GPIO Discovery Tool to Norns
# This script copies the GPIO discovery tool to your Norns device

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
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "=============================================="
    echo "  Deploy GPIO Discovery Tool to Norns"
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
    log_info "Checking requirements..."
    
    # Check if discovery script exists
    if [ ! -f "$LOCAL_SCRIPT_DIR/discover_gpio_pins.sh" ]; then
        log_error "discover_gpio_pins.sh not found in $LOCAL_SCRIPT_DIR"
        exit 1
    fi
    
    # Check if script is executable
    if [ ! -x "$LOCAL_SCRIPT_DIR/discover_gpio_pins.sh" ]; then
        log_warn "Making discover_gpio_pins.sh executable..."
        chmod +x "$LOCAL_SCRIPT_DIR/discover_gpio_pins.sh"
    fi
    
    # Check SSH connectivity
    if ! command -v ssh >/dev/null 2>&1; then
        log_error "SSH client not found"
        exit 1
    fi
    
    # Check SCP
    if ! command -v scp >/dev/null 2>&1; then
        log_error "SCP not found"
        exit 1
    fi
    
    log_info "✓ Requirements check passed"
}

test_connection() {
    log_info "Testing connection to Norns at $NORNS_IP..."
    
    if ! ssh -o ConnectTimeout=10 -o BatchMode=yes "$NORNS_USER@$NORNS_IP" "echo 'Connection test successful'" >/dev/null 2>&1; then
        log_error "Cannot connect to Norns at $NORNS_IP"
        log_error "Please check:"
        log_error "  1. Norns is powered on and connected to network"
        log_error "  2. SSH is enabled on Norns (SYSTEM > WIFI > ADD/EDIT > SSH ON)"
        log_error "  3. IP address is correct: $NORNS_IP"
        log_error "  4. You can SSH manually: ssh $NORNS_USER@$NORNS_IP"
        echo ""
        echo "To specify a different IP address:"
        echo "  NORNS_IP=192.168.1.100 $0"
        exit 1
    fi
    
    log_info "✓ Connection to Norns successful"
}

create_remote_directory() {
    log_info "Creating remote directory structure..."
    
    ssh "$NORNS_USER@$NORNS_IP" "mkdir -p $REMOTE_DIR" || {
        log_error "Failed to create remote directory $REMOTE_DIR"
        exit 1
    }
    
    log_info "✓ Remote directory ready"
}

deploy_files() {
    log_info "Deploying GPIO discovery tool to Norns..."
    
    # Copy main discovery script
    if scp "$LOCAL_SCRIPT_DIR/discover_gpio_pins.sh" "$NORNS_USER@$NORNS_IP:$REMOTE_DIR/"; then
        log_info "✓ discover_gpio_pins.sh deployed"
    else
        log_error "Failed to deploy discover_gpio_pins.sh"
        exit 1
    fi
    
    # Copy documentation if it exists
    if [ -f "$LOCAL_SCRIPT_DIR/GPIO_DISCOVERY.md" ]; then
        if scp "$LOCAL_SCRIPT_DIR/GPIO_DISCOVERY.md" "$NORNS_USER@$NORNS_IP:$REMOTE_DIR/"; then
            log_info "✓ GPIO_DISCOVERY.md deployed"
        else
            log_warn "Failed to deploy GPIO_DISCOVERY.md (non-critical)"
        fi
    fi
    
    # Ensure script is executable on remote
    ssh "$NORNS_USER@$NORNS_IP" "chmod +x $REMOTE_DIR/discover_gpio_pins.sh" || {
        log_error "Failed to make script executable on remote"
        exit 1
    }
    
    log_info "✓ All files deployed successfully"
}

show_usage_instructions() {
    echo ""
    echo -e "${GREEN}=============================================="
    echo "  Deployment Completed Successfully!"
    echo -e "===============================================${NC}"
    echo ""
    echo -e "${BLUE}GPIO Discovery Tool is now installed on your Norns.${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo ""
    echo "1. SSH to your Norns:"
    echo "   ssh $NORNS_USER@$NORNS_IP"
    echo ""
    echo "2. Navigate to the directory:"
    echo "   cd $REMOTE_DIR"
    echo ""
    echo "3. Run the GPIO discovery tool:"
    echo "   sudo ./discover_gpio_pins.sh quick"
    echo ""
    echo -e "${BLUE}Discovery options:${NC}"
    echo "   sudo ./discover_gpio_pins.sh quick      # Complete discovery test"
    echo "   sudo ./discover_gpio_pins.sh discover   # Find available GPIO pins"
    echo "   sudo ./discover_gpio_pins.sh test       # Interactive button testing"
    echo "   sudo ./discover_gpio_pins.sh monitor    # Live GPIO monitoring"
    echo ""
    echo -e "${BLUE}The discovery will:${NC}"
    echo "   • Find available GPIO pins on your Norns"
    echo "   • Test which pins respond to K2 and K3 buttons"
    echo "   • Generate correct configuration for hardware_boot_selector.sh"
    echo "   • Create gpio_config.sh with the correct pin numbers"
    echo ""
    echo -e "${YELLOW}After discovery:${NC}"
    echo "   1. Note the GPIO pin numbers it finds"
    echo "   2. Update hardware_boot_selector.sh with correct pins"
    echo "   3. Test the boot selector"
    echo "   4. Deploy with install_boot_selector.sh"
    echo ""
    echo -e "${RED}Important:${NC} Always run discovery before deploying the boot selector!"
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
    echo ""
    echo "Examples:"
    echo "  $0                                    # Deploy to norns.local"
    echo "  NORNS_IP=192.168.1.100 $0           # Deploy to specific IP"
    echo "  NORNS_USER=myuser NORNS_IP=myip $0   # Custom user and IP"
    echo ""
    echo "This script deploys the GPIO discovery tool to your Norns device"
    echo "so you can determine the correct GPIO pin mappings for the"
    echo "hardware boot selector."
}

main() {
    # Handle command line arguments
    case "${1:-}" in
        "--help"|"-h"|"help")
            show_help
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
    check_requirements
    test_connection
    create_remote_directory
    deploy_files
    
    show_usage_instructions
}

# Handle Ctrl+C gracefully
trap 'echo ""; log_warn "Deployment interrupted"; exit 130' INT

# Execute main function
main "$@"