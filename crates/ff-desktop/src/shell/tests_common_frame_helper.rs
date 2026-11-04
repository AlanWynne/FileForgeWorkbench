//! Unit tests for the shared common-frame chrome helper
//! (`shell::common_frame::apply_common_frame`, CR-NR-115 Phase 2H,
//! screen-snapshot-scrm Requirement 30). The helper composes the eight common
//! frame elements onto a per-Context `ScreenModel` from LIVE logical state
//! (never egui): the window title bar, the menu-bar echo, the FULL tab bar (all
//! open tabs' `[XXX]` tags + an ASCII close marker), the common+per-Kind
//! Status_Bar (no RUNNING), the Message_Line (above the status bar, omitted when
//! empty), and the pfshow key bar (only when visible).

#![allow(unused_imports)]
use super::tests_common::*;
use crate::tab_state::KindTag;
use ff_screen_model::ScreenModel;

/// Open the Alias editor (a table Context) through the real front door.
fn alias_shell() -> super::WorkbenchShell {
    let mut shell = make_shell();
    shell.dispatch_command_string("ALIASES");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "precondition: ALIASES opens the Alias Editor"
    );
    shell
}

// Validates: screen-snapshot-scrm Requirement 30.2, 30.3 -- apply_common_frame
// prepends the window title bar (the ONE shared 'FileForge Workbench' constant +
// the ASCII window controls) and the live menu-bar echo as header lines.
#[test]
fn common_frame_adds_window_title_and_menu_bar_echo() {
    let shell = alias_shell();
    let mut model = ScreenModel::new("Alias Editor");
    shell.apply_common_frame(&mut model);

    assert!(
        model
            .header_lines
            .iter()
            .any(|l| l.contains(crate::screen_snapshot::WINDOW_TITLE)),
        "a window-title-bar header line carries the shared title; got {:?}",
        model.header_lines
    );
    // The window controls are ASCII (documentation-charset): underscore / [] / X.
    assert!(
        model
            .header_lines
            .iter()
            .any(|l| l.contains(crate::screen_snapshot::WINDOW_TITLE) && l.contains("[] X")),
        "the window title bar carries ASCII window controls; got {:?}",
        model.header_lines
    );
    // The menu-bar echo lists the bar-visible POM options (e.g. SETTINGS, HELP,
    // SCRM) joined by ' | '. At least SETTINGS is always bar-visible.
    assert!(
        model
            .header_lines
            .iter()
            .any(|l| l.contains(" | ") && l.to_uppercase().contains("SETTINGS")),
        "a menu-bar echo header line lists the bar options; got {:?}",
        model.header_lines
    );
}

// Validates: screen-snapshot-scrm Requirement 30.4 -- the FULL tab bar is a
// header line listing EVERY open tab's [XXX] tag (not just the active one) plus
// an ASCII 'x' close marker.
#[test]
fn common_frame_adds_full_tab_bar_with_all_tags() {
    // Build a deterministic TWO-tab shell: a home POM tab ([POM]) plus a new
    // untitled tab ([EDITOR]), so the full tab bar must list BOTH tags.
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.tabs.new_untitled_tab(&shell.runtime);
    assert!(
        shell.tabs.len() >= 2,
        "precondition: at least two tabs open"
    );

    let mut model = ScreenModel::new("Untitled");
    shell.apply_common_frame(&mut model);

    let tab_bar = model
        .header_lines
        .iter()
        .find(|l| l.contains("[POM]"))
        .expect("a tab-bar header line carries the home [POM] tag");
    // The FULL bar lists ALL tags, including the non-active ones, proving it is
    // not active-tag-only (the untitled tab maps to the [EDITOR] base tag).
    assert!(
        tab_bar.contains("[EDITOR]"),
        "the tab bar lists the second tab tag too; got {tab_bar:?}"
    );
    // An ASCII close marker 'x' accompanies the tabs.
    assert!(
        tab_bar.contains('x'),
        "the tab bar carries an ASCII close marker; got {tab_bar:?}"
    );
}

