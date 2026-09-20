// Suppress console window on Windows when launched from a shortcut or file association.
// Logging goes exclusively to the log file (ff-logging upholds its side of this contract).
#![cfg_attr(not(test), windows_subsystem = "windows")]

//! # ff-desktop -- FileForgeWorkbench Desktop Shell
//!
//! Entry point for the `ffwb` binary. Boots the platform-core stack and
//! launches the egui/eframe rendering window.

mod about_dialog;
mod automation;
mod batch;
mod catalog_manager_dialog;
mod catalog_registry;
mod command_config;
mod command_palette;
mod config_panel;
mod context_menu;
mod dataset_alloc_dialog;
mod editor_panel;
mod event_log_panel;
mod exclude_manager;
mod explorer_view;
mod fftest_cli;
mod files_panel;
mod find_manager;
mod keys_editor_panel;
mod kinds_editor_panel;
mod macro_library_panel;
mod menu_workspace;
mod menus_editor_panel;
mod nav_manager;
mod nav_model;
mod notification;
mod panel_layout;
mod plugin_manager_panel;
mod posix_provider;
mod primary_option_menu;
mod scroll_amount;
mod search_results_panel;
mod session_manager;
mod shell;
mod tab_manager;
mod tab_state;
mod theme_defaults;
mod theme_editor_panel;
mod toolchain_panel;
mod workspace_kind;

use anyhow::Context as _;
use eframe::egui;
use ff_config::init::{init, shutdown as config_shutdown, ConfigInitOptions};
use ff_core::WorkbenchApp;
use ff_logging::{
    init_default, reconfigure as logging_reconfigure, shutdown as logging_shutdown, LogConfig,
    LogLevel, LoggingStatus,
};
use ff_session::UserDataDir;
use shell::WorkbenchShell;
use tokio::runtime::Runtime;

