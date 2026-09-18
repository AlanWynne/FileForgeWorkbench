//! # WorkbenchShell — egui/eframe Rendering Shell
//!
//! Sole point of contact between egui/eframe and the platform-core layer.
//! Implements `eframe::App` and owns the `TabManager` (all open tabs),
//! the Tokio runtime, and the active theme palette.

use std::sync::{Arc, Mutex};

use eframe::egui;
use ff_command::CommandHistory as DispatchHistory;
use ff_command::{
    CommandDispatch, CommandHandler, CommandId, CommandMetadata, CommandParams, CommandRegistry,
    CommandResult, ExecutionContext,
};
use ff_command_semantics::CommandEngine;
use ff_config::ConfigHandle;
use ff_core::WorkbenchApp;
use ff_keys::{CommandHistory, KeyLabelBarModel, KeyMap, KeyMapResolver, RetrieveState};
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
use crate::notification::{Notification, NotificationQueue, NotificationSender};
use crate::plugin_manager_panel::PluginManagerPanelState;
pub(crate) use crate::scroll_amount::{ScrollAmount, SplitScreenState};
use crate::session_manager::SessionManager;
use crate::tab_manager::TabManager;
use crate::toolchain_panel::ToolchainPanelState;
use ff_session::{load_workspace, save_workspace, WorkspaceState};

// ── Built-in command handlers ────────────────────────────────────────────────

/// Handler for `file.open` — sets `pending_open` via a shared channel.
/// The shell reads `pending_open` at the top of each frame.
struct FileOpenHandler {
    pending: Arc<std::sync::Mutex<Option<String>>>,
}

impl CommandHandler for FileOpenHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&self, _ctx: &ExecutionContext, params: &CommandParams) -> CommandResult {
        match params.get_string("path") {
            Some(path) if !path.is_empty() => {
                *self.pending.lock().expect("pending lock") = Some(path.to_string());
                CommandResult::Ok
            }
            _ => CommandResult::Err(ff_command::CommandError::ExecutionFailed {
                id: "file.open".to_string(),
                description: "missing or empty 'path' parameter".to_string(),
            }),
        }
    }
}

/// Handler for `file.exit` — sets a shared close flag.
struct FileExitHandler {
    should_close: Arc<std::sync::Mutex<bool>>,
}

impl CommandHandler for FileExitHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
        *self.should_close.lock().expect("close lock") = true;
        CommandResult::Ok
    }
}

/// Handler for `menu.open` -- a marker registration so the id is dispatchable
/// and palette-visible (menu-workspace Requirement 11.6). The actual
/// menu-opening is performed by the shell, which intercepts `MENU` /
/// `menu.open` in `handle_command` before registry dispatch.
struct MenuOpenHandler;

/// A [`ContextProvider`](ff_command::ContextProvider) backed by the shell's live
/// Cursor_Context snapshot (CR-CH-028, command-framework Requirement 12.4).
///
/// The shell refreshes the shared `snapshot` cell from live focus/selection at
/// each dispatch; this provider returns a fresh `ExecutionContext` carrying a
/// clone of that snapshot, so a command dispatched through the registry receives
/// the SAME package the string-path commands see (command parity, Requirement
/// 12.4). `current_context` takes `&self`, hence the shared cell.
struct ShellContextProvider {
    snapshot: Arc<Mutex<ff_command::CursorContext>>,
}

impl ff_command::ContextProvider for ShellContextProvider {
    fn current_context(&self) -> ExecutionContext {
        let cc = self.snapshot.lock().map(|g| g.clone()).unwrap_or_default();
        ExecutionContext::builder().cursor_context(cc).build()
    }
}

impl CommandHandler for MenuOpenHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
        // Routing is handled by the shell intercept; nothing to do here.
        CommandResult::Ok
    }
}

/// Marker handler for `config.open` (CR-CH-025). Registration makes bare
/// `CONFIG` resolve as a built-in command through the resolution chain; the
/// actual opening of the flat config-key view is performed by the shell
/// intercept in `handle_command`.
struct ConfigOpenHandler;

impl CommandHandler for ConfigOpenHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
        // Routing is handled by the shell intercept; nothing to do here.
        CommandResult::Ok
    }
}

// ── Menu bar (data-driven) — Validates: menu-workspace Req 17.1 ────────────
// CR-NR-080 (menu-workspace Req 17.1): the menu bar is now DATA-DRIVEN, rendered
// from the compiled default Menu_Bar (`menu_workspace::defaults::default_menubar_menu`)
// rather than a hardcoded label list. The former `MENU_BAR_TOP_LEVEL_LABELS`
// const (and its `debug_assert_eq!` in `render_menu_bar`) were removed; the
// bar's top-level entries are the default Menu_Bar's option `description`s, and
// tests assert against `default_menubar_menu()` instead.

