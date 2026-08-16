//! Command registration and metadata for file operations.
//!
//! Registers all file commands with the command framework and
//! provides menu layout contributions.

/// Command IDs for all file operations.
pub mod ids {
    /// Create a new empty document.
    pub const FILE_NEW: &str = "file.new";
    /// Open an existing file.
    pub const FILE_OPEN: &str = "file.open";
    /// Open a recent file.
    pub const FILE_OPEN_RECENT: &str = "file.open_recent";
    /// Save the current document.
    pub const FILE_SAVE: &str = "file.save";
    /// Save the current document to a new location.
    pub const FILE_SAVE_AS: &str = "file.save_as";
    /// Revert to the last saved version.
    pub const FILE_REVERT: &str = "file.revert";
    /// Close the current document tab.
    pub const FILE_CLOSE: &str = "file.close";
    /// Exit the application.
    pub const FILE_EXIT: &str = "file.exit";
    /// Toggle read-only mode on the current document.
    pub const FILE_TOGGLE_READ_ONLY: &str = "file.toggle_read_only";
}

/// Event names emitted by file operations.
pub mod events {
    /// Emitted when a new file is created.
    pub const FILE_NEW_CREATED: &str = "file.new_created";
    /// Emitted when a file is opened.
    pub const FILE_OPENED: &str = "file.opened";
    /// Emitted when a file is saved.
    pub const FILE_SAVED: &str = "file.saved";
    /// Emitted when a file is reverted.
    pub const FILE_REVERTED: &str = "file.reverted";
}

/// ISPF command line aliases for file operations.
pub mod aliases {
    /// Alias for `file.new`.
    pub const NEW: &str = "NEW";
    /// Alias for `file.open`.
    pub const OPEN: &str = "OPEN";
    /// Alias for `file.save`.
    pub const SAVE: &str = "SAVE";
    /// Alias for `file.save_as`.
    pub const SAVEAS: &str = "SAVEAS";
    /// Alias for `file.revert`.
    pub const REVERT: &str = "REVERT";
}

/// Default keyboard shortcuts for file operations.
pub mod shortcuts {
    /// New file shortcut.
    pub const NEW: &str = "Ctrl+N";
    /// Open file shortcut.
    pub const OPEN: &str = "Ctrl+O";
    /// Save file shortcut.
    pub const SAVE: &str = "Ctrl+S";
    /// Save As shortcut.
    pub const SAVE_AS: &str = "Ctrl+Shift+S";
    /// Close tab shortcut.
    pub const CLOSE: &str = "Ctrl+W";
    /// Exit application shortcut.
    pub const EXIT: &str = "Alt+F4";
}

/// Command metadata for file operations.
///
/// Addresses: Requirement 10 AC 10.2
#[derive(Debug, Clone)]
pub struct FileCommandMetadata {
    /// Command ID.
    pub id: &'static str,
    /// Display name for menus and palettes.
    pub display_name: &'static str,
    /// Short description.
    pub description: &'static str,
    /// Category.
    pub category: &'static str,
    /// Default keyboard shortcut (if any).
    pub shortcut: Option<&'static str>,
    /// ISPF command line alias (if any).
    pub alias: Option<&'static str>,
}

