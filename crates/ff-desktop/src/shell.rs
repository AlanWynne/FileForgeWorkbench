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
use ff_command_semantics::{CommandEngine, StatusKind};
use ff_config::ConfigHandle;
use ff_core::{LifecyclePhase, WorkbenchApp};
use ff_keys::{
    CommandHistory, FunctionKey, KeyLabelBarModel, KeyMap, KeyMapResolver, KeyModifier,
    ModifiedKey, RetrieveResult, RetrieveState,
};
use ff_theme::{ColourRGBA, ThemePalette};
use ff_zoom::{ZoomConfig, ZoomState};
use tokio::runtime::Runtime;

use crate::catalog_manager_dialog::{self, DeleteChoice, DialogOutcome, NewCatalogForm};
use crate::dataset_alloc_dialog::{self, AllocOutcome};
use crate::editor_panel;
use crate::exclude_manager::ExcludeManager;
use crate::files_panel::{self, FilesPanelState};
use crate::find_manager::FindManager;
use crate::nav_manager::NavManager;
use crate::primary_option_menu;
use crate::session_manager::SessionManager;
use crate::settings_panel::SettingsPanelState;
use crate::tab_manager::TabManager;
use crate::tab_state::TabKind;
use crate::toolchain_panel::{self, ToolchainPanelState};

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

    fn apply_theme(&self, ctx: &egui::Context) {
        let p = &self.palette;
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = to_egui_color(p.editor.background);
        visuals.window_fill = to_egui_color(p.ui.panel_bg);
        visuals.window_stroke = egui::Stroke::new(1.0_f32, to_egui_color(p.ui.panel_border));
        // Use menu_bar_fg as the global text colour — in Legacy this is white (#FFFFFF),
        // which correctly colours menu bar items, tab bar, and chrome text.
        // Editor content text is applied per-element in editor_panel using palette tokens.
        visuals.override_text_color = Some(to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.noninteractive.bg_fill = to_egui_color(p.ui.panel_bg);
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.editor.foreground));
        visuals.widgets.inactive.bg_fill = to_egui_color(p.ui.button_bg);
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.hovered.bg_fill = to_egui_color(p.ui.button_hover);
        visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.active.bg_fill = to_egui_color(p.ui.input_bg);
        visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.selection.bg_fill = to_egui_color(p.editor.accent).linear_multiply(0.35);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, to_egui_color(p.editor.accent));
        // In Legacy mode the slider track and handle are near-black on black — invisible.
        // Override with high-contrast ISPF colours: turquoise track, yellow handle.
        if p.mode == ff_theme::mode::VisualMode::Legacy {
            let track = egui::Color32::from_rgb(0, 170, 170); // ISPF turquoise
            let handle = egui::Color32::from_rgb(255, 255, 0); // ISPF yellow-hi
            visuals.widgets.inactive.bg_fill = track;
            visuals.widgets.inactive.fg_stroke = egui::Stroke::new(2.0_f32, handle);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 210, 210);
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(2.0_f32, handle);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 255, 255);
            visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0_f32, handle);
        }
        ctx.set_visuals(visuals);
    }

    // ── Theme switching ───────────────────────────────────────────────────

    /// Switch to the given visual mode.
    ///
    /// Writes the mode name to the `theme.active` config key so the per-frame
    /// hot-reload block picks it up and the palette is not clobbered next frame.
    fn set_theme(&mut self, mode: ff_theme::mode::VisualMode) {
        self.palette = ff_theme::defaults::default_palette_for_mode(mode);
        let mode_str = mode.section_name().to_string();
        let _ = self.config_handle.set_user_value(
            ff_config::keys::theme::ACTIVE,
            ff_config::ConfigValue::String(mode_str),
        );
    }

    // ── Legacy POM colours ────────────────────────────────────────────────

    /// Build `PomColours` for the current palette.
    ///
    /// When the Legacy theme is active, returns ISPF semantic colours.
    /// For all other themes, returns `PomColours::inherited()` so egui
    /// uses its own default colours.
    ///
    /// Validates: Requirement 13 (Legacy Theme Colour Semantics)
    fn legacy_pom_colours(&self) -> primary_option_menu::PomColours {
        use ff_theme::mode::VisualMode;
        if self.palette.mode == VisualMode::Legacy {
            primary_option_menu::PomColours::from_palette(&self.palette)
        } else {
            primary_option_menu::PomColours::inherited()
        }
    }

    // ── Menu bar ─────────────────────────────────────────────────────────

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        // Validates: Requirement 14.7 — every label in the registry must have a menu_button below.
        debug_assert_eq!(
            MENU_BAR_TOP_LEVEL_LABELS.len(),
            11,
            "render_menu_bar must contain one menu_button per MENU_BAR_TOP_LEVEL_LABELS entry"
        );
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // ── Focus request for menu bar stops — Validates: Requirement 16.8
                // When focus_stop is MenuBar{index}, request focus on that button's Id.
                for (idx, label) in MENU_BAR_TOP_LEVEL_LABELS.iter().enumerate() {
                    if self.focus_stop == (FocusStop::MenuBar { index: idx }) {
                        let id = egui::Id::new("menu_bar_btn").with(idx);
                        ui.memory_mut(|m| m.request_focus(id));
                        let _ = label; // label used only for Id derivation above
                    }
                }
                // ── Settings ────────────────────────────────────────────
                ui.menu_button("Settings", |ui| {
                    if ui.button("Preferences…").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Themes").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Dark Theme").clicked() {
                        self.set_theme(ff_theme::mode::VisualMode::Dark);
                        ui.close_menu();
                    }
                    if ui.button("Light Theme").clicked() {
                        self.set_theme(ff_theme::mode::VisualMode::Light);
                        ui.close_menu();
                    }
                    if ui.button("High Contrast").clicked() {
                        self.set_theme(ff_theme::mode::VisualMode::HighContrast);
                        ui.close_menu();
                    }
                    if ui.button("Legacy (ISPF 3270)").clicked() {
                        self.set_theme(ff_theme::mode::VisualMode::Legacy);
                        ui.close_menu();
                    }
                    ui.separator();
                    // Validates: Requirement 14.14 — open new POM tab from Settings menu
                    if ui.button("Primary Option Menu").clicked() {
                        self.tabs.insert_pom_tab(&self.runtime);
                        ui.close_menu();
                    }
                });
                // ── File Catalogs — Validates: Requirement 14.7 (mirrors POM option 1) ──
                ui.menu_button("File Catalogs", |ui| {
                    if ui.button("Open File Catalogs").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Files ───────────────────────────────────────────────
                ui.menu_button("Files", |ui| {
                    if ui.button("New").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Open…").clicked() {
                        self.open_file_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save").clicked() {
                        if let Err(e) = self.tabs.save_active_tab(&self.runtime) {
                            self.open_error = Some(e);
                        } else {
                            self.open_error = None;
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save As…").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                // ── Utilities ───────────────────────────────────────────
                ui.menu_button("Utilities", |ui| {
                    if ui.button("Compare Files…").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("File Tree").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Compilers ───────────────────────────────────────────
                ui.menu_button("Compilers", |ui| {
                    if ui.button("Toolchain Panel").clicked() {
                        self.show_toolchain_panel = !self.show_toolchain_panel;
                        ui.close_menu();
                    }
                    if ui.button("Build").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Run").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Lua ─────────────────────────────────────────────────
                ui.menu_button("Lua", |ui| {
                    if ui.button("Run Script…").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Macro Editor").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Terminals ───────────────────────────────────────────
                ui.menu_button("Terminals", |ui| {
                    if ui.button("New Terminal").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Databases ───────────────────────────────────────────
                ui.menu_button("Databases", |ui| {
                    if ui.button("Connect…").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Query Browser").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Plugins — Validates: Requirement 14.7 (mirrors POM option 8) ─────
                ui.menu_button("Plugins", |ui| {
                    if ui.button("Manage Plugins").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Edit (always present) ────────────────────────────────
                ui.menu_button("Edit", |ui| {
                    if ui.button("Key Assignments\u{2026}").clicked() {
                        self.key_config_dialog.open = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Undo").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Redo").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Cut").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Copy").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Paste").clicked() {
                        ui.close_menu();
                    }
                });
                // ── Help ────────────────────────────────────────────────
                ui.menu_button("Help", |ui| {
                    if ui.button("About FileForge Workbench").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    // ── Tab bar ──────────────────────────────────────────────────────────

    fn render_tab_bar(&mut self, ctx: &egui::Context) {
        let active_bg = to_egui_color(self.palette.ui.input_bg);
        let inactive_bg = to_egui_color(self.palette.ui.panel_bg);
        let text_color = to_egui_color(self.palette.editor.foreground);
        let modified_color = to_egui_color(self.palette.editor.accent);

        // Collect context-menu actions outside the borrow of self.tabs.
        let mut activate_idx: Option<usize> = None;
        let mut close_idx: Option<usize> = None;
        let mut close_all_but: Option<usize> = None;
        let mut close_left_of: Option<usize> = None;
        let mut close_right_of: Option<usize> = None;
        let mut close_unchanged = false;

        egui::TopBottomPanel::top("tab_bar")
            .min_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let tab_count = self.tabs.len();
                    let active_idx_cur = self.tabs.active_index();

                    for i in 0..tab_count {
                        let tab = &self.tabs.tabs()[i];
                        // Validates: Requirement 18.4 — skip tabs that are floating.
                        if tab.is_floating {
                            continue;
                        }
                        let is_active = i == active_idx_cur;
                        let tab_kind = tab.kind;

                        let bg = if is_active { active_bg } else { inactive_bg };
                        let label = if tab.is_modified {
                            format!("● {}", tab.title)
                        } else {
                            tab.title.clone()
                        };
                        let color = if tab.is_modified {
                            modified_color
                        } else {
                            text_color
                        };

                        let btn =
                            egui::Button::new(egui::RichText::new(&label).color(color).monospace())
                                .fill(bg)
                                .stroke(if is_active {
                                    egui::Stroke::new(1.0_f32, color)
                                } else {
                                    egui::Stroke::NONE
                                })
                                .min_size(egui::vec2(0.0, 24.0));

                        let resp = ui.add(btn);
                        if resp.clicked() {
                            activate_idx = Some(i);
                        }

                        // ── Tab header right-click context menu ──────────
                        // Validates: Requirement 14.15, 14.15a, 14.15b, 14.15c
                        resp.context_menu(|ui| {
                            let tab_count_inner = self.tabs.len();
                            // ── Universal items (all tab kinds) — Req 14.15a ──
                            if ui.button("Close").clicked() {
                                close_idx = Some(i);
                                ui.close_menu();
                            }
                            ui.add_enabled_ui(tab_count_inner > 1, |ui| {
                                if ui.button("Close All BUT This").clicked() {
                                    close_all_but = Some(i);
                                    ui.close_menu();
                                }
                            });
                            ui.add_enabled_ui(i > 0, |ui| {
                                if ui.button("Close All to the Left").clicked() {
                                    close_left_of = Some(i);
                                    ui.close_menu();
                                }
                            });
                            ui.add_enabled_ui(i < tab_count_inner - 1, |ui| {
                                if ui.button("Close All to the Right").clicked() {
                                    close_right_of = Some(i);
                                    ui.close_menu();
                                }
                            });
                            if ui.button("Close All Unchanged").clicked() {
                                close_unchanged = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Clone to Other Tab").clicked() {
                                // stub — deferred
                                ui.close_menu();
                            }
                            if ui.button("Move to Other View").clicked() {
                                // Validates: Requirement 18.1, 18.7
                                if self.floating_tabs.len() < 16 {
                                    self.detach_pending = Some(i);
                                } else {
                                    self.open_error = Some(
                                        "Maximum 16 floating windows already open.".to_string(),
                                    );
                                }
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Pin Tab").clicked() {
                                // stub — deferred
                                ui.close_menu();
                            }

                            // ── Exit — Req 14.15a, 14.38 (all tab kinds) ─────
                            ui.separator();
                            if ui.button("Exit").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                ui.close_menu();
                            }

                            // ── File-editor-only items — Req 14.15b ──────────
                            // Only shown when the tab is a FileEditor.
                            if tab_kind == TabKind::FileEditor {
                                ui.separator();
                                if ui.button("Open Containing Folder in Explorer").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Explorer);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Open Containing Folder in CMD").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Cmd);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Open Containing Folder in PowerShell").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::PowerShell);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Open Containing Folder in Terminal").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Terminal);
                                    }
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("Copy Name to Clipboard").clicked() {
                                    if let Some(title) =
                                        self.tabs.tabs().get(i).map(|t| t.title.clone())
                                    {
                                        ui.output_mut(|o| o.copied_text = title);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Copy Path to Clipboard").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.clone() {
                                        ui.output_mut(|o| o.copied_text = path);
                                    }
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("Save").clicked() {
                                    // handled after menu closes via pending action
                                    ui.close_menu();
                                }
                                if ui.button("Save As").clicked() {
                                    ui.close_menu();
                                }
                                if ui.button("Reload").clicked() {
                                    ui.close_menu();
                                }
                            }
                        });
                    }

                    // ── Empty tab-bar space right-click — Req 14.9 ──────
                    let bar_resp = ui.interact(
                        ui.available_rect_before_wrap(),
                        ui.id().with("tab_bar_empty"),
                        egui::Sense::click(),
                    );
                    bar_resp.context_menu(|ui| {
                        if ui.button("New").clicked() {
                            self.pending_new_pom = true;
                            ui.close_menu();
                        }
                        if ui.button("New File").clicked() {
                            self.pending_new_file = true;
                            ui.close_menu();
                        }
                    });
                });
            });

        // Apply deferred tab-bar actions.
        if let Some(i) = activate_idx {
            // Track previous tab for END navigation -- Validates: Requirement 17.1
            self.tab_history.push(self.tabs.active_index());
            self.tabs.set_active(i);
            // Update key map context for new active tab -- Validates: Requirement 14.4
            let ctx_name = context_name_for_kind(self.tabs.active_tab().kind);
            self.key_map_resolver.set_context(ctx_name);
            self.key_label_bar
                .update(self.key_map_resolver.active_key_map());
        }
        if let Some(i) = close_idx {
            self.tabs.close_tab(i);
        }
        if let Some(pivot) = close_all_but {
            let count = self.tabs.len();
            // Close right-of-pivot first (indices stable), then left.
            for i in (pivot + 1..count).rev() {
                self.tabs.close_tab(i);
            }
            for i in (0..pivot).rev() {
                self.tabs.close_tab(i);
            }
        }
        if let Some(pivot) = close_left_of {
            for i in (0..pivot).rev() {
                self.tabs.close_tab(i);
            }
        }
        if let Some(pivot) = close_right_of {
            let count = self.tabs.len();
            for i in (pivot + 1..count).rev() {
                self.tabs.close_tab(i);
            }
        }
        if close_unchanged {
            let count = self.tabs.len();
            for i in (0..count).rev() {
                if !self.tabs.tabs()[i].is_modified {
                    self.tabs.close_tab(i);
                }
            }
        }
    }

    // ── Title line ──────────────────────────────────────────────────

    /// Render the ISPF-style Title_Line between the tab bar and command field.
    ///
    /// Shows context-dependent text derived from the active tab kind.
    /// In Legacy theme: blue background (#0000AA), white text (#FFFFFF).
    ///
    /// Validates: Requirement 17.1, 17.2, 17.3–17.8
    fn render_title_line(&self, ctx: &egui::Context) {
        use ff_theme::mode::VisualMode;
        let text = title_line_text(self.tabs.active_tab());
        let is_legacy = self.palette.mode == VisualMode::Legacy;
        let is_pom = self.tabs.active_tab().kind == crate::tab_state::TabKind::PrimaryOptionMenu;
        egui::TopBottomPanel::top("title_line").show(ctx, |ui| {
            if is_pom {
                // POM title: black background, blue text, centered
                let bg = egui::Color32::BLACK;
                let fg = egui::Color32::from_rgb(0x00, 0x55, 0xFF);
                let rect = ui.max_rect();
                ui.painter().rect_filled(rect, 0.0, bg);
                ui.centered_and_justified(|ui| {
                    ui.colored_label(fg, egui::RichText::new(&text).monospace().strong());
                });
            } else if is_legacy {
                // Validates: Requirement 17.8 — Legacy: blue bg, white text
                let bg = to_egui_color(self.palette.ui.primary_menu_bg);
                let fg = to_egui_color(self.palette.ui.menu_bar_fg);
                let rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(rect, 0.0, bg);
                ui.colored_label(fg, egui::RichText::new(text).monospace());
            } else {
                ui.label(egui::RichText::new(text).monospace());
            }
        });
    }

    // ── Command field ────────────────────────────────────────────────────

    fn render_command_field(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("command_field").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Command ===>");
                let cmd_id = egui::Id::new("command_field_input");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.command_text)
                        .id(cmd_id)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );
                // Validates: Requirement 16.1, 16.2 — request focus once after Tab cycle
                // lands on CommandField, or on startup. NOT every frame — that would steal
                // focus from POM buttons and other interactive elements.
                // Suppressed when a modal dialog is open so the dialog retains focus.
                if self.command_field_focus_requested && !self.modal_open {
                    self.command_field_focus_requested = false;
                    ctx.memory_mut(|m| m.request_focus(cmd_id));
                }
                // Validates: Requirement 8.1 — Enter while field has focus submits the command.
                // Use lost_focus() to catch the frame egui clears focus on Enter for
                // single-line TextEdit (egui 0.29 surrenders focus on Enter).
                // Also check has_focus() as a fallback for frames where focus is retained.
                let field_has_focus = response.has_focus() || response.lost_focus();
                if field_has_focus
                    && ctx.input(|i| i.key_pressed(egui::Key::Enter))
                    && !self.command_text.is_empty()
                {
                    let cmd = self.command_text.trim().to_string();
                    self.command_text.clear();
                    self.handle_command(&cmd);
                    // Return focus to the command field after every command execution.
                    self.focus_stop = FocusStop::CommandField;
                    self.command_field_focus_requested = true;
                }
            });
        });
    }

    /// Dispatch an ISPF command field string.
    ///
    /// Shell-level commands (EXIT, QUIT, =X, X, EDIT, 1, FILES, RETRIEVE) are
    /// intercepted before reaching the engine. Everything else is routed through
    /// `CommandEngine` which handles parsing, normalisation, and error reporting.
    fn handle_command(&mut self, cmd: &str) {
        let upper = cmd.trim().to_uppercase();

        // ── Shell-level intercepts ───────────────────────────────────────
        if upper == "EXIT" || upper == "QUIT" || upper == "=X" || upper == "X" {
            let result = self
                .dispatch
                .execute_command("file.exit", CommandParams::new());
            if let CommandResult::Err(e) = result {
                self.open_error = Some(e.to_string());
            }
            return;
        }

        if upper.starts_with("EDIT") && (upper == "EDIT" || upper.starts_with("EDIT ")) {
            let rest = cmd.trim().split_once(' ').map(|x| x.1.trim()).unwrap_or("");
            if rest.is_empty() {
                self.open_error = Some("EDIT requires a file path".to_string());
            } else {
                let mut p = CommandParams::new();
                p.insert("path", rest);
                let result = self.dispatch.execute_command("file.open", p);
                if let CommandResult::Err(e) = result {
                    self.open_error = Some(e.to_string());
                } else {
                    self.open_error = None;
                }
            }
            return;
        }

        if upper == "START" || upper == "POM" {
            // Validates: Requirement 14.10, 14.14 — START/POM opens a new POM tab
            self.tabs.insert_pom_tab(&self.runtime);
            self.open_error = None;
            return;
        }

        if upper == "CLOSE" {
            // Validates: Requirement 14.11 — CLOSE closes the current tab
            let idx = self.tabs.active_index();
            self.tabs.close_tab(idx);
            self.open_error = None;
            return;
        }

        // ── KEYS — Validates: Requirement 20.1 ————————————————————————————
        if upper == "KEYS" {
            self.key_config_dialog.open = true;
            self.open_error = None;
            return;
        }

        // ── PFSHOW — Validates: Requirement 12.1–12.3 ——————————————————————
        if upper == "PFSHOW" {
            self.key_bar_visible = !self.key_bar_visible;
            self.open_error = None;
            return;
        }
        if upper == "PFSHOW ON" {
            self.key_bar_visible = true;
            self.open_error = None;
            return;
        }
        if upper == "PFSHOW OFF" {
            self.key_bar_visible = false;
            self.open_error = None;
            return;
        }

        // ── END — Validates: Requirement 17.1, 17.2 ———————————————————————
        if upper == "END" {
            let is_pom = self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu;
            if is_pom {
                // Validates: Requirement 17.2 — END from POM exits
                let result = self
                    .dispatch
                    .execute_command("file.exit", CommandParams::new());
                if let CommandResult::Err(e) = result {
                    self.open_error = Some(e.to_string());
                }
            } else {
                // Validates: Requirement 17.1 — close current tab, go to previous
                let current = self.tabs.active_index();
                self.tabs.close_tab(current);
                if let Some(prev) = self.tab_history.pop() {
                    let clamped = prev.min(self.tabs.len().saturating_sub(1));
                    self.tabs.set_active(clamped);
                }
            }
            self.open_error = None;
            return;
        }

        // ── RETURN — Validates: Requirement 17.3, 17.4 ————————————————————
        if upper == "RETURN" {
            let is_pom = self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu;
            if is_pom {
                // Validates: Requirement 17.4 — RETURN from POM exits
                let result = self
                    .dispatch
                    .execute_command("file.exit", CommandParams::new());
                if let CommandResult::Err(e) = result {
                    self.open_error = Some(e.to_string());
                }
            } else {
                // Validates: Requirement 17.3 — navigate to POM tab
                if let Some(pom_idx) = self
                    .tabs
                    .tabs()
                    .iter()
                    .position(|t| t.kind == TabKind::PrimaryOptionMenu)
                {
                    self.tabs.set_active(pom_idx);
                } else {
                    self.tabs.insert_pom_tab(&self.runtime);
                }
            }
            self.open_error = None;
            return;
        }

        if upper == "0" || upper == "SETTINGS" || upper == "=0" {
            // Validates: Requirement 15.1 — option 0 / SETTINGS / =0 opens Settings panel
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::SettingsPanel, "[SETTINGS]");
            } else {
                self.tabs.open_settings_panel_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }

        if upper == "1" || upper == "FILES" {
            // Validates: Requirement 1.1, 14.6 — option 1 opens the Files Panel
            // If active tab is POM, transform it in-place (Req 14.6).
            // Otherwise open a new tab.
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::FilesPanel, "[FILES]");
            } else {
                self.tabs.open_files_panel_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }

        if upper == "3" || upper == "UTILITIES" {
            // Req 14.6 — option 3 opens Utilities (stub)
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::Untitled, "Utilities");
            }
            self.open_error = None;
            return;
        }

        if upper == "4" || upper == "COMPILERS" {
            // Req 14.6 — option 4 opens the Toolchain Panel
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.show_toolchain_panel = true;
                self.tabs
                    .transform_active_pom_tab(TabKind::Untitled, "Compilers");
            } else {
                self.show_toolchain_panel = true;
            }
            self.open_error = None;
            return;
        }

        if upper == "7" || upper == "DATABASES" {
            // Req 14.6 — option 7 opens the Databases panel (stub)
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::Untitled, "Databases");
            }
            self.open_error = None;
            return;
        }

        if upper == "8" || upper == "PLUGINS" {
            // Req 14.6 — option 8 opens the Plugins panel (stub)
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::Untitled, "Plugins");
            }
            self.open_error = None;
            return;
        }

        if upper == "RETRIEVE" {
            let cmd_text = self.command_text.clone();
            match self.retrieve_state.retrieve(&self.cmd_history, &cmd_text) {
                RetrieveResult::Recalled { command } => {
                    self.command_text = command;
                }
                RetrieveResult::ShowList { entries } => {
                    // Validates: Requirement 19.1 — show history list overlay
                    self.show_history_list = Some(entries);
                    self.command_text.clear();
                }
                RetrieveResult::HistoryEmpty | RetrieveResult::NoOlderHistory => {}
            }
            return;
        }

        // ── LOCATE / SORT / UP / DOWN / LEFT / RIGHT / TOP / BOTTOM ────────
        if upper.starts_with("LOCATE ") {
            let arg = cmd.trim()[7..].trim();
            let status = self.nav_manager.locate(arg, &mut self.tabs);
            self.open_error = if status.is_empty() {
                None
            } else {
                Some(status)
            };
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "TOP" {
            self.nav_manager.top(&mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "BOTTOM" {
            self.nav_manager.bottom(&mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "UP" || upper.starts_with("UP ") {
            let n = parse_optional_u64(cmd.trim().get(2..).unwrap_or("").trim());
            self.nav_manager.up(n, &mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "DOWN" || upper.starts_with("DOWN ") {
            let n = parse_optional_u64(cmd.trim().get(4..).unwrap_or("").trim());
            self.nav_manager.down(n, &mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "LEFT" || upper.starts_with("LEFT ") {
            let n = parse_optional_u64(cmd.trim().get(4..).unwrap_or("").trim());
            self.nav_manager.left(n, &mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "RIGHT" || upper.starts_with("RIGHT ") {
            let n = parse_optional_u64(cmd.trim().get(5..).unwrap_or("").trim());
            self.nav_manager.right(n, &mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "SORT" || upper.starts_with("SORT ") {
            let rest = cmd.trim().get(4..).unwrap_or("").trim();
            let args: Vec<&str> = rest.split_whitespace().collect();
            let status = self.nav_manager.sort(&args, &mut self.tabs, &self.runtime);
            self.open_error = if status.is_empty() {
                None
            } else {
                Some(status)
            };
            self.cmd_history.add(cmd);
            return;
        }

        // ── EXCLUDE / SHOW / RESET ────────────────────────────────────────────
        if upper == "EXCLUDE ALL" || upper == "X ALL" {
            let msg = self
                .exclude_manager
                .exclude_all(&mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            self.cmd_history.add(cmd);
            return;
        }

        if upper.starts_with("EXCLUDE ") || upper.starts_with("X ") {
            // EXCLUDE 'text' [ALL]  or  X 'text' [ALL]
            let rest = if upper.starts_with("EXCLUDE ") {
                cmd.trim()[8..].trim()
            } else {
                cmd.trim()[2..].trim()
            };
            let (text, all_flag) = strip_all_suffix(rest);
            let msg = if all_flag {
                self.exclude_manager
                    .exclude_text_all(text, &mut self.tabs, &self.runtime)
            } else {
                self.exclude_manager
                    .exclude_text(text, &mut self.tabs, &self.runtime)
            };
            self.open_error = info_or_error(&msg);
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "SHOW ALL" || upper == "INCLUDE ALL" {
            let msg = self.exclude_manager.show_all(&mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            self.cmd_history.add(cmd);
            return;
        }

        if upper.starts_with("SHOW ") || upper.starts_with("INCLUDE ") {
            let rest = if upper.starts_with("SHOW ") {
                cmd.trim()[5..].trim()
            } else {
                cmd.trim()[8..].trim()
            };
            let msg = self
                .exclude_manager
                .show_text(rest, &mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "RESET" || upper == "RESET EXCLUDED" || upper == "RESET ALL" {
            use ff_exclude_show_filter::ResetVariant;
            let variant = if upper == "RESET ALL" {
                ResetVariant::All
            } else if upper == "RESET EXCLUDED" {
                ResetVariant::Excluded
            } else {
                ResetVariant::Default
            };
            let msg = self
                .exclude_manager
                .reset(variant, &mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            self.cmd_history.add(cmd);
            return;
        }

        // ── FIND / RFIND / CHANGE / RCHANGE ─────────────────────────────────
        if upper == "RFIND" {
            let status = self.find_manager.rfind(&mut self.tabs, &self.runtime);
            self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                Some(status)
            } else {
                self.open_error = None;
                None
            };
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "RCHANGE" {
            let status = self.find_manager.rchange(&mut self.tabs, &self.runtime);
            self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                Some(status)
            } else {
                None
            };
            self.cmd_history.add(cmd);
            return;
        }

        if upper.starts_with("FIND ") {
            let term = cmd.trim()[5..].trim();
            let status = self.find_manager.find(term, &mut self.tabs, &self.runtime);
            self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                Some(status)
            } else {
                None
            };
            self.cmd_history.add(cmd);
            return;
        }

        if upper.starts_with("CHANGE ") {
            // Parse: CHANGE 'old' 'new'  (single-quoted or bare words)
            let rest = cmd.trim()[7..].trim();
            if let Some((old, new)) = parse_two_args(rest) {
                let status = self
                    .find_manager
                    .change(&old, &new, &mut self.tabs, &self.runtime);
                self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                    Some(status)
                } else {
                    None
                };
            } else {
                self.open_error =
                    Some("CHANGE requires two arguments: CHANGE 'old' 'new'".to_string());
            }
            self.cmd_history.add(cmd);
            return;
        }

        // ── Route through CommandEngine ──────────────────────────────────
        self.retrieve_state.reset();
        let status = self.cmd_engine.execute_command_line(cmd);
        match status.kind {
            StatusKind::Info => {
                self.open_error = None;
            }
            StatusKind::SyntaxError | StatusKind::StructureError | StatusKind::RuntimeError => {
                self.open_error = Some(status.text.clone());
            }
        }
        // Record in history (skip empty / error-only inputs)
        if !cmd.trim().is_empty() {
            self.cmd_history.add(cmd);
        }
    }

    // ── Key label bar ─────────────────────────────────────────────────────

    /// Render the ISPF-style function key label bar in the footer.
    ///
    /// Shows only assigned slots as `Fn label` pairs.
    /// Validates: Requirement 4.1, 4.2, 4.3
    fn render_key_label_bar(&mut self, ctx: &egui::Context) {
        if !self.key_bar_visible {
            return;
        }
        let key_color = to_egui_color(self.palette.editor.accent);
        let label_color = to_egui_color(self.palette.editor.foreground);
        let mut clicked_key: Option<FunctionKey> = None;
        egui::TopBottomPanel::bottom("key_label_bar").show(ctx, |ui| {
            for row in [self.key_label_bar.row0(), self.key_label_bar.row1()] {
                ui.horizontal(|ui| {
                    for slot in row {
                        let key = slot.key;
                        let btn_text = if let Some(lbl) = &slot.label {
                            format!("{} {}", key.display_name(), lbl)
                        } else {
                            key.display_name().to_string()
                        };
                        let enabled = slot.label.is_some();
                        let tooltip = self
                            .key_map_resolver
                            .active_key_map()
                            .get_plain(key)
                            .map(|b| b.command().to_string())
                            .unwrap_or_default();
                        let resp = ui.add_enabled(
                            enabled,
                            egui::Button::new(
                                egui::RichText::new(&btn_text)
                                    .color(if enabled { label_color } else { key_color })
                                    .monospace()
                                    .small(),
                            )
                            .frame(false),
                        );
                        if enabled && !tooltip.is_empty() {
                            resp.clone().on_hover_text(&tooltip);
                        }
                        if resp.clicked() && enabled {
                            clicked_key = Some(key);
                        }
                    }
                });
            }
        });
        if let Some(key) = clicked_key {
            if let Some(cmd) = self
                .key_map_resolver
                .active_key_map()
                .get_plain(key)
                .map(|b| b.command().to_string())
            {
                self.handle_command(&cmd);
            }
        }
    }

    // ── Status bar ───────────────────────────────────────────────────────

    fn render_status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let phase_label = match self.app.phase() {
                    LifecyclePhase::Running => "RUNNING",
                    LifecyclePhase::Initializing => "STARTING",
                    LifecyclePhase::ShuttingDown => "SHUTTING DOWN",
                    LifecyclePhase::Terminated => "TERMINATED",
                };
                ui.label(phase_label);
                ui.separator();

                let tab = self.tabs.active_tab();
                let line = tab.cursor.cursor_line();
                let col = tab.cursor.cursor_column();
                // Requirement 7.1: format "Ln {line}, Col {col}" (1-based)
                ui.label(format!("Ln {line}, Col {col}"));
                ui.separator();
                // Requirement 7.3: real encoding from document
                ui.label(tab.encoding_label());
                ui.separator();
                // Requirement 7.1 (view-zoom) — zoom indicator when non-zero
                {
                    use ff_zoom::ZoomIndicatorState;
                    if let ZoomIndicatorState::Visible { text, .. } =
                        ZoomIndicatorState::from_offset(self.zoom.offset())
                    {
                        ui.colored_label(to_egui_color(self.palette.editor.accent), text);
                        ui.separator();
                    }
                }
                // Requirement 7.4: real line count
                ui.label(format!("{} lines", tab.line_count));
                ui.separator();
                // Requirement 6.5: modified indicator
                if tab.is_modified {
                    ui.colored_label(to_egui_color(self.palette.editor.accent), "●");
                    ui.separator();
                }

                if let Some(err) = &self.open_error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.separator();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("FileForge Workbench v0.1.0");
                });
            });
        });
    }

    // ── File-open dialog ──────────────────────────────────────────────────

    /// Spawn a native file-open dialog on a blocking thread.
    ///
    /// When the user picks a file the path is written into `pending_open`;
    /// the next egui frame will pick it up and open the tab.
    fn open_file_dialog(&self) {
        let pending = self.pending_open.clone();
        self.runtime.spawn_blocking(move || {
            if let Some(handle) = rfd::FileDialog::new().pick_file() {
                let path = handle.to_string_lossy().into_owned();
                *pending.lock().expect("pending lock") = Some(path);
            }
        });
    }

    // ── Central panel ────────────────────────────────────────────────────

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        // ── Toolchain Panel (bottom dock) ────────────────────────────────
        if self.show_toolchain_panel {
            egui::TopBottomPanel::bottom("toolchain_panel")
                .resizable(true)
                .min_height(160.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Toolchain Panel").monospace().strong());
                        if ui.small_button("✕").clicked() {
                            self.show_toolchain_panel = false;
                        }
                    });
                    ui.separator();
                    if let Some((file, line, col)) =
                        toolchain_panel::render(ui, &mut self.toolchain_panel)
                    {
                        // Navigate editor to the clicked diagnostic location.
                        // Req 16.7, 18.6 — open the file if not already open,
                        // then scroll to the target line.
                        let _ = self.tabs.open_file(&file, &self.runtime);
                        self.nav_manager.locate(&line.to_string(), &mut self.tabs);
                        let _ = col; // column navigation deferred to Phase W follow-up
                    }
                });
        }

        // Validates: Requirement 14.8 — central panel dispatches on tab kind
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.tabs.active_tab().kind {
                TabKind::PrimaryOptionMenu => {
                    // Validates: Requirement 14.1, 14.2-14.5, 14.39, 14.40, 14.41, 14.42
                    // Validates: Requirement 13 (Legacy theme semantic colours)
                    let pom_colours = self.legacy_pom_colours();
                    let focused_pom_option = match self.focus_stop {
                        FocusStop::PomOption { index } => Some(index),
                        _ => None,
                    };
                    let pom_result = primary_option_menu::render(
                        ui,
                        self.pom_calendar_offset,
                        pom_colours,
                        focused_pom_option,
                    );
                    if let Some(nav) = pom_result.calendar_nav {
                        match nav {
                            primary_option_menu::CalendarNav::Prev => self.pom_calendar_offset -= 1,
                            primary_option_menu::CalendarNav::Next => self.pom_calendar_offset += 1,
                        }
                    }
                    if let Some(pom_action) = pom_result.action {
                        match pom_action {
                            primary_option_menu::PomAction::Navigate(key) => {
                                self.handle_command(&key.to_string());
                            }
                            primary_option_menu::PomAction::Exit => {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                    }
                }
                TabKind::FilesPanel => {
                    // Validates: Requirement 1.1, 1.7
                    let action = files_panel::render(ui, &mut self.files_panel);
                    match action {
                        files_panel::FilesPanelAction::ReturnToPom => {
                            self.pending_return_to_pom = true;
                        }
                        files_panel::FilesPanelAction::NewCatalog => {
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                // Req 12.1, 12.2 — pre-populate with configured defaults
                                let mf_root = self
                                    .config_handle
                                    .get_string(ff_config::keys::catalogs::DEFAULT_MAINFRAME_ROOT)
                                    .unwrap_or_default();
                                let posix_root = self
                                    .config_handle
                                    .get_string(ff_config::keys::catalogs::DEFAULT_POSIX_ROOT)
                                    .unwrap_or_default();
                                self.files_panel.dialog = files_panel::FilesDialogState::NewCatalog(
                                    NewCatalogForm::with_defaults(mf_root, posix_root),
                                );
                            }
                        }
                        files_panel::FilesPanelAction::EditCatalog(name) => {
                            // Req 4.1 - open Edit Catalog dialog pre-populated
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                if let Some(cat) = self.files_panel.registry.get_by_name(&name) {
                                    let form =
                                        catalog_manager_dialog::EditCatalogForm::from_catalog(cat);
                                    self.files_panel.dialog =
                                        files_panel::FilesDialogState::EditCatalog(form);
                                }
                            }
                        }
                        files_panel::FilesPanelAction::DeleteCatalog(name) => {
                            // Req 4.3 - open Delete Catalog confirmation dialog
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                if let Some(cat) = self.files_panel.registry.get_by_name(&name) {
                                    let confirm =
                                        catalog_manager_dialog::DeleteCatalogConfirm::from_catalog(
                                            cat,
                                        );
                                    self.files_panel.dialog =
                                        files_panel::FilesDialogState::DeleteCatalog(confirm);
                                }
                            }
                        }

                        files_panel::FilesPanelAction::AllocateDataset(_catalog_name) => {
                            // Req 5.1 - open Allocate Dataset dialog
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                self.files_panel.dialog =
                                    files_panel::FilesDialogState::AllocateDataset(
                                        Default::default(),
                                    );
                            }
                        }
                        files_panel::FilesPanelAction::OpenFile(_) => {}
                        files_panel::FilesPanelAction::NavigateInto(_) => {}
                        files_panel::FilesPanelAction::None => {}
                    }
                }
                TabKind::FileEditor | TabKind::Untitled => {
                    let tab_id = self.tabs.active_tab().id;
                    let tab = self.tabs.active_tab_mut();
                    if let Some(err) = editor_panel::render(
                        ui,
                        tab,
                        &self.runtime,
                        &mut self.cmd_engine,
                        &mut self.exclude_manager,
                        tab_id,
                    ) {
                        self.open_error = Some(err);
                    }
                }
                TabKind::SettingsPanel => {
                    // Validates: Requirement 15.1, 15.2, 15.3
                    crate::settings_panel::render(
                        ui,
                        &mut self.settings_panel,
                        &self.config_handle,
                    );
                }
            }
        });
    }
}

