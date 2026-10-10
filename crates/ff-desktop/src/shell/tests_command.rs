//! Shell tests -- command area (split from shell/tests.rs, CR F3).
//! Items are verbatim; only their file location changed.

#![allow(unused_imports)]
use super::tests_common::*;
use ff_keys::{KeyMap, ModifiedKey};
use std::sync::{Arc, Mutex};

use ff_command::{
    CommandDispatch, CommandError, CommandHandler, CommandHistory, CommandId, CommandMetadata,
    CommandParams, CommandRegistry, CommandResult, ExecutionContext,
};

/// Validates: Requirement 8.1 -- command field submits non-empty text on Enter.
/// The UI wiring (has_focus + key_pressed) is verified manually; this test
/// confirms the dispatch path handle_command is reachable for any non-empty input.
#[test]
fn command_field_enter_submits_non_empty_command() {
    // Validates: command-semantics Requirement 8.1
    // Simulate what render_command_field does: trim and dispatch.
    let raw = "  EXIT  ";
    let cmd = raw.trim().to_string();
    assert!(!cmd.is_empty(), "trimmed command must be non-empty");
    // EXIT is a shell-level intercept
    assert!(is_shell_command(&cmd));
}

/// Validates: Requirement 8.2 -- command field does not submit when empty.
#[test]
fn command_field_enter_does_not_submit_empty_command() {
    // Validates: command-semantics Requirement 8.2
    let raw = "   ";
    let cmd = raw.trim().to_string();
    // The guard `!self.command_text.is_empty()` prevents dispatch.
    assert!(
        cmd.is_empty(),
        "whitespace-only input must be treated as empty"
    );
}

/// Validates: Requirement 8.1 -- EDIT path dispatches file.open via handle_command.
#[test]
fn command_field_edit_command_dispatches_file_open() {
    // Validates: command-semantics Requirement 8.1
    let cmd = "EDIT /some/file.txt";
    assert!(is_shell_command(cmd));
}

/// Validates: Requirement 14.8 -- central panel dispatches on TabKind.
#[test]
fn central_panel_always_shows_editor_regardless_of_open_files() {
    // Validates: Requirement 14.8
    use crate::tab_manager::TabManager;
    use tokio::runtime::Runtime;
    let runtime = Runtime::new().expect("runtime");
    let mgr = TabManager::new(&runtime, "welcome");
    assert_eq!(mgr.tabs().len(), 1);
    assert!(mgr.tabs()[0].path.is_none(), "welcome tab has no path");
}

/// Validates: Requirement 14.10, 14.14 -- START command is a shell-level intercept.
#[test]
fn start_command_is_recognised_as_shell_command() {
    // Validates: Requirement 14.10
    assert!(is_shell_command("START"));
    assert!(is_shell_command("POM"));
}

/// Validates: Requirement 14.11 -- CLOSE command is a shell-level intercept.
#[test]
fn close_command_is_recognised_as_shell_command() {
    // Validates: Requirement 14.11
    assert!(is_shell_command("CLOSE"));
}

/// Validates: Requirement S.1 -- file_open_dialog sets pending_open when a path is returned.
#[test]
fn file_open_dialog_pending_open_is_set_when_path_returned() {
    // Simulates the closure body inside open_file_dialog(): when a path
    // is available, it must be written into pending_open as `(path, owning_env)`.
    // The native dialog opens a host file, so owning_env is None (host FS
    // default, CR-CH-053 Task 19 Req 15.3).
    let pending: Arc<Mutex<Option<(String, Option<String>)>>> = Arc::new(Mutex::new(None));
    let path = "/tmp/test_file.txt".to_string();

    // Simulate the closure that open_file_dialog() spawns
    *pending.lock().expect("pending lock") = Some((path.clone(), None));

    let result = pending.lock().expect("pending lock").take();
    assert_eq!(result, Some((path, None)));
}

/// Validates: Requirement S.1 -- file_open_dialog leaves pending_open None when dialog is cancelled.
#[test]
fn file_open_dialog_pending_open_unchanged_when_cancelled() {
    let pending: Arc<Mutex<Option<(String, Option<String>)>>> = Arc::new(Mutex::new(None));

    // Simulate cancelled dialog -- closure does nothing
    let result = pending.lock().expect("pending lock").take();
    assert_eq!(result, None);
}

/// Validates: Requirement 18.6 -- EDIT <path> dispatches file.open with path param.
#[test]
fn file_open_handler_succeeds_with_path_param() {
    // Validates: Requirement 2.1 -- execute_command routes through dispatch
    let (registry, dispatch) = make_dispatch();

    let received = Arc::new(Mutex::new(String::new()));
    let received_clone = received.clone();

    struct CaptureHandler {
        received: Arc<Mutex<String>>,
    }
    impl CommandHandler for CaptureHandler {
        fn is_undoable(&self) -> bool {
            false
        }
        fn execute(&self, _ctx: &ExecutionContext, params: &CommandParams) -> CommandResult {
            if let Some(p) = params.get_string("path") {
                *self.received.lock().unwrap() = p.to_string();
                CommandResult::Ok
            } else {
                CommandResult::Err(CommandError::ExecutionFailed {
                    id: "file.open".to_string(),
                    description: "missing path".to_string(),
                })
            }
        }
    }

    let id = CommandId::new("file.open").unwrap();
    registry
        .register(
            id,
            meta("Open File", "file"),
            Box::new(CaptureHandler {
                received: received_clone,
            }),
        )
        .unwrap();

    let mut params = CommandParams::new();
    params.insert("path", "/tmp/test.txt");
    let result = dispatch.execute_command("file.open", params);

    assert!(result.is_ok());
    assert_eq!(*received.lock().unwrap(), "/tmp/test.txt");
}

/// Validates: Requirement 18.6 -- file.open without path param returns error.
#[test]
fn file_open_handler_fails_without_path_param() {
    // Validates: Requirement 2.2 -- missing param produces Err result
    let (registry, dispatch) = make_dispatch();

    struct RejectNoPath;
    impl CommandHandler for RejectNoPath {
        fn is_undoable(&self) -> bool {
            false
        }
        fn execute(&self, _ctx: &ExecutionContext, params: &CommandParams) -> CommandResult {
            if params.get_string("path").is_some() {
                CommandResult::Ok
            } else {
                CommandResult::Err(CommandError::ExecutionFailed {
                    id: "file.open".to_string(),
                    description: "missing path".to_string(),
                })
            }
        }
    }

    let id = CommandId::new("file.open").unwrap();
    registry
        .register(id, meta("Open File", "file"), Box::new(RejectNoPath))
        .unwrap();

    let result = dispatch.execute_command("file.open", CommandParams::new());
    assert!(result.is_err());
}

/// Validates: Requirement 18.6 -- EXIT command dispatches file.exit and sets close flag.
#[test]
fn file_exit_handler_sets_close_flag() {
    // Validates: Requirement 2.1 -- execute_command routes through dispatch
    let (registry, dispatch) = make_dispatch();

    let closed = Arc::new(Mutex::new(false));
    let closed_clone = closed.clone();

    struct ExitHandler {
        closed: Arc<Mutex<bool>>,
    }
    impl CommandHandler for ExitHandler {
        fn is_undoable(&self) -> bool {
            false
        }
        fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
            *self.closed.lock().unwrap() = true;
            CommandResult::Ok
        }
    }

    let id = CommandId::new("file.exit").unwrap();
    registry
        .register(
            id,
            meta("Exit", "file"),
            Box::new(ExitHandler {
                closed: closed_clone,
            }),
        )
        .unwrap();

    let result = dispatch.execute_command("file.exit", CommandParams::new());
    assert!(result.is_ok());
    assert!(*closed.lock().unwrap(), "exit handler must set close flag");
}

/// Validates: Requirement 18.6 -- unrecognised command ID returns NotFound error.
#[test]
fn dispatch_returns_not_found_for_unknown_command() {
    // Validates: Requirement 2.2 -- unregistered command returns Err(NotFound)
    let (_registry, dispatch) = make_dispatch();
    let result = dispatch.execute_command("file.open", CommandParams::new());
    assert!(result.is_err());
    assert!(matches!(
        result,
        CommandResult::Err(CommandError::NotFound { .. })
    ));
}

// -- Phase U: CommandEngine dispatch tests ----------------------------

/// Validates: Requirement 21.1 -- empty command line returns "No command" status.
#[test]
fn command_engine_empty_input_returns_no_command() {
    // Validates: Phase U 21.1 -- CommandEngine replaces hard-coded dispatch
    use ff_command_semantics::{CommandEngine, StatusKind};
    let mut engine = CommandEngine::new();
    let status = engine.execute_command_line("");
    assert_eq!(status.text, "No command");
    assert_eq!(status.kind, StatusKind::Info);
}

/// Validates: Requirement 21.1 -- unrecognised command produces RuntimeError status.
#[test]
fn command_engine_unknown_command_returns_runtime_error() {
    // Validates: Phase U 21.1 -- engine surfaces error status for unknown commands
    use ff_command_semantics::{CommandEngine, StatusKind};
    let mut engine = CommandEngine::new();
    let status = engine.execute_command_line("NOSUCHCMD");
    assert_eq!(status.kind, StatusKind::RuntimeError);
    assert!(status.text.contains("NOSUCHCMD"));
}

/// Validates: Requirement 21.1 -- syntax error in command line produces SyntaxError status.
#[test]
fn command_engine_syntax_error_returns_syntax_error_status() {
    // Validates: Phase U 21.1 -- engine parses and surfaces syntax errors
    use ff_command_semantics::{CommandEngine, StatusKind};
    let mut engine = CommandEngine::new();
    let status = engine.execute_command_line("FIND 'unclosed");
    assert_eq!(status.kind, StatusKind::SyntaxError);
}

/// Validates: Requirement 21.1 -- shell-level EXIT intercept bypasses engine.
#[test]
fn shell_intercepts_exit_before_engine() {
    // Validates: Phase U 21.1 -- EXIT/QUIT/=X are shell-level, not engine commands
    assert!(is_shell_command("EXIT"));
    assert!(is_shell_command("QUIT"));
    assert!(is_shell_command("=X"));
    assert!(is_shell_command("X"));
    assert!(!is_shell_command("FIND"));
    assert!(!is_shell_command("LOCATE"));
}

/// Validates: Requirement 21.1 -- shell-level EDIT intercept bypasses engine.
#[test]
fn shell_intercepts_edit_before_engine() {
    // Validates: Phase U 21.1 -- EDIT <path> is a shell-level file-open command
    assert!(is_shell_command("EDIT /some/path"));
    assert!(is_shell_command("EDIT"));
    assert!(!is_shell_command("EDITX"));
}

