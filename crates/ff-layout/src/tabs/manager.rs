//! Tab group manager — coordinates split operations and tab movement.

use crate::error::LayoutError;
use crate::tabs::group::{SplitDirection, TabGroup, TabGroupId, TabGroupTree};
use crate::MIN_TAB_GROUP_SIZE;

/// Manages the center area's tab group split tree.
///
/// Coordinates split operations, tab movement between groups, and
/// empty group elimination.
#[derive(Debug)]
pub struct TabGroupManager {
    /// The tab group tree representing the center area layout.
    tree: TabGroupTree,
    /// The currently active tab group.
    active_group: TabGroupId,
    /// Next ID to assign to a new tab group.
    next_id: u32,
    /// Minimum tab group size in logical pixels.
    #[allow(dead_code)]
    min_group_size: f32,
}

impl TabGroupManager {
    /// Creates a new tab group manager with a single default group.
    pub fn new() -> Self {
        let initial_id = TabGroupId::new(1);
        Self {
            tree: TabGroupTree::Leaf(TabGroup::new(initial_id, vec![])),
            active_group: initial_id,
            next_id: 2,
            min_group_size: MIN_TAB_GROUP_SIZE,
        }
    }

    /// Creates a tab group manager from an existing tree.
    pub fn from_tree(tree: TabGroupTree, active_group: TabGroupId) -> Self {
        let max_id = tree
            .all_group_ids()
            .iter()
            .map(|id| id.value())
            .max()
            .unwrap_or(0);
        Self {
            tree,
            active_group,
            next_id: max_id + 1,
            min_group_size: MIN_TAB_GROUP_SIZE,
        }
    }

    /// Returns a reference to the tab group tree.
    pub fn tree(&self) -> &TabGroupTree {
        &self.tree
    }

    /// Returns a mutable reference to the tab group tree.
    pub fn tree_mut(&mut self) -> &mut TabGroupTree {
        &mut self.tree
    }

    /// Returns the currently active tab group ID.
    pub fn active_group(&self) -> TabGroupId {
        self.active_group
    }

    /// Sets the active tab group.
    pub fn set_active_group(&mut self, group_id: TabGroupId) -> Result<(), LayoutError> {
        if self.tree.find_group(group_id).is_none() {
            return Err(LayoutError::TabGroupNotFound { group_id });
        }
        self.active_group = group_id;
        Ok(())
    }

    /// Splits the active tab group horizontally (side-by-side).
    ///
    /// Moves the active tab from the current group to the new group.
    /// Returns the new group's ID.
    pub fn split_horizontal(&mut self) -> Result<TabGroupId, LayoutError> {
        self.split(SplitDirection::Horizontal)
    }

    /// Splits the active tab group vertically (top/bottom).
    ///
    /// Moves the active tab from the current group to the new group.
    /// Returns the new group's ID.
    pub fn split_vertical(&mut self) -> Result<TabGroupId, LayoutError> {
        self.split(SplitDirection::Vertical)
    }

    /// Adds a tab to the active group or a specified group.
    pub fn add_tab(
        &mut self,
        tab_id: &str,
        target_group: Option<TabGroupId>,
    ) -> Result<(), LayoutError> {
        let group_id = target_group.unwrap_or(self.active_group);
        let group = self
            .tree
            .find_group_mut(group_id)
            .ok_or(LayoutError::TabGroupNotFound { group_id })?;
        group.tabs.push(tab_id.to_string());
        group.active_tab = group.tabs.len() - 1;
        Ok(())
    }

