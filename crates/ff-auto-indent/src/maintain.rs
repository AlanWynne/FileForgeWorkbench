//! Maintain-indent engine — copies reference line whitespace.
//!
//! Implements the simplest auto-indent behaviour: the new line receives
//! exactly the same indentation as the reference line (the line where
//! Enter was pressed), respecting caret position within whitespace.

use crate::config::IndentConfig;
use crate::decision::IndentDecision;

/// Information about a line's leading whitespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndentInfo {
    /// The raw whitespace characters (tabs and/or spaces).
    pub whitespace: String,
    /// The column width of the whitespace when tabs are expanded.
    pub column_width: u32,
    /// The column of the first non-whitespace character.
    pub first_content_column: u32,
}

/// Parse the leading whitespace of a line, computing column positions.
///
/// Scans from the start of the line until a non-whitespace character is found.
/// Tab characters are expanded using the given `tab_size`.
pub fn parse_line_indent(line_text: &str, tab_size: u32) -> LineIndentInfo {
    let mut whitespace = String::new();
    let mut column: u32 = 0;

    for ch in line_text.chars() {
        match ch {
            ' ' => {
                whitespace.push(' ');
                column += 1;
            }
            '\t' => {
                whitespace.push('\t');
                column = column + tab_size - (column % tab_size);
            }
            _ => break,
        }
    }

    LineIndentInfo {
        whitespace,
        column_width: column,
        first_content_column: column,
    }
}

/// Extract whitespace from a line up to the given target column.
///
/// Walks the leading whitespace of `line_text`, accumulating characters
/// until the target column is reached or exceeded. Generates the result
/// whitespace string using the configured `style` setting.
pub fn extract_whitespace_to_column(
    line_text: &str,
    target_column: u32,
    config: &IndentConfig,
) -> String {
    if target_column == 0 {
        return String::new();
    }

    // Walk through leading whitespace, tracking column
    let mut current_column: u32 = 0;

    for ch in line_text.chars() {
        match ch {
            ' ' => {
                current_column += 1;
            }
            '\t' => {
                current_column =
                    current_column + config.tab_size() - (current_column % config.tab_size());
            }
            _ => break,
        }

        if current_column >= target_column {
            break;
        }
    }

    // Generate whitespace for the target column using the config's style
    let effective_columns = target_column.min(current_column);
    config.whitespace_for_columns(effective_columns)
}

