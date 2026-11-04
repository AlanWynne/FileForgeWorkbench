//! `Last:` stamp tests (CR-NR-115, workspace-framework Requirement 7.16): the
//! segment is the time of the most recent action OR screen transition,
//! initialised to the startup time, and never `--:--`. Deterministic: the stamp
//! is pinned to a fixed past instant and each transition must move it forward.

use super::tests_common::*;
use ff_session::session_state::WorkspaceDescriptor;

fn pinned_past() -> chrono::DateTime<chrono::Local> {
    use chrono::TimeZone;
    chrono::Local
        .with_ymd_and_hms(2000, 1, 1, 0, 0, 0)
        .single()
        .expect("fixed local time")
}

fn startup_label(shell: &super::WorkbenchShell) -> String {
    format!("Last: {}", shell.session_start.format("%H:%M"))
}

// Validates: workspace-framework Requirement 7.16 -- a freshly constructed
// shell shows the startup HH:MM, never `--:--`.
#[test]
fn fresh_shell_last_segment_is_startup_time_never_placeholder() {
    let shell = make_shell();
    assert_eq!(shell.format_last_action(), startup_label(&shell));
    assert_ne!(shell.format_last_action(), "Last: --:--");
}

// Validates: workspace-framework Requirement 7.16 -- an active-tab change is a
// screen transition and restamps `Last:`; an unchanged tab does not.
#[test]
fn tab_switch_updates_last_action_stamp() {
    let mut shell = make_shell();
    shell.handle_command("START");
    assert!(shell.tabs.len() >= 2, "precondition: two tabs");
    shell.sync_active_tab_transition();
    shell.last_action_at = pinned_past();
    assert!(!shell.sync_active_tab_transition(), "no change, no stamp");
    assert_eq!(shell.last_action_at, pinned_past());
    let other = if shell.tabs.active_index() == 0 { 1 } else { 0 };
    shell.tabs.set_active(other);
    assert!(shell.sync_active_tab_transition(), "tab change is detected");
    assert!(shell.last_action_at > pinned_past());
}

// Validates: workspace-framework Requirement 7.16 -- closing the active tab
// (focus moves to another tab) is a screen transition.
#[test]
fn tab_close_updates_last_action_stamp() {
    let mut shell = make_shell();
    shell.handle_command("START");
    shell.sync_active_tab_transition();
    shell.last_action_at = pinned_past();
    let idx = shell.tabs.active_index();
    shell.tabs.close_tab(idx);
    assert!(shell.sync_active_tab_transition());
    assert!(shell.last_action_at > pinned_past());
}

// Validates: workspace-framework Requirement 7.16 -- navigate_to PUSH and the
// END pop both stamp `Last:` through the shared reconstruct seam.
#[test]
fn navigate_push_and_end_pop_update_last_action_stamp() {
    let mut shell = make_shell();
    shell.handle_command("START");
    shell.last_action_at = pinned_past();
    shell.navigate_to(
        WorkspaceDescriptor::Menu {
            name: "settings".to_string(),
        },
        true,
    );
    assert!(shell.last_action_at > pinned_past(), "push must stamp");
    assert!(
        !shell.tabs.active_tab().nav_stack.is_empty(),
        "precondition: a level to pop"
    );
    shell.last_action_at = pinned_past();
    shell.nav_end();
    assert!(shell.last_action_at > pinned_past(), "END pop must stamp");
}

// Validates: workspace-framework Requirement 7.16 -- RETURN collapse stamps.
#[test]
fn return_collapse_updates_last_action_stamp() {
    let mut shell = make_shell();
    shell.handle_command("START");
    shell.navigate_to(
        WorkspaceDescriptor::Menu {
            name: "settings".to_string(),
        },
        true,
    );
    shell.last_action_at = pinned_past();
    shell.nav_return();
    assert!(shell.last_action_at > pinned_past(), "RETURN must stamp");
}

// Validates: workspace-framework Requirement 7.16 and screen-snapshot-scrm
// Requirement 30.10 -- a snapshot right after startup shows the startup time in
// the captured status band.
#[test]
fn snapshot_after_startup_shows_startup_time() {
    super::i18n_bootstrap::install_catalogue_for_tests();
    ff_session::set_active_profile(None);
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    let expected = startup_label(&shell);
    shell.run_command_line("SNAPSHOT");
    let text = shell
        .snapshot
        .pending
        .as_ref()
        .expect("SNAPSHOT stashes a pending request")
        .text
        .clone();
    assert!(text.contains(&expected), "{expected} missing:\n{text}");
    assert!(!text.contains("--:--"), "{text}");
}
