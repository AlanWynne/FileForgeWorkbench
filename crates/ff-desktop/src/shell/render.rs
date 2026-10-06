//! # Shell Panel Rendering -- frame orchestration
//!
//! Central-panel orchestration and the shared Boundary_Policy focus helpers
//! (`render_central_panel`, `render_workspace_context`, `apply_interior_focus`,
//! `honour_interior_focus_latch`, `apply_shell_requests`, `placement_of`). The
//! per-region render helpers live in sibling `render_*.rs` modules (TASK 2.2
//! split, pure code movement).

use eframe::egui;

use crate::tab_state::KindTag;
use crate::toolchain_panel;

use super::WorkbenchShell;

/// Direction of an arrow-history step requested by a focused command field
/// (CR-NR-096, function-keys-and-history Requirement 23).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HistoryStep {
    /// Up arrow -- recall an OLDER entry (same as RETRIEVE, Req 23.1).
    Older,
    /// Down arrow -- step NEWER / restore the In_Progress_Line (Req 23.2, 23.3).
    Newer,
}

/// What a single frame of the shared command-field body observed: an optional
/// submitted command line (Enter) and/or an optional arrow-history step. Enter
/// and a history step are mutually exclusive within a frame (Enter wins). The
/// caller -- which owns the shared command processor history and In_Progress_Line
/// -- acts on whichever signal is present.
///
/// Validates: function-keys-and-history Requirement 23.9 (one behaviour, every field)
#[derive(Debug, Clone, Default)]
pub(super) struct CommandFieldSignal {
    /// `Some(command)` when the user pressed Enter on a non-empty line this frame.
    pub submitted: Option<String>,
    /// `Some(direction)` when the user pressed Up/Down while the field had focus.
    pub history_step: Option<HistoryStep>,
}

/// Compute the logging-degradation reason for the status-bar indicator (B038,
/// CR-NR-086, logging-subsystem Req 8.7). Returns `Some(reason)` when logging
/// has degraded -- the subsystem is in fallback (no-op) mode, or records have
/// been dropped -- and `None` when logging is healthy (indicator hidden).
///
/// Pure so the decision is unit-testable without a running log subsystem; the
/// live values are supplied by the caller via `ff_logging::is_fallback()` /
/// `dropped_count()`.
pub(crate) fn logging_degradation_reason(is_fallback: bool, dropped: u64) -> Option<String> {
    if !is_fallback && dropped == 0 {
        return None;
    }
    let mut reason = String::new();
    if is_fallback {
        reason.push_str("log file unavailable (fallback mode)");
    }
    if dropped > 0 {
        if !reason.is_empty() {
            reason.push_str("; ");
        }
        reason.push_str(&format!("{dropped} log record(s) dropped"));
    }
    Some(reason)
}

impl WorkbenchShell {
    /// The derived [`Placement`](crate::tab_manager::Placement) of the Workspace
    /// instance `tab_id` (CR-CH-041, Req 16.5/16.6). Detached takes precedence:
    /// a tab recorded in `floating_tabs` is in its own OS window; otherwise it is
    /// Docked in the layout-tree leaf that owns it (the root leaf when unsplit).
    /// Falls back to `Docked { root }` when the tab is neither floating nor found
    /// in any leaf (should not happen for a live tab; keeps the accessor total).
    ///
    /// Placement is DERIVED every call from the floating set + layout tree; it is
    /// never stored on the instance or persisted on its `Workspace_Descriptor`
    /// (the layout snapshot is the single source of truth for placement).
    // CR-CH-041: the public model accessor for an instance's derived Placement
    // (Req 16.5/16.6), validated by unit + full-shell tests. The in-window split
    // render resolves the leaf directly from the tree walk (so it does not call
    // this), but it is the canonical Placement query for the detached-fold-in
    // path and callers that hold only a TabId; `allow(dead_code)` because the
    // lib/bin clippy scope does not see its test consumers.
    #[allow(dead_code)]
    pub(super) fn placement_of(
        &self,
        tab_id: crate::tab_state::TabId,
    ) -> crate::tab_manager::Placement {
        use crate::tab_manager::Placement;
        if self
            .detach_split
            .floating_tabs
            .iter()
            .any(|ft| ft.tab_id == tab_id)
        {
            return Placement::Detached;
        }
        match self.tabs.docked_leaf_of(tab_id) {
            Some(leaf) => Placement::Docked { leaf },
            None => Placement::Docked {
                leaf: ff_layout::TabGroupId::new(0),
            },
        }
    }

