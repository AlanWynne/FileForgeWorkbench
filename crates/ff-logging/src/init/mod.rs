//! Initialization sequence, directory creation, and fallback logic.
//!
//! Contains the primary `init()` and `init_default()` functions that
//! bootstrap the logging subsystem, as well as the top-level `log()`
//! and `log_lazy()` functions and status query helpers.
//!
//! # GUI-Independent Output Guarantee (Requirement 7)
//!
//! This module -- and the entire `ff-logging` crate -- maintains the following
//! invariants for GUI-independent process execution:
//!
//! - **No stdout/stderr output** (AC 7.4, 7.5): The logging subsystem never
//!   writes to `stdout` or `stderr`. All diagnostic output is routed
//!   exclusively through the file-based log sink. There are no `println!`,
//!   `eprintln!`, `print!`, `eprint!`, `dbg!`, or direct writes to
//!   `std::io::stdout()` / `std::io::stderr()` in production code.
//!
//! - **No console allocation** (AC 7.6): The crate never calls `AllocConsole`
//!   (Windows API) or any platform equivalent to create a console window.
//!
//! - **No child process spawning** (AC 7.6): The crate never uses
//!   `std::process::Command` or any other mechanism to spawn child processes.
//!   The only thread spawned is the internal writer thread via
//!   `std::thread::Builder::new()`, which is an in-process OS thread.
//!
//! The panic hook installed by `install_panic_hook()` chains to the previous
//! hook (which may print to stderr). This is acceptable because:
//! 1. It only fires during a panic (abnormal termination path).
//! 2. The chaining preserves standard Rust panic behavior.
//! 3. When `#![windows_subsystem = "windows"]` is set on the binary, stderr
//!    is not attached to a visible console anyway.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU8};
use std::sync::{Mutex, OnceLock};

use crate::channel::LogSender;
use crate::config::{default_log_directory, LogConfig};
use crate::error::LoggingError;

/// Status returned by `init()` and `init_default()`.
///
/// Indicates whether the logging subsystem is operating normally
/// (writing to a file) or has fallen back to a no-op sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoggingStatus {
    /// File sink is active; logging operates normally.
    Active,
    /// Fell back to no-op sink; no file I/O is occurring.
    Fallback,
}

/// The runtime state of the logging subsystem.
///
/// Stored as a global singleton behind `OnceLock`. Contains the sender
/// half of the channel, the current level for atomic filtering, and the
/// writer thread handle for joining on shutdown.
pub(crate) struct LogSubsystem {
    /// Current minimum level stored as a `u8` for lock-free reads.
    pub(super) level: AtomicU8,
    /// Sender half of the bounded channel.
    pub(super) sender: LogSender,
    /// Handle to the writer thread (consumed during shutdown).
    /// Wrapped in a `Mutex` because `OnceLock` does not allow interior mutation,
    /// and shutdown needs to take the handle to join the thread.
    pub(super) writer_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// Configuration snapshot for rotation decisions.
    #[allow(dead_code)]
    max_file_size_mb: u32,
    /// Configuration snapshot for retention decisions.
    #[allow(dead_code)]
    max_retained_files: u32,
}

/// Global flag indicating whether the subsystem has received a shutdown signal.
///
/// Once set to `true`, all subsequent `log()` and `log_lazy()` calls return
/// immediately without sending records to the channel.
pub(crate) static IS_SHUTDOWN: AtomicBool = AtomicBool::new(false);

/// Global singleton holding the logging subsystem state.
///
/// Uses `OnceLock` to ensure single initialization and safe concurrent access.
pub(super) static SUBSYSTEM: OnceLock<LogSubsystem> = OnceLock::new();

/// Creates the log directory at the given path, including all intermediate
/// parent directories.
///
/// # Errors
///
/// Returns `LoggingError::DirectoryCreation` if directory creation fails
/// due to permission errors, a read-only filesystem, or other I/O issues.
pub(crate) fn ensure_log_directory(path: &Path) -> Result<(), LoggingError> {
    std::fs::create_dir_all(path).map_err(|source| LoggingError::DirectoryCreation {
        path: path.to_path_buf(),
        source,
    })
}

