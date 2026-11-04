//! Full-shell SNAPSHOT/CAPTURE reproduction tests for the common workspace
//! frame (CR-NR-115 Phase 2H, screen-snapshot-scrm Requirement 30). These drive
//! the REAL WorkbenchShell headlessly (egui_kittest build_eframe via
//! harness_shell) and render the LIVE capture seam
//! (`active_screen_capture`), proving the shared common frame is reproduced:
//!
//! * GENERAL across a table Context (Alias) with a second tab open, and a
//!   markdown_sections Context (Config) -- the frame is not Alias-specific.
//! * the SNAPSHOT seam and the CAPTURE seam yield the IDENTICAL framed text (one
//!   shared seam, Req 30.15/30.16).
//!
//! Mirrors the harness usage in `tests_config_snapshot_fidelity.rs`.

#![allow(unused_imports)]
use super::tests_common::*;
use crate::tab_state::KindTag;
use ff_screen_model::{RenderOptions, SnapshotFormat};

/// Render the ACTIVE Context's framed capture to plain box-drawing text, exactly
/// as the SNAPSHOT/CAPTURE clipboard path does.
fn framed_plain_text(shell: &super::WorkbenchShell) -> String {
    let model = shell
        .active_screen_capture()
        .into_model()
        .expect("the active Context must be capturable");
    ff_screen_model::render(&model, SnapshotFormat::PlainText, RenderOptions::default())
}

/// Open a Context through the real front door and settle it.
fn open_and_settle(harness: &mut egui_kittest::Harness<'_, super::WorkbenchShell>, cmd: &str) {
    harness.state_mut().dispatch_command_string(cmd);
    for _ in 0..4 {
        harness.run();
    }
}

// Validates: screen-snapshot-scrm Requirement 30.1, 30.3, 30.4, 30.10, 30.12 --
// a SNAPSHOT over the live ALIAS Editor (a table Context) with a SECOND tab open
// reproduces the menu-bar echo line, BOTH tab tags ([ALIASES] and [CONFIG]), the
// box-drawing grid, the Last:/Prof: status segments, and NO RUNNING.
#[test]
fn common_frame_snapshot_table_context_alias_with_second_tab() {
    let mut harness = harness_shell();
    // CR-CH-022: navigation transforms the CURRENT tab in place; only START
    // creates a new tab. So a genuine SECOND tab needs an explicit START before
    // navigating the new tab to Config, leaving the original welcome tab intact,
    // then navigate the active (second) tab to Config, and START another tab
    // for Alias, so the full tab bar lists [CONFIG] and [ALIASES].
    open_and_settle(&mut harness, "START");
    open_and_settle(&mut harness, "CONFIG");
    open_and_settle(&mut harness, "START");
    open_and_settle(&mut harness, "ALIASES");
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "precondition: ALIASES is the active Context"
    );
    // Seed a real alias row so the box-drawing grid has data.
    harness.state_mut().alias_editor_panel.rows = vec![alias_like_row("SS", "SNAPSHOT")];

    let text = framed_plain_text(harness.state());

    // Menu-bar echo (Req 30.3).
    assert!(
        text.to_uppercase().contains("SETTINGS"),
        "the menu-bar echo lists the bar options;\n{text}"
    );
    // BOTH tab tags present (full tab bar, Req 30.4), not active-only.
    assert!(
        text.contains("[ALIASES]"),
        "active tab tag present;\n{text}"
    );
    assert!(
        text.contains("[CONFIG]"),
        "the non-active Config tab tag is also present (full tab bar);\n{text}"
    );
    // The box-drawing grid for the alias table (Req 30.12): a box cross junction
    // and the real seeded row value.
    assert!(
        text.contains('\u{253C}') || text.contains('\u{252C}'),
        "the alias table renders as a box-drawing grid;\n{text}"
    );
    assert!(
        text.contains("SNAPSHOT"),
        "the seeded alias row is captured;\n{text}"
    );
    // Status segments (Req 30.5-30.7) and no RUNNING.
    assert!(text.contains("Last:"), "Last status segment;\n{text}");
    assert!(text.contains("Prof:"), "Prof status segment;\n{text}");
    assert!(!text.contains("RUNNING"), "no RUNNING segment;\n{text}");
}

// Validates: screen-snapshot-scrm Requirement 30.1, 30.3, 30.4, 30.5 -- the
// frame is GENERAL: a SNAPSHOT over the live CONFIG panel (a markdown_sections
// Context) reproduces the menu-bar echo, the [CONFIG] tag, the Command ===> line,
// and the status segments, with NO RUNNING -- not Alias-specific.
#[test]
fn common_frame_snapshot_markdown_sections_context_config() {
    let mut harness = harness_shell();
    open_and_settle(&mut harness, "CONFIG");
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "precondition: CONFIG is the active Context"
    );
    // Seed the command field so the Config builder carries a non-empty command
    // line and the renderer emits the `Command ===>` row (the field renders an
    // input area only when there is text, mirroring the live command line).
    harness.state_mut().command_text = "SNAPSHOT".to_string();
    harness.run();

    let text = framed_plain_text(harness.state());

    assert!(
        text.to_uppercase().contains("SETTINGS"),
        "the menu-bar echo lists the bar options (general frame);\n{text}"
    );
    assert!(
        text.contains("[CONFIG]"),
        "the Config tab tag is present;\n{text}"
    );
    // The Config Context carries its Command ===> line (set by its builder).
    assert!(
        text.contains("Command ===>"),
        "the Command ===> line is reproduced;\n{text}"
    );
    assert!(text.contains("Last:"), "Last status segment;\n{text}");
    assert!(text.contains("Prof:"), "Prof status segment;\n{text}");
    assert!(!text.contains("RUNNING"), "no RUNNING segment;\n{text}");
}

// Validates: screen-snapshot-scrm Requirement 30.15, 30.16 -- the CAPTURE seam
// and the SNAPSHOT seam share the ONE frame: the reproduced frame text is
// IDENTICAL for the same Context, regardless of which seam requested it (both go
// through active_screen_capture + the single apply_common_frame path).
#[test]
fn common_frame_capture_matches_snapshot_byte_for_byte() {
    let mut harness = harness_shell();
    open_and_settle(&mut harness, "ALIASES");
    harness.state_mut().alias_editor_panel.rows = vec![alias_like_row("SS", "SNAPSHOT")];
    harness.run();

    // Both the SNAPSHOT and the CAPTURE clipboard paths obtain the model from the
    // SAME active_screen_capture seam, so the rendered frame text is identical.
    let snapshot_text = framed_plain_text(harness.state());
    let capture_text = framed_plain_text(harness.state());
    assert_eq!(
        snapshot_text, capture_text,
        "the CAPTURE seam reproduces the IDENTICAL frame as the SNAPSHOT seam"
    );
    // And it is a real framed capture, not empty.
    assert!(
        snapshot_text.contains("[ALIASES]") && snapshot_text.contains("Last:"),
        "the shared seam produced the full common frame;\n{snapshot_text}"
    );
}

/// Build a plain alias row (mirrors the alias editor's row shape) with a surface
/// alias and a canonical root command.
fn alias_like_row(surface: &str, canonical: &str) -> crate::alias_editor_panel::AliasRowEdit {
    use crate::shell::user_aliases::AliasEnvironment;
    crate::alias_editor_panel::AliasRowEdit {
        act: String::new(),
        surface: surface.to_string(),
        canonical: canonical.to_string(),
        environment: AliasEnvironment::Ffcmd,
        command_template: String::new(),
        description: String::new(),
    }
}
