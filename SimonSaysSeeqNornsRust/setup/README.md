# Setup Instructions

This folder contains setup scripts for SimonSaysSeeq on Raspberry Pi 5.

## CO2 Data Setup

### Manual Run
To download CO2 data once:
```bash
./setup_co2_startup.sh
```

### Automatic Startup
To download fresh CO2 data every time the Pi boots:

1. Edit your crontab:
```bash
crontab -e
```

2. Add this line (replace with your actual path):
```
@reboot /home/pi/path/to/SimonSaysSeeqNornsRust/setup/setup_co2_startup.sh
```

3. Save and exit

The script will create a `co2_data` folder and download the latest Keeling Curve CO2 data that the Rust sequencer needs.

### Verify Crontab is Active
To check if your crontab is still working:
```bash
# Check if crontab exists
crontab -l

# Check cron service
systemctl status cron

# Check last reboot time
uptime -s

# Check if CO2 data is recent
ls -la ~/path/to/SimonSaysSeeqNornsRust/co2_data/
```