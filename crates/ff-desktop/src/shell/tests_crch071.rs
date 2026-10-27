//! CR-CH-071 behaviour tests: ISPF Save/End/Cancel/Return/Refresh for editable
//! Contexts, the shared focused-editable-Context seam, the FFCMD-base verb
//! handling, and the retirement of the Alias Editor's private interpreter.
//!
//! These exercise the Alias Editor as the v1 seam implementor. The root
//! enumeration (`root_commands`/`contains_root`) and the no-shadow consequence
//! are proven in `environment_registry.rs` and `alias_editor.rs` respectively;
//! this module covers the full verb matrix and the single-dispatch-path closure.

use super::environment_ffedit::is_store_affecting_verb;
use super::environment_registry::{EnvironmentRegistry, FFCMD_NAME, FFEDIT_NAME};
use super::tests_common::make_shell;
use super::user_aliases::AliasEnvironment;
use super::WorkbenchShell;
use crate::alias_editor_panel::AliasRowEdit;
use crate::tab_state::KindTag;

/// Point the shell's User_Locale_Override_Store BASE at an isolated temp dir and
/// return the guard dir (kept alive by the caller).
///
/// CR-CH-072: the override is a per-locale DIR base, not a single `user.toml`.
fn point_user_aliases_at_temp(shell: &mut WorkbenchShell) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("tempdir");
    shell.dir_overrides.user_aliases = Some(dir.path().join("aliases"));
    dir
}

/// Build a valid FFCMD editor row (GG -> GSEARCH).
fn ffcmd_row(surface: &str, canonical: &str) -> AliasRowEdit {
    AliasRowEdit {
        surface: surface.to_string(),
        canonical: canonical.to_string(),
        environment: AliasEnvironment::Ffcmd,
        ..Default::default()
    }
}

/// Open the Alias Editor on an isolated temp store and return the guard dir.
fn shell_on_alias_editor() -> (WorkbenchShell, tempfile::TempDir) {
    let mut shell = make_shell();
    let dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "fixture must land on the Alias Editor"
    );
    (shell, dir)
}

// === SAVE: persist + STAY (Req 20a.1, 20a.2) ===============================

// Validates: command-environments Requirement 20a.1, 20a.2 -- SAVE on the Alias
// Editor persists the working copy (file written, live store refreshed) and
// STAYS in the Context.
#[test]
fn save_persists_and_stays_on_alias_editor() {
    let (mut shell, dir) = shell_on_alias_editor();
    shell.alias_editor_panel.rows = vec![ffcmd_row("GG", "GSEARCH")];
    shell.dispatch_command_string("SAVE");
    // Stayed in the Context.
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "SAVE stays in the Alias Editor (Req 20a.1)"
    );
    // The file was written.
    // CR-CH-072: the FFCMD override writes to <base>/en/FFCMD.toml.
    let path = super::user_aliases::user_alias_override_file(
        &dir.path().join("aliases"),
        "en",
        FFCMD_NAME,
    );
    assert!(
        path.exists(),
        "SAVE wrote the user override file (Req 20a.2)"
    );
    assert!(shell.alias_editor_panel.error.is_none());
    // Live store refreshed -- the alias resolves at once.
    use super::locale_resolver::LocaleResolveOutcome;
    assert_eq!(
        shell.resolve_locale_verb("GG", "FFCMD"),
        LocaleResolveOutcome::Root("GSEARCH".to_string()),
        "SAVE refreshed the live resolver (Req 20a.2)"
    );
}

// === END: persist THEN leave; on failure STAY (Req 20a.1) ==================

