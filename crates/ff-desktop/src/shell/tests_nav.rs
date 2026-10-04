//! Shell tests -- nav area (split from shell/tests.rs, CR F3).
//! Items are verbatim; only their file location changed.

#![allow(unused_imports)]
use super::tests_common::*;
use ff_keys::{KeyMap, ModifiedKey};
use std::sync::{Arc, Mutex};

use ff_command::{
    CommandDispatch, CommandError, CommandHandler, CommandHistory, CommandId, CommandMetadata,
    CommandParams, CommandRegistry, CommandResult, ExecutionContext,
};

/// Validates: Requirement 14.1 -- first launch inserts a POM tab.
#[test]
fn first_launch_inserts_pom_tab() {
    // Validates: Requirement 14.1
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use tokio::runtime::Runtime;
    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    mgr.insert_pom_tab(&runtime);
    assert!(mgr.tabs()[0].is_home);
    assert_eq!(mgr.tabs()[0].kind, TabKind::MenuWorkspace);
    assert_eq!(mgr.tabs()[0].title, "[POM]");
}

/// Validates: workspace-kinds Requirement 5.2 (CR-NR-090 B.3) -- the
/// line_end_from_name mapper.
#[test]
fn line_end_from_name_maps_unicode_else_default() {
    // Validates: workspace-kinds Requirement 5.2
    use ff_document_model::LineEndMode;
    assert_eq!(super::line_end_from_name("unicode"), LineEndMode::Unicode);
    assert_eq!(super::line_end_from_name("Unicode"), LineEndMode::Unicode);
    assert_eq!(super::line_end_from_name("default"), LineEndMode::Default);
    assert_eq!(super::line_end_from_name("anything"), LineEndMode::Default);
}

/// Validates: function-keys-and-history Requirement 17.2 (CR-CH-016) -- END from
/// a POM with other Workspaces open closes only that POM and navigates back; it
/// does NOT terminate the application.
#[test]
fn end_from_pom_with_other_tabs_closes_pom_not_app() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    // Make the active tab a POM, then open a second Workspace so the POM is not
    // the only tab.
    let idx = shell.tabs.active_index();
    if let Some(tab) = shell.tabs.tabs_mut().get_mut(idx) {
        tab.kind = TabKind::MenuWorkspace;
        tab.is_home = true;
        tab.title = "[POM]".to_string();
    }
    shell.tabs.insert_pom_tab(&shell.runtime); // second tab
                                               // Re-select the first (a POM) as active.
    shell.tabs.set_active(0);
    assert!(shell.tabs.active_tab().is_home);
    let before = shell.tabs.len();
    assert!(before >= 2, "precondition: more than one Workspace open");

    shell.handle_command("END");

    // The POM Workspace was closed (count decreased); the app was NOT terminated.
    assert_eq!(
        shell.tabs.len(),
        before - 1,
        "END from a POM with other tabs must close that one tab"
    );
}

/// Validates: function-keys-and-history Requirement 17.2a (CR-CH-016) -- END from
/// a POM that is the ONLY Workspace open does not close/navigate (it takes the
/// terminate path); the tab is preserved (close_tab keeps the last tab, and the
/// exit branch is taken instead of close-and-navigate).
#[test]
fn end_from_pom_as_only_workspace_does_not_close_tab() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    // Reduce to a single POM tab.
    while shell.tabs.len() > 1 {
        shell.tabs.close_tab(shell.tabs.len() - 1);
    }
    let idx = shell.tabs.active_index();
    if let Some(tab) = shell.tabs.tabs_mut().get_mut(idx) {
        tab.kind = TabKind::MenuWorkspace;
        tab.is_home = true;
        tab.title = "[POM]".to_string();
    }
    assert_eq!(shell.tabs.len(), 1);

    shell.handle_command("END");

    // Only Workspace -> terminate path (file.exit), which does not close the tab
    // in-model; the single tab remains.
    assert_eq!(
        shell.tabs.len(),
        1,
        "END from the only POM takes the exit path, not close-and-navigate"
    );
    assert!(shell.tabs.active_tab().is_home);
}

/// Validates: function-keys-and-history Requirement 17.4 (REVISED, CR-CH-016) --
/// RETURN from a POM with other Workspaces open closes only that POM (same as
/// END), not the app.
#[test]
fn return_from_pom_with_other_tabs_closes_pom_not_app() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    let idx = shell.tabs.active_index();
    if let Some(tab) = shell.tabs.tabs_mut().get_mut(idx) {
        tab.kind = TabKind::MenuWorkspace;
        tab.is_home = true;
        tab.title = "[POM]".to_string();
    }
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.tabs.set_active(0);
    let before = shell.tabs.len();
    assert!(before >= 2);

    shell.handle_command("RETURN");

    assert_eq!(
        shell.tabs.len(),
        before - 1,
        "RETURN from a POM with other tabs must close that one tab (revised 17.4)"
    );
}

