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

// === NavUiState ======================================================
// Modern-explorer (NavModel-backed) UI interaction state: selection, the
// rename / delete / new-child dialogs, keyboard-focus flag, and file clipboard.
// Grouped from the former flat `nav_*` fields (behaviour-preserving: only the
// access path changes, e.g. `self.nav_selection` -> `self.nav_ui.nav_selection`).

/// Modern-explorer UI interaction state grouped from the former flat `nav_*`
/// `WorkbenchShell` fields. A plain data container: every field keeps the same
/// type and semantics it had as a flat field; only the access path changes.
#[derive(Debug, Default)]
pub(crate) struct NavUiState {
    /// Selection/cursor state for the NavModel-backed explorer (Requirement 24.2).
    pub nav_selection: crate::explorer_view::ExplorerSelection,

    /// Active rename dialog for the modern explorer: (target node, edit buffer).
    /// `None` when no rename is in progress. (CR-NR-060 Slice A, Req 16 Rename.)
    pub nav_rename: Option<(ff_file_tree::NodeId, String)>,

    /// Active delete-confirmation dialog for the modern explorer: (target node,
    /// display label). `None` when no delete is pending. (Req 16 Delete.)
    pub nav_delete: Option<(ff_file_tree::NodeId, String)>,

    /// Active new-child dialog for the modern explorer: (parent directory node,
    /// is_directory, name buffer). `None` when none pending. (Req 16 New.)
    pub nav_new: Option<(ff_file_tree::NodeId, bool, String)>,

    /// When true, the modern explorer node list holds keyboard focus (Tab moved
    /// focus from the shell Command ===> into the tree). (Req 24.9 / 20.1.)
    pub nav_focused: bool,

    /// File clipboard for the modern explorer: source resource URIs marked for a
    /// copy, pasted into a target directory on Paste. (Req 21.1 / 21.3.)
    pub nav_file_clipboard: Vec<ff_vfs::ResourceUri>,
}

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
    /// Active tab index observed on the previous frame (focus re-arm trigger).
    pub last_active_tab: usize,
    /// One-shot latch: focus the active Workspace's FIRST interior this frame.
    pub focus_first_interior_requested: bool,
    /// One-shot latch (reverse Shift+Tab): focus the LAST interior this frame.
    pub focus_last_interior_requested: bool,
}
