//! Full-shell locale-aware menu + resolver behaviour tests (CR-NR-109 Task
//! 11.6). These drive the REAL `WorkbenchShell` headlessly (egui_kittest
//! `build_eframe` via `harness_shell`) so the Req 11.6 single path -- a menu
//! option's command column shows the active-locale alias surface, while
//! selecting it still dispatches and RECORDS the ROOT -- is exercised end to
//! end, and the `en` behaviour-neutral baseline (Req 10.2) is pinned.
//!
//! The existing menu-kind first-Tab focus test
//! (`tests_focus::full_shell_first_tab_focuses_reported_first_interior`) stays
//! GREEN: these tests do not touch the focus latch.

use super::tests_common::{harness_shell, install_verb_find_soek_catalogue, overlay_alias_row};
use crate::shell::environment_registry::FFCMD_NAME;

/// Build a temp `menus/` dir with a `locmenu.toml` carrying a single enabled
/// `FIND` option (calendar off), returning the TempDir (kept alive by the
/// caller) and the menu name to open.
fn write_find_menu() -> (tempfile::TempDir, &'static str) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let menus = dir.path().join("menus");
    std::fs::create_dir_all(&menus).expect("create menus dir");
    std::fs::write(
        menus.join("locmenu.toml"),
        "title = \"Loc Menu\"\nshow_calendar = false\n\
         [[options]]\nkey = \"1\"\ncommand = \"FIND\"\ndescription = \"Find text\"\n",
    )
    .expect("write menu toml");
    (dir, "locmenu")
}

// Validates: localization Requirement 11.7, 11.9 -- with an af overlay
// SOEK -> FIND (key verb-find) on the FFCMD table and a test catalogue defining
// verb-find = SOEK, the menu option's command column shows the localised surface
// SOEK; activating the option (typing SOEK on the command line, the FFCMD seam)
// dispatches and RECORDS the ROOT command FIND -- the single dispatched/persisted
// identity.
#[test]
fn full_shell_menu_option_shows_localised_surface_and_dispatches_root() {
    let (_menus_dir, menu_name) = write_find_menu();
    let mut harness = harness_shell();

    // Point the shell at the temp menus dir and open the FIND menu as the active
    // Workspace (FFCMD is the active environment for a menu Context).
    harness.state_mut().dir_overrides.menus = Some(_menus_dir.path().join("menus"));
    harness.state_mut().open_menu_by_name(menu_name);
    for _ in 0..4 {
        harness.run();
    }

    // Install the non-en surface: FFCMD table SOEK -> FIND (key verb-find) and a
    // catalogue resolving verb-find = SOEK. Order: overlay first (needs the key),
    // then install the catalogue so t(verb-find) = SOEK.
    overlay_alias_row(harness.state(), FFCMD_NAME, "SOEK", "FIND");
    install_verb_find_soek_catalogue();

    // Req 11.7: the computed command-column surface for the FIND option is the
    // localised surface SOEK, not the raw root.
    let surfaces = harness.state().menu_option_command_surfaces();
    assert_eq!(
        surfaces,
        vec!["SOEK".to_string()],
        "the FIND option's command column must show the active-locale surface SOEK (Req 11.7)"
    );

    // Req 11.9: typing the localised surface SOEK on the command line dispatches
    // and RECORDS the ROOT command FIND (the resolver rewrites the verb to its
    // root before the prelude records it).
    harness.state_mut().dispatch_command_string("SOEK");
    assert_eq!(
        harness.state().command_line_history.most_recent(),
        Some("FIND"),
        "the ROOT command FIND must be recorded, not the surface SOEK (Req 11.9)"
    );
}

// Validates: localization Requirement 10.2 -- with no overlay and the default en
// catalogue, the menu command column shows the ROOT verb unchanged and a typed
// root dispatches/records unchanged (behaviour-neutral baseline).
#[test]
fn full_shell_menu_option_en_shows_root_verb_unchanged() {
    let (_menus_dir, menu_name) = write_find_menu();
    let mut harness = harness_shell();
    harness.state_mut().dir_overrides.menus = Some(_menus_dir.path().join("menus"));
    harness.state_mut().open_menu_by_name(menu_name);
    for _ in 0..4 {
        harness.run();
    }

    // en / no overlay: the FFCMD table is identity-only, so the FIND option's
    // command column shows the raw root FIND.
    let surfaces = harness.state().menu_option_command_surfaces();
    assert_eq!(
        surfaces,
        vec!["FIND".to_string()],
        "at en the command column shows the root verb unchanged (Req 10.2)"
    );

    // A typed root dispatches/records unchanged (verb == root at en).
    harness.state_mut().dispatch_command_string("FIND");
    assert_eq!(
        harness.state().command_line_history.most_recent(),
        Some("FIND"),
        "at en the root command is recorded unchanged (Req 10.2)"
    );
}

// Validates: localization Requirement 11.7, menu-and-statusbar Req 16.3 -- the
// menu-kind first-Tab focus contract is UNCHANGED by the localised command
// column: with an active-locale surface in effect, the POM still lands the first
// Tab on its reported first interior control (no phantom stop). This guards that
// the command-column change did not disturb the focus latch.
#[test]
fn full_shell_localised_menu_first_tab_still_focuses_first_interior() {
    use super::tests_common::cmd_field_id;
    let mut harness = harness_shell();
    // Apply a localised surface to the live FFCMD store + catalogue while the POM
    // is the active menu.
    overlay_alias_row(harness.state(), FFCMD_NAME, "KATALOGUSSE", "CATALOGS");
    install_verb_find_soek_catalogue();
    for _ in 0..2 {
        harness.run();
    }
    let expected = harness.state().focus.first_interior_id;
    assert!(
        expected.is_some(),
        "the POM must still report a first interior control with a localised column"
    );
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "command field holds focus on entering the POM"
    );
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        expected,
        "first Tab must still focus the reported first interior, not a phantom stop"
    );
}