// ── Detachable tab windows — Validates: Requirement 18.1–18.7 ──────────────

/// Tracks a tab that has been detached into a floating OS window.
///
/// Validates: Requirement 18.1, 18.2, 18.3
#[allow(dead_code)]
pub(crate) struct FloatingTab {
    /// egui viewport id allocated for this floating window.
    pub viewport_id: egui::ViewportId,
    /// Current index of this tab in `TabManager` (kept in sync on redock).
    pub tab_index: usize,
    /// The tab index at the moment of detach — used to restore position on redock.
    pub origin_index: usize,
}

// ── WorkbenchShell ───────────────────────────────────────────────────────────

/// The egui/eframe application shell.
pub struct WorkbenchShell {
    app: WorkbenchApp,
    runtime: Runtime,
    palette: ThemePalette,
    tabs: TabManager,
    /// ISPF-style command field text.
    command_text: String,
    /// One-shot startup flag.
    started: bool,
    /// Files to open on the first frame (from CLI arguments).
    cli_files: Vec<String>,
    /// When Some, open this path at the start of the next frame.
    pending_open: Arc<std::sync::Mutex<Option<String>>>,
    /// Set to true by the file.exit handler; checked in update().
    should_close: Arc<std::sync::Mutex<bool>>,
    /// Error message to display in the status bar (cleared on next open).
    open_error: Option<String>,
    /// Command dispatch — routes file.open / file.exit through the registry.
    dispatch: CommandDispatch,
    /// The per-invocation Cursor_Context snapshot (CR-CH-028, command-framework
    /// Requirement 12). The shell refreshes this from live focus/selection at
    /// each dispatch; the registered `ShellContextProvider` holds an `Arc` clone
    /// and returns it so commands dispatched through the registry receive a
    /// populated package. A shared cell is required because `ContextProvider`
    /// takes `&self`.
    cursor_context_snapshot: Arc<Mutex<ff_command::CursorContext>>,
    /// The focused Menu_Option recorded during the last menu render (its egui id,
    /// command, and label), so a focused id resolves to its semantic identity
    /// when building the Cursor_Context (Requirement 12.2). `None` when no menu
    /// option is focused.
    focused_menu_option: Option<(egui::Id, String, String)>,
    /// Shared command registry — used by the Command Palette to enumerate commands.
    ///
    /// Validates: command-palette Requirement 2.1
    cmd_registry: Arc<CommandRegistry>,
    /// ISPF command semantics engine — parses and executes primary commands.
    cmd_engine: CommandEngine,
    /// Command history for RETRIEVE cycling.
    cmd_history: CommandHistory,
    /// RETRIEVE pointer state.
    retrieve_state: RetrieveState,
    /// Find/replace engine — FIND, RFIND, CHANGE, RCHANGE.
    find_manager: FindManager,
    /// Navigation engine — LOCATE, SORT, UP, DOWN, LEFT, RIGHT, TOP, BOTTOM.
    nav_manager: NavManager,
    /// Exclude/Show/Reset engine — EXCLUDE, SHOW, RESET.
    exclude_manager: ExcludeManager,
    /// Active key map resolver — global key map (no profile active at startup).
    key_map_resolver: KeyMapResolver,
    /// Key label bar display model — derived from the active key map.
    key_label_bar: KeyLabelBarModel,
    /// Whether the Key Label Bar is currently visible.
    ///
    /// Validates: Requirement 12.4
    key_bar_visible: bool,
    /// History of previously active tab indices for END navigation.
    ///
    /// Validates: Requirement 17.1
    tab_history: Vec<usize>,
    /// When Some, show the history-list overlay with these entries.
    ///
    /// Validates: Requirement 19.3
    show_history_list: Option<Vec<String>>,
    /// When Some, show the SWAP tab-picker overlay (multi-tab-editor Req 18.3).
    /// The unit payload keeps the state a simple open/closed flag; the picker
    /// reads the live tab list at render time.
    show_swap_list: Option<()>,
    /// Session persistence -- None when User Data Dir is unavailable.
    session: Option<SessionManager>,
    /// Active workspace -- None when no workspace is loaded.
    ///
    /// Validates: workspace-model Requirement 2.6
    pub(crate) active_workspace: Option<WorkspaceState>,
    /// Pending workspace path deferred while the unsaved-changes dialog is open.
    ///
    /// Validates: workspace-model Requirement 2.5
    pub(crate) pending_workspace_open: Option<std::path::PathBuf>,
    /// Whether the unsaved-workspace-changes dialog is currently open.
    ///
    /// Validates: workspace-model Requirement 2.5
    pub(crate) show_unsaved_workspace_dialog: bool,
    /// Configuration handle — used to read catalog default paths.
    config_handle: ConfigHandle,
    /// User-defined command definitions (`commands/commands.toml`).
    ///
    /// Feeds Target_Resolution so a menu option or keyboard shortcut whose
    /// command value equals a definition id runs that definition's target.
    ///
    /// Validates: command-configurator Requirement 1, 4.3; menu-workspace
    /// Requirement 10.3
    pub(crate) command_store: crate::command_config::store::CommandStore,
    /// Command Configurator Context UI state (list + edit form + delete confirm).
    ///
    /// Validates: command-configurator Requirement 2.1, 2.3
    pub(crate) command_configurator_panel: crate::command_config::render::CommandConfiguratorState,

