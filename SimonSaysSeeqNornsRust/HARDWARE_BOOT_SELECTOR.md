# SimonSaysSeeq Hardware Boot Selector (Shell Script)

This document explains how to implement **hardware button-based boot selection** for your Norns device using a lightweight shell script, allowing you to choose between normal menu startup and direct Rust app launch by holding buttons during boot.

## Overview

The hardware boot selector runs **before** Norns fully starts up and detects button presses during the boot process. This gives you:

- **K2 held during boot**: Launch SimonSaysSeeqRust directly
- **K3 held during boot**: Boot to normal Norns menu  
- **No buttons held**: Use previously saved preference
- **5-second timeout**: Automatic selection if no input detected

## Why Shell Script?

The shell script approach provides:

- **Minimal dependencies**: Only requires bash and basic system tools
- **Fast execution**: No interpreter overhead, starts immediately
- **Simple and reliable**: Fewer things that can break
- **Easy to understand**: Every line is readable and modifiable
- **Norns-optimized**: Tailored specifically for standard Norns hardware
- **Lightweight**: <1MB memory footprint during execution

## Installation

### Automatic Installation (Recommended)
```bash
# SSH to your Norns
ssh we@norns.local

# Navigate to the project directory
cd /home/we/dust/code/SimonSaysSeeqRust

# Run the installer (requires sudo)
sudo ./install_boot_selector.sh
```

The installer will:
1. Check system requirements (bash, bc, GPIO access)
2. Set up GPIO permissions and udev rules
3. Install and configure the systemd service
4. Test the installation
5. Provide usage instructions

### Test Before Installing
```bash
# Test the selector without installing
sudo ./install_boot_selector.sh --test
```

### Manual Installation

#### 1. Copy Files
```bash
# Copy selector script to your Norns
scp hardware_boot_selector.sh we@norns.local:/home/we/dust/code/SimonSaysSeeqRust/
scp simonsaysseeq-boot-selector.service we@norns.local:/tmp/
```

#### 2. Install Service
```bash
# SSH to Norns
ssh we@norns.local

# Install systemd service
sudo cp /tmp/simonsaysseeq-boot-selector.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable simonsaysseeq-boot-selector.service

# Make script executable
chmod +x /home/we/dust/code/SimonSaysSeeqRust/hardware_boot_selector.sh
```

#### 3. Set Up Permissions
```bash
# Add user to GPIO group
sudo usermod -a -G gpio we

# Create udev rules for GPIO access
sudo tee /etc/udev/rules.d/99-simonsaysseeq-gpio.rules << 'EOF'
KERNEL=="gpiochip*", GROUP="gpio", MODE="0664"
SUBSYSTEM=="gpio", GROUP="gpio", MODE="0664"
SUBSYSTEM=="gpio", KERNEL=="export", GROUP="gpio", MODE="0220"
SUBSYSTEM=="gpio", KERNEL=="unexport", GROUP="gpio", MODE="0220"
EOF

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

## Requirements

### System Dependencies
- **bash shell** (standard on all Linux systems)
- **bc calculator** (for timing calculations)
- **systemd** (for service management)
- **GPIO sysfs interface** (standard on Raspberry Pi/Norns)

### Hardware Requirements
- **Standard Norns device** (any generation)
- **K2 and K3 buttons** functioning normally
- **GPIO pins 27 and 22** accessible (standard Norns layout)

## Configuration

### GPIO Pin Mappings
```bash
# Default Norns mappings (in hardware_boot_selector.sh)
K2_GPIO=27  # Key 2 - Select Rust app
K3_GPIO=22  # Key 3 - Select Normal menu
```

### Customizing Pin Mappings
Edit the pin assignments in `hardware_boot_selector.sh`:
```bash
# Custom pin mappings for modified hardware
K2_GPIO=17  # Your custom K2 pin
K3_GPIO=18  # Your custom K3 pin
```

### Timing Configuration
```bash
# Adjust timeout and polling interval
TIMEOUT_SECONDS=5      # How long to wait for button press
CHECK_INTERVAL=0.1     # How often to check buttons (seconds)
```

### Default Behavior
- **First run**: Defaults to normal menu mode
- **Saved preference**: Remembers your last manual choice
- **Config file**: `/home/we/.config/simonsaysseeq/startup_mode`

## Usage

### During Boot
1. **Power on** your Norns
2. **Within 5 seconds** of startup:
   - **Hold K2**: Direct to Rust app
   - **Hold K3**: Normal menu
   - **Do nothing**: Use saved preference
3. **Release button** once selection is made

### Via SSH (Manual Override)
```bash
# Check current status
./toggle_startup_mode.sh status

