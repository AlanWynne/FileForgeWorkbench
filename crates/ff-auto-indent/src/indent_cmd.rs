//! Indent/Unindent command handler and registration.
//!
//! Implements the `edit.indent` and `edit.unindent` commands for explicit
//! indentation control of selected lines.

use crate::config::IndentConfig;
use crate::maintain::parse_line_indent;

/// A single line edit produced by indent/unindent operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndentLineEdit {
    /// The line number affected (0-based).
    pub line: u64,
    /// The new leading whitespace for the line (replaces existing).
    pub new_indent: String,
    /// Whether the line was actually modified (false if already at target).
    pub modified: bool,
}

/// Indent the given lines by one indent level.
///
/// - Prepends one indent_string worth of whitespace columns to each line.
/// - Normalises mixed whitespace to the current use_tabs setting.
/// - Returns the edits to apply.
pub fn indent_lines(
    lines: &[u64],
    line_contents: &[&str],
    config: &IndentConfig,
) -> Vec<IndentLineEdit> {
    lines
        .iter()
        .zip(line_contents.iter())
        .map(|(&line_num, &content)| {
            let indent_info = parse_line_indent(content, config.tab_size());
            let new_columns = indent_info.column_width + config.indent_size();
            let new_indent = config.whitespace_for_columns(new_columns);

            IndentLineEdit {
                line: line_num,
                new_indent,
                modified: true,
            }
        })
        .collect()
}

/// Unindent the given lines by one indent level.
///
/// - Removes one indent_string worth of leading whitespace from each line.
/// - Lines with less than one full indent level have all whitespace removed.
/// - Lines already at column 0 are unchanged.
pub fn unindent_lines(
    lines: &[u64],
    line_contents: &[&str],
    config: &IndentConfig,
) -> Vec<IndentLineEdit> {
    lines
        .iter()
        .zip(line_contents.iter())
        .map(|(&line_num, &content)| {
            let indent_info = parse_line_indent(content, config.tab_size());

            if indent_info.column_width == 0 {
                // Already at column 0 — no change
                return IndentLineEdit {
                    line: line_num,
                    new_indent: String::new(),
                    modified: false,
                };
            }

            let remove_columns = config.indent_size().min(indent_info.column_width);
            let new_columns = indent_info.column_width - remove_columns;
            let new_indent = config.whitespace_for_columns(new_columns);

            IndentLineEdit {
                line: line_num,
                new_indent,
                modified: true,
            }
        })
        .collect()
}

