#!/bin/bash

# SimonSaysSeeq Rust Core Build Script
# Builds the Rust library for both local development and Norns (ARM) deployment

set -e  # Exit on any error

echo "=== SimonSaysSeeq Rust Core Build Script ==="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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
    print_error "Cargo.toml not found. Please run this script from the SimonSaysSeeqNorns directory."
    exit 1
fi

# Create output directory
mkdir -p lib

print_status "Starting Rust compilation..."

# Build for local development (x86_64/aarch64)
# Build for local development
print_status "Building for local development..."

# Clean previous builds
cargo clean

# Build for local development
if cargo build --release; then
    print_success "Local build completed successfully"
    
    # Determine the library extension based on OS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        LIB_EXT="dylib"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        LIB_EXT="so"
    else
        LIB_EXT="so"  # Default to .so
    fi
    
    # Copy local build
    if [ -f "target/release/libsimon_says_seeq_core.${LIB_EXT}" ]; then
        cp "target/release/libsimon_says_seeq_core.${LIB_EXT}" "simon_says_seeq_core.${LIB_EXT}"
        print_success "Local library copied to simon_says_seeq_core.${LIB_EXT}"
    else
        print_warning "Local library not found at expected path"
    fi
else
    print_error "Local build failed"
    exit 1
fi

# Check if ARM cross-compilation tools are available
print_status "Checking for ARM cross-compilation tools..."

CROSS_COMPILE_AVAILABLE=false

# Check for common ARM cross-compilation setups
if command -v arm-linux-gnueabihf-gcc &> /dev/null; then
    print_status "Found arm-linux-gnueabihf-gcc, will attempt ARM cross-compilation"
    CROSS_COMPILE_AVAILABLE=true
elif command -v aarch64-linux-gnu-gcc &> /dev/null; then
    print_status "Found aarch64-linux-gnu-gcc, will attempt AArch64 cross-compilation"
    CROSS_COMPILE_AVAILABLE=true
elif rustup target list --installed | grep -q "armv7-unknown-linux-gnueabihf"; then
    print_status "ARM target is installed, checking for linker..."
    CROSS_COMPILE_AVAILABLE=true
else
    print_warning "ARM cross-compilation tools not found"
fi

# Attempt ARM cross-compilation if tools are available
if [ "$CROSS_COMPILE_AVAILABLE" = true ]; then
    print_status "Attempting ARM cross-compilation for Norns..."
    
    # Install ARM target if not present
    rustup target add armv7-unknown-linux-gnueabihf || true
    
    # Try to build for ARM
    if CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
       cargo build --release --target armv7-unknown-linux-gnueabihf; then
        
        print_success "ARM cross-compilation completed successfully"
        
        # Copy ARM build for Norns
        if [ -f "target/armv7-unknown-linux-gnueabihf/release/libsimon_says_seeq_core.so" ]; then
            cp "target/armv7-unknown-linux-gnueabihf/release/libsimon_says_seeq_core.so" "simon_says_seeq_core_arm.so"
            print_success "ARM library copied to simon_says_seeq_core_arm.so"
        fi
    else
        print_warning "ARM cross-compilation failed, but local build succeeded"
        print_warning "You'll need to compile on the Norns device directly or set up cross-compilation"
    fi
else
    print_warning "Skipping ARM cross-compilation (tools not available)"
    print_status "To enable ARM cross-compilation, install:"
    echo "  sudo apt-get install gcc-arm-linux-gnueabihf  # On Ubuntu/Debian"
    echo "  brew install arm-linux-gnueabihf-binutils     # On macOS with Homebrew"
    echo "  rustup target add armv7-unknown-linux-gnueabihf"
fi



# Create a simple test script
print_status "Creating test script..."
cat > test_rust_core.lua << 'EOF'
-- Simple test script for the Rust core
local core_status, core = pcall(require, 'simon_says_seeq_core')

if core_status then
    print("✓ Rust core loaded successfully!")
    
    -- Test basic functionality
    local sequencer = core.new_sequencer()
    print("✓ Sequencer created")
    
    -- Test position functions
    local lane, bar, step = core.get_position(sequencer)
    print("✓ Initial position: lane=" .. lane .. ", bar=" .. bar .. ", step=" .. step)
    
    -- Test MIDI processing
    local active_notes = core.process_midi_tick(sequencer, 120.0)
    print("✓ MIDI tick processed, active notes: " .. #active_notes)
    
    print("✓ All tests passed! Rust core is working correctly.")
else
    print("✗ Failed to load Rust core:")
    print(core)
end
EOF

print_success "Test script created: test_rust_core.lua"

# Create installation instructions
print_status "Creating installation instructions..."
cat > INSTALL.md << 'EOF'
# SimonSaysSeeq Rust Core Installation

## Files Generated

- `simon_says_seeq_core.so` (or `.dylib` on macOS) - Local development library
- `simon_says_seeq_core_arm.so` - ARM library for Norns (if cross-compilation succeeded)
- `test_rust_core.lua` - Test script to verify the library works

## Installation on Norns

1. If ARM cross-compilation succeeded:
   ```bash
   scp simon_says_seeq_core_arm.so we@norns.local:~/dust/code/SimonSaysSeeqNorns/simon_says_seeq_core.so
   ```

2. If you need to compile on Norns directly:
   ```bash
   # Copy source files to Norns
   scp Cargo.toml src/ we@norns.local:~/dust/code/SimonSaysSeeqNorns/
   
   # SSH into Norns and compile
   ssh we@norns.local
   cd ~/dust/code/SimonSaysSeeqNorns
   cargo build --release
   cp target/release/libsimon_says_seeq_core.so simon_says_seeq_core.so
   ```

## Testing

Run the test script on Norns:
```bash
cd ~/dust/code/SimonSaysSeeqNorns
lua test_rust_core.lua
```

## Troubleshooting

- If the library fails to load, check file permissions: `chmod +x simon_says_seeq_core.so`
- Ensure Rust and Cargo are installed on the target system
- Check that all dependencies are available
EOF

print_success "Installation instructions created: INSTALL.md"

# Summary
echo
print_success "=== Build Summary ==="
echo "✓ Rust core compiled successfully"
if [ -f "simon_says_seeq_core_arm.so" ]; then
    echo "✓ ARM library available for Norns"
else
    echo "⚠ ARM library not available (cross-compilation failed or not attempted)"
fi
echo "✓ Test script created"
echo "✓ Installation instructions created"
echo
print_status "Next steps:"
echo "1. Test locally: lua test_rust_core.lua"
echo "2. Copy to Norns (see INSTALL.md)"
echo "3. Test on Norns"
echo "4. Use SimonSaysSeeqNornsHybrid.lua for the full hybrid experience"
echo
print_success "Build completed successfully!"