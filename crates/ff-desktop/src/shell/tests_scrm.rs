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

/// Validates: screen-snapshot-scrm Req 9.10, 9.11 -- a command-caused
/// auto-capture records the triggering command (canonical string + resolved
/// target kind), taken from the single dispatch seam (no bespoke intercept).
#[test]
fn auto_capture_records_triggering_command_from_dispatch_seam() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE START Flow");
    // Dispatch FILES through the SINGLE front door: resolve_target classifies it
    // as a CustomWorkspace and navigate_to fires the departing-Context capture.
    shell.dispatch_command_string("FILES");
    let collection = shell
        .scrm
        .active_collection()
        .expect("collection active after START");
    let cap = collection
        .screens
        .last()
        .expect("a capture was recorded on the transition");
    let tc = cap
        .triggering_command
        .as_ref()
        .expect("the command-caused capture carries a triggering command");
    assert_eq!(tc.command, "FILES");
    assert_eq!(tc.target_kind, "custom_workspace");
}

/// Validates: screen-snapshot-scrm Req 9.12 -- a non-command (interval/rule)
/// capture records NO triggering command: CAPTURE SCREEN captures with no
/// dispatched command staged, so the triggering command is explicitly absent.
#[test]
fn manual_capture_screen_has_absent_triggering_command() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE START C");
    // CAPTURE SCREEN is not a dispatched-command transition; it must not pick up
    // a stale triggering command.
    shell.handle_command("CAPTURE SCREEN");
    let cap = shell
        .scrm
        .active_collection()
        .expect("active")
        .screens
        .last()
        .expect("captured");
    assert!(
        cap.triggering_command.is_none(),
        "a manual/interval capture records an absent triggering command"
    );
}

/// Validates: screen-snapshot-scrm Req 9.12 -- after a command-caused capture
/// consumes the staged command, a SUBSEQUENT interval-style capture does not
/// inherit the previous command (the stash is read-and-cleared).
#[test]
fn staged_command_is_consumed_and_not_reused() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE START Flow");
    shell.dispatch_command_string("FILES");
    // A manual CAPTURE SCREEN afterwards must NOT carry the consumed FILES.
    shell.handle_command("CAPTURE SCREEN");
    let cap = shell
        .scrm
        .active_collection()
        .expect("active")
        .screens
        .last()
        .expect("captured");
    assert!(
        cap.triggering_command.is_none(),
        "the staged command must be cleared after it is consumed once"
    );
}

/// Validates: screen-snapshot-scrm Req 10.12 -- bare CAPTURE REPLAY opens the
/// saved-session picker modal (Stream SCRM Phase 2C FEAT-003 changed the bare
/// form from "open the in-memory viewer" to "pick a saved session"). With no
/// active collection it still opens the picker (it lists saved files on disk,
/// not the in-memory session) and reports no error.
#[test]
fn capture_replay_bare_without_collection_opens_picker() {
    // Validates: Requirement 10.12
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE REPLAY");
    assert!(
        shell.scrm_ui.replay_picker.is_some(),
        "bare CAPTURE REPLAY opens the saved-session picker"
    );
    assert!(
        shell.open_error.is_none(),
        "opening the picker is not an error"
    );
}

/// Validates: screen-snapshot-scrm Req 10.12 -- bare CAPTURE REPLAY opens the
/// picker even when a collection is active (the user chooses a SAVED session,
/// not only the in-memory one).
#[test]
fn capture_replay_bare_with_collection_opens_picker() {
    // Validates: Requirement 10.12
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE START C");
    shell.handle_command("CAPTURE SCREEN");
    shell.handle_command("CAPTURE REPLAY");
    assert!(
        shell.scrm_ui.replay_picker.is_some(),
        "bare CAPTURE REPLAY opens the picker"
    );
}

/// Validates: screen-snapshot-scrm Req 10.11, 16.1 -- CAPTURE REPLAY <file>
/// loads the saved .ffscrm archive then opens the SCRM viewer Context on the
/// loaded Collection.
#[test]
fn capture_replay_file_loads_then_opens_viewer() {
    // Validates: Requirement 10.11
    use crate::tab_state::KindTag;
    let (mut shell, dir) = make_shell_with_scrm_dir();
    // Save the seeded collection, then purge the in-memory one so REPLAY <file>
    // genuinely loads from disk.
    shell.handle_command("CAPTURE SAVE repro.ffscrm");
    shell.handle_command("CAPTURE PURGE");
    assert!(!shell.scrm.is_active(), "purged before replay");
    let path = dir.path().join("repro.ffscrm");
    shell.handle_command(&format!("CAPTURE REPLAY {}", path.display()));
    assert!(
        shell.scrm.is_active(),
        "CAPTURE REPLAY <file> loads the saved collection"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ScrmViewer,
        "CAPTURE REPLAY <file> opens the SCRM viewer Context"
    );
}

