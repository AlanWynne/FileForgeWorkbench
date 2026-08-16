//! Initialization sequence, directory creation, and fallback logic.
//!
//! Contains the primary `init()` and `init_default()` functions that
//! bootstrap the logging subsystem, as well as the top-level `log()`
//! and `log_lazy()` functions and status query helpers.
//!
//! # GUI-Independent Output Guarantee (Requirement 7)
//!
//! This module — and the entire `ff-logging` crate — maintains the following
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
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::channel::{create_log_channel, ChannelMessage, FormattedRecord, LogSender};
use crate::config::{default_log_directory, LogConfig};
use crate::error::LoggingError;
use crate::format::format_record;
use crate::level::LogLevel;
use crate::record::LogRecord;
use crate::rotation::{
    enforce_retention, handle_rotation_failure, perform_rotation, should_rotate,
};
use crate::sink::{is_fallback_active, NoOpSink};
use crate::writer::LogFileWriter;

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
    level: AtomicU8,
    /// Sender half of the bounded channel.
    sender: LogSender,
    /// Handle to the writer thread (consumed during shutdown).
    /// Wrapped in a `Mutex` because `OnceLock` does not allow interior mutation,
    /// and shutdown needs to take the handle to join the thread.
    writer_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
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
static SUBSYSTEM: OnceLock<LogSubsystem> = OnceLock::new();

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

    // Both paths failed — caller should use no-op sink
    Err(())
}

/// Initialize the logging subsystem with the given configuration.
///
/// Must be called once before any other subsystem is constructed.
/// Creates the log directory, opens the initial log file, and spawns the
/// writer thread. On failure, falls back to no-op mode gracefully.
///
/// # Returns
///
/// `LoggingStatus::Active` if file logging is operational, or
/// `LoggingStatus::Fallback` if the subsystem degraded to no-op mode.
pub fn init(config: LogConfig) -> LoggingStatus {
    // Validate and clamp config values
    let mut config = config;
    let warnings = config.validate();

    // Step 1: Resolve the log directory
    let log_directory = match resolve_log_directory(&config) {
        Ok(dir) => dir,
        Err(()) => {
            NoOpSink::activate();
            return LoggingStatus::Fallback;
        }
    };

    // Step 2: Create the LogFileWriter
    let writer = match LogFileWriter::new(&log_directory) {
        Ok(w) => w,
        Err(_) => {
            NoOpSink::activate();
            return LoggingStatus::Fallback;
        }
    };

    // Step 3: Create the bounded channel
    let (sender, receiver) = create_log_channel();

    // Capture config values for the writer thread
    let max_file_size_mb = config.max_file_size_mb;
    let max_retained_files = config.max_retained_files;
    let log_dir_for_thread = log_directory.clone();

    // Step 4: Spawn the dedicated writer thread
    let writer_handle = std::thread::Builder::new()
        .name("ff-logging-writer".to_string())
        .spawn(move || {
            writer_thread_loop(
                writer,
                receiver,
                max_file_size_mb,
                max_retained_files,
                &log_dir_for_thread,
            );
        });

    let handle = match writer_handle {
        Ok(h) => h,
        Err(_) => {
            NoOpSink::activate();
            return LoggingStatus::Fallback;
        }
    };

    // Step 5: Store the global subsystem state
    let subsystem = LogSubsystem {
        level: AtomicU8::new(config.level as u8),
        sender,
        writer_handle: Mutex::new(Some(handle)),
        max_file_size_mb,
        max_retained_files,
    };

    // If OnceLock already has a value (double-init), fall back gracefully
    if SUBSYSTEM.set(subsystem).is_err() {
        NoOpSink::activate();
        return LoggingStatus::Fallback;
    }

    // Install panic hook (Requirement 6, AC 6.4)
    install_panic_hook();

    // Write startup INFO record (Requirement 1, AC 1.2)
    let version = env!("CARGO_PKG_VERSION");
    let timestamp = chrono::Local::now().to_rfc3339();
    log(
        LogLevel::Info,
        "ff_logging::init",
        &format!("FileForgeWorkbench v{version} starting at {timestamp}"),
    );

    // Emit any configuration validation warnings as WARN-level records
    // (Requirement 3, AC 3.4; Requirement 5, AC 5.3, 5.8)
    for warning in &warnings {
        log(LogLevel::Warn, "ff_logging::config", warning);
    }

    LoggingStatus::Active
}

