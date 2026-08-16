//! Integration tests for per-project configuration management.
//!
//! Covers Requirement 5 (AC 5.1–5.7): project config detection, loading,
//! priority overrides, hot-reload, unload with callback invocation, and
//! graceful failure handling.
//!
//! These tests exercise the full project lifecycle through the ReloadManager
//! API, verifying end-to-end behaviour that spans multiple internal modules.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ff_config::callback::CallbackRegistry;
use ff_config::layer::ConfigLayer;
use ff_config::loader::{load_toml_file, LayerData};
use ff_config::reload::ReloadManager;
use ff_config::schema::SchemaRegistry;
use ff_config::value::ConfigValue;
use ff_config::watcher::ConfigWatcher;

use tempfile::TempDir;

/// Helper: create a project directory with `.ffworkbench/config.toml`.
fn setup_project(dir: &std::path::Path, toml_content: &str) -> std::path::PathBuf {
    let ffworkbench_dir = dir.join(".ffworkbench");
    std::fs::create_dir_all(&ffworkbench_dir).unwrap();
    let config_path = ffworkbench_dir.join("config.toml");
    std::fs::write(&config_path, toml_content).unwrap();
    config_path
}

/// Helper: create a LayerData from a temp file.
fn make_layer(dir: &TempDir, name: &str, layer: ConfigLayer, content: &str) -> LayerData {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    let values = load_toml_file(&path).unwrap();
    LayerData {
        layer,
        source_path: path,
        values,
    }
}

// ============================================================================
// AC 5.1: Recognizes `.ffworkbench/config.toml` as project config source
// ============================================================================

// Validates: Requirement 5.1 — project config detected at .ffworkbench/config.toml
#[test]
fn project_config_at_well_known_path_is_detected_and_loaded() {
    let dir = TempDir::new().unwrap();
    let project_root = dir.path().join("my-project");
    setup_project(&project_root, "[editor]\ntab_size = 2\n");

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(Vec::new(), schema);

    let event = manager.load_project(&project_root).unwrap();

    assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(2))
    );
    assert!(manager.has_project_layer());
}

// ============================================================================
// AC 5.2: Automatically detects and loads project config when project opened
// ============================================================================

// Validates: Requirement 5.2 — open_project automatically loads config
#[test]
fn open_project_automatically_detects_and_loads_config() {
    let dir = TempDir::new().unwrap();
    let project_root = dir.path().join("auto-detect");
    setup_project(&project_root, "[logging]\nlevel = \"debug\"\n");

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(Vec::new(), schema);

    // open_project is the "automatic" entry point
    let event = manager.open_project(&project_root);

    assert!(event.changed_keys.contains(&"logging.level".to_string()));
    assert_eq!(
        manager.store().get_value("logging.level"),
        Some(&ConfigValue::String("debug".to_string()))
    );
}

// ============================================================================
// AC 5.3: Project layer overrides User and Profile but NOT Workspace
// ============================================================================

// Validates: Requirement 5.3 — Project (priority 4) overrides User (priority 2)
#[test]
fn project_layer_overrides_user_layer() {
    let dir = TempDir::new().unwrap();
    let user_layer = make_layer(
        &dir,
        "user.toml",
        ConfigLayer::User,
        "[editor]\ntab_size = 4\n",
    );

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(vec![user_layer], schema);
    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(4))
    );

    let project_root = dir.path().join("proj");
    setup_project(&project_root, "[editor]\ntab_size = 2\n");
    manager.load_project(&project_root).unwrap();

    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(2)),
        "Project (priority 4) must override User (priority 2)"
    );
}

// Validates: Requirement 5.3 — Project (priority 4) overrides Profile (priority 3)
#[test]
fn project_layer_overrides_profile_layer() {
    let dir = TempDir::new().unwrap();
    let profile_layer = make_layer(
        &dir,
        "profile.toml",
        ConfigLayer::Profile,
        "[editor]\ntab_size = 3\n",
    );

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(vec![profile_layer], schema);
    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(3))
    );

    let project_root = dir.path().join("proj");
    setup_project(&project_root, "[editor]\ntab_size = 2\n");
    manager.load_project(&project_root).unwrap();

    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(2)),
        "Project (priority 4) must override Profile (priority 3)"
    );
}

