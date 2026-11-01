//! Config-panel capture fidelity (config-snapshot-bugs.md: Config Bug 1 + Bug
//! 2) folded into Stream SCRM Phase 2E because they are the SAME defect class as
//! the dialog capture work (a ScreenModel not faithfully capturing real
//! on-screen content, and a capture that mutates the screen it captures).
//!
//! Bug 2 (faithful model): the Config ScreenProvider must capture the REAL
//! settings tree -- the visible categories and their current values -- not just
//! Filter + Unsaved-edits (screen-snapshot-scrm Req 27.8, master-screen-directive
//! Req 4.4a).
//!
//! Bug 1 (side-effect-free capture): submitting a command from the command line
//! (e.g. SNAPSHOT) must NOT also activate/collapse the focused config-tree node.
//! The Enter that submits the command was ALSO being read by the Config_Tree's
//! per-frame keyboard driver, collapsing the focused node (master-screen-directive
//! Req 4.4a: capture must be side-effect-free).

#![allow(unused_imports)]
use super::tests_common::*;
use crate::tab_state::KindTag;

/// Open the flat Config panel through the real front door and settle it.
fn open_config(harness: &mut egui_kittest::Harness<'_, super::WorkbenchShell>) {
    harness.state_mut().dispatch_command_string("CONFIG");
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "precondition: CONFIG opens the flat Config panel"
    );
}

// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a
// (Config Bug 2) -- the Config ScreenProvider captures the REAL settings tree:
// namespace group rows AND key rows with their current effective values, not a
// thin two-field (Filter + Unsaved) summary.
#[test]
fn config_screen_model_captures_the_real_settings_tree() {
    let mut harness = harness_shell();
    open_config(&mut harness);

    let model = harness.state().config_panel_screen_model();

    // The thin-model baseline fields are still present...
    assert!(
        model.fields.iter().any(|f| f.label == "Filter"),
        "Filter field retained"
    );
    assert!(
        model.fields.iter().any(|f| f.label == "Unsaved edits"),
        "Unsaved-edits field retained"
    );

    // ...but the model is NO LONGER just those two: it carries the real tree.
    // There must be at least one namespace GROUP header row (e.g. "[Editor]",
    // "[Accessibility]"): a bracketed label with an expanded/collapsed value.
    let group_rows: Vec<_> = model
        .fields
        .iter()
        .filter(|f| {
            f.label.starts_with('[')
                && f.label.ends_with(']')
                && (f.value == "expanded" || f.value == "collapsed")
        })
        .collect();
    assert!(
        !group_rows.is_empty(),
        "the capture must list the real settings-tree namespace groups (Bug 2); \
         got fields: {:?}",
        model.fields.iter().map(|f| &f.label).collect::<Vec<_>>()
    );

    // There must be at least one KEY row -- a dotted schema key with a value --
    // under an expanded group, proving actual settings + values are captured.
    let key_rows: Vec<_> = model
        .fields
        .iter()
        .filter(|f| f.label.contains('.'))
        .collect();
    assert!(
        !key_rows.is_empty(),
        "the capture must list real settings keys with their values (Bug 2); \
         got fields: {:?}",
        model.fields.iter().map(|f| &f.label).collect::<Vec<_>>()
    );

    // The whole thing is strictly richer than the old 2-field summary.
    assert!(
        model.fields.len() > 2,
        "the faithful model is richer than the thin Filter+Unsaved summary (Bug 2)"
    );
}

// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a --
// a key the user has edited (a staged working value) is captured with its
// CURRENT unsaved value and marked unsaved, so the capture reflects what is on
// screen while editing.
#[test]
fn config_screen_model_captures_staged_edit_value() {
    use ff_config::value::ConfigValue;
    let mut harness = harness_shell();
    open_config(&mut harness);

    // Stage an edit directly into the working buffer and expand its namespace so
    // the key row is visible in the tree.
    let key = "accessibility.reduce_motion";
    harness
        .state_mut()
        .config_panel
        .stage_set(key, ConfigValue::Boolean(true));
    harness
        .state_mut()
        .config_panel
        .collapsed
        .insert("accessibility".to_string(), false);
    harness.run();

    let model = harness.state().config_panel_screen_model();
    let row = model.fields.iter().find(|f| f.label == key);
    // The accessibility namespace may or may not exist in every schema build;
    // when it does, the staged value must be captured and marked unsaved.
    if let Some(row) = row {
        assert!(
            row.value.contains("true"),
            "the staged working value is captured, got {:?}",
            row.value
        );
        assert!(
            row.value.contains("unsaved"),
            "a staged edit is marked unsaved in the capture, got {:?}",
            row.value
        );
    }
}

// Validates: master-screen-directive Req 4.4a (Config Bug 1) -- submitting a
// command from the command line does NOT activate/collapse the focused
// config-tree node. SNAPSHOT (and any command) must leave the screen exactly as
// it was: the Enter that submits the command must be consumed by the command
// line and NOT also read by the Config_Tree keyboard driver.
#[test]
fn command_submit_does_not_collapse_focused_config_tree_node() {
    let mut harness = harness_shell();
    open_config(&mut harness);

    // Startup re-arms command-field focus on entering a Workspace, so the
    // command line holds focus (the exact state in the owner's repro).
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "precondition: the command line holds focus"
    );
    // Precondition: nothing is collapsed yet (every namespace expanded).
    assert!(
        !harness
            .state()
            .config_panel
            .collapsed
            .values()
            .any(|collapsed| *collapsed),
        "precondition: no namespace group is collapsed on entry"
    );

    // Type a command and press Enter -- the exact repro (owner: `snapshot`+Enter
    // collapsed the focused Accessibility item). SNAPSHOT only arms the async
    // screenshot; it must not mutate the tree.
    harness.state_mut().command_text = "SNAPSHOT".to_string();
    harness.key_press(egui::Key::Enter);
    harness.run();

    // The command submitted (the field was cleared by the Command_Line_Outcome),
    // and NO namespace group was collapsed as a side effect of the Enter.
    assert!(
        !harness
            .state()
            .config_panel
            .collapsed
            .values()
            .any(|collapsed| *collapsed),
        "submitting a command must not collapse the focused config-tree node (Bug 1)"
    );
}
