//! SimonSaysSeeq - Pure Rust implementation for Norns hardware
//! 
//! This library provides a complete music sequencer implementation designed
//! for the Norns platform, with support for monome grid controllers via
//! serial communication.

pub mod config;

pub mod grid_osc;
pub mod hardware;
pub mod midi;
pub mod screen;
pub mod sequencer;
pub mod co2;

// Re-export commonly used types
pub use config::Config;
// Use OSC-based grid manager for proper serialosc communication
pub use grid_osc::{GridManager, GridButtonEvent};
pub use sequencer::Sequencer;
pub use midi::MidiManager;
pub use screen::ScreenManager;
pub use hardware::NornsHardware;
pub use co2::Co2Manager;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default configuration for the application
pub fn default_config() -> Config {
    Config::default()
}

/// Initialize logging with default settings
pub fn init_logging() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_exists() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert_eq!(config.midi.default_channel, 1);
        assert_eq!(config.grid.rotation, 0);
        assert_eq!(config.sequencer.default_tempo, 120.0);
    }
}