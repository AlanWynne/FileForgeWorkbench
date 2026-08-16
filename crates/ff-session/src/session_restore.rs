//! Session restore decision logic — determines what files and state
//! to restore based on configuration, CLI arguments, and session validity.
//!
//! Addresses: Requirement 5 (Session Restore on Launch), Requirement 7 (Empty Startup State)

use crate::cli::CliArgs;
use crate::config_keys::SessionConfig;
use crate::session_state::{SessionState, TabState};

/// The restore mode determined by the decision matrix.
///
/// Addresses: Requirement 5 AC 5.1–5.9
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreMode {
    /// Open specific files from CLI arguments.
    /// Layout and geometry are still restored from session.
    CliArgs,

    /// Open the configured startup file.
    /// Layout and geometry are still restored from session.
    StartupFile,

    /// Full session restore — reopen all tabs with per-tab state,
    /// restore layout and geometry.
    FullRestore,

    /// Layout and geometry only — restore layout but don't reopen tabs.
    LayoutOnly,

    /// Empty startup state — default layout, no files opened.
    /// Geometry may still be restored if save_window_geometry is true.
    Empty,
}

/// Describes what files should be opened after Phase 8.
///
/// Addresses: Requirement 5, Requirement 6
#[derive(Debug, Clone, PartialEq)]
pub enum FileOpenTargets {
    /// Open specific files from CLI arguments.
    CliArgs(Vec<String>),
    /// Open the configured startup file.
    StartupFile(String),
    /// Restore tabs from session state.
    SessionRestore(Vec<TabState>),
    /// No files to open — show empty/welcome state.
    Empty,
}

/// Determine the restore mode based on configuration, CLI args, and session state.
///
/// Decision matrix (evaluated in priority order):
/// 1. CLI args present → RestoreMode::CliArgs (layout/geometry still restored)
/// 2. startup_file set (no CLI args) → RestoreMode::StartupFile
/// 3. restore_on_startup=false → RestoreMode::Empty (geometry if save_window_geometry)
/// 4. restore_on_startup=true + restore_tabs=true + valid session → FullRestore
/// 5. restore_on_startup=true + restore_tabs=false → LayoutOnly
/// 6. No valid session → Empty
///
/// Addresses: Requirement 5 AC 5.1, 5.3, 5.6, 5.7; Requirement 6 AC 6.4, 6.8, 6.9
pub fn determine_restore_mode(
    config: &SessionConfig,
    cli_args: &CliArgs,
    session_state: &SessionState,
) -> RestoreMode {
    // Priority 1: CLI args override everything for file opening
    if cli_args.has_source_args() {
        return RestoreMode::CliArgs;
    }

    // --no-session-restore CLI flag overrides config
    if cli_args.no_session_restore {
        return RestoreMode::Empty;
    }

    // Priority 2: startup_file overrides session tab restore
    if config.startup_file.is_some() {
        return RestoreMode::StartupFile;
    }

    // Priority 3: restore_on_startup=false → empty state
    if !config.restore_on_startup {
        return RestoreMode::Empty;
    }

    // Priority 4/5: Check if session has valid content
    if !session_state.has_content() {
        return RestoreMode::Empty;
    }

    // Priority 4: Full restore or layout only
    if config.restore_tabs_on_startup {
        RestoreMode::FullRestore
    } else {
        RestoreMode::LayoutOnly
    }
}

/// Resolve the file open targets based on the restore mode.
///
/// Addresses: Requirement 5 AC 5.1–5.9
pub fn resolve_file_open_targets(
    mode: &RestoreMode,
    cli_args: &CliArgs,
    config: &SessionConfig,
    session_state: &SessionState,
) -> FileOpenTargets {
    match mode {
        RestoreMode::CliArgs => FileOpenTargets::CliArgs(cli_args.source_args.clone()),
        RestoreMode::StartupFile => {
            if let Some(ref startup_file) = config.startup_file {
                FileOpenTargets::StartupFile(startup_file.clone())
            } else {
                FileOpenTargets::Empty
            }
        }
        RestoreMode::FullRestore => {
            if session_state.tabs.is_empty() {
                FileOpenTargets::Empty
            } else {
                FileOpenTargets::SessionRestore(session_state.tabs.clone())
            }
        }
        RestoreMode::LayoutOnly | RestoreMode::Empty => FileOpenTargets::Empty,
    }
}

/// Whether geometry should be restored for the given restore mode and config.
///
/// Geometry is restored unless save_window_geometry is false.
/// Even in Empty mode, geometry is restored if the config allows it.
///
/// Addresses: Requirement 8 AC 8.6
pub fn should_restore_geometry(config: &SessionConfig) -> bool {
    config.save_window_geometry
}

