use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

use crate::channel::{create_log_channel, FormattedRecord};
use crate::config::LogConfig;
use crate::format::format_record;
use crate::level::LogLevel;
use crate::record::LogRecord;
use crate::sink::{is_fallback_active, NoOpSink};
use crate::writer::LogFileWriter;

use super::writer_loop::writer_thread_loop;
use super::{
    get_sender, resolve_log_directory, LogSubsystem, LoggingStatus, IS_SHUTDOWN, SUBSYSTEM,
};

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

/// Re-apply configuration to an already-initialized logging subsystem.
///
/// This is the second half of two-phase startup: `init` / `init_default` runs
/// first (before the configuration system exists), and `reconfigure` runs once
/// the configuration has been loaded. If the directory changed, the writer
/// thread flushes and closes the current file and opens a new file under the
/// new directory. The minimum level is applied atomically on this thread so
/// producers see it immediately; rotation settings are handed to the writer
/// thread. Out-of-range rotation values are clamped, each emitting a WARN
/// record (same rules as [`init`]).
///
/// Returns [`LoggingStatus::Active`] when the subsystem is initialized and the
/// request was dispatched, or [`LoggingStatus::Fallback`] when the subsystem is
/// uninitialized, in no-op fallback mode, or has already received a shutdown
/// signal. Never returns an error and never panics.
///
/// # Requirement Coverage
///
/// Implements Requirement 11 (AC 11.1-11.6, 11.8-11.10).
pub fn reconfigure(config: LogConfig) -> LoggingStatus {
    // No-op after shutdown signal (Requirement 11, AC 11.6).
    if IS_SHUTDOWN.load(Ordering::Acquire) {
        return LoggingStatus::Fallback;
    }

    // No-op if never initialized (Requirement 11, AC 11.6).
    let Some(subsystem) = SUBSYSTEM.get() else {
        return LoggingStatus::Fallback;
    };

    // If we are in no-op fallback mode, there is no file sink to reconfigure.
    if is_fallback_active() {
        return LoggingStatus::Fallback;
    }

    // Validate and clamp rotation values (Requirement 11, AC 11.4).
    let mut config = config;
    let warnings = config.validate();

    // Apply the new minimum level atomically so producers filter against it
    // immediately (Requirement 11, AC 11.4).
    subsystem.level.store(config.level as u8, Ordering::Relaxed);

    // Emit any clamping warnings through the normal channel.
    for warning in &warnings {
        log(LogLevel::Warn, "ff_logging::reconfigure", warning);
    }

    // Hand the directory and rotation settings to the writer thread. The
    // directory is always provided; the writer compares it to its current
    // directory and only switches files when it actually differs
    // (Requirement 11, AC 11.2 / 11.3).
    subsystem
        .sender
        .send_reconfigure(crate::channel::ReconfigureRequest {
            directory: Some(config.directory.clone()),
            max_file_size_mb: config.max_file_size_mb,
            max_retained_files: config.max_retained_files,
        });

    LoggingStatus::Active
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
            // This is a best-effort attempt -- if the writer thread is stuck
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
