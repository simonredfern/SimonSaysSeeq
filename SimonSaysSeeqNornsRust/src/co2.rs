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
    /// Current position in step-based cycling
    total_step_counter: usize,
    /// Current position in tick-based cycling
    total_tick_counter: usize,

    /// Approximate days per year for seasonal anomaly calculations
    days_per_year: usize,
    /// Tempo analysis for wow/flutter detection
    tempo_analysis: Co2TempoAnalysis,
    /// Configuration
    config: Co2Config,
    /// Data file paths
    all_daily_path: PathBuf,
    /// Maximum delta between consecutive records (for scaling)
    max_delta: f32,
    /// Maximum seasonal anomaly delta (for year-over-year scaling)
    max_seasonal_delta: f32,
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
        }
    }
}

impl Co2Manager {
    /// Create a new CO2 manager
    pub fn new(config: Co2Config) -> Result<Self> {
        let data_dir = PathBuf::from(&config.data_dir);
        
        let mut manager = Self {
            records: Vec::new(),
            total_step_counter: 0,
            total_tick_counter: 0,

            days_per_year: 365, // Will be updated when data is loaded
            max_seasonal_delta: 1.0,
            tempo_analysis: Co2TempoAnalysis {
                wow_window: VecDeque::with_capacity(config.window_size),
                flutter_window: VecDeque::with_capacity(config.window_size),
                wow_average: 0.0,
                flutter_average: 0.0,
                wow_episodes: 0,
                flutter_episodes: 0,
                is_wow_stable: false,
                is_flutter_stable: false,
            },
            config,
            all_daily_path: data_dir.join("simon_says_seeq_web_data_co2_ppm_gml_noaa_gov_ccgg_all_daily.csv"),
            max_delta: 0.0,
        };
        
        if manager.config.enabled {
            // info!("🔍 CO2 Debug: Starting data loading process");
            // info!("🔍 CO2 Debug: Data directory: {:?}", manager.config.data_dir);
            // info!("🔍 CO2 Debug: All daily path: {:?}", manager.all_daily_path);
            
            match manager.load_data() {
                Ok(()) => {
                    // info!("CO2 data loaded successfully");
                    // info!("🔍 CO2 Debug: Records loaded: {}", manager.records.len());
                    if !manager.records.is_empty() {
                        let first = &manager.records[0];
                        let last = &manager.records[manager.records.len() - 1];
                        // info!("🔍 CO2 Debug: First record: {}/{}/{} = {:.2} ppm", first.year, first.month, first.day, first.co2_ppm);
                        // info!("🔍 CO2 Debug: Last record: {}/{}/{} = {:.2} ppm", last.year, last.month, last.day, last.co2_ppm);
                        // info!("⏱️  CO2 timing: 6 ticks per step (1 tick = 1 MIDI clock pulse)");
                    }
                }
                Err(e) => {
                    // warn!("CO2 data loading failed: {}", e);
                    // warn!("CO2 manager will continue without data");
                    // warn!("🔍 CO2 Debug: Records after failure: {}", manager.records.len());
                    // Continue with empty data - don't fail initialization
                }
            }
        } else {
            // info!("CO2 features disabled in configuration");
        }
        
        Ok(manager)
    }
    
    /// Load CO2 data from files
    pub fn load_data(&mut self) -> Result<()> {
        // Load all historical data
        self.load_all_daily_records()?;
        
        // Initialize counters
        self.reset_counters();
        
        Ok(())
    }

    
    /// Load all daily CO2 records from CSV
    fn load_all_daily_records(&mut self) -> Result<()> {
        // info!("🔍 CO2 Debug: Checking all daily file: {:?}", self.all_daily_path);
        // info!("🔍 CO2 Debug: File exists: {}", self.all_daily_path.exists());
        
        if !self.all_daily_path.exists() {
            // warn!("All daily CO2 file not found: {:?}", self.all_daily_path);
            return Ok(());
        }

        let metadata = fs::metadata(&self.all_daily_path)?;
        // info!("🔍 CO2 Debug: File size: {} bytes", metadata.len());

        let content = fs::read_to_string(&self.all_daily_path)?;
        let total_lines = content.lines().count();
        // info!("🔍 CO2 Debug: Total lines in file: {}", total_lines);
        
        let mut valid_records = 0;
        let mut invalid_records = 0;
        let mut empty_lines = 0;

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                empty_lines += 1;
                continue;
            }

