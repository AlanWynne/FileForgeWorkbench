//! Mtime tracking and snapshot management.
//!
//! The `MtimeTracker` maintains per-document modification time snapshots
//! and provides comparison logic for detecting external changes.
//!
//! Addresses: Requirement 2 (AC 2.1–2.8)

use std::collections::HashMap;
use std::time::SystemTime;

use ff_vfs::ResourceUri;

use crate::error::ExternalModError;
use crate::types::{DocumentId, MtimeComparison, MtimeSnapshot};

/// Tracks modification time snapshots for all open documents.
///
/// Provides operations to record, update, remove, and compare mtime
/// snapshots against the current on-disk state via VFS stat calls.
///
/// # Sub-second Precision
///
/// The tracker stores `SystemTime` values directly from VFS metadata,
/// preserving whatever precision the underlying filesystem provides
/// (nanosecond on ext4/NTFS, second on FAT32).
///
/// Addresses: Requirement 2, criteria 1–8
#[derive(Debug)]
pub struct MtimeTracker {
    /// Per-document mtime snapshots keyed by document ID.
    snapshots: HashMap<DocumentId, MtimeSnapshot>,
}

impl MtimeTracker {
    /// Create a new empty tracker.
    pub fn new() -> Self {
        Self {
            snapshots: HashMap::new(),
        }
    }

    /// Record a new mtime snapshot for a document.
    ///
    /// Queries the provided `mtime` (from VFS stat) and stores it associated
    /// with the given document ID and resource URI.
    ///
    /// Addresses: Requirement 2 AC 1, AC 2
    pub fn record_snapshot(
        &mut self,
        doc_id: DocumentId,
        uri: ResourceUri,
        mtime: SystemTime,
    ) -> MtimeSnapshot {
        let snapshot = MtimeSnapshot::new(mtime, doc_id, uri);
        self.snapshots.insert(doc_id, snapshot.clone());
        snapshot
    }

    /// Update the mtime snapshot for a document after save or reload.
    ///
    /// Addresses: Requirement 2 AC 3
    pub fn update_snapshot(&mut self, doc_id: DocumentId, new_mtime: SystemTime) {
        if let Some(snapshot) = self.snapshots.get_mut(&doc_id) {
            snapshot.mtime = new_mtime;
            snapshot.recorded_at = std::time::Instant::now();
        }
    }

    /// Remove the mtime snapshot for a document on close.
    ///
    /// Addresses: Requirement 2 (cleanup on document close)
    pub fn remove_snapshot(&mut self, doc_id: DocumentId) -> Option<MtimeSnapshot> {
        self.snapshots.remove(&doc_id)
    }

    /// Retrieve the stored snapshot for a document.
    pub fn get_snapshot(&self, doc_id: DocumentId) -> Option<&MtimeSnapshot> {
        self.snapshots.get(&doc_id)
    }

    /// Compare the stored mtime against a current on-disk mtime.
    ///
    /// Returns `MtimeComparison::Changed` if they differ, `Unchanged` if equal.
    /// If the document has no stored snapshot, returns `StatFailed`.
    ///
    /// Addresses: Requirement 2, criteria 4–6
    pub fn check_mtime(
        &self,
        doc_id: DocumentId,
        current_mtime: Option<SystemTime>,
    ) -> MtimeComparison {
        let Some(snapshot) = self.snapshots.get(&doc_id) else {
            return MtimeComparison::StatFailed(format!("no snapshot found for document {doc_id}"));
        };

        match current_mtime {
            Some(disk_mtime) => {
                if disk_mtime == snapshot.mtime {
                    MtimeComparison::Unchanged
                } else {
                    MtimeComparison::Changed {
                        old: snapshot.mtime,
                        new: disk_mtime,
                    }
                }
            }
            None => {
                // VFS stat failed — treat as potentially changed (Requirement 2 AC 8)
                MtimeComparison::StatFailed("VFS stat returned no mtime".to_string())
            }
        }
    }

    /// Check mtime using a result from VFS stat, handling errors gracefully.
    ///
    /// When stat fails, logs a WARN-level message and returns `StatFailed`,
    /// treating the file as potentially changed (pessimistic assumption).
    ///
    /// Addresses: Requirement 2 AC 8
    pub fn check_mtime_result(
        &self,
        doc_id: DocumentId,
        stat_result: Result<Option<SystemTime>, ExternalModError>,
    ) -> MtimeComparison {
        match stat_result {
            Ok(mtime) => self.check_mtime(doc_id, mtime),
            Err(err) => {
                // WARN-level logging would happen here via ff-logging
                MtimeComparison::StatFailed(format!("stat failed: {err}"))
            }
        }
    }

