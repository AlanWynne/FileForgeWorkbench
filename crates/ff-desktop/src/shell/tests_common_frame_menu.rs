//! GOLDEN tests for the Menu Kind (POM, Settings, any `menus/<name>.toml`) on the
//! common-frame SNAPSHOT/CAPTURE path (CR-NR-115 follow-up, screen-snapshot-scrm
//! Requirement 30.27). The menu path used to fold its own `Command ===>` row and
//! PF footer into the body, so the key line sat INSIDE the content above the
//! status band and the triggering command (`ss`) showed with short underscores.
//! These tests compare FULL text so a presence-only pass can never hide that.

use super::tests_common::*;
use super::tests_common_frame_golden::{assert_golden, fixed_stamp};
use crate::tab_state::KindTag;

pub(super) const CONFIRMATION: &str = "Snapshot copied to clipboard: picture + Markdown.";

/// Pin the menu calendar clock so the POM calendar/Time block is reproducible.
fn pin_menu_clock() {
    super::screen_provider::TEST_MENU_NOW.with(|c| c.set(Some(fixed_stamp())));
}

/// A deterministic shell whose ONLY tab is a seeded POM, with the triggering
/// command `ss` still in the command field (as it is during real dispatch).
pub(super) fn pom_shell(pfshow: bool, message: Option<&str>) -> super::WorkbenchShell {
    super::i18n_bootstrap::install_catalogue_for_tests();
    ff_session::set_active_profile(None);
    pin_menu_clock();
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    let pom_id = shell.tabs.active_tab().id;
    while let Some(index) = shell.tabs.tabs().iter().position(|t| t.id != pom_id) {
        shell.tabs.close_tab(index);
    }
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::MenuWorkspace);
    shell.last_action_at = fixed_stamp();
    shell.key_bar_visible = pfshow;
    shell.command_text = "ss".to_string();
    shell.message_line = message.map(|m| vec![m.to_string()]).unwrap_or_default();
    shell
}

pub(super) fn capture(shell: &super::WorkbenchShell) -> String {
    shell
        .snapshot_text_for_active("")
        .expect("menu Context must be capturable")
        .0
        .trim_end_matches('\n')
        .to_string()
}

/// Structural frame assertions shared by every Menu Kind (no name branch).
pub(super) fn assert_menu_frame_shape(text: &str, pfshow: bool, message: Option<&str>) {
    let lines: Vec<&str> = text.lines().collect();
    let n = lines.len();
    assert!(lines[n - 1].starts_with('\u{2514}'), "bottom border last");
    let status = lines
        .iter()
        .position(|l| l.contains("Last: 11:02"))
        .expect("status band row");
    let band = lines[status];
    let last = band.find("Last: 11:02").expect("Last");
    let prof = band.find("Prof: default").expect("Prof");
    let zoom = band.find("Zoom: 0").expect("Zoom");
    assert!(
        last < prof && prof < zoom,
        "Last | Prof | Zoom order: {band}"
    );
    if pfshow {
        assert!(
            lines[n - 2].contains("F1 Help"),
            "key line is the LAST band"
        );
        assert_eq!(
            status + 2,
            n - 2,
            "key line directly follows the status band"
        );
    } else {
        assert_eq!(status + 1, n - 1, "no key line when pfshow is off");
    }
    let command = lines
        .iter()
        .find(|l| l.contains("Command ===>"))
        .expect("command row");
    assert!(
        !command.contains("ss"),
        "triggering command must not show: {command}"
    );
    let after = command.split("Command ===> ").nth(1).expect("field");
    let field = after.trim_end_matches('\u{2502}').trim_end();
    assert!(
        !field.is_empty() && field.chars().all(|c| c == '_'),
        "field is underscores only: {command}"
    );
    assert!(
        after.ends_with(" \u{2502}"),
        "underscores fill to the right border: {command}"
    );
    if let Some(m) = message {
        let msg = lines.iter().position(|l| l.contains(m)).expect("message");
        assert!(msg < status, "message band is above the status band");
    }
}

// Validates: screen-snapshot-scrm Requirement 30.27, 30.10 and
// workspace-framework Requirement 7.16 -- the POM (pfshow ON, with a message)
// matches the GOLDEN byte-for-byte: key line LAST as one cell, status band
// Last|Prof|Zoom, command field empty with underscores to the border, message
// band above the status band.
#[test]
fn menu_kind_pom_snapshot_matches_golden_pfshow_on_with_message() {
    let shell = pom_shell(true, Some(CONFIRMATION));
    let text = capture(&shell);
    assert_menu_frame_shape(&text, true, Some(CONFIRMATION));
    assert_golden("menu_pom_message_pfshow_on.txt", &text);
}

// Validates: screen-snapshot-scrm Requirement 30.27 -- the SAME frame behaviour
// for another Menu Kind (the built-in `settings` menu, pfshow OFF, no message):
// proves there is no POM-specific branch on the shared menu path.
#[test]
fn menu_kind_settings_snapshot_matches_golden_pfshow_off() {
    let mut shell = pom_shell(false, None);
    shell.dispatch_command_string("SETTINGS");
    // Req 7.16: the SETTINGS navigation is a screen transition and restamped
    // `Last:`; re-pin it so the golden is deterministic.
    shell.last_action_at = fixed_stamp();
    shell.command_text = "ss".to_string();
    shell.message_line.clear();
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::MenuWorkspace);
    let text = capture(&shell);
    assert_menu_frame_shape(&text, false, None);
    assert_golden("menu_settings_pfshow_off.txt", &text);
}

// Validates: screen-snapshot-scrm Requirement 30.16, 30.27 -- SNAPSHOT equals
// CAPTURE byte-for-byte for the Menu Kind (one shared seam).
#[test]
fn menu_kind_snapshot_equals_capture() {
    let shell = pom_shell(true, Some(CONFIRMATION));
    let model = shell.active_screen_model().expect("capturable");
    let captured =
        crate::screen_snapshot::render_snapshot(&model, ff_screen_model::SnapshotFormat::PlainText);
    assert_eq!(capture(&shell), captured.trim_end_matches('\n'));
}

// NOTE (Requirement 30.27, user `menus/<name>.toml`): a user menu opens through
// the same MenuWorkspace arm as POM and Settings (dispatch is by KindTag, never
// by menu name), so the two goldens above plus the structural assertion cover
// it. Loading a user menu in this harness needs the menus_dir resolver, which
// is not cheap to isolate here, so no third fixture is carried.
