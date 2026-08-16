//! Error types for the ff-menu crate.
//!
//! All errors follow the `[ff-menu] operation: description` format.

use std::path::PathBuf;

/// Errors originating from the ff-menu crate.
///
/// Formatted per Error Message Standards: `[ff-menu] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MenuError {
    /// Attempted to register a status segment with a duplicate ID.
    #[error("[ff-menu] status: segment '{id}' is already registered")]
    DuplicateSegmentId {
        /// The duplicate segment ID.
        id: String,
    },

    /// Invalid segment ID format (must be 1–64 ASCII alphanumeric/underscore).
    #[error("[ff-menu] status: invalid segment ID '{id}' — must be 1-64 ASCII alphanumeric or underscore")]
    InvalidSegmentId {
        /// The invalid segment ID.
        id: String,
    },

    /// Menu item references a command that is not registered.
    #[error("[ff-menu] bind: command '{command_id}' is not registered")]
    CommandNotFound {
        /// The missing command ID.
        command_id: String,
    },

    /// Plugin menu contribution targets a menu path that cannot be resolved.
    #[error("[ff-menu] contribute: cannot resolve menu path '{path}' for plugin '{plugin}'")]
    PluginContributionError {
        /// The unresolvable menu path.
        path: String,
        /// The plugin that contributed the item.
        plugin: String,
    },

    /// Recent files persistence I/O error.
    #[error("[ff-menu] recent_files: {operation} failed for '{}': {source}", path.display())]
    RecentFilesIoError {
        /// The operation that failed (e.g., "load", "save").
        operation: String,
        /// The file path involved.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Recent files JSON parse error.
    #[error("[ff-menu] recent_files: parse error in '{}': {detail}", path.display())]
    RecentFilesParseError {
        /// The file path that failed to parse.
        path: PathBuf,
        /// Description of the parse error.
        detail: String,
    },

    /// Command field submission with empty text.
    #[error("[ff-menu] command_field: cannot submit empty command")]
    EmptyCommand,
}