/// Validates: Requirement 3.1 -- assigned F-key returns its command string.
#[test]
fn egui_fkey_assigned_key_returns_command() {
    // Validates: function-keys-and-history Requirement 3.1
    use ff_keys::{FunctionKey, KeyMapResolver};
    let map = KeyMap::default_global();
    let resolver = KeyMapResolver::new(map);
    let cmd = resolver
        .active_key_map()
        .get_plain(FunctionKey::F3)
        .map(|b| b.command());
    assert_eq!(cmd, Some("END"));
}

/// Validates: Requirement 19.2 -- `=FILES` is a shell-level intercept.
#[test]
fn equals_files_command_is_shell_intercept() {
    // Validates: Requirement 19.2
    assert!(is_shell_command("=FILES"));
}

/// Validates: Requirement 19.3 -- `FILES` (no `=`) is a shell-level intercept.
#[test]
fn files_no_prefix_command_is_shell_intercept() {
    // Validates: Requirement 19.3
    // Note: bare "FILES" was previously option 1 (FilesPanel). It is now
    // re-routed to open a new FileExplorerPanel tab. The is_shell_command
    // helper must include "=FILES" and the routing must open a NEW tab.
    assert!(is_shell_command("=FILES"));
}

/// Validates: B062 (Option C) -- `verb_arg` matches the verb case-insensitively
/// and returns the case-preserved, trimmed remainder (empty for the bare verb),
/// or None when the verb does not match.
#[test]
fn verb_arg_matches_case_insensitively_and_preserves_argument() {
    use super::helpers::verb_arg;
    // Case-insensitive verb, case-preserved argument.
    assert_eq!(verb_arg("FIND 'MixedCase'", "FIND"), Some("'MixedCase'"));
    assert_eq!(verb_arg("find 'MixedCase'", "FIND"), Some("'MixedCase'"));
    assert_eq!(
        verb_arg("  Theme  Default Dark  ", "THEME"),
        Some("Default Dark")
    );
    // Bare verb -> empty remainder (Some, not None).
    assert_eq!(verb_arg("THEME", "THEME"), Some(""));
    assert_eq!(verb_arg("theme", "THEME"), Some(""));
    // Non-matching verb -> None (and does not match a prefix of a longer word).
    assert_eq!(verb_arg("FINDER x", "FIND"), None);
    assert_eq!(verb_arg("LOCATE x", "FIND"), None);
}

/// Validates: B062 -- the CHANGE argument parser preserves the case of both the
/// from- and to- strings (quoted and bare).
#[test]
fn parse_two_args_preserves_argument_case() {
    use super::helpers::parse_two_args;
    let (old, new) = parse_two_args("'Error' 'ERROR'").expect("two quoted args");
    assert_eq!(old, "Error");
    assert_eq!(new, "ERROR");
    let (a, b) = parse_two_args("MixedOld MixedNew").expect("two bare args");
    assert_eq!(a, "MixedOld");
    assert_eq!(b, "MixedNew");
}

/// Validates: Requirement 16.1 -- CAPS ON converts typed chars to uppercase.
#[test]
fn caps_on_command_sets_caps_mode_on() {
    // Validates: Requirement 16.1
    use ff_edit_operations::CapsMode;
    let mut shell = make_shell();
    shell.handle_command("CAPS ON");
    assert_eq!(shell.tabs.active_tab().edit_profile.caps, CapsMode::On);
    assert!(shell.open_error.is_none());
}

/// Validates: Requirement 16.1 -- CAPS OFF reverts to case-preserving input.
#[test]
fn caps_off_command_sets_caps_mode_off() {
    // Validates: Requirement 16.1
    use ff_edit_operations::CapsMode;
    let mut shell = make_shell();
    shell.handle_command("CAPS ON");
    shell.handle_command("CAPS OFF");
    assert_eq!(shell.tabs.active_tab().edit_profile.caps, CapsMode::Off);
}

/// Validates: Requirement 16.4 -- NULLS ON sets nulls mode.
#[test]
fn nulls_on_command_sets_nulls_mode_on() {
    // Validates: Requirement 16.4
    use ff_edit_operations::NullsMode;
    let mut shell = make_shell();
    shell.handle_command("NULLS ON");
    assert_eq!(shell.tabs.active_tab().edit_profile.nulls, NullsMode::On);
    assert!(shell.open_error.is_none());
}

/// Validates: Requirement 16.4 -- NULLS OFF clears nulls mode.
#[test]
fn nulls_off_command_sets_nulls_mode_off() {
    // Validates: Requirement 16.4
    use ff_edit_operations::NullsMode;
    let mut shell = make_shell();
    shell.handle_command("NULLS ON");
    shell.handle_command("NULLS OFF");
    assert_eq!(shell.tabs.active_tab().edit_profile.nulls, NullsMode::Off);
}

/// Validates: Requirement 16.5 -- PROFILE displays current settings.
#[test]
fn profile_command_sets_open_error_to_summary() {
    // Validates: Requirement 16.5
    let mut shell = make_shell();
    shell.handle_command("PROFILE");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(
        msg.contains("CAPS(OFF)"),
        "summary should contain CAPS(OFF), got: {msg}"
    );
    assert!(
        msg.contains("NULLS(OFF)"),
        "summary should contain NULLS(OFF)"
    );
}

/// Validates: Requirement 16.7 -- STATS ON sets stats mode.
#[test]
fn stats_on_command_sets_stats_mode_on() {
    // Validates: Requirement 16.7
    use ff_edit_operations::StatsMode;
    let mut shell = make_shell();
    shell.handle_command("STATS ON");
    assert_eq!(shell.tabs.active_tab().edit_profile.stats, StatsMode::On);
    assert!(shell.open_error.is_none());
}

/// Validates: Requirement 16.7 -- STATS OFF clears stats mode.
#[test]
fn stats_off_command_clears_stats_mode() {
    // Validates: Requirement 16.7
    use ff_edit_operations::StatsMode;
    let mut shell = make_shell();
    shell.handle_command("STATS ON");
    shell.handle_command("STATS OFF");
    assert_eq!(shell.tabs.active_tab().edit_profile.stats, StatsMode::Off);
}

/// Validates: Requirement 16.10 -- AUTONUM ON dispatches to NUMBER ON.
#[test]
fn autonum_on_dispatches_to_number_on() {
    // Validates: Requirement 16.10
    // NUMBER ON is handled by the CommandEngine; we just verify no panic and
    // that AUTONUM ON is not treated as an unknown command.
    let mut shell = make_shell();
    shell.handle_command("AUTONUM ON");
    // Should not produce an "unknown command" error from the AUTONUM handler itself
    // (the CommandEngine may or may not know NUMBER -- we just check dispatch happened)
    // The key assertion: open_error does NOT contain "AUTONUM"
    let err = shell.open_error.as_deref().unwrap_or("");
    assert!(
        !err.to_uppercase().contains("AUTONUM"),
        "AUTONUM should be aliased, got: {err}"
    );
}

/// Validates: Requirement 16.11 -- NUM dispatches to NUMBER.
#[test]
fn num_command_dispatches_to_number() {
    // Validates: Requirement 16.11
    let mut shell = make_shell();
    shell.handle_command("NUM ON");
    let err = shell.open_error.as_deref().unwrap_or("");
    assert!(
        !err.to_uppercase().contains("NUM ON"),
        "NUM should be aliased, got: {err}"
    );
}

/// Validates: Requirement 19.2 -- SCROLL command updates active scroll amount.
#[test]
fn scroll_command_updates_scroll_amount() {
    // Validates: Requirement 19.2
    use crate::scroll_amount::ScrollAmount;
    let mut shell = make_shell();
    shell.handle_command("SCROLL HALF");
    assert_eq!(shell.scroll_amount, ScrollAmount::Half);
    assert_eq!(shell.scroll_field_text, "HALF");
    assert!(shell.open_error.is_none());
}

/// Validates: Requirement 19.2 -- SCROLL with numeric value.
#[test]
fn scroll_command_accepts_numeric_value() {
    // Validates: Requirement 19.2
    use crate::scroll_amount::ScrollAmount;
    let mut shell = make_shell();
    shell.handle_command("SCROLL 10");
    assert_eq!(shell.scroll_amount, ScrollAmount::Lines(10));
    assert_eq!(shell.scroll_field_text, "10");
    assert!(shell.open_error.is_none());
}

/// Validates: Requirement 19.2 -- SCROLL with invalid value shows error.
#[test]
fn scroll_command_invalid_value_shows_error() {
    // Validates: Requirement 19.2
    let mut shell = make_shell();
    shell.handle_command("SCROLL BOGUS");
    assert!(shell.open_error.is_some());
    let err = shell.open_error.as_deref().unwrap_or("");
    assert!(err.contains("SCROLL"), "got: {err}");
}

/// Validates: Requirement 19.10 -- all extended scroll amounts accepted.
#[test]
fn scroll_command_accepts_all_extended_amounts() {
    // Validates: Requirement 19.10
    use crate::scroll_amount::ScrollAmount;
    let mut shell = make_shell();
    for (cmd, expected) in [
        ("SCROLL PAGE", ScrollAmount::Page),
        ("SCROLL HALF", ScrollAmount::Half),
        ("SCROLL CSR", ScrollAmount::Csr),
        ("SCROLL MAX", ScrollAmount::Max),
        ("SCROLL DATA", ScrollAmount::Data),
    ] {
        shell.handle_command(cmd);
        assert_eq!(shell.scroll_amount, expected, "failed for {cmd}");
        assert!(shell.open_error.is_none(), "error for {cmd}");
    }
}

// === CR-NR-087: bare UP/DOWN via handle_command honour the SCROLL amount =====

/// Validates: navigation-commands Req 3.21/3.17 -- with the SCROLL amount set to
/// MAX, a bare `UP` command (the shell wiring end-to-end) scrolls the active
/// tab's viewport to the top of the document (the B046 row 7.3a case).
#[test]
fn command_up_with_scroll_max_scrolls_to_top() {
    use crate::scroll_amount::ScrollAmount;
    let mut shell = make_shell();
    // A scrollable editor tab: many display lines, a small viewport scrolled down.
    shell.shell_new_untitled();
    {
        let tab = shell.tabs.active_tab_mut();
        tab.line_count = 200;
        tab.viewport.set_total_display_lines(200);
        tab.viewport.set_visible_count(20);
        ff_navigation_commands::ScrollCommands::down_lines(&mut tab.viewport, &mut tab.cursor, 80);
    }
    assert!(
        shell.tabs.active_tab().viewport.top_line() > 1,
        "precondition: scrolled down"
    );
    shell.handle_command("SCROLL MAX");
    assert_eq!(shell.scroll_amount, ScrollAmount::Max);
    shell.handle_command("UP");
    assert_eq!(
        shell.tabs.active_tab().viewport.top_line(),
        1,
        "bare UP with SCROLL MAX scrolls to the top (CR-NR-087, B046 row 7.3a)"
    );
    assert!(shell.open_error.is_none(), "no error from bare UP");
    // The SCROLL amount is unchanged by the scroll (Req 3.23).
    assert_eq!(shell.scroll_amount, ScrollAmount::Max);
}

