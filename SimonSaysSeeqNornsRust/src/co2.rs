//! CO2 Data Integration Module
//! 
//! Loads and processes atmospheric CO2 data for climate-aware music generation.
//! Integrates with NOAA's atmospheric CO2 measurements to influence sequencer parameters.

use anyhow::{Result, anyhow};
use log::{info, warn, debug, error};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

/// CO2 measurement record from NOAA data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Co2Record {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub decimal_date: f64,
    pub co2_ppm: f32,
}

/// CO2 tempo analysis for wow and flutter detection
#[derive(Debug, Clone)]
pub struct Co2TempoAnalysis {
    pub wow_window: VecDeque<f32>,
    pub flutter_window: VecDeque<f32>,
    pub wow_average: f32,
    pub flutter_average: f32,
    pub wow_episodes: u32,
    pub flutter_episodes: u32,
    pub is_wow_stable: bool,
    pub is_flutter_stable: bool,
}

/// Main CO2 data manager
#[derive(Debug)]
pub struct Co2Manager {
    /// All historical CO2 records
    records: Vec<Co2Record>,
    /// Latest daily CO2 value
    latest_daily_value: Option<f32>,
    /// Current position in step-based CO2 cycling
    step_counter: usize,
    /// Current position in tick-based CO2 cycling
    tick_counter: usize,
    /// Tempo analysis for wow/flutter detection
    tempo_analysis: Co2TempoAnalysis,
    /// Configuration
    config: Co2Config,
    /// Data file paths
    daily_latest_path: PathBuf,
    all_daily_path: PathBuf,
}

/// CO2 configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Co2Config {
    /// Enable CO2 features
    pub enabled: bool,
    /// Data directory path
    pub data_dir: String,
    /// Wow threshold for tempo instability (BPM difference)
    pub wow_threshold: f32,
    /// Flutter threshold for micro-timing instability (BPM difference)
    pub flutter_threshold: f32,
    /// Window size for averaging (in ticks/steps)
    pub window_size: usize,
    /// CO2 voltage scaling factor (for CV output)
    pub voltage_scale: f32,
    /// Minimum CO2 value for scaling
    pub co2_min: f32,
    /// Maximum CO2 value for scaling
    pub co2_max: f32,
}

impl Default for Co2Config {
    fn default() -> Self {
        Self {
            enabled: true,
            data_dir: "/home/simonredfern/Documents/workspace_2025/SimonSaysSeeq/SimonSaysSeeqNornsRust/co2_data".to_string(),
            wow_threshold: 3.0,
            flutter_threshold: 0.25,
            window_size: 192, // 16 steps * 12 ticks
            voltage_scale: 50.0,
            co2_min: 280.0, // Pre-industrial baseline
            co2_max: 450.0, // High estimate for current era
        }
    }
}

impl Co2Manager {
    /// Create a new CO2 manager
    pub fn new(config: Co2Config) -> Result<Self> {
        let data_dir = PathBuf::from(&config.data_dir);
        
        let mut manager = Self {
            records: Vec::new(),
            latest_daily_value: None,
            step_counter: 1,
            tick_counter: 1,
            tempo_analysis: Co2TempoAnalysis {
                wow_window: VecDeque::with_capacity(config.window_size),
                flutter_window: VecDeque::with_capacity(config.window_size),
                wow_average: 0.0,
                flutter_average: 0.0,
                wow_episodes: 0,
                flutter_episodes: 0,
                is_wow_stable: true,
                is_flutter_stable: true,
            },
            config,
            daily_latest_path: data_dir.join("simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_daily_latest.csv"),
            all_daily_path: data_dir.join("simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_all_daily.csv"),
        };
        
        if manager.config.enabled {
            manager.load_data()?;
        } else {
            info!("CO2 features disabled in configuration");
        }
        
        Ok(manager)
    }
    
    /// Load CO2 data from files
    pub fn load_data(&mut self) -> Result<()> {
        // Load latest daily value
        self.load_latest_daily_value()?;
        
        // Load all historical data
        self.load_all_daily_records()?;
        
        // Initialize counters
        self.reset_counters();
        
        Ok(())
    }
    
