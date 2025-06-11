# SimonSaysSeeq Startup Control System

This document explains how to control the startup behavior of your Norns device with several options:
1. **Normal Mode**: Boot to standard Norns menu, then manually select `SimonSaysSeeqNorns.lua`
2. **Direct Mode**: Boot directly into the `SimonSaysSeeqRust` application, bypassing the menu
3. **Boot-Time Interactive Menu**: Show a visual selection screen during startup
4. **Boot-Time Hardware Selection**: Use Norns buttons during boot to choose mode

## Quick Start

### Interactive Control
```bash
# SSH to your Norns
ssh we@norns.local

# Navigate to the SimonSaysSeeqRust directory
cd /home/we/dust/code/SimonSaysSeeqRust

# Run the interactive startup controller
./toggle_startup_mode.sh
```

### Command Line Control
```bash
# Set to normal Norns menu startup
./toggle_startup_mode.sh menu

# Set to direct Rust app startup  
./toggle_startup_mode.sh rust

# Check current status
./toggle_startup_mode.sh status

# Start/stop service manually
./toggle_startup_mode.sh start
./toggle_startup_mode.sh stop

# View logs
./toggle_startup_mode.sh logs

# Enable boot-time menu
./toggle_startup_mode.sh boot-menu

# Enable hardware selection
./toggle_startup_mode.sh hardware-select
```

## Startup Modes Explained

### Normal Mode (Default)
- **What happens**: Norns boots to the standard Lua-based menu system
- **How to run SimonSaysSeeq**: Navigate to `SELECT > SimonSaysSeeqNorns`
- **Best for**: 
  - Regular Norns usage with multiple scripts
  - Development and testing
  - When you want menu access to other scripts

### Direct Mode
- **What happens**: SimonSaysSeeqRust starts automatically on boot
- **Interface**: Rust application takes control immediately
- **Best for**:
  - Live performance setups
  - Dedicated SimonSaysSeeq installations
  - Minimal boot time requirements

### Boot-Time Interactive Menu
- **What happens**: Shows a visual menu on Norns screen during boot
- **Controls**: Use encoders to select, K3 to confirm
- **Timeout**: Auto-selects saved preference after 10 seconds
- **Best for**:
  - Flexible setups where mode changes frequently
  - Situations where you want visual confirmation
  - Users who prefer GUI selection

### Boot-Time Hardware Selection
- **What happens**: Hold K2 or K3 during boot to choose mode
- **Controls**: K2 = Rust app, K3 = Normal menu
- **Timeout**: Uses saved preference after 5 seconds if no button held
- **Best for**:
  - Quick selection without looking at screen
  - Performance situations with muscle memory
  - Minimal visual distraction during boot

## Technical Implementation

### systemd Service
The direct mode uses a systemd service (`simonsaysseeq-rust.service`) that:
- Starts automatically on boot when enabled
- Manages the Rust process lifecycle
- Provides logging and monitoring capabilities
- Can be controlled with standard systemd commands

### Configuration Storage
- Config file: `/home/we/.config/simonsaysseeq/startup_mode`
- Contains either `menu` or `rust`
- Created automatically by the toggle script

### Boot-Time Menu Integration
To enable boot-time selection, you need to integrate one of the boot selector scripts into your Norns startup process:

#### Option A: Interactive Visual Menu
```bash
# Copy the boot selector to your Norns script directory
cp boot_time_selector.lua /home/we/dust/code/SimonSaysSeeqBootMenu.lua

# Set it to run on startup (requires Norns integration)
# This would typically be added to a custom startup script
```

#### Option B: Hardware Button Selection
```bash
# Copy the hardware selector
cp hardware_boot_selector.lua /home/we/dust/code/SimonSaysSeeqHardwareSelect.lua

# Integrate into boot process (see Advanced Integration section)
```

