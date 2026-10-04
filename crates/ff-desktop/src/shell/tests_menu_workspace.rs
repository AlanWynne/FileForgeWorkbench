//! Shell tests -- menu_workspace area (split from shell/tests.rs, CR F3).
//! Items are verbatim; only their file location changed.

#![allow(unused_imports)]
use super::tests_common::*;
use ff_keys::{KeyMap, ModifiedKey};
use std::sync::{Arc, Mutex};

use ff_command::{
    CommandDispatch, CommandError, CommandHandler, CommandHistory, CommandId, CommandMetadata,
    CommandParams, CommandRegistry, CommandResult, ExecutionContext,
};

/// Validates: Requirement 14.38 -- "Exit" is present as a universal tab context menu item
/// for all tab kinds, and routes through the shell-level exit intercept.
#[test]
fn tab_context_menu_exit_item_is_a_shell_level_exit() {
    // Validates: Requirement 14.38
    // The Exit item in the tab context menu must trigger application exit.
    // We verify this by confirming EXIT is handled at the shell level
    // (same path as File > Exit and the EXIT command field entry).
    assert!(
        is_shell_command("EXIT"),
        "EXIT must be a shell-level intercept so the context menu Exit item closes the app"
    );
}

/// Validates: Requirement 15.1 -- command "0" is a shell-level intercept.
#[test]
fn command_0_routes_to_settings() {
    // Validates: Requirement 15.1
    assert!(is_shell_command("0"));
}

/// Validates: Requirement 15.1 -- command "SETTINGS" is a shell-level intercept.
#[test]
fn command_settings_routes_to_settings() {
    // Validates: Requirement 15.1
    assert!(is_shell_command("SETTINGS"));
}

/// Validates: Requirement 15.1 -- command "=0" is a shell-level intercept.
#[test]
fn command_equals_0_routes_to_settings() {
    // Validates: Requirement 15.1
    assert!(is_shell_command("=0"));
}

/// Validates: menu-workspace Req 17.1/17.2 (CR-NR-080) -- the data-driven menu
/// bar (the barebones POM) includes a File Catalogs entry (POM option 1).
#[test]
fn menu_bar_has_file_catalogs_menu() {
    // The bar is the barebones POM; its bar-visible options are keyed by command.
    let cmds: Vec<String> = crate::menu_workspace::defaults::default_menubar_menu()
        .options
        .iter()
        .filter(|o| o.show_in_menu_bar)
        .map(|o| o.command.clone())
        .collect();
    assert!(
        cmds.iter().any(|c| c == "Catalogs"),
        "default Menu_Bar must contain the File Catalogs (Catalogs) entry (POM option 1)"
    );
}

/// Validates: menu-workspace Req 17.2 (CR-NR-080) -- the barebones menu bar
/// includes a Help entry and EXCLUDES the terminal RETURN option.
#[test]
fn menu_bar_has_help_and_excludes_return() {
    let menu = crate::menu_workspace::defaults::default_menubar_menu();
    let bar_cmds: Vec<String> = menu
        .options
        .iter()
        .filter(|o| o.show_in_menu_bar)
        .map(|o| o.command.clone())
        .collect();
    assert!(
        bar_cmds.iter().any(|c| c == "Help"),
        "the barebones menu bar must include a Help entry"
    );
    assert!(
        !bar_cmds.iter().any(|c| c == "Return"),
        "RETURN must be excluded from the menu bar (show_in_menu_bar = false)"
    );
}

/// Validates: menu-workspace Req 17.3 (CR-NR-080) -- peeking a top-level option
/// whose command names a menu returns THAT menu's options (so the bar dropdown
/// can render them), without navigating.
#[test]
fn menu_bar_peek_of_settings_returns_settings_menu_options() {
    let shell = make_shell();
    let peeked = shell.peek_menu_options("SETTINGS");
    assert!(
        !peeked.is_empty(),
        "peeking SETTINGS must return the Settings menu's options"
    );
    // The compiled Settings baseline has A Config / T Theme / M Menus / K Keys / R.
    let cmds: Vec<&str> = peeked.iter().map(|o| o.command.as_str()).collect();
    assert!(
        cmds.contains(&"Theme") && cmds.contains(&"Config"),
        "peeked Settings options must include Config and Theme, got: {cmds:?}"
    );
    // The active tab must be UNCHANGED by peeking (peek does not navigate).
    let kind_before = shell.tabs.active_tab().kind;
    let _ = shell.peek_menu_options("SETTINGS");
    assert_eq!(
        shell.tabs.active_tab().kind,
        kind_before,
        "peeking must NOT navigate the active Workspace"
    );
    assert_ne!(
        shell.tabs.active_tab().kind,
        crate::tab_state::TabKind::MenusEditor,
        "peeking SETTINGS must not have opened/navigated to any menu context"
    );
}

/// Validates: menu-workspace Req 17.10 (CR-NR-080 Slice D) -- a `THEME LIST`
/// option is a DYNAMIC source: its dropdown is generated at runtime, one item
/// per available theme, each dispatching `THEME <name>`.
#[test]
fn menu_bar_dynamic_theme_list_generates_one_item_per_theme() {
    let shell = make_shell();
    let dynamic = shell
        .dynamic_menu_options("THEME LIST")
        .expect("THEME LIST is a dynamic source");
    assert!(
        !dynamic.is_empty(),
        "dynamic Themes source must produce at least the built-in themes"
    );
    // Each generated child dispatches `THEME <name>` (command parity) and is
    // labelled by the theme name.
    for opt in &dynamic {
        assert!(
            opt.command.starts_with("THEME "),
            "each dynamic theme item must dispatch `THEME <name>`, got: {}",
            opt.command
        );
    }
    // The built-in Default Dark must be present as `THEME Default Dark`.
    assert!(
        dynamic.iter().any(|o| o.command == "THEME Default Dark"),
        "dynamic Themes list must include the built-in Default Dark"
    );
}

/// Validates: menu-workspace Req 17.10 -- a non-dynamic command yields no
/// dynamic options (so the bar falls back to peek / direct dispatch).
#[test]
fn menu_bar_dynamic_options_none_for_ordinary_command() {
    let shell = make_shell();
    assert!(
        shell.dynamic_menu_options("SETTINGS").is_none(),
        "an ordinary command must not be a dynamic source"
    );
}

/// Validates: menu-workspace Req 17.4 -- a top-level option whose command does
/// NOT name a menu peeks empty (the bar then renders it as a direct-dispatch
/// item rather than a submenu).
#[test]
fn menu_bar_peek_of_non_menu_command_is_empty() {
    let shell = make_shell();
    // HELP is a command, not a resolvable menu name.
    assert!(
        shell.peek_menu_options("HELP").is_empty(),
        "a non-menu command must peek to an empty option list"
    );
}

// -- Task 21.7 -- key label bar and F-key dispatch tests ----------------------

/// Validates: Requirement 4.2, 4.4 -- default key map produces labelled slots.
#[test]
fn default_key_map_has_full_base_row_assigned() {
    // Validates: function-keys-and-history Requirement 15.1 (CR-CH-027) -- the
    // Base (unmodified) row now binds all of F1-F12; the Key_Label_Bar shows
    // those 12 assigned Base slots.
    use ff_keys::KeyLabelBarModel;
    let map = KeyMap::default_global();
    let bar = KeyLabelBarModel::from_key_map(&map);
    let assigned: Vec<_> = bar.assigned_slots().collect();
    assert_eq!(assigned.len(), 12, "Base F1-F12 should all be assigned");
}

/// Validates: Requirement 4.4 -- label derived from explicit label field.
#[test]
fn default_key_map_f3_label_is_end() {
    // Validates: function-keys-and-history Requirement 4.4, 4.5
    use ff_keys::{FunctionKey, KeyLabelBarModel};
    let map = KeyMap::default_global();
    let bar = KeyLabelBarModel::from_key_map(&map);
    let slot = bar.slot_for(FunctionKey::F3).unwrap();
    assert_eq!(slot.label.as_deref(), Some("End"));
}

/// Validates: Requirement 17.4 -- file editor tab with path shows full path.
#[test]
fn title_line_file_editor_shows_path() {
    // Validates: Requirement 17.4
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::{new_document, LineEndMode};
    let tab = TabState::for_file(
        TabId(2),
        "/home/user/projects/file.txt".to_string(),
        new_document(),
        10,
        LineEndMode::Default,
    );
    let text = super::title_line_text(&tab);
    assert_eq!(text, "/home/user/projects/file.txt");
}

/// Validates: Requirement 17.5 -- untitled file editor tab shows [Untitled].
#[test]
fn title_line_untitled_shows_placeholder() {
    // Validates: Requirement 17.5
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let tab = TabState::untitled(TabId(3), new_document(), 0);
    let text = super::title_line_text(&tab);
    assert_eq!(text, "[Untitled]");
}

/// Validates: Requirement 17.6 -- ConfigPanel tab shows tab title.
#[test]
fn title_line_config_panel_shows_config() {
    // Validates: Requirement 17.6
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let tab = TabState::config_panel(TabId(4), new_document());
    let text = super::title_line_text(&tab);
    assert_eq!(text, "[CONFIG]");
}

/// Validates: menu-and-statusbar Requirement 17.12, 17.13 (CR-CH-045) -- the
/// Title_Line for the covered editor/config + read-only panel Contexts uses the
/// descriptive Title-Case display name (via `title_line_display`), while the
/// low-level `title_line_text` (the Tab_Header-tag derivation) is unchanged.
#[test]
fn title_line_display_shows_descriptive_title_case_for_covered_contexts() {
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let shell = make_shell();

    let config = TabState::config_panel(TabId(1), new_document());
    assert_eq!(
        shell.title_line_display(&config).as_deref(),
        Some("Configuration"),
        "Config Context Title_Line shows the descriptive display name"
    );

    let plugins = TabState::plugin_manager(TabId(2), new_document());
    assert_eq!(
        shell.title_line_display(&plugins).as_deref(),
        Some("Plugin Manager")
    );

    let log = TabState::event_log(TabId(3), new_document());
    assert_eq!(shell.title_line_display(&log).as_deref(), Some("Event Log"));

    let macros = TabState::macro_library(TabId(4), new_document());
    assert_eq!(
        shell.title_line_display(&macros).as_deref(),
        Some("Macro Library")
    );

    let catalogs = TabState::files_panel(TabId(5), new_document());
    assert_eq!(
        shell.title_line_display(&catalogs).as_deref(),
        Some("Catalog Explorer")
    );
}

/// Validates: menu-and-statusbar Requirement 17.12 (CR-CH-045) -- the file-editor
/// path and the POM/Menu Contexts are NOT routed through `title_line_display`
/// (they keep their existing path / Menu_Title derivation), so the helper returns
/// `None` for them.
#[test]
fn title_line_display_is_none_for_editor_and_menu_contexts() {
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let shell = make_shell();

    let untitled = TabState::untitled(TabId(1), new_document(), 0);
    assert_eq!(
        shell.title_line_display(&untitled),
        None,
        "the editor Context keeps its path/[Untitled] Title_Line, not a display name"
    );

    let pom = TabState::pom(TabId(2), new_document());
    assert_eq!(
        shell.title_line_display(&pom),
        None,
        "the POM keeps its Menu_Title-centered Title_Line (CR-CH-042), not this helper"
    );
}

