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
/// configuration is missing or corrupt can always reach Settings (and, through
/// it, the Menus / Keys / Theme / Kinds editors and Reset-bare) and Help to
/// rebuild or recover.
///
/// CR-CH-073: the barebones core was RECONSIDERED and TRIMMED to exactly three
/// options -- `0` Settings, `3` Help, `X` terminate. The former `1` Catalogs and
/// `2` Files options are REMOVED (reserved for a future extension,
/// wf_21b74c6aa15ba293). The underlying `CATALOGS` / `FILES` commands and their
/// Contexts REMAIN reachable when typed (owner-approved Option B).
///
/// CR-NR-114 / screen-snapshot-scrm Req 22.3 (OWNER-CONFIRMED FRAMEWORK CHANGE
/// (b)): the bare-core POM now ALSO carries an `SCRM` option
/// ("SCRM : Screen Capture and Recording"), in ADDITION to `0` Settings /
/// `3` Help / `X` terminate. This DELIBERATELY changes the former CR-CH-073
/// invariant (bare POM was EXACTLY `0`/`3`/`X`): because the menu bar is now a
/// RUNTIME ECHO of the POM (Req 22.1), SCRM-in-the-POM is what surfaces SCRM on
/// the bar -- there is no separate hardwired bar entry. The `SCRM` command
/// resolves to the code-only SCRM menu screen ([`DEFAULT_SCRM_MENU_TOML`],
/// Req 25), which becomes the SCRM bar item's dropdown.
///
/// Selecting an option dispatches its `command` string (config-driven, no
/// behaviour keyed to the key character). A single group means no stray
/// separator is drawn.
///
/// Validates: menu-workspace Requirement 12.1, 12.2, 12.7, 2.1h;
/// screen-snapshot-scrm Requirement 22.3 (CR-NR-114)
pub const DEFAULT_POM_TOML: &str = r#"title = "FileForge Workbench -- Primary Option Menu"

[[options]]
key = "0"
command = "Settings"
description = "FFWB Settings and Client Parameters"
group = "Core"

[[options]]
key = "3"
command = "Help"
description = "Help -- workbench help and topics"
group = "Core"

[[options]]
key = "S"
command = "SCRM"
description = "SCRM : Screen Capture and Recording"
group = "Core"

[[options]]
key = "X"
command = "X"
description = "Collapse to the visual root / close workspace (exit when last)"
group = "Core"
show_in_menu_bar = false
"#;

/// Compiled Recovery_Baseline content for the Settings menu (menu-workspace
/// Req 12, CR-CH-021).
///
/// CODE-ONLY barebones Settings: never written to disk. Fallback rendered when
/// no valid user `menus/settings.toml` exists. Minimal set: the Config browser,
/// the Alias editor, the Theme editor, the Menus editor, the Keys editor, the
/// Kinds editor -- enough to rebuild the rest -- plus the RESET BARE recovery
/// affordance.
///
/// CR-CH-066: Config moved from mnemonic `A` to `C` and a new `A` -> `Aliases`
/// (Alias Editor, CR-NR-113) option was inserted in SECOND position. New order:
/// `C` Config, `A` Aliases, `T` Theme, `M` Menus, `K` Keys, `W` Kinds (Core),
/// then `R` Reset bare (Recovery).
///
/// Validates: Requirement 12.1, 12.3, 12.7, 22.1, 22.4, 22.5 (menu-workspace)
pub const DEFAULT_SETTINGS_TOML: &str = r#"title = "Settings"
show_calendar = false
group_separator = "line"

[[options]]
key = "C"
command = "Config"
description = "All settings -- browse every configuration key"
group = "Core"

[[options]]
key = "A"
command = "Aliases"
description = "Alias editor -- create, edit and save command aliases"
group = "Core"

[[options]]
key = "T"
command = "Theme"
description = "Theme editor -- copy, edit, save and select themes"
group = "Core"

[[options]]
key = "M"
command = "Menus"
description = "Menus editor -- create, change and save menus"
group = "Core"

[[options]]
key = "K"
command = "Keys"
description = "Keys -- edit and save per-workspace key assignments"
group = "Core"

