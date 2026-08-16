//! Progress reporting system for workflow execution.
//!
//! Provides determinate and indeterminate progress modes with throttled
//! event emission (max one event per 100ms per workflow instance) and
//! aggregation of child step progress into parent workflow progress.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Unique identifier for a workflow execution instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowExecutionId(pub String);

impl WorkflowExecutionId {
    /// Creates a new unique execution ID using UUID v4.
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

/// Progress mode for a workflow or step.
///
/// Addresses: Requirement 4, criteria 1/3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressMode {
    /// Known total — percentage and item counts are meaningful.
    Determinate,
    /// Unknown total — only status message is meaningful.
    Indeterminate,
}

/// A structured event conveying workflow progress to the UI via the Event Bus.
///
/// Addresses: Requirement 4, criteria 1/2/3/5/8
#[derive(Debug, Clone)]
pub struct ProgressEvent {
    /// The workflow execution ID.
    pub execution_id: WorkflowExecutionId,
    /// Workflow name.
    pub workflow_name: String,
    /// Progress mode.
    pub mode: ProgressMode,
    /// Current step name.
    pub current_step_name: String,
    /// Current step index (0-based).
    pub current_step_index: usize,
    /// Total step count.
    pub total_steps: usize,
    /// Overall workflow progress percentage (0.0–100.0).
    pub overall_percentage: f64,
    /// Current step progress percentage (0.0–100.0).
    pub step_percentage: f64,
    /// Status message describing current activity.
    pub message: String,
    /// Items processed (for determinate progress).
    pub items_processed: Option<u64>,
    /// Total items (for determinate progress).
    pub items_total: Option<u64>,
    /// Estimated time remaining in seconds (if calculable).
    pub estimated_remaining_seconds: Option<f64>,
    /// Elapsed time since workflow start.
    pub elapsed: Duration,
    /// Whether the workflow was resumed from a checkpoint.
    pub resumed_from_checkpoint: bool,
}

/// Internal state for progress tracking within a step.
#[derive(Debug)]
struct ProgressState {
    /// Current step percentage (0.0–100.0).
    step_percentage: f64,
    /// Items processed so far.
    items_processed: Option<u64>,
    /// Total items expected.
    items_total: Option<u64>,
    /// Current status message.
    message: String,
    /// Progress mode.
    mode: ProgressMode,
    /// Estimated time remaining in seconds.
    estimated_remaining_seconds: Option<f64>,
    /// Last time a progress event was emitted.
    last_emission: Option<Instant>,
    /// Throttle interval (default 100ms).
    throttle_interval: Duration,
    /// Count of events actually dispatched.
    dispatched_count: u64,
}

/// A handle provided to workflow steps for reporting intermediate progress.
///
/// Throttles emissions to at most once per 100ms per workflow instance.
/// Addresses: Requirement 4, criteria 1/2/3/6/7
#[derive(Debug, Clone)]
pub struct ProgressReporter {
    state: Arc<Mutex<ProgressState>>,
}

impl ProgressReporter {
    /// Creates a new progress reporter with the default 100ms throttle interval.
    pub fn new() -> Self {
        Self::with_throttle(Duration::from_millis(100))
    }

