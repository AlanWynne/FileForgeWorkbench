//! Shell tests -- FEAT-002 CR-NR-119: the enforced-root-aware HELP open
//! (`open_root_help`) and the `--help` / `-h` / `/?` prelude short-circuit on
//! the single front door. These exercise the two dispatch REACH-paths that
//! bring both the ISPF `HELP <cmd>` form and the POSIX `<cmd> --help` form to
//! the SAME `cmd:<ROOT>` topic through the one `open_help`/`display` seam.

#![allow(unused_imports)]
use super::tests_common::*;
use crate::tab_state::KindTag;
use ff_help::{HelpTopic, TopicKey, TopicSource};

/// Seed the registry with a file topic for `cmd:<UPPER>` so a resolved
/// `cmd:<ROOT>` is observable as the displayed topic key.
fn seed_cmd_topic(shell: &mut super::WorkbenchShell, upper: &str) {
    shell.seed_help_registry_for_test(vec![HelpTopic::new(
        TopicKey::command(upper),
        format!("{upper} Command"),
        format!("The {upper} command."),
        TopicSource::FileBased {
            file_path: std::path::PathBuf::from("cmd.help.md"),
        },
    )]);
}

/// The TopicKey currently shown in the Help Context panel (or None).
fn shown_key(shell: &super::WorkbenchShell) -> Option<TopicKey> {
    shell
        .help_panel_for_test()
        .model()
        .current_topic_key()
        .cloned()
}

// Validates: context-help Requirement 22.8 -- `HELP KEYS` resolves cmd:KEYS
// (an enforced FFCMD root) DIRECTLY, taking precedence over the legacy
// KEYS -> feature:function_keys special-case.
#[test]
fn help_keys_resolves_cmd_keys_not_feature() {
    let mut shell = make_shell();
    seed_cmd_topic(&mut shell, "KEYS");
    shell.handle_command("HELP KEYS");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::HelpContext);
    assert_eq!(
        shown_key(&shell),
        Some(TopicKey::command("KEYS")),
        "HELP KEYS must open cmd:KEYS, not feature:function_keys"
    );
}

// Validates: context-help Requirement 22.8 -- `HELP CONFIG` resolves cmd:CONFIG
// (an enforced FFCMD root) DIRECTLY, not the feature:configuration topic.
#[test]
fn help_config_resolves_cmd_config_not_feature() {
    let mut shell = make_shell();
    seed_cmd_topic(&mut shell, "CONFIG");
    shell.handle_command("HELP CONFIG");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::HelpContext);
    assert_eq!(
        shown_key(&shell),
        Some(TopicKey::command("CONFIG")),
        "HELP CONFIG must open cmd:CONFIG, not feature:configuration"
    );
}

// Validates: context-help Requirement 22.1 -- a non-root HELP argument still
// flows through open_help/resolve_help_argument (here `HELP CHANGE` opens
// cmd:CHANGE, which is a verb but not in builtin_ffcmd_roots()).
#[test]
fn help_non_root_still_resolves_via_open_help() {
    let mut shell = make_shell();
    seed_cmd_topic(&mut shell, "CHANGE");
    shell.handle_command("HELP CHANGE");
    assert_eq!(
        shown_key(&shell),
        Some(TopicKey::command("CHANGE")),
        "a non-root HELP arg resolves cmd:CHANGE via open_help as before"
    );
}

// Validates: context-help Requirement 22.1 -- `HELP OFF` still closes the Help
// Context (OFF is NOT a root, so open_root_help delegates to open_help where
// resolve_help_argument maps OFF -> Close).
#[test]
fn help_off_still_closes_via_open_help() {
    let mut shell = make_shell();
    shell.handle_command("HELP");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::HelpContext);
    shell.handle_command("HELP OFF");
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::HelpContext,
        "HELP OFF closes the Help Context (OFF is not a root; open_help handles it)"
    );
}

// Validates: command-semantics Requirement 15.1 -- `EXIT --help` opens the
// cmd:EXIT topic and does NOT execute EXIT (the app-close flag stays false).
#[test]
fn exit_help_flag_opens_help_without_executing() {
    let mut shell = make_shell();
    seed_cmd_topic(&mut shell, "EXIT");
    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "precondition: app is not already closing"
    );
    // The --help short-circuit lives on the single front door; exercise it via
    // the typed-line seam (dispatch_command_string), not the direct
    // handle_command ladder entry.
    shell.dispatch_command_string("EXIT --help");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::HelpContext);
    assert_eq!(shown_key(&shell), Some(TopicKey::command("EXIT")));
    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "`EXIT --help` opens help WITHOUT executing EXIT"
    );
}

