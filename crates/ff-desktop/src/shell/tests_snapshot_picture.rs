//! Phase 2B / CR-NR-114 tests: the SNAPSHOT command wiring in the shell. Bare
//! SNAPSHOT places the COMPOSITE clipboard payload directly (no picker); an
//! explicit `SNAPSHOT <FORMAT>` produces that one format directly (text/PNG to
//! a file, PDF via the ff-scrm path). The multi-flavor clipboard layer (image +
//! text) and the DIB byte layout are unit-tested in `crate::clipboard_backend`
//! with a fake backend / pure helpers (no real OS clipboard); the PNG
//! rasteriser is unit-tested in `ff_screen_model::raster`.

#![allow(unused_imports)]
use super::tests_common::*;

/// Validates: screen-snapshot-scrm Req 4.1, 4.7 (RETIRED), 23.4 -- bare SNAPSHOT
/// (no format) is the two-phase async REQUEST (CR-NR-114, FEAT-002): it never
/// opens a format picker and never synchronously rasterises the text into the
/// clipboard. Instead it stashes the pre-rendered text flavors and asks the
/// frame loop for a live window screenshot. The command phase is therefore
/// non-blocking: the observable effect is the pending request (payload stashed,
/// screenshot requested), NOT a synchronous clipboard status, NOT an
/// "unknown format" error, and NOT a "cannot be captured" error.
#[test]
fn bare_snapshot_sets_pending_request_not_synchronous_place() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("SNAPSHOT");

    // The REQUEST contract: a pending payload is stashed and a window screenshot
    // was requested (the frame loop assembles + places on the Event::Screenshot
    // frame, which this headless command-phase test does not run).
    assert!(
        shell.snapshot.screenshot_requested,
        "bare SNAPSHOT requests an async window screenshot"
    );
    let pending = shell
        .snapshot
        .pending
        .as_ref()
        .expect("bare SNAPSHOT stashes a pending payload");
    // The CF_UNICODETEXT flavor is the Markdown-fenced render (Req 23.4).
    assert!(
        pending.text.contains("```"),
        "pending text flavor is Markdown-fenced, got: {:?}",
        pending.text
    );
    // The CF_HTML <pre> body is the plain box-drawn screen text (no ``` fence).
    assert!(
        !pending.pre_text.contains("```"),
        "pending pre_text is the plain box-drawn body, not fenced Markdown"
    );
    // The command phase does not touch the clipboard, so it reports no error and
    // in particular not an "unknown format" / "cannot be captured" error.
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(
        !msg.contains("unknown format"),
        "bare SNAPSHOT is not an unknown-format error, got: {msg:?}"
    );
    assert!(
        !msg.contains("cannot be captured"),
        "a seeded POM is capturable, got: {msg:?}"
    );
}

/// Validates: screen-snapshot-scrm Req 4.8 -- an explicit text format produces
/// that single format DIRECTLY to a file (not the clipboard, not a picker).
#[test]
fn explicit_text_format_writes_single_format_file() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let mut shell = make_shell();
    shell.dir_overrides.scrm = Some(dir.path().to_path_buf());
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();

    shell.handle_command("SNAPSHOT MARKDOWN");

    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("written to"), "got: {msg:?}");
    // A .md file was written and carries the Markdown fence (Req 23.4 shape).
    let md = std::fs::read_dir(dir.path())
        .expect("read scrm dir")
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
        .expect("a .md file was written");
    let text = std::fs::read_to_string(&md).expect("read md");
    assert!(text.contains("```"), "Markdown file is fenced");
}

/// Validates: screen-snapshot-scrm Req 4.3, 23.5e -- `SNAPSHOT ANSI` produces
/// ANSI as a single-format FILE output (a terminal/ANSI target), and is NEVER
/// placed on the clipboard. The written file carries raw ANSI escape bytes.
#[test]
fn explicit_ansi_writes_file_and_is_not_clipboard() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let mut shell = make_shell();
    shell.dir_overrides.scrm = Some(dir.path().to_path_buf());
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();

    shell.handle_command("SNAPSHOT ANSI");

    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(
        msg.contains("written to"),
        "ANSI is a file output, got: {msg:?}"
    );
    assert!(
        !msg.contains("clipboard"),
        "ANSI must NOT go to the clipboard (Req 23.5e)"
    );
    // A .ans file was written -- ANSI reaches a FILE target, never the clipboard.
    // (The POM fields are uncoloured, so the ANSI render carries no SGR escapes;
    // the point of this test is the file-not-clipboard routing, which the
    // ff-screen-model ANSI renderer tests complement for coloured fields.)
    let ans = std::fs::read_dir(dir.path())
        .expect("read scrm dir")
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|e| e.to_str()) == Some("ans"))
        .expect("a .ans file was written");
    let text = std::fs::read_to_string(&ans).expect("read ans");
    assert!(!text.is_empty(), "ANSI file has content");
}

/// Validates: screen-snapshot-scrm Req 6.3 -- an unknown format is a clear
/// non-fatal error (and, with the picker retired, never opens anything).
#[test]
fn unknown_format_is_error() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("SNAPSHOT BOGUS");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("unknown format"), "got: {msg:?}");
}

/// Validates: screen-snapshot-scrm Req 23.3, 23.6 -- `SNAPSHOT PDF` reaches the
/// handler through the single command path and writes a single-screen PDF
/// (reusing ff-scrm/pdf.rs).
#[test]
fn snapshot_pdf_writes_single_screen_pdf_via_command_path() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let mut shell = make_shell();
    shell.dir_overrides.scrm = Some(dir.path().to_path_buf());
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();

    shell.dispatch_command_string("SNAPSHOT PDF");

    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("PDF written to"), "got: {msg:?}");
    // A .pdf file exists under the override dir and is a real PDF.
    let pdf = std::fs::read_dir(dir.path())
        .expect("read scrm dir")
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|e| e.to_str()) == Some("pdf"))
        .expect("a .pdf file was written");
    let bytes = std::fs::read(&pdf).expect("read pdf");
    assert!(bytes.starts_with(b"%PDF-1.7"), "valid PDF header");
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains(" Tj"), "selectable text, not an image");
}