/// Get metadata for all registered file commands.
///
/// Addresses: Requirement 10 AC 10.1, 10.2, 10.3
pub fn all_command_metadata() -> Vec<FileCommandMetadata> {
    vec![
        FileCommandMetadata {
            id: ids::FILE_NEW,
            display_name: "New",
            description: "Create a new empty document",
            category: "file",
            shortcut: Some(shortcuts::NEW),
            alias: Some(aliases::NEW),
        },
        FileCommandMetadata {
            id: ids::FILE_OPEN,
            display_name: "Open...",
            description: "Open an existing file",
            category: "file",
            shortcut: Some(shortcuts::OPEN),
            alias: Some(aliases::OPEN),
        },
        FileCommandMetadata {
            id: ids::FILE_OPEN_RECENT,
            display_name: "Open Recent",
            description: "Open a recently used file",
            category: "file",
            shortcut: None,
            alias: None,
        },
        FileCommandMetadata {
            id: ids::FILE_SAVE,
            display_name: "Save",
            description: "Save the current document",
            category: "file",
            shortcut: Some(shortcuts::SAVE),
            alias: Some(aliases::SAVE),
        },
        FileCommandMetadata {
            id: ids::FILE_SAVE_AS,
            display_name: "Save As...",
            description: "Save the current document to a new location",
            category: "file",
            shortcut: Some(shortcuts::SAVE_AS),
            alias: Some(aliases::SAVEAS),
        },
        FileCommandMetadata {
            id: ids::FILE_REVERT,
            display_name: "Revert to Saved",
            description: "Discard changes and reload from disk",
            category: "file",
            shortcut: None,
            alias: Some(aliases::REVERT),
        },
        FileCommandMetadata {
            id: ids::FILE_CLOSE,
            display_name: "Close",
            description: "Close the current tab",
            category: "file",
            shortcut: Some(shortcuts::CLOSE),
            alias: None,
        },
        FileCommandMetadata {
            id: ids::FILE_EXIT,
            display_name: "Exit",
            description: "Exit the application",
            category: "file",
            shortcut: Some(shortcuts::EXIT),
            alias: None,
        },
        FileCommandMetadata {
            id: ids::FILE_TOGGLE_READ_ONLY,
            display_name: "Toggle Read-Only",
            description: "Toggle read-only mode for the current document",
            category: "file",
            shortcut: None,
            alias: None,
        },
    ]
}