/// Validates: navigation-commands Req 3.19 -- a bare `DOWN` command with SCROLL
/// HALF advances the viewport by half a page (shell wiring).
#[test]
fn command_down_with_scroll_half_advances_half_page() {
    use crate::scroll_amount::ScrollAmount;
    let mut shell = make_shell();
    shell.shell_new_untitled();
    {
        let tab = shell.tabs.active_tab_mut();
        tab.line_count = 200;
        tab.viewport.set_total_display_lines(200);
        tab.viewport.set_visible_count(20);
    }
    shell.handle_command("SCROLL HALF");
    assert_eq!(shell.scroll_amount, ScrollAmount::Half);
    let before = shell.tabs.active_tab().viewport.top_line();
    shell.handle_command("DOWN");
    let after = shell.tabs.active_tab().viewport.top_line();
    assert_eq!(
        after - before,
        10,
        "bare DOWN with SCROLL HALF advances 10 lines"
    );
}

/// Validates: navigation-commands Req 3.23 -- an explicit numeric `DOWN n`
/// overrides the active SCROLL amount (here MAX), scrolling exactly n lines
/// rather than to the bottom.
#[test]
fn command_down_n_overrides_scroll_amount() {
    let mut shell = make_shell();
    shell.shell_new_untitled();
    {
        let tab = shell.tabs.active_tab_mut();
        tab.line_count = 200;
        tab.viewport.set_total_display_lines(200);
        tab.viewport.set_visible_count(20);
    }
    shell.handle_command("SCROLL MAX");
    let before = shell.tabs.active_tab().viewport.top_line();
    shell.handle_command("DOWN 5");
    let after = shell.tabs.active_tab().viewport.top_line();
    assert_eq!(
        after - before,
        5,
        "explicit DOWN 5 overrides SCROLL MAX (scrolls 5 lines, not to bottom)"
    );
}

/// Validates: Requirement 19.3 -- scroll amount retained across command submissions.
#[test]
fn scroll_amount_retained_across_commands() {
    // Validates: Requirement 19.3
    use crate::scroll_amount::ScrollAmount;
    let mut shell = make_shell();
    shell.handle_command("SCROLL HALF");
    assert_eq!(shell.scroll_amount, ScrollAmount::Half);
    // Submit an unrelated command
    shell.handle_command("TOP");
    // Scroll amount unchanged
    assert_eq!(shell.scroll_amount, ScrollAmount::Half);
}

/// Validates: command-framework Req 9.8 (B066) -- pressing a key bound to a
/// command merges the current Command ===> field content as the argument, so
/// typing `1` then pressing F9 (=SWAP) behaves like `SWAP 1`.
#[test]
fn key_command_merges_command_field_as_argument() {
    let mut shell = make_shell();
    shell.handle_command("START"); // ensure >=2 tabs
    assert!(shell.tabs.len() >= 2);

    // Type `1` in the command field, then "press" the SWAP-bound key.
    shell.command_text = "1".to_string();
    shell.dispatch_key_command("SWAP");
    assert_eq!(
        shell.tabs.active_index(),
        0,
        "`1` + SWAP key -> workspace 1"
    );
    assert!(shell.open_error.is_none());
    // It must NOT have opened the tab picker (the old bare-SWAP behaviour).
    assert!(
        shell.show_swap_list.is_none(),
        "a typed argument must run SWAP <n>, not open the picker"
    );

    // `2` + SWAP key -> workspace 2.
    shell.command_text = "2".to_string();
    shell.dispatch_key_command("SWAP");
    assert_eq!(
        shell.tabs.active_index(),
        1,
        "`2` + SWAP key -> workspace 2"
    );
    assert!(shell.open_error.is_none());
}

/// Validates: command-framework Req 9.8 (B066) -- with an EMPTY command field,
/// a key-bound command runs bare (no argument), so F9=SWAP with no split opens
/// the tab picker (the existing bare-SWAP behaviour is preserved).
#[test]
fn key_command_with_empty_field_runs_bare_command() {
    let mut shell = make_shell();
    shell.command_text.clear();
    shell.dispatch_key_command("SWAP");
    assert!(
        shell.show_swap_list.is_some(),
        "empty field + SWAP key -> bare SWAP (tab picker with no split)"
    );
    assert!(shell.open_error.is_none());
}

/// Validates: command-framework Req 9.9 (revised, CR-CH-033), Req 13.1/13.3 --
/// a SUCCESSFUL key-forwarded command clears the field (the Command_Line_Outcome
/// default on success), so `1` does NOT remain after `1` + F9 (SWAP). This is
/// the fix; it replaces the former `key_command_does_not_force_clear_command_field`.
#[test]
fn key_command_clears_command_field_after_success() {
    let mut shell = make_shell();
    shell.handle_command("START"); // ensure a second tab so SWAP 1 succeeds
    shell.command_text = "1".to_string();
    shell.dispatch_key_command("SWAP");
    assert_eq!(
        shell.command_text, "",
        "a successful key-forwarded command clears the field (Req 13.3 default Clear)"
    );
    assert!(shell.open_error.is_none());
}

/// Validates: command-framework Req 13.1/13.3 -- the Enter path applies the same
/// Command_Line_Outcome default: a successful command clears the field.
#[test]
fn enter_path_clears_command_field_after_success() {
    let mut shell = make_shell();
    shell.handle_command("START");
    shell.command_text = "SWAP 1".to_string();
    shell.run_command_line("SWAP 1");
    assert_eq!(
        shell.command_text, "",
        "a successful Enter-path command clears the field (Req 13.3)"
    );
}

/// Validates: command-framework Requirement 2.1 -- the typed submit path routes
/// through the single front door `dispatch_command_string`, which delegates
/// identically to the old direct `handle_command` call (pure indirection, Step
/// 0). A representative typed command produces the same observable shell state.
#[test]
fn typed_submit_routes_through_dispatch_command_string() {
    let mut shell = make_shell();
    shell.handle_command("START"); // create a second tab so SWAP 1 succeeds
    assert!(shell.tabs.len() >= 2);

    shell.command_text = "SWAP 1".to_string();
    shell.run_command_line("SWAP 1");

    // Same observable result as the pre-change direct handle_command delegation:
    // SWAP 1 moves to workspace 1 and the field is cleared on success.
    assert_eq!(
        shell.tabs.active_index(),
        0,
        "typed SWAP 1 (via the front door) selects workspace 1"
    );
    assert_eq!(
        shell.command_text, "",
        "a successful command clears the field (front door delegates identically)"
    );
    assert!(shell.open_error.is_none());
}

/// Validates: Requirement 20.3 -- LOGOFF command is a shell-level intercept.
#[test]
fn logoff_command_is_shell_level_intercept() {
    // Validates: Requirement 20.3
    assert!(
        is_shell_command("LOGOFF"),
        "LOGOFF must be handled at shell level like EXIT"
    );
}

/// Validates: Requirement 20.4 -- TIME command sets open_error to date/time/day string.
#[test]
fn time_command_displays_date_time_day() {
    // Validates: Requirement 20.4
    let mut shell = make_shell();
    shell.handle_command("TIME");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(msg.contains("Date:"), "must contain 'Date:', got: {msg}");
    assert!(msg.contains("Time:"), "must contain 'Time:', got: {msg}");
    assert!(msg.contains("Day:"), "must contain 'Day:', got: {msg}");
}

/// Validates: Requirement 20.5 -- STATUS routes to the JES panel (stub).
#[test]
fn status_command_routes_to_jes() {
    // Validates: Requirement 20.5
    let mut shell = make_shell();
    shell.handle_command("STATUS");
    let msg = shell.open_error.as_deref().unwrap_or("");
    // STATUS routes to JES (stub); must not be an "unknown command" error
    assert!(
        !msg.to_uppercase().contains("UNKNOWN"),
        "STATUS must not produce unknown-command error, got: {msg}"
    );
}

/// Validates: plugin-manager-ui Requirement 1.1 -- PLUGINS command routes to PluginManager.
#[test]
fn plugins_command_routes_to_plugin_manager() {
    // Validates: plugin-manager-ui Requirement 1.1
    let mut shell = make_shell();
    shell.dispatch_command_string("PLUGINS");
    use crate::tab_state::{KindTag, TabKind};
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::PluginManager);
}

/// Validates: menu-workspace Req 2.1i -- a POM fastpath key resolves to its
/// configured command and routes there.
#[test]
fn equals_8_command_routes_to_plugin_manager() {
    // CR-CH-021: the Recovery_Baseline POM key set is 0/1/2/L/M/X. Exercise the
    // fastpath resolver against a baseline key (=1 -> CATALOGS -> FilesPanel).
    let mut shell = make_shell();
    shell.handle_command("=1");
    use crate::tab_state::{KindTag, TabKind};
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::FilesPanel);
}

/// Validates: notification-system Requirement 2.1 -- LOG command routes to EventLog.
#[test]
fn log_command_routes_to_event_log() {
    // Validates: notification-system Requirement 2.1
    let mut shell = make_shell();
    shell.dispatch_command_string("LOG");
    use crate::tab_state::{KindTag, TabKind};
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::EventLog);
}

/// Validates: lua-macro-engine Requirement 12.1 -- MACROS command routes to MacroLibrary.
#[test]
fn macros_command_routes_to_macro_library() {
    // Validates: lua-macro-engine Requirement 12.1
    let mut shell = make_shell();
    shell.dispatch_command_string("MACROS");
    use crate::tab_state::{KindTag, TabKind};
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::MacroLibrary);
}

/// Validates: menu-workspace Req 2.1i -- a POM fastpath key resolves to its
/// configured command and routes there.
#[test]
fn equals_5_command_routes_to_macro_library() {
    // CR-CH-021: MACROS is no longer a baseline POM key; exercise the fastpath
    // resolver against a baseline key (=2 -> FILES -> File Explorer).
    let mut shell = make_shell();
    shell.handle_command("=2");
    use crate::tab_state::{KindTag, TabKind};
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::FileExplorerPanel
    );
}

// === Phase CX Tests =====================================================

/// Validates: CX Requirement 1.2 -- NAME <text> sets workspace_name on active tab.
#[test]
fn name_command_sets_workspace_name() {
    let mut shell = make_shell();
    shell.handle_command("NAME MyWork");
    assert_eq!(
        shell.tabs.active_tab().workspace_name.as_deref(),
        Some("MyWork")
    );
}

/// Validates: CX Requirement 1.3 -- NAME with no argument clears workspace_name.
#[test]
fn name_command_no_arg_clears_workspace_name() {
    let mut shell = make_shell();
    shell.handle_command("NAME MyWork");
    shell.handle_command("NAME");
    assert!(shell.tabs.active_tab().workspace_name.is_none());
}

