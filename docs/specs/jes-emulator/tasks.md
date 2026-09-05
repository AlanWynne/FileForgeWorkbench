# Implementation Plan: Job Entry Subsystem (`ff-jes`)

## Overview

This task plan implements the `ff-jes` crate — the FileForge Workbench Job Entry Subsystem. It provides mainframe-style batch job management: job submission via FFJCL, priority queue scheduling, initiator-pool-based execution, SDSF-style monitoring panels, dataset catalog integration, retained output with configurable purge, and a provider abstraction for future extensibility.

**Crate location:** `crates/ff-jes`
**Upstream dependencies:** `ff-plugin` (Wave 2), `ff-command` (Wave 2), `ff-layout` (Wave 2), `ff-workflow` (Wave 2), `ff-vfs` (Wave 3), `ff-config` (Wave 2), `ff-logging` (Wave 0), `ff-dataset-catalog` (Wave 13), `ff-dataset-allocator` (Wave 13)
**Downstream consumers:** Lua macro scripts, file-tree-panel (catalog links), future remote providers

---

## Tasks

- [x] 1. Project scaffold and core types
  - [x] 1.1 Create `crates/ff-jes/Cargo.toml` with dependencies (tokio, async-trait, thiserror, anyhow, chrono, serde, serde_json, uuid, rusqlite with bundled feature, ff-plugin, ff-command, ff-layout, ff-workflow, ff-vfs, ff-config, ff-logging, ff-dataset-catalog, ff-dataset-allocator) and dev-dependencies (proptest, tempfile, pretty_assertions, tokio-test)
  - [x] 1.2 Create `crates/ff-jes/src/lib.rs` with crate-level doc comment and public module declarations (plugin, model, queue, scheduler, initiator, engine, ffjcl, dataset_bridge, log_manager, sysout, retention, panels, commands, provider, api, config, async_infra, error)
  - [x] 1.3 Implement `src/error.rs` — define `JesError` enum with variants (SubmissionFailed, ValidationError, SchedulerError, InitiatorFailed, CatalogResolutionFailed, PurgeError, ProviderUnavailable, QueuePersistenceError, JobNotFound, InvalidJobState, CancellationTimeout, FfjclParseError, LogAccessError)
  - [x] 1.4 Implement `src/model.rs` — define `Job` struct (id: JobId, name: String, owner: String, status: JobStatus, priority: u32, submit_time: DateTime, start_time: Option, end_time: Option, initiator_id: Option, return_code: Option, steps: Vec<JobStep>, definition: FfjclDefinition)
  - [x] 1.5 Implement `JobStatus` enum (Queued, Held, Active, Completed, Failed, Cancelled) with Display, PartialEq, Eq, Clone, Serialize, Deserialize
  - [x] 1.6 Implement `JobId` newtype wrapping u64, monotonically increasing, with Display, FromStr, Eq, Hash, Ord
  - [x] 1.7 Implement `JobStep` struct (name: String, program: String, dd_statements: Vec<DdStatement>, return_code: Option<i32>, status: StepStatus, start_time: Option, end_time: Option)
  - [x] 1.8 Implement `DdStatement` struct (dd_name: String, dsn: Option<String>, disp: Disposition, resolved_path: Option<PathBuf>)
  - [x] 1.9 Implement `Disposition` enum (New, Old, Shr, Mod) with from_str parsing
  - [x] 1.10 Write unit tests for model types: JobStatus transitions, JobId ordering, Disposition parsing, Display implementations
    - Validates: Requirement 2 AC 2, AC 4; Requirement 6 AC 1–3

- [x] 2. FFJCL parser
  - [x] 2.1 Implement `src/ffjcl/mod.rs` — module structure with parser, ast, and validator sub-modules
  - [x] 2.2 Implement `src/ffjcl/ast.rs` — define `FfjclDefinition` struct (job_name: String, owner: Option<String>, priority: Option<u32>, class: Option<String>, steps: Vec<FfjclStep>, comments: Vec<String>), `FfjclStep` struct (step_name: String, program: String, dd_statements: Vec<FfjclDd>, condition: Option<StepCondition>), `FfjclDd` struct (dd_name, dsn, disp, space, dcb params)
  - [x] 2.3 Implement `src/ffjcl/parser.rs` — `parse_ffjcl(input: &str) -> Result<FfjclDefinition, JesError>` parsing job header (JOB statement), EXEC statements, DD statements with DSN/DISP/SPACE/DCB parameters, continuation lines, and comments
  - [x] 2.4 Implement `src/ffjcl/validator.rs` — `validate_definition(def: &FfjclDefinition) -> Result<(), Vec<ValidationIssue>>` checking: job name present, at least one step, DD names unique per step, DSN format valid (delegating to ff-dataset-catalog DSN validation), required fields present
  - [x] 2.5 Write unit tests for FFJCL parsing: valid single-step job, multi-step job, continuation lines, comments, invalid syntax, missing job name, duplicate DD names
    - Validates: Requirement 2 AC 1, AC 7
  - [x] 2.6 Write property test: FFJCL round-trip (Property 1) — generate valid FfjclDefinition ASTs, serialize to FFJCL text, re-parse, assert structural equality
    - Validates: Requirement 2 AC 1
  - [x] 2.7 Write property test: FFJCL validation rejects invalid definitions (Property 2) — generate definitions missing required fields or with invalid DSN refs, assert ValidationError with meaningful messages
    - Validates: Requirement 2 AC 7

- [x] 3. Job queue with persistence
  - [x] 3.1 Implement `src/queue/mod.rs` — module structure with store and operations sub-modules
  - [x] 3.2 Implement `src/queue/store.rs` — define `JobQueueStore` struct wrapping rusqlite Connection; implement schema (jobs table with columns: id, name, owner, status, priority, submit_time, start_time, end_time, initiator_id, return_code, definition_json, failure_reason, cancel_requester, cancel_time)
  - [x] 3.3 Implement `JobQueueStore::initialize(path)` — create SQLite database with WAL journal mode, execute schema migration
  - [x] 3.4 Implement `JobQueueStore::insert_job(job)` — persist new job to database, assign next monotonic JobId
  - [x] 3.5 Implement `JobQueueStore::update_status(id, new_status, metadata)` — update job status and associated timestamp fields atomically
  - [x] 3.6 Implement `JobQueueStore::get_job(id)` — retrieve single job by ID
  - [x] 3.7 Implement `JobQueueStore::query_jobs(filter)` — query jobs by status, owner, name pattern, date range; support sorting by priority, submit_time, name, id
  - [x] 3.8 Implement `JobQueueStore::next_job_id()` — return next monotonic ID (max(id)+1 or session-based counter)
  - [x] 3.9 Implement `JobQueueStore::get_eligible_jobs()` — return jobs with status=Queued ordered by priority DESC, submit_time ASC (preconditions met)
  - [x] 3.10 Write unit tests for queue store: insert/query/update lifecycle, persistence across re-open, monotonic ID generation, eligible job ordering
    - Validates: Requirement 2 AC 2, AC 3, AC 4, AC 6; Requirement 3 AC 1, AC 2
  - [x] 3.11 Write property test: monotonic JobId invariant (Property 3) — submit N jobs in arbitrary order, assert all IDs are strictly increasing
    - Validates: Requirement 2 AC 2
  - [x] 3.12 Write property test: queue ordering correctness (Property 4) — insert jobs with random priorities and timestamps, query eligible, assert highest-priority first then FIFO within same priority
    - Validates: Requirement 3 AC 1, AC 2, AC 3

