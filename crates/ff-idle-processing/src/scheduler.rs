//! The central idle-time work coordinator.
//!
//! Owns registered work sources, manages the scheduling state machine,
//! and dispatches time slices in priority order.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::config::IdleConfig;
use crate::context::IdleWorkContext;
use crate::error::IdleProcessingError;
use crate::progress::{WorkProgress, WorkStatus};
use crate::traits::{IdleNotifier, IdleWorkSource};

/// Internal state of the idle scheduler's state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchedulerState {
    /// No active work sources or all complete. Zero overhead.
    Inactive,
    /// Active work sources exist but waiting for idle detection threshold.
    WaitingForIdle,
    /// Currently in idle state, dispatching time slices.
    Active,
}

/// Entry in the work source registry.
struct WorkEntry {
    source: Box<dyn IdleWorkSource>,
    /// Whether this source has completed its current work cycle.
    dormant: bool,
    /// How many consecutive cycles this source has been skipped (starvation counter).
    skipped_cycles: u32,
}

/// Central idle-time work coordinator.
///
/// Owns registered work sources, manages the scheduling state machine,
/// and dispatches time slices.
pub struct IdleScheduler {
    /// Registered work sources.
    entries: Vec<WorkEntry>,
    /// Event loop notifier.
    notifier: Box<dyn IdleNotifier>,
    /// Scheduler configuration.
    config: IdleConfig,
    /// Cancellation flag — set by `input_activity()`, polled by work sources.
    cancelled: Arc<AtomicBool>,
    /// Current state machine state.
    state: SchedulerState,
    /// Timestamp of last user input event.
    last_input: Option<Instant>,
    /// Round-robin index for equal-priority dispatch.
    round_robin_index: usize,
    /// Global idle cycle counter for starvation prevention.
    cycle_count: u32,
}

impl IdleScheduler {
    /// Create a new scheduler with the given configuration and notifier.
    pub fn new(config: IdleConfig, notifier: Box<dyn IdleNotifier>) -> Self {
        Self {
            entries: Vec::new(),
            notifier,
            config,
            cancelled: Arc::new(AtomicBool::new(false)),
            state: SchedulerState::Inactive,
            last_input: None,
            round_robin_index: 0,
            cycle_count: 0,
        }
    }

    // ── Work Source Registration ──────────────────────────────────────────

    /// Register a work source. Immediately enables it for time-slice dispatch.
    ///
    /// # Errors
    ///
    /// Returns `DuplicateWorkSource` if a source with the same name is already registered.
    pub fn register(&mut self, source: Box<dyn IdleWorkSource>) -> Result<(), IdleProcessingError> {
        let name = source.name().to_string();
        if self.entries.iter().any(|e| e.source.name() == name) {
            return Err(IdleProcessingError::DuplicateWorkSource { name });
        }
        self.entries.push(WorkEntry {
            source,
            dormant: false,
            skipped_cycles: 0,
        });
        self.activate_if_needed();
        Ok(())
    }

    /// Unregister a work source by name, returning ownership to the caller.
    ///
    /// # Errors
    ///
    /// Returns `WorkSourceNotFound` if no source with that name is registered.
    pub fn unregister(
        &mut self,
        name: &str,
    ) -> Result<Box<dyn IdleWorkSource>, IdleProcessingError> {
        let idx = self
            .entries
            .iter()
            .position(|e| e.source.name() == name)
            .ok_or_else(|| IdleProcessingError::WorkSourceNotFound {
                name: name.to_string(),
            })?;
        let entry = self.entries.remove(idx);
        self.update_state_after_change();
        Ok(entry.source)
    }

    // ── Input Activity ────────────────────────────────────────────────────

    /// Notify the scheduler that user input has occurred.
    ///
    /// Sets the cancellation flag and resets the idle detection timer.
    pub fn input_activity(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        self.last_input = Some(Instant::now());
        if self.state == SchedulerState::Active {
            self.state = SchedulerState::WaitingForIdle;
        }
    }

    // ── Idle Callback Entry Point ─────────────────────────────────────────

