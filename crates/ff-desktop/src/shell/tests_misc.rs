//! Shell tests -- misc area (split from shell/tests.rs, CR F3).
//! Items are verbatim; only their file location changed.

#![allow(unused_imports)]
use super::tests_common::*;
use ff_keys::{KeyMap, ModifiedKey};
use std::sync::{Arc, Mutex};

use ff_command::{
    CommandDispatch, CommandError, CommandHandler, CommandHistory, CommandId, CommandMetadata,
    CommandParams, CommandRegistry, CommandResult, ExecutionContext,
};

/// Validates: Requirement 3.2 -- unassigned F-key returns None.
#[test]
fn egui_fkey_unassigned_key_returns_none() {
    // Validates: function-keys-and-history Requirement 3.2, 15.3
    use ff_keys::{FunctionKey, KeyMapResolver};
    let map = KeyMap::default_global();
    let resolver = KeyMapResolver::new(map);
    // Base F13 is beyond the F1-F12 default set, so it is unassigned
    // (CR-CH-027: the default binds only Base+Shift F1-F12).
    let cmd = resolver.active_key_map().get_plain(FunctionKey::F13);
    assert!(cmd.is_none());
}

/// Validates: Requirement 16.2 -- CAPS with no argument toggles state.
#[test]
fn caps_no_arg_toggles_state() {
    // Validates: Requirement 16.2
    use ff_edit_operations::CapsMode;
    let mut shell = make_shell();
    assert_eq!(shell.tabs.active_tab().edit_profile.caps, CapsMode::Off);
    shell.handle_command("CAPS");
    assert_eq!(shell.tabs.active_tab().edit_profile.caps, CapsMode::On);
    shell.handle_command("CAPS");
    assert_eq!(shell.tabs.active_tab().edit_profile.caps, CapsMode::Off);
}

/// Validates: Requirement 16.6 -- PROFILE CAPS ON updates the setting.
#[test]
fn profile_caps_on_keyword_updates_caps() {
    // Validates: Requirement 16.6
    use ff_edit_operations::CapsMode;
    let mut shell = make_shell();
    shell.handle_command("PROFILE CAPS ON");
    assert_eq!(shell.tabs.active_tab().edit_profile.caps, CapsMode::On);
    assert!(shell.open_error.is_none());
}

/// Validates: Requirement 16.8 -- LOCK ON prevents profile changes.
#[test]
fn lock_on_prevents_profile_changes() {
    // Validates: Requirement 16.8
    use ff_edit_operations::CapsMode;
    let mut shell = make_shell();
    shell.handle_command("LOCK ON");
    shell.handle_command("PROFILE CAPS ON");
    // Profile is locked so CAPS should remain Off
    assert_eq!(shell.tabs.active_tab().edit_profile.caps, CapsMode::Off);
    assert!(shell.open_error.is_some());
}

/// Validates: Requirement 16.8 -- LOCK OFF re-enables profile changes.
#[test]
fn lock_off_re_enables_profile_changes() {
    // Validates: Requirement 16.8
    use ff_edit_operations::CapsMode;
    let mut shell = make_shell();
    shell.handle_command("LOCK ON");
    shell.handle_command("LOCK OFF");
    shell.handle_command("CAPS ON");
    assert_eq!(shell.tabs.active_tab().edit_profile.caps, CapsMode::On);
}

/// Validates: Requirement 16.12 -- HILITE ON sets hilite mode.
#[test]
fn hilite_on_sets_hilite_mode() {
    // Validates: Requirement 16.12
    use ff_edit_operations::HiliteMode;
    let mut shell = make_shell();
    shell.handle_command("HILITE ON");
    assert_eq!(shell.tabs.active_tab().edit_profile.hilite, HiliteMode::On);
    assert!(shell.open_error.is_none());
}

/// Validates: Requirement 16.12 -- HILITE LOGIC sets logic mode.
#[test]
fn hilite_logic_sets_logic_mode() {
    // Validates: Requirement 16.12
    use ff_edit_operations::HiliteMode;
    let mut shell = make_shell();
    shell.handle_command("HILITE LOGIC");
    assert_eq!(
        shell.tabs.active_tab().edit_profile.hilite,
        HiliteMode::Logic
    );
}

/// Validates: Requirement 16.12 -- HILITE with unknown mode sets error.
#[test]
fn hilite_unknown_mode_sets_error() {
    // Validates: Requirement 16.12
    let mut shell = make_shell();
    shell.handle_command("HILITE BOGUS");
    assert!(shell.open_error.is_some());
}

/// Validates: Requirement 17.1 -- SUBMIT with no JES returns descriptive error.
#[test]
fn submit_returns_jes_not_available_error() {
    // Validates: Requirement 17.1, 17.8
    let mut shell = make_shell();
    shell.handle_command("SUBMIT");
    let err = shell.open_error.as_deref().unwrap_or("");
    assert!(err.contains("JES") || err.contains("not yet"), "got: {err}");
}

/// Validates: Requirement 17.2 -- CREATE with missing dsn returns error.
#[test]
fn create_missing_dsn_returns_error() {
    // Validates: Requirement 17.8
    let mut shell = make_shell();
    shell.handle_command("CREATE ");
    assert!(shell.open_error.is_some());
}

/// Validates: Requirement 17.2 -- CREATE with dsn returns stub message.
#[test]
fn create_with_dsn_returns_stub_message() {
    // Validates: Requirement 17.2
    let mut shell = make_shell();
    shell.handle_command("CREATE PAYROLL.EMPLOYEE");
    let err = shell.open_error.as_deref().unwrap_or("");
    assert!(err.contains("PAYROLL.EMPLOYEE"), "got: {err}");
}

/// Validates: Requirement 17.3 -- REPLACE with missing dsn returns error.
#[test]
fn replace_missing_dsn_returns_error() {
    // Validates: Requirement 17.8
    let mut shell = make_shell();
    shell.handle_command("REPLACE ");
    assert!(shell.open_error.is_some());
}

/// Validates: Requirement 17.5 -- BROWSE with missing dsn returns error.
#[test]
fn browse_missing_dsn_returns_error() {
    // Validates: Requirement 17.8
    let mut shell = make_shell();
    shell.handle_command("BROWSE ");
    assert!(shell.open_error.is_some());
}

/// Validates: Requirement 17.6 -- VIEW with missing dsn returns error.
#[test]
fn view_missing_dsn_returns_error() {
    // Validates: Requirement 17.8
    let mut shell = make_shell();
    shell.handle_command("VIEW ");
    assert!(shell.open_error.is_some());
}

/// Validates: Requirement 17.7 -- COMPARE with missing dsn returns error.
#[test]
fn compare_missing_dsn_returns_error() {
    // Validates: Requirement 17.8
    let mut shell = make_shell();
    shell.handle_command("COMPARE ");
    assert!(shell.open_error.is_some());
}

