//! Shell-level tests for the Alias Editor Context (command-configurator Req 5,
//! CR-NR-113): open/add/save, the live-resolver refresh, the third-overlay load
//! with routing + no-shadow + cross-layer discipline, and the end-to-end
//! parameterised-alias expansion.

use super::tests_common::make_shell;
use super::user_aliases::{AliasEnvironment, UserAliasRowToml};
use super::WorkbenchShell;
use crate::alias_editor_panel::{AliasEditorAction, AliasRowEdit};
use crate::tab_state::KindTag;

/// Point the shell's User_Alias_Store at an isolated temp file and return the
/// guard dir (kept alive by the caller).
fn point_user_aliases_at_temp(shell: &mut WorkbenchShell) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("tempdir");
    shell.dir_overrides.user_aliases = Some(dir.path().join("aliases").join("user.toml"));
    dir
}

/// Build an editor row.
fn row(surface: &str, canonical: &str, env: AliasEnvironment, template: &str) -> AliasRowEdit {
    AliasRowEdit {
        surface: surface.to_string(),
        canonical: canonical.to_string(),
        environment: env,
        command_template: template.to_string(),
    }
}

// Validates: command-configurator Requirement 5.2 -- the ALIASES verb opens the
// Alias Editor Context (a CommandConfigurator editor = "aliases") through the
// single front door; no new WorkspaceKind.
#[test]
fn aliases_verb_opens_the_alias_editor_context() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "ALIASES opens the Alias Editor Context"
    );
    assert_eq!(shell.tabs.active_tab().title, "[ALIASES]");
}

// Validates: menu-workspace Requirement 22.2 (CR-CH-066) -- selecting the
// Settings option with mnemonic `A` dispatches the `ALIASES` verb through the
// SAME single command-dispatch front door and opens the Alias Editor Context
// (CR-NR-113), introducing no new verb and no new dispatcher. Selection goes
// through the single Option-Selection path (the active Settings Menu_Workspace
// resolves the option key to its command).
#[test]
fn settings_alias_option_dispatches_aliases_verb() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.handle_command("SETTINGS");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::MenuWorkspace,
        "SETTINGS opens the Settings Menu_Workspace"
    );
    // Select the `A` option by its key (same path as a click / typed Option_Key).
    shell.handle_command("A");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "selecting Settings option A must open the Alias Editor Context (CR-NR-113)"
    );
    assert_eq!(shell.tabs.active_tab().title, "[ALIASES]");
}

// Validates: command-configurator Requirement 5.2, 5.9 -- the tab title is
// [ALIASES] and END returns via the per-tab Navigation_Stack to the opener
// (parity with the Menus Editor).
#[test]
fn alias_editor_end_returns_to_opener_via_nav_stack() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    let tabs_before = shell.tabs.len();
    // Open the Alias Editor IN PLACE (navigate_to pushes the opener onto the
    // per-tab Navigation_Stack); the title is [ALIASES].
    shell.dispatch_command_string("ALIASES");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::AliasEditor);
    assert_eq!(shell.tabs.active_tab().title, "[ALIASES]");
    assert_eq!(
        shell.tabs.len(),
        tabs_before,
        "opening in place never spawned a tab"
    );
    assert!(
        !shell.tabs.active_tab().nav_stack.is_empty(),
        "the opener was pushed onto the Navigation_Stack"
    );
    // END pops one Navigation_Stack level back to the opener Context (parity with
    // the Menus Editor): the tab is no longer the Alias Editor and no tab spawned.
    shell.handle_command("END");
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "END returns to the opener Context via the Navigation_Stack"
    );
    assert_eq!(
        shell.tabs.len(),
        tabs_before,
        "returning never spawned a tab"
    );
}

// Validates: command-configurator Requirement 5.3, 5.5 -- Add appends a blank
// row and Delete removes a row from the working copy.
#[test]
fn add_and_delete_mutate_the_working_copy() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    assert_eq!(shell.alias_editor_panel.rows.len(), 0);
    shell.apply_alias_editor_action(AliasEditorAction::AddRow);
    shell.apply_alias_editor_action(AliasEditorAction::AddRow);
    assert_eq!(shell.alias_editor_panel.rows.len(), 2);
    shell.apply_alias_editor_action(AliasEditorAction::DeleteRow(0));
    assert_eq!(shell.alias_editor_panel.rows.len(), 1);
}

