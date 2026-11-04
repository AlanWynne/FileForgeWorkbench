//! Full-shell tests for the on-dialog direct COPY affordance and the
//! Ctrl+C-while-a-dialog-is-open shortcut (screen-snapshot-scrm Req 27.3).
//!
//! The per-surface model-build/redact/place logic is unit-tested in
//! `dialog_capture.rs`; here we drive the LIVE shell (egui_kittest full-shell
//! harness) to prove the Ctrl+C seam fires while an overlay is open, routes to
//! the single `copy_frontmost_surface` path (which places the SAME multi-flavor
//! COMPOSITE as SNAPSHOT -- Req 27.3 REVISED, owner decision B -- by arming the
//! async window screenshot), and leaves the dialog open. The on-dialog Copy
//! BUTTONS route to the same helper via each dialog's Outcome/return seam, which
//! is covered by the per-dialog `Outcome::Copy` tests and the
//! `dialog_capture.rs` helper tests.

#![allow(unused_imports)]
use super::tests_common::*;

/// Open the external-run confirmation overlay through real shell state. It has
/// NO interior text field, so `wants_keyboard_input()` stays false and the
/// Ctrl+C dialog-copy shortcut is free to fire (not owned by a text field).
fn open_external_confirm(shell: &mut super::WorkbenchShell) {
    shell.pending_external = Some(crate::shell::external_adapter::PendingExternal {
        program: "echo".to_string(),
        args: vec!["hi".to_string()],
        working_dir: None,
        mode: ff_command::ExternalMode::Detached,
    });
}

// Validates: screen-snapshot-scrm Req 27.3 (REVISED, owner decision B) -- while
// a message/dialog box is open, Ctrl+C copies the frontmost dialog through the
// single copy path as the SAME multi-flavor COMPOSITE as SNAPSHOT (it arms the
// async window screenshot and stashes the pending composite), and the dialog
// stays OPEN. The frame loop may consume the stashed request on the
// Event::Screenshot frame, so here we assert the copy STATUS (proving the single
// copy path ran); the compose-and-arm contract is proven in the unit tests.
#[test]
fn ctrl_c_while_dialog_open_copies_composite_and_leaves_open() {
    let mut harness = harness_shell();
    open_external_confirm(harness.state_mut());
    harness.run();
    assert!(
        harness.state().any_dialog_open(),
        "precondition: an overlay is open"
    );

    // The external-confirm overlay has no text field, but the command line keeps
    // focus from startup; a FOCUSED command field owns Ctrl+C for its own text
    // (the shortcut deliberately defers to a focused text field). Surrender that
    // focus so the dialog-copy shortcut is free to fire, mirroring the owner's
    // scenario where the overlay -- not the command line -- is the user's focus.
    harness.ctx.memory_mut(|m| m.stop_text_input());
    harness
        .ctx
        .memory_mut(|m| m.surrender_focus(cmd_field_id()));
    harness.run();

    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::C);
    harness.run();

    assert!(
        harness.state().pending_external.is_some(),
        "Ctrl+C copy must leave the dialog OPEN (27.3)"
    );
    // The copy path ran: `copy_frontmost_surface` sets a status line (the
    // "Copied" message, possibly superseded by the composite-place status once
    // the frame loop consumes the screenshot). A non-None status proves the
    // single copy path executed. The compose-and-arm contract is proven in the
    // `dialog_capture.rs` unit tests, which do not run the frame loop.
    assert!(
        !super::tests_common::reported_text(harness.state()).is_empty(),
        "Ctrl+C routes to the single copy path and reports a status (27.3)"
    );
}

// Validates: screen-snapshot-scrm Req 27.3 -- the SCRM Save-As dialog's Copy
// button (its `Outcome::Copy`) copies through the single path and LEAVES the
// modal open; it does not take/clear `scrm_ui.save_as` and does not dispatch a
// save command.
#[test]
fn scrm_save_as_copy_outcome_leaves_dialog_open() {
    use crate::scrm_save_as_dialog::{Outcome, SaveAsState};
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.scrm_ui.save_as = Some(SaveAsState::new(
        "Repro".to_string(),
        std::path::PathBuf::from("/data"),
    ));
    shell.apply_scrm_save_as_outcome(Outcome::Copy);
    assert!(
        shell.scrm_ui.save_as.is_some(),
        "the Save-As Copy button must leave the dialog OPEN (27.3)"
    );
    assert!(
        shell.snapshot.screenshot_requested && shell.snapshot.pending.is_some(),
        "the Save-As Copy button arms the composite screenshot path (27.3, owner B)"
    );
}

// Validates: screen-snapshot-scrm Req 27.3 -- the SCRM replay picker's Copy
// button (its `Outcome::Copy`) copies through the single path and LEAVES the
// modal open.
#[test]
fn scrm_replay_picker_copy_outcome_leaves_dialog_open() {
    use crate::scrm_session_picker::{Outcome, ReplayPickerState};
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.scrm_ui.replay_picker = Some(ReplayPickerState::new());
    shell.apply_scrm_replay_outcome(Outcome::Copy);
    assert!(
        shell.scrm_ui.replay_picker.is_some(),
        "the replay picker Copy button must leave the dialog OPEN (27.3)"
    );
    assert!(
        shell.snapshot.screenshot_requested && shell.snapshot.pending.is_some(),
        "the replay picker Copy button arms the composite screenshot path (27.3, owner B)"
    );
}

// Validates: screen-snapshot-scrm Req 27.3 -- with NO overlay open, Ctrl+C is
// NOT intercepted by the dialog-copy shortcut (normal Ctrl+C elsewhere is
// untouched): no screenshot is armed and no dialog is conjured.
#[test]
fn ctrl_c_with_no_dialog_open_is_not_intercepted() {
    let mut harness = harness_shell();
    assert!(
        !harness.state().any_dialog_open(),
        "precondition: no overlay open"
    );
    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::C);
    harness.run();
    assert!(
        !harness.state().snapshot.screenshot_requested,
        "no dialog -> dialog-copy shortcut does not fire"
    );
    assert!(
        !harness.state().any_dialog_open(),
        "no dialog -> Ctrl+C conjures no overlay"
    );
}