- [x] 4. Scheduler
  - [x] 4.1 Implement `src/scheduler.rs` — define `Scheduler` struct with fields: queue_store (Arc), initiator_pool (Arc), poll_interval_ms (configurable), running flag (AtomicBool), event_tx (broadcast channel sender)
  - [x] 4.2 Implement `Scheduler::start()` — spawn async background task that polls for eligible jobs and available initiators at configured interval
  - [x] 4.3 Implement `Scheduler::dispatch_loop()` — on each tick: query eligible jobs, query idle initiators, dispatch highest-priority job to first available initiator; change status from Queued→Active, record start_time and initiator_id
  - [x] 4.4 Implement `Scheduler::stop()` — set running flag to false, await task completion
  - [x] 4.5 Implement dispatch precondition checking — verify predecessor jobs completed (if defined in FFJCL), verify required datasets resolvable via catalog bridge
  - [x] 4.6 Implement scheduling strategy selection — support FIFO (default) and Priority strategies via configuration
  - [x] 4.7 Write unit tests for scheduler dispatch: FIFO ordering, priority ordering, held jobs skipped, cancelled jobs skipped, precondition blocking, concurrent dispatch up to pool capacity
    - Validates: Requirement 3 AC 1–7

- [x] 5. Initiator pool
  - [x] 5.1 Implement `src/initiator/mod.rs` — module structure with pool and worker sub-modules
  - [x] 5.2 Implement `src/initiator/pool.rs` — define `InitiatorPool` struct with fields: workers (Vec<Initiator>), capacity (usize), config
  - [x] 5.3 Implement `Initiator` struct (id: InitiatorId, status: InitiatorStatus, current_job: Option<JobId>, handle: Option<JoinHandle>)
  - [x] 5.4 Implement `InitiatorStatus` enum (Idle, Starting, Active, Stopping, Stopped, Failed, Draining) with Display
  - [x] 5.5 Implement `InitiatorPool::new(capacity)` — create pool with configured number of initiators in Idle state
  - [x] 5.6 Implement `InitiatorPool::get_available()` — return first initiator with status Idle
  - [x] 5.7 Implement `InitiatorPool::dispatch(initiator_id, job)` — assign job to initiator, set status to Active, spawn execution task on Tokio runtime
  - [x] 5.8 Implement `InitiatorPool::start_initiator(id)` — transition specific initiator from Stopped→Idle
  - [x] 5.9 Implement `InitiatorPool::stop_initiator(id)` — set Stopping; if active job running, wait for completion then set Stopped
  - [x] 5.10 Implement `InitiatorPool::drain_initiator(id)` — set Draining; complete current job but accept no new work
  - [x] 5.11 Implement initiator failure recovery — when execution panics or errors unrecoverably, mark initiator as Failed, log error, continue with remaining initiators
  - [x] 5.12 Write unit tests for pool: capacity enforcement, dispatch to idle, stop with active job, drain semantics, failure recovery, concurrent dispatch limit
    - Validates: Requirement 4 AC 1–8
  - [x] 5.13 Write property test: pool capacity invariant (Property 5) — dispatch N jobs to pool of capacity C, assert active count never exceeds C
    - Validates: Requirement 4 AC 1; Requirement 3 AC 7

- [x] 6. Job execution engine
  - [x] 6.1 Implement `src/engine/mod.rs` — module structure with executor and step_runner sub-modules
  - [x] 6.2 Implement `src/engine/executor.rs` — define `JobExecutor` struct; implement `execute(job: &mut Job, log_writer: &dyn JobLogWriter)` that iterates through job steps sequentially
  - [x] 6.3 Implement step execution: for each JobStep, resolve DD statements via dataset bridge, spawn process for program, capture stdout/stderr as SYSOUT, record return code
  - [x] 6.4 Implement process spawning via `tokio::process::Command` with environment setup, working directory from DD resolution, stdin/stdout/stderr capture
  - [x] 6.5 Implement step condition evaluation — check COND parameter from FFJCL (e.g., COND=(0,NE) skips step if prior RC≠0)
  - [x] 6.6 Implement job completion handling — set status to Completed if all steps pass; set Failed if any step abends; record final return code (max RC across steps)
  - [x] 6.7 Implement cancellation handling — receive cancel signal via CancellationToken, send SIGTERM/TerminateProcess to active process, wait configurable timeout, force-kill if timeout expires
  - [x] 6.8 Implement elapsed time tracking — record start/end per step and overall job
  - [x] 6.9 Write unit tests for executor: single-step success, multi-step with condition codes, step failure propagation, cancellation signal handling, timeout force-kill
    - Validates: Requirement 6 AC 1–8; Requirement 15 AC 1

- [x] 7. Dataset resolution bridge
  - [x] 7.1 Implement `src/dataset_bridge.rs` — define `DatasetBridge` struct wrapping Arc<ff-dataset-allocator> and Arc<ff-dataset-catalog> references
  - [x] 7.2 Implement `DatasetBridge::resolve_dd(dd: &DdStatement) -> Result<ResolvedDd>` — resolve DSN through allocator API: OLD/SHR → catalog lookup, NEW → allocate via allocator, MOD → lookup existing or create new
  - [x] 7.3 Implement GDG relative reference resolution — `(+1)` triggers new generation allocation, `(0)` resolves to current, `(-N)` resolves to Nth prior generation
  - [x] 7.4 Implement allocation message generation — for each DD resolution, produce structured allocation messages (DSN, resolved path, disposition, catalog entry metadata) for the job log
  - [x] 7.5 Implement failure handling — if DSN not found and DISP≠NEW, return CatalogResolutionFailed with descriptive error written to job log
  - [x] 7.6 Write unit tests for dataset bridge: resolve OLD existing, resolve NEW allocation, resolve SHR, resolve MOD (existing/new), GDG relative refs, not-found failure, allocation message content
    - Validates: Requirement 11 AC 1–7
  - [x] 7.7 Write property test: disposition resolution consistency (Property 6) — generate random DD statements with valid/invalid DSNs and dispositions, assert OLD/SHR fail on missing, NEW always creates, MOD handles both cases
    - Validates: Requirement 11 AC 1, AC 2, AC 3

- [x] 8. Job log manager and SYSOUT handling
  - [x] 8.1 Implement `src/log_manager/mod.rs` — module structure with writer, reader, and storage sub-modules
  - [x] 8.2 Implement `src/log_manager/storage.rs` — define log storage layout: per-job directory (`spool/{JOB_ID}/`) containing `jeslog.txt`, `stepN_sysout.txt`, `stepN_stderr.txt`, `alloc_messages.txt`
  - [x] 8.3 Implement `JobLogWriter` trait — methods: write_jes_message, write_step_output, write_alloc_message, write_error, flush; implementations write to spool files
  - [x] 8.4 Implement `FileJobLogWriter` — writes log entries to spool files with timestamps and structured format
  - [x] 8.5 Implement `src/sysout.rs` — define `SysoutCapture` struct that captures process stdout/stderr via async channels, writes to spool, and supports live streaming to UI subscribers
  - [x] 8.6 Implement `SysoutCapture::subscribe()` — return async Receiver for live log streaming (new subscribers get buffered history + live tail)
  - [x] 8.7 Implement `JobLogReader` — methods: read_full_log(job_id), read_section(job_id, section), stream_live(job_id) returning async Stream of log lines
  - [x] 8.8 Implement log section parsing — partition stored logs into sections: JES Log, Step Logs (per step), SYSOUT, Error Output, Allocation Messages
  - [x] 8.9 Implement incremental log loading — for large logs, load by page/offset to avoid UI blocking
  - [x] 8.10 Write unit tests for log writer: write all message types, verify file structure; log reader: read sections, stream live lines, incremental loading
    - Validates: Requirement 7 AC 1–7; Requirement 15 AC 3

