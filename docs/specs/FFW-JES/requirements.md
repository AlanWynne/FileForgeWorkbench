# Requirements Document

## Introduction

This feature specifies **FFW-JES** (FileForge Workbench Job Entry Subsystem) — a cross-platform JES/SDSF-style emulator delivered as a workbench plugin (`ff-jes` crate). FFW-JES emulates mainframe batch processing on Windows, Linux, and macOS: job submission, queue management, initiator-based execution, SDSF-style monitoring, dataset allocation via the catalog, and retained job output.

The subsystem integrates with the workbench platform through:
- **Plugin Architecture** (`ff-plugin`): registers as a `FileForgePlugin`, contributes panels, commands, and APIs
- **Command Framework** (`ff-command`): all job and catalog operations are registered commands under the `jes.*` namespace
- **Layout and Docking** (`ff-layout`): Job Monitor panels implement `DockablePanel`
- **Workflow Engine** (`ff-workflow`): multi-step job execution modelled as state-machine workflows
- **Virtual File System** (`ff-vfs`): dataset resolution and job log access flow through VFS
- **Dataset Catalog** (`ff-dataset-catalog`): DSN resolution leverages the existing catalog subsystem
- **Dataset Allocator** (`ff-dataset-allocator`): disposition handling (NEW/OLD/SHR/MOD) and allocation semantics for job DD statements

**Source references:**
- **JES** = FFW-JES EARS Requirements document
- **FFW-ARCH** = FileForgeWorkbench architecture specs

## Glossary

- **JesPlugin**: The top-level `FileForgePlugin` implementation that bootstraps the JES subsystem, registers panels, commands, and APIs. [FFW-ARCH]
- **Job**: A unit of batch work defined by a job definition (FFJCL), submitted to the queue, dispatched to an initiator, and producing output. [JES]
- **Job_ID**: A unique identifier assigned to each submitted job. [JES]
- **Job_Queue**: The ordered collection of jobs awaiting execution (Input Queue). [JES]
- **Initiator**: A worker thread/process that executes a dispatched job. Desktop equivalent of a mainframe initiator. [JES]
- **Initiator_Pool**: The managed collection of initiators with configurable capacity. [JES]
- **Job_Monitor**: The SDSF-style DockablePanel displaying job queues, status, and logs. [JES]
- **Job_Log**: The complete execution record for a job: scheduling messages, allocation messages, step output, return codes. [JES]
- **SYSOUT**: System output produced by a job step, retained in the spool. [JES]
- **Job_Status**: Enum of lifecycle states: QUEUED, HELD, ACTIVE, COMPLETED, FAILED, CANCELLED. [JES]
- **Scheduler**: The component that selects eligible queued jobs and dispatches them to available initiators. [JES]
- **Retention_Policy**: Configurable rules governing how long completed job output is retained before purge. [JES]
- **FFJCL**: FileForge Job Control Language — the desktop job definition format. [JES]
- **GDG**: Generation Data Group — a dataset with multiple generations referenced by relative offset. [JES]

---

## Requirements

### Requirement 1: Plugin Registration and Lifecycle

**User Story:** As a workbench user, I want the JES subsystem to load as a plugin that registers its panels, commands, and APIs with the platform, so that job management integrates seamlessly with the workbench.

**Source:** FFW-PLG-001, FFW-ARCH plugin-architecture. [JES, FFW-ARCH]

#### Acceptance Criteria

1.1. THE `ff-jes` crate SHALL implement the `FileForgePlugin` trait, providing `initialize`, `activate`, `deactivate`, and `shutdown` lifecycle methods.

1.2. WHEN `initialize` is called, THE JesPlugin SHALL register all JES commands with the command registry via `PluginContext` under the `jes.*` namespace (job submission, hold, release, cancel, purge, monitor, catalog commands).

1.3. WHEN `activate` is called, THE JesPlugin SHALL register all JES panels with the Panel_Registry (JobMonitorPanel, JobLogViewerPanel).