// Validates: command-environments Requirement 20a.1 -- END on the Alias Editor
// SAVES the working copy then LEAVES via the per-tab Navigation_Stack.
#[test]
fn end_persists_then_leaves_on_alias_editor() {
    let (mut shell, dir) = shell_on_alias_editor();
    let tabs_before = shell.tabs.len();
    shell.alias_editor_panel.rows = vec![ffcmd_row("GG", "GSEARCH")];
    shell.handle_command("END");
    // Left the Context (no tab spawned).
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "END leaves the Alias Editor after saving (Req 20a.1)"
    );
    assert_eq!(shell.tabs.len(), tabs_before, "END did not spawn a tab");
    // The save happened before leaving (file written to the per-locale override).
    let path = super::user_aliases::user_alias_override_file(
        &dir.path().join("aliases"),
        "en",
        FFCMD_NAME,
    );
    assert!(path.exists(), "END saved before leaving (Req 20a.1)");
}

// Validates: command-environments Requirement 20a.1 -- END on a save that FAILS
// validation STAYS in the Context with an error surfaced, and the on-disk store
// is UNCHANGED (no write of the invalid set).
#[test]
fn end_on_validation_failure_stays_and_leaves_store_unchanged() {
    let (mut shell, dir) = shell_on_alias_editor();
    // CONFIG is a reserved built-in root -> the row is rejected at save.
    shell.alias_editor_panel.rows = vec![ffcmd_row("CONFIG", "EDIT")];
    shell.handle_command("END");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "END stays when the save fails validation (Req 20a.1)"
    );
    assert!(
        shell.alias_editor_panel.error.is_some(),
        "the validation error is surfaced inline"
    );
    let path = super::user_aliases::user_alias_override_file(
        &dir.path().join("aliases"),
        "en",
        FFCMD_NAME,
    );
    assert!(
        !path.exists(),
        "a failed END save leaves the store unchanged (not written)"
    );
}

// === CANCEL / RETURN: discard THEN leave, NO save (Req 20a.3) ==============

// Validates: command-environments Requirement 20a.3 -- CANCEL on the Alias
// Editor DISCARDS the working edits (not persisted) and leaves via nav_end.
#[test]
fn cancel_discards_and_leaves_without_saving() {
    let (mut shell, dir) = shell_on_alias_editor();
    shell.alias_editor_panel.rows = vec![ffcmd_row("GG", "GSEARCH")];
    shell.handle_command("CANCEL");
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "CANCEL leaves the Context (Req 20a.3)"
    );
    let path = super::user_aliases::user_alias_override_file(
        &dir.path().join("aliases"),
        "en",
        FFCMD_NAME,
    );
    assert!(
        !path.exists(),
        "CANCEL did NOT persist the working edits (Req 20a.3)"
    );
}

// Validates: command-environments Requirement 20a.3 -- RETURN on the Alias
// Editor also discards the working edits (not persisted) and leaves.
#[test]
fn return_discards_and_leaves_without_saving() {
    let (mut shell, dir) = shell_on_alias_editor();
    shell.alias_editor_panel.rows = vec![ffcmd_row("GG", "GSEARCH")];
    shell.handle_command("RETURN");
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "RETURN leaves the Context (Req 20a.3)"
    );
    let path = super::user_aliases::user_alias_override_file(
        &dir.path().join("aliases"),
        "en",
        FFCMD_NAME,
    );
    assert!(
        !path.exists(),
        "RETURN did NOT persist the working edits (Req 20a.3)"
    );
}

// === REFRESH: discard + reload last-saved + STAY (Req 20a.4) ===============

// Validates: command-environments Requirement 20a.4 -- REFRESH discards the
// unsaved working edits, RELOADS the last-saved state from disk, and STAYS.
#[test]
fn refresh_discards_edits_reloads_last_saved_and_stays() {
    let (mut shell, _dir) = shell_on_alias_editor();
    // Save a known baseline (GG -> GSEARCH).
    shell.alias_editor_panel.rows = vec![ffcmd_row("GG", "GSEARCH")];
    shell.dispatch_command_string("SAVE");
    assert!(shell.alias_editor_panel.error.is_none());
    // Make an UNSAVED edit (add a second row).
    shell.alias_editor_panel.rows.push(ffcmd_row("LL", "LOG"));
    assert_eq!(shell.alias_editor_panel.rows.len(), 2);
    // REFRESH: drop the unsaved edit and reload the last-saved single row.
    shell.dispatch_command_string("REFRESH");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "REFRESH stays in the Context (Req 20a.4)"
    );
    assert_eq!(
        shell.alias_editor_panel.rows.len(),
        1,
        "REFRESH dropped the unsaved edit and reloaded last-saved (Req 20a.4)"
    );
    assert_eq!(shell.alias_editor_panel.rows[0].surface, "GG");
}

