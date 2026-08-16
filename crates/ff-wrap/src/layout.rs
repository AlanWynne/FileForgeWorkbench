//! Line-breaking and sub-line height computation.
//!
//! Provides the core algorithm for computing where line breaks occur
//! and how many display sub-lines a wrapped document line occupies.

use crate::indent::WrapIndentMode;
use crate::mode::WrapMode;

/// Compute the character offsets where line breaks occur for word wrapping.
///
/// Breaks at word boundaries (whitespace) where possible. Falls back to
/// character-level breaking when a single word exceeds `wrap_width`.
///
/// Addresses: Requirement 1 AC 3, AC 4
pub fn compute_word_breaks(line: &str, wrap_width: u32, indent_offset: usize) -> Vec<usize> {
    if wrap_width == 0 {
        return Vec::new();
    }

    let chars: Vec<char> = line.chars().collect();
    let total_chars = chars.len();

    if total_chars == 0 {
        return Vec::new();
    }

    let first_line_width = wrap_width as usize;
    // Continuation lines have reduced width due to indent
    let cont_width = if wrap_width as usize > indent_offset {
        wrap_width as usize - indent_offset
    } else {
        1 // Minimum 1 character per line to avoid infinite loops
    };

    let mut breaks = Vec::new();
    let mut pos = 0;
    let mut is_first_line = true;

    while pos < total_chars {
        let available = if is_first_line {
            first_line_width
        } else {
            cont_width
        };

        if pos + available >= total_chars {
            break; // Remaining content fits
        }

        // Try to find a word boundary (whitespace) within the available width
        let end = pos + available;
        let mut break_pos = None;

        // Look backward from the end for the last whitespace
        for i in (pos..end).rev() {
            if chars[i].is_whitespace() {
                break_pos = Some(i + 1); // Break after the whitespace
                break;
            }
        }

        match break_pos {
            Some(bp) if bp > pos => {
                breaks.push(bp);
                pos = bp;
            }
            _ => {
                // No word boundary found — force break at character position
                breaks.push(end);
                pos = end;
            }
        }

        is_first_line = false;
    }

    breaks
}

/// Compute the character offsets where line breaks occur for character wrapping.
///
/// Breaks at exact character positions without regard to word boundaries.
///
/// Addresses: Requirement 1 AC 5
pub fn compute_char_breaks(line: &str, wrap_width: u32, indent_offset: usize) -> Vec<usize> {
    if wrap_width == 0 {
        return Vec::new();
    }

    let total_chars = line.chars().count();

    if total_chars == 0 {
        return Vec::new();
    }

    let first_line_width = wrap_width as usize;
    let cont_width = if wrap_width as usize > indent_offset {
        wrap_width as usize - indent_offset
    } else {
        1
    };

    let mut breaks = Vec::new();
    let mut pos = 0;
    let mut is_first_line = true;

    while pos < total_chars {
        let available = if is_first_line {
            first_line_width
        } else {
            cont_width
        };

        if pos + available >= total_chars {
            break;
        }

        breaks.push(pos + available);
        pos += available;
        is_first_line = false;
    }

    breaks
}

/// Compute line break positions based on the wrap mode.
///
/// Returns character offsets where the line should be broken.
/// Returns an empty vec for `WrapMode::None` or lines that fit.
pub fn compute_breaks(
    line: &str,
    wrap_width: u32,
    mode: WrapMode,
    indent_offset: usize,
) -> Vec<usize> {
    match mode {
        WrapMode::None => Vec::new(),
        WrapMode::Word => compute_word_breaks(line, wrap_width, indent_offset),
        WrapMode::Character => compute_char_breaks(line, wrap_width, indent_offset),
    }
}

/// Compute the number of display sub-lines a line occupies.
///
/// Returns 1 when `mode` is `None` or the line fits within `wrap_width`.
/// Returns `breaks.len() + 1` otherwise.
///
/// Addresses: Requirement 6 AC 1
pub fn compute_sub_line_count(
    line: &str,
    wrap_width: u32,
    mode: WrapMode,
    indent_offset: usize,
) -> u32 {
    if mode == WrapMode::None || wrap_width == 0 {
        return 1;
    }

    let breaks = compute_breaks(line, wrap_width, mode, indent_offset);
    (breaks.len() as u32) + 1
}

/// Compute the display height for a line given its character count.
///
/// This is a simplified version that works with character counts rather than
/// the actual line content. Used when only the width is known.
///
/// Returns 1 when mode is `None`. Otherwise computes ceiling division.
pub fn compute_height_from_width(
    line_width: usize,
    wrap_width: u16,
    mode: WrapMode,
    indent_offset: usize,
) -> u32 {
    if mode == WrapMode::None || wrap_width == 0 {
        return 1;
    }

    if line_width == 0 {
        return 1;
    }

    let first_line_width = wrap_width as usize;
    if line_width <= first_line_width {
        return 1;
    }

    let cont_width = if (wrap_width as usize) > indent_offset {
        wrap_width as usize - indent_offset
    } else {
        1
    };

    // First line takes first_line_width chars, then each continuation takes cont_width
    let remaining = line_width - first_line_width;
    let additional_lines = remaining.div_ceil(cont_width);
    1 + additional_lines as u32
}

