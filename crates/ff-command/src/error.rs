//! Error types for the command framework.
//!
//! All errors follow the `[command] operation: description` format per
//! cross-cutting Requirement 8.

/// Errors produced by the command framework.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CommandError {
    /// Command ID is not registered in the registry.
    #[error("[command] dispatch: command '{id}' is not registered")]
    NotFound {
        /// The unrecognized command ID.
        id: String,
    },

    /// Command is currently disabled (enabled predicate returned false).
    #[error("[command] dispatch: command '{id}' is not currently available")]
    Disabled {
        /// The disabled command ID.
        id: String,
    },

    /// Duplicate command registration attempt.
    #[error("[command] register: command '{id}' is already registered")]
    DuplicateId {
        /// The duplicate command ID.
        id: String,
    },

    /// Invalid command ID format.
    #[error("[command] register: invalid command ID '{id}' — {reason}")]
    InvalidId {
        /// The invalid ID string.
        id: String,
        /// Explanation of why the ID is invalid.
        reason: String,
    },

    /// Shortcut binding conflicts with an existing binding.
    #[error(
        "[command] shortcut: binding '{binding}' conflicts with existing command '{existing_id}'"
    )]
    ShortcutConflict {
        /// String representation of the conflicting binding.
        binding: String,
        /// The command attempting to claim the binding.
        new_id: String,
        /// The command that already owns the binding.
        existing_id: String,
    },

    /// Shortcut binding conflicts with a reserved shortcut.
    #[error("[command] shortcut: binding '{binding}' is reserved and cannot be overridden")]
    ShortcutReserved {
        /// String representation of the reserved binding.
        binding: String,
    },

    /// Command handler returned an execution error.
    #[error("[command] execute '{id}': {description}")]
    ExecutionFailed {
        /// The command that failed.
        id: String,
        /// Description of the failure.
        description: String,
    },

    /// Undo operation failed.
    #[error("[command] undo '{id}': {description}")]
    UndoFailed {
        /// The command whose undo failed.
        id: String,
        /// Description of the undo failure.
        description: String,
    },

    /// Redo operation failed.
    #[error("[command] redo '{id}': {description}")]
    RedoFailed {
        /// The command whose redo failed.
        id: String,
        /// Description of the redo failure.
        description: String,
    },

    /// History persistence I/O error.
    #[error("[command] history: {operation} failed — {source}")]
    HistoryIo {
        /// The I/O operation that failed (e.g., "load", "save").
        operation: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Scripting bridge conversion or execution error.
    #[error("[command] scripting: {description}")]
    ScriptingError {
        /// Description of the scripting error.
        description: String,
    },
}

/// Error type for the scripting bridge (converted to Lua errors).
#[derive(Debug, thiserror::Error)]
pub enum ScriptingError {
    /// Command not found during scripting invocation.
    #[error("command '{id}' not found")]
    CommandNotFound {
        /// The missing command ID.
        id: String,
    },

    /// Command execution failed during scripting invocation.
    #[error("command '{id}' failed: {description}")]
    ExecutionFailed {
        /// The command that failed.
        id: String,
        /// Description of the failure.
        description: String,
    },

    /// Parameter conversion failed.
    #[error("parameter conversion failed: {description}")]
    ParamConversion {
        /// Description of the conversion failure.
        description: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_error_displays_command_id() {
        let err = CommandError::NotFound {
            id: "file.open".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[command] dispatch: command 'file.open' is not registered"
        );
    }

    #[test]
    fn disabled_error_displays_command_id() {
        let err = CommandError::Disabled {
            id: "edit.paste".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[command] dispatch: command 'edit.paste' is not currently available"
        );
    }

    #[test]
    fn duplicate_id_error_displays_command_id() {
        let err = CommandError::DuplicateId {
            id: "file.save".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[command] register: command 'file.save' is already registered"
        );
    }

    #[test]
    fn invalid_id_error_displays_reason() {
        let err = CommandError::InvalidId {
            id: "File.Save".to_string(),
            reason: "contains uppercase characters".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[command] register: invalid command ID 'File.Save' — contains uppercase characters"
        );
    }

    #[test]
    fn shortcut_conflict_error_displays_both_ids() {
        let err = CommandError::ShortcutConflict {
            binding: "Ctrl+S".to_string(),
            new_id: "plugin.save".to_string(),
            existing_id: "file.save".to_string(),
        };
        assert!(err.to_string().contains("file.save"));
        assert!(err.to_string().contains("Ctrl+S"));
    }

    #[test]
    fn shortcut_reserved_error_displays_binding() {
        let err = CommandError::ShortcutReserved {
            binding: "Ctrl+Z".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "[command] shortcut: binding 'Ctrl+Z' is reserved and cannot be overridden"
        );
    }
}