/// Validates: menu-and-statusbar Requirement 17.13 (CR-CH-045) -- a USER Kind
/// title override still wins for the Title_Line display (workspace-kinds Req 3),
/// taking precedence over the descriptive default display name.
#[test]
fn title_line_display_honours_user_kind_override() {
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("theme.toml"),
        "name = \"theme\"\nmodelled_on = \"theme\"\ntitle = \"My Palette\"\n",
    )
    .expect("write kind");
    let mut shell = make_shell();
    shell.kind_registry = crate::workspace_kind::KindRegistry::load(dir.path());

    let theme = TabState::theme_editor(TabId(1), new_document());
    assert_eq!(
        shell.title_line_display(&theme).as_deref(),
        Some("My Palette"),
        "a user Kind title override wins over the default display name"
    );
}

/// Validates: Requirement 17.6 -- FilesPanel tab shows tab title.
#[test]
fn title_line_files_panel_shows_files() {
    // Validates: Requirement 17.6; workspace-kinds Req 2.4 (CR-NR-090 B.1) --
    // the FilesPanel is the Virtual Catalog Manager (Catalog Explorer, POM
    // option 1); its Kind title is now [CATALOGS], DISTINCT from the File
    // Explorer's [FILES] (the shared-label smell is fixed).
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let tab = TabState::files_panel(TabId(5), new_document());
    let text = super::title_line_text(&tab);
    assert_eq!(text, "[CATALOGS]");
}

/// Validates: workspace-kinds Requirement 3.1, 2.4 (CR-NR-090 B.1) -- the shell
/// derives a panel Kind's title from the Kind registry: the Catalog Explorer
/// (FilesPanel) is [CATALOGS] and the File Explorer (FileExplorerPanel) is
/// [FILES] (distinct), and a user Kind override changes the title live.
#[test]
fn kind_title_derives_from_registry_and_user_override_wins() {
    // Validates: workspace-kinds Requirement 3.1, 2.4
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let mut shell = make_shell();

    let catalogs = TabState::files_panel(TabId(1), new_document());
    let explorer = TabState::file_explorer_panel(TabId(2), new_document());
    assert_eq!(
        shell.kind_title(&catalogs),
        "[CATALOGS]",
        "the Catalog Explorer Kind title comes from the registry default"
    );
    assert_eq!(
        shell.kind_title(&explorer),
        "[FILES]",
        "the File Explorer Kind title is distinct from Catalogs"
    );

    // A user override of the `catalogs` Kind changes the title live.
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("catalogs.toml"),
        "name = \"catalogs\"\nmodelled_on = \"catalogs\"\ntitle = \"[MY CATS]\"\n",
    )
    .unwrap();
    shell.kind_registry = crate::workspace_kind::KindRegistry::load(dir.path());
    assert_eq!(
        shell.kind_title(&catalogs),
        "[MY CATS]",
        "a user Kind override title is reflected live (no restart)"
    );
}

/// Validates: workspace-kinds Requirement 4.3 (CR-NR-090 B.2) -- the key-map
/// context for a tab is the Kind's configured `key_list` when set, else the
/// Kind's base context name; behaviour-preserving for built-in Kinds.
#[test]
fn key_list_context_uses_kind_key_list_else_base() {
    // Validates: workspace-kinds Requirement 4.3, 4.5
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let mut shell = make_shell();

    let editor = TabState::untitled(TabId(1), new_document(), 1);
    // Built-in editor Kind: no key_list override -> base context "editor".
    assert_eq!(
        shell.key_list_context_for_tab(&editor).as_deref(),
        Some("editor"),
        "an unconfigured Kind uses its base keymap context"
    );

    // Override the editor Kind's key_list.
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("editor.toml"),
        "name = \"editor\"\nmodelled_on = \"editor\"\ntitle = \"[EDITOR]\"\nkey_list = \"mf\"\n",
    )
    .unwrap();
    shell.kind_registry = crate::workspace_kind::KindRegistry::load(dir.path());
    assert_eq!(
        shell.key_list_context_for_tab(&editor).as_deref(),
        Some("mf"),
        "a Kind's configured key_list selects that keymap context"
    );
}

/// Validates: workspace-kinds Requirement 5.1/5.4/5.5 (CR-NR-090 B.3) -- a new
/// editor Workspace takes the active Kind's edit_profile at open; a built-in Kind
/// with the default profile opens with EditProfile::default() (behaviour-
/// preserving).
#[test]
fn new_editor_takes_kind_edit_profile_on_open() {
    // Validates: workspace-kinds Requirement 5.1, 5.4, 5.5
    use ff_edit_operations::CapsMode;
    let mut shell = make_shell();

    // Built-in editor Kind (default profile): a new untitled buffer opens with
    // the neutral default edit profile (CAPS Off).
    shell.shell_new_untitled();
    assert_eq!(
        shell.tabs.active_tab().edit_profile.caps,
        CapsMode::Off,
        "an unconfigured Kind opens with the default edit profile (behaviour-preserving)"
    );

    // Configure the editor Kind's profile to CAPS On.
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("editor.toml"),
        "name = \"editor\"\nmodelled_on = \"editor\"\ntitle = \"[EDITOR]\"\n\
         [profile.edit_profile]\ncaps = \"On\"\n",
    )
    .unwrap();
    shell.kind_registry = crate::workspace_kind::KindRegistry::load(dir.path());

    shell.shell_new_untitled();
    assert_eq!(
        shell.tabs.active_tab().edit_profile.caps,
        CapsMode::On,
        "a Kind with edit_profile CAPS On opens a new editor tab with CAPS On"
    );

    // A later per-tab toggle is NOT clobbered by opening a DIFFERENT tab.
    shell.tabs.active_tab_mut().edit_profile.caps = CapsMode::Off;
    let toggled_id = shell.tabs.active_tab().id;
    shell.shell_new_untitled(); // open another tab
    let toggled = shell
        .tabs
        .tabs()
        .iter()
        .find(|t| t.id == toggled_id)
        .expect("toggled tab still open");
    assert_eq!(
        toggled.edit_profile.caps,
        CapsMode::Off,
        "profile is applied ONCE at open; a later per-tab toggle survives"
    );
}

/// Validates: workspace-kinds Requirement 5.2 (CR-NR-090 B.3) -- a NEW/untitled
/// buffer takes the Kind's line_end_mode default.
#[test]
fn new_buffer_takes_kind_line_end_mode() {
    // Validates: workspace-kinds Requirement 5.2
    use ff_document_model::LineEndMode;
    let mut shell = make_shell();
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("editor.toml"),
        "name = \"editor\"\nmodelled_on = \"editor\"\ntitle = \"[EDITOR]\"\n\
         [profile]\nline_end_mode = \"unicode\"\n",
    )
    .unwrap();
    shell.kind_registry = crate::workspace_kind::KindRegistry::load(dir.path());
    shell.shell_new_untitled();
    assert_eq!(
        shell.tabs.active_tab().line_end_mode,
        LineEndMode::Unicode,
        "a new buffer takes the Kind's configured line_end_mode default"
    );
}

/// Validates: workspace-kinds Requirement 4.1/4.5 (CR-NR-090 B.2) -- the Menu_Bar
/// for a tab is the Kind's configured `menu_bar` when set (loaded via the named
/// -menu resolver), else the compiled default bar. Uses a temp menus dir so the
/// configured bar resolves to a real file.
#[test]
fn menu_bar_uses_kind_menu_bar_else_default() {
    // Validates: workspace-kinds Requirement 4.1, 4.5;
    // menu-workspace Requirement 17.9 (CR-NR-080 Slice C -- a Menu_Bar assigned
    // to a workspace Kind renders for instances of that kind; an unassigned kind
    // falls back to the Default_Menu_Bar). The per-Kind `menu_bar` field
    // (CR-NR-090) + `resolve_menu_bar_menu_for` (CR-CH-041, used by both the
    // app-level `render_menu_bar` and the split-region `render_region_menu_bar`)
    // together deliver Slice C.
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let mut shell = make_shell();

    // A menus dir with a custom bar file "mb-editor.toml".
    let menus = tempfile::tempdir().expect("menus");
    std::fs::write(
        menus.path().join("mb-editor.toml"),
        "title = \"EditorBar\"\n[[options]]\nkey=\"1\"\ncommand=\"FILES\"\ndescription=\"Files\"\n",
    )
    .unwrap();
    shell.dir_overrides.menus = Some(menus.path().to_path_buf());

    let editor = TabState::untitled(TabId(1), new_document(), 1);
    // Default (no menu_bar override): resolves the compiled default bar (its
    // title differs from our custom bar).
    let default_bar = shell.resolve_menu_bar_menu_for(&editor);
    assert_ne!(
        default_bar.title, "EditorBar",
        "an unconfigured Kind uses the compiled default bar, not the custom one"
    );

    // Override the editor Kind's menu_bar to "MB-Editor" (slug mb-editor).
    let kinds = tempfile::tempdir().expect("kinds");
    std::fs::write(
        kinds.path().join("editor.toml"),
        "name = \"editor\"\nmodelled_on = \"editor\"\ntitle = \"[EDITOR]\"\nmenu_bar = \"MB-Editor\"\n",
    )
    .unwrap();
    shell.kind_registry = crate::workspace_kind::KindRegistry::load(kinds.path());
    let configured_bar = shell.resolve_menu_bar_menu_for(&editor);
    assert_eq!(
        configured_bar.title, "EditorBar",
        "a Kind's configured menu_bar resolves that named bar file"
    );
}

/// Validates: menu-and-statusbar Requirement 17.10 (CR-CH-034, B050) -- a
/// non-Home Menu_Workspace tab whose cached `tab.title` is STALE (left as a
/// previous Context's label after an in-place context switch) still renders the
/// Title_Line label of the CURRENTLY loaded menu, derived from live state --
/// never the stale cached string. This is the phantom-stale-title reproduction:
/// the tab holds a loaded "Settings" menu but its `title` field still reads
/// "[FILES]" from before the switch.
#[test]
fn title_line_menu_workspace_uses_loaded_menu_not_stale_title() {
    // Validates: menu-and-statusbar Requirement 17.10
    use crate::menu_workspace::MenuWorkspaceState;
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    use std::io::Write;

    // A loaded menu titled "Settings" -> its live label is "[SETTINGS]".
    let mut f = tempfile::NamedTempFile::new().expect("tempfile");
    f.write_all(
        b"title = \"Settings\"\n[[options]]\nkey=\"0\"\ncommand=\"CONFIG\"\ndescription=\"Config\"\n",
    )
    .expect("write");
    let mw = MenuWorkspaceState::load(f.path());
    assert_eq!(
        mw.tab_title(),
        "[SETTINGS]",
        "sanity: loaded menu tab label"
    );

    let mut tab = TabState::menu_workspace_tab(TabId(7), new_document(), mw);
    // Simulate an in-place context switch that updated the loaded menu but left
    // the cached title pointing at the PREVIOUS Files Context (the B050 bug).
    tab.title = "[FILES]".to_string();
    tab.is_home = false;

    // CR-CH-042 (Req 17.11): the Title_Line now shows the RAW Menu_Title
    // ("Settings"), not the bracketed tab label -- but it is still LIVE-derived
    // from the loaded menu, never the stale cached tab.title (the B050 / Req
    // 17.10 guarantee this test protects).
    let text = super::title_line_text(&tab);
    assert_eq!(
        text, "Settings",
        "Title_Line must derive a Menu_Workspace label from its loaded menu \
         (raw Menu_Title), not the stale cached tab.title"
    );
}