### Service Management
```bash
# Manual systemd control (advanced users)
sudo systemctl enable simonsaysseeq-rust   # Auto-start on boot
sudo systemctl disable simonsaysseeq-rust  # Return to menu mode
sudo systemctl start simonsaysseeq-rust    # Start now
sudo systemctl stop simonsaysseeq-rust     # Stop now
sudo systemctl status simonsaysseeq-rust   # Check status
```

## Troubleshooting

### "Service not found" Error
If you get a service not found error:
1. Ensure you've deployed the Rust app: `./deploy_to_norns.sh`
2. Check if service file exists: `ls -la /etc/systemd/system/simonsaysseeq-rust.service`
3. If missing, re-run the deployment script

### Rust App Won't Start
1. Check the binary exists: `ls -la /home/we/dust/code/SimonSaysSeeqRust/simon_says_seeq`
2. Verify permissions: `chmod +x simon_says_seeq`
3. Check logs: `journalctl -u simonsaysseeq-rust -f`
4. Test manually: `./simon_says_seeq`

### Stuck in Wrong Mode
If your Norns boots into the wrong mode:
1. SSH into Norns: `ssh we@norns.local`
2. Run: `cd /home/we/dust/code/SimonSaysSeeqRust && ./toggle_startup_mode.sh menu`
3. Reboot: `sudo reboot`

### Emergency Reset
To force normal menu mode:
```bash
# Disable the service
sudo systemctl disable simonsaysseeq-rust
sudo systemctl stop simonsaysseeq-rust

# Remove config file
rm -f /home/we/.config/simonsaysseeq/startup_mode

# Reboot
sudo reboot
```

## File Locations

### Scripts and Binaries
- Toggle script: `/home/we/dust/code/SimonSaysSeeqRust/toggle_startup_mode.sh`
- Rust binary: `/home/we/dust/code/SimonSaysSeeqRust/simon_says_seeq`
- Lua script: `/home/we/dust/code/SimonSaysSeeqNorns/SimonSaysSeeqNorns.lua`

### Configuration and Logs
- Config file: `/home/we/.config/simonsaysseeq/startup_mode`
- Service logs: `journalctl -u simonsaysseeq-rust`
- Boot logs: `/tmp/simonsaysseeq_boot.log`
- Runtime logs: `/tmp/simonsaysseeq_rust.log`

### System Files
- Service definition: `/etc/systemd/system/simonsaysseeq-rust.service`
- Boot selector (basic): `/home/we/dust/code/SimonSaysSeeqRust/boot_selector.lua`
- Interactive menu: `/home/we/dust/code/SimonSaysSeeqRust/boot_time_selector.lua`
- Hardware selector: `/home/we/dust/code/SimonSaysSeeqRust/hardware_boot_selector.lua`

## Advanced Usage

### Boot-Time Menu Integration

#### Method 1: Replace Norns Startup Script
```bash
# Backup original startup
sudo cp /home/we/norns/lua/core/startup.lua /home/we/norns/lua/core/startup.lua.backup

# Modify startup.lua to include boot selector
# Add this line before other script loading:
# dofile("/home/we/dust/code/SimonSaysSeeqRust/boot_time_selector.lua")
```

#### Method 2: Custom Init Script
```bash
# Create a custom init script that runs the boot selector
cat > /home/we/dust/code/SimonSaysSeeqInit.lua << 'EOF'
-- SimonSaysSeeq Custom Init
local function run_boot_selector()
    dofile("/home/we/dust/code/SimonSaysSeeqRust/boot_time_selector.lua")
end

-- Run selector on startup
run_boot_selector()
EOF

# Then manually select this script as your startup script
```

#### Method 3: systemd Integration
```bash
# Create a systemd service that runs before Norns
sudo tee /etc/systemd/system/simonsaysseeq-boot-selector.service << 'EOF'
[Unit]
Description=SimonSaysSeeq Boot Selector
Before=norns.service
After=multi-user.target

[Service]
Type=oneshot
User=we
ExecStart=/usr/bin/lua /home/we/dust/code/SimonSaysSeeqRust/hardware_boot_selector.lua
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

# Enable the service
sudo systemctl enable simonsaysseeq-boot-selector.service
```