1.4. WHEN `activate` is called, THE JesPlugin SHALL initialize the Initiator_Pool with the configured number of workers and start the Scheduler.

1.5. WHEN `deactivate` is called, THE JesPlugin SHALL gracefully stop all initiators (allowing active jobs to complete or cancel), persist queue state, and deregister all capabilities.

1.6. WHEN `shutdown` is called, THE JesPlugin SHALL persist all retained job output and catalog state, close all resources.

1.7. THE JesPlugin's `metadata` SHALL declare the plugin name as `"ffw-jes"`, declare capabilities `[Commands, Viewers, Providers]`, and specify dependencies on `ff-vfs`, `ff-workflow`, `ff-dataset-catalog`, and `ff-dataset-allocator`.

1.8. THE JesPlugin SHALL be independently enable/disable-able and SHALL support independent versioning from the workbench core.

1.9. ALL JES panels SHALL implement the `DockablePanel` trait compatible with the workbench layout system.

---

### Requirement 2: Job Submission

**User Story:** As a developer, I want to submit a job definition to the JES queue so that it can be scheduled and executed by an available initiator.

**Source:** FFW-JES-001 (Job Submission). [JES]

#### Acceptance Criteria

2.1. WHEN a user invokes "Submit Job" (command `jes.job.submit`), THE system SHALL parse the job definition (FFJCL), validate it, and create a new job record in the Input Queue.

2.2. THE submitted job SHALL receive a unique Job_ID that is monotonically increasing and never reused within the same workbench session.

2.3. THE system SHALL record the submission timestamp and the submitting user or process identity on the job record.

2.4. THE initial job status SHALL be set to `QUEUED`.

2.5. THE submitted job SHALL appear immediately in the Job Monitor Input Queue panel.

2.6. THE queued job state SHALL survive an application restart — job queue persistence uses a local database or file store.

2.7. IF the job definition fails validation (syntax errors, missing required fields, unresolvable DSN references), THEN THE system SHALL reject the submission with a meaningful validation message and SHALL NOT create a queue entry.

2.8. THE system SHALL support submitting jobs from: the command line (`jes.job.submit`), the FFJCL editor context menu, the Job API, and Lua macro scripts.

---

### Requirement 3: Job Queue and Scheduling

**User Story:** As an operator, I want jobs to be scheduled from the queue to available initiators based on priority and eligibility, so that work executes efficiently.

**Source:** FFW-JES-002, FFW-JES-004 (Queue Visibility, Scheduling). [JES]

#### Acceptance Criteria

3.1. THE Scheduler SHALL support FIFO scheduling (first-in-first-out by submission time) as the default dispatch strategy.

3.2. THE Scheduler SHALL support priority-based scheduling where higher-priority jobs are dispatched before lower-priority jobs regardless of submission order.

3.3. WHEN an initiator becomes available AND a queued job is eligible, THE Scheduler SHALL dispatch the highest-priority eligible job to that initiator.

3.4. THE Scheduler SHALL NOT dispatch jobs with status `HELD` or `CANCELLED`.

3.5. THE Scheduler SHALL NOT dispatch jobs whose preconditions (predecessor job completion, required datasets) are unmet.

3.6. WHEN a job is dispatched, THE system SHALL change its status from `QUEUED` to `ACTIVE`, record the start timestamp, and assign the initiator identifier.

3.7. THE system SHALL prevent dispatching more concurrent jobs than the configured initiator pool capacity.

3.8. THE Job Monitor SHALL display all queued jobs in the Input Queue panel, sortable by: Job Name, Job ID, Owner/User, Submit Time, Priority, Status.

3.9. THE queue display SHALL update automatically when jobs change status — no manual refresh required for state transitions.

3.10. THE user SHALL be able to distinguish between QUEUED, HELD, ACTIVE, COMPLETED, FAILED, and CANCELLED jobs by visual indicators (icons, colours, or labels).

---

### Requirement 4: Initiator Pool

**User Story:** As an operator, I want to configure and manage a pool of initiators (workers) so that I can control execution concurrency and resource usage.

