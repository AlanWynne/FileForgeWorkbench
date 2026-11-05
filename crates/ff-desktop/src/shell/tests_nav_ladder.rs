//! CR-CH-116 ISPF END / RETURN / EXIT model -- full-shell egui_kittest tests.
//!
//! Proves the model end-to-end through the real `WorkbenchShell` (headless
//! `build_eframe` via `harness_shell` / `make_shell`):
//!   - bare `X` is the FFCMD seed alias of EXIT, so on every NON-editor Context
//!     it terminates the app UNCONDITIONALLY (the old nav_x collapse/close/exit-
//!     when-last verb is RETIRED);
//!   - `=X` reinitialises to the POM (FFCMD) then runs `X` -> EXIT (also exit);
//!   - END (PF3) pops exactly ONE level of the Navigation_Stack;
//!   - RETURN unwinds to the tab's primary panel (Tab_Visual_Root) in one step;
//!   - on an editor Context FFEDIT claims bare `X` as EXCLUDE (active-wins), so
//!     X does NOT exit there; `=X` escapes to FFCMD and exits.
//!
//! These assert on the shell model (active tab kind / is_home / nav_stack depth /
//! tab count / `should_close`), not pixels, within a bounded frame budget.

#![allow(unused_imports)]
use super::tests_common::*;
use crate::tab_state::{KindTag, TabKind};

/// Validates: navigation-commands Requirement 21.4; menu-workspace Req 14.13
/// (CR-CH-116) -- bare `X` on a non-editor Context ABOVE the visual root
/// terminates the app via the FFCMD X -> EXIT seed (it no longer collapses to the
/// visual root). The old nav_x collapse behaviour is retired.
#[test]
fn bare_x_exits_app_from_drilled_non_editor_context() {
    let mut shell = make_shell();
    // POM root, then drill into CONFIG (non-editor Context above the root).
    shell.set_active_tab_home();
    shell.dispatch_command_string("CONFIG");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "precondition: drilled into the Config Context"
    );
    assert!(!shell.tabs.active_tab().nav_stack.is_empty());

    shell.dispatch_command_string("X");

    assert!(
        *shell.should_close.lock().expect("close lock"),
        "bare X on a non-editor Context terminates the app (X -> EXIT, CR-CH-116)"
    );
}

/// Validates: navigation-commands Requirement 21.4; menu-workspace Req 14.13
/// (CR-CH-116) -- bare `X` at the visual root of a non-editor Context exits the
/// app unconditionally, even with other tabs open (no "close just this tab").
#[test]
fn bare_x_exits_app_even_with_other_tabs_open() {
    let mut shell = make_shell();
    shell.set_active_tab_home();
    // A second POM tab so that, under the OLD model, X would merely close one
    // tab. Under CR-CH-116 X exits the app regardless.
    shell.dispatch_command_string("START");
    assert!(
        shell.tabs.len() >= 2,
        "precondition: more than one tab open"
    );
    assert!(
        shell.tabs.active_tab().nav_stack.is_empty(),
        "precondition: the new START tab is at its visual root (empty stack)"
    );

    shell.dispatch_command_string("X");

    assert!(
        *shell.should_close.lock().expect("close lock"),
        "bare X exits the app unconditionally (CR-CH-116), not close-one-tab"
    );
}

/// Validates: navigation-commands Requirement 21.5; menu-workspace Req 14.14
/// (CR-CH-116) -- `=X` reinitialises to the POM (FFCMD) then runs `X` -> EXIT, so
/// it terminates the app.
#[test]
fn equals_x_exits_app() {
    let mut shell = make_shell();
    shell.set_active_tab_home();
    shell.dispatch_command_string("=X");
    assert!(
        *shell.should_close.lock().expect("close lock"),
        "`=X` reinitialises to POM then runs X -> EXIT, terminating the app"
    );
}

/// Validates: navigation-commands Requirement 21.1 (CR-CH-116 confirms unchanged)
/// -- END (PF3) pops ONE level of the Navigation_Stack.
#[test]
fn end_pops_one_level() {
    let mut harness = harness_shell();
    // Build a 2-deep stack: POM -> CONFIG (stack [POM]) -> KEYS (stack [POM, CONFIG]).
    harness.state_mut().dispatch_command_string("CONFIG");
    for _ in 0..4 {
        harness.run();
    }
    harness.state_mut().dispatch_command_string("KEYS");
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::KeysEditor,
        "precondition: drilled two levels (on Keys)"
    );
    let depth_before = harness.state().tabs.active_tab().nav_stack.len();
    assert_eq!(depth_before, 2, "precondition: stack depth 2 [POM, CONFIG]");

    harness.state_mut().dispatch_command_string("END");
    for _ in 0..4 {
        harness.run();
    }

    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "END pops ONE level, back to Config"
    );
    assert_eq!(
        harness.state().tabs.active_tab().nav_stack.len(),
        depth_before - 1,
        "END decreases the stack depth by exactly one"
    );
    assert!(
        !*harness.state().should_close.lock().expect("close lock"),
        "END pops a level -- it is NOT app-exit (PF3 stays END, CR-CH-116)"
    );
}