[[options]]
key = "W"
command = "Kinds"
description = "Workspace kinds -- configure title, menu bar, key list, profile"
group = "Core"

[[options]]
key = "R"
command = "Reset bare"
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

/// Compiled Recovery_Baseline content for the SCRM menu screen (screen-snapshot
/// -scrm Req 25, CR-NR-114).
///
/// CODE-ONLY, never written to disk (CR-CH-021): it is the fallback dropdown /
/// submenu the `SCRM` POM option opens, and the source RESET BARE regenerates
/// (Req 22.8, 25.5). A user `menus/scrm.toml` OVERRIDES it (the standard
/// user-override-of-default rule).
///
/// Every option's `command` is an EXISTING SCRM verb resolved through the single
/// front door (Req 22.6, 25.4): Snapshot -> `SNAPSHOT` (bare composite clipboard,
/// Req 4.1); Capture start -> `CAPTURE START`; Capture stop -> `CAPTURE STOP`
/// (the 2C Save-As dialog); Capture replay -> `CAPTURE REPLAY` (the 2C saved
/// -session picker); SCRM Help -> `HELP SCRM` (the SCRM help topic); Exit -> `X`
/// (the uniform FFCMD close/END verb). NO new verb is introduced.
///
/// Mnemonic keys are the owner-confirmed (settled-but-tunable) set from Req 25.3:
/// Snapshot `S`, Capture start `CS`, Capture stop `CT`, Capture replay `CR`,
/// SCRM Help `H`, Exit `X`.
///
/// Validates: screen-snapshot-scrm Requirement 25.1, 25.2, 25.3, 25.4 (CR-NR-114)
pub const DEFAULT_SCRM_MENU_TOML: &str = r#"title = "FileForge Workbench -- Screen Capture and Recording"

[[options]]
key = "S"
command = "SNAPSHOT"
description = "Snapshot -- copy the current screen to the clipboard"
group = "Core"

[[options]]
key = "CS"
command = "CAPTURE START"
description = "Capture start -- begin recording a capture session"
group = "Core"

[[options]]
key = "CT"
command = "CAPTURE STOP"
description = "Capture stop -- stop and save the capture session"
group = "Core"

[[options]]
key = "CR"
command = "CAPTURE REPLAY"
description = "Capture replay -- open a saved capture session"
group = "Core"

[[options]]
key = "H"
command = "HELP SCRM"
description = "SCRM Help -- screen capture and recording help"
group = "Core"

[[options]]
key = "X"
command = "X"
description = "Exit -- close this menu (collapse to the visual root)"
group = "Core"
"#;