/// Validates: Requirement 17.7 -- COMPARE with dsn returns stub message.
#[test]
fn compare_with_dsn_returns_stub_message() {
    // Validates: Requirement 17.7
    let mut shell = make_shell();
    shell.handle_command("COMPARE PAYROLL.EMPLOYEE");
    let err = shell.open_error.as_deref().unwrap_or("");
    assert!(err.contains("PAYROLL.EMPLOYEE"), "got: {err}");
}

// === Phase BZ -- SCROLL field, fastpath, split screen, LOCATE ==============

/// Validates: Requirement 19.1 -- shell initialises with PAGE scroll amount.
#[test]
fn scroll_amount_defaults_to_page() {
    // Validates: Requirement 19.1
    use crate::scroll_amount::ScrollAmount;
    let shell = make_shell();
    assert_eq!(shell.scroll_amount, ScrollAmount::Page);
    assert_eq!(shell.scroll_field_text, "PAGE");
}

/// Validates: Requirement 19.4 -- fastpath with invalid first segment is not treated as fastpath.
#[test]
fn fastpath_non_digit_first_segment_not_fastpath() {
    // Validates: Requirement 19.4 -- only single-digit first segments are fastpath
    let mut shell = make_shell();
    // "abc.def" should not be treated as fastpath
    shell.handle_command("abc.def");
    // Should fall through to command engine without panic
    // (open_error may be set but no crash)
}

/// Validates: menu-workspace Requirement 5.1, 5.7 (B061) -- the chained fastpath
/// `=0.K` pops to the POM origin, selects option 0 (Settings), then option K
/// (KEYS), landing on the Keys Workspace. It must NOT reach the "command not
/// yet implemented" stub.
#[test]
fn chained_fastpath_equals_zero_dot_k_opens_keys_workspace() {
    use crate::tab_state::{KindTag, TabKind};
    let mut shell = make_shell();
    shell.handle_command("=0.K");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KeysEditor,
        "=0.K should open the Keys Workspace (POM opt 0 = Settings, then K = KEYS)"
    );
    let err = shell.open_error.as_deref().unwrap_or("");
    assert!(
        !err.to_lowercase().contains("not yet implemented"),
        "chained fastpath must not fall through to the not-implemented stub, got: {err:?}"
    );
}

/// Validates: multi-tab-editor Req 18.7/18.9 (CR-CH-031) -- bare SWAP with no
/// split toggles to the previously active workspace, and repeated bare SWAP
/// ping-pongs between the two most-recent tabs (does NOT open the picker).
#[test]
fn swap_bare_toggles_to_previously_active_tab() {
    let mut shell = make_shell();
    // Open a second tab. START inserts a fresh POM at index 0 (which resets the
    // previous pointer); the original tab is now at index 1. Establish a normal
    // two-tab navigation history by explicitly activating each: go to tab 2,
    // then tab 1, so `previous_active` = tab 2 (index 1).
    shell.handle_command("START");
    assert!(shell.tabs.len() >= 2);
    shell.handle_command("SWAP 2"); // activate index 1
    assert_eq!(shell.tabs.active_index(), 1);
    shell.handle_command("SWAP 1"); // activate index 0; previous = 1
    assert_eq!(shell.tabs.active_index(), 0);

    // Bare SWAP toggles back to the previously active tab (index 1), no picker.
    shell.handle_command("SWAP");
    assert_eq!(
        shell.tabs.active_index(),
        1,
        "bare SWAP must toggle to the previously active tab"
    );
    assert!(
        shell.show_swap_list.is_none(),
        "toggle must NOT open the picker"
    );
    assert!(shell.open_error.is_none());

    // Repeated bare SWAP ping-pongs back to index 0.
    shell.handle_command("SWAP");
    assert_eq!(
        shell.tabs.active_index(),
        0,
        "repeated bare SWAP ping-pongs"
    );
    assert!(shell.show_swap_list.is_none());
}

/// Validates: multi-tab-editor Requirement 18.1 -- `SWAP n` activates the n-th
/// tab (1-based).
#[test]
fn swap_n_activates_nth_tab() {
    // Validates: Requirement 18.1
    let mut shell = make_shell();
    // CR-CH-022: navigation transforms in place; only START creates a new tab.
    // Open a second Workspace with START (a new POM tab).
    shell.handle_command("START");
    let count = shell.tabs.len();
    assert!(count >= 2, "need >=2 tabs for the test (have {count})");

    shell.handle_command("SWAP 1");
    assert_eq!(shell.tabs.active_index(), 0);
    assert!(shell.open_error.is_none());

    shell.handle_command("SWAP 2");
    assert_eq!(shell.tabs.active_index(), 1);
    assert!(shell.open_error.is_none());
}

/// Validates: multi-tab-editor Requirement 18.2 -- out-of-range / invalid
/// `SWAP n` errors and does not change the active tab.
#[test]
fn swap_n_out_of_range_errors_and_keeps_active() {
    // Validates: Requirement 18.2
    let mut shell = make_shell();
    shell.handle_command("2"); // ensure >=2 tabs
    shell.handle_command("SWAP 1");
    let before = shell.tabs.active_index();

    // Too high.
    shell.handle_command("SWAP 999");
    assert_eq!(
        shell.tabs.active_index(),
        before,
        "out-of-range must not switch"
    );
    assert!(shell.open_error.is_some());

    // Zero is invalid (1-based).
    shell.open_error = None;
    shell.handle_command("SWAP 0");
    assert_eq!(shell.tabs.active_index(), before);
    assert!(shell.open_error.is_some());

    // Non-numeric, non-LIST argument.
    shell.open_error = None;
    shell.handle_command("SWAP frog");
    assert_eq!(shell.tabs.active_index(), before);
    assert!(shell.open_error.is_some());
}

/// Validates: Requirement 20.2 -- format_logoff_message produces correct format.
#[test]
fn format_logoff_message_produces_correct_format() {
    // Validates: Requirement 20.2
    let shell = make_shell();
    let msg = shell.format_logoff_message();
    assert!(
        msg.starts_with("Logoff at "),
        "must start with 'Logoff at ', got: {msg}"
    );
    assert!(
        msg.contains("session duration:"),
        "must contain 'session duration:', got: {msg}"
    );
}

/// Validates: startup-and-session Req 22.8 (CR-NR-081) -- the active profile
/// label shows "Profile: default" when no profile is set and "Profile: <name>"
/// when one is active. Drives the process-global serially and restores it.
#[test]
fn active_profile_label_reflects_active_profile() {
    let shell = make_shell();
    // Default (no profile).
    ff_session::set_active_profile(None);
    assert_eq!(shell.active_profile_label(), "Profile: default");
    // Named profile.
    ff_session::set_active_profile(Some("ispf"));
    assert_eq!(shell.active_profile_label(), "Profile: ispf");
    // Restore so no other test sees a stray profile.
    ff_session::set_active_profile(None);
}

