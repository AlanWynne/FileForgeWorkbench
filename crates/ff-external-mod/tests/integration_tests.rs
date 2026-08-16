//! Integration tests for ff-external-mod.
//!
//! Tests end-to-end flows with the detector, policy engine, batch coalescer,
//! and focus-gained checker working together.

use std::time::{Duration, SystemTime};

use ff_external_mod::batch_coalescer::BatchCoalescer;
use ff_external_mod::change_event::ChangeType;
use ff_external_mod::config::ExternalModConfig;
use ff_external_mod::detector::ExternalModificationDetector;
use ff_external_mod::focus_check::FocusGainedChecker;
use ff_external_mod::mtime_tracker::MtimeTracker;
use ff_external_mod::prompt::BatchAction;
use ff_external_mod::reload_policy::{PolicyAction, ReloadPolicy, ReloadPolicyEngine};
use ff_external_mod::types::DocumentId;
use ff_vfs::ResourceUri;

fn make_uri(path: &str) -> ResourceUri {
    ResourceUri::new("local", path)
}

/// Integration test: full open → external modify → detect → prompt → reload cycle.
///
/// Validates: Requirements 1.2, 2.2, 2.4, 3.1, 3.2, 4.1, 3.7
#[test]
fn full_open_modify_detect_prompt_reload_cycle() {
    // Validates: Requirement 1.2, 2.2
    let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
    let doc_id = DocumentId(1);
    let uri = make_uri("/project/main.rs");
    let initial_mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

    // 1. Open document — register watch and record mtime
    detector.register_document(doc_id, uri.clone(), initial_mtime, None, false);
    assert!(detector.mtime_tracker.get_snapshot(doc_id).is_some());

    // 2. External modification occurs — new mtime detected
    let new_mtime = initial_mtime + Duration::from_secs(60);
    let change = detector.process_modified_event(doc_id, Some(new_mtime));
    assert!(change.is_some());

    let event = change.unwrap();
    assert_eq!(event.change_type, ChangeType::ContentChanged);
    assert_eq!(event.old_mtime, Some(initial_mtime));
    assert_eq!(event.new_mtime, Some(new_mtime));
    assert!(!event.is_dirty);

    // 3. Policy engine evaluates — prompt mode shows prompt
    let action =
        ReloadPolicyEngine::evaluate(ReloadPolicy::Prompt, event.is_dirty, &event.change_type);
    assert_eq!(action, PolicyAction::ShowPrompt);

    // 4. User selects "Reload" → mark responded, update mtime
    detector.mark_responded(doc_id, new_mtime);
    assert!(!detector.has_pending_notification(doc_id));

    // 5. Verify mtime snapshot updated
    let snapshot = detector.mtime_tracker.get_snapshot(doc_id).unwrap();
    assert_eq!(snapshot.mtime, new_mtime);
}

/// Integration test: auto-reload for clean buffer.
///
/// Validates: Requirements 3.3, 5.1, 5.2
#[test]
fn auto_reload_for_clean_buffer() {
    let mut config = ExternalModConfig::default();
    config.policy = ReloadPolicy::Auto;
    let mut detector = ExternalModificationDetector::new(config);

    let doc_id = DocumentId(1);
    let initial_mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_mtime = initial_mtime + Duration::from_secs(10);

    detector.register_document(doc_id, make_uri("/file.rs"), initial_mtime, None, false);
    // Buffer is NOT dirty

    let change = detector.process_modified_event(doc_id, Some(new_mtime));
    assert!(change.is_some());

    let event = change.unwrap();
    let action =
        ReloadPolicyEngine::evaluate(detector.config().policy, event.is_dirty, &event.change_type);
    assert_eq!(action, PolicyAction::AutoReload);

    // After auto-reload, update snapshot
    detector.mark_responded(doc_id, new_mtime);
    let snapshot = detector.mtime_tracker.get_snapshot(doc_id).unwrap();
    assert_eq!(snapshot.mtime, new_mtime);
}