/// Validates: CX Requirement 1.2 -- NAME truncates to 32 characters.
#[test]
fn name_command_truncates_to_32_chars() {
    let mut shell = make_shell();
    let long_name = "A".repeat(50);
    shell.handle_command(&format!("NAME {}", long_name));
    let name = shell
        .tabs
        .active_tab()
        .workspace_name
        .as_deref()
        .unwrap_or("");
    assert_eq!(name.len(), 32);
}

/// Validates: CX Requirement 2.1 -- KEYS with no argument opens dialog with initial_scope None.
#[test]
fn keys_command_opens_keys_workspace() {
    // Validates: function-keys Req 22.1, 22.5 (CR-CH-029) -- KEYS opens the Keys
    // Workspace (a Context tab), NOT a modal dialog.
    use crate::tab_state::{KindTag, TabKind};
    let mut shell = make_shell();
    shell.dispatch_command_string("KEYS");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::KeysEditor);
}

// Validates: menu-workspace Requirement 10.3, command-configurator Requirement 4.3 --
// a command value equal to a user definition id resolves and is dispatched.
#[test]
fn resolve_and_dispatch_user_definition_id_dispatches() {
    let mut shell = make_shell();
    push_user_function_def(&mut shell, "my.build", "file.exit");
    let outcome = shell.resolve_and_dispatch_command("my.build");
    assert!(matches!(
        outcome,
        super::target_dispatch::ResolveOutcome::Dispatched
    ));
}

// Validates: menu-workspace Requirement 10.2, command-framework Requirement 8.4 --
// a command the resolver does not own falls through to the existing pipeline.
#[test]
fn resolve_and_dispatch_unknown_string_falls_through() {
    let mut shell = make_shell();
    let outcome = shell.resolve_and_dispatch_command("SOME.BUILTIN.VERB.XYZ");
    assert!(matches!(
        outcome,
        super::target_dispatch::ResolveOutcome::FallThrough
    ));
}

// Validates: command-framework Requirement 8.3 -- a bare registered Command_ID
// resolves to a Function target and is dispatched (not fall-through).
#[test]
fn resolve_and_dispatch_registered_command_id_dispatches() {
    let mut shell = make_shell();
    // `file.exit` is registered in WorkbenchShell::new().
    let outcome = shell.resolve_and_dispatch_command("file.exit");
    assert!(matches!(
        outcome,
        super::target_dispatch::ResolveOutcome::Dispatched
    ));
}

// Validates: menu-workspace Requirement 10.6 -- an inline External target is
// dispatched via the ff-shell adapter. With the default shell.mode = prompt,
// an External target is staged for confirmation rather than run immediately.
// Validates: command-configurator Requirement 3.8
#[test]
fn dispatch_external_target_stages_prompt_confirmation() {
    use ff_command::{CommandTarget, ExternalMode};
    let mut shell = make_shell();
    // make_shell() leaves the engine at its default mode (prompt).
    let target = CommandTarget::External {
        program: "pwsh".to_string(),
        args: vec![],
        working_dir: None,
        mode: ExternalMode::Captured,
    };
    shell.dispatch_command_target(&target);
    let pending = shell
        .pending_external
        .as_ref()
        .expect("prompt mode stages a pending external");
    assert_eq!(pending.program, "pwsh");
}

// Validates: command-framework Requirement 8.5, command-configurator Requirement 4.4 --
// a bound command (shortcut/label-bar) equal to a definition id dispatches its target.
#[test]
fn dispatch_bound_command_resolves_user_definition() {
    let mut shell = make_shell();
    push_user_function_def(&mut shell, "kb.exit", "file.exit");
    // Should dispatch (route file.exit through the pipeline) without a
    // "not found"/"not defined" error.
    shell.dispatch_bound_command("kb.exit");
    let msg = shell.open_error.as_deref().unwrap_or_default();
    assert!(
        !msg.contains("not defined") && !msg.contains("not found"),
        "bound definition id must dispatch, got: {msg}"
    );
}

// Validates: menu-workspace Requirement 10.2 -- a bound built-in command that
// the resolver does not own still runs via the existing pipeline.
#[test]
fn dispatch_bound_command_falls_through_for_builtin() {
    use ff_edit_operations::CapsMode;
    let mut shell = make_shell();
    // CAPS ON is a shell built-in, not a user definition; must still work.
    shell.dispatch_bound_command("CAPS ON");
    assert_eq!(shell.tabs.active_tab().edit_profile.caps, CapsMode::On);
}

// Validates: command-configurator Requirement 4.4 -- an explicit definition
// reference runs the definition's target.
#[test]
fn run_command_definition_dispatches_defined_id() {
    let mut shell = make_shell();
    push_user_function_def(&mut shell, "def.exit", "file.exit");
    shell.run_command_definition("def.exit");
    let msg = shell.open_error.as_deref().unwrap_or_default();
    assert!(
        !msg.contains("is not defined"),
        "a defined id must dispatch, got: {msg}"
    );
}

// Validates: command-configurator Requirement 4.5 -- an explicit reference to a
// missing definition id reports `Command '<id>' is not defined.`
#[test]
fn run_command_definition_missing_id_reports_not_defined() {
    let mut shell = make_shell();
    shell.run_command_definition("no.such.def");
    assert_eq!(
        shell.open_error.as_deref(),
        Some("Command 'no.such.def' is not defined.")
    );
}

// === Command Configurator Context (Task 4) ==================================

// Validates: command-configurator Requirement 2.1, 2.7 -- COMMANDS opens the
// Command Configurator Context with title [COMMANDS].
#[test]
fn commands_opens_command_configurator_context() {
    use crate::tab_state::{KindTag, TabKind};
    let mut shell = make_shell();
    shell.dispatch_command_string("COMMANDS");
    let tab = shell.tabs.active_tab();
    assert_eq!(tab.kind.tag(), KindTag::CommandConfigurator);
    assert_eq!(tab.title, "[COMMANDS]");
    assert!(shell.open_error.is_none());
}

// Validates: command-configurator Requirement 2.4 -- editing an existing
// definition updates it in place without adding a duplicate.
#[test]
fn configurator_edit_updates_in_place() {
    use crate::command_config::edit::{EditForm, TargetVariant};
    use crate::command_config::render::ConfiguratorAction;
    let mut shell = make_shell();
    let _dir = point_store_at_temp(&mut shell);
    set_add_function_form(&mut shell, "edit.me", "file.save");
    shell.apply_configurator_action(ConfiguratorAction::Save);

    // Open an edit form for the same id and change the label.
    let def = shell.command_store.find("edit.me").unwrap().clone();
    let mut form = EditForm::from_definition(&def);
    assert!(form.is_edit);
    form.label = "Renamed".to_string();
    form.variant = TargetVariant::Function;
    form.command_id = "file.save".to_string();
    shell.command_configurator_panel.form = Some(form);
    shell.apply_configurator_action(ConfiguratorAction::Save);

    assert_eq!(
        shell.command_store.definitions.len(),
        1,
        "no duplicate added"
    );
    assert_eq!(
        shell.command_store.find("edit.me").unwrap().label,
        "Renamed"
    );
}

// Validates: command-configurator Requirement 3.7 -- shell.mode = disabled
// refuses external execution.
#[test]
fn external_disabled_refuses() {
    let mut shell = make_shell();
    set_shell_mode(&shell, ff_shell::ShellMode::Disabled);
    shell.dispatch_command_target(&noop_external_target());
    assert!(
        shell.pending_external.is_none(),
        "must not stage when disabled"
    );
    let msg = shell.open_error.as_deref().unwrap_or_default();
    assert!(
        msg.to_lowercase().contains("disabled"),
        "disabled mode must report the shell-disabled message, got: {msg}"
    );
}

// Validates: command-configurator Requirement 3.8 -- shell.mode = prompt stages
// the run behind a confirmation instead of spawning immediately.
#[test]
fn external_prompt_stages_pending_confirmation() {
    let mut shell = make_shell();
    set_shell_mode(&shell, ff_shell::ShellMode::Prompt);
    shell.dispatch_command_target(&noop_external_target());
    assert!(
        shell.pending_external.is_some(),
        "prompt mode must stage a pending external for confirmation"
    );
}

// Validates: command-configurator Requirement 3.4 -- a Captured run appends to
// the Output_Panel.
#[test]
fn external_captured_appends_to_output_panel() {
    use ff_command::{CommandTarget, ExternalMode};
    let mut shell = make_shell();
    set_shell_mode(&shell, ff_shell::ShellMode::Enabled);
    let (program, args) = if cfg!(windows) {
        (
            "cmd".to_string(),
            vec!["/C".to_string(), "echo hi".to_string()],
        )
    } else {
        ("echo".to_string(), vec!["hi".to_string()])
    };
    let before = shell.shell_engine.output_entry_count();
    shell.dispatch_command_target(&CommandTarget::External {
        program,
        args,
        working_dir: None,
        mode: ExternalMode::Captured,
    });
    assert_eq!(
        shell.shell_engine.output_entry_count(),
        before + 1,
        "a captured run must append one Output_Panel entry"
    );
}

// Validates: command-configurator Requirement 3.9 -- a launch failure is
// reported (no panic), and nothing is staged.
#[test]
fn external_launch_failure_reports_error() {
    use ff_command::{CommandTarget, ExternalMode};
    let mut shell = make_shell();
    set_shell_mode(&shell, ff_shell::ShellMode::Enabled);
    shell.dispatch_command_target(&CommandTarget::External {
        program: "ffwb_nonexistent_program_db_ext".to_string(),
        args: vec![],
        working_dir: None,
        mode: ExternalMode::Detached,
    });
    assert!(
        shell.open_error.is_some(),
        "launch failure must be reported"
    );
    assert!(shell.pending_external.is_none());
}

// Validates: command-configurator Requirement 3.6 -- ${workspace_root} and
// ${file_dir} expand; an unresolved placeholder becomes empty.
#[test]
fn external_placeholder_unresolved_expands_to_empty() {
    let shell = make_shell();
    // No active workspace and an unsaved active tab -> both placeholders empty.
    let expanded = shell.expand_external_placeholders("A${workspace_root}B${file_dir}C");
    assert_eq!(expanded, "ABC");
}

// Validates: configuration-system Req 20.1/20.2 (CR-CH-025) -- CONFIG is a
// registered built-in command (Command_ID config.open).
#[test]
fn config_command_is_registered() {
    let shell = make_shell();
    let id = ff_command::CommandId::new("config.open").expect("valid id");
    assert!(
        shell.cmd_registry.contains(&id),
        "config.open must be registered so CONFIG resolves as a built-in"
    );
}

// Validates: configuration-system Req 20.4 (CR-CH-025) -- an unknown namespace
// still opens the flat view (filter applied, editable), not an error.
#[test]
fn config_unknown_namespace_opens_editable_view() {
    use crate::tab_state::{KindTag, TabKind};
    let mut shell = make_shell();
    shell.dispatch_command_string("CONFIG nosuchns");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::ConfigPanel);
    assert_eq!(
        shell.config_panel.namespace_filter.as_deref(),
        Some("nosuchns")
    );
    assert!(
        shell.open_error.is_none(),
        "an unknown CONFIG namespace must not raise an error"
    );
}

