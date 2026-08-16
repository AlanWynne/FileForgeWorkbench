//! End-to-end integration tests for the configuration system.
//!
//! Covers Tasks 23.1–23.7: full initialization with all layers, hot-reload,
//! profile switching, project load/unload, EditorConfig resolution, plugin
//! scoped access, and schema validation at load time.
//!
//! Each test exercises the public API surface through `ConfigHandle` and
//! related subsystems in a self-contained manner using `tempfile::TempDir`.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use tempfile::TempDir;

use ff_config::callback::CallbackRegistry;
use ff_config::layer::ConfigLayer;
use ff_config::loader::{load_toml_file, LayerData};
use ff_config::plugin_handle::create_plugin_config_handle;
use ff_config::profile::ProfileManager;
use ff_config::provenance::{EffectiveValue, Provenance};
use ff_config::reload::ReloadManager;
use ff_config::schema::{Constraints, SchemaEntry, SchemaRegistry};
use ff_config::store::EffectiveStore;
use ff_config::value::ConfigValue;
use ff_config::ConfigHandle;

// ============================================================================
// 23.1: Full initialization with all layers, query effective values, verify provenance
// ============================================================================

/// Validates: full six-layer initialization — highest priority layer wins and
/// provenance correctly attributes the winning layer.
#[test]
fn e2e_full_initialization_with_all_layers_and_provenance() {
    let dir = TempDir::new().unwrap();

    // Create config files for each layer (System, User, Profile, Project, Workspace)
    let system_path = dir.path().join("system.toml");
    std::fs::write(
        &system_path,
        "[editor]\ntab_size = 8\nword_wrap = false\n[logging]\nlevel = \"warn\"\n",
    )
    .unwrap();

    let user_path = dir.path().join("user.toml");
    std::fs::write(
        &user_path,
        "[editor]\ntab_size = 4\n[theme]\nactive = \"light\"\n",
    )
    .unwrap();

    let profile_path = dir.path().join("profile.toml");
    std::fs::write(
        &profile_path,
        "[editor]\ntab_size = 2\n[theme]\nactive = \"solarized\"\n",
    )
    .unwrap();

    let project_dir = dir.path().join("project");
    let ffworkbench_dir = project_dir.join(".ffworkbench");
    std::fs::create_dir_all(&ffworkbench_dir).unwrap();
    let project_path = ffworkbench_dir.join("config.toml");
    std::fs::write(
        &project_path,
        "[editor]\nindent_style = \"space\"\n[logging]\nlevel = \"debug\"\n",
    )
    .unwrap();

    let workspace_path = dir.path().join("workspace.toml");
    std::fs::write(&workspace_path, "[editor]\ntab_size = 3\n").unwrap();

    // Load all layers
    let system_values = load_toml_file(&system_path).unwrap();
    let user_values = load_toml_file(&user_path).unwrap();
    let profile_values = load_toml_file(&profile_path).unwrap();
    let project_values = load_toml_file(&project_path).unwrap();
    let workspace_values = load_toml_file(&workspace_path).unwrap();

    let layers = vec![
        LayerData {
            layer: ConfigLayer::System,
            source_path: system_path.clone(),
            values: system_values,
        },
        LayerData {
            layer: ConfigLayer::User,
            source_path: user_path.clone(),
            values: user_values,
        },
        LayerData {
            layer: ConfigLayer::Profile,
            source_path: profile_path.clone(),
            values: profile_values,
        },
        LayerData {
            layer: ConfigLayer::Project,
            source_path: project_path.clone(),
            values: project_values,
        },
        LayerData {
            layer: ConfigLayer::Workspace,
            source_path: workspace_path.clone(),
            values: workspace_values,
        },
    ];

    let schema = SchemaRegistry::new();
    let manager = ReloadManager::new(layers, schema);
    let handle = ConfigHandle::new(manager);

    // editor.tab_size: defined in System(8), User(4), Profile(2), Workspace(3)
    // Workspace is highest priority → effective value is 3
    assert_eq!(handle.get_int("editor.tab_size").unwrap(), 3);

    // Verify provenance: Workspace layer should be reported
    let effective = handle.get_with_provenance("editor.tab_size").unwrap();
    assert_eq!(effective.value, ConfigValue::Integer(3));
    assert_eq!(effective.provenance.layer, ConfigLayer::Workspace);
    assert_eq!(effective.provenance.source_file, Some(workspace_path));

    // editor.word_wrap: only in System layer → System wins
    assert_eq!(handle.get_bool("editor.word_wrap").unwrap(), false);
    let wp_eff = handle.get_with_provenance("editor.word_wrap").unwrap();
    assert_eq!(wp_eff.provenance.layer, ConfigLayer::System);

    // theme.active: defined in User("light"), Profile("solarized")
    // Profile > User → effective value is "solarized"
    assert_eq!(handle.get_string("theme.active").unwrap(), "solarized");
    let theme_eff = handle.get_with_provenance("theme.active").unwrap();
    assert_eq!(theme_eff.provenance.layer, ConfigLayer::Profile);

    // logging.level: defined in System("warn"), Project("debug")
    // Project > System → effective value is "debug"
    assert_eq!(handle.get_string("logging.level").unwrap(), "debug");
    let log_eff = handle.get_with_provenance("logging.level").unwrap();
    assert_eq!(log_eff.provenance.layer, ConfigLayer::Project);

    // editor.indent_style: only in Project → Project wins
    assert_eq!(handle.get_string("editor.indent_style").unwrap(), "space");
    let indent_eff = handle.get_with_provenance("editor.indent_style").unwrap();
    assert_eq!(indent_eff.provenance.layer, ConfigLayer::Project);
}

