//! # Built-in configuration schema registration (startup).
//!
//! Split out of `main.rs` (400-line rule, `rust-standards.md`). Holds
//! `register_builtin_schema`, the single startup step that registers every
//! reserved core namespace entry with the configuration system. The crate root
//! re-exports it (`use startup_schema::register_builtin_schema;`) so both
//! `main()` and `crate::register_builtin_schema` callers resolve unchanged.

/// Register all built-in core schema entries.
///
/// Called once at startup after `ff_config::init()`. Covers all reserved
/// core namespaces: `editor`, `logging`, `theme`, `vfs`, `catalogs`.
/// Best-effort -- logs a warning on conflict but does not abort startup.
///
/// Validates: Requirement 9.1 -- schema must contain every known key.
pub(crate) fn register_builtin_schema(
    config: &ff_config::ConfigHandle,
    user_data_dir: &std::path::Path,
) {
    use ff_config::error::ValueType;
    use ff_config::schema::{Constraints, SchemaEntry};
    use ff_config::value::ConfigValue;

    let log_dir = user_data_dir.join("logs").to_string_lossy().into_owned();
    let mainframe_root = user_data_dir
        .join("catalogs")
        .join("mainframe")
        .to_string_lossy()
        .into_owned();
    let posix_root = user_data_dir
        .join("catalogs")
        .join("posix")
        .to_string_lossy()
        .into_owned();

    let entries: &[SchemaEntry] = &[
        // == Editor ======================================================
        SchemaEntry {
            key: ff_config::keys::editor::TAB_SIZE.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(4),
            description: "Number of spaces per tab stop".to_string(),
            constraints: Some(Constraints {
                min: Some(1.0),
                max: Some(16.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        SchemaEntry {
            key: ff_config::keys::editor::INDENT_STYLE.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("space".to_string()),
            description: "Indentation style: 'space' or 'tab'".to_string(),
            constraints: Some(Constraints {
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
            key: ff_config::keys::editor::LINE_ENDINGS.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("lf".to_string()),
            description: "Default line ending style: 'lf', 'crlf', or 'cr'".to_string(),
            constraints: Some(Constraints {
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
            key: ff_config::keys::editor::TRIM_TRAILING_WHITESPACE.to_string(),
            value_type: ValueType::Boolean,
            default: ConfigValue::Boolean(false),
            description: "Remove trailing whitespace on save".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: ff_config::keys::editor::INSERT_FINAL_NEWLINE.to_string(),
            value_type: ValueType::Boolean,
            default: ConfigValue::Boolean(true),
            description: "Ensure file ends with a newline on save".to_string(),
            constraints: None,
        },
        // == Logging =====================================================
        SchemaEntry {
            key: ff_config::keys::logging::LEVEL.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("info".to_string()),
            description: "Minimum log level: 'debug', 'info', 'warn', or 'error'".to_string(),
            constraints: Some(Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("debug".to_string()),
                    ConfigValue::String("info".to_string()),
                    ConfigValue::String("warn".to_string()),
                    ConfigValue::String("error".to_string()),
                ]),
                pattern: None,
            }),
        },
        SchemaEntry {
            key: ff_config::keys::logging::DIRECTORY.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String(log_dir),
            description: "Directory where log files are written".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: ff_config::keys::logging::MAX_FILE_SIZE_MB.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(10),
            description: "Maximum log file size in megabytes before rotation".to_string(),
            constraints: Some(Constraints {
                min: Some(1.0),
                max: Some(500.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        SchemaEntry {
            key: ff_config::keys::logging::MAX_RETAINED_FILES.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(5),
            description: "Number of rotated log files to keep".to_string(),
            constraints: Some(Constraints {
                min: Some(1.0),
                max: Some(50.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        // == Theme ========================================================
        SchemaEntry {
            key: ff_config::keys::theme::ACTIVE.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("dark".to_string()),
            // The allowed values MUST match `VisualMode::section_name()` exactly,
            // because `set_theme()` persists `section_name()` into this key and
            // config validation rejects any value not in this set (substituting
            // the default). `section_name()` uses the underscore spelling
            // `high_contrast`; using the hyphen form here caused High Contrast to
            // be rejected on reload and revert to `dark` (B039).
            description: "Active colour theme: 'dark', 'light', 'high_contrast', or 'legacy'"
                .to_string(),
            constraints: Some(Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("dark".to_string()),
                    ConfigValue::String("light".to_string()),
                    ConfigValue::String("high_contrast".to_string()),
                    ConfigValue::String("legacy".to_string()),
                ]),
                pattern: None,
            }),
        },
        SchemaEntry {
            // Active theme NAME (theme file / built-in). Empty => fall back to the
            // mode in `theme.active`. Free-form (a theme name), so no allowed_values.
            // Validates: theme-and-appearance Requirement 19.3.
            key: ff_config::keys::theme::ACTIVE_NAME.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String(String::new()),
            description: "Active theme name (resolves to themes/<slug>.toml or a built-in); empty uses theme.active mode".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: ff_config::keys::theme::FONT_SIZE.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(14),
            description: "Editor font size in points".to_string(),
            constraints: Some(Constraints {
                min: Some(8.0),
                max: Some(72.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        // == VFS ==========================================================
        SchemaEntry {
            key: ff_config::keys::vfs::DEFAULT_PROVIDER.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("local".to_string()),
            description: "Default VFS provider used when opening files".to_string(),
            constraints: Some(Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![ConfigValue::String("local".to_string())]),
                pattern: None,
            }),
        },
        // == Catalogs =====================================================
        SchemaEntry {
            key: ff_config::keys::catalogs::DEFAULT_MAINFRAME_ROOT.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String(mainframe_root),
            description: "Default repository root for new Mainframe catalogs".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: ff_config::keys::catalogs::DEFAULT_POSIX_ROOT.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String(posix_root),
            description: "Default root directory for new POSIX catalogs".to_string(),
            constraints: None,
        },
        // == Accessibility ================================================
        SchemaEntry {
            key: ff_config::keys::accessibility::REDUCE_MOTION.to_string(),
            value_type: ValueType::Boolean,
            default: ConfigValue::Boolean(false),
            description: "Disable non-essential animations (overrides OS preference)".to_string(),
            constraints: None,
        },
        // == Menu Workspace ===============================================
        // Validates: menu-workspace Requirement 9.1, 9.6
        SchemaEntry {
            key: ff_config::keys::menu::SOFT_OPTION_LIMIT.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(64),
            description: "Advisory maximum options per menu; above it a warning \
                          advises grouping into sub-menus"
                .to_string(),
            constraints: Some(Constraints {
                min: Some(0.0),
                max: None,
                allowed_values: None,
                pattern: None,
            }),
        },
        SchemaEntry {
            key: ff_config::keys::menu::HARD_OPTION_LIMIT.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(256),
            description: "Hard maximum options per menu; above it the menu file \
                          is rejected as a load error"
                .to_string(),
            constraints: Some(Constraints {
                min: Some(0.0),
                max: None,
                allowed_values: None,
                pattern: None,
            }),
        },
    ];

    for entry in entries {
        if let Err(e) = config.register_schema_entry(entry.clone()) {
            ff_logging::log_warn!(
                "[desktop] schema registration failed for '{}': {e}",
                entry.key
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_config::init::{init, ConfigInitOptions};
    use tempfile::TempDir;

    /// Validates: accessibility Requirement 5.1 -- OS reduce-motion preference is read at startup.
    /// On Windows, SPI_GETCLIENTAREAANIMATION is queried; result is applied to config.
    #[test]
    fn reduce_motion_config_key_is_registered_in_schema() {
        // Validates: accessibility Requirement 5.1, 5.2
        // Use a temp dir as project root so we get a clean config with no user overrides.
        let tmp = TempDir::new().expect("tempdir");
        let config = init(
            ConfigInitOptions::new()
                .with_hot_reload(false)
                .with_project_root(tmp.path().to_path_buf()),
        )
        .expect("config init");
        register_builtin_schema(&config, tmp.path());
        // The key must be registered -- get_bool must not return KeyNotFound.
        // (The actual value may be true if the OS has reduce-motion enabled.)
        let result = config.get_bool(ff_config::keys::accessibility::REDUCE_MOTION);
        assert!(
            result.is_ok(),
            "accessibility.reduce_motion must be registered in schema, got: {:?}",
            result
        );
    }

    /// Validates: menu-workspace Requirement 9.1, 9.6 -- menu option limit keys are
    /// registered with their default values (64 / 256) and readable as integers.
    #[test]
    fn menu_limit_keys_have_correct_defaults() {
        let tmp = TempDir::new().expect("tempdir");
        let config = init(
            ConfigInitOptions::new()
                .with_hot_reload(false)
                .with_project_root(tmp.path().to_path_buf()),
        )
        .expect("config init");
        register_builtin_schema(&config, tmp.path());

        let soft = config.get_int(ff_config::keys::menu::SOFT_OPTION_LIMIT);
        let hard = config.get_int(ff_config::keys::menu::HARD_OPTION_LIMIT);
        assert_eq!(
            soft.ok(),
            Some(64),
            "menu.soft_option_limit default must be 64"
        );
        assert_eq!(
            hard.ok(),
            Some(256),
            "menu.hard_option_limit default must be 256"
        );
    }

    /// Validates: logging-subsystem Requirement 11.7 / Requirement 4.2 --
    /// a `logging.directory` set in a project-layer `.ffworkbench/config.toml`
    /// is surfaced by the configuration system, which is exactly the value the
    /// two-phase startup feeds into `ff_logging::reconfigure`. This proves the
    /// config-only redirect path (no recompile) end-to-end from the config side.
    #[test]
    fn project_config_logging_directory_is_resolved_for_reconfigure() {
        use std::fs;

        let tmp = TempDir::new().expect("tempdir");
        let custom_log_dir = tmp.path().join("my_project_logs");

        // Write a project-layer config that redirects logs into the project.
        let ffwb = tmp.path().join(".ffworkbench");
        fs::create_dir_all(&ffwb).expect("create .ffworkbench");
        let toml = format!(
            "[logging]\ndirectory = {:?}\nlevel = \"debug\"\n",
            custom_log_dir.to_string_lossy()
        );
        fs::write(ffwb.join("config.toml"), toml).expect("write project config");

        let config = init(
            ConfigInitOptions::new()
                .with_hot_reload(false)
                .with_project_root(tmp.path().to_path_buf()),
        )
        .expect("config init");
        register_builtin_schema(&config, tmp.path());

        // The resolved logging.directory must be the project-configured path,
        // overriding the schema default -- this is the value apply_logging_config
        // hands to reconfigure().
        let resolved = config
            .get_string(ff_config::keys::logging::DIRECTORY)
            .expect("logging.directory must resolve");
        assert_eq!(
            std::path::PathBuf::from(&resolved),
            custom_log_dir,
            "project-layer logging.directory must override the schema default"
        );

        // And the level override is likewise surfaced.
        let level = config
            .get_string(ff_config::keys::logging::LEVEL)
            .expect("logging.level must resolve");
        assert_eq!(level, "debug");
    }
}
