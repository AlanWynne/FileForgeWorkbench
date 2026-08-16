//! Per-editor-instance zoom state.
//!
//! Each open document tab owns one [`ZoomState`]. The offset is independent
//! across all editor instances — changing zoom in one tab does not affect others.

use crate::config::ZoomConfig;
use crate::operations::ZoomResult;
use crate::types::ZoomOffset;

/// Per-editor-instance zoom state.
///
/// Holds the current zoom offset and a reference to the active configuration.
/// All zoom operations (in, out, reset, set) are methods on this struct.
///
/// # Independence
///
/// Each editor tab creates its own `ZoomState`. There is no global zoom
/// registry — per-editor independence is architecturally enforced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoomState {
    /// The current zoom offset for this editor instance.
    offset: ZoomOffset,
    /// Configuration governing zoom behaviour.
    config: ZoomConfig,
}

impl ZoomState {
    /// Create a new zoom state initialised to the configured default offset.
    ///
    /// # Arguments
    ///
    /// * `config` — The validated zoom configuration.
    pub fn new(config: &ZoomConfig) -> Self {
        let offset = ZoomOffset::new(config.default_offset, config.min_offset, config.max_offset);
        Self {
            offset,
            config: config.clone(),
        }
    }

    /// Restore a zoom state from a persisted offset value.
    ///
    /// The offset is clamped to the current configuration range, handling
    /// the case where config limits changed between sessions.
    pub fn from_persisted(offset: i32, config: &ZoomConfig) -> Self {
        let clamped = ZoomOffset::new(offset, config.min_offset, config.max_offset);
        Self {
            offset: clamped,
            config: config.clone(),
        }
    }

    /// Get the current zoom offset.
    pub fn offset(&self) -> ZoomOffset {
        self.offset
    }

    /// Get the current configuration.
    pub fn config(&self) -> &ZoomConfig {
        &self.config
    }

    /// Compute the effective font size given a base size in points.
    ///
    /// Delegates to [`ZoomOffset::effective_font_size`].
    pub fn effective_font_size(&self, base_size: u32) -> u32 {
        self.offset.effective_font_size(base_size)
    }

    /// Apply a configuration change (e.g., from hot-reload).
    ///
    /// Clamps the current offset to the new range if it falls outside.
    pub fn apply_config_change(&mut self, new_config: &ZoomConfig) {
        self.config = new_config.clone();
        self.offset = ZoomOffset::new(
            self.offset.value(),
            new_config.min_offset,
            new_config.max_offset,
        );
    }

    /// Zoom in: increase offset by one step.
    ///
    /// Returns [`ZoomResult::Applied`] with the new offset, or
    /// [`ZoomResult::AtLimit`] if already at maximum.
    pub fn zoom_in(&mut self) -> ZoomResult {
        let current = self.offset.value();
        let max = self.config.max_offset;
        if current >= max {
            return ZoomResult::AtLimit {
                limit: max,
                message: format!("Maximum zoom reached (+{max})"),
            };
        }
        let new_value = (current + self.config.step as i32).min(max);
        self.offset = ZoomOffset::new(new_value, self.config.min_offset, max);
        ZoomResult::Applied {
            new_offset: self.offset.value(),
        }
    }

    /// Zoom out: decrease offset by one step.
    ///
    /// Returns [`ZoomResult::Applied`] with the new offset, or
    /// [`ZoomResult::AtLimit`] if already at minimum.
    pub fn zoom_out(&mut self) -> ZoomResult {
        let current = self.offset.value();
        let min = self.config.min_offset;
        if current <= min {
            return ZoomResult::AtLimit {
                limit: min,
                message: format!("Minimum zoom reached ({min})"),
            };
        }
        let new_value = (current - self.config.step as i32).max(min);
        self.offset = ZoomOffset::new(new_value, min, self.config.max_offset);
        ZoomResult::Applied {
            new_offset: self.offset.value(),
        }
    }

    /// Reset zoom offset to zero.
    ///
    /// Always returns [`ZoomResult::Applied`] with offset 0.
    pub fn zoom_reset(&mut self) -> ZoomResult {
        self.offset = ZoomOffset::new(0, self.config.min_offset, self.config.max_offset);
        ZoomResult::Applied {
            new_offset: self.offset.value(),
        }
    }