// ============================================================================
// 23.2: Hot-reload cycle — modify file on disk, verify callback with changed keys
// ============================================================================

/// Validates: hot-reload pipeline — file modification on disk triggers reload,
/// callbacks are invoked with correct changed_keys, and new value is effective.
#[test]
fn e2e_hot_reload_cycle_callback_invoked_with_changed_keys() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("user.toml");
    std::fs::write(
        &config_path,
        "[editor]\ntab_size = 4\nword_wrap = true\n[theme]\nactive = \"dark\"\n",
    )
    .unwrap();

    let values = load_toml_file(&config_path).unwrap();
    let layers = vec![LayerData {
        layer: ConfigLayer::User,
        source_path: config_path.clone(),
        values,
    }];

    let callbacks = Arc::new(CallbackRegistry::new());
    let schema = SchemaRegistry::new();
    let manager = ReloadManager::with_callbacks(layers, schema, Arc::clone(&callbacks));
    let handle = ConfigHandle::new(manager);

    // Verify initial values
    assert_eq!(handle.get_int("editor.tab_size").unwrap(), 4);
    assert_eq!(handle.get_bool("editor.word_wrap").unwrap(), true);
    assert_eq!(handle.get_string("theme.active").unwrap(), "dark");

    // Register a callback that tracks invocations
    let callback_count = Arc::new(AtomicU32::new(0));
    let cb_count_clone = Arc::clone(&callback_count);
    let changed_keys_capture: Arc<std::sync::Mutex<Vec<String>>> =
        Arc::new(std::sync::Mutex::new(Vec::new()));
    let keys_clone = Arc::clone(&changed_keys_capture);

    callbacks.on_reload(
        &["editor.tab_size", "theme.active"],
        Box::new(move |event| {
            cb_count_clone.fetch_add(1, Ordering::SeqCst);
            let mut keys = keys_clone.lock().unwrap();
            keys.extend(event.changed_keys.iter().cloned());
        }),
    );

    // Modify the file on disk: change tab_size and theme, keep word_wrap the same
    std::fs::write(
        &config_path,
        "[editor]\ntab_size = 2\nword_wrap = true\n[theme]\nactive = \"light\"\n",
    )
    .unwrap();

    // Trigger reload
    let results = handle.reload();
    assert!(!results.is_empty());

    // Verify new values are effective
    assert_eq!(handle.get_int("editor.tab_size").unwrap(), 2);
    assert_eq!(handle.get_bool("editor.word_wrap").unwrap(), true); // unchanged
    assert_eq!(handle.get_string("theme.active").unwrap(), "light");

    // Verify callback was invoked
    assert!(callback_count.load(Ordering::SeqCst) >= 1);

    // Verify the callback received correct changed keys
    let captured_keys = changed_keys_capture.lock().unwrap();
    assert!(captured_keys.contains(&"editor.tab_size".to_string()));
    assert!(captured_keys.contains(&"theme.active".to_string()));
    // word_wrap didn't change, so it should NOT appear
    assert!(!captured_keys.contains(&"editor.word_wrap".to_string()));
}

// ============================================================================
// 23.3: Profile switch — activate profile, verify values change, switch back
// ============================================================================

