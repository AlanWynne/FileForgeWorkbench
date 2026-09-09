//! TOML loader for Menu_Files.
//!
//! Validates: Requirement 1.1-1.7 (menu-workspace)

use std::path::Path;

use serde::Deserialize;

use super::{MenuFile, MenuOption};

// === Option limits ==========================================================

/// Configured option-count limits for a Menu_File.
///
/// Validates: menu-workspace Requirement 9.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionLimits {
    /// Advisory limit; above it the menu loads with a warning advisory.
    pub soft: u32,
    /// Hard limit; above it the menu file is rejected as a load error.
    pub hard: u32,
}

impl OptionLimits {
    /// Default limits used when configuration provides no override.
    ///
    /// Validates: menu-workspace Requirement 9.6
    pub const DEFAULT_SOFT: u32 = 64;
    /// Default hard limit.
    pub const DEFAULT_HARD: u32 = 256;

    /// Create limits from raw soft/hard values.
    // Used by tests now and by the POM/Settings migration phase (which will
    // construct limits from config) once those menus adopt the pattern.
    #[allow(dead_code)]
    pub fn new(soft: u32, hard: u32) -> Self {
        Self { soft, hard }
    }

    /// The effective soft limit, never greater than the hard limit.
    ///
    /// Validates: menu-workspace Requirement 9.5
    fn effective_soft(&self) -> u32 {
        self.soft.min(self.hard)
    }
}

impl Default for OptionLimits {
    fn default() -> Self {
        Self {
            soft: Self::DEFAULT_SOFT,
            hard: Self::DEFAULT_HARD,
        }
    }
}

/// Resolve [`OptionLimits`] from the configuration system.
///
/// Reads `menu.soft_option_limit` and `menu.hard_option_limit`. A key that is
/// absent, non-integer, or negative falls back to the default for that key
/// (the config layer already substitutes the schema default on type mismatch;
/// this function additionally guards against negative integers).
///
/// Validates: menu-workspace Requirement 9.1, 9.6, 9.7
// Public entry point for the POM/Settings migration phase, which will read the
// configured limits when constructing a MenuWorkspaceState. Exercised by tests.
#[allow(dead_code)]
pub fn option_limits_from_config(config: &ff_config::ConfigHandle) -> OptionLimits {
    let soft = read_u32_or_default(
        config,
        ff_config::keys::menu::SOFT_OPTION_LIMIT,
        OptionLimits::DEFAULT_SOFT,
    );
    let hard = read_u32_or_default(
        config,
        ff_config::keys::menu::HARD_OPTION_LIMIT,
        OptionLimits::DEFAULT_HARD,
    );
    OptionLimits::new(soft, hard)
}

/// Read an integer config key as `u32`, falling back to `default` when the key
/// is missing, non-integer, or negative. Logs a WARN on a negative value.
///
/// Validates: menu-workspace Requirement 9.6, 9.7
// Only reached via option_limits_from_config (migration-phase entry point).
#[allow(dead_code)]
fn read_u32_or_default(config: &ff_config::ConfigHandle, key: &str, default: u32) -> u32 {
    match config.get_int(key) {
        Ok(v) if v >= 0 => u32::try_from(v).unwrap_or(default),
        Ok(v) => {
            ff_logging::log_warn!(
                "[menu] config key '{}' is negative ({}); using default {}",
                key,
                v,
                default
            );
            default
        }
        Err(_) => default,
    }
}

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
    /// Optional inline `[options.target]` table (a serialised Command_Target).
    ///
    /// Validates: menu-workspace Requirement 10.6
    #[serde(default)]
    target: Option<ff_command::CommandTarget>,
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

