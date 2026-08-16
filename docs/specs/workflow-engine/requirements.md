# Requirements Document

## Introduction

This feature specifies the Workflow Engine for FileForgeWorkbench (`ff-workflow` crate). The workflow engine provides a state-machine-based execution framework for multi-step operations that require sequencing, progress reporting, cancellation, error recovery, and optional persistence. Complex operations in the workbench — such as data transfer, file import/export, compare-merge, and bulk rename — are modelled as workflows rather than monolithic functions, enabling consistent user experience (progress indication, cancel button, error recovery dialogs) across all long-running operations.

The workflow engine is a **platform-core subsystem** — it operates within the GUI-independent layer and communicates with the GUI shell through the event bus. It depends on `ff-logging` for diagnostic output, integrates with the `command-framework` for workflow invocation, and is accessible to plugins via the `plugin-architecture` trait. All workflow I/O operations use the async I/O principle (Tokio-based) per Architecture Brief §9.

**Source references:**
- **WB** = Workbench Architecture Brief §11 (workflow state machines, progress, cancellation)
- **FFE** = FileForgeEditor (background-io, compare-merge — operations now modelled as workflows)

## Glossary

- **Workflow**: A multi-step operation modelled as a state machine with a defined set of states, transitions, and execution logic. A workflow accepts input parameters, executes steps in sequence (or parallel/conditional), reports progress, and produces a result or error. [WB]
- **Workflow_Definition**: A declarative description of a workflow's structure — its states, transitions, step implementations, error policy, and cancellation behaviour. Definitions are data-driven, not hardcoded control flow. [WB]
- **Workflow_Step**: A single unit of work within a workflow. A step may be synchronous or async, reports its own progress, and produces an output that feeds into the workflow context. [WB]
- **Workflow_State**: The current position of a workflow within its state machine — which step is active, which steps are completed, and what the next transition will be. [WB]
- **Workflow_Context**: A typed key-value store that carries state between steps within a single workflow execution. Steps read inputs from and write outputs to the context. [WB]
- **Workflow_Runner**: The execution engine that drives a workflow through its states, invoking steps, handling errors, propagating cancellation, and emitting progress events. [WB]
- **Cancellation_Token**: A cooperative signal that indicates a workflow should stop execution gracefully. Propagated to all async operations within the workflow. [WB]
- **Progress_Event**: A structured event emitted by the workflow runner or individual steps, conveying progress information (percentage, status message, items processed) to the UI via the event bus. [WB]
- **Workflow_Registry**: A central lookup table where workflows are registered by name and category, enabling discovery by the command framework, plugins, and UI. [WB]
- **Error_Policy**: A per-workflow configuration that determines how step failures are handled: fail-fast (abort immediately), continue-on-error (skip failed step), or retry (attempt the step again). [WB]
- **Compensating_Action**: A rollback operation defined by a workflow that undoes the effects of a completed step when the workflow is aborted or a later step fails. [WB]
- **Workflow_Checkpoint**: A serialized snapshot of a workflow's state (context, current step, progress) that enables resumption after pause or application restart. [WB]
- **Event_Bus**: The platform-core publish/subscribe messaging system through which workflow progress events are emitted for UI consumption. [WB]

## Requirements

### Requirement 1: Workflow Definition

**User Story:** As a workbench developer, I want to define workflows declaratively as state machines with typed steps and transitions, so that complex operations are composable, testable, and maintainable without hardcoded control flow.

**Source:** WB Architecture Brief §11 — workflow state machines, declarative definition. [WB]

#### Acceptance Criteria

