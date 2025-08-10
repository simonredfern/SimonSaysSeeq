//! Screen module - Handle framebuffer rendering for Norns OLED display
//! 
//! Provides drawing capabilities for the 128x64 monochrome OLED screen on Norns.

use anyhow::Result;
#[cfg(feature = "framebuffer-support")]
use framebuffer::Framebuffer;
use log::info;

/// Font data for simple 6x8 pixel font
const FONT_6X8: &[&[u8]] = &[
    // Space (32)
    &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    // ! (33)
    &[0x00, 0x00, 0x5F, 0x00, 0x00, 0x00],
    // " (34)
    &[0x00, 0x07, 0x00, 0x07, 0x00, 0x00],
    // # (35)
    &[0x14, 0x7F, 0x14, 0x7F, 0x14, 0x00],
    // $ (36)
    &[0x24, 0x2A, 0x7F, 0x2A, 0x12, 0x00],
    // % (37)
    &[0x23, 0x13, 0x08, 0x64, 0x62, 0x00],
    // & (38)
    &[0x36, 0x49, 0x55, 0x22, 0x50, 0x00],
    // ' (39)
    &[0x00, 0x05, 0x03, 0x00, 0x00, 0x00],
    // ( (40)
    &[0x00, 0x1C, 0x22, 0x41, 0x00, 0x00],
    // ) (41)
    &[0x00, 0x41, 0x22, 0x1C, 0x00, 0x00],
    // * (42)
    &[0x08, 0x2A, 0x1C, 0x2A, 0x08, 0x00],
    // + (43)
    &[0x08, 0x08, 0x3E, 0x08, 0x08, 0x00],
    // , (44)
    &[0x00, 0x50, 0x30, 0x00, 0x00, 0x00],
    // - (45)
    &[0x08, 0x08, 0x08, 0x08, 0x08, 0x00],
    // . (46)
    &[0x00, 0x60, 0x60, 0x00, 0x00, 0x00],
    // / (47)
    &[0x20, 0x10, 0x08, 0x04, 0x02, 0x00],
    // 0 (48)
    &[0x3E, 0x51, 0x49, 0x45, 0x3E, 0x00],
    // 1 (49)
    &[0x00, 0x42, 0x7F, 0x40, 0x00, 0x00],
    // 2 (50)
    &[0x42, 0x61, 0x51, 0x49, 0x46, 0x00],
    // 3 (51)
    &[0x21, 0x41, 0x45, 0x4B, 0x31, 0x00],
    // 4 (52)
    &[0x18, 0x14, 0x12, 0x7F, 0x10, 0x00],
    // 5 (53)
    &[0x27, 0x45, 0x45, 0x45, 0x39, 0x00],
    // 6 (54)
    &[0x3C, 0x4A, 0x49, 0x49, 0x30, 0x00],
    // 7 (55)
    &[0x01, 0x71, 0x09, 0x05, 0x03, 0x00],
    // 8 (56)
    &[0x36, 0x49, 0x49, 0x49, 0x36, 0x00],
    // 9 (57)
    &[0x06, 0x49, 0x49, 0x29, 0x1E, 0x00],
    // : (58)
    &[0x00, 0x36, 0x36, 0x00, 0x00, 0x00],
    // ; (59)
    &[0x00, 0x56, 0x36, 0x00, 0x00, 0x00],
    // < (60)
    &[0x00, 0x08, 0x14, 0x22, 0x41, 0x00],
    // = (61)
    &[0x14, 0x14, 0x14, 0x14, 0x14, 0x00],
    // > (62)
    &[0x41, 0x22, 0x14, 0x08, 0x00, 0x00],
    // ? (63)
    &[0x02, 0x01, 0x51, 0x09, 0x06, 0x00],
    // A (65)
    &[0x7E, 0x11, 0x11, 0x11, 0x7E, 0x00],
    // B (66)
    &[0x7F, 0x49, 0x49, 0x49, 0x36, 0x00],
    // C (67)
    &[0x3E, 0x41, 0x41, 0x41, 0x22, 0x00],
    // D (68)
    &[0x7F, 0x41, 0x41, 0x22, 0x1C, 0x00],
    // E (69)
    &[0x7F, 0x49, 0x49, 0x49, 0x41, 0x00],
    // F (70)
    &[0x7F, 0x09, 0x09, 0x01, 0x01, 0x00],
    // G (71)
    &[0x3E, 0x41, 0x41, 0x51, 0x32, 0x00],
    // H (72)
    &[0x7F, 0x08, 0x08, 0x08, 0x7F, 0x00],
    // I (73)
    &[0x00, 0x41, 0x7F, 0x41, 0x00, 0x00],
    // J (74)
    &[0x20, 0x40, 0x41, 0x3F, 0x01, 0x00],
    // K (75)
    &[0x7F, 0x08, 0x14, 0x22, 0x41, 0x00],
    // L (76)
    &[0x7F, 0x40, 0x40, 0x40, 0x40, 0x00],
    // M (77)
    &[0x7F, 0x02, 0x04, 0x02, 0x7F, 0x00],
    // N (78)
    &[0x7F, 0x04, 0x08, 0x10, 0x7F, 0x00],
    // O (79)
    &[0x3E, 0x41, 0x41, 0x41, 0x3E, 0x00],
    // P (80)
    &[0x7F, 0x09, 0x09, 0x09, 0x06, 0x00],
    // Q (81)
    &[0x3E, 0x41, 0x51, 0x21, 0x5E, 0x00],
    // R (82)
    &[0x7F, 0x09, 0x19, 0x29, 0x46, 0x00],
    // S (83)
    &[0x46, 0x49, 0x49, 0x49, 0x31, 0x00],
    // T (84)
    &[0x01, 0x01, 0x7F, 0x01, 0x01, 0x00],
    // U (85)
    &[0x3F, 0x40, 0x40, 0x40, 0x3F, 0x00],
    // V (86)
    &[0x1F, 0x20, 0x40, 0x20, 0x1F, 0x00],
    // W (87)
    &[0x7F, 0x20, 0x18, 0x20, 0x7F, 0x00],
    // X (88)
    &[0x63, 0x14, 0x08, 0x14, 0x63, 0x00],
    // Y (89)
    &[0x03, 0x04, 0x78, 0x04, 0x03, 0x00],
    // Z (90)
    &[0x61, 0x51, 0x49, 0x45, 0x43, 0x00],
];

