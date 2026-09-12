//! Navigation model for the File Explorer Context (CR-NR-060 Slice A).
//!
//! Wraps the canonical `ff-file-tree` `TreeState` and adds the shell-side
//! `NodeId -> ResourceUri` association plus the provider-defined
//! `Namespace_Mapping` that converts VFS listing entries into tree nodes.
//!
//! This is the single source of truth for File Explorer structure. Identity is
//! the `ff-file-tree` `NodeId` (and, at the VFS boundary, a `ResourceUri`) --
//! NEVER a reconstructed path string (ADR-002 D1). Two entries that share a
//! display label are distinct nodes with distinct `NodeId`s and do not collide.
//!
//! Validates: Requirement 24.1, 24.2, 24.4 (file-tree-panel)
//!
//! NOTE: this module is the tested foundation for the File Explorer rewire
//! (CR-NR-060 Slice A, task group 26). Its consumers -- the File Explorer render
//! and load path -- are wired in later tasks (26.4-26.9). Until then these items
//! are unused in the compiled binary; the crate-following `allow(dead_code)` keeps
//! the tested foundation in the tree without warnings (same pattern as
//! `command_config`). REMOVE the allow when the panel consumes NavModel (26.9).
#![allow(dead_code)]

use std::collections::HashMap;

use ff_file_tree::{FileCategory, NodeId, NodeType, TreeNodeData, TreeState};
use ff_vfs::{ResourceUri, VfsEntry, VfsEntryType};

/// The File Explorer navigation model: the canonical `TreeState` plus a side
/// table associating each materialized node with its VFS `ResourceUri`.
///
/// The side table keeps `TreeState`/`TreeNode` pure (no URI field on the model);
/// the shell owns the `NodeId -> ResourceUri` mapping for I/O and open/copy/paste.
///
/// Validates: Requirement 24.1, 24.2
#[derive(Default)]
pub struct NavModel {
    /// The canonical tree model (nodes, expansion, selection, filter, sort).
    pub tree: TreeState,
    /// Association of materialized nodes to their VFS resource URI.
    uris: HashMap<NodeId, ResourceUri>,
}

impl NavModel {
    /// Create a new, empty navigation model (three root categories from TreeState).
    pub fn new() -> Self {
        Self {
            tree: TreeState::new(),
            uris: HashMap::new(),
        }
    }

    /// Associate a node with its resource URI.
    ///
    /// Validates: Requirement 24.2
    pub fn set_uri(&mut self, id: NodeId, uri: ResourceUri) {
        self.uris.insert(id, uri);
    }

    /// Look up the resource URI for a node, if one has been associated.
    ///
    /// Validates: Requirement 24.2
    pub fn uri_of(&self, id: NodeId) -> Option<&ResourceUri> {
        self.uris.get(&id)
    }

    /// Number of node -> URI associations currently held.
    pub fn uri_count(&self) -> usize {
        self.uris.len()
    }

    /// Drop URI associations for nodes that no longer exist in the tree.
    ///
    /// Called after structural edits (delete/refresh) to keep the side table in
    /// step with the model.
    pub fn prune_uris(&mut self) {
        let live: std::collections::HashSet<NodeId> =
            self.tree.all_node_ids().into_iter().collect();
        self.uris.retain(|id, _| live.contains(id));
    }
}

/// The provider-defined `Namespace_Mapping`: convert a provider's directory
/// listing (`VfsEntry` values) into `ff-file-tree` `TreeNodeData` with the
/// appropriate `NodeType`/`FileCategory` for that provider's scheme.
///
/// Slice A implements the POSIX/local mapping (schemes `posix` / `local`):
/// directories -> `Directory` (expandable), files -> `File` (leaf, category by
/// extension), symlinks -> `SymbolicLink`. Other schemes (e.g. mainframe
/// `catalog`) are mapped generically here in Slice A and refined in Slice B.
///
/// Validates: Requirement 24.4
pub fn map_entries(scheme: &str, entries: &[VfsEntry]) -> Vec<TreeNodeData> {
    entries.iter().map(|e| map_entry(scheme, e)).collect()
}

/// Map a single VFS entry to a tree node, per the provider's scheme.
///
/// Validates: Requirement 24.4
pub fn map_entry(scheme: &str, entry: &VfsEntry) -> TreeNodeData {
    match scheme {
        // POSIX / local filesystem providers.
        "posix" | "local" => map_posix_entry(entry),
        // Generic mapping for any other provider in Slice A (Slice B refines the
        // mainframe `catalog` scheme into qualifier-group / dataset nodes).
        _ => map_generic_entry(entry),
    }
}

fn map_posix_entry(entry: &VfsEntry) -> TreeNodeData {
    match entry.entry_type {
        VfsEntryType::Directory => {
            let mut d = TreeNodeData::directory(&entry.name);
            d.size = entry.size;
            d
        }
        VfsEntryType::Symlink => TreeNodeData {
            label: entry.name.clone(),
            node_type: NodeType::SymbolicLink,
            size: entry.size,
            category: FileCategory::SymbolicLink,
            has_structure: false,
            is_hidden: entry.name.starts_with('.'),
        },
        // File, Other, and any future variant render as leaf file nodes;
        // category by extension. (VfsEntryType is #[non_exhaustive].)
        VfsEntryType::File | VfsEntryType::Other => {
            let mut f = TreeNodeData::file(&entry.name);
            f.size = entry.size;
            f
        }
        _ => {
            let mut f = TreeNodeData::file(&entry.name);
            f.size = entry.size;
            f
        }
    }
}

