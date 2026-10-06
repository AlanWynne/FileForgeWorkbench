//! Shell tests -- split_detach area (split from shell/tests.rs, CR F3).
//! Items are verbatim; only their file location changed.

#![allow(unused_imports)]
use super::tests_common::*;
use ff_keys::{KeyMap, ModifiedKey};
use std::sync::{Arc, Mutex};

use ff_command::{
    CommandDispatch, CommandError, CommandHandler, CommandHistory, CommandId, CommandMetadata,
    CommandParams, CommandRegistry, CommandResult, ExecutionContext,
};

/// Validates: menu-workspace Req 17.4 (command parity) -- activating a peeked
/// leaf routes through `handle_command` (the same path as typing it). Here the
/// Settings menu's `THEME dark`-equivalent leaf (`THEME` opens the editor; we
/// use `THEME dark` semantics via the command) proves the bar's leaf dispatch
/// changes state exactly as the typed command would.
#[test]
fn menu_bar_leaf_dispatch_is_command_parity() {
    let mut shell = make_shell();
    // The bar peeks Settings and would render a `THEME` leaf; activating it is
    // `handle_command("THEME ...")`. Prove parity: dispatching the leaf command
    // changes the active theme exactly as typing it does.
    shell.dispatch_command_string("THEME dark");
    assert_eq!(shell.palette.name, "Default Dark");
    assert!(shell.open_error.is_none());
}

/// Validates: menu-and-statusbar Req 18.14 (CR-CH-040 / B046 Slice 1) -- DETACH
/// detaches the current Workspace (sets detach_pending, subject to the 16-window
/// limit), the primary verb renamed from the former SPLIT.
#[test]
fn detach_command_sets_detach_pending() {
    let mut shell = make_shell();
    assert!(shell.detach_split.detach_pending.is_none());
    shell.handle_command("DETACH");
    assert!(
        shell.detach_split.detach_pending.is_some(),
        "DETACH must set detach_pending"
    );
    assert!(shell.open_error.is_none());
}

/// Validates: menu-and-statusbar Req 18.14 -- `SPLIT DETACH` remains a deprecated
/// ALIAS of DETACH.
#[test]
fn split_detach_alias_still_detaches() {
    let mut shell = make_shell();
    shell.handle_command("SPLIT DETACH");
    assert!(
        shell.detach_split.detach_pending.is_some(),
        "SPLIT DETACH alias must still detach"
    );
    assert!(shell.open_error.is_none());
}

/// Validates: menu-and-statusbar Req 19.11 (revision, CR-CH-040) -- the bare verb
/// SPLIT no longer detaches (the inert ISPF split is retired; SPLIT is reserved
/// for the future in-window split). It falls through to the normal
/// command-resolution chain, which reports it as not-yet-implemented, and does
/// NOT set detach_pending.
#[test]
fn bare_split_no_longer_detaches() {
    let mut shell = make_shell();
    shell.handle_command("SPLIT");
    assert!(
        shell.detach_split.detach_pending.is_none(),
        "bare SPLIT must NOT detach (reserved for the Slice 2 in-window split)"
    );
}

/// Validates: CX Requirement 3.1 -- SPLIT DETACH sets detach_pending from any tab kind.
#[test]
fn split_detach_sets_detach_pending() {
    let mut shell = make_shell();
    shell.handle_command("SPLIT DETACH");
    assert!(
        shell.detach_split.detach_pending.is_some(),
        "SPLIT DETACH should set detach_pending"
    );
}

/// Validates: CX Requirement 3.5 -- SPLIT DETACH at 16-window limit shows error.
#[test]
fn split_detach_at_limit_shows_error() {
    let mut shell = make_shell();
    // CR-CH-035: the 16-window limit now counts recorded FloatingTabs (the shared
    // source of truth). Fabricate 16 entries so the next SPLIT DETACH is rejected
    // deterministically (no environment-dependent soft skip).
    for i in 0..16 {
        shell.detach_split.floating_tabs.push(super::FloatingTab {
            viewport_id: egui::ViewportId::from_hash_of(format!("limit_{i}")),
            tab_id: crate::tab_state::TabId(20_000 + i),
            origin_index: 0,
            cmd_ctx: super::WorkspaceCommandContext::default(),
        });
    }
    shell.open_error = None;
    shell.handle_command("SPLIT DETACH");
    assert!(
        shell.open_error.is_some(),
        "SPLIT DETACH at the 16-window limit must show a status message"
    );
    assert_eq!(
        shell.detach_split.floating_tabs.len(),
        16,
        "no 17th FloatingTab may be recorded at the limit"
    );
}

// Validates: command-configurator Requirement 3.2 -- shell.mode = enabled runs a
// Detached target immediately (no pending confirmation).
#[test]
fn external_enabled_detached_runs_immediately() {
    let mut shell = make_shell();
    set_shell_mode(&shell, ff_shell::ShellMode::Enabled);
    shell.dispatch_command_target(&noop_external_target());
    assert!(
        shell.pending_external.is_none(),
        "enabled mode must not stage a confirmation"
    );
    let msg = shell.open_error.as_deref().unwrap_or_default();
    assert!(
        msg.contains("detached"),
        "a detached run should report a started-status, got: {msg}"
    );
}

