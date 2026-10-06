//! CR-CH-052 uniform navigation / exit model -- full-shell egui_kittest tests.
//!
//! Proves the TWO-ROOTS model end-to-end through the real `WorkbenchShell`
//! (headless `build_eframe` via `harness_shell` / `make_shell`): the `=` family
//! targets the FFCMD_Root (the POM), while bare `X` / END / RETURN target the
//! Tab_Visual_Root. The behaviour is UNIFORM across the POM and a non-POM
//! (`START SETTINGS`) Workspace -- there is no POM special-casing.
//!
//! Model: `nav_x` collapses to the Tab_Visual_Root (`nav_stack[0]`) when the
//! stack is non-empty, else closes the Workspace (exit only when last). `=` is
//! the single front-door reinitialise-to-POM step. These assert on the shell
//! model (active tab kind / is_home / nav_stack depth / tab count /
//! `should_close`), not pixels, within a bounded frame budget.

#![allow(unused_imports)]
use super::tests_common::*;
use crate::tab_state::{KindTag, TabKind};

/// Validates: menu-workspace Requirement 14.13 (CR-CH-052) -- bare `X` ABOVE the
/// Tab_Visual_Root collapses to that root in one action; it does not close.
#[test]
fn bare_x_collapses_to_visual_root_when_above_root() {
    let mut harness = harness_shell();
    // POM -> CONFIG pushes the POM onto the stack (visual root = POM).
    harness.state_mut().dispatch_command_string("CONFIG");
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "precondition: drilled into the Config Context"
    );
    assert!(!harness.state().tabs.active_tab().nav_stack.is_empty());

    harness.state_mut().dispatch_command_string("X");
    for _ in 0..4 {
        harness.run();
    }

    assert!(
        harness.state().tabs.active_tab().is_home,
        "bare X collapses to the Tab_Visual_Root (the POM)"
    );
    assert!(
        harness.state().tabs.active_tab().nav_stack.is_empty(),
        "collapse clears the Navigation_Stack"
    );
    assert!(
        !*harness.state().should_close.lock().expect("close lock"),
        "collapse must not app-exit"
    );
}

/// Validates: menu-workspace Requirement 14.14 (CR-CH-052) -- bare `X` AT the
/// Tab_Visual_Root (empty stack) with other tabs open closes just this one
/// Workspace; it does not app-exit.
#[test]
fn bare_x_at_visual_root_closes_workspace() {
    let mut harness = harness_shell();
    // A second POM tab so the close path removes a tab rather than app-exiting.
    harness.state_mut().dispatch_command_string("START");
    for _ in 0..4 {
        harness.run();
    }
    let before = harness.state().tabs.len();
    assert!(before >= 2, "precondition: more than one tab open");
    assert!(
        harness.state().tabs.active_tab().nav_stack.is_empty(),
        "precondition: the new START tab is at its visual root (empty stack)"
    );

    harness.state_mut().dispatch_command_string("X");
    for _ in 0..4 {
        harness.run();
    }

    assert_eq!(
        harness.state().tabs.len(),
        before - 1,
        "bare X at the visual root closes this one Workspace"
    );
    assert!(
        !*harness.state().should_close.lock().expect("close lock"),
        "closing one of several tabs must not app-exit"
    );
}

/// Validates: menu-workspace Requirement 14.14; command-framework Req 10.2
/// (CR-CH-052) -- `=X` closes the Workspace (reinit to POM, then X at the empty
/// root), NOT an unconditional app-exit: with other tabs open it closes one tab;
/// as the last tab it app-exits. The multi-tab case runs full-shell; the
/// app-exit (`should_close`) assertion is made at the model level with
/// `make_shell` (mirroring `end_at_empty_stack_last_tab_exits`), since the exit
/// is a `file.exit` dispatch whose close-flag is the authoritative signal.
#[test]
fn equals_x_closes_not_app_exit() {
    // More than one tab: `=X` closes one tab, no app-exit (full shell).
    let mut harness = harness_shell();
    harness.state_mut().dispatch_command_string("START");
    for _ in 0..4 {
        harness.run();
    }
    let before = harness.state().tabs.len();
    assert!(before >= 2);

    harness.state_mut().dispatch_command_string("=X");
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.len(),
        before - 1,
        "`=X` with other tabs open closes one Workspace"
    );
    assert!(
        !*harness.state().should_close.lock().expect("close lock"),
        "`=X` is not an unconditional app-exit"
    );

    // Exactly one tab: `=X` closes it and app-exits (model-level close flag).
    let mut shell = make_shell();
    assert_eq!(shell.tabs.len(), 1, "precondition: single tab");
    shell.dispatch_command_string("=X");
    assert!(
        *shell.should_close.lock().expect("close lock"),
        "`=X` on the last tab closes the Workspace and app-exits"
    );
}

