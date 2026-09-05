# Design Document: Job Entry Subsystem (`ff-jes`)

## Overview

The `ff-jes` crate is the **mainframe JES/SDSF-style batch processing emulator** for FileForgeWorkbench. It provides a complete job lifecycle — submission, queue management, priority-based scheduling, initiator-pool execution, real-time monitoring, log viewing, dataset allocation integration, retained output management, and purge — all delivered as a workbench plugin.

### Purpose

- Emulate mainframe batch processing locally on Windows, Linux, and macOS
- Submit job definitions (FFJCL) to a persistent queue for scheduled execution
- Dispatch queued jobs to a configurable pool of async initiators (Tokio tasks)
- Provide an SDSF-style Job Monitor panel with filtered views per lifecycle state
- Stream live job logs and SYSOUT to a dockable log viewer panel
- Integrate with the dataset catalog for DSN resolution and allocation during execution
- Retain job output according to configurable retention policies with manual/auto purge
- Expose a `JobProvider` trait for future extensibility to remote JES environments
- Expose Job API and Dataset API for programmatic access by other plugins and Lua macros

### Position in Architecture

```
Wave 13.5 — Job Entry Subsystem (depends on Wave 2 Platform + Wave 13 Dataset)

┌─────────────────────────────────────────────────────────────────┐
│                    Application Binary (ffwb)                      │
│              (ff-desktop / GUI shell)                             │
├─────────────────────────────────────────────────────────────────┤
│  JobMonitorPanel │ JobLogViewerPanel (DockablePanel impls)        │
├─────────────────────────────────────────────────────────────────┤
│               ff-jes (THIS CRATE) — Wave 13.5                    │
│  Plugin, Scheduler, Initiators, Queue, Monitor, Provider         │
├─────────────────────────────────────────────────────────────────┤
│  ff-plugin │ ff-command │ ff-layout │ ff-workflow │ ff-vfs       │
│  ff-dataset-allocator │ ff-dataset-catalog │ ff-config           │
│  ff-logging                                                      │
│         (Waves 0–2 Platform + Wave 13 Dataset)                   │
└─────────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **FFW-ARCH-001 (Req 1)**: All file I/O (job logs, SYSOUT, spool) flows through VFS — no direct `std::fs` in consuming code
- **GUI Independence (Req 2)**: Core JES logic (scheduler, queue, initiators) has zero GUI dependencies; panels use `egui` only via `DockablePanel::render`
- **Plugin Architecture (Req 3)**: Implements `FileForgePlugin` trait; registers panels, commands, and APIs via `PluginContext`
- **Command-Driven (Req 4)**: All JES operations registered as commands under `jes.*` namespace via `ff-command`
- **Async I/O (Req 6)**: Initiators run jobs on Tokio tasks; scheduler is an async background task; no UI blocking
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-jes`
- **Error Message Standards (Req 8)**: Errors follow `[jes] operation: description` format
- **Provider Abstraction**: `JobProvider` trait decouples monitor from execution backend
- **Dataset Resolution via Allocator**: DSN resolution delegates to `ff-dataset-allocator` (which delegates to `ff-dataset-catalog`)
- **Persistence**: Queue state and retained output survive application restarts via local database/spool

### Upstream Dependencies

| Crate | Relationship |
|-------|-------------|
| `ff-plugin` | Implements `FileForgePlugin` trait; uses `PluginContext` for registration |
| `ff-command` | Registers all `jes.*` commands with the command registry |
| `ff-layout` | Panels implement `DockablePanel` trait; registers with Panel_Registry |
| `ff-workflow` | Job execution modelled as state-machine workflows |
| `ff-vfs` | Job logs and SYSOUT accessible via VFS Resource_URIs |
| `ff-dataset-allocator` | DSN resolution, DISP handling, GDG generation resolution for DD statements |
| `ff-dataset-catalog` | Indirect — catalog queries flow through `ff-dataset-allocator` |
| `ff-config` | Reads `[plugins.ffw-jes]` configuration namespace |
| `ff-logging` | Structured log records for all JES operations |

### Downstream Consumers

| Crate | Relationship |
|-------|-------------|
| `ff-desktop` | Renders JobMonitorPanel and JobLogViewerPanel |
| `ff-lua-macro-engine` | Lua macros invoke Job API via command bridge |
| Other plugins | Job API and Dataset API accessible via `PluginContext` |

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Shell [Shell Layer]
        DESKTOP[ff-desktop — egui GUI shell]
    end

    subgraph Panels [DockablePanel Implementations]
        MONITOR[JobMonitorPanel<br/>SDSF-style tabbed monitor]
        LOGVIEW[JobLogViewerPanel<br/>streaming log viewer]
    end

    subgraph ff-jes [ff-jes Crate]
        PLUGIN[JesPlugin<br/>FileForgePlugin impl]
        SCHED[Scheduler<br/>priority-based dispatch]
        POOL[InitiatorPool<br/>async worker management]
        QUEUE[JobQueue<br/>persistent job storage]
        EXEC[JobExecutor<br/>step execution engine]
        PROV[JobProvider trait<br/>provider abstraction]
        DPROV[DesktopJesProvider<br/>local implementation]
        RETAIN[RetentionManager<br/>purge scheduling]
        SPOOL[SpoolManager<br/>log/SYSOUT storage]
        JOBAPI[JobApi<br/>programmatic interface]
        DSAPI[DatasetApi<br/>dataset operations]
        CMD[CommandRegistrar<br/>jes.* commands]
        PARSER[FfjclParser<br/>job definition parsing]
    end

    subgraph Upstream [Upstream Crates]
        PLUG[ff-plugin — FileForgePlugin trait]
        COMMAND[ff-command — CommandRegistry]
        LAYOUT[ff-layout — DockablePanel trait]
        WORKFLOW[ff-workflow — WorkflowRunner]
        VFS[ff-vfs — VfsProvider]
        ALLOC[ff-dataset-allocator — DD resolution]
        CONFIG[ff-config — configuration]
        LOG[ff-logging — diagnostics]
    end

    DESKTOP -->|renders| MONITOR
    DESKTOP -->|renders| LOGVIEW
    MONITOR -->|queries| DPROV
    LOGVIEW -->|streams| SPOOL

    PLUGIN -->|registers| CMD
    PLUGIN -->|registers| MONITOR
    PLUGIN -->|registers| LOGVIEW
    PLUGIN -->|starts| SCHED
    PLUGIN -->|starts| POOL
    PLUGIN -->|starts| RETAIN

    SCHED -->|dequeues from| QUEUE
    SCHED -->|dispatches to| POOL
    POOL -->|executes via| EXEC
    EXEC -->|writes to| SPOOL
    EXEC -->|resolves DSN via| ALLOC
    EXEC -->|runs as| WORKFLOW

    DPROV -->|implements| PROV
    DPROV -->|delegates to| QUEUE
    DPROV -->|delegates to| POOL
    DPROV -->|delegates to| SPOOL

    JOBAPI -->|uses| DPROV
    DSAPI -->|uses| ALLOC

    CMD -->|registers with| COMMAND
    PLUGIN -->|implements| PLUG
    MONITOR -->|implements| LAYOUT
    LOGVIEW -->|implements| LAYOUT
    SPOOL -->|stores via| VFS
    PLUGIN -->|reads| CONFIG
    PLUGIN -->|logs via| LOG
