//! Shell tests -- session area (split from shell/tests.rs, CR F3).
//! Items are verbatim; only their file location changed.

#![allow(unused_imports)]
use super::tests_common::*;
use ff_keys::{KeyMap, ModifiedKey};
use std::sync::{Arc, Mutex};

use ff_command::{
    CommandDispatch, CommandError, CommandHandler, CommandHistory, CommandId, CommandMetadata,
    CommandParams, CommandRegistry, CommandResult, ExecutionContext,
};

/// Validates: Requirement 21.7 -- command history records submitted commands.
#[test]
fn command_history_records_entries() {
    // Validates: Phase U 21.7 -- ff-keys CommandHistory integration
    use ff_keys::CommandHistory;
    let mut history = CommandHistory::new(100);
    history.add("FIND hello");
    history.add("LOCATE 42");
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(0).map(|e| e.command()), Some("LOCATE 42"));
}

/// Validates: Requirement 21.7 -- RETRIEVE cycles through command history.
#[test]
fn retrieve_state_cycles_through_history() {
    // Validates: Phase U 21.7 -- RetrieveState steps back through history
    use ff_keys::{CommandHistory, RetrieveResult, RetrieveState};
    let mut history = CommandHistory::new(100);
    history.add("FIND hello");
    history.add("LOCATE 42");
    let mut retrieve = RetrieveState::new();
    let first = retrieve.retrieve(&history, "");
    assert!(matches!(first, RetrieveResult::Recalled { .. }));
    if let RetrieveResult::Recalled { command } = first {
        assert_eq!(command, "LOCATE 42");
    }
}

// === B062: command-line arguments are case-preserved (verb case-insensitive) ==
// The command line matches VERBS case-insensitively but MUST pass ARGUMENTS to
// handlers with their original case (critical for FIND/CHANGE/LOCATE search
// strings). These regression tests lock that guarantee in place.

/// Validates: B062 -- `handle_command` does NOT uppercase the whole line before
/// recording it; the recorded command-line history preserves argument case.
/// Every arm slices its argument from this same original `cmd`, so a preserved
/// recorded line evidences preserved handler arguments.
#[test]
fn command_arguments_preserve_case_in_history() {
    let mut shell = make_shell();
    // Mixed-case arguments across the case-sensitive commands.
    shell.handle_command("FIND 'MixedCase'");
    assert_eq!(
        shell.command_line_history.most_recent(),
        Some("FIND 'MixedCase'"),
        "FIND search term case must be preserved (not uppercased)"
    );
    shell.handle_command("LOCATE MyLabel");
    assert_eq!(
        shell.command_line_history.most_recent(),
        Some("LOCATE MyLabel")
    );
    shell.handle_command("CHANGE 'Old' 'New'");
    assert_eq!(
        shell.command_line_history.most_recent(),
        Some("CHANGE 'Old' 'New'")
    );
    // A lowercase VERB still matches (verb is case-insensitive) and its argument
    // keeps case.
    shell.handle_command("find 'AlsoMixed'");
    assert_eq!(
        shell.command_line_history.most_recent(),
        Some("find 'AlsoMixed'")
    );
}

/// Validates: function-keys-and-history Req 6.2 -- at startup the shell loads a
/// persisted command history file into the command-line history.
#[test]
fn startup_loads_persisted_command_history() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let hist = dir.path().join("command_history.toml");
    // Most-recent-first, matching the History_Store schema.
    std::fs::write(
        &hist,
        "schema_version = 1\n\
         [[entries]]\ncommand = \"THEME legacy\"\n\
         [[entries]]\ncommand = \"LOCATE 1\"\n",
    )
    .expect("write history");

    let shell = make_shell_with_history_path(&hist);
    assert_eq!(
        shell.command_line_history.list(),
        vec!["THEME legacy".to_string(), "LOCATE 1".to_string()],
        "startup must load the persisted history most-recent-first"
    );
    // And RETRIEVE recalls the most recent loaded entry.
    assert_eq!(
        shell.command_line_history.most_recent(),
        Some("THEME legacy")
    );
}

