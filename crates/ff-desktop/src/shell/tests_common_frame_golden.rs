//! GOLDEN tests for the full SNAPSHOT/CAPTURE common-frame reproduction
//! (CR-NR-115, screen-snapshot-scrm Requirement 30.18-30.24). Earlier rounds of
//! this gate passed PRESENCE/POSITION tests while the real rendered text was
//! still wrong (ruled bands missing, a blank row before the status bar, no
//! cell dividers), so these tests compare the FULL rendered text, line for
//! line, against a FIXED fixture captured once and reviewed by hand (the
//! `golden/*.txt` files next to this module). A deterministic fixed width
//! (fixed last-action time, fixed profile, two open tabs, seeded content) makes
//! the fixture reproducible.

#![allow(unused_imports)]
use super::tests_common::*;
use crate::tab_state::KindTag;

/// Fixed local timestamp so the `Last: HH:MM` status segment is deterministic.
pub(super) fn fixed_stamp() -> chrono::DateTime<chrono::Local> {
    use chrono::TimeZone;
    chrono::Local
        .with_ymd_and_hms(2026, 11, 4, 11, 2, 0)
        .single()
        .expect("fixed local time")
}

fn open_and_settle(harness: &mut egui_kittest::Harness<'_, super::WorkbenchShell>, cmd: &str) {
    harness.state_mut().dispatch_command_string(cmd);
    for _ in 0..4 {
        harness.run();
    }
}

/// The harness can carry extra tabs that a concurrently running test persisted
/// and the session restore re-opened, which would make the tab bar (and so the
/// frame width) nondeterministic. Keep exactly two tabs: the ACTIVE Context and
/// the first [POM] tab, and close every other one (by tab id, so closing a tab
/// never shifts which one is kept).
fn keep_active_and_first_pom_tabs(shell: &mut super::WorkbenchShell) {
    let active_id = shell.tabs.active_tab().id;
    let pom_id = shell
        .tabs
        .tabs()
        .iter()
        .find(|t| t.kind.tag() == KindTag::MenuWorkspace && t.id != active_id)
        .map(|t| t.id)
        .expect("a [POM] tab besides the active Context");
    while let Some(index) = shell
        .tabs
        .tabs()
        .iter()
        .position(|t| t.id != active_id && t.id != pom_id)
    {
        shell.tabs.close_tab(index);
    }
    assert_eq!(shell.tabs.len(), 2, "exactly two tabs for a stable frame");
}

/// A deterministic two-tab shell ([POM] + the given Context), fixed last-action
/// time and the default profile, with pfshow and the message line set as given.
fn fixed_shell(
    open_cmd: &str,
    pfshow: bool,
    message: Option<&str>,
) -> egui_kittest::Harness<'static, super::WorkbenchShell> {
    ff_session::set_active_profile(None);
    let mut h = harness_shell();
    open_and_settle(&mut h, "START");
    open_and_settle(&mut h, open_cmd);
    keep_active_and_first_pom_tabs(h.state_mut());
    if open_cmd == "ALIASES" {
        h.state_mut().alias_editor_panel.rows = vec![crate::alias_editor_panel::AliasRowEdit {
            act: String::new(),
            surface: "SS".into(),
            canonical: "SNAPSHOT".into(),
            environment: crate::shell::user_aliases::AliasEnvironment::Ffcmd,
            command_template: String::new(),
            description: String::new(),
        }];
    }
    // Settle the tab-change transition stamp (Req 7.16) BEFORE pinning `Last:`,
    // so the final frame does not restamp it with the wall clock.
    h.state_mut().sync_active_tab_transition();
    h.state_mut().last_action_at = fixed_stamp();
    // The harness restores the machine's persisted session, which can carry a
    // non-default zoom; pin it so the `Zoom:` segment of the golden is stable.
    h.state_mut().zoom.set_offset(0);
    h.state_mut().key_bar_visible = pfshow;
    h.state_mut().message_line = message.map(|m| vec![m.to_string()]).unwrap_or_default();
    h.run();
    h
}

/// Load a golden fixture, normalising a trailing newline the write tool adds.
pub(super) fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/shell/golden")
        .join(name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {name} must be readable: {e}"));
    raw.replace("\r\n", "\n").trim_end_matches('\n').to_string()
}

fn snapshot_text(h: &egui_kittest::Harness<'_, super::WorkbenchShell>) -> String {
    h.state()
        .snapshot_text_for_active("")
        .expect("active Context must be capturable")
        .0
        .trim_end_matches('\n')
        .to_string()
}

fn capture_text(h: &egui_kittest::Harness<'_, super::WorkbenchShell>) -> String {
    let model = h
        .state()
        .active_screen_model()
        .expect("active Context must be capturable");
    crate::screen_snapshot::render_snapshot(&model, ff_screen_model::SnapshotFormat::PlainText)
        .trim_end_matches('\n')
        .to_string()
}

