//! Compile-time key definitions.
//!
//! Provides well-known configuration key paths as constants, ensuring
//! consistent key names across all consumers without typo risk.

/// Editor-related configuration keys.
pub mod editor {
    /// Key for the tab size setting (number of spaces per tab).
    pub const TAB_SIZE: &str = "editor.tab_size";
    /// Key for the indent size setting (columns per indent level).
    pub const INDENT_SIZE: &str = "editor.indent_size";
    /// Key for the tab width setting (display width of a tab character).
    pub const TAB_WIDTH: &str = "editor.tab_width";
    /// Key for the indent style setting (spaces or tabs).
    pub const INDENT_STYLE: &str = "editor.indent_style";
    /// Key for the line endings setting (lf, crlf, cr).
    pub const LINE_ENDINGS: &str = "editor.line_endings";
    /// Key for the end-of-line style (alias for line_endings used by EditorConfig).
    pub const END_OF_LINE: &str = "editor.end_of_line";
    /// Key for the file charset/encoding setting.
    pub const CHARSET: &str = "editor.charset";
    /// Key for the trim trailing whitespace setting.
    pub const TRIM_TRAILING_WHITESPACE: &str = "editor.trim_trailing_whitespace";
    /// Key for the insert final newline setting.
    pub const INSERT_FINAL_NEWLINE: &str = "editor.insert_final_newline";
}

/// Logging-related configuration keys.
pub mod logging {
    /// Key for the log level setting.
    pub const LEVEL: &str = "logging.level";
    /// Key for the log output directory.
    pub const DIRECTORY: &str = "logging.directory";
    /// Key for the maximum log file size in megabytes.
    pub const MAX_FILE_SIZE_MB: &str = "logging.max_file_size_mb";
    /// Key for the maximum number of retained log files.
    pub const MAX_RETAINED_FILES: &str = "logging.max_retained_files";
}

/// Theme-related configuration keys.
pub mod theme {
    /// Key for the active theme name.
    pub const ACTIVE: &str = "theme.active";
    /// Key for the font size setting.
    pub const FONT_SIZE: &str = "theme.font_size";
}

/// Virtual File System configuration keys.
pub mod vfs {
    /// Key for the default VFS provider.
    pub const DEFAULT_PROVIDER: &str = "vfs.default_provider";
}

/// Catalog-related configuration keys.
pub mod catalogs {
    /// Default local repository root for new Mainframe catalogs.
    pub const DEFAULT_MAINFRAME_ROOT: &str = "catalogs.default_mainframe_root";
    /// Default root directory for new POSIX catalogs.
    pub const DEFAULT_POSIX_ROOT: &str = "catalogs.default_posix_root";
}

/// Accessibility configuration keys.
pub mod accessibility {
    /// When true, disables non-essential animations (smooth scroll, transitions).
    /// Defaults to false; set to true to honour OS reduce-motion preference.
    pub const REDUCE_MOTION: &str = "accessibility.reduce_motion";
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates that a key is a valid dot-separated path:
    /// 1. Contains exactly one dot (namespace.key format)
    /// 2. Starts with its expected namespace prefix
    /// 3. All characters are lowercase ASCII, digits, underscores, or dots
    fn assert_valid_key(key: &str, expected_namespace: &str) {
        // Must contain exactly one dot
        let dot_count = key.chars().filter(|&c| c == '.').count();
        assert_eq!(
            dot_count, 1,
            "Key '{key}' should contain exactly one dot, found {dot_count}"
        );

        // Must start with expected namespace prefix
        assert!(
            key.starts_with(&format!("{expected_namespace}.")),
            "Key '{key}' should start with '{expected_namespace}.'"
        );

        // All characters must be lowercase ASCII, digits, underscores, or dots
        for ch in key.chars() {
            assert!(
                ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '.',
                "Key '{key}' contains invalid character '{ch}'"
            );
        }
    }

    #[test]
    fn editor_keys_are_valid_dot_separated_paths() {
        // Validates: Requirement 7.2
        assert_valid_key(editor::TAB_SIZE, "editor");
        assert_valid_key(editor::INDENT_SIZE, "editor");
        assert_valid_key(editor::TAB_WIDTH, "editor");
        assert_valid_key(editor::INDENT_STYLE, "editor");
        assert_valid_key(editor::LINE_ENDINGS, "editor");
        assert_valid_key(editor::END_OF_LINE, "editor");
        assert_valid_key(editor::CHARSET, "editor");
        assert_valid_key(editor::TRIM_TRAILING_WHITESPACE, "editor");
        assert_valid_key(editor::INSERT_FINAL_NEWLINE, "editor");
    }

    #[test]
    fn logging_keys_are_valid_dot_separated_paths() {
        // Validates: Requirement 7.2
        assert_valid_key(logging::LEVEL, "logging");
        assert_valid_key(logging::DIRECTORY, "logging");
        assert_valid_key(logging::MAX_FILE_SIZE_MB, "logging");
        assert_valid_key(logging::MAX_RETAINED_FILES, "logging");
    }

    #[test]
    fn theme_keys_are_valid_dot_separated_paths() {
        // Validates: Requirement 7.2
        assert_valid_key(theme::ACTIVE, "theme");
        assert_valid_key(theme::FONT_SIZE, "theme");
    }

    #[test]
    fn vfs_keys_are_valid_dot_separated_paths() {
        // Validates: Requirement 7.2
        assert_valid_key(vfs::DEFAULT_PROVIDER, "vfs");
    }

    #[test]
    fn accessibility_keys_are_valid_dot_separated_paths() {
        // Validates: Requirement 5.2 (accessibility) -- reduce_motion config key
        assert_valid_key(accessibility::REDUCE_MOTION, "accessibility");
    }

    #[test]
    fn all_keys_have_unique_values() {
        // Validates: Requirement 7.2
        let all_keys = [
            editor::TAB_SIZE,
            editor::INDENT_SIZE,
            editor::TAB_WIDTH,
            editor::INDENT_STYLE,
            editor::LINE_ENDINGS,
            editor::END_OF_LINE,
            editor::CHARSET,
            editor::TRIM_TRAILING_WHITESPACE,
            editor::INSERT_FINAL_NEWLINE,
            logging::LEVEL,
            logging::DIRECTORY,
            logging::MAX_FILE_SIZE_MB,
            logging::MAX_RETAINED_FILES,
            theme::ACTIVE,
            theme::FONT_SIZE,
            vfs::DEFAULT_PROVIDER,
            accessibility::REDUCE_MOTION,
        ];

        let unique: std::collections::HashSet<&str> = all_keys.iter().copied().collect();
        assert_eq!(
            all_keys.len(),
            unique.len(),
            "All key constants must have unique values"
        );
    }
}
