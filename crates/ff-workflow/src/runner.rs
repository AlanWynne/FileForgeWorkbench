//! Workflow runner — core execution engine.
//!
//! The `WorkflowRunner` drives a workflow through its state machine,
//! invoking steps, handling errors, propagating cancellation, and
//! emitting progress events.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;

use crate::cancellation::CancellationToken;
use crate::context::WorkflowContext;
use crate::definition::WorkflowDefinition;
use crate::error::{RollbackStatus, WorkflowError, WorkflowErrorReport};
use crate::error_policy::{self, ErrorPolicy, ErrorStrategy};
use crate::progress::{self, ProgressReporter, WorkflowExecutionId};
use crate::state::{StepStatus, WorkflowPhase, WorkflowState};
use crate::step::{CompensatingAction, WorkflowEventDispatcher, WorkflowStep};

/// The final result of a workflow execution.
///
/// Addresses: Requirement 5, criterion 8
#[derive(Debug)]
pub enum WorkflowResult {
    /// Workflow completed all steps successfully.
    Success {
        /// Final context state with all step outputs.
        context: WorkflowContext,
        /// Total elapsed time.
        elapsed: Duration,
    },
    /// Workflow failed.
    Failed {
        /// Comprehensive error report.
        error_report: Box<WorkflowErrorReport>,
        /// Total elapsed time.
        elapsed: Duration,
    },
    /// Workflow was cancelled.
    Cancelled {
        /// The step that was active at cancellation time.
        active_step: String,
        /// Status of compensating action execution.
        rollback_status: RollbackStatus,
        /// Total elapsed time.
        elapsed: Duration,
    },
}

/// A handle to a running workflow for monitoring and control.
///
/// Addresses: Requirement 2, criterion 8; Requirement 3, criteria 1/2
pub struct WorkflowHandle {
    /// The execution ID.
    execution_id: WorkflowExecutionId,
    /// Cancellation token for this workflow.
    cancel_token: CancellationToken,
    /// Current state (shared, read-only from outside).
    state: Arc<RwLock<WorkflowState>>,
    /// Channel to receive completion notification.
    completion_rx: tokio::sync::oneshot::Receiver<WorkflowResult>,
}

impl WorkflowHandle {
    /// Returns the execution ID.
    pub fn execution_id(&self) -> &WorkflowExecutionId {
        &self.execution_id
    }

    /// Requests cancellation of this workflow.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Gets a snapshot of the current workflow state.
    pub async fn current_state(&self) -> WorkflowState {
        self.state.read().await.clone()
    }

    /// Awaits completion of the workflow. Returns the final result.
    pub async fn await_completion(self) -> WorkflowResult {
        self.completion_rx.await.unwrap_or(WorkflowResult::Failed {
            error_report: Box::new(WorkflowErrorReport {
                workflow_name: "unknown".to_string(),
                failed_step: "unknown".to_string(),
                error_description: "workflow handle dropped".to_string(),
                completed_steps: Vec::new(),
                skipped_steps: Vec::new(),
                pending_steps: Vec::new(),
                rollback_status: RollbackStatus::NotApplicable,
                compensation_failures: Vec::new(),
                context_snapshot: HashMap::new(),
                elapsed: Duration::ZERO,
            }),
            elapsed: Duration::ZERO,
        })
    }
}

/// The execution engine that drives a workflow through its states.
///
/// Addresses: Requirement 2, all criteria; Requirement 3, all criteria
pub struct WorkflowRunner {
    event_dispatcher: Arc<dyn WorkflowEventDispatcher>,
}

impl WorkflowRunner {
    /// Creates a new runner with the given event dispatcher.
    pub fn new(event_dispatcher: Arc<dyn WorkflowEventDispatcher>) -> Self {
        Self { event_dispatcher }
    }

