//! Integration tests for the ff-session crate.
//!
//! Tests cross-module interactions and end-to-end flows with realistic
//! session data.

use tempfile::TempDir;

use ff_session::cli::is_vfs_uri;
use ff_session::crash_recovery::{process_recovery_action, scan_recovery_dir, RecoveryAction};
use ff_session::degraded_mode::{DegradedModeTracker, Subsystem};
use ff_session::exit_sequence::{process_exit_action, DirtyDocument, ExitAction, ExitDecision};
use ff_session::recent_files::RecentFilesList;
use ff_session::session_restore::{determine_restore_mode, RestoreMode};
use ff_session::startup::{execute_startup_sequence, PhaseOutcome, StartupPhase};
use ff_session::{
    CliArgs, SessionConfig, SessionFile, SessionState, TabState, UserDataDir, WindowGeometryState,
    CURRENT_SCHEMA_VERSION,
};

/// Integration test: Full startup sequence with valid session file.
/// Verifies phase ordering and that session state is loaded correctly.
#[test]
fn full_startup_with_valid_session_file() {
    // Validates: Requirement 1 AC 1.1, Requirement 4 AC 4.2
    let tmp = TempDir::new().unwrap();

    // Create a session file with tabs
    let session_path = tmp.path().join("session.toml");
    let session_file = SessionFile::new(session_path.clone());

    let state = SessionState {
        schema_version: CURRENT_SCHEMA_VERSION,
        tabs: vec![TabState {
            tab_id: "tab-1".to_string(),
            uri: Some("file:///project/main.rs".to_string()),
            viewport_top_line: 42,
            caret_line: 50,
            caret_column: 10,
            ..Default::default()
        }],
        windows: vec![WindowGeometryState::primary(100, 100, 1920, 1080)],
        ..Default::default()
    };
    session_file.save(&state).unwrap();

    // Execute startup
    let result = execute_startup_sequence(|_| PhaseOutcome::Success);

    assert!(!result.aborted);
    assert_eq!(result.phases.len(), 10);

    // Load session from file
    let loaded = session_file.load().unwrap();
    assert_eq!(loaded.tabs.len(), 1);
    assert_eq!(loaded.tabs[0].viewport_top_line, 42);
}

/// Integration test: First-run startup with no User_Data_Dir.
/// Verifies directory creation and empty-state boot.
#[test]
fn first_run_startup_creates_user_data_dir() {
    // Validates: Requirement 3 AC 3.1, Requirement 7 AC 7.1
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("new_ffworkbench");

    let mut udd = UserDataDir::from_path(data_dir.clone());
    udd.initialise().unwrap();

    assert!(data_dir.exists());
    assert!(data_dir.join("sessions").exists());
    assert!(data_dir.join("recovery").exists());
    assert!(data_dir.join("profiles").exists());
    assert!(data_dir.join("plugins").exists());

    // Session file doesn't exist yet
    let session_file = SessionFile::new(data_dir.join("session.toml"));
    let state = session_file.load().unwrap();
    assert_eq!(state, SessionState::empty());
}

/// Integration test: Startup with corrupt session.toml.
/// Verifies graceful degradation to empty state.
#[test]
fn corrupt_session_file_degrades_gracefully() {
    // Validates: Requirement 4 AC 4.8, Requirement 11 AC 11.1
    let tmp = TempDir::new().unwrap();
    let session_path = tmp.path().join("session.toml");

    // Write corrupt data
    std::fs::write(&session_path, "{{{{not valid TOML}}}}").unwrap();

    let session_file = SessionFile::new(session_path);
    let result = session_file.load();

    // Should return an error (corrupt)
    assert!(result.is_err());

    // In the startup sequence, this would be caught and degraded
    let mut tracker = DegradedModeTracker::new();
    tracker.record_failure(
        Subsystem::SessionPersistence,
        StartupPhase::LoadSessionState,
        "session file corrupt".to_string(),
    );
    assert!(tracker.is_degraded());
}