- [x] 9. Retention and purge engine
  - [x] 9.1 Implement `src/retention/mod.rs` — module structure with policy and purge sub-modules
  - [x] 9.2 Implement `src/retention/policy.rs` — define `RetentionPolicy` struct (max_days: u32, max_jobs: u32) loaded from configuration
  - [x] 9.3 Implement `RetentionEngine::new(policy, queue_store, log_storage)` — construct engine with references to queue and spool storage
  - [x] 9.4 Implement `RetentionEngine::purge_job(job_id)` — remove job logs and SYSOUT from spool, update job record (mark purged), do NOT delete catalogued datasets unless explicit flag
  - [x] 9.5 Implement `RetentionEngine::batch_purge(filter)` — purge multiple jobs matching filter criteria (by date range, status, owner)
  - [x] 9.6 Implement `RetentionEngine::auto_purge()` — background task that runs on configurable schedule, identifies jobs exceeding retention policy (age > max_days OR total count > max_jobs), purges oldest first
  - [x] 9.7 Implement purge confirmation requirement — destructive purge actions emit a confirmation event that the UI must acknowledge before proceeding
  - [x] 9.8 Write unit tests for retention: policy evaluation, single purge, batch purge by filter, auto-purge ordering (oldest first), dataset preservation, confirmation flag
    - Validates: Requirement 8 AC 1–6
  - [x] 9.9 Write property test: retention policy correctness (Property 7) — generate N jobs with random completion dates, apply policy with max_days=D and max_jobs=M, assert retained set satisfies both constraints
    - Validates: Requirement 8 AC 1, AC 3

- [x] 10. Provider abstraction
  - [x] 10.1 Implement `src/provider/mod.rs` — module structure with trait definition and desktop provider
  - [x] 10.2 Implement `src/provider/trait.rs` — define `JobProvider` trait with async methods: list_jobs(filter), submit_job(definition), hold_job(id), release_job(id), cancel_job(id), get_job_log(id), subscribe_to_events() returning broadcast Receiver
  - [x] 10.3 Implement `src/provider/desktop.rs` — define `DesktopJesProvider` struct implementing `JobProvider` by delegating to local JobQueueStore, Scheduler, InitiatorPool, and LogManager
  - [x] 10.4 Implement `DesktopJesProvider::new(config)` — construct with all local subsystem references
  - [x] 10.5 Implement provider identification — each provider has a `name()` and `provider_id()` method; jobs carry a `source_provider` field
  - [x] 10.6 Implement `ProviderRegistry` — manages multiple registered providers, routes operations to correct provider based on job source
  - [x] 10.7 Implement provider error handling — connection errors per provider are isolated; one provider failing does not affect others
  - [x] 10.8 Write unit tests for provider trait: desktop provider delegates correctly, provider registry routes to correct provider, error isolation between providers
    - Validates: Requirement 14 AC 1–7

- [x] 11. Job Monitor panel (SDSF-style)
  - [x] 11.1 Implement `src/panels/mod.rs` — module structure for JobMonitorPanel and JobLogViewerPanel
  - [x] 11.2 Implement `src/panels/job_monitor.rs` — define `JobMonitorPanel` struct implementing `DockablePanel` trait with `panel_id()`, `title()`, `default_dock_zone()` (Bottom), `render()`, `on_event()`
  - [x] 11.3 Implement tabbed sub-panels: InputQueueTab, ActiveJobsTab, HeldJobsTab, OutputTab, FailedTab, CancelledTab — each displaying job count in tab header
  - [x] 11.4 Implement job table rendering per tab — columns: Job Name, Job ID, Owner, Submit Time, Priority, Status, Start Time, Elapsed, Initiator, Return Code (as applicable per tab)
  - [x] 11.5 Implement table sorting — click column header to sort by that column; support ascending/descending toggle
  - [x] 11.6 Implement filtering UI — filter bar with fields: Owner, Job Name, Job ID, Status, Date Range, Return Code; filters persist across tab switches
  - [x] 11.7 Implement auto-refresh — configurable interval (default 2000ms) using event push where available, polling as fallback; refresh does NOT reset filters or scroll position
  - [x] 11.8 Implement visual status indicators — icons/colours distinguishing Queued, Held, Active, Completed, Failed, Cancelled states
  - [x] 11.9 Implement context menu per job row — actions: View Log, Hold, Release, Cancel, Purge, Properties; enable/disable based on job status
  - [x] 11.10 Implement active job details — for active jobs show: elapsed time (updating), current step, process metrics (PID, CPU%, Memory) where OS provides them
  - [x] 11.11 Write unit tests for panel: tab rendering with correct job counts, filter application, sort ordering, context menu enable/disable logic, auto-refresh state preservation
    - Validates: Requirement 9 AC 1–10; Requirement 5 AC 1–5; Requirement 3 AC 8–10

- [x] 12. Job Log Viewer panel
  - [x] 12.1 Implement `src/panels/job_log_viewer.rs` — define `JobLogViewerPanel` struct implementing `DockablePanel` trait with `panel_id()`, `title()`, `default_dock_zone()` (Center), `render()`, `on_event()`
  - [x] 12.2 Implement sectioned display — tabs or collapsible sections for: JES Log, Step Log (per step), SYSOUT, Error Output, Allocation Messages
  - [x] 12.3 Implement search within log content — Ctrl+F search bar with next/previous navigation, highlight matches
  - [x] 12.4 Implement copy-to-clipboard and export-to-file (via VFS) actions
  - [x] 12.5 Implement live log streaming — for active jobs, append new output lines in real-time via SysoutCapture subscription
  - [x] 12.6 Implement large log virtualization — only render visible lines, load incrementally, maintain scroll position during live updates
  - [x] 12.7 Write unit tests for log viewer: section navigation, search highlight, live stream append without scroll reset, export generates valid VFS path
    - Validates: Requirement 7 AC 1–7

- [x] 13. Command registration
  - [x] 13.1 Implement `src/commands/mod.rs` — module structure for all JES commands
  - [x] 13.2 Implement `jes.job.submit` command — params: jcl_source (path or inline text); parse FFJCL, validate, submit to queue; return JobId
  - [x] 13.3 Implement `jes.job.hold` command — params: job_id; validate job is Queued, change status to Held; error if Active
  - [x] 13.4 Implement `jes.job.release` command — params: job_id; validate job is Held, change status to Queued
  - [x] 13.5 Implement `jes.job.cancel` command — params: job_id; if Queued/Held set Cancelled; if Active send termination signal
  - [x] 13.6 Implement `jes.job.purge` command — params: job_id or filter; delegate to RetentionEngine with confirmation
  - [x] 13.7 Implement `jes.job.view_log` command — params: job_id; open JobLogViewerPanel with specified job's log
  - [x] 13.8 Implement `jes.monitor.refresh` command — trigger immediate refresh of JobMonitorPanel; default shortcut F5
  - [x] 13.9 Implement `jes.initiator.start` command — params: initiator_id; start specific initiator
  - [x] 13.10 Implement `jes.initiator.stop` command — params: initiator_id; stop specific initiator (graceful)
  - [x] 13.11 Implement `jes.initiator.drain` command — params: initiator_id; drain specific initiator
  - [x] 13.12 Implement `jes.catalog.browse` command — open file-tree-panel focused on Catalogs node
  - [x] 13.13 Implement command metadata — each command with: display name, description, category (jes.job | jes.initiator | jes.catalog), default keyboard shortcut where applicable
  - [x] 13.14 Implement enabled predicates — jes.job.cancel enabled only when job is Queued or Active; jes.job.hold only when Queued; jes.job.release only when Held
  - [x] 13.15 Write unit tests for command registration, parameter validation, enabled predicate logic, dispatch to correct subsystem operations
    - Validates: Requirement 13 AC 1–4; Requirement 10 AC 1–4

