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
