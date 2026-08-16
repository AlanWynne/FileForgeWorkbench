//! Sequence number generation engine.
//!
//! Produces zero-padded numeric or alpha-prefix sequences and writes
//! them into specified column positions in the edit buffer.

use crate::error::SeqNumError;
use crate::state::AutoNumberState;
use crate::traits::DocumentMutate;
use crate::types::{ColumnRange, SequenceFormat};

/// Result of a numbering operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberResult {
    /// Number of lines numbered.
    pub lines_numbered: usize,
    /// Whether any sequence values overflowed the column width.
    pub overflow_occurred: bool,
    /// Overflow status message (if overflow occurred).
    pub overflow_message: Option<String>,
}

/// Generate a sequence of formatted numbers.
///
/// Returns a vector of formatted strings, each exactly `width` characters wide.
/// If a value overflows, it is truncated to fit and the overflow flag is set.
pub fn generate_sequence(
    width: u32,
    start: u32,
    increment: u32,
    count: usize,
    format: &SequenceFormat,
) -> (Vec<String>, bool) {
    let mut values = Vec::with_capacity(count);
    let mut overflow = false;
    let max_val = format.max_value(width);

    for i in 0..count {
        let value = start as u64 + (i as u64 * increment as u64);
        if let Some(formatted) = format.format_value(value, width) {
            values.push(formatted);
        } else {
            // Overflow: truncate to max
            overflow = true;
            let truncated = format
                .format_value(max_val, width)
                .unwrap_or_else(|| "9".repeat(width as usize));
            values.push(truncated);
        }
    }

    (values, overflow)
}

/// Write sequence numbers into the specified column range on all lines.
pub fn apply_numbering(
    document: &mut dyn DocumentMutate,
    range: &ColumnRange,
    start: u32,
    increment: u32,
    format: &SequenceFormat,
    scope: Option<(usize, usize)>,
) -> NumberResult {
    let (start_line, end_line) = match scope {
        Some((s, e)) => (s, e.min(document.line_count())),
        None => (0, document.line_count()),
    };

    let count = end_line.saturating_sub(start_line);
    if count == 0 {
        return NumberResult {
            lines_numbered: 0,
            overflow_occurred: false,
            overflow_message: None,
        };
    }

    let (values, overflow) = generate_sequence(range.width(), start, increment, count, format);

    for (i, value) in values.iter().enumerate() {
        let line_idx = start_line + i;
        document.replace_columns(line_idx, range, value);
    }

    let overflow_message = if overflow {
        Some(format!(
            "NUMBER: sequence overflow — numbers truncated to fit COLS {}-{}",
            range.start(),
            range.end()
        ))
    } else {
        None
    };

    NumberResult {
        lines_numbered: count,
        overflow_occurred: overflow,
        overflow_message,
    }
}

/// Assign the next auto-number to a newly inserted line.
///
/// Called by the insert operation hook when NUMBER ON is active.
pub fn auto_number_line(
    document: &mut dyn DocumentMutate,
    line_index: usize,
    auto_state: &mut AutoNumberState,
) -> Result<(), SeqNumError> {
    let value = auto_state.next_value;
    let width = auto_state.target_columns.width();

    if let Some(formatted) = auto_state.format.format_value(value, width) {
        document.replace_columns(line_index, &auto_state.target_columns, &formatted);
        auto_state.next_value += auto_state.increment;
        Ok(())
    } else {
        Err(SeqNumError::OverflowWarning {
            start: auto_state.target_columns.start(),
            end: auto_state.target_columns.end(),
        })
    }
}

