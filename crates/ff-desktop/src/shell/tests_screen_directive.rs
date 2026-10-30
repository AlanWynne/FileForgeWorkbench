//! Master-screen-directive conformance harness (Phase 1A, task 1A.2; extended
//! by Stream SCRM C.2/C.3).
//!
//! A REUSABLE helper asserting the snapshot/capture contract of the master
//! screen directive: for a given open Context, SNAPSHOT returns a model (no
//! "cannot be captured yet") OR the Context carries the DOCUMENTED opt-out that
//! flows through the per-Context `ScreenProvider` seam
//! (`shell/screen_provider.rs`).
//!
//! The harness is the authoritative checklist for the retrofit matrix
//! (master-screen-directive Requirement 5): it walks EVERY core Context and
//! asserts it is now CAPTURABLE (Stream SCRM C.2/C.3 flipped them from the
//! Phase 1A pending-provider opt-out to a real model), plus the ONE remaining
//! DOCUMENTED opt-out -- the parked editor Context (FileEditor / Untitled).
//!
//! Each Context's kind is set on the active tab through the shared
//! `set_active_tab_context` choke point, then the contract is asserted at BOTH
//! layers: the `active_screen_capture()` seam outcome AND the user-visible
//! `SNAPSHOT` result (so the harness proves the not-capturable path flows
//! through the opt-out seam, never a stray hard check).
//!
//! Validates: master-screen-directive Requirement 4.1, 4.2, 4.3, 5.1, 5.3;
//! screen-snapshot-scrm Requirement 2.1, 2.2, 2.3.

#![allow(unused_imports)]
use super::screen_provider::{ScreenCapture, PARKED_EDITOR_REASON};
use super::tests_common::*;
use crate::tab_state::{KindTag, TabKind};

/// What the master-directive conformance harness expects of a Context's
/// snapshot/capture contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureExpectation {
    /// The Context has a `ScreenProvider`: SNAPSHOT MUST return a model and MUST
    /// NOT report "cannot be captured yet".
    Capturable,
    /// The Context carries the documented parked-editor opt-out: SNAPSHOT
    /// legitimately reports not-capturable through the seam.
    OptOutParkedEditor,
}

/// Build the shell, set the active tab to `kind`, run frames to settle, confirm
/// the active Context is `expect_kind`, then assert the Context's
/// snapshot/capture contract matches `expect` at BOTH the seam and the
/// user-visible SNAPSHOT layer.
///
/// Validates: master-screen-directive Requirement 4.1, 4.2, 4.3.
fn assert_capture_contract(kind: TabKind, expect_kind: KindTag, expect: CaptureExpectation) {
    let mut harness = harness_shell();
    harness
        .state_mut()
        .set_active_tab_context(kind, "[CONFORMANCE]");
    for _ in 0..2 {
        harness.run();
    }

    // Precondition: the active Context is the one under test.
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        expect_kind,
        "active Context must be {expect_kind:?}"
    );

    // Layer 1: the per-Context ScreenProvider dispatch seam outcome.
    let capture = harness.state().active_screen_capture();
    // Layer 2: the user-visible SNAPSHOT result (default PlainText format).
    let snapshot = harness.state().snapshot_text_for_active("");

    match expect {
        CaptureExpectation::Capturable => {
            assert!(
                matches!(capture, ScreenCapture::Model(_)),
                "{expect_kind:?} must provide a ScreenModel through the seam; got {capture:?}"
            );
            let (text, _format) = snapshot.unwrap_or_else(|e| {
                panic!("{expect_kind:?} must be capturable; SNAPSHOT errored: {e}")
            });
            assert!(
                !text.contains("cannot be captured yet"),
                "a capturable Context must not report the not-capturable message"
            );
            assert!(!text.is_empty(), "captured snapshot text must be non-empty");
        }
        CaptureExpectation::OptOutParkedEditor => {
            match capture {
                ScreenCapture::OptOut(reason) => assert_eq!(
                    reason, PARKED_EDITOR_REASON,
                    "{expect_kind:?} must report the documented parked-editor opt-out"
                ),
                ScreenCapture::Model(_) => {
                    panic!("{expect_kind:?} is the documented opt-out; expected not-capturable")
                }
            }
            let err = snapshot.expect_err("an opt-out Context's SNAPSHOT must be an error");
            assert!(
                err.contains("cannot be captured yet"),
                "the not-capturable response must flow through the seam; got: {err:?}"
            );
        }
    }
}

// Validates: master-screen-directive Req 4.1, 4.2; screen-snapshot-scrm Req 2.1,
// 2.2 -- the POM (Menu_Workspace) is capturable (the shell launches on it).
#[test]
fn conformance_pom_is_capturable() {
    let mut harness = harness_shell();
    for _ in 0..2 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::MenuWorkspace,
        "the shell launches on the POM"
    );
    let capture = harness.state().active_screen_capture();
    assert!(
        matches!(capture, ScreenCapture::Model(_)),
        "the POM must be capturable through the seam"
    );
    let (text, _) = harness
        .state()
        .snapshot_text_for_active("")
        .expect("POM SNAPSHOT must succeed");
    assert!(!text.contains("cannot be captured yet"));
}

// Validates: master-screen-directive Req 4.1, 4.2, 5.1, 5.3 -- EVERY core
// Context in the retrofit matrix is now CAPTURABLE through the per-Context
// ScreenProvider seam (Stream SCRM C.2/C.3 flipped them from the Phase 1A
// opt-out). This parameterised walk is the authoritative checklist for Req 5.
#[test]
fn conformance_every_retrofitted_context_is_capturable() {
    let cases: Vec<(TabKind, KindTag)> = vec![
        (TabKind::ConfigPanel, KindTag::ConfigPanel),
        (TabKind::ThemeEditor, KindTag::ThemeEditor),
        (TabKind::MenusEditor, KindTag::MenusEditor),
        (TabKind::KeysEditor, KindTag::KeysEditor),
        (TabKind::KindsEditor, KindTag::KindsEditor),
        (TabKind::AliasEditor, KindTag::AliasEditor),
        (TabKind::CommandConfigurator, KindTag::CommandConfigurator),
        (TabKind::FilesPanel, KindTag::FilesPanel),
        (TabKind::FileExplorerPanel, KindTag::FileExplorerPanel),
        (TabKind::SearchResults, KindTag::SearchResults),
        (TabKind::PluginManager, KindTag::PluginManager),
        (TabKind::EventLog, KindTag::EventLog),
        (TabKind::MacroLibrary, KindTag::MacroLibrary),
        (TabKind::HelpContext, KindTag::HelpContext),
        (TabKind::ScrmViewer, KindTag::ScrmViewer),
    ];
    for (kind, tag) in cases {
        assert_capture_contract(kind, tag, CaptureExpectation::Capturable);
    }
}

// Validates: master-screen-directive Req 4.3, 4.4; screen-snapshot-scrm Req 2.3
// -- the parked editor Context (FileEditor / Untitled) is the ONE remaining
// DOCUMENTED opt-out and reports the not-capturable state THROUGH the seam.
#[test]
fn conformance_parked_editor_reports_documented_opt_out() {
    assert_capture_contract(
        TabKind::Untitled,
        KindTag::Untitled,
        CaptureExpectation::OptOutParkedEditor,
    );
    assert_capture_contract(
        TabKind::FileEditor,
        KindTag::FileEditor,
        CaptureExpectation::OptOutParkedEditor,
    );
}