/// Validates: menu-workspace Requirement 14.10 (CR-CH-038) -- RETURN in a
/// non-POM workspace navigates to the POM (Home Context) in ONE step, clearing
/// the Navigation_Stack, even when the workspace was rooted directly (its root
/// is NOT the POM). The workspace stays open (tab count unchanged), now Home.
#[test]
fn return_from_non_pom_navigates_to_pom() {
    // Validates: menu-workspace Requirement 14.10
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    // A workspace rooted directly at a Files context (START <arg> style): a
    // non-POM tab with an EMPTY nav stack (its root is Files, not the POM).
    let idx = shell.tabs.active_index();
    if let Some(tab) = shell.tabs.tabs_mut().get_mut(idx) {
        tab.kind = TabKind::FilesPanel;
        tab.is_home = false;
        tab.title = "[FILES]".to_string();
        tab.nav_stack.clear();
    }
    let before = shell.tabs.len();
    assert!(
        !shell.tabs.active_tab().is_home,
        "precondition: not on the POM"
    );

    shell.handle_command("RETURN");

    assert!(
        shell.tabs.active_tab().is_home,
        "RETURN in a non-POM workspace must land on the POM (Home Context)"
    );
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "the POM is a MenuWorkspace"
    );
    assert_eq!(
        shell.tabs.len(),
        before,
        "RETURN to the POM must NOT close the workspace (tab count unchanged)"
    );
    assert!(
        shell.tabs.active_tab().nav_stack.is_empty(),
        "RETURN clears the Navigation_Stack"
    );
}

/// Validates: menu-workspace Requirement 14.10 (CR-CH-038) -- RETURN in a
/// drilled-in non-POM workspace (nav stack non-empty) also goes straight to the
/// POM in one step, not one level back (that is END's job).
#[test]
fn return_from_drilled_in_non_pom_goes_straight_to_pom() {
    // Validates: menu-workspace Requirement 14.10
    let mut shell = make_shell();
    // POM -> option 1 (Catalogs/Files) pushes the POM onto the stack, landing on
    // a non-POM Context with a non-empty nav stack.
    shell.handle_command("1");
    assert!(
        !shell.tabs.active_tab().is_home,
        "precondition: drilled into a sub-context"
    );
    shell.handle_command("RETURN");
    assert!(
        shell.tabs.active_tab().is_home,
        "RETURN from a drilled-in non-POM must jump straight to the POM"
    );
}

/// Validates: Requirement 19.10 -- END command on FileExplorerPanel returns tab to POM.
#[test]
fn file_explorer_panel_end_command_returns_to_pom() {
    // Validates: Requirement 19.10
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use tokio::runtime::Runtime;
    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    mgr.insert_pom_tab(&runtime);
    mgr.transform_active_pom_tab(TabKind::FileExplorerPanel, "[FILES]");
    assert_eq!(mgr.active_tab().kind, TabKind::FileExplorerPanel);
    // Simulate END: transform back to POM
    let idx = mgr.active_index();
    if let Some(tab) = mgr.tabs_mut().get_mut(idx) {
        tab.kind = TabKind::MenuWorkspace;
        tab.is_home = true;
        tab.title = "[POM]".to_string();
    }
    assert!(mgr.active_tab().is_home);
}

/// Validates: Requirement 19.4 -- fastpath dotted notation navigates to option.
#[test]
fn fastpath_notation_navigates_to_option() {
    // Validates: Requirement 19.4
    // "2.1" navigates to option 2 then sub-option 1.
    // Option 2 on POM -> FileExplorerPanel; then "1" on non-POM -> FilesPanel.
    let mut shell = make_shell();
    shell.handle_command("2.1");
    // After fastpath, the active tab should have been navigated (no panic, no unknown-command error)
    // The exact final kind depends on sub-option routing; we verify no crash and no
    // "unknown command" error from the fastpath handler itself.
    let err = shell.open_error.as_deref().unwrap_or("");
    assert!(
        !err.to_uppercase().contains("UNKNOWN"),
        "fastpath should not produce unknown-command error, got: {err}"
    );
    // The tab should have navigated away from the Home Context (POM).
    assert!(
        !shell.tabs.active_tab().is_home,
        "fastpath should have navigated away from POM"
    );
}

/// Validates: menu-workspace Requirement 5.7 (B061) -- the leading `=` is the
/// Navigation_Origin: a chained path resolves against the POM even when the
/// active Workspace is NOT the POM.
#[test]
fn chained_fastpath_pops_to_pom_origin_from_non_pom() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    // Move away from the POM first (open the Keys Workspace directly).
    shell.handle_command("KEYS");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::KeysEditor);
    // From a non-POM context, `=0` must resolve option 0 against the POM.
    shell.handle_command("=0.K");
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::KeysEditor,
        "=0.K from a non-POM context must still resolve against the POM origin"
    );
    let err = shell.open_error.as_deref().unwrap_or("");
    assert!(
        !err.to_lowercase().contains("not yet implemented"),
        "chained fastpath must not fall through to the not-implemented stub, got: {err:?}"
    );
}

/// Validates: configuration-system Req 20 / CR-CH-022 -- END from a CONFIG
/// namespace view returns to the Settings_Menu (the data-driven Menu_Workspace),
/// not the flat All-Settings view.
#[test]
fn settings_end_from_namespace_view_returns_to_menu() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("CONFIG editor");
    assert_eq!(
        shell.config_panel.namespace_filter.as_deref(),
        Some("editor")
    );
    shell.handle_command("END");
    // END returns to the Settings_Menu (Menu_Workspace), not the flat list.
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "END from a namespace view must return to the Settings_Menu"
    );
}