// === Non-editable Context: SAVE/REFRESH no-op, END/RETURN plain nav (20a.14)

// Validates: command-environments Requirement 20a.14 -- on a NON-editable
// Context (the POM) SAVE and REFRESH are no-ops (the seam is not engaged) and do
// not leave the Context.
#[test]
fn save_and_refresh_are_noops_on_non_editable_context() {
    let mut shell = make_shell();
    // Fresh shell is on the POM (a non-editable Context).
    assert!(shell.editable_context_kind().is_none());
    let kind_before = shell.tabs.active_tab().kind.tag();
    shell.dispatch_command_string("SAVE");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        kind_before,
        "SAVE is a no-op on a non-editable Context (Req 20a.14)"
    );
    shell.dispatch_command_string("REFRESH");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        kind_before,
        "REFRESH is a no-op on a non-editable Context (Req 20a.14)"
    );
}

// === FFEDIT buffer SAVE active-wins boundary unchanged (Req 20a.7, 20a.8) ==

// Validates: command-environments Requirement 20a.7, 20a.8 -- the active-wins
// boundary is intact: SAVE remains the one store-affecting FFEDIT verb (so on an
// editor BUFFER FFEDIT claims SAVE FIRST, byte-identical), AND an editor-buffer
// kind is NOT in the FFCMD editable-Context set, so the new FFCMD SAVE root never
// shadows the buffer SAVE. `is_ffedit_active` is true for editor buffers, so the
// FFEDIT claim runs before the FFCMD ladder on a buffer.
#[test]
fn ffedit_buffer_save_active_wins_boundary_is_unchanged() {
    // SAVE is still the store-affecting FFEDIT verb (FFEDIT owns the buffer SAVE).
    assert!(
        is_store_affecting_verb("SAVE"),
        "FFEDIT still owns the store-affecting buffer SAVE (Req 20a.7)"
    );
    // SAVE/CANCEL/REFRESH are FFCMD roots, NOT FFEDIT roots -- so they cannot be
    // claimed as FFEDIT in-buffer verbs. (SAVE is an FFEDIT root for the buffer
    // store-write; CANCEL/REFRESH are NOT FFEDIT roots in v1, Req 20a.8.)
    assert!(
        !EnvironmentRegistry::contains_root(FFEDIT_NAME, "CANCEL"),
        "CANCEL is not an FFEDIT root in v1 (Req 20a.8)"
    );
    assert!(
        !EnvironmentRegistry::contains_root(FFEDIT_NAME, "REFRESH"),
        "REFRESH is not an FFEDIT root in v1 (Req 20a.8)"
    );
    // The editable-Context set excludes editor-buffer kinds: an editor buffer is
    // never treated as an editable non-buffer Context by the FFCMD seam. Proven
    // via the registry active-wins predicate: editor buffers are FFEDIT-active,
    // so FFEDIT claims SAVE before the FFCMD ladder ever sees it.
    let reg = EnvironmentRegistry::with_builtins();
    assert!(
        reg.is_ffedit_active(KindTag::FileEditor, false),
        "an editor buffer is FFEDIT-active, so FFEDIT claims SAVE first (Req 20a.7)"
    );
    assert!(
        reg.is_ffedit_active(KindTag::Untitled, false),
        "an untitled editor buffer is FFEDIT-active (Req 20a.7)"
    );
    // SAVE IS an FFCMD root too (the editable-Context seam verb), but it is
    // reached only on a NON-buffer Context because FFEDIT wins first on a buffer.
    assert!(EnvironmentRegistry::contains_root(FFCMD_NAME, "SAVE"));
}

// === One dispatch path: typed line == F3/global END (Req 20a.10) ===========

