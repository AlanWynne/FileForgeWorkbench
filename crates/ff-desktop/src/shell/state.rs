//! # WorkbenchShell State
//!
//! The `WorkbenchShell` struct definition -- the egui/eframe application shell's
//! fields. Moved out of `mod.rs` verbatim as part of the Phase 2 task 2.2
//! file-size split (per the ff-desktop layout in `rust-standards.md`, which puts
//! the struct's fields in `shell/state.rs`). Field types, doc comments, and
//! visibility are unchanged except where a sibling module's direct access
//! requires the minimum necessary widening (private -> `pub(crate)`), which is
//! crate-internal and behaviour-preserving. The struct is re-exported from
//! `mod.rs` so `super::WorkbenchShell` / `crate::shell::WorkbenchShell` resolve
//! unchanged.

use std::sync::{Arc, Mutex};

use eframe::egui;
use ff_command::{CommandDispatch, CommandLineHistory, CommandRegistry};
use ff_command_semantics::CommandEngine;
use ff_config::ConfigHandle;
use ff_core::WorkbenchApp;
use ff_keys::{HistoryStore, KeyLabelBarModel, KeyMapResolver};
use ff_session::WorkspaceState;
use ff_theme::ThemePalette;
use ff_zoom::ZoomState;
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

use super::{command_line_outcome, reset_bare, state_groups, KeyBarScope};

/// The deferred-open channel payload: `(path, owning_env)` where `owning_env` is
/// the optional Owning_Environment NAME captured at open (CR-CH-053 Task 19,
/// Req 15.2); `None` -> host FS default (Req 15.3). A `type` alias to keep the
/// nested `Arc<Mutex<Option<..>>>` readable (and satisfy clippy::type_complexity).
pub(super) type PendingOpen = Arc<Mutex<Option<(String, Option<String>)>>>;