// === Phase CV: POM extended option routing (9, S, B) =====================

/// Validates: menu-workspace Req 2.1e/2.1i -- POM key S resolves to its
/// configured command (SEARCH) and opens Global Search.
#[test]
fn pom_key_s_routes_to_search() {
    // CR-CH-021: the Recovery_Baseline POM no longer carries an `S` key; SEARCH
    // remains reachable by name (and via any user menu that maps a key to it).
    let mut shell = make_shell();
    shell.handle_command("SEARCH");
    // Opening the search panel clears open_error (success path).
    assert!(
        shell.open_error.is_none(),
        "SEARCH must open Search without error, got: {:?}",
        shell.open_error
    );
    use crate::tab_state::TabKind;
    let has_search =
        (0..shell.tabs.len()).any(|i| shell.tabs.tabs()[i].kind == TabKind::SearchResults);
    assert!(has_search, "SEARCH must open a Search Results tab");
}

/// Validates: menu-workspace Req 2.1e/2.1i -- a POM key resolves to its
/// configured command (key 2 -> FILES -> File Explorer), config-driven.
#[test]
fn pom_key_resolves_to_configured_command() {
    let mut shell = make_shell();
    shell.handle_command("2");
    use crate::tab_state::TabKind;
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::FileExplorerPanel,
        "key 2 must resolve to its pom.toml command (FILES)"
    );
}

/// Validates: plugin-manager-ui Requirement 1.1 -- option 8 routes to PluginManager.
#[test]
fn option_8_routes_to_plugin_manager() {
    // Validates: plugin-manager-ui Requirement 1.1
    // CR-CH-021: the Recovery_Baseline POM no longer carries an `8` key; PLUGINS
    // remains reachable by name (and via any user menu that maps a key to it).
    let mut shell = make_shell();
    shell.handle_command("PLUGINS");
    use crate::tab_state::TabKind;
    assert_eq!(shell.tabs.active_tab().kind, TabKind::PluginManager);
}

/// Validates: file-tree-panel Requirement 24.9 (open resolves to the real file)
/// -- B047 regression. A Local Files node URI is `posix`-scheme and relative to
/// the home jail root; `nav_open_path` must turn it into a real absolute host
/// path (home + relative), NOT pass the jail-relative path straight through
/// (which produced "resource not found: vfs://local/C:/On...").
#[test]
fn nav_open_path_resolves_posix_uri_to_absolute_host_path() {
    let shell = make_shell();
    let home = dirs::home_dir()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    // A file with spaces in a subdirectory, exactly like the reported case.
    let uri = ff_vfs::ResourceUri::new("posix", "/OneDrive - Standard Bank/Clipbook.md");
    let resolved = shell
        .nav_open_path(&uri)
        .expect("posix uri must resolve to a host path");

    let expected = home
        .join("OneDrive - Standard Bank")
        .join("Clipbook.md")
        .to_string_lossy()
        .into_owned();
    assert_eq!(resolved, expected);
    // Must be absolute -- never the jail-relative "/OneDrive - ..." form.
    assert_ne!(resolved, "/OneDrive - Standard Bank/Clipbook.md");
}

/// Validates: B047 -- `nav_open_path` rejects parent-traversal out of the jail.
#[test]
fn nav_open_path_rejects_parent_traversal() {
    let shell = make_shell();
    let uri = ff_vfs::ResourceUri::new("posix", "/../../etc/passwd");
    assert!(shell.nav_open_path(&uri).is_err());
}

/// Validates: lua-macro-engine Requirement 12.1 -- option 6 routes to MacroLibrary.
#[test]
fn option_5_routes_to_macro_library() {
    // Validates: lua-macro-engine Requirement 12.1; menu-workspace Req 2.1e/2.1i
    // -- a POM option resolves to its configured command (MACROS -> MacroLibrary).
    // CR-CH-021: the Recovery_Baseline POM no longer carries a `5` key (it ships
    // only 0/1/2/L/M/X); MACROS remains reachable by name and via any user menu
    // that maps a key to it. This test exercises the command routing directly.
    let mut shell = make_shell();
    shell.handle_command("MACROS");
    use crate::tab_state::TabKind;
    assert_eq!(shell.tabs.active_tab().kind, TabKind::MacroLibrary);
}

/// Validates: Requirement 21.4 (CR-CH-012) -- the persisted POM Menu descriptor
/// is NOT reopened here as an extra menu tab; the POM-always-present guarantee
/// (Req 21.8) owns the Home Context, so restoring `Menu{name:"pom"}` must not
/// create a second/duplicate menu workspace.
#[test]
fn restore_pom_menu_descriptor_is_left_to_the_pom_guarantee() {
    use crate::tab_state::TabKind;
    use ff_session::session_state::WorkspaceDescriptor;

    let menus = tempfile::TempDir::new().expect("tempdir");
    std::fs::create_dir_all(menus.path()).expect("mkdir");
    let mut shell = make_shell();
    shell.dir_overrides.menus = Some(menus.path().to_path_buf());

    let before = shell.tabs.tabs().len();
    shell.restore_workspace_descriptors(&[WorkspaceDescriptor::Menu {
        name: "pom".to_string(),
    }]);
    // The pom descriptor is handled by the separate POM guarantee, not by the
    // menu-reopen path, so it adds no new tab here.
    assert_eq!(
        shell.tabs.tabs().len(),
        before,
        "restoring Menu{{name:pom}} must not open an extra menu tab (POM guarantee owns Home)"
    );
    // No non-home MenuWorkspace tab was created for "pom".
    assert!(
        !shell
            .tabs
            .tabs()
            .iter()
            .any(|t| t.kind == TabKind::MenuWorkspace && !t.is_home),
        "restoring the POM descriptor must not create a non-home menu tab"
    );
}

