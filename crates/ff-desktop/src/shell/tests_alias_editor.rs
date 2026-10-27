//! Shell-level tests for the Alias Editor Context (command-configurator Req 5 +
//! Req 6 CR-CH-072): open/add/save, the live-resolver refresh, the per-locale
//! override load with no-shadow + root-validity discipline, and the end-to-end
//! parameterised-alias expansion.
//!
//! CR-CH-072: the Alias Editor now edits the ACTIVE-locale EFFECTIVE table
//! (shipped seed overlaid by the User_Locale_Override_Store) and WRITES to the
//! per-locale override `<base>/<locale>/<ENV>.toml`. These tests were updated
//! from the superseded CR-NR-113 flat `user.toml` model; each supersession is
//! noted inline with a `// CR-CH-072:` comment.

use super::tests_common::make_shell;
use super::user_aliases::{self, AliasEnvironment, UserAliasRowToml};
use super::WorkbenchShell;
use crate::alias_editor_panel::{AliasEditorAction, AliasRowEdit};
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

/// The per-locale override file for the active locale + FFCMD under the temp base.
fn override_file(dir: &tempfile::TempDir, locale: &str, env_name: &str) -> std::path::PathBuf {
    user_aliases::user_alias_override_file(&dir.path().join("aliases"), locale, env_name)
}

/// Build an editor row.
fn row(surface: &str, canonical: &str, env: AliasEnvironment, template: &str) -> AliasRowEdit {
    AliasRowEdit {
        surface: surface.to_string(),
        canonical: canonical.to_string(),
        environment: env,
        command_template: template.to_string(),
        ..Default::default()
    }
}

/// Build an override row (the per-locale file row shape, no localisation_key).
fn override_row(surface: &str, canonical: &str, env: AliasEnvironment) -> UserAliasRowToml {
    UserAliasRowToml {
        surface: surface.to_string(),
        canonical: canonical.to_string(),
        environment: env,
        command_template: String::new(),
    }
}

/// Run a `Command ===>` SAVE (the redesign's save path): apply pending line
/// commands then validate + write (Req 5.14, 5.22).
fn save(shell: &mut WorkbenchShell) {
    shell.apply_alias_editor_action(AliasEditorAction::CommandLine("SAVE".to_string()));
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
// SAME single command-dispatch front door and opens the Alias Editor Context,
// introducing no new verb and no new dispatcher.
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
    shell.handle_command("A");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "selecting Settings option A must open the Alias Editor Context"
    );
    assert_eq!(shell.tabs.active_tab().title, "[ALIASES]");
}

// Validates: command-configurator Requirement 5.2, 5.9 -- the tab title is
// [ALIASES] and END returns via the per-tab Navigation_Stack to the opener.
#[test]
fn alias_editor_end_returns_to_opener_via_nav_stack() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    let tabs_before = shell.tabs.len();
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

// Validates: command-configurator Requirement 5.16 -- the ISPF line commands
// supersede the footer Add/Delete: an `I` Act inserts a blank row and a `D` Act
// deletes a row when the batch is applied (CR-CH-068).
//
// CR-CH-072: under `en` with no shipped seed the effective table is empty, so
// the editor opens with zero rows (the fix observed for en; af opens populated).
#[test]
fn line_commands_insert_and_delete_the_working_copy() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    assert_eq!(shell.alias_editor_panel.rows.len(), 0);
    let mut r = row("ECOB", "EDIT", AliasEnvironment::Ffcmd, "");
    r.act = "I".to_string();
    shell.alias_editor_panel.rows = vec![r];
    shell.apply_alias_editor_action(AliasEditorAction::ApplyLineCommands);
    assert_eq!(shell.alias_editor_panel.rows.len(), 2);
    shell.alias_editor_panel.rows[0].act = "D".to_string();
    shell.apply_alias_editor_action(AliasEditorAction::ApplyLineCommands);
    assert_eq!(shell.alias_editor_panel.rows.len(), 1);
}

// Validates: command-configurator Requirement 6.3, 6.4 -- a valid Save writes the
// per-locale override file (created on first save) and the live resolver
// refreshes in place so the alias resolves immediately.
//
// CR-CH-072: the save target is `<base>/<locale>/FFCMD.toml`, not `user.toml`.
#[test]
fn save_writes_file_and_live_resolver_resolves_new_alias() {
    let mut shell = make_shell();
    let dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    shell.alias_editor_panel.rows = vec![row("GG", "GSEARCH", AliasEnvironment::Ffcmd, "")];
    save(&mut shell);
    assert!(
        shell.alias_editor_panel.error.is_none(),
        "a valid save has no error: {:?}",
        shell.alias_editor_panel.error
    );
    let path = override_file(&dir, "en", "FFCMD");
    assert!(
        path.exists(),
        "the per-locale FFCMD override file is created on first save"
    );
    use super::locale_resolver::LocaleResolveOutcome;
    assert_eq!(
        shell.resolve_locale_verb("GG", "FFCMD"),
        LocaleResolveOutcome::Root("GSEARCH".to_string()),
        "the saved alias resolves in FFCMD without a restart"
    );
}

