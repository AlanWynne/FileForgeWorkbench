//! Painting and pointer interaction for the editor panel adapter.
//!
//! Reads the visible document lines, builds the display list (interleaving
//! exclusion-block placeholders), paints each row with its editable prefix
//! area, handles mouse click/drag selection, and performs Ctrl+C clipboard
//! copy. The pure geometry/display helpers come from the `ff-editor-panel`
//! crate; the `TabState`/`CommandEngine`/`ExcludeManager`/runtime wiring stays
//! here in `ff-desktop`.

use eframe::egui;
use ff_command_semantics::CommandEngine;
use ff_document_model::LineNumber;
use tokio::runtime::Runtime;

use super::{
    build_display_list, extract_selected_text, line_char_count, normalise_selection, DisplayRow,
    PREFIX_WIDTH,
};
use crate::exclude_manager::ExcludeManager;
use crate::tab_state::{TabId, TabState};

/// Paint the visible rows and handle pointer interaction and Ctrl+C copy.
///
/// Returns a status message to surface (prefix command error, or a copy
/// confirmation) when applicable. Behaviour is identical to the original inline
/// `render` body; only the extraction into a helper is new.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_and_interact(
    ui: &mut egui::Ui,
    tab: &mut TabState,
    runtime: &Runtime,
    cmd_engine: &mut CommandEngine,
    exclude_manager: &mut ExcludeManager,
    tab_id: TabId,
    available: egui::Rect,
    line_height_px: f32,
    visible_lines: u64,
    effective_font_pt: f32,
) -> Option<String> {
    let top_line = tab.viewport.top_line();
    let end_line = (top_line + visible_lines).saturating_sub(1);

    // ── Read visible lines ───────────────────────────────────────────────
    let lines: Vec<String> = runtime.block_on(async {
        let doc = tab.document.read().await;
        let line_count = doc.line_count();
        let actual_end = end_line.min(line_count);
        (top_line..=actual_end)
            .map(|ln| {
                let start = doc.line_start(LineNumber(ln - 1));
                let end = doc.line_end(LineNumber(ln - 1));
                let len = end.0.saturating_sub(start.0);
                if len == 0 {
                    String::new()
                } else {
                    doc.get_range(start, len)
                        .map(|b| String::from_utf8_lossy(&b).into_owned())
                        .unwrap_or_default()
                }
            })
            .collect()
    });

    // ── Build display list (interleave placeholders for exclusion blocks) ──
    let blocks = exclude_manager.exclusion_blocks(tab_id.0);
    let display_rows = build_display_list(top_line, end_line, tab.line_count, &lines, &blocks);

    // ── Paint lines ──────────────────────────────────────────────────────────
    let text_color = ui
        .visuals()
        .override_text_color
        .unwrap_or(egui::Color32::LIGHT_GRAY);
    let highlight_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 18);
    let selection_color = egui::Color32::from_rgba_unmultiplied(70, 130, 200, 80);
    let caret_color = egui::Color32::from_rgb(220, 220, 220);
    let prefix_bg = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8);
    let placeholder_color = egui::Color32::from_rgb(120, 120, 80);
    // Use effective font size from zoom state (Req 1.3, 1.6 view-zoom)
    let font = egui::FontId::monospace(effective_font_pt);
    let cursor_line = tab.cursor.cursor_line();
    let cursor_col = tab.cursor.cursor_column();
    let mut y = available.top();
    let mut prefix_error: Option<String> = None;

    for row in &display_rows {
        match row {
            DisplayRow::Placeholder { block } => {
                // ── Placeholder row (Req 6.2, 6.4) ───────────────────────
                let line_rect = egui::Rect::from_min_size(
                    egui::pos2(available.left(), y),
                    egui::vec2(available.width(), line_height_px),
                );
                ui.painter().rect_filled(
                    line_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(80, 80, 40, 30),
                );
                // Fixed "- - -" indicator in prefix column (Req 6.4)
                ui.painter().text(
                    egui::pos2(available.left() + 2.0, y),
                    egui::Align2::LEFT_TOP,
                    "- - -",
                    font.clone(),
                    placeholder_color,
                );
                // Placeholder text in content area
                ui.painter().text(
                    egui::pos2(available.left() + PREFIX_WIDTH, y),
                    egui::Align2::LEFT_TOP,
                    block.placeholder_text(),
                    font.clone(),
                    placeholder_color,
                );
                y += line_height_px;
            }
            DisplayRow::Line { doc_line, content } => {
                let display_ln = *doc_line;
                let line_rect = egui::Rect::from_min_size(
                    egui::pos2(available.left(), y),
                    egui::vec2(available.width(), line_height_px),
                );

                // Current-line highlight (Req 13.3)
                if display_ln == cursor_line {
                    ui.painter().rect_filled(line_rect, 0.0, highlight_color);
                }

                // Selection highlight (Req 13.5, 13.6)
                // Validates: Requirement 13.5 -- selected region is visually highlighted
                if let Some(sel) = tab.canvas_selection {
                    let (sl, sc, el, ec) = normalise_selection(sel.0, sel.1, sel.2, sel.3);
                    if display_ln >= sl && display_ln <= el {
                        let col_from = if display_ln == sl {
                            sc.saturating_sub(1)
                        } else {
                            0
                        };
                        let col_to = if display_ln == el {
                            ec.saturating_sub(1)
                        } else {
                            80
                        };
                        let sel_x = available.left() + PREFIX_WIDTH + col_from as f32 * 8.0;
                        let sel_w = (col_to.saturating_sub(col_from)) as f32 * 8.0;
                        if sel_w > 0.0 {
                            let sel_rect = egui::Rect::from_min_size(
                                egui::pos2(sel_x, y),
                                egui::vec2(sel_w, line_height_px),
                            );
                            ui.painter().rect_filled(sel_rect, 0.0, selection_color);
                        }
                    }
                }

                // ── Editable prefix area ──────────────────────────────────
                let prefix_rect = egui::Rect::from_min_size(
                    egui::pos2(available.left(), y),
                    egui::vec2(PREFIX_WIDTH - 4.0, line_height_px),
                );
                ui.painter().rect_filled(prefix_rect, 0.0, prefix_bg);
                let prefix_text = tab.prefix_inputs.entry(display_ln).or_default();
                let prefix_id = egui::Id::new(("prefix", display_ln));
                let prefix_response = ui.put(
                    prefix_rect,
                    egui::TextEdit::singleline(prefix_text)
                        .id(prefix_id)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(PREFIX_WIDTH - 4.0)
                        .clip_text(true),
                );
                // Submit when the TextEdit loses focus (egui loses focus on Enter press).
                // Validates: Requirement 1 line-commands -- gutter input submits to engine
                if prefix_response.lost_focus() && !prefix_text.trim().is_empty() {
                    let text = prefix_text.trim().to_string();
                    match cmd_engine.submit_line_command(display_ln, &text) {
                        Ok(()) => {
                            *prefix_text = String::new();
                        }
                        Err(status) => {
                            prefix_error = Some(status.text.clone());
                            *prefix_text = String::new();
                        }
                    }
                }

                // ── Line content ──────────────────────────────────────────
                let content_x = available.left() + PREFIX_WIDTH;
                ui.painter().text(
                    egui::pos2(content_x, y),
                    egui::Align2::LEFT_TOP,
                    content,
                    font.clone(),
                    text_color,
                );

                // Caret bar (Req 13.4)
                if display_ln == cursor_line {
                    let caret_x = content_x + (cursor_col.saturating_sub(1) as f32) * 8.0;
                    let caret_rect = egui::Rect::from_min_size(
                        egui::pos2(caret_x, y),
                        egui::vec2(2.0, line_height_px),
                    );
                    ui.painter().rect_filled(caret_rect, 0.0, caret_color);
                }

                y += line_height_px;
            }
        }
    }

    // ── Mouse click / drag -> cursor placement and text selection (Req 13.1-13.10) ──
    // Validates: Requirement 13.1 -- click sets cursor
    // Validates: Requirement 13.2 -- drag extends selection
    // Validates: Requirement 13.7 -- click without drag clears selection
    let response = ui.allocate_rect(available, egui::Sense::click_and_drag());

    let pos_to_line_col = |pos: egui::Pos2| -> (u64, u64) {
        let clicked_line_idx = ((pos.y - available.top()) / line_height_px).floor() as u64;
        let clicked_line = (top_line + clicked_line_idx).max(1);
        let col_offset = ((pos.x - available.left() - PREFIX_WIDTH) / 8.0).floor();
        let clicked_col = (col_offset as i64).max(0) as u64 + 1;
        (clicked_line, clicked_col)
    };

    if response.drag_started() {
        // Anchor the selection at the drag start position
        if let Some(pos) = response.interact_pointer_pos() {
            let (ln, col) = pos_to_line_col(pos);
            let line_len = runtime.block_on(async {
                let doc = tab.document.read().await;
                line_char_count(&doc, ln.saturating_sub(1))
            });
            let clamped_col = col.min(line_len + 1).max(1);
            tab.cursor.set_position(ln, clamped_col);
            // Start selection with zero extent at anchor
            tab.canvas_selection = Some((ln, clamped_col, ln, clamped_col));
        }
    } else if response.dragged() {
        // Extend selection to current pointer position
        if let Some(pos) = response.interact_pointer_pos() {
            let (ln, col) = pos_to_line_col(pos);
            let line_len = runtime.block_on(async {
                let doc = tab.document.read().await;
                line_char_count(&doc, ln.saturating_sub(1))
            });
            let clamped_col = col.min(line_len + 1).max(1);
            if let Some(sel) = tab.canvas_selection.as_mut() {
                sel.2 = ln;
                sel.3 = clamped_col;
            }
        }
    } else if response.clicked() {
        // Plain click: move cursor, clear selection
        if let Some(pos) = response.interact_pointer_pos() {
            let (clicked_line, clicked_col) = pos_to_line_col(pos);
            let line_len = runtime.block_on(async {
                let doc = tab.document.read().await;
                let line_idx = clicked_line.saturating_sub(1);
                line_char_count(&doc, line_idx)
            });
            let clamped_col = clicked_col.min(line_len + 1).max(1);
            tab.cursor.set_position(clicked_line, clamped_col);
            tab.canvas_selection = None;
        }
    }

    // ── Ctrl+C -- copy selection to OS clipboard (Req 20.1-20.6) ────────────
    // Validates: Requirement 20.1 -- Ctrl+C with active selection writes to OS clipboard
    // Validates: Requirement 20.3 -- no-op when no selection
    let ctrl_c = ui.input(|i| i.key_pressed(egui::Key::C) && i.modifiers.ctrl);
    if ctrl_c {
        if let Some(sel) = tab.canvas_selection {
            let (sl, sc, el, ec) = normalise_selection(sel.0, sel.1, sel.2, sel.3);
            if (sl, sc) != (el, ec) {
                let text = runtime.block_on(async {
                    let doc = tab.document.read().await;
                    extract_selected_text(&doc, (sel.0, sel.1, sel.2, sel.3))
                });
                let char_count = text.chars().count();
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let _ = cb.set_text(&text);
                }
                // Validates: Requirement 20.2 -- status message shows character count
                return Some(format!("Copied {char_count} characters"));
            }
        }
    }

    prefix_error
}
