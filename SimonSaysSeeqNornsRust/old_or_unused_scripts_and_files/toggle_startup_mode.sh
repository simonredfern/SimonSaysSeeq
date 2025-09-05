#!/bin/bash

# SimonSaysSeeq Startup Mode Toggle
# This script allows you to choose between:
# 1. Normal Norns menu startup (default)
# 2. Direct boot to SimonSaysSeeqRust application

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE_NAME="simonsaysseeq-rust"
CONFIG_FILE="/home/we/.config/simonsaysseeq/startup_mode"
CONFIG_DIR="$(dirname "$CONFIG_FILE")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "=================================="
    echo "  SimonSaysSeeq Startup Control"
    echo "=================================="
    echo -e "${NC}"
}

print_status() {
    echo -e "\n${YELLOW}Current Status:${NC}"
    
    # Check systemd service status
    if systemctl is-enabled "$SERVICE_NAME" >/dev/null 2>&1; then
        if systemctl is-enabled "$SERVICE_NAME" | grep -q "enabled"; then
            echo -e "  Startup Mode: ${GREEN}Direct Rust App Boot${NC}"
            echo -e "  Service: ${GREEN}Enabled${NC}"
        else
            echo -e "  Startup Mode: ${YELLOW}Normal Norns Menu${NC}"
            echo -e "  Service: ${YELLOW}Disabled${NC}"
        fi
    else
        echo -e "  Startup Mode: ${YELLOW}Normal Norns Menu${NC}"
        echo -e "  Service: ${RED}Not Available${NC}"
    fi
    
    # Check if service is currently running
    if systemctl is-active "$SERVICE_NAME" >/dev/null 2>&1; then
        if systemctl is-active "$SERVICE_NAME" | grep -q "active"; then
            echo -e "  Current State: ${GREEN}Running${NC}"
        else
            echo -e "  Current State: ${RED}Stopped${NC}"
        fi
    else
        echo -e "  Current State: ${YELLOW}Not Available${NC}"
    fi
    
    # Check config file
    if [ -f "$CONFIG_FILE" ]; then
        mode=$(cat "$CONFIG_FILE" 2>/dev/null || echo "unknown")
        echo -e "  Config File: ${GREEN}$mode${NC}"
    else
        echo -e "  Config File: ${YELLOW}Not Set${NC}"
    fi
}

create_config_dir() {
    if [ ! -d "$CONFIG_DIR" ]; then
        echo -e "${YELLOW}Creating config directory...${NC}"
        mkdir -p "$CONFIG_DIR"
    fi
}

set_menu_mode() {
    echo -e "\n${YELLOW}Setting startup mode to: Normal Norns Menu${NC}"
    
    # Disable systemd service
    if systemctl is-enabled "$SERVICE_NAME" >/dev/null 2>&1; then
        echo "Disabling systemd service..."
        sudo systemctl disable "$SERVICE_NAME"
        sudo systemctl stop "$SERVICE_NAME"
    fi
    
    # Update config file
    create_config_dir
    echo "menu" > "$CONFIG_FILE"
    
    echo -e "${GREEN}✓ Startup mode set to Normal Norns Menu${NC}"
    echo -e "${BLUE}  Next boot: Norns will show the standard menu${NC}"
    echo -e "${BLUE}  You can select SimonSaysSeeqNorns.lua from SELECT menu${NC}"
}

set_rust_mode() {
    echo -e "\n${YELLOW}Setting startup mode to: Direct Rust App Boot${NC}"
    
    # Check if service file exists
    if [ ! -f "/etc/systemd/system/$SERVICE_NAME.service" ]; then
        echo -e "${RED}Error: systemd service not found!${NC}"
        echo -e "${YELLOW}Please run ./deploy_to_norns.sh first to install the service${NC}"
        return 1
    fi
    
    # Enable systemd service
    echo "Enabling systemd service..."
    sudo systemctl daemon-reload
    sudo systemctl enable "$SERVICE_NAME"
    
    # Update config file
    create_config_dir
    echo "rust" > "$CONFIG_FILE"
    
    echo -e "${GREEN}✓ Startup mode set to Direct Rust App Boot${NC}"
    echo -e "${BLUE}  Next boot: SimonSaysSeeqRust will start automatically${NC}"
    echo -e "${BLUE}  The Rust application will bypass the Norns menu${NC}"
}

start_service() {
    echo -e "\n${YELLOW}Starting SimonSaysSeeqRust service now...${NC}"
    if sudo systemctl start "$SERVICE_NAME"; then
        echo -e "${GREEN}✓ Service started successfully${NC}"
    else
        echo -e "${RED}✗ Failed to start service${NC}"
        echo "Check logs with: journalctl -u $SERVICE_NAME -f"
    fi
}

stop_service() {
    echo -e "\n${YELLOW}Stopping SimonSaysSeeqRust service...${NC}"
    if sudo systemctl stop "$SERVICE_NAME"; then
        echo -e "${GREEN}✓ Service stopped successfully${NC}"
    else
        echo -e "${RED}✗ Failed to stop service${NC}"
    fi
}

show_logs() {
    echo -e "\n${YELLOW}Showing recent logs for $SERVICE_NAME:${NC}"
    echo -e "${BLUE}Press Ctrl+C to exit log view${NC}\n"
    journalctl -u "$SERVICE_NAME" -f --since "10 minutes ago"
}

show_menu() {
    echo -e "\n${BLUE}Choose an option:${NC}"
    echo "  1) Set Normal Norns Menu startup (default)"
    echo "  2) Set Direct Rust App Boot startup"
    echo "  3) Start Rust service now (without changing boot mode)"
    echo "  4) Stop Rust service now"
    echo "  5) View service logs"
    echo "  6) Show current status"
    echo "  7) Exit"
    echo -n -e "\n${YELLOW}Enter choice [1-7]: ${NC}"
}

main() {
    print_banner
    
    # If arguments provided, handle them
    case "$1" in
        "menu"|"lua")
            set_menu_mode
            exit 0
            ;;
        "rust"|"direct")
            set_rust_mode
            exit 0
            ;;
        "status")
            print_status
            exit 0
            ;;
        "start")
            start_service
            exit 0
            ;;
        "stop")
            stop_service
            exit 0
            ;;
        "logs")
            show_logs
            exit 0
            ;;
        "help"|"-h"|"--help")
            echo "Usage: $0 [option]"
            echo ""
            echo "Options:"
            echo "  menu     Set Normal Norns Menu startup"
            echo "  rust     Set Direct Rust App Boot startup"
            echo "  status   Show current status"
            echo "  start    Start Rust service now"
            echo "  stop     Stop Rust service now"
            echo "  logs     View service logs"
            echo "  help     Show this help"
            echo ""
            echo "Interactive mode: Run without arguments"
            exit 0
            ;;
    esac
    
    # Interactive mode
    print_status
    
    while true; do
        show_menu
        read -r choice
        
        case $choice in
            1)
                set_menu_mode
                ;;
            2)
                set_rust_mode
                ;;
            3)
                start_service
                ;;
            4)
                stop_service
                ;;
            5)
                show_logs
                ;;
            6)
                print_status
                ;;
            7)
                echo -e "\n${GREEN}Goodbye!${NC}"
                exit 0
                ;;
            *)
                echo -e "\n${RED}Invalid option. Please choose 1-7.${NC}"
                ;;
        esac
        
        echo -e "\n${BLUE}Press Enter to continue...${NC}"
        read -r
    done
}

# Ensure script is executable
if [ ! -x "$0" ]; then
    echo -e "${YELLOW}Making script executable...${NC}"
    chmod +x "$0"
fi

main "$@"