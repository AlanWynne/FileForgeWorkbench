//! Clipboard configuration — typed access to clipboard-related settings.
//!
//! Provides [`ClipboardConfig`] with defaults and validation. Invalid values
//! are logged as warnings and fall back to documented defaults.

/// Typed configuration for clipboard behaviour.
///
/// All fields have sensible defaults defined by the `Default` implementation.
/// Invalid configuration values are replaced with these defaults and a warning
/// is logged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardConfig {
    /// Whether Ctrl+C with no selection copies the entire line.
    /// Default: `true`.
    pub line_copy_when_no_selection: bool,

    /// Whether rectangular paste creates new lines beyond document end.
    /// Default: `true`.
    pub rectangular_paste_adds_lines: bool,

    /// Timeout in milliseconds for clipboard access operations.
    /// Default: `500`.
    pub access_timeout_ms: u32,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            line_copy_when_no_selection: true,
            rectangular_paste_adds_lines: true,
            access_timeout_ms: 500,
        }
    }
}

impl ClipboardConfig {
    /// Create a config with all defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate and normalize the config, replacing invalid values with defaults.
    ///
    /// Returns a list of keys that were corrected.
    pub fn validate(&mut self) -> Vec<String> {
        let mut corrected = Vec::new();

        if self.access_timeout_ms == 0 {
            self.access_timeout_ms = 500;
            corrected.push("clipboard.access_timeout_ms".to_string());
        }

        corrected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        // Validates: Requirement 19.1, 19.2, 19.3
        let config = ClipboardConfig::default();
        assert!(config.line_copy_when_no_selection);
        assert!(config.rectangular_paste_adds_lines);
        assert_eq!(config.access_timeout_ms, 500);
    }

    #[test]
    fn validate_corrects_zero_timeout() {
        // Validates: Requirement 19.4
        let mut config = ClipboardConfig {
            access_timeout_ms: 0,
            ..Default::default()
        };
        let corrected = config.validate();
        assert_eq!(config.access_timeout_ms, 500);
        assert!(corrected.contains(&"clipboard.access_timeout_ms".to_string()));
    }

    #[test]
    fn validate_leaves_valid_config_unchanged() {
        let mut config = ClipboardConfig {
            line_copy_when_no_selection: false,
            rectangular_paste_adds_lines: false,
            access_timeout_ms: 1000,
        };
        let corrected = config.validate();
        assert!(corrected.is_empty());
        assert!(!config.line_copy_when_no_selection);
        assert!(!config.rectangular_paste_adds_lines);
        assert_eq!(config.access_timeout_ms, 1000);
    }
}
