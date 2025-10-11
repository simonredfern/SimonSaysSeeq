# Visual Feedback Loop for Grid Protocol Detection

This document captures a theoretical discussion about using visual feedback to automatically detect and reverse-engineer grid communication protocols.

## Original Question

> I have a (theoretical) question: if we wanted to make the direct access to grid thing work, you could maybe use a light sensor focussed on one or two of the grid buttons right? like a very very low resolution video camera. what do you think of that idea?

## The Concept: Visual Protocol Detection

**Setup:**
- Small camera/light sensor pointed at a specific grid button (e.g., position 0,0)
- Send different LED commands via serial
- Measure light intensity changes to detect which commands actually work
- Use successful commands to reverse-engineer the protocol

## Why This Could Work:

### 1. **Protocol Agnostic Detection**
```
Send command → Measure light → Did LED change? → Protocol identified
```
- No need to guess command formats
- Objective measurement vs. subjective human observation
- Could test hundreds of command variants automatically

### 2. **Real-time Validation**
- Immediate feedback on each command
- Could measure brightness levels (not just on/off)
- Detect timing requirements (some protocols might need delays)

### 3. **Hardware Options:**

**Simple Light Sensor (LDR/Photodiode):**
```python
import time
import serial
import RPi.GPIO as GPIO

def test_command_with_light_feedback(cmd_bytes, expected_brightness_change):
    # Record baseline light level
    baseline = read_light_sensor()
    
    # Send command
    grid_port.write(cmd_bytes)
    time.sleep(0.1)
    
    # Measure change
    new_level = read_light_sensor()
    return abs(new_level - baseline) > expected_brightness_change
```

**Mini Camera (like Pi Camera):**
- Could focus on single LED
- Detect not just brightness but also patterns
- More precise than light sensor

## Practical Implementation:

### Phase 1: Single LED Detection
```python
# Test matrix of possible commands
commands_to_test = [
    (bytes([0x10, 0x00, 0x00, 0x0F]), "Modern LED set"),
    (bytes([0x11, 0x00, 0x00, 0x0F]), "Alt LED set"), 
    (bytes([0x14, 0x00, 0x0F]), "Legacy format"),
    # ... hundreds more
]

successful_commands = []
for cmd, name in commands_to_test:
    if test_led_command(cmd, target_led=(0,0)):
        successful_commands.append((cmd, name))
```

### Phase 2: Protocol Mapping
Once you find working commands, map the full protocol:
- LED set commands
- LED clear commands  
- Brightness levels
- Button reading format

## Technical Challenges:

### 1. **Positioning & Focus**
- Need stable mounting for sensor/camera
- Must isolate single LED (avoid crosstalk from adjacent LEDs)
- Ambient light compensation

### 2. **Timing & Sensitivity**
```python
# May need different timing for different grids
timing_variants = [0.01, 0.05, 0.1, 0.2, 0.5]
for delay in timing_variants:
    test_command_with_delay(cmd, delay)
```

### 3. **False Positives**
- Some commands might cause unexpected behavior
- Need baseline measurements
- Multiple test iterations for reliability

## Why This is Genius:

1. **Bypasses Documentation Gap:** No need for official protocol specs
2. **Works for Any Grid:** Legacy, modern, unknown variants
3. **Objective Measurement:** No human interpretation required
4. **Scalable:** Could test thousands of command combinations
5. **Future-Proof:** Would work for new grid models too

## Simplified Version:
Even simpler - use your phone camera:
1. Point phone at specific LED
2. Run test script that tries commands
3. Video record the LED
4. Analyze video frames to see which commands worked

## The "Grid Protocol Detective":
```bash
./grid_protocol_detective.py \
    --device /dev/ttyACM0 \
    --sensor-pin 18 \
    --target-led 0,0 \
    --test-database protocol_commands.json
```

This could automatically discover:
- Which commands turn LEDs on/off
- Brightness control methods
- Button press formats
- Communication timing requirements

**Verdict:** This is actually a really elegant solution to the "black box" protocol problem. It's hardware-assisted reverse engineering! 🔬

## Camera Resolution Analysis

### Pi Camera LED Resolution Analysis

**Pi Camera Specs:**
- Pi Camera v2: 3280×2464 pixels (8MP)
- Pi Camera v3: 4608×2592 pixels (12MP) 
- Grid: 16×8 = 128 LEDs total

**Theoretical Resolution:**
If we pointed the camera at the entire grid:
- **Full grid coverage:** ~290×185 pixels per LED (v2), ~410×259 pixels per LED (v3)
- **That's massive overkill** - way more than needed to detect on/off states

## Practical Approaches:

### **Option 1: Single LED Focus (Simplest)**
```
┌─────┐
│ 📷  │ ← Camera focused on one LED
└─────┘
    ↓
   ┌─┐
   │●│ ← Target LED (e.g., 0,0)
   └─┘
```
**Pros:** 
- Simple setup/calibration
- Very reliable detection
- Easy light isolation

**Cons:**
- Only tests one command at a time
- Slow protocol discovery

