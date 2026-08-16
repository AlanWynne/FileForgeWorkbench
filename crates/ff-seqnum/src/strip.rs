//! Strip engine — core column clearing logic.
//!
//! Replaces sequence column content with spaces and stores originals
//! in the side-table for potential restoration or overlay display.

use crate::state::{SeqNumState, SideTable};
use crate::traits::DocumentMutate;
use crate::types::ColumnRange;

/// Result of a strip operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StripResult {
    /// Number of lines modified (had non-blank content cleared).
    pub lines_modified: usize,
    /// Total lines examined.
    pub lines_examined: usize,
    /// Column ranges that were stripped.
    pub columns_stripped: Vec<ColumnRange>,
}

/// Strip the specified column range from a single line, returning the new line content.
///
/// If the line is shorter than `range.start_offset()`, it is left unchanged (returns None).
/// Replaces column range bytes with spaces.
pub fn strip_columns(line: &str, range: &ColumnRange) -> Option<String> {
    let start = range.start_offset();
    let end = range.end_offset();

    if line.len() <= start {
        return None; // Line shorter than start — unchanged
    }

    let actual_end = end.min(line.len());
    let col_content = &line[start..actual_end];

    // Skip if already all spaces
    if col_content.chars().all(|c| c == ' ') {
        return None;
    }

    let mut result = String::with_capacity(line.len());
    result.push_str(&line[..start]);
    result.push_str(&" ".repeat(actual_end - start));
    if actual_end < line.len() {
        result.push_str(&line[actual_end..]);
    }
    Some(result)
}

/// Extract the column content from a line for the given range.
///
/// Returns None if the line is shorter than the start of the range.
pub fn extract_columns(line: &str, range: &ColumnRange) -> Option<String> {
    let start = range.start_offset();
    let end = range.end_offset();

    if line.len() <= start {
        return None;
    }

    let actual_end = end.min(line.len());
    Some(line[start..actual_end].to_string())
}

/// Strip sequence columns from all lines in the document.
///
/// Stores originals in the side-table. Returns the count of modified lines.
/// Lines where the range is already all spaces are left unchanged.
pub fn strip_document(
    document: &mut dyn DocumentMutate,
    ranges: &[ColumnRange],
    state: &mut SeqNumState,
) -> StripResult {
    let line_count = document.line_count();
    strip_range_impl(document, ranges, 0, line_count, &mut state.side_table)
}

/// Strip sequence columns from a range of lines (CC block scoped).
pub fn strip_line_range(
    document: &mut dyn DocumentMutate,
    ranges: &[ColumnRange],
    start_line: usize,
    end_line: usize,
    side_table: &mut SideTable,
) -> StripResult {
    strip_range_impl(document, ranges, start_line, end_line, side_table)
}

/// Internal implementation for range-based stripping.
fn strip_range_impl(
    document: &mut dyn DocumentMutate,
    ranges: &[ColumnRange],
    start_line: usize,
    end_line: usize,
    side_table: &mut SideTable,
) -> StripResult {
    let actual_end = end_line.min(document.line_count());
    let mut lines_modified = 0;

    for line_idx in start_line..actual_end {
        let mut line_was_modified = false;

        if let Some(line_content) = document.line_content(line_idx) {
            let line_content = line_content.to_string(); // Borrow release

            for range in ranges {
                let original = extract_columns(&line_content, range);
                if let Some(ref orig) = original {
                    // Check if already all spaces
                    if orig.chars().all(|c| c == ' ') {
                        continue;
                    }
                    // Store original in side-table
                    if ranges.len() == 1 {
                        // Single range — determine if it's front or back by position
                        if range.start() <= 10 {
                            side_table.store_stripped_values(line_idx, Some(orig), None);
                        } else {
                            side_table.store_stripped_values(line_idx, None, Some(orig));
                        }
                    } else if range == ranges.first().unwrap() {
                        side_table.store_stripped_values(line_idx, Some(orig), None);
                    } else {
                        side_table.store_stripped_values(line_idx, None, Some(orig));
                    }

                    // Replace with spaces
                    let spaces = " ".repeat(range.width() as usize);
                    document.replace_columns(line_idx, range, &spaces);
                    line_was_modified = true;
                }
            }
        }

        if line_was_modified {
            lines_modified += 1;
        }
    }

    StripResult {
        lines_modified,
        lines_examined: actual_end - start_line,
        columns_stripped: ranges.to_vec(),
    }
}