end
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **JesPlugin** | `FileForgePlugin` implementation; bootstraps all subsystems, manages lifecycle |
| **Scheduler** | Async background task; selects eligible jobs, dispatches to available initiators |
| **InitiatorPool** | Manages a configurable set of async workers; tracks status per initiator |
| **JobQueue** | Persistent storage of job records; supports query, filter, priority ordering |
| **JobExecutor** | Runs a single job's steps via `ff-workflow`; manages process lifecycle |
| **DesktopJesProvider** | `JobProvider` implementation for local execution; orchestrates queue + pool + spool |
| **SpoolManager** | Stores job logs and SYSOUT; provides streaming access; VFS-backed |
| **RetentionManager** | Background task enforcing retention policies; auto-purge on schedule |
| **FfjclParser** | Parses and validates FFJCL job definitions before queue insertion |
| **JobApi** | Programmatic interface for job operations (submit, hold, release, cancel, query) |
| **DatasetApi** | Programmatic interface for dataset operations (delegates to `ff-dataset-allocator`) |
| **CommandRegistrar** | Registers all `jes.*` commands with metadata, enabled predicates, shortcuts |
| **JobMonitorPanel** | SDSF-style DockablePanel with tabbed sub-panels per job state |
| **JobLogViewerPanel** | DockablePanel for viewing/streaming job logs with search and export |

### Job Lifecycle State Machine

```
┌──────────┐   submit    ┌────────┐   hold    ┌──────┐
│(external)│────────────▶│ QUEUED │──────────▶│ HELD │
└──────────┘             └────────┘           └──────┘
                              │                    │
                      dispatch│            release │
                              ▼                    │
                         ┌────────┐◀───────────────┘
                         │ ACTIVE │
                         └────────┘
                          │  │  │
               ┌──────────┘  │  └──────────────┐
               ▼             ▼                  ▼
         ┌───────────┐  ┌────────┐       ┌───────────┐
         │ COMPLETED │  │ FAILED │       │ CANCELLED │
         └───────────┘  └────────┘       └───────────┘
                              │
              All terminal states → retained in spool → purge
```

### Scheduler Dispatch Flow

```
loop (async, every scheduler_poll_ms):
    1. Query JobQueue for eligible jobs (status=QUEUED, preconditions met, ordered by priority then submit time)
    2. Query InitiatorPool for available initiators (status=IDLE)
    3. For each (job, initiator) pair:
        a. Transition job status: QUEUED → ACTIVE
        b. Assign initiator ID to job record
        c. Record start timestamp
        d. Dispatch job to initiator via JobExecutor
    4. Sleep scheduler_poll_ms
```

---

## Components and Interfaces

```
crates/ff-jes/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports, crate docs
│   ├── plugin.rs                   # JesPlugin: FileForgePlugin implementation
│   ├── provider/
│   │   ├── mod.rs                  # Re-exports for provider module
│   │   ├── traits.rs              # JobProvider trait definition
│   │   └── desktop.rs             # DesktopJesProvider: local implementation
│   ├── queue/
│   │   ├── mod.rs                  # Re-exports for queue module
│   │   ├── job_queue.rs           # JobQueue: persistent job storage
│   │   ├── persistence.rs         # SQLite-backed queue persistence
│   │   └── filter.rs             # JobFilter: query predicates for queue views
│   ├── scheduler/
│   │   ├── mod.rs                  # Re-exports for scheduler module
│   │   ├── scheduler.rs           # Scheduler: async dispatch loop
│   │   └── strategy.rs           # SchedulingStrategy: FIFO, Priority
│   ├── initiator/
│   │   ├── mod.rs                  # Re-exports for initiator module
│   │   ├── pool.rs               # InitiatorPool: worker lifecycle management
│   │   ├── initiator.rs          # Initiator: single worker state machine
│   │   └── executor.rs           # JobExecutor: runs job steps via ff-workflow
│   ├── spool/
│   │   ├── mod.rs                  # Re-exports for spool module
│   │   ├── spool_manager.rs      # SpoolManager: log/SYSOUT storage
│   │   ├── job_log.rs            # JobLog: structured log assembly
│   │   └── retention.rs          # RetentionManager: auto-purge logic
│   ├── parser/
│   │   ├── mod.rs                  # Re-exports for parser module
│   │   ├── ffjcl.rs              # FfjclParser: job definition parsing
│   │   └── validation.rs         # FFJCL validation rules
│   ├── monitor/
│   │   ├── mod.rs                  # Re-exports for monitor module
│   │   ├── job_monitor_panel.rs   # JobMonitorPanel: DockablePanel implementation
│   │   └── job_log_viewer.rs     # JobLogViewerPanel: DockablePanel implementation
│   ├── api/
│   │   ├── mod.rs                  # Re-exports for API module
│   │   ├── job_api.rs            # JobApi: programmatic job operations
│   │   └── dataset_api.rs        # DatasetApi: programmatic dataset operations
│   ├── commands.rs                 # CommandRegistrar: all jes.* command registrations
│   ├── models.rs                   # Core data types (Job, JobId, JobStatus, etc.)
│   ├── config.rs                   # Configuration reading ([plugins.ffw-jes])
│   ├── events.rs                   # JES event types for status change notifications
│   └── error.rs                    # JesError enum
└── tests/
    ├── plugin_lifecycle_tests.rs   # Plugin init/activate/deactivate/shutdown tests
    ├── queue_tests.rs              # Job queue persistence and ordering tests
    ├── scheduler_tests.rs          # Scheduler dispatch logic property tests
    ├── initiator_tests.rs          # Initiator pool management property tests
    ├── executor_tests.rs           # Job execution lifecycle tests
    ├── spool_tests.rs              # Spool storage and retrieval tests
    ├── retention_tests.rs          # Retention policy enforcement property tests
    ├── parser_tests.rs             # FFJCL parsing property tests
    ├── provider_tests.rs           # JobProvider trait compliance tests
    ├── monitor_tests.rs            # Job Monitor state and filtering tests
    ├── api_tests.rs                # Job API and Dataset API tests
    ├── command_tests.rs            # Command registration and predicate tests
    └── integration.rs              # End-to-end job submit → execute → complete tests
```

---

## Data Models

### JobId

```rust
/// A unique, monotonically increasing job identifier.
/// Never reused within the same workbench session.
/// Format: JOB00001, JOB00002, ... (display) backed by u64 internally.
///
/// Addresses: Requirement 2 AC 2.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JobId(u64);

impl JobId {
    /// Create a new JobId from a raw numeric value.
    pub fn new(value: u64) -> Self;

    /// Get the raw numeric value.
    pub fn value(&self) -> u64;
}

impl Display for JobId {
    /// Formats as "JOB{:05}" (e.g., "JOB00001").
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}
```

### JobStatus

```rust
/// The lifecycle state of a job.
///
/// Addresses: Requirement 3 AC 3.10, Requirement 6
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum JobStatus {
    /// Job is in the input queue awaiting dispatch.
    Queued,
    /// Job is held — not eligible for scheduling.
    Held,
    /// Job is currently executing on an initiator.
    Active,
    /// Job completed successfully.
    Completed,
    /// Job terminated abnormally.
    Failed,
    /// Job was cancelled by user before or during execution.
    Cancelled,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}
```

### Job

