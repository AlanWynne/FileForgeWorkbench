//! TreeState — the complete in-memory tree model.

use std::collections::{HashMap, HashSet};

use crate::node::{FileCategory, NodeId, NodeType, TreeNode, TreeNodeData};

/// The complete in-memory tree model. Owns all nodes, manages expansion,
/// selection, caching, and provides query methods for the renderer.
pub struct TreeState {
    /// All nodes indexed by ID for O(1) access.
    nodes: HashMap<NodeId, TreeNode>,
    /// Counter for generating unique NodeIds (starts at 1; 0 is ROOT sentinel).
    next_id: u64,
    /// Currently selected node (single selection).
    selected: Option<NodeId>,
    /// The three top-level category node IDs [LocalFiles, Catalogs, Connections].
    pub root_categories: [NodeId; 3],
    /// Pre-filter expansion state (saved when filter activates).
    pre_filter_expansion: Option<HashMap<NodeId, bool>>,
    /// Whether a search filter is currently active.
    pub filter_active: bool,
    /// Set of node IDs visible under the current filter (None = all visible).
    pub visible_nodes: Option<HashSet<NodeId>>,
    /// Whether to show hidden files.
    pub show_hidden_files: bool,
}

impl TreeState {
    /// Create initial state with three empty root categories.
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        let mut next_id = 1u64;

        let make_root = |id: u64, label: &str| TreeNode {
            id: NodeId(id),
            parent: NodeId::ROOT,
            label: label.to_string(),
            node_type: NodeType::RootCategory,
            expanded: true,
            loading: false,
            children: Vec::new(),
            children_loaded: false,
            size: None,
            category: FileCategory::Directory,
            has_structure: false,
            is_hidden: false,
            depth: 0,
        };

        let local_id = NodeId(next_id);
        nodes.insert(local_id, make_root(next_id, "Local Files"));
        next_id += 1;

        let catalogs_id = NodeId(next_id);
        nodes.insert(catalogs_id, make_root(next_id, "Catalogs"));
        next_id += 1;

        let connections_id = NodeId(next_id);
        nodes.insert(connections_id, make_root(next_id, "Connections"));
        next_id += 1;

