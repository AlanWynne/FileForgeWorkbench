//! Zoom configuration model.
//!
//! Defines [`ZoomConfig`] which holds validated settings for the zoom
//! subsystem: step size, min/max offset range, and default offset.
//! Configuration is loaded from the `[view.zoom]` TOML namespace.

/// Configuration for the zoom subsystem.
///
/// All values are validated and clamped on construction. Invalid values
/// emit [`ConfigWarning`] diagnostics.
///
/// # Defaults
///
/// | Key | Default | Valid Range |
/// |-----|---------|-------------|
/// | `default_offset` | 0 | [min_offset, max_offset] |
/// | `step` | 1 | 1–10 |
/// | `min_offset` | -10 | -20 to 0 |
/// | `max_offset` | 60 | 0 to 100 |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoomConfig {
    /// Initial zoom offset for new editor instances.
    pub default_offset: i32,
    /// Points added/removed per zoom in/out operation.
    pub step: u32,
    /// Minimum permitted zoom offset.
    pub min_offset: i32,
    /// Maximum permitted zoom offset.
    pub max_offset: i32,
}

impl Default for ZoomConfig {
    fn default() -> Self {
        Self {
            default_offset: 0,
            step: 1,
            min_offset: -10,
            max_offset: 60,
        }
    }
}

impl ZoomConfig {
    /// Validate and normalise the config in place.
    ///
    /// Returns warnings for any values that were clamped or reset.
    ///
    /// # Validation Rules
    ///
    /// - `step` clamped to [1, 10]
    /// - `min_offset` clamped to [-20, 0]
    /// - `max_offset` clamped to [0, 100]
    /// - If `min_offset >= max_offset`, both reset to defaults (-10, 60)
    /// - `default_offset` clamped to [min_offset, max_offset]
    pub fn validate(&mut self) -> Vec<ConfigWarning> {
        let mut warnings = Vec::new();

        // Validate step
        if self.step < 1 || self.step > 10 {
            warnings.push(ConfigWarning {
                key: "view.zoom.step".to_string(),
                message: format!(
                    "step value {} is out of range [1, 10], clamped to {}",
                    self.step,
                    self.step.clamp(1, 10)
                ),
            });
            self.step = self.step.clamp(1, 10);
        }

        // Validate min_offset
        if self.min_offset < -20 || self.min_offset > 0 {
            warnings.push(ConfigWarning {
                key: "view.zoom.min_offset".to_string(),
                message: format!(
                    "min_offset value {} is out of range [-20, 0], clamped to {}",
                    self.min_offset,
                    self.min_offset.clamp(-20, 0)
                ),
            });
            self.min_offset = self.min_offset.clamp(-20, 0);
        }

        // Validate max_offset
        if self.max_offset < 0 || self.max_offset > 100 {
            warnings.push(ConfigWarning {
                key: "view.zoom.max_offset".to_string(),
                message: format!(
                    "max_offset value {} is out of range [0, 100], clamped to {}",
                    self.max_offset,
                    self.max_offset.clamp(0, 100)
                ),
            });
            self.max_offset = self.max_offset.clamp(0, 100);
        }

        // Validate min < max
        if self.min_offset >= self.max_offset {
            warnings.push(ConfigWarning {
                key: "view.zoom.min_offset/max_offset".to_string(),
                message: format!(
                    "min_offset ({}) must be less than max_offset ({}), using defaults (-10, 60)",
                    self.min_offset, self.max_offset
                ),
            });
            self.min_offset = -10;
            self.max_offset = 60;
        }

        // Validate default_offset within range
        let clamped_default = self.default_offset.clamp(self.min_offset, self.max_offset);
        if clamped_default != self.default_offset {
            warnings.push(ConfigWarning {
                key: "view.zoom.default_offset".to_string(),
                message: format!(
                    "default_offset {} is outside [{}, {}], clamped to {}",
                    self.default_offset, self.min_offset, self.max_offset, clamped_default
                ),
            });
            self.default_offset = clamped_default;
        }

        warnings
    }

    /// Create a validated config from raw configuration values.
    ///
    /// Missing values use defaults; invalid values are clamped with warnings.
    pub fn from_raw(raw: RawZoomConfig) -> (Self, Vec<ConfigWarning>) {
        let mut config = Self {
            default_offset: raw.default_offset.unwrap_or(0) as i32,
            step: raw.step.unwrap_or(1) as u32,
            min_offset: raw.min_offset.unwrap_or(-10) as i32,
            max_offset: raw.max_offset.unwrap_or(60) as i32,
        };
        let warnings = config.validate();
        (config, warnings)
    }

    /// Simulate hot-reload: parse new raw values and re-validate.
    pub fn on_config_changed(raw: RawZoomConfig) -> (Self, Vec<ConfigWarning>) {
        Self::from_raw(raw)
    }
}

/// Raw configuration values before validation (direct from TOML parse).
#[derive(Debug, Clone, Default)]
pub struct RawZoomConfig {
    /// The raw default_offset value, if present.
    pub default_offset: Option<i64>,
    /// The raw step value, if present.
    pub step: Option<i64>,
    /// The raw min_offset value, if present.
    pub min_offset: Option<i64>,
    /// The raw max_offset value, if present.
    pub max_offset: Option<i64>,
}

