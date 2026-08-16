# Design Document: Background I/O (`ff-background-io`)

## 1. Overview

The `ff-background-io` crate is the **async file loading and saving** coordination layer for the FileForgeWorkbench platform. It ensures the GUI remains responsive during all file I/O by spawning background tasks on the Tokio runtime, providing chunked streaming reads with progress reporting, cancellable operations, crash-safe atomic saves, and memory-efficient large-file streaming (>100 MB).

### Purpose

- Spawn async load/save tasks that never block the GUI thread (≤16ms frame budget)
- Stream file content in configurable chunks via the VFS provider interface
- Deliver progress updates (bytes, percentage, ETA) to the UI layer without blocking
- Provide cooperative cancellation with guaranteed cleanup
- Implement crash-safe saves using temp-file + atomic rename strategy
- Enforce concurrency limits and queue management for I/O tasks
- Apply large-file streaming mode for resources exceeding the configurable threshold
- Retry transient errors with exponential backoff

### Position in Architecture (Wave 8)

```
┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
├─────────────────────────────────────────────────────────────┤
│  Feature Crates: ff-file-operations, ff-external-modification│
│         (invoke background-io for all async I/O work)        │
├─────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-background-io ← Wave 8                      │
│  (coordinates async tasks between VFS and document-model)    │
├─────────────────────────────────────────────────────────────┤
│  Core Layer: ff-vfs (Wave 3), ff-document-model (Wave 4),   │
│              ff-config (Wave 2), ff-workflow-engine (Wave 2)  │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging (Wave 0)            │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **FFW-ARCH-001**: ALL file I/O flows through the VFS abstraction — no `std::fs`, `tokio::fs`, or platform-specific I/O
- **Async I/O Principle**: GUI thread never blocked >16ms by any file operation
- **Multi-Crate Workspace**: Crate at `crates/ff-background-io`
- **Error Standards**: Errors follow `[background-io] phase: description (uri: resource_uri, transferred: N bytes)` format
- **GUI Independence**: ff-background-io has zero GUI dependencies — progress delivery is via channels, not UI calls
- **Cooperative Cancellation**: No `tokio::task::abort()` — all cancellation is cooperative via `CancellationToken`

---

## 2. Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Consumers [Consuming Crates]
        FOPS[ff-file-operations]
        EXTMOD[ff-external-modification]
        LFP[ff-large-file-performance]
        WORKFLOW[ff-workflow-engine]
    end

    subgraph ff-background-io [ff-background-io Crate]
        SERVICE[BackgroundIoService]
        QUEUE[Task Queue & Scheduler]
        LOAD[LoadTask Engine]
        SAVE[SaveTask Engine]
        PROGRESS[Progress Reporter]
        CANCEL[Cancellation Coordinator]
        RETRY[Retry Policy Engine]
        CONFIG[IoConfig Reader]
        ERR[IoError Mapping]
    end

    subgraph Upstream [Upstream Crates]
        VFS[ff-vfs: VfsProvider interface]
        DOC[ff-document-model: chunk delivery]
        CFGSYS[ff-config: configuration values]
        LOG[ff-logging: diagnostic output]
    end

    FOPS -->|spawn_load / spawn_save| SERVICE
    EXTMOD -->|spawn_load for reload| SERVICE
    LFP -->|progress queries| SERVICE
    WORKFLOW -->|orchestration hooks| SERVICE

    SERVICE --> QUEUE
    QUEUE --> LOAD
    QUEUE --> SAVE
    LOAD --> PROGRESS
    SAVE --> PROGRESS
    LOAD --> CANCEL
    SAVE --> CANCEL
    LOAD --> RETRY
    SERVICE --> CONFIG
    SERVICE --> ERR

    LOAD -->|read_stream / stat| VFS
    SAVE -->|write / rename / delete| VFS
    LOAD -->|deliver chunks| DOC
    SAVE -->|read chunks from| DOC
    CONFIG -->|read settings| CFGSYS
    ERR -->|log errors| LOG
end
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **BackgroundIoService** | Central service: task submission, concurrency enforcement, lifecycle management |
| **Task Queue & Scheduler** | FIFO queue with concurrency limit, slot allocation |
| **LoadTask Engine** | Async streaming read via VFS, chunk delivery to document-model |
| **SaveTask Engine** | Async write via temp-file + atomic rename, crash-safe persistence |
| **Progress Reporter** | Non-blocking progress channel with latest-value semantics |
| **Cancellation Coordinator** | CancellationToken management, auto-cancel on document close |
| **Retry Policy Engine** | Exponential backoff retry for transient VFS errors |
| **IoConfig Reader** | Reads chunk size, thresholds, concurrency limits from ff-config |
| **IoError Mapping** | Wraps VfsError with operation phase, URI, bytes-transferred context |

### Request Flow (Load)

```
Consumer calls service.spawn_load(uri, options)
    │
    ▼
BackgroundIoService checks concurrency slots
    │  (if full → enqueue, return IoTaskHandle immediately)
    ▼
Allocate CancellationToken + ProgressState channel
    │
    ▼
