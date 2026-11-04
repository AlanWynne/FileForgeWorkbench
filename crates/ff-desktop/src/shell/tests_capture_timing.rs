//! Capture-timing tests (CR-NR-115 follow-up, workspace-framework Requirement
//! 7.29, screen-snapshot-scrm Requirement 30.28): a SNAPSHOT/CAPTURE reproduces
//! the frame AS IT WAS WHEN THE COMMAND WAS ISSUED. These drive the REAL command
//! seams (`run_command_line`, `dispatch_key_command`), not a helper.

use super::message_routing::SnapshotPlaceOutcome;
use super::tests_common::*;

const CONFIRMATION: &str = "Snapshot copied to clipboard: picture + Markdown.";

fn old_stamp() -> chrono::DateTime<chrono::Local> {
    use chrono::TimeZone;
    chrono::Local
        .with_ymd_and_hms(2026, 11, 4, 4, 17, 0)
        .single()
        .expect("fixed local time")
}

fn pom_shell() -> super::WorkbenchShell {
    super::i18n_bootstrap::install_catalogue_for_tests();
    ff_session::set_active_profile(None);
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.last_action_at = old_stamp();
    shell
}

fn pending_text(shell: &super::WorkbenchShell) -> String {
    shell
        .snapshot
        .pending
        .as_ref()
        .expect("SNAPSHOT stashes a pending request")
        .text
        .clone()
}

// Validates: workspace-framework Requirement 7.29 -- the capture shows the
// message band that was present BEFORE the snapshot command, the Last time as it
// was before the action, and the command field WITHOUT the triggering command.
#[test]
fn snapshot_captures_message_present_before_the_command() {
    let mut shell = pom_shell();
    shell.message_line = vec!["previous action message".to_string()];
    shell.command_text = "SNAPSHOT".to_string();
    shell.run_command_line("SNAPSHOT");
    let text = pending_text(&shell);
    assert!(text.contains("previous action message"), "{text}");
    assert!(
        text.contains("Last: 04:17"),
        "pre-action Last time:\n{text}"
    );
    assert!(!text.contains("Command ===> SNAPSHOT"), "{text}");
    assert!(shell.message_line.is_empty(), "the action cleared the band");
}

// Validates: workspace-framework Requirement 7.29 -- the FIRST snapshot does not
// contain its own confirmation, and the confirmation then appears live.
#[test]
fn first_snapshot_has_no_confirmation_then_live_band_shows_it() {
    let mut shell = pom_shell();
    shell.run_command_line("SNAPSHOT");
    assert!(!pending_text(&shell).contains(CONFIRMATION));
    shell.post_snapshot_place_outcome(SnapshotPlaceOutcome::Both, "Markdown");
    assert_eq!(shell.message_line, vec![CONFIRMATION.to_string()]);
}

// Validates: workspace-framework Requirement 7.29 -- the SECOND snapshot shows
// the first snapshot's confirmation in its message band, and then the band is
// cleared by that action until the second confirmation is posted.
#[test]
fn second_snapshot_contains_first_snapshots_confirmation() {
    let mut shell = pom_shell();
    shell.run_command_line("SNAPSHOT");
    shell.post_snapshot_place_outcome(SnapshotPlaceOutcome::Both, "Markdown");
    shell.snapshot.pending = None;
    shell.run_command_line("SNAPSHOT");
    assert!(pending_text(&shell).contains(CONFIRMATION));
    assert!(shell.message_line.is_empty());
    shell.post_snapshot_place_outcome(SnapshotPlaceOutcome::Both, "Markdown");
    assert_eq!(shell.message_line, vec![CONFIRMATION.to_string()]);
}

// Validates: workspace-framework Requirement 7.29 -- the key-forward path calls
// note_action_dispatched TWICE (begin_command_line, then dispatch_bound_command);
// the second call must not overwrite the pre-action state.
#[test]
fn key_forward_double_invoke_keeps_pre_action_state() {
    let mut shell = pom_shell();
    shell.message_line = vec!["before key press".to_string()];
    shell.dispatch_key_command("SNAPSHOT");
    assert!(pending_text(&shell).contains("before key press"));
}

fn command_line_of(text: &str) -> &str {
    text.lines()
        .find(|l| l.contains("Command ===>"))
        .unwrap_or_else(|| panic!("no Command ===> line:\n{text}"))
}

// Validates: screen-snapshot-scrm Requirement 30.24(b) / workspace-framework
// Requirement 7.29 -- a NON-EMPTY field typed BEFORE the snapshot is preserved in
// the capture (the key-forward path: begin_command_line stashes the pre-action
// frame while the field still holds the user's text, then SNAPSHOT runs).
#[test]
fn key_forward_snapshot_preserves_non_empty_field_typed_before() {
    let mut shell = pom_shell();
    shell.command_text = "FOO".to_string();
    // The seam dispatch_key_command runs before dispatching the bound command.
    // A merged `SNAPSHOT FOO` would be a format argument, so drive the bare
    // handler directly after the shared seam to model a bare key-bound SNAPSHOT.
    shell.begin_command_line();
    shell.handle_snapshot("");
    let text = pending_text(&shell);
    let line = command_line_of(&text);
    assert!(line.contains("Command ===> FOO_"), "{line}");
    assert!(!line.contains("SNAPSHOT"), "{line}");
}

// Validates: screen-snapshot-scrm Requirement 30.24(b) -- the TYPED command
// itself is the triggering command: the field "before" it was empty, so the
// capture shows underscores only and never echoes SNAPSHOT.
#[test]
fn typed_snapshot_command_is_excluded_from_the_command_line() {
    let mut shell = pom_shell();
    shell.command_text = "SNAPSHOT".to_string();
    shell.run_command_line("SNAPSHOT");
    let text = pending_text(&shell);
    let line = command_line_of(&text);
    assert!(line.contains("Command ===> _"), "{line}");
    assert!(!line.contains("SNAPSHOT"), "{line}");
}

// Validates: workspace-framework Requirement 7.29 -- a capture taken OUTSIDE an
// action uses the live state, and the pre-action stash does not outlive its
// action.
#[test]
fn capture_outside_an_action_uses_live_state() {
    let mut shell = pom_shell();
    shell.message_line = vec!["old".to_string()];
    shell.run_command_line("SNAPSHOT");
    shell.message_line = vec!["live now".to_string()];
    let (text, _) = shell.snapshot_text_for_active("").expect("capturable");
    assert!(text.contains("live now"), "{text}");
    assert!(!text.contains("old"), "{text}");
}