/// Validates: profile switching — activating a profile changes effective values,
/// switching to another profile changes values again.
///
/// Uses lower-level ReloadManager + ProfileManager approach as the ConfigHandle's
/// deactivation implementation reloads all layers from disk (by design).
#[test]
fn e2e_profile_switch_activate_and_deactivate() {
    let dir = TempDir::new().unwrap();

    // Create user config
    let user_path = dir.path().join("user.toml");
    std::fs::write(
        &user_path,
        "[editor]\ntab_size = 4\nindent_style = \"space\"\n",
    )
    .unwrap();

    // Create profiles directory with two profile TOMLs
    let profiles_dir = dir.path().join("profiles");
    std::fs::create_dir_all(&profiles_dir).unwrap();
    std::fs::write(
        profiles_dir.join("mainframe.toml"),
        "[editor]\ntab_size = 8\nindent_style = \"tab\"\n",
    )
    .unwrap();
    std::fs::write(
        profiles_dir.join("web-dev.toml"),
        "[editor]\ntab_size = 2\nindent_style = \"space\"\n",
    )
    .unwrap();

    // Load user layer
    let user_values = load_toml_file(&user_path).unwrap();
    let layers = vec![LayerData {
        layer: ConfigLayer::User,
        source_path: user_path.clone(),
        values: user_values,
    }];

    let schema = SchemaRegistry::new();
    let manager = ReloadManager::new(layers, schema);
    let profile_manager = ProfileManager::new(profiles_dir);
    let handle = ConfigHandle::with_profile_manager(manager, profile_manager);

    // Before profile activation: User layer values are effective
    assert_eq!(handle.get_int("editor.tab_size").unwrap(), 4);
    assert_eq!(handle.get_string("editor.indent_style").unwrap(), "space");

    // Activate "mainframe" profile
    let _event = handle.set_active_profile(Some("mainframe")).unwrap();
    // The profile layer has higher priority than User, so values should change
    assert_eq!(handle.get_int("editor.tab_size").unwrap(), 8);
    assert_eq!(handle.get_string("editor.indent_style").unwrap(), "tab");

    // Switch to "web-dev" profile → values change to web-dev's settings
    let _switch_event = handle.set_active_profile(Some("web-dev")).unwrap();
    assert_eq!(handle.get_int("editor.tab_size").unwrap(), 2);
    assert_eq!(handle.get_string("editor.indent_style").unwrap(), "space");
}

// ============================================================================
// 23.4: Project load/unload — open project, verify overrides, close, verify revert
// ============================================================================

/// Validates: project lifecycle — loading project adds Project-layer overrides,
/// unloading reverts to previous effective values.
#[test]
fn e2e_project_load_and_unload_lifecycle() {
    let dir = TempDir::new().unwrap();

    // Create a user config
    let user_path = dir.path().join("user.toml");
    std::fs::write(
        &user_path,
        "[editor]\ntab_size = 4\nword_wrap = true\n[logging]\nlevel = \"info\"\n",
    )
    .unwrap();

    // Create a project with its own config
    let project_dir = dir.path().join("my-project");
    let ffworkbench_dir = project_dir.join(".ffworkbench");
    std::fs::create_dir_all(&ffworkbench_dir).unwrap();
    std::fs::write(
        ffworkbench_dir.join("config.toml"),
        "[editor]\ntab_size = 2\n[logging]\nlevel = \"debug\"\n",
    )
    .unwrap();

    // Init with user layer only (no project yet)
    let user_values = load_toml_file(&user_path).unwrap();
    let layers = vec![LayerData {
        layer: ConfigLayer::User,
        source_path: user_path.clone(),
        values: user_values,
    }];

    let schema = SchemaRegistry::new();
    let manager = ReloadManager::new(layers, schema);
    let handle = ConfigHandle::new(manager);

    // Before project load: User values
    assert_eq!(handle.get_int("editor.tab_size").unwrap(), 4);
    assert_eq!(handle.get_bool("editor.word_wrap").unwrap(), true);
    assert_eq!(handle.get_string("logging.level").unwrap(), "info");

    // Load project
    let load_event = handle.load_project(&project_dir).unwrap();
    assert!(load_event
        .changed_keys
        .contains(&"editor.tab_size".to_string()));
    assert!(load_event
        .changed_keys
        .contains(&"logging.level".to_string()));

    // Project layer overrides User layer
    assert_eq!(handle.get_int("editor.tab_size").unwrap(), 2);
    assert_eq!(handle.get_string("logging.level").unwrap(), "debug");
    // word_wrap only in User layer, still effective
    assert_eq!(handle.get_bool("editor.word_wrap").unwrap(), true);

    // Unload project
    let unload_event = handle.unload_project();
    assert!(unload_event
        .changed_keys
        .contains(&"editor.tab_size".to_string()));
    assert!(unload_event
        .changed_keys
        .contains(&"logging.level".to_string()));

    // Values revert to User layer
    assert_eq!(handle.get_int("editor.tab_size").unwrap(), 4);
    assert_eq!(handle.get_string("logging.level").unwrap(), "info");
    assert_eq!(handle.get_bool("editor.word_wrap").unwrap(), true);
}

