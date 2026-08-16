//! Delete line command execution (D, Dn, DD).

use ff_document_model::{Document, LineNumber};
use ff_edit_operations::{EditorTransaction, LineSnapshot};

use crate::error::LineCommandError;

/// Execute a delete operation — remove `count` lines starting at `start_line`.
///
/// Returns an `EditorTransaction` recording the deleted content.
pub fn execute_delete(
    document: &mut Document,
    start_line: u64,
    count: u64,
) -> Result<EditorTransaction, LineCommandError> {
    let total_lines = document.line_count();

    if start_line >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "delete".to_string(),
            line: start_line,
            total: total_lines,
        });
    }

    if start_line + count > total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "delete".to_string(),
            line: start_line + count - 1,
            total: total_lines,
        });
    }

    // Capture before snapshot
    let mut before_snapshot = Vec::new();
    let mut affected_lines = Vec::new();
    for i in start_line..start_line + count {
        let content = get_line_content(document, i);
        before_snapshot.push(LineSnapshot::new(i, content));
        affected_lines.push(i);
    }

    // Perform deletion from last to first to avoid shifting issues
    for i in (start_line..start_line + count).rev() {
        delete_line(document, i)?;
    }

    Ok(EditorTransaction::new(
        affected_lines,
        before_snapshot,
        vec![], // after snapshot empty — lines are gone
        format!("Delete {} line(s) starting at line {}", count, start_line),
    ))
}

/// Get the text content of a line (without line ending).
pub fn get_line_content(document: &Document, line: u64) -> String {
    let start = document.line_start(LineNumber(line));
    let end = document.line_end(LineNumber(line));
    let len = u64::from(end) - u64::from(start);
    if len == 0 {
        return String::new();
    }
    match document.get_range(start, len) {
        Some(bytes) => {
            let s = String::from_utf8_lossy(&bytes).to_string();
            // Strip trailing line ending
            s.trim_end_matches('\n').trim_end_matches('\r').to_string()
        }
        None => String::new(),
    }
}

/// Delete a single line from the document by its line number.
fn delete_line(document: &mut Document, line: u64) -> Result<(), LineCommandError> {
    let total = document.line_count();
    let start = document.line_start(LineNumber(line));

    if line + 1 < total {
        // Not the last line — delete from line_start(line) to line_start(line+1)
        let next_start = document.line_start(LineNumber(line + 1));
        let delete_len = u64::from(next_start) - u64::from(start);
        if delete_len > 0 {
            document
                .delete(start, delete_len)
                .map_err(|e| LineCommandError::DocumentError {
                    operation: "delete".to_string(),
                    description: e.to_string(),
                })?;
        }
    } else if line > 0 {
        // Last line (not the only line) — delete the preceding newline and line content
        let prev_line_end = document.line_end(LineNumber(line - 1));
        let doc_length = ff_document_model::BytePosition(document.length());
        let delete_len = u64::from(doc_length) - u64::from(prev_line_end);
        if delete_len > 0 {
            document.delete(prev_line_end, delete_len).map_err(|e| {
                LineCommandError::DocumentError {
                    operation: "delete".to_string(),
                    description: e.to_string(),
                }
            })?;
        }
    } else {
        // Only line in the document — delete all content
        let length = document.length();
        if length > 0 {
            document
                .delete(ff_document_model::BytePosition::ZERO, length)
                .map_err(|e| LineCommandError::DocumentError {
                    operation: "delete".to_string(),
                    description: e.to_string(),
                })?;
        }
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
    fn delete_single_line_removes_one_line() {
        let mut doc = make_document(&["line1", "line2", "line3"]);
        assert_eq!(doc.line_count(), 3);

        let txn = execute_delete(&mut doc, 1, 1).unwrap();
        assert_eq!(doc.line_count(), 2);
        assert_eq!(get_line_content(&doc, 0), "line1");
        assert_eq!(get_line_content(&doc, 1), "line3");
        assert_eq!(txn.before_snapshot[0].content, "line2");
    }

    #[test]
    fn delete_counted_removes_n_lines() {
        let mut doc = make_document(&["a", "b", "c", "d", "e"]);
        let txn = execute_delete(&mut doc, 1, 3).unwrap();
        assert_eq!(doc.line_count(), 2);
        assert_eq!(get_line_content(&doc, 0), "a");
        assert_eq!(get_line_content(&doc, 1), "e");
        assert_eq!(txn.before_snapshot.len(), 3);
    }

    #[test]
    fn delete_out_of_range_returns_error() {
        let mut doc = make_document(&["a", "b"]);
        let result = execute_delete(&mut doc, 5, 1);
        assert!(matches!(
            result,
            Err(LineCommandError::LineOutOfRange { .. })
        ));
    }

    #[test]
    fn delete_count_exceeding_doc_returns_error() {
        let mut doc = make_document(&["a", "b", "c"]);
        let result = execute_delete(&mut doc, 1, 5);
        assert!(matches!(
            result,
            Err(LineCommandError::LineOutOfRange { .. })
        ));
    }

    #[test]
    fn delete_first_line() {
        let mut doc = make_document(&["first", "second", "third"]);
        execute_delete(&mut doc, 0, 1).unwrap();
        assert_eq!(doc.line_count(), 2);
        assert_eq!(get_line_content(&doc, 0), "second");
    }

    #[test]
    fn delete_last_line() {
        let mut doc = make_document(&["first", "second", "third"]);
        execute_delete(&mut doc, 2, 1).unwrap();
        assert_eq!(doc.line_count(), 2);
        assert_eq!(get_line_content(&doc, 1), "second");
    }
}
