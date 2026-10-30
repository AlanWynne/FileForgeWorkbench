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
/// Contexts REMAIN reachable when typed (owner-approved Option B); only the
/// default MENU options are removed. Survivor keys `0`/`3`/`X` are unchanged.
///
/// Selecting an option dispatches its `command` string (config-driven, no
/// behaviour keyed to the key character). A single group means no stray
/// separator is drawn.
///
/// Validates: Requirement 12.1, 12.2, 12.7, 2.1h (menu-workspace)
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

/// Compiled Recovery_Baseline content for the default Menu_Bar (menu-workspace
/// Req 17, CR-NR-080; screen-snapshot-scrm Req 21, CR-NR-098).
///
/// CODE-ONLY, never written to disk. The bar mirrors the barebones POM's
/// bar-visible core (`Settings` / `Help`, with the terminate `X` option retained
/// but hidden via `show_in_menu_bar = false`) and ADDS one SCRM entry so the
/// already-built snapshot/capture family is discoverable from the menu bar
/// (FORWARD-PLAN resolved decision 1).
///
/// CR-CH-073 invariant: this is a SEPARATE constant from [`DEFAULT_POM_TOML`], so
/// the bare-core POM stays EXACTLY `0` Settings / `3` Help / `X` and is NOT
/// re-expanded by the SCRM surfacing. The SCRM entry lives ONLY on the bar.
///
/// The `SNAPSHOT` entry is a DYNAMIC source (see
/// `shell::render_menu_bar::dynamic_menu_options`): its dropdown children are the
/// existing `SNAPSHOT <format>` / `CAPTURE ...` verbs, each dispatched through the
/// single front door (command parity, Req 21.3/21.4). No new verb, intercept,
/// dispatcher, or navigation stack is introduced (framework-conformance).
///
/// Validates: menu-workspace Requirement 17.1, 17.2; screen-snapshot-scrm
/// Requirement 21.1, 21.2, 21.3 (CR-NR-098)
pub const DEFAULT_MENU_BAR_TOML: &str = r#"title = "FileForge Workbench -- Menu Bar"

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
command = "SNAPSHOT"
description = "Screen snapshot / capture (SCRM)"
group = "Core"

[[options]]
key = "X"
command = "X"
description = "Collapse to the visual root / close workspace (exit when last)"
group = "Core"
show_in_menu_bar = false
"#;

/// The default Menu_Bar NAME (menu-workspace Req 17.8, CR-NR-080). A user file
/// at `menus/<slug>.toml` (slug = `mb-pom`) OVERRIDES the compiled default bar;
/// the `MB-` prefix is a naming CONVENTION, not an enforced rule. Slice C will
/// let a workspace kind select a different bar name; until then this is the
/// single default the shell resolves.
pub const DEFAULT_MENU_BAR_NAME: &str = "MB-POM";