    /// Theme Editor Context state (CR-NR-074).
    ///
    /// Validates: theme-and-appearance Requirement 20.1
    pub(crate) theme_editor_panel: crate::theme_editor_panel::ThemeEditorState,
    /// Menus Editor Context state (menu-workspace Req 13, CR-NR-075).
    pub(crate) menus_editor_panel: crate::menus_editor_panel::MenusEditorState,
    /// Keys Editor Context state (function-keys Req 22, CR-CH-029).
    pub(crate) keys_editor_panel: crate::keys_editor_panel::KeysEditorState,
    /// Shell engine for external program execution (Detached / Captured).
    ///
    /// Backs the External Command_Target adapter: runs a program by name +
    /// argument list, gated by `shell.mode`, with captured output routed to the
    /// Output_Panel.
    ///
    /// Validates: command-configurator Requirement 3; shell-command Requirement 19
    pub(crate) shell_engine: ff_shell::ShellEngine,
    /// Pending External target awaiting a shell.mode = prompt confirmation.
    ///
    /// Validates: command-configurator Requirement 3.8
    pub(crate) pending_external: Option<crate::shell::external_adapter::PendingExternal>,
    /// Files Panel (Virtual Catalog Manager) state.
    files_panel: FilesPanelState,
    /// Persisted width of the File Explorer side panel (logical pixels).
    /// Session-persisted via `file_explorer_sidebar_width` (ff-session).
    ///
    /// Validates: Requirement 1.3 file-tree-panel (fix B019), 23.9
    file_explorer_panel_width: f32,
    /// Canonical ff-file-tree-backed navigation model for the File Explorer
    /// (CR-NR-060 Slice A). Identity is NodeId + ResourceUri (Req 24.1, 24.2).
    nav_model: crate::nav_model::NavModel,
    /// Selection/cursor state for the NavModel-backed explorer (Requirement 24.2).
    nav_selection: crate::explorer_view::ExplorerSelection,

    /// Active rename dialog for the modern explorer: (target node, edit buffer).
    /// `None` when no rename is in progress. (CR-NR-060 Slice A, Req 16 Rename.)
    nav_rename: Option<(ff_file_tree::NodeId, String)>,

    /// Active delete-confirmation dialog for the modern explorer: (target node,
    /// display label). `None` when no delete is pending. (Req 16 Delete.)
    nav_delete: Option<(ff_file_tree::NodeId, String)>,

    /// Active new-child dialog for the modern explorer: (parent directory node,
    /// is_directory, name buffer). `None` when none pending. (Req 16 New.)
    nav_new: Option<(ff_file_tree::NodeId, bool, String)>,

    /// When true, the modern explorer node list holds keyboard focus (Tab moved
    /// focus from the shell Command ===> into the tree). (Req 24.9 / 20.1.)
    nav_focused: bool,

    /// File clipboard for the modern explorer: source resource URIs marked for a
    /// copy, pasted into a target directory on Paste. (Req 21.1 / 21.3.)
    nav_file_clipboard: Vec<ff_vfs::ResourceUri>,
    /// Toolchain panel state — GCC and Rust plugin entries.
    toolchain_panel: ToolchainPanelState,
    /// Whether the Toolchain Panel is currently visible.
    show_toolchain_panel: bool,
    /// Deferred: open a new POM tab on the next frame (set by tab-bar context menu).
    pending_new_pom: bool,
    /// Deferred: open a new untitled tab on the next frame (set by tab-bar context menu).
    pending_new_file: bool,
    /// Deferred: return the active FilesPanel tab to POM view (set by F3/END in Files Panel).
    pending_return_to_pom: bool,
    /// Deferred: option to execute from a Menu_Workspace option click (set by
    /// render, processed in update). Carries the full option so an inline
    /// `[options.target]` is honoured on click as well as by typing.
    ///
    /// Validates: menu-workspace Requirement 3.2, 10.6
    pub(crate) pending_menu_option: Option<crate::menu_workspace::MenuOption>,
    /// Global application zoom — single level shared across all tabs and panels.
    ///
    /// Addresses: Requirement 3.1 (view-zoom) — zoom carries forward across context switches.
    zoom: ZoomState,
    /// Month offset for the POM calendar (0 = current month, -1 = prev, +1 = next).
    ///
    /// Validates: Requirement 14.42
    pom_calendar_offset: i32,

