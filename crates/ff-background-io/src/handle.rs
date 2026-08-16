//! IoTaskHandle — the user-facing handle for querying progress, cancelling,
//! and awaiting completion of background I/O tasks.
//!
//! Returned immediately when a task is spawned via `BackgroundIoService::spawn_load`
//! or `BackgroundIoService::spawn_save`. Cloneable — multiple consumers can observe
//! the same task.

use std::sync::Arc;

use tokio::sync::{watch, Notify, RwLock};

use crate::cancellation::IoCancellationToken;
use crate::error::IoError;
use crate::progress::ProgressState;
use crate::types::{IoSuccess, TaskId, TaskState};

/// Type alias for the shared result storage.
type SharedResult = Arc<RwLock<Option<Arc<Result<IoSuccess, IoError>>>>>;

/// A handle returned when an I/O task is spawned.
///
/// Provides methods to query progress, cancel, and await completion.
/// Cloneable — multiple consumers can observe the same task.
#[derive(Clone)]
pub struct IoTaskHandle {
    /// Unique task identifier.
    id: TaskId,
    /// Latest progress state (watch channel receiver).
    progress_rx: watch::Receiver<ProgressState>,
    /// Cancellation token for this task.
    cancel_token: IoCancellationToken,
    /// Completion signal (broadcast notification).
    completion: Arc<Notify>,
    /// Final result (populated on terminal state).
    result: SharedResult,
    /// Current task state.
    state: Arc<RwLock<TaskState>>,
}

impl IoTaskHandle {
    /// Create a new IoTaskHandle with the provided components.
    ///
    /// This is a crate-internal constructor used by `BackgroundIoService` when
    /// spawning tasks.
    pub(crate) fn new(
        id: TaskId,
        progress_rx: watch::Receiver<ProgressState>,
        cancel_token: IoCancellationToken,
    ) -> Self {
        Self {
            id,
            progress_rx,
            cancel_token,
            completion: Arc::new(Notify::new()),
            result: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(TaskState::Queued)),
        }
    }

    /// Returns the unique task identifier.
    pub fn id(&self) -> TaskId {
        self.id
    }

    /// Returns the most recent ProgressState without blocking.
    ///
    /// Enables the UI to poll for updates at its own refresh rate.
    pub fn progress(&self) -> ProgressState {
        self.progress_rx.borrow().clone()
    }

    /// Returns an async receiver for reactive progress updates.
    ///
    /// The receiver yields the latest ProgressState whenever it changes.
    /// Uses watch channel semantics — if multiple updates arrive between polls,
    /// only the latest is delivered.
    pub fn subscribe_progress(&self) -> watch::Receiver<ProgressState> {
        self.progress_rx.clone()
    }

    /// Triggers cooperative cancellation. Returns immediately without waiting
    /// for the task to finish.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Awaits the task reaching a terminal state (complete, failed, cancelled).
    pub async fn await_completion(&self) {
        // If already in terminal state, return immediately
        {
            let state = self.state.read().await;
            if matches!(
                *state,
                TaskState::Complete | TaskState::Failed | TaskState::Cancelled
            ) {
                return;
            }
        }
        self.completion.notified().await;
    }

    /// Returns the final result once the task is in a terminal state.
    ///
    /// Returns None if the task is still in progress or queued.
    /// The result is wrapped in Arc since IoError is not Clone.
    pub async fn result(&self) -> Option<Arc<Result<IoSuccess, IoError>>> {
        let guard = self.result.read().await;
        guard.clone()
    }

    /// Returns the current task state (non-blocking via try_read, falls back to Queued).
    pub fn state(&self) -> TaskState {
        match self.state.try_read() {
            Ok(guard) => *guard,
            Err(_) => TaskState::Queued,
        }
    }

    /// Returns whether the task is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state(),
            TaskState::Complete | TaskState::Failed | TaskState::Cancelled
        )
    }

    /// Returns the cancellation token for this task.
    #[allow(dead_code)]
    pub(crate) fn cancel_token(&self) -> &IoCancellationToken {
        &self.cancel_token
    }

    /// Set the task to in-progress state.
    pub(crate) async fn set_in_progress(&self) {
        let mut state = self.state.write().await;
        *state = TaskState::InProgress;
    }

    /// Complete the task with a success result.
    pub(crate) async fn complete_success(&self, success: IoSuccess) {
        {
            let mut result = self.result.write().await;
            *result = Some(Arc::new(Ok(success)));
        }
        {
            let mut state = self.state.write().await;
            *state = TaskState::Complete;
        }
        self.completion.notify_waiters();
    }

    /// Complete the task with a failure result.
    pub(crate) async fn complete_failure(&self, error: IoError) {
        {
            let mut result = self.result.write().await;
            *result = Some(Arc::new(Err(error)));
        }
        {
            let mut state = self.state.write().await;
            *state = TaskState::Failed;
        }
        self.completion.notify_waiters();
    }

    /// Complete the task with a cancelled state.
    pub(crate) async fn complete_cancelled(&self, error: IoError) {
        {
            let mut result = self.result.write().await;
            *result = Some(Arc::new(Err(error)));
        }
        {
            let mut state = self.state.write().await;
            *state = TaskState::Cancelled;
        }
        self.completion.notify_waiters();
    }
}

