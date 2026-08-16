//! Configuration for the completion subsystem.
//!
//! All settings are read from the `completion.*` namespace in the
//! configuration system. Invalid values fall back to documented defaults.

use crate::error::CompletionError;

/// Trigger activation mode.
///
/// Controls when the completion popup is activated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerMode {
    /// Only activate on explicit Ctrl+Space.
    Manual,
    /// Activate after typing N characters (auto_trigger_chars threshold).
    Automatic,
    /// Both automatic and manual triggers are active.
    Both,
}

impl TriggerMode {
    /// Parses a trigger mode string, returning the default on invalid input.
    pub fn from_str_with_fallback(s: &str) -> Result<Self, CompletionError> {
        match s.to_lowercase().as_str() {
            "manual" => Ok(Self::Manual),
            "automatic" => Ok(Self::Automatic),
            "both" => Ok(Self::Both),
            _ => Err(CompletionError::InvalidConfig {
                key: "completion.trigger_mode".to_string(),
                value: s.to_string(),
                default: "both".to_string(),
            }),
        }
    }
}

/// Matching algorithm selection.
///
/// Determines how typed text is compared against candidate labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchingMode {
    /// Strict prefix match (candidate must start with typed text).
    Prefix,
    /// Subsequence match (all typed characters appear in order).
    Fuzzy,
}

impl MatchingMode {
    /// Parses a matching mode string, returning the default on invalid input.
    pub fn from_str_with_fallback(s: &str) -> Result<Self, CompletionError> {
        match s.to_lowercase().as_str() {
            "prefix" => Ok(Self::Prefix),
            "fuzzy" => Ok(Self::Fuzzy),
            _ => Err(CompletionError::InvalidConfig {
                key: "completion.matching_mode".to_string(),
                value: s.to_string(),
                default: "prefix".to_string(),
            }),
        }
    }
}

/// Typed representation of all `completion.*` configuration keys.
///
/// Read from ff-config at engine initialization and on hot-reload.
/// Values are validated and clamped to their documented ranges.
#[derive(Debug, Clone)]
pub struct CompletionConfig {
    /// Trigger mode: Manual, Automatic, or Both.
    pub trigger_mode: TriggerMode,
    /// Character count threshold for automatic triggering (1–10).
    pub auto_trigger_chars: u8,
    /// Matching algorithm: Prefix or Fuzzy.
    pub matching_mode: MatchingMode,
    /// Whether matching is case-sensitive.
    pub case_sensitive: bool,
    /// Maximum visible candidates in popup (3–50).
    pub popup_max_items: u8,
    /// Maximum popup width in logical pixels (100–1000).
    pub popup_max_width: u16,
    /// Whether to auto-hide when zero candidates match.
    pub auto_hide: bool,
    /// Whether to dismiss when cursor retreats past anchor.
    pub cancel_at_start_pos: bool,
    /// Whether to auto-accept when only one candidate matches.
    pub choose_single: bool,
    /// Whether arrow navigation wraps around list edges.
    pub wrap_navigation: bool,
    /// Characters that dismiss the popup when typed.
    pub stop_chars: Vec<char>,
    /// Characters that accept the selection when typed.
    pub fill_up_chars: Vec<char>,
    /// Whether prefix-area completion is enabled.
    pub line_command_completion: bool,
    /// Whether accepting a candidate removes trailing text up to next word boundary.
    pub drop_rest_of_word: bool,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            trigger_mode: TriggerMode::Both,
            auto_trigger_chars: 2,
            matching_mode: MatchingMode::Prefix,
            case_sensitive: false,
            popup_max_items: 10,
            popup_max_width: 400,
            auto_hide: true,
            cancel_at_start_pos: true,
            choose_single: false,
            wrap_navigation: true,
            stop_chars: vec![' ', ';'],
            fill_up_chars: vec![],
            line_command_completion: true,
            drop_rest_of_word: false,
        }
    }
}

impl CompletionConfig {
    /// Clamps `popup_max_items` to the valid range [3, 50].
    pub fn clamp_popup_max_items(value: i64) -> u8 {
        value.clamp(3, 50) as u8
    }

    /// Clamps `popup_max_width` to the valid range [100, 1000].
    pub fn clamp_popup_max_width(value: i64) -> u16 {
        value.clamp(100, 1000) as u16
    }

    /// Clamps `auto_trigger_chars` to the valid range [1, 10].
    pub fn clamp_auto_trigger_chars(value: i64) -> u8 {
        value.clamp(1, 10) as u8
    }