    /// The active theme's backing file path and its last-seen modification time,
    /// used to hot-reload the palette when the file changes on disk (CR-NR-074).
    /// `None` when the active theme is not file-backed (e.g. a compiled built-in
    /// with no on-disk file yet).
    ///
    /// Validates: theme-and-appearance Requirement 19.6
    active_theme_file: Option<(std::path::PathBuf, std::time::SystemTime)>,

    /// Test-only override for the themes directory. Production leaves this
    /// `None` (the real `<User_Data_Dir>/themes/`); tests set it to a TempDir so
    /// Theme editor file operations are deterministic and isolated.
    themes_dir_override: Option<std::path::PathBuf>,

    /// Test-only override for the menus directory (menu-workspace Req 13,
    /// CR-NR-075). Production leaves this `None` (the real
    /// `<User_Data_Dir>/menus/`); tests set it to a TempDir so Menus editor file
    /// operations are deterministic and isolated.
    menus_dir_override: Option<std::path::PathBuf>,
    /// Test-only override for the keymaps directory (function-keys Req 22,
    /// CR-CH-029). Production leaves this `None` (the real
    /// `<User_Data_Dir>/keymaps/`); tests set it to a TempDir so Keys editor
    /// file operations are deterministic and isolated.
    keymaps_dir_override: Option<std::path::PathBuf>,
    /// Last pixels_per_point applied by zoom — avoids overwriting OS DPI every frame.
    last_ppp: f32,
    /// True while the user is holding the mouse button down (window drag in progress).
    /// DPI-driven resize is suppressed during a drag to prevent mid-move stuttering.
    is_dragging: bool,
    /// Pending pixels_per_point to apply once the drag is released.
    pending_ppp: Option<f32>,
    /// Whether the Help > About dialog is currently open.
    ///
    /// Validates: Requirement 13.1
    show_about: bool,
    /// Command Palette state -- open/closed, query, filtered list, selection.
    ///
    /// Validates: command-palette Requirement 1.1, 4.3
    pub(crate) palette_state: CommandPaletteState,
    /// Recently-used command IDs executed via the palette (most recent first, max 10).
    ///
    /// Validates: command-palette Requirement 5.1, 5.2
    pub(crate) recent_palette_commands: Vec<String>,
    /// Global Search Results panel state.
    ///
    /// Validates: global-search Requirement 1.1
    pub(crate) search_results_panel: crate::search_results_panel::SearchResultsPanelState,
    /// Active scroll amount for the SCROLL ===> field.
    ///
    /// Validates: Requirement 19.1, 19.2, 19.3
    pub(crate) scroll_amount: ScrollAmount,
    /// Split screen state -- Some when split screen is active.
    ///
    /// Validates: Requirement 19.11, 19.12, 19.13, 19.14
    pub(crate) split_screen: Option<SplitScreenState>,
    /// Text buffer for the SCROLL ===> field input.
    ///
    /// Validates: Requirement 19.1
    pub(crate) scroll_field_text: String,
    /// True when any modal dialog is open — suppresses the shell Tab-cycle and
    /// command-field focus steal so keystrokes reach the dialog's own widgets.
    modal_open: bool,
    /// True when the RESET BARE confirmation dialog is open (CR-CH-021,
    /// configuration-system Req 19.2). No configuration is archived or reset
    /// until the user confirms.
    reset_bare_confirm_open: bool,
    /// Config Panel state (the flat config-key browser opened by `CONFIG`).
    ///
    /// Validates: Requirement 15.1, 15.2
    config_panel: ConfigPanelState,
    /// Plugin Manager panel state.
    ///
    /// Validates: plugin-manager-ui Requirement 1.1
    plugin_manager_panel: PluginManagerPanelState,
    /// Macro Library panel state.
    ///
    /// Validates: lua-macro-engine Requirement 12.1
    macro_library_panel: crate::macro_library_panel::MacroLibraryPanelState,
    /// Event Log panel state.
    ///
    /// Validates: notification-system Requirement 2.2
    event_log_panel: EventLogPanelState,
    /// Notification channel receiver -- drained each frame.
    ///
    /// Validates: notification-system Requirement 1.1
    notification_rx: std::sync::mpsc::Receiver<Notification>,
    /// Notification channel sender -- cloned for background tasks.
    ///
    /// Validates: notification-system Requirement 3.1
    #[allow(dead_code)]
    notification_tx: std::sync::mpsc::SyncSender<Notification>,
    /// Shared notification queue -- drained each frame from the channel.
    ///
    /// Validates: notification-system Requirement 1.1
    pub(crate) notification_queue: std::sync::Arc<std::sync::Mutex<NotificationQueue>>,
    /// True for exactly one frame after Tab/Shift+Tab moves focus back to the
    /// command field, on the startup frame, or on any Workspace entry. Causes a
    /// one-shot request_focus on the command field. Cleared immediately after
    /// the request fires so we do NOT steal focus every frame.
    ///
    /// Validates: Requirement 16.1, 16.1a, 16.2
    command_field_focus_requested: bool,
    /// Egui id of the FIRST focusable interior control of the active Workspace,
    /// reported by the Workspace renderer each frame. The shell Boundary_Policy
    /// focuses it when Tab is pressed from the command field. `None` when the
    /// Workspace has no interior control (focus goes straight to the menu bar).
    ///
    /// Validates: Requirement 16.3, 16.14 (CR-CH-023)
    first_interior_id: Option<egui::Id>,
    /// Egui id of the LAST focusable interior control of the active Workspace.
    /// The shell Boundary_Policy moves focus from it to the first Menu_Bar item
    /// on Tab (menu-bar-last).
    ///
    /// Validates: Requirement 16.5, 16.14 (CR-CH-023)
    last_interior_id: Option<egui::Id>,
    /// Egui id of the FIRST top-level Menu_Bar button ("Settings"), captured by
    /// the menu-bar render for the Boundary_Policy (last-interior -> menu bar).
    ///
    /// Validates: Requirement 16.5, 16.7 (CR-CH-023)
    menu_first_id: Option<egui::Id>,
    /// Egui id of the LAST top-level Menu_Bar button ("Help"), captured by the
    /// menu-bar render for the Boundary_Policy (last-menu -> wrap to command).
    ///
    /// Validates: Requirement 16.7, 16.8 (CR-CH-023)
    menu_last_id: Option<egui::Id>,
    /// Active tab index observed on the previous frame. When it changes, the
    /// shell re-arms command-field focus (Req 16.1a: entering a Workspace via a
    /// tab switch places focus on the command field).
    last_active_tab: usize,
    /// One-shot latch: set when Tab is pressed from the command field so the
    /// active Workspace render focuses its FIRST interior control using the id
    /// it allocates THIS frame (a stale previous-frame id does not round-trip
    /// through egui focus; requesting the fresh same-frame id does -- B056).
    /// Cleared by the render that honours it.
    ///
    /// Validates: Requirement 16.3 (CR-CH-023)
    pub(crate) focus_first_interior_requested: bool,
    /// One-shot latch (reverse Shift+Tab): focus the active Workspace's LAST
    /// interior control using the fresh same-frame id.
    ///
    /// Validates: Requirement 16.8 (CR-CH-023)
    pub(crate) focus_last_interior_requested: bool,
    /// Session start timestamp -- recorded when the shell is created.
    ///
    /// Validates: Requirement 20.1, 20.2
    pub(crate) session_start: chrono::DateTime<chrono::Local>,
    /// Automation registry -- exposes control state to the FFTest runner.
    ///
    /// Validates: Requirement 2.1, 2.5 (automated-dialog-testing)
    pub(crate) automation: ShellAutomationRegistry,
    /// All currently floating (detached) tabs.
    ///
    /// Validates: Requirement 18.1, 18.2
    #[allow(dead_code)]
    floating_tabs: Vec<FloatingTab>,
    /// Index of the tab to detach on the next frame (set by context menu).
    ///
    /// Validates: Requirement 18.2
    #[allow(dead_code)]
    detach_pending: Option<usize>,
    /// Origin indices of floating tabs that have been closed and need redocking.
    ///
    /// Written by the floating viewport's close callback; read by the primary frame.
    /// Validates: Requirement 18.3
    #[allow(dead_code)]
    redock_pending: Arc<Mutex<Vec<usize>>>,
}

