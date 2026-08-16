# Requirements Document

## Introduction

This feature specifies the **Background I/O** subsystem for FileForgeWorkbench — the `ff-background-io` crate. The background-io subsystem provides the async file loading and saving infrastructure that keeps the GUI responsive during all file I/O operations. It implements chunked streaming reads with progress reporting, cancellable operations, background save with temp-file + atomic rename for data integrity, and large-file streaming (>100 MB) with memory-efficient processing.

All file operations flow through the **Virtual File System abstraction** (FFW-ARCH-001) — background-io uses the VFS provider async interface (`read_stream`, `write`, `open`) and never calls `std::fs`, `tokio::fs`, or any platform-specific I/O directly. The subsystem is a **coordination layer** between the VFS providers and the document-model: it schedules async tasks, manages progress channels, propagates errors, and coordinates cancellation tokens.

Background-io integrates with the workbench logging subsystem (`ff-logging`) for diagnostic output and uses Tokio-based async execution per the Architecture Brief async I/O principle (§9).

**Source references:**
- **[SCI-STE-IO]** = SciTE gap analysis — background file loading/saving, progress bar, large-file handling
- **[WB]** = Workbench Platform Architecture Brief — async I/O principle, VFS-first access, non-blocking GUI
- **[FFE-MVP-1]** = FileForgeEditor MVP Requirement 1 — streaming file reader, progressive display
- **[FFW-ARCH-001]** = Overriding connectivity principle — all content through VFS abstraction layer

## Glossary

- **LoadTask**: An async task that reads a resource from the VFS in chunks and delivers content progressively to the document model. [SCI-STE-IO]
- **SaveTask**: An async task that writes document content to the VFS using a temp-file and atomic rename strategy to prevent data corruption. [SCI-STE-IO]
- **CancellationToken**: A cooperative signal (Tokio `CancellationToken`) that indicates an I/O operation should terminate gracefully. Shared between the task and its caller. [WB]
- **ProgressCallback**: A function or channel through which a LoadTask or SaveTask reports progress updates (bytes transferred, percentage, status) to the UI layer. [SCI-STE-IO]
- **ProgressState**: A struct representing the current state of an I/O operation: bytes transferred, total bytes (if known), percentage, elapsed time, and estimated remaining time. [SCI-STE-IO]
- **ChunkSize**: The configurable size of each read or write chunk during streaming I/O. Default 64 KB for normal files, configurable up to 1 MB. [SCI-STE-IO]
- **LargeFileThreshold**: The size boundary (default 100 MB) above which a file is treated as a large file with memory-mapped or streaming-only access patterns. [SCI-STE-IO]
- **AtomicRename**: The save strategy where content is written to a temporary file in the same directory, then renamed over the target — ensuring the target is never in a partially-written state. [SCI-STE-IO]
- **IoTaskHandle**: A handle returned when an I/O task is spawned, providing methods to query progress, await completion, or cancel the task. [WB]
- **IoError**: The error type for background-io operations, wrapping `VfsError` with additional context about the operation phase and resource URI. [WB]
- **BackgroundIoService**: The central service that manages active I/O tasks, enforces concurrency limits, and coordinates with the VFS provider registry. [WB]
- **VFS**: Virtual File System — the abstraction layer through which all file access flows (FFW-ARCH-001). [WB]

---

## Requirements

### Requirement 1: Async Streaming File Load

**User Story:** As a user, I want files to load in the background without freezing the UI, so that I can continue working while large files are being read from disk or a remote source.

**Source:** [SCI-STE-IO], [FFE-MVP-1], [WB]

#### Acceptance Criteria

