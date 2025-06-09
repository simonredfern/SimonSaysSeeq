#!/bin/bash

# Build script for SimonSaysSeeq Rust core for Norns
# This script cross-compiles the Rust library for ARM architecture used by Norns

set -e  # Exit on any error

echo "Building SimonSaysSeeq Rust core for Norns..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TARGET_TRIPLE="armv7-unknown-linux-gnueabihf"
NORNS_IP="${NORNS_IP:-norns.local}"
NORNS_USER="${NORNS_USER:-we}"
LIB_NAME="libsimon_says_seeq_core.so"
NORNS_LIB_PATH="/home/we/.local/lib/lua/5.3"

echo -e "${BLUE}Configuration:${NC}"
echo "  Target: $TARGET_TRIPLE"
echo "  Norns IP: $NORNS_IP"
echo "  Norns User: $NORNS_USER"
echo "  Library: $LIB_NAME"
echo "  Norns Library Path: $NORNS_LIB_PATH"
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Cargo.toml not found. Please run this script from the Rust project directory.${NC}"
    exit 1
fi

# Check if cross-compilation target is installed
echo -e "${YELLOW}Checking if Rust target $TARGET_TRIPLE is installed...${NC}"
if ! rustup target list --installed | grep -q "$TARGET_TRIPLE"; then
    echo -e "${YELLOW}Installing Rust target $TARGET_TRIPLE...${NC}"
    rustup target add "$TARGET_TRIPLE"
else
    echo -e "${GREEN}Rust target $TARGET_TRIPLE is already installed.${NC}"
fi

# Check for cross-compiler
echo -e "${YELLOW}Checking for ARM cross-compiler...${NC}"
if ! command -v arm-linux-gnueabihf-gcc &> /dev/null; then
    echo -e "${RED}ARM cross-compiler not found!${NC}"
    echo "Please install it with one of:"
    echo "  Ubuntu/Debian: sudo apt-get install gcc-arm-linux-gnueabihf"
    echo "  macOS: brew install arm-linux-gnueabihf-binutils"
    echo "  Arch: sudo pacman -S arm-linux-gnueabihf-gcc"
    exit 1
else
    echo -e "${GREEN}ARM cross-compiler found.${NC}"
fi

# Clean previous builds
echo -e "${YELLOW}Cleaning previous builds...${NC}"
cargo clean

# Build for Norns (ARM)
echo -e "${YELLOW}Building Rust library for Norns (ARM)...${NC}"
if cargo build --release --target "$TARGET_TRIPLE"; then
    echo -e "${GREEN}Build successful!${NC}"
else
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

# Check if the library was created
LIB_PATH="target/$TARGET_TRIPLE/release/$LIB_NAME"
if [ ! -f "$LIB_PATH" ]; then
    echo -e "${RED}Library file not found at $LIB_PATH${NC}"
    exit 1
fi

echo -e "${GREEN}Library built successfully: $LIB_PATH${NC}"

# Get file info
echo -e "${BLUE}Library information:${NC}"
ls -lh "$LIB_PATH"
file "$LIB_PATH"
echo

# Copy to Norns
echo -e "${YELLOW}Copying library to Norns...${NC}"

# Create the library directory on Norns if it doesn't exist
echo "Creating library directory on Norns..."
ssh "$NORNS_USER@$NORNS_IP" "mkdir -p $NORNS_LIB_PATH"

# Copy the library file
echo "Copying $LIB_NAME to Norns..."
if scp "$LIB_PATH" "$NORNS_USER@$NORNS_IP:$NORNS_LIB_PATH/simon_says_seeq_core.so"; then
    echo -e "${GREEN}Library copied successfully to Norns!${NC}"
else
    echo -e "${RED}Failed to copy library to Norns!${NC}"
    exit 1
fi

# Verify the library on Norns
echo -e "${YELLOW}Verifying library on Norns...${NC}"
ssh "$NORNS_USER@$NORNS_IP" "ls -la $NORNS_LIB_PATH/simon_says_seeq_core.so"

# Test if the library can be loaded
echo -e "${YELLOW}Testing library loading on Norns...${NC}"
if ssh "$NORNS_USER@$NORNS_IP" "cd /home/we && lua -e \"local core = require('simon_says_seeq_core'); print('Library loaded successfully'); local seq = core.new_sequencer(); print('Sequencer created successfully')\""; then
    echo -e "${GREEN}Library test passed! The Rust core is working on Norns.${NC}"
else
    echo -e "${RED}Library test failed! There may be an issue with the library.${NC}"
    exit 1
fi

echo
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Build and deployment completed successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo
echo "The Rust library has been built and deployed to your Norns."
echo "You can now run SimonSaysSeeqNornsHybrid.lua on your Norns and it should find the Rust core."
echo
echo "To test, SSH into your Norns and run:"
echo "  cd ~/dust/code/"
echo "  lua SimonSaysSeeqNornsHybrid.lua"
echo
echo "Or run it through the Norns menu system."