**Source:** FFW-JES-003 (Initiator Pool). [JES]

#### Acceptance Criteria

4.1. THE number of initiators in the pool SHALL be configurable via the workbench configuration system (`[plugins.ffw-jes].initiator_count`, default: 3).

4.2. EACH initiator SHALL have a unique identifier visible in the Job Monitor.

4.3. THE user SHALL be able to view each initiator's current status: IDLE, STARTING, ACTIVE, STOPPING, STOPPED, FAILED.

4.4. THE system SHALL support starting an individual initiator (command `jes.initiator.start`).

4.5. THE system SHALL support stopping an individual initiator (command `jes.initiator.stop`) — an active job on that initiator completes before the initiator stops.

4.6. THE system SHALL support pausing an initiator from accepting new work (command `jes.initiator.drain`) without terminating the currently active job.

4.7. THE Initiator_Pool SHALL execute jobs asynchronously on the Tokio runtime, ensuring the UI remains responsive during job execution.

4.8. WHEN an initiator encounters an unrecoverable error, THE system SHALL mark that initiator as FAILED, log the error, and continue operating with remaining healthy initiators.

---

### Requirement 5: Active Job Monitoring

**User Story:** As an operator, I want to see real-time status of executing jobs including elapsed time, current step, and resource usage, so that I can monitor workload health.

**Source:** FFW-JES-005 (Active Job Monitoring). [JES]

#### Acceptance Criteria

5.1. THE Job Monitor SHALL display the following for active jobs: Job Name, Job ID, Owner/User, Assigned Initiator ID, Start Time, Elapsed Time, Current Step, Current Step Status.

5.2. WHERE the operating system provides process-level metrics, THE Job Monitor SHALL display Process ID, CPU usage, and Memory usage for active jobs.

5.3. THE active job display SHALL update automatically while jobs are running (configurable refresh interval, default: 1 second).

5.4. THE user SHALL be able to open the live job log for an active job (streaming output as it is produced).

5.5. THE user SHALL be able to request cancellation of an active job from the Job Monitor (command `jes.job.cancel`).

---

### Requirement 6: Job Completion, Failure, and Cancellation

**User Story:** As an operator, I want completed, failed, and cancelled jobs to retain their output and be inspectable from the Job Monitor.

**Source:** FFW-JES-006, FFW-JES-007, FFW-JES-008 (Completion, Failure, Cancellation). [JES]

#### Acceptance Criteria

6.1. WHEN a job completes successfully, THE system SHALL set its status to `COMPLETED`, record the end timestamp, calculate elapsed runtime, and store the final return code.

6.2. WHEN a job terminates abnormally, THE system SHALL set its status to `FAILED`, record the failure reason, the failing step (where applicable), the abnormal termination code, and retain any diagnostic information (stack trace, error details).

6.3. WHEN a user cancels a queued job, THE system SHALL set its status to `CANCELLED` without executing it, recording who requested the cancellation and the cancellation timestamp.

6.4. WHEN a user cancels an active job, THE system SHALL send a termination signal to the executing process, wait for graceful shutdown (configurable timeout), and force-terminate if the timeout expires.

6.5. AFTER any terminal status (COMPLETED, FAILED, CANCELLED), THE system SHALL release the assigned initiator for the next eligible job.

6.6. AFTER any terminal status, THE system SHALL retain job logs, SYSOUT output, and output datasets according to the configured Retention_Policy.

6.7. THE completed/failed/cancelled job SHALL appear in the appropriate Output panel in the Job Monitor.

6.8. Logs generated before cancellation or failure SHALL be preserved and viewable.

---

### Requirement 7: Job Logs and SYSOUT

**User Story:** As a developer, I want to view complete execution logs for any job, so that I can diagnose issues and review output.

**Source:** FFW-JES-009, FFW-SDSF-003 (Job Logs, View SYSOUT). [JES]

#### Acceptance Criteria

7.1. WHEN a user requests job output (command `jes.job.view_log`), THE system SHALL display the complete execution log in the JobLogViewerPanel.

