#!/usr/bin/env python3
"""
Visual Grid Protocol Detector

This script uses a webcam to visually detect LED changes on monome grids
to automatically discover communication protocols. Designed for development
machine testing with existing grids connected via serialosc.

Usage:
    python3 visual_grid_detector.py --grid-device /dev/ttyACM0 --camera-id 0

Requirements:
    pip install opencv-python numpy pyserial
"""

import cv2
import numpy as np
import serial
import time
import argparse
import json
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
import threading
import queue

@dataclass
class LEDPosition:
    """Represents a single LED position on the grid"""
    grid_x: int
    grid_y: int
    pixel_x: int
    pixel_y: int
    roi_coords: Tuple[int, int, int, int]  # x1, y1, x2, y2

@dataclass 
class GridCalibration:
    """Grid calibration data"""
    led_positions: Dict[Tuple[int, int], LEDPosition]
    grid_bounds: Tuple[int, int, int, int]  # x1, y1, x2, y2
    baseline_brightness: Dict[Tuple[int, int], float]

class VisualGridDetector:
    def __init__(self, camera_id: int = 0, grid_device: str = "/dev/ttyACM0"):
        self.camera_id = camera_id
        self.grid_device = grid_device
        self.camera = None
        self.serial_port = None
        self.calibration: Optional[GridCalibration] = None
        self.running = False
        
        # Detection parameters
        self.roi_size = 20  # Size of region of interest around each LED
        self.brightness_threshold = 15  # Minimum brightness change to detect
        self.detection_delay = 0.2  # Seconds to wait after command
        
    def initialize_camera(self) -> bool:
        """Initialize camera connection"""
        try:
            self.camera = cv2.VideoCapture(self.camera_id)
            if not self.camera.isOpened():
                print(f"❌ Failed to open camera {self.camera_id}")
                return False
                
            # Set camera properties for better detection
            self.camera.set(cv2.CAP_PROP_FRAME_WIDTH, 1280)
            self.camera.set(cv2.CAP_PROP_FRAME_HEIGHT, 720)
            self.camera.set(cv2.CAP_PROP_FPS, 30)
            
            # Test capture
            ret, frame = self.camera.read()
            if not ret:
                print("❌ Failed to capture test frame")
                return False
                
            print(f"✅ Camera initialized: {frame.shape[1]}x{frame.shape[0]}")
            return True
            
        except Exception as e:
            print(f"❌ Camera initialization error: {e}")
            return False
    
    def initialize_serial(self) -> bool:
        """Initialize serial connection to grid"""
        try:
            self.serial_port = serial.Serial(
                port=self.grid_device,
                baudrate=115200,
                timeout=0.5
            )
            print(f"✅ Serial connection established: {self.grid_device}")
            return True
            
        except Exception as e:
            print(f"❌ Serial connection failed: {e}")
            print("   Note: This detector works with direct serial access.")
            print("   For OSC-based testing, you'd need to modify for OSC commands.")
            return False
    
    def capture_frame(self) -> Optional[np.ndarray]:
        """Capture a single frame from camera"""
        if not self.camera:
            return None
            
        ret, frame = self.camera.read()
        if not ret:
            return None
            
        # Convert to grayscale for brightness analysis
        gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
        return gray
    
    def send_serial_command(self, command_bytes: bytes, delay: float = None) -> bool:
        """Send command to grid via serial"""
        if not self.serial_port:
            return False
            
        try:
            self.serial_port.write(command_bytes)
            self.serial_port.flush()
            
            # Wait for command to take effect
            wait_time = delay if delay else self.detection_delay
            time.sleep(wait_time)
            return True
            
        except Exception as e:
            print(f"❌ Serial command failed: {e}")
            return False
    
    def get_brightness_at_position(self, frame: np.ndarray, led_pos: LEDPosition) -> float:
        """Get average brightness in LED region of interest"""
        x1, y1, x2, y2 = led_pos.roi_coords
        
        # Ensure coordinates are within frame bounds
        h, w = frame.shape
        x1, y1 = max(0, x1), max(0, y1)
        x2, y2 = min(w, x2), min(h, y2)
        
        if x2 <= x1 or y2 <= y1:
            return 0.0
            
        roi = frame[y1:y2, x1:x2]
        return np.mean(roi)
    
    def detect_led_changes(self, before_frame: np.ndarray, after_frame: np.ndarray) -> Dict[Tuple[int, int], float]:
        """Detect which LEDs changed between two frames"""
        if not self.calibration:
            return {}
            
        changes = {}
        for grid_pos, led_pos in self.calibration.led_positions.items():
            before_brightness = self.get_brightness_at_position(before_frame, led_pos)
            after_brightness = self.get_brightness_at_position(after_frame, led_pos)
            
            change = after_brightness - before_brightness
            if abs(change) > self.brightness_threshold:
                changes[grid_pos] = change
                
        return changes
    
    def interactive_calibration(self) -> bool:
        """Interactive calibration to map LED positions"""
        print("\n🎯 Starting interactive grid calibration...")
        print("   This will help map camera pixels to grid LED positions")
        print("   Watch your grid and follow the prompts")
        
        if not self.capture_frame():
            print("❌ Cannot capture frames for calibration")
            return False
            
        led_positions = {}
        
        # Test a few key positions for initial calibration
        test_positions = [
            (0, 0), (3, 0), (7, 0), (15, 0),  # Top row
            (0, 3), (15, 3),                   # Middle
            (0, 7), (3, 7), (7, 7), (15, 7)   # Bottom row
        ]
        
        print(f"\n📍 Will test {len(test_positions)} LED positions for calibration")
        
        for grid_x, grid_y in test_positions:
            print(f"\n🔸 Testing LED at grid position ({grid_x}, {grid_y})")
            
            # Capture before frame
            before_frame = self.capture_frame()
            if before_frame is None:
                continue
                
            # Send command to light up this LED
            # Try multiple command formats
            commands_to_try = [
                bytes([0x10, grid_x, grid_y, 15]),  # Modern format
                bytes([0x11, grid_x, grid_y, 15]),  # Alt format 1
                bytes([0x14, grid_x, grid_y, 15]),  # Alt format 2
            ]
            
            led_detected = False
            for cmd in commands_to_try:
                if not self.send_serial_command(cmd, 0.3):
                    continue
                    
                # Capture after frame
                after_frame = self.capture_frame()
                if after_frame is None:
                    continue
                    
                # Find brightest change
                diff = cv2.absdiff(after_frame, before_frame)
                
                # Find brightest spot
                min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(diff)
                
                if max_val > self.brightness_threshold:
                    # Found LED change
                    pixel_x, pixel_y = max_loc
                    
                    # Create ROI around detected position
                    half_roi = self.roi_size // 2
                    roi_coords = (
                        pixel_x - half_roi,
                        pixel_y - half_roi, 
                        pixel_x + half_roi,
                        pixel_y + half_roi
                    )
                    
                    led_pos = LEDPosition(
                        grid_x=grid_x,
                        grid_y=grid_y,
                        pixel_x=pixel_x,
                        pixel_y=pixel_y,
                        roi_coords=roi_coords
                    )
                    
                    led_positions[(grid_x, grid_y)] = led_pos
                    print(f"   ✅ LED detected at pixel ({pixel_x}, {pixel_y})")
                    led_detected = True
                    break
                    
                # Try clearing the LED
                clear_cmd = bytes([cmd[0], grid_x, grid_y, 0])
                self.send_serial_command(clear_cmd, 0.1)
            
            if not led_detected:
                print(f"   ❌ Could not detect LED at ({grid_x}, {grid_y})")
        
        if len(led_positions) < 4:
            print(f"❌ Calibration failed - only detected {len(led_positions)} LEDs")
            print("   Check camera positioning and LED commands")
            return False
            
        # Calculate baseline brightness
        baseline_frame = self.capture_frame()
        baseline_brightness = {}
        
        if baseline_frame is not None:
            for grid_pos, led_pos in led_positions.items():
                brightness = self.get_brightness_at_position(baseline_frame, led_pos)
                baseline_brightness[grid_pos] = brightness
        
        # Estimate grid bounds
        pixels_x = [pos.pixel_x for pos in led_positions.values()]
        pixels_y = [pos.pixel_y for pos in led_positions.values()]
        
        grid_bounds = (
            min(pixels_x) - self.roi_size,
            min(pixels_y) - self.roi_size,
            max(pixels_x) + self.roi_size,
            max(pixels_y) + self.roi_size
        )
        
        self.calibration = GridCalibration(
            led_positions=led_positions,
            grid_bounds=grid_bounds,
            baseline_brightness=baseline_brightness
        )
        
        print(f"✅ Calibration complete! Detected {len(led_positions)} LED positions")
        return True
    
    def test_protocol_commands(self) -> List[Tuple[bytes, str, Dict]]:
        """Test various protocol commands and detect responses"""
        if not self.calibration:
            print("❌ No calibration data - run calibration first")
            return []
            
        print("\n🧪 Testing protocol commands...")
        
        # Commands to test
        test_commands = [
            # Clear commands
            (bytes([0x12, 0x00]), "Modern: Clear all LEDs"),
            (bytes([0x13, 0x00]), "Alt: Clear all LEDs"),
            (bytes([0x14, 0x00]), "Legacy: Clear all LEDs"),
            
            # Single LED commands  
            (bytes([0x10, 0, 0, 15]), "Modern: Light LED (0,0)"),
            (bytes([0x11, 0, 0, 15]), "Alt: Light LED (0,0)"),
            (bytes([0x14, 0, 0, 15]), "Legacy: Light LED (0,0)"),
            
            # Pattern commands
            (bytes([0x10, 0, 0, 8]), "Modern: Dim LED (0,0)"),
            (bytes([0x10, 1, 0, 8]), "Modern: Dim LED (1,0)"),
        ]
        
        successful_commands = []
        
        for cmd_bytes, cmd_name in test_commands:
            print(f"\n🔍 Testing: {cmd_name}")
            print(f"   Command: {' '.join(f'{b:02x}' for b in cmd_bytes)}")
            
            # Capture before state
            before_frame = self.capture_frame()
            if before_frame is None:
                continue
                
            # Send command
            if not self.send_serial_command(cmd_bytes):
                continue
                
            # Capture after state
            after_frame = self.capture_frame()
            if after_frame is None:
                continue
                
            # Detect changes
            changes = self.detect_led_changes(before_frame, after_frame)
            
            if changes:
                print(f"   ✅ Changes detected: {len(changes)} LEDs")
                for pos, change in changes.items():
                    direction = "brighter" if change > 0 else "dimmer"
                    print(f"      LED ({pos[0]},{pos[1]}): {abs(change):.1f} {direction}")
                    
                successful_commands.append((cmd_bytes, cmd_name, changes))
            else:
                print(f"   ❌ No changes detected")
        
        return successful_commands
    
    def save_calibration(self, filename: str = "grid_calibration.json") -> bool:
        """Save calibration data to file"""
        if not self.calibration:
            return False
            
        try:
            # Convert to serializable format
            data = {
                'led_positions': {
                    f"{pos[0]},{pos[1]}": {
                        'grid_x': led.grid_x,
                        'grid_y': led.grid_y,
                        'pixel_x': led.pixel_x,
                        'pixel_y': led.pixel_y,
                        'roi_coords': led.roi_coords
                    }
                    for pos, led in self.calibration.led_positions.items()
                },
                'grid_bounds': self.calibration.grid_bounds,
                'baseline_brightness': {
                    f"{pos[0]},{pos[1]}": brightness
                    for pos, brightness in self.calibration.baseline_brightness.items()
                }
            }
            
            with open(filename, 'w') as f:
                json.dump(data, f, indent=2)
                
            print(f"✅ Calibration saved to {filename}")
            return True
            
        except Exception as e:
            print(f"❌ Failed to save calibration: {e}")
            return False
    
    def cleanup(self):
        """Clean up resources"""
        if self.camera:
            self.camera.release()
        if self.serial_port:
            self.serial_port.close()
        cv2.destroyAllWindows()

