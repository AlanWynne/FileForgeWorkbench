//! Batch notification coalescing.
//!
//! The `BatchCoalescer` groups rapid external change events within a
//! configurable debounce window into a single `BatchNotification` to
//! avoid notification storms during bulk operations.
//!
//! Addresses: Requirement 8, criteria 1–7

use std::time::Duration;

use crate::change_event::{ChangeType, ExternalChange};
use crate::prompt::BatchNotification;

/// Groups rapid external change events within a debounce window.
///
/// Events arriving within the window are buffered. When the window expires,
/// all buffered events are assembled into a single `BatchNotification`.
///
/// If events keep arriving after the window expires (streaming changes),
/// the current batch is emitted and a new window starts.
///
/// Addresses: Requirement 8, criteria 1–7
#[derive(Debug)]
pub struct BatchCoalescer {
    /// Debounce window duration.
    debounce_window: Duration,
    /// Buffered events waiting for the window to expire.
    pending_events: Vec<ExternalChange>,
    /// Whether the coalescer is currently in an active debounce window.
    window_active: bool,
}

impl BatchCoalescer {
    /// Create a new coalescer with the given debounce window.
    ///
    /// # Arguments
    ///
    /// * `debounce_ms` - Debounce window in milliseconds (valid: 100–5000)
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            debounce_window: Duration::from_millis(debounce_ms),
            pending_events: Vec::new(),
            window_active: false,
        }
    }

    /// Add an event to the current batch.
    ///
    /// If this is the first event, starts the debounce window.
    /// Returns `true` if this was the first event (window just opened).
    pub fn add_event(&mut self, event: ExternalChange) -> bool {
        let is_first = self.pending_events.is_empty();
        self.pending_events.push(event);
        if is_first {
            self.window_active = true;
        }
        is_first
    }

    /// Check if there are pending events.
    pub fn has_pending_events(&self) -> bool {
        !self.pending_events.is_empty()
    }

    /// Returns the number of pending events.
    pub fn pending_count(&self) -> usize {
        self.pending_events.len()
    }

    /// Returns the debounce window duration.
    pub fn debounce_window(&self) -> Duration {
        self.debounce_window
    }

    /// Update the debounce window (hot-reload).
    pub fn set_debounce_window(&mut self, debounce_ms: u64) {
        self.debounce_window = Duration::from_millis(debounce_ms);
    }

    /// Flush the current batch — assemble all pending events into a BatchNotification.
    ///
    /// Returns `None` if there are no pending events.
    /// After flushing, the coalescer is ready for a new batch.
    ///
    /// Addresses: Requirement 8 AC 7 — streaming cutoff
    pub fn flush(&mut self) -> Option<BatchNotification> {
        if self.pending_events.is_empty() {
            return None;
        }

        let events = std::mem::take(&mut self.pending_events);
        self.window_active = false;

        let mut notification = BatchNotification::default();

        for event in events {
            match &event.change_type {
                ChangeType::ContentChanged => notification.modified.push(event),
                ChangeType::FileDeleted => notification.deleted.push(event),
                ChangeType::FileRenamed { .. } => notification.renamed.push(event),
            }
        }

        Some(notification)
    }

    /// Determine if the batch should be emitted as a single-item prompt
    /// (bypass batch UI for single events).
    pub fn should_emit_individually(&self) -> bool {
        self.pending_events.len() == 1
    }

    /// Reset the coalescer, discarding any pending events.
    pub fn reset(&mut self) {
        self.pending_events.clear();
        self.window_active = false;
    }

    /// Returns whether the debounce window is currently active.
    pub fn is_window_active(&self) -> bool {
        self.window_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    use crate::types::DocumentId;
    use ff_vfs::ResourceUri;

    fn make_content_change(doc_id: u64, dirty: bool) -> ExternalChange {
        ExternalChange::content_changed(
            DocumentId(doc_id),
            SystemTime::UNIX_EPOCH,
            SystemTime::now(),
            dirty,
        )
    }

    fn make_deleted(doc_id: u64) -> ExternalChange {
        ExternalChange::file_deleted(DocumentId(doc_id), false)
    }

    fn make_renamed(doc_id: u64) -> ExternalChange {
        ExternalChange::file_renamed(
            DocumentId(doc_id),
            ResourceUri::new("local", "/old.rs"),
            ResourceUri::new("local", "/new.rs"),
            false,
        )
    }

    #[test]
    fn new_coalescer_has_no_pending_events() {
        let coalescer = BatchCoalescer::new(500);
        assert!(!coalescer.has_pending_events());
        assert_eq!(coalescer.pending_count(), 0);
        assert!(!coalescer.is_window_active());
    }

    #[test]
    fn add_event_starts_window_on_first_event() {
        // Validates: Requirement 8.1 — coalesce within debounce window
        let mut coalescer = BatchCoalescer::new(500);
        let is_first = coalescer.add_event(make_content_change(1, false));

        assert!(is_first);
        assert!(coalescer.has_pending_events());
        assert_eq!(coalescer.pending_count(), 1);
        assert!(coalescer.is_window_active());
    }

    #[test]
    fn add_event_returns_false_for_subsequent_events() {
        let mut coalescer = BatchCoalescer::new(500);
        coalescer.add_event(make_content_change(1, false));
        let is_first = coalescer.add_event(make_content_change(2, false));

        assert!(!is_first);
        assert_eq!(coalescer.pending_count(), 2);
    }

    #[test]
    fn flush_assembles_batch_notification_by_type() {
        // Validates: Requirement 8.1, 8.2
        let mut coalescer = BatchCoalescer::new(500);
        coalescer.add_event(make_content_change(1, false));
        coalescer.add_event(make_content_change(2, true));
        coalescer.add_event(make_deleted(3));
        coalescer.add_event(make_renamed(4));

        let batch = coalescer.flush().unwrap();

        assert_eq!(batch.modified.len(), 2);
        assert_eq!(batch.deleted.len(), 1);
        assert_eq!(batch.renamed.len(), 1);
        assert_eq!(batch.total_count(), 4);
    }

    #[test]
    fn flush_clears_pending_events() {
        // Validates: Requirement 8.7 — streaming cutoff
        let mut coalescer = BatchCoalescer::new(500);
        coalescer.add_event(make_content_change(1, false));
        coalescer.flush();

        assert!(!coalescer.has_pending_events());
        assert!(!coalescer.is_window_active());
    }

    #[test]
    fn flush_returns_none_when_empty() {
        let mut coalescer = BatchCoalescer::new(500);
        assert!(coalescer.flush().is_none());
    }

    #[test]
    fn dirty_documents_excluded_from_clean_list() {
        // Validates: Requirement 8.4 — dirty files highlighted separately
        let mut coalescer = BatchCoalescer::new(500);
        coalescer.add_event(make_content_change(1, false));
        coalescer.add_event(make_content_change(2, true));
        coalescer.add_event(make_content_change(3, false));

        let batch = coalescer.flush().unwrap();
        let dirty = batch.dirty_documents();
        let clean = batch.clean_documents();

        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0], DocumentId(2));
        assert_eq!(clean.len(), 2);
        assert!(clean.contains(&DocumentId(1)));
        assert!(clean.contains(&DocumentId(3)));
    }

    #[test]
    fn should_emit_individually_for_single_event() {
        let mut coalescer = BatchCoalescer::new(500);
        coalescer.add_event(make_content_change(1, false));
        assert!(coalescer.should_emit_individually());
    }

    #[test]
    fn should_not_emit_individually_for_multiple_events() {
        let mut coalescer = BatchCoalescer::new(500);
        coalescer.add_event(make_content_change(1, false));
        coalescer.add_event(make_content_change(2, false));
        assert!(!coalescer.should_emit_individually());
    }

    #[test]
    fn set_debounce_window_updates_duration() {
        // Validates: Requirement 8.6 — configurable debounce window
        let mut coalescer = BatchCoalescer::new(500);
        assert_eq!(coalescer.debounce_window(), Duration::from_millis(500));

        coalescer.set_debounce_window(1000);
        assert_eq!(coalescer.debounce_window(), Duration::from_millis(1000));
    }

    #[test]
    fn reset_discards_pending_events() {
        let mut coalescer = BatchCoalescer::new(500);
        coalescer.add_event(make_content_change(1, false));
        coalescer.add_event(make_content_change(2, false));

        coalescer.reset();
        assert!(!coalescer.has_pending_events());
        assert!(!coalescer.is_window_active());
    }

    #[test]
    fn consecutive_flushes_produce_independent_batches() {
        // Validates: Requirement 8.7 — streaming cutoff, new window starts
        let mut coalescer = BatchCoalescer::new(500);

        coalescer.add_event(make_content_change(1, false));
        let batch1 = coalescer.flush().unwrap();
        assert_eq!(batch1.total_count(), 1);

        coalescer.add_event(make_content_change(2, false));
        coalescer.add_event(make_content_change(3, false));
        let batch2 = coalescer.flush().unwrap();
        assert_eq!(batch2.total_count(), 2);
    }

    #[test]
    fn no_event_lost_or_duplicated_across_batches() {
        // Validates: Requirement 8.1 — exactly one batch per event
        let mut coalescer = BatchCoalescer::new(500);

        for i in 1..=10 {
            coalescer.add_event(make_content_change(i, false));
        }
        let batch = coalescer.flush().unwrap();
        assert_eq!(batch.total_count(), 10);

        // No leftover events
        assert!(coalescer.flush().is_none());
    }
}
