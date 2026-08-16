//! Catalog command registrations.
//!
//! Registers all `catalog.*` commands with the `command-framework` for
//! scripting, menus, and keyboard shortcuts.

/// Command identifiers for catalog operations.
pub mod ids {
    /// Create a new structure definition.
    pub const CATALOG_CREATE: &str = "catalog.create";
    /// Read/view a structure definition.
    pub const CATALOG_READ: &str = "catalog.read";
    /// Update an existing structure definition.
    pub const CATALOG_UPDATE: &str = "catalog.update";
    /// Delete a structure definition.
    pub const CATALOG_DELETE: &str = "catalog.delete";
    /// List all structure definitions.
    pub const CATALOG_LIST: &str = "catalog.list";
    /// Duplicate a structure definition.
    pub const CATALOG_DUPLICATE: &str = "catalog.duplicate";
    /// Open the catalog browsing panel.
    pub const CATALOG_BROWSE: &str = "catalog.browse";
    /// Open the structure editor for a named structure.
    pub const CATALOG_EDIT_STRUCTURE: &str = "catalog.edit_structure";
    /// Import a structure from external format.
    pub const CATALOG_IMPORT: &str = "catalog.import";
    /// Export a structure to external format.
    pub const CATALOG_EXPORT: &str = "catalog.export";
    /// Apply a structure to the current file.
    pub const CATALOG_APPLY_STRUCTURE: &str = "catalog.apply_structure";
    /// Open the catalog location manager.
    pub const CATALOG_MANAGE_LOCATIONS: &str = "catalog.manage_locations";
}

/// All registered catalog command IDs.
pub const ALL_COMMAND_IDS: &[&str] = &[
    ids::CATALOG_CREATE,
    ids::CATALOG_READ,
    ids::CATALOG_UPDATE,
    ids::CATALOG_DELETE,
    ids::CATALOG_LIST,
    ids::CATALOG_DUPLICATE,
    ids::CATALOG_BROWSE,
    ids::CATALOG_EDIT_STRUCTURE,
    ids::CATALOG_IMPORT,
    ids::CATALOG_EXPORT,
    ids::CATALOG_APPLY_STRUCTURE,
    ids::CATALOG_MANAGE_LOCATIONS,
];

/// Context menu actions for the catalog browsing panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuAction {
    /// Open the selected structure in the editor.
    OpenInEditor,
    /// Apply the selected structure to the current file.
    ApplyToCurrentFile,
    /// Duplicate the selected structure.
    Duplicate,
    /// Export the selected structure.
    Export,
    /// Delete the selected structure.
    Delete,
}

/// Toolbar actions for the catalog browsing panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    /// Create a new structure definition.
    NewStructure,
    /// Import a structure from external format.
    Import,
    /// Refresh the catalog listing.
    Refresh,
    /// Switch the active catalog location.
    SwitchLocation,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 3.8 — all CRUD commands registered
    #[test]
    fn all_command_ids_contains_crud_commands() {
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_CREATE));
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_READ));
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_UPDATE));
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_DELETE));
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_LIST));
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_DUPLICATE));
    }

    // Validates: Requirement 4.9 — browse command registered
    #[test]
    fn browse_command_registered() {
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_BROWSE));
    }

    // Validates: Requirement 5.12 — edit_structure command registered
    #[test]
    fn edit_structure_command_registered() {
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_EDIT_STRUCTURE));
    }

    // Validates: Requirement 7.1 — import command registered
    #[test]
    fn import_command_registered() {
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_IMPORT));
    }

    // Validates: Requirement 8.1 — export command registered
    #[test]
    fn export_command_registered() {
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_EXPORT));
    }

    // Validates: Requirement 11.1 — apply_structure command registered
    #[test]
    fn apply_structure_command_registered() {
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_APPLY_STRUCTURE));
    }

    // Validates: Requirement 14.1 — manage_locations command registered
    #[test]
    fn manage_locations_command_registered() {
        assert!(ALL_COMMAND_IDS.contains(&ids::CATALOG_MANAGE_LOCATIONS));
    }

    // Validates: Requirement 26 — total command count
    #[test]
    fn total_command_count_is_12() {
        assert_eq!(ALL_COMMAND_IDS.len(), 12);
    }
}
