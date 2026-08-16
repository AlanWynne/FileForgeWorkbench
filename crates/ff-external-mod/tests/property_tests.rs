//! Property-based tests for the ff-external-mod crate.
//!
//! Uses proptest to verify invariants across randomised inputs.

use std::time::{Duration, SystemTime};

use proptest::prelude::*;

use ff_external_mod::batch_coalescer::BatchCoalescer;
use ff_external_mod::change_event::ChangeType;
use ff_external_mod::config::{
    ExternalModConfig, BATCH_DEBOUNCE_MS_MAX, BATCH_DEBOUNCE_MS_MIN, POLLING_INTERVAL_MS_MAX,
    POLLING_INTERVAL_MS_MIN,
};
use ff_external_mod::detector::ExternalModificationDetector;
use ff_external_mod::focus_check::FocusGainedChecker;
use ff_external_mod::mtime_tracker::MtimeTracker;
use ff_external_mod::reload_policy::{ReloadPolicy, ReloadPolicyEngine};
use ff_external_mod::types::DocumentId;
use ff_external_mod::*;
use ff_vfs::ResourceUri;

// ─── Strategies ─────────────────────────────────────────────────────────────

fn arb_system_time() -> impl Strategy<Value = SystemTime> {
    // Generate times between 2000 and 2030 with nanosecond variation
    (946684800u64..1893456000u64, 0u32..999_999_999u32)
        .prop_map(|(secs, nanos)| SystemTime::UNIX_EPOCH + Duration::new(secs, nanos))
}

fn arb_document_id() -> impl Strategy<Value = DocumentId> {
    (1u64..1000).prop_map(DocumentId)
}

fn arb_reload_policy() -> impl Strategy<Value = ReloadPolicy> {
    prop_oneof![
        Just(ReloadPolicy::Prompt),
        Just(ReloadPolicy::Auto),
        Just(ReloadPolicy::Ignore),
    ]
}

fn arb_change_type() -> impl Strategy<Value = ChangeType> {
    prop_oneof![
        Just(ChangeType::ContentChanged),
        Just(ChangeType::FileDeleted),
        (1u64..100, 1u64..100).prop_map(|(a, b)| ChangeType::FileRenamed {
            old_uri: ResourceUri::new("local", format!("/file_{a}.rs")),
            new_uri: ResourceUri::new("local", format!("/file_{b}.rs")),
        }),
    ]
}

// ─── Property 1: Mtime Comparison Correctness ───────────────────────────────

proptest! {
    /// Feature: ff-external-mod, Property 1: Mtime comparison correctness
    ///
    /// **Validates: Requirements 2.4, 2.5, 2.6**
    ///
    /// For any stored mtime and any current on-disk mtime, the comparison
    /// correctly identifies changes (different mtime → Changed) and
    /// non-changes (equal mtime → Unchanged).
    #[test]
    fn mtime_comparison_correctness(
        stored_mtime in arb_system_time(),
        same_or_different in prop::bool::ANY,
        offset_micros in 1u64..1_000_000_000u64,
    ) {
        let mut tracker = MtimeTracker::new();
        let doc_id = DocumentId(1);
        let uri = ResourceUri::new("local", "/test/file.rs");

        tracker.record_snapshot(doc_id, uri, stored_mtime);

        let current_mtime = if same_or_different {
            stored_mtime // same
        } else {
            stored_mtime + Duration::from_micros(offset_micros) // different
        };

        let result = tracker.check_mtime(doc_id, Some(current_mtime));

        if same_or_different {
            prop_assert_eq!(result, MtimeComparison::Unchanged);
        } else {
            prop_assert_eq!(result, MtimeComparison::Changed {
                old: stored_mtime,
                new: current_mtime,
            });
        }
    }
}

// ─── Property 2: Deduplication ──────────────────────────────────────────────

proptest! {
    /// Feature: ff-external-mod, Property 2: Deduplication — at most one notification per change
    ///
    /// **Validates: Requirement 3.6**
    ///
    /// For any sequence of VFS Modified events for the same document, the
    /// detector emits at most one ExternalChange event per distinct mtime transition.
    #[test]
    fn deduplication_at_most_one_notification_per_change(
        initial_mtime in arb_system_time(),
        event_count in 2usize..50,
        new_mtime_offset in 1u64..1_000_000u64,
    ) {
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let doc_id = DocumentId(1);
        let uri = ResourceUri::new("local", "/test/file.rs");
        let new_mtime = initial_mtime + Duration::from_micros(new_mtime_offset);

        detector.register_document(doc_id, uri, initial_mtime, None, false);

        let mut emissions = 0;
        for _ in 0..event_count {
            if detector.process_modified_event(doc_id, Some(new_mtime)).is_some() {
                emissions += 1;
            }
        }

        // At most one emission for the same mtime transition
        prop_assert!(emissions <= 1,
            "Expected at most 1 emission, got {emissions} for {event_count} events");
    }
}