/// The egui/eframe application shell.
pub struct WorkbenchShell {
    pub(super) app: WorkbenchApp,
    pub(super) runtime: Runtime,
    pub(super) palette: ThemePalette,
    pub(super) tabs: TabManager,
    /// ISPF-style command field text.
    pub(super) command_text: String,
    /// Per-dispatch Command_Line_Outcome stash: a command arm MAY set this to
    /// override the default field disposition (e.g. RETRIEVE -> Set/Leave). It is
    /// consumed and reset once per command-line run (CR-CH-033, Req 13.4).
    pub(super) pending_command_line_outcome: Option<command_line_outcome::CommandLineOutcome>,
    /// One-shot startup flag.
    pub(super) started: bool,
    /// Files to open on the first frame (from CLI arguments).
    pub(super) cli_files: Vec<String>,
    /// When Some, open `(path, owning_env)` at the start of the next frame. The
    /// optional `owning_env` is the Owning_Environment NAME captured from the
    /// originating catalog/provider (CR-CH-053 Task 19, Req 15.2); `None` ->
    /// defaults to the host FS environment on open (Req 15.3).
    pub(super) pending_open: PendingOpen,
    /// Set to true by the file.exit handler; checked in update().
    pub(super) should_close: Arc<std::sync::Mutex<bool>>,
    /// Error message to display in the status bar (cleared on next open).
    pub(super) open_error: Option<String>,
    /// Command dispatch -- routes file.open / file.exit through the registry.
    pub(super) dispatch: CommandDispatch,
    /// The per-invocation Cursor_Context snapshot (CR-CH-028, command-framework
    /// Requirement 12). The shell refreshes this from live focus/selection at
    /// each dispatch; the registered `ShellContextProvider` holds an `Arc` clone
    /// and returns it so commands dispatched through the registry receive a
    /// populated package. A shared cell is required because `ContextProvider`
    /// takes `&self`.
    pub(super) cursor_context_snapshot: Arc<Mutex<ff_command::CursorContext>>,
    /// The focused Menu_Option recorded during the last menu render (its egui id,
    /// command, and label), so a focused id resolves to its semantic identity
    /// when building the Cursor_Context (Requirement 12.2). `None` when no menu
    /// option is focused.
    pub(super) focused_menu_option: Option<(egui::Id, String, String)>,
    /// Shared command registry -- used by the Command Palette to enumerate commands.
    ///
    /// Validates: command-palette Requirement 2.1
    pub(super) cmd_registry: Arc<CommandRegistry>,
    /// ISPF command semantics engine -- parses and executes primary commands.
    pub(super) cmd_engine: CommandEngine,
    /// Command-line history + RETRIEVE pointer, owned by the command-processor
    /// layer (CR-NR-084, Option B). The shell forwards every submitted command
    /// line to it (`record`) and drives recall through it (`retrieve`).
    pub(super) command_line_history: CommandLineHistory,
    /// The In_Progress_Line captured at the start of an arrow-history cycle
    /// (CR-NR-096, function-keys-and-history Requirement 23). The FIRST Up of a
    /// cycle stores the current field text here (before recalling the newest
    /// entry); when Down steps NEWER past the newest entry the workbench restores
    /// this text and clears the store. `None` when no History_Cycle is active.
    /// Shared like `command_line_history` because a single shared Retrieve_Pointer
    /// means at most one cycle is active at a time across all command fields
    /// (Req 23.9).
    pub(super) command_line_in_progress: Option<String>,
    /// Persistence store for the command-line history (function-keys-and-history
    /// Requirement 6). Resolved once at startup to
    /// `<User_Data_Dir>/command_history.toml` (or the `FFWB_HISTORY_PATH`
    /// override for test isolation); `None` when the path cannot be resolved.
    /// Loaded in `new`, saved in `on_exit`.
    pub(super) history_store: Option<HistoryStore>,
    /// Find/replace engine -- FIND, RFIND, CHANGE, RCHANGE.
    pub(super) find_manager: FindManager,
    /// Navigation engine -- LOCATE, SORT, UP, DOWN, LEFT, RIGHT, TOP, BOTTOM.
    pub(super) nav_manager: NavManager,
    /// Exclude/Show/Reset engine -- EXCLUDE, SHOW, RESET.
    pub(super) exclude_manager: ExcludeManager,
    /// Active key map resolver -- global key map (no profile active at startup).
    pub(super) key_map_resolver: KeyMapResolver,
    /// Key label bar display model -- derived from the active key map.
    pub(super) key_label_bar: KeyLabelBarModel,
    /// Whether the Key Label Bar is currently visible.
    ///
    /// Validates: Requirement 12.4
    pub(super) key_bar_visible: bool,
    /// Which modifier layer the Key Label Bar shows when visible (CR-CH-046).
    /// Retained across hide/show so `PFSHOW ON` restores the last scope.
    ///
    /// Validates: Requirement 12.4, 12.8-12.12
    pub(super) key_bar_scope: KeyBarScope,
    /// History of previously active tab indices for END navigation.
    ///
    /// Validates: Requirement 17.1
    pub(super) tab_history: Vec<usize>,
    /// When Some, show the history-list overlay with these entries.
    ///
    /// Validates: Requirement 19.3
    pub(super) show_history_list: Option<Vec<String>>,
    /// When Some, show the SWAP tab-picker overlay (multi-tab-editor Req 18.3).
    /// The unit payload keeps the state a simple open/closed flag; the picker
    /// reads the live tab list at render time.
    pub(super) show_swap_list: Option<()>,
    /// Session persistence -- None when User Data Dir is unavailable.
    pub(super) session: Option<SessionManager>,
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
    /// Configuration handle -- used to read catalog default paths.
    pub(super) config_handle: ConfigHandle,
    /// User-defined command definitions (`commands/commands.toml`).
    ///
    /// Feeds Target_Resolution so a menu option or keyboard shortcut whose
    /// command value equals a definition id runs that definition's target.
    ///
    /// Validates: command-configurator Requirement 1, 4.3; menu-workspace
    /// Requirement 10.3
    pub(crate) command_store: crate::command_config::store::CommandStore,
    /// Configurable Workspace Kind registry (CR-NR-090 B.1): the single source of
    /// truth for each Kind's effective title (and, later, menu bar / key list /
    /// profile). Loaded once at startup from `<User_Data_Dir>/workspace-kinds/`;
    /// compiled built-in defaults guarantee it is never empty.
    ///
    /// Validates: workspace-kinds Requirement 2, 3
    pub(crate) kind_registry: crate::workspace_kind::KindRegistry,
    /// Built Environment_Registry (CR-CH-053 Task 17): the shell-owned collection
    /// of the command environments that exist in the running shell (FFCMD base,
    /// FFEDIT, host-FS placeholder), registered in code at startup. The
    /// active-environment derivation and the active-wins claim gate read FROM this
    /// registry by the focused kind's command-environment name, replacing the
    /// former closed `EnvironmentKind` enum / `environment_for_kind` match /
    /// `== FfEdit` literal. It is a pure data/lookup structure feeding the one
    /// shared ladder path -- NOT a second dispatcher or navigation stack.
    ///
    /// Validates: command-environments Requirement 13.1, 13.2, 13.7
    pub(super) environments: super::environment_registry::EnvironmentRegistry,
    /// Live `ff-vfs` Provider_Registry (CR-CH-053 Task 22): a shell-owned
    /// `ProviderRegistry` registered LIVE at startup, seeded with the host-FS
    /// `local` provider so a provider is resolvable BY SCHEME at runtime (Req
    /// 17.1). This is the seam a plugin-provided `VfsProvider` (e.g. the mainframe
    /// VFS provider in the `V` stream) registers into -- without it such a
    /// provider has nowhere to land (Req 17.2). Registering it is ADDITIVE: the
    /// host-path open/save path reads through `LocalFsProvider` /
    /// `BackendEnvironment` directly and does NOT consult this registry, so native
    /// file access is unchanged whether or not a non-host provider is registered
    /// (Req 17.3, 17.4). `Arc` so a background provider producer could share it.
    ///
    /// Validates: command-environments Requirement 17.1, 17.2, 17.3, 17.4
    //
    // `allow(dead_code)`: this field is the LIVE registration SEAM required by
    // Req 17.1/17.2 -- its job is to EXIST at runtime so a plugin-provided
    // `VfsProvider` (the `V`-stream mainframe provider) has somewhere to land.
    // Its runtime CONSUMER (resolving a non-host provider by scheme during a
    // non-host open) is a later phase (Req 17.4), so no non-test code reads it
    // yet. The seam and its "additive, host path unchanged" contract are proven
    // by the Task 22 tests. Remove the allow when the first non-host consumer
    // lands.
    #[allow(dead_code)]
    pub(super) provider_registry: Arc<ff_vfs::ProviderRegistry>,
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
    /// Kinds Editor Context state (workspace-kinds Req 6, CR-NR-090 B.4).
    pub(crate) kinds_editor_panel: crate::kinds_editor_panel::KindsEditorState,
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
    pub(super) files_panel: FilesPanelState,
    /// Persisted width of the File Explorer side panel (logical pixels).
    /// Session-persisted via `file_explorer_sidebar_width` (ff-session).
    ///
    /// Validates: Requirement 1.3 file-tree-panel (fix B019), 23.9
    pub(super) file_explorer_panel_width: f32,
    /// Canonical ff-file-tree-backed navigation model for the File Explorer
    /// (CR-NR-060 Slice A). Identity is NodeId + ResourceUri (Req 24.1, 24.2).
    pub(super) nav_model: crate::nav_model::NavModel,
    /// Selection/cursor state for the NavModel-backed explorer (Requirement 24.2).
    pub(super) nav_selection: crate::explorer_view::ExplorerSelection,