/// Apply option-count limits to an already-parsed menu.
///
/// Separated from I/O so it is directly unit-testable without a temp file.
///
/// Validates: menu-workspace Requirement 9.2, 9.3, 9.4, 9.5, 9.8
fn apply_limits(
    menu: MenuFile,
    limits: OptionLimits,
    path_label: &str,
) -> Result<LoadedMenu, String> {
    // Req 9.8: count parsed options before enabled filtering.
    let count = menu.options.len() as u64;
    let hard = limits.hard as u64;
    let effective_soft = limits.effective_soft() as u64;

    // Req 9.5: warn when the hard limit dominates a larger soft limit.
    if limits.soft > limits.hard {
        ff_logging::log_warn!(
            "[menu] soft_option_limit ({}) exceeds hard_option_limit ({}); \
             clamping effective soft limit to {}",
            limits.soft,
            limits.hard,
            limits.hard
        );
    }

    // Req 9.4: reject above the hard limit.
    if count > hard {
        return Err(format!(
            "Menu file error: too many options: {count} exceeds hard limit {hard}"
        ));
    }

    // Req 9.3: advise above the soft limit.
    if count > effective_soft {
        ff_logging::log_warn!(
            "[menu] '{}' has {} options (advised maximum {})",
            path_label,
            count,
            effective_soft
        );
        let advisory = format!(
            "This menu has {count} options (advised maximum {effective_soft}). \
             Consider grouping options into sub-menus."
        );
        return Ok(LoadedMenu {
            menu,
            advisory: Some(advisory),
        });
    }

    // Req 9.2: within the soft limit, no advisory.
    Ok(LoadedMenu {
        menu,
        advisory: None,
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
    // Req 10.6: when both `command` and an inline `[options.target]` are
    // present, the inline target wins; note the ignored `command` at DEBUG.
    if raw.target.is_some() {
        ff_logging::log_debug!(
            "[menu] option '{}' has an inline [options.target]; the 'command' value \
             '{}' is ignored for target resolution",
            key,
            raw.command.trim()
        );
    }
    Ok(MenuOption {
        key,
        command: raw.command,
        description: raw.description,
        enabled: raw.enabled,
        group: raw.group,
        target: raw.target,
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

    // === Option limits (Requirement 9) ==================================

    /// Build a MenuFile with `n` enabled options and `disabled` extra disabled
    /// options, for exercising the limit logic directly.
    fn menu_with(n: usize, disabled: usize) -> MenuFile {
        let mut options = Vec::new();
        for i in 0..n {
            options.push(MenuOption {
                key: format!("{:X}", i % 16),
                command: "NOOP".to_string(),
                description: format!("Option {i}"),
                enabled: true,
                group: None,
                target: None,
            });
        }
        for i in 0..disabled {
            options.push(MenuOption {
                key: format!("D{i}"),
                command: "NOOP".to_string(),
                description: format!("Disabled {i}"),
                enabled: false,
                group: None,
                target: None,
            });
        }
        MenuFile {
            title: "T".to_string(),
            options,
        }
    }

    // Validates: Requirement 9.2 -- count at/below soft limit -> no advisory
    #[test]
    fn count_at_or_below_soft_limit_no_advisory() {
        let limits = OptionLimits::new(64, 256);
        // Exactly at the soft limit.
        let loaded = apply_limits(menu_with(64, 0), limits, "t.toml").expect("ok");
        assert!(loaded.advisory.is_none());
        assert_eq!(loaded.menu.options.len(), 64);
    }

    // Validates: Requirement 9.3 -- above soft, at/below hard -> advisory set
    #[test]
    fn count_above_soft_below_hard_sets_advisory() {
        let limits = OptionLimits::new(64, 256);
        let loaded = apply_limits(menu_with(65, 0), limits, "t.toml").expect("ok");
        let advisory = loaded.advisory.expect("advisory expected");
        assert!(advisory.contains("65 options"));
        assert!(advisory.contains("advised maximum 64"));
        // Menu still fully loaded.
        assert_eq!(loaded.menu.options.len(), 65);
    }

    // Validates: Requirement 9.4 -- above hard limit -> load error, no menu
    #[test]
    fn count_above_hard_returns_load_error() {
        let limits = OptionLimits::new(64, 256);
        let err = apply_limits(menu_with(257, 0), limits, "t.toml").expect_err("must reject");
        assert!(err.contains("too many options"));
        assert!(err.contains("257"));
        assert!(err.contains("hard limit 256"));
    }

    // Validates: Requirement 9.5 -- hard below soft clamps effective soft to hard
    #[test]
    fn hard_below_soft_clamps_effective_soft_to_hard() {
        // soft 100, hard 10 -> effective soft is 10.
        let limits = OptionLimits::new(100, 10);
        // 10 options: at effective soft, no advisory.
        let ok = apply_limits(menu_with(10, 0), limits, "t.toml").expect("ok");
        assert!(ok.advisory.is_none());
        // 11 options: above effective soft (10) but must still be rejected as
        // above hard (10) since hard dominates.
        let err = apply_limits(menu_with(11, 0), limits, "t.toml").expect_err("reject");
        assert!(err.contains("hard limit 10"));
    }

    // Validates: Requirement 9.8 -- disabled options count toward the limits
    #[test]
    fn disabled_options_count_toward_limits() {
        let limits = OptionLimits::new(64, 256);
        // 60 enabled + 6 disabled = 66 total -> above soft (64).
        let loaded = apply_limits(menu_with(60, 6), limits, "t.toml").expect("ok");
        let advisory = loaded.advisory.expect("advisory expected because 66 > 64");
        assert!(advisory.contains("66 options"));
    }

    // Validates: Requirement 9.6 -- default limits are 64 / 256
    #[test]
    fn default_limits_are_64_and_256() {
        let d = OptionLimits::default();
        assert_eq!(d.soft, 64);
        assert_eq!(d.hard, 256);
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

    // Validates: Requirement 9.6, 9.7 -- limits fall back to defaults when the
    // config keys are absent or unreadable (get_int returns Err -> default).
    #[test]
    fn option_limits_from_config_falls_back_to_defaults() {
        use ff_config::init::{init, ConfigInitOptions};
        use tempfile::TempDir;
        let tmp = TempDir::new().expect("tempdir");
        let config = init(
            ConfigInitOptions::new()
                .with_hot_reload(false)
                .with_project_root(tmp.path().to_path_buf()),
        )
        .expect("config init");
        // No menu.* schema registered here, so get_int errors and we fall back.
        let limits = option_limits_from_config(&config);
        assert_eq!(limits.soft, OptionLimits::DEFAULT_SOFT);
        assert_eq!(limits.hard, OptionLimits::DEFAULT_HARD);
    }
}