/// Integration test: dirty buffer protection — auto policy falls back to prompt.
///
/// Validates: Requirements 3.4, 5.6
#[test]
fn dirty_buffer_protection_auto_policy_falls_back_to_prompt() {
    let mut config = ExternalModConfig::default();
    config.policy = ReloadPolicy::Auto;
    let mut detector = ExternalModificationDetector::new(config);

    let doc_id = DocumentId(1);
    let initial_mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_mtime = initial_mtime + Duration::from_secs(10);

    detector.register_document(doc_id, make_uri("/file.rs"), initial_mtime, None, false);
    detector.set_dirty(doc_id, true); // Buffer IS dirty

    let change = detector.process_modified_event(doc_id, Some(new_mtime));
    assert!(change.is_some());

    let event = change.unwrap();
    assert!(event.is_dirty);

    // Policy should fall back to prompt for dirty buffers
    let action =
        ReloadPolicyEngine::evaluate(detector.config().policy, event.is_dirty, &event.change_type);
    assert_eq!(action, PolicyAction::ShowPrompt);
}

/// Integration test: file deletion detection and KeepEditing response.
///
/// Validates: Requirements 6.1, 6.3, 6.5
#[test]
fn file_deletion_detection_and_keep_editing() {
    let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
    let doc_id = DocumentId(1);
    let uri = make_uri("/project/deleted.rs");

    detector.register_document(doc_id, uri.clone(), SystemTime::now(), None, false);

    // File is deleted externally
    let change = detector.process_deleted_event(doc_id);
    assert!(change.is_some());

    let event = change.unwrap();
    assert_eq!(event.change_type, ChangeType::FileDeleted);
    assert_eq!(event.document_id, doc_id);

    // User selects "KeepEditing" — mark dirty, cancel watch
    detector.set_dirty(doc_id, true);
    detector.watch_registry.cancel_watch(doc_id);

    // Watch should be removed
    assert!(!detector.watch_registry.has_watch(doc_id));
}

/// Integration test: file rename detection and FollowRename response.
///
/// Validates: Requirements 7.1, 7.3
#[test]
fn file_rename_detection_and_follow_rename() {
    let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
    let doc_id = DocumentId(1);
    let old_uri = make_uri("/project/old_name.rs");
    let new_uri = make_uri("/project/new_name.rs");
    let initial_mtime = SystemTime::now();

    detector.register_document(doc_id, old_uri.clone(), initial_mtime, None, false);

    // File is renamed externally
    let change = detector.process_renamed_event(doc_id, old_uri.clone(), new_uri.clone());
    assert!(change.is_some());

    let event = change.unwrap();
    assert_eq!(
        event.change_type,
        ChangeType::FileRenamed {
            old_uri: old_uri.clone(),
            new_uri: new_uri.clone(),
        }
    );

    // After FollowRename, mark responded
    detector.mark_responded(doc_id, initial_mtime);
    assert!(!detector.has_pending_notification(doc_id));
}

/// Integration test: batch notification coalescing for multi-file changes.
///
/// Validates: Requirements 8.1, 8.2, 8.4, 8.5
#[test]
fn batch_notification_coalescing_multi_file() {
    let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
    let mut coalescer = BatchCoalescer::new(500);

    let base_mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let new_mtime = base_mtime + Duration::from_secs(10);

    // Register 5 documents (simulating git checkout scenario)
    for i in 1..=5 {
        let doc_id = DocumentId(i);
        let uri = make_uri(&format!("/project/file_{i}.rs"));
        detector.register_document(doc_id, uri, base_mtime, None, false);
    }

    // Make doc 3 dirty
    detector.set_dirty(DocumentId(3), true);

    // All 5 files are externally modified (within debounce window)
    for i in 1..=5 {
        let doc_id = DocumentId(i);
        if let Some(event) = detector.process_modified_event(doc_id, Some(new_mtime)) {
            coalescer.add_event(event);
        }
    }

    // Flush the batch
    let batch = coalescer.flush().unwrap();
    assert_eq!(batch.total_count(), 5);
    assert_eq!(batch.modified.len(), 5);

    // Dirty documents are correctly identified
    let dirty = batch.dirty_documents();
    assert_eq!(dirty.len(), 1);
    assert_eq!(dirty[0], DocumentId(3));

    // Clean documents available for bulk reload
    let clean = batch.clean_documents();
    assert_eq!(clean.len(), 4);
}