1. THE Workflow_Definition SHALL describe a workflow as a directed graph of states and transitions, where each state corresponds to a named Workflow_Step and transitions define the conditions under which execution advances to the next state.
2. THE Workflow_Definition SHALL support three step-sequencing modes: sequential (steps execute one after another), parallel (multiple steps execute concurrently with a join barrier), and conditional (transitions chosen based on a predicate evaluated against the Workflow_Context).
3. WHEN a Workflow_Definition is constructed, THE workflow engine SHALL validate that the definition has exactly one initial state, at least one terminal state (success or failure), and no unreachable states; IF validation fails, THEN THE workflow engine SHALL return an error describing the structural problem.
4. THE Workflow_Definition SHALL be data-driven: definitions are constructed from structured data (Rust builder API or deserialized from a configuration format) — not expressed as hardcoded `match` or `if/else` chains in application code.
5. EACH Workflow_Step within a definition SHALL declare its expected input types (read from Workflow_Context) and output types (written to Workflow_Context), enabling the workflow engine to verify type compatibility between connected steps at definition time.
6. THE Workflow_Definition SHALL support parameterization: each workflow declares a set of named input parameters with types and optional default values that must be supplied when the workflow is started.
7. THE workflow engine SHALL provide built-in workflow definitions for common workbench operations including: data transfer, file import/export, compare-merge, and bulk rename; additional workflows SHALL be registerable by plugins.

---

### Requirement 2: Workflow Execution

**User Story:** As a workbench developer, I want the workflow runner to execute workflow steps in the defined sequence with shared context, so that each step can build upon the outputs of previous steps and the overall operation proceeds predictably.

**Source:** WB Architecture Brief §11 — step sequencing, context passing, async execution. [WB]

#### Acceptance Criteria

1. THE Workflow_Runner SHALL execute steps in the order defined by the workflow's state machine, advancing through transitions after each step completes successfully.
2. WHEN a step completes, THE Workflow_Runner SHALL store the step's output in the Workflow_Context, making it available to subsequent steps via typed accessors.
3. THE Workflow_Runner SHALL support both synchronous and asynchronous (Tokio-based) step implementations; async steps SHALL be spawned on the Tokio runtime and SHALL NOT block the thread that drives the workflow state machine.
4. WHEN parallel steps are defined, THE Workflow_Runner SHALL spawn all parallel steps concurrently and wait for all to complete (or fail) before advancing past the parallel group's join barrier.
5. WHEN a conditional transition is encountered, THE Workflow_Runner SHALL evaluate the transition predicate against the current Workflow_Context and follow the branch whose predicate returns true; IF no predicate matches, THEN THE Workflow_Runner SHALL follow a declared default transition or transition to a failure state with a descriptive error.
6. THE Workflow_Runner SHALL emit a Progress_Event at the start and completion of each step, including the step name, step index, total step count, and the step's own progress percentage (0% at start, 100% at completion).
7. EACH Workflow_Step SHALL be able to emit intermediate Progress_Events during its execution (e.g., percentage complete, items processed out of total, status message) via a progress reporter handle provided by the Workflow_Runner.
8. THE Workflow_Runner SHALL support pause and resume: WHEN a workflow is paused, THE runner SHALL complete the currently executing step, persist the workflow state, and stop advancing; WHEN resumed, THE runner SHALL continue from the next pending step using the persisted context.

---

### Requirement 3: Cancellation Support

**User Story:** As a user, I want to cancel any long-running workflow operation gracefully, so that I can regain control of the application without data corruption or resource leaks.

**Source:** WB Architecture Brief §11 — cooperative cancellation, graceful shutdown. [WB]

#### Acceptance Criteria

1. ALL workflows SHALL support cooperative cancellation via a Cancellation_Token that is checked between steps and propagated to async operations within each step.
2. WHEN cancellation is requested, THE Workflow_Runner SHALL allow the currently executing step to complete (or reach its next cancellation check point within the step), then transition the workflow to a cancelled state without executing subsequent steps.
3. WHEN a workflow transitions to the cancelled state, THE Workflow_Runner SHALL execute any cleanup or compensating actions defined for completed steps (in reverse order of completion) before emitting the final cancellation Progress_Event.
4. THE Cancellation_Token SHALL be propagated to all Tokio tasks, futures, and I/O operations spawned by the workflow, enabling async operations to detect cancellation and return early with an appropriate error.
5. WHEN a Workflow_Step receives a cancellation signal during async I/O, THE step SHALL abort the I/O operation within a bounded time (configurable per-workflow, default 5 seconds) and return a cancellation result; IF the step does not respond within the timeout, THE Workflow_Runner SHALL consider the step force-cancelled and proceed with cleanup.
6. THE workflow engine SHALL provide UI-facing cancellation entry points: a cancel button on the progress dialog, a cancel action in the status bar progress indicator, and the Escape key (when the progress dialog has focus).
7. WHEN cancellation completes, THE Workflow_Runner SHALL emit a Progress_Event indicating the workflow was cancelled, including which step was active at the time of cancellation and whether partial results were preserved or rolled back.

