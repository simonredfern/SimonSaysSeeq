//! Configuration module for SimonSaysSeeq
//! 
//! Handles loading and saving application configuration from TOML files.

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// MIDI configuration
    pub midi: MidiConfig,
    /// Grid configuration
    pub grid: GridConfig,
    /// Sequencer configuration
    pub sequencer: SequencerConfig,
    /// Hardware configuration
    pub hardware: HardwareConfig,
    /// Display configuration
    pub display: DisplayConfig,
    /// CO2 data configuration
    pub co2: crate::co2::Co2Config,
}

/// MIDI-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiConfig {
    /// MIDI output device name (empty string = auto-detect)
    pub device: String,
    /// Default MIDI channel (1-16)
    pub default_channel: u8,
    /// Default velocity
    pub default_velocity: u8,
    /// Send MIDI clock
    pub send_clock: bool,
    /// MIDI clock PPQ (pulses per quarter note)
    pub clock_ppq: u16,
    /// Stuck note cleanup timeout (seconds)
    pub stuck_note_timeout: u64,
}

/// Grid-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    /// Grid rotation (0, 90, 180, 270 degrees)
    pub rotation: u16,
    /// Default LED brightness (0-15)
    pub default_brightness: u8,
    /// Button debounce time (milliseconds)
    pub debounce_ms: u64,
    /// Auto-detect grids
    pub auto_detect: bool,
}

/// Sequencer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerConfig {
    /// Default tempo (BPM)
    pub default_tempo: f32,
    /// Steps per bar
    pub steps_per_bar: usize,
    /// Ticks per step
    pub ticks_per_step: u32,
    /// Default first step for new patterns
    pub default_first_step: usize,
    /// Default last step for new patterns
    pub default_last_step: usize,

    /// Auto-save interval (seconds, 0 = disabled)
    pub auto_save_interval: u64,
}

/// Hardware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    /// Input device polling interval (milliseconds)
    pub input_poll_ms: u64,
    /// Encoder sensitivity multiplier
    pub encoder_sensitivity: f32,
    /// Button hold time for secondary functions (milliseconds)
    pub button_hold_ms: u64,
    /// Hardware simulation mode (for development)
    pub simulation_mode: bool,
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Screen refresh rate (FPS)
    pub refresh_rate: u32,
    /// Screen brightness (0-255)
    pub brightness: u8,
    /// Show beat indicators
    pub show_beat_indicators: bool,
    /// Show tempo visualization
    pub show_tempo_viz: bool,
    /// Text font scale
    pub font_scale: u8,
}



impl Default for Config {
    fn default() -> Self {
        Self {
            midi: MidiConfig::default(),
            grid: GridConfig::default(),
            sequencer: SequencerConfig::default(),
            hardware: HardwareConfig::default(),
            display: DisplayConfig::default(),
            co2: crate::co2::Co2Config::default(),
        }
    }
}

impl Default for MidiConfig {
    fn default() -> Self {
        Self {
            device: String::new(), // Auto-detect
            default_channel: 1,
            default_velocity: 100,
            send_clock: true,
            clock_ppq: 24,
            stuck_note_timeout: 5,
        }
    }
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            rotation: 0,
            default_brightness: 5,
            debounce_ms: 50,
            auto_detect: true,
        }
    }
}

impl Default for SequencerConfig {
    fn default() -> Self {
        Self {
            default_tempo: 60.0,
            steps_per_bar: 16,
            ticks_per_step: 12,
            default_first_step: 1,
            default_last_step: 16,

            auto_save_interval: 300, // 5 minutes
        }
    }
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            input_poll_ms: 10,
            encoder_sensitivity: 1.0,
            button_hold_ms: 500,
            simulation_mode: false,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            refresh_rate: 30,
            brightness: 255,
            show_beat_indicators: true,
            show_tempo_viz: true,
            font_scale: 1,
        }
    }
}



impl Config {
    /// Load configuration from file, or create default if not found
    pub fn load_or_default() -> Result<Self> {
        let config_path = Self::get_config_path();
        
        if config_path.exists() {
            Self::load(&config_path)
        } else {
            info!("Configuration file not found, creating default at: {:?}", config_path);
            let config = Self::default();
            config.save(&config_path)?;
            Ok(config)
        }
    }
    
    /// Load configuration from specified path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let config: Config = toml::from_str(&content)?;
        
        info!("Configuration loaded from: {:?}", path.as_ref());
        config.validate()?;
        