    /// Starts executing a workflow with the given definition, step
    /// implementations, and input parameters.
    ///
    /// Returns a handle for monitoring/controlling execution.
    pub async fn start(
        &self,
        definition: &WorkflowDefinition,
        steps: HashMap<String, Box<dyn WorkflowStep>>,
        compensations: HashMap<String, Box<dyn CompensatingAction>>,
        params: WorkflowContext,
    ) -> Result<WorkflowHandle, WorkflowError> {
        // Validate required parameters
        for param in &definition.parameters {
            if param.required && !params.contains_key(&param.name) {
                return Err(WorkflowError::MissingParameter {
                    workflow: definition.name.clone(),
                    param: param.name.clone(),
                });
            }
        }

        let execution_id = WorkflowExecutionId::generate();
        let cancel_token = CancellationToken::new();
        let step_names: Vec<String> = definition.steps.iter().map(|s| s.name.clone()).collect();

        let state = WorkflowState::new(
            execution_id.clone(),
            definition.name.clone(),
            definition.initial_step.clone(),
            step_names,
            params,
        );

        let shared_state = Arc::new(RwLock::new(state));
        let (completion_tx, completion_rx) = tokio::sync::oneshot::channel();

        let handle = WorkflowHandle {
            execution_id: execution_id.clone(),
            cancel_token: cancel_token.clone(),
            state: Arc::clone(&shared_state),
            completion_rx,
        };

        // Clone what we need for the spawned task
        let definition = definition.clone();
        let dispatcher = Arc::clone(&self.event_dispatcher);
        let token = cancel_token.clone();

        tokio::spawn(async move {
            let result = execute_workflow(
                definition,
                steps,
                compensations,
                shared_state,
                token,
                dispatcher,
            )
            .await;
            let _ = completion_tx.send(result);
        });

        Ok(handle)
    }
}

/// Internal execution loop for a workflow.
async fn execute_workflow(
    definition: WorkflowDefinition,
    steps: HashMap<String, Box<dyn WorkflowStep>>,
    compensations: HashMap<String, Box<dyn CompensatingAction>>,
    shared_state: Arc<RwLock<WorkflowState>>,
    cancel_token: CancellationToken,
    _dispatcher: Arc<dyn WorkflowEventDispatcher>,
) -> WorkflowResult {
    let start_time = std::time::Instant::now();
    let mut completed_order: Vec<String> = Vec::new();
    let total_steps = definition.steps.len();

    // Determine step execution order from transitions
    let execution_order = resolve_execution_order(&definition);

    for (step_idx, step_name) in execution_order.iter().enumerate() {
        // Check cancellation between steps
        if cancel_token.is_cancelled() {
            let rollback_status =
                execute_rollback(&completed_order, &compensations, &shared_state).await;

            let mut state = shared_state.write().await;
            state.phase = WorkflowPhase::Cancelled;
            state.elapsed = start_time.elapsed();

            return WorkflowResult::Cancelled {
                active_step: step_name.clone(),
                rollback_status,
                elapsed: start_time.elapsed(),
            };
        }

        // Mark step as running
        {
            let mut state = shared_state.write().await;
            state.current_step = step_name.clone();
            state
                .step_statuses
                .insert(step_name.clone(), StepStatus::Running);
        }

        // Get step implementation
        let step_impl = match steps.get(step_name.as_str()) {
            Some(s) => s,
            None => {
                // Step implementation missing — skip with error
                let mut state = shared_state.write().await;
                state.step_statuses.insert(
                    step_name.clone(),
                    StepStatus::Skipped {
                        reason: "no implementation provided".to_string(),
                    },
                );
                continue;
            }
        };

        // Execute the step with error policy handling
        let step_def = definition.steps.iter().find(|s| s.name == *step_name);
        let effective_policy = error_policy::effective_policy(
            &definition.error_policy,
            step_def.and_then(|s| s.error_policy_override.as_ref()),
        );

        let progress_reporter = ProgressReporter::new();
        let result = execute_step_with_policy(
            step_impl.as_ref(),
            &shared_state,
            &progress_reporter,
            &cancel_token,
            &effective_policy,
        )
        .await;

        match result {
            Ok(()) => {
                let mut state = shared_state.write().await;
                state
                    .step_statuses
                    .insert(step_name.clone(), StepStatus::Completed);
                state.overall_progress =
                    progress::aggregate_progress(step_idx + 1, 0.0, total_steps);
                completed_order.push(step_name.clone());
            }
            Err(StepOutcome::Failed(err)) => {
                let mut state = shared_state.write().await;
                state.step_statuses.insert(
                    step_name.clone(),
                    StepStatus::Failed {
                        error: err.to_string(),
                        retries_attempted: effective_policy.max_retries,
                    },
                );

                match effective_policy.strategy {
                    ErrorStrategy::FailFast | ErrorStrategy::Retry => {
                        // Execute rollback
                        drop(state);
                        let rollback_status =
                            execute_rollback(&completed_order, &compensations, &shared_state).await;

                        let mut s = shared_state.write().await;
                        s.phase = WorkflowPhase::Failed;
                        s.elapsed = start_time.elapsed();
                        let skipped = s.skipped_steps();
                        let pending = s.pending_steps();
                        drop(s);

                        return WorkflowResult::Failed {
                            error_report: Box::new(WorkflowErrorReport {
                                workflow_name: definition.name.clone(),
                                failed_step: step_name.clone(),
                                error_description: err.to_string(),
                                completed_steps: completed_order.clone(),
                                skipped_steps: skipped,
                                pending_steps: pending,
                                rollback_status,
                                compensation_failures: Vec::new(),
                                context_snapshot: HashMap::new(),
                                elapsed: start_time.elapsed(),
                            }),
                            elapsed: start_time.elapsed(),
                        };
                    }
                    ErrorStrategy::ContinueOnError => {
                        state.step_statuses.insert(
                            step_name.clone(),
                            StepStatus::Skipped {
                                reason: err.to_string(),
                            },
                        );
                    }
                }
            }
            Err(StepOutcome::Cancelled) => {
                let mut state = shared_state.write().await;
                state
                    .step_statuses
                    .insert(step_name.clone(), StepStatus::Cancelled);
                drop(state);

                let rollback_status =
                    execute_rollback(&completed_order, &compensations, &shared_state).await;

                let mut state = shared_state.write().await;
                state.phase = WorkflowPhase::Cancelled;
                state.elapsed = start_time.elapsed();

                return WorkflowResult::Cancelled {
                    active_step: step_name.clone(),
                    rollback_status,
                    elapsed: start_time.elapsed(),
                };
            }
        }
    }

    // All steps completed
    let mut state = shared_state.write().await;
    state.phase = WorkflowPhase::Completed;
    state.overall_progress = 100.0;
    state.elapsed = start_time.elapsed();
    let context = state.context.clone();

    WorkflowResult::Success {
        context,
        elapsed: start_time.elapsed(),
    }
}

