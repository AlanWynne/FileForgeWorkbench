# Implementation Plan: Background I/O (`ff-background-io`)

## Overview

This task plan implements the `ff-background-io` crate — the async file loading and saving infrastructure for FileForgeWorkbench. The crate provides chunked streaming reads with progress reporting, cancellable operations, background save with temp-file + atomic rename, and large-file streaming (>100 MB). All I/O flows through the VFS abstraction layer (FFW-ARCH-001); background-io never performs platform-specific file access directly.

**Crate location:** `crates/ff-background-io`
**Upstream dependencies:** `ff-vfs` (VFS provider interface), `ff-logging` (diagnostics), `ff-core` (service registry)
**Downstream consumers:** `file-operations`, `document-model` (chunk delivery), `external-modification` (reload)

---

## Tasks

- [x] 1. Crate scaffolding and core types
  - [x] 1.1 Create `crates/ff-background-io/Cargo.toml` with dependencies (tokio, tokio-util, async-trait, thiserror, ff-vfs, ff-logging, ff-core) and dev-dependencies (proptest, tokio-test, pretty_assertions, tempfile, mockall)
  - [x] 1.2 Create `crates/ff-background-io/src/lib.rs` with crate-level doc comment and public module declarations
  - [x] 1.3 Implement `src/error.rs` — define `IoError` enum wrapping `VfsError` with variants for each operation phase (Open, ReadChunk, WriteChunk, Flush, Rename, Cleanup, DiskSpace, Timeout) including resource URI and bytes transferred context
  - [x] 1.4 Implement Display for `IoError` conforming to format `[background-io] phase: description (uri: resource_uri, transferred: N bytes)`
  - [x] 1.5 Write unit tests for `IoError` Display format compliance
    - Validates: Requirement 6 AC 1, AC 2

- [x] 2. ProgressState, CancellationToken, and IoTaskHandle types
  - [x] 2.1 Implement `src/progress.rs` — define `ProgressState` struct with fields: bytes_transferred, total_bytes (Option), percentage (Option<u8>), elapsed, estimated_remaining (Option), status (enum: Reading, Writing, Finalizing, Cancelled, Failed, Complete)
  - [x] 2.2 Implement exponential moving average rate calculator for estimated time remaining (5-second window, returns None if <2 seconds of data)
  - [x] 2.3 Implement `src/cancellation.rs` — define `CancellationToken` wrapper around `tokio_util::sync::CancellationToken` with `cancel()` and `is_cancelled()` methods
  - [x] 2.4 Implement `src/handle.rs` — define `IoTaskHandle` struct with methods: `progress()` (non-blocking latest state), `subscribe_progress()` (async channel receiver), `cancel()`, `await_completion()`, `result()`, `state()` (queued/in-progress/complete/failed/cancelled)
  - [x] 2.5 Implement latest-value progress channel using `tokio::sync::watch` for non-blocking producer/consumer semantics
  - [x] 2.6 Write unit tests for ProgressState construction, percentage calculation, status transitions, and EMA rate calculation
    - Validates: Requirement 2 AC 1, AC 3, AC 4, AC 5, AC 6, AC 7, AC 8
  - [x] 2.7 Write unit tests for CancellationToken trigger and query
    - Validates: Requirement 3 AC 1, AC 8
  - [x] 2.8 Write unit tests for IoTaskHandle progress polling and subscribe semantics
    - Validates: Requirement 2 AC 5, AC 6; Requirement 3 AC 5

