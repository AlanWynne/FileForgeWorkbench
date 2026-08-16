//! Insert line command execution (I, In).

use ff_document_model::{Document, LineNumber};
use ff_edit_operations::{EditorTransaction, LineSnapshot};

use crate::error::LineCommandError;

/// Execute an insert operation — insert `count` blank lines after `after_line`.
///
/// Returns an `EditorTransaction` recording the insertion.
pub fn execute_insert(
    document: &mut Document,
    after_line: u64,
    count: u32,
) -> Result<EditorTransaction, LineCommandError> {
    let total_lines = document.line_count();

    if after_line >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "insert".to_string(),
            line: after_line,
            total: total_lines,
        });
    }

    // Insert position: after the end of the target line (including its newline)
    let insert_pos = if after_line + 1 < total_lines {
        document.line_start(LineNumber(after_line + 1))
    } else {
        document.line_end(LineNumber(after_line))
    };

    // Build the content to insert: count blank lines
    let newlines: String = "\n".repeat(count as usize);

    document
        .insert(insert_pos, newlines.as_bytes())
        .map_err(|e| LineCommandError::DocumentError {
            operation: "insert".to_string(),
            description: e.to_string(),
        })?;

    // Build after snapshot
    let mut after_snapshot = Vec::new();
    let mut affected_lines = Vec::new();
    for i in 0..u64::from(count) {
        let line_num = after_line + 1 + i;
        after_snapshot.push(LineSnapshot::new(line_num, String::new()));
        affected_lines.push(line_num);
    }

    Ok(EditorTransaction::new(
        affected_lines,
        vec![], // before snapshot empty — lines didn't exist
        after_snapshot,
        format!("Insert {} blank line(s) after line {}", count, after_line),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::delete::get_line_content;

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
    fn insert_single_blank_line() {
        let mut doc = make_document(&["line1", "line2", "line3"]);
        assert_eq!(doc.line_count(), 3);

        let txn = execute_insert(&mut doc, 1, 1).unwrap();
        assert_eq!(doc.line_count(), 4);
        assert_eq!(get_line_content(&doc, 0), "line1");
        assert_eq!(get_line_content(&doc, 1), "line2");
        assert_eq!(get_line_content(&doc, 2), "");
        assert_eq!(get_line_content(&doc, 3), "line3");
        assert_eq!(txn.after_snapshot.len(), 1);
    }

    #[test]
    fn insert_counted_blank_lines() {
        let mut doc = make_document(&["a", "b"]);
        execute_insert(&mut doc, 0, 3).unwrap();
        assert_eq!(doc.line_count(), 5);
        assert_eq!(get_line_content(&doc, 0), "a");
        assert_eq!(get_line_content(&doc, 1), "");
        assert_eq!(get_line_content(&doc, 2), "");
        assert_eq!(get_line_content(&doc, 3), "");
        assert_eq!(get_line_content(&doc, 4), "b");
    }

    #[test]
    fn insert_out_of_range_returns_error() {
        let mut doc = make_document(&["a"]);
        let result = execute_insert(&mut doc, 5, 1);
        assert!(matches!(
            result,
            Err(LineCommandError::LineOutOfRange { .. })
        ));
    }

    #[test]
    fn insert_after_last_line() {
        let mut doc = make_document(&["first", "last"]);
        execute_insert(&mut doc, 1, 2).unwrap();
        assert_eq!(doc.line_count(), 4);
        assert_eq!(get_line_content(&doc, 0), "first");
        assert_eq!(get_line_content(&doc, 1), "last");
    }
}
