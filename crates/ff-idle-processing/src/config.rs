//! Configuration parameters for the idle scheduler.

use std::time::Duration;

/// Configuration parameters for the idle scheduler.
///
/// Loaded from the configuration-system's `[idle-processing]` namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdleConfig {
    /// Duration of input inactivity before entering idle state.
    /// Default: 200ms. Range: [50ms, 5000ms].
    pub idle_detection_threshold: Duration,

    /// Maximum duration of a single time slice.
    /// Default: 10ms. Set to zero to disable idle processing entirely.
    pub time_budget: Duration,

    /// Maximum lines a work source should process per time slice (guidance).
    /// Default: 256.
    pub lines_per_slice: usize,

    /// Number of idle cycles before lower-priority sources get a guaranteed slice.
    /// Default: 10.
    pub starvation_cycle_limit: u32,
}

impl Default for IdleConfig {
    fn default() -> Self {
        Self {
            idle_detection_threshold: Duration::from_millis(200),
            time_budget: Duration::from_millis(10),
            lines_per_slice: 256,
            starvation_cycle_limit: 10,
        }
    }
}

impl IdleConfig {
    /// Returns true if idle processing is disabled (time_budget == 0).
    pub fn is_disabled(&self) -> bool {
        self.time_budget.is_zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        // Validates: Requirement 1 AC 2, Requirement 2 AC 1
        let cfg = IdleConfig::default();
        assert_eq!(cfg.idle_detection_threshold, Duration::from_millis(200));
        assert_eq!(cfg.time_budget, Duration::from_millis(10));
        assert_eq!(cfg.lines_per_slice, 256);
        assert_eq!(cfg.starvation_cycle_limit, 10);
    }

    #[test]
    fn zero_budget_is_disabled() {
        // Validates: Requirement 2 AC 5
        let cfg = IdleConfig {
            time_budget: Duration::ZERO,
            ..Default::default()
        };
        assert!(cfg.is_disabled());
    }

    #[test]
    fn nonzero_budget_is_enabled() {
        let cfg = IdleConfig::default();
        assert!(!cfg.is_disabled());
    }
}
