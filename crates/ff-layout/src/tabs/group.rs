//! Tab group data types — `TabGroup`, `TabGroupId`, `TabGroupTree`, `SplitDirection`.
//!
//! Tab groups subdivide the center editor area. Multiple tab groups can
//! coexist via horizontal or vertical splits.

/// Opaque identifier for a tab group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TabGroupId(pub(crate) u32);

impl TabGroupId {
    /// Creates a new tab group ID from a raw value.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw numeric value.
    pub fn value(self) -> u32 {
        self.0
    }
}

/// A subdivision of the center editor area holding one or more editor tabs.
///
/// Multiple tab groups can coexist via horizontal or vertical splits.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TabGroup {
    /// Unique identifier for this tab group.
    pub id: TabGroupId,
    /// Ordered list of tab identifiers within this group.
    pub tabs: Vec<String>,
    /// Index of the currently active tab (0-based).
    pub active_tab: usize,
}

impl TabGroup {
    /// Creates a new tab group with the given ID and tabs.
    pub fn new(id: TabGroupId, tabs: Vec<String>) -> Self {
        Self {
            id,
            tabs,
            active_tab: 0,
        }
    }

    /// Returns true if this group has no tabs.
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Returns the number of tabs in this group.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }
}

/// Hierarchical tree representing tab group splits.
///
/// The center editor area is modeled as a binary tree of splits. Each leaf
/// is a single `TabGroup`; each internal node splits the space between two
/// children in a specified direction.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TabGroupTree {
    /// A leaf node containing a single tab group.
    Leaf(TabGroup),
    /// A split node containing two children with a split direction and proportion.
    Split {
        /// Direction of the split.
        direction: SplitDirection,
        /// Proportion allocated to the first child [0.0, 1.0].
        proportion: f32,
        /// First child (left or top depending on direction).
        first: Box<TabGroupTree>,
        /// Second child (right or bottom depending on direction).
        second: Box<TabGroupTree>,
    },
}

impl TabGroupTree {
    /// Returns the total number of tabs across all groups in this tree.
    pub fn total_tab_count(&self) -> usize {
        match self {
            TabGroupTree::Leaf(group) => group.tab_count(),
            TabGroupTree::Split { first, second, .. } => {
                first.total_tab_count() + second.total_tab_count()
            }
        }
    }

    /// Returns all tab group IDs in this tree.
    pub fn all_group_ids(&self) -> Vec<TabGroupId> {
        match self {
            TabGroupTree::Leaf(group) => vec![group.id],
            TabGroupTree::Split { first, second, .. } => {
                let mut ids = first.all_group_ids();
                ids.extend(second.all_group_ids());
                ids
            }
        }
    }

    /// Returns all tabs across all groups in this tree.
    pub fn all_tabs(&self) -> Vec<&str> {
        match self {
            TabGroupTree::Leaf(group) => group.tabs.iter().map(|s| s.as_str()).collect(),
            TabGroupTree::Split { first, second, .. } => {
                let mut tabs = first.all_tabs();
                tabs.extend(second.all_tabs());
                tabs
            }
        }
    }

    /// Returns a mutable reference to the tab group with the given ID, if found.
    pub fn find_group_mut(&mut self, id: TabGroupId) -> Option<&mut TabGroup> {
        match self {
            TabGroupTree::Leaf(group) => {
                if group.id == id {
                    Some(group)
                } else {
                    None
                }
            }
            TabGroupTree::Split { first, second, .. } => first
                .find_group_mut(id)
                .or_else(|| second.find_group_mut(id)),
        }
    }

    /// Returns an immutable reference to the tab group with the given ID, if found.
    pub fn find_group(&self, id: TabGroupId) -> Option<&TabGroup> {
        match self {
            TabGroupTree::Leaf(group) => {
                if group.id == id {
                    Some(group)
                } else {
                    None
                }
            }
            TabGroupTree::Split { first, second, .. } => {
                first.find_group(id).or_else(|| second.find_group(id))
            }
        }
    }

    /// Returns true if any leaf group has zero tabs.
    pub fn has_empty_groups(&self) -> bool {
        match self {
            TabGroupTree::Leaf(group) => group.is_empty(),
            TabGroupTree::Split { first, second, .. } => {
                first.has_empty_groups() || second.has_empty_groups()
            }
        }
    }

