//! Defaults loader — configuration and language integration.
//!
//! Loads default tab stops and mask from the configuration system and language
//! definitions at session initialization. Applies the precedence rules:
//! Language_Definition > global config > built-in every-8-columns.

use crate::mask::MaskLine;
use crate::state::{MaskState, TabStopSource, TabsMaskState, TabsState};
use crate::tab_stops::TabStopList;
use crate::traits::{ConfigProvider, LanguageDefinitionRef};

/// Loads default tab stops and mask at session initialization.
///
/// Applies precedence: Language_Definition > global config > built-in defaults.
///
/// Addresses: Requirements 4, 10, 13
pub struct DefaultsLoader;

impl DefaultsLoader {
    /// Loads tab stops for a new session.
    ///
    /// Precedence: language definition > global config > every-8-columns.
    ///
    /// Addresses: Requirement 4, criteria 4.1–4.7; Requirement 13, criterion 13.6
    pub fn load_tab_stops(
        config: &dyn ConfigProvider,
        language_def: Option<&LanguageDefinitionRef<'_>>,
        max_column: u32,
    ) -> (TabStopList, TabStopSource) {
        // 1. Try language definition first (highest precedence)
        if let Some(lang) = language_def {
            if let Some(stops) = lang.default_tab_stops() {
                if !stops.is_empty() {
                    return (
                        TabStopList::from_columns(stops),
                        TabStopSource::LanguageDefinition,
                    );
                }
            }
        }

        // 2. Try global config
        let config_stops = config.get_tab_stops();
        if !config_stops.is_empty() {
            return (
                TabStopList::from_columns(config_stops),
                TabStopSource::GlobalConfig,
            );
        }

        // 3. Fall back to every-8-columns
        (
            TabStopList::every_n_columns(8, max_column),
            TabStopSource::BuiltIn,
        )
    }

    /// Loads the insert mask for a new session.
    ///
    /// Precedence: language definition > no mask.
    ///
    /// Addresses: Requirement 10, criteria 10.1, 10.2
    pub fn load_mask(language_def: Option<&LanguageDefinitionRef<'_>>) -> MaskState {
        if let Some(lang) = language_def {
            if let Some(mask_value) = lang.default_mask() {
                if let Some(mask_line) = MaskManager::from_language_default(mask_value) {
                    return MaskState::with_mask(mask_line, true);
                }
            }
        }
        MaskState::empty()
    }

    /// Initializes the complete TabsMaskState for a new editing session.
    ///
    /// Addresses: Requirements 4, 10, 15
    pub fn init_session(
        config: &dyn ConfigProvider,
        language_def: Option<&LanguageDefinitionRef<'_>>,
        max_column: u32,
    ) -> TabsMaskState {
        let (tab_stops, source) = Self::load_tab_stops(config, language_def, max_column);
        let tabs_state = TabsState::new(tab_stops, source);
        let mask_state = Self::load_mask(language_def);
        TabsMaskState::new(tabs_state, mask_state)
    }
}

/// Manages insert mask operations: content access, line application, editing.
///
/// Addresses: Requirements 6, 7, 8, 9, 10, 16
pub struct MaskManager;

impl MaskManager {
    /// Applies the active mask to generate content for a newly inserted blank line.
    ///
    /// Returns the mask content padded/truncated to `line_width`, or `None` if no mask active.
    ///
    /// Addresses: Requirement 9, criteria 9.1, 9.3, 9.5, 9.6
    pub fn apply_mask(mask_state: &MaskState, line_width: usize) -> Option<String> {
        mask_state
            .mask()
            .map(|mask| mask.apply_to_width(line_width))
    }

    /// Applies the active mask to n newly inserted lines.
    ///
    /// Returns a Vec of n line contents, or empty vec if no mask active.
    ///
    /// Addresses: Requirement 9, criterion 9.2
    pub fn apply_mask_to_n_lines(
        mask_state: &MaskState,
        line_width: usize,
        count: usize,
    ) -> Vec<String> {
        match mask_state.mask() {
            Some(mask) => {
                let content = mask.apply_to_width(line_width);
                vec![content; count]
            }
            None => Vec::new(),
        }
    }

