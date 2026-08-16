//! Help subsystem configuration.
//!
//! Typed configuration loaded from the `[help]` TOML section.

use crate::error::HelpError;

/// The configured dock zone for the Help Panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpPanelPosition {
    /// Dock to the right side (default).
    Right,
    /// Dock to the left side.
    Left,
    /// Dock to the bottom.
    Bottom,
}

impl HelpPanelPosition {
    /// Parse a string into a `HelpPanelPosition`.
    ///
    /// Accepts `"right"`, `"left"`, `"bottom"` (case-insensitive).
    /// Returns `None` for unrecognised values.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "right" => Some(Self::Right),
            "left" => Some(Self::Left),
            "bottom" => Some(Self::Bottom),
            _ => None,
        }
    }
}

/// Typed configuration for the help subsystem, loaded from `[help]` TOML section.
///
/// # Fields
///
/// - `directory` — Custom path to help content directory. `None` uses default search locations.
/// - `panel_width_ratio` — Help Panel width as fraction of window (0.2–0.5, default 0.35).
/// - `panel_position` — Default dock zone for the Help Panel.
/// - `search_highlight` — Whether to highlight search matches in content.
#[derive(Debug, Clone, PartialEq)]
pub struct HelpConfig {
    /// Custom path to help content directory. `None` uses default search locations.
    pub directory: Option<String>,
    /// Help Panel width as a fraction of window width (0.2–0.5, default 0.35).
    pub panel_width_ratio: f32,
    /// Default dock zone for the Help Panel.
    pub panel_position: HelpPanelPosition,
    /// Whether to highlight search matches in help content.
    pub search_highlight: bool,
}

impl Default for HelpConfig {
    fn default() -> Self {
        Self {
            directory: None,
            panel_width_ratio: 0.35,
            panel_position: HelpPanelPosition::Right,
            search_highlight: true,
        }
    }
}

impl HelpConfig {
    /// Validate and clamp the `panel_width_ratio` to the acceptable range (0.2–0.5).
    ///
    /// Returns `Ok(())` if valid, or `Err(HelpError::ConfigInvalid)` with the
    /// reason if clamping was applied.
    pub fn validate_panel_width_ratio(&mut self) -> Result<(), HelpError> {
        if self.panel_width_ratio < 0.2 || self.panel_width_ratio > 0.5 {
            let original = self.panel_width_ratio;
            self.panel_width_ratio = 0.35;
            return Err(HelpError::ConfigInvalid {
                key: "help.panel_width_ratio".to_string(),
                reason: format!("value {original} outside valid range 0.2–0.5"),
            });
        }
        Ok(())
    }

    /// Validate the `panel_position` from a raw string, applying default on failure.
    ///
    /// Returns `Ok(())` if valid, or `Err(HelpError::ConfigInvalid)` if the
    /// value was unrecognised and the default was applied.
    pub fn validate_panel_position_from_str(&mut self, raw: &str) -> Result<(), HelpError> {
        match HelpPanelPosition::from_str_opt(raw) {
            Some(pos) => {
                self.panel_position = pos;
                Ok(())
            }
            None => {
                self.panel_position = HelpPanelPosition::Right;
                Err(HelpError::ConfigInvalid {
                    key: "help.panel_position".to_string(),
                    reason: format!("unrecognised value \"{raw}\", expected right/left/bottom"),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 16.1 — Default configuration values
    #[test]
    fn default_config_has_expected_values() {
        let config = HelpConfig::default();
        assert_eq!(config.directory, None);
        assert!((config.panel_width_ratio - 0.35).abs() < f32::EPSILON);
        assert_eq!(config.panel_position, HelpPanelPosition::Right);
        assert!(config.search_highlight);
    }

    // Validates: Requirement 16.2 — Width ratio validation rejects out-of-range
    #[test]
    fn validate_panel_width_ratio_rejects_too_small() {
        let mut config = HelpConfig {
            panel_width_ratio: 0.1,
            ..Default::default()
        };
        let result = config.validate_panel_width_ratio();
        assert!(result.is_err());
        assert!((config.panel_width_ratio - 0.35).abs() < f32::EPSILON);
    }

    // Validates: Requirement 16.2 — Width ratio validation rejects out-of-range
    #[test]
    fn validate_panel_width_ratio_rejects_too_large() {
        let mut config = HelpConfig {
            panel_width_ratio: 0.8,
            ..Default::default()
        };
        let result = config.validate_panel_width_ratio();
        assert!(result.is_err());
        assert!((config.panel_width_ratio - 0.35).abs() < f32::EPSILON);
    }

    // Validates: Requirement 16.2 — Width ratio validation accepts valid range
    #[test]
    fn validate_panel_width_ratio_accepts_valid() {
        let mut config = HelpConfig {
            panel_width_ratio: 0.4,
            ..Default::default()
        };
        let result = config.validate_panel_width_ratio();
        assert!(result.is_ok());
        assert!((config.panel_width_ratio - 0.4).abs() < f32::EPSILON);
    }

    // Validates: Requirement 16.2 — Position validation rejects unknown string
    #[test]
    fn validate_panel_position_rejects_unknown() {
        let mut config = HelpConfig::default();
        let result = config.validate_panel_position_from_str("top");
        assert!(result.is_err());
        assert_eq!(config.panel_position, HelpPanelPosition::Right);
    }

    // Validates: Requirement 16.2 — Position validation accepts known strings
    #[test]
    fn validate_panel_position_accepts_valid_strings() {
        let mut config = HelpConfig::default();

        config.validate_panel_position_from_str("left").unwrap();
        assert_eq!(config.panel_position, HelpPanelPosition::Left);

        config.validate_panel_position_from_str("bottom").unwrap();
        assert_eq!(config.panel_position, HelpPanelPosition::Bottom);

        config.validate_panel_position_from_str("Right").unwrap();
        assert_eq!(config.panel_position, HelpPanelPosition::Right);
    }
}