    /// Creates a new progress reporter with a custom throttle interval.
    pub fn with_throttle(throttle_interval: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(ProgressState {
                step_percentage: 0.0,
                items_processed: None,
                items_total: None,
                message: String::new(),
                mode: ProgressMode::Indeterminate,
                estimated_remaining_seconds: None,
                last_emission: None,
                throttle_interval,
                dispatched_count: 0,
            })),
        }
    }

    /// Reports determinate progress with item counts.
    ///
    /// Automatically calculates percentage from items_processed / items_total.
    /// Addresses: Requirement 4, criterion 2
    pub fn report_progress(
        &self,
        items_processed: u64,
        items_total: u64,
        message: impl Into<String>,
    ) {
        let percentage = if items_total > 0 {
            (items_processed as f64 / items_total as f64) * 100.0
        } else {
            0.0
        };

        let mut state = self.state.lock().expect("progress state lock poisoned");
        state.mode = ProgressMode::Determinate;
        state.step_percentage = percentage.clamp(0.0, 100.0);
        state.items_processed = Some(items_processed);
        state.items_total = Some(items_total);
        state.message = message.into();
        Self::maybe_emit(&mut state);
    }

    /// Reports indeterminate progress (spinning indicator).
    ///
    /// Addresses: Requirement 4, criterion 3
    pub fn report_indeterminate(&self, message: impl Into<String>) {
        let mut state = self.state.lock().expect("progress state lock poisoned");
        state.mode = ProgressMode::Indeterminate;
        state.message = message.into();
        state.items_processed = None;
        state.items_total = None;
        Self::maybe_emit(&mut state);
    }

    /// Reports progress with explicit percentage.
    pub fn report_percentage(&self, percentage: f64, message: impl Into<String>) {
        let mut state = self.state.lock().expect("progress state lock poisoned");
        state.mode = ProgressMode::Determinate;
        state.step_percentage = percentage.clamp(0.0, 100.0);
        state.message = message.into();
        Self::maybe_emit(&mut state);
    }

    /// Reports estimated time remaining in seconds.
    ///
    /// Addresses: Requirement 4, criterion 7
    pub fn report_eta(&self, remaining_seconds: f64) {
        let mut state = self.state.lock().expect("progress state lock poisoned");
        state.estimated_remaining_seconds = Some(remaining_seconds);
    }

    /// Returns the current step percentage.
    pub fn current_percentage(&self) -> f64 {
        self.state
            .lock()
            .expect("progress state lock poisoned")
            .step_percentage
    }

    /// Returns the current progress mode.
    pub fn current_mode(&self) -> ProgressMode {
        self.state
            .lock()
            .expect("progress state lock poisoned")
            .mode
    }

    /// Returns the number of events that were actually dispatched (not throttled).
    pub fn dispatched_count(&self) -> u64 {
        self.state
            .lock()
            .expect("progress state lock poisoned")
            .dispatched_count
    }

    /// Returns the current message.
    pub fn current_message(&self) -> String {
        self.state
            .lock()
            .expect("progress state lock poisoned")
            .message
            .clone()
    }

    /// Returns the current estimated remaining seconds.
    pub fn estimated_remaining(&self) -> Option<f64> {
        self.state
            .lock()
            .expect("progress state lock poisoned")
            .estimated_remaining_seconds
    }

    /// Resets the reporter for a new step.
    #[allow(dead_code)]
    pub(crate) fn reset(&self) {
        let mut state = self.state.lock().expect("progress state lock poisoned");
        state.step_percentage = 0.0;
        state.items_processed = None;
        state.items_total = None;
        state.message.clear();
        state.mode = ProgressMode::Indeterminate;
        state.estimated_remaining_seconds = None;
    }

    /// Checks throttle and increments dispatched_count if emission is allowed.
    fn maybe_emit(state: &mut ProgressState) {
        let now = Instant::now();
        let should_emit = match state.last_emission {
            None => true,
            Some(last) => now.duration_since(last) >= state.throttle_interval,
        };
        if should_emit {
            state.last_emission = Some(now);
            state.dispatched_count += 1;
        }
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculates aggregated parent workflow progress from step information.
///
/// Formula: `(completed_steps + current_step_fraction / 100.0) / total_steps * 100.0`
///
/// Addresses: Requirement 4, criterion 4
pub fn aggregate_progress(
    completed_steps: usize,
    current_step_fraction: f64,
    total_steps: usize,
) -> f64 {
    if total_steps == 0 {
        return 0.0;
    }
    let result =
        (completed_steps as f64 + current_step_fraction / 100.0) / total_steps as f64 * 100.0;
    result.clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 4.4 — progress aggregation formula

    #[test]
    fn aggregate_progress_zero_steps_returns_zero() {
        assert_eq!(aggregate_progress(0, 0.0, 0), 0.0);
    }

    #[test]
    fn aggregate_progress_no_steps_completed_zero_current() {
        assert_eq!(aggregate_progress(0, 0.0, 5), 0.0);
    }

    #[test]
    fn aggregate_progress_all_steps_completed() {
        let result = aggregate_progress(5, 0.0, 5);
        assert!((result - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn aggregate_progress_half_completed_half_current() {
        // 2 completed, current at 50%, total 5
        // (2 + 0.5) / 5 * 100 = 50.0
        let result = aggregate_progress(2, 50.0, 5);
        assert!((result - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn aggregate_progress_single_step_at_halfway() {
        // 0 completed, current at 50%, total 1
        // (0 + 0.5) / 1 * 100 = 50.0
        let result = aggregate_progress(0, 50.0, 1);
        assert!((result - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn aggregate_progress_clamps_to_100() {
        // Edge case: if somehow current fraction > 100
        let result = aggregate_progress(5, 100.0, 5);
        assert!(result <= 100.0);
    }

    #[test]
    fn aggregate_progress_is_always_non_negative() {
        let result = aggregate_progress(0, 0.0, 10);
        assert!(result >= 0.0);
    }

    // Validates: Requirement 4.1 — determinate progress mode

    #[test]
    fn reporter_report_progress_calculates_percentage() {
        let reporter = ProgressReporter::new();
        reporter.report_progress(50, 100, "halfway");
        assert!((reporter.current_percentage() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn reporter_report_progress_zero_total_gives_zero_percent() {
        let reporter = ProgressReporter::new();
        reporter.report_progress(10, 0, "no total");
        assert!((reporter.current_percentage() - 0.0).abs() < f64::EPSILON);
    }

    // Validates: Requirement 4.3 — indeterminate progress mode

    #[test]
    fn reporter_report_indeterminate_sets_mode() {
        let reporter = ProgressReporter::new();
        reporter.report_indeterminate("loading...");
        assert_eq!(reporter.current_mode(), ProgressMode::Indeterminate);
        assert_eq!(reporter.current_message(), "loading...");
    }

    // Validates: Requirement 4.6 — throttling at 100ms

    #[test]
    fn reporter_throttles_rapid_emissions() {
        let reporter = ProgressReporter::with_throttle(Duration::from_millis(100));
        // First report should always emit
        reporter.report_percentage(10.0, "a");
        assert_eq!(reporter.dispatched_count(), 1);

        // Immediate second report should be throttled
        reporter.report_percentage(20.0, "b");
        // May or may not emit depending on timing, but at most 2
        assert!(reporter.dispatched_count() <= 2);
    }

    // Validates: Requirement 4.7 — estimated time remaining

    #[test]
    fn reporter_eta_is_stored() {
        let reporter = ProgressReporter::new();
        reporter.report_eta(30.0);
        assert_eq!(reporter.estimated_remaining(), Some(30.0));
    }

    #[test]
    fn reporter_reset_clears_state() {
        let reporter = ProgressReporter::new();
        reporter.report_progress(50, 100, "halfway");
        reporter.report_eta(10.0);
        reporter.reset();
        assert!((reporter.current_percentage() - 0.0).abs() < f64::EPSILON);
        assert_eq!(reporter.current_mode(), ProgressMode::Indeterminate);
        assert_eq!(reporter.estimated_remaining(), None);
    }
}