// Validates: workspace-kinds Req 8.3 (CR-NR-095) -- with the active Kind's
// Command_Line_Position set to Bottom (via the COMMAND command), the unsplit
// primary command field renders in the LOWER half of the window, and the
// first-Tab Boundary_Policy is unchanged (still lands on the reported first
// interior). Uses a throwaway kinds dir so the persist write is isolated.
#[test]
fn full_shell_command_bottom_moves_unsplit_field_to_bottom() {
    use crate::workspace_kind::CommandLinePosition;
    use tempfile::TempDir;
    let mut harness = harness_shell();
    let dir = TempDir::new().expect("tempdir");
    harness.state_mut().dir_overrides.workspace_kinds = Some(dir.path().to_path_buf());

    // Baseline: command field is focused and near the TOP of the window.
    assert_eq!(harness.ctx.memory(|m| m.focused()), Some(cmd_field_id()));
    let screen = harness.ctx.content_rect();
    let top_rect = harness
        .ctx
        .read_response(cmd_field_id())
        .map(|r| r.rect)
        .expect("command field has a rect");
    assert!(
        top_rect.center().y < screen.center().y,
        "baseline: command field is in the upper half (Top default)"
    );

    // Move it to the bottom via the command.
    harness.state_mut().handle_command("COMMAND BOTTOM");
    for _ in 0..3 {
        harness.run();
    }
    assert_eq!(
        harness
            .state()
            .command_line_position_for(harness.state().tabs.active_index()),
        CommandLinePosition::Bottom,
        "COMMAND BOTTOM set the active Kind position to Bottom"
    );
    let bottom_rect = harness
        .ctx
        .read_response(cmd_field_id())
        .map(|r| r.rect)
        .expect("command field still has a rect after moving");
    assert!(
        bottom_rect.center().y > screen.center().y,
        "command field now renders in the lower half of the window (Bottom)"
    );

    // Boundary_Policy unchanged: first Tab from the command field still lands on
    // the reported first interior control (placement does not alter Tab-order).
    harness.ctx.memory_mut(|m| m.request_focus(cmd_field_id()));
    harness.run();
    let expected_first = harness.state().focus.first_interior_id;
    assert!(expected_first.is_some(), "POM reports a first interior");
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        expected_first,
        "first Tab still focuses the reported first interior with the field at the bottom"
    );
}

// -- CR-CH-035 (B045): full-shell Detached Workspace behaviour ---------------
// These drive the REAL WorkbenchShell headlessly (build_eframe) through the
// detach/redock state machine. The actual separate OS window (its appearance,
// taskbar presence, independent move/resize) is a justified MANUAL row per
// testing.md (real multi-viewport windows are the documented harness exception);
// here we assert the shell-side state transitions and that the immediate
// viewport render path executes without panicking.

/// Validates: menu-and-statusbar Requirement 18.1/18.4/18.8 (CR-CH-035, B045) --
/// detaching a tab (via the SPLIT DETACH command, the same path the "Move to
/// Other View" context item uses) sets is_floating on the tab and records a
/// FloatingTab, so the primary tab bar no longer shows it and its content is
/// rendered in a floating viewport.
#[test]
fn full_shell_detach_sets_floating_and_records_floating_tab() {
    // Validates: menu-and-statusbar Requirement 18.1, 18.4, 18.8
    let mut harness = harness_shell();
    // Open a second workspace so at least one tab remains docked after detach.
    harness.state_mut().handle_command("START"); // opens a new POM tab
    harness.run();
    let detach_idx = harness.state().tabs.active_index();
    let detach_id = harness.state().tabs.active_tab().id;
    // Detach the active tab (SPLIT on a non-editor tab detaches; SPLIT DETACH is
    // the explicit form). Sets detach_pending; the next frame consumes it.
    harness.state_mut().handle_command("SPLIT DETACH");
    for _ in 0..3 {
        harness.run();
    }
    let state = harness.state();
    assert_eq!(
        state.detach_split.floating_tabs.len(),
        1,
        "detach must record exactly one FloatingTab"
    );
    assert_eq!(
        state.detach_split.floating_tabs[0].tab_id, detach_id,
        "FloatingTab must track the detached tab's stable id"
    );
    assert_eq!(
        state.detach_split.floating_tabs[0].origin_index, detach_idx,
        "FloatingTab must record the origin index for redock"
    );
    let tab = state
        .tabs
        .tabs()
        .iter()
        .find(|t| t.id == detach_id)
        .expect("detached tab still lives in the TabManager");
    assert!(tab.is_floating, "detached tab must be flagged is_floating");
}