- [x] 3. BackgroundIoService — task management and concurrency
  - [x] 3.1 Implement `src/service.rs` — define `BackgroundIoService` struct with Tokio runtime handle, concurrency semaphore, active task registry, and VFS provider registry reference
  - [x] 3.2 Implement `new()` constructor accepting configuration (max_concurrent_tasks, chunk_size, large_file_threshold) and VFS registry reference
  - [x] 3.3 Implement concurrency limiting with `tokio::sync::Semaphore` — tasks exceeding max_concurrent (default 4, range 1–16) are queued FIFO
  - [x] 3.4 Implement task registry — track all active/queued tasks with their states, expose `list_tasks()` method
  - [x] 3.5 Implement thread-safety: `BackgroundIoService` is `Send + Sync`, all public methods are safe from any thread
  - [x] 3.6 Implement `shutdown()` — cancel all LoadTasks, await all SaveTasks (up to configurable timeout, default 30s), drain queue, log ERROR for incomplete saves
  - [x] 3.7 Write unit tests for concurrency limiting (spawn N+1 tasks with limit N, verify queuing)
    - Validates: Requirement 7 AC 1, AC 2, AC 3, AC 4
  - [x] 3.8 Write unit tests for task registry listing and state tracking
    - Validates: Requirement 7 AC 5
  - [x] 3.9 Write unit tests for shutdown behaviour (LoadTasks cancelled, SaveTasks awaited, timeout handling)
    - Validates: Requirement 7 AC 6, AC 7

- [x] 4. LoadTask — chunked streaming read with progress
  - [x] 4.1 Implement `src/load.rs` — define `LoadTask` struct encapsulating VFS read_stream handle, chunk buffer, progress sender, cancellation token
  - [x] 4.2 Implement `spawn_load()` on BackgroundIoService — query VFS stat for file size, spawn async task, return IoTaskHandle immediately
  - [x] 4.3 Implement chunked reading loop — read configurable chunks (default 64 KB, min 4 KB, max 1 MB, clamped), deliver each chunk to document model callback as it arrives
  - [x] 4.4 Implement progress reporting after each chunk — emit ProgressState via watch channel with latest-value semantics (non-blocking producer)
  - [x] 4.5 Implement completion handling — signal completion on IoTaskHandle, transition to Complete state, release I/O resources (stream handles, buffers)
  - [x] 4.6 Implement unknown-size handling — when VFS stat returns no size, report progress as bytes-loaded without percentage
  - [x] 4.7 Write unit tests for chunked read with mock VFS provider (verify chunk delivery order, progress updates)
    - Validates: Requirement 1 AC 1, AC 2, AC 3, AC 6
  - [x] 4.8 Write unit tests for chunk size clamping (below 4 KB clamped up, above 1 MB clamped down)
    - Validates: Requirement 1 AC 7
  - [x] 4.9 Write unit tests for unknown-size progress reporting
    - Validates: Requirement 1 AC 4, AC 5
  - [x] 4.10 Write unit tests verifying LoadTask uses only VFS interface (no std::fs/tokio::fs)
    - Validates: Requirement 1 AC 8; Requirement 8 AC 1, AC 8

- [x] 5. SaveTask — atomic write with temp-file strategy
  - [x] 5.1 Implement `src/save.rs` — define `SaveTask` struct encapsulating VFS write handle, progress sender, cancellation token, temp file path, target path
  - [x] 5.2 Implement `spawn_save()` on BackgroundIoService — generate temp filename (`{target}.ffwtmp.{random6}`), spawn async task, return IoTaskHandle immediately
  - [x] 5.3 Implement chunked write loop — write document content in configurable chunks, report progress after each chunk
  - [x] 5.4 Implement fsync-equivalent via VFS provider after all content flushed to temp file
  - [x] 5.5 Implement atomic rename — rename temp file over target via VFS provider's `rename` method
  - [x] 5.6 Implement permission preservation — query original file metadata before rename, apply permissions after rename (where platform supports)
  - [x] 5.7 Implement fallback for providers without atomic rename — write-in-place with truncate, log WARN
  - [x] 5.8 Implement failure cleanup — on any error (write, flush, fsync, rename), delete temp file, leave original unmodified, propagate IoError
  - [x] 5.9 Implement disk space pre-check — query VFS for available space before starting write, return IoError if insufficient
  - [x] 5.10 Write unit tests for atomic save flow (write → flush → fsync → rename) with mock VFS
    - Validates: Requirement 4 AC 1, AC 4, AC 5
  - [x] 5.11 Write unit tests for temp file naming pattern and uniqueness
    - Validates: Requirement 4 AC 2
  - [x] 5.12 Write unit tests for save progress reporting
    - Validates: Requirement 4 AC 3
  - [x] 5.13 Write unit tests for failure cleanup (error at each phase, verify temp deleted and original unmodified)
    - Validates: Requirement 4 AC 7
  - [x] 5.14 Write unit tests for permission preservation after rename
    - Validates: Requirement 4 AC 8
  - [x] 5.15 Write unit tests for fallback write-in-place when provider lacks rename capability
    - Validates: Requirement 4 AC 6
  - [x] 5.16 Write unit tests for disk space pre-check
    - Validates: Requirement 4 AC 9
  - [x] 5.17 Write unit tests verifying SaveTask uses only VFS interface
    - Validates: Requirement 4 AC 10; Requirement 8 AC 2, AC 8