def main():
    parser = argparse.ArgumentParser(description="Visual Grid Protocol Detector")
    parser.add_argument("--camera-id", type=int, default=0, help="Camera device ID")
    parser.add_argument("--grid-device", type=str, default="/dev/ttyACM0", help="Grid serial device")
    parser.add_argument("--calibrate-only", action="store_true", help="Only run calibration")
    parser.add_argument("--test-only", action="store_true", help="Only run protocol tests (requires existing calibration)")
    
    args = parser.parse_args()
    
    print("🔬 Visual Grid Protocol Detector")
    print("=" * 50)
    print(f"Camera: {args.camera_id}")
    print(f"Grid device: {args.grid_device}")
    print()
    
    detector = VisualGridDetector(args.camera_id, args.grid_device)
    
    try:
        # Initialize hardware
        if not detector.initialize_camera():
            return 1
            
        if not detector.initialize_serial():
            return 1
        
        # Run calibration
        if not args.test_only:
            if not detector.interactive_calibration():
                return 1
            detector.save_calibration()
        
        # Run protocol tests
        if not args.calibrate_only:
            successful_commands = detector.test_protocol_commands()
            
            print(f"\n📊 Protocol Detection Results:")
            print("=" * 40)
            
            if successful_commands:
                print(f"✅ Found {len(successful_commands)} working commands:")
                for cmd_bytes, cmd_name, changes in successful_commands:
                    print(f"   • {cmd_name}")
                    print(f"     Command: {' '.join(f'{b:02x}' for b in cmd_bytes)}")
                    print(f"     Effects: {len(changes)} LED changes")
            else:
                print("❌ No working commands detected")
                print("   Try adjusting camera position or detection parameters")
        
        print(f"\n🎯 Detection complete!")
        
    except KeyboardInterrupt:
        print(f"\n\n⏸️  Interrupted by user")
    except Exception as e:
        print(f"\n💥 Error: {e}")
        return 1
    finally:
        detector.cleanup()
    
    return 0

if __name__ == "__main__":
    exit(main())