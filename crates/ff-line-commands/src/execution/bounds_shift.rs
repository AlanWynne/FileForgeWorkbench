//! Bounds-aware shift line command execution (), )), (, (().
//!
//! Shifts content within column bounds, preserving characters outside bounds.

use ff_document_model::{Document, LineNumber};
use ff_edit_operations::{EditBounds, EditorTransaction, LineSnapshot};

use crate::error::LineCommandError;
use crate::execution::delete::get_line_content;

/// Execute a bounds-aware shift right — shift content within bounds one position right.
///
/// Characters outside the bounds are preserved exactly.
pub fn execute_bounds_shift_right(
    document: &mut Document,
    start_line: u64,
    end_line: u64,
    bounds: &EditBounds,
) -> Result<EditorTransaction, LineCommandError> {
    let total_lines = document.line_count();

    if end_line >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "bounds_shift_right".to_string(),
            line: end_line,
            total: total_lines,
        });
    }

    let mut before_snapshot = Vec::new();
    let mut after_snapshot = Vec::new();
    let mut affected_lines = Vec::new();

    // Process each line from last to first
    for line in (start_line..=end_line).rev() {
        let content = get_line_content(document, line);
        before_snapshot.push(LineSnapshot::new(line, content.clone()));

        let new_content = shift_within_bounds_right(&content, bounds);
        after_snapshot.push(LineSnapshot::new(line, new_content.clone()));
        affected_lines.push(line);

        // Replace the line content
        replace_line_content(document, line, &new_content)?;
    }

    before_snapshot.reverse();
    after_snapshot.reverse();
    affected_lines.reverse();

    Ok(EditorTransaction::new(
        affected_lines,
        before_snapshot,
        after_snapshot,
        format!(
            "Bounds shift right on lines {}-{} (bounds {}-{})",
            start_line, end_line, bounds.left, bounds.right
        ),
    ))
}

/// Execute a bounds-aware shift left — shift content within bounds one position left.
///
/// Characters outside the bounds are preserved exactly.
pub fn execute_bounds_shift_left(
    document: &mut Document,
    start_line: u64,
    end_line: u64,
    bounds: &EditBounds,
) -> Result<EditorTransaction, LineCommandError> {
    let total_lines = document.line_count();

    if end_line >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "bounds_shift_left".to_string(),
            line: end_line,
            total: total_lines,
        });
    }

    let mut before_snapshot = Vec::new();
    let mut after_snapshot = Vec::new();
    let mut affected_lines = Vec::new();

    for line in (start_line..=end_line).rev() {
        let content = get_line_content(document, line);
        before_snapshot.push(LineSnapshot::new(line, content.clone()));

        let new_content = shift_within_bounds_left(&content, bounds);
        after_snapshot.push(LineSnapshot::new(line, new_content.clone()));
        affected_lines.push(line);

        replace_line_content(document, line, &new_content)?;
    }

    before_snapshot.reverse();
    after_snapshot.reverse();
    affected_lines.reverse();

    Ok(EditorTransaction::new(
        affected_lines,
        before_snapshot,
        after_snapshot,
        format!(
            "Bounds shift left on lines {}-{} (bounds {}-{})",
            start_line, end_line, bounds.left, bounds.right
        ),
    ))
}

/// Shift the content within [left, right] columns one position to the right.
///
/// Bounds are 1-based. Characters outside the bounds are unchanged.
/// The rightmost character within bounds is lost (shifted off the right edge).
fn shift_within_bounds_right(content: &str, bounds: &EditBounds) -> String {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();

    // Bounds are 1-based columns. Convert to 0-based indices.
    let left_idx = (bounds.left as usize).saturating_sub(1);
    let right_idx = (bounds.right as usize).saturating_sub(1);

    if left_idx >= len {
        // Line is shorter than bounds start — nothing to shift
        return content.to_string();
    }

    let mut result: Vec<char> = chars.clone();

    // Pad to at least right_idx + 1 characters if needed
    while result.len() <= right_idx {
        result.push(' ');
    }

    // Shift content within bounds right by 1
    // Rightmost char is lost, leftmost position gets a space
    let capped_right = right_idx.min(result.len() - 1);

    for i in (left_idx + 1..=capped_right).rev() {
        result[i] = result[i - 1];
    }
    if left_idx <= capped_right {
        result[left_idx] = ' ';
    }

    // Trim trailing spaces back to original length if line was padded
    while result.len() > len && result.last() == Some(&' ') {
        result.pop();
    }

    result.into_iter().collect()
}

