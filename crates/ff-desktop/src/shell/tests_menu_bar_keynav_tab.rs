//! Full-shell tests for opening a menu-bar dropdown with Down when a heading is
//! keyboard-focused but NO dropdown is open yet (menu-workspace Req 17.5f).
//!
//! Split from `tests_menu_bar_keynav_bar.rs` (test-file size rule). The reported
//! bug: after TABbing to a bar heading (e.g. Settings) with no dropdown open,
//! pressing Down did nothing. Expected: Down OPENS that heading's dropdown and
//! moves focus to its FIRST item -- the same end state as clicking the heading
//! then pressing Down (Req 17.5a). These tests pin that behaviour and guard the
//! regression where an over-broad arrow consume starved other Down consumers.

#![allow(unused_imports)]
use super::tests_common::*;
use eframe::egui;

/// Put keyboard focus on the FIRST top-level menu-bar heading WITHOUT opening
/// its dropdown (the state a user reaches by Tabbing to the heading), then
/// settle a frame. Returns the heading id. This mirrors the Tab-landing state:
/// the bar reports the heading id (`menu_first_id`), focus is forced onto it,
/// and no `Popup` is open.
fn focus_first_heading_no_dropdown(
    harness: &mut egui_kittest::Harness<super::WorkbenchShell>,
) -> egui::Id {
    let menu_first = harness
        .state()
        .focus
        .menu_first_id
        .expect("menu bar must report its first top-level button id");
    harness.ctx.memory_mut(|m| m.request_focus(menu_first));
    harness.run();
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(menu_first),
        "precondition: the first heading holds keyboard focus"
    );
    assert!(
        !egui::Popup::is_id_open(&harness.ctx, menu_first.with("popup")),
        "precondition: no dropdown is open on the focused heading"
    );
    menu_first
}

// Validates: menu-workspace Req 17.5f -- with a bar heading keyboard-focused and
// its dropdown NOT open, Down OPENS the dropdown and focuses its FIRST item (the
// same end state as click/Enter then Down).
#[test]
fn full_shell_tab_to_heading_then_down_opens_dropdown_first_item() {
    let mut harness = harness_shell();
    let menu_first = focus_first_heading_no_dropdown(&mut harness);

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();

    assert!(
        egui::Popup::is_id_open(&harness.ctx, menu_first.with("popup")),
        "Down on a focused heading must OPEN its dropdown"
    );
    let focused = harness.ctx.memory(|m| m.focused());
    assert!(
        focused.is_some(),
        "Down must move focus into the newly-opened dropdown"
    );
    assert_ne!(
        focused,
        Some(menu_first),
        "Down must move focus OFF the heading, onto the first dropdown item"
    );
    assert_ne!(
        focused,
        Some(cmd_field_id()),
        "Down must not leave focus on the command field"
    );
}

// Validates: menu-workspace Req 17.5f, 17.5b, 17.5d -- after Down opens the
// dropdown from a Tab-focused heading, further Down/Up walk the dropdown as
// before, and Up from the first item returns to the heading.
#[test]
fn full_shell_tab_opened_dropdown_walks_and_returns_to_heading() {
    let mut harness = harness_shell();
    let menu_first = focus_first_heading_no_dropdown(&mut harness);

    // Down opens + focuses item 0.
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    let first_item = harness.ctx.memory(|m| m.focused());
    assert_ne!(first_item, Some(menu_first));

    // A further Down advances to item 1.
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    let second_item = harness.ctx.memory(|m| m.focused());
    assert_ne!(
        second_item, first_item,
        "a further Down must advance to the next dropdown item"
    );

    // Up returns to item 0.
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        first_item,
        "Up must return to the previous item"
    );

    // Up from item 0 returns to the parent heading (Req 17.5d).
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(menu_first),
        "Up from the first item must return focus to the heading"
    );
}

// Validates: menu-workspace Req 17.5f (regression guard) -- when focus is NOT on
// a bar heading (here the command field holds focus and the active Context is
// the POM), the new Down-opens-dropdown behaviour MUST NOT fire: Down opens no
// menu-bar dropdown, and the command field still receives arrows (Up recalls
// history). This pins the regression where an over-broad arrow consume starved
// the command field.
#[test]
fn full_shell_down_not_stolen_from_command_field() {
    let mut harness = harness_shell();
    // Seed history so Up recall has something to step through.
    seed_history(&mut harness, &["FIRST", "SECOND"]);

    // Focus the command field (entry focus already there, but be explicit) and
    // confirm no bar heading is focused.
    harness.ctx.memory_mut(|m| m.request_focus(cmd_field_id()));
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "precondition: the command field holds focus, not a bar heading"
    );
    let menu_first = harness.state().focus.menu_first_id;

    // Down with the command field focused must NOT open a bar dropdown (the new
    // Down-opens behaviour is gated on a heading holding focus).
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    if let Some(heading) = menu_first {
        assert!(
            !egui::Popup::is_id_open(&harness.ctx, heading.with("popup")),
            "Down must NOT open a menu-bar dropdown when the command field is focused"
        );
    }
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "Down must leave command-field focus intact (not stolen by the bar)"
    );

    // And the command field still receives arrows: Up recalls history.
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(
        harness.state().command_text,
        "SECOND",
        "Up must still recall history when focus is on the command field"
    );
}

// Validates: menu-workspace Req 17.5f (regression guard) -- when focus is on the
// config tree (not a bar heading), Down is NOT consumed by the menu bar: the
// config-tree cursor still moves. Pins the regression against central-panel
// consumers.
#[test]
fn full_shell_down_not_stolen_from_config_tree() {
    use crate::config_panel::ConfigNodeId;
    use crate::tab_state::KindTag;

    let mut harness = harness_shell();
    harness.state_mut().dispatch_command_string("CONFIG");
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel
    );

    harness.ctx.memory_mut(|m| m.stop_text_input());
    harness
        .ctx
        .memory_mut(|m| m.request_focus(egui::Id::new("config_tree_focus_none")));
    harness
        .ctx
        .memory_mut(|m| m.surrender_focus(egui::Id::new("config_tree_focus_none")));

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    let cursor = harness.state().config_panel.cursor.clone();
    assert!(
        matches!(cursor, Some(ConfigNodeId::Namespace(_))),
        "Down must still move the config-tree cursor when no heading is focused, got {cursor:?}"
    );
}