/// Validates: function-keys-and-history Req 6.5/6.6 -- a missing or corrupt
/// history file yields an empty history without failing startup.
#[test]
fn startup_missing_or_corrupt_history_is_empty_no_panic() {
    // Missing file.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let missing = dir.path().join("does_not_exist.toml");
    let shell = make_shell_with_history_path(&missing);
    assert!(shell.command_line_history.is_empty());

    // Corrupt file.
    let corrupt = dir.path().join("corrupt.toml");
    std::fs::write(&corrupt, "this is { not valid toml =").expect("write");
    let shell2 = make_shell_with_history_path(&corrupt);
    assert!(
        shell2.command_line_history.is_empty(),
        "a corrupt history file must degrade to empty, not panic"
    );
}

/// Validates: function-keys-and-history Req 6.3 -- on exit the shell writes the
/// current command history to the History_Store, which reloads on next startup.
#[test]
fn exit_saves_command_history_and_reloads() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let hist = dir.path().join("command_history.toml");

    {
        let mut shell = make_shell_with_history_path(&hist);
        shell.handle_command("LOCATE 1");
        shell.dispatch_command_string("THEME legacy");
        shell.persist_command_history(); // the on_exit save path
        let _ = shell
            .config_handle
            .remove_user_value(ff_config::keys::theme::ACTIVE);
    }
    assert!(hist.exists(), "on-exit save must write the history file");

    // A fresh shell reloads what was saved (most-recent-first).
    let reloaded = make_shell_with_history_path(&hist);
    assert_eq!(
        reloaded.command_line_history.list(),
        vec!["THEME legacy".to_string(), "LOCATE 1".to_string()]
    );
}

/// Validates: command-framework Req 13.2/13.3 -- an UNRESOLVED command (a typo)
/// leaves the field for correction (Restore of the executed text), and a
/// RESOLVED-but-errored command likewise restores it.
#[test]
fn unresolved_or_errored_command_restores_field_for_correction() {
    // Unresolved: a gibberish command sets open_error -> Restore keeps the text.
    let mut shell = make_shell();
    shell.run_command_line("ZXQWV nonsense");
    assert_eq!(
        shell.command_text, "ZXQWV nonsense",
        "an unresolved command keeps the typed text for correction (Req 13.2)"
    );
    assert!(shell.open_error.is_some());

    // Resolved but errored: SWAP to an out-of-range tab restores the text.
    let mut shell2 = make_shell();
    shell2.command_text = "SWAP 999".to_string();
    shell2.run_command_line("SWAP 999");
    assert_eq!(
        shell2.command_text, "SWAP 999",
        "a resolved-but-failed command restores the executed text (Req 13.3)"
    );
    assert!(shell2.open_error.is_some());
}

/// Validates: command-framework Req 13.4 -- RETRIEVE returns `Set(<recalled>)`,
/// so the recalled command lands in the field after a key-forwarded F12 with a
/// non-empty field (preserves B067). The recall survives the outcome pass.
#[test]
fn key_command_retrieve_keeps_recalled_field() {
    let mut shell = make_shell();
    // Seed history with a recallable command.
    shell.run_command_line("THEME legacy");
    // Type something, then press the RETRIEVE key: merged `RETRIEVE <field>`.
    shell.command_text = "LOC".to_string();
    shell.dispatch_key_command("RETRIEVE");
    assert_eq!(
        shell.command_text, "THEME legacy",
        "RETRIEVE recalls the most recent command INTO the field (Set outcome, Req 13.4)"
    );
}

// === Phase CA -- TSO Session Lifecycle Commands (Requirement 20) ===========

/// Validates: Requirement 20.1 -- session start time is recorded on shell creation.
#[test]
fn session_start_time_is_recorded_on_startup() {
    // Validates: Requirement 20.1
    use chrono::Local;
    let before = Local::now();
    let shell = make_shell();
    let after = Local::now();
    assert!(
        shell.session_start >= before && shell.session_start <= after,
        "session_start must be set during WorkbenchShell::new()"
    );
}

