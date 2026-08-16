//! Error types for the ff-zoom crate.
//!
//! All errors follow the `[zoom] operation: description` format per
//! workbench error message standards.

/// Errors originating from the ff-zoom crate.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ZoomError {
    /// Zoom in attempted when already at maximum offset.
    #[error("[zoom] zoom_in: maximum zoom reached (+{max_offset})")]
    AtMaximum {
        /// The maximum offset value that was reached.
        max_offset: i32,
    },

    /// Zoom out attempted when already at minimum offset.
    #[error("[zoom] zoom_out: minimum zoom reached ({min_offset})")]
    AtMinimum {
        /// The minimum offset value that was reached.
        min_offset: i32,
    },

    /// Invalid argument to ZOOM command.
    #[error("[zoom] command: invalid argument '{arg}' — expected integer, IN, OUT, or RESET")]
    InvalidCommandArg {
        /// The argument that could not be parsed.
        arg: String,
    },

    /// No active editor instance to apply zoom to.
    #[error("[zoom] apply: no active editor instance")]
    NoActiveEditor,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.6 — error message for maximum reached
    #[test]
    fn at_maximum_error_formats_with_offset() {
        let err = ZoomError::AtMaximum { max_offset: 60 };
        assert_eq!(
            err.to_string(),
            "[zoom] zoom_in: maximum zoom reached (+60)"
        );
    }

    // Validates: Requirement 2.7 — error message for minimum reached
    #[test]
    fn at_minimum_error_formats_with_offset() {
        let err = ZoomError::AtMinimum { min_offset: -10 };
        assert_eq!(
            err.to_string(),
            "[zoom] zoom_out: minimum zoom reached (-10)"
        );
    }

    // Validates: Requirement 8 — invalid command argument error
    #[test]
    fn invalid_command_arg_formats_correctly() {
        let err = ZoomError::InvalidCommandArg {
            arg: "abc".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[zoom] command: invalid argument 'abc' — expected integer, IN, OUT, or RESET"
        );
    }

    #[test]
    fn no_active_editor_error_formats_correctly() {
        let err = ZoomError::NoActiveEditor;
        assert_eq!(err.to_string(), "[zoom] apply: no active editor instance");
    }
}
