//! Traits for idle work sources and event loop notifiers.

use crate::context::IdleWorkContext;
use crate::priority::WorkPriority;
use crate::progress::{WorkProgress, WorkStatus};

/// Trait that background work providers implement to participate in
/// idle-time scheduling. Object-safe for heterogeneous collections.
///
/// # Contract
///
/// - `perform_work` MUST poll `context.is_cancelled()` at least once per
///   significant unit of work and return `WorkStatus::Interrupted` if cancelled.
/// - `perform_work` SHOULD check `context.budget_exhausted()` and return
///   `WorkStatus::MoreWork` when the budget is exhausted.
/// - `invalidate` MUST reset progress so the source can be re-dispatched.
pub trait IdleWorkSource: Send + Sync {
    /// Execute a bounded unit of work within the time budget.
    fn perform_work(&mut self, context: &mut IdleWorkContext) -> WorkStatus;

    /// Returns the priority level of this work source.
    fn priority(&self) -> WorkPriority;

    /// Returns a human-readable identifier for diagnostics and logging.
    fn name(&self) -> &str;

    /// Returns the current progress state for tracking.
    fn progress(&self) -> WorkProgress;

    /// Reset progress to the beginning. Called when previous work is stale.
    ///
    /// Default implementation does nothing.
    fn invalidate(&mut self) {}
}

/// Abstraction over the GUI event loop's idle callback mechanism.
///
/// The GUI shell implements this trait to integrate with the scheduler.
pub trait IdleNotifier: Send + Sync {
    /// Request the event loop to invoke `IdleScheduler::on_idle()` when idle.
    fn request_idle_callback(&self);

    /// Cancel a previously requested idle callback.
    fn cancel_idle_callback(&self);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::ManualIdleNotifier;

    struct MockSource {
        name: String,
        priority: WorkPriority,
        work_done: u64,
        total: u64,
    }

    impl IdleWorkSource for MockSource {
        fn perform_work(&mut self, ctx: &mut IdleWorkContext) -> WorkStatus {
            if ctx.is_cancelled() {
                return WorkStatus::Interrupted;
            }
            self.work_done += 1;
            if self.work_done >= self.total {
                WorkStatus::Complete
            } else {
                WorkStatus::MoreWork
            }
        }
        fn priority(&self) -> WorkPriority {
            self.priority
        }
        fn name(&self) -> &str {
            &self.name
        }
        fn progress(&self) -> WorkProgress {
            WorkProgress {
                completed_units: self.work_done,
                total_units: self.total,
                is_complete: self.work_done >= self.total,
            }
        }
        fn invalidate(&mut self) {
            self.work_done = 0;
        }
    }

    #[test]
    fn idle_work_source_is_object_safe() {
        // Validates: Requirement 3 AC 6
        let sources: Vec<Box<dyn IdleWorkSource>> = vec![
            Box::new(MockSource {
                name: "a".into(),
                priority: WorkPriority(10),
                work_done: 0,
                total: 5,
            }),
            Box::new(MockSource {
                name: "b".into(),
                priority: WorkPriority(20),
                work_done: 0,
                total: 3,
            }),
        ];
        assert_eq!(sources.len(), 2);
    }

    #[test]
    fn manual_notifier_tracks_requests() {
        // Validates: Requirement 9 AC 7
        let notifier = ManualIdleNotifier::new();
        assert!(!notifier.is_idle_requested());
        notifier.request_idle_callback();
        assert!(notifier.is_idle_requested());
        notifier.cancel_idle_callback();
        assert!(!notifier.is_idle_requested());
    }
}