7.2. THE Job_Log SHALL contain: JES-style scheduling messages, allocation messages (dataset resolution), step logs, application output (SYSOUT), error output, and return codes per step.

7.3. THE JobLogViewerPanel SHALL support multiple output sections displayed as tabs or collapsible sections: JES Log, Step Log, SYSOUT, Error Output, Allocation Messages.

7.4. THE JobLogViewerPanel SHALL support search within log content, copy to clipboard, and export to file (via VFS).

7.5. THE JobLogViewerPanel SHALL support viewing logs for active jobs (streaming live output), completed jobs, failed jobs, and cancelled jobs.

7.6. THE system SHALL handle large job logs without freezing the UI — logs are loaded incrementally or virtualized for rendering.

7.7. THE Job_Log SHALL be stored in a stable format that survives application restarts and is independent from the physical output datasets.

---

### Requirement 8: Retained Output and Purge

**User Story:** As an operator, I want job output retained according to configurable rules and purgeable when no longer needed.

**Source:** FFW-JES-010 (Retained Output). [JES]

#### Acceptance Criteria

8.1. THE Retention_Policy SHALL be configurable via `[plugins.ffw-jes].retention_days` (default: 7 days) and `[plugins.ffw-jes].retention_max_jobs` (default: 1000).

8.2. THE system SHALL support manual purge of individual jobs (command `jes.job.purge`) or batch purge by filter criteria.

8.3. THE system SHALL support automatic purge — background task removes jobs exceeding the retention policy on a configurable schedule.

8.4. WHEN purging a job, THE system SHALL remove retained logs and SYSOUT output according to policy.

8.5. WHEN purging a job, THE system SHALL NOT remove catalogued datasets unless the user explicitly requests dataset deletion alongside the purge.

8.6. THE system SHALL display a confirmation warning before destructive purge actions that would remove output permanently.

---

### Requirement 9: Job Monitor Panel (SDSF-Style)

**User Story:** As an operator, I want an SDSF-style Job Monitor with filterable panels for each queue state, so that I can efficiently manage batch workloads.

**Source:** FFW-SDSF-001, FFW-SDSF-002, FFW-SDSF-004 (Job Monitor, Filtering, Refresh). [JES]

#### Acceptance Criteria

9.1. THE JobMonitorPanel SHALL implement `DockablePanel` with `default_dock_zone` of `Bottom` and SHALL provide tabbed sub-panels for: Input Queue, Active Jobs, Held Jobs, Output/Completed Jobs, Failed Jobs, Cancelled Jobs.

9.2. EACH sub-panel SHALL display the job count in its tab header.

9.3. THE user SHALL be able to open job details and job logs from any panel via double-click or context menu.

9.4. THE user SHALL be able to filter jobs by: Owner/User, Job Name, Job ID, Status, Submit Date range, Start Date range, End Date range, Return Code, Queue.

9.5. Filters SHALL be clearable and SHALL NOT alter stored job state.

9.6. Filter results SHALL update dynamically when job state changes.

9.7. THE Job Monitor SHALL refresh automatically at a configurable interval (`[plugins.ffw-jes].monitor_refresh_ms`, default: 2000ms), preferring push-style event updates where feasible.

9.8. Manual refresh SHALL remain available (command `jes.monitor.refresh`, shortcut: F5).

9.9. Automatic refresh SHALL NOT reset user-selected filters, collapse expanded nodes, or interrupt active log viewing.

9.10. THE Job Monitor SHALL support context menu actions per job: View Log, Hold, Release, Cancel, Purge, Properties.

---

### Requirement 10: Job Hold and Release

**User Story:** As an operator, I want to hold a queued job to prevent execution and release it when ready.

**Source:** FFW-JES-004 (Scheduling — held jobs). [JES]

#### Acceptance Criteria

10.1. WHEN the user invokes "Hold Job" (command `jes.job.hold`) on a queued job, THE system SHALL change its status to `HELD` and prevent the Scheduler from dispatching it.