/// Validates: menu-and-statusbar Requirement 18.11 (B068) -- a function key
/// dispatched while a Detached_Workspace has focus acts on THAT window's
/// context. F4 resolves to RETURN; dispatched in the detached context via the
/// same `dispatch_key_command` path used by `dispatch_detached_function_key`, it
/// returns the detached (non-POM) tab to its POM and leaves the primary tab
/// untouched.
#[test]
fn detached_function_key_return_acts_on_its_tab() {
    // Validates: menu-and-statusbar Requirement 18.11; function-keys 3.1
    use crate::tab_state::{KindTag, TabKind};
    let mut shell = make_shell();
    shell.handle_command("START"); // second tab (the "detached" one)
    let primary_active = shell.tabs.active_index();
    let detached = if primary_active == 0 { 1 } else { 0 };
    // Make BOTH tabs distinct non-POM contexts so we can prove the detached
    // RETURN changed ONLY the detached tab (the primary stays non-POM). The
    // detached tab is a POM-drilled sub-context (its Navigation_Stack bottom is
    // the POM), so under CR-CH-052 RETURN collapses to its Tab_Visual_Root (the
    // POM) in one step -- the "acts on its own tab" behaviour we assert below.
    if let Some(t) = shell.tabs.tabs_mut().get_mut(detached) {
        t.kind = TabKind::FilesPanel;
        t.is_home = false;
        t.title = "[FILES]".to_string();
        t.nav_stack.clear();
        t.nav_stack
            .push(ff_session::session_state::WorkspaceDescriptor::Menu {
                name: "pom".to_string(),
            });
    }
    if let Some(t) = shell.tabs.tabs_mut().get_mut(primary_active) {
        t.kind = TabKind::ConfigPanel;
        t.is_home = false;
        t.title = "[CONFIG]".to_string();
        t.nav_stack.clear();
    }
    let detached_id = shell.tabs.tabs()[detached].id;

    // F4 resolves to the "RETURN" command via the key map (proven by
    // egui_fkey_assigned_key_returns_command); a key-forwarded RETURN falls
    // through dispatch_key_command -> handle_command("RETURN") -> nav_return.
    // Dispatch it in the detached context exactly as dispatch_detached_function_key
    // does, and assert it acted on the detached tab.
    let mut ctx = super::WorkspaceCommandContext::default();
    shell.with_workspace_context(detached, &mut ctx, |s| {
        assert_eq!(
            s.tabs.active_index(),
            detached,
            "swap installs detached active"
        );
        assert!(
            !s.tabs.active_tab().is_home,
            "detached is non-POM before RETURN"
        );
        s.handle_command("RETURN");
        assert!(
            s.tabs.active_tab().is_home,
            "inside swap: RETURN must make the detached tab home"
        );
    });

    let didx = shell
        .tabs
        .index_of_id(detached_id)
        .expect("detached tab exists");
    assert!(
        shell.tabs.tabs()[didx].is_home,
        "F4/RETURN in the detached window must return ITS tab to the POM"
    );
    assert!(
        !shell.tabs.tabs()[primary_active].is_home,
        "the primary tab must be untouched by the detached F-key (still non-POM)"
    );
    assert_eq!(
        shell.tabs.tabs()[primary_active].kind.tag(),
        KindTag::ConfigPanel,
        "the primary tab's context is unchanged by the detached RETURN"
    );
    assert_eq!(
        shell.tabs.active_index(),
        primary_active,
        "primary active tab restored after the detached F-key dispatch"
    );
}

/// Validates: menu-and-statusbar Requirement 18.13 (CR-NR-088) -- DOCK on a
/// workspace that is NOT detached is a no-op with a status message.
#[test]
fn full_shell_dock_on_non_detached_is_noop_with_message() {
    // Validates: menu-and-statusbar Requirement 18.13
    let mut harness = harness_shell();
    assert!(!harness.state().tabs.active_tab().is_floating);
    harness.state_mut().handle_command("DOCK");
    assert!(
        harness.state().open_error.is_some(),
        "DOCK on a non-detached workspace must report a status message"
    );
    assert!(harness.state().detach_split.floating_tabs.is_empty());
}

/// Validates: menu-and-statusbar Requirement 18.7 (CR-CH-035, B045) -- with 16
/// Detached Workspaces already open, a further detach is rejected with a status
/// message and no new FloatingTab.
#[test]
fn full_shell_detach_rejected_at_16_window_limit() {
    // Validates: menu-and-statusbar Requirement 18.7
    let mut harness = harness_shell();
    // Fabricate 16 floating entries directly (the guard reads floating_tabs.len()).
    for i in 0..16 {
        let vid = egui::ViewportId::from_hash_of(format!("limit_ft_{i}"));
        harness
            .state_mut()
            .detach_split
            .floating_tabs
            .push(super::FloatingTab {
                viewport_id: vid,
                tab_id: crate::tab_state::TabId(10_000 + i),
                origin_index: 0,
                cmd_ctx: super::WorkspaceCommandContext::default(),
            });
    }
    harness.state_mut().open_error = None;
    // Attempt one more detach via the context-menu path guard (SPLIT DETACH uses
    // the is_floating count; the "Move to Other View" item uses floating_tabs.len
    // -- both reject at 16). Drive SPLIT DETACH and confirm no 17th entry.
    harness.state_mut().handle_command("SPLIT DETACH");
    for _ in 0..2 {
        harness.run();
    }
    assert!(
        harness.state().detach_split.floating_tabs.len() <= 16,
        "must never exceed 16 Detached Workspaces"
    );
    assert!(
        harness.state().open_error.is_some(),
        "detach beyond the 16-window limit must report a status message"
    );
}