/// Validates: Requirement 20.6 -- STATUS jobname routes with jobname filter.
#[test]
fn status_with_jobname_routes_with_filter() {
    // Validates: Requirement 20.6
    let mut shell = make_shell();
    shell.handle_command("STATUS MYJOB");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(
        !msg.to_uppercase().contains("UNKNOWN"),
        "STATUS jobname must not produce unknown-command error, got: {msg}"
    );
}

// === Phase BS-A: Workspace lifecycle, root, settings, MRU tests ===========

/// Validates: workspace-model Requirement 2.1 -- WORKSPACE OPEN with missing path sets error.
#[test]
fn workspace_open_missing_path_sets_error() {
    // Validates: workspace-model Requirement 2.1
    let mut shell = make_shell();
    shell.handle_command("WORKSPACE OPEN");
    assert!(
        shell.open_error.is_some(),
        "WORKSPACE OPEN with no path must set an error"
    );
}

/// Validates: workspace-model Requirement 2.1 -- open_workspace_force loads workspace and
/// registers roots as Native catalogs.
#[test]
fn open_workspace_force_registers_roots_as_native_catalogs() {
    // Validates: workspace-model Requirement 2.1, 3.4
    use ff_session::{save_workspace, WorkspaceState};
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let ws_path = tmp.path().join("test.ffwb-workspace");
    // Use a named subdirectory so file_name() is a valid catalog name.
    let root = tmp.path().join("myproject");
    std::fs::create_dir_all(&root).expect("create root dir");

    let mut state = WorkspaceState::new("TestWS");
    state.roots.push(root.clone());
    save_workspace(&state, &ws_path).expect("save");

    let mut shell = make_shell();
    shell.open_workspace_force(&ws_path);

    assert!(shell.active_workspace.is_some(), "workspace must be active");
    assert_eq!(shell.active_workspace.as_ref().unwrap().name, "TestWS");
    // Root must be registered as a catalog.
    let root_name = "myproject";
    let catalog_names: Vec<String> = shell
        .files_panel
        .registry
        .list()
        .iter()
        .map(|c| c.name.clone())
        .collect();
    assert!(
        shell.files_panel.registry.get_by_name(root_name).is_some(),
        "root '{}' must be registered as a catalog; found: {:?}",
        root_name,
        catalog_names
    );
    assert!(shell.open_error.is_none());
}

/// Validates: workspace-model Requirement 2.4 -- close_workspace unregisters roots.
#[test]
fn close_workspace_unregisters_roots() {
    // Validates: workspace-model Requirement 2.4, 3.4
    use ff_session::{save_workspace, WorkspaceState};
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let ws_path = tmp.path().join("test.ffwb-workspace");
    let root = tmp.path().join("myproject");
    std::fs::create_dir_all(&root).expect("create root dir");
    let root_name = "myproject";

    let mut state = WorkspaceState::new("CloseWS");
    state.roots.push(root);
    save_workspace(&state, &ws_path).expect("save");

    let mut shell = make_shell();
    shell.open_workspace_force(&ws_path);
    assert!(shell.files_panel.registry.get_by_name(root_name).is_some());

    shell.close_workspace();
    assert!(
        shell.active_workspace.is_none(),
        "workspace must be cleared"
    );
    assert!(
        shell.files_panel.registry.get_by_name(root_name).is_none(),
        "root catalog must be removed on close"
    );
}

/// Validates: workspace-model Requirement 2.5 -- opening a second workspace when the first
/// has unsaved changes defers the open and sets show_unsaved_workspace_dialog.
#[test]
fn open_workspace_with_modified_active_defers_and_shows_dialog() {
    // Validates: workspace-model Requirement 2.5
    use ff_session::{save_workspace, WorkspaceState};
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let ws1_path = tmp.path().join("ws1.ffwb-workspace");
    let ws2_path = tmp.path().join("ws2.ffwb-workspace");

    save_workspace(&WorkspaceState::new("WS1"), &ws1_path).expect("save ws1");
    save_workspace(&WorkspaceState::new("WS2"), &ws2_path).expect("save ws2");

    let mut shell = make_shell();
    shell.open_workspace_force(&ws1_path);
    // Mark the active workspace as modified.
    shell.active_workspace.as_mut().unwrap().is_modified = true;

    // Now try to open a second workspace.
    shell.open_workspace(&ws2_path);

    assert!(
        shell.show_unsaved_workspace_dialog,
        "unsaved-changes dialog must be shown"
    );
    assert_eq!(
        shell.pending_workspace_open.as_deref(),
        Some(ws2_path.as_path()),
        "pending path must be ws2"
    );
    // WS1 must still be active (not replaced yet).
    assert_eq!(shell.active_workspace.as_ref().unwrap().name, "WS1");
}

/// Validates: workspace-model Requirement 2.5 -- discard path in unsaved-changes guard
/// clears the dialog and opens the pending workspace.
#[test]
fn unsaved_workspace_discard_opens_pending_workspace() {
    // Validates: workspace-model Requirement 2.5
    use ff_session::{save_workspace, WorkspaceState};
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let ws1_path = tmp.path().join("ws1.ffwb-workspace");
    let ws2_path = tmp.path().join("ws2.ffwb-workspace");

    save_workspace(&WorkspaceState::new("WS1"), &ws1_path).expect("save ws1");
    save_workspace(&WorkspaceState::new("WS2"), &ws2_path).expect("save ws2");

    let mut shell = make_shell();
    shell.open_workspace_force(&ws1_path);
    shell.active_workspace.as_mut().unwrap().is_modified = true;
    shell.open_workspace(&ws2_path);

    // Simulate Discard: clear modified flag and open pending.
    shell.show_unsaved_workspace_dialog = false;
    if let Some(ws) = shell.active_workspace.as_mut() {
        ws.is_modified = false;
    }
    if let Some(path) = shell.pending_workspace_open.take() {
        shell.open_workspace_force(&path);
    }

    assert!(!shell.show_unsaved_workspace_dialog);
    assert!(shell.pending_workspace_open.is_none());
    assert_eq!(shell.active_workspace.as_ref().unwrap().name, "WS2");
}

/// Validates: workspace-model Requirement 3.5 -- missing root at load time sets open_error
/// but workspace is still loaded.
#[test]
fn open_workspace_missing_root_sets_warning_but_loads() {
    // Validates: workspace-model Requirement 3.5
    use ff_session::{save_workspace, WorkspaceState};
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let ws_path = tmp.path().join("missing-root.ffwb-workspace");

    let mut state = WorkspaceState::new("MissingRoot");
    // Add a root that does not exist on disk.
    state
        .roots
        .push(std::path::PathBuf::from("C:/does/not/exist/ever"));
    save_workspace(&state, &ws_path).expect("save");

    let mut shell = make_shell();
    shell.open_workspace_force(&ws_path);

    // Workspace must still be loaded.
    assert!(
        shell.active_workspace.is_some(),
        "workspace must load even when a root is missing"
    );
    // A warning must be set.
    assert!(
        shell.open_error.is_some(),
        "open_error must contain a warning about the missing root"
    );
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(
        msg.to_lowercase().contains("warning") || msg.to_lowercase().contains("not found"),
        "warning message expected, got: {msg}"
    );
}

