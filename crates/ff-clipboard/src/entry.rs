//! Clipboard entry types — structured clipboard content with mode and segments.
//!
//! A [`ClipboardEntry`] represents clipboard content along with metadata about
//! how it was acquired (stream, line, or rectangular selection) and optional
//! per-line segments for rectangular or multi-caret content.

use std::time::Instant;

/// Indicates how clipboard content was acquired, affecting paste behaviour.
///
/// The mode determines whether paste inserts inline (Stream), as new lines
/// above the caret (Line), or as a column block (Rectangular).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ClipboardMode {
    /// Normal character-stream selection copy. Paste inserts inline at caret.
    #[default]
    Stream,
    /// Full-line copy (no selection active). Paste inserts as new line(s) above caret line.
    Line,
    /// Rectangular (column) selection copy. Paste inserts as column block.
    Rectangular,
}

/// A structured clipboard content unit with mode and optional per-segment storage.
///
/// Stores the full text (written to the system clipboard) along with the
/// [`ClipboardMode`] and independent segments for rectangular/multi-caret content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardEntry {
    /// The full text content written to/read from the system clipboard.
    text: String,
    /// How the content was acquired — determines paste semantics.
    mode: ClipboardMode,
    /// Independent line segments for Rectangular or Multi-Caret modes.
    /// Empty for Stream/Line modes (text is used directly).
    segments: Vec<String>,
    /// Timestamp of when this entry was created (for history ordering).
    created_at: Instant,
}

impl ClipboardEntry {
    /// Create a stream-mode clipboard entry from the given text.
    ///
    /// Stream mode is the default for normal character selections.
    /// Paste inserts the text inline at the caret position.
    pub fn stream(text: String) -> Self {
        Self {
            text,
            mode: ClipboardMode::Stream,
            segments: Vec::new(),
            created_at: Instant::now(),
        }
    }

    /// Create a line-mode clipboard entry from the given text.
    ///
    /// Line mode is used when copying with no active selection.
    /// Paste inserts the content as new line(s) above the caret line.
    pub fn line(text: String) -> Self {
        Self {
            text,
            mode: ClipboardMode::Line,
            segments: Vec::new(),
            created_at: Instant::now(),
        }
    }

    /// Create a rectangular-mode clipboard entry from per-line segments.
    ///
    /// The full text is constructed by joining segments with newlines.
    /// Paste inserts each segment as a column block at the caret column.
    pub fn rectangular(segments: Vec<String>) -> Self {
        let text = segments.join("\n");
        Self {
            text,
            mode: ClipboardMode::Rectangular,
            segments,
            created_at: Instant::now(),
        }
    }

    /// Create a multi-caret clipboard entry from per-caret segments.
    ///
    /// Stored as Stream mode for the system clipboard text, but retains
    /// per-caret segments for distribution during paste.
    pub fn multi_caret(segments: Vec<String>) -> Self {
        let text = segments.join("\n");
        Self {
            text,
            mode: ClipboardMode::Stream,
            segments,
            created_at: Instant::now(),
        }
    }

    /// Create an entry from raw text with a specified mode (used when reading
    /// back from the clipboard engine with cached metadata).
    pub fn from_text(text: String, mode: ClipboardMode) -> Self {
        Self {
            text,
            mode,
            segments: Vec::new(),
            created_at: Instant::now(),
        }
    }

    /// Create an entry with explicit text, mode, and segments.
    pub fn with_segments(text: String, mode: ClipboardMode, segments: Vec<String>) -> Self {
        Self {
            text,
            mode,
            segments,
            created_at: Instant::now(),
        }
    }

    /// The full text content of this clipboard entry.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The clipboard mode determining paste behaviour.
    pub fn mode(&self) -> ClipboardMode {
        self.mode
    }

    /// Per-line or per-caret segments, if any.
    ///
    /// Empty for Stream/Line modes; populated for Rectangular and Multi-Caret.
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// Number of segments stored (0 for Stream/Line, N for Rectangular/Multi-Caret).
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Whether this entry has any text content.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// When this entry was created.
    pub fn created_at(&self) -> Instant {
        self.created_at
    }
}

/// Metadata stored alongside a clipboard write to detect internal vs external changes.
///
/// When the system clipboard text matches the last-written text from this instance,
/// we know the clipboard was not modified externally and can use the stored mode/segments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardMetadata {
    /// The clipboard mode at the time of writing.
    pub mode: ClipboardMode,
    /// Per-line segments stored with the write (if any).
    pub segments: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_entry_has_correct_mode_and_empty_segments() {
        // Validates: Requirement 1.4
        let entry = ClipboardEntry::stream("hello world".to_string());
        assert_eq!(entry.mode(), ClipboardMode::Stream);
        assert_eq!(entry.text(), "hello world");
        assert!(entry.segments().is_empty());
        assert_eq!(entry.segment_count(), 0);
    }

    #[test]
    fn line_entry_has_correct_mode() {
        // Validates: Requirement 1.4
        let entry = ClipboardEntry::line("full line\n".to_string());
        assert_eq!(entry.mode(), ClipboardMode::Line);
        assert_eq!(entry.text(), "full line\n");
        assert!(entry.segments().is_empty());
    }

    #[test]
    fn rectangular_entry_stores_segments_and_joins_text() {
        // Validates: Requirement 1.4, 12.1
        let segments = vec!["abc".to_string(), "def".to_string(), "ghi".to_string()];
        let entry = ClipboardEntry::rectangular(segments.clone());
        assert_eq!(entry.mode(), ClipboardMode::Rectangular);
        assert_eq!(entry.text(), "abc\ndef\nghi");
        assert_eq!(entry.segments(), &segments);
        assert_eq!(entry.segment_count(), 3);
    }

    #[test]
    fn multi_caret_entry_stores_segments_with_stream_mode() {
        // Validates: Requirement 13.1
        let segments = vec!["one".to_string(), "two".to_string()];
        let entry = ClipboardEntry::multi_caret(segments.clone());
        assert_eq!(entry.mode(), ClipboardMode::Stream);
        assert_eq!(entry.text(), "one\ntwo");
        assert_eq!(entry.segments(), &segments);
        assert_eq!(entry.segment_count(), 2);
    }

    #[test]
    fn from_text_creates_entry_with_given_mode() {
        // Validates: Requirement 1.5
        let entry = ClipboardEntry::from_text("some text".to_string(), ClipboardMode::Line);
        assert_eq!(entry.mode(), ClipboardMode::Line);
        assert_eq!(entry.text(), "some text");
        assert!(entry.segments().is_empty());
    }

    #[test]
    fn with_segments_preserves_all_fields() {
        // Validates: Requirement 1.4
        let segments = vec!["a".to_string(), "b".to_string()];
        let entry = ClipboardEntry::with_segments(
            "a\nb".to_string(),
            ClipboardMode::Rectangular,
            segments.clone(),
        );
        assert_eq!(entry.mode(), ClipboardMode::Rectangular);
        assert_eq!(entry.text(), "a\nb");
        assert_eq!(entry.segments(), &segments);
    }

    #[test]
    fn is_empty_returns_true_for_empty_text() {
        let entry = ClipboardEntry::stream(String::new());
        assert!(entry.is_empty());
    }

    #[test]
    fn is_empty_returns_false_for_non_empty_text() {
        let entry = ClipboardEntry::stream("x".to_string());
        assert!(!entry.is_empty());
    }

    #[test]
    fn default_clipboard_mode_is_stream() {
        // Validates: Requirement 1.5
        assert_eq!(ClipboardMode::default(), ClipboardMode::Stream);
    }
}
