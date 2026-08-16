//! Configuration for the wrap subsystem.
//!
//! Loaded from the `[view.wrap]` TOML namespace in the configuration hierarchy.
//! Validates raw values and emits warnings for invalid/out-of-range settings.

use crate::boundary::WrapBoundary;
use crate::indent::WrapIndentMode;
use crate::mode::WrapMode;
use crate::visual_flags::WrapVisualFlags;

/// A configuration validation warning.
///
/// Emitted when a configuration value is invalid or out of range and
/// a default has been substituted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigWarning {
    /// The configuration key that had an issue.
    pub key: String,
    /// A human-readable description of the problem.
    pub message: String,
}

/// Raw configuration values before validation (direct from TOML parse).
///
/// All fields are optional — missing keys receive defaults during validation.
#[derive(Debug, Clone, Default)]
pub struct RawWrapConfig {
    /// Raw `default_mode` string value.
    pub default_mode: Option<String>,
    /// Raw `wrap_column` integer value.
    pub wrap_column: Option<i64>,
    /// Raw `indent_mode` string value.
    pub indent_mode: Option<String>,
    /// Raw `indent_amount` integer value.
    pub indent_amount: Option<i64>,
    /// Raw `visual_flags` string value.
    pub visual_flags: Option<String>,
}

/// Configuration for the wrap subsystem, loaded from `[view.wrap]` TOML namespace.
///
/// Addresses: Requirement 12 (Configuration Defaults)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapConfig {
    /// Initial WrapMode for new editor instances.
    /// Default: `WrapMode::None`.
    pub default_mode: WrapMode,

    /// Wrap boundary. `Viewport` for dynamic wrapping at text area width,
    /// `Column(n)` for fixed column wrapping.
    /// Default: `Viewport` (wrap_column = 0).
    pub wrap_column: WrapBoundary,

    /// Wrap indent mode for continuation lines.
    /// Default: `Fixed`.
    pub indent_mode: WrapIndentMode,

    /// Fixed indent amount in characters (used when `indent_mode` is `Fixed`).
    /// Valid range: 0–40. Default: 0.
    pub indent_amount: u8,

    /// Wrap visual flags (continuation markers).
    /// Default: `None`.
    pub visual_flags: WrapVisualFlags,
}

impl Default for WrapConfig {
    fn default() -> Self {
        Self {
            default_mode: WrapMode::None,
            wrap_column: WrapBoundary::Viewport,
            indent_mode: WrapIndentMode::Fixed,
            indent_amount: 0,
            visual_flags: WrapVisualFlags::None,
        }
    }
}

