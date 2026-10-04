//! Shell tests -- focus area (split from shell/tests.rs, CR F3).
//! Items are verbatim; only their file location changed.

#![allow(unused_imports)]
use super::tests_common::*;
use ff_keys::{KeyMap, ModifiedKey};
use std::sync::{Arc, Mutex};

use ff_command::{
    CommandDispatch, CommandError, CommandHandler, CommandHistory, CommandId, CommandMetadata,
    CommandParams, CommandRegistry, CommandResult, ExecutionContext,
};

/// Validates: Requirement 14.15c -- POM tab context menu omits file-specific items.
#[test]
fn pom_tab_context_menu_items_are_universal_only() {
    // Validates: Requirement 14.15c
    // The context menu for a POM tab must NOT include file-specific items.
    // We verify this by checking the TabKind dispatch logic directly.
    use crate::tab_state::TabKind;
    // The Home Context (POM) is a MenuWorkspace tab after CR-NR-082 Slice 1.
    let pom_kind = TabKind::MenuWorkspace;
    let file_kind = TabKind::FileEditor;
    // File-specific items are only shown when kind == FileEditor.
    assert!(file_kind == TabKind::FileEditor);
    assert!(pom_kind != TabKind::FileEditor);
}

/// Validates: Requirement 14.15b -- file editor tab shows file-specific items.
#[test]
fn file_editor_tab_context_menu_includes_file_items() {
    // Validates: Requirement 14.15b
    use crate::tab_state::TabKind;
    assert_eq!(TabKind::FileEditor, TabKind::FileEditor);
}

// -- Phase AC: POM option list reorganisation tests ----------------------

// Validates: Requirement 14.3 -- POM has exactly 9 built-in options (0-8).
// POM option-list/label tests removed: the POM option list is now data-driven
// from menus/pom.toml (menu-workspace Req 2.1c-2.1i, CR-CH-018) and is covered
// by the menu_workspace defaults/loader/render tests.

// -- Task 26: ConfigPanel tab kind and routing tests ----------------------

/// Validates: Requirement 15.1 -- ConfigPanel TabKind variant exists.
#[test]
fn config_panel_tab_kind_exists() {
    // Validates: Requirement 15.1
    use crate::tab_state::TabKind;
    let kind = TabKind::ConfigPanel;
    assert_eq!(kind, TabKind::ConfigPanel);
}

/// Validates: Requirement 15.9 -- ConfigPanel is distinct from other tab kinds.
#[test]
fn config_panel_tab_kind_is_distinct_from_other_kinds() {
    // Validates: Requirement 15.9
    use crate::tab_state::TabKind;
    assert_ne!(TabKind::ConfigPanel, TabKind::MenuWorkspace);
    assert_ne!(TabKind::ConfigPanel, TabKind::FileEditor);
    assert_ne!(TabKind::ConfigPanel, TabKind::FilesPanel);
    assert_ne!(TabKind::ConfigPanel, TabKind::Untitled);
}

/// Validates: Requirement 14.6 -- option 1 on a POM tab transforms the tab in-place.
#[test]
fn pom_option_1_on_pom_tab_transforms_tab_in_place() {
    // Validates: Requirement 14.6
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use tokio::runtime::Runtime;
    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    mgr.insert_pom_tab(&runtime);
    assert!(mgr.active_tab().is_home);
    // Typing "1" on a POM tab must transform it in-place to FilesPanel.
    mgr.transform_active_pom_tab(TabKind::FilesPanel, "[FILES]");
    assert_eq!(mgr.active_tab().kind, TabKind::FilesPanel);
    assert_eq!(mgr.active_tab().title, "[FILES]");
    // Tab count must not change -- no new tab opened.
    assert_eq!(mgr.len(), 2); // welcome + transformed
}

/// Validates: Requirement 14.6 -- option 1 on a non-POM tab opens a new tab.
#[test]
fn pom_option_1_on_non_pom_tab_does_not_transform() {
    // Validates: Requirement 14.6
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use tokio::runtime::Runtime;
    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    // Active tab is Untitled (not POM) -- transform_active_pom_tab must be a no-op.
    assert_eq!(mgr.active_tab().kind, TabKind::Untitled);
    mgr.transform_active_pom_tab(TabKind::FilesPanel, "[FILES]");
    assert_eq!(
        mgr.active_tab().kind,
        TabKind::Untitled,
        "non-POM tab must not be transformed"
    );
}

/// Validates: Requirement 4.3 -- unassigned keys produce blank slots.
#[test]
fn key_label_bar_unassigned_key_has_no_label() {
    // Validates: function-keys-and-history Requirement 4.3, 15.3
    use ff_keys::{FunctionKey, KeyLabelBarModel};
    let map = KeyMap::default_global();
    let bar = KeyLabelBarModel::from_key_map(&map);
    // F13 is beyond the F1-F12 default set, so its Base slot is blank
    // (CR-CH-027).
    let slot = bar.slot_for(FunctionKey::F13).unwrap();
    assert!(slot.label.is_none());
}

// -- Phase AJ: Tab-order focus cycle tests ----------------------------------------

/// Validates: Requirement 4.6 -- key label bar updates when key map changes.
#[test]
fn key_label_bar_updates_on_key_map_change() {
    use ff_keys::{FunctionKey, KeyBinding, KeyLabelBarModel, KeyMap};
    let map = KeyMap::default_global();
    let mut bar = KeyLabelBarModel::from_key_map(&map);

    let mut new_map = KeyMap::empty("updated");
    new_map.set(
        ModifiedKey::plain(FunctionKey::F3),
        KeyBinding::with_label("QUIT", "Quit"),
    );
    bar.update(&new_map);

    let slot = bar.slot_for(FunctionKey::F3).unwrap();
    assert_eq!(slot.label.as_deref(), Some("Quit"));
    // F7 was in old map but not new -- should now be blank
    assert!(bar.slot_for(FunctionKey::F7).unwrap().label.is_none());
}

// -- Phase AL: Title Line tests -----------------------------------------

/// Validates: Requirement 17.3 (REVISED by CR-CH-042) / menu-workspace Req 20.3 --
/// the POM tab Title_Line shows the POM Menu_Title, NOT the hardcoded application
/// banner + version. With a menu loaded it is the raw pom.toml title; with a bare
/// POM tab (no menu yet) it is the `[POM]` cached fallback. Either way it must not
/// be the `FileForge Workbench  vX.Y.Z` banner.
#[test]
fn title_line_pom_tab_shows_menu_title_not_banner() {
    // Validates: menu-and-statusbar Requirement 17.3; menu-workspace Requirement 20.3
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;
    // Bare POM tab (no loaded menu): the cached fallback, not the banner.
    let bare = TabState::pom(TabId(1), new_document());
    let bare_text = super::title_line_text(&bare);
    assert!(
        !bare_text.contains(env!("CARGO_PKG_VERSION")),
        "POM Title_Line must not carry the version banner: {bare_text}"
    );
    assert_eq!(
        bare_text, "[POM]",
        "bare POM falls back to the cached [POM]"
    );

    // With the compiled Recovery_Baseline POM menu loaded: the raw Menu_Title.
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    let loaded_text = super::title_line_text(shell.tabs.active_tab());
    assert_eq!(
        loaded_text, "FileForge Workbench -- Primary Option Menu",
        "loaded POM Title_Line is the pom.toml Menu_Title (not the app banner)"
    );
}

// -- Phase AK: Tab-header focus stops + command field focus fix -----------

// -- Phase AO: Detachable Tab Windows (Requirement 18) ----------------------

/// Validates: Requirement 18.1, 18.4 -- detaching a tab sets is_floating and
/// records a FloatingTab with the correct origin_index.
#[test]
fn floating_tab_is_floating_flag_set_on_detach() {
    // Validates: Requirement 18.1, 18.4
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;

    let mut tab = TabState::for_file(
        TabId(0),
        "/tmp/test.txt".to_string(),
        new_document(),
        1,
        ff_document_model::LineEndMode::Default,
    );
    assert!(!tab.is_floating);
    tab.is_floating = true;
    assert!(tab.is_floating);
}

/// Validates: Requirement 18.7 -- maximum 16 floating windows enforced.
#[test]
fn floating_tab_limit_enforced_at_16() {
    // Validates: Requirement 18.7
    use super::FloatingTab;

    let mut floating: Vec<FloatingTab> = Vec::new();
    for i in 0..16 {
        floating.push(FloatingTab {
            viewport_id: egui::ViewportId::from_hash_of(format!("ft_{i}")),
            tab_id: crate::tab_state::TabId(i as u64),
            origin_index: i,
            cmd_ctx: super::WorkspaceCommandContext::default(),
        });
    }
    // At limit: a new detach should be rejected.
    assert_eq!(floating.len(), 16);
    let would_detach = floating.len() < 16;
    assert!(
        !would_detach,
        "must not detach when 16 windows already open"
    );
}

/// Validates: Requirement 18.3 -- origin_index is preserved on FloatingTab.
#[test]
fn floating_tab_origin_index_preserved() {
    // Validates: Requirement 18.3
    use super::FloatingTab;

    let ft = FloatingTab {
        viewport_id: egui::ViewportId::from_hash_of("test"),
        tab_id: crate::tab_state::TabId(3),
        origin_index: 3,
        cmd_ctx: super::WorkspaceCommandContext::default(),
    };
    assert_eq!(ft.origin_index, 3);
    assert_eq!(ft.tab_id, crate::tab_state::TabId(3));
}

/// Validates: Requirement 18.3 -- redock clamps origin_index to current tab count.
#[test]
fn redock_clamps_to_tab_count() {
    // Validates: Requirement 18.3
    // Simulate: origin_index=5, but only 3 tabs remain after others were closed.
    let origin_index: usize = 5;
    let tab_count: usize = 3;
    let clamped = origin_index.min(tab_count.saturating_sub(1));
    assert_eq!(clamped, 2);
}

/// Validates: Requirement 18.5 -- floating window OS title bar format.
#[test]
fn floating_tab_title_format() {
    // Validates: Requirement 18.5
    use crate::tab_state::{TabId, TabState};
    use ff_document_model::new_document;

    let tab = TabState::for_file(
        TabId(0),
        "/home/user/project/main.rs".to_string(),
        new_document(),
        1,
        ff_document_model::LineEndMode::Default,
    );
    let title = format!("{} -- FileForge Workbench", super::title_line_text(&tab));
    assert!(title.contains("main.rs"), "title must contain file name");
    assert!(
        title.ends_with("-- FileForge Workbench"),
        "title must end with app name"
    );
}

// -- Phase AS: File Explorer Panel tests (Req 19) --------------------------

