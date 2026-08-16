//! Bookmark command registration.
//!
//! Registers bookmark operations (toggle, next, previous, clear_all) as
//! commands in the command-framework for keyboard/menu access.

/// Command ID for bookmark toggle.
pub const CMD_BOOKMARK_TOGGLE: &str = "decorations.bookmark.toggle";
/// Command ID for next bookmark navigation.
pub const CMD_BOOKMARK_NEXT: &str = "decorations.bookmark.next";
/// Command ID for previous bookmark navigation.
pub const CMD_BOOKMARK_PREVIOUS: &str = "decorations.bookmark.previous";
/// Command ID for clearing all bookmarks.
pub const CMD_BOOKMARK_CLEAR_ALL: &str = "decorations.bookmark.clear_all";
