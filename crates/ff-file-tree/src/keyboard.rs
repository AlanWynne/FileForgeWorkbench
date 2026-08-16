//! KeyboardHandler — processes keyboard input for tree navigation and actions.

use crate::node::NodeId;
use crate::state::TreeState;
use std::time::{Duration, Instant};

/// Actions the keyboard handler can request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeAction {
    SelectNext,
    SelectPrevious,
    Expand(NodeId),
    Collapse(NodeId),
    SelectFirstChild(NodeId),
    SelectParent(NodeId),
    Open(NodeId),
    ToggleExpand(NodeId),
    Delete(NodeId),
    Rename(NodeId),
    SelectFirst,
    SelectLast,
    TypeAheadJump(String),
}

/// Key events the handler understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyEvent {
    ArrowDown,
    ArrowUp,
    ArrowRight,
    ArrowLeft,
    Enter,
    Delete,
    F2,
    Home,
    End,
    Char(char),
}

/// Processes keyboard input for tree navigation and actions.
pub struct KeyboardHandler {
    /// Type-ahead buffer for incremental search.
    type_ahead_buffer: String,
    /// Timestamp of last type-ahead keystroke.
    type_ahead_last: Option<Instant>,
}

/// Timeout after which the type-ahead buffer is reset.
const TYPE_AHEAD_TIMEOUT: Duration = Duration::from_millis(800);

impl KeyboardHandler {
    /// Create a new KeyboardHandler.
    pub fn new() -> Self {
        Self {
            type_ahead_buffer: String::new(),
            type_ahead_last: None,
        }
    }

