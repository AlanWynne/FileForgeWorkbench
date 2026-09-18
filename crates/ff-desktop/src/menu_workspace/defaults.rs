//! Compiled Recovery_Baseline menu content (menu-workspace Req 12, CR-CH-021).
//!
//! Built-in menus are CODE-ONLY: [`DEFAULT_POM_TOML`] and
//! [`DEFAULT_SETTINGS_TOML`] are the barebones POM and Settings content used as
//! the fallback when no valid user `menus/*.toml` exists. They are NEVER written
//! to disk (revised Req 4.1/4.2, mirroring the themes code-only rule).
//! [`ensure_menus_dir`] only creates the (possibly empty) `menus/` directory.
//!
//! Validates: Requirement 4.1, 4.2 (revised), 4.6, 12 (menu-workspace)

use std::path::Path;

// === Default content ========================================================

/// Compiled Recovery_Baseline content for the POM (menu-workspace Req 12,
/// CR-CH-021).
///
/// This is the CODE-ONLY barebones POM: it is NEVER written to disk (CR-CH-021
/// revised Req 4.1). It is the fallback rendered when no valid user
/// `menus/pom.toml` exists, and it is deliberately minimal so a user whose
/// configuration is missing or corrupt can always reach Settings, Catalogs,
/// Files, the event Log, and the Menus editor to rebuild or recover.
///
/// Selecting an option dispatches its `command` string (config-driven, no
/// behaviour keyed to the key character). A single group means no stray
/// separator is drawn.
///
/// Validates: Requirement 12.1, 12.2, 12.7 (menu-workspace)
pub const DEFAULT_POM_TOML: &str = r#"title = "FileForge Workbench -- Primary Option Menu"

[[options]]
key = "0"
command = "SETTINGS"
description = "FFWB Settings and Client Parameters"
group = "Core"

[[options]]
key = "1"
command = "CATALOGS"
description = "Virtual File Catalogs -- Mainframe, POSIX, Native"
group = "Core"

[[options]]
key = "2"
command = "FILES"
description = "File Explorer -- Browse catalogs and files in a tree view"
group = "Core"

[[options]]
key = "L"
command = "LOG"
description = "Event Log -- view startup and runtime messages"
group = "Core"

[[options]]
key = "M"
command = "MENUS"
description = "Menus editor -- create, change and save menus"
group = "Core"

[[options]]
key = "X"
command = "RETURN"
description = "Return to the Primary Option Menu (exit when last)"
group = "Core"
"#;

/// Compiled Recovery_Baseline content for the Settings menu (menu-workspace
/// Req 12, CR-CH-021).
///
/// CODE-ONLY barebones Settings: never written to disk. Fallback rendered when
/// no valid user `menus/settings.toml` exists. Minimal set: the Theme editor,
/// the Menus editor, and the browse-all-keys view -- enough to rebuild the rest.
///
/// Validates: Requirement 12.1, 12.3, 12.7 (menu-workspace)
pub const DEFAULT_SETTINGS_TOML: &str = r#"title = "Settings"
show_calendar = false
group_separator = "line"

[[options]]
key = "A"
command = "CONFIG"
description = "All settings -- browse every configuration key"
group = "Core"

[[options]]
key = "T"
command = "THEME"
description = "Theme editor -- copy, edit, save and select themes"
group = "Core"

[[options]]
key = "M"
command = "MENUS"
description = "Menus editor -- create, change and save menus"
group = "Core"

[[options]]
key = "K"
command = "KEYS"
description = "Keys -- edit and save per-workspace key assignments"
group = "Core"

[[options]]
key = "R"
command = "RESET BARE"
description = "Reset to barebones -- archive config and start fresh"
group = "Recovery"
"#;

/// Build the compiled Recovery_Baseline POM `MenuFile` (menu-workspace Req 12).
///
/// Parses [`DEFAULT_POM_TOML`] so there is a SINGLE source of the compiled
/// content (Req 12.7). Panics only on a programmer error (the constant failing
/// to parse), which a unit test guards against.
///
/// Validates: Requirement 12.1, 12.2, 12.7
pub fn recovery_pom_menu() -> crate::menu_workspace::MenuFile {
    crate::menu_workspace::loader::parse_menu_str(DEFAULT_POM_TOML)
        .expect("compiled Recovery_Baseline POM must parse")
}

