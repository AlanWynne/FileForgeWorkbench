//! Test support types: ManualIdleNotifier and mock work sources.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::traits::IdleNotifier;

/// A manual idle notifier for headless/test mode.
///
/// Allows tests to trigger `on_idle()` directly without a real event loop.
///
/// # Examples
///
/// ```
/// use ff_idle_processing::test_support::ManualIdleNotifier;
/// use ff_idle_processing::traits::IdleNotifier;
///
/// let notifier = ManualIdleNotifier::new();
/// notifier.request_idle_callback();
/// assert!(notifier.is_idle_requested());
/// ```
pub struct ManualIdleNotifier {
    idle_requested: Arc<AtomicBool>,
}

impl ManualIdleNotifier {
    /// Create a new ManualIdleNotifier.
    pub fn new() -> Self {
        Self {
            idle_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns true if an idle callback has been requested.
    pub fn is_idle_requested(&self) -> bool {
        self.idle_requested.load(Ordering::Acquire)
    }
}

impl Default for ManualIdleNotifier {
    fn default() -> Self {
        Self::new()
    }
}

impl IdleNotifier for ManualIdleNotifier {
    fn request_idle_callback(&self) {
        self.idle_requested.store(true, Ordering::Release);
    }

    fn cancel_idle_callback(&self) {
        self.idle_requested.store(false, Ordering::Release);
    }
}

/// A simple mock work source that completes after a fixed number of units.
pub struct MockWorkSource {
    name: String,
    priority: crate::priority::WorkPriority,
    units_done: u64,
    total_units: u64,
}

impl MockWorkSource {
    /// Create a new mock source.
    pub fn new(name: &str, priority: crate::priority::WorkPriority, total_units: u64) -> Self {
        Self {
            name: name.to_string(),
            priority,
            units_done: 0,
            total_units,
        }
    }

    /// How many units have been processed.
    pub fn units_done(&self) -> u64 {
        self.units_done
    }
}

impl crate::traits::IdleWorkSource for MockWorkSource {
    fn perform_work(
        &mut self,
        ctx: &mut crate::context::IdleWorkContext,
    ) -> crate::progress::WorkStatus {
        if ctx.is_cancelled() {
            return crate::progress::WorkStatus::Interrupted;
        }
        self.units_done += 1;
        if self.units_done >= self.total_units {
            crate::progress::WorkStatus::Complete
        } else {
            crate::progress::WorkStatus::MoreWork
        }
    }

    fn priority(&self) -> crate::priority::WorkPriority {
        self.priority
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn progress(&self) -> crate::progress::WorkProgress {
        crate::progress::WorkProgress {
            completed_units: self.units_done,
            total_units: self.total_units,
            is_complete: self.units_done >= self.total_units,
        }
    }

    fn invalidate(&mut self) {
        self.units_done = 0;
    }
}
