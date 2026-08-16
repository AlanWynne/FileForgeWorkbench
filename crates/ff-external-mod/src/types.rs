//! Core types for the external modification detection system.
//!
//! Contains `DocumentId`, `MtimeSnapshot`, and `DocumentRegistration` used
//! throughout the crate to track open documents and their external state.

use std::time::{Instant, SystemTime};

use ff_vfs::ResourceUri;

/// Opaque identifier for an open document within the external modification system.
///
/// Maps 1:1 to a `DocumentHandle` in `ff-document-model`. Used internally to
/// decouple detection logic from the full document API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentId(pub u64);

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DocumentId({})", self.0)
    }
}

/// A recorded modification timestamp for a document's backing file.
///
/// Used as the baseline for detecting external changes. The snapshot records
/// the mtime from VFS stat with sub-second precision where supported.
///
/// Addresses: Requirement 2, criteria 1–8
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MtimeSnapshot {
    /// The modification time from VFS stat, with sub-second precision where supported.
    pub mtime: SystemTime,

    /// The document identifier this snapshot belongs to.
    pub document_id: DocumentId,

    /// The resource URI for the backing file.
    pub resource_uri: ResourceUri,

    /// When this snapshot was recorded (monotonic clock for internal timing).
    pub recorded_at: Instant,
}

impl MtimeSnapshot {
    /// Create a new snapshot from a successful VFS stat result.
    pub fn new(mtime: SystemTime, document_id: DocumentId, resource_uri: ResourceUri) -> Self {
        Self {
            mtime,
            document_id,
            resource_uri,
            recorded_at: Instant::now(),
        }
    }
}

/// The result of comparing a stored mtime against the current on-disk mtime.
///
/// Addresses: Requirement 2, criteria 4–6
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MtimeComparison {
    /// The on-disk mtime matches the stored snapshot — no external change.
    Unchanged,

    /// The on-disk mtime differs from the stored snapshot — external change detected.
    Changed {
        /// The previously stored mtime.
        old: SystemTime,
        /// The new mtime detected on disk.
        new: SystemTime,
    },

    /// VFS stat failed — unable to determine mtime, treat as potentially changed.
    StatFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_id_display_shows_inner_value() {
        let id = DocumentId(42);
        assert_eq!(format!("{id}"), "DocumentId(42)");
    }

    #[test]
    fn document_id_equality_checks_inner_value() {
        assert_eq!(DocumentId(1), DocumentId(1));
        assert_ne!(DocumentId(1), DocumentId(2));
    }

    #[test]
    fn document_id_hash_is_consistent() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(DocumentId(1), "first");
        map.insert(DocumentId(2), "second");
        assert_eq!(map.get(&DocumentId(1)), Some(&"first"));
        assert_eq!(map.get(&DocumentId(2)), Some(&"second"));
    }

    #[test]
    fn mtime_snapshot_new_records_current_time() {
        let mtime = SystemTime::now();
        let doc_id = DocumentId(1);
        let uri = ResourceUri::new("local", "/test/file.rs");
        let before = Instant::now();
        let snapshot = MtimeSnapshot::new(mtime, doc_id, uri.clone());
        let after = Instant::now();

        assert_eq!(snapshot.mtime, mtime);
        assert_eq!(snapshot.document_id, doc_id);
        assert_eq!(snapshot.resource_uri, uri);
        assert!(snapshot.recorded_at >= before);
        assert!(snapshot.recorded_at <= after);
    }

    #[test]
    fn mtime_comparison_unchanged_variant() {
        let cmp = MtimeComparison::Unchanged;
        assert_eq!(cmp, MtimeComparison::Unchanged);
    }

    #[test]
    fn mtime_comparison_changed_variant_holds_both_times() {
        let old = SystemTime::UNIX_EPOCH;
        let new = SystemTime::now();
        let cmp = MtimeComparison::Changed { old, new };
        if let MtimeComparison::Changed { old: o, new: n } = cmp {
            assert_eq!(o, old);
            assert_eq!(n, new);
        } else {
            panic!("Expected Changed variant");
        }
    }

    #[test]
    fn mtime_comparison_stat_failed_holds_message() {
        let cmp = MtimeComparison::StatFailed("permission denied".to_string());
        if let MtimeComparison::StatFailed(msg) = cmp {
            assert_eq!(msg, "permission denied");
        } else {
            panic!("Expected StatFailed variant");
        }
    }
}
