//! Auto-strip on file open orchestration.
//!
//! Coordinates detection and stripping when a file is opened in
//! Standard_Text_Mode with `auto_unnum` enabled.

use crate::config::SeqNumConfig;
use crate::detector::SequenceDetector;
use crate::state::SeqNumState;
use crate::strip::strip_document;
use crate::traits::{DocumentMutate, LanguageProfile};
use crate::types::{ColumnRange, DetectionResult};

/// Result of the auto-strip-on-open operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoStripResult {
    /// Sequence numbers were detected and stripped from the edit buffer.
    Stripped {
        /// Front column range that was stripped (if any).
        front: Option<ColumnRange>,
        /// Back column range that was stripped (if any).
        back: Option<ColumnRange>,
        /// Status message for the user.
        message: String,
    },
    /// Sequence numbers were detected but NOT stripped (auto_unnum = false).
    Detected {
        /// Status message for the user.
        message: String,
    },
    /// No sequence numbers were detected in the defined column ranges.
    NoSequenceNumbers,
    /// No sequence columns are configured for this language.
    NoColumnsConfigured,
}

/// Perform the auto-strip-on-open sequence.
///
/// 1. Checks if the language profile defines sequence columns.
/// 2. Runs detection on the document content.
/// 3. If detected and `auto_unnum` is true, strips the columns.
/// 4. Returns a status result for UI feedback.
///
/// This operation is NOT added to the undo stack — it is classified as a
/// session initialisation operation (Requirement 3.5).
pub fn auto_strip_on_open(
    document: &mut dyn DocumentMutate,
    profile: &dyn LanguageProfile,
    config: &SeqNumConfig,
    state: &mut SeqNumState,
) -> AutoStripResult {
    let front_cols = profile.sequence_cols_front();
    let back_cols = profile.sequence_cols_back();

    // No columns defined → nothing to do
    if front_cols.is_none() && back_cols.is_none() {
        return AutoStripResult::NoColumnsConfigured;
    }

    // Run detection
    let detector = SequenceDetector::new(config);
    let detection = detector.detect(document, profile);

    let front_detected = detection.front == DetectionResult::Present;
    let back_detected = detection.back == DetectionResult::Present;

    // Store detection result
    state.detection = Some(detection);

    if !front_detected && !back_detected {
        return AutoStripResult::NoSequenceNumbers;
    }

    // Check auto_unnum flag
    if !profile.auto_unnum() {
        let message = "SEQUENCE NUMBERS DETECTED — not removed".to_string();
        return AutoStripResult::Detected { message };
    }

    // Build list of ranges to strip
    let mut ranges_to_strip = Vec::new();
    if front_detected {
        if let Some(cols) = front_cols {
            ranges_to_strip.push(cols);
        }
    }
    if back_detected {
        if let Some(cols) = back_cols {
            ranges_to_strip.push(cols);
        }
    }

    // Strip
    let _result = strip_document(document, &ranges_to_strip, state);

    // Record stripped ranges in state
    if front_detected {
        state.stripped_front = front_cols;
    }
    if back_detected {
        state.stripped_back = back_cols;
    }

    // Build status message
    let range_strs: Vec<String> = ranges_to_strip.iter().map(|r| format!("{r}")).collect();
    let message = format!("SEQUENCE NUMBERS REMOVED: COLS {}", range_strs.join(", "));

    AutoStripResult::Stripped {
        front: state.stripped_front,
        back: state.stripped_back,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::DocumentAccess;

    // ─── Test Helpers ───────────────────────────────────────────────────────

    struct MockDoc {
        lines: Vec<String>,
    }

    impl MockDoc {
        fn new(lines: &[&str]) -> Self {
            Self {
                lines: lines.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl DocumentAccess for MockDoc {
        fn line_count(&self) -> usize {
            self.lines.len()
        }
        fn line_content(&self, index: usize) -> Option<&str> {
            self.lines.get(index).map(|s| s.as_str())
        }
    }

    impl DocumentMutate for MockDoc {
        fn replace_columns(&mut self, line_index: usize, range: &ColumnRange, content: &str) {
            if let Some(line) = self.lines.get_mut(line_index) {
                let start = range.start_offset();
                let end = range.end_offset();
                if line.len() <= start {
                    return;
                }
                let actual_end = end.min(line.len());
                let mut new_line = String::with_capacity(line.len());
                new_line.push_str(&line[..start]);
                new_line.push_str(content);
                if actual_end < line.len() {
                    new_line.push_str(&line[actual_end..]);
                }
                *line = new_line;
            }
        }
    }

    struct MockProfile {
        front: Option<ColumnRange>,
        back: Option<ColumnRange>,
        auto_unnum: bool,
    }

    impl LanguageProfile for MockProfile {
        fn sequence_cols_front(&self) -> Option<ColumnRange> {
            self.front
        }
        fn sequence_cols_back(&self) -> Option<ColumnRange> {
            self.back
        }
        fn auto_unnum(&self) -> bool {
            self.auto_unnum
        }
        fn language_id(&self) -> &str {
            "test"
        }
    }

    fn make_80col_line(front: &str, body: &str, back: &str) -> String {
        let f = format!("{:<6}", front);
        let b_pad = format!("{:<66}", body);
        let bk = format!("{:<8}", back);
        format!("{}{}{}", &f[..6], &b_pad[..66], &bk[..8])
    }

    // ─── Tests ──────────────────────────────────────────────────────────────

    #[test]
    fn auto_strip_enabled_and_detected() {
        // Validates: Requirements 3.1, 3.4
        let lines: Vec<String> = (1..=10)
            .map(|i| {
                make_80col_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut doc = MockDoc::new(&line_refs);
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum: true,
        };
        let config = SeqNumConfig::default();
        let mut state = SeqNumState::new();

        let result = auto_strip_on_open(&mut doc, &profile, &config, &mut state);

        match result {
            AutoStripResult::Stripped {
                front,
                back,
                message,
            } => {
                assert!(front.is_some());
                assert!(back.is_some());
                assert!(message.contains("SEQUENCE NUMBERS REMOVED"));
                assert!(message.contains("1-6"));
                assert!(message.contains("73-80"));
            }
            _ => panic!("Expected Stripped result"),
        }

        // Verify edit buffer is stripped
        assert!(doc.line_content(0).unwrap().starts_with("      "));
    }

    #[test]
    fn auto_strip_disabled_but_detected() {
        // Validates: Requirement 3.6
        let lines: Vec<String> = (1..=10)
            .map(|i| {
                make_80col_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut doc = MockDoc::new(&line_refs);
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum: false,
        };
        let config = SeqNumConfig::default();
        let mut state = SeqNumState::new();

        let result = auto_strip_on_open(&mut doc, &profile, &config, &mut state);

        match result {
            AutoStripResult::Detected { message } => {
                assert!(message.contains("not removed"));
            }
            _ => panic!("Expected Detected result"),
        }

        // Verify edit buffer is NOT stripped
        assert!(doc.line_content(0).unwrap().starts_with("000100"));
    }

    #[test]
    fn auto_strip_not_detected() {
        // Validates: Requirement 3.1 (no strip when not detected)
        let lines: Vec<String> = (1..=10)
            .map(|_| make_80col_line("      ", " CODE.", "        "))
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut doc = MockDoc::new(&line_refs);
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum: true,
        };
        let config = SeqNumConfig::default();
        let mut state = SeqNumState::new();

        let result = auto_strip_on_open(&mut doc, &profile, &config, &mut state);

        assert_eq!(result, AutoStripResult::NoSequenceNumbers);
    }

    #[test]
    fn auto_strip_no_columns_configured() {
        // Validates: Requirement 1.9
        let lines: Vec<String> = (1..=10)
            .map(|i| {
                make_80col_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut doc = MockDoc::new(&line_refs);
        let profile = MockProfile {
            front: None,
            back: None,
            auto_unnum: true,
        };
        let config = SeqNumConfig::default();
        let mut state = SeqNumState::new();

        let result = auto_strip_on_open(&mut doc, &profile, &config, &mut state);

        assert_eq!(result, AutoStripResult::NoColumnsConfigured);
    }

    #[test]
    fn auto_strip_stores_side_table() {
        // Validates: Requirement 3.9
        let lines: Vec<String> = (1..=5)
            .map(|i| {
                make_80col_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut doc = MockDoc::new(&line_refs);
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum: true,
        };
        let config = SeqNumConfig::default();
        let mut state = SeqNumState::new();

        auto_strip_on_open(&mut doc, &profile, &config, &mut state);

        assert!(!state.side_table.is_empty());
    }
}