- [x] 6. Large-file handling
  - [x] 6.1 Implement large-file threshold detection in `spawn_load()` — when file size exceeds LargeFileThreshold (default 100 MB, configurable 10–4096 MB, clamped), activate streaming-only mode
  - [x] 6.2 Implement streaming-only LoadTask mode — never buffer more than 2× chunk_size of data at any time, deliver chunks progressively for incremental line-index construction
  - [x] 6.3 Implement streaming-only SaveTask mode — read document content in chunks from document model (no single-allocation request), stream directly to VFS write
  - [x] 6.4 Implement progress rate limiting for large files — emit progress events no faster than once per 50 ms to avoid flooding
  - [x] 6.5 Implement memory-pressure callback — register callback with BackgroundIoService; when triggered, pause in-progress large-file LoadTasks until memory freed or user resumes
  - [x] 6.6 Implement below-threshold optimisation — for small files, allow single-operation buffered load if VFS provider supports efficient single-read
  - [x] 6.7 Write unit tests for threshold detection and mode activation
    - Validates: Requirement 5 AC 1, AC 2
  - [x] 6.8 Write unit tests for streaming buffer constraint (verify ≤ 2× chunk_size held at any time)
    - Validates: Requirement 5 AC 1, AC 3
  - [x] 6.9 Write unit tests for progress rate limiting (no events faster than 50 ms apart)
    - Validates: Requirement 5 AC 6
  - [x] 6.10 Write unit tests for memory-pressure pause/resume
    - Validates: Requirement 5 AC 7
  - [x] 6.11 Write unit tests for below-threshold buffered load
    - Validates: Requirement 5 AC 8
  - [x] 6.12 Write unit tests verifying no maximum file size limit imposed
    - Validates: Requirement 5 AC 5

- [x] 7. Cancellation — cooperative cancel and cleanup
  - [x] 7.1 Implement cancellation check in LoadTask read loop — check CancellationToken before each chunk read, stop within one chunk-read cycle + 100ms cleanup
  - [x] 7.2 Implement cancellation in SaveTask — stop writing, delete temp file, transition to cancelled without modifying original
  - [x] 7.3 Implement auto-cancel on document close — BackgroundIoService triggers CancellationToken when owning document is closed
  - [x] 7.4 Ensure cooperative-only cancellation — never use `tokio::task::abort()`; all cleanup code (handle closure, temp file deletion) always executes
  - [x] 7.5 Implement IoTaskHandle cancel/await flow — `cancel()` returns immediately, `await_completion()` waits for terminal state
  - [x] 7.6 Write unit tests for LoadTask cancellation latency (within one chunk + 100ms)
    - Validates: Requirement 3 AC 2, AC 4
  - [x] 7.7 Write unit tests for SaveTask cancellation (temp file deleted, original unmodified)
    - Validates: Requirement 3 AC 3
  - [x] 7.8 Write unit tests for auto-cancel on document close
    - Validates: Requirement 3 AC 6
  - [x] 7.9 Write unit tests for cooperative cancellation (no abort, cleanup always runs)
    - Validates: Requirement 3 AC 7
  - [x] 7.10 Write unit tests for cancel/await_completion separation
    - Validates: Requirement 3 AC 8