    // ── Central panel ────────────────────────────────────────────────────

    /// Honour the one-shot interior-focus latches set by the Boundary_Policy
    /// (B056). Called by each Workspace render arm AFTER it has produced its
    /// interior controls this frame, so the id passed is the FRESH same-frame id
    /// that egui will actually recognise (a stale previous-frame id does not
    /// round-trip through `request_focus`).
    ///
    /// Validates: Requirement 16.3, 16.8 (CR-CH-023)
    fn honour_interior_focus_latch(
        &mut self,
        ctx: &egui::Context,
        first_interior: Option<egui::Id>,
        last_interior: Option<egui::Id>,
    ) {
        if self.focus.focus_first_interior_requested {
            self.focus.focus_first_interior_requested = false;
            if let Some(id) = first_interior {
                ctx.memory_mut(|m| m.request_focus(id));
            }
        }
        if self.focus.focus_last_interior_requested {
            self.focus.focus_last_interior_requested = false;
            if let Some(id) = last_interior {
                ctx.memory_mut(|m| m.request_focus(id));
            }
        }
    }

    /// Apply the [`ShellRequest`]s a `WorkspaceContext` enqueued during its
    /// `render`, routing each through the existing shell pipelines so the
    /// observable result is identical to the pre-framework per-arm handling
    /// (CR-NR-078, Requirement 2.2). Drained AFTER the panel is put back, so no
    /// borrow conflicts arise.
    pub(super) fn apply_shell_requests(
        &mut self,
        requests: Vec<super::workspace_context::ShellRequest>,
    ) {
        use super::workspace_context::ShellRequest;
        for req in requests {
            match req {
                ShellRequest::Command(cmd) => self.handle_command(&cmd),
                ShellRequest::Target(target) => self.dispatch_command_target(&target),
                ShellRequest::OpenFile(path) => {
                    let mut p = ff_command::CommandParams::new();
                    p.insert("path", path.as_str());
                    if let ff_command::CommandResult::Err(e) =
                        self.dispatch.execute_command("file.open", p)
                    {
                        self.open_error = Some(e.to_string());
                    }
                }
                ShellRequest::Status(message) => self.open_error = Some(message),
            }
        }
    }

    /// Dispatch a [`WorkspaceContext`] through the single framework code path
    /// (CR-NR-078, Requirement 1.2), using the owned-panel-swap borrow strategy
    /// (Option A): the caller moves the panel OUT of the shell, hands it here
    /// with a `ShellServices` builder that borrows the shell's OTHER fields, we
    /// render, drain the panel's `ShellRequest`s, and honour the focus latch with
    /// the returned `InteriorFocus`. The panel is put back by the caller. This is
    /// the ONE place the focus contract is applied, so a migrated arm cannot omit
    /// it.
    ///
    /// Returns the `InteriorFocus` the Context reported (so the caller records it
    /// on `first_interior_id`/`last_interior_id`).
    pub(super) fn render_workspace_context<C>(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        context: &mut C,
    ) -> super::workspace_context::InteriorFocus
    where
        C: super::workspace_context::WorkspaceContext,
    {
        use super::workspace_context::ShellServices;
        let mut requests = Vec::new();
        let focus = {
            let mut services = ShellServices {
                config: &self.config_handle,
                runtime: &self.runtime,
                notifications: &self.notification_queue,
                themes_dir: self.themes_dir(),
                menus_dir: self.menus_dir(),
                command_store: &self.command_store,
                requests: &mut requests,
            };
            context.render(ui, &mut services)
        };
        self.apply_shell_requests(requests);
        self.apply_interior_focus(ctx, focus);
        focus
    }

    /// Record a Context's reported [`InteriorFocus`] on the shell and honour the
    /// focus latch -- the ONE place the CR-CH-023 Boundary_Policy interior anchors
    /// are applied (CR-NR-078, Requirement 1.2). Both the generic trait dispatch
    /// (`render_workspace_context`) and the specialized MenuWorkspace arm (whose
    /// render signature carries menu-specific calendar inputs/outputs and so does
    /// not fit the `ShellServices`-only trait) route through here, so no arm can
    /// "forget" to apply the focus contract.
    pub(super) fn apply_interior_focus(
        &mut self,
        ctx: &egui::Context,
        focus: super::workspace_context::InteriorFocus,
    ) {
        self.focus.first_interior_id = focus.first;
        self.focus.last_interior_id = focus.last;
        self.honour_interior_focus_latch(ctx, focus.first, focus.last);
    }

