//! Version information for SimonSaysSeeq
//!
//! This module provides version information read from the VERSION file.

use std::fs;

/// Get the version number only from the VERSION file
pub fn get_version() -> String {
    match fs::read_to_string("VERSION") {
        Ok(content) => {
            let trimmed = content.trim();
            // Split on " - " and take first part (version number)
            if let Some(version_part) = trimmed.split(" - ").next() {
                version_part.to_string()
            } else {
                trimmed.to_string()
            }
        },
        Err(_) => "unknown".to_string(),
    }
}

/// Get the description part from the VERSION file
pub fn get_version_description() -> String {
    match fs::read_to_string("VERSION") {
        Ok(content) => {
            let trimmed = content.trim();
            // Split on " - " and take second part (description)
            if let Some(description_part) = trimmed.split(" - ").nth(1) {
                description_part.to_string()
            } else {
                "".to_string()
            }
        },
        Err(_) => "".to_string(),
    }
}

/// Get the full version line from the VERSION file
pub fn get_full_version() -> String {
    match fs::read_to_string("VERSION") {
        Ok(content) => content.trim().to_string(),
        Err(_) => "unknown".to_string(),
    }
}

/// Get a formatted version string for display
pub fn get_version_string() -> String {
    format!("SimonSaysSeeq v{}", get_version())
}

/// Get version info with additional details
pub fn get_version_info() -> String {
    let version = get_version();
    let description = get_version_description();
    if description.is_empty() {
        format!("SimonSaysSeeq Rust v{} - CO2 Environmental Data Sonification System", version)
    } else {
        format!("SimonSaysSeeq Rust v{} - {}", version, description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_functions() {
        // These tests will pass regardless of VERSION file contents
        let version = get_version();
        let description = get_version_description();
        let full_version = get_full_version();
        let version_string = get_version_string();
        let version_info = get_version_info();
        
        assert!(!version.is_empty());
        assert!(version_string.contains("SimonSaysSeeq"));
        assert!(version_info.contains("SimonSaysSeeq"));
        assert!(!full_version.is_empty());
        // Description can be empty, so no assertion needed
    }
}