//! Control button press counts per jam, to see over time which control buttons are used.
//!
//! Every press of a control-row button is counted. When the pattern is saved (on each clock
//! stop, i.e. after each jam) one line is added to `control_button_counts.jsonl` with the date
//! and that jam's presses, and the counts start again from zero.

// `anyhow::Result` is a Result whose error can be any error type
use anyhow::Result;
// Derive macro for turning a struct into JSON
use serde::Serialize;
// BTreeMap keeps its keys sorted, so each line lists buttons in column order
use std::collections::BTreeMap;
// `Write` gives files the `writeln!` method
use std::io::Write;
// File paths
use std::path::{Path, PathBuf};

/// One line of the file: one jam
// Borrowed fields (`&'a`) so writing a line doesn't copy the counts
#[derive(Serialize)]
struct JamRow<'a> {
    /// When the jam ended (the clock stopped), UTC
    date: String,
    /// Presses by button name, e.g. "grid one col 08: last step, straight"
    presses: &'a BTreeMap<String, u64>,
}

/// Presses counted since the last saved line
#[derive(Debug, Default)]
pub struct ButtonStats {
    /// Presses by button name for the current jam
    pub presses: BTreeMap<String, u64>,
}

impl ButtonStats {
    /// The file, next to current_pattern.json in the working directory
    pub fn default_path() -> PathBuf {
        // A relative path: resolved against the service's working directory
        PathBuf::from("control_button_counts.jsonl")
    }

    /// Count one press of the button called `name`
    pub fn record_press(&mut self, name: &str) {
        // `entry(..).or_insert(0)` starts at 0 the first time, then we add one
        *self.presses.entry(name.to_string()).or_insert(0) += 1;
    }

    /// Add one line for this jam to the end of `path` (creating the file if needed),
    /// then start counting the next jam from zero
    pub fn append_jam(&mut self, path: &Path) -> Result<()> {
        // The line's content: the date now and this jam's presses
        let row = JamRow { date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(), presses: &self.presses };
        // Open for appending; `create(true)` makes the file the first time
        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
        // One JSON object on one line
        writeln!(file, "{}", serde_json::to_string(&row)?)?;
        // Next jam starts at zero
        self.presses.clear();
        // Success
        Ok(())
    }
}

/// Name of a grid two, row 7 button by column, for the counts file
pub fn grid_two_button_name(column: usize) -> String {
    // What each column does today
    let label = match column {
        // Clock light, and first half of the re-detect pair
        0 => "MIDI re-detect (with 1) / clock light",
        // Second half of the re-detect pair
        1 => "MIDI re-detect (with 0)",
        // The three Crow search buttons
        4 | 5 | 6 => "Crow search (4+5+6)",
        // Crow output mutes 1-4
        7..=10 => "Crow mute",
        // Transport
        12 => "Stop",
        // Transport
        13 => "Start",
        // Tempo buttons do nothing with an external clock
        14 | 15 => "Tempo (no effect)",
        // Columns with nothing on them
        _ => "unused",
    };
    // `{:02}` pads the column to two digits so each line sorts in column order
    format!("grid two col {:02}: {}", column, label)
}

#[cfg(test)]
mod tests {
    // Everything from the module above
    use super::*;

    #[test]
    fn each_jam_adds_one_line_and_counting_restarts() {
        // A file in the system temp directory, unique to this test run
        let path = std::env::temp_dir().join(format!("button_stats_test_{}.jsonl", std::process::id()));
        // Start from no file
        let _ = std::fs::remove_file(&path);
        // First jam: two undos, one redo
        let mut stats = ButtonStats::default();
        stats.record_press("Undo");
        stats.record_press("Undo");
        stats.record_press("Redo");
        stats.append_jam(&path).unwrap();
        // Second jam: one undo
        stats.record_press("Undo");
        stats.append_jam(&path).unwrap();
        // Two lines, one per jam
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        // Each line is JSON; `[..]` indexes into it
        let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        let second: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        // The first jam's counts
        assert_eq!(first["presses"]["Undo"], 2);
        assert_eq!(first["presses"]["Redo"], 1);
        // The second jam counted from zero
        assert_eq!(second["presses"]["Undo"], 1);
        assert!(second["presses"].get("Redo").is_none());
        // Each line has a date
        assert!(first["date"].as_str().unwrap().ends_with("UTC"));
        // Tidy up
        let _ = std::fs::remove_file(&path);
    }
}
