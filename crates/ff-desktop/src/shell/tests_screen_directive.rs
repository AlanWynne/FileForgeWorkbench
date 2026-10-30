//! Master-screen-directive conformance harness (Phase 1A, task 1A.2).
//!
//! A REUSABLE full-shell `egui_kittest` helper asserting the snapshot/capture
//! contract of the master screen directive: for a given open Context, SNAPSHOT
//! returns a model (no "cannot be captured yet") OR the Context carries the
//! DOCUMENTED opt-out that flows through the new per-Context `ScreenProvider`
//! seam (`shell/screen_provider.rs`). It mirrors the per-arm first-Tab focus
//! test style used across `tests_focus.rs` (full-shell `harness_shell()` driven
//! through the real command path).
//!
//! Phase 1A exercises the harness for TWO Contexts: the POM / Menu_Workspace
//! (MUST pass -- returns a model) and one representative non-menu Context (Event
//! Log -- currently has NO provider, so it MUST report the documented
//! not-capturable state THROUGH the new seam). The harness is parameterised by
//! [`CaptureExpectation`] so Stream SCRM (task C.2) can call it per Context and
//! flip each from `OptOut` to `Model` as providers are added.
//!
//! Validates: master-screen-directive Requirement 4.1, 4.2, 4.3;
//! screen-snapshot-scrm Requirement 2.1, 2.2, 2.3.

#![allow(unused_imports)]
use super::screen_provider::{ScreenCapture, PENDING_PROVIDER_REASON};
use super::tests_common::*;
use crate::tab_state::KindTag;

/// What the master-directive conformance harness expects of a Context's
/// snapshot/capture contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureExpectation {
    /// The Context has a `ScreenProvider`: SNAPSHOT MUST return a model and MUST
    /// NOT report "cannot be captured yet".
    Capturable,
    /// The Context carries the documented Phase 1A pending-provider opt-out:
    /// SNAPSHOT legitimately reports not-capturable through the new seam.
    OptOutPending,
}

/// Open `command` in the full shell through the SINGLE front door
/// (`dispatch_command_string`, screen-snapshot-scrm Req 11), run frames to
/// settle, confirm the active Context is the expected `KindTag`, then assert the
/// Context's snapshot/capture contract matches `expect`.
///
/// This is the reusable, parameterised conformance check Stream SCRM calls per
/// Context. It asserts at BOTH layers: the `active_screen_capture()` seam
/// outcome AND the user-visible `SNAPSHOT` result, so the harness proves the
/// not-capturable path flows through the opt-out seam rather than a stray hard
/// check.
///
/// Validates: master-screen-directive Requirement 4.1, 4.2, 4.3.
fn assert_capture_contract(command: &str, expect_kind: KindTag, expect: CaptureExpectation) {
    let mut harness = harness_shell();
    harness.state_mut().dispatch_command_string(command);
    for _ in 0..4 {
        harness.run();
    }

    // Precondition: the command opened the Context under test.
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        expect_kind,
        "`{command}` must open the {expect_kind:?} Context"
    );

    // Layer 1: the per-Context ScreenProvider dispatch seam outcome.
    let capture = harness.state().active_screen_capture();
    // Layer 2: the user-visible SNAPSHOT result (default PlainText format).
    let snapshot = harness.state().snapshot_text_for_active("");

    match expect {
        CaptureExpectation::Capturable => {
            assert!(
                matches!(capture, ScreenCapture::Model(_)),
                "`{command}` Context must provide a ScreenModel through the seam; got {capture:?}"
            );
            let (text, _format) = snapshot.unwrap_or_else(|e| {
                panic!("`{command}` Context must be capturable; SNAPSHOT errored: {e}")
            });
            assert!(
                !text.contains("cannot be captured yet"),
                "a capturable Context must not report the not-capturable message"
            );
            assert!(!text.is_empty(), "captured snapshot text must be non-empty");
        }
        CaptureExpectation::OptOutPending => {
            match capture {
                ScreenCapture::OptOut(reason) => assert_eq!(
                    reason, PENDING_PROVIDER_REASON,
                    "`{command}` must report the documented pending-provider opt-out"
                ),
                ScreenCapture::Model(_) => panic!(
                    "`{command}` has no provider in Phase 1A; expected the documented opt-out"
                ),
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
// 2.2 -- the POM (Menu_Workspace) is capturable: the harness obtains a model and
// SNAPSHOT succeeds (menu snapshot behaviour unchanged by the seam).
#[test]
fn conformance_pom_is_capturable() {
    // The shell launches on the POM; POM is the opened Context.
    assert_capture_contract(
        "POM",
        KindTag::MenuWorkspace,
        CaptureExpectation::Capturable,
    );
}

// Validates: master-screen-directive Req 4.3, 4.4; screen-snapshot-scrm Req 2.3
// -- a representative non-menu Context (Event Log) legitimately has no provider
// in Phase 1A and reports the DOCUMENTED not-capturable state THROUGH the new
// opt-out seam, not the old hard-coded menu-only check. Stream SCRM C.2 flips
// this to `Capturable` when the Event Log provider lands.
#[test]
fn conformance_non_menu_context_reports_documented_opt_out() {
    assert_capture_contract("LOG", KindTag::EventLog, CaptureExpectation::OptOutPending);
}
