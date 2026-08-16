//! Graceful shutdown, flush timeout, and shutdown signaling.
//!
//! Provides the `shutdown()` function that drains buffered records,
//! writes the final shutdown message, and joins the writer thread
//! within a 5-second timeout.

use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::init::{get_sender, take_writer_handle, IS_SHUTDOWN};
use crate::level::LogLevel;

/// The maximum time `shutdown()` will wait for the writer thread to
/// drain and flush all buffered records before returning.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Gracefully shut down the logging subsystem.
///
/// Performs the following sequence:
/// 1. Sets the shutdown flag so no new log calls are accepted.
/// 2. Writes a final INFO-level "Application shutdown complete" record
///    through the normal channel (bypassing the shutdown flag internally).
/// 3. Sends the `Shutdown` channel message to the writer thread, which
///    drains all remaining records, flushes, and exits.
/// 4. Joins the writer thread with a 5-second timeout.
///
/// After this call returns, no further log records will be written.
/// If the writer thread does not complete within 5 seconds, this
/// function returns anyway to avoid blocking application exit.
///
/// Safe to call multiple times — subsequent calls are no-ops.
pub fn shutdown() {
    // Set the shutdown flag atomically. If it was already set, another
    // call to shutdown() is in progress or completed — return early.
    if IS_SHUTDOWN.swap(true, Ordering::AcqRel) {
        return;
    }

    // Write the final "Application shutdown complete" INFO record.
    // We bypass the IS_SHUTDOWN check by sending directly through the sender.
    if let Some(sender) = get_sender() {
        let record = crate::record::LogRecord::new(
            LogLevel::Info,
            "ff_logging::shutdown",
            "Application shutdown complete",
        );
        let formatted = crate::format::format_record(&record);
        let formatted_record = crate::channel::FormattedRecord {
            line: formatted,
            level: LogLevel::Info,
        };
        sender.send_record(formatted_record);

        // Send the shutdown signal to the writer thread.
        // This causes it to drain remaining records, flush, and exit.
        sender.send_shutdown();
    }

    // Take the writer thread handle and join with a 5-second timeout.
    if let Some(handle) = take_writer_handle() {
        // Spawn a helper thread that performs the blocking join, then
        // wait on it with a timeout via a one-shot channel.
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = handle.join();
            let _ = tx.send(());
        });
        // Wait up to 5 seconds for the writer thread to finish.
        let _ = rx.recv_timeout(SHUTDOWN_TIMEOUT);
    }
}
