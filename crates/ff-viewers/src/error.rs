//! Error types for the ff-viewers crate.
//!
//! All error messages follow the `[viewers] operation: description` format
//! as required by the Error Message Standards.

/// Errors produced by the viewer framework.
///
/// Each variant produces a human-readable message in the format
/// `[viewers] operation: description` that includes contextual information
/// (e.g., the offending key) for diagnostics.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ViewerError {
    /// Invalid ViewerKey format.
    ///
    /// Returned when a viewer key string fails validation — must be non-empty,
    /// contain only lowercase ASCII letters, digits, and hyphens, and be at
    /// most 64 characters long.
    #[error("[viewers] key: invalid format '{key}' — {reason}")]
    InvalidKeyFormat { key: String, reason: String },

    /// Attempted to register a duplicate viewer key.
    ///
    /// Returned when a registration is attempted with a Viewer_Key that
    /// already exists in the ViewerRegistry.
    #[error("[viewers] register: viewer '{key}' is already registered")]
    DuplicateKey { key: String },

    /// Viewer key not found in the registry.
    ///
    /// Returned when a lookup, deregistration, or command references a
    /// viewer key that does not exist in the ViewerRegistry.
    #[error("[viewers] lookup: viewer '{key}' is not registered")]
    UnknownKey { key: String },

    /// Viewer read-only constraint violated.
    ///
    /// Returned when a viewer implementation attempts to invoke a
    /// document-mutating command through the command framework while
    /// Viewer_Mode is active.
    #[error(
        "[viewers] readonly: viewer '{key}' attempted document mutation via command '{command}'"
    )]
    ViewerReadOnlyViolation { key: String, command: String },

    /// Viewer render or content processing failure.
    ///
    /// Returned when a viewer's `on_content_changed` fails or when content
    /// cannot be rendered.
    #[error("[viewers] refresh: viewer '{key}' failed to process content update — {reason}")]
    RenderError { key: String, reason: String },

    /// Configuration error (invalid value in `[viewers]` section).
    ///
    /// Returned when a configuration key contains an invalid value that
    /// cannot be parsed into the expected type or range.
    #[error("[viewers] config: invalid value for key '{key}' — {reason}")]
    ConfigError { key: String, reason: String },

    /// Plugin viewer is no longer available.
    ///
    /// Returned when a plugin-contributed viewer is referenced after the
    /// contributing plugin has shut down.
    #[error("[viewers] plugin: viewer '{key}' is no longer available (plugin shut down)")]
    PluginViewerUnavailable { key: String },

    /// No suitable viewer found for the given resource.
    #[error("[viewers] select: no suitable viewer for resource '{uri}'")]
    NoSuitableViewer { uri: String },

    /// Content read failed via VFS.
    #[error("[viewers] content: failed to read resource '{uri}' — {reason}")]
    ContentReadFailed { uri: String, reason: String },

    /// Command framework integration error.
    #[error("[viewers] command: {0}")]
    CommandError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_key_format_display_includes_key_and_reason() {
        // Validates: Requirement 1 AC 1
        let err = ViewerError::InvalidKeyFormat {
            key: "BAD KEY!".to_string(),
            reason: "contains uppercase and special characters".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[viewers] key:"),
            "Expected prefix, got: {msg}"
        );
        assert!(msg.contains("BAD KEY!"));
        assert!(msg.contains("contains uppercase and special characters"));
    }

    #[test]
    fn duplicate_key_display_includes_key() {
        // Validates: Requirement 1 AC 6
        let err = ViewerError::DuplicateKey {
            key: "asa-report".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[viewers] register:"));
        assert!(msg.contains("asa-report"));
        assert!(msg.contains("already registered"));
    }

    #[test]
    fn unknown_key_display_includes_key() {
        // Validates: Requirement 1 AC 8
        let err = ViewerError::UnknownKey {
            key: "nonexistent".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[viewers] lookup:"));
        assert!(msg.contains("nonexistent"));
        assert!(msg.contains("not registered"));
    }

    #[test]
    fn viewer_read_only_violation_display_includes_key_and_command() {
        // Validates: Requirement 8 AC 4
        let err = ViewerError::ViewerReadOnlyViolation {
            key: "hex".to_string(),
            command: "edit.delete-line".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[viewers] readonly:"));
        assert!(msg.contains("hex"));
        assert!(msg.contains("edit.delete-line"));
        assert!(msg.contains("document mutation"));
    }

    #[test]
    fn render_error_display_includes_key_and_reason() {
        let err = ViewerError::RenderError {
            key: "csv-table".to_string(),
            reason: "malformed CSV input".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[viewers] refresh:"));
        assert!(msg.contains("csv-table"));
        assert!(msg.contains("malformed CSV input"));
    }

    #[test]
    fn config_error_display_includes_key_and_reason() {
        let err = ViewerError::ConfigError {
            key: "split_ratio".to_string(),
            reason: "value 2.0 is outside valid range 0.1–0.9".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[viewers] config:"));
        assert!(msg.contains("split_ratio"));
        assert!(msg.contains("outside valid range"));
    }

    #[test]
    fn plugin_viewer_unavailable_display_includes_key() {
        let err = ViewerError::PluginViewerUnavailable {
            key: "custom-viewer".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[viewers] plugin:"));
        assert!(msg.contains("custom-viewer"));
        assert!(msg.contains("no longer available"));
    }

    #[test]
    fn no_suitable_viewer_display_includes_uri() {
        let err = ViewerError::NoSuitableViewer {
            uri: "file:///path/to/unknown.xyz".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[viewers] select:"));
        assert!(msg.contains("file:///path/to/unknown.xyz"));
    }

    #[test]
    fn command_error_display_shows_message() {
        let err = ViewerError::CommandError("failed to register command".to_string());
        let msg = err.to_string();
        assert!(msg.starts_with("[viewers] command:"));
        assert!(msg.contains("failed to register command"));
    }
}
