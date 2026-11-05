//! Serialise a [`MenuFile`] back to Menu_File TOML (menu-workspace Req 13.8,
//! CR-NR-075).
//!
//! The loader (`loader.rs`) is Deserialize-only and the built-in menus are
//! hand-written string constants, so the Menus editor needs this writer to save
//! edited menus. The output round-trips through [`crate::menu_workspace::loader::parse_menu_str`]
//! (Req 13.10): parsing a serialised menu yields an equal [`MenuFile`].
//!
//! Fields that equal their loader default are omitted to keep files clean
//! (`enabled = true`, `show_calendar = true`, `group_separator = space`,
//! `group_headers = false`, and an absent `group`).

use super::{GroupSeparator, MenuFile, MenuOption};

/// The config-model version stamp written into a Complete_Config_File
/// (CR-CH-116, configuration-system Req 23.9). It leaves room for a future
/// deferred UPGRADE/merge command (Req 23.10) without implementing one now. It
/// is crate-private and purely additive: the loader accepts and ignores it.
pub(crate) const CONFIG_VERSION: u32 = 1;

/// Serialise a [`MenuFile`] to COMPLETE Menu_File TOML text (CR-CH-116,
/// configuration-system Req 23.4). Unlike [`serialise`], which omits any field
/// equal to its loader default for clean editor output, this emits EVERY flag
/// explicitly -- `title`, `show_calendar`, `group_separator`, `group_headers`,
/// and for each option `key`/`command`/`description`/`enabled`/`group`/
/// `show_in_menu_bar` -- plus a top-level `config_version` stamp. A
/// Complete_Config_File fully determines behaviour with no reliance on a hidden
/// loader default (e.g. the `show_calendar`-defaults-to-true trap). It
/// round-trips through [`crate::menu_workspace::loader::parse_menu_str`].
///
/// Validates: configuration-system Requirement 23.4, 23.9
pub fn serialise_complete(menu: &MenuFile) -> String {
    let mut out = String::new();

    out.push_str(&format!("config_version = {CONFIG_VERSION}\n"));
    out.push_str(&format!("title = {}\n", toml_string(&menu.title)));
    out.push_str(&format!("show_calendar = {}\n", menu.show_calendar));
    out.push_str(&format!(
        "group_separator = {}\n",
        toml_string(group_separator_str(menu.group_separator))
    ));
    out.push_str(&format!("group_headers = {}\n", menu.group_headers));

    for option in &menu.options {
        out.push('\n');
        out.push_str(&serialise_option_complete(option));
    }

    out
}

/// Serialise a single option as a `[[options]]` block with EVERY flag explicit
/// (COMPLETE mode). An absent `group` is emitted as an empty string so the field
/// is always present; the loader normalises an empty/absent group identically.
fn serialise_option_complete(option: &MenuOption) -> String {
    let mut s = String::new();
    s.push_str("[[options]]\n");
    s.push_str(&format!("key = {}\n", toml_string(&option.key)));
    s.push_str(&format!("command = {}\n", toml_string(&option.command)));
    s.push_str(&format!(
        "description = {}\n",
        toml_string(&option.description)
    ));
    s.push_str(&format!("enabled = {}\n", option.enabled));
    if let Some(group) = &option.group {
        s.push_str(&format!("group = {}\n", toml_string(group)));
    }
    s.push_str(&format!("show_in_menu_bar = {}\n", option.show_in_menu_bar));
    // Inline Command_Target, same handling as the clean serialiser.
    if let Some(target) = &option.target {
        if let Ok(table) = toml::to_string(target) {
            s.push_str("[options.target]\n");
            for line in table.lines() {
                s.push_str(line);
                s.push('\n');
            }
        }
    }
    s
}

/// Serialise a [`MenuFile`] to TOML text.
///
/// Validates: menu-workspace Requirement 13.8, 13.10
pub fn serialise(menu: &MenuFile) -> String {
    let mut out = String::new();

    out.push_str(&format!("title = {}\n", toml_string(&menu.title)));

    // Only emit display settings that differ from the loader defaults.
    if !menu.show_calendar {
        out.push_str("show_calendar = false\n");
    }
    if menu.group_separator != GroupSeparator::default() {
        out.push_str(&format!(
            "group_separator = {}\n",
            toml_string(group_separator_str(menu.group_separator))
        ));
    }
    if menu.group_headers {
        out.push_str("group_headers = true\n");
    }

    for option in &menu.options {
        out.push('\n');
        out.push_str(&serialise_option(option));
    }

    out
}