        Ok(config)
    }
    
    /// Save configuration to specified path
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path.as_ref(), content)?;
        
        info!("Configuration saved to: {:?}", path.as_ref());
        Ok(())
    }
    
    /// Get the default configuration file path
    pub fn get_config_path() -> PathBuf {
        // Try to use XDG config directory, fall back to current directory
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("simon-says-seeq").join("config.toml")
        } else {
            PathBuf::from("simon_says_seeq_config.toml")
        }
    }
    
    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        // Validate MIDI config
        if self.midi.default_channel < 1 || self.midi.default_channel > 16 {
            return Err(anyhow::anyhow!("MIDI channel must be 1-16"));
        }
        
        if self.midi.default_velocity > 127 {
            return Err(anyhow::anyhow!("MIDI velocity must be 0-127"));
        }
        
        // Validate grid config
        if ![0, 90, 180, 270].contains(&self.grid.rotation) {
            return Err(anyhow::anyhow!("Grid rotation must be 0, 90, 180, or 270 degrees"));
        }
        
        if self.grid.default_brightness > 15 {
            return Err(anyhow::anyhow!("Grid brightness must be 0-15"));
        }
        
        // Validate sequencer config
        if self.sequencer.default_tempo < 60.0 || self.sequencer.default_tempo > 200.0 {
            return Err(anyhow::anyhow!("Tempo must be between 60-200 BPM"));
        }
        
        if self.sequencer.steps_per_bar == 0 || self.sequencer.steps_per_bar > 64 {
            return Err(anyhow::anyhow!("Steps per bar must be 1-64"));
        }
        
        if self.sequencer.ticks_per_step == 0 || self.sequencer.ticks_per_step > 96 {
            return Err(anyhow::anyhow!("Ticks per step must be 1-96"));
        }
        
        if self.sequencer.default_first_step == 0 || 
           self.sequencer.default_last_step == 0 ||
           self.sequencer.default_first_step > self.sequencer.default_last_step ||
           self.sequencer.default_last_step > self.sequencer.steps_per_bar {
            return Err(anyhow::anyhow!("Invalid step range"));
        }
        

        
        // Validate display config
        if self.display.refresh_rate == 0 || self.display.refresh_rate > 120 {
            return Err(anyhow::anyhow!("Refresh rate must be 1-120 FPS"));
        }
        
        if self.display.font_scale == 0 || self.display.font_scale > 4 {
            return Err(anyhow::anyhow!("Font scale must be 1-4"));
        }
        
        Ok(())
    }
    
    /// Get MIDI device name (for backward compatibility)
    pub fn midi_device(&self) -> &str {
        &self.midi.device
    }
    
    /// Create a minimal configuration for testing
    pub fn minimal() -> Self {
        Self {
            midi: MidiConfig {
                device: "".to_string(),
                default_channel: 1,
                default_velocity: 100,
                send_clock: false,
                clock_ppq: 24,
                stuck_note_timeout: 5,
            },
            grid: GridConfig {
                rotation: 0,
                default_brightness: 5,
                debounce_ms: 50,
                auto_detect: false,
            },
            sequencer: SequencerConfig {
                default_tempo: 120.0,
                steps_per_bar: 16,
                ticks_per_step: 12,
                default_first_step: 1,
                default_last_step: 16,

                auto_save_interval: 0, // Disabled
            },
            hardware: HardwareConfig {
                input_poll_ms: 10,
                encoder_sensitivity: 1.0,
                button_hold_ms: 500,
                simulation_mode: true, // Enable simulation for testing
            },
            display: DisplayConfig {
                refresh_rate: 30,
                brightness: 255,
                show_beat_indicators: true,
                show_tempo_viz: false,
                font_scale: 1,
            },
            co2: crate::co2::Co2Config {
                enabled: false,
                data_dir: "/tmp/co2_data".to_string(),
                wow_threshold: 20.0,
                flutter_threshold: 10.0,
                window_size: 100,
                voltage_scale: 1.0,
                co2_min: 320.0,
                co2_max: 450.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.midi.default_channel, deserialized.midi.default_channel);
        assert_eq!(config.sequencer.default_tempo, deserialized.sequencer.default_tempo);
    }
    
    #[test]
    fn test_config_save_load() {
        let config = Config::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        config.save(temp_file.path()).unwrap();
        let loaded_config = Config::load(temp_file.path()).unwrap();
        
        assert_eq!(config.midi.default_channel, loaded_config.midi.default_channel);
        assert_eq!(config.sequencer.default_tempo, loaded_config.sequencer.default_tempo);
    }
    
    #[test]
    fn test_validation() {
        let mut config = Config::default();
        
        // Test invalid MIDI channel
        config.midi.default_channel = 17;
        assert!(config.validate().is_err());
        
        config.midi.default_channel = 1;
        assert!(config.validate().is_ok());
        
        // Test invalid tempo
        config.sequencer.default_tempo = 300.0;
        assert!(config.validate().is_err());
        
        config.sequencer.default_tempo = 120.0;
        assert!(config.validate().is_ok());
        
        // Test invalid step range
        config.sequencer.default_first_step = 10;
        config.sequencer.default_last_step = 5;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_minimal_config() {
        let config = Config::minimal();
        assert!(config.validate().is_ok());
        assert!(config.hardware.simulation_mode);
        assert!(!config.co2.enabled);
    }
}