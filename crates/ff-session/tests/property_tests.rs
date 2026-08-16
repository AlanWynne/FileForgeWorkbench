//! Property-based tests for the ff-session crate.
//!
//! Uses proptest to verify invariants across random inputs.

use proptest::prelude::*;

use ff_session::cli::CliArgs;
use ff_session::config_keys::SessionConfig;
use ff_session::recent_files::RecentFilesList;
use ff_session::session_file::{deserialize_session_state, serialize_session_state};
use ff_session::session_restore::{determine_restore_mode, RestoreMode};
use ff_session::session_state::{
    RecentFileEntry, SelectionRange, SessionState, TabState, WindowGeometryState,
    CURRENT_SCHEMA_VERSION,
};
use ff_session::startup::{execute_startup_sequence, PhaseOutcome, StartupPhase};
use ff_session::window_geometry::{clamp_to_display, is_visible_on, DisplayBounds};

// ─── Strategies ──────────────────────────────────────────────────────────────

fn arb_selection_range() -> impl Strategy<Value = SelectionRange> {
    (1..1000usize, 1..200usize, 1..1000usize, 1..200usize).prop_map(|(sl, sc, el, ec)| {
        SelectionRange {
            start_line: sl,
            start_column: sc,
            end_line: el,
            end_column: ec,
        }
    })
}

fn arb_tab_state() -> impl Strategy<Value = TabState> {
    (
        "[a-z]{4,8}",                                           // tab_id
        proptest::option::of("[a-z/]{5,30}"),                   // uri
        1..5000usize,                                           // viewport_top_line
        0..200usize,                                            // viewport_horizontal_offset
        1..5000usize,                                           // caret_line
        1..200usize,                                            // caret_column
        proptest::collection::vec(arb_selection_range(), 0..3), // selections
        proptest::option::of("[a-z]{3,10}"),                    // language_override
        any::<bool>(),                                          // is_pinned
    )
        .prop_map(
            |(tab_id, uri, vtl, vho, cl, cc, sels, lo, pinned)| TabState {
                tab_id,
                uri,
                viewport_top_line: vtl,
                viewport_horizontal_offset: vho,
                caret_line: cl,
                caret_column: cc,
                selections: sels,
                language_override: lo,
                is_pinned: pinned,
                ..Default::default()
            },
        )
}

fn arb_window_geometry() -> impl Strategy<Value = WindowGeometryState> {
    (
        "[a-z]{3,10}",                       // window_id
        -5000..5000i32,                      // x
        -5000..5000i32,                      // y
        100..4000u32,                        // width
        100..4000u32,                        // height
        any::<bool>(),                       // is_maximised
        any::<bool>(),                       // is_fullscreen
        proptest::option::of("[a-z]{3,10}"), // display_id
    )
        .prop_map(|(wid, x, y, w, h, max, fs, did)| WindowGeometryState {
            window_id: wid,
            x,
            y,
            width: w,
            height: h,
            is_maximised: max,
            is_fullscreen: fs,
            display_id: did,
        })
}

fn arb_recent_file_entry() -> impl Strategy<Value = RecentFileEntry> {
    (
        "[a-z/]{5,30}",                     // uri
        "[a-z.]{3,15}",                     // display_name
        "2024-01-[0-9]{2}T[0-9]{2}:00:00Z", // last_accessed
        proptest::option::of(1..5000usize), // last_viewport_top_line
        any::<bool>(),                      // available
    )
        .prop_map(|(uri, name, ts, vp, avail)| RecentFileEntry {
            uri,
            display_name: name,
            last_accessed: ts,
            last_viewport_top_line: vp,
            available: avail,
        })
}

fn arb_session_state() -> impl Strategy<Value = SessionState> {
    (
        proptest::collection::vec(arb_tab_state(), 0..10),
        proptest::option::of("[a-z]{3,8}"),
        proptest::collection::vec(arb_window_geometry(), 0..3),
        proptest::collection::vec(arb_recent_file_entry(), 0..10),
        proptest::option::of("[a-z]{3,10}"),
    )
        .prop_map(
            |(tabs, active_tab, windows, recent, profile)| SessionState {
                schema_version: CURRENT_SCHEMA_VERSION,
                tabs,
                active_tab_id: active_tab,
                layout: None, // TOML Value is hard to generate arbitrarily
                windows,
                recent_files: recent,
                active_profile: profile,
                last_saved: Some("2024-01-15T10:00:00Z".to_string()),
                show_pom: true,
                global_zoom_offset: 0,
                key_bar_visible: true,
            },
        )
}

fn arb_display_bounds() -> impl Strategy<Value = DisplayBounds> {
    (
        -2000..2000i32, // x
        -2000..2000i32, // y
        800..3840u32,   // width
        600..2160u32,   // height
    )
        .prop_map(|(x, y, w, h)| DisplayBounds {
            x,
            y,
            width: w,
            height: h,
            display_id: Some("test".to_string()),
        })
}