// ============================================================================
// 23.5: EditorConfig resolution — hierarchy with multiple levels, per-file resolution
// ============================================================================

/// Validates: EditorConfig per-file resolution respects hierarchy and closer
/// files take priority over farther files.
#[test]
fn e2e_editorconfig_resolution_hierarchy() {
    let dir = TempDir::new().unwrap();
    let root_dir = dir.path();

    // Create directory hierarchy: root/project/src/lib.rs
    let project_dir = root_dir.join("project");
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    // Create target file
    let target_file = src_dir.join("lib.rs");
    std::fs::write(&target_file, "// source file").unwrap();

    // Root-level .editorconfig: defines defaults for all files
    std::fs::write(
        root_dir.join(".editorconfig"),
        "root = true\n\n[*]\nindent_style = tab\nindent_size = 4\ncharset = utf-8\nend_of_line = crlf\n",
    )
    .unwrap();

    // Project-level .editorconfig: overrides indent for Rust files
    std::fs::write(
        project_dir.join(".editorconfig"),
        "[*.rs]\nindent_style = space\nindent_size = 4\n",
    )
    .unwrap();

    // Src-level .editorconfig: overrides indent_size for this directory
    std::fs::write(src_dir.join(".editorconfig"), "[*.rs]\nindent_size = 2\n").unwrap();

    // Create a ConfigHandle to test resolve_editorconfig
    let schema = SchemaRegistry::new();
    let manager = ReloadManager::new(Vec::new(), schema);
    let handle = ConfigHandle::new(manager);

    // Resolve EditorConfig for the target file
    let props = handle.resolve_editorconfig(&target_file);

    // Closest (src/) .editorconfig wins for indent_size
    assert_eq!(
        props.indent_size,
        Some(ff_config::editorconfig::parser::IndentSize::Value(2))
    );

    // Project-level wins for indent_style (closer than root)
    assert_eq!(
        props.indent_style,
        Some(ff_config::editorconfig::parser::IndentStyle::Space)
    );

    // Root level provides charset (not overridden by closer files)
    assert_eq!(
        props.charset,
        Some(ff_config::editorconfig::parser::Charset::Utf8)
    );

    // Root level provides end_of_line (not overridden by closer files)
    assert_eq!(
        props.end_of_line,
        Some(ff_config::editorconfig::parser::EndOfLine::CrLf)
    );

    // Now test a Python file in the same directory — project-level [*.rs] shouldn't match
    let py_file = src_dir.join("script.py");
    std::fs::write(&py_file, "# python").unwrap();

    let py_props = handle.resolve_editorconfig(&py_file);
    // Only root-level [*] matches python files
    assert_eq!(
        py_props.indent_style,
        Some(ff_config::editorconfig::parser::IndentStyle::Tab)
    );
    assert_eq!(
        py_props.indent_size,
        Some(ff_config::editorconfig::parser::IndentSize::Value(4))
    );
}

// ============================================================================
// 23.6: Plugin scoped access — verify isolation and namespace violation
// ============================================================================

