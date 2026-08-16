//! Paste handler — implements paste operations with mode-aware insertion logic.
//!
//! Paste behaviour varies based on the [`ClipboardMode`] of the entry being pasted:
//! - Stream: inserts inline at caret, replacing any active selection
//! - Line: inserts as new lines above the caret line
//! - Rectangular: inserts as column block at caret position
//! Multi-caret distribution is handled separately based on segment/caret count matching.

use crate::config::ClipboardConfig;
use crate::entry::{ClipboardEntry, ClipboardMode};
use crate::error::ClipboardError;
use crate::splitter::LineSplitter;

/// Result of a paste operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteResult {
    /// The lines that should be inserted into the document.
    /// The caller is responsible for performing the actual insertion.
    pub lines_to_insert: Vec<String>,
    /// Number of logical lines inserted (for status display).
    pub lines_inserted: usize,
    /// The paste mode that was applied.
    pub mode: ClipboardMode,
}

/// Implements paste operations with mode-aware insertion logic.
///
/// The paste handler determines what content to insert based on the clipboard
/// entry's mode and returns a [`PasteResult`] that the caller uses to perform
/// the actual document insertion and undo recording.
pub struct PasteHandler;

impl PasteHandler {
    /// Prepare a stream-mode paste: split clipboard text into lines for insertion.
    ///
    /// The text is split on line endings (LF, CRLF, CR). Trailing terminators
    /// do not produce an empty line. Whitespace is preserved exactly.
    pub fn paste_stream(entry: &ClipboardEntry) -> Result<PasteResult, ClipboardError> {
        if entry.is_empty() {
            return Err(ClipboardError::Empty);
        }
        let split = LineSplitter::split(entry.text());
        Ok(PasteResult {
            lines_inserted: split.lines.len(),
            lines_to_insert: split.lines,
            mode: ClipboardMode::Stream,
        })
    }

    /// Prepare a line-mode paste: split clipboard text into lines for insertion
    /// above the caret line.
    ///
    /// Line-mode paste inserts as new lines without splitting the current line.
    pub fn paste_line(entry: &ClipboardEntry) -> Result<PasteResult, ClipboardError> {
        if entry.is_empty() {
            return Err(ClipboardError::Empty);
        }
        let split = LineSplitter::split(entry.text());
        Ok(PasteResult {
            lines_inserted: split.lines.len(),
            lines_to_insert: split.lines,
            mode: ClipboardMode::Line,
        })
    }

    /// Prepare a rectangular-mode paste: use stored segments for column block insertion.
    ///
    /// Each segment is inserted on successive lines at the caret column.
    /// If `rectangular_paste_adds_lines` is false in config, excess segments
    /// beyond the document end are discarded.
    pub fn paste_rectangular(
        entry: &ClipboardEntry,
        config: &ClipboardConfig,
    ) -> Result<PasteResult, ClipboardError> {
        let segments = if entry.segments().is_empty() {
            // Fallback: split the text into lines if no segments stored
            LineSplitter::split(entry.text()).lines
        } else {
            entry.segments().to_vec()
        };

        if segments.is_empty() {
            return Err(ClipboardError::Empty);
        }

        let _ = config; // used by caller to decide whether to add lines
        Ok(PasteResult {
            lines_inserted: segments.len(),
            lines_to_insert: segments,
            mode: ClipboardMode::Rectangular,
        })
    }

    /// Prepare a multi-caret paste with matched distribution.
    ///
    /// When the clipboard contains exactly N segments and there are N carets,
    /// segment[i] is distributed to caret[i].
    pub fn paste_multi_caret_matched(
        entry: &ClipboardEntry,
        caret_count: usize,
    ) -> Result<Vec<PasteResult>, ClipboardError> {
        let segments = entry.segments();
        if segments.len() != caret_count {
            return Err(ClipboardError::Empty); // mismatch — caller should use broadcast
        }

        let results = segments
            .iter()
            .map(|seg| {
                let split = LineSplitter::split(seg);
                PasteResult {
                    lines_inserted: split.lines.len(),
                    lines_to_insert: split.lines,
                    mode: ClipboardMode::Stream,
                }
            })
            .collect();

        Ok(results)
    }

    /// Prepare a multi-caret paste with broadcast (full content at each caret).
    ///
    /// When segment count doesn't match caret count, the full clipboard content
    /// is pasted at each caret position.
    pub fn paste_multi_caret_broadcast(
        entry: &ClipboardEntry,
        caret_count: usize,
    ) -> Result<Vec<PasteResult>, ClipboardError> {
        if entry.is_empty() {
            return Err(ClipboardError::Empty);
        }

        let split = LineSplitter::split(entry.text());
        let base_result = PasteResult {
            lines_inserted: split.lines.len(),
            lines_to_insert: split.lines,
            mode: ClipboardMode::Stream,
        };

        Ok(vec![base_result; caret_count])
    }

    /// Determine the appropriate paste mode based on the clipboard entry mode
    /// and current context (caret count, segments).
    pub fn resolve_paste_mode(entry: &ClipboardEntry, caret_count: usize) -> PasteMode {
        match entry.mode() {
            ClipboardMode::Line => PasteMode::Line,
            ClipboardMode::Rectangular => PasteMode::Rectangular,
            ClipboardMode::Stream => {
                if caret_count > 1 && entry.segment_count() == caret_count {
                    PasteMode::MultiCaretMatched
                } else if caret_count > 1 {
                    PasteMode::MultiCaretBroadcast
                } else {
                    PasteMode::Stream
                }
            }
        }
    }
}

