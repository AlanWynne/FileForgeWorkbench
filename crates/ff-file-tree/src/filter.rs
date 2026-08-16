//! FilterEngine — search/glob pattern matching for tree nodes.

use crate::node::NodeId;
use crate::state::TreeState;
use std::collections::HashSet;

/// The current state of the search/filter box.
#[derive(Debug, Clone)]
pub struct FilterState {
    /// The current filter text (empty = no filter).
    pub text: String,
    /// Whether the filter text contains glob characters (* or ?).
    pub is_glob: bool,
}

impl FilterState {
    /// Create an empty (inactive) filter state.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            is_glob: false,
        }
    }

    /// Set the filter text, detecting glob patterns.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.is_glob = text.contains('*') || text.contains('?');
    }

    /// Returns true if a filter is currently active.
    pub fn is_active(&self) -> bool {
        !self.text.is_empty()
    }

    /// Returns true if the given label matches this filter.
    pub fn matches(&self, label: &str) -> bool {
        if self.text.is_empty() {
            return true;
        }
        if self.is_glob {
            glob_match(&self.text, label)
        } else {
            label
                .to_ascii_lowercase()
                .contains(&self.text.to_ascii_lowercase())
        }
    }

    /// Clear the filter.
    pub fn clear(&mut self) {
        self.text.clear();
        self.is_glob = false;
    }
}

impl Default for FilterState {
    fn default() -> Self {
        Self::new()
    }
}

/// Applies search text or glob patterns to compute visible node set.
/// Operates on cached tree data only — never triggers VFS operations.
pub struct FilterEngine;

impl FilterEngine {
    /// Compute the set of visible node IDs given a filter state.
    ///
    /// A node is visible if:
    /// - It matches the filter, OR
    /// - It is an ancestor of a matching node
    pub fn compute_visible_set(state: &TreeState, filter: &FilterState) -> HashSet<NodeId> {
        let mut visible = HashSet::new();

        if !filter.is_active() {
            // All nodes visible
            for id in state.all_node_ids() {
                visible.insert(id);
            }
            return visible;
        }

        // First pass: find all matching nodes
        let matching: Vec<NodeId> = state
            .all_node_ids()
            .into_iter()
            .filter(|&id| {
                state
                    .get_node(id)
                    .map(|n| filter.matches(&n.label))
                    .unwrap_or(false)
            })
            .collect();

        // Second pass: add matching nodes and all their ancestors
        for id in matching {
            visible.insert(id);
            // Walk up the ancestor chain
            let mut current = id;
            while let Some(n) = state.get_node(current) {
                let parent = n.parent;
                if parent == NodeId::ROOT {
                    break;
                }
                if !visible.insert(parent) {
                    break;
                }
                current = parent;
            }
        }

        visible
    }
}

/// Simple glob matching supporting `*` (zero or more chars) and `?` (exactly one char).
/// Case-insensitive.
fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.to_ascii_lowercase();
    let text = text.to_ascii_lowercase();
    glob_match_inner(pattern.as_bytes(), text.as_bytes())
}

fn glob_match_inner(pattern: &[u8], text: &[u8]) -> bool {
    match (pattern.first(), text.first()) {
        (None, None) => true,
        (Some(&b'*'), _) => {
            // * matches zero chars
            glob_match_inner(&pattern[1..], text)
                // * matches one or more chars
                || (!text.is_empty() && glob_match_inner(pattern, &text[1..]))
        }
        (Some(&b'?'), Some(_)) => glob_match_inner(&pattern[1..], &text[1..]),
        (Some(p), Some(t)) if p == t => glob_match_inner(&pattern[1..], &text[1..]),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_state_inactive_when_empty() {
        // Validates: Requirement 9.1 — search box inactive by default
        let f = FilterState::new();
        assert!(!f.is_active());
    }

    #[test]
    fn filter_state_active_after_set_text() {
        // Validates: Requirement 9.2 — filter activates on text input
        let mut f = FilterState::new();
        f.set_text("main");
        assert!(f.is_active());
    }

    #[test]
    fn filter_state_detects_glob_chars() {
        // Validates: Requirement 9.6 — glob pattern detection
        let mut f = FilterState::new();
        f.set_text("*.rs");
        assert!(f.is_glob);
        f.set_text("main?.rs");
        assert!(f.is_glob);
        f.set_text("main");
        assert!(!f.is_glob);
    }

    #[test]
    fn substring_match_case_insensitive() {
        // Validates: Requirement 9.2 — case-insensitive substring match
        let mut f = FilterState::new();
        f.set_text("MAIN");
        assert!(f.matches("main.rs"));
        assert!(f.matches("Main.rs"));
        assert!(!f.matches("other.rs"));
    }

    #[test]
    fn glob_star_matches_zero_or_more() {
        // Validates: Requirement 9.6 — * matches zero or more chars
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("*.rs", ".rs"));
        assert!(!glob_match("*.rs", "main.go"));
        assert!(glob_match("main*", "main.rs"));
        assert!(glob_match("main*", "main"));
    }

    #[test]
    fn glob_question_matches_exactly_one() {
        // Validates: Requirement 9.6 — ? matches exactly one char
        assert!(glob_match("main.?s", "main.rs"));
        assert!(glob_match("main.?s", "main.ts"));
        assert!(!glob_match("main.?s", "main.rs2"));
        assert!(!glob_match("?.rs", ".rs")); // ? requires exactly one char
    }

    #[test]
    fn glob_match_case_insensitive() {
        // Validates: Requirement 9.6 — glob is case-insensitive
        assert!(glob_match("*.RS", "main.rs"));
        assert!(glob_match("MAIN*", "main.rs"));
    }

    #[test]
    fn filter_clear_deactivates() {
        // Validates: Requirement 9.5 — clearing filter restores full tree
        let mut f = FilterState::new();
        f.set_text("test");
        f.clear();
        assert!(!f.is_active());
    }

    #[test]
    fn compute_visible_set_empty_filter_returns_all() {
        // Validates: Requirement 9.7 — no filter = all nodes visible
        use crate::state::TreeState;
        let state = TreeState::new();
        let filter = FilterState::new();
        let visible = FilterEngine::compute_visible_set(&state, &filter);
        assert_eq!(visible.len(), state.node_count());
    }

    #[test]
    fn compute_visible_set_includes_ancestors_of_matches() {
        // Validates: Requirement 9.3 — ancestors of matches are visible
        use crate::node::{NodeType, TreeNodeData};
        use crate::state::TreeState;

        let mut state = TreeState::new();
        let local = state.root_categories[0];
        state.apply_children(
            local,
            vec![
                TreeNodeData::file("needle.rs"),
                TreeNodeData::file("other.txt"),
            ],
        );

        let mut filter = FilterState::new();
        filter.set_text("needle");
        let visible = FilterEngine::compute_visible_set(&state, &filter);

        // local (ancestor) must be visible
        assert!(visible.contains(&local));
        // needle.rs must be visible
        let needle_id = state
            .get_node(local)
            .unwrap()
            .children
            .iter()
            .copied()
            .find(|&id| state.get_node(id).unwrap().label == "needle.rs")
            .unwrap();
        assert!(visible.contains(&needle_id));
        // other.txt must NOT be visible
        let other_id = state
            .get_node(local)
            .unwrap()
            .children
            .iter()
            .copied()
            .find(|&id| state.get_node(id).unwrap().label == "other.txt")
            .unwrap();
        assert!(!visible.contains(&other_id));
    }
}