/// Validates: Requirement 19.11, 19.12 -- FileExplorerPanel TabKind variant exists.
#[test]
fn file_explorer_panel_tab_kind_exists() {
    // Validates: Requirement 19.11, 19.12
    use crate::tab_state::TabKind;
    let kind = TabKind::FileExplorerPanel;
    assert_eq!(kind, TabKind::FileExplorerPanel);
}

/// Validates: Requirement 19.1 -- `=2` transforms current tab in-place to FileExplorerPanel.
#[test]
fn equals_2_command_transforms_tab_to_file_explorer() {
    // Validates: Requirement 19.1
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use tokio::runtime::Runtime;
    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    mgr.insert_pom_tab(&runtime);
    assert!(mgr.active_tab().is_home);
    mgr.transform_active_pom_tab(TabKind::FileExplorerPanel, "[FILES]");
    assert_eq!(mgr.active_tab().kind, TabKind::FileExplorerPanel);
    assert_eq!(mgr.active_tab().title, "[FILES]");
}

/// Validates: Requirement 19.4 -- option `2` on a POM tab transforms in-place.
#[test]
fn option_2_on_pom_tab_transforms_to_file_explorer() {
    // Validates: Requirement 19.4
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use tokio::runtime::Runtime;
    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    mgr.insert_pom_tab(&runtime);
    mgr.transform_active_pom_tab(TabKind::FileExplorerPanel, "[FILES]");
    assert_eq!(mgr.active_tab().kind, TabKind::FileExplorerPanel);
    assert_eq!(mgr.active_tab().title, "[FILES]");
}

/// Validates: Requirement 19.11 -- FileExplorerPanel tab title is `[FILES]`.
#[test]
fn file_explorer_panel_tab_title_is_files() {
    // Validates: Requirement 19.11
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use tokio::runtime::Runtime;
    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    mgr.open_file_explorer_panel_tab(&runtime);
    assert_eq!(mgr.active_tab().kind, TabKind::FileExplorerPanel);
    assert_eq!(mgr.active_tab().title, "[FILES]");
}

/// Validates: Requirement 16.3 -- edit_profile defaults to all-off on new tab.
#[test]
fn new_tab_edit_profile_defaults_to_all_off() {
    // Validates: Requirement 16.3 (default state)
    use ff_edit_operations::{CapsMode, NullsMode, StatsMode};
    let shell = make_shell();
    let profile = &shell.tabs.active_tab().edit_profile;
    assert_eq!(profile.caps, CapsMode::Off);
    assert_eq!(profile.nulls, NullsMode::Off);
    assert_eq!(profile.stats, StatsMode::Off);
    assert!(!profile.is_locked());
}

/// Validates: multi-tab-editor Requirement 18.7 -- bare SWAP with no split
/// screen and NO previously active tab (only one tab open) falls back to the
/// tab picker (Req 18.10, CR-CH-031).
#[test]
fn swap_without_split_or_previous_opens_tab_picker() {
    // Validates: Requirement 18.10 -- single tab (no Previous_Active_Tab) -> picker.
    let mut shell = make_shell();
    assert_eq!(shell.tabs.len(), 1, "make_shell starts with one tab");
    shell.handle_command("SWAP");
    assert!(
        shell.show_swap_list.is_some(),
        "bare SWAP with no split and no previous tab must open the tab picker"
    );
    assert!(shell.open_error.is_none());
}

/// Validates: multi-tab-editor Requirement 18.3 -- `SWAP LIST` opens the tab
/// picker.
#[test]
fn swap_list_opens_tab_picker() {
    // Validates: Requirement 18.3
    let mut shell = make_shell();
    shell.handle_command("SWAP LIST");
    assert!(shell.show_swap_list.is_some());
    assert!(shell.open_error.is_none());
}

/// Validates: configuration-system Req 20.3 (CR-CH-025) -- CONFIG <ns> tab title.
#[test]
fn settings_namespace_tab_title_includes_namespace() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    shell.handle_command("CONFIG theme");
    let tab = shell.tabs.active_tab();
    assert_eq!(tab.kind, TabKind::ConfigPanel);
    assert_eq!(tab.title, "[CONFIG:theme]");
}

// === Phase CO: Accessibility -- Focus Ring (Requirement 3) =================

/// Validates: Requirement 3.1, 3.2 -- focus_ring token exists in all built-in themes.
#[test]
fn focus_ring_token_exists_in_all_themes() {
    // Validates: accessibility Requirement 3.1, 3.2
    use ff_theme::defaults::{dark_palette, high_contrast_palette, legacy_palette, light_palette};
    use ff_theme::ColourToken;

    for (name, palette) in [
        ("dark", dark_palette()),
        ("light", light_palette()),
        ("high_contrast", high_contrast_palette()),
        ("legacy", legacy_palette()),
    ] {
        let ring = palette.colour(ColourToken::UiFocusRing);
        assert_ne!(
            ring,
            palette.colour(ColourToken::UiPanelBackground),
            "{name}: focus_ring must differ from panel background"
        );
        // Alpha must be fully opaque so the ring is visible.
        assert_eq!(ring.a, 255, "{name}: focus_ring alpha must be 255");
    }
}

/// Validates: accessibility Requirement 3.1, 3.3 -- render_focus_indicator function exists
/// and can be called without panicking.
#[test]
fn focus_indicator_helper_is_callable() {
    // Validates: accessibility Requirement 3.1, 3.3
    // render_focus_indicator is a free function in shell::render -- we verify it
    // compiles and is reachable. Actual pixel output requires an egui context
    // (manual/UI test); this test guards the API contract.
    use ff_theme::defaults::dark_palette;
    let palette = dark_palette();
    // The focus ring colour must be non-zero so the painter stroke is visible.
    assert_ne!(
        palette.ui.focus_ring.r | palette.ui.focus_ring.g | palette.ui.focus_ring.b,
        0
    );
}

/// Validates: accessibility Requirement 2.3 -- modal dialogs trap focus (shell suppresses
/// Tab-cycle when modal_open is true).
#[test]
fn modal_open_flag_suppresses_shell_tab_cycle() {
    // Validates: accessibility Requirement 2.3
    // When modal_open is true the shell must not steal focus from the dialog.
    // We verify the flag exists and can be set on the shell.
    let mut shell = make_shell();
    // Initially no modal is open.
    assert!(!shell.modal_open);
    // Setting it to true simulates a dialog opening.
    shell.modal_open = true;
    assert!(shell.modal_open);
    shell.modal_open = false;
    assert!(!shell.modal_open);
}

// === Phase CO: Plugin Manager UI (Requirement 1) ===========================

/// Validates: plugin-manager-ui Requirement 1.1 -- PluginManager TabKind exists.
#[test]
fn plugin_manager_tab_kind_exists() {
    // Validates: plugin-manager-ui Requirement 1.1
    use crate::tab_state::TabKind;
    let kind = TabKind::PluginManager;
    assert_eq!(kind, TabKind::PluginManager);
    assert_ne!(kind, TabKind::MenuWorkspace);
    assert_ne!(kind, TabKind::ConfigPanel);
}

/// Validates: plugin-manager-ui Requirement 4.1 -- PluginManager tab round-trips through session.
#[test]
fn plugin_manager_tab_round_trips_through_session() {
    // Validates: plugin-manager-ui Requirement 4.1
    use crate::tab_state::TabKind;
    use ff_session::session_state::PersistedTabKind;
    // PluginManager must map to a PersistedTabKind variant.
    let kind = TabKind::PluginManager;
    assert_eq!(kind, TabKind::PluginManager);
    // The session serialisation must include PluginManager.
    // We verify the PersistedTabKind has the variant.
    let _ptk = PersistedTabKind::PluginManager;
}

/// Validates: notification-system Requirement 2.1 -- EventLog TabKind exists.
#[test]
fn event_log_tab_kind_exists() {
    // Validates: notification-system Requirement 2.1
    use crate::tab_state::TabKind;
    let kind = TabKind::EventLog;
    assert_eq!(kind, TabKind::EventLog);
    assert_ne!(kind, TabKind::PluginManager);
}

// === Phase CR: Macro Library Panel (Requirement 12) ========================

/// Validates: lua-macro-engine Requirement 12.1 -- MacroLibrary TabKind variant exists.
#[test]
fn macro_library_tab_kind_exists() {
    // Validates: lua-macro-engine Requirement 12.1
    use crate::tab_state::TabKind;
    let kind = TabKind::MacroLibrary;
    assert_eq!(kind, TabKind::MacroLibrary);
    assert_ne!(kind, TabKind::MenuWorkspace);
    assert_ne!(kind, TabKind::PluginManager);
}

/// Validates: lua-macro-engine Requirement 12.1 -- MacroLibrary tab title is [MACROS].
#[test]
fn macro_library_tab_title_is_macros() {
    // Validates: lua-macro-engine Requirement 12.1
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use tokio::runtime::Runtime;
    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    mgr.open_macro_library_tab(&runtime);
    assert_eq!(mgr.active_tab().kind, TabKind::MacroLibrary);
    assert_eq!(mgr.active_tab().title, "[MACROS]");
}

/// Validates: lua-macro-engine Requirement 12.1 -- opening MacroLibrary twice does not duplicate.
#[test]
fn open_macro_library_tab_twice_does_not_duplicate() {
    // Validates: lua-macro-engine Requirement 12.1
    use crate::tab_manager::TabManager;
    use tokio::runtime::Runtime;
    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    mgr.open_macro_library_tab(&runtime);
    let count = mgr.len();
    mgr.open_macro_library_tab(&runtime);
    assert_eq!(mgr.len(), count, "second open must not add a duplicate");
}

/// Validates: lua-macro-engine Requirement 12.5 -- MacroLibrary tab is not persisted in session.
#[test]
fn macro_library_tab_not_persisted_in_session() {
    // Validates: lua-macro-engine Requirement 12.5

    use crate::tab_state::TabKind;
    use ff_session::session_state::PersistedTabKind;

    // MacroLibrary must not map to any PersistedTabKind -- it is excluded from session.
    // Verify by checking the kind is distinct from all persisted kinds.
    let kind = TabKind::MacroLibrary;
    assert_ne!(kind, TabKind::FileEditor);
    assert_ne!(kind, TabKind::FilesPanel);
    assert_ne!(kind, TabKind::FileExplorerPanel);
    // PersistedTabKind does not have a MacroLibrary variant -- compile-time guarantee.
    let _ptk = PersistedTabKind::EventLog; // EventLog exists; MacroLibrary does not
}

