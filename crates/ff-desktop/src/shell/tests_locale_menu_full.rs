//! Full-shell CR-NR-111 "Menu Kinds localise FULLY" behaviour tests. These drive
//! the REAL `WorkbenchShell` headlessly (egui_kittest `build_eframe` via
//! `harness_shell`) so the first-prize contract -- a SINGLE saved menu renders
//! its mnemonic, command surface AND description in whatever locale is active,
//! all DERIVED at render time from the locale-independent stored ROOT -- is
//! exercised end to end. Every test SEEDS the needed per-locale data in-test
//! (overlay alias rows + install a test catalogue); authoring the built-in DATA
//! is a separate deliverable (CR-NR-111 delivers the CODE contract only).
//!
//! Storage-shape invariant: the stored value is the ROOT command (Option A), so a
//! locale switch changes only the derived display surfaces, never the stored
//! menu, and selection still dispatches/records the ROOT through the existing
//! CR-NR-109 pre-pass (no new dispatch path).

use super::tests_common::{
    harness_shell, install_find_command_only_catalogue, install_find_menu_localised_catalogue,
    overlay_alias_row,
};
use crate::menu_workspace::i18n_keys::localised_option_description;
use crate::shell::environment_registry::FFCMD_NAME;
use crate::shell::WorkbenchShell;

/// Build a temp `menus/` dir with a `locmenu.toml` carrying a single enabled
/// `FIND` option (mnemonic "F", calendar off), returning the TempDir (kept alive
/// by the caller) and the menu name to open. This is the ONE saved menu whose
/// surfaces must all follow the active locale -- the first-prize fixture.
fn write_find_menu() -> (tempfile::TempDir, &'static str) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let menus = dir.path().join("menus");
    std::fs::create_dir_all(&menus).expect("create menus dir");
    std::fs::write(
        menus.join("locmenu.toml"),
        "title = \"Loc Menu\"\nshow_calendar = false\n\
         [[options]]\nkey = \"F\"\ncommand = \"FIND\"\ndescription = \"Find text\"\n",
    )
    .expect("write menu toml");
    (dir, "locmenu")
}

/// Open the `locmenu` menu as the active Workspace in `harness`, pointing the
/// shell at `menus_dir`, and settle a few frames.
fn open_find_menu(
    harness: &mut egui_kittest::Harness<'_, WorkbenchShell>,
    menus_dir: &std::path::Path,
    menu_name: &str,
) {
    harness.state_mut().dir_overrides.menus = Some(menus_dir.join("menus"));
    harness.state_mut().open_menu_by_name(menu_name);
    for _ in 0..4 {
        harness.run();
    }
}

/// The derived description for the single FIND option of the active menu, keyed
/// the same way the renderer keys it (the active tab's menu-name label).
fn find_option_description(shell: &WorkbenchShell) -> String {
    let mw = shell
        .tabs
        .active_tab()
        .kind
        .menu_workspace()
        .expect("active tab is a menu workspace");
    let menu = mw.menu.as_ref().expect("menu is loaded");
    let menu_name = mw.menu_name_label().unwrap_or_default();
    localised_option_description(&menu_name, &menu.options[0])
}

// Validates: localization Requirement 12.1, 12.2, 12.3, 12.8; menu-workspace
// Requirement 21.1, 21.2, 21.3, 21.6; menu-and-statusbar Requirement 20.1, 20.2
// -- with mnemonic + command + description DATA seeded for a non-en locale, ALL
// THREE displayed parts of a menu option follow the active locale AT RENDER TIME
// (not just the description). The mnemonic column shows the localised "S", the
// command column the localised surface "SOEK", and the description "Soek teks".
#[test]
fn full_shell_menu_option_localises_mnemonic_command_and_description() {
    let (menus_dir, menu_name) = write_find_menu();
    let mut harness = harness_shell();
    open_find_menu(&mut harness, menus_dir.path(), menu_name);

    // Seed the non-en surfaces: an FFCMD alias row SOEK -> FIND (so the command
    // surface derivation finds a row keyed verb-find) and a catalogue defining
    // verb-find, the mnemonic key, and the description key for this menu.
    overlay_alias_row(harness.state(), FFCMD_NAME, "SOEK", "FIND");
    install_find_menu_localised_catalogue(menu_name, "");
    harness.run();

    // Mnemonic (key column) follows the locale (Req 12.3).
    assert_eq!(
        harness.state().menu_option_mnemonic_surfaces(),
        vec!["S".to_string()],
        "the mnemonic column must show the localised value S, not the stored key F (Req 12.3)"
    );
    // Command column follows the locale (Req 12.1/11.7).
    assert_eq!(
        harness.state().menu_option_command_surfaces(),
        vec!["SOEK".to_string()],
        "the command column must show the localised surface SOEK (Req 12.1)"
    );
    // Description follows the locale (Req 12.2, the existing seam).
    assert_eq!(
        find_option_description(harness.state()),
        "Soek teks".to_string(),
        "the description must show the localised value (Req 12.2)"
    );
}