    pub(super) fn render_central_panel(&mut self, ctx: &egui::Context) {
        // ── File Explorer side panel (ctx-level, resizable) ────────────
        // Must be shown before CentralPanel so egui allocates space correctly.
        // Validates: Requirement 1.3 file-tree-panel, fix B019
        // Validates: Requirement 20.1 -- Tab from CommandField in FilesPanel transfers focus
        // to the first catalog node in the catalog tree.
        // ── Tab from FilesPanel command field → first catalog node ──────
        // Validates: Requirement 20.1
        // The Files Panel has its own internal "Command ===>" TextEdit.
        // When that field has egui focus and the user presses plain Tab,
        // we consume the Tab event and request focus on the first catalog node.
        // We cannot use focus_stop here because focus_stop tracks the shell's
        // top-level command field, not the panel-internal one.
        let is_files_panel = self.tabs.active_tab().kind.tag() == KindTag::FilesPanel;
        if is_files_panel && !self.modal_open {
            let files_cmd_id = egui::Id::new("files_panel_cmd");
            let files_cmd_focused = ctx.memory(|m| m.focused() == Some(files_cmd_id));
            if files_cmd_focused {
                let tab_pressed = ctx.input_mut(|i| {
                    if i.key_pressed(egui::Key::Tab) && !i.modifiers.shift {
                        i.events.retain(|e| {
                            !matches!(
                                e,
                                egui::Event::Key {
                                    key: egui::Key::Tab,
                                    ..
                                }
                            )
                        });
                        return true;
                    }
                    false
                });
                if tab_pressed {
                    self.files_panel.tree_focus_requested = true;
                }
            }
        }

        // CR-NR-093 B071: the ctx-level full-window File Explorer only applies
        // when the Workspace is NOT split. While split, the File Explorer renders
        // INSIDE its region (via render_active_tab_body's FileExplorerPanel arm),
        // so the split must stay visible -- do not take the full-window branch.
        let is_file_explorer = self.tabs.active_tab().kind.tag() == KindTag::FileExplorerPanel
            && !self.tabs.is_split();
        // CR-NR-060 Slice A: the NavModel-backed modern explorer is the sole File
        // Explorer content (legacy inline tree retired).
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
                        let _ = self.shell_open_file(&file);
                        self.nav_manager.locate(&line.to_string(), &mut self.tabs);
                        let _ = col; // column navigation deferred to Phase W follow-up
                    }
                });
        }

        // CR-NR-060 Slice A: the modern NavModel explorer is the File Explorer
        // Context content. Rendered here -- after all bottom/side panels -- so
        // the CentralPanel is added last (egui panel-ordering requirement).
        if is_file_explorer {
            self.render_nav_explorer(ctx);
        }

        // Validates: Requirement 14.8 — central panel dispatches on tab kind
        if !is_file_explorer {
            egui::CentralPanel::default().show(ctx, |ui| {
                // CR-NR-092 (Slice 2b): when split, the central panel is divided
                // into two regions, each with its own tab bar + Context body and
                // a draggable Splitter between them (Req 13.1, 13.4, 13.5, 13.6).
                if self.tabs.is_split() {
                    self.render_split_central(ctx, ui);
                } else {
                    self.render_active_tab_body(ctx, ui);
                }
            });
        } // end !is_file_explorer
    }
}

// === Helpers ================================================================

/// Draw a 2 px focus ring around `rect` using the theme's `focus_ring` colour.
///
/// Retained for accessibility Requirement 3.1/3.3/3.4 (focus indicator helper).
/// Under CR-CH-023 the shell no longer drives a per-widget focus ring
/// (egui draws its own focus highlight for the focused Interior_Control and
/// Menu_Bar item), so this helper currently has no production caller; kept for
/// the accessibility API contract and potential future custom indicators.
#[allow(dead_code)]
pub(crate) fn render_focus_indicator(
    ui: &egui::Ui,
    rect: egui::Rect,
    palette: &ff_theme::ThemePalette,
) {
    // CR-CH-056 Req 23.3/23.8: the focus-ring colour comes from the Theme's
    // egui-native chrome layer.
    let colour = palette.chrome_style.focus_ring_color();
    ui.painter().rect_stroke(
        rect.expand(2.0),
        2.0,
        egui::Stroke::new(2.0_f32, colour),
        egui::StrokeKind::Outside,
    );
}
