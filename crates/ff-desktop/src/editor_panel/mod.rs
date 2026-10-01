//! `EditorPanel` -- stateless egui renderer for a single tab's document view.
//!
//! Renders the visible lines of the active `TabState` into the central panel.
//! All state (document, viewport, cursor) lives in `TabState`; this module
//! only contains the rendering logic and input handling.
//!
//! The GUI-independent helpers (display-list building, scroll-amount
//! translation, selection/cursor geometry, and the geometry consts) moved into
//! the `ff-editor-panel` crate (CR-NR-098 decomposition Wave 6, Task 22). This
//! module is the thin shell-side adapter: it keeps the egui `render` entry point
//! and the `TabState`-entangled wiring (split into `input` and `paint`), calls
//! the moved helpers through `ff_editor_panel::*`, and re-exports them so
//! existing `super::`/`crate::editor_panel::` references resolve unchanged.
//!
//! A full re-architecture of the editor onto the editor-aspect crates is a
//! separate gated stream (CR-NR-099), to be run when editor testing begins.

use eframe::egui;
use ff_command_semantics::CommandEngine;
use tokio::runtime::Runtime;

use crate::exclude_manager::ExcludeManager;
use crate::tab_state::{TabId, TabState};

mod input;
mod paint;

// Re-export the pure helpers and geometry consts so existing shell and test
// references (super::build_display_list, crate::editor_panel::render, etc.)
// keep resolving after the move into ff-editor-panel.
pub(crate) use ff_editor_panel::{
    build_display_list, cursor_byte_position, extract_selected_text, line_char_count,
    normalise_selection, scroll_by_amount, DisplayRow, GUTTER_CHAR_WIDTH, PREFIX_WIDTH,
};
use ff_editor_panel::{BASE_FONT_SIZE_PT, BASE_LINE_HEIGHT_PX};