```rust
/// A complete job record with all lifecycle metadata.
///
/// Addresses: Requirements 2, 3, 5, 6
#[derive(Debug, Clone)]
pub struct Job {
    /// Unique job identifier.
    pub id: JobId,
    /// Job name from the FFJCL JOB statement.
    pub name: String,
    /// Current lifecycle status.
    pub status: JobStatus,
    /// Priority (higher = dispatched sooner). Default: 0.
    pub priority: i32,
    /// Owner/submitter identity.
    pub owner: String,
    /// Submission timestamp (UTC).
    pub submit_time: DateTime<Utc>,
    /// Start timestamp (set when ACTIVE).
    pub start_time: Option<DateTime<Utc>>,
    /// End timestamp (set on terminal status).
    pub end_time: Option<DateTime<Utc>>,
    /// Assigned initiator ID (set when ACTIVE).
    pub initiator_id: Option<InitiatorId>,
    /// Current step name (updated during execution).
    pub current_step: Option<String>,
    /// Final return code (set on COMPLETED).
    pub return_code: Option<i32>,
    /// Failure reason (set on FAILED).
    pub failure_reason: Option<String>,
    /// Failing step name (set on FAILED).
    pub failing_step: Option<String>,
    /// Cancellation requester (set on CANCELLED).
    pub cancelled_by: Option<String>,
    /// Cancellation timestamp.
    pub cancel_time: Option<DateTime<Utc>>,
    /// Source provider identifier.
    pub provider_id: String,
    /// The parsed FFJCL job definition.
    pub definition: FfjclJob,
    /// Process ID of the executing process (if available).
    pub process_id: Option<u32>,
}

impl Job {
    /// Calculate elapsed runtime from start_time to end_time (or now if active).
    pub fn elapsed(&self) -> Option<Duration>;

    /// Returns true if the job is in a terminal state.
    pub fn is_terminal(&self) -> bool;

    /// Returns true if the job is eligible for scheduling.
    pub fn is_eligible(&self) -> bool;
}
```

### InitiatorId

```rust
/// Unique identifier for an initiator in the pool.
///
/// Addresses: Requirement 4 AC 4.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InitiatorId(u32);

impl InitiatorId {
    pub fn new(value: u32) -> Self;
    pub fn value(&self) -> u32;
}

impl Display for InitiatorId {
    /// Formats as "INIT{:02}" (e.g., "INIT01").
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}
```

### InitiatorStatus

```rust
/// The lifecycle state of an initiator worker.
///
/// Addresses: Requirement 4 AC 4.3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InitiatorStatus {
    /// Initiator is idle and available for work.
    Idle,
    /// Initiator is starting up.
    Starting,
    /// Initiator is executing a job.
    Active,
    /// Initiator is draining — finishing current job but accepting no new work.
    Draining,
    /// Initiator is shutting down.
    Stopping,
    /// Initiator has been stopped (inactive).
    Stopped,
    /// Initiator encountered an unrecoverable error.
    Failed,
}
```

### Initiator

```rust
/// An individual initiator (worker) in the pool.
///
/// Addresses: Requirement 4
#[derive(Debug)]
pub struct Initiator {
    /// Unique initiator identifier.
    pub id: InitiatorId,
    /// Current status.
    pub status: InitiatorStatus,
    /// Currently assigned job (if Active or Draining).
    pub current_job: Option<JobId>,
    /// Number of jobs completed by this initiator.
    pub jobs_completed: u64,
    /// Last error message (if Failed).
    pub last_error: Option<String>,
}
```

### InitiatorPool

```rust
/// Manages a configurable pool of async initiator workers.
///
/// Addresses: Requirement 4, Requirement 15
pub struct InitiatorPool {
    /// The set of managed initiators.
    initiators: Vec<Initiator>,
    /// Configured capacity (from config).
    capacity: usize,
    /// Tokio runtime handle for spawning workers.
    runtime: tokio::runtime::Handle,
}

impl InitiatorPool {
    /// Create a new pool with the specified capacity.
    pub fn new(capacity: usize, runtime: tokio::runtime::Handle) -> Self;

    /// Start all initiators in the pool.
    /// Addresses: Requirement 1 AC 1.4
    pub async fn start_all(&mut self) -> Result<(), JesError>;

    /// Stop all initiators gracefully (allow active jobs to complete).
    /// Addresses: Requirement 1 AC 1.5
    pub async fn stop_all(&mut self) -> Result<(), JesError>;

    /// Start a specific initiator by ID.
    /// Addresses: Requirement 4 AC 4.4
    pub async fn start_initiator(&mut self, id: InitiatorId) -> Result<(), JesError>;

    /// Stop a specific initiator (current job completes first).
    /// Addresses: Requirement 4 AC 4.5
    pub async fn stop_initiator(&mut self, id: InitiatorId) -> Result<(), JesError>;

    /// Drain a specific initiator (finish current job, accept no new work).
    /// Addresses: Requirement 4 AC 4.6
    pub async fn drain_initiator(&mut self, id: InitiatorId) -> Result<(), JesError>;

    /// Get all available (idle) initiators.
    pub fn available(&self) -> Vec<InitiatorId>;

    /// Get the status of all initiators.
    pub fn status_all(&self) -> Vec<&Initiator>;

    /// Dispatch a job to a specific idle initiator.
    pub async fn dispatch(&mut self, id: InitiatorId, job: Job) -> Result<(), JesError>;
}
```

### Scheduler

```rust
/// The async job scheduler that dispatches eligible jobs to available initiators.
///
/// Addresses: Requirement 3
pub struct Scheduler {
    /// Reference to the job queue.
    queue: Arc<JobQueue>,
    /// Reference to the initiator pool.
    pool: Arc<Mutex<InitiatorPool>>,
    /// Scheduling strategy.
    strategy: SchedulingStrategy,
    /// Poll interval in milliseconds.
    poll_interval_ms: u64,
    /// Cancellation token for graceful shutdown.
    cancel_token: CancellationToken,
}

impl Scheduler {
    /// Create a new scheduler.
    pub fn new(
        queue: Arc<JobQueue>,
        pool: Arc<Mutex<InitiatorPool>>,
        strategy: SchedulingStrategy,
        poll_interval_ms: u64,
    ) -> Self;

    /// Start the scheduler dispatch loop as an async background task.
    /// Addresses: Requirement 15 AC 15.2
    pub async fn run(&self) -> Result<(), JesError>;

    /// Request graceful shutdown of the scheduler.
    pub fn shutdown(&self);
}

/// Scheduling strategy for job dispatch.
///
/// Addresses: Requirement 3 AC 3.1, 3.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingStrategy {
    /// First-in-first-out by submission time (default).
    Fifo,
    /// Higher-priority jobs dispatched first, then FIFO within same priority.
    Priority,
}
```

### JobQueue

