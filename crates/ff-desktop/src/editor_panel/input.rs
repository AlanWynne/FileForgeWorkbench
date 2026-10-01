//! Keyboard and mouse-wheel input handling for the editor panel adapter.
//!
//! These functions mutate the active `TabState` (document, cursor, viewport,
//! undo stack) in response to egui input events. They are the `TabState`-
//! entangled part of the editor that stays in `ff-desktop`; the pure geometry
//! helpers they call live in the `ff-editor-panel` crate.

use eframe::egui;
use ff_viewport_scrolling::CaretPolicyEngine;
use tokio::runtime::Runtime;

use super::{cursor_byte_position, line_char_count, scroll_by_amount};
use crate::tab_state::{TabState, UndoEntry};

/// Handle text entry and editing/navigation keys for the current frame.
///
/// Covers typed characters, Backspace (including col-1 line join), Enter
/// (line split), arrow navigation, Page Up/Down (scroll-amount aware), Ctrl+Z
/// undo, and mouse-wheel scrolling. Behaviour is identical to the original
/// inline `render` body; only the extraction into a helper is new.
pub(super) fn handle_edit_and_nav(
    ui: &mut egui::Ui,
    tab: &mut TabState,
    runtime: &Runtime,
    scroll_amount: &crate::scroll_amount::ScrollAmount,
    available: egui::Rect,
) {
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
            // Req 4.2 -- Backspace at col 1: delete the newline at the end of the previous line,
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
                // Validates: Requirement 14.1-14.6 -- scroll amount controls page size
                scroll_by_amount(&mut tab.viewport, &mut tab.cursor, scroll_amount, true);
            }
            egui::Key::PageUp => {
                // Validates: Requirement 14.1-14.6 -- scroll amount controls page size
                scroll_by_amount(&mut tab.viewport, &mut tab.cursor, scroll_amount, false);
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

    // ── Mouse wheel -- document scroll only (Ctrl+Scroll handled at shell level) ──
    // Validates: Requirement 3.3 (view-zoom) -- Ctrl NOT held: normal document scroll.
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
}