/// Screen manager for Norns OLED display
pub struct ScreenManager {
    #[cfg(feature = "framebuffer-support")]
    framebuffer: Option<Framebuffer>,
    width: usize,
    height: usize,
    buffer: Vec<u8>,
    #[cfg(feature = "framebuffer-support")]
    fb_buffer: Vec<u8>,
    cursor_x: usize,
    cursor_y: usize,
    beat_indicator: usize,
}

impl ScreenManager {
    /// Create a new screen manager
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            #[cfg(feature = "framebuffer-support")]
            framebuffer: None,
            width: 128,
            height: 64,
            buffer: vec![0u8; 128 * 64 / 8], // 1 bit per pixel, packed
            #[cfg(feature = "framebuffer-support")]
            fb_buffer: vec![0u8; 128 * 64 * 2], // 16 bits per pixel for framebuffer
            cursor_x: 0,
            cursor_y: 0,
            beat_indicator: 0,
        };
        
        #[cfg(feature = "framebuffer-support")]
        manager.initialize()?;
        #[cfg(not(feature = "framebuffer-support"))]
        info!("Screen simulation mode - no actual framebuffer access");
        
        Ok(manager)
    }
    
    /// Initialize the framebuffer
    #[cfg(feature = "framebuffer-support")]
    fn initialize(&mut self) -> Result<()> {
        match Framebuffer::new("/dev/fb0") {
            Ok(fb) => {
                info!("Framebuffer initialized: {}x{}", fb.var_screen_info.xres, fb.var_screen_info.yres);
                
                // Update dimensions from actual framebuffer
                self.width = fb.var_screen_info.xres as usize;
                self.height = fb.var_screen_info.yres as usize;
                
                // Reallocate buffers if needed
                let buffer_size = (self.width * self.height) / 8;
                if self.buffer.len() != buffer_size {
                    self.buffer = vec![0u8; buffer_size];
                }
                
                // Reallocate framebuffer buffer to match expected format
                let fb_buffer_size = self.width * self.height * 2; // 16 bits per pixel
                self.fb_buffer = vec![0u8; fb_buffer_size];
                
                self.framebuffer = Some(fb);
                info!("Screen initialized: {}x{}", self.width, self.height);
            }
            Err(e) => {
                warn!("Failed to initialize framebuffer: {} - using simulation mode", e);
                // Continue without framebuffer for testing/simulation
            }
        }
        
        Ok(())
    }
    
    /// Clear the screen buffer
    pub fn clear(&mut self) {
        self.buffer.fill(0);
        self.cursor_x = 0;
        self.cursor_y = 0;
    }
    
    /// Set a pixel at the given coordinates
    pub fn set_pixel(&mut self, x: usize, y: usize, on: bool) {
        if x >= self.width || y >= self.height {
            return;
        }
        
        let byte_index = (y / 8) * self.width + x;
        let bit_index = y % 8;
        
        if byte_index < self.buffer.len() {
            if on {
                self.buffer[byte_index] |= 1 << bit_index;
            } else {
                self.buffer[byte_index] &= !(1 << bit_index);
            }
        }
    }
    
    /// Get pixel state at coordinates
    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        
        let byte_index = (y / 8) * self.width + x;
        let bit_index = y % 8;
        
        if byte_index < self.buffer.len() {
            (self.buffer[byte_index] & (1 << bit_index)) != 0
        } else {
            false
        }
    }
    
    /// Draw text at the current cursor position
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        let mut current_x = x;
        let current_y = y;
        
        for ch in text.chars() {
            if current_x + 6 > self.width {
                break; // Text would go off screen
            }
            
            self.draw_char(current_x, current_y, ch);
            current_x += 6; // Each character is 6 pixels wide
        }
    }
    
    /// Draw a single character
    fn draw_char(&mut self, x: usize, y: usize, ch: char) {
        let char_code = ch as usize;
        
        // Map character to font index
        let font_index = if char_code >= 32 && char_code <= 90 {
            char_code - 32
        } else {
            0 // Default to space
        };
        
        if font_index < FONT_6X8.len() {
            let char_data = FONT_6X8[font_index];
            
            for (col, &byte) in char_data.iter().enumerate() {
                if x + col >= self.width {
                    break;
                }
                
                for row in 0..8 {
                    if y + row >= self.height {
                        break;
                    }
                    
                    let pixel_on = (byte & (1 << row)) != 0;
                    self.set_pixel(x + col, y + row, pixel_on);
                }
            }
        }
    }
    
    /// Draw a horizontal line
    pub fn draw_hline(&mut self, x: usize, y: usize, width: usize) {
        for i in 0..width {
            if x + i < self.width {
                self.set_pixel(x + i, y, true);
            }
        }
    }
    
    /// Draw a vertical line
    pub fn draw_vline(&mut self, x: usize, y: usize, height: usize) {
        for i in 0..height {
            if y + i < self.height {
                self.set_pixel(x, y + i, true);
            }
        }
    }
    
    /// Draw a rectangle outline
    pub fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize) {
        // Top and bottom lines
        self.draw_hline(x, y, width);
        if height > 1 {
            self.draw_hline(x, y + height - 1, width);
        }
        
        // Left and right lines
        self.draw_vline(x, y, height);
        if width > 1 {
            self.draw_vline(x + width - 1, y, height);
        }
    }
    
    /// Fill a rectangle
    pub fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize) {
        for row in 0..height {
            if y + row < self.height {
                self.draw_hline(x, y + row, width);
            }
        }
    }
    
    /// Draw the sequencer grid visualization
    pub fn draw_grid(&mut self, grid: &[Vec<u8>], current_step: usize, _current_bar: usize) {
        let grid_x = 64; // Start grid at x=64
        let grid_y = 8;  // Start grid at y=8
        let cell_width = 3;
        let cell_height = 6;
        
        for (col, column) in grid.iter().enumerate() {
            if col >= 16 { break; } // Only show first 16 columns
            
            for (row, &value) in column.iter().enumerate() {
                if row >= 7 { break; } // Only show first 7 rows (sequence rows)
                
                let cell_x = grid_x + col * cell_width;
                let cell_y = grid_y + row * cell_height;
                
                // Draw cell based on value and current step
                if col + 1 == current_step {
                    // Current step - draw with highlight
                    if value > 0 {
                        self.fill_rect(cell_x, cell_y, cell_width - 1, cell_height - 1);
                    } else {
                        self.draw_rect(cell_x, cell_y, cell_width - 1, cell_height - 1);
                    }
                } else if value > 0 {
                    // Active cell, not current step
                    let brightness = if value >= 2 { 2 } else { 1 }; // Ratchet vs normal
                    
                    if brightness == 2 {
                        self.fill_rect(cell_x, cell_y, cell_width - 1, cell_height - 1);
                    } else {
                        self.draw_rect(cell_x, cell_y, cell_width - 1, cell_height - 1);
                        self.set_pixel(cell_x + 1, cell_y + 1, true);
                    }
                }
                // Inactive cells are left empty
            }
        }
    }
    
    /// Set beat indicator (for visual metronome)
    pub fn set_beat_indicator(&mut self, beat: usize) {
        self.beat_indicator = beat;
    }
    
    /// Draw beat indicator
    pub fn draw_beat_indicator(&mut self) {
        // Draw beat dots in top right corner
        let base_x = self.width - 20;
        let y = 2;
        
        for i in 1..=4 {
            let x = base_x + (i - 1) * 4;
            if i == self.beat_indicator {
                // Current beat - filled circle
                self.fill_rect(x, y, 3, 3);
            } else {
                // Other beats - empty circle
                self.draw_rect(x, y, 3, 3);
            }
        }
    }
    
    /// Draw tempo visualization
    pub fn draw_tempo_viz(&mut self, tempo: f32) {
        // Simple tempo bar
        let bar_x = 64;
        let bar_y = 2;
        let bar_width = 40;
        let bar_height = 4;
        
        // Background
        self.draw_rect(bar_x, bar_y, bar_width, bar_height);
        
        // Fill based on tempo (20-200 BPM range)
        let tempo_normalized = ((tempo - 20.0) / 180.0).clamp(0.0, 1.0);
        let fill_width = (bar_width as f32 * tempo_normalized) as usize;
        
        if fill_width > 0 {
            self.fill_rect(bar_x + 1, bar_y + 1, fill_width.saturating_sub(2), bar_height - 2);
        }
    }
    
    /// Update the physical display
    pub fn update(&mut self) -> Result<()> {
        // Draw beat indicator
        self.draw_beat_indicator();
        
        #[cfg(feature = "framebuffer-support")]
        {
            // Convert 1-bit buffer to 16-bit framebuffer format first
            self.convert_buffer_to_framebuffer();
            
            if let Some(ref mut fb) = self.framebuffer {
                // Copy converted buffer to the framebuffer
                let fb_length = fb.frame.len();
                let copy_length = std::cmp::min(self.fb_buffer.len(), fb_length);
                
                fb.frame[..copy_length].copy_from_slice(&self.fb_buffer[..copy_length]);
                
                // Write the converted buffer to framebuffer
                fb.write_frame(&self.fb_buffer[..copy_length]);
            }
        }
        
        #[cfg(not(feature = "framebuffer-support"))]
        {
            // Screen update in simulation mode (no logging to reduce noise)
        }
        
        Ok(())
    }
    
    #[cfg(feature = "framebuffer-support")]
    fn convert_buffer_to_framebuffer(&mut self) {
        // Convert 1-bit packed buffer to 16-bit RGB565 format
        // RGB565: RRRRRGGGGGGBBBBB (16 bits total)
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel_on = self.get_pixel(x, y);
                let fb_index = (y * self.width + x) * 2;
                
                if fb_index + 1 < self.fb_buffer.len() {
                    let pixel_value: u16 = if pixel_on {
                        0xFFFF  // White: all bits set
                    } else {
                        0x0000  // Black: all bits clear
                    };
                    
                    // Convert to little-endian bytes
                    self.fb_buffer[fb_index] = (pixel_value & 0xFF) as u8;
                    self.fb_buffer[fb_index + 1] = (pixel_value >> 8) as u8;
                }
            }
        }
    }
    
    /// Get screen dimensions
    pub fn get_dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    
    /// Draw a simple progress bar
    pub fn draw_progress_bar(&mut self, x: usize, y: usize, width: usize, height: usize, progress: f32) {
        // Background
        self.draw_rect(x, y, width, height);
        
        // Fill
        let fill_width = (width as f32 * progress.clamp(0.0, 1.0)) as usize;
        if fill_width > 2 {
            self.fill_rect(x + 1, y + 1, fill_width - 2, height - 2);
        }
    }
    
    /// Draw sequence step indicators
    pub fn draw_step_indicators(&mut self, current_step: usize, total_steps: usize) {
        let indicator_y = self.height - 8;
        let indicator_width = self.width / total_steps;
        
        for step in 1..=total_steps {
            let x = (step - 1) * indicator_width;
            
            if step == current_step {
                // Current step - filled
                self.fill_rect(x, indicator_y, indicator_width - 1, 6);
            } else {
                // Other steps - outline only
                self.draw_rect(x, indicator_y, indicator_width - 1, 6);
            }
        }
    }
}

