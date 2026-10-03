//! TOML loader for Menu_Files.
//!
//! Validates: Requirement 1.1-1.7 (menu-workspace)

use std::path::Path;

use serde::Deserialize;

use super::validate::{apply_limits, validate_option};
use super::MenuFile;

// Re-export the limit/validation surface moved to the sibling `validate` module
// so the existing `menu_workspace::loader::<item>` call-site paths (six shell
// modules + the `menu_workspace::OptionLimits` re-export in mod.rs) keep
// resolving unchanged. A single `pub use` both imports the names into this
// module's scope (for the load-fn signatures) and re-exports them.
pub use super::validate::{option_limits_from_config, validate_menu, OptionLimits};

/// A successfully loaded Menu_File plus an optional soft-limit advisory.
///
/// Validates: menu-workspace Requirement 9.2, 9.3
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedMenu {
    /// The parsed and validated menu.
    pub menu: MenuFile,
    /// Populated when the option count exceeds the soft limit (Req 9.3).
    pub advisory: Option<String>,
}

// === Serde shims ============================================================

/// Raw TOML representation of a menu file (serde target).
#[derive(Deserialize)]
struct RawMenuFile {
    title: String,
    #[serde(default)]
    options: Vec<RawMenuOption>,
    /// Whether to draw the calendar panel (menu-workspace Req 1.8). Default true.
    #[serde(default = "default_true")]
    show_calendar: bool,
    /// Group boundary style (menu-workspace Req 2.4/4a, CR-CH-021). Default
    /// `space` (a blank line).
    #[serde(default)]
    group_separator: super::GroupSeparator,
    /// Whether to render group header labels (menu-workspace Req 4b). Default false.
    #[serde(default)]
    group_headers: bool,
}

/// Raw TOML representation of a single option.
///
/// Fields are `pub(super)` because the sibling `validate` module's
/// `validate_option` normalises a `RawMenuOption` into a `MenuOption`.
#[derive(Deserialize)]
pub(super) struct RawMenuOption {
    pub(super) key: String,
    pub(super) command: String,
    pub(super) description: String,
    #[serde(default = "default_true")]
    pub(super) enabled: bool,
    pub(super) group: Option<String>,
    /// Whether this option appears when the menu is rendered as a horizontal
    /// Menu_Bar. Defaults to `true` (menu-workspace Req 17.2, CR-NR-080).
    #[serde(default = "default_true")]
    pub(super) show_in_menu_bar: bool,
    /// Optional inline `[options.target]` table (a serialised Command_Target).
    ///
    /// Validates: menu-workspace Requirement 10.6
    #[serde(default)]
    pub(super) target: Option<ff_command::CommandTarget>,
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
        show_calendar: raw.show_calendar,
        group_separator: raw.group_separator,
        group_headers: raw.group_headers,
    })
}

/// Parse a Menu_File and apply configured option-count limits.
///
/// Parses and validates the file via [`load_menu_file`], then enforces the
/// soft and hard option-count limits:
///
/// - count `<=` effective soft limit: returns [`LoadedMenu`] with no advisory.
/// - count `>` soft but `<=` hard: returns [`LoadedMenu`] with an advisory and
///   logs a WARN.
/// - count `>` hard: returns `Err(..)` using the load-error format so the
///   caller renders no option rows.
///
/// The option count is taken before any `enabled = false` filtering, so
/// disabled options count toward both limits (Requirement 9.8).
///
/// # Errors
///
/// Returns the same error strings as [`load_menu_file`], plus a hard-limit
/// rejection of the form
/// `"Menu file error: too many options: <n> exceeds hard limit <hard>"`.
///
/// Validates: menu-workspace Requirement 9.2, 9.3, 9.4, 9.5, 9.8
pub fn load_menu_file_with_limits(path: &Path, limits: OptionLimits) -> Result<LoadedMenu, String> {
    let menu = load_menu_file(path)?;
    apply_limits(menu, limits, &path.display().to_string())
}