impl WrapConfig {
    /// Validate and normalise raw config values from TOML.
    ///
    /// Emits warnings for invalid values; applies defaults for missing/invalid keys.
    ///
    /// Addresses: Requirement 12 AC 1, AC 2
    pub fn from_raw(raw: RawWrapConfig) -> (Self, Vec<ConfigWarning>) {
        let mut warnings = Vec::new();
        let mut config = Self::default();

        // Parse default_mode
        if let Some(ref mode_str) = raw.default_mode {
            match mode_str.to_lowercase().as_str() {
                "none" => config.default_mode = WrapMode::None,
                "word" => config.default_mode = WrapMode::Word,
                "character" => config.default_mode = WrapMode::Character,
                _ => {
                    warnings.push(ConfigWarning {
                        key: "view.wrap.default_mode".to_string(),
                        message: format!(
                            "invalid value '{}' — expected 'none', 'word', or 'character'; using default 'none'",
                            mode_str
                        ),
                    });
                }
            }
        }

        // Parse wrap_column
        if let Some(col_val) = raw.wrap_column {
            let (boundary, valid) = WrapBoundary::from_column_value(col_val);
            config.wrap_column = boundary;
            if !valid {
                warnings.push(ConfigWarning {
                    key: "view.wrap.wrap_column".to_string(),
                    message: format!(
                        "value {} is out of range (0–10000) — using default (viewport)",
                        col_val
                    ),
                });
            }
        }

        // Parse indent_mode
        if let Some(ref mode_str) = raw.indent_mode {
            match mode_str.to_lowercase().as_str() {
                "fixed" => config.indent_mode = WrapIndentMode::Fixed,
                "same" => config.indent_mode = WrapIndentMode::Same,
                "indent" => config.indent_mode = WrapIndentMode::Indent,
                "deep_indent" => config.indent_mode = WrapIndentMode::DeepIndent,
                _ => {
                    warnings.push(ConfigWarning {
                        key: "view.wrap.indent_mode".to_string(),
                        message: format!(
                            "invalid value '{}' — expected 'fixed', 'same', 'indent', or 'deep_indent'; using default 'fixed'",
                            mode_str
                        ),
                    });
                }
            }
        }

        // Parse indent_amount
        if let Some(amount) = raw.indent_amount {
            if amount < 0 {
                config.indent_amount = 0;
                warnings.push(ConfigWarning {
                    key: "view.wrap.indent_amount".to_string(),
                    message: format!("value {} is out of range (0–40) — clamped to 0", amount),
                });
            } else if amount > 40 {
                config.indent_amount = 40;
                warnings.push(ConfigWarning {
                    key: "view.wrap.indent_amount".to_string(),
                    message: format!("value {} is out of range (0–40) — clamped to 40", amount),
                });
            } else {
                config.indent_amount = amount as u8;
            }
        }

        // Parse visual_flags
        if let Some(ref flags_str) = raw.visual_flags {
            match flags_str.to_lowercase().as_str() {
                "none" => config.visual_flags = WrapVisualFlags::None,
                "end" => config.visual_flags = WrapVisualFlags::End,
                "start" => config.visual_flags = WrapVisualFlags::Start,
                "start_end" => config.visual_flags = WrapVisualFlags::StartEnd,
                "margin" => config.visual_flags = WrapVisualFlags::Margin,
                _ => {
                    warnings.push(ConfigWarning {
                        key: "view.wrap.visual_flags".to_string(),
                        message: format!(
                            "invalid value '{}' — expected 'none', 'end', 'start', 'start_end', or 'margin'; using default 'none'",
                            flags_str
                        ),
                    });
                }
            }
        }

        (config, warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::WrapColumn;

    #[test]
    fn default_config_has_expected_values() {
        // Validates: Requirement 12.1
        let config = WrapConfig::default();
        assert_eq!(config.default_mode, WrapMode::None);
        assert_eq!(config.wrap_column, WrapBoundary::Viewport);
        assert_eq!(config.indent_mode, WrapIndentMode::Fixed);
        assert_eq!(config.indent_amount, 0);
        assert_eq!(config.visual_flags, WrapVisualFlags::None);
    }

    #[test]
    fn from_raw_with_all_defaults_produces_default_config() {
        let (config, warnings) = WrapConfig::from_raw(RawWrapConfig::default());
        assert_eq!(config, WrapConfig::default());
        assert!(warnings.is_empty());
    }

    #[test]
    fn from_raw_parses_valid_mode() {
        // Validates: Requirement 12.1
        let raw = RawWrapConfig {
            default_mode: Some("word".to_string()),
            ..Default::default()
        };
        let (config, warnings) = WrapConfig::from_raw(raw);
        assert_eq!(config.default_mode, WrapMode::Word);
        assert!(warnings.is_empty());
    }

    #[test]
    fn from_raw_invalid_mode_resets_to_default_with_warning() {
        // Validates: Requirement 12.2
        let raw = RawWrapConfig {
            default_mode: Some("turbo".to_string()),
            ..Default::default()
        };
        let (config, warnings) = WrapConfig::from_raw(raw);
        assert_eq!(config.default_mode, WrapMode::None);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].key.contains("default_mode"));
    }

    #[test]
    fn from_raw_valid_wrap_column() {
        let raw = RawWrapConfig {
            wrap_column: Some(80),
            ..Default::default()
        };
        let (config, warnings) = WrapConfig::from_raw(raw);
        assert_eq!(
            config.wrap_column,
            WrapBoundary::Column(WrapColumn::new(80).unwrap())
        );
        assert!(warnings.is_empty());
    }

    #[test]
    fn from_raw_negative_column_resets_to_viewport_with_warning() {
        // Validates: Requirement 4.7
        let raw = RawWrapConfig {
            wrap_column: Some(-5),
            ..Default::default()
        };
        let (config, warnings) = WrapConfig::from_raw(raw);
        assert_eq!(config.wrap_column, WrapBoundary::Viewport);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn from_raw_column_exceeds_max_resets_with_warning() {
        // Validates: Requirement 4.7
        let raw = RawWrapConfig {
            wrap_column: Some(10_001),
            ..Default::default()
        };
        let (config, warnings) = WrapConfig::from_raw(raw);
        assert_eq!(config.wrap_column, WrapBoundary::Viewport);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn from_raw_indent_amount_clamped_high() {
        // Validates: Requirement 5.8
        let raw = RawWrapConfig {
            indent_amount: Some(50),
            ..Default::default()
        };
        let (config, warnings) = WrapConfig::from_raw(raw);
        assert_eq!(config.indent_amount, 40);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn from_raw_indent_amount_clamped_low() {
        // Validates: Requirement 5.8
        let raw = RawWrapConfig {
            indent_amount: Some(-3),
            ..Default::default()
        };
        let (config, warnings) = WrapConfig::from_raw(raw);
        assert_eq!(config.indent_amount, 0);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn from_raw_valid_indent_mode() {
        let raw = RawWrapConfig {
            indent_mode: Some("deep_indent".to_string()),
            ..Default::default()
        };
        let (config, warnings) = WrapConfig::from_raw(raw);
        assert_eq!(config.indent_mode, WrapIndentMode::DeepIndent);
        assert!(warnings.is_empty());
    }

    #[test]
    fn from_raw_invalid_visual_flags_resets_with_warning() {
        let raw = RawWrapConfig {
            visual_flags: Some("rainbow".to_string()),
            ..Default::default()
        };
        let (config, warnings) = WrapConfig::from_raw(raw);
        assert_eq!(config.visual_flags, WrapVisualFlags::None);
        assert_eq!(warnings.len(), 1);
    }
}