/// Information about sub-line content for rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubLineInfo {
    /// Start character offset within the document line.
    pub start_offset: usize,
    /// End character offset within the document line (exclusive).
    pub end_offset: usize,
    /// Indent offset in characters for this sub-line (0 for first sub-line).
    pub indent_chars: usize,
    /// Whether this is a continuation line (not the first sub-line).
    pub is_continuation: bool,
}

/// Compute full sub-line layout information for rendering a wrapped line.
///
/// Addresses: Requirement 13 AC 1
pub fn compute_sub_lines(
    line: &str,
    wrap_width: u32,
    mode: WrapMode,
    indent_offset: usize,
) -> Vec<SubLineInfo> {
    let total_chars = line.chars().count();

    if mode == WrapMode::None || wrap_width == 0 || total_chars == 0 {
        return vec![SubLineInfo {
            start_offset: 0,
            end_offset: total_chars,
            indent_chars: 0,
            is_continuation: false,
        }];
    }

    let breaks = compute_breaks(line, wrap_width, mode, indent_offset);

    if breaks.is_empty() {
        return vec![SubLineInfo {
            start_offset: 0,
            end_offset: total_chars,
            indent_chars: 0,
            is_continuation: false,
        }];
    }

    let mut sub_lines = Vec::with_capacity(breaks.len() + 1);

    // First sub-line
    sub_lines.push(SubLineInfo {
        start_offset: 0,
        end_offset: breaks[0],
        indent_chars: 0,
        is_continuation: false,
    });

    // Middle sub-lines
    for i in 0..breaks.len() - 1 {
        sub_lines.push(SubLineInfo {
            start_offset: breaks[i],
            end_offset: breaks[i + 1],
            indent_chars: indent_offset,
            is_continuation: true,
        });
    }

    // Last sub-line
    sub_lines.push(SubLineInfo {
        start_offset: *breaks.last().unwrap(),
        end_offset: total_chars,
        indent_chars: indent_offset,
        is_continuation: true,
    });

    sub_lines
}

/// Determine the column of the first non-whitespace character in a line.
///
/// Used by indent modes (`Same`, `Indent`, `DeepIndent`) to align continuation lines.
pub fn first_non_whitespace_col(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace()).count()
}