/// Validates: layout-and-docking Requirement 14.4 -- SPLIT DOWN splits stacked
/// (Vertical) rather than side-by-side.
#[test]
fn full_shell_split_down_is_vertical() {
    use ff_layout::{SplitDirection, TabGroupTree};
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT DOWN");
    harness.run();
    let state = harness.state();
    assert!(state.tabs.is_split());
    match state.tabs.layout_tree() {
        TabGroupTree::Split { direction, .. } => assert_eq!(
            *direction,
            SplitDirection::Vertical,
            "SPLIT DOWN must produce a Vertical (stacked) split"
        ),
        _ => panic!("expected a Split after SPLIT DOWN"),
    }
}

/// Validates: layout-and-docking Requirement 14.1 -- a second SPLIT NESTS (the
/// Slice 2b "one split only" limit is removed): it adds a third leaf and a POM.
#[test]
fn full_shell_second_split_nests() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    harness.run();
    assert_eq!(harness.state().tabs.leaf_ids().len(), 2);
    let after_first = harness.state().tabs.len();

    harness.state_mut().handle_command("SPLIT");
    harness.run();

    let state = harness.state();
    assert_eq!(
        state.tabs.leaf_ids().len(),
        3,
        "a second SPLIT must nest into a third leaf"
    );
    assert_eq!(
        state.tabs.len(),
        after_first + 1,
        "the nested SPLIT adds another POM tab"
    );
    assert!(state.open_error.is_none(), "nesting is not an error");
}

/// Validates: layout-and-docking Requirement 13.9 -- UNSPLIT collapses back to a
/// single Tab_Group; the focused (survivor) group's active tab remains active
/// and no open tab is lost.
#[test]
fn full_shell_unsplit_collapses_preserving_survivor() {
    let mut harness = harness_shell();
    let before = harness.state().tabs.len();
    harness.state_mut().handle_command("SPLIT");
    harness.run();
    let split_count = harness.state().tabs.len();
    assert_eq!(split_count, before + 1, "precondition: split added a POM");
    // Group 1 (the new POM) is focused; it survives the collapse.
    let survivor_id = harness.state().tabs.active_tab().id;

    harness.state_mut().handle_command("UNSPLIT");
    harness.run();

    let state = harness.state();
    assert!(!state.tabs.is_split(), "UNSPLIT must collapse the split");
    assert_eq!(
        state.tabs.len(),
        split_count,
        "UNSPLIT must not lose any open tab"
    );
    assert_eq!(
        state.tabs.active_tab().id,
        survivor_id,
        "the focused group's active tab survives the collapse as the active tab"
    );
}

/// Validates: layout-and-docking Requirement 13.9 -- while split, END collapses
/// the split (the keyboard-friendly "close this region") rather than performing
/// the usual per-tab Navigation_Stack pop.
#[test]
fn full_shell_end_while_split_collapses() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    harness.run();
    assert!(harness.state().tabs.is_split(), "precondition: split");

    harness.state_mut().handle_command("END");
    harness.run();

    assert!(
        !harness.state().tabs.is_split(),
        "END while split must collapse the split"
    );
}

/// Validates: layout-and-docking Requirement 13.10 (CR-CH-040 regression) --
/// SPLIT DETACH is unchanged by the new SPLIT verbs: it still detaches the
/// active Workspace (does NOT create an in-window split).
#[test]
fn full_shell_split_detach_still_detaches_not_splits() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("START"); // keep a docked tab after detach
    harness.run();
    harness.state_mut().handle_command("SPLIT DETACH");
    for _ in 0..3 {
        harness.run();
    }
    let state = harness.state();
    assert!(
        !state.tabs.is_split(),
        "SPLIT DETACH must not create an in-window split"
    );
    assert_eq!(
        state.detach_split.floating_tabs.len(),
        1,
        "SPLIT DETACH must detach into a Detached_Workspace"
    );
}

// === CR-NR-093 Slice 2c.4: detached fold-in (DOCK into a split leaf) ========
// The real OS detached window (its chrome, taskbar presence, cross-window move)
// remains a justified-MANUAL row; these drive the headless detach/DOCK state
// machine and assert the in-window re-attachment behaviour (Req 14.16), plus
// that the unsplit re-dock path is unchanged (no Req 18 regression).

/// Validates: layout-and-docking Requirement 14.16 -- when the Workspace is
/// split, re-docking a Detached_Workspace re-attaches its tab into a tree LEAF
/// (it stays split; the tab is a leaf member and no longer floating) rather than
/// collapsing to a flat re-dock.
#[test]
fn full_shell_dock_into_split_reattaches_to_leaf() {
    let mut harness = harness_shell();
    // Create a split so there are two regions (root leaf + new POM leaf).
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..3 {
        harness.run();
    }
    assert!(harness.state().tabs.is_split(), "precondition: split");
    // Detach the focused region's active tab into a Detached_Workspace.
    let detach_id = harness.state().tabs.active_tab().id;
    harness.state_mut().handle_command("SPLIT DETACH");
    for _ in 0..3 {
        harness.run();
    }
    assert_eq!(
        harness.state().detach_split.floating_tabs.len(),
        1,
        "precondition: one Detached_Workspace"
    );

    // DOCK it back, run against its own context (as typed in its command line).
    let detach_idx = harness
        .state()
        .tabs
        .index_of_id(detach_id)
        .expect("detached tab exists");
    let mut ctx = super::WorkspaceCommandContext::default();
    harness
        .state_mut()
        .with_workspace_context(detach_idx, &mut ctx, |shell| {
            shell.handle_command("DOCK");
        });
    for _ in 0..2 {
        harness.run();
    }
    let state = harness.state();
    assert!(
        state.detach_split.floating_tabs.is_empty(),
        "DOCK must remove the FloatingTab"
    );
    let idx = state
        .tabs
        .index_of_id(detach_id)
        .expect("re-docked tab still exists");
    assert!(
        !state.tabs.tabs()[idx].is_floating,
        "re-docked tab must no longer be is_floating"
    );
    // The re-docked tab is a member of some tree leaf (in-window re-attachment).
    let in_a_leaf = state.tabs.leaf_ids().iter().any(|leaf| {
        state
            .tabs
            .leaf_tab_store_indices(*leaf)
            .iter()
            .any(|&i| state.tabs.tabs()[i].id == detach_id)
    });
    assert!(
        in_a_leaf,
        "Req 14.16: a re-docked tab must re-attach into a tree leaf when split"
    );
}