Spawn LoadTask on Tokio runtime
    │
    ▼
LoadTask: VFS stat(uri) → obtain total_size (if available)
    │
    ▼
LoadTask: VFS read_stream(uri) → obtain AsyncRead stream
    │
    ▼
Loop: read chunk → check CancellationToken → deliver to doc-model → emit progress
    │
    ▼
Complete: emit final ProgressState("complete") → close channel → release slot
```

### Request Flow (Save)

```
Consumer calls service.spawn_save(uri, document_handle, options)
    │
    ▼
BackgroundIoService checks concurrency slots → enqueue if full
    │
    ▼
Allocate CancellationToken + ProgressState channel
    │
    ▼
Spawn SaveTask on Tokio runtime
    │
    ▼
SaveTask: generate temp path "{target}.ffwtmp.{random6}"
    │
    ▼
SaveTask: check disk space (if provider supports stat on parent dir)
    │
    ▼
Loop: read chunk from doc-model → check CancellationToken → write to temp → emit progress
    │
    ▼
SaveTask: flush + fsync temp file
    │
    ▼
SaveTask: preserve original metadata → atomic rename temp → target
    │
    ▼
Complete: emit final ProgressState("complete") → close channel → release slot
```

---

## 3. Module Structure

```
crates/ff-background-io/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API re-exports, crate docs, BackgroundIoService facade
│   ├── service.rs          # BackgroundIoService implementation, task submission, lifecycle
│   ├── load_task.rs        # LoadTask: async streaming read, chunk delivery, large-file mode
│   ├── save_task.rs        # SaveTask: temp-file write, fsync, atomic rename, fallback
│   ├── progress.rs         # ProgressState, ProgressCallback channel, ETA calculation
│   ├── cancellation.rs     # CancellationToken wrapper, auto-cancel logic, cleanup
│   ├── handle.rs           # IoTaskHandle: progress(), cancel(), await_completion(), result()
│   ├── queue.rs            # Task queue with FIFO ordering and concurrency slot management
│   ├── retry.rs            # RetryPolicy, exponential backoff, transient error detection
│   ├── config.rs           # IoConfig: chunk size, thresholds, concurrency, retry settings
│   ├── error.rs            # IoError enum wrapping VfsError with phase context
│   └── types.rs            # Shared types: ChunkSize, LargeFileThreshold, IoSuccess, TaskState
└── tests/
    ├── load_tests.rs       # LoadTask unit and integration tests with mock VFS
    ├── save_tests.rs       # SaveTask unit and integration tests with mock VFS
    ├── progress_tests.rs   # Progress reporting, ETA calculation, channel semantics
    ├── cancellation_tests.rs # Cooperative cancellation, cleanup verification
    ├── queue_tests.rs      # Concurrency limiting, FIFO ordering, slot management
    ├── retry_tests.rs      # Retry policy, backoff timing, transient error detection
    ├── config_tests.rs     # Configuration loading, clamping, defaults
    ├── error_tests.rs      # Error format compliance, context propagation
    └── property_tests.rs   # Property-based tests (proptest) for correctness properties
```

---

## 4. Key Data Models and Types

### BackgroundIoService

```rust
/// The central service managing all background I/O tasks.
/// Thread-safe, singleton, registered with platform-core ServiceRegistry.
/// Enforces concurrency limits and coordinates task lifecycle.
///
/// Addresses: Requirement 7 AC 1–8
pub struct BackgroundIoService {
    /// VFS instance for all I/O operations
    vfs: Arc<Vfs>,
    /// Task queue with concurrency enforcement
    queue: Arc<TaskQueue>,
    /// Active task registry (for listing, shutdown)
    active_tasks: Arc<RwLock<HashMap<TaskId, IoTaskEntry>>>,
    /// Configuration (chunk size, thresholds, concurrency)
    config: Arc<IoConfig>,
    /// Shutdown token for graceful termination
    shutdown_token: CancellationToken,
}
```

### IoTaskHandle

```rust
/// A handle returned when an I/O task is spawned.
/// Provides methods to query progress, cancel, and await completion.
/// Cloneable — multiple consumers can observe the same task.
///
/// Addresses: Requirement 1 AC 1, Requirement 3 AC 8
#[derive(Clone)]
pub struct IoTaskHandle {
    /// Unique task identifier
    id: TaskId,
    /// Latest progress state (watch channel receiver)
    progress_rx: tokio::sync::watch::Receiver<ProgressState>,
    /// Cancellation token for this task
    cancel_token: CancellationToken,
    /// Completion signal (oneshot broadcast)
    completion: Arc<tokio::sync::Notify>,
    /// Final result (populated on terminal state)
    result: Arc<RwLock<Option<Result<IoSuccess, IoError>>>>,
}

impl IoTaskHandle {
    /// Returns the most recent ProgressState without blocking.
    /// Addresses: Requirement 2 AC 5
    pub fn progress(&self) -> ProgressState;

    /// Returns an async receiver for reactive progress updates.
    /// Addresses: Requirement 2 AC 6
    pub fn subscribe_progress(&self) -> tokio::sync::watch::Receiver<ProgressState>;

