//! # Shell Panel Rendering -- per-region split chrome
//!
//! One split region's tab bar + Context body + focus highlight, plus the
//! per-region menu bar and command field. Split out of `render.rs` /
//! `render_split.rs` (TASK 2.2, pure code movement, no behaviour change).

use eframe::egui;

use super::render_split::{split_region_strip_rects, SplitRegionRects};
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Render one split region (CR-NR-093): a per-leaf tab bar in a strip at the
    /// top of `rect`, then that leaf's active Context body below it, then a
    /// focus-highlight border if `is_focused`. Clicking a tab header or the body
    /// focuses this region and activates the clicked tab (Req 14.5).
    pub(super) fn render_split_region(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        leaf_id: ff_layout::TabGroupId,
        rect: egui::Rect,
        is_focused: bool,
    ) {
        // CR-CH-041 (Req 16.2) + B072: a split region draws the FULL per-instance
        // chrome of the instance placed in it, TOP TO BOTTOM in the SAME order as
        // the unsplit / detached placements (ISPF order): tab bar, then the
        // instance's Kind menu bar, then its Title_Line, then this region's own
        // `Command ===>` line, then the Context body. The rect math is a PURE
        // helper (`split_region_strip_rects`) so the ordering is unit-testable
        // without egui (B072 regression guard).
        // CR-NR-095 (Req 8.4): the command line's position is that of the Kind
        // of the instance shown in THIS region (its active tab), resolved via the
        // same registry seam as everywhere else. Falls back to Top when the leaf
        // has no active tab.
        let region_cmd_position = self
            .tabs
            .leaf_active_store_index(leaf_id)
            .map(|idx| self.command_line_position_for(idx))
            .unwrap_or(crate::workspace_kind::CommandLinePosition::Top);
        let SplitRegionRects {
            bar_rect,
            menu_rect,
            title_rect,
            cmd_rect,
            body_rect,
        } = split_region_strip_rects(rect, region_cmd_position);

        // CR-NR-093 Slice 2c.2: record this leaf's rect so a tab-header drag can
        // be resolved to a drop target on release (Req 14.6).
        self.detach_split.split_leaf_rects.push((leaf_id, rect));

        // === Per-leaf tab bar ===
        let indices = self.tabs.leaf_tab_store_indices(leaf_id);
        let active_store = self.tabs.leaf_active_store_index(leaf_id);
        // CR-CH-056 Req 23.3/23.9: per-leaf tab-bar chrome colours from the
        // Theme's egui-native chrome layer (not the flat `palette.tab_bar.*`).
        let chrome = &self.palette.chrome_style;
        let active_bg = chrome.tab_active_bg_color();
        let inactive_bg = chrome.tab_inactive_bg_color();
        let active_text = chrome.tab_active_text_color();
        let inactive_text = chrome.tab_inactive_text_color();

        let mut clicked_store: Option<usize> = None;
        // A tab-header drag that STARTED this frame in this region: (tab_id, leaf).
        let mut drag_started: Option<(crate::tab_state::TabId, ff_layout::TabGroupId)> = None;
        let mut bar_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(bar_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        bar_ui.set_clip_rect(bar_rect);
        for store_idx in &indices {
            let store_idx = *store_idx;
            let Some(tab) = self.tabs.tabs().get(store_idx) else {
                continue;
            };
            let tab_id = tab.id;
            let is_active = Some(store_idx) == active_store;
            let base_title = self.kind_title(tab);
            let label = if tab.is_modified {
                format!("\u{25cf} {}", base_title)
            } else {
                base_title
            };
            let bg = if is_active { active_bg } else { inactive_bg };
            let fg = if is_active {
                active_text
            } else {
                inactive_text
            };
            // CR-NR-093 Slice 2c.2: headers sense click AND drag so a tab can be
            // dragged from one region and dropped onto another (Req 14.6).
            let btn = egui::Button::new(egui::RichText::new(&label).color(fg).monospace())
                .fill(bg)
                .stroke(if is_active {
                    egui::Stroke::new(1.0_f32, fg)
                } else {
                    egui::Stroke::NONE
                })
                .min_size(egui::vec2(0.0, bar_rect.height()))
                .sense(egui::Sense::click_and_drag());
            let resp = bar_ui.add(btn);
            if resp.clicked() {
                clicked_store = Some(store_idx);
            }
            if resp.drag_started() {
                drag_started = Some((tab_id, leaf_id));
            }
        }
        // Record a newly-started drag on the shell (resolved on release in
        // resolve_split_tab_drop). A fresh drag_started supersedes any stale one.
        if let Some(started) = drag_started {
            self.detach_split.split_tab_drag = Some(started);
        }

        // === Per-region menu bar (CR-CH-041, Req 16.2) ===
        // The instance placed in this region owns its menu bar; resolve it from
        // that instance's Kind and draw it into the region's menu strip via the
        // shared Ui-level renderer (the same one the app-level and detached bars
        // use). The bar is scoped by leaf id so its widget ids are per-region and
        // never collide with another region's or the (suppressed-while-split)
        // app-level bar.
        self.render_region_menu_bar(ui, leaf_id, menu_rect);

        // === Per-region Title_Line (CR-CH-041, Req 16.2) ===
        // The instance's Title_Line, drawn into the region's title strip via the
        // shared Ui-level painter (byte-identical styling to the app-level one).
        if let Some(store_idx) = active_store {
            let mut title_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(title_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            title_ui.set_clip_rect(title_rect);
            self.render_title_line_into_ui(&mut title_ui, store_idx);
        }

        // === Region body: render this leaf's active Context ===
        let body_response = ui.interact(
            body_rect,
            ui.id().with(("split_region_body", leaf_id.value())),
            egui::Sense::click(),
        );
        let token = self.tabs.set_render_focus_leaf(leaf_id);
        let mut body_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(body_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        body_ui.set_clip_rect(body_rect);
        self.render_active_tab_body(ctx, &mut body_ui);
        self.tabs.restore_render_focus(token);

        // === Per-region command line (CR-NR-094 Slice 2d, Req 15.1-15.9) ===
        self.render_region_command_field(ctx, ui, leaf_id, cmd_rect);

        // === Focus highlight (Req 14.4) ===
        if is_focused {
            let accent = self.palette.chrome_style.accent_color();
            ui.painter().rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(2.0_f32, accent),
                egui::StrokeKind::Inside,
            );
        }

        // === Drop_Zone highlight (Req 14.9) ===
        // While a tab-header drag is in progress, mark the region under the
        // pointer as the drop target (unless it is the tab's own region).
        if let Some((_, source_leaf)) = self.detach_split.split_tab_drag {
            if source_leaf != leaf_id {
                let over = ctx
                    .input(|i| i.pointer.interact_pos())
                    .map(|p| rect.contains(p))
                    .unwrap_or(false);
                if over {
                    let accent = self.palette.chrome_style.accent_color();
                    // A translucent fill + a thicker border so the drop target is
                    // unambiguous before release.
                    let fill = egui::Color32::from_rgba_unmultiplied(
                        accent.r(),
                        accent.g(),
                        accent.b(),
                        40,
                    );
                    ui.painter().rect_filled(rect, 0.0, fill);
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(3.0_f32, accent),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }

        // === Focus routing (Req 14.5) ===
        if let Some(store_idx) = clicked_store {
            self.tabs.focus_leaf_and_activate(leaf_id, store_idx);
        } else if body_response.clicked() {
            // Clicking anywhere in the region focuses it without changing its
            // active tab.
            self.tabs.focus_leaf(leaf_id);
        }
    }

    /// Render the per-region menu bar into `menu_rect` for the instance placed in
    /// leaf `leaf_id` (CR-CH-041, Req 16.2). The bar is the instance's Kind menu
    /// bar (resolved via `resolve_menu_bar_menu_for` -- workspace-kinds Req 4),
    /// drawn through the shared Ui-level renderer inside a per-leaf-salted id
    /// scope so its widget ids never collide with another region's bar or the
    /// (suppressed-while-split) app-level bar. No-op when the leaf has no active
    /// tab (an empty region degrades gracefully).
    ///
    /// Validates: layout-and-docking Requirement 16.2, 16.4, 16.8
    fn render_region_menu_bar(
        &mut self,
        ui: &mut egui::Ui,
        leaf_id: ff_layout::TabGroupId,
        menu_rect: egui::Rect,
    ) {
        let Some(store_idx) = self.tabs.leaf_active_store_index(leaf_id) else {
            return;
        };
        let Some(tab) = self.tabs.tabs().get(store_idx) else {
            return;
        };
        let menu = self.resolve_menu_bar_menu_for(tab);
        let mut menu_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(menu_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        menu_ui.set_clip_rect(menu_rect);
        // Per-region id scope so menu button ids are salted by leaf (Req 16.8:
        // stable, non-colliding ids; the workspace-conformance no-phantom-stop
        // contract). The shared renderer also updates menu_first_id/menu_last_id;
        // while split the app-level bar is suppressed so the focused region's bar
        // is the live one the Boundary_Policy uses.
        menu_ui.push_id(("region_menu_bar", leaf_id.value()), |ui| {
            self.render_menu_bar_into_ui(ui, &menu);
        });
        // B073: if THIS is the focused region, record its menu-bar first-button id
        // so the shell can include it in the focused-region Tab cycle (keeping Tab
        // inside the region). `render_menu_bar_into_ui` just set `menu_first_id`
        // for the bar it drew; capture it for the focused leaf only.
        if self.tabs.focused_leaf_id() == leaf_id {
            self.detach_split.focused_region_menu_first = self.focus.menu_first_id;
        }
    }

    /// Render one split region's own `Command ===>` line into `cmd_rect` and, on
    /// Enter, dispatch the command against THAT region's active tab (CR-NR-094,
    /// Slice 2d, Req 15.1-15.9).
    ///
    /// The region's command context is taken out of `region_cmd_ctx` for the
    /// duration of the render/dispatch so the shared field body (which borrows
    /// only the field state, never `&mut self`) can be bound to it, and dispatch
    /// can route through `with_workspace_context` -- the SAME Focus_Context seam
    /// the Detached_Workspace uses -- installing the region's active tab and its
    /// command buffers, running the UNCHANGED pipeline, then swapping the
    /// (possibly command-modified) buffers back. The context is reinstated into
    /// the map afterwards so its text/status/scroll persist across frames while
    /// the split lives (Req 15.4, 15.6). Submitting a non-focused region's line
    /// also focuses that region (Req 15.7).
    ///
    /// The widget id is salted by `leaf_id` so it is a stable, per-region Tab
    /// stop that never collides with another region or the top-level field
    /// (B056, Req 15.9).
    ///
    /// Validates: layout-and-docking Requirement 15.1, 15.2, 15.3, 15.4, 15.6, 15.7, 15.9
    fn render_region_command_field(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        leaf_id: ff_layout::TabGroupId,
        cmd_rect: egui::Rect,
    ) {
        // Take this region's context out of the map so the shared body can borrow
        // its fields without also borrowing `self`. Reconciliation guarantees an
        // entry exists for every current leaf, but be defensive.
        let mut region_ctx = self
            .detach_split
            .region_cmd_ctx
            .remove(&leaf_id)
            .unwrap_or_default();
        let cmd_id = egui::Id::new(("region_command_field_input", leaf_id.value()));
        let accent = self.palette.chrome_style.accent_color();
        let modal_open = self.modal_open;

        let mut command_text = std::mem::take(&mut region_ctx.command_text);
        let mut focus_requested = region_ctx.command_field_focus_requested;
        let open_error = region_ctx.open_error.clone();

        let mut field_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(cmd_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        field_ui.set_clip_rect(cmd_rect);
        let signal = Self::render_command_field_body(
            ctx,
            &mut field_ui,
            cmd_id,
            &mut command_text,
            &mut focus_requested,
            modal_open,
            accent,
            open_error.as_deref(),
        );

        region_ctx.command_text = command_text;
        region_ctx.command_field_focus_requested = focus_requested;

        if let Some(cmd) = signal.submitted {
            // Dispatch against this region's active tab through the shared
            // Focus_Context seam. Focus the region first so a submit from a
            // non-focused region acts on and focuses that region (Req 15.7).
            self.tabs.focus_leaf(leaf_id);
            if let Some(store_index) = self.tabs.leaf_active_store_index(leaf_id) {
                self.with_workspace_context(store_index, &mut region_ctx, |shell| {
                    shell.run_command_line(&cmd);
                });
            }
            region_ctx.command_field_focus_requested = true;
        } else if let Some(step) = signal.history_step {
            // CR-NR-096 (Req 23.9): the SAME arrow-history behaviour for a split
            // region. Route through the Focus_Context seam so `self.command_text`
            // is this region's buffer while stepping; the history and the
            // In_Progress_Line are shared across all fields.
            if let Some(store_index) = self.tabs.leaf_active_store_index(leaf_id) {
                self.with_workspace_context(store_index, &mut region_ctx, |shell| {
                    shell.step_command_history(step);
                });
            }
        }

        // Reinstate the (possibly modified) context so it persists across frames.
        self.detach_split.region_cmd_ctx.insert(leaf_id, region_ctx);
    }
}