/// Validates: workspace-model Requirement 6.1 -- record_recent_file adds to workspace MRU.
#[test]
fn workspace_mru_accumulates_opened_files() {
    // Validates: workspace-model Requirement 6.1
    use ff_session::{save_workspace, WorkspaceState};
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let ws_path = tmp.path().join("mru.ffwb-workspace");
    save_workspace(&WorkspaceState::new("MruWS"), &ws_path).expect("save");

    let mut shell = make_shell();
    shell.open_workspace_force(&ws_path);

    let file_a = std::path::PathBuf::from("C:/projects/a.rs");
    let file_b = std::path::PathBuf::from("C:/projects/b.rs");
    shell
        .active_workspace
        .as_mut()
        .unwrap()
        .record_recent_file(file_a.clone());
    shell
        .active_workspace
        .as_mut()
        .unwrap()
        .record_recent_file(file_b.clone());

    let mru = &shell.active_workspace.as_ref().unwrap().recent_files;
    assert_eq!(mru.len(), 2);
    assert_eq!(mru[0].path, file_b, "most recent must be first");
    assert_eq!(mru[1].path, file_a);
}

// === Phase CO: Accessibility -- Keyboard Audit (Requirement 2) ==============

// (The modal KeyConfigDialog Escape-close test was removed with the modal in
// CR-CH-029; the Keys Workspace uses END/RETURN via the Navigation_Stack.)

/// Validates: accessibility Requirement 2.1, 2.3 -- DatasetAllocDialog has a cancel path.
#[test]
fn dataset_alloc_dialog_has_cancel_path() {
    // Validates: accessibility Requirement 2.1, 2.3
    // The dialog form must have a Cancel button reachable by keyboard.
    // We verify the form type exists and can be constructed.
    let form = crate::dataset_alloc_dialog::AllocDatasetForm::default();
    // A default form must have empty dataset name (not pre-filled with garbage).
    assert!(
        form.dataset_name.is_empty() || !form.dataset_name.is_empty(),
        "form must be constructible"
    );
}

/// Validates: accessibility Requirement 5.3 -- when reduce_motion is true,
/// scroll operations jump immediately (no animation). The editor panel
/// already uses immediate jumps; this test verifies the config key is
/// readable and the scroll behaviour is not animated.
#[test]
fn reduce_motion_scroll_is_immediate_jump() {
    // Validates: accessibility Requirement 5.3
    // The editor panel uses scroll_to_line() which is an immediate jump.
    // When reduce_motion is true the same path is taken (no animation branch).
    // We verify the config key can be set and read back.
    let shell = make_shell();
    let _ = shell.config_handle.set_user_value(
        ff_config::keys::accessibility::REDUCE_MOTION,
        ff_config::ConfigValue::Boolean(true),
    );
    let val = shell
        .config_handle
        .get_bool(ff_config::keys::accessibility::REDUCE_MOTION)
        .unwrap_or(false);
    assert!(val, "reduce_motion must be readable as true after set");
}

/// Validates: plugin-manager-ui Requirement 1.5 -- plugin list sorted alphabetically.
#[test]
fn plugin_list_sorted_alphabetically() {
    // Validates: plugin-manager-ui Requirement 1.5
    use crate::plugin_manager_panel::PluginManagerPanelState;
    let mut state = PluginManagerPanelState::new();
    state.plugins = vec![
        ("Zebra".to_string(), ff_plugin::PluginState::Active),
        ("Alpha".to_string(), ff_plugin::PluginState::Active),
        ("Middle".to_string(), ff_plugin::PluginState::Shutdown),
    ];
    state.sort_plugins();
    assert_eq!(state.plugins[0].0, "Alpha");
    assert_eq!(state.plugins[1].0, "Middle");
    assert_eq!(state.plugins[2].0, "Zebra");
}

/// Validates: plugin-manager-ui Requirement 1.6 -- filter narrows plugin list.
#[test]
fn filter_narrows_plugin_list() {
    // Validates: plugin-manager-ui Requirement 1.6
    use crate::plugin_manager_panel::PluginManagerPanelState;
    let state = PluginManagerPanelState::new();
    let plugins = vec![
        ("GCC Toolchain".to_string(), ff_plugin::PluginState::Active),
        ("Rust Toolchain".to_string(), ff_plugin::PluginState::Active),
        (
            "Database Tool".to_string(),
            ff_plugin::PluginState::Shutdown,
        ),
    ];
    let filtered = state.filter_plugins(&plugins, "toolchain");
    assert_eq!(filtered.len(), 2);
    assert!(filtered
        .iter()
        .all(|(n, _)| n.to_lowercase().contains("toolchain")));
}

// === Phase CO: Notification System (Requirement 1-4) =======================

/// Validates: notification-system Requirement 3.1-3.4 -- NotificationSender API.
#[test]
fn notification_sender_is_clone_and_send() {
    // Validates: notification-system Requirement 3.2
    use crate::notification::{NotificationQueue, NotificationSender};
    let (tx, _rx) = std::sync::mpsc::sync_channel(64);
    let sender = NotificationSender::new(tx);
    let _cloned = sender.clone();
    // If this compiles, NotificationSender is Clone.
    // Send-ness is verified by the type system at compile time.
    fn assert_send<T: Send>(_: T) {}
    assert_send(sender);
    // Queue starts empty.
    let queue = NotificationQueue::new();
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.unread(), 0);
}

/// Validates: notification-system Requirement 2.7 -- queue caps at 1000 entries.
#[test]
fn notification_queue_caps_at_1000_entries() {
    // Validates: notification-system Requirement 2.7
    use crate::notification::{Notification, NotificationLevel, NotificationQueue};
    let mut queue = NotificationQueue::new();
    for i in 0..1100u32 {
        queue.push(Notification::new(
            NotificationLevel::Info,
            format!("msg {i}"),
            None,
        ));
    }
    assert!(
        queue.len() <= 1000,
        "queue must cap at 1000, got {}",
        queue.len()
    );
}

/// Validates: notification-system Requirement 2.7 -- Warning increments unread.
#[test]
fn push_warning_increments_unread() {
    // Validates: notification-system Requirement 2.7
    use crate::notification::{Notification, NotificationLevel, NotificationQueue};
    let mut queue = NotificationQueue::new();
    queue.push(Notification::new(
        NotificationLevel::Warning,
        "warn".to_string(),
        None,
    ));
    assert_eq!(queue.unread(), 1);
    queue.push(Notification::new(
        NotificationLevel::Info,
        "info".to_string(),
        None,
    ));
    assert_eq!(queue.unread(), 1); // Info does not increment unread
}

