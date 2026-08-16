//! Cut handler — implements cut operations (copy to clipboard then delete from document).
//!
//! Cut operations produce a single undo record for the combined copy+delete action.
//! If the clipboard write fails, the document is NOT modified (failure safety).

use crate::config::ClipboardConfig;
use crate::engine::ClipboardEngine;
use crate::entry::{ClipboardEntry, ClipboardMode};
use crate::error::ClipboardError;

/// Result of a cut operation including information about what was cut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutResult {
    /// The text that was cut from the document.
    pub cut_text: String,
    /// The clipboard mode used for the cut content.
    pub mode: ClipboardMode,
}

/// Implements cut operations — copies to clipboard then deletes from document.
///
/// Cut operations follow failure-safety: if the clipboard write fails, the
/// document is not modified. The operation records a single undo record.
pub struct CutHandler;

impl CutHandler {
    /// Cut stream-selected text: write to clipboard, return cut result for caller to delete.
    ///
    /// The caller is responsible for deleting the text from the document and
    /// recording the undo record — this handler ensures clipboard write succeeds
    /// before deletion proceeds.
    ///
    /// # Errors
    ///
    /// Returns an error if the clipboard write fails (document not modified).
    pub fn cut_stream(
        engine: &mut ClipboardEngine,
        text: &str,
    ) -> Result<CutResult, ClipboardError> {
        let entry = ClipboardEntry::stream(text.to_string());
        engine.write(entry)?;
        Ok(CutResult {
            cut_text: text.to_string(),
            mode: ClipboardMode::Stream,
        })
    }

    /// Cut rectangular selection: write segments to clipboard, return cut result.
    ///
    /// # Errors
    ///
    /// Returns an error if the clipboard write fails (document not modified).
    pub fn cut_rectangular(
        engine: &mut ClipboardEngine,
        segments: Vec<String>,
    ) -> Result<CutResult, ClipboardError> {
        let text = segments.join("\n");
        let entry = ClipboardEntry::rectangular(segments);
        engine.write(entry)?;
        Ok(CutResult {
            cut_text: text,
            mode: ClipboardMode::Rectangular,
        })
    }

    /// Cut multi-caret selections: write per-caret segments to clipboard.
    ///
    /// # Errors
    ///
    /// Returns an error if the clipboard write fails (document not modified).
    pub fn cut_multi_caret(
        engine: &mut ClipboardEngine,
        segments: Vec<String>,
    ) -> Result<CutResult, ClipboardError> {
        let text = segments.join("\n");
        let entry = ClipboardEntry::multi_caret(segments);
        engine.write(entry)?;
        Ok(CutResult {
            cut_text: text,
            mode: ClipboardMode::Stream,
        })
    }

    /// Cut entire current line (line-cut-when-no-selection mode).
    ///
    /// Respects the `line_copy_when_no_selection` config setting.
    ///
    /// # Errors
    ///
    /// Returns an error if the clipboard write fails or if config disables line-cut.
    pub fn cut_line(
        engine: &mut ClipboardEngine,
        line_text: &str,
        config: &ClipboardConfig,
    ) -> Result<CutResult, ClipboardError> {
        if !config.line_copy_when_no_selection {
            return Ok(CutResult {
                cut_text: String::new(),
                mode: ClipboardMode::Line,
            });
        }
        let entry = ClipboardEntry::line(line_text.to_string());
        engine.write(entry)?;
        Ok(CutResult {
            cut_text: line_text.to_string(),
            mode: ClipboardMode::Line,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::InMemoryClipboardProvider;

    fn make_engine() -> ClipboardEngine {
        let provider = InMemoryClipboardProvider::new();
        ClipboardEngine::new(Box::new(provider), ClipboardConfig::default())
    }

    fn make_engine_with_provider() -> (ClipboardEngine, InMemoryClipboardProvider) {
        let provider = InMemoryClipboardProvider::new();
        let provider_clone = provider.clone();
        let engine = ClipboardEngine::new(Box::new(provider), ClipboardConfig::default());
        (engine, provider_clone)
    }

    #[test]
    fn cut_stream_writes_to_clipboard_and_returns_cut_text() {
        // Validates: Requirement 3.1
        let mut engine = make_engine();
        let result = CutHandler::cut_stream(&mut engine, "selected text").unwrap();
        assert_eq!(result.cut_text, "selected text");
        assert_eq!(result.mode, ClipboardMode::Stream);

        let clipboard = engine.read().unwrap();
        assert_eq!(clipboard.text(), "selected text");
    }

    #[test]
    fn cut_rectangular_writes_segments_to_clipboard() {
        // Validates: Requirement 3.2
        let mut engine = make_engine();
        let segments = vec!["abc".to_string(), "def".to_string()];
        let result = CutHandler::cut_rectangular(&mut engine, segments).unwrap();
        assert_eq!(result.mode, ClipboardMode::Rectangular);

        let clipboard = engine.read().unwrap();
        assert_eq!(clipboard.mode(), ClipboardMode::Rectangular);
    }

    #[test]
    fn cut_multi_caret_writes_segments() {
        // Validates: Requirement 3.3
        let mut engine = make_engine();
        let segments = vec!["one".to_string(), "two".to_string()];
        let result = CutHandler::cut_multi_caret(&mut engine, segments).unwrap();
        assert_eq!(result.cut_text, "one\ntwo");
    }

    #[test]
    fn cut_line_writes_with_line_mode() {
        // Validates: Requirement 3.4
        let mut engine = make_engine();
        let config = ClipboardConfig::default();
        let result = CutHandler::cut_line(&mut engine, "entire line\n", &config).unwrap();
        assert_eq!(result.cut_text, "entire line\n");
        assert_eq!(result.mode, ClipboardMode::Line);
    }

    #[test]
    fn cut_line_does_nothing_when_config_disabled() {
        // Validates: Requirement 14.5
        let mut engine = make_engine();
        let config = ClipboardConfig {
            line_copy_when_no_selection: false,
            ..Default::default()
        };
        let result = CutHandler::cut_line(&mut engine, "line\n", &config).unwrap();
        assert!(result.cut_text.is_empty());
    }

    #[test]
    fn cut_stream_failure_safety_does_not_modify_clipboard_on_unavailable() {
        // Validates: Requirement 3.5 (failure safety — clipboard write failure)
        let (mut engine, provider) = make_engine_with_provider();
        provider.set_available(false);

        let result = CutHandler::cut_stream(&mut engine, "text");
        assert!(result.is_err());
        // Document would not be modified because we return error before deletion
    }
}