- [x] 8. Configuration integration
  - [x] 8.1 Implement `src/config.rs` — define `IoConfig` struct with fields: chunk_size_kb (u32, default 64, range 4–1024), large_file_threshold_mb (u32, default 100, range 10–4096), max_concurrent_tasks (u8, default 4, range 1–16), retry_count (u8, default 3), retry_backoff_ms (u64, default 500), shutdown_timeout_secs (u32, default 30)
  - [x] 8.2 Implement configuration loading from workbench configuration-system — read from `io.*` namespace
  - [x] 8.3 Implement value clamping for all configurable fields (values outside valid range clamped to nearest bound)
  - [x] 8.4 Write unit tests for config defaults, clamping behaviour, and deserialization
    - Validates: Requirement 1 AC 7; Requirement 5 AC 2; Requirement 7 AC 2

- [x] 9. Error propagation and retry logic
  - [x] 9.1 Implement retry policy in LoadTask — on transient VFS errors (Timeout, network errors), retry failed chunk up to N times (default 3) with exponential backoff starting at 500ms
  - [x] 9.2 Implement resume-from-position on retry success — continue from last delivered chunk, never restart from beginning
  - [x] 9.3 Implement error logging — log all I/O errors at ERROR level via ff-logging with full error chain (VfsError → IoError → context)
  - [x] 9.4 Implement retry logging — log WARN for each retry attempt with retry count, error, and backoff duration
  - [x] 9.5 Implement IoTaskHandle `result()` method — returns `Result<IoSuccess, IoError>` on terminal state
  - [x] 9.6 Implement partial content preservation on LoadTask failure — content already delivered to document model is preserved
  - [x] 9.7 Write unit tests for retry with exponential backoff (verify timing and retry count)
    - Validates: Requirement 6 AC 7
  - [x] 9.8 Write unit tests for resume-from-position after transient error
    - Validates: Requirement 6 AC 8
  - [x] 9.9 Write unit tests for error logging format compliance
    - Validates: Requirement 6 AC 6, AC 9
  - [x] 9.10 Write unit tests for partial content preservation on failure
    - Validates: Requirement 6 AC 3
  - [x] 9.11 Write unit tests for IoTaskHandle result() on success and failure
    - Validates: Requirement 6 AC 5

- [x] 10. VFS integration and provider interaction
  - [x] 10.1 Implement VFS provider resolution — obtain provider references through VFS ProviderRegistry, never construct directly
  - [x] 10.2 Implement capability verification before operations — check write capability before save, watch capability before watch-integrated load
  - [x] 10.3 Implement VfsError::Timeout as transient error eligible for retry
  - [x] 10.4 Implement metadata pass-through — forward all provider-specific metadata (encoding hints, record format, line-ending type) to document model without modification
  - [x] 10.5 Implement random-access read support — when provider declares random_access capability, use seek-based partial loading for viewport-only reads
  - [x] 10.6 Write unit tests for provider resolution via registry
    - Validates: Requirement 8 AC 3
  - [x] 10.7 Write unit tests for capability verification (write check before save, watch check)
    - Validates: Requirement 8 AC 5
  - [x] 10.8 Write unit tests for timeout-as-transient handling
    - Validates: Requirement 8 AC 6
  - [x] 10.9 Write unit tests for metadata pass-through
    - Validates: Requirement 8 AC 7
  - [x] 10.10 Write unit tests for random-access partial read when capability present
    - Validates: Requirement 8 AC 4

- [x] 11. Subsystem registration with platform-core
  - [x] 11.1 Implement `src/subsystem.rs` — define `BackgroundIoSubsystem` implementing ff-core `Subsystem` trait
  - [x] 11.2 Implement `initialize()` — create BackgroundIoService, register as singleton with ServiceRegistry
  - [x] 11.3 Implement `shutdown()` — delegate to BackgroundIoService::shutdown(), log completion
  - [x] 11.4 Write integration test: subsystem initializes, registers service, shuts down cleanly
    - Validates: Requirement 7 AC 8

