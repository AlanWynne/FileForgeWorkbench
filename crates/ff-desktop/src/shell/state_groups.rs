//! Cohesive sub-structs that group related `WorkbenchShell` fields.
//!
//! These are plain data containers introduced by the Phase 2 shell
//! simplification (review finding F4 / S3) to tame the `WorkbenchShell`
//! god-struct. They carry NO behaviour of their own: every field keeps the same
//! type and semantics it had as a flat `WorkbenchShell` field; only the access
//! path changes (e.g. `self.focus.first_interior_id`). No runtime behaviour,
//! rendering, focus order, or command behaviour changes as a result.

use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui;

use super::{FloatingTab, WorkspaceCommandContext};

// === DirOverrides ====================================================
// Test-only directory overrides. Production leaves every field `None` and the
// `pub(super)` resolver methods (`themes_dir()`, `menus_dir()`, `keymaps_dir()`,
// `workspace_kinds_dir()`, `scrm_dir()`) fall back to the real `<User_Data_Dir>`
// locations. These are READ by non-test code (the resolvers are not cfg(test)),
// so this struct is deliberately NOT gated behind `#[cfg(test)]`.

/// Test-only overrides for the per-feature on-disk directories. Each is `None`
/// in production; tests point them at a `TempDir` for deterministic, isolated
/// file operations.
#[derive(Debug, Default)]
pub(crate) struct DirOverrides {
    /// Override for the themes directory (CR-NR-074).
    pub themes: Option<PathBuf>,
    /// Override for the menus directory (menu-workspace Req 13, CR-NR-075).
    pub menus: Option<PathBuf>,
    /// Override for the keymaps directory (function-keys Req 22, CR-CH-029).
    pub keymaps: Option<PathBuf>,
    /// Override for the workspace-kinds directory (CR-NR-090 B.4).
    pub workspace_kinds: Option<PathBuf>,
    /// Override for the screen-collections directory (CR-NR-098 Wave 3).
    pub scrm: Option<PathBuf>,
    /// Override for the User_Locale_Override_Store BASE directory (CR-CH-072).
    /// Points at an isolated `aliases/` base in tests so the Alias Editor never
    /// touches the developer's real override store; the per-locale, per-ENV
    /// override file is `<base>/<locale>/<ENV>.toml`.
    pub user_aliases: Option<PathBuf>,
}

// === DetachSplitState ================================================
// Detached-window (floating tab) and split-region bookkeeping.

/// Detached-window and split-region bookkeeping for the shell.
///
/// Validates: menu-and-statusbar Requirement 18.1, 18.2; layout-and-docking
/// Requirement 14.6, 15.3, 16.8 (fields retain their original semantics).
#[derive(Default)]
pub(crate) struct DetachSplitState {
    /// All currently floating (detached) tabs.
    pub floating_tabs: Vec<FloatingTab>,
    /// Index of the tab to detach on the next frame.
    pub detach_pending: Option<usize>,
    /// In-progress drag of a tab header between split regions (CR-NR-093).
    pub split_tab_drag: Option<(crate::tab_state::TabId, ff_layout::TabGroupId)>,
    /// Per-frame accumulator of each rendered split leaf's screen rect.
    pub split_leaf_rects: Vec<(ff_layout::TabGroupId, egui::Rect)>,
    /// Per-region command-line contexts, keyed by split leaf id (CR-NR-094).
    pub region_cmd_ctx: HashMap<ff_layout::TabGroupId, WorkspaceCommandContext>,
    /// The focused split region's menu-bar first-button id (B073).
    pub focused_region_menu_first: Option<egui::Id>,
}

// core-001 strip: NavUiState (the modern-explorer selection / rename / delete /
// new-child dialogs / file clipboard, backed by the parked ff-explorer-view and
// ff-file-tree crates) is removed with the file navigator domain.

// === HelpState =======================================================
// The shell-owned Help Topic Registry, the Help Context panel, and the
// session-scoped missing-topic tally (CR-NR-097). Grouped from the former flat
// `help_*` fields (behaviour-preserving: only the access path changes).

