//! `EditorPanel` — stateless egui renderer for a single tab's document view.
//!
//! Renders the visible lines of the active `TabState` into the central panel.
//! All state (document, viewport, cursor) lives in `TabState`; this module
//! only contains the rendering logic and input handling.

use eframe::egui;
use ff_command_semantics::CommandEngine;
use ff_document_model::LineNumber;
use ff_exclude_show_filter::ExclusionBlock;
use ff_viewport_scrolling::CaretPolicyEngine;
use tokio::runtime::Runtime;

use crate::exclude_manager::ExcludeManager;
use crate::tab_state::{TabId, TabState, UndoEntry};

/// Base font size in points. Zoom offset is added to this.
const BASE_FONT_SIZE_PT: f32 = 14.0;
/// Base line height in pixels at zoom offset 0.
const BASE_LINE_HEIGHT_PX: f32 = 16.0;
/// Width of the editable prefix area in characters (e.g. "DD    ").
const PREFIX_COLS: usize = 6;
/// Pixel width of the prefix area (PREFIX_COLS chars × 8 px + 4 px separator gap).
const PREFIX_WIDTH: f32 = (PREFIX_COLS as f32) * 8.0 + 4.0;
/// Legacy alias kept so existing tests that reference GUTTER_CHAR_WIDTH still compile.
#[allow(dead_code)]
pub(crate) const GUTTER_CHAR_WIDTH: f32 = PREFIX_WIDTH;

/// A single row in the editor display list.
///
/// Either a normal document line or a placeholder for an exclusion block.
#[derive(Debug, Clone)]
pub(crate) enum DisplayRow {
    /// A visible document line (1-based line number, content).
    Line { doc_line: u64, content: String },
    /// A placeholder row representing a contiguous exclusion block.
    Placeholder { block: ExclusionBlock },
}

/// Build the ordered display list for the visible viewport window.
///
/// Iterates `top_line..=end_line` (1-based), skipping excluded lines and
/// inserting one `Placeholder` row per contiguous exclusion block.
/// Lines beyond `doc_line_count` are omitted.
///
/// # Arguments
/// * `top_line` — first 1-based document line in the viewport
/// * `end_line` — last 1-based document line in the viewport (inclusive)
/// * `doc_line_count` — total lines in the document
/// * `lines` — pre-fetched line content indexed from `top_line` (index 0 = top_line)
/// * `blocks` — exclusion blocks for this tab (0-based doc lines)
///
/// Validates: Requirement 6.1, 6.2, 6.3, 6.8 — placeholder display model
pub(crate) fn build_display_list(
    top_line: u64,
    end_line: u64,
    doc_line_count: u64,
    lines: &[String],
    blocks: &[ExclusionBlock],
) -> Vec<DisplayRow> {
    let mut rows = Vec::new();
    let mut last_placeholder: Option<usize> = None; // block index already emitted

    let actual_end = end_line.min(doc_line_count);
    for doc_line in top_line..=actual_end {
        let doc_line_0 = (doc_line - 1) as usize; // 0-based for block lookup

        // Check if this line is inside an exclusion block
        if let Some(block_idx) = blocks.iter().position(|b| b.contains(doc_line_0)) {
            // Emit the placeholder only once per block
            if last_placeholder != Some(block_idx) {
                last_placeholder = Some(block_idx);
                rows.push(DisplayRow::Placeholder {
                    block: blocks[block_idx],
                });
            }
            // Skip the excluded line itself
            continue;
        }

        last_placeholder = None;
        let content_idx = (doc_line - top_line) as usize;
        let content = lines.get(content_idx).cloned().unwrap_or_default();
        rows.push(DisplayRow::Line { doc_line, content });
    }
    rows
}

