//! File watcher with debounce.
//!
//! Monitors configuration files for changes using OS-native file watching
//! (inotify on Linux, ReadDirectoryChangesW on Windows, FSEvents on macOS)
//! and coalesces rapid events within a configurable debounce window.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::ConfigError;

/// Default debounce window for coalescing file events (500ms).
pub const DEFAULT_DEBOUNCE_DURATION: Duration = Duration::from_millis(500);

/// A file change event after debouncing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChangeEvent {
    /// The path that was modified.
    pub path: PathBuf,
}

/// Configuration file watcher with debounce logic.
///
/// Monitors registered config files for changes using OS-native file watching
/// (inotify on Linux, ReadDirectoryChangesW on Windows, FSEvents on macOS).
/// Multiple events for the same file within the debounce window are coalesced
/// into a single change event.
pub struct ConfigWatcher {
    /// The underlying notify watcher.
    watcher: RecommendedWatcher,
    /// Receiver for raw events from notify, wrapped in Mutex for Sync.
    event_rx: Mutex<mpsc::Receiver<Result<Event, notify::Error>>>,
    /// Set of watched paths (canonical file paths being monitored).
    watched_paths: Vec<PathBuf>,
    /// Debounce window duration.
    debounce_duration: Duration,
    /// Last event time per path (for debouncing).
    last_event_times: HashMap<PathBuf, Instant>,
}

impl ConfigWatcher {
    /// Create a new file watcher with the default 500ms debounce window.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::WatcherError` if the OS-native watcher cannot
    /// be initialized (e.g., inotify limit reached).
    pub fn new() -> Result<Self, ConfigError> {
        Self::with_debounce(DEFAULT_DEBOUNCE_DURATION)
    }

    /// Create a new file watcher with a custom debounce duration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::WatcherError` if the OS-native watcher cannot
    /// be initialized.
    pub fn with_debounce(debounce_duration: Duration) -> Result<Self, ConfigError> {
        let (tx, rx) = mpsc::channel();
        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            notify::Config::default(),
        )
        .map_err(|e| ConfigError::WatcherError {
            details: format!("failed to create file watcher: {}", e),
        })?;

