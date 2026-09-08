//! TOML loader for Menu_Files.
//!
//! Validates: Requirement 1.1-1.7 (menu-workspace)

use std::path::Path;

use serde::Deserialize;

use super::{MenuFile, MenuOption};

// === Serde shims ============================================================

/// Raw TOML representation of a menu file (serde target).
#[derive(Deserialize)]
struct RawMenuFile {
    title: String,
    #[serde(default)]
    options: Vec<RawMenuOption>,
}

/// Raw TOML representation of a single option.
#[derive(Deserialize)]
struct RawMenuOption {
    key: String,
    command: String,
    description: String,
    #[serde(default = "default_true")]
    enabled: bool,
    group: Option<String>,
}

fn default_true() -> bool {
    true
}

// === load_menu_file =========================================================

/// Parse a Menu_File from `path` and return a validated [`MenuFile`].
///
/// # Errors
///
/// Returns a human-readable error string on any failure:
/// - File not found or unreadable: `"Menu file not found: <path>"`
/// - Invalid TOML or missing required field: `"Menu file error: <reason>"`
///
/// Validates: Requirement 1.1-1.7
pub fn load_menu_file(path: &Path) -> Result<MenuFile, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|_| format!("Menu file not found: {}", path.display()))?;

    let raw: RawMenuFile =
        toml::from_str(&source).map_err(|e| format!("Menu file error: {}", e))?;

    if raw.title.is_empty() {
        return Err("Menu file error: 'title' field is required and must not be empty".to_string());
    }

    let options = raw
        .options
        .into_iter()
        .enumerate()
        .map(|(i, o)| validate_option(o, i))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(MenuFile {
        title: raw.title,
        options,
    })
}

/// Validate and normalise a single raw option.
///
/// Validates: Requirement 1.2 -- key 1-4 chars, stored uppercase
fn validate_option(raw: RawMenuOption, index: usize) -> Result<MenuOption, String> {
    let key = raw.key.trim().to_uppercase();
    if key.is_empty() || key.len() > 4 {
        return Err(format!(
            "Menu file error: option {} key '{}' must be 1-4 characters",
            index + 1,
            raw.key
        ));
    }
    if raw.command.trim().is_empty() {
        return Err(format!(
            "Menu file error: option {} (key '{}') 'command' field is required",
            index + 1,
            key
        ));
    }
    if raw.description.trim().is_empty() {
        return Err(format!(
            "Menu file error: option {} (key '{}') 'description' field is required",
            index + 1,
            key
        ));
    }
    Ok(MenuOption {
        key,
        command: raw.command,
        description: raw.description,
        enabled: raw.enabled,
        group: raw.group,
    })
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_toml(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("tempfile");
        f.write_all(content.as_bytes()).expect("write");
        f
    }

    // Validates: Requirement 1.1 -- valid file parses correctly
    #[test]
    fn load_valid_menu_file() {
        let toml = r#"
title = "Primary Option Menu"
[[options]]
key = "0"
command = "SETTINGS"
description = "FFWB Settings"
group = "System"

[[options]]
key = "1"
command = "FILES"
description = "File Explorer"
enabled = true
"#;
        let f = write_toml(toml);
        let menu = load_menu_file(f.path()).expect("load ok");
        assert_eq!(menu.title, "Primary Option Menu");
        assert_eq!(menu.options.len(), 2);
        assert_eq!(menu.options[0].key, "0");
        assert_eq!(menu.options[0].command, "SETTINGS");
        assert_eq!(menu.options[0].group.as_deref(), Some("System"));
        assert!(menu.options[0].enabled);
        assert_eq!(menu.options[1].key, "1");
    }

    // Validates: Requirement 1.2 -- key normalised to uppercase
    #[test]
    fn load_key_normalised_to_uppercase() {
        let toml =
            "title = \"T\"\n[[options]]\nkey = \"jes\"\ncommand = \"JES\"\ndescription = \"JES\"\n";
        let f = write_toml(toml);
        let menu = load_menu_file(f.path()).expect("load ok");
        assert_eq!(menu.options[0].key, "JES");
    }

    // Validates: Requirement 1.3 -- enabled defaults to true when absent
    #[test]
    fn load_enabled_defaults_to_true() {
        let toml = "title = \"T\"\n[[options]]\nkey = \"1\"\ncommand = \"FILES\"\ndescription = \"Files\"\n";
        let f = write_toml(toml);
        let menu = load_menu_file(f.path()).expect("load ok");
        assert!(menu.options[0].enabled);
    }

    // Validates: Requirement 1.3 -- enabled = false is preserved
    #[test]
    fn load_enabled_false_preserved() {
        let toml = "title = \"T\"\n[[options]]\nkey = \"1\"\ncommand = \"FILES\"\ndescription = \"Files\"\nenabled = false\n";
        let f = write_toml(toml);
        let menu = load_menu_file(f.path()).expect("load ok");
        assert!(!menu.options[0].enabled);
    }

    // Validates: Requirement 1.4 -- unknown keys silently ignored
    #[test]
    fn load_unknown_key_is_ignored() {
        let toml = "title = \"T\"\nunknown_field = \"ignored\"\n[[options]]\nkey = \"1\"\ncommand = \"FILES\"\ndescription = \"Files\"\nextra = \"also ignored\"\n";
        let f = write_toml(toml);
        // Should not error
        let result = load_menu_file(f.path());
        assert!(result.is_ok(), "unknown keys must be silently ignored");
    }

    // Validates: Requirement 1.5 -- missing file returns error
    #[test]
    fn load_missing_file_returns_error() {
        let err = load_menu_file(Path::new("/nonexistent/menu.toml")).expect_err("should fail");
        assert!(err.contains("Menu file not found"));
    }

    // Validates: Requirement 1.6 -- invalid TOML returns error
    #[test]
    fn load_invalid_toml_returns_error() {
        let f = write_toml("not valid toml [[[");
        let err = load_menu_file(f.path()).expect_err("should fail");
        assert!(err.contains("Menu file error"));
    }

    // Validates: Requirement 1.6 -- missing required field returns error
    #[test]
    fn load_missing_required_field_returns_error() {
        // Missing 'title'
        let f =
            write_toml("[[options]]\nkey = \"1\"\ncommand = \"FILES\"\ndescription = \"Files\"\n");
        let err = load_menu_file(f.path()).expect_err("should fail");
        assert!(err.contains("Menu file error"));
    }

    // Validates: Requirement 1.2 -- key longer than 4 chars returns error
    #[test]
    fn load_key_too_long_returns_error() {
        let toml = "title = \"T\"\n[[options]]\nkey = \"TOOLONG\"\ncommand = \"FILES\"\ndescription = \"Files\"\n";
        let f = write_toml(toml);
        let err = load_menu_file(f.path()).expect_err("should fail");
        assert!(err.contains("1-4 characters"));
    }

    // Validates: Requirement 1.1 -- empty options array is valid
    #[test]
    fn load_empty_options_is_valid() {
        let f = write_toml("title = \"Empty Menu\"\n");
        let menu = load_menu_file(f.path()).expect("load ok");
        assert_eq!(menu.title, "Empty Menu");
        assert!(menu.options.is_empty());
    }
}