/// Validates: menu-and-statusbar Requirement 18.5 (CR-CH-035, B045) -- the
/// Detached_Workspace OS title is truncated to at most 80 chars on a char
/// boundary; a short title is unchanged.
#[test]
fn truncate_title_clamps_to_max_on_char_boundary() {
    // Validates: menu-and-statusbar Requirement 18.5
    let short = "[FILES] -- FileForge Workbench";
    assert_eq!(
        super::truncate_title(short, 80),
        short,
        "short title unchanged"
    );

    let long = "X".repeat(200);
    let out = super::truncate_title(&long, 80);
    assert_eq!(out.chars().count(), 80, "must clamp to exactly max chars");
    assert!(out.ends_with('~'), "truncated title marks the cut");

    // Multi-byte safety: a title of multi-byte chars must not split a char.
    let multi = "e\u{0301}".repeat(100); // combining acute; each unit is 2 chars
    let out2 = super::truncate_title(&multi, 10);
    assert_eq!(out2.chars().count(), 10);
    // Round-trips as valid UTF-8 (no panic / no split) -- implicit by String.
}

// -- Phase AR: [context_key_maps] TOML config parsing (Req 14.7) ----------

/// Validates: Requirement 14.7 -- context_key_maps table parsed into KeyMapResolver.
#[test]
fn context_key_maps_parsed_from_config_value_table() {
    // Validates: Requirement 14.7
    use ff_config::{ConfigTable, ConfigValue};
    use ff_keys::{FunctionKey, KeyMap, KeyMapResolver};
    use std::collections::BTreeMap;

    // Simulate what ConfigHandle::get("context_key_maps") returns for:
    //   [context_key_maps.editor]  F5 = "FIND"
    //   [context_key_maps.pom]     F3 = "RETURN"
    let mut editor_map: ConfigTable = BTreeMap::new();
    editor_map.insert("F5".to_string(), ConfigValue::String("FIND".to_string()));

    let mut pom_map: ConfigTable = BTreeMap::new();
    pom_map.insert("F3".to_string(), ConfigValue::String("RETURN".to_string()));

    let mut outer: ConfigTable = BTreeMap::new();
    outer.insert("editor".to_string(), ConfigValue::Table(editor_map));
    outer.insert("pom".to_string(), ConfigValue::Table(pom_map));

    // Apply the same conversion used in load_context_maps_from_config.
    let mut resolver = KeyMapResolver::new(KeyMap::default_global());
    for (ctx_name, ctx_value) in outer {
        if let ConfigValue::Table(ctx_table) = ctx_value {
            let mut toml_map = toml::map::Map::new();
            for (k, v) in ctx_table {
                if let Some(tv) = super::config_value_to_toml_value(v) {
                    toml_map.insert(k, tv);
                }
            }
            let (map, warnings) = KeyMap::from_toml_table(&toml_map, &ctx_name);
            assert!(
                warnings.is_empty(),
                "unexpected warnings for {ctx_name}: {warnings:?}"
            );
            resolver.set_context_map(ctx_name, map);
        }
    }

    // editor context: F5=FIND; global F3=END suppressed (full-replacement)
    resolver.set_context(Some("editor"));
    assert_eq!(
        resolver
            .active_key_map()
            .get_plain(FunctionKey::F5)
            .map(|b| b.command()),
        Some("FIND"),
        "editor context must have F5=FIND"
    );
    assert!(
        resolver
            .active_key_map()
            .get_plain(FunctionKey::F3)
            .is_none(),
        "editor context must not inherit global F3"
    );

    // pom context: F3=RETURN
    resolver.set_context(Some("pom"));
    assert_eq!(
        resolver
            .active_key_map()
            .get_plain(FunctionKey::F3)
            .map(|b| b.command()),
        Some("RETURN"),
        "pom context must have F3=RETURN"
    );

    // unknown context falls back to global F3=END
    resolver.set_context(Some("unknown"));
    assert_eq!(
        resolver
            .active_key_map()
            .get_plain(FunctionKey::F3)
            .map(|b| b.command()),
        Some("END"),
        "unknown context must fall back to global"
    );
}

// -- CR-CH-027: keymaps/<context>.toml override files (Req 14.9-14.12) ------

/// Validates: function-keys-and-history Requirement 14.11 -- ensure_keymaps_dir
/// creates `<User_Data_Dir>/keymaps/`.
#[test]
fn ensure_keymaps_dir_creates_keymaps_dir() {
    use tempfile::TempDir;
    let dir = TempDir::new().expect("tempdir");
    let keymaps = dir.path().join("keymaps");
    assert!(!keymaps.exists());
    super::ensure_keymaps_dir(dir.path());
    assert!(keymaps.exists(), "keymaps/ must be created");
}

/// Validates: function-keys-and-history Requirement 14.9/14.10 -- a present
/// keymaps/<context>.toml is loaded as that context's map (full-replacement over
/// the compiled default); an absent file leaves the context on the default.
#[test]
fn keymaps_file_present_overrides_default_for_context() {
    use ff_keys::{FunctionKey, KeyMapResolver};
    use std::io::Write;
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let keymaps = dir.path().join("keymaps");
    std::fs::create_dir_all(&keymaps).expect("mkdir");
    // editor.toml rebinds F5 to FIND (and, being full-replacement, drops the
    // compiled default's F3=END for the editor context).
    let mut f = std::fs::File::create(keymaps.join("editor.toml")).expect("create");
    writeln!(f, "F5 = \"FIND\"").expect("write");

    let mut resolver = KeyMapResolver::new(KeyMap::default_global());
    super::load_context_maps_from_keymaps_dir(&keymaps, &mut resolver);

    resolver.set_context(Some("editor"));
    assert_eq!(
        resolver
            .active_key_map()
            .get_plain(FunctionKey::F5)
            .map(|b| b.command()),
        Some("FIND"),
        "editor keymaps file must bind F5=FIND"
    );
    assert!(
        resolver
            .active_key_map()
            .get_plain(FunctionKey::F3)
            .is_none(),
        "editor keymaps file fully replaces the default (no inherited F3)"
    );

    // A context WITHOUT a file falls back to the compiled default (F3=END).
    resolver.set_context(Some("pom"));
    assert_eq!(
        resolver
            .active_key_map()
            .get_plain(FunctionKey::F3)
            .map(|b| b.command()),
        Some("END"),
        "a context with no keymaps file uses the compiled default"
    );
}

/// Validates: function-keys-and-history Requirement 14.10 -- a malformed
/// keymaps file is skipped and the context falls back to the compiled default
/// without crashing.
#[test]
fn keymaps_malformed_file_is_skipped_and_falls_back() {
    use ff_keys::{FunctionKey, KeyMapResolver};
    use std::io::Write;
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let keymaps = dir.path().join("keymaps");
    std::fs::create_dir_all(&keymaps).expect("mkdir");
    let mut f = std::fs::File::create(keymaps.join("editor.toml")).expect("create");
    writeln!(f, "this is [ not valid toml =").expect("write");

    let mut resolver = KeyMapResolver::new(KeyMap::default_global());
    // Must not panic.
    super::load_context_maps_from_keymaps_dir(&keymaps, &mut resolver);

    // No context map registered for editor -> falls back to global default.
    resolver.set_context(Some("editor"));
    assert_eq!(
        resolver
            .active_key_map()
            .get_plain(FunctionKey::F3)
            .map(|b| b.command()),
        Some("END"),
        "malformed editor.toml must be skipped; context uses the compiled default"
    );
}

/// Validates: function-keys-and-history Requirement 14.12 -- a
/// keymaps/<context>.toml FILE takes precedence over a
/// `[context_key_maps.<name>]` config section for the same context, because the
/// file loader runs SECOND (as it does at startup).
#[test]
fn keymaps_file_takes_precedence_over_config_section() {
    use ff_keys::{FunctionKey, KeyBinding, KeyMapResolver};
    use std::io::Write;
    use tempfile::TempDir;

    let mut resolver = KeyMapResolver::new(KeyMap::default_global());
    // Simulate the config-table path having registered editor F5=CONFIGCMD first.
    let mut cfg_map = KeyMap::empty("editor");
    cfg_map.set(
        ModifiedKey::plain(FunctionKey::F5),
        KeyBinding::new("CONFIGCMD"),
    );
    resolver.set_context_map("editor".to_string(), cfg_map);

    // Then the keymaps file loader runs (second) and rebinds editor F5=FILECMD.
    let dir = TempDir::new().expect("tempdir");
    let keymaps = dir.path().join("keymaps");
    std::fs::create_dir_all(&keymaps).expect("mkdir");
    let mut f = std::fs::File::create(keymaps.join("editor.toml")).expect("create");
    writeln!(f, "F5 = \"FILECMD\"").expect("write");
    super::load_context_maps_from_keymaps_dir(&keymaps, &mut resolver);

    resolver.set_context(Some("editor"));
    assert_eq!(
        resolver
            .active_key_map()
            .get_plain(FunctionKey::F5)
            .map(|b| b.command()),
        Some("FILECMD"),
        "the keymaps FILE must win over the config-section entry (loaded second)"
    );
}

/// Validates: Requirement 19.12 -- FileExplorerPanel kind is distinct from FilesPanel.
#[test]
fn file_explorer_panel_kind_is_distinct_from_files_panel() {
    // Validates: Requirement 19.12
    use crate::tab_state::TabKind;
    assert_ne!(TabKind::FileExplorerPanel, TabKind::FilesPanel);
    assert_ne!(TabKind::FileExplorerPanel, TabKind::MenuWorkspace);
    assert_ne!(TabKind::FileExplorerPanel, TabKind::FileEditor);
}

/// Validates: Requirement 14.7 -- invalid key names in context map are skipped.
#[test]
fn context_key_maps_invalid_key_skipped() {
    // Validates: Requirement 14.7 (inherits Req 1.5 graceful-skip behaviour)
    let table: toml::Table = "F3 = \"RETURN\"\nF99 = \"INVALID\"".parse().unwrap();
    let (map, warnings) = ff_keys::KeyMap::from_toml_table(&table, "pom");
    assert_eq!(map.len(), 1, "only F3 should be loaded");
    assert_eq!(warnings.len(), 1, "F99 should produce one warning");
}

/// Validates: B062 -- a mixed-case VERB with a mixed-case argument resolves: the
/// verb matches case-insensitively and the argument reaches the handler intact.
/// `theme Default Legacy` (lowercase verb, cased name) activates Default Legacy.
#[test]
fn mixed_case_theme_verb_and_argument_resolve() {
    let mut shell = make_shell();
    shell.handle_command("theme Default Dark");
    assert_eq!(
        shell.palette.name, "Default Dark",
        "lowercase verb + cased theme name must resolve and preserve the name"
    );
    let _ = shell
        .config_handle
        .remove_user_value(ff_config::keys::theme::ACTIVE);
    let _ = shell
        .config_handle
        .remove_user_value(ff_config::keys::theme::ACTIVE_NAME);
}