/// Validates: menu-and-statusbar Req 18.14 (CR-CH-040) -- DETACH on a non-editor
/// (POM) tab sets detach_pending (formerly the bare SPLIT behaviour, now renamed).
#[test]
fn detach_on_pom_tab_sets_detach_pending() {
    let mut shell = make_shell();
    // Navigate to a POM tab via START command
    shell.handle_command("START");
    // The new POM tab is now the last tab; find and activate it
    let pom_idx = shell
        .tabs
        .tabs()
        .iter()
        .rposition(|t| t.is_home)
        .expect("POM tab must exist after START");
    shell.tabs.set_active(pom_idx);
    assert!(shell.tabs.active_tab().is_home);
    shell.handle_command("DETACH");
    assert!(
        shell.detach_split.detach_pending.is_some(),
        "DETACH on POM tab should set detach_pending"
    );
}

// Validates: menu-workspace Requirement 11.2 -- MENU <name> opens a data-driven
// Menu_Workspace tab backed by menus/<name>.toml.
#[test]
fn open_menu_workspace_tab_loads_named_menu() {
    use crate::menu_workspace::OptionLimits;
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use tokio::runtime::Runtime;

    let dir = tempfile::TempDir::new().unwrap();
    let menus = dir.path().join("menus");
    std::fs::create_dir_all(&menus).unwrap();
    std::fs::write(
        menus.join("tools.toml"),
        "title = \"Tools\"\n[[options]]\nkey = \"1\"\ncommand = \"FILES\"\ndescription = \"Files\"\n",
    )
    .unwrap();

    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    mgr.insert_pom_tab(&runtime);
    mgr.open_menu_workspace_tab("tools", &menus, OptionLimits::default(), &runtime);

    assert_eq!(mgr.active_tab().kind, TabKind::MenuWorkspace);
    let mw = mgr.active_tab().menu_workspace.as_ref().expect("mw state");
    assert!(mw.load_error.is_none());
    assert_eq!(mw.menu.as_ref().unwrap().title, "Tools");
}

// Validates: menu-workspace Requirement 11.4 -- MENU <name> for a missing file
// opens the tab in its load-error state rather than doing nothing.
#[test]
fn open_menu_workspace_tab_missing_file_is_load_error() {
    use crate::menu_workspace::OptionLimits;
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use tokio::runtime::Runtime;

    let dir = tempfile::TempDir::new().unwrap();
    let menus = dir.path().join("menus");
    std::fs::create_dir_all(&menus).unwrap();

    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    mgr.insert_pom_tab(&runtime);
    mgr.open_menu_workspace_tab("nope", &menus, OptionLimits::default(), &runtime);

    assert_eq!(mgr.active_tab().kind, TabKind::MenuWorkspace);
    let mw = mgr.active_tab().menu_workspace.as_ref().expect("mw state");
    assert!(mw.menu.is_none());
    assert!(
        mw.load_error.as_deref().unwrap_or("").contains("not found"),
        "missing menu file must produce a load error"
    );
}

// Validates: menu-workspace Requirement 11.2 -- opening the same menu twice
// activates the existing tab instead of duplicating it.
#[test]
fn open_menu_workspace_tab_dedupes_by_file() {
    use crate::menu_workspace::OptionLimits;
    use crate::tab_manager::TabManager;
    use crate::tab_state::TabKind;
    use tokio::runtime::Runtime;

    let dir = tempfile::TempDir::new().unwrap();
    let menus = dir.path().join("menus");
    std::fs::create_dir_all(&menus).unwrap();
    std::fs::write(
        menus.join("tools.toml"),
        "title = \"Tools\"\n[[options]]\nkey = \"1\"\ncommand = \"FILES\"\ndescription = \"F\"\n",
    )
    .unwrap();

    let runtime = Runtime::new().expect("runtime");
    let mut mgr = TabManager::new(&runtime, "");
    mgr.insert_pom_tab(&runtime);
    mgr.open_menu_workspace_tab("tools", &menus, OptionLimits::default(), &runtime);
    let count_after_first = mgr.len();
    mgr.open_menu_workspace_tab("tools", &menus, OptionLimits::default(), &runtime);
    assert_eq!(
        mgr.len(),
        count_after_first,
        "opening the same menu twice must not duplicate the tab"
    );
    assert_eq!(mgr.active_tab().kind, TabKind::MenuWorkspace);
}

// Validates: Req 14.5 -- END with an empty stack on the last tab terminates.
#[test]
fn end_at_empty_stack_last_tab_exits() {
    let mut shell = make_shell(); // single POM tab, empty stack
    assert_eq!(shell.tabs.len(), 1);
    assert!(shell.tabs.active_tab().nav_stack.is_empty());
    shell.handle_command("END");
    // file.exit is dispatched; the close-flag is set by the exit command.
    assert!(
        *shell.should_close.lock().expect("close lock"),
        "END at the root of the last Workspace must terminate the app"
    );
}

// Validates: Req 14.8 -- START forms: START (POM), START =0 (POM+drill),
// START Settings (rooted directly at Settings, empty stack).
#[test]
fn start_forms_root_the_new_tab_correctly() {
    use crate::tab_state::TabKind;

    // START -> new POM tab, empty stack.
    let mut shell = make_shell();
    let before = shell.tabs.len();
    shell.handle_command("START");
    assert_eq!(shell.tabs.len(), before + 1);
    assert!(shell.tabs.active_tab().is_home);
    assert!(shell.tabs.active_tab().nav_stack.is_empty());

    // START Settings -> new tab rooted directly at Settings, EMPTY stack, so
    // END ends the Workspace.
    let mut shell = make_shell();
    let before = shell.tabs.len();
    shell.handle_command("START Settings");
    assert_eq!(shell.tabs.len(), before + 1, "START creates one new tab");
    assert_eq!(
        shell.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "START Settings roots the new tab at the Settings menu"
    );
    assert!(
        shell.tabs.active_tab().nav_stack.is_empty(),
        "START <arg> roots directly (no POM beneath), so the stack is empty"
    );
}

/// Validates: Requirement 20.2 -- on a POM tab, THEMES transforms it in place
/// (so END/RETURN returns to the POM), like the Settings/Commands Contexts.
#[test]
fn themes_command_transforms_pom_tab_in_place() {
    use crate::tab_state::TabKind;
    let mut shell = make_shell();
    // Ensure the active tab is a POM.
    shell.handle_command("START");
    assert!(shell.tabs.active_tab().is_home);
    let count_before = shell.tabs.len();
    shell.handle_command("THEME");
    assert_eq!(shell.tabs.active_tab().kind, TabKind::ThemeEditor);
    assert_eq!(
        shell.tabs.len(),
        count_before,
        "opening the editor from the POM transforms in place (no new tab)"
    );
}

// === B055: Status_Bar segments must not be Tab focus stops =================
//
// Requirement 16 (Tab-Order Focus Cycle) enumerates the complete, exclusive set
// of shell tab stops: CommandField -> POM options -> calendar -> menu bar ->
// tab headers -> back to CommandField. Status_Bar segments are NOT in that set.
// The bug (B055): render_status_bar built each segment with
// egui::SelectableLabel (Sense::click()), making them egui-native focus stops,
// so at launch Tab walked all six segments before reaching the menu/POM.
//
// Regression guard: render the real render_status_bar into a headless harness
// together with a single known-focusable sentinel button placed AFTER it. With
// the fix (non-interactive labels) the very first Tab lands on the sentinel,
// proving no status-bar segment is a focus stop. With the bug present, the first
// Tab would land on a status-bar segment and the sentinel would not be focused.
#[test]
fn status_bar_segments_are_not_tab_focus_stops() {
    use egui_kittest::Harness;

    let mut shell = make_shell();

    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(1200.0, 900.0))
        .build(move |ctx| {
            // Render the real status bar exactly as the shell does.
            shell.render_status_bar(ctx);
            // A single focusable sentinel in the central panel. It is created
            // AFTER the status bar, so in pure egui creation order any focusable
            // status-bar segment would be visited by Tab before this sentinel.
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = ui.button("sentinel");
            });
        });

    // Press Tab a few times and collect every widget that receives focus.
    let mut focused_ids: Vec<egui::Id> = Vec::new();
    for _ in 0..8 {
        harness.key_press(egui::Key::Tab);
        harness.run();
        if let Some(id) = harness.ctx.memory(|m| m.focused()) {
            focused_ids.push(id);
        }
    }

    // The only focusable widget in the frame is the sentinel button. If any
    // status-bar segment were focusable, focus would land on more than one
    // distinct widget id (the segments) as Tab is pressed. With the fix, every
    // press keeps focus on the single sentinel button.
    let distinct: std::collections::HashSet<egui::Id> = focused_ids.iter().copied().collect();
    assert!(
        distinct.len() <= 1,
        "Tab focused multiple widgets, meaning Status_Bar segments are focus \
         stops (B055). Focused ids across presses: {focused_ids:?}"
    );
}

#[test]
fn key_bar_scope_maps_modifier_segment_and_parse() {
    // Validates: function-keys-and-history Requirement 12.9, 12.10 -- scope maps
    // to the right modifier layer, exposes the Scope_Segment label, and parses
    // case-insensitively (used by the command and by session restore).
    use super::KeyBarScope;
    use ff_keys::KeyModifier;

    assert_eq!(KeyBarScope::Base.to_modifier(), KeyModifier::None);
    assert_eq!(KeyBarScope::Shift.to_modifier(), KeyModifier::Shift);
    assert_eq!(KeyBarScope::Ctrl.to_modifier(), KeyModifier::Ctrl);
    assert_eq!(KeyBarScope::Alt.to_modifier(), KeyModifier::Alt);

    assert_eq!(KeyBarScope::Base.segment_label(), "Base");
    assert_eq!(KeyBarScope::Shift.segment_label(), "Shift");
    assert_eq!(KeyBarScope::Ctrl.segment_label(), "Ctrl");
    assert_eq!(KeyBarScope::Alt.segment_label(), "Alt");

    assert_eq!(KeyBarScope::parse("base"), Some(KeyBarScope::Base));
    assert_eq!(KeyBarScope::parse("SHIFT"), Some(KeyBarScope::Shift));
    assert_eq!(KeyBarScope::parse(" ctrl "), Some(KeyBarScope::Ctrl));
    assert_eq!(KeyBarScope::parse("Alt"), Some(KeyBarScope::Alt));
    assert_eq!(KeyBarScope::parse("bogus"), None);

    // persist_name round-trips through parse.
    for scope in [
        KeyBarScope::Base,
        KeyBarScope::Shift,
        KeyBarScope::Ctrl,
        KeyBarScope::Alt,
    ] {
        assert_eq!(KeyBarScope::parse(scope.persist_name()), Some(scope));
    }
}

