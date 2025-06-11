# SimonSaysSeeq Hardware Boot Selector

Simple shell script that lets you choose your Norns startup mode by holding buttons during boot.

## Quick Setup

```bash
# SSH to your Norns
ssh we@norns.local
cd /home/we/dust/code/SimonSaysSeeqRust

# Install the boot selector
sudo ./install_boot_selector.sh
```

## How to Use

During Norns startup (within 5 seconds):

- **Hold K2**: Boot directly to SimonSaysSeeqRust app
- **Hold K3**: Boot to normal Norns menu
- **No buttons**: Use your previously saved preference

## What It Does

- **Lightweight**: Pure shell script, minimal dependencies
- **Fast**: <1MB memory, adds exactly 5 seconds to boot time
- **Reliable**: Direct GPIO access, no complex libraries
- **Simple**: Easy to understand and modify

## Files

- `hardware_boot_selector.sh` - Main boot selector script
- `install_boot_selector.sh` - One-command installer
- `toggle_startup_mode.sh` - Manual control via SSH

## Manual Control

```bash
# Check current status
./toggle_startup_mode.sh status

# Set modes manually
./toggle_startup_mode.sh rust    # Direct Rust app
./toggle_startup_mode.sh menu    # Normal menu

# Start/stop service
./toggle_startup_mode.sh start
./toggle_startup_mode.sh stop
```

## Troubleshooting

```bash
# Check service status
systemctl status simonsaysseeq-boot-selector

# View logs
tail -f /tmp/simonsaysseeq_boot_selector.log
journalctl -u simonsaysseeq-boot-selector -f

# Test without installing
sudo ./install_boot_selector.sh --test
```

## Remove

```bash
# Uninstall completely
sudo ./install_boot_selector.sh --uninstall
```

## Requirements

- Standard Norns device (any generation)
- bash and bc (standard Linux tools)
- GPIO pins 27 (K2) and 22 (K3)

For detailed documentation, see `HARDWARE_BOOT_SELECTOR.md`.