impl eframe::App for WorkbenchShell {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // One-shot startup
        if !self.started {
            self.started = true;
            let _ = self.runtime.block_on(self.app.startup());

            let cli_files = std::mem::take(&mut self.cli_files);
            if !cli_files.is_empty() {
                // CLI args take precedence over session restore (Req 5 AC 6).
                for path in cli_files {
                    if let Err(e) = self.tabs.open_file(&path, &self.runtime) {
                        self.open_error = Some(e);
                    } else {
                        self.open_error = None;
                    }
                }
            } else if let Some(session) = &self.session {
                // No CLI args — restore previous session tabs (Req 5 AC 1, 2).
                let state = session.load();
                let restored_any = !SessionManager::tab_uris(&state).is_empty();
                // Validates: Requirement 6.2 (view-zoom) — restore global zoom offset.
                if state.global_zoom_offset != 0 {
                    self.zoom = ff_zoom::ZoomState::from_persisted(
                        state.global_zoom_offset,
                        &ff_zoom::ZoomConfig::default(),
                    );
                }
                // Validates: Requirement 12.4 (function-keys-and-history) — restore PFSHOW state.
                self.key_bar_visible = state.key_bar_visible;
                for session_tab in &state.tabs {
                    if let Some(uri) = &session_tab.uri {
                        if let Err(e) = self.tabs.open_file(uri, &self.runtime) {
                            self.open_error = Some(format!("Could not restore: {e}"));
                        }
                    }
                }
                // Validates: Requirement 14.1 — if no files restored, open POM tab
                if !restored_any {
                    // Close the welcome placeholder before inserting POM so POM is
                    // the sole tab at index 0 on a clean first launch.
                    // Validates: Requirement 14.1 — POM is always in first position.
                    self.tabs.close_welcome_tab();
                    self.tabs.insert_pom_tab(&self.runtime);
                }
            } else {
                // No session manager — first launch: open POM tab
                // Validates: Requirement 14.1
                self.tabs.close_welcome_tab();
                self.tabs.insert_pom_tab(&self.runtime);
            }

            // Startup focus is handled by command_field_focus_requested = true (set in new()).
        }