/// Validates: screen-snapshot-scrm Req 10.11 -- CAPTURE REPLAY <missing-file>
/// reports the load error and does NOT open the viewer (no stale collection).
#[test]
fn capture_replay_missing_file_reports_error_no_viewer() {
    // Validates: Requirement 10.11
    use crate::tab_state::KindTag;
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE REPLAY does-not-exist.ffscrm");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("CAPTURE LOAD failed"), "got: {msg:?}");
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ScrmViewer,
        "a failed load must not open the viewer"
    );
}

/// Validates: screen-snapshot-scrm Req 10.12 -- the shell enumerates the
/// *.ffscrm files in scrm_dir() as picker entries (sorted, non-ffscrm ignored).
#[test]
fn enumerate_saved_sessions_lists_ffscrm_only() {
    // Validates: Requirement 10.12
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE SAVE beta.ffscrm");
    shell.handle_command("CAPTURE SAVE alpha.ffscrm");
    // A non-ffscrm file must be ignored.
    shell.handle_command("CAPTURE EXPORT TEXT notes.txt");
    let path = dir.path().join("alpha.ffscrm");
    assert!(path.exists(), "archive saved");
    let entries = shell.enumerate_saved_scrm_sessions();
    let stems: Vec<&str> = entries.iter().map(|e| e.stem.as_str()).collect();
    assert_eq!(
        stems,
        vec!["alpha", "beta"],
        "only .ffscrm files, sorted by stem; got: {stems:?}"
    );
}

/// Validates: screen-snapshot-scrm Req 10.12 -- selecting a session (the picker
/// overlay's Selected branch) dispatches CAPTURE REPLAY <path> through the
/// single front door, which loads the archive and opens the viewer.
#[test]
fn picker_selection_dispatches_replay_through_front_door() {
    // Validates: Requirement 10.12
    use crate::tab_state::KindTag;
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE SAVE repro.ffscrm");
    shell.handle_command("CAPTURE PURGE");
    // Open the picker (bare REPLAY), then mirror the overlay's Selected branch.
    shell.handle_command("CAPTURE REPLAY");
    assert!(shell.scrm_ui.replay_picker.is_some(), "picker open");
    let entries = shell.enumerate_saved_scrm_sessions();
    let chosen = entries
        .iter()
        .find(|e| e.stem == "repro")
        .expect("repro listed");
    shell.scrm_ui.replay_picker = None;
    shell.handle_command(&format!("CAPTURE REPLAY {}", chosen.path.display()));
    assert!(shell.scrm.is_active(), "selection loaded the collection");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ScrmViewer,
        "selection opened the viewer"
    );
    let _ = dir; // keep the TempDir alive
}

/// Validates: screen-snapshot-scrm Req 10.13, 12.4 -- a SAVED session exports to
/// PDF with screen content as selectable text (a %PDF header + a Tj text
/// operator), via the existing CAPTURE EXPORT PDF front-door path.
#[test]
fn saved_session_exports_to_pdf_with_selectable_text() {
    // Validates: Requirement 10.13
    let (mut shell, dir) = make_shell_with_scrm_dir();
    // Persist and purge so the export acts on a freshly LOADED saved session.
    shell.handle_command("CAPTURE SAVE repro.ffscrm");
    shell.handle_command("CAPTURE PURGE");
    let archive = dir.path().join("repro.ffscrm");
    shell.handle_command(&format!("CAPTURE LOAD {}", archive.display()));
    assert!(shell.scrm.is_active(), "saved session loaded");
    shell.handle_command("CAPTURE EXPORT PDF saved.pdf");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("exported"), "got: {msg:?}");
    let path = dir.path().join("saved.pdf");
    assert!(path.exists(), "PDF written at {}", path.display());
    let bytes = std::fs::read(&path).expect("read pdf");
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(16)]);
    assert!(head.starts_with("%PDF-"), "real PDF header; got: {head:?}");
    let full = String::from_utf8_lossy(&bytes);
    assert!(
        full.contains(" Tj"),
        "saved-session PDF has selectable text (not raster)"
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

// === Stream SCRM Phase 2C FEAT-002: capture-STOP Save-As modal ==============

/// Validates: screen-snapshot-scrm Req 7.6, 24.1 -- CAPTURE STOP with a
/// non-empty active Collection stops the session AND opens the Save-As modal,
/// defaulting the directory to scrm_dir() and pre-filling the session name.
#[test]
fn capture_stop_with_collection_opens_save_as_modal() {
    // Validates: Requirement 7.6, 24.1
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE STOP");
    let state = shell
        .scrm_ui
        .save_as
        .as_ref()
        .expect("STOP with a non-empty collection opens the Save-As modal");
    assert_eq!(state.name, "Repro", "name pre-filled from collection");
    assert_eq!(
        state.directory,
        dir.path().to_string_lossy(),
        "directory defaults to scrm_dir()"
    );
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(
        msg.contains("stopped"),
        "STOP still reports stopped: {msg:?}"
    );
}

/// Validates: screen-snapshot-scrm Req 24.1 -- CAPTURE STOP with no active
/// collection opens NO modal and keeps the status message.
#[test]
fn capture_stop_without_collection_opens_no_modal() {
    // Validates: Requirement 24.1
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE STOP");
    assert!(
        shell.scrm_ui.save_as.is_none(),
        "STOP with no collection opens no modal"
    );
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("No active screen collection"), "got: {msg:?}");
}

