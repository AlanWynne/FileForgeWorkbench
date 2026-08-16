//! Workflow step trait definitions.
//!
//! The `WorkflowStep` trait is implemented by all step implementations.
//! Steps execute within the workflow runner and have access to the shared
//! context, progress reporter, and cancellation token.

use async_trait::async_trait;

use crate::cancellation::CancellationToken;
use crate::context::WorkflowContext;
use crate::error::WorkflowError;
use crate::progress::ProgressReporter;

/// The execution trait for a workflow step.
///
/// Implementors define the step's logic. Steps read from and write to
/// the shared `WorkflowContext`, report progress via the `ProgressReporter`,
/// and check cancellation via the `CancellationToken`.
///
/// Addresses: Requirement 2, criteria 1/3/7
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    /// Executes the step.
    ///
    /// Reads from and writes to the workflow context. The progress reporter
    /// handle allows intermediate progress updates. The cancellation token
    /// should be checked periodically for cooperative cancellation.
    ///
    /// Addresses: Requirement 2, criteria 2/3/7; Requirement 3, criterion 1
    async fn execute(
        &self,
        context: &mut WorkflowContext,
        progress: &ProgressReporter,
        cancel: &CancellationToken,
    ) -> Result<(), WorkflowError>;

    /// Human-readable name for diagnostics and progress reporting.
    fn name(&self) -> &str;
}

/// The trait for compensating (rollback) actions.
///
/// Compensating actions undo the effects of a completed step when a
/// workflow is aborted or cancelled. They execute in reverse order of
/// step completion.
///
/// Addresses: Requirement 5, criteria 4/5/6
#[async_trait]
pub trait CompensatingAction: Send + Sync {
    /// Executes the compensating action to undo a step's effects.
    ///
    /// Must not panic. If this fails, the error is logged and rollback
    /// continues with remaining compensating actions.
    async fn compensate(&self, context: &WorkflowContext) -> Result<(), WorkflowError>;

    /// Human-readable description of what this action undoes.
    fn description(&self) -> &str;
}

/// Trait abstracting the event bus interface for workflow progress events.
///
/// Implemented by platform-core to bridge to the Event Bus. The workflow
/// engine does not depend on ff-core directly — it receives this trait
/// object at construction time.
///
/// Addresses: Requirement 4, criterion 5
#[async_trait]
pub trait WorkflowEventDispatcher: Send + Sync {
    /// Dispatches a progress event to the event bus.
    fn dispatch_progress(&self, event: crate::progress::ProgressEvent);

    /// Dispatches a workflow error event (for user interaction).
    fn dispatch_error(
        &self,
        execution_id: &crate::progress::WorkflowExecutionId,
        error: &WorkflowError,
        options: Vec<crate::error_policy::UserErrorAction>,
    );

    /// Awaits user's response to an error dialog.
    async fn await_user_response(
        &self,
        execution_id: &crate::progress::WorkflowExecutionId,
    ) -> crate::error_policy::UserErrorAction;
}

/// A no-op event dispatcher for testing and workflows that don't need
/// event bus integration.
#[derive(Debug, Clone)]
pub struct NoOpEventDispatcher;

#[async_trait]
impl WorkflowEventDispatcher for NoOpEventDispatcher {
    fn dispatch_progress(&self, _event: crate::progress::ProgressEvent) {}

    fn dispatch_error(
        &self,
        _execution_id: &crate::progress::WorkflowExecutionId,
        _error: &WorkflowError,
        _options: Vec<crate::error_policy::UserErrorAction>,
    ) {
    }

    async fn await_user_response(
        &self,
        _execution_id: &crate::progress::WorkflowExecutionId,
    ) -> crate::error_policy::UserErrorAction {
        crate::error_policy::UserErrorAction::Abort
    }
}