/// Validates: navigation-commands Requirement 21.2 (CR-CH-116) -- RETURN unwinds
/// the Navigation_Stack to the tab's primary panel (Tab_Visual_Root) in one
/// step, and does NOT app-exit when above the root.
#[test]
fn return_unwinds_to_primary_panel() {
    let mut harness = harness_shell();
    // POM -> CONFIG -> KEYS (two deep above the POM visual root).
    harness.state_mut().dispatch_command_string("CONFIG");
    for _ in 0..4 {
        harness.run();
    }
    harness.state_mut().dispatch_command_string("KEYS");
    for _ in 0..4 {
        harness.run();
    }
    assert!(!harness.state().tabs.active_tab().nav_stack.is_empty());

    harness.state_mut().dispatch_command_string("RETURN");
    for _ in 0..4 {
        harness.run();
    }

    assert!(
        harness.state().tabs.active_tab().is_home,
        "RETURN unwinds to the primary panel (the POM visual root) in one step"
    );
    assert!(
        harness.state().tabs.active_tab().nav_stack.is_empty(),
        "RETURN clears the Navigation_Stack back to the primary panel"
    );
    assert!(
        !*harness.state().should_close.lock().expect("close lock"),
        "RETURN above the root does not app-exit"
    );
}

/// Validates: navigation-commands Requirement 21.4 (CR-CH-116) -- X = EXIT is
/// UNIFORM across the POM and a non-POM (START SETTINGS) Workspace: bare `X` on
/// either terminates the app; the POM is not special-cased.
#[test]
fn x_exits_uniformly_pom_and_non_pom() {
    // Shell A: a POM-rooted tab drilled into CONFIG.
    let mut a = make_shell();
    a.set_active_tab_home();
    a.dispatch_command_string("CONFIG");
    assert_eq!(a.tabs.active_tab().kind.tag(), KindTag::ConfigPanel);
    a.dispatch_command_string("X");
    let a_closed = *a.should_close.lock().expect("close lock");

    // Shell B: a SETTINGS-rooted tab (START SETTINGS) drilled into CONFIG.
    let mut b = make_shell();
    b.set_active_tab_home();
    b.dispatch_command_string("START SETTINGS");
    assert!(
        !b.tabs.active_tab().is_home,
        "precondition: START SETTINGS roots a non-POM Workspace"
    );
    b.dispatch_command_string("CONFIG");
    assert_eq!(b.tabs.active_tab().kind.tag(), KindTag::ConfigPanel);
    b.dispatch_command_string("X");
    let b_closed = *b.should_close.lock().expect("close lock");

    assert!(a_closed, "POM-rooted tab: X exits the app");
    assert!(b_closed, "SETTINGS-rooted tab: X exits the app");
    assert_eq!(
        a_closed, b_closed,
        "X = EXIT is uniform: the POM is not special-cased (CR-CH-116)"
    );
}

/// Validates: navigation-commands Requirement 21.5; command-configurator Req 8.5
/// (CR-CH-116) -- on an editor Context FFEDIT claims bare `X` as EXCLUDE
/// (active-wins), so X does NOT exit; `=X` escapes to FFCMD and exits.
#[test]
fn ffedit_bare_x_stays_exclude_but_equals_x_exits() {
    // Bare X on an editor Context: FFEDIT EXCLUDE, not an exit.
    let mut harness = harness_shell();
    harness.state_mut().shell_new_untitled();
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::Untitled
    );
    let tabs_before = harness.state().tabs.len();

    harness.state_mut().dispatch_command_string("X ALL");
    for _ in 0..4 {
        harness.run();
    }
    assert!(
        !*harness.state().should_close.lock().expect("close lock"),
        "bare `X ALL` on an editor Context is FFEDIT EXCLUDE, not an exit"
    );
    assert_eq!(
        harness.state().tabs.len(),
        tabs_before,
        "FFEDIT EXCLUDE does not close/exit the editor Workspace"
    );

    // `=X` escapes FFEDIT: reinit to POM (FFCMD) then X -> EXIT terminates.
    // Assert the close flag directly after dispatch WITHOUT running frames: the
    // file.exit dispatch keeps requesting repaint (the shutdown signal), which
    // would exceed the harness frame budget. The close flag is the authoritative
    // app-exit signal.
    harness.state_mut().dispatch_command_string("=X");
    assert!(
        *harness.state().should_close.lock().expect("close lock"),
        "`=X` escapes FFEDIT (reinit to POM) then X -> EXIT terminates the app"
    );
}

/// Validates: command-framework Requirement 10.2 (CR-CH-073) -- `=0` from a
/// non-POM tab reinitialises to the POM and resolves option 0 (Settings -> the
/// Settings Menu_Workspace), proving `=` targets the FFCMD_Root from ANY tab.
#[test]
fn equals_0_works_from_a_non_pom_tab() {
    let mut harness = harness_shell();
    harness.state_mut().dispatch_command_string("START KEYS");
    for _ in 0..4 {
        harness.run();
    }
    assert!(
        !harness.state().tabs.active_tab().is_home,
        "precondition: on a non-POM (Keys) tab"
    );

    harness.state_mut().dispatch_command_string("=0");
    for _ in 0..4 {
        harness.run();
    }

    // POM option 0 is `Settings` -> the Settings Menu_Workspace.
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::MenuWorkspace,
        "`=0` from a non-POM tab resolves as POM option 0 (Settings)"
    );
}

/// Validates: navigation-commands Requirement 21.4; menu-workspace Req 14.16
/// (CR-CH-116) -- a Detached_Workspace is a non-editor Context, so bare `X`
/// there also exits the app (X = EXIT everywhere). END/RETURN still close it.
#[test]
fn detached_workspace_x_exits_app() {
    let mut shell = make_shell();
    shell.set_active_tab_home();
    // Create a second tab and mark it floating (detached) at its visual root.
    shell.dispatch_command_string("START");
    let detached_idx = shell.tabs.active_index();
    if let Some(t) = shell.tabs.tabs_mut().get_mut(detached_idx) {
        t.is_floating = true;
        t.nav_stack.clear();
    }

    shell.dispatch_command_string("X");

    assert!(
        *shell.should_close.lock().expect("close lock"),
        "bare X on a detached (non-editor) Workspace exits the app (X = EXIT, CR-CH-116)"
    );
}
