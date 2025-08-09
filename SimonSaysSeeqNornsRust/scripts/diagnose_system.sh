#!/bin/bash

# System diagnosis script for SimonSaysSeeq grid discovery issues
# This script checks for common issues that can cause system hangs

set -e

echo "🔍 SimonSaysSeeq System Diagnosis Script"
echo "========================================"
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_section() {
    echo -e "${BLUE}### $1${NC}"
    echo ""
}

print_ok() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_section "System Information"
echo "Kernel: $(uname -r)"
echo "Distribution: $(lsb_release -d 2>/dev/null | cut -f2 || echo 'Unknown')"
echo "Architecture: $(uname -m)"
echo "Uptime: $(uptime -p 2>/dev/null || uptime)"
echo ""

print_section "Memory and CPU Usage"
echo "Memory usage:"
free -h | head -2
echo ""
echo "CPU load averages:"
uptime | awk -F'load average:' '{print $2}'
echo ""
echo "Top memory consumers:"
ps aux --sort=-%mem | head -5
echo ""

print_section "Rust and Cargo Environment"
if command -v rustc &> /dev/null; then
    print_ok "Rust installed: $(rustc --version)"
    print_ok "Cargo version: $(cargo --version)"
else
    print_error "Rust/Cargo not found in PATH"
fi
echo ""

print_section "USB Device Detection"
if command -v lsusb &> /dev/null; then
    echo "All USB devices:"
    lsusb
    echo ""
    
    echo "Checking for Monome devices (VID CAFE or 0A6A):"
    if lsusb | grep -iE "(cafe|0a6a)" > /dev/null; then
        print_ok "Potential Monome devices found:"
        lsusb | grep -iE "(cafe|0a6a)" | while read line; do
            echo "  $line"
        done
    else
        print_warning "No Monome devices detected (VID CAFE or 0A6A)"
        echo "  This is normal if no grids are connected"
    fi
else
    print_error "lsusb command not available"
fi
echo ""

print_section "Serial Port Detection"
echo "Available serial ports:"
if ls /dev/tty{ACM,USB}* 2>/dev/null; then
    print_ok "Serial ports found"
    for port in /dev/tty{ACM,USB}*; do
        if [ -e "$port" ]; then
            echo "  $port - $(stat -c '%A %U:%G' "$port")"
        fi
    done
else
    print_warning "No /dev/ttyACM* or /dev/ttyUSB* devices found"
fi
echo ""

print_section "User Permissions"
echo "Current user: $(whoami)"
echo "User groups: $(groups)"
echo ""

# Check critical groups
if groups | grep -q dialout; then
    print_ok "User is in 'dialout' group (can access serial ports)"
else
    print_error "User NOT in 'dialout' group"
    echo "  Fix: sudo usermod -a -G dialout $USER"
    echo "  Then log out and back in"
fi

if groups | grep -q tty; then
    print_ok "User is in 'tty' group"
else
    print_warning "User not in 'tty' group (may be needed for some serial devices)"
    echo "  Consider: sudo usermod -a -G tty $USER"
fi
echo ""

print_section "Recent System Messages"
echo "Recent USB-related kernel messages (last 20 lines):"
dmesg | grep -i usb | tail -20 || echo "No USB messages or dmesg not accessible"
echo ""

echo "Recent serial/tty messages:"
dmesg | grep -iE "(tty|serial|acm)" | tail -10 || echo "No serial messages found"
echo ""

print_section "Process Information"
echo "Checking for hanging Rust processes:"
if pgrep -f "cargo\|rust" > /dev/null; then
    print_warning "Found active Rust/Cargo processes:"
    ps aux | grep -E "(cargo|rust)" | grep -v grep
    echo ""
    echo "If these are stuck, you may need to kill them:"
    echo "  pkill -f cargo"
    echo "  pkill -f rust"
else
    print_ok "No hanging Rust/Cargo processes detected"
fi
echo ""

print_section "System Resource Limits"
echo "File descriptor limits:"
ulimit -n
echo ""

echo "Maximum processes:"
ulimit -u
echo ""

print_section "Cargo Build Environment"
if [ -d "target" ]; then
    print_ok "Cargo target directory exists"
    echo "Target directory size: $(du -sh target 2>/dev/null | cut -f1)"
else
    print_warning "No cargo target directory (run from wrong location?)"
fi
echo ""

if [ -f "Cargo.toml" ]; then
    print_ok "Cargo.toml found"
    echo "Project name: $(grep '^name' Cargo.toml | head -1)"
else
    print_error "No Cargo.toml found - run this script from the project root"
fi
echo ""

print_section "Hardware-specific Checks"
echo "Checking for potential problematic hardware:"

# Check for known problematic USB controllers
if lspci | grep -i "usb\|serial" > /dev/null; then
    echo "USB/Serial controllers:"
    lspci | grep -iE "(usb|serial)" | while read line; do
        echo "  $line"
    done
else
    print_warning "No USB controllers detected with lspci"
fi
echo ""

print_section "Recommendations"
echo "Based on this diagnosis:"
echo ""

# Check if we're likely to have issues
has_monome=false
has_permissions=false
has_serial_ports=false

if lsusb 2>/dev/null | grep -iE "(cafe|0a6a)" > /dev/null; then
    has_monome=true
fi

if groups | grep -q dialout; then
    has_permissions=true
fi

if ls /dev/tty{ACM,USB}* 2>/dev/null > /dev/null; then
    has_serial_ports=true
fi

if $has_monome && $has_permissions && $has_serial_ports; then
    print_ok "System appears configured correctly for Monome grid detection"
    echo "  If you still experience hangs, try the safe discovery example:"
    echo "  cargo run --example test_grid_discovery_safe --features desktop"
elif ! $has_permissions; then
    print_error "PRIORITY: Fix permission issues first"
    echo "  1. Add user to dialout group: sudo usermod -a -G dialout $USER"
    echo "  2. Log out and back in"
    echo "  3. Run this diagnosis script again"
elif ! $has_serial_ports && $has_monome; then
    print_warning "Monome device detected but no serial ports visible"
    echo "  This suggests a driver or permission issue"
    echo "  Try reconnecting the device or checking dmesg output"
else
    print_ok "Run the safe discovery example to test without risk:"
    echo "  cargo run --example test_grid_discovery_safe --features desktop"
fi

echo ""
echo "🏁 Diagnosis complete. If issues persist, share this output when asking for help."