fn map_generic_entry(entry: &VfsEntry) -> TreeNodeData {
    match entry.entry_type {
        VfsEntryType::Directory => TreeNodeData::directory(&entry.name),
        _ => TreeNodeData::file(&entry.name),
    }
}

/// Build the child `ResourceUri` for an entry under a parent URI, using
/// forward-slash separators regardless of host OS (Requirement 24.4).
///
/// Example: parent `vfs://local/home/user`, entry `docs` ->
/// `vfs://local/home/user/docs`.
///
/// Validates: Requirement 24.4
pub fn child_uri(parent: &ResourceUri, entry_name: &str) -> ResourceUri {
    let base = parent.path().trim_end_matches('/');
    let path = if base.is_empty() {
        format!("/{entry_name}")
    } else {
        format!("{base}/{entry_name}")
    };
    ResourceUri::new(parent.scheme(), path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, ty: VfsEntryType) -> VfsEntry {
        VfsEntry {
            name: name.to_string(),
            entry_type: ty,
            size: None,
            modified: None,
        }
    }

    #[test]
    fn nav_model_starts_with_three_root_categories() {
        // Validates: Requirement 24.1 -- model driven by ff-file-tree TreeState
        let m = NavModel::new();
        assert_eq!(m.tree.root_categories.len(), 3);
        assert_eq!(m.uri_count(), 0);
    }

    #[test]
    fn identity_is_node_id_and_uri_side_table() {
        // Validates: Requirement 24.2 -- identity is NodeId + ResourceUri
        let mut m = NavModel::new();
        let local = m.tree.root_categories[0];
        m.tree
            .apply_children(local, vec![TreeNodeData::file("a.txt")]);
        let child = m.tree.get_node(local).unwrap().children[0];
        let uri = ResourceUri::new("local", "/home/user/a.txt");
        m.set_uri(child, uri.clone());
        assert_eq!(m.uri_of(child), Some(&uri));
        assert_eq!(m.uri_count(), 1);
    }

    #[test]
    fn same_label_siblings_do_not_collide() {
        // Validates: Requirement 24.2 -- two nodes with the same label are
        // distinct NodeIds and carry distinct URIs (no path-string collision).
        let mut m = NavModel::new();
        let local = m.tree.root_categories[0];
        let catalogs = m.tree.root_categories[1];
        // Same display label "SYS1" under two different parents.
        m.tree
            .apply_children(local, vec![TreeNodeData::directory("SYS1")]);
        m.tree
            .apply_children(catalogs, vec![TreeNodeData::directory("SYS1")]);
        let a = m.tree.get_node(local).unwrap().children[0];
        let b = m.tree.get_node(catalogs).unwrap().children[0];
        assert_ne!(a, b, "same-label nodes must be distinct NodeIds");
        m.set_uri(a, ResourceUri::new("local", "/work/SYS1"));
        m.set_uri(b, ResourceUri::new("catalog", "/SYS1"));
        assert_ne!(m.uri_of(a), m.uri_of(b));
    }

    #[test]
    fn posix_mapping_dir_to_directory_file_to_file() {
        // Validates: Requirement 24.4 -- POSIX/local mapping
        let mapped = map_entries(
            "posix",
            &[
                entry("src", VfsEntryType::Directory),
                entry("main.rs", VfsEntryType::File),
                entry("link", VfsEntryType::Symlink),
            ],
        );
        assert_eq!(mapped[0].node_type, NodeType::Directory);
        assert_eq!(mapped[1].node_type, NodeType::File);
        assert_eq!(mapped[2].node_type, NodeType::SymbolicLink);
    }

    #[test]
    fn local_scheme_maps_like_posix() {
        // Validates: Requirement 24.4 -- both posix and local schemes map POSIX-style
        let mapped = map_entry("local", &entry("dir", VfsEntryType::Directory));
        assert_eq!(mapped.node_type, NodeType::Directory);
    }

    #[test]
    fn child_uri_uses_forward_slash() {
        // Validates: Requirement 24.4 -- forward-slash separators regardless of host OS
        let parent = ResourceUri::new("local", "/home/user");
        let child = child_uri(&parent, "docs");
        assert_eq!(child.scheme(), "local");
        assert_eq!(child.path(), "/home/user/docs");
        assert!(!child.path().contains('\\'));
    }

    #[test]
    fn child_uri_handles_trailing_slash_and_root() {
        // Validates: Requirement 24.4 -- no double slash, root handled
        let with_trailing = ResourceUri::new("local", "/home/user/");
        assert_eq!(child_uri(&with_trailing, "a").path(), "/home/user/a");
        let root = ResourceUri::new("local", "/");
        assert_eq!(child_uri(&root, "a").path(), "/a");
    }

    #[test]
    fn prune_uris_drops_removed_nodes() {
        // Validates: Requirement 24.2 -- side table stays in step with the model
        let mut m = NavModel::new();
        let local = m.tree.root_categories[0];
        m.tree
            .apply_children(local, vec![TreeNodeData::file("a.txt")]);
        let child = m.tree.get_node(local).unwrap().children[0];
        m.set_uri(child, ResourceUri::new("local", "/a.txt"));
        assert_eq!(m.uri_count(), 1);
        m.tree.remove_node(child);
        m.prune_uris();
        assert_eq!(m.uri_count(), 0);
    }

    #[test]
    fn hidden_file_detected_in_posix_mapping() {
        // Validates: Requirement 24.4 -- dotfile classified hidden
        let mapped = map_entry("posix", &entry(".env", VfsEntryType::File));
        assert!(mapped.is_hidden);
    }
}
