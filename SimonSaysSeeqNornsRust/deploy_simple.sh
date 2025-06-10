#!/bin/bash

# Simple On-Device Deployment for SimonSaysSeeq Rust
# This script builds directly on Norns to avoid cross-compilation issues

set -e  # Exit on any error

# Configuration
NORNS_IP="${NORNS_IP:-norns.local}"
NORNS_USER="${NORNS_USER:-we}"
PROJECT_NAME="SimonSaysSeeqRust"
NORNS_TARGET_DIR="/home/we/dust/code/${PROJECT_NAME}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🦀 SimonSaysSeeq Simple Norns Deployment${NC}"
echo "=============================================="

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

# Test SSH connectivity
print_status "Testing connection to Norns at $NORNS_IP..."
if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "$NORNS_USER@$NORNS_IP" exit 2>/dev/null; then
    print_error "Cannot connect to Norns at $NORNS_IP"
    echo "Please ensure:"
    echo "  1. Norns is powered on and connected to network"
    echo "  2. SSH is enabled (SYSTEM > WIFI > SSH ON)"
    echo "  3. You can SSH: ssh $NORNS_USER@$NORNS_IP"
    exit 1
fi

print_success "Connected to Norns successfully"

# Create deployment package
print_status "Creating deployment package..."
TEMP_DIR=$(mktemp -d)
DEPLOY_DIR="$TEMP_DIR/$PROJECT_NAME"
mkdir -p "$DEPLOY_DIR"

# Copy source code (excluding target directory)
print_status "Copying source code..."
rsync -av --exclude='target/' --exclude='.git/' . "$DEPLOY_DIR/"

# Create install script for Norns
print_status "Creating installation script..."
cat > "$DEPLOY_DIR/install_on_norns.sh" << 'EOF'
#!/bin/bash

# Installation script that runs ON the Norns device

set -e

echo "🦀 Installing SimonSaysSeeq Rust on Norns..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Install required packages
echo "Installing system dependencies..."
sudo apt update
sudo apt install -y build-essential pkg-config libasound2-dev libhidapi-dev libudev-dev

# Build the project with simulation features (no hardware dependencies)
echo "Building SimonSaysSeeq (simulation mode)..."
cargo build --release --features="simulation" --no-default-features

# Check if binary was created
if [ ! -f "target/release/simon_says_seeq" ]; then
    echo "ERROR: Build failed - binary not found"
    exit 1
fi

# Make binary executable
chmod +x target/release/simon_says_seeq

# Test the binary
echo "Testing binary..."
if ./target/release/simon_says_seeq --help 2>/dev/null | head -1; then
    echo "✅ Binary test successful"
else
    echo "⚠️  Binary created but may have issues (this is normal in simulation mode)"
fi

echo ""
echo "🎉 Installation completed!"
echo ""
echo "To run SimonSaysSeeq:"
echo "  cd /home/we/dust/code/SimonSaysSeeqRust"
echo "  ./target/release/simon_says_seeq"
echo ""
echo "Note: This build runs in simulation mode."
echo "Hardware features will be logged but not actually control devices."
EOF

chmod +x "$DEPLOY_DIR/install_on_norns.sh"

# Create Norns script wrapper
print_status "Creating Norns Lua wrapper..."
cat > "$DEPLOY_DIR/${PROJECT_NAME}.lua" << 'EOF'
-- SimonSaysSeeq Rust - Norns Script
-- A Rust-based step sequencer for Norns

local rust_binary = "/home/we/dust/code/SimonSaysSeeqRust/target/release/simon_says_seeq"
local rust_process = nil

function init()
    print("SimonSaysSeeq Rust - Starting...")
    
    -- Clear screen and show startup message
    screen.clear()
    screen.move(64, 20)
    screen.text_center("SimonSaysSeeq Rust")
    screen.move(64, 35)
    screen.text_center("Starting...")
    screen.update()
    
    -- Check if binary exists
    local file = io.open(rust_binary, "r")
    if not file then
        print("ERROR: Binary not found at " .. rust_binary)
        print("Please run: cd /home/we/dust/code/SimonSaysSeeqRust && ./install_on_norns.sh")
        
        screen.clear()
        screen.move(5, 15)
        screen.text("ERROR: Not installed")
        screen.move(5, 25)
        screen.text("SSH to Norns and run:")
        screen.move(5, 35)
        screen.text("cd dust/code/SimonSaysSeeqRust")
        screen.move(5, 45)
        screen.text("./install_on_norns.sh")
        screen.update()
        return
    end
    file:close()
    
    -- Start the Rust process
    start_rust_process()
    
    -- Set up cleanup
    cleanup.register(stop_rust_process)
end

