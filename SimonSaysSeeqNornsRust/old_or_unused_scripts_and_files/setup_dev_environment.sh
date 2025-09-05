#!/bin/bash

# Development Environment Setup for SimonSaysSeeqNornsRust
# This script installs all required dependencies for building and deploying to Norns

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 SimonSaysSeeq Development Environment Setup${NC}"
echo "====================================================="

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

# Detect OS
OS=""
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
    if [ -f /etc/debian_version ]; then
        DISTRO="debian"
    elif [ -f /etc/fedora-release ]; then
        DISTRO="fedora"
    elif [ -f /etc/arch-release ]; then
        DISTRO="arch"
    else
        DISTRO="unknown"
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    OS="windows"
else
    OS="unknown"
fi

print_status "Detected OS: $OS"

# Check if Rust is installed
print_status "Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    print_warning "Rust not found. Installing Rust via rustup..."
    
    if [ "$OS" = "windows" ]; then
        print_error "Please install Rust manually on Windows: https://rustup.rs/"
        exit 1
    else
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
else
    print_success "Rust is already installed"
fi

# Show Rust version
rustc --version
cargo --version

# Install ARM cross-compilation target
print_status "Installing ARM cross-compilation target..."
rustup target add armv7-unknown-linux-gnueabihf

# Install additional useful targets
rustup target add aarch64-unknown-linux-gnu

# Install system dependencies based on OS
print_status "Installing system dependencies..."

case "$OS" in
    "linux")
        case "$DISTRO" in
            "debian")
                print_status "Installing dependencies for Debian/Ubuntu..."
                sudo apt update
                
                # Core build tools
                sudo apt install -y build-essential pkg-config
                
                # Cross-compilation tools
                sudo apt install -y gcc-arm-linux-gnueabihf gcc-aarch64-linux-gnu
                
                # Audio libraries (for cpal)
                sudo apt install -y libasound2-dev portaudio19-dev
                
                # HID libraries (for grid communication)
                sudo apt install -y libhidapi-dev libudev-dev
                
                # Input libraries (for evdev)
                sudo apt install -y libevdev-dev libinput-dev
                
                # Additional useful tools
                sudo apt install -y openssh-client rsync
                
                print_success "Debian/Ubuntu dependencies installed"
                ;;
                
            "fedora")
                print_status "Installing dependencies for Fedora..."
                
                # Core build tools
                sudo dnf install -y gcc gcc-c++ pkgconf-pkg-config
                
                # Cross-compilation tools
                sudo dnf install -y gcc-arm-linux-gnu gcc-aarch64-linux-gnu
                
                # Audio libraries
                sudo dnf install -y alsa-lib-devel portaudio-devel
                
                # HID libraries
                sudo dnf install -y hidapi-devel systemd-devel
                
                # Input libraries
                sudo dnf install -y libevdev-devel libinput-devel
                
                # Additional tools
                sudo dnf install -y openssh-clients rsync
                
                print_success "Fedora dependencies installed"
                ;;
                
            "arch")
                print_status "Installing dependencies for Arch Linux..."
                
                # Core build tools
                sudo pacman -S --needed base-devel pkgconf
                
                # Cross-compilation tools (from AUR)
                if ! command -v arm-linux-gnueabihf-gcc &> /dev/null; then
                    print_warning "ARM cross-compiler not found. Please install from AUR:"
                    echo "  yay -S arm-linux-gnueabihf-gcc"
                    echo "  or manually build arm-linux-gnueabihf toolchain"
                fi
                
                # Audio libraries
                sudo pacman -S --needed alsa-lib portaudio
                
                # HID libraries
                sudo pacman -S --needed hidapi systemd
                
                # Input libraries
                sudo pacman -S --needed libevdev libinput
                
                # Additional tools
                sudo pacman -S --needed openssh rsync
                
                print_success "Arch Linux dependencies installed"
                ;;
                
            *)
                print_warning "Unknown Linux distribution. Please install manually:"
                echo "  - Cross-compilation toolchain (arm-linux-gnueabihf-gcc)"
                echo "  - ALSA development libraries"
                echo "  - HID API libraries"
                echo "  - Input device libraries"
                ;;
        esac
        ;;
        
    "macos")
        print_status "Installing dependencies for macOS..."
        
        # Check for Homebrew
        if ! command -v brew &> /dev/null; then
            print_warning "Homebrew not found. Installing Homebrew..."
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        fi
        
        # Install cross-compilation tools
        brew install arm-unknown-linux-gnueabihf
        
        # Audio libraries
        brew install portaudio
        
        # HID libraries
        brew install hidapi
        
        # Additional tools
        brew install rsync
        
        print_success "macOS dependencies installed"
        ;;
        
    "windows")
        print_error "Windows setup requires manual configuration:"
        echo "  1. Install Rust: https://rustup.rs/"
        echo "  2. Install Windows Subsystem for Linux (WSL2)"
        echo "  3. Install cross-compilation tools in WSL2"
        echo "  4. Or use Docker for cross-compilation"
        exit 1
        ;;
        
    *)
        print_error "Unknown operating system. Please install dependencies manually."
        exit 1
        ;;
