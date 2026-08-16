//! Bounded MPSC channel, overflow handling, and drop counter.
//!
//! Provides the internal communication channel between log producers
//! (any thread) and the dedicated writer thread. Uses `crossbeam-channel`
//! with a capacity of 10,000 records.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};

use crate::level::LogLevel;

/// Channel capacity: maximum number of pending records before overflow.
const CHANNEL_CAPACITY: usize = 10_000;

/// Timeout for the writer thread's `recv_timeout` call. When this elapses
/// without receiving a message, the writer flushes its buffer.
const RECV_TIMEOUT: Duration = Duration::from_secs(1);

/// Messages sent through the internal channel from producer threads to
/// the writer thread.
#[derive(Debug, Clone)]
pub(crate) enum ChannelMessage {
    /// A pre-formatted log line ready for writing, with its level for flush decisions.
    Record(FormattedRecord),
    /// Request the writer to flush all buffered data to disk.
    Flush,
    /// Signal the writer thread to drain remaining records and shut down.
    Shutdown,
}

/// A pre-formatted log line ready for writing to disk.
///
/// Formatting happens on the caller's thread to distribute CPU cost
/// and avoid holding locks during string formatting.
#[derive(Debug, Clone)]
pub(crate) struct FormattedRecord {
    /// The fully formatted line (including trailing newline).
    pub line: String,
    /// The level of this record (used for flush-on-warn/error decisions).
    pub level: LogLevel,
}

/// Shared state for tracking dropped records across all producer threads.
///
/// This is wrapped in `Arc` so both the sender side (producers) and the
/// channel handle can access the counter.
#[derive(Debug)]
pub(crate) struct OverflowState {
    /// Cumulative count of records dropped due to channel overflow.
    dropped_count: AtomicU64,
    /// Whether a drop-warning needs to be emitted on the next successful send.
    pending_overflow_warn: AtomicBool,
}

impl OverflowState {
    /// Creates a new overflow state with zero drops.
    fn new() -> Self {
        Self {
            dropped_count: AtomicU64::new(0),
            pending_overflow_warn: AtomicBool::new(false),
        }
    }

    /// Returns the cumulative count of dropped records.
    ///
    /// Safe to call from any thread without blocking.
    pub(crate) fn dropped_count(&self) -> u64 {
        self.dropped_count.load(Ordering::Relaxed)
    }

    /// Increments the drop counter and sets the pending-warn flag.
    fn record_drop(&self) {
        self.dropped_count.fetch_add(1, Ordering::Relaxed);
        self.pending_overflow_warn.store(true, Ordering::Relaxed);
    }

    /// Checks and clears the pending-warn flag, returning `true` if a WARN
    /// should be emitted about dropped records.
    fn take_pending_warn(&self) -> bool {
        self.pending_overflow_warn.swap(false, Ordering::Relaxed)
    }
}

/// The producer-side handle for sending log records into the channel.
///
/// Clone-able and safe to share across threads.
#[derive(Debug, Clone)]
pub(crate) struct LogSender {
    /// The crossbeam channel sender.
    sender: Sender<ChannelMessage>,
    /// Shared overflow tracking state.
    overflow: Arc<OverflowState>,
}

impl LogSender {
    /// Attempts to send a formatted record through the channel.
    ///
    /// If the channel is full, the record is dropped and the overflow counter
    /// is incremented. On the next successful send after an overflow, a WARN
    /// message reporting total drops is emitted first.
    pub(crate) fn send_record(&self, record: FormattedRecord) {
        // If there was a previous overflow, emit a WARN about it first
        if self.overflow.take_pending_warn() {
            let total = self.overflow.dropped_count();
            let warn = FormattedRecord {
                line: format!(
                    "WARN  [ff_logging::channel] Log channel overflow: {total} record(s) dropped so far\n"
                ),
                level: LogLevel::Warn,
            };
            // Best-effort: if this also fails, we just increment the counter again
            if let Err(TrySendError::Full(_)) = self.sender.try_send(ChannelMessage::Record(warn)) {
                self.overflow.record_drop();
                // Re-set the pending flag since we couldn't deliver the warning
                self.overflow
                    .pending_overflow_warn
                    .store(true, Ordering::Relaxed);
            }
        }

        // Send the actual record
        if let Err(TrySendError::Full(_)) = self.sender.try_send(ChannelMessage::Record(record)) {
            self.overflow.record_drop();
        }
    }

    /// Sends a flush command to the writer thread.
    pub(crate) fn send_flush(&self) {
        let _ = self.sender.try_send(ChannelMessage::Flush);
    }

    /// Sends a shutdown signal to the writer thread.
    ///
    /// Uses a blocking send to ensure the shutdown message is delivered
    /// even when the channel is near capacity.
    pub(crate) fn send_shutdown(&self) {
        let _ = self.sender.send(ChannelMessage::Shutdown);
    }

