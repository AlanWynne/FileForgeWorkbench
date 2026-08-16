//! Error types for the sequence numbers subsystem.
//!
//! All errors follow the `[seqnum] operation: description` format
//! per the workspace error message standard.

/// Errors originating from the ff-seqnum crate.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SeqNumError {
    /// A column range string is malformed (not "start-end", start > end, or zero values).
    #[error("[seqnum] column-range: invalid range '{value}' — {reason}")]
    InvalidColumnRange { value: String, reason: String },

    /// No sequence columns are defined for the active language.
    #[error("[seqnum] {command}: no sequence columns defined for this language — use {command} COLS to specify a range")]
    NoSequenceColumns { command: String },

    /// The alpha-prefix is too long for the target column width.
    #[error("[seqnum] number: prefix '{prefix}' too long for column range (width {width})")]
    PrefixTooLong { prefix: String, width: u32 },

    /// Sequence number generation overflowed the column width.
    #[error("[seqnum] number: sequence overflow — numbers truncated to fit COLS {start}-{end}")]
    OverflowWarning { start: u32, end: u32 },

    /// The command is not applicable in Grid_Edit_Mode.
    #[error("[seqnum] {command}: not applicable in Grid Edit Mode")]
    GridEditModeNotAllowed { command: String },

    /// A configuration value is outside the valid range (clamped).
    #[error("[seqnum] config: value {value} for '{key}' outside valid range {min}–{max}, clamped to {clamped}")]
    ConfigOutOfRange {
        key: String,
        value: i64,
        min: i64,
        max: i64,
        clamped: i64,
    },

    /// Front sequence columns are not defined for this language.
    #[error("[seqnum] {command}: front sequence columns not defined for this language")]
    FrontColumnsNotDefined { command: String },

    /// Back sequence columns are not defined for this language.
    #[error("[seqnum] {command}: back sequence columns not defined for this language")]
    BackColumnsNotDefined { command: String },

    /// Start value or increment must be positive.
    #[error("[seqnum] number: {param} must be a positive integer, got {value}")]
    InvalidNumberParam { param: String, value: i64 },
}

impl SeqNumError {
    /// Returns true if this error should be treated as a graceful degradation
    /// (logged as WARN but not propagated as a fatal error).
    pub fn is_degradable(&self) -> bool {
        matches!(
            self,
            Self::ConfigOutOfRange { .. } | Self::OverflowWarning { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_column_range_error_message_format() {
        // Validates: Cross-cutting error handling standard
        let err = SeqNumError::InvalidColumnRange {
            value: "abc".to_string(),
            reason: "not a valid start-end format".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.starts_with("[seqnum]"));
        assert!(msg.contains("abc"));
    }

    #[test]
    fn no_sequence_columns_error_message_format() {
        // Validates: Requirement 5.2
        let err = SeqNumError::NoSequenceColumns {
            command: "UNNUM".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.starts_with("[seqnum]"));
        assert!(msg.contains("UNNUM"));
        assert!(msg.contains("COLS"));
    }

    #[test]
    fn prefix_too_long_error_message_format() {
        // Validates: Requirement 7.4
        let err = SeqNumError::PrefixTooLong {
            prefix: "ABCDEF".to_string(),
            width: 6,
        };
        let msg = format!("{err}");
        assert!(msg.starts_with("[seqnum]"));
        assert!(msg.contains("ABCDEF"));
    }

    #[test]
    fn grid_edit_mode_error_message_format() {
        // Validates: Requirement 13.2
        let err = SeqNumError::GridEditModeNotAllowed {
            command: "UNNUM".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.starts_with("[seqnum]"));
        assert!(msg.contains("Grid Edit Mode"));
    }

    #[test]
    fn config_out_of_range_is_degradable() {
        // Validates: Requirement 2.8 - clamping is a warning, not fatal
        let err = SeqNumError::ConfigOutOfRange {
            key: "detection_threshold".to_string(),
            value: 120,
            min: 50,
            max: 100,
            clamped: 100,
        };
        assert!(err.is_degradable());
    }

    #[test]
    fn overflow_warning_is_degradable() {
        // Validates: Requirement 6.11
        let err = SeqNumError::OverflowWarning { start: 73, end: 80 };
        assert!(err.is_degradable());
    }

    #[test]
    fn invalid_column_range_is_not_degradable() {
        let err = SeqNumError::InvalidColumnRange {
            value: "bad".to_string(),
            reason: "test".to_string(),
        };
        assert!(!err.is_degradable());
    }
}