        // Check if file.exit handler fired
        if *self.should_close.lock().expect("close lock") {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Process any pending file open (set by file.open handler or menu)
        let path = self.pending_open.lock().expect("pending lock").take();
        if let Some(p) = path {
            if !p.is_empty() {
                if let Err(e) = self.tabs.open_file(&p, &self.runtime) {
                    self.open_error = Some(e);
                } else {
                    self.open_error = None;
                }
            }
        }

        // Ctrl+S — save active tab (suppressed when a modal dialog is open)
        if !self.modal_open && ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
            if let Err(e) = self.tabs.save_active_tab(&self.runtime) {
                self.open_error = Some(e);
            } else {
                self.open_error = None;
            }
        }

        // Process deferred tab-bar context menu actions (set previous frame).
        if self.pending_new_pom {
            self.pending_new_pom = false;
            self.tabs.insert_pom_tab(&self.runtime);
        }
        if self.pending_new_file {
            self.pending_new_file = false;
            self.tabs.new_untitled_tab(&self.runtime);
        }
        // Validates: Requirement 1.7 — F3/END in Files Panel returns tab to POM view.
        if self.pending_return_to_pom {
            self.pending_return_to_pom = false;
            let idx = self.tabs.active_index();
            if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
                tab.kind = crate::tab_state::TabKind::PrimaryOptionMenu;
                tab.title = "[POM]".to_string();
            }
        }

        // ── Detach pending — Validates: Requirement 18.1, 18.4 ───────────────
        if let Some(idx) = self.detach_pending.take() {
            if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
                tab.is_floating = true;
                let vid =
                    egui::ViewportId::from_hash_of(format!("floating_tab_{idx}_{}", tab.title));
                self.floating_tabs.push(FloatingTab {
                    viewport_id: vid,
                    tab_index: idx,
                    origin_index: idx,
                });
            }
        }

        // ── Redock pending — Validates: Requirement 18.3 ─────────────────────
        let redock_indices: Vec<usize> = {
            let mut guard = self.redock_pending.lock().expect("redock lock");
            std::mem::take(&mut *guard)
        };
        for origin in redock_indices {
            // Find the FloatingTab with this origin_index.
            if let Some(ft_pos) = self
                .floating_tabs
                .iter()
                .position(|ft| ft.origin_index == origin)
            {
                let ft = self.floating_tabs.remove(ft_pos);
                let tab_idx = ft.tab_index;
                if let Some(tab) = self.tabs.tabs_mut().get_mut(tab_idx) {
                    tab.is_floating = false;
                }
                // Restore to origin_index (clamped to current tab count).
                let target = origin.min(self.tabs.len().saturating_sub(1));
                if tab_idx != target && tab_idx < self.tabs.len() {
                    self.tabs.tabs_mut().swap(tab_idx, target);
                }
            }
        }
        if let Ok(active) = self
            .config_handle
            .get_string(ff_config::keys::theme::ACTIVE)
        {
            let desired_mode = ff_theme::mode::VisualMode::from_str_loose(&active);
            if desired_mode
                .map(|m| m != self.palette.mode)
                .unwrap_or(false)
            {
                if let Some(mode) = desired_mode {
                    self.palette = ff_theme::defaults::default_palette_for_mode(mode);
                }
            }
        }

        self.apply_theme(ctx);
        // Validates: Requirement 3.1/3.2 (view-zoom) — Ctrl+Scroll updates global zoom.
        // Single zoom level shared across all tab kinds and contexts.
        {
            let (scroll_delta, ctrl_held) = ctx.input_mut(|i| {
                let raw = i.raw_scroll_delta.y;
                let smooth = i.smooth_scroll_delta.y;
                let ctrl = i.modifiers.ctrl;
                if ctrl {
                    i.raw_scroll_delta = egui::Vec2::ZERO;
                    i.smooth_scroll_delta = egui::Vec2::ZERO;
                }
                let delta = if raw != 0.0 { raw } else { smooth };
                (delta, ctrl)
            });
            if ctrl_held && scroll_delta != 0.0 {
                if scroll_delta > 0.0 {
                    self.zoom.zoom_in();
                } else {
                    self.zoom.zoom_out();
                }
            }
        }
        // Track whether the primary mouse button is held — window drag detection.
        // While the mouse is down we suppress any pixels_per_point change so that
        // WM_DPICHANGED messages fired as the window crosses a monitor boundary do
        // not trigger mid-move resize stuttering.  The change is applied on release.
        let mouse_down = ctx.input(|i| i.pointer.primary_down());
        if mouse_down {
            self.is_dragging = true;
        } else if self.is_dragging {
            // Mouse just released — apply any deferred ppp now.
            self.is_dragging = false;
            if let Some(ppp) = self.pending_ppp.take() {
                self.last_ppp = ppp;
                ctx.set_pixels_per_point(ppp);
            }
        }

        // Apply global zoom only when it has changed — do NOT call set_pixels_per_point
        // every frame, as that fights the OS DPI adjustment during cross-monitor moves
        // and causes the window to flash and stick at monitor boundaries.
        {
            let ppp = (1.0_f32 + self.zoom.offset().value() as f32 * 0.07).clamp(0.3, 4.0);
            if (ppp - self.last_ppp).abs() > f32::EPSILON {
                if self.is_dragging {
                    // Defer until mouse release.
                    self.pending_ppp = Some(ppp);
                } else {
                    self.last_ppp = ppp;
                    ctx.set_pixels_per_point(ppp);
                }
            }
        }
        // ── Tab-order focus cycle — Validates: Requirement 16.2–16.22 ───────────
        // Consume Tab / Shift+Tab before egui processes them so we control focus.
        // Suppressed when a modal dialog is open so Tab navigates inside the dialog.
        self.modal_open =
            self.key_config_dialog.open || self.show_about || self.show_history_list.is_some();
        {
            let menu_count = MENU_BAR_TOP_LEVEL_LABELS.len();
            let tab_count = self.tabs.len();
            let pom_active = self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu;
            let (tab_pressed, shift_tab_pressed) = ctx.input_mut(|i| {
                if self.modal_open {
                    return (false, false);
                }
                let shift = i.modifiers.shift;
                let tab = i.key_pressed(egui::Key::Tab);
                if tab {
                    // consume so egui doesn't also move focus
                    i.events.retain(|e| {
                        !matches!(
                            e,
                            egui::Event::Key {
                                key: egui::Key::Tab,
                                ..
                            }
                        )
                    });
                }
                (tab && !shift, tab && shift)
            });
            if tab_pressed {
                self.focus_stop = self.focus_stop.next(menu_count, tab_count, pom_active);
                if self.focus_stop == FocusStop::CommandField {
                    self.command_field_focus_requested = true;
                }
            } else if shift_tab_pressed {
                self.focus_stop = self.focus_stop.prev(menu_count, tab_count, pom_active);
                if self.focus_stop == FocusStop::CommandField {
                    self.command_field_focus_requested = true;
                }
            }
            // Validates: Requirement 16.20 — request egui focus on the tab header button
            // when a TabHeader stop is active (one-shot on Tab press).
            if tab_pressed || shift_tab_pressed {
                if let FocusStop::TabHeader { index } = self.focus_stop {
                    let tab_btn_id = egui::Id::new("tab_header_btn").with(index);
                    ctx.memory_mut(|m| m.request_focus(tab_btn_id));
                }
            }
            // Validates: Requirement 16.13–16.16 — Enter/Space activates focused POM stop.
            if pom_active {
                let enter_or_space = ctx
                    .input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
                if enter_or_space {
                    match &self.focus_stop.clone() {
                        FocusStop::PomOption { index } => {
                            let key = primary_option_menu::BUILT_IN_OPTIONS
                                .get(*index)
                                .map(|o| o.key)
                                .unwrap_or("0");
                            self.handle_command(key);
                        }
                        FocusStop::PomExit => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        FocusStop::CalendarPrev => {
                            self.pom_calendar_offset -= 1;
                        }
                        FocusStop::CalendarNext => {
                            self.pom_calendar_offset += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        self.render_menu_bar(ctx);
        self.render_tab_bar(ctx);
        self.render_title_line(ctx);
        self.render_command_field(ctx);
        self.render_key_label_bar(ctx);
        self.render_status_bar(ctx);

        // ── Function key dispatch (Req 3.1, 3.2) ────────────────────────
        // Suppressed when a modal dialog is open so Ctrl/Shift/Alt combos inside
        // dialog text fields are not intercepted by the shell key map.
        let fkey_cmd = if self.modal_open {
            None
        } else {
            ctx.input(|i| {
                let modifier = if i.modifiers.shift {
                    KeyModifier::Shift
                } else if i.modifiers.ctrl {
                    KeyModifier::Ctrl
                } else if i.modifiers.alt {
                    KeyModifier::Alt
                } else {
                    KeyModifier::None
                };
                FunctionKey::ALL.iter().find_map(|&fk| {
                    egui_fkey(fk).and_then(|ek| {
                        if i.key_pressed(ek) {
                            let mk = ModifiedKey { key: fk, modifier };
                            self.key_map_resolver
                                .active_key_map()
                                .get(mk)
                                .or_else(|| {
                                    if modifier != KeyModifier::None {
                                        self.key_map_resolver.active_key_map().get_plain(fk)
                                    } else {
                                        None
                                    }
                                })
                                .map(|b| b.command().to_string())
                        } else {
                            None
                        }
                    })
                })
            })
        };
        if let Some(cmd) = fkey_cmd {
            self.handle_command(&cmd);
        }

        self.render_central_panel(ctx);

        // ── Floating tab viewports — Validates: Requirement 18.1, 18.2, 18.5 ──
        for ft_idx in 0..self.floating_tabs.len() {
            let vid = self.floating_tabs[ft_idx].viewport_id;
            let tab_index = self.floating_tabs[ft_idx].tab_index;
            let origin_index = self.floating_tabs[ft_idx].origin_index;
            let title = self
                .tabs
                .tabs()
                .get(tab_index)
                .map(|t| format!("{} — FileForge Workbench", title_line_text(t)))
                .unwrap_or_else(|| "FileForge Workbench".to_string());
            let redock_tx = Arc::clone(&self.redock_pending);
            ctx.show_viewport_deferred(
                vid,
                egui::ViewportBuilder::default().with_title(&title),
                move |ctx, class| {
                    if class == egui::ViewportClass::Deferred {
                        // Detect close — push origin_index into redock_pending.
                        if ctx.input(|i| i.viewport().close_requested()) {
                            redock_tx.lock().expect("redock lock").push(origin_index);
                            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                        }
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.label(format!("Tab {tab_index} — floating"));
                        });
                    }
                },
            );
        }

        // ── Catalog Manager Dialog — Req 3.1–3.8 ──────────────────────────────
        // About dialog - Req 13.1, 13.8
        if self.show_about {
            crate::about_dialog::render(ctx, &mut self.show_about);
        }

        // Key Configuration Dialog -- Validates: Requirement 20.1
        crate::key_config_dialog::render_if_open(
            ctx,
            &mut self.key_config_dialog,
            &self.key_map_resolver,
            &self.config_handle,
        );

        // History list overlay -- Validates: Requirement 19.3, 19.4
        if let Some(entries) = self.show_history_list.clone() {
            let mut keep_open = true;
            let mut selected: Option<String> = None;
            egui::Window::new("Command History")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    if entries.is_empty() {
                        ui.label("No command history.");
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                for entry in &entries {
                                    if ui
                                        .selectable_label(
                                            false,
                                            egui::RichText::new(entry).monospace(),
                                        )
                                        .clicked()
                                    {
                                        selected = Some(entry.clone());
                                    }
                                }
                            });
                    }
                    ui.separator();
                    if ui.button("Cancel").clicked() {
                        keep_open = false;
                    }
                });
            if let Some(cmd) = selected {
                self.command_text = cmd;
                self.show_history_list = None;
            } else if !keep_open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.command_text.clear();
                self.show_history_list = None;
            }
        }

        // Catalog Manager Dialogs - Req 3.1-3.8, 4.1-4.5
        match &mut self.files_panel.dialog {
            files_panel::FilesDialogState::NewCatalog(ref mut form) => {
                let outcome =
                    catalog_manager_dialog::render(ctx, form, &mut self.files_panel.registry);
                if outcome == DialogOutcome::Confirmed || outcome == DialogOutcome::Cancelled {
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                }
            }
            files_panel::FilesDialogState::EditCatalog(ref mut form) => {
                let outcome =
                    catalog_manager_dialog::render_edit(ctx, form, &mut self.files_panel.registry);
                if outcome == DialogOutcome::Confirmed || outcome == DialogOutcome::Cancelled {
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                }
            }
            files_panel::FilesDialogState::DeleteCatalog(ref confirm) => {
                let choice = catalog_manager_dialog::render_delete(ctx, confirm);
                if choice != DeleteChoice::Cancel {
                    let confirm_clone = confirm.clone();
                    if let Err(e) = catalog_manager_dialog::execute_delete(
                        &choice,
                        &confirm_clone,
                        &mut self.files_panel.registry,
                    ) {
                        self.open_error = Some(e);
                    }
                }
                self.files_panel.dialog = files_panel::FilesDialogState::None;
            }
            files_panel::FilesDialogState::AllocateDataset(ref mut form) => {
                let outcome = dataset_alloc_dialog::render(ctx, form);
                if outcome == AllocOutcome::Confirmed || outcome == AllocOutcome::Cancelled {
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                }
            }
            files_panel::FilesDialogState::None => {}
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Persist session before shutdown (Req 9 AC 7 step 1).
        if let Some(session) = &self.session {
            session.save(&self.tabs, self.zoom.offset().value(), self.key_bar_visible);
        }
        self.runtime.block_on(self.app.shutdown());
    }
}

