# Requirements Document

## Introduction

> **Naming Note:** This sub-project folder was named `FFW-JES` for historical reasons and was renamed to `jes-emulator` in Phase BR to match the kebab-case convention. The crate name `ff-jes` is unchanged.

This feature specifies **FFW-JES** (FileForge Workbench Job Entry Subsystem) — a cross-platform emulator of IBM JES2/JES3 batch processing concepts (JES2 and JES3 are IBM z/OS job entry subsystems; this crate emulates their concepts on the desktop — it has no dependency on actual IBM software) delivered as a workbench plugin (`ff-jes` crate). FFW-JES emulates mainframe batch processing on Windows, Linux, and macOS: job submission, queue management, initiator-based execution, SDSF-style monitoring, dataset allocation via the catalog, and retained job output.

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

1. THE `ff-jes` crate SHALL implement the `FileForgePlugin` trait, providing `initialize`, `activate`, `deactivate`, and `shutdown` lifecycle methods.

2. WHEN `initialize` is called, THE JesPlugin SHALL register all JES commands with the command registry via `PluginContext` under the `jes.*` namespace (job submission, hold, release, cancel, purge, monitor, catalog commands).

3. WHEN `activate` is called, THE JesPlugin SHALL register all JES panels with the Panel_Registry (JobMonitorPanel, JobLogViewerPanel).

4. WHEN `activate` is called, THE JesPlugin SHALL initialize the Initiator_Pool with the configured number of workers and start the Scheduler.

5. WHEN `deactivate` is called, THE JesPlugin SHALL gracefully stop all initiators (allowing active jobs to complete or cancel), persist queue state, and deregister all capabilities.

6. WHEN `shutdown` is called, THE JesPlugin SHALL persist all retained job output and catalog state, close all resources.

7. THE JesPlugin's `metadata` SHALL declare the plugin name as `"ffw-jes"`, declare capabilities `[Commands, Viewers, Providers]`, and specify dependencies on `ff-vfs`, `ff-workflow`, `ff-dataset-catalog`, and `ff-dataset-allocator`.

8. THE JesPlugin SHALL be independently enable/disable-able and SHALL support independent versioning from the workbench core.

9. ALL JES panels SHALL implement the `DockablePanel` trait compatible with the workbench layout system.

---

### Requirement 2: Job Submission

**User Story:** As a developer, I want to submit a job definition to the JES queue so that it can be scheduled and executed by an available initiator.

**Source:** FFW-JES-001 (Job Submission). [JES]

#### Acceptance Criteria

1. WHEN a user invokes "Submit Job" (command `jes.job.submit`), THE system SHALL parse the job definition (FFJCL), validate it, and create a new job record in the Input Queue.

2. THE submitted job SHALL receive a unique Job_ID that is monotonically increasing and never reused within the same workbench session.

3. THE system SHALL record the submission timestamp and the submitting user or process identity on the job record.

4. THE initial job status SHALL be set to `QUEUED`.

5. THE submitted job SHALL appear immediately in the Job Monitor Input Queue panel.

6. THE queued job state SHALL survive an application restart — job queue persistence uses a local database or file store.

7. IF the job definition fails validation (syntax errors, missing required fields, unresolvable DSN references), THEN THE system SHALL reject the submission with a meaningful validation message and SHALL NOT create a queue entry.

8. THE system SHALL support submitting jobs from: the command line (`jes.job.submit`), the FFJCL editor context menu, the Job API, and Lua macro scripts.

---

### Requirement 3: Job Queue and Scheduling

**User Story:** As an operator, I want jobs to be scheduled from the queue to available initiators based on priority and eligibility, so that work executes efficiently.

**Source:** FFW-JES-002, FFW-JES-004 (Queue Visibility, Scheduling). [JES]

#### Acceptance Criteria

1. THE Scheduler SHALL support FIFO scheduling (first-in-first-out by submission time) as the default dispatch strategy.

2. THE Scheduler SHALL support priority-based scheduling where higher-priority jobs are dispatched before lower-priority jobs regardless of submission order.

3. WHEN an initiator becomes available AND a queued job is eligible, THE Scheduler SHALL dispatch the highest-priority eligible job to that initiator.

