//! # Shell Panel Rendering -- split Workspace area
//!
//! The recursive split central panel, per-region tab bar / menu bar / command
//! field / Title_Line rendering, the split-region strip-rect geometry, and the
//! split-tab drop resolution. Split out of `render.rs` (TASK 2.2, pure code
//! movement, no behaviour change).

use eframe::egui;

use super::WorkbenchShell;

/// The five stacked strip rects of a split region, top to bottom (CR-CH-041 +
/// B072). Extracted as a pure value so the chrome ORDER is unit-testable without
/// egui: tab bar, menu bar, Title_Line, command line, then the Context body.
pub(crate) struct SplitRegionRects {
    pub bar_rect: egui::Rect,
    pub menu_rect: egui::Rect,
    pub title_rect: egui::Rect,
    pub cmd_rect: egui::Rect,
    pub body_rect: egui::Rect,
}

/// Compute a split region's stacked chrome strips from its outer `rect`
/// (CR-CH-041 Req 16.2, B072; CR-NR-095 Req 8.4). The top strips are always
/// TOP TO BOTTOM: tab bar (24), menu bar (24), Title_Line (20). The command line
/// (24) is placed per `position`:
/// - `Top` (default): directly UNDER the Title_Line and ABOVE the body -- the
///   unsplit / detached ISPF order, NOT the region bottom (the B072 fix).
/// - `Bottom`: the LAST strip, pinned to the region foot, with the body filling
///   the space between the Title_Line and the command line.
///
/// Each strip is clamped to `rect.max.y` so a very short region degrades
/// gracefully (the body shrinks toward empty rather than overflowing).
pub(crate) fn split_region_strip_rects(
    rect: egui::Rect,
    position: crate::workspace_kind::CommandLinePosition,
) -> SplitRegionRects {
    use crate::workspace_kind::CommandLinePosition;
    let tab_bar_h = 24.0_f32;
    let menu_bar_h = 24.0_f32;
    let title_h = 20.0_f32;
    let cmd_field_h = 24.0_f32;
    let bar_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.max.x, (rect.min.y + tab_bar_h).min(rect.max.y)),
    );
    let menu_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, bar_rect.max.y),
        egui::pos2(rect.max.x, (bar_rect.max.y + menu_bar_h).min(rect.max.y)),
    );
    let title_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, menu_rect.max.y),
        egui::pos2(rect.max.x, (menu_rect.max.y + title_h).min(rect.max.y)),
    );
    let (cmd_rect, body_rect) = match position {
        CommandLinePosition::Top => {
            // Command line directly UNDER the Title_Line (B072); body below it.
            let cmd_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x, title_rect.max.y),
                egui::pos2(rect.max.x, (title_rect.max.y + cmd_field_h).min(rect.max.y)),
            );
            let body_rect =
                egui::Rect::from_min_max(egui::pos2(rect.min.x, cmd_rect.max.y), rect.max);
            (cmd_rect, body_rect)
        }
        CommandLinePosition::Bottom => {
            // Command line is the LAST strip at the region foot; body fills the
            // space between the Title_Line and the command line. Clamp the command
            // strip top so a very short region does not push it above the title.
            let cmd_top = (rect.max.y - cmd_field_h).max(title_rect.max.y);
            let cmd_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x, cmd_top),
                egui::pos2(rect.max.x, rect.max.y),
            );
            let body_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x, title_rect.max.y),
                egui::pos2(rect.max.x, cmd_rect.min.y),
            );
            (cmd_rect, body_rect)
        }
    };
    SplitRegionRects {
        bar_rect,
        menu_rect,
        title_rect,
        cmd_rect,
        body_rect,
    }
}
impl WorkbenchShell {
    /// Render the recursive split inside the CentralPanel (CR-NR-093, Slice 2c.1).
    ///
    /// Walks the authoritative `TabGroupTree` to ARBITRARY depth: each internal
    /// `Split` node divides its rect by direction/proportion with a draggable
    /// Splitter; each `Leaf` is a region drawn by
    /// [`render_split_region`](Self::render_split_region) (its own tab bar + the
    /// active Context body via [`render_active_tab_body`], focused-leaf highlight).
    /// A snapshot of the tree is cloned up front so the immutable walk can compute
    /// rects while the per-leaf render borrows `&mut self`.
    ///
    /// Validates: layout-and-docking Requirement 14.1, 14.2, 14.4
    pub(super) fn render_split_central(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Defensive: caller checked is_split(); a single leaf renders normally.
        if !self.tabs.is_split() {
            self.render_active_tab_body(ctx, ui);
            return;
        }
        // CR-NR-094 Slice 2d: keep the per-region command-line contexts in
        // lockstep with the current leaves before rendering the regions.
        self.reconcile_region_cmd_ctx();
        // B073: reset the focused-region menu-first anchor; the focused region's
        // menu render (below) re-captures it this frame.
        self.detach_split.focused_region_menu_first = None;
        let tree = self.tabs.layout_tree().clone();
        let focused = self.tabs.focused_leaf_id();
        let full = ui.available_rect_before_wrap();
        // CR-NR-093 Slice 2c.2: rebuild the per-frame leaf-rect map so a
        // tab-header drag can be resolved to a drop target on release.
        self.detach_split.split_leaf_rects.clear();
        self.render_tree_node(ctx, ui, &tree, full, focused);
        // Resolve a completed tab-header drag (drop) now that every leaf rect is
        // known this frame (Req 14.6, 14.7, 14.8).
        self.resolve_split_tab_drop(ctx);
    }

