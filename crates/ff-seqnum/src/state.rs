//! Per-document sequence number state tracking.
//!
//! Manages the side-table of original stripped values, detection state,
//! NUMBER SHOW mode, and auto-numbering mode for each open document.

use std::collections::HashMap;

use crate::detector::FullDetectionResult;
use crate::types::{ColumnRange, DetectionResult, SequenceFormat};

/// Per-document state tracking for sequence number processing.
#[derive(Debug, Clone)]
pub struct SeqNumState {
    /// The detection result from file open (or re-detection).
    pub detection: Option<FullDetectionResult>,
    /// The front column range that was stripped (if any).
    pub stripped_front: Option<ColumnRange>,
    /// The back column range that was stripped (if any).
    pub stripped_back: Option<ColumnRange>,
    /// Side-table storing original values per line.
    pub side_table: SideTable,
    /// Whether NUMBER SHOW overlay mode is active.
    pub number_show_active: bool,
    /// Whether NUMBER ON auto-numbering mode is active.
    pub auto_numbering_active: bool,
    /// The current auto-numbering state.
    pub auto_number_state: Option<AutoNumberState>,
}

impl SeqNumState {
    /// Create a new empty state.
    pub fn new() -> Self {
        Self {
            detection: None,
            stripped_front: None,
            stripped_back: None,
            side_table: SideTable::new(),
            number_show_active: false,
            auto_numbering_active: false,
            auto_number_state: None,
        }
    }

    /// Returns the status indicator for the status bar.
    pub fn status_indicator(&self) -> SeqNumStatusIndicator {
        if self.number_show_active {
            return SeqNumStatusIndicator::ShowMode;
        }
        if self.stripped_front.is_some() || self.stripped_back.is_some() {
            return SeqNumStatusIndicator::Stripped {
                has_front: self.stripped_front.is_some(),
                has_back: self.stripped_back.is_some(),
            };
        }
        if let Some(ref detection) = self.detection {
            if detection.front == DetectionResult::Present
                || detection.back == DetectionResult::Present
            {
                return SeqNumStatusIndicator::DetectedNotStripped;
            }
        }
        SeqNumStatusIndicator::None
    }

    /// Returns the active sequence column ranges.
    pub fn active_columns(&self) -> (Option<ColumnRange>, Option<ColumnRange>) {
        (self.stripped_front, self.stripped_back)
    }
}

impl Default for SeqNumState {
    fn default() -> Self {
        Self::new()
    }
}

/// Side-table storing original sequence number column content stripped from the edit buffer.
///
/// Enables NUMBER SHOW overlay rendering and `restore_on_save` functionality.
#[derive(Debug, Clone)]
pub struct SideTable {
    /// Entries keyed by 0-based line index.
    entries: HashMap<usize, SideTableEntry>,
}

/// A single line's original sequence number content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideTableEntry {
    /// Original front column content (if stripped).
    pub front_content: Option<String>,
    /// Original back column content (if stripped).
    pub back_content: Option<String>,
}

impl SideTable {
    /// Create a new empty side-table.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Store stripped values for a line.
    pub fn store_stripped_values(
        &mut self,
        line_index: usize,
        front: Option<&str>,
        back: Option<&str>,
    ) {
        let entry = self
            .entries
            .entry(line_index)
            .or_insert_with(|| SideTableEntry {
                front_content: None,
                back_content: None,
            });
        if let Some(f) = front {
            entry.front_content = Some(f.to_string());
        }
        if let Some(b) = back {
            entry.back_content = Some(b.to_string());
        }
    }

    /// Retrieve stored values for a line.
    pub fn get_original_values(&self, line_index: usize) -> Option<&SideTableEntry> {
        self.entries.get(&line_index)
    }

    /// Returns the number of lines with stored entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if no entries are stored.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all stored entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Adjust line indices after lines are inserted at the given index.
    pub fn on_lines_inserted(&mut self, at_index: usize, count: usize) {
        let mut new_entries = HashMap::new();
        for (idx, entry) in self.entries.drain() {
            if idx >= at_index {
                new_entries.insert(idx + count, entry);
            } else {
                new_entries.insert(idx, entry);
            }
        }
        self.entries = new_entries;
    }

    /// Adjust line indices after lines are deleted starting at the given index.
    pub fn on_lines_deleted(&mut self, at_index: usize, count: usize) {
        let mut new_entries = HashMap::new();
        for (idx, entry) in self.entries.drain() {
            if idx >= at_index + count {
                new_entries.insert(idx - count, entry);
            } else if idx < at_index {
                new_entries.insert(idx, entry);
            }
            // Entries in the deleted range are discarded
        }
        self.entries = new_entries;
    }

    /// Returns an iterator over all entries.
    pub fn iter(&self) -> impl Iterator<Item = (&usize, &SideTableEntry)> {
        self.entries.iter()
    }
}