4. THE Scheduler SHALL NOT dispatch jobs with status `HELD` or `CANCELLED`.

5. THE Scheduler SHALL NOT dispatch jobs whose preconditions (predecessor job completion, required datasets) are unmet.

6. WHEN a job is dispatched, THE system SHALL change its status from `QUEUED` to `ACTIVE`, record the start timestamp, and assign the initiator identifier.

7. THE system SHALL prevent dispatching more concurrent jobs than the configured initiator pool capacity.

8. THE Job Monitor SHALL display all queued jobs in the Input Queue panel, sortable by: Job Name, Job ID, Owner/User, Submit Time, Priority, Status.

9. THE queue display SHALL update automatically when jobs change status — no manual refresh required for state transitions.

10. THE user SHALL be able to distinguish between QUEUED, HELD, ACTIVE, COMPLETED, FAILED, and CANCELLED jobs by visual indicators (icons, colours, or labels).

---

### Requirement 4: Initiator Pool

**User Story:** As an operator, I want to configure and manage a pool of initiators (workers) so that I can control execution concurrency and resource usage.

**Source:** FFW-JES-003 (Initiator Pool). [JES]

#### Acceptance Criteria

1. THE number of initiators in the pool SHALL be configurable via the workbench configuration system (`[plugins.ffw-jes].initiator_count`, default: 3).

4.2. EACH initiator SHALL have a unique identifier visible in the Job Monitor.

3. THE user SHALL be able to view each initiator's current status: IDLE, STARTING, ACTIVE, STOPPING, STOPPED, FAILED.

4. THE system SHALL support starting an individual initiator (command `jes.initiator.start`).

5. THE system SHALL support stopping an individual initiator (command `jes.initiator.stop`) — an active job on that initiator completes before the initiator stops.

6. THE system SHALL support pausing an initiator from accepting new work (command `jes.initiator.drain`) without terminating the currently active job.

7. THE Initiator_Pool SHALL execute jobs asynchronously on the Tokio runtime, ensuring the UI remains responsive during job execution.

8. WHEN an initiator encounters an unrecoverable error, THE system SHALL mark that initiator as FAILED, log the error, and continue operating with remaining healthy initiators.

---

### Requirement 5: Active Job Monitoring

**User Story:** As an operator, I want to see real-time status of executing jobs including elapsed time, current step, and resource usage, so that I can monitor workload health.

**Source:** FFW-JES-005 (Active Job Monitoring). [JES]

#### Acceptance Criteria

1. THE Job Monitor SHALL display the following for active jobs: Job Name, Job ID, Owner/User, Assigned Initiator ID, Start Time, Elapsed Time, Current Step, Current Step Status.

5.2. WHERE the operating system provides process-level metrics, THE Job Monitor SHALL display Process ID, CPU usage, and Memory usage for active jobs.

3. THE active job display SHALL update automatically while jobs are running (configurable refresh interval, default: 1 second).

4. THE user SHALL be able to open the live job log for an active job (streaming output as it is produced).

5. THE user SHALL be able to request cancellation of an active job from the Job Monitor (command `jes.job.cancel`).

---

### Requirement 6: Job Completion, Failure, and Cancellation

**User Story:** As an operator, I want completed, failed, and cancelled jobs to retain their output and be inspectable from the Job Monitor.

**Source:** FFW-JES-006, FFW-JES-007, FFW-JES-008 (Completion, Failure, Cancellation). [JES]

#### Acceptance Criteria

1. WHEN a job completes successfully, THE system SHALL set its status to `COMPLETED`, record the end timestamp, calculate elapsed runtime, and store the final return code.

2. WHEN a job terminates abnormally, THE system SHALL set its status to `FAILED`, record the failure reason, the failing step (where applicable), the abnormal termination code, and retain any diagnostic information (stack trace, error details).

3. WHEN a user cancels a queued job, THE system SHALL set its status to `CANCELLED` without executing it, recording who requested the cancellation and the cancellation timestamp.

4. WHEN a user cancels an active job, THE system SHALL send a termination signal to the executing process, wait for graceful shutdown (configurable timeout), and force-terminate if the timeout expires.

