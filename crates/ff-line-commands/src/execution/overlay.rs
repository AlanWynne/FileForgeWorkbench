//! Overlay line command execution (O, On).
//!
//! Overlays target line(s) with source content: non-blank source characters
//! replace the corresponding target characters. Blank source characters leave
//! the target character unchanged.

use ff_document_model::{Document, LineNumber};
use ff_edit_operations::{EditorTransaction, LineSnapshot};

use crate::error::LineCommandError;
use crate::execution::delete::get_line_content;

/// Overlay `count` target lines starting at `target_start` with source lines
/// starting at `source_start`.
///
/// For each position, if the source character is non-blank it replaces the
/// target character; blank source characters leave the target unchanged.
/// Lines are padded with spaces if the source is longer than the target.
///
/// Returns an `EditorTransaction` recording the change.
pub fn execute_overlay(
    document: &mut Document,
    source_start: u64,
    source_end: u64,
    target_start: u64,
    count: u32,
) -> Result<EditorTransaction, LineCommandError> {
    let total_lines = document.line_count();
    let target_end = target_start + count as u64 - 1;

    if source_end >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "overlay".to_string(),
            line: source_end,
            total: total_lines,
        });
    }
    if target_end >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "overlay".to_string(),
            line: target_end,
            total: total_lines,
        });
    }

    let source_count = (source_end - source_start + 1) as usize;
    let mut before_snapshot = Vec::new();
    let mut after_snapshot = Vec::new();
    let mut affected_lines = Vec::new();

    for i in 0..count as u64 {
        let target_line = target_start + i;
        let source_line = source_start + (i as usize % source_count) as u64;

        let target_content = get_line_content(document, target_line);
        let source_content = get_line_content(document, source_line);

        before_snapshot.push(LineSnapshot::new(target_line, target_content.clone()));

        let merged = merge_overlay(&source_content, &target_content);

        // Replace the target line content: delete existing content, insert merged
        let line_start = document.line_start(LineNumber(target_line));
        let line_end = document.line_end(LineNumber(target_line));
        let content_len = u64::from(line_end) - u64::from(line_start);

        if content_len > 0 {
            document.delete(line_start, content_len).map_err(|e| {
                LineCommandError::DocumentError {
                    operation: "overlay".to_string(),
                    description: e.to_string(),
                }
            })?;
        }
        if !merged.is_empty() {
            document
                .insert(line_start, merged.as_bytes())
                .map_err(|e| LineCommandError::DocumentError {
                    operation: "overlay".to_string(),
                    description: e.to_string(),
                })?;
        }

        after_snapshot.push(LineSnapshot::new(target_line, merged));
        affected_lines.push(target_line);
    }

    Ok(EditorTransaction::new(
        affected_lines,
        before_snapshot,
        after_snapshot,
        format!(
            "Overlay lines {}-{} onto lines {}-{}",
            source_start,
            source_end,
            target_start,
            target_start + count as u64 - 1
        ),
    ))
}

/// Merge source over target: non-blank source chars replace target chars.
fn merge_overlay(source: &str, target: &str) -> String {
    let src_chars: Vec<char> = source.chars().collect();
    let tgt_chars: Vec<char> = target.chars().collect();
    let max_len = src_chars.len().max(tgt_chars.len());
    let mut result = String::with_capacity(max_len);

    for i in 0..max_len {
        let src_ch = src_chars.get(i).copied().unwrap_or(' ');
        let tgt_ch = tgt_chars.get(i).copied().unwrap_or(' ');
        if src_ch != ' ' {
            result.push(src_ch);
        } else {
            result.push(tgt_ch);
        }
    }

    // Trim trailing spaces to match document conventions
    let trimmed = result.trim_end_matches(' ');
    trimmed.to_string()
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
    fn merge_overlay_non_blank_source_replaces_target() {
        // Validates: Requirement 15.1
        let result = merge_overlay("ABC", "xyz");
        assert_eq!(result, "ABC");
    }

    #[test]
    fn merge_overlay_blank_source_preserves_target() {
        // Validates: Requirement 15.1 -- blank source chars leave target unchanged
        // source "A C": A=non-blank, ' '=blank, C=non-blank
        // target "xyz": x, y, z
        // result: A replaces x, ' ' preserves y, C replaces z -> "AyC"
        let result = merge_overlay("A C", "xyz");
        assert_eq!(result, "AyC");
    }

    #[test]
    fn merge_overlay_source_shorter_preserves_tail() {
        // Validates: Requirement 15.1
        let result = merge_overlay("AB", "xyzw");
        assert_eq!(result, "ABzw");
    }

    #[test]
    fn execute_overlay_single_line_replaces_non_blank() {
        // Validates: Requirement 15.1, 15.10
        let mut doc = make_document(&["hello world", "ABC   GHI"]);
        let txn = execute_overlay(&mut doc, 1, 1, 0, 1).unwrap();
        let result = get_line_content(&doc, 0);
        // source "ABC   GHI", target "hello world"
        // A->h, B->e, C->l, ' '->l, ' '->o, ' '->' ', G->w, H->o, I->r, ' '->l, ' '->d
        assert_eq!(result, "ABClo GHIld");
        assert!(!txn.description.is_empty());
    }

    #[test]
    fn execute_overlay_counted_applies_to_n_lines() {
        // Validates: Requirement 15.2, 15.10
        let mut doc = make_document(&["aaaa", "bbbb", "cccc", "XXXX"]);
        execute_overlay(&mut doc, 3, 3, 0, 3).unwrap();
        // source line 3 = "XXXX" overlaid onto lines 0, 1, 2
        assert_eq!(get_line_content(&doc, 0), "XXXX");
        assert_eq!(get_line_content(&doc, 1), "XXXX");
        assert_eq!(get_line_content(&doc, 2), "XXXX");
    }

    #[test]
    fn execute_overlay_out_of_range_returns_error() {
        let mut doc = make_document(&["a", "b"]);
        let result = execute_overlay(&mut doc, 0, 0, 5, 1);
        assert!(matches!(
            result,
            Err(LineCommandError::LineOutOfRange { .. })
        ));
    }

    #[test]
    fn execute_overlay_produces_transaction() {
        // Validates: Requirement 15.10
        let mut doc = make_document(&["hello", "WORLD"]);
        let txn = execute_overlay(&mut doc, 1, 1, 0, 1).unwrap();
        assert!(!txn.before_snapshot.is_empty());
    }
}