1. WHEN a file load is initiated, THE BackgroundIoService SHALL spawn an async LoadTask on the Tokio runtime that reads the resource via the VFS provider's `read_stream` interface, returning an IoTaskHandle immediately without blocking the calling thread. [SCI-STE-IO, WB]
2. THE LoadTask SHALL read the resource content in configurable chunks (default 64 KB per chunk), delivering each chunk to the document model as it arrives so that partially-loaded content is available for display. [SCI-STE-IO, FFE-MVP-1]
3. THE LoadTask SHALL NOT load the entire file into a single contiguous memory allocation before delivering content — streaming delivery SHALL begin after the first chunk is received. [SCI-STE-IO]
4. WHEN a LoadTask is spawned, THE BackgroundIoService SHALL first query the resource metadata via the VFS `stat` operation to obtain the total file size (if available), enabling percentage-based progress reporting. [SCI-STE-IO]
5. IF the VFS provider does not report file size (e.g., streaming-only providers), THEN THE LoadTask SHALL report progress as bytes-loaded without percentage, and SHALL indicate that total size is unknown. [SCI-STE-IO]
6. WHEN all chunks have been read and delivered, THE LoadTask SHALL signal completion to the IoTaskHandle, transition to a completed state, and release all I/O resources (stream handles, buffers). [SCI-STE-IO]
7. THE chunk size SHALL be configurable per-load via an options parameter, with a minimum of 4 KB and a maximum of 1 MB; values outside this range SHALL be clamped. [SCI-STE-IO]
8. ALL file reads SHALL flow through the VFS abstraction layer — the LoadTask SHALL NOT use `std::fs`, `tokio::fs`, or any platform-specific I/O directly. [FFW-ARCH-001]

---

### Requirement 2: Progress Reporting

**User Story:** As a user, I want to see how much of a file has loaded or saved, so that I know the operation is progressing and can estimate how long it will take.

**Source:** [SCI-STE-IO], [WB]

#### Acceptance Criteria

1. THE LoadTask and SaveTask SHALL report progress via a ProgressCallback channel after each chunk is processed, emitting a ProgressState containing: bytes transferred so far, total bytes (if known), percentage (if total is known), elapsed time since task start, and estimated time remaining. [SCI-STE-IO]
2. THE progress channel SHALL be non-blocking on the producer side — if the UI consumer has not read previous progress events, THE task SHALL continue processing without waiting, overwriting the latest progress state (latest-value semantics). [WB]
3. WHEN percentage can be calculated (total size known), THE ProgressState SHALL report percentage as a value between 0 and 100, computed as `(bytes_transferred * 100) / total_bytes`. [SCI-STE-IO]
4. THE estimated time remaining SHALL be calculated using an exponential moving average of the transfer rate over the last 5 seconds of activity, returning `None` if fewer than 2 seconds of data are available. [SCI-STE-IO]
5. THE IoTaskHandle SHALL expose a `progress()` method that returns the most recent ProgressState without blocking, enabling the UI to poll for updates at its own refresh rate. [SCI-STE-IO]
6. THE IoTaskHandle SHALL expose a `subscribe_progress()` method that returns an async channel receiver, enabling the UI to await progress updates reactively rather than polling. [WB]
7. THE ProgressState SHALL include a human-readable status string describing the current phase: "reading", "writing", "finalizing", "cancelled", "failed", or "complete". [SCI-STE-IO]
8. WHEN a task transitions to a terminal state (complete, failed, or cancelled), THE task SHALL emit one final ProgressState reflecting the terminal status, then close the progress channel. [SCI-STE-IO]

---

### Requirement 3: Cancellation

**User Story:** As a user, I want to cancel a long-running file load or save, so that I can abort operations that are taking too long or that I started by mistake.

**Source:** [SCI-STE-IO], [WB]

#### Acceptance Criteria

1. EACH IoTaskHandle SHALL carry a CancellationToken that the caller can trigger to request cooperative cancellation of the associated I/O task. [WB]
2. WHEN the CancellationToken is triggered, THE LoadTask SHALL stop reading further chunks from the VFS stream within one chunk-read cycle, release all I/O resources, and transition to a cancelled state. [SCI-STE-IO]
3. WHEN the CancellationToken is triggered during a SaveTask, THE SaveTask SHALL stop writing further chunks, delete the temporary file (if one was created), and transition to a cancelled state without modifying the original target file. [SCI-STE-IO]
4. THE LoadTask SHALL check the CancellationToken before each chunk read — the maximum latency between cancellation request and task termination SHALL be bounded by one chunk-read duration plus 100ms of cleanup. [WB]
5. WHEN a task is cancelled, THE IoTaskHandle SHALL report the final state as "cancelled" with the number of bytes that were successfully transferred before cancellation. [SCI-STE-IO]
6. IF a document is closed while a LoadTask is still in progress, THE BackgroundIoService SHALL automatically trigger the CancellationToken for that task to prevent resource leaks. [WB]
7. CANCELLATION SHALL be cooperative — the LoadTask and SaveTask SHALL never be forcibly aborted (no `tokio::task::abort()`), ensuring all cleanup code (file handle closure, temp file deletion) executes. [WB]
8. THE IoTaskHandle SHALL expose a `cancel()` method that triggers the CancellationToken and returns immediately without waiting for the task to finish; a separate `await_completion()` method SHALL be available to wait for the task to reach a terminal state. [WB]