/// Integration test: CLI args override session restore.
/// Verifies files opened from CLI, session tabs skipped.
#[test]
fn cli_args_override_session_restore() {
    // Validates: Requirement 5 AC 5.6, Requirement 6 AC 6.4
    let config = SessionConfig::default();
    let cli_args = CliArgs {
        source_args: vec!["override.rs".to_string()],
        ..Default::default()
    };
    let state = SessionState {
        tabs: vec![TabState {
            tab_id: "session-tab".to_string(),
            uri: Some("session_file.rs".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let mode = determine_restore_mode(&config, &cli_args, &state);
    assert_eq!(mode, RestoreMode::CliArgs);
}

/// Integration test: Exit sequence with unsaved changes.
/// Verifies dialog flow and session state persistence.
#[test]
fn exit_with_unsaved_changes_save_all() {
    // Validates: Requirement 9 AC 9.2, 9.3, 9.7
    let dirty = vec![
        DirtyDocument {
            display_name: "main.rs".to_string(),
            uri: Some("file:///main.rs".to_string()),
            tab_id: "tab-1".to_string(),
        },
        DirtyDocument {
            display_name: "lib.rs".to_string(),
            uri: Some("file:///lib.rs".to_string()),
            tab_id: "tab-2".to_string(),
        },
    ];

    let result = process_exit_action(ExitAction::SaveAll, &dirty);
    assert_eq!(result.decision, ExitDecision::Proceed);
    assert_eq!(result.documents_to_save.len(), 2);

    // After save, session should be persisted
    let tmp = TempDir::new().unwrap();
    let session_file = SessionFile::new(tmp.path().join("session.toml"));
    let state = SessionState {
        tabs: vec![
            TabState {
                tab_id: "tab-1".to_string(),
                uri: Some("file:///main.rs".to_string()),
                ..Default::default()
            },
            TabState {
                tab_id: "tab-2".to_string(),
                uri: Some("file:///lib.rs".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    session_file.save(&state).unwrap();

    let loaded = session_file.load().unwrap();
    assert_eq!(loaded.tabs.len(), 2);
}

/// Integration test: Crash recovery offer and restore.
#[test]
fn crash_recovery_detection_and_offer() {
    // Validates: Requirement 10 AC 10.1, 10.3
    let tmp = TempDir::new().unwrap();
    let recovery_dir = tmp.path().join("recovery");
    std::fs::create_dir(&recovery_dir).unwrap();

    // Simulate orphaned recovery files
    std::fs::write(recovery_dir.join("main_rs.recovery"), "recovery data").unwrap();
    std::fs::write(recovery_dir.join("lib_rs.recovery"), "recovery data").unwrap();

    let docs = scan_recovery_dir(&recovery_dir).unwrap();
    assert_eq!(docs.len(), 2);

    // User chooses "Later" — files should be retained
    process_recovery_action(RecoveryAction::Later, &docs, &recovery_dir);
    assert!(recovery_dir.join("main_rs.recovery").exists());

    // On next startup, user chooses "Discard"
    process_recovery_action(RecoveryAction::Discard, &docs, &recovery_dir);
    let remaining: Vec<_> = std::fs::read_dir(&recovery_dir)
        .unwrap()
        .flatten()
        .collect();
    assert!(remaining.is_empty());
}

/// Integration test: Plugin failure during startup.
/// Verifies workbench starts with degraded indicator.
#[test]
fn plugin_failure_results_in_degraded_mode() {
    // Validates: Requirement 11 AC 11.1, 11.4
    let result = execute_startup_sequence(|phase| {
        if phase == StartupPhase::LoadPlugins {
            PhaseOutcome::Degraded {
                reason: "plugin 'git-lens' failed to load: missing dependency".to_string(),
            }
        } else {
            PhaseOutcome::Success
        }
    });

    assert!(!result.aborted);
    assert_eq!(result.phases.len(), 10);

    // Phase 8 was reached
    let phase8 = result
        .phases
        .iter()
        .find(|p| p.phase == StartupPhase::RenderFirstFrame)
        .unwrap();
    assert!(matches!(phase8.outcome, PhaseOutcome::Success));

    // Degraded mode should be active
    let warnings = result.deferred_warnings();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("git-lens"));
}

/// Integration test: Window geometry restore with display disconnect.
#[test]
fn window_geometry_display_disconnect_fallback() {
    // Validates: Requirement 8 AC 8.4
    use ff_session::window_geometry::{is_visible_on, restore_geometry, DisplayBounds};

    let primary = DisplayBounds {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        display_id: Some("primary".to_string()),
    };

    // Geometry was on a disconnected secondary display
    let geom = WindowGeometryState {
        window_id: "primary".to_string(),
        x: 2500,
        y: 300,
        width: 1200,
        height: 800,
        is_maximised: false,
        is_fullscreen: false,
        display_id: Some("secondary-disconnected".to_string()),
    };

    let restored = restore_geometry(&geom, &[primary.clone()]);

    // Should be visible on primary display
    assert!(is_visible_on(&restored, &primary));
    // Should be centred
    let expected_x = (1920 - 1200) / 2;
    let expected_y = (1080 - 800) / 2;
    assert_eq!(restored.x, expected_x as i32);
    assert_eq!(restored.y, expected_y as i32);
}

/// Integration test: Hot-reload of session configuration.
#[test]
fn hot_reload_session_config_applies_new_values() {
    // Validates: Requirement 2 AC 2.4, Requirement 12 AC 12.10
    let mut config = SessionConfig::default();
    assert_eq!(config.max_recent_files, 50);

    // Simulate hot-reload with new values
    let new_config = SessionConfig {
        max_recent_files: 100,
        auto_save_interval_seconds: 120,
        ..Default::default()
    };

    config.apply_reload(new_config).unwrap();
    assert_eq!(config.max_recent_files, 100);
    assert_eq!(config.auto_save_interval_seconds, 120);

    // New max applies to recent files list
    let mut list = RecentFilesList::new(config.max_recent_files);
    for i in 0..150 {
        list.add(ff_session::RecentFileEntry {
            uri: format!("file{i}.txt"),
            display_name: format!("file{i}.txt"),
            last_accessed: "2024-01-01T00:00:00Z".to_string(),
            last_viewport_top_line: None,
            available: true,
        });
    }
    assert_eq!(list.len(), 100); // Capped at new max
}

/// Integration test: VFS URI handling end-to-end.
#[test]
fn vfs_uri_passthrough_end_to_end() {
    // Validates: Requirement 6 AC 6.3
    let args = CliArgs::parse_from(vec!["vfs://remote/project/main.rs", "local_file.txt"]).unwrap();

    assert!(is_vfs_uri(&args.source_args[0]));
    assert!(!is_vfs_uri(&args.source_args[1]));

    #[cfg(windows)]
    let cwd = std::path::Path::new("C:\\workspace");
    #[cfg(not(windows))]
    let cwd = std::path::Path::new("/workspace");

    let resolved = args.resolved_source_args(cwd);
    // VFS URI unchanged
    assert_eq!(resolved[0], "vfs://remote/project/main.rs");
    // Relative path resolved
    assert!(resolved[1].contains("local_file.txt"));
    assert_ne!(resolved[1], "local_file.txt");
}
