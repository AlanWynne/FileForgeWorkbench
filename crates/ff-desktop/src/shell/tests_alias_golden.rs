//! GOLDEN tests for the Alias Editor SNAPSHOT in the CR-NR-116 framed-band +
//! UPPERCASE layout (command-configurator Req 7.14-7.28, design 9.11.1). Sibling
//! of `tests_common_frame_golden.rs`; it reuses that module's deterministic
//! fixtures (`fixed_stamp`, `assert_golden`, `fixture`) but drives a richer
//! THREE-ROW working copy set directly on `alias_editor_panel.rows`, so the
//! grid shows the real seed-and-user rows and the trailing scratch row.
//!
//! The goldens prove: the X -> EXIT seed row is the FIRST data row and editable;
//! the SS row's empty cells render as underscore runs (never `(&1)` / `note`);
//! there is no blank or caption row around the grid; the CE combo cells show
//! UPPERCASE `[FFCMD v]`; and the Alias status cells (`Aliases: n`,
//! `Rows 1-m visible`, and `Filter: FFCMD/SNAPSHOT` when filtered) follow the
//! common Last | Prof | Zoom segments.

#![allow(unused_imports)]
use super::tests_common::*;
use super::tests_common_frame_golden::{assert_golden, fixed_stamp, fixture};
use crate::alias_editor_panel::AliasRowEdit;
use crate::shell::user_aliases::AliasEnvironment;
use crate::tab_state::KindTag;

fn open_and_settle(harness: &mut egui_kittest::Harness<'_, super::WorkbenchShell>, cmd: &str) {
    harness.state_mut().dispatch_command_string(cmd);
    for _ in 0..4 {
        harness.run();
    }
}

/// Keep exactly two tabs (the active Context + the first [POM]) so the tab bar,
/// and therefore the frame width, is deterministic. Mirrors the helper in
/// `tests_common_frame_golden.rs`.
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

/// One alias working-copy row (surface alias, canonical root, FFCMD, with an
/// optional parameter template and description).
fn row(surface: &str, canonical: &str, template: &str, description: &str) -> AliasRowEdit {
    AliasRowEdit {
        act: String::new(),
        surface: surface.to_string(),
        canonical: canonical.to_string(),
        environment: AliasEnvironment::Ffcmd,
        command_template: template.to_string(),
        description: description.to_string(),
    }
}

/// The deterministic THREE-ROW seed-and-user working copy for the goldens: the
/// English `X -> EXIT` seed row first, then the user `SS -> SNAPSHOT` row, then a
/// `CS -> CAPTURE START` row carrying a `&1` parameter and a description.
fn three_rows() -> Vec<AliasRowEdit> {
    vec![
        row("X", "EXIT", "", "Exit the application"),
        row("SS", "SNAPSHOT", "", ""),
        row("CS", "CAPTURE START", "&1", "Session Recording start"),
    ]
}

/// A deterministic two-tab Alias shell with the given rows and filter, a fixed
/// last-action time, the default profile, zoom pinned to 0, pfshow OFF and no
/// message line. Settled and ready to snapshot.
fn alias_shell(
    rows: Vec<AliasRowEdit>,
    filter_ce: Option<AliasEnvironment>,
    filter_root: &str,
) -> egui_kittest::Harness<'static, super::WorkbenchShell> {
    ff_session::set_active_profile(None);
    let mut h = harness_shell();
    open_and_settle(&mut h, "START");
    open_and_settle(&mut h, "ALIASES");
    keep_active_and_first_pom_tabs(h.state_mut());
    h.state_mut().alias_editor_panel.rows = rows;
    h.state_mut().alias_editor_panel.filter_ce = filter_ce;
    h.state_mut().alias_editor_panel.filter_root = filter_root.to_string();
    h.state_mut().sync_active_tab_transition();
    h.state_mut().last_action_at = fixed_stamp();
    h.state_mut().zoom.set_offset(0);
    h.state_mut().key_bar_visible = false;
    h.state_mut().message_line = Vec::new();
    h.run();
    h
}

fn snapshot_text(h: &egui_kittest::Harness<'_, super::WorkbenchShell>) -> String {
    h.state()
        .snapshot_text_for_active("")
        .expect("active Context must be capturable")
        .0
        .trim_end_matches('\n')
        .to_string()
}

// Validates: command-configurator Requirement 7.16-7.18, 7.22, 7.25 -- the Alias
// SNAPSHOT with the three-row working copy matches the GOLDEN byte-for-byte: the
// X -> EXIT seed row is the first data row, the SS empty cells are underscore
// runs, the grid has no blank/caption row, the CE cells are UPPERCASE combos,
// and the status band ends `Aliases: 3 | Rows 1-3 visible`.
#[test]
fn alias_snapshot_three_rows_matches_golden() {
    let h = alias_shell(three_rows(), None, "");
    let text = snapshot_text(&h);
    assert_golden("alias_pfshow_off_rows.txt", &text);
}

