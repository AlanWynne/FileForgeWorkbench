//! Copy line command execution (C, CC + A/B).

use ff_document_model::{Document, LineNumber};
use ff_edit_operations::{EditorTransaction, LineSnapshot};

use crate::command::{SourceTarget, TargetPosition};
use crate::error::LineCommandError;
use crate::execution::delete::get_line_content;

/// Execute a copy-to-target operation.
///
/// Copies source lines to the target position without modifying the source.
pub fn execute_copy(
    document: &mut Document,
    source_target: &SourceTarget,
) -> Result<EditorTransaction, LineCommandError> {
    let total_lines = document.line_count();

    // Validate source range
    if source_target.source_end >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "copy".to_string(),
            line: source_target.source_end,
            total: total_lines,
        });
    }

    // Validate target line
    if source_target.target_line >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "copy".to_string(),
            line: source_target.target_line,
            total: total_lines,
        });
    }

    // Collect source content
    let mut source_content = Vec::new();
    for i in source_target.source_start..=source_target.source_end {
        source_content.push(get_line_content(document, i));
    }

    // Determine insertion point
    let insert_pos = match source_target.target_position {
        TargetPosition::After => {
            if source_target.target_line + 1 < total_lines {
                document.line_start(LineNumber(source_target.target_line + 1))
            } else {
                document.line_end(LineNumber(source_target.target_line))
            }
        }
        TargetPosition::Before => document.line_start(LineNumber(source_target.target_line)),
    };

    // Build insert text
    let mut insert_text = String::new();
    let is_after_last = source_target.target_position == TargetPosition::After
        && source_target.target_line + 1 >= total_lines;

    for (idx, content) in source_content.iter().enumerate() {
        if idx > 0 || is_after_last {
            insert_text.push('\n');
        }
        insert_text.push_str(content);
        if idx < source_content.len() - 1 && !is_after_last {
            // Not needed — already handling via separating newlines
        }
        if !is_after_last && idx == source_content.len() - 1 {
            insert_text.push('\n');
        }
    }

    // Simplify: join with newlines and handle position correctly
    let joined = source_content.join("\n");
    let insert_text = if is_after_last {
        format!("\n{}", joined)
    } else {
        format!("{}\n", joined)
    };

    document
        .insert(insert_pos, insert_text.as_bytes())
        .map_err(|e| LineCommandError::DocumentError {
            operation: "copy".to_string(),
            description: e.to_string(),
        })?;

    let copy_count = source_target.source_end - source_target.source_start + 1;
    let insert_start = match source_target.target_position {
        TargetPosition::After => source_target.target_line + 1,
        TargetPosition::Before => source_target.target_line,
    };

    let mut after_snapshot = Vec::new();
    let mut affected_lines = Vec::new();
    for (idx, content) in source_content.iter().enumerate() {
        let new_line = insert_start + idx as u64;
        after_snapshot.push(LineSnapshot::new(new_line, content.clone()));
        affected_lines.push(new_line);
    }

    Ok(EditorTransaction::new(
        affected_lines,
        vec![],
        after_snapshot,
        format!(
            "Copy {} line(s) from {}-{} to line {}",
            copy_count,
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
    fn copy_single_line_after_target() {
        let mut doc = make_document(&["a", "b", "c", "d"]);
        let st = SourceTarget {
            operation: SourceOperation::Copy,
            source_start: 1,
            source_end: 1,
            target_line: 3,
            target_position: TargetPosition::After,
        };
        execute_copy(&mut doc, &st).unwrap();
        assert_eq!(doc.line_count(), 5);
        // Source unchanged
        assert_eq!(get_line_content(&doc, 1), "b");
    }

    #[test]
    fn copy_block_before_target() {
        let mut doc = make_document(&["a", "b", "c", "d", "e"]);
        let st = SourceTarget {
            operation: SourceOperation::Copy,
            source_start: 1,
            source_end: 2,
            target_line: 0,
            target_position: TargetPosition::Before,
        };
        execute_copy(&mut doc, &st).unwrap();
        assert_eq!(doc.line_count(), 7);
        assert_eq!(get_line_content(&doc, 0), "b");
        assert_eq!(get_line_content(&doc, 1), "c");
        assert_eq!(get_line_content(&doc, 2), "a");
    }

    #[test]
    fn copy_does_not_modify_source_content() {
        let mut doc = make_document(&["hello", "world", "target"]);
        let original_source = get_line_content(&doc, 0);
        let st = SourceTarget {
            operation: SourceOperation::Copy,
            source_start: 0,
            source_end: 0,
            target_line: 2,
            target_position: TargetPosition::After,
        };
        execute_copy(&mut doc, &st).unwrap();
        assert_eq!(get_line_content(&doc, 0), original_source);
    }
}
