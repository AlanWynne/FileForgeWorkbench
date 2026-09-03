# Implementation Plan: Workflow Engine (`ff-workflow`)

## Overview

This plan covers the complete implementation of the `ff-workflow` crate — the state-machine-based workflow execution engine for FileForgeWorkbench. The workflow engine provides declarative workflow definitions, step-by-step execution with shared context, cooperative cancellation, real-time progress reporting, error recovery policies, a central registry for workflow discovery, and optional persistence for long-running operations.

This is a **Wave 2 (Platform Architecture)** sub-project. It depends on `ff-logging` (Wave 0) and integrates with `platform-core`, `command-framework`, and `plugin-architecture` (all Wave 2 peers).

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-workflow/Cargo.toml` with dependencies (tokio, serde, thiserror, crossbeam-channel, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-workflow/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `definition.rs`, `step.rs`, `state.rs`, `context.rs`, `runner.rs`, `cancellation.rs`, `progress.rs`, `error_policy.rs`, `compensation.rs`, `registry.rs`, `persistence.rs`, `checkpoint.rs`, `error.rs`
  - [x] 1.4 Add `ff-workflow` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Workflow context and typed key-value store
  - [x] 2.1 Define `WorkflowContext` struct with typed get/set accessors using `TypeId`-keyed storage
  - [x] 2.2 Implement `get::<T>(&self, key: &str) -> Option<&T>` and `set::<T>(&mut self, key: &str, value: T)`
  - [x] 2.3 Implement `Serialize`/`Deserialize` support for persistent workflows via trait bounds
  - [x] 2.4 Implement compile-time enforcement: persistent workflow contexts require all values to be `Serialize + Deserialize`
  - [x] 2.5 Write unit tests for typed access, overwrite semantics, and missing key behavior
  - Covers: Requirement 2 (AC 2.2), Requirement 7 (AC 7.8)

- [x] 3. Workflow step definition and type system
  - [x] 3.1 Define `WorkflowStep` trait with `execute(&self, ctx: &mut WorkflowContext, progress: &ProgressReporter, cancel: &CancellationToken) -> Result<StepOutput>`
  - [x] 3.2 Define `StepDescriptor` struct declaring input types (read from context) and output types (written to context)
  - [x] 3.3 Implement support for both sync and async step implementations via `AsyncWorkflowStep` trait
  - [x] 3.4 Implement `CompensatingAction` trait for steps that define rollback behavior
  - [x] 3.5 Write unit tests for step descriptor validation and type compatibility checks
  - Covers: Requirement 1 (AC 1.5), Requirement 2 (AC 2.3), Requirement 5 (AC 5.5)

- [x] 4. Workflow state machine and definition
  - [x] 4.1 Define `WorkflowDefinition` struct as a directed graph of states and transitions
  - [x] 4.2 Implement sequential step-sequencing mode (steps execute one after another)
  - [x] 4.3 Implement parallel step-sequencing mode (concurrent execution with join barrier)
  - [x] 4.4 Implement conditional transitions with predicates evaluated against `WorkflowContext`
  - [x] 4.5 Implement definition validation: exactly one initial state, at least one terminal state, no unreachable states
  - [x] 4.6 Implement builder API for constructing definitions from structured data
  - [x] 4.7 Implement input parameter declaration with types and optional defaults
  - [x] 4.8 Write unit tests for definition construction, validation success/failure cases
  - Covers: Requirement 1 (AC 1.1, 1.2, 1.3, 1.4, 1.5, 1.6)

- [x] 5. Cancellation token and cooperative cancellation
  - [x] 5.1 Define `CancellationToken` struct with atomic is_cancelled flag and waker notification
  - [x] 5.2 Implement `cancel()` method that sets the flag and notifies all waiters
  - [x] 5.3 Implement `is_cancelled()` polling method for sync steps
  - [x] 5.4 Implement `cancelled()` async method that resolves when cancellation is requested (for Tokio integration)
  - [x] 5.5 Implement configurable per-workflow cancellation timeout (default 5 seconds)
  - [x] 5.6 Implement propagation to child Tokio tasks and futures via token cloning
  - [x] 5.7 Write unit tests for cancellation signaling, timeout behavior, and propagation
  - Covers: Requirement 3 (AC 3.1, 3.4, 3.5)

- [x] 6. Progress reporting system
  - [x] 6.1 Define `ProgressEvent` struct with fields: workflow_name, step_name, step_index, total_steps, percentage, items_processed, total_items, status_message, elapsed_time, estimated_remaining
  - [x] 6.2 Implement determinate progress mode (known total with percentage calculation)
  - [x] 6.3 Implement indeterminate progress mode (unknown total with activity indicator flag)
  - [x] 6.4 Define `ProgressReporter` handle provided to steps for emitting intermediate progress
  - [x] 6.5 Implement progress aggregation: parent percentage = (completed_steps + current_step_fraction) / total_steps * 100
  - [x] 6.6 Implement progress throttling at 100ms maximum emission rate per workflow instance
  - [x] 6.7 Implement estimated time remaining calculation (items/second × remaining items)
  - [x] 6.8 Write unit tests for both progress modes, aggregation math, and throttle behavior
  - Covers: Requirement 4 (AC 4.1, 4.2, 4.3, 4.4, 4.6, 4.7, 4.8)

- [x] 7. Error policy and retry logic
  - [x] 7.1 Define `ErrorPolicy` enum: FailFast, ContinueOnError, Retry { max_attempts, delay }
  - [x] 7.2 Implement per-workflow default error policy configuration
  - [x] 7.3 Implement per-step error policy override that takes precedence over workflow-level policy
  - [x] 7.4 Implement retry logic: re-execute failed step up to max_attempts (default 3) with configurable delay (default 1s)
  - [x] 7.5 Implement escalation: after retries exhausted, apply next-level policy (skip or abort)
  - [x] 7.6 Implement user-interaction error mode: emit error event and wait for user response (Retry/Skip/Abort)
  - [x] 7.7 Write unit tests for each policy mode, retry exhaustion, and escalation paths
  - Covers: Requirement 5 (AC 5.1, 5.2, 5.3, 5.7)

- [x] 8. Compensating actions and rollback
  - [x] 8.1 Implement compensating action registration per step during definition
  - [x] 8.2 Implement rollback execution in reverse order of step completion when workflow aborts
  - [x] 8.3 Implement compensation failure handling: log ERROR, continue remaining compensations, collect all failures
  - [x] 8.4 Implement rollback on cancellation: execute compensating actions for completed steps in reverse order
  - [x] 8.5 Write unit tests for rollback ordering, partial rollback, and compensation failure resilience
  - Covers: Requirement 5 (AC 5.4, 5.5, 5.6), Requirement 3 (AC 3.3)

- [x] 9. Workflow runner — core execution engine
  - [x] 9.1 Implement `WorkflowRunner` that drives a workflow through its state machine
  - [x] 9.2 Implement sequential step execution with context passing between steps
  - [x] 9.3 Implement parallel step execution: spawn concurrent tasks with join barrier
  - [x] 9.4 Implement conditional transition evaluation against current context
  - [x] 9.5 Implement default transition fallback when no predicate matches (or transition to failure state)
  - [x] 9.6 Implement step-start and step-completion progress event emission
  - [x] 9.7 Implement cancellation check between steps and force-cancel timeout enforcement
  - [x] 9.8 Implement pause/resume: complete current step, persist state, stop advancing; resume from next pending step
  - [x] 9.9 Write unit tests for sequential, parallel, conditional flows, and pause/resume behavior
  - Covers: Requirement 2 (AC 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8), Requirement 3 (AC 3.2, 3.5)

- [x] 10. Error reporting and structured error context
  - [x] 10.1 Define `WorkflowError` enum covering step failure, cancellation, persistence, and definition errors
  - [x] 10.2 Implement structured error report: workflow name, failed step, error description with context chain, completed steps, skipped steps, rollback status
  - [x] 10.3 Implement error context preservation: original error, step name, step index, relevant context values at failure time
  - [x] 10.4 Integrate error reporting with `ff-logging` for structured diagnostic output
  - [x] 10.5 Write unit tests for error report completeness and context chain preservation
  - Covers: Requirement 5 (AC 5.8, 5.9)

- [x] 11. Workflow registry
  - [x] 11.1 Define `WorkflowRegistry` struct with thread-safe internal storage (RwLock)
  - [x] 11.2 Implement `register(name, definition)` with duplicate name rejection
  - [x] 11.3 Implement category tagging: each workflow declares one or more category tags
  - [x] 11.4 Implement query by name (exact match), by category, and by input parameter type capability
  - [x] 11.5 Implement plugin workflow lifecycle: register on plugin load, remove all on plugin unload
  - [x] 11.6 Implement metadata exposure: display name, description, categories, input params, capabilities (cancel, pause, persist)
  - [x] 11.7 Implement thread-safe concurrent access for registration, unregistration, and queries
  - [x] 11.8 Write unit tests for registration, duplicate rejection, query modes, and plugin lifecycle
  - Covers: Requirement 6 (AC 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7)

- [x] 12. Event bus integration for progress events
  - [x] 12.1 Implement `ProgressEvent` emission via the platform-core event bus
  - [x] 12.2 Implement event throttling logic (max one event per 100ms per workflow instance)
  - [x] 12.3 Implement coalescing of intermediate updates between throttle windows
  - [x] 12.4 Implement cancellation event emission (which step was active, partial results status)
  - [x] 12.5 Write unit tests for event emission, throttling, and coalescing behavior
  - Covers: Requirement 4 (AC 4.5, 4.6), Requirement 3 (AC 3.7)

- [x] 13. Workflow persistence and checkpointing
  - [x] 13.1 Implement `supports_persistence` flag on `WorkflowDefinition`
  - [x] 13.2 Implement checkpoint serialization: context, current step index, step completion status, progress counters
  - [x] 13.3 Implement checkpoint storage to configurable directory (default: session data directory)
  - [x] 13.4 Implement unique workflow execution ID generation for checkpoint identification
  - [x] 13.5 Implement checkpoint on pause and graceful application shutdown
  - [x] 13.6 Implement startup scan for incomplete checkpoints with resumable workflow metadata
  - [x] 13.7 Implement checkpoint deserialization and workflow resumption from next pending step
  - [x] 13.8 Implement invalid checkpoint handling: log ERROR, remove from storage, report to user
  - [x] 13.9 Implement automatic checkpoint cleanup: success → immediate delete, failure → retain for configurable period (default 7 days)
  - [x] 13.10 Write unit tests for checkpoint round-trip, resume flow, invalid checkpoint handling, and cleanup
  - Covers: Requirement 7 (AC 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8)

- [x] 14. Command framework integration
  - [x] 14.1 Implement workflow invocation via command framework: lookup by name, validate parameters, start execution
  - [x] 14.2 Implement parameter validation against workflow's declared input schema before execution
  - [x] 14.3 Implement UI cancellation entry points: cancel action, Escape key handling when progress dialog has focus
  - [x] 14.4 Write integration tests for command-to-workflow invocation flow
  - Covers: Requirement 6 (AC 6.5), Requirement 3 (AC 3.6)

- [x] 15. Built-in workflow definitions
  - [x] 15.1 Implement data transfer workflow definition (source → transform → destination)
  - [x] 15.2 Implement file import/export workflow definition (read → validate → convert → write)
  - [x] 15.3 Implement compare-merge workflow definition (load-pair → diff → resolve → apply)
  - [x] 15.4 Implement bulk rename workflow definition (scan → preview → confirm → apply)
  - [x] 15.5 Write unit tests verifying each built-in definition passes validation
  - Covers: Requirement 1 (AC 1.7)

- [x] 16. Plugin workflow registration API
  - [x] 16.1 Define plugin-facing workflow registration trait accessible via `PluginContext`
  - [x] 16.2 Implement plugin workflow registration that routes through `WorkflowRegistry`
  - [x] 16.3 Implement automatic cleanup of plugin-registered workflows on plugin unload
  - [x] 16.4 Write unit tests for plugin registration and cleanup lifecycle
  - Covers: Requirement 1 (AC 1.7), Requirement 6 (AC 6.3)

- [x] 17. Property-based tests
  - [x] 17.1 Write PBT: workflow definition validation property
  - [x] 17.2 Write PBT: progress aggregation correctness property
  - [x] 17.3 Write PBT: error policy determinism property
  - [x] 17.4 Write PBT: cancellation safety property
  - [x] 17.5 Write PBT: checkpoint round-trip property
  - [x] 17.6 Write PBT: registry uniqueness invariant property
  - [x] 17.7 Write PBT: context type safety property
  - [x] 17.8 Write PBT: retry exhaustion convergence property
  - Covers: All requirements via invariant verification

---

## Property-Based Test Definitions

### Property 1: Workflow Definition Validation Completeness

**Validates: Requirement 1.3**

- **Statement:** For any directed graph of states and transitions, the definition validator SHALL accept the graph if and only if it has exactly one initial state, at least one terminal state, and no unreachable states; all other graphs SHALL be rejected with a descriptive error.
- **Strategy:** Generate:
  - Number of states: [1, 20]
  - Transitions: random edges between states (including potentially invalid graphs)
  - Initial state: randomly assigned (0 or 1 or 2+ initial states)
  - Terminal states: randomly assigned (0 or 1+ terminal states)
- **Invariant:** `validate(graph).is_ok()` ⟺ `initial_count == 1 ∧ terminal_count >= 1 ∧ unreachable_count == 0`

### Property 2: Progress Aggregation Correctness

**Validates: Requirement 4.4**

- **Statement:** For any workflow with N steps where each step reports a progress percentage in [0, 100], the aggregated parent progress SHALL equal `(completed_steps + current_step_progress / 100.0) / total_steps * 100.0` and SHALL always be in the range [0, 100].
- **Strategy:** Generate:
  - Total steps: [1, 50]
  - Completed steps: [0, total_steps - 1]
  - Current step progress: f64 in [0.0, 100.0]
- **Invariant:** `aggregated == (completed + current/100.0) / total * 100.0` ∧ `0.0 <= aggregated <= 100.0`

### Property 3: Error Policy Determinism

**Validates: Requirement 5.1, 5.2**

- **Statement:** For any combination of workflow-level error policy and per-step override, the effective policy for a step SHALL be deterministic: if a step override exists it takes precedence; otherwise the workflow-level policy applies.
- **Strategy:** Generate:
  - Workflow policy: uniform from {FailFast, ContinueOnError, Retry(n)}
  - Per-step override: Option<ErrorPolicy> (None or Some with random policy)
  - Step count: [1, 20]
- **Invariant:** `effective_policy(step) == step.override.unwrap_or(workflow.policy)` for all steps

### Property 4: Cancellation Safety

**Validates: Requirement 3.1, 3.2**

- **Statement:** For any workflow execution that receives a cancellation signal between steps, the workflow SHALL transition to the cancelled state without executing any subsequent steps, and all completed steps SHALL have their compensating actions executed in reverse order.
- **Strategy:** Generate:
  - Total steps: [2, 15]
  - Cancellation point: random step index in [0, total_steps - 1] (cancel after this step completes)
  - Steps with compensating actions: random subset
- **Invariant:** After cancellation at step K: `executed_steps == steps[0..=K]` ∧ `compensated_steps == reverse(steps[0..=K].filter(has_compensation))`

### Property 5: Checkpoint Round-Trip

**Validates: Requirement 7.2, 7.5**

- **Statement:** For any workflow state (context values, current step index, completion statuses), serializing to a checkpoint and deserializing SHALL produce an equivalent state, and resumption SHALL continue from the next pending step.
- **Strategy:** Generate:
  - Context entries: [0, 20] key-value pairs with string/integer/bool values
  - Current step index: [0, total_steps - 1]
  - Step completion statuses: random assignment of Completed/Pending/Failed
- **Invariant:** `deserialize(serialize(state)) == state` ∧ `resume_step == first_pending_step_after(current_step)`

### Property 6: Registry Uniqueness Invariant

**Validates: Requirement 6.1**

- **Statement:** For any sequence of register/unregister operations, the registry SHALL never contain duplicate workflow names; a registration with an existing name SHALL be rejected and leave the registry unchanged.
- **Strategy:** Generate:
  - Operation sequence: [10, 100] operations of {Register(name, def), Unregister(name), Query(name)}
  - Names drawn from a pool of [3, 10] unique strings (to force collisions)
- **Invariant:** At all times `registry.names().is_unique()` ∧ `register(existing_name).is_err()` ∧ `registry_after_failed_register == registry_before`

### Property 7: Context Type Safety

**Validates: Requirement 2.2, Requirement 1.5**

- **Statement:** For any typed value stored in the workflow context under a key, retrieving that key with the correct type SHALL return the value, and retrieving with an incorrect type SHALL return None (not panic or corrupt state).
- **Strategy:** Generate:
  - Keys: random strings [1, 20]
  - Values: random selection from {i32, String, bool, Vec<u8>}
  - Retrieval type: random (matching or mismatching)
- **Invariant:** `ctx.get::<T>(key) == Some(value)` when T matches stored type; `ctx.get::<U>(key) == None` when U differs from stored type

### Property 8: Retry Exhaustion Convergence

**Validates: Requirement 5.3**

- **Statement:** For any step configured with retry policy (max_attempts in [1, 10]), the step SHALL be executed at most max_attempts times regardless of failure pattern; after exhaustion, the next-level policy (skip or abort) SHALL be applied exactly once.
- **Strategy:** Generate:
  - Max attempts: [1, 10]
  - Failure pattern: sequence of booleans (true=fail, false=succeed) of length >= max_attempts
  - Next-level policy: {Skip, Abort}
- **Invariant:** `execution_count <= max_attempts` ∧ (if all attempts fail then `next_policy_applied == true` ∧ `next_policy_applied_count == 1`)

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types", "tasks": ["2", "3", "5", "6", "7"], "dependsOn": [0] },
    { "id": 2, "label": "Definition and Validation", "tasks": ["4", "8", "10"], "dependsOn": [1] },
    { "id": 3, "label": "Execution Engine", "tasks": ["9", "12"], "dependsOn": [2] },
    { "id": 4, "label": "Registry and Persistence", "tasks": ["11", "13"], "dependsOn": [3] },
    { "id": 5, "label": "Integration and Built-ins", "tasks": ["14", "15", "16"], "dependsOn": [4] },
    { "id": 6, "label": "Property-Based Tests", "tasks": ["17"], "dependsOn": [5] }
  ]
}
```

