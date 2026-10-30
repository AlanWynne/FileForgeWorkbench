//! Full-shell tests for Tab / Shift+Tab moving ALONG the menu bar WHILE a
//! dropdown is open (menu-workspace Requirement 17.5g).
//!
//! Split from `tests_menu_bar_keynav_bar.rs` (test-file size rule). The reported
//! bug: once a menu-bar dropdown is OPEN, Tab stopped doing anything (the key
//! was effectively swallowed). Option A (user-confirmed): while a dropdown is
//! open, Tab behaves like Right and Shift+Tab like Left -- moving focus to the
//! next/previous heading and opening its dropdown, clamping at the last/first
//! heading (no wrap), reusing the SAME code path as the Left/Right walk.
//!
//! A regression guard proves that when NO dropdown is open, Tab is NOT hijacked
//! by the bar and still advances normal focus order.

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

/// True when the dropdown owned by `heading` is currently open in egui memory
/// (egui 0.33: a `menu_button`'s popup id is `button_id.with("popup")`).
fn dropdown_open_for(
    harness: &egui_kittest::Harness<super::WorkbenchShell>,
    heading: egui::Id,
) -> bool {
    egui::Popup::is_id_open(&harness.ctx, heading.with("popup"))
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

// Validates: menu-workspace Req 17.5g -- with a heading focused (dropdown open),
// Tab moves focus to the NEXT heading along the bar and OPENS its dropdown
// (identical to pressing Right).
#[test]
fn full_shell_menu_bar_tab_moves_to_next_heading_and_opens_dropdown() {
    let mut harness = harness_shell();
    focus_first_heading(&mut harness);
    let headings = harness.state().focus.menu_bar_heading_ids.clone();
    assert!(
        headings.len() >= 2,
        "default bar must have at least two headings for this test"
    );

    harness.key_press(egui::Key::Tab);
    harness.run();
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(headings[1]),
        "Tab must move focus to the next heading (like Right)"
    );
    assert!(
        dropdown_open_for(&harness, headings[1]),
        "Tab must open the newly-focused heading's dropdown"
    );
}

// Validates: menu-workspace Req 17.5g -- with a dropdown open, Shift+Tab moves
// focus to the PREVIOUS heading and opens its dropdown (identical to Left).
#[test]
fn full_shell_menu_bar_shift_tab_moves_to_prev_heading_and_opens_dropdown() {
    let mut harness = harness_shell();
    let menu_first = focus_first_heading(&mut harness);
    let headings = harness.state().focus.menu_bar_heading_ids.clone();
    assert!(headings.len() >= 2);

    // Tab to the second heading first.
    harness.key_press(egui::Key::Tab);
    harness.run();
    harness.run();
    assert_eq!(harness.ctx.memory(|m| m.focused()), Some(headings[1]));

    // Shift+Tab back to the first.
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::Tab);
    harness.run();
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(menu_first),
        "Shift+Tab must move focus back to the previous heading (like Left)"
    );
    assert!(
        dropdown_open_for(&harness, menu_first),
        "Shift+Tab must open the previous heading's dropdown"
    );
}

// Validates: menu-workspace Req 17.5g -- Tab at the LAST heading clamps (stays
// on the last, matching Right); Shift+Tab at the FIRST heading clamps (stays on
// the first, matching Left). No wrap.
#[test]
fn full_shell_menu_bar_tab_clamps_at_last_shift_tab_clamps_at_first() {
    let mut harness = harness_shell();
    let menu_first = focus_first_heading(&mut harness);
    let headings = harness.state().focus.menu_bar_heading_ids.clone();
    let last = *headings.last().expect("at least one heading");

    // Repeated Tab past the end must clamp on the LAST heading (like Right).
    for _ in 0..headings.len() + 2 {
        harness.key_press(egui::Key::Tab);
        harness.run();
        harness.run();
    }
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(last),
        "repeated Tab must clamp on the LAST heading, not wrap"
    );
    assert!(
        dropdown_open_for(&harness, last),
        "the clamped last heading keeps its dropdown open"
    );

    // Repeated Shift+Tab back past the start must clamp on the FIRST heading.
    for _ in 0..headings.len() + 2 {
        harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::Tab);
        harness.run();
        harness.run();
    }
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(menu_first),
        "repeated Shift+Tab must clamp on the FIRST heading, not wrap"
    );
    assert!(
        dropdown_open_for(&harness, menu_first),
        "the clamped first heading keeps its dropdown open"
    );
}

// Validates: menu-workspace Req 17.5g (regression guard) -- when NO menu-bar
// dropdown is open, Tab is NOT hijacked by the bar: it advances normal focus
// order. From the command field (entry focus), the first Tab lands on the POM's
// reported first interior control (the unified Tab-order model, Req 15/17.6),
// exactly as before this feature.
#[test]
fn full_shell_tab_not_hijacked_when_no_dropdown_open() {
    let mut harness = harness_shell();
    // Entry focus is the command field; no dropdown is open.
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "precondition: command field holds focus, no dropdown open"
    );
    let first_interior = harness.state().focus.first_interior_id;
    assert!(
        first_interior.is_some(),
        "POM must report a first interior control"
    );

    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        first_interior,
        "with no dropdown open, Tab must advance normal focus order (not be hijacked by the bar)"
    );
}