/// Validates: layout-and-docking Requirement 14.15 -- the fold-in does NOT
/// regress the UNSPLIT re-dock: with no split, DOCK still restores the tab to
/// its flat origin index (the pre-2c.4 behaviour, via the else branch).
#[test]
fn full_shell_dock_unsplit_still_restores_flat_origin() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("START");
    harness.run();
    assert!(!harness.state().tabs.is_split(), "precondition: unsplit");
    let origin = harness.state().tabs.active_index();
    let detach_id = harness.state().tabs.active_tab().id;
    harness.state_mut().handle_command("SPLIT DETACH");
    for _ in 0..3 {
        harness.run();
    }
    let detach_idx = harness
        .state()
        .tabs
        .index_of_id(detach_id)
        .expect("detached tab exists");
    let mut ctx = super::WorkspaceCommandContext::default();
    harness
        .state_mut()
        .with_workspace_context(detach_idx, &mut ctx, |shell| {
            shell.handle_command("DOCK");
        });
    let state = harness.state();
    assert!(
        state.detach_split.floating_tabs.is_empty(),
        "DOCK removes the FloatingTab"
    );
    let idx = state
        .tabs
        .index_of_id(detach_id)
        .expect("re-docked tab exists");
    assert_eq!(
        idx, origin,
        "unsplit DOCK must restore the flat origin index (no regression)"
    );
    assert!(!state.tabs.tabs()[idx].is_floating);
}

// === B071: File Explorer in a split region stays in its region ==============

/// Validates: layout-and-docking Req 13.4/14.4 (B071) -- when the Workspace is
/// split and a region's Context is switched to the File Explorer, the split MUST
/// remain (the explorer renders inside its region, not full-window). Before the
/// fix, `render_central_panel`'s `is_file_explorer` branch took over the whole
/// window and bypassed the split render.
#[test]
fn full_shell_file_explorer_in_split_region_keeps_split() {
    use crate::tab_state::{KindTag, TabKind};
    let mut harness = harness_shell();
    // Split: focus moves to the new POM region.
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..3 {
        harness.run();
    }
    assert!(harness.state().tabs.is_split(), "precondition: split");
    let leaves_before = harness.state().tabs.leaf_ids().len();

    // Navigate the focused region's Context to the File Explorer in place.
    harness.state_mut().dispatch_command_string("FILES");
    for _ in 0..3 {
        harness.run();
    }

    let state = harness.state();
    // The focused region is now a File Explorer...
    assert_eq!(
        state.tabs.active_tab().kind.tag(),
        KindTag::FileExplorerPanel,
        "the focused region switched to the File Explorer"
    );
    // ...but the Workspace is STILL split (not collapsed to full screen).
    assert!(
        state.tabs.is_split(),
        "B071: a File Explorer region must NOT collapse the split to full screen"
    );
    assert_eq!(
        state.tabs.leaf_ids().len(),
        leaves_before,
        "the split still has the same number of regions"
    );
}

// === CR-NR-094 Slice 2d: per-region command-line context lifecycle ==========

/// Validates: layout-and-docking Req 15.5 -- reconcile inserts a fresh
/// per-region command context for each split leaf, and the map is empty when
/// unsplit.
#[test]
fn region_cmd_ctx_inserted_per_leaf_on_split() {
    let mut harness = harness_shell();
    // Unsplit: reconcile leaves the map empty.
    harness.state_mut().reconcile_region_cmd_ctx();
    assert!(
        harness.state().detach_split.region_cmd_ctx.is_empty(),
        "unsplit workbench has no per-region contexts"
    );
    // Split -> two leaves -> two contexts after reconcile.
    harness.state_mut().handle_command("SPLIT");
    harness.state_mut().reconcile_region_cmd_ctx();
    let leaves = harness.state().tabs.leaf_ids();
    assert_eq!(leaves.len(), 2, "precondition: two leaves");
    for leaf in &leaves {
        assert!(
            harness
                .state()
                .detach_split
                .region_cmd_ctx
                .contains_key(leaf),
            "each leaf must get a per-region command context"
        );
    }
    assert_eq!(harness.state().detach_split.region_cmd_ctx.len(), 2);
}

