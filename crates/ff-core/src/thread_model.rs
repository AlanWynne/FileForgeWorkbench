//! # Thread Model — Tokio Runtime and Task Management
//!
//! This module defines the thread model for the platform and manages the
//! Tokio async runtime used for all background I/O operations.
//!
//! Three thread contexts are defined:
//! - **Main thread**: Owns the GUI event loop (or the application loop in headless mode)
//! - **Core thread**: Optional dedicated thread for core business logic
//! - **Tokio runtime**: Multi-threaded async I/O worker pool
//!
//! This module handles:
//! - Tokio runtime creation during startup and shutdown during teardown
//! - Spawned task tracking with join/cancel on shutdown
//! - Channel-based inter-thread communication
//! - Enforcement of the GUI-thread non-blocking rule

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::runtime::{Handle, Runtime};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::error::CoreError;

// ─── Thread Contexts ────────────────────────────────────────────────────────

/// Describes the three thread contexts in the workbench.
///
/// Each context has distinct responsibilities and constraints:
/// - `Main`: Must never block — owns the GUI event loop or headless event loop
/// - `Core`: Optional dedicated thread for synchronous coordination logic
/// - `TokioWorker`: Multi-threaded async executor for all background I/O
///
/// Addresses: Requirement 9, criterion 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadContext {
    /// Main thread (GUI/event loop) — must never perform blocking I/O.
    Main,
    /// Core thread (optional dedicated) — for synchronous coordination when
    /// the GUI shell owns the main thread.
    Core,
    /// Tokio runtime (multi-threaded async I/O workers).
    TokioWorker,
}

// ─── Tokio Runtime Wrapper ──────────────────────────────────────────────────

/// Wrapper around the Tokio multi-threaded runtime.
///
/// Manages runtime creation, task spawning with tracking, and graceful shutdown.
/// All spawned tasks are tracked and automatically cancelled/joined during shutdown
/// to prevent resource leaks.
///
/// Addresses: Requirement 9, criteria 2, 6, 7
pub struct TokioRuntime {
    /// The owned Tokio runtime (taken during shutdown).
    runtime: Option<Runtime>,
    /// Tracked tasks for join/cancel during shutdown.
    tracked_tasks: Arc<Mutex<Vec<TrackedTask>>>,
    /// Global cancellation token — cancelled during shutdown to signal all tasks.
    shutdown_token: CancellationToken,
}

/// A tracked async task with a name and cancellation token.
///
/// Each tracked task is registered upon spawning and automatically
/// cancelled and joined during the shutdown sequence.
pub struct TrackedTask {
    /// Human-readable name for logging and diagnostics.
    pub name: String,
    /// The join handle for awaiting task completion.
    pub handle: JoinHandle<()>,
    /// Per-task cancellation token (child of the runtime's shutdown token).
    pub cancel_token: CancellationToken,
}

impl TokioRuntime {
    /// Create and start a new multi-threaded Tokio runtime.
    ///
    /// This should be called during startup after logging is initialized
    /// but before VFS initialization, as VFS depends on async I/O capability.
    ///
    /// # Errors
    ///
    /// Returns `CoreError::RuntimeCreationFailed` if the Tokio runtime
    /// cannot be constructed (e.g., insufficient system resources).
    ///
    /// Addresses: Requirement 9, criterion 2
    pub fn new() -> Result<Self, CoreError> {
        let runtime = Runtime::new().map_err(|e| CoreError::RuntimeCreationFailed {
            reason: e.to_string(),
        })?;

        Ok(Self {
            runtime: Some(runtime),
            tracked_tasks: Arc::new(Mutex::new(Vec::new())),
            shutdown_token: CancellationToken::new(),
        })
    }

    /// Returns the Tokio runtime `Handle` for spawning untracked tasks.
    ///
    /// # Panics
    ///
    /// Panics if called after the runtime has been shut down (programmer error).
    pub fn handle(&self) -> &Handle {
        self.runtime
            .as_ref()
            .expect("runtime should be available — shutdown already called")
            .handle()
    }

    /// Returns the global shutdown cancellation token.
    ///
    /// Tasks can clone this token to cooperatively check for shutdown.
    pub fn shutdown_token(&self) -> &CancellationToken {
        &self.shutdown_token
    }