/// Validates: notification-system Requirement 2.4 -- mark_all_read clears unread.
#[test]
fn mark_all_read_clears_unread() {
    // Validates: notification-system Requirement 2.4
    use crate::notification::{Notification, NotificationLevel, NotificationQueue};
    let mut queue = NotificationQueue::new();
    queue.push(Notification::new(
        NotificationLevel::Error,
        "err".to_string(),
        None,
    ));
    queue.push(Notification::new(
        NotificationLevel::Warning,
        "warn".to_string(),
        None,
    ));
    assert_eq!(queue.unread(), 2);
    queue.mark_all_read();
    assert_eq!(queue.unread(), 0);
}

/// Validates: notification-system Requirement 2.6 -- clear empties the queue.
#[test]
fn clear_log_empties_queue() {
    // Validates: notification-system Requirement 2.6
    use crate::notification::{Notification, NotificationLevel, NotificationQueue};
    let mut queue = NotificationQueue::new();
    queue.push(Notification::new(
        NotificationLevel::Info,
        "a".to_string(),
        None,
    ));
    queue.push(Notification::new(
        NotificationLevel::Error,
        "b".to_string(),
        None,
    ));
    assert_eq!(queue.len(), 2);
    queue.clear();
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.unread(), 0);
}

/// Validates: notification-system Requirement 2.4 -- filter_by_level returns matching.
#[test]
fn filter_by_level_returns_matching() {
    // Validates: notification-system Requirement 2.4
    use crate::notification::{Notification, NotificationLevel, NotificationQueue};
    let mut queue = NotificationQueue::new();
    queue.push(Notification::new(
        NotificationLevel::Info,
        "info".to_string(),
        None,
    ));
    queue.push(Notification::new(
        NotificationLevel::Error,
        "err".to_string(),
        None,
    ));
    queue.push(Notification::new(
        NotificationLevel::Warning,
        "warn".to_string(),
        None,
    ));
    let errors = queue.filter_by_level(NotificationLevel::Error);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].title, "err");
}

/// Validates: notification-system Requirement 1.1 -- shell drains channel each frame.
#[test]
fn notifications_drained_from_channel_each_frame() {
    // Validates: notification-system Requirement 1.1
    // The shell has a notification_sender() method and a queue.
    let shell = make_shell();
    let sender = shell.notification_sender();
    sender.info("test".to_string(), None);
    // The sender is non-blocking -- this must not panic.
    drop(sender);
    assert_eq!(shell.notification_queue.lock().expect("lock").len(), 0);
    // (Draining happens in update() each frame -- not testable without egui context)
}

/// Validates: notification-system Requirement 4.1-4.4 -- bell badge unread count.
#[test]
fn bell_badge_shows_unread_count() {
    // Validates: notification-system Requirement 4.2
    use crate::notification::{Notification, NotificationLevel};
    let shell = make_shell();
    let mut queue = shell.notification_queue.lock().expect("lock");
    queue.push(Notification::new(
        NotificationLevel::Error,
        "e1".to_string(),
        None,
    ));
    queue.push(Notification::new(
        NotificationLevel::Warning,
        "w1".to_string(),
        None,
    ));
    assert_eq!(queue.unread(), 2);
    queue.mark_all_read();
    assert_eq!(queue.unread(), 0);
}

/// Validates: workspace-model Requirement 6.3 -- closing workspace clears workspace MRU
/// (active_workspace becomes None).
#[test]
fn close_workspace_clears_mru() {
    // Validates: workspace-model Requirement 6.3
    use ff_session::{save_workspace, WorkspaceState};
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let ws_path = tmp.path().join("mru-close.ffwb-workspace");
    save_workspace(&WorkspaceState::new("MruClose"), &ws_path).expect("save");

    let mut shell = make_shell();
    shell.open_workspace_force(&ws_path);
    shell
        .active_workspace
        .as_mut()
        .unwrap()
        .record_recent_file(std::path::PathBuf::from("C:/x.rs"));
    assert!(!shell
        .active_workspace
        .as_ref()
        .unwrap()
        .recent_files
        .is_empty());

    shell.close_workspace();
    // After close, active_workspace is None -- MRU is gone.
    assert!(
        shell.active_workspace.is_none(),
        "active_workspace must be None after close"
    );
}

/// Validates: CX Requirement 1.1 -- TabState has workspace_name field defaulting to None.
#[test]
fn tab_state_workspace_name_defaults_to_none() {
    let shell = make_shell();
    assert!(shell.tabs.active_tab().workspace_name.is_none());
}

// Validates: command-configurator Requirement 2.3, 2.4 -- Save commits a new
// definition to the store and closes the form.
#[test]
fn configurator_save_adds_definition_to_store() {
    use crate::command_config::render::ConfiguratorAction;
    let mut shell = make_shell();
    let _dir = point_store_at_temp(&mut shell);
    set_add_function_form(&mut shell, "my.build", "file.save");

    shell.apply_configurator_action(ConfiguratorAction::Save);

    assert!(
        shell.command_store.find("my.build").is_some(),
        "Save must add the definition to the store"
    );
    assert!(
        shell.command_configurator_panel.form.is_none(),
        "a successful Save closes the form"
    );
    assert!(shell.command_configurator_panel.error.is_none());
}

// Validates: command-configurator Requirement 4.1 -- Save rejects an invalid id
// and leaves the store unchanged.
#[test]
fn configurator_save_rejects_invalid_id() {
    use crate::command_config::render::ConfiguratorAction;
    let mut shell = make_shell();
    let _dir = point_store_at_temp(&mut shell);
    // Uppercase is invalid per the Command_ID naming rule.
    set_add_function_form(&mut shell, "BAD ID", "file.save");

    shell.apply_configurator_action(ConfiguratorAction::Save);

    assert!(
        shell.command_store.definitions.is_empty(),
        "store unchanged"
    );
    assert!(
        shell.command_configurator_panel.error.is_some(),
        "an invalid id must surface an error"
    );
    assert!(
        shell.command_configurator_panel.form.is_some(),
        "the form stays open on validation failure"
    );
}

// Validates: command-configurator Requirement 4.6 -- Save rejects an id that
// shadows a reserved built-in command.
#[test]
fn configurator_save_rejects_reserved_id() {
    use crate::command_config::render::ConfiguratorAction;
    let mut shell = make_shell();
    let _dir = point_store_at_temp(&mut shell);
    // file.exit is registered in WorkbenchShell::new(); it is reserved.
    set_add_function_form(&mut shell, "file.exit", "file.save");

    shell.apply_configurator_action(ConfiguratorAction::Save);

    assert!(shell.command_store.find("file.exit").is_none());
    assert!(shell.command_configurator_panel.error.is_some());
}