- [x] 12. Property-based tests
  - [x] 12.1 Write property test: chunk size clamping (Property 1) — generate arbitrary u32 values, verify clamped to [4 KB, 1 MB] range
    - Validates: Requirement 1 AC 7; Requirement 5 AC 2
  - [x] 12.2 Write property test: progress percentage invariant (Property 2) — for any bytes_transferred ≤ total_bytes, percentage == (bytes_transferred * 100) / total_bytes and is in [0, 100]
    - Validates: Requirement 2 AC 3
  - [x] 12.3 Write property test: progress monotonicity (Property 3) — bytes_transferred in successive ProgressState events is non-decreasing within a single task
    - Validates: Requirement 2 AC 1
  - [x] 12.4 Write property test: cancellation bounded latency (Property 4) — cancel at arbitrary point during chunked read, verify task terminates within one chunk + 100ms
    - Validates: Requirement 3 AC 4
  - [x] 12.5 Write property test: atomic save integrity (Property 5) — simulate crash at arbitrary write position, verify original file is never partially written (either old content or new content, never mixed)
    - Validates: Requirement 4 AC 1, AC 7
  - [x] 12.6 Write property test: temp file name uniqueness (Property 6) — generate N concurrent save tasks for same target, verify all temp file names are distinct
    - Validates: Requirement 4 AC 2
  - [x] 12.7 Write property test: large-file buffer bound (Property 7) — for files above threshold, verify buffered data never exceeds 2× chunk_size
    - Validates: Requirement 5 AC 1
  - [x] 12.8 Write property test: retry backoff timing (Property 8) — for N retries, verify backoff durations follow exponential pattern (500ms, 1000ms, 2000ms, ...)
    - Validates: Requirement 6 AC 7
  - [x] 12.9 Write property test: concurrency limit enforcement (Property 9) — spawn M tasks with limit N < M, verify at most N tasks execute simultaneously
    - Validates: Requirement 7 AC 1, AC 3
  - [x] 12.10 Write property test: error format compliance (Property 10) — generate all IoError variants with arbitrary context, verify Display starts with `[background-io]` and includes phase, URI, and bytes transferred
    - Validates: Requirement 6 AC 1, AC 2