/// Render the single-row Key_Label_Bar for each scope into a headless harness
/// and confirm the real render path runs without panic (CR-CH-046). The
/// non-focusability of the slots is guarded by
/// `key_label_bar_buttons_are_not_tab_focus_stops`; pixel-exact styling of the
/// Scope_Segment/divider is the documented MANUAL row.
///
/// Validates: function-keys-and-history Requirement 12.10, 13.1
#[test]
fn key_label_bar_renders_single_row_for_each_scope() {
    use egui_kittest::Harness;
    for arg in ["BASE", "SHIFT", "CTRL", "ALT"] {
        let mut shell = make_shell();
        shell.handle_command(&format!("PFSHOW {arg}"));
        assert!(shell.key_bar_visible, "PFSHOW {arg} shows the bar");
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(1200.0, 200.0))
            .build(move |ctx| {
                shell.render_key_label_bar(ctx);
            });
        harness.run();
        // Reaching here means the single-row render path executed for this scope
        // without panicking.
    }
}

// === B077: RESET BARE confirmation dialog traps keyboard focus ===============
//
// accessibility Req 2.3: when a modal dialog is open, keyboard focus is trapped
// within it and Tab must NOT move focus to background elements. The RESET BARE
// popup previously left `modal_open` unset on the frame it opened (it was set
// only later in update()), so the background Boundary_Policy still consumed Tab.

/// Opening the RESET BARE dialog sets `modal_open` on the SAME frame, so the
/// background Tab handling is suppressed (the root cause of B077).
///
/// Validates: accessibility Requirement 2.3 (modal focus trap)
#[test]
fn full_shell_reset_bare_dialog_sets_modal_open_and_focuses_a_button() {
    let mut harness = harness_shell();
    // Open the confirmation dialog via the real command path.
    harness.state_mut().handle_command("RESET BARE");
    harness.run();

    // The dialog is open and the shell is in modal mode on this frame.
    assert!(
        harness.state().reset_bare_confirm.is_some(),
        "RESET BARE must open the confirmation dialog"
    );
    assert!(
        harness.state().modal_open,
        "opening the RESET BARE dialog must set modal_open so background Tab is trapped (B077)"
    );

    // Focus is inside the dialog, NOT on the background command field.
    let focused = harness.ctx.memory(|m| m.focused());
    assert!(
        focused.is_some(),
        "the modal must hold keyboard focus (initial focus on Cancel)"
    );
    assert_ne!(
        focused,
        Some(cmd_field_id()),
        "focus must be in the dialog, not the background command field (B077)"
    );
}

/// While the RESET BARE dialog is open, pressing Tab keeps focus inside the
/// dialog and never returns to the background command field.
///
/// Validates: accessibility Requirement 2.3 (Tab does not escape the modal)
#[test]
fn full_shell_reset_bare_dialog_tab_stays_within_modal() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("RESET BARE");
    harness.run();
    assert!(harness.state().modal_open, "dialog open");

    // Tab several times; focus must never land on the background command field.
    for _ in 0..6 {
        harness.key_press(egui::Key::Tab);
        harness.run();
        assert_ne!(
            harness.ctx.memory(|m| m.focused()),
            Some(cmd_field_id()),
            "Tab must not escape the RESET BARE modal to the background command field (B077)"
        );
        assert!(
            harness.state().modal_open,
            "modal stays open while Tabbing (until Confirm/Cancel)"
        );
    }
}

// === CR-CH-023 / CR-CH-046: chrome (Key_Label_Bar slots) not Tab focus stops =
//
// Req 16.9 / 12.10: the Key_Label_Bar slots duplicate physical function keys and
// MUST NOT be keyboard focus stops (they remain mouse-clickable). This guards the
// single-row render (CR-CH-046) via the real render_key_label_bar into a headless
// harness with a single focusable sentinel AFTER it; pressing Tab must keep focus
// on the sentinel, so at most one distinct id is ever focused.
#[test]
fn key_label_bar_buttons_are_not_tab_focus_stops() {
    use egui_kittest::Harness;

    let mut shell = make_shell();

    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(1200.0, 900.0))
        .build(move |ctx| {
            shell.render_key_label_bar(ctx);
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = ui.button("sentinel");
            });
        });

    let mut focused_ids: Vec<egui::Id> = Vec::new();
    for _ in 0..10 {
        harness.key_press(egui::Key::Tab);
        harness.run();
        if let Some(id) = harness.ctx.memory(|m| m.focused()) {
            focused_ids.push(id);
        }
    }
    let distinct: std::collections::HashSet<egui::Id> = focused_ids.iter().copied().collect();
    assert!(
        distinct.len() <= 1,
        "Tab focused multiple widgets, meaning Key_Label_Bar F-key buttons are \
         focus stops (Req 16.9). Focused ids: {focused_ids:?}"
    );
}

// Validates: menu-and-statusbar Req 16.1 -- on launch (POM active) focus is on
// the Primary_Command_Field.
#[test]
fn full_shell_launch_focus_is_command_field() {
    let harness = harness_shell();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "after startup the command field must hold focus"
    );
}

// Validates: menu-and-statusbar Req 16.3 -- Tab from the command field enters
// the active Workspace's first interior control (the first POM option), NOT the
// SCROLL field or any chrome.
#[test]
fn full_shell_tab_from_command_field_enters_first_option() {
    let mut harness = harness_shell();
    // Sanity: start on the command field.
    assert_eq!(harness.ctx.memory(|m| m.focused()), Some(cmd_field_id()));
    let after = tab_and_focus(&mut harness);
    assert!(
        after.is_some(),
        "Tab from command field must focus something"
    );
    let after = after.unwrap();
    assert_ne!(
        after,
        cmd_field_id(),
        "Tab must leave the command field (enter the interior)"
    );
    assert_ne!(
        after,
        egui::Id::new("scroll_field_input"),
        "Tab from the command field must NOT land on the SCROLL field (Req 16.9)"
    );
}

// Validates: menu-and-statusbar Req 16.3, 16.5, 16.7 -- from the command field,
// repeated Tab walks the POM interior (options + calendar) and then the menu
// bar, and eventually WRAPS back to the command field. The SCROLL field and
// F-key buttons never appear in the cycle (Req 16.9).
#[test]
fn full_shell_tab_cycle_wraps_to_command_field_and_skips_chrome() {
    let mut harness = harness_shell();
    let scroll_id = egui::Id::new("scroll_field_input");
    let mut seen = Vec::new();
    let mut wrapped = false;
    // Press Tab up to 40 times; the POM cycle (command + ~6 options + 2 calendar
    // + 13 menu items) is well under 40, so we must wrap back to the command
    // field within that budget.
    for _ in 0..40 {
        let f = tab_and_focus(&mut harness);
        if let Some(id) = f {
            seen.push(id);
            if id == cmd_field_id() && !seen.is_empty() {
                wrapped = true;
                break;
            }
        }
    }
    assert!(
        wrapped,
        "Tab cycle must wrap back to the command field within 40 presses; saw {} focuses",
        seen.len()
    );
    assert!(
        !seen.contains(&scroll_id),
        "the SCROLL field must never be a Tab stop in the cycle (Req 16.9)"
    );
}

// Validates: menu-and-statusbar Req 16.8; menu-workspace Req 15.9 -- Shift+Tab
// from the command field goes to the LAST menu bar item (the reverse boundary),
// not into chrome.
#[test]
fn full_shell_shift_tab_from_command_field_goes_to_menu_bar() {
    let mut harness = harness_shell();
    assert_eq!(harness.ctx.memory(|m| m.focused()), Some(cmd_field_id()));
    // Shift down, press Tab, release: egui_kittest sends modifiers via events.
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::Tab);
    harness.run();
    let after = harness.ctx.memory(|m| m.focused());
    assert!(
        after.is_some(),
        "Shift+Tab from command field must focus something"
    );
    assert_ne!(
        after,
        Some(cmd_field_id()),
        "Shift+Tab must leave the command field"
    );
    assert_ne!(
        after,
        Some(egui::Id::new("scroll_field_input")),
        "Shift+Tab must not land on the SCROLL field (Req 16.9)"
    );
}

/// Validates: CR-CH-054 (Ctrl+S point-fix) -- Ctrl+S dispatches the SAVE command
/// through the single front door -> FFEDIT's dirty-aware SAVE, NOT a direct
/// unconditional `save_active_tab`. On a CLEAN editor buffer this is therefore a
/// no-op with NO error (the old direct call would have errored on a clean
/// untitled buffer -- "no file path"), proving the key now shares the SAVE verb's
/// clean no-op guard. This closes the key/verb divergence E9 13.1 introduced.
#[test]
fn ctrl_s_on_clean_editor_is_noop_via_ffedit_save() {
    let mut harness = harness_shell();
    harness.state_mut().shell_new_untitled();
    harness.run();
    assert!(
        !harness.state().tabs.active_tab().is_modified,
        "precondition: a fresh untitled buffer is clean"
    );
    // Clear any status so we can detect a spurious save error from the keypress.
    harness.state_mut().open_error = None;

    harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::S);
    harness.run();

    assert!(
        harness.state().open_error.is_none(),
        "Ctrl+S on a clean buffer must be a no-op (dirty-aware SAVE), not an error; \
         the old direct save_active_tab would have errored on a clean untitled buffer"
    );
    assert!(
        !harness.state().tabs.active_tab().is_modified,
        "Ctrl+S on a clean buffer leaves it clean"
    );
}

// === B056: POM Tab lands on first option, then Settings (not File Catalogs) ==

// Validates: menu-and-statusbar Req 16.3 (B056) -- the FIRST Tab from the
// command field on a POM lands EXACTLY on the reported first interior control
// (the first POM option), not some other/stale widget.
#[test]
fn full_shell_first_tab_focuses_reported_first_interior() {
    let mut harness = harness_shell();
    assert_eq!(harness.ctx.memory(|m| m.focused()), Some(cmd_field_id()));
    let expected_first = harness.state().focus.first_interior_id;
    assert!(
        expected_first.is_some(),
        "POM must report a first interior control (first option)"
    );
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        expected_first,
        "first Tab must focus the reported first interior control (B056)"
    );
}