// Validates: command-configurator Requirement 7.16-7.18, 7.25 -- the X -> EXIT
// seed row is present as the FIRST data row and is a real editable backing row
// (it is one of the three displayed data rows, before the scratch row).
#[test]
fn alias_snapshot_seed_row_is_first_data_row() {
    let h = alias_shell(three_rows(), None, "");
    let rows = &h.state().alias_editor_panel.rows;
    assert_eq!(rows[0].surface, "X", "X is the first backing row");
    assert_eq!(rows[0].canonical, "EXIT", "its root is EXIT");
    // The grid lists the seed row's values (EXIT) and the user rows (SNAPSHOT).
    let text = snapshot_text(&h);
    assert!(
        text.contains("EXIT"),
        "the seed row value is captured;\n{text}"
    );
    assert!(
        text.contains("[FFCMD v]"),
        "CE renders as an UPPERCASE combo cell;\n{text}"
    );
}

// Validates: command-configurator Requirement 7.15, 7.22 -- the SS row's empty
// Parameters/Description cells render as underscore RUNS, never the live hint
// text `(&1)` / `note`.
#[test]
fn alias_snapshot_empty_cells_are_underscore_runs_not_hints() {
    let h = alias_shell(three_rows(), None, "");
    let text = snapshot_text(&h);
    assert!(
        text.contains("____"),
        "empty cells are underscore runs;\n{text}"
    );
    assert!(
        !text.contains("(&1)") && !text.contains("note"),
        "no live hint text leaks into the snapshot;\n{text}"
    );
}

// Validates: command-configurator Requirement 7.22, screen-snapshot-scrm
// Requirement 30.40 -- the Alias Markdown clipboard flavour is the fenced box
// FIRST, then exactly ONE `### Aliases` section table, and NO `| Property |
// Value |` table (the former fields dot-leader table is gone). HTML mirrors it.
#[test]
fn alias_clipboard_is_box_first_then_one_aliases_table_no_property_value() {
    let h = alias_shell(three_rows(), None, "");
    let model = h
        .state()
        .active_screen_model()
        .expect("active Context must be capturable");

    let md = ff_screen_model::render(
        &model,
        ff_screen_model::SnapshotFormat::Markdown,
        ff_screen_model::RenderOptions::default(),
    );
    let fence = md.find("```").expect("markdown opens with a fenced box");
    let heading = md.find("### Aliases").expect("one ### Aliases heading");
    assert!(
        fence < heading,
        "the fenced box comes BEFORE the Aliases table"
    );
    assert_eq!(
        md.matches("### Aliases").count(),
        1,
        "exactly one ### Aliases table"
    );
    assert!(
        !md.contains("| Property | Value |"),
        "no Property/Value table in the markdown;\n{md}"
    );

    let html = ff_screen_model::render_tables_html(&model);
    assert!(
        html.contains("Aliases"),
        "the HTML table flavour carries the Aliases section;\n{html}"
    );
    assert!(
        !html.contains("Property"),
        "no Property/Value table in the HTML;\n{html}"
    );
}

// Validates: command-configurator Requirement 7.24, 7.25 -- a filter of
// FFCMD/SNAPSHOT shows only the matching row plus the scratch row, and the
// status band ends `Filter: FFCMD/SNAPSHOT`.
#[test]
fn alias_snapshot_filtered_matches_golden() {
    let h = alias_shell(three_rows(), Some(AliasEnvironment::Ffcmd), "SNAPSHOT");
    let text = snapshot_text(&h);
    assert_golden("alias_filtered.txt", &text);
}

// Validates: screen-snapshot-scrm Requirement 30.12, 30.18 -- the ASCII fallback
// (unicode_box = false) is the exact `+ - |` equivalent of the filtered Unicode
// golden.
#[test]
fn alias_snapshot_filtered_ascii_matches_golden() {
    let h = alias_shell(three_rows(), Some(AliasEnvironment::Ffcmd), "SNAPSHOT");
    let model = h
        .state()
        .active_screen_model()
        .expect("active Context must be capturable");
    let text = ff_screen_model::render(
        &model,
        ff_screen_model::SnapshotFormat::PlainText,
        ff_screen_model::RenderOptions { unicode_box: false },
    );
    assert_golden("alias_filtered_ascii.txt", text.trim_end_matches('\n'));
}