    /// Set the zoom offset to an absolute value (clamped to range).
    ///
    /// Returns [`ZoomResult::Applied`] with the clamped offset.
    pub fn set_offset(&mut self, value: i32) -> ZoomResult {
        self.offset = ZoomOffset::new(value, self.config.min_offset, self.config.max_offset);
        ZoomResult::Applied {
            new_offset: self.offset.value(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> ZoomConfig {
        ZoomConfig::default()
    }

    // Validates: Requirement 5.2 — new instance uses default_offset
    #[test]
    fn new_state_uses_default_offset() {
        let config = default_config();
        let state = ZoomState::new(&config);
        assert_eq!(state.offset().value(), 0);
    }

    // Validates: Requirement 5.2 — new instance with non-zero default
    #[test]
    fn new_state_with_custom_default_offset() {
        let config = ZoomConfig {
            default_offset: 5,
            ..Default::default()
        };
        let state = ZoomState::new(&config);
        assert_eq!(state.offset().value(), 5);
    }

    // Validates: Requirement 6.3 — persisted offset clamped to current range
    #[test]
    fn from_persisted_clamps_to_range() {
        let config = ZoomConfig {
            max_offset: 30,
            ..Default::default()
        };
        let state = ZoomState::from_persisted(50, &config);
        assert_eq!(state.offset().value(), 30);
    }

    // Validates: Requirement 6.2 — persisted offset within range preserved
    #[test]
    fn from_persisted_preserves_valid_offset() {
        let config = default_config();
        let state = ZoomState::from_persisted(10, &config);
        assert_eq!(state.offset().value(), 10);
    }

    // Validates: Requirement 1.2 — effective font size delegation
    #[test]
    fn effective_font_size_delegates_correctly() {
        let config = default_config();
        let state = ZoomState::from_persisted(3, &config);
        assert_eq!(state.effective_font_size(12), 15);
    }

    // Validates: Requirement 4.6 — config change clamps offset
    #[test]
    fn apply_config_change_clamps_offset() {
        let config = default_config();
        let mut state = ZoomState::from_persisted(50, &config);
        assert_eq!(state.offset().value(), 50);

        let new_config = ZoomConfig {
            max_offset: 30,
            ..Default::default()
        };
        state.apply_config_change(&new_config);
        assert_eq!(state.offset().value(), 30);
    }

    // Validates: Requirement 2.1 — zoom in increments by step
    #[test]
    fn zoom_in_increments_by_step() {
        let config = default_config();
        let mut state = ZoomState::new(&config);
        let result = state.zoom_in();
        assert_eq!(result, ZoomResult::Applied { new_offset: 1 });
        assert_eq!(state.offset().value(), 1);
    }

    // Validates: Requirement 2.6 — zoom in at max returns limit
    #[test]
    fn zoom_in_at_max_returns_at_limit() {
        let config = default_config();
        let mut state = ZoomState::from_persisted(60, &config);
        let result = state.zoom_in();
        assert!(matches!(result, ZoomResult::AtLimit { limit: 60, .. }));
        assert_eq!(state.offset().value(), 60);
    }

    // Validates: Requirement 2.2 — zoom out decrements by step
    #[test]
    fn zoom_out_decrements_by_step() {
        let config = default_config();
        let mut state = ZoomState::from_persisted(5, &config);
        let result = state.zoom_out();
        assert_eq!(result, ZoomResult::Applied { new_offset: 4 });
    }

    // Validates: Requirement 2.7 — zoom out at min returns limit
    #[test]
    fn zoom_out_at_min_returns_at_limit() {
        let config = default_config();
        let mut state = ZoomState::from_persisted(-10, &config);
        let result = state.zoom_out();
        assert!(matches!(result, ZoomResult::AtLimit { limit: -10, .. }));
    }

    // Validates: Requirement 2.3 — zoom reset sets to zero
    #[test]
    fn zoom_reset_sets_to_zero() {
        let config = default_config();
        let mut state = ZoomState::from_persisted(15, &config);
        let result = state.zoom_reset();
        assert_eq!(result, ZoomResult::Applied { new_offset: 0 });
        assert_eq!(state.offset().value(), 0);
    }

    // Validates: Requirement 8.2 — set_offset clamps to range
    #[test]
    fn set_offset_clamps_above_max() {
        let config = default_config();
        let mut state = ZoomState::new(&config);
        let result = state.set_offset(100);
        assert_eq!(result, ZoomResult::Applied { new_offset: 60 });
    }

    // Validates: Requirement 8.2 — set_offset clamps below min
    #[test]
    fn set_offset_clamps_below_min() {
        let config = default_config();
        let mut state = ZoomState::new(&config);
        let result = state.set_offset(-20);
        assert_eq!(result, ZoomResult::Applied { new_offset: -10 });
    }

    // Validates: Requirement 2.1 — step size > 1
    #[test]
    fn zoom_in_with_step_3() {
        let config = ZoomConfig {
            step: 3,
            ..Default::default()
        };
        let mut state = ZoomState::new(&config);
        state.zoom_in();
        assert_eq!(state.offset().value(), 3);
        state.zoom_in();
        assert_eq!(state.offset().value(), 6);
    }

    // Validates: Requirement 2.1 — step clamped at boundary
    #[test]
    fn zoom_in_step_clamped_at_max() {
        let config = ZoomConfig {
            step: 5,
            max_offset: 7,
            ..Default::default()
        };
        let mut state = ZoomState::from_persisted(5, &config);
        let result = state.zoom_in();
        assert_eq!(result, ZoomResult::Applied { new_offset: 7 });
    }

    // Validates: Requirement 5.1 — independent instances
    #[test]
    fn two_instances_are_independent() {
        let config = default_config();
        let mut state1 = ZoomState::new(&config);
        let mut state2 = ZoomState::new(&config);

        state1.zoom_in();
        state1.zoom_in();
        state2.zoom_out();

        assert_eq!(state1.offset().value(), 2);
        assert_eq!(state2.offset().value(), -1);
    }
}
