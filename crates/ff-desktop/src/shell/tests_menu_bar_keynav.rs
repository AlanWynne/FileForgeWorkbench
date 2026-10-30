//! Full-shell keyboard-navigation tests for the horizontal Menu_Bar dropdowns
//! (menu-workspace Requirement 17.5, B-fix).
//!
//! These drive the REAL `WorkbenchShell` headlessly (egui_kittest `build_eframe`)
//! so the whole open-dropdown -> arrow-key -> Enter path is exercised end to end.
//! They guard the reported regression: the first Down then Enter on the Settings
//! dropdown invoked the WRONG item (Aliases) because a manual focus walk
//! double-advanced on top of egui's own navigation. The shell now OWNS the walk
//! as a tracked item index (arrow keys are consumed so egui never also moves
//! focus) and FORCES the selected item into focus after egui's MenuBar runs, so
//! Down focuses item 0 (Config) and the SECOND item (Aliases) is reached only by
//! a further Down.

#![allow(unused_imports)]
use super::tests_common::*;
use eframe::egui;

/// Open the FIRST top-level menu-bar dropdown (Settings) by focusing its button
/// and pressing Enter (keyboard open), then settle a few frames so the dropdown
/// content closure is rendering. Returns the focused id after opening (the menu
/// button itself -- NOT yet a dropdown item).
fn open_first_dropdown(harness: &mut egui_kittest::Harness<super::WorkbenchShell>) -> egui::Id {
    let menu_first = harness
        .state()
        .focus
        .menu_first_id
        .expect("menu bar must report its first top-level button id");
    // Put keyboard focus on the first top-level button and open it with Enter.
    harness.ctx.memory_mut(|m| m.request_focus(menu_first));
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    harness.run();
    menu_first
}

// Validates: menu-workspace Req 17.5a -- when a menu-bar dropdown is open and no
// item is focused yet, the FIRST Down arrow moves focus INTO the dropdown, onto
// the first item (not left on the top-level button, not the command field).
#[test]
fn full_shell_menu_bar_down_focuses_first_dropdown_item() {
    let mut harness = harness_shell();
    let menu_first = open_first_dropdown(&mut harness);

    // Press Down: requested focus applies next frame, so run twice.
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();

    let focused = harness.ctx.memory(|m| m.focused());
    assert!(
        focused.is_some(),
        "Down must move focus somewhere in the open dropdown"
    );
    assert_ne!(
        focused,
        Some(cmd_field_id()),
        "Down must not leave focus on the command field"
    );
    assert_ne!(
        focused,
        Some(menu_first),
        "Down must move focus OFF the top-level button, onto the first dropdown item"
    );
}

// Validates: menu-workspace Req 17.5b -- pressing Down again advances focus to
// the NEXT dropdown item (the Settings dropdown has several items).
#[test]
fn full_shell_menu_bar_down_advances_through_dropdown_items() {
    let mut harness = harness_shell();
    open_first_dropdown(&mut harness);

    // First Down: focus the first item.
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    let first_item = harness.ctx.memory(|m| m.focused());
    assert!(first_item.is_some(), "first Down focuses the first item");

    // Second Down: focus the next item.
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    let second_item = harness.ctx.memory(|m| m.focused());
    assert!(
        second_item.is_some(),
        "second Down keeps focus in the dropdown"
    );
    assert_ne!(
        second_item, first_item,
        "a second Down must advance focus to the NEXT dropdown item"
    );
}

// Validates: menu-workspace Req 17.5b -- Up after Down returns focus to the
// previous item (clamped at the first; here Down,Down,Up lands back on item 0).
#[test]
fn full_shell_menu_bar_up_returns_to_previous_item() {
    let mut harness = harness_shell();
    open_first_dropdown(&mut harness);

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    let first_item = harness.ctx.memory(|m| m.focused());

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    let second_item = harness.ctx.memory(|m| m.focused());
    assert_ne!(second_item, first_item);

    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    harness.run();
    let back = harness.ctx.memory(|m| m.focused());
    assert_eq!(
        back, first_item,
        "Up must move focus back to the previous dropdown item"
    );
}

// Validates: menu-workspace Req 17.5c -- Enter on a focused dropdown item invokes
// it through the SAME command path as a click. The Settings dropdown's first item
// is `Config`, so Down then Enter must change the active Context to the Config
// panel (ConfigPanel), exactly as typing CONFIG / clicking the item would.
#[test]
fn full_shell_menu_bar_enter_invokes_focused_item() {
    use crate::tab_state::KindTag;
    let mut harness = harness_shell();
    // Precondition: active Context is the POM Menu Workspace (not Config yet).
    assert_ne!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "precondition: not already on the Config Context"
    );

    open_first_dropdown(&mut harness);

    // Down focuses the first item (Config), Enter invokes it.
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    for _ in 0..4 {
        harness.run();
    }

    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "Enter on the focused first Settings item (Config) must open the Config Context"
    );
}

// Validates: menu-workspace Req 17.5b, 17.5c -- a SECOND Down focuses the second
// Settings item (Aliases), and Enter there invokes it through the same command
// path, opening the Alias Editor Context (AliasEditor). This is the counterpart
// of the first-item test: it pins the exact off-by-one that caused the reported
// regression -- a single Down must NOT already be sitting on Aliases. Config is
// item 0 (one Down) and Aliases is item 1 (two Downs).
#[test]
fn full_shell_menu_bar_enter_invokes_second_item_opens_alias_editor() {
    use crate::tab_state::KindTag;
    let mut harness = harness_shell();
    assert_ne!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "precondition: not already on the Alias Editor Context"
    );

    open_first_dropdown(&mut harness);

    // Down,Down focuses the SECOND item (Aliases), Enter invokes it.
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    for _ in 0..4 {
        harness.run();
    }

    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "Enter on the focused second Settings item (Aliases) must open the Alias Editor Context"
    );
}