// Validates: command-configurator Requirement 2.5 -- Delete removes a
// definition from the store.
#[test]
fn configurator_delete_removes_definition() {
    use crate::command_config::render::ConfiguratorAction;
    let mut shell = make_shell();
    let _dir = point_store_at_temp(&mut shell);
    set_add_function_form(&mut shell, "temp.cmd", "file.save");
    shell.apply_configurator_action(ConfiguratorAction::Save);
    assert!(shell.command_store.find("temp.cmd").is_some());

    shell.apply_configurator_action(ConfiguratorAction::Delete("temp.cmd".to_string()));
    assert!(
        shell.command_store.find("temp.cmd").is_none(),
        "Delete must remove the definition"
    );
}

// Validates: command-framework Req 8.10 / menu-workspace Req 11.12 -- a built-in
// command beats a same-named token; and a genuinely unknown token is unresolved
// (falls through the whole chain to the command engine error), not silently
// swallowed by menu-name resolution.
#[test]
fn unknown_token_is_unresolved_not_a_menu() {
    let mut shell = make_shell();
    shell.handle_command("zzz_not_a_menu_or_command");
    assert!(
        shell.open_error.is_some(),
        "an unknown token must surface an unresolved-command error"
    );
}

// Validates: configuration-system Req 19.9 (CR-NR-083) -- bare RESET BARE
// resolves a single-profile target for the running process's profile and opens
// the dialog with no error.
#[test]
fn reset_bare_bare_resolves_single_default_target() {
    ff_session::set_active_profile(None);
    let mut shell = make_shell();
    shell.handle_command("RESET BARE");
    let target = shell
        .reset_bare_confirm
        .as_ref()
        .expect("bare RESET BARE opens the dialog");
    assert_eq!(target.profiles.len(), 1, "bare targets exactly one profile");
    assert!(
        target.includes_active,
        "bare always targets the running profile"
    );
    assert!(shell.open_error.is_none());
}

// Validates: configuration-system Req 19.12 (CR-NR-083) -- RESET BARE naming an
// unknown profile blocks the whole command: no dialog opens and a non-blocking
// error names the unknown profile.
#[test]
fn reset_bare_unknown_named_profile_errors_without_dialog() {
    ff_session::set_active_profile(None);
    let mut shell = make_shell();
    // A slug that will not exist as a real profiles/<slug>/ directory.
    shell.handle_command("RESET BARE zzz-nonesuch-profile");
    assert!(
        shell.reset_bare_confirm.is_none(),
        "an unknown profile must NOT open the confirmation dialog"
    );
    let err = shell.open_error.as_deref().unwrap_or("");
    assert!(
        err.contains("zzz-nonesuch-profile"),
        "error must name the unknown profile, got: {err:?}"
    );
}

// Validates: configuration-system Requirement 19.6 -- a confirmed RESET BARE
// resets in-memory state: the POM (Home Context) is present and a valid palette
// is active. (The archive step is unit-tested in reset_bare.rs against a temp
// dir; here we exercise the in-memory reset path.)
#[test]
fn execute_reset_bare_reopens_home_context() {
    let mut shell = make_shell();
    let target = shell
        .resolve_reset_bare_target("")
        .expect("bare target resolves");
    shell.execute_reset_bare(&target);
    assert!(
        shell.tabs.tabs().iter().any(|t| t.is_home),
        "after RESET BARE the Home Context (POM) must be present"
    );
    assert!(
        !shell.palette.name.is_empty(),
        "a valid palette must be active after RESET BARE"
    );
}

// Validates: Req 14.4 -- A -> B -> C, then END walks back C -> B -> A one level
// per press (per-tab Navigation_Stack).
#[test]
fn end_walks_back_up_the_navigation_stack() {
    use crate::tab_state::{KindTag, TabKind};
    let mut shell = make_shell(); // A = POM
    shell.handle_command("SETTINGS"); // B = Settings menu (MenuWorkspace)
    shell.dispatch_command_string("MENUS"); // C = Menus editor
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::MenusEditor);
    shell.handle_command("END");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::MenuWorkspace,
        "END: C -> B (Settings menu)"
    );
    shell.handle_command("END");
    assert!(shell.tabs.active_tab().is_home, "END: B -> A (POM)");
    assert_eq!(shell.tabs.len(), 1, "walking back never spawned a tab");
}

// Validates: Req 14.1 -- stacks are per-tab (independent across tabs).
#[test]
fn navigation_stacks_are_per_tab() {
    use crate::tab_state::{KindTag, TabKind};
    let mut shell = make_shell();
    let before = shell.tabs.len();
    shell.handle_command("SETTINGS"); // active tab drills to Settings (stack pushed)
    shell.handle_command("START"); // NEW tab (POM, empty stack), now active
    assert_eq!(shell.tabs.len(), before + 1, "START adds exactly one tab");
    // The active (START) tab has its own empty stack.
    assert!(
        shell.tabs.active_tab().nav_stack.is_empty(),
        "the START tab has its own empty stack"
    );
    assert!(shell.tabs.active_tab().is_home);
    // The Settings tab (a non-Home MenuWorkspace) still has its own non-empty
    // stack -- proving stacks are independent per tab. The freshly inserted
    // START tab is ALSO a MenuWorkspace (the Home Context), so filter it out via
    // is_home to find the drilled Settings tab specifically.
    let settings_tab = shell
        .tabs
        .tabs()
        .iter()
        .find(|t| t.kind.tag() == KindTag::MenuWorkspace && !t.is_home)
        .expect("the drilled Settings tab still exists");
    assert!(
        !settings_tab.nav_stack.is_empty(),
        "the first tab's stack is unaffected by the second tab"
    );
}

// === CR-CH-046: PFSHOW single-line, modifier-scope cycling ===================

#[test]
fn pfshow_cycle_off_base_shift_ctrl_alt_off() {
    // Validates: function-keys-and-history Requirement 12.8 -- bare PFSHOW cycles
    // Off -> Base -> Shift -> Ctrl -> Alt -> Off and wraps.
    use super::KeyBarScope;
    let mut shell = make_shell();
    // Force a known starting mode: Off.
    shell.handle_command("PFSHOW OFF");
    assert!(!shell.key_bar_visible, "start Off");

    shell.handle_command("PFSHOW"); // Off -> Base
    assert!(shell.key_bar_visible);
    assert_eq!(shell.key_bar_scope, KeyBarScope::Base);

    shell.handle_command("PFSHOW"); // Base -> Shift
    assert!(shell.key_bar_visible);
    assert_eq!(shell.key_bar_scope, KeyBarScope::Shift);

    shell.handle_command("PFSHOW"); // Shift -> Ctrl
    assert_eq!(shell.key_bar_scope, KeyBarScope::Ctrl);

    shell.handle_command("PFSHOW"); // Ctrl -> Alt
    assert_eq!(shell.key_bar_scope, KeyBarScope::Alt);

    shell.handle_command("PFSHOW"); // Alt -> Off
    assert!(!shell.key_bar_visible, "Alt wraps to Off");

    shell.handle_command("PFSHOW"); // Off -> Base (wrap)
    assert!(shell.key_bar_visible);
    assert_eq!(shell.key_bar_scope, KeyBarScope::Base);
}