---

### Requirement 4: Background Save with Atomic Rename

**User Story:** As a user, I want file saves to be crash-safe, so that if the application crashes or the system loses power during a save, my original file is not corrupted.

**Source:** [SCI-STE-IO], [WB]

#### Acceptance Criteria

1. WHEN a save is initiated, THE BackgroundIoService SHALL spawn an async SaveTask that writes content to a temporary file in the same directory as the target file, then atomically renames the temporary file over the target. [SCI-STE-IO]
2. THE temporary file SHALL be named with the pattern `{target_filename}.ffwtmp.{random_suffix}` where `random_suffix` is a 6-character alphanumeric string, ensuring uniqueness even with concurrent saves. [SCI-STE-IO]
3. THE SaveTask SHALL write document content in configurable chunks (same chunk size semantics as LoadTask), reporting progress after each chunk via the ProgressCallback channel. [SCI-STE-IO]
4. AFTER all content has been written and flushed to the temporary file, THE SaveTask SHALL invoke an fsync-equivalent operation (via the VFS provider) to ensure data is durably persisted to storage before attempting the rename. [SCI-STE-IO]
5. AFTER fsync completes, THE SaveTask SHALL atomically rename the temporary file to the target path, replacing the original file — this operation SHALL be performed through the VFS provider's `rename` method. [SCI-STE-IO]
6. IF the VFS provider does not support atomic rename (as declared by its capabilities), THEN THE SaveTask SHALL fall back to a write-in-place strategy: truncate the target file and write content directly, logging a WARN-level record indicating that atomic save is unavailable for this provider. [SCI-STE-IO]
7. IF any step of the save fails (write, flush, fsync, or rename), THEN THE SaveTask SHALL attempt to delete the temporary file, transition to a failed state, and propagate the error through the IoTaskHandle — the original target file SHALL NOT be modified. [SCI-STE-IO]
8. THE SaveTask SHALL preserve the original file's permissions and ownership metadata after the atomic rename, querying metadata before the rename and applying it after (where the platform supports it). [SCI-STE-IO]
9. IF the target directory does not have sufficient space for both the temporary file and the original (i.e., disk space check), THE BackgroundIoService SHALL report an IoError before starting the write rather than failing mid-save when possible. [SCI-STE-IO]
10. ALL save operations SHALL flow through the VFS abstraction layer — the SaveTask SHALL NOT use `std::fs`, `tokio::fs`, or any platform-specific I/O directly. [FFW-ARCH-001]

---

### Requirement 5: Large-File Streaming Support

**User Story:** As a user, I want to open and save files larger than 100 MB without the application running out of memory or becoming unresponsive, so that I can work with log files, data dumps, and other large resources.

**Source:** [SCI-STE-IO], [FFE-MVP-1], [WB]

#### Acceptance Criteria

1. WHEN a resource's size exceeds the LargeFileThreshold (default 100 MB, configurable), THE BackgroundIoService SHALL apply large-file streaming mode: reading content in chunks without holding more than 2× chunk size of buffered data at any time. [SCI-STE-IO]
2. THE LargeFileThreshold SHALL be configurable via the workbench configuration-system (`io.large_file_threshold_mb`), with a minimum of 10 MB and maximum of 4096 MB; values outside this range SHALL be clamped. [SCI-STE-IO]
3. IN large-file streaming mode, THE LoadTask SHALL deliver chunks to the document model as they arrive, enabling progressive line-index construction without waiting for the complete file to be buffered. [FFE-MVP-1]
4. IN large-file streaming mode, THE SaveTask SHALL read document content in chunks from the document model (not by requesting the entire content as a single allocation), streaming chunks directly to the VFS write operation. [SCI-STE-IO]
5. THE BackgroundIoService SHALL NOT impose a maximum file size limit — files of any size SHALL be loadable as long as the system has sufficient memory for the document model's representation (which may be smaller than the raw file due to structural sharing). [SCI-STE-IO]
6. WHEN loading a large file, THE LoadTask SHALL emit progress events at a rate no faster than once per 50 milliseconds to avoid flooding the progress channel, even if chunks complete faster. [SCI-STE-IO]
7. THE BackgroundIoService SHALL support a memory-pressure callback: if the system reports low memory conditions, in-progress LoadTasks for large files SHALL pause until memory is freed or the user explicitly requests continuation. [WB]
8. FOR files below the LargeFileThreshold, THE LoadTask MAY buffer the entire file before delivering it to the document model in a single operation, if the VFS provider supports efficient single-read access. [SCI-STE-IO]