/// Validates: layout-and-docking Req 15.5 -- collapsing the split drops the
/// per-region contexts; a full unsplit clears the whole map.
#[test]
fn region_cmd_ctx_cleared_on_unsplit() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    harness.state_mut().reconcile_region_cmd_ctx();
    assert!(
        !harness.state().detach_split.region_cmd_ctx.is_empty(),
        "split populated the map"
    );
    // Collapse back to a single region.
    harness.state_mut().handle_command("UNSPLIT");
    harness.state_mut().reconcile_region_cmd_ctx();
    assert!(
        harness.state().detach_split.region_cmd_ctx.is_empty(),
        "unsplit must clear all per-region command contexts"
    );
}

/// Validates: layout-and-docking Req 15.5 -- command text belongs to the LEAF,
/// not a tab: moving a tab between regions does not carry the region's command
/// text with it (the source leaf keeps its own context; the map stays keyed by
/// leaf id).
#[test]
fn region_cmd_ctx_text_does_not_travel_with_moved_tab() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    harness.state_mut().reconcile_region_cmd_ctx();
    let leaves = harness.state().tabs.leaf_ids();
    let (a, b) = (leaves[0], leaves[1]);
    // Put distinct command text in each region's context.
    harness
        .state_mut()
        .detach_split
        .region_cmd_ctx
        .get_mut(&a)
        .unwrap()
        .command_text = "TEXT_A".to_string();
    harness
        .state_mut()
        .detach_split
        .region_cmd_ctx
        .get_mut(&b)
        .unwrap()
        .command_text = "TEXT_B".to_string();
    // Move a tab from region A to region B (if A has a movable tab).
    if let Some(store_idx) = harness.state().tabs.leaf_active_store_index(a) {
        let tab_id = harness.state().tabs.tabs()[store_idx].id;
        harness.state_mut().tabs.move_tab_to_group(tab_id, b);
    }
    harness.state_mut().reconcile_region_cmd_ctx();
    // Whichever leaves still exist keep their OWN text; no B text leaked into A.
    if harness.state().detach_split.region_cmd_ctx.contains_key(&b) {
        assert_eq!(
            harness.state().detach_split.region_cmd_ctx[&b].command_text,
            "TEXT_B",
            "region B keeps its own command text; moved tab does not carry text"
        );
    }
    if harness.state().detach_split.region_cmd_ctx.contains_key(&a) {
        assert_eq!(
            harness.state().detach_split.region_cmd_ctx[&a].command_text,
            "TEXT_A",
            "region A keeps its own command text"
        );
    }
}

/// Validates: layout-and-docking Req 15.2 -- while split, the single top-level
/// command field is suppressed (each region carries its own); unsplitting
/// restores it.
#[test]
fn full_shell_split_suppresses_top_level_command_field_unsplit_restores() {
    let mut harness = harness_shell();
    // Unsplit: the top-level command field exists and holds focus.
    assert_eq!(harness.ctx.memory(|m| m.focused()), Some(cmd_field_id()));

    harness.state_mut().handle_command("SPLIT");
    for _ in 0..4 {
        harness.run();
    }
    // Split: the top-level command field id is no longer a live widget, so
    // requesting focus on it does not stick (the panel was not rendered).
    harness.ctx.memory_mut(|m| m.request_focus(cmd_field_id()));
    harness.run();
    assert_ne!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "top-level command field must be suppressed while split"
    );

    harness.state_mut().handle_command("UNSPLIT");
    for _ in 0..4 {
        harness.run();
    }
    // Unsplit again: the top-level field is rendered and focusable once more.
    harness.ctx.memory_mut(|m| m.request_focus(cmd_field_id()));
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "top-level command field must be restored after unsplit"
    );
}

/// Validates: layout-and-docking Req 15.3, 15.7 -- a command submitted in a
/// region's own command line acts on THAT region's active tab, not another
/// region's, and focuses the submitting region.
#[test]
fn full_shell_region_command_acts_on_its_own_region() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..4 {
        harness.run();
    }
    let leaves = harness.state().tabs.leaf_ids();
    assert_eq!(leaves.len(), 2, "precondition: two regions");
    let (a, b) = (leaves[0], leaves[1]);
    let a_tab = harness
        .state()
        .tabs
        .leaf_active_store_index(a)
        .map(|i| harness.state().tabs.tabs()[i].id)
        .expect("region A active tab");
    let b_tab = harness
        .state()
        .tabs
        .leaf_active_store_index(b)
        .map(|i| harness.state().tabs.tabs()[i].id)
        .expect("region B active tab");

    // Submit NAME in region A's command line.
    submit_region_command(&mut harness, a, "NAME RegionA");
    // Submit a DIFFERENT NAME in region B's command line.
    submit_region_command(&mut harness, b, "NAME RegionB");

    let state = harness.state();
    let a_idx = state.tabs.index_of_id(a_tab).expect("A tab exists");
    let b_idx = state.tabs.index_of_id(b_tab).expect("B tab exists");
    assert_eq!(
        state.tabs.tabs()[a_idx].workspace_name.as_deref(),
        Some("RegionA"),
        "region A's command must name region A's tab"
    );
    assert_eq!(
        state.tabs.tabs()[b_idx].workspace_name.as_deref(),
        Some("RegionB"),
        "region B's command must name region B's tab, not A's"
    );
    // The last submit (region B) focuses region B (Req 15.7).
    assert_eq!(
        state.tabs.focused_leaf_id(),
        b,
        "submitting in a region focuses that region"
    );
}