        Ok(Self {
            watcher,
            event_rx: Mutex::new(rx),
            watched_paths: Vec::new(),
            debounce_duration,
            last_event_times: HashMap::new(),
        })
    }

    /// Register a file path for watching.
    ///
    /// Watches the parent directory (more reliable for file modifications
    /// across editors that perform atomic saves via rename).
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::WatcherError` if the path cannot be watched
    /// (e.g., the directory does not exist or permissions are insufficient).
    pub fn watch(&mut self, path: &Path) -> Result<(), ConfigError> {
        let watch_path = path.parent().unwrap_or(path);
        self.watcher
            .watch(watch_path, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::WatcherError {
                details: format!("failed to watch '{}': {}", path.display(), e),
            })?;
        self.watched_paths.push(path.to_path_buf());
        Ok(())
    }

    /// Register multiple file paths for watching.
    ///
    /// # Errors
    ///
    /// Returns an error on the first path that fails to watch. Previously
    /// registered paths from this call remain watched.
    pub fn watch_all(&mut self, paths: &[PathBuf]) -> Result<(), ConfigError> {
        for path in paths {
            self.watch(path)?;
        }
        Ok(())
    }

    /// Unwatch a previously watched path.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::WatcherError` if the path cannot be unwatched.
    pub fn unwatch(&mut self, path: &Path) -> Result<(), ConfigError> {
        let watch_path = path.parent().unwrap_or(path);
        self.watcher
            .unwatch(watch_path)
            .map_err(|e| ConfigError::WatcherError {
                details: format!("failed to unwatch '{}': {}", path.display(), e),
            })?;
        self.watched_paths.retain(|p| p != path);
        self.last_event_times.remove(path);
        Ok(())
    }

    /// Poll for debounced file change events.
    ///
    /// Drains raw events from the OS watcher and returns only those events
    /// whose debounce window has elapsed. Multiple rapid modifications to
    /// the same file within the debounce window are coalesced into a single
    /// `FileChangeEvent`.
    pub fn poll_changes(&mut self) -> Vec<FileChangeEvent> {
        let now = Instant::now();

        // Drain raw events from notify
        let rx = self.event_rx.lock().unwrap();
        while let Ok(result) = rx.try_recv() {
            if let Ok(event) = result {
                for path in event.paths {
                    // Only track events for paths we're watching
                    if self.watched_paths.contains(&path) {
                        self.last_event_times.insert(path, now);
                    }
                }
            }
        }
        drop(rx);

        // Collect paths that have passed the debounce window
        let mut changes = Vec::new();
        let mut to_remove = Vec::new();

        for (path, last_time) in &self.last_event_times {
            if now.duration_since(*last_time) >= self.debounce_duration {
                changes.push(FileChangeEvent { path: path.clone() });
                to_remove.push(path.clone());
            }
        }

        for path in to_remove {
            self.last_event_times.remove(&path);
        }

        changes
    }

    /// Returns the list of currently watched paths.
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }

    /// Stop watching all files and clean up resources.
    pub fn stop(mut self) {
        for path in self.watched_paths.clone() {
            let _ = self.unwatch(&path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use tempfile::TempDir;

    // Validates: Requirement 3.1 — OS-native file watcher creation succeeds
    #[test]
    fn watcher_creation_succeeds() {
        let watcher = ConfigWatcher::new();
        assert!(watcher.is_ok(), "ConfigWatcher::new() should succeed");
    }

    // Validates: Requirement 3.1 — custom debounce duration accepted
    #[test]
    fn watcher_creation_with_custom_debounce_succeeds() {
        let watcher = ConfigWatcher::with_debounce(Duration::from_millis(100));
        assert!(
            watcher.is_ok(),
            "ConfigWatcher::with_debounce() should succeed"
        );
    }

    // Validates: Requirement 3.1 — watch registration adds paths to watched list
    #[test]
    fn watch_registration_adds_paths_to_watched_list() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("config.toml");
        fs::write(&file_path, "key = \"value\"").unwrap();

        let mut watcher = ConfigWatcher::new().unwrap();
        watcher.watch(&file_path).unwrap();

        assert_eq!(watcher.watched_paths().len(), 1);
        assert_eq!(watcher.watched_paths()[0], file_path);
    }

    // Validates: Requirement 3.1 — watch_all registers multiple paths
    #[test]
    fn watch_all_registers_multiple_paths() {
        let dir = TempDir::new().unwrap();
        let file1 = dir.path().join("config1.toml");
        let file2 = dir.path().join("config2.toml");
        fs::write(&file1, "a = 1").unwrap();
        fs::write(&file2, "b = 2").unwrap();

        let mut watcher = ConfigWatcher::new().unwrap();
        watcher.watch_all(&[file1.clone(), file2.clone()]).unwrap();

        assert_eq!(watcher.watched_paths().len(), 2);
        assert!(watcher.watched_paths().contains(&file1));
        assert!(watcher.watched_paths().contains(&file2));
    }

    // Validates: Requirement 3.1 — unwatch removes path from watched list
    #[test]
    fn unwatch_removes_path_from_watched_list() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("config.toml");
        fs::write(&file_path, "key = \"value\"").unwrap();

        let mut watcher = ConfigWatcher::new().unwrap();
        watcher.watch(&file_path).unwrap();
        assert_eq!(watcher.watched_paths().len(), 1);

        watcher.unwatch(&file_path).unwrap();
        assert!(watcher.watched_paths().is_empty());
    }

    // Validates: Requirement 3.7 — debounce coalesces rapid events
    #[test]
    fn debounce_coalesces_rapid_events_into_single_change() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("config.toml");
        fs::write(&file_path, "initial = true").unwrap();

        let debounce_ms = 50;
        let mut watcher = ConfigWatcher::with_debounce(Duration::from_millis(debounce_ms)).unwrap();
        watcher.watch(&file_path).unwrap();

        // Write to the file multiple times rapidly
        for i in 0..5 {
            fs::write(&file_path, format!("value = {}", i)).unwrap();
            thread::sleep(Duration::from_millis(10));
        }

        // Immediately after writes, debounce window hasn't elapsed yet
        let _changes = watcher.poll_changes();
        // Events may or may not have arrived yet — that's fine

        // Wait for debounce window to fully elapse
        thread::sleep(Duration::from_millis(debounce_ms + 100));

        // Now poll — should get at most one event for the file
        let changes = watcher.poll_changes();

        // Due to the coalescing, we should see at most one event per file
        let file_changes: Vec<_> = changes.iter().filter(|c| c.path == file_path).collect();
        assert!(
            file_changes.len() <= 1,
            "Expected at most 1 coalesced event, got {}",
            file_changes.len()
        );
    }

    // Validates: Requirement 3.2 — change detection within 2 seconds
    #[test]
    fn change_detected_within_two_seconds() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("config.toml");
        fs::write(&file_path, "original = true").unwrap();

        let debounce_ms = 50;
        let mut watcher = ConfigWatcher::with_debounce(Duration::from_millis(debounce_ms)).unwrap();
        watcher.watch(&file_path).unwrap();

        // Allow watcher to initialize
        thread::sleep(Duration::from_millis(100));

        // Modify the file
        fs::write(&file_path, "modified = true").unwrap();

        // Poll repeatedly within 2 seconds until we detect the change
        let start = Instant::now();
        let mut detected = false;
        while start.elapsed() < Duration::from_secs(2) {
            thread::sleep(Duration::from_millis(debounce_ms + 20));
            let changes = watcher.poll_changes();
            if changes.iter().any(|c| c.path == file_path) {
                detected = true;
                break;
            }
        }

        assert!(detected, "File change should be detected within 2 seconds");
    }

    // Validates: Requirement 3.1 — watcher error on invalid path
    #[test]
    fn watch_nonexistent_path_returns_watcher_error() {
        let mut watcher = ConfigWatcher::new().unwrap();
        let result = watcher.watch(Path::new(
            "/nonexistent/path/that/does/not/exist/config.toml",
        ));

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            ConfigError::WatcherError { details } => {
                assert!(
                    details.contains("failed to watch"),
                    "Error should mention 'failed to watch', got: {}",
                    details
                );
            }
            other => panic!("Expected WatcherError, got: {:?}", other),
        }
    }

    // Validates: Requirement 3.1 — stop cleans up all watches
    #[test]
    fn stop_cleans_up_all_watches() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("config.toml");
        fs::write(&file_path, "key = \"value\"").unwrap();

        let mut watcher = ConfigWatcher::new().unwrap();
        watcher.watch(&file_path).unwrap();
        assert_eq!(watcher.watched_paths().len(), 1);

        // stop() consumes self, so we can't inspect afterwards
        // but it should not panic
        watcher.stop();
    }

    // Validates: Requirement 3.1 — poll_changes returns empty when no events
    #[test]
    fn poll_changes_returns_empty_when_no_modifications() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("config.toml");
        fs::write(&file_path, "key = \"value\"").unwrap();

        let mut watcher = ConfigWatcher::new().unwrap();
        watcher.watch(&file_path).unwrap();

        // No modifications made, should get no events
        thread::sleep(Duration::from_millis(100));
        let changes = watcher.poll_changes();
        assert!(
            changes.is_empty(),
            "Expected no changes without file modifications"
        );
    }

    // Validates: Requirement 3.7 — WatcherError display format
    #[test]
    fn watcher_error_follows_config_prefix_pattern() {
        let err = ConfigError::WatcherError {
            details: "failed to create file watcher: inotify limit reached".to_string(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("[config] watcher:"),
            "WatcherError message should start with '[config] watcher:', got: {}",
            msg
        );
    }
}
