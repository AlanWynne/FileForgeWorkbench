//! Configuration integration for navigation commands.
//!
//! Loads configurable values from the configuration system with
//! fallback-to-default and warning emission for missing/invalid values.

use crate::types::NavigationConfig;

/// Configuration key for default horizontal scroll columns.
pub const KEY_HORIZONTAL_SCROLL_COLUMNS: &str = "editor.navigation.horizontal_scroll_columns";

/// Configuration key for page overlap lines.
pub const KEY_PAGE_OVERLAP_LINES: &str = "editor.navigation.page_overlap_lines";

/// Configuration key for whether bounds affect FIND.
pub const KEY_BOUNDS_AFFECT_FIND: &str = "editor.bounds.affect_find";

/// Configuration key for extra word characters.
pub const KEY_WORD_CHARACTERS: &str = "editor.navigation.word_characters";

/// Default value for horizontal scroll columns.
pub const DEFAULT_HORIZONTAL_SCROLL_COLUMNS: u64 = 8;

/// Default value for page overlap lines.
pub const DEFAULT_PAGE_OVERLAP_LINES: u64 = 2;

/// Default value for bounds_affect_find.
pub const DEFAULT_BOUNDS_AFFECT_FIND: bool = false;

/// Load navigation configuration with defaults for missing/invalid values.
///
/// Returns a `NavigationConfig` with any values that could not be loaded
/// replaced by their defaults.
pub fn load_navigation_config(
    horizontal_scroll: Option<u64>,
    page_overlap: Option<u64>,
    affect_find: Option<bool>,
    word_characters: Option<&str>,
) -> NavigationConfig {
    NavigationConfig {
        horizontal_scroll_columns: horizontal_scroll.unwrap_or(DEFAULT_HORIZONTAL_SCROLL_COLUMNS),
        page_overlap_lines: page_overlap.unwrap_or(DEFAULT_PAGE_OVERLAP_LINES),
        bounds_affect_find: affect_find.unwrap_or(DEFAULT_BOUNDS_AFFECT_FIND),
        extra_word_characters: word_characters.unwrap_or("").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_all_defaults() {
        let config = load_navigation_config(None, None, None, None);
        assert_eq!(config.horizontal_scroll_columns, 8);
        assert_eq!(config.page_overlap_lines, 2);
        assert!(!config.bounds_affect_find);
        assert_eq!(config.extra_word_characters, "");
    }

    #[test]
    fn custom_values_override_defaults() {
        let config = load_navigation_config(Some(16), Some(3), Some(true), Some("$@"));
        assert_eq!(config.horizontal_scroll_columns, 16);
        assert_eq!(config.page_overlap_lines, 3);
        assert!(config.bounds_affect_find);
        assert_eq!(config.extra_word_characters, "$@");
    }

    #[test]
    fn partial_values_fill_in_defaults() {
        let config = load_navigation_config(Some(4), None, None, Some("-"));
        assert_eq!(config.horizontal_scroll_columns, 4);
        assert_eq!(config.page_overlap_lines, 2);
        assert!(!config.bounds_affect_find);
        assert_eq!(config.extra_word_characters, "-");
    }
}