    /// Validates and creates a MaskLine from a language definition `default_mask` value.
    ///
    /// Returns `None` if the value is not a valid string.
    ///
    /// Addresses: Requirement 10, criteria 10.3, 10.6
    pub fn from_language_default(value: &toml::Value) -> Option<MaskLine> {
        value.as_str().map(MaskLine::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test ConfigProvider that returns configured values.
    struct TestConfig {
        tab_stops: Vec<u32>,
        tab_size: u32,
    }

    impl ConfigProvider for TestConfig {
        fn get_tab_stops(&self) -> Vec<u32> {
            self.tab_stops.clone()
        }

        fn get_tab_size(&self) -> u32 {
            self.tab_size
        }
    }

    #[test]
    fn load_tab_stops_language_def_takes_precedence() {
        // Validates: Requirement 4.3, 13.6
        let config = TestConfig {
            tab_stops: vec![9, 17, 25],
            tab_size: 8,
        };
        let toml_val: toml::Value = toml::from_str(r#"default_tab_stops = [7, 12, 72]"#).unwrap();
        let lang_def = LanguageDefinitionRef::new(&toml_val);

        let (stops, source) = DefaultsLoader::load_tab_stops(&config, Some(&lang_def), 80);
        assert_eq!(stops, TabStopList::from_columns(vec![7, 12, 72]));
        assert_eq!(source, TabStopSource::LanguageDefinition);
    }

    #[test]
    fn load_tab_stops_global_config_used_when_no_language_def() {
        // Validates: Requirement 4.4
        let config = TestConfig {
            tab_stops: vec![9, 17, 25],
            tab_size: 8,
        };

        let (stops, source) = DefaultsLoader::load_tab_stops(&config, None, 80);
        assert_eq!(stops, TabStopList::from_columns(vec![9, 17, 25]));
        assert_eq!(source, TabStopSource::GlobalConfig);
    }

    #[test]
    fn load_tab_stops_every_8_columns_fallback() {
        // Validates: Requirement 4.2
        let config = TestConfig {
            tab_stops: vec![],
            tab_size: 8,
        };

        let (stops, source) = DefaultsLoader::load_tab_stops(&config, None, 80);
        assert_eq!(stops, TabStopList::every_n_columns(8, 80));
        assert_eq!(source, TabStopSource::BuiltIn);
    }

    #[test]
    fn load_mask_from_language_definition() {
        // Validates: Requirement 10.1
        let toml_val: toml::Value = toml::from_str(r#"default_mask = "      *""#).unwrap();
        let lang_def = LanguageDefinitionRef::new(&toml_val);

        let state = DefaultsLoader::load_mask(Some(&lang_def));
        assert!(state.is_active());
        assert_eq!(state.mask().unwrap().content(), "      *");
        assert!(state.from_language());
    }

    #[test]
    fn load_mask_no_language_definition_returns_empty() {
        // Validates: Requirement 10.2
        let state = DefaultsLoader::load_mask(None);
        assert!(!state.is_active());
    }

    #[test]
    fn load_mask_non_string_value_returns_empty() {
        // Validates: Requirement 10.6
        let toml_val: toml::Value = toml::from_str("default_mask = 42").unwrap();
        let lang_def = LanguageDefinitionRef::new(&toml_val);

        let state = DefaultsLoader::load_mask(Some(&lang_def));
        assert!(!state.is_active());
    }

    #[test]
    fn init_session_combines_tab_stops_and_mask() {
        // Validates: Requirement 4, 10, 15
        let config = TestConfig {
            tab_stops: vec![],
            tab_size: 8,
        };
        let toml_val: toml::Value =
            toml::from_str("default_tab_stops = [7, 12, 72]\ndefault_mask = \"      *\"").unwrap();
        let lang_def = LanguageDefinitionRef::new(&toml_val);

        let state = DefaultsLoader::init_session(&config, Some(&lang_def), 80);
        assert_eq!(
            state.tabs().tab_stops(),
            &TabStopList::from_columns(vec![7, 12, 72])
        );
        assert!(state.mask().is_active());
    }

    #[test]
    fn apply_mask_with_active_mask_returns_padded_content() {
        // Validates: Requirement 9.1, 9.5
        let state = MaskState::with_mask(MaskLine::new("ABC"), false);
        let result = MaskManager::apply_mask(&state, 8);
        assert_eq!(result, Some("ABC     ".to_string()));
    }

    #[test]
    fn apply_mask_no_active_mask_returns_none() {
        // Validates: Requirement 9.3
        let state = MaskState::empty();
        let result = MaskManager::apply_mask(&state, 8);
        assert_eq!(result, None);
    }

    #[test]
    fn apply_mask_to_n_lines_produces_n_copies() {
        // Validates: Requirement 9.2
        let state = MaskState::with_mask(MaskLine::new("XY"), false);
        let lines = MaskManager::apply_mask_to_n_lines(&state, 5, 3);
        assert_eq!(lines.len(), 3);
        for line in &lines {
            assert_eq!(line, "XY   ");
        }
    }

    #[test]
    fn apply_mask_to_n_lines_no_mask_returns_empty_vec() {
        let state = MaskState::empty();
        let lines = MaskManager::apply_mask_to_n_lines(&state, 5, 3);
        assert!(lines.is_empty());
    }

    #[test]
    fn from_language_default_valid_string() {
        // Validates: Requirement 10.3
        let val = toml::Value::String("      *".to_string());
        let mask = MaskManager::from_language_default(&val);
        assert!(mask.is_some());
        assert_eq!(mask.unwrap().content(), "      *");
    }

    #[test]
    fn from_language_default_non_string_returns_none() {
        // Validates: Requirement 10.6
        let val = toml::Value::Integer(42);
        let mask = MaskManager::from_language_default(&val);
        assert!(mask.is_none());
    }

    #[test]
    fn language_def_with_invalid_tab_stops_filters_them() {
        // Validates: Requirement 4.6
        let toml_val: toml::Value =
            toml::from_str("default_tab_stops = [0, 5, -3, 10, 5]").unwrap();
        let lang_def = LanguageDefinitionRef::new(&toml_val);
        let stops = lang_def.default_tab_stops().unwrap();
        // Only positive values pass through (0 and negative filtered by the trait)
        // Note: TOML integers can be negative, but our filter is > 0
        assert!(stops.contains(&5));
        assert!(stops.contains(&10));
        assert!(!stops.contains(&0));
    }
}
