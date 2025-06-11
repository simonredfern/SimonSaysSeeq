# GPIO Pin Discovery for Norns Hardware Boot Selector

## ⚠️ IMPORTANT: Verify GPIO Pins Before Deployment

The GPIO pin numbers in `hardware_boot_selector.sh` (K2_GPIO=27, K3_GPIO=22) are **educated guesses** based on common Raspberry Pi configurations. **These may be completely wrong for your specific Norns device.**

Different Norns models, revisions, and modifications use different GPIO pin mappings. You **must** verify the correct pins before deploying the boot selector.

## Quick Discovery

```bash
# Run the automated discovery tool
sudo ./discover_gpio_pins.sh quick
```

This will:
1. Find available GPIO pins
2. Test button responses interactively
3. Generate a configuration file with correct pin numbers
4. Check existing Norns configuration files

## Manual Discovery Methods

### Method 1: Interactive Testing (Recommended)

```bash
# SSH to your Norns
ssh we@norns.local
cd /home/we/dust/code/SimonSaysSeeqRust

# Run discovery script
sudo ./discover_gpio_pins.sh

# Choose option 2: Interactive button testing
# Follow prompts to press K2 and K3 buttons
```

### Method 2: Live Monitoring

```bash
# Monitor all GPIO pins while pressing buttons
sudo ./discover_gpio_pins.sh monitor

# Press K2 and K3 buttons to see which pins change
# Press Ctrl+C to stop monitoring
```

### Method 3: Check Norns Source Code

Look for GPIO definitions in Norns source:

```bash
# Check Norns configuration files
find /home/we -name "*.lua" -exec grep -l -i "gpio\|button\|key" {} \;
grep -r -i "gpio.*[0-9]" /home/we/norns/ 2>/dev/null | head -10
```

### Method 4: Manual GPIO Testing

```bash
# Test specific GPIO pins manually
# Replace XX with pin number to test

# Export pin for testing
echo XX > /sys/class/gpio/export

# Set as input
echo "in" > /sys/class/gpio/gpioXX/direction

# Read current value
cat /sys/class/gpio/gpioXX/value

# Press button and read again
cat /sys/class/gpio/gpioXX/value

# Clean up
echo XX > /sys/class/gpio/unexport
```

### Method 5: Check Hardware Documentation

Different Norns models have different GPIO layouts:

- **Original Norns**: Custom GPIO mapping
- **Norns Shield**: Raspberry Pi GPIO with modifications  
- **DIY Builds**: Varies by implementation

Check your specific hardware documentation or schematic.

## Common GPIO Pin Ranges

Most Norns devices use GPIO pins in these ranges:

```bash
# Common button GPIO pins to test
Common pins: 2, 3, 4, 17, 18, 22, 23, 24, 25, 27

# Less common but possible
Extended range: 5, 6, 12, 13, 16, 19, 20, 21, 26
```

## Discovery Results

After testing, you should get results like:

```
K2 button candidates: 27
K3 button candidates: 22

Suggested GPIO pin mappings:
K2_GPIO=27  # Key 2 button  
K3_GPIO=22  # Key 3 button
```

## Applying Your Discovery

1. **Edit hardware_boot_selector.sh**:
   ```bash
   nano hardware_boot_selector.sh
   
   # Update these lines at the top:
   K2_GPIO=YOUR_K2_PIN  # Replace with discovered pin
   K3_GPIO=YOUR_K3_PIN  # Replace with discovered pin
   ```

2. **Test the configuration**:
   ```bash
   # Test without installing
   sudo ./hardware_boot_selector.sh
   
   # Press K2 and K3 to verify detection works
   ```

3. **Deploy if working**:
   ```bash
   sudo ./install_boot_selector.sh
   ```

## Troubleshooting Discovery

### No GPIO Changes Detected

If no GPIO pins respond to button presses:

1. **Check permissions**:
   ```bash
   ls -la /sys/class/gpio/
   # Should be writable by root
   ```

2. **Verify button functionality**:
   - Test buttons in normal Norns operation
   - Ensure buttons are not stuck or damaged

3. **Try input device method**:
   ```bash
   # Monitor input devices instead
   sudo ./discover_gpio_pins.sh
   # Choose option 5: Check input devices
   ```

4. **Check for alternative input methods**:
   - Some Norns use I2C or SPI for buttons
   - May use input device drivers instead of direct GPIO

### Multiple Pins Respond

If multiple GPIO pins change for one button:

1. **Test each candidate individually**:
   ```bash
   # Test specific pin
   echo 27 > /sys/class/gpio/export
   watch -n 0.1 cat /sys/class/gpio/gpio27/value
   # Press button and observe
   ```

2. **Choose the most responsive pin**:
   - Pick the pin with clearest 0/1 transitions
   - Avoid pins that fluctuate or show noise

3. **Verify with longer tests**:
   ```bash
   # 30-second test for stability
   sudo ./discover_gpio_pins.sh test
   ```

### Permission Errors

```bash
# Ensure you're running as root
sudo ./discover_gpio_pins.sh

# Add user to gpio group (for future reference)
sudo usermod -a -G gpio we

# Check group membership
groups we
```

## Hardware-Specific Notes

### Norns Shield
- Based on Raspberry Pi GPIO
- May use standard Pi pin layouts
- Check Pi GPIO documentation

### Original Norns  
- Custom hardware design
- GPIO pins may be non-standard
- Check monome documentation

### DIY/Modified Norns
- GPIO pins depend on your specific build
- Check your schematic or documentation
- May require custom pin mappings

## Verification Before Deployment

**Always verify before installing**:

```bash
# 1. Discover pins
sudo ./discover_gpio_pins.sh quick

# 2. Update hardware_boot_selector.sh with discovered pins

# 3. Test manually
sudo ./hardware_boot_selector.sh

# 4. Verify button detection works

# 5. Only then install
sudo ./install_boot_selector.sh
```

## Alternative Approaches

If GPIO discovery fails, consider:

1. **Input device approach**: Use `/dev/input/event*` files
2. **Polling existing Norns code**: Hook into existing button handling
3. **systemd service**: Run selector after Norns starts
4. **Manual configuration**: Always use saved preference, control via SSH

## Getting Help

If you can't determine GPIO pins:

1. **Share discovery output**:
   ```bash
   sudo ./discover_gpio_pins.sh quick > gpio_discovery.log 2>&1
   # Share gpio_discovery.log
   ```

2. **Provide hardware info**:
   - Norns model and revision
   - Any hardware modifications
   - Output of `cat /proc/cpuinfo`

3. **Test with known working scripts**:
   - Use existing Norns scripts that read buttons
   - See how they access button state

Remember: It's better to spend time on discovery than to deploy with wrong GPIO pins!