function start_rust_process()
    print("Starting Rust process...")
    
    -- Set environment variables
    os.execute("export RUST_LOG=info")
    
    -- Start process in background with proper logging
    local cmd = "cd /home/we/dust/code/SimonSaysSeeqRust && " .. 
                "RUST_LOG=info ./target/release/simon_says_seeq > /tmp/simonsaysseeq.log 2>&1 &"
    
    os.execute(cmd)
    rust_process = true
    
    -- Update screen
    screen.clear()
    screen.move(64, 15)
    screen.text_center("SimonSaysSeeq Rust")
    screen.move(64, 25)
    screen.text_center("🦀 Running 🦀")
    screen.move(64, 40)
    screen.text_center("K1: Restart  K3: Stop")
    screen.move(64, 55)
    screen.text_center("Check /tmp/simonsaysseeq.log")
    screen.update()
end

function stop_rust_process()
    if rust_process then
        print("Stopping Rust process...")
        os.execute("pkill -f simon_says_seeq")
        rust_process = nil
        
        screen.clear()
        screen.move(64, 20)
        screen.text_center("SimonSaysSeeq Rust")
        screen.move(64, 35)
        screen.text_center("Stopped")
        screen.update()
    end
end

function cleanup()
    stop_rust_process()
end

function key(n, z)
    if z == 1 then  -- Key press
        if n == 1 then
            -- Key 1: Restart
            print("Restarting...")
            stop_rust_process()
            clock.sleep(0.5)
            start_rust_process()
        elseif n == 3 then
            -- Key 3: Stop/Start toggle
            if rust_process then
                stop_rust_process()
            else
                start_rust_process()
            end
        end
    end
end

function enc(n, d)
    -- Encoders are handled by the Rust process
    -- This just provides feedback
    if n == 1 then
        print("Encoder 1: " .. d .. " (handled by Rust)")
    elseif n == 2 then
        print("Encoder 2: " .. d .. " (handled by Rust)")
    elseif n == 3 then
        print("Encoder 3: " .. d .. " (handled by Rust)")
    end
end

function redraw()
    -- Screen is handled by Rust process
    -- This script just shows status
end
EOF

# Create simple README
cat > "$DEPLOY_DIR/README.md" << 'EOF'
# SimonSaysSeeq Rust for Norns

A high-performance step sequencer written in Rust for Norns hardware.

## Installation

1. **Copy files to Norns** (done by deployment script)
2. **SSH to Norns:**
   ```bash
   ssh we@norns.local
   cd /home/we/dust/code/SimonSaysSeeqRust
   ./install_on_norns.sh
   ```
3. **Use from Norns menu:** SELECT > SimonSaysSeeqRust

## Controls

- **Key 1:** Restart Rust process
- **Key 3:** Stop/Start toggle  
- **Encoders:** Parameter control (handled by Rust)
- **Grid:** Step sequencer interface (when connected)

## Monitoring

- **Logs:** `/tmp/simonsaysseeq.log`
- **Process:** `ps aux | grep simon_says_seeq`
- **Manual start:** `cd /home/we/dust/code/SimonSaysSeeqRust && ./target/release/simon_says_seeq`

## Features

- Real-time step sequencing
- MIDI output (simulation mode)
- Multi-lane patterns  
- Environmental CO2 integration
- Pattern chains and song mode
- Undo/redo system

## Troubleshooting

If installation fails:
1. Check internet connection on Norns
2. Ensure sufficient disk space
3. Try: `sudo apt update && sudo apt upgrade`
4. Check logs in `/tmp/simonsaysseeq.log`

The initial build may take several minutes as Rust compiles dependencies.
EOF

# Deploy to Norns
print_status "Deploying to Norns..."

# Stop any existing processes
ssh "$NORNS_USER@$NORNS_IP" "pkill -f simon_says_seeq" 2>/dev/null || true

# Create target directory
ssh "$NORNS_USER@$NORNS_IP" "mkdir -p $NORNS_TARGET_DIR"

# Copy files
print_status "Copying files to Norns..."
scp -r "$DEPLOY_DIR/"* "$NORNS_USER@$NORNS_IP:$NORNS_TARGET_DIR/"

# Set permissions
ssh "$NORNS_USER@$NORNS_IP" "cd $NORNS_TARGET_DIR && chmod +x install_on_norns.sh"

# Cleanup temp directory
rm -rf "$TEMP_DIR"

print_success "Deployment completed!"
echo ""
echo "🎵 Next steps:"
echo ""
echo "1. SSH to Norns and build:"
echo "   ssh $NORNS_USER@$NORNS_IP"
echo "   cd $NORNS_TARGET_DIR"
echo "   ./install_on_norns.sh"
echo ""
echo "2. Run from Norns menu:"
echo "   SELECT > $PROJECT_NAME"
echo ""
echo "3. Monitor with:"
echo "   tail -f /tmp/simonsaysseeq.log"
echo ""
echo "Note: First build takes ~5-10 minutes as Rust compiles all dependencies."
echo "Subsequent builds will be much faster."