/// Serialise a single option as a `[[options]]` block.
fn serialise_option(option: &MenuOption) -> String {
    let mut s = String::new();
    s.push_str("[[options]]\n");
    s.push_str(&format!("key = {}\n", toml_string(&option.key)));
    s.push_str(&format!("command = {}\n", toml_string(&option.command)));
    s.push_str(&format!(
        "description = {}\n",
        toml_string(&option.description)
    ));
    if !option.enabled {
        s.push_str("enabled = false\n");
    }
    if let Some(group) = &option.group {
        s.push_str(&format!("group = {}\n", toml_string(group)));
    }
    // Default is true; emit only when hidden from the bar (CR-NR-080).
    if !option.show_in_menu_bar {
        s.push_str("show_in_menu_bar = false\n");
    }
    // Inline Command_Target (menu-workspace Req 10.6). CommandTarget derives
    // Serialize, so emit it as an `[options.target]` sub-table. If serialisation
    // ever fails, fall back to the bare `command` (already written above).
    if let Some(target) = &option.target {
        if let Ok(table) = toml::to_string(target) {
            s.push_str("[options.target]\n");
            for line in table.lines() {
                s.push_str(line);
                s.push('\n');
            }
        }
    }
    s
}

/// The lowercase TOML token for a [`GroupSeparator`] (matches the loader's
/// `#[serde(rename_all = "lowercase")]`).
fn group_separator_str(sep: GroupSeparator) -> &'static str {
    match sep {
        GroupSeparator::Line => "line",
        GroupSeparator::Space => "space",
        GroupSeparator::None => "none",
    }
}

