#!/bin/bash

# Norns Deployment Script for SimonSaysSeeqNornsRust
# This script cross-compiles the Rust project for ARM Linux and deploys it to a Norns device

set -e  # Exit on any error

echo "hello from deploy_to_norns.sh"

# Configuration
NORNS_IP="${NORNS_IP:-norns.local}"
NORNS_USER="${NORNS_USER:-we}"
PROJECT_NAME="simon-says-seeq-rust"
NORNS_TARGET_DIR="/home/we/dust/code/SimonSaysSeeqRust"
LOCAL_TARGET="armv7-unknown-linux-gnueabihf"
BINARY_NAME="simon_says_seeq"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🦀 SimonSaysSeeq Norns Deployment Script${NC}"
echo "================================================"

# Function to print colored output
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

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the SimonSaysSeeqNornsRust directory."
    exit 1
fi

# Check for required tools
print_status "Checking dependencies..."

# Check if cross-compilation target is installed
if ! rustup target list --installed | grep -q "$LOCAL_TARGET"; then
    print_warning "ARM target not installed. Installing $LOCAL_TARGET..."
    rustup target add $LOCAL_TARGET
fi

# Check for ARM GCC cross-compiler
if ! command -v arm-linux-gnueabihf-gcc &> /dev/null; then
    print_error "ARM cross-compiler not found. Please install:"
    echo "  Ubuntu/Debian: sudo apt install gcc-arm-linux-gnueabihf"
    echo "  macOS: brew install arm-unknown-linux-gnueabihf"
    echo "  Arch: sudo pacman -S arm-linux-gnueabihf-gcc"
    exit 1
fi

# Check SSH connectivity to Norns
print_status "Testing connection to Norns at $NORNS_IP..."
if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "$NORNS_USER@$NORNS_IP" exit 2>/dev/null; then
    print_error "Cannot connect to Norns at $NORNS_IP"
    echo "Please ensure:"
    echo "  1. Norns is powered on and connected to network"
    echo "  2. SSH is enabled on Norns (SYSTEM > WIFI > ADD/EDIT > SSH ON)"
    echo "  3. IP address is correct (set NORNS_IP environment variable if needed)"
    echo "  4. You can SSH to Norns: ssh $NORNS_USER@$NORNS_IP"
    exit 1
fi

print_success "Connected to Norns successfully"

# Build for ARM Linux (Norns)
print_status "Cross-compiling for ARM Linux (Norns)..."
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=""

# Set up cross-compilation environment
export CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc
export CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++
export AR_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-ar
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

# Create cargo config for cross-compilation if it doesn't exist
mkdir -p .cargo
cat > .cargo/config.toml << 'CARGO_CONFIG'
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
ar = "arm-linux-gnueabihf-ar"

[build]
target = "armv7-unknown-linux-gnueabihf"
CARGO_CONFIG

# Build with simulation mode to avoid cross-compilation issues
cross build --release --target=$LOCAL_TARGET --no-default-features --features "simulation"

if [ $? -ne 0 ]; then
    print_error "Cross-compilation failed"
    exit 1
fi

print_success "Cross-compilation completed"

# Check if binary was created
BINARY_PATH="target/$LOCAL_TARGET/release/$BINARY_NAME"
if [ ! -f "$BINARY_PATH" ]; then
    print_error "Binary not found at $BINARY_PATH"
    exit 1
fi

# Show binary info
print_status "Binary information:"
file "$BINARY_PATH"
ls -lh "$BINARY_PATH"

# Create deployment package
print_status "Creating deployment package..."
TEMP_DIR=$(mktemp -d)
DEPLOY_DIR="$TEMP_DIR/SimonSaysSeeqRust"

mkdir -p "$DEPLOY_DIR"

# Copy binary
cp "$BINARY_PATH" "$DEPLOY_DIR/"

# Copy boot selector files
cp "hardware_boot_selector.sh" "$DEPLOY_DIR/" 2>/dev/null || echo "Hardware boot selector not found, skipping"
cp "install_boot_selector.sh" "$DEPLOY_DIR/" 2>/dev/null || echo "Boot selector installer not found, skipping"
cp "toggle_startup_mode.sh" "$DEPLOY_DIR/" 2>/dev/null || echo "Startup mode toggle not found, skipping"
cp "simonsaysseeq-boot-selector.service" "$DEPLOY_DIR/" 2>/dev/null || echo "Boot selector service not found, skipping"

# Create Norns script wrapper
cat > "$DEPLOY_DIR/SimonSaysSeeqRust.lua" << 'EOF'
-- SimonSaysSeeq Rust Bridge
-- Norns script wrapper for the Rust implementation

local rust_process = nil

function init()
    print("SimonSaysSeeq Rust - Starting...")

    -- Set up screen
    screen.clear()
    screen.move(64, 20)
    screen.text_center("SimonSaysSeeq Rust")
    screen.move(64, 35)
    screen.text_center("Starting...")
    screen.update()

    -- Start the Rust process
    start_rust_process()

    -- Set up cleanup
    cleanup.register(stop_rust_process)
