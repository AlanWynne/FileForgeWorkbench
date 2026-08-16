//! Error types for the auto-indent subsystem.
//!
//! All errors follow the `[auto-indent] operation: description` format
//! per the workspace error message standard.

/// Errors originating from the ff-auto-indent crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AutoIndentError {
    /// Invalid auto-indent mode string in configuration.
    #[error(
        "[auto-indent] config: invalid mode '{value}' — expected 'none', 'maintain', or 'smart'"
    )]
    InvalidMode { value: String },

    /// A regex pattern in the language definition failed to compile.
    #[error("[auto-indent] pattern: failed to compile '{pattern_name}' for language '{language_id}': {reason}")]
    PatternCompileError {
        language_id: String,
        pattern_name: String,
        reason: String,
    },

    /// Invalid indent configuration value.
    #[error("[auto-indent] config: invalid value for '{key}': {reason}")]
    InvalidConfig { key: String, reason: String },

    /// Line number out of bounds during indent computation.
    #[error("[auto-indent] compute: line {line} out of bounds (document has {total} lines)")]
    LineOutOfBounds { line: u64, total: u64 },

    /// Language patterns not loaded for the requested language.
    #[error("[auto-indent] lookup: patterns for language '{language_id}' not loaded")]
    PatternsNotLoaded { language_id: String },

    /// Command registration failed.
    #[error("[auto-indent] register: failed to register command '{command_id}': {reason}")]
    CommandRegistrationError { command_id: String, reason: String },
}

impl AutoIndentError {
    /// Returns true if this error should be treated as a graceful degradation
    /// (logged as WARN but not propagated as a fatal error).
    pub fn is_degradable(&self) -> bool {
        matches!(
            self,
            Self::PatternCompileError { .. } | Self::InvalidConfig { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_mode_error_message_format() {
        // Validates: Cross-cutting error handling standard
        let err = AutoIndentError::InvalidMode {
            value: "banana".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.starts_with("[auto-indent]"));
        assert!(msg.contains("banana"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn pattern_compile_error_message_format() {
        // Validates: Cross-cutting error handling standard
        let err = AutoIndentError::PatternCompileError {
            language_id: "rust".to_string(),
            pattern_name: "increase_pattern".to_string(),
            reason: "unbalanced bracket".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.starts_with("[auto-indent]"));
        assert!(msg.contains("rust"));
        assert!(msg.contains("increase_pattern"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn invalid_config_error_message_format() {
        // Validates: Cross-cutting error handling standard
        let err = AutoIndentError::InvalidConfig {
            key: "editor.indent_size".to_string(),
            reason: "must be between 1 and 8".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.starts_with("[auto-indent]"));
        assert!(msg.contains("editor.indent_size"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn line_out_of_bounds_error_message_format() {
        // Validates: Cross-cutting error handling standard
        let err = AutoIndentError::LineOutOfBounds {
            line: 100,
            total: 50,
        };
        let msg = format!("{}", err);
        assert!(msg.starts_with("[auto-indent]"));
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn patterns_not_loaded_error_message_format() {
        // Validates: Cross-cutting error handling standard
        let err = AutoIndentError::PatternsNotLoaded {
            language_id: "python".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.starts_with("[auto-indent]"));
        assert!(msg.contains("python"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn command_registration_error_message_format() {
        // Validates: Cross-cutting error handling standard
        let err = AutoIndentError::CommandRegistrationError {
            command_id: "edit.indent".to_string(),
            reason: "duplicate command".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.starts_with("[auto-indent]"));
        assert!(msg.contains("edit.indent"));
        assert!(msg.len() <= 200);
    }

    #[test]
    fn pattern_compile_error_is_degradable() {
        // Validates: Cross-cutting — graceful degradation
        let err = AutoIndentError::PatternCompileError {
            language_id: "rust".to_string(),
            pattern_name: "increase".to_string(),
            reason: "bad regex".to_string(),
        };
        assert!(err.is_degradable());
    }

    #[test]
    fn invalid_config_is_degradable() {
        // Validates: Cross-cutting — graceful degradation
        let err = AutoIndentError::InvalidConfig {
            key: "editor.indent_size".to_string(),
            reason: "out of range".to_string(),
        };
        assert!(err.is_degradable());
    }

    #[test]
    fn invalid_mode_is_not_degradable() {
        // Validates: Cross-cutting — invalid mode is a user-facing error
        let err = AutoIndentError::InvalidMode {
            value: "bad".to_string(),
        };
        assert!(!err.is_degradable());
    }
}