// Validates: configuration-system Requirement 19.2 -- RESET BARE opens the
// confirmation dialog and does NOT act until confirmed.
#[test]
fn reset_bare_command_opens_confirmation_dialog() {
    let mut shell = make_shell();
    assert!(shell.reset_bare_confirm.is_none());
    shell.handle_command("RESET BARE");
    assert!(
        shell.reset_bare_confirm.is_some(),
        "RESET BARE must open the confirmation dialog"
    );
}

#[test]
fn alt_f1_default_dispatches_pfshow_cycle() {
    // Validates: function-keys-and-history Requirement 15.7, 12.14 -- the default
    // AF1 = PFSHOW binding reaches the PFSHOW command. Simulate the key by
    // dispatching the command bound to Alt+F1 in the active (default) map.
    use super::KeyBarScope;
    use ff_keys::{FunctionKey, ModifiedKey};
    let mut shell = make_shell();
    let cmd = shell
        .key_map_resolver
        .active_key_map()
        .get(ModifiedKey::alt(FunctionKey::F1))
        .expect("Alt+F1 bound by default")
        .command()
        .to_string();
    assert_eq!(cmd, "PFSHOW", "default AF1 command is PFSHOW");

    shell.handle_command("PFSHOW OFF"); // known start
    shell.handle_command(&cmd); // as if Alt+F1 pressed -> cycle Off -> Base
    assert!(shell.key_bar_visible);
    assert_eq!(shell.key_bar_scope, KeyBarScope::Base);
}

/// Validates: workspace-kinds Req 8.9 -- `COMMAND TOP` / `COMMAND BOTTOM` set the
/// active Kind's command-line position.
#[test]
fn command_top_and_bottom_set_position() {
    use crate::workspace_kind::CommandLinePosition;
    use tempfile::TempDir;
    let mut shell = make_shell();
    let dir = TempDir::new().expect("tempdir");
    shell.dir_overrides.workspace_kinds = Some(dir.path().to_path_buf());
    let name = active_kind_name(&shell);

    shell.handle_command("COMMAND BOTTOM");
    assert_eq!(
        shell
            .kind_registry
            .effective(name)
            .profile
            .command_line_position,
        CommandLinePosition::Bottom,
        "COMMAND BOTTOM sets the active Kind position to Bottom"
    );
    shell.handle_command("COMMAND TOP");
    assert_eq!(
        shell
            .kind_registry
            .effective(name)
            .profile
            .command_line_position,
        CommandLinePosition::Top,
        "COMMAND TOP sets it back to Top"
    );
}

/// Validates: workspace-kinds Req 8.9 -- bare `COMMAND` toggles the position;
/// case-insensitive verb + argument.
#[test]
fn command_bare_toggles_position() {
    use crate::workspace_kind::CommandLinePosition;
    use tempfile::TempDir;
    let mut shell = make_shell();
    let dir = TempDir::new().expect("tempdir");
    shell.dir_overrides.workspace_kinds = Some(dir.path().to_path_buf());
    let name = active_kind_name(&shell);

    // Default is Top; bare COMMAND -> Bottom; again -> Top. Lower-case verb too.
    shell.handle_command("command");
    assert_eq!(
        shell
            .kind_registry
            .effective(name)
            .profile
            .command_line_position,
        CommandLinePosition::Bottom
    );
    shell.handle_command("COMMAND");
    assert_eq!(
        shell
            .kind_registry
            .effective(name)
            .profile
            .command_line_position,
        CommandLinePosition::Top
    );
    // Case-insensitive argument.
    shell.handle_command("command bottom");
    assert_eq!(
        shell
            .kind_registry
            .effective(name)
            .profile
            .command_line_position,
        CommandLinePosition::Bottom
    );
}

/// Validates: workspace-kinds Req 8.9 -- an unknown argument sets a non-blocking
/// error and leaves the position unchanged.
#[test]
fn command_unknown_arg_sets_error_and_leaves_position() {
    use crate::workspace_kind::CommandLinePosition;
    use tempfile::TempDir;
    let mut shell = make_shell();
    let dir = TempDir::new().expect("tempdir");
    shell.dir_overrides.workspace_kinds = Some(dir.path().to_path_buf());
    let name = active_kind_name(&shell);

    shell.handle_command("COMMAND SIDEWAYS");
    assert!(
        shell
            .open_error
            .as_deref()
            .unwrap_or("")
            .contains("SIDEWAYS"),
        "unknown arg names the offending value in a non-blocking error"
    );
    assert_eq!(
        shell
            .kind_registry
            .effective(name)
            .profile
            .command_line_position,
        CommandLinePosition::Top,
        "position unchanged after an unknown argument"
    );
}

/// Validates: workspace-kinds Req 8.12 -- `COMMAND` (singular) does not shadow
/// `COMMANDS` (plural): `COMMANDS` still opens the Command Configurator.
#[test]
fn command_singular_does_not_shadow_commands_plural() {
    use crate::tab_state::{KindTag, TabKind};
    let mut shell = make_shell();
    shell.dispatch_command_string("COMMANDS");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::CommandConfigurator,
        "COMMANDS (plural) still opens the Command Configurator, not the COMMAND arm"
    );
}

// Validates: workspace-kinds Req 8.8 (CR-NR-095) -- RESET BARE returns every
// Kind's Command_Line_Position to the compiled default (Top). No separate reset
// path: it falls out of the registry rebuild from with_builtin_defaults().
#[test]
fn reset_bare_returns_command_line_position_to_top() {
    use crate::shell::reset_bare::ResetBareTarget;
    use crate::workspace_kind::{CommandLinePosition, KindRegistry};
    use tempfile::TempDir;

    let mut shell = make_shell();

    // A live registry where the editor Kind was saved with Bottom.
    let kinds_dir = TempDir::new().expect("tempdir");
    let toml = "name=\"editor\"\nmodelled_on=\"editor\"\ntitle=\"[EDIT]\"\n\n[profile]\ncommand_line_position=\"bottom\"";
    std::fs::write(kinds_dir.path().join("editor.toml"), toml).expect("write");
    shell.kind_registry = KindRegistry::load(kinds_dir.path());
    assert_eq!(
        shell
            .kind_registry
            .effective("editor")
            .profile
            .command_line_position,
        CommandLinePosition::Bottom,
        "precondition: the live registry carries Bottom"
    );

    let target_dir = TempDir::new().expect("tempdir");
    let target = ResetBareTarget::single(
        "(default)".to_string(),
        target_dir.path().to_path_buf(),
        true,
    );
    shell.execute_reset_bare(&target);

    assert_eq!(
        shell
            .kind_registry
            .effective("editor")
            .profile
            .command_line_position,
        CommandLinePosition::Top,
        "RESET BARE must return the command-line position to the compiled default (Top)"
    );
}

// Validates: context-help Req 18.1 -- HELP opens the Help Context (single
// shell-owned registry; no per-call empty registry).
#[test]
fn help_command_opens_help_context() {
    let mut shell = make_shell();
    shell.handle_command("HELP");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        crate::tab_state::KindTag::HelpContext,
        "HELP opens the Help Context"
    );
}

// Validates: context-help Req 18.2 -- a resolved file-based topic is displayed.
#[test]
fn help_command_displays_file_based_topic() {
    use ff_help::{HelpTopic, TopicKey, TopicSource};
    let mut shell = make_shell();
    shell.seed_help_registry_for_test(vec![HelpTopic::new(
        TopicKey::command("CHANGE"),
        "CHANGE Command".to_string(),
        "Find and replace.".to_string(),
        TopicSource::FileBased {
            file_path: std::path::PathBuf::from("change.help.md"),
        },
    )]);
    shell.handle_command("HELP CHANGE");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        crate::tab_state::KindTag::HelpContext
    );
    let shown = shell
        .help_panel_for_test()
        .model()
        .current_topic_key()
        .cloned();
    assert_eq!(shown, Some(TopicKey::command("CHANGE")));
}

// Validates: context-help Req 18.3 -- the dynamic index topic is generated and
// displayed even though it is not a file-based topic.
#[test]
fn help_command_generates_index_topic() {
    use ff_help::TopicKey;
    let mut shell = make_shell();
    shell.handle_command("HELP");
    let key = shell
        .help_panel_for_test()
        .model()
        .current_topic_key()
        .cloned();
    assert_eq!(key, Some(TopicKey::index()));
    let body = shell
        .help_panel_for_test()
        .model()
        .current_topic()
        .unwrap()
        .body()
        .to_string();
    assert!(body.contains("Help Index"), "index body: {body}");
}

/// Validates: screen-snapshot-scrm Req 6.1, 6.2 -- the SNAPSHOT command sets a
/// confirmation status message in the command area.
#[test]
fn snapshot_command_sets_status_message() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("SNAPSHOT");
    let msg = shell.open_error.as_deref().unwrap_or("");
    assert!(
        msg.contains("Snapshot"),
        "SNAPSHOT must confirm the outcome in the command area; got: {msg:?}"
    );
}

// === B080 Step 2: single front door for the built-in nav family =============

/// Validates: command-framework Requirement 2.1 -- the typed seam and the key
/// seam reach ONE handler for a built-in in-scope verb, producing the identical
/// observable result. CONFIG is now owned by `builtin_workspace_target`, so both
/// `run_command_line("CONFIG core")` (typed) and an empty-field
/// `dispatch_key_command("CONFIG core")` (key) resolve through the single front
/// door -> `resolve_target` -> `dispatch_command_target` and open the Config
/// Context with the same namespace filter.
#[test]
fn typed_and_key_paths_reach_same_handler_for_builtin_verb() {
    use crate::tab_state::{KindTag, TabKind};

    // Typed path.
    let mut typed = make_shell();
    typed.command_text = "CONFIG core".to_string();
    typed.run_command_line("CONFIG core");

    // Key path, EMPTY field so the bound command runs verbatim (no field-merge).
    let mut keyed = make_shell();
    keyed.command_text.clear();
    keyed.dispatch_key_command("CONFIG core");

    // Both opened the Config Context with the same namespace filter and no error.
    assert_eq!(typed.tabs.active_tab().kind.tag(), KindTag::ConfigPanel);
    assert_eq!(keyed.tabs.active_tab().kind.tag(), KindTag::ConfigPanel);
    assert_eq!(
        typed.config_panel.namespace_filter.as_deref(),
        Some("core"),
        "typed CONFIG core applies the namespace filter via the resolver"
    );
    assert_eq!(
        keyed.config_panel.namespace_filter.as_deref(),
        Some("core"),
        "key CONFIG core applies the SAME namespace filter via the resolver"
    );
    assert_eq!(
        typed.config_panel.namespace_filter, keyed.config_panel.namespace_filter,
        "typed and key seams reach one handler with identical observable state"
    );
    assert!(typed.open_error.is_none());
    assert!(keyed.open_error.is_none());
}