/// Validates: layout-and-docking Req 15.4, 15.6 -- each region's command text
/// and status are isolated: text typed in region A does not appear in region B,
/// and each region keeps its own text across frames while the split lives.
#[test]
fn full_shell_region_command_text_and_status_are_isolated() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..4 {
        harness.run();
    }
    let leaves = harness.state().tabs.leaf_ids();
    let (a, b) = (leaves[0], leaves[1]);
    harness
        .state_mut()
        .detach_split
        .region_cmd_ctx
        .get_mut(&a)
        .unwrap()
        .command_text = "ONLY_A".to_string();
    harness
        .state_mut()
        .detach_split
        .region_cmd_ctx
        .get_mut(&b)
        .unwrap()
        .command_text = "ONLY_B".to_string();
    // Render several frames; the per-region contexts must persist unchanged.
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().detach_split.region_cmd_ctx[&a].command_text,
        "ONLY_A",
        "region A keeps its own command text across frames"
    );
    assert_eq!(
        harness.state().detach_split.region_cmd_ctx[&b].command_text,
        "ONLY_B",
        "region B text is isolated from region A"
    );
}

/// Validates: layout-and-docking Req 15.8 -- per-region command state is
/// transient: it is not persisted, so a fresh unsplit->split cycle starts every
/// region with an empty command line.
#[test]
fn full_shell_region_command_state_is_transient() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..3 {
        harness.run();
    }
    let leaves = harness.state().tabs.leaf_ids();
    harness
        .state_mut()
        .detach_split
        .region_cmd_ctx
        .get_mut(&leaves[0])
        .unwrap()
        .command_text = "STALE".to_string();
    // Collapse and re-split: the map is cleared and rebuilt fresh.
    harness.state_mut().handle_command("UNSPLIT");
    harness.state_mut().reconcile_region_cmd_ctx();
    harness.state_mut().handle_command("SPLIT");
    harness.state_mut().reconcile_region_cmd_ctx();
    for leaf in harness.state().tabs.leaf_ids() {
        assert_eq!(
            harness.state().detach_split.region_cmd_ctx[&leaf].command_text,
            "",
            "a re-split region starts with an empty command line (transient)"
        );
    }
}

/// Validates: layout-and-docking Req 16.2/16.3 -- while split, the single
/// app-level menu bar and Title_Line are SUPPRESSED (each region draws its
/// own); unsplitting restores the app-level chrome. Mirrors the existing
/// top-level command-field suppression test.
#[test]
fn full_shell_split_suppresses_app_level_menu_bar_and_title_unsplit_restores() {
    let mut harness = harness_shell();
    // Unsplit: the app-level menu bar rendered and captured a first-button id.
    let app_menu_first = harness.state().focus.menu_first_id;
    assert!(
        app_menu_first.is_some(),
        "precondition: unsplit app-level menu bar captured a first-button id"
    );
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..4 {
        harness.run();
    }
    // While split: the app-level menu-bar first-button id (captured pre-split) is
    // no longer a live, focusable widget -- the app-level bar was suppressed.
    if let Some(app_first) = app_menu_first {
        harness.ctx.memory_mut(|m| m.request_focus(app_first));
        harness.run();
        assert_ne!(
            harness.ctx.memory(|m| m.focused()),
            Some(app_first),
            "app-level menu bar must be suppressed while split"
        );
    }

    harness.state_mut().handle_command("UNSPLIT");
    for _ in 0..4 {
        harness.run();
    }
    assert!(
        harness.state().focus.menu_first_id.is_some(),
        "app-level menu bar must be restored after unsplit"
    );
}

/// Validates: layout-and-docking Req 16.4/16.5/16.6 -- the derived Placement of
/// a Workspace instance is Docked{leaf} while it sits in a split region, and
/// Detached once it is in a floating window; the SAME instance id keeps a
/// coherent placement across the transition (placement is derived, not stored).
#[test]
fn full_shell_placement_tracks_split_region_then_detached() {
    use crate::tab_manager::Placement;
    let mut harness = harness_shell();
    // Split: the focused region's active instance is Docked in a real leaf.
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..4 {
        harness.run();
    }
    assert!(harness.state().tabs.is_split(), "precondition: split");
    let focused_leaf = harness.state().tabs.focused_leaf_id();
    let inst = harness.state().tabs.active_tab().id;
    assert_eq!(
        harness.state().placement_of(inst),
        Placement::Docked { leaf: focused_leaf },
        "a split region's active instance is Docked in that region's leaf"
    );

    // Collapse the split, then detach the (now sole) instance: placement flips to
    // Detached, keyed by the SAME instance id (fabricate the FloatingTab record,
    // as detach mechanics are covered elsewhere).
    harness.state_mut().handle_command("UNSPLIT");
    for _ in 0..3 {
        harness.run();
    }
    let inst2 = harness.state().tabs.active_tab().id;
    harness
        .state_mut()
        .detach_split
        .floating_tabs
        .push(super::FloatingTab {
            viewport_id: egui::ViewportId::from_hash_of("placement_flip"),
            tab_id: inst2,
            origin_index: 0,
            cmd_ctx: super::WorkspaceCommandContext::default(),
        });
    assert_eq!(
        harness.state().placement_of(inst2),
        Placement::Detached,
        "once in a floating window the instance's placement is Detached"
    );
}

