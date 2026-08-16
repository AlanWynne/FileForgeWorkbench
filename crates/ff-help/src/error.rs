//! Error types for the help subsystem.
//!
//! All errors follow the `[help] operation: description` format standard.

/// All errors produced by the help subsystem.
///
/// Follows the `[help] operation: description` format per the error message standards.
///
/// # Variants
///
/// Each variant corresponds to a specific failure mode within the help system.
#[derive(Debug, thiserror::Error)]
pub enum HelpError {
    /// Requested topic key does not exist in the registry.
    #[error("[help] lookup: topic not found — {key}")]
    TopicNotFound {
        /// The topic key that was not found.
        key: String,
    },

    /// Help content file could not be found on disk.
    #[error("[help] content: file not found — {path}")]
    ContentFileNotFound {
        /// The file path that was missing.
        path: String,
    },

    /// Failed to parse a `.help.md` file (invalid topic delimiter format).
    #[error("[help] parse: invalid topic format in {path} at line {line} — {reason}")]
    ContentParseError {
        /// Path to the file that failed to parse.
        path: String,
        /// Line number where the error occurred.
        line: usize,
        /// Description of what went wrong.
        reason: String,
    },

    /// Help content directory not found at any search location.
    #[error("[help] content: help directory not found (searched: {searched_paths})")]
    ContentDirectoryMissing {
        /// Comma-separated list of paths that were searched.
        searched_paths: String,
    },

    /// Internal lock was poisoned (concurrent access failure).
    #[error("[help] registry: lock poisoned — {context}")]
    RegistryLockPoisoned {
        /// Description of which lock failed.
        context: String,
    },

    /// Topic key string does not match any valid format.
    #[error("[help] parse: invalid topic key format — {raw}")]
    InvalidTopicKey {
        /// The raw string that could not be parsed.
        raw: String,
    },

    /// Search query is too short (minimum 2 characters).
    #[error("[help] search: query too short (minimum 2 characters) — got {length}")]
    SearchQueryTooShort {
        /// Length of the query that was rejected.
        length: usize,
    },

    /// Navigation stack is empty — cannot navigate back or forward.
    #[error("[help] navigation: {direction} — no topic in history")]
    NavigationStackEmpty {
        /// The direction attempted ("back" or "forward").
        direction: String,
    },

    /// Plugin topic registration conflict.
    #[error("[help] plugin: duplicate topic key {key} from plugin {plugin_id}")]
    PluginTopicConflict {
        /// The plugin that caused the conflict.
        plugin_id: String,
        /// The conflicting topic key.
        key: String,
    },

    /// Configuration value is invalid (fallback applied, logged as warning).
    #[error("[help] config: invalid value for {key}, using default — {reason}")]
    ConfigInvalid {
        /// The configuration key that had an invalid value.
        key: String,
        /// Why the value was rejected.
        reason: String,
    },

    /// Hot-reload file watcher registration failed.
    #[error("[help] reload: failed to watch {path} — {reason}")]
    HotReloadFailed {
        /// Path that could not be watched.
        path: String,
        /// Reason for the failure.
        reason: String,
    },
}