// ─── Property 3: Reload Policy Evaluation Completeness ──────────────────────

proptest! {
    /// Feature: ff-external-mod, Property 3: Reload policy evaluation completeness
    ///
    /// **Validates: Requirements 3.2, 3.3, 3.4, 3.5**
    ///
    /// For every combination of (ReloadPolicy, dirty_state, change_type), the
    /// policy engine produces a defined PolicyAction with deterministic mapping.
    #[test]
    fn reload_policy_evaluation_completeness(
        policy in arb_reload_policy(),
        is_dirty in prop::bool::ANY,
        change_type in arb_change_type(),
    ) {
        let action = ReloadPolicyEngine::evaluate(policy, is_dirty, &change_type);

        // Verify deterministic invariants
        match policy {
            ReloadPolicy::Ignore => {
                prop_assert_eq!(action, PolicyAction::UpdateSnapshotOnly,
                    "Ignore policy should always return UpdateSnapshotOnly");
            }
            ReloadPolicy::Auto if is_dirty => {
                prop_assert_eq!(action, PolicyAction::ShowPrompt,
                    "Auto + dirty should always fall back to ShowPrompt");
            }
            ReloadPolicy::Auto => {
                prop_assert_eq!(action, PolicyAction::AutoReload,
                    "Auto + clean should always return AutoReload");
            }
            ReloadPolicy::Prompt => {
                prop_assert_eq!(action, PolicyAction::ShowPrompt,
                    "Prompt policy should always return ShowPrompt");
            }
        }
    }
}

// ─── Property 4: Batch Coalescing Bounded-Size ──────────────────────────────

proptest! {
    /// Feature: ff-external-mod, Property 4: Batch coalescing bounded-size
    ///
    /// **Validates: Requirements 8.1, 8.7**
    ///
    /// All events added to the coalescer are included in exactly one batch.
    /// No events are lost or duplicated across batch boundaries.
    #[test]
    fn batch_coalescing_bounded_size(
        debounce_ms in 100u64..5000,
        event_count in 1usize..100,
        flush_after in 1usize..50,
    ) {
        let mut coalescer = BatchCoalescer::new(debounce_ms);
        let mut total_flushed = 0usize;

        for i in 0..event_count {
            let event = ExternalChange::content_changed(
                DocumentId(i as u64 + 1),
                SystemTime::UNIX_EPOCH,
                SystemTime::UNIX_EPOCH + Duration::from_secs(i as u64 + 1),
                false,
            );
            coalescer.add_event(event);

            // Flush periodically (simulating debounce window expiry)
            if (i + 1) % flush_after == 0 {
                if let Some(batch) = coalescer.flush() {
                    total_flushed += batch.total_count();
                }
            }
        }

        // Final flush for remaining events
        if let Some(batch) = coalescer.flush() {
            total_flushed += batch.total_count();
        }

        // Every event appears in exactly one batch
        prop_assert_eq!(total_flushed, event_count);
    }
}

// ─── Property 5: Auto-Reload Dirty-Buffer Safety ────────────────────────────

proptest! {
    /// Feature: ff-external-mod, Property 5: Auto-reload dirty-buffer safety
    ///
    /// **Validates: Requirements 3.4, 5.1, 5.6**
    ///
    /// Auto-reload never silently replaces content in a dirty buffer.
    /// When policy is Auto and buffer is dirty, the action is always ShowPrompt.
    #[test]
    fn auto_reload_dirty_buffer_safety(
        change_type in arb_change_type(),
    ) {
        // Auto + dirty MUST always prompt (never auto-reload)
        let action = ReloadPolicyEngine::evaluate(ReloadPolicy::Auto, true, &change_type);
        prop_assert_eq!(action, PolicyAction::ShowPrompt);
    }
}

// ─── Property 6: Focus-Gained Check Consistency ─────────────────────────────