```rust
/// Persistent job queue with query, filter, and ordering support.
///
/// Addresses: Requirements 2, 3
pub struct JobQueue {
    /// SQLite-backed persistent storage.
    db: SqliteJobStore,
    /// In-memory index for fast status queries.
    index: RwLock<JobIndex>,
    /// Next job ID counter (monotonically increasing).
    next_id: AtomicU64,
    /// Event sender for job state changes.
    event_tx: broadcast::Sender<JobEvent>,
}

impl JobQueue {
    /// Create or open a persistent job queue at the given path.
    pub async fn open(db_path: &Path) -> Result<Self, JesError>;

    /// Submit a new job to the queue.
    /// Addresses: Requirement 2 AC 2.1–2.5
    pub async fn submit(&self, definition: FfjclJob, owner: &str) -> Result<JobId, JesError>;

    /// Get a job by ID.
    pub async fn get(&self, id: JobId) -> Result<Option<Job>, JesError>;

    /// Query jobs by filter criteria.
    /// Addresses: Requirement 9 AC 9.4
    pub async fn query(&self, filter: &JobFilter) -> Result<Vec<Job>, JesError>;

    /// Update job status with associated metadata.
    pub async fn update_status(&self, id: JobId, update: JobStatusUpdate) -> Result<(), JesError>;

    /// Get eligible jobs for scheduling (QUEUED, preconditions met, ordered by strategy).
    /// Addresses: Requirement 3 AC 3.3–3.5
    pub async fn eligible_jobs(&self, strategy: SchedulingStrategy) -> Result<Vec<Job>, JesError>;

    /// Subscribe to job state change events.
    /// Addresses: Requirement 12 AC 12.4
    pub fn subscribe(&self) -> broadcast::Receiver<JobEvent>;

    /// Get job counts by status (for tab headers).
    /// Addresses: Requirement 9 AC 9.2
    pub async fn counts_by_status(&self) -> Result<HashMap<JobStatus, usize>, JesError>;

    /// Purge a job (remove from queue and spool).
    /// Addresses: Requirement 8 AC 8.2
    pub async fn purge(&self, id: JobId) -> Result<(), JesError>;

    /// Persist current queue state for restart recovery.
    /// Addresses: Requirement 2 AC 2.6
    pub async fn persist(&self) -> Result<(), JesError>;
}
```

### JobFilter

```rust
/// Query predicates for filtering jobs in the queue/monitor.
///
/// Addresses: Requirement 9 AC 9.4, 9.5
#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    /// Filter by owner/user.
    pub owner: Option<String>,
    /// Filter by job name (prefix match).
    pub name: Option<String>,
    /// Filter by job ID.
    pub id: Option<JobId>,
    /// Filter by status (multiple allowed).
    pub statuses: Option<Vec<JobStatus>>,
    /// Filter by submit date range.
    pub submit_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Filter by start date range.
    pub start_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Filter by end date range.
    pub end_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Filter by return code.
    pub return_code: Option<i32>,
    /// Filter by provider.
    pub provider_id: Option<String>,
    /// Sort order.
    pub sort: Option<JobSortField>,
    /// Sort direction.
    pub ascending: bool,
}

/// Fields by which jobs can be sorted in the monitor.
///
/// Addresses: Requirement 3 AC 3.8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobSortField {
    Name,
    Id,
    Owner,
    SubmitTime,
    StartTime,
    EndTime,
    Priority,
    Status,
    ReturnCode,
}
```

### JobStatusUpdate

```rust
/// A status transition update for a job record.
///
/// Addresses: Requirements 3, 6, 10
#[derive(Debug, Clone)]
pub enum JobStatusUpdate {
    /// Job dispatched to an initiator.
    Dispatched {
        initiator_id: InitiatorId,
        start_time: DateTime<Utc>,
    },
    /// Job step progress update.
    StepProgress {
        step_name: String,
        process_id: Option<u32>,
    },
    /// Job completed successfully.
    Completed {
        end_time: DateTime<Utc>,
        return_code: i32,
    },
    /// Job failed.
    Failed {
        end_time: DateTime<Utc>,
        reason: String,
        failing_step: Option<String>,
    },
    /// Job cancelled.
    Cancelled {
        cancel_time: DateTime<Utc>,
        cancelled_by: String,
    },
    /// Job held.
    Held,
    /// Job released from hold.
    Released,
}
```

### JobEvent

```rust
/// Events emitted when job state changes, consumed by the monitor and API subscribers.
///
/// Addresses: Requirement 12 AC 12.4, Requirement 15 AC 15.5
#[derive(Debug, Clone)]
pub struct JobEvent {
    /// The job that changed.
    pub job_id: JobId,
    /// The new status.
    pub new_status: JobStatus,
    /// Previous status (for transition tracking).
    pub previous_status: Option<JobStatus>,
    /// Timestamp of the event.
    pub timestamp: DateTime<Utc>,
    /// Provider that owns this job.
    pub provider_id: String,
}
```

### FfjclJob

```rust
/// A parsed FFJCL job definition ready for execution.
///
/// Addresses: Requirement 2 AC 2.1
#[derive(Debug, Clone)]
pub struct FfjclJob {
    /// Job name from the JOB statement.
    pub name: String,
    /// Job class (optional, for future priority mapping).
    pub class: Option<char>,
    /// Priority override (from JCL or default).
    pub priority: Option<i32>,
    /// Execution steps in order.
    pub steps: Vec<FfjclStep>,
    /// Job-level DD statements (e.g., JOBLIB).
    pub job_dds: Vec<FfjclDd>,
    /// Raw source text (for log display).
    pub source: String,
}

/// A single execution step within an FFJCL job.
#[derive(Debug, Clone)]
pub struct FfjclStep {
    /// Step name.
    pub name: String,
    /// Program or script to execute.
    pub exec: StepExecTarget,
    /// DD statements for this step.
    pub dds: Vec<FfjclDd>,
    /// Condition code checking (COND parameter equivalent).
    pub cond: Option<StepCondition>,
}

/// What an FFJCL step executes.
#[derive(Debug, Clone)]
pub enum StepExecTarget {
    /// Execute a native program or script.
    Program { path: String, args: Vec<String> },
    /// Execute a Lua macro.
    Macro { name: String, args: HashMap<String, String> },
    /// Execute a workbench command.
    Command { id: String, params: HashMap<String, String> },
}

/// A DD statement within an FFJCL step.
#[derive(Debug, Clone)]
pub struct FfjclDd {
    /// DD name.
    pub ddname: String,
    /// Dataset name reference.
    pub dsn: Option<String>,
    /// Disposition (maps to ff-dataset-allocator DISP semantics).
    pub disp: Option<String>,
    /// Whether this is SYSOUT.
    pub sysout: Option<char>,
    /// Whether this is DUMMY.
    pub dummy: bool,
    /// Inline data content.
    pub inline_data: Option<String>,
}

/// Condition code check for step execution (like COND= on EXEC).
#[derive(Debug, Clone)]
pub struct StepCondition {
    /// Conditions: (code, operator) pairs — if ANY is true, step is bypassed.
    pub conditions: Vec<(i32, CondOperator)>,
}

/// Comparison operators for COND parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CondOperator {
    Gt,
    Ge,
    Eq,
    Lt,
    Le,
    Ne,
}
```

### JobLog

```rust
/// Complete execution log for a job, assembled from multiple sources.
///
/// Addresses: Requirement 7
#[derive(Debug, Clone)]
pub struct JobLog {
    /// The job this log belongs to.
    pub job_id: JobId,
    /// JES-style scheduling messages (queue placement, dispatch, etc.).
    pub jes_messages: Vec<LogEntry>,
    /// Allocation messages (DSN resolution per DD).
    pub allocation_messages: Vec<LogEntry>,
    /// Per-step execution logs.
    pub step_logs: Vec<StepLog>,
    /// Final JES completion messages.
    pub completion_messages: Vec<LogEntry>,
}

/// Log for a single step's execution.
#[derive(Debug, Clone)]
pub struct StepLog {
    /// Step name.
    pub step_name: String,
    /// Standard output (SYSOUT).
    pub sysout: Vec<LogEntry>,
    /// Error output.
    pub syserr: Vec<LogEntry>,
    /// Step return code.
    pub return_code: Option<i32>,
    /// Step start time.
    pub start_time: Option<DateTime<Utc>>,
    /// Step end time.
    pub end_time: Option<DateTime<Utc>>,
}

/// A single log entry with timestamp and content.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp of the log line.
    pub timestamp: DateTime<Utc>,
    /// Log level/category.
    pub level: LogLevel,
    /// The log message text.
    pub message: String,
}

/// Log entry classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Informational JES message.
    Info,
    /// Warning (non-fatal).
    Warning,
    /// Error (fatal to step or job).
    Error,
    /// Application output (SYSOUT).
    Output,
    /// Allocation/resolution message.
    Allocation,
}
```

