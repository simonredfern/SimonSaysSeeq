# Troubleshooting SimonSaysSeeq Startup Issues

This guide helps you diagnose why the SimonSaysSeeq sequencer service may not be starting properly on your Raspberry Pi 5.

## Overview of Logging

The systemd service logs to **multiple locations** during startup:

### 1. Wrapper Script Log
**Location**: `/tmp/simonsaysseeq_crow_boot_fix.log`

This log captures the Crow CV boot fix sequence (start-stop-start). It's the first place to check if the service is having initialization problems.

### 2. Systemd Journal
**Location**: `journalctl -u simonsaysseeq-rpi`

The main application logs are sent to the systemd journal. However, the service is configured with `RUST_LOG=error` by default, which **suppresses info and debug logs**.

### 3. AI Startup Log
**Location**: `ai_startup.log` (in the project directory)

Contains version info, feature flags, and startup timestamp. Written even when RUST_LOG=error.

### 4. Formal State Log
**Location**: Check application working directory

Contains detailed state transitions during operation.

## Quick Diagnostic Commands

Run these commands in order to diagnose startup issues:

```bash
# 1. Check if service is running
sudo systemctl status simonsaysseeq-rpi

# 2. Check wrapper script log (Crow CV boot fix)
cat /tmp/simonsaysseeq_crow_boot_fix.log

# 3. Check recent journal entries (last 100 lines)
sudo journalctl -u simonsaysseeq-rpi -n 100 --no-pager

# 4. Check ALL journal entries with full output
sudo journalctl -u simonsaysseeq-rpi --no-pager -l

# 5. Check if the binary exists and is executable
ls -la target/release/simon_says_seeq

# 6. Check AI startup log
cat ai_startup.log

# 7. Verify serialosc is running (required dependency)
sudo systemctl status serialosc

# 8. Check for udev device errors
dmesg | tail -50
```

## Common Issues and Solutions

### Issue 1: Service Shows as "Active" but Sequencer Not Running

**Symptoms:**
- `systemctl status` shows "active (running)"
- No audio/MIDI output
- Grids not responding

**Check:**
```bash
# See if process is actually running
ps aux | grep simon_says_seeq

# Check for crashes in journal
sudo journalctl -u simonsaysseeq-rpi -n 200 | grep -i "error\|panic\|crash\|failed"
```

**Possible causes:**
- Grid connection issues (see Issue 3)
- MIDI device not found
- Audio device initialization failure

### Issue 2: Service Fails to Start (Dead/Failed)

**Symptoms:**
- `systemctl status` shows "failed" or "inactive (dead)"
- Service won't stay running

**Check:**
```bash
# Get detailed failure info
sudo journalctl -u simonsaysseeq-rpi -xe --no-pager

# Check if binary exists
test -f target/release/simon_says_seeq && echo "Binary exists" || echo "Binary missing!"

# Check binary permissions
ls -l target/release/simon_says_seeq

# Try running binary manually to see errors
./target/release/simon_says_seeq
```

**Possible causes:**
- Binary not built (`./rpi5_build_and_run.sh --build`)
- Wrong binary path in service config
- Missing permissions (user not in audio/gpio groups)

### Issue 3: Grid Not Detected

**Symptoms:**
- Service starts but grids don't light up
- "Waiting for grids" messages in logs

**Check:**
```bash
# Check serialosc is running
sudo systemctl status serialosc

# List connected serial devices
serialosc-detector

# Check USB connections
lsusb | grep -i monome

# Check udev rules
ls -la /etc/udev/rules.d/ | grep -i serialosc
```

**Fix:**
```bash
# Restart serialosc
sudo systemctl restart serialosc

# Wait a moment then restart SimonSaysSeeq
sleep 3
sudo systemctl restart simonsaysseeq-rpi

# Check if grids are now detected
sudo journalctl -u simonsaysseeq-rpi -n 50 --no-pager | grep -i grid
```

### Issue 4: Crow CV Outputs Not Working

**Symptoms:**
- Sequencer runs but Crow CV outputs produce no voltage
- MIDI works but CV doesn't

**Explanation:**
The wrapper script implements a start-stop-start sequence specifically to fix this. Check if the fix is working:

```bash
# Check wrapper log
cat /tmp/simonsaysseeq_crow_boot_fix.log

# Should show three steps:
# Step 1: Starting service for initial hardware setup...
# Step 2: Stopping service (key part of Crow CV fix)...
# Step 3: Final start - Crow CV should now work...
```