proptest! {
    /// Feature: ff-external-mod, Property 6: Focus-gained check consistency
    ///
    /// **Validates: Requirements 9.1, 9.2, 9.7**
    ///
    /// After a focus-gained check, documents with changed mtimes produce events;
    /// documents with unchanged mtimes do not. Previously dismissed changes are
    /// not re-prompted.
    #[test]
    fn focus_gained_check_consistency(
        doc_count in 1usize..30,
        changed_ratio in 0.0f64..1.0,
        dismissed_ratio in 0.0f64..0.5,
    ) {
        let config = ExternalModConfig::default();
        let checker = FocusGainedChecker::new(&config);
        let mut tracker = MtimeTracker::new();

        let stored_mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let changed_mtime = stored_mtime + Duration::from_secs(100);

        let mut documents = Vec::new();
        let mut expected_events = 0;

        for i in 0..doc_count {
            let doc_id = DocumentId(i as u64 + 1);
            let uri = ResourceUri::new("local", format!("/file_{i}.rs"));
            tracker.record_snapshot(doc_id, uri, stored_mtime);

            let is_changed = (i as f64) < (doc_count as f64 * changed_ratio);
            let is_dismissed = is_changed && (i as f64) < (doc_count as f64 * dismissed_ratio);

            let current = if is_changed { Some(changed_mtime) } else { Some(stored_mtime) };
            let last_asked = if is_dismissed { Some(changed_mtime) } else { None };

            if is_changed && !is_dismissed {
                expected_events += 1;
            }

            documents.push((doc_id, current, last_asked, false));
        }

        let changes = checker.check_all(&documents, &tracker);
        prop_assert_eq!(changes.len(), expected_events);
    }
}

// ─── Property 7: Configuration Clamping Invariant ───────────────────────────

proptest! {
    /// Feature: ff-external-mod, Property 7: Configuration clamping invariant
    ///
    /// **Validates: Requirements 10.6, 10.7, 10.9**
    ///
    /// For any integer configuration value (including far out-of-range), the parsed
    /// configuration always produces a valid value within the defined range.
    #[test]
    fn configuration_clamping_invariant(
        batch_debounce in 0u64..100_000,
        polling_interval in 0u64..1_000_000,
    ) {
        let mut config = ExternalModConfig {
            batch_debounce_ms: batch_debounce,
            polling_interval_ms: polling_interval,
            ..Default::default()
        };

        config.clamp_all();

        prop_assert!(config.batch_debounce_ms >= BATCH_DEBOUNCE_MS_MIN,
            "batch_debounce_ms {} is below minimum {}", config.batch_debounce_ms, BATCH_DEBOUNCE_MS_MIN);
        prop_assert!(config.batch_debounce_ms <= BATCH_DEBOUNCE_MS_MAX,
            "batch_debounce_ms {} is above maximum {}", config.batch_debounce_ms, BATCH_DEBOUNCE_MS_MAX);
        prop_assert!(config.polling_interval_ms >= POLLING_INTERVAL_MS_MIN,
            "polling_interval_ms {} is below minimum {}", config.polling_interval_ms, POLLING_INTERVAL_MS_MIN);
        prop_assert!(config.polling_interval_ms <= POLLING_INTERVAL_MS_MAX,
            "polling_interval_ms {} is above maximum {}", config.polling_interval_ms, POLLING_INTERVAL_MS_MAX);
    }
}

// ─── Property 8: Watch Lifecycle Cleanup ────────────────────────────────────

proptest! {
    /// Feature: ff-external-mod, Property 8: Watch lifecycle cleanup
    ///
    /// **Validates: Requirements 1.2, 1.3**
    ///
    /// For any sequence of document open/close operations, the number of active
    /// watches equals the number of currently open documents. No watches leak.
    #[test]
    fn watch_lifecycle_cleanup(
        operations in proptest::collection::vec(
            (1u64..20, prop::bool::ANY), // (doc_id, is_open=true / is_close=false)
            20..100
        ),
    ) {
        let mut detector = ExternalModificationDetector::new(ExternalModConfig::default());
        let mut open_docs: std::collections::HashSet<u64> = std::collections::HashSet::new();

        for (raw_id, is_open) in operations {
            let doc_id = DocumentId(raw_id);
            let uri = ResourceUri::new("local", format!("/file_{raw_id}.rs"));

            if is_open && !open_docs.contains(&raw_id) {
                // Open operation
                detector.register_document(doc_id, uri, SystemTime::now(), None, false);
                open_docs.insert(raw_id);
            } else if !is_open && open_docs.contains(&raw_id) {
                // Close operation
                let _ = detector.unregister_document(doc_id);
                open_docs.remove(&raw_id);
            }
        }

        // Invariant: tracked documents == open documents count
        prop_assert_eq!(
            detector.mtime_tracker.count(),
            open_docs.len(),
            "Tracker count {} != open docs count {}",
            detector.mtime_tracker.count(),
            open_docs.len()
        );

        // After closing everything, no state should remain
        for &raw_id in &open_docs {
            let _ = detector.unregister_document(DocumentId(raw_id));
        }
        prop_assert_eq!(detector.mtime_tracker.count(), 0, "Tracker should be empty after all closes");
    }
}
