//! Indent configuration types and accessors.
//!
//! Defines `IndentStyle` and `IndentConfig` which determine the physical
//! representation of indentation (tabs vs spaces, sizes).

use crate::types::IndentLevel;

/// Whether indentation uses tab characters or space characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndentStyle {
    /// Indentation is composed of tab characters.
    Tabs,
    /// Indentation is composed of space characters.
    Spaces,
}

impl Default for IndentStyle {
    /// Defaults to `Spaces`.
    fn default() -> Self {
        Self::Spaces
    }
}

/// Physical indentation settings for a document.
///
/// Encapsulates indent_size, tab_size, and style to determine how
/// indentation is physically represented in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndentConfig {
    /// Number of columns per logical indent level (e.g., 4).
    indent_size: u32,
    /// Display width of a tab character in columns (e.g., 4 or 8).
    tab_size: u32,
    /// Whether to use tab characters or spaces for indentation.
    style: IndentStyle,
}

impl IndentConfig {
    /// Create a new `IndentConfig` with explicit values.
    ///
    /// # Panics
    ///
    /// Never panics. Values of 0 for indent_size or tab_size are clamped to 1.
    pub fn new(indent_size: u32, tab_size: u32, style: IndentStyle) -> Self {
        Self {
            indent_size: indent_size.max(1),
            tab_size: tab_size.max(1),
            style,
        }
    }

    /// Returns the configured indent size in columns.
    pub fn indent_size(&self) -> u32 {
        self.indent_size
    }

    /// Returns the configured tab display width in columns.
    pub fn tab_size(&self) -> u32 {
        self.tab_size
    }

    /// Returns the configured indent style.
    pub fn style(&self) -> IndentStyle {
        self.style
    }

    /// Generate the physical indent string for one level of indentation.
    ///
    /// Returns a single tab character if style is `Tabs`, or `indent_size`
    /// space characters if style is `Spaces`.
    pub fn indent_string(&self) -> String {
        match self.style {
            IndentStyle::Tabs => "\t".to_string(),
            IndentStyle::Spaces => " ".repeat(self.indent_size as usize),
        }
    }

    /// Generate the physical whitespace string for the given number of indent levels.
    pub fn whitespace_for_level(&self, level: IndentLevel) -> String {
        let levels = level.value();
        match self.style {
            IndentStyle::Tabs => "\t".repeat(levels as usize),
            IndentStyle::Spaces => " ".repeat((levels * self.indent_size) as usize),
        }
    }

    /// Convert a column width to the corresponding indent level (integer division).
    ///
    /// Partial levels are truncated (floor division).
    pub fn columns_to_level(&self, columns: u32) -> IndentLevel {
        IndentLevel::new(columns / self.indent_size)
    }

    /// Convert an indent level to the corresponding column width.
    pub fn level_to_columns(&self, level: IndentLevel) -> u32 {
        level.value() * self.indent_size
    }

    /// Calculate the column width of a given whitespace string,
    /// accounting for tab expansion.
    pub fn column_width_of(&self, whitespace: &str) -> u32 {
        let mut column: u32 = 0;
        for ch in whitespace.chars() {
            match ch {
                '\t' => {
                    // Tab advances to next tab stop
                    column = column + self.tab_size - (column % self.tab_size);
                }
                ' ' => {
                    column += 1;
                }
                _ => break, // Stop at first non-whitespace
            }
        }
        column
    }

    /// Convert a target column width to the appropriate whitespace string,
    /// respecting the current `style` setting.
    pub fn whitespace_for_columns(&self, columns: u32) -> String {
        match self.style {
            IndentStyle::Tabs => {
                let full_tabs = columns / self.tab_size;
                let remaining_spaces = columns % self.tab_size;
                let mut result = "\t".repeat(full_tabs as usize);
                if remaining_spaces > 0 {
                    result.push_str(&" ".repeat(remaining_spaces as usize));
                }
                result
            }
            IndentStyle::Spaces => " ".repeat(columns as usize),
        }
    }
}