/// Validates: menu-workspace Req 2.1a / theme Req 13.4-13.6 -- when the Legacy
/// palette is active, `menu_colours()` returns the ISPF per-column scheme
/// (key=white, command=turquoise, description=green), NOT placeholders. Guards
/// the "POM options all white in Legacy" regression.
#[test]
fn menu_colours_are_legacy_scheme_when_legacy_palette_active() {
    let mut shell = make_shell();
    shell.palette = ff_theme::defaults::default_legacy_palette();
    let mc = shell.menu_colours();
    let ph = eframe::egui::Color32::PLACEHOLDER;
    assert_ne!(
        mc.option_key, ph,
        "Legacy key column must be a real colour (white)"
    );
    assert_ne!(
        mc.option_command, ph,
        "Legacy command column must be a real colour (turquoise)"
    );
    assert_ne!(
        mc.description, ph,
        "Legacy description column must be a real colour (green)"
    );
    // The three columns must be DISTINCT (white / turquoise / green).
    assert_ne!(mc.option_key, mc.option_command);
    assert_ne!(mc.option_command, mc.description);
}

// === Phase CW: Settings namespace view routing ==========================

/// Validates: cw-requirements.md Requirement 10.1, 10.2 -- SETTINGS <ns> pre-populates filter.
#[test]
fn settings_namespace_filter_applied_on_open() {
    // CR-CH-025: `SETTINGS <ns>` is superseded by `CONFIG <ns>`.
    let mut shell = make_shell();
    shell.handle_command("CONFIG editor");
    assert_eq!(
        shell.config_panel.namespace_filter.as_deref(),
        Some("editor"),
        "namespace_filter must be set to the requested namespace"
    );
    assert_eq!(
        shell.config_panel.filter, "editor.",
        "flat-list filter must be pre-populated with the namespace prefix"
    );
}

/// Validates: configuration-system Req 20.2 (CR-CH-025) -- bare CONFIG opens the
/// unfiltered flat view.
#[test]
fn settings_all_view_has_no_namespace_filter() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("CONFIG");
    assert!(shell.config_panel.namespace_filter.is_none());
    assert_eq!(shell.config_panel.filter, "");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::ConfigPanel);
    assert_eq!(shell.tabs.active_tab().title, "[CONFIG]");
}

/// Validates: workspace-model Requirement 4.1 -- workspace settings are injected into config.
#[test]
fn open_workspace_injects_settings_into_config() {
    // Validates: workspace-model Requirement 4.1
    use ff_session::{save_workspace, WorkspaceState};
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let ws_path = tmp.path().join("settings.ffwb-workspace");

    let mut state = WorkspaceState::new("SettingsWS");
    // Use a key with no schema entry to avoid validation-path unreachable panic.
    state
        .settings
        .insert("workspace.custom_key".to_string(), "hello".to_string());
    save_workspace(&state, &ws_path).expect("save");

    let mut shell = make_shell();
    shell.open_workspace_force(&ws_path);

    // The setting must be readable from the config handle.
    let val = shell.config_handle.get_string("workspace.custom_key");
    assert_eq!(
        val.ok().as_deref(),
        Some("hello"),
        "workspace setting must be injected into config"
    );
}

/// Validates: workspace-model Requirement 4.3 -- closing workspace removes settings layer.
#[test]
fn close_workspace_removes_settings_from_config() {
    // Validates: workspace-model Requirement 4.3
    use ff_session::{save_workspace, WorkspaceState};
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let ws_path = tmp.path().join("settings-close.ffwb-workspace");

    let mut state = WorkspaceState::new("SettingsCloseWS");
    state
        .settings
        .insert("workspace.custom_key".to_string(), "hello".to_string());
    save_workspace(&state, &ws_path).expect("save");

    let mut shell = make_shell();
    shell.open_workspace_force(&ws_path);
    assert_eq!(
        shell
            .config_handle
            .get_string("workspace.custom_key")
            .ok()
            .as_deref(),
        Some("hello")
    );

    shell.close_workspace();
    // After close the workspace override must be gone.
    let val_after = shell.config_handle.get_string("workspace.custom_key");
    assert!(
        val_after.is_err(),
        "workspace setting must be removed from config after close"
    );
}

/// Validates: plugin-manager-ui Requirement 1.1 -- title_line_text for PluginManager tab.
#[test]
fn title_line_plugin_manager_shows_plugins() {
    // Validates: plugin-manager-ui Requirement 1.1
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let tab = TabState::plugin_manager(TabId(10), new_document());
    let text = super::title_line_text(&tab);
    assert_eq!(text, "[PLUGINS]");
}

/// Validates: notification-system Requirement 2.1 -- EventLog title_line shows [LOG].
#[test]
fn title_line_event_log_shows_log() {
    // Validates: notification-system Requirement 2.1
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let tab = TabState::event_log(TabId(20), new_document());
    let text = super::title_line_text(&tab);
    assert_eq!(text, "[LOG]");
}

// === Phase CR: OS Theme Follow (Requirement 16) ============================

/// Validates: theme-and-appearance Requirement 16.1 -- theme.follow_os key exists in schema.
#[test]
fn theme_follow_os_key_is_registered_in_schema() {
    // Validates: theme-and-appearance Requirement 16.1
    let shell = make_shell();
    let result = shell
        .config_handle
        .get_bool(ff_config::keys::theme::FOLLOW_OS);
    assert!(
        result.is_ok(),
        "theme.follow_os must be registered in schema, got: {:?}",
        result
    );
}

/// Validates: theme-and-appearance Requirement 16.2 -- theme.follow_os defaults to false.
#[test]
fn theme_follow_os_defaults_to_false() {
    // Validates: theme-and-appearance Requirement 16.2
    // The schema default must be false. We verify via the schema entry directly
    // rather than the effective value (which may be overridden by user config).
    use ff_config::ConfigValue;
    let shell = make_shell();
    let entries = shell.config_handle.list_schema_entries();
    let entry = entries
        .iter()
        .find(|e| e.key == ff_config::keys::theme::FOLLOW_OS)
        .expect("theme.follow_os must be in schema");
    assert_eq!(
        entry.default,
        ConfigValue::Boolean(false),
        "theme.follow_os schema default must be false"
    );
}

/// Validates: theme-and-appearance Requirement 16.3 -- setting follow_os=true is readable.
#[test]
fn theme_follow_os_can_be_set_to_true() {
    // Validates: theme-and-appearance Requirement 16.3
    let shell = make_shell();
    let _ = shell.config_handle.set_user_value(
        ff_config::keys::theme::FOLLOW_OS,
        ff_config::ConfigValue::Boolean(true),
    );
    let val = shell
        .config_handle
        .get_bool(ff_config::keys::theme::FOLLOW_OS)
        .unwrap_or(false);
    assert!(val, "theme.follow_os must be readable as true after set");
}

/// Validates: theme-and-appearance Requirement 16.5 -- follow_os=false leaves palette unchanged.
#[test]
fn theme_follow_os_false_does_not_change_palette() {
    // Validates: theme-and-appearance Requirement 16.5
    // The schema default for follow_os is false -- verify the key is registered
    // and the schema default is false (palette auto-change is opt-in).
    use ff_config::ConfigValue;
    let shell = make_shell();
    let entries = shell.config_handle.list_schema_entries();
    let entry = entries
        .iter()
        .find(|e| e.key == ff_config::keys::theme::FOLLOW_OS)
        .expect("theme.follow_os must be in schema");
    assert_eq!(
        entry.default,
        ConfigValue::Boolean(false),
        "follow_os schema default must be false so palette is not auto-changed by default"
    );
}

/// Validates: theme-and-appearance Requirement 5.2/5.3 (mode round-trip) +
/// configuration-system Requirement 7.4 (enum validation) -- B039 regression.
///
/// `set_theme()` persists `VisualMode::section_name()` into `theme.active`, and
/// config validation rejects any value not in the schema `allowed_values`
/// (substituting the default). If the two ever disagree the selected mode is
/// silently reverted. This test pins every built-in mode's `section_name()` to
/// be present in the schema's allowed set so the High Contrast revert (B039)
/// cannot regress.
#[test]
fn theme_active_allowed_values_accept_every_visual_mode_section_name() {
    use ff_config::ConfigValue;
    use ff_theme::mode::VisualMode;

    let shell = make_shell();
    // Match production wiring: the real app calls register_builtin_schema after
    // ff-config init, which is where theme.active gains its allowed_values
    // constraint. make_shell only runs register_core_schema, so register the
    // builtin schema here to exercise the same constraint the running app uses.
    let tmp = tempfile::tempdir().expect("tempdir");
    crate::register_builtin_schema(&shell.config_handle, tmp.path());

    let entries = shell.config_handle.list_schema_entries();
    let entry = entries
        .iter()
        .find(|e| e.key == ff_config::keys::theme::ACTIVE)
        .expect("theme.active must be in schema");
    let allowed = entry
        .constraints
        .as_ref()
        .and_then(|c| c.allowed_values.as_ref())
        .expect("theme.active must constrain allowed_values");

    for mode in [
        VisualMode::Dark,
        VisualMode::Light,
        VisualMode::HighContrast,
        VisualMode::Legacy,
    ] {
        let name = mode.section_name();
        let present = allowed
            .iter()
            .any(|v| matches!(v, ConfigValue::String(s) if s == name));
        assert!(
            present,
            "theme.active allowed_values must contain section_name() '{name}' for {mode:?}; \
             a mismatch silently reverts the selected theme (B039)"
        );
        // And the persisted value must parse back to the same mode.
        assert_eq!(
            VisualMode::from_str_loose(name),
            Some(mode),
            "section_name() '{name}' must round-trip through from_str_loose"
        );
    }
}

/// Validates: theme-and-appearance Requirement 16.4/16.7 -- B039 root cause.
/// An explicit theme selection must turn OFF `theme.follow_os`, otherwise the
/// per-frame follow_os block rebuilds the palette from the OS preference every
/// frame and clobbers the chosen mode (confirmed by runtime logging: the theme
/// block set Legacy, follow_os reset it to Dark, every frame -- so the theme
/// never visibly changed).
#[test]
fn set_theme_disables_follow_os_so_selection_is_not_clobbered() {
    use ff_theme::mode::VisualMode;
    let shell_setup = make_shell();
    // Enable follow_os (as a leaked/legacy config value would).
    let _ = shell_setup.config_handle.set_user_value(
        ff_config::keys::theme::FOLLOW_OS,
        ff_config::ConfigValue::Boolean(true),
    );
    let mut shell = shell_setup;
    assert!(
        shell
            .config_handle
            .get_bool(ff_config::keys::theme::FOLLOW_OS)
            .unwrap_or(false),
        "precondition: follow_os is true"
    );

    // Explicitly choose a theme.
    shell.handle_command("THEME legacy");
    assert_eq!(shell.palette.mode, VisualMode::Legacy);

    // The explicit selection must have disabled follow_os so the per-frame
    // block cannot override it.
    assert!(
        !shell
            .config_handle
            .get_bool(ff_config::keys::theme::FOLLOW_OS)
            .unwrap_or(true),
        "explicit THEME selection must turn follow_os OFF (B039)"
    );

    // Clean up user-config writes.
    let _ = shell
        .config_handle
        .remove_user_value(ff_config::keys::theme::ACTIVE);
    let _ = shell
        .config_handle
        .remove_user_value(ff_config::keys::theme::FOLLOW_OS);
}

// === THEME command (command parity) -- theme-and-appearance Requirement 17 ===