/// Compute maintain-indent for a new line.
///
/// Copies the reference line's leading whitespace to the new line,
/// respecting the caret position:
/// - If caret is at column 0, return zero indent.
/// - If caret is within leading whitespace, return whitespace up to caret.
/// - Otherwise, return the full leading whitespace of the reference line.
pub fn compute_maintain_indent(
    reference_line: &str,
    caret_column: u64,
    config: &IndentConfig,
) -> IndentDecision {
    // Caret at column 0 → no indent
    if caret_column == 0 {
        return IndentDecision::no_indent();
    }

    let indent_info = parse_line_indent(reference_line, config.tab_size());

    // If reference line has no whitespace, no indent
    if indent_info.column_width == 0 {
        return IndentDecision::no_indent();
    }

    let caret_col = caret_column as u32;

    // Caret is within leading whitespace: reproduce only up to caret position
    if caret_col < indent_info.first_content_column {
        let whitespace = extract_whitespace_to_column(reference_line, caret_col, config);
        let level = config.columns_to_level(caret_col).value();
        return IndentDecision::maintain(whitespace, level);
    }

    // Caret is at or after first content: reproduce full leading whitespace
    let whitespace = config.whitespace_for_columns(indent_info.column_width);
    let level = config.columns_to_level(indent_info.column_width).value();
    IndentDecision::maintain(whitespace, level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IndentStyle;

    #[test]
    fn parse_line_indent_spaces_only() {
        // Validates: Requirement 2.2
        let info = parse_line_indent("    hello", 4);
        assert_eq!(info.whitespace, "    ");
        assert_eq!(info.column_width, 4);
        assert_eq!(info.first_content_column, 4);
    }

    #[test]
    fn parse_line_indent_tabs_only() {
        // Validates: Requirement 2.2
        let info = parse_line_indent("\t\thello", 4);
        assert_eq!(info.whitespace, "\t\t");
        assert_eq!(info.column_width, 8);
        assert_eq!(info.first_content_column, 8);
    }

    #[test]
    fn parse_line_indent_mixed_tabs_spaces() {
        // Validates: Requirement 2.2
        let info = parse_line_indent("\t  hello", 4);
        assert_eq!(info.whitespace, "\t  ");
        assert_eq!(info.column_width, 6); // tab → col 4, then 2 spaces
        assert_eq!(info.first_content_column, 6);
    }

    #[test]
    fn parse_line_indent_no_whitespace() {
        // Validates: Requirement 2.2
        let info = parse_line_indent("hello", 4);
        assert_eq!(info.whitespace, "");
        assert_eq!(info.column_width, 0);
    }

    #[test]
    fn parse_line_indent_empty_line() {
        // Validates: Requirement 2.2
        let info = parse_line_indent("", 4);
        assert_eq!(info.whitespace, "");
        assert_eq!(info.column_width, 0);
    }

    #[test]
    fn parse_line_indent_all_whitespace() {
        // Validates: Requirement 2.2
        let info = parse_line_indent("      ", 4);
        assert_eq!(info.whitespace, "      ");
        assert_eq!(info.column_width, 6);
    }

    #[test]
    fn maintain_indent_caret_at_column_zero() {
        // Validates: Requirement 2.5 — Enter at column 0 produces no indent
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        let result = compute_maintain_indent("    hello", 0, &config);
        assert_eq!(result.indent_text, "");
        assert_eq!(result.indent_level, 0);
    }

    #[test]
    fn maintain_indent_caret_after_content_reproduces_full_whitespace() {
        // Validates: Requirement 2.1 — maintain copies reference whitespace
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        let result = compute_maintain_indent("    hello world", 10, &config);
        assert_eq!(result.indent_text, "    ");
        assert_eq!(result.indent_level, 1);
    }

    #[test]
    fn maintain_indent_caret_within_whitespace() {
        // Validates: Requirement 2.6 — caret within indent copies partial
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        let result = compute_maintain_indent("        hello", 4, &config);
        // Caret at column 4 in 8-space indent → reproduce 4 cols
        assert_eq!(result.indent_text, "    ");
        assert_eq!(result.indent_level, 1);
    }

    #[test]
    fn maintain_indent_empty_reference_line() {
        // Validates: Requirement 2.1
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        let result = compute_maintain_indent("", 5, &config);
        assert_eq!(result.indent_text, "");
        assert_eq!(result.indent_level, 0);
    }

    #[test]
    fn maintain_indent_tabs_reference() {
        // Validates: Requirement 2.1 with tabs
        let config = IndentConfig::new(4, 4, IndentStyle::Tabs);
        let result = compute_maintain_indent("\t\thello", 10, &config);
        // Reference has 2 tabs = 8 columns, reproduce as tabs
        assert_eq!(result.indent_text, "\t\t");
        assert_eq!(result.indent_level, 2);
    }

    #[test]
    fn maintain_indent_mixed_reference_uses_config_style() {
        // Validates: Requirement 2.3 — reproduces using configured style
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        // Reference has tab (col 4) + 2 spaces (col 6)
        let result = compute_maintain_indent("\t  hello", 10, &config);
        // Should produce 6 columns as spaces
        assert_eq!(result.indent_text, "      ");
    }

    #[test]
    fn extract_whitespace_to_column_zero() {
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        assert_eq!(extract_whitespace_to_column("    hello", 0, &config), "");
    }

    #[test]
    fn extract_whitespace_to_column_partial() {
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        let result = extract_whitespace_to_column("        hello", 6, &config);
        assert_eq!(result, "      "); // 6 spaces
    }

    #[test]
    fn extract_whitespace_to_column_with_tabs() {
        let config = IndentConfig::new(4, 4, IndentStyle::Tabs);
        // Tab goes to col 4, target is col 4
        let result = extract_whitespace_to_column("\thello", 4, &config);
        assert_eq!(result, "\t");
    }
}
