#!/bin/bash
# Minimal CO2 startup script for SimonSaysSeeq on Raspberry Pi 5

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CO2_DATA_DIR="$PROJECT_DIR/co2_data"

# Create CO2 data directory
mkdir -p "$CO2_DATA_DIR"

# Wait for network connectivity before downloading CO2 data
echo "Checking network connectivity..."
MAX_WAIT=300  # 5 minutes maximum wait
WAIT_TIME=0
SLEEP_INTERVAL=10

while [ $WAIT_TIME -lt $MAX_WAIT ]; do
    if ping -c 1 gml.noaa.gov >/dev/null 2>&1; then
        echo "Network connectivity confirmed to gml.noaa.gov"
        break
    else
        echo "Waiting for network connectivity... (${WAIT_TIME}s/${MAX_WAIT}s)"
        sleep $SLEEP_INTERVAL
        WAIT_TIME=$((WAIT_TIME + SLEEP_INTERVAL))
    fi
done

if [ $WAIT_TIME -ge $MAX_WAIT ]; then
    echo "Network connectivity timeout after ${MAX_WAIT} seconds"
    echo "Will attempt CO2 download anyway..."
fi

# Download CO2 data at startup
echo "Downloading latest CO2 data..."
cd "$PROJECT_DIR"
python3 scripts/download_keeling_curve_CO2_from_MLO.py "$CO2_DATA_DIR/"

echo "CO2 data downloaded to: $CO2_DATA_DIR"
echo "Setup complete!"