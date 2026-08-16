//! External modification detector — central coordinator.
//!
//! The `ExternalModificationDetector` subscribes to VFS watch events, maintains
//! per-document tracking state, and coordinates detection with the policy engine
//! and batch coalescer.
//!
//! Addresses: Requirement 1 (AC 1.1–1.6), Requirement 3 (AC 3.1–3.7)

use std::collections::HashMap;
use std::time::SystemTime;

use ff_vfs::{ResourceUri, WatchHandle};

use crate::change_event::ExternalChange;
use crate::config::ExternalModConfig;
use crate::error::ExternalModError;
use crate::mtime_tracker::MtimeTracker;
use crate::types::{DocumentId, MtimeComparison};

/// Tracks per-document watch state and deduplication.
#[derive(Debug)]
#[allow(dead_code)]
pub struct DocumentState {
    /// The resource URI being watched.
    pub uri: ResourceUri,
    /// Whether this document has a pending (unanswered) notification.
    pub pending_notification: bool,
    /// The mtime at which the user was last prompted (prevents re-prompting).
    pub last_asked_mtime: Option<SystemTime>,
    /// Whether the document buffer is dirty (has unsaved local changes).
    pub is_dirty: bool,
    /// Whether this document uses polling fallback (no watch support).
    pub uses_polling: bool,
}

/// Registry of active VFS watches for open documents.
///
/// Manages watch lifecycle: registration, event routing, cancellation,
/// and fallback to polling when the VFS provider doesn't support watching.
///
/// Addresses: Requirement 1, criteria 1–6
#[derive(Debug)]
pub struct WatchRegistry {
    /// Active watch handles keyed by document ID.
    handles: HashMap<DocumentId, WatchHandle>,
    /// Document IDs that use polling fallback (no watch support).
    polling_documents: Vec<DocumentId>,
}

impl WatchRegistry {
    /// Create a new empty watch registry.
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            polling_documents: Vec::new(),
        }
    }

    /// Register a watch handle for a document.
    ///
    /// Addresses: Requirement 1 AC 2
    pub fn register_watch(&mut self, doc_id: DocumentId, handle: WatchHandle) {
        self.handles.insert(doc_id, handle);
    }

    /// Register a document for polling fallback.
    ///
    /// Called when VFS watch returns UnsupportedOperation.
    ///
    /// Addresses: Requirement 1 AC 5
    pub fn register_polling(&mut self, doc_id: DocumentId) {
        if !self.polling_documents.contains(&doc_id) {
            self.polling_documents.push(doc_id);
        }
    }

    /// Cancel and remove the watch for a document.
    ///
    /// Addresses: Requirement 1 AC 3
    pub fn cancel_watch(&mut self, doc_id: DocumentId) {
        if let Some(handle) = self.handles.remove(&doc_id) {
            handle.cancel();
        }
        self.polling_documents.retain(|id| *id != doc_id);
    }

    /// Cancel all active watches (shutdown).
    pub fn cancel_all(&mut self) {
        for (_id, handle) in self.handles.drain() {
            handle.cancel();
        }
        self.polling_documents.clear();
    }

    /// Returns the number of active watches.
    pub fn active_watch_count(&self) -> usize {
        self.handles.len()
    }

    /// Returns the number of documents using polling fallback.
    pub fn polling_count(&self) -> usize {
        self.polling_documents.len()
    }

    /// Returns true if the given document has an active watch.
    pub fn has_watch(&self, doc_id: DocumentId) -> bool {
        self.handles.contains_key(&doc_id)
    }

    /// Returns true if the given document uses polling fallback.
    pub fn uses_polling(&self, doc_id: DocumentId) -> bool {
        self.polling_documents.contains(&doc_id)
    }

    /// Get a mutable reference to a watch handle for receiving events.
    pub fn get_handle_mut(&mut self, doc_id: DocumentId) -> Option<&mut WatchHandle> {
        self.handles.get_mut(&doc_id)
    }

    /// Returns all document IDs that are being monitored (watch or polling).
    pub fn all_monitored_documents(&self) -> Vec<DocumentId> {
        let mut docs: Vec<DocumentId> = self.handles.keys().copied().collect();
        for &doc_id in &self.polling_documents {
            if !docs.contains(&doc_id) {
                docs.push(doc_id);
            }
        }
        docs
    }
}