// Validates: command-configurator Requirement 6.4 -- the in-place refresh is a
// re-seed that re-applies BOTH the seed layer AND the user override: after save
// the built-in FFCMD roots STILL resolve AND the new user alias resolves.
#[test]
fn refresh_keeps_base_layer_and_adds_user_layer() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    shell.alias_editor_panel.rows = vec![row("GG", "GSEARCH", AliasEnvironment::Ffcmd, "")];
    save(&mut shell);
    use super::locale_resolver::LocaleResolveOutcome;
    assert_eq!(
        shell.resolve_locale_verb("CONFIG", "FFCMD"),
        LocaleResolveOutcome::Root("CONFIG".to_string()),
        "the re-seed did not drop the identity/base layer"
    );
    assert_eq!(
        shell.resolve_locale_verb("GG", "FFCMD"),
        LocaleResolveOutcome::Root("GSEARCH".to_string())
    );
}

// Validates: command-configurator Requirement 6.3 -- a row with
// environment = ffedit is grouped into the FFEDIT override file and resolves in
// FFEDIT; a defaulted row goes into the FFCMD file. Routing is per-row (grouped
// into per-ENV override files by the save).
#[test]
fn user_rows_route_into_their_target_environment() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    shell.alias_editor_panel.rows = vec![
        row("SOEK", "FIND", AliasEnvironment::Ffedit, ""),
        row("GG", "GSEARCH", AliasEnvironment::Ffcmd, ""),
    ];
    save(&mut shell);
    assert!(shell.alias_editor_panel.error.is_none());
    use super::locale_resolver::LocaleResolveOutcome;
    assert_eq!(
        shell.resolve_locale_verb("SOEK", "FFEDIT"),
        LocaleResolveOutcome::Root("FIND".to_string())
    );
    assert_eq!(
        shell.resolve_locale_verb("SOEK", "FFCMD"),
        LocaleResolveOutcome::Unknown("SOEK".to_string())
    );
    assert_eq!(
        shell.resolve_locale_verb("GG", "FFCMD"),
        LocaleResolveOutcome::Root("GSEARCH".to_string())
    );
}

// Validates: localization Requirement 13.5 -- a user override row whose surface
// equals a reserved built-in root (the no-shadow invariant) is skipped at LOAD
// with a WARN, so it never shadows the built-in. The LOAD path enforces it even
// when a file reaches disk by other means.
//
// CR-CH-072: the shadowing row is written to the per-locale FFCMD override file.
#[test]
fn load_skips_user_row_that_shadows_builtin_root() {
    let mut shell = make_shell();
    let dir = point_user_aliases_at_temp(&mut shell);
    let path = override_file(&dir, "en", "FFCMD");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let rows = vec![override_row("CONFIG", "EDIT", AliasEnvironment::Ffcmd)];
    std::fs::write(&path, user_aliases::serialise_user_aliases(&rows).unwrap()).unwrap();
    shell.refresh_alias_store_in_place();
    use super::locale_resolver::LocaleResolveOutcome;
    assert_eq!(
        shell.resolve_locale_verb("CONFIG", "FFCMD"),
        LocaleResolveOutcome::Root("CONFIG".to_string()),
        "a user override shadowing a built-in root is skipped at load"
    );
}

// Validates: command-environments Requirement 20.7 (CR-CH-067) -- when the user
// override is LOADED and a row's root is NOT a valid root of its environment,
// that row is SKIPPED (with a WARN) and loading CONTINUES: a VALID row loaded
// alongside it is still applied. Here the invalid row declares the FFEDIT root
// FIND in the FFCMD override file.
#[test]
fn load_skips_user_row_with_root_invalid_in_environment() {
    let mut shell = make_shell();
    let dir = point_user_aliases_at_temp(&mut shell);
    let path = override_file(&dir, "en", "FFCMD");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let rows = vec![
        // INVALID: FIND is not an FFCMD root -> skipped-with-WARN at load.
        override_row("BADROW", "FIND", AliasEnvironment::Ffcmd),
        // VALID: GSEARCH is an FFCMD root -> applied.
        override_row("GG", "GSEARCH", AliasEnvironment::Ffcmd),
    ];
    std::fs::write(&path, user_aliases::serialise_user_aliases(&rows).unwrap()).unwrap();
    shell.refresh_alias_store_in_place();
    use super::locale_resolver::LocaleResolveOutcome;
    assert_eq!(
        shell.resolve_locale_verb("BADROW", "FFCMD"),
        LocaleResolveOutcome::Unknown("BADROW".to_string()),
        "a row whose root is invalid in its environment is skipped at load"
    );
    assert_eq!(
        shell.resolve_locale_verb("GG", "FFCMD"),
        LocaleResolveOutcome::Root("GSEARCH".to_string()),
        "a valid row is still applied after an invalid row is skipped"
    );
}