// Validates: Requirement 5.3 — Project (priority 4) does NOT override Workspace (priority 5)
#[test]
fn project_layer_does_not_override_workspace_layer() {
    let dir = TempDir::new().unwrap();
    let workspace_layer = make_layer(
        &dir,
        "workspace.toml",
        ConfigLayer::Workspace,
        "[editor]\ntab_size = 1\n",
    );

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(vec![workspace_layer], schema);
    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(1))
    );

    let project_root = dir.path().join("proj");
    setup_project(&project_root, "[editor]\ntab_size = 2\n");
    manager.load_project(&project_root).unwrap();

    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(1)),
        "Workspace (priority 5) must NOT be overridden by Project (priority 4)"
    );
}

// Validates: Requirement 5.3 — Full override chain: User < Profile < Project < Workspace
#[test]
fn full_override_chain_user_profile_project_workspace() {
    let dir = TempDir::new().unwrap();

    let user_layer = make_layer(
        &dir,
        "user.toml",
        ConfigLayer::User,
        "[editor]\ntab_size = 8\nword_wrap = true\n",
    );
    let profile_layer = make_layer(
        &dir,
        "profile.toml",
        ConfigLayer::Profile,
        "[editor]\ntab_size = 6\n",
    );
    let workspace_layer = make_layer(
        &dir,
        "workspace.toml",
        ConfigLayer::Workspace,
        "[editor]\nindent_style = \"tab\"\n",
    );

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(vec![user_layer, profile_layer, workspace_layer], schema);

    // Before project: Profile overrides User for tab_size
    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(6))
    );

    let project_root = dir.path().join("proj");
    setup_project(
        &project_root,
        "[editor]\ntab_size = 2\nindent_style = \"space\"\n",
    );
    manager.load_project(&project_root).unwrap();

    // Project overrides Profile for tab_size
    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(2)),
        "Project should override Profile for tab_size"
    );
    // Workspace overrides Project for indent_style
    assert_eq!(
        manager.store().get_value("editor.indent_style"),
        Some(&ConfigValue::String("tab".to_string())),
        "Workspace should override Project for indent_style"
    );
    // User layer key not overridden by anyone
    assert_eq!(
        manager.store().get_value("editor.word_wrap"),
        Some(&ConfigValue::Boolean(true)),
        "User layer key should still be visible"
    );
}

// ============================================================================
// AC 5.4: Project config is version-control suitable (file at well-known path)
// ============================================================================

// Validates: Requirement 5.4 — config is a plain file at a well-known relative path
#[test]
fn project_config_is_at_deterministic_path_suitable_for_version_control() {
    let project_root = std::path::Path::new("/some/project");
    let config_path = ff_config::paths::project_config_path(project_root);

    // The path must be deterministic and relative to project root
    assert_eq!(
        config_path,
        std::path::PathBuf::from("/some/project/.ffworkbench/config.toml")
    );
    // Being a well-known relative path makes it suitable for .git tracking
}

// ============================================================================
// AC 5.5: Hot-reload for project config file changes
// ============================================================================

// Validates: Requirement 5.5 — modified project config is re-read and applied
#[test]
fn hot_reload_modified_project_config_is_reread_and_applied() {
    let dir = TempDir::new().unwrap();
    let project_root = dir.path().join("proj");
    let config_path = setup_project(&project_root, "[editor]\ntab_size = 2\n");

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(Vec::new(), schema);
    manager.load_project(&project_root).unwrap();

    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(2))
    );

    // Modify the file on disk (simulating external edit)
    std::fs::write(&config_path, "[editor]\ntab_size = 4\n").unwrap();

    // Trigger reload (this is what the watcher loop would call)
    let event = manager
        .reload_file(&config_path, ConfigLayer::Project)
        .unwrap();

    assert!(event.is_some());
    let event = event.unwrap();
    assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(4))
    );
}

