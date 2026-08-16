//! Undo/Redo integration for sequence number operations.
//!
//! Provides helpers for wrapping UNNUM and NUMBER operations in
//! Sequence_Transactions via the UndoRecorder trait.

use crate::traits::UndoRecorder;
use crate::types::ColumnRange;

/// A recorded column change for undo purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnChange {
    /// Line index where the change occurred.
    pub line_index: usize,
    /// The column range that was modified.
    pub range: ColumnRange,
    /// The original content before modification.
    pub original_content: String,
}

/// Record an UNNUM operation in the undo system.
///
/// Wraps all line modifications in a single Sequence_Transaction.
pub fn record_unnum_transaction(recorder: &mut dyn UndoRecorder, changes: &[ColumnChange]) {
    recorder.begin_sequence_transaction("UNNUM");
    for change in changes {
        recorder.record_column_change(
            change.line_index,
            &change.range,
            change.original_content.clone(),
        );
    }
    recorder.commit();
}

/// Record a NUMBER operation in the undo system.
///
/// Wraps all line modifications in a single Sequence_Transaction.
pub fn record_number_transaction(recorder: &mut dyn UndoRecorder, changes: &[ColumnChange]) {
    recorder.begin_sequence_transaction("NUMBER");
    for change in changes {
        recorder.record_column_change(
            change.line_index,
            &change.range,
            change.original_content.clone(),
        );
    }
    recorder.commit();
}

/// Check whether an operation should be recorded in the undo stack.
///
/// Auto-strip on file open is NOT recorded (Requirement 9.3).
/// NUMBER SHOW toggle is NOT recorded (Requirement 8.6).
/// All UNNUM and NUMBER commands ARE recorded.
pub fn should_record_undo(is_auto_strip: bool, is_show_toggle: bool) -> bool {
    !is_auto_strip && !is_show_toggle
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRecorder {
        began: bool,
        changes: Vec<(usize, ColumnRange, String)>,
        committed: bool,
        aborted: bool,
        description: String,
    }

    impl MockRecorder {
        fn new() -> Self {
            Self {
                began: false,
                changes: Vec::new(),
                committed: false,
                aborted: false,
                description: String::new(),
            }
        }
    }

    impl UndoRecorder for MockRecorder {
        fn begin_sequence_transaction(&mut self, description: &str) {
            self.began = true;
            self.description = description.to_string();
        }

        fn record_column_change(
            &mut self,
            line_index: usize,
            range: &ColumnRange,
            original_content: String,
        ) {
            self.changes.push((line_index, *range, original_content));
        }

        fn commit(&mut self) {
            self.committed = true;
        }

        fn abort(&mut self) {
            self.aborted = true;
        }
    }

    #[test]
    fn record_unnum_transaction_wraps_all_changes() {
        // Validates: Requirement 9.1
        let mut recorder = MockRecorder::new();
        let changes = vec![
            ColumnChange {
                line_index: 0,
                range: ColumnRange::new(1, 6).unwrap(),
                original_content: "000100".to_string(),
            },
            ColumnChange {
                line_index: 1,
                range: ColumnRange::new(1, 6).unwrap(),
                original_content: "000200".to_string(),
            },
        ];

        record_unnum_transaction(&mut recorder, &changes);

        assert!(recorder.began);
        assert_eq!(recorder.description, "UNNUM");
        assert_eq!(recorder.changes.len(), 2);
        assert!(recorder.committed);
        assert!(!recorder.aborted);
    }

    #[test]
    fn record_number_transaction_wraps_all_changes() {
        // Validates: Requirement 9.2
        let mut recorder = MockRecorder::new();
        let changes = vec![ColumnChange {
            line_index: 0,
            range: ColumnRange::new(73, 80).unwrap(),
            original_content: "        ".to_string(),
        }];

        record_number_transaction(&mut recorder, &changes);

        assert!(recorder.began);
        assert_eq!(recorder.description, "NUMBER");
        assert!(recorder.committed);
    }

    #[test]
    fn auto_strip_not_recorded_in_undo() {
        // Validates: Requirement 9.3
        assert!(!should_record_undo(true, false));
    }

    #[test]
    fn show_toggle_not_recorded_in_undo() {
        // Validates: Requirement 8.6
        assert!(!should_record_undo(false, true));
    }

    #[test]
    fn unnum_command_is_recorded() {
        // Validates: Requirement 9.1
        assert!(should_record_undo(false, false));
    }
}