    /// Moves a tab from one group to another.
    ///
    /// If the source group becomes empty, it is removed from the tree.
    pub fn move_tab(
        &mut self,
        source_group: TabGroupId,
        tab_index: usize,
        target_group: TabGroupId,
        insert_index: usize,
    ) -> Result<(), LayoutError> {
        // Validate source
        let source = self
            .tree
            .find_group(source_group)
            .ok_or(LayoutError::TabGroupNotFound {
                group_id: source_group,
            })?;
        if tab_index >= source.tab_count() {
            return Err(LayoutError::TabIndexOutOfBounds {
                group_id: source_group,
                index: tab_index,
                count: source.tab_count(),
            });
        }

        // Validate target exists
        if self.tree.find_group(target_group).is_none() {
            return Err(LayoutError::TabGroupNotFound {
                group_id: target_group,
            });
        }

        // Remove from source
        let tab_id = self
            .tree
            .find_group_mut(source_group)
            .unwrap()
            .tabs
            .remove(tab_index);

        // Fix active_tab for source
        let source = self.tree.find_group_mut(source_group).unwrap();
        if source.active_tab >= source.tabs.len() && !source.tabs.is_empty() {
            source.active_tab = source.tabs.len() - 1;
        }

        // Insert into target
        let target = self.tree.find_group_mut(target_group).unwrap();
        let clamped_index = insert_index.min(target.tabs.len());
        target.tabs.insert(clamped_index, tab_id);
        target.active_tab = clamped_index;

        // Remove empty groups
        self.eliminate_empty_groups();

        Ok(())
    }

    /// Returns the total tab count across all groups.
    pub fn total_tab_count(&self) -> usize {
        self.tree.total_tab_count()
    }

    /// Allocates the next tab group ID.
    fn next_group_id(&mut self) -> TabGroupId {
        let id = TabGroupId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// Performs a split on the active group.
    fn split(&mut self, direction: SplitDirection) -> Result<TabGroupId, LayoutError> {
        let active_group =
            self.tree
                .find_group(self.active_group)
                .ok_or(LayoutError::TabGroupNotFound {
                    group_id: self.active_group,
                })?;

        // Cannot split if the active group has no tabs (nothing to move)
        if active_group.tabs.is_empty() {
            return Err(LayoutError::CannotEmptyEditor);
        }

        // Get the active tab to move
        let active_tab_idx = active_group.active_tab.min(active_group.tabs.len() - 1);
        let tab_to_move = active_group.tabs[active_tab_idx].clone();

        // Create new group with the moved tab
        let new_id = self.next_group_id();
        let new_group = TabGroup::new(new_id, vec![tab_to_move.clone()]);

        // Remove the tab from the active group
        let group = self.tree.find_group_mut(self.active_group).unwrap();
        group.tabs.remove(active_tab_idx);
        if group.active_tab >= group.tabs.len() && !group.tabs.is_empty() {
            group.active_tab = group.tabs.len() - 1;
        }

        // Replace the leaf in the tree with a split node
        self.replace_group_with_split(self.active_group, new_group, direction);

        // Set the new group as active
        self.active_group = new_id;

        Ok(new_id)
    }

    /// Replaces a group in the tree with a split containing the original
    /// and a new group.
    fn replace_group_with_split(
        &mut self,
        original_id: TabGroupId,
        new_group: TabGroup,
        direction: SplitDirection,
    ) {
        self.tree = Self::replace_in_tree(self.tree.clone(), original_id, new_group, direction);
    }

    fn replace_in_tree(
        tree: TabGroupTree,
        target_id: TabGroupId,
        new_group: TabGroup,
        direction: SplitDirection,
    ) -> TabGroupTree {
        match tree {
            TabGroupTree::Leaf(group) if group.id == target_id => TabGroupTree::Split {
                direction,
                proportion: 0.5,
                first: Box::new(TabGroupTree::Leaf(group)),
                second: Box::new(TabGroupTree::Leaf(new_group)),
            },
            TabGroupTree::Leaf(group) => TabGroupTree::Leaf(group),
            TabGroupTree::Split {
                direction: d,
                proportion,
                first,
                second,
            } => TabGroupTree::Split {
                direction: d,
                proportion,
                first: Box::new(Self::replace_in_tree(
                    *first,
                    target_id,
                    new_group.clone(),
                    direction,
                )),
                second: Box::new(Self::replace_in_tree(
                    *second, target_id, new_group, direction,
                )),
            },
        }
    }

    /// Eliminates empty groups from the tree.
    fn eliminate_empty_groups(&mut self) {
        if let Some(new_tree) = self.tree.clone().remove_empty_groups() {
            self.tree = new_tree;
            // Ensure active group still exists
            if self.tree.find_group(self.active_group).is_none() {
                // Fall back to the first available group
                if let Some(first_id) = self.tree.all_group_ids().into_iter().next() {
                    self.active_group = first_id;
                }
            }
        }
    }
}

impl Default for TabGroupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager_has_single_empty_group() {
        let mgr = TabGroupManager::new();
        assert_eq!(mgr.total_tab_count(), 0);
        assert_eq!(mgr.tree().all_group_ids().len(), 1);
    }