/// Build the compiled Recovery_Baseline Settings `MenuFile` (menu-workspace
/// Req 12). Parses [`DEFAULT_SETTINGS_TOML`] (single content source, Req 12.7).
///
/// Validates: Requirement 12.1, 12.3, 12.7
pub fn recovery_settings_menu() -> crate::menu_workspace::MenuFile {
    crate::menu_workspace::loader::parse_menu_str(DEFAULT_SETTINGS_TOML)
        .expect("compiled Recovery_Baseline Settings menu must parse")
}

// === ensure_default_menu_files ==============================================

/// Ensure the `menus/` directory exists under `user_data_dir` (creating it if
/// absent), so a user has a place to author Menu_Files. The directory may be
/// empty.
///
/// CR-CH-021 (revised Req 4.1/4.2): built-in menus are CODE-ONLY -- this
/// function NO LONGER writes `pom.toml`/`settings.toml`. The compiled
/// Recovery_Baseline ([`recovery_pom_menu`] / [`recovery_settings_menu`]) is
/// used at render time when no valid user file exists. This mirrors the themes
/// code-only rule (`theme_defaults::ensure_default_theme_files`).
///
/// Validates: Requirement 4.1, 4.2 (revised), 4.6
pub fn ensure_menus_dir(user_data_dir: &Path) {
    let menus_dir = user_data_dir.join("menus");
    // Best-effort -- ignore errors (graceful degradation).
    let _ = std::fs::create_dir_all(&menus_dir);
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Validates: Requirement 4.6 -- menus/ directory created automatically
    #[test]
    fn ensure_menus_dir_creates_menus_dir() {
        let dir = TempDir::new().expect("tempdir");
        let menus_dir = dir.path().join("menus");
        assert!(!menus_dir.exists());
        ensure_menus_dir(dir.path());
        assert!(menus_dir.exists());
    }

    // Validates: Requirement 4.1/4.2 (revised, CR-CH-021) -- built-in menus are
    // CODE-ONLY; ensure_menus_dir never writes pom.toml/settings.toml.
    #[test]
    fn ensure_menus_dir_does_not_materialise_built_in_menus() {
        let dir = TempDir::new().expect("tempdir");
        ensure_menus_dir(dir.path());
        let menus_dir = dir.path().join("menus");
        assert!(
            !menus_dir.join("pom.toml").exists(),
            "pom.toml must NOT be materialised (menus are code-only, CR-CH-021)"
        );
        assert!(
            !menus_dir.join("settings.toml").exists(),
            "settings.toml must NOT be materialised (menus are code-only, CR-CH-021)"
        );
    }

    // Validates: Requirement 4.2 (revised) -- a pre-existing user file is left
    // untouched (ensure_menus_dir does not read or write menu files at all).
    #[test]
    fn ensure_menus_dir_leaves_existing_user_file_untouched() {
        let dir = TempDir::new().expect("tempdir");
        let menus_dir = dir.path().join("menus");
        std::fs::create_dir_all(&menus_dir).expect("create dir");
        let pom_path = menus_dir.join("pom.toml");
        std::fs::write(&pom_path, b"custom content").expect("write");
        ensure_menus_dir(dir.path());
        let content = std::fs::read_to_string(&pom_path).expect("read");
        assert_eq!(content, "custom content", "user file must not be touched");
    }

    // Validates: Requirement 12.1, 12.2, 12.7 -- Recovery_Baseline POM parses
    // from the single compiled source and has the barebones option set.
    #[test]
    fn recovery_pom_menu_has_barebones_options() {
        let menu = recovery_pom_menu();
        let keys: Vec<&str> = menu.options.iter().map(|o| o.key.as_str()).collect();
        assert_eq!(keys, vec!["0", "1", "2", "L", "M", "X"]);
        let commands: Vec<&str> = menu.options.iter().map(|o| o.command.as_str()).collect();
        assert_eq!(
            commands,
            vec!["SETTINGS", "CATALOGS", "FILES", "LOG", "MENUS", "RETURN"]
        );
        // Single group -> no stray separator boundary.
        let groups: std::collections::BTreeSet<&str> = menu
            .options
            .iter()
            .filter_map(|o| o.group.as_deref())
            .collect();
        assert_eq!(groups.len(), 1, "Recovery POM uses a single group");
    }

    // Validates: Requirement 12.1, 12.3, 12.7 -- Recovery_Baseline Settings.
    #[test]
    fn recovery_settings_menu_has_barebones_options() {
        let menu = recovery_settings_menu();
        let keys: Vec<&str> = menu.options.iter().map(|o| o.key.as_str()).collect();
        // CR-CH-025 + CR-CH-029: ordered A Config / T Theme / M Menus / K Keys
        // (group Core), then R Reset-to-barebones (group Recovery). Affordances
        // dispatch via the menu option = command path; `K -> KEYS` opens the
        // Keys Workspace (function-keys Req 22).
        assert_eq!(keys, vec!["A", "T", "M", "K", "R"]);
        let commands: Vec<&str> = menu.options.iter().map(|o| o.command.as_str()).collect();
        assert_eq!(
            commands,
            vec!["CONFIG", "THEME", "MENUS", "KEYS", "RESET BARE"]
        );
    }

    // Validates: Requirement 7.4 (cv-requirements.md) -- DEFAULT_POM_TOML is valid TOML
    #[test]
    fn default_pom_toml_is_valid_toml() {
        let result: Result<toml::Value, _> = toml::from_str(DEFAULT_POM_TOML);
        assert!(result.is_ok(), "DEFAULT_POM_TOML must be valid TOML");
    }

    // Validates: Requirement 12.2 (menu-workspace, CR-CH-021) -- the compiled
    // Recovery_Baseline POM ships exactly the barebones option set whose commands
    // the shell resolves by name.
    #[test]
    fn default_pom_toml_has_recovery_baseline_options() {
        let val: toml::Value = toml::from_str(DEFAULT_POM_TOML).expect("valid TOML");
        let options = val
            .get("options")
            .and_then(|v| v.as_array())
            .expect("options array");
        assert_eq!(
            options.len(),
            6,
            "Recovery_Baseline POM must contain exactly 6 options"
        );
        let commands: Vec<&str> = options
            .iter()
            .filter_map(|o| o.get("command").and_then(|c| c.as_str()))
            .collect();
        assert_eq!(
            commands,
            vec!["SETTINGS", "CATALOGS", "FILES", "LOG", "MENUS", "RETURN"],
        );
    }

    // Validates: Requirement 2.1g (menu-workspace) -- terminate is a data-driven
    // X -> RETURN option, not a bespoke exit line.
    #[test]
    fn default_pom_toml_terminate_is_x_return() {
        let val: toml::Value = toml::from_str(DEFAULT_POM_TOML).expect("valid TOML");
        let options = val
            .get("options")
            .and_then(|v| v.as_array())
            .expect("options array");
        let x = options
            .iter()
            .find(|o| o.get("key").and_then(|k| k.as_str()) == Some("X"))
            .expect("X option present");
        assert_eq!(x.get("command").and_then(|c| c.as_str()), Some("RETURN"));
    }

    // Validates: Requirement 7.1 (cv-requirements.md) -- title matches spec
    #[test]
    fn default_pom_toml_title_matches_spec() {
        let val: toml::Value = toml::from_str(DEFAULT_POM_TOML).expect("valid TOML");
        let title = val.get("title").and_then(|v| v.as_str()).expect("title");
        assert_eq!(title, "FileForge Workbench -- Primary Option Menu");
    }

    // Validates: Requirement 4.2 -- DEFAULT_SETTINGS_TOML is valid TOML
    #[test]
    fn default_settings_toml_is_valid_toml() {
        let result: Result<toml::Value, _> = toml::from_str(DEFAULT_SETTINGS_TOML);
        assert!(result.is_ok(), "DEFAULT_SETTINGS_TOML must be valid TOML");
    }

    // Validates: Requirement 4.1 -- DEFAULT_POM_TOML uses only ASCII
    #[test]
    fn default_pom_toml_ascii_only() {
        assert!(
            DEFAULT_POM_TOML.is_ascii(),
            "DEFAULT_POM_TOML must use only ASCII characters"
        );
    }

    // Validates: Requirement 12.3 (menu-workspace, CR-CH-021) + CR-NR-075 task
    // 24.9 -- Recovery_Baseline Settings: T Themes / M Menus / A All, plus the
    // R RESET BARE affordance (dispatched via the menu-option = command path,
    // configuration-system Req 19.7).
    #[test]
    fn default_settings_toml_has_recovery_baseline_options() {
        let val: toml::Value = toml::from_str(DEFAULT_SETTINGS_TOML).expect("valid TOML");
        let options = val
            .get("options")
            .and_then(|v| v.as_array())
            .expect("options array");
        assert_eq!(
            options.len(),
            5,
            "Recovery_Baseline Settings: A/T/M/K + the RESET BARE affordance"
        );
        // CR-CH-025 + CR-CH-029: ordered A CONFIG, T THEME, M MENUS, K KEYS
        // (group Core), then R RESET BARE (group Recovery).
        let keys: Vec<&str> = options
            .iter()
            .filter_map(|o| o.get("key").and_then(|c| c.as_str()))
            .collect();
        assert_eq!(keys, vec!["A", "T", "M", "K", "R"]);
        let commands: Vec<&str> = options
            .iter()
            .filter_map(|o| o.get("command").and_then(|c| c.as_str()))
            .collect();
        assert_eq!(
            commands,
            vec!["CONFIG", "THEME", "MENUS", "KEYS", "RESET BARE"]
        );
        let groups: Vec<&str> = options
            .iter()
            .filter_map(|o| o.get("group").and_then(|c| c.as_str()))
            .collect();
        assert_eq!(groups, vec!["Core", "Core", "Core", "Core", "Recovery"]);
    }

    // Validates: Requirement 11.1 (cw-requirements.md) -- title matches spec
    #[test]
    fn default_settings_toml_title_matches_spec() {
        let val: toml::Value = toml::from_str(DEFAULT_SETTINGS_TOML).expect("valid TOML");
        let title = val.get("title").and_then(|v| v.as_str()).expect("title");
        // Just "Settings" -- the "FileForge Workbench --" prefix is redundant
        // (we already know which app we are in). B050-follow-up / owner request.
        assert_eq!(title, "Settings");
    }

    // Validates: Requirement 4.2 -- DEFAULT_SETTINGS_TOML uses only ASCII
    #[test]
    fn default_settings_toml_ascii_only() {
        assert!(
            DEFAULT_SETTINGS_TOML.is_ascii(),
            "DEFAULT_SETTINGS_TOML must use only ASCII characters"
        );
    }

    // Validates: menu-workspace Requirement 16.1 (CR-CH-026, B060) -- the
    // compiled Settings default hides the calendar, so the Settings
    // Menu_Workspace has no calendar and no calendar Tab stops by default.
    #[test]
    fn default_settings_toml_hides_calendar() {
        let val: toml::Value = toml::from_str(DEFAULT_SETTINGS_TOML).expect("valid TOML");
        let show = val
            .get("show_calendar")
            .and_then(|v| v.as_bool())
            .expect("show_calendar key present");
        assert!(
            !show,
            "Settings default must set show_calendar = false (Req 16.1)"
        );
        // And it survives the loader (which defaults absent -> true).
        let menu =
            crate::menu_workspace::loader::parse_menu_str(DEFAULT_SETTINGS_TOML).expect("parse ok");
        assert!(
            !menu.show_calendar,
            "loader must preserve show_calendar = false for Settings"
        );
    }
}
