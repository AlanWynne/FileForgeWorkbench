//! Error types for the logging subsystem.
//!
//! These errors are used internally; the public API degrades gracefully
//! rather than propagating errors to callers.

use std::path::PathBuf;

/// Errors that can occur within the logging subsystem.
///
/// The public API handles these internally and degrades gracefully rather
/// than propagating them to callers. They are exposed publicly for
/// diagnostic purposes and testing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoggingError {
    /// Failed to create the log directory.
    #[error("failed to create log directory '{path}': {source}")]
    DirectoryCreation {
        /// The path that could not be created.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Failed to create or open a log file.
    #[error("failed to open log file '{path}': {source}")]
    FileOpen {
        /// The path that could not be opened.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Failed to write to the log file.
    #[error("log write failed: {source}")]
    Write {
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Failed to flush buffered records.
    #[error("log flush failed: {source}")]
    Flush {
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Failed to rotate log file.
    #[error("log rotation failed for '{path}': {source}")]
    Rotation {
        /// The path involved in the failed rotation.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Failed to delete old log file during cleanup.
    #[error("failed to delete old log file '{path}': {source}")]
    Cleanup {
        /// The path of the file that could not be deleted.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Invalid configuration value.
    #[error("invalid logging config: {description}")]
    InvalidConfig {
        /// Description of what was invalid.
        description: String,
    },
}