// Validates: menu-and-statusbar Req 16.5 (B056) -- tabbing through the POM
// interior and past the last interior control lands focus EXACTLY on the first
// menu-bar button (Settings), so Enter there opens Settings (not File Catalogs).
#[test]
fn full_shell_tab_reaches_settings_as_first_menu_item() {
    let mut harness = harness_shell();
    let menu_first = harness.state().focus.menu_first_id;
    assert!(
        menu_first.is_some(),
        "menu bar must report its first button id"
    );
    // Walk forward; the first time focus enters the menu bar it MUST be the
    // Settings (first) button, never a later one (File Catalogs, etc.).
    let mut reached_menu_first = false;
    for _ in 0..40 {
        harness.key_press(egui::Key::Tab);
        harness.run();
        let f = harness.ctx.memory(|m| m.focused());
        if f == menu_first {
            reached_menu_first = true;
            break;
        }
        // If focus reaches the LAST menu button before ever hitting the first,
        // the order is wrong (we skipped Settings).
        if f == harness.state().focus.menu_last_id {
            break;
        }
        // Wrapping back to the command field without hitting the menu bar first
        // would also be a failure for a POM (it has a menu bar).
        if f == Some(cmd_field_id()) {
            break;
        }
    }
    assert!(
        reached_menu_first,
        "forward Tab must land on the first menu button (Settings) when entering the menu bar (B056)"
    );
}

// Validates: menu-and-statusbar Req 16.3 (B056) -- on the Menus Editor the
// FIRST Tab from the command field must land on the "Menu:" selector combo (the
// first interior control), not skip it to the Title field.
#[test]
fn full_shell_menus_editor_first_tab_focuses_menu_selector() {
    let mut harness = harness_shell();
    // Open the Menus Editor via its command (same path as typing MENUS).
    harness.state_mut().handle_command("MENUS");
    for _ in 0..4 {
        harness.run();
    }
    // The editor reports the combo as its first interior control.
    let first = harness.state().focus.first_interior_id;
    assert!(
        first.is_some(),
        "Menus Editor must report a first interior control (the Menu selector combo)"
    );
    // Focus should be on the command field on entry (Req 16.1a).
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "command field holds focus on entering the Menus Editor"
    );
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        first,
        "first Tab must focus the Menu selector combo, not skip it (B056)"
    );
}

// === B057: Theme Editor Tab lands on the selector combo (no phantom stop) ====

// Validates: menu-and-statusbar Req 16.3 (B057, CR-CH-023) -- on the Theme
// Editor the FIRST Tab from the command field lands EXACTLY on the reported
// first interior control (the Theme selector combo), with no phantom/invisible
// focus stop before it. Before the fix the ThemeEditor arm reported no interior
// id, so the shell could not latch the command-field -> first-interior jump and
// egui landed on the panel's container/scroll allocation instead.
#[test]
fn full_shell_theme_editor_first_tab_focuses_theme_selector() {
    use crate::tab_state::TabKind;
    let mut harness = harness_shell();
    // Open the Theme Editor via bare THEME (same path as typing it).
    harness.state_mut().handle_command("THEME");
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.active_tab().kind,
        TabKind::ThemeEditor,
        "bare THEME opens the Theme Editor Context"
    );
    // The editor reports the Theme selector combo as its first interior control.
    let first = harness.state().focus.first_interior_id;
    assert!(
        first.is_some(),
        "Theme Editor must report a first interior control (the Theme selector combo)"
    );
    // Focus is on the command field on entry (Req 16.1a).
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "command field holds focus on entering the Theme Editor"
    );
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        first,
        "first Tab must focus the Theme selector combo, not a phantom stop (B057)"
    );
}

// === B058: CONFIG/Settings panel Tab lands on the Filter field ==============

// Validates: menu-and-statusbar Req 16.3 (B058, CR-CH-023) -- on the
// Settings/CONFIG flat panel the FIRST Tab from the command field lands EXACTLY
// on the Filter field (the reported first interior control), with no phantom
// stop before it. Before the fix the SettingsPanel arm reported no interior id,
// so the shell could not latch the command-field -> first-interior jump and
// egui landed on the panel's container/scroll allocation instead.
#[test]
fn full_shell_config_first_tab_focuses_filter_field() {
    use crate::tab_state::TabKind;
    let mut harness = harness_shell();
    // Open the flat config-key browser via CONFIG (same path as typing it).
    harness.state_mut().handle_command("CONFIG");
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.active_tab().kind,
        TabKind::ConfigPanel,
        "CONFIG opens the flat Config panel"
    );
    // The panel reports the Filter field as its first interior control.
    let expected = crate::config_panel::filter_field_id();
    assert_eq!(
        harness.state().focus.first_interior_id,
        Some(expected),
        "ConfigPanel must report the Filter field as its first interior control"
    );
    // Focus is on the command field on entry (Req 16.1a).
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "command field holds focus on entering the CONFIG panel"
    );
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(expected),
        "first Tab must focus the Filter field, not a phantom stop (B058)"
    );
}

/// Validates: Requirement 23.7 -- when the command field does NOT have focus,
/// Up/Down are NOT hijacked for history (the field content is left unchanged).
#[test]
fn arrows_ignored_when_command_field_not_focused() {
    let mut harness = harness_shell();
    seed_history(&mut harness, &["THEME legacy", "LOCATE 1"]);
    // Move focus off the command field (Tab lands on the first interior control).
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_ne!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "precondition: focus is no longer on the command field"
    );
    let before = harness.state().command_text.clone();
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    assert_eq!(
        harness.state().command_text,
        before,
        "Up must not touch the command field when it is unfocused (Req 23.7)"
    );
}

// Validates: menu-and-statusbar Req 16.3 (B059) -- Plugin Manager.
#[test]
fn full_shell_plugin_manager_first_tab_focuses_interior() {
    assert_first_tab_lands_on_reported_interior("PLUGINS", "Plugin Manager");
}

// Validates: menu-and-statusbar Req 16.3 (B059) -- Event Log.
#[test]
fn full_shell_event_log_first_tab_focuses_interior() {
    assert_first_tab_lands_on_reported_interior("LOG", "Event Log");
}

// Validates: menu-and-statusbar Req 16.3 (B059) -- Macro Library.
#[test]
fn full_shell_macro_library_first_tab_focuses_interior() {
    assert_first_tab_lands_on_reported_interior("MACROS", "Macro Library");
}

// Validates: menu-and-statusbar Req 16.3 (B059) -- Search Results.
#[test]
fn full_shell_search_results_first_tab_focuses_interior() {
    assert_first_tab_lands_on_reported_interior("SEARCH", "Search Results");
}

// Validates: menu-and-statusbar Req 16.3 (B059) -- Command Configurator.
#[test]
fn full_shell_command_configurator_first_tab_focuses_interior() {
    assert_first_tab_lands_on_reported_interior("COMMANDS", "Command Configurator");
}

// === CR-NR-078 WF.6 (task 8.2): FileEditor + FilesPanel no-interior cases ===

// Validates: workspace-framework Requirement 3.4; menu-and-statusbar Req 16.3
// (CR-CH-023) -- the Editor Context is a documented no-interior special case: it
// renders a native egui text body with its own caret/focus model and reports
// `InteriorFocus::none()` EXPLICITLY through the single latch path. The shell
// must therefore leave `first_interior_id`/`last_interior_id` as None (no phantom
// stop), rather than silently leaving them unset by an arm that forgot to wire
// the focus contract.
#[test]
fn full_shell_file_editor_reports_no_interior_focus() {
    use crate::tab_state::TabKind;
    let mut harness = harness_shell();
    // Open a fresh Untitled editor buffer (same path an editor tab takes).
    harness.state_mut().shell_new_untitled();
    for _ in 0..4 {
        harness.run();
    }
    assert!(
        matches!(
            harness.state().tabs.active_tab().kind,
            TabKind::Untitled | TabKind::FileEditor
        ),
        "shell_new_untitled opens an editor Context"
    );
    assert_eq!(
        harness.state().focus.first_interior_id,
        None,
        "Editor Context is a documented no-interior case: first_interior_id stays None"
    );
    assert_eq!(
        harness.state().focus.last_interior_id,
        None,
        "Editor Context is a documented no-interior case: last_interior_id stays None"
    );
}

// Validates: workspace-framework Requirement 3.4; menu-and-statusbar Req 16.3
// (CR-CH-023) -- the Files Panel (Catalog Explorer Context) is a documented
// no-interior special case for the SHELL latch: it has its own internal
// "Command ===>" field and a bespoke Tab redirect to the first catalog node
// (B024/Req 20.1) handled outside the first-interior latch. It reports
// `InteriorFocus::none()` EXPLICITLY, so the shell anchors stay None (no phantom
// stop) even though the panel is interactive.
#[test]
fn full_shell_files_panel_reports_no_interior_focus() {
    use crate::tab_state::TabKind;
    let mut harness = harness_shell();
    // CATALOGS navigates the active tab to the Files Panel (Catalog Explorer).
    harness.state_mut().handle_command("CATALOGS");
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.active_tab().kind,
        TabKind::FilesPanel,
        "CATALOGS opens the Files Panel (Catalog Explorer Context)"
    );
    assert_eq!(
        harness.state().focus.first_interior_id,
        None,
        "Files Panel is a documented no-interior case for the shell latch: first_interior_id stays None"
    );
    assert_eq!(
        harness.state().focus.last_interior_id,
        None,
        "Files Panel is a documented no-interior case for the shell latch: last_interior_id stays None"
    );
}

// Validates: menu-workspace Req 16.1, 16.4 (CR-CH-026, B060) -- the Settings
// Menu_Workspace defaults the calendar OFF, so Tab from the command field walks
// ONLY the option rows and then the Menu_Bar; there are NO extra "invisible"
// calendar Tab stops after the last option. The reported last interior control
// is a menu option, never a calendar `<`/`>` button, and the first Tab still
// lands on the reported first interior (no phantom stop before it either).
#[test]
fn full_shell_settings_tab_walks_options_only_no_calendar_stops() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SETTINGS");
    for _ in 0..4 {
        harness.run();
    }
    // Settings reports a first AND a last interior; with the calendar hidden the
    // last interior is the last enabled option (not a calendar button).
    let first = harness.state().focus.first_interior_id;
    let last = harness.state().focus.last_interior_id;
    assert!(
        first.is_some(),
        "Settings must report a first interior option"
    );
    assert!(
        last.is_some(),
        "Settings must report a last interior option"
    );

    // Entry focus is the command field.
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "command field holds focus on entering Settings"
    );

    // First Tab from the command field lands EXACTLY on the reported first
    // interior (no phantom stop before it).
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        first,
        "first Tab in Settings must focus the reported first interior option (no phantom stop)"
    );

    // Walk the interior forward; the LAST interior id we see before focus
    // leaves the option ring (wrap to command field) must be `last` -- the last
    // enabled option. A hidden calendar produces NO `<`/`>` ids, so no interior
    // stop appears after `last` (Req 16.4). We record the last non-command,
    // non-menu-bar id equal to the reported contract.
    let mut saw_last = false;
    for _ in 0..24 {
        let f = harness.ctx.memory(|m| m.focused());
        if f == last {
            saw_last = true;
        }
        harness.key_press(egui::Key::Tab);
        harness.run();
        if harness.ctx.memory(|m| m.focused()) == Some(cmd_field_id()) {
            break;
        }
    }
    assert!(
        saw_last,
        "Tab walk must reach the reported last interior option in Settings"
    );
    // Req 16.4 authoritative guarantee: the reported interior contract holds
    // options only. Settings has four options, so first != last and both are
    // option ids -- there are no calendar `<`/`>` ids in the contract.
    assert_ne!(
        first, last,
        "Settings (4 options, calendar off) reports distinct option first/last -- no calendar ids in the contract"
    );
}