/// Shift the content within [left, right] columns one position to the left.
///
/// Bounds are 1-based. Characters outside the bounds are unchanged.
/// The leftmost character within bounds is lost (shifted off the left edge).
fn shift_within_bounds_left(content: &str, bounds: &EditBounds) -> String {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();

    let left_idx = (bounds.left as usize).saturating_sub(1);
    let right_idx = (bounds.right as usize).saturating_sub(1);

    if left_idx >= len {
        return content.to_string();
    }

    let mut result: Vec<char> = chars.clone();

    while result.len() <= right_idx {
        result.push(' ');
    }

    let capped_right = right_idx.min(result.len() - 1);

    // Shift content within bounds left by 1
    // Leftmost char is lost, rightmost position gets a space
    for i in left_idx..capped_right {
        result[i] = result[i + 1];
    }
    if capped_right >= left_idx {
        result[capped_right] = ' ';
    }

    // Trim trailing spaces back to original length if line was padded
    while result.len() > len && result.last() == Some(&' ') {
        result.pop();
    }

    result.into_iter().collect()
}

/// Replace the entire content of a line in the document.
fn replace_line_content(
    document: &mut Document,
    line: u64,
    new_content: &str,
) -> Result<(), LineCommandError> {
    let line_start = document.line_start(LineNumber(line));
    let line_end = document.line_end(LineNumber(line));
    let old_len = u64::from(line_end) - u64::from(line_start);

    // Get existing content to check for line endings
    let existing = if old_len > 0 {
        document.get_range(line_start, old_len).unwrap_or_default()
    } else {
        vec![]
    };

    // Find the actual content length (without line ending)
    let content_len = existing
        .iter()
        .rposition(|&b| b != b'\n' && b != b'\r')
        .map(|i| i + 1)
        .unwrap_or(0);

    // Delete old content (just the text, not the line ending)
    if content_len > 0 {
        document
            .delete(line_start, content_len as u64)
            .map_err(|e| LineCommandError::DocumentError {
                operation: "bounds_shift".to_string(),
                description: e.to_string(),
            })?;
    }

    // Insert new content at the same position
    if !new_content.is_empty() {
        document
            .insert(line_start, new_content.as_bytes())
            .map_err(|e| LineCommandError::DocumentError {
                operation: "bounds_shift".to_string(),
                description: e.to_string(),
            })?;
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
    fn bounds_shift_right_preserves_outer_content() {
        // "ABCDEFGHIJ" with bounds [3, 7] (1-based)
        // Chars: A B C D E F G H I J
        //        1 2 3 4 5 6 7 8 9 10
        // Within bounds: C D E F G → shift right → " C D E F"
        // Result: "AB CDEFHIJ" (G is lost from right edge of bounds)
        let content = "ABCDEFGHIJ";
        let bounds = EditBounds::new(3, 7).unwrap();
        let result = shift_within_bounds_right(content, &bounds);
        assert_eq!(&result[..2], "AB"); // Outside bounds left
        assert_eq!(&result[7..], "HIJ"); // Outside bounds right
    }

    #[test]
    fn bounds_shift_left_preserves_outer_content() {
        let content = "ABCDEFGHIJ";
        let bounds = EditBounds::new(3, 7).unwrap();
        let result = shift_within_bounds_left(content, &bounds);
        assert_eq!(&result[..2], "AB"); // Outside bounds left
        assert_eq!(&result[7..], "HIJ"); // Outside bounds right
    }

    #[test]
    fn bounds_shift_right_inserts_space_at_left() {
        let content = "ABCDEFGHIJ";
        let bounds = EditBounds::new(3, 7).unwrap();
        let result = shift_within_bounds_right(content, &bounds);
        // Position 3 (0-idx 2) should be space
        let chars: Vec<char> = result.chars().collect();
        assert_eq!(chars[2], ' ');
    }

    #[test]
    fn bounds_shift_left_inserts_space_at_right() {
        let content = "ABCDEFGHIJ";
        let bounds = EditBounds::new(3, 7).unwrap();
        let result = shift_within_bounds_left(content, &bounds);
        // Position 7 (0-idx 6) should be space
        let chars: Vec<char> = result.chars().collect();
        assert_eq!(chars[6], ' ');
    }

    #[test]
    fn execute_bounds_shift_right_on_document() {
        let mut doc = make_document(&["ABCDEFGHIJ"]);
        let bounds = EditBounds::new(3, 7).unwrap();
        let txn = execute_bounds_shift_right(&mut doc, 0, 0, &bounds).unwrap();
        assert!(txn.is_valid());
    }

    #[test]
    fn execute_bounds_shift_left_on_document() {
        let mut doc = make_document(&["ABCDEFGHIJ"]);
        let bounds = EditBounds::new(3, 7).unwrap();
        let txn = execute_bounds_shift_left(&mut doc, 0, 0, &bounds).unwrap();
        assert!(txn.is_valid());
    }

    #[test]
    fn bounds_shift_out_of_range_returns_error() {
        let mut doc = make_document(&["a"]);
        let bounds = EditBounds::new(1, 5).unwrap();
        let result = execute_bounds_shift_right(&mut doc, 5, 5, &bounds);
        assert!(matches!(
            result,
            Err(LineCommandError::LineOutOfRange { .. })
        ));
    }
}
