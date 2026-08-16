//! `TabId` — unique, stable identifier for a tab within a workbench session.
//!
//! A TabId does not change when a tab is moved or reordered. It is generated
//! via UUID v4 for runtime tabs and can be reconstructed from a string for
//! session restore.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A unique, stable identifier for a tab within a workbench session.
///
/// Wraps a UUID v4 string representation. Does not change when a tab
/// is moved or reordered.
///
/// # Examples
///
/// ```
/// use ff_tabs::TabId;
///
/// let id = TabId::new();
/// println!("tab: {id}");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TabId(String);

impl TabId {
    /// Generate a new unique TabId using UUID v4.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create a TabId from a known string (used during session restore).
    ///
    /// # Panics
    ///
    /// Does not panic. If the string is not a valid UUID, it is stored as-is.
    /// Validation should be performed at the call site if needed.
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the underlying string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for TabId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TabId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for TabId {
    fn from(s: &str) -> Self {
        Self::from_string(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_generates_unique_ids() {
        let id1 = TabId::new();
        let id2 = TabId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn from_string_preserves_value() {
        let id = TabId::from_string("test-tab-id-123");
        assert_eq!(id.as_str(), "test-tab-id-123");
    }

    #[test]
    fn display_shows_inner_string() {
        let id = TabId::from_string("abc-def");
        assert_eq!(format!("{id}"), "abc-def");
    }

    #[test]
    fn clone_produces_equal_id() {
        let id = TabId::new();
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn hash_consistent_for_equal_ids() {
        use std::collections::HashSet;
        let id = TabId::from_string("same");
        let id2 = TabId::from_string("same");
        let mut set = HashSet::new();
        set.insert(id);
        assert!(set.contains(&id2));
    }

    #[test]
    fn ordering_is_lexicographic() {
        let a = TabId::from_string("aaa");
        let b = TabId::from_string("bbb");
        assert!(a < b);
    }

    #[test]
    fn default_generates_unique_id() {
        let id1 = TabId::default();
        let id2 = TabId::default();
        assert_ne!(id1, id2);
    }
}
