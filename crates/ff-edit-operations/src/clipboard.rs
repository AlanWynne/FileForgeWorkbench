//! Clipboard integration — edit-side cut/copy/paste semantics.
//!
//! This module defines the `ClipboardContent` type and the logic for
//! preparing content for the clipboard and interpreting pasted content.
//! System clipboard access is NOT handled here — only the edit-side
//! semantics of what to copy and how to paste.

/// Represents clipboard content with metadata about its source.
///
/// Contains the text plus metadata indicating how it was copied
/// (line copy, rectangular, multi-caret), which affects paste behaviour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardContent {
    /// The full text content (for stream/simple operations).
    pub text: String,
    /// Whether this was a "line copy" (entire line with no selection).
    /// When true, paste inserts as a new line above rather than inline.
    pub is_line_copy: bool,
    /// Whether this has rectangular selection metadata.
    /// When true, paste inserts as a column block.
    pub is_rectangular: bool,
    /// Individual segments (for multi-caret or rectangular copies).
    /// For rectangular: one segment per line in the rectangle.
    /// For multi-caret: one segment per caret.
    pub segments: Vec<String>,
}

impl ClipboardContent {
    /// Creates a simple text clipboard content (stream copy).
    pub fn text(content: String) -> Self {
        Self {
            text: content,
            is_line_copy: false,
            is_rectangular: false,
            segments: Vec::new(),
        }
    }

    /// Creates a line-copy clipboard content.
    pub fn line_copy(content: String) -> Self {
        Self {
            text: content,
            is_line_copy: true,
            is_rectangular: false,
            segments: Vec::new(),
        }
    }

    /// Creates a rectangular clipboard content with per-line segments.
    pub fn rectangular(segments: Vec<String>) -> Self {
        let text = segments.join("\n");
        Self {
            text,
            is_line_copy: false,
            is_rectangular: true,
            segments,
        }
    }

    /// Creates a multi-caret clipboard content with per-caret segments.
    pub fn multi_caret(segments: Vec<String>) -> Self {
        let text = segments.join("\n");
        Self {
            text,
            is_line_copy: false,
            is_rectangular: false,
            segments,
        }
    }

    /// Returns true if this content has multiple segments that can be
    /// distributed across carets.
    pub fn has_segments(&self) -> bool {
        !self.segments.is_empty()
    }

    /// Returns the number of segments.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Returns true if the content is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

impl Default for ClipboardContent {
    fn default() -> Self {
        Self::text(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_creates_simple_content() {
        let content = ClipboardContent::text("hello".to_string());
        assert_eq!(content.text, "hello");
        assert!(!content.is_line_copy);
        assert!(!content.is_rectangular);
        assert!(!content.has_segments());
    }

    #[test]
    fn line_copy_sets_flag() {
        let content = ClipboardContent::line_copy("line content\n".to_string());
        assert!(content.is_line_copy);
        assert!(!content.is_rectangular);
    }

    #[test]
    fn rectangular_creates_segmented_content() {
        let segments = vec!["abc".to_string(), "def".to_string(), "ghi".to_string()];
        let content = ClipboardContent::rectangular(segments);
        assert!(content.is_rectangular);
        assert_eq!(content.segment_count(), 3);
        assert_eq!(content.text, "abc\ndef\nghi");
    }

    #[test]
    fn multi_caret_creates_segmented_content() {
        let segments = vec!["first".to_string(), "second".to_string()];
        let content = ClipboardContent::multi_caret(segments);
        assert!(!content.is_rectangular);
        assert!(content.has_segments());
        assert_eq!(content.segment_count(), 2);
    }

    #[test]
    fn is_empty_reflects_text_content() {
        let empty = ClipboardContent::text(String::new());
        assert!(empty.is_empty());

        let non_empty = ClipboardContent::text("x".to_string());
        assert!(!non_empty.is_empty());
    }
}