    /// Load the latest daily CO2 value
    fn load_latest_daily_value(&mut self) -> Result<()> {
        if self.daily_latest_path.exists() {
            match fs::read_to_string(&self.daily_latest_path) {
                Ok(content) => {
                    let trimmed = content.trim();
                    if trimmed.is_empty() {
                        info!("Latest daily CO2 file is empty, skipping load");
                    } else {
                        match self.validate_co2_value(trimmed) {
                            Some(value) => {
                                self.latest_daily_value = Some(value);
                                info!("Loaded latest daily CO2 value: {:.2} ppm", value);
                            }
                            None => {
                                warn!("Invalid CO2 value in latest daily file: {}", trimmed);
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read latest daily CO2 file: {}", e);
                }
            }
        } else {
            warn!("Latest daily CO2 file not found: {:?}", self.daily_latest_path);
        }
        
        Ok(())
    }
    
    /// Load all daily CO2 records from CSV
    fn load_all_daily_records(&mut self) -> Result<()> {
        if !self.all_daily_path.exists() {
            warn!("All daily CO2 file not found: {:?}", self.all_daily_path);
            return Ok(());
        }
        
        let content = fs::read_to_string(&self.all_daily_path)?;
        let mut valid_records = 0;
        let mut invalid_records = 0;
        
        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            
            // Parse CSV line: year,month,day,decimal_date,co2_ppm
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if parts.len() >= 5 {
                match self.parse_co2_record(&parts) {
                    Ok(record) => {
                        self.records.push(record);
                        valid_records += 1;
                    }
                    Err(e) => {
                        if invalid_records < 10 { // Only log first 10 errors
                            debug!("Invalid CO2 record at line {}: {} - {}", line_num + 1, line, e);
                        }
                        invalid_records += 1;
                    }
                }
            } else {
                if invalid_records < 10 {
                    debug!("Malformed CO2 record at line {}: {}", line_num + 1, line);
                }
                invalid_records += 1;
            }
        }
        
        if valid_records > 0 {
            info!("Loaded {} valid CO2 records ({} invalid records skipped)", valid_records, invalid_records);
        } else {
            warn!("No valid CO2 records found in file");
        }
        
        Ok(())
    }
    
    /// Parse a single CO2 record from CSV parts
    fn parse_co2_record(&self, parts: &[&str]) -> Result<Co2Record> {
        let year: u16 = parts[0].parse()
            .map_err(|_| anyhow!("Invalid year: {}", parts[0]))?;
        
        let month: u8 = parts[1].parse()
            .map_err(|_| anyhow!("Invalid month: {}", parts[1]))?;
        
        let day: u8 = parts[2].parse()
            .map_err(|_| anyhow!("Invalid day: {}", parts[2]))?;
        
        let decimal_date: f64 = parts[3].parse()
            .map_err(|_| anyhow!("Invalid decimal date: {}", parts[3]))?;
        
        let co2_ppm = self.validate_co2_value(parts[4])
            .ok_or_else(|| anyhow!("Invalid CO2 value: {}", parts[4]))?;
        
        // Basic sanity checks
        if month < 1 || month > 12 {
            return Err(anyhow!("Month out of range: {}", month));
        }
        
        if day < 1 || day > 31 {
            return Err(anyhow!("Day out of range: {}", day));
        }
        
        if year < 1958 || year > 2100 {
            return Err(anyhow!("Year out of reasonable range: {}", year));
        }
        
        Ok(Co2Record {
            year,
            month,
            day,
            decimal_date,
            co2_ppm,
        })
    }
    
    /// Validate a CO2 value string and return parsed value if valid
    fn validate_co2_value(&self, raw_value: &str) -> Option<f32> {
        match raw_value.parse::<f32>() {
            Ok(value) => {
                // Check if value is positive, within reasonable range, and not NaN
                if value > 0.0 && value < 10000.0 && value.is_finite() {
                    Some(value)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
    
    /// Reset all counters
    pub fn reset_counters(&mut self) {
        if !self.records.is_empty() {
            self.step_counter = 1;
            self.tick_counter = 1;
        } else {
            self.step_counter = 0;
            self.tick_counter = 0;
        }
        
        // Reset tempo analysis
        self.tempo_analysis.wow_window.clear();
        self.tempo_analysis.flutter_window.clear();
        self.tempo_analysis.wow_average = 0.0;
        self.tempo_analysis.flutter_average = 0.0;
        self.tempo_analysis.wow_episodes = 0;
        self.tempo_analysis.flutter_episodes = 0;
        self.tempo_analysis.is_wow_stable = true;
        self.tempo_analysis.is_flutter_stable = true;
        
        debug!("CO2 counters reset");
    }
    
    /// Advance step counter and return current CO2 value
    pub fn advance_step(&mut self) -> Option<f32> {
        if self.records.is_empty() {
            return None;
        }
        
        let co2_value = self.records[self.step_counter - 1].co2_ppm;
        self.step_counter = self.step_counter % self.records.len() + 1;
        
        debug!("Step CO2: {:.2} ppm (step {})", co2_value, self.step_counter);
        Some(co2_value)
    }
    
    /// Advance tick counter and return current CO2 value
    pub fn advance_tick(&mut self) -> Option<f32> {
        if self.records.is_empty() {
            return None;
        }
        
        let co2_value = self.records[self.tick_counter - 1].co2_ppm;
        self.tick_counter = self.tick_counter % self.records.len() + 1;
        
        Some(co2_value)
    }
    
    /// Get current step CO2 value without advancing
    pub fn get_current_step_co2(&self) -> Option<f32> {
        if self.records.is_empty() || self.step_counter == 0 {
            return None;
        }
        
        Some(self.records[self.step_counter - 1].co2_ppm)
    }
    
    /// Get current tick CO2 value without advancing
    pub fn get_current_tick_co2(&self) -> Option<f32> {
        if self.records.is_empty() || self.tick_counter == 0 {
            return None;
        }
        
        Some(self.records[self.tick_counter - 1].co2_ppm)
    }
    
    /// Get latest daily CO2 value
    pub fn get_latest_daily_value(&self) -> Option<f32> {
        self.latest_daily_value
    }
    
    /// Get CO2 voltage offset for CV output (scaled)
    pub fn get_co2_voltage_offset(&self, co2_value: f32) -> f32 {
        // Scale CO2 value to voltage range
        let normalized = (co2_value - self.config.co2_min) / (self.config.co2_max - self.config.co2_min);
        let clamped = normalized.clamp(0.0, 1.0);
        clamped * 10.0 // 0-10V range typical for CV
    }
    
    /// Get CO2 value scaled for step offset
    pub fn get_step_offset(&self, co2_value: f32) -> f32 {
        co2_value / self.config.voltage_scale
    }
    
    /// Get CO2 value scaled for tick offset  
    pub fn get_tick_offset(&self, co2_value: f32) -> f32 {
        co2_value / self.config.voltage_scale
    }
    
    /// Analyze tempo stability using CO2 data influence
    pub fn analyze_tempo_stability(&mut self, current_tempo: f32) {
        // Add current tempo to analysis windows
        if self.tempo_analysis.wow_window.len() >= self.config.window_size {
            self.tempo_analysis.wow_window.pop_front();
        }
        self.tempo_analysis.wow_window.push_back(current_tempo);
        
        if self.tempo_analysis.flutter_window.len() >= self.config.window_size {
            self.tempo_analysis.flutter_window.pop_front();
        }
        self.tempo_analysis.flutter_window.push_back(current_tempo);
        
        // Calculate averages
        if !self.tempo_analysis.wow_window.is_empty() {
            self.tempo_analysis.wow_average = 
                self.tempo_analysis.wow_window.iter().sum::<f32>() / self.tempo_analysis.wow_window.len() as f32;
        }
        
        if !self.tempo_analysis.flutter_window.is_empty() {
            self.tempo_analysis.flutter_average = 
                self.tempo_analysis.flutter_window.iter().sum::<f32>() / self.tempo_analysis.flutter_window.len() as f32;
        }
        
        // Check for wow (large tempo instability)
        let wow_deviation = (current_tempo - self.tempo_analysis.wow_average).abs();
        if wow_deviation > self.config.wow_threshold {
            if self.tempo_analysis.is_wow_stable {
                self.tempo_analysis.wow_episodes += 1;
                self.tempo_analysis.is_wow_stable = false;
                debug!("Wow episode detected: deviation {:.2} BPM", wow_deviation);
            }
        } else {
            self.tempo_analysis.is_wow_stable = true;
        }
        
        // Check for flutter (small tempo instability)
        let flutter_deviation = (current_tempo - self.tempo_analysis.flutter_average).abs();
        if flutter_deviation > self.config.flutter_threshold {
            if self.tempo_analysis.is_flutter_stable {
                self.tempo_analysis.flutter_episodes += 1;
                self.tempo_analysis.is_flutter_stable = false;
                debug!("Flutter episode detected: deviation {:.2} BPM", flutter_deviation);
            }
        } else {
            self.tempo_analysis.is_flutter_stable = true;
        }
    }
    
    /// Get tempo stability status
    pub fn get_tempo_stability(&self) -> (bool, bool, u32, u32) {
        (
            self.tempo_analysis.is_wow_stable,
            self.tempo_analysis.is_flutter_stable,
            self.tempo_analysis.wow_episodes,
            self.tempo_analysis.flutter_episodes,
        )
    }
    
    /// Get number of loaded records
    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
    
    /// Check if CO2 data is available
    pub fn has_data(&self) -> bool {
        !self.records.is_empty()
    }
    
    /// Get CO2 status string for display
    pub fn get_status_string(&self) -> String {
        match self.latest_daily_value {
            Some(value) => format!("CO2: {:.2} ppm", value),
            None => "CO2: UNKNOWN".to_string(),
        }
    }
    
    /// Get detailed CO2 information
    /// Get CO2 information for display/debugging
    pub fn get_info(&self) -> Co2Info {
        Co2Info {
            enabled: self.config.enabled,
            has_data: self.has_data(),
            record_count: self.get_record_count(),
            latest_daily_value: self.latest_daily_value,
            current_step_value: self.get_current_step_co2(),
            current_tick_value: self.get_current_tick_co2(),
            step_counter: self.step_counter,
            tick_counter: self.tick_counter,
            wow_stable: self.tempo_analysis.is_wow_stable,
            flutter_stable: self.tempo_analysis.is_flutter_stable,
            wow_episodes: self.tempo_analysis.wow_episodes,
            flutter_episodes: self.tempo_analysis.flutter_episodes,
        }
    }

    /// Get detailed summary of loaded CO2 data for logging/debugging
    pub fn get_data_summary(&self) -> String {
        if !self.has_data() {
            return "No CO2 data loaded".to_string();
        }

        let first_record = &self.records[0];
        let last_record = &self.records[self.records.len() - 1];
        
        let min_co2 = self.records.iter()
            .map(|r| r.co2_ppm)
            .fold(f32::INFINITY, f32::min);
        let max_co2 = self.records.iter()
            .map(|r| r.co2_ppm)
            .fold(f32::NEG_INFINITY, f32::max);
        let avg_co2 = self.records.iter()
            .map(|r| r.co2_ppm)
            .sum::<f32>() / self.records.len() as f32;

        format!(
            "CO2 data: {} records from {}/{}/{} to {}/{}/{} | Range: {:.2}-{:.2} ppm | Avg: {:.2} ppm",
            self.records.len(),
            first_record.month, first_record.day, first_record.year,
            last_record.month, last_record.day, last_record.year,
            min_co2, max_co2, avg_co2
        )
    }
    
    /// Reload data from files
    pub fn reload_data(&mut self) -> Result<()> {
        info!("Reloading CO2 data");
        self.records.clear();
        self.latest_daily_value = None;
        self.load_data()
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: Co2Config) {
        let needs_reload = config.data_dir != self.config.data_dir || 
                          config.enabled != self.config.enabled;
        
        self.config = config;
        
        if needs_reload && self.config.enabled {
            if let Err(e) = self.reload_data() {
                error!("Failed to reload CO2 data after config update: {}", e);
            }
        }
    }
}

/// CO2 information structure for external consumption
#[derive(Debug, Clone)]
pub struct Co2Info {
    pub enabled: bool,
    pub has_data: bool,
    pub record_count: usize,
    pub latest_daily_value: Option<f32>,
    pub current_step_value: Option<f32>,
    pub current_tick_value: Option<f32>,
    pub step_counter: usize,
    pub tick_counter: usize,
    pub wow_stable: bool,
    pub flutter_stable: bool,
    pub wow_episodes: u32,
    pub flutter_episodes: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    fn create_test_data_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test latest daily file
        let latest_path = temp_dir.path().join("simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_daily_latest.csv");
        fs::write(&latest_path, "415.5").unwrap();
        
        // Create test all daily file
        let all_daily_path = temp_dir.path().join("simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_all_daily.csv");
        let test_data = "2023,1,1,2023.001,410.5\n2023,1,2,2023.003,411.0\n2023,1,3,2023.005,411.5\n";
        fs::write(&all_daily_path, test_data).unwrap();
        
        temp_dir
    }
    
    #[test]
    fn test_co2_manager_creation() {
        let temp_dir = create_test_data_dir();
        
        let config = Co2Config {
            enabled: true,
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        
        let manager = Co2Manager::new(config).unwrap();
        assert!(manager.has_data());
        assert_eq!(manager.get_record_count(), 3);
        assert_eq!(manager.get_latest_daily_value(), Some(415.5));
    }
    
    #[test]
    fn test_co2_validation() {
        let config = Co2Config::default();
        let manager = Co2Manager::new(config).unwrap();
        
        assert_eq!(manager.validate_co2_value("415.5"), Some(415.5));
        assert_eq!(manager.validate_co2_value("invalid"), None);
        assert_eq!(manager.validate_co2_value("-10"), None);
        assert_eq!(manager.validate_co2_value("99999"), None);
    }
    
    #[test]
    fn test_counter_advancement() {
        let temp_dir = create_test_data_dir();
        
        let config = Co2Config {
            enabled: true,
            data_dir: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        
        let mut manager = Co2Manager::new(config).unwrap();
        
        // Test step advancement
        assert_eq!(manager.advance_step(), Some(410.5));
        assert_eq!(manager.advance_step(), Some(411.0));
        assert_eq!(manager.advance_step(), Some(411.5));
        assert_eq!(manager.advance_step(), Some(410.5)); // Wraps around
        
        // Test tick advancement
        manager.reset_counters();
        assert_eq!(manager.advance_tick(), Some(410.5));
        assert_eq!(manager.advance_tick(), Some(411.0));
    }
    
    #[test]
    fn test_voltage_scaling() {
        let config = Co2Config {
            co2_min: 400.0,
            co2_max: 420.0,
            ..Default::default()
        };
        
        let manager = Co2Manager::new(config).unwrap();
        
        assert_eq!(manager.get_co2_voltage_offset(400.0), 0.0);
        assert_eq!(manager.get_co2_voltage_offset(410.0), 5.0);
        assert_eq!(manager.get_co2_voltage_offset(420.0), 10.0);
    }
    
    #[test]
    fn test_tempo_stability_analysis() {
        let config = Co2Config {
            wow_threshold: 5.0,
            flutter_threshold: 1.0,
            ..Default::default()
        };
        
        let mut manager = Co2Manager::new(config).unwrap();
        
        // Stable tempo
        manager.analyze_tempo_stability(120.0);
        manager.analyze_tempo_stability(120.5);
        manager.analyze_tempo_stability(119.5);
        
        let (wow_stable, flutter_stable, _, _) = manager.get_tempo_stability();
        assert!(wow_stable);
        assert!(flutter_stable);
        
        // Cause wow instability
        manager.analyze_tempo_stability(130.0);
        let (wow_stable, _, _, _) = manager.get_tempo_stability();
        assert!(!wow_stable);
    }
}