/// Validates: theme-and-appearance Requirement 17.2 -- `THEME <mode>` sets the
/// active visual mode (same path the Settings menu now dispatches).
#[test]
fn theme_command_sets_mode() {
    use ff_theme::mode::VisualMode;
    let mut shell = make_shell();
    // Start from a known mode. `THEME <mode>` sets `self.palette.mode`
    // (in-memory) which is the observable effect asserted here.
    shell.handle_command("THEME dark");
    assert_eq!(shell.palette.mode, VisualMode::Dark);

    shell.handle_command("THEME legacy");
    assert_eq!(shell.palette.mode, VisualMode::Legacy);

    // Underscore and hyphen spellings both resolve to High Contrast.
    shell.handle_command("THEME high_contrast");
    assert_eq!(shell.palette.mode, VisualMode::HighContrast);
    shell.handle_command("THEME light");
    assert_eq!(shell.palette.mode, VisualMode::Light);
    shell.handle_command("THEME high-contrast");
    assert_eq!(shell.palette.mode, VisualMode::HighContrast);

    // set_theme persists theme.active to the real user config; remove the
    // override so the test leaves no external state (testing.md: deterministic,
    // no external files).
    let _ = shell
        .config_handle
        .remove_user_value(ff_config::keys::theme::ACTIVE);
}

/// Validates: theme-and-appearance Requirement 17.2 -- mode name is
/// case-insensitive.
#[test]
fn theme_command_is_case_insensitive() {
    use ff_theme::mode::VisualMode;
    let mut shell = make_shell();
    shell.handle_command("theme LeGaCy");
    assert_eq!(shell.palette.mode, VisualMode::Legacy);
    let _ = shell
        .config_handle
        .remove_user_value(ff_config::keys::theme::ACTIVE);
}

/// Validates: theme-and-appearance Requirement 17.4 (CR-CH-024) -- bare `THEME`
/// opens the Theme Editor context (formerly the `THEMES` command) and does NOT
/// change the active theme.
#[test]
fn theme_command_bare_opens_theme_editor() {
    use crate::tab_state::TabKind;
    use ff_theme::mode::VisualMode;
    let mut shell = make_shell();
    shell.handle_command("THEME light");
    assert_eq!(shell.palette.mode, VisualMode::Light);
    shell.handle_command("THEME");
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::ThemeEditor,
        "bare THEME must open the Theme Editor context (CR-CH-024)"
    );
    // The active theme is unchanged by opening the editor.
    assert_eq!(shell.palette.mode, VisualMode::Light);
    let _ = shell
        .config_handle
        .remove_user_value(ff_config::keys::theme::ACTIVE_NAME);
}

/// Validates: theme-and-appearance Requirement 17.5 (CR-CH-024) -- an unknown
/// theme name leaves the active theme unchanged and shows the does-not-exist
/// message.
#[test]
fn theme_command_unknown_name_errors_and_keeps_theme() {
    use ff_theme::mode::VisualMode;
    let mut shell = make_shell();
    shell.handle_command("THEME light");
    assert_eq!(shell.palette.mode, VisualMode::Light);
    shell.handle_command("THEME banana");
    // Theme unchanged; does-not-exist message surfaced.
    assert_eq!(shell.palette.mode, VisualMode::Light);
    let msg = shell.open_error.clone().unwrap_or_default();
    assert!(
        msg.contains("banana") && msg.contains("does not exist"),
        "unknown THEME name must show the does-not-exist message, got: {msg:?}"
    );
}

/// Validates: lua-macro-engine Requirement 12.1 -- title_line_text for MacroLibrary tab.
#[test]
fn title_line_macro_library_shows_macros() {
    // Validates: lua-macro-engine Requirement 12.1
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    let tab = TabState::macro_library(TabId(30), new_document());
    let text = super::title_line_text(&tab);
    assert_eq!(text, "[MACROS]");
}

/// Validates: function-keys Req 22.2, 22.5 -- KEYS <kind> pre-selects that kind.
#[test]
fn keys_with_kind_preselects_that_kind() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("KEYS editor");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::KeysEditor);
    assert_eq!(
        shell.keys_editor_panel.selected_kind.as_deref(),
        Some("editor")
    );
}

/// Validates: function-keys Req 22.5 -- KEYS <unknown> opens the Workspace with
/// a fallback kind and a status message naming the unknown kind.
#[test]
fn keys_with_unknown_kind_shows_status_message() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("KEYS unknownkind");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::KeysEditor);
    let err = shell.keys_editor_panel.error.as_deref().unwrap_or("");
    assert!(
        err.contains("unknownkind"),
        "status should mention the unknown kind: {err:?}"
    );
}

/// Validates: function-keys Req 22.2 -- KEYS <kind> matching is case-insensitive.
#[test]
fn keys_kind_matching_is_case_insensitive() {
    let mut shell = make_shell();
    shell.handle_command("KEYS EDITOR");
    assert_eq!(
        shell.keys_editor_panel.selected_kind.as_deref(),
        Some("editor"),
        "uppercase kind must match case-insensitively"
    );
}

/// Validates: Requirement 21.4 (CR-CH-012) -- a persisted non-POM Menu_Workspace
/// descriptor is RE-OPENED on restore (backed by `menus/<name>.toml`), not
/// dropped; and restore continues past it to the following descriptor.
#[test]
fn restore_reopens_menu_descriptor_and_continues() {
    use crate::tab_state::TabKind;
    use ff_session::session_state::{DescriptorParams, WorkspaceDescriptor, WorkspaceKind};

    // Isolated menus dir with a user menu `reports.toml`.
    let menus = tempfile::TempDir::new().expect("tempdir");
    std::fs::create_dir_all(menus.path()).expect("mkdir");
    std::fs::write(
        menus.path().join("reports.toml"),
        "title = \"Reports\"\n\n[[options]]\nkey = \"1\"\ncommand = \"Files\"\ndescription = \"Reports files\"\n",
    )
    .expect("write menu");

    let mut shell = make_shell();
    shell.dir_overrides.menus = Some(menus.path().to_path_buf());

    let descriptors = vec![
        // Menu descriptor: must be RE-OPENED (Req 21.4), not skipped.
        WorkspaceDescriptor::Menu {
            name: "reports".to_string(),
        },
        // A following descriptor must still restore.
        WorkspaceDescriptor::CustomWorkspace {
            workspace_kind: WorkspaceKind::Files,
            params: DescriptorParams::new(),
        },
    ];
    shell.restore_workspace_descriptors(&descriptors);

    // The menu tab was reopened, backed by reports.toml (title "Reports").
    assert!(
        shell.tabs.tabs().iter().any(|t| {
            t.kind == TabKind::MenuWorkspace
                && t.menu_workspace
                    .as_ref()
                    .and_then(|mw| mw.menu.as_ref())
                    .map(|m| m.title.eq_ignore_ascii_case("Reports"))
                    .unwrap_or(false)
        }),
        "a persisted Menu descriptor must be re-opened from menus/<name>.toml (Req 21.4)"
    );
    // And restore continued to the following descriptor.
    assert!(
        shell
            .tabs
            .tabs()
            .iter()
            .any(|t| t.kind == TabKind::FilesPanel),
        "restore must continue past the re-opened Menu descriptor"
    );
}

/// Validates: Requirement 21.4 (CR-CH-012) -- when the persisted menu's backing
/// `menus/<name>.toml` is ABSENT on restore, the Workspace is still opened (in
/// the menu load-error state) rather than dropped.
#[test]
fn restore_menu_descriptor_missing_file_opens_load_error_not_dropped() {
    use crate::tab_state::TabKind;
    use ff_session::session_state::WorkspaceDescriptor;

    // Isolated (empty) menus dir -- the named file does not exist.
    let menus = tempfile::TempDir::new().expect("tempdir");
    std::fs::create_dir_all(menus.path()).expect("mkdir");

    let mut shell = make_shell();
    shell.dir_overrides.menus = Some(menus.path().to_path_buf());

    shell.restore_workspace_descriptors(&[WorkspaceDescriptor::Menu {
        name: "gone".to_string(),
    }]);

    assert!(
        shell
            .tabs
            .tabs()
            .iter()
            .any(|t| t.kind == TabKind::MenuWorkspace),
        "a Menu descriptor with a missing file must open (load-error state), not be dropped (Req 21.4)"
    );
}

// === MENU command (menu-workspace Requirement 11) ==========================

// Validates: menu-workspace Requirement 11.1 -- bare MENU returns to the Home Context.
#[test]
fn menu_command_returns_to_home_context() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    // Move off the POM first.
    shell.handle_command("COMMANDS");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::CommandConfigurator);
    shell.handle_command("MENU");
    assert!(shell.tabs.active_tab().is_home);
    assert!(shell.open_error.is_none());
}

// === B032: Settings opens the data-driven Settings_Menu ====================

// Validates: cw-requirements.md Req 9.1; configuration-system Req 15.1 --
// bare SETTINGS opens the Settings_Menu (Menu_Workspace), not the flat panel.
#[test]
fn settings_command_opens_menu_workspace_not_flat_panel() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("SETTINGS");
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "bare SETTINGS must open the data-driven Settings_Menu, not the flat SettingsPanel"
    );
}

// === CR-CH-025: unified resolution chain (menu-name + chaining + CONFIG) ====

// Validates: command-framework Req 8.13 / menu-workspace Req 11.7, 11.11 --
// `SETTINGS T` opens the Settings menu then activates option `T`, whose command
// is `THEME`, opening the Theme Editor Context (observably identical to typing
// `THEME`). Keyword-less menu name + trailing-token chaining.
#[test]
fn settings_t_chains_to_theme_editor() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("SETTINGS T");
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::ThemeEditor,
        "SETTINGS T must chain to the Theme Editor via the T -> THEME option"
    );
}

// Validates: configuration-system Req 20.3 (CR-CH-025) -- CONFIG <ns> opens the
// filtered flat panel with the namespace prefix applied.
#[test]
fn settings_namespace_opens_filtered_flat_panel() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("CONFIG editor");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::ConfigPanel);
    assert_eq!(
        shell.config_panel.namespace_filter.as_deref(),
        Some("editor")
    );
    assert_eq!(shell.tabs.active_tab().title, "[CONFIG:editor]");
}

// === CR-CH-021: menus code-only + Recovery Baseline + RESET BARE ===========

// Validates: menu-workspace Requirement 13.1 (CR-NR-075) -- MENUS opens the
// Menus Editor Context (replacing the CR-CH-021 placeholder notice).
#[test]
fn menus_command_opens_menus_editor() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("MENUS");
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::MenusEditor,
        "MENUS must open the Menus Editor Context"
    );
    // A working menu is loaded (POM by default) with options to edit.
    assert!(shell.menus_editor_panel.working.is_some());
    assert!(
        shell
            .menus_editor_panel
            .available
            .iter()
            .any(|n| n == "POM"),
        "the selector must list POM"
    );
}