- [x] 14. Job and Dataset APIs
  - [x] 14.1 Implement `src/api/mod.rs` — module structure for Job API and Dataset API
  - [x] 14.2 Implement `src/api/job_api.rs` — define `JobApi` struct exposing: submit, hold, release, cancel, query_status, retrieve_logs, retrieve_output, subscribe_events methods; accessible from other plugins and Lua macros
  - [x] 14.3 Implement `JobApi::subscribe_events()` — return broadcast Receiver delivering JobStatusChange events (job_id, old_status, new_status, timestamp)
  - [x] 14.4 Implement `src/api/dataset_api.rs` — define `DatasetApi` struct exposing: allocate, read, write, delete, resolve_dsn, query_metadata, open_in_editor methods; delegates to ff-dataset-allocator and ff-dataset-catalog
  - [x] 14.5 Implement Lua scripting bridge integration — register Job API operations as invocable via `workbench.execute("jes.job.submit", {jcl = "..."})` and similar patterns
  - [x] 14.6 Write unit tests for Job API: submit returns valid JobId, subscribe receives status transitions, cancel emits event; Dataset API: resolve delegates to allocator, allocate creates entry
    - Validates: Requirement 12 AC 1–5

- [x] 15. Configuration
  - [x] 15.1 Implement `src/config.rs` — define `JesConfig` struct deserializable from `[plugins.ffw-jes]` TOML table with fields: initiator_count (default 3), retention_days (default 7), retention_max_jobs (default 1000), monitor_refresh_ms (default 2000), scheduler_poll_ms (default 500), job_cancel_timeout_ms (default 30000)
  - [x] 15.2 Implement `JesConfig::load(config_service)` — read from ff-config, validate values (initiator_count > 0, all intervals > 0, retention_days > 0)
  - [x] 15.3 Implement hot-reload support — subscribe to config change events, apply initiator count changes (grow/shrink pool), update intervals without restart
  - [x] 15.4 Write unit tests for config loading, validation, default values, hot-reload application
    - Validates: Requirement 1 AC 4; Requirement 4 AC 1; Requirement 8 AC 1; Requirement 9 AC 7

- [x] 16. Async infrastructure
  - [x] 16.1 Implement `src/async_infra.rs` — define shared Tokio runtime configuration, cancellation token hierarchy (plugin-level → scheduler → initiators), graceful shutdown coordination
  - [x] 16.2 Implement async channel infrastructure — define typed channels: job_status_events (broadcast), log_lines (mpsc per job), scheduler_commands (mpsc), initiator_commands (mpsc per initiator)
  - [x] 16.3 Implement `spawn_blocking` wrappers for synchronous operations (SQLite writes, file I/O) that must not block the async runtime
  - [x] 16.4 Implement event-driven Job Monitor refresh — status change events push to UI subscription rather than polling where feasible
  - [x] 16.5 Write unit tests for async infrastructure: cancellation propagation, channel delivery, spawn_blocking does not block async tasks, event-driven refresh triggers UI update
    - Validates: Requirement 15 AC 1–5

- [x] 17. Plugin entry point
  - [x] 17.1 Implement `src/plugin.rs` — define `JesPlugin` struct implementing `FileForgePlugin` trait with fields: config, queue_store, scheduler, initiator_pool, log_manager, retention_engine, provider_registry, job_api, dataset_api
  - [x] 17.2 Implement `JesPlugin::metadata()` — return PluginMetadata with name="ffw-jes", capabilities=[Commands, Viewers, Providers], dependencies=[ff-vfs, ff-workflow, ff-dataset-catalog, ff-dataset-allocator]
  - [x] 17.3 Implement `JesPlugin::initialize(ctx: &PluginContext)` — register all JES commands with command registry under `jes.*` namespace
  - [x] 17.4 Implement `JesPlugin::activate(ctx: &PluginContext)` — register panels (JobMonitorPanel, JobLogViewerPanel), initialize initiator pool, start scheduler, start retention auto-purge background task
  - [x] 17.5 Implement `JesPlugin::deactivate(ctx: &PluginContext)` — stop scheduler, gracefully stop all initiators (allow active jobs to complete or cancel), persist queue state, deregister capabilities
  - [x] 17.6 Implement `JesPlugin::shutdown(ctx: &PluginContext)` — persist retained job output and catalog state, flush logs, close all resources (SQLite connections, channels)
  - [x] 17.7 Implement enable/disable support — plugin can be disabled without unloading; disabled state stops scheduler and ignores commands
  - [x] 17.8 Write unit tests for plugin lifecycle: initialize registers commands, activate starts subsystems, deactivate persists and stops, shutdown cleans up; verify independent enable/disable
    - Validates: Requirement 1 AC 1–9
  - [x] 17.9 Write property test: plugin lifecycle state machine (Property 8) — generate random sequences of initialize/activate/deactivate/shutdown calls, assert no panics and correct state transitions (cannot activate before initialize, cannot double-activate, etc.)
    - Validates: Requirement 1 AC 1, AC 5, AC 6

- [x] 18. Hold and release operations
  - [x] 18.1 Implement `src/operations/hold_release.rs` — define `hold_job(queue_store, job_id)` — validate job status is Queued, update to Held; return error if Active or terminal
  - [x] 18.2 Implement `release_job(queue_store, job_id)` — validate job status is Held, update to Queued; return error if not Held
  - [x] 18.3 Implement status transition validation — enforce valid transitions: Queued→Held (hold), Held→Queued (release); reject all other hold/release combinations with descriptive errors
  - [x] 18.4 Write unit tests for hold/release: valid transitions, invalid transitions (hold Active, release Queued), error messages
    - Validates: Requirement 10 AC 1–4
  - [x] 18.5 Write property test: job status transition validity (Property 9) — generate random (status, action) pairs, assert only valid transitions succeed and invalid ones return appropriate errors
    - Validates: Requirement 10 AC 1–4; Requirement 6 AC 1–3