- [x] 13. Integration tests
  - [x] 13.1 Write integration test: full load lifecycle — spawn load with mock VFS, verify chunks arrive in order, progress updates received, completion signalled
    - Validates: Requirement 1 AC 1–6; Requirement 2 AC 1, AC 8
  - [x] 13.2 Write integration test: full save lifecycle — spawn save with mock VFS, verify temp write → fsync → rename sequence, progress, and completion
    - Validates: Requirement 4 AC 1, AC 3–5
  - [x] 13.3 Write integration test: concurrent load and save — spawn multiple loads and saves within concurrency limit, verify all complete without interference
    - Validates: Requirement 7 AC 1, AC 4
  - [x] 13.4 Write integration test: large-file streaming load — create mock 200 MB stream, verify memory usage stays within 2× chunk_size bound, progress rate limited to ≥50ms intervals
    - Validates: Requirement 5 AC 1, AC 3, AC 6
  - [x] 13.5 Write integration test: cancel during load — start load, cancel mid-stream, verify partial content preserved, cancelled state reported, resources released
    - Validates: Requirement 3 AC 2, AC 5; Requirement 6 AC 3
  - [x] 13.6 Write integration test: cancel during save — start save, cancel mid-write, verify temp file deleted, original unmodified, cancelled state reported
    - Validates: Requirement 3 AC 3, AC 5
  - [x] 13.7 Write integration test: transient error with retry — mock VFS that fails first 2 reads then succeeds, verify retry with backoff, resume from position, completion
    - Validates: Requirement 6 AC 7, AC 8, AC 9
  - [x] 13.8 Write integration test: shutdown with active tasks — start loads and saves, trigger shutdown, verify loads cancelled, saves awaited (or timed out), queue drained
    - Validates: Requirement 7 AC 6, AC 7
  - [x] 13.9 Write integration test: provider capability check — attempt save on read-only provider, verify IoError before write starts
    - Validates: Requirement 8 AC 5
  - [x] 13.10 Write integration test: end-to-end VFS-only access — verify no std::fs or tokio::fs symbols imported in crate source
    - Validates: Requirement 1 AC 8; Requirement 4 AC 10; Requirement 8 AC 1, AC 2

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Async Streaming File Load | AC 1 (spawn async, return handle) | 4.2, 4.7, 13.1 |
| Req 1: Async Streaming File Load | AC 2 (chunked read, deliver each chunk) | 4.3, 4.7, 13.1 |
| Req 1: Async Streaming File Load | AC 3 (no single contiguous alloc) | 4.3, 4.7 |
| Req 1: Async Streaming File Load | AC 4 (stat for total size) | 4.2, 4.9 |
| Req 1: Async Streaming File Load | AC 5 (unknown size handling) | 4.6, 4.9 |
| Req 1: Async Streaming File Load | AC 6 (completion signal, release resources) | 4.5, 4.7, 13.1 |
| Req 1: Async Streaming File Load | AC 7 (chunk size configurable, clamped) | 4.3, 4.8, 8.1, 12.1 |
| Req 1: Async Streaming File Load | AC 8 (VFS-only, no std::fs) | 4.10, 13.10 |
| Req 2: Progress Reporting | AC 1 (ProgressState after each chunk) | 2.1, 4.4, 13.1 |
| Req 2: Progress Reporting | AC 2 (non-blocking producer) | 2.5, 2.8 |
| Req 2: Progress Reporting | AC 3 (percentage 0–100) | 2.1, 2.6, 12.2 |
| Req 2: Progress Reporting | AC 4 (EMA time remaining) | 2.2, 2.6 |
| Req 2: Progress Reporting | AC 5 (progress() non-blocking poll) | 2.4, 2.8 |
| Req 2: Progress Reporting | AC 6 (subscribe_progress() async channel) | 2.4, 2.8 |
| Req 2: Progress Reporting | AC 7 (status string in ProgressState) | 2.1, 2.6 |
| Req 2: Progress Reporting | AC 8 (terminal state final event) | 2.6, 13.1 |
| Req 3: Cancellation | AC 1 (CancellationToken on handle) | 2.3, 2.7 |
| Req 3: Cancellation | AC 2 (LoadTask stops within one chunk) | 7.1, 7.6, 13.5 |
| Req 3: Cancellation | AC 3 (SaveTask stops, deletes temp) | 7.2, 7.7, 13.6 |
| Req 3: Cancellation | AC 4 (bounded latency: chunk + 100ms) | 7.1, 7.6, 12.4 |
| Req 3: Cancellation | AC 5 (cancelled state with bytes count) | 7.6, 13.5, 13.6 |
| Req 3: Cancellation | AC 6 (auto-cancel on document close) | 7.3, 7.8 |
| Req 3: Cancellation | AC 7 (cooperative only, no abort) | 7.4, 7.9 |
| Req 3: Cancellation | AC 8 (cancel() + await_completion()) | 2.4, 7.5, 7.10 |
| Req 4: Background Save | AC 1 (temp file + atomic rename) | 5.2, 5.5, 5.10, 13.2 |
| Req 4: Background Save | AC 2 (temp name pattern) | 5.2, 5.11, 12.6 |
| Req 4: Background Save | AC 3 (chunked write with progress) | 5.3, 5.12, 13.2 |
| Req 4: Background Save | AC 4 (fsync before rename) | 5.4, 5.10, 13.2 |
| Req 4: Background Save | AC 5 (atomic rename via VFS) | 5.5, 5.10, 13.2 |
| Req 4: Background Save | AC 6 (fallback write-in-place) | 5.7, 5.15 |
| Req 4: Background Save | AC 7 (failure cleanup, original unmodified) | 5.8, 5.13, 12.5 |
| Req 4: Background Save | AC 8 (permission preservation) | 5.6, 5.14 |
| Req 4: Background Save | AC 9 (disk space pre-check) | 5.9, 5.16 |
| Req 4: Background Save | AC 10 (VFS-only, no std::fs) | 5.17, 13.10 |
| Req 5: Large-File Streaming | AC 1 (streaming mode, ≤ 2× chunk buffer) | 6.1, 6.2, 6.7, 6.8, 12.7, 13.4 |
| Req 5: Large-File Streaming | AC 2 (threshold configurable, clamped) | 6.1, 6.7, 8.1, 12.1 |
| Req 5: Large-File Streaming | AC 3 (progressive delivery for line-index) | 6.2, 6.8, 13.4 |
| Req 5: Large-File Streaming | AC 4 (SaveTask streams from doc model) | 6.3 |
| Req 5: Large-File Streaming | AC 5 (no max file size limit) | 6.12 |
| Req 5: Large-File Streaming | AC 6 (progress ≤ once per 50ms) | 6.4, 6.9, 13.4 |
| Req 5: Large-File Streaming | AC 7 (memory-pressure pause) | 6.5, 6.10 |
| Req 5: Large-File Streaming | AC 8 (below-threshold buffered load) | 6.6, 6.11 |
| Req 6: Error Propagation | AC 1 (IoError enum with phase/URI/bytes) | 1.3, 1.5, 12.10 |
| Req 6: Error Propagation | AC 2 (diagnostic message format) | 1.4, 1.5, 12.10 |
| Req 6: Error Propagation | AC 3 (LoadTask preserves partial content) | 9.6, 9.10, 13.5 |
| Req 6: Error Propagation | AC 4 (SaveTask cleanup on error) | 5.8, 5.13 |
| Req 6: Error Propagation | AC 5 (result() method) | 9.5, 9.11 |
| Req 6: Error Propagation | AC 6 (ERROR-level logging) | 9.3, 9.9 |
| Req 6: Error Propagation | AC 7 (retry policy with backoff) | 9.1, 9.7, 12.8, 13.7 |
| Req 6: Error Propagation | AC 8 (resume from position) | 9.2, 9.8, 13.7 |
| Req 6: Error Propagation | AC 9 (WARN-level retry logging) | 9.4, 9.9, 13.7 |
| Req 7: Concurrency and Task Mgmt | AC 1 (max concurrent, default 4) | 3.3, 3.7, 12.9, 13.3 |
| Req 7: Concurrency and Task Mgmt | AC 2 (configurable 1–16) | 3.3, 8.1, 8.4 |
| Req 7: Concurrency and Task Mgmt | AC 3 (FIFO queue, handle returned immediately) | 3.3, 3.7, 12.9 |
| Req 7: Concurrency and Task Mgmt | AC 4 (thread-safe) | 3.5, 3.7, 13.3 |
| Req 7: Concurrency and Task Mgmt | AC 5 (list tasks with states) | 3.4, 3.8 |
| Req 7: Concurrency and Task Mgmt | AC 6 (shutdown: cancel loads, await saves) | 3.6, 3.9, 13.8 |
| Req 7: Concurrency and Task Mgmt | AC 7 (shutdown timeout, ERROR log) | 3.6, 3.9, 13.8 |
| Req 7: Concurrency and Task Mgmt | AC 8 (singleton via platform-core) | 11.1–11.4 |
| Req 8: VFS Integration | AC 1 (read_stream from provider) | 4.1, 4.10, 10.1, 13.10 |
| Req 8: VFS Integration | AC 2 (write via provider) | 5.1, 5.17, 10.1, 13.10 |
| Req 8: VFS Integration | AC 3 (provider via registry) | 10.1, 10.6 |
| Req 8: VFS Integration | AC 4 (random-access partial load) | 10.5, 10.10 |
| Req 8: VFS Integration | AC 5 (capability verification) | 10.2, 10.7, 13.9 |
| Req 8: VFS Integration | AC 6 (Timeout as transient) | 10.3, 10.8 |
| Req 8: VFS Integration | AC 7 (metadata pass-through) | 10.4, 10.9 |
| Req 8: VFS Integration | AC 8 (ResourceUri addressing) | 4.10, 5.17, 10.1 |