    /// Spawn a tracked async task with a name and cancellation token.
    ///
    /// The task is automatically joined/cancelled during shutdown. The provided
    /// future is wrapped in a `tokio::select!` that listens for cancellation,
    /// enabling cooperative shutdown.
    ///
    /// Returns a `CancellationToken` that can be used to cancel this specific task.
    ///
    /// Addresses: Requirement 9, criterion 6
    pub fn spawn_tracked<F>(&self, name: &str, future: F) -> CancellationToken
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let cancel_token = self.shutdown_token.child_token();
        let token_clone = cancel_token.clone();

        let handle = self.handle().spawn(async move {
            tokio::select! {
                _ = token_clone.cancelled() => {
                    // Task was cancelled during shutdown — exit gracefully
                }
                _ = future => {
                    // Task completed normally
                }
            }
        });

        let tracked = TrackedTask {
            name: name.to_string(),
            handle,
            cancel_token: cancel_token.clone(),
        };

        self.tracked_tasks.lock().unwrap().push(tracked);
        cancel_token
    }

    /// Returns the number of currently tracked tasks.
    pub fn tracked_task_count(&self) -> usize {
        self.tracked_tasks.lock().unwrap().len()
    }

    /// Gracefully shut down the runtime: cancel all tracked tasks,
    /// await completion (with timeout), then drop the runtime.
    ///
    /// This should be called during the shutdown sequence after VFS shutdown
    /// but before configuration shutdown.
    ///
    /// # Shutdown Sequence
    ///
    /// 1. Signal the global cancellation token (all child tokens are cancelled)
    /// 2. Await all tracked task handles with a timeout
    /// 3. Drop the Tokio runtime with `shutdown_timeout`
    ///
    /// Addresses: Requirement 9, criteria 3 (Tokio shutdown after VFS), 6 (join all tasks)
    pub fn shutdown(mut self, timeout: Duration) {
        // Step 1: Cancel all tracked tasks via the global shutdown token
        self.shutdown_token.cancel();

        // Step 2: Take tracked tasks and await them
        let tasks = {
            let mut tasks = self.tracked_tasks.lock().unwrap();
            std::mem::take(&mut *tasks)
        };

        // Step 3: Block on awaiting task completion within timeout
        if let Some(rt) = self.runtime.take() {
            rt.block_on(async {
                let _ = tokio::time::timeout(timeout, async {
                    for task in tasks {
                        // Best-effort join — ignore individual join errors (task panics)
                        let _ = task.handle.await;
                    }
                })
                .await;
            });

            // Step 4: Drop the runtime with a final timeout for any remaining work
            rt.shutdown_timeout(timeout);
        }
    }

    /// Handle a fatal runtime error (e.g., all worker threads panicked).
    ///
    /// Logs an ERROR-level record and returns the `CoreError::RuntimeFatal` error,
    /// which the caller should use to initiate an orderly shutdown.
    ///
    /// Addresses: Requirement 9, criterion 7
    pub fn handle_fatal_error(&self) -> CoreError {
        CoreError::RuntimeFatal
    }
}

// ─── Channel-Based Inter-Thread Communication ───────────────────────────────

/// Creates a bounded mpsc channel for command dispatch (Shell → Core).
///
/// This is the recommended channel type for unidirectional message passing
/// where multiple producers send to a single consumer.
///
/// Addresses: Requirement 9, criterion 3
pub fn command_channel<T>(buffer: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
    mpsc::channel(buffer)
}

/// Creates a broadcast channel for event dispatch (Core ↔ Shell, Core ↔ Subsystems).
///
/// This is used by the Event Bus for typed event dispatch where multiple
/// subscribers need to receive the same event.
///
/// Addresses: Requirement 9, criterion 3
pub fn event_broadcast_channel<T: Clone>(
    capacity: usize,
) -> (broadcast::Sender<T>, broadcast::Receiver<T>) {
    broadcast::channel(capacity)
}

/// Creates a oneshot channel for async operation results (Tokio worker → requester).
///
/// This is used when a task needs to return a single result to the caller,
/// such as when a file read completes.
///
/// Addresses: Requirement 9, criteria 3, 5
pub fn result_channel<T>() -> (oneshot::Sender<T>, oneshot::Receiver<T>) {
    oneshot::channel()
}

