//! Viewer configuration — TOML `[viewers]` section parsing and validation.
//!
//! Handles the workbench configuration for the viewer framework, including
//! auto-offer, default position, split ratio, and debounce interval.

use crate::error::ViewerError;
use crate::panel::ViewerPosition;

/// Default auto_offer value.
pub const DEFAULT_AUTO_OFFER: bool = true;
/// Default split ratio.
pub const DEFAULT_SPLIT_RATIO: f32 = 0.5;
/// Default debounce interval in milliseconds.
pub const DEFAULT_REFRESH_DEBOUNCE_MS: u64 = 300;
/// Minimum valid split ratio.
pub const MIN_SPLIT_RATIO: f32 = 0.1;
/// Maximum valid split ratio.
pub const MAX_SPLIT_RATIO: f32 = 0.9;
/// Minimum valid debounce interval.
pub const MIN_DEBOUNCE_MS: u64 = 50;
/// Maximum valid debounce interval.
pub const MAX_DEBOUNCE_MS: u64 = 5000;

/// Parsed representation of the `[viewers]` TOML configuration section.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewerConfig {
    /// Whether to display auto-detection notifications. Default: true.
    pub auto_offer: bool,
    /// Where the ViewerPanel opens relative to the editor. Default: SplitRight.
    pub default_position: ViewerPosition,
    /// Split ratio (viewer fraction) for split positions. Default: 0.5.
    pub split_ratio: f32,
    /// Debounce interval in milliseconds for viewer refresh. Default: 300.
    pub refresh_debounce_ms: u64,
    /// Warnings produced during config parsing.
    pub warnings: Vec<String>,
}

impl Default for ViewerConfig {
    fn default() -> Self {
        Self {
            auto_offer: DEFAULT_AUTO_OFFER,
            default_position: ViewerPosition::SplitRight,
            split_ratio: DEFAULT_SPLIT_RATIO,
            refresh_debounce_ms: DEFAULT_REFRESH_DEBOUNCE_MS,
            warnings: Vec::new(),
        }
    }
}

impl ViewerConfig {
    /// Parse a `ViewerConfig` from a TOML value representing the `[viewers]` section.
    ///
    /// Invalid values produce warnings and fall back to defaults. No errors
    /// are propagated — the config is always usable.
    pub fn from_toml(value: &toml::Value) -> Self {
        let mut config = Self::default();

        let table = match value.as_table() {
            Some(t) => t,
            None => {
                config
                    .warnings
                    .push("[viewers] section is not a table — using all defaults".to_string());
                return config;
            }
        };

        // auto_offer
        if let Some(v) = table.get("auto_offer") {
            match v.as_bool() {
                Some(b) => config.auto_offer = b,
                None => {
                    config.warnings.push(format!(
                        "[viewers] config: invalid value for key 'auto_offer' — expected boolean, using default {}",
                        DEFAULT_AUTO_OFFER
                    ));
                }
            }
        }

        // default_position
        if let Some(v) = table.get("default_position") {
            match v.as_str() {
                Some("split-right") => config.default_position = ViewerPosition::SplitRight,
                Some("split-bottom") => config.default_position = ViewerPosition::SplitBottom,
                Some("tab") => config.default_position = ViewerPosition::Tab,
                Some("float") => config.default_position = ViewerPosition::Float,
                _ => {
                    config.warnings.push(
                        "[viewers] config: invalid value for key 'default_position' — expected one of split-right, split-bottom, tab, float; using default 'split-right'".to_string()
                    );
                }
            }
        }

        // split_ratio
        if let Some(v) = table.get("split_ratio") {
            match v
                .as_float()
                .map(|f| f as f32)
                .or_else(|| v.as_integer().map(|i| i as f32))
            {
                Some(ratio) if (MIN_SPLIT_RATIO..=MAX_SPLIT_RATIO).contains(&ratio) => {
                    config.split_ratio = ratio;
                }
                Some(ratio) => {
                    // Clamp to valid range
                    config.split_ratio = ratio.clamp(MIN_SPLIT_RATIO, MAX_SPLIT_RATIO);
                    config.warnings.push(format!(
                        "[viewers] config: invalid value for key 'split_ratio' — {ratio} is outside valid range {MIN_SPLIT_RATIO}–{MAX_SPLIT_RATIO}, clamped to {}",
                        config.split_ratio
                    ));
                }
                None => {
                    config.warnings.push(format!(
                        "[viewers] config: invalid value for key 'split_ratio' — expected float, using default {DEFAULT_SPLIT_RATIO}"
                    ));
                }
            }
        }

        // refresh_debounce_ms
        if let Some(v) = table.get("refresh_debounce_ms") {
            match v.as_integer() {
                Some(ms)
                    if ms > 0 && (MIN_DEBOUNCE_MS..=MAX_DEBOUNCE_MS).contains(&(ms as u64)) =>
                {
                    config.refresh_debounce_ms = ms as u64;
                }
                Some(ms) if ms > 0 => {
                    // Clamp to valid range
                    config.refresh_debounce_ms =
                        (ms as u64).clamp(MIN_DEBOUNCE_MS, MAX_DEBOUNCE_MS);
                    config.warnings.push(format!(
                        "[viewers] config: invalid value for key 'refresh_debounce_ms' — {ms} is outside valid range {MIN_DEBOUNCE_MS}–{MAX_DEBOUNCE_MS}, clamped to {}",
                        config.refresh_debounce_ms
                    ));
                }
                _ => {
                    config.warnings.push(format!(
                        "[viewers] config: invalid value for key 'refresh_debounce_ms' — expected positive integer, using default {DEFAULT_REFRESH_DEBOUNCE_MS}"
                    ));
                }
            }
        }

        config
    }