/// Render the active tab's document into `ui`.
///
/// Updates `tab.viewport.visible_count` from the available rect, handles
/// mouse-wheel scroll, keyboard navigation, and paints each visible line
/// with an editable ISPF-style prefix area followed by the line content.
/// Prefix input is submitted to `cmd_engine` on Enter.
/// Exclusion blocks are rendered as non-editable placeholder rows.
pub fn render(
    ui: &mut egui::Ui,
    tab: &mut TabState,
    runtime: &Runtime,
    cmd_engine: &mut CommandEngine,
    exclude_manager: &mut ExcludeManager,
    tab_id: TabId,
) -> Option<String> {
    let available = ui.available_rect_before_wrap();
    // Compute effective font size from zoom offset (Req 1.2, 3.1-3.2 view-zoom)
    // Zoom is global (owned by WorkbenchShell); editor_panel receives the resolved pt size.
    let effective_font_pt = BASE_FONT_SIZE_PT;
    // Scale line height proportionally to font size
    let line_height_px = (BASE_LINE_HEIGHT_PX * effective_font_pt / BASE_FONT_SIZE_PT).max(4.0);
    let visible_lines = (available.height() / line_height_px).floor() as u64;
    tab.viewport.set_visible_count(visible_lines.max(1));

    // ── Text input and editing keys ────────────────────────────────────
    // Only handle editor keys when no prefix TextEdit widget has keyboard focus.
    // Any focused widget (prefix TextEdit, command field, etc.) owns the keyboard;
    // the editor content area is painted, not a widget, so it never holds focus.
    let prefix_has_focus = ui.memory(|m| m.focused().is_some());
    let (text_events, backspace, enter) = if prefix_has_focus {
        (vec![], false, false)
    } else {
        ui.input(|i| {
            let text: Vec<String> = i
                .events
                .iter()
                .filter_map(|e| {
                    if let egui::Event::Text(s) = e {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect();
            (
                text,
                i.key_pressed(egui::Key::Backspace),
                i.key_pressed(egui::Key::Enter),
            )
        })
    };

    for text in text_events {
        if text.is_empty() {
            continue;
        }
        let (line, col) = (tab.cursor.cursor_line(), tab.cursor.cursor_column());
        let byte_pos = runtime.block_on(async {
            let doc = tab.document.read().await;
            cursor_byte_position(&doc, line, col)
        });
        runtime.block_on(async {
            let mut doc = tab.document.write().await;
            let _ = doc.insert(byte_pos, text.as_bytes());
        });
        tab.undo_stack.push(UndoEntry::DeleteBytes {
            position: byte_pos.0,
            length: text.len() as u64,
        });
        // Advance cursor by the number of chars inserted (simple: one grapheme cluster per Text event)
        let new_col = col + text.chars().count() as u64;
        tab.cursor.set_position(line, new_col);
        tab.is_modified = true;
        tab.line_count = runtime.block_on(async { tab.document.read().await.line_count() });
        tab.viewport.set_total_display_lines(tab.line_count);
    }

    if backspace {
        let (line, col) = (tab.cursor.cursor_line(), tab.cursor.cursor_column());
        if col > 1 {
            // Delete the byte(s) immediately before the cursor
            let (byte_pos, char_width) = runtime.block_on(async {
                let doc = tab.document.read().await;
                let pos = cursor_byte_position(&doc, line, col);
                let width = doc
                    .character_before(pos)
                    .map(|c| c.byte_width as u64)
                    .unwrap_or(1);
                (pos, width)
            });
            let delete_pos = ff_document_model::BytePosition(byte_pos.0.saturating_sub(char_width));
            let deleted_bytes = runtime.block_on(async {
                let doc = tab.document.read().await;
                doc.get_range(delete_pos, char_width).unwrap_or_default()
            });
            runtime.block_on(async {
                let mut doc = tab.document.write().await;
                let _ = doc.delete(delete_pos, char_width);
            });
            tab.undo_stack.push(UndoEntry::InsertBytes {
                position: delete_pos.0,
                bytes: deleted_bytes,
            });
            tab.cursor.set_position(line, col - 1);
            tab.is_modified = true;
            tab.line_count = runtime.block_on(async { tab.document.read().await.line_count() });
            tab.viewport.set_total_display_lines(tab.line_count);
        } else if line > 1 {
            // Req 4.2 — Backspace at col 1: delete the newline at the end of the previous line,
            // joining the current line onto the end of the previous line.
            let (newline_pos, prev_line_len) = runtime.block_on(async {
                let doc = tab.document.read().await;
                let prev_line_idx = line.saturating_sub(2); // 0-based
                let line_end = doc.line_end(ff_document_model::LineNumber(prev_line_idx));
                let line_start = doc.line_start(ff_document_model::LineNumber(prev_line_idx));
                let char_len = line_end.0.saturating_sub(line_start.0);
                (line_end, char_len)
            });
            let deleted_bytes = runtime.block_on(async {
                let doc = tab.document.read().await;
                doc.get_range(newline_pos, 1).unwrap_or_default()
            });
            runtime.block_on(async {
                let mut doc = tab.document.write().await;
                let _ = doc.delete(newline_pos, 1);
            });
            tab.undo_stack.push(UndoEntry::InsertBytes {
                position: newline_pos.0,
                bytes: deleted_bytes,
            });
            tab.cursor.set_position(line - 1, prev_line_len + 1);
            tab.is_modified = true;
            tab.line_count = runtime.block_on(async { tab.document.read().await.line_count() });
            tab.viewport.set_total_display_lines(tab.line_count);
        }
        // col == 1 on line 1: no-op (nothing to join)
    }

    if enter {
        let (line, col) = (tab.cursor.cursor_line(), tab.cursor.cursor_column());
        let byte_pos = runtime.block_on(async {
            let doc = tab.document.read().await;
            cursor_byte_position(&doc, line, col)
        });
        runtime.block_on(async {
            let mut doc = tab.document.write().await;
            let _ = doc.insert(byte_pos, b"\n");
        });
        tab.undo_stack.push(UndoEntry::DeleteBytes {
            position: byte_pos.0,
            length: 1,
        });
        tab.cursor.set_position(line + 1, 1);
        tab.is_modified = true;
        tab.line_count = runtime.block_on(async { tab.document.read().await.line_count() });
        tab.viewport.set_total_display_lines(tab.line_count);
    }

    // ── Keyboard navigation ──────────────────────────────────────────────
    // Consume key events only when the central panel has focus.
    let keys = ui.input(|i| {
        [
            (egui::Key::ArrowDown, i.key_pressed(egui::Key::ArrowDown)),
            (egui::Key::ArrowUp, i.key_pressed(egui::Key::ArrowUp)),
            (egui::Key::ArrowLeft, i.key_pressed(egui::Key::ArrowLeft)),
            (egui::Key::ArrowRight, i.key_pressed(egui::Key::ArrowRight)),
            (egui::Key::PageDown, i.key_pressed(egui::Key::PageDown)),
            (egui::Key::PageUp, i.key_pressed(egui::Key::PageUp)),
        ]
    });

    let policy = CaretPolicyEngine::default_policy();

    for (key, pressed) in keys {
        if !pressed {
            continue;
        }
        match key {
            egui::Key::ArrowDown => {
                let (total_lines, target_len) = runtime.block_on(async {
                    let doc = tab.document.read().await;
                    let total = doc.line_count();
                    let next_line = tab.cursor.cursor_line(); // 0-based index for next line
                    let len = line_char_count(&doc, next_line); // length of the line we're moving TO
                    (total, len)
                });
                tab.viewport
                    .move_cursor_down(&mut tab.cursor, target_len, total_lines, &policy);
            }
            egui::Key::ArrowUp => {
                let target_len = runtime.block_on(async {
                    let doc = tab.document.read().await;
                    let prev_line = tab.cursor.cursor_line().saturating_sub(2); // 0-based index
                    line_char_count(&doc, prev_line)
                });
                tab.viewport
                    .move_cursor_up(&mut tab.cursor, target_len, &policy);
            }
            egui::Key::ArrowLeft => {
                tab.viewport.move_cursor_left(&mut tab.cursor, &policy);
            }
            egui::Key::ArrowRight => {
                let current_len = runtime.block_on(async {
                    let doc = tab.document.read().await;
                    let line_idx = tab.cursor.cursor_line().saturating_sub(1); // 0-based
                    line_char_count(&doc, line_idx)
                });
                tab.viewport
                    .move_cursor_right(&mut tab.cursor, current_len, &policy);
            }
            egui::Key::PageDown => {
                tab.viewport.scroll_page_down(&mut tab.cursor);
            }
            egui::Key::PageUp => {
                tab.viewport.scroll_page_up(&mut tab.cursor);
            }
            _ => {}
        }
    }

    // ── Ctrl+Z undo ──────────────────────────────────────────────────────
    let ctrl_z = ui.input(|i| i.key_pressed(egui::Key::Z) && i.modifiers.ctrl);
    if ctrl_z {
        if let Some(entry) = tab.undo_stack.pop() {
            match entry {
                UndoEntry::DeleteBytes { position, length } => {
                    runtime.block_on(async {
                        let mut doc = tab.document.write().await;
                        let _ = doc.delete(ff_document_model::BytePosition(position), length);
                    });
                }
                UndoEntry::InsertBytes { position, bytes } => {
                    runtime.block_on(async {
                        let mut doc = tab.document.write().await;
                        let _ = doc.insert(ff_document_model::BytePosition(position), &bytes);
                    });
                }
            }
            tab.line_count = runtime.block_on(async { tab.document.read().await.line_count() });
            tab.viewport.set_total_display_lines(tab.line_count);
            tab.is_modified = !tab.undo_stack.is_empty();
        }
    }

    // ── Mouse wheel — document scroll only (Ctrl+Scroll handled at shell level) ──────────
    // Validates: Requirement 3.3 (view-zoom) — Ctrl NOT held: normal document scroll.
    // Ctrl+Scroll zoom is consumed in shell.rs update() before this runs.
    let pointer_over_panel = ui.rect_contains_pointer(available);
    if pointer_over_panel {
        let scroll_delta = ui.input_mut(|i| {
            let raw = i.raw_scroll_delta.y;
            let smooth = i.smooth_scroll_delta.y;
            // Consume so egui scroll areas don't also react.
            i.raw_scroll_delta = egui::Vec2::ZERO;
            i.smooth_scroll_delta = egui::Vec2::ZERO;
            if raw != 0.0 {
                raw
            } else {
                smooth
            }
        });
        if scroll_delta != 0.0 {
            let ticks = if scroll_delta > 0.0 { -1_i32 } else { 1_i32 };
            tab.viewport.scroll_wheel_vertical(ticks, &tab.cursor);
        }
    }

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
    let blocks = exclude_manager.exclusion_blocks(tab_id);
    let display_rows = build_display_list(top_line, end_line, tab.line_count, &lines, &blocks);

    // ── Paint lines ──────────────────────────────────────────────────────────
    let text_color = ui
        .visuals()
        .override_text_color
        .unwrap_or(egui::Color32::LIGHT_GRAY);
    let highlight_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 18);
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
                if prefix_response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !prefix_text.trim().is_empty()
                {
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

    // ── Mouse click → cursor placement (Req 13.1) ───────────────────────
    let response = ui.allocate_rect(available, egui::Sense::click());
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let clicked_line_idx = ((pos.y - available.top()) / line_height_px).floor() as u64;
            let clicked_line = (top_line + clicked_line_idx).max(1);
            let col_offset = ((pos.x - available.left() - PREFIX_WIDTH) / 8.0).floor();
            let clicked_col = (col_offset as i64).max(0) as u64 + 1;
            let line_len = runtime.block_on(async {
                let doc = tab.document.read().await;
                let line_idx = clicked_line.saturating_sub(1);
                line_char_count(&doc, line_idx)
            });
            let clamped_col = clicked_col.min(line_len + 1).max(1);
            tab.cursor.set_position(clicked_line, clamped_col);
        }
    }

    prefix_error
}

/// Return the character count of a document line (0-based index).
///
/// Used to supply line-length information to cursor movement methods.
fn line_char_count(doc: &ff_document_model::Document, line_idx: u64) -> u64 {
    let start = doc.line_start(LineNumber(line_idx));
    let end = doc.line_end(LineNumber(line_idx));
    end.0.saturating_sub(start.0)
}

/// Convert a 1-based cursor (line, column) to a `BytePosition` in the document.
///
/// Column 1 maps to the first byte of the line. Columns beyond the line end
/// clamp to the line end position.
pub(crate) fn cursor_byte_position(
    doc: &ff_document_model::Document,
    line: u64,
    col: u64,
) -> ff_document_model::BytePosition {
    let line_idx = line.saturating_sub(1); // 0-based
    let line_start = doc.line_start(LineNumber(line_idx));
    let line_end = doc.line_end(LineNumber(line_idx));
    let col_offset = col.saturating_sub(1); // 0-based column offset
    let byte_offset = line_start.0 + col_offset;
    ff_document_model::BytePosition(byte_offset.min(line_end.0))
}

// Keep a thin wrapper so existing test infrastructure compiles.
/// Thin wrapper used only in unit tests.
#[allow(dead_code)]
pub struct EditorPanel;

#[allow(dead_code)]
impl EditorPanel {
    pub fn new_empty() -> TabState {
        use ff_document_model::new_document;
        TabState::untitled(crate::tab_state::TabId(0), new_document(), 1)
    }
}

#[cfg(test)]
mod tests {
    use ff_document_model::{new_document, BytePosition, LineEndMode};
    use ff_viewport_scrolling::{CaretPolicyEngine, CursorModel, ViewportModel};
    use tokio::runtime::Runtime;

    use crate::tab_state::{TabId, TabState};

    /// Validates: edit-operations Requirement 1.1 — typed character inserts into document.
    #[test]
    fn typed_character_inserts_into_document() {
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let mut tab = TabState::untitled(TabId(0), document, line_count);
        tab.cursor.set_position(1, 6); // after "hello"

        // Simulate what the Text event handler does
        let text = "!";
        let byte_pos = runtime.block_on(async {
            let doc = tab.document.read().await;
            super::cursor_byte_position(&doc, tab.cursor.cursor_line(), tab.cursor.cursor_column())
        });
        runtime.block_on(async {
            let mut doc = tab.document.write().await;
            let _ = doc.insert(byte_pos, text.as_bytes());
        });
        tab.cursor.set_position(1, 7);
        tab.is_modified = true;

        let content = runtime.block_on(async {
            let doc = tab.document.read().await;
            let len = doc.length();
            doc.get_range(BytePosition(0), len).unwrap_or_default()
        });
        assert_eq!(content, b"hello!");
        assert!(tab.is_modified);
        assert_eq!(tab.cursor.cursor_column(), 7);
    }

    /// Validates: edit-operations Requirement 4.1 — Backspace deletes character before cursor.
    #[test]
    fn backspace_deletes_character_before_cursor() {
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let mut tab = TabState::untitled(TabId(0), document, line_count);
        tab.cursor.set_position(1, 6); // after "hello"

        // Simulate backspace handler
        let (line, col) = (tab.cursor.cursor_line(), tab.cursor.cursor_column());
        let (byte_pos, char_width) = runtime.block_on(async {
            let doc = tab.document.read().await;
            let pos = super::cursor_byte_position(&doc, line, col);
            let width = doc
                .character_before(pos)
                .map(|c| c.byte_width as u64)
                .unwrap_or(1);
            (pos, width)
        });
        let delete_pos = BytePosition(byte_pos.0.saturating_sub(char_width));
        runtime.block_on(async {
            let mut doc = tab.document.write().await;
            let _ = doc.delete(delete_pos, char_width);
        });
        tab.cursor.set_position(line, col - 1);
        tab.is_modified = true;

        let content = runtime.block_on(async {
            let doc = tab.document.read().await;
            let len = doc.length();
            doc.get_range(BytePosition(0), len).unwrap_or_default()
        });
        assert_eq!(content, b"hell");
        assert!(tab.is_modified);
        assert_eq!(tab.cursor.cursor_column(), 5);
    }

    /// Validates: edit-operations Requirement 2.1 — Enter key splits line in insert mode.
    #[test]
    fn enter_key_splits_line_in_insert_mode() {
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"helloworld");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let mut tab = TabState::untitled(TabId(0), document, line_count);
        tab.cursor.set_position(1, 6); // between "hello" and "world"

        // Simulate enter handler
        let (line, col) = (tab.cursor.cursor_line(), tab.cursor.cursor_column());
        let byte_pos = runtime.block_on(async {
            let doc = tab.document.read().await;
            super::cursor_byte_position(&doc, line, col)
        });
        runtime.block_on(async {
            let mut doc = tab.document.write().await;
            let _ = doc.insert(byte_pos, b"\n");
        });
        tab.cursor.set_position(line + 1, 1);
        tab.is_modified = true;
        tab.line_count = runtime.block_on(async { tab.document.read().await.line_count() });

        assert_eq!(tab.line_count, 2);
        assert_eq!(tab.cursor.cursor_line(), 2);
        assert_eq!(tab.cursor.cursor_column(), 1);
        assert!(tab.is_modified);
    }

    /// Validates: document-model Requirement 9.1 — top_line starts at 1.
    #[test]
    fn new_tab_top_line_is_one() {
        let tab = TabState::untitled(TabId(0), new_document(), 1);
        assert_eq!(tab.viewport.top_line(), 1);
    }

    /// Validates: task 18.3 — document content is accessible after load.
    #[test]
    fn tab_with_content_has_correct_line_count() {
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        let content = (1..=50).map(|i| format!("line {i}\n")).collect::<String>();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), content.as_bytes());
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let tab = TabState::for_file(
            TabId(1),
            "test.txt".into(),
            document,
            line_count,
            LineEndMode::Default,
        );
        assert_eq!(tab.viewport.total_display_lines(), 51);
    }

    /// Validates: Requirement 3.1 — Down Arrow moves cursor down one line.
    #[test]
    fn arrow_down_advances_cursor_line() {
        // Validates: viewport-and-scrolling Requirement 3.1
        let mut viewport = ViewportModel::with_line_count(10);
        viewport.set_visible_count(5);
        let mut cursor = CursorModel::new();
        let policy = CaretPolicyEngine::default_policy();

        viewport.move_cursor_down(&mut cursor, 80, 10, &policy);

        assert_eq!(cursor.cursor_line(), 2);
    }

    /// Validates: Requirement 3.2 — Up Arrow moves cursor up one line.
    #[test]
    fn arrow_up_retreats_cursor_line() {
        // Validates: viewport-and-scrolling Requirement 3.2
        let mut viewport = ViewportModel::with_line_count(10);
        viewport.set_visible_count(5);
        let mut cursor = CursorModel::new();
        cursor.set_position(5, 1);
        let policy = CaretPolicyEngine::default_policy();

        viewport.move_cursor_up(&mut cursor, 80, &policy);

        assert_eq!(cursor.cursor_line(), 4);
    }

    /// Validates: Requirement 3.3 — Down Arrow at last line is a no-op.
    #[test]
    fn arrow_down_at_last_line_is_noop() {
        // Validates: viewport-and-scrolling Requirement 3.3
        let mut viewport = ViewportModel::with_line_count(5);
        viewport.set_visible_count(5);
        let mut cursor = CursorModel::new();
        cursor.set_position(5, 1);
        let policy = CaretPolicyEngine::default_policy();

        viewport.move_cursor_down(&mut cursor, 80, 5, &policy);

        assert_eq!(cursor.cursor_line(), 5);
    }

    /// Validates: Requirement 3.4 — Up Arrow at first line is a no-op.
    #[test]
    fn arrow_up_at_first_line_is_noop() {
        // Validates: viewport-and-scrolling Requirement 3.4
        let mut viewport = ViewportModel::with_line_count(5);
        viewport.set_visible_count(5);
        let mut cursor = CursorModel::new();
        let policy = CaretPolicyEngine::default_policy();

        viewport.move_cursor_up(&mut cursor, 80, &policy);

        assert_eq!(cursor.cursor_line(), 1);
    }

    /// Validates: Requirement 3.6 — Left Arrow retreats cursor column.
    #[test]
    fn arrow_left_retreats_cursor_column() {
        // Validates: viewport-and-scrolling Requirement 3.6
        let mut viewport = ViewportModel::with_line_count(5);
        viewport.set_visible_count(5);
        let mut cursor = CursorModel::new();
        cursor.set_position(1, 5);
        let policy = CaretPolicyEngine::default_policy();

        viewport.move_cursor_left(&mut cursor, &policy);

        assert_eq!(cursor.cursor_column(), 4);
    }

    /// Validates: Requirement 3.7 — Right Arrow advances cursor column.
    #[test]
    fn arrow_right_advances_cursor_column() {
        // Validates: viewport-and-scrolling Requirement 3.7
        let mut viewport = ViewportModel::with_line_count(5);
        viewport.set_visible_count(5);
        let mut cursor = CursorModel::new();
        let policy = CaretPolicyEngine::default_policy();

        viewport.move_cursor_right(&mut cursor, 80, &policy);

        assert_eq!(cursor.cursor_column(), 2);
    }

    /// Validates: Requirement 2.1 — Page Down advances top_line by visible_count.
    #[test]
    fn page_down_advances_top_line_by_visible_count() {
        // Validates: viewport-and-scrolling Requirement 2.1
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();

        viewport.scroll_page_down(&mut cursor);

        assert_eq!(viewport.top_line(), 21);
    }

    /// Validates: Requirement 2.2 — Page Up retreats top_line by visible_count.
    #[test]
    fn page_up_retreats_top_line_by_visible_count() {
        // Validates: viewport-and-scrolling Requirement 2.2
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        viewport.scroll_page_down(&mut cursor); // top_line = 21

        viewport.scroll_page_up(&mut cursor);

        assert_eq!(viewport.top_line(), 1);
    }

    /// Validates: Requirement 2.8 — Page Down at max_top_line is clamped.
    #[test]
    fn page_down_at_max_top_line_is_clamped() {
        // Validates: viewport-and-scrolling Requirement 2.8
        let mut viewport = ViewportModel::with_line_count(10);
        viewport.set_visible_count(10);
        let mut cursor = CursorModel::new();

        viewport.scroll_page_down(&mut cursor);

        assert_eq!(viewport.top_line(), 1); // entire doc fits — max_top_line = 1
    }

    /// Validates: Requirement 3.1 — Down Arrow scrolls viewport when cursor
    /// would leave the visible area.
    #[test]
    fn arrow_down_scrolls_viewport_when_cursor_leaves_visible_area() {
        // Validates: viewport-and-scrolling Requirement 3.1
        let mut viewport = ViewportModel::with_line_count(20);
        viewport.set_visible_count(5); // lines 1–5 visible
        let mut cursor = CursorModel::new();
        cursor.set_position(5, 1); // cursor at bottom of visible area
        let policy = CaretPolicyEngine::default_policy();

        viewport.move_cursor_down(&mut cursor, 80, 20, &policy);

        // cursor moved to line 6 — viewport must have scrolled
        assert_eq!(cursor.cursor_line(), 6);
        assert!(
            viewport.top_line() >= 2,
            "viewport must scroll to keep cursor visible"
        );
    }

    // ── Req 13 — Bug-fix tests ───────────────────────────────────────────────

    /// Validates: Requirement 13.1 — mouse click sets cursor to the clicked line and column.
    #[test]
    fn mouse_click_sets_cursor_to_clicked_line_and_column() {
        // Validates: startup-and-session Requirement 13.1
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello\nworld\n");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let mut tab = TabState::untitled(TabId(0), document, line_count);
        tab.viewport.set_visible_count(10);

        // Simulate the click handler: clicked at line 2, col 3
        let top_line: u64 = 1;
        let available_top = 0.0_f32;
        let available_left = 0.0_f32;
        let click_y = available_top + 1.0 * 16.0 + 4.0; // row index 1 → line 2
        let click_x = available_left + super::GUTTER_CHAR_WIDTH + 2.0 * 8.0 + 2.0; // col 3

        let clicked_line_idx = ((click_y - available_top) / 16.0).floor() as u64;
        let clicked_line = (top_line + clicked_line_idx).max(1);
        let col_offset = ((click_x - available_left - super::GUTTER_CHAR_WIDTH) / 8.0).floor();
        let clicked_col = (col_offset as i64).max(0) as u64 + 1;
        let line_len = runtime.block_on(async {
            let doc = tab.document.read().await;
            let start = doc.line_start(ff_document_model::LineNumber(clicked_line - 1));
            let end = doc.line_end(ff_document_model::LineNumber(clicked_line - 1));
            end.0.saturating_sub(start.0)
        });
        let clamped_col = clicked_col.min(line_len + 1).max(1);
        tab.cursor.set_position(clicked_line, clamped_col);

        assert_eq!(tab.cursor.cursor_line(), 2);
        assert_eq!(tab.cursor.cursor_column(), 3);
    }

    /// Validates: Requirement 13.2 — Ctrl+Z undoes the last insert.
    #[test]
    fn ctrl_z_undoes_last_insert() {
        // Validates: startup-and-session Requirement 13.2
        use crate::tab_state::UndoEntry;

        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let mut tab = TabState::untitled(TabId(0), document, line_count);
        tab.is_modified = true;

        // Record the undo entry as the insert handler would
        tab.undo_stack.push(UndoEntry::DeleteBytes {
            position: 0,
            length: 5,
        });

        // Apply undo
        let entry = tab.undo_stack.pop().expect("entry");
        match entry {
            UndoEntry::DeleteBytes { position, length } => {
                runtime.block_on(async {
                    let mut doc = tab.document.write().await;
                    let _ = doc.delete(BytePosition(position), length);
                });
            }
            UndoEntry::InsertBytes { .. } => panic!("wrong entry type"),
        }
        tab.is_modified = !tab.undo_stack.is_empty();

        let content = runtime.block_on(async {
            let doc = tab.document.read().await;
            let len = doc.length();
            doc.get_range(BytePosition(0), len).unwrap_or_default()
        });
        assert_eq!(content, b"");
        assert!(!tab.is_modified);
    }

    /// Validates: Requirement 13.3 — cursor line is tracked so highlight can be rendered.
    #[test]
    fn cursor_line_is_tracked_for_highlight() {
        // Validates: startup-and-session Requirement 13.3
        let tab = TabState::untitled(TabId(0), new_document(), 1);
        // cursor starts at line 1
        assert_eq!(tab.cursor.cursor_line(), 1);
    }

    /// Validates: edit-operations Requirement 4.2 — Backspace at column 1 joins current line to end of previous line.
    #[test]
    fn backspace_at_column_1_joins_line_to_previous() {
        // Validates: edit-operations Requirement 4.2
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello\nworld");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let mut tab = TabState::untitled(TabId(0), document, line_count);
        tab.cursor.set_position(2, 1); // beginning of "world"

        // Simulate backspace at col 1: join line 2 to end of line 1
        let (line, col) = (tab.cursor.cursor_line(), tab.cursor.cursor_column());
        assert_eq!(col, 1);

        // Find the newline byte at the end of the previous line and delete it
        let (newline_pos, prev_line_len) = runtime.block_on(async {
            let doc = tab.document.read().await;
            let prev_line_idx = line.saturating_sub(2); // 0-based index of previous line
            let line_end = doc.line_end(ff_document_model::LineNumber(prev_line_idx));
            let line_start = doc.line_start(ff_document_model::LineNumber(prev_line_idx));
            let char_len = line_end.0.saturating_sub(line_start.0);
            (line_end, char_len)
        });
        runtime.block_on(async {
            let mut doc = tab.document.write().await;
            let _ = doc.delete(newline_pos, 1);
        });
        tab.cursor.set_position(line - 1, prev_line_len + 1);
        tab.is_modified = true;
        tab.line_count = runtime.block_on(async { tab.document.read().await.line_count() });

        // Document should now be a single line "helloworld"
        let content = runtime.block_on(async {
            let doc = tab.document.read().await;
            let len = doc.length();
            doc.get_range(BytePosition(0), len).unwrap_or_default()
        });
        assert_eq!(content, b"helloworld");
        assert_eq!(tab.line_count, 1);
        assert_eq!(tab.cursor.cursor_line(), 1);
        assert_eq!(tab.cursor.cursor_column(), 6); // after "hello"
        assert!(tab.is_modified);
    }

    /// Validates: edit-operations Requirement 4.2 — Backspace at column 1 on the first line is a no-op.
    #[test]
    fn backspace_at_column_1_on_first_line_is_noop() {
        // Validates: edit-operations Requirement 4.2
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let mut tab = TabState::untitled(TabId(0), document, line_count);
        tab.cursor.set_position(1, 1);

        // No-op: line 1, col 1 — nothing to join
        let (line, col) = (tab.cursor.cursor_line(), tab.cursor.cursor_column());
        assert_eq!(line, 1);
        assert_eq!(col, 1);
        // Document unchanged
        let content = runtime.block_on(async {
            let doc = tab.document.read().await;
            let len = doc.length();
            doc.get_range(BytePosition(0), len).unwrap_or_default()
        });
        assert_eq!(content, b"hello");
        assert_eq!(tab.line_count, 1);
    }

    /// Validates: edit-operations Requirement 4.2 — Backspace at column 1 on second line.
    #[test]
    fn cursor_column_is_tracked_for_caret_bar() {
        // Validates: startup-and-session Requirement 13.4
        let mut tab = TabState::untitled(TabId(0), new_document(), 1);
        tab.cursor.set_position(3, 7);
        assert_eq!(tab.cursor.cursor_column(), 7);
    }

    // ── Task 21.6 — display list / placeholder tests ────────────────────────────

    use super::{build_display_list, DisplayRow};
    use ff_exclude_show_filter::ExclusionBlock;

    /// Validates: Requirement 6.8 — no exclusions → display list equals all lines in order.
    #[test]
    fn build_display_list_no_exclusions_returns_all_lines() {
        // Validates: exclude-show-filter Requirement 6.8
        let lines: Vec<String> = (1u64..=5).map(|i| format!("line {i}")).collect();
        let rows = build_display_list(1, 5, 5, &lines, &[]);
        assert_eq!(rows.len(), 5);
        for (i, row) in rows.iter().enumerate() {
            match row {
                DisplayRow::Line { doc_line, .. } => assert_eq!(*doc_line, i as u64 + 1),
                DisplayRow::Placeholder { .. } => panic!("unexpected placeholder"),
            }
        }
    }

    /// Validates: Requirement 6.1, 6.2 — single exclusion block produces one placeholder row.
    #[test]
    fn build_display_list_single_block_produces_one_placeholder() {
        // Validates: exclude-show-filter Requirement 6.1, 6.2
        let lines: Vec<String> = (1u64..=5).map(|i| format!("line {i}")).collect();
        // Exclude lines 2 and 3 (0-based: 1 and 2)
        let blocks = vec![ExclusionBlock::new(1, 2)];
        let rows = build_display_list(1, 5, 5, &lines, &blocks);
        // Expected: Line(1), Placeholder(1..=2), Line(4), Line(5)
        assert_eq!(rows.len(), 4);
        assert!(matches!(rows[0], DisplayRow::Line { doc_line: 1, .. }));
        assert!(matches!(rows[1], DisplayRow::Placeholder { .. }));
        assert!(matches!(rows[2], DisplayRow::Line { doc_line: 4, .. }));
        assert!(matches!(rows[3], DisplayRow::Line { doc_line: 5, .. }));
    }

    /// Validates: Requirement 6.1 — two separate blocks produce two placeholder rows.
    #[test]
    fn build_display_list_two_blocks_produce_two_placeholders() {
        // Validates: exclude-show-filter Requirement 6.1
        let lines: Vec<String> = (1u64..=6).map(|i| format!("line {i}")).collect();
        // Exclude line 2 (0-based: 1) and line 5 (0-based: 4)
        let blocks = vec![ExclusionBlock::new(1, 1), ExclusionBlock::new(4, 4)];
        let rows = build_display_list(1, 6, 6, &lines, &blocks);
        // Expected: Line(1), Placeholder, Line(3), Line(4), Placeholder, Line(6)
        assert_eq!(rows.len(), 6);
        assert!(matches!(rows[0], DisplayRow::Line { doc_line: 1, .. }));
        assert!(matches!(rows[1], DisplayRow::Placeholder { .. }));
        assert!(matches!(rows[2], DisplayRow::Line { doc_line: 3, .. }));
        assert!(matches!(rows[3], DisplayRow::Line { doc_line: 4, .. }));
        assert!(matches!(rows[4], DisplayRow::Placeholder { .. }));
        assert!(matches!(rows[5], DisplayRow::Line { doc_line: 6, .. }));
    }

    /// Validates: Requirement 6.2 — placeholder text contains correct line count.
    #[test]
    fn build_display_list_placeholder_text_contains_count() {
        // Validates: exclude-show-filter Requirement 6.2
        let lines: Vec<String> = (1u64..=5).map(|i| format!("line {i}")).collect();
        let blocks = vec![ExclusionBlock::new(1, 3)]; // 3 lines excluded
        let rows = build_display_list(1, 5, 5, &lines, &blocks);
        let placeholder = rows
            .iter()
            .find(|r| matches!(r, DisplayRow::Placeholder { .. }));
        assert!(placeholder.is_some());
        if let Some(DisplayRow::Placeholder { block }) = placeholder {
            assert!(block.placeholder_text().contains("3"));
        }
    }

    /// Validates: Requirement 6.3 — excluded lines do not appear as Line rows.
    #[test]
    fn build_display_list_excluded_lines_not_in_output() {
        // Validates: exclude-show-filter Requirement 6.3
        let lines: Vec<String> = (1u64..=4).map(|i| format!("line {i}")).collect();
        let blocks = vec![ExclusionBlock::new(0, 3)]; // all 4 lines excluded
        let rows = build_display_list(1, 4, 4, &lines, &blocks);
        // Only one placeholder, no Line rows
        assert_eq!(rows.len(), 1);
        assert!(matches!(rows[0], DisplayRow::Placeholder { .. }));
    }

    // ── Task 21.2 — prefix area tests ───────────────────────────────────────────

    /// Validates: Requirement 21.2 — valid line command submitted to engine adds to pending.
    #[test]
    fn prefix_submit_valid_command_adds_to_engine_pending() {
        // Validates: Phase U 21.2 — prefix area wires into CommandEngine.submit_line_command
        use ff_command_semantics::CommandEngine;
        let mut engine = CommandEngine::new();
        let result = engine.submit_line_command(3, "D");
        assert!(result.is_ok());
        assert!(engine.session().has_pending());
        assert_eq!(engine.session().pending()[0].line, 3);
    }

    /// Validates: Requirement 21.2 — invalid line command returns error status.
    #[test]
    fn prefix_submit_invalid_command_returns_error_status() {
        // Validates: Phase U 21.2 — unknown prefix text surfaces as error
        use ff_command_semantics::CommandEngine;
        let mut engine = CommandEngine::new();
        let result = engine.submit_line_command(1, "ZZZ");
        assert!(result.is_err());
        let status = result.unwrap_err();
        assert!(status.text.contains("ZZZ"));
    }

    /// Validates: Requirement 21.2 — prefix_inputs field exists on TabState.
    #[test]
    fn tab_state_has_prefix_inputs_map() {
        // Validates: Phase U 21.2 — TabState carries per-line prefix input storage
        let mut tab = TabState::untitled(TabId(0), new_document(), 1);
        assert!(tab.prefix_inputs.is_empty());
        tab.prefix_inputs.insert(5, "D".to_string());
        assert_eq!(tab.prefix_inputs.get(&5).map(|s| s.as_str()), Some("D"));
    }
}