/// Help subsystem state grouped from the former flat `help_*` `WorkbenchShell`
/// fields. A plain data container: every field keeps the same type and
/// semantics it had as a flat field; only the access path changes.
pub(crate) struct HelpState {
    /// The single shell-owned Help Topic Registry, loaded ONCE at startup from
    /// the shipped `help/` directory (plus command-metadata topics). Reused by
    /// every F1 press / HELP invocation -- the shell never news an empty registry
    /// per call.
    ///
    /// Validates: context-help Requirement 18.1 (CR-NR-097)
    pub registry: std::sync::Arc<ff_help::HelpTopicRegistry>,
    /// The Help Context panel (renders the resolved topic; a `WorkspaceContext`).
    ///
    /// Validates: context-help Requirement 18.2, 18.5 (CR-NR-097)
    pub context_panel: crate::help_context::HelpContextPanel,
    /// Session-scoped, in-memory tally of help topics that were requested but not
    /// found (distinct Topic_Key -> request count). Not persisted; never written
    /// to any project document.
    ///
    /// Validates: context-help Requirement 19.2, 19.6 (CR-NR-097)
    pub missing_tally: std::collections::HashMap<String, u32>,
}

// === PendingTabActions ===============================================
// Deferred tab-bar context-menu actions, set in one frame and applied on the
// next. Grouped from the former flat `pending_new_pom` / `pending_new_file` /
// `pending_return_to_pom` fields (behaviour-preserving: only the access path
// changes).

/// Deferred tab-bar/context actions grouped from the former flat `pending_*`
/// `WorkbenchShell` bool fields. A plain data container: every field keeps the
/// same type and semantics it had as a flat field; only the access path changes.
#[derive(Debug, Default)]
pub(crate) struct PendingTabActions {
    /// Deferred: open a new POM tab on the next frame (set by tab-bar context menu).
    pub new_pom: bool,
    /// Deferred: open a new untitled tab on the next frame (set by tab-bar context menu).
    pub new_file: bool,
    /// Deferred: return the active FilesPanel tab to POM view (set by F3/END in Files Panel).
    pub return_to_pom: bool,
}

// === PendingSnapshot ================================================
// The two-phase async bare-SNAPSHOT state (CR-NR-114, FEAT-002). Bare SNAPSHOT
// is a REQUEST: the command phase resolves the active Context's screen, renders
// its text flavors, stashes them here, and asks the egui frame loop for a live
// window screenshot (which is ASYNC in egui 0.33). The frame loop issues the
// ViewportCommand::Screenshot, and when the Event::Screenshot arrives a later
// frame it assembles the composite picture+text+CF_HTML payload and places it
// on the clipboard in one transaction. Grouped (not two loose god-struct
// fields) per wiring-standard.md.

/// The deferred bare-SNAPSHOT request (CR-NR-114, FEAT-002). A plain data
/// container holding the pre-rendered text flavors captured in the command
/// phase, waiting for the async window screenshot the frame loop requests.
///
/// Both strings are rendered from the SAME logical screen model in the command
/// phase (where the `ScreenProvider` seam lives), so the frame loop stays
/// egui-only and never touches the model: `text` is the Markdown-fenced
/// rendering published as the `CF_UNICODETEXT` flavor; `pre_text` is the plain
/// Unicode box-drawn body placed inside the CF_HTML `<pre>` block.
///
/// Validates: screen-snapshot-scrm Requirement 4.1, 23.4, 23.5c, 23.5d.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PendingSnapshot {
    /// The Markdown-fenced text flavor (`CF_UNICODETEXT`).
    pub text: String,
    /// The plain Unicode box-drawn screen body (the CF_HTML `<pre>` body).
    pub pre_text: String,
}

/// The shell's two-phase bare-SNAPSHOT state (CR-NR-114, FEAT-002). A grouped
/// sub-struct (not loose `WorkbenchShell` fields) per wiring-standard.md.
///
/// Validates: screen-snapshot-scrm Requirement 4.1, 23.4.
#[derive(Debug, Clone, Default)]
pub(crate) struct SnapshotState {
    /// When `Some`, a bare SNAPSHOT is waiting for the async window screenshot;
    /// the frame loop assembles and places the composite payload on the
    /// `Event::Screenshot` frame, then clears this.
    pub pending: Option<PendingSnapshot>,
    /// One-shot: set by the command phase to ask the frame loop to issue
    /// `ViewportCommand::Screenshot` next frame. Cleared once the request is
    /// issued (the `pending` payload stays in place until the image arrives).
    pub screenshot_requested: bool,
}