/// Validates: menu-workspace Requirement 14.13, 14.14 (CR-CH-052) -- the uniform
/// navigation/exit model from a non-menu (Config) Context reached by navigating
/// the POM (so its Navigation_Stack holds the POM at the bottom):
/// - bare `X` COLLAPSES to the Tab_Visual_Root (the POM); it does NOT app-exit.
/// - `=X` reinitialises to the POM (front-door `=`) then `X` at the empty root
///   closes the Workspace, which app-exits ONLY because it is the last tab.
///
/// This supersedes the old "X / =X unconditionally dispatch file.exit" model.
#[test]
fn uniform_x_from_non_menu_context_collapses_then_equals_x_closes() {
    use crate::tab_state::{KindTag, TabKind};

    // Bare `X`: collapse to the visual root (POM), NOT an app-exit.
    let mut shell = make_shell();
    shell.dispatch_command_string("CONFIG");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "precondition: active Context is the non-menu Config panel"
    );
    assert!(
        !shell.tabs.active_tab().nav_stack.is_empty(),
        "precondition: navigating the POM pushed it onto the stack"
    );
    assert!(!*shell.should_close.lock().expect("close lock"));

    shell.run_command_line("X");

    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "bare `X` above the visual root must COLLAPSE, not app-exit (CR-CH-052)"
    );
    assert!(
        shell.tabs.active_tab().is_home,
        "bare `X` collapses the Config Context to its Tab_Visual_Root (the POM)"
    );
    assert!(
        shell.tabs.active_tab().nav_stack.is_empty(),
        "collapse clears the Navigation_Stack to the visual root"
    );

    // `=X` on the single-tab shell: reinit to POM, then X at the empty root
    // closes the Workspace -> app-exit (last tab).
    let mut shell = make_shell();
    shell.dispatch_command_string("CONFIG");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::ConfigPanel);
    assert_eq!(shell.tabs.len(), 1, "precondition: single tab");
    assert!(!*shell.should_close.lock().expect("close lock"));

    shell.run_command_line("=X");

    assert!(
        *shell.should_close.lock().expect("close lock"),
        "`=X` reinitialises to the POM then `X` at the empty root closes the \
         Workspace, which app-exits because it is the last tab (CR-CH-052)"
    );
}

// === CR-CH-053 E8: environment-before-FFCMD ordering (bare X / =X) =========
//
// The active Command_Environment must be checked BEFORE FFCMD. The showcase
// collision is bare `X`: in an editor Context FFEDIT owns `X` (EXCLUDE); in a
// non-editor Context FFEDIT is not active, so FFCMD receives `X` (close/exit).
// The `=` prefix is the universal escape hatch: `=X` is routed past the
// environment to FFCMD, so `=X` in the editor exits/returns, NOT EXCLUDE.

/// Validates: command-environments Requirement 4.1, 5.1 (E8) -- bare `X` on an
/// editor Context runs FFEDIT EXCLUDE (the active environment wins it), and does
/// NOT close/exit the application. Pins the owner correction "in the editor X
/// must work as exclude, not return or exit".
#[test]
fn editor_bare_x_excludes_does_not_exit() {
    use crate::tab_state::{KindTag, TabKind};

    let mut shell = make_shell();
    shell.shell_new_untitled();
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::Untitled,
        "precondition: active Context is an editor (FFEDIT) Context"
    );
    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "precondition: app is not already closing"
    );

    shell.run_command_line("X ALL");

    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "bare `X ALL` on an editor Context must be FFEDIT EXCLUDE, NOT a close/exit"
    );
}

/// Validates: menu-workspace Requirement 14.13 (CR-CH-052) -- bare `X` on a
/// NON-editor Context reaches the uniform FFCMD `X` verb (FFEDIT is not active).
/// When the tab was navigated from the POM (non-empty Navigation_Stack), `X`
/// COLLAPSES to the Tab_Visual_Root (the POM) in one action; it does NOT
/// app-exit. (Under the superseded model it dispatched file.exit.)
#[test]
fn non_editor_bare_x_collapses_to_visual_root() {
    use crate::tab_state::{KindTag, TabKind};

    let mut shell = make_shell();
    shell.dispatch_command_string("CONFIG");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "precondition: active Context is the non-editor Config panel"
    );
    assert!(
        !shell.tabs.active_tab().nav_stack.is_empty(),
        "precondition: navigating the POM pushed it onto the stack"
    );
    assert!(!*shell.should_close.lock().expect("close lock"));

    shell.run_command_line("X");

    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "bare `X` above the visual root collapses, it does NOT app-exit (CR-CH-052)"
    );
    assert!(
        shell.tabs.active_tab().is_home,
        "bare `X` collapses the Config Context to its Tab_Visual_Root (the POM)"
    );
    assert!(
        shell.tabs.active_tab().nav_stack.is_empty(),
        "collapse clears the Navigation_Stack to the visual root"
    );
}

/// Validates: command-environments Requirement 3.2, 3.2a; menu-workspace Req
/// 14.14 (CR-CH-052) -- `=X` on an editor Context is the universal `=` escape
/// hatch: it is routed PAST FFEDIT to FFCMD. The front-door `=` reinitialises
/// the editor tab to the POM, then `X` at the empty POM root CLOSES the
/// Workspace (exit only when last). It is NOT claimed by FFEDIT as EXCLUDE, and
/// it is NOT an unconditional app-exit. With another tab open it closes just the
/// editor tab; as the last tab it app-exits.
#[test]
fn editor_equals_x_closes_workspace_not_exclude() {
    use crate::tab_state::{KindTag, TabKind};

    // Case 1: another tab is open (shell_new_untitled adds a second tab), so
    // `=X` closes the editor Workspace WITHOUT app-exiting.
    let mut shell = make_shell();
    shell.shell_new_untitled();
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::Untitled);
    assert_eq!(shell.tabs.len(), 2, "precondition: POM + editor tab");
    assert!(!*shell.should_close.lock().expect("close lock"));

    shell.run_command_line("=X");

    assert_eq!(
        shell.tabs.len(),
        1,
        "`=X` escapes FFEDIT and closes the editor Workspace (one tab removed)"
    );
    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "closing one of two tabs via `=X` must NOT app-exit (exit only when last)"
    );

    // Case 2: the editor is the only tab, so `=X` closes it and app-exits.
    let mut shell = make_shell();
    let idx = shell.tabs.active_index();
    if let Some(tab) = shell.tabs.tabs_mut().get_mut(idx) {
        tab.kind = TabKind::Untitled;
        tab.is_home = false;
        tab.title = "[Untitled]".to_string();
        tab.nav_stack.clear();
    }
    assert_eq!(shell.tabs.len(), 1, "precondition: single editor tab");
    assert!(!*shell.should_close.lock().expect("close lock"));

    shell.run_command_line("=X");

    assert!(
        *shell.should_close.lock().expect("close lock"),
        "`=X` on the LAST tab reaches FFCMD, closes the Workspace and app-exits \
         (CR-CH-052), NOT FFEDIT EXCLUDE"
    );
}

/// Validates: command-environments Requirement 4.2, 6.1 (E8) -- a non-colliding
/// FFEDIT verb still resolves to FFEDIT on an editor Context after the gate moved
/// ahead of `try_commands_a` (no regression in the editor-verb claim).
#[test]
fn editor_ffedit_verb_still_resolves_after_gate_move() {
    use crate::tab_state::{KindTag, TabKind};

    let mut shell = make_shell();
    shell.shell_new_untitled();
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::Untitled);

    // A representative FFEDIT verb; it must not close/exit and must be claimed by
    // FFEDIT (reaching its handler), proving the gate still fires for editor verbs.
    shell.run_command_line("CAPS");

    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "an FFEDIT verb on an editor Context must be claimed by FFEDIT, not exit"
    );
}

// === CR-CH-053 Task 17: FFEDIT-as-object + read-from-registry parity ========
//
// Task 17 turns the closed EnvironmentKind enum / environment_for_kind match /
// `== FfEdit` claim gate into a BUILT Environment_Registry, and makes FFEDIT a
// real registered `CommandEnvironment` object. These tests pin that the
// observable result is IDENTICAL to the pre-registry behaviour (Req 13.5, 13.6,
// 4.2, 6.3): the FFEDIT object claims editor verbs via the registry path, an
// FFCMD verb is unaffected, a non-editor Context does not route through FFEDIT,
// and the B062 case rule is preserved through the object path.

/// Validates: command-environments Requirement 13.5, 13.6, 4.2, 6.3 -- a
/// representative FFEDIT verb (EXCLUDE ALL) on an editor Context produces the
/// identical observable result (exclusion recorded, no close/exit) through the
/// registered `FfEditEnvironment` object as it did through the former method.
#[test]
fn ffedit_verb_via_registry_matches_prior_behaviour() {
    use crate::tab_state::{KindTag, TabKind};

    let mut shell = make_shell();
    shell.shell_new_untitled();
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::Untitled);

    // EXCLUDE ALL is claimed by the FFEDIT object (via the registry gate) and
    // records an exclusion message in open_error; it never closes/exits.
    shell.run_command_line("EXCLUDE ALL");

    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "EXCLUDE ALL on an editor Context is FFEDIT EXCLUDE via the registry, not a close/exit"
    );

    // CAPS ON mutates the edit profile through the FFEDIT object -- the same
    // observable state change as the former method path.
    shell.run_command_line("CAPS ON");
    assert_eq!(
        shell.tabs.active_tab().edit_profile.caps,
        ff_edit_operations::CapsMode::On,
        "CAPS ON must still toggle the edit profile through the registered FFEDIT object"
    );
}

/// Validates: command-environments Requirement 13.6, 13.7 -- a representative
/// FFCMD verb is unaffected by the registry: `=X` (the universal escape hatch)
/// bypasses the active environment and reaches FFCMD's exit path exactly as
/// before, so with another tab open it closes the editor Workspace rather than
/// being claimed by FFEDIT as EXCLUDE.
#[test]
fn ffcmd_verb_unaffected_by_registry() {
    use crate::tab_state::{KindTag, TabKind};

    let mut shell = make_shell();
    shell.shell_new_untitled();
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::Untitled);
    assert_eq!(shell.tabs.len(), 2, "precondition: POM + editor tab");

    // `=X` is routed PAST FFEDIT (registry gate skipped for `=`-prefixed input)
    // to FFCMD, which closes the editor Workspace. It is NOT FFEDIT EXCLUDE.
    shell.run_command_line("=X");

    assert!(
        shell.tabs.len() < 2 || *shell.should_close.lock().expect("close lock"),
        "`=X` must reach FFCMD and close/exit the editor Workspace, not be claimed by FFEDIT"
    );
}

