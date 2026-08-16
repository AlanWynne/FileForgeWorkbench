//! Core types for the sequence numbers subsystem.
//!
//! Defines `ColumnRange`, `SequenceFormat`, and `DetectionResult` which
//! are used throughout the crate.

use crate::error::SeqNumError;

/// Represents a validated column range for sequence numbers.
/// Column numbers are 1-based, matching ISPF conventions.
///
/// # Invariants
/// - `start >= 1`
/// - `end >= start`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnRange {
    start: u32,
    end: u32,
}

impl ColumnRange {
    /// Create a column range from explicit start and end values.
    ///
    /// Returns `Err(InvalidColumnRange)` if `start` is zero, or `start > end`.
    ///
    /// # Examples
    /// ```
    /// use ff_seqnum::ColumnRange;
    /// let range = ColumnRange::new(1, 6).unwrap();
    /// assert_eq!(range.start(), 1);
    /// assert_eq!(range.end(), 6);
    /// ```
    pub fn new(start: u32, end: u32) -> Result<Self, SeqNumError> {
        if start == 0 || end == 0 {
            return Err(SeqNumError::InvalidColumnRange {
                value: format!("{start}-{end}"),
                reason: "column numbers must be greater than zero".to_string(),
            });
        }
        if start > end {
            return Err(SeqNumError::InvalidColumnRange {
                value: format!("{start}-{end}"),
                reason: format!("start ({start}) must be less than or equal to end ({end})"),
            });
        }
        Ok(Self { start, end })
    }

    /// Parse a column range from a `"start-end"` string (e.g., `"1-6"`, `"73-80"`).
    ///
    /// Returns `Err(InvalidColumnRange)` if the format is invalid (non-numeric,
    /// start > end, or zero values).
    ///
    /// # Examples
    /// ```
    /// use ff_seqnum::ColumnRange;
    /// let range = ColumnRange::parse("73-80").unwrap();
    /// assert_eq!(range.start(), 73);
    /// assert_eq!(range.end(), 80);
    /// ```
    pub fn parse(s: &str) -> Result<Self, SeqNumError> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(SeqNumError::InvalidColumnRange {
                value: s.to_string(),
                reason: "expected format 'start-end' (e.g., '1-6')".to_string(),
            });
        }
        let start: u32 = parts[0]
            .trim()
            .parse()
            .map_err(|_| SeqNumError::InvalidColumnRange {
                value: s.to_string(),
                reason: format!("'{}' is not a valid column number", parts[0].trim()),
            })?;
        let end: u32 = parts[1]
            .trim()
            .parse()
            .map_err(|_| SeqNumError::InvalidColumnRange {
                value: s.to_string(),
                reason: format!("'{}' is not a valid column number", parts[1].trim()),
            })?;
        Self::new(start, end)
    }

    /// Returns the starting column (1-based).
    pub fn start(&self) -> u32 {
        self.start
    }

    /// Returns the ending column (1-based).
    pub fn end(&self) -> u32 {
        self.end
    }

    /// Returns the width of the column range (`end - start + 1`).
    pub fn width(&self) -> u32 {
        self.end - self.start + 1
    }

    /// Returns the 0-based byte offset for the start of this range within a line.
    pub fn start_offset(&self) -> usize {
        (self.start - 1) as usize
    }

    /// Returns the 0-based byte offset for the end of this range within a line (exclusive).
    pub fn end_offset(&self) -> usize {
        self.end as usize
    }
}

impl std::fmt::Display for ColumnRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

/// The format specification for generated sequence numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequenceFormat {
    /// Pure numeric format: zero-padded decimal filling the entire column width.
    /// Example: `"000100"` for value 100 in a 6-column range.
    Numeric,
    /// Alphanumeric prefix format: fixed alphabetic prefix followed by zero-padded digits.
    /// Example: `"ABC001"` for prefix `"ABC"`, value 1, in a 6-column range.
    AlphaPrefix {
        /// The alphabetic prefix string (uppercase).
        prefix: String,
    },
}

impl SequenceFormat {
    /// Returns the number of digit positions available for the given column width.
    pub fn digit_width(&self, column_width: u32) -> u32 {
        match self {
            Self::Numeric => column_width,
            Self::AlphaPrefix { prefix } => column_width.saturating_sub(prefix.len() as u32),
        }
    }

    /// Returns the maximum sequence value representable in the given column width.
    pub fn max_value(&self, column_width: u32) -> u64 {
        let digits = self.digit_width(column_width);
        if digits == 0 {
            return 0;
        }
        10u64.saturating_pow(digits) - 1
    }

    /// Validates that this format can produce at least one digit in the given width.
    pub fn validate_for_width(&self, column_width: u32) -> bool {
        self.digit_width(column_width) >= 1
    }

    /// Format a sequence value into a string of the specified width.
    ///
    /// Returns `None` if the value overflows the available digit positions.
    pub fn format_value(&self, value: u64, column_width: u32) -> Option<String> {
        let digit_width = self.digit_width(column_width) as usize;
        if digit_width == 0 {
            return None;
        }
        if value > self.max_value(column_width) {
            return None;
        }
        let digits = format!("{:0>width$}", value, width = digit_width);
        match self {
            Self::Numeric => Some(digits),
            Self::AlphaPrefix { prefix } => Some(format!("{prefix}{digits}")),
        }
    }
}

/// The result of sequence number detection for a single column range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionResult {
    /// Sequence numbers are present in the column range.
    Present,
    /// Sequence numbers are not present in the column range.
    Absent,
}