    /// Reconcile the per-region command-line context map with the current split
    /// leaves (CR-NR-094, Slice 2d, Req 15.5). Inserts a fresh default
    /// `WorkspaceCommandContext` for any leaf id that lacks one, and drops any
    /// entry whose leaf no longer exists (collapsed/merged). Because leaf ids are
    /// allocated monotonically and never reused within a split session, a moved
    /// tab (Req 14.6) changes leaf MEMBERSHIP but not leaf IDENTITY, so command
    /// text stays with the leaf and never travels with a moved tab.
    ///
    /// Validates: layout-and-docking Requirement 15.5
    pub(super) fn reconcile_region_cmd_ctx(&mut self) {
        if !self.tabs.is_split() {
            // Unsplit: no per-region contexts (the single top-level field is used).
            if !self.detach_split.region_cmd_ctx.is_empty() {
                self.detach_split.region_cmd_ctx.clear();
            }
            return;
        }
        let leaves: std::collections::HashSet<ff_layout::TabGroupId> =
            self.tabs.leaf_ids().into_iter().collect();
        // Drop contexts for leaves that no longer exist.
        self.detach_split
            .region_cmd_ctx
            .retain(|id, _| leaves.contains(id));
        // Insert a fresh context for any new leaf.
        for id in leaves {
            self.detach_split.region_cmd_ctx.entry(id).or_default();
        }
    }