5. AFTER any terminal status (COMPLETED, FAILED, CANCELLED), THE system SHALL release the assigned initiator for the next eligible job.

6. AFTER any terminal status, THE system SHALL retain job logs, SYSOUT output, and output datasets according to the configured Retention_Policy.

7. THE completed/failed/cancelled job SHALL appear in the appropriate Output panel in the Job Monitor.

6.8. Logs generated before cancellation or failure SHALL be preserved and viewable.

---

### Requirement 7: Job Logs and SYSOUT

**User Story:** As a developer, I want to view complete execution logs for any job, so that I can diagnose issues and review output.

**Source:** FFW-JES-009, FFW-SDSF-003 (Job Logs, View SYSOUT). [JES]

#### Acceptance Criteria

1. WHEN a user requests job output (command `jes.job.view_log`), THE system SHALL display the complete execution log in the JobLogViewerPanel.

2. THE Job_Log SHALL contain: JES-style scheduling messages, allocation messages (dataset resolution), step logs, application output (SYSOUT), error output, and return codes per step.

3. THE JobLogViewerPanel SHALL support multiple output sections displayed as tabs or collapsible sections: JES Log, Step Log, SYSOUT, Error Output, Allocation Messages.

4. THE JobLogViewerPanel SHALL support search within log content, copy to clipboard, and export to file (via VFS).

5. THE JobLogViewerPanel SHALL support viewing logs for active jobs (streaming live output), completed jobs, failed jobs, and cancelled jobs.

6. THE system SHALL handle large job logs without freezing the UI — logs are loaded incrementally or virtualized for rendering.

7. THE Job_Log SHALL be stored in a stable format that survives application restarts and is independent from the physical output datasets.

---

### Requirement 8: Retained Output and Purge

**User Story:** As an operator, I want job output retained according to configurable rules and purgeable when no longer needed.

**Source:** FFW-JES-010 (Retained Output). [JES]

#### Acceptance Criteria

1. THE Retention_Policy SHALL be configurable via `[plugins.ffw-jes].retention_days` (default: 7 days) and `[plugins.ffw-jes].retention_max_jobs` (default: 1000).

2. THE system SHALL support manual purge of individual jobs (command `jes.job.purge`) or batch purge by filter criteria.

3. THE system SHALL support automatic purge — background task removes jobs exceeding the retention policy on a configurable schedule.

4. WHEN purging a job, THE system SHALL remove retained logs and SYSOUT output according to policy.

5. WHEN purging a job, THE system SHALL NOT remove catalogued datasets unless the user explicitly requests dataset deletion alongside the purge.

6. THE system SHALL display a confirmation warning before destructive purge actions that would remove output permanently.

---

### Requirement 9: Job Monitor Panel (SDSF-Style)

**User Story:** As an operator, I want an SDSF-style Job Monitor with filterable panels for each queue state, so that I can efficiently manage batch workloads.

**Source:** FFW-SDSF-001, FFW-SDSF-002, FFW-SDSF-004 (Job Monitor, Filtering, Refresh). [JES]

#### Acceptance Criteria

1. THE JobMonitorPanel SHALL implement `DockablePanel` with `default_dock_zone` of `Bottom` and SHALL provide tabbed sub-panels for: Input Queue, Active Jobs, Held Jobs, Output/Completed Jobs, Failed Jobs, Cancelled Jobs.

9.2. EACH sub-panel SHALL display the job count in its tab header.

3. THE user SHALL be able to open job details and job logs from any panel via double-click or context menu.

4. THE user SHALL be able to filter jobs by: Owner/User, Job Name, Job ID, Status, Submit Date range, Start Date range, End Date range, Return Code, Queue.

9.5. Filters SHALL be clearable and SHALL NOT alter stored job state.

9.6. Filter results SHALL update dynamically when job state changes.

7. THE Job Monitor SHALL refresh automatically at a configurable interval (`[plugins.ffw-jes].monitor_refresh_ms`, default: 2000ms), preferring push-style event updates where feasible.

9.8. Manual refresh SHALL remain available (command `jes.monitor.refresh`, shortcut: F5).

9.9. Automatic refresh SHALL NOT reset user-selected filters, collapse expanded nodes, or interrupt active log viewing.

