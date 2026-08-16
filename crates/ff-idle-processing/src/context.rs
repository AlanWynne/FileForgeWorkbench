//! Idle work context — time budget and cancellation signal.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Context provided to a work source during its time slice.
///
/// Exposes time budget queries and the cancellation signal.
/// Work sources MUST poll `is_cancelled()` at least once per significant
/// unit of work (e.g., per line processed).
pub struct IdleWorkContext<'a> {
    /// Reference to the cancellation flag (set by input_activity).
    cancelled: &'a AtomicBool,
    /// Instant when this time slice started.
    slice_start: Instant,
    /// Total time budget for this slice.
    time_budget: Duration,
}

impl<'a> IdleWorkContext<'a> {
    /// Create a new context for a time slice.
    pub(crate) fn new(cancelled: &'a AtomicBool, time_budget: Duration) -> Self {
        Self {
            cancelled,
            slice_start: Instant::now(),
            time_budget,
        }
    }

    /// Check whether a cancellation event has occurred.
    ///
    /// Work sources MUST poll this at least once per significant work unit.
    /// Uses `Acquire` ordering for sub-millisecond visibility.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    /// Returns the remaining time in the current time slice.
    ///
    /// When this reaches zero, the work source should yield.
    pub fn time_remaining(&self) -> Duration {
        let elapsed = self.slice_start.elapsed();
        self.time_budget.saturating_sub(elapsed)
    }

    /// Returns true if the time budget has been exhausted.
    pub fn budget_exhausted(&self) -> bool {
        self.slice_start.elapsed() >= self.time_budget
    }

    /// Returns elapsed time since the slice started.
    pub fn elapsed(&self) -> Duration {
        self.slice_start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_cancelled_false_initially() {
        // Validates: Requirement 5 AC 2
        let flag = AtomicBool::new(false);
        let ctx = IdleWorkContext::new(&flag, Duration::from_millis(10));
        assert!(!ctx.is_cancelled());
    }

    #[test]
    fn is_cancelled_true_after_flag_set() {
        // Validates: Requirement 5 AC 2, AC 5
        let flag = AtomicBool::new(false);
        let ctx = IdleWorkContext::new(&flag, Duration::from_millis(10));
        flag.store(true, Ordering::Release);
        assert!(ctx.is_cancelled());
    }

    #[test]
    fn time_remaining_decreases_over_time() {
        // Validates: Requirement 2 AC 2
        let flag = AtomicBool::new(false);
        let ctx = IdleWorkContext::new(&flag, Duration::from_millis(100));
        let r1 = ctx.time_remaining();
        std::thread::sleep(Duration::from_millis(5));
        let r2 = ctx.time_remaining();
        assert!(r2 <= r1);
    }

    #[test]
    fn budget_exhausted_after_sleep() {
        // Validates: Requirement 2 AC 2
        let flag = AtomicBool::new(false);
        let ctx = IdleWorkContext::new(&flag, Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(5));
        assert!(ctx.budget_exhausted());
    }

    #[test]
    fn time_remaining_saturates_at_zero() {
        // Validates: Requirement 2 AC 2
        let flag = AtomicBool::new(false);
        let ctx = IdleWorkContext::new(&flag, Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(ctx.time_remaining(), Duration::ZERO);
    }
}