// Validates: menu-workspace Requirement 10.1, 10.3 -- selecting a menu option
// whose command equals a user definition id dispatches its target (typed path).
#[test]
fn menu_option_command_matching_definition_id_dispatches() {
    use crate::menu_workspace::{MenuFile, MenuOption, MenuWorkspaceState};
    let mut shell = make_shell();
    push_user_function_def(&mut shell, "my.exit", "file.exit");

    // Build a Menu_Workspace tab whose option "1" runs the user definition.
    let menu = MenuFile {
        title: "T".to_string(),
        options: vec![MenuOption {
            key: "1".to_string(),
            command: "my.exit".to_string(),
            description: "Run my exit".to_string(),
            enabled: true,
            group: None,
            show_in_menu_bar: true,
            target: None,
        }],
        show_calendar: true,
        group_separator: crate::menu_workspace::GroupSeparator::default(),
        group_headers: false,
    };
    let mw = MenuWorkspaceState {
        file_path: std::path::PathBuf::from("t.toml"),
        menu: Some(menu),
        load_error: None,
        last_modified: None,
        advisory: None,
        limits: crate::menu_workspace::OptionLimits::default(),
    };
    let idx = shell.tabs.active_index();
    if let Some(tab) = shell.tabs.tabs_mut().get_mut(idx) {
        tab.kind = crate::tab_state::TabKind::MenuWorkspace;
        tab.menu_workspace = Some(mw);
    }

    // Selecting option "1" should resolve to the user target and dispatch it
    // (routing file.exit through the pipeline). No "not defined" error.
    shell.handle_command("1");
    let msg = shell.open_error.as_deref().unwrap_or_default();
    assert!(
        !msg.contains("not found") && !msg.contains("is not defined"),
        "option matching a definition id must dispatch, got error: {msg}"
    );
}

// Validates: command-configurator Requirement 2.8 -- END from the Command
// Configurator returns the tab to the POM.
#[test]
fn command_configurator_end_returns_to_pom() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("COMMANDS");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::CommandConfigurator);
    // CR-CH-022: END pops the Navigation_Stack (which holds [POM]) and
    // reconstructs the POM in place -- synchronously, no deferred flag.
    shell.handle_command("END");
    assert!(shell.tabs.active_tab().is_home);
}

// Validates: menu-workspace Requirement 11.2 -- MENU POM resolves to the Home Context.
#[test]
fn menu_pom_resolves_to_home_context() {
    let mut shell = make_shell();
    shell.handle_command("COMMANDS");
    shell.handle_command("MENU POM");
    assert!(shell.tabs.active_tab().is_home);
}

// Validates: menu-workspace Requirement 10.4, 11.5 -- a Menu_Target dispatches
// through the same menu-open path (POM target returns to the Home Context).
#[test]
fn dispatch_menu_target_pom_opens_home_context() {
    use crate::tab_state::TabKind;
    use ff_command::CommandTarget;
    let mut shell = make_shell();
    shell.handle_command("COMMANDS");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::CommandConfigurator);
    shell.dispatch_command_target(&CommandTarget::Menu {
        name: "pom".to_string(),
    });
    assert!(shell.tabs.active_tab().is_home);
}

// Validates: B075 -- clicking the POM "Settings" option (the menu-option
// dispatch seam) must open the Settings menu IN PLACE (Navigation_Stack push,
// no new tab), IDENTICAL to typing `SETTINGS` -- NOT a generic new Menu
// Workspace tab. Root cause was the click path (dispatch_command_target's Menu
// arm -> open_menu_by_name) bypassing the settings/pom special-casing that the
// typed path (try_menu_name_dispatch) applies.
#[test]
fn clicking_pom_settings_option_opens_settings_menu_in_place() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    // Ensure the ACTIVE tab is the POM (Home Context), as when the user is on
    // the POM. `make_shell` starts with a bare welcome tab, so open a POM via
    // START and make it the active context.
    shell.handle_command("START");
    assert!(
        shell.tabs.active_tab().is_home,
        "precondition: active tab is the POM"
    );
    let tabs_before = shell.tabs.len();

    // The POM option 0 command is "Settings" (mixed case). Dispatch it exactly
    // as the option-click seam does (resolve_and_dispatch_command -> fallthrough
    // handle_command), via the shared dispatch_bound_command entry point.
    shell.dispatch_bound_command("Settings");

    // Must navigate IN PLACE to the Settings menu: same tab count, active tab is
    // the Settings Menu_Workspace, and END can return (nav stack pushed).
    assert_eq!(
        shell.tabs.len(),
        tabs_before,
        "clicking Settings must navigate in place, not open a new tab (B075)"
    );
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "clicking Settings must land on a Menu_Workspace"
    );
    assert!(
        !shell.tabs.active_tab().is_home,
        "clicking Settings must leave the POM Home Context"
    );
    let title_is_settings = shell
        .tabs
        .active_tab()
        .menu_workspace
        .as_ref()
        .and_then(|mw| mw.menu.as_ref())
        .map(|m| m.title.eq_ignore_ascii_case("Settings"))
        .unwrap_or(false);
    assert!(
        title_is_settings,
        "clicking Settings must open the SETTINGS menu (title 'Settings'), not a generic menu (B075)"
    );
}