- [x] 19. Integration tests and end-to-end validation
  - [x] 19.1 Write integration test: full job lifecycle — submit FFJCL job, observe Queued status, scheduler dispatches to initiator, job executes (simple echo program), observe Active→Completed, verify return code and job log content
  - [x] 19.2 Write integration test: job cancellation — submit job that sleeps, cancel while Active, verify SIGTERM sent, timeout triggers force-kill, final status is Cancelled with preserved partial logs
  - [x] 19.3 Write integration test: hold and release — submit job, hold before dispatch, verify scheduler skips it, release, verify scheduler dispatches it
  - [x] 19.4 Write integration test: dataset resolution — submit FFJCL with DSN references (existing OLD, new NEW allocation, GDG relative ref), verify allocation messages in job log, verify datasets created in catalog
  - [x] 19.5 Write integration test: retention and purge — submit and complete multiple jobs, configure short retention, trigger auto-purge, verify oldest jobs purged while datasets preserved
  - [x] 19.6 Write integration test: provider abstraction — register DesktopJesProvider, submit job through provider API, verify provider_id tagged on job, verify provider registry routes correctly
  - [x] 19.7 Write integration test: plugin lifecycle — initialize JesPlugin, verify commands registered; activate, verify panels and scheduler running; deactivate, verify persistence and cleanup; reinitialize and verify queue state restored
  - [x] 19.8 Write integration test: concurrent job execution — submit N jobs to pool of C initiators, verify max C run concurrently, all eventually complete, no race conditions on queue state
    - Validates: All requirements end-to-end

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Plugin Registration and Lifecycle | AC 1 (FileForgePlugin trait) | 17.1, 17.2 |
| Req 1: Plugin Registration and Lifecycle | AC 2 (initialize registers commands) | 17.3, 17.8 |
| Req 1: Plugin Registration and Lifecycle | AC 3 (activate registers panels) | 17.4, 17.8 |
| Req 1: Plugin Registration and Lifecycle | AC 4 (activate starts pool/scheduler) | 17.4, 15.1 |
| Req 1: Plugin Registration and Lifecycle | AC 5 (deactivate graceful stop) | 17.5, 17.9 |
| Req 1: Plugin Registration and Lifecycle | AC 6 (shutdown persists state) | 17.6, 17.9 |
| Req 1: Plugin Registration and Lifecycle | AC 7 (metadata declaration) | 17.2, 17.8 |
| Req 1: Plugin Registration and Lifecycle | AC 8 (enable/disable) | 17.7, 17.8 |
| Req 1: Plugin Registration and Lifecycle | AC 9 (DockablePanel) | 11.2, 12.1 |
| Req 2: Job Submission | AC 1 (parse and create job) | 2.3, 2.4, 13.2 |
| Req 2: Job Submission | AC 2 (unique monotonic JobId) | 1.6, 3.8, 3.11 |
| Req 2: Job Submission | AC 3 (submission timestamp/owner) | 1.4, 3.4 |
| Req 2: Job Submission | AC 4 (initial status Queued) | 1.5, 3.4, 3.10 |
| Req 2: Job Submission | AC 5 (appears in monitor) | 11.3, 11.7 |
| Req 2: Job Submission | AC 6 (queue persistence) | 3.2, 3.3, 3.10 |
| Req 2: Job Submission | AC 7 (validation rejection) | 2.4, 2.7, 13.2 |
| Req 2: Job Submission | AC 8 (multiple submit sources) | 13.2, 14.2, 14.5 |
| Req 3: Job Queue and Scheduling | AC 1 (FIFO default) | 4.6, 3.12 |
| Req 3: Job Queue and Scheduling | AC 2 (priority scheduling) | 4.6, 3.12 |
| Req 3: Job Queue and Scheduling | AC 3 (dispatch highest priority) | 4.3, 4.7 |
| Req 3: Job Queue and Scheduling | AC 4 (skip Held/Cancelled) | 4.3, 4.7 |
| Req 3: Job Queue and Scheduling | AC 5 (precondition checking) | 4.5, 4.7 |
| Req 3: Job Queue and Scheduling | AC 6 (Queued→Active transition) | 4.3, 4.7 |
| Req 3: Job Queue and Scheduling | AC 7 (capacity enforcement) | 5.13, 4.7 |
| Req 3: Job Queue and Scheduling | AC 8 (monitor display) | 11.4, 11.5 |
| Req 3: Job Queue and Scheduling | AC 9 (auto-update) | 11.7 |
| Req 3: Job Queue and Scheduling | AC 10 (visual status indicators) | 11.8 |
| Req 4: Initiator Pool | AC 1 (configurable count) | 5.5, 15.1 |
| Req 4: Initiator Pool | AC 2 (unique initiator ID) | 5.3, 5.12 |
| Req 4: Initiator Pool | AC 3 (initiator status visible) | 5.4, 11.10 |
| Req 4: Initiator Pool | AC 4 (start command) | 5.8, 13.9 |
| Req 4: Initiator Pool | AC 5 (stop command) | 5.9, 13.10 |
| Req 4: Initiator Pool | AC 6 (drain command) | 5.10, 13.11 |
| Req 4: Initiator Pool | AC 7 (async on Tokio) | 5.7, 16.1 |
| Req 4: Initiator Pool | AC 8 (failure recovery) | 5.11, 5.12 |
| Req 5: Active Job Monitoring | AC 1 (active job details) | 11.10, 11.4 |
| Req 5: Active Job Monitoring | AC 2 (process metrics) | 11.10 |
| Req 5: Active Job Monitoring | AC 3 (auto-refresh) | 11.7 |
| Req 5: Active Job Monitoring | AC 4 (live log viewing) | 12.5, 8.6 |
| Req 5: Active Job Monitoring | AC 5 (cancel from monitor) | 11.9, 13.5 |
| Req 6: Job Completion, Failure, Cancellation | AC 1 (completed status) | 6.6, 6.9 |
| Req 6: Job Completion, Failure, Cancellation | AC 2 (failed status) | 6.6, 6.9 |
| Req 6: Job Completion, Failure, Cancellation | AC 3 (cancelled queued job) | 13.5, 18.3 |
| Req 6: Job Completion, Failure, Cancellation | AC 4 (cancelled active job) | 6.7, 6.9 |
| Req 6: Job Completion, Failure, Cancellation | AC 5 (release initiator) | 5.7, 5.12 |
| Req 6: Job Completion, Failure, Cancellation | AC 6 (retain output) | 9.4, 9.8 |
| Req 6: Job Completion, Failure, Cancellation | AC 7 (output panel) | 11.3 |
| Req 6: Job Completion, Failure, Cancellation | AC 8 (preserved logs) | 8.4, 8.10 |
| Req 7: Job Logs and SYSOUT | AC 1 (view_log command) | 13.7, 12.1 |
| Req 7: Job Logs and SYSOUT | AC 2 (log content structure) | 8.3, 8.4, 8.8 |
| Req 7: Job Logs and SYSOUT | AC 3 (sectioned display) | 12.2, 12.7 |
| Req 7: Job Logs and SYSOUT | AC 4 (search/copy/export) | 12.3, 12.4 |
| Req 7: Job Logs and SYSOUT | AC 5 (live/completed/failed/cancelled) | 12.5, 12.7 |
| Req 7: Job Logs and SYSOUT | AC 6 (large log handling) | 8.9, 12.6 |
| Req 7: Job Logs and SYSOUT | AC 7 (stable storage format) | 8.2, 8.10 |
| Req 8: Retained Output and Purge | AC 1 (retention config) | 9.2, 15.1 |
| Req 8: Retained Output and Purge | AC 2 (manual purge) | 9.4, 9.5, 13.6 |
| Req 8: Retained Output and Purge | AC 3 (auto purge) | 9.6, 9.9 |
| Req 8: Retained Output and Purge | AC 4 (remove logs on purge) | 9.4, 9.8 |
| Req 8: Retained Output and Purge | AC 5 (preserve datasets) | 9.4, 9.8 |
| Req 8: Retained Output and Purge | AC 6 (confirmation warning) | 9.7, 9.8 |
| Req 9: Job Monitor Panel | AC 1 (DockablePanel with tabs) | 11.2, 11.3 |
| Req 9: Job Monitor Panel | AC 2 (job count in tab) | 11.3, 11.11 |
| Req 9: Job Monitor Panel | AC 3 (open details/logs) | 11.9, 13.7 |
| Req 9: Job Monitor Panel | AC 4 (filters) | 11.6, 11.11 |
| Req 9: Job Monitor Panel | AC 5 (filters clearable) | 11.6, 11.11 |
| Req 9: Job Monitor Panel | AC 6 (dynamic filter results) | 11.6, 11.7 |
| Req 9: Job Monitor Panel | AC 7 (auto-refresh interval) | 11.7, 15.1 |
| Req 9: Job Monitor Panel | AC 8 (manual refresh F5) | 13.8, 11.7 |
| Req 9: Job Monitor Panel | AC 9 (refresh preserves state) | 11.7, 11.11 |
| Req 9: Job Monitor Panel | AC 10 (context menu) | 11.9, 11.11 |
| Req 10: Job Hold and Release | AC 1 (hold queued→held) | 18.1, 18.4 |
| Req 10: Job Hold and Release | AC 2 (release held→queued) | 18.2, 18.4 |
| Req 10: Job Hold and Release | AC 3 (held jobs panel) | 11.3 |
| Req 10: Job Hold and Release | AC 4 (cannot hold active) | 18.1, 18.5 |
| Req 11: Dataset Catalog Integration | AC 1 (DSN resolution via allocator) | 7.2, 7.6 |
| Req 11: Dataset Catalog Integration | AC 2 (not-found with non-NEW) | 7.5, 7.6 |
| Req 11: Dataset Catalog Integration | AC 3 (NEW allocation) | 7.2, 7.6 |
| Req 11: Dataset Catalog Integration | AC 4 (allocation messages) | 7.4, 7.6 |
| Req 11: Dataset Catalog Integration | AC 5 (GDG references) | 7.3, 7.6 |
| Req 11: Dataset Catalog Integration | AC 6 (catalog node in file-tree) | 13.12 |
| Req 11: Dataset Catalog Integration | AC 7 (cross-platform) | 7.2, 7.6 |
| Req 12: Job and Dataset APIs | AC 1 (Job API) | 14.2, 14.6 |
| Req 12: Job and Dataset APIs | AC 2 (Dataset API) | 14.4, 14.6 |
| Req 12: Job and Dataset APIs | AC 3 (Lua invocable) | 14.5 |
| Req 12: Job and Dataset APIs | AC 4 (event subscription) | 14.3, 14.6 |
| Req 12: Job and Dataset APIs | AC 5 (Dataset API delegates) | 14.4, 14.6 |
| Req 13: Command Integration | AC 1 (jes.* namespace) | 13.1–13.12, 13.15 |
| Req 13: Command Integration | AC 2 (command metadata) | 13.13, 13.15 |
| Req 13: Command Integration | AC 3 (enabled predicates) | 13.14, 13.15 |
| Req 13: Command Integration | AC 4 (invocable from palette/menus/keys/Lua) | 13.2–13.12, 14.5 |
| Req 14: Provider Abstraction | AC 1 (JobProvider trait) | 10.2, 10.8 |
| Req 14: Provider Abstraction | AC 2 (DesktopJesProvider) | 10.3, 10.4 |
| Req 14: Provider Abstraction | AC 3 (multi-provider display) | 10.6, 11.4 |
| Req 14: Provider Abstraction | AC 4 (provider source indicator) | 10.5, 11.4 |
| Req 14: Provider Abstraction | AC 5 (action enablement per provider) | 10.6, 11.9 |
| Req 14: Provider Abstraction | AC 6 (error isolation) | 10.7, 10.8 |
| Req 14: Provider Abstraction | AC 7 (trait methods) | 10.2, 10.8 |
| Req 15: Async Execution | AC 1 (async job execution) | 6.4, 16.1 |
| Req 15: Async Execution | AC 2 (async scheduler) | 4.2, 16.1 |
| Req 15: Async Execution | AC 3 (log streaming channels) | 8.5, 8.6, 16.2 |
| Req 15: Async Execution | AC 4 (async persistence) | 16.3, 16.5 |
| Req 15: Async Execution | AC 5 (event-driven refresh) | 16.4, 16.5 |