// Validates: Requirement 5.5 — hot-reload of project config invokes callbacks
#[test]
fn hot_reload_project_config_invokes_callbacks_for_changed_keys() {
    let dir = TempDir::new().unwrap();
    let project_root = dir.path().join("proj");
    let config_path = setup_project(&project_root, "[editor]\ntab_size = 2\n");

    let schema = SchemaRegistry::new();
    let callbacks = Arc::new(CallbackRegistry::new());
    let mut manager = ReloadManager::with_callbacks(Vec::new(), schema, Arc::clone(&callbacks));

    let invocation_count = Arc::new(AtomicU32::new(0));
    let count_clone = Arc::clone(&invocation_count);
    let _handle = callbacks.on_reload(
        &["editor.tab_size"],
        Box::new(move |_event| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        }),
    );

    manager.load_project(&project_root).unwrap();
    // Reset count after load_project's callback
    invocation_count.store(0, Ordering::SeqCst);

    // Modify and reload
    std::fs::write(&config_path, "[editor]\ntab_size = 8\n").unwrap();
    manager
        .reload_file(&config_path, ConfigLayer::Project)
        .unwrap();

    assert_eq!(
        invocation_count.load(Ordering::SeqCst),
        1,
        "Callback should fire once on hot-reload"
    );
    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(8))
    );
}

// Validates: Requirement 5.5 — watcher detects project config change end-to-end
#[test]
fn watcher_detects_project_config_modification_end_to_end() {
    let dir = TempDir::new().unwrap();
    let project_root = dir.path().join("proj");
    let config_path = setup_project(&project_root, "[editor]\ntab_size = 2\n");

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(Vec::new(), schema);

    let watcher = ConfigWatcher::with_debounce(Duration::from_millis(50)).unwrap();
    manager.set_watcher(watcher);

    manager.load_project(&project_root).unwrap();

    // Allow watcher to fully initialize
    std::thread::sleep(Duration::from_millis(100));

    // Modify the file
    std::fs::write(&config_path, "[editor]\ntab_size = 6\n").unwrap();

    // Poll until change detected (max 2 seconds per Requirement 3.2)
    let start = std::time::Instant::now();
    let mut detected = false;
    while start.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(70));
        let changes = manager.watcher_mut().unwrap().poll_changes();
        if changes.iter().any(|c| c.path == config_path) {
            detected = true;
            break;
        }
    }
    assert!(detected, "Watcher must detect file change within 2 seconds");

    // Apply the reload
    let event = manager
        .reload_file(&config_path, ConfigLayer::Project)
        .unwrap()
        .unwrap();

    assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(6))
    );
}

// ============================================================================
// AC 5.6: Unload reverts values and invokes callbacks
// ============================================================================

// Validates: Requirement 5.6 — unload reverts to lower-priority layer values
#[test]
fn unload_project_reverts_effective_values_to_lower_layers() {
    let dir = TempDir::new().unwrap();
    let user_layer = make_layer(
        &dir,
        "user.toml",
        ConfigLayer::User,
        "[editor]\ntab_size = 4\nword_wrap = true\n",
    );

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(vec![user_layer], schema);

    let project_root = dir.path().join("proj");
    setup_project(&project_root, "[editor]\ntab_size = 2\n");
    manager.load_project(&project_root).unwrap();

    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(2))
    );

    // Unload
    let event = manager.unload_project();

    assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(4)),
        "Should revert to User layer value after unload"
    );
    // Keys not in project layer are unaffected
    assert_eq!(
        manager.store().get_value("editor.word_wrap"),
        Some(&ConfigValue::Boolean(true))
    );
}

