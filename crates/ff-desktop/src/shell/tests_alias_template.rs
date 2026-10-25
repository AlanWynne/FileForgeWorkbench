//! Full-shell end-to-end test for CR-NR-112 parameterised aliases: the worked
//! example flows through the EXISTING single front door
//! (`dispatch_command_string` -> pre-pass -> `resolve_target` -> the EDIT ladder
//! arm) with no new dispatch path.
//!
//! The pre-pass (`rewrite_verb_to_root`) expands the parameterised alias ECOB
//! (root EDIT, template `FFWB.COBOL.MASTER(&1)`) into the resolved STRING
//! `EDIT FFWB.COBOL.MASTER(helloW)`, which the UNCHANGED EDIT ladder arm consumes
//! -- forwarding the dataset token as the `path` param to `file.open`, observable
//! as the shell's `pending_open` request. This proves command-semantics Req 14.8,
//! 14.9 and command-framework Req 19.1, 19.2 end to end.

use super::tests_common::make_shell;

/// Validates: command-semantics Requirement 14.9; command-framework Requirement
/// 19.1 -- typing `ecob helloW` expands (in the pre-pass) to
/// `EDIT FFWB.COBOL.MASTER(helloW)` and reaches the EXISTING EDIT routing, which
/// sets `pending_open` to the expanded dataset token. The expansion flows through
/// the single front door unchanged (no new dispatcher).
#[test]
fn alias_template_end_to_end_opens_expanded_edit_target() {
    let mut shell = make_shell();
    // Seed the parameterised alias into the ACTIVE environment of the default
    // (POM) Context, so the typed-line pre-pass resolves it. EDIT is NOT an
    // FFEDIT verb on a non-editor Context, so the expanded string reaches the
    // EDIT ladder arm via resolve_target unchanged.
    let env = shell.active_environment_name();
    {
        let mut store = shell.i18n.alias_store.lock().expect("alias store lock");
        store.overlay_row_for_test_with_template(
            env,
            "ECOB",
            "EDIT",
            Some("FFWB.COBOL.MASTER(&1)"),
        );
    }

    // Drive the typed command through the single front door.
    shell.run_command_line("ecob helloW");

    // Observable effect of the EXISTING EDIT ladder arm: a pending open for the
    // expanded dataset token (literal template preserved, &1 substituted).
    let pending = shell.pending_open.lock().expect("pending_open").clone();
    let pending = pending.expect("the expanded EDIT must set a pending open");
    assert_eq!(
        pending.path, "FFWB.COBOL.MASTER(helloW)",
        "the alias expanded to EDIT FFWB.COBOL.MASTER(helloW) and routed through EDIT (Req 14.9)"
    );
    assert!(
        shell.open_error.is_none(),
        "a successful expanded EDIT clears any open_error"
    );
}

/// Validates: command-framework Requirement 19.6 -- a template alias supplied
/// with NO argument (the menu-option-style activation, where the command-line
/// field is not merged) expands with an EMPTY &1, so `ecob` alone resolves to
/// `EDIT FFWB.COBOL.MASTER()` and routes through the existing EDIT arm with the
/// empty-parenthesised token. Confirms the documented menu-activation consequence
/// without routing the command-line field into menu activation.
#[test]
fn alias_template_with_no_argument_expands_empty_and_routes() {
    let mut shell = make_shell();
    let env = shell.active_environment_name();
    {
        let mut store = shell.i18n.alias_store.lock().expect("alias store lock");
        store.overlay_row_for_test_with_template(
            env,
            "ECOB",
            "EDIT",
            Some("FFWB.COBOL.MASTER(&1)"),
        );
    }

    shell.run_command_line("ecob");

    let pending = shell.pending_open.lock().expect("pending_open").clone();
    let pending = pending.expect("an empty-arg expanded EDIT still sets a pending open");
    assert_eq!(
        pending.path, "FFWB.COBOL.MASTER()",
        "an unsupplied &1 expands to the empty string per Req 14.4/19.6"
    );
}