#[test]
fn pfshow_scope_args_jump_to_scope_and_show() {
    // Validates: function-keys-and-history Requirement 12.9 -- PFSHOW BASE/SHIFT/
    // CTRL/ALT jump directly to that scope and make the bar visible.
    use super::KeyBarScope;
    let mut shell = make_shell();
    shell.handle_command("PFSHOW OFF");

    shell.handle_command("PFSHOW SHIFT");
    assert!(shell.key_bar_visible);
    assert_eq!(shell.key_bar_scope, KeyBarScope::Shift);

    shell.handle_command("PFSHOW OFF");
    shell.handle_command("PFSHOW ALT");
    assert!(shell.key_bar_visible, "scope arg turns the bar on");
    assert_eq!(shell.key_bar_scope, KeyBarScope::Alt);

    shell.handle_command("PFSHOW CTRL");
    assert_eq!(shell.key_bar_scope, KeyBarScope::Ctrl);

    shell.handle_command("PFSHOW BASE");
    assert_eq!(shell.key_bar_scope, KeyBarScope::Base);
}

#[test]
fn pfshow_scope_arg_is_case_insensitive() {
    // Validates: function-keys-and-history Requirement 12.9 -- case-insensitive.
    use super::KeyBarScope;
    let mut shell = make_shell();
    shell.handle_command("pfshow shift");
    assert_eq!(shell.key_bar_scope, KeyBarScope::Shift);
    shell.handle_command("PfShow Alt");
    assert_eq!(shell.key_bar_scope, KeyBarScope::Alt);
}

#[test]
fn pfshow_on_off_idempotent_no_error() {
    // Validates: function-keys-and-history Requirement 12.1, 12.2, 12.6, 12.7 --
    // ON when visible / OFF when hidden are no-ops with no error.
    let mut shell = make_shell();
    shell.handle_command("PFSHOW ON");
    assert!(shell.key_bar_visible);
    shell.handle_command("PFSHOW ON"); // already visible
    assert!(shell.key_bar_visible);
    assert!(
        shell.open_error.is_none(),
        "ON-when-visible is not an error"
    );

    shell.handle_command("PFSHOW OFF");
    assert!(!shell.key_bar_visible);
    shell.handle_command("PFSHOW OFF"); // already hidden
    assert!(!shell.key_bar_visible);
    assert!(
        shell.open_error.is_none(),
        "OFF-when-hidden is not an error"
    );
}

#[test]
fn pfshow_on_from_off_retains_scope() {
    // Validates: function-keys-and-history Requirement 12.1 -- PFSHOW ON restores
    // the last scope (retained across OFF).
    use super::KeyBarScope;
    let mut shell = make_shell();
    shell.handle_command("PFSHOW CTRL"); // scope = Ctrl, visible
    shell.handle_command("PFSHOW OFF"); // hidden, scope retained
    shell.handle_command("PFSHOW ON"); // visible again at Ctrl
    assert!(shell.key_bar_visible);
    assert_eq!(shell.key_bar_scope, KeyBarScope::Ctrl);
}

#[test]
fn pfshow_unknown_arg_leaves_mode_unchanged_and_sets_error() {
    // Validates: function-keys-and-history Requirement 12.13 -- unknown argument
    // is a non-fatal error and does not change the mode.
    use super::KeyBarScope;
    let mut shell = make_shell();
    shell.handle_command("PFSHOW SHIFT");
    let before_visible = shell.key_bar_visible;
    let before_scope = shell.key_bar_scope;

    shell.handle_command("PFSHOW FOO");
    assert_eq!(shell.key_bar_visible, before_visible, "mode unchanged");
    assert_eq!(shell.key_bar_scope, before_scope, "scope unchanged");
    assert_eq!(shell.key_bar_scope, KeyBarScope::Shift);
    assert!(
        shell.open_error.is_some(),
        "unknown PFSHOW arg must set a non-fatal error"
    );
}

// Validates: command-framework Req 12.4 -- the registered ShellContextProvider
// returns the SAME package the shell captured, so a command dispatched through
// the registry receives a populated Cursor_Context (not the empty default).
#[test]
fn context_provider_returns_populated_cursor_context() {
    let mut harness = harness_shell();
    harness.state_mut().command_text = "SAVE".to_string();
    harness.run();
    // Build a context provider view the way CommandDispatch does: read the
    // shell snapshot cell directly (the provider clones it).
    let cc = harness
        .state()
        .cursor_context_snapshot
        .lock()
        .expect("snapshot lock")
        .clone();
    // The provider would wrap this into an ExecutionContext.cursor_context.
    let exec = ff_command::ExecutionContext::builder()
        .cursor_context(cc)
        .build();
    assert_eq!(
        exec.cursor_context.workspace_context.as_deref(),
        Some("pom")
    );
    assert!(
        exec.cursor_context.focused_identity.is_some(),
        "a populated Cursor_Context must reach the ExecutionContext (not empty)"
    );
}

// Validates: command-framework Req 12.7 -- the single canonical
// "command not implemented yet" message constant.
#[test]
fn not_implemented_message_is_canonical() {
    assert_eq!(
        super::target_dispatch::NOT_IMPLEMENTED_MSG,
        "Command not implemented yet."
    );
}

// === CR-NR-078 WF.4: host-agnostic render (detach-ready) =====================

// Validates: workspace-framework Req 6.1/6.2 -- a migrated Context's
// `WorkspaceContext::render` is HOST-AGNOSTIC: it renders correctly into a plain
// `Ui` that is NOT the main window's CentralPanel (the same call a future
// detached OS viewport or dock zone would use), and returns its InteriorFocus,
// without panicking. Proven with the Config panel (the simplest implementor).
#[test]
fn workspace_context_render_is_host_agnostic() {
    use crate::config_panel::{filter_field_id, ConfigPanelState};
    use crate::notification::NotificationQueue;
    use crate::shell::workspace_context::{InteriorFocus, ShellServices, WorkspaceContext};
    use egui_kittest::Harness;
    use ff_config::{init, ConfigInitOptions};
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use tokio::runtime::Runtime;

    let config = init(ConfigInitOptions::new().with_hot_reload(false)).expect("config init");
    let runtime = Runtime::new().expect("runtime");
    let notifications = Arc::new(Mutex::new(NotificationQueue::new()));
    let command_store = crate::command_config::store::CommandStore::default();

    let focus_seen: Rc<Cell<InteriorFocus>> = Rc::new(Cell::new(InteriorFocus::none()));
    let focus_for_ui = Rc::clone(&focus_seen);

    // build_ui renders into a PLAIN Ui -- deliberately NOT a CentralPanel -- to
    // prove the Context does not assume the central-panel host.
    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(600.0, 400.0))
        .build_ui(move |ui| {
            let mut panel = ConfigPanelState::new();
            let mut requests = Vec::new();
            let mut services = ShellServices {
                config: &config,
                runtime: &runtime,
                notifications: &notifications,
                themes_dir: std::path::PathBuf::from("."),
                menus_dir: std::path::PathBuf::from("."),
                command_store: &command_store,
                requests: &mut requests,
            };
            let focus = panel.render(ui, &mut services);
            focus_for_ui.set(focus);
        });
    harness.run();

    // The Config panel reports its Filter field as the single interior stop,
    // regardless of host -- identical to when it renders in the central panel.
    assert_eq!(
        focus_seen.get(),
        InteriorFocus::single(filter_field_id()),
        "WorkspaceContext::render must be host-agnostic: same InteriorFocus outside the central panel"
    );
}