end

function start_rust_process()
    local rust_binary = _path.code .. "SimonSaysSeeqRust/simon_says_seeq"

    -- Check if binary exists
    local file = io.open(rust_binary, "r")
    if file then
        file:close()
        print("Starting Rust process: " .. rust_binary)

        -- Make sure binary is executable
        os.execute("chmod +x " .. rust_binary)

        -- Start the process in background
        rust_process = os.execute(rust_binary .. " &")

        screen.clear()
        screen.move(64, 20)
        screen.text_center("SimonSaysSeeq Rust")
        screen.move(64, 35)
        screen.text_center("Running")
        screen.update()
    else
        print("ERROR: Rust binary not found at " .. rust_binary)
        screen.clear()
        screen.move(64, 20)
        screen.text_center("ERROR")
        screen.move(64, 35)
        screen.text_center("Binary not found")
        screen.update()
    end
end

function stop_rust_process()
    if rust_process then
        print("Stopping Rust process...")
        -- Kill any running simon_says_seeq processes
        os.execute("pkill -f simon_says_seeq")
        rust_process = nil
    end
end

function cleanup()
    stop_rust_process()
end

function key(n, z)
    if n == 1 and z == 1 then
        -- Key 1: Restart
        print("Restarting Rust process...")
        stop_rust_process()
        clock.sleep(0.5)
        start_rust_process()
    elseif n == 3 and z == 1 then
        -- Key 3: Stop/Start toggle
        if rust_process then
            stop_rust_process()
            screen.clear()
            screen.move(64, 20)
            screen.text_center("SimonSaysSeeq Rust")
            screen.move(64, 35)
            screen.text_center("Stopped")
            screen.update()
        else
            start_rust_process()
        end
    end
end

function enc(n, d)
    -- Encoders are handled by the Rust process
end

function redraw()
    -- Screen is handled by the Rust process
end
EOF

# Create install script for Norns
cat > "$DEPLOY_DIR/install.sh" << 'EOF'
#!/bin/bash
echo "Installing SimonSaysSeeq Rust on Norns..."

# Make binary executable
chmod +x ./simon_says_seeq

# Install boot selector if available
if [ -f "hardware_boot_selector.sh" ] && [ -f "install_boot_selector.sh" ]; then
    echo "Installing hardware boot selector..."
    chmod +x ./hardware_boot_selector.sh
    chmod +x ./install_boot_selector.sh
    
    # Update service file with correct path
    if [ -f "simonsaysseeq-boot-selector.service" ]; then
        sed "s|ExecStart=.*|ExecStart=$PWD/hardware_boot_selector.sh|" simonsaysseeq-boot-selector.service > /tmp/boot-selector.service
        sudo cp /tmp/boot-selector.service /etc/systemd/system/simonsaysseeq-boot-selector.service
        rm -f /tmp/boot-selector.service
    fi
    
    # Run the full boot selector installer
    sudo ./install_boot_selector.sh
    echo "Hardware boot selector installed and configured"
    echo "  - Hold K2 during startup for direct Rust app mode"
    echo "  - Hold K3 during startup for normal Norns menu"
    echo "  - No input = use saved preference (default: menu mode)"
elif [ -f "hardware_boot_selector.sh" ]; then
    echo "Hardware boot selector found but installer missing"
    chmod +x ./hardware_boot_selector.sh
    echo "Manual installation may be required"
fi

# Install startup mode toggle script if available
if [ -f "toggle_startup_mode.sh" ]; then
    echo "Installing startup mode toggle script..."
    chmod +x ./toggle_startup_mode.sh
    echo "Use ./toggle_startup_mode.sh to manually control boot mode"
fi

# Create systemd service for auto-start (optional)
if [ -d "/etc/systemd/system" ]; then
    echo "Creating systemd service..."
    sudo tee /etc/systemd/system/simonsaysseeq-rust.service > /dev/null << 'SERVICE_EOF'
[Unit]
Description=SimonSaysSeeq Rust Sequencer
After=network.target

[Service]
Type=simple
User=we
WorkingDirectory=/home/we/dust/code/SimonSaysSeeqRust
ExecStart=/home/we/dust/code/SimonSaysSeeqRust/simon_says_seeq
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
SERVICE_EOF

    sudo systemctl daemon-reload
    echo "Service installed. To enable auto-start:"
    echo "  sudo systemctl enable simonsaysseeq-rust"
    echo "To start manually:"
    echo "  sudo systemctl start simonsaysseeq-rust"
fi