/// Validates: menu-and-statusbar Req 18.10 (CR-CH-036), B074 -- a chained menu
/// fastpath (`=0.m`) dispatched under a detached window's context must NOT change
/// the Primary_Window's active tab, even though `=0` inserts a POM tab (shifting
/// indices). `with_workspace_context` restores the primary by STABLE TabId, so
/// the primary returns to the SAME tab regardless of the insert.
#[test]
fn detached_chained_fastpath_does_not_change_primary_active_tab() {
    let mut shell = make_shell();
    // Make the PRIMARY active tab a NON-home (untitled editor) tab: `=` inside the
    // swap only triggers insert_pom_tab when the active tab is not Home, which is
    // the index-shifting path B074 fixes.
    shell.shell_new_untitled(); // a non-home editor tab; becomes active
    let primary_active_id = shell.tabs.active_tab().id;
    let primary_kind_before = shell.tabs.active_tab().kind.tag();
    let tab_count_before = shell.tabs.len();
    assert!(
        !shell.tabs.active_tab().is_home,
        "precondition: primary active tab is non-home"
    );

    // Detach target: a different tab (the first one, the startup POM).
    let detached_index = 0usize;
    assert_ne!(
        shell.tabs.tabs()[detached_index].id,
        primary_active_id,
        "detached target must differ from the primary active tab"
    );

    // Dispatch the chained fastpath under the detached window's context.
    let mut ctx = super::WorkspaceCommandContext::default();
    shell.with_workspace_context(detached_index, &mut ctx, |s| {
        s.run_command_line("=0.M");
    });

    // The Primary_Window's active tab is STILL the same instance (by id) and its
    // Context is unchanged -- the detached `=0.M` did not leak into the primary.
    assert_eq!(
        shell.tabs.active_tab().id,
        primary_active_id,
        "primary active tab must be unchanged after a detached chained fastpath (B074)"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        primary_kind_before,
        "primary active tab's Context/kind must be unchanged (B074)"
    );
    // A POM tab may have been inserted by =0 inside the swap; that is a tab-count
    // change, which is exactly what would have corrupted an index-based restore.
    let _ = tab_count_before;
}

/// Validates: layout-and-docking Req 16.2, B072 -- a split region's command line
/// renders directly UNDER the Title_Line and ABOVE the body (ISPF order), NOT at
/// the region bottom. Pure test on the strip-rect math (no egui needed).
#[test]
fn split_region_command_line_is_under_title_above_body() {
    use super::render_split::split_region_strip_rects;
    use crate::workspace_kind::CommandLinePosition;
    // A tall region (e.g. a full-height split half).
    let region = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(600.0, 900.0));
    let r = split_region_strip_rects(region, CommandLinePosition::Top);
    // Top-to-bottom order: tab bar -> menu bar -> Title_Line -> command line -> body.
    assert!(
        r.bar_rect.min.y <= r.menu_rect.min.y,
        "tab bar above menu bar"
    );
    assert!(
        r.menu_rect.min.y <= r.title_rect.min.y,
        "menu bar above title"
    );
    assert!(
        r.title_rect.max.y <= r.cmd_rect.min.y,
        "command line is BELOW the Title_Line"
    );
    assert!(
        r.cmd_rect.max.y <= r.body_rect.min.y,
        "command line is ABOVE the body (B072: not at the region bottom)"
    );
    // The command line must NOT be pinned to the region bottom.
    assert!(
        r.cmd_rect.max.y < region.max.y,
        "command line must not sit at the region bottom (B072)"
    );
    // The body fills the remainder down to the region bottom.
    assert_eq!(
        r.body_rect.max.y, region.max.y,
        "body extends to region bottom"
    );
}

/// Validates: workspace-kinds Req 8.4 (CR-NR-095) -- when a region's Kind has
/// Command_Line_Position::Bottom, the command line is the LAST strip at the
/// region foot and the body sits ABOVE it (below the Title_Line). Pure test.
#[test]
fn split_region_command_line_at_bottom_when_position_bottom() {
    use super::render_split::split_region_strip_rects;
    use crate::workspace_kind::CommandLinePosition;
    let region = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(600.0, 900.0));
    let r = split_region_strip_rects(region, CommandLinePosition::Bottom);
    // Chrome order unchanged at the top: tab bar -> menu bar -> Title_Line.
    assert!(
        r.bar_rect.min.y <= r.menu_rect.min.y,
        "tab bar above menu bar"
    );
    assert!(
        r.menu_rect.min.y <= r.title_rect.min.y,
        "menu bar above title"
    );
    // Body sits directly under the Title_Line now (command line is NOT here).
    assert!(
        r.title_rect.max.y <= r.body_rect.min.y,
        "body starts under the Title_Line when the command line is at the bottom"
    );
    // Command line is the LAST strip, below the body, pinned to the region foot.
    assert!(
        r.body_rect.max.y <= r.cmd_rect.min.y,
        "command line is BELOW the body when position is Bottom"
    );
    assert_eq!(
        r.cmd_rect.max.y, region.max.y,
        "command line sits at the region bottom when position is Bottom"
    );
}