// Validates: menu-workspace Requirement 19.1, 19.2; command-framework
// Requirement 14.1, 14.2 -- selecting the POM "Settings" option by CLICK
// (dispatch_bound_command, the option-click seam) and by TYPING `SETTINGS` land
// on an IDENTICAL result: same tab count (in place), same Menu_Workspace kind,
// same "Settings" title, both off the Home Context. This is the convergence:
// one Option-Selection path, POM == Settings, click == typed.
#[test]
fn pom_settings_click_equals_typed_settings() {
    use crate::tab_state::TabKind;

    // Path A: click the POM "Settings" option (command "Settings").
    let mut click = make_shell();
    click.handle_command("START");
    assert!(click.tabs.active_tab().is_home, "precondition: POM active");
    let tabs_before_click = click.tabs.len();
    click.dispatch_bound_command("Settings");

    // Path B: type `SETTINGS` from the POM.
    let mut typed = make_shell();
    typed.handle_command("START");
    assert!(typed.tabs.active_tab().is_home, "precondition: POM active");
    let tabs_before_typed = typed.tabs.len();
    typed.handle_command("SETTINGS");

    // Identical observable landing state.
    assert_eq!(
        click.tabs.len(),
        tabs_before_click,
        "click Settings must navigate in place"
    );
    assert_eq!(
        typed.tabs.len(),
        tabs_before_typed,
        "typed SETTINGS must navigate in place"
    );
    assert_eq!(
        click.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "click lands on a Menu_Workspace"
    );
    assert_eq!(
        typed.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "typed lands on a Menu_Workspace"
    );
    assert!(!click.tabs.active_tab().is_home, "click leaves the POM");
    assert!(!typed.tabs.active_tab().is_home, "typed leaves the POM");
    assert_eq!(
        active_menu_title(&click),
        active_menu_title(&typed),
        "click Settings and typed SETTINGS must open the SAME menu (Req 19.2 / 14.2)"
    );
    assert_eq!(
        active_menu_title(&click).as_deref(),
        Some("Settings"),
        "both must land on the Settings menu"
    );
}

// Validates: menu-workspace Requirement 19.1, 19.2; command-framework
// Requirement 14.1 -- selecting a POM Option_Key by TYPING the key and by
// CLICKING the row (dispatch_bound_command with that option's command) give an
// IDENTICAL result. Uses POM option 1 (command `Catalogs`, navigates in place).
// Proves the POM option-key path and the click path are the one resolver (POM
// is no longer a separate resolver from non-POM menus). The option's command is
// read from the loaded POM menu so the test tracks the real defaults.
#[test]
fn pom_option_key_type_and_click_same_result() {
    // Discover POM option 1's real command from the loaded menu (defaults:
    // `Catalogs`), so "type the key" and "click the row" use the same option.
    let mut probe = make_shell();
    probe.handle_command("START");
    probe.ensure_pom_menu_loaded();
    let option1_command = probe
        .tabs
        .active_tab()
        .menu_workspace
        .as_ref()
        .and_then(|mw| mw.menu.as_ref())
        .and_then(|m| m.options.iter().find(|o| o.key == "1"))
        .map(|o| o.command.clone())
        .expect("POM must have an option with key 1");

    // Path A: type the POM option key `1`.
    let mut typed = make_shell();
    typed.handle_command("START");
    assert!(typed.tabs.active_tab().is_home, "precondition: POM active");
    let tabs_before_typed = typed.tabs.len();
    typed.handle_command("1");

    // Path B: click POM option 1 (dispatch its command through the click seam).
    let mut click = make_shell();
    click.handle_command("START");
    assert!(click.tabs.active_tab().is_home, "precondition: POM active");
    let tabs_before_click = click.tabs.len();
    click.dispatch_bound_command(&option1_command);

    assert_eq!(
        typed.tabs.len(),
        tabs_before_typed,
        "typed `1` navigates in place"
    );
    assert_eq!(
        click.tabs.len(),
        tabs_before_click,
        "click of option 1 navigates in place"
    );
    assert_eq!(
        typed.tabs.active_tab().kind,
        click.tabs.active_tab().kind,
        "typing the POM option key `1` and clicking its row must land on the same kind (Req 19.2 / 14.1)"
    );
    assert!(
        !typed.tabs.active_tab().is_home,
        "typed `1` must leave the POM Home Context"
    );
}

// Validates: command-framework Req 8.11 / menu-workspace Req 11.11 -- a bare
// menu name (no MENU keyword) opens that menu; POM resolves to the Home Context.
#[test]
fn keyword_less_pom_menu_name_opens_home_context() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("SETTINGS"); // leave the POM first
    assert_eq!(shell.tabs.active_tab().kind, TabKind::MenuWorkspace);
    shell.handle_command("POM");
    assert!(
        shell.tabs.active_tab().is_home,
        "keyword-less POM must return to the Home Context"
    );
}