/// Validates: menu-workspace Requirement 14.4 (CR-CH-052 confirms unchanged) --
/// END pops ONE level of the Navigation_Stack (distinct from X / RETURN collapse).
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
}

/// Validates: menu-workspace Requirement 14.14; CR-CH-016 (CR-CH-052) -- bare
/// `X` at the visual root of the LAST Workspace app-exits. Asserted at the model
/// level via the `file.exit` close flag (mirroring `end_at_empty_stack_last_tab_exits`),
/// the authoritative app-exit signal.
#[test]
fn app_exits_only_on_last_close() {
    let mut shell = make_shell();
    // make_shell's single tab is the welcome editor; make it the POM root
    // deterministically (the Home Context at an empty visual root) so bare `X`
    // reaches the uniform FFCMD close verb rather than FFEDIT EXCLUDE.
    shell.set_active_tab_home();
    assert_eq!(shell.tabs.len(), 1, "precondition: single tab");
    assert!(
        shell.tabs.active_tab().is_home,
        "precondition: the single tab is the POM root"
    );
    assert!(
        shell.tabs.active_tab().nav_stack.is_empty(),
        "precondition: at the visual root (empty stack)"
    );

    shell.dispatch_command_string("X");

    assert!(
        *shell.should_close.lock().expect("close lock"),
        "bare X at the root of the last Workspace app-exits"
    );
}

/// Validates: menu-workspace Requirement 14.13 (CR-CH-052) -- the POM is NOT
/// special-cased: bare `X` after a POM->CONFIG navigation behaves identically to
/// bare `X` after a (non-POM-rooted) START SETTINGS -> CONFIG navigation; both
/// collapse to their respective Tab_Visual_Root.
#[test]
fn pom_not_special_cased() {
    // Shell A: a POM-rooted tab drilled into CONFIG. Visual root = POM.
    let mut a = harness_shell();
    a.state_mut().dispatch_command_string("CONFIG");
    for _ in 0..4 {
        a.run();
    }
    assert_eq!(a.state().tabs.active_tab().kind.tag(), KindTag::ConfigPanel);
    a.state_mut().dispatch_command_string("X");
    for _ in 0..4 {
        a.run();
    }
    assert!(
        a.state().tabs.active_tab().is_home,
        "POM-rooted tab: X collapses to the POM visual root"
    );
    let a_closed = *a.state().should_close.lock().expect("close lock");

    // Shell B: a SETTINGS-rooted tab (START SETTINGS = empty stack, non-POM)
    // drilled into CONFIG. Visual root = Settings. X collapses back to Settings.
    let mut b = harness_shell();
    b.state_mut().dispatch_command_string("START SETTINGS");
    for _ in 0..4 {
        b.run();
    }
    assert!(
        !b.state().tabs.active_tab().is_home,
        "precondition: START SETTINGS roots a non-POM Workspace"
    );
    assert!(
        b.state().tabs.active_tab().nav_stack.is_empty(),
        "precondition: START SETTINGS leaves an empty stack (Settings is the root)"
    );
    b.state_mut().dispatch_command_string("CONFIG");
    for _ in 0..4 {
        b.run();
    }
    assert_eq!(b.state().tabs.active_tab().kind.tag(), KindTag::ConfigPanel);
    b.state_mut().dispatch_command_string("X");
    for _ in 0..4 {
        b.run();
    }

    // The SAME uniform rule applies: both collapsed to their own visual root in
    // one action and neither app-exited (both had their drilled context above
    // the root). The POM is not treated differently.
    assert!(
        !b.state().tabs.active_tab().is_home,
        "SETTINGS-rooted tab: X collapses to the Settings visual root (NOT the POM)"
    );
    assert_eq!(
        b.state().tabs.active_tab().kind.tag(),
        KindTag::MenuWorkspace,
        "the Settings visual root is a MenuWorkspace"
    );
    let b_closed = *b.state().should_close.lock().expect("close lock");
    assert_eq!(
        a_closed, b_closed,
        "neither the POM-rooted nor the Settings-rooted collapse app-exits -- \
         the POM is not special-cased"
    );
    assert!(!a_closed, "collapse above the root never app-exits");
}