// Validates: command-configurator Requirement 5.1, 5.4, 5.6 -- a valid Save
// writes the user store file (created on first save) and the live resolver
// refreshes in place so the alias resolves immediately.
#[test]
fn save_writes_file_and_live_resolver_resolves_new_alias() {
    let mut shell = make_shell();
    let dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    // Author an FFCMD shorthand GG -> GSEARCH.
    shell.alias_editor_panel.rows = vec![row("GG", "GSEARCH", AliasEnvironment::Ffcmd, "")];
    shell.apply_alias_editor_action(AliasEditorAction::Save);
    assert!(
        shell.alias_editor_panel.error.is_none(),
        "a valid save has no error: {:?}",
        shell.alias_editor_panel.error
    );
    // The file was created on first save (Req 5.1).
    let path = dir.path().join("aliases").join("user.toml");
    assert!(path.exists(), "user.toml created on first save");
    // The live resolver resolves the new alias at once (Req 5.6), routed to FFCMD.
    use super::locale_resolver::LocaleResolveOutcome;
    assert_eq!(
        shell.resolve_locale_verb("GG", "FFCMD"),
        LocaleResolveOutcome::Root("GSEARCH".to_string()),
        "the saved alias resolves in FFCMD without a restart"
    );
}

// Validates: command-configurator Requirement 5.6 -- the in-place refresh is a
// re-seed that re-applies BOTH the base/per-locale layer AND the user-store
// overlay: after save the built-in FFCMD roots STILL resolve (base not dropped)
// AND the new user alias resolves (user layer applied).
#[test]
fn refresh_keeps_base_layer_and_adds_user_layer() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    shell.alias_editor_panel.rows = vec![row("GG", "GSEARCH", AliasEnvironment::Ffcmd, "")];
    shell.apply_alias_editor_action(AliasEditorAction::Save);
    use super::locale_resolver::LocaleResolveOutcome;
    // Base layer retained: a built-in FFCMD root still self-resolves.
    assert_eq!(
        shell.resolve_locale_verb("CONFIG", "FFCMD"),
        LocaleResolveOutcome::Root("CONFIG".to_string()),
        "the re-seed did not drop the identity/base layer"
    );
    // User layer applied in the same re-seed.
    assert_eq!(
        shell.resolve_locale_verb("GG", "FFCMD"),
        LocaleResolveOutcome::Root("GSEARCH".to_string())
    );
}

// Validates: command-configurator Requirement 5.7 -- a row with
// environment = "ffedit" loads into the FFEDIT table and resolves there; a
// defaulted row loads into FFCMD. Routing is per-row.
#[test]
fn user_rows_route_into_their_target_environment() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    shell.alias_editor_panel.rows = vec![
        row("ECOB", "EDIT", AliasEnvironment::Ffedit, ""),
        row("GG", "GSEARCH", AliasEnvironment::Ffcmd, ""),
    ];
    shell.apply_alias_editor_action(AliasEditorAction::Save);
    assert!(shell.alias_editor_panel.error.is_none());
    use super::locale_resolver::LocaleResolveOutcome;
    // ECOB resolves in FFEDIT (its target environment)...
    assert_eq!(
        shell.resolve_locale_verb("ECOB", "FFEDIT"),
        LocaleResolveOutcome::Root("EDIT".to_string())
    );
    // ...and NOT in FFCMD.
    assert_eq!(
        shell.resolve_locale_verb("ECOB", "FFCMD"),
        LocaleResolveOutcome::Unknown("ECOB".to_string())
    );
    // GG resolves in FFCMD.
    assert_eq!(
        shell.resolve_locale_verb("GG", "FFCMD"),
        LocaleResolveOutcome::Root("GSEARCH".to_string())
    );
}