### RetentionPolicy

```rust
/// Configurable rules for job output retention and purge.
///
/// Addresses: Requirement 8
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// Maximum days to retain completed job output (default: 7).
    pub retention_days: u32,
    /// Maximum number of retained jobs (default: 1000).
    pub max_jobs: usize,
    /// Auto-purge check interval in seconds (default: 3600).
    pub purge_interval_secs: u64,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            retention_days: 7,
            max_jobs: 1000,
            purge_interval_secs: 3600,
        }
    }
}
```

### SpoolManager

```rust
/// Manages persistent storage of job logs and SYSOUT output.
/// Stores data via VFS and provides streaming access for live log viewing.
///
/// Addresses: Requirements 7, 8
pub struct SpoolManager {
    /// Base VFS path for spool storage.
    spool_root: String,
    /// Retention policy configuration.
    retention_policy: RetentionPolicy,
}

impl SpoolManager {
    /// Create a new spool manager.
    pub fn new(spool_root: String, retention_policy: RetentionPolicy) -> Self;

    /// Write a log entry to the spool for a job.
    pub async fn write_log(&self, job_id: JobId, entry: LogEntry) -> Result<(), JesError>;

    /// Get the complete job log.
    /// Addresses: Requirement 7 AC 7.1, 7.2
    pub async fn get_job_log(&self, job_id: JobId) -> Result<JobLog, JesError>;

    /// Stream live log output for an active job.
    /// Addresses: Requirement 5 AC 5.4, Requirement 15 AC 15.3
    pub fn stream_log(&self, job_id: JobId) -> broadcast::Receiver<LogEntry>;

    /// Purge all output for a specific job.
    /// Addresses: Requirement 8 AC 8.2, 8.4
    pub async fn purge_job(&self, job_id: JobId) -> Result<(), JesError>;

    /// Run auto-purge: remove jobs exceeding retention policy.
    /// Addresses: Requirement 8 AC 8.3
    pub async fn auto_purge(&self, queue: &JobQueue) -> Result<usize, JesError>;

    /// Get the VFS URI for a job's spool directory.
    pub fn job_uri(&self, job_id: JobId) -> String;
}
```

### JesConfig

```rust
/// Configuration for the JES plugin, read from [plugins.ffw-jes].
///
/// Addresses: Cross-Cutting Configuration
#[derive(Debug, Clone)]
pub struct JesConfig {
    /// Number of initiators in the pool (default: 3).
    pub initiator_count: usize,
    /// Retention days for completed job output (default: 7).
    pub retention_days: u32,
    /// Maximum retained jobs (default: 1000).
    pub retention_max_jobs: usize,
    /// Job Monitor refresh interval in milliseconds (default: 2000).
    pub monitor_refresh_ms: u64,
    /// Scheduler poll interval in milliseconds (default: 500).
    pub scheduler_poll_ms: u64,
    /// Job cancellation timeout in milliseconds (default: 30000).
    pub job_cancel_timeout_ms: u64,
    /// Scheduling strategy (default: Priority).
    pub scheduling_strategy: SchedulingStrategy,
    /// Spool storage root path (VFS URI).
    pub spool_root: String,
    /// Queue database path.
    pub queue_db_path: String,
}

impl Default for JesConfig {
    fn default() -> Self {
        Self {
            initiator_count: 3,
            retention_days: 7,
            retention_max_jobs: 1000,
            monitor_refresh_ms: 2000,
            scheduler_poll_ms: 500,
            job_cancel_timeout_ms: 30000,
            scheduling_strategy: SchedulingStrategy::Priority,
            spool_root: "vfs://local/.ffwb/spool".to_string(),
            queue_db_path: ".ffwb/jes-queue.db".to_string(),
        }
    }
}
```

---

## Public API Surface

### JobProvider Trait

```rust
/// Provider abstraction for job management operations.
/// Enables future extensibility to remote JES environments.
///
/// Addresses: Requirement 14
#[async_trait::async_trait]
pub trait JobProvider: Send + Sync {
    /// Returns a unique identifier for this provider.
    fn provider_id(&self) -> &str;

    /// Returns a human-readable display name.
    fn display_name(&self) -> &str;

    /// List jobs matching the given filter.
    /// Addresses: Requirement 14 AC 14.7
    async fn list_jobs(&self, filter: &JobFilter) -> Result<Vec<Job>, JesError>;

    /// Submit a job definition.
    /// Addresses: Requirement 14 AC 14.7
    async fn submit_job(&self, definition: FfjclJob, owner: &str) -> Result<JobId, JesError>;

    /// Hold a queued job.
    /// Addresses: Requirement 14 AC 14.7
    async fn hold_job(&self, id: JobId) -> Result<(), JesError>;

    /// Release a held job.
    /// Addresses: Requirement 14 AC 14.7
    async fn release_job(&self, id: JobId) -> Result<(), JesError>;

    /// Cancel a job (queued or active).
    /// Addresses: Requirement 14 AC 14.7
    async fn cancel_job(&self, id: JobId, requester: &str) -> Result<(), JesError>;

    /// Get the complete job log.
    /// Addresses: Requirement 14 AC 14.7
    async fn get_job_log(&self, id: JobId) -> Result<JobLog, JesError>;

    /// Subscribe to job state change events from this provider.
    /// Addresses: Requirement 14 AC 14.7
    fn subscribe_to_events(&self) -> broadcast::Receiver<JobEvent>;

    /// Query supported actions for a job in its current state.
    /// Addresses: Requirement 14 AC 14.5
    fn supported_actions(&self, job: &Job) -> Vec<JobAction>;

    /// Check provider health/connectivity.
    /// Addresses: Requirement 14 AC 14.6
    async fn health_check(&self) -> Result<ProviderHealth, JesError>;
}

/// Actions that can be performed on a job.
/// Addresses: Requirement 14 AC 14.5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobAction {
    ViewLog,
    Hold,
    Release,
    Cancel,
    Purge,
    Properties,
}

/// Provider health status.
/// Addresses: Requirement 14 AC 14.6
#[derive(Debug, Clone)]
pub enum ProviderHealth {
    /// Provider is healthy and responsive.
    Healthy,
    /// Provider is degraded (partial functionality).
    Degraded { reason: String },
    /// Provider is unavailable.
    Unavailable { reason: String },
}
```

### DesktopJesProvider

