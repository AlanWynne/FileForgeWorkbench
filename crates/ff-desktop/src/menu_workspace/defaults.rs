//! Default Menu_File content and first-launch creation.
//!
//! Provides stub default content for `pom.toml` and `settings.toml`.
//! Phase CV will replace DEFAULT_POM_TOML with the final 12-option content.
//! Phase CW will replace DEFAULT_SETTINGS_TOML with the final 10-option content.
//!
//! Validates: Requirement 4.1, 4.2, 4.6 (menu-workspace)

use std::path::Path;

// === Default content ========================================================

/// Default content for `menus/pom.toml` -- 12-option POM (Phase CV).
///
/// Options 0-8 are the Core group (unchanged from Phase AC).
/// Options 9, S, B are the Extended group added in Phase CV.
///
/// Validates: Requirement 7.1, 7.4, 7.5 (cv-requirements.md)
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
key = "3"
command = "UTILITIES"
description = "Perform utility functions"
group = "Core"

[[options]]
key = "4"
command = "COMPILERS"
description = "Interactive language processing"
group = "Core"

[[options]]
key = "5"
command = "MACROS"
description = "Run and manage Lua macros"
group = "Core"

[[options]]
key = "6"
command = "TERMINALS"
description = "Enter TSO or Workstation commands"
group = "Core"

[[options]]
key = "7"
command = "DATABASES"
description = "Database tool and query browser"
group = "Core"

[[options]]
key = "8"
command = "PLUGINS"
description = "Vendor added plugins"
group = "Core"

[[options]]
key = "9"
command = "JOBS"
description = "JES job monitor and spool viewer"
group = "Extended"

[[options]]
key = "S"
command = "SEARCH"
description = "Global search and replace across files"
group = "Extended"

[[options]]
key = "B"
command = "BATCH"
description = "Batch command execution (IKJEFT01 analogue)"
group = "Extended"
"#;

/// Stub default content for `menus/settings.toml`.
///
/// Phase CW will replace this with the final 10-option Settings_Menu content.
///
/// Validates: Requirement 4.2
pub const DEFAULT_SETTINGS_TOML: &str = r#"title = "Settings"

# Phase CW will populate this file with the full 10-option Settings menu.

[[options]]
key = "A"
command = "SETTINGS"
description = "All Settings (flat list)"
"#;

// === ensure_default_menu_files ==============================================

/// Create `menus/pom.toml` and `menus/settings.toml` under `user_data_dir`
/// if they do not already exist.
///
/// Also creates the `menus/` directory if absent.
///
/// Validates: Requirement 4.1, 4.2, 4.6
pub fn ensure_default_menu_files(user_data_dir: &Path) {
    let menus_dir = user_data_dir.join("menus");
    if let Err(_e) = std::fs::create_dir_all(&menus_dir) {
        // Best-effort -- ignore errors (graceful degradation)
        return;
    }

    write_if_absent(&menus_dir.join("pom.toml"), DEFAULT_POM_TOML);
    write_if_absent(&menus_dir.join("settings.toml"), DEFAULT_SETTINGS_TOML);
}

/// Write `content` to `path` only when the file does not already exist.
///
/// Validates: Requirement 4.1, 4.2 -- does not overwrite existing files
fn write_if_absent(path: &Path, content: &str) {
    if path.exists() {
        return;
    }
    if let Err(_e) = std::fs::write(path, content) {
        // Best-effort -- ignore write errors (graceful degradation)
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Validates: Requirement 4.1 -- pom.toml created when absent
    #[test]
    fn ensure_default_menu_files_creates_pom_toml() {
        let dir = TempDir::new().expect("tempdir");
        ensure_default_menu_files(dir.path());
        assert!(dir.path().join("menus").join("pom.toml").exists());
    }

    // Validates: Requirement 4.2 -- settings.toml created when absent
    #[test]
    fn ensure_default_menu_files_creates_settings_toml() {
        let dir = TempDir::new().expect("tempdir");
        ensure_default_menu_files(dir.path());
        assert!(dir.path().join("menus").join("settings.toml").exists());
    }

    // Validates: Requirement 4.6 -- menus/ directory created automatically
    #[test]
    fn ensure_default_menu_files_creates_menus_dir() {
        let dir = TempDir::new().expect("tempdir");
        let menus_dir = dir.path().join("menus");
        assert!(!menus_dir.exists());
        ensure_default_menu_files(dir.path());
        assert!(menus_dir.exists());
    }

    // Validates: Requirement 4.1 -- does not overwrite existing pom.toml
    #[test]
    fn ensure_default_menu_files_does_not_overwrite_existing() {
        let dir = TempDir::new().expect("tempdir");
        let menus_dir = dir.path().join("menus");
        std::fs::create_dir_all(&menus_dir).expect("create dir");
        let pom_path = menus_dir.join("pom.toml");
        std::fs::write(&pom_path, b"custom content").expect("write");
        ensure_default_menu_files(dir.path());
        let content = std::fs::read_to_string(&pom_path).expect("read");
        assert_eq!(
            content, "custom content",
            "existing file must not be overwritten"
        );
    }

    // Validates: Requirement 7.4 (cv-requirements.md) -- DEFAULT_POM_TOML is valid TOML
    #[test]
    fn default_pom_toml_is_valid_toml() {
        let result: Result<toml::Value, _> = toml::from_str(DEFAULT_POM_TOML);
        assert!(result.is_ok(), "DEFAULT_POM_TOML must be valid TOML");
    }

    // Validates: Requirement 7.1 (cv-requirements.md) -- DEFAULT_POM_TOML has 12 options
    #[test]
    fn default_pom_toml_has_12_options() {
        let val: toml::Value = toml::from_str(DEFAULT_POM_TOML).expect("valid TOML");
        let options = val.get("options").and_then(|v| v.as_array()).expect("options array");
        assert_eq!(options.len(), 12, "DEFAULT_POM_TOML must have exactly 12 options");
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

    // Validates: Requirement 4.2 -- DEFAULT_SETTINGS_TOML uses only ASCII
    #[test]
    fn default_settings_toml_ascii_only() {
        assert!(
            DEFAULT_SETTINGS_TOML.is_ascii(),
            "DEFAULT_SETTINGS_TOML must use only ASCII characters"
        );
    }
}
