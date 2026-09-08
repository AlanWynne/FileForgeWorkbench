//! Option selection dispatch and chained path resolver for Menu_Workspace.
//!
//! Validates: Requirement 3, 5 (menu-workspace)

use super::MenuFile;

// === Option lookup ==========================================================

/// Look up an option by key (case-insensitive) in a `MenuFile`.
///
/// Returns the `Option_Command` string if found and enabled, or an error
/// message string if not found or disabled.
///
/// Validates: Requirement 3.1, 3.6, 3.7
pub fn lookup_option(key: &str, menu: &MenuFile) -> Result<String, String> {
    let upper = key.trim().to_uppercase();
    match menu.options.iter().find(|o| o.key == upper) {
        Some(opt) if opt.enabled => Ok(opt.command.clone()),
        Some(opt) => Err(format!("Option '{}' is not available.", opt.key)),
        None => Err(format!("Option '{}' not found in this menu.", upper)),
    }
}

// === Chained path resolver ==================================================

/// Resolve a chained dotted path (e.g. `=0.Themes`) against a menu.
///
/// Strips the leading `=` if present, then splits on `.` and resolves each
/// segment against the provided menu. Returns the final `Option_Command` to
/// execute, or an error string if any segment is not found.
///
/// Maximum depth: 4 segments. Deeper paths return an error.
///
/// Validates: Requirement 5.1-5.4
#[allow(dead_code)]
pub fn resolve_chained_path(path: &str, menu: &MenuFile) -> Result<String, String> {
    let stripped = path.trim().strip_prefix('=').unwrap_or(path.trim());
    let segments: Vec<&str> = stripped.splitn(5, '.').collect();

    if segments.len() > 4 {
        return Err("Chained path exceeds maximum depth of 4 segments.".to_string());
    }

    // For a single segment, just look it up directly.
    if segments.len() == 1 {
        return lookup_option(segments[0], menu);
    }

    // For multi-segment paths, resolve the first segment and return the
    // remaining path as a command for the shell to re-dispatch.
    // The shell will open the sub-menu and re-invoke handle_command with
    // the remaining segments. This is the simplest correct implementation
    // that avoids needing a menu registry at this layer.
    let first = segments[0];
    let rest = segments[1..].join(".");

    match menu
        .options
        .iter()
        .find(|o| o.key == first.trim().to_uppercase())
    {
        Some(opt) if opt.enabled => {
            // Return the first command; the shell will chain the rest.
            // We encode the remainder as a special chained continuation.
            Ok(format!("{}|CHAIN:{}", opt.command, rest))
        }
        Some(opt) => Err(format!("Option '{}' is not available.", opt.key)),
        None => Err(format!(
            "Option '{}' not found.",
            first.trim().to_uppercase()
        )),
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_workspace::{MenuFile, MenuOption};

    fn make_menu() -> MenuFile {
        MenuFile {
            title: "Test".to_string(),
            options: vec![
                MenuOption {
                    key: "0".to_string(),
                    command: "SETTINGS".to_string(),
                    description: "Settings".to_string(),
                    enabled: true,
                    group: None,
                },
                MenuOption {
                    key: "1".to_string(),
                    command: "FILES".to_string(),
                    description: "Files".to_string(),
                    enabled: true,
                    group: None,
                },
                MenuOption {
                    key: "DIS".to_string(),
                    command: "DISABLED".to_string(),
                    description: "Disabled option".to_string(),
                    enabled: false,
                    group: None,
                },
            ],
        }
    }

    // Validates: Requirement 3.1 -- option key lookup returns command
    #[test]
    fn option_key_lookup_returns_command() {
        let menu = make_menu();
        let cmd = lookup_option("0", &menu).expect("found");
        assert_eq!(cmd, "SETTINGS");
    }

    // Validates: Requirement 3.1 -- lookup is case-insensitive
    #[test]
    fn option_key_lookup_case_insensitive() {
        let menu = make_menu();
        let cmd = lookup_option("dis", &menu);
        // DIS is disabled, so should return error
        assert!(cmd.is_err());
        // But the lookup itself found it (disabled error, not not-found error)
        assert!(cmd.unwrap_err().contains("not available"));
    }

    // Validates: Requirement 3.6 -- unknown key returns not-found message
    #[test]
    fn unknown_key_shows_not_found_message() {
        let menu = make_menu();
        let err = lookup_option("Z", &menu).expect_err("should fail");
        assert!(err.contains("not found in this menu"));
    }

    // Validates: Requirement 3.7 -- disabled option returns not-available message
    #[test]
    fn disabled_option_shows_not_available_message() {
        let menu = make_menu();
        let err = lookup_option("DIS", &menu).expect_err("should fail");
        assert!(err.contains("not available"));
    }

    // Validates: Requirement 5.1 -- single-segment chained path resolves correctly
    #[test]
    fn chained_path_single_segment() {
        let menu = make_menu();
        let cmd = resolve_chained_path("=0", &menu).expect("found");
        assert_eq!(cmd, "SETTINGS");
    }

    // Validates: Requirement 5.1 -- two-level chained path returns first command with chain
    #[test]
    fn chained_path_two_levels() {
        let menu = make_menu();
        let result = resolve_chained_path("=0.Themes", &menu).expect("found");
        assert!(result.starts_with("SETTINGS"));
        assert!(result.contains("CHAIN:Themes"));
    }

    // Validates: Requirement 5.3 -- unknown segment returns error
    #[test]
    fn chained_path_unknown_segment_stops_at_last_resolved() {
        let menu = make_menu();
        let err = resolve_chained_path("=Z.sub", &menu).expect_err("should fail");
        assert!(err.contains("not found"));
    }

    // Validates: Requirement 5.2 -- path exceeding 4 segments returns error
    #[test]
    fn chained_path_max_depth_exceeded_returns_error() {
        let menu = make_menu();
        let err = resolve_chained_path("=0.a.b.c.d", &menu).expect_err("should fail");
        assert!(err.contains("maximum depth"));
    }

    // Validates: Requirement 5.4 -- single-segment fastpath unchanged
    #[test]
    fn single_segment_fastpath_unchanged() {
        let menu = make_menu();
        let cmd = resolve_chained_path("=1", &menu).expect("found");
        assert_eq!(cmd, "FILES");
    }
}