/// Menu layout entry for the File menu.
///
/// Addresses: Requirement 10 AC 10.4
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuEntry {
    /// A command menu item.
    Command(&'static str),
    /// A submenu.
    Submenu {
        label: &'static str,
        id: &'static str,
    },
    /// A visual separator.
    Separator,
}

/// Get the standard File menu layout.
///
/// Addresses: Requirement 10 AC 10.4
pub fn file_menu_layout() -> Vec<MenuEntry> {
    vec![
        MenuEntry::Command(ids::FILE_NEW),
        MenuEntry::Command(ids::FILE_OPEN),
        MenuEntry::Submenu {
            label: "Recent Files",
            id: ids::FILE_OPEN_RECENT,
        },
        MenuEntry::Separator,
        MenuEntry::Command(ids::FILE_SAVE),
        MenuEntry::Command(ids::FILE_SAVE_AS),
        MenuEntry::Separator,
        MenuEntry::Command(ids::FILE_REVERT),
        MenuEntry::Separator,
        MenuEntry::Command(ids::FILE_CLOSE),
        MenuEntry::Command(ids::FILE_EXIT),
    ]
}

/// Enabled-state predicate for `file.save`.
///
/// Save is disabled when the document is clean AND has an associated URI.
/// Addresses: Requirement 10 AC 10.6
pub fn is_save_enabled(is_dirty: bool, has_uri: bool) -> bool {
    is_dirty || !has_uri
}

/// Enabled-state predicate for `file.revert`.
///
/// Revert is disabled when the document has no associated URI (untitled).
/// Addresses: Requirement 10 AC 10.5
pub fn is_revert_enabled(has_uri: bool) -> bool {
    has_uri
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_ids_are_unique() {
        let all_ids = [
            ids::FILE_NEW,
            ids::FILE_OPEN,
            ids::FILE_OPEN_RECENT,
            ids::FILE_SAVE,
            ids::FILE_SAVE_AS,
            ids::FILE_REVERT,
            ids::FILE_CLOSE,
            ids::FILE_EXIT,
            ids::FILE_TOGGLE_READ_ONLY,
        ];
        let mut seen = std::collections::HashSet::new();
        for id in &all_ids {
            assert!(seen.insert(id), "Duplicate command ID: {id}");
        }
    }

    #[test]
    fn event_names_follow_file_prefix() {
        let all_events = [
            events::FILE_NEW_CREATED,
            events::FILE_OPENED,
            events::FILE_SAVED,
            events::FILE_REVERTED,
        ];
        for event in &all_events {
            assert!(
                event.starts_with("file."),
                "Event should start with 'file.': {event}"
            );
        }
    }

    #[test]
    fn aliases_are_uppercase() {
        let all_aliases = [
            aliases::NEW,
            aliases::OPEN,
            aliases::SAVE,
            aliases::SAVEAS,
            aliases::REVERT,
        ];
        for alias in &all_aliases {
            assert_eq!(
                *alias,
                alias.to_uppercase(),
                "Alias should be uppercase: {alias}"
            );
        }
    }

    // Validates: Requirement 10 AC 10.1 — all commands have metadata
    #[test]
    fn all_commands_have_metadata() {
        let metadata = all_command_metadata();
        assert_eq!(metadata.len(), 9);

        let ids: Vec<&str> = metadata.iter().map(|m| m.id).collect();
        assert!(ids.contains(&ids::FILE_NEW));
        assert!(ids.contains(&ids::FILE_OPEN));
        assert!(ids.contains(&ids::FILE_SAVE));
        assert!(ids.contains(&ids::FILE_SAVE_AS));
        assert!(ids.contains(&ids::FILE_REVERT));
        assert!(ids.contains(&ids::FILE_CLOSE));
        assert!(ids.contains(&ids::FILE_EXIT));
        assert!(ids.contains(&ids::FILE_TOGGLE_READ_ONLY));
    }

    // Validates: Requirement 10 AC 10.2 — metadata has category
    #[test]
    fn all_commands_have_file_category() {
        let metadata = all_command_metadata();
        for cmd in &metadata {
            assert_eq!(
                cmd.category, "file",
                "Command {} should have 'file' category",
                cmd.id
            );
        }
    }

    // Validates: Requirement 10 AC 10.3 — shortcuts
    #[test]
    fn default_shortcuts_are_assigned() {
        let metadata = all_command_metadata();
        let new_cmd = metadata.iter().find(|m| m.id == ids::FILE_NEW).unwrap();
        assert_eq!(new_cmd.shortcut, Some("Ctrl+N"));

        let save_cmd = metadata.iter().find(|m| m.id == ids::FILE_SAVE).unwrap();
        assert_eq!(save_cmd.shortcut, Some("Ctrl+S"));

        let save_as_cmd = metadata.iter().find(|m| m.id == ids::FILE_SAVE_AS).unwrap();
        assert_eq!(save_as_cmd.shortcut, Some("Ctrl+Shift+S"));
    }

    // Validates: Requirement 10 AC 10.4 — menu layout
    #[test]
    fn file_menu_layout_has_correct_order() {
        let layout = file_menu_layout();
        assert_eq!(layout[0], MenuEntry::Command(ids::FILE_NEW));
        assert_eq!(layout[1], MenuEntry::Command(ids::FILE_OPEN));
        assert_eq!(
            layout[2],
            MenuEntry::Submenu {
                label: "Recent Files",
                id: ids::FILE_OPEN_RECENT
            }
        );
        assert_eq!(layout[3], MenuEntry::Separator);
        assert_eq!(layout[4], MenuEntry::Command(ids::FILE_SAVE));
        assert_eq!(layout[5], MenuEntry::Command(ids::FILE_SAVE_AS));
        assert_eq!(layout[6], MenuEntry::Separator);
        assert_eq!(layout[7], MenuEntry::Command(ids::FILE_REVERT));
        assert_eq!(layout[8], MenuEntry::Separator);
        assert_eq!(layout[9], MenuEntry::Command(ids::FILE_CLOSE));
        assert_eq!(layout[10], MenuEntry::Command(ids::FILE_EXIT));
    }

    // Validates: Requirement 10 AC 10.6 — save enabled predicate
    #[test]
    fn save_enabled_when_dirty() {
        assert!(is_save_enabled(true, true));
        assert!(is_save_enabled(true, false));
    }

    #[test]
    fn save_disabled_when_clean_with_uri() {
        assert!(!is_save_enabled(false, true));
    }

    #[test]
    fn save_enabled_when_untitled_even_if_clean() {
        // Untitled (no URI) should allow save (delegates to save_as)
        assert!(is_save_enabled(false, false));
    }

    // Validates: Requirement 10 AC 10.5 — revert enabled predicate
    #[test]
    fn revert_enabled_with_uri() {
        assert!(is_revert_enabled(true));
    }

    #[test]
    fn revert_disabled_without_uri() {
        assert!(!is_revert_enabled(false));
    }

    // Validates: Requirement 10 AC 10.7 — ISPF aliases
    #[test]
    fn ispf_aliases_are_assigned() {
        let metadata = all_command_metadata();
        let new_cmd = metadata.iter().find(|m| m.id == ids::FILE_NEW).unwrap();
        assert_eq!(new_cmd.alias, Some("NEW"));

        let save_cmd = metadata.iter().find(|m| m.id == ids::FILE_SAVE).unwrap();
        assert_eq!(save_cmd.alias, Some("SAVE"));
    }
}