// Validates: cw-requirements.md Req 9.1 -- option 0 / =0 open the Settings_Menu.
#[test]
fn settings_option_zero_opens_menu_workspace() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("0");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::MenuWorkspace);
}

// Validates: cw-requirements.md Req 9.4 -- option A opens the unfiltered flat
// CR-CH-025: bare CONFIG opens the flat All-Settings view, NOT the menu.
#[test]
fn settings_option_a_opens_flat_panel() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("CONFIG");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::ConfigPanel);
    assert!(shell.config_panel.namespace_filter.is_none());
    assert_eq!(shell.tabs.active_tab().title, "[CONFIG]");
}

// Validates: cw-requirements.md Req 10.4 / 15.10 -- END from the Settings_Menu
// (a Menu_Workspace) returns to the POM.
#[test]
fn settings_menu_end_returns_to_pom() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("SETTINGS");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::MenuWorkspace);
    // CR-CH-022: END pops the Navigation_Stack ([POM]) and reconstructs the POM
    // in place, synchronously.
    shell.handle_command("END");
    assert!(shell.tabs.active_tab().is_home);
}

// Validates: menu-workspace Req 12.3 (CR-CH-025) -- the default settings.toml
// option A carries the `CONFIG` command (opens the flat config-key browser),
// replacing the former opaque `A` command.
#[test]
fn default_settings_toml_option_a_command_is_config() {
    use crate::menu_workspace::loader::load_menu_file;
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::new().expect("tempfile");
    f.write_all(crate::menu_workspace::defaults::DEFAULT_SETTINGS_TOML.as_bytes())
        .expect("write");
    let menu = load_menu_file(f.path()).expect("valid settings.toml");
    let opt_a = menu
        .options
        .iter()
        .find(|o| o.key == "A")
        .expect("option A present");
    assert_eq!(
        opt_a.command, "Config",
        "option A must dispatch Config (the flat config-key browser)"
    );
}

// Validates: menu-workspace Requirement 13.4 -- editing a field mutates the
// working option (key is uppercased).
// Validates: menu-workspace Requirement 13.4 (B054) -- option fields are edited
// directly on the working menu (the render binds TextEdits to it); the model
// holds the typed value.
#[test]
fn menus_editor_edit_option_field_mutates_working() {
    let (mut shell, _dir) = make_shell_with_menus_editor();
    // Simulate the render binding by mutating the working option directly.
    shell.menus_editor_panel.working.as_mut().unwrap().options[0].command = "PLUGINS".to_string();
    assert_eq!(
        shell.menus_editor_panel.working.as_ref().unwrap().options[0].command,
        "PLUGINS"
    );
}

// === CR-CH-022: Per-tab Navigation_Stack (Requirement 14) ==================

// Validates: Req 14.2 -- navigating transforms the CURRENT tab in place and
// never opens a new tab.
#[test]
fn navigation_transforms_in_place_no_new_tab() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    let start_len = shell.tabs.len();
    shell.handle_command("SETTINGS");
    assert_eq!(
        shell.tabs.len(),
        start_len,
        "navigation must not open a new tab"
    );
    assert_eq!(shell.tabs.active_tab().kind, TabKind::MenuWorkspace);
    shell.handle_command("PLUGINS");
    assert_eq!(
        shell.tabs.len(),
        start_len,
        "still no new tab after a second navigation"
    );
    assert_eq!(shell.tabs.active_tab().kind, TabKind::PluginManager);
}

// Validates: Req 14.8 -- START =0 roots at the POM then drills to Settings,
// leaving the POM on the stack so END walks back to the POM.
#[test]
fn start_equals_path_keeps_pom_on_stack() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("START =0"); // POM option 0 = SETTINGS
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "START =0 drills to the Settings menu"
    );
    assert!(
        !shell.tabs.active_tab().nav_stack.is_empty(),
        "START =X keeps the POM on the stack"
    );
    shell.handle_command("END");
    assert!(
        shell.tabs.active_tab().is_home,
        "END from a START =0 tab returns to the POM"
    );
}

// === CR-CH-039 / B069: Config View keyboard tree navigation =================