impl std::fmt::Debug for IoTaskHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IoTaskHandle")
            .field("id", &self.id)
            .field("state", &self.state())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::progress::IoPhase;
    use ff_vfs::ResourceUri;

    fn create_test_handle() -> (IoTaskHandle, watch::Sender<ProgressState>) {
        let (tx, rx) = watch::channel(ProgressState::new_queued());
        let token = IoCancellationToken::new();
        let handle = IoTaskHandle::new(TaskId::new(1), rx, token);
        (handle, tx)
    }

    #[test]
    fn handle_id_returns_correct_value() {
        // Validates: Requirement 2 AC 5
        let (handle, _tx) = create_test_handle();
        assert_eq!(handle.id(), TaskId::new(1));
    }

    #[test]
    fn handle_initial_state_is_queued() {
        // Validates: Requirement 3 AC 5
        let (handle, _tx) = create_test_handle();
        assert_eq!(handle.state(), TaskState::Queued);
    }

    #[test]
    fn handle_progress_returns_latest_state() {
        // Validates: Requirement 2 AC 5
        let (handle, tx) = create_test_handle();

        let new_state = ProgressState {
            bytes_transferred: 1024,
            total_bytes: Some(4096),
            percentage: Some(25),
            elapsed: Duration::from_millis(100),
            estimated_remaining: None,
            phase: IoPhase::Reading,
        };
        tx.send(new_state.clone()).unwrap();

        let got = handle.progress();
        assert_eq!(got.bytes_transferred, 1024);
        assert_eq!(got.percentage, Some(25));
        assert_eq!(got.phase, IoPhase::Reading);
    }

    #[test]
    fn handle_subscribe_returns_receiver() {
        // Validates: Requirement 2 AC 6
        let (handle, tx) = create_test_handle();

        let mut rx = handle.subscribe_progress();
        let new_state = ProgressState {
            bytes_transferred: 2048,
            total_bytes: Some(4096),
            percentage: Some(50),
            elapsed: Duration::from_millis(200),
            estimated_remaining: None,
            phase: IoPhase::Reading,
        };
        tx.send(new_state).unwrap();

        // The receiver should see the update
        assert!(rx.has_changed().unwrap_or(false));
    }

    #[test]
    fn handle_cancel_triggers_cancellation_token() {
        // Validates: Requirement 3 AC 8
        let (handle, _tx) = create_test_handle();
        assert!(!handle.cancel_token().is_cancelled());

        handle.cancel();
        assert!(handle.cancel_token().is_cancelled());
    }

    #[tokio::test]
    async fn handle_await_completion_resolves_on_success() {
        // Validates: Requirement 3 AC 8
        let (handle, _tx) = create_test_handle();

        let handle_clone = handle.clone();
        let join = tokio::spawn(async move {
            handle_clone.await_completion().await;
            handle_clone.state()
        });

        tokio::task::yield_now().await;

        handle
            .complete_success(IoSuccess {
                bytes_transferred: 1000,
                elapsed: Duration::from_millis(500),
                uri: ResourceUri::new("local", "/test.txt"),
            })
            .await;

        let state = join.await.unwrap();
        assert_eq!(state, TaskState::Complete);
    }

    #[tokio::test]
    async fn handle_await_completion_resolves_on_failure() {
        // Validates: Requirement 3 AC 8
        let (handle, _tx) = create_test_handle();

        handle
            .complete_failure(IoError::Timeout {
                uri: "vfs://local/test.txt".to_string(),
                description: "timed out".to_string(),
                bytes_transferred: 0,
            })
            .await;

        // Should not block since already in terminal state
        handle.await_completion().await;
        assert_eq!(handle.state(), TaskState::Failed);
    }

    #[tokio::test]
    async fn handle_result_returns_none_before_completion() {
        // Validates: Requirement 6 AC 5
        let (handle, _tx) = create_test_handle();
        assert!(handle.result().await.is_none());
    }

    #[tokio::test]
    async fn handle_result_returns_success_after_completion() {
        // Validates: Requirement 6 AC 5
        let (handle, _tx) = create_test_handle();

        handle
            .complete_success(IoSuccess {
                bytes_transferred: 5000,
                elapsed: Duration::from_secs(1),
                uri: ResourceUri::new("local", "/data.bin"),
            })
            .await;

        let result = handle.result().await;
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());
    }

    #[tokio::test]
    async fn handle_set_in_progress_transitions_state() {
        // Validates: Requirement 3 AC 5
        let (handle, _tx) = create_test_handle();

        handle.set_in_progress().await;
        assert_eq!(handle.state(), TaskState::InProgress);
    }

    #[tokio::test]
    async fn handle_complete_cancelled_sets_cancelled_state() {
        // Validates: Requirement 3 AC 5
        let (handle, _tx) = create_test_handle();

        handle
            .complete_cancelled(IoError::Cancelled {
                uri: "vfs://local/file.txt".to_string(),
                bytes_transferred: 512,
            })
            .await;

        assert_eq!(handle.state(), TaskState::Cancelled);
        assert!(handle.is_terminal());
    }
}
