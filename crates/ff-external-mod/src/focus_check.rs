//! Focus-gained and tab-switch mtime revalidation.
//!
//! The `FocusGainedChecker` performs mtime checks on open documents when the
//! application window gains focus, ensuring detection even if VFS watch events
//! were missed or delayed.
//!
//! Addresses: Requirement 9, criteria 1–7

use std::time::SystemTime;

use crate::change_event::ExternalChange;
use crate::config::ExternalModConfig;
use crate::mtime_tracker::MtimeTracker;
use crate::types::{DocumentId, MtimeComparison};

/// Result of a focus-gained mtime scan for a single document.
#[derive(Debug, Clone, PartialEq)]
pub enum FocusCheckResult {
    /// No change detected.
    Unchanged,
    /// External change detected.
    Changed(Box<ExternalChange>),
    /// Stat failed — treated as potentially changed.
    StatFailed(DocumentId),
    /// Skipped because user already dismissed this change.
    AlreadyDismissed,
}

/// Coordinates synchronous mtime validation on window focus and tab switch.
///
/// Performs bulk mtime checks against the stored snapshots and emits
/// `ExternalChange` events for any mismatches.
///
/// Addresses: Requirement 9, criteria 1–7
#[derive(Debug)]
pub struct FocusGainedChecker {
    /// Whether focus-gained checks are enabled.
    enabled: bool,
}

impl FocusGainedChecker {
    /// Create a new checker with the given configuration.
    pub fn new(config: &ExternalModConfig) -> Self {
        Self {
            enabled: config.check_on_focus,
        }
    }

    /// Whether focus-gained checks are enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Update the enabled state (hot-reload).
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check a single document's mtime against its stored snapshot.
    ///
    /// Returns a `FocusCheckResult` indicating whether a change was detected.
    /// Respects the `last_asked_mtime` to avoid re-prompting for dismissed changes.
    ///
    /// Addresses: Requirement 9 AC 6, AC 7
    pub fn check_document(
        &self,
        doc_id: DocumentId,
        current_mtime: Option<SystemTime>,
        tracker: &MtimeTracker,
        last_asked_mtime: Option<SystemTime>,
        is_dirty: bool,
    ) -> FocusCheckResult {
        if !self.enabled {
            return FocusCheckResult::Unchanged;
        }

        let comparison = tracker.check_mtime(doc_id, current_mtime);

        match comparison {
            MtimeComparison::Unchanged => FocusCheckResult::Unchanged,
            MtimeComparison::Changed { old, new } => {
                // Check if user already dismissed this specific mtime (Req 9 AC 7)
                if last_asked_mtime == Some(new) {
                    return FocusCheckResult::AlreadyDismissed;
                }
                FocusCheckResult::Changed(Box::new(ExternalChange::content_changed(
                    doc_id, old, new, is_dirty,
                )))
            }
            MtimeComparison::StatFailed(_) => FocusCheckResult::StatFailed(doc_id),
        }
    }

