#!/bin/bash
# Minimal CO2 startup script for SimonSaysSeeq on Raspberry Pi 5

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CO2_DATA_DIR="$PROJECT_DIR/co2_data"

# Create CO2 data directory
mkdir -p "$CO2_DATA_DIR"

# Download CO2 data at startup
echo "Downloading latest CO2 data..."
cd "$PROJECT_DIR"
python3 scripts/download_keeling_curve_CO2_from_MLO.py "$CO2_DATA_DIR/"

echo "CO2 data downloaded to: $CO2_DATA_DIR"
echo "Setup complete!"