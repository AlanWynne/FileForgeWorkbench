//! `FileSink` and `NoOpSink` implementations.
//!
//! Provides the output destinations for log records. `FileSink` writes to
//! disk via a buffered writer; `NoOpSink` discards all records silently
//! when file I/O is unavailable (graceful degradation).

use std::sync::atomic::{AtomicBool, Ordering};

/// Global diagnostic flag indicating whether the logging subsystem is
/// operating in fallback (no-op) mode.
///
/// When `true`, the subsystem cannot write to disk and is silently
/// discarding all log records. Any GUI status bar can query this flag
/// to display a warning.
static IS_FALLBACK: AtomicBool = AtomicBool::new(false);

/// A no-op log sink that discards all records silently.
///
/// Used as the fallback when file I/O is unavailable (e.g., directory
/// creation failure, permission errors, or disk-full conditions).
/// This ensures the application continues running without panicking
/// even when logging cannot function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NoOpSink;

impl NoOpSink {
    /// Creates a new `NoOpSink` and sets the global fallback diagnostic flag.
    ///
    /// After calling this, `is_fallback_active()` will return `true`.
    pub(crate) fn activate() -> Self {
        IS_FALLBACK.store(true, Ordering::Release);
        NoOpSink
    }

    /// Accepts a formatted log line and discards it.
    ///
    /// This is intentionally a no-op — all records are silently dropped.
    #[allow(dead_code)]
    pub(crate) fn write(&self, _line: &str) {
        // Intentionally empty — discard all records
    }

    /// Flush is a no-op since nothing is buffered.
    #[allow(dead_code)]
    pub(crate) fn flush(&self) {
        // Intentionally empty — nothing to flush
    }
}

/// Returns `true` if the logging subsystem is operating in fallback
/// (no-op) mode, meaning log records are being silently discarded.
///
/// Safe to call from any thread at any time. This flag is set during
/// initialization if the log directory or file cannot be created.
pub fn is_fallback_active() -> bool {
    IS_FALLBACK.load(Ordering::Acquire)
}

/// Resets the fallback flag to `false`.
///
/// Used internally when the subsystem transitions from fallback mode
/// to active mode (e.g., during re-initialization).
#[allow(dead_code)]
pub(crate) fn reset_fallback_flag() {
    IS_FALLBACK.store(false, Ordering::Release);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serialises all tests that touch the global IS_FALLBACK flag.
    static FLAG_LOCK: Mutex<()> = Mutex::new(());

    // Reset the global flag before each test to avoid interference
    fn reset_flag() {
        IS_FALLBACK.store(false, Ordering::Release);
    }

    // ─── NoOpSink Tests ─────────────────────────────────────────────────────

    #[test]
    fn noop_sink_activate_sets_fallback_flag() {
        // Validates: Requirement 1.4
        let _g = FLAG_LOCK.lock().expect("lock");
        reset_flag();
        assert!(!is_fallback_active());

        let _sink = NoOpSink::activate();
        assert!(is_fallback_active());

        reset_flag();
    }

    #[test]
    fn noop_sink_write_does_not_panic() {
        // Validates: Requirement 1.3
        let _g = FLAG_LOCK.lock().expect("lock");
        reset_flag();
        let sink = NoOpSink::activate();

        sink.write("2024-01-15T10:30:00.000+00:00 INFO  [test::module] hello world\n");
        sink.write("");
        sink.write("a".repeat(10_000).as_str());

        reset_flag();
    }

    #[test]
    fn noop_sink_flush_does_not_panic() {
        // Validates: Requirement 1.3
        let _g = FLAG_LOCK.lock().expect("lock");
        reset_flag();
        let sink = NoOpSink::activate();
        sink.flush();

        reset_flag();
    }

    #[test]
    fn is_fallback_active_returns_false_initially() {
        // Validates: Requirement 1.4
        let _g = FLAG_LOCK.lock().expect("lock");
        reset_flag();
        assert!(!is_fallback_active());
    }

    #[test]
    fn reset_fallback_flag_clears_the_flag() {
        // Validates: Requirement 1.4
        let _g = FLAG_LOCK.lock().expect("lock");
        IS_FALLBACK.store(true, Ordering::Release);
        assert!(is_fallback_active());

        reset_fallback_flag();
        assert!(!is_fallback_active());
    }

    #[test]
    fn fallback_flag_is_thread_safe() {
        // Validates: Requirement 7.7, Requirement 8.1
        //
        // Run serially to avoid races with other tests that call reset_flag().
        // The global IS_FALLBACK is process-wide; parallel test threads can
        // reset it between the spawn and the assertion.
        let _g = FLAG_LOCK.lock().expect("lock");
        reset_flag();

        let handle = std::thread::spawn(|| {
            let _sink = NoOpSink::activate();
        });
        handle.join().expect("thread panicked");

        // Acquire load ensures we see the store from the spawned thread.
        assert!(IS_FALLBACK.load(Ordering::Acquire));

        reset_flag();
    }
}