impl Drop for ScreenManager {
    fn drop(&mut self) {
        // Clear screen on exit - catch any panics to prevent issues during cleanup
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.clear();
            let _ = self.update();
        })).is_err() {
            // If clearing fails, just log and continue
            info!("Screen manager dropped (cleanup failed)");
        } else {
            info!("Screen manager dropped");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_screen_creation() {
        let screen = ScreenManager::new().unwrap();
        assert_eq!(screen.get_dimensions(), (128, 64));
    }
    
    #[test]
    fn test_pixel_operations() {
        let mut screen = ScreenManager::new().unwrap();
        
        // Test setting and getting pixels
        screen.set_pixel(10, 10, true);
        assert!(screen.get_pixel(10, 10));
        
        screen.set_pixel(10, 10, false);
        assert!(!screen.get_pixel(10, 10));
        
        // Test bounds checking
        screen.set_pixel(200, 200, true); // Should not panic
        assert!(!screen.get_pixel(200, 200));
    }
    
    #[test]
    fn test_clear() {
        let mut screen = ScreenManager::new().unwrap();
        
        // Set some pixels
        screen.set_pixel(10, 10, true);
        screen.set_pixel(20, 20, true);
        
        // Clear
        screen.clear();
        
        // Check pixels are cleared
        assert!(!screen.get_pixel(10, 10));
        assert!(!screen.get_pixel(20, 20));
    }
    
    #[test]
    fn test_drawing_primitives() {
        let mut screen = ScreenManager::new().unwrap();
        
        // Test horizontal line
        screen.draw_hline(10, 10, 5);
        for i in 0..5 {
            assert!(screen.get_pixel(10 + i, 10));
        }
        
        // Test vertical line
        screen.clear();
        screen.draw_vline(10, 10, 5);
        for i in 0..5 {
            assert!(screen.get_pixel(10, 10 + i));
        }
        
        // Test rectangle
        screen.clear();
        screen.draw_rect(10, 10, 5, 5);
        
        // Check corners
        assert!(screen.get_pixel(10, 10));      // Top-left
        assert!(screen.get_pixel(14, 10));      // Top-right
        assert!(screen.get_pixel(10, 14));      // Bottom-left
        assert!(screen.get_pixel(14, 14));      // Bottom-right
        
        // Check center is empty
        assert!(!screen.get_pixel(12, 12));
    }
}