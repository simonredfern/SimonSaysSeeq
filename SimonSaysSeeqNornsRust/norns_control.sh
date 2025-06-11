#!/bin/bash

# Norns Remote Control Script
# Control your Norns boot mode from your local machine

set -e

# Configuration
NORNS_IP="${NORNS_IP:-norns.local}"
NORNS_USER="${NORNS_USER:-we}"
PROJECT_DIR="/home/we/dust/code/SimonSaysSeeqRust"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "========================================"
    echo "   Norns Remote Control"
    echo "========================================"
    echo -e "${NC}"
}

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test connection to Norns
test_connection() {
    print_status "Testing connection to Norns at $NORNS_IP..."
    if ssh -o ConnectTimeout=5 -o BatchMode=yes "$NORNS_USER@$NORNS_IP" exit 2>/dev/null; then
        print_success "Connected to Norns successfully"
        return 0
    else
        print_error "Cannot connect to Norns at $NORNS_IP"
        echo "Please ensure:"
        echo "  1. Norns is powered on and connected to network"
        echo "  2. SSH is enabled on Norns"
        echo "  3. IP address is correct (set NORNS_IP environment variable if needed)"
        return 1
    fi
}

# Get current status from Norns
get_status() {
    print_status "Getting current Norns status..."
    ssh "$NORNS_USER@$NORNS_IP" "cd $PROJECT_DIR && ./toggle_startup_mode.sh status" 2>/dev/null || {
        print_error "Failed to get status from Norns"
        return 1
    }
}

# Set boot mode to Rust app
set_rust_mode() {
    print_status "Setting Norns to boot directly to Rust app..."
    ssh "$NORNS_USER@$NORNS_IP" "cd $PROJECT_DIR && ./toggle_startup_mode.sh rust" || {
        print_error "Failed to set Rust mode"
        return 1
    }
    print_success "Norns set to boot to Rust app"
    print_warning "Reboot Norns to take effect"
}

# Set boot mode to normal menu
set_menu_mode() {
    print_status "Setting Norns to boot to normal menu..."
    ssh "$NORNS_USER@$NORNS_IP" "cd $PROJECT_DIR && ./toggle_startup_mode.sh menu" || {
        print_error "Failed to set menu mode"
        return 1
    }
    print_success "Norns set to boot to normal menu"
    print_warning "Reboot Norns to take effect"
}

# Reboot Norns
reboot_norns() {
    print_status "Rebooting Norns..."
    ssh "$NORNS_USER@$NORNS_IP" "sudo reboot" || {
        print_error "Failed to reboot Norns"
        return 1
    }
    print_success "Norns is rebooting..."
    print_status "Wait about 30 seconds for Norns to come back online"
}

# Start Rust service now (without rebooting)
start_rust_now() {
    print_status "Starting Rust app on Norns now..."
    ssh "$NORNS_USER@$NORNS_IP" "cd $PROJECT_DIR && sudo systemctl start simonsaysseeq-rust" || {
        print_error "Failed to start Rust service"
        return 1
    }
    print_success "Rust app started on Norns"
}

# Stop Rust service
stop_rust_now() {
    print_status "Stopping Rust app on Norns..."
    ssh "$NORNS_USER@$NORNS_IP" "cd $PROJECT_DIR && sudo systemctl stop simonsaysseeq-rust" || {
        print_error "Failed to stop Rust service"
        return 1
    }
    print_success "Rust app stopped on Norns"
}

# Show menu
show_menu() {
    echo -e "\n${BLUE}What would you like to do?${NC}"
    echo "  1) Get current status"
    echo "  2) Set to Rust app boot mode"
    echo "  3) Set to normal Norns menu mode"
    echo "  4) Reboot Norns"
    echo "  5) Start Rust app now (no reboot)"
    echo "  6) Stop Rust app now"
    echo "  7) Set Rust mode and reboot"
    echo "  8) Set menu mode and reboot"
    echo "  9) Exit"
    echo -n -e "\n${YELLOW}Enter choice [1-9]: ${NC}"
}

# Show help
show_help() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  status          Get current Norns status"
    echo "  rust            Set to Rust app boot mode"
    echo "  menu            Set to normal menu boot mode"
    echo "  reboot          Reboot Norns"
    echo "  start           Start Rust app now (no reboot)"
    echo "  stop            Stop Rust app now"
    echo "  rust-reboot     Set Rust mode and reboot"
    echo "  menu-reboot     Set menu mode and reboot"
    echo "  help            Show this help"
    echo ""
    echo "Environment variables:"
    echo "  NORNS_IP        Norns IP address (default: norns.local)"
    echo "  NORNS_USER      Norns username (default: we)"
    echo ""
    echo "Examples:"
    echo "  $0 status                    # Get current status"
    echo "  $0 rust-reboot              # Switch to Rust and reboot"
    echo "  NORNS_IP=192.168.1.100 $0 rust  # Use specific IP"
}

# Main function
main() {
    print_banner
    
    # Parse command line arguments
    case "${1:-}" in
        "status")
            test_connection && get_status
            exit 0
            ;;
        "rust")
            test_connection && set_rust_mode
            exit 0
            ;;
        "menu")
            test_connection && set_menu_mode
            exit 0
            ;;
        "reboot")
            test_connection && reboot_norns
            exit 0
            ;;
        "start")
            test_connection && start_rust_now
            exit 0
            ;;
        "stop")
            test_connection && stop_rust_now
            exit 0
            ;;
        "rust-reboot")
            if test_connection; then
                set_rust_mode && reboot_norns
            fi
            exit 0
            ;;
        "menu-reboot")
            if test_connection; then
                set_menu_mode && reboot_norns
            fi
            exit 0
            ;;
        "help"|"-h"|"--help")
            show_help
            exit 0
            ;;
        "")
            # Interactive mode
            ;;
        *)
            print_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
    
    # Interactive mode
    if ! test_connection; then
        exit 1
    fi
    
    while true; do
        show_menu
        read -r choice
        
        case $choice in
            1)
                get_status
                ;;
            2)
                set_rust_mode
                ;;
            3)
                set_menu_mode
                ;;
            4)
                reboot_norns
                ;;
            5)
                start_rust_now
                ;;
            6)
                stop_rust_now
                ;;
            7)
                set_rust_mode && reboot_norns
                ;;
            8)
                set_menu_mode && reboot_norns
                ;;
            9)
                print_success "Goodbye!"
                exit 0
                ;;
            *)
                print_error "Invalid option. Please choose 1-9."
                ;;
        esac
        
        echo -e "\n${BLUE}Press Enter to continue...${NC}"
        read -r
    done
}

# Make sure script is executable
if [ ! -x "$0" ]; then
    chmod +x "$0"
fi

# Run main function
main "$@"