/// Normalise mixed leading whitespace to the configured style.
///
/// Converts the leading whitespace of a line to pure tabs or pure spaces
/// based on the current `IndentConfig` style.
pub fn normalise_whitespace(line_content: &str, config: &IndentConfig) -> String {
    let indent_info = parse_line_indent(line_content, config.tab_size());
    let normalised_ws = config.whitespace_for_columns(indent_info.column_width);
    let content_after_ws = &line_content[indent_info.whitespace.len()..];
    format!("{}{}", normalised_ws, content_after_ws)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IndentStyle;

    fn make_config() -> IndentConfig {
        IndentConfig::new(4, 4, IndentStyle::Spaces)
    }

    #[test]
    fn indent_single_line() {
        // Validates: Requirement 7.1 — indent adds one level
        let config = make_config();
        let result = indent_lines(&[0], &["    hello"], &config);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].new_indent, "        "); // 4 → 8 spaces
        assert!(result[0].modified);
    }

    #[test]
    fn indent_multiple_lines() {
        // Validates: Requirement 7.1 — indent all selected lines
        let config = make_config();
        let result = indent_lines(&[0, 1, 2], &["hello", "    world", "        end"], &config);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].new_indent, "    "); // 0 → 4
        assert_eq!(result[1].new_indent, "        "); // 4 → 8
        assert_eq!(result[2].new_indent, "            "); // 8 → 12
    }

    #[test]
    fn indent_empty_line() {
        // Validates: Requirement 7.1
        let config = make_config();
        let result = indent_lines(&[0], &[""], &config);
        assert_eq!(result[0].new_indent, "    ");
        assert!(result[0].modified);
    }

    #[test]
    fn unindent_single_line() {
        // Validates: Requirement 8.1 — unindent removes one level
        let config = make_config();
        let result = unindent_lines(&[0], &["        hello"], &config);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].new_indent, "    "); // 8 → 4
        assert!(result[0].modified);
    }

    #[test]
    fn unindent_multiple_lines() {
        // Validates: Requirement 8.1
        let config = make_config();
        let result = unindent_lines(
            &[0, 1, 2],
            &["        a", "    b", "            c"],
            &config,
        );
        assert_eq!(result[0].new_indent, "    "); // 8 → 4
        assert_eq!(result[1].new_indent, ""); // 4 → 0
        assert_eq!(result[2].new_indent, "        "); // 12 → 8
    }

    #[test]
    fn unindent_below_floor_removes_all() {
        // Validates: Requirement 8.2 — less than one level removes all
        let config = make_config();
        let result = unindent_lines(&[0], &["  hello"], &config);
        assert_eq!(result[0].new_indent, ""); // 2 cols < 4, remove all
        assert!(result[0].modified);
    }

    #[test]
    fn unindent_at_column_zero_unchanged() {
        // Validates: Requirement 8.2 — already at zero, no change
        let config = make_config();
        let result = unindent_lines(&[0], &["hello"], &config);
        assert_eq!(result[0].new_indent, "");
        assert!(!result[0].modified);
    }

    #[test]
    fn indent_with_tabs() {
        // Validates: Requirement 7.1 with tab style
        let config = IndentConfig::new(4, 4, IndentStyle::Tabs);
        let result = indent_lines(&[0], &["\thello"], &config);
        assert_eq!(result[0].new_indent, "\t\t"); // 1 tab → 2 tabs
    }

    #[test]
    fn unindent_with_tabs() {
        // Validates: Requirement 8.7 — tab counts as tab_size columns
        let config = IndentConfig::new(4, 4, IndentStyle::Tabs);
        let result = unindent_lines(&[0], &["\t\thello"], &config);
        assert_eq!(result[0].new_indent, "\t"); // 2 tabs → 1 tab
    }

    #[test]
    fn normalise_whitespace_mixed_to_spaces() {
        // Validates: Requirement 7.5 — normalise mixed whitespace
        let config = make_config();
        // Tab (4 cols) + 2 spaces = 6 columns total
        let result = normalise_whitespace("\t  hello", &config);
        assert_eq!(result, "      hello"); // 6 spaces
    }

    #[test]
    fn normalise_whitespace_mixed_to_tabs() {
        // Validates: Requirement 7.5
        let config = IndentConfig::new(4, 4, IndentStyle::Tabs);
        // 8 spaces = 2 tab stops
        let result = normalise_whitespace("        hello", &config);
        assert_eq!(result, "\t\thello");
    }

    #[test]
    fn indent_unindent_roundtrip() {
        // Validates: Requirements 7.1, 8.1 — roundtrip identity
        let config = make_config();
        let original = "        hello"; // 8 spaces = level 2
        let indented = indent_lines(&[0], &[original], &config);
        // Simulate the indented line
        let indented_line = format!("{}hello", indented[0].new_indent);
        let restored = unindent_lines(&[0], &[&indented_line], &config);
        assert_eq!(restored[0].new_indent, "        "); // back to 8
    }

    #[test]
    fn indent_size_2() {
        // Test with different indent size
        let config = IndentConfig::new(2, 4, IndentStyle::Spaces);
        let result = indent_lines(&[0], &["  hello"], &config);
        assert_eq!(result[0].new_indent, "    "); // 2 → 4 spaces
    }

    #[test]
    fn unindent_size_2() {
        let config = IndentConfig::new(2, 4, IndentStyle::Spaces);
        let result = unindent_lines(&[0], &["    hello"], &config);
        assert_eq!(result[0].new_indent, "  "); // 4 → 2 spaces
    }
}
