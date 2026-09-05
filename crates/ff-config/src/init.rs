//! Configuration system initialization.
//!
//! Entry point for bootstrapping the configuration system: resolves paths,
//! loads all layer files, registers core schema entries, starts the file
//! watcher, and returns the initialized configuration store.
//!
//! The initialization sequence automatically detects and loads the project-layer
//! configuration when a `project_root` is provided in `ConfigInitOptions`.

use std::path::PathBuf;

use crate::config_handle::ConfigHandle;
use crate::error::{ConfigError, ValueType};
use crate::layer::ConfigLayer;
use crate::loader::{load_toml_file, LayerData};
use crate::paths;
use crate::profile::ProfileManager;
use crate::reload::{ReloadEvent, ReloadManager};
use crate::schema::{SchemaEntry, SchemaRegistry};
use crate::value::ConfigValue;
use crate::watcher::ConfigWatcher;

/// Options for initializing the configuration system.
///
/// Controls which layers are loaded and whether file watching is enabled.
/// When `project_root` is `Some`, the initialization sequence automatically
/// detects and loads the project-layer configuration file if it exists.
///
/// Addresses: Requirement 5, criterion 2 (automatic detection on project open)
#[derive(Debug, Clone)]
pub struct ConfigInitOptions {
    /// The project root directory. When provided, the initialization sequence
    /// automatically detects `.ffworkbench/config.toml` in this directory and
    /// loads it as the Project layer. If the file does not exist, no error is
    /// raised — the project simply has no project-layer config.
    pub project_root: Option<PathBuf>,

    /// The workspace root directory. When provided, the initialization sequence
    /// loads the workspace-layer configuration file from this path.
    pub workspace_root: Option<PathBuf>,

    /// Whether to start file watching for hot-reload (default: true).
    pub enable_hot_reload: bool,
}

impl Default for ConfigInitOptions {
    fn default() -> Self {
        Self {
            project_root: None,
            workspace_root: None,
            enable_hot_reload: true,
        }
    }
}

impl ConfigInitOptions {
    /// Create a new `ConfigInitOptions` with default values.
    ///
    /// By default, no project or workspace root is set and hot-reload is enabled.
    pub fn new() -> Self {
        Self {
            project_root: None,
            workspace_root: None,
            enable_hot_reload: true,
        }
    }

    /// Set the project root directory for automatic project config detection.
    pub fn with_project_root(mut self, project_root: PathBuf) -> Self {
        self.project_root = Some(project_root);
        self
    }

    /// Set the workspace root directory.
    pub fn with_workspace_root(mut self, workspace_root: PathBuf) -> Self {
        self.workspace_root = Some(workspace_root);
        self
    }

    /// Set whether hot-reload file watching should be enabled.
    pub fn with_hot_reload(mut self, enable: bool) -> Self {
        self.enable_hot_reload = enable;
        self
    }
}