### **Option 2: Partial Grid (Sweet Spot?)**
Focus on a **4×4 section** (16 LEDs):
```
Grid section view:
┌───┬───┬───┬───┬─ ─ ─
│ 0 │ 1 │ 2 │ 3 │
├───┼───┼───┼───┼─ ─ ─  
│ 4 │ 5 │ 6 │ 7 │
├───┼───┼───┼───┼─ ─ ─
│ 8 │ 9 │10 │11 │  
├───┼───┼───┼───┼─ ─ ─
│12 │13 │14 │15 │
├─ ─┼─ ─┼─ ─┼─ ─┼─ ─ ─
```

**Resolution needed:** ~200×200 pixels per LED - still plenty!

### **Option 3: Full Grid (Ambitious)**
Monitor all 128 LEDs simultaneously:

**Benefits:**
- Test complex patterns instantly
- Detect multi-LED commands
- See full protocol behavior

**Challenges:**
- More complex image processing
- LED identification/mapping
- Calibration complexity

## Recommended Approach: **4×4 Section**

Here's why this is the sweet spot:

### **1. Sufficient Test Coverage:**
```python
# Test pattern that uses 4x4 section
test_patterns = [
    # Single LEDs
    [(0,0), (1,0), (2,0), (3,0)],
    # Diagonal
    [(0,0), (1,1), (2,2), (3,3)], 
    # Full section
    [(x,y) for x in range(4) for y in range(4)],
    # Clearing tests
    [], # All off
]
```

### **2. Pattern Recognition:**
Can detect different command types:
- **Individual LED commands:** Only one LED changes
- **Row/Column commands:** Whole row/column changes  
- **Area commands:** Rectangle changes
- **Broadcast commands:** All LEDs change

### **3. Implementation:**
```python
class GridSectionDetector:
    def __init__(self, section_size=(4, 4)):
        self.camera = PiCamera()
        self.section_size = section_size
        self.led_positions = self.calibrate_led_positions()
    
    def detect_led_changes(self, before_image, after_image):
        """Compare images and return which LEDs changed"""
        changes = {}
        for led_id, (x, y) in self.led_positions.items():
            # Extract LED region (e.g. 20x20 pixels around LED center)
            before_brightness = self.get_led_brightness(before_image, x, y)
            after_brightness = self.get_led_brightness(after_image, x, y)
            
            if abs(after_brightness - before_brightness) > threshold:
                changes[led_id] = after_brightness - before_brightness
                
        return changes
```

### **4. Protocol Discovery Speed:**
With 16 LEDs visible:
- Test **multiple LED positions** per command
- Detect **command patterns** (e.g., "this command affects position X,Y")
- **Much faster** than single LED approach

## Calibration Process:

### **1. Initial Setup:**
```python
def calibrate_led_positions():
    """Light up each LED individually and record pixel coordinates"""
    led_positions = {}
    
    for y in range(4):
        for x in range(4):
            # Try to light LED at grid position (x,y)
            send_test_command(x, y, brightness=15)
            time.sleep(0.1)
            
            # Capture image and find brightest spot
            image = camera.capture_array()
            brightest_pixel = find_brightest_region(image)
            led_positions[(x,y)] = brightest_pixel
            
            # Turn LED off
            send_clear_command(x, y)
            
    return led_positions
```

## **Recommendation: Start with 4×4**

This gives you:
- ✅ **Fast protocol discovery** (16x faster than single LED)
- ✅ **Pattern recognition** (detect command types)  
- ✅ **Manageable complexity** (simpler than full grid)
- ✅ **High success rate** (plenty of pixels per LED)

You could even start with a **2×2 section** (4 LEDs) for proof of concept, then expand to 4×4 once it's working.

## Multi-Camera Setup

**Q: Would the pi mini camera differentiate between different leds or would we just point it at one led? To be useful how many LEDs should we measure at once given we have 16 X 8 grid?**

### **Pi Camera Specs:**
- Pi Camera v2: 3280×2464 pixels (8MP)
- Pi Camera v3: 4608×2592 pixels (12MP) 
- Grid: 16×8 = 128 LEDs total

### **Theoretical Resolution:**
If we pointed the camera at the entire grid:
- **Full grid coverage:** ~290×185 pixels per LED (v2), ~410×259 pixels per LED (v3)
- **That's massive overkill** - way more than needed to detect on/off states

### **One Camera Per Grid**

Yes, exactly! **One camera per grid** is the most practical approach.

**Coverage per Camera:**
```
Camera 1 → Grid 1 (16×8 LEDs)
Camera 2 → Grid 2 (16×8 LEDs)
Camera 3 → Grid 3 (if you had more)
Camera 4 → Grid 4 (if you had more)
```

**Your Current Setup:**
```
📷 Camera 1 → Legacy Grid (STM 0483:5740)
📷 Camera 2 → Modern Grid (cafe:4001)
```

## Camera Hardware Options

**Q: Would this camera be ok: 5MP camera for Raspberry Pi? How would we connect 4 of them to one raspberry pi 5?**

### **5MP Camera Analysis**