    /// Validate the config and return any errors (for strict validation).
    pub fn validate(&self) -> Result<(), ViewerError> {
        if !(MIN_SPLIT_RATIO..=MAX_SPLIT_RATIO).contains(&self.split_ratio) {
            return Err(ViewerError::ConfigError {
                key: "split_ratio".to_string(),
                reason: format!(
                    "value {} is outside valid range {}–{}",
                    self.split_ratio, MIN_SPLIT_RATIO, MAX_SPLIT_RATIO
                ),
            });
        }

        if self.refresh_debounce_ms < MIN_DEBOUNCE_MS || self.refresh_debounce_ms > MAX_DEBOUNCE_MS
        {
            return Err(ViewerError::ConfigError {
                key: "refresh_debounce_ms".to_string(),
                reason: format!(
                    "value {} is outside valid range {}–{}",
                    self.refresh_debounce_ms, MIN_DEBOUNCE_MS, MAX_DEBOUNCE_MS
                ),
            });
        }

        Ok(())
    }
}

/// Parse per-viewer configuration from `[viewers.<key>]` TOML sections.
///
/// Returns the raw `toml::Value` for the viewer to consume via `configure()`.
pub fn get_per_viewer_config(
    viewers_section: &toml::Value,
    viewer_key: &str,
) -> Option<toml::Value> {
    viewers_section
        .as_table()
        .and_then(|t| t.get(viewer_key))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        // Validates: Requirement 10 AC 1
        let config = ViewerConfig::default();
        assert!(config.auto_offer);
        assert_eq!(config.default_position, ViewerPosition::SplitRight);
        assert!((config.split_ratio - 0.5).abs() < f32::EPSILON);
        assert_eq!(config.refresh_debounce_ms, 300);
        assert!(config.warnings.is_empty());
    }

    #[test]
    fn from_toml_parses_valid_config() {
        // Validates: Requirement 10 AC 1
        let toml_str = r#"
            auto_offer = false
            default_position = "split-bottom"
            split_ratio = 0.3
            refresh_debounce_ms = 500
        "#;
        let value: toml::Value = toml_str.parse().unwrap();
        let config = ViewerConfig::from_toml(&value);

        assert!(!config.auto_offer);
        assert_eq!(config.default_position, ViewerPosition::SplitBottom);
        assert!((config.split_ratio - 0.3).abs() < f32::EPSILON);
        assert_eq!(config.refresh_debounce_ms, 500);
        assert!(config.warnings.is_empty());
    }

    #[test]
    fn from_toml_invalid_auto_offer_uses_default() {
        // Validates: Requirement 10 AC 2
        let toml_str = r#"auto_offer = "not a bool""#;
        let value: toml::Value = toml_str.parse().unwrap();
        let config = ViewerConfig::from_toml(&value);

        assert!(config.auto_offer); // default
        assert_eq!(config.warnings.len(), 1);
        assert!(config.warnings[0].contains("auto_offer"));
    }

    #[test]
    fn from_toml_invalid_position_uses_default() {
        // Validates: Requirement 10 AC 2
        let toml_str = r#"default_position = "invalid""#;
        let value: toml::Value = toml_str.parse().unwrap();
        let config = ViewerConfig::from_toml(&value);

        assert_eq!(config.default_position, ViewerPosition::SplitRight);
        assert_eq!(config.warnings.len(), 1);
    }

    #[test]
    fn from_toml_split_ratio_out_of_range_clamped() {
        // Validates: Requirement 10 AC 2
        let toml_str = r#"split_ratio = 2.0"#;
        let value: toml::Value = toml_str.parse().unwrap();
        let config = ViewerConfig::from_toml(&value);

        assert!((config.split_ratio - MAX_SPLIT_RATIO).abs() < f32::EPSILON);
        assert_eq!(config.warnings.len(), 1);
        assert!(config.warnings[0].contains("split_ratio"));
    }

    #[test]
    fn from_toml_split_ratio_below_min_clamped() {
        // Validates: Requirement 10 AC 2
        let toml_str = r#"split_ratio = 0.01"#;
        let value: toml::Value = toml_str.parse().unwrap();
        let config = ViewerConfig::from_toml(&value);

        assert!((config.split_ratio - MIN_SPLIT_RATIO).abs() < f32::EPSILON);
        assert_eq!(config.warnings.len(), 1);
    }

    #[test]
    fn from_toml_negative_debounce_uses_default() {
        // Validates: Requirement 10 AC 2
        let toml_str = r#"refresh_debounce_ms = -100"#;
        let value: toml::Value = toml_str.parse().unwrap();
        let config = ViewerConfig::from_toml(&value);

        assert_eq!(config.refresh_debounce_ms, DEFAULT_REFRESH_DEBOUNCE_MS);
        assert_eq!(config.warnings.len(), 1);
    }

    #[test]
    fn from_toml_debounce_above_max_clamped() {
        // Validates: Requirement 10 AC 2
        let toml_str = r#"refresh_debounce_ms = 10000"#;
        let value: toml::Value = toml_str.parse().unwrap();
        let config = ViewerConfig::from_toml(&value);

        assert_eq!(config.refresh_debounce_ms, MAX_DEBOUNCE_MS);
        assert_eq!(config.warnings.len(), 1);
    }

    #[test]
    fn from_toml_all_positions_valid() {
        for (input, expected) in [
            ("split-right", ViewerPosition::SplitRight),
            ("split-bottom", ViewerPosition::SplitBottom),
            ("tab", ViewerPosition::Tab),
            ("float", ViewerPosition::Float),
        ] {
            let toml_str = format!(r#"default_position = "{input}""#);
            let value: toml::Value = toml_str.parse().unwrap();
            let config = ViewerConfig::from_toml(&value);
            assert_eq!(config.default_position, expected);
            assert!(config.warnings.is_empty());
        }
    }

    #[test]
    fn per_viewer_config_extraction() {
        // Validates: Requirement 10 AC 4
        let toml_str = r#"
            [asa-report]
            page_break_style = "line"

            [csv-table]
            delimiter = "\t"
        "#;
        let value: toml::Value = toml_str.parse().unwrap();

        let asa_config = get_per_viewer_config(&value, "asa-report");
        assert!(asa_config.is_some());
        let asa_table = asa_config.unwrap();
        assert_eq!(
            asa_table.get("page_break_style").unwrap().as_str().unwrap(),
            "line"
        );

        let missing = get_per_viewer_config(&value, "hex");
        assert!(missing.is_none());
    }

    #[test]
    fn validate_accepts_valid_config() {
        let config = ViewerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_rejects_out_of_range_split_ratio() {
        let config = ViewerConfig {
            split_ratio: 1.5,
            ..ViewerConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