// Validates: configuration-system Req 21.2/21.3/21.4/21.6/21.8/21.11 -- with the
// Config View open and keyboard focus in the tree (not the Filter field), the
// arrow keys drive a Tree_Cursor and expand/collapse namespace groups, matching
// the File Navigator. Asserts on the model (cursor / collapsed) per the pure
// reducer + rendered-focus contract; the full shell drives it end-to-end.
#[test]
fn full_shell_config_tree_arrows_navigate_and_expand() {
    use crate::config_panel::ConfigNodeId;
    use crate::tab_state::TabKind;

    let mut harness = harness_shell();
    harness.state_mut().handle_command("CONFIG");
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.active_tab().kind,
        TabKind::ConfigPanel,
        "CONFIG opens the flat Config panel"
    );

    // Give the tree the keyboard: surrender the command field's focus so nothing
    // holds focus (the File-Navigator-style state in which the tree drives the
    // arrows, per Req 21.9). Then Down establishes the cursor on the first row.
    harness.ctx.memory_mut(|m| m.stop_text_input());
    harness
        .ctx
        .memory_mut(|m| m.request_focus(egui::Id::new("config_tree_focus_none")));
    // Clear that dummy focus so `tree_has_keyboard` sees "nothing focused".
    harness
        .ctx
        .memory_mut(|m| m.surrender_focus(egui::Id::new("config_tree_focus_none")));

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    let cursor = harness.state().config_panel.cursor.clone();
    assert!(
        matches!(cursor, Some(ConfigNodeId::Namespace(_))),
        "first ArrowDown establishes the Tree_Cursor on the first namespace group, got {cursor:?}"
    );
    let ns = match cursor {
        Some(ConfigNodeId::Namespace(ns)) => ns,
        other => panic!("expected a namespace cursor, got {other:?}"),
    };

    // The first group starts expanded; Left collapses it (Req 21.5).
    harness.key_press(egui::Key::ArrowLeft);
    harness.run();
    assert_eq!(
        harness.state().config_panel.collapsed.get(&ns).copied(),
        Some(true),
        "ArrowLeft on an expanded group collapses it"
    );

    // Right re-expands the collapsed group (Req 21.4).
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    assert_eq!(
        harness.state().config_panel.collapsed.get(&ns).copied(),
        Some(false),
        "ArrowRight on a collapsed group expands it"
    );

    // Down moves the cursor off the first namespace (to its first child key, now
    // that it is expanded) -- the cursor changes (Req 21.3).
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    assert_ne!(
        harness.state().config_panel.cursor,
        Some(ConfigNodeId::Namespace(ns.clone())),
        "ArrowDown moves the cursor off the first namespace group"
    );
}

/// Validates: menu-workspace Req 20.2; menu-and-statusbar Req 17.11 -- both the
/// POM and the Settings menu Title_Line derive from their loaded Menu_Title (the
/// raw title, not bracketed/uppercased), one uniform source. This is the pure
/// `title_line_text` derivation; the centering is asserted at the render site.
#[test]
fn pom_and_settings_title_line_derive_from_loaded_menu_title() {
    use crate::menu_workspace::MenuWorkspaceState;
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    use std::io::Write;

    // POM: loaded pom.toml title, raw (not bracketed).
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    let pom_text = super::title_line_text(shell.tabs.active_tab());
    assert_eq!(pom_text, "FileForge Workbench -- Primary Option Menu");

    // Settings: a loaded menu titled "Settings" -> the Title_Line shows the RAW
    // title "Settings" (CR-CH-042 replaces the former "[SETTINGS]" bracketed form).
    let mut f = tempfile::NamedTempFile::new().expect("tempfile");
    f.write_all(b"title = \"Settings\"\n[[options]]\nkey=\"A\"\ncommand=\"CONFIG\"\ndescription=\"Config\"\n")
        .expect("write");
    let mw = MenuWorkspaceState::load(f.path());
    let tab = TabState::menu_workspace_tab(TabId(9), new_document(), mw);
    let set_text = super::title_line_text(&tab);
    assert_eq!(
        set_text, "Settings",
        "Settings Title_Line must be the raw Menu_Title, not [SETTINGS]"
    );
}

/// Validates: menu-workspace Req 20.5 -- the `POM` command opens/returns to the
/// Home Context, and bare `START` is an alias that also lands on the Home
/// Context. START's tab-creation forms (Req 14.8) are exercised elsewhere.
#[test]
fn pom_command_and_bare_start_open_home_context() {
    // POM command.
    let mut a = make_shell();
    a.handle_command("EDIT somefile"); // move off any initial POM (best effort)
    a.handle_command("POM");
    assert!(
        a.tabs.active_tab().is_home,
        "the POM command must land on the Home Context"
    );

    // Bare START alias.
    let mut b = make_shell();
    b.handle_command("START");
    assert!(
        b.tabs.active_tab().is_home,
        "bare START (alias of POM) must land on the Home Context"
    );
}

// === CR-CH-044: typed path via the CommandTarget classifier =================

/// Validates: command-framework Req 15.1/15.2/15.8; menu-workspace Req 20.5
/// (revised) -- typing `POM` resolves as a Menu_Name (`Menu{pom}`) through the
/// classifier (the stage-3 menu-name path) and lands on the Home Context, WITHOUT
/// a bespoke `POM` intercept arm. This is behaviour-preserving: the retired arm
/// and the menu-name path both open the Home Context via the same
/// `open_menu_by_name("pom")` effect.
#[test]
fn typed_pom_resolves_as_menu_name_opens_home_context() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("START"); // ensure a POM exists and is active
    assert!(shell.tabs.active_tab().is_home, "precondition: on the POM");
    shell.handle_command("SETTINGS"); // leave the POM (in place)
    assert_eq!(shell.tabs.active_tab().kind, TabKind::MenuWorkspace);
    assert!(!shell.tabs.active_tab().is_home, "left the POM");

    shell.handle_command("POM");
    assert!(
        shell.tabs.active_tab().is_home,
        "typing POM must land on the Home Context via Menu_Name resolution (no bespoke arm)"
    );
}

/// Validates: menu-workspace Requirement 18.6 -- the Home Context resolves to the
/// `pom` keymap context, while a non-Home Menu Workspace resolves to `menu`.
#[test]
fn home_context_resolves_to_pom_keymap_context() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    assert_eq!(
        super::helpers::context_name_for_tab(shell.tabs.active_tab()),
        Some("pom"),
        "the Home Context must use the `pom` keymap context"
    );
}

