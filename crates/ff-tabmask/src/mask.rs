//! Insert mask template management.
//!
//! Provides the [`MaskLine`] type — the content of an insert mask template that
//! pre-fills newly inserted blank lines with boilerplate content.

use std::fmt;

/// The content of an insert mask template.
///
/// Represents a fixed-width template string applied to newly inserted blank lines.
/// Characters at each column position define pre-filled content; spaces are "empty" positions.
///
/// Addresses: Requirements 6, 7, 8, 9, 10, 16
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskLine {
    /// The mask template content. Each character maps to its column position.
    content: String,
}

impl MaskLine {
    /// Creates a new MaskLine from a string value.
    ///
    /// The content is stored verbatim without transformation.
    ///
    /// Addresses: Requirement 10, criterion 10.4
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }

    /// Creates an empty MaskLine (no mask content).
    pub fn empty() -> Self {
        Self {
            content: String::new(),
        }
    }

    /// Returns true if the mask has no content.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Returns the mask content as a string slice.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the length of the mask in characters.
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Applies this mask to create a new line of the given `line_width`.
    ///
    /// If the mask is shorter than `line_width`, pads with spaces.
    /// If the mask is longer than `line_width`, truncates at `line_width`.
    ///
    /// Addresses: Requirement 9, criteria 9.5, 9.6
    pub fn apply_to_width(&self, line_width: usize) -> String {
        if line_width == 0 {
            return String::new();
        }
        if self.content.len() >= line_width {
            self.content[..line_width].to_string()
        } else {
            let mut result = self.content.clone();
            result.extend(std::iter::repeat_n(' ', line_width - self.content.len()));
            result
        }
    }

    /// Updates the mask content (from in-place MASK_Line editing).
    ///
    /// Addresses: Requirement 6, criterion 6.4
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }
}

impl fmt::Display for MaskLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_mask_with_verbatim_content() {
        // Validates: Requirement 10.4
        let mask = MaskLine::new("      *");
        assert_eq!(mask.content(), "      *");
        assert_eq!(mask.len(), 7);
        assert!(!mask.is_empty());
    }

    #[test]
    fn empty_creates_empty_mask() {
        let mask = MaskLine::empty();
        assert!(mask.is_empty());
        assert_eq!(mask.len(), 0);
        assert_eq!(mask.content(), "");
    }

    #[test]
    fn apply_to_width_pads_shorter_mask() {
        // Validates: Requirement 9.5
        let mask = MaskLine::new("ABC");
        let result = mask.apply_to_width(8);
        assert_eq!(result, "ABC     ");
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn apply_to_width_truncates_longer_mask() {
        // Validates: Requirement 9.6
        let mask = MaskLine::new("ABCDEFGHIJ");
        let result = mask.apply_to_width(5);
        assert_eq!(result, "ABCDE");
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn apply_to_width_exact_length_unchanged() {
        let mask = MaskLine::new("ABCDE");
        let result = mask.apply_to_width(5);
        assert_eq!(result, "ABCDE");
    }

    #[test]
    fn apply_to_width_zero_returns_empty() {
        let mask = MaskLine::new("ABC");
        let result = mask.apply_to_width(0);
        assert_eq!(result, "");
    }

    #[test]
    fn set_content_updates_mask() {
        // Validates: Requirement 6.4
        let mut mask = MaskLine::new("old");
        mask.set_content("new content");
        assert_eq!(mask.content(), "new content");
    }

    #[test]
    fn display_shows_content() {
        let mask = MaskLine::new("      *");
        assert_eq!(format!("{mask}"), "      *");
    }

    #[test]
    fn preserves_special_characters() {
        // Validates: Requirement 10.4 — tab characters and special characters preserved as-is
        let mask = MaskLine::new("col1\tcol2\t");
        assert_eq!(mask.content(), "col1\tcol2\t");
    }
}