    /// Active rename dialog for the modern explorer: (target node, edit buffer).
    /// `None` when no rename is in progress. (CR-NR-060 Slice A, Req 16 Rename.)
    pub(super) nav_rename: Option<(ff_file_tree::NodeId, String)>,

    /// Active delete-confirmation dialog for the modern explorer: (target node,
    /// display label). `None` when no delete is pending. (Req 16 Delete.)
    pub(super) nav_delete: Option<(ff_file_tree::NodeId, String)>,

    /// Active new-child dialog for the modern explorer: (parent directory node,
    /// is_directory, name buffer). `None` when none pending. (Req 16 New.)
    pub(super) nav_new: Option<(ff_file_tree::NodeId, bool, String)>,

    /// When true, the modern explorer node list holds keyboard focus (Tab moved
    /// focus from the shell Command ===> into the tree). (Req 24.9 / 20.1.)
    pub(super) nav_focused: bool,

    /// File clipboard for the modern explorer: source resource URIs marked for a
    /// copy, pasted into a target directory on Paste. (Req 21.1 / 21.3.)
    pub(super) nav_file_clipboard: Vec<ff_vfs::ResourceUri>,
    /// Toolchain panel state -- GCC and Rust plugin entries.
    pub(super) toolchain_panel: ToolchainPanelState,
    /// Whether the Toolchain Panel is currently visible.
    pub(super) show_toolchain_panel: bool,
    /// Deferred: open a new POM tab on the next frame (set by tab-bar context menu).
    pub(super) pending_new_pom: bool,
    /// Deferred: open a new untitled tab on the next frame (set by tab-bar context menu).
    pub(super) pending_new_file: bool,
    /// Deferred: return the active FilesPanel tab to POM view (set by F3/END in Files Panel).
    pub(super) pending_return_to_pom: bool,
    /// Deferred: option to execute from a Menu_Workspace option click (set by
    /// render, processed in update). Carries the full option so an inline
    /// `[options.target]` is honoured on click as well as by typing.
    ///
    /// Validates: menu-workspace Requirement 3.2, 10.6
    pub(crate) pending_menu_option: Option<crate::menu_workspace::MenuOption>,
    /// Global application zoom -- single level shared across all tabs and panels.
    ///
    /// Addresses: Requirement 3.1 (view-zoom) -- zoom carries forward across context switches.
    pub(super) zoom: ZoomState,
    /// Month offset for the POM calendar (0 = current month, -1 = prev, +1 = next).
    ///
    /// Validates: Requirement 14.42
    pub(super) pom_calendar_offset: i32,