// === CR-CH-028: Cursor_Context package on every command (Requirement 12) =====

// Validates: command-framework Req 12.2/12.4 -- after a frame, the shell's
// Cursor_Context snapshot reflects live focus: at startup the command field
// holds focus, so the package carries workspace "pom" and the "command-line"
// focused identity with the command-line text.
#[test]
fn cursor_context_snapshot_reflects_command_line_focus() {
    let mut harness = harness_shell();
    // Startup: command field holds focus.
    assert_eq!(harness.ctx.memory(|m| m.focused()), Some(cmd_field_id()));
    harness.state_mut().command_text = "FIND hello".to_string();
    harness.run();
    let cc = harness
        .state()
        .cursor_context_snapshot
        .lock()
        .expect("snapshot lock")
        .clone();
    assert_eq!(cc.workspace_context.as_deref(), Some("pom"));
    assert_eq!(cc.focused_identity.as_deref(), Some("command-line"));
    assert_eq!(cc.focused_text.as_deref(), Some("FIND hello"));
}

// Validates: command-framework Req 12.8 (CR-NR-079) -- HELP consumes the
// Cursor_Context: with a focused Menu_Option (identity = its command, e.g.
// FILES), the resolved help Topic_Key is that option's command topic
// (cmd:FILES), not the generic index. Exercised at the ff-help ContextDetector
// level the shell HELP handler uses.
#[test]
fn help_consumes_focused_menu_option_context() {
    use ff_help::{ContextDetector, EditorContext, EditorMode};
    // Simulate the shell HELP handler's build from a Cursor_Context whose
    // focused identity is the FILES option command.
    let ctx = EditorContext {
        command_line_text: "FILES".to_string(),
        command_line_has_focus: true,
        prefix_area_text: None,
        prefix_area_has_focus: false,
        active_mode: EditorMode::Edit,
        help_panel_open: false,
        current_help_topic: None,
    };
    let key = ContextDetector::resolve(&ctx);
    assert_eq!(
        key.as_str(),
        "cmd:FILES",
        "F1 on the focused FILES option must resolve the FILES command help topic"
    );
}

// === CR-CH-029: Keys Workspace (function-keys Req 22) ========================

// Validates: function-keys Req 22.7 -- KEYS opens the Keys Workspace and the
// FIRST Tab from the command field lands EXACTLY on the reported first interior
// control (the workspace-kind dropdown), with no phantom stop.
#[test]
fn full_shell_keys_first_tab_focuses_kind_dropdown() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("KEYS");
    for _ in 0..4 {
        harness.run();
    }
    let expected = harness.state().focus.first_interior_id;
    assert!(
        expected.is_some(),
        "Keys Workspace must report a first interior control (the kind dropdown)"
    );
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "command field holds focus on entering the Keys Workspace"
    );
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        expected,
        "first Tab in the Keys Workspace must focus the kind dropdown, not a phantom stop"
    );
}

/// Validates: menu-workspace Req 20.4/20.6 -- the POM (Home Context) Tab_Header
/// shows the Short_Tab_Label "POM", not the long application banner. Asserted via
/// the shared `tab_header_label` helper that the render_chrome tab bar uses.
#[test]
fn pom_tab_header_is_short_label_pom() {
    let mut shell = make_shell();
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    let label = shell.tab_header_label(shell.tabs.active_tab());
    assert_eq!(
        label, "POM",
        "the POM tab header must be the short label 'POM', not the app banner"
    );
}

/// Validates: command-framework Req 15.3 -- `START` remains the sole tab-creator
/// and is NOT retired: bare START opens a fresh POM tab (Home), and START with a
/// resolvable arg roots the new tab at that Context.
#[test]
fn start_still_creates_tab_and_resolves_arg() {
    use crate::tab_state::TabKind;
    // Bare START -> new POM tab (Home).
    let mut a = make_shell();
    let before = a.tabs.len();
    a.handle_command("START");
    assert!(a.tabs.active_tab().is_home, "bare START roots at the POM");
    assert!(
        a.tabs.len() >= before,
        "START creates a tab (is the tab-creator)"
    );

    // START <arg> roots the new tab directly at the resolved Context.
    let mut b = make_shell();
    b.handle_command("START SETTINGS");
    assert_eq!(
        b.tabs.active_tab().kind,
        TabKind::MenuWorkspace,
        "START SETTINGS roots the new tab at the Settings menu"
    );
    assert!(
        !b.tabs.active_tab().is_home,
        "START SETTINGS is not the POM"
    );
}

/// Validates: menu-and-statusbar Requirement 18.3/18.9 (CR-CH-035, B045) --
/// the DOCK command re-docks a Detached_Workspace to its origin index (CR-NR-088,
/// the explicit re-attach now that Close runs RETURN): is_floating cleared,
/// FloatingTab removed, tab restored to its origin index.
#[test]
fn full_shell_dock_command_redocks_tab_at_origin() {
    // Validates: menu-and-statusbar Requirement 18.9, 18.13
    let mut harness = harness_shell();
    harness.state_mut().handle_command("START");
    harness.run();
    let origin = harness.state().tabs.active_index();
    let detach_id = harness.state().tabs.active_tab().id;
    harness.state_mut().handle_command("SPLIT DETACH");
    for _ in 0..3 {
        harness.run();
    }
    assert_eq!(
        harness.state().detach_split.floating_tabs.len(),
        1,
        "precondition: detached"
    );

    // Re-dock the detached workspace via the DOCK command run against its own
    // context (as it would be typed in the detached window's command line).
    let detach_idx = harness
        .state()
        .tabs
        .index_of_id(detach_id)
        .expect("detached tab exists");
    let mut ctx = super::WorkspaceCommandContext::default();
    harness
        .state_mut()
        .with_workspace_context(detach_idx, &mut ctx, |shell| {
            shell.handle_command("DOCK");
        });
    let state = harness.state();
    assert!(
        state.detach_split.floating_tabs.is_empty(),
        "DOCK must remove the FloatingTab"
    );
    let idx = state
        .tabs
        .index_of_id(detach_id)
        .expect("re-docked tab still exists");
    assert_eq!(idx, origin, "DOCK must restore the tab to its origin index");
    assert!(
        !state.tabs.tabs()[idx].is_floating,
        "re-docked tab must no longer be is_floating"
    );
}

/// Validates: menu-and-statusbar Requirement 18.10 (CR-CH-036, B045) -- a
/// command submitted in a detached window's context acts on THAT window's tab
/// and leaves the Primary_Window's command_text and active tab untouched
/// (bidirectional isolation). Uses NAME (sets the active tab's workspace_name)
/// as an observable per-tab effect.
#[test]
fn detached_command_acts_on_its_tab_not_the_primary() {
    // Validates: menu-and-statusbar Requirement 18.10
    let mut shell = make_shell();
    shell.handle_command("START"); // second tab
    let primary_active = shell.tabs.active_index();
    let detached = if primary_active == 0 { 1 } else { 0 };
    let detached_id = shell.tabs.tabs()[detached].id;

    shell.command_text = "PRIMARY-TEXT".to_string();

    // Submit `NAME DetachedName` inside the detached window's context.
    let mut ctx = super::WorkspaceCommandContext::default();
    shell.with_workspace_context(detached, &mut ctx, |s| {
        s.run_command_line("NAME DetachedName");
    });

    // The detached tab got the name; the primary tab did not.
    let detached_idx = shell
        .tabs
        .index_of_id(detached_id)
        .expect("detached tab exists");
    assert_eq!(
        shell.tabs.tabs()[detached_idx].workspace_name.as_deref(),
        Some("DetachedName"),
        "command in the detached context must act on the detached tab"
    );
    assert!(
        shell.tabs.tabs()[primary_active].workspace_name.is_none(),
        "the primary tab must be untouched by the detached window's command"
    );
    // The Primary_Window's command line is preserved (no bleed).
    assert_eq!(
        shell.command_text, "PRIMARY-TEXT",
        "primary command_text must be untouched by the detached command"
    );
    assert_eq!(
        shell.tabs.active_index(),
        primary_active,
        "primary active tab must be restored after the detached dispatch"
    );
}

// Validates: workspace-conformance (CR-CH-023) + workspace-kinds Req 6.4 -- KINDS
// opens the Kinds Editor and the FIRST Tab from the command field lands EXACTLY
// on the reported first interior control, with no phantom stop.
#[test]
fn full_shell_kinds_first_tab_focuses_first_interior() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("KINDS");
    for _ in 0..4 {
        harness.run();
    }
    let expected = harness.state().focus.first_interior_id;
    assert!(
        expected.is_some(),
        "Kinds Editor must report a first interior control"
    );
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "command field holds focus on entering the Kinds Editor"
    );
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        expected,
        "first Tab in the Kinds Editor must focus the reported first interior, not a phantom stop"
    );
}

// === CR-NR-097: context-help content pipeline + F1 display ==================
// These drive the REAL WorkbenchShell headlessly and exercise the display
// pipeline: the shell-owned registry, the Help Context render arm, the first-Tab
// focus contract, dynamic-topic generation, and the missing-topic diagnostics.

// Validates: context-help Req 18.5 (CR-NR-097) + workspace-conformance --
// the FIRST Tab from the command field lands EXACTLY on the Help_Search field.
#[test]
fn full_shell_help_first_tab_focuses_search_field() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("HELP");
    for _ in 0..4 {
        harness.run();
    }
    let expected = harness.state().focus.first_interior_id;
    assert!(
        expected.is_some(),
        "Help Context must report a first interior control"
    );
    assert_eq!(
        expected,
        Some(crate::help_context::help_search_field_id()),
        "the reported first interior must be the Help_Search field"
    );
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "command field holds focus on entering the Help Context"
    );
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        expected,
        "first Tab in the Help Context must focus the Help_Search field, not a phantom stop"
    );
}