/// Parse a Menu_File from an in-memory TOML string (same validation as
/// [`load_menu_file`]). Used for built-in fallback content (e.g. the default
/// POM) so a menu always has its options even when the on-disk file is absent.
///
/// Validates: menu-workspace Requirement 1.1-1.4
pub fn parse_menu_str(source: &str) -> Result<MenuFile, String> {
    let raw: RawMenuFile = toml::from_str(source).map_err(|e| format!("Menu file error: {}", e))?;
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
        show_calendar: raw.show_calendar,
        group_separator: raw.group_separator,
        group_headers: raw.group_headers,
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

    // Validates: Requirement 10.6 -- inline [options.target] parses into the option
    #[test]
    fn load_inline_options_target_parses() {
        let toml = r#"
title = "T"
[[options]]
key = "B"
command = "IGNORED"
description = "Build Release"
[options.target]
kind = "external"
program = "pwsh"
args = ["-File", "build.ps1"]
mode = "captured"
"#;
        let f = write_toml(toml);
        let menu = load_menu_file(f.path()).expect("load ok");
        let opt = &menu.options[0];
        assert!(opt.target.is_some(), "inline target should be parsed");
        match opt.target.as_ref().unwrap() {
            ff_command::CommandTarget::External { program, mode, .. } => {
                assert_eq!(program, "pwsh");
                assert_eq!(*mode, ff_command::ExternalMode::Captured);
            }
            other => panic!("expected External target, got {:?}", other),
        }
    }

    // Validates: Requirement 10.6 -- absent [options.target] leaves target None
    #[test]
    fn load_option_without_target_is_none() {
        let toml =
            "title = \"T\"\n[[options]]\nkey = \"1\"\ncommand = \"FILES\"\ndescription = \"Files\"\n";
        let f = write_toml(toml);
        let menu = load_menu_file(f.path()).expect("load ok");
        assert!(menu.options[0].target.is_none());
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

    // Validates: menu-workspace Req 1.3, 17.2 (CR-NR-080) -- show_in_menu_bar
    // defaults to true when the key is absent.
    #[test]
    fn load_show_in_menu_bar_defaults_to_true() {
        let toml = "title = \"T\"\n[[options]]\nkey = \"1\"\ncommand = \"FILES\"\ndescription = \"Files\"\n";
        let f = write_toml(toml);
        let menu = load_menu_file(f.path()).expect("load ok");
        assert!(
            menu.options[0].show_in_menu_bar,
            "show_in_menu_bar must default to true"
        );
    }

    // Validates: menu-workspace Req 17.2 -- show_in_menu_bar = false is parsed
    // and preserved (so an option can be hidden from the horizontal bar).
    #[test]
    fn load_show_in_menu_bar_false_preserved() {
        let toml = "title = \"T\"\n[[options]]\nkey = \"X\"\ncommand = \"RETURN\"\ndescription = \"Return\"\nshow_in_menu_bar = false\n";
        let f = write_toml(toml);
        let menu = load_menu_file(f.path()).expect("load ok");
        assert!(
            !menu.options[0].show_in_menu_bar,
            "show_in_menu_bar = false must be preserved"
        );
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

    // Validates: Requirement 1.8 -- show_calendar defaults to true when absent
    #[test]
    fn load_show_calendar_defaults_to_true() {
        let f = write_toml("title = \"Menu\"\n");
        let menu = load_menu_file(f.path()).expect("load ok");
        assert!(
            menu.show_calendar,
            "show_calendar must default to true when the key is absent"
        );
    }

    // Validates: Requirement 1.8 -- show_calendar = false is parsed and preserved
    #[test]
    fn load_show_calendar_false_preserved() {
        let f = write_toml("title = \"Menu\"\nshow_calendar = false\n");
        let menu = load_menu_file(f.path()).expect("load ok");
        assert!(
            !menu.show_calendar,
            "show_calendar = false must be preserved by the loader"
        );
    }

    // Validates: Requirement 2.4/4a (CR-CH-021) -- group_separator defaults to
    // Space when the key is absent.
    #[test]
    fn load_group_separator_defaults_to_space() {
        let f = write_toml("title = \"Menu\"\n");
        let menu = load_menu_file(f.path()).expect("load ok");
        assert_eq!(menu.group_separator, super::super::GroupSeparator::Space);
        assert!(!menu.group_headers, "group_headers defaults to false");
    }

    // Validates: Requirement 2.4/4a -- group_separator values parse from TOML.
    #[test]
    fn load_group_separator_values_parse() {
        for (toml_val, expected) in [
            ("line", super::super::GroupSeparator::Line),
            ("space", super::super::GroupSeparator::Space),
            ("none", super::super::GroupSeparator::None),
        ] {
            let f = write_toml(&format!(
                "title = \"Menu\"\ngroup_separator = \"{toml_val}\"\n"
            ));
            let menu = load_menu_file(f.path()).expect("load ok");
            assert_eq!(menu.group_separator, expected, "for value {toml_val}");
        }
    }

    // Validates: Requirement 4b -- group_headers = true parses.
    #[test]
    fn load_group_headers_true_parses() {
        let f = write_toml("title = \"Menu\"\ngroup_headers = true\n");
        let menu = load_menu_file(f.path()).expect("load ok");
        assert!(menu.group_headers);
    }

    // Validates: Requirement 9.2 -- load_menu_file_with_limits wires I/O + limits
    #[test]
    fn load_with_limits_end_to_end_within_soft() {
        let toml =
            "title = \"T\"\n[[options]]\nkey=\"1\"\ncommand=\"FILES\"\ndescription=\"Files\"\n";
        let f = write_toml(toml);
        let loaded = load_menu_file_with_limits(f.path(), OptionLimits::default()).expect("ok");
        assert!(loaded.advisory.is_none());
        assert_eq!(loaded.menu.options.len(), 1);
    }
}