---

### Requirement 4: Progress Reporting

**User Story:** As a user, I want to see real-time progress information for long-running operations, so that I know how much work remains and can make informed decisions about waiting or cancelling.

**Source:** WB Architecture Brief §11 — progress reporting, event bus integration. [WB]

#### Acceptance Criteria

1. THE workflow engine SHALL support two progress modes: determinate (known total — reports percentage, items processed, and total items) and indeterminate (unknown total — reports only that work is in progress with a status message).
2. WHEN a workflow step reports determinate progress, THE Progress_Event SHALL include: percentage complete (0–100), items processed count, total items count, current status message, and optional estimated time remaining.
3. WHEN a workflow step reports indeterminate progress, THE Progress_Event SHALL include: a status message describing the current activity, and a flag indicating indeterminate mode so the UI can display a spinning or pulsing indicator.
4. THE Workflow_Runner SHALL aggregate progress from child steps into parent workflow progress: the parent percentage SHALL be calculated as `(completed_steps + current_step_fraction) / total_steps * 100`, where `current_step_fraction` is the active step's own reported percentage divided by 100.
5. THE Workflow_Runner SHALL emit Progress_Events via the platform-core event bus, enabling any subscribed UI component (progress dialog, status bar, notification area) to display progress without direct coupling to the workflow engine.
6. THE workflow engine SHALL throttle Progress_Event emission to at most one event per 100 milliseconds per workflow instance, coalescing intermediate updates, to prevent flooding the event bus and the UI render loop.
7. WHEN a workflow step can calculate estimated time remaining (based on items processed per second and items remaining), THE Progress_Event SHALL include the estimate in seconds; IF the estimate cannot be calculated (insufficient data or indeterminate progress), THE field SHALL be absent.
8. THE Progress_Event SHALL include a workflow-level summary: workflow name, total step count, current step index, current step name, overall percentage, and elapsed time since workflow start.

---

### Requirement 5: Error Handling and Recovery

**User Story:** As a user, I want workflow errors to be handled gracefully with options to retry, skip, or abort, so that a single step failure does not necessarily destroy the entire operation's progress.

**Source:** WB Architecture Brief §11 — error handling, recovery, rollback. [WB]

#### Acceptance Criteria

1. WHEN a Workflow_Step fails, THE Workflow_Runner SHALL consult the workflow's Error_Policy to determine the action: fail-fast (abort the workflow immediately), continue-on-error (mark the step as failed, skip it, and advance to the next step), or retry (re-execute the step up to a configured maximum retry count).
2. THE Error_Policy SHALL be configurable per-workflow at definition time, with an optional per-step override that takes precedence over the workflow-level policy.
3. WHEN the Error_Policy specifies retry, THE Workflow_Runner SHALL re-execute the failed step up to the configured maximum retry count (default 3), with an optional configurable delay between retries (default 1 second); IF all retries are exhausted, THEN THE runner SHALL treat the step as a permanent failure and apply the next-level policy (skip or abort).
4. WHEN the Error_Policy specifies fail-fast and a step fails, THE Workflow_Runner SHALL transition immediately to the workflow's failure state, executing compensating actions for all previously completed steps (in reverse order) if rollback is defined.
5. THE Workflow_Definition SHALL support compensating actions: each step MAY define a compensating action that undoes its effects; WHEN rollback is triggered, THE Workflow_Runner SHALL execute compensating actions in reverse order of step completion.
6. IF a compensating action itself fails during rollback, THEN THE Workflow_Runner SHALL log the failure at ERROR level, continue executing remaining compensating actions, and include all compensating action failures in the final error report.
7. WHEN a step failure occurs and the workflow is configured for user interaction, THE Workflow_Runner SHALL emit an error event that the UI can present as a dialog with options: Retry (re-execute the step), Skip (continue to next step), and Abort (cancel the workflow); THE runner SHALL wait for the user's response before proceeding.
8. THE error report for a failed or partially-failed workflow SHALL include: workflow name, the step that failed, the error description with full context chain, a list of steps that completed successfully, a list of steps that were skipped, and the rollback status (completed, partially completed, or not applicable).
9. ALL error context SHALL be preserved in a structured format accessible to the logging subsystem and to any error-display UI component, including the original error, the step name, the step index, and any relevant Workflow_Context values at the time of failure.