10.2. WHEN the user invokes "Release Job" (command `jes.job.release`) on a held job, THE system SHALL change its status back to `QUEUED`, making it eligible for scheduling.

10.3. THE Held Jobs panel in the Job Monitor SHALL display all jobs in HELD status.

10.4. A job that is already ACTIVE SHALL NOT be held — the hold command SHALL return an error indicating the job is already executing.

---

### Requirement 11: Dataset Catalog Integration

**User Story:** As a developer, I want jobs to resolve DSN references through the workbench dataset catalog so that job definitions can reference logical dataset names.

**Source:** FFW-CAT-001 through FFW-CAT-005 (Dataset Catalog). [JES]

#### Acceptance Criteria

11.1. WHEN a job definition references `DSN=qualifier.name`, THE system SHALL resolve the DSN through the `ff-dataset-allocator` crate's allocation API (which delegates to `ff-dataset-catalog` for catalog lookup).

11.2. IF a referenced DSN is not found in the catalog AND the job definition does not specify `DISP=NEW`, THEN THE system SHALL fail allocation with an error message written to the job log.

11.3. WHEN a job allocates a new dataset (`DISP=NEW`), THE system SHALL delegate to the `ff-dataset-allocator` crate's allocation API, which creates the catalog entry and physical file via `ff-dataset-catalog`.

11.4. THE system SHALL write dataset resolution messages to the job log for each DD statement (resolved path, catalog entry metadata).

11.5. THE system SHALL support Generation Data Group references (`DSN=MY.FILE.GDG(+1)`, `(0)`, `(-1)`) by delegating to the `ff-dataset-allocator` GDG relative generation resolution (which queries `ff-dataset-catalog` for generation state).

