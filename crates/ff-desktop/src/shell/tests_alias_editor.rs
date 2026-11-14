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
    // CR-CH-116: construction seeds the live alias store from the SHARED isolated
    // base (which the first-run write populated with the X -> EXIT seed). After
    // repointing the override base to this empty temp dir, refresh the live store
    // so it reflects the repointed (empty) base -- otherwise the editor would
    // still show the shared base's X -> EXIT seed row. Production keeps the store
    // and the base consistent; this mirrors that for the test's fresh base.
    shell.refresh_alias_store_in_place();
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
        description: String::new(),
    }
}

/// Run a `Command ===>` SAVE (the redesign's save path): apply pending line
/// commands then validate + write (Req 5.16, 5.22). CR-CH-077 single-chrome: the
/// shell's command field routes the typed verb into the Alias
/// command-environment handler, so the test drives that handler directly.
fn save(shell: &mut WorkbenchShell) {
    shell.apply_alias_command_line("SAVE");
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

// Validates: command-configurator Requirement 8.6 (CR-CH-116) -- the FFCMD
// X -> EXIT seed is an editable user-override row, visible in the Alias Editor.
// make_shell's first-run write seeds it into the shared isolated alias base, so
// opening the editor WITHOUT repointing the base shows the X -> EXIT row.
#[test]
fn x_to_exit_seed_row_is_visible_in_the_alias_editor() {
    let mut shell = make_shell();
    shell.dispatch_command_string("ALIASES");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::AliasEditor,
        "ALIASES opens the Alias Editor Context"
    );
    assert!(
        shell.alias_editor_panel.rows.iter().any(|r| {
            r.surface.eq_ignore_ascii_case("X")
                && r.canonical.eq_ignore_ascii_case("EXIT")
                && r.environment == AliasEnvironment::Ffcmd
        }),
        "the FFCMD X -> EXIT seed row must be visible/editable in the Alias Editor (Req 8.6): {:?}",
        shell.alias_editor_panel.rows
    );
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

// Validates: command-configurator Requirement 7.3 -- a non-empty description
// saved on an alias row persists to the override file and reloads through the
// effective-row load path (a round-trip).
#[test]
fn description_persists_and_reloads_through_effective_rows() {
    let mut shell = make_shell();
    let dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    let mut r = row("GG", "GSEARCH", AliasEnvironment::Ffcmd, "");
    r.description = "Global search shorthand".to_string();
    shell.alias_editor_panel.rows = vec![r];
    save(&mut shell);
    assert!(
        shell.alias_editor_panel.error.is_none(),
        "a valid save has no error: {:?}",
        shell.alias_editor_panel.error
    );
    // The override file carries the description.
    let path = override_file(&dir, "en", "FFCMD");
    let text = std::fs::read_to_string(&path).expect("file written");
    assert!(
        text.contains("description = \"Global search shorthand\""),
        "the saved override carries the description: {text}"
    );
    // Reloading the effective rows (seed + override) restores the description.
    shell.alias_editor_reload();
    let reloaded = shell
        .alias_editor_panel
        .rows
        .iter()
        .find(|row| row.surface.eq_ignore_ascii_case("GG"))
        .expect("the GG row reloads");
    assert_eq!(
        reloaded.description, "Global search shorthand",
        "the description round-trips through the effective-row reload"
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

// Validates: command-resolution-model Requirement 5.4 (SUPERSEDES localization
// Req 13.7 for the USER-shorthand case) -- after a SAVE, a USER shorthand does
// NOT drive the MENU command surface: editing an FFCMD menu-root alias
// (KAT -> CATALOGS) and saving leaves the live store's
// display_surface_for_canonical("CATALOGS") as the CANONICAL "CATALOGS" (the menu
// shows the canonical root, never a user shorthand, Req 5.4). The saved alias is
// still typeable (canonical_verb resolves KAT) and still listed in the Alias
// Editor; only the MENU display column ignores it. (A LOCALE translation would
// still drive the menu surface; see
// environment::locale_seed_drives_menu_display_surface.)
#[test]
fn save_user_alias_does_not_change_menu_render_surface() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    // Edit: an FFCMD alias KAT -> CATALOGS (CATALOGS is a valid FFCMD menu root).
    shell.alias_editor_panel.rows = vec![row("KAT", "CATALOGS", AliasEnvironment::Ffcmd, "")];
    save(&mut shell);
    assert!(shell.alias_editor_panel.error.is_none());
    let store = shell.i18n.alias_store.lock().expect("alias store lock");
    let table = store.table_for("FFCMD").expect("FFCMD table");
    // CR-NR-121 Req 5.4: the menu command surface for CATALOGS stays the
    // CANONICAL root, NOT the user shorthand KAT.
    assert_eq!(
        table.display_surface_for_canonical("CATALOGS"),
        Some("CATALOGS".to_string()),
        "a user shorthand must not drive the menu command surface (Req 5.4)"
    );
    // The saved alias is still typeable (resolution is unchanged by Stage B).
    assert_eq!(
        table.canonical_verb("KAT"),
        Some("CATALOGS"),
        "the saved user alias is still typeable and resolves to its root"
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

// Validates: command-configurator Requirement 7.5a -- every SAVE writes the
// per-locale override file in the canonical sorted order (uppercased surface
// ascending, canonical tie-break), regardless of the working-copy row order.
#[test]
fn save_writes_rows_in_sorted_surface_order() {
    let mut shell = make_shell();
    let dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    // Intentionally UNSORTED working rows (all valid FFCMD roots).
    shell.alias_editor_panel.rows = vec![
        row("ZED", "VIEW", AliasEnvironment::Ffcmd, ""),
        row("abc", "EDIT", AliasEnvironment::Ffcmd, ""),
        row("MID", "BROWSE", AliasEnvironment::Ffcmd, ""),
    ];
    save(&mut shell);
    assert!(
        shell.alias_editor_panel.error.is_none(),
        "a valid save has no error: {:?}",
        shell.alias_editor_panel.error
    );
    // Read the file back and confirm the surfaces appear in uppercased order.
    let path = override_file(&dir, "en", "FFCMD");
    let text = std::fs::read_to_string(&path).expect("file written");
    let rows = user_aliases::parse_user_alias_toml(&text).expect("parses");
    let surfaces: Vec<String> = rows.iter().map(|r| r.surface.clone()).collect();
    assert_eq!(
        surfaces,
        vec!["ABC".to_string(), "MID".to_string(), "ZED".to_string()],
        "the save writes rows in uppercased-surface ascending order (Req 7.5a)"
    );
}

// Validates: command-configurator Requirement 7.5 / 7.5a -- a manual `SORT`
// command reorders the WORKING rows into the SAME canonical order the
// auto-sort-on-save produces (both via alias_sort_key), so the two orderings are
// identical.
#[test]
fn manual_sort_matches_auto_sort_on_save_order() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    shell.alias_editor_panel.rows = vec![
        row("ZED", "VIEW", AliasEnvironment::Ffcmd, ""),
        row("abc", "EDIT", AliasEnvironment::Ffcmd, ""),
        row("MID", "BROWSE", AliasEnvironment::Ffcmd, ""),
    ];
    // A bare SORT orders by the first sortable column (Alias = surface).
    // CR-CH-077 single-chrome: routed through the Alias command-environment handler.
    shell.apply_alias_command_line("SORT");
    let surfaces: Vec<String> = shell
        .alias_editor_panel
        .rows
        .iter()
        .map(|r| r.surface.clone())
        .collect();
    // The working rows are reordered by uppercased surface ascending -- the same
    // key the save uses. (Surfaces are not uppercased in-place by a sort; the
    // KEY is uppercased, so "abc" sorts as "ABC" but keeps its stored form.)
    assert_eq!(
        surfaces,
        vec!["abc".to_string(), "MID".to_string(), "ZED".to_string()],
        "manual SORT reorders working rows by the canonical key (Req 7.5)"
    );
    // No field value changed -- only the order.
    assert_eq!(shell.alias_editor_panel.rows.len(), 3);
}

// Validates: command-configurator Requirement 7.4 -- a C + A copy applied
// end-to-end through apply_alias_editor_action duplicates the source row after
// the target (the shared line-command pass), with the copy's Act cleared.
#[test]
fn copy_after_applies_end_to_end_through_action() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    shell.dispatch_command_string("ALIASES");
    let mut src = row("GG", "GSEARCH", AliasEnvironment::Ffcmd, "");
    src.act = "C".to_string();
    let mut tgt = row("TT", "EDIT", AliasEnvironment::Ffcmd, "");
    tgt.act = "A".to_string();
    shell.alias_editor_panel.rows = vec![src, tgt];
    // A bare Enter on the command line applies the pending line commands.
    shell.apply_alias_editor_action(AliasEditorAction::ApplyLineCommands);
    // [GG(C), TT(A)] -> copy GG after TT: [GG, TT, GG-copy].
    assert_eq!(shell.alias_editor_panel.rows.len(), 3);
    assert_eq!(shell.alias_editor_panel.rows[2].surface, "GG");
    assert_eq!(shell.alias_editor_panel.rows[2].canonical, "GSEARCH");
    assert!(
        shell.alias_editor_panel.rows[2].act.is_empty(),
        "the copied row's Act cell is cleared"
    );
}

// Validates: command-configurator Requirement 9.1, 9.2, 9.5; context-help
// Requirement 22.2 -- an empty-description alias row (the X -> EXIT seed)
// DISPLAYS the inherited one-line description of its canonical root EXIT, drawn
// from the cmd:EXIT help topic through short_description, WITHOUT writing that
// value into the persisted row (the stored Description stays empty).
#[test]
fn empty_description_row_inherits_root_one_liner_display_only() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    // Seed a cmd:EXIT help topic with a Description first paragraph.
    shell.seed_help_registry_for_test(vec![ff_help::HelpTopic::new(
        ff_help::TopicKey::command("EXIT"),
        "EXIT Command".to_string(),
        "## Description\n\nExit the application.\n\n## See Also\n".to_string(),
        ff_help::TopicSource::FileBased {
            file_path: std::path::PathBuf::from("exit.help.md"),
        },
    )]);
    // An X -> EXIT row with an EMPTY stored description (the seed shape).
    shell.dispatch_command_string("ALIASES");
    shell.alias_editor_panel.rows = vec![row("X", "EXIT", AliasEnvironment::Ffcmd, "")];
    // Recompute the inherited map now that the help topic is seeded.
    shell.refresh_alias_inherited_descriptions();

    // The inherited map carries the EXIT one-liner, keyed uppercase.
    assert_eq!(
        shell
            .alias_editor_panel
            .inherited_descriptions
            .get("EXIT")
            .map(String::as_str),
        Some("Exit the application."),
        "the inherited map carries the cmd:EXIT one-liner for display"
    );
    // CRITICAL (Req 9.2): the inheritance is DISPLAY-time only -- the stored row
    // Description stays EMPTY, so SAVE / Reset-bare are unchanged.
    assert_eq!(
        shell.alias_editor_panel.rows[0].description, "",
        "the inherited description is never written into the persisted row"
    );
}

// Validates: command-configurator Requirement 9.4 -- a row whose canonical root
// has no cmd:<ROOT> help topic contributes no inherited description (the map
// omits it); the display falls back to empty, never a panic.
#[test]
fn row_with_no_root_topic_contributes_no_inherited_description() {
    let mut shell = make_shell();
    let _dir = point_user_aliases_at_temp(&mut shell);
    // Seed an UNRELATED topic so the registry is non-empty but lacks cmd:GSEARCH.
    shell.seed_help_registry_for_test(vec![ff_help::HelpTopic::new(
        ff_help::TopicKey::command("EXIT"),
        "EXIT Command".to_string(),
        "## Description\n\nExit the application.\n\n## See Also\n".to_string(),
        ff_help::TopicSource::FileBased {
            file_path: std::path::PathBuf::from("exit.help.md"),
        },
    )]);
    shell.dispatch_command_string("ALIASES");
    shell.alias_editor_panel.rows = vec![row("GG", "GSEARCH", AliasEnvironment::Ffcmd, "")];
    shell.refresh_alias_inherited_descriptions();
    assert!(
        !shell
            .alias_editor_panel
            .inherited_descriptions
            .contains_key("GSEARCH"),
        "a root with no cmd:<ROOT> topic contributes no inherited description (Req 9.4)"
    );
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