// === ScrmUiState =====================================================
// SCRM shell-side UI/bookkeeping state that is NOT part of the GUI-free
// ff-scrm engine (which the `scrm: ScrmSession` field owns). Grouped per
// wiring-standard.md rather than added as a loose WorkbenchShell field.

/// SCRM shell-side bookkeeping (CR-NR-098 Wave 2R). A plain data container.
///
/// `last_dispatched` records the most recently dispatched command at the SINGLE
/// command-dispatch seam (`shell/dispatch.rs`) as `(canonical command string,
/// resolved CommandTarget kind label)`. The Context-transition auto-capture hook
/// reads-and-clears it so a command-caused transition records its triggering
/// command (Req 9.10/9.11), while a timed-interval / rule capture finds it
/// already `None` and records an absent triggering command (Req 9.12). The
/// `CommandTarget` is mapped to a label `String` in the shell so `ff-scrm` keeps
/// no `ff-command` dependency.
///
/// Validates: screen-snapshot-scrm Requirement 9.10, 9.11, 9.12.
#[derive(Debug, Default)]
pub(crate) struct ScrmUiState {
    /// `(canonical command string, resolved CommandTarget kind label)` of the
    /// last command dispatched at the single seam, consumed by the next
    /// Context-transition auto-capture. `None` when the last transition cause
    /// was not a command (interval/rule) or has already been consumed.
    pub last_dispatched: Option<(String, String)>,
    /// The capture-STOP Save-As modal state (Stream SCRM Phase 2C FEAT-002).
    /// `Some` means the modal is open: `CAPTURE STOP` with a non-empty active
    /// Collection sets it; SAVE/CANCEL clears it. On SAVE the shell dispatches
    /// the EXISTING `CAPTURE SAVE` / `CAPTURE EXPORT` verbs through the single
    /// front door (no new persistence path); on CANCEL nothing is written.
    ///
    /// Validates: screen-snapshot-scrm Requirement 24.1, 24.4, 24.5.
    pub save_as: Option<crate::scrm_save_as_dialog::SaveAsState>,
    /// The saved-session picker modal state (Stream SCRM Phase 2C FEAT-003).
    /// `Some` means the modal is open: bare `CAPTURE REPLAY` sets it; selecting
    /// a session dispatches `CAPTURE REPLAY <path>` through the single front door
    /// and clears it; Cancel/Escape clears it without dispatching. The shell
    /// enumerates the `*.ffscrm` files in `scrm_dir()` fresh each frame.
    ///
    /// Validates: screen-snapshot-scrm Requirement 10.12.
    pub replay_picker: Option<crate::scrm_session_picker::ReplayPickerState>,
}

// === I18nState =======================================================
// Localization runtime state owned by the shell (CR-NR-109). Currently holds the
// locale-overlaid per-environment alias store, shared with the process-global
// reload slot as an `Arc<Mutex<..>>` so a `ui.locale` change re-seeds+overlays it
// in place and the shell (holding the SAME Arc) observes the swap -- mirroring the
// help-registry reload seam (help.rs), adding NO new reload mechanism.

/// Shell-owned localization runtime state (CR-NR-109). A grouped sub-struct, not
/// a flat `WorkbenchShell` field (wiring-standard.md).
///
/// Validates: localization Requirement 11.2, 11.3
#[derive(Debug, Default)]
pub(crate) struct I18nState {
    /// The locale-overlaid per-environment alias store (FFCMD + FFEDIT), seeded
    /// at construction and re-populated on a `ui.locale` reload. Shared as an
    /// `Arc<Mutex<..>>` with the process-global reload slot so the reload closure
    /// updates the SAME store the shell reads from.
    ///
    /// Validates: localization Requirement 11.2, 11.3, 11.5
    //
    // Read by the CR-NR-109 FEAT-002 locale-aware resolver
    // (`WorkbenchShell::resolve_locale_verb` / `menu_option_command_surfaces` in
    // shell/locale_resolver.rs); seeded at construction and kept in sync via the
    // process-global reload slot.
    pub alias_store: std::sync::Arc<std::sync::Mutex<super::alias_reader::LocaleAliasStore>>,
}