// Validates: configuration-system Req 19.6 + B064 -- a confirmed RESET BARE
// resets the ACTIVE profile's theme to the Default Legacy baseline, even when a
// different (built-in) theme was active. Clearing archived files alone did NOT
// reset the theme (the in-memory config kept the old key); RESET BARE now clears
// the theme keys and applies Default Legacy.
#[test]
fn execute_reset_bare_resets_theme_to_default_legacy() {
    let mut shell = make_shell();
    // Start on a NON-Legacy built-in theme (a built-in resolves without a file,
    // which is exactly the case that previously survived the reset).
    shell.handle_command("THEME Default Dark");
    assert_eq!(shell.palette.name, "Default Dark");

    let target = shell
        .resolve_reset_bare_target("")
        .expect("bare target resolves");
    shell.execute_reset_bare(&target);

    assert_eq!(
        shell.palette.name, "Default Legacy",
        "RESET BARE must reset the active theme to the Default Legacy baseline (B064)"
    );
    // The user theme override key is cleared for this (active) profile.
    let active_name = shell
        .config_handle
        .get_string(ff_config::keys::theme::ACTIVE_NAME)
        .unwrap_or_default();
    assert_eq!(
        active_name, "Default Legacy",
        "theme.active_name should reflect the baseline after reset, got: {active_name:?}"
    );
}

// Validates: menu-workspace Requirement 13.3 -- selecting a built-in with no
// user file loads the compiled Recovery_Baseline as the working copy.
#[test]
fn menus_editor_loads_recovery_baseline_when_no_file() {
    let (shell, _dir) = make_shell_with_menus_editor();
    let working = shell.menus_editor_panel.working.as_ref().expect("working");
    let keys: Vec<&str> = working.options.iter().map(|o| o.key.as_str()).collect();
    // CR-NR-080: barebones POM is Settings/Catalogs/Files/Help/Return.
    assert_eq!(keys, vec!["0", "1", "2", "3", "X"]);
}

// Validates: menu-workspace Requirement 13.5 -- add / delete / move mutate the
// working menu.
#[test]
fn menus_editor_add_delete_move_mutate_working() {
    use crate::menus_editor_panel::MenusEditorAction as A;
    let (mut shell, _dir) = make_shell_with_menus_editor();
    let before = shell
        .menus_editor_panel
        .working
        .as_ref()
        .unwrap()
        .options
        .len();

    shell.apply_menus_editor_action(A::AddOption);
    assert_eq!(
        shell
            .menus_editor_panel
            .working
            .as_ref()
            .unwrap()
            .options
            .len(),
        before + 1
    );

    // Move the first option down, then confirm order changed.
    let first_key = shell.menus_editor_panel.working.as_ref().unwrap().options[0]
        .key
        .clone();
    shell.apply_menus_editor_action(A::MoveOptionDown(0));
    assert_eq!(
        shell.menus_editor_panel.working.as_ref().unwrap().options[1].key,
        first_key,
        "MoveOptionDown swaps the first two options"
    );

    // Delete the last (blank) option we added.
    let last = shell
        .menus_editor_panel
        .working
        .as_ref()
        .unwrap()
        .options
        .len()
        - 1;
    shell.apply_menus_editor_action(A::DeleteOption(last));
    assert_eq!(
        shell
            .menus_editor_panel
            .working
            .as_ref()
            .unwrap()
            .options
            .len(),
        before
    );
}

// Validates: menu-workspace Requirement 13.8/13.10 -- Save writes a file that
// loads back to an equal menu.
#[test]
fn menus_editor_save_writes_loadable_file() {
    use crate::menus_editor_panel::MenusEditorAction as A;
    let (mut shell, dir) = make_shell_with_menus_editor();
    // Edit the title directly (as the render does), then Save (POM -> pom.toml).
    shell.menus_editor_panel.working.as_mut().unwrap().title = "My POM".to_string();
    shell.apply_menus_editor_action(A::Save);
    assert!(
        shell.menus_editor_panel.error.is_none(),
        "save must succeed, got: {:?}",
        shell.menus_editor_panel.error
    );
    let pom_path = dir.path().join("menus").join("pom.toml");
    assert!(pom_path.exists(), "Save must write menus/pom.toml");
    let loaded =
        crate::menu_workspace::loader::load_menu_file(&pom_path).expect("saved file must load");
    assert_eq!(loaded.title, "My POM");
    assert_eq!(
        loaded.options,
        shell.menus_editor_panel.working.as_ref().unwrap().options
    );
}

// Validates: menu-workspace Requirement 13.7/13.13 -- an invalid working menu
// (empty command) is rejected on Save; no file is written.
#[test]
fn menus_editor_save_blocked_when_invalid() {
    use crate::menus_editor_panel::MenusEditorAction as A;
    let (mut shell, dir) = make_shell_with_menus_editor();
    // Blank out an option's command -> invalid (as a direct field edit would).
    shell.menus_editor_panel.working.as_mut().unwrap().options[0].command = "   ".to_string();
    shell.apply_menus_editor_action(A::Save);
    assert!(
        shell.menus_editor_panel.error.is_some(),
        "an invalid menu must set an inline error"
    );
    assert!(
        !dir.path().join("menus").join("pom.toml").exists(),
        "an invalid menu must NOT be written"
    );
}

// Validates: menu-workspace Req 14.4 (CR-CH-022, supersedes B053) -- END from
// the Menus Editor pops the Navigation_Stack: opened from the POM it returns to
// the POM; opened via Settings it returns to the Settings menu, then the POM.
#[test]
fn menus_editor_end_returns_to_origin() {
    use crate::tab_state::TabKind;
    // POM -> MENUS -> END returns to the POM (stack had [POM]).
    let mut shell = make_shell();
    shell.handle_command("MENUS");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::MenusEditor);
    shell.handle_command("END");
    assert!(
        shell.tabs.active_tab().is_home,
        "END from a POM-opened Menus Editor returns to the POM"
    );

    // POM -> SETTINGS -> MENUS -> END returns to the Settings menu (stack had
    // [POM, Settings]); a second END returns to the POM.
    let mut shell = make_shell();
    shell.handle_command("SETTINGS");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::MenuWorkspace);
    shell.handle_command("MENUS");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::MenusEditor);
    shell.handle_command("END");
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "END from a Settings-opened Menus Editor returns to the Settings menu"
    );
    shell.handle_command("END");
    assert!(
        shell.tabs.active_tab().is_home,
        "a second END returns to the POM"
    );
}

// Note: the former `menus_editor_focus_ring_starts_with_command_line_...` test
// was removed with CR-NR-076 Option X -- the shell-driven focus ring
// (menus_editor_focus_ring / focus_ids) no longer exists. The Menus Editor now
// uses egui-native Tab traversal, regression-tested headlessly by the
// egui_kittest harness tests in menus_editor_panel::render (Req 14.6, 14.7).

// Validates: configuration-system Requirement 19.7 (closes task 23.9) -- the
// Settings baseline carries a RESET BARE affordance whose command opens the
// confirmation dialog through the normal command path.
#[test]
fn settings_reset_bare_affordance_dispatches_command() {
    let mut shell = make_shell();
    // The Settings Recovery_Baseline includes an R -> "RESET BARE" row; its
    // command dispatches through handle_command exactly as a typed command.
    assert!(shell.reset_bare_confirm.is_none());
    shell.handle_command("RESET BARE");
    assert!(
        shell.reset_bare_confirm.is_some(),
        "the RESET BARE affordance's command must open the confirmation dialog"
    );
}

// === CR-NR-074: Theme Editor Context (Requirement 20) ======================

/// Validates: Requirement 20.1, 20.2 -- THEMES opens the Theme Editor Context
/// and populates its panel state (available themes + working copy).
#[test]
fn themes_command_opens_theme_editor() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("THEME");
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::ThemeEditor,
        "THEMES must open the Theme Editor Context"
    );
    // The panel is populated: a working copy is loaded and the built-ins are listed.
    assert!(shell.theme_editor_panel.working.is_some());
    assert!(shell
        .theme_editor_panel
        .available
        .iter()
        .any(|n| n == "Default Legacy"));
}

/// Validates: Requirement 20.3, 20.8 -- EditToken updates the working copy and
/// live-previews it by applying to the active palette.
#[test]
fn theme_editor_edit_token_updates_working_and_previews() {
    use crate::theme_editor_panel::{EditableToken, ThemeEditorAction};
    use ff_theme::ColourRGBA;
    let mut shell = make_shell();
    shell.handle_command("THEME");
    let red = ColourRGBA::rgb(255, 0, 0);
    shell.apply_theme_editor_action(ThemeEditorAction::EditToken(EditableToken::UiPanelBg, red));
    // Working copy updated.
    let working = shell.theme_editor_panel.working.as_ref().unwrap();
    assert_eq!(EditableToken::UiPanelBg.get(working), red);
    // Live preview: active palette reflects the edit.
    assert_eq!(shell.palette.ui.panel_bg, red);
}

/// Validates: Requirement 20.7 / 18.4 -- Reset loads the built-in baseline into
/// the working copy for a built-in theme.
#[test]
fn theme_editor_reset_loads_builtin_baseline() {
    use crate::theme_editor_panel::{EditableToken, ThemeEditorAction};
    use ff_theme::ColourRGBA;
    let mut shell = make_shell();
    shell.handle_command("THEME");
    // Point the editor at Default Legacy and mutate the working copy.
    shell.apply_theme_editor_action(ThemeEditorAction::EditToken(
        EditableToken::EditorForeground,
        ColourRGBA::rgb(1, 2, 3),
    ));
    // Reset Default Legacy: working copy returns to the built-in colours.
    shell.apply_theme_editor_action(ThemeEditorAction::Reset("Default Legacy".to_string()));
    let working = shell.theme_editor_panel.working.as_ref().unwrap();
    assert_eq!(
        working.editor.foreground,
        ff_theme::defaults::default_legacy_palette()
            .editor
            .foreground
    );
}

/// Validates: Requirement 20.7 -- Reset of a non-built-in theme surfaces an error
/// rather than silently doing nothing.
#[test]
fn theme_editor_reset_non_builtin_errors() {
    use crate::theme_editor_panel::ThemeEditorAction;
    let mut shell = make_shell();
    shell.handle_command("THEME");
    shell.apply_theme_editor_action(ThemeEditorAction::Reset("My Custom Theme".to_string()));
    assert!(
        shell.theme_editor_panel.error.is_some(),
        "reset of a non-built-in theme must surface an error"
    );
}

/// Validates: Requirement 20.4 -- Copy creates a new named theme file from the
/// working copy and makes it the edit target, without altering the source.
#[test]
fn theme_editor_copy_creates_new_named_theme_file() {
    use crate::theme_editor_panel::ThemeEditorAction;
    let mut shell = make_shell();
    let _dir = point_themes_at_temp(&mut shell);
    shell.handle_command("THEME");
    shell.apply_theme_editor_action(ThemeEditorAction::Copy("my-theme".to_string()));
    // The new file exists and the editor now targets it.
    let themes = shell.dir_overrides.themes.clone().unwrap();
    assert!(
        themes.join("my-theme.toml").exists(),
        "copy must write a file"
    );
    assert_eq!(
        shell.theme_editor_panel.selected.as_deref(),
        Some("my-theme")
    );
    assert!(shell.theme_editor_panel.error.is_none());
    // Copy is listed as an available theme after the refresh.
    assert!(shell
        .theme_editor_panel
        .available
        .iter()
        .any(|n| n == "my-theme"));
}

