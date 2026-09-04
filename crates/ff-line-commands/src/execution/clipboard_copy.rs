//! Clipboard copy line command execution (W, WW).
//!
//! Collects line(s) content as a String for the caller to write to the
//! system clipboard. This crate does not depend on ff-clipboard (which
//! depends on ff-line-commands), so the clipboard write is delegated to
//! the caller layer (ff-desktop or ff-clipboard).
//!
//! This is a session-state operation -- it does NOT produce an EditorTransaction.

use ff_document_model::Document;

use crate::error::LineCommandError;
use crate::execution::delete::get_line_content;

/// Collect lines [start_line, end_line] as a newline-joined String for clipboard writing.
///
/// Returns the text to be written to the clipboard. The caller is responsible
/// for the actual clipboard write. No EditorTransaction is produced.
pub fn collect_clipboard_text(
    document: &Document,
    start_line: u64,
    end_line: u64,
) -> Result<String, LineCommandError> {
    let total_lines = document.line_count();

    if end_line >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "clipboard_copy".to_string(),
            line: end_line,
            total: total_lines,
        });
    }

    let lines: Vec<String> = (start_line..=end_line)
        .map(|l| get_line_content(document, l))
        .collect();
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_document(lines: &[&str]) -> Document {
        let mut doc = Document::new();
        let content = lines.join("\n");
        if !content.is_empty() {
            doc.insert(ff_document_model::BytePosition::ZERO, content.as_bytes())
                .unwrap();
        }
        doc
    }

    #[test]
    fn collect_single_line_returns_line_text() {
        // Validates: Requirement 15.3
        let doc = make_document(&["hello world", "second line"]);
        let text = collect_clipboard_text(&doc, 0, 0).unwrap();
        assert_eq!(text, "hello world");
    }

    #[test]
    fn collect_block_joins_lines_with_newline() {
        // Validates: Requirement 15.4
        let doc = make_document(&["line one", "line two", "line three"]);
        let text = collect_clipboard_text(&doc, 0, 2).unwrap();
        assert_eq!(text, "line one\nline two\nline three");
    }

    #[test]
    fn collect_does_not_modify_document() {
        // Validates: Requirement 15.11 -- document unchanged after collection
        let doc = make_document(&["unchanged"]);
        let before_count = doc.line_count();
        collect_clipboard_text(&doc, 0, 0).unwrap();
        assert_eq!(doc.line_count(), before_count);
        assert_eq!(get_line_content(&doc, 0), "unchanged");
    }

    #[test]
    fn collect_out_of_range_returns_error() {
        let doc = make_document(&["a", "b"]);
        let result = collect_clipboard_text(&doc, 0, 5);
        assert!(matches!(
            result,
            Err(LineCommandError::LineOutOfRange { .. })
        ));
    }

    #[test]
    fn collect_returns_string_not_transaction() {
        // Validates: Requirement 15.11 -- return type is String, not EditorTransaction
        let doc = make_document(&["test"]);
        let result = collect_clipboard_text(&doc, 0, 0);
        assert!(result.is_ok());
        // The Ok value is a String -- no transaction by design
        let text = result.unwrap();
        assert_eq!(text, "test");
    }
}
