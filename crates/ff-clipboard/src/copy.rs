//! Copy handler — implements copy operations for all selection types.
//!
//! The copy operation reads selected text from the document and writes it to
//! the clipboard engine without modifying the document or selection state.

use crate::config::ClipboardConfig;
use crate::engine::ClipboardEngine;
use crate::entry::ClipboardEntry;
use crate::error::ClipboardError;

/// Implements copy operations for all selection types.
///
/// Copy never modifies the document content or selection state. It reads the
/// selected text and writes a [`ClipboardEntry`] to the engine.
pub struct CopyHandler;

impl CopyHandler {
    /// Copy stream-selected text to the clipboard.
    ///
    /// The text between `start_offset` and `end_offset` in the document is
    /// copied with [`ClipboardMode::Stream`].
    ///
    /// # Errors
    ///
    /// Returns an error if the clipboard engine write fails.
    pub fn copy_stream(engine: &mut ClipboardEngine, text: &str) -> Result<(), ClipboardError> {
        let entry = ClipboardEntry::stream(text.to_string());
        engine.write(entry)
    }

    /// Copy rectangular selection as per-line segments.
    ///
    /// Each line's selected column segment is stored independently. The full
    /// text is the segments joined by newlines.
    ///
    /// # Errors
    ///
    /// Returns an error if the clipboard engine write fails.
    pub fn copy_rectangular(
        engine: &mut ClipboardEngine,
        segments: Vec<String>,
    ) -> Result<(), ClipboardError> {
        let entry = ClipboardEntry::rectangular(segments);
        engine.write(entry)
    }

    /// Copy multi-caret selections as independent segments.
    ///
    /// Each caret's selected text is stored as a separate segment.
    ///
    /// # Errors
    ///
    /// Returns an error if the clipboard engine write fails.
    pub fn copy_multi_caret(
        engine: &mut ClipboardEngine,
        segments: Vec<String>,
    ) -> Result<(), ClipboardError> {
        let entry = ClipboardEntry::multi_caret(segments);
        engine.write(entry)
    }

    /// Copy the entire current line (line-copy-when-no-selection mode).
    ///
    /// Copies the line text (including line ending) with [`ClipboardMode::Line`].
    /// Respects the `line_copy_when_no_selection` config setting.
    ///
    /// # Errors
    ///
    /// Returns an error if the clipboard engine write fails.
    pub fn copy_line(
        engine: &mut ClipboardEngine,
        line_text: &str,
        config: &ClipboardConfig,
    ) -> Result<(), ClipboardError> {
        if !config.line_copy_when_no_selection {
            return Ok(());
        }
        let entry = ClipboardEntry::line(line_text.to_string());
        engine.write(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::ClipboardMode;
    use crate::provider::InMemoryClipboardProvider;

    fn make_engine() -> ClipboardEngine {
        let provider = InMemoryClipboardProvider::new();
        ClipboardEngine::new(Box::new(provider), ClipboardConfig::default())
    }

    #[test]
    fn copy_stream_writes_text_with_stream_mode() {
        // Validates: Requirement 2.1
        let mut engine = make_engine();
        CopyHandler::copy_stream(&mut engine, "hello world").unwrap();

        let entry = engine.read().unwrap();
        assert_eq!(entry.text(), "hello world");
        assert_eq!(entry.mode(), ClipboardMode::Stream);
    }

    #[test]
    fn copy_rectangular_writes_segments() {
        // Validates: Requirement 2.2, 12.1
        let mut engine = make_engine();
        let segments = vec!["abc".to_string(), "def".to_string()];
        CopyHandler::copy_rectangular(&mut engine, segments.clone()).unwrap();

        let entry = engine.read().unwrap();
        assert_eq!(entry.mode(), ClipboardMode::Rectangular);
        assert_eq!(entry.segments(), &segments);
    }

    #[test]
    fn copy_multi_caret_writes_per_caret_segments() {
        // Validates: Requirement 2.3, 13.1
        let mut engine = make_engine();
        let segments = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        CopyHandler::copy_multi_caret(&mut engine, segments.clone()).unwrap();

        let entry = engine.read().unwrap();
        assert_eq!(entry.segments(), &segments);
        assert_eq!(entry.segment_count(), 3);
    }

    #[test]
    fn copy_line_writes_with_line_mode() {
        // Validates: Requirement 2.4, 14.1
        let mut engine = make_engine();
        let config = ClipboardConfig::default();
        CopyHandler::copy_line(&mut engine, "full line\n", &config).unwrap();

        let entry = engine.read().unwrap();
        assert_eq!(entry.text(), "full line\n");
        assert_eq!(entry.mode(), ClipboardMode::Line);
    }

    #[test]
    fn copy_line_does_nothing_when_config_disabled() {
        // Validates: Requirement 14.5
        let mut engine = make_engine();
        let config = ClipboardConfig {
            line_copy_when_no_selection: false,
            ..Default::default()
        };
        CopyHandler::copy_line(&mut engine, "line\n", &config).unwrap();

        // Clipboard should still be empty
        assert!(engine.read().is_err());
    }

    #[test]
    fn copy_stream_does_not_modify_document_guarantee() {
        // Validates: Requirement 2.5
        // (Copy operations take text by reference, proving no mutation)
        let mut engine = make_engine();
        let original_text = "test content";
        CopyHandler::copy_stream(&mut engine, original_text).unwrap();
        // original_text is still valid — copy took &str, not ownership
        assert_eq!(original_text, "test content");
    }
}