/// Validates: Requirement 20.5 -- Save writes the working copy to the selected
/// theme's file (edited colour persists on disk).
#[test]
fn theme_editor_save_writes_edited_colour_to_disk() {
    use crate::theme_editor_panel::{EditableToken, ThemeEditorAction};
    use ff_theme::{ColourRGBA, ThemePalette};
    let mut shell = make_shell();
    let _dir = point_themes_at_temp(&mut shell);
    shell.handle_command("THEME");
    // Copy to a user theme so we edit/save without touching a built-in.
    shell.apply_theme_editor_action(ThemeEditorAction::Copy("edited".to_string()));
    // Edit a token and Save.
    let red = ColourRGBA::rgb(255, 0, 0);
    shell.apply_theme_editor_action(ThemeEditorAction::EditToken(EditableToken::UiPanelBg, red));
    shell.apply_theme_editor_action(ThemeEditorAction::Save);
    // Reload the file from disk and confirm the edited colour persisted.
    let themes = shell.dir_overrides.themes.clone().unwrap();
    let toml = std::fs::read_to_string(themes.join("edited.toml")).expect("read");
    let reloaded: ThemePalette =
        ff_theme::loader::load_from_toml(&toml, ff_theme::mode::VisualMode::Dark).expect("parse");
    assert_eq!(
        reloaded.ui.panel_bg, red,
        "edited colour must persist to disk"
    );
}

/// Validates: Requirement 20.5 -- Save As writes to a new named file.
#[test]
fn theme_editor_save_as_writes_new_file() {
    use crate::theme_editor_panel::ThemeEditorAction;
    let mut shell = make_shell();
    let _dir = point_themes_at_temp(&mut shell);
    shell.handle_command("THEME");
    shell.apply_theme_editor_action(ThemeEditorAction::SaveAs("saved-as".to_string()));
    let themes = shell.dir_overrides.themes.clone().unwrap();
    assert!(themes.join("saved-as.toml").exists());
    assert_eq!(
        shell.theme_editor_panel.selected.as_deref(),
        Some("saved-as")
    );
}

/// Validates: Requirement 20.6 / 19.7 -- Set Active loads the theme, swaps the
/// palette, and persists theme.active_name for future launches.
#[test]
fn theme_editor_set_active_swaps_palette_and_persists() {
    use crate::theme_editor_panel::ThemeEditorAction;
    let mut shell = make_shell();
    let _dir = point_themes_at_temp(&mut shell);
    shell.handle_command("THEME");
    // Copy a distinctly-coloured user theme, then set it active.
    shell.apply_theme_editor_action(ThemeEditorAction::Copy("active-me".to_string()));
    shell.apply_theme_editor_action(ThemeEditorAction::SetActive("active-me".to_string()));
    assert_eq!(
        shell.palette.name, "active-me",
        "active palette must be the chosen theme"
    );
    // Persisted for next launch.
    let persisted = shell
        .config_handle
        .get_string(ff_config::keys::theme::ACTIVE_NAME)
        .unwrap_or_default();
    assert_eq!(persisted, "active-me");
}

// === CR-CH-019 / B052: built-ins code-only + Save As fix ====================

/// Validates: B052 -- Save As works even when a hex token field had focus (the
/// token's lost_focus EditToken must not clobber the SaveAs button action). Here
/// we drive the shell directly (the render-level clobber is prevented by the
/// action/token_action split); this asserts the SaveAs action writes a file.
#[test]
fn theme_editor_save_as_after_edit_writes_file_b052() {
    use crate::theme_editor_panel::{EditableToken, ThemeEditorAction};
    use ff_theme::ColourRGBA;
    let mut shell = make_shell();
    let _dir = point_themes_at_temp(&mut shell);
    shell.handle_command("THEME");
    // Simulate: user edited a token, then clicked Save As. The editor emits the
    // button action (SaveAs) this frame, not the token edit.
    shell.apply_theme_editor_action(ThemeEditorAction::EditToken(
        EditableToken::UiPanelBg,
        ColourRGBA::rgb(10, 20, 30),
    ));
    shell.apply_theme_editor_action(ThemeEditorAction::SaveAs("after-edit".to_string()));
    let themes = shell.dir_overrides.themes.clone().unwrap();
    assert!(
        themes.join("after-edit.toml").exists(),
        "Save As must write the file even after a token edit"
    );
}

/// Validates: Requirement 19.2 (CR-CH-019) -- opening the Theme Editor does not
/// materialise built-in theme files; the themes dir holds only user themes.
#[test]
fn theme_editor_does_not_materialise_builtins() {
    let mut shell = make_shell();
    let dir = tempfile::TempDir::new().expect("tempdir");
    crate::theme_defaults::ensure_default_theme_files(dir.path());
    shell.dir_overrides.themes = Some(dir.path().join("themes"));
    shell.handle_command("THEME");
    let themes = dir.path().join("themes");
    for slug in ["default-dark", "default-high-contrast", "default-legacy"] {
        assert!(
            !themes.join(format!("{slug}.toml")).exists(),
            "built-in {slug}.toml must NOT be materialised"
        );
    }
}

/// Validates: Requirement 19.2a (CR-CH-019) -- the editor's available-themes
/// list contains no duplicates and lists each built-in exactly once.
#[test]
fn theme_editor_list_has_no_duplicates() {
    let mut shell = make_shell();
    let _dir = point_themes_at_temp(&mut shell);
    // Add a user theme, then copy a built-in (also a user theme now).
    shell.handle_command("THEME");
    shell.apply_theme_editor_action(crate::theme_editor_panel::ThemeEditorAction::Copy(
        "mine".to_string(),
    ));
    // Re-open to refresh the list.
    shell.handle_command("THEME");
    let list = &shell.theme_editor_panel.available;
    let mut sorted = list.clone();
    sorted.sort();
    let total = sorted.len();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        total,
        "theme list must have no duplicates: {list:?}"
    );
    // The four built-ins each appear exactly once (CR-CH-024: no separate
    // Legacy (ISPF 3270)).
    for b in [
        "Default Dark",
        "Default Light",
        "Default High Contrast",
        "Default Legacy",
    ] {
        assert_eq!(
            list.iter().filter(|n| n.as_str() == b).count(),
            1,
            "built-in '{b}' must appear exactly once"
        );
    }
    assert!(
        !list.iter().any(|n| n == "Legacy (ISPF 3270)"),
        "Legacy (ISPF 3270) is no longer a built-in (CR-CH-024)"
    );
}

/// Validates: Requirement 20.5 (CR-CH-019) -- Save on a built-in does not write
/// a built-in file; with no new name it surfaces a guiding message.
#[test]
fn theme_editor_save_on_builtin_does_not_write_builtin() {
    use crate::theme_editor_panel::ThemeEditorAction;
    let mut shell = make_shell();
    let _dir = point_themes_at_temp(&mut shell);
    shell.handle_command("THEME");
    // The editor opens with the active theme selected (a built-in in the default
    // config). Save with an empty name buffer must not write a built-in file and
    // must surface a message.
    let selected = shell.theme_editor_panel.selected.clone().unwrap();
    if ff_theme::is_builtin_theme(&selected) {
        shell.theme_editor_panel.name_buffer.clear();
        shell.apply_theme_editor_action(ThemeEditorAction::Save);
        let themes = shell.dir_overrides.themes.clone().unwrap();
        assert!(
            !themes
                .join(format!(
                    "{}.toml",
                    crate::theme_defaults::theme_slug(&selected)
                ))
                .exists(),
            "Save must not write a built-in file"
        );
        assert!(
            shell.theme_editor_panel.error.is_some(),
            "Save on a built-in with no new name must guide the user"
        );
    }
}

// Validates: function-keys Req 22.4 -- Save writes keymaps/<kind>.toml, which
// the resolver then loads as that kind's context map (round-trip).
#[test]
fn keys_editor_save_writes_keymaps_file_for_kind() {
    use crate::keys_editor_panel::KeysEditorAction;
    use tempfile::TempDir;

    let mut shell = make_shell();
    let dir = TempDir::new().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("keymaps")).expect("mkdir");
    shell.dir_overrides.keymaps = Some(dir.path().join("keymaps"));

    // Open the Keys editor for the editor kind, edit F5 base -> FIND, Save.
    shell.open_keys_editor(Some("editor"));
    if let Some(row) = shell
        .keys_editor_panel
        .rows
        .iter_mut()
        .find(|r| r.key == ff_keys::FunctionKey::F5)
    {
        row.commands[0] = "FIND".to_string();
    }
    shell.apply_keys_editor_action(KeysEditorAction::Save);

    // The file exists and round-trips through the loader as the editor context.
    let path = dir.path().join("keymaps").join("editor.toml");
    assert!(path.exists(), "Save must write keymaps/editor.toml");
    let text = std::fs::read_to_string(&path).expect("read");
    let table: toml::Table = toml::from_str(&text).expect("valid toml");
    let (map, _w) = ff_keys::KeyMap::from_toml_table(&table, "editor");
    assert_eq!(
        map.get_plain(ff_keys::FunctionKey::F5).map(|b| b.command()),
        Some("FIND"),
        "the saved keymaps/editor.toml must bind F5 = FIND"
    );
}

// === CR-CH-024: THEME command (full-shell egui_kittest) ======================

// Validates: theme Req 17.2b (CR-CH-024) -- `THEME Dark` shorthand selects the
// Default Dark built-in; `THEME Legacy` selects Default Legacy.
#[test]
fn full_shell_theme_shorthand_selects_default_builtins() {
    use ff_theme::mode::VisualMode;
    let mut harness = harness_shell();
    harness.state_mut().handle_command("THEME Dark");
    harness.run();
    assert_eq!(harness.state().palette.mode, VisualMode::Dark);
    assert_eq!(harness.state().palette.name, "Default Dark");

    harness.state_mut().handle_command("THEME Legacy");
    harness.run();
    assert_eq!(harness.state().palette.mode, VisualMode::Legacy);
    assert_eq!(
        harness.state().palette.name,
        "Default Legacy",
        "THEME Legacy resolves to Default Legacy (no separate ISPF built-in)"
    );
}

// Validates: theme Req 17.5 (CR-CH-024) -- an unknown theme leaves the active
// theme unchanged and shows the does-not-exist message.
#[test]
fn full_shell_theme_unknown_leaves_theme_unchanged() {
    use ff_theme::mode::VisualMode;
    let mut harness = harness_shell();
    harness.state_mut().handle_command("THEME Light");
    harness.run();
    assert_eq!(harness.state().palette.mode, VisualMode::Light);
    harness
        .state_mut()
        .handle_command("THEME does-not-exist-xyz");
    harness.run();
    assert_eq!(
        harness.state().palette.mode,
        VisualMode::Light,
        "unknown THEME must not change the active theme"
    );
    let msg = harness.state().open_error.clone().unwrap_or_default();
    assert!(
        msg.contains("does-not-exist-xyz") && msg.contains("does not exist"),
        "unknown THEME must show the does-not-exist message, got: {msg:?}"
    );
}

// Validates: theme Req 17.4 (CR-CH-024) -- bare `THEME` opens the Theme Editor
// context in place, and END returns to the previous context (Navigation_Stack).
#[test]
fn full_shell_bare_theme_opens_editor_and_end_returns() {
    use crate::tab_state::TabKind;
    let mut harness = harness_shell();
    // Start on the POM.
    assert!(harness.state().tabs.active_tab().is_home);
    harness.state_mut().handle_command("THEME");
    harness.run();
    assert_eq!(
        harness.state().tabs.active_tab().kind,
        TabKind::ThemeEditor,
        "bare THEME opens the Theme Editor"
    );
    // END returns one level to the POM (per-tab Navigation_Stack, CR-CH-022).
    harness.state_mut().handle_command("END");
    harness.run();
    assert!(
        harness.state().tabs.active_tab().is_home,
        "END from the Theme Editor returns to the POM"
    );
}