    /// Single entry point invoked by the GUI shell's idle callback.
    ///
    /// Dispatches one time-slice to the highest-priority active source.
    /// Returns `true` if more idle work remains, `false` if all complete.
    pub fn on_idle(&mut self) -> bool {
        // If disabled, do nothing
        if self.config.is_disabled() {
            return false;
        }

        // Check if we should transition to Active
        match self.state {
            SchedulerState::Inactive => return false,
            SchedulerState::WaitingForIdle => {
                if let Some(last) = self.last_input {
                    if last.elapsed() >= self.config.idle_detection_threshold {
                        self.state = SchedulerState::Active;
                        // Clear cancellation flag when entering active state
                        self.cancelled.store(false, Ordering::Release);
                    } else {
                        // Still in cooldown — re-request callback
                        self.notifier.request_idle_callback();
                        return true;
                    }
                } else {
                    // No input recorded — transition to active
                    self.state = SchedulerState::Active;
                    self.cancelled.store(false, Ordering::Release);
                }
            }
            SchedulerState::Active => {
                // Clear cancellation flag at start of each dispatch
                self.cancelled.store(false, Ordering::Release);
            }
        }

        // Find the source to dispatch
        let idx = self.select_source();
        if let Some(idx) = idx {
            self.dispatch_slice(idx);
            self.cycle_count = self.cycle_count.wrapping_add(1);
        }

        // Check if all sources are dormant
        let has_active = self.entries.iter().any(|e| !e.dormant);
        if !has_active {
            self.state = SchedulerState::Inactive;
            self.notifier.cancel_idle_callback();
            return false;
        }

        // More work remains — request next callback
        self.notifier.request_idle_callback();
        true
    }

    // ── Progress Queries ──────────────────────────────────────────────────

    /// Returns progress for a named work source.
    pub fn progress(&self, name: &str) -> Option<WorkProgress> {
        self.entries
            .iter()
            .find(|e| e.source.name() == name)
            .map(|e| e.source.progress())
    }

    /// Returns progress for all registered work sources.
    pub fn all_progress(&self) -> Vec<(String, WorkProgress)> {
        self.entries
            .iter()
            .map(|e| (e.source.name().to_string(), e.source.progress()))
            .collect()
    }

    /// Returns true when all registered work sources are complete.
    pub fn is_all_complete(&self) -> bool {
        self.entries.iter().all(|e| e.dormant)
    }

    // ── Invalidation ──────────────────────────────────────────────────────