/// Validates: command-environments Requirement 5.1; menu-workspace Req 14.14
/// (CR-CH-052) -- on an editor Context FFEDIT claims bare `X` as EXCLUDE (no
/// close/exit), but `=X` escapes to FFCMD and closes/exits via the uniform path.
#[test]
fn ffedit_bare_x_stays_exclude_but_equals_x_escapes() {
    // Bare X on an editor Context: FFEDIT EXCLUDE, not a close/exit.
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
        "bare `X ALL` on an editor Context is FFEDIT EXCLUDE, not a close/exit"
    );
    assert_eq!(
        harness.state().tabs.len(),
        tabs_before,
        "FFEDIT EXCLUDE does not close the editor Workspace"
    );

    // `=X` escapes FFEDIT: reinit to POM then X at the empty root closes/exits.
    harness.state_mut().dispatch_command_string("=X");
    for _ in 0..4 {
        harness.run();
    }
    // shell_new_untitled added a tab, so `=X` closes that one; if it was the
    // last it app-exits. Assert the editor Workspace is gone one way or another.
    let closed_or_exited = *harness.state().should_close.lock().expect("close lock")
        || harness.state().tabs.len() < tabs_before;
    assert!(
        closed_or_exited,
        "`=X` escapes FFEDIT and closes the Workspace (exit only when last)"
    );
}

/// Validates: command-framework Requirement 10.2 (CR-CH-052) -- `=1` from a
/// non-POM tab reinitialises to the POM and resolves option 1 (Catalogs ->
/// FilesPanel), proving `=` targets the FFCMD_Root from ANY tab.
#[test]
fn equals_1_works_from_a_non_pom_tab() {
    let mut harness = harness_shell();
    harness
        .state_mut()
        .dispatch_command_string("START SETTINGS");
    for _ in 0..4 {
        harness.run();
    }
    assert!(
        !harness.state().tabs.active_tab().is_home,
        "precondition: on a non-POM (Settings) tab"
    );

    harness.state_mut().dispatch_command_string("=1");
    for _ in 0..4 {
        harness.run();
    }

    // POM option 1 is `Catalogs` -> the Files (Catalog Explorer) panel.
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::FilesPanel,
        "`=1` from a non-POM tab resolves as POM option 1 (Catalogs)"
    );
}

/// Validates: menu-and-statusbar Requirement 18.9/18.11; menu-workspace Req
/// 14.14 (CR-CH-052) -- a Detached_Workspace closes via the SAME
/// close-workspace-or-exit path. Driven at the model level: the detached tab is
/// closed via bare `X` at its visual root and the tab count drops (the OS-window
/// teardown aspect is covered manually -- see TCR).
#[test]
fn detached_workspace_closes_via_same_path() {
    let mut shell = make_shell();
    // Create a second tab and mark it floating (detached) at its visual root.
    shell.dispatch_command_string("START");
    let detached_idx = shell.tabs.active_index();
    let detached_id = shell.tabs.tabs()[detached_idx].id;
    if let Some(t) = shell.tabs.tabs_mut().get_mut(detached_idx) {
        t.is_floating = true;
        t.nav_stack.clear();
    }
    let before = shell.tabs.len();
    assert!(before >= 2, "precondition: a detached tab plus the primary");

    // Bare X at the detached tab's visual root closes it via the shared
    // close_workspace_or_exit path (exit only when last).
    shell.dispatch_command_string("X");

    assert!(
        shell.tabs.index_of_id(detached_id).is_none() || shell.tabs.len() == before - 1,
        "the detached Workspace closes via the same close-workspace-or-exit path"
    );
    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "closing a detached tab while the primary remains must not app-exit"
    );
}