impl Default for WatchRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// The central service managing external modification detection for all open documents.
///
/// Coordinates between VFS watch events, mtime tracking, deduplication, and
/// the policy engine to detect and respond to external file changes.
///
/// Addresses: Requirements 1–3
#[derive(Debug)]
pub struct ExternalModificationDetector {
    /// Per-document mtime tracking.
    pub mtime_tracker: MtimeTracker,
    /// Active VFS watch registry.
    pub watch_registry: WatchRegistry,
    /// Per-document state for deduplication and pending notifications.
    pub document_states: HashMap<DocumentId, DocumentState>,
    /// Current configuration.
    pub(crate) config: ExternalModConfig,
    /// Next document ID to assign.
    next_id: u64,
}

impl ExternalModificationDetector {
    /// Create a new detector with the given configuration.
    pub fn new(config: ExternalModConfig) -> Self {
        Self {
            mtime_tracker: MtimeTracker::new(),
            watch_registry: WatchRegistry::new(),
            document_states: HashMap::new(),
            config,
            next_id: 1,
        }
    }

    /// Allocate the next document ID.
    pub fn next_document_id(&mut self) -> DocumentId {
        let id = DocumentId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Register a document for external modification tracking.
    ///
    /// Records the initial mtime snapshot and registers a watch handle.
    ///
    /// Addresses: Requirement 1 AC 2, Requirement 2 AC 2
    pub fn register_document(
        &mut self,
        doc_id: DocumentId,
        uri: ResourceUri,
        mtime: SystemTime,
        watch_handle: Option<WatchHandle>,
        uses_polling: bool,
    ) {
        self.mtime_tracker
            .record_snapshot(doc_id, uri.clone(), mtime);

        if let Some(handle) = watch_handle {
            self.watch_registry.register_watch(doc_id, handle);
        }

        if uses_polling {
            self.watch_registry.register_polling(doc_id);
        }

        self.document_states.insert(
            doc_id,
            DocumentState {
                uri,
                pending_notification: false,
                last_asked_mtime: None,
                is_dirty: false,
                uses_polling,
            },
        );
    }

    /// Unregister a document (called when document is closed).
    ///
    /// Cancels the VFS watch and removes all tracking state.
    ///
    /// Addresses: Requirement 1 AC 3
    #[allow(clippy::result_large_err)]
    pub fn unregister_document(&mut self, doc_id: DocumentId) -> Result<(), ExternalModError> {
        self.watch_registry.cancel_watch(doc_id);
        self.mtime_tracker.remove_snapshot(doc_id);
        self.document_states.remove(&doc_id);
        Ok(())
    }

    /// Notify the detector that a document was saved.
    ///
    /// Updates the mtime snapshot to the post-save value.
    ///
    /// Addresses: Requirement 2 AC 3
    pub fn notify_document_saved(&mut self, doc_id: DocumentId, new_mtime: SystemTime) {
        self.mtime_tracker.update_snapshot(doc_id, new_mtime);
        if let Some(state) = self.document_states.get_mut(&doc_id) {
            state.is_dirty = false;
            state.pending_notification = false;
        }
    }

    /// Update the dirty state of a document.
    pub fn set_dirty(&mut self, doc_id: DocumentId, dirty: bool) {
        if let Some(state) = self.document_states.get_mut(&doc_id) {
            state.is_dirty = dirty;
        }
    }

    /// Process a VFS Modified event for a document.
    ///
    /// Compares the new mtime against the stored snapshot and emits an
    /// `ExternalChange` event if the mtime has actually changed.
    ///
    /// Addresses: Requirement 2 AC 4–6, Requirement 3 AC 1, AC 6
    pub fn process_modified_event(
        &mut self,
        doc_id: DocumentId,
        current_mtime: Option<SystemTime>,
    ) -> Option<ExternalChange> {
        let comparison = self.mtime_tracker.check_mtime(doc_id, current_mtime);

        match comparison {
            MtimeComparison::Unchanged => {
                // Spurious event — discard (Requirement 2 AC 6)
                None
            }
            MtimeComparison::Changed { old, new } => {
                let state = self.document_states.get_mut(&doc_id)?;

                // Deduplication: suppress if already pending for same mtime (Req 3 AC 6)
                if state.pending_notification {
                    return None;
                }

                // Check last_asked_mtime to prevent re-prompting for dismissed changes
                if state.last_asked_mtime == Some(new) {
                    return None;
                }

                state.pending_notification = true;

                Some(ExternalChange::content_changed(
                    doc_id,
                    old,
                    new,
                    state.is_dirty,
                ))
            }
            MtimeComparison::StatFailed(_reason) => {
                // WARN-level logging: treat as potentially changed (Req 2 AC 8)
                let state = self.document_states.get_mut(&doc_id)?;
                if state.pending_notification {
                    return None;
                }

                // Use current time as a sentinel for the "unknown new mtime" case
                let old_mtime = self
                    .mtime_tracker
                    .get_snapshot(doc_id)
                    .map(|s| s.mtime)
                    .unwrap_or(SystemTime::UNIX_EPOCH);

                state.pending_notification = true;
                Some(ExternalChange::content_changed(
                    doc_id,
                    old_mtime,
                    SystemTime::now(),
                    state.is_dirty,
                ))
            }
        }
    }

    /// Process a VFS Deleted event for a document.
    ///
    /// Addresses: Requirement 6 AC 1
    pub fn process_deleted_event(&mut self, doc_id: DocumentId) -> Option<ExternalChange> {
        let state = self.document_states.get_mut(&doc_id)?;
        let is_dirty = state.is_dirty;
        state.pending_notification = true;
        Some(ExternalChange::file_deleted(doc_id, is_dirty))
    }

    /// Process a VFS Renamed event for a document.
    ///
    /// Addresses: Requirement 7 AC 1
    pub fn process_renamed_event(
        &mut self,
        doc_id: DocumentId,
        old_uri: ResourceUri,
        new_uri: ResourceUri,
    ) -> Option<ExternalChange> {
        let state = self.document_states.get_mut(&doc_id)?;
        let is_dirty = state.is_dirty;
        state.pending_notification = true;
        Some(ExternalChange::file_renamed(
            doc_id, old_uri, new_uri, is_dirty,
        ))
    }

    /// Mark a notification as responded to (user has answered the prompt).
    ///
    /// Updates the mtime snapshot and clears the pending state.
    ///
    /// Addresses: Requirement 3 AC 7
    pub fn mark_responded(&mut self, doc_id: DocumentId, current_mtime: SystemTime) {
        if let Some(state) = self.document_states.get_mut(&doc_id) {
            state.pending_notification = false;
            state.last_asked_mtime = Some(current_mtime);
        }
        self.mtime_tracker.update_snapshot(doc_id, current_mtime);
    }

    /// Update configuration (hot-reload callback).
    ///
    /// Addresses: Requirement 10 AC 8
    pub fn update_config(&mut self, new_config: ExternalModConfig) {
        self.config = new_config;
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &ExternalModConfig {
        &self.config
    }

    /// Returns true if a document has a pending notification.
    pub fn has_pending_notification(&self, doc_id: DocumentId) -> bool {
        self.document_states
            .get(&doc_id)
            .map(|s| s.pending_notification)
            .unwrap_or(false)
    }

    /// Look up a document ID by resource URI.
    pub fn find_document_by_uri(&self, uri: &ResourceUri) -> Option<DocumentId> {
        self.document_states
            .iter()
            .find(|(_, state)| state.uri == *uri)
            .map(|(&id, _)| id)
    }

    /// Get the URI for a registered document.
    pub fn get_document_uri(&self, doc_id: DocumentId) -> Option<&ResourceUri> {
        self.document_states.get(&doc_id).map(|s| &s.uri)
    }

    /// Shut down the detector, cancelling all watches.
    pub fn shutdown(&mut self) {
        self.watch_registry.cancel_all();
        self.document_states.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    use crate::change_event::ChangeType;

    fn make_uri(path: &str) -> ResourceUri {
        ResourceUri::new("local", path)
    }

    fn make_watch_handle() -> WatchHandle {
        let (_, rx) = mpsc::channel(1);
        let token = CancellationToken::new();
        WatchHandle::new(rx, token)
    }

    // --- WatchRegistry tests ---

    #[test]
    fn watch_registry_register_and_count() {
        // Validates: Requirement 1.2
        let mut registry = WatchRegistry::new();
        assert_eq!(registry.active_watch_count(), 0);

        registry.register_watch(DocumentId(1), make_watch_handle());
        assert_eq!(registry.active_watch_count(), 1);
        assert!(registry.has_watch(DocumentId(1)));
    }

    #[test]
    fn watch_registry_cancel_removes_watch() {
        // Validates: Requirement 1.3
        let mut registry = WatchRegistry::new();
        registry.register_watch(DocumentId(1), make_watch_handle());

        registry.cancel_watch(DocumentId(1));
        assert_eq!(registry.active_watch_count(), 0);
        assert!(!registry.has_watch(DocumentId(1)));
    }

    #[test]
    fn watch_registry_cancel_all_clears_everything() {
        let mut registry = WatchRegistry::new();
        registry.register_watch(DocumentId(1), make_watch_handle());
        registry.register_watch(DocumentId(2), make_watch_handle());
        registry.register_polling(DocumentId(3));

        registry.cancel_all();
        assert_eq!(registry.active_watch_count(), 0);
        assert_eq!(registry.polling_count(), 0);
    }

    #[test]
    fn watch_registry_polling_fallback() {
        // Validates: Requirement 1.5
        let mut registry = WatchRegistry::new();
        registry.register_polling(DocumentId(1));

        assert!(registry.uses_polling(DocumentId(1)));
        assert_eq!(registry.polling_count(), 1);
        assert!(!registry.has_watch(DocumentId(1)));
    }

    #[test]
    fn watch_registry_cancel_removes_polling_too() {
        let mut registry = WatchRegistry::new();
        registry.register_polling(DocumentId(1));

        registry.cancel_watch(DocumentId(1));
        assert_eq!(registry.polling_count(), 0);
    }

    #[test]
    fn watch_registry_all_monitored_documents() {
        let mut registry = WatchRegistry::new();
        registry.register_watch(DocumentId(1), make_watch_handle());
        registry.register_polling(DocumentId(2));

        let mut docs = registry.all_monitored_documents();
        docs.sort_by_key(|id| id.0);
        assert_eq!(docs, vec![DocumentId(1), DocumentId(2)]);
    }

    // --- ExternalModificationDetector tests ---

    #[test]
    fn detector_register_and_unregister_document() {
        // Validates: Requirement 1.2, 1.3
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = detector.next_document_id();
        let uri = make_uri("/test/file.rs");
        let mtime = SystemTime::now();

        detector.register_document(doc_id, uri.clone(), mtime, Some(make_watch_handle()), false);
        assert_eq!(detector.watch_registry.active_watch_count(), 1);
        assert!(detector.mtime_tracker.get_snapshot(doc_id).is_some());

        detector.unregister_document(doc_id).unwrap();
        assert_eq!(detector.watch_registry.active_watch_count(), 0);
        assert!(detector.mtime_tracker.get_snapshot(doc_id).is_none());
    }

    #[test]
    fn detector_process_modified_event_unchanged_returns_none() {
        // Validates: Requirement 2.6 — spurious event filtering
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);
        let mtime = SystemTime::now();

        detector.register_document(doc_id, make_uri("/file.rs"), mtime, None, false);
        let result = detector.process_modified_event(doc_id, Some(mtime));

        assert!(result.is_none());
    }

    #[test]
    fn detector_process_modified_event_changed_emits_event() {
        // Validates: Requirement 3.1 — emit ExternalChange on mtime change
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);
        let old_mtime = SystemTime::UNIX_EPOCH;
        let new_mtime = SystemTime::now();

        detector.register_document(doc_id, make_uri("/file.rs"), old_mtime, None, false);
        let result = detector.process_modified_event(doc_id, Some(new_mtime));

        assert!(result.is_some());
        let change = result.unwrap();
        assert_eq!(change.document_id, doc_id);
        assert_eq!(change.change_type, ChangeType::ContentChanged);
        assert_eq!(change.old_mtime, Some(old_mtime));
        assert_eq!(change.new_mtime, Some(new_mtime));
        assert!(!change.is_dirty);
    }

    #[test]
    fn detector_deduplication_suppresses_second_event() {
        // Validates: Requirement 3.6 — at most one event per change
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);
        let old_mtime = SystemTime::UNIX_EPOCH;
        let new_mtime = SystemTime::now();

        detector.register_document(doc_id, make_uri("/file.rs"), old_mtime, None, false);

        // First event should emit
        let first = detector.process_modified_event(doc_id, Some(new_mtime));
        assert!(first.is_some());

        // Second event for same change should be suppressed (pending notification)
        let second = detector.process_modified_event(doc_id, Some(new_mtime));
        assert!(second.is_none());
    }