esac

# Install useful Rust tools
print_status "Installing Rust development tools..."

# Cargo tools for development
cargo install --locked cargo-watch || print_warning "cargo-watch installation failed"
cargo install --locked cargo-edit || print_warning "cargo-edit installation failed"
cargo install --locked cargo-audit || print_warning "cargo-audit installation failed"

# Cross-compilation helper
cargo install --locked cross || print_warning "cross installation failed"

print_success "Rust tools installed"

# Set up udev rules for USB device access (Linux only)
if [ "$OS" = "linux" ]; then
    print_status "Setting up USB device permissions..."
    
    # Create udev rules for monome grid
    sudo tee /etc/udev/rules.d/40-monome.rules > /dev/null << 'EOF'
# Monome Grid devices
SUBSYSTEM=="usb", ATTR{idVendor}=="0403", ATTR{idProduct}=="6001", MODE="0666", GROUP="plugdev"
SUBSYSTEM=="usb", ATTR{idVendor}=="0403", ATTR{idProduct}=="6010", MODE="0666", GROUP="plugdev"
SUBSYSTEM=="usb", ATTR{idVendor}=="0403", ATTR{idProduct}=="6011", MODE="0666", GROUP="plugdev"

# HID devices
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", MODE="0666", GROUP="plugdev"
EOF
    
    # Add user to plugdev group
    sudo usermod -a -G plugdev $USER
    
    # Reload udev rules
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    
    print_success "USB permissions configured"
    print_warning "You may need to log out and back in for group changes to take effect"
fi

# Verify installation
print_status "Verifying installation..."

# Check Rust targets
if rustup target list --installed | grep -q "armv7-unknown-linux-gnueabihf"; then
    print_success "ARM target installed"
else
    print_error "ARM target installation failed"
fi

# Check cross-compiler
if command -v arm-linux-gnueabihf-gcc &> /dev/null; then
    print_success "ARM cross-compiler available"
    arm-linux-gnueabihf-gcc --version | head -1
else
    print_warning "ARM cross-compiler not found - needed for Norns deployment"
fi

# Create project directory structure
print_status "Setting up project structure..."

# Make scripts executable
chmod +x deploy_to_norns.sh
chmod +x test_local.sh

# Create data directories
mkdir -p ~/.local/share/simonsaysseeq
mkdir -p ~/.config/simonsaysseeq

print_success "Project structure created"

# Final summary
echo ""
print_success "Development environment setup completed! 🎉"
echo ""
echo "Summary:"
echo "  ✅ Rust toolchain with ARM cross-compilation"
echo "  ✅ System dependencies for audio, HID, and input"
echo "  ✅ Development tools installed"
echo "  ✅ USB permissions configured (Linux)"
echo "  ✅ Project scripts made executable"
echo ""
echo "Next steps:"
echo "  1. Test local build:     ./test_local.sh"
echo "  2. Test cross-compile:   ./test_local.sh --hardware"
echo "  3. Deploy to Norns:      ./deploy_to_norns.sh"
echo ""
echo "Environment variables you can set:"
echo "  NORNS_IP=your.norns.ip     # IP address of your Norns"
echo "  NORNS_USER=we              # Username on Norns (default: we)"
echo "  RUST_LOG=debug             # Increase logging verbosity"
echo ""

if [ "$OS" = "linux" ] && groups $USER | grep -q plugdev; then
    echo "USB access: Ready"
else
    print_warning "USB access: You may need to log out and back in for USB access"
fi

echo ""
echo "Happy coding! 🦀🎵"