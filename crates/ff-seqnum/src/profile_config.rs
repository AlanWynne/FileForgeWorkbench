//! Configuration resolution merging language profile and config overrides.
//!
//! Implements the precedence chain:
//! config-system per-language > config-system global > language profile TOML.

use crate::config::{LanguageOverride, SeqNumConfig};
use crate::traits::LanguageProfile;
use crate::types::ColumnRange;

/// Resolved sequence configuration after merging all sources.
#[derive(Debug, Clone)]
pub struct ResolvedSequenceConfig {
    /// Front sequence column range (resolved).
    pub sequence_cols_front: Option<ColumnRange>,
    /// Back sequence column range (resolved).
    pub sequence_cols_back: Option<ColumnRange>,
    /// Whether auto-unnum is enabled (resolved).
    pub auto_unnum: bool,
    /// Detection threshold (resolved).
    pub detection_threshold: u8,
    /// Sample size (resolved).
    pub sample_size: u8,
}

/// Resolve the effective configuration by merging:
/// 1. Language profile TOML (base)
/// 2. Global SeqNumConfig (overlay)
/// 3. Per-language override (highest priority)
pub fn resolve_config(
    profile: &dyn LanguageProfile,
    config: &SeqNumConfig,
    language_override: Option<&LanguageOverride>,
) -> ResolvedSequenceConfig {
    let mut result = ResolvedSequenceConfig {
        sequence_cols_front: profile.sequence_cols_front(),
        sequence_cols_back: profile.sequence_cols_back(),
        auto_unnum: profile.auto_unnum(),
        detection_threshold: config.detection_threshold,
        sample_size: config.sample_size,
    };

    // Apply per-language overrides (highest priority)
    if let Some(override_cfg) = language_override {
        if let Some(front) = override_cfg.sequence_cols_front {
            result.sequence_cols_front = Some(front);
        }
        if let Some(back) = override_cfg.sequence_cols_back {
            result.sequence_cols_back = Some(back);
        }
        if let Some(auto) = override_cfg.auto_unnum {
            result.auto_unnum = auto;
        }
        if let Some(threshold) = override_cfg.detection_threshold {
            let (clamped, _) = SeqNumConfig::clamp_threshold(threshold);
            result.detection_threshold = clamped;
        }
        if let Some(sample) = override_cfg.sample_size {
            let (clamped, _) = SeqNumConfig::clamp_sample_size(sample);
            result.sample_size = clamped;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProfile {
        front: Option<ColumnRange>,
        back: Option<ColumnRange>,
        auto_unnum_val: bool,
    }

    impl LanguageProfile for MockProfile {
        fn sequence_cols_front(&self) -> Option<ColumnRange> {
            self.front
        }
        fn sequence_cols_back(&self) -> Option<ColumnRange> {
            self.back
        }
        fn auto_unnum(&self) -> bool {
            self.auto_unnum_val
        }
        fn language_id(&self) -> &str {
            "cobol"
        }
    }

    #[test]
    fn resolve_config_uses_profile_defaults() {
        // Validates: Requirement 12.3 (base case)
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum_val: true,
        };
        let config = SeqNumConfig::default();

        let resolved = resolve_config(&profile, &config, None);

        assert_eq!(resolved.sequence_cols_front.unwrap().start(), 1);
        assert_eq!(resolved.sequence_cols_back.unwrap().start(), 73);
        assert!(resolved.auto_unnum);
        assert_eq!(resolved.detection_threshold, 80);
    }

    #[test]
    fn resolve_config_per_language_override_auto_unnum() {
        // Validates: Requirement 12.4
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum_val: true,
        };
        let config = SeqNumConfig::default();
        let override_cfg = LanguageOverride {
            auto_unnum: Some(false),
            ..Default::default()
        };

        let resolved = resolve_config(&profile, &config, Some(&override_cfg));

        assert!(!resolved.auto_unnum);
    }

    #[test]
    fn resolve_config_per_language_override_columns() {
        // Validates: Requirement 12.3
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum_val: true,
        };
        let config = SeqNumConfig::default();
        let override_cfg = LanguageOverride {
            sequence_cols_front: Some(ColumnRange::new(1, 5).unwrap()),
            ..Default::default()
        };

        let resolved = resolve_config(&profile, &config, Some(&override_cfg));

        assert_eq!(resolved.sequence_cols_front.unwrap().end(), 5);
        assert_eq!(resolved.sequence_cols_back.unwrap().end(), 80); // Unchanged
    }

    #[test]
    fn resolve_config_missing_override_uses_defaults() {
        // Validates: Requirement 12.3 (no override case)
        let profile = MockProfile {
            front: None,
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum_val: true,
        };
        let config = SeqNumConfig::default();

        let resolved = resolve_config(&profile, &config, None);

        assert!(resolved.sequence_cols_front.is_none());
        assert!(resolved.sequence_cols_back.is_some());
    }
}