# Set modes manually
./toggle_startup_mode.sh rust    # Direct Rust app
./toggle_startup_mode.sh menu    # Normal menu

# Start/stop service without changing boot mode
./toggle_startup_mode.sh start
./toggle_startup_mode.sh stop
```

## How It Works

### Boot Process Integration
1. **systemd service** starts during early boot
2. **GPIO pins** are configured for input with pull-up resistors
3. **Button monitoring** begins for the timeout period
4. **Selection made** based on button state or timeout
5. **System configuration** updated (enable/disable Rust service)
6. **Normal boot** continues with chosen mode

### GPIO Button Detection
```bash
# GPIO sysfs interface usage
echo $pin > /sys/class/gpio/export           # Make pin available
echo "in" > /sys/class/gpio/gpio$pin/direction   # Set as input
cat /sys/class/gpio/gpio$pin/value           # Read button state (0=pressed, 1=released)
```

### Service Management
```bash
# For Rust app mode
systemctl enable simonsaysseeq-rust    # Auto-start on boot
systemctl start simonsaysseeq-rust     # Start now

# For menu mode  
systemctl disable simonsaysseeq-rust   # Disable auto-start
systemctl stop simonsaysseeq-rust      # Stop if running
```

## Troubleshooting

### Boot Selector Not Working
```bash
# Check if service is installed and enabled
systemctl status simonsaysseeq-boot-selector

# Check logs
journalctl -u simonsaysseeq-boot-selector -f
tail -f /tmp/simonsaysseeq_boot_selector.log

# Test script manually
sudo /home/we/dust/code/SimonSaysSeeqRust/hardware_boot_selector.sh
```

### Button Detection Issues
```bash
# Check GPIO sysfs availability
ls -la /sys/class/gpio/

# Test GPIO pin access manually
echo 27 > /sys/class/gpio/export
cat /sys/class/gpio/gpio27/value
echo 27 > /sys/class/gpio/unexport

# Check permissions
groups we  # Should include 'gpio' group
```

### Permission Problems
```bash
# Re-add user to gpio group
sudo usermod -a -G gpio we

# Check udev rules
cat /etc/udev/rules.d/99-simonsaysseeq-gpio.rules

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### Service Won't Start
```bash
# Check systemd service status
sudo systemctl status simonsaysseeq-boot-selector

# Check script permissions and syntax
ls -la /home/we/dust/code/SimonSaysSeeqRust/hardware_boot_selector.sh
bash -n /home/we/dust/code/SimonSaysSeeqRust/hardware_boot_selector.sh

# Check dependencies
which bc  # Should return /usr/bin/bc or similar
```

### Script Errors
```bash
# Check for missing dependencies
sudo apt-get install bc

# Verify GPIO sysfs support
ls /sys/class/gpio/

# Test bc calculator
echo "1 + 1" | bc  # Should output: 2
```

## Uninstallation

### Automatic Uninstall
```bash
sudo ./install_boot_selector.sh --uninstall
```

### Manual Uninstall
```bash
# Stop and disable service
sudo systemctl stop simonsaysseeq-boot-selector
sudo systemctl disable simonsaysseeq-boot-selector

# Remove files
sudo rm -f /etc/systemd/system/simonsaysseeq-boot-selector.service
sudo rm -f /etc/udev/rules.d/99-simonsaysseeq-gpio.rules

# Reload systemd
sudo systemctl daemon-reload
sudo udevadm control --reload-rules
```

## Advanced Configuration

### Custom Timeout
```bash
# Edit hardware_boot_selector.sh
TIMEOUT_SECONDS=10  # Wait 10 seconds instead of 5
```

### Different Button Mapping
```bash
# Use K1 and K2 instead of K2 and K3
K1_GPIO=17  # K1 for Rust app
K2_GPIO=27  # K2 for menu mode
```

### Conditional Activation
```bash
# Only activate selector if Grid is connected
if lsusb | grep -q "monome"; then
    # Run boot selector
    main "$@"
else
    # Skip to saved mode
    exit 0
fi
```

### Debug Mode
```bash
# Enable verbose logging in the script
DEBUG=1 ./hardware_boot_selector.sh

# Or modify the script to add debug output
echo "DEBUG: Button K2 state = $k2_state" >> /tmp/debug.log
```