impl WorkbenchShell {
    /// Construct the shell with an already-initialised `WorkbenchApp`.
    ///
    /// `cli_files` contains absolute paths collected from command-line arguments;
    /// they are opened as tabs on the first rendered frame.
    pub fn new(
        app: WorkbenchApp,
        runtime: Runtime,
        palette: ThemePalette,
        cli_files: Vec<String>,
        config_handle: ConfigHandle,
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

        // Notification channel -- Validates: notification-system Requirement 3.1, 3.3
        let (notification_tx, notification_rx) = std::sync::mpsc::sync_channel::<Notification>(64);
        let notification_queue =
            std::sync::Arc::new(std::sync::Mutex::new(NotificationQueue::new()));

        // Build the default global key map using the built-in defaults.
        let global_map = KeyMap::default_global();
        let key_label_bar = KeyLabelBarModel::from_key_map(&global_map);
        let mut key_map_resolver = KeyMapResolver::new(global_map);
        // Load [context_key_maps] from config at startup — Validates: Requirement 14.7
        load_context_maps_from_config(&config_handle, &mut key_map_resolver);
        // Then load keymaps/<context>.toml override FILES, which take precedence
        // over the config-table entries (loaded second) — Validates:
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

        Self {
            app,
            runtime,
            palette,
            tabs,
            command_text: String::new(),
            started: false,
            cli_files,
            pending_open,
            should_close,
            open_error: None,
            dispatch,
            cmd_registry,
            cmd_engine: CommandEngine::new(),
            cmd_history: CommandHistory::new(500),
            retrieve_state: RetrieveState::new(),
            find_manager: FindManager::new(),
            nav_manager: NavManager::new(),
            exclude_manager: ExcludeManager::new(),
            key_map_resolver,
            key_label_bar,
            key_bar_visible: true,
            tab_history: Vec::new(),
            show_history_list: None,
            show_swap_list: None,
            session,
            active_workspace: None,
            pending_workspace_open: None,
            show_unsaved_workspace_dialog: false,
            config_handle,
            command_store,
            command_configurator_panel:
                crate::command_config::render::CommandConfiguratorState::new(),
            theme_editor_panel: crate::theme_editor_panel::ThemeEditorState::new(),
            menus_editor_panel: crate::menus_editor_panel::MenusEditorState::new(),
            keys_editor_panel: crate::keys_editor_panel::KeysEditorState::new(),
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
            themes_dir_override: None,
            menus_dir_override: None,
            keymaps_dir_override: None,
            last_ppp: 1.0,
            is_dragging: false,
            pending_ppp: None,
            show_about: false,
            palette_state: CommandPaletteState::default(),
            recent_palette_commands: Vec::new(),
            search_results_panel: crate::search_results_panel::SearchResultsPanelState::new(),
            scroll_amount: ScrollAmount::default(),
            split_screen: None,
            scroll_field_text: "PAGE".to_string(),
            modal_open: false,
            reset_bare_confirm_open: false,
            config_panel: ConfigPanelState::new(),
            plugin_manager_panel: PluginManagerPanelState::new(),
            macro_library_panel: crate::macro_library_panel::MacroLibraryPanelState::new(),
            event_log_panel: EventLogPanelState::new(),
            notification_rx,
            notification_tx,
            notification_queue,
            command_field_focus_requested: true,
            first_interior_id: None,
            last_interior_id: None,
            menu_first_id: None,
            menu_last_id: None,
            last_active_tab: 0,
            focus_first_interior_requested: false,
            focus_last_interior_requested: false,
            automation: ShellAutomationRegistry::new(),
            floating_tabs: Vec::new(),
            detach_pending: None,
            redock_pending: Arc::new(Mutex::new(Vec::new())),
            session_start: chrono::Local::now(),
        }
    }

