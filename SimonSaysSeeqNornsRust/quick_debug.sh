#!/bin/bash

# Quick Norns Input Debug Script
# Simple script to debug button input for boot selector

echo "=== Norns Input Device Debug ==="

# Check input devices
echo -e "\n1. Available input devices:"
ls -la /dev/input/

# Check device details
echo -e "\n2. Input device details:"
cat /proc/bus/input/devices 2>/dev/null || echo "No device info available"

# Check overlays
echo -e "\n3. Boot config overlays:"
grep -i overlay /boot/config.txt 2>/dev/null || echo "No overlays found"

# Check permissions
echo -e "\n4. Current user groups:"
groups

# Check for norns-specific files
echo -e "\n5. Norns overlay files:"
ls -la /boot/overlays/*norns* 2>/dev/null || echo "No norns overlays found"

# Quick button test
echo -e "\n6. Testing button input (press any key to continue)..."
read -n 1 -s

echo "Now testing input devices for 5 seconds..."
echo "Press K2 and K3 buttons now!"

for device in /dev/input/event*; do
    if [ -r "$device" ]; then
        echo "Testing $device..."
        timeout 2 hexdump -C "$device" 2>/dev/null | head -3 | while read line; do
            echo "  $line"
        done &
    fi
done

sleep 5
pkill -f hexdump 2>/dev/null || true

echo -e "\nDebug complete!"
echo "Look for hex patterns when you pressed buttons above."
echo "Common patterns:"
echo "  K2 usually shows code '02' in hex output"
echo "  K3 usually shows code '03' in hex output"