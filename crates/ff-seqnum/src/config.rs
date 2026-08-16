//! Configuration for the sequence numbers subsystem.
//!
//! Provides `SeqNumConfig` which holds typed, validated access to all
//! `editor.sequence_numbers.*` settings from the configuration system.

use crate::types::{ColumnRange, SequenceFormat};

/// Typed representation of all `editor.sequence_numbers.*` configuration keys.
///
/// Values are validated and clamped to their legal ranges on construction.
/// Out-of-range values are clamped and logged at WARN level.
#[derive(Debug, Clone)]
pub struct SeqNumConfig {
    /// Detection threshold percentage (50–100, default 80).
    /// Minimum percentage of sampled lines that must match numeric pattern.
    pub detection_threshold: u8,
    /// Sample size — maximum non-blank lines to sample (5–100, default 20).
    pub sample_size: u8,
    /// Whether to highlight sequence columns with background shading.
    pub highlight_columns: bool,
    /// Default sequence number format.
    pub default_format: SequenceFormat,
    /// Whether to restore sequence numbers on save.
    pub restore_on_save: bool,
}

impl Default for SeqNumConfig {
    fn default() -> Self {
        Self {
            detection_threshold: 80,
            sample_size: 20,
            highlight_columns: false,
            default_format: SequenceFormat::Numeric,
            restore_on_save: false,
        }
    }
}

impl SeqNumConfig {
    /// Create a new config with all defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clamp detection_threshold to the valid range [50, 100].
    /// Returns the clamped value and whether clamping occurred.
    pub fn clamp_threshold(value: u8) -> (u8, bool) {
        if value < 50 {
            (50, true)
        } else if value > 100 {
            (100, true)
        } else {
            (value, false)
        }
    }

    /// Clamp sample_size to the valid range [5, 100].
    /// Returns the clamped value and whether clamping occurred.
    pub fn clamp_sample_size(value: u8) -> (u8, bool) {
        if value < 5 {
            (5, true)
        } else if value > 100 {
            (100, true)
        } else {
            (value, false)
        }
    }

    /// Set the detection threshold, clamping to [50, 100].
    /// Returns true if clamping was needed.
    pub fn set_detection_threshold(&mut self, value: u8) -> bool {
        let (clamped, was_clamped) = Self::clamp_threshold(value);
        self.detection_threshold = clamped;
        was_clamped
    }

    /// Set the sample size, clamping to [5, 100].
    /// Returns true if clamping was needed.
    pub fn set_sample_size(&mut self, value: u8) -> bool {
        let (clamped, was_clamped) = Self::clamp_sample_size(value);
        self.sample_size = clamped;
        was_clamped
    }
}

/// Per-language configuration override.
///
/// Allows overriding sequence number settings for a specific language
/// without modifying the language profile TOML.
#[derive(Debug, Clone, Default)]
pub struct LanguageOverride {
    /// Override auto_unnum setting for this language.
    pub auto_unnum: Option<bool>,
    /// Override front sequence columns.
    pub sequence_cols_front: Option<ColumnRange>,
    /// Override back sequence columns.
    pub sequence_cols_back: Option<ColumnRange>,
    /// Override detection threshold.
    pub detection_threshold: Option<u8>,
    /// Override sample size.
    pub sample_size: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        // Validates: Requirement 12.1
        let config = SeqNumConfig::default();
        assert_eq!(config.detection_threshold, 80);
        assert_eq!(config.sample_size, 20);
        assert!(!config.highlight_columns);
        assert_eq!(config.default_format, SequenceFormat::Numeric);
        assert!(!config.restore_on_save);
    }

    #[test]
    fn clamp_threshold_below_min() {
        // Validates: Requirement 2.8
        let (clamped, was_clamped) = SeqNumConfig::clamp_threshold(30);
        assert_eq!(clamped, 50);
        assert!(was_clamped);
    }

    #[test]
    fn clamp_threshold_above_max() {
        // Validates: Requirement 2.8
        let (clamped, was_clamped) = SeqNumConfig::clamp_threshold(120);
        assert_eq!(clamped, 100);
        assert!(was_clamped);
    }

    #[test]
    fn clamp_threshold_in_range() {
        // Validates: Requirement 2.8
        let (clamped, was_clamped) = SeqNumConfig::clamp_threshold(75);
        assert_eq!(clamped, 75);
        assert!(!was_clamped);
    }

    #[test]
    fn clamp_threshold_boundary_min() {
        // Validates: Requirement 2.8
        let (clamped, was_clamped) = SeqNumConfig::clamp_threshold(50);
        assert_eq!(clamped, 50);
        assert!(!was_clamped);
    }

    #[test]
    fn clamp_threshold_boundary_max() {
        // Validates: Requirement 2.8
        let (clamped, was_clamped) = SeqNumConfig::clamp_threshold(100);
        assert_eq!(clamped, 100);
        assert!(!was_clamped);
    }

    #[test]
    fn clamp_sample_size_below_min() {
        // Validates: Requirement 12.1
        let (clamped, was_clamped) = SeqNumConfig::clamp_sample_size(2);
        assert_eq!(clamped, 5);
        assert!(was_clamped);
    }

    #[test]
    fn clamp_sample_size_above_max() {
        // Validates: Requirement 12.1
        let (clamped, was_clamped) = SeqNumConfig::clamp_sample_size(200);
        assert_eq!(clamped, 100);
        assert!(was_clamped);
    }

    #[test]
    fn set_detection_threshold_returns_clamped_flag() {
        // Validates: Requirement 2.8
        let mut config = SeqNumConfig::new();
        assert!(config.set_detection_threshold(10));
        assert_eq!(config.detection_threshold, 50);
        assert!(!config.set_detection_threshold(80));
        assert_eq!(config.detection_threshold, 80);
    }
}