**5MP camera specs typically:**
- Resolution: ~2592×1944 pixels
- **For single LED focus:** ~2592 pixels across entire LED = massive overkill ✅
- **For 4×4 section:** ~162×121 pixels per LED = still excellent ✅
- **Sufficient for the task:** Absolutely!

## Connecting 4 Cameras to Raspberry Pi 5

### **Pi 5 Camera Interfaces:**
- **2x MIPI CSI connectors** (ribbon cable cameras)
- **4x USB 3.0 ports** (USB cameras)

### **Option 1: Mixed Approach (Recommended)**
```
Pi 5 Setup:
├── 2x CSI cameras (ribbon cable) → Monitor 2 grids
├── 2x USB cameras → Monitor 2 more grids  
└── Total: 4 cameras, 4 grids
```

### **Option 2: USB Hub Expansion**
```
Pi 5 → USB Hub → 4x USB cameras
```

**Bandwidth consideration:** 4x 5MP cameras might push USB bandwidth limits for simultaneous capture.

## Multi-Camera Grid Monitoring

### **Practical Setup:**
```
Grid Layout:
┌─────────┐  ┌─────────┐
│ Grid 1  │  │ Grid 2  │
│ 📷 ← ─  │  │  ─ → 📷 │
└─────────┘  └─────────┘

┌─────────┐  ┌─────────┐  
│ Grid 3  │  │ Grid 4  │
│ 📷 ← ─  │  │  ─ → 📷 │
└─────────┘  └─────────┘
```

### **Code Architecture:**
```python
class MultiGridDetector:
    def __init__(self):
        self.cameras = {
            'grid1': Camera('/dev/video0'),  # CSI camera 1
            'grid2': Camera('/dev/video1'),  # CSI camera 2  
            'grid3': Camera('/dev/video2'),  # USB camera 1
            'grid4': Camera('/dev/video3'),  # USB camera 2
        }
        
        self.grid_ports = {
            'grid1': serial.Serial('/dev/ttyACM0'),
            'grid2': serial.Serial('/dev/ttyACM1'), 
            'grid3': serial.Serial('/dev/ttyACM2'),
            'grid4': serial.Serial('/dev/ttyACM3'),
        }
    
    def test_command_on_all_grids(self, command_bytes):
        # Capture before images
        before_images = {name: cam.capture() for name, cam in self.cameras.items()}
        
        # Send command to all grids
        for grid_name, port in self.grid_ports.items():
            port.write(command_bytes)
        
        time.sleep(0.1)
        
        # Capture after images  
        after_images = {name: cam.capture() for name, cam in self.cameras.items()}
        
        # Analyze changes
        results = {}
        for grid_name in self.cameras.keys():
            changes = self.detect_changes(before_images[grid_name], after_images[grid_name])
            results[grid_name] = changes
            
        return results
```

## Performance Considerations

### **Camera Bandwidth:**
```
5MP @ 30fps = ~150MB/s per camera
4 cameras = ~600MB/s total
USB 3.0 = ~400MB/s practical limit
```

**Solution:** Lower frame rate or sequential capture:
```python
# Sequential capture (slower but reliable)
for camera_name, camera in cameras.items():
    image = camera.capture()  # One at a time
    
# Or lower resolution for real-time
cameras.configure(resolution=(1280, 720))  # 720p instead of 5MP
```

### **Processing Power:**
Pi 5 should handle 4x 5MP image analysis fine, especially since we're just looking for brightness changes, not complex image processing.

## Recommended Hardware Setup

### **For 4 Grids:**
```
Equipment needed:
├── 1x Raspberry Pi 5 (8GB recommended)
├── 2x 5MP CSI cameras + ribbon cables 
├── 2x 5MP USB cameras
├── 1x Powered USB 3.0 hub (for stability)
├── 4x Mounting arms/clamps for cameras
└── Good lighting setup
```

### **For 2 Grids (Simpler Start):**
```
Equipment needed:
├── 1x Raspberry Pi 5  
├── 2x 5MP cameras (CSI or USB)
├── 2x Mounting solutions
└── Your existing 2 grids
```

## **Recommendation:**

**Start with 2 cameras for your 2 existing grids:**
1. Test the concept with your current setup
2. Prove the visual protocol detection works
3. Then expand to more grids if needed

**5MP cameras are definitely sufficient** - you could even use 2MP cameras and still have plenty of resolution for LED detection.

The concept is **one camera per grid** - this gives you:
- ✅ **Individual Protocol Discovery:** Each camera can detect which commands work on its specific grid
- ✅ **Comparative Testing:** Send same command to both grids, see which responds
- ✅ **Grid-Specific Optimization:** Focus each camera on the most important area of its grid
- ✅ **Parallel Processing:** Test both grids simultaneously

## Simple Test Scenario:
```python
# Test if command works on either/both grids
result = detector.test_command_comparison(bytes([0x12, 0x00]))

if result['legacy_grid']['changes_detected']:
    print("✅ Legacy grid responds to 0x12 0x00")
if result['modern_grid']['changes_detected']:
    print("✅ Modern grid responds to 0x12 0x00")
```

---

*[Conversation then shifted to discussing serialosc installation and build scripts for the Pi]*