        Self {
            nodes,
            next_id,
            selected: None,
            root_categories: [local_id, catalogs_id, connections_id],
            pre_filter_expansion: None,
            filter_active: false,
            visible_nodes: None,
            show_hidden_files: false,
        }
    }

    fn alloc_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Insert a new node as a child of `parent`. Returns its NodeId.
    pub fn insert_node(&mut self, parent: NodeId, mut node: TreeNode) -> NodeId {
        let id = self.alloc_id();
        node.id = id;
        node.parent = parent;
        if let Some(p) = self.nodes.get_mut(&parent) {
            p.children.push(id);
        }
        self.nodes.insert(id, node);
        id
    }

    /// Remove a node and all its descendants.
    pub fn remove_node(&mut self, id: NodeId) {
        let children: Vec<NodeId> = self
            .nodes
            .get(&id)
            .map(|n| n.children.clone())
            .unwrap_or_default();
        for child in children {
            self.remove_node(child);
        }
        if let Some(node) = self.nodes.remove(&id) {
            if let Some(parent) = self.nodes.get_mut(&node.parent) {
                parent.children.retain(|&c| c != id);
            }
        }
        if self.selected == Some(id) {
            self.selected = None;
        }
    }

    /// Get a reference to a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&TreeNode> {
        self.nodes.get(&id)
    }

    /// Get a mutable reference to a node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut TreeNode> {
        self.nodes.get_mut(&id)
    }

    /// Toggle expansion state of a node.
    pub fn toggle_expand(&mut self, id: NodeId) {
        if let Some(node) = self.nodes.get_mut(&id) {
            if node.node_type.is_expandable() {
                node.expanded = !node.expanded;
            }
        }
    }

    /// Apply loaded children to a node (replaces loading indicator).
    pub fn apply_children(&mut self, parent: NodeId, entries: Vec<TreeNodeData>) {
        // Remove existing children first
        let old_children: Vec<NodeId> = self
            .nodes
            .get(&parent)
            .map(|n| n.children.clone())
            .unwrap_or_default();
        for child in old_children {
            self.remove_node(child);
        }

        let parent_depth = self.nodes.get(&parent).map(|n| n.depth).unwrap_or(0);

        for data in entries {
            let id = self.alloc_id();
            let node = TreeNode {
                id,
                parent,
                label: data.label.clone(),
                node_type: data.node_type,
                expanded: false,
                loading: false,
                children: Vec::new(),
                children_loaded: false,
                size: data.size,
                category: data.category,
                has_structure: data.has_structure,
                is_hidden: data.is_hidden,
                depth: parent_depth + 1,
            };
            self.nodes.insert(id, node);
            if let Some(p) = self.nodes.get_mut(&parent) {
                p.children.push(id);
            }
        }

        if let Some(p) = self.nodes.get_mut(&parent) {
            p.children_loaded = true;
            p.loading = false;
            p.expanded = true;
        }
    }

    /// Apply an error result to a node (replaces loading indicator with error node).
    pub fn apply_error(&mut self, parent: NodeId, message: String) {
        let old_children: Vec<NodeId> = self
            .nodes
            .get(&parent)
            .map(|n| n.children.clone())
            .unwrap_or_default();
        for child in old_children {
            self.remove_node(child);
        }

        let parent_depth = self.nodes.get(&parent).map(|n| n.depth).unwrap_or(0);
        let id = self.alloc_id();
        let error_node = TreeNode {
            id,
            parent,
            label: message,
            node_type: NodeType::ErrorIndicator,
            expanded: false,
            loading: false,
            children: Vec::new(),
            children_loaded: false,
            size: None,
            category: FileCategory::Unknown,
            has_structure: false,
            is_hidden: false,
            depth: parent_depth + 1,
        };
        self.nodes.insert(id, error_node);
        if let Some(p) = self.nodes.get_mut(&parent) {
            p.children.push(id);
            p.loading = false;
        }
    }

    /// Get the currently selected node ID.
    pub fn selected(&self) -> Option<NodeId> {
        self.selected
    }

    /// Set selection to a specific node.
    pub fn select(&mut self, id: Option<NodeId>) {
        self.selected = id;
    }

    /// Returns visible nodes in display order (depth-first, respecting expansion
    /// and filter state). Hidden files are excluded when `show_hidden_files` is false.
    pub fn visible_nodes_iter(&self) -> Vec<&TreeNode> {
        let mut result = Vec::new();
        for &root_id in &self.root_categories {
            self.collect_visible(root_id, &mut result);
        }
        result
    }

    fn collect_visible<'a>(&'a self, id: NodeId, out: &mut Vec<&'a TreeNode>) {
        let node = match self.nodes.get(&id) {
            Some(n) => n,
            None => return,
        };

        // Apply hidden file filter
        if node.is_hidden && !self.show_hidden_files && node.depth > 0 {
            return;
        }

        // Apply search filter
        if let Some(ref visible) = self.visible_nodes {
            if !visible.contains(&id) {
                return;
            }
        }

        out.push(node);

        if node.expanded {
            for &child_id in &node.children {
                self.collect_visible(child_id, out);
            }
        }
    }

    /// Invalidate cached children for a node (triggers reload on next expand).
    pub fn invalidate_cache(&mut self, id: NodeId) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.children_loaded = false;
        }
    }

    /// Invalidate all caches (full refresh).
    pub fn invalidate_all_caches(&mut self) {
        for node in self.nodes.values_mut() {
            node.children_loaded = false;
        }
    }

    /// Total number of nodes in the tree (including root categories).
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Save current expansion state before applying a filter.
    pub fn save_expansion_state(&mut self) {
        let state: HashMap<NodeId, bool> =
            self.nodes.iter().map(|(&id, n)| (id, n.expanded)).collect();
        self.pre_filter_expansion = Some(state);
    }

    /// Restore expansion state saved before a filter was applied.
    pub fn restore_expansion_state(&mut self) {
        if let Some(saved) = self.pre_filter_expansion.take() {
            for (id, expanded) in saved {
                if let Some(node) = self.nodes.get_mut(&id) {
                    node.expanded = expanded;
                }
            }
        }
        self.filter_active = false;
        self.visible_nodes = None;
    }

    /// Expand all ancestors of the given node (used during filter application).
    pub fn expand_ancestors(&mut self, id: NodeId) {
        let mut current = id;
        while let Some(n) = self.nodes.get(&current) {
            let parent = n.parent;
            if parent == NodeId::ROOT {
                break;
            }
            if let Some(p) = self.nodes.get_mut(&parent) {
                p.expanded = true;
            }
            current = parent;
        }
    }

    /// Returns all node IDs in the tree.
    pub fn all_node_ids(&self) -> Vec<NodeId> {
        self.nodes.keys().copied().collect()
    }
}