    /// The active theme's backing file path and its last-seen modification time,
    /// used to hot-reload the palette when the file changes on disk (CR-NR-074).
    /// `None` when the active theme is not file-backed (e.g. a compiled built-in
    /// with no on-disk file yet).
    ///
    /// Validates: theme-and-appearance Requirement 19.6
    pub(super) active_theme_file: Option<(std::path::PathBuf, std::time::SystemTime)>,

    /// Test-only overrides for the per-feature on-disk directories (themes,
    /// menus, keymaps, workspace-kinds, screen-collections). Production leaves
    /// every field `None` and the resolver methods fall back to the real
    /// `<User_Data_Dir>` locations; tests point them at a TempDir.
    ///
    /// Grouped sub-struct (Phase 2 F4/S3); see [`state_groups::DirOverrides`].
    /// NOT cfg-gated because the resolver methods that read it are not cfg(test).
    pub(super) dir_overrides: state_groups::DirOverrides,
    /// Last pixels_per_point applied by zoom -- avoids overwriting OS DPI every frame.
    pub(super) last_ppp: f32,
    /// True while the user is holding the mouse button down (window drag in progress).
    /// DPI-driven resize is suppressed during a drag to prevent mid-move stuttering.
    pub(super) is_dragging: bool,
    /// Pending pixels_per_point to apply once the drag is released.
    pub(super) pending_ppp: Option<f32>,
    /// Whether the Help > About dialog is currently open.
    ///
    /// Validates: Requirement 13.1
    pub(super) show_about: bool,
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
    /// Text buffer for the SCROLL ===> field input.
    ///
    /// Validates: Requirement 19.1
    pub(crate) scroll_field_text: String,
    /// True when any modal dialog is open -- suppresses the shell Tab-cycle and
    /// command-field focus steal so keystrokes reach the dialog's own widgets.
    pub(super) modal_open: bool,
    /// The resolved RESET BARE target when the confirmation dialog is open, else
    /// `None` (CR-CH-021 / CR-NR-083, configuration-system Req 19.2, 19.9-19.15).
    /// Carries the profile list the dialog names and the execute path resets. No
    /// configuration is archived or reset until the user confirms.
    pub(super) reset_bare_confirm: Option<reset_bare::ResetBareTarget>,
    /// One-shot: true for the first frame the RESET BARE dialog is open, so it
    /// can request initial keyboard focus on the Cancel button (B077,
    /// accessibility Req 2.3 -- modal focus trap; Cancel is the safe default).
    pub(super) reset_bare_focus_requested: bool,
    /// Config Panel state (the flat config-key browser opened by `CONFIG`).
    ///
    /// Validates: Requirement 15.1, 15.2
    pub(super) config_panel: ConfigPanelState,
    /// The single shell-owned Help Topic Registry, loaded ONCE at startup from
    /// the shipped `help/` directory (plus command-metadata topics). Reused by
    /// every F1 press / HELP invocation -- the shell never news an empty registry
    /// per call.
    ///
    /// Validates: context-help Requirement 18.1 (CR-NR-097)
    pub(super) help_registry: std::sync::Arc<ff_help::HelpTopicRegistry>,
    /// The Help Context panel (renders the resolved topic; a `WorkspaceContext`).
    ///
    /// Validates: context-help Requirement 18.2, 18.5 (CR-NR-097)
    pub(super) help_context_panel: crate::help_context::HelpContextPanel,
    /// Session-scoped, in-memory tally of help topics that were requested but not
    /// found (distinct Topic_Key -> request count). Not persisted; never written
    /// to any project document.
    ///
    /// Validates: context-help Requirement 19.2, 19.6 (CR-NR-097)
    pub(super) help_missing_tally: std::collections::HashMap<String, u32>,
    /// Plugin Manager panel state.
    ///
    /// Validates: plugin-manager-ui Requirement 1.1
    pub(super) plugin_manager_panel: PluginManagerPanelState,
    /// Macro Library panel state.
    ///
    /// Validates: lua-macro-engine Requirement 12.1
    pub(super) macro_library_panel: crate::macro_library_panel::MacroLibraryPanelState,
    /// Event Log panel state.
    ///
    /// Validates: notification-system Requirement 2.2
    pub(super) event_log_panel: EventLogPanelState,