fn main() -> anyhow::Result<()> {
    // == 0. Application Profile (CR-NR-081, startup-and-session Req 22) =====
    // Extract `--profile <name>` / `-p <name>` and set the process-global active
    // profile BEFORE anything resolves the User_Data_Dir or loads configuration
    // (Req 22.5), so every subsystem is isolated to the selected profile. The
    // flag + its value are REMOVED from the argument list here so they are never
    // treated as a file to open (Req 22.7). An absent flag -> DEFAULT_PROFILE
    // (today's location, no behaviour change, Req 22.2).
    let mut cli_args: Vec<String> = std::env::args().skip(1).collect();
    let active_profile = extract_profile_arg(&mut cli_args);
    // Set the active profile on BOTH layers before any resolve/config init:
    // ff-session (User_Data_Dir: themes/menus/keymaps/session/catalogs/logs) and
    // ff-config (user config.toml). ff-config keeps its own mirror because it
    // cannot depend on ff-session (layering). Both slug identically so they
    // resolve the SAME profiles/<slug>/ directory (Req 22.3).
    ff_session::set_active_profile(active_profile.as_deref());
    ff_config::paths::set_active_profile(active_profile.as_deref());

    // == 1. Logging (Phase 1: defaults) ====================================
    // Initialize logging FIRST with platform defaults so no diagnostic record
    // is lost while the configuration system loads. The configured directory
    // and level are applied in step 2c once config is available (Req 11.7).
    let mut logging_status: LoggingStatus = init_default();

    // == 2. Configuration ==================================================
    let config_handle = init(ConfigInitOptions::new())
        .context("[desktop] configuration system initialisation failed")?;

    // == 2a. Register all built-in schema entries =========================
    // Resolve the user data dir to derive concrete default paths for keys
    // whose defaults are platform-specific (logging dir, catalog roots).
    {
        let user_data_dir = UserDataDir::resolve(None)
            .map(|u| u.path().to_path_buf())
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        register_builtin_schema(&config_handle, &user_data_dir);
    }

    // == 2c. Logging (Phase 2: apply configured settings) =================
    // Re-point the log sink at the configured directory/level now that config
    // is loaded, before the GUI shell is constructed (Req 11.7). Config-only
    // log redirection works from here with no recompile.
    logging_status = apply_logging_config(&config_handle, logging_status);

    // == 2b. Apply OS reduce-motion preference if user has not overridden ==
    // Validates: accessibility Requirement 5.1
    apply_os_reduce_motion(&config_handle);

    // == 3. Tokio runtime ==================================================
    let runtime = Runtime::new().context("[desktop] failed to create Tokio runtime")?;

    // == 4. WorkbenchApp ===================================================
    let app = WorkbenchApp::new(Box::new(config_handle.clone()), logging_status)
        .context("[desktop] WorkbenchApp construction failed")?;

    // == 5. Initial theme palette (file-backed) ============================
    // CR-NR-074 Req 19.1/19.2/19.4: materialise built-in theme files on first
    // launch, then resolve the active theme from config and load it BEFORE the
    // first frame so all rendering is driven by the theme file (fallback to the
    // Default Legacy built-in when the active theme cannot be resolved).
    let themes_dir = theme_defaults::themes_dir();
    if let Ok(mut udd) = ff_session::UserDataDir::resolve(None) {
        let _ = udd.initialise();
        theme_defaults::ensure_default_theme_files(udd.path());
    }
    let palette = theme_defaults::resolve_startup_palette(&config_handle, &themes_dir);

    // == 6. CLI file arguments (Requirement 6.1-6.5) =======================
    // `cli_args` was collected in step 0 with the `--profile`/`-p` flag + value
    // already removed (Req 22.7), so file-path resolution never sees them.
    let cwd = std::env::current_dir().unwrap_or_default();
    let all_args: Vec<String> = cli_args;

    // == 6a. Headless FFTest mode (Req 6.1, 6.2, 6.3) =====================
    if let Some(mode) = fftest_cli::detect_cli_mode(&all_args) {
        let exit_code = fftest_cli::run_headless(&mode, &cwd);
        std::process::exit(exit_code);
    }

    // == 6b. --help (Req 1.5 batch) ========================================
    if all_args.iter().any(|a| a == "--help" || a == "-h") {
        batch::cli::print_help();
        std::process::exit(0);
    }

    // == 6c. Batch mode (Req 1.1-1.6) =====================================
    match batch::cli::parse_batch_args(&all_args) {
        Err(msg) => {
            eprintln!("ffwb: {}", msg);
            std::process::exit(12);
        }
        Ok(Some(batch_args)) => {
            let rc = batch::run_batch(batch_args);
            std::process::exit(rc);
        }
        Ok(None) => {} // not batch mode -- continue to GUI
    }

    let cli_files = resolve_cli_paths(all_args.into_iter(), &cwd);

    // == 7. eframe window =================================================
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("FileForge Workbench")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 480.0])
            .with_resizable(true)
            .with_drag_and_drop(false),
        ..Default::default()
    };

    // Clone before move into closure so config_shutdown can use the original.
    let config_handle_for_shell = config_handle.clone();
    eframe::run_native(
        "FileForge Workbench",
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(WorkbenchShell::new(
                app,
                runtime,
                palette,
                cli_files,
                config_handle_for_shell,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!("[desktop] eframe error: {e}"))?;

    // == 8. Post-window cleanup ============================================
    config_shutdown(&config_handle);
    logging_shutdown();

    Ok(())
}

/// Query the OS reduce-motion preference and apply it to the config if the
/// user has not already set `accessibility.reduce_motion` explicitly.
///
/// - Windows: `SystemParametersInfo(SPI_GETCLIENTAREAANIMATION)`
/// - macOS/Linux: not yet implemented (returns false, no-op)
///
/// Validates: accessibility Requirement 5.1
fn apply_os_reduce_motion(config: &ff_config::ConfigHandle) {
    // Only apply if the user has not explicitly set the key.
    if config
        .get_bool(ff_config::keys::accessibility::REDUCE_MOTION)
        .unwrap_or(false)
    {
        return; // user already opted in -- respect their choice
    }

    if os_prefers_reduce_motion() {
        let _ = config.set_user_value(
            ff_config::keys::accessibility::REDUCE_MOTION,
            ff_config::ConfigValue::Boolean(true),
        );
    }
}

/// Return true if the host OS reports a reduce-motion preference.
///
/// Validates: accessibility Requirement 5.1
fn os_prefers_reduce_motion() -> bool {
    #[cfg(target_os = "windows")]
    {
        // SPI_GETCLIENTAREAANIMATION = 0x1042
        // Returns TRUE when animations are enabled, FALSE when the user has
        // turned them off (i.e. reduce-motion is active).
        use std::ffi::c_int;
        extern "system" {
            fn SystemParametersInfoW(
                ui_action: u32,
                ui_param: u32,
                pv_param: *mut std::ffi::c_void,
                f_win_ini: u32,
            ) -> c_int;
        }
        let mut animations_enabled: u32 = 1;
        let ok = unsafe {
            SystemParametersInfoW(
                0x1042, // SPI_GETCLIENTAREAANIMATION
                0,
                &mut animations_enabled as *mut u32 as *mut std::ffi::c_void,
                0,
            )
        };
        // ok != 0 means the call succeeded; animations_enabled == 0 means reduce-motion.
        ok != 0 && animations_enabled == 0
    }
    #[cfg(not(target_os = "windows"))]
    {
        false // macOS/Linux detection deferred
    }
}

/// Phase 2 of two-phase logging init: apply the resolved `logging.*` settings
/// to the already-initialized logging subsystem.
///
/// Reads the effective `logging.level`, `logging.directory`,
/// `logging.max_file_size_mb`, and `logging.max_retained_files` from the loaded
/// configuration (any layer -- system, user, project `.ffworkbench/config.toml`,
/// etc.) and calls `ff_logging::reconfigure` so log output is redirected to the
/// configured directory with no recompile.
///
/// An empty `logging.directory` means "keep the platform default already in
/// effect" -- in that case the resolved default directory registered in the
/// schema is used, which equals the current directory, so no file switch occurs.
///
/// Returns the resulting `LoggingStatus` so the caller can keep the status-bar
/// fallback indicator accurate. If the subsystem is in fallback (no-op) mode,
/// the current status is returned unchanged.
///
/// Validates: logging-subsystem Requirement 11 (AC 11.7), Requirement 4 (AC 4.2).
fn apply_logging_config(config: &ff_config::ConfigHandle, current: LoggingStatus) -> LoggingStatus {
    // Level: parse leniently; fall back to Info on anything unexpected.
    let mut log_config = LogConfig {
        level: LogLevel::Info,
        directory: std::path::PathBuf::new(),
        max_file_size_mb: 10,
        max_retained_files: 5,
    };

    if let Ok(level_str) = config.get_string(ff_config::keys::logging::LEVEL) {
        // set_level_from_str applies the value or defaults to Info with a warning.
        let _ = log_config.set_level_from_str(&level_str);
    }

    if let Ok(dir) = config.get_string(ff_config::keys::logging::DIRECTORY) {
        log_config.directory = std::path::PathBuf::from(dir);
    }

    if let Ok(size) = config.get_int(ff_config::keys::logging::MAX_FILE_SIZE_MB) {
        // Clamp defensively into u32 range; reconfigure clamps to 1..=1024 too.
        log_config.max_file_size_mb = size.clamp(1, u32::MAX as i64) as u32;
    }

    if let Ok(retained) = config.get_int(ff_config::keys::logging::MAX_RETAINED_FILES) {
        log_config.max_retained_files = retained.clamp(1, u32::MAX as i64) as u32;
    }

    // If the configured directory resolves to empty, keep the platform default
    // that Phase 1 already opened: reconfigure with the schema default path
    // (which the config resolves to) rather than an empty path.
    if log_config.directory.as_os_str().is_empty() {
        // Nothing to redirect; only level/rotation may change. Leave the
        // directory as the currently active platform default by not switching.
        // reconfigure treats an empty directory the same as the current dir
        // only if it matches; to be safe, skip the directory change entirely
        // by reusing the current-status short-circuit below.
        return current;
    }

    logging_reconfigure(log_config)
}

/// Register all built-in core schema entries.
///
/// Called once at startup after `ff_config::init()`. Covers all reserved
/// core namespaces: `editor`, `logging`, `theme`, `vfs`, `catalogs`.
/// Best-effort -- logs a warning on conflict but does not abort startup.
///
/// Validates: Requirement 9.1 -- schema must contain every known key.
fn register_builtin_schema(config: &ff_config::ConfigHandle, user_data_dir: &std::path::Path) {
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

/// Extract the Application_Profile name from the argument list, REMOVING the
/// `--profile`/`-p` flag AND its value in place (CR-NR-081, startup-and-session
/// Requirement 22).
///
/// Returns `Some(name)` when a non-empty profile name follows the flag, else
/// `None` (the DEFAULT_PROFILE). WHEN the flag is present but its value is
/// missing or blank, the flag is removed, a WARN is logged, and `None` is
/// returned (Requirement 22.6). Both `--profile foo` and `-p foo` forms are
/// accepted; the flag and its value are stripped so they never reach the
/// positional file-path resolver (Requirement 22.7).
///
/// Validates: startup-and-session Requirement 22.1, 22.6, 22.7
pub fn extract_profile_arg(args: &mut Vec<String>) -> Option<String> {
    let pos = args.iter().position(|a| a == "--profile" || a == "-p")?;
    // Remove the flag itself.
    args.remove(pos);
    // The value (if any) is now at `pos`. A missing value, or a value that looks
    // like another flag, is treated as absent (DEFAULT_PROFILE + WARN, Req 22.6).
    let value = match args.get(pos) {
        Some(v) if !v.starts_with('-') => {
            let v = v.clone();
            args.remove(pos);
            v
        }
        _ => String::new(),
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        ff_logging::log_warn!(
            "[desktop] --profile/-p given with no value; using the default profile"
        );
        return None;
    }
    Some(trimmed.to_string())
}

/// Collect positional CLI arguments as absolute file paths.
///
/// - Skips named flags (anything starting with `--` or `-`).
/// - Resolves relative paths against `cwd`.
/// - Absolute paths are kept as-is.
///
/// Addresses: Requirement 6.1, 6.2
pub fn resolve_cli_paths(args: impl Iterator<Item = String>, cwd: &std::path::Path) -> Vec<String> {
    args.filter(|a| !a.starts_with('-'))
        .map(|a| {
            let p = std::path::Path::new(&a);
            if p.is_absolute() {
                a
            } else {
                cwd.join(p).to_string_lossy().into_owned()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_config::init::{init, ConfigInitOptions};
    use ff_core::WorkbenchApp;
    use ff_logging::LoggingStatus;
    use tokio::runtime::Runtime;

    /// Validates: accessibility Requirement 5.1 -- OS reduce-motion preference is read at startup.
    /// On Windows, SPI_GETCLIENTAREAANIMATION is queried; result is applied to config.
    #[test]
    fn reduce_motion_config_key_is_registered_in_schema() {
        // Validates: accessibility Requirement 5.1, 5.2
        // Use a temp dir as project root so we get a clean config with no user overrides.
        use tempfile::TempDir;
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
        use tempfile::TempDir;
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

    /// Validates: accessibility Requirement 7.6 -- binary crate constructs WorkbenchApp and
    /// boots/shuts down cleanly without a GUI window.
    #[test]
    fn workbench_app_boots_and_shuts_down_cleanly() {
        let config_handle = init(ConfigInitOptions::new().with_hot_reload(false))
            .expect("config init must succeed");

        let runtime = Runtime::new().expect("runtime must be created");

        let mut app = WorkbenchApp::new(Box::new(config_handle.clone()), LoggingStatus::Fallback)
            .expect("WorkbenchApp construction must succeed");

        runtime
            .block_on(app.startup())
            .expect("startup must succeed");

        use ff_core::LifecyclePhase;
        assert_eq!(app.phase(), LifecyclePhase::Running);

        runtime.block_on(app.shutdown());
        assert_eq!(app.phase(), LifecyclePhase::Terminated);
    }

    /// Validates: Requirement 6.1 -- positional args are collected as file paths.
    #[test]
    fn resolve_cli_paths_collects_positional_args() {
        // Validates: startup-and-session Requirement 6.1
        let cwd = std::path::Path::new("/workspace");
        let args = vec![
            "/absolute/file.txt".to_string(),
            "relative/file.rs".to_string(),
        ];
        let result = resolve_cli_paths(args.into_iter(), cwd);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "/absolute/file.txt");
        assert!(result[1].contains("relative/file.rs"));
    }

    /// Validates: Requirement 6.2 -- relative paths are resolved against cwd.
    #[test]
    fn resolve_cli_paths_resolves_relative_against_cwd() {
        // Validates: startup-and-session Requirement 6.2
        let cwd = std::path::Path::new("/home/user/projects");
        let args = vec!["src/main.rs".to_string()];
        let result = resolve_cli_paths(args.into_iter(), cwd);
        assert_eq!(result.len(), 1);
        assert!(
            result[0].contains("src") && result[0].contains("main.rs"),
            "resolved path must contain both path components"
        );
        assert!(
            result[0].starts_with("/home/user/projects"),
            "resolved path must be rooted at cwd"
        );
    }

    /// Validates: Requirement 6.6 -- named flags (--flag) are skipped.
    #[test]
    fn resolve_cli_paths_skips_named_flags() {
        // Validates: startup-and-session Requirement 6.6
        let cwd = std::path::Path::new("/workspace");
        let args = vec![
            "--no-session-restore".to_string(),
            "--profile".to_string(),
            "default".to_string(),
            "file.txt".to_string(),
        ];
        let result = resolve_cli_paths(args.into_iter(), cwd);
        // --no-session-restore and --profile are skipped; "default" and "file.txt" are kept
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|p| p.contains("file.txt")));
    }

    // Validates: startup-and-session Req 22.1, 22.7 (CR-NR-081) -- `--profile
    // <name>` is extracted and REMOVED (flag + value) from the arg list.
    #[test]
    fn extract_profile_arg_long_form_extracts_and_removes() {
        let mut args = vec![
            "--profile".to_string(),
            "ispf".to_string(),
            "file.txt".to_string(),
        ];
        let profile = extract_profile_arg(&mut args);
        assert_eq!(profile.as_deref(), Some("ispf"));
        assert_eq!(args, vec!["file.txt".to_string()], "flag + value removed");
    }

    // Validates: startup-and-session Req 22.1, 22.7 -- the short `-p` form works.
    #[test]
    fn extract_profile_arg_short_form_extracts_and_removes() {
        let mut args = vec!["-p".to_string(), "rust".to_string(), "file.txt".to_string()];
        let profile = extract_profile_arg(&mut args);
        assert_eq!(profile.as_deref(), Some("rust"));
        assert_eq!(args, vec!["file.txt".to_string()]);
    }

    // Validates: startup-and-session Req 22.7 -- `ffwb -p rust file.txt` still
    // opens file.txt (the file arg survives profile extraction + path resolve).
    #[test]
    fn extract_profile_then_resolve_paths_keeps_file_arg() {
        let mut args = vec!["-p".to_string(), "rust".to_string(), "file.txt".to_string()];
        let _ = extract_profile_arg(&mut args);
        let cwd = std::path::Path::new("/workspace");
        let files = resolve_cli_paths(args.into_iter(), cwd);
        assert_eq!(files.len(), 1);
        assert!(files[0].contains("file.txt"));
    }

    // Validates: startup-and-session Req 22.2 -- no `--profile` -> None (default).
    #[test]
    fn extract_profile_arg_absent_returns_none() {
        let mut args = vec!["file.txt".to_string()];
        assert_eq!(extract_profile_arg(&mut args), None);
        assert_eq!(args, vec!["file.txt".to_string()], "args unchanged");
    }

    // Validates: startup-and-session Req 22.6 -- flag with a missing/flag-like
    // value -> None (DEFAULT_PROFILE); the flag is still removed.
    #[test]
    fn extract_profile_arg_missing_value_returns_none_and_removes_flag() {
        // Value missing entirely (flag is last).
        let mut args = vec!["--profile".to_string()];
        assert_eq!(extract_profile_arg(&mut args), None);
        assert!(args.is_empty(), "the flag is removed even with no value");
        // Value looks like another flag -> treated as absent.
        let mut args2 = vec!["-p".to_string(), "--other".to_string()];
        assert_eq!(extract_profile_arg(&mut args2), None);
        assert_eq!(
            args2,
            vec!["--other".to_string()],
            "only the -p flag removed"
        );
    }

    /// Validates: Requirement 6.1 -- empty arg list produces empty result.
    #[test]
    fn resolve_cli_paths_empty_args_returns_empty() {
        // Validates: startup-and-session Requirement 6.1
        let cwd = std::path::Path::new("/workspace");
        let result = resolve_cli_paths(std::iter::empty(), cwd);
        assert!(result.is_empty());
    }

    /// Validates: logging-subsystem Requirement 11.7 / Requirement 4.2 --
    /// a `logging.directory` set in a project-layer `.ffworkbench/config.toml`
    /// is surfaced by the configuration system, which is exactly the value the
    /// two-phase startup feeds into `ff_logging::reconfigure`. This proves the
    /// config-only redirect path (no recompile) end-to-end from the config side.
    #[test]
    fn project_config_logging_directory_is_resolved_for_reconfigure() {
        use std::fs;
        use tempfile::TempDir;

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

    /// Validates: logging-subsystem Requirement 11.6 -- `apply_logging_config`
    /// is safe to call regardless of logging subsystem state. In this test
    /// binary the global subsystem is not file-active, so reconfigure is a
    /// no-op returning Fallback; the call must not panic and must return a status.
    #[test]
    fn apply_logging_config_is_safe_when_logging_not_active() {
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("tempdir");
        let config = init(
            ConfigInitOptions::new()
                .with_hot_reload(false)
                .with_project_root(tmp.path().to_path_buf()),
        )
        .expect("config init");
        register_builtin_schema(&config, tmp.path());

        // Should not panic. Returns whatever the subsystem state allows.
        let status = apply_logging_config(&config, LoggingStatus::Fallback);
        let _ = status; // status value depends on global state; correctness = no panic
    }
}