    #[test]
    fn add_tab_to_active_group() {
        // Validates: Requirement 2 criterion 9
        let mut mgr = TabGroupManager::new();
        mgr.add_tab("file1.rs", None).unwrap();
        assert_eq!(mgr.total_tab_count(), 1);
    }

    #[test]
    fn split_horizontal_preserves_tab_count() {
        // Validates: Requirement 2 criterion 2
        let mut mgr = TabGroupManager::new();
        mgr.add_tab("file1.rs", None).unwrap();
        mgr.add_tab("file2.rs", None).unwrap();
        mgr.add_tab("file3.rs", None).unwrap();

        let original_count = mgr.total_tab_count();
        mgr.split_horizontal().unwrap();

        assert_eq!(mgr.total_tab_count(), original_count);
        assert_eq!(mgr.tree().all_group_ids().len(), 2);
    }

    #[test]
    fn split_vertical_preserves_tab_count() {
        // Validates: Requirement 2 criterion 3
        let mut mgr = TabGroupManager::new();
        mgr.add_tab("file1.rs", None).unwrap();
        mgr.add_tab("file2.rs", None).unwrap();

        let original_count = mgr.total_tab_count();
        mgr.split_vertical().unwrap();

        assert_eq!(mgr.total_tab_count(), original_count);
    }

    #[test]
    fn split_empty_group_returns_error() {
        let mut mgr = TabGroupManager::new();
        let result = mgr.split_horizontal();
        assert!(matches!(result, Err(LayoutError::CannotEmptyEditor)));
    }

    #[test]
    fn move_tab_between_groups() {
        // Validates: Requirement 2 criterion 4
        let mut mgr = TabGroupManager::new();
        mgr.add_tab("file1.rs", None).unwrap();
        mgr.add_tab("file2.rs", None).unwrap();

        // Split to create two groups
        let new_id = mgr.split_horizontal().unwrap();
        let original_id = mgr
            .tree()
            .all_group_ids()
            .iter()
            .find(|id| **id != new_id)
            .copied()
            .unwrap();

        // Move tab from new group back to original
        let total_before = mgr.total_tab_count();
        mgr.move_tab(new_id, 0, original_id, 0).unwrap();
        assert_eq!(mgr.total_tab_count(), total_before);
    }

    #[test]
    fn move_last_tab_eliminates_empty_group() {
        // Validates: Requirement 2 criterion 5
        let mut mgr = TabGroupManager::new();
        mgr.add_tab("file1.rs", None).unwrap();
        mgr.add_tab("file2.rs", None).unwrap();

        let new_id = mgr.split_horizontal().unwrap();
        let original_id = mgr
            .tree()
            .all_group_ids()
            .iter()
            .find(|id| **id != new_id)
            .copied()
            .unwrap();

        // New group has 1 tab; move it away
        mgr.move_tab(new_id, 0, original_id, 0).unwrap();

        // Empty group should be eliminated
        assert!(!mgr.tree().has_empty_groups());
        assert_eq!(mgr.tree().all_group_ids().len(), 1);
    }

    #[test]
    fn set_active_group_validates_existence() {
        let mut mgr = TabGroupManager::new();
        let result = mgr.set_active_group(TabGroupId::new(999));
        assert!(matches!(result, Err(LayoutError::TabGroupNotFound { .. })));
    }
}
