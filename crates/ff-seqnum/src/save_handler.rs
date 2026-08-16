//! Save-time preservation and restoration of sequence numbers.
//!
//! Handles the `restore_on_save` configuration option which restores
//! original sequence numbers from the side-table into save output.

use crate::config::SeqNumConfig;
use crate::state::SeqNumState;
use crate::traits::DocumentAccess;
use crate::types::ColumnRange;

/// The decision on how to handle save content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveContentDecision {
    /// Save the edit buffer content as-is (sequence columns contain spaces).
    SaveAsIs,
    /// Restore sequence numbers before saving.
    RestoreAndSave {
        /// Lines to modify with restored content.
        restorations: Vec<LineRestoration>,
    },
}

/// A single line restoration entry for save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineRestoration {
    /// The 0-based line index.
    pub line_index: usize,
    /// Content to insert at the front column range.
    pub front_content: Option<String>,
    /// Content to insert at the back column range.
    pub back_content: Option<String>,
}

/// Determine the save content decision based on state and config.
///
/// When `restore_on_save` is true and stripping occurred, the side-table
/// content is restored into the save output without modifying the edit buffer.
pub fn prepare_save_content(
    _document: &dyn DocumentAccess,
    state: &SeqNumState,
    config: &SeqNumConfig,
) -> SaveContentDecision {
    // Default behaviour: save as-is
    if !config.restore_on_save {
        return SaveContentDecision::SaveAsIs;
    }

    // If nothing was stripped, save as-is
    if state.stripped_front.is_none() && state.stripped_back.is_none() {
        return SaveContentDecision::SaveAsIs;
    }

    // If side-table is empty, nothing to restore
    if state.side_table.is_empty() {
        return SaveContentDecision::SaveAsIs;
    }

    // Build restorations from side-table
    let mut restorations = Vec::new();
    for (&line_idx, entry) in state.side_table.iter() {
        let restoration = LineRestoration {
            line_index: line_idx,
            front_content: entry.front_content.clone(),
            back_content: entry.back_content.clone(),
        };
        restorations.push(restoration);
    }
    restorations.sort_by_key(|r| r.line_index);

    SaveContentDecision::RestoreAndSave { restorations }
}

/// Apply restorations to a line content string for save output.
///
/// Does not modify the edit buffer — produces a new string for the save pipeline.
pub fn apply_restoration_to_line(
    line: &str,
    restoration: &LineRestoration,
    front_range: Option<&ColumnRange>,
    back_range: Option<&ColumnRange>,
) -> String {
    let mut result = line.to_string();

    if let (Some(content), Some(range)) = (&restoration.front_content, front_range) {
        result = replace_in_string(&result, range, content);
    }
    if let (Some(content), Some(range)) = (&restoration.back_content, back_range) {
        result = replace_in_string(&result, range, content);
    }

    result
}

/// Replace a column range in a string with new content.
fn replace_in_string(line: &str, range: &ColumnRange, content: &str) -> String {
    let start = range.start_offset();
    let end = range.end_offset();

    if line.len() <= start {
        // Pad and append
        let mut result = line.to_string();
        result.push_str(&" ".repeat(start - line.len()));
        result.push_str(content);
        return result;
    }

    let actual_end = end.min(line.len());
    let mut result = String::with_capacity(line.len());
    result.push_str(&line[..start]);
    result.push_str(content);
    if actual_end < line.len() {
        result.push_str(&line[actual_end..]);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDoc {
        lines: Vec<String>,
    }

    impl DocumentAccess for MockDoc {
        fn line_count(&self) -> usize {
            self.lines.len()
        }
        fn line_content(&self, index: usize) -> Option<&str> {
            self.lines.get(index).map(|s| s.as_str())
        }
    }

    #[test]
    fn default_save_as_is_when_restore_off() {
        // Validates: Requirements 11.1, 11.2
        let doc = MockDoc {
            lines: vec!["test".to_string()],
        };
        let state = SeqNumState::new();
        let config = SeqNumConfig::default(); // restore_on_save = false

        let decision = prepare_save_content(&doc, &state, &config);
        assert_eq!(decision, SaveContentDecision::SaveAsIs);
    }

    #[test]
    fn restore_on_save_with_side_table() {
        // Validates: Requirement 11.5
        let doc = MockDoc {
            lines: vec!["      CODE                                                                        ".to_string()],
        };
        let mut state = SeqNumState::new();
        state.stripped_front = Some(ColumnRange::new(1, 6).unwrap());
        state
            .side_table
            .store_stripped_values(0, Some("000100"), None);

        let mut config = SeqNumConfig::default();
        config.restore_on_save = true;

        let decision = prepare_save_content(&doc, &state, &config);
        match decision {
            SaveContentDecision::RestoreAndSave { restorations } => {
                assert_eq!(restorations.len(), 1);
                assert_eq!(restorations[0].line_index, 0);
                assert_eq!(restorations[0].front_content.as_deref(), Some("000100"));
            }
            _ => panic!("Expected RestoreAndSave"),
        }
    }

    #[test]
    fn save_as_is_when_nothing_stripped() {
        // Validates: Requirement 11.1
        let doc = MockDoc {
            lines: vec!["test".to_string()],
        };
        let state = SeqNumState::new();
        let mut config = SeqNumConfig::default();
        config.restore_on_save = true;

        let decision = prepare_save_content(&doc, &state, &config);
        assert_eq!(decision, SaveContentDecision::SaveAsIs);
    }

    #[test]
    fn apply_restoration_to_line_front() {
        // Validates: Requirement 11.5
        let line = "      CODE HERE";
        let restoration = LineRestoration {
            line_index: 0,
            front_content: Some("000100".to_string()),
            back_content: None,
        };
        let front_range = ColumnRange::new(1, 6).unwrap();

        let result = apply_restoration_to_line(line, &restoration, Some(&front_range), None);
        assert!(result.starts_with("000100"));
        assert!(result.contains("CODE HERE"));
    }

    #[test]
    fn apply_restoration_preserves_remaining_content() {
        // Validates: Requirement 11.5
        let line = "      MOVE A TO B.                                                          ";
        let restoration = LineRestoration {
            line_index: 0,
            front_content: Some("000100".to_string()),
            back_content: None,
        };
        let front_range = ColumnRange::new(1, 6).unwrap();

        let result = apply_restoration_to_line(line, &restoration, Some(&front_range), None);
        assert_eq!(&result[..6], "000100");
        assert_eq!(&result[6..], &line[6..]);
    }
}
