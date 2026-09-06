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
use crate::event_log_panel::EventLogPanelState;
use crate::exclude_manager::ExcludeManager;
use crate::file_explorer_panel::FileExplorerPanelState;
use crate::files_panel::FilesPanelState;
use crate::find_manager::FindManager;
use crate::nav_manager::NavManager;
use crate::notification::{Notification, NotificationQueue, NotificationSender};
use crate::plugin_manager_panel::PluginManagerPanelState;
use crate::primary_option_menu;
pub(crate) use crate::scroll_amount::{ScrollAmount, SplitScreenState};
use crate::session_manager::SessionManager;
use crate::settings_panel::SettingsPanelState;
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

// ── Tab-order focus cycle — Validates: Requirement 16 ──────────────────────

/// The current keyboard focus stop in the shell tab-order cycle.
///
/// Full POM cycle (Tab):
///   CommandField → PomOption(0..8) → PomExit → CalendarPrev → CalendarNext
///   → MenuBar(0..N-1) → TabHeader(0..T-1) → CommandField
/// Non-POM cycle (Tab):
///   CommandField → MenuBar(0..N-1) → TabHeader(0..T-1) → CommandField
/// Shift+Tab is the exact reverse.
///
/// Validates: Requirement 16.1–16.22
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FocusStop {
    /// The primary command field ("Command ===>").
    CommandField,
    /// A numbered POM option row (0-based index into BUILT_IN_OPTIONS).
    PomOption { index: usize },
    /// The POM exit line ("Enter X to Terminate…").
    PomExit,
    /// The calendar `<` (previous-month) button on the active POM tab.
    CalendarPrev,
    /// The calendar `>` (next-month) button on the active POM tab.
    CalendarNext,
    /// A top-level menu bar heading at the given 0-based index.
    MenuBar { index: usize },
    /// A tab header button at the given 0-based index.
    ///
    /// Validates: Requirement 16.20, 16.21
    TabHeader { index: usize },
}

impl FocusStop {
    /// Advance to the next stop in the forward (Tab) direction.
    ///
    /// `menu_count` is the number of top-level menu bar headings.
    /// `tab_count` is the number of open tabs.
    /// `pom_active` is true when the active tab is a POM tab.
    ///
    /// Validates: Requirement 16.3–16.10, 16.19–16.21
    pub(crate) fn next(&self, menu_count: usize, tab_count: usize, pom_active: bool) -> FocusStop {
        let pom_count = primary_option_menu::BUILT_IN_OPTIONS.len(); // 9
        match self {
            FocusStop::CommandField => {
                if pom_active {
                    FocusStop::PomOption { index: 0 }
                } else {
                    FocusStop::MenuBar { index: 0 }
                }
            }
            FocusStop::PomOption { index } => {
                let next = index + 1;
                if next < pom_count {
                    FocusStop::PomOption { index: next }
                } else {
                    FocusStop::PomExit
                }
            }
            FocusStop::PomExit => FocusStop::CalendarPrev,
            FocusStop::CalendarPrev => FocusStop::CalendarNext,
            FocusStop::CalendarNext => FocusStop::MenuBar { index: 0 },
            FocusStop::MenuBar { index } => {
                let next = index + 1;
                if next < menu_count {
                    FocusStop::MenuBar { index: next }
                } else if tab_count > 0 {
                    FocusStop::TabHeader { index: 0 }
                } else {
                    FocusStop::CommandField
                }
            }
            FocusStop::TabHeader { index } => {
                let next = index + 1;
                if next < tab_count {
                    FocusStop::TabHeader { index: next }
                } else {
                    FocusStop::CommandField
                }
            }
        }
    }

    /// Advance to the previous stop in the backward (Shift+Tab) direction.
    ///
    /// Validates: Requirement 16.11, 16.19, 16.22
    pub(crate) fn prev(&self, menu_count: usize, tab_count: usize, pom_active: bool) -> FocusStop {
        let pom_count = primary_option_menu::BUILT_IN_OPTIONS.len(); // 9
        match self {
            FocusStop::CommandField => {
                if tab_count > 0 {
                    FocusStop::TabHeader {
                        index: tab_count - 1,
                    }
                } else {
                    FocusStop::MenuBar {
                        index: menu_count.saturating_sub(1),
                    }
                }
            }
            FocusStop::PomOption { index } => {
                if *index == 0 {
                    FocusStop::CommandField
                } else {
                    FocusStop::PomOption { index: index - 1 }
                }
            }
            FocusStop::PomExit => FocusStop::PomOption {
                index: pom_count - 1,
            },
            FocusStop::CalendarPrev => FocusStop::PomExit,
            FocusStop::CalendarNext => FocusStop::CalendarPrev,
            FocusStop::MenuBar { index } => {
                if *index == 0 {
                    if pom_active {
                        FocusStop::CalendarNext
                    } else if tab_count > 0 {
                        FocusStop::TabHeader {
                            index: tab_count - 1,
                        }
                    } else {
                        FocusStop::CommandField
                    }
                } else {
                    FocusStop::MenuBar { index: index - 1 }
                }
            }
            FocusStop::TabHeader { index } => {
                if *index == 0 {
                    FocusStop::MenuBar {
                        index: menu_count.saturating_sub(1),
                    }
                } else {
                    FocusStop::TabHeader { index: index - 1 }
                }
            }
        }
    }
}