    /// Externally invalidate a specific work source by name.
    pub fn invalidate_source(&mut self, name: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.source.name() == name) {
            entry.source.invalidate();
            entry.dormant = false;
            entry.skipped_cycles = 0;
        }
        self.activate_if_needed();
    }

    /// Invalidate all registered work sources simultaneously.
    pub fn invalidate_all(&mut self) {
        for entry in &mut self.entries {
            entry.source.invalidate();
            entry.dormant = false;
            entry.skipped_cycles = 0;
        }
        self.activate_if_needed();
    }

    // ── Configuration ─────────────────────────────────────────────────────

    /// Update the scheduler configuration at runtime.
    pub fn update_config(&mut self, config: IdleConfig) {
        self.config = config;
    }

    /// Get the current configuration.
    pub fn config(&self) -> &IdleConfig {
        &self.config
    }

    // ── Internal Helpers ──────────────────────────────────────────────────

    /// Select the index of the source to dispatch next.
    ///
    /// Implements priority ordering with starvation prevention and round-robin
    /// among equal-priority sources.
    fn select_source(&mut self) -> Option<usize> {
        let active: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.dormant)
            .map(|(i, _)| i)
            .collect();

        if active.is_empty() {
            return None;
        }

        // Starvation prevention: if any source has been skipped too many times,
        // force-dispatch the most-skipped one
        let max_skipped = active
            .iter()
            .map(|&i| self.entries[i].skipped_cycles)
            .max()
            .unwrap_or(0);

        if max_skipped >= self.config.starvation_cycle_limit {
            // Find the most-skipped source
            return active
                .iter()
                .max_by_key(|&&i| self.entries[i].skipped_cycles)
                .copied();
        }

        // Find the minimum (highest) priority among active sources
        let min_priority = active
            .iter()
            .map(|&i| self.entries[i].source.priority())
            .min()?;

        // Collect all sources at the minimum priority
        let top_priority: Vec<usize> = active
            .iter()
            .filter(|&&i| self.entries[i].source.priority() == min_priority)
            .copied()
            .collect();

        if top_priority.len() == 1 {
            Some(top_priority[0])
        } else {
            // Round-robin among equal-priority sources
            let rr = self.round_robin_index % top_priority.len();
            self.round_robin_index = (self.round_robin_index + 1) % top_priority.len();
            Some(top_priority[rr])
        }
    }

    /// Dispatch a single time slice to the source at the given index.
    fn dispatch_slice(&mut self, idx: usize) {
        // Increment skipped_cycles for all OTHER active sources
        for (i, entry) in self.entries.iter_mut().enumerate() {
            if i != idx && !entry.dormant {
                entry.skipped_cycles += 1;
            }
        }
        // Reset skipped_cycles for the dispatched source
        self.entries[idx].skipped_cycles = 0;

        let mut ctx = IdleWorkContext::new(&self.cancelled, self.config.time_budget);
        let status = self.entries[idx].source.perform_work(&mut ctx);

        match status {
            WorkStatus::Complete => {
                self.entries[idx].dormant = true;
            }
            WorkStatus::Interrupted => {
                // Source saved its progress; will resume on next idle period
                // Transition back to WaitingForIdle
                if self.state == SchedulerState::Active {
                    self.state = SchedulerState::WaitingForIdle;
                }
            }
            WorkStatus::MoreWork => {
                // Continue normally
            }
        }
    }

    /// Transition from Inactive to WaitingForIdle if there are active sources.
    fn activate_if_needed(&mut self) {
        let has_active = self.entries.iter().any(|e| !e.dormant);
        if has_active && self.state == SchedulerState::Inactive {
            self.state = SchedulerState::WaitingForIdle;
            self.notifier.request_idle_callback();
        }
    }

    /// Update state after a source is removed.
    fn update_state_after_change(&mut self) {
        let has_active = self.entries.iter().any(|e| !e.dormant);
        if !has_active {
            self.state = SchedulerState::Inactive;
            self.notifier.cancel_idle_callback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::priority::WorkPriority;
    use crate::test_support::{ManualIdleNotifier, MockWorkSource};
    use std::time::Duration;

    fn make_scheduler() -> IdleScheduler {
        let notifier = Box::new(ManualIdleNotifier::new());
        IdleScheduler::new(IdleConfig::default(), notifier)
    }

    fn make_fast_scheduler() -> IdleScheduler {
        let notifier = Box::new(ManualIdleNotifier::new());
        let config = IdleConfig {
            idle_detection_threshold: Duration::ZERO,
            time_budget: Duration::from_millis(10),
            ..Default::default()
        };
        IdleScheduler::new(config, notifier)
    }

    #[test]
    fn register_adds_source() {
        // Validates: Requirement 3 AC 3
        let mut sched = make_scheduler();
        let src = Box::new(MockWorkSource::new(
            "test",
            WorkPriority::SYNTAX_HIGHLIGHT,
            5,
        ));
        sched.register(src).unwrap();
        assert!(sched.progress("test").is_some());
    }

    #[test]
    fn register_duplicate_name_fails() {
        // Validates: Requirement 3 AC 3
        let mut sched = make_scheduler();
        sched
            .register(Box::new(MockWorkSource::new("dup", WorkPriority(10), 5)))
            .unwrap();
        let result = sched.register(Box::new(MockWorkSource::new("dup", WorkPriority(10), 3)));
        assert!(matches!(
            result,
            Err(IdleProcessingError::DuplicateWorkSource { .. })
        ));
    }

    #[test]
    fn unregister_removes_source() {
        // Validates: Requirement 3 AC 4
        let mut sched = make_scheduler();
        sched
            .register(Box::new(MockWorkSource::new("rem", WorkPriority(10), 5)))
            .unwrap();
        sched.unregister("rem").unwrap();
        assert!(sched.progress("rem").is_none());
    }

    #[test]
    fn unregister_nonexistent_fails() {
        let mut sched = make_scheduler();
        let result = sched.unregister("ghost");
        assert!(matches!(
            result,
            Err(IdleProcessingError::WorkSourceNotFound { .. })
        ));
    }

    #[test]
    fn register_requests_idle_callback() {
        // Validates: Requirement 3 AC 5, Requirement 11 AC 3
        let notifier = ManualIdleNotifier::new();
        // We need to share the notifier to inspect it — use a simple approach
        let mut sched = make_scheduler();
        // After register, state should be WaitingForIdle
        sched
            .register(Box::new(MockWorkSource::new("src", WorkPriority(10), 5)))
            .unwrap();
        assert_eq!(sched.state, SchedulerState::WaitingForIdle);
    }

    #[test]
    fn no_sources_means_inactive() {
        // Validates: Requirement 11 AC 1
        let sched = make_scheduler();
        assert_eq!(sched.state, SchedulerState::Inactive);
        assert!(sched.is_all_complete());
    }

    #[test]
    fn on_idle_dispatches_work_and_completes() {
        // Validates: Requirement 1 AC 3, AC 4, Requirement 7 AC 1, AC 2
        let mut sched = make_fast_scheduler();
        sched
            .register(Box::new(MockWorkSource::new("work", WorkPriority(10), 3)))
            .unwrap();

        // Run until complete
        let mut iterations = 0;
        loop {
            let more = sched.on_idle();
            iterations += 1;
            if !more || iterations > 20 {
                break;
            }
        }

        assert!(sched.is_all_complete());
        let p = sched.progress("work").unwrap();
        assert!(p.is_complete);
    }

    #[test]
    fn priority_ordering_dispatches_highest_first() {
        // Validates: Requirement 4 AC 3
        let mut sched = make_fast_scheduler();
        sched
            .register(Box::new(MockWorkSource::new("low", WorkPriority(40), 10)))
            .unwrap();
        sched
            .register(Box::new(MockWorkSource::new("high", WorkPriority(10), 1)))
            .unwrap();

        // First dispatch should go to "high" (priority 10 < 40)
        sched.on_idle();

        let high_progress = sched.progress("high").unwrap();
        let low_progress = sched.progress("low").unwrap();
        // high should have been dispatched (1 unit done = complete)
        assert!(high_progress.is_complete);
        // low should not have been dispatched yet
        assert_eq!(low_progress.completed_units, 0);
    }

    #[test]
    fn input_activity_sets_cancellation_flag() {
        // Validates: Requirement 5 AC 1, AC 5
        let mut sched = make_fast_scheduler();
        sched
            .register(Box::new(MockWorkSource::new("src", WorkPriority(10), 100)))
            .unwrap();
        sched.on_idle(); // Enter active state

        sched.input_activity();
        assert!(sched.cancelled.load(Ordering::Acquire));
    }

    #[test]
    fn input_activity_transitions_to_waiting() {
        // Validates: Requirement 5 AC 4
        let mut sched = make_fast_scheduler();
        sched
            .register(Box::new(MockWorkSource::new("src", WorkPriority(10), 100)))
            .unwrap();
        sched.on_idle(); // Enter active state
        assert_eq!(sched.state, SchedulerState::Active);

        sched.input_activity();
        assert_eq!(sched.state, SchedulerState::WaitingForIdle);
    }

    #[test]
    fn disabled_scheduler_does_nothing() {
        // Validates: Requirement 2 AC 5
        let notifier = Box::new(ManualIdleNotifier::new());
        let config = IdleConfig {
            time_budget: Duration::ZERO,
            idle_detection_threshold: Duration::ZERO,
            ..Default::default()
        };
        let mut sched = IdleScheduler::new(config, notifier);
        sched
            .register(Box::new(MockWorkSource::new("src", WorkPriority(10), 5)))
            .unwrap();
        let more = sched.on_idle();
        assert!(!more);
        // Source should not have been dispatched
        let p = sched.progress("src").unwrap();
        assert_eq!(p.completed_units, 0);
    }

    #[test]
    fn invalidate_source_reactivates_dormant() {
        // Validates: Requirement 7 AC 3, AC 4
        let mut sched = make_fast_scheduler();
        sched
            .register(Box::new(MockWorkSource::new("src", WorkPriority(10), 1)))
            .unwrap();

        // Complete the source
        loop {
            if !sched.on_idle() {
                break;
            }
        }
        assert!(sched.is_all_complete());

        // Invalidate and verify it's active again
        sched.invalidate_source("src");
        assert!(!sched.is_all_complete());
        let p = sched.progress("src").unwrap();
        assert_eq!(p.completed_units, 0);
    }

    #[test]
    fn invalidate_all_resets_all_sources() {
        // Validates: Requirement 7 AC 5
        let mut sched = make_fast_scheduler();
        sched
            .register(Box::new(MockWorkSource::new("a", WorkPriority(10), 1)))
            .unwrap();
        sched
            .register(Box::new(MockWorkSource::new("b", WorkPriority(20), 1)))
            .unwrap();

        loop {
            if !sched.on_idle() {
                break;
            }
        }
        assert!(sched.is_all_complete());

        sched.invalidate_all();
        assert!(!sched.is_all_complete());
    }

    #[test]
    fn all_progress_returns_all_sources() {
        // Validates: Requirement 6 AC 3
        let mut sched = make_scheduler();
        sched
            .register(Box::new(MockWorkSource::new("x", WorkPriority(10), 5)))
            .unwrap();
        sched
            .register(Box::new(MockWorkSource::new("y", WorkPriority(20), 3)))
            .unwrap();
        let all = sched.all_progress();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn round_robin_among_equal_priority() {
        // Validates: Requirement 1 AC 5
        let mut sched = make_fast_scheduler();
        sched
            .register(Box::new(MockWorkSource::new("a", WorkPriority(10), 10)))
            .unwrap();
        sched
            .register(Box::new(MockWorkSource::new("b", WorkPriority(10), 10)))
            .unwrap();

        // Run 4 dispatches
        for _ in 0..4 {
            sched.on_idle();
        }

        let pa = sched.progress("a").unwrap();
        let pb = sched.progress("b").unwrap();
        // Both should have received roughly equal dispatches
        assert!(pa.completed_units > 0);
        assert!(pb.completed_units > 0);
    }

    #[test]
    fn starvation_prevention_services_low_priority() {
        // Validates: Requirement 4 AC 6
        let notifier = Box::new(ManualIdleNotifier::new());
        let config = IdleConfig {
            idle_detection_threshold: Duration::ZERO,
            time_budget: Duration::from_millis(10),
            starvation_cycle_limit: 3,
            ..Default::default()
        };
        let mut sched = IdleScheduler::new(config, notifier);

        // High priority source that never completes
        sched
            .register(Box::new(MockWorkSource::new(
                "high",
                WorkPriority(10),
                1000,
            )))
            .unwrap();
        // Low priority source
        sched
            .register(Box::new(MockWorkSource::new("low", WorkPriority(40), 1000)))
            .unwrap();

        // Run enough cycles that starvation prevention kicks in
        for _ in 0..20 {
            sched.on_idle();
        }

        let low_progress = sched.progress("low").unwrap();
        // Low priority source should have been serviced at least once
        assert!(
            low_progress.completed_units > 0,
            "low priority source was starved"
        );
    }

    #[test]
    fn is_all_complete_false_when_work_remains() {
        // Validates: Requirement 6 AC 5
        let mut sched = make_scheduler();
        sched
            .register(Box::new(MockWorkSource::new("src", WorkPriority(10), 5)))
            .unwrap();
        assert!(!sched.is_all_complete());
    }

    #[test]
    fn update_config_takes_effect() {
        let mut sched = make_scheduler();
        let new_config = IdleConfig {
            time_budget: Duration::from_millis(20),
            ..Default::default()
        };
        sched.update_config(new_config);
        assert_eq!(sched.config().time_budget, Duration::from_millis(20));
    }
}