    /// Perform a bulk mtime check on all provided documents.
    ///
    /// Returns a list of documents that have detected changes.
    /// Skips documents where the change was already dismissed.
    ///
    /// Addresses: Requirement 9 AC 1–3
    pub fn check_all(
        &self,
        documents: &[(DocumentId, Option<SystemTime>, Option<SystemTime>, bool)],
        tracker: &MtimeTracker,
    ) -> Vec<ExternalChange> {
        if !self.enabled {
            return Vec::new();
        }

        let mut changes = Vec::new();

        for &(doc_id, current_mtime, last_asked, is_dirty) in documents {
            if let FocusCheckResult::Changed(event) =
                self.check_document(doc_id, current_mtime, tracker, last_asked, is_dirty)
            {
                changes.push(*event);
            }
        }

        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use ff_vfs::ResourceUri;

    fn make_uri(path: &str) -> ResourceUri {
        ResourceUri::new("local", path)
    }

    fn make_tracker_with_doc(doc_id: DocumentId, mtime: SystemTime) -> MtimeTracker {
        let mut tracker = MtimeTracker::new();
        tracker.record_snapshot(doc_id, make_uri("/file.rs"), mtime);
        tracker
    }

    #[test]
    fn check_document_unchanged_when_mtime_matches() {
        // Validates: Requirement 9.2
        let config = ExternalModConfig::default();
        let checker = FocusGainedChecker::new(&config);
        let mtime = SystemTime::now();
        let tracker = make_tracker_with_doc(DocumentId(1), mtime);

        let result = checker.check_document(DocumentId(1), Some(mtime), &tracker, None, false);
        assert_eq!(result, FocusCheckResult::Unchanged);
    }

    #[test]
    fn check_document_detects_changed_mtime() {
        // Validates: Requirement 9.2
        let config = ExternalModConfig::default();
        let checker = FocusGainedChecker::new(&config);
        let old_mtime = SystemTime::UNIX_EPOCH;
        let new_mtime = SystemTime::now();
        let tracker = make_tracker_with_doc(DocumentId(1), old_mtime);

        let result = checker.check_document(DocumentId(1), Some(new_mtime), &tracker, None, false);
        match result {
            FocusCheckResult::Changed(event) => {
                assert_eq!(event.document_id, DocumentId(1));
                assert_eq!(event.old_mtime, Some(old_mtime));
                assert_eq!(event.new_mtime, Some(new_mtime));
            }
            _ => panic!("Expected Changed result"),
        }
    }

    #[test]
    fn check_document_skips_when_disabled() {
        // Validates: Requirement 9.5 — check_on_focus config
        let mut config = ExternalModConfig::default();
        config.check_on_focus = false;
        let checker = FocusGainedChecker::new(&config);
        let old_mtime = SystemTime::UNIX_EPOCH;
        let new_mtime = SystemTime::now();
        let tracker = make_tracker_with_doc(DocumentId(1), old_mtime);

        let result = checker.check_document(DocumentId(1), Some(new_mtime), &tracker, None, false);
        assert_eq!(result, FocusCheckResult::Unchanged);
    }

    #[test]
    fn check_document_respects_last_asked_mtime() {
        // Validates: Requirement 9.7 — do not re-prompt for dismissed changes
        let config = ExternalModConfig::default();
        let checker = FocusGainedChecker::new(&config);
        let old_mtime = SystemTime::UNIX_EPOCH;
        let new_mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let tracker = make_tracker_with_doc(DocumentId(1), old_mtime);

        // User already dismissed this particular mtime change
        let result = checker.check_document(
            DocumentId(1),
            Some(new_mtime),
            &tracker,
            Some(new_mtime),
            false,
        );
        assert_eq!(result, FocusCheckResult::AlreadyDismissed);
    }

    #[test]
    fn check_document_stat_failed_returns_stat_failed() {
        // Validates: Requirement 9 — handle stat failures
        let config = ExternalModConfig::default();
        let checker = FocusGainedChecker::new(&config);
        let tracker = make_tracker_with_doc(DocumentId(1), SystemTime::now());

        let result = checker.check_document(DocumentId(1), None, &tracker, None, false);
        assert_eq!(result, FocusCheckResult::StatFailed(DocumentId(1)));
    }

    #[test]
    fn check_all_returns_only_changed_documents() {
        // Validates: Requirement 9.1 — bulk mtime check
        let config = ExternalModConfig::default();
        let checker = FocusGainedChecker::new(&config);

        let old = SystemTime::UNIX_EPOCH;
        let current = SystemTime::now();

        let mut tracker = MtimeTracker::new();
        tracker.record_snapshot(DocumentId(1), make_uri("/a.rs"), old);
        tracker.record_snapshot(DocumentId(2), make_uri("/b.rs"), current);
        tracker.record_snapshot(DocumentId(3), make_uri("/c.rs"), old);

        let documents = vec![
            (DocumentId(1), Some(current), None, false), // changed
            (DocumentId(2), Some(current), None, false), // unchanged
            (DocumentId(3), Some(current), None, true),  // changed + dirty
        ];

        let changes = checker.check_all(&documents, &tracker);
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].document_id, DocumentId(1));
        assert_eq!(changes[1].document_id, DocumentId(3));
        assert!(changes[1].is_dirty);
    }

    #[test]
    fn check_all_returns_empty_when_disabled() {
        // Validates: Requirement 9.5
        let mut config = ExternalModConfig::default();
        config.check_on_focus = false;
        let checker = FocusGainedChecker::new(&config);
        let tracker = make_tracker_with_doc(DocumentId(1), SystemTime::UNIX_EPOCH);

        let documents = vec![(DocumentId(1), Some(SystemTime::now()), None, false)];
        let changes = checker.check_all(&documents, &tracker);
        assert!(changes.is_empty());
    }

    #[test]
    fn check_all_excludes_dismissed_changes() {
        // Validates: Requirement 9.7
        let config = ExternalModConfig::default();
        let checker = FocusGainedChecker::new(&config);

        let old = SystemTime::UNIX_EPOCH;
        let new = SystemTime::UNIX_EPOCH + Duration::from_secs(100);

        let mut tracker = MtimeTracker::new();
        tracker.record_snapshot(DocumentId(1), make_uri("/a.rs"), old);
        tracker.record_snapshot(DocumentId(2), make_uri("/b.rs"), old);

        let documents = vec![
            (DocumentId(1), Some(new), Some(new), false), // dismissed
            (DocumentId(2), Some(new), None, false),      // not dismissed
        ];

        let changes = checker.check_all(&documents, &tracker);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].document_id, DocumentId(2));
    }

    #[test]
    fn set_enabled_toggles_checker() {
        let config = ExternalModConfig::default();
        let mut checker = FocusGainedChecker::new(&config);
        assert!(checker.is_enabled());

        checker.set_enabled(false);
        assert!(!checker.is_enabled());
    }

    #[test]
    fn check_document_includes_dirty_state_in_event() {
        // Validates: Requirement 9 — dirty state enrichment
        let config = ExternalModConfig::default();
        let checker = FocusGainedChecker::new(&config);
        let tracker = make_tracker_with_doc(DocumentId(1), SystemTime::UNIX_EPOCH);

        let result = checker.check_document(
            DocumentId(1),
            Some(SystemTime::now()),
            &tracker,
            None,
            true, // dirty
        );
        match result {
            FocusCheckResult::Changed(event) => assert!(event.is_dirty),
            _ => panic!("Expected Changed result"),
        }
    }
}
