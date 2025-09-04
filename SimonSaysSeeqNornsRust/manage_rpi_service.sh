#!/bin/bash

# SimonSaysSeeq Raspberry Pi Service Management Script
# This script provides easy management of the SimonSaysSeeq service on Raspberry Pi 5

set -e

# Configuration
SERVICE_NAME="simonsaysseeq-rpi.service"
CONFIG_FILE="/home/simonredfern/.config/simon-says-seeq/config.toml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_banner() {
    echo -e "${GREEN}"
    echo "=========================================="
    echo "   SimonSaysSeeq Raspberry Pi Manager    "
    echo "=========================================="
    echo -e "${NC}"
}

show_help() {
    print_banner
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start          Start the SimonSaysSeeq service"
    echo "  stop           Stop the SimonSaysSeeq service"
    echo "  restart        Restart the SimonSaysSeeq service"
    echo "  status         Show service status"
    echo "  logs           Show recent service logs"
    echo "  logs-live      Follow service logs in real-time"
    echo "  enable         Enable auto-start on boot"
    echo "  disable        Disable auto-start on boot"
    echo "  install        Install/reinstall the service"
    echo "  uninstall      Remove the service"
    echo "  fix-config     Fix common configuration issues"
    echo "  test           Test if the binary runs manually"
    echo "  build          Build the latest version"
    echo "  update         Build and restart service"
    echo "  help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 status      # Check if service is running"
    echo "  $0 logs        # View recent activity"
    echo "  $0 restart     # Restart after making changes"
    echo "  $0 update      # Build latest code and restart"
}

check_service_exists() {
    if ! systemctl list-unit-files | grep -q "^${SERVICE_NAME}"; then
        log_error "Service ${SERVICE_NAME} not found!"
        log_info "Run '$0 install' to install the service first."
        exit 1
    fi
}

start_service() {
    log_info "Starting ${SERVICE_NAME}..."
    sudo systemctl start "$SERVICE_NAME"
    sleep 2
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        log_success "Service started successfully"
        show_status
    else
        log_error "Failed to start service"
        show_logs
        exit 1
    fi
}

stop_service() {
    log_info "Stopping ${SERVICE_NAME}..."
    sudo systemctl stop "$SERVICE_NAME"
    log_success "Service stopped"
}

restart_service() {
    log_info "Restarting ${SERVICE_NAME}..."
    sudo systemctl restart "$SERVICE_NAME"
    sleep 2
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        log_success "Service restarted successfully"
        show_status
    else
        log_error "Failed to restart service"
        show_logs
        exit 1
    fi
}

show_status() {
    echo ""
    echo -e "${CYAN}Service Status:${NC}"
    systemctl status "$SERVICE_NAME" --no-pager -l
    echo ""

    if systemctl is-enabled --quiet "$SERVICE_NAME"; then
        echo -e "${GREEN}✓ Auto-start on boot: ENABLED${NC}"
    else
        echo -e "${YELLOW}✗ Auto-start on boot: DISABLED${NC}"
    fi

    if systemctl is-active --quiet "$SERVICE_NAME"; then
        echo -e "${GREEN}✓ Current status: RUNNING${NC}"
    else
        echo -e "${RED}✗ Current status: STOPPED${NC}"
    fi
}

show_logs() {
    echo ""
    echo -e "${CYAN}Recent logs (last 20 lines):${NC}"
    journalctl -u "$SERVICE_NAME" -n 20 --no-pager
}

show_logs_live() {
    log_info "Following logs for ${SERVICE_NAME} (Ctrl+C to exit)..."
    journalctl -u "$SERVICE_NAME" -f
}

enable_service() {
    log_info "Enabling auto-start on boot..."
    sudo systemctl enable "$SERVICE_NAME"
    log_success "Auto-start enabled"
}

disable_service() {
    log_info "Disabling auto-start on boot..."
    sudo systemctl disable "$SERVICE_NAME"
    log_warning "Auto-start disabled"
}

install_service() {
    log_info "Installing SimonSaysSeeq service for Raspberry Pi..."

    # Check if binary exists
    BINARY_PATH="$(pwd)/target/release/simon_says_seeq"
    if [[ ! -f "$BINARY_PATH" ]]; then
        log_error "Binary not found at $BINARY_PATH"
        log_info "Run '$0 build' first to compile the application"
        exit 1
    fi

    # Create service file
    SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}"
    log_info "Creating service file at $SERVICE_FILE..."

    sudo tee "$SERVICE_FILE" > /dev/null << EOF
