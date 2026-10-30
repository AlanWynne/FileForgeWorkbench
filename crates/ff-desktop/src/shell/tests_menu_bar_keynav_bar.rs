//! Full-shell tests for Menu_Bar keyboard navigation ALONG the bar and the
//! Up-to-heading transition (menu-workspace Requirement 17.5d/17.5e).
//!
//! Split from `tests_menu_bar_keynav.rs` (test-file size rule): that file covers
//! the dropdown item walk (Down/Up/Enter inside an open dropdown); this one
//! covers the two behaviours added to the shell-owned walk:
//!
//! - Up from the FIRST dropdown item returns focus to the parent heading
//!   (Req 17.5d), and Down from the heading re-enters the dropdown;
//! - Left/Right move focus between the top-level headings along the bar and
//!   OPEN the newly-focused heading's dropdown, clamping at the first/last
//!   heading (Req 17.5e).
//!
//! A regression guard proves Left/Right are NOT consumed by the bar when no
//! dropdown is open (the prior arrow-consume regression).

#![allow(unused_imports)]
use super::tests_common::*;
use eframe::egui;

/// Open the FIRST top-level menu-bar dropdown (Settings) by focusing its button
/// and pressing Enter, then settle a few frames. Returns the heading id.
fn open_first_dropdown(harness: &mut egui_kittest::Harness<super::WorkbenchShell>) -> egui::Id {
    let menu_first = harness
        .state()
        .focus
        .menu_first_id
        .expect("menu bar must report its first top-level button id");
    harness.ctx.memory_mut(|m| m.request_focus(menu_first));
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    harness.run();
    menu_first
}

/// The egui id of the open menu-bar dropdown's popup for a given heading button
/// id (egui 0.33: a `menu_button`'s popup id is `button_id.with("popup")`).
fn dropdown_popup_id(heading: egui::Id) -> egui::Id {
    heading.with("popup")
}

/// True when the dropdown owned by `heading` is currently open in egui memory.
fn dropdown_open_for(
    harness: &egui_kittest::Harness<super::WorkbenchShell>,
    heading: egui::Id,
) -> bool {
    egui::Popup::is_id_open(&harness.ctx, dropdown_popup_id(heading))
}

/// Open the first dropdown and return focus to its heading (bar keyboard mode):
/// Down into item 0, then Up back to the heading. Returns the heading id.
fn focus_first_heading(harness: &mut egui_kittest::Harness<super::WorkbenchShell>) -> egui::Id {
    let menu_first = open_first_dropdown(harness);
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(menu_first),
        "precondition: Up from item 0 focuses the heading"
    );
    menu_first
}

// Validates: menu-workspace Req 17.5d -- when focus is on the FIRST item of an
// open dropdown and Up is pressed, focus moves UP to the PARENT heading button
// (the top-level bar button), NOT clamped on the first item.
#[test]
fn full_shell_menu_bar_up_from_first_item_focuses_heading() {
    let mut harness = harness_shell();
    let menu_first = open_first_dropdown(&mut harness);

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    let first_item = harness.ctx.memory(|m| m.focused());
    assert_ne!(
        first_item,
        Some(menu_first),
        "precondition: Down moved focus onto the first dropdown item"
    );

    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(menu_first),
        "Up from the first dropdown item must focus the parent heading"
    );
}

// Validates: menu-workspace Req 17.5d -- after Up returns focus to the heading,
// Down re-enters the dropdown at its first item (consistent with opening).
#[test]
fn full_shell_menu_bar_down_from_heading_reenters_dropdown() {
    let mut harness = harness_shell();
    let menu_first = open_first_dropdown(&mut harness);

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    let first_item = harness.ctx.memory(|m| m.focused());

    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    harness.run();
    assert_eq!(harness.ctx.memory(|m| m.focused()), Some(menu_first));

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        first_item,
        "Down from the heading must re-enter the dropdown at its first item"
    );
}

// Validates: menu-workspace Req 17.5e -- with a heading focused (dropdown open),
// Right moves focus to the NEXT heading along the bar and OPENS its dropdown.
#[test]
fn full_shell_menu_bar_right_moves_to_next_heading_and_opens_dropdown() {
    let mut harness = harness_shell();
    focus_first_heading(&mut harness);
    let headings = harness.state().focus.menu_bar_heading_ids.clone();
    assert!(
        headings.len() >= 2,
        "default bar must have at least two headings for this test"
    );

    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(headings[1]),
        "Right must move focus to the next heading"
    );
    assert!(
        dropdown_open_for(&harness, headings[1]),
        "moving Right must open the newly-focused heading's dropdown"
    );
}

// Validates: menu-workspace Req 17.5e -- Left moves focus to the PREVIOUS
// heading and opens its dropdown; Left at the FIRST heading clamps (stays).
#[test]
fn full_shell_menu_bar_left_moves_to_prev_heading_and_clamps() {
    let mut harness = harness_shell();
    let menu_first = focus_first_heading(&mut harness);
    let headings = harness.state().focus.menu_bar_heading_ids.clone();
    assert!(headings.len() >= 2);

    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    harness.run();
    assert_eq!(harness.ctx.memory(|m| m.focused()), Some(headings[1]));

    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(menu_first),
        "Left must move focus back to the previous heading"
    );
    assert!(
        dropdown_open_for(&harness, menu_first),
        "moving Left must open the previous heading's dropdown"
    );

    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(menu_first),
        "Left at the first heading must clamp, not wrap"
    );
}

// Validates: menu-workspace Req 17.5e -- Right at the LAST heading clamps (stays
// on the last heading), matching the desktop convention of clamp (not wrap).
#[test]
fn full_shell_menu_bar_right_at_last_heading_clamps() {
    let mut harness = harness_shell();
    focus_first_heading(&mut harness);
    let headings = harness.state().focus.menu_bar_heading_ids.clone();
    let last = *headings.last().expect("at least one heading");

    for _ in 0..headings.len() + 2 {
        harness.key_press(egui::Key::ArrowRight);
        harness.run();
        harness.run();
    }
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(last),
        "repeated Right must clamp on the LAST heading, not wrap past it"
    );
    assert!(
        dropdown_open_for(&harness, last),
        "the clamped last heading keeps its dropdown open"
    );
}

// Validates: menu-workspace Req 17.5e (regression guard) -- when NO menu-bar
// dropdown is open, Left/Right are NOT consumed by the bar: the Config tree's
// own Left/Right (collapse/expand a group) still works. This pins the prior
// regression where an unconditional arrow consume starved other consumers.
#[test]
fn full_shell_left_right_not_stolen_from_config_tree() {
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
    let ns = match harness.state().config_panel.cursor.clone() {
        Some(ConfigNodeId::Namespace(ns)) => ns,
        other => panic!("expected a namespace cursor, got {other:?}"),
    };

    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    assert_eq!(
        harness.state().config_panel.collapsed.get(&ns).copied(),
        Some(true),
        "Left must still collapse the config group when no dropdown is open"
    );
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(
        harness.state().config_panel.collapsed.get(&ns).copied(),
        Some(false),
        "Right must still expand the config group when no dropdown is open"
    );
}