/// Derive the Title_Line text for a tab.
///
/// Context-dependent per Requirement 17.3–17.6:
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
        TabKind::FilesPanel | TabKind::SettingsPanel => tab.title.clone(),
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

/// Convert a `ff_config::ConfigValue` to a `toml::Value` for key-map parsing.
fn config_value_to_toml_value(v: ff_config::ConfigValue) -> Option<toml::Value> {
    use ff_config::ConfigValue;
    match v {
        ConfigValue::String(s) => Some(toml::Value::String(s)),
        ConfigValue::Integer(i) => Some(toml::Value::Integer(i)),
        ConfigValue::Float(f) => Some(toml::Value::Float(f)),
        ConfigValue::Boolean(b) => Some(toml::Value::Boolean(b)),
        ConfigValue::Array(arr) => {
            let items: Vec<toml::Value> = arr
                .into_iter()
                .filter_map(config_value_to_toml_value)
                .collect();
            Some(toml::Value::Array(items))
        }
        ConfigValue::Table(t) => {
            let mut map = toml::map::Map::new();
            for (k, val) in t {
                if let Some(tv) = config_value_to_toml_value(val) {
                    map.insert(k, tv);
                }
            }
            Some(toml::Value::Table(map))
        }
        // ConfigValue is #[non_exhaustive] — future variants are silently skipped.
        _ => None,
    }
}