impl Default for TreeState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{NodeType, TreeNodeData};

    fn make_dir_node(label: &str, parent: NodeId, depth: u32) -> TreeNode {
        TreeNode::new(NodeId(0), parent, label, NodeType::Directory, depth)
    }

    #[test]
    fn new_state_has_three_root_categories() {
        // Validates: Requirement 2.1 — three top-level root categories
        let state = TreeState::new();
        assert_eq!(state.root_categories.len(), 3);
        let labels: Vec<&str> = state
            .root_categories
            .iter()
            .map(|&id| state.get_node(id).unwrap().label.as_str())
            .collect();
        assert_eq!(labels, ["Local Files", "Catalogs", "Connections"]);
    }

    #[test]
    fn insert_node_adds_to_parent_children() {
        // Validates: Requirement 2.1 — hierarchy integrity
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        let node = make_dir_node("workspace", local, 1);
        let id = state.insert_node(local, node);
        assert!(state.get_node(local).unwrap().children.contains(&id));
        assert_eq!(state.get_node(id).unwrap().parent, local);
    }

    #[test]
    fn remove_node_removes_from_parent_and_descendants() {
        // Validates: Requirement 2.1 — remove cleans up hierarchy
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        let parent_node = make_dir_node("parent", local, 1);
        let parent_id = state.insert_node(local, parent_node);
        let child_node = make_dir_node("child", parent_id, 2);
        let child_id = state.insert_node(parent_id, child_node);

        state.remove_node(parent_id);

        assert!(state.get_node(parent_id).is_none());
        assert!(state.get_node(child_id).is_none());
        assert!(!state.get_node(local).unwrap().children.contains(&parent_id));
    }

    #[test]
    fn toggle_expand_flips_state() {
        // Validates: Requirement 2.7 — section expand/collapse
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        // Root categories start expanded
        assert!(state.get_node(local).unwrap().expanded);
        state.toggle_expand(local);
        assert!(!state.get_node(local).unwrap().expanded);
        state.toggle_expand(local);
        assert!(state.get_node(local).unwrap().expanded);
    }

    #[test]
    fn apply_children_replaces_loading_indicator() {
        // Validates: Requirement 3.3 — replace Loading_Indicator with entries
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        let entries = vec![
            TreeNodeData::directory("src"),
            TreeNodeData::file("main.rs"),
        ];
        state.apply_children(local, entries);
        let children = &state.get_node(local).unwrap().children;
        assert_eq!(children.len(), 2);
        assert!(state.get_node(local).unwrap().children_loaded);
        assert!(state.get_node(local).unwrap().expanded);
    }

    #[test]
    fn apply_error_inserts_error_node() {
        // Validates: Requirement 3.4 — error node on VFS failure
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        state.apply_error(local, "Permission denied".to_string());
        let children = &state.get_node(local).unwrap().children;
        assert_eq!(children.len(), 1);
        let error_node = state.get_node(children[0]).unwrap();
        assert_eq!(error_node.node_type, NodeType::ErrorIndicator);
        assert_eq!(error_node.label, "Permission denied");
    }

    #[test]
    fn visible_nodes_iter_respects_expansion() {
        // Validates: Requirement 3.1 — non-blocking expansion
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        // Collapse local files
        state.get_node_mut(local).unwrap().expanded = false;
        let entries = vec![TreeNodeData::directory("src")];
        state.apply_children(local, entries);
        // After apply_children, expanded = true; collapse again
        state.get_node_mut(local).unwrap().expanded = false;

        let visible: Vec<_> = state.visible_nodes_iter();
        // Should see all 3 root categories but not the child
        assert_eq!(visible.len(), 3);
    }

    #[test]
    fn hidden_files_excluded_when_show_hidden_false() {
        // Validates: Requirement 4.7 — hidden files filtered
        let mut state = TreeState::new();
        state.show_hidden_files = false;
        let local = state.root_categories[0];
        let entries = vec![
            TreeNodeData::file(".hidden"),
            TreeNodeData::file("visible.rs"),
        ];
        state.apply_children(local, entries);
        let visible: Vec<_> = state.visible_nodes_iter();
        let labels: Vec<&str> = visible.iter().map(|n| n.label.as_str()).collect();
        assert!(!labels.contains(&".hidden"));
        assert!(labels.contains(&"visible.rs"));
    }

    #[test]
    fn hidden_files_shown_when_show_hidden_true() {
        // Validates: Requirement 4.7 — hidden files shown when enabled
        let mut state = TreeState::new();
        state.show_hidden_files = true;
        let local = state.root_categories[0];
        let entries = vec![
            TreeNodeData::file(".hidden"),
            TreeNodeData::file("visible.rs"),
        ];
        state.apply_children(local, entries);
        let visible: Vec<_> = state.visible_nodes_iter();
        let labels: Vec<&str> = visible.iter().map(|n| n.label.as_str()).collect();
        assert!(labels.contains(&".hidden"));
        assert!(labels.contains(&"visible.rs"));
    }

    #[test]
    fn selection_set_and_cleared() {
        // Validates: Requirement 7.6 — single-click selection
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        state.select(Some(local));
        assert_eq!(state.selected(), Some(local));
        state.select(None);
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn invalidate_cache_clears_children_loaded() {
        // Validates: Requirement 12.2 — full refresh invalidates caches
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        state.apply_children(local, vec![TreeNodeData::file("a.txt")]);
        assert!(state.get_node(local).unwrap().children_loaded);
        state.invalidate_cache(local);
        assert!(!state.get_node(local).unwrap().children_loaded);
    }

    #[test]
    fn invalidate_all_caches_clears_all_nodes() {
        // Validates: Requirement 12.1 — refresh command invalidates all caches
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        state.apply_children(local, vec![TreeNodeData::file("a.txt")]);
        state.invalidate_all_caches();
        for id in state.all_node_ids() {
            assert!(!state.get_node(id).unwrap().children_loaded);
        }
    }

    #[test]
    fn remove_node_clears_selection() {
        // Validates: Requirement 12.5 — selection cleared when node deleted
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        let node = make_dir_node("target", local, 1);
        let id = state.insert_node(local, node);
        state.select(Some(id));
        state.remove_node(id);
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn expand_ancestors_expands_parent_chain() {
        // Validates: Requirement 9.3 — ancestor expansion during filter
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        let parent_node = make_dir_node("parent", local, 1);
        let parent_id = state.insert_node(local, parent_node);
        let child_node = make_dir_node("child", parent_id, 2);
        let child_id = state.insert_node(parent_id, child_node);

        // Collapse parent
        state.get_node_mut(parent_id).unwrap().expanded = false;
        state.expand_ancestors(child_id);
        assert!(state.get_node(parent_id).unwrap().expanded);
    }
}
