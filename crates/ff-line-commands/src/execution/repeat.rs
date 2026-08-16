//! Repeat line command execution (R, Rn, RR).

use ff_document_model::{Document, LineNumber};
use ff_edit_operations::{EditorTransaction, LineSnapshot};

use crate::error::LineCommandError;
use crate::execution::delete::get_line_content;

/// Execute a single-line repeat — duplicate `line` content `count` times immediately after it.
pub fn execute_repeat(
    document: &mut Document,
    line: u64,
    count: u32,
) -> Result<EditorTransaction, LineCommandError> {
    let total_lines = document.line_count();

    if line >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "repeat".to_string(),
            line,
            total: total_lines,
        });
    }

    let content = get_line_content(document, line);

    // Build insert text: count copies of the line
    let mut insert_text = String::new();
    let total_lines = document.line_count();
    let is_last_line = line + 1 >= total_lines;

    for i in 0..count {
        if is_last_line || i > 0 || !is_last_line {
            // For inserting before the next line, we prepend content+\n
            insert_text.push_str(&content);
            insert_text.push('\n');
        }
    }

    // Insert position: at the start of the next line (or end of doc for last line)
    let insert_pos = if !is_last_line {
        document.line_start(LineNumber(line + 1))
    } else {
        document.line_end(LineNumber(line))
    };

    // For last line, format is \ncontent repeated
    let final_text = if is_last_line {
        let mut t = String::new();
        for _ in 0..count {
            t.push('\n');
            t.push_str(&content);
        }
        t
    } else {
        insert_text
    };

    document
        .insert(insert_pos, final_text.as_bytes())
        .map_err(|e| LineCommandError::DocumentError {
            operation: "repeat".to_string(),
            description: e.to_string(),
        })?;

    // Build transaction
    let mut after_snapshot = Vec::new();
    let mut affected_lines = Vec::new();
    for i in 0..u64::from(count) {
        let new_line = line + 1 + i;
        after_snapshot.push(LineSnapshot::new(new_line, content.clone()));
        affected_lines.push(new_line);
    }

    Ok(EditorTransaction::new(
        affected_lines,
        vec![],
        after_snapshot,
        format!("Repeat line {} {} time(s)", line, count),
    ))
}

/// Execute a block repeat — duplicate lines [start_line, end_line] and insert after end_line.
pub fn execute_repeat_block(
    document: &mut Document,
    start_line: u64,
    end_line: u64,
) -> Result<EditorTransaction, LineCommandError> {
    let total_lines = document.line_count();

    if end_line >= total_lines {
        return Err(LineCommandError::LineOutOfRange {
            operation: "repeat".to_string(),
            line: end_line,
            total: total_lines,
        });
    }

    // Collect content of the block
    let mut block_content = Vec::new();
    for i in start_line..=end_line {
        block_content.push(get_line_content(document, i));
    }

    // Build insert text
    let is_last_line = end_line + 1 >= total_lines;
    let insert_text = if is_last_line {
        let mut t = String::new();
        for line_content in &block_content {
            t.push('\n');
            t.push_str(line_content);
        }
        t
    } else {
        let mut t = String::new();
        for line_content in &block_content {
            t.push_str(line_content);
            t.push('\n');
        }
        t
    };

    // Insert after end_line
    let insert_pos = if !is_last_line {
        document.line_start(LineNumber(end_line + 1))
    } else {
        document.line_end(LineNumber(end_line))
    };

    document
        .insert(insert_pos, insert_text.as_bytes())
        .map_err(|e| LineCommandError::DocumentError {
            operation: "repeat".to_string(),
            description: e.to_string(),
        })?;

    let block_size = end_line - start_line + 1;
    let mut after_snapshot = Vec::new();
    let mut affected_lines = Vec::new();
    for (idx, content) in block_content.iter().enumerate() {
        let new_line = end_line + 1 + idx as u64;
        after_snapshot.push(LineSnapshot::new(new_line, content.clone()));
        affected_lines.push(new_line);
    }

    Ok(EditorTransaction::new(
        affected_lines,
        vec![],
        after_snapshot,
        format!(
            "Repeat block lines {}-{} ({} lines)",
            start_line, end_line, block_size
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
    fn repeat_single_line_once() {
        let mut doc = make_document(&["hello", "world"]);
        execute_repeat(&mut doc, 0, 1).unwrap();
        assert_eq!(doc.line_count(), 3);
        assert_eq!(get_line_content(&doc, 0), "hello");
        assert_eq!(get_line_content(&doc, 1), "hello");
        assert_eq!(get_line_content(&doc, 2), "world");
    }

    #[test]
    fn repeat_single_line_multiple_times() {
        let mut doc = make_document(&["abc", "def"]);
        execute_repeat(&mut doc, 0, 3).unwrap();
        assert_eq!(doc.line_count(), 5);
        assert_eq!(get_line_content(&doc, 1), "abc");
        assert_eq!(get_line_content(&doc, 2), "abc");
        assert_eq!(get_line_content(&doc, 3), "abc");
    }

    #[test]
    fn repeat_out_of_range_returns_error() {
        let mut doc = make_document(&["a"]);
        let result = execute_repeat(&mut doc, 5, 1);
        assert!(matches!(
            result,
            Err(LineCommandError::LineOutOfRange { .. })
        ));
    }

    #[test]
    fn repeat_block_duplicates_range() {
        let mut doc = make_document(&["a", "b", "c", "d"]);
        execute_repeat_block(&mut doc, 1, 2).unwrap();
        assert_eq!(doc.line_count(), 6);
        assert_eq!(get_line_content(&doc, 0), "a");
        assert_eq!(get_line_content(&doc, 1), "b");
        assert_eq!(get_line_content(&doc, 2), "c");
        assert_eq!(get_line_content(&doc, 3), "b");
        assert_eq!(get_line_content(&doc, 4), "c");
        assert_eq!(get_line_content(&doc, 5), "d");
    }
}