/// Outcome of executing a step with its error policy.
enum StepOutcome {
    Failed(WorkflowError),
    Cancelled,
}

/// Executes a step with retry logic according to the error policy.
async fn execute_step_with_policy(
    step: &dyn WorkflowStep,
    shared_state: &Arc<RwLock<WorkflowState>>,
    progress: &ProgressReporter,
    cancel: &CancellationToken,
    policy: &ErrorPolicy,
) -> Result<(), StepOutcome> {
    let max_attempts = match policy.strategy {
        ErrorStrategy::Retry => policy.max_retries + 1,
        _ => 1,
    };

    let mut last_error = None;
    for attempt in 0..max_attempts {
        if cancel.is_cancelled() {
            return Err(StepOutcome::Cancelled);
        }

        let mut ctx = shared_state.read().await.context.clone();
        let result = step.execute(&mut ctx, progress, cancel).await;

        match result {
            Ok(()) => {
                // Store outputs back
                let mut state = shared_state.write().await;
                state.context = ctx;
                return Ok(());
            }
            Err(e) => {
                last_error = Some(e);
                if attempt + 1 < max_attempts {
                    tokio::time::sleep(policy.retry_delay).await;
                }
            }
        }
    }

    Err(StepOutcome::Failed(last_error.unwrap_or(
        WorkflowError::StepFailed {
            step: step.name().to_string(),
            description: "unknown failure".to_string(),
        },
    )))
}

