//! Error type for the SCRM engine.

use thiserror::Error;

/// Errors produced by the SCRM engine (persistence, export, replay).
#[derive(Debug, Error)]
pub enum ScrmError {
    /// An I/O error while reading or writing a collection or journal.
    #[error("[scrm] io: {operation}: {source}")]
    Io {
        /// What the engine was doing.
        operation: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// An error building or reading the zip archive.
    #[error("[scrm] archive: {operation}: {reason}")]
    Archive {
        /// What the engine was doing.
        operation: String,
        /// A human-readable reason.
        reason: String,
    },

    /// A (de)serialisation error for collection.json / metadata.
    #[error("[scrm] serialize: {0}")]
    Serialize(String),

    /// The archive was missing a required member (e.g. collection.json).
    #[error("[scrm] archive missing required member: {0}")]
    MissingMember(String),
}

/// Convenience result alias.
pub type Result<T> = std::result::Result<T, ScrmError>;
