//! Workflow runtime state tracking.
//!
//! Tracks the execution state of a running workflow instance, including
//! current step, step statuses, and lifecycle phase.

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::context::WorkflowContext;
use crate::progress::WorkflowExecutionId;

/// The lifecycle phase of a running workflow.
///
/// Addresses: Requirement 2, criterion 8; Requirement 3, criterion 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WorkflowPhase {
    /// Workflow is actively executing steps.
    Running,
    /// Workflow is paused (current step completed, not advancing).
    Paused,
    /// Workflow completed all steps successfully.
    Completed,
    /// Workflow failed (error policy exhausted).
    Failed,
    /// Workflow was cancelled by user.
    Cancelled,
    /// Workflow is executing compensating actions (rollback).
    RollingBack,
}

/// The execution status of a single workflow step.
///
/// Addresses: Requirement 2, criterion 1; Requirement 5, criterion 1
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StepStatus {
    /// Step has not yet started.
    Pending,
    /// Step is currently executing.
    Running,
    /// Step completed successfully.
    Completed,
    /// Step failed (error policy may allow continuation).
    Failed {
        /// Error description.
        error: String,
        /// Number of retry attempts made.
        retries_attempted: u32,
    },
    /// Step was skipped (continue-on-error policy).
    Skipped {
        /// Reason for skipping.
        reason: String,
    },
    /// Step was cancelled.
    Cancelled,
}

/// The runtime execution state of a workflow instance.
///
/// Addresses: Requirement 2, criteria 1/8
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Unique execution ID for this workflow run.
    pub execution_id: WorkflowExecutionId,
    /// Name of the workflow definition being executed.
    pub workflow_name: String,
    /// Current lifecycle phase.
    pub phase: WorkflowPhase,
    /// The current step being executed (or last completed).
    pub current_step: String,
    /// Status of each step.
    pub step_statuses: HashMap<String, StepStatus>,
    /// The shared context.
    pub context: WorkflowContext,
    /// Timestamp when execution started (RFC 3339).
    pub started_at: String,
    /// Elapsed time (excluding paused time).
    pub elapsed: Duration,
    /// Overall progress percentage (0.0–100.0).
    pub overall_progress: f64,
}

impl WorkflowState {
    /// Creates a new initial state for a workflow execution.
    pub fn new(
        execution_id: WorkflowExecutionId,
        workflow_name: String,
        initial_step: String,
        step_names: Vec<String>,
        context: WorkflowContext,
    ) -> Self {
        let mut step_statuses = HashMap::new();
        for name in &step_names {
            step_statuses.insert(name.clone(), StepStatus::Pending);
        }

        Self {
            execution_id,
            workflow_name,
            phase: WorkflowPhase::Running,
            current_step: initial_step,
            step_statuses,
            context,
            started_at: chrono::Utc::now().to_rfc3339(),
            elapsed: Duration::ZERO,
            overall_progress: 0.0,
        }
    }

    /// Returns the names of all steps that have completed successfully.
    pub fn completed_steps(&self) -> Vec<String> {
        self.step_statuses
            .iter()
            .filter(|(_, status)| matches!(status, StepStatus::Completed))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Returns the names of all steps that were skipped.
    pub fn skipped_steps(&self) -> Vec<String> {
        self.step_statuses
            .iter()
            .filter(|(_, status)| matches!(status, StepStatus::Skipped { .. }))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Returns the names of all steps still pending.
    pub fn pending_steps(&self) -> Vec<String> {
        self.step_statuses
            .iter()
            .filter(|(_, status)| matches!(status, StepStatus::Pending))
            .map(|(name, _)| name.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_has_running_phase() {
        let state = WorkflowState::new(
            WorkflowExecutionId("test-1".to_string()),
            "test-workflow".to_string(),
            "step1".to_string(),
            vec!["step1".to_string(), "step2".to_string()],
            WorkflowContext::new(),
        );
        assert_eq!(state.phase, WorkflowPhase::Running);
    }

    #[test]
    fn new_state_marks_all_steps_pending() {
        let state = WorkflowState::new(
            WorkflowExecutionId("test-1".to_string()),
            "wf".to_string(),
            "a".to_string(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            WorkflowContext::new(),
        );
        assert_eq!(state.step_statuses.len(), 3);
        assert!(state
            .step_statuses
            .values()
            .all(|s| matches!(s, StepStatus::Pending)));
    }

    #[test]
    fn completed_steps_returns_only_completed() {
        let mut state = WorkflowState::new(
            WorkflowExecutionId("test-1".to_string()),
            "wf".to_string(),
            "a".to_string(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            WorkflowContext::new(),
        );
        state
            .step_statuses
            .insert("a".to_string(), StepStatus::Completed);
        state.step_statuses.insert(
            "b".to_string(),
            StepStatus::Failed {
                error: "oops".to_string(),
                retries_attempted: 0,
            },
        );

        let completed = state.completed_steps();
        assert_eq!(completed, vec!["a".to_string()]);
    }

    #[test]
    fn state_serialization_round_trip() {
        let state = WorkflowState::new(
            WorkflowExecutionId("ser-test".to_string()),
            "wf".to_string(),
            "start".to_string(),
            vec!["start".to_string(), "end".to_string()],
            WorkflowContext::new(),
        );
        let json = serde_json::to_string(&state).expect("serialize");
        let restored: WorkflowState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.execution_id, state.execution_id);
        assert_eq!(restored.workflow_name, "wf");
        assert_eq!(restored.phase, WorkflowPhase::Running);
    }
}