    /// Resolve an in-progress tab-header drag on release (CR-NR-093, Slice 2c.2).
    ///
    /// While `split_tab_drag` is set, on pointer release: if the pointer is over
    /// a DIFFERENT leaf's recorded rect, move the dragged tab into that leaf
    /// (Req 14.6); a drop over the tab's own leaf is a no-op (Req 14.8). Dropping
    /// OUTSIDE every leaf rect (e.g. beyond the workbench) falls through to the
    /// existing detach gesture handled by the tab bar, so it is left alone here.
    /// The drag state is cleared on release regardless.
    ///
    /// Validates: layout-and-docking Requirement 14.6, 14.7, 14.8
    fn resolve_split_tab_drop(&mut self, ctx: &egui::Context) {
        let Some((tab_id, source_leaf)) = self.detach_split.split_tab_drag else {
            return;
        };
        // Still dragging? Keep the state and wait for release.
        let pointer_released = ctx.input(|i| i.pointer.any_released());
        if !pointer_released {
            return;
        }
        // Release: clear the drag and, if over another leaf, perform the move.
        self.detach_split.split_tab_drag = None;
        let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) else {
            return;
        };
        let target = self
            .detach_split
            .split_leaf_rects
            .iter()
            .find(|(_, r)| r.contains(pos))
            .map(|(id, _)| *id);
        if let Some(target) = target {
            if target != source_leaf {
                self.tabs.move_tab_to_group(tab_id, target);
            }
        }
    }

    /// Recursively render one `TabGroupTree` node into `rect` (CR-NR-093).
    fn render_tree_node(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        node: &ff_layout::TabGroupTree,
        rect: egui::Rect,
        focused: ff_layout::TabGroupId,
    ) {
        use ff_layout::TabGroupTree;
        match node {
            TabGroupTree::Leaf(group) => {
                self.render_split_region(ctx, ui, group.id, rect, group.id == focused);
            }
            TabGroupTree::Split {
                direction,
                proportion,
                first,
                second,
            } => {
                let horizontal = matches!(direction, ff_layout::SplitDirection::Horizontal);
                let splitter_thickness = 6.0_f32;
                let min = ff_layout::MIN_TAB_GROUP_SIZE;
                let (first_rect, splitter_rect, second_rect) = if horizontal {
                    let avail = (rect.width() - splitter_thickness).max(0.0);
                    let mut first_w = (avail * *proportion).clamp(0.0, avail);
                    if avail >= 2.0 * min {
                        first_w = first_w.clamp(min, avail - min);
                    }
                    let x_split = rect.min.x + first_w;
                    (
                        egui::Rect::from_min_max(rect.min, egui::pos2(x_split, rect.max.y)),
                        egui::Rect::from_min_max(
                            egui::pos2(x_split, rect.min.y),
                            egui::pos2(x_split + splitter_thickness, rect.max.y),
                        ),
                        egui::Rect::from_min_max(
                            egui::pos2(x_split + splitter_thickness, rect.min.y),
                            rect.max,
                        ),
                    )
                } else {
                    let avail = (rect.height() - splitter_thickness).max(0.0);
                    let mut first_h = (avail * *proportion).clamp(0.0, avail);
                    if avail >= 2.0 * min {
                        first_h = first_h.clamp(min, avail - min);
                    }
                    let y_split = rect.min.y + first_h;
                    (
                        egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, y_split)),
                        egui::Rect::from_min_max(
                            egui::pos2(rect.min.x, y_split),
                            egui::pos2(rect.max.x, y_split + splitter_thickness),
                        ),
                        egui::Rect::from_min_max(
                            egui::pos2(rect.min.x, y_split + splitter_thickness),
                            rect.max,
                        ),
                    )
                };

                // Recurse into children first (they draw regions + nested nodes).
                self.render_tree_node(ctx, ui, first, first_rect, focused);
                self.render_tree_node(ctx, ui, second, second_rect, focused);

                // Draggable Splitter for THIS node (Req 14.4). Identity: the
                // first leaf id of the first child (stable per node).
                let node_key = first.all_group_ids().first().copied();
                let splitter_id = ui
                    .id()
                    .with(("workspace_splitter", node_key.map(|k| k.value())));
                let resp = ui.interact(splitter_rect, splitter_id, egui::Sense::click_and_drag());
                let hovered = resp.hovered() || resp.dragged();
                let visual = if hovered {
                    ui.visuals().widgets.active.bg_fill
                } else {
                    ui.visuals().widgets.noninteractive.bg_stroke.color
                };
                ui.painter().rect_filled(splitter_rect, 0.0, visual);
                if hovered {
                    ctx.set_cursor_icon(if horizontal {
                        egui::CursorIcon::ResizeHorizontal
                    } else {
                        egui::CursorIcon::ResizeVertical
                    });
                }
                if resp.dragged() {
                    if let (Some(pos), Some(key)) =
                        (ctx.input(|i| i.pointer.interact_pos()), node_key)
                    {
                        let new_prop = if horizontal {
                            (pos.x - rect.min.x) / rect.width().max(1.0)
                        } else {
                            (pos.y - rect.min.y) / rect.height().max(1.0)
                        };
                        self.tabs.set_node_proportion(key, new_prop);
                    }
                }
            }
        }
    }
}