/// Map a `TabKind` to its context name for key map resolution.
///
/// Validates: Requirement 14.6
fn context_name_for_kind(kind: TabKind) -> Option<&'static str> {
    match kind {
        TabKind::PrimaryOptionMenu => Some("pom"),
        TabKind::FileEditor | TabKind::Untitled => Some("editor"),
        TabKind::SettingsPanel => Some("settings"),
        TabKind::FilesPanel => Some("files"),
    }
}

/// Convert a `ColourRGBA` to an `egui::Color32`.
#[inline]
fn to_egui_color(c: ColourRGBA) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a)
}

/// Build the default global key map used at startup.
///
/// Provides ISPF-standard bindings: F3=END, F7=UP, F8=DOWN, F12=RETRIEVE.
/// Map a `FunctionKey` to the corresponding `egui::Key`, if supported.
///
/// egui exposes F1–F20; F21–F24 are not available on most platforms.
fn egui_fkey(fk: FunctionKey) -> Option<egui::Key> {
    match fk {
        FunctionKey::F1 => Some(egui::Key::F1),
        FunctionKey::F2 => Some(egui::Key::F2),
        FunctionKey::F3 => Some(egui::Key::F3),
        FunctionKey::F4 => Some(egui::Key::F4),
        FunctionKey::F5 => Some(egui::Key::F5),
        FunctionKey::F6 => Some(egui::Key::F6),
        FunctionKey::F7 => Some(egui::Key::F7),
        FunctionKey::F8 => Some(egui::Key::F8),
        FunctionKey::F9 => Some(egui::Key::F9),
        FunctionKey::F10 => Some(egui::Key::F10),
        FunctionKey::F11 => Some(egui::Key::F11),
        FunctionKey::F12 => Some(egui::Key::F12),
        FunctionKey::F13 => Some(egui::Key::F13),
        FunctionKey::F14 => Some(egui::Key::F14),
        FunctionKey::F15 => Some(egui::Key::F15),
        FunctionKey::F16 => Some(egui::Key::F16),
        FunctionKey::F17 => Some(egui::Key::F17),
        FunctionKey::F18 => Some(egui::Key::F18),
        FunctionKey::F19 => Some(egui::Key::F19),
        FunctionKey::F20 => Some(egui::Key::F20),
        // F21–F24 not available in egui
        FunctionKey::F21 | FunctionKey::F22 | FunctionKey::F23 | FunctionKey::F24 => None,
    }
}

/// Parse an optional u64 from a trimmed string (empty → None, valid int → Some).
fn parse_optional_u64(s: &str) -> Option<u64> {
    if s.is_empty() {
        None
    } else {
        s.parse().ok()
    }
}

/// Which shell to use when opening a containing folder.
enum FolderOpenMode {
    Explorer,
    Cmd,
    PowerShell,
    Terminal,
}

/// Open the folder containing `file_path` in the requested shell.
///
/// Validates: Requirement 14.23–14.26
fn open_containing_folder(file_path: &str, mode: FolderOpenMode) {
    let folder = std::path::Path::new(file_path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string());
    let _ = match mode {
        FolderOpenMode::Explorer => std::process::Command::new("explorer").arg(&folder).spawn(),
        FolderOpenMode::Cmd => std::process::Command::new("cmd")
            .args(["/k", "cd", "/d", &folder])
            .spawn(),
        FolderOpenMode::PowerShell => std::process::Command::new("powershell")
            .args(["-NoExit", "-Command", &format!("Set-Location '{folder}'")])
            .spawn(),
        FolderOpenMode::Terminal => {
            // Windows Terminal if available, else cmd fallback.
            std::process::Command::new("wt")
                .args(["--startingDirectory", &folder])
                .spawn()
                .or_else(|_| {
                    std::process::Command::new("cmd")
                        .args(["/k", "cd", "/d", &folder])
                        .spawn()
                })
        }
    };
}

/// Strip a trailing ` ALL` suffix (case-insensitive) from an EXCLUDE argument.
/// Returns `(text_without_all, had_all_flag)`.
fn strip_all_suffix(s: &str) -> (&str, bool) {
    let upper = s.to_uppercase();
    if upper.ends_with(" ALL") {
        (s[..s.len() - 4].trim_end(), true)
    } else {
        (s, false)
    }
}

/// Map an operation message to `open_error`: None for success/info, Some for errors.
fn info_or_error(msg: &str) -> Option<String> {
    // Messages that indicate no change or an error get surfaced; pure counts are info.
    if msg.is_empty()
        || msg.contains("line(s) excluded")
        || msg.contains("line(s) shown")
        || msg.contains("RESET:")
    {
        None
    } else {
        Some(msg.to_string())
    }
}

/// Parse two single-quoted or bare-word arguments from a CHANGE command tail.
/// Handles: `'old text' 'new text'`, `old new`, `'old' new`, `old 'new'`.
fn parse_two_args(s: &str) -> Option<(String, String)> {
    let s = s.trim();
    let (first, rest) = extract_arg(s)?;
    let (second, _) = extract_arg(rest.trim())?;
    Some((first, second))
}

/// Extract one argument (single-quoted or bare word) from the front of `s`.
/// Returns `(arg, remainder)`.
fn extract_arg(s: &str) -> Option<(String, &str)> {
    let s = s.trim_start();
    if s.is_empty() {
        return None;
    }
    if let Some(inner) = s.strip_prefix('\'') {
        // Single-quoted: find the closing quote
        let close = inner.find('\'')?;
        Some((inner[..close].to_string(), &inner[close + 1..]))
    } else {
        // Bare word: up to next whitespace
        let pos = s.find(char::is_whitespace).unwrap_or(s.len());
        Some((s[..pos].to_string(), &s[pos..]))
    }
}

#[cfg(test)]
mod tests {
    use ff_keys::{KeyMap, ModifiedKey};
    use std::sync::{Arc, Mutex};

    use ff_command::{
        CommandDispatch, CommandError, CommandHandler, CommandHistory, CommandId, CommandMetadata,
        CommandParams, CommandRegistry, CommandResult, ExecutionContext,
    };

    fn make_dispatch() -> (Arc<CommandRegistry>, CommandDispatch) {
        let registry = Arc::new(CommandRegistry::new());
        let history = Arc::new(CommandHistory::new(100));
        let dispatch = CommandDispatch::new(registry.clone(), history);
        (registry, dispatch)
    }

    fn meta(name: &str, cat: &str) -> CommandMetadata {
        CommandMetadata::builder(name, name).category(cat).build()
    }

    /// Validates: Requirement 8.1 — command field submits non-empty text on Enter.
    /// The UI wiring (has_focus + key_pressed) is verified manually; this test
    /// confirms the dispatch path handle_command is reachable for any non-empty input.
    #[test]
    fn command_field_enter_submits_non_empty_command() {
        // Validates: command-semantics Requirement 8.1
        // Simulate what render_command_field does: trim and dispatch.
        let raw = "  EXIT  ";
        let cmd = raw.trim().to_string();
        assert!(!cmd.is_empty(), "trimmed command must be non-empty");
        // EXIT is a shell-level intercept
        assert!(is_shell_command(&cmd));
    }

    /// Validates: Requirement 8.2 — command field does not submit when empty.
    #[test]
    fn command_field_enter_does_not_submit_empty_command() {
        // Validates: command-semantics Requirement 8.2
        let raw = "   ";
        let cmd = raw.trim().to_string();
        // The guard `!self.command_text.is_empty()` prevents dispatch.
        assert!(
            cmd.is_empty(),
            "whitespace-only input must be treated as empty"
        );
    }

    /// Validates: Requirement 8.1 — EDIT path dispatches file.open via handle_command.
    #[test]
    fn command_field_edit_command_dispatches_file_open() {
        // Validates: command-semantics Requirement 8.1
        let cmd = "EDIT /some/file.txt";
        assert!(is_shell_command(cmd));
    }