// Validates: command-environments Requirement 20a.10 -- the Alias Editor's
// private SAVE/CANCEL/END interpreter is retired: a typed `Command ===>` END and
// a globally-typed END produce the SAME result (save-then-leave). The typed line
// routes through the single FFCMD front door, closing the two-parallel-paths
// divergence.
#[test]
fn typed_command_line_end_matches_global_end() {
    use crate::alias_editor_panel::AliasEditorAction;

    // Path A: the typed `Command ===>` line END (via the panel action).
    let (mut shell_a, dir_a) = shell_on_alias_editor();
    shell_a.alias_editor_panel.rows = vec![ffcmd_row("GG", "GSEARCH")];
    shell_a.apply_alias_editor_action(AliasEditorAction::CommandLine("END".to_string()));
    let left_a = shell_a.tabs.active_tab().kind.tag() != KindTag::AliasEditor;
    let saved_a = super::user_aliases::user_alias_override_file(
        &dir_a.path().join("aliases"),
        "en",
        FFCMD_NAME,
    )
    .exists();

    // Path B: a globally-typed END (F3-equivalent via handle_command).
    let (mut shell_b, dir_b) = shell_on_alias_editor();
    shell_b.alias_editor_panel.rows = vec![ffcmd_row("GG", "GSEARCH")];
    shell_b.handle_command("END");
    let left_b = shell_b.tabs.active_tab().kind.tag() != KindTag::AliasEditor;
    let saved_b = super::user_aliases::user_alias_override_file(
        &dir_b.path().join("aliases"),
        "en",
        FFCMD_NAME,
    )
    .exists();

    assert!(left_a && left_b, "both END paths leave the Context");
    assert!(saved_a && saved_b, "both END paths save before leaving");
    assert_eq!(
        (left_a, saved_a),
        (left_b, saved_b),
        "the typed Command line END and global END produce the SAME result (Req 20a.10)"
    );
}

// Validates: command-environments Requirement 20a.10 -- a typed `Command ===>`
// CANCEL discards + leaves and REFRESH discards + reloads + stays through the
// single front door (the retired private arms no longer intercept them).
#[test]
fn typed_command_line_cancel_and_refresh_route_through_front_door() {
    use crate::alias_editor_panel::AliasEditorAction;

    // CANCEL via the typed line discards + leaves (no save).
    let (mut shell, dir) = shell_on_alias_editor();
    shell.alias_editor_panel.rows = vec![ffcmd_row("GG", "GSEARCH")];
    shell.apply_alias_editor_action(AliasEditorAction::CommandLine("CANCEL".to_string()));
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "typed CANCEL leaves via the single path (Req 20a.10)"
    );
    assert!(
        !super::user_aliases::user_alias_override_file(
            &dir.path().join("aliases"),
            "en",
            FFCMD_NAME
        )
        .exists(),
        "typed CANCEL did not save (Req 20a.3/20a.10)"
    );

    // REFRESH via the typed line discards + reloads + stays.
    let (mut shell2, _dir2) = shell_on_alias_editor();
    shell2.alias_editor_panel.rows = vec![ffcmd_row("GG", "GSEARCH")];
    shell2.dispatch_command_string("SAVE");
    shell2.alias_editor_panel.rows.push(ffcmd_row("LL", "LOG"));
    shell2.apply_alias_editor_action(AliasEditorAction::CommandLine("REFRESH".to_string()));
    assert_eq!(
        shell2.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "typed REFRESH stays (Req 20a.10)"
    );
    assert_eq!(
        shell2.alias_editor_panel.rows.len(),
        1,
        "typed REFRESH reloaded last-saved (Req 20a.4/20a.10)"
    );
}

// === EXIT app-exit family unchanged (Req 20a.13) ===========================

