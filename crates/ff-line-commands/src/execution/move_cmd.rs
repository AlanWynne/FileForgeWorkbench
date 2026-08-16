//! Move line command execution (M, MM + A/B).

use ff_document_model::{Document, LineNumber};
use ff_edit_operations::{EditorTransaction, LineSnapshot};

use crate::command::{SourceTarget, TargetPosition};
use crate::error::LineCommandError;
use crate::execution::delete::get_line_content;

/// Execute a move-to-target operation.
///
/// Removes source lines and inserts them at the target position.
pub fn execute_move(
    document: &mut Document,
    source_target: &SourceTarget,
) -> Result<EditorTransaction, LineCommandError> {
    let total_lines = document.line_count();

    // Validate source range
    if source_target.source_end >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "move".to_string(),
            line: source_target.source_end,
            total: total_lines,
        });
    }

    // Validate target line
    if source_target.target_line >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "move".to_string(),
            line: source_target.target_line,
            total: total_lines,
        });
    }

    // Validate target is not inside source block
    if source_target.target_line >= source_target.source_start
        && source_target.target_line <= source_target.source_end
    {
        return Err(LineCommandError::TargetInsideSource);
    }

    // Collect source content before modification
    let mut source_content = Vec::new();
    for i in source_target.source_start..=source_target.source_end {
        source_content.push(get_line_content(document, i));
    }

    let source_count = source_target.source_end - source_target.source_start + 1;

    // Build before snapshot
    let mut before_snapshot = Vec::new();
    let mut affected_lines = Vec::new();
    for i in source_target.source_start..=source_target.source_end {
        before_snapshot.push(LineSnapshot::new(
            i,
            source_content[(i - source_target.source_start) as usize].clone(),
        ));
        affected_lines.push(i);
    }

    // Strategy: delete source lines, then insert at adjusted target
    // We need to adjust the target line if it's after the source
    let adjusted_target = if source_target.target_line > source_target.source_end {
        source_target.target_line - source_count
    } else {
        source_target.target_line
    };

    // Delete source lines (from last to first)
    for i in (source_target.source_start..=source_target.source_end).rev() {
        let total = document.line_count();
        if i + 1 < total {
            // Not the last line — delete from line_start(i) to line_start(i+1)
            let start = document.line_start(LineNumber(i));
            let next_start = document.line_start(LineNumber(i + 1));
            let delete_len = u64::from(next_start) - u64::from(start);
            if delete_len > 0 {
                document.delete(start, delete_len).map_err(|e| {
                    LineCommandError::DocumentError {
                        operation: "move".to_string(),
                        description: e.to_string(),
                    }
                })?;
            }
        } else if i > 0 {
            // Last line (not only line) — include preceding newline
            let prev_end = document.line_end(LineNumber(i - 1));
            let doc_length = ff_document_model::BytePosition(document.length());
            let delete_len = u64::from(doc_length) - u64::from(prev_end);
            if delete_len > 0 {
                document.delete(prev_end, delete_len).map_err(|e| {
                    LineCommandError::DocumentError {
                        operation: "move".to_string(),
                        description: e.to_string(),
                    }
                })?;
            }
        } else {
            // Only line — delete all
            let length = document.length();
            if length > 0 {
                document
                    .delete(ff_document_model::BytePosition::ZERO, length)
                    .map_err(|e| LineCommandError::DocumentError {
                        operation: "move".to_string(),
                        description: e.to_string(),
                    })?;
            }
        }
    }

    // Now insert at the adjusted target position
    let current_total = document.line_count();
    let clamped_target = adjusted_target.min(current_total.saturating_sub(1));

    let insert_pos = match source_target.target_position {
        TargetPosition::After => {
            if clamped_target + 1 < current_total {
                document.line_start(LineNumber(clamped_target + 1))
            } else {
                document.line_end(LineNumber(clamped_target))
            }
        }
        TargetPosition::Before => {
            if clamped_target < current_total {
                document.line_start(LineNumber(clamped_target))
            } else {
                document.line_end(LineNumber(current_total.saturating_sub(1)))
            }
        }
    };

    let joined = source_content.join("\n");
    let is_after_last = source_target.target_position == TargetPosition::After
        && clamped_target + 1 >= current_total;

    // When inserting Before a line, the existing line already has no preceding
    // separator at position 0, or the newline belongs to the previous line.
    // We insert "content\n" to push the existing content down, but only if
    // it's not the last position.
    let insert_text = if is_after_last {
        // Inserting after the last line: need a leading newline
        format!("\n{}", joined)
    } else if source_target.target_position == TargetPosition::Before {
        // Inserting before a line: content + newline separates from existing
        format!("{}\n", joined)
    } else {
        // Inserting after a non-last line: content + newline
        format!("{}\n", joined)
    };

    document
        .insert(insert_pos, insert_text.as_bytes())
        .map_err(|e| LineCommandError::DocumentError {
            operation: "move".to_string(),
            description: e.to_string(),
        })?;

    // Build after snapshot
    let insert_start_line = match source_target.target_position {
        TargetPosition::After => clamped_target + 1,
        TargetPosition::Before => clamped_target,
    };

    let mut after_snapshot = Vec::new();
    for (idx, content) in source_content.iter().enumerate() {
        let new_line = insert_start_line + idx as u64;
        after_snapshot.push(LineSnapshot::new(new_line, content.clone()));
        if !affected_lines.contains(&new_line) {
            affected_lines.push(new_line);
        }
    }

    Ok(EditorTransaction::new(
        affected_lines,
        before_snapshot,
        after_snapshot,
        format!(
            "Move {} line(s) from {}-{} to line {}",
            source_count,
            source_target.source_start,
            source_target.source_end,
            source_target.target_line
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::SourceOperation;

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
    fn move_single_line_after_target() {
        let mut doc = make_document(&["a", "b", "c", "d"]);
        let st = SourceTarget {
            operation: SourceOperation::Move,
            source_start: 1,
            source_end: 1,
            target_line: 3,
            target_position: TargetPosition::After,
        };
        execute_move(&mut doc, &st).unwrap();
        // Line count preserved
        assert_eq!(doc.line_count(), 4);
        assert_eq!(get_line_content(&doc, 0), "a");
        assert_eq!(get_line_content(&doc, 1), "c");
        assert_eq!(get_line_content(&doc, 2), "d");
        assert_eq!(get_line_content(&doc, 3), "b");
    }

    #[test]
    fn move_block_before_target() {
        let mut doc = make_document(&["a", "b", "c", "d", "e"]);
        assert_eq!(doc.line_count(), 5);

        // Manually trace: delete lines 3 and 4 (d, e)
        // After: should be "a\nb\nc"
        // Then insert "d\ne" before line 0

        let st = SourceTarget {
            operation: SourceOperation::Move,
            source_start: 3,
            source_end: 4,
            target_line: 0,
            target_position: TargetPosition::Before,
        };
        execute_move(&mut doc, &st).unwrap();
        assert_eq!(
            doc.line_count(),
            5,
            "line count should be 5, got {} with content: {:?}, length={}",
            doc.line_count(),
            (0..doc.line_count())
                .map(|i| get_line_content(&doc, i))
                .collect::<Vec<_>>(),
            doc.length()
        );
        assert_eq!(get_line_content(&doc, 0), "d");
        assert_eq!(get_line_content(&doc, 1), "e");
        assert_eq!(get_line_content(&doc, 2), "a");
    }

    #[test]
    fn move_target_inside_source_returns_error() {
        let mut doc = make_document(&["a", "b", "c", "d"]);
        let st = SourceTarget {
            operation: SourceOperation::Move,
            source_start: 1,
            source_end: 3,
            target_line: 2,
            target_position: TargetPosition::After,
        };
        let result = execute_move(&mut doc, &st);
        assert!(matches!(result, Err(LineCommandError::TargetInsideSource)));
    }

    #[test]
    fn move_preserves_line_count() {
        let mut doc = make_document(&["a", "b", "c", "d", "e"]);
        let before_count = doc.line_count();
        let st = SourceTarget {
            operation: SourceOperation::Move,
            source_start: 0,
            source_end: 1,
            target_line: 4,
            target_position: TargetPosition::After,
        };
        execute_move(&mut doc, &st).unwrap();
        assert_eq!(doc.line_count(), before_count);
    }
}