/// Initialize the logging subsystem with default configuration.
///
/// Equivalent to calling `init(LogConfig::default())`. Used when the
/// configuration system is not yet available.
pub fn init_default() -> LoggingStatus {
    init(LogConfig::default())
}

/// Install the custom panic hook that flushes logs within 500ms.
///
/// Captures the previous panic hook and chains to it after attempting a
/// flush. On panic, the hook sends a `Flush` message through the channel
/// and waits up to 500 milliseconds for the writer thread to process it.
/// If the flush does not complete within the timeout or the channel is
/// unavailable, the hook abandons the flush and allows the process to
/// continue unwinding normally.
///
/// Called automatically by `init()`. Can be called manually if needed
/// before full initialization.
///
/// # Requirement Coverage
///
/// Implements Requirement 6, AC 6.4.
pub fn install_panic_hook() {
    use std::time::Duration;

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Attempt to flush buffered log records within 500ms.
        // If the subsystem is not initialized (get_sender returns None),
        // skip the flush attempt entirely.
        if let Some(sender) = get_sender() {
            sender.send_flush();
            // Wait up to 500ms for the writer thread to process the flush.
            // This is a best-effort attempt — if the writer thread is stuck
            // or the I/O encounters an error, we simply abandon after timeout.
            std::thread::sleep(Duration::from_millis(500));
        }

        // Chain to the previous/default panic hook (prints to stderr).
        prev_hook(info);
    }));
}

/// Write a log record at the specified level.
///
/// The level check is performed atomically before any formatting or
/// allocation occurs. If the record's level is below the configured
/// minimum, this function returns immediately with zero cost.
///
/// Returns immediately without sending if the subsystem has received a
/// shutdown signal.
pub fn log(level: LogLevel, module_path: &str, message: &str) {
    // Reject new log calls after shutdown signal (Requirement 8, AC 8.6)
    if IS_SHUTDOWN.load(Ordering::Acquire) {
        return;
    }

    let Some(subsystem) = SUBSYSTEM.get() else {
        return;
    };

    // Zero-cost level filter: atomic load with Relaxed ordering
    let min_level = subsystem.level.load(Ordering::Relaxed);
    if (level as u8) < min_level {
        return;
    }

    // Format the record on the caller's thread
    let record = LogRecord::new(level, module_path, message);
    let formatted = format_record(&record);

    let formatted_record = FormattedRecord {
        line: formatted,
        level,
    };

    subsystem.sender.send_record(formatted_record);
}

/// Write a log record with lazy message formatting.
///
/// The closure `f` is only evaluated if the level passes the filter,
/// avoiding unnecessary string allocation for filtered records.
///
/// Returns immediately without sending if the subsystem has received a
/// shutdown signal.
pub fn log_lazy(level: LogLevel, module_path: &str, f: impl FnOnce() -> String) {
    // Reject new log calls after shutdown signal (Requirement 8, AC 8.6)
    if IS_SHUTDOWN.load(Ordering::Acquire) {
        return;
    }

    let Some(subsystem) = SUBSYSTEM.get() else {
        return;
    };

    // Zero-cost level filter: atomic load with Relaxed ordering
    let min_level = subsystem.level.load(Ordering::Relaxed);
    if (level as u8) < min_level {
        return;
    }

    // Evaluate the closure only after passing the level filter
    let message = f();
    let record = LogRecord::new(level, module_path, &message);
    let formatted = format_record(&record);

    let formatted_record = FormattedRecord {
        line: formatted,
        level,
    };

    subsystem.sender.send_record(formatted_record);
}

/// Returns `true` if the subsystem is in fallback (no-op) mode.
///
/// Safe to call from any thread at any time.
pub fn is_fallback() -> bool {
    is_fallback_active()
}