    /// Screen Collection and Replay Manager session: the active screen
    /// Collection and automatic-capture mode (CR-NR-098, Wave 2).
    ///
    /// Validates: screen-snapshot-scrm Requirement 7, 8, 9.
    pub(super) scrm: crate::scrm_session::ScrmSession,

    /// The SCRM Replay viewer Context state (CR-NR-098, Wave 2).
    ///
    /// Validates: screen-snapshot-scrm Requirement 16.
    pub(super) scrm_viewer: crate::scrm_viewer_panel::ScrmViewerState,
    /// Notification channel receiver -- drained each frame.
    ///
    /// Validates: notification-system Requirement 1.1
    pub(super) notification_rx: std::sync::mpsc::Receiver<Notification>,
    /// Notification channel sender -- cloned for background tasks.
    ///
    /// Validates: notification-system Requirement 3.1
    #[allow(dead_code)]
    pub(super) notification_tx: std::sync::mpsc::SyncSender<Notification>,
    /// Shared notification queue -- drained each frame from the channel.
    ///
    /// Validates: notification-system Requirement 1.1
    pub(crate) notification_queue: std::sync::Arc<std::sync::Mutex<NotificationQueue>>,
    /// Unified Tab-order focus anchors and one-shot focus latches (CR-CH-023).
    ///
    /// Grouped sub-struct (Phase 2 F4/S3). Fields keep their original types and
    /// semantics; see [`state_groups::FocusState`].
    ///
    /// Validates: Requirement 16.1-16.14 (CR-CH-023)
    pub(crate) focus: state_groups::FocusState,
    /// Session start timestamp -- recorded when the shell is created.
    ///
    /// Validates: Requirement 20.1, 20.2
    pub(crate) session_start: chrono::DateTime<chrono::Local>,
    /// Automation registry -- exposes control state to the FFTest runner.
    ///
    /// Validates: Requirement 2.1, 2.5 (automated-dialog-testing)
    pub(crate) automation: ShellAutomationRegistry,
    /// Detached-window (floating tab) and split-region bookkeeping.
    ///
    /// Grouped sub-struct (Phase 2 F4/S3). Fields keep their original types and
    /// semantics; see [`state_groups::DetachSplitState`].
    ///
    /// Validates: Requirement 18.1, 18.2; layout-and-docking Requirement 14.6,
    /// 14.9, 15.3, 15.5, 16.8; menu-and-statusbar Req 16.15
    pub(crate) detach_split: state_groups::DetachSplitState,
}
