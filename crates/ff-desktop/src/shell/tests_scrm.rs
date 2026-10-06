//! Shell tests -- scrm area (split from shell/tests.rs, CR F3).
//! Items are verbatim; only their file location changed.

#![allow(unused_imports)]
use super::tests_common::*;
use ff_keys::{KeyMap, ModifiedKey};
use std::sync::{Arc, Mutex};

use ff_command::{
    CommandDispatch, CommandError, CommandHandler, CommandHistory, CommandId, CommandMetadata,
    CommandParams, CommandRegistry, CommandResult, ExecutionContext,
};

/// Validates: screen-snapshot-scrm Req 4.4 -- a format argument selects the
/// renderer (MARKDOWN produces a fenced code block / heading).
#[test]
fn snapshot_text_for_active_markdown_format_arg() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    let (text, format) = shell
        .snapshot_text_for_active("MARKDOWN")
        .expect("POM is capturable");
    assert_eq!(format, ff_screen_model::SnapshotFormat::Markdown);
    assert!(text.contains("```"), "markdown must contain a fenced block");
}

/// Validates: screen-snapshot-scrm Req 6.3 -- an unknown format argument is an
/// error, not a panic or a silent no-op.
#[test]
fn snapshot_unknown_format_is_error() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    let result = shell.snapshot_text_for_active("bogus");
    assert!(result.is_err(), "unknown format must be an error");
}

// === CR-NR-098 Wave 2: CAPTURE lifecycle + auto-capture + SCRM viewer =======

/// Validates: screen-snapshot-scrm Req 7.1, 7.7 -- CAPTURE START opens a
/// collection and CAPTURE STATUS reports it.
#[test]
fn capture_start_then_status_reports_active_collection() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE START Repro");
    shell.handle_command("CAPTURE STATUS");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(
        msg.contains("Repro"),
        "STATUS must name the collection; got: {msg:?}"
    );
}

/// Validates: screen-snapshot-scrm Req 8.1, 8.2 -- CAPTURE SCREEN appends a
/// capture of the active (POM) Context.
#[test]
fn capture_screen_appends_capture() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE START C");
    shell.handle_command("CAPTURE SCREEN");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("Captured screen 1"), "got: {msg:?}");
    shell.handle_command("CAPTURE LIST");
    let listing = shell.open_error.as_deref().unwrap_or("");
    assert!(
        listing.contains("1."),
        "LIST enumerates the capture; got: {listing:?}"
    );
}

/// Validates: screen-snapshot-scrm Req 9.1, 9.5 -- with automatic capture on
/// (CAPTURE START enables it), navigating the Home Context to another Context
/// records a capture at the transition choke point (navigate_to).
#[test]
fn auto_capture_records_on_navigation() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    // START enables auto-capture and captures nothing yet by itself.
    shell.handle_command("CAPTURE START Flow");
    // Navigate the POM to another Context: the transition hook fires a capture.
    shell.dispatch_command_string("FILES");
    shell.handle_command("CAPTURE STATUS");
    let msg = shell.open_error.as_deref().unwrap_or("");
    // At least one capture was recorded automatically on the transition.
    assert!(
        msg.contains("capture(s)") && !msg.contains("0 capture(s)"),
        "auto-capture must record on navigation; status: {msg:?}"
    );
}

/// Validates: screen-snapshot-scrm Req 10.1, 16.1 -- CAPTURE REPLAY with no
/// collection reports nothing to replay (does not open an empty viewer).
#[test]
fn capture_replay_without_collection_reports_nothing() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE REPLAY");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("No screen collection"), "got: {msg:?}");
}

/// Validates: screen-snapshot-scrm Req 10.1, 16.1 -- CAPTURE REPLAY navigates to
/// the SCRM viewer Context when a collection exists.
#[test]
fn capture_replay_opens_viewer_when_collection_exists() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE START C");
    shell.handle_command("CAPTURE SCREEN");
    shell.handle_command("CAPTURE REPLAY");
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::ScrmViewer,
        "CAPTURE REPLAY opens the SCRM viewer Context"
    );
}

/// Validates: screen-snapshot-scrm Req 12.1, 11.1 -- CAPTURE EXPORT TEXT writes
/// a text file under the screen-collections dir.
#[test]
fn capture_export_text_writes_file() {
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE EXPORT TEXT flow.txt");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("exported"), "got: {msg:?}");
    let path = dir.path().join("flow.txt");
    assert!(
        path.exists(),
        "export file must exist at {}",
        path.display()
    );
    let body = std::fs::read_to_string(&path).expect("read export");
    assert!(
        body.contains("Repro"),
        "export contains the collection name"
    );
}

