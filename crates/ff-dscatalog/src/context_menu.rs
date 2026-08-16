//! Context menu definitions for catalog tree nodes.
//!
//! Provides command descriptors for right-click menus on each node type.

/// A context menu item descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItemDescriptor {
    /// Display label.
    pub label: String,
    /// Command ID to invoke.
    pub command_id: String,
    /// Whether the item is enabled.
    pub enabled: bool,
}

/// Node type in the catalog tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogNodeType {
    /// Root "Catalogs" node.
    Root,
    /// A mounted catalog.
    Catalog,
    /// A sequential dataset (PS).
    SequentialDataset,
    /// A partitioned dataset (PDS/PDSE).
    PartitionedDataset,
    /// A PDS member.
    Member,
    /// A GDG base.
    GdgBase,
    /// A GDG generation.
    GdgGeneration,
}

/// Generate context menu items for a given node type.
pub fn menu_for_node(node_type: CatalogNodeType) -> Vec<MenuItemDescriptor> {
    match node_type {
        CatalogNodeType::Root => vec![
            item("Mount Catalog…", "catalog.mount"),
            item("Create New Catalog…", "catalog.create"),
            item("Import Catalog…", "catalog.import"),
        ],
        CatalogNodeType::Catalog => vec![
            item("Unmount", "catalog.unmount"),
            item("New Dataset…", "dataset.allocate"),
            item("Properties", "dataset.properties"),
            item("Export…", "catalog.export"),
        ],
        CatalogNodeType::SequentialDataset => vec![
            item("Open", "vfs.open"),
            item("Rename…", "dataset.rename"),
            item("Delete", "dataset.delete"),
            item("Properties", "dataset.properties"),
        ],
        CatalogNodeType::PartitionedDataset => vec![
            item("New Member…", "member.create"),
            item("Rename…", "dataset.rename"),
            item("Delete", "dataset.delete"),
            item("Properties", "dataset.properties"),
        ],
        CatalogNodeType::Member => vec![
            item("Open", "vfs.open"),
            item("Rename…", "member.rename"),
            item("Delete", "member.delete"),
            item("Properties", "dataset.properties"),
        ],
        CatalogNodeType::GdgBase => vec![
            item("New Generation…", "gdg.create_generation"),
            item("List Generations", "gdg.list_generations"),
            item("Properties", "dataset.properties"),
            item("Delete GDG", "gdg.delete_base"),
        ],
        CatalogNodeType::GdgGeneration => vec![
            item("Open", "vfs.open"),
            item("Delete", "dataset.delete"),
            item("Properties", "dataset.properties"),
        ],
    }
}

fn item(label: &str, command_id: &str) -> MenuItemDescriptor {
    MenuItemDescriptor {
        label: label.to_string(),
        command_id: command_id.to_string(),
        enabled: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_menu_has_mount_create_import() {
        // Validates: Requirement 12 AC 1
        let menu = menu_for_node(CatalogNodeType::Root);
        assert_eq!(menu.len(), 3);
        assert!(menu.iter().any(|m| m.command_id == "catalog.mount"));
        assert!(menu.iter().any(|m| m.command_id == "catalog.create"));
        assert!(menu.iter().any(|m| m.command_id == "catalog.import"));
    }

    #[test]
    fn catalog_menu_has_unmount_and_export() {
        // Validates: Requirement 12 AC 2
        let menu = menu_for_node(CatalogNodeType::Catalog);
        assert!(menu.iter().any(|m| m.command_id == "catalog.unmount"));
        assert!(menu.iter().any(|m| m.command_id == "catalog.export"));
    }

    #[test]
    fn pds_menu_has_new_member() {
        // Validates: Requirement 12 AC 4
        let menu = menu_for_node(CatalogNodeType::PartitionedDataset);
        assert!(menu.iter().any(|m| m.command_id == "member.create"));
    }

    #[test]
    fn gdg_base_menu_has_create_generation() {
        // Validates: Requirement 12 AC 6
        let menu = menu_for_node(CatalogNodeType::GdgBase);
        assert!(menu.iter().any(|m| m.command_id == "gdg.create_generation"));
    }
}