---

## Property-Based Test Summary

| Property | Statement | Task | Validates |
|----------|-----------|------|-----------|
| P1 | FFJCL round-trip: generate valid AST → serialize → re-parse → assert structural equality | 2.6 | Req 2 AC 1 |
| P2 | FFJCL validation rejects invalid definitions with meaningful messages | 2.7 | Req 2 AC 7 |
| P3 | Monotonic JobId invariant: submit N jobs in arbitrary order, all IDs strictly increasing | 3.11 | Req 2 AC 2 |
| P4 | Queue ordering: jobs dispatched highest-priority first, FIFO within same priority | 3.12 | Req 3 AC 1, 2, 3 |
| P5 | Pool capacity invariant: active initiator count never exceeds configured capacity | 5.13 | Req 4 AC 1; Req 3 AC 7 |
| P6 | Disposition resolution consistency: OLD/SHR fail on missing DSN, NEW always creates, MOD handles both | 7.7 | Req 11 AC 1, 2, 3 |
| P7 | Retention policy correctness: retained jobs satisfy both max_days and max_jobs constraints | 9.9 | Req 8 AC 1, 3 |
| P8 | Plugin lifecycle state machine: random call sequences never panic, only valid transitions succeed | 17.9 | Req 1 AC 1, 5, 6 |
| P9 | Job status transition validity: only legal (status, action) pairs succeed | 18.5 | Req 10 AC 1–4; Req 6 AC 1–3 |

---

## Notes