/// Validates: Requirement 20.1 -- format_session_start produces Started: HH:MM.
#[test]
fn format_session_start_produces_started_hhmm() {
    // Validates: Requirement 20.1
    let shell = make_shell();
    let label = shell.format_session_start();
    assert!(
        label.starts_with("Started: "),
        "label must start with 'Started: ', got: {label}"
    );
    // HH:MM format: 8 chars total ("Started: " = 9, then HH:MM = 5)
    assert_eq!(
        label.len(),
        14,
        "'Started: HH:MM' is 14 chars, got: {label}"
    );
}

/// Validates: workspace-model Requirement 6.2 -- MRU list persists through save/load round-trip.
#[test]
fn workspace_mru_persists_through_save_load() {
    // Validates: workspace-model Requirement 6.2
    use ff_session::{load_workspace, save_workspace, WorkspaceRecentFile, WorkspaceState};
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let ws_path = tmp.path().join("mru-persist.ffwb-workspace");

    let mut state = WorkspaceState::new("MruPersist");
    state.recent_files.push(WorkspaceRecentFile {
        path: std::path::PathBuf::from("C:/projects/main.rs"),
        opened_at: "2026-01-01T00:00:00+00:00".to_string(),
    });
    save_workspace(&state, &ws_path).expect("save");

    let loaded = load_workspace(&ws_path).expect("load");
    assert_eq!(loaded.recent_files.len(), 1);
    assert_eq!(
        loaded.recent_files[0].path,
        std::path::PathBuf::from("C:/projects/main.rs")
    );
}

/// Validates: function-keys-and-history Requirement 21.7 -- shell-intercept
/// commands are recorded in the RETRIEVE history so F12 can recall them.
/// Regression for the owner report: `THEME legacy` (a shell intercept) was not
/// retrievable while `LOCATE 1` (engine-routed) was, because history was only
/// recorded on some paths. History is now recorded once at the top of
/// handle_command for every submitted command.
#[test]
fn shell_intercept_commands_are_recorded_in_history() {
    let mut shell = make_shell();
    // A shell-intercept command (previously not recorded).
    shell.dispatch_command_string("THEME legacy");
    // An engine-routed command (was already recorded).
    shell.handle_command("LOCATE 1");

    // Both must be present in history (most-recent-first).
    let hist = shell.command_line_history.list();
    assert_eq!(hist.first().map(String::as_str), Some("LOCATE 1"));
    assert_eq!(hist.get(1).map(String::as_str), Some("THEME legacy"));

    // RETRIEVE itself must NOT be recorded (it is the recall action).
    shell.handle_command("RETRIEVE");
    assert_eq!(
        shell.command_line_history.most_recent(),
        Some("LOCATE 1"),
        "RETRIEVE must not be added to history"
    );

    // Clean up the theme.active user-config write made by THEME legacy.
    let _ = shell
        .config_handle
        .remove_user_value(ff_config::keys::theme::ACTIVE);
}

/// Validates: function-keys-and-history Req 19.1/19.2 (B067) -- pressing F12
/// (RETRIEVE) with a NON-EMPTY command field still recalls the previous
/// command. Regression from B066: `dispatch_key_command` merges the field, so
/// F12 dispatches `RETRIEVE <field>`; the RETRIEVE recall must still fire (the
/// verb is matched by prefix, not exact string).
#[test]
fn retrieve_via_key_with_nonempty_field_recalls_previous_command() {
    let mut shell = make_shell();
    shell.handle_command("LOCATE 1");
    shell.dispatch_command_string("THEME legacy");
    // History (most recent first): ["THEME legacy", "LOCATE 1"].

    // Simulate F12 while the user has typed something in the field.
    shell.command_text = "some typed text".to_string();
    shell.dispatch_key_command("RETRIEVE");

    // The most recent command is recalled INTO the field (Req 19.2), not an error.
    assert_eq!(
        shell.command_text, "THEME legacy",
        "F12 with a non-empty field must recall the most recent command"
    );
    assert!(shell.open_error.is_none(), "RETRIEVE must not error");

    // A second F12 walks one older (Req 19.3). The field now holds the recalled
    // command; RETRIEVE reads the field to decide LIST/empty, and "THEME legacy"
    // is neither, so it advances the pointer.
    shell.dispatch_key_command("RETRIEVE");
    assert_eq!(shell.command_text, "LOCATE 1", "second F12 walks older");

    // The merged `RETRIEVE ...` form must NOT be added to history (Req 19.6).
    assert_eq!(
        shell.command_line_history.most_recent(),
        Some("THEME legacy"),
        "RETRIEVE (even merged) must not pollute history"
    );

    let _ = shell
        .config_handle
        .remove_user_value(ff_config::keys::theme::ACTIVE);
}

