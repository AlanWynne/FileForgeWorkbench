//! File watcher implementation using the `notify` crate.
//!
//! Provides OS-native file watching with configurable debouncing.
//!
//! Addresses: Requirement 3, all acceptance criteria

pub mod debounce;
pub mod event;
pub mod handle;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{mpsc, RwLock};
use tokio_util::sync::CancellationToken;

use crate::error::map_io_error;
use crate::path::NativePath;
use ff_vfs::watch::WatchEvent;
use ff_vfs::VfsError;

pub use handle::WatchId;

/// Manages OS-native file watching subscriptions with debouncing.
///
/// Uses the `notify` crate internally for cross-platform support.
///
/// Addresses: Requirement 3, all acceptance criteria
pub struct FileWatcher {
    /// Active watch registrations keyed by watch ID.
    watches: Arc<RwLock<HashMap<WatchId, WatchRegistration>>>,
    /// Next watch ID to assign.
    next_id: Arc<std::sync::atomic::AtomicU64>,
    /// Debounce window duration.
    debounce_window: Duration,
    /// Cancellation token for shutdown.
    cancel: CancellationToken,
    /// The underlying notify watcher (behind Arc<Mutex> for thread safety).
    watcher: Arc<std::sync::Mutex<RecommendedWatcher>>,
    /// Background task handle for event processing.
    _event_task: Option<tokio::task::JoinHandle<()>>,
}

/// Internal record of an active watch subscription.
struct WatchRegistration {
    /// The native path being watched.
    path: PathBuf,
    /// Whether this is a recursive watch.
    #[allow(dead_code)]
    recursive: bool,
    /// Sender for delivering events to the consumer.
    sender: mpsc::Sender<WatchEvent>,
    /// Per-path last-event timestamps for debouncing.
    last_events: Arc<std::sync::Mutex<HashMap<PathBuf, Instant>>>,
    /// Cancellation token for this specific watch.
    cancel_token: CancellationToken,
}