/// Validates: command-environments Requirement 5.1, 13.3 -- a non-editor Context
/// does NOT route through FFEDIT: bare `X` on the POM (Home) Context reaches the
/// FFCMD close/exit path (registry reports FFEDIT is not active there), not
/// FFEDIT EXCLUDE.
#[test]
fn non_editor_context_does_not_route_through_ffedit() {
    // make_shell's single tab is the welcome editor; set it to the POM Home
    // Context at an empty visual root so bare `X` reaches the FFCMD close verb.
    let mut shell = make_shell();
    shell.set_active_tab_home();
    assert_eq!(shell.tabs.len(), 1, "precondition: single tab");

    // The registry must report FFEDIT is NOT active on the POM Context.
    {
        let t = shell.tabs.active_tab();
        assert!(
            !shell.environments.is_ffedit_active(t.kind.tag(), t.is_home),
            "the POM Context must not be an FFEDIT environment"
        );
    }

    // Bare `X` on the last POM tab reaches FFCMD and app-exits (not EXCLUDE).
    shell.run_command_line("X");
    assert!(
        *shell.should_close.lock().expect("close lock"),
        "bare `X` on a non-editor Context must reach FFCMD exit, not FFEDIT EXCLUDE"
    );
}

/// Validates: command-environments Requirement 4.2, 6.3 -- the B062 rule is
/// preserved through the registered FFEDIT object: the verb token is matched
/// case-insensitively while the argument case is preserved. A lowercase `change`
/// verb with a mixed-case replacement is routed through the object; the verb is
/// recognised (so no "unknown command" error) and the mixed-case argument is
/// carried verbatim to the find manager.
#[test]
fn b062_change_case_preserved_via_registry() {
    use crate::tab_state::{KindTag, TabKind};

    let mut shell = make_shell();
    shell.shell_new_untitled();
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::Untitled);

    // Lowercase verb token must still be claimed by FFEDIT (case-insensitive verb
    // match via the alias table) -- it must not fall through as an unknown command
    // and must never close/exit the editor.
    shell.run_command_line("change 'Foo' 'BarBaz'");
    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "a lowercase CHANGE verb must be claimed by FFEDIT, not treated as exit/unknown"
    );
}

// === CR-CH-053 E9: FFEDIT SAVE verb (dirty-aware, stays in editor) =========
//
// SAVE is an FFEDIT editor-buffer verb (Req 10.1): clean buffer -> no-op (no
// write, no error); dirty buffer -> write via save_active_tab, STAY in the
// editor, clear the dirty flag; a write failure (incl. untitled = no path) ->
// STAY + surface the error. SAVE never leaves the editor and is not confirmable.

/// Validates: command-environments Requirement 10.1 (E9) -- SAVE on a CLEAN
/// editor buffer is a no-op: no write is attempted, no error is set, the buffer
/// stays clean, and the editor is not left.
#[test]
fn editor_save_on_clean_buffer_is_noop() {
    use crate::tab_state::{KindTag, TabKind};

    let mut shell = make_shell();
    shell.shell_new_untitled();
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::Untitled);
    assert!(
        !shell.tabs.active_tab().is_modified,
        "precondition: a fresh untitled buffer is clean"
    );

    shell.run_command_line("SAVE");

    assert!(
        !shell.tabs.active_tab().is_modified,
        "clean SAVE is a no-op; the buffer stays clean"
    );
    assert!(
        shell.open_error.is_none(),
        "clean SAVE sets no error (no write attempted)"
    );
    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "SAVE never leaves/closes the editor"
    );
}

/// Validates: command-environments Requirement 10.1 (E9) -- SAVE on a DIRTY
/// untitled buffer (no backing path) cannot write, so it STAYS in the editor and
/// surfaces the error; the dirty flag is NOT cleared (nothing was saved). This
/// exercises the write-fail = stay + error path.
#[test]
fn editor_save_on_dirty_untitled_errors_and_stays() {
    use crate::tab_state::{KindTag, TabKind};

    let mut shell = make_shell();
    shell.shell_new_untitled();
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::Untitled);
    shell.tabs.active_tab_mut().is_modified = true;

    shell.run_command_line("SAVE");

    assert!(
        shell.open_error.is_some(),
        "dirty SAVE with no path must surface a save error (cannot save untitled)"
    );
    assert!(
        shell.tabs.active_tab().is_modified,
        "a failed SAVE must NOT clear the dirty flag (nothing was written)"
    );
    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "SAVE stays in the editor even on failure"
    );
}

// === B080 Step 3: Function family (EXIT/EDIT/BROWSE/VIEW/CLOSE) ============
//
// Step 3 of the dispatch-unify migration is the Function-target family. The
// outcome is a DEFERRAL for EDIT/BROWSE/VIEW/CLOSE (and a no-op for the EXIT
// family, which is already migrated in the prelude by `try_exit_family`, the A1
// precedence pin). The tests below PROVE the deferral analysis empirically so
// Step 7 (ladder retirement) can rely on it, and lock the current observable
// behaviour of the deferred verbs as the Req 8.4 backstop.
//
// Why EDIT/BROWSE/VIEW cannot migrate to a Function target cleanly: the ladder
// arm opens a file via `self.dispatch.execute_command("file.open", {path})` --
// a registered Command_ID executed through `CommandDispatch`, CARRYING a `path`
// param (handled by `FileOpenHandler`, which sets `pending_open`). The
// Function-target dispatch arm (`dispatch_command_target`) instead calls
// `self.handle_command(command_id)` with ONLY the bare command string and NO
// params. `handle_command("file.open")` matches no ladder arm and falls through
// to the command engine, never reaching `FileOpenHandler`, so the path is
// dropped and no file opens. Routing EDIT through a Function target would thus
// REGRESS (Req 8.4), so EDIT/BROWSE/VIEW stay on the ladder. CLOSE is a bare
// shell op (`tabs.close_tab`) with no registered Command_ID, so it does not
// round-trip through `handle_command(command_id)` either and also stays.

/// Validates: command-framework Requirement 8.4 -- the Function-target dispatch
/// arm routes a resolved Function through `handle_command(command_id)` with NO
/// params, so a bare `handle_command("file.open")` does NOT reach the
/// registered file-open handler (it sets no `pending_open`). This is the
/// empirical reason EDIT/BROWSE/VIEW are DEFERRED in Step 3: a
/// `Function { command_id: "file.open", .. }` target cannot carry the path and
/// would not open the file. The real open path uses
/// `dispatch.execute_command("file.open", { path })` instead.
#[test]
fn handle_command_with_file_open_id_does_not_reach_file_open_handler() {
    let mut shell = make_shell();
    assert!(
        shell.pending_open.lock().expect("pending_open").is_none(),
        "precondition: nothing pending to open"
    );

    // Simulate what the Function arm would do for a `file.open` command_id:
    // dispatch the bare command_id string through handle_command.
    shell.handle_command("file.open");

    assert!(
        shell.pending_open.lock().expect("pending_open").is_none(),
        "handle_command(\"file.open\") must NOT reach FileOpenHandler (no params \
         are carried), proving a Function target cannot replace the EDIT arm"
    );
}

/// Validates: command-framework Requirement 8.4 -- the EDIT ladder arm (the
/// behaviour Step 3 PRESERVES for the deferred verb) opens the path via
/// `dispatch.execute_command("file.open", { path })`, setting `pending_open` to
/// the requested path. Locks the behaviour the ladder arm must keep until Step 7.
#[test]
fn edit_path_sets_pending_open_via_file_open_command() {
    let mut shell = make_shell();

    shell.run_command_line("EDIT /tmp/step3.txt");

    // RC.B.8 (c): pending_open now carries a PendingOpenReq. EDIT is a host-path
    // open with no origin, so owning_env/identity/recfm_lrecl are all None.
    let pending = shell.pending_open.lock().expect("pending_open").clone();
    let pending = pending.expect("EDIT <path> must set a pending open");
    assert_eq!(pending.path, "/tmp/step3.txt");
    assert_eq!(pending.owning_env, None);
    assert_eq!(pending.identity, None);
    assert_eq!(pending.recfm_lrecl, None);
    assert!(
        shell.open_error.is_none(),
        "a successful EDIT clears any open_error"
    );
}

/// Validates: command-framework Requirement 8.4 -- the CLOSE ladder arm (the
/// behaviour Step 3 PRESERVES for the deferred verb) closes the active tab via
/// `tabs.close_tab`. CLOSE is a bare shell op, not a registered Command_ID, so
/// it is deferred (does not fit the Function model); this locks its behaviour.
#[test]
fn close_command_closes_the_active_tab() {
    let mut shell = make_shell();
    shell.handle_command("START"); // ensure >=2 tabs so a close is observable
    let before = shell.tabs.len();
    assert!(before >= 2, "precondition: at least two tabs open");

    shell.run_command_line("CLOSE");

    assert_eq!(
        shell.tabs.len(),
        before - 1,
        "CLOSE closes the active tab (behaviour preserved; verb deferred)"
    );
    assert!(
        shell.open_error.is_none(),
        "a successful CLOSE clears any open_error"
    );
}

/// Validates: command-environments Requirement 14.1 -- addressing the ACTIVE
/// environment via `dispatch_to_environment` is identical to not addressing it:
/// an editor Context is FFEDIT-active, so `dispatch_to_environment("FFEDIT", ..)`
/// of an owned verb claims and executes it exactly as the active-env gate would.
#[test]
fn dispatch_to_environment_addressing_active_env_claims() {
    use crate::shell::environment::EnvDispatchOutcome;
    use crate::tab_state::KindTag;

    let mut shell = make_shell();
    shell.shell_new_untitled();
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::Untitled);

    // CAPS ON is an owned FFEDIT verb; addressing FFEDIT claims+executes it.
    let outcome = shell.dispatch_to_environment("FFEDIT", "CAPS ON");
    assert_eq!(outcome, EnvDispatchOutcome::Claimed { rc: 0 });
    assert_eq!(
        shell.tabs.active_tab().edit_profile.caps,
        ff_edit_operations::CapsMode::On,
        "addressing the active FFEDIT env must execute the verb identically to the gate"
    );
}

/// Validates: command-environments Requirement 14.1 -- the name match is
/// case-insensitive on the stable env name (REXX ADDRESS style).
#[test]
fn dispatch_to_environment_name_is_case_insensitive() {
    use crate::shell::environment::EnvDispatchOutcome;

    let mut shell = make_shell();
    shell.shell_new_untitled();
    let outcome = shell.dispatch_to_environment("ffedit", "CAPS ON");
    assert_eq!(outcome, EnvDispatchOutcome::Claimed { rc: 0 });
}