```rust
/// Local desktop implementation of the JobProvider trait.
/// Orchestrates queue, pool, scheduler, and spool for local job execution.
///
/// Addresses: Requirement 14 AC 14.2
pub struct DesktopJesProvider {
    queue: Arc<JobQueue>,
    pool: Arc<Mutex<InitiatorPool>>,
    spool: Arc<SpoolManager>,
    scheduler: Arc<Scheduler>,
    config: JesConfig,
}

#[async_trait::async_trait]
impl JobProvider for DesktopJesProvider {
    fn provider_id(&self) -> &str { "desktop" }
    fn display_name(&self) -> &str { "Local Desktop JES" }

    async fn list_jobs(&self, filter: &JobFilter) -> Result<Vec<Job>, JesError>;
    async fn submit_job(&self, definition: FfjclJob, owner: &str) -> Result<JobId, JesError>;
    async fn hold_job(&self, id: JobId) -> Result<(), JesError>;
    async fn release_job(&self, id: JobId) -> Result<(), JesError>;
    async fn cancel_job(&self, id: JobId, requester: &str) -> Result<(), JesError>;
    async fn get_job_log(&self, id: JobId) -> Result<JobLog, JesError>;
    fn subscribe_to_events(&self) -> broadcast::Receiver<JobEvent>;
    fn supported_actions(&self, job: &Job) -> Vec<JobAction>;
    async fn health_check(&self) -> Result<ProviderHealth, JesError>;
}
```

### Job API

```rust
/// Programmatic interface for job management operations.
/// Accessible to other plugins and Lua macros.
///
/// Addresses: Requirement 12 AC 12.1, 12.3, 12.4
pub struct JobApi {
    providers: Arc<RwLock<Vec<Box<dyn JobProvider>>>>,
}

impl JobApi {
    /// Submit a job to the default provider.
    pub async fn submit(&self, jcl: &str, owner: &str) -> Result<JobId, JesError>;

    /// Hold a job.
    pub async fn hold(&self, id: JobId) -> Result<(), JesError>;

    /// Release a held job.
    pub async fn release(&self, id: JobId) -> Result<(), JesError>;

    /// Cancel a job.
    pub async fn cancel(&self, id: JobId, requester: &str) -> Result<(), JesError>;

    /// Query job status.
    pub async fn status(&self, id: JobId) -> Result<Option<Job>, JesError>;

    /// Retrieve job log.
    pub async fn get_log(&self, id: JobId) -> Result<JobLog, JesError>;

    /// List jobs matching filter criteria.
    pub async fn list(&self, filter: &JobFilter) -> Result<Vec<Job>, JesError>;

    /// Subscribe to job state change events.
    pub fn subscribe(&self) -> broadcast::Receiver<JobEvent>;

    /// Register an additional job provider.
    /// Addresses: Requirement 14 AC 14.3
    pub async fn register_provider(&self, provider: Box<dyn JobProvider>);
}
```

### Dataset API

```rust
/// Programmatic interface for dataset operations within the JES context.
/// Delegates to ff-dataset-allocator for allocation and ff-dataset-catalog for queries.
///
/// Addresses: Requirement 12 AC 12.2, 12.5
pub struct DatasetApi {
    allocator: Arc<dyn CatalogProvider>,
}

impl DatasetApi {
    /// Allocate a new dataset.
    pub async fn allocate(&self, dsn: &str, disp: &str, dcb: Option<&str>) -> Result<String, JesError>;

    /// Read dataset content.
    pub async fn read(&self, dsn: &str) -> Result<Vec<u8>, JesError>;

    /// Write to a dataset.
    pub async fn write(&self, dsn: &str, data: &[u8]) -> Result<(), JesError>;

    /// Delete a dataset.
    pub async fn delete(&self, dsn: &str) -> Result<(), JesError>;

    /// Resolve a DSN to its physical path.
    pub async fn resolve(&self, dsn: &str) -> Result<String, JesError>;

    /// Query dataset metadata.
    pub async fn metadata(&self, dsn: &str) -> Result<DatasetMetadata, JesError>;

    /// Open a dataset in the editor.
    pub async fn open_in_editor(&self, dsn: &str) -> Result<(), JesError>;
}
```

### JesPlugin — FileForgePlugin Implementation

```rust
/// Top-level plugin implementation that bootstraps the entire JES subsystem.
///
/// Addresses: Requirement 1
pub struct JesPlugin {
    /// Plugin metadata.
    metadata: PluginMetadata,
    /// Configuration.
    config: JesConfig,
    /// Job API handle (available after activation).
    job_api: Option<Arc<JobApi>>,
    /// Dataset API handle (available after activation).
    dataset_api: Option<Arc<DatasetApi>>,
    /// Desktop provider (available after activation).
    provider: Option<Arc<DesktopJesProvider>>,
    /// Scheduler handle (available after activation).
    scheduler_handle: Option<tokio::task::JoinHandle<()>>,
    /// Retention manager handle.
    retention_handle: Option<tokio::task::JoinHandle<()>>,
}

impl FileForgePlugin for JesPlugin {
    fn metadata(&self) -> &PluginMetadata;

    fn plugin_capabilities(&self) -> &[Capability] {
        &[Capability::Commands, Capability::Viewers, Capability::Providers]
    }

    /// Register all JES commands with the command registry.
    /// Addresses: Requirement 1 AC 1.2
    fn initialize(&mut self, context: Arc<PluginContext>) -> Result<(), PluginError>;

    /// Register panels, start initiator pool and scheduler.
    /// Addresses: Requirement 1 AC 1.3, 1.4
    fn activate(&mut self) -> Result<(), PluginError>;

    /// Stop initiators, persist queue state, deregister capabilities.
    /// Addresses: Requirement 1 AC 1.5
    fn deactivate(&mut self) -> Result<(), PluginError>;

    /// Persist retained output and close resources.
    /// Addresses: Requirement 1 AC 1.6
    fn shutdown(&mut self) -> Result<(), PluginError>;
}
```

### Command Registration

```rust
/// Registers all JES commands with the command framework.
///
/// Addresses: Requirement 13
pub fn register_jes_commands(registry: &CommandRegistry) -> Result<(), JesError>;

/// Registered commands:
///
/// | Command ID              | Display Name         | Category        | Default Shortcut |
/// |-------------------------|---------------------|-----------------|------------------|
/// | `jes.job.submit`        | Submit Job          | jes.job         | Ctrl+Shift+S     |
/// | `jes.job.hold`          | Hold Job            | jes.job         | —                |
/// | `jes.job.release`       | Release Job         | jes.job         | —                |
/// | `jes.job.cancel`        | Cancel Job          | jes.job         | —                |
/// | `jes.job.purge`         | Purge Job           | jes.job         | —                |
/// | `jes.job.view_log`      | View Job Log        | jes.job         | —                |
/// | `jes.monitor.refresh`   | Refresh Monitor     | jes.monitor     | F5               |
/// | `jes.initiator.start`   | Start Initiator     | jes.initiator   | —                |
/// | `jes.initiator.stop`    | Stop Initiator      | jes.initiator   | —                |
/// | `jes.initiator.drain`   | Drain Initiator     | jes.initiator   | —                |
/// | `jes.catalog.browse`    | Browse Catalog      | jes.catalog     | —                |
///
/// Each command has an enabled predicate:
/// - `jes.job.hold`: enabled when selected job is QUEUED
/// - `jes.job.release`: enabled when selected job is HELD
/// - `jes.job.cancel`: enabled when selected job is QUEUED or ACTIVE
/// - `jes.job.purge`: enabled when selected job is in terminal state
/// - `jes.job.view_log`: enabled when any job is selected
///
/// Addresses: Requirement 13 AC 13.1–13.4
```

### JobMonitorPanel