    /// Creates a `CompletionConfig` from raw configuration values with validation.
    ///
    /// Invalid or out-of-range values are clamped/defaulted and errors are collected.
    ///
    /// # Errors
    ///
    /// Returns a vec of `CompletionError::InvalidConfig` for each invalid value,
    /// but always produces a usable config (with fallbacks applied).
    pub fn from_raw_values(values: &RawConfigValues) -> (Self, Vec<CompletionError>) {
        let mut errors = Vec::new();
        let defaults = Self::default();

        let trigger_mode = match &values.trigger_mode {
            Some(s) => match TriggerMode::from_str_with_fallback(s) {
                Ok(m) => m,
                Err(e) => {
                    errors.push(e);
                    defaults.trigger_mode
                }
            },
            None => defaults.trigger_mode,
        };

        let matching_mode = match &values.matching_mode {
            Some(s) => match MatchingMode::from_str_with_fallback(s) {
                Ok(m) => m,
                Err(e) => {
                    errors.push(e);
                    defaults.matching_mode
                }
            },
            None => defaults.matching_mode,
        };

        let auto_trigger_chars = values
            .auto_trigger_chars
            .map(Self::clamp_auto_trigger_chars)
            .unwrap_or(defaults.auto_trigger_chars);

        let popup_max_items = values
            .popup_max_items
            .map(Self::clamp_popup_max_items)
            .unwrap_or(defaults.popup_max_items);

        let popup_max_width = values
            .popup_max_width
            .map(Self::clamp_popup_max_width)
            .unwrap_or(defaults.popup_max_width);

        let config = Self {
            trigger_mode,
            auto_trigger_chars,
            matching_mode,
            case_sensitive: values.case_sensitive.unwrap_or(defaults.case_sensitive),
            popup_max_items,
            popup_max_width,
            auto_hide: values.auto_hide.unwrap_or(defaults.auto_hide),
            cancel_at_start_pos: values
                .cancel_at_start_pos
                .unwrap_or(defaults.cancel_at_start_pos),
            choose_single: values.choose_single.unwrap_or(defaults.choose_single),
            wrap_navigation: values.wrap_navigation.unwrap_or(defaults.wrap_navigation),
            stop_chars: values.stop_chars.clone().unwrap_or(defaults.stop_chars),
            fill_up_chars: values
                .fill_up_chars
                .clone()
                .unwrap_or(defaults.fill_up_chars),
            line_command_completion: values
                .line_command_completion
                .unwrap_or(defaults.line_command_completion),
            drop_rest_of_word: values
                .drop_rest_of_word
                .unwrap_or(defaults.drop_rest_of_word),
        };

        (config, errors)
    }
}

/// Raw configuration values before validation.
///
/// Used as an intermediate representation when loading from the configuration system.
/// All fields are optional — missing values get defaults.
#[derive(Debug, Clone, Default)]
pub struct RawConfigValues {
    /// Raw trigger mode string.
    pub trigger_mode: Option<String>,
    /// Raw auto-trigger character count.
    pub auto_trigger_chars: Option<i64>,
    /// Raw matching mode string.
    pub matching_mode: Option<String>,
    /// Raw case sensitivity flag.
    pub case_sensitive: Option<bool>,
    /// Raw popup max items count.
    pub popup_max_items: Option<i64>,
    /// Raw popup max width.
    pub popup_max_width: Option<i64>,
    /// Raw auto-hide flag.
    pub auto_hide: Option<bool>,
    /// Raw cancel-at-start-pos flag.
    pub cancel_at_start_pos: Option<bool>,
    /// Raw choose-single flag.
    pub choose_single: Option<bool>,
    /// Raw wrap-navigation flag.
    pub wrap_navigation: Option<bool>,
    /// Raw stop chars.
    pub stop_chars: Option<Vec<char>>,
    /// Raw fill-up chars.
    pub fill_up_chars: Option<Vec<char>>,
    /// Raw line-command-completion flag.
    pub line_command_completion: Option<bool>,
    /// Raw drop-rest-of-word flag.
    pub drop_rest_of_word: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 9.1 (defaults)
    #[test]
    fn default_config_has_documented_values() {
        let config = CompletionConfig::default();
        assert_eq!(config.trigger_mode, TriggerMode::Both);
        assert_eq!(config.auto_trigger_chars, 2);
        assert_eq!(config.matching_mode, MatchingMode::Prefix);
        assert!(!config.case_sensitive);
        assert_eq!(config.popup_max_items, 10);
        assert_eq!(config.popup_max_width, 400);
        assert!(config.auto_hide);
        assert!(config.cancel_at_start_pos);
        assert!(!config.choose_single);
        assert!(config.wrap_navigation);
        assert_eq!(config.stop_chars, vec![' ', ';']);
        assert!(config.fill_up_chars.is_empty());
        assert!(config.line_command_completion);
        assert!(!config.drop_rest_of_word);
    }