    #[test]
    fn detector_last_asked_mtime_prevents_re_prompting() {
        // Validates: Requirement 3.6 — last_asked_mtime check
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);
        let old_mtime = SystemTime::UNIX_EPOCH;
        let new_mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(100);

        detector.register_document(doc_id, make_uri("/file.rs"), old_mtime, None, false);

        // First event emits
        let first = detector.process_modified_event(doc_id, Some(new_mtime));
        assert!(first.is_some());

        // User responds
        detector.mark_responded(doc_id, new_mtime);

        // Same mtime arrives again — should be suppressed (last_asked_mtime)
        let again = detector.process_modified_event(doc_id, Some(new_mtime));
        assert!(again.is_none());
    }

    #[test]
    fn detector_dirty_state_enrichment() {
        // Validates: Requirement 3.1 — dirty state in event
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);
        let old_mtime = SystemTime::UNIX_EPOCH;
        let new_mtime = SystemTime::now();

        detector.register_document(doc_id, make_uri("/file.rs"), old_mtime, None, false);
        detector.set_dirty(doc_id, true);

        let result = detector.process_modified_event(doc_id, Some(new_mtime));
        assert!(result.is_some());
        assert!(result.unwrap().is_dirty);
    }

    #[test]
    fn detector_process_deleted_event() {
        // Validates: Requirement 6.1
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);

        detector.register_document(doc_id, make_uri("/file.rs"), SystemTime::now(), None, false);
        let result = detector.process_deleted_event(doc_id);

        assert!(result.is_some());
        let change = result.unwrap();
        assert_eq!(change.change_type, ChangeType::FileDeleted);
    }

    #[test]
    fn detector_process_renamed_event() {
        // Validates: Requirement 7.1
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);
        let old_uri = make_uri("/old/file.rs");
        let new_uri = make_uri("/new/file.rs");

        detector.register_document(doc_id, old_uri.clone(), SystemTime::now(), None, false);
        let result = detector.process_renamed_event(doc_id, old_uri.clone(), new_uri.clone());

        assert!(result.is_some());
        let change = result.unwrap();
        assert_eq!(
            change.change_type,
            ChangeType::FileRenamed { old_uri, new_uri }
        );
    }

    #[test]
    fn detector_notify_document_saved_updates_snapshot() {
        // Validates: Requirement 2.3
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);
        let old_mtime = SystemTime::UNIX_EPOCH;
        let save_mtime = SystemTime::now();

        detector.register_document(doc_id, make_uri("/file.rs"), old_mtime, None, false);
        detector.set_dirty(doc_id, true);
        detector.notify_document_saved(doc_id, save_mtime);

        let snapshot = detector.mtime_tracker.get_snapshot(doc_id).unwrap();
        assert_eq!(snapshot.mtime, save_mtime);
    }

    #[test]
    fn detector_find_document_by_uri() {
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);
        let uri = make_uri("/file.rs");

        detector.register_document(doc_id, uri.clone(), SystemTime::now(), None, false);
        assert_eq!(detector.find_document_by_uri(&uri), Some(doc_id));
        assert_eq!(detector.find_document_by_uri(&make_uri("/other.rs")), None);
    }

    #[test]
    fn detector_mark_responded_clears_pending_and_updates_mtime() {
        // Validates: Requirement 3.7
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);
        let old_mtime = SystemTime::UNIX_EPOCH;
        let new_mtime = SystemTime::now();

        detector.register_document(doc_id, make_uri("/file.rs"), old_mtime, None, false);
        detector.process_modified_event(doc_id, Some(new_mtime));
        assert!(detector.has_pending_notification(doc_id));

        detector.mark_responded(doc_id, new_mtime);
        assert!(!detector.has_pending_notification(doc_id));

        let snapshot = detector.mtime_tracker.get_snapshot(doc_id).unwrap();
        assert_eq!(snapshot.mtime, new_mtime);
    }

    #[test]
    fn detector_shutdown_cancels_all() {
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        detector.register_document(
            DocumentId(1),
            make_uri("/a.rs"),
            SystemTime::now(),
            Some(make_watch_handle()),
            false,
        );
        detector.register_document(
            DocumentId(2),
            make_uri("/b.rs"),
            SystemTime::now(),
            Some(make_watch_handle()),
            false,
        );

        detector.shutdown();
        assert_eq!(detector.watch_registry.active_watch_count(), 0);
        assert!(detector.document_states.is_empty());
    }

    #[test]
    fn detector_stat_failed_emits_event_pessimistically() {
        // Validates: Requirement 2.8 — pessimistic assumption on stat failure
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);

        detector.register_document(doc_id, make_uri("/file.rs"), SystemTime::now(), None, false);
        let result = detector.process_modified_event(doc_id, None);

        // StatFailed with no mtime should still emit event (pessimistic)
        assert!(result.is_some());
    }
}