/// Validates: function-keys-and-history Req 19.1 (B067) -- `RETRIEVE LIST`
/// dispatched via a key (merged form) still opens the history-list overlay.
#[test]
fn retrieve_list_via_key_opens_history_overlay() {
    let mut shell = make_shell();
    shell.handle_command("LOCATE 1");
    // Type LIST then press the RETRIEVE key -> dispatch_key_command merges to
    // "RETRIEVE LIST".
    shell.command_text = "LIST".to_string();
    shell.dispatch_key_command("RETRIEVE");
    assert!(
        shell.show_history_list.is_some(),
        "RETRIEVE LIST via key must open the history overlay"
    );
}

// === Phase DB (DB.11): descriptor-based restore (startup-and-session Req 21) ===

/// Validates: Requirement 21.5 -- a Files CustomWorkspace descriptor re-opens the Files panel.
#[test]
fn restore_files_descriptor_opens_files_panel() {
    use crate::tab_state::{KindTag, TabKind};
    use ff_session::session_state::{DescriptorParams, WorkspaceDescriptor, WorkspaceKind};

    let mut shell = make_shell();
    let descriptors = vec![WorkspaceDescriptor::CustomWorkspace {
        workspace_kind: WorkspaceKind::Files,
        params: DescriptorParams::new(),
    }];
    shell.restore_workspace_descriptors(&descriptors);
    assert!(
        shell
            .tabs
            .tabs()
            .iter()
            .any(|t| t.kind.tag() == KindTag::FilesPanel),
        "a Files descriptor must reconstruct a FilesPanel tab"
    );
}

/// Validates: Requirement 21.5 -- a FileExplorer descriptor re-opens the File Explorer panel.
#[test]
fn restore_file_explorer_descriptor_opens_explorer_panel() {
    use crate::tab_state::{KindTag, TabKind};
    use ff_session::session_state::{DescriptorParams, WorkspaceDescriptor, WorkspaceKind};

    let mut shell = make_shell();
    let descriptors = vec![WorkspaceDescriptor::CustomWorkspace {
        workspace_kind: WorkspaceKind::FileExplorer,
        params: DescriptorParams::new(),
    }];
    shell.restore_workspace_descriptors(&descriptors);
    assert!(
        shell
            .tabs
            .tabs()
            .iter()
            .any(|t| t.kind.tag() == KindTag::FileExplorerPanel),
        "a FileExplorer descriptor must reconstruct a FileExplorerPanel tab"
    );
}

/// Validates: Requirement 21.3 -- a Config descriptor with a namespace param
/// restores the Config Context with that namespace filter applied.
#[test]
fn restore_config_descriptor_applies_namespace_filter() {
    use crate::tab_state::{KindTag, TabKind};
    use ff_session::session_state::{
        DescriptorParams, DescriptorValue, WorkspaceDescriptor, WorkspaceKind,
    };

    let mut shell = make_shell();
    let mut params = DescriptorParams::new();
    params.insert("namespace".to_string(), DescriptorValue::from("editor"));
    let descriptors = vec![WorkspaceDescriptor::CustomWorkspace {
        workspace_kind: WorkspaceKind::Config,
        params,
    }];
    shell.restore_workspace_descriptors(&descriptors);

    assert!(
        shell
            .tabs
            .tabs()
            .iter()
            .any(|t| t.kind.tag() == KindTag::ConfigPanel),
        "a Config descriptor must reconstruct a ConfigPanel tab"
    );
    assert_eq!(
        shell.config_panel.namespace_filter.as_deref(),
        Some("editor"),
        "the namespace filter must be restored from the descriptor param"
    );
    assert_eq!(shell.config_panel.filter, "editor.");
}