// Validates: screen-snapshot-scrm Requirement 30.10 (revised 2026-11-04) and
// workspace-framework Requirement 7.16-7.19 -- the ONLY shell-common Status_Bar
// segments are Last and Prof; a settings-type Workspace (Alias) carries NO
// Ln/Col, encoding or line-count segment (those are workspace-owned, Req 7.17,
// 7.21) and NO RUNNING segment.
#[test]
fn common_frame_status_bar_has_common_segments_and_no_running() {
    let shell = alias_shell();
    let mut model = ScreenModel::new("Alias Editor");
    shell.apply_common_frame(&mut model);

    let status = &model.status_bar.text;
    assert!(
        status.contains("Last:"),
        "Last segment present; got {status:?}"
    );
    assert!(
        status.contains("Prof:"),
        "Prof segment present; got {status:?}"
    );
    for gone in ["Ln ", "Col ", "UTF-8", "Lines", "RUNNING"] {
        assert!(
            !status.contains(gone),
            "{gone:?} must not appear on a settings-type Workspace; got {status:?}"
        );
    }
}

// Validates: screen-snapshot-scrm Requirement 30.8 -- the Message_Line is
// composed above the status bar when present and OMITTED when empty.
#[test]
fn common_frame_message_line_present_when_set_and_omitted_when_empty() {
    // Empty store: no message rows added.
    let shell = alias_shell();
    let mut empty = ScreenModel::new("Alias Editor");
    let before = empty.messages.len();
    shell.apply_common_frame(&mut empty);
    assert_eq!(
        empty.messages.len(),
        before,
        "no Message_Line rows are added when the store is empty"
    );

    // Populated store: each message is carried.
    let mut shell2 = alias_shell();
    shell2.message_line = vec!["info one".to_string(), "info two".to_string()];
    let mut model = ScreenModel::new("Alias Editor");
    shell2.apply_common_frame(&mut model);
    assert!(
        model.messages.iter().any(|m| m == "info one"),
        "the first Message_Line entry is carried; got {:?}",
        model.messages
    );
    assert!(
        model.messages.iter().any(|m| m == "info two"),
        "the second Message_Line entry is carried; got {:?}",
        model.messages
    );
}

// Validates: screen-snapshot-scrm Requirement 30.9, 30.10 -- the pfshow key bar
// is a footer line ONLY when the Key_Label_Bar is visible, and is OMITTED when
// pfshow is off (so the capture reflects the live pfshow state).
#[test]
fn common_frame_key_bar_follows_pfshow_for_table_context() {
    // pfshow ON: a key-bar footer line is present.
    let mut on = alias_shell();
    on.handle_pfshow("ON");
    assert!(on.key_bar_visible, "precondition: pfshow ON");
    let mut model_on = ScreenModel::new("Alias Editor");
    on.apply_common_frame(&mut model_on);
    assert!(
        model_on.footer_lines.iter().any(|l| l.contains("F1")),
        "pfshow ON adds a key-bar footer line; got {:?}",
        model_on.footer_lines
    );

    // pfshow OFF: no key-bar footer line.
    let mut off = alias_shell();
    off.handle_pfshow("OFF");
    assert!(!off.key_bar_visible, "precondition: pfshow OFF");
    let mut model_off = ScreenModel::new("Alias Editor");
    off.apply_common_frame(&mut model_off);
    assert!(
        !model_off.footer_lines.iter().any(|l| l.contains("F1")),
        "pfshow OFF omits the key-bar footer line; got {:?}",
        model_off.footer_lines
    );
}

// Validates: screen-snapshot-scrm Requirement 30.15, 30.16 -- the menu/POM path
// is NOT double-framed: apply_common_frame adds the shared title/menu/tab/status
// chrome but does NOT add a second PF-key footer (the menu body already folds its
// own command line + PF footer), so the key-bar footer_lines stays empty for a
// MenuWorkspace Context even with pfshow ON.
#[test]
fn common_frame_does_not_double_frame_menu_pf_footer() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_pfshow("ON");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::MenuWorkspace);

    // The menu path folds its own command line + PF footer into body_lines, so
    // the helper must NOT add a second footer for a MenuWorkspace Context. Model
    // the menu case: a non-empty body (what the menu builder produces) and an
    // empty footer the helper must leave empty.
    let mut model = ScreenModel::new("Primary Option Menu")
        .with_body_line("Command ===> ____")
        .with_body_line("0 Settings : FFWB Settings (Settings)");
    shell.apply_common_frame(&mut model);

    // Shared chrome IS added (window title + menu echo + tab bar).
    assert!(
        model
            .header_lines
            .iter()
            .any(|l| l.contains(crate::screen_snapshot::WINDOW_TITLE)),
        "menu path still gets the shared window title chrome; got {:?}",
        model.header_lines
    );
    // But NO second PF footer is added by the helper (the menu body owns it).
    assert!(
        model.footer_lines.is_empty(),
        "the menu path must not get a second PF footer from the helper; got {:?}",
        model.footer_lines
    );
}
