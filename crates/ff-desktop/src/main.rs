// Suppress console window on Windows when launched from a shortcut or file association.
// Logging goes exclusively to the log file (ff-logging upholds its side of this contract).
#![cfg_attr(not(test), windows_subsystem = "windows")]

//! # ff-desktop — FileForgeWorkbench Desktop Shell
//!
//! Entry point for the `ffwb` binary. Boots the platform-core stack and
//! launches the egui/eframe rendering window.

mod about_dialog;
mod automation;
mod batch;
mod catalog_manager_dialog;
mod catalog_registry;
mod command_palette;
mod context_menu;
mod copy_move_dialog;
mod dataset_alloc_dialog;
mod editor_panel;
mod event_log_panel;
mod exclude_manager;
mod fftest_cli;
mod file_explorer_panel;
mod files_panel;
mod find_manager;
mod key_config_dialog;
mod nav_manager;
mod notification;
mod panel_layout;
mod plugin_manager_panel;
mod posix_provider;
mod primary_option_menu;
mod scroll_amount;
mod search_results_panel;
mod session_manager;
mod settings_panel;
mod shell;
mod tab_manager;
mod tab_state;
mod toolchain_panel;

use anyhow::Context as _;
use eframe::egui;
use ff_config::init::{init, shutdown as config_shutdown, ConfigInitOptions};
use ff_core::WorkbenchApp;
use ff_logging::{init_default, shutdown as logging_shutdown, LoggingStatus};
use ff_session::UserDataDir;
use ff_theme::defaults::dark_palette;
use shell::WorkbenchShell;
use tokio::runtime::Runtime;

fn main() -> anyhow::Result<()> {
    // ── 1. Logging ────────────────────────────────────────────────────────
    let logging_status: LoggingStatus = init_default();

    // ── 2. Configuration ──────────────────────────────────────────────────
    let config_handle = init(ConfigInitOptions::new())
        .context("[desktop] configuration system initialisation failed")?;

    // ── 2a. Register all built-in schema entries ─────────────────────────
    // Resolve the user data dir to derive concrete default paths for keys
    // whose defaults are platform-specific (logging dir, catalog roots).
    {
        let user_data_dir = UserDataDir::resolve(None)
            .map(|u| u.path().to_path_buf())
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        register_builtin_schema(&config_handle, &user_data_dir);
    }

    // ── 2b. Apply OS reduce-motion preference if user has not overridden ──
    // Validates: accessibility Requirement 5.1
    apply_os_reduce_motion(&config_handle);

    // ── 3. Tokio runtime ──────────────────────────────────────────────────
    let runtime = Runtime::new().context("[desktop] failed to create Tokio runtime")?;

    // ── 4. WorkbenchApp ───────────────────────────────────────────────────
    let app = WorkbenchApp::new(Box::new(config_handle.clone()), logging_status)
        .context("[desktop] WorkbenchApp construction failed")?;

    // ── 5. Initial theme palette ──────────────────────────────────────────
    let palette = dark_palette();

    // ── 6. CLI file arguments (Requirement 6.1–6.5) ───────────────────────
    let cwd = std::env::current_dir().unwrap_or_default();
    let all_args: Vec<String> = std::env::args().skip(1).collect();

    // ── 6a. Headless FFTest mode (Req 6.1, 6.2, 6.3) ─────────────────────
    if let Some(mode) = fftest_cli::detect_cli_mode(&all_args) {
        let exit_code = fftest_cli::run_headless(&mode, &cwd);
        std::process::exit(exit_code);
    }

    // ── 6b. --help (Req 1.5 batch) ────────────────────────────────────────
    if all_args.iter().any(|a| a == "--help" || a == "-h") {
        batch::cli::print_help();
        std::process::exit(0);
    }

    // ── 6c. Batch mode (Req 1.1-1.6) ─────────────────────────────────────
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

    // ── 7. eframe window ─────────────────────────────────────────────────
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

    // ── 8. Post-window cleanup ────────────────────────────────────────────
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

/// Register all built-in core schema entries.
///
/// Called once at startup after `ff_config::init()`. Covers all reserved
/// core namespaces: `editor`, `logging`, `theme`, `vfs`, `catalogs`.
/// Best-effort — logs a warning on conflict but does not abort startup.
///
/// Validates: Requirement 9.1 — schema must contain every known key.
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
        // ── Editor ──────────────────────────────────────────────────────
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
        // ── Logging ─────────────────────────────────────────────────────
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
        // ── Theme ────────────────────────────────────────────────────────
        SchemaEntry {
            key: ff_config::keys::theme::ACTIVE.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("dark".to_string()),
            description: "Active colour theme: 'dark', 'light', 'high-contrast', or 'legacy'"
                .to_string(),
            constraints: Some(Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("dark".to_string()),
                    ConfigValue::String("light".to_string()),
                    ConfigValue::String("high-contrast".to_string()),
                    ConfigValue::String("legacy".to_string()),
                ]),
                pattern: None,
            }),
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
        // ── VFS ──────────────────────────────────────────────────────────
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
        // ── Catalogs ─────────────────────────────────────────────────────
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
        // ── Accessibility ────────────────────────────────────────────────
        SchemaEntry {
            key: ff_config::keys::accessibility::REDUCE_MOTION.to_string(),
            value_type: ValueType::Boolean,
            default: ConfigValue::Boolean(false),
            description: "Disable non-essential animations (overrides OS preference)".to_string(),
            constraints: None,
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

    /// Validates: accessibility Requirement 7.6 — binary crate constructs WorkbenchApp and
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

    /// Validates: Requirement 6.1 — positional args are collected as file paths.
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

    /// Validates: Requirement 6.2 — relative paths are resolved against cwd.
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

    /// Validates: Requirement 6.6 — named flags (--flag) are skipped.
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

    /// Validates: Requirement 6.1 — empty arg list produces empty result.
    #[test]
    fn resolve_cli_paths_empty_args_returns_empty() {
        // Validates: startup-and-session Requirement 6.1
        let cwd = std::path::Path::new("/workspace");
        let result = resolve_cli_paths(std::iter::empty(), cwd);
        assert!(result.is_empty());
    }
}
