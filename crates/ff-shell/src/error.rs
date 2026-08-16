//! Error types for the `ff-shell` crate.
//!
//! All errors follow the `[shell] operation: description` format per
//! cross-cutting Requirement 8.

/// All errors produced by the `ff-shell` crate.
///
/// Follows the `[shell] operation: description` format for user-facing messages.
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    /// Shell access is disabled by configuration.
    #[error("[shell] execute: shell access is disabled by configuration")]
    ShellDisabled,

    /// User declined the confirmation prompt (shell.mode = prompt).
    #[error("[shell] execute: user declined shell execution")]
    UserDeclined,

    /// The specified shell executable was not found.
    #[error("[shell] resolve: shell executable not found: {path}")]
    ShellNotFound {
        /// The path or name that could not be resolved.
        path: String,
    },

    /// The specified shell executable cannot be executed (permission error).
    #[error("[shell] resolve: permission denied for shell executable: {path}")]
    ShellPermissionDenied {
        /// The path with insufficient permissions.
        path: String,
    },

    /// Failed to spawn the child process.
    #[error("[shell] spawn: failed to start process: {reason}")]
    SpawnFailed {
        /// Description of the spawn failure.
        reason: String,
    },

    /// I/O error reading process output.
    #[error("[shell] io: error reading process output: {0}")]
    IoError(#[from] std::io::Error),

    /// Command timed out and was terminated.
    #[error("[shell] timeout: command exceeded {seconds}s timeout and was terminated")]
    Timeout {
        /// The configured timeout in seconds.
        seconds: u64,
    },

    /// Process was cancelled by user.
    #[error("[shell] cancel: command was cancelled by user")]
    Cancelled,

    /// Invalid command form (incompatible line commands, etc.).
    #[error("[shell] validate: {reason}")]
    InvalidCommandForm {
        /// Description of the validation failure.
        reason: String,
    },

    /// Document capture failed — non-zero exit code.
    #[error("[shell] capture: command exited with code {code}")]
    CaptureExitError {
        /// The non-zero exit code.
        code: i32,
        /// Stderr output lines from the failed command.
        stderr: Vec<String>,
    },

    /// PTY creation failed.
    #[error("[shell] pty: failed to create pseudo-terminal: {reason}")]
    PtyError {
        /// Description of the PTY failure.
        reason: String,
    },

    /// Terminal session not found.
    #[error("[shell] terminal: session {id} not found")]
    SessionNotFound {
        /// The session ID that was not found.
        id: u64,
    },

    /// Configuration error (invalid value in shell.* namespace).
    #[error("[shell] config: {reason}")]
    ConfigError {
        /// Description of the configuration issue.
        reason: String,
    },

    /// Environment variable expansion referenced an undefined variable.
    #[error("[shell] env: undefined variable referenced: {var_name}")]
    UndefinedVariable {
        /// The variable name that was not defined.
        var_name: String,
    },

    /// Working directory does not exist or is inaccessible.
    #[error("[shell] cwd: working directory not accessible: {path}")]
    WorkingDirError {
        /// The inaccessible path.
        path: String,
    },

    /// Macro invocation refused by shell.mode or macro security.
    #[error("[shell] macro: shell access denied — {reason}")]
    MacroAccessDenied {
        /// Description of why macro access was denied.
        reason: String,
    },
}