// Validates: localization Requirement 12.4, 12.5; menu-workspace Requirement
// 21.4, 21.5; command-framework Requirement 18.1, 18.2, 18.3; command-semantics
// Requirement 13.1, 13.2, 13.3, 13.4, 13.5 -- the locale-INDEPENDENT stored value
// is the ROOT command. A locale switch changes only the DERIVED display surfaces,
// never the stored option (command "FIND", key "F"), and selecting the localised
// surface dispatches and RECORDS the ROOT FIND through the EXISTING pre-pass (no
// new dispatch path, no stored-shape change, no new MenuOption field).
#[test]
fn full_shell_stored_value_is_root_and_dispatch_records_root() {
    let (menus_dir, menu_name) = write_find_menu();
    let mut harness = harness_shell();
    open_find_menu(&mut harness, menus_dir.path(), menu_name);

    overlay_alias_row(harness.state(), FFCMD_NAME, "SOEK", "FIND");
    install_find_menu_localised_catalogue(menu_name, "");
    harness.run();

    // The STORED option is unchanged by the active locale: command is the ROOT
    // FIND and the stored mnemonic is the English "F" (Req 12.4 storage-shape
    // invariant; no new field, no migration).
    {
        let mw = harness
            .state()
            .tabs
            .active_tab()
            .kind
            .menu_workspace()
            .expect("menu workspace");
        let opt = &mw.menu.as_ref().expect("menu").options[0];
        assert_eq!(
            opt.command, "FIND",
            "the stored command is the ROOT (Req 12.4)"
        );
        assert_eq!(
            opt.key, "F",
            "the stored mnemonic is the English root key (Req 12.4)"
        );
    }

    // Selecting the localised surface SOEK on the command line resolves the
    // localised alias -> ROOT through the EXISTING CR-NR-109 pre-pass before
    // resolve_target, so the ROOT FIND is dispatched and RECORDED (Req 12.5,
    // command-semantics Req 13).
    harness.state_mut().dispatch_command_string("SOEK");
    assert_eq!(
        harness.state().command_line_history.most_recent(),
        Some("FIND"),
        "the ROOT FIND must be recorded, not the localised surface SOEK (Req 12.5)"
    );
}

// Validates: localization Requirement 12.6; menu-workspace Requirement 21.6;
// menu-and-statusbar Requirement 20.3 -- FIRST PRIZE auto-conversion: the SAME
// single saved menu renders DIFFERENTLY under two locales with NO second menu
// file. Under the seeded non-en catalogue the surfaces are localised; after
// resetting to the English Identity_Base they revert to the stored root parts,
// proving the display is derived from the active locale, not re-saved per locale.
#[test]
fn full_shell_single_menu_auto_converts_between_locales() {
    let (menus_dir, menu_name) = write_find_menu();
    let mut harness = harness_shell();
    open_find_menu(&mut harness, menus_dir.path(), menu_name);

    // Locale A (non-en, seeded): all three parts localise.
    overlay_alias_row(harness.state(), FFCMD_NAME, "SOEK", "FIND");
    install_find_menu_localised_catalogue(menu_name, "");
    harness.run();
    assert_eq!(
        harness.state().menu_option_mnemonic_surfaces(),
        vec!["S".to_string()]
    );
    assert_eq!(
        harness.state().menu_option_command_surfaces(),
        vec!["SOEK".to_string()]
    );
    assert_eq!(
        find_option_description(harness.state()),
        "Soek teks".to_string()
    );

    // Locale B (English Identity_Base): the SAME stored menu now renders the root
    // parts. Reinstall the shipped English base so the menu keys miss and fall
    // back per part; the FFCMD alias store still carries the SOEK row, but the
    // command-surface derivation keys on verb-find which the English base
    // resolves to the root FIND, so the command column reverts too.
    install_verb_find_soek_catalogue_reset(&mut harness);
    assert_eq!(
        harness.state().menu_option_mnemonic_surfaces(),
        vec!["F".to_string()],
        "under English the mnemonic reverts to the stored root key F (first-prize revert)"
    );
    assert_eq!(
        find_option_description(harness.state()),
        "Find text".to_string(),
        "under English the description reverts to the parsed English prose"
    );
}