impl Default for SideTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Auto-numbering state for NUMBER ON mode.
#[derive(Debug, Clone)]
pub struct AutoNumberState {
    /// The next sequence value to assign.
    pub next_value: u64,
    /// The increment between values.
    pub increment: u64,
    /// The target column range.
    pub target_columns: ColumnRange,
    /// The format to use for generated numbers.
    pub format: SequenceFormat,
}

/// The status indicator displayed in the status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeqNumStatusIndicator {
    /// No sequence columns detected or defined — no indicator shown.
    None,
    /// Sequence numbers detected and stripped.
    Stripped { has_front: bool, has_back: bool },
    /// Sequence numbers detected but NOT stripped.
    DetectedNotStripped,
    /// NUMBER SHOW overlay is active.
    ShowMode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_table_store_and_retrieve() {
        // Validates: Requirement 3.9
        let mut table = SideTable::new();
        table.store_stripped_values(0, Some("000100"), Some("00000100"));
        table.store_stripped_values(1, Some("000200"), None);

        let entry0 = table.get_original_values(0).unwrap();
        assert_eq!(entry0.front_content.as_deref(), Some("000100"));
        assert_eq!(entry0.back_content.as_deref(), Some("00000100"));

        let entry1 = table.get_original_values(1).unwrap();
        assert_eq!(entry1.front_content.as_deref(), Some("000200"));
        assert_eq!(entry1.back_content, None);
    }

    #[test]
    fn side_table_clear() {
        // Validates: Requirement 3.9
        let mut table = SideTable::new();
        table.store_stripped_values(0, Some("000100"), None);
        assert!(!table.is_empty());
        table.clear();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn side_table_on_lines_inserted() {
        // Validates: Side-table maintenance on insert
        let mut table = SideTable::new();
        table.store_stripped_values(0, Some("A"), None);
        table.store_stripped_values(1, Some("B"), None);
        table.store_stripped_values(2, Some("C"), None);

        table.on_lines_inserted(1, 2);

        assert_eq!(
            table
                .get_original_values(0)
                .unwrap()
                .front_content
                .as_deref(),
            Some("A")
        );
        assert!(table.get_original_values(1).is_none());
        assert!(table.get_original_values(2).is_none());
        assert_eq!(
            table
                .get_original_values(3)
                .unwrap()
                .front_content
                .as_deref(),
            Some("B")
        );
        assert_eq!(
            table
                .get_original_values(4)
                .unwrap()
                .front_content
                .as_deref(),
            Some("C")
        );
    }

    #[test]
    fn side_table_on_lines_deleted() {
        // Validates: Side-table maintenance on delete
        let mut table = SideTable::new();
        table.store_stripped_values(0, Some("A"), None);
        table.store_stripped_values(1, Some("B"), None);
        table.store_stripped_values(2, Some("C"), None);
        table.store_stripped_values(3, Some("D"), None);

        table.on_lines_deleted(1, 2); // Delete lines 1 and 2

        assert_eq!(
            table
                .get_original_values(0)
                .unwrap()
                .front_content
                .as_deref(),
            Some("A")
        );
        assert!(table.get_original_values(1).is_some()); // was 3, now 1
        assert_eq!(
            table
                .get_original_values(1)
                .unwrap()
                .front_content
                .as_deref(),
            Some("D")
        );
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn status_indicator_none_when_no_detection() {
        // Validates: Requirement 4.1
        let state = SeqNumState::new();
        assert_eq!(state.status_indicator(), SeqNumStatusIndicator::None);
    }

    #[test]
    fn status_indicator_stripped() {
        // Validates: Requirement 4.1
        let mut state = SeqNumState::new();
        state.stripped_front = Some(ColumnRange::new(1, 6).unwrap());
        state.stripped_back = Some(ColumnRange::new(73, 80).unwrap());
        assert_eq!(
            state.status_indicator(),
            SeqNumStatusIndicator::Stripped {
                has_front: true,
                has_back: true
            }
        );
    }

    #[test]
    fn status_indicator_show_mode_takes_precedence() {
        // Validates: Requirement 4.4
        let mut state = SeqNumState::new();
        state.stripped_front = Some(ColumnRange::new(1, 6).unwrap());
        state.number_show_active = true;
        assert_eq!(state.status_indicator(), SeqNumStatusIndicator::ShowMode);
    }

    #[test]
    fn status_indicator_detected_not_stripped() {
        // Validates: Requirement 4.2
        let mut state = SeqNumState::new();
        state.detection = Some(FullDetectionResult {
            front: DetectionResult::Present,
            back: DetectionResult::Absent,
            front_columns: Some(ColumnRange::new(1, 6).unwrap()),
            back_columns: None,
            front_format: None,
            back_format: None,
            lines_sampled: 10,
        });
        assert_eq!(
            state.status_indicator(),
            SeqNumStatusIndicator::DetectedNotStripped
        );
    }
}