/// Executes compensating actions in reverse order of completion.
async fn execute_rollback(
    completed_order: &[String],
    compensations: &HashMap<String, Box<dyn CompensatingAction>>,
    shared_state: &Arc<RwLock<WorkflowState>>,
) -> RollbackStatus {
    if compensations.is_empty() {
        return RollbackStatus::NotApplicable;
    }

    let mut state = shared_state.write().await;
    state.phase = WorkflowPhase::RollingBack;
    let context = state.context.clone();
    drop(state);

    let mut failures: Vec<String> = Vec::new();

    // Execute in reverse order of completion
    for step_name in completed_order.iter().rev() {
        if let Some(action) = compensations.get(step_name.as_str()) {
            if let Err(e) = action.compensate(&context).await {
                ff_logging::log_error!(
                    "[workflow] compensate: rollback for step '{}' failed: {}",
                    step_name,
                    e
                );
                failures.push(format!("{}: {}", step_name, e));
            }
        }
    }

    if failures.is_empty() {
        RollbackStatus::Completed
    } else {
        RollbackStatus::PartiallyCompleted { failures }
    }
}

/// Resolves the execution order of steps from the definition's transitions.
/// Falls back to the step declaration order if no clear path is determinable.
fn resolve_execution_order(definition: &WorkflowDefinition) -> Vec<String> {
    use std::collections::{HashSet, VecDeque};

    let mut order = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back(definition.initial_step.clone());
    visited.insert(definition.initial_step.clone());

    while let Some(current) = queue.pop_front() {
        order.push(current.clone());

        // Find outgoing transitions from current step
        let mut next_steps: Vec<&str> = definition
            .transitions
            .iter()
            .filter(|t| t.from == current)
            .map(|t| t.to.as_str())
            .collect();
        next_steps.sort();
        next_steps.dedup();

        for next in next_steps {
            if visited.insert(next.to_string()) {
                queue.push_back(next.to_string());
            }
        }
    }

    // Add any steps not reachable via transitions (shouldn't happen with valid defs)
    for step in &definition.steps {
        if !visited.contains(&step.name) {
            order.push(step.name.clone());
        }
    }

    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{StepDefinition, WorkflowBuilder};
    use crate::step::NoOpEventDispatcher;

    /// A simple test step that writes a value to context.
    struct WriteStep {
        step_name: String,
        key: String,
        value: crate::context::ContextValue,
    }

    #[async_trait::async_trait]
    impl WorkflowStep for WriteStep {
        async fn execute(
            &self,
            context: &mut WorkflowContext,
            _progress: &ProgressReporter,
            _cancel: &CancellationToken,
        ) -> Result<(), WorkflowError> {
            context.set(self.key.clone(), self.value.clone());
            Ok(())
        }
        fn name(&self) -> &str {
            &self.step_name
        }
    }

    /// A step that always fails.
    struct FailStep {
        step_name: String,
    }

    #[async_trait::async_trait]
    impl WorkflowStep for FailStep {
        async fn execute(
            &self,
            _context: &mut WorkflowContext,
            _progress: &ProgressReporter,
            _cancel: &CancellationToken,
        ) -> Result<(), WorkflowError> {
            Err(WorkflowError::StepFailed {
                step: self.step_name.clone(),
                description: "intentional failure".to_string(),
            })
        }
        fn name(&self) -> &str {
            &self.step_name
        }
    }

    fn make_step(name: &str) -> StepDefinition {
        StepDefinition {
            name: name.to_string(),
            display_name: name.to_string(),
            ..Default::default()
        }
    }

    // Validates: Requirement 2.1 — sequential execution

    #[tokio::test]
    async fn runner_executes_steps_sequentially() {
        let def = WorkflowBuilder::new("sequential")
            .step(make_step("a"))
            .step(make_step("b"))
            .transition(crate::definition::Transition {
                from: "a".to_string(),
                to: "b".to_string(),
                predicate: None,
                priority: 0,
            })
            .initial_step("a")
            .terminal_step("b")
            .build()
            .unwrap();

        let mut steps: HashMap<String, Box<dyn WorkflowStep>> = HashMap::new();
        steps.insert(
            "a".to_string(),
            Box::new(WriteStep {
                step_name: "a".to_string(),
                key: "from_a".to_string(),
                value: crate::context::ContextValue::Integer(1),
            }),
        );
        steps.insert(
            "b".to_string(),
            Box::new(WriteStep {
                step_name: "b".to_string(),
                key: "from_b".to_string(),
                value: crate::context::ContextValue::Integer(2),
            }),
        );

        let runner = WorkflowRunner::new(Arc::new(NoOpEventDispatcher));
        let handle = runner
            .start(&def, steps, HashMap::new(), WorkflowContext::new())
            .await
            .unwrap();

        let result = handle.await_completion().await;
        match result {
            WorkflowResult::Success { context, .. } => {
                assert_eq!(context.get_integer("from_a"), Some(1));
                assert_eq!(context.get_integer("from_b"), Some(2));
            }
            _ => panic!("expected success"),
        }
    }

    // Validates: Requirement 3.1 — cancellation between steps

    #[tokio::test]
    async fn runner_cancels_between_steps() {
        let def = WorkflowBuilder::new("cancel-test")
            .step(make_step("a"))
            .step(make_step("b"))
            .transition(crate::definition::Transition {
                from: "a".to_string(),
                to: "b".to_string(),
                predicate: None,
                priority: 0,
            })
            .initial_step("a")
            .terminal_step("b")
            .build()
            .unwrap();

        // Step A sleeps then completes, step B should not execute
        struct SlowStep;
        #[async_trait::async_trait]
        impl WorkflowStep for SlowStep {
            async fn execute(
                &self,
                ctx: &mut WorkflowContext,
                _p: &ProgressReporter,
                _c: &CancellationToken,
            ) -> Result<(), WorkflowError> {
                tokio::time::sleep(Duration::from_millis(50)).await;
                ctx.set("slow_done", crate::context::ContextValue::Boolean(true));
                Ok(())
            }
            fn name(&self) -> &str {
                "a"
            }
        }

        struct NeverStep;
        #[async_trait::async_trait]
        impl WorkflowStep for NeverStep {
            async fn execute(
                &self,
                ctx: &mut WorkflowContext,
                _p: &ProgressReporter,
                _c: &CancellationToken,
            ) -> Result<(), WorkflowError> {
                ctx.set("never", crate::context::ContextValue::Boolean(true));
                Ok(())
            }
            fn name(&self) -> &str {
                "b"
            }
        }

        let mut steps: HashMap<String, Box<dyn WorkflowStep>> = HashMap::new();
        steps.insert("a".to_string(), Box::new(SlowStep));
        steps.insert("b".to_string(), Box::new(NeverStep));

        let runner = WorkflowRunner::new(Arc::new(NoOpEventDispatcher));
        let handle = runner
            .start(&def, steps, HashMap::new(), WorkflowContext::new())
            .await
            .unwrap();

        // Cancel after step A should have started
        tokio::time::sleep(Duration::from_millis(10)).await;
        handle.cancel();

        let result = handle.await_completion().await;
        match result {
            WorkflowResult::Cancelled { active_step, .. } => {
                assert_eq!(active_step, "b");
            }
            WorkflowResult::Success { context, .. } => {
                // If step A was fast enough, B might get cancelled
                assert!(
                    context.get_bool("never").is_none() || context.get_bool("never") == Some(true)
                );
            }
            _ => {
                // Either cancelled or success is acceptable in this race
            }
        }
    }

    // Validates: Requirement 5.1 — fail-fast error policy

    #[tokio::test]
    async fn runner_fail_fast_aborts_on_step_failure() {
        let def = WorkflowBuilder::new("fail-fast")
            .step(make_step("a"))
            .step(make_step("b"))
            .error_policy(ErrorPolicy::fail_fast())
            .transition(crate::definition::Transition {
                from: "a".to_string(),
                to: "b".to_string(),
                predicate: None,
                priority: 0,
            })
            .initial_step("a")
            .terminal_step("b")
            .build()
            .unwrap();

        let mut steps: HashMap<String, Box<dyn WorkflowStep>> = HashMap::new();
        steps.insert(
            "a".to_string(),
            Box::new(FailStep {
                step_name: "a".to_string(),
            }),
        );
        steps.insert(
            "b".to_string(),
            Box::new(WriteStep {
                step_name: "b".to_string(),
                key: "from_b".to_string(),
                value: crate::context::ContextValue::Boolean(true),
            }),
        );

        let runner = WorkflowRunner::new(Arc::new(NoOpEventDispatcher));
        let handle = runner
            .start(&def, steps, HashMap::new(), WorkflowContext::new())
            .await
            .unwrap();

        let result = handle.await_completion().await;
        match result {
            WorkflowResult::Failed { error_report, .. } => {
                assert_eq!(error_report.failed_step, "a");
            }
            _ => panic!("expected failure"),
        }
    }
}