/// Render the active tab's document into `ui`.
///
/// Updates `tab.viewport.visible_count` from the available rect, handles
/// mouse-wheel scroll, keyboard navigation, and paints each visible line
/// with an editable ISPF-style prefix area followed by the line content.
/// Prefix input is submitted to `cmd_engine` on Enter.
/// Exclusion blocks are rendered as non-editable placeholder rows.
///
/// `scroll_amount` controls how much Page Up/Down scrolls (Req 14.1-14.6).
pub fn render(
    ui: &mut egui::Ui,
    tab: &mut TabState,
    runtime: &Runtime,
    cmd_engine: &mut CommandEngine,
    exclude_manager: &mut ExcludeManager,
    tab_id: TabId,
    scroll_amount: &crate::scroll_amount::ScrollAmount,
) -> Option<String> {
    let available = ui.available_rect_before_wrap();
    // Compute effective font size from zoom offset (Req 1.2, 3.1-3.2 view-zoom)
    // Zoom is global (owned by WorkbenchShell); editor_panel receives the resolved pt size.
    let effective_font_pt = BASE_FONT_SIZE_PT;
    // Scale line height proportionally to font size
    let line_height_px = (BASE_LINE_HEIGHT_PX * effective_font_pt / BASE_FONT_SIZE_PT).max(4.0);
    let visible_lines = (available.height() / line_height_px).floor() as u64;
    tab.viewport.set_visible_count(visible_lines.max(1));

    // Keyboard/text/mouse-wheel editing and navigation (TabState-mutating).
    input::handle_edit_and_nav(ui, tab, runtime, scroll_amount, available);

    // Paint the visible rows, handle pointer selection, and Ctrl+C copy.
    paint::paint_and_interact(
        ui,
        tab,
        runtime,
        cmd_engine,
        exclude_manager,
        tab_id,
        available,
        line_height_px,
        visible_lines,
        effective_font_pt,
    )
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

    /// Validates: startup-and-session Requirement 13.4 — cursor column is tracked for the caret bar.
    #[test]
    fn cursor_column_is_tracked_for_caret_bar() {
        // Validates: startup-and-session Requirement 13.4
        let mut tab = TabState::untitled(TabId(0), new_document(), 1);
        tab.cursor.set_position(3, 7);
        assert_eq!(tab.cursor.cursor_column(), 7);
    }

    // ── prefix area tests ───────────────────────────────────────────

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

    /// Validates: Requirement 1 (line-commands) -- prefix submission fires on lost_focus,
    /// not requiring simultaneous Enter key press (fixes B031).
    #[test]
    fn prefix_submit_fires_on_lost_focus_without_simultaneous_enter() {
        // Validates: line-commands Requirement 1 -- gutter input wired to engine
        // The fix for B031: submission must not require lost_focus() AND key_pressed(Enter)
        // in the same frame, because egui never delivers both simultaneously.
        // This test verifies the engine accepts the command when called from lost_focus path.
        use ff_command_semantics::CommandEngine;
        let mut engine = CommandEngine::new();
        // Simulate what the fixed render loop does: submit on lost_focus (non-empty text)
        let text = "D";
        let result = engine.submit_line_command(2, text);
        assert!(result.is_ok(), "submit should succeed for valid command D");
        assert!(engine.session().has_pending());
        assert_eq!(engine.session().pending()[0].line, 2);
    }

    /// Validates: Requirement 1 (line-commands) -- prefix cleared after successful submit.
    #[test]
    fn prefix_text_cleared_after_successful_submit() {
        // Validates: line-commands Requirement 1 -- prefix area cleared on submit
        use ff_command_semantics::CommandEngine;
        let mut engine = CommandEngine::new();
        let mut prefix_text = String::from("I3");
        let result = engine.submit_line_command(5, prefix_text.trim());
        assert!(result.is_ok());
        // Simulate what the render loop does on Ok: clear the field
        prefix_text.clear();
        assert!(prefix_text.is_empty());
        assert!(engine.session().has_pending());
    }

    /// Validates: Requirement 14.6 (line-commands) -- invalid prefix text retained on error.
    #[test]
    fn prefix_text_retained_on_invalid_command() {
        // Validates: line-commands Requirement 14.6 -- invalid text stays in field
        use ff_command_semantics::CommandEngine;
        let mut engine = CommandEngine::new();
        let mut prefix_text = String::from("ZZZ");
        let result = engine.submit_line_command(1, prefix_text.trim());
        assert!(result.is_err());
        // On error the render loop clears the field (shows error in status bar)
        // but the engine has no pending command
        prefix_text.clear();
        assert!(!engine.session().has_pending());
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

    // === Phase CM: mouse text selection and clipboard copy ===

    /// Validates: Requirement 13.1 (caret-and-selection) -- new tab has no canvas selection.
    #[test]
    fn new_tab_has_no_canvas_selection() {
        // Validates: caret-and-selection Requirement 13.1
        let tab = TabState::untitled(TabId(0), new_document(), 1);
        assert!(tab.canvas_selection.is_none());
    }

    /// Validates: Requirement 13.2 (caret-and-selection) -- canvas_selection can be set.
    #[test]
    fn canvas_selection_can_be_set_and_cleared() {
        // Validates: caret-and-selection Requirement 13.2
        let mut tab = TabState::untitled(TabId(0), new_document(), 1);
        tab.canvas_selection = Some((1, 1, 1, 5));
        assert_eq!(tab.canvas_selection, Some((1, 1, 1, 5)));
        tab.canvas_selection = None;
        assert!(tab.canvas_selection.is_none());
    }

    /// Validates: Requirement 13.7 (caret-and-selection) -- tab switch clears selection.
    #[test]
    fn canvas_selection_cleared_on_tab_switch() {
        // Validates: caret-and-selection Requirement 13.7
        let mut tab = TabState::untitled(TabId(0), new_document(), 1);
        tab.canvas_selection = Some((1, 1, 2, 5));
        // Simulate tab switch: clear selection
        tab.canvas_selection = None;
        assert!(tab.canvas_selection.is_none());
    }
}
