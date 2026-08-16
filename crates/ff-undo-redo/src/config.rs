//! Configuration types for the undo/redo system.
//!
//! [`UndoConfig`] holds parsed, validated configuration values sourced from
//! `editor.undo.*` and `editor.recovery.*` settings in the configuration system.

use ff_logging::{log, LogLevel};

/// Default maximum undo stack depth.
pub const DEFAULT_MAX_LEVELS: u32 = 100;
/// Minimum allowed maximum undo levels.
pub const MIN_MAX_LEVELS: u32 = 0;
/// Maximum allowed maximum undo levels.
pub const MAX_MAX_LEVELS: u32 = 10_000;

/// Default coalesce timeout in milliseconds.
pub const DEFAULT_COALESCE_TIMEOUT_MS: u32 = 2000;
/// Minimum coalesce timeout.
pub const MIN_COALESCE_TIMEOUT_MS: u32 = 100;
/// Maximum coalesce timeout.
pub const MAX_COALESCE_TIMEOUT_MS: u32 = 10_000;

/// Default recovery interval in seconds.
pub const DEFAULT_RECOVERY_INTERVAL_SECONDS: u32 = 60;

/// Parsed configuration values for the undo system.
///
/// All values are validated and clamped to their valid ranges on construction.
///
/// # Configuration Keys
///
/// - `editor.undo.max_levels` — stack depth limit (0 disables undo)
/// - `editor.undo.coalesce_timeout_ms` — typing coalesce window
/// - `editor.undo.selection_history` — enable/disable selection restoration
/// - `editor.recovery.interval_seconds` — recovery file write interval (0 disables)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoConfig {
    /// Maximum undo stack depth. Range: [0, 10000]. Default: 100.
    pub max_levels: u32,
    /// Coalesce timeout in milliseconds. Range: [100, 10000]. Default: 2000.
    pub coalesce_timeout_ms: u32,
    /// Whether selection history is enabled. Default: true.
    pub selection_history_enabled: bool,
    /// Recovery file write interval in seconds. 0 = disabled. Default: 60.
    pub recovery_interval_seconds: u32,
}

impl Default for UndoConfig {
    fn default() -> Self {
        Self {
            max_levels: DEFAULT_MAX_LEVELS,
            coalesce_timeout_ms: DEFAULT_COALESCE_TIMEOUT_MS,
            selection_history_enabled: true,
            recovery_interval_seconds: DEFAULT_RECOVERY_INTERVAL_SECONDS,
        }
    }
}

impl UndoConfig {
    /// Create a new configuration with validated values.
    ///
    /// Values outside their valid ranges are clamped and a warning is logged.
    ///
    /// # Parameters
    ///
    /// - `max_levels`: If negative (passed as i32), defaults to 100 with a warning.
    /// - `coalesce_timeout_ms`: Clamped to [100, 10000].
    /// - `selection_history_enabled`: Pass-through.
    /// - `recovery_interval_seconds`: Pass-through (0 disables).
    pub fn new(
        max_levels: i32,
        coalesce_timeout_ms: u32,
        selection_history_enabled: bool,
        recovery_interval_seconds: u32,
    ) -> Self {
        let validated_max_levels = if max_levels < 0 {
            log(
                LogLevel::Warn,
                "ff_undo_redo::config",
                &format!(
                    "editor.undo.max_levels is negative ({}), applying default of {}",
                    max_levels, DEFAULT_MAX_LEVELS
                ),
            );
            DEFAULT_MAX_LEVELS
        } else {
            let v = max_levels as u32;
            v.min(MAX_MAX_LEVELS)
        };

        let validated_coalesce =
            coalesce_timeout_ms.clamp(MIN_COALESCE_TIMEOUT_MS, MAX_COALESCE_TIMEOUT_MS);

        if validated_coalesce != coalesce_timeout_ms {
            log(
                LogLevel::Warn,
                "ff_undo_redo::config",
                &format!(
                    "editor.undo.coalesce_timeout_ms {} clamped to {}",
                    coalesce_timeout_ms, validated_coalesce
                ),
            );
        }

        Self {
            max_levels: validated_max_levels,
            coalesce_timeout_ms: validated_coalesce,
            selection_history_enabled,
            recovery_interval_seconds,
        }
    }

    /// Returns true if undo is disabled (max_levels == 0).
    pub fn is_undo_disabled(&self) -> bool {
        self.max_levels == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_correct_values() {
        let config = UndoConfig::default();
        assert_eq!(config.max_levels, 100);
        assert_eq!(config.coalesce_timeout_ms, 2000);
        assert!(config.selection_history_enabled);
        assert_eq!(config.recovery_interval_seconds, 60);
    }

    #[test]
    fn negative_max_levels_uses_default() {
        let config = UndoConfig::new(-5, 2000, true, 60);
        assert_eq!(config.max_levels, DEFAULT_MAX_LEVELS);
    }

    #[test]
    fn max_levels_clamped_to_upper_bound() {
        let config = UndoConfig::new(50_000, 2000, true, 60);
        assert_eq!(config.max_levels, MAX_MAX_LEVELS);
    }

    #[test]
    fn coalesce_timeout_clamped_to_min() {
        let config = UndoConfig::new(100, 10, true, 60);
        assert_eq!(config.coalesce_timeout_ms, MIN_COALESCE_TIMEOUT_MS);
    }

    #[test]
    fn coalesce_timeout_clamped_to_max() {
        let config = UndoConfig::new(100, 99_999, true, 60);
        assert_eq!(config.coalesce_timeout_ms, MAX_COALESCE_TIMEOUT_MS);
    }

    #[test]
    fn zero_max_levels_disables_undo() {
        let config = UndoConfig::new(0, 2000, true, 60);
        assert!(config.is_undo_disabled());
    }

    #[test]
    fn nonzero_max_levels_enables_undo() {
        let config = UndoConfig::new(50, 2000, true, 60);
        assert!(!config.is_undo_disabled());
    }

    #[test]
    fn zero_recovery_interval_is_valid() {
        let config = UndoConfig::new(100, 2000, true, 0);
        assert_eq!(config.recovery_interval_seconds, 0);
    }
}