    /// Triggers cooperative cancellation. Returns immediately.
    /// Addresses: Requirement 3 AC 8
    pub fn cancel(&self);

    /// Awaits the task reaching a terminal state (complete, failed, cancelled).
    /// Addresses: Requirement 3 AC 8
    pub async fn await_completion(&self);

    /// Returns the final result once the task is in a terminal state.
    /// Returns None if the task is still in progress.
    /// Addresses: Requirement 6 AC 5
    pub fn result(&self) -> Option<Result<IoSuccess, IoError>>;

    /// Returns the current task state.
    pub fn state(&self) -> TaskState;

    /// Returns the unique task identifier.
    pub fn id(&self) -> TaskId;
}
```

### LoadTask

```rust
/// Internal async task that reads a resource from the VFS in chunks
/// and delivers content progressively to the document model.
///
/// Addresses: Requirement 1, Requirement 5
pub(crate) struct LoadTask {
    /// Resource URI to load
    uri: ResourceUri,
    /// VFS provider instance (obtained through VFS registry)
    vfs: Arc<Vfs>,
    /// Chunk size for this load (clamped to 4KB–1MB)
    chunk_size: ChunkSize,
    /// Whether large-file streaming mode is active
    large_file_mode: bool,
    /// Cancellation token (checked before each chunk read)
    cancel_token: CancellationToken,
    /// Progress sender (watch channel, latest-value semantics)
    progress_tx: tokio::sync::watch::Sender<ProgressState>,
    /// Retry policy for transient errors
    retry_policy: RetryPolicy,
    /// Minimum interval between progress emissions (large-file throttle)
    progress_throttle: Duration,
}
```

### SaveTask

```rust
/// Internal async task that writes document content to the VFS
/// using temp-file + atomic rename for crash safety.
///
/// Addresses: Requirement 4, Requirement 5
pub(crate) struct SaveTask {
    /// Target resource URI
    uri: ResourceUri,
    /// VFS instance for write operations
    vfs: Arc<Vfs>,
    /// Chunk size for this save
    chunk_size: ChunkSize,
    /// Cancellation token
    cancel_token: CancellationToken,
    /// Progress sender
    progress_tx: tokio::sync::watch::Sender<ProgressState>,
    /// Whether the VFS provider supports atomic rename
    supports_atomic_rename: bool,
}
```

### ProgressState

```rust
/// Represents the current state of an I/O operation.
/// Emitted after each chunk via a watch channel (latest-value semantics).
///
/// Addresses: Requirement 2 AC 1–8
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressState {
    /// Bytes transferred so far
    pub bytes_transferred: u64,
    /// Total bytes (None if unknown, e.g., streaming-only providers)
    pub total_bytes: Option<u64>,
    /// Percentage complete (0–100, None if total unknown)
    pub percentage: Option<u8>,
    /// Elapsed time since task start
    pub elapsed: Duration,
    /// Estimated time remaining (None if < 2 seconds of data)
    pub estimated_remaining: Option<Duration>,
    /// Human-readable phase description
    pub phase: IoPhase,
}

/// The current phase of an I/O operation.
///
/// Addresses: Requirement 2 AC 7
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IoPhase {
    /// Queued, waiting for a concurrency slot
    Queued,
    /// Reading chunks from VFS stream
    Reading,
    /// Writing chunks to VFS
    Writing,
    /// Flushing and syncing to durable storage
    Finalizing,
    /// Operation was cancelled by the user
    Cancelled,
    /// Operation failed with an error
    Failed,
    /// Operation completed successfully
    Complete,
}

impl Display for IoPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Reading => write!(f, "reading"),
            Self::Writing => write!(f, "writing"),
            Self::Finalizing => write!(f, "finalizing"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Failed => write!(f, "failed"),
            Self::Complete => write!(f, "complete"),
        }
    }
}
```

### CancellationToken (Wrapper)

```rust
/// Wrapper around tokio_util::sync::CancellationToken providing
/// background-io-specific semantics: cooperative, checked per-chunk.
///
/// Addresses: Requirement 3 AC 1–7
pub struct IoCancellationToken {
    /// The underlying Tokio cancellation token
    inner: tokio_util::sync::CancellationToken,
}

impl IoCancellationToken {
    /// Create a new cancellation token.
    pub fn new() -> Self;

    /// Trigger cancellation. Returns immediately.
    pub fn cancel(&self);

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool;

    /// Await cancellation (for select! patterns within tasks).
    pub async fn cancelled(&self);

    /// Create a child token that is cancelled when this token is cancelled.
    pub fn child_token(&self) -> Self;
}
```

### Configuration Types

```rust
/// Configuration for the background I/O subsystem.
/// Read from ff-config at startup, hot-reloadable.
///
/// Addresses: Requirement 1 AC 7, Requirement 5 AC 1–2, Requirement 7 AC 1–2
#[derive(Debug, Clone)]
pub struct IoConfig {
    /// Default chunk size in bytes. Range: 4KB–1MB. Default: 64KB.
    pub chunk_size: ChunkSize,
    /// Large file threshold in bytes. Range: 10MB–4096MB. Default: 100MB.
    pub large_file_threshold: LargeFileThreshold,
    /// Maximum concurrent I/O tasks. Range: 1–16. Default: 4.
    pub max_concurrent_tasks: u8,
    /// Maximum retry attempts for transient errors. Default: 3.
    pub retry_count: u8,
    /// Initial retry backoff in milliseconds. Default: 500.
    pub retry_backoff_ms: u64,
    /// Shutdown timeout in seconds. Default: 30.
    pub shutdown_timeout_secs: u64,
}

/// A validated chunk size (4KB–1MB). Values outside range are clamped.
///
/// Addresses: Requirement 1 AC 7
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkSize(u32);

impl ChunkSize {
    pub const MIN: u32 = 4 * 1024;         // 4 KB
    pub const MAX: u32 = 1024 * 1024;      // 1 MB
    pub const DEFAULT: u32 = 64 * 1024;    // 64 KB

    /// Create a ChunkSize, clamping to valid range.
    pub fn new(bytes: u32) -> Self;

    /// Get the size in bytes.
    pub fn as_bytes(&self) -> u32;
}

/// A validated large-file threshold (10MB–4096MB). Values outside range are clamped.
///
/// Addresses: Requirement 5 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LargeFileThreshold(u64);

impl LargeFileThreshold {
    pub const MIN: u64 = 10 * 1024 * 1024;       // 10 MB
    pub const MAX: u64 = 4096 * 1024 * 1024;     // 4096 MB
    pub const DEFAULT: u64 = 100 * 1024 * 1024;  // 100 MB

    /// Create a LargeFileThreshold, clamping to valid range.
    pub fn new(bytes: u64) -> Self;

    /// Get the threshold in bytes.
    pub fn as_bytes(&self) -> u64;
}
```

### Supporting Types

```rust
/// Unique identifier for a background I/O task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

/// The lifecycle state of an I/O task.
///
/// Addresses: Requirement 7 AC 5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TaskState {
    /// Waiting in the queue for a concurrency slot
    Queued,
    /// Currently executing
    InProgress,
    /// Completed successfully
    Complete,
    /// Failed with an error
    Failed,
    /// Cancelled by the user or system
    Cancelled,
}

/// Successful completion of an I/O task.
#[derive(Debug, Clone)]
pub struct IoSuccess {
    /// Total bytes transferred
    pub bytes_transferred: u64,
    /// Total elapsed time
    pub elapsed: Duration,
    /// Resource URI that was operated on
    pub uri: ResourceUri,
}

/// Options for a load operation.
///
/// Addresses: Requirement 1 AC 7
#[derive(Debug, Clone)]
pub struct LoadOptions {
    /// Override chunk size for this load (None = use config default)
    pub chunk_size: Option<ChunkSize>,
    /// Override large-file threshold for this load
    pub large_file_threshold: Option<LargeFileThreshold>,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            chunk_size: None,
            large_file_threshold: None,
        }
    }
}

/// Options for a save operation.
///
/// Addresses: Requirement 4
#[derive(Debug, Clone)]
pub struct SaveOptions {
    /// Override chunk size for this save (None = use config default)
    pub chunk_size: Option<ChunkSize>,
    /// Whether to attempt atomic rename (default: true, falls back if unsupported)
    pub atomic: bool,
    /// Whether to preserve original file metadata after rename
    pub preserve_metadata: bool,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            chunk_size: None,
            atomic: true,
            preserve_metadata: true,
        }
    }
}

/// Retry policy configuration for transient errors.
///
/// Addresses: Requirement 6 AC 7–9
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u8,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Backoff multiplier (exponential)
    pub backoff_multiplier: f64,
}

/// An entry in the task list for the task manager UI.
///
/// Addresses: Requirement 7 AC 5
#[derive(Debug, Clone)]
pub struct IoTaskEntry {
    pub id: TaskId,
    pub uri: ResourceUri,
    pub task_type: IoTaskType,
    pub state: TaskState,
    pub progress: ProgressState,
}

/// Discriminator for load vs save tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoTaskType {
    Load,
    Save,
}
```

---

## 5. Public API Surface

### BackgroundIoService — Construction and Lifecycle

```rust
impl BackgroundIoService {
    /// Create a new BackgroundIoService with the given VFS and configuration.
    /// Registered as a singleton with platform-core ServiceRegistry.
    ///
    /// Addresses: Requirement 7 AC 8
    pub fn new(vfs: Arc<Vfs>, config: IoConfig) -> Self;

    /// Graceful shutdown: cancel LoadTasks, await SaveTasks, drain queue.
    /// Returns after all tasks terminate or shutdown timeout expires.
    ///
    /// Addresses: Requirement 7 AC 6–7
    pub async fn shutdown(&self, timeout: Duration);
}
```

### Task Spawning

```rust
impl BackgroundIoService {
    /// Spawn an async load task for the given resource URI.
    /// Returns an IoTaskHandle immediately without blocking.
    /// If concurrency limit is reached, the task is enqueued (FIFO).
    ///
    /// Addresses: Requirement 1 AC 1, Requirement 7 AC 3
    pub fn spawn_load(
        &self,
        uri: ResourceUri,
        options: LoadOptions,
    ) -> IoTaskHandle;

    /// Spawn an async save task for the given resource URI.
    /// The document_source provides chunked content to write.
    /// Returns an IoTaskHandle immediately without blocking.
    ///
    /// Addresses: Requirement 4 AC 1, Requirement 7 AC 3
    pub fn spawn_save(
        &self,
        uri: ResourceUri,
        document_source: Arc<dyn DocumentChunkSource>,
        options: SaveOptions,
    ) -> IoTaskHandle;
}
```

### Cancellation

```rust
impl BackgroundIoService {
    /// Cancel a specific task by ID. Triggers cooperative cancellation.
    /// Returns immediately without waiting for the task to finish.
    ///
    /// Addresses: Requirement 3 AC 8
    pub fn cancel(&self, task_id: TaskId);

    /// Cancel all tasks associated with a given resource URI.
    /// Used when a document is closed to prevent resource leaks.
    ///
    /// Addresses: Requirement 3 AC 6
    pub fn cancel_for_uri(&self, uri: &ResourceUri);
}
```

### Progress and Status Queries

```rust
impl BackgroundIoService {
    /// Query the progress of a specific task.
    /// Returns the most recent ProgressState without blocking.
    ///
    /// Addresses: Requirement 2 AC 5
    pub fn query_progress(&self, task_id: TaskId) -> Option<ProgressState>;

    /// List all active and queued tasks with their current states.
    /// Used for task manager UI display.
    ///
    /// Addresses: Requirement 7 AC 5
    pub fn list_tasks(&self) -> Vec<IoTaskEntry>;

    /// Await completion of a specific task (terminal state).
    /// Convenience wrapper around IoTaskHandle::await_completion().
    ///
    /// Addresses: Requirement 3 AC 8
    pub async fn await_completion(&self, task_id: TaskId);
}
```

### DocumentChunkSource Trait

```rust
/// Trait for providing document content in chunks during save operations.
/// Implemented by the document-model to support streaming saves without
/// requiring the entire document content as a single allocation.
///
/// Addresses: Requirement 5 AC 4
pub trait DocumentChunkSource: Send + Sync {
    /// Returns the total content size in bytes (if known).
    fn total_size(&self) -> Option<u64>;

    /// Read the next chunk of content. Returns empty slice when complete.
    /// The chunk size hint guides the implementor but is not mandatory.
    fn next_chunk(&self, chunk_size_hint: usize) -> Option<Vec<u8>>;

    /// Reset to the beginning (for retry scenarios).
    fn reset(&self);
}
```

### Memory Pressure Callback

```rust
impl BackgroundIoService {
    /// Register a memory-pressure callback. When invoked, pauses
    /// large-file LoadTasks until memory is freed or user continues.
    ///
    /// Addresses: Requirement 5 AC 7
    pub fn set_memory_pressure_callback(
        &self,
        callback: Box<dyn Fn() -> bool + Send + Sync>,
    );
}
```

---

## 6. Error Types

```rust
/// Error type for all background I/O operations.
/// Wraps VfsError with operation-phase context, resource URI, and transfer state.
/// Every variant produces a message conforming to:
/// `[background-io] phase: description (uri: resource_uri, transferred: N bytes)`
///
/// Addresses: Requirement 6 AC 1–2
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IoError {
    /// Failed to open or initiate the VFS stream
    #[error("[background-io] open: {description} (uri: {uri}, transferred: 0 bytes)")]
    OpenFailed {
        uri: String,
        description: String,
        #[source]
        source: VfsError,
    },

    /// Failed during chunk read
    #[error("[background-io] read-chunk: {description} (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    ReadChunkFailed {
        uri: String,
        description: String,
        bytes_transferred: u64,
        #[source]
        source: VfsError,
    },

    /// Failed during chunk write
    #[error("[background-io] write-chunk: {description} (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    WriteChunkFailed {
        uri: String,
        description: String,
        bytes_transferred: u64,
        #[source]
        source: VfsError,
    },

    /// Failed during flush/fsync
    #[error("[background-io] flush: {description} (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    FlushFailed {
        uri: String,
        description: String,
        bytes_transferred: u64,
        #[source]
        source: VfsError,
    },

    /// Failed during atomic rename
    #[error("[background-io] rename: {description} (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    RenameFailed {
        uri: String,
        description: String,
        bytes_transferred: u64,
        #[source]
        source: VfsError,
    },

    /// Failed during cleanup (temp file deletion)
    #[error("[background-io] cleanup: {description} (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    CleanupFailed {
        uri: String,
        description: String,
        bytes_transferred: u64,
        #[source]
        source: VfsError,
    },

    /// Insufficient disk space detected before write
    #[error("[background-io] space-check: insufficient space for save (uri: {uri}, required: {required_bytes} bytes, available: {available_bytes} bytes)")]
    InsufficientSpace {
        uri: String,
        required_bytes: u64,
        available_bytes: u64,
    },

    /// Task was cancelled
    #[error("[background-io] cancelled: operation cancelled by user (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    Cancelled {
        uri: String,
        bytes_transferred: u64,
    },

    /// All retries exhausted for a transient error
    #[error("[background-io] retries-exhausted: {description} after {attempts} attempts (uri: {uri}, transferred: {bytes_transferred} bytes)")]
    RetriesExhausted {
        uri: String,
        description: String,
        bytes_transferred: u64,
        attempts: u8,
        #[source]
        source: VfsError,
    },

    /// Provider does not support required capability
    #[error("[background-io] capability: provider '{provider}' does not support {capability} (uri: {uri})")]
    UnsupportedCapability {
        uri: String,
        provider: String,
        capability: String,
    },

    /// Task not found (invalid TaskId)
    #[error("[background-io] lookup: task {task_id} not found")]
    TaskNotFound {
        task_id: u64,
    },
}
```

---

## 7. Integration Points

### With `ff-vfs` (Wave 3 — upstream)

- **Dependency direction**: ff-background-io depends on ff-vfs
- **API consumed**: `VfsProvider::read_stream`, `VfsProvider::write`, `VfsProvider::open`, `VfsProvider::stat`, `VfsProvider::rename`, `VfsProvider::delete`
- **Provider access**: Obtained through `Vfs::registry()` — never constructs providers directly
- **Capability checks**: Verifies `WRITE` before save, `RENAME` before atomic save, `RANDOM_ACCESS` before seek-based partial load
- **Error mapping**: `VfsError` variants wrapped into `IoError` with phase context
- **Timeout handling**: `VfsError::Timeout` treated as transient error eligible for retry
- **Resource identifiers**: All paths are `ResourceUri` values from the VFS spec

### With `ff-document-model` (Wave 4 — peer/downstream consumer)

- **Dependency direction**: ff-background-io depends on ff-document-model's `DocumentChunkSource` trait (or defines it locally and document-model implements it)
- **Load delivery**: LoadTask delivers chunks to the document model's streaming interface for progressive line-index construction
- **Save source**: SaveTask reads content from document-model via `DocumentChunkSource::next_chunk()` to avoid single-allocation requirement
- **Large-file coordination**: In streaming mode, chunks are delivered as they arrive without buffering the entire file

### With `ff-file-operations` (Wave 8 — downstream consumer)

- **Dependency direction**: ff-file-operations depends on ff-background-io
- **Integration**: file-operations invokes `spawn_load`/`spawn_save` for all Open/Save/Revert commands
- **Coordination**: file-operations manages the user-facing workflow (encoding selection, overwrite confirmation); background-io handles execution
- **Error handling**: file-operations receives `IoError` from `IoTaskHandle::result()` and presents it to the user

### With `ff-workflow-engine` (Wave 2 — upstream)

- **Dependency direction**: ff-background-io may implement workflow steps for complex I/O sequences
- **Integration**: Long-running I/O (large-file loads) can be wrapped as workflow steps for user interaction (encoding dialog, overwrite confirmation)
- **Progress reporting**: Workflow engine can observe I/O progress through `IoTaskHandle::subscribe_progress()`
- **Cancellation**: Workflow cancellation propagates to the I/O task via the CancellationToken chain

### With `ff-config` (Wave 2 — upstream)

- **Dependency direction**: ff-background-io depends on ff-config
- **Configuration namespace**: `[io]` in the workbench TOML file
- **Configuration keys**:
  - `io.chunk_size_kb` — default chunk size in KB (default: 64)
  - `io.large_file_threshold_mb` — large-file threshold in MB (default: 100)
  - `io.max_concurrent_tasks` — concurrency limit (default: 4)
  - `io.retry_count` — max retries for transient errors (default: 3)
  - `io.retry_backoff_ms` — initial retry backoff in ms (default: 500)
  - `io.shutdown_timeout_secs` — graceful shutdown timeout (default: 30)
- **Hot-reload**: Configuration changes apply to newly spawned tasks; in-progress tasks keep their initial config

### With `ff-logging` (Wave 0 — upstream)

- **Dependency direction**: ff-background-io depends on ff-logging
- **Log prefix**: `[background-io]`
- **ERROR level**: All I/O errors logged before propagation (Requirement 6 AC 6)
- **WARN level**: Retry attempts (Requirement 6 AC 9), fallback to non-atomic save (Requirement 4 AC 6), shutdown timeout exceeded
- **INFO level**: Task spawn, task completion, large-file mode activation
- **DEBUG level**: Per-chunk progress, retry backoff timing

### With `ff-external-modification` (Wave 8 — downstream consumer)

- **Dependency direction**: ff-external-modification depends on ff-background-io
- **Integration**: When VFS file-watcher detects external changes, external-modification invokes `spawn_load` for async reload
- **Cancellation**: If user declines reload, external-modification cancels the load task

### With `ff-large-file-performance` (Wave 15 — downstream consumer)

- **Dependency direction**: ff-large-file-performance depends on ff-background-io
- **Integration**: Observes progress of large-file loads to coordinate chunked rendering and measurement caching
- **Memory pressure**: May trigger the memory-pressure callback to pause large-file loading

### Dependency Direction Summary

```
ff-logging ← ff-background-io ← ff-file-operations (consumer)
ff-config  ← ff-background-io ← ff-external-modification (consumer)
ff-vfs     ← ff-background-io ← ff-large-file-performance (consumer)
ff-document-model ← ff-background-io
ff-workflow-engine ← ff-background-io (optional workflow step integration)
```

---

## 8. Configuration

ff-background-io owns the `[io]` namespace in the workbench TOML configuration file.

### TOML Schema

```toml
[io]
# Default chunk size in KB for streaming reads/writes.
# Range: 4–1024 (4 KB to 1 MB). Default: 64
chunk_size_kb = 64