// Validates: command-semantics Requirement 15.2 -- `-h` behaves identically to
// `--help`.
#[test]
fn exit_short_h_flag_opens_help_without_executing() {
    let mut shell = make_shell();
    seed_cmd_topic(&mut shell, "EXIT");
    shell.dispatch_command_string("EXIT -h");
    assert_eq!(shown_key(&shell), Some(TopicKey::command("EXIT")));
    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "`EXIT -h` opens help WITHOUT executing EXIT"
    );
}

// Validates: command-semantics Requirement 15.2 -- `/?` behaves identically to
// `--help`.
#[test]
fn exit_slash_question_flag_opens_help_without_executing() {
    let mut shell = make_shell();
    seed_cmd_topic(&mut shell, "EXIT");
    shell.dispatch_command_string("EXIT /?");
    assert_eq!(shown_key(&shell), Some(TopicKey::command("EXIT")));
    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "`EXIT /?` opens help WITHOUT executing EXIT"
    );
}

// Validates: command-semantics Requirement 15.3 -- the short-circuit runs AFTER
// rewrite_verb_to_root, so the FFCMD seed alias `SS` resolves to cmd:SNAPSHOT
// (the root), not cmd:SS.
#[test]
fn ss_help_flag_resolves_rewritten_root_snapshot() {
    let mut shell = make_shell();
    seed_cmd_topic(&mut shell, "SNAPSHOT");
    // Seed the SS -> SNAPSHOT alias on the ACTIVE environment's table so
    // rewrite_verb_to_root rewrites the surface to its root before the
    // short-circuit classifies it.
    let env = shell.active_environment_name();
    shell
        .i18n
        .alias_store
        .lock()
        .expect("alias store lock")
        .overlay_row_for_test(env, "SS", "SNAPSHOT");
    shell.dispatch_command_string("SS --help");
    assert_eq!(
        shown_key(&shell),
        Some(TopicKey::command("SNAPSHOT")),
        "SS --help opens cmd:SNAPSHOT because the alias rewrite runs first"
    );
}

// Validates: command-semantics Requirement 15.4 -- the help flag fires only
// when it is the FIRST argument token: `FIND --help foo` short-circuits, but
// `FIND foo --help` does NOT (so FIND runs normally and no help opens).
#[test]
fn help_flag_fires_only_as_first_arg_token() {
    let mut shell = make_shell();
    seed_cmd_topic(&mut shell, "FIND");
    // --help is the first arg token -> short-circuits to cmd:FIND.
    shell.dispatch_command_string("FIND --help foo");
    assert_eq!(
        shown_key(&shell),
        Some(TopicKey::command("FIND")),
        "`FIND --help foo` short-circuits to cmd:FIND"
    );

    // --help is NOT the first arg token -> no short-circuit, no Help Context.
    let mut shell2 = make_shell();
    seed_cmd_topic(&mut shell2, "FIND");
    shell2.dispatch_command_string("FIND foo --help");
    assert_ne!(
        shell2.tabs.active_tab().kind.tag(),
        KindTag::HelpContext,
        "`FIND foo --help` does NOT short-circuit (help flag is not the first arg token)"
    );
}

// Validates: command-semantics Requirement 15.1 -- a help flag on a root whose
// topic is missing degrades gracefully (no panic, no unrecognised-command
// error): the Help Context opens on the index-with-message miss path.
#[test]
fn help_flag_missing_topic_degrades_gracefully() {
    let mut shell = make_shell();
    // Empty registry (make_shell finds no shipped help beside the test binary).
    shell.dispatch_command_string("EXIT --help");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::HelpContext,
        "a missing cmd:EXIT topic still opens the Help Context (index + message)"
    );
    assert!(
        !*shell.should_close.lock().expect("close lock"),
        "a missing-topic help flag still does NOT execute EXIT"
    );
}

// Validates: command-semantics Requirement 15.9 -- two-word roots are a
// documented deferred limitation: `FILE CATALOGS --help` does NOT short-circuit
// on --help (the first ARG token after the single-whitespace verb split is
// `CATALOGS`, not a help flag), while `HELP FILE CATALOGS` resolves the
// cmd:FILE CATALOGS topic through the ladder path.
#[test]
fn two_word_root_help_flag_is_deferred_but_help_cmd_resolves() {
    let mut shell = make_shell();
    shell.seed_help_registry_for_test(vec![HelpTopic::new(
        TopicKey::command("FILE CATALOGS"),
        "FILE CATALOGS".to_string(),
        "Open the catalog explorer.".to_string(),
        TopicSource::FileBased {
            file_path: std::path::PathBuf::from("cmd.help.md"),
        },
    )]);
    // `HELP FILE CATALOGS` resolves the two-word cmd: topic through open_root_help.
    shell.handle_command("HELP FILE CATALOGS");
    assert_eq!(
        shown_key(&shell),
        Some(TopicKey::command("FILE CATALOGS")),
        "HELP FILE CATALOGS resolves cmd:FILE CATALOGS (two-word root reachable via HELP)"
    );
}