    // ── Theme ────────────────────────────────────────────────────────────

    // render methods are in render.rs

    /// Return a cloned `NotificationSender` for use by background tasks.
    ///
    /// Validates: notification-system Requirement 3.1
    #[allow(dead_code)]
    pub fn notification_sender(&self) -> NotificationSender {
        NotificationSender::new(self.notification_tx.clone())
    }

    /// Return the list of directories to scan for Lua macro files.
    ///
    /// Validates: lua-macro-engine Requirement 12.2
    pub(crate) fn macro_dirs(&self) -> Vec<String> {
        if let Some(base) = dirs::data_dir() {
            let dir = base.join("FileForgeWorkbench").join("macros");
            vec![dir.to_string_lossy().into_owned()]
        } else {
            Vec::new()
        }
    }

    // ── Session lifecycle helpers — Validates: Requirement 20.1, 20.2 ────

    /// Format the session start time as `Started: HH:MM`.
    ///
    /// Validates: Requirement 20.1
    pub(crate) fn format_session_start(&self) -> String {
        format!("Started: {}", self.session_start.format("%H:%M"))
    }

    /// The Active_Profile label for the Status_Bar / Title_Line (CR-NR-081,
    /// startup-and-session Requirement 22.8). Shows the running Application_Profile
    /// name, or `Profile: default` when no `--profile` was given.
    ///
    /// Validates: startup-and-session Requirement 22.8
    pub(crate) fn active_profile_label(&self) -> String {
        match ff_session::active_profile() {
            Some(name) => format!("Profile: {name}"),
            None => "Profile: default".to_string(),
        }
    }

    /// Format the logoff message as `Logoff at HH:MM -- session duration: Xm Ys`.
    ///
    /// Validates: Requirement 20.2
    pub(crate) fn format_logoff_message(&self) -> String {
        let now = chrono::Local::now();
        let duration = now.signed_duration_since(self.session_start);
        let total_secs = duration.num_seconds().max(0) as u64;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!(
            "Logoff at {} -- session duration: {}m {}s",
            now.format("%H:%M"),
            mins,
            secs
        )
    }

    // ── Workspace lifecycle helpers -- Validates: workspace-model Req 2, 3, 4, 6 ──