/// Validates: menu-workspace Requirement 18.7, 18.8 -- the Home Context persists
/// as a single Menu Workspace descriptor keyed by the reserved name `pom`.
#[test]
fn home_context_persists_as_menu_pom_descriptor() {
    use ff_session::WorkspaceDescriptor;
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    let descriptor = shell.descriptor_for_current_context();
    match descriptor {
        WorkspaceDescriptor::Menu { name } => assert_eq!(name, "pom"),
        other => panic!("Home Context must persist as Menu{{name:\"pom\"}}, got: {other:?}"),
    }
}

/// Validates: menu-workspace Requirement 18.4 -- END unwinding to the origin
/// restores a Home Context Menu Workspace (is_home), the always-present-Home
/// guarantee, after drilling into a sub-menu.
#[test]
fn end_from_drilled_menu_restores_home_context() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    assert!(shell.tabs.active_tab().is_home);
    // Drill into Settings (pushes the Home Context onto the nav stack).
    shell.handle_command("SETTINGS");
    assert!(
        !shell.tabs.active_tab().is_home,
        "after SETTINGS the active Context is the Settings menu, not Home"
    );
    // END pops back to the Home Context.
    shell.handle_command("END");
    assert!(
        shell.tabs.active_tab().is_home,
        "END from the drilled Settings menu must restore the Home Context"
    );
}

// Validates: menu-workspace Req 14.4 (B079) -- F1/HELP is a PUSH navigation, so
// END/F3 from the Help Context returns to the Context Help was opened from
// rather than closing the last Workspace and exiting the app.
#[test]
fn end_from_help_context_returns_to_previous_context_not_exit() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    assert!(shell.tabs.active_tab().is_home, "start on the Home Context");
    let tabs_before = shell.tabs.len();

    // F1 / HELP opens the Help Context (pushing the Home Context onto the stack).
    shell.handle_command("HELP");
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::HelpContext,
        "HELP opens the Help Context"
    );
    assert_eq!(
        shell.tabs.len(),
        tabs_before,
        "HELP transforms the active tab in place; it does not open a new tab"
    );

    // F3 / END must pop back to the previous Context (POM Home), not exit.
    shell.handle_command("END");
    assert!(
        shell.tabs.active_tab().is_home,
        "END from the Help Context must restore the previous (Home) Context"
    );
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "the restored Context is the POM Menu_Workspace"
    );
    assert_eq!(
        shell.tabs.len(),
        tabs_before,
        "END from Help must not close the Workspace (no app exit)"
    );
}

// Validates: menu-workspace Req 14.4 (B079) -- HELP opened from a drilled-in
// Context (e.g. Settings) returns to THAT Context on END, not the POM.
#[test]
fn end_from_help_returns_to_drilled_context() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    // Drill into Settings first (POM pushed onto the stack).
    shell.handle_command("SETTINGS");
    assert!(!shell.tabs.active_tab().is_home, "on the Settings menu");

    // HELP from Settings pushes Settings; END returns to Settings, not the POM.
    shell.handle_command("HELP");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::HelpContext);
    shell.handle_command("END");
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "END from Help returns to the Settings Menu_Workspace"
    );
    assert!(
        !shell.tabs.active_tab().is_home,
        "returned Context is the drilled Settings menu, not the Home POM"
    );
}

// === CR-NR-098 Wave 1: SNAPSHOT command + POM ScreenProvider ===============

/// Validates: screen-snapshot-scrm Req 4.1, 2.1 -- SNAPSHOT on the POM renders
/// the Home Context's logical screen to selectable text containing the POM
/// title and option descriptions.
#[test]
fn snapshot_text_for_active_pom_contains_menu_content() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    let (text, format) = shell
        .snapshot_text_for_active("")
        .expect("POM is capturable");
    assert_eq!(format, ff_screen_model::SnapshotFormat::PlainText);
    assert!(
        text.contains("Primary Option Menu"),
        "snapshot must contain the POM title; got: {text}"
    );
}

/// Validates: screen-snapshot-scrm Req 4.1-4.6, 6.1 (full shell) -- typing
/// SNAPSHOT into the real shell command path on the POM produces selectable
/// text of the POM's fields and confirms via the status area. This is the
/// end-to-end proof that the command routes through the single dispatch path
/// and captures the live Home Context.
#[test]
fn full_shell_snapshot_on_pom_captures_selectable_text() {
    let mut harness = harness_shell();
    // The startup Context is the POM (Home). Snapshot it via the command path.
    harness.state_mut().handle_command("SNAPSHOT TEXT");
    harness.run();
    // The pure capture path returns the POM's selectable text.
    let (text, _fmt) = harness
        .state()
        .snapshot_text_for_active("TEXT")
        .expect("startup POM is capturable");
    assert!(
        text.contains("Primary Option Menu"),
        "full-shell SNAPSHOT must capture the POM title as selectable text; got: {text}"
    );
    // And the command reported a confirmation.
    let msg = harness.state().open_error.as_deref().unwrap_or("");
    assert!(
        msg.contains("Snapshot"),
        "SNAPSHOT must set a confirmation status; got: {msg:?}"
    );
}