/// Register core schema entries for all well-known configuration keys.
///
/// Populates the schema registry with the default entries for editor, logging,
/// theme, and VFS namespaces. These defaults serve as the Defaults layer (priority 0)
/// in the six-layer model.
///
/// Addresses: Requirement 2 (AC 2.1), Requirement 9 (AC 9.1)
pub fn register_core_schema(schema: &mut SchemaRegistry) {
    let entries = [
        SchemaEntry {
            key: crate::keys::editor::TAB_SIZE.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(4),
            description: "Number of spaces per tab stop".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: Some(1.0),
                max: Some(16.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::editor::INDENT_STYLE.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("space".to_string()),
            description: "Indent style: space or tab".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("space".to_string()),
                    ConfigValue::String("tab".to_string()),
                ]),
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::editor::LINE_ENDINGS.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("lf".to_string()),
            description: "Line ending style: lf, crlf, or cr".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("lf".to_string()),
                    ConfigValue::String("crlf".to_string()),
                    ConfigValue::String("cr".to_string()),
                ]),
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::editor::TRIM_TRAILING_WHITESPACE.to_string(),
            value_type: ValueType::Boolean,
            default: ConfigValue::Boolean(false),
            description: "Whether to trim trailing whitespace on save".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: crate::keys::editor::INSERT_FINAL_NEWLINE.to_string(),
            value_type: ValueType::Boolean,
            default: ConfigValue::Boolean(true),
            description: "Whether to insert a final newline on save".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: crate::keys::logging::LEVEL.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("info".to_string()),
            description: "Logging level: trace, debug, info, warn, error".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("trace".to_string()),
                    ConfigValue::String("debug".to_string()),
                    ConfigValue::String("info".to_string()),
                    ConfigValue::String("warn".to_string()),
                    ConfigValue::String("error".to_string()),
                ]),
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::logging::DIRECTORY.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String(String::new()),
            description: "Directory for log file output".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: crate::keys::logging::MAX_FILE_SIZE_MB.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(10),
            description: "Maximum log file size in megabytes before rotation".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: Some(1.0),
                max: Some(1024.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::logging::MAX_RETAINED_FILES.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(5),
            description: "Maximum number of retained rotated log files".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: Some(1.0),
                max: Some(100.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::theme::ACTIVE.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("default".to_string()),
            description: "Active theme name".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: crate::keys::theme::FONT_SIZE.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(14),
            description: "Font size in points".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: Some(6.0),
                max: Some(72.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::vfs::DEFAULT_PROVIDER.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("local".to_string()),
            description: "Default virtual file system provider".to_string(),
            constraints: None,
        },
    ];

    for entry in entries {
        // Core schema registration should never conflict — unwrap is safe here
        // since we control all entries and they have unique keys.
        schema
            .register(entry)
            .expect("core schema entries must not conflict");
    }
}

/// Register catalog schema entries with resolved default paths.
///
/// Called from `ff-desktop` after the user data directory is resolved,
/// so the defaults are concrete filesystem paths rather than templates.
///
/// Addresses: Requirement 12.3, 12.4, 12.5
pub fn register_catalog_schema(
    schema: &mut SchemaRegistry,
    mainframe_root: &str,
    posix_root: &str,
) {
    let entries = [
        SchemaEntry {
            key: crate::keys::catalogs::DEFAULT_MAINFRAME_ROOT.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String(mainframe_root.to_string()),
            description: "Default repository root directory for new Mainframe catalogs".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: crate::keys::catalogs::DEFAULT_POSIX_ROOT.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String(posix_root.to_string()),
            description: "Default root directory for new POSIX catalogs".to_string(),
            constraints: None,
        },
    ];
    for entry in entries {
        schema
            .register(entry)
            .expect("catalog schema entries must not conflict");
    }
}