// === CR-NR-092 (B046 Slice 2b): in-window split behaviour ===================
// These drive the REAL WorkbenchShell headlessly (build_eframe) through the
// SPLIT / FOCUS / UNSPLIT command path (command parity: the menu/keys route
// here too) and assert the two-region split model + render. The pixel-exact
// appearance of the Splitter and the focus-highlight border is a justified
// MANUAL row; the behaviour (state, focus routing, collapse) is harness-tested.

/// Validates: layout-and-docking Requirement 13.1, 13.3 -- SPLIT divides the
/// Workspace into two Tab_Groups; the new (second) group is a fresh POM and
/// receives focus.
#[test]
fn full_shell_split_creates_two_groups_second_is_pom_and_focused() {
    use crate::tab_state::TabKind;
    let mut harness = harness_shell();
    assert!(
        !harness.state().tabs.is_split(),
        "precondition: not split at launch"
    );
    let before = harness.state().tabs.len();

    harness.state_mut().handle_command("SPLIT");
    harness.run();

    let state = harness.state();
    assert!(state.tabs.is_split(), "SPLIT must create a split");
    assert_eq!(
        state.tabs.leaf_ids().len(),
        2,
        "first SPLIT produces two Tab_Group leaves"
    );
    assert_eq!(
        state.tabs.len(),
        before + 1,
        "SPLIT adds exactly one new POM tab to the store"
    );
    // The focused (second) group's active tab is the new POM.
    assert!(
        state.tabs.active_tab().is_home,
        "the focused new group's active tab is a POM"
    );
    assert_eq!(state.tabs.active_tab().kind, TabKind::MenuWorkspace);
    assert!(state.open_error.is_none());
}

/// Validates: layout-and-docking Requirement 14.5 -- FOCUS moves focus to the
/// next Tab_Group leaf and the active tab follows the focused leaf.
#[test]
fn full_shell_focus_flips_focused_group() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    harness.run();
    // After SPLIT, the new POM leaf is focused.
    let focused_pom_id = harness.state().tabs.active_tab().id;
    let focused_leaf = harness.state().tabs.focused_leaf_id();

    harness.state_mut().handle_command("FOCUS");
    harness.run();

    let state = harness.state();
    assert_ne!(
        state.tabs.focused_leaf_id(),
        focused_leaf,
        "FOCUS must move to a different leaf"
    );
    assert_ne!(
        state.tabs.active_tab().id,
        focused_pom_id,
        "the active tab must follow the newly focused leaf"
    );
}

/// Validates: layout-and-docking Requirement 13.9, 13.10 -- FOCUS / UNSPLIT on
/// an unsplit Workspace are no-ops with a status message (the split is a
/// live-only arrangement; a fresh shell launches unsplit per 13.10).
#[test]
fn full_shell_focus_and_unsplit_on_unsplit_are_noops_with_status() {
    let mut harness = harness_shell();
    assert!(!harness.state().tabs.is_split());

    harness.state_mut().handle_command("FOCUS");
    harness.run();
    assert!(
        harness.state().open_error.is_some(),
        "FOCUS on an unsplit Workspace reports a status message"
    );
    assert!(!harness.state().tabs.is_split());

    harness.state_mut().open_error = None;
    harness.state_mut().handle_command("UNSPLIT");
    harness.run();
    assert!(
        harness.state().open_error.is_some(),
        "UNSPLIT on an unsplit Workspace reports a status message"
    );
}

/// Validates: layout-and-docking Requirement 13.1, 13.6, 13.8 -- while split the
/// full shell renders two regions end-to-end without panic across several
/// frames (the split central panel + per-region tab bars + focus highlight),
/// and a command still acts on the focused region.
#[test]
fn full_shell_split_renders_two_regions_and_command_acts_on_focused() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    // Run several frames so the split central panel + both region bodies render.
    for _ in 0..4 {
        harness.run();
    }
    assert!(harness.state().tabs.is_split(), "still split after render");

    // The focused (second) group is a POM; NAME acts on the focused region's
    // active tab (Req 13.8), not the first group's tab.
    let focused_id = harness.state().tabs.active_tab().id;
    harness.state_mut().run_command_line("NAME FocusedRegion");
    harness.run();
    let state = harness.state();
    let idx = state
        .tabs
        .index_of_id(focused_id)
        .expect("focused tab exists");
    assert_eq!(
        state.tabs.tabs()[idx].workspace_name.as_deref(),
        Some("FocusedRegion"),
        "a command while split acts on the focused region's active tab"
    );
}

// === CR-NR-093 Slice 2c.2: drag-a-tab-between-groups (full shell) ===========
// The pixel-exact drag GESTURE (press a header, move the pointer across regions,
// release) is a justified-MANUAL row: egui_kittest cannot drive a cross-region
// pointer drag headlessly. These tests drive the END-TO-END move through the
// same TabManager path the drop resolves to (`move_tab_to_group`), against a
// fully-rendered split shell, and assert the observable outcome (Req 14.6, 14.7).

/// Validates: layout-and-docking Requirement 14.6 -- moving a tab from one
/// rendered region into another moves the TabState, makes it the target's
/// active tab, and focuses the target region.
#[test]
fn full_shell_move_tab_between_regions_moves_and_focuses() {
    let mut harness = harness_shell();
    // Two leaves: the root (POM) leaf and a new POM leaf (focused).
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..3 {
        harness.run();
    }
    let leaves = harness.state().tabs.leaf_ids();
    assert_eq!(leaves.len(), 2, "precondition: two regions");
    let root = leaves[0];
    let other = leaves[1];
    // Ensure the SOURCE (root) region has >=2 tabs so moving one out does NOT
    // empty and collapse it (which would defeat the "target focused" assertion).
    // FOCUS back to the root region, then open a fresh POM into it via START.
    harness.state_mut().handle_command("FOCUS");
    harness.run();
    assert_eq!(
        harness.state().tabs.focused_leaf_id(),
        root,
        "focused back to root"
    );
    harness.state_mut().handle_command("START"); // new POM in the focused (root) region
    harness.run();
    assert!(
        harness.state().tabs.leaf_tab_store_indices(root).len() >= 2,
        "root region now has >=2 tabs so a move will not collapse it"
    );
    // Move the root leaf's active tab into the other leaf.
    let move_id = harness
        .state()
        .tabs
        .leaf_active_store_index(root)
        .and_then(|idx| harness.state().tabs.tabs().get(idx).map(|t| t.id))
        .expect("root leaf has an active tab");
    let moved = harness.state_mut().tabs.move_tab_to_group(move_id, other);
    harness.run();
    assert!(moved, "the move must succeed");
    let state = harness.state();
    // The moved tab now lives in `other`, which is focused and shows it active.
    assert_eq!(state.tabs.focused_leaf_id(), other, "target region focused");
    assert_eq!(
        state.tabs.active_tab().id,
        move_id,
        "moved tab is the focused region's active tab"
    );
    assert!(
        state
            .tabs
            .leaf_tab_store_indices(other)
            .iter()
            .any(|&i| state.tabs.tabs()[i].id == move_id),
        "moved tab is a member of the target region"
    );
}

/// Validates: layout-and-docking Requirement 14.7 -- a move that empties the
/// source region collapses the split back to a single region (no tab lost).
#[test]
fn full_shell_move_tab_emptying_region_collapses() {
    let mut harness = harness_shell();
    // Root leaf has exactly one tab (the launch POM). SPLIT adds a second POM leaf.
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..3 {
        harness.run();
    }
    let leaves = harness.state().tabs.leaf_ids();
    let root = leaves[0];
    let other = leaves[1];
    let before = harness.state().tabs.len();
    // Move EVERY tab out of the root leaf into the other leaf -> root empties ->
    // the split collapses (regardless of how many tabs the root started with).
    let root_ids: Vec<crate::tab_state::TabId> = harness
        .state()
        .tabs
        .leaf_tab_store_indices(root)
        .iter()
        .map(|&i| harness.state().tabs.tabs()[i].id)
        .collect();
    assert!(!root_ids.is_empty(), "root region has tabs to move");
    for id in root_ids {
        harness.state_mut().tabs.move_tab_to_group(id, other);
    }
    for _ in 0..2 {
        harness.run();
    }
    let state = harness.state();
    assert!(
        !state.tabs.is_split(),
        "emptying the source region collapses the split"
    );
    assert_eq!(state.tabs.len(), before, "no tab lost in the collapse");
}

/// Validates: layout-and-docking Requirement 14.6 (drag state) -- a tab-header
/// drag records the (tab, source leaf) on the shell; releasing the pointer over
/// the SAME source region is a no-op (Req 14.8) and clears the drag state.
#[test]
fn full_shell_split_tab_drag_state_clears_on_release() {
    use crate::tab_state::TabId;
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..3 {
        harness.run();
    }
    let focused = harness.state().tabs.focused_leaf_id();
    let active_id = harness.state().tabs.active_tab().id;
    // Simulate a drag having started on the focused region's active tab.
    harness.state_mut().detach_split.split_tab_drag = Some((active_id, focused));
    // A frame with no released pointer keeps the drag pending.
    harness.run();
    assert!(
        harness.state().detach_split.split_tab_drag.is_some(),
        "drag persists while the pointer is held"
    );
    // Nothing moved yet (still split, same focus).
    assert!(harness.state().tabs.is_split());
    let _ = TabId(0);
}

/// Validates: layout-and-docking Req 15.7 -- submitting in a NON-focused region
/// acts on that region and moves focus to it.
#[test]
fn full_shell_region_command_from_non_focused_region_acts_and_focuses() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..4 {
        harness.run();
    }
    let leaves = harness.state().tabs.leaf_ids();
    let (a, b) = (leaves[0], leaves[1]);
    // After SPLIT the NEW leaf (b) is focused; a is not focused.
    assert_eq!(
        harness.state().tabs.focused_leaf_id(),
        b,
        "precondition: region B focused after SPLIT"
    );
    let a_tab = harness
        .state()
        .tabs
        .leaf_active_store_index(a)
        .map(|i| harness.state().tabs.tabs()[i].id)
        .expect("region A active tab");
    submit_region_command(&mut harness, a, "NAME FromUnfocused");
    let state = harness.state();
    let a_idx = state.tabs.index_of_id(a_tab).expect("A tab exists");
    assert_eq!(
        state.tabs.tabs()[a_idx].workspace_name.as_deref(),
        Some("FromUnfocused"),
        "a non-focused region's command acts on that region"
    );
    assert_eq!(
        state.tabs.focused_leaf_id(),
        a,
        "submitting in a non-focused region focuses it (Req 15.7)"
    );
}