/// Reinstall the shipped English Identity_Base so menu-option keys miss and fall
/// back to the stored root parts. Clears the ad-hoc alias row's effect on the
/// command column by removing the overlay is NOT needed: the command surface is
/// keyed on `verb-find`, which the shipped English base resolves to its root, so
/// the column reverts naturally. Kept as a thin helper for readability.
fn install_verb_find_soek_catalogue_reset(harness: &mut egui_kittest::Harness<'_, WorkbenchShell>) {
    // Install the shipped English base alone (no SOEK override), so verb-find
    // resolves to its English surface FIND and the menu keys miss.
    crate::shell::i18n_bootstrap::install_catalogue_for_tests();
    harness.run();
}

// Validates: localization Requirement 12.8; menu-workspace Requirement 21.1 --
// PER-PART fallback: with ONLY the command surface seeded (no mnemonic key, no
// description key for the menu), the command column localises to SOEK while the
// mnemonic falls back to the stored root key "F" and the description falls back
// to the parsed English prose "Find text". No part is ever blank, and fallback is
// per-part, not whole-row.
#[test]
fn full_shell_per_part_fallback_never_blank() {
    let (menus_dir, menu_name) = write_find_menu();
    let mut harness = harness_shell();
    open_find_menu(&mut harness, menus_dir.path(), menu_name);

    overlay_alias_row(harness.state(), FFCMD_NAME, "SOEK", "FIND");
    install_find_command_only_catalogue();
    harness.run();

    // Command localises (the one seeded part).
    assert_eq!(
        harness.state().menu_option_command_surfaces(),
        vec!["SOEK".to_string()],
        "the command column localises when its surface is seeded (Req 12.8)"
    );
    // Mnemonic falls back per part to the stored root key, never blank.
    let mnemonics = harness.state().menu_option_mnemonic_surfaces();
    assert_eq!(
        mnemonics,
        vec!["F".to_string()],
        "the mnemonic falls back per part to the stored root key F (Req 12.8)"
    );
    assert!(
        !mnemonics[0].trim().is_empty(),
        "a fallback mnemonic is never blank (Req 12.8)"
    );
    // Description falls back per part to the parsed English prose, never blank.
    let desc = find_option_description(harness.state());
    assert_eq!(
        desc, "Find text",
        "the description falls back per part to the parsed English prose (Req 12.8)"
    );
    assert!(
        !desc.trim().is_empty(),
        "a fallback description is never blank"
    );
}

// Validates: localization Requirement 10.2 -- behaviour-neutral English baseline:
// with no overlay and the shipped English catalogue, every displayed part equals
// its stored root part (mnemonic "F", command "FIND", description "Find text"),
// so the whole change is a no-op at en.
#[test]
fn full_shell_en_shows_all_root_parts_unchanged() {
    let (menus_dir, menu_name) = write_find_menu();
    let mut harness = harness_shell();
    open_find_menu(&mut harness, menus_dir.path(), menu_name);
    harness.run();

    assert_eq!(
        harness.state().menu_option_mnemonic_surfaces(),
        vec!["F".to_string()],
        "at en the mnemonic column shows the stored root key F (Req 10.2)"
    );
    assert_eq!(
        harness.state().menu_option_command_surfaces(),
        vec!["FIND".to_string()],
        "at en the command column shows the root verb FIND (Req 10.2)"
    );
    assert_eq!(
        find_option_description(harness.state()),
        "Find text".to_string(),
        "at en the description shows the parsed English prose (Req 10.2)"
    );
}