/// Validates: Requirement 21.3 -- a Config descriptor with no namespace restores
/// the unfiltered Config view.
#[test]
fn restore_config_descriptor_without_namespace_is_unfiltered() {
    use ff_session::session_state::{DescriptorParams, WorkspaceDescriptor, WorkspaceKind};

    let mut shell = make_shell();
    let descriptors = vec![WorkspaceDescriptor::CustomWorkspace {
        workspace_kind: WorkspaceKind::Config,
        params: DescriptorParams::new(),
    }];
    shell.restore_workspace_descriptors(&descriptors);
    assert!(shell.config_panel.namespace_filter.is_none());
    assert_eq!(shell.config_panel.filter, "");
}

/// Validates: Requirement 21.5 -- multiple descriptors restore multiple Workspaces.
#[test]
fn restore_multiple_descriptors_opens_each_workspace() {
    use crate::tab_state::{KindTag, TabKind};
    use ff_session::session_state::{DescriptorParams, WorkspaceDescriptor, WorkspaceKind};

    let mut shell = make_shell();
    let descriptors = vec![
        WorkspaceDescriptor::CustomWorkspace {
            workspace_kind: WorkspaceKind::Files,
            params: DescriptorParams::new(),
        },
        WorkspaceDescriptor::CustomWorkspace {
            workspace_kind: WorkspaceKind::PluginManager,
            params: DescriptorParams::new(),
        },
        WorkspaceDescriptor::CustomWorkspace {
            workspace_kind: WorkspaceKind::EventLog,
            params: DescriptorParams::new(),
        },
    ];
    shell.restore_workspace_descriptors(&descriptors);
    let kinds: Vec<KindTag> = shell.tabs.tabs().iter().map(|t| t.kind.tag()).collect();
    assert!(kinds.contains(&KindTag::FilesPanel));
    assert!(kinds.contains(&KindTag::PluginManager));
    assert!(kinds.contains(&KindTag::EventLog));
}

/// Validates: Requirement 21.2 -- an Editor descriptor with a uri param re-opens the file.
#[test]
fn restore_editor_descriptor_opens_file() {
    use crate::tab_state::{KindTag, TabKind};
    use ff_session::session_state::{
        DescriptorParams, DescriptorValue, WorkspaceDescriptor, WorkspaceKind,
    };
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().expect("tempfile");
    writeln!(tmp, "hello from restore").expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let mut shell = make_shell();
    let mut params = DescriptorParams::new();
    params.insert("uri".to_string(), DescriptorValue::from(path.clone()));
    let descriptors = vec![WorkspaceDescriptor::CustomWorkspace {
        workspace_kind: WorkspaceKind::Editor,
        params,
    }];
    shell.restore_workspace_descriptors(&descriptors);
    assert!(
        shell.tabs.tabs().iter().any(
            |t| t.kind.tag() == KindTag::FileEditor && t.path.as_deref() == Some(path.as_str())
        ),
        "an Editor descriptor with a uri must reopen that file"
    );
}