/// The format classification detected during sampling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedFormat {
    /// Pure numeric sequence (all digits or spaces with at least one all-digit line).
    Numeric,
    /// Alphanumeric sequence with a consistent prefix.
    AlphaPrefix {
        /// The detected alphabetic prefix.
        prefix: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── ColumnRange Tests ──────────────────────────────────────────────────

    #[test]
    fn column_range_new_valid() {
        // Validates: Requirement 1.4
        let range = ColumnRange::new(1, 6).unwrap();
        assert_eq!(range.start(), 1);
        assert_eq!(range.end(), 6);
        assert_eq!(range.width(), 6);
    }

    #[test]
    fn column_range_new_single_column() {
        // Validates: Requirement 1.4
        let range = ColumnRange::new(5, 5).unwrap();
        assert_eq!(range.width(), 1);
    }

    #[test]
    fn column_range_new_zero_start_fails() {
        // Validates: Requirement 1.4
        let result = ColumnRange::new(0, 6);
        assert!(result.is_err());
    }

    #[test]
    fn column_range_new_zero_end_fails() {
        // Validates: Requirement 1.4
        let result = ColumnRange::new(1, 0);
        assert!(result.is_err());
    }

    #[test]
    fn column_range_new_start_greater_than_end_fails() {
        // Validates: Requirement 1.4
        let result = ColumnRange::new(8, 3);
        assert!(result.is_err());
    }

    #[test]
    fn column_range_parse_valid_cobol_front() {
        // Validates: Requirement 1.1
        let range = ColumnRange::parse("1-6").unwrap();
        assert_eq!(range.start(), 1);
        assert_eq!(range.end(), 6);
    }

    #[test]
    fn column_range_parse_valid_cobol_back() {
        // Validates: Requirement 1.2
        let range = ColumnRange::parse("73-80").unwrap();
        assert_eq!(range.start(), 73);
        assert_eq!(range.end(), 80);
    }

    #[test]
    fn column_range_parse_valid_fortran_front() {
        // Validates: Requirement 1.1
        let range = ColumnRange::parse("1-5").unwrap();
        assert_eq!(range.start(), 1);
        assert_eq!(range.end(), 5);
    }

    #[test]
    fn column_range_parse_invalid_no_dash() {
        // Validates: Requirement 1.4
        assert!(ColumnRange::parse("abc").is_err());
    }

    #[test]
    fn column_range_parse_invalid_empty() {
        // Validates: Requirement 1.4
        assert!(ColumnRange::parse("").is_err());
    }

    #[test]
    fn column_range_parse_invalid_zero_start() {
        // Validates: Requirement 1.4
        assert!(ColumnRange::parse("0-6").is_err());
    }

    #[test]
    fn column_range_parse_invalid_start_gt_end() {
        // Validates: Requirement 1.4
        assert!(ColumnRange::parse("8-3").is_err());
    }

    #[test]
    fn column_range_offsets() {
        // Validates: Requirement 1.1
        let range = ColumnRange::new(1, 6).unwrap();
        assert_eq!(range.start_offset(), 0);
        assert_eq!(range.end_offset(), 6);
    }

    #[test]
    fn column_range_display() {
        let range = ColumnRange::new(73, 80).unwrap();
        assert_eq!(format!("{range}"), "73-80");
    }

    // ─── SequenceFormat Tests ───────────────────────────────────────────────

    #[test]
    fn numeric_format_value() {
        // Validates: Requirement 7.1
        let fmt = SequenceFormat::Numeric;
        assert_eq!(fmt.format_value(100, 6), Some("000100".to_string()));
    }

    #[test]
    fn numeric_format_value_max() {
        // Validates: Requirement 7.1
        let fmt = SequenceFormat::Numeric;
        assert_eq!(fmt.format_value(999999, 6), Some("999999".to_string()));
    }

    #[test]
    fn numeric_format_overflow() {
        // Validates: Requirement 6.11
        let fmt = SequenceFormat::Numeric;
        assert_eq!(fmt.format_value(1_000_000, 6), None);
    }

    #[test]
    fn alpha_prefix_format_value() {
        // Validates: Requirement 7.2
        let fmt = SequenceFormat::AlphaPrefix {
            prefix: "ABC".to_string(),
        };
        assert_eq!(fmt.format_value(1, 6), Some("ABC001".to_string()));
    }

    #[test]
    fn alpha_prefix_validate_too_long() {
        // Validates: Requirement 7.4
        let fmt = SequenceFormat::AlphaPrefix {
            prefix: "ABCDEF".to_string(),
        };
        assert!(!fmt.validate_for_width(6));
    }

    #[test]
    fn alpha_prefix_validate_fits() {
        // Validates: Requirement 7.4
        let fmt = SequenceFormat::AlphaPrefix {
            prefix: "ABC".to_string(),
        };
        assert!(fmt.validate_for_width(6));
    }

    #[test]
    fn numeric_digit_width_equals_column_width() {
        // Validates: Requirement 7.1
        let fmt = SequenceFormat::Numeric;
        assert_eq!(fmt.digit_width(8), 8);
    }

    #[test]
    fn alpha_prefix_digit_width() {
        // Validates: Requirement 7.2
        let fmt = SequenceFormat::AlphaPrefix {
            prefix: "XY".to_string(),
        };
        assert_eq!(fmt.digit_width(8), 6);
    }

    #[test]
    fn numeric_max_value() {
        let fmt = SequenceFormat::Numeric;
        assert_eq!(fmt.max_value(6), 999_999);
    }
}