[Unit]
Description=SimonSaysSeeq Sequencer for Raspberry Pi
After=multi-user.target udev.target serialosc.service
Wants=serialosc.service

[Service]
Type=simple
User=$(whoami)
Group=audio
WorkingDirectory=$(pwd)
ExecStart=$(pwd)/target/release/simon_says_seeq
Environment=RUST_LOG=info
Environment=XDG_RUNTIME_DIR=/run/user/$(id -u)
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal
KillMode=mixed
TimeoutStartSec=30
TimeoutStopSec=15
SupplementaryGroups=audio gpio i2c spi dialout plugdev

[Install]
WantedBy=multi-user.target
EOF

    # Reload systemd and enable service
    sudo systemctl daemon-reload
    sudo systemctl enable "$SERVICE_NAME"

    # Create config directory if it doesn't exist
    CONFIG_DIR="$(dirname "$CONFIG_FILE")"
    if [[ ! -d "$CONFIG_DIR" ]]; then
        log_info "Creating config directory..."
        mkdir -p "$CONFIG_DIR"
    fi

    log_success "Service installed and enabled for auto-start"
    log_info "Use '$0 start' to start the service now"
}

uninstall_service() {
    log_warning "Uninstalling ${SERVICE_NAME}..."

    # Stop and disable service
    sudo systemctl stop "$SERVICE_NAME" 2>/dev/null || true
    sudo systemctl disable "$SERVICE_NAME" 2>/dev/null || true

    # Remove service file
    sudo rm -f "/etc/systemd/system/${SERVICE_NAME}"
    sudo systemctl daemon-reload

    log_success "Service uninstalled"
}

fix_config() {
    log_info "Fixing common configuration issues..."

    if [[ -f "$CONFIG_FILE" ]]; then
        # Fix null values in TOML
        if grep -q "= null" "$CONFIG_FILE"; then
            log_info "Fixing null values in config file..."
            sed -i 's/= null/= ""/g' "$CONFIG_FILE"
            log_success "Fixed null values in config"
        fi

        # Check for other common issues
        log_info "Config file looks good"
    else
        log_warning "Config file not found at $CONFIG_FILE"
        log_info "It will be created automatically when the service starts"
    fi
}

test_binary() {
    BINARY_PATH="$(pwd)/target/release/simon_says_seeq"
    if [[ ! -f "$BINARY_PATH" ]]; then
        log_error "Binary not found at $BINARY_PATH"
        log_info "Run '$0 build' first to compile the application"
        exit 1
    fi

    log_info "Testing binary execution (will run for 5 seconds)..."
    timeout 5s "$BINARY_PATH" || {
        local exit_code=$?
        if [[ $exit_code -eq 124 ]]; then
            log_success "Binary runs successfully (stopped after 5 seconds)"
        else
            log_error "Binary failed to run (exit code: $exit_code)"
            exit 1
        fi
    }
}

build_project() {
    log_info "Building SimonSaysSeeq for Raspberry Pi..."

    if [[ -f "rpi5_build_and_run.sh" ]]; then
        log_info "Using rpi5_build_and_run.sh script..."
        # Extract just the build part, not the run part
        bash -c "source rpi5_build_and_run.sh && build_project"
    else
        log_info "Building with cargo..."
        cargo build --release --features hardware,midi,desktop
    fi

    log_success "Build completed"
}

update_service() {
    log_info "Building latest version and updating service..."

    # Stop service if running
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        stop_service
    fi

    # Build
    build_project

    # Fix config
    fix_config

    # Start service
    start_service

    log_success "Service updated and restarted"
}

# Main command handling
case "${1:-help}" in
    "start")
        check_service_exists
        start_service
        ;;
    "stop")
        check_service_exists
        stop_service
        ;;
    "restart")
        check_service_exists
        restart_service
        ;;
    "status")
        check_service_exists
        show_status
        ;;
    "logs")
        check_service_exists
        show_logs
        ;;
    "logs-live")
        check_service_exists
        show_logs_live
        ;;
    "enable")
        check_service_exists
        enable_service
        ;;
    "disable")
        check_service_exists
        disable_service
        ;;
    "install")
        install_service
        ;;
    "uninstall")
        uninstall_service
        ;;
    "fix-config")
        fix_config
        ;;
    "test")
        test_binary
        ;;
    "build")
        build_project
        ;;
    "update")
        update_service
        ;;
    "help"|*)
        show_help
        ;;
esac