---

### Requirement 6: Workflow Registry

**User Story:** As a workbench developer, I want workflows to be registered in a central registry by name and category, so that the command framework, plugins, and UI can discover and invoke workflows without hardcoded references.

**Source:** WB Architecture Brief §11 — workflow registry, plugin extensibility. [WB]

#### Acceptance Criteria

1. THE Workflow_Registry SHALL maintain a mapping of workflow names to their Workflow_Definitions, where each name is unique within the registry; IF a duplicate name is registered, THEN THE registry SHALL return an error and reject the registration.
2. THE Workflow_Registry SHALL support categorization: each registered workflow SHALL declare one or more category tags (e.g., "file-operation", "data-transfer", "refactoring") enabling queries by category.
3. WHEN a plugin is loaded, THE plugin SHALL be able to register custom Workflow_Definitions with the Workflow_Registry via the Plugin_Context; WHEN the plugin is unloaded, THE registry SHALL remove all workflows registered by that plugin.
4. THE Workflow_Registry SHALL support querying available workflows by: name (exact match), category (all workflows in a category), and capability (workflows that accept a given input parameter type).
5. WHEN a workflow is invoked via the command framework, THE command handler SHALL look up the workflow by name in the Workflow_Registry, validate the supplied parameters against the workflow's declared input parameter schema, and start execution via the Workflow_Runner.
6. THE Workflow_Registry SHALL expose workflow metadata for UI consumption: display name, description, category tags, expected input parameters with types and descriptions, and whether the workflow supports cancellation, pause/resume, and persistence.
7. THE Workflow_Registry SHALL be thread-safe: registration, unregistration, and queries SHALL be safe to perform from any thread without external synchronization.

---

### Requirement 7: Workflow Persistence (Long-Running)

**User Story:** As a user, I want long-running workflows to survive application restarts, so that I do not lose progress on operations that take significant time (large data transfers, bulk processing).

**Source:** WB Architecture Brief §11 — workflow persistence, checkpoint, resume. [WB]

#### Acceptance Criteria

1. THE Workflow_Definition SHALL declare whether a workflow supports persistence via a `supports_persistence` flag; only workflows that declare persistence support SHALL be eligible for checkpoint and resume operations.
2. WHEN a persistent workflow is paused or the application is shutting down gracefully, THE Workflow_Runner SHALL serialize the workflow's current state (Workflow_Context, current step index, step completion status, progress counters) to a checkpoint in the workflow storage.
3. THE workflow storage SHALL persist checkpoints to a platform-appropriate location (configurable via `workflow.storage_directory` in Workbench_Config, defaulting to the session data directory); each checkpoint SHALL be identified by a unique workflow execution ID.
4. WHEN the application starts, THE Workflow_Runner SHALL scan the workflow storage for incomplete checkpoints and present them to the user (via the UI) as resumable workflows, displaying the workflow name, progress at time of checkpoint, and elapsed time.
5. WHEN a user chooses to resume a checkpointed workflow, THE Workflow_Runner SHALL deserialize the checkpoint, restore the Workflow_Context, and continue execution from the next pending step; THE first Progress_Event after resumption SHALL indicate that the workflow was resumed from a checkpoint.
6. IF a checkpoint cannot be deserialized (due to schema changes, corruption, or missing workflow definition), THEN THE Workflow_Runner SHALL log an ERROR-level record, remove the invalid checkpoint from storage, and present the failure to the user with an explanation.
7. THE Workflow_Runner SHALL automatically clean up checkpoint files for workflows that complete successfully (either success or intentional cancellation with no resume needed); failed workflows SHALL retain their checkpoint for a configurable retention period (default 7 days) before automatic cleanup.
8. ALL data written to the Workflow_Context by steps SHALL implement serialization (via `serde::Serialize` and `serde::Deserialize`) for workflows that declare persistence support; IF a step attempts to write a non-serializable value to the context of a persistent workflow, THEN THE workflow engine SHALL return a compile-time error (enforced via trait bounds).

