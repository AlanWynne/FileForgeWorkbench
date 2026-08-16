//! Configuration integration for external modification detection.
//!
//! Defines the `ExternalModConfig` struct and configuration key constants
//! for the `[editor.external_modification]` namespace.

use crate::reload_policy::ReloadPolicy;

/// Configuration namespace prefix for external modification settings.
pub const CONFIG_NAMESPACE: &str = "editor.external_modification";

/// Configuration key for the reload policy.
pub const KEY_POLICY: &str = "editor.external_modification.policy";

/// Configuration key for reload-preserves-undo setting.
pub const KEY_RELOAD_PRESERVES_UNDO: &str = "editor.external_modification.reload_preserves_undo";

/// Configuration key for check-on-focus setting.
pub const KEY_CHECK_ON_FOCUS: &str = "editor.external_modification.check_on_focus";

/// Configuration key for auto-follow-rename setting.
pub const KEY_AUTO_FOLLOW_RENAME: &str = "editor.external_modification.auto_follow_rename";

/// Configuration key for batch debounce window in milliseconds.
pub const KEY_BATCH_DEBOUNCE_MS: &str = "editor.external_modification.batch_debounce_ms";

/// Configuration key for polling interval in milliseconds.
pub const KEY_POLLING_INTERVAL_MS: &str = "editor.external_modification.polling_interval_ms";

/// Minimum value for batch debounce window (ms).
pub const BATCH_DEBOUNCE_MS_MIN: u64 = 100;
/// Maximum value for batch debounce window (ms).
pub const BATCH_DEBOUNCE_MS_MAX: u64 = 5000;
/// Default value for batch debounce window (ms).
pub const BATCH_DEBOUNCE_MS_DEFAULT: u64 = 500;

/// Minimum value for polling interval (ms).
pub const POLLING_INTERVAL_MS_MIN: u64 = 1000;
/// Maximum value for polling interval (ms).
pub const POLLING_INTERVAL_MS_MAX: u64 = 60000;
/// Default value for polling interval (ms).
pub const POLLING_INTERVAL_MS_DEFAULT: u64 = 5000;

/// Typed configuration for the external modification subsystem.
///
/// All fields have documented defaults and valid ranges. Out-of-range
/// numeric values are clamped to the nearest valid bound.
///
/// Addresses: Requirement 10, all criteria
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalModConfig {
    /// Reload policy: prompt, auto, or ignore.
    pub policy: ReloadPolicy,
    /// Whether reload preserves undo history.
    pub reload_preserves_undo: bool,
    /// Whether to perform mtime scan on focus-gained.
    pub check_on_focus: bool,
    /// Whether to auto-follow renames for non-dirty buffers.
    pub auto_follow_rename: bool,
    /// Debounce window for batch coalescing (ms), clamped to [100, 5000].
    pub batch_debounce_ms: u64,
    /// Fallback polling interval when VFS watch unavailable (ms), clamped to [1000, 60000].
    pub polling_interval_ms: u64,
}

impl Default for ExternalModConfig {
    fn default() -> Self {
        Self {
            policy: ReloadPolicy::Prompt,
            reload_preserves_undo: false,
            check_on_focus: true,
            auto_follow_rename: false,
            batch_debounce_ms: BATCH_DEBOUNCE_MS_DEFAULT,
            polling_interval_ms: POLLING_INTERVAL_MS_DEFAULT,
        }
    }
}

impl ExternalModConfig {
    /// Clamp `batch_debounce_ms` to its valid range [100, 5000].
    ///
    /// Returns `true` if the value was clamped (was out of range).
    pub fn clamp_batch_debounce_ms(&mut self) -> bool {
        let original = self.batch_debounce_ms;
        self.batch_debounce_ms = self
            .batch_debounce_ms
            .clamp(BATCH_DEBOUNCE_MS_MIN, BATCH_DEBOUNCE_MS_MAX);
        self.batch_debounce_ms != original
    }

    /// Clamp `polling_interval_ms` to its valid range [1000, 60000].
    ///
    /// Returns `true` if the value was clamped (was out of range).
    pub fn clamp_polling_interval_ms(&mut self) -> bool {
        let original = self.polling_interval_ms;
        self.polling_interval_ms = self
            .polling_interval_ms
            .clamp(POLLING_INTERVAL_MS_MIN, POLLING_INTERVAL_MS_MAX);
        self.polling_interval_ms != original
    }

    /// Clamp all numeric fields to their valid ranges.
    ///
    /// Returns a list of field names that were clamped.
    pub fn clamp_all(&mut self) -> Vec<&'static str> {
        let mut clamped = Vec::new();
        if self.clamp_batch_debounce_ms() {
            clamped.push("batch_debounce_ms");
        }
        if self.clamp_polling_interval_ms() {
            clamped.push("polling_interval_ms");
        }
        clamped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = ExternalModConfig::default();
        assert_eq!(config.policy, ReloadPolicy::Prompt);
        assert!(!config.reload_preserves_undo);
        assert!(config.check_on_focus);
        assert!(!config.auto_follow_rename);
        assert_eq!(config.batch_debounce_ms, 500);
        assert_eq!(config.polling_interval_ms, 5000);
    }

    #[test]
    fn clamp_batch_debounce_ms_clamps_below_minimum() {
        let mut config = ExternalModConfig {
            batch_debounce_ms: 50,
            ..Default::default()
        };
        let clamped = config.clamp_batch_debounce_ms();
        assert!(clamped);
        assert_eq!(config.batch_debounce_ms, 100);
    }

    #[test]
    fn clamp_batch_debounce_ms_clamps_above_maximum() {
        let mut config = ExternalModConfig {
            batch_debounce_ms: 10000,
            ..Default::default()
        };
        let clamped = config.clamp_batch_debounce_ms();
        assert!(clamped);
        assert_eq!(config.batch_debounce_ms, 5000);
    }

    #[test]
    fn clamp_batch_debounce_ms_does_not_clamp_valid_value() {
        let mut config = ExternalModConfig {
            batch_debounce_ms: 1000,
            ..Default::default()
        };
        let clamped = config.clamp_batch_debounce_ms();
        assert!(!clamped);
        assert_eq!(config.batch_debounce_ms, 1000);
    }

    #[test]
    fn clamp_polling_interval_ms_clamps_below_minimum() {
        let mut config = ExternalModConfig {
            polling_interval_ms: 500,
            ..Default::default()
        };
        let clamped = config.clamp_polling_interval_ms();
        assert!(clamped);
        assert_eq!(config.polling_interval_ms, 1000);
    }

    #[test]
    fn clamp_polling_interval_ms_clamps_above_maximum() {
        let mut config = ExternalModConfig {
            polling_interval_ms: 100_000,
            ..Default::default()
        };
        let clamped = config.clamp_polling_interval_ms();
        assert!(clamped);
        assert_eq!(config.polling_interval_ms, 60000);
    }

    #[test]
    fn clamp_all_returns_names_of_clamped_fields() {
        let mut config = ExternalModConfig {
            batch_debounce_ms: 0,
            polling_interval_ms: 999_999,
            ..Default::default()
        };
        let clamped = config.clamp_all();
        assert_eq!(clamped.len(), 2);
        assert!(clamped.contains(&"batch_debounce_ms"));
        assert!(clamped.contains(&"polling_interval_ms"));
    }

    #[test]
    fn clamp_all_returns_empty_for_valid_values() {
        let mut config = ExternalModConfig::default();
        let clamped = config.clamp_all();
        assert!(clamped.is_empty());
    }
}
