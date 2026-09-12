use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::cancellation::CancellationToken;
use crate::definition::WorkflowDefinition;
use crate::error::{RollbackStatus, WorkflowError, WorkflowErrorReport};
use crate::error_policy::{self, ErrorPolicy, ErrorStrategy};
use crate::progress::{self, ProgressReporter};
use crate::state::{StepStatus, WorkflowPhase, WorkflowState};
use crate::step::{CompensatingAction, WorkflowEventDispatcher, WorkflowStep};

use super::WorkflowResult;

/// Internal execution loop for a workflow.
pub(super) async fn execute_workflow(
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
                // Step implementation missing -- skip with error
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