// Validates: Requirement 5.6 — keys only in project layer disappear after unload
#[test]
fn unload_project_removes_keys_only_defined_in_project_layer() {
    let dir = TempDir::new().unwrap();

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(Vec::new(), schema);

    let project_root = dir.path().join("proj");
    setup_project(&project_root, "[custom]\nproject_only_key = \"value\"\n");
    manager.load_project(&project_root).unwrap();

    assert_eq!(
        manager.store().get_value("custom.project_only_key"),
        Some(&ConfigValue::String("value".to_string()))
    );

    let event = manager.unload_project();

    assert!(event
        .changed_keys
        .contains(&"custom.project_only_key".to_string()));
    assert_eq!(
        manager.store().get_value("custom.project_only_key"),
        None,
        "Key only defined in project layer must disappear after unload"
    );
}

// Validates: Requirement 5.6 — unload invokes callbacks for changed keys
#[test]
fn unload_project_invokes_callbacks_for_reverted_keys() {
    let dir = TempDir::new().unwrap();
    let user_layer = make_layer(
        &dir,
        "user.toml",
        ConfigLayer::User,
        "[editor]\ntab_size = 4\n",
    );

    let schema = SchemaRegistry::new();
    let callbacks = Arc::new(CallbackRegistry::new());
    let mut manager =
        ReloadManager::with_callbacks(vec![user_layer], schema, Arc::clone(&callbacks));

    let callback_fired = Arc::new(AtomicU32::new(0));
    let fired_clone = Arc::clone(&callback_fired);
    let _handle = callbacks.on_reload(
        &["editor.tab_size"],
        Box::new(move |_event| {
            fired_clone.fetch_add(1, Ordering::SeqCst);
        }),
    );

    let project_root = dir.path().join("proj");
    setup_project(&project_root, "[editor]\ntab_size = 2\n");
    manager.load_project(&project_root).unwrap();
    // Reset after load callback
    callback_fired.store(0, Ordering::SeqCst);

    manager.unload_project();

    assert_eq!(
        callback_fired.load(Ordering::SeqCst),
        1,
        "Callback must fire when unload reverts a watched key"
    );
}

// Validates: Requirement 5.6 — loading new project replaces previous one
#[test]
fn loading_new_project_replaces_previous_project_layer() {
    let dir = TempDir::new().unwrap();

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(Vec::new(), schema);

    // Load first project
    let proj1 = dir.path().join("proj1");
    setup_project(&proj1, "[editor]\ntab_size = 2\nindent_style = \"space\"\n");
    manager.load_project(&proj1).unwrap();

    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(2))
    );
    assert_eq!(
        manager.store().get_value("editor.indent_style"),
        Some(&ConfigValue::String("space".to_string()))
    );

    // Load second project (replaces first)
    let proj2 = dir.path().join("proj2");
    setup_project(&proj2, "[editor]\ntab_size = 8\n");
    let event = manager.load_project(&proj2).unwrap();

    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(8)),
        "New project value should be in effect"
    );
    // indent_style from proj1 should be gone
    assert_eq!(
        manager.store().get_value("editor.indent_style"),
        None,
        "Keys from previous project must be removed"
    );
    assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
    assert!(event
        .changed_keys
        .contains(&"editor.indent_style".to_string()));
}

// ============================================================================
// AC 5.7: Graceful failure handling (WARN log, skip, continue)
// ============================================================================

// Validates: Requirement 5.7 — invalid TOML: WARN, skip, continue operating
#[test]
fn invalid_toml_in_project_config_is_handled_gracefully() {
    let dir = TempDir::new().unwrap();
    let user_layer = make_layer(
        &dir,
        "user.toml",
        ConfigLayer::User,
        "[editor]\ntab_size = 4\n",
    );

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(vec![user_layer], schema);

    let project_root = dir.path().join("proj");
    let ffworkbench_dir = project_root.join(".ffworkbench");
    std::fs::create_dir_all(&ffworkbench_dir).unwrap();
    std::fs::write(
        ffworkbench_dir.join("config.toml"),
        "this is [[[not valid TOML",
    )
    .unwrap();

    // open_project handles errors gracefully
    let event = manager.open_project(&project_root);

    assert!(event.changed_keys.is_empty());
    assert!(!manager.has_project_layer());
    // User layer still accessible
    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(4)),
        "Other layers must remain accessible after project config failure"
    );
}

