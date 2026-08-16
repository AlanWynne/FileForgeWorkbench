//! Delegation command registrations.
//!
//! Registers delegation-only commands (SAVE, CANCEL, END, LOAD, RELOAD,
//! DELETE, COPY, MOVE, MACRO/EXEC/RUN, UNDO, REDO) that are dispatched
//! to their owning crates.

/// Metadata for a delegation command registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationEntry {
    /// The command name (primary).
    pub name: &'static str,
    /// Aliases for this command.
    pub aliases: &'static [&'static str],
    /// The owning crate module that handles execution.
    pub owner: &'static str,
    /// Whether this command is undoable.
    pub undoable: bool,
    /// Help text for this command.
    pub help_text: &'static str,
}

/// All delegation command registrations.
pub fn delegation_entries() -> Vec<DelegationEntry> {
    vec![
        // Requirement 11: SAVE, CANCEL, END
        DelegationEntry {
            name: "SAVE",
            aliases: &[],
            owner: "file-operations",
            undoable: false,
            help_text: "Save the current document to disk.",
        },
        DelegationEntry {
            name: "CANCEL",
            aliases: &[],
            owner: "file-operations",
            undoable: false,
            help_text: "Cancel changes and revert to last saved state.",
        },
        DelegationEntry {
            name: "END",
            aliases: &[],
            owner: "file-operations",
            undoable: false,
            help_text: "End the current editing session (save and close).",
        },
        // Requirement 12: LOAD, RELOAD
        DelegationEntry {
            name: "LOAD",
            aliases: &[],
            owner: "file-operations",
            undoable: false,
            help_text: "Load a file into the editor.",
        },
        DelegationEntry {
            name: "RELOAD",
            aliases: &[],
            owner: "file-operations",
            undoable: false,
            help_text: "Reload the current file from disk, discarding in-memory changes.",
        },
        // Requirement 13: DELETE
        DelegationEntry {
            name: "DELETE",
            aliases: &[],
            owner: "edit-operations",
            undoable: true,
            help_text: "Delete the selected lines or block.",
        },
        // Requirement 14: COPY
        DelegationEntry {
            name: "COPY",
            aliases: &[],
            owner: "edit-operations",
            undoable: true,
            help_text: "Copy lines to a target location within the document.",
        },
        // Requirement 15: MOVE
        DelegationEntry {
            name: "MOVE",
            aliases: &[],
            owner: "edit-operations",
            undoable: true,
            help_text: "Move lines to a target location within the document.",
        },
        // Requirement 16: MACRO/EXEC/RUN
        DelegationEntry {
            name: "MACRO",
            aliases: &["EXEC", "RUN"],
            owner: "lua-macro-engine",
            undoable: false,
            help_text: "Execute a Lua macro script.",
        },
        // Requirement 17: UNDO, REDO
        DelegationEntry {
            name: "UNDO",
            aliases: &[],
            owner: "undo-redo-transactions",
            undoable: false, // UNDO itself is not recorded in history
            help_text: "Undo the last undoable operation.",
        },
        DelegationEntry {
            name: "REDO",
            aliases: &[],
            owner: "undo-redo-transactions",
            undoable: false,
            help_text: "Redo the last undone operation.",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_delegation_entries_have_valid_metadata() {
        let entries = delegation_entries();
        assert!(!entries.is_empty());
        for entry in &entries {
            assert!(!entry.name.is_empty());
            assert!(!entry.owner.is_empty());
            assert!(!entry.help_text.is_empty());
        }
    }

    #[test]
    fn macro_has_exec_and_run_aliases() {
        // Validates: Requirement 16
        let entries = delegation_entries();
        let macro_entry = entries.iter().find(|e| e.name == "MACRO").unwrap();
        assert!(macro_entry.aliases.contains(&"EXEC"));
        assert!(macro_entry.aliases.contains(&"RUN"));
    }

    #[test]
    fn undo_redo_are_not_undoable() {
        // Validates: Requirement 17
        let entries = delegation_entries();
        let undo = entries.iter().find(|e| e.name == "UNDO").unwrap();
        let redo = entries.iter().find(|e| e.name == "REDO").unwrap();
        assert!(!undo.undoable);
        assert!(!redo.undoable);
    }

    #[test]
    fn edit_operations_are_undoable() {
        let entries = delegation_entries();
        let delete = entries.iter().find(|e| e.name == "DELETE").unwrap();
        let copy = entries.iter().find(|e| e.name == "COPY").unwrap();
        let move_cmd = entries.iter().find(|e| e.name == "MOVE").unwrap();
        assert!(delete.undoable);
        assert!(copy.undoable);
        assert!(move_cmd.undoable);
    }

    #[test]
    fn file_operations_are_not_undoable() {
        let entries = delegation_entries();
        for name in ["SAVE", "CANCEL", "END", "LOAD", "RELOAD"] {
            let entry = entries.iter().find(|e| e.name == name).unwrap();
            assert!(!entry.undoable, "{name} should not be undoable");
        }
    }
}