    /// Process a key event. Returns an action to perform (if any).
    pub fn handle_key(&mut self, key: KeyEvent, state: &TreeState) -> Option<TreeAction> {
        let selected = state.selected();

        match key {
            KeyEvent::ArrowDown => Some(TreeAction::SelectNext),
            KeyEvent::ArrowUp => Some(TreeAction::SelectPrevious),

            KeyEvent::ArrowRight => {
                let id = selected?;
                let node = state.get_node(id)?;
                if node.node_type.is_expandable() {
                    if node.expanded {
                        // Move to first child if any
                        node.children
                            .first()
                            .copied()
                            .map(TreeAction::SelectFirstChild)
                    } else {
                        Some(TreeAction::Expand(id))
                    }
                } else {
                    None
                }
            }

            KeyEvent::ArrowLeft => {
                let id = selected?;
                let node = state.get_node(id)?;
                if node.node_type.is_expandable() && node.expanded {
                    Some(TreeAction::Collapse(id))
                } else {
                    // Move to parent
                    let parent = node.parent;
                    if parent == NodeId::ROOT {
                        None
                    } else {
                        Some(TreeAction::SelectParent(parent))
                    }
                }
            }

            KeyEvent::Enter => {
                let id = selected?;
                let node = state.get_node(id)?;
                if node.node_type.is_expandable() {
                    Some(TreeAction::ToggleExpand(id))
                } else {
                    Some(TreeAction::Open(id))
                }
            }

            KeyEvent::Delete => selected.map(TreeAction::Delete),

            KeyEvent::F2 => selected.map(TreeAction::Rename),

            KeyEvent::Home => Some(TreeAction::SelectFirst),

            KeyEvent::End => Some(TreeAction::SelectLast),

            KeyEvent::Char(c) if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' => {
                // Reset buffer if timeout elapsed
                if let Some(last) = self.type_ahead_last {
                    if last.elapsed() > TYPE_AHEAD_TIMEOUT {
                        self.type_ahead_buffer.clear();
                    }
                }
                self.type_ahead_buffer.push(c);
                self.type_ahead_last = Some(Instant::now());
                Some(TreeAction::TypeAheadJump(self.type_ahead_buffer.clone()))
            }

            KeyEvent::Char(_) => None,
        }
    }

    /// Clear the type-ahead buffer.
    pub fn clear_type_ahead(&mut self) {
        self.type_ahead_buffer.clear();
        self.type_ahead_last = None;
    }

    /// Current type-ahead buffer content.
    pub fn type_ahead_buffer(&self) -> &str {
        &self.type_ahead_buffer
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Navigate to the next visible node after `current` in display order.
pub fn next_visible_node(state: &TreeState, current: NodeId) -> Option<NodeId> {
    let visible = state.visible_nodes_iter();
    let mut found = false;
    for node in visible {
        if found {
            return Some(node.id);
        }
        if node.id == current {
            found = true;
        }
    }
    None
}

/// Navigate to the previous visible node before `current` in display order.
pub fn prev_visible_node(state: &TreeState, current: NodeId) -> Option<NodeId> {
    let visible = state.visible_nodes_iter();
    let mut prev = None;
    for node in visible {
        if node.id == current {
            return prev;
        }
        prev = Some(node.id);
    }
    None
}

/// Find the first visible node in the tree.
pub fn first_visible_node(state: &TreeState) -> Option<NodeId> {
    state.visible_nodes_iter().first().map(|n| n.id)
}

/// Find the last visible node in the tree.
pub fn last_visible_node(state: &TreeState) -> Option<NodeId> {
    state.visible_nodes_iter().last().map(|n| n.id)
}

/// Find the next sibling whose label starts with the given prefix (type-ahead).
pub fn type_ahead_jump(state: &TreeState, current: NodeId, prefix: &str) -> Option<NodeId> {
    let prefix_lower = prefix.to_ascii_lowercase();
    let visible = state.visible_nodes_iter();
    // Search from node after current, wrapping around
    let mut past_current = false;
    let mut first_match: Option<NodeId> = None;

    for node in &visible {
        let matches = node.label.to_ascii_lowercase().starts_with(&prefix_lower);
        if node.id == current {
            past_current = true;
            continue;
        }
        if matches {
            if past_current {
                return Some(node.id);
            } else if first_match.is_none() {
                first_match = Some(node.id);
            }
        }
    }
    first_match
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{NodeType, TreeNodeData};
    use crate::state::TreeState;

    fn setup_tree() -> (TreeState, NodeId, NodeId, NodeId) {
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        state.apply_children(
            local,
            vec![
                TreeNodeData::file("alpha.rs"),
                TreeNodeData::file("beta.rs"),
            ],
        );
        let children = state.get_node(local).unwrap().children.clone();
        (state, local, children[0], children[1])
    }

    #[test]
    fn arrow_down_returns_select_next() {
        // Validates: Requirement 8.1 — Down Arrow moves selection down
        let (state, local, _, _) = setup_tree();
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::ArrowDown, &state);
        assert_eq!(action, Some(TreeAction::SelectNext));
    }

    #[test]
    fn arrow_up_returns_select_previous() {
        // Validates: Requirement 8.2 — Up Arrow moves selection up
        let (state, _, _, _) = setup_tree();
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::ArrowUp, &state);
        assert_eq!(action, Some(TreeAction::SelectPrevious));
    }

    #[test]
    fn arrow_right_on_collapsed_expandable_returns_expand() {
        // Validates: Requirement 8.3 — Right Arrow expands collapsed directory
        let mut state = TreeState::new();
        let local = state.root_categories[0];
        state.get_node_mut(local).unwrap().expanded = false;
        state.select(Some(local));
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::ArrowRight, &state);
        assert_eq!(action, Some(TreeAction::Expand(local)));
    }

    #[test]
    fn arrow_right_on_expanded_node_moves_to_first_child() {
        // Validates: Requirement 8.4 — Right Arrow on expanded node moves to first child
        let (mut state, local, child_a, _) = setup_tree();
        state.select(Some(local));
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::ArrowRight, &state);
        assert_eq!(action, Some(TreeAction::SelectFirstChild(child_a)));
    }

    #[test]
    fn arrow_left_on_expanded_directory_collapses() {
        // Validates: Requirement 8.5 — Left Arrow collapses expanded directory
        let (mut state, local, _, _) = setup_tree();
        state.select(Some(local));
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::ArrowLeft, &state);
        assert_eq!(action, Some(TreeAction::Collapse(local)));
    }

    #[test]
    fn arrow_left_on_child_moves_to_parent() {
        // Validates: Requirement 8.6 — Left Arrow on child moves to parent
        let (mut state, local, child_a, _) = setup_tree();
        state.select(Some(child_a));
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::ArrowLeft, &state);
        assert_eq!(action, Some(TreeAction::SelectParent(local)));
    }

    #[test]
    fn enter_on_file_opens_file() {
        // Validates: Requirement 8.7 — Enter on file node opens file
        let (mut state, _, child_a, _) = setup_tree();
        state.select(Some(child_a));
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::Enter, &state);
        assert_eq!(action, Some(TreeAction::Open(child_a)));
    }

    #[test]
    fn enter_on_directory_toggles_expansion() {
        // Validates: Requirement 8.8 — Enter on directory toggles expansion
        let (mut state, local, _, _) = setup_tree();
        state.select(Some(local));
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::Enter, &state);
        assert_eq!(action, Some(TreeAction::ToggleExpand(local)));
    }

    #[test]
    fn delete_key_triggers_delete_action() {
        // Validates: Requirement 8.9 — Delete key triggers delete confirmation
        let (mut state, _, child_a, _) = setup_tree();
        state.select(Some(child_a));
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::Delete, &state);
        assert_eq!(action, Some(TreeAction::Delete(child_a)));
    }

    #[test]
    fn f2_triggers_rename_action() {
        // Validates: Requirement 8.10 — F2 activates inline rename
        let (mut state, _, child_a, _) = setup_tree();
        state.select(Some(child_a));
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::F2, &state);
        assert_eq!(action, Some(TreeAction::Rename(child_a)));
    }

    #[test]
    fn home_returns_select_first() {
        // Validates: Requirement 8.11 — Home moves to first visible node
        let (state, _, _, _) = setup_tree();
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::Home, &state);
        assert_eq!(action, Some(TreeAction::SelectFirst));
    }

    #[test]
    fn end_returns_select_last() {
        // Validates: Requirement 8.11 — End moves to last visible node
        let (state, _, _, _) = setup_tree();
        let mut handler = KeyboardHandler::new();
        let action = handler.handle_key(KeyEvent::End, &state);
        assert_eq!(action, Some(TreeAction::SelectLast));
    }

    #[test]
    fn alphanumeric_char_builds_type_ahead_buffer() {
        // Validates: Requirement 8.12 — type-ahead search
        let (state, _, _, _) = setup_tree();
        let mut handler = KeyboardHandler::new();
        handler.handle_key(KeyEvent::Char('a'), &state);
        handler.handle_key(KeyEvent::Char('l'), &state);
        assert_eq!(handler.type_ahead_buffer(), "al");
    }

    #[test]
    fn next_visible_node_advances_in_order() {
        // Validates: Requirement 8.1 — Down Arrow visits nodes in display order
        let (mut state, local, child_a, child_b) = setup_tree();
        // local -> child_a -> child_b in display order
        let next = next_visible_node(&state, local);
        assert_eq!(next, Some(child_a));
        let next2 = next_visible_node(&state, child_a);
        assert_eq!(next2, Some(child_b));
    }

    #[test]
    fn prev_visible_node_goes_backwards() {
        // Validates: Requirement 8.2 — Up Arrow visits nodes in reverse order
        let (mut state, local, child_a, child_b) = setup_tree();
        let prev = prev_visible_node(&state, child_b);
        assert_eq!(prev, Some(child_a));
        let prev2 = prev_visible_node(&state, child_a);
        assert_eq!(prev2, Some(local));
    }

    #[test]
    fn type_ahead_jump_finds_next_sibling_with_prefix() {
        // Validates: Requirement 8.12 — type-ahead jumps to matching sibling
        let (mut state, local, child_a, child_b) = setup_tree();
        // child_a = "alpha.rs", child_b = "beta.rs"
        // From local, jump to "al" prefix -> child_a
        let result = type_ahead_jump(&state, local, "al");
        assert_eq!(result, Some(child_a));
        // From child_a, jump to "be" prefix -> child_b
        let result2 = type_ahead_jump(&state, child_a, "be");
        assert_eq!(result2, Some(child_b));
    }
}