///
/// Performs the full initialization sequence:
/// 1. Register core schema defaults
/// 2. Load system configuration file (gracefully skip if missing)
/// 3. Load user configuration file (gracefully skip if missing)
/// 4. Auto-activate persisted profile (if recorded in user config)
/// 5. Detect and load project configuration file (if project_root provided)
/// 6. Detect and load workspace configuration file (if workspace_root provided)
/// 7. Start file watcher (if enable_hot_reload is true)
///
/// Missing layer files are skipped silently — only files that exist and are
/// readable are loaded. Invalid TOML in any layer is logged as WARN and the
/// layer is skipped.
///
/// Addresses: Requirement 1 (AC 1.1, 1.2), Requirement 2 (AC 2.1), Requirement 4 (AC 4.5)
pub fn init(options: ConfigInitOptions) -> Result<ConfigHandle, ConfigError> {
    // Step 1: Register core schema defaults
    let mut schema = SchemaRegistry::new();
    register_core_schema(&mut schema);

    // Step 2: Load layers in order (gracefully handle missing files)
    let mut layers: Vec<LayerData> = Vec::new();

    // System config
    let system_path = paths::system_config_path();
    if system_path.exists() {
        match load_toml_file(&system_path) {
            Ok(values) => {
                layers.push(LayerData {
                    layer: ConfigLayer::System,
                    source_path: system_path,
                    values,
                });
            }
            Err(e) => {
                ff_logging::log_warn!("[config] init: skipping system config: {}", e);
            }
        }
    }

    // User config
    if let Some(user_path) = paths::user_config_path() {
        if user_path.exists() {
            match load_toml_file(&user_path) {
                Ok(values) => {
                    layers.push(LayerData {
                        layer: ConfigLayer::User,
                        source_path: user_path,
                        values,
                    });
                }
                Err(e) => {
                    ff_logging::log_warn!("[config] init: skipping user config: {}", e);
                }
            }
        }
    }

    // Step 3: Build ReloadManager with initial layers
    let manager = ReloadManager::new(layers, schema);

    // Step 4: Create ConfigHandle (with profile manager if profiles dir exists)
    let handle = if let Some(profiles_dir) = paths::user_profiles_dir() {
        let profile_manager = ProfileManager::new(profiles_dir);
        ConfigHandle::with_profile_manager(manager, profile_manager)
    } else {
        ConfigHandle::new(manager)
    };

    // Step 5: Auto-activate persisted profile
    // The persisted profile name is stored under [_session].active_profile in the user config.
    // We attempt to read it from the effective store and activate if present.
    if let Ok(profile_name) = handle.get_string("_session.active_profile") {
        if !profile_name.is_empty() {
            if let Err(e) = handle.set_active_profile(Some(&profile_name)) {
                ff_logging::log_warn!(
                    "[config] init: failed to auto-activate profile '{}': {}",
                    profile_name,
                    e
                );
            }
        }
    }

    // Step 6: Load project config if project_root specified
    if let Some(ref project_root) = options.project_root {
        if let Err(e) = handle.load_project(project_root) {
            ff_logging::log_warn!(
                "[config] init: failed to load project config from '{}': {}",
                project_root.display(),
                e
            );
        }
    }

    // Step 7: Load workspace config if workspace_root specified
    if let Some(ref workspace_root) = options.workspace_root {
        let workspace_config_path = workspace_root.join(".ffworkbench").join("config.toml");
        if workspace_config_path.exists() {
            let mut system = handle.inner_write();
            if let Err(e) = system
                .manager
                .reload_file(&workspace_config_path, ConfigLayer::Workspace)
            {
                ff_logging::log_warn!("[config] init: skipping workspace config: {}", e);
            }
        }
    }

    // Step 8: Start file watcher if enabled
    if options.enable_hot_reload {
        match ConfigWatcher::new() {
            Ok(watcher) => {
                handle.set_watcher(watcher);
            }
            Err(e) => {
                ff_logging::log_warn!("[config] init: failed to start file watcher: {}", e);
            }
        }
    }

    Ok(handle)
}

/// Shut down the configuration system.
///
/// Performs orderly shutdown:
/// 1. Stops file watching (if active)
/// 2. Deregisters all reload callbacks
///
/// After shutdown, the handle can still be used for reads (existing data remains)
/// but no further reload events will be generated.
///
/// Addresses: Requirement 1, Requirement 3 (lifecycle management)
pub fn shutdown(handle: &ConfigHandle) {
    // Stop the file watcher
    handle.stop_watcher();

    // Deregister all callbacks
    handle.clear_callbacks();
}