---

## Notes

- This is a Wave 2 (Platform Architecture) crate depending on `ff-logging` (Wave 0)
- The workflow engine uses Tokio for async step execution but the runner state machine itself is driven synchronously for predictability
- `WorkflowContext` uses type-erased storage internally (`Box<dyn Any>`) with typed accessors — persistent workflows enforce `Serialize + Deserialize` bounds at compile time via a separate `PersistentWorkflowContext` wrapper
- Progress events are emitted via the platform-core event bus; the event bus trait is defined in `platform-core` and injected into the runner at construction time
- The cancellation token design follows `tokio_util::sync::CancellationToken` patterns but is owned by `ff-workflow` to avoid coupling to a specific Tokio utility version
- Built-in workflow definitions (Task 15) are structural templates — their step implementations will be provided by downstream crates (e.g., `file-operations`, `compare-and-merge`)
- Plugin workflow registration (Task 16) depends on the `PluginContext` trait from `plugin-architecture`; the integration is via a trait object to avoid circular dependencies
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The UI cancellation entry points (Task 14.3) define the interface contract; actual UI rendering is implemented by the GUI shell in a later wave

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Workflow Definition | AC 1.1–1.4 | Task 4 |
| Req 1: Workflow Definition | AC 1.5 | Tasks 3, 4 |
| Req 1: Workflow Definition | AC 1.6 | Task 4 |
| Req 1: Workflow Definition | AC 1.7 | Tasks 15, 16 |
| Req 2: Workflow Execution | AC 2.1 | Task 9 |
| Req 2: Workflow Execution | AC 2.2 | Tasks 2, 9 |
| Req 2: Workflow Execution | AC 2.3 | Tasks 3, 9 |
| Req 2: Workflow Execution | AC 2.4 | Task 9 |
| Req 2: Workflow Execution | AC 2.5 | Task 9 |
| Req 2: Workflow Execution | AC 2.6–2.7 | Tasks 6, 9 |
| Req 2: Workflow Execution | AC 2.8 | Task 9 |
| Req 3: Cancellation Support | AC 3.1 | Tasks 5, 9 |
| Req 3: Cancellation Support | AC 3.2 | Task 9 |
| Req 3: Cancellation Support | AC 3.3 | Task 8 |
| Req 3: Cancellation Support | AC 3.4 | Task 5 |
| Req 3: Cancellation Support | AC 3.5 | Tasks 5, 9 |
| Req 3: Cancellation Support | AC 3.6 | Task 14 |
| Req 3: Cancellation Support | AC 3.7 | Task 12 |
| Req 4: Progress Reporting | AC 4.1–4.3 | Task 6 |
| Req 4: Progress Reporting | AC 4.4 | Tasks 6, 9 |
| Req 4: Progress Reporting | AC 4.5 | Task 12 |
| Req 4: Progress Reporting | AC 4.6 | Tasks 6, 12 |
| Req 4: Progress Reporting | AC 4.7–4.8 | Task 6 |
| Req 5: Error Handling | AC 5.1–5.3 | Task 7 |
| Req 5: Error Handling | AC 5.4–5.6 | Task 8 |
| Req 5: Error Handling | AC 5.7 | Task 7 |
| Req 5: Error Handling | AC 5.8–5.9 | Task 10 |
| Req 6: Workflow Registry | AC 6.1–6.2 | Task 11 |
| Req 6: Workflow Registry | AC 6.3 | Tasks 11, 16 |
| Req 6: Workflow Registry | AC 6.4 | Task 11 |
| Req 6: Workflow Registry | AC 6.5 | Task 14 |
| Req 6: Workflow Registry | AC 6.6–6.7 | Task 11 |
| Req 7: Workflow Persistence | AC 7.1 | Task 13 |
| Req 7: Workflow Persistence | AC 7.2–7.3 | Task 13 |
| Req 7: Workflow Persistence | AC 7.4–7.5 | Task 13 |
| Req 7: Workflow Persistence | AC 7.6 | Task 13 |
| Req 7: Workflow Persistence | AC 7.7 | Task 13 |
| Req 7: Workflow Persistence | AC 7.8 | Tasks 2, 13 |