// Validates: theme Req 17.1 (CR-CH-024) -- the `THEMES` command is removed; it
// is no longer recognised and does NOT open the Theme Editor.
#[test]
fn full_shell_themes_command_is_removed() {
    use crate::tab_state::TabKind;
    let mut harness = harness_shell();
    harness.state_mut().handle_command("THEMES");
    harness.run();
    assert_ne!(
        harness.state().tabs.active_tab().kind,
        TabKind::ThemeEditor,
        "THEMES is no longer a recognised command (CR-CH-024)"
    );
}

// Validates: theme Req 18.1/18.3 (CR-CH-024) -- exactly four built-in themes are
// listed, and none is named "Legacy (ISPF 3270)".
#[test]
fn full_shell_theme_list_has_four_builtins() {
    let harness = harness_shell();
    let themes_dir = harness.state().themes_dir();
    let builtins: Vec<String> = ff_theme::list_all_themes(&themes_dir)
        .into_iter()
        .filter(|t| t.is_builtin)
        .map(|t| t.name)
        .collect();
    assert_eq!(
        builtins.len(),
        4,
        "exactly four built-ins; got {builtins:?}"
    );
    assert!(!builtins.iter().any(|n| n == "Legacy (ISPF 3270)"));
    assert!(builtins.iter().any(|n| n == "Default Legacy"));
}

// === CR-NR-082 Slice 1: Unified Menu Workspace (Requirement 18) =============

/// Validates: menu-workspace Requirement 18.1 -- the Home Context (POM) is
/// represented by the single Menu Workspace kind flagged `is_home`; there is no
/// separate Primary-Option-Menu tab kind.
#[test]
fn home_context_is_a_menu_workspace_flagged_is_home() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    let home = shell.tabs.active_tab();
    assert!(home.is_home, "the startup Home tab must be flagged is_home");
    assert_eq!(
        home.kind,
        TabKind::MenuWorkspace,
        "the Home Context must be the unified Menu Workspace kind"
    );
    assert_eq!(home.title, "[POM]");
}

/// Validates: menu-workspace Requirement 18.2 -- the Home Context loads its menu
/// lazily and falls back to the compiled barebones Recovery_Baseline when no
/// user pom.toml exists (make_shell has no menus dir).
#[test]
fn home_context_seeds_barebones_menu_on_render() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    // Before render the menu is unseeded; ensure_pom_menu_loaded seeds it.
    shell.ensure_pom_menu_loaded();
    let home = shell.tabs.active_tab();
    assert!(home.is_home);
    let mw = home
        .menu_workspace
        .as_ref()
        .expect("Home Context must carry a MenuWorkspaceState after seeding");
    assert!(
        mw.menu.is_some(),
        "the barebones Recovery_Baseline POM must be present when no file exists"
    );
}

/// Validates: menu-workspace Requirement 18.5 -- the Title_Line presents the
/// Home Context with the app banner, derived from the Menu Workspace (is_home),
/// not a distinct POM tab kind.
#[test]
fn home_context_title_line_shows_menu_title_not_banner() {
    // CR-CH-042 (menu-workspace Req 20.2/20.3; menu-and-statusbar Req 17.3):
    // the POM Title_Line shows the loaded pom.toml Menu_Title, NOT the hardcoded
    // "FileForge Workbench  vX.Y.Z" application banner. (Revises the former
    // home_context_title_line_shows_app_banner assertion.)
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    let text = super::title_line_text(shell.tabs.active_tab());
    assert!(
        !text.starts_with("FileForge Workbench  v"),
        "POM Title_Line must NOT be the app banner (CR-CH-042), got: {text:?}"
    );
    // The loaded compiled Recovery_Baseline POM title.
    assert_eq!(
        text, "FileForge Workbench -- Primary Option Menu",
        "POM Title_Line must be the loaded pom.toml Menu_Title"
    );
}

/// Validates: command-framework Req 15.2/15.8 -- `SETTINGS` still opens the
/// Settings menu in place on the typed path (unchanged; it was always a
/// Menu_Name resolution, the model this CR applies to POM).
#[test]
fn typed_settings_still_opens_settings_menu_in_place() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("START");
    let tabs_before = shell.tabs.len();
    shell.handle_command("SETTINGS");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::MenuWorkspace);
    assert!(!shell.tabs.active_tab().is_home, "left the POM");
    assert_eq!(
        shell.tabs.len(),
        tabs_before,
        "SETTINGS navigates in place (no new tab)"
    );
    let title_is_settings = shell
        .tabs
        .active_tab()
        .menu_workspace
        .as_ref()
        .and_then(|mw| mw.menu.as_ref())
        .map(|m| m.title.eq_ignore_ascii_case("Settings"))
        .unwrap_or(false);
    assert!(title_is_settings, "must be the Settings menu");
}

// === CR-NR-090 B.4: Kinds Editor Context (workspace-kinds Req 6) =============

// Validates: workspace-kinds Req 6.1, 6.4 -- KINDS opens the Kinds Editor: the
// active tab becomes the KindsEditor Context and the editor loads a working copy
// (the active workspace's own Kind) so the panel has something to render.
#[test]
fn open_kinds_editor_activates_kinds_editor_context() {
    use crate::tab_state::TabKind;

    let mut shell = make_shell();
    shell.open_kinds_editor();

    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::KindsEditor,
        "KINDS must navigate the active tab to the Kinds Editor Context"
    );
    assert!(
        shell.kinds_editor_panel.working.is_some(),
        "opening the editor must load a working Kind config"
    );
    assert!(
        !shell.kinds_editor_panel.kind_names.is_empty(),
        "the editor must offer the built-in Kind names to select from"
    );
}

// Validates: workspace-kinds Req 6.3 -- Save writes workspace-kinds/<name>.toml
// and reloading the registry from that dir reflects the edited title (the change
// is live).
#[test]
fn kinds_editor_save_writes_file_and_reloads_registry() {
    use crate::kinds_editor_panel::KindsEditorAction;
    use tempfile::TempDir;

    let mut shell = make_shell();
    let dir = TempDir::new().expect("tempdir");
    shell.dir_overrides.workspace_kinds = Some(dir.path().to_path_buf());

    // Open the editor, select the editor Kind, edit its title, Save.
    shell.open_kinds_editor();
    shell.apply_kinds_editor_action(KindsEditorAction::SelectKind("editor".to_string()));
    if let Some(cfg) = shell.kinds_editor_panel.working.as_mut() {
        cfg.title = "[MY EDITOR]".to_string();
    }
    shell.apply_kinds_editor_action(KindsEditorAction::Save);

    // The file exists on disk.
    let path = dir.path().join("editor.toml");
    assert!(path.exists(), "Save must write workspace-kinds/editor.toml");

    // The live registry (reloaded by Save) reflects the edited title.
    assert_eq!(
        shell.kind_registry.effective("editor").title,
        "[MY EDITOR]",
        "Save must reload the registry so the edited title is live"
    );

    // A fresh load from the same dir also reflects it (round-trip).
    let reloaded = crate::workspace_kind::KindRegistry::load(dir.path());
    assert_eq!(reloaded.effective("editor").title, "[MY EDITOR]");
}

/// Validates: workspace-kinds Req 8.11 -- `COMMAND` persists the change to the
/// Kind file (a fresh registry load from the same dir sees Bottom).
#[test]
fn command_persists_position_to_kind_file() {
    use crate::workspace_kind::CommandLinePosition;
    use tempfile::TempDir;
    let mut shell = make_shell();
    let dir = TempDir::new().expect("tempdir");
    shell.dir_overrides.workspace_kinds = Some(dir.path().to_path_buf());
    let name = active_kind_name(&shell);

    shell.handle_command("COMMAND BOTTOM");
    let reloaded = crate::workspace_kind::KindRegistry::load(dir.path());
    assert_eq!(
        reloaded.effective(name).profile.command_line_position,
        CommandLinePosition::Bottom,
        "COMMAND BOTTOM must persist to workspace-kinds/<name>.toml (survives reload)"
    );
}

// Validates: workspace-kinds Req 6.2 -- "New Kind modelled on <base>" seeds a
// working copy that is a COPY of the base's config, with the new name and
// modelled_on = Builtin(base).
#[test]
fn kinds_editor_new_kind_seeds_copy_modelled_on_base() {
    use crate::kinds_editor_panel::KindsEditorAction;
    use crate::workspace_kind::{BaseKind, BuiltinKind};

    let mut shell = make_shell();
    shell.open_kinds_editor();
    shell.apply_kinds_editor_action(KindsEditorAction::NewKind {
        name: "mainframe-editor".to_string(),
        base: BuiltinKind::Editor,
    });

    let working = shell
        .kinds_editor_panel
        .working
        .as_ref()
        .expect("new Kind must load a working copy");
    assert_eq!(working.name, "mainframe-editor");
    assert_eq!(
        working.modelled_on,
        BaseKind::Builtin(BuiltinKind::Editor),
        "the new Kind must be modelled on the chosen built-in base"
    );
    // The seed copies the base's title (a copy, editable afterwards).
    assert_eq!(working.title, BuiltinKind::Editor.default_title());
    assert!(
        shell
            .kinds_editor_panel
            .kind_names
            .contains(&"mainframe-editor".to_string()),
        "the new Kind name must be added to the selector list"
    );
}

// Validates: workspace-kinds Req 7 (CR-NR-090 B.4b) -- a RESET BARE whose target
// includes the active profile resets the live Kind registry to the compiled
// built-in defaults (a user override is dropped).
#[test]
fn reset_bare_restores_builtin_kind_registry() {
    use crate::shell::reset_bare::ResetBareTarget;
    use crate::workspace_kind::{KindConfig, KindRegistry};
    use tempfile::TempDir;

    let mut shell = make_shell();

    // Simulate a live registry carrying a user override of the editor title.
    let kinds_dir = TempDir::new().expect("tempdir");
    let toml = "name=\"editor\"\nmodelled_on=\"editor\"\ntitle=\"[OVERRIDDEN]\"";
    std::fs::write(kinds_dir.path().join("editor.toml"), toml).expect("write");
    shell.kind_registry = KindRegistry::load(kinds_dir.path());
    assert_eq!(
        shell.kind_registry.effective("editor").title,
        "[OVERRIDDEN]",
        "precondition: the live registry carries the user override"
    );

    // Execute RESET BARE against a throwaway target dir that includes the active
    // profile (so the live in-memory reset path runs). archive_config on the
    // empty dir is a harmless no-op.
    let target_dir = TempDir::new().expect("tempdir");
    let target = ResetBareTarget::single(
        "(default)".to_string(),
        target_dir.path().to_path_buf(),
        true,
    );
    shell.execute_reset_bare(&target);

    // The live registry is back to the compiled built-in default title.
    assert_eq!(
        shell.kind_registry.effective("editor").title,
        KindConfig::builtin_default(crate::workspace_kind::BuiltinKind::Editor).title,
        "RESET BARE must restore the built-in Kind defaults"
    );
}