/// Apply automatic project configuration detection during initialization.
///
/// If `options.project_root` is `Some`, this function calls
/// `ReloadManager::open_project()` which detects `.ffworkbench/config.toml`
/// in the project root and loads it if present. Errors are handled gracefully
/// (logged as WARN, not propagated).
///
/// This function is intended to be called as part of the full initialization
/// sequence (Task 21) after all lower-priority layers have been loaded.
///
/// Returns the `ReloadEvent` from project loading (empty if no config found
/// or if loading failed gracefully).
///
/// Addresses: Requirement 5, criterion 2 (automatic detection when project opened)
pub fn auto_detect_project_config(
    manager: &mut ReloadManager,
    options: &ConfigInitOptions,
) -> Option<ReloadEvent> {
    options
        .project_root
        .as_ref()
        .map(|project_root| manager.open_project(project_root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::ConfigLayer;
    use crate::loader::load_toml_file;
    use crate::loader::LayerData;
    use crate::schema::SchemaRegistry;
    use crate::value::ConfigValue;
    use tempfile::TempDir;

    // ========================================================================
    // 15.3 — Automatic project config detection when project is opened
    // ========================================================================

    // Validates: Requirement 5.2 — auto_detect_project_config loads project config when project_root is set
    #[test]
    fn auto_detect_loads_project_config_when_project_root_set() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let options = ConfigInitOptions::new().with_project_root(dir.path().to_path_buf());

        let event = auto_detect_project_config(&mut manager, &options);

        assert!(
            event.is_some(),
            "Should return Some when project_root is set"
        );
        let event = event.unwrap();
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
        assert_eq!(event.source_layer, ConfigLayer::Project);

        // Verify the project layer was loaded
        assert!(manager.has_project_layer());
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2))
        );
    }

    // Validates: Requirement 5.2 — auto_detect returns None when no project_root in options
    #[test]
    fn auto_detect_returns_none_when_no_project_root() {
        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let options = ConfigInitOptions::new();

        let event = auto_detect_project_config(&mut manager, &options);

        assert!(
            event.is_none(),
            "Should return None when project_root is not set"
        );
        assert!(!manager.has_project_layer());
    }

    // Validates: Requirement 5.2 — auto_detect does nothing when config file doesn't exist
    #[test]
    fn auto_detect_does_nothing_when_no_config_file_exists() {
        let dir = TempDir::new().unwrap();
        // No .ffworkbench/config.toml

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let options = ConfigInitOptions::new().with_project_root(dir.path().to_path_buf());

        let event = auto_detect_project_config(&mut manager, &options);

        assert!(event.is_some());
        let event = event.unwrap();
        assert!(
            event.changed_keys.is_empty(),
            "Should have no changed keys when config file doesn't exist"
        );
        assert!(!manager.has_project_layer());
    }

    // Validates: Requirement 5.7 — auto_detect handles invalid TOML gracefully (no error propagated)
    #[test]
    fn auto_detect_handles_invalid_toml_gracefully() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "this is not valid [[[toml content",
        )
        .unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let options = ConfigInitOptions::new().with_project_root(dir.path().to_path_buf());

        // Should NOT panic or propagate error
        let event = auto_detect_project_config(&mut manager, &options);

        assert!(event.is_some());
        let event = event.unwrap();
        assert!(
            event.changed_keys.is_empty(),
            "Should have no changed keys when TOML is invalid"
        );
        assert!(!manager.has_project_layer());
    }

    // Validates: Requirement 5.2 — auto_detect merges project config at correct priority
    #[test]
    fn auto_detect_merges_at_project_priority_overriding_user_layer() {
        let dir = TempDir::new().unwrap();

        // User layer with tab_size = 4
        let user_path = dir.path().join("user.toml");
        std::fs::write(&user_path, "[editor]\ntab_size = 4\n").unwrap();
        let user_values = load_toml_file(&user_path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: user_path,
            values: user_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Verify User layer is effective
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(4))
        );

        // Set up project config
        let project_root = dir.path().join("my-project");
        let ffworkbench_dir = project_root.join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();

        let options = ConfigInitOptions::new().with_project_root(project_root);

        auto_detect_project_config(&mut manager, &options);

        // Project layer (priority 4) should override User (priority 2)
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2))
        );
    }

    // Validates: Requirement 5.2 — open_project on ReloadManager works as automatic entry point
    #[test]
    fn open_project_loads_config_seamlessly() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[logging]\nlevel = \"debug\"\n",
        )
        .unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        // open_project is the runtime API for "a project was just opened"
        let event = manager.open_project(dir.path());

        assert!(event.changed_keys.contains(&"logging.level".to_string()));
        assert!(manager.has_project_layer());
        assert_eq!(
            manager.store().get_value("logging.level"),
            Some(&ConfigValue::String("debug".to_string()))
        );
    }

    // Validates: Requirement 5.2 — open_project with no config does nothing
    #[test]
    fn open_project_with_no_config_file_is_noop() {
        let dir = TempDir::new().unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let event = manager.open_project(dir.path());

        assert!(event.changed_keys.is_empty());
        assert!(!manager.has_project_layer());
    }

    // Validates: Requirement 5.7 — open_project with invalid TOML doesn't propagate error
    #[test]
    fn open_project_with_invalid_toml_returns_empty_event() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(ffworkbench_dir.join("config.toml"), "broken = [[[").unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        // Should not panic — errors are handled gracefully
        let event = manager.open_project(dir.path());

        assert!(event.changed_keys.is_empty());
        assert!(!manager.has_project_layer());
    }

    // Validates: ConfigInitOptions builder pattern works correctly
    #[test]
    fn config_init_options_builder_sets_fields() {
        let options = ConfigInitOptions::new()
            .with_project_root(PathBuf::from("/my/project"))
            .with_workspace_root(PathBuf::from("/my/workspace"))
            .with_hot_reload(false);

        assert_eq!(options.project_root, Some(PathBuf::from("/my/project")));
        assert_eq!(options.workspace_root, Some(PathBuf::from("/my/workspace")));
        assert!(!options.enable_hot_reload);
    }

    // Validates: ConfigInitOptions::default() has expected defaults
    #[test]
    fn config_init_options_default_has_hot_reload_enabled() {
        let options = ConfigInitOptions::default();

        assert!(options.project_root.is_none());
        assert!(options.workspace_root.is_none());
        assert!(options.enable_hot_reload); // Default is true per design
    }

    // ========================================================================
    // 21.1 — init() function returns ConfigHandle
    // ========================================================================

    // Validates: Requirement 1.1, 1.2 — init with no config files succeeds with schema defaults
    // Uses ReloadManager directly to avoid loading the real user config file from disk,
    // which may override schema defaults on the developer's machine (B029).
    #[test]
    fn init_with_no_config_files_succeeds_with_schema_defaults() {
        let mut schema = SchemaRegistry::new();
        register_core_schema(&mut schema);
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);

        // Schema defaults should be available
        assert_eq!(handle.get_int("editor.tab_size").unwrap(), 4);
        assert_eq!(handle.get_string("editor.indent_style").unwrap(), "space");
        assert_eq!(handle.get_string("editor.line_endings").unwrap(), "lf");
        assert_eq!(
            handle.get_bool("editor.trim_trailing_whitespace").unwrap(),
            false
        );
        assert_eq!(
            handle.get_bool("editor.insert_final_newline").unwrap(),
            true
        );
        assert_eq!(handle.get_string("logging.level").unwrap(), "info");
        assert_eq!(handle.get_string("logging.directory").unwrap(), "");
        assert_eq!(handle.get_int("logging.max_file_size_mb").unwrap(), 10);
        assert_eq!(handle.get_int("logging.max_retained_files").unwrap(), 5);
        assert_eq!(handle.get_string("theme.active").unwrap(), "default");
        assert_eq!(handle.get_int("theme.font_size").unwrap(), 14);
        assert_eq!(handle.get_string("vfs.default_provider").unwrap(), "local");
    }

    // ========================================================================
    // 21.2 — Initialization ordering: layer loading sequence
    // ========================================================================

    // Validates: Requirement 2.1 — init loads project config from project_root
    // Uses ReloadManager directly to avoid loading the real user config file from disk,
    // which may override schema defaults on the developer's machine (B029).
    #[test]
    fn init_loads_project_config_when_project_root_provided() {
        let dir = TempDir::new().unwrap();
        let project_root = dir.path().join("my-project");
        let ffworkbench_dir = project_root.join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();

        let mut schema = SchemaRegistry::new();
        register_core_schema(&mut schema);
        let mut manager = ReloadManager::new(Vec::new(), schema);
        manager.open_project(&project_root);
        let handle = ConfigHandle::new(manager);

        // Project config should override schema default
        assert_eq!(handle.get_int("editor.tab_size").unwrap(), 2);
        // Other schema defaults still apply (not overridden by real user config)
        assert_eq!(handle.get_string("logging.level").unwrap(), "info");
    }

    // Validates: Requirement 2.1 — init loads workspace config from workspace_root
    #[test]
    fn init_loads_workspace_config_when_workspace_root_provided() {
        let dir = TempDir::new().unwrap();
        let workspace_root = dir.path().join("my-workspace");
        let ffworkbench_dir = workspace_root.join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 8\n[logging]\nlevel = \"debug\"\n",
        )
        .unwrap();

        let options = ConfigInitOptions::new()
            .with_workspace_root(workspace_root)
            .with_hot_reload(false);

        let handle = init(options).unwrap();

        // Workspace config should override schema default
        assert_eq!(handle.get_int("editor.tab_size").unwrap(), 8);
        assert_eq!(handle.get_string("logging.level").unwrap(), "debug");
    }

    // Validates: Requirement 2.1 — workspace overrides project in layer precedence
    #[test]
    fn init_workspace_overrides_project_layer() {
        let dir = TempDir::new().unwrap();

        // Set up project config with tab_size = 2
        let project_root = dir.path().join("project");
        let project_ffwb = project_root.join(".ffworkbench");
        std::fs::create_dir_all(&project_ffwb).unwrap();
        std::fs::write(project_ffwb.join("config.toml"), "[editor]\ntab_size = 2\n").unwrap();

        // Set up workspace config with tab_size = 8
        let workspace_root = dir.path().join("workspace");
        let workspace_ffwb = workspace_root.join(".ffworkbench");
        std::fs::create_dir_all(&workspace_ffwb).unwrap();
        std::fs::write(
            workspace_ffwb.join("config.toml"),
            "[editor]\ntab_size = 8\n",
        )
        .unwrap();

        let options = ConfigInitOptions::new()
            .with_project_root(project_root)
            .with_workspace_root(workspace_root)
            .with_hot_reload(false);

        let handle = init(options).unwrap();

        // Workspace (priority 5) should override Project (priority 4)
        assert_eq!(handle.get_int("editor.tab_size").unwrap(), 8);
    }

    // ========================================================================
    // 21.5 — Graceful handling of missing layer files
    // ========================================================================

    // Validates: Requirement 1.1 — missing project config file is skipped silently
    #[test]
    fn init_skips_missing_project_config_gracefully() {
        let dir = TempDir::new().unwrap();
        // project_root exists but has no .ffworkbench/config.toml
        let project_root = dir.path().join("empty-project");
        std::fs::create_dir_all(&project_root).unwrap();

        let options = ConfigInitOptions::new()
            .with_project_root(project_root)
            .with_hot_reload(false);

        // Should succeed without error
        let handle = init(options).unwrap();

        // Schema defaults are still available
        assert_eq!(handle.get_int("editor.tab_size").unwrap(), 4);
    }

    // Validates: Requirement 1.1 — missing workspace config file is skipped silently
    #[test]
    fn init_skips_missing_workspace_config_gracefully() {
        let dir = TempDir::new().unwrap();
        let workspace_root = dir.path().join("empty-workspace");
        std::fs::create_dir_all(&workspace_root).unwrap();

        let options = ConfigInitOptions::new()
            .with_workspace_root(workspace_root)
            .with_hot_reload(false);

        let handle = init(options).unwrap();
        assert_eq!(handle.get_int("editor.tab_size").unwrap(), 4);
    }

    // Validates: Requirement 1.1 — partial layer availability loads what is available
    #[test]
    fn init_with_partial_layer_availability_loads_available_layers() {
        let dir = TempDir::new().unwrap();

        // Only project config exists, workspace doesn't
        let project_root = dir.path().join("project");
        let project_ffwb = project_root.join(".ffworkbench");
        std::fs::create_dir_all(&project_ffwb).unwrap();
        std::fs::write(
            project_ffwb.join("config.toml"),
            "[theme]\nactive = \"dark\"\n",
        )
        .unwrap();

        let workspace_root = dir.path().join("workspace");
        std::fs::create_dir_all(&workspace_root).unwrap();
        // No .ffworkbench/config.toml in workspace

        let options = ConfigInitOptions::new()
            .with_project_root(project_root)
            .with_workspace_root(workspace_root)
            .with_hot_reload(false);

        let handle = init(options).unwrap();

        // Project layer loaded successfully
        assert_eq!(handle.get_string("theme.active").unwrap(), "dark");
        // Schema defaults still available for other keys
        assert_eq!(handle.get_int("editor.tab_size").unwrap(), 4);
    }

    // ========================================================================
    // 21.4 — shutdown() cleanup
    // ========================================================================

    // Validates: Requirement 1 — shutdown stops file watcher and deregisters callbacks
    #[test]
    fn shutdown_stops_watcher_and_deregisters_callbacks() {
        let options = ConfigInitOptions::new().with_hot_reload(true);
        let handle = init(options).unwrap();

        // Register a callback
        let callbacks = handle.callbacks();
        let _cb_handle = callbacks.on_reload(&["editor.tab_size"], Box::new(|_event| {}));
        assert_eq!(callbacks.len(), 1);

        // Shutdown
        shutdown(&handle);

        // After shutdown, callbacks should be cleared
        assert_eq!(callbacks.len(), 0);

        // Handle is still usable for reads (data remains)
        assert_eq!(handle.get_int("editor.tab_size").unwrap(), 4);
    }

    // Validates: Requirement 1 — shutdown is safe when no watcher is active
    // Uses ReloadManager directly to avoid loading the real user config file from disk,
    // which may override schema defaults on the developer's machine (B029).
    #[test]
    fn shutdown_without_watcher_is_safe() {
        let mut schema = SchemaRegistry::new();
        register_core_schema(&mut schema);
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);

        // Should not panic even without a watcher
        shutdown(&handle);

        // Handle still works
        assert_eq!(handle.get_string("logging.level").unwrap(), "info");
    }

    // ========================================================================
    // 21.6 — register_core_schema populates all expected entries
    // ========================================================================

    // Validates: Requirement 9.1 — register_core_schema registers all well-known keys
    #[test]
    fn register_core_schema_registers_all_well_known_keys() {
        let mut schema = SchemaRegistry::new();
        register_core_schema(&mut schema);

        // Verify all expected keys are registered
        assert!(schema.get("editor.tab_size").is_some());
        assert!(schema.get("editor.indent_style").is_some());
        assert!(schema.get("editor.line_endings").is_some());
        assert!(schema.get("editor.trim_trailing_whitespace").is_some());
        assert!(schema.get("editor.insert_final_newline").is_some());
        assert!(schema.get("logging.level").is_some());
        assert!(schema.get("logging.directory").is_some());
        assert!(schema.get("logging.max_file_size_mb").is_some());
        assert!(schema.get("logging.max_retained_files").is_some());
        assert!(schema.get("theme.active").is_some());
        assert!(schema.get("theme.font_size").is_some());
        assert!(schema.get("vfs.default_provider").is_some());

        // Total: 12 core entries
        assert_eq!(schema.len(), 12);
    }

    // Validates: Requirement 9.1 — register_core_schema is idempotent (can be called twice)
    #[test]
    fn register_core_schema_is_idempotent() {
        let mut schema = SchemaRegistry::new();
        register_core_schema(&mut schema);
        let first_count = schema.len();

        // Call again — should not panic or produce conflicts
        register_core_schema(&mut schema);
        assert_eq!(schema.len(), first_count);
    }
}