impl Default for IndentConfig {
    /// Default: indent_size=4, tab_size=4, style=Spaces
    fn default() -> Self {
        Self {
            indent_size: 4,
            tab_size: 4,
            style: IndentStyle::Spaces,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indent_style_default_is_spaces() {
        // Validates: Requirement 1.1
        assert_eq!(IndentStyle::default(), IndentStyle::Spaces);
    }

    #[test]
    fn indent_config_default_values() {
        // Validates: Requirement 1.5
        let config = IndentConfig::default();
        assert_eq!(config.indent_size(), 4);
        assert_eq!(config.tab_size(), 4);
        assert_eq!(config.style(), IndentStyle::Spaces);
    }

    #[test]
    fn indent_string_with_spaces_returns_n_spaces() {
        // Validates: Requirement 1.5
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        assert_eq!(config.indent_string(), "    ");
    }

    #[test]
    fn indent_string_with_tabs_returns_single_tab() {
        // Validates: Requirement 1.5
        let config = IndentConfig::new(4, 4, IndentStyle::Tabs);
        assert_eq!(config.indent_string(), "\t");
    }

    #[test]
    fn indent_string_with_two_spaces() {
        // Validates: Requirement 1.5
        let config = IndentConfig::new(2, 4, IndentStyle::Spaces);
        assert_eq!(config.indent_string(), "  ");
    }

    #[test]
    fn whitespace_for_level_spaces() {
        // Validates: Requirement 1.5
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        assert_eq!(config.whitespace_for_level(IndentLevel::new(0)), "");
        assert_eq!(config.whitespace_for_level(IndentLevel::new(1)), "    ");
        assert_eq!(config.whitespace_for_level(IndentLevel::new(2)), "        ");
    }

    #[test]
    fn whitespace_for_level_tabs() {
        // Validates: Requirement 1.5
        let config = IndentConfig::new(4, 4, IndentStyle::Tabs);
        assert_eq!(config.whitespace_for_level(IndentLevel::new(0)), "");
        assert_eq!(config.whitespace_for_level(IndentLevel::new(1)), "\t");
        assert_eq!(config.whitespace_for_level(IndentLevel::new(3)), "\t\t\t");
    }

    #[test]
    fn columns_to_level_truncates() {
        // Validates: Requirement 4.6
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        assert_eq!(config.columns_to_level(0).value(), 0);
        assert_eq!(config.columns_to_level(3).value(), 0);
        assert_eq!(config.columns_to_level(4).value(), 1);
        assert_eq!(config.columns_to_level(7).value(), 1);
        assert_eq!(config.columns_to_level(8).value(), 2);
    }

    #[test]
    fn level_to_columns() {
        // Validates: Requirement 4.6
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        assert_eq!(config.level_to_columns(IndentLevel::new(0)), 0);
        assert_eq!(config.level_to_columns(IndentLevel::new(1)), 4);
        assert_eq!(config.level_to_columns(IndentLevel::new(3)), 12);
    }

    #[test]
    fn column_width_of_spaces() {
        // Validates: Requirement 2.2
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        assert_eq!(config.column_width_of(""), 0);
        assert_eq!(config.column_width_of("    "), 4);
        assert_eq!(config.column_width_of("        "), 8);
    }

    #[test]
    fn column_width_of_tabs() {
        // Validates: Requirement 2.2
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        assert_eq!(config.column_width_of("\t"), 4);
        assert_eq!(config.column_width_of("\t\t"), 8);
    }

    #[test]
    fn column_width_of_mixed_tabs_spaces() {
        // Validates: Requirement 2.2
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        // Tab at col 0 goes to col 4, then 2 spaces = col 6
        assert_eq!(config.column_width_of("\t  "), 6);
        // 2 spaces then tab: col 2, tab goes to col 4
        assert_eq!(config.column_width_of("  \t"), 4);
    }

    #[test]
    fn column_width_of_tab_size_8() {
        // Validates: Requirement 2.2
        let config = IndentConfig::new(4, 8, IndentStyle::Spaces);
        assert_eq!(config.column_width_of("\t"), 8);
        assert_eq!(config.column_width_of("    \t"), 8); // 4 spaces + tab goes to 8
    }

    #[test]
    fn whitespace_for_columns_spaces() {
        // Validates: Requirement 2.3
        let config = IndentConfig::new(4, 4, IndentStyle::Spaces);
        assert_eq!(config.whitespace_for_columns(0), "");
        assert_eq!(config.whitespace_for_columns(4), "    ");
        assert_eq!(config.whitespace_for_columns(6), "      ");
    }

    #[test]
    fn whitespace_for_columns_tabs() {
        // Validates: Requirement 2.3
        let config = IndentConfig::new(4, 4, IndentStyle::Tabs);
        assert_eq!(config.whitespace_for_columns(0), "");
        assert_eq!(config.whitespace_for_columns(4), "\t");
        assert_eq!(config.whitespace_for_columns(8), "\t\t");
        assert_eq!(config.whitespace_for_columns(6), "\t  "); // 1 tab (4 cols) + 2 spaces
    }

    #[test]
    fn indent_size_clamped_to_minimum_one() {
        let config = IndentConfig::new(0, 0, IndentStyle::Spaces);
        assert_eq!(config.indent_size(), 1);
        assert_eq!(config.tab_size(), 1);
    }
}
