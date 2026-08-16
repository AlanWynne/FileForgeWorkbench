//! Error types for the Lua macro engine.
//!
//! All errors follow the `[lua] operation: description` format
//! per cross-cutting Requirement 8.

use crate::security::SecurityMode;

/// Errors produced by the Lua macro engine.
///
/// Follows cross-cutting Requirement 8: `[lua] operation: description`.
///
/// Addresses: Requirement 6 (all criteria)
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LuaEngineError {
    /// Lua runtime error during script execution.
    #[error("[lua] execute '{script}': {message}")]
    ScriptError {
        /// Name or path of the failing script.
        script: String,
        /// The Lua error message.
        message: String,
        /// Optional stack traceback (when debug mode is enabled).
        traceback: Option<String>,
    },

    /// Instruction limit exceeded (infinite loop protection).
    ///
    /// Addresses: Requirement 1 AC 5
    #[error("[lua] execute '{script}': instruction limit exceeded ({count} instructions)")]
    InstructionLimitExceeded {
        /// Name of the script that hit the limit.
        script: String,
        /// Number of instructions executed before termination.
        count: u64,
    },

    /// Memory limit exceeded.
    ///
    /// Addresses: Requirement 1 AC 4, AC 5
    #[error("[lua] execute '{script}': memory limit exceeded ({used_bytes} bytes)")]
    MemoryLimitExceeded {
        /// Name of the script that hit the limit.
        script: String,
        /// Bytes used when the limit was reached.
        used_bytes: usize,
    },

    /// Macro not found in configured directories.
    ///
    /// Addresses: Requirement 5 AC 5
    #[error("[lua] resolve: macro not found: '{name}'")]
    MacroNotFound {
        /// The macro name that was not found.
        name: String,
    },

    /// File not found or not readable.
    ///
    /// Addresses: Requirement 5 AC 6
    #[error("[lua] load: cannot open macro file: '{path}'")]
    FileNotReadable {
        /// The file path that could not be opened.
        path: String,
    },

    /// Security policy denied execution.
    ///
    /// Addresses: Requirement 7 AC 2
    #[error("[lua] security: {reason}")]
    SecurityDenied {
        /// Name of the script that was denied.
        script: String,
        /// The security mode that caused the denial.
        mode: SecurityMode,
        /// Human-readable denial reason.
        reason: String,
    },

    /// Line number out of range in editor API call.
    ///
    /// Addresses: Requirement 2 AC 11
    #[error("[lua] editor.{function}: line {line} is out of range (valid: 1..{max})")]
    LineOutOfRange {
        /// The editor API function that was called.
        function: String,
        /// The invalid line number.
        line: usize,
        /// The maximum valid line number.
        max: usize,
    },

    /// Transaction rollback failed.
    ///
    /// Addresses: Requirement 6 AC 7
    #[error("[lua] rollback: failed to roll back transaction for '{script}': {reason}")]
    RollbackFailed {
        /// Name of the script whose transaction failed to roll back.
        script: String,
        /// Reason for the rollback failure.
        reason: String,
    },

    /// Lua runtime initialization failed.
    #[error("[lua] init: failed to initialize Lua runtime: {reason}")]
    InitFailed {
        /// Reason initialization failed.
        reason: String,
    },

    /// Auto-reload error (non-fatal, logged as warning).
    ///
    /// Addresses: Requirement 8 AC 4
    #[error("[lua] reload '{script}': {message}")]
    ReloadError {
        /// Name of the script that failed to reload.
        script: String,
        /// The error message.
        message: String,
    },

    /// Directory scanning error.
    #[error("[lua] scan: failed to scan directory '{path}': {reason}")]
    ScanError {
        /// Path that could not be scanned.
        path: String,
        /// Reason for the scan failure.
        reason: String,
    },

    /// Plugin context not available.
    #[error("[lua] context: plugin context not initialized")]
    ContextNotInitialized,

    /// Configuration error.
    #[error("[lua] config: {message}")]
    ConfigError {
        /// Description of the configuration error.
        message: String,
    },
}

impl LuaEngineError {
    /// Creates a script error from a name and message.
    pub fn script_error(script: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ScriptError {
            script: script.into(),
            message: message.into(),
            traceback: None,
        }
    }

    /// Creates a script error with traceback.
    pub fn script_error_with_traceback(
        script: impl Into<String>,
        message: impl Into<String>,
        traceback: impl Into<String>,
    ) -> Self {
        Self::ScriptError {
            script: script.into(),
            message: message.into(),
            traceback: Some(traceback.into()),
        }
    }
}

/// Convenience alias for results from the Lua engine.
pub type LuaResult<T> = Result<T, LuaEngineError>;
