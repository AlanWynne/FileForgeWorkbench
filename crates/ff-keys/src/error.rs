//! Error types for the function-keys-and-history subsystem.
//!
//! All error messages follow the `[keys] operation: description` format
//! per cross-cutting requirement 8.

/// Error type for all function-keys-and-history failures.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum KeysError {
    /// A function key identifier could not be parsed.
    #[error("[keys] parse: invalid function key identifier '{key}' — expected F1–F24")]
    InvalidFunctionKey {
        /// The invalid key string that was provided.
        key: String,
    },

    /// Failed to load a key map from configuration.
    #[error("[keys] key-map-load: {reason}")]
    KeyMapLoadFailed {
        /// Description of the load failure.
        reason: String,
    },

    /// A key map entry has an invalid format.
    #[error("[keys] key-map-entry: invalid entry for key '{key}' — {reason}")]
    KeyMapEntryInvalid {
        /// The key that has the invalid entry.
        key: String,
        /// Description of what is invalid.
        reason: String,
    },

    /// History store file could not be loaded.
    #[error("[keys] history-load: failed to read history file — {reason}")]
    HistoryStoreLoadFailed {
        /// Description of the load failure.
        reason: String,
    },

    /// History store file is corrupt or has wrong schema.
    #[error("[keys] history-parse: invalid TOML in history file — {reason}")]
    HistoryStoreCorrupt {
        /// Description of the parse error.
        reason: String,
    },

    /// History store file could not be written.
    #[error("[keys] history-save: failed to write history file — {reason}")]
    HistoryStoreWriteFailed {
        /// Description of the write failure.
        reason: String,
    },

    /// A command assigned to a function key is not registered in the command framework.
    #[error("[keys] dispatch: command '{command}' is not registered")]
    CommandNotRegistered {
        /// The command ID that was not found.
        command: String,
    },

    /// RETRIEVE was invoked but history is empty.
    #[error("[keys] retrieve: command history is empty")]
    RetrieveEmptyHistory,

    /// RETRIEVE reached the end of history.
    #[error("[keys] retrieve: no older history entries available")]
    RetrieveEndOfHistory,

    /// A configuration value is invalid.
    #[error(
        "[keys] config: invalid value for '{key}' — expected {expected}, using default {default}"
    )]
    ConfigInvalid {
        /// The configuration key with the invalid value.
        key: String,
        /// What was expected.
        expected: String,
        /// The default value applied.
        default: String,
    },

    /// A command string assigned to a key is empty.
    #[error("[keys] config: empty command string for key {key}")]
    EmptyCommandString {
        /// The key with the empty command.
        key: String,
    },

    /// Command dispatch failed for a function key press.
    #[error("[keys] dispatch: command execution failed for {key} — {reason}")]
    DispatchFailed {
        /// The function key that triggered the dispatch.
        key: String,
        /// Description of the failure.
        reason: String,
    },

    /// Generic I/O error with operation context.
    #[error("[keys] {operation}: I/O error — {source}")]
    Io {
        /// The operation being attempted.
        operation: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}