---

### Requirement 6: Error Propagation and Recovery

**User Story:** As a user, I want clear error messages when file operations fail, and I want the application to recover gracefully without losing my unsaved work.

**Source:** [SCI-STE-IO], [WB]

#### Acceptance Criteria

1. THE `ff-background-io` crate SHALL define an `IoError` enum that wraps `VfsError` variants with additional context: the operation phase (open, read-chunk, write-chunk, flush, rename, cleanup), the resource URI, and the bytes transferred before failure. [WB]
2. EACH `IoError` variant SHALL carry sufficient context to produce a diagnostic message conforming to the logging standard: `[background-io] phase: description (uri: resource_uri, transferred: N bytes)`. [WB]
3. WHEN a LoadTask encounters a VFS error during chunk reading, THE task SHALL transition to a failed state, preserve any content already delivered to the document model, and propagate the IoError through the IoTaskHandle. [SCI-STE-IO]
4. WHEN a SaveTask encounters a VFS error, THE task SHALL attempt to delete the temporary file, preserve the original file unmodified, transition to a failed state, and propagate the IoError through the IoTaskHandle. [SCI-STE-IO]
5. THE IoTaskHandle SHALL expose a `result()` method that returns `Result<IoSuccess, IoError>` once the task reaches a terminal state, enabling the caller to inspect the outcome and decide on recovery action. [WB]
6. THE BackgroundIoService SHALL log all I/O errors at ERROR level via the logging subsystem, including the full error chain (VfsError → IoError → context), before propagating to the caller. [WB]
7. IF a transient error occurs during a LoadTask (e.g., network timeout from a remote VFS provider), THE BackgroundIoService SHALL support a configurable retry policy: retry the failed chunk up to N times (default 3) with exponential backoff (starting at 500ms), before declaring failure. [SCI-STE-IO]
8. WHEN a retry succeeds after a transient failure, THE LoadTask SHALL continue from the last successfully delivered position — it SHALL NOT restart from the beginning of the file. [SCI-STE-IO]
9. THE BackgroundIoService SHALL emit a WARN-level log record for each retry attempt, including the retry count, the error that triggered the retry, and the backoff duration. [WB]

---

### Requirement 7: Concurrency and Task Management

**User Story:** As a developer, I want the background I/O service to manage concurrent operations safely, so that multiple files can be loaded or saved simultaneously without resource exhaustion or data races.

**Source:** [WB], [SCI-STE-IO]

#### Acceptance Criteria

1. THE BackgroundIoService SHALL enforce a configurable maximum number of concurrent I/O tasks (default 4), queuing additional tasks until a slot becomes available. [WB]
2. THE maximum concurrent tasks setting SHALL be configurable via the workbench configuration-system (`io.max_concurrent_tasks`), with a minimum of 1 and maximum of 16. [WB]
3. WHEN the concurrency limit is reached and a new task is requested, THE BackgroundIoService SHALL enqueue the task and return the IoTaskHandle immediately — the task SHALL begin execution when a slot becomes available (FIFO ordering). [WB]
4. THE BackgroundIoService SHALL be thread-safe: task submission, progress queries, and cancellation SHALL be safe to invoke from any thread (GUI thread, Tokio workers, plugin threads) without external synchronization. [WB]
5. THE BackgroundIoService SHALL track all active and queued tasks, exposing a method to list current tasks with their states (queued, in-progress, complete, failed, cancelled) for display in a task manager UI. [SCI-STE-IO]
6. WHEN the application is shutting down, THE BackgroundIoService SHALL cancel all in-progress LoadTasks (via CancellationToken), await completion of all in-progress SaveTasks (to avoid data loss), and drain the queue, within a configurable shutdown timeout (default 30 seconds). [WB]
7. IF the shutdown timeout expires with SaveTasks still running, THE BackgroundIoService SHALL log an ERROR-level record for each incomplete save and allow process termination — it SHALL NOT block indefinitely. [WB]
8. THE BackgroundIoService SHALL be a singleton service registered with platform-core, accessible to all subsystems (file-operations, document-model, plugins) via dependency injection. [WB]

