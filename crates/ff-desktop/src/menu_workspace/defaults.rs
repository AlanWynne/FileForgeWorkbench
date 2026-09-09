//! Default Menu_File content and first-launch creation.
//!
//! Provides the default content for `pom.toml` (12-option POM, Phase CV)
//! and `settings.toml` (10-option Settings_Menu, Phase CW), plus first-launch
//! creation via `ensure_default_menu_files()`.
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

/// Default content for `menus/settings.toml` -- 10-option Settings_Menu (Phase CW).
///
/// Options E-X are the Namespaces group; option A is the All group.
/// Each namespace option carries a `SETTINGS <namespace>` command that opens
/// a filtered Settings_Namespace_View; option A opens the unfiltered flat list.
///
/// Validates: Requirement 9.1, 11.1, 11.4, 11.5 (cw-requirements.md)
pub const DEFAULT_SETTINGS_TOML: &str = r#"title = "FileForge Workbench -- Settings"

[[options]]
key = "E"
command = "SETTINGS editor"
description = "Text editing behaviour -- indentation, line endings, encoding"
group = "Namespaces"

[[options]]
key = "T"
command = "SETTINGS theme"
description = "Appearance -- active theme, font size, OS dark/light follow"
group = "Namespaces"

[[options]]
key = "C"
command = "SETTINGS catalog"
description = "Default catalog roots for Mainframe and POSIX catalogs"
group = "Namespaces"

[[options]]
key = "V"
command = "SETTINGS vfs"
description = "Virtual File System provider settings"
group = "Namespaces"

[[options]]
key = "L"
command = "SETTINGS logging"
description = "Log level, output directory, file rotation"
group = "Namespaces"

[[options]]
key = "K"
command = "SETTINGS keymap"
description = "Function key bindings and per-context key maps"
group = "Namespaces"

[[options]]
key = "S"
command = "SETTINGS session"
description = "Session persistence, restore behaviour, recent files"
group = "Namespaces"

[[options]]
key = "P"
command = "SETTINGS plugin"
description = "Plugin-specific configuration namespaces"
group = "Namespaces"

[[options]]
key = "X"
command = "SETTINGS accessibility"
description = "Reduce motion, focus indicators, contrast settings"
group = "Namespaces"

[[options]]
key = "A"
command = "A"
description = "Browse all configuration keys (unfiltered flat list)"
group = "All""#;

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
        let options = val
            .get("options")
            .and_then(|v| v.as_array())
            .expect("options array");
        assert_eq!(
            options.len(),
            12,
            "DEFAULT_POM_TOML must have exactly 12 options"
        );
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

    // Validates: Requirement 11.1 (cw-requirements.md) -- DEFAULT_SETTINGS_TOML has 10 options
    #[test]
    fn default_settings_toml_has_10_options() {
        let val: toml::Value = toml::from_str(DEFAULT_SETTINGS_TOML).expect("valid TOML");
        let options = val
            .get("options")
            .and_then(|v| v.as_array())
            .expect("options array");
        assert_eq!(
            options.len(),
            10,
            "DEFAULT_SETTINGS_TOML must have exactly 10 options"
        );
    }

    // Validates: Requirement 11.1 (cw-requirements.md) -- title matches spec
    #[test]
    fn default_settings_toml_title_matches_spec() {
        let val: toml::Value = toml::from_str(DEFAULT_SETTINGS_TOML).expect("valid TOML");
        let title = val.get("title").and_then(|v| v.as_str()).expect("title");
        assert_eq!(title, "FileForge Workbench -- Settings");
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