10. THE Job Monitor SHALL support context menu actions per job: View Log, Hold, Release, Cancel, Purge, Properties.

---

### Requirement 10: Job Hold and Release

**User Story:** As an operator, I want to hold a queued job to prevent execution and release it when ready.

**Source:** FFW-JES-004 (Scheduling — held jobs). [JES]

#### Acceptance Criteria

1. WHEN the user invokes "Hold Job" (command `jes.job.hold`) on a queued job, THE system SHALL change its status to `HELD` and prevent the Scheduler from dispatching it.

2. WHEN the user invokes "Release Job" (command `jes.job.release`) on a held job, THE system SHALL change its status back to `QUEUED`, making it eligible for scheduling.

3. THE Held Jobs panel in the Job Monitor SHALL display all jobs in HELD status.

10.4. A job that is already ACTIVE SHALL NOT be held — the hold command SHALL return an error indicating the job is already executing.

---

### Requirement 11: Dataset Catalog Integration

**User Story:** As a developer, I want jobs to resolve DSN references through the workbench dataset catalog so that job definitions can reference logical dataset names.

**Source:** FFW-CAT-001 through FFW-CAT-005 (Dataset Catalog). [JES]

#### Acceptance Criteria

1. WHEN a job definition references `DSN=qualifier.name`, THE system SHALL resolve the DSN through the `ff-dataset-allocator` crate's allocation API (which delegates to `ff-dataset-catalog` for catalog lookup).

2. IF a referenced DSN is not found in the catalog AND the job definition does not specify `DISP=NEW`, THEN THE system SHALL fail allocation with an error message written to the job log.

3. WHEN a job allocates a new dataset (`DISP=NEW`), THE system SHALL delegate to the `ff-dataset-allocator` crate's allocation API, which creates the catalog entry and physical file via `ff-dataset-catalog`.

4. THE system SHALL write dataset resolution messages to the job log for each DD statement (resolved path, catalog entry metadata).

5. THE system SHALL support Generation Data Group references (`DSN=MY.FILE.GDG(+1)`, `(0)`, `(-1)`) by delegating to the `ff-dataset-allocator` GDG relative generation resolution (which queries `ff-dataset-catalog` for generation state).

