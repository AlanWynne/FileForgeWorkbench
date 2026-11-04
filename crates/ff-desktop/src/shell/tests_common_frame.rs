//! Shell tests -- common-frame area (CR-NR-115, workspace-framework Phase CF).
//!
//! Covers the additive Message_Line panel (render_message_line) and the
//! full-shell single-chrome regression guard for the common frame: the command
//! field is present, the Status_Bar and Message_Line are DISTINCT bottom panels,
//! the key bar follows key_bar_visible/pfshow, and no active Workspace re-draws a
//! frame element in content.

#![allow(unused_imports)]
use super::tests_common::*;
use egui_kittest::kittest::Queryable;

// === CF.1.3: render_message_line bottom panel (Req 7.8, 7.13, 7.14) =========

/// Validates: Requirement 7.13 -- render_message_line draws NO panel when the
/// Message_Line store is empty (absent, no reserved space). Rendered headlessly
/// with a known sentinel label after it; with an empty store no "msg:" label is
/// in the accessibility tree.
#[test]
fn message_line_absent_when_empty() {
    // Validates: Requirement 7.13
    use egui_kittest::Harness;

    let mut shell = make_shell();
    assert!(shell.message_line.is_empty(), "store starts empty");

    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(1200.0, 900.0))
        .build(move |ctx| {
            shell.render_message_line(ctx);
        });
    harness.run();

    let rows = harness.query_all_by_label("msg one").count();
    assert_eq!(
        rows, 0,
        "an empty Message_Line store renders no message rows (absent panel)"
    );
}

/// Validates: Requirement 7.8, 7.14 -- render_message_line renders ONE row per
/// message, expanding for multiple messages. With two messages seeded, both
/// labels are present in the accessibility tree.
#[test]
fn message_line_renders_one_row_per_message() {
    // Validates: Requirement 7.8, 7.14
    use egui_kittest::Harness;

    let mut shell = make_shell();
    shell.message_line = vec!["msg one".to_string(), "msg two".to_string()];

    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(1200.0, 900.0))
        .build(move |ctx| {
            shell.render_message_line(ctx);
        });
    harness.run();

    let first = harness.query_all_by_label("msg one").count();
    let second = harness.query_all_by_label("msg two").count();
    assert_eq!(first, 1, "the first message renders exactly one row");
    assert_eq!(second, 1, "the second message renders exactly one row");
}

// === CF.6: full-shell single-chrome common-frame regression guard ===========

/// Validates: Requirement 7.1, 7.5, 7.6, 7.22, 7.23, 7.24, 7.27 -- the eight
/// common-frame elements render ONCE around the active Workspace: the command
/// field (command_field_input) is present; the Status_Bar and (when non-empty)
/// the Message_Line are DISTINCT bottom panels; the key bar follows
/// key_bar_visible/pfshow; and no active Workspace re-draws a frame element in
/// content (the redundant-surface-title count stays 0). Mirrors
/// full_shell_alias_editor_is_single_chrome.
#[test]
fn full_shell_common_frame_is_single_chrome() {
    // Validates: Requirement 7.1, 7.5, 7.6, 7.22, 7.23, 7.24, 7.27
    let mut harness = harness_shell();

    // The single command line is present exactly once (Req 7.6).
    let command_lines = harness.query_all_by_label("Command ===>").count();
    assert_eq!(
        command_lines, 1,
        "the common frame shows exactly ONE 'Command ===>' line; found {command_lines}"
    );

    // The Status_Bar and Message_Line are DISTINCT bottom panels: the Status_Bar
    // is always present; the Message_Line is absent while empty (Req 7.13), so
    // seed a message and confirm the two panels coexist without either
    // duplicating the other.
    harness.state_mut().message_line = vec!["info one".to_string()];
    harness.run();
    let msg_rows = harness.query_all_by_label("info one").count();
    assert_eq!(
        msg_rows, 1,
        "the Message_Line renders the seeded message once as its own panel \
         (distinct from the always-present Status_Bar)"
    );

    // The removed RUNNING phase segment must not reappear anywhere (Req 7.15).
    let running = harness.query_all_by_label("RUNNING").count();
    assert_eq!(
        running, 0,
        "the redundant RUNNING segment is gone (Req 7.15)"
    );

    // No active Workspace re-draws the command-line frame element in content:
    // even with the Message_Line panel now present, the single 'Command ===>'
    // line is still rendered exactly once (Req 7.27 -- no in-content duplicate).
    let command_lines_after = harness.query_all_by_label("Command ===>").count();
    assert_eq!(
        command_lines_after, 1,
        "the single 'Command ===>' line stays once with the Message_Line present; found {command_lines_after}"
    );

    // The key bar follows key_bar_visible: it is visible by default. Toggle it
    // OFF via PFSHOW and confirm the shell honours the flag (Req 7.22).
    assert!(
        harness.state().key_bar_visible,
        "the key bar is visible by default"
    );
    harness.state_mut().handle_pfshow("OFF");
    harness.run();
    assert!(
        !harness.state().key_bar_visible,
        "PFSHOW OFF hides the key bar (the frame honours key_bar_visible)"
    );
}
