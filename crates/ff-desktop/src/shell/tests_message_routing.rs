//! Message routing tests (CR-NR-115 follow-up, workspace-framework
//! Requirement 7.28): INFORMATIONAL / confirmation messages go to the
//! Message_Line band, ERRORS stay on the Status_Bar error slot (`open_error`).

use super::message_routing::SnapshotPlaceOutcome;
use super::tests_common::*;

const CONFIRMATION: &str = "Snapshot copied to clipboard: picture + Markdown.";

fn shell() -> super::WorkbenchShell {
    super::i18n_bootstrap::install_catalogue_for_tests();
    make_shell()
}

// Validates: workspace-framework Requirement 7.28 -- after a snapshot is placed
// the confirmation lands in message_line (the message band) and NOT in the
// status bar error slot.
#[test]
fn snapshot_confirmation_goes_to_message_line_not_status_bar() {
    let mut shell = shell();
    shell.post_snapshot_place_outcome(SnapshotPlaceOutcome::Both, "Markdown");
    assert_eq!(shell.message_line, vec![CONFIRMATION.to_string()]);
    assert!(shell.open_error.is_none(), "status bar slot stays empty");
}

// Validates: workspace-framework Requirement 7.28 -- a clipboard FAILURE is an
// error: it stays on the status bar error slot and is not dropped.
#[test]
fn snapshot_clipboard_failure_stays_an_error() {
    let mut shell = shell();
    shell.post_snapshot_place_outcome(SnapshotPlaceOutcome::Failed, "Markdown");
    assert!(shell.open_error.is_some());
    assert!(shell.message_line.is_empty());
}

// Validates: workspace-framework Requirement 7.28 -- a successful informational
// command (TIME) reports through the message band, not open_error.
#[test]
fn info_command_result_goes_to_message_line_not_open_error() {
    let mut shell = shell();
    shell.run_command_line("TIME");
    assert_eq!(shell.message_line.len(), 1, "{:?}", shell.message_line);
    assert!(shell.open_error.is_none());
}

// Validates: workspace-framework Requirement 7.28 -- bad-argument errors stay on
// open_error and leave the message band empty.
#[test]
fn error_messages_stay_on_open_error() {
    for cmd in ["SNAPSHOT bogus", "PFSHOW bogus"] {
        let mut shell = shell();
        shell.run_command_line(cmd);
        assert!(shell.open_error.is_some(), "{cmd}");
        assert!(shell.message_line.is_empty(), "{cmd}");
    }
}

// Validates: workspace-framework Requirement 7.28 and command-framework
// Requirement 13.1, 13.2 -- a SUCCESSFUL info command clears the command field
// (it used to be restored because the info text sat in open_error), while a
// failed command still restores it for correction.
#[test]
fn info_message_clears_the_field_and_error_restores_it() {
    let mut shell = shell();
    shell.command_text = "TIME".to_string();
    shell.run_command_line("TIME");
    assert!(shell.command_text.is_empty());

    shell.command_text = "PFSHOW bogus".to_string();
    shell.run_command_line("PFSHOW bogus");
    assert_eq!(shell.command_text, "PFSHOW bogus");
}

// Validates: workspace-framework Requirement 7.28 -- CAPTURE SAVE/EXPORT/EVIDENCE
// SUCCESS confirmations go to the message band, not the status bar error slot.
#[test]
fn capture_file_success_goes_to_message_line_not_open_error() {
    for cmd in [
        "CAPTURE SAVE s.ffscrm",
        "CAPTURE EXPORT TEXT e.txt",
        "CAPTURE EVIDENCE TC-1 PASS ev.json",
    ] {
        let (mut shell, _dir) = make_shell_with_scrm_dir();
        // The fixture's own setup already posted earlier messages, so compare
        // the band before and after the command.
        let before = shell.message_line.len();
        shell.handle_command(cmd);
        let after = &shell.message_line;
        assert!(after.len() > before, "{cmd}: {after:?}");
        assert!(shell.open_error.is_none(), "{cmd}: {:?}", shell.open_error);
    }
}

// Validates: workspace-framework Requirement 7.28 -- CAPTURE SAVE/EXPORT/EVIDENCE/
// LOAD FAILURES stay on the error slot and leave the message band empty.
#[test]
fn capture_file_failure_stays_on_open_error() {
    for cmd in [
        "CAPTURE SAVE s.ffscrm",
        "CAPTURE EXPORT TEXT e.txt",
        "CAPTURE EVIDENCE TC-1 PASS ev.json",
        "CAPTURE LOAD",
    ] {
        let mut shell = shell();
        shell.handle_command(cmd);
        assert!(shell.open_error.is_some(), "{cmd}");
        assert!(shell.message_line.is_empty(), "{cmd}");
    }
}

// Validates: workspace-framework Requirement 7.12, 7.14 -- a message survives
// until the next action, which clears it.
#[test]
fn message_clears_on_the_next_action() {
    let mut shell = shell();
    shell.post_info("first");
    assert_eq!(shell.message_line, vec!["first".to_string()]);
    shell.run_command_line("PFSHOW bogus");
    assert!(shell.message_line.is_empty());
}