            // Log first few lines for debugging
            if line_num < 5 {
                // info!("🔍 CO2 Debug: Line {}: {:?}", line_num + 1, line);
            }

            // Parse CSV line: year,month,day,decimal_date,co2_ppm
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if line_num < 5 {
                // info!("🔍 CO2 Debug: Parsed parts: {:?}", parts);
            }

            if parts.len() >= 5 {
                match self.parse_co2_record(&parts) {
                    Ok(record) => {
                        if valid_records < 3 {
                            // info!("🔍 CO2 Debug: Valid record {}: {}/{}/{} = {:.2} ppm", valid_records + 1, record.year, record.month, record.day, record.co2_ppm);
                        }
                        self.records.push(record);
                        valid_records += 1;
                    }
                    Err(e) => {
                        if invalid_records < 10 { // Only log first 10 errors
                            // warn!("🔍 CO2 Debug: Invalid CO2 record at line {}: {} - {}", line_num + 1, line, e);
                        }
                        invalid_records += 1;
                    }
                }
            } else {
                if invalid_records < 10 {
                    // warn!("🔍 CO2 Debug: Malformed CO2 record at line {} (parts: {}): {}", line_num + 1, parts.len(), line);
                }
                invalid_records += 1;
            }
        }

        // info!("🔍 CO2 Debug: Processing complete - Valid: {}, Invalid: {}, Empty: {}", valid_records, invalid_records, empty_lines);

        if valid_records > 0 {
            // info!("Loaded {} valid CO2 records (ignored {} invalid)", valid_records, invalid_records);
            // Calculate max delta for bipolar voltage scaling
            self.calculate_max_delta();
            // Calculate approximate days per year from data span
            self.calculate_days_per_year();
            // Calculate max seasonal delta for year-over-year scaling
            self.calculate_max_seasonal_delta();
        } else {
            // warn!("No valid CO2 records loaded from file");
            // warn!("🔍 CO2 Debug: This is why 'no data available' appears");
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
        // Always reset to 0 (0-indexed)
        self.total_step_counter = 0;
        self.total_tick_counter = 0;

        
        // Reset tempo analysis
        self.tempo_analysis.wow_window.clear();
        self.tempo_analysis.flutter_window.clear();
        self.tempo_analysis.wow_average = 0.0;
        self.tempo_analysis.flutter_average = 0.0;
        self.tempo_analysis.wow_episodes = 0;
        self.tempo_analysis.flutter_episodes = 0;
        self.tempo_analysis.is_wow_stable = true;
        self.tempo_analysis.is_flutter_stable = true;
        
        // debug!("CO2 counters reset");
    }
    
    /// Advance step counter and return current CO2 value
    pub fn advance_step(&mut self) -> Option<f32> {
        if self.records.is_empty() {
            return None;
        }
        
        let co2_value = self.records[self.total_step_counter].co2_ppm;
        self.total_step_counter = (self.total_step_counter + 1) % self.records.len();
        
        // debug!("Step CO2: {:.2} ppm (step counter {})", co2_value, self.total_step_counter);
        Some(co2_value)
    }
    
    /// Advance tick counter and return current CO2 value
    pub fn advance_tick(&mut self) -> Option<f32> {
        if self.records.is_empty() {
            return None;
        }
        
        let co2_value = self.records[self.total_tick_counter].co2_ppm;
        self.total_tick_counter = (self.total_tick_counter + 1) % self.records.len();
        
        Some(co2_value)
    }
    
    /// Get current step CO2 value without advancing
    pub fn get_current_step_co2(&self) -> Option<f32> {
        if self.records.is_empty() {
            return None;
        }
        
        Some(self.records[self.total_step_counter].co2_ppm)
    }
    
    /// Get current tick CO2 value without advancing
    pub fn get_current_tick_co2(&self) -> Option<f32> {
        if self.records.is_empty() {
            return None;
        }
        
        Some(self.records[self.total_tick_counter].co2_ppm)
    }

    
    /// Get CO2 voltage offset for CV output (scaled)
    pub fn get_co2_voltage_offset(&self, co2_value: f32) -> f32 {
        // Scale CO2 value to voltage range (318-800 ppm hardcoded)
        const CO2_MIN: f32 = 318.0; // NOAA data baseline (1958)
        const CO2_MAX: f32 = 800.0; // Extreme future scenario
        let normalized = (co2_value - CO2_MIN) / (CO2_MAX - CO2_MIN);
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

    /// Get current step counter (record index for step-based advancement)
    pub fn get_step_counter(&self) -> usize {
        self.total_step_counter
    }

    /// Get current tick counter (record index for tick-based advancement)
    pub fn get_tick_counter(&self) -> usize {
        self.total_tick_counter
    }



    /// Get step-based delta (current step to next step)
    pub fn get_step_delta(&self) -> f32 {
        if self.records.len() < 2 {
            return 0.0;
        }
        
        let current_value = self.records[self.total_step_counter].co2_ppm;
        let next_index = (self.total_step_counter + 1) % self.records.len();
        let next_value = self.records[next_index].co2_ppm;
        
        next_value - current_value
    }

    /// Get tick-based delta (current tick to next tick)  
    pub fn get_tick_delta(&self) -> f32 {
        if self.records.len() < 2 {
            return 0.0;
        }
        
        let current_value = self.records[self.total_tick_counter].co2_ppm;
        let next_index = (self.total_tick_counter + 1) % self.records.len();
        let next_value = self.records[next_index].co2_ppm;
        
        next_value - current_value
    }



    /// Get seasonal anomaly delta (current value vs same time last year)
    pub fn get_seasonal_anomaly_delta(&self) -> f32 {
        if self.records.len() < self.days_per_year {
            return 0.0; // Not enough data for year-over-year comparison
        }
        
        let current_index = self.total_step_counter;
        
        // Calculate year-ago index with bounds checking
        if current_index >= self.days_per_year {
            let year_ago_index = current_index - self.days_per_year;
            let current_value = self.records[current_index].co2_ppm;
            let year_ago_value = self.records[year_ago_index].co2_ppm;
            current_value - year_ago_value
        } else {
            0.0 // Not enough historical data
        }
    }

    /// Get seasonal anomaly delta scaled to bipolar voltage (-5V to +5V)
    pub fn get_seasonal_anomaly_voltage(&self) -> f32 {
        if self.max_seasonal_delta == 0.0 {
            return 0.0;
        }
        
        let delta = self.get_seasonal_anomaly_delta();
        // Scale to -5V to +5V range using seasonal-specific scaling
        (delta / self.max_seasonal_delta) * 5.0
    }

    /// Get step delta scaled to bipolar voltage (-5V to +5V)
    pub fn get_step_delta_voltage(&self) -> f32 {
        if self.max_delta == 0.0 {
            return 0.0;
        }
        
        let delta = self.get_step_delta();
        // Scale to -5V to +5V range
        (delta / self.max_delta) * 5.0
    }

    /// Get tick delta scaled to bipolar voltage (-5V to +5V)
    pub fn get_tick_delta_voltage(&self) -> f32 {
        if self.max_delta == 0.0 {
            return 0.0;
        }
        
        let delta = self.get_tick_delta();
        // Scale to -5V to +5V range  
        (delta / self.max_delta) * 5.0
    }



    /// Calculate maximum delta between consecutive records for scaling
    fn calculate_max_delta(&mut self) {
        if self.records.len() < 2 {
            self.max_delta = 1.0; // Avoid division by zero
            return;
        }

        let mut max_delta = 0.0_f32;
        
        for i in 0..self.records.len() {
            let current_value = self.records[i].co2_ppm;
            let next_index = (i + 1) % self.records.len();
            let next_value = self.records[next_index].co2_ppm;
            let delta = (next_value - current_value).abs();
            
            if delta > max_delta {
                max_delta = delta;
            }
        }
        
        self.max_delta = max_delta.max(0.1); // Ensure minimum value to avoid division issues
        // debug!("Calculated max CO2 delta: {:.3} ppm", self.max_delta);
    }

    /// Calculate approximate days per year from the data span
    fn calculate_days_per_year(&mut self) {
        if self.records.len() < 2 {
            return;
        }

        let first_record = &self.records[0];
        let last_record = &self.records[self.records.len() - 1];
        
        // Calculate year difference
        let year_diff = last_record.year as f32 - first_record.year as f32;
        if year_diff > 0.0 {
            self.days_per_year = (self.records.len() as f32 / year_diff) as usize;
            // Clamp to reasonable bounds (360-370 days to handle leap years and data gaps)
            self.days_per_year = self.days_per_year.clamp(360, 370);
            // debug!("Calculated days per year: {} (from {} years of data)", self.days_per_year, year_diff);
        }
    }

    /// Calculate maximum seasonal anomaly delta for proper scaling
    fn calculate_max_seasonal_delta(&mut self) {
        if self.records.len() < self.days_per_year {
            self.max_seasonal_delta = 1.0; // Not enough data
            return;
        }

        let mut max_seasonal_delta = 0.0_f32;
        
        // Check all possible year-over-year comparisons
        for i in self.days_per_year..self.records.len() {
            let current_value = self.records[i].co2_ppm;
            let year_ago_value = self.records[i - self.days_per_year].co2_ppm;
            let seasonal_delta = (current_value - year_ago_value).abs();
            
            if seasonal_delta > max_seasonal_delta {
                max_seasonal_delta = seasonal_delta;
            }
        }
        
        self.max_seasonal_delta = max_seasonal_delta.max(0.1); // Ensure minimum value
        // debug!("Calculated max seasonal anomaly delta: {:.3} ppm", self.max_seasonal_delta);
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
                // debug!("Wow episode detected: deviation {:.2} BPM", wow_deviation);
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
                // debug!("Flutter episode detected: deviation {:.2} BPM", flutter_deviation);
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
        let has_data = !self.records.is_empty();
        if !has_data {
            // info!("🔍 CO2 Debug: has_data() returning false - records.len() = {}", self.records.len());
        }
        has_data
    }
    
    /// Get CO2 status string for display
    pub fn get_status_string(&self) -> String {
        if let Some(latest_record) = self.records.last() {
            format!("CO2: {:.2} ppm", latest_record.co2_ppm)
        } else {
            "CO2: UNKNOWN".to_string()
        }
    }
    
    /// Get detailed CO2 information
    /// Get CO2 information for display/debugging
    pub fn get_info(&self) -> Co2Info {
        Co2Info {
            enabled: self.config.enabled,
            has_data: self.has_data(),
            record_count: self.get_record_count(),
            latest_daily_value: self.records.last().map(|r| r.co2_ppm),
            current_step_value: self.get_current_step_co2(),
            current_tick_value: self.get_current_tick_co2(),
            total_step_counter: self.total_step_counter,
            total_tick_counter: self.total_tick_counter,

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
        // info!("Reloading CO2 data");
        self.records.clear();
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
    pub total_step_counter: usize,
    pub total_tick_counter: usize,

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