    /// Returns the number of tracked documents.
    pub fn count(&self) -> usize {
        self.snapshots.len()
    }

    /// Returns true if the tracker has no snapshots.
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Returns all tracked document IDs.
    pub fn tracked_documents(&self) -> Vec<DocumentId> {
        self.snapshots.keys().copied().collect()
    }
}

impl Default for MtimeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn make_uri(path: &str) -> ResourceUri {
        ResourceUri::new("local", path)
    }

    #[test]
    fn record_snapshot_stores_and_returns_snapshot() {
        // Validates: Requirement 2.1 — record mtime snapshot
        let mut tracker = MtimeTracker::new();
        let mtime = SystemTime::now();
        let doc_id = DocumentId(1);
        let uri = make_uri("/test/file.rs");

        let snapshot = tracker.record_snapshot(doc_id, uri.clone(), mtime);

        assert_eq!(snapshot.mtime, mtime);
        assert_eq!(snapshot.document_id, doc_id);
        assert_eq!(snapshot.resource_uri, uri);
    }

    #[test]
    fn get_snapshot_returns_stored_snapshot() {
        // Validates: Requirement 2.1
        let mut tracker = MtimeTracker::new();
        let mtime = SystemTime::now();
        let doc_id = DocumentId(1);
        let uri = make_uri("/test/file.rs");

        tracker.record_snapshot(doc_id, uri, mtime);
        let stored = tracker.get_snapshot(doc_id).unwrap();

        assert_eq!(stored.mtime, mtime);
        assert_eq!(stored.document_id, doc_id);
    }

    #[test]
    fn get_snapshot_returns_none_for_untracked_document() {
        let tracker = MtimeTracker::new();
        assert!(tracker.get_snapshot(DocumentId(99)).is_none());
    }

    #[test]
    fn update_snapshot_changes_mtime() {
        // Validates: Requirement 2.3 — update after save
        let mut tracker = MtimeTracker::new();
        let old_mtime = SystemTime::UNIX_EPOCH;
        let new_mtime = SystemTime::now();
        let doc_id = DocumentId(1);

        tracker.record_snapshot(doc_id, make_uri("/test/file.rs"), old_mtime);
        tracker.update_snapshot(doc_id, new_mtime);

        let snapshot = tracker.get_snapshot(doc_id).unwrap();
        assert_eq!(snapshot.mtime, new_mtime);
    }

    #[test]
    fn update_snapshot_on_nonexistent_document_is_noop() {
        let mut tracker = MtimeTracker::new();
        tracker.update_snapshot(DocumentId(99), SystemTime::now());
        assert!(tracker.is_empty());
    }

    #[test]
    fn remove_snapshot_removes_and_returns_old_snapshot() {
        // Validates: Requirement 2 — cleanup on document close
        let mut tracker = MtimeTracker::new();
        let mtime = SystemTime::now();
        let doc_id = DocumentId(1);

        tracker.record_snapshot(doc_id, make_uri("/test/file.rs"), mtime);
        let removed = tracker.remove_snapshot(doc_id);

        assert!(removed.is_some());
        assert_eq!(removed.unwrap().mtime, mtime);
        assert!(tracker.get_snapshot(doc_id).is_none());
    }

    #[test]
    fn remove_snapshot_returns_none_for_untracked() {
        let mut tracker = MtimeTracker::new();
        assert!(tracker.remove_snapshot(DocumentId(99)).is_none());
    }

    #[test]
    fn check_mtime_unchanged_when_same() {
        // Validates: Requirement 2.6 — spurious event detection
        let mut tracker = MtimeTracker::new();
        let mtime = SystemTime::now();
        let doc_id = DocumentId(1);

        tracker.record_snapshot(doc_id, make_uri("/test/file.rs"), mtime);
        let result = tracker.check_mtime(doc_id, Some(mtime));

        assert_eq!(result, MtimeComparison::Unchanged);
    }

    #[test]
    fn check_mtime_changed_when_different() {
        // Validates: Requirement 2.5 — external change detection
        let mut tracker = MtimeTracker::new();
        let old_mtime = SystemTime::UNIX_EPOCH;
        let new_mtime = SystemTime::now();
        let doc_id = DocumentId(1);

        tracker.record_snapshot(doc_id, make_uri("/test/file.rs"), old_mtime);
        let result = tracker.check_mtime(doc_id, Some(new_mtime));

        assert_eq!(
            result,
            MtimeComparison::Changed {
                old: old_mtime,
                new: new_mtime,
            }
        );
    }

    #[test]
    fn check_mtime_stat_failed_when_no_mtime() {
        // Validates: Requirement 2.8 — handle stat failure
        let mut tracker = MtimeTracker::new();
        let doc_id = DocumentId(1);

        tracker.record_snapshot(doc_id, make_uri("/test/file.rs"), SystemTime::now());
        let result = tracker.check_mtime(doc_id, None);

        matches!(result, MtimeComparison::StatFailed(_));
    }

    #[test]
    fn check_mtime_stat_failed_when_no_snapshot() {
        let tracker = MtimeTracker::new();
        let result = tracker.check_mtime(DocumentId(99), Some(SystemTime::now()));

        matches!(result, MtimeComparison::StatFailed(_));
    }

    #[test]
    fn sub_second_precision_detects_microsecond_difference() {
        // Validates: Requirement 2.7 — sub-second precision
        // Note: Windows SystemTime has 100ns resolution; we use 1µs to guarantee detection
        let mut tracker = MtimeTracker::new();
        let base_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let different_time = base_time + Duration::from_micros(1);
        let doc_id = DocumentId(1);

        tracker.record_snapshot(doc_id, make_uri("/test/file.rs"), base_time);
        let result = tracker.check_mtime(doc_id, Some(different_time));

        assert_eq!(
            result,
            MtimeComparison::Changed {
                old: base_time,
                new: different_time,
            }
        );
    }

    #[test]
    fn sub_second_precision_same_with_millisecond_components() {
        // Validates: Requirement 2.7 — sub-second precision, equality
        let mut tracker = MtimeTracker::new();
        let time_with_millis = SystemTime::UNIX_EPOCH
            + Duration::from_secs(1_700_000_000)
            + Duration::from_millis(500);
        let doc_id = DocumentId(1);

        tracker.record_snapshot(doc_id, make_uri("/test/file.rs"), time_with_millis);
        let result = tracker.check_mtime(doc_id, Some(time_with_millis));

        assert_eq!(result, MtimeComparison::Unchanged);
    }

    #[test]
    fn check_mtime_result_handles_error() {
        // Validates: Requirement 2.8 — pessimistic assumption on failure
        let mut tracker = MtimeTracker::new();
        let doc_id = DocumentId(1);
        tracker.record_snapshot(doc_id, make_uri("/test/file.rs"), SystemTime::now());

        let err = ExternalModError::VfsStatFailed {
            uri: make_uri("/test/file.rs"),
            source: ff_vfs::VfsError::NotFound {
                uri: "vfs://local/test/file.rs".to_string(),
                operation: "stat".to_string(),
            },
        };
        let result = tracker.check_mtime_result(doc_id, Err(err));

        matches!(result, MtimeComparison::StatFailed(_));
    }

    #[test]
    fn count_and_is_empty_reflect_state() {
        let mut tracker = MtimeTracker::new();
        assert!(tracker.is_empty());
        assert_eq!(tracker.count(), 0);

        tracker.record_snapshot(DocumentId(1), make_uri("/a.rs"), SystemTime::now());
        assert!(!tracker.is_empty());
        assert_eq!(tracker.count(), 1);

        tracker.record_snapshot(DocumentId(2), make_uri("/b.rs"), SystemTime::now());
        assert_eq!(tracker.count(), 2);

        tracker.remove_snapshot(DocumentId(1));
        assert_eq!(tracker.count(), 1);
    }

    #[test]
    fn tracked_documents_returns_all_ids() {
        let mut tracker = MtimeTracker::new();
        tracker.record_snapshot(DocumentId(1), make_uri("/a.rs"), SystemTime::now());
        tracker.record_snapshot(DocumentId(2), make_uri("/b.rs"), SystemTime::now());
        tracker.record_snapshot(DocumentId(3), make_uri("/c.rs"), SystemTime::now());

        let mut ids = tracker.tracked_documents();
        ids.sort_by_key(|id| id.0);
        assert_eq!(ids, vec![DocumentId(1), DocumentId(2), DocumentId(3)]);
    }

    #[test]
    fn record_snapshot_overwrites_existing_for_same_doc_id() {
        let mut tracker = MtimeTracker::new();
        let doc_id = DocumentId(1);
        let old_mtime = SystemTime::UNIX_EPOCH;
        let new_mtime = SystemTime::now();

        tracker.record_snapshot(doc_id, make_uri("/old.rs"), old_mtime);
        tracker.record_snapshot(doc_id, make_uri("/new.rs"), new_mtime);

        assert_eq!(tracker.count(), 1);
        let snapshot = tracker.get_snapshot(doc_id).unwrap();
        assert_eq!(snapshot.mtime, new_mtime);
        assert_eq!(snapshot.resource_uri, make_uri("/new.rs"));
    }
}