/// Integration test: focus-gained check detects missed changes.
///
/// Validates: Requirements 9.1, 9.2, 9.4
#[test]
fn focus_gained_check_detects_missed_changes() {
    let config = ExternalModConfig::default();
    let checker = FocusGainedChecker::new(&config);

    let mut tracker = MtimeTracker::new();
    let old_mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
    let changed_mtime = old_mtime + Duration::from_secs(60);

    // Set up 3 documents
    tracker.record_snapshot(DocumentId(1), make_uri("/a.rs"), old_mtime);
    tracker.record_snapshot(DocumentId(2), make_uri("/b.rs"), old_mtime);
    tracker.record_snapshot(DocumentId(3), make_uri("/c.rs"), old_mtime);

    // After focus-gained, doc 1 and 3 have changed, doc 2 is the same
    let documents = vec![
        (DocumentId(1), Some(changed_mtime), None, false),
        (DocumentId(2), Some(old_mtime), None, false),
        (DocumentId(3), Some(changed_mtime), None, true),
    ];

    let changes = checker.check_all(&documents, &tracker);
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].document_id, DocumentId(1));
    assert_eq!(changes[1].document_id, DocumentId(3));
    assert!(changes[1].is_dirty);
}

/// Integration test: polling fallback when VFS watch unsupported.
///
/// Validates: Requirement 1.5
#[test]
fn polling_fallback_when_vfs_watch_unsupported() {
    let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
    let doc_id = DocumentId(1);
    let uri = make_uri("/remote/file.rs");
    let mtime = SystemTime::now();

    // Register with polling (no watch handle)
    detector.register_document(doc_id, uri, mtime, None, true);

    assert!(detector.watch_registry.uses_polling(doc_id));
    assert!(!detector.watch_registry.has_watch(doc_id));
    assert_eq!(detector.watch_registry.polling_count(), 1);
}

/// Integration test: configuration hot-reload changes behaviour immediately.
///
/// Validates: Requirement 10.8
#[test]
fn configuration_hot_reload_changes_behaviour() {
    let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
    let doc_id = DocumentId(1);
    let old_mtime = SystemTime::UNIX_EPOCH;
    let new_mtime = SystemTime::now();

    detector.register_document(doc_id, make_uri("/file.rs"), old_mtime, None, false);

    // Initially policy is Prompt → ShowPrompt
    let change = detector.process_modified_event(doc_id, Some(new_mtime));
    assert!(change.is_some());
    let action = ReloadPolicyEngine::evaluate(
        detector.config().policy,
        false,
        &change.unwrap().change_type,
    );
    assert_eq!(action, PolicyAction::ShowPrompt);

    // Hot-reload: change to Ignore policy
    detector.mark_responded(doc_id, new_mtime);
    let mut new_config = ExternalModConfig::default();
    new_config.policy = ReloadPolicy::Ignore;
    detector.update_config(new_config);

    // Next event with new mtime uses Ignore policy
    let newer_mtime = new_mtime + Duration::from_secs(100);
    let change2 = detector.process_modified_event(doc_id, Some(newer_mtime));
    assert!(change2.is_some());
    let action2 = ReloadPolicyEngine::evaluate(
        detector.config().policy,
        false,
        &change2.unwrap().change_type,
    );
    assert_eq!(action2, PolicyAction::UpdateSnapshotOnly);
}

/// Integration test: document close cancels watch and cleans up mtime snapshot.
///
/// Validates: Requirements 1.3, 2 (cleanup)
#[test]
fn document_close_cancels_watch_and_cleans_up() {
    let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
    let doc_id = DocumentId(1);
    let uri = make_uri("/file.rs");

    detector.register_document(doc_id, uri, SystemTime::now(), None, false);
    assert!(detector.mtime_tracker.get_snapshot(doc_id).is_some());

    // Close document
    detector.unregister_document(doc_id).unwrap();

    // All state should be cleaned up
    assert!(detector.mtime_tracker.get_snapshot(doc_id).is_none());
    assert!(!detector.watch_registry.has_watch(doc_id));
    assert!(!detector.watch_registry.uses_polling(doc_id));
    assert!(!detector.has_pending_notification(doc_id));
}