    /// Returns the cumulative count of dropped log records.
    ///
    /// Safe to call from any thread without blocking.
    pub(crate) fn dropped_count(&self) -> u64 {
        self.overflow.dropped_count()
    }
}

/// The consumer-side handle for the writer thread.
///
/// Provides blocking receive with timeout for periodic flush behavior.
#[derive(Debug)]
pub(crate) struct LogReceiver {
    /// The crossbeam channel receiver.
    receiver: Receiver<ChannelMessage>,
}

impl LogReceiver {
    /// Blocks until a message is available or the timeout elapses.
    ///
    /// Returns `Ok(message)` if a message is received, or `Err(())` on timeout.
    /// The writer thread uses the timeout to trigger periodic buffer flushes.
    pub(crate) fn recv_timeout(&self) -> Result<ChannelMessage, ()> {
        self.receiver.recv_timeout(RECV_TIMEOUT).map_err(|_| ())
    }

    /// Drains all remaining messages from the channel without blocking.
    ///
    /// Used during shutdown to process any queued records before exiting.
    pub(crate) fn drain(&self) -> Vec<ChannelMessage> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.receiver.try_recv() {
            messages.push(msg);
        }
        messages
    }
}

/// Creates the bounded channel pair for log record communication.
///
/// Returns a `(LogSender, LogReceiver)` tuple. The sender is clonable and
/// can be shared across all producer threads. The receiver is owned by the
/// dedicated writer thread.
///
/// Channel capacity is 10,000 records.
pub(crate) fn create_log_channel() -> (LogSender, LogReceiver) {
    let (sender, receiver) = bounded(CHANNEL_CAPACITY);
    let overflow = Arc::new(OverflowState::new());

    let log_sender = LogSender { sender, overflow };

    let log_receiver = LogReceiver { receiver };

    (log_sender, log_receiver)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // ─── Channel Creation Tests ─────────────────────────────────────────────

    #[test]
    fn create_log_channel_returns_connected_sender_and_receiver() {
        // Validates: Requirement 8.3
        let (sender, receiver) = create_log_channel();

        let record = FormattedRecord {
            line: "test line\n".to_string(),
            level: LogLevel::Info,
        };
        sender.send_record(record.clone());

        let msg = receiver.recv_timeout().expect("should receive message");
        match msg {
            ChannelMessage::Record(r) => assert_eq!(r.line, "test line\n"),
            _ => panic!("Expected Record message"),
        }
    }

    #[test]
    fn channel_capacity_is_10000() {
        // Validates: Requirement 8.4
        let (sender, _receiver) = create_log_channel();

        // Fill the channel to capacity
        for i in 0..CHANNEL_CAPACITY {
            let record = FormattedRecord {
                line: format!("line {i}\n"),
                level: LogLevel::Info,
            };
            sender.send_record(record);
        }

        // No drops yet
        assert_eq!(sender.dropped_count(), 0);
    }

    // ─── Overflow Tests ─────────────────────────────────────────────────────

    #[test]
    fn overflow_increments_drop_counter() {
        // Validates: Requirement 8.4
        let (sender, _receiver) = create_log_channel();

        // Fill channel to capacity
        for i in 0..CHANNEL_CAPACITY {
            let record = FormattedRecord {
                line: format!("line {i}\n"),
                level: LogLevel::Info,
            };
            sender.send_record(record);
        }

        // This should overflow
        let overflow_record = FormattedRecord {
            line: "overflow!\n".to_string(),
            level: LogLevel::Info,
        };
        sender.send_record(overflow_record);

        assert_eq!(sender.dropped_count(), 1);
    }

    #[test]
    fn overflow_counter_is_monotonically_increasing() {
        // Validates: Requirement 8.4, 8.5
        let (sender, _receiver) = create_log_channel();

        // Fill channel
        for i in 0..CHANNEL_CAPACITY {
            let record = FormattedRecord {
                line: format!("line {i}\n"),
                level: LogLevel::Info,
            };
            sender.send_record(record);
        }

        // Multiple overflows
        let mut prev_count = 0u64;
        for _ in 0..10 {
            let record = FormattedRecord {
                line: "overflow\n".to_string(),
                level: LogLevel::Info,
            };
            sender.send_record(record);
            let current = sender.dropped_count();
            assert!(current >= prev_count, "Counter should never decrease");
            prev_count = current;
        }
    }

    #[test]
    fn dropped_count_starts_at_zero() {
        // Validates: Requirement 8.5
        let (sender, _receiver) = create_log_channel();
        assert_eq!(sender.dropped_count(), 0);
    }

    // ─── Flush and Shutdown Tests ───────────────────────────────────────────

    #[test]
    fn send_flush_delivers_flush_message() {
        // Validates: Requirement 6.2
        let (sender, receiver) = create_log_channel();

        sender.send_flush();

        let msg = receiver.recv_timeout().expect("should receive flush");
        assert!(matches!(msg, ChannelMessage::Flush));
    }

    #[test]
    fn send_shutdown_delivers_shutdown_message() {
        // Validates: Requirement 8.6
        let (sender, receiver) = create_log_channel();

        sender.send_shutdown();

        let msg = receiver.recv_timeout().expect("should receive shutdown");
        assert!(matches!(msg, ChannelMessage::Shutdown));
    }

    // ─── Periodic Flush (Timeout) Tests ─────────────────────────────────────

    #[test]
    fn recv_timeout_returns_error_after_timeout_with_empty_channel() {
        // Validates: Requirement 6.2 (periodic flush via timeout)
        let (_sender, receiver) = create_log_channel();

        let start = std::time::Instant::now();
        let result = receiver.recv_timeout();
        let elapsed = start.elapsed();

        assert!(result.is_err(), "Should timeout on empty channel");
        // Should have waited approximately 1 second
        assert!(
            elapsed >= Duration::from_millis(900),
            "Timeout should be ~1s, was {:?}",
            elapsed
        );
        assert!(
            elapsed < Duration::from_millis(1500),
            "Timeout should not exceed 1.5s, was {:?}",
            elapsed
        );
    }

    // ─── Drain Tests ────────────────────────────────────────────────────────

    #[test]
    fn drain_collects_all_pending_messages() {
        // Validates: Requirement 8.6 (shutdown drains remaining records)
        let (sender, receiver) = create_log_channel();

        // Send 5 records
        for i in 0..5 {
            let record = FormattedRecord {
                line: format!("line {i}\n"),
                level: LogLevel::Info,
            };
            sender.send_record(record);
        }

        let messages = receiver.drain();
        assert_eq!(messages.len(), 5);
    }

    #[test]
    fn drain_returns_empty_vec_on_empty_channel() {
        // Validates: Requirement 8.6
        let (_sender, receiver) = create_log_channel();

        let messages = receiver.drain();
        assert!(messages.is_empty());
    }

    // ─── Overflow WARN Emission Tests ───────────────────────────────────────

    #[test]
    fn overflow_warn_is_emitted_on_next_successful_send() {
        // Validates: Requirement 8.4
        let (sender, receiver) = create_log_channel();

        // Fill channel
        for i in 0..CHANNEL_CAPACITY {
            let record = FormattedRecord {
                line: format!("line {i}\n"),
                level: LogLevel::Info,
            };
            sender.send_record(record);
        }

        // Overflow one record
        let overflow_record = FormattedRecord {
            line: "this gets dropped\n".to_string(),
            level: LogLevel::Info,
        };
        sender.send_record(overflow_record);
        assert_eq!(sender.dropped_count(), 1);

        // Drain the channel to make space
        let _ = receiver.drain();

        // Next send should first emit a WARN about the drop, then the record
        let record = FormattedRecord {
            line: "after overflow\n".to_string(),
            level: LogLevel::Info,
        };
        sender.send_record(record);

        // Should get the WARN first, then the record
        let messages = receiver.drain();
        assert!(
            messages.len() >= 2,
            "Expected at least 2 messages (WARN + record)"
        );

        // First message should be the overflow warning
        match &messages[0] {
            ChannelMessage::Record(r) => {
                assert!(
                    r.line.contains("overflow"),
                    "First message should be overflow WARN, got: {}",
                    r.line
                );
                assert!(r.line.contains("1 record(s) dropped"));
                assert_eq!(r.level, LogLevel::Warn);
            }
            _ => panic!("Expected Record message for overflow WARN"),
        }

        // Second message should be the actual record
        match &messages[1] {
            ChannelMessage::Record(r) => {
                assert_eq!(r.line, "after overflow\n");
            }
            _ => panic!("Expected Record message for actual log line"),
        }
    }

    // ─── Thread Safety Test ─────────────────────────────────────────────────

    #[test]
    fn sender_is_usable_from_multiple_threads() {
        // Validates: Requirement 8.1
        let (sender, receiver) = create_log_channel();

        let handles: Vec<_> = (0..4)
            .map(|thread_id| {
                let s = sender.clone();
                std::thread::spawn(move || {
                    for i in 0..100 {
                        let record = FormattedRecord {
                            line: format!("thread {thread_id} line {i}\n"),
                            level: LogLevel::Info,
                        };
                        s.send_record(record);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().expect("thread panicked");
        }

        // Should have received 400 messages
        let messages = receiver.drain();
        assert_eq!(messages.len(), 400);
    }
}
