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
mod help_context;
mod keys_editor_panel;
mod kinds_editor_panel;
mod macro_library_panel;
mod menu_workspace;
mod menus_editor_panel;
mod nav_manager;
mod nav_model;
mod notification;
mod plugin_manager_panel;
mod posix_provider;
mod primary_option_menu;
mod screen_snapshot;
mod scrm_session;
mod scrm_viewer_panel;
mod scroll_amount;
mod search_results_panel;
mod session_manager;
mod shell;
mod startup_env;
mod startup_schema;
mod tab_manager;
mod tab_state;
mod tab_state_ctors;
mod theme_defaults;
mod theme_editor_panel;
mod toolchain_panel;
mod workspace_kind;

use anyhow::Context as _;
use eframe::egui;
use ff_config::init::{init, shutdown as config_shutdown, ConfigInitOptions};
use ff_core::WorkbenchApp;
use ff_logging::{init_default, shutdown as logging_shutdown, LoggingStatus};
use ff_session::UserDataDir;
use shell::WorkbenchShell;
use startup_env::{apply_logging_config, apply_os_reduce_motion};
use startup_schema::register_builtin_schema;
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
}
