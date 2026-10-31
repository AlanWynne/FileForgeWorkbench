//! Mnemonic-letter menu-bar accelerator tests (screen-snapshot-scrm Req 26,
//! CR-NR-114).
//!
//! Covers the pure mnemonic derivation (`menu_bar_mnemonics`) and the full-shell
//! Alt+<letter> accelerator behaviour (opening the matching bar item's menu
//! through the single front-door dispatch seam).

#![allow(unused_imports)]
use super::tests_common::*;
use eframe::egui;

/// Validates: screen-snapshot-scrm Req 26.1, 26.3, 26.4 (CR-NR-114) -- the
/// mnemonic-letter accelerators for the bare POM-echo bar are Alt+S Settings,
/// Alt+H Help, Alt+C SCRM: a digit key (Settings=0) maps to the command's first
/// letter (S), SCRM's own key `S` COLLIDES with Settings so it is resolved to a
/// distinct letter (C, the next letter of "SCRM"), and no mnemonic overloads a
/// POM numeric key.
#[test]
fn menu_bar_mnemonics_for_bare_pom_are_distinct_letters() {
    use super::render_menu_bar::menu_bar_mnemonics;
    use crate::menu_workspace::defaults::recovery_pom_menu;
    let pom = recovery_pom_menu();
    let mnemonics = menu_bar_mnemonics(&pom.options);
    assert_eq!(
        mnemonics,
        vec![
            ('S', "Settings".to_string()),
            ('H', "Help".to_string()),
            ('C', "SCRM".to_string()),
        ],
        "bare POM mnemonics must be Alt+S Settings / Alt+H Help / Alt+C SCRM (Req 26.1/26.3)"
    );
    // Distinct letters (no collision, Req 26.3).
    let letters: std::collections::BTreeSet<char> = mnemonics.iter().map(|(c, _)| *c).collect();
    assert_eq!(
        letters.len(),
        mnemonics.len(),
        "every mnemonic must be distinct"
    );
    // No mnemonic is a POM numeric option key (Req 26.4): letters only.
    assert!(
        mnemonics.iter().all(|(c, _)| c.is_ascii_alphabetic()),
        "mnemonics must be letters, never POM numeric option keys (Req 26.4)"
    );
    // The hidden terminate option (X, show_in_menu_bar = false) gets no mnemonic.
    assert!(
        !mnemonics.iter().any(|(_, cmd)| cmd == "X"),
        "the bar-hidden terminate option must not receive a mnemonic"
    );
}

/// Validates: screen-snapshot-scrm Req 26.5, 22.7 (CR-NR-114) -- when an
/// Alt+<letter> accelerator fires it opens the matching bar item's menu through
/// the SAME front-door dispatch seam as a click. Driving the shell with an Alt+C
/// event (the SCRM mnemonic) opens the SCRM menu screen -- observably the same as
/// clicking the SCRM bar item or typing `SCRM`.
#[test]
fn full_shell_alt_mnemonic_opens_matching_bar_menu() {
    let mut harness = harness_shell();
    // Precondition: the active tab is the Home Context (POM), not the SCRM menu.
    assert!(
        harness.state().tabs.active_tab().is_home,
        "precondition: start on the POM"
    );
    // Fire Alt+C (SCRM mnemonic for the bare POM-echo bar).
    harness.key_press_modifiers(egui::Modifiers::ALT, egui::Key::C);
    harness.run();
    harness.run();
    // The SCRM menu screen is now the active menu (its title is the Req 25.1
    // title), reached through the same dispatch seam a click/typed `SCRM` uses.
    let title = harness
        .state()
        .tabs
        .active_tab()
        .kind
        .menu_workspace()
        .and_then(|mw| mw.menu_title());
    assert_eq!(
        title.as_deref(),
        Some("FileForge Workbench -- Screen Capture and Recording"),
        "Alt+C must open the SCRM menu screen via the front-door dispatch seam (Req 26.5)"
    );
}
