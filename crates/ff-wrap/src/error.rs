//! Error types for the ff-wrap crate.
//!
//! All errors follow the `[wrap] operation: description` format per
//! the project's error message standards.

/// Errors originating from the ff-wrap crate.
///
/// Formatted per Error Message Standards: `[wrap] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WrapError {
    /// Invalid sub-command provided to WRAP command.
    #[error(
        "[wrap] command: invalid sub-command '{arg}' — valid: ON, OFF, TOGGLE, WORD, CHAR, COL <n>"
    )]
    InvalidSubCommand {
        /// The invalid argument that was provided.
        arg: String,
    },

    /// Invalid column value for WRAP COL command.
    #[error("[wrap] command: invalid column '{value}' — must be 0–10000")]
    InvalidColumn {
        /// The invalid column value string.
        value: String,
    },

    /// Configuration key has invalid value.
    #[error("[wrap] config: key '{key}' has invalid value '{value}' — using default '{default}'")]
    InvalidConfig {
        /// The configuration key name.
        key: String,
        /// The invalid value that was provided.
        value: String,
        /// The default value being used instead.
        default: String,
    },

    /// Wrap column out of valid range in configuration.
    #[error(
        "[wrap] config: wrap_column {value} is out of range (0–10000) — using default (viewport)"
    )]
    ColumnOutOfRange {
        /// The out-of-range column value.
        value: i64,
    },

    /// Indent amount out of valid range.
    #[error("[wrap] config: indent_amount {value} is out of range (0–40) — clamped to {clamped}")]
    IndentAmountOutOfRange {
        /// The out-of-range indent amount.
        value: i64,
        /// The clamped value being used.
        clamped: u8,
    },

    /// No active editor instance to apply wrap operation to.
    #[error("[wrap] apply: no active editor instance")]
    NoActiveEditor,

    /// Session restore encountered an unrecognised wrap mode.
    #[error("[wrap] restore: unrecognised mode '{mode}' — falling back to None")]
    UnrecognisedPersistedMode {
        /// The unrecognised mode string.
        mode: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_sub_command_error_format_includes_arg() {
        // Validates: Requirement 3.14
        let err = WrapError::InvalidSubCommand {
            arg: "BANANA".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("BANANA"));
        assert!(msg.contains("[wrap] command:"));
        assert!(msg.contains("ON, OFF, TOGGLE, WORD, CHAR, COL"));
    }

    #[test]
    fn invalid_column_error_format_includes_value() {
        let err = WrapError::InvalidColumn {
            value: "abc".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("abc"));
        assert!(msg.contains("0–10000"));
    }

    #[test]
    fn column_out_of_range_error_format() {
        let err = WrapError::ColumnOutOfRange { value: -5 };
        let msg = err.to_string();
        assert!(msg.contains("-5"));
        assert!(msg.contains("0–10000"));
    }

    #[test]
    fn indent_amount_out_of_range_error_format() {
        let err = WrapError::IndentAmountOutOfRange {
            value: 50,
            clamped: 40,
        };
        let msg = err.to_string();
        assert!(msg.contains("50"));
        assert!(msg.contains("40"));
    }

    #[test]
    fn unrecognised_persisted_mode_error_format() {
        let err = WrapError::UnrecognisedPersistedMode {
            mode: "turbo".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("turbo"));
        assert!(msg.contains("falling back to None"));
    }
}
