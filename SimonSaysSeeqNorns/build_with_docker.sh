#!/bin/bash

# Docker-based build script for SimonSaysSeeq Rust core for Norns
# This script uses Docker to provide a consistent cross-compilation environment

set -e  # Exit on any error

echo "Building SimonSaysSeeq Rust core for Norns using Docker..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NORNS_IP="${NORNS_IP:-norns.local}"
NORNS_USER="${NORNS_USER:-we}"
NORNS_LIB_PATH="/usr/local/lib/lua/5.3"
DOCKER_IMAGE="simon-says-seeq-builder"
CONTAINER_NAME="simon-says-seeq-build"

echo -e "${BLUE}Configuration:${NC}"
echo "  Norns IP: $NORNS_IP"
echo "  Norns User: $NORNS_USER"
echo "  Norns Library Path: $NORNS_LIB_PATH"
echo "  Docker Image: $DOCKER_IMAGE"
echo "  Lua Version: 5.3 (for Norns compatibility)"
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -f "Dockerfile" ]; then
    echo -e "${RED}Error: Cargo.toml or Dockerfile not found. Please run this script from the Rust project directory.${NC}"
    exit 1
fi

# Check if Docker is installed and running
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Docker is not installed or not in PATH.${NC}"
    echo "Please install Docker first:"
    echo "  Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

if ! docker info &> /dev/null; then
    echo -e "${RED}Docker is not running. Please start Docker first.${NC}"
    exit 1
fi

# Remove any existing container
echo -e "${YELLOW}Cleaning up previous builds...${NC}"
docker rm -f "$CONTAINER_NAME" 2>/dev/null || true

# Build Docker image with no cache to ensure clean build
echo -e "${YELLOW}Building Docker image for cross-compilation...${NC}"
if docker build --no-cache -t "$DOCKER_IMAGE" .; then
    echo -e "${GREEN}Docker image built successfully!${NC}"
else
    echo -e "${RED}Failed to build Docker image!${NC}"
    exit 1
fi

# Run the build in Docker container
echo -e "${YELLOW}Running cross-compilation in Docker container...${NC}"
if docker run --name "$CONTAINER_NAME" "$DOCKER_IMAGE"; then
    echo -e "${GREEN}Cross-compilation completed successfully!${NC}"
else
    echo -e "${RED}Cross-compilation failed!${NC}"
    exit 1
fi

# Extract the built library from the container
echo -e "${YELLOW}Extracting built library from Docker container...${NC}"
mkdir -p ./output
if docker cp "$CONTAINER_NAME:/output/simon_says_seeq_core.so" ./output/; then
    echo -e "${GREEN}Library extracted successfully!${NC}"
else
    echo -e "${RED}Failed to extract library from container!${NC}"
    exit 1
fi

# Clean up container
docker rm "$CONTAINER_NAME" > /dev/null

# Show library information
echo -e "${BLUE}Built library information:${NC}"
ls -lh ./output/simon_says_seeq_core.so
file ./output/simon_says_seeq_core.so
echo

# Ask user if they want to deploy to Norns
echo -e "${YELLOW}Do you want to deploy the library to your Norns? (y/N)${NC}"
read -r response
if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    echo -e "${YELLOW}Deploying to Norns...${NC}"
    
    # Test connectivity to Norns
    echo "Testing connection to Norns..."
    if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "$NORNS_USER@$NORNS_IP" exit 2>/dev/null; then
        echo -e "${RED}Cannot connect to Norns at $NORNS_IP${NC}"
        echo "Please check:"
        echo "  1. Norns is powered on and connected to network"
        echo "  2. SSH is enabled on Norns"
        echo "  3. IP address is correct (current: $NORNS_IP)"
        echo "  4. You can SSH manually: ssh $NORNS_USER@$NORNS_IP"
        exit 1
    fi
    
    # Create the library directory on Norns
    echo "Creating library directory on Norns..."
    ssh "$NORNS_USER@$NORNS_IP" "mkdir -p $NORNS_LIB_PATH"
    
    # Copy the library file
    echo "Copying library to Norns..."
    if scp ./output/simon_says_seeq_core.so "$NORNS_USER@$NORNS_IP:$NORNS_LIB_PATH/"; then
        echo -e "${GREEN}Library deployed successfully to Norns!${NC}"
    else
        echo -e "${RED}Failed to deploy library to Norns!${NC}"
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
        echo "The library was copied but may not be compatible or have missing dependencies."
    fi
    
    echo
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Build and deployment completed!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo
    echo "Your SimonSaysSeeq Hybrid script should now work with the Rust core."
    echo "Try running it on your Norns!"
    
else
    echo -e "${BLUE}Library built and saved to: ./output/simon_says_seeq_core.so${NC}"
    echo "To manually deploy to Norns:"
    echo "  scp ./output/simon_says_seeq_core.so $NORNS_USER@$NORNS_IP:$NORNS_LIB_PATH/"
fi

echo
echo "To clean up Docker images later, run:"
echo "  docker rmi $DOCKER_IMAGE"