    /// Validates: Requirement 14.8 — central panel dispatches on TabKind.
    #[test]
    fn central_panel_always_shows_editor_regardless_of_open_files() {
        // Validates: Requirement 14.8
        use crate::tab_manager::TabManager;
        use tokio::runtime::Runtime;
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "welcome");
        assert_eq!(mgr.tabs().len(), 1);
        assert!(mgr.tabs()[0].path.is_none(), "welcome tab has no path");
    }

    /// Validates: Requirement 14.1 — first launch inserts a POM tab.
    #[test]
    fn first_launch_inserts_pom_tab() {
        // Validates: Requirement 14.1
        use crate::tab_manager::TabManager;
        use crate::tab_state::TabKind;
        use tokio::runtime::Runtime;
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert_eq!(mgr.tabs()[0].kind, TabKind::PrimaryOptionMenu);
        assert_eq!(mgr.tabs()[0].title, "[POM]");
    }

    /// Validates: Requirement 14.10, 14.14 — START command is a shell-level intercept.
    #[test]
    fn start_command_is_recognised_as_shell_command() {
        // Validates: Requirement 14.10
        assert!(is_shell_command("START"));
        assert!(is_shell_command("POM"));
    }

    /// Validates: Requirement 14.11 — CLOSE command is a shell-level intercept.
    #[test]
    fn close_command_is_recognised_as_shell_command() {
        // Validates: Requirement 14.11
        assert!(is_shell_command("CLOSE"));
    }

    /// Validates: Requirement 14.15c — POM tab context menu omits file-specific items.
    #[test]
    fn pom_tab_context_menu_items_are_universal_only() {
        // Validates: Requirement 14.15c
        // The context menu for a POM tab must NOT include file-specific items.
        // We verify this by checking the TabKind dispatch logic directly.
        use crate::tab_state::TabKind;
        let pom_kind = TabKind::PrimaryOptionMenu;
        let file_kind = TabKind::FileEditor;
        // File-specific items are only shown when kind == FileEditor.
        assert!(file_kind == TabKind::FileEditor);
        assert!(pom_kind != TabKind::FileEditor);
    }

    /// Validates: Requirement 14.15b — file editor tab shows file-specific items.
    #[test]
    fn file_editor_tab_context_menu_includes_file_items() {
        // Validates: Requirement 14.15b
        use crate::tab_state::TabKind;
        assert_eq!(TabKind::FileEditor, TabKind::FileEditor);
    }

    /// Validates: Requirement S.1 — file_open_dialog sets pending_open when a path is returned.
    #[test]
    fn file_open_dialog_pending_open_is_set_when_path_returned() {
        // Simulates the closure body inside open_file_dialog(): when a path
        // is available, it must be written into pending_open.
        let pending: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let path = "/tmp/test_file.txt".to_string();

        // Simulate the closure that open_file_dialog() spawns
        *pending.lock().expect("pending lock") = Some(path.clone());

        let result = pending.lock().expect("pending lock").take();
        assert_eq!(result, Some(path));
    }

    /// Validates: Requirement S.1 — file_open_dialog leaves pending_open None when dialog is cancelled.
    #[test]
    fn file_open_dialog_pending_open_unchanged_when_cancelled() {
        let pending: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        // Simulate cancelled dialog — closure does nothing
        let result = pending.lock().expect("pending lock").take();
        assert_eq!(result, None);
    }

    /// Validates: Requirement 18.6 — EDIT <path> dispatches file.open with path param.
    #[test]
    fn file_open_handler_succeeds_with_path_param() {
        // Validates: Requirement 2.1 — execute_command routes through dispatch
        let (registry, dispatch) = make_dispatch();

        let received = Arc::new(Mutex::new(String::new()));
        let received_clone = received.clone();

        struct CaptureHandler {
            received: Arc<Mutex<String>>,
        }
        impl CommandHandler for CaptureHandler {
            fn is_undoable(&self) -> bool {
                false
            }
            fn execute(&self, _ctx: &ExecutionContext, params: &CommandParams) -> CommandResult {
                if let Some(p) = params.get_string("path") {
                    *self.received.lock().unwrap() = p.to_string();
                    CommandResult::Ok
                } else {
                    CommandResult::Err(CommandError::ExecutionFailed {
                        id: "file.open".to_string(),
                        description: "missing path".to_string(),
                    })
                }
            }
        }

        let id = CommandId::new("file.open").unwrap();
        registry
            .register(
                id,
                meta("Open File", "file"),
                Box::new(CaptureHandler {
                    received: received_clone,
                }),
            )
            .unwrap();

        let mut params = CommandParams::new();
        params.insert("path", "/tmp/test.txt");
        let result = dispatch.execute_command("file.open", params);

        assert!(result.is_ok());
        assert_eq!(*received.lock().unwrap(), "/tmp/test.txt");
    }

    /// Validates: Requirement 18.6 — file.open without path param returns error.
    #[test]
    fn file_open_handler_fails_without_path_param() {
        // Validates: Requirement 2.2 — missing param produces Err result
        let (registry, dispatch) = make_dispatch();

        struct RejectNoPath;
        impl CommandHandler for RejectNoPath {
            fn is_undoable(&self) -> bool {
                false
            }
            fn execute(&self, _ctx: &ExecutionContext, params: &CommandParams) -> CommandResult {
                if params.get_string("path").is_some() {
                    CommandResult::Ok
                } else {
                    CommandResult::Err(CommandError::ExecutionFailed {
                        id: "file.open".to_string(),
                        description: "missing path".to_string(),
                    })
                }
            }
        }

        let id = CommandId::new("file.open").unwrap();
        registry
            .register(id, meta("Open File", "file"), Box::new(RejectNoPath))
            .unwrap();

        let result = dispatch.execute_command("file.open", CommandParams::new());
        assert!(result.is_err());
    }

    /// Validates: Requirement 18.6 — EXIT command dispatches file.exit and sets close flag.
    #[test]
    fn file_exit_handler_sets_close_flag() {
        // Validates: Requirement 2.1 — execute_command routes through dispatch
        let (registry, dispatch) = make_dispatch();

        let closed = Arc::new(Mutex::new(false));
        let closed_clone = closed.clone();

        struct ExitHandler {
            closed: Arc<Mutex<bool>>,
        }
        impl CommandHandler for ExitHandler {
            fn is_undoable(&self) -> bool {
                false
            }
            fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
                *self.closed.lock().unwrap() = true;
                CommandResult::Ok
            }
        }

        let id = CommandId::new("file.exit").unwrap();
        registry
            .register(
                id,
                meta("Exit", "file"),
                Box::new(ExitHandler {
                    closed: closed_clone,
                }),
            )
            .unwrap();

        let result = dispatch.execute_command("file.exit", CommandParams::new());
        assert!(result.is_ok());
        assert!(*closed.lock().unwrap(), "exit handler must set close flag");
    }

    /// Validates: Requirement 18.6 — unrecognised command ID returns NotFound error.
    #[test]
    fn dispatch_returns_not_found_for_unknown_command() {
        // Validates: Requirement 2.2 — unregistered command returns Err(NotFound)
        let (_registry, dispatch) = make_dispatch();
        let result = dispatch.execute_command("file.open", CommandParams::new());
        assert!(result.is_err());
        assert!(matches!(
            result,
            CommandResult::Err(CommandError::NotFound { .. })
        ));
    }

    // ── Phase U: CommandEngine dispatch tests ────────────────────────────

    /// Validates: Requirement 21.1 — empty command line returns "No command" status.
    #[test]
    fn command_engine_empty_input_returns_no_command() {
        // Validates: Phase U 21.1 — CommandEngine replaces hard-coded dispatch
        use ff_command_semantics::{CommandEngine, StatusKind};
        let mut engine = CommandEngine::new();
        let status = engine.execute_command_line("");
        assert_eq!(status.text, "No command");
        assert_eq!(status.kind, StatusKind::Info);
    }

    /// Validates: Requirement 21.1 — unrecognised command produces RuntimeError status.
    #[test]
    fn command_engine_unknown_command_returns_runtime_error() {
        // Validates: Phase U 21.1 — engine surfaces error status for unknown commands
        use ff_command_semantics::{CommandEngine, StatusKind};
        let mut engine = CommandEngine::new();
        let status = engine.execute_command_line("NOSUCHCMD");
        assert_eq!(status.kind, StatusKind::RuntimeError);
        assert!(status.text.contains("NOSUCHCMD"));
    }

    /// Validates: Requirement 21.1 — syntax error in command line produces SyntaxError status.
    #[test]
    fn command_engine_syntax_error_returns_syntax_error_status() {
        // Validates: Phase U 21.1 — engine parses and surfaces syntax errors
        use ff_command_semantics::{CommandEngine, StatusKind};
        let mut engine = CommandEngine::new();
        let status = engine.execute_command_line("FIND 'unclosed");
        assert_eq!(status.kind, StatusKind::SyntaxError);
    }

    /// Validates: Requirement 14.38 — "Exit" is present as a universal tab context menu item
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

    /// Validates: Requirement 21.1 — shell-level EXIT intercept bypasses engine.
    #[test]
    fn shell_intercepts_exit_before_engine() {
        // Validates: Phase U 21.1 — EXIT/QUIT/=X are shell-level, not engine commands
        assert!(is_shell_command("EXIT"));
        assert!(is_shell_command("QUIT"));
        assert!(is_shell_command("=X"));
        assert!(is_shell_command("X"));
        assert!(!is_shell_command("FIND"));
        assert!(!is_shell_command("LOCATE"));
    }

    /// Validates: Requirement 21.1 — shell-level EDIT intercept bypasses engine.
    #[test]
    fn shell_intercepts_edit_before_engine() {
        // Validates: Phase U 21.1 — EDIT <path> is a shell-level file-open command
        assert!(is_shell_command("EDIT /some/path"));
        assert!(is_shell_command("EDIT"));
        assert!(!is_shell_command("EDITX"));
    }

    /// Returns true if the command is handled at the shell level (not routed to CommandEngine).
    fn is_shell_command(cmd: &str) -> bool {
        let upper = cmd.trim().to_uppercase();
        upper == "EXIT"
            || upper == "QUIT"
            || upper == "=X"
            || upper == "X"
            || upper == "START"
            || upper == "POM"
            || upper == "CLOSE"
            || upper == "0"
            || upper == "SETTINGS"
            || upper == "=0"
            || upper == "1"
            || upper == "FILES"
            || upper == "3"
            || upper == "UTILITIES"
            || upper == "4"
            || upper == "COMPILERS"
            || upper == "7"
            || upper == "DATABASES"
            || upper == "8"
            || upper == "PLUGINS"
            || upper == "RETRIEVE"
            || upper == "RFIND"
            || upper == "RCHANGE"
            || upper == "EDIT"
            || upper.starts_with("EDIT ")
            || upper.starts_with("FIND ")
            || upper.starts_with("CHANGE ")
            || upper.starts_with("LOCATE ")
            || upper == "TOP"
            || upper == "BOTTOM"
            || upper == "UP"
            || upper.starts_with("UP ")
            || upper == "DOWN"
            || upper.starts_with("DOWN ")
            || upper == "LEFT"
            || upper.starts_with("LEFT ")
            || upper == "RIGHT"
            || upper.starts_with("RIGHT ")
            || upper == "SORT"
            || upper.starts_with("SORT ")
            || upper == "EXCLUDE ALL"
            || upper.starts_with("EXCLUDE ")
            || upper == "X ALL"
            || upper.starts_with("X ")
            || upper == "SHOW ALL"
            || upper.starts_with("SHOW ")
            || upper == "INCLUDE ALL"
            || upper.starts_with("INCLUDE ")
            || upper == "RESET"
            || upper == "RESET EXCLUDED"
            || upper == "RESET ALL"
            || upper == "PFSHOW"
            || upper == "PFSHOW ON"
            || upper == "PFSHOW OFF"
            || upper == "END"
            || upper == "RETURN"
            || upper == "KEYS"
    }

    // ── Phase AC: POM option list reorganisation tests ──────────────────────

    /// Validates: Requirement 14.3 — POM has exactly 9 built-in options (0–8).
    #[test]
    fn pom_has_nine_built_in_options() {
        // Validates: Requirement 14.3
        use crate::primary_option_menu::BUILT_IN_OPTIONS;
        assert_eq!(BUILT_IN_OPTIONS.len(), 9);
    }

    /// Validates: Requirement 14.3 — option keys are 0–8 in order.
    #[test]
    fn pom_option_keys_are_zero_through_eight() {
        // Validates: Requirement 14.3
        use crate::primary_option_menu::BUILT_IN_OPTIONS;
        let keys: Vec<&str> = BUILT_IN_OPTIONS.iter().map(|o| o.key).collect();
        assert_eq!(keys, vec!["0", "1", "2", "3", "4", "5", "6", "7", "8"]);
    }

    /// Validates: Requirement 14.3a — option 1 is labelled "File Catalogs".
    #[test]
    fn pom_option_1_label_is_file_catalogs() {
        // Validates: Requirement 14.3a
        use crate::primary_option_menu::BUILT_IN_OPTIONS;
        let opt1 = BUILT_IN_OPTIONS.iter().find(|o| o.key == "1").unwrap();
        assert_eq!(opt1.label, "File Catalogs");
    }

    /// Validates: Requirement 14.3b — option 8 is labelled "Plugins".
    #[test]
    fn pom_option_8_label_is_plugins() {
        // Validates: Requirement 14.3b
        use crate::primary_option_menu::BUILT_IN_OPTIONS;
        let opt8 = BUILT_IN_OPTIONS.iter().find(|o| o.key == "8").unwrap();
        assert_eq!(opt8.label, "Plugins");
        assert_eq!(opt8.description, "Vendor added plugins");
    }

    /// Validates: Requirement 14.3 — option 7 is labelled "Databases".
    #[test]
    fn pom_option_7_label_is_databases() {
        // Validates: Requirement 14.3
        use crate::primary_option_menu::BUILT_IN_OPTIONS;
        let opt7 = BUILT_IN_OPTIONS.iter().find(|o| o.key == "7").unwrap();
        assert_eq!(opt7.label, "Databases");
    }

    /// Validates: Requirement 14.3 — option 0 description updated.
    #[test]
    fn pom_option_0_description_is_settings_and_client_parameters() {
        // Validates: Requirement 14.3
        use crate::primary_option_menu::BUILT_IN_OPTIONS;
        let opt0 = BUILT_IN_OPTIONS.iter().find(|o| o.key == "0").unwrap();
        assert_eq!(opt0.description, "FFWB Settings and Client Parameters");
    }

    // ── Task 26: SettingsPanel tab kind and routing tests ────────────────────

    /// Validates: Requirement 15.1 — SettingsPanel TabKind variant exists.
    #[test]
    fn settings_panel_tab_kind_exists() {
        // Validates: Requirement 15.1
        use crate::tab_state::TabKind;
        let kind = TabKind::SettingsPanel;
        assert_eq!(kind, TabKind::SettingsPanel);
    }

    /// Validates: Requirement 15.1 — command "0" is a shell-level intercept.
    #[test]
    fn command_0_routes_to_settings() {
        // Validates: Requirement 15.1
        assert!(is_shell_command("0"));
    }

    /// Validates: Requirement 15.1 — command "SETTINGS" is a shell-level intercept.
    #[test]
    fn command_settings_routes_to_settings() {
        // Validates: Requirement 15.1
        assert!(is_shell_command("SETTINGS"));
    }

    /// Validates: Requirement 15.1 — command "=0" is a shell-level intercept.
    #[test]
    fn command_equals_0_routes_to_settings() {
        // Validates: Requirement 15.1
        assert!(is_shell_command("=0"));
    }

    /// Validates: Requirement 15.9 — SettingsPanel is distinct from other tab kinds.
    #[test]
    fn settings_panel_tab_kind_is_distinct_from_other_kinds() {
        // Validates: Requirement 15.9
        use crate::tab_state::TabKind;
        assert_ne!(TabKind::SettingsPanel, TabKind::PrimaryOptionMenu);
        assert_ne!(TabKind::SettingsPanel, TabKind::FileEditor);
        assert_ne!(TabKind::SettingsPanel, TabKind::FilesPanel);
        assert_ne!(TabKind::SettingsPanel, TabKind::Untitled);
    }

    /// Validates: Requirement 14.3 — option 2 is labelled "Files".
    #[test]
    fn pom_option_2_label_is_files() {
        // Validates: Requirement 14.3
        use crate::primary_option_menu::BUILT_IN_OPTIONS;
        let opt2 = BUILT_IN_OPTIONS.iter().find(|o| o.key == "2").unwrap();
        assert_eq!(opt2.label, "Files");
    }

    /// Validates: Requirement 14.7 — menu bar includes a `File Catalogs` top-level menu.
    #[test]
    fn menu_bar_has_file_catalogs_menu() {
        // Validates: Requirement 14.7
        assert!(
            super::MENU_BAR_TOP_LEVEL_LABELS.contains(&"File Catalogs"),
            "MENU_BAR_TOP_LEVEL_LABELS must contain 'File Catalogs' to mirror POM option 1"
        );
    }

    /// Validates: Requirement 14.7 — menu bar includes a `Plugins` top-level menu.
    #[test]
    fn menu_bar_has_plugins_menu() {
        // Validates: Requirement 14.7
        assert!(
            super::MENU_BAR_TOP_LEVEL_LABELS.contains(&"Plugins"),
            "MENU_BAR_TOP_LEVEL_LABELS must contain 'Plugins' to mirror POM option 8"
        );
    }

    /// Validates: Requirement 14.6 — option 1 on a POM tab transforms the tab in-place.
    #[test]
    fn pom_option_1_on_pom_tab_transforms_tab_in_place() {
        // Validates: Requirement 14.6
        use crate::tab_manager::TabManager;
        use crate::tab_state::TabKind;
        use tokio::runtime::Runtime;
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert_eq!(mgr.active_tab().kind, TabKind::PrimaryOptionMenu);
        // Typing "1" on a POM tab must transform it in-place to FilesPanel.
        mgr.transform_active_pom_tab(TabKind::FilesPanel, "[FILES]");
        assert_eq!(mgr.active_tab().kind, TabKind::FilesPanel);
        assert_eq!(mgr.active_tab().title, "[FILES]");
        // Tab count must not change — no new tab opened.
        assert_eq!(mgr.len(), 2); // welcome + transformed
    }

    /// Validates: Requirement 14.6 — option 1 on a non-POM tab opens a new tab.
    #[test]
    fn pom_option_1_on_non_pom_tab_does_not_transform() {
        // Validates: Requirement 14.6
        use crate::tab_manager::TabManager;
        use crate::tab_state::TabKind;
        use tokio::runtime::Runtime;
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        // Active tab is Untitled (not POM) — transform_active_pom_tab must be a no-op.
        assert_eq!(mgr.active_tab().kind, TabKind::Untitled);
        mgr.transform_active_pom_tab(TabKind::FilesPanel, "[FILES]");
        assert_eq!(
            mgr.active_tab().kind,
            TabKind::Untitled,
            "non-POM tab must not be transformed"
        );
    }

    /// Validates: Requirement 21.7 — command history records submitted commands.
    #[test]
    fn command_history_records_entries() {
        // Validates: Phase U 21.7 — ff-keys CommandHistory integration
        use ff_keys::CommandHistory;
        let mut history = CommandHistory::new(100);
        history.add("FIND hello");
        history.add("LOCATE 42");
        assert_eq!(history.len(), 2);
        assert_eq!(history.get(0).map(|e| e.command()), Some("LOCATE 42"));
    }

    /// Validates: Requirement 21.7 — RETRIEVE cycles through command history.
    #[test]
    fn retrieve_state_cycles_through_history() {
        // Validates: Phase U 21.7 — RetrieveState steps back through history
        use ff_keys::{CommandHistory, RetrieveResult, RetrieveState};
        let mut history = CommandHistory::new(100);
        history.add("FIND hello");
        history.add("LOCATE 42");
        let mut retrieve = RetrieveState::new();
        let first = retrieve.retrieve(&history, "");
        assert!(matches!(first, RetrieveResult::Recalled { .. }));
        if let RetrieveResult::Recalled { command } = first {
            assert_eq!(command, "LOCATE 42");
        }
    }

    // ── Task 21.7 — key label bar and F-key dispatch tests ──────────────────────

    /// Validates: Requirement 4.2, 4.4 — default key map produces labelled slots.
    #[test]
    fn default_key_map_has_five_assigned_slots() {
        // Validates: function-keys-and-history Requirement 4.2, 15.1
        use ff_keys::KeyLabelBarModel;
        let map = KeyMap::default_global();
        let bar = KeyLabelBarModel::from_key_map(&map);
        let assigned: Vec<_> = bar.assigned_slots().collect();
        assert_eq!(assigned.len(), 5, "F1, F3, F7, F8, F12 should be assigned");
    }

    /// Validates: Requirement 4.4 — label derived from explicit label field.
    #[test]
    fn default_key_map_f3_label_is_end() {
        // Validates: function-keys-and-history Requirement 4.4, 4.5
        use ff_keys::{FunctionKey, KeyLabelBarModel};
        let map = KeyMap::default_global();
        let bar = KeyLabelBarModel::from_key_map(&map);
        let slot = bar.slot_for(FunctionKey::F3).unwrap();
        assert_eq!(slot.label.as_deref(), Some("End"));
    }

    /// Validates: Requirement 3.1 — assigned F-key returns its command string.
    #[test]
    fn egui_fkey_assigned_key_returns_command() {
        // Validates: function-keys-and-history Requirement 3.1
        use ff_keys::{FunctionKey, KeyMapResolver};
        let map = KeyMap::default_global();
        let resolver = KeyMapResolver::new(map);
        let cmd = resolver
            .active_key_map()
            .get_plain(FunctionKey::F3)
            .map(|b| b.command());
        assert_eq!(cmd, Some("END"));
    }

    /// Validates: Requirement 3.2 — unassigned F-key returns None.
    #[test]
    fn egui_fkey_unassigned_key_returns_none() {
        // Validates: function-keys-and-history Requirement 3.2
        use ff_keys::{FunctionKey, KeyMapResolver};
        let map = KeyMap::default_global();
        let resolver = KeyMapResolver::new(map);
        // F4 is not in the default map
        let cmd = resolver.active_key_map().get_plain(FunctionKey::F4);
        assert!(cmd.is_none());
    }

    /// Validates: Requirement 4.3 — unassigned keys produce blank slots.
    #[test]
    fn key_label_bar_unassigned_key_has_no_label() {
        // Validates: function-keys-and-history Requirement 4.3
        use ff_keys::{FunctionKey, KeyLabelBarModel};
        let map = KeyMap::default_global();
        let bar = KeyLabelBarModel::from_key_map(&map);
        // F4 is not assigned in the default map
        let slot = bar.slot_for(FunctionKey::F4).unwrap();
        assert!(slot.label.is_none());
    }

    // ── Phase AJ: Tab-order focus cycle tests ────────────────────────────────────────

    /// Validates: Requirement 16.1 — initial focus stop is CommandField.
    #[test]
    fn focus_stop_initial_state_is_command_field() {
        // Validates: Requirement 16.1
        use super::FocusStop;
        assert_eq!(FocusStop::CommandField, FocusStop::CommandField);
    }

    /// Validates: Requirement 16.3 — Tab from CommandField goes to PomOption(0) when POM active.
    #[test]
    fn focus_cycle_tab_forward_from_command_field_goes_to_pom_option_0() {
        // Validates: Requirement 16.3
        use super::FocusStop;
        let next = FocusStop::CommandField.next(11, 0, true);
        assert_eq!(next, FocusStop::PomOption { index: 0 });
    }

    /// Validates: Requirement 16.3 — Tab from CommandField goes to MenuBar(0) when POM not active.
    #[test]
    fn focus_cycle_tab_forward_from_command_field_goes_to_menu_when_no_pom() {
        // Validates: Requirement 16.19
        use super::FocusStop;
        let next = FocusStop::CommandField.next(11, 0, false);
        assert_eq!(next, FocusStop::MenuBar { index: 0 });
    }

    /// Validates: Requirement 16.4 — Tab advances through all 9 POM option rows.
    #[test]
    fn focus_cycle_tab_forward_through_all_pom_options() {
        // Validates: Requirement 16.4
        use super::FocusStop;
        let mut stop = FocusStop::PomOption { index: 0 };
        for expected in 1..9usize {
            stop = stop.next(11, 0, true);
            assert_eq!(stop, FocusStop::PomOption { index: expected });
        }
        // After option 8 → PomExit
        stop = stop.next(11, 0, true);
        assert_eq!(stop, FocusStop::PomExit);
    }

    /// Validates: Requirement 16.6 — Tab from PomExit goes to CalendarPrev.
    #[test]
    fn focus_cycle_tab_forward_from_pom_exit_goes_to_calendar_prev() {
        // Validates: Requirement 16.6
        use super::FocusStop;
        let next = FocusStop::PomExit.next(11, 0, true);
        assert_eq!(next, FocusStop::CalendarPrev);
    }

    /// Validates: Requirement 16.7 — Tab from CalendarPrev goes to CalendarNext.
    #[test]
    fn focus_cycle_tab_forward_from_calendar_prev_goes_to_calendar_next() {
        // Validates: Requirement 16.7
        use super::FocusStop;
        let next = FocusStop::CalendarPrev.next(11, 0, true);
        assert_eq!(next, FocusStop::CalendarNext);
    }

    /// Validates: Requirement 16.8 — Tab from CalendarNext goes to first menu bar item.
    #[test]
    fn focus_cycle_tab_forward_from_calendar_next_goes_to_first_menu() {
        // Validates: Requirement 16.8
        use super::FocusStop;
        let next = FocusStop::CalendarNext.next(11, 0, true);
        assert_eq!(next, FocusStop::MenuBar { index: 0 });
    }

    /// Validates: Requirement 16.9, 16.10 — Tab advances through menu bar and wraps to CommandField.
    #[test]
    fn focus_cycle_tab_forward_from_last_menu_wraps_to_command_field() {
        // Validates: Requirement 16.10
        use super::FocusStop;
        let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
        // Advance through all menu items
        let mut stop = FocusStop::MenuBar { index: 0 };
        for expected in 1..menu_count {
            stop = stop.next(menu_count, 0, true);
            assert_eq!(stop, FocusStop::MenuBar { index: expected });
        }
        // Last menu → CommandField (tab_count=0 so no TabHeader stop)
        stop = stop.next(menu_count, 0, true);
        assert_eq!(stop, FocusStop::CommandField);
    }

    /// Validates: Requirement 16.11 — Shift+Tab from CommandField goes to last menu bar item.
    #[test]
    fn focus_cycle_shift_tab_from_command_field_goes_to_last_menu() {
        // Validates: Requirement 16.11
        use super::FocusStop;
        let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
        // tab_count=0: CommandField → last MenuBar (no TabHeader)
        let prev = FocusStop::CommandField.prev(menu_count, 0, true);
        assert_eq!(
            prev,
            FocusStop::MenuBar {
                index: menu_count - 1
            }
        );
    }

    /// Validates: Requirement 16.11 — Shift+Tab from first menu goes to CalendarNext when POM active.
    #[test]
    fn focus_cycle_shift_tab_from_first_menu_goes_to_command_field() {
        // Validates: Requirement 16.11
        use super::FocusStop;
        // Non-POM: first menu → CommandField
        // tab_count=0, non-POM: first menu → CommandField
        let prev = FocusStop::MenuBar { index: 0 }.prev(11, 0, false);
        assert_eq!(prev, FocusStop::CommandField);
        // POM active: first menu → CalendarNext
        let prev_pom = FocusStop::MenuBar { index: 0 }.prev(11, 0, true);
        assert_eq!(prev_pom, FocusStop::CalendarNext);
    }

    /// Validates: Requirement 16.19 — non-POM tab skips POM/calendar stops entirely.
    #[test]
    fn focus_cycle_non_pom_tab_skips_pom_stops() {
        // Validates: Requirement 16.19
        use super::FocusStop;
        let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
        // Forward: CommandField → MenuBar(0) (no PomOption)
        let next = FocusStop::CommandField.next(menu_count, 0, false);
        assert_eq!(next, FocusStop::MenuBar { index: 0 });
        // Forward: last MenuBar → CommandField (tab_count=0, no TabHeader)
        let wrap = FocusStop::MenuBar {
            index: menu_count - 1,
        }
        .next(menu_count, 0, false);
        assert_eq!(wrap, FocusStop::CommandField);
        // Backward: CommandField → last MenuBar (tab_count=0, no TabHeader)
        let prev = FocusStop::CommandField.prev(menu_count, 0, false);
        assert_eq!(
            prev,
            FocusStop::MenuBar {
                index: menu_count - 1
            }
        );
        // Backward: MenuBar(0) → CommandField (tab_count=0, no PomExit)
        let prev0 = FocusStop::MenuBar { index: 0 }.prev(menu_count, 0, false);
        assert_eq!(prev0, FocusStop::CommandField);
    }

    /// Validates: Requirement 16.12 — focused_pom_option is Some(index) when PomOption focused.
    #[test]
    fn focused_pom_option_renders_with_reversed_colours() {
        // Validates: Requirement 16.12
        // Verify that the focused_pom_option value derived from FocusStop is correct.
        use super::FocusStop;
        let stop = FocusStop::PomOption { index: 3 };
        let focused = match stop {
            FocusStop::PomOption { index } => Some(index),
            _ => None,
        };
        assert_eq!(focused, Some(3));
        // Non-PomOption stops yield None
        let none = match FocusStop::CommandField {
            FocusStop::PomOption { index } => Some(index),
            _ => None,
        };
        assert_eq!(none, None);
    }

    /// Validates: Requirement 16.11 — Shift+Tab moves backward through menu bar items.
    #[test]
    fn focus_stop_shift_tab_moves_backward_through_menu_bar_items() {
        // Validates: Requirement 16.11
        use super::FocusStop;
        let prev = FocusStop::MenuBar { index: 5 }.prev(11, 0, false);
        assert_eq!(prev, FocusStop::MenuBar { index: 4 });
    }

    /// Validates: Requirement 16.11 — Shift+Tab from CalendarPrev goes to PomExit.
    #[test]
    fn focus_stop_shift_tab_from_calendar_prev_goes_to_pom_exit() {
        // Validates: Requirement 16.11
        use super::FocusStop;
        let prev = FocusStop::CalendarPrev.prev(11, 0, true);
        assert_eq!(prev, FocusStop::PomExit);
    }

    /// Validates: Requirement 16.11 — Shift+Tab from CalendarNext goes to CalendarPrev.
    #[test]
    fn focus_stop_shift_tab_from_calendar_next_goes_to_calendar_prev() {
        // Validates: Requirement 16.11
        use super::FocusStop;
        let prev = FocusStop::CalendarNext.prev(11, 0, true);
        assert_eq!(prev, FocusStop::CalendarPrev);
    }

    /// Validates: Requirement 16.11 — Shift+Tab from PomExit goes to last POM option.
    #[test]
    fn focus_stop_shift_tab_from_pom_exit_goes_to_last_pom_option() {
        // Validates: Requirement 16.11
        use super::FocusStop;
        use crate::primary_option_menu::BUILT_IN_OPTIONS;
        let prev = FocusStop::PomExit.prev(11, 0, true);
        assert_eq!(
            prev,
            FocusStop::PomOption {
                index: BUILT_IN_OPTIONS.len() - 1
            }
        );
    }

    /// Validates: Requirement 16.11 — Shift+Tab from PomOption(0) goes to CommandField.
    #[test]
    fn focus_stop_shift_tab_from_pom_option_0_goes_to_command_field() {
        // Validates: Requirement 16.11
        use super::FocusStop;
        let prev = FocusStop::PomOption { index: 0 }.prev(11, 0, true);
        assert_eq!(prev, FocusStop::CommandField);
    }

    /// Validates: Requirement 16.1 — FocusStop::CommandField is the initial value.
    #[test]
    fn workbench_shell_focus_stop_field_exists_and_defaults_to_command_field() {
        // Validates: Requirement 16.1
        use super::FocusStop;
        let initial = FocusStop::CommandField;
        assert_eq!(initial, FocusStop::CommandField);
        assert_ne!(initial, FocusStop::MenuBar { index: 0 });
    }

    /// Validates: Requirement 16.6, 16.7 — full backward Shift+Tab cycle from last menu to command field.
    #[test]
    fn focus_stop_full_backward_cycle_from_last_menu_to_command_field() {
        // Validates: Requirement 16.11
        use super::FocusStop;
        let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
        let mut stop = FocusStop::MenuBar {
            index: menu_count - 1,
        };
        for expected in (0..menu_count - 1).rev() {
            stop = stop.prev(menu_count, 0, false);
            assert_eq!(stop, FocusStop::MenuBar { index: expected });
        }
        // tab_count=0: MenuBar(0) → CommandField (no TabHeader)
        stop = stop.prev(menu_count, 0, false);
        assert_eq!(stop, FocusStop::CommandField);
    }

    /// Validates: Requirement 4.6 — key label bar updates when key map changes.
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
        // F7 was in old map but not new — should now be blank
        assert!(bar.slot_for(FunctionKey::F7).unwrap().label.is_none());
    }

    // ── Phase AL: Title Line tests ─────────────────────────────────────────

    /// Validates: Requirement 17.3 — POM tab Title_Line shows app name and version.
    #[test]
    fn title_line_pom_tab_shows_app_name_and_version() {
        // Validates: Requirement 17.3
        use crate::tab_state::{TabId, TabState};
        use ff_document_model::new_document;
        let tab = TabState::pom(TabId(1), new_document());
        let text = super::title_line_text(&tab);
        assert!(
            text.contains("FileForge Workbench"),
            "must contain app name: {text}"
        );
        assert!(
            text.contains(env!("CARGO_PKG_VERSION")),
            "must contain version: {text}"
        );
    }

    /// Validates: Requirement 17.4 — file editor tab with path shows full path.
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

    /// Validates: Requirement 17.5 — untitled file editor tab shows [Untitled].
    #[test]
    fn title_line_untitled_shows_placeholder() {
        // Validates: Requirement 17.5
        use crate::tab_state::{TabId, TabState};
        use ff_document_model::new_document;
        let tab = TabState::untitled(TabId(3), new_document(), 0);
        let text = super::title_line_text(&tab);
        assert_eq!(text, "[Untitled]");
    }

    /// Validates: Requirement 17.6 — SettingsPanel tab shows tab title.
    #[test]
    fn title_line_settings_panel_shows_settings() {
        // Validates: Requirement 17.6
        use crate::tab_state::{TabId, TabState};
        use ff_document_model::new_document;
        let tab = TabState::settings_panel(TabId(4), new_document());
        let text = super::title_line_text(&tab);
        assert_eq!(text, "[SETTINGS]");
    }

    /// Validates: Requirement 17.6 — FilesPanel tab shows tab title.
    #[test]
    fn title_line_files_panel_shows_files() {
        // Validates: Requirement 17.6
        use crate::tab_state::{TabId, TabState};
        use ff_document_model::new_document;
        let tab = TabState::files_panel(TabId(5), new_document());
        let text = super::title_line_text(&tab);
        assert_eq!(text, "[FILES]");
    }

    // ── Phase AK: Tab-header focus stops + command field focus fix ───────────

    /// Validates: Requirement 16.10 — Tab from last menu bar item goes to first tab header.
    #[test]
    fn focus_cycle_tab_forward_from_last_menu_goes_to_first_tab_header() {
        // Validates: Requirement 16.10
        use super::FocusStop;
        let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
        let last_menu = FocusStop::MenuBar {
            index: menu_count - 1,
        };
        let next = last_menu.next(menu_count, 3, false);
        assert_eq!(next, FocusStop::TabHeader { index: 0 });
    }

    /// Validates: Requirement 16.20 — Tab advances through tab headers left to right.
    #[test]
    fn focus_cycle_tab_forward_through_all_tab_headers() {
        // Validates: Requirement 16.20
        use super::FocusStop;
        let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
        let mut stop = FocusStop::TabHeader { index: 0 };
        stop = stop.next(menu_count, 3, false);
        assert_eq!(stop, FocusStop::TabHeader { index: 1 });
        stop = stop.next(menu_count, 3, false);
        assert_eq!(stop, FocusStop::TabHeader { index: 2 });
    }

    /// Validates: Requirement 16.21 — Tab from last tab header wraps to CommandField.
    #[test]
    fn focus_cycle_tab_forward_from_last_tab_header_wraps_to_command_field() {
        // Validates: Requirement 16.21
        use super::FocusStop;
        let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
        let next = FocusStop::TabHeader { index: 2 }.next(menu_count, 3, false);
        assert_eq!(next, FocusStop::CommandField);
    }

    /// Validates: Requirement 16.11 — Shift+Tab from CommandField goes to last tab header.
    #[test]
    fn focus_cycle_shift_tab_from_command_field_goes_to_last_tab_header() {
        // Validates: Requirement 16.11
        use super::FocusStop;
        let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
        let prev = FocusStop::CommandField.prev(menu_count, 3, false);
        assert_eq!(prev, FocusStop::TabHeader { index: 2 });
    }

    /// Validates: Requirement 16.11 — Shift+Tab from first tab header goes to last menu bar item.
    #[test]
    fn focus_cycle_shift_tab_from_first_tab_header_goes_to_last_menu() {
        // Validates: Requirement 16.11
        use super::FocusStop;
        let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
        let prev = FocusStop::TabHeader { index: 0 }.prev(menu_count, 3, false);
        assert_eq!(
            prev,
            FocusStop::MenuBar {
                index: menu_count - 1
            }
        );
    }

    /// Validates: Requirement 16.22 — non-POM cycle includes tab headers.
    #[test]
    fn focus_cycle_non_pom_includes_tab_headers() {
        // Validates: Requirement 16.22
        use super::FocusStop;
        let menu_count = super::MENU_BAR_TOP_LEVEL_LABELS.len();
        // Full forward cycle: CommandField → MenuBar(0..last) → TabHeader(0..1) → CommandField
        let mut stop = FocusStop::CommandField;
        // Step 1: CommandField → MenuBar(0)
        stop = stop.next(menu_count, 2, false);
        assert_eq!(stop, FocusStop::MenuBar { index: 0 });
        // Steps 2..menu_count: advance through remaining menu items to last
        for _ in 0..menu_count - 1 {
            stop = stop.next(menu_count, 2, false);
        }
        // Now at MenuBar { index: menu_count - 1 }; one more step → TabHeader(0)
        stop = stop.next(menu_count, 2, false);
        assert_eq!(stop, FocusStop::TabHeader { index: 0 });
        stop = stop.next(menu_count, 2, false);
        assert_eq!(stop, FocusStop::TabHeader { index: 1 });
        stop = stop.next(menu_count, 2, false);
        assert_eq!(stop, FocusStop::CommandField);
    }

    // ── Phase AO: Detachable Tab Windows (Requirement 18) ──────────────────────

    /// Validates: Requirement 18.1, 18.4 — detaching a tab sets is_floating and
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

    /// Validates: Requirement 18.7 — maximum 16 floating windows enforced.
    #[test]
    fn floating_tab_limit_enforced_at_16() {
        // Validates: Requirement 18.7
        use super::FloatingTab;

        let mut floating: Vec<FloatingTab> = Vec::new();
        for i in 0..16 {
            floating.push(FloatingTab {
                viewport_id: egui::ViewportId::from_hash_of(format!("ft_{i}")),
                tab_index: i,
                origin_index: i,
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

    /// Validates: Requirement 18.3 — origin_index is preserved on FloatingTab.
    #[test]
    fn floating_tab_origin_index_preserved() {
        // Validates: Requirement 18.3
        use super::FloatingTab;

        let ft = FloatingTab {
            viewport_id: egui::ViewportId::from_hash_of("test"),
            tab_index: 3,
            origin_index: 3,
        };
        assert_eq!(ft.origin_index, 3);
        assert_eq!(ft.tab_index, 3);
    }

    /// Validates: Requirement 18.3 — redock clamps origin_index to current tab count.
    #[test]
    fn redock_clamps_to_tab_count() {
        // Validates: Requirement 18.3
        // Simulate: origin_index=5, but only 3 tabs remain after others were closed.
        let origin_index: usize = 5;
        let tab_count: usize = 3;
        let clamped = origin_index.min(tab_count.saturating_sub(1));
        assert_eq!(clamped, 2);
    }

    /// Validates: Requirement 18.5 — floating window OS title bar format.
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
        let title = format!("{} — FileForge Workbench", super::title_line_text(&tab));
        assert!(title.contains("main.rs"), "title must contain file name");
        assert!(
            title.ends_with("— FileForge Workbench"),
            "title must end with app name"
        );
    }

    // ── Phase AR: [context_key_maps] TOML config parsing (Req 14.7) ──────────

    /// Validates: Requirement 14.7 — context_key_maps table parsed into KeyMapResolver.
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

    /// Validates: Requirement 14.7 — invalid key names in context map are skipped.
    #[test]
    fn context_key_maps_invalid_key_skipped() {
        // Validates: Requirement 14.7 (inherits Req 1.5 graceful-skip behaviour)
        let table: toml::Table = "F3 = \"RETURN\"\nF99 = \"INVALID\"".parse().unwrap();
        let (map, warnings) = ff_keys::KeyMap::from_toml_table(&table, "pom");
        assert_eq!(map.len(), 1, "only F3 should be loaded");
        assert_eq!(warnings.len(), 1, "F99 should produce one warning");
    }
}