// ─── Property 1: Startup Sequence Phase Ordering Invariant ───────────────────

proptest! {
    /// **Validates: Requirements 1.1, 1.2, 1.3**
    ///
    /// Feature: startup-and-session, Property 1: Startup sequence phase ordering
    ///
    /// For any combination of phase successes and failures, Phases 1–7 always
    /// complete before Phase 8 executes, and Phases 9–10 always execute after
    /// Phase 8.
    #[test]
    fn startup_phase_ordering_invariant(
        failure_set in proptest::collection::vec(2..8u8, 0..5)
    ) {
        let result = execute_startup_sequence(|phase| {
            let phase_num = phase.number();
            if failure_set.contains(&phase_num) {
                PhaseOutcome::Degraded {
                    reason: format!("simulated failure in phase {}", phase_num),
                }
            } else {
                PhaseOutcome::Success
            }
        });

        // All 10 phases should have executed (no fatal failures)
        prop_assert_eq!(result.phases.len(), 10);
        prop_assert!(!result.aborted);

        let render_order = result.phases.iter()
            .find(|p| p.phase == StartupPhase::RenderFirstFrame)
            .unwrap()
            .execution_order;

        // Pre-render phases (1-7) all have lower execution_order than Phase 8
        for phase_result in &result.phases {
            if phase_result.phase.is_pre_render() {
                prop_assert!(
                    phase_result.execution_order < render_order,
                    "Pre-render phase {:?} executed after render",
                    phase_result.phase
                );
            }
        }

        // Post-render phases (9-10) all have higher execution_order than Phase 8
        for phase_result in &result.phases {
            if phase_result.phase.is_post_render() {
                prop_assert!(
                    phase_result.execution_order > render_order,
                    "Post-render phase {:?} executed before render",
                    phase_result.phase
                );
            }
        }
    }
}

// ─── Property 2: Session State TOML Round-Trip Fidelity ──────────────────────

proptest! {
    /// **Validates: Requirements 4.2, 4.6**
    ///
    /// Feature: startup-and-session, Property 2: Session state TOML round-trip fidelity
    ///
    /// For any valid SessionState, serializing to TOML and deserializing back
    /// produces an identical SessionState.
    #[test]
    fn session_state_toml_round_trip(state in arb_session_state()) {
        let serialized = serialize_session_state(&state).unwrap();
        let deserialized = deserialize_session_state(&serialized).unwrap();
        prop_assert_eq!(&deserialized, &state);
    }
}

// ─── Property 3: Recent Files Bounded-List Invariant ─────────────────────────

proptest! {
    /// **Validates: Requirements 4.4, 12.3**
    ///
    /// Feature: startup-and-session, Property 3: Recent Files bounded-list invariant
    ///
    /// For any sequence of add operations with a configured max of N,
    /// the list length never exceeds N.
    #[test]
    fn recent_files_bounded_list_invariant(
        max_count in 1..100u32,
        uris in proptest::collection::vec("[a-z]{3,10}", 10..100)
    ) {
        let mut list = RecentFilesList::new(max_count);

        for uri in &uris {
            list.add(RecentFileEntry {
                uri: uri.clone(),
                display_name: uri.clone(),
                last_accessed: "2024-01-01T00:00:00Z".to_string(),
                last_viewport_top_line: None,
                available: true,
            });

            prop_assert!(
                list.len() <= max_count as usize,
                "List length {} exceeded max_count {}",
                list.len(),
                max_count
            );
        }
    }
}

// ─── Property 4: Recent Files Deduplication ──────────────────────────────────

proptest! {
    /// **Validates: Requirement 4.4**
    ///
    /// Feature: startup-and-session, Property 4: Recent Files deduplication
    ///
    /// The Recent Files list never contains duplicate URIs. Adding a URI
    /// that already exists moves it to the top without creating a second entry.
    #[test]
    fn recent_files_deduplication_property(
        pool in proptest::collection::vec("[a-z]{3,8}", 3..15),
        indices in proptest::collection::vec(0..15usize, 20..100)
    ) {
        let mut list = RecentFilesList::new(500);

        for idx in &indices {
            let uri_idx = idx % pool.len();
            let uri = &pool[uri_idx];

            list.add(RecentFileEntry {
                uri: uri.clone(),
                display_name: uri.clone(),
                last_accessed: "2024-01-01T00:00:00Z".to_string(),
                last_viewport_top_line: None,
                available: true,
            });

            // Check no duplicates
            let entries = list.list();
            let uris: Vec<&str> = entries.iter().map(|e| e.uri.as_str()).collect();
            let mut deduped = uris.clone();
            deduped.sort();
            deduped.dedup();
            prop_assert_eq!(
                uris.len(),
                deduped.len(),
                "Duplicate URIs found in list"
            );

            // Most recently added is at position 0
            prop_assert_eq!(
                entries[0].uri.as_str(),
                uri.as_str(),
                "Last added URI should be at position 0"
            );
        }
    }
}

