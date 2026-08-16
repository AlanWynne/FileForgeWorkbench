//! Forward and backward blank-line scanning for indent guide extension.

use super::level::indent_level_of;

/// Maximum number of blank lines to scan through before giving up.
const MAX_SCAN_DISTANCE: u64 = 2000;

/// Scan forward from a line to find the next non-blank line's indent level.
///
/// Used by `LookForward` and `LookBoth` modes.
///
/// # Arguments
///
/// * `start_line` - The line index to start scanning from (exclusive).
/// * `line_count` - Total number of lines in the document.
/// * `get_line` - Closure that returns the content of a line by index.
/// * `tab_size` - Tab stop size for indent computation.
///
/// Addresses: Requirement 3 AC 3.4
pub fn scan_forward_indent<F>(
    start_line: u64,
    line_count: u64,
    get_line: F,
    tab_size: u32,
) -> Option<u32>
where
    F: Fn(u64) -> Vec<u8>,
{
    let mut scanned = 0u64;
    let mut line = start_line + 1;

    while line < line_count && scanned < MAX_SCAN_DISTANCE {
        let content = get_line(line);
        if !is_blank(&content) {
            return Some(indent_level_of(&content, tab_size));
        }
        line += 1;
        scanned += 1;
    }

    None
}

/// Scan backward from a line to find the previous non-blank line's indent level.
///
/// Used by `LookBoth` mode.
///
/// # Arguments
///
/// * `start_line` - The line index to start scanning from (exclusive).
/// * `get_line` - Closure that returns the content of a line by index.
/// * `tab_size` - Tab stop size for indent computation.
///
/// Addresses: Requirement 3 AC 3.5
pub fn scan_backward_indent<F>(start_line: u64, get_line: F, tab_size: u32) -> Option<u32>
where
    F: Fn(u64) -> Vec<u8>,
{
    if start_line == 0 {
        return None;
    }

    let mut scanned = 0u64;
    let mut line = start_line - 1;

    loop {
        let content = get_line(line);
        if !is_blank(&content) {
            return Some(indent_level_of(&content, tab_size));
        }

        scanned += 1;
        if scanned >= MAX_SCAN_DISTANCE || line == 0 {
            break;
        }
        line -= 1;
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
    fn scan_forward_finds_next_non_blank() {
        let lines: Vec<Vec<u8>> = vec![
            b"    hello".to_vec(),
            b"".to_vec(),
            b"   ".to_vec(),
            b"        world".to_vec(),
        ];
        let result = scan_forward_indent(0, lines.len() as u64, |i| lines[i as usize].clone(), 4);
        assert_eq!(result, Some(8));
    }

    #[test]
    fn scan_forward_no_non_blank_returns_none() {
        let lines: Vec<Vec<u8>> = vec![b"hello".to_vec(), b"".to_vec(), b"   ".to_vec()];
        let result = scan_forward_indent(0, lines.len() as u64, |i| lines[i as usize].clone(), 4);
        assert_eq!(result, None);
    }

    #[test]
    fn scan_backward_finds_previous_non_blank() {
        let lines: Vec<Vec<u8>> = vec![
            b"        prev".to_vec(),
            b"".to_vec(),
            b"    current".to_vec(),
        ];
        let result = scan_backward_indent(2, |i| lines[i as usize].clone(), 4);
        assert_eq!(result, Some(8));
    }

    #[test]
    fn scan_backward_from_zero_returns_none() {
        let result = scan_backward_indent(0, |_| Vec::new(), 4);
        assert_eq!(result, None);
    }

    #[test]
    fn scan_backward_all_blank_returns_none() {
        let lines: Vec<Vec<u8>> = vec![b"   ".to_vec(), b"".to_vec(), b"    current".to_vec()];
        let result = scan_backward_indent(2, |i| lines[i as usize].clone(), 4);
        assert_eq!(result, None);
    }
}
