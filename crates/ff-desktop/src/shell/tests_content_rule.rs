//! Live-screen tests for the common-frame Content_Area boundary rule
//! (workspace-framework Requirement 7.30, 7.31): ONE shell-owned separator sits
//! between the `Command ===>` panel and the central panel, for every Workspace,
//! and it is never a focus stop.

use super::render_content_rule::{content_rule_line_id, primary_content_rule_panel_id};
use super::tests_common::*;

fn rect_of(harness: &egui_kittest::Harness<'_, super::WorkbenchShell>, id: egui::Id) -> egui::Rect {
    harness
        .ctx
        .read_response(id)
        .map(|r| r.rect)
        .expect("response rect recorded")
}

// Validates: workspace-framework Requirement 7.30, 7.31 -- on the POM (a Menu
// Kind) the rule exists directly below the Command line and above the content.
#[test]
fn boundary_rule_sits_between_command_panel_and_central_panel_on_pom() {
    let mut harness = harness_shell();
    for _ in 0..4 {
        harness.run();
    }
    let cmd = rect_of(&harness, cmd_field_id());
    let rule = rect_of(
        &harness,
        content_rule_line_id(primary_content_rule_panel_id()),
    );
    assert!(rule.height() > 0.0, "the rule has a visible height");
    assert!(
        rule.top() >= cmd.bottom(),
        "rule {rule:?} must be below the command field {cmd:?}"
    );
    let available = harness.ctx.available_rect();
    assert!(
        available.top() >= rule.bottom() - 0.5,
        "the central area {available:?} starts at or below the rule {rule:?}"
    );
}

// Validates: workspace-framework Requirement 7.30, 7.31 -- the SAME rule appears
// for a non-menu workspace (Config), identical placement, no workspace branch.
#[test]
fn boundary_rule_is_identical_for_config_workspace() {
    let mut harness = harness_shell();
    for _ in 0..4 {
        harness.run();
    }
    let pom_rule = rect_of(
        &harness,
        content_rule_line_id(primary_content_rule_panel_id()),
    );
    harness.state_mut().dispatch_command_string("CONFIG");
    for _ in 0..4 {
        harness.run();
    }
    let rule = rect_of(
        &harness,
        content_rule_line_id(primary_content_rule_panel_id()),
    );
    assert_eq!(rule, pom_rule, "same frame rule for every workspace");
}

// Validates: workspace-framework Requirement 7.30 -- the rule is NON-focusable:
// it never takes keyboard focus, so it cannot be a Tab stop.
#[test]
fn boundary_rule_is_never_focused() {
    let mut harness = harness_shell();
    for _ in 0..4 {
        harness.run();
    }
    let rule_id = content_rule_line_id(primary_content_rule_panel_id());
    for _ in 0..6 {
        harness.key_press(egui::Key::Tab);
        harness.run();
        assert_ne!(harness.ctx.memory(|m| m.focused()), Some(rule_id));
    }
}