- Tasks 1 and 2 are foundation work with no internal dependencies and can be developed in parallel
- Task 3 (queue store) depends on task 1 (model types and error enum)
- Tasks 4 and 5 (scheduler, initiator pool) depend on task 3 (queue store provides eligible jobs)
- Task 6 (execution engine) depends on tasks 4 and 5 (dispatched by scheduler to initiators)
- Task 7 (dataset bridge) depends on task 2 (FFJCL parser provides DD statements) and external crates (ff-dataset-allocator, ff-dataset-catalog)
- Task 8 (log manager) depends on task 6 (engine produces log output during execution)
- Task 9 (retention) depends on tasks 3 and 8 (needs queue store and log storage)
- Task 10 (provider abstraction) depends on tasks 3, 4, 5, and 8 (wraps all core subsystems)
- Tasks 11 and 12 (panels) depend on tasks 3, 8, and 10 (display data from queue, logs, and provider)
- Task 13 (commands) depends on tasks 3, 5, 8, 9, and 18 (commands invoke subsystem operations)
- Task 14 (APIs) depends on tasks 3, 7, 10, and 13 (API wraps provider + commands)
- Tasks 15 and 16 (config, async) can be developed in parallel with tasks 4–6 since they provide infrastructure used by all components
- Task 17 (plugin entry point) depends on ALL other tasks since it wires everything together
- Task 18 (hold/release) depends on task 3 (queue store for status transitions)
- Task 19 (integration tests) runs last as it exercises the full stack
- All property tests use the `proptest` crate with a minimum of 100 iterations
- All async tests use `#[tokio::test]` where applicable
- Physical file operations use `tempfile::TempDir` in tests to avoid polluting the real filesystem
- Mock implementations of ff-dataset-catalog, ff-dataset-allocator, and ff-plugin traits should be defined in `tests/support/` for integration tests
- The JES subsystem does NOT create a separate dataset browser panel — it links to the file-tree-panel's existing "Catalogs" node provided by ff-dataset-catalog

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Project scaffold, core types, and FFJCL parser", "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7", "1.8", "1.9", "1.10", "2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7"] },
    { "id": 1, "label": "Job queue persistence and configuration", "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7", "3.8", "3.9", "3.10", "3.11", "3.12", "15.1", "15.2", "15.3", "15.4"], "dependsOn": [0] },
    { "id": 2, "label": "Scheduler, initiator pool, and async infrastructure", "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8", "5.9", "5.10", "5.11", "5.12", "5.13", "16.1", "16.2", "16.3", "16.4", "16.5"], "dependsOn": [1] },
    { "id": 3, "label": "Execution engine, dataset bridge, and hold/release", "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "6.8", "6.9", "7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "18.1", "18.2", "18.3", "18.4", "18.5"], "dependsOn": [2] },
    { "id": 4, "label": "Log manager, SYSOUT, and retention engine", "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8", "8.9", "8.10", "9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "9.8", "9.9"], "dependsOn": [3] },
    { "id": 5, "label": "Provider abstraction and APIs", "tasks": ["10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8", "14.1", "14.2", "14.3", "14.4", "14.5", "14.6"], "dependsOn": [4] },
    { "id": 6, "label": "Panels, commands, and UI", "tasks": ["11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7", "11.8", "11.9", "11.10", "11.11", "12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7", "13.1", "13.2", "13.3", "13.4", "13.5", "13.6", "13.7", "13.8", "13.9", "13.10", "13.11", "13.12", "13.13", "13.14", "13.15"], "dependsOn": [5] },
    { "id": 7, "label": "Plugin entry point and integration tests", "tasks": ["17.1", "17.2", "17.3", "17.4", "17.5", "17.6", "17.7", "17.8", "17.9", "19.1", "19.2", "19.3", "19.4", "19.5", "19.6", "19.7", "19.8"], "dependsOn": [6] }
  ]
}
```

- [x] 20. SDSF panel framework core -- action bar, title line, SCROLL field, filter lines
  - [x] 20.1 Implement action bar with pull-down menus (File, View, Help) in JobMonitorPanel
    - Validates: Requirement 16.1
  - [x] 20.2 Implement title line with panel name and visible row range display
    - Validates: Requirement 16.2
  - [x] 20.3 Implement SCROLL ===> field adjacent to command input, retaining last-used amount
    - Validates: Requirement 16.3
  - [x] 20.4 Implement filter information lines (PREFIX=/DEST=/OWNER=) below title line
    - Validates: Requirement 16.4, 16.25
  - [x] 20.5 Implement title line message area for command feedback
    - Validates: Requirement 16.21
  - [x] 20.6 Implement COMMAND INPUT ===> field for SDSF commands
    - Validates: Requirement 16.22
  - [x] 20.7 Write unit tests for panel chrome: title line content, filter line display, SCROLL field retention, message area update
    - Validates: Requirement 16.1-16.4, 16.21-16.22

- [x] 21. NP column and action character system
  - [x] 21.1 Implement NP column as fixed leftmost column (non-scrolling) with action character input
    - Validates: Requirement 16.5
  - [x] 21.2 Implement fixed JOBNAME column during horizontal scroll
    - Validates: Requirement 16.6
  - [x] 21.3 Implement action character dispatch: S, ?, C, H, A, P, D, E, J, W
    - Validates: Requirement 16.7, 16.8, 16.23
  - [x] 21.4 Implement = repeat action character
    - Validates: Requirement 16.9
  - [x] 21.5 Implement // block action syntax (first and last row of block)
    - Validates: Requirement 16.10
  - [x] 21.6 Implement command-line action syntax ("2 C" in command field)
    - Validates: Requirement 16.11
  - [x] 21.7 Implement SET ROWNUM ON/OFF -- row numbers in NP area
    - Validates: Requirement 16.12
  - [x] 21.8 Write unit tests for NP column: action dispatch, repeat =, block //, command-line syntax, invalid action rejection, SET ROWNUM toggle
    - Validates: Requirement 16.5-16.12, 16.23

- [x] 22. Main panel (MENU command) and command groups
  - [x] 22.1 Implement MENU command navigating to main panel listing all SDSF panel commands
    - Validates: Requirement 16.13, 16.17
  - [x] 22.2 Implement command groups (Jobs, Output, JES, Log, Memory, Other) with expand/collapse
    - Validates: Requirement 16.14
  - [x] 22.3 Implement S action on main panel row to navigate to selected panel
    - Validates: Requirement 16.15
  - [x] 22.4 Implement SET MAIN GROUP command for grouped main panel display
    - Validates: Requirement 16.16
  - [x] 22.5 Write unit tests for main panel: group rendering, S action navigation, SET MAIN GROUP toggle, MENU command from sub-panel
    - Validates: Requirement 16.13-16.17

- [x] 23. PREFIX, OWNER, DEST filter commands
  - [x] 23.1 Implement PREFIX filter command -- filter job list by job name prefix; PREFIX * clears
    - Validates: Requirement 16.18
  - [x] 23.2 Implement OWNER filter command -- filter by job owner; OWNER * clears
    - Validates: Requirement 16.19
  - [x] 23.3 Implement DEST filter command -- filter by output destination; DEST * clears
    - Validates: Requirement 16.20
  - [x] 23.4 Write unit tests for filter commands: PREFIX match, OWNER match, DEST match, wildcard clear, combined filters, filter persistence across tab switch
    - Validates: Requirement 16.18-16.20, 16.25

- [x] 24. Job table column definitions and SORT command
  - [x] 24.1 Implement full column set: JOBNAME, JOBID, OWNER, STATUS, CLASS, PRTY, QUEUE, START, END, RC, STEPNAME, PROCSTEP
    - Validates: Requirement 16.24
  - [x] 24.2 Implement column hide/show and reorder support
    - Validates: Requirement 16.24
  - [x] 24.3 Implement SORT command -- SORT colname [A|D]; SORT with no args restores submission-time order
    - Validates: Requirement 16.26
  - [x] 24.4 Write unit tests for column definitions: all columns present, hide/show toggle, SORT ascending/descending, SORT reset
    - Validates: Requirement 16.24, 16.26

- [x] 25. Integration tests for SDSF panel framework
  - [x] 25.1 Write integration test: full NP column action cycle -- enter action char, verify dispatch, verify message area feedback
    - Validates: Requirement 16.7-16.8
  - [x] 25.2 Write integration test: PREFIX + OWNER + DEST combined filter -- verify only matching jobs shown
    - Validates: Requirement 16.18-16.20
  - [x] 25.3 Write integration test: SORT + filter interaction -- sort filtered result, verify order preserved after filter change
    - Validates: Requirement 16.26
  - [x] 25.4 Write integration test: MENU navigation -- MENU from input queue, S to select panel, verify navigation
    - Validates: Requirement 16.13-16.15

- [x] 26. ST panel and advanced filter/find/locate commands
  - [x] 26.1 Implement ST (Status) sub-panel showing all jobs with STATUS column
    - Validates: Requirement 17.1, 17.14
  - [x] 26.2 Implement FILTER command -- advanced filter expression with field comparisons, AND/OR, wildcard
    - Validates: Requirement 17.2, 17.12, 17.13
  - [x] 26.3 Implement FIND command -- search within panel data, FIND NEXT/PREV, case-insensitive default, FIND C for case-sensitive
    - Validates: Requirement 17.3, 17.15, 17.16
  - [x] 26.4 Implement LOCATE command -- scroll to first JOBNAME match, nearest alphabetic on no match
    - Validates: Requirement 17.4, 17.16
  - [x] 26.5 Write unit tests for ST panel, FILTER expression parsing (operators, AND/OR, wildcard), FIND (next/prev/case), LOCATE (match/no-match)
    - Validates: Requirement 17.1-17.4, 17.12-17.16

- [x] 27. SDSF scroll commands
  - [x] 27.1 Implement UP/DOWN/LEFT/RIGHT scroll commands with n/HALF/PAGE/MAX amounts
    - Validates: Requirement 17.5
  - [x] 27.2 Implement scroll amount defaulting from SCROLL ===> field; update SCROLL field after scroll
    - Validates: Requirement 17.5, 17.17
  - [x] 27.3 Write unit tests for scroll commands: each direction, each amount keyword, SCROLL field sync
    - Validates: Requirement 17.5, 17.17

- [x] 28. SET ACTION/MAIN/ROWNUM commands, WHO, QUERY AUTH
  - [x] 28.1 Implement SET ACTION -- display valid action characters with descriptions
    - Validates: Requirement 17.6
  - [x] 28.2 Implement SET MAIN [panel-name] -- set default MENU panel
    - Validates: Requirement 17.7
  - [x] 28.3 Implement SET ROWNUM ON/OFF -- toggle row numbers in NP area (extends Requirement 16.12)
    - Validates: Requirement 17.8
  - [x] 28.4 Implement WHO command -- session information summary
    - Validates: Requirement 17.9
  - [x] 28.5 Implement QUERY AUTH command -- display authorised commands and action characters
    - Validates: Requirement 17.10
  - [x] 28.6 Write unit tests for SET ACTION display, SET MAIN default, SET ROWNUM toggle, WHO output fields, QUERY AUTH list
    - Validates: Requirement 17.6-17.10

- [x] 29. SET settings persistence and integration tests
  - [x] 29.1 Implement persistence of SET ACTION preference, SET MAIN default, SET ROWNUM state via session mechanism
    - Validates: Requirement 17.11
  - [x] 29.2 Write unit tests for SET settings round-trip through session persistence
    - Validates: Requirement 17.11
  - [x] 29.3 Write integration test: FILTER + FIND interaction -- apply FILTER, then FIND within filtered result, verify scope
    - Validates: Requirement 17.2, 17.3
  - [x] 29.4 Write integration test: SET MAIN + MENU -- set default panel, restart session, verify MENU opens correct panel
    - Validates: Requirement 17.7, 17.11
  - [x] 29.5 Write integration test: scroll commands -- UP/DOWN/LEFT/RIGHT with all amount keywords, verify SCROLL field updates
    - Validates: Requirement 17.5, 17.17

- [x] 30. Overtype fields
  - [x] 30.1 Implement visual distinction for overtypeable fields (theme colour or underline style)
  - [x] 30.2 Implement direct overtype: user types new value over field, Enter applies change and refreshes panel
  - [x] 30.3 Implement command-line overtype syntax: `<field-name> <value>` updates named field for cursor/NP row
  - [x] 30.4 Implement Overtype Extension pop-up for values exceeding column width
  - [x] 30.5 Write unit tests for overtype visual flag, direct overtype apply, command-line overtype, extension pop-up trigger
  - Covers: Requirement 18 (AC 18.1, 18.2, 18.3, 18.4)

- [x] 31. Help system (HELP, ACTH, COLH, CMDH, SEARCH)
  - [x] 31.1 Implement context-sensitive HELP command / PF1: display panel help with purpose, commands, and column definitions
  - [x] 31.2 Implement ACTH command: list valid action characters with descriptions for current panel
  - [x] 31.3 Implement COLH command: list column names with data type, width, and description
  - [x] 31.4 Implement CMDH command: list valid primary commands with syntax and description
  - [x] 31.5 Implement SEARCH <text> within help panel: scroll to first match
  - [x] 31.6 Write unit tests for HELP panel content, ACTH/COLH/CMDH lists, SEARCH match and no-match
  - Covers: Requirement 18 (AC 18.5, 18.6, 18.7, 18.8, 18.9)

- [x] 32. Log panels (LOG, ULOG, NEXT, PREV, SNAPSHOT) and system panels (SYS, DASH, INIT, JC, SP)
  - [x] 32.1 Implement LOG command: open System Log panel in reverse-chronological order
  - [x] 32.2 Implement ULOG command: open User Log panel for current user
  - [x] 32.3 Implement NEXT/PREV commands in log panels: scroll forward/backward through log segments
  - [x] 32.4 Implement SNAPSHOT command: capture current log content to dataset or file
  - [x] 32.5 Implement SYS panel: active address spaces with status and resource consumption
  - [x] 32.6 Implement DASH panel: system health metrics summary (CPU, memory, I/O rates)
  - [x] 32.7 Implement INIT panel: initiator pool status (class assignments, active/idle state)
  - [x] 32.8 Implement JC panel: job class definitions and scheduling parameters
  - [x] 32.9 Implement SP panel: spool volume utilisation and track allocation
  - [x] 32.10 Write unit tests for LOG/ULOG open, NEXT/PREV navigation, SNAPSHOT output, and each system panel data model
  - Covers: Requirement 18 (AC 18.10, 18.11, 18.12, 18.13, 18.14, 18.15, 18.16, 18.17, 18.18)

- [x] 33. Browse and print (browse settings, PRINT action, COLS command)
  - [x] 33.1 Implement browse settings: line width, record format display, FIND within output stream
  - [x] 33.2 Implement PRINT action character: route job output dataset to configured print destination
  - [x] 33.3 Implement COLS command in browse: display column ruler showing horizontal scroll position and column numbers
  - [x] 33.4 Write unit tests for browse settings persistence, PRINT routing, COLS ruler display
  - Covers: Requirement 18 (AC 18.19, 18.20, 18.21)

- [x] 34. SET P2 commands and persistence
  - [x] 34.1 Implement SET BCOLOR <color>: set panel background colour, persist across sessions
  - [x] 34.2 Implement SET CONFIRM ON/OFF: control confirmation prompt for destructive actions
  - [x] 34.3 Implement SET CURSOR <field>: set default cursor landing position on panel open
  - [x] 34.4 Implement SET DATE <format>: set date display format (MDY, DMY, YMD, JUL) for date columns
  - [x] 34.5 Implement SET DELAY <seconds>: set automatic refresh interval; 0 disables auto-refresh
  - [x] 34.6 Implement SET HEX ON/OFF: toggle hexadecimal display of field values
  - [x] 34.7 Implement SET SCHARS <chars>: define special characters for field delimiters
  - [x] 34.8 Implement SET SCREEN <rows> <cols>: set logical screen dimensions for panel layout
  - [x] 34.9 Implement SET P2 persistence: persist all SET P2 settings via session mechanism (extends Task 29.1)
  - [x] 34.10 Write unit tests for each SET P2 command, default values, and round-trip persistence
  - Covers: Requirement 18 (AC 18.22, 18.23, 18.24, 18.25, 18.26, 18.27, 18.28, 18.29, 18.30)