## Integration with Other Scripts

### Skip Boot Selector
```bash
# Create skip file to bypass selector
touch /tmp/simonsaysseeq_skip_boot_selector
```

### External Control
```bash
# Force specific mode from another script
echo "rust" > /home/we/.config/simonsaysseeq/startup_mode
```

### Status Checking
```bash
# Check current mode from other scripts
current_mode=$(cat /home/we/.config/simonsaysseeq/startup_mode 2>/dev/null || echo "menu")
if [ "$current_mode" = "rust" ]; then
    echo "Rust app mode is active"
fi
```

## Performance and Resources

### System Impact
- **Boot time**: Adds exactly 5 seconds (timeout period)
- **Memory usage**: <1MB during execution
- **CPU usage**: Minimal polling during detection phase
- **Storage**: ~10KB for script and config files

### Optimization Tips
```bash
# Reduce timeout for faster boot
TIMEOUT_SECONDS=3

# Increase polling frequency for more responsive detection
CHECK_INTERVAL=0.05
```

## Security Considerations

- **Root privileges**: Required for systemd and GPIO access
- **File permissions**: Config files owned by 'we' user
- **GPIO access**: Limited to specific button pins only
- **Service isolation**: Runs with restricted system access

## Development and Testing

### Test Without Rebooting
```bash
# Test the selector script directly
sudo ./hardware_boot_selector.sh

# Test with debug output
DEBUG=1 sudo ./hardware_boot_selector.sh
```

### Simulate Button Presses
```bash
# Manually trigger GPIO states for testing
echo 27 > /sys/class/gpio/export
echo "out" > /sys/class/gpio/gpio27/direction
echo 0 > /sys/class/gpio/gpio27/value  # Simulate button press
echo 1 > /sys/class/gpio/gpio27/value  # Simulate button release
echo 27 > /sys/class/gpio/unexport
```

### Script Debugging
```bash
# Add debug output to script
echo "DEBUG: $(date) - Button check started" >> /tmp/debug.log

# Check script syntax
bash -n hardware_boot_selector.sh

# Run with verbose shell output
bash -x hardware_boot_selector.sh
```

## Example Modifications

### Add Third Button Option
```bash
# Add K1 for a special mode
K1_GPIO=17
# ... in the button checking loop:
if [ "$(read_gpio $K1_GPIO)" = "0" ]; then
    echo "special" > $CONFIG_FILE
    # Configure special mode
fi
```

### Different Hardware Layout
```bash
# For custom Norns builds with different GPIO pins
K2_GPIO=18  # Your custom button pin
K3_GPIO=19  # Your custom button pin
```

### Longer Hold Required
```bash
# Require button to be held for 2 seconds
HOLD_REQUIRED=2
hold_time=0
while [ $hold_time -lt $HOLD_REQUIRED ]; do
    if [ "$(read_gpio $K2_GPIO)" = "0" ]; then
        hold_time=$(echo "$hold_time + $CHECK_INTERVAL" | bc)
    else
        hold_time=0  # Reset if button released
    fi
    sleep $CHECK_INTERVAL
done
```

## Support and Maintenance

### Log Files
- **Service logs**: `journalctl -u simonsaysseeq-boot-selector`
- **Script logs**: `/tmp/simonsaysseeq_boot_selector.log`
- **Debug logs**: `/tmp/debug.log` (if enabled)

### Regular Maintenance
```bash
# Check service status monthly
systemctl status simonsaysseeq-boot-selector

# Clean old logs
sudo journalctl --vacuum-time=30d

# Verify GPIO permissions
ls -la /sys/class/gpio/
```

### Getting Help
For issues, collect this information:
- Output of `./install_boot_selector.sh --test`
- Contents of `/tmp/simonsaysseeq_boot_selector.log`
- Output of `systemctl status simonsaysseeq-boot-selector`
- Your Norns model and any hardware modifications

## Conclusion

The shell script hardware boot selector provides a simple, reliable way to choose your Norns startup mode during boot. With minimal dependencies and fast execution, it's optimized for the standard Norns hardware while remaining easy to understand and modify.

The 5-second timeout ensures you can make a quick selection during boot, while the saved preference system means you don't need to interact with it every time if you have a preferred mode.

For most Norns users, this solution provides the perfect balance of functionality, reliability, and simplicity.