# File size threshold (MB) above which large-file streaming mode activates.
# Range: 10–4096. Default: 100
large_file_threshold_mb = 100

# Maximum number of concurrent background I/O tasks.
# Range: 1–16. Default: 4
max_concurrent_tasks = 4

# Maximum retry attempts for transient errors (network timeout, etc.).
# Range: 0–10. Default: 3
retry_count = 3

# Initial retry backoff in milliseconds. Subsequent retries double this.
# Range: 100–10000. Default: 500
retry_backoff_ms = 500

# Graceful shutdown timeout in seconds.
# Range: 5–120. Default: 30
shutdown_timeout_secs = 30
```

### Config Resolution Rules

| Setting | Absent | Invalid Value | Out of Range |
|---------|--------|---------------|--------------|
| `chunk_size_kb` | Default to 64 | Default to 64 + WARN log | Clamp to [4–1024] + WARN |
| `large_file_threshold_mb` | Default to 100 | Default to 100 + WARN log | Clamp to [10–4096] + WARN |
| `max_concurrent_tasks` | Default to 4 | Default to 4 + WARN log | Clamp to [1–16] + WARN |
| `retry_count` | Default to 3 | Default to 3 + WARN log | Clamp to [0–10] + WARN |
| `retry_backoff_ms` | Default to 500 | Default to 500 + WARN log | Clamp to [100–10000] + WARN |
| `shutdown_timeout_secs` | Default to 30 | Default to 30 + WARN log | Clamp to [5–120] + WARN |

---

## 9. Correctness Properties (Property-Based Testing)

The following properties are suitable for property-based testing with the `proptest` crate. Each property is universal — it must hold for all valid inputs.

### Property 1: ChunkSize Clamping Idempotence

**Statement:** For any u32 value, `ChunkSize::new(v)` always produces a value within [4KB, 1MB]. Applying `ChunkSize::new` to an already-valid ChunkSize returns the same value.

```
∀ v: u32,
    4096 ≤ ChunkSize::new(v).as_bytes() ≤ 1_048_576
∀ cs: ChunkSize,
    ChunkSize::new(cs.as_bytes()) == cs
```

**Validates:** Requirement 1 AC 7

### Property 2: LargeFileThreshold Clamping Idempotence

**Statement:** For any u64 value, `LargeFileThreshold::new(v)` always produces a value within [10MB, 4096MB]. Applying construction to an already-valid threshold is identity.

```
∀ v: u64,
    10_485_760 ≤ LargeFileThreshold::new(v).as_bytes() ≤ 4_294_967_296
∀ t: LargeFileThreshold,
    LargeFileThreshold::new(t.as_bytes()) == t
```

**Validates:** Requirement 5 AC 2

### Property 3: Progress Percentage Monotonicity

**Statement:** For a LoadTask with known total size, the sequence of emitted ProgressState percentages is monotonically non-decreasing and bounded by [0, 100].

```
∀ progress_sequence ps from a load with known total:
    ∀ i < j: ps[i].percentage ≤ ps[j].percentage
    ∀ p in ps: 0 ≤ p.percentage ≤ 100
```

**Validates:** Requirement 2 AC 3

### Property 4: Progress Bytes Transferred Monotonicity

**Statement:** For any I/O task (load or save), the bytes_transferred field in successive progress emissions is strictly non-decreasing until a terminal state.

```
∀ progress_sequence ps (non-terminal states only):
    ∀ i < j: ps[i].bytes_transferred ≤ ps[j].bytes_transferred
```

**Validates:** Requirement 2 AC 1

### Property 5: Cancellation Bounded Latency

**Statement:** After a CancellationToken is triggered, the LoadTask terminates within at most one chunk-read duration plus 100ms. Specifically, no more than one additional chunk is read after cancellation.

```
∀ load_task with cancellation triggered at byte position B:
    final_bytes_transferred ≤ B + chunk_size + overhead
    elapsed_after_cancel ≤ one_chunk_read_time + 100ms
```

**Validates:** Requirement 3 AC 4

### Property 6: Atomic Save — Original Unmodified on Failure

**Statement:** If a SaveTask fails at any phase (write, flush, rename), the original target file remains unmodified. The target file's content and metadata are identical before and after the failed save.

```
∀ save_task that fails:
    content(target, after_failure) == content(target, before_save)
    metadata(target, after_failure) == metadata(target, before_save)