---

### Requirement 8: Integration with VFS Provider Async Interface

**User Story:** As a developer, I want background-io to use the VFS provider's async interface directly, so that all I/O operations benefit from provider-specific optimizations (buffering, connection pooling, caching) without background-io reimplementing them.

**Source:** [WB], [FFW-ARCH-001]

#### Acceptance Criteria

1. THE LoadTask SHALL obtain an async read stream by calling the VFS provider's `read_stream(path)` method, consuming the `impl AsyncRead` returned by the provider. [WB]
2. THE SaveTask SHALL write content by calling the VFS provider's `write(path, data)` or `open(path, write_options)` method, using the provider's write interface rather than constructing raw I/O operations. [WB]
3. THE BackgroundIoService SHALL obtain VFS provider references through the VFS Provider_Registry, never by constructing provider instances directly — ensuring all provider lifecycle management flows through the VFS layer. [FFW-ARCH-001]
4. WHEN a VFS provider supports random-access reads (as declared by its capabilities), THE LoadTask MAY use seek-based access for partial file loading — reading only the portions needed for viewport display rather than streaming the entire file. [SCI-STE-IO]
5. THE BackgroundIoService SHALL respect provider capability declarations: before initiating a save, it SHALL verify that the provider supports write operations; before initiating a watch-integrated load, it SHALL verify watch capability. [WB]
6. IF the VFS provider returns a `VfsError::Timeout` during an async operation, THE BackgroundIoService SHALL treat it as a transient error eligible for retry per the policy defined in Requirement 6 criterion 7. [WB]
7. THE BackgroundIoService SHALL pass-through all provider-specific metadata (encoding hints, record format, line-ending type) returned by VFS operations to the document model, without interpreting or modifying them. [WB]
8. ALL resource identifiers used by background-io SHALL be `ResourceUri` values (as defined by the VFS spec), ensuring consistent addressing across the entire I/O pipeline. [FFW-ARCH-001]

---

## Cross-References

- **`virtual-file-system`**: Background-io is the primary consumer of the VFS async provider interface (`read_stream`, `write`, `stat`, `rename`). All I/O operations are routed through VFS; background-io never performs platform-specific file access. [FFW-ARCH-001]
- **`document-model`**: Background-io delivers loaded content to the document model's streaming interface (Requirement 4: Streaming File Loading). The document model defines the chunk-delivery API; background-io drives it with data from the VFS. [FFE-MVP-1]
- **`file-operations`**: The file-operations spec defines the user-facing commands (Open, Save, Save As, Revert). File-operations invokes background-io to perform the actual async I/O work. Background-io is the execution layer; file-operations is the coordination layer. [SCI-STE-IO]
- **`logging-subsystem`**: Background-io uses `ff-logging` for all diagnostic output (error logging, retry warnings, performance metrics). Log records follow the workbench logging format and are prefixed with `[background-io]`. [WB]
- **`workflow-engine`**: Long-running I/O operations (especially large-file loads/saves) may be modelled as workflows when they require user interaction (e.g., encoding selection dialogs, overwrite confirmation). Background-io provides the execution primitives; workflow-engine provides the orchestration. [WB]
- **`configuration-system`**: Background-io reads its configuration from the workbench config: `io.chunk_size_kb`, `io.large_file_threshold_mb`, `io.max_concurrent_tasks`, `io.retry_count`, `io.retry_backoff_ms`. [WB]
- **`large-file-performance`**: The large-file-performance spec defines rendering optimizations for files loaded via background-io's streaming mode (chunked rendering, measurement caching). Background-io provides the async loading; large-file-performance handles display-side performance. [SCI-STE-IO]
- **`external-modification`**: When the VFS file-watcher detects external changes, external-modification may trigger a background-io reload. Background-io provides the async reload mechanism; external-modification decides when to invoke it. [SCI-STE-IO]