---

## Property-Based Test Summary

| Property | Statement | Task | Validates |
|----------|-----------|------|-----------|
| P1 | Chunk size clamping: any u32 input clamped to [4 KB, 1 MB] | 12.1 | Req 1.7, Req 5.2 |
| P2 | Progress percentage invariant: percentage == (bytes_transferred * 100) / total_bytes, always in [0, 100] | 12.2 | Req 2.3 |
| P3 | Progress monotonicity: bytes_transferred is non-decreasing across successive events | 12.3 | Req 2.1 |
| P4 | Cancellation bounded latency: task terminates within one chunk-read duration + 100ms of cancel request | 12.4 | Req 3.4 |
| P5 | Atomic save integrity: simulated crash at any write position leaves original file unchanged | 12.5 | Req 4.1, Req 4.7 |
| P6 | Temp file name uniqueness: N concurrent saves for same target produce N distinct temp names | 12.6 | Req 4.2 |
| P7 | Large-file buffer bound: in streaming mode, buffered data never exceeds 2× chunk_size | 12.7 | Req 5.1 |
| P8 | Retry backoff timing: N retries follow exponential pattern (base × 2^i) | 12.8 | Req 6.7 |
| P9 | Concurrency limit enforcement: at most N tasks execute simultaneously when limit is N | 12.9 | Req 7.1, Req 7.3 |
| P10 | Error format compliance: all IoError Display output starts with `[background-io]` and includes phase, URI, bytes | 12.10 | Req 6.1, Req 6.2 |