```rust
/// SDSF-style Job Monitor implementing DockablePanel.
/// Tabbed sub-panels for each lifecycle state.
///
/// Addresses: Requirement 9
pub struct JobMonitorPanel {
    /// Active provider connections.
    providers: Vec<Arc<dyn JobProvider>>,
    /// Current filter state.
    filter: JobFilter,
    /// Selected job (for context menu actions).
    selected_job: Option<JobId>,
    /// Tab state: which sub-panel is active.
    active_tab: MonitorTab,
    /// Cached job counts per tab.
    tab_counts: HashMap<MonitorTab, usize>,
    /// Auto-refresh interval.
    refresh_interval_ms: u64,
    /// Event receiver for push updates.
    event_rx: Option<broadcast::Receiver<JobEvent>>,
}

/// Tab identifiers for the Job Monitor sub-panels.
///
/// Addresses: Requirement 9 AC 9.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MonitorTab {
    InputQueue,
    ActiveJobs,
    HeldJobs,
    CompletedJobs,
    FailedJobs,
    CancelledJobs,
}

impl DockablePanel for JobMonitorPanel {
    fn panel_id(&self) -> &str { "jes.job_monitor" }
    fn display_name(&self) -> &str { "Job Monitor" }
    fn default_dock_zone(&self) -> DockZone { DockZone::Bottom }
    fn render(&mut self, ui: &mut egui::Ui, ctx: &PanelContext);
}
```

### JobLogViewerPanel

```rust
/// Panel for viewing complete job logs with streaming support.
///
/// Addresses: Requirement 7 AC 7.1–7.6
pub struct JobLogViewerPanel {
    /// The job whose log is being viewed.
    job_id: Option<JobId>,
    /// Log content (loaded incrementally for large logs).
    log: Option<JobLog>,
    /// Active section/tab within the log viewer.
    active_section: LogSection,
    /// Search query within log content.
    search_query: Option<String>,
    /// Live streaming receiver (for active jobs).
    live_stream: Option<broadcast::Receiver<LogEntry>>,
    /// Whether currently streaming live output.
    is_streaming: bool,
}

/// Sections within the log viewer.
///
/// Addresses: Requirement 7 AC 7.3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSection {
    JesLog,
    StepLog,
    Sysout,
    ErrorOutput,
    AllocationMessages,
}

impl DockablePanel for JobLogViewerPanel {
    fn panel_id(&self) -> &str { "jes.job_log_viewer" }
    fn display_name(&self) -> &str { "Job Log" }
    fn default_dock_zone(&self) -> DockZone { DockZone::Center }
    fn render(&mut self, ui: &mut egui::Ui, ctx: &PanelContext);
}
```

---

## Error Types

```rust
/// All errors produced by the ff-jes crate.
/// Follows `[jes] operation: description` format.
///
/// Addresses: Cross-Cutting Error Handling
#[derive(Debug, thiserror::Error)]
pub enum JesError {
    /// Job submission failed (parse error, validation error, queue full).
    #[error("[jes] submission failed: {0}")]
    SubmissionFailed(String),

    /// FFJCL validation error (syntax, missing fields, unresolvable DSN).
    #[error("[jes] validation error at line {line}: {message}")]
    ValidationError { line: usize, message: String },

    /// Scheduler error (dispatch failure, no eligible jobs).
    #[error("[jes] scheduler error: {0}")]
    SchedulerError(String),

    /// Initiator failure (unrecoverable worker error).
    #[error("[jes] initiator {id} failed: {reason}")]
    InitiatorFailed { id: InitiatorId, reason: String },

    /// Dataset catalog resolution failed during job execution.
    #[error("[jes] catalog resolution failed for DSN '{dsn}': {reason}")]
    CatalogResolutionFailed { dsn: String, reason: String },

    /// Purge operation failed.
    #[error("[jes] purge failed for job {job_id}: {reason}")]
    PurgeError { job_id: JobId, reason: String },

    /// Provider is unavailable or returned an error.
    #[error("[jes] provider '{provider}' unavailable: {reason}")]
    ProviderUnavailable { provider: String, reason: String },

    /// Job not found in queue.
    #[error("[jes] job {0} not found")]
    JobNotFound(JobId),

    /// Invalid state transition (e.g., hold on active job).
    #[error("[jes] invalid state transition for job {job_id}: cannot {action} from {current_status}")]
    InvalidStateTransition {
        job_id: JobId,
        action: String,
        current_status: JobStatus,
    },

    /// Configuration error.
    #[error("[jes] configuration error: {0}")]
    ConfigError(String),

    /// I/O error (spool, persistence).
    #[error("[jes] I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Internal error (unexpected state).
    #[error("[jes] internal error: {0}")]
    Internal(String),
}
```

---

## Integration Points

### ff-plugin Integration

The `JesPlugin` struct implements `FileForgePlugin` with:
- **metadata**: name `"ffw-jes"`, capabilities `[Commands, Viewers, Providers]`, dependencies `["ff-vfs", "ff-workflow", "ff-dataset-catalog", "ff-dataset-allocator"]`
- **initialize**: Registers all `jes.*` commands with the command registry via `PluginContext`
- **activate**: Registers panels (`JobMonitorPanel`, `JobLogViewerPanel`) with the Panel_Registry, starts `InitiatorPool` and `Scheduler`
- **deactivate**: Gracefully stops all initiators, persists queue state, deregisters capabilities
- **shutdown**: Persists retained output, closes database connections

### ff-command Integration

All commands are registered under the `jes.*` namespace with:
- Unique command IDs (e.g., `jes.job.submit`)
- Display names and descriptions
- Category tags (`jes.job`, `jes.initiator`, `jes.catalog`)
- Enabled predicates (context-sensitive activation)
- Default keyboard shortcuts where applicable
- Invocable from: command palette, menus, shortcuts, context menus, Lua bridge

### ff-layout Integration

Both panels implement `DockablePanel`:
- `JobMonitorPanel`: `default_dock_zone = Bottom`, panel_id `"jes.job_monitor"`
- `JobLogViewerPanel`: `default_dock_zone = Center`, panel_id `"jes.job_log_viewer"`
- Registered via `PluginContext::register_panel()` during `activate`

### ff-workflow Integration

Job execution is modelled as a workflow:
- Each job maps to a `WorkflowDefinition` with steps corresponding to FFJCL steps
- The `JobExecutor` uses `WorkflowRunner` to execute the step graph
- Supports cancellation via `CancellationToken` propagation
- Progress reporting flows through workflow events to the Job Monitor

### ff-vfs Integration

Job logs and SYSOUT are stored via VFS:
- Resource URIs: `vfs://local/.ffwb/spool/JOB00001/jes_log`, `vfs://local/.ffwb/spool/JOB00001/step1/sysout`
- Logs appear in the file tree and can be opened in the editor
- VFS abstraction ensures cross-platform path handling

### ff-dataset-allocator Integration

During job execution, DD statement resolution:
- Each `FfjclDd` with a DSN reference is resolved via `ff-dataset-allocator`'s resolution pipeline
- DISP handling (NEW/OLD/SHR/MOD) follows the allocator's disposition semantics
- GDG relative generation references are resolved through the allocator
- Resolution messages are written to the job's allocation log section
- Failures produce `CatalogResolutionFailed` errors in the job log

### ff-config Integration

All configuration under `[plugins.ffw-jes]`:
- `initiator_count` (u32, default: 3)
- `retention_days` (u32, default: 7)
- `retention_max_jobs` (u32, default: 1000)
- `monitor_refresh_ms` (u64, default: 2000)
- `scheduler_poll_ms` (u64, default: 500)
- `job_cancel_timeout_ms` (u64, default: 30000)

### ff-logging Integration

Structured log records for:
- Job submissions (INFO)
- State transitions (INFO)
- Scheduler dispatch decisions (DEBUG)
- Initiator lifecycle changes (INFO)
- Errors and failures (ERROR)
- Retention purge operations (INFO)