// Validates: screen-snapshot-scrm Requirement 30.18-30.24 -- the Config editor
// (a markdown_sections Context), pfshow ON, matches the GOLDEN fixture
// byte-for-byte: ruled bands, the title-bar '#' cell + right-aligned window
// controls, cell dividers with correct junctions, the Command line filled to
// the border, and the key line last.
#[test]
fn config_snapshot_matches_golden_fixture_pfshow_on() {
    let h = fixed_shell("CONFIG", true, None);
    assert_eq!(snapshot_text(&h), fixture("config_pfshow_on.txt"));
}

// Validates: screen-snapshot-scrm Requirement 30.18-30.24 -- the Alias editor
// (a table Context), pfshow ON, matches the GOLDEN fixture byte-for-byte,
// including the box-drawing grid staying inside the frame.
#[test]
fn alias_snapshot_matches_golden_fixture_pfshow_on() {
    let h = fixed_shell("ALIASES", true, None);
    assert_eq!(snapshot_text(&h), fixture("alias_pfshow_on.txt"));
}

// Validates: screen-snapshot-scrm Requirement 30.8, 30.9, 30.11, 30.21-30.23 --
// with a message present and pfshow OFF, the Config editor matches the GOLDEN
// fixture: the message band sits between content and status with its own
// rules, no key line, and no blank row anywhere that isn't a real band.
#[test]
fn config_snapshot_matches_golden_fixture_message_pfshow_off() {
    let h = fixed_shell("CONFIG", false, Some("info one"));
    assert_eq!(snapshot_text(&h), fixture("config_message_pfshow_off.txt"));
}

// Validates: screen-snapshot-scrm Requirement 30.8, 30.11, 30.15, 30.21-30.23 --
// the same message/pfshow-off placement holds for the Alias (table) Context.
#[test]
fn alias_snapshot_matches_golden_fixture_message_pfshow_off() {
    let h = fixed_shell("ALIASES", false, Some("info one"));
    assert_eq!(snapshot_text(&h), fixture("alias_message_pfshow_off.txt"));
}

// Validates: screen-snapshot-scrm Requirement 30.12, 30.18 -- the ASCII
// fallback (unicode_box = false) produces the exact `+ - |` equivalent of the
// Unicode golden fixture for one Context (Alias, message + pfshow off).
#[test]
fn alias_snapshot_ascii_fallback_matches_golden_fixture() {
    let h = fixed_shell("ALIASES", false, Some("info one"));
    let model = h
        .state()
        .active_screen_model()
        .expect("active Context must be capturable");
    let text = ff_screen_model::render(
        &model,
        ff_screen_model::SnapshotFormat::PlainText,
        ff_screen_model::RenderOptions { unicode_box: false },
    );
    assert_eq!(
        text.trim_end_matches('\n'),
        fixture("alias_message_pfshow_off_ascii.txt")
    );
}

// Validates: screen-snapshot-scrm Requirement 30.10, 30.26 and
// workspace-framework Requirement 7.16-7.21 -- the status band of a settings-type
// Workspace (Config, Alias) carries ONLY the three shell-common segments, Last,
// Prof and Zoom. Encoding, Ln/Col and line count are workspace-owned and absent.
#[test]
fn config_and_alias_status_band_is_last_prof_and_zoom_only() {
    for cmd in ["CONFIG", "ALIASES"] {
        let h = fixed_shell(cmd, false, None);
        let text = snapshot_text(&h);
        let status = text
            .lines()
            .find(|l| l.contains("Last:"))
            .expect("status band row");
        assert!(status.contains("Prof: default"), "{cmd}: {status}");
        assert!(status.contains("Zoom: 0"), "{cmd}: {status}");
        for gone in ["Ln ", "Col ", "UTF-8", "Lines "] {
            assert!(
                !status.contains(gone),
                "{cmd} must not show {gone:?}: {status}"
            );
        }
    }
}

// Validates: screen-snapshot-scrm Requirement 30.10, 30.26 and
// workspace-framework Requirement 7.17, 7.21 -- the EDITOR Context owns its
// document segment: its status text still shows Last, Prof, Ln/Col, the
// document encoding and the line count (no regression from the status split).
// The editor is parked for capture, so the composed status text is asserted.
#[test]
fn editor_context_status_text_keeps_ln_col_encoding_and_line_count() {
    super::i18n_bootstrap::install_catalogue_for_tests();
    ff_session::set_active_profile(None);
    let mut shell = make_shell();
    shell.shell_new_untitled();
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::Untitled);
    shell.last_action_at = fixed_stamp();
    let mut model = ff_screen_model::ScreenModel::new("Untitled");
    shell.apply_common_frame(&mut model);
    assert_eq!(
        model.status_bar.text,
        "Last: 11:02 | Prof: default | Zoom: 0 | Ln 1, Col 1 | UTF-8 | 1 lines"
    );
}

// Validates: screen-snapshot-scrm Requirement 30.16 -- SNAPSHOT == CAPTURE
// byte-for-byte for both Contexts, using the SAME deterministic fixture state.
#[test]
fn snapshot_equals_capture_byte_for_byte_for_config_and_alias() {
    let config = fixed_shell("CONFIG", true, None);
    assert_eq!(snapshot_text(&config), capture_text(&config));
    let alias = fixed_shell("ALIASES", true, None);
    assert_eq!(snapshot_text(&alias), capture_text(&alias));
}