    /// Load a workspace from `path`, register its roots as Native catalogs,
    /// inject its settings layer, and restore its MRU list.
    ///
    /// If a modified workspace is already active, defers the open and shows the
    /// unsaved-changes dialog instead.
    ///
    /// Validates: workspace-model Requirement 2.1, 2.5, 3.4, 4.1, 6.1
    pub(crate) fn open_workspace(&mut self, path: &std::path::Path) {
        // Validates: Requirement 2.5 -- prompt before discarding unsaved changes.
        if let Some(ws) = &self.active_workspace {
            if ws.is_modified {
                self.pending_workspace_open = Some(path.to_path_buf());
                self.show_unsaved_workspace_dialog = true;
                return;
            }
        }
        self.open_workspace_force(path);
    }

    /// Open a workspace unconditionally, closing any existing one first.
    ///
    /// Validates: workspace-model Requirement 2.1, 3.4, 4.1, 6.1
    pub(crate) fn open_workspace_force(&mut self, path: &std::path::Path) {
        match load_workspace(path) {
            Err(e) => {
                self.open_error = Some(format!("Cannot open workspace: {e}"));
            }
            Ok(ws) => {
                // Close any existing workspace first.
                if self.active_workspace.is_some() {
                    self.close_workspace();
                }
                // Register each root as a Native catalog.
                let mut root_warning: Option<String> = None;
                for root in &ws.roots {
                    if !root.exists() {
                        root_warning = Some(format!(
                            "Workspace warning: root '{}' not found on disk",
                            root.display()
                        ));
                        continue;
                    }
                    let cat = crate::catalog_registry::VirtualCatalog {
                        name: root
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| root.to_string_lossy().into_owned()),
                        catalog_type: crate::catalog_registry::CatalogType::Native,
                        path: root.to_string_lossy().into_owned(),
                        description: Some("Workspace root".to_string()),
                        auto_mount: true,
                        default_hlq: None,
                        mount_point: None,
                        read_only: false,
                    };
                    let _ = self.files_panel.registry.register(cat);
                }
                // Inject workspace settings into config as highest-priority layer.
                for (key, val) in &ws.settings {
                    let _ = self
                        .config_handle
                        .set_user_value(key, ff_config::ConfigValue::String(val.clone()));
                }
                self.active_workspace = Some(ws);
                // Preserve root warning; only clear error when everything succeeded.
                self.open_error = root_warning;
            }
        }
    }

    /// Save the active workspace to its current file path, or to `path` if provided.
    ///
    /// Validates: workspace-model Requirement 2.2, 2.3
    pub(crate) fn save_workspace_to(&mut self, path: Option<&std::path::Path>) {
        let Some(ws) = self.active_workspace.as_mut() else {
            self.open_error = Some("No active workspace to save".to_string());
            return;
        };
        let target = match path {
            Some(p) => p.to_path_buf(),
            None => match &ws.file_path {
                Some(p) => p.clone(),
                None => {
                    self.open_error = Some(
                        "No workspace file path set -- use WORKSPACE SAVE AS <path>".to_string(),
                    );
                    return;
                }
            },
        };
        if let Some(p) = path {
            ws.file_path = Some(p.to_path_buf());
        }
        ws.is_modified = false;
        match save_workspace(ws, &target) {
            Ok(()) => self.open_error = None,
            Err(e) => self.open_error = Some(format!("Cannot save workspace: {e}")),
        }
    }

    /// Unload the active workspace: unregister its roots and remove its settings layer.
    ///
    /// Validates: workspace-model Requirement 2.4, 3.4, 4.3
    pub(crate) fn close_workspace(&mut self) {
        let Some(ws) = self.active_workspace.take() else {
            return;
        };
        // Unregister workspace roots from the catalog registry.
        for root in &ws.roots {
            let name = root
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| root.to_string_lossy().into_owned());
            let _ = self.files_panel.registry.remove(&name);
        }
        // Remove workspace settings overrides (reset to user-layer values).
        for key in ws.settings.keys() {
            let _ = self.config_handle.remove_user_value(key);
        }
        self.open_error = None;
    }
}

/// - POM tab → app name + version
/// - FileEditor with path → full path
/// - FileEditor without path (Untitled) → "[Untitled]"
/// - All other kinds → tab title string
///
/// Validates: Requirement 17.3, 17.4, 17.5, 17.6
pub(crate) fn title_line_text(tab: &crate::tab_state::TabState) -> String {
    use crate::tab_state::TabKind;
    match tab.kind {
        TabKind::PrimaryOptionMenu => {
            format!("FileForge Workbench  v{}", env!("CARGO_PKG_VERSION"))
        }
        TabKind::FileEditor => tab
            .path
            .as_deref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "[Untitled]".to_string()),
        TabKind::Untitled => "[Untitled]".to_string(),
        TabKind::FilesPanel
        | TabKind::ConfigPanel
        | TabKind::FileExplorerPanel
        | TabKind::SearchResults
        | TabKind::PluginManager
        | TabKind::EventLog
        | TabKind::MacroLibrary
        | TabKind::CommandConfigurator
        | TabKind::ThemeEditor
        | TabKind::MenusEditor
        | TabKind::KeysEditor => tab.title.clone(),
        TabKind::MenuWorkspace => tab.title.clone(),
    }
}