// === FocusState ======================================================
// Interior/menu focus anchors and the one-shot focus latches (CR-CH-023).

/// The shell's unified Tab-order focus anchors and one-shot latches.
///
/// Validates: menu-and-statusbar Requirement 16.1-16.14 (CR-CH-023). Fields
/// retain their original semantics; only their access path changes.
#[derive(Default)]
pub(crate) struct FocusState {
    /// One-shot: request focus on the command field next frame.
    pub command_field_focus_requested: bool,
    /// Egui id of the FIRST focusable interior control of the active Workspace.
    pub first_interior_id: Option<egui::Id>,
    /// Egui id of the LAST focusable interior control of the active Workspace.
    pub last_interior_id: Option<egui::Id>,
    /// Egui id of the FIRST top-level Menu_Bar button.
    pub menu_first_id: Option<egui::Id>,
    /// Egui id of the LAST top-level Menu_Bar button.
    pub menu_last_id: Option<egui::Id>,
    /// One-shot: egui id of the open menu-bar dropdown item that the arrow-key
    /// walk wants focused NEXT frame (menu-workspace Req 17.5). It is stashed
    /// INSIDE the dropdown content closure and applied by the bar renderer AFTER
    /// egui's own `MenuBar` navigation has run, so the shell's deterministic
    /// walk is the LAST focus request and wins over egui's (which otherwise
    /// double-advances on Down and escapes the dropdown on Up). Cleared each
    /// frame once applied.
    pub menu_dropdown_focus_target: Option<egui::Id>,
    /// Index of the keyboard-selected item in the currently OPEN menu-bar
    /// dropdown, or `None` when no dropdown is open or none is selected yet
    /// (menu-workspace Req 17.5). The shell owns this walk deterministically:
    /// Down advances it, Up retreats it (clamped), and the selected item's id is
    /// FORCED into focus every frame via `menu_dropdown_focus_target` so egui's
    /// own menu navigation cannot drift it. Reset to `None` when the dropdown is
    /// not rendered (closed).
    pub menu_dropdown_index: Option<usize>,
    /// Set true by the dropdown renderer each frame a menu-bar dropdown is OPEN;
    /// the bar renderer reads it after egui's MenuBar runs to tell an open
    /// dropdown from a closed one (so it can reset `menu_dropdown_index` on
    /// close) and clears it for the next frame.
    pub menu_dropdown_open_this_frame: bool,
    /// Egui ids of ALL top-level Menu_Bar buttons (headings) that are visible on
    /// the bar, captured in bar order every frame (menu-workspace Req 17.5d/e).
    /// The shell-owned Left/Right walk along the bar indexes INTO this list to
    /// focus the previous/next heading and open its dropdown. Rebuilt each frame
    /// by the bar renderer; empty until the first render.
    pub menu_bar_heading_ids: Vec<egui::Id>,
    /// Index into `menu_bar_heading_ids` of the heading that currently owns the
    /// keyboard focus on the BAR (menu-workspace Req 17.5d/e). `Some` means the
    /// bar is in keyboard mode: the heading is focused and its dropdown is held
    /// open, Left/Right move between headings, and Down enters the open dropdown.
    /// `None` means the bar is not keyboard-driving the headings (focus is on a
    /// dropdown ITEM, or the bar is idle). Set when a dropdown opens, when Up
    /// returns from the first item to its heading, and when Left/Right move along
    /// the bar; cleared when the dropdowns all close or Down enters an item.
    pub menu_bar_index: Option<usize>,
    /// Index (into the bar order) of the heading whose dropdown actually RENDERED
    /// this frame, captured inside that heading's dropdown content closure
    /// (menu-workspace Req 17.5d). Lets the bar renderer learn which heading a
    /// CLICK opened (a click does not set `menu_bar_index`) so Left/Right and
    /// Up-to-heading know the current heading. Reset each frame before render.
    pub menu_dropdown_owner_index: Option<usize>,
    /// Active tab index observed on the previous frame (focus re-arm trigger).
    pub last_active_tab: usize,
    /// One-shot latch: focus the active Workspace's FIRST interior this frame.
    pub focus_first_interior_requested: bool,
    /// One-shot latch (reverse Shift+Tab): focus the LAST interior this frame.
    pub focus_last_interior_requested: bool,
}