6. THE JES subsystem SHALL leverage the existing file-tree-panel "Catalogs" node (provided by `ff-dataset-catalog`'s VFS provider) for dataset browsing — it SHALL NOT create a separate DatasetExplorerPanel. The JES Job Monitor's dataset references link to the file-tree-panel's catalog view.

11.7. Dataset resolution SHALL work consistently on Windows, Linux, and macOS using the dataset-catalog's platform-independent path mapping.

---

### Requirement 12: Job and Dataset APIs

**User Story:** As a plugin developer, I want programmatic APIs for job management and dataset operations so that other workbench components can automate batch workflows.

**Source:** FFW-PLG-002, FFW-PLG-003 (Dataset API, Job API). [JES]

#### Acceptance Criteria

1. THE JesPlugin SHALL expose a Job API accessible to other workbench plugins and Lua macros, supporting: submit, hold, release, cancel, query status, retrieve logs, retrieve output, and subscribe to status change events.

2. THE JesPlugin SHALL expose a Dataset API accessible to other workbench plugins, supporting: allocate, read, write, delete, resolve DSN, query metadata, and open in editor.

3. ALL Job API operations SHALL be invocable from the Lua scripting bridge (e.g., `workbench.execute("jes.job.submit", {jcl = "..."})`).

4. THE Job API SHALL support event subscription — callers can register callbacks for job state transitions (QUEUED→ACTIVE, ACTIVE→COMPLETED, etc.).

5. THE Dataset API SHALL delegate to the `ff-dataset-allocator` crate for allocation operations (DISP=NEW/OLD/SHR/MOD) and `ff-dataset-catalog` for catalog metadata queries.

---

### Requirement 13: Command Integration

**User Story:** As a workbench user, I want all JES operations available as registered commands with keyboard shortcuts.

**Source:** FFW-ARCH command-framework. [FFW-ARCH]

#### Acceptance Criteria

1. ALL user-facing JES operations SHALL be registered as commands under the `jes.*` namespace: `jes.job.submit`, `jes.job.hold`, `jes.job.release`, `jes.job.cancel`, `jes.job.purge`, `jes.job.view_log`, `jes.monitor.refresh`, `jes.initiator.start`, `jes.initiator.stop`, `jes.initiator.drain`, `jes.catalog.browse`.

13.2. EACH JES command SHALL have associated metadata: display name, description, category (`jes.job`, `jes.initiator`, `jes.catalog`), and default keyboard shortcut where applicable.

13.3. EACH JES command SHALL have an enabled predicate (e.g., `jes.job.cancel` enabled only when a job is selected and in QUEUED or ACTIVE status).

4. ALL JES commands SHALL be invocable from the command palette, menus, keyboard shortcuts, context menus, and the Lua scripting bridge.

---

### Requirement 14: Provider Abstraction (Future Extensibility)

**User Story:** As an architect, I want the JES subsystem to define a provider abstraction so that future remote execution environments (real z/OS JES, Linux batch, Windows Task Scheduler) can plug in without redesigning the monitor.

**Source:** FFW-JES-FUT-001, FFW-JES-FUT-002 (Provider Abstraction, Unified Monitor). [JES]

#### Acceptance Criteria

1. THE system SHALL define a `JobProvider` trait that abstracts job queue operations (submit, hold, release, cancel, query, retrieve logs) behind a provider-agnostic interface.

2. THE initial release SHALL ship with a single provider: `DesktopJesProvider` — the local queue and initiator pool implementation.

3. THE Job Monitor SHALL be designed to display jobs from multiple providers simultaneously when additional providers are registered in future releases.

14.4. EACH job displayed in the monitor SHALL indicate its source provider, and filtering by provider SHALL be supported.

14.5. Job actions in the monitor SHALL be limited to actions supported by the relevant provider — unsupported actions SHALL be greyed out.

14.6. Provider connection errors SHALL be visible in the Job Monitor without crashing the application or affecting other providers.

7. THE `JobProvider` trait SHALL support: list_jobs, submit_job, hold_job, release_job, cancel_job, get_job_log, subscribe_to_events.

---

### Requirement 15: Async Execution and Concurrency

**User Story:** As a workbench user, I want job execution and monitoring to be fully async so that the UI remains responsive during batch processing.

**Source:** FFW-ARCH async I/O principle, Tokio runtime. [FFW-ARCH]

#### Acceptance Criteria

1. ALL job execution SHALL be async — initiators run jobs on Tokio tasks or `spawn_blocking` threads without blocking the egui render loop.

2. THE Scheduler dispatch loop SHALL run as an async background task, polling for eligible jobs and available initiators.

15.3. Job log streaming (live log viewing) SHALL use async channels to deliver output lines to the UI incrementally.

15.4. Queue state persistence SHALL be async and SHALL NOT block job submission or status transitions.

5. THE Job Monitor refresh SHALL be event-driven where possible (job status change events push to the UI) with polling as fallback.

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

---

### Requirement 16: SDSF Panel Framework Core (P1)

**User Story:** As a workbench user, I want the Job Monitor to present an SDSF-style panel
framework with action bar, title line, SCROLL field, filter information lines, NP column,
fixed first column, main panel command list, and PREFIX/OWNER/DEST filter commands, so that
the interface matches the mainframe SDSF experience.

**Source:** SDSF-1.1 through SDSF-1.8, SDSF-2.2 through SDSF-2.6, SDSF-4.1 through SDSF-4.6,
SDSF-FILTER-1 through SDSF-FILTER-3, SDSF-1.3 (PAR), SDSF-1.4 (PAR), SDSF-2.1 (PAR),
SDSF-JQ-6 (PAR), SDSF-JQ-7 (PAR), SDSF-FILTER-5 (PAR). [JES, FFW-ARCH]

#### Acceptance Criteria

1. THE JobMonitorPanel SHALL display an action bar at the top with pull-down menus
   (File, View, Help at minimum) consistent with SDSF panel layout.

2. THE JobMonitorPanel SHALL display a title line showing the panel name and the
   line range of currently visible rows (e.g., "INPUT QUEUE -- Row 1 to 25 of 47").

3. THE JobMonitorPanel SHALL display a SCROLL ===> field adjacent to the command
   input field, retaining the last-used scroll amount across panel interactions.

4. THE JobMonitorPanel SHALL display filter information lines below the title line
   showing active filter values in the form PREFIX=value, DEST=value, OWNER=value;
   lines are omitted when the corresponding filter is not set.

5. THE JobMonitorPanel SHALL provide an NP (non-print) column as the leftmost
   fixed column for entering action characters; the NP column SHALL NOT scroll
   horizontally with the data columns.

6. THE first data column (JOBNAME) SHALL remain fixed and visible during horizontal
   scrolling of the remaining columns.

7. WHEN the user types an action character in the NP column and presses Enter,
   THE system SHALL execute the corresponding action on that job row.

8. THE system SHALL support the following action characters in the NP column:
   S (select/view), ? (display valid actions), C (cancel), H (hold), A (release),
   P (purge), D (delete output), E (edit JCL), J (view JCL), W (who has job).

9. THE system SHALL support = as a repeat action character -- entering = in the NP
   column repeats the previous action character on that row.

10. THE system SHALL support // block action syntax -- entering // in the NP column
    of the first and last rows of a block applies the action to all rows in the block.

11. THE system SHALL support command-line action syntax -- entering "2 C" in the
    command field cancels the job in row 2 without using the NP column.

12. WHEN the user issues SET ROWNUM ON, THE system SHALL display row numbers in the
    NP area instead of the action character input field.

13. THE JobMonitorPanel SHALL provide a main panel (accessible via the MENU command)
    listing all available SDSF panel commands with name, description, and group.

14. THE main panel SHALL organise commands into groups: Jobs, Output, JES, Log,
    Memory, and Other; groups SHALL be expandable and collapsible.

15. WHEN the user enters S in the NP column of a main panel row, THE system SHALL
    navigate to the selected panel command.

16. WHEN the user issues SET MAIN GROUP, THE system SHALL display the main panel
    in grouped format with expandable/collapsible command groups.

17. THE MENU command SHALL return the user to the main panel from any sub-panel.

18. THE system SHALL support the PREFIX filter command -- PREFIX value filters the
    job list to show only jobs whose names begin with the specified prefix;
    PREFIX * or PREFIX (empty) clears the filter.

19. THE system SHALL support the OWNER filter command -- OWNER value filters the
    job list to show only jobs owned by the specified user; OWNER * clears the filter.

20. THE system SHALL support the DEST filter command -- DEST value filters the job
    list to show only jobs with the specified output destination; DEST * clears the filter.

21. THE JobMonitorPanel title line SHALL include a message area displaying the most
    recent informational or error message from the last command execution.

22. THE JobMonitorPanel SHALL provide a COMMAND INPUT ===> field for entering SDSF
    commands (PREFIX, OWNER, DEST, SORT, FIND, LOCATE, SET, MENU, WHO, QUERY AUTH).

23. THE NP column SHALL support the full set of SDSF action characters: S, ?, C, H,
    A, P, D, E, J, W; unsupported actions for a given job state SHALL be rejected
    with a message in the title line message area.

24. THE job table SHALL define the following columns: JOBNAME, JOBID, OWNER, STATUS,
    CLASS, PRTY (priority), QUEUE, START, END, RC (return code), STEPNAME, PROCSTEP;
    columns SHALL be individually hideable and reorderable.

25. THE PREFIX, OWNER, and DEST filter fields SHALL be displayable as dedicated
    filter input rows above the job table, pre-populated with the current filter
    values, and editable in-place.

26. THE system SHALL support the SORT command -- SORT colname [A|D] sorts the job
    table by the specified column in ascending (default) or descending order;
    SORT with no arguments restores submission-time order.

---

### Requirement 17: SDSF Panel Framework Extended (P1)

**User Story:** As a workbench user, I want the Job Monitor to provide a dedicated ST (status)
panel showing all jobs, advanced FILTER/FIND/LOCATE commands, SDSF-style scroll behaviour,
SET ACTION/MAIN/ROWNUM/WHO/QUERY AUTH commands, and persistent SET settings, so that the
full P1 SDSF command set is available.

**Source:** SDSF-JQ-4 (PAR), SDSF-FILTER-4, SDSF-FILTER-6, SDSF-FILTER-7, SDSF-SCROLL-1-5 (PAR),
SET-1, SET-8, SET-9, SET-12, SET-13, PERSIST-1 (PAR). [JES, FFW-ARCH]

#### Acceptance Criteria

1. THE JobMonitorPanel SHALL provide a dedicated ST (Status) sub-panel displaying all jobs
   regardless of queue state, with STATUS column showing QUEUED/HELD/ACTIVE/COMPLETED/FAILED/CANCELLED.

2. THE system SHALL support the FILTER command -- FILTER expression applies an advanced
   filter to the job table using field comparisons (e.g., FILTER JOBNAME=PAY* AND STATUS=ACTIVE);
   FILTER with no arguments clears the active filter.

3. THE system SHALL support the FIND command -- FIND string searches within the currently
   visible panel data and highlights the first matching row; FIND NEXT advances to the
   next match; FIND PREV moves to the previous match.

4. THE system SHALL support the LOCATE command -- LOCATE jobname scrolls the job table
   to the first row whose JOBNAME begins with the specified string; if no match exists
   the panel scrolls to the nearest alphabetic position.

5. THE JobMonitorPanel SHALL support SDSF-style scroll commands: UP [n|HALF|PAGE|MAX],
   DOWN [n|HALF|PAGE|MAX], LEFT [n|HALF|PAGE|MAX], RIGHT [n|HALF|PAGE|MAX]; the scroll
   amount defaults to the value in the SCROLL ===> field.

6. WHEN the user issues SET ACTION, THE system SHALL display a pop-up or inline list of
   all valid action characters for the current panel with their descriptions.

7. WHEN the user issues SET MAIN [panel-name], THE system SHALL set the specified panel
   as the default panel opened by the MENU command; if no panel-name is given the current
   panel becomes the default.

8. WHEN the user issues SET ROWNUM ON, THE system SHALL display row sequence numbers in
   the NP column area; WHEN the user issues SET ROWNUM OFF, THE system SHALL restore the
   NP action character input field.

9. WHEN the user issues WHO, THE system SHALL display a session information summary
   showing: current user identity, session start time, active filters (PREFIX/OWNER/DEST),
   current SET settings (ROWNUM, MAIN), and provider name.

10. WHEN the user issues QUERY AUTH, THE system SHALL display the list of JES commands
    and action characters the current user is authorised to execute, based on the
    capability model defined in command-semantics Requirement 9.16.

11. THE system SHALL persist all SET command settings (SET ACTION display preference,
    SET MAIN default panel, SET ROWNUM state) across application restarts using the
    workbench session persistence mechanism.

12. THE FILTER command SHALL support the following comparison operators in filter
    expressions: = (equals), != (not equals), > (greater than), < (less than),
    >= (greater than or equal), <= (less than or equal), and wildcard * in string values.

13. THE FILTER command SHALL support AND and OR logical operators to combine multiple
    field comparisons in a single filter expression.

14. THE ST panel SHALL be accessible via the command ST entered in the COMMAND INPUT
    ===> field, and via the S action on the main panel row for the ST command.

15. THE FIND command SHALL be case-insensitive by default; FIND C string performs a
    case-sensitive search.

16. WHEN a LOCATE or FIND command finds no match, THE system SHALL display a message
    in the title line message area: "string NOT FOUND" and leave the scroll position
    unchanged.

17. THE scroll commands (UP/DOWN/LEFT/RIGHT) SHALL update the SCROLL ===> field to
    reflect the most recently used scroll amount, consistent with Requirement 16.3.

---

---

### Requirement 18: SDSF P2 -- Overtype Fields, Help System, Log and System Panels, Browse and Print, SET P2 Commands

**User Story:** As a user, I want to modify job attributes directly in the SDSF panel by overtyping field values, access context-sensitive help for actions and columns, view system and user logs, browse and print job output, and configure display settings via SET commands, so that the SDSF emulation provides the full P2 operational experience.

**Source:** TSO-EARS SDSF panel framework (SDSF-3.x overtype, SDSF-5.x help), SDSF log panels (SDSF-LOG-1 through SDSF-LOG-4), SDSF system panels (SDSF-SYS-1 through SDSF-SYS-5), SDSF browse/print (SDSF-BROWSE-2 through SDSF-BROWSE-4), SET P2 commands (SET-2 through SET-11). [JES, WB]

#### Acceptance Criteria

1. THE SDSF panel SHALL visually distinguish overtypeable fields from read-only fields using a distinct colour or underline style defined by the active theme.
2. WHEN a user types a new value directly over an overtypeable field in the panel and presses Enter, THE panel SHALL apply the change to the underlying job or resource attribute and refresh the display.
3. THE panel SHALL support command-line overtype syntax: `<field-name> <value>` entered in the COMMAND INPUT field SHALL update the named field for the row identified by the cursor or NP column position.
4. WHEN an overtypeable field value exceeds the column width, THE panel SHALL display an Overtype Extension pop-up allowing the user to enter the full value in a larger input area.
5. THE SDSF panel SHALL provide context-sensitive help accessible via the HELP command or PF1: WHEN issued from a panel, THE help system SHALL display a help panel describing the current panel's purpose, available commands, and column definitions.
6. THE `ACTH` command SHALL display a help panel listing all valid action characters for the current panel with a one-line description of each action.
7. THE `COLH` command SHALL display a help panel listing all column names visible in the current panel with their data type, width, and description.
8. THE `CMDH` command SHALL display a help panel listing all primary commands valid in the current panel with syntax and description.
9. THE `SEARCH <text>` command within a help panel SHALL search the help content for the given text and scroll to the first match.
10. THE `LOG` command SHALL open the System Log panel displaying the JES system log output in reverse-chronological order (most recent entry first).
11. THE `ULOG` command SHALL open the User Log panel displaying messages directed to the current user's log.
12. WHEN the System Log or User Log panel is open, THE `NEXT` command SHALL scroll forward to the next log segment and `PREV` SHALL scroll backward to the previous segment.
13. THE `SNAPSHOT` command in a log panel SHALL capture the current log content to a dataset or file for offline review.
14. THE `SYS` command SHALL open the System Information panel displaying active address spaces, their status, and resource consumption.
15. THE `DASH` command SHALL open the System Dashboard panel displaying a summary of system health metrics (CPU, memory, I/O rates).
16. THE `INIT` command SHALL open the Initiator panel displaying the JES initiator pool status (class assignments, active/idle state).
17. THE `JC` command SHALL open the Job Class panel displaying job class definitions and their scheduling parameters.
18. THE `SP` command SHALL open the Spool panel displaying spool volume utilisation and track allocation.
19. WHEN browsing job output, THE panel SHALL support browse settings: line width, record format display, and FIND within the output stream.
20. THE `PRINT` action character applied to a job output dataset SHALL route the output to the configured print destination (local file or printer queue).
21. WHEN browsing job output, THE `COLS` command SHALL display a column ruler line showing the current horizontal scroll position and column numbers.
22. THE `SET BCOLOR <color>` command SHALL set the background colour of the SDSF panel display area, persisted across sessions.
23. THE `SET CONFIRM ON/OFF` command SHALL control whether destructive actions (cancel, purge) require a confirmation prompt before execution.
24. THE `SET CURSOR <field>` command SHALL set the default cursor landing position when a panel is opened.
25. THE `SET DATE <format>` command SHALL set the date display format (MDY, DMY, YMD, JUL) used in date columns across all SDSF panels.
26. THE `SET DELAY <seconds>` command SHALL set the automatic refresh interval for SDSF panels; `SET DELAY 0` disables automatic refresh.
27. THE `SET HEX ON/OFF` command SHALL toggle hexadecimal display of field values in the current panel.
28. THE `SET SCHARS <chars>` command SHALL define the set of special characters recognised as field delimiters in overtype and filter expressions.
29. THE `SET SCREEN <rows> <cols>` command SHALL set the logical screen dimensions used for panel layout calculations.
30. ALL SET P2 command settings (BCOLOR, CONFIRM, CURSOR, DATE, DELAY, HEX, SCHARS, SCREEN) SHALL be persisted across sessions using the same mechanism as SET P1 settings defined in Requirement 17.17.