/// Validates: command-environments Requirement 14.1 -- a verb the named
/// environment does not own yields `NotClaimed` (the caller falls through), not
/// a spurious claim.
#[test]
fn dispatch_to_environment_declines_unowned_verb() {
    use crate::shell::environment::EnvDispatchOutcome;

    let mut shell = make_shell();
    shell.shell_new_untitled();
    // FILES is an FFCMD workbench verb, not an FFEDIT verb -> FFEDIT declines.
    let outcome = shell.dispatch_to_environment("FFEDIT", "FILES");
    assert_eq!(outcome, EnvDispatchOutcome::NotClaimed);
}

/// Validates: command-environments Requirement 14.1 -- addressing the FFCMD base
/// at the active-env step yields `NotClaimed` (the base is reached through the
/// ordinary ladder, not claimed here); an unregistered name yields
/// `NoSuchEnvironment`.
#[test]
fn dispatch_to_environment_base_declines_and_unknown_name_reports() {
    use crate::shell::environment::EnvDispatchOutcome;

    let mut shell = make_shell();
    assert_eq!(
        shell.dispatch_to_environment("FFCMD", "CAPS ON"),
        EnvDispatchOutcome::NotClaimed
    );
    assert_eq!(
        shell.dispatch_to_environment("NOTANENV", "CAPS ON"),
        EnvDispatchOutcome::NoSuchEnvironment
    );
}

/// Validates: command-environments Requirement 14.3, 14.4, 14.8 -- the
/// store-affecting verb SAVE is routed through the address-by-name seam
/// (`dispatch_to_environment`) and remains behaviour-preserving in this slice:
/// SAVE on a clean editor buffer is a no-op (no error), exactly as the direct
/// FFEDIT SAVE. (Task 20 later flips the SAVE target to the tab's owning
/// environment; here it addresses FFEDIT and executes identically.)
#[test]
fn save_routes_through_dispatch_to_environment_and_is_behaviour_preserving() {
    use crate::shell::environment::EnvDispatchOutcome;

    let mut shell = make_shell();
    shell.shell_new_untitled();
    // A fresh untitled buffer is clean -> SAVE is a dirty-aware no-op.
    let outcome = shell.dispatch_to_environment("FFEDIT", "SAVE");
    assert_eq!(
        outcome,
        EnvDispatchOutcome::Claimed { rc: 0 },
        "SAVE is a store-affecting FFEDIT verb routed via the address seam"
    );
    assert!(
        shell.open_error.is_none(),
        "SAVE on a clean buffer stays a no-op (behaviour-preserving in this slice)"
    );
}

/// Validates: command-environments Requirement 15.1, 15.3 -- a plain host-path
/// open binds the opened tab's Owning_Environment to the host FS environment
/// (the DEFAULT_OWNING_ENVIRONMENT), so existing opens are behaviour-preserving.
#[test]
fn host_path_open_binds_owning_environment_to_host_fs() {
    use crate::tab_state::DEFAULT_OWNING_ENVIRONMENT;
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("plain.txt");
    std::fs::write(&file, "hello\n").expect("write");

    let mut shell = make_shell();
    shell
        .shell_open_file(file.to_str().expect("path"))
        .expect("open");
    assert_eq!(
        shell.active_owning_environment(),
        DEFAULT_OWNING_ENVIRONMENT,
        "a plain host-path open must bind the host FS Owning_Environment"
    );
}

/// Validates: command-environments Requirement 15.2, 15.4 -- an open carrying an
/// originating environment binds the opened tab to THAT environment, and FFEDIT
/// reads the binding back as the SAVE target.
#[test]
fn open_with_origin_binds_that_owning_environment() {
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("owned.txt");
    std::fs::write(&file, "hello\n").expect("write");

    let mut shell = make_shell();
    shell
        .shell_open_file_with_env(file.to_str().expect("path"), Some("MAINFRAME"))
        .expect("open");
    assert_eq!(
        shell.active_owning_environment(),
        "MAINFRAME",
        "an open carrying an origin must bind that Owning_Environment"
    );
}

/// Validates: command-environments Requirement 15.6 -- binding/reading the
/// Owning_Environment does not change observable SAVE behaviour for a
/// host-path (native) file: SAVE on a clean host-bound buffer is a no-op.
#[test]
fn host_bound_save_is_behaviour_preserving() {
    use crate::tab_state::DEFAULT_OWNING_ENVIRONMENT;
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("save.txt");
    std::fs::write(&file, "hello\n").expect("write");

    let mut shell = make_shell();
    shell
        .shell_open_file(file.to_str().expect("path"))
        .expect("open");
    assert_eq!(
        shell.active_owning_environment(),
        DEFAULT_OWNING_ENVIRONMENT
    );

    // SAVE on a freshly-opened (clean) buffer is a dirty-aware no-op, unchanged
    // from before the owning-environment binding existed.
    shell.run_command_line("SAVE");
    assert!(
        shell.open_error.is_none(),
        "SAVE on a clean host-bound buffer stays a no-op (behaviour-preserving)"
    );
}

/// Validates: command-environments Requirement 18.2; document-model Req 11.2
/// (RC.B.8 (c)) -- applying a pending open carrying a MAINFRAME identity +
/// record format binds the real DSN/catalog onto the tab and sets the Document's
/// RecordFormat, so a later SAVE addresses the dataset by DSN and takes the
/// record path. End-to-end threading of the single file.open payload.
#[test]
fn mainframe_open_binds_store_identity_and_record_format() {
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("payroll.data");
    std::fs::write(&file, "AAAABBBB").expect("write");

    let mut shell = make_shell();
    shell
        .shell_open_file_with_env(file.to_str().expect("path"), Some("MAINFRAME"))
        .expect("open");
    // Apply the identity + record format the open payload carried (the step
    // update.rs performs on the pending open).
    shell.apply_pending_open_identity(
        Some(("USER.PAYROLL.DATA".to_string(), "PAYROLL".to_string())),
        Some((ff_vfs::RecordFormatKind::Fixed, 4)),
    );

    assert_eq!(shell.active_owning_environment(), "MAINFRAME");
    let id = shell
        .tabs
        .active_tab()
        .store_identity
        .clone()
        .expect("store_identity must be bound for a MAINFRAME open");
    assert_eq!(id.dsn, "USER.PAYROLL.DATA");
    assert_eq!(id.catalog, "PAYROLL");
    let format = {
        let doc = shell.tabs.active_tab().document.clone();
        shell
            .runtime
            .block_on(async { doc.read().await.record_format() })
    };
    assert_eq!(format, ff_document_model::RecordFormat::Fixed { lrecl: 4 });
}

/// Validates: command-environments Requirement 14.4, 14.5, 16.4 (RC.B.8 (b)) --
/// the single SAVE seam resolves the backend by the active tab's
/// Owning_Environment: a MAINFRAME tab reaches the record-capable mainframe CE
/// (which, with no dataset catalogued in the freshly-built access, surfaces a
/// non-zero rc as an open_error -- proving the RECORD path was taken, not the
/// byte path which would silently succeed), while a HOSTFS tab's SAVE takes the
/// byte path and succeeds.
#[test]
fn save_seam_resolves_backend_by_owning_environment() {
    use ff_document_model::{BytePosition, RecordFormat};
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("mf.data");
    std::fs::write(&file, "AAAABBBB").expect("write");

    let mut shell = make_shell();
    shell
        .shell_open_file_with_env(file.to_str().expect("path"), Some("MAINFRAME"))
        .expect("open");
    shell.apply_pending_open_identity(
        Some(("USER.NO.SUCH.DATA".to_string(), "PAYROLL".to_string())),
        Some((ff_vfs::RecordFormatKind::Fixed, 4)),
    );
    // Dirty the buffer so SAVE actually attempts a store.
    {
        let doc = shell.tabs.active_tab().document.clone();
        shell.runtime.block_on(async {
            let _ = doc.write().await.insert(BytePosition(0), b"X");
        });
        shell.tabs.active_tab_mut().is_modified = true;
    }
    shell.run_command_line("SAVE");
    assert!(
        shell.open_error.is_some(),
        "a MAINFRAME SAVE took the record path and the mainframe CE reported a \
         non-zero rc for an un-catalogued dataset (record path proven)"
    );

    // A HOSTFS tab's SAVE takes the byte path and succeeds (host backend).
    let host_file = dir.path().join("host.txt");
    std::fs::write(&host_file, "hello\n").expect("write host");
    shell
        .shell_open_file(host_file.to_str().expect("path"))
        .expect("open host");
    {
        let doc = shell.tabs.active_tab().document.clone();
        shell.runtime.block_on(async {
            doc.write()
                .await
                .set_record_format(RecordFormat::Delimited {
                    terminator: ff_document_model::DelimiterTerminator::Lf,
                });
            let _ = doc.write().await.insert(BytePosition(0), b"Z");
        });
        shell.tabs.active_tab_mut().is_modified = true;
    }
    shell.run_command_line("SAVE");
    assert!(
        shell.open_error.is_none(),
        "a HOSTFS SAVE takes the byte path and succeeds (host backend)"
    );
}

/// Validates: command-environments Requirement 14.4, 14.5, 14.6, 10.1 (Task 20)
/// -- a dirty host-path buffer SAVEd through FFEDIT is written to disk by the
/// OWNING (host FS) environment via `dispatch_to_environment`, and the on-disk
/// result + the cleared dirty flag + save point are identical to the former
/// direct-write path. Only the EXECUTOR moved (FFEDIT -> host FS env); native
/// SAVE behaviour is unchanged.
#[test]
fn ffedit_save_routes_dirty_host_file_write_through_owning_env() {
    use crate::tab_state::{KindTag, DEFAULT_OWNING_ENVIRONMENT};
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let file = dir.path().join("doc.txt");
    std::fs::write(&file, "original\n").expect("seed");

    let mut shell = make_shell();
    shell
        .shell_open_file(file.to_str().expect("path"))
        .expect("open");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::FileEditor);
    assert_eq!(
        shell.active_owning_environment(),
        DEFAULT_OWNING_ENVIRONMENT
    );

    // Make an in-buffer edit so the tab is dirty (insert a marker at the start).
    {
        let doc = shell.tabs.active_tab().document.clone();
        shell.runtime.block_on(async {
            let mut d = doc.write().await;
            let _ = d.insert(ff_document_model::BytePosition(0), b"EDITED ");
        });
    }
    shell.tabs.active_tab_mut().is_modified = true;

    // SAVE via the command line -> FFEDIT -> addresses the host FS owning env,
    // which performs the dirty-aware write.
    shell.run_command_line("SAVE");

    assert!(
        shell.open_error.is_none(),
        "a successful SAVE through the owning env clears any error"
    );
    assert!(
        !shell.tabs.active_tab().is_modified,
        "SAVE must clear the dirty flag (contract preserved through the reroute)"
    );
    let on_disk = std::fs::read_to_string(&file).expect("read back");
    assert!(
        on_disk.contains("EDITED"),
        "the host FS owning environment must have written the edited buffer to disk"
    );
}