// Validates: command-configurator Requirement 5.7 -- a user row whose surface
// equals a reserved built-in root (the NEW CR-NR-113 no-shadow invariant) is
// skipped at LOAD with a WARN, so it never shadows the built-in. The LOAD path
// enforces it even when a file reaches disk by other means.
#[test]
fn load_skips_user_row_that_shadows_builtin_root() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    // Write a shadowing row DIRECTLY to the file (bypassing save-time validation).
    let path = shell.user_aliases_path().expect("path");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let rows = vec![UserAliasRowToml {
        surface: "CONFIG".to_string(),
        canonical: "EDIT".to_string(),
        environment: AliasEnvironment::Ffcmd,
        command_template: String::new(),
        localisation_key: String::new(),
    }];
    std::fs::write(
        &path,
        super::user_aliases::serialise_user_aliases(&rows).unwrap(),
    )
    .unwrap();
    // Refresh the live store from the file.
    shell.refresh_alias_store_in_place();
    use super::locale_resolver::LocaleResolveOutcome;
    // CONFIG still resolves to CONFIG (the built-in root), NOT to the shadowing
    // EDIT: the user row was skipped.
    assert_eq!(
        shell.resolve_locale_verb("CONFIG", "FFCMD"),
        LocaleResolveOutcome::Root("CONFIG".to_string()),
        "a user row shadowing a built-in root is skipped at load"
    );
}

// Validates: command-configurator Requirement 5.4 -- a Save that fails
// validation (built-in-root shadow) surfaces an error and leaves the on-disk
// file UNCHANGED.
#[test]
fn save_validation_failure_leaves_file_unchanged() {
    let mut shell = make_shell();
    let dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    // First, a valid save establishes a known file.
    shell.alias_editor_panel.rows = vec![row("GG", "GSEARCH", AliasEnvironment::Ffcmd, "")];
    shell.apply_alias_editor_action(AliasEditorAction::Save);
    let path = dir.path().join("aliases").join("user.toml");
    let before = std::fs::read_to_string(&path).expect("file written");
    // Now author an invalid set (CONFIG shadows a built-in root) and Save.
    shell.alias_editor_panel.rows = vec![row("CONFIG", "EDIT", AliasEnvironment::Ffcmd, "")];
    shell.apply_alias_editor_action(AliasEditorAction::Save);
    assert!(
        shell.alias_editor_panel.error.is_some(),
        "an invalid save surfaces an error"
    );
    let after = std::fs::read_to_string(&path).expect("file still present");
    assert_eq!(
        before, after,
        "a failed save leaves the on-disk file unchanged"
    );
}

// Validates: command-configurator Requirement 5.8 -- a parameterised alias
// authored in the editor expands/dispatches per CR-NR-112: ECOB -> EDIT with
// template FFWB.COBOL.MASTER(&1), saved, then `ecob helloW` rewrites to
// `EDIT FFWB.COBOL.MASTER(helloW)` through the existing pre-pass (no new
// behaviour).
#[test]
fn parameterised_alias_authored_in_editor_expands_end_to_end() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    // Author the parameterised alias. At a fresh shell the active tab is the POM,
    // whose environment is FFCMD, so author the row in FFCMD so the active-env
    // pre-pass (which the front door uses) sees it; the ffcmd/ffedit ROUTING is
    // separately proven by `user_rows_route_into_their_target_environment`.
    let env = shell.active_environment_name();
    let environment = if env == "FFEDIT" {
        AliasEnvironment::Ffedit
    } else {
        AliasEnvironment::Ffcmd
    };
    shell.alias_editor_panel.rows = vec![row("ECOB", "EDIT", environment, "FFWB.COBOL.MASTER(&1)")];
    shell.apply_alias_editor_action(AliasEditorAction::Save);
    assert!(shell.alias_editor_panel.error.is_none());
    // The authored parameterised alias expands through the EXISTING CR-NR-112
    // pre-pass (no new behaviour): `ecob helloW` -> `EDIT FFWB.COBOL.MASTER(helloW)`.
    assert_eq!(
        shell.rewrite_verb_to_root("ecob helloW"),
        "EDIT FFWB.COBOL.MASTER(helloW)",
        "the authored parameterised alias expands per CR-NR-112 (Req 5.8)"
    );
}
