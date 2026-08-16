//! Workflow engine error types.
//!
//! All errors follow the `[workflow] operation: description` format
//! per workbench error message standards.

use std::collections::HashMap;

/// Errors produced by the workflow engine.
///
/// Addresses: Cross-cutting error format: `[workflow] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WorkflowError {
    /// Workflow definition validation failed.
    #[error("[workflow] definition: {description}")]
    InvalidDefinition {
        /// Description of the structural problem.
        description: String,
    },

    /// No initial state defined in workflow.
    #[error("[workflow] definition: workflow '{name}' has no initial state")]
    NoInitialState {
        /// Workflow name.
        name: String,
    },

    /// Unreachable states detected in workflow graph.
    #[error("[workflow] definition: workflow '{name}' has unreachable states: {states:?}")]
    UnreachableStates {
        /// Workflow name.
        name: String,
        /// List of unreachable state names.
        states: Vec<String>,
    },

    /// No terminal states defined.
    #[error("[workflow] definition: workflow '{name}' has no terminal states")]
    NoTerminalStates {
        /// Workflow name.
        name: String,
    },

    /// Type incompatibility between connected steps.
    #[error("[workflow] definition: type mismatch between step '{from}' output and step '{to}' input for key '{key}'")]
    TypeMismatch {
        /// Source step name.
        from: String,
        /// Target step name.
        to: String,
        /// Context key with mismatched type.
        key: String,
    },

    /// Duplicate workflow name in registry.
    #[error("[workflow] registry: workflow '{name}' is already registered")]
    DuplicateName {
        /// The duplicate name.
        name: String,
    },

    /// Workflow not found in registry.
    #[error("[workflow] registry: workflow '{name}' not found")]
    NotFound {
        /// The name that was looked up.
        name: String,
    },

    /// Required parameter missing at invocation time.
    #[error(
        "[workflow] start: required parameter '{param}' not supplied for workflow '{workflow}'"
    )]
    MissingParameter {
        /// Workflow name.
        workflow: String,
        /// Missing parameter name.
        param: String,
    },

    /// Step execution failed.
    #[error("[workflow] step '{step}' failed: {description}")]
    StepFailed {
        /// Step name.
        step: String,
        /// Error description.
        description: String,
    },

    /// Step timed out during cancellation.
    #[error("[workflow] step '{step}' did not respond to cancellation within {timeout_seconds}s")]
    CancellationTimeout {
        /// Step name.
        step: String,
        /// Timeout duration in seconds.
        timeout_seconds: u64,
    },

    /// Compensating action failed during rollback.
    #[error("[workflow] compensate: rollback action for step '{step}' failed: {description}")]
    CompensationFailed {
        /// Step name.
        step: String,
        /// Failure description.
        description: String,
    },

    /// Checkpoint serialization/deserialization failed.
    #[error(
        "[workflow] checkpoint: {operation} failed for execution '{execution_id}': {description}"
    )]
    CheckpointError {
        /// The operation that failed (save, load, etc.).
        operation: String,
        /// The execution ID.
        execution_id: String,
        /// Failure description.
        description: String,
    },

    /// Checkpoint schema version mismatch.
    #[error("[workflow] checkpoint: incompatible schema version {found} (expected {expected}) for execution '{execution_id}'")]
    CheckpointSchemaMismatch {
        /// The execution ID.
        execution_id: String,
        /// Expected schema version.
        expected: u32,
        /// Found schema version.
        found: u32,
    },

    /// No matching transition predicate (and no default).
    #[error("[workflow] transition: no matching predicate at step '{step}' and no default transition defined")]
    NoMatchingTransition {
        /// Step name where evaluation failed.
        step: String,
    },

    /// Workflow does not support the requested operation.
    #[error("[workflow] operation: workflow '{name}' does not support {operation}")]
    UnsupportedOperation {
        /// Workflow name.
        name: String,
        /// Unsupported operation description.
        operation: String,
    },

    /// Context key not found when expected by a step.
    #[error("[workflow] context: key '{key}' not found (expected by step '{step}')")]
    ContextKeyMissing {
        /// The missing key.
        key: String,
        /// The step that expected it.
        step: String,
    },

    /// I/O error during checkpoint storage.
    #[error("[workflow] io: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("[workflow] serialization: {0}")]
    Serialization(String),
}

/// Status of the rollback/compensating actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RollbackStatus {
    /// All compensating actions completed successfully.
    Completed,
    /// Some compensating actions failed.
    PartiallyCompleted {
        /// Descriptions of which compensations failed.
        failures: Vec<String>,
    },
    /// No compensating actions were defined.
    NotApplicable,
}

/// Comprehensive error report for a failed or partially-failed workflow.
///
/// Addresses: Requirement 5, criteria 8/9
#[derive(Debug, Clone)]
pub struct WorkflowErrorReport {
    /// Workflow name.
    pub workflow_name: String,
    /// The step that caused the failure.
    pub failed_step: String,
    /// Error description with full context chain.
    pub error_description: String,
    /// Steps that completed successfully before failure.
    pub completed_steps: Vec<String>,
    /// Steps that were skipped (continue-on-error).
    pub skipped_steps: Vec<String>,
    /// Steps that were not executed.
    pub pending_steps: Vec<String>,
    /// Rollback status.
    pub rollback_status: RollbackStatus,
    /// Compensating action failures (if any).
    pub compensation_failures: Vec<String>,
    /// Relevant context values at time of failure.
    pub context_snapshot: HashMap<String, String>,
    /// Total elapsed time.
    pub elapsed: std::time::Duration,
}
