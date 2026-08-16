//! Status bar segment types and alignment.
//!
//! Each segment represents a display region in the status bar with a unique ID,
//! alignment grouping, and rendering priority.

use crate::error::MenuError;

/// Alignment grouping for status bar segments.
///
/// Segments are laid out in groups: Left segments first, then Center, then Right.
/// Within each group, segments are ordered by priority (lower = renders first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SegmentAlignment {
    /// Left-aligned segments (editor mode, insert/overstrike, encoding).
    Left,
    /// Center-aligned segments (reserved for extension).
    Center,
    /// Right-aligned segments (line/col, modified indicator, total lines).
    Right,
}

/// A single segment within the status bar.
///
/// Each segment has a unique ID, alignment, priority, and optional minimum width.
#[derive(Debug, Clone)]
pub struct StatusSegment {
    /// Unique identifier (1–64 ASCII alphanumeric/underscore chars).
    pub id: String,
    /// Alignment group within the status bar.
    pub alignment: SegmentAlignment,
    /// Ordering priority within the alignment group (lower = renders first).
    pub priority: u32,
    /// Minimum width in logical pixels (0 = auto-size to content).
    pub min_width: f32,
    /// Whether this segment is currently visible.
    pub visible: bool,
    /// Contributing plugin name (None for built-in segments).
    pub contributed_by: Option<String>,
    /// The current display content of the segment.
    pub content: String,
}

impl StatusSegment {
    /// Creates a new status segment with the given ID, alignment, and priority.
    ///
    /// # Errors
    ///
    /// Returns `MenuError::InvalidSegmentId` if the ID is not 1–64 ASCII
    /// alphanumeric or underscore characters.
    pub fn new(
        id: impl Into<String>,
        alignment: SegmentAlignment,
        priority: u32,
    ) -> Result<Self, MenuError> {
        let id = id.into();
        validate_segment_id(&id)?;
        Ok(Self {
            id,
            alignment,
            priority,
            min_width: 0.0,
            visible: true,
            contributed_by: None,
            content: String::new(),
        })
    }

    /// Sets the minimum width for this segment.
    pub fn with_min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    /// Sets the contributing plugin name.
    pub fn with_plugin(mut self, plugin: impl Into<String>) -> Self {
        self.contributed_by = Some(plugin.into());
        self
    }

    /// Updates the display content of this segment.
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }
}

/// Trait for providing content to a status bar segment.
///
/// Implemented by built-in providers and plugins. Providers are registered
/// with the `StatusBar` and are queried each frame for rendering.
pub trait StatusSegmentProvider: Send + Sync {
    /// Returns the unique segment identifier.
    fn segment_id(&self) -> &str;

    /// Returns the current display text for this segment.
    fn content(&self) -> &str;

    /// Returns the alignment group for this segment.
    fn alignment(&self) -> SegmentAlignment;

    /// Returns the ordering priority (lower = renders first within group).
    fn priority(&self) -> u32;

    /// Returns whether the segment currently has content to display.
    /// Segments returning false may be collapsed to save space.
    fn has_content(&self) -> bool {
        true
    }
}

/// Validates that a segment ID is 1–64 ASCII alphanumeric or underscore characters.
pub fn validate_segment_id(id: &str) -> Result<(), MenuError> {
    if id.is_empty() || id.len() > 64 {
        return Err(MenuError::InvalidSegmentId { id: id.to_string() });
    }
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(MenuError::InvalidSegmentId { id: id.to_string() });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_segment_id_accepted() {
        assert!(validate_segment_id("editor_mode").is_ok());
        assert!(validate_segment_id("a").is_ok());
        assert!(validate_segment_id("segment_123").is_ok());
        let long_id = "a".repeat(64);
        assert!(validate_segment_id(&long_id).is_ok());
    }

    #[test]
    fn empty_segment_id_rejected() {
        assert!(validate_segment_id("").is_err());
    }

    #[test]
    fn too_long_segment_id_rejected() {
        let long_id = "a".repeat(65);
        assert!(validate_segment_id(&long_id).is_err());
    }

    #[test]
    fn invalid_chars_in_segment_id_rejected() {
        assert!(validate_segment_id("has space").is_err());
        assert!(validate_segment_id("has-dash").is_err());
        assert!(validate_segment_id("has.dot").is_err());
        assert!(validate_segment_id("has/slash").is_err());
    }

    #[test]
    fn status_segment_new_validates_id() {
        let valid = StatusSegment::new("editor_mode", SegmentAlignment::Left, 0);
        assert!(valid.is_ok());

        let invalid = StatusSegment::new("", SegmentAlignment::Left, 0);
        assert!(invalid.is_err());
    }

    #[test]
    fn status_segment_set_content_updates_display() {
        let mut segment = StatusSegment::new("mode", SegmentAlignment::Left, 0).unwrap();
        segment.set_content("Edit");
        assert_eq!(segment.content, "Edit");
    }
}
