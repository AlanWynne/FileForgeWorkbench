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

use crate::exclude_manager::ExcludeManager;
use crate::file_explorer_panel::FileExplorerPanelState;
use crate::files_panel::FilesPanelState;
use crate::find_manager::FindManager;
use crate::nav_manager::NavManager;
use crate::primary_option_menu;
use crate::session_manager::SessionManager;
use crate::settings_panel::SettingsPanelState;
use crate::tab_manager::TabManager;
use crate::toolchain_panel::ToolchainPanelState;

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
    "Utilities",
    "Compilers",
    "Lua",
    "Terminals",
    "Databases",
    "Plugins",
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
    /// Session persistence — None when User Data Dir is unavailable.
    session: Option<SessionManager>,
    /// Configuration handle — used to read catalog default paths.
    config_handle: ConfigHandle,
    /// Files Panel (Virtual Catalog Manager) state.
    files_panel: FilesPanelState,
    /// File Explorer Panel state (expand/collapse per catalog node).
    ///
    /// Validates: Requirement 19.5, 19.6
    file_explorer_panel: FileExplorerPanelState,
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

        let dispatch = CommandDispatch::new(registry, history);

        let session = SessionManager::try_init();

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
            config_handle,
            files_panel: FilesPanelState::new(),
            file_explorer_panel: FileExplorerPanelState::new(),
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
            modal_open: false,
            key_config_dialog: crate::key_config_dialog::KeyConfigDialog::new(),
            settings_panel: SettingsPanelState::new(),
            focus_stop: FocusStop::CommandField,
            command_field_focus_requested: true,
            floating_tabs: Vec::new(),
            detach_pending: None,
            redock_pending: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // ── Theme ────────────────────────────────────────────────────────────

    // render methods are in render.rs
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
        TabKind::FilesPanel | TabKind::SettingsPanel | TabKind::FileExplorerPanel => {
            tab.title.clone()
        }
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
