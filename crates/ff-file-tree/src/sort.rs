//! SortEngine — configurable sort order for tree node children.

use crate::node::{FileCategory, NodeType, TreeNodeData};

/// Configurable sort order for tree node children within a directory.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum SortOrder {
    /// Directories listed before files; within each group alphabetical case-insensitive.
    #[default]
    DirectoriesFirst,
    /// Purely alphabetical case-insensitive (no directory preference).
    Alphabetical,
    /// Grouped by file extension, then alphabetical within each group.
    Type,
    /// Most recently modified first (falls back to alphabetical when no time available).
    ModifiedDate,
}

/// Sorts tree node children according to the configured sort order.
pub struct SortEngine {
    order: SortOrder,
}

impl SortEngine {
    /// Create a new SortEngine with the given order.
    pub fn new(order: SortOrder) -> Self {
        Self { order }
    }

    /// Sort a slice of TreeNodeData in place.
    pub fn sort(&self, entries: &mut [TreeNodeData]) {
        match self.order {
            SortOrder::DirectoriesFirst => {
                entries.sort_by(|a, b| {
                    let a_dir = is_directory_like(a);
                    let b_dir = is_directory_like(b);
                    match (a_dir, b_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a
                            .label
                            .to_ascii_lowercase()
                            .cmp(&b.label.to_ascii_lowercase()),
                    }
                });
            }
            SortOrder::Alphabetical => {
                entries.sort_by(|a, b| {
                    a.label
                        .to_ascii_lowercase()
                        .cmp(&b.label.to_ascii_lowercase())
                });
            }
            SortOrder::Type => {
                entries.sort_by(|a, b| {
                    let a_ext = extension_of(&a.label);
                    let b_ext = extension_of(&b.label);
                    a_ext.cmp(&b_ext).then_with(|| {
                        a.label
                            .to_ascii_lowercase()
                            .cmp(&b.label.to_ascii_lowercase())
                    })
                });
            }
            SortOrder::ModifiedDate => {
                // Without modification time in TreeNodeData, fall back to alphabetical.
                entries.sort_by(|a, b| {
                    a.label
                        .to_ascii_lowercase()
                        .cmp(&b.label.to_ascii_lowercase())
                });
            }
        }
    }

    /// Update the sort order.
    pub fn set_order(&mut self, order: SortOrder) {
        self.order = order;
    }

    /// Current sort order.
    pub fn order(&self) -> SortOrder {
        self.order
    }
}

fn is_directory_like(entry: &TreeNodeData) -> bool {
    matches!(
        entry.node_type,
        NodeType::Directory
            | NodeType::BookmarkedRoot
            | NodeType::RootCategory
            | NodeType::HlqGroup
            | NodeType::DatasetPartitioned
            | NodeType::GdgBase
            | NodeType::CatalogRoot
    ) || entry.category == FileCategory::Directory
}

fn extension_of(label: &str) -> String {
    label
        .rsplit('.')
        .next()
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{FileCategory, NodeType, TreeNodeData};

    fn dir(label: &str) -> TreeNodeData {
        TreeNodeData::directory(label)
    }

    fn file(label: &str) -> TreeNodeData {
        TreeNodeData::file(label)
    }

    #[test]
    fn directories_first_puts_dirs_before_files() {
        // Validates: Requirement 4.1 — directories listed before files
        let mut entries = vec![
            file("main.rs"),
            dir("src"),
            file("Cargo.toml"),
            dir("tests"),
        ];
        let engine = SortEngine::new(SortOrder::DirectoriesFirst);
        engine.sort(&mut entries);
        assert_eq!(entries[0].label, "src");
        assert_eq!(entries[1].label, "tests");
        assert_eq!(entries[2].label, "Cargo.toml");
        assert_eq!(entries[3].label, "main.rs");
    }

    #[test]
    fn directories_first_alphabetical_within_groups() {
        // Validates: Requirement 4.1 — alphabetical within each group
        let mut entries = vec![dir("z_dir"), dir("a_dir"), file("z_file"), file("a_file")];
        let engine = SortEngine::new(SortOrder::DirectoriesFirst);
        engine.sort(&mut entries);
        assert_eq!(entries[0].label, "a_dir");
        assert_eq!(entries[1].label, "z_dir");
        assert_eq!(entries[2].label, "a_file");
        assert_eq!(entries[3].label, "z_file");
    }

    #[test]
    fn alphabetical_ignores_dir_file_distinction() {
        // Validates: Requirement 4.2 — alphabetical mode
        let mut entries = vec![file("b.txt"), dir("a_dir"), file("c.rs")];
        let engine = SortEngine::new(SortOrder::Alphabetical);
        engine.sort(&mut entries);
        assert_eq!(entries[0].label, "a_dir");
        assert_eq!(entries[1].label, "b.txt");
        assert_eq!(entries[2].label, "c.rs");
    }

    #[test]
    fn alphabetical_is_case_insensitive() {
        // Validates: Requirement 4.1 — case-insensitive sort
        let mut entries = vec![file("Zebra.txt"), file("apple.rs"), file("Mango.go")];
        let engine = SortEngine::new(SortOrder::Alphabetical);
        engine.sort(&mut entries);
        assert_eq!(entries[0].label, "apple.rs");
        assert_eq!(entries[1].label, "Mango.go");
        assert_eq!(entries[2].label, "Zebra.txt");
    }

    #[test]
    fn type_sort_groups_by_extension() {
        // Validates: Requirement 4.2 — type sort mode
        let mut entries = vec![file("b.rs"), file("a.toml"), file("c.rs"), file("d.toml")];
        let engine = SortEngine::new(SortOrder::Type);
        engine.sort(&mut entries);
        // rs group before toml (alphabetical by ext), then alpha within
        assert_eq!(entries[0].label, "b.rs");
        assert_eq!(entries[1].label, "c.rs");
        assert_eq!(entries[2].label, "a.toml");
        assert_eq!(entries[3].label, "d.toml");
    }

    #[test]
    fn sort_is_idempotent() {
        // Validates: Requirement 4.1 — sorting already-sorted list is stable
        let mut entries = vec![dir("a"), dir("b"), file("c.rs"), file("d.txt")];
        let engine = SortEngine::new(SortOrder::DirectoriesFirst);
        engine.sort(&mut entries);
        let first: Vec<String> = entries.iter().map(|e| e.label.clone()).collect();
        engine.sort(&mut entries);
        let second: Vec<String> = entries.iter().map(|e| e.label.clone()).collect();
        assert_eq!(first, second);
    }

    #[test]
    fn set_order_changes_sort_behaviour() {
        // Validates: Requirement 4.2 — sort order config change
        let mut engine = SortEngine::new(SortOrder::DirectoriesFirst);
        engine.set_order(SortOrder::Alphabetical);
        assert_eq!(engine.order(), SortOrder::Alphabetical);
    }

    #[test]
    fn empty_entries_sort_without_panic() {
        let mut entries: Vec<TreeNodeData> = vec![];
        let engine = SortEngine::new(SortOrder::DirectoriesFirst);
        engine.sort(&mut entries);
        assert!(entries.is_empty());
    }
}
