//! Shift left line command execution (<, <n, <<).
//!
//! Data-loss prevention: truncates only up to first non-whitespace character.

use ff_document_model::{Document, LineNumber};
use ff_edit_operations::{EditorTransaction, LineSnapshot};

use crate::error::LineCommandError;
use crate::execution::delete::get_line_content;

/// Execute a shift-left operation — remove leading whitespace up to `columns` chars
/// from all lines in [start_line, end_line].
///
/// Data-loss prevention: never removes non-whitespace characters.
pub fn execute_shift_left(
    document: &mut Document,
    start_line: u64,
    end_line: u64,
    columns: u32,
) -> Result<EditorTransaction, LineCommandError> {
    let total_lines = document.line_count();

    if end_line >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "shift_left".to_string(),
            line: end_line,
            total: total_lines,
        });
    }

    let mut before_snapshot = Vec::new();
    let mut after_snapshot = Vec::new();
    let mut affected_lines = Vec::new();

    // Process each line from last to first to avoid position shift issues
    for line in (start_line..=end_line).rev() {
        let content = get_line_content(document, line);
        before_snapshot.push(LineSnapshot::new(line, content.clone()));

        // Calculate how many leading whitespace characters to remove
        let leading_ws = content.chars().take_while(|c| c.is_whitespace()).count();
        let actual_shift = (columns as usize).min(leading_ws);

        if actual_shift > 0 {
            let line_start = document.line_start(LineNumber(line));
            document
                .delete(line_start, actual_shift as u64)
                .map_err(|e| LineCommandError::DocumentError {
                    operation: "shift_left".to_string(),
                    description: e.to_string(),
                })?;
        }

        let new_content = content[actual_shift..].to_string();
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
            "Shift left {} column(s) on lines {}-{}",
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
    fn shift_left_removes_leading_whitespace() {
        let mut doc = make_document(&["    hello", "world"]);
        execute_shift_left(&mut doc, 0, 0, 2).unwrap();
        assert_eq!(get_line_content(&doc, 0), "  hello");
    }

    #[test]
    fn shift_left_non_destructive_preserves_content() {
        let mut doc = make_document(&["  hello"]);
        execute_shift_left(&mut doc, 0, 0, 10).unwrap();
        // Should only remove the 2 leading spaces, not damage "hello"
        assert_eq!(get_line_content(&doc, 0), "hello");
    }

    #[test]
    fn shift_left_no_whitespace_does_nothing() {
        let mut doc = make_document(&["hello"]);
        execute_shift_left(&mut doc, 0, 0, 5).unwrap();
        assert_eq!(get_line_content(&doc, 0), "hello");
    }

    #[test]
    fn shift_left_block_shifts_all_lines() {
        let mut doc = make_document(&["    a", "  b", "      c"]);
        execute_shift_left(&mut doc, 0, 2, 3).unwrap();
        assert_eq!(get_line_content(&doc, 0), " a");
        assert_eq!(get_line_content(&doc, 1), "b"); // only 2 spaces available
        assert_eq!(get_line_content(&doc, 2), "   c");
    }

    #[test]
    fn shift_left_out_of_range_returns_error() {
        let mut doc = make_document(&["a"]);
        let result = execute_shift_left(&mut doc, 5, 5, 2);
        assert!(matches!(
            result,
            Err(LineCommandError::LineOutOfRange { .. })
        ));
    }

    #[test]
    fn shift_left_preserves_line_count() {
        let mut doc = make_document(&["  a", "  b", "  c"]);
        let before_count = doc.line_count();
        execute_shift_left(&mut doc, 0, 2, 2).unwrap();
        assert_eq!(doc.line_count(), before_count);
    }
}