// ─── Property 5: Window Geometry Off-Screen Clamping ─────────────────────────

proptest! {
    /// **Validates: Requirements 8.4, 8.5**
    ///
    /// Feature: startup-and-session, Property 5: Window geometry off-screen clamping
    ///
    /// For any stored geometry and display configuration, the clamped geometry
    /// always places the window fully on-screen.
    #[test]
    fn window_geometry_clamping_property(
        geom in arb_window_geometry(),
        display in arb_display_bounds()
    ) {
        let clamped = clamp_to_display(&geom, &display);

        // Window must be fully visible after clamping
        prop_assert!(
            is_visible_on(&clamped, &display),
            "Clamped geometry ({}, {}, {}x{}) not visible on display ({}, {}, {}x{})",
            clamped.x, clamped.y, clamped.width, clamped.height,
            display.x, display.y, display.width, display.height
        );

        // Width and height must be at least MIN_WINDOW_SIZE (100)
        prop_assert!(clamped.width >= 100);
        prop_assert!(clamped.height >= 100);
    }
}

// ─── Property 6: CLI Argument Resolution ─────────────────────────────────────

proptest! {
    /// **Validates: Requirements 6.1, 6.2, 6.3**
    ///
    /// Feature: startup-and-session, Property 6: CLI argument resolution
    ///
    /// Relative paths are resolved against the working directory.
    /// VFS URIs pass through unchanged. No duplicates after resolution.
    #[test]
    fn cli_argument_resolution_property(
        relative_paths in proptest::collection::vec("[a-z]{3,8}/[a-z]{3,8}", 1..5),
        vfs_uris in proptest::collection::vec("vfs://[a-z]{3,8}/[a-z]{3,8}", 0..3),
    ) {
        let mut source_args: Vec<String> = relative_paths.iter().map(|s| s.to_string()).collect();
        source_args.extend(vfs_uris.iter().map(|s| s.to_string()));

        let args = CliArgs {
            source_args: source_args.clone(),
            ..Default::default()
        };

        #[cfg(windows)]
        let working_dir = std::path::Path::new("C:\\work");
        #[cfg(not(windows))]
        let working_dir = std::path::Path::new("/work");

        let resolved = args.resolved_source_args(working_dir);

        // VFS URIs should be unchanged
        for vfs in &vfs_uris {
            prop_assert!(
                resolved.contains(vfs),
                "VFS URI {} should be unchanged in resolved args",
                vfs
            );
        }

        // Relative paths should be resolved (contain working dir)
        for rel_path in &relative_paths {
            let found = resolved.iter().any(|r| r.contains(rel_path));
            prop_assert!(
                found,
                "Relative path {} should appear in resolved args",
                rel_path
            );
        }

        // Same number of args in and out
        prop_assert_eq!(resolved.len(), source_args.len());
    }
}

// ─── Property 7: Graceful Degradation Survivability ──────────────────────────

proptest! {
    /// **Validates: Requirement 11.1**
    ///
    /// Feature: startup-and-session, Property 7: Graceful degradation survivability
    ///
    /// For any combination of non-fatal subsystem failures, the startup always
    /// completes Phase 8 (renders first frame).
    #[test]
    fn graceful_degradation_survivability(
        failure_phases in proptest::collection::vec(
            prop_oneof![
                Just(2u8),
                Just(3u8),
                Just(4u8),
                Just(5u8),
                Just(6u8),
                Just(7u8),
            ],
            0..6
        )
    ) {
        let result = execute_startup_sequence(|phase| {
            if failure_phases.contains(&phase.number()) {
                PhaseOutcome::Degraded {
                    reason: "simulated failure".to_string(),
                }
            } else {
                PhaseOutcome::Success
            }
        });

        // Phase 8 should always be reached
        prop_assert!(!result.aborted);
        let phase8 = result.phases.iter().find(|p| p.phase == StartupPhase::RenderFirstFrame);
        prop_assert!(phase8.is_some(), "Phase 8 (RenderFirstFrame) should always execute");
    }
}

// ─── Property 8: Session Restore Decision Matrix ─────────────────────────────