/// Resolves the log directory by trying the configured path first, then
/// falling back to the platform default.
///
/// # Returns
///
/// - `Ok(PathBuf)` with the successfully-created directory path
/// - `Err(())` if neither the configured path nor the platform default
///   could be created (caller should use the no-op sink)
pub(crate) fn resolve_log_directory(config: &LogConfig) -> Result<PathBuf, ()> {
    // Try the configured directory first
    if ensure_log_directory(&config.directory).is_ok() {
        return Ok(config.directory.clone());
    }

    // If the configured path failed, try the platform default
    // (skip if the configured path IS the platform default)
    let platform_default = default_log_directory();
    if config.directory != platform_default && ensure_log_directory(&platform_default).is_ok() {
        return Ok(platform_default);
    }

    // Both paths failed -- caller should use no-op sink
    Err(())
}

/// Returns a reference to the global subsystem sender, if initialized.
///
/// Used by the shutdown module to send the shutdown signal.
pub(crate) fn get_sender() -> Option<&'static LogSender> {
    SUBSYSTEM.get().map(|s| &s.sender)
}

/// Returns the writer thread handle for joining during shutdown.
///
/// Takes the handle from the internal `Mutex`, ensuring it can only be
/// consumed once. Called by the shutdown module to join the writer thread.
pub(crate) fn take_writer_handle() -> Option<std::thread::JoinHandle<()>> {
    let subsystem = SUBSYSTEM.get()?;
    let mut guard = subsystem.writer_handle.lock().ok()?;
    guard.take()
}

mod api;
mod writer_loop;

pub use api::{
    current_level, dropped_count, init, init_default, install_panic_hook, is_fallback,
    is_logging_available, log, log_lazy, reconfigure,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LogLevel;

    // === ensure_log_directory Tests =========================================

    #[test]
    fn ensure_log_directory_creates_simple_directory() {
        // Validates: Requirement 1.5
        let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
        let log_dir = tmp.path().join("logs");

        let result = ensure_log_directory(&log_dir);
        assert!(result.is_ok());
        assert!(log_dir.exists());
        assert!(log_dir.is_dir());
    }

    #[test]
    fn ensure_log_directory_creates_nested_directories() {
        // Validates: Requirement 1.5
        let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
        let log_dir = tmp.path().join("deep").join("nested").join("logs");

        let result = ensure_log_directory(&log_dir);
        assert!(result.is_ok());
        assert!(log_dir.exists());
        assert!(log_dir.is_dir());
    }

    #[test]
    fn ensure_log_directory_succeeds_if_already_exists() {
        // Validates: Requirement 1.5
        let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
        let log_dir = tmp.path().join("logs");
        std::fs::create_dir_all(&log_dir).expect("manual creation failed");

        let result = ensure_log_directory(&log_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn ensure_log_directory_returns_error_for_invalid_path() {
        // Validates: Requirement 1.6
        // Attempt to create a directory inside a file (which should fail)
        let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
        let file_path = tmp.path().join("a_file.txt");
        std::fs::write(&file_path, "content").expect("failed to write file");

        let invalid_dir = file_path.join("cannot_create_here");
        let result = ensure_log_directory(&invalid_dir);
        assert!(result.is_err());

        match result.unwrap_err() {
            LoggingError::DirectoryCreation { path, .. } => {
                assert_eq!(path, invalid_dir);
            }
            other => panic!("expected DirectoryCreation error, got: {other:?}"),
        }
    }

    // === resolve_log_directory Tests ========================================

    #[test]
    fn resolve_log_directory_uses_configured_path_when_valid() {
        // Validates: Requirement 4.2
        let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
        let log_dir = tmp.path().join("my_logs");

        let config = LogConfig {
            level: LogLevel::Info,
            directory: log_dir.clone(),
            max_file_size_mb: 10,
            max_retained_files: 5,
        };

        let result = resolve_log_directory(&config);
        assert_eq!(result, Ok(log_dir));
    }

    #[test]
    fn resolve_log_directory_falls_back_to_platform_default_on_failure() {
        // Validates: Requirement 4.4
        let tmp = tempfile::TempDir::new().expect("failed to create temp dir");
        let file_path = tmp.path().join("blocker_file");
        std::fs::write(&file_path, "block").expect("failed to write blocker");

        // Use an invalid path (inside a file)
        let invalid_dir = file_path.join("cannot_create");

        let config = LogConfig {
            level: LogLevel::Info,
            directory: invalid_dir,
            max_file_size_mb: 10,
            max_retained_files: 5,
        };

        // This will try the invalid path, fail, then try platform default.
        // On CI/test environments the platform default should succeed.
        let result = resolve_log_directory(&config);
        if result.is_ok() {
            let resolved = result.unwrap();
            assert_eq!(resolved, default_log_directory());
        }
        // If the platform default also fails (e.g., unusual CI env), Err is valid
    }
}