---

## Correctness Properties

These properties are suitable for property-based testing with `proptest`. Each property references the requirement and acceptance criteria it validates.

### Property 1: Job ID Monotonicity

**Validates: Requirement 2.2**

For any sequence of N job submissions, the assigned Job IDs are strictly monotonically increasing: for all i < j, `job_ids[i] < job_ids[j]`.

```
∀ submissions s₁, s₂ where s₁ occurs before s₂:
    job_id(s₁) < job_id(s₂)
```

**Testing strategy**: Generate a random sequence of valid FFJCL definitions, submit them concurrently, verify the resulting IDs form a strictly increasing sequence.

### Property 2: Job Status Transition Validity

**Validates: Requirements 3.6, 6.1–6.5, 10.1–10.4**

Jobs can only transition through valid state paths. The only valid transitions are:
- QUEUED → ACTIVE (dispatch)
- QUEUED → HELD (hold)
- QUEUED → CANCELLED (cancel)
- HELD → QUEUED (release)
- ACTIVE → COMPLETED (success)
- ACTIVE → FAILED (error)
- ACTIVE → CANCELLED (cancel)

No other transitions are permitted.

```
∀ job j, ∀ status transition (old → new):
    (old, new) ∈ VALID_TRANSITIONS
```

**Testing strategy**: Generate random sequences of status update operations, verify that only valid transitions succeed and invalid transitions return `InvalidStateTransition` error.

### Property 3: Scheduler Never Exceeds Pool Capacity

**Validates: Requirement 3.7**

At no point in time does the number of ACTIVE jobs exceed the configured initiator pool capacity.

```
∀ time t:
    count(jobs where status = ACTIVE at time t) ≤ initiator_pool_capacity
```

**Testing strategy**: Configure pool with capacity N, submit M > N jobs, run scheduler, verify active count never exceeds N at any observation point.

### Property 4: Scheduler Does Not Dispatch Ineligible Jobs

**Validates: Requirements 3.4, 3.5**

The scheduler never dispatches a job whose status is HELD or CANCELLED, or whose preconditions are unmet.

```
∀ dispatched job j:
    j.status = QUEUED ∧ preconditions_met(j)
```

**Testing strategy**: Generate a mix of QUEUED, HELD, and CANCELLED jobs with random precondition states; run scheduler; verify only eligible jobs get dispatched.

### Property 5: Priority Ordering in Dispatch

**Validates: Requirement 3.2**

When using priority scheduling, a lower-priority job is never dispatched while a higher-priority eligible job remains queued.

```
∀ dispatched job j₁ at time t, ∀ queued job j₂ at time t:
    j₂.priority > j₁.priority → j₂ is not eligible at time t
```

**Testing strategy**: Generate jobs with random priorities and submission times, run priority scheduler, verify dispatch order respects priority then submission time.

### Property 6: Hold Prevents Dispatch

**Validates: Requirement 10.1**

A held job is never dispatched by the scheduler, regardless of its priority or submission time.

```
∀ job j where j.status = HELD:
    j is never in scheduler dispatch output
```

**Testing strategy**: Generate a queue with a mix of held and queued jobs; run scheduler for many cycles; verify no held job ever transitions to ACTIVE.

### Property 7: Initiator Release on Terminal Status

**Validates: Requirement 6.5**

After any job reaches a terminal status (COMPLETED, FAILED, CANCELLED), its assigned initiator is released back to IDLE state and becomes available for the next job.

```
∀ job j where j.status ∈ {COMPLETED, FAILED, CANCELLED}:
    initiator(j).status = IDLE ∨ initiator(j).status = DRAINING
    ∧ initiator(j).current_job = None
```

**Testing strategy**: Submit and execute multiple jobs to completion/failure/cancellation, verify after each terminal event the initiator is available.

### Property 8: Retention Policy Enforcement

**Validates: Requirement 8.1, 8.3**

After auto-purge runs, no retained job violates both the age limit AND the count limit simultaneously. Specifically:
- If job count > max_jobs, oldest jobs beyond the limit are purged
- If job age > retention_days, those jobs are purged

```
∀ job j retained after auto_purge:
    age(j) ≤ retention_days ∨ position(j) ≤ max_jobs
```

**Testing strategy**: Generate random sets of completed jobs with random submission times; run auto-purge; verify remaining jobs satisfy the policy.

### Property 9: Queue Persistence Round-Trip

**Validates: Requirement 2.6**

For any queue state, persisting and restoring produces an equivalent queue: same jobs, same statuses, same ordering.

```
∀ queue Q:
    restore(persist(Q)) ≡ Q
```

**Testing strategy**: Generate random queue states with various job records, persist to database, restore from database, verify deep equality of all fields.

### Property 10: Filter Does Not Mutate State

**Validates: Requirement 9.5**

Applying any filter to the job queue does not alter stored job state — filters are pure read-only projections.

```
∀ queue Q, ∀ filter F:
    let result = Q.query(F);
    Q_after == Q_before  // queue unchanged
```

**Testing strategy**: Generate random queue states and random filters, apply filter, verify the underlying queue is byte-for-byte identical before and after.

### Property 11: FFJCL Validation Rejects Invalid Definitions

**Validates: Requirement 2.7**

If an FFJCL definition has syntax errors or missing required fields, submission is rejected and no queue entry is created.

```
∀ invalid FFJCL definition d:
    submit(d) = Err(ValidationError) ∧ queue_size_after = queue_size_before
```

**Testing strategy**: Generate invalid FFJCL inputs (missing JOB statement, empty steps, invalid DDnames, syntax errors); verify submission fails with validation error and queue size unchanged.

### Property 12: Cancel Active Job Releases Initiator Within Timeout

**Validates: Requirement 6.4**

When an active job is cancelled, the initiator is released within the configured cancel timeout period.

```
∀ active job j, cancel(j) at time t:
    ∃ t' where t < t' ≤ t + cancel_timeout:
        j.status ∈ {CANCELLED} ∧ initiator(j).current_job = None
```

**Testing strategy**: Start jobs, immediately cancel them, verify the initiator is released within the timeout period (using mock executors with configurable shutdown delay).

### Property 13: Event Subscription Delivers All Transitions

**Validates: Requirement 12.4, 15.5**

Every job state transition produces exactly one event delivered to all active subscribers.

```
∀ state transition (old → new) for job j:
    ∃ exactly one JobEvent e in subscriber channels where
        e.job_id = j.id ∧ e.new_status = new ∧ e.previous_status = Some(old)
```

**Testing strategy**: Subscribe to events, perform random sequences of job operations, verify event count matches transition count and event content is correct.

### Property 14: Provider Isolation

**Validates: Requirement 14.6**

A provider connection error does not crash the application or affect other providers.

```
∀ provider P₁ failure, ∀ healthy provider P₂:
    P₂.list_jobs() succeeds ∧ P₂.submit_job() succeeds
```

**Testing strategy**: Register multiple mock providers, make one return errors, verify other providers continue to function normally and the monitor displays the error for the failing provider.

---

## Testing Framework

All property-based tests use `proptest` with a minimum of 100 iterations per property. Tests are annotated with requirement links using:

```rust
// Feature: FFW-JES, Property N: <property statement>
// Validates: Requirement X.Y
```

Integration tests use `tempfile::TempDir` for database and spool storage, and mock implementations of `ff-dataset-allocator`'s `CatalogProvider` trait for DSN resolution testing without a live catalog.