proptest! {
    /// **Validates: Requirements 5.1, 5.3, 5.6, 5.7**
    ///
    /// Feature: startup-and-session, Property 8: Session restore decision matrix
    ///
    /// The session restore decision logic correctly selects the restore mode
    /// based on the combination of settings.
    #[test]
    fn session_restore_decision_matrix(
        restore_on in any::<bool>(),
        restore_tabs in any::<bool>(),
        has_cli_args in any::<bool>(),
        has_startup_file in any::<bool>(),
        has_no_session_flag in any::<bool>(),
        session_has_content in any::<bool>(),
    ) {
        let config = SessionConfig {
            restore_on_startup: restore_on,
            restore_tabs_on_startup: restore_tabs,
            startup_file: if has_startup_file { Some("start.rs".to_string()) } else { None },
            ..Default::default()
        };

        let cli_args = CliArgs {
            source_args: if has_cli_args { vec!["file.txt".to_string()] } else { vec![] },
            no_session_restore: has_no_session_flag,
            ..Default::default()
        };

        let session_state = if session_has_content {
            SessionState {
                tabs: vec![TabState { tab_id: "t1".to_string(), ..Default::default() }],
                ..Default::default()
            }
        } else {
            SessionState::empty()
        };

        let mode = determine_restore_mode(&config, &cli_args, &session_state);

        // Verify decision matrix rules
        if has_cli_args {
            prop_assert_eq!(mode, RestoreMode::CliArgs, "CLI args should always win");
        } else if has_no_session_flag {
            prop_assert_eq!(mode, RestoreMode::Empty, "--no-session-restore should force empty");
        } else if has_startup_file {
            prop_assert_eq!(mode, RestoreMode::StartupFile, "startup_file should override tabs");
        } else if !restore_on {
            prop_assert_eq!(mode, RestoreMode::Empty, "restore_on=false should give empty");
        } else if !session_has_content {
            prop_assert_eq!(mode, RestoreMode::Empty, "no content should give empty");
        } else if restore_tabs {
            prop_assert_eq!(mode, RestoreMode::FullRestore, "should be full restore");
        } else {
            prop_assert_eq!(mode, RestoreMode::LayoutOnly, "should be layout only");
        }
    }
}

// ─── Property 9: Exit Sequence Completeness ──────────────────────────────────

proptest! {
    /// **Validates: Requirements 9.1, 9.2, 9.6**
    ///
    /// Feature: startup-and-session, Property 9: Exit sequence unsaved-changes completeness
    ///
    /// When SaveAll is chosen, all dirty documents are in the save list.
    /// When DiscardAll is chosen, all dirty documents are in the discard list.
    /// When Cancel is chosen, no documents are in either list.
    #[test]
    fn exit_sequence_completeness(
        num_dirty in 1..20usize,
        action_idx in 0..4u8,
    ) {
        use ff_session::exit_sequence::{DirtyDocument, ExitAction, ExitDecision, process_exit_action};

        let dirty: Vec<DirtyDocument> = (0..num_dirty)
            .map(|i| DirtyDocument {
                display_name: format!("file{i}.txt"),
                uri: Some(format!("file://file{i}.txt")),
                tab_id: format!("tab-{i}"),
            })
            .collect();

        let action = match action_idx {
            0 => ExitAction::SaveAll,
            1 => ExitAction::DiscardAll,
            2 => ExitAction::Cancel,
            _ => ExitAction::ReviewEach,
        };

        let result = process_exit_action(action, &dirty);

        match action {
            ExitAction::SaveAll => {
                prop_assert_eq!(result.decision, ExitDecision::Proceed);
                prop_assert_eq!(result.documents_to_save.len(), num_dirty);
                prop_assert!(result.documents_to_discard.is_empty());
            }
            ExitAction::DiscardAll => {
                prop_assert_eq!(result.decision, ExitDecision::Proceed);
                prop_assert!(result.documents_to_save.is_empty());
                prop_assert_eq!(result.documents_to_discard.len(), num_dirty);
            }
            ExitAction::Cancel => {
                prop_assert_eq!(result.decision, ExitDecision::Cancelled);
                prop_assert!(result.documents_to_save.is_empty());
                prop_assert!(result.documents_to_discard.is_empty());
            }
            ExitAction::ReviewEach => {
                prop_assert_eq!(result.decision, ExitDecision::Proceed);
            }
        }
    }
}

// ─── Property 10: Schema Migration Forward-Compatibility ─────────────────────

proptest! {
    /// **Validates: Requirement 4.6**
    ///
    /// Feature: startup-and-session, Property 10: Schema migration forward-compatibility
    ///
    /// Loading a session with schema_version <= current always produces a valid
    /// state at the current version. Unknown keys do not cause parse failure.
    #[test]
    fn schema_migration_forward_compatibility(
        version in 0..=CURRENT_SCHEMA_VERSION,
    ) {
        let content = format!(
            r#"
schema_version = {}
active_profile = "test"
last_saved = "2024-01-01T00:00:00Z"
"#,
            version
        );

        let result = deserialize_session_state(&content);
        prop_assert!(result.is_ok(), "Deserialization failed for version {}: {:?}", version, result.err());

        let state = result.unwrap();
        prop_assert_eq!(state.schema_version, CURRENT_SCHEMA_VERSION);
    }
}
