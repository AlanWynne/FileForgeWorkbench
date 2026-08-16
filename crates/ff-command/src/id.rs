//! `CommandId` newtype with validation.
//!
//! A `CommandId` is a non-empty UTF-8 string containing only lowercase ASCII
//! letters, digits, dots, and underscores. Dot serves as namespace separator.

use std::fmt;

/// A validated command identifier.
///
/// Non-empty UTF-8 string containing only lowercase ASCII letters `[a-z]`,
/// digits `[0-9]`, dots `.`, and underscores `_`. Dot serves as namespace
/// separator (e.g., `"file.save"`, `"edit.undo"`).
///
/// # Examples
///
/// ```
/// use ff_command::CommandId;
///
/// let id = CommandId::new("file.save").unwrap();
/// assert_eq!(id.as_str(), "file.save");
/// assert_eq!(id.category(), "file");
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CommandId(String);

impl CommandId {
    /// Attempts to create a `CommandId` from a string, validating format.
    ///
    /// Returns `Some` if the string is valid, `None` otherwise.
    ///
    /// # Validation Rules
    ///
    /// - Must not be empty
    /// - Must contain only lowercase ASCII letters, digits, dots, and underscores
    /// - Must not start or end with a dot
    /// - Must not contain consecutive dots
    pub fn new(id: impl Into<String>) -> Option<Self> {
        let id = id.into();
        if Self::is_valid(&id) {
            Some(Self(id))
        } else {
            None
        }
    }

    /// Returns the category prefix (everything before the first dot).
    ///
    /// For `"file.save"` returns `"file"`.
    /// For `"plugin.git.commit"` returns `"plugin"`.
    /// For `"standalone"` (no dot) returns the full ID.
    pub fn category(&self) -> &str {
        self.0.split('.').next().unwrap_or(&self.0)
    }

    /// Returns the full ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if this ID starts with the given prefix followed by a dot.
    pub fn has_prefix(&self, prefix: &str) -> bool {
        if prefix.is_empty() {
            return false;
        }
        self.0.starts_with(prefix) && self.0[prefix.len()..].starts_with('.')
    }

    /// Validates a string as a command ID without constructing one.
    pub fn is_valid(id: &str) -> bool {
        if id.is_empty() {
            return false;
        }
        if id.starts_with('.') || id.ends_with('.') {
            return false;
        }
        if id.contains("..") {
            return false;
        }
        id.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_')
    }
}

impl fmt::Display for CommandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for CommandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CommandId(\"{}\")", self.0)
    }
}

impl AsRef<str> for CommandId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1.1
    #[test]
    fn valid_ids_are_accepted() {
        assert!(CommandId::new("file.save").is_some());
        assert!(CommandId::new("edit.undo").is_some());
        assert!(CommandId::new("view.zoom_in").is_some());
        assert!(CommandId::new("plugin.git.commit").is_some());
        assert!(CommandId::new("a").is_some());
        assert!(CommandId::new("x1").is_some());
        assert!(CommandId::new("test_command").is_some());
    }

    // Validates: Requirement 1.1
    #[test]
    fn empty_string_is_rejected() {
        assert!(CommandId::new("").is_none());
    }

    // Validates: Requirement 1.1
    #[test]
    fn uppercase_characters_are_rejected() {
        assert!(CommandId::new("File.Save").is_none());
        assert!(CommandId::new("EDIT.UNDO").is_none());
        assert!(CommandId::new("editA").is_none());
    }

    // Validates: Requirement 1.1
    #[test]
    fn spaces_are_rejected() {
        assert!(CommandId::new("file save").is_none());
        assert!(CommandId::new(" file.save").is_none());
    }

    // Validates: Requirement 1.1
    #[test]
    fn leading_dot_is_rejected() {
        assert!(CommandId::new(".file.save").is_none());
    }

    // Validates: Requirement 1.1
    #[test]
    fn trailing_dot_is_rejected() {
        assert!(CommandId::new("file.save.").is_none());
    }

    // Validates: Requirement 1.1
    #[test]
    fn consecutive_dots_are_rejected() {
        assert!(CommandId::new("file..save").is_none());
    }

    // Validates: Requirement 1.1
    #[test]
    fn special_characters_are_rejected() {
        assert!(CommandId::new("file-save").is_none());
        assert!(CommandId::new("file@save").is_none());
        assert!(CommandId::new("file#save").is_none());
    }

    #[test]
    fn category_returns_first_segment() {
        let id = CommandId::new("file.save").unwrap();
        assert_eq!(id.category(), "file");
    }

    #[test]
    fn category_returns_full_id_when_no_dot() {
        let id = CommandId::new("standalone").unwrap();
        assert_eq!(id.category(), "standalone");
    }

    #[test]
    fn has_prefix_matches_category() {
        let id = CommandId::new("file.save").unwrap();
        assert!(id.has_prefix("file"));
        assert!(!id.has_prefix("fil"));
        assert!(!id.has_prefix("edit"));
        assert!(!id.has_prefix(""));
    }

    #[test]
    fn display_shows_id_string() {
        let id = CommandId::new("edit.undo").unwrap();
        assert_eq!(format!("{}", id), "edit.undo");
    }

    #[test]
    fn debug_shows_wrapped_id() {
        let id = CommandId::new("edit.undo").unwrap();
        assert_eq!(format!("{:?}", id), "CommandId(\"edit.undo\")");
    }

    #[test]
    fn clone_and_eq_work() {
        let id1 = CommandId::new("file.save").unwrap();
        let id2 = id1.clone();
        assert_eq!(id1, id2);
    }

    #[test]
    fn different_ids_are_not_equal() {
        let id1 = CommandId::new("file.save").unwrap();
        let id2 = CommandId::new("file.open").unwrap();
        assert_ne!(id1, id2);
    }
}