    // Validates: Requirement 9.5 (clamping)
    #[test]
    fn clamp_popup_max_items_respects_range() {
        assert_eq!(CompletionConfig::clamp_popup_max_items(0), 3);
        assert_eq!(CompletionConfig::clamp_popup_max_items(3), 3);
        assert_eq!(CompletionConfig::clamp_popup_max_items(25), 25);
        assert_eq!(CompletionConfig::clamp_popup_max_items(50), 50);
        assert_eq!(CompletionConfig::clamp_popup_max_items(100), 50);
        assert_eq!(CompletionConfig::clamp_popup_max_items(-5), 3);
    }

    #[test]
    fn clamp_popup_max_width_respects_range() {
        assert_eq!(CompletionConfig::clamp_popup_max_width(0), 100);
        assert_eq!(CompletionConfig::clamp_popup_max_width(100), 100);
        assert_eq!(CompletionConfig::clamp_popup_max_width(400), 400);
        assert_eq!(CompletionConfig::clamp_popup_max_width(1000), 1000);
        assert_eq!(CompletionConfig::clamp_popup_max_width(5000), 1000);
        assert_eq!(CompletionConfig::clamp_popup_max_width(-100), 100);
    }

    #[test]
    fn clamp_auto_trigger_chars_respects_range() {
        assert_eq!(CompletionConfig::clamp_auto_trigger_chars(0), 1);
        assert_eq!(CompletionConfig::clamp_auto_trigger_chars(1), 1);
        assert_eq!(CompletionConfig::clamp_auto_trigger_chars(5), 5);
        assert_eq!(CompletionConfig::clamp_auto_trigger_chars(10), 10);
        assert_eq!(CompletionConfig::clamp_auto_trigger_chars(20), 10);
        assert_eq!(CompletionConfig::clamp_auto_trigger_chars(-3), 1);
    }

    // Validates: Requirement 9.5 (invalid matching_mode)
    #[test]
    fn invalid_matching_mode_falls_back_to_prefix() {
        let raw = RawConfigValues {
            matching_mode: Some("invalid_mode".to_string()),
            ..Default::default()
        };
        let (config, errors) = CompletionConfig::from_raw_values(&raw);
        assert_eq!(config.matching_mode, MatchingMode::Prefix);
        assert_eq!(errors.len(), 1);
    }

    // Validates: Requirement 9.5 (invalid trigger_mode)
    #[test]
    fn invalid_trigger_mode_falls_back_to_both() {
        let raw = RawConfigValues {
            trigger_mode: Some("unknown".to_string()),
            ..Default::default()
        };
        let (config, errors) = CompletionConfig::from_raw_values(&raw);
        assert_eq!(config.trigger_mode, TriggerMode::Both);
        assert_eq!(errors.len(), 1);
    }

    // Validates: Requirement 9.5 (out of range values get clamped)
    #[test]
    fn out_of_range_values_are_clamped() {
        let raw = RawConfigValues {
            popup_max_items: Some(200),
            popup_max_width: Some(5000),
            auto_trigger_chars: Some(-5),
            ..Default::default()
        };
        let (config, errors) = CompletionConfig::from_raw_values(&raw);
        assert_eq!(config.popup_max_items, 50);
        assert_eq!(config.popup_max_width, 1000);
        assert_eq!(config.auto_trigger_chars, 1);
        assert!(errors.is_empty()); // clamping doesn't produce errors, only invalid enums do
    }

    #[test]
    fn valid_raw_values_produce_correct_config() {
        let raw = RawConfigValues {
            trigger_mode: Some("manual".to_string()),
            matching_mode: Some("fuzzy".to_string()),
            auto_trigger_chars: Some(3),
            case_sensitive: Some(true),
            popup_max_items: Some(20),
            popup_max_width: Some(600),
            auto_hide: Some(false),
            cancel_at_start_pos: Some(false),
            choose_single: Some(true),
            wrap_navigation: Some(false),
            stop_chars: Some(vec![' ']),
            fill_up_chars: Some(vec!['(', '.']),
            line_command_completion: Some(false),
            drop_rest_of_word: Some(true),
        };
        let (config, errors) = CompletionConfig::from_raw_values(&raw);
        assert!(errors.is_empty());
        assert_eq!(config.trigger_mode, TriggerMode::Manual);
        assert_eq!(config.matching_mode, MatchingMode::Fuzzy);
        assert_eq!(config.auto_trigger_chars, 3);
        assert!(config.case_sensitive);
        assert_eq!(config.popup_max_items, 20);
        assert_eq!(config.popup_max_width, 600);
        assert!(!config.auto_hide);
        assert!(!config.cancel_at_start_pos);
        assert!(config.choose_single);
        assert!(!config.wrap_navigation);
        assert_eq!(config.stop_chars, vec![' ']);
        assert_eq!(config.fill_up_chars, vec!['(', '.']);
        assert!(!config.line_command_completion);
        assert!(config.drop_rest_of_word);
    }
}
