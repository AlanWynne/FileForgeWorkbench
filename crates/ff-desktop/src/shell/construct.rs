//! # WorkbenchShell Construction
//!
//! The `WorkbenchShell` constructors (`new` and the `new_with_history_store`
//! injection seam), moved out of `mod.rs` verbatim as part of the Phase 2 task
//! 2.2 file-size split. Behaviour, signatures, visibility, and the exact field
//! initialisation order are unchanged.
//!
//! Validates: function-keys-and-history Requirement 6 (persistence seam);
//! notification-system Requirement 3.1, 3.3; CR-CH-025, CR-CH-028, CR-NR-090,
//! CR-NR-097

use std::sync::{Arc, Mutex};

use ff_command::CommandHistory as DispatchHistory;
use ff_command::{
    CommandDispatch, CommandId, CommandLineHistory, CommandMetadata, CommandRegistry,
};
use ff_command_semantics::CommandEngine;
use ff_config::ConfigHandle;
use ff_core::WorkbenchApp;
use ff_keys::{HistoryStore, KeyLabelBarModel, KeyMap, KeyMapResolver};
use ff_theme::ThemePalette;
use ff_zoom::{ZoomConfig, ZoomState};
use tokio::runtime::Runtime;

use crate::automation::ShellAutomationRegistry;
use crate::command_palette::CommandPaletteState;
use crate::config_panel::ConfigPanelState;
use crate::event_log_panel::EventLogPanelState;
use crate::exclude_manager::ExcludeManager;
use crate::files_panel::FilesPanelState;
use crate::find_manager::FindManager;
use crate::nav_manager::NavManager;
use crate::notification::{Notification, NotificationQueue};
use crate::plugin_manager_panel::PluginManagerPanelState;
use crate::scroll_amount::ScrollAmount;
use crate::session_manager::SessionManager;
use crate::tab_manager::TabManager;
use crate::toolchain_panel::ToolchainPanelState;

use super::handlers::{
    ConfigOpenHandler, FileExitHandler, FileOpenHandler, MenuOpenHandler, ShellContextProvider,
};
use super::mod_helpers::{
    ensure_keymaps_dir, load_context_maps_from_config, load_context_maps_from_keymaps_dir,
    resolve_history_path,
};
use super::{state_groups, KeyBarScope, WorkbenchShell};

impl WorkbenchShell {
    /// Construct the shell with an already-initialised `WorkbenchApp`.
    ///
    /// `cli_files` contains absolute paths collected from command-line arguments;
    /// they are opened as tabs on the first rendered frame. The command-line
    /// History_Store is resolved from `resolve_history_path()`; this delegates to
    /// `new_with_history_store` with that resolved store.
    pub fn new(
        app: WorkbenchApp,
        runtime: Runtime,
        palette: ThemePalette,
        cli_files: Vec<String>,
        config_handle: ConfigHandle,
    ) -> Self {
        // Production path: resolve the command-line History_Store from the
        // profile-aware UserDataDir (honouring the `FFWB_HISTORY_PATH` override).
        let history_store = resolve_history_path().map(HistoryStore::new);
        Self::new_with_history_store(
            app,
            runtime,
            palette,
            cli_files,
            config_handle,
            history_store,
        )
    }