/// Build the compiled default Menu_Bar `MenuFile` (menu-workspace Req 17,
/// CR-NR-080; screen-snapshot-scrm Req 21, CR-NR-098).
///
/// A Menu_Bar is a `MenuFile` rendered HORIZONTALLY (each top-level option is a
/// dropdown button). It parses [`DEFAULT_MENU_BAR_TOML`] so there is a SINGLE
/// source for the compiled bar content (Req 12.7 analogue). The horizontal
/// render skips options whose `show_in_menu_bar` is `false` (Req 17.2), so the
/// terminate option (key `X`, marked `show_in_menu_bar = false`) does NOT appear
/// on the bar, leaving Settings / Help (CR-CH-073 trimmed core; Catalogs/Files
/// removed) plus the SCRM `SNAPSHOT` entry (FORWARD-PLAN decision 1).
///
/// The bar is INTENTIONALLY distinct from the barebones POM
/// ([`recovery_pom_menu`]): the POM stays exactly `0` Settings / `3` Help / `X`
/// (CR-CH-073, do NOT re-expand), while the bar additionally surfaces SCRM. The
/// two therefore no longer share a source constant.
///
/// A top-level option's `command`, when it names a resolvable menu (e.g.
/// `SETTINGS`), is PEEKED as a dropdown (Req 17.3); when it names a DYNAMIC
/// source (`SNAPSHOT`, `THEME LIST`) its children are generated at runtime
/// (Req 17.10); otherwise the button dispatches the command directly (Req 17.4).
///
/// Validates: menu-workspace Requirement 17.1, 17.2; screen-snapshot-scrm
/// Requirement 21.1, 21.2 (CR-NR-098)
pub fn default_menubar_menu() -> crate::menu_workspace::MenuFile {
    crate::menu_workspace::loader::parse_menu_str(DEFAULT_MENU_BAR_TOML)
        .expect("compiled default Menu_Bar must parse")
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

    // Validates: Requirement 12.2 (menu-workspace, CR-CH-073) -- Recovery_Baseline
    // POM parses from the single compiled source and ships the reconsidered
    // trimmed core option set. Catalogs (key 1) and Files (key 2) are REMOVED
    // from the barebones core (reserved for the future extension
    // wf_21b74c6aa15ba293); the keys 0/3/X of the survivors are retained unchanged.
    #[test]
    fn recovery_pom_menu_has_barebones_options() {
        let menu = recovery_pom_menu();
        let keys: Vec<&str> = menu.options.iter().map(|o| o.key.as_str()).collect();
        // CR-CH-073: trimmed core = Settings (0) / Help (3) / terminate (X).
        assert_eq!(keys, vec!["0", "3", "X"]);
        let commands: Vec<&str> = menu.options.iter().map(|o| o.command.as_str()).collect();
        // CR-CH-052: the terminate option's command is the uniform `X` close
        // verb (collapse-to-visual-root / close-workspace), not `Return`.
        assert_eq!(commands, vec!["Settings", "Help", "X"]);
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

    // Validates: menu-workspace Req 17.1, 17.2 (CR-NR-080); screen-snapshot-scrm
    // Req 21.1, 21.2 (CR-NR-098) -- the default Menu_Bar is NOW a DISTINCT source
    // from the barebones POM: it adds the SCRM `SNAPSHOT` entry the POM must not
    // carry (CR-CH-073). So default_menubar_menu() must NOT equal
    // recovery_pom_menu().
    #[test]
    fn default_menubar_differs_from_barebones_pom_by_scrm_entry() {
        let bar = default_menubar_menu();
        let pom = recovery_pom_menu();
        assert_ne!(
            bar, pom,
            "the default Menu_Bar must differ from the POM (it adds SCRM; POM stays 0/3/X)"
        );
        // The POM itself is unchanged: still exactly Settings / Help / terminate.
        let pom_cmds: Vec<&str> = pom.options.iter().map(|o| o.command.as_str()).collect();
        assert_eq!(
            pom_cmds,
            vec!["Settings", "Help", "X"],
            "the bare-core POM must stay EXACTLY 0 Settings / 3 Help / X (CR-CH-073)"
        );
    }

    // Validates: menu-workspace Req 17.2a (CR-CH-073); screen-snapshot-scrm
    // Req 21.1 (CR-NR-098) -- the bar-visible options are Settings / Help / the
    // SCRM SNAPSHOT entry (terminate X excluded; Catalogs/Files removed).
    #[test]
    fn default_menubar_bar_visible_options_are_settings_help_snapshot() {
        let menu = default_menubar_menu();
        let bar_cmds: Vec<&str> = menu
            .options
            .iter()
            .filter(|o| o.show_in_menu_bar)
            .map(|o| o.command.as_str())
            .collect();
        assert_eq!(
            bar_cmds,
            vec!["Settings", "Help", "SNAPSHOT"],
            "bar-visible options must be Settings / Help / SNAPSHOT \
             (terminate X hidden; Catalogs/Files removed; SCRM surfaced per \
             FORWARD-PLAN decision 1)"
        );
        // The terminate option (key X, command X after CR-CH-052) is present in
        // the bar menu but hidden from the bar.
        let ret = menu
            .options
            .iter()
            .find(|o| o.key == "X")
            .expect("terminate (X) option present in the bar menu");
        assert!(
            !ret.show_in_menu_bar,
            "the terminate (X) option must be hidden from the menu bar (show_in_menu_bar = false)"
        );
    }

    // Validates: screen-snapshot-scrm Req 21.1, 21.2 (CR-NR-098, CR-CH-073) --
    // adding the SCRM entry to the BAR must NOT re-expand the bare-core POM: the
    // DEFAULT_POM_TOML still has exactly three options and no SNAPSHOT entry.
    #[test]
    fn scrm_surfacing_does_not_expand_the_pom() {
        let pom = recovery_pom_menu();
        assert_eq!(
            pom.options.len(),
            3,
            "POM must stay at 3 options (CR-CH-073)"
        );
        assert!(
            !pom.options.iter().any(|o| o.command == "SNAPSHOT"),
            "the POM must NOT carry the SCRM SNAPSHOT entry (it lives only on the bar)"
        );
        assert!(
            !DEFAULT_POM_TOML.contains("SNAPSHOT"),
            "DEFAULT_POM_TOML must not mention SNAPSHOT"
        );
    }

    // Validates: screen-snapshot-scrm Req 21.1, 21.3 (CR-NR-098) -- the compiled
    // default Menu_Bar carries a top-level SCRM entry whose command is the
    // existing SNAPSHOT verb (so it routes through the single front door).
    #[test]
    fn default_menubar_has_scrm_snapshot_entry() {
        let menu = default_menubar_menu();
        let scrm = menu
            .options
            .iter()
            .find(|o| o.command == "SNAPSHOT")
            .expect("default Menu_Bar must carry a SNAPSHOT entry");
        assert!(
            scrm.show_in_menu_bar,
            "the SCRM SNAPSHOT entry must be visible on the bar"
        );
    }

    // Validates: screen-snapshot-scrm Req 21.2 (CR-NR-098) -- DEFAULT_MENU_BAR_TOML
    // is valid ASCII TOML.
    #[test]
    fn default_menu_bar_toml_is_valid_ascii_toml() {
        assert!(
            DEFAULT_MENU_BAR_TOML.is_ascii(),
            "DEFAULT_MENU_BAR_TOML must use only ASCII characters"
        );
        let result: Result<toml::Value, _> = toml::from_str(DEFAULT_MENU_BAR_TOML);
        assert!(result.is_ok(), "DEFAULT_MENU_BAR_TOML must be valid TOML");
    }

    // Validates: menu-workspace Req 17.3 -- the Settings top-level option's
    // command names the Settings menu, so opening it can PEEK that menu.
    #[test]
    fn default_menubar_settings_option_commands_settings_menu() {
        let menu = default_menubar_menu();
        let settings = menu
            .options
            .iter()
            .find(|o| o.command == "Settings")
            .expect("Settings entry present");
        assert_eq!(settings.command, "Settings");
    }

    // Validates: Requirement 7.4 (cv-requirements.md) -- DEFAULT_POM_TOML is valid TOML
    #[test]
    fn default_pom_toml_is_valid_toml() {
        let result: Result<toml::Value, _> = toml::from_str(DEFAULT_POM_TOML);
        assert!(result.is_ok(), "DEFAULT_POM_TOML must be valid TOML");
    }

    // Validates: Requirement 12.2, 2.1h (menu-workspace, CR-CH-073) -- the
    // compiled Recovery_Baseline POM ships EXACTLY the reconsidered trimmed core
    // option set (Settings / Help / terminate) whose commands the shell resolves
    // by name. Catalogs/Files removed from the barebones core.
    #[test]
    fn default_pom_toml_has_recovery_baseline_options() {
        let val: toml::Value = toml::from_str(DEFAULT_POM_TOML).expect("valid TOML");
        let options = val
            .get("options")
            .and_then(|v| v.as_array())
            .expect("options array");
        assert_eq!(
            options.len(),
            3,
            "Recovery_Baseline POM must contain exactly 3 options (CR-CH-073 trim)"
        );
        // CR-CH-073: trimmed core = Settings / Help / terminate.
        let commands: Vec<&str> = options
            .iter()
            .filter_map(|o| o.get("command").and_then(|c| c.as_str()))
            .collect();
        // CR-CH-052: terminate option command is `X`, not `Return`.
        assert_eq!(commands, vec!["Settings", "Help", "X"],);
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