    /// Removes empty groups from the tree, collapsing splits where possible.
    /// Returns None if the entire tree would be empty.
    pub fn remove_empty_groups(self) -> Option<TabGroupTree> {
        match self {
            TabGroupTree::Leaf(group) => {
                if group.is_empty() {
                    None
                } else {
                    Some(TabGroupTree::Leaf(group))
                }
            }
            TabGroupTree::Split {
                direction,
                proportion,
                first,
                second,
            } => {
                let first_cleaned = first.remove_empty_groups();
                let second_cleaned = second.remove_empty_groups();
                match (first_cleaned, second_cleaned) {
                    (Some(f), Some(s)) => Some(TabGroupTree::Split {
                        direction,
                        proportion,
                        first: Box::new(f),
                        second: Box::new(s),
                    }),
                    (Some(f), None) => Some(f),
                    (None, Some(s)) => Some(s),
                    (None, None) => None,
                }
            }
        }
    }
}

/// Direction of a tab group split.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SplitDirection {
    /// Side-by-side (left/right).
    Horizontal,
    /// Stacked (top/bottom).
    Vertical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_group_new_creates_with_zero_active_tab() {
        let group = TabGroup::new(TabGroupId(1), vec!["file1.rs".to_string()]);
        assert_eq!(group.id, TabGroupId(1));
        assert_eq!(group.active_tab, 0);
        assert_eq!(group.tab_count(), 1);
        assert!(!group.is_empty());
    }

    #[test]
    fn tab_group_empty_detection() {
        let group = TabGroup::new(TabGroupId(1), vec![]);
        assert!(group.is_empty());
        assert_eq!(group.tab_count(), 0);
    }

    #[test]
    fn tab_group_tree_leaf_total_count() {
        let tree = TabGroupTree::Leaf(TabGroup::new(
            TabGroupId(1),
            vec!["a.rs".to_string(), "b.rs".to_string()],
        ));
        assert_eq!(tree.total_tab_count(), 2);
    }

    #[test]
    fn tab_group_tree_split_total_count() {
        let tree = TabGroupTree::Split {
            direction: SplitDirection::Horizontal,
            proportion: 0.5,
            first: Box::new(TabGroupTree::Leaf(TabGroup::new(
                TabGroupId(1),
                vec!["a.rs".to_string(), "b.rs".to_string()],
            ))),
            second: Box::new(TabGroupTree::Leaf(TabGroup::new(
                TabGroupId(2),
                vec!["c.rs".to_string()],
            ))),
        };
        assert_eq!(tree.total_tab_count(), 3);
    }

    #[test]
    fn tab_group_tree_remove_empty_groups_collapses_split() {
        let tree = TabGroupTree::Split {
            direction: SplitDirection::Horizontal,
            proportion: 0.5,
            first: Box::new(TabGroupTree::Leaf(TabGroup::new(
                TabGroupId(1),
                vec!["a.rs".to_string()],
            ))),
            second: Box::new(TabGroupTree::Leaf(TabGroup::new(TabGroupId(2), vec![]))),
        };
        let result = tree.remove_empty_groups().unwrap();
        match result {
            TabGroupTree::Leaf(group) => {
                assert_eq!(group.id, TabGroupId(1));
                assert_eq!(group.tabs, vec!["a.rs"]);
            }
            _ => panic!("Expected leaf after removing empty group"),
        }
    }

    #[test]
    fn tab_group_tree_find_group_mut() {
        let mut tree = TabGroupTree::Split {
            direction: SplitDirection::Vertical,
            proportion: 0.5,
            first: Box::new(TabGroupTree::Leaf(TabGroup::new(
                TabGroupId(1),
                vec!["a.rs".to_string()],
            ))),
            second: Box::new(TabGroupTree::Leaf(TabGroup::new(
                TabGroupId(2),
                vec!["b.rs".to_string()],
            ))),
        };
        let group = tree.find_group_mut(TabGroupId(2)).unwrap();
        group.tabs.push("c.rs".to_string());
        assert_eq!(tree.find_group(TabGroupId(2)).unwrap().tab_count(), 2);
    }

    #[test]
    fn split_direction_serialization_round_trip() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Wrapper {
            direction: SplitDirection,
        }

        let directions = [SplitDirection::Horizontal, SplitDirection::Vertical];
        for dir in &directions {
            let wrapper = Wrapper { direction: *dir };
            let serialized = toml::to_string(&wrapper).unwrap();
            let deserialized: Wrapper = toml::from_str(&serialized).unwrap();
            assert_eq!(*dir, deserialized.direction);
        }
    }
}
