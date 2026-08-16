//! Error types for the `ff-session` crate.
//!
//! Provides a unified error enum covering all failure modes across
//! startup sequence, session persistence, CLI processing, exit sequence,
//! and crash recovery subsystems.

use std::path::PathBuf;

/// Unified error type for the `ff-session` crate.
///
/// Each variant carries enough context to diagnose the underlying problem
/// without requiring the caller to inspect nested error chains.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// Configuration loading failed during startup Phase 2.
    #[error("[session] config load: {reason}")]
    ConfigLoadFailed {
        /// Description of the configuration failure.
        reason: String,
    },

    /// The User Data Directory could not be located or created.
    #[error("[session] user data dir unavailable: {path} — {reason}")]
    UserDataDirUnavailable {
        /// The path that was attempted.
        path: PathBuf,
        /// Description of the failure (permission denied, disk full, etc.).
        reason: String,
    },

    /// The session file exists but could not be parsed or validated.
    #[error("[session] session file corrupt: {path} — {reason}")]
    SessionFileCorrupt {
        /// Path to the corrupt session file.
        path: PathBuf,
        /// Description of the parse or validation failure.
        reason: String,
    },

    /// Writing the session file to disk failed.
    #[error("[session] session file write failed: {path} — {reason}")]
    SessionFileWriteFailed {
        /// Path where the write was attempted.
        path: PathBuf,
        /// Description of the I/O failure.
        reason: String,
    },

    /// A plugin failed to initialise during startup Phase 5.
    #[error("[session] plugin init failed: {plugin_name} — {reason}")]
    PluginInitFailed {
        /// Name of the plugin that failed.
        plugin_name: String,
        /// Description of the initialisation failure.
        reason: String,
    },

    /// Layout restoration failed during startup Phase 7.
    #[error("[session] layout restore failed: {reason}")]
    LayoutRestoreFailed {
        /// Description of the layout restoration failure.
        reason: String,
    },

    /// Scanning recovery files failed during startup Phase 10.
    #[error("[session] recovery file scan failed: {path} — {reason}")]
    RecoveryFileScanFailed {
        /// Path to the recovery directory that was scanned.
        path: PathBuf,
        /// Description of the scan failure.
        reason: String,
    },

    /// A recovery file is corrupt and cannot be applied.
    #[error("[session] recovery file corrupt: {path} — {reason}")]
    RecoveryFileCorrupt {
        /// Path to the corrupt recovery file.
        path: PathBuf,
        /// Description of the corruption.
        reason: String,
    },

    /// A command-line argument is invalid and cannot be processed.
    #[error("[session] invalid CLI argument: {argument} — {reason}")]
    CliArgInvalid {
        /// The argument that was invalid.
        argument: String,
        /// Description of why the argument is invalid.
        reason: String,
    },

    /// Window geometry data is invalid or cannot be applied.
    #[error("[session] invalid window geometry: {reason}")]
    WindowGeometryInvalid {
        /// Description of the geometry validation failure.
        reason: String,
    },

    /// The exit sequence was aborted by the user selecting Cancel.
    #[error("[session] exit aborted: user cancelled")]
    ExitAborted,

    /// The shutdown sequence exceeded the allowed time limit.
    #[error("[session] shutdown timeout: exceeded {timeout_seconds}s — {reason}")]
    ShutdownTimeout {
        /// The configured timeout in seconds.
        timeout_seconds: u64,
        /// Description of what stalled.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_load_failed_displays_descriptive_message() {
        let err = SessionError::ConfigLoadFailed {
            reason: "missing TOML table".to_string(),
        };
        assert!(err.to_string().contains("[session] config load:"));
        assert!(err.to_string().contains("missing TOML table"));
    }

    #[test]
    fn user_data_dir_unavailable_includes_path_and_reason() {
        let err = SessionError::UserDataDirUnavailable {
            path: PathBuf::from("/home/user/.config/ffworkbench"),
            reason: "permission denied".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[session] user data dir unavailable:"));
        assert!(msg.contains("ffworkbench"));
        assert!(msg.contains("permission denied"));
    }

    #[test]
    fn session_file_corrupt_includes_path_and_reason() {
        let err = SessionError::SessionFileCorrupt {
            path: PathBuf::from("/data/session.toml"),
            reason: "unexpected EOF".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[session] session file corrupt:"));
        assert!(msg.contains("session.toml"));
        assert!(msg.contains("unexpected EOF"));
    }

    #[test]
    fn session_file_write_failed_includes_path_and_reason() {
        let err = SessionError::SessionFileWriteFailed {
            path: PathBuf::from("/data/session.toml"),
            reason: "disk full".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[session] session file write failed:"));
        assert!(msg.contains("disk full"));
    }

    #[test]
    fn plugin_init_failed_includes_plugin_name() {
        let err = SessionError::PluginInitFailed {
            plugin_name: "git-lens".to_string(),
            reason: "dependency not found".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[session] plugin init failed: git-lens"));
        assert!(msg.contains("dependency not found"));
    }

    #[test]
    fn layout_restore_failed_includes_reason() {
        let err = SessionError::LayoutRestoreFailed {
            reason: "panel type not registered".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[session] layout restore failed:"));
        assert!(msg.contains("panel type not registered"));
    }

    #[test]
    fn recovery_file_scan_failed_includes_path() {
        let err = SessionError::RecoveryFileScanFailed {
            path: PathBuf::from("/data/recovery"),
            reason: "IO error".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[session] recovery file scan failed:"));
        assert!(msg.contains("recovery"));
    }

    #[test]
    fn recovery_file_corrupt_includes_path() {
        let err = SessionError::RecoveryFileCorrupt {
            path: PathBuf::from("/data/recovery/file1.rec"),
            reason: "schema mismatch".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[session] recovery file corrupt:"));
        assert!(msg.contains("schema mismatch"));
    }

    #[test]
    fn cli_arg_invalid_includes_argument_and_reason() {
        let err = SessionError::CliArgInvalid {
            argument: "--unknown-flag".to_string(),
            reason: "unrecognised flag".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[session] invalid CLI argument: --unknown-flag"));
        assert!(msg.contains("unrecognised flag"));
    }

    #[test]
    fn window_geometry_invalid_includes_reason() {
        let err = SessionError::WindowGeometryInvalid {
            reason: "width is zero".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[session] invalid window geometry:"));
        assert!(msg.contains("width is zero"));
    }

    #[test]
    fn exit_aborted_displays_user_cancelled() {
        let err = SessionError::ExitAborted;
        assert!(err.to_string().contains("exit aborted: user cancelled"));
    }

    #[test]
    fn shutdown_timeout_includes_seconds_and_reason() {
        let err = SessionError::ShutdownTimeout {
            timeout_seconds: 5,
            reason: "plugin git-lens did not respond".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("[session] shutdown timeout: exceeded 5s"));
        assert!(msg.contains("plugin git-lens did not respond"));
    }
}
