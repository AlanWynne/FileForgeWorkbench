//! Command registration and metadata.
//!
//! Registers all navigation commands with the command framework,
//! providing metadata, help text, mode validity, and undo classification.

/// Mode in which a command is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandMode {
    /// Valid in both Browse and Edit modes.
    BrowseAndEdit,
    /// Valid only in Edit mode.
    EditOnly,
}

/// Undo classification for a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoClass {
    /// Command does not produce an undo record.
    NonUndoable,
    /// Command produces an undo record.
    Undoable,
    /// Command is delegated; undo handling is owned by another crate.
    Delegation,
}

/// Metadata for a registered command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavCommandMetadata {
    /// Primary command name.
    pub name: &'static str,
    /// Display name for UI.
    pub display_name: &'static str,
    /// Aliases for this command.
    pub aliases: &'static [&'static str],
    /// Help text describing syntax and usage.
    pub help_text: &'static str,
    /// Mode validity.
    pub mode: CommandMode,
    /// Undo classification.
    pub undo_class: UndoClass,
}

/// All owned (non-delegation) command registrations.
pub fn owned_command_metadata() -> Vec<NavCommandMetadata> {
    vec![
        NavCommandMetadata {
            name: "LOCATE",
            display_name: "Locate",
            aliases: &[],
            help_text: "LOCATE n | LOCATE label — Jump to line number or named label.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "SORT",
            display_name: "Sort",
            aliases: &[],
            help_text: "SORT [col1 col2] [A|D] [TAGGED|VISIBLE] — Sort lines by column key.",
            mode: CommandMode::EditOnly,
            undo_class: UndoClass::Undoable,
        },
        NavCommandMetadata {
            name: "COLS",
            display_name: "Columns",
            aliases: &[],
            help_text: "COLS — Toggle a column ruler display line.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "BOUNDS",
            display_name: "Bounds",
            aliases: &["BNDS"],
            help_text: "BOUNDS [left right] | BNDS [left right] — Set or clear column boundaries.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "UP",
            display_name: "Scroll Up",
            aliases: &[],
            help_text: "UP [n] — Scroll viewport up by n lines or one page.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "DOWN",
            display_name: "Scroll Down",
            aliases: &[],
            help_text: "DOWN [n] — Scroll viewport down by n lines or one page.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "LEFT",
            display_name: "Scroll Left",
            aliases: &[],
            help_text: "LEFT [n] — Scroll viewport left by n columns or default amount.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "RIGHT",
            display_name: "Scroll Right",
            aliases: &[],
            help_text: "RIGHT [n] — Scroll viewport right by n columns or default amount.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "TOP",
            display_name: "Top",
            aliases: &[],
            help_text: "TOP — Scroll to the first line of the document.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "BOTTOM",
            display_name: "Bottom",
            aliases: &[],
            help_text: "BOTTOM — Scroll to the last line of the document.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "PARA_UP",
            display_name: "Paragraph Up",
            aliases: &[],
            help_text: "PARA_UP — Move caret to the previous paragraph boundary.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "PARA_DOWN",
            display_name: "Paragraph Down",
            aliases: &[],
            help_text: "PARA_DOWN — Move caret to the next paragraph boundary.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "WORD_LEFT",
            display_name: "Word Left",
            aliases: &[],
            help_text: "WORD_LEFT — Move caret to the start of the previous word.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "WORD_RIGHT",
            display_name: "Word Right",
            aliases: &[],
            help_text: "WORD_RIGHT — Move caret to the start of the next word.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "WORD_PART_LEFT",
            display_name: "Word Part Left",
            aliases: &[],
            help_text: "WORD_PART_LEFT — Move caret to the previous sub-word boundary.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "WORD_PART_RIGHT",
            display_name: "Word Part Right",
            aliases: &[],
            help_text: "WORD_PART_RIGHT — Move caret to the next sub-word boundary.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "DOC_START",
            display_name: "Document Start",
            aliases: &[],
            help_text: "DOC_START — Move caret to the beginning of the document.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
        NavCommandMetadata {
            name: "DOC_END",
            display_name: "Document End",
            aliases: &[],
            help_text: "DOC_END — Move caret to the end of the document.",
            mode: CommandMode::BrowseAndEdit,
            undo_class: UndoClass::NonUndoable,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_commands_have_help_text() {
        for cmd in owned_command_metadata() {
            assert!(!cmd.help_text.is_empty(), "{} missing help text", cmd.name);
        }
    }

    #[test]
    fn only_sort_is_undoable() {
        // Validates: Requirement 19
        for cmd in owned_command_metadata() {
            if cmd.name == "SORT" {
                assert_eq!(cmd.undo_class, UndoClass::Undoable);
            } else {
                assert_eq!(
                    cmd.undo_class,
                    UndoClass::NonUndoable,
                    "{} should be non-undoable",
                    cmd.name
                );
            }
        }
    }

    #[test]
    fn sort_is_edit_only() {
        // Validates: Requirement 19
        let sort = owned_command_metadata()
            .into_iter()
            .find(|c| c.name == "SORT")
            .unwrap();
        assert_eq!(sort.mode, CommandMode::EditOnly);
    }

    #[test]
    fn navigation_commands_valid_in_browse_and_edit() {
        // Validates: Requirement 19
        for cmd in owned_command_metadata() {
            if cmd.name != "SORT" {
                assert_eq!(
                    cmd.mode,
                    CommandMode::BrowseAndEdit,
                    "{} should be valid in both modes",
                    cmd.name
                );
            }
        }
    }

    #[test]
    fn bounds_has_bnds_alias() {
        // Validates: Requirement 19
        let bounds = owned_command_metadata()
            .into_iter()
            .find(|c| c.name == "BOUNDS")
            .unwrap();
        assert!(bounds.aliases.contains(&"BNDS"));
    }
}