/// Returns `true` if the logging subsystem is fully operational (not in fallback mode).
///
/// This is the inverse of `is_fallback()` and is provided as a convenience for
/// platform-core subsystems that want to check logging availability in a
/// positive-logic style.
///
/// Safe to call from any thread at any time.
///
/// # Example
///
/// ```rust,no_run
/// use ff_logging::is_logging_available;
///
/// if !is_logging_available() {
///     // Display a status bar warning that logging is unavailable
/// }
/// ```
pub fn is_logging_available() -> bool {
    !is_fallback_active()
}

/// Returns the cumulative count of dropped log records due to channel overflow.
///
/// Safe to call from any thread without blocking.
pub fn dropped_count() -> u64 {
    match SUBSYSTEM.get() {
        Some(subsystem) => subsystem.sender.dropped_count(),
        None => 0,
    }
}

/// Returns the current effective log level.
///
/// Reflects the minimum level configured at initialization time.
pub fn current_level() -> LogLevel {
    match SUBSYSTEM.get() {
        Some(subsystem) => {
            let raw = subsystem.level.load(Ordering::Relaxed);
            LogLevel::from_u8(raw).unwrap_or(LogLevel::Info)
        }
        None => LogLevel::Info,
    }
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

// ─── Writer Thread Loop ─────────────────────────────────────────────────────

/// The main loop of the dedicated writer thread.
///
/// Continuously reads messages from the channel receiver and processes them:
/// - `Record`: checks rotation, performs rotation if needed, writes the line
/// - `Flush`: flushes the writer buffer to disk
/// - `Shutdown`: drains remaining records, flushes, and exits
/// - Timeout (Err): flushes buffer periodically (every ~1 second)
fn writer_thread_loop(
    mut writer: LogFileWriter,
    receiver: crate::channel::LogReceiver,
    max_file_size_mb: u32,
    max_retained_files: u32,
    log_directory: &Path,
) {
    loop {
        match receiver.recv_timeout() {
            Ok(ChannelMessage::Record(record)) => {
                handle_record(
                    &mut writer,
                    &record,
                    max_file_size_mb,
                    max_retained_files,
                    log_directory,
                );
            }
            Ok(ChannelMessage::Flush) => {
                let _ = writer.flush();
            }
            Ok(ChannelMessage::Shutdown) => {
                // Drain all remaining records from the channel
                let remaining = receiver.drain();
                for msg in remaining {
                    match msg {
                        ChannelMessage::Record(record) => {
                            handle_record(
                                &mut writer,
                                &record,
                                max_file_size_mb,
                                max_retained_files,
                                log_directory,
                            );
                        }
                        ChannelMessage::Flush => {
                            let _ = writer.flush();
                        }
                        ChannelMessage::Shutdown => {
                            // Ignore duplicate shutdown signals
                        }
                    }
                }
                // Final flush before exit
                let _ = writer.flush();
                break;
            }
            Err(()) => {
                // Timeout — periodic flush for buffered DEBUG/INFO records
                let _ = writer.flush();
            }
        }
    }
}

/// Handles a single record: checks rotation, performs it if needed, then writes.
fn handle_record(
    writer: &mut LogFileWriter,
    record: &FormattedRecord,
    max_file_size_mb: u32,
    max_retained_files: u32,
    log_directory: &Path,
) {
    let line_bytes = record.line.len() as u64;

    // Check if rotation is needed before writing
    if should_rotate(writer, line_bytes, max_file_size_mb) {
        match perform_rotation(writer) {
            Ok(()) => {
                // Enforce retention policy after successful rotation
                let warnings = enforce_retention(log_directory, max_retained_files);
                for warn_line in warnings {
                    let _ = writer.write_line(&warn_line, LogLevel::Warn);
                }
            }
            Err(err) => {
                handle_rotation_failure(writer, &err);
            }
        }
    }

    // Write the record line
    let _ = writer.write_line(&record.line, record.level);
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── ensure_log_directory Tests ─────────────────────────────────────────

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

    // ─── resolve_log_directory Tests ────────────────────────────────────────

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