/// Emit a TOML basic string with the necessary escaping (quotes and
/// backslashes). Menu strings are short single-line values.
fn toml_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_workspace::loader::parse_menu_str;

    // Validates: configuration-system Requirement 23.4 (CR-CH-116) --
    // serialise_complete emits show_calendar EXPLICITLY (true for the POM) and
    // the result round-trips through parse_menu_str.
    #[test]
    fn serialise_complete_pom_emits_show_calendar_true_and_round_trips() {
        let original = crate::menu_workspace::defaults::recovery_pom_menu();
        let toml = serialise_complete(&original);
        assert!(
            toml.contains("show_calendar = true"),
            "POM complete output must state show_calendar = true explicitly: {toml}"
        );
        assert!(
            toml.contains("config_version = 1"),
            "complete output carries the config_version stamp: {toml}"
        );
        let parsed = parse_menu_str(&toml).expect("complete POM must parse");
        assert_eq!(parsed, original);
    }

    // Validates: configuration-system Requirement 23.4 (CR-CH-116) --
    // serialise_complete emits show_calendar=false EXPLICITLY for Settings
    // (whose default is false) and round-trips.
    #[test]
    fn serialise_complete_settings_emits_show_calendar_false_and_round_trips() {
        let original = crate::menu_workspace::defaults::recovery_settings_menu();
        let toml = serialise_complete(&original);
        assert!(
            toml.contains("show_calendar = false"),
            "Settings complete output must state show_calendar = false explicitly: {toml}"
        );
        let parsed = parse_menu_str(&toml).expect("complete Settings must parse");
        assert_eq!(parsed, original);
    }

    // Validates: configuration-system Requirement 23.4 (CR-CH-116) -- every flag
    // is present in complete output, including group_separator, group_headers,
    // and per-option enabled/show_in_menu_bar; the existing serialise() stays
    // clean (default-omitting) by contrast.
    #[test]
    fn serialise_complete_emits_every_flag() {
        let menu = MenuFile {
            title: "M".to_string(),
            options: vec![MenuOption {
                key: "A".to_string(),
                command: "A".to_string(),
                description: "All".to_string(),
                enabled: true,
                group: None,
                show_in_menu_bar: true,
                target: None,
            }],
            show_calendar: true,
            group_separator: GroupSeparator::Space,
            group_headers: false,
        };
        let toml = serialise_complete(&menu);
        for needle in [
            "show_calendar = true",
            "group_separator = \"space\"",
            "group_headers = false",
            "enabled = true",
            "show_in_menu_bar = true",
        ] {
            assert!(
                toml.contains(needle),
                "complete output missing {needle}: {toml}"
            );
        }
        // The clean serialiser omits these defaults; the two modes differ.
        assert!(!serialise(&menu).contains("show_calendar"));
        let parsed = parse_menu_str(&toml).expect("parse");
        assert_eq!(parsed, menu);
    }

    // Validates: Requirement 13.10 -- the Recovery_Baseline POM round-trips
    // through serialise -> parse unchanged.
    #[test]
    fn recovery_pom_round_trips() {
        let original = crate::menu_workspace::defaults::recovery_pom_menu();
        let toml = serialise(&original);
        let parsed = parse_menu_str(&toml).expect("serialised POM must parse");
        assert_eq!(parsed, original);
    }

    // Validates: Requirement 13.10 -- the Recovery_Baseline Settings menu
    // round-trips.
    #[test]
    fn recovery_settings_round_trips() {
        let original = crate::menu_workspace::defaults::recovery_settings_menu();
        let toml = serialise(&original);
        let parsed = parse_menu_str(&toml).expect("serialised Settings must parse");
        assert_eq!(parsed, original);
    }

    // Validates: Requirement 13.10 -- a menu exercising every serialised field
    // (disabled option, groups, non-default display settings) round-trips.
    #[test]
    fn full_featured_menu_round_trips() {
        let original = MenuFile {
            title: "Custom \"Quoted\" Menu".to_string(),
            options: vec![
                MenuOption {
                    key: "1".to_string(),
                    command: "FILES".to_string(),
                    description: "Files".to_string(),
                    enabled: true,
                    group: Some("Core".to_string()),
                    show_in_menu_bar: true,
                    target: None,
                },
                MenuOption {
                    key: "2".to_string(),
                    command: "PLUGINS".to_string(),
                    description: "Plugins".to_string(),
                    enabled: false,
                    group: Some("Extended".to_string()),
                    // CR-NR-080: non-default (false) must round-trip.
                    show_in_menu_bar: false,
                    target: None,
                },
            ],
            show_calendar: false,
            group_separator: GroupSeparator::Line,
            group_headers: true,
        };
        let toml = serialise(&original);
        let parsed = parse_menu_str(&toml).expect("serialised menu must parse");
        assert_eq!(parsed, original);
    }

    // Validates: Requirement 13.8 -- default-valued display settings and the
    // enabled flag are omitted (clean output), but still parse back to defaults.
    #[test]
    fn defaults_are_omitted_but_parse_back() {
        let menu = MenuFile {
            title: "M".to_string(),
            options: vec![MenuOption {
                key: "A".to_string(),
                command: "A".to_string(),
                description: "All".to_string(),
                enabled: true,
                group: None,
                show_in_menu_bar: true,
                target: None,
            }],
            show_calendar: true,
            group_separator: GroupSeparator::Space,
            group_headers: false,
        };
        let toml = serialise(&menu);
        assert!(
            !toml.contains("show_calendar"),
            "default show_calendar omitted"
        );
        assert!(
            !toml.contains("group_separator"),
            "default separator omitted"
        );
        assert!(!toml.contains("group_headers"), "default headers omitted");
        assert!(!toml.contains("enabled"), "default enabled omitted");
        assert!(!toml.contains("group ="), "absent group omitted");
        assert!(
            !toml.contains("show_in_menu_bar"),
            "default show_in_menu_bar (true) omitted"
        );
        let parsed = parse_menu_str(&toml).expect("parse");
        assert_eq!(parsed, menu);
    }

    // Validates: Requirement 13.8 -- a title with quotes/backslashes is escaped
    // and round-trips.
    #[test]
    fn special_characters_in_title_escaped() {
        let menu = MenuFile {
            title: r#"back\slash and "quote""#.to_string(),
            options: vec![MenuOption {
                key: "X".to_string(),
                command: "RETURN".to_string(),
                description: "Return".to_string(),
                enabled: true,
                group: None,
                show_in_menu_bar: true,
                target: None,
            }],
            show_calendar: true,
            group_separator: GroupSeparator::Space,
            group_headers: false,
        };
        let toml = serialise(&menu);
        let parsed = parse_menu_str(&toml).expect("parse");
        assert_eq!(parsed.title, menu.title);
    }
}