impl FileWatcher {
    /// Construct a new `FileWatcher` with the specified debounce window.
    ///
    /// Validates: Requirement 3, criteria 5–7
    pub fn new(debounce_window: Duration) -> Result<Self, VfsError> {
        let watches: Arc<RwLock<HashMap<WatchId, WatchRegistration>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let cancel = CancellationToken::new();

        let watches_clone = Arc::clone(&watches);
        let debounce = debounce_window;
        let cancel_clone = cancel.clone();

        // Create the notify watcher with an event handler
        let (notify_tx, mut notify_rx) = mpsc::channel::<notify::Event>(1024);

        let watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    // Non-blocking send — drop events if buffer full
                    let _ = notify_tx.try_send(event);
                }
            })
            .map_err(|e| VfsError::Io {
                uri: String::new(),
                operation: "init_watcher".to_string(),
                source: std::io::Error::other(e.to_string()),
            })?;

        let watcher = Arc::new(std::sync::Mutex::new(watcher));

        // Spawn background task for processing notify events with debouncing
        let event_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_clone.cancelled() => break,
                    event = notify_rx.recv() => {
                        match event {
                            Some(notify_event) => {
                                Self::process_notify_event(
                                    &watches_clone,
                                    notify_event,
                                    debounce,
                                ).await;
                            }
                            None => break,
                        }
                    }
                }
            }
        });

        Ok(Self {
            watches,
            next_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            debounce_window,
            cancel,
            watcher,
            _event_task: Some(event_task),
        })
    }

    /// Register a watch on a file or directory.
    ///
    /// Returns a `WatchId` and a receiver for watch events.
    ///
    /// Validates: Requirement 3, criteria 1–4, 8
    pub async fn watch(
        &self,
        path: &NativePath,
        recursive: bool,
    ) -> Result<(WatchId, mpsc::Receiver<WatchEvent>), VfsError> {
        let uri = format!("vfs://local{}", path.to_string_lossy().replace('\\', "/"));

        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        // Register with the OS watcher
        {
            let mut watcher = self.watcher.lock().unwrap();
            watcher.watch(path.as_path(), mode).map_err(|e| {
                ff_logging::log_warn!(
                    "[connector-local-fs] watch: OS watch error for {}: {}",
                    uri,
                    e
                );
                map_io_error(std::io::Error::other(e.to_string()), "watch", &uri)
            })?;
        }

        let id = WatchId(
            self.next_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        );

        let (tx, rx) = mpsc::channel(256);
        let cancel_token = CancellationToken::new();

        let registration = WatchRegistration {
            path: path.as_path().to_path_buf(),
            recursive,
            sender: tx,
            last_events: Arc::new(std::sync::Mutex::new(HashMap::new())),
            cancel_token,
        };

        self.watches.write().await.insert(id, registration);

        ff_logging::log_info!(
            "[connector-local-fs] watch: registered watch {} on {} (recursive={})",
            id.0,
            uri,
            recursive
        );

        Ok((id, rx))
    }

    /// Remove a watch by ID, releasing OS resources.
    ///
    /// Validates: Requirement 3, criterion 9
    pub async fn unwatch(&self, id: WatchId) -> Result<(), VfsError> {
        let mut watches = self.watches.write().await;
        if let Some(registration) = watches.remove(&id) {
            registration.cancel_token.cancel();

            // Unwatch from the OS watcher
            let mut watcher = self.watcher.lock().unwrap();
            let _ = watcher.unwatch(registration.path.as_path());

            ff_logging::log_info!(
                "[connector-local-fs] watch: removed watch {} on {}",
                id.0,
                registration.path.display()
            );

            Ok(())
        } else {
            Err(VfsError::NotFound {
                uri: format!("watch:{}", id.0),
                operation: "unwatch".to_string(),
            })
        }
    }

    /// Shut down the file watcher, cancelling all active watches.
    pub async fn shutdown(&self) {
        self.cancel.cancel();
        let mut watches = self.watches.write().await;
        for (_, reg) in watches.drain() {
            reg.cancel_token.cancel();
        }
    }

    /// Returns the number of active watches.
    pub fn active_watch_count(&self) -> usize {
        // Use try_read to avoid async in this sync context
        self.watches.try_read().map(|w| w.len()).unwrap_or(0)
    }

    /// Returns the debounce window.
    pub fn debounce_window(&self) -> Duration {
        self.debounce_window
    }

    /// Process a raw notify event, applying debounce and forwarding to consumers.
    async fn process_notify_event(
        watches: &Arc<RwLock<HashMap<WatchId, WatchRegistration>>>,
        notify_event: notify::Event,
        debounce: Duration,
    ) {
        let watch_event = event::convert_notify_event(&notify_event);
        if let Some(vfs_event) = watch_event {
            let watches_read = watches.read().await;
            for registration in watches_read.values() {
                // Check if this event is relevant to this watch
                let relevant = notify_event
                    .paths
                    .iter()
                    .any(|p| p.starts_with(&registration.path));

                if !relevant {
                    continue;
                }

                // Apply debounce logic
                let should_emit = {
                    let mut last_events = registration.last_events.lock().unwrap();
                    let now = Instant::now();

                    let primary_path = notify_event.paths.first().cloned().unwrap_or_default();
                    if let Some(last_time) = last_events.get(&primary_path) {
                        if now.duration_since(*last_time) < debounce {
                            false
                        } else {
                            last_events.insert(primary_path, now);
                            true
                        }
                    } else {
                        last_events.insert(primary_path, now);
                        true
                    }
                };

                if should_emit {
                    // Non-blocking send — drop if buffer full
                    let _ = registration.sender.try_send(vfs_event.clone());
                }
            }
        }
    }
}

// FileWatcher is Send + Sync because all interior state is protected
// by Arc<RwLock/Mutex>.
unsafe impl Send for FileWatcher {}
unsafe impl Sync for FileWatcher {}
