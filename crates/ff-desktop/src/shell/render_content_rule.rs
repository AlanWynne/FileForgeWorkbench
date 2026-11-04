//! # Common-frame Content_Area boundary rule (live screen)
//!
//! The shell-owned horizontal RULE drawn between the `Command ===>` panel and
//! the workspace Content_Area, identical for EVERY Workspace Kind. It is the
//! live-screen twin of the rule the framed snapshot renderer draws in the same
//! place (the snapshot twin lives in `ff-screen-model`'s framed renderer).
//!
//! The rule is purely visual: it allocates a hover-only region (no focus sense),
//! so it is NEVER a Tab stop and cannot disturb the unified tab order or the
//! first-Tab focus contract (workspace-conformance.md).
//!
//! Validates: workspace-framework Requirement 7.30, 7.31.

use eframe::egui;

/// Height of the rule's panel in points: a hairline with a little air either side.
const CONTENT_RULE_HEIGHT: f32 = 3.0;

/// The stable id of the primary window's boundary-rule panel.
pub(crate) fn primary_content_rule_panel_id() -> egui::Id {
    egui::Id::new("content_rule")
}

/// The id under which a boundary-rule panel records its line's rect, readable
/// through `Context::read_response` (used by the live-screen tests).
pub(crate) fn content_rule_line_id(panel_id: egui::Id) -> egui::Id {
    panel_id.with("line")
}

/// Draw the boundary rule as its own thin panel, directly on the CONTENT side of
/// the command panel: call it immediately after the command panel is shown.
/// `at_top` mirrors the command panel (top panels stack downward, bottom panels
/// upward), so the rule always sits between the command line and the content.
pub(super) fn render_content_rule(ctx: &egui::Context, panel_id: egui::Id, at_top: bool) {
    let panel = if at_top {
        egui::TopBottomPanel::top(panel_id)
    } else {
        egui::TopBottomPanel::bottom(panel_id)
    };
    panel
        .exact_height(CONTENT_RULE_HEIGHT)
        .frame(egui::Frame::NONE)
        .show_separator_line(false)
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            // Hover-only: records the rect for tests, never takes focus.
            ui.interact(rect, content_rule_line_id(panel_id), egui::Sense::hover());
            let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
            ui.painter().hline(rect.x_range(), rect.center().y, stroke);
        });
}
