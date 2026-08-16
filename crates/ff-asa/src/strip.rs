//! ASA strip/restore engine.
//!
//! Provides transparent removal of column 1 ASA control characters during
//! editing, with restoration on save. The `AsaControlMap` preserves the
//! original control characters in a parallel data structure.

use crate::control::AsaControl;

/// Parallel metadata structure preserving original ASA control characters
/// when column 1 has been stripped for editing.
///
/// Keyed by 0-based document line number.
// Validates: Requirement 7.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsaControlMap {
    /// Map from 0-based document line number to original ASA control character.
    entries: Vec<AsaControl>,
}

impl AsaControlMap {
    /// Create from a document by extracting column 1 of each line.
    // Validates: Requirement 7.1, 7.2
    pub fn from_lines(lines: &[&str]) -> Self {
        let entries = lines
            .iter()
            .map(|line| {
                let first_char = line.chars().next().unwrap_or(' ');
                AsaControl::from_char(first_char)
            })
            .collect();
        Self { entries }
    }

    /// Create from a pre-built list of controls.
    pub fn from_controls(controls: Vec<AsaControl>) -> Self {
        Self { entries: controls }
    }

    /// Get the control character for a given line.
    pub fn get(&self, line: usize) -> Option<AsaControl> {
        self.entries.get(line).copied()
    }

    /// Insert a new entry at line position (for line insertion during edit).
    ///
    /// Defaults to `AsaControl::Space` per Requirement 7.4.
    // Validates: Requirement 7.4
    pub fn insert_line(&mut self, line: usize) {
        let line = line.min(self.entries.len());
        self.entries.insert(line, AsaControl::Space);
    }

    /// Remove an entry at line position (for line deletion during edit).
    // Validates: Requirement 7.5
    pub fn remove_line(&mut self, line: usize) {
        if line < self.entries.len() {
            self.entries.remove(line);
        }
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all controls as a slice.
    pub fn controls(&self) -> &[AsaControl] {
        &self.entries
    }
}

/// Strip ASA control characters from column 1 of all lines.
///
/// Returns the modified lines (with column 1 removed) and the control map
/// for restoration.
// Validates: Requirement 7.1, 7.2
pub fn strip_asa(lines: &[&str]) -> (Vec<String>, AsaControlMap) {
    let control_map = AsaControlMap::from_lines(lines);
    let stripped: Vec<String> = lines
        .iter()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                line.chars().skip(1).collect()
            }
        })
        .collect();
    (stripped, control_map)
}

/// Restore ASA control characters to column 1 using the control map.
///
/// Returns lines with the original control characters prepended.
// Validates: Requirement 7.3
pub fn restore_asa(lines: &[&str], control_map: &AsaControlMap) -> Vec<String> {
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let control = control_map.get(i).unwrap_or(AsaControl::Space);
            format!("{}{}", control.to_char(), line)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Validates: Requirement 7.1
    fn strip_removes_column_1() {
        let lines = vec![" DATA LINE", "0DOUBLE", "1PAGE"];
        let (stripped, _map) = strip_asa(&lines);
        assert_eq!(stripped[0], "DATA LINE");
        assert_eq!(stripped[1], "DOUBLE");
        assert_eq!(stripped[2], "PAGE");
    }

    #[test]
    // Validates: Requirement 7.2
    fn strip_preserves_controls_in_map() {
        let lines = vec![" DATA", "0DOUBLE", "-TRIPLE", "1PAGE", "+OVER", "HHALT"];
        let (_stripped, map) = strip_asa(&lines);
        assert_eq!(map.get(0), Some(AsaControl::Space));
        assert_eq!(map.get(1), Some(AsaControl::DoubleSpace));
        assert_eq!(map.get(2), Some(AsaControl::TripleSpace));
        assert_eq!(map.get(3), Some(AsaControl::PageEject));
        assert_eq!(map.get(4), Some(AsaControl::Overstrike));
        assert_eq!(map.get(5), Some(AsaControl::Halt));
    }

    #[test]
    // Validates: Requirement 7.3
    fn restore_prepends_control_characters() {
        let lines = vec![" DATA", "0DOUBLE", "1PAGE"];
        let (stripped, map) = strip_asa(&lines);
        let stripped_refs: Vec<&str> = stripped.iter().map(|s| s.as_str()).collect();
        let restored = restore_asa(&stripped_refs, &map);
        assert_eq!(restored[0], " DATA");
        assert_eq!(restored[1], "0DOUBLE");
        assert_eq!(restored[2], "1PAGE");
    }

    #[test]
    // Validates: Requirement 7.1, 7.3
    fn strip_restore_round_trip_is_identity() {
        let lines = vec![" LINE 1", "0LINE 2", "-LINE 3", "1LINE 4", "+LINE 5"];
        let (stripped, map) = strip_asa(&lines);
        let stripped_refs: Vec<&str> = stripped.iter().map(|s| s.as_str()).collect();
        let restored = restore_asa(&stripped_refs, &map);
        let original: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        assert_eq!(restored, original);
    }

    #[test]
    // Validates: Requirement 7.4
    fn insert_line_defaults_to_space() {
        let lines = vec![" DATA", "1PAGE"];
        let (_stripped, mut map) = strip_asa(&lines);
        map.insert_line(1);
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(1), Some(AsaControl::Space));
        // Original entries shifted
        assert_eq!(map.get(2), Some(AsaControl::PageEject));
    }

    #[test]
    // Validates: Requirement 7.5
    fn remove_line_deletes_entry() {
        let lines = vec![" DATA", "0DOUBLE", "1PAGE"];
        let (_stripped, mut map) = strip_asa(&lines);
        map.remove_line(1);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(0), Some(AsaControl::Space));
        assert_eq!(map.get(1), Some(AsaControl::PageEject));
    }

    #[test]
    fn strip_handles_empty_lines() {
        let lines: Vec<&str> = vec!["", " DATA"];
        let (stripped, map) = strip_asa(&lines);
        assert_eq!(stripped[0], "");
        assert_eq!(stripped[1], "DATA");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn control_map_from_controls() {
        let controls = vec![AsaControl::Space, AsaControl::PageEject];
        let map = AsaControlMap::from_controls(controls);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(0), Some(AsaControl::Space));
        assert_eq!(map.get(1), Some(AsaControl::PageEject));
    }
}