11.6. THE JES subsystem SHALL leverage the existing file-tree-panel "Catalogs" node (provided by `ff-dataset-catalog`'s VFS provider) for dataset browsing — it SHALL NOT create a separate DatasetExplorerPanel. The JES Job Monitor's dataset references link to the file-tree-panel's catalog view.

11.7. Dataset resolution SHALL work consistently on Windows, Linux, and macOS using the dataset-catalog's platform-independent path mapping.

---

### Requirement 12: Job and Dataset APIs

**User Story:** As a plugin developer, I want programmatic APIs for job management and dataset operations so that other workbench components can automate batch workflows.

**Source:** FFW-PLG-002, FFW-PLG-003 (Dataset API, Job API). [JES]

#### Acceptance Criteria

12.1. THE JesPlugin SHALL expose a Job API accessible to other workbench plugins and Lua macros, supporting: submit, hold, release, cancel, query status, retrieve logs, retrieve output, and subscribe to status change events.

12.2. THE JesPlugin SHALL expose a Dataset API accessible to other workbench plugins, supporting: allocate, read, write, delete, resolve DSN, query metadata, and open in editor.

12.3. ALL Job API operations SHALL be invocable from the Lua scripting bridge (e.g., `workbench.execute("jes.job.submit", {jcl = "..."})`).

12.4. THE Job API SHALL support event subscription — callers can register callbacks for job state transitions (QUEUED→ACTIVE, ACTIVE→COMPLETED, etc.).

12.5. THE Dataset API SHALL delegate to the `ff-dataset-allocator` crate for allocation operations (DISP=NEW/OLD/SHR/MOD) and `ff-dataset-catalog` for catalog metadata queries.

---

### Requirement 13: Command Integration

**User Story:** As a workbench user, I want all JES operations available as registered commands with keyboard shortcuts.

**Source:** FFW-ARCH command-framework. [FFW-ARCH]

#### Acceptance Criteria

13.1. ALL user-facing JES operations SHALL be registered as commands under the `jes.*` namespace: `jes.job.submit`, `jes.job.hold`, `jes.job.release`, `jes.job.cancel`, `jes.job.purge`, `jes.job.view_log`, `jes.monitor.refresh`, `jes.initiator.start`, `jes.initiator.stop`, `jes.initiator.drain`, `jes.catalog.browse`.

13.2. EACH JES command SHALL have associated metadata: display name, description, category (`jes.job`, `jes.initiator`, `jes.catalog`), and default keyboard shortcut where applicable.

13.3. EACH JES command SHALL have an enabled predicate (e.g., `jes.job.cancel` enabled only when a job is selected and in QUEUED or ACTIVE status).

13.4. ALL JES commands SHALL be invocable from the command palette, menus, keyboard shortcuts, context menus, and the Lua scripting bridge.

---

### Requirement 14: Provider Abstraction (Future Extensibility)

**User Story:** As an architect, I want the JES subsystem to define a provider abstraction so that future remote execution environments (real z/OS JES, Linux batch, Windows Task Scheduler) can plug in without redesigning the monitor.

**Source:** FFW-JES-FUT-001, FFW-JES-FUT-002 (Provider Abstraction, Unified Monitor). [JES]

#### Acceptance Criteria

14.1. THE system SHALL define a `JobProvider` trait that abstracts job queue operations (submit, hold, release, cancel, query, retrieve logs) behind a provider-agnostic interface.

14.2. THE initial release SHALL ship with a single provider: `DesktopJesProvider` — the local queue and initiator pool implementation.

14.3. THE Job Monitor SHALL be designed to display jobs from multiple providers simultaneously when additional providers are registered in future releases.

14.4. EACH job displayed in the monitor SHALL indicate its source provider, and filtering by provider SHALL be supported.

14.5. Job actions in the monitor SHALL be limited to actions supported by the relevant provider — unsupported actions SHALL be greyed out.

14.6. Provider connection errors SHALL be visible in the Job Monitor without crashing the application or affecting other providers.

14.7. THE `JobProvider` trait SHALL support: list_jobs, submit_job, hold_job, release_job, cancel_job, get_job_log, subscribe_to_events.

---

### Requirement 15: Async Execution and Concurrency

**User Story:** As a workbench user, I want job execution and monitoring to be fully async so that the UI remains responsive during batch processing.

**Source:** FFW-ARCH async I/O principle, Tokio runtime. [FFW-ARCH]

#### Acceptance Criteria

15.1. ALL job execution SHALL be async — initiators run jobs on Tokio tasks or `spawn_blocking` threads without blocking the egui render loop.

15.2. THE Scheduler dispatch loop SHALL run as an async background task, polling for eligible jobs and available initiators.

15.3. Job log streaming (live log viewing) SHALL use async channels to deliver output lines to the UI incrementally.

15.4. Queue state persistence SHALL be async and SHALL NOT block job submission or status transitions.

15.5. THE Job Monitor refresh SHALL be event-driven where possible (job status change events push to the UI) with polling as fallback.

---

## Cross-Cutting Concerns

### Error Handling

All JES errors SHALL use `thiserror` with variants: SubmissionFailed, ValidationError, SchedulerError, InitiatorFailed, CatalogResolutionFailed, PurgeError, ProviderUnavailable. Application-level code uses `anyhow` for context chains.

### Logging

All significant JES operations (job submissions, state transitions, scheduler decisions, initiator lifecycle, errors) SHALL emit structured log records via `ff-logging` at appropriate levels.

### Configuration

JES settings SHALL be stored under `[plugins.ffw-jes]` in the workbench configuration system: `initiator_count`, `retention_days`, `retention_max_jobs`, `monitor_refresh_ms`, `scheduler_poll_ms`, `job_cancel_timeout_ms`.

### VFS Integration

Job log files and SYSOUT output SHALL be accessible via VFS Resource_URIs (e.g., `vfs://local/path/to/job-output/JOB00001/SYSOUT`), enabling them to appear in the file tree and be opened in the editor.

---

## Source Reference Key

| Tag | Source |
|-----|--------|
| JES | FFW-JES EARS Requirements document |
| FFW-ARCH | FileForgeWorkbench architecture specs (command-framework, plugin-architecture, layout-and-docking, workflow-engine, VFS) |