// Validates: startup-and-session Requirement 21.2/21.3; command-configurator
// Requirement 2.1 -- a CommandConfigurator descriptor reconstructs the Context.
#[test]
fn restore_command_configurator_descriptor_reopens_context() {
    use crate::tab_state::{KindTag, TabKind};
    use ff_session::session_state::{DescriptorParams, WorkspaceDescriptor, WorkspaceKind};
    let mut shell = make_shell();
    let descriptors = vec![WorkspaceDescriptor::CustomWorkspace {
        workspace_kind: WorkspaceKind::CommandConfigurator,
        params: DescriptorParams::new(),
    }];
    shell.restore_workspace_descriptors(&descriptors);
    assert!(
        shell
            .tabs
            .tabs()
            .iter()
            .any(|t| t.kind.tag() == KindTag::CommandConfigurator),
        "a CommandConfigurator descriptor must reconstruct the Context"
    );
}

/// Validates: Requirement 23.1, 23.6, 23.10 -- with the command field focused,
/// the first Up captures the (empty) in-progress line and recalls the most-recent
/// entry into the field WITHOUT executing it; a second Up steps one entry older.
#[test]
fn command_field_up_recalls_older_history() {
    let mut harness = harness_shell();
    seed_history(&mut harness, &["THEME legacy", "LOCATE 1"]);
    // Focus is on the command field on entry (Req 16.1a); confirm the gate.
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "command field holds focus on POM entry"
    );
    // First Up -> most-recent entry (Req 23.1).
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(
        harness.state().command_text,
        "LOCATE 1",
        "first Up recalls the most-recent history entry"
    );
    // Second Up -> one entry older (Req 23.1, shared single-step recall).
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(
        harness.state().command_text,
        "THEME legacy",
        "second Up steps to the older entry"
    );
    // The recalled command is only placed, never executed (still on POM).
    assert!(
        harness.state().tabs.active_tab().is_home,
        "recall must not execute the command (Req 23.10)"
    );
}

/// Validates: Requirement 23.2, 23.3, 23.6 -- Down steps newer, and stepping past
/// the newest entry restores the In_Progress_Line captured at the cycle start.
#[test]
fn command_field_down_restores_in_progress_line() {
    let mut harness = harness_shell();
    seed_history(&mut harness, &["THEME legacy", "LOCATE 1"]);
    // Type an in-progress line before starting the cycle.
    harness.state_mut().command_text = "IN PROGRESS".to_string();
    harness.run();
    // Up twice: LOCATE 1 (newest) then THEME legacy (older).
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(harness.state().command_text, "THEME legacy");
    // Down: back to the newer entry (Req 23.2).
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(
        harness.state().command_text,
        "LOCATE 1",
        "Down steps one entry newer"
    );
    // Down again: past the newest -> restore the in-progress line (Req 23.3).
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(
        harness.state().command_text,
        "IN PROGRESS",
        "Down past newest restores the captured in-progress line"
    );
    // A further Down at initial is a no-op (Req 23.3 second sentence).
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_eq!(
        harness.state().command_text,
        "IN PROGRESS",
        "Down at initial position is a no-op"
    );
}

/// Validates: Requirement 23.1, 23.8 -- the arrow gesture shares the SAME
/// Retrieve_Pointer as the RETRIEVE command: an Up followed by a RETRIEVE
/// continues stepping older from where the arrow left off, and the arrow records
/// nothing in history.
#[test]
fn up_shares_pointer_with_retrieve() {
    let mut harness = harness_shell();
    seed_history(&mut harness, &["THEME legacy", "LOCATE 1"]);
    let len_before = harness.state().command_line_history.len();
    // Up recalls the newest (LOCATE 1) and advances the shared pointer.
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(harness.state().command_text, "LOCATE 1");
    // RETRIEVE now steps to the OLDER entry (shared pointer), not back to newest.
    // Use the full command-line path so the Command_Line_Outcome (Set(recalled))
    // is applied to the field, exactly as an Enter/F12 submission would.
    harness.state_mut().run_command_line("RETRIEVE");
    harness.run();
    assert_eq!(
        harness.state().command_text,
        "THEME legacy",
        "RETRIEVE continues from the arrow's pointer position (shared pointer)"
    );
    // The arrow gesture recorded nothing new (Req 23.8).
    assert_eq!(
        harness.state().command_line_history.len(),
        len_before,
        "arrow stepping must not add history entries"
    );
}