/// A configuration validation warning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigWarning {
    /// The configuration key that triggered the warning.
    pub key: String,
    /// A human-readable description of the issue and resolution.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 4.1 — default config values
    #[test]
    fn default_config_has_correct_values() {
        let config = ZoomConfig::default();
        assert_eq!(config.default_offset, 0);
        assert_eq!(config.step, 1);
        assert_eq!(config.min_offset, -10);
        assert_eq!(config.max_offset, 60);
    }

    // Validates: Requirement 4.4 — step clamped to [1, 10]
    #[test]
    fn validate_clamps_step_too_high() {
        let mut config = ZoomConfig {
            step: 20,
            ..Default::default()
        };
        let warnings = config.validate();
        assert_eq!(config.step, 10);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].key.contains("step"));
    }

    // Validates: Requirement 4.4 — step clamped to [1, 10]
    #[test]
    fn validate_clamps_step_zero() {
        let mut config = ZoomConfig {
            step: 0,
            ..Default::default()
        };
        let warnings = config.validate();
        assert_eq!(config.step, 1);
        assert_eq!(warnings.len(), 1);
    }

    // Validates: Requirement 4.2 — min >= max resets both to defaults
    #[test]
    fn validate_resets_when_min_equals_max() {
        let mut config = ZoomConfig {
            min_offset: 0,
            max_offset: 0,
            ..Default::default()
        };
        let warnings = config.validate();
        assert_eq!(config.min_offset, -10);
        assert_eq!(config.max_offset, 60);
        assert!(warnings
            .iter()
            .any(|w| w.key.contains("min_offset/max_offset")));
    }

    // Validates: Requirement 4.2 — min > max resets both
    #[test]
    fn validate_resets_when_min_greater_than_max() {
        // After clamping, min_offset=0 and max_offset=0 → min >= max → reset
        let mut config = ZoomConfig {
            min_offset: 0,
            max_offset: 0,
            step: 1,
            default_offset: 0,
        };
        let warnings = config.validate();
        assert_eq!(config.min_offset, -10);
        assert_eq!(config.max_offset, 60);
        assert!(!warnings.is_empty());
    }

    // Validates: Requirement 4.3 — default_offset clamped to [min, max]
    #[test]
    fn validate_clamps_default_offset_above_max() {
        let mut config = ZoomConfig {
            default_offset: 70,
            ..Default::default()
        };
        let warnings = config.validate();
        assert_eq!(config.default_offset, 60);
        assert!(warnings.iter().any(|w| w.key.contains("default_offset")));
    }

    // Validates: Requirement 4.3 — default_offset clamped to [min, max]
    #[test]
    fn validate_clamps_default_offset_below_min() {
        let mut config = ZoomConfig {
            default_offset: -15,
            ..Default::default()
        };
        let warnings = config.validate();
        assert_eq!(config.default_offset, -10);
        assert!(warnings.iter().any(|w| w.key.contains("default_offset")));
    }

    // Validates: Requirement 4.1 — from_raw uses defaults for missing values
    #[test]
    fn from_raw_uses_defaults_when_absent() {
        let raw = RawZoomConfig::default();
        let (config, warnings) = ZoomConfig::from_raw(raw);
        assert_eq!(config, ZoomConfig::default());
        assert!(warnings.is_empty());
    }

    // Validates: Requirement 4.5 — invalid type results in default with warning
    #[test]
    fn from_raw_with_valid_overrides() {
        let raw = RawZoomConfig {
            default_offset: Some(5),
            step: Some(2),
            min_offset: Some(-15),
            max_offset: Some(80),
        };
        let (config, warnings) = ZoomConfig::from_raw(raw);
        assert_eq!(config.default_offset, 5);
        assert_eq!(config.step, 2);
        assert_eq!(config.min_offset, -15);
        assert_eq!(config.max_offset, 80);
        assert!(warnings.is_empty());
    }

    // Validates: Requirement 4.6 — hot-reload applies new limits
    #[test]
    fn on_config_changed_validates_new_values() {
        let raw = RawZoomConfig {
            step: Some(15), // out of range
            ..Default::default()
        };
        let (config, warnings) = ZoomConfig::on_config_changed(raw);
        assert_eq!(config.step, 10);
        assert!(!warnings.is_empty());
    }

    // Validates: Requirement 4.1 — min_offset range validation
    #[test]
    fn validate_clamps_min_offset_too_low() {
        let mut config = ZoomConfig {
            min_offset: -30,
            ..Default::default()
        };
        let warnings = config.validate();
        assert_eq!(config.min_offset, -20);
        assert!(warnings.iter().any(|w| w.key.contains("min_offset")));
    }

    // Validates: Requirement 4.1 — max_offset range validation
    #[test]
    fn validate_clamps_max_offset_too_high() {
        let mut config = ZoomConfig {
            max_offset: 150,
            ..Default::default()
        };
        let warnings = config.validate();
        assert_eq!(config.max_offset, 100);
        assert!(warnings.iter().any(|w| w.key.contains("max_offset")));
    }
}
