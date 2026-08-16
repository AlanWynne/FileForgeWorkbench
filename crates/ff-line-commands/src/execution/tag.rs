//! Tag and Untag line command execution (T, TT, U, UU).
//!
//! Session-state only — does NOT produce an EditorTransaction.
//! Tag/untag state is maintained externally (this implementation is a no-op
//! placeholder since the Document model doesn't directly store tag state).
//! In the full system, tag state is stored in DocumentSession.

use ff_document_model::Document;

use crate::error::LineCommandError;

/// Execute a tag operation — set tagged flag on lines [start_line, end_line].
///
/// This is a session-state operation that does NOT produce an EditorTransaction.
/// In the full system, tag state is stored in DocumentSession.
pub fn execute_tag(
    _document: &mut Document,
    _start_line: u64,
    end_line: u64,
) -> Result<(), LineCommandError> {
    let total = _document.line_count();
    if end_line >= total {
        return Err(LineCommandError::LineOutOfRange {
            operation: "tag".to_string(),
            line: end_line,
            total,
        });
    }
    // Tag state is stored in the session layer, not in Document directly.
    // This function validates bounds and signals success.
    // The actual tag storage is handled by the caller/session.
    Ok(())
}

/// Execute an untag operation — clear tagged flag on lines [start_line, end_line].
///
/// This is a session-state operation that does NOT produce an EditorTransaction.
pub fn execute_untag(
    _document: &mut Document,
    _start_line: u64,
    end_line: u64,
) -> Result<(), LineCommandError> {
    let total = _document.line_count();
    if end_line >= total {
        return Err(LineCommandError::LineOutOfRange {
            operation: "untag".to_string(),
            line: end_line,
            total,
        });
    }
    Ok(())
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
    fn tag_single_line_succeeds() {
        let mut doc = make_document(&["a", "b", "c"]);
        let result = execute_tag(&mut doc, 1, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn tag_block_succeeds() {
        let mut doc = make_document(&["a", "b", "c", "d"]);
        let result = execute_tag(&mut doc, 0, 3);
        assert!(result.is_ok());
    }

    #[test]
    fn tag_out_of_range_returns_error() {
        let mut doc = make_document(&["a", "b"]);
        let result = execute_tag(&mut doc, 0, 5);
        assert!(matches!(
            result,
            Err(LineCommandError::LineOutOfRange { .. })
        ));
    }

    #[test]
    fn untag_single_line_succeeds() {
        let mut doc = make_document(&["a", "b", "c"]);
        let result = execute_untag(&mut doc, 2, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn untag_out_of_range_returns_error() {
        let mut doc = make_document(&["a"]);
        let result = execute_untag(&mut doc, 0, 5);
        assert!(matches!(
            result,
            Err(LineCommandError::LineOutOfRange { .. })
        ));
    }

    #[test]
    fn tag_does_not_produce_transaction() {
        // Verified by API design — returns () not EditorTransaction
        let mut doc = make_document(&["a", "b"]);
        let result = execute_tag(&mut doc, 0, 1);
        assert!(result.is_ok());
    }
}