// -- CR-CH-036 (B045): independent per-window command contexts ---------------

/// Validates: menu-and-statusbar Requirement 18.10 (CR-CH-036, B045) --
/// `with_workspace_context` installs a detached window's buffers + active tab
/// for the duration of the closure, then restores the Primary_Window's context
/// exactly (active index + command_text + scroll + open_error + focus/outcome).
#[test]
fn with_workspace_context_saves_and_restores_primary_context() {
    // Validates: menu-and-statusbar Requirement 18.10
    let mut shell = make_shell();
    shell.handle_command("START"); // a 2nd tab so we have index 1
    let primary_active = shell.tabs.active_index();
    shell.command_text = "PRIMARY".to_string();
    shell.scroll_field_text = "HALF".to_string();
    shell.open_error = Some("primary error".to_string());

    let target = if primary_active == 0 { 1 } else { 0 };
    let mut ctx = super::WorkspaceCommandContext {
        command_text: "DETACHED".to_string(),
        ..Default::default()
    };
    let mut observed_active = usize::MAX;
    let mut observed_cmd = String::new();
    shell.with_workspace_context(target, &mut ctx, |s| {
        observed_active = s.tabs.active_index();
        observed_cmd = s.command_text.clone();
        // Mutate the detached context's command line inside the swap.
        s.command_text = "DETACHED-EDITED".to_string();
    });

    // Inside the swap, the detached tab + its buffer were active.
    assert_eq!(
        observed_active, target,
        "swap must install the detached tab as active"
    );
    assert_eq!(
        observed_cmd, "DETACHED",
        "swap must install the detached buffer"
    );
    // The mutation landed back in the context, not the primary shell.
    assert_eq!(ctx.command_text, "DETACHED-EDITED", "ctx captures the edit");
    // The Primary_Window context is fully restored.
    assert_eq!(
        shell.tabs.active_index(),
        primary_active,
        "active index restored"
    );
    assert_eq!(
        shell.command_text, "PRIMARY",
        "primary command_text restored"
    );
    assert_eq!(shell.scroll_field_text, "HALF", "primary scroll restored");
    assert_eq!(
        shell.open_error.as_deref(),
        Some("primary error"),
        "primary error restored"
    );
}

/// Validates: command-environments Requirement 15.5 (CR-CH-053 Task 19.2) -- the
/// Owning_Environment is NOT persisted as a new descriptor field; a tab reopened
/// from its `WorkspaceDescriptor` RECAPTURES its owning environment from the
/// origin by re-entering the same `file.open` seam. For a host-path editor
/// descriptor that recapture is the host FS environment default, so no new
/// persistence format is introduced and the reopened tab is bound correctly.
#[test]
fn reopened_editor_descriptor_recaptures_owning_environment() {
    use crate::tab_state::{KindTag, DEFAULT_OWNING_ENVIRONMENT};
    use ff_session::session_state::{
        DescriptorParams, DescriptorValue, WorkspaceDescriptor, WorkspaceKind,
    };
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().expect("tempfile");
    writeln!(tmp, "persisted body").expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let mut shell = make_shell();
    let mut params = DescriptorParams::new();
    params.insert("uri".to_string(), DescriptorValue::from(path.clone()));
    let descriptors = vec![WorkspaceDescriptor::CustomWorkspace {
        workspace_kind: WorkspaceKind::Editor,
        params,
    }];
    shell.restore_workspace_descriptors(&descriptors);

    let reopened = shell
        .tabs
        .tabs()
        .iter()
        .find(|t| t.kind.tag() == KindTag::FileEditor && t.path.as_deref() == Some(path.as_str()))
        .expect("the editor descriptor must reopen the file");
    assert_eq!(
        reopened.owning_environment, DEFAULT_OWNING_ENVIRONMENT,
        "a reopened host-path editor tab recaptures the host FS Owning_Environment \
         (no descriptor field; recaptured via the file.open seam)"
    );
}
