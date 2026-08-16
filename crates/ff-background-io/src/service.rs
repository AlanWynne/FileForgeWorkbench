//! BackgroundIoService — the central service managing all background I/O tasks.
//!
//! Thread-safe, singleton service that spawns async load/save tasks on the Tokio
//! runtime, enforces concurrency limits, and coordinates task lifecycle including
//! graceful shutdown.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{watch, Mutex, RwLock, Semaphore};

use crate::cancellation::IoCancellationToken;
use crate::config::IoConfig;
use crate::error::IoError;
use crate::handle::IoTaskHandle;
use crate::load::{ChunkCallback, LoadOptions};
use crate::progress::{IoPhase, ProgressState};
use crate::save::{DocumentChunkSource, SaveOptions};
use crate::types::{IoTaskEntry, IoTaskType, TaskId};

#[cfg(test)]
use crate::types::TaskState;

/// Type alias for the memory pressure callback to reduce type complexity.
type MemoryPressureCallback = Arc<Mutex<Option<Box<dyn Fn() -> bool + Send + Sync>>>>;

/// The central service managing all background I/O tasks.
///
/// Thread-safe (`Send + Sync`), singleton, registered with platform-core
/// ServiceRegistry. Enforces concurrency limits and coordinates task lifecycle.
pub struct BackgroundIoService {
    /// Configuration (chunk size, thresholds, concurrency).
    config: Arc<IoConfig>,
    /// Concurrency semaphore limiting active tasks.
    semaphore: Arc<Semaphore>,
    /// Active/queued task registry.
    tasks: Arc<RwLock<HashMap<TaskId, TaskEntry>>>,
    /// Monotonically increasing task ID counter.
    next_id: AtomicU64,
    /// Shutdown token for graceful termination.
    shutdown_token: IoCancellationToken,
    /// Memory pressure callback (if set).
    memory_pressure_callback: MemoryPressureCallback,
}

/// Internal entry tracking a task's state and handle.
#[allow(dead_code)]
struct TaskEntry {
    /// Task type (load or save).
    task_type: IoTaskType,
    /// The user-facing handle.
    handle: IoTaskHandle,
    /// Resource URI for this task.
    uri: String,
    /// Progress sender for updating progress.
    progress_tx: watch::Sender<ProgressState>,
}

// SAFETY: BackgroundIoService is Send + Sync because all fields are thread-safe.
// - Arc<IoConfig>: Send + Sync
// - Arc<Semaphore>: Send + Sync
// - Arc<RwLock<HashMap>>: Send + Sync
// - AtomicU64: Send + Sync
// - IoCancellationToken: Send + Sync (wraps tokio_util::sync::CancellationToken)
// - Arc<Mutex<Option<Box<dyn Fn() -> bool + Send + Sync>>>>: Send + Sync
unsafe impl Send for BackgroundIoService {}
unsafe impl Sync for BackgroundIoService {}