// Validates: Requirement 5.7 — missing/unreadable project config: WARN, skip, continue
#[test]
fn missing_project_config_is_handled_gracefully() {
    let dir = TempDir::new().unwrap();
    let project_root = dir.path().join("no-config-proj");
    // Don't create .ffworkbench directory at all

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(Vec::new(), schema);

    let event = manager.open_project(&project_root);

    assert!(event.changed_keys.is_empty());
    assert!(!manager.has_project_layer());
}

// Validates: Requirement 5.7 — I/O error (directory where file expected): WARN, skip
#[test]
fn io_error_in_project_config_is_handled_gracefully() {
    let dir = TempDir::new().unwrap();
    let project_root = dir.path().join("io-err-proj");
    let ffworkbench_dir = project_root.join(".ffworkbench");
    std::fs::create_dir_all(&ffworkbench_dir).unwrap();
    // Create a directory where config.toml should be (causes I/O error)
    std::fs::create_dir_all(ffworkbench_dir.join("config.toml")).unwrap();

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(Vec::new(), schema);

    // Should not panic — graceful handling
    let event = manager.open_project(&project_root);

    assert!(event.changed_keys.is_empty());
    assert!(!manager.has_project_layer());
}

// Validates: Requirement 5.7 — hot-reload rejects invalid TOML, retains previous values
#[test]
fn hot_reload_with_invalid_toml_retains_previous_project_values() {
    let dir = TempDir::new().unwrap();
    let project_root = dir.path().join("proj");
    let config_path = setup_project(&project_root, "[editor]\ntab_size = 2\n");

    let schema = SchemaRegistry::new();
    let mut manager = ReloadManager::new(Vec::new(), schema);
    manager.load_project(&project_root).unwrap();

    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(2))
    );

    // Corrupt the file
    std::fs::write(&config_path, "broken [[[").unwrap();

    // reload_file should reject and retain previous values
    let result = manager
        .reload_file(&config_path, ConfigLayer::Project)
        .unwrap();
    assert!(result.is_none(), "Invalid TOML reload should be rejected");

    assert_eq!(
        manager.store().get_value("editor.tab_size"),
        Some(&ConfigValue::Integer(2)),
        "Previous values must be retained after invalid TOML reload"
    );
}

// ============================================================================
// Additional lifecycle scenarios
// ============================================================================

// Validates: Requirement 5.6 — callbacks fire on project load when keys change
#[test]
fn callbacks_fire_on_project_load_when_keys_change() {
    let dir = TempDir::new().unwrap();
    let user_layer = make_layer(
        &dir,
        "user.toml",
        ConfigLayer::User,
        "[editor]\ntab_size = 4\n",
    );

    let schema = SchemaRegistry::new();
    let callbacks = Arc::new(CallbackRegistry::new());
    let mut manager =
        ReloadManager::with_callbacks(vec![user_layer], schema, Arc::clone(&callbacks));

    let load_fired = Arc::new(AtomicU32::new(0));
    let fired_clone = Arc::clone(&load_fired);
    let _handle = callbacks.on_reload(
        &["editor.tab_size"],
        Box::new(move |_event| {
            fired_clone.fetch_add(1, Ordering::SeqCst);
        }),
    );

    let project_root = dir.path().join("proj");
    setup_project(&project_root, "[editor]\ntab_size = 2\n");
    manager.load_project(&project_root).unwrap();

    assert_eq!(
        load_fired.load(Ordering::SeqCst),
        1,
        "Callback should fire on project load when watched key changes"
    );
}