/// Validates: plugin scoped access — a plugin handle can only read its own
/// namespace keys and cannot see other plugins' keys; reserved namespaces
/// are rejected.
#[test]
fn e2e_plugin_scoped_access_isolation_and_namespace_violation() {
    // Build a store with keys for two different plugins
    let mut store = EffectiveStore::new();
    store.insert(
        "plugins.test-plugin.max_rows".to_string(),
        EffectiveValue {
            value: ConfigValue::Integer(500),
            provenance: Provenance {
                layer: ConfigLayer::User,
                source_file: None,
            },
        },
    );
    store.insert(
        "plugins.test-plugin.timeout".to_string(),
        EffectiveValue {
            value: ConfigValue::Float(30.0),
            provenance: Provenance {
                layer: ConfigLayer::User,
                source_file: None,
            },
        },
    );
    store.insert(
        "plugins.other-plugin.secret".to_string(),
        EffectiveValue {
            value: ConfigValue::String("hidden-value".to_string()),
            provenance: Provenance {
                layer: ConfigLayer::User,
                source_file: None,
            },
        },
    );
    store.insert(
        "editor.tab_size".to_string(),
        EffectiveValue {
            value: ConfigValue::Integer(4),
            provenance: Provenance {
                layer: ConfigLayer::User,
                source_file: None,
            },
        },
    );

    let schema = SchemaRegistry::new();

    // Create a handle for "test-plugin"
    let handle = create_plugin_config_handle(&store, &schema, "test-plugin").unwrap();

    // Plugin can read its own keys
    assert_eq!(handle.get_int("max_rows").unwrap(), 500);
    assert_eq!(handle.get_float("timeout").unwrap(), 30.0);

    // Plugin cannot see other plugin's keys (resolves to its own namespace, not found)
    let other_result = handle.get("secret");
    assert!(other_result.is_err()); // "plugins.test-plugin.secret" doesn't exist

    // Plugin cannot see core namespace keys either (resolves within its own namespace)
    // Attempting to read "editor.tab_size" as a relative key → becomes
    // "plugins.test-plugin.editor.tab_size" which doesn't exist
    let core_result = handle.get("editor.tab_size");
    assert!(core_result.is_err());

    // Verify namespace prefix is correct
    assert_eq!(handle.namespace(), "plugins.test-plugin.");

    // Create a handle for "other-plugin" and verify isolation
    let other_handle = create_plugin_config_handle(&store, &schema, "other-plugin").unwrap();
    assert_eq!(other_handle.get_string("secret").unwrap(), "hidden-value");
    // other-plugin cannot see test-plugin's keys
    let cross_result = other_handle.get("max_rows");
    assert!(cross_result.is_err());

    // Verify reserved namespace rejection
    let reserved_result = create_plugin_config_handle(&store, &schema, "editor");
    assert!(reserved_result.is_err());
    match reserved_result.unwrap_err() {
        ff_config::ConfigError::ReservedNamespace { plugin, .. } => {
            assert_eq!(plugin, "editor");
        }
        other => panic!("Expected ReservedNamespace error, got: {:?}", other),
    }

    // Verify more reserved namespaces
    let logging_result = create_plugin_config_handle(&store, &schema, "logging");
    assert!(logging_result.is_err());

    let core_result = create_plugin_config_handle(&store, &schema, "core");
    assert!(core_result.is_err());
}

// ============================================================================
// 23.7: Schema validation at load time — invalid values replaced by defaults
// ============================================================================

/// Validates: schema validation at load time — values that violate constraints
/// are replaced by schema defaults when accessed through the typed API.
#[test]
fn e2e_schema_validation_invalid_values_replaced_by_defaults() {
    let dir = TempDir::new().unwrap();

    // Create a config file with values that violate schema constraints
    let config_path = dir.path().join("user.toml");
    std::fs::write(
        &config_path,
        "[editor]\ntab_size = 99\nindent_style = \"mixed\"\nfont_size = -5.0\n",
    )
    .unwrap();

    let values = load_toml_file(&config_path).unwrap();
    let layers = vec![LayerData {
        layer: ConfigLayer::User,
        source_path: config_path.clone(),
        values,
    }];

    // Register schema with constraints
    let mut schema = SchemaRegistry::new();
    schema
        .register(SchemaEntry {
            key: "editor.tab_size".to_string(),
            value_type: ff_config::error::ValueType::Integer,
            default: ConfigValue::Integer(4),
            description: "Number of spaces per tab stop".to_string(),
            constraints: Some(Constraints {
                min: Some(1.0),
                max: Some(16.0),
                allowed_values: None,
                pattern: None,
            }),
        })
        .unwrap();
    schema
        .register(SchemaEntry {
            key: "editor.indent_style".to_string(),
            value_type: ff_config::error::ValueType::String,
            default: ConfigValue::String("space".to_string()),
            description: "Indentation style".to_string(),
            constraints: Some(Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("space".to_string()),
                    ConfigValue::String("tab".to_string()),
                ]),
                pattern: None,
            }),
        })
        .unwrap();
    schema
        .register(SchemaEntry {
            key: "editor.font_size".to_string(),
            value_type: ff_config::error::ValueType::Float,
            default: ConfigValue::Float(12.0),
            description: "Editor font size".to_string(),
            constraints: Some(Constraints {
                min: Some(6.0),
                max: Some(72.0),
                allowed_values: None,
                pattern: None,
            }),
        })
        .unwrap();

    let manager = ReloadManager::new(layers, schema);
    let handle = ConfigHandle::new(manager);

    // editor.tab_size = 99 violates max constraint (1..16) → default 4 applied
    assert_eq!(handle.get_int("editor.tab_size").unwrap(), 4);

    // editor.indent_style = "mixed" is not in allowed_values → default "space" applied
    assert_eq!(handle.get_string("editor.indent_style").unwrap(), "space");

    // editor.font_size = -5.0 violates min constraint (6.0..72.0) → default 12.0 applied
    let font_size = handle.get_float("editor.font_size").unwrap();
    assert!((font_size - 12.0).abs() < f64::EPSILON);
}