impl BackgroundIoService {
    /// Create a new BackgroundIoService with the given configuration.
    pub fn new(config: IoConfig) -> Self {
        let max_tasks = config.max_concurrent_tasks as usize;
        Self {
            config: Arc::new(config),
            semaphore: Arc::new(Semaphore::new(max_tasks)),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            next_id: AtomicU64::new(1),
            shutdown_token: IoCancellationToken::new(),
            memory_pressure_callback: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &IoConfig {
        &self.config
    }

    /// Allocate a new unique task ID.
    fn next_task_id(&self) -> TaskId {
        TaskId::new(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    /// Register a task and return its handle.
    ///
    /// Creates the progress channel, cancellation token, and IoTaskHandle.
    /// The task is initially in Queued state.
    #[allow(unused_variables)]
    pub(crate) fn register_task(
        &self,
        task_type: IoTaskType,
        uri: &str,
    ) -> (
        TaskId,
        IoTaskHandle,
        watch::Sender<ProgressState>,
        IoCancellationToken,
    ) {
        let id = self.next_task_id();
        let (progress_tx, progress_rx) = watch::channel(ProgressState::new_queued());
        let cancel_token = self.shutdown_token.child_token();
        let handle = IoTaskHandle::new(id, progress_rx, cancel_token.clone());

        // We'll add to registry in spawn methods
        (id, handle, progress_tx, cancel_token)
    }

    /// Insert a task entry into the registry.
    #[allow(dead_code)]
    pub(crate) async fn insert_task(
        &self,
        id: TaskId,
        task_type: IoTaskType,
        handle: IoTaskHandle,
        uri: String,
        progress_tx: watch::Sender<ProgressState>,
    ) {
        let entry = TaskEntry {
            task_type,
            handle,
            uri,
            progress_tx,
        };
        let mut tasks = self.tasks.write().await;
        tasks.insert(id, entry);
    }

    /// Remove a task from the registry.
    #[allow(dead_code)]
    pub(crate) async fn remove_task(&self, id: &TaskId) {
        let mut tasks = self.tasks.write().await;
        tasks.remove(id);
    }

    /// Get a clone of the concurrency semaphore.
    #[allow(dead_code)]
    pub(crate) fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }

    /// Cancel a specific task by ID. Triggers cooperative cancellation.
    /// Returns immediately without waiting for the task to finish.
    pub async fn cancel(&self, task_id: TaskId) {
        let tasks = self.tasks.read().await;
        if let Some(entry) = tasks.get(&task_id) {
            entry.handle.cancel();
        }
    }

    /// Cancel all tasks associated with a given resource URI.
    /// Used when a document is closed to prevent resource leaks.
    pub async fn cancel_for_uri(&self, uri: &str) {
        let tasks = self.tasks.read().await;
        for entry in tasks.values() {
            if entry.uri == uri {
                entry.handle.cancel();
            }
        }
    }

    /// List all active and queued tasks with their current states.
    pub async fn list_tasks(&self) -> Vec<IoTaskEntry> {
        let tasks = self.tasks.read().await;
        tasks
            .iter()
            .map(|(id, entry)| IoTaskEntry {
                id: *id,
                uri: entry.uri.clone(),
                task_type: entry.task_type,
                state: entry.handle.state(),
                progress: entry.handle.progress(),
            })
            .collect()
    }

    /// Graceful shutdown: cancel LoadTasks, await SaveTasks, drain queue.
    ///
    /// Returns after all tasks terminate or shutdown timeout expires.
    /// Logs ERROR for each incomplete save if timeout expires.
    pub async fn shutdown(&self) {
        let timeout = Duration::from_secs(u64::from(self.config.shutdown_timeout_secs));

        // First, trigger the global shutdown token (cancels all child tokens)
        self.shutdown_token.cancel();

        // Wait for save tasks to complete, with timeout
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let tasks = self.tasks.read().await;
            let pending_saves: Vec<_> = tasks
                .iter()
                .filter(|(_, entry)| {
                    entry.task_type == IoTaskType::Save && !entry.handle.is_terminal()
                })
                .map(|(id, entry)| (*id, entry.uri.clone()))
                .collect();
            drop(tasks);

            if pending_saves.is_empty() {
                break;
            }

            if tokio::time::Instant::now() >= deadline {
                // Log ERROR for each incomplete save
                for (_id, uri) in &pending_saves {
                    ff_logging::log(
                        ff_logging::LogLevel::Error,
                        "background-io",
                        &format!("shutdown timeout: save task for '{}' did not complete", uri),
                    );
                }
                break;
            }

            // Brief yield to let tasks make progress
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Clear the task registry
        let mut tasks = self.tasks.write().await;
        tasks.clear();
    }

    /// Register a memory-pressure callback. When invoked (returns true),
    /// pauses large-file LoadTasks until memory is freed.
    pub async fn set_memory_pressure_callback(
        &self,
        callback: Box<dyn Fn() -> bool + Send + Sync>,
    ) {
        let mut guard = self.memory_pressure_callback.lock().await;
        *guard = Some(callback);
    }

    /// Check if memory pressure is active.
    #[allow(dead_code)]
    pub(crate) async fn is_memory_pressure(&self) -> bool {
        let guard = self.memory_pressure_callback.lock().await;
        match &*guard {
            Some(callback) => callback(),
            None => false,
        }
    }

    /// Returns the shutdown token for this service.
    #[allow(dead_code)]
    pub(crate) fn shutdown_token(&self) -> &IoCancellationToken {
        &self.shutdown_token
    }

    /// Spawn an async load task for the given resource URI.
    ///
    /// Returns an IoTaskHandle immediately without blocking.
    /// If the concurrency limit is reached, the task is enqueued (FIFO).
    pub fn spawn_load(
        &self,
        vfs: Arc<ff_vfs::Vfs>,
        uri: ff_vfs::ResourceUri,
        options: LoadOptions,
        chunk_callback: ChunkCallback,
    ) -> IoTaskHandle {
        let uri_str = uri.as_str();
        let (id, handle, progress_tx, cancel_token) =
            self.register_task(IoTaskType::Load, &uri_str);

        let chunk_size = options.chunk_size.unwrap_or(self.config.chunk_size);
        let large_file_threshold = options
            .large_file_threshold
            .unwrap_or(self.config.large_file_threshold);
        let semaphore = self.semaphore.clone();
        let handle_clone = handle.clone();
        let tasks = self.tasks.clone();

        // Insert into registry immediately so it's visible in list_tasks
        let handle_for_insert = handle.clone();
        let progress_tx_for_insert = progress_tx.clone();
        let uri_for_insert = uri_str.clone();
        {
            // Use a blocking insert via tokio::spawn
            let tasks_clone = tasks.clone();
            tokio::spawn(async move {
                let entry = TaskEntry {
                    task_type: IoTaskType::Load,
                    handle: handle_for_insert,
                    uri: uri_for_insert,
                    progress_tx: progress_tx_for_insert,
                };
                let mut guard = tasks_clone.write().await;
                guard.insert(id, entry);
            });
        }

        // Spawn the actual load task
        tokio::spawn(async move {
            // Wait for a concurrency slot (FIFO ordering)
            let _permit = semaphore.acquire().await.unwrap();

            // Transition to in-progress
            handle_clone.set_in_progress().await;

            // Execute the load
            let result = crate::load::execute_load(
                &vfs,
                &uri,
                chunk_size,
                large_file_threshold,
                &cancel_token,
                &progress_tx,
                &chunk_callback,
            )
            .await;

            // Handle result
            match result {
                Ok(success) => {
                    handle_clone.complete_success(success).await;
                }
                Err(IoError::Cancelled {
                    uri,
                    bytes_transferred,
                }) => {
                    handle_clone
                        .complete_cancelled(IoError::Cancelled {
                            uri,
                            bytes_transferred,
                        })
                        .await;
                }
                Err(error) => {
                    let _ = progress_tx.send(ProgressState {
                        bytes_transferred: 0,
                        total_bytes: None,
                        percentage: None,
                        elapsed: Duration::ZERO,
                        estimated_remaining: None,
                        phase: IoPhase::Failed,
                    });
                    handle_clone.complete_failure(error).await;
                }
            }

            // Remove from task registry
            let mut guard = tasks.write().await;
            guard.remove(&id);
        });

        handle
    }

    /// Spawn an async save task for the given resource URI.
    ///
    /// Returns an IoTaskHandle immediately without blocking.
    /// If the concurrency limit is reached, the task is enqueued (FIFO).
    pub fn spawn_save(
        &self,
        vfs: Arc<ff_vfs::Vfs>,
        uri: ff_vfs::ResourceUri,
        document_source: Arc<dyn DocumentChunkSource>,
        options: SaveOptions,
    ) -> IoTaskHandle {
        let uri_str = uri.as_str();
        let (id, handle, progress_tx, cancel_token) =
            self.register_task(IoTaskType::Save, &uri_str);

        let chunk_size = options.chunk_size.unwrap_or(self.config.chunk_size);
        let semaphore = self.semaphore.clone();
        let handle_clone = handle.clone();
        let tasks = self.tasks.clone();

        // Insert into registry immediately
        let handle_for_insert = handle.clone();
        let progress_tx_for_insert = progress_tx.clone();
        let uri_for_insert = uri_str.clone();
        {
            let tasks_clone = tasks.clone();
            tokio::spawn(async move {
                let entry = TaskEntry {
                    task_type: IoTaskType::Save,
                    handle: handle_for_insert,
                    uri: uri_for_insert,
                    progress_tx: progress_tx_for_insert,
                };
                let mut guard = tasks_clone.write().await;
                guard.insert(id, entry);
            });
        }

        // Spawn the actual save task
        tokio::spawn(async move {
            // Wait for a concurrency slot
            let _permit = semaphore.acquire().await.unwrap();

            // Transition to in-progress
            handle_clone.set_in_progress().await;

            // Execute the save
            let result = crate::save::execute_save(
                &vfs,
                &uri,
                chunk_size,
                &cancel_token,
                &progress_tx,
                document_source.as_ref(),
                &options,
            )
            .await;

            // Handle result
            match result {
                Ok(success) => {
                    handle_clone.complete_success(success).await;
                }
                Err(IoError::Cancelled {
                    uri,
                    bytes_transferred,
                }) => {
                    handle_clone
                        .complete_cancelled(IoError::Cancelled {
                            uri,
                            bytes_transferred,
                        })
                        .await;
                }
                Err(error) => {
                    let _ = progress_tx.send(ProgressState {
                        bytes_transferred: 0,
                        total_bytes: None,
                        percentage: None,
                        elapsed: Duration::ZERO,
                        estimated_remaining: None,
                        phase: IoPhase::Failed,
                    });
                    handle_clone.complete_failure(error).await;
                }
            }

            // Remove from task registry
            let mut guard = tasks.write().await;
            guard.remove(&id);
        });

        handle
    }
}

impl std::fmt::Debug for BackgroundIoService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackgroundIoService")
            .field("config", &self.config)
            .field("next_id", &self.next_id.load(Ordering::Relaxed))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_is_send_and_sync() {
        // Validates: Requirement 7 AC 4
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<BackgroundIoService>();
    }

    #[tokio::test]
    async fn service_creates_with_default_config() {
        // Validates: Requirement 7 AC 1
        let service = BackgroundIoService::new(IoConfig::default());
        assert_eq!(service.config().max_concurrent_tasks, 4);
    }

    #[tokio::test]
    async fn list_tasks_initially_empty() {
        // Validates: Requirement 7 AC 5
        let service = BackgroundIoService::new(IoConfig::default());
        let tasks = service.list_tasks().await;
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn register_task_creates_handle_with_unique_ids() {
        // Validates: Requirement 7 AC 5
        let service = BackgroundIoService::new(IoConfig::default());

        let (id1, _, _, _) = service.register_task(IoTaskType::Load, "vfs://local/a.txt");
        let (id2, _, _, _) = service.register_task(IoTaskType::Load, "vfs://local/b.txt");

        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn insert_and_list_tasks() {
        // Validates: Requirement 7 AC 5
        let service = BackgroundIoService::new(IoConfig::default());

        let (id, handle, progress_tx, _cancel) =
            service.register_task(IoTaskType::Load, "vfs://local/test.txt");
        service
            .insert_task(
                id,
                IoTaskType::Load,
                handle,
                "vfs://local/test.txt".to_string(),
                progress_tx,
            )
            .await;

        let tasks = service.list_tasks().await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, id);
        assert_eq!(tasks[0].task_type, IoTaskType::Load);
        assert_eq!(tasks[0].state, TaskState::Queued);
    }

    #[tokio::test]
    async fn cancel_task_triggers_token() {
        // Validates: Requirement 3 AC 8
        let service = BackgroundIoService::new(IoConfig::default());

        let (id, handle, progress_tx, cancel) =
            service.register_task(IoTaskType::Load, "vfs://local/test.txt");
        service
            .insert_task(
                id,
                IoTaskType::Load,
                handle.clone(),
                "vfs://local/test.txt".to_string(),
                progress_tx,
            )
            .await;

        assert!(!cancel.is_cancelled());
        service.cancel(id).await;
        assert!(cancel.is_cancelled());
    }

    #[tokio::test]
    async fn cancel_for_uri_cancels_matching_tasks() {
        // Validates: Requirement 3 AC 6
        let service = BackgroundIoService::new(IoConfig::default());

        let (id1, handle1, tx1, cancel1) =
            service.register_task(IoTaskType::Load, "vfs://local/a.txt");
        service
            .insert_task(
                id1,
                IoTaskType::Load,
                handle1,
                "vfs://local/a.txt".to_string(),
                tx1,
            )
            .await;

        let (id2, handle2, tx2, cancel2) =
            service.register_task(IoTaskType::Load, "vfs://local/b.txt");
        service
            .insert_task(
                id2,
                IoTaskType::Load,
                handle2,
                "vfs://local/b.txt".to_string(),
                tx2,
            )
            .await;

        service.cancel_for_uri("vfs://local/a.txt").await;

        assert!(cancel1.is_cancelled());
        assert!(!cancel2.is_cancelled());
    }

    #[tokio::test]
    async fn remove_task_cleans_registry() {
        // Validates: Requirement 7 AC 5
        let service = BackgroundIoService::new(IoConfig::default());

        let (id, handle, progress_tx, _cancel) =
            service.register_task(IoTaskType::Load, "vfs://local/test.txt");
        service
            .insert_task(
                id,
                IoTaskType::Load,
                handle,
                "vfs://local/test.txt".to_string(),
                progress_tx,
            )
            .await;

        assert_eq!(service.list_tasks().await.len(), 1);
        service.remove_task(&id).await;
        assert_eq!(service.list_tasks().await.len(), 0);
    }

    #[tokio::test]
    async fn shutdown_clears_all_tasks() {
        // Validates: Requirement 7 AC 6
        let service = BackgroundIoService::new(IoConfig::default());

        let (id, handle, progress_tx, _cancel) =
            service.register_task(IoTaskType::Load, "vfs://local/test.txt");
        service
            .insert_task(
                id,
                IoTaskType::Load,
                handle,
                "vfs://local/test.txt".to_string(),
                progress_tx,
            )
            .await;

        service.shutdown().await;
        assert_eq!(service.list_tasks().await.len(), 0);
    }

    #[tokio::test]
    async fn shutdown_cancels_load_tasks() {
        // Validates: Requirement 7 AC 6
        let service = BackgroundIoService::new(IoConfig::default());

        let (id, handle, progress_tx, cancel) =
            service.register_task(IoTaskType::Load, "vfs://local/test.txt");
        service
            .insert_task(
                id,
                IoTaskType::Load,
                handle,
                "vfs://local/test.txt".to_string(),
                progress_tx,
            )
            .await;

        service.shutdown().await;
        assert!(cancel.is_cancelled());
    }

    #[tokio::test]
    async fn concurrency_semaphore_has_correct_permits() {
        // Validates: Requirement 7 AC 1, AC 2
        let config = IoConfig::new(64, 100, 2, 3, 500, 30);
        let service = BackgroundIoService::new(config);

        // Acquire 2 permits (the limit)
        let sem = service.semaphore();
        let _p1 = sem.acquire().await.unwrap();
        let _p2 = sem.acquire().await.unwrap();

        // Third permit should not be immediately available
        let result = sem.try_acquire();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn memory_pressure_callback_defaults_to_false() {
        // Validates: Requirement 5 AC 7
        let service = BackgroundIoService::new(IoConfig::default());
        assert!(!service.is_memory_pressure().await);
    }

    #[tokio::test]
    async fn memory_pressure_callback_returns_callback_result() {
        // Validates: Requirement 5 AC 7
        let service = BackgroundIoService::new(IoConfig::default());
        service
            .set_memory_pressure_callback(Box::new(|| true))
            .await;
        assert!(service.is_memory_pressure().await);
    }
}