/// Validates: screen-snapshot-scrm Req 12.2 -- CAPTURE EXPORT MD writes markdown.
#[test]
fn capture_export_markdown_writes_file() {
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE EXPORT MD flow.md");
    let path = dir.path().join("flow.md");
    assert!(path.exists());
    let body = std::fs::read_to_string(&path).expect("read export");
    assert!(
        body.contains("# Repro"),
        "markdown has the collection heading"
    );
}

/// Validates: screen-snapshot-scrm Req 12.5, 11.1 -- CAPTURE SAVE writes the
/// native archive and CAPTURE LOAD reads it back into the active session.
#[test]
fn capture_save_then_load_round_trips_collection() {
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE SAVE repro.ffscrm");
    let path = dir.path().join("repro.ffscrm");
    assert!(path.exists(), "archive must exist at {}", path.display());
    // Purge the active collection, then LOAD it back.
    shell.handle_command("CAPTURE PURGE");
    assert!(!shell.scrm.is_active(), "purged");
    shell.handle_command("CAPTURE LOAD repro.ffscrm");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("Loaded collection"), "got: {msg:?}");
    assert!(
        shell.scrm.is_active(),
        "collection reloaded into the session"
    );
}

/// Validates: screen-snapshot-scrm Req 12.4, 12.6 -- CAPTURE EXPORT PDF writes a
/// real PDF file with selectable text (a %PDF header and a Tj text operator),
/// not a rasterised image.
#[test]
fn capture_export_pdf_writes_selectable_pdf() {
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE EXPORT PDF out.pdf");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("exported"), "got: {msg:?}");
    let path = dir.path().join("out.pdf");
    assert!(path.exists(), "PDF file must exist at {}", path.display());
    let bytes = std::fs::read(&path).expect("read pdf");
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(16)]);
    assert!(head.starts_with("%PDF-"), "real PDF header; got: {head:?}");
    let full = String::from_utf8_lossy(&bytes);
    assert!(
        full.contains(" Tj"),
        "PDF has selectable text operators (not raster)"
    );
}

/// Validates: screen-snapshot-scrm Req 11.1 -- CAPTURE EXPORT with no active
/// collection reports the empty state (no panic, no file).
#[test]
fn capture_export_without_collection_reports_empty() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE EXPORT TEXT");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("No active screen collection"), "got: {msg:?}");
}

/// Validates: screen-snapshot-scrm Req 14.1-14.4, 20.4 -- CAPTURE EVIDENCE
/// writes an evidence package JSON with the test-case id, pass/fail status, and
/// a content hash.
#[test]
fn capture_evidence_writes_package_with_hash() {
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE EVIDENCE TC-42 PASS ev.json");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("Evidence package written"), "got: {msg:?}");
    let path = dir.path().join("ev.json");
    assert!(
        path.exists(),
        "evidence file must exist at {}",
        path.display()
    );
    let body = std::fs::read_to_string(&path).expect("read evidence");
    assert!(body.contains("TC-42"), "test-case id recorded");
    assert!(body.contains("\"pass\""), "pass status recorded");
    assert!(
        body.contains("sha256:"),
        "content hash recorded (tamper-evidence)"
    );
}

/// Validates: screen-snapshot-scrm Req 14.1 -- CAPTURE EVIDENCE with no active
/// collection reports the empty state.
#[test]
fn capture_evidence_without_collection_reports_empty() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE EVIDENCE");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("No active screen collection"), "got: {msg:?}");
}

/// Validates: screen-snapshot-scrm Req 20.1, 20.2 -- CAPTURE EXPORT PDF
/// PROTECTED writes an encrypted (edit-locked) PDF file.
#[test]
fn capture_export_pdf_protected_writes_encrypted_pdf() {
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE EXPORT PDF PROTECTED my-owner-pw locked.pdf");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("Protected PDF written"), "got: {msg:?}");
    let path = dir.path().join("locked.pdf");
    assert!(
        path.exists(),
        "protected PDF must exist at {}",
        path.display()
    );
    let bytes = std::fs::read(&path).expect("read pdf");
    assert!(bytes.starts_with(b"%PDF-"), "valid PDF header");
    // The document is encrypted: it carries an /Encrypt reference in the trailer.
    let full = String::from_utf8_lossy(&bytes);
    assert!(
        full.contains("/Encrypt"),
        "protected PDF carries an /Encrypt dict"
    );
}

/// Validates: screen-snapshot-scrm Req 20.2 -- CAPTURE EXPORT PDF PROTECTED with
/// no owner password uses a default (still produces an encrypted file).
#[test]
fn capture_export_pdf_protected_default_owner_password() {
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE EXPORT PDF PROTECTED");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("Protected PDF written"), "got: {msg:?}");
    // Default file name is <stem>-protected.pdf under the scrm dir.
    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".pdf"))
        .collect();
    assert!(!entries.is_empty(), "a protected PDF file was written");
}