    /// Construct the shell with an explicitly supplied command-line History_Store
    /// instead of resolving it from `resolve_history_path()`.
    ///
    /// This is the injection seam that lets tests point the history at a specific
    /// file DIRECTLY, rather than via the process-global `FFWB_HISTORY_PATH`
    /// environment variable. Threads cannot race on an injected value the way they
    /// race on a shared env var, so history-persistence tests built on this seam
    /// are robust under plain `cargo test` (shared-process, multi-thread), not only
    /// under `nextest` (process-per-test). Passing `None` yields an empty,
    /// non-persisting history. `new` delegates here with the resolved store.
    ///
    /// Validates: function-keys-and-history Requirement 6 (persistence seam)
    pub(crate) fn new_with_history_store(
        app: WorkbenchApp,
        runtime: Runtime,
        palette: ThemePalette,
        cli_files: Vec<String>,
        config_handle: ConfigHandle,
        history_store: Option<HistoryStore>,
    ) -> Self {
        let welcome = "Welcome to FileForge Workbench\n\nUse File > Open to open a file.\n";
        let tabs = TabManager::new(&runtime, welcome);

        let pending_open: Arc<std::sync::Mutex<Option<String>>> =
            Arc::new(std::sync::Mutex::new(None));
        let should_close: Arc<std::sync::Mutex<bool>> = Arc::new(std::sync::Mutex::new(false));

        let registry = Arc::new(CommandRegistry::new());
        let history = Arc::new(DispatchHistory::new(500));

        // Register file.open
        let open_id = CommandId::new("file.open").expect("valid id");
        let open_meta = CommandMetadata::builder("Open File", "Open a file from disk")
            .category("file")
            .build();
        registry
            .register(
                open_id,
                open_meta,
                Box::new(FileOpenHandler {
                    pending: pending_open.clone(),
                }),
            )
            .expect("file.open registration");

        // Register file.exit
        let exit_id = CommandId::new("file.exit").expect("valid id");
        let exit_meta = CommandMetadata::builder("Exit", "Exit the application")
            .category("file")
            .build();
        registry
            .register(
                exit_id,
                exit_meta,
                Box::new(FileExitHandler {
                    should_close: should_close.clone(),
                }),
            )
            .expect("file.exit registration");

        // Register menu.open (menu-workspace Requirement 11.6) -- marker handler;
        // the shell intercepts MENU / menu.open in handle_command.
        let menu_open_id = CommandId::new("menu.open").expect("valid id");
        let menu_open_meta = CommandMetadata::builder("Open Menu", "Open or return to a menu")
            .category("menu")
            .build();
        registry
            .register(menu_open_id, menu_open_meta, Box::new(MenuOpenHandler))
            .expect("menu.open registration");

        // Register config.open (CR-CH-025, configuration-system Requirement 20)
        // -- marker handler; the shell intercepts CONFIG in handle_command. Being
        // registered lets a bare `CONFIG` resolve as a built-in (chain stage 2),
        // shadowing any same-named menu/macro.
        let config_open_id = CommandId::new("config.open").expect("valid id");
        let config_open_meta = CommandMetadata::builder(
            "Configuration",
            "Browse all configuration keys (optionally filtered by namespace)",
        )
        .build();
        registry
            .register(
                config_open_id,
                config_open_meta,
                Box::new(ConfigOpenHandler),
            )
            .expect("config.open registration");

        let cmd_registry = registry.clone();
        let dispatch = CommandDispatch::new(registry, history);

        // CR-CH-028: wire the shell-backed Cursor_Context provider so commands
        // dispatched through the registry receive a populated package. The shell
        // refreshes `cursor_context_snapshot` at each dispatch; the provider
        // returns a clone of it (Requirement 12.4).
        let cursor_context_snapshot: Arc<Mutex<ff_command::CursorContext>> =
            Arc::new(Mutex::new(ff_command::CursorContext::empty()));
        dispatch.set_context_provider(Box::new(ShellContextProvider {
            snapshot: Arc::clone(&cursor_context_snapshot),
        }));

        let session = SessionManager::try_init();

        // Load user-defined commands from <User_Data_Dir>/commands/commands.toml.
        // An absent file yields an empty store (command-configurator Req 1.5).
        let command_store = {
            let path = dirs::data_dir()
                .map(|base| {
                    base.join("FileForgeWorkbench")
                        .join("commands")
                        .join("commands.toml")
                })
                .unwrap_or_else(|| std::path::PathBuf::from("commands/commands.toml"));
            crate::command_config::store::CommandStore::load(path)
        };

        // Load the Workspace Kind registry (CR-NR-090 B.1): compiled built-in
        // defaults plus any user Kinds under <User_Data_Dir>/workspace-kinds/.
        // An absent dir leaves just the built-in defaults.
        let kind_registry = {
            let dir = dirs::data_dir()
                .map(|base| base.join("FileForgeWorkbench").join("workspace-kinds"))
                .unwrap_or_else(|| std::path::PathBuf::from("workspace-kinds"));
            let reg = crate::workspace_kind::KindRegistry::load(&dir);
            // Surface any non-blocking load notices (unparseable user Kind files,
            // unresolved external bases) at WARN so a bad Workspace-Kind file is
            // discoverable from the logs (CR-NR-090 B.1).
            for notice in reg.notices() {
                ff_logging::log_warn!("{}", notice);
            }
            reg
        };

        // Load the Help Topic Registry ONCE (context-help Req 18.1, CR-NR-097):
        // the shipped `help/` `.help.md` content set. Searched, in order: the
        // exe-dir `help/` (installed layout), the exe-dir `../../help` (the dev
        // `target/<profile>/` -> workspace-root `help/` fallback), and the
        // user-data `help/`. An absent directory leaves an empty registry (F1
        // then reports "help not yet available" and records the miss).
        let help_registry = {
            let mut search_paths: Vec<std::path::PathBuf> = Vec::new();
            if let Ok(exe) = std::env::current_exe() {
                if let Some(exe_dir) = exe.parent() {
                    search_paths.push(exe_dir.join("help"));
                    search_paths.push(exe_dir.join("..").join("..").join("help"));
                }
            }
            if let Some(data_dir) = dirs::data_dir() {
                search_paths.push(data_dir.join("FileForgeWorkbench").join("help"));
            }
            let registry = ff_help::HelpTopicRegistry::new();
            let loader = ff_help::ContentLoader::new(search_paths);
            match loader.load_all() {
                Ok(result) => {
                    for (path, warning) in &result.warnings {
                        ff_logging::log_warn!(
                            "[help] content: failed to parse {} -- {}",
                            path.display(),
                            warning
                        );
                    }
                    registry.load_file_topics(result.topics);
                }
                Err(e) => {
                    ff_logging::log_warn!("[help] content: {}", e);
                }
            }
            std::sync::Arc::new(registry)
        };

        // Notification channel -- Validates: notification-system Requirement 3.1, 3.3
        let (notification_tx, notification_rx) = std::sync::mpsc::sync_channel::<Notification>(64);
        let notification_queue =
            std::sync::Arc::new(std::sync::Mutex::new(NotificationQueue::new()));

        // Build the default global key map using the built-in defaults.
        let global_map = KeyMap::default_global();
        let key_label_bar = KeyLabelBarModel::from_key_map(&global_map);
        let mut key_map_resolver = KeyMapResolver::new(global_map);
        // Load [context_key_maps] from config at startup -- Validates: Requirement 14.7
        load_context_maps_from_config(&config_handle, &mut key_map_resolver);
        // Then load keymaps/<context>.toml override FILES, which take precedence
        // over the config-table entries (loaded second) -- Validates:
        // function-keys-and-history Requirement 14.9-14.12 (CR-CH-027).
        {
            let user_data_dir = ff_session::UserDataDir::resolve(None)
                .map(|udd| udd.path().to_path_buf())
                .unwrap_or_else(|_| {
                    dirs::data_dir()
                        .map(|base| base.join("FileForgeWorkbench"))
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                });
            ensure_keymaps_dir(&user_data_dir);
            load_context_maps_from_keymaps_dir(
                &user_data_dir.join("keymaps"),
                &mut key_map_resolver,
            );
        }

        // Command-line history persistence (function-keys-and-history Req 6):
        // load any persisted history from the supplied store and seed the owner.
        // Missing/corrupt file -> empty history, no failure (Req 6.5, 6.6).
        let mut command_line_history = CommandLineHistory::new(500);
        if let Some(store) = &history_store {
            let (ring, warnings) = store.load(500);
            for w in warnings {
                ff_logging::log_warn!("[keys] command history load: {}: {}", w.field, w.message);
            }
            command_line_history.load_command_strings(ring.to_command_strings());
        }

        Self {
            app,
            runtime,
            palette,
            tabs,
            command_text: String::new(),
            pending_command_line_outcome: None,
            started: false,
            cli_files,
            pending_open,
            should_close,
            open_error: None,
            dispatch,
            cmd_registry,
            cmd_engine: CommandEngine::new(),
            command_line_history,
            command_line_in_progress: None,
            history_store,
            find_manager: FindManager::new(),
            nav_manager: NavManager::new(),
            exclude_manager: ExcludeManager::new(),
            key_map_resolver,
            key_label_bar,
            key_bar_visible: true,
            key_bar_scope: KeyBarScope::Base,
            tab_history: Vec::new(),
            show_history_list: None,
            show_swap_list: None,
            session,
            active_workspace: None,
            pending_workspace_open: None,
            show_unsaved_workspace_dialog: false,
            config_handle,
            command_store,
            kind_registry,
            command_configurator_panel:
                crate::command_config::render::CommandConfiguratorState::new(),
            theme_editor_panel: crate::theme_editor_panel::ThemeEditorState::new(),
            menus_editor_panel: crate::menus_editor_panel::MenusEditorState::new(),
            keys_editor_panel: crate::keys_editor_panel::KeysEditorState::new(),
            kinds_editor_panel: crate::kinds_editor_panel::KindsEditorState::default(),
            shell_engine: ff_shell::ShellEngine::new(ff_shell::ShellConfigProvider::new()),
            pending_external: None,
            files_panel: FilesPanelState::new(),
            file_explorer_panel_width: 260.0,
            nav_model: crate::nav_model::NavModel::new(),
            nav_selection: crate::explorer_view::ExplorerSelection::default(),
            nav_rename: None,
            nav_delete: None,
            nav_new: None,
            nav_focused: false,
            nav_file_clipboard: Vec::new(),
            toolchain_panel: ToolchainPanelState::new(),
            show_toolchain_panel: false,
            pending_new_pom: false,
            pending_new_file: false,
            pending_return_to_pom: false,
            pending_menu_option: None,
            zoom: ZoomState::new(&ZoomConfig::default()),
            pom_calendar_offset: 0,
            cursor_context_snapshot,
            focused_menu_option: None,
            active_theme_file: None,
            dir_overrides: state_groups::DirOverrides::default(),
            last_ppp: 1.0,
            is_dragging: false,
            pending_ppp: None,
            show_about: false,
            palette_state: CommandPaletteState::default(),
            recent_palette_commands: Vec::new(),
            search_results_panel: crate::search_results_panel::SearchResultsPanelState::new(),
            scroll_amount: ScrollAmount::default(),
            scroll_field_text: "PAGE".to_string(),
            modal_open: false,
            reset_bare_confirm: None,
            reset_bare_focus_requested: false,
            help_context_panel: crate::help_context::HelpContextPanel::new(
                help_registry.clone(),
                ff_help::HelpConfig::default(),
            ),
            help_registry,
            help_missing_tally: std::collections::HashMap::new(),
            config_panel: ConfigPanelState::new(),
            plugin_manager_panel: PluginManagerPanelState::new(),
            macro_library_panel: crate::macro_library_panel::MacroLibraryPanelState::new(),
            event_log_panel: EventLogPanelState::new(),
            scrm: crate::scrm_session::ScrmSession::default(),
            scrm_viewer: crate::scrm_viewer_panel::ScrmViewerState::default(),
            notification_rx,
            notification_tx,
            notification_queue,
            focus: state_groups::FocusState {
                command_field_focus_requested: true,
                ..state_groups::FocusState::default()
            },
            automation: ShellAutomationRegistry::new(),
            detach_split: state_groups::DetachSplitState::default(),
            session_start: chrono::Local::now(),
        }
    }
}
