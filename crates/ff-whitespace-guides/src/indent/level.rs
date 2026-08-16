//! Indent level computation for a single line.

/// Compute the indent level (in columns) of a line given `tab_size`.
///
/// Stops at the first non-whitespace byte. Tabs advance to the next
/// tab stop; spaces advance by one column.
///
/// Addresses: Requirement 3 AC 3.3
pub fn indent_level_of(line: &[u8], tab_size: u32) -> u32 {
    let tab_size = tab_size.max(1);
    let mut column: u32 = 0;

    for &byte in line {
        match byte {
            b' ' => column += 1,
            b'\t' => {
                column = column + tab_size - (column % tab_size);
            }
            _ => break,
        }
    }

    column
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_line_has_zero_indent() {
        assert_eq!(indent_level_of(b"", 4), 0);
    }

    #[test]
    fn no_indent_returns_zero() {
        assert_eq!(indent_level_of(b"hello", 4), 0);
    }

    #[test]
    fn spaces_counted_as_columns() {
        assert_eq!(indent_level_of(b"    hello", 4), 4);
    }

    #[test]
    fn single_tab_advances_to_tab_stop() {
        assert_eq!(indent_level_of(b"\thello", 4), 4);
    }

    #[test]
    fn two_tabs() {
        assert_eq!(indent_level_of(b"\t\thello", 4), 8);
    }

    #[test]
    fn mixed_spaces_and_tab() {
        // 2 spaces + tab: columns 0,1 are spaces, tab at col 2 advances to col 4
        assert_eq!(indent_level_of(b"  \thello", 4), 4);
    }

    #[test]
    fn all_whitespace_line() {
        assert_eq!(indent_level_of(b"      ", 4), 6);
    }

    #[test]
    fn tab_size_one() {
        assert_eq!(indent_level_of(b"\thello", 1), 1);
    }

    #[test]
    fn tab_size_eight() {
        assert_eq!(indent_level_of(b"\thello", 8), 8);
    }
}