/// Restore previously stripped content from the side-table back into the document.
///
/// Used for UNDO reversal of strip operations and `restore_on_save`.
pub fn restore_from_side_table(
    document: &mut dyn DocumentMutate,
    side_table: &SideTable,
    front_range: Option<&ColumnRange>,
    back_range: Option<&ColumnRange>,
) -> usize {
    let mut lines_restored = 0;

    for (&line_idx, entry) in side_table.iter() {
        let mut restored = false;

        if let (Some(ref content), Some(range)) = (&entry.front_content, front_range) {
            document.replace_columns(line_idx, range, content);
            restored = true;
        }

        if let (Some(ref content), Some(range)) = (&entry.back_content, back_range) {
            document.replace_columns(line_idx, range, content);
            restored = true;
        }

        if restored {
            lines_restored += 1;
        }
    }

    lines_restored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::DocumentAccess;

    // ─── Test Helpers ───────────────────────────────────────────────────────

    struct MockDoc {
        lines: Vec<String>,
    }

    impl MockDoc {
        fn new(lines: &[&str]) -> Self {
            Self {
                lines: lines.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl DocumentAccess for MockDoc {
        fn line_count(&self) -> usize {
            self.lines.len()
        }

        fn line_content(&self, index: usize) -> Option<&str> {
            self.lines.get(index).map(|s| s.as_str())
        }
    }

    impl DocumentMutate for MockDoc {
        fn replace_columns(&mut self, line_index: usize, range: &ColumnRange, content: &str) {
            if let Some(line) = self.lines.get_mut(line_index) {
                let start = range.start_offset();
                let end = range.end_offset();
                if line.len() <= start {
                    return;
                }
                let actual_end = end.min(line.len());
                let mut new_line = String::with_capacity(line.len());
                new_line.push_str(&line[..start]);
                new_line.push_str(content);
                if actual_end < line.len() {
                    new_line.push_str(&line[actual_end..]);
                }
                *line = new_line;
            }
        }
    }

    fn make_80col_line(front: &str, body: &str, back: &str) -> String {
        let f = format!("{:<6}", front);
        let b_pad = format!("{:<66}", body);
        let bk = format!("{:<8}", back);
        format!("{}{}{}", &f[..6], &b_pad[..66], &bk[..8])
    }

    // ─── Unit Tests ─────────────────────────────────────────────────────────

    #[test]
    fn strip_columns_replaces_with_spaces() {
        // Validates: Requirement 3.2
        let line = "000100 MOVE A TO B.                                                  00000100";
        let range = ColumnRange::new(1, 6).unwrap();
        let result = strip_columns(line, &range).unwrap();
        assert_eq!(&result[..6], "      ");
        assert_eq!(&result[6..], &line[6..]);
    }

    #[test]
    fn strip_columns_line_shorter_than_start_returns_none() {
        // Validates: Requirement 3.2 (lines shorter than range start unchanged)
        let line = "short";
        let range = ColumnRange::new(73, 80).unwrap();
        assert!(strip_columns(line, &range).is_none());
    }

    #[test]
    fn strip_columns_already_blank_returns_none() {
        // Validates: Requirement 5.8
        let line = "       MOVE A TO B.";
        let range = ColumnRange::new(1, 6).unwrap();
        // First 6 chars are spaces
        assert!(strip_columns(line, &range).is_none());
    }

    #[test]
    fn strip_document_stores_originals_in_side_table() {
        // Validates: Requirement 3.9
        let lines: Vec<String> = (1..=5)
            .map(|i| {
                make_80col_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut doc = MockDoc::new(&line_refs);
        let mut state = SeqNumState::new();
        let front = ColumnRange::new(1, 6).unwrap();
        let back = ColumnRange::new(73, 80).unwrap();

        let result = strip_document(&mut doc, &[front, back], &mut state);

        assert_eq!(result.lines_modified, 5);
        assert_eq!(result.lines_examined, 5);

        // Verify side-table has entries
        assert!(!state.side_table.is_empty());
        let entry = state.side_table.get_original_values(0).unwrap();
        assert_eq!(entry.front_content.as_deref(), Some("000100"));
        assert_eq!(entry.back_content.as_deref(), Some("00000100"));
    }

    #[test]
    fn strip_document_skips_already_blank_lines() {
        // Validates: Requirement 5.8
        let lines = vec![
            make_80col_line("000100", " CODE.", "00000100"),
            make_80col_line("      ", " CODE.", "        "),
            make_80col_line("000300", " CODE.", "00000300"),
        ];
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut doc = MockDoc::new(&line_refs);
        let mut state = SeqNumState::new();
        let front = ColumnRange::new(1, 6).unwrap();
        let back = ColumnRange::new(73, 80).unwrap();

        let result = strip_document(&mut doc, &[front, back], &mut state);

        assert_eq!(result.lines_modified, 2); // Line 1 is already blank
    }

    #[test]
    fn strip_range_restricts_to_scope() {
        // Validates: Requirement 5.7
        let lines: Vec<String> = (1..=10)
            .map(|i| {
                make_80col_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut doc = MockDoc::new(&line_refs);
        let mut side_table = SideTable::new();
        let front = ColumnRange::new(1, 6).unwrap();

        let result = strip_line_range(&mut doc, &[front], 2, 5, &mut side_table);

        assert_eq!(result.lines_modified, 3); // Lines 2, 3, 4
        assert_eq!(result.lines_examined, 3);

        // Lines 0, 1 should be unchanged
        assert!(doc.line_content(0).unwrap().starts_with("000100"));
        assert!(doc.line_content(1).unwrap().starts_with("000200"));
        // Lines 2-4 should be stripped
        assert!(doc.line_content(2).unwrap().starts_with("      "));
    }

    #[test]
    fn restore_from_side_table_restores_originals() {
        // Validates: Requirement 9.5
        let lines: Vec<String> = (1..=3)
            .map(|i| {
                make_80col_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let original_lines = line_refs.clone();
        let mut doc = MockDoc::new(&line_refs);
        let mut state = SeqNumState::new();
        let front = ColumnRange::new(1, 6).unwrap();
        let back = ColumnRange::new(73, 80).unwrap();

        // Strip
        strip_document(&mut doc, &[front, back], &mut state);

        // Verify stripped
        assert!(doc.line_content(0).unwrap().starts_with("      "));

        // Restore
        let restored =
            restore_from_side_table(&mut doc, &state.side_table, Some(&front), Some(&back));
        assert_eq!(restored, 3);

        // Verify restored content matches original
        for (i, original) in original_lines.iter().enumerate() {
            assert_eq!(doc.line_content(i).unwrap(), *original);
        }
    }
}