```

**Validates:** Requirement 4 AC 7

### Property 7: Concurrency Limit Invariant

**Statement:** At no point in time does the number of concurrently executing I/O tasks exceed `max_concurrent_tasks`. Tasks beyond the limit are queued and execute in FIFO order.

```
∀ time t:
    count(tasks where state == InProgress at time t) ≤ max_concurrent_tasks
∀ queued tasks t1, t2 where t1 enqueued before t2:
    t1.start_time ≤ t2.start_time
```

**Validates:** Requirement 7 AC 1, AC 3

### Property 8: Error Format Compliance

**Statement:** Every `IoError` variant's `Display` output: (a) starts with `[background-io]`, (b) contains a phase identifier, (c) contains the resource URI when applicable, (d) is ≤200 characters.

```
∀ error: IoError:
    error.to_string().starts_with("[background-io]")
    ∧ error.to_string().contains(phase_name)
    ∧ (has_uri → error.to_string().contains(uri))
    ∧ error.to_string().len() ≤ 200
```

**Validates:** Requirement 6 AC 2, Cross-cutting Requirement 8

### Property 9: Retry Preserves Position

**Statement:** When a transient error occurs and retry succeeds, the LoadTask continues from the last successfully delivered byte position — it does not restart from byte 0.

```
∀ load_task with transient error at byte B, retry succeeds:
    next_chunk_start_position == B (not 0)
    total_bytes_delivered == file_size (no gaps, no duplicates)
```

**Validates:** Requirement 6 AC 8

### Property 10: Large-File Memory Bound

**Statement:** In large-file streaming mode, the LoadTask never holds more than 2× chunk_size of buffered data at any point during loading, regardless of file size.

```
∀ load_task in large_file_mode, ∀ time t:
    buffered_data_size(t) ≤ 2 * chunk_size
```

**Validates:** Requirement 5 AC 1

### Property 11: Terminal State Finality

**Statement:** Once a task reaches a terminal state (Complete, Failed, Cancelled), its state never changes. The progress channel emits exactly one final ProgressState and then closes.

```
∀ task that reaches terminal state S:
    ∀ subsequent queries: task.state() == S
    progress_channel.recv() after terminal == None (closed)
    count(progress_emissions with phase == terminal_phase) == 1
```

**Validates:** Requirement 2 AC 8

### Property 12: Save Temp File Cleanup

**Statement:** After a SaveTask completes (whether success, failure, or cancellation), no temporary files (`*.ffwtmp.*`) remain in the target directory. The temp file is either renamed (success) or deleted (failure/cancel).

```
∀ save_task after reaching terminal state:
    count(files matching "{target}.ffwtmp.*" in target_dir) == 0
```

**Validates:** Requirement 4 AC 2, AC 7, Requirement 3 AC 3

---

## 10. Testing Strategy

### Unit Tests
- `load_tests.rs`: Streaming read with mock VFS, chunk delivery verification, large-file mode activation
- `save_tests.rs`: Temp-file creation, atomic rename, fallback write-in-place, metadata preservation
- `progress_tests.rs`: ETA calculation accuracy, percentage computation, throttle enforcement
- `cancellation_tests.rs`: Cooperative cancel, cleanup verification, bounded latency
- `queue_tests.rs`: Concurrency limiting, FIFO ordering, slot release on completion
- `retry_tests.rs`: Exponential backoff, position preservation, max-retries enforcement
- `config_tests.rs`: Clamping, default application, invalid value handling
- `error_tests.rs`: Format compliance, context propagation, Display output validation

### Property-Based Tests (proptest)
- ChunkSize clamping (Property 1)
- LargeFileThreshold clamping (Property 2)
- Progress monotonicity (Properties 3, 4)
- Cancellation bounded latency (Property 5)
- Atomic save integrity (Property 6)
- Concurrency limit invariant (Property 7)
- Error format compliance (Property 8)
- Retry position preservation (Property 9)
- Large-file memory bound (Property 10)
- Terminal state finality (Property 11)
- Temp file cleanup (Property 12)

### Integration Tests
- End-to-end load with in-memory VFS provider: spawn → progress → completion
- End-to-end save with atomic rename: write → fsync → rename → verify content
- Concurrent task scheduling: spawn N+1 tasks with limit N, verify queuing
- Cancellation mid-load: trigger cancel, verify partial content preserved
- Large-file streaming: 200MB mock file, verify memory stays within bounds
- Retry with transient failures: inject VfsError::Timeout, verify resumption
- Shutdown graceful: spawn tasks → shutdown → verify saves complete, loads cancel

### Test Infrastructure
- **MockVfsProvider**: A configurable VFS provider backed by in-memory storage with injectable delays, errors, and size reporting
- **Testing framework**: `proptest` for property-based tests, `#[tokio::test]` for async tests
- **Minimum proptest iterations**: 100 per property
- **Time simulation**: Use `tokio::time::pause()` for deterministic timing in progress/retry tests
