//! Command registrations for the catalog subsystem.
//!
//! Registers all catalog/dataset/member/GDG commands with the command framework.

/// Command identifiers for catalog operations.
pub mod ids {
    /// Mount a catalog from repository path.
    pub const CATALOG_MOUNT: &str = "catalog.mount";
    /// Unmount a catalog by name.
    pub const CATALOG_UNMOUNT: &str = "catalog.unmount";
    /// Create a new empty catalog.
    pub const CATALOG_CREATE: &str = "catalog.create";
    /// Remove a catalog.
    pub const CATALOG_REMOVE: &str = "catalog.remove";
    /// Export catalog to ZIP archive.
    pub const CATALOG_EXPORT: &str = "catalog.export";
    /// Import catalog from ZIP archive.
    pub const CATALOG_IMPORT: &str = "catalog.import";
    /// List datasets matching filter pattern.
    pub const CATALOG_LISTCAT: &str = "catalog.listcat";
    /// Display detailed dataset information.
    pub const CATALOG_LISTDS: &str = "catalog.listds";
    /// Allocate (create) a new dataset.
    pub const DATASET_ALLOCATE: &str = "dataset.allocate";
    /// Delete a dataset.
    pub const DATASET_DELETE: &str = "dataset.delete";
    /// Rename a dataset.
    pub const DATASET_RENAME: &str = "dataset.rename";
    /// Retrieve dataset properties.
    pub const DATASET_PROPERTIES: &str = "dataset.properties";
    /// Create a new PDS member.
    pub const MEMBER_CREATE: &str = "member.create";
    /// Delete a PDS member.
    pub const MEMBER_DELETE: &str = "member.delete";
    /// Rename a PDS member.
    pub const MEMBER_RENAME: &str = "member.rename";
    /// Create a GDG base.
    pub const GDG_CREATE_BASE: &str = "gdg.create_base";
    /// Create a new GDG generation.
    pub const GDG_CREATE_GENERATION: &str = "gdg.create_generation";
    /// Delete a GDG base and all generations.
    pub const GDG_DELETE_BASE: &str = "gdg.delete_base";
    /// List GDG generations.
    pub const GDG_LIST_GENERATIONS: &str = "gdg.list_generations";
}

/// All command IDs registered by this crate.
pub const ALL_COMMANDS: &[&str] = &[
    ids::CATALOG_MOUNT,
    ids::CATALOG_UNMOUNT,
    ids::CATALOG_CREATE,
    ids::CATALOG_REMOVE,
    ids::CATALOG_EXPORT,
    ids::CATALOG_IMPORT,
    ids::CATALOG_LISTCAT,
    ids::CATALOG_LISTDS,
    ids::DATASET_ALLOCATE,
    ids::DATASET_DELETE,
    ids::DATASET_RENAME,
    ids::DATASET_PROPERTIES,
    ids::MEMBER_CREATE,
    ids::MEMBER_DELETE,
    ids::MEMBER_RENAME,
    ids::GDG_CREATE_BASE,
    ids::GDG_CREATE_GENERATION,
    ids::GDG_DELETE_BASE,
    ids::GDG_LIST_GENERATIONS,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_commands_list_complete() {
        // Validates: Requirement 5 AC 8, Req 6 AC 9, Req 7 AC 11, Req 8 AC 10, Req 9 AC 9
        assert_eq!(ALL_COMMANDS.len(), 19);
        assert!(ALL_COMMANDS.contains(&ids::CATALOG_MOUNT));
        assert!(ALL_COMMANDS.contains(&ids::GDG_LIST_GENERATIONS));
    }

    #[test]
    fn command_ids_follow_dot_notation() {
        for cmd in ALL_COMMANDS {
            assert!(cmd.contains('.'), "Command '{cmd}' should use dot notation");
        }
    }
}
