//! Display line mapper trait for wrapping/folding integration.
//!
//! The viewport model consumes this trait to correctly handle scroll operations
//! when document lines differ from display lines (due to word-wrap, folding,
//! or line exclusion).

/// Translates between document lines and display lines for correct scrolling
/// with wrapping and folding.
///
/// When no mapper is attached, the viewport model uses an identity mapping
/// (each document line = exactly one display line).
pub trait DisplayLineMapper: Send + Sync {
    /// Total number of display lines (accounting for wraps and folds).
    fn total_display_lines(&self) -> u64;

    /// Convert a document line to its first display line.
    fn doc_to_display(&self, doc_line: u64) -> u64;

    /// Convert a display line to its document line.
    fn display_to_doc(&self, display_line: u64) -> u64;

    /// Whether a document line is currently visible (not folded/excluded).
    fn is_visible(&self, doc_line: u64) -> bool;

    /// Number of display lines produced by a document line (wrapping).
    fn display_lines_for_doc_line(&self, doc_line: u64) -> u64;
}

/// Identity mapper: 1:1 mapping between document lines and display lines.
/// Used as a fallback when no DisplayLineMapper is provided.
#[allow(dead_code)]
pub(crate) struct IdentityMapper {
    pub(crate) total_lines: u64,
}

impl DisplayLineMapper for IdentityMapper {
    fn total_display_lines(&self) -> u64 {
        self.total_lines
    }

    fn doc_to_display(&self, doc_line: u64) -> u64 {
        doc_line
    }

    fn display_to_doc(&self, display_line: u64) -> u64 {
        display_line
    }

    fn is_visible(&self, _doc_line: u64) -> bool {
        true
    }

    fn display_lines_for_doc_line(&self, _doc_line: u64) -> u64 {
        1
    }
}