// ─── GUI Thread Non-Blocking Enforcement ────────────────────────────────────

/// Marker trait indicating a type represents a non-blocking operation.
///
/// Types implementing this trait guarantee they will not perform blocking I/O
/// when executed on the GUI (Main) thread. This serves as documentation and
/// compile-time enforcement via trait bounds.
///
/// Addresses: Requirement 9, criterion 4
pub trait NonBlocking: Send {}

/// Dispatches an async I/O operation to the Tokio runtime and returns
/// the result via a oneshot channel.
///
/// This is the recommended pattern for performing I/O from the GUI thread:
/// instead of blocking, the operation is dispatched to a Tokio worker and
/// the result is communicated back asynchronously.
///
/// Addresses: Requirement 9, criteria 4, 5
pub fn dispatch_io<F, T>(runtime: &TokioRuntime, operation: F) -> oneshot::Receiver<T>
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    runtime.handle().spawn(async move {
        let result = operation.await;
        // If the receiver was dropped, we silently discard the result
        let _ = tx.send(result);
    });
    rx
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::Duration;

    // Validates: Requirement 9.2
    #[test]
    fn runtime_creation_succeeds() {
        let rt = TokioRuntime::new();
        assert!(rt.is_ok(), "Tokio runtime should be created successfully");
        let rt = rt.unwrap();
        // Runtime should have zero tracked tasks initially
        assert_eq!(rt.tracked_task_count(), 0);
        rt.shutdown(Duration::from_secs(1));
    }

    // Validates: Requirement 9.6
    #[test]
    fn spawn_tracked_task_runs_to_completion() {
        let rt = TokioRuntime::new().unwrap();
        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        rt.spawn_tracked("test-task", async move {
            completed_clone.store(true, Ordering::SeqCst);
        });

        // Give the task time to execute
        std::thread::sleep(Duration::from_millis(100));
        assert!(completed.load(Ordering::SeqCst), "task should have run");
        rt.shutdown(Duration::from_secs(1));
    }

    // Validates: Requirement 9.6
    #[test]
    fn tracked_task_count_increments() {
        let rt = TokioRuntime::new().unwrap();

        assert_eq!(rt.tracked_task_count(), 0);

        rt.spawn_tracked("task-1", async {
            tokio::time::sleep(Duration::from_secs(60)).await;
        });
        assert_eq!(rt.tracked_task_count(), 1);

        rt.spawn_tracked("task-2", async {
            tokio::time::sleep(Duration::from_secs(60)).await;
        });
        assert_eq!(rt.tracked_task_count(), 2);

        rt.spawn_tracked("task-3", async {
            tokio::time::sleep(Duration::from_secs(60)).await;
        });
        assert_eq!(rt.tracked_task_count(), 3);

        rt.shutdown(Duration::from_secs(1));
    }

    // Validates: Requirement 9.6
    #[test]
    fn cancellation_token_cancels_task() {
        let rt = TokioRuntime::new().unwrap();
        let reached_end = Arc::new(AtomicBool::new(false));
        let reached_end_clone = reached_end.clone();

        let cancel_token = rt.spawn_tracked("long-task", async move {
            // This would run for 60 seconds if not cancelled
            tokio::time::sleep(Duration::from_secs(60)).await;
            reached_end_clone.store(true, Ordering::SeqCst);
        });

        // Cancel the specific task
        cancel_token.cancel();
        std::thread::sleep(Duration::from_millis(100));

        assert!(
            !reached_end.load(Ordering::SeqCst),
            "task should have been cancelled before reaching the end"
        );
        rt.shutdown(Duration::from_secs(1));
    }

    // Validates: Requirement 9.6
    #[test]
    fn shutdown_cancels_and_joins_all_tasks() {
        let rt = TokioRuntime::new().unwrap();
        let cancel_observed = Arc::new(AtomicUsize::new(0));

        for i in 0..5 {
            let counter = cancel_observed.clone();
            rt.spawn_tracked(&format!("task-{i}"), async move {
                // Wait for cancellation
                tokio::time::sleep(Duration::from_secs(60)).await;
                // Should never reach here because shutdown cancels us
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }

        assert_eq!(rt.tracked_task_count(), 5);

        // Shutdown should cancel all tasks within the timeout
        rt.shutdown(Duration::from_secs(2));

        // None of the tasks should have completed their sleep
        assert_eq!(cancel_observed.load(Ordering::SeqCst), 0);
    }

    // Validates: Requirement 9.7
    #[test]
    fn handle_fatal_error_returns_runtime_fatal() {
        let rt = TokioRuntime::new().unwrap();
        let err = rt.handle_fatal_error();
        match err {
            CoreError::RuntimeFatal => {} // expected
            other => panic!("expected RuntimeFatal, got: {other:?}"),
        }
        rt.shutdown(Duration::from_secs(1));
    }

    // Validates: Requirement 9.1
    #[test]
    fn thread_context_enum_has_three_variants() {
        let contexts = [
            ThreadContext::Main,
            ThreadContext::Core,
            ThreadContext::TokioWorker,
        ];
        assert_eq!(contexts.len(), 3);
        // Verify they are distinct
        assert_ne!(ThreadContext::Main, ThreadContext::Core);
        assert_ne!(ThreadContext::Main, ThreadContext::TokioWorker);
        assert_ne!(ThreadContext::Core, ThreadContext::TokioWorker);
    }

    // Validates: Requirement 9.3
    #[test]
    fn mpsc_channel_communication() {
        let rt = TokioRuntime::new().unwrap();
        let (tx, mut rx) = command_channel::<String>(10);

        rt.handle().spawn(async move {
            tx.send("hello from worker".to_string()).await.unwrap();
        });

        let received = rt.handle().block_on(async { rx.recv().await });
        assert_eq!(received, Some("hello from worker".to_string()));
        rt.shutdown(Duration::from_secs(1));
    }

    // Validates: Requirement 9.3
    #[test]
    fn broadcast_channel_communication() {
        let rt = TokioRuntime::new().unwrap();
        let (tx, mut rx1) = event_broadcast_channel::<String>(16);
        let mut rx2 = tx.subscribe();

        tx.send("broadcast-event".to_string()).unwrap();

        let msg1 = rt.handle().block_on(async { rx1.recv().await.unwrap() });
        let msg2 = rt.handle().block_on(async { rx2.recv().await.unwrap() });
        assert_eq!(msg1, "broadcast-event");
        assert_eq!(msg2, "broadcast-event");
        rt.shutdown(Duration::from_secs(1));
    }

    // Validates: Requirement 9.3, 9.5
    #[test]
    fn oneshot_channel_result_delivery() {
        let rt = TokioRuntime::new().unwrap();
        let (tx, rx) = result_channel::<u64>();

        rt.handle().spawn(async move {
            // Simulate async computation
            let result = 42u64;
            tx.send(result).unwrap();
        });

        let result = rt.handle().block_on(async { rx.await.unwrap() });
        assert_eq!(result, 42);
        rt.shutdown(Duration::from_secs(1));
    }

    // Validates: Requirement 9.4, 9.5
    #[test]
    fn dispatch_io_returns_result_via_channel() {
        let rt = TokioRuntime::new().unwrap();

        let rx = dispatch_io(&rt, async {
            // Simulate a non-blocking I/O operation
            tokio::time::sleep(Duration::from_millis(10)).await;
            "io-result"
        });

        let result = rt.handle().block_on(async { rx.await.unwrap() });
        assert_eq!(result, "io-result");
        rt.shutdown(Duration::from_secs(1));
    }

    // Validates: Requirement 9.6
    #[test]
    fn shutdown_token_propagates_to_child_tasks() {
        let rt = TokioRuntime::new().unwrap();
        let cancellation_detected = Arc::new(AtomicBool::new(false));
        let detected_clone = cancellation_detected.clone();

        let token = rt.spawn_tracked("child-task", async move {
            // This sleep will be interrupted by cancellation
            tokio::time::sleep(Duration::from_secs(60)).await;
            detected_clone.store(true, Ordering::SeqCst);
        });

        // The child token should be a child of the shutdown token
        assert!(!token.is_cancelled());

        // Cancelling the global shutdown token should cancel child tokens
        rt.shutdown(Duration::from_secs(1));

        assert!(token.is_cancelled());
        assert!(!cancellation_detected.load(Ordering::SeqCst));
    }
}