---

## Notes

- Tasks 1 and 2 form the foundation and must be completed before any other phase
- Tasks 4 (LoadTask) and 5 (SaveTask) are independent of each other and can be implemented in parallel once tasks 1–3 are done
- Task 6 (large-file) extends tasks 4 and 5, so it depends on both
- Task 7 (cancellation) depends on tasks 4 and 5 since it modifies their read/write loops
- Task 8 (configuration) is independent and can be done as early as after task 1
- Task 10 (VFS integration) depends on tasks 4 and 5 for the concrete implementations to integrate
- All property tests (task 12) depend on the implementation tasks they validate
- Integration tests (task 13) depend on all preceding implementation tasks
- All async tests use `#[tokio::test]` with multi-threaded runtime flavour
- Mock VFS providers for testing should be defined in `tests/mock_provider.rs` shared across test files
- The `proptest` crate is used for all property-based tests with a minimum of 100 iterations

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Crate scaffold, error types, and core types", "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7", "2.8"] },
    { "id": 1, "label": "BackgroundIoService and configuration", "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7", "3.8", "3.9", "8.1", "8.2", "8.3", "8.4"], "dependsOn": [0] },
    { "id": 2, "label": "LoadTask and SaveTask implementation", "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "4.8", "4.9", "4.10", "5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8", "5.9", "5.10", "5.11", "5.12", "5.13", "5.14", "5.15", "5.16", "5.17"], "dependsOn": [1] },
    { "id": 3, "label": "Large-file handling and cancellation", "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "6.8", "6.9", "6.10", "6.11", "6.12", "7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "7.8", "7.9", "7.10"], "dependsOn": [2] },
    { "id": 4, "label": "Error propagation, retry, and VFS integration", "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "9.8", "9.9", "9.10", "9.11", "10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8", "10.9", "10.10"], "dependsOn": [3] },
    { "id": 5, "label": "Subsystem registration", "tasks": ["11.1", "11.2", "11.3", "11.4"], "dependsOn": [4] },
    { "id": 6, "label": "Property-based tests", "tasks": ["12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7", "12.8", "12.9", "12.10"], "dependsOn": [4] },
    { "id": 7, "label": "Integration tests", "tasks": ["13.1", "13.2", "13.3", "13.4", "13.5", "13.6", "13.7", "13.8", "13.9", "13.10"], "dependsOn": [5, 6] }
  ]
}
```