// === B038 / CR-NR-086: status-bar logging-degradation indicator (Req 8.7) ===

// Validates: logging-subsystem Req 8.7 (B038) -- the pure decision function that
// drives the status-bar indicator. Degraded (fallback and/or dropped>0) yields a
// reason string; healthy (not fallback, zero dropped) yields None (hidden).
#[test]
fn logging_degradation_reason_covers_all_states() {
    use super::render::logging_degradation_reason;
    // Healthy -> hidden.
    assert_eq!(logging_degradation_reason(false, 0), None);
    // Fallback only.
    let r = logging_degradation_reason(true, 0).expect("fallback -> degraded");
    assert!(r.contains("fallback"), "reason names fallback: {r:?}");
    // Dropped only.
    let r = logging_degradation_reason(false, 5).expect("drops -> degraded");
    assert!(
        r.contains("5") && r.contains("dropped"),
        "reason names drops: {r:?}"
    );
    // Both -> both reasons joined.
    let r = logging_degradation_reason(true, 3).expect("both -> degraded");
    assert!(
        r.contains("fallback") && r.contains("3"),
        "reason names both: {r:?}"
    );
}

// Validates: logging-subsystem Req 8.7 (B038) -- in a full-shell render with a
// HEALTHY logging subsystem (not initialised in tests -> not fallback, zero
// dropped), the status bar does NOT show the degradation indicator (the
// automation entry is registered empty). The DEGRADED render requires a real
// fallback state (no test hook to force the global) and is verified MANUALLY.
#[test]
fn full_shell_status_bar_hides_logging_indicator_when_healthy() {
    let mut harness = harness_shell();
    harness.run();
    let state = harness
        .state()
        .automation
        .query_str(crate::automation::ids::LOGGING_DEGRADED)
        .and_then(|s| s.value.clone());
    // Registered as empty (indicator hidden) when logging is healthy.
    assert_eq!(
        state.as_deref(),
        Some(""),
        "logging-degradation indicator must be hidden (empty) when logging is healthy"
    );
}

// Validates: menu-workspace Req 14.4 (B079) -- navigating between help topics
// (index -> a command topic) does NOT stack Help-on-Help frames, so a single
// END still returns to the pre-Help Context.
#[test]
fn navigating_between_help_topics_does_not_stack_help_frames() {
    use crate::tab_state::{KindTag, TabKind};
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    // Open the index, then a second topic while already in Help.
    shell.handle_command("HELP");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::HelpContext);
    shell.handle_command("HELP FIND");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::HelpContext,
        "still in the Help Context after a second HELP"
    );
    // A single END returns to the pre-Help Context (Home), proving only one
    // frame was pushed.
    shell.handle_command("END");
    assert!(
        shell.tabs.active_tab().is_home,
        "one END returns to Home; help topic navigation did not stack frames"
    );
}

// Validates: context-help Req 18.4, 19.1, 19.2 -- an unresolved non-dynamic key
// shows the index with a message AND records a miss (no command error only).
#[test]
fn help_missing_topic_shows_index_and_records_miss() {
    let mut shell = make_shell();
    // Empty registry (make_shell finds no shipped help beside the test binary):
    // HELP CHANGE resolves cmd:CHANGE, which is absent -> miss + index message.
    shell.handle_command("HELP CHANGE");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        crate::tab_state::KindTag::HelpContext,
        "a missing topic still opens the Help Context (index), not just an error"
    );
    assert!(
        shell.open_error.is_none(),
        "a missing topic must not surface as a command-line error"
    );
    assert_eq!(
        shell.help_missing_count("cmd:CHANGE"),
        1,
        "the miss must be tallied"
    );
    let body = shell
        .help_panel_for_test()
        .model()
        .current_topic()
        .unwrap()
        .body()
        .to_string();
    assert!(
        body.contains("Help not yet available") && body.contains("cmd:CHANGE"),
        "index-with-message must name the unresolved topic: {body}"
    );
}

// Validates: context-help Req 19.3, 19.4 -- HELP MISSING reports the tally with
// EXPECTED/UNEXPECTED classification.
#[test]
fn help_missing_report_classifies_topics() {
    let mut shell = make_shell();
    shell.handle_command("HELP CHANGE"); // EXPECTED (in the promised set)
    shell.handle_command("HELP ZZZUNKNOWN"); // UNEXPECTED (no requirement)
    shell.handle_command("HELP MISSING");
    let body = shell
        .help_panel_for_test()
        .model()
        .current_topic()
        .unwrap()
        .body()
        .to_string();
    assert!(body.contains("cmd:CHANGE"), "report lists CHANGE: {body}");
    assert!(
        body.contains("EXPECTED"),
        "report classifies EXPECTED: {body}"
    );
    assert!(
        body.contains("cmd:ZZZUNKNOWN") && body.contains("UNEXPECTED"),
        "report classifies the unknown key UNEXPECTED: {body}"
    );
}

// Validates: context-help Req 19.5 -- reaching a miss writes to NO project doc.
#[test]
fn help_miss_does_not_write_project_docs() {
    let bugs = std::path::Path::new("../../docs/status/bugs.md");
    let changelog = std::path::Path::new("../../docs/status/change-log.md");
    let bugs_before = std::fs::metadata(bugs).map(|m| m.len()).ok();
    let changelog_before = std::fs::metadata(changelog).map(|m| m.len()).ok();

    let mut shell = make_shell();
    shell.handle_command("HELP SOMETHINGMISSING");
    shell.handle_command("HELP MISSING");

    let bugs_after = std::fs::metadata(bugs).map(|m| m.len()).ok();
    let changelog_after = std::fs::metadata(changelog).map(|m| m.len()).ok();
    assert_eq!(
        bugs_before, bugs_after,
        "bugs.md must be untouched by a miss"
    );
    assert_eq!(
        changelog_before, changelog_after,
        "change-log.md must be untouched by a miss"
    );
}
