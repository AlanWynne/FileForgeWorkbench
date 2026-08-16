//! Navigation stack for Help Panel back/forward history.
//!
//! Each topic visit pushes onto the stack. Back/forward traverse without
//! removing entries. The stack is cleared when the Help Panel is closed.

use crate::topic_key::TopicKey;

/// A back/forward navigation history for the Help Panel.
///
/// Each topic visit pushes onto the stack. Back/forward traverse without
/// removing entries. The stack is cleared when the Help Panel is closed
/// and reopened (fresh session per Requirement 3.6).
///
/// # Invariants
///
/// - `position` is always a valid index into `history` when the stack is non-empty.
/// - `can_go_back()` is true iff `position > 0`.
/// - `can_go_forward()` is true iff `position < history.len() - 1`.
/// - After `push(key)`, forward history is discarded.
#[derive(Debug, Clone)]
pub struct NavigationStack {
    /// Ordered history of visited topics.
    history: Vec<TopicKey>,
    /// Current position index within history (0-based).
    position: usize,
}

impl NavigationStack {
    /// Create a new empty navigation stack.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            position: 0,
        }
    }

    /// Push a new topic onto the stack, discarding any forward history.
    ///
    /// After a push, `can_go_forward()` is always `false`.
    pub fn push(&mut self, key: TopicKey) {
        if !self.history.is_empty() {
            // Truncate forward history
            self.history.truncate(self.position + 1);
        }
        self.history.push(key);
        self.position = self.history.len() - 1;
    }

    /// Navigate back. Returns the previous `TopicKey`, or `None` if at start.
    pub fn back(&mut self) -> Option<&TopicKey> {
        if self.position > 0 {
            self.position -= 1;
            Some(&self.history[self.position])
        } else {
            None
        }
    }

    /// Navigate forward. Returns the next `TopicKey`, or `None` if at end.
    pub fn forward(&mut self) -> Option<&TopicKey> {
        if self.position + 1 < self.history.len() {
            self.position += 1;
            Some(&self.history[self.position])
        } else {
            None
        }
    }

    /// Returns the topic at the current pointer position.
    pub fn current(&self) -> Option<&TopicKey> {
        self.history.get(self.position)
    }

    /// Whether back navigation is possible.
    pub fn can_go_back(&self) -> bool {
        self.position > 0
    }

    /// Whether forward navigation is possible.
    pub fn can_go_forward(&self) -> bool {
        self.position + 1 < self.history.len()
    }

    /// Clear the entire stack (called when Help Panel is closed and reopened).
    pub fn clear(&mut self) {
        self.history.clear();
        self.position = 0;
    }

    /// Returns the total number of entries in the stack.
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
}

impl Default for NavigationStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 3.1 — Navigation stack records visited topics
    #[test]
    fn push_adds_topics_to_history() {
        let mut stack = NavigationStack::new();
        assert!(stack.is_empty());

        stack.push(TopicKey::command("FIND"));
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.current(), Some(&TopicKey::command("FIND")));

        stack.push(TopicKey::command("CHANGE"));
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.current(), Some(&TopicKey::command("CHANGE")));
    }

    // Validates: Requirement 3.2 — Back navigation
    #[test]
    fn back_returns_previous_topic() {
        let mut stack = NavigationStack::new();
        stack.push(TopicKey::index());
        stack.push(TopicKey::command("FIND"));
        stack.push(TopicKey::command("CHANGE"));

        let back = stack.back().cloned();
        assert_eq!(back, Some(TopicKey::command("FIND")));
        assert_eq!(stack.current(), Some(&TopicKey::command("FIND")));
    }

    // Validates: Requirement 3.2 — Forward navigation
    #[test]
    fn forward_returns_next_topic_after_back() {
        let mut stack = NavigationStack::new();
        stack.push(TopicKey::index());
        stack.push(TopicKey::command("FIND"));
        stack.push(TopicKey::command("CHANGE"));

        stack.back(); // now at FIND
        let fwd = stack.forward().cloned();
        assert_eq!(fwd, Some(TopicKey::command("CHANGE")));
    }

    // Validates: Requirement 3.2 — Back returns None at beginning
    #[test]
    fn back_returns_none_at_beginning() {
        let mut stack = NavigationStack::new();
        stack.push(TopicKey::index());
        assert_eq!(stack.back(), None);
    }

    // Validates: Requirement 3.2 — Forward returns None at end
    #[test]
    fn forward_returns_none_at_end() {
        let mut stack = NavigationStack::new();
        stack.push(TopicKey::index());
        assert_eq!(stack.forward(), None);
    }

    // Validates: Requirement 3.3 — Push after back truncates forward history
    #[test]
    fn push_after_back_truncates_forward_history() {
        let mut stack = NavigationStack::new();
        stack.push(TopicKey::index());
        stack.push(TopicKey::command("FIND"));
        stack.push(TopicKey::command("CHANGE"));

        stack.back(); // at FIND
        stack.push(TopicKey::command("SAVE")); // should truncate CHANGE

        assert!(!stack.can_go_forward());
        assert_eq!(stack.len(), 3); // index, FIND, SAVE
        assert_eq!(stack.current(), Some(&TopicKey::command("SAVE")));
    }

    // Validates: Requirement 3.2 — can_go_back and can_go_forward
    #[test]
    fn can_go_back_and_forward_reflect_position() {
        let mut stack = NavigationStack::new();
        assert!(!stack.can_go_back());
        assert!(!stack.can_go_forward());

        stack.push(TopicKey::index());
        assert!(!stack.can_go_back());
        assert!(!stack.can_go_forward());

        stack.push(TopicKey::command("FIND"));
        assert!(stack.can_go_back());
        assert!(!stack.can_go_forward());

        stack.back();
        assert!(!stack.can_go_back());
        assert!(stack.can_go_forward());
    }

    // Validates: Requirement 3.6 — Clear on reopen
    #[test]
    fn clear_resets_stack_completely() {
        let mut stack = NavigationStack::new();
        stack.push(TopicKey::index());
        stack.push(TopicKey::command("FIND"));

        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.current(), None);
        assert!(!stack.can_go_back());
        assert!(!stack.can_go_forward());
    }

    // Validates: Requirement 3.2 — Back then forward returns to same topic
    #[test]
    fn back_then_forward_returns_to_original() {
        let mut stack = NavigationStack::new();
        stack.push(TopicKey::index());
        stack.push(TopicKey::command("FIND"));

        let before_back = stack.current().cloned();
        stack.back();
        stack.forward();
        assert_eq!(stack.current().cloned(), before_back);
    }
}