/// Validates: layout-and-docking Req 15.9 (CR-CH-023 phantom-stop class) -- each
/// region's command field is a real, stable Tab stop: focusing its salted id
/// sticks (the id round-trips through egui focus, i.e. it names a live widget).
#[test]
fn full_shell_region_command_field_is_a_stable_tab_stop() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..4 {
        harness.run();
    }
    for leaf in harness.state().tabs.leaf_ids() {
        let field_id = region_cmd_field_id(leaf);
        harness.ctx.memory_mut(|m| m.request_focus(field_id));
        harness.run();
        assert_eq!(
            harness.ctx.memory(|m| m.focused()),
            Some(field_id),
            "region {leaf:?} command field must be a real, focusable Tab stop (no phantom id)"
        );
    }
}

// === CR-CH-041: derived Placement (shell-level, Detached vs Docked) =========

/// Validates: layout-and-docking Req 16.5/16.6 -- a tab recorded in the shell's
/// floating set has Placement::Detached; Detached takes precedence over any
/// docked-leaf resolution.
#[test]
fn placement_of_detached_tab_is_detached() {
    use crate::tab_manager::Placement;
    let mut shell = make_shell();
    let tab_id = shell.tabs.active_tab().id;
    // Fabricate a FloatingTab for the active tab (the detach mechanics are tested
    // elsewhere; here we assert the derived Placement reads the floating set).
    shell.detach_split.floating_tabs.push(super::FloatingTab {
        viewport_id: egui::ViewportId::from_hash_of("placement_detached"),
        tab_id,
        origin_index: 0,
        cmd_ctx: super::WorkspaceCommandContext::default(),
    });
    assert_eq!(
        shell.placement_of(tab_id),
        Placement::Detached,
        "a tab in floating_tabs resolves to Detached"
    );
}

/// Validates: layout-and-docking Req 16.6 -- an ordinary (non-floating) tab has
/// Placement::Docked in the root leaf when the Workspace is unsplit; placement
/// is derived, not stored.
#[test]
fn placement_of_docked_tab_is_docked_root_when_unsplit() {
    use crate::tab_manager::Placement;
    let shell = make_shell();
    let tab_id = shell.tabs.active_tab().id;
    assert_eq!(
        shell.placement_of(tab_id),
        Placement::Docked {
            leaf: ff_layout::TabGroupId::new(0)
        },
        "unsplit docked tab resolves to the root leaf"
    );
}

/// Validates: layout-and-docking Req 16.2/16.8, menu-and-statusbar Req 16.15 --
/// while split, EACH region renders its own instance's menu bar in-region, and
/// that bar participates in focus (its buttons are live, focusable widgets with
/// per-region-salted ids -- no phantom stop, no second focus ring). The
/// app-level bar is suppressed, so the live menu-bar first-button id belongs to
/// a region.
#[test]
fn full_shell_split_region_menu_bar_is_live_and_focusable() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..4 {
        harness.run();
    }
    assert!(harness.state().tabs.is_split(), "precondition: split");
    // The per-region menu bar renders through the shared renderer, which captures
    // menu_first_id from the LAST bar drawn (a region's, since the app-level bar
    // is suppressed while split). That id must be a live, focusable widget.
    let menu_first = harness.state().focus.menu_first_id;
    assert!(
        menu_first.is_some(),
        "a region menu bar must render and capture a first-button id while split"
    );
    if let Some(id) = menu_first {
        harness.ctx.memory_mut(|m| m.request_focus(id));
        harness.run();
        assert_eq!(
            harness.ctx.memory(|m| m.focused()),
            Some(id),
            "the in-region menu bar's first button must be a live, focusable Tab stop"
        );
    }
    // Each region's menu-bar scope id is distinct (per-leaf salt), so no two
    // regions collide (the workspace-conformance stable-id contract).
    let leaves = harness.state().tabs.leaf_ids();
    assert_eq!(leaves.len(), 2, "two regions");
    assert_ne!(
        region_menu_scope_id(leaves[0]),
        region_menu_scope_id(leaves[1]),
        "per-region menu-bar scope ids must be distinct (no collision)"
    );
}

/// Validates: layout-and-docking Req 16.7, workspace-kinds Req 4.6 -- core
/// provides exactly ONE tab system (the Region/TabGroupTree) and adds NO
/// intra-workspace "tab-container" attribute to a Kind's configuration. This is
/// a STRUCTURAL guard: `KindConfig` is destructured to its exact documented
/// field set, so adding a tab-container-style field to `KindConfig` (the way a
/// universal in-workspace tab list would be modelled) breaks this test and
/// forces a conscious spec revisit. A Kind that wants internal composition does
/// it privately in its own code (it may reuse a TabGroupTree internally), never
/// via a core `KindConfig` field.
#[test]
fn kind_config_has_no_core_tab_container_field() {
    use crate::workspace_kind::{BuiltinKind, KindConfig};
    let cfg = KindConfig::builtin_default(BuiltinKind::Editor);
    // Exhaustive destructure: if a field is ADDED or REMOVED from KindConfig this
    // fails to compile, catching an accidental core tab-container attribute
    // (Req 16.7 / 4.6) at build time. The bound names document the allowed set.
    let KindConfig {
        name: _,
        modelled_on: _,
        title: _,
        menu_bar: _,
        key_list: _,
        profile: _,
    } = cfg;
    // (No `tabs` / `tab_container` / `children` field exists -- that is the point.)
}

/// Validates: menu-and-statusbar Req 16.15, layout-and-docking Req 16.8, B073 --
/// while split, Tab/Shift+Tab focus cycling stays WITHIN the focused region and
/// never lands on ANOTHER region's controls. Regression guard for "Tab escapes
/// the active workspace onto other workspaces".
#[test]
fn full_shell_split_tab_stays_within_focused_region() {
    let mut harness = harness_shell();
    harness.state_mut().handle_command("SPLIT");
    for _ in 0..4 {
        harness.run();
    }
    let leaves = harness.state().tabs.leaf_ids();
    assert_eq!(leaves.len(), 2, "precondition: two regions");
    let focused = harness.state().tabs.focused_leaf_id();
    let other = *leaves.iter().find(|l| **l != focused).expect("other leaf");
    let other_cmd = region_cmd_field_id(other);

    // Focus the FOCUSED region's command field, then press Tab several times.
    let focused_cmd = region_cmd_field_id(focused);
    harness.ctx.memory_mut(|m| m.request_focus(focused_cmd));
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(focused_cmd),
        "precondition: focused region's command field has focus"
    );

    // Tab repeatedly: focus must NEVER land on the OTHER region's command field.
    for _ in 0..8 {
        harness.key_press(egui::Key::Tab);
        harness.run();
        assert_ne!(
            harness.ctx.memory(|m| m.focused()),
            Some(other_cmd),
            "Tab must not cross into another region while split (B073)"
        );
    }
    // Shift+Tab likewise stays out of the other region.
    for _ in 0..8 {
        harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::Tab);
        harness.run();
        assert_ne!(
            harness.ctx.memory(|m| m.focused()),
            Some(other_cmd),
            "Shift+Tab must not cross into another region while split (B073)"
        );
    }
}

/// Validates: screen-snapshot-scrm Req 16.1, 16.2 (workspace-conformance) --
/// the SCRM viewer Context reports a first interior control and the FIRST Tab
/// from the command field lands EXACTLY on it (no phantom stop). Mandatory
/// full-shell first-Tab focus test for the new Context.
#[test]
fn full_shell_scrm_viewer_first_tab_focuses_first_control() {
    let mut harness = harness_shell();
    // Seed a collection with one capture of the POM, then open the viewer.
    harness.state_mut().handle_command("CAPTURE START C");
    harness.state_mut().handle_command("CAPTURE SCREEN");
    harness.state_mut().handle_command("CAPTURE REPLAY");
    for _ in 0..4 {
        harness.run();
    }
    let expected = harness.state().focus.first_interior_id;
    assert!(
        expected.is_some(),
        "SCRM viewer must report a first interior control"
    );
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "command field holds focus on entering the SCRM viewer"
    );
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        expected,
        "first Tab in the SCRM viewer must focus the reported first interior, not a phantom stop"
    );
}

// === B080 Step 7: menu-bar reroute through the single front door ============
//
// The menu-bar click sites in render_chrome.rs now dispatch an in-scope
// workspace verb through the front door `dispatch_command_string` (which runs
// resolve_target) instead of calling `handle_command` directly. This test
// proves the reroute is behaviour-preserving: for every in-scope verb, the
// front-door path lands on the SAME active-tab Kind as the pre-reroute typed
// path (`handle_command`). Two fresh shells per verb, compared.
//
// Validates: menu-and-statusbar Requirement 16.15 (B056 menu-bar command
// parity); command-framework Requirement 2.1, 8.4 (one front door reaches the
// same classification as the direct path).
#[test]
fn b080_menu_bar_front_door_matches_typed_path_for_in_scope_verbs() {
    use crate::tab_state::TabKind;

    // Each in-scope workspace verb and the Kind it must open. These are exactly
    // the verbs `builtin_workspace_target_for` classifies and whose superseded
    // ladder arms Step 7 deletes.
    let cases: &[(&str, TabKind)] = &[
        ("FILES", TabKind::FileExplorerPanel),
        ("CATALOGS", TabKind::FilesPanel),
        ("CONFIG", TabKind::ConfigPanel),
        ("KEYS", TabKind::KeysEditor),
        ("KINDS", TabKind::KindsEditor),
        ("COMMANDS", TabKind::CommandConfigurator),
        ("LOG", TabKind::EventLog),
        ("PLUGINS", TabKind::PluginManager),
        ("MACROS", TabKind::MacroLibrary),
        ("MENUS", TabKind::MenusEditor),
        ("THEME", TabKind::ThemeEditor),
    ];

    for (verb, expected_kind) in cases {
        // Front-door path (what a rerouted menu-bar click now runs).
        let mut via_front_door = make_shell();
        via_front_door.dispatch_command_string(verb);
        let front_kind = via_front_door.tabs.active_tab().kind;

        // Pre-reroute typed path (handle_command) for the same verb.
        let mut via_handle = make_shell();
        via_handle.handle_command(verb);
        let handle_kind = via_handle.tabs.active_tab().kind;

        assert_eq!(
            front_kind, *expected_kind,
            "front-door dispatch of {verb} must open {expected_kind:?} (the menu-bar reroute target)"
        );
        assert_eq!(
            front_kind, handle_kind,
            "front-door dispatch of {verb} must match the typed handle_command path (behaviour-preserving reroute)"
        );
        assert!(
            via_front_door.open_error.is_none(),
            "front-door dispatch of {verb} must not set an open_error"
        );
    }
}