/// Build the compiled Recovery_Baseline SCRM `MenuFile` (screen-snapshot-scrm
/// Req 25). Parses [`DEFAULT_SCRM_MENU_TOML`] (single content source).
///
/// Validates: screen-snapshot-scrm Requirement 25.1, 25.3, 25.4 (CR-NR-114)
pub fn recovery_scrm_menu() -> crate::menu_workspace::MenuFile {
    crate::menu_workspace::loader::parse_menu_str(DEFAULT_SCRM_MENU_TOML)
        .expect("compiled Recovery_Baseline SCRM menu must parse")
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
        // screen-snapshot-scrm Req 22.8 / 25.5 (CR-NR-114): the SCRM menu screen
        // is ALSO code-only and must never be materialised to disk by RESET BARE
        // / startup scaffolding.
        assert!(
            !menus_dir.join("scrm.toml").exists(),
            "scrm.toml must NOT be materialised (SCRM menu is code-only, Req 22.8/25.5)"
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

    // Validates: Requirement 12.2 (menu-workspace, CR-CH-073) -- Recovery_Baseline
    // POM parses from the single compiled source and ships the reconsidered
    // trimmed core option set. Catalogs (key 1) and Files (key 2) are REMOVED
    // from the barebones core (reserved for the future extension
    // wf_21b74c6aa15ba293); the keys 0/3/X of the survivors are retained unchanged.
    #[test]
    fn recovery_pom_menu_has_barebones_options() {
        let menu = recovery_pom_menu();
        let keys: Vec<&str> = menu.options.iter().map(|o| o.key.as_str()).collect();
        // CR-NR-114 (Req 22.3, UPDATED from the retired CR-CH-073 0/3/X invariant):
        // the bare core is now Settings (0) / Help (3) / SCRM (S) / terminate (X).
        assert_eq!(keys, vec!["0", "3", "S", "X"]);
        let commands: Vec<&str> = menu.options.iter().map(|o| o.command.as_str()).collect();
        // CR-CH-052: the terminate option's command is the uniform `X` close
        // verb (collapse-to-visual-root / close-workspace), not `Return`.
        assert_eq!(commands, vec!["Settings", "Help", "SCRM", "X"]);
        // Single group -> no stray separator boundary.
        let groups: std::collections::BTreeSet<&str> = menu
            .options
            .iter()
            .filter_map(|o| o.group.as_deref())
            .collect();
        assert_eq!(groups.len(), 1, "Recovery POM uses a single group");
    }

    // Validates: Requirement 12.1, 12.3, 12.7, 22.1 -- Recovery_Baseline Settings.
    #[test]
    fn recovery_settings_menu_has_barebones_options() {
        let menu = recovery_settings_menu();
        let keys: Vec<&str> = menu.options.iter().map(|o| o.key.as_str()).collect();
        // CR-CH-025 + CR-CH-029 + CR-NR-090 + CR-CH-066: ordered C Config /
        // A Aliases / T Theme / M Menus / K Keys / W Kinds (group Core), then
        // R Reset-to-barebones (group Recovery). Affordances dispatch via the
        // menu option = command path; `A -> ALIASES` opens the Alias Editor
        // (CR-NR-113), `K -> KEYS` opens the Keys Workspace (function-keys Req 22),
        // and `W -> KINDS` opens the Workspace Kinds editor (workspace-kinds Req 6).
        assert_eq!(keys, vec!["C", "A", "T", "M", "K", "W", "R"]);
        let commands: Vec<&str> = menu.options.iter().map(|o| o.command.as_str()).collect();
        assert_eq!(
            commands,
            vec![
                "Config",
                "Aliases",
                "Theme",
                "Menus",
                "Keys",
                "Kinds",
                "Reset bare"
            ]
        );
    }

    // Validates: screen-snapshot-scrm Req 22.3, 22.4 (CR-NR-114, UPDATED from the
    // retired CR-CH-073 0/3/X invariant) -- the menu bar is now a RUNTIME ECHO of
    // the POM (built in the shell, not a separate DEFAULT_MENU_BAR_TOML), so there
    // is no longer a distinct hardwired bar constant. The bare-core POM instead
    // CONTAINS the SCRM option, which is what surfaces SCRM on the bar.
    #[test]
    fn bare_pom_contains_scrm_and_stays_minimal() {
        let pom = recovery_pom_menu();
        let cmds: Vec<&str> = pom.options.iter().map(|o| o.command.as_str()).collect();
        // Req 22.3: bare POM is now Settings / Help / SCRM / terminate (the former
        // EXACTLY-0/3/X invariant is DELIBERATELY changed, owner-confirmed (b)).
        assert_eq!(
            cmds,
            vec!["Settings", "Help", "SCRM", "X"],
            "the bare-core POM must be Settings / Help / SCRM / terminate (Req 22.3)"
        );
    }

    // Validates: screen-snapshot-scrm Req 22.3, 22.4 (CR-NR-114) -- the SCRM POM
    // option is bar-visible (so it echoes onto the bar) and its command is the
    // `SCRM` verb that opens the SCRM menu screen (Req 25), while the terminate
    // X option stays hidden from the bar.
    #[test]
    fn bare_pom_scrm_option_is_bar_visible_and_commands_scrm_menu() {
        let pom = recovery_pom_menu();
        let scrm = pom
            .options
            .iter()
            .find(|o| o.command == "SCRM")
            .expect("bare POM must carry the SCRM option (Req 22.3)");
        assert!(
            scrm.show_in_menu_bar,
            "the SCRM option must be bar-visible so it echoes onto the menu bar"
        );
        assert_eq!(
            scrm.description, "SCRM : Screen Capture and Recording",
            "the SCRM option label/description must match Req 25.2"
        );
        // The terminate option (key X) stays hidden from the bar.
        let x = pom
            .options
            .iter()
            .find(|o| o.key == "X")
            .expect("terminate (X) option present");
        assert!(
            !x.show_in_menu_bar,
            "the terminate (X) option must stay hidden from the menu bar"
        );
    }

    // Validates: screen-snapshot-scrm Req 25.1, 25.3, 25.4 (CR-NR-114) -- the
    // code-only SCRM menu screen has the Req 25.1 title and the Req 25.3 options
    // (Snapshot=S / Capture start=CS / Capture stop=CT / Capture replay=CR /
    // SCRM Help=H / Exit=X), each dispatching an EXISTING SCRM verb (Req 25.4).
    #[test]
    fn recovery_scrm_menu_has_owner_confirmed_keys_and_verbs() {
        let menu = recovery_scrm_menu();
        assert_eq!(
            menu.title, "FileForge Workbench -- Screen Capture and Recording",
            "SCRM menu title must match Req 25.1"
        );
        let pairs: Vec<(&str, &str)> = menu
            .options
            .iter()
            .map(|o| (o.key.as_str(), o.command.as_str()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("S", "SNAPSHOT"),
                ("CS", "CAPTURE START"),
                ("CT", "CAPTURE STOP"),
                ("CR", "CAPTURE REPLAY"),
                ("H", "HELP SCRM"),
                ("X", "X"),
            ],
            "SCRM menu keys/commands must match Req 25.3/25.4"
        );
    }

    // Validates: screen-snapshot-scrm Req 25 (CR-NR-114) -- DEFAULT_SCRM_MENU_TOML
    // is valid ASCII TOML.
    #[test]
    fn default_scrm_menu_toml_is_valid_ascii_toml() {
        assert!(
            DEFAULT_SCRM_MENU_TOML.is_ascii(),
            "DEFAULT_SCRM_MENU_TOML must use only ASCII characters"
        );
        let result: Result<toml::Value, _> = toml::from_str(DEFAULT_SCRM_MENU_TOML);
        assert!(result.is_ok(), "DEFAULT_SCRM_MENU_TOML must be valid TOML");
    }

    // Validates: Requirement 7.4 (cv-requirements.md) -- DEFAULT_POM_TOML is valid TOML
    #[test]
    fn default_pom_toml_is_valid_toml() {
        let result: Result<toml::Value, _> = toml::from_str(DEFAULT_POM_TOML);
        assert!(result.is_ok(), "DEFAULT_POM_TOML must be valid TOML");
    }

    // Validates: menu-workspace Requirement 12.2, 2.1h; screen-snapshot-scrm
    // Requirement 22.3 (CR-NR-114, UPDATED from CR-CH-073) -- the compiled
    // Recovery_Baseline POM ships Settings / Help / SCRM / terminate (Catalogs/
    // Files remain removed; SCRM added by Req 22.3), whose commands the shell
    // resolves by name.
    #[test]
    fn default_pom_toml_has_recovery_baseline_options() {
        let val: toml::Value = toml::from_str(DEFAULT_POM_TOML).expect("valid TOML");
        let options = val
            .get("options")
            .and_then(|v| v.as_array())
            .expect("options array");
        assert_eq!(
            options.len(),
            4,
            "bare-core POM now has 4 options: Settings / Help / SCRM / terminate (Req 22.3)"
        );
        // CR-NR-114 (Req 22.3): bare core = Settings / Help / SCRM / terminate.
        let commands: Vec<&str> = options
            .iter()
            .filter_map(|o| o.get("command").and_then(|c| c.as_str()))
            .collect();
        // CR-CH-052: terminate option command is `X`, not `Return`.
        assert_eq!(commands, vec!["Settings", "Help", "SCRM", "X"],);
    }

    // Validates: Requirement 2.1g (menu-workspace, revised CR-CH-052) -- the POM
    // terminate option is the data-driven `X` -> `X` close verb (collapse to the
    // visual root / close workspace, exit when last), not the former `X -> Return`.
    #[test]
    fn default_pom_toml_terminate_is_x_x() {
        let val: toml::Value = toml::from_str(DEFAULT_POM_TOML).expect("valid TOML");
        let options = val
            .get("options")
            .and_then(|v| v.as_array())
            .expect("options array");
        let x = options
            .iter()
            .find(|o| o.get("key").and_then(|k| k.as_str()) == Some("X"))
            .expect("X option present");
        assert_eq!(x.get("command").and_then(|c| c.as_str()), Some("X"));
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

    // Validates: Requirement 12.3, 22.1 (menu-workspace, CR-CH-021 + CR-CH-066)
    // -- Recovery_Baseline Settings: C Config / A Aliases / T Theme / M Menus /
    // K Keys / W Kinds (Core), plus the R RESET BARE affordance (dispatched via
    // the menu-option = command path, configuration-system Req 19.7).
    #[test]
    fn default_settings_toml_has_recovery_baseline_options() {
        let val: toml::Value = toml::from_str(DEFAULT_SETTINGS_TOML).expect("valid TOML");
        let options = val
            .get("options")
            .and_then(|v| v.as_array())
            .expect("options array");
        assert_eq!(
            options.len(),
            7,
            "Recovery_Baseline Settings: C/A/T/M/K/W + the RESET BARE affordance"
        );
        // CR-CH-025 + CR-CH-029 + CR-NR-090 + CR-CH-066: ordered C CONFIG,
        // A ALIASES, T THEME, M MENUS, K KEYS, W KINDS (group Core), then R RESET
        // BARE (group Recovery).
        let keys: Vec<&str> = options
            .iter()
            .filter_map(|o| o.get("key").and_then(|c| c.as_str()))
            .collect();
        assert_eq!(keys, vec!["C", "A", "T", "M", "K", "W", "R"]);
        let commands: Vec<&str> = options
            .iter()
            .filter_map(|o| o.get("command").and_then(|c| c.as_str()))
            .collect();
        assert_eq!(
            commands,
            vec![
                "Config",
                "Aliases",
                "Theme",
                "Menus",
                "Keys",
                "Kinds",
                "Reset bare"
            ]
        );
        let groups: Vec<&str> = options
            .iter()
            .filter_map(|o| o.get("group").and_then(|c| c.as_str()))
            .collect();
        assert_eq!(
            groups,
            vec!["Core", "Core", "Core", "Core", "Core", "Core", "Recovery"]
        );
    }

    // Validates: menu-workspace Requirement 22.1, 22.5 (CR-CH-066) -- the
    // compiled Settings Recovery_Baseline is the single source of the Settings
    // option list and presents, in exact order with exact mnemonics, `C` CONFIG,
    // `A` ALIASES (Alias Editor), `T` THEME, `M` MENUS, `K` KEYS, `W` KINDS
    // (group Core), then `R` RESET BARE (group Recovery). Config's mnemonic is
    // `C` (freed `A` for the Alias Editor) -- a deliberate change, no existing
    // users.
    #[test]
    fn default_settings_toml_has_alias_editor_and_config_mnemonic_c() {
        let menu = recovery_settings_menu();
        let pairs: Vec<(&str, &str)> = menu
            .options
            .iter()
            .map(|o| (o.key.as_str(), o.command.as_str()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("C", "Config"),
                ("A", "Aliases"),
                ("T", "Theme"),
                ("M", "Menus"),
                ("K", "Keys"),
                ("W", "Kinds"),
                ("R", "Reset bare"),
            ],
            "Settings must present C Config / A Aliases / T Theme / M Menus / \
             K Keys / W Kinds / R Reset bare in that exact order (CR-CH-066)"
        );
        // Config is mnemonic C; A is the Alias Editor (ALIASES verb).
        let config = menu
            .options
            .iter()
            .find(|o| o.command.eq_ignore_ascii_case("Config"))
            .expect("Config option present");
        assert_eq!(config.key, "C", "Config must use mnemonic C (CR-CH-066)");
        let alias = menu
            .options
            .iter()
            .find(|o| o.key == "A")
            .expect("A option present");
        assert_eq!(
            alias.command, "Aliases",
            "the A option must dispatch the Aliases (ALIASES) verb (CR-NR-113)"
        );
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
