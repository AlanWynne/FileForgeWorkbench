//! Indent guide column computation.

use crate::indent::level::indent_level_of;

/// Compute guide columns for a line using `Real` mode.
///
/// Returns guide columns at each tab-stop within the line's leading whitespace.
///
/// Addresses: Requirement 3 AC 3.3
pub fn compute_real_guides(line: &[u8], tab_size: u32) -> Vec<u32> {
    let tab_size = tab_size.max(1);
    let indent = indent_level_of(line, tab_size);

    let mut guides = Vec::new();
    let mut col = tab_size;
    while col <= indent {
        guides.push(col);
        col += tab_size;
    }
    guides
}

/// Compute guide columns using `LookForward` mode.
///
/// Scans forward through blank/short-indent lines to determine the effective
/// indent level for guide rendering.
///
/// Addresses: Requirement 3 AC 3.4
pub fn compute_look_forward_guides(lines: &[&[u8]], line_index: usize, tab_size: u32) -> Vec<u32> {
    let tab_size = tab_size.max(1);

    let own_indent = indent_level_of(lines[line_index], tab_size);
    let next_indent = scan_forward(lines, line_index, tab_size);

    let effective_indent = match next_indent {
        Some(next) => own_indent.max(next),
        None => own_indent,
    };

    let mut guides = Vec::new();
    let mut col = tab_size;
    while col <= effective_indent {
        guides.push(col);
        col += tab_size;
    }
    guides
}

/// Compute guide columns using `LookBoth` mode.
///
/// Scans both forward and backward, using the maximum indent level from
/// surrounding non-blank lines.
///
/// Addresses: Requirement 3 AC 3.5
pub fn compute_look_both_guides(lines: &[&[u8]], line_index: usize, tab_size: u32) -> Vec<u32> {
    let tab_size = tab_size.max(1);

    let own_indent = indent_level_of(lines[line_index], tab_size);
    let next_indent = scan_forward(lines, line_index, tab_size);
    let prev_indent = scan_backward(lines, line_index, tab_size);

    let effective_indent = own_indent
        .max(next_indent.unwrap_or(0))
        .max(prev_indent.unwrap_or(0));

    let mut guides = Vec::new();
    let mut col = tab_size;
    while col <= effective_indent {
        guides.push(col);
        col += tab_size;
    }
    guides
}

/// Scan forward from `start_line + 1` to find the next non-blank line's indent level.
fn scan_forward(lines: &[&[u8]], start_line: usize, tab_size: u32) -> Option<u32> {
    lines[(start_line + 1)..]
        .iter()
        .find(|line| !is_blank(line))
        .map(|line| indent_level_of(line, tab_size))
}

/// Scan backward from `start_line - 1` to find the previous non-blank line's indent level.
fn scan_backward(lines: &[&[u8]], start_line: usize, tab_size: u32) -> Option<u32> {
    if start_line == 0 {
        return None;
    }
    for i in (0..start_line).rev() {
        if !is_blank(lines[i]) {
            return Some(indent_level_of(lines[i], tab_size));
        }
    }
    None
}

/// Check if a line is blank (empty or all whitespace).
fn is_blank(line: &[u8]) -> bool {
    line.iter().all(|&b| b == b' ' || b == b'\t')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_guides_empty_line_returns_empty() {
        // Validates: Requirement 3.3
        let result = compute_real_guides(b"", 4);
        assert!(result.is_empty());
    }

    #[test]
    fn real_guides_no_indent_returns_empty() {
        // Validates: Requirement 3.3
        let result = compute_real_guides(b"hello", 4);
        assert!(result.is_empty());
    }

    #[test]
    fn real_guides_single_tab_returns_one_guide() {
        // Validates: Requirement 3.3
        let result = compute_real_guides(b"    hello", 4);
        assert_eq!(result, vec![4]);
    }

    #[test]
    fn real_guides_two_levels() {
        // Validates: Requirement 3.3
        let result = compute_real_guides(b"        hello", 4);
        assert_eq!(result, vec![4, 8]);
    }

    #[test]
    fn real_guides_tab_character() {
        // Validates: Requirement 3.3
        let result = compute_real_guides(b"\t\thello", 4);
        assert_eq!(result, vec![4, 8]);
    }

    #[test]
    fn look_forward_extends_through_blank() {
        // Validates: Requirement 3.4
        let lines: Vec<&[u8]> = vec![b"    hello", b"", b"    world"];
        let result = compute_look_forward_guides(&lines, 1, 4);
        // Blank line at index 1 should get guides from forward scan (indent=4)
        assert_eq!(result, vec![4]);
    }

    #[test]
    fn look_both_extends_through_blank_using_max() {
        // Validates: Requirement 3.5
        let lines: Vec<&[u8]> = vec![
            b"        prev", // indent 8
            b"",             // blank
            b"    next",     // indent 4
        ];
        let result = compute_look_both_guides(&lines, 1, 4);
        // Max of prev=8, next=4 → effective=8 → guides at 4, 8
        assert_eq!(result, vec![4, 8]);
    }

    #[test]
    fn look_forward_no_following_non_blank() {
        // Validates: Requirement 3.4
        let lines: Vec<&[u8]> = vec![b"    hello", b""];
        let result = compute_look_forward_guides(&lines, 1, 4);
        // No forward non-blank line, blank line own indent is 0
        assert!(result.is_empty());
    }

    #[test]
    fn real_guides_columns_are_multiples_of_tab_size() {
        // Validates: Requirement 3.3
        let result = compute_real_guides(b"            deep", 4);
        for col in &result {
            assert_eq!(col % 4, 0);
        }
    }
}