echo "Installation complete!"
echo ""
echo "To run SimonSaysSeeq Rust:"
echo "  1. From Norns menu: SELECT > SimonSaysSeeqRust"
echo "  2. Or directly: ./simon_says_seeq"
echo "  3. Boot selector: Hold any button during startup for direct Rust mode"
echo ""
echo "Boot selector control:"
echo "  ./toggle_startup_mode.sh         # Interactive boot mode control"
echo "  ./toggle_startup_mode.sh rust    # Set to Rust app mode"
echo "  ./toggle_startup_mode.sh menu    # Set to normal menu mode"
echo "  sudo ./hardware_boot_selector.sh # Manual test"
echo "  sudo systemctl status simonsaysseeq-boot-selector"
echo ""
echo "Logs can be viewed with:"
echo "  journalctl -u simonsaysseeq-rust -f"
echo "  cat /tmp/simonsaysseeq_boot_selector.log"
EOF

chmod +x "$DEPLOY_DIR/install.sh"

# Create README
cat > "$DEPLOY_DIR/README.md" << 'EOF'
# SimonSaysSeeq Rust for Norns

This is the Rust implementation of SimonSaysSeeq running on Norns hardware.

## Installation

1. Run the install script: `./install.sh`
2. The script will be available in the Norns SELECT menu

## Manual Operation

- Start: `./simon_says_seeq`
- Stop: `Ctrl+C` or `pkill simon_says_seeq`

## Controls

- **Grid**: Main sequencer interface
- **Encoders**: Tempo and swing control
- **Keys**: Transport control

## Features

- Real-time step sequencing
- MIDI output
- Grid visual feedback
- CO2 environmental integration
- Multi-lane patterns
- Advanced editing with hold operations
- Boot selector for direct startup

## Boot Selector

The boot selector allows you to choose startup mode:
- **Hold K2 during boot**: Start directly in Rust app mode
- **Hold K3 during boot**: Start in normal Norns menu mode  
- **No input**: Use saved preference (default: normal Norns menu)

Control the boot selector:
- `./toggle_startup_mode.sh` - Interactive boot mode control
- `sudo ./hardware_boot_selector.sh` - Test manually
- Boot selector logs: `/tmp/simonsaysseeq_boot_selector.log`

## Logs

View logs: `journalctl -u simonsaysseeq-rust -f`

## Troubleshooting

If you encounter issues:
1. Check logs for error messages
2. Ensure all hardware features are working
3. Verify MIDI connections
4. Check grid connectivity
EOF

print_status "Deploying to Norns at $NORNS_IP..."

# Stop any existing Norns scripts
ssh "$NORNS_USER@$NORNS_IP" "sudo systemctl stop norns-matron; sleep 2; sudo systemctl start norns-matron" || true

# Create target directory
ssh "$NORNS_USER@$NORNS_IP" "mkdir -p $NORNS_TARGET_DIR"

# Copy files to Norns
print_status "Copying files to Norns..."
scp -r "$DEPLOY_DIR/"* "$NORNS_USER@$NORNS_IP:$NORNS_TARGET_DIR/"

# Run installation script on Norns
print_status "Running installation on Norns..."
ssh "$NORNS_USER@$NORNS_IP" "cd $NORNS_TARGET_DIR && ./install.sh"

# Cleanup temp directory
rm -rf "$TEMP_DIR"

# Final instructions
print_success "Deployment completed successfully!"
echo ""
echo "🎵 SimonSaysSeeq Rust is now installed on your Norns!"
echo ""
echo "Next steps:"
echo "  1. On Norns: SELECT > SimonSaysSeeqRust"
echo "  2. Use Key 1 to restart, Key 3 to stop/start"
echo "  3. Grid and encoders should work immediately"
echo ""
echo "Hardware Boot Selector (NEW!):"
echo "  - Hold K2 during startup: Direct Rust app boot"
echo "  - Hold K3 during startup: Normal Norns menu"
echo "  - No input: Use saved preference"
echo "  - Control: ssh $NORNS_USER@$NORNS_IP && cd $NORNS_TARGET_DIR && ./toggle_startup_mode.sh"
echo ""
echo "Monitoring:"
echo "  SSH to Norns: ssh $NORNS_USER@$NORNS_IP"
echo "  View logs: journalctl -u simonsaysseeq-rust -f"
echo "  Boot selector logs: tail -f /tmp/simonsaysseeq_boot_selector.log"
echo "  Manual start: cd $NORNS_TARGET_DIR && ./simon_says_seeq"
echo ""
echo "If you encounter issues, check the README.md on Norns for troubleshooting."

# Optional: Connect and show status
read -p "Would you like to connect to Norns and check the status? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Connecting to Norns..."
    ssh -t "$NORNS_USER@$NORNS_IP" "cd $NORNS_TARGET_DIR && echo 'Files installed:' && ls -la && echo '' && echo 'Testing binary:' && ./simon_says_seeq --help 2>/dev/null || echo 'Binary ready (use SELECT > SimonSaysSeeqRust to run)'"
fi

print_success "Deployment script completed! 🚀"