### Automatic Mode Detection
You can create scripts that automatically switch modes based on conditions:

```bash
#!/bin/bash
# Example: Switch to direct mode if Grid is connected
if lsusb | grep -q "monome"; then
    ./toggle_startup_mode.sh rust
else
    ./toggle_startup_mode.sh menu
fi
```

### Integration with Other Scripts
The startup control system can be integrated with other Norns management scripts by checking the current mode:

```bash
current_mode=$(cat /home/we/.config/simonsaysseeq/startup_mode 2>/dev/null || echo "menu")
if [ "$current_mode" = "rust" ]; then
    echo "SimonSaysSeeq is in direct boot mode"
fi
```

### Performance Considerations
- **Direct mode**: Faster boot, immediate access to sequencer
- **Menu mode**: Slower boot, but access to all Norns functionality
- **Service overhead**: Minimal (< 1% CPU when idle)

## Development Workflow

When developing, you might want to frequently switch modes:

```bash
# Development cycle
./toggle_startup_mode.sh menu    # Switch to menu for testing
sudo reboot                      # Test boot process
# ... test in menu mode ...
./toggle_startup_mode.sh rust    # Switch to direct mode
sudo reboot                      # Test auto-start
# ... test direct boot ...
```

## Boot-Time Menu Controls

### Interactive Menu Controls
- **Encoder 2/3**: Navigate between options
- **Key 3**: Select highlighted option
- **Key 1**: Cancel (defaults to normal menu)
- **Timeout**: Auto-selects after 10 seconds

### Hardware Selection Controls
- **Hold K2 during boot**: Select Rust app mode
- **Hold K3 during boot**: Select normal menu mode
- **No buttons**: Uses saved preference after 5 seconds
- **Both buttons**: Defaults to menu mode

### Menu Options Explained
1. **Normal Norns Menu**: Standard Norns behavior
2. **Direct Rust App**: Immediate launch of SimonSaysSeeqRust
3. **Use Saved Setting**: Continue with previously configured mode

## Safety Features

- **Fallback**: If Rust app fails to start, system falls back to menu mode
- **Override**: Emergency override possible via SSH
- **Logging**: All operations are logged for debugging
- **Validation**: Config files are validated before use
- **Skip mechanism**: Boot menus can be bypassed if needed
- **Button debouncing**: Hardware selection includes proper input handling

## Support

If you encounter issues:
1. Check the logs: `./toggle_startup_mode.sh logs`
2. Verify status: `./toggle_startup_mode.sh status`
3. Try manual control: `./toggle_startup_mode.sh menu`
4. Reboot and test: `sudo reboot`

### Boot Menu Troubleshooting

#### Menu Not Appearing
1. Check if boot selector is properly integrated
2. Verify file permissions: `chmod +x boot_time_selector.lua`
3. Check for syntax errors: `lua -c boot_time_selector.lua`
4. Look for boot logs: `/tmp/simonsaysseeq_boot.log`

#### Hardware Selection Not Working
1. Test button functionality with other Norns scripts
2. Check timing - buttons must be held during boot, not just pressed
3. Verify hardware_boot_selector.lua is executable
4. Check for conflicts with other boot scripts

#### Menu Appears But Selection Doesn't Work
1. Verify systemd service is properly installed
2. Check file permissions on Rust binary
3. Test selection manually: `./toggle_startup_mode.sh rust`
4. Check logs during boot process

For persistent issues, collect the following information:
- Output of `./toggle_startup_mode.sh status`
- Contents of `journalctl -u simonsaysseeq-rust --since "1 hour ago"`
- Contents of `/tmp/simonsaysseeq_boot.log`
- Boot selector logs from `/tmp/`
- Description of which boot method you're trying to use