/// Validate numbering parameters.
pub fn validate_number_params(start: i64, increment: i64) -> Result<(u32, u32), SeqNumError> {
    if start <= 0 {
        return Err(SeqNumError::InvalidNumberParam {
            param: "start_value".to_string(),
            value: start,
        });
    }
    if increment <= 0 {
        return Err(SeqNumError::InvalidNumberParam {
            param: "increment".to_string(),
            value: increment,
        });
    }
    Ok((start as u32, increment as u32))
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
                    // Pad with spaces to reach start, then add content
                    let padding = " ".repeat(start - line.len());
                    line.push_str(&padding);
                    line.push_str(content);
                    return;
                }
                let actual_end = end.min(line.len());
                let mut new_line = String::with_capacity(line.len().max(end));
                new_line.push_str(&line[..start]);
                new_line.push_str(content);
                if actual_end < line.len() {
                    new_line.push_str(&line[actual_end..]);
                }
                *line = new_line;
            }
        }
    }

    // ─── Tests ──────────────────────────────────────────────────────────────

    #[test]
    fn generate_numeric_sequence_6_col() {
        // Validates: Requirements 6.6, 7.1
        let (values, overflow) = generate_sequence(6, 100, 100, 5, &SequenceFormat::Numeric);
        assert!(!overflow);
        assert_eq!(values.len(), 5);
        assert_eq!(values[0], "000100");
        assert_eq!(values[1], "000200");
        assert_eq!(values[2], "000300");
        assert_eq!(values[3], "000400");
        assert_eq!(values[4], "000500");
    }

    #[test]
    fn generate_numeric_sequence_8_col() {
        // Validates: Requirements 6.6, 7.1
        let (values, overflow) = generate_sequence(8, 10, 10, 3, &SequenceFormat::Numeric);
        assert!(!overflow);
        assert_eq!(values[0], "00000010");
        assert_eq!(values[1], "00000020");
        assert_eq!(values[2], "00000030");
    }

    #[test]
    fn generate_alpha_prefix_sequence() {
        // Validates: Requirement 7.2
        let format = SequenceFormat::AlphaPrefix {
            prefix: "ABC".to_string(),
        };
        let (values, overflow) = generate_sequence(6, 1, 1, 3, &format);
        assert!(!overflow);
        assert_eq!(values[0], "ABC001");
        assert_eq!(values[1], "ABC002");
        assert_eq!(values[2], "ABC003");
    }

    #[test]
    fn generate_sequence_overflow_detected() {
        // Validates: Requirement 6.11
        let (values, overflow) = generate_sequence(3, 990, 10, 3, &SequenceFormat::Numeric);
        assert!(overflow);
        assert_eq!(values[0], "990");
        // 990 + 10 = 1000 > 999 → overflow
        assert_eq!(values[1], "999"); // Truncated to max
    }

    #[test]
    fn alpha_prefix_too_long_rejected() {
        // Validates: Requirement 7.4
        let format = SequenceFormat::AlphaPrefix {
            prefix: "ABCDEF".to_string(),
        };
        assert!(!format.validate_for_width(6));
    }

    #[test]
    fn apply_numbering_to_all_lines() {
        // Validates: Requirement 6.3
        let lines = vec![
            "      MOVE A TO B.                                                          ",
            "      MOVE C TO D.                                                          ",
            "      MOVE E TO F.                                                          ",
        ];
        let mut doc = MockDoc::new(&lines);
        let range = ColumnRange::new(73, 80).unwrap();

        let result = apply_numbering(&mut doc, &range, 100, 100, &SequenceFormat::Numeric, None);

        assert_eq!(result.lines_numbered, 3);
        assert!(!result.overflow_occurred);
        assert!(doc.line_content(0).unwrap().contains("00000100"));
        assert!(doc.line_content(1).unwrap().contains("00000200"));
        assert!(doc.line_content(2).unwrap().contains("00000300"));
    }

    #[test]
    fn apply_numbering_scoped() {
        // Validates: Requirement 6.12
        let lines = vec![
            "      LINE 1                                                                ",
            "      LINE 2                                                                ",
            "      LINE 3                                                                ",
            "      LINE 4                                                                ",
        ];
        let mut doc = MockDoc::new(&lines);
        let range = ColumnRange::new(73, 80).unwrap();

        let result = apply_numbering(
            &mut doc,
            &range,
            1,
            1,
            &SequenceFormat::Numeric,
            Some((1, 3)),
        );

        assert_eq!(result.lines_numbered, 2); // Lines 1, 2
                                              // Line 0 unchanged
        assert!(!doc.line_content(0).unwrap().contains("00000001"));
        // Lines 1-2 numbered
        assert!(doc.line_content(1).unwrap().contains("00000001"));
        assert!(doc.line_content(2).unwrap().contains("00000002"));
    }

    #[test]
    fn validate_zero_start_rejected() {
        // Validates: Requirement 6.5
        let result = validate_number_params(0, 10);
        assert!(result.is_err());
    }

    #[test]
    fn validate_negative_increment_rejected() {
        // Validates: Requirement 6.5
        let result = validate_number_params(10, -1);
        assert!(result.is_err());
    }

    #[test]
    fn validate_valid_params() {
        // Validates: Requirement 6.5
        let (start, inc) = validate_number_params(10, 10).unwrap();
        assert_eq!(start, 10);
        assert_eq!(inc, 10);
    }

    #[test]
    fn auto_number_line_assigns_next_value() {
        // Validates: Requirement 6.7
        let mut doc = MockDoc::new(&[
            "        line content                                                            ",
        ]);
        let range = ColumnRange::new(1, 6).unwrap();
        let mut auto_state = AutoNumberState {
            next_value: 100,
            increment: 100,
            target_columns: range,
            format: SequenceFormat::Numeric,
        };

        auto_number_line(&mut doc, 0, &mut auto_state).unwrap();

        assert!(doc.line_content(0).unwrap().starts_with("000100"));
        assert_eq!(auto_state.next_value, 200);
    }

    #[test]
    fn default_start_and_increment() {
        // Validates: Requirement 6.6
        let (values, _) = generate_sequence(6, 1, 1, 3, &SequenceFormat::Numeric);
        assert_eq!(values[0], "000001");
        assert_eq!(values[1], "000002");
        assert_eq!(values[2], "000003");
    }
}