// Validates: command-environments Requirement 20a.13 -- EXIT remains the
// app-exit family and is NOT repurposed as a save-then-leave verb. EXIT / QUIT /
// LOGOFF are still FFCMD roots (unchanged) and SAVE / CANCEL / REFRESH did not
// displace them.
#[test]
fn exit_family_remains_app_exit_not_context_save_then_leave() {
    for verb in ["EXIT", "QUIT", "LOGOFF"] {
        assert!(
            EnvironmentRegistry::contains_root(FFCMD_NAME, verb),
            "{verb} remains an FFCMD app-exit root (Req 20a.13)"
        );
    }
    // The save-then-leave verb for an editable Context is END, not EXIT: END is a
    // root, and EXIT is not the editable-Context verb.
    assert!(EnvironmentRegistry::contains_root(FFCMD_NAME, "END"));
}

// === No-shadow at LOAD for SAVE/CANCEL/REFRESH (Req 20a.12) ================

// Validates: command-environments Requirement 20a.12 -- a user alias row whose
// surface equals SAVE / CANCEL / REFRESH is SKIPPED-with-WARN at LOAD (it never
// shadows the built-in root), because those three are now `builtin_ffcmd_roots()`
// members (CR-CH-071 Req 20a.5) and the load-side no-shadow guard reads that set.
// The shadowing rows are written DIRECTLY to the file (bypassing save-time
// validation) so the LOAD path is what enforces it.
#[test]
fn load_skips_user_rows_shadowing_save_cancel_refresh() {
    use super::locale_resolver::LocaleResolveOutcome;
    use super::user_aliases::{serialise_user_aliases, UserAliasRowToml};

    let mut shell = make_shell();
    let dir = point_user_aliases_at_temp(&mut shell);
    // CR-CH-072: the override is written to the per-locale FFCMD override file
    // `<base>/en/FFCMD.toml`, and rows carry no localisation_key.
    let path = super::user_aliases::user_alias_override_file(
        &dir.path().join("aliases"),
        "en",
        FFCMD_NAME,
    );
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mk = |surface: &str, canonical: &str| UserAliasRowToml {
        surface: surface.to_string(),
        canonical: canonical.to_string(),
        environment: AliasEnvironment::Ffcmd,
        command_template: String::new(),
    };
    // Each shadowing row names a valid root (EDIT) so ONLY the no-shadow guard
    // can reject it; plus one genuinely valid row that MUST still load.
    let rows = vec![
        mk("SAVE", "EDIT"),
        mk("CANCEL", "EDIT"),
        mk("REFRESH", "EDIT"),
        mk("GG", "GSEARCH"),
    ];
    std::fs::write(&path, serialise_user_aliases(&rows).unwrap()).unwrap();
    shell.refresh_alias_store_in_place();
    // The shadowing surfaces still self-resolve to their built-in root (the user
    // rows were skipped), never to the shadowing EDIT.
    for verb in ["SAVE", "CANCEL", "REFRESH"] {
        assert_eq!(
            shell.resolve_locale_verb(verb, "FFCMD"),
            LocaleResolveOutcome::Root(verb.to_string()),
            "a user row shadowing {verb} is skipped at load (Req 20a.12)"
        );
    }
    // The valid row alongside them still loaded.
    assert_eq!(
        shell.resolve_locale_verb("GG", "FFCMD"),
        LocaleResolveOutcome::Root("GSEARCH".to_string()),
        "a valid row still loads after shadowing rows are skipped"
    );
}

// === Sibling editors unchanged in v1 (Req 20a.11 fast-follow) ==============

// Validates: command-environments Requirement 20a.11, 20a.14 -- the sibling
// config editors are NOT wired in v1: the Menus Editor is NOT recognised by the
// editable-Context seam yet, so END on it behaves exactly as today (plain
// navigation, no seam engagement). Guards against accidental sibling regression.
#[test]
fn menus_editor_is_not_wired_into_the_seam_in_v1() {
    let (shell, _dir) = super::tests_common::make_shell_with_menus_editor();
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::MenusEditor,
        "fixture lands on the Menus Editor"
    );
    assert!(
        shell.editable_context_kind().is_none(),
        "the Menus Editor is a fast-follow sibling, not wired into the seam in v1 (Req 20a.11)"
    );
}