/// Whether layout should be restored for the given restore mode.
///
/// Layout is restored for all modes except Empty (when restore_on_startup=false).
pub fn should_restore_layout(mode: &RestoreMode) -> bool {
    !matches!(mode, RestoreMode::Empty)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_state_with_tabs() -> SessionState {
        SessionState {
            tabs: vec![TabState {
                tab_id: "tab-1".to_string(),
                uri: Some("file.rs".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn cli_args_override_session_restore() {
        // Validates: Requirement 5 AC 5.6, Requirement 6 AC 6.4
        let config = SessionConfig::default();
        let cli_args = CliArgs {
            source_args: vec!["file.txt".to_string()],
            ..Default::default()
        };
        let state = default_state_with_tabs();

        let mode = determine_restore_mode(&config, &cli_args, &state);
        assert_eq!(mode, RestoreMode::CliArgs);
    }

    #[test]
    fn no_session_restore_flag_forces_empty() {
        // Validates: Requirement 6 AC 6.6
        let config = SessionConfig::default();
        let cli_args = CliArgs {
            no_session_restore: true,
            ..Default::default()
        };
        let state = default_state_with_tabs();

        let mode = determine_restore_mode(&config, &cli_args, &state);
        assert_eq!(mode, RestoreMode::Empty);
    }

    #[test]
    fn startup_file_overrides_session_tabs() {
        // Validates: Requirement 6 AC 6.9
        let config = SessionConfig {
            startup_file: Some("startup.rs".to_string()),
            ..Default::default()
        };
        let cli_args = CliArgs::default();
        let state = default_state_with_tabs();

        let mode = determine_restore_mode(&config, &cli_args, &state);
        assert_eq!(mode, RestoreMode::StartupFile);
    }

    #[test]
    fn cli_args_take_precedence_over_startup_file() {
        // Validates: Requirement 6 AC 6.8
        let config = SessionConfig {
            startup_file: Some("startup.rs".to_string()),
            ..Default::default()
        };
        let cli_args = CliArgs {
            source_args: vec!["override.txt".to_string()],
            ..Default::default()
        };
        let state = default_state_with_tabs();

        let mode = determine_restore_mode(&config, &cli_args, &state);
        assert_eq!(mode, RestoreMode::CliArgs);
    }

    #[test]
    fn restore_on_startup_false_gives_empty_state() {
        // Validates: Requirement 5 AC 5.7
        let config = SessionConfig {
            restore_on_startup: false,
            ..Default::default()
        };
        let cli_args = CliArgs::default();
        let state = default_state_with_tabs();

        let mode = determine_restore_mode(&config, &cli_args, &state);
        assert_eq!(mode, RestoreMode::Empty);
    }

    #[test]
    fn full_restore_when_all_conditions_met() {
        // Validates: Requirement 5 AC 5.1
        let config = SessionConfig {
            restore_on_startup: true,
            restore_tabs_on_startup: true,
            ..Default::default()
        };
        let cli_args = CliArgs::default();
        let state = default_state_with_tabs();

        let mode = determine_restore_mode(&config, &cli_args, &state);
        assert_eq!(mode, RestoreMode::FullRestore);
    }

    #[test]
    fn layout_only_when_restore_tabs_false() {
        // Validates: Requirement 5 AC 5.3
        let config = SessionConfig {
            restore_on_startup: true,
            restore_tabs_on_startup: false,
            ..Default::default()
        };
        let cli_args = CliArgs::default();
        let state = default_state_with_tabs();

        let mode = determine_restore_mode(&config, &cli_args, &state);
        assert_eq!(mode, RestoreMode::LayoutOnly);
    }

    #[test]
    fn empty_session_gives_empty_mode() {
        // Validates: Requirement 7 AC 7.1
        let config = SessionConfig::default();
        let cli_args = CliArgs::default();
        let state = SessionState::empty();

        let mode = determine_restore_mode(&config, &cli_args, &state);
        assert_eq!(mode, RestoreMode::Empty);
    }

    #[test]
    fn resolve_cli_args_targets() {
        let config = SessionConfig::default();
        let cli_args = CliArgs {
            source_args: vec!["a.txt".to_string(), "b.rs".to_string()],
            ..Default::default()
        };
        let state = SessionState::empty();

        let targets = resolve_file_open_targets(&RestoreMode::CliArgs, &cli_args, &config, &state);
        assert_eq!(
            targets,
            FileOpenTargets::CliArgs(vec!["a.txt".to_string(), "b.rs".to_string()])
        );
    }

    #[test]
    fn resolve_startup_file_targets() {
        let config = SessionConfig {
            startup_file: Some("start.rs".to_string()),
            ..Default::default()
        };
        let cli_args = CliArgs::default();
        let state = SessionState::empty();

        let targets =
            resolve_file_open_targets(&RestoreMode::StartupFile, &cli_args, &config, &state);
        assert_eq!(
            targets,
            FileOpenTargets::StartupFile("start.rs".to_string())
        );
    }

    #[test]
    fn resolve_full_restore_targets() {
        let config = SessionConfig::default();
        let cli_args = CliArgs::default();
        let state = default_state_with_tabs();

        let targets =
            resolve_file_open_targets(&RestoreMode::FullRestore, &cli_args, &config, &state);
        assert!(matches!(targets, FileOpenTargets::SessionRestore(_)));
    }

    #[test]
    fn resolve_empty_targets() {
        let config = SessionConfig::default();
        let cli_args = CliArgs::default();
        let state = SessionState::empty();

        let targets = resolve_file_open_targets(&RestoreMode::Empty, &cli_args, &config, &state);
        assert_eq!(targets, FileOpenTargets::Empty);
    }

    #[test]
    fn should_restore_geometry_respects_config() {
        // Validates: Requirement 8 AC 8.6
        let config_true = SessionConfig {
            save_window_geometry: true,
            ..Default::default()
        };
        let config_false = SessionConfig {
            save_window_geometry: false,
            ..Default::default()
        };

        assert!(should_restore_geometry(&config_true));
        assert!(!should_restore_geometry(&config_false));
    }

    #[test]
    fn should_restore_layout_for_non_empty_modes() {
        assert!(should_restore_layout(&RestoreMode::CliArgs));
        assert!(should_restore_layout(&RestoreMode::StartupFile));
        assert!(should_restore_layout(&RestoreMode::FullRestore));
        assert!(should_restore_layout(&RestoreMode::LayoutOnly));
        assert!(!should_restore_layout(&RestoreMode::Empty));
    }
}
