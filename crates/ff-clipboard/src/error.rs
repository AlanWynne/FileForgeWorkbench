//! Error types for the clipboard subsystem.
//!
//! All errors follow the `[clipboard] operation: description` format standard.

/// All errors produced by the clipboard subsystem.
///
/// Each variant carries enough context to diagnose the problem and produce
/// a user-facing message following the `[clipboard] operation: description` format.
#[derive(Debug, thiserror::Error)]
pub enum ClipboardError {
    /// System clipboard is empty.
    #[error("[clipboard] read: clipboard is empty")]
    Empty,

    /// System clipboard contains non-text content (image, binary, etc.).
    #[error("[clipboard] read: clipboard contains non-text content")]
    NoTextContent,

    /// System clipboard cannot be accessed (permissions, platform error).
    #[error("[clipboard] access: clipboard unavailable \u{2014} {reason}")]
    Unavailable {
        /// Description of why the clipboard is unavailable.
        reason: String,
    },

    /// Clipboard access timed out.
    #[error("[clipboard] access: operation timed out after {timeout_ms}ms")]
    Timeout {
        /// The configured timeout in milliseconds.
        timeout_ms: u32,
    },

    /// Write to system clipboard failed.
    #[error("[clipboard] write: failed to write to clipboard \u{2014} {reason}")]
    WriteFailed {
        /// Description of the write failure.
        reason: String,
    },

    /// File not found for file-insert mode.
    #[error("[clipboard] file-insert: file not found \u{2014} {path}")]
    FileNotFound {
        /// The resolved path that was not found.
        path: String,
    },

    /// File access permission error for file-insert mode.
    #[error("[clipboard] file-insert: access denied \u{2014} {path}")]
    FileAccessDenied {
        /// The path that could not be accessed.
        path: String,
    },

    /// File is binary/non-text for file-insert mode.
    #[error("[clipboard] file-insert: file is not plain text \u{2014} {path}")]
    FileBinary {
        /// The path to the binary file.
        path: String,
    },

    /// File I/O error for file-insert mode.
    #[error("[clipboard] file-insert: I/O error reading {path} \u{2014} {source}")]
    FileIo {
        /// The path being read.
        path: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// COPY command requires an A or B target line command.
    #[error("[clipboard] COPY: target line command A or B is required")]
    NoTarget,

    /// COPY command has conflicting source line commands with file path.
    #[error("[clipboard] COPY: source line commands cannot be combined with a file path argument")]
    ConflictingSourceAndPath,

    /// COPY command is incomplete — source pending but no target.
    #[error("[clipboard] COPY: pending source commands require a target (A or B)")]
    IncompleteSourceTarget,

    /// Configuration value is invalid (logged as warning, fallback applied).
    #[error("[clipboard] config: invalid value for {key}, using default")]
    InvalidConfig {
        /// The configuration key with the invalid value.
        key: String,
    },
}

impl From<std::io::Error> for ClipboardError {
    fn from(err: std::io::Error) -> Self {
        ClipboardError::FileIo {
            path: String::from("<unknown>"),
            source: err,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_error_format_matches_spec() {
        // Validates: Requirement 6.1
        let err = ClipboardError::Empty;
        assert_eq!(err.to_string(), "[clipboard] read: clipboard is empty");
    }

    #[test]
    fn no_text_content_error_format_matches_spec() {
        // Validates: Requirement 6.3
        let err = ClipboardError::NoTextContent;
        assert_eq!(
            err.to_string(),
            "[clipboard] read: clipboard contains non-text content"
        );
    }

    #[test]
    fn unavailable_error_includes_reason() {
        // Validates: Requirement 6.2
        let err = ClipboardError::Unavailable {
            reason: "locked by another process".to_string(),
        };
        assert!(err.to_string().contains("locked by another process"));
        assert!(err.to_string().starts_with("[clipboard] access:"));
    }

    #[test]
    fn timeout_error_includes_milliseconds() {
        // Validates: Requirement 1.6
        let err = ClipboardError::Timeout { timeout_ms: 500 };
        assert!(err.to_string().contains("500ms"));
        assert!(err.to_string().starts_with("[clipboard] access:"));
    }

    #[test]
    fn write_failed_error_includes_reason() {
        // Validates: Requirement 6.4
        let err = ClipboardError::WriteFailed {
            reason: "permission denied".to_string(),
        };
        assert!(err.to_string().contains("permission denied"));
        assert!(err.to_string().starts_with("[clipboard] write:"));
    }

    #[test]
    fn file_not_found_error_includes_path() {
        // Validates: Requirement 10.1
        let err = ClipboardError::FileNotFound {
            path: "/tmp/missing.txt".to_string(),
        };
        assert!(err.to_string().contains("/tmp/missing.txt"));
        assert!(err.to_string().starts_with("[clipboard] file-insert:"));
    }

    #[test]
    fn file_access_denied_error_includes_path() {
        // Validates: Requirement 10.2
        let err = ClipboardError::FileAccessDenied {
            path: "/etc/secret".to_string(),
        };
        assert!(err.to_string().contains("/etc/secret"));
        assert!(err.to_string().contains("access denied"));
    }

    #[test]
    fn file_binary_error_includes_path() {
        // Validates: Requirement 10.3
        let err = ClipboardError::FileBinary {
            path: "image.png".to_string(),
        };
        assert!(err.to_string().contains("image.png"));
        assert!(err.to_string().contains("not plain text"));
    }

    #[test]
    fn file_io_error_includes_path_and_source() {
        // Validates: Requirement 10.2
        let io_err = std::io::Error::other("disk full");
        let err = ClipboardError::FileIo {
            path: "data.txt".to_string(),
            source: io_err,
        };
        assert!(err.to_string().contains("data.txt"));
        assert!(err.to_string().contains("disk full"));
    }

    #[test]
    fn no_target_error_format() {
        // Validates: Requirement 8.2
        let err = ClipboardError::NoTarget;
        assert!(err.to_string().contains("target line command A or B"));
    }

    #[test]
    fn conflicting_source_and_path_error_format() {
        // Validates: Requirement 8.7
        let err = ClipboardError::ConflictingSourceAndPath;
        assert!(err.to_string().contains("cannot be combined"));
    }

    #[test]
    fn incomplete_source_target_error_format() {
        // Validates: Requirement 8.8
        let err = ClipboardError::IncompleteSourceTarget;
        assert!(err.to_string().contains("pending source commands"));
    }

    #[test]
    fn invalid_config_error_includes_key() {
        // Validates: Requirement 19.4
        let err = ClipboardError::InvalidConfig {
            key: "clipboard.access_timeout_ms".to_string(),
        };
        assert!(err.to_string().contains("clipboard.access_timeout_ms"));
    }

    #[test]
    fn from_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ClipboardError = io_err.into();
        match err {
            ClipboardError::FileIo { path, source } => {
                assert_eq!(path, "<unknown>");
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("expected FileIo variant"),
        }
    }
}