// Validates: command-configurator Requirement 5.4 -- a Save that fails validation
// (built-in-root shadow) surfaces an error and leaves the on-disk override
// UNCHANGED.
#[test]
fn save_validation_failure_leaves_file_unchanged() {
    let mut shell = make_shell();
    let dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    shell.alias_editor_panel.rows = vec![row("GG", "GSEARCH", AliasEnvironment::Ffcmd, "")];
    save(&mut shell);
    let path = override_file(&dir, "en", "FFCMD");
    let before = std::fs::read_to_string(&path).expect("file written");
    shell.alias_editor_panel.rows = vec![row("CONFIG", "EDIT", AliasEnvironment::Ffcmd, "")];
    save(&mut shell);
    assert!(
        shell.alias_editor_panel.error.is_some(),
        "an invalid save surfaces an error"
    );
    let after = std::fs::read_to_string(&path).expect("file still present");
    assert_eq!(
        before, after,
        "a failed save leaves the on-disk override unchanged"
    );
}

// Validates: localization Requirement 13.7; command-configurator Requirement 6.4
// -- after a SAVE, the MENU render surface for the edited root reflects the new
// surface: editing an FFCMD menu-root alias (KAT -> CATALOGS) and saving makes
// the live store's display_surface_for_canonical("CATALOGS") return "KAT", which
// is exactly the active-locale surface the menu command column reads
// (menu_option_command_surfaces -> display_surface_for_canonical). No restart.
#[test]
fn save_makes_menu_render_surface_reflect_the_edited_alias() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    // Edit: an FFCMD alias KAT -> CATALOGS (CATALOGS is a valid FFCMD menu root).
    shell.alias_editor_panel.rows = vec![row("KAT", "CATALOGS", AliasEnvironment::Ffcmd, "")];
    save(&mut shell);
    assert!(shell.alias_editor_panel.error.is_none());
    // The live store's per-root display surface (what the menu column renders) is
    // now the edited surface KAT (Req 13.7 / 6.4).
    let store = shell.i18n.alias_store.lock().expect("alias store lock");
    let table = store.table_for("FFCMD").expect("FFCMD table");
    assert_eq!(
        table.display_surface_for_canonical("CATALOGS"),
        Some("KAT".to_string()),
        "after save the menu render surface for CATALOGS reflects the edited alias (Req 13.7)"
    );
}

// Validates: localization Requirement 13.6; command-configurator Requirement 6.3
// -- a SAVE writes ONLY the user per-locale override under the shell's override
// base; it never writes into the shipped read-only seed tree. The shipped
// `i18n/<locale>/aliases/` root is resolved separately and is not under the
// TempDir override base, so a save cannot touch it.
#[test]
fn save_writes_only_user_override_not_shipped_seed() {
    let mut shell = make_shell();
    let dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    shell.alias_editor_panel.rows = vec![row("GG", "GSEARCH", AliasEnvironment::Ffcmd, "")];
    save(&mut shell);
    assert!(shell.alias_editor_panel.error.is_none());
    // The write landed under the user override base...
    assert!(
        override_file(&dir, "en", "FFCMD").exists(),
        "the save wrote the user override file"
    );
    // ...and the shipped i18n root (if any) is a DIFFERENT path, never under the
    // TempDir override base, so the seed cannot have been written by this save.
    if let Some(shipped) = super::i18n_bootstrap::resolve_i18n_root() {
        assert!(
            !shipped.starts_with(dir.path()),
            "the shipped seed root must not live under the user override base"
        );
    }
}

// Validates: command-configurator Requirement 6.7 -- a parameterised alias
// authored in the editor expands/dispatches per CR-NR-112: ECOB -> EDIT with
// template FFWB.COBOL.MASTER(&1), saved, then `ecob helloW` rewrites to
// `EDIT FFWB.COBOL.MASTER(helloW)` through the existing pre-pass.
#[test]
fn parameterised_alias_authored_in_editor_expands_end_to_end() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    let env = shell.active_environment_name();
    let environment = if env == "FFEDIT" {
        AliasEnvironment::Ffedit
    } else {
        AliasEnvironment::Ffcmd
    };
    shell.alias_editor_panel.rows = vec![row("ECOB", "EDIT", environment, "FFWB.COBOL.MASTER(&1)")];
    save(&mut shell);
    assert!(shell.alias_editor_panel.error.is_none());
    assert_eq!(
        shell.rewrite_verb_to_root("ecob helloW"),
        "EDIT FFWB.COBOL.MASTER(helloW)",
        "the authored parameterised alias expands per CR-NR-112 (Req 6.7)"
    );
}
