//! Caret blink model.
//!
//! The `BlinkState` manages blink period and phase computation. The GUI shell
//! owns the clock; this model only computes visibility from elapsed time.

/// Default blink period in milliseconds.
pub const DEFAULT_BLINK_PERIOD_MS: u32 = 530;

/// Manages caret blink state computation.
///
/// The model is timer-agnostic — the GUI shell drives the clock. This struct
/// stores only the period and the timestamp of the last reset.
///
/// Addresses: Requirement 3, criteria 3.1–3.7
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlinkState {
    /// Blink period in milliseconds. 0 = always visible (no blinking).
    period_ms: u32,
    /// Timestamp (ms) of the last blink reset (on caret move or focus gain).
    last_reset_timestamp_ms: u64,
}

impl BlinkState {
    /// Creates a new blink state with the given period.
    ///
    /// Addresses: Requirement 3, criterion 3.2 (default period = 530ms)
    pub fn new(period_ms: u32) -> Self {
        Self {
            period_ms,
            last_reset_timestamp_ms: 0,
        }
    }

    /// Queries whether the caret should be visible at the given time.
    ///
    /// - When `period_ms` is 0, always returns true (no blinking).
    /// - Otherwise, the caret is visible in the first half of the cycle
    ///   and hidden in the second half.
    ///
    /// Addresses: Requirement 3, criteria 3.3, 3.5
    pub fn is_visible(&self, current_time_ms: u64) -> bool {
        if self.period_ms == 0 {
            return true;
        }
        let elapsed = current_time_ms.saturating_sub(self.last_reset_timestamp_ms);
        let phase = elapsed % (self.period_ms as u64);
        phase < (self.period_ms as u64 / 2)
    }

    /// Resets the blink cycle to the visible phase.
    ///
    /// Called when the caret moves or the pane gains focus, ensuring
    /// the caret is immediately visible.
    ///
    /// Addresses: Requirement 3, criterion 3.6
    pub fn reset(&mut self, current_time_ms: u64) {
        self.last_reset_timestamp_ms = current_time_ms;
    }

    /// Updates the blink period.
    ///
    /// A period of 0 means no blinking (always visible).
    ///
    /// Addresses: Requirement 3, criterion 3.7
    pub fn set_period(&mut self, period_ms: u32) {
        self.period_ms = period_ms;
    }

    /// Returns the current blink period in milliseconds.
    pub fn period_ms(&self) -> u32 {
        self.period_ms
    }

    /// Returns the last reset timestamp.
    pub fn last_reset_timestamp_ms(&self) -> u64 {
        self.last_reset_timestamp_ms
    }
}

impl Default for BlinkState {
    fn default() -> Self {
        Self::new(DEFAULT_BLINK_PERIOD_MS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_period_is_530ms() {
        // Validates: Requirement 3.2
        let blink = BlinkState::default();
        assert_eq!(blink.period_ms(), 530);
    }

    #[test]
    fn is_visible_returns_true_at_start_of_cycle() {
        // Validates: Requirement 3.5
        let mut blink = BlinkState::new(500);
        blink.reset(1000);
        // At time 1000 (elapsed = 0), should be visible
        assert!(blink.is_visible(1000));
    }

    #[test]
    fn is_visible_returns_true_in_first_half_of_cycle() {
        // Validates: Requirement 3.5
        let mut blink = BlinkState::new(500);
        blink.reset(1000);
        // At time 1249 (elapsed = 249, < 250), should be visible
        assert!(blink.is_visible(1249));
    }

    #[test]
    fn is_visible_returns_false_in_second_half_of_cycle() {
        // Validates: Requirement 3.5
        let mut blink = BlinkState::new(500);
        blink.reset(1000);
        // At time 1250 (elapsed = 250, >= 250), should be hidden
        assert!(!blink.is_visible(1250));
    }

    #[test]
    fn is_visible_cycles_back_to_visible_after_full_period() {
        // Validates: Requirement 3.5
        let mut blink = BlinkState::new(500);
        blink.reset(1000);
        // At time 1500 (elapsed = 500, 500 % 500 = 0, < 250), visible again
        assert!(blink.is_visible(1500));
    }

    #[test]
    fn period_zero_always_visible() {
        // Validates: Requirement 3.3
        let blink = BlinkState::new(0);
        assert!(blink.is_visible(0));
        assert!(blink.is_visible(1000));
        assert!(blink.is_visible(999_999));
    }

    #[test]
    fn reset_makes_caret_visible_again() {
        // Validates: Requirement 3.6
        let mut blink = BlinkState::new(500);
        blink.reset(0);
        // At time 300 (hidden phase)
        assert!(!blink.is_visible(300));
        // Reset at 300
        blink.reset(300);
        // Now at 300 (elapsed = 0) should be visible
        assert!(blink.is_visible(300));
    }

    #[test]
    fn set_period_updates_blink_rate() {
        // Validates: Requirement 3.7
        let mut blink = BlinkState::new(500);
        blink.set_period(1000);
        assert_eq!(blink.period_ms(), 1000);
    }

    #[test]
    fn set_period_to_zero_disables_blinking() {
        // Validates: Requirement 3.3
        let mut blink = BlinkState::new(500);
        blink.reset(0);
        assert!(!blink.is_visible(300)); // hidden at 300ms in 500ms period
        blink.set_period(0);
        assert!(blink.is_visible(300)); // now always visible
    }
}