/// Load `[context_key_maps]` from the workbench configuration into the resolver.
///
/// Reads the `context_key_maps` top-level table from `config_handle`.
/// Each sub-table key is a context name (e.g. `"editor"`, `"pom"`) and its
/// value is a key-map table using the same schema as `[global_key_map]`.
/// Invalid entries are silently skipped (warnings are not surfaced at startup).
///
/// Validates: Requirement 14.7
pub(crate) fn load_context_maps_from_config(config: &ConfigHandle, resolver: &mut KeyMapResolver) {
    use ff_config::ConfigValue;
    use ff_keys::KeyMap;

    let Ok(ConfigValue::Table(outer)) = config.get("context_key_maps") else {
        return;
    };
    for (ctx_name, ctx_value) in outer {
        if let ConfigValue::Table(ctx_table) = ctx_value {
            // Convert ConfigTable (BTreeMap<String, ConfigValue>) to toml::Table
            // so we can reuse KeyMap::from_toml_table.
            let mut toml_map = toml::map::Map::new();
            for (k, v) in ctx_table {
                if let Some(tv) = config_value_to_toml_value(v) {
                    toml_map.insert(k, tv);
                }
            }
            let (map, _warnings) = KeyMap::from_toml_table(&toml_map, &ctx_name);
            resolver.set_context_map(ctx_name, map);
        }
    }
}

/// Ensure the `keymaps/` directory exists under `user_data_dir` (creating it if
/// absent), so a user has a place to author per-context key-map override files
/// (`keymaps/<context>.toml`). The directory may be empty.
///
/// Mirrors `ensure_menus_dir` / `ensure_default_theme_files`: built-in default
/// key maps are CODE-ONLY (compiled `KeyMap::default_global`) and are NEVER
/// written here.
///
/// Validates: function-keys-and-history Requirement 14.11 (CR-CH-027)
pub(crate) fn ensure_keymaps_dir(user_data_dir: &std::path::Path) {
    let keymaps_dir = user_data_dir.join("keymaps");
    // Best-effort -- ignore errors (graceful degradation).
    let _ = std::fs::create_dir_all(&keymaps_dir);
}

/// Load per-context key-map override FILES from `<user_data_dir>/keymaps/*.toml`
/// into the resolver, keyed by the file stem (the context name).
///
/// Each `keymaps/<context>.toml` uses the same key-name schema as
/// `[global_key_map]` (Base `F1`-`F12` plus `SF`/`CF`/`AF`/`GF`/`XF` prefixes).
/// A present file is registered as the Context_Key_Map for that context
/// (full-replacement over the compiled default); a file that cannot be read or
/// parsed is skipped (DEBUG record) and the context falls back to the compiled
/// default.
///
/// Called at startup AFTER [`load_context_maps_from_config`], so a
/// `keymaps/<context>.toml` FILE takes precedence over a
/// `[context_key_maps.<name>]` config section for the same context
/// (Requirement 14.12).
///
/// Validates: function-keys-and-history Requirement 14.9, 14.10, 14.12 (CR-CH-027)
pub(crate) fn load_context_maps_from_keymaps_dir(
    keymaps_dir: &std::path::Path,
    resolver: &mut KeyMapResolver,
) {
    use ff_keys::KeyMap;

    let Ok(entries) = std::fs::read_dir(keymaps_dir) else {
        return; // Absent/unreadable dir: every context keeps the compiled default.
    };
    for entry in entries.flatten() {
        let path = entry.path();
        // Only `*.toml` files; the stem is the context name.
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let Some(ctx_name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            ff_logging::log_debug!("[keys] could not read keymaps file {}", path.display());
            continue;
        };
        let table: toml::Table = match toml::from_str(&text) {
            Ok(t) => t,
            Err(e) => {
                // Req 14.10: malformed file skipped; context keeps the default.
                ff_logging::log_debug!(
                    "[keys] skipping malformed keymaps file {}: {e}",
                    path.display()
                );
                continue;
            }
        };
        let (map, _warnings) = KeyMap::from_toml_table(&table, ctx_name);
        resolver.set_context_map(ctx_name.to_string(), map);
    }
}

mod commands;
mod configurator;
mod external_adapter;
/// Convert a `ff_config::ConfigValue` to a `toml::Value` for key-map parsing.
mod helpers;
mod keys_editor;
mod menus_editor;
mod nav_stack;
mod render;
mod render_chrome;
mod reset_bare;
mod target_dispatch;
mod update;
pub(crate) mod workspace_context;

use helpers::*;

#[cfg(test)]
mod tests;