/// Resolved paste mode after examining clipboard entry and context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteMode {
    /// Insert inline at caret.
    Stream,
    /// Insert as new lines above caret line.
    Line,
    /// Insert as column block.
    Rectangular,
    /// Distribute segments to matching carets.
    MultiCaretMatched,
    /// Paste full content at each caret.
    MultiCaretBroadcast,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paste_stream_splits_text_into_lines() {
        // Validates: Requirement 4.1, 4.6
        let entry = ClipboardEntry::stream("hello\nworld".to_string());
        let result = PasteHandler::paste_stream(&entry).unwrap();
        assert_eq!(result.lines_to_insert, vec!["hello", "world"]);
        assert_eq!(result.lines_inserted, 2);
        assert_eq!(result.mode, ClipboardMode::Stream);
    }

    #[test]
    fn paste_stream_trailing_terminator_no_extra_line() {
        // Validates: Requirement 4.7
        let entry = ClipboardEntry::stream("line1\nline2\n".to_string());
        let result = PasteHandler::paste_stream(&entry).unwrap();
        assert_eq!(result.lines_to_insert, vec!["line1", "line2"]);
    }

    #[test]
    fn paste_stream_preserves_whitespace() {
        // Validates: Requirement 4.8
        let entry = ClipboardEntry::stream("  indented\n\ttabbed  ".to_string());
        let result = PasteHandler::paste_stream(&entry).unwrap();
        assert_eq!(result.lines_to_insert, vec!["  indented", "\ttabbed  "]);
    }

    #[test]
    fn paste_stream_empty_returns_error() {
        let entry = ClipboardEntry::stream(String::new());
        let result = PasteHandler::paste_stream(&entry);
        assert!(matches!(result, Err(ClipboardError::Empty)));
    }

    #[test]
    fn paste_line_splits_and_reports_line_mode() {
        // Validates: Requirement 4.2, 14.2, 14.3
        let entry = ClipboardEntry::line("line A\nline B\n".to_string());
        let result = PasteHandler::paste_line(&entry).unwrap();
        assert_eq!(result.lines_to_insert, vec!["line A", "line B"]);
        assert_eq!(result.mode, ClipboardMode::Line);
    }

    #[test]
    fn paste_rectangular_uses_stored_segments() {
        // Validates: Requirement 4.3, 12.2
        let segments = vec!["abc".to_string(), "def".to_string(), "ghi".to_string()];
        let entry = ClipboardEntry::rectangular(segments.clone());
        let config = ClipboardConfig::default();
        let result = PasteHandler::paste_rectangular(&entry, &config).unwrap();
        assert_eq!(result.lines_to_insert, segments);
        assert_eq!(result.mode, ClipboardMode::Rectangular);
    }

    #[test]
    fn paste_multi_caret_matched_distributes_segments() {
        // Validates: Requirement 4.4, 13.2
        let segments = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        let entry = ClipboardEntry::multi_caret(segments);
        let results = PasteHandler::paste_multi_caret_matched(&entry, 3).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].lines_to_insert, vec!["one"]);
        assert_eq!(results[1].lines_to_insert, vec!["two"]);
        assert_eq!(results[2].lines_to_insert, vec!["three"]);
    }

    #[test]
    fn paste_multi_caret_matched_mismatch_returns_error() {
        // Validates: Requirement 4.5, 13.3
        let segments = vec!["one".to_string(), "two".to_string()];
        let entry = ClipboardEntry::multi_caret(segments);
        let result = PasteHandler::paste_multi_caret_matched(&entry, 3);
        assert!(result.is_err());
    }

    #[test]
    fn paste_multi_caret_broadcast_pastes_full_content() {
        // Validates: Requirement 4.5, 13.3
        let entry = ClipboardEntry::stream("full content\n".to_string());
        let results = PasteHandler::paste_multi_caret_broadcast(&entry, 3).unwrap();
        assert_eq!(results.len(), 3);
        for r in &results {
            assert_eq!(r.lines_to_insert, vec!["full content"]);
        }
    }

    #[test]
    fn resolve_paste_mode_stream_single_caret() {
        let entry = ClipboardEntry::stream("text".to_string());
        assert_eq!(
            PasteHandler::resolve_paste_mode(&entry, 1),
            PasteMode::Stream
        );
    }

    #[test]
    fn resolve_paste_mode_line() {
        let entry = ClipboardEntry::line("line\n".to_string());
        assert_eq!(PasteHandler::resolve_paste_mode(&entry, 1), PasteMode::Line);
    }

    #[test]
    fn resolve_paste_mode_rectangular() {
        let entry = ClipboardEntry::rectangular(vec!["a".to_string()]);
        assert_eq!(
            PasteHandler::resolve_paste_mode(&entry, 1),
            PasteMode::Rectangular
        );
    }

    #[test]
    fn resolve_paste_mode_multi_caret_matched() {
        let segments = vec!["a".to_string(), "b".to_string()];
        let entry = ClipboardEntry::multi_caret(segments);
        assert_eq!(
            PasteHandler::resolve_paste_mode(&entry, 2),
            PasteMode::MultiCaretMatched
        );
    }

    #[test]
    fn resolve_paste_mode_multi_caret_broadcast() {
        let entry = ClipboardEntry::stream("text".to_string());
        assert_eq!(
            PasteHandler::resolve_paste_mode(&entry, 3),
            PasteMode::MultiCaretBroadcast
        );
    }
}