// ── Menu bar top-level label registry — Validates: Requirement 14.7 ────────
/// Ordered top-level menu bar labels mirroring the 9-option POM plus Help.
/// Tests assert against this array; render_menu_bar must contain a menu_button for each entry.
pub(crate) const MENU_BAR_TOP_LEVEL_LABELS: &[&str] = &[
    "Settings",
    "File Catalogs",
    "Files",
    "View",
    "Utilities",
    "Compilers",
    "Lua",
    "Terminals",
    "Databases",
    "Plugins",
    "Search",
    "Edit",
    "Help",
];

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
    /// Files Panel (Virtual Catalog Manager) state.
    files_panel: FilesPanelState,
    /// File Explorer Panel state (expand/collapse per catalog node).
    ///
    /// Validates: Requirement 19.5, 19.6
    file_explorer_panel: FileExplorerPanelState,
    /// Persisted width of the File Explorer side panel (logical pixels).
    ///
    /// Validates: Requirement 1.3 file-tree-panel (fix B019)
    file_explorer_panel_width: f32,
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
    /// Global application zoom — single level shared across all tabs and panels.
    ///
    /// Addresses: Requirement 3.1 (view-zoom) — zoom carries forward across context switches.
    zoom: ZoomState,
    /// Month offset for the POM calendar (0 = current month, -1 = prev, +1 = next).
    ///
    /// Validates: Requirement 14.42
    pom_calendar_offset: i32,
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
    /// Key Configuration Dialog state.
    ///
    /// Validates: Requirement 20.1
    key_config_dialog: crate::key_config_dialog::KeyConfigDialog,
    /// Settings Panel state.
    ///
    /// Validates: Requirement 15.1, 15.2
    settings_panel: SettingsPanelState,
    /// Plugin Manager panel state.
    ///
    /// Validates: plugin-manager-ui Requirement 1.1
    plugin_manager_panel: PluginManagerPanelState,
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
    /// Current keyboard focus stop in the shell tab-order cycle.
    ///
    /// Validates: Requirement 16.1–16.7
    focus_stop: FocusStop,
    /// True for exactly one frame after Tab/Shift+Tab moves focus_stop to CommandField
    /// or on the startup frame. Causes a one-shot request_focus on the command field.
    /// Cleared immediately after the request fires so we do NOT steal focus every frame.
    ///
    /// Validates: Requirement 16.1, 16.2
    command_field_focus_requested: bool,
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

        let cmd_registry = registry.clone();
        let dispatch = CommandDispatch::new(registry, history);

        let session = SessionManager::try_init();

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
            session,
            active_workspace: None,
            pending_workspace_open: None,
            show_unsaved_workspace_dialog: false,
            config_handle,
            files_panel: FilesPanelState::new(),
            file_explorer_panel: FileExplorerPanelState::new(),
            file_explorer_panel_width: 260.0,
            toolchain_panel: ToolchainPanelState::new(),
            show_toolchain_panel: false,
            pending_new_pom: false,
            pending_new_file: false,
            pending_return_to_pom: false,
            zoom: ZoomState::new(&ZoomConfig::default()),
            pom_calendar_offset: 0,
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
            key_config_dialog: crate::key_config_dialog::KeyConfigDialog::new(),
            settings_panel: SettingsPanelState::new(),
            plugin_manager_panel: PluginManagerPanelState::new(),
            event_log_panel: EventLogPanelState::new(),
            notification_rx,
            notification_tx,
            notification_queue,
            focus_stop: FocusStop::CommandField,
            command_field_focus_requested: true,
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

    // ── Session lifecycle helpers — Validates: Requirement 20.1, 20.2 ────

    /// Format the session start time as `Started: HH:MM`.
    ///
    /// Validates: Requirement 20.1
    pub(crate) fn format_session_start(&self) -> String {
        format!("Started: {}", self.session_start.format("%H:%M"))
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
        | TabKind::SettingsPanel
        | TabKind::FileExplorerPanel
        | TabKind::SearchResults
        | TabKind::PluginManager
        | TabKind::EventLog => tab.title.clone(),
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

mod commands;
/// Convert a `ff_config::ConfigValue` to a `toml::Value` for key-map parsing.
mod helpers;
mod render;
mod render_chrome;
mod update;

use helpers::*;

#[cfg(test)]
mod tests;