**If wrapper isn't working:**
```bash
# Check if wrapper exists and is executable
ls -la /usr/local/bin/simonsaysseeq_wrapper.sh

# Reinstall service
./rpi5_build_and_run.sh service install
```

### Issue 5: No Logs in Journal (RUST_LOG=error Too Restrictive)

**Symptoms:**
- Very few log entries
- Can't see startup sequence
- Hard to debug what's happening

**Temporary Fix - Enable Verbose Logging:**

Edit the service file to show more logs:

```bash
# Edit service file
sudo nano /etc/systemd/system/simonsaysseeq-rpi.service

# Change this line:
#   Environment=RUST_LOG=error
# To:
#   Environment=RUST_LOG=info

# Or for maximum debugging:
#   Environment=RUST_LOG=debug

# Reload and restart
sudo systemctl daemon-reload
sudo systemctl restart simonsaysseeq-rpi

# Now check logs again
sudo journalctl -u simonsaysseeq-rpi -f
```

**Note:** Remember to change back to `RUST_LOG=error` after debugging to reduce log spam.

## Expected Startup Log Sequence

When working correctly, you should see logs similar to:

```
════════════════════════════════════════════════════════
🎵 SimonSaysSeeq Rust vX.X.X
📦 Cargo Version: vX.X.X (git_hash)
📅 Startup Time: 2024-XX-XX HH:MM:SS UTC
🔧 Build Profile: release
⚙️  Features: MIDI=true, Hardware=true
⏰ Timing Mode: EXTERNAL CLOCK SLAVE ONLY (basic sync)
════════════════════════════════════════════════════════
📝 Formal state logger initialized
[Hardware initialization messages...]
[Grid detection messages...]
[MIDI device detection messages...]
```

## Manual Start for Testing

If the service won't start, try running manually to see full output:

```bash
# Stop the service first
sudo systemctl stop simonsaysseeq-rpi

# Run manually with full logging
cd /path/to/SimonSaysSeeqNornsRust
RUST_LOG=debug ./target/release/simon_says_seeq

# Watch for:
# - Hardware initialization errors
# - Grid connection errors  
# - MIDI device errors
# - Config file errors
# - Permission errors

# Press Ctrl+C to stop
```

## Service Control Commands

```bash
# Start the service
sudo systemctl start simonsaysseeq-rpi

# Stop the service
sudo systemctl stop simonsaysseeq-rpi

# Restart the service
sudo systemctl restart simonsaysseeq-rpi

# Check status
sudo systemctl status simonsaysseeq-rpi

# Enable autostart on boot
sudo systemctl enable simonsaysseeq-rpi

# Disable autostart
sudo systemctl disable simonsaysseeq-rpi

# View live logs (follow mode)
sudo journalctl -u simonsaysseeq-rpi -f
```

## Rebuilding and Reinstalling Service

If all else fails, rebuild and reinstall:

```bash
# Full clean rebuild and service reinstall
./rpi5_build_and_run.sh --build
./rpi5_build_and_run.sh service install

# Start the service
sudo systemctl start simonsaysseeq-rpi

# Check status
sudo systemctl status simonsaysseeq-rpi
```

## Getting Help

When asking for help, please provide:

1. Output of `sudo systemctl status simonsaysseeq-rpi`
2. Output of `sudo journalctl -u simonsaysseeq-rpi -n 200 --no-pager`
3. Contents of `/tmp/simonsaysseeq_crow_boot_fix.log`
4. Output of `lsusb` (to show connected USB devices)
5. Output of `sudo systemctl status serialosc`

## Advanced Debugging

### Enable Core Dumps

```bash
# Allow core dumps
ulimit -c unlimited

# Set core dump pattern
echo '/tmp/core.%e.%p' | sudo tee /proc/sys/kernel/core_pattern

# Run manually to capture crashes
./target/release/simon_says_seeq
```

### Check System Resources

```bash
# Check CPU/memory usage
top

# Check disk space
df -h

# Check for out-of-memory kills
dmesg | grep -i "killed process"

# Check system temperature (thermal throttling)
vcgencmd measure_temp
```

### Verify Dependencies

```bash
# Check required system libraries
ldd target/release/simon_says_seeq

# Verify user groups
groups $USER

# Should include: audio gpio i2c spi dialout plugdev
```

## Configuration Files

- Service file: `/etc/systemd/system/simonsaysseeq-rpi.service`
- Wrapper script: `/usr/local/bin/simonsaysseeq_wrapper.sh`
- App config: `config.toml` (in project directory)
- Serialosc service: `/etc/systemd/system/serialosc.service`

---

**Last Updated:** 2024
**For:** SimonSaysSeeq Raspberry Pi 5 Installation