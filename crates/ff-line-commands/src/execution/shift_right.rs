//! Shift right line command execution (>, >n, >>).

use ff_document_model::{Document, LineNumber};
use ff_edit_operations::{EditorTransaction, LineSnapshot};

use crate::error::LineCommandError;
use crate::execution::delete::get_line_content;

/// Execute a shift-right operation — prepend `columns` spaces to all lines in [start_line, end_line].
///
/// Returns an `EditorTransaction` recording the shift.
pub fn execute_shift_right(
    document: &mut Document,
    start_line: u64,
    end_line: u64,
    columns: u32,
) -> Result<EditorTransaction, LineCommandError> {
    let total_lines = document.line_count();

    if end_line >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "shift_right".to_string(),
            line: end_line,
            total: total_lines,
        });
    }

    let prefix = " ".repeat(columns as usize);
    let mut before_snapshot = Vec::new();
    let mut after_snapshot = Vec::new();
    let mut affected_lines = Vec::new();

    // Process each line from last to first to avoid position shift issues
    for line in (start_line..=end_line).rev() {
        let content = get_line_content(document, line);
        before_snapshot.push(LineSnapshot::new(line, content.clone()));

        // Insert spaces at the beginning of the line
        let line_start = document.line_start(LineNumber(line));
        document
            .insert(line_start, prefix.as_bytes())
            .map_err(|e| LineCommandError::DocumentError {
                operation: "shift_right".to_string(),
                description: e.to_string(),
            })?;

        let new_content = format!("{}{}", prefix, content);
        after_snapshot.push(LineSnapshot::new(line, new_content));
        affected_lines.push(line);
    }

    // Reverse to maintain order
    before_snapshot.reverse();
    after_snapshot.reverse();
    affected_lines.reverse();

    Ok(EditorTransaction::new(
        affected_lines,
        before_snapshot,
        after_snapshot,
        format!(
            "Shift right {} column(s) on lines {}-{}",
            columns, start_line, end_line
        ),
    ))
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
    fn shift_right_single_line_adds_spaces() {
        let mut doc = make_document(&["hello", "world"]);
        execute_shift_right(&mut doc, 0, 0, 3).unwrap();
        assert_eq!(get_line_content(&doc, 0), "   hello");
        assert_eq!(get_line_content(&doc, 1), "world");
    }

    #[test]
    fn shift_right_block_adds_spaces_to_all() {
        let mut doc = make_document(&["a", "b", "c"]);
        execute_shift_right(&mut doc, 0, 2, 2).unwrap();
        assert_eq!(get_line_content(&doc, 0), "  a");
        assert_eq!(get_line_content(&doc, 1), "  b");
        assert_eq!(get_line_content(&doc, 2), "  c");
    }

    #[test]
    fn shift_right_counted_uses_specified_columns() {
        let mut doc = make_document(&["test"]);
        execute_shift_right(&mut doc, 0, 0, 5).unwrap();
        assert_eq!(get_line_content(&doc, 0), "     test");
    }

    #[test]
    fn shift_right_out_of_range_returns_error() {
        let mut doc = make_document(&["a"]);
        let result = execute_shift_right(&mut doc, 5, 5, 2);
        assert!(matches!(
            result,
            Err(LineCommandError::LineOutOfRange { .. })
        ));
    }

    #[test]
    fn shift_right_preserves_line_count() {
        let mut doc = make_document(&["a", "b", "c"]);
        let before_count = doc.line_count();
        execute_shift_right(&mut doc, 0, 2, 4).unwrap();
        assert_eq!(doc.line_count(), before_count);
    }
}