/// Validates: screen-snapshot-scrm Req 24.1 -- CAPTURE STOP with an active but
/// EMPTY collection opens no modal (nothing worth saving).
#[test]
fn capture_stop_with_empty_collection_opens_no_modal() {
    // Validates: Requirement 24.1
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE START Empty");
    shell.handle_command("CAPTURE STOP");
    assert!(
        shell.scrm_ui.save_as.is_none(),
        "STOP with an empty collection opens no modal"
    );
}

/// Validates: screen-snapshot-scrm Req 24.4 -- the modal's SAVE command for the
/// native archive dispatches `CAPTURE SAVE <path>` through the front door and
/// the file lands under the scrm dir (no new persistence path).
#[test]
fn save_as_ffscrm_dispatches_capture_save_through_front_door() {
    // Validates: Requirement 24.4
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE STOP");
    let command = shell
        .scrm_ui
        .save_as
        .as_ref()
        .expect("modal open")
        .save_command();
    assert!(command.starts_with("CAPTURE SAVE "), "got: {command}");
    // Mirror the overlay's SAVE branch: dispatch through the single front door.
    shell.scrm_ui.save_as = None;
    shell.handle_command(&command);
    let path = dir.path().join("Repro.ffscrm");
    assert!(
        path.exists(),
        "native archive must land under scrm_dir(): {}",
        path.display()
    );
}

/// Validates: screen-snapshot-scrm Req 24.4 -- a PDF choice dispatches
/// `CAPTURE EXPORT PDF <path>` through the front door and writes a PDF.
#[test]
fn save_as_pdf_dispatches_capture_export_through_front_door() {
    // Validates: Requirement 24.4
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE STOP");
    {
        let state = shell.scrm_ui.save_as.as_mut().expect("modal open");
        state.output_type = crate::scrm_save_as_dialog::OutputType::Pdf;
    }
    let command = shell
        .scrm_ui
        .save_as
        .take()
        .expect("modal open")
        .save_command();
    assert!(command.starts_with("CAPTURE EXPORT PDF "), "got: {command}");
    shell.handle_command(&command);
    let path = dir.path().join("Repro.pdf");
    assert!(
        path.exists(),
        "PDF must land under scrm_dir(): {}",
        path.display()
    );
    let bytes = std::fs::read(&path).expect("read pdf");
    assert!(bytes.starts_with(b"%PDF-"), "valid PDF header");
}

/// Validates: screen-snapshot-scrm Req 24.5 -- CANCEL writes nothing but the
/// capture stays stopped. Mirrors the overlay's Cancel branch: clear save_as and
/// write no file.
#[test]
fn save_as_cancel_writes_nothing_but_stays_stopped() {
    // Validates: Requirement 24.5
    let (mut shell, dir) = make_shell_with_scrm_dir();
    shell.handle_command("CAPTURE STOP");
    assert!(shell.scrm_ui.save_as.is_some(), "modal open after STOP");
    // Cancel: the overlay clears the state and dispatches no command.
    shell.scrm_ui.save_as = None;
    // Auto-capture is off after stop (Req 7.6) and nothing is written.
    assert!(
        !shell.scrm.auto_capture_enabled(),
        "stopped: auto-capture off"
    );
    let wrote_any = std::fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(|e| e.ok())
        .any(|e| {
            let n = e.file_name();
            let n = n.to_string_lossy();
            n.ends_with(".ffscrm") || n.ends_with(".pdf") || n.ends_with(".txt")
        });
    assert!(!wrote_any, "CANCEL must not write any session file");
    // The in-memory collection remains available (Req 24.5 / 7.6).
    assert!(
        shell.scrm.active_collection().is_some(),
        "the collection remains available after cancel"
    );
}
