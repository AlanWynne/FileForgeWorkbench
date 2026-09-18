//! Option selection dispatch and chained path resolver for Menu_Workspace.
//!
//! Validates: Requirement 3, 5 (menu-workspace)

use super::{MenuFile, MenuOption};

// === Option lookup ==========================================================

/// Look up an option by key (case-insensitive) and return the full
/// [`MenuOption`], so callers can read its inline `[options.target]` in
/// addition to the bare `command` string.
///
/// Returns an error message string when the key is not found or the option is
/// disabled. This is the production lookup used by the command-resolution chain
/// (the former string-only `lookup_option` was removed with B061 as it had no
/// remaining production caller).
///
/// Validates: menu-workspace Requirement 3.1, 3.6, 3.7, 10.6
pub fn find_option<'a>(key: &str, menu: &'a MenuFile) -> Result<&'a MenuOption, String> {
    let upper = key.trim().to_uppercase();
    match menu.options.iter().find(|o| o.key == upper) {
        Some(opt) if opt.enabled => Ok(opt),
        Some(opt) => Err(format!("Option '{}' is not available.", opt.key)),
        None => Err(format!("Option '{}' not found in this menu.", upper)),
    }
}

// Note (B061): chained-path navigation (menu-workspace Requirement 5) is now
// implemented at the shell level in `WorkbenchShell::try_chained_fastpath`,
// which splits a path like `=0.K` into segments and dispatches each as an
// Option_Key through `handle_command` (opening each sub-menu and activating the
// option on the real navigation path). The former `resolve_chained_path` helper
// here -- which returned an unconsumed `|CHAIN:` continuation marker and was
// never wired in (`#[allow(dead_code)]`) -- has been removed as superseded.

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
                    show_in_menu_bar: true,
                    target: None,
                },
                MenuOption {
                    key: "1".to_string(),
                    command: "FILES".to_string(),
                    description: "Files".to_string(),
                    enabled: true,
                    group: None,
                    show_in_menu_bar: true,
                    target: None,
                },
                MenuOption {
                    key: "DIS".to_string(),
                    command: "DISABLED".to_string(),
                    description: "Disabled option".to_string(),
                    enabled: false,
                    group: None,
                    show_in_menu_bar: true,
                    target: None,
                },
            ],
            show_calendar: true,
            group_separator: crate::menu_workspace::GroupSeparator::default(),
            group_headers: false,
        }
    }

    // Validates: Requirement 3.1 -- option key lookup returns command
    #[test]
    fn option_key_lookup_returns_command() {
        let menu = make_menu();
        let opt = find_option("0", &menu).expect("found");
        assert_eq!(opt.command, "SETTINGS");
    }

    // Validates: Requirement 3.1 -- lookup is case-insensitive
    #[test]
    fn option_key_lookup_case_insensitive() {
        let menu = make_menu();
        let result = find_option("dis", &menu);
        // DIS is disabled, so should return error
        assert!(result.is_err());
        // But the lookup itself found it (disabled error, not not-found error)
        assert!(result.unwrap_err().contains("not available"));
    }

    // Validates: Requirement 3.6 -- unknown key returns not-found message
    #[test]
    fn unknown_key_shows_not_found_message() {
        let menu = make_menu();
        let err = find_option("Z", &menu).expect_err("should fail");
        assert!(err.contains("not found in this menu"));
    }

    // Validates: Requirement 3.7 -- disabled option returns not-available message
    #[test]
    fn disabled_option_shows_not_available_message() {
        let menu = make_menu();
        let err = find_option("DIS", &menu).expect_err("should fail");
        assert!(err.contains("not available"));
    }

    // Chained-path navigation (menu-workspace Requirement 5) is now covered at
    // the shell level by `shell::tests::chained_fastpath_*` (it drives the real
    // menu-open + option-activation path); the former `resolve_chained_path`
    // unit tests were removed with that superseded helper (B061).
}