/// Compute the full indent offset for continuation lines based on the indent mode.
///
/// Combines `WrapIndentMode` logic with line content analysis.
pub fn resolve_indent_offset(
    mode: WrapIndentMode,
    indent_amount: u8,
    line: &str,
    indent_width: u8,
) -> usize {
    let first_non_ws = first_non_whitespace_col(line);
    mode.compute_indent(indent_amount, first_non_ws, indent_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Word wrap tests ---

    #[test]
    fn word_wrap_short_line_no_breaks() {
        // Validates: Requirement 1.3
        let breaks = compute_word_breaks("hello world", 20, 0);
        assert!(breaks.is_empty());
    }

    #[test]
    fn word_wrap_breaks_at_word_boundary() {
        // Validates: Requirement 1.3
        let line = "hello world foo bar";
        let breaks = compute_word_breaks(line, 10, 0);
        // "hello " fits in 10, break after whitespace at position 6
        assert!(!breaks.is_empty());
        // Each segment should not exceed wrap_width
        let mut prev = 0;
        for &b in &breaks {
            assert!(
                b - prev <= 10,
                "segment from {} to {} exceeds width",
                prev,
                b
            );
            prev = b;
        }
    }

    #[test]
    fn word_wrap_long_word_falls_back_to_char_break() {
        // Validates: Requirement 1.4
        let line = "abcdefghijklmnopqrstuvwxyz"; // 26 chars, no whitespace
        let breaks = compute_word_breaks(line, 10, 0);
        // Should break at character positions since no whitespace exists
        assert!(!breaks.is_empty());
        assert_eq!(breaks[0], 10);
    }

    // --- Character wrap tests ---

    #[test]
    fn char_wrap_short_line_no_breaks() {
        // Validates: Requirement 1.5
        let breaks = compute_char_breaks("hello", 10, 0);
        assert!(breaks.is_empty());
    }

    #[test]
    fn char_wrap_breaks_at_exact_position() {
        // Validates: Requirement 1.5
        let breaks = compute_char_breaks("abcdefghij12345", 10, 0);
        assert_eq!(breaks, vec![10]);
    }

    #[test]
    fn char_wrap_multiple_breaks() {
        let line = "a".repeat(25);
        let breaks = compute_char_breaks(&line, 10, 0);
        assert_eq!(breaks, vec![10, 20]);
    }

    // --- Sub-line count tests ---

    #[test]
    fn sub_line_count_none_mode_always_one() {
        // Validates: Requirement 1.2, 6.2
        assert_eq!(
            compute_sub_line_count("a".repeat(100).as_str(), 10, WrapMode::None, 0),
            1
        );
    }

    #[test]
    fn sub_line_count_short_line_is_one() {
        assert_eq!(compute_sub_line_count("hello", 10, WrapMode::Word, 0), 1);
    }

    #[test]
    fn sub_line_count_long_line_greater_than_one() {
        // Validates: Requirement 6.1
        let line = "a".repeat(25);
        let count = compute_sub_line_count(&line, 10, WrapMode::Character, 0);
        assert_eq!(count, 3); // 10 + 10 + 5
    }

    #[test]
    fn sub_line_count_empty_line_is_one() {
        assert_eq!(compute_sub_line_count("", 10, WrapMode::Word, 0), 1);
    }

    // --- Height from width tests ---

    #[test]
    fn height_from_width_none_mode_always_one() {
        assert_eq!(compute_height_from_width(100, 10, WrapMode::None, 0), 1);
    }

    #[test]
    fn height_from_width_short_is_one() {
        assert_eq!(compute_height_from_width(5, 10, WrapMode::Word, 0), 1);
    }

    #[test]
    fn height_from_width_exact_boundary_is_one() {
        assert_eq!(compute_height_from_width(10, 10, WrapMode::Word, 0), 1);
    }

    #[test]
    fn height_from_width_one_over_is_two() {
        assert_eq!(compute_height_from_width(11, 10, WrapMode::Word, 0), 2);
    }

    #[test]
    fn height_from_width_with_indent_offset() {
        // 10 char first line, then continuation lines have width 10 - 4 = 6
        // 25 chars: first line takes 10, remaining 15 in chunks of 6 → ceil(15/6) = 3
        // Total: 1 + 3 = 4
        assert_eq!(compute_height_from_width(25, 10, WrapMode::Character, 4), 4);
    }

    // --- Sub-line info tests ---

    #[test]
    fn sub_lines_none_mode_single_entry() {
        let sub_lines = compute_sub_lines("hello world", 10, WrapMode::None, 0);
        assert_eq!(sub_lines.len(), 1);
        assert!(!sub_lines[0].is_continuation);
    }

    #[test]
    fn sub_lines_first_is_not_continuation() {
        // Validates: Requirement 13.2
        let sub_lines = compute_sub_lines("hello world foo bar baz", 10, WrapMode::Character, 0);
        assert!(!sub_lines[0].is_continuation);
        assert!(sub_lines[1].is_continuation);
    }

    #[test]
    fn sub_lines_reconstruct_original() {
        // Validates: content preservation
        let line = "hello world foo bar baz";
        let sub_lines = compute_sub_lines(line, 10, WrapMode::Character, 0);
        let chars: Vec<char> = line.chars().collect();
        let mut reconstructed = String::new();
        for sl in &sub_lines {
            for &c in &chars[sl.start_offset..sl.end_offset] {
                reconstructed.push(c);
            }
        }
        assert_eq!(reconstructed, line);
    }

    // --- Indent resolution ---

    #[test]
    fn first_non_whitespace_col_no_indent() {
        assert_eq!(first_non_whitespace_col("hello"), 0);
    }

    #[test]
    fn first_non_whitespace_col_with_spaces() {
        assert_eq!(first_non_whitespace_col("    hello"), 4);
    }

    #[test]
    fn resolve_indent_offset_fixed() {
        // Validates: Requirement 5.2
        let offset = resolve_indent_offset(WrapIndentMode::Fixed, 4, "    hello", 4);
        assert_eq!(offset, 4);
    }

    #[test]
    fn resolve_indent_offset_same() {
        // Validates: Requirement 5.3
        let offset = resolve_indent_offset(WrapIndentMode::Same, 0, "    hello", 4);
        assert_eq!(offset, 4);
    }

    #[test]
    fn resolve_indent_offset_indent_mode() {
        // Validates: Requirement 5.4
        let offset = resolve_indent_offset(WrapIndentMode::Indent, 0, "    hello", 4);
        assert_eq!(offset, 8);
    }

    #[test]
    fn resolve_indent_offset_deep_indent() {
        // Validates: Requirement 5.5
        let offset = resolve_indent_offset(WrapIndentMode::DeepIndent, 0, "    hello", 4);
        assert_eq!(offset, 12);
    }
}
