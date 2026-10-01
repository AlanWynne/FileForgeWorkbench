//! GUI-independent editor-panel helpers.
//!
//! These are the pure (non-egui, non-`TabState`) helpers extracted from the
//! `ff-desktop` editor panel (CR-NR-098 decomposition Wave 6, Task 22): the
//! display-list builder that interleaves exclusion-block placeholders, the
//! scroll-amount-to-viewport translation, and the selection/cursor geometry
//! helpers. They operate only on `ff-document-model`, `ff-viewport-scrolling`,
//! `ff-scroll-amount`, and `ff-exclude-show-filter` types.
//!
//! The egui render surface and the `TabState`-entangled wiring (keyboard/mouse
//! event handling, undo stack, prefix command submission) remain in the
//! `ff-desktop` `editor_panel` adapter, which re-exports these helpers so the
//! shell render path calls them unchanged.
//!
//! A full re-architecture of the editor onto the editor-aspect crates is a
//! separate gated stream (CR-NR-099), to be run when editor testing begins.

use ff_document_model::LineNumber;
use ff_exclude_show_filter::ExclusionBlock;
use ff_scroll_amount::ScrollAmount;

/// Base font size in points. Zoom offset is added to this.
pub const BASE_FONT_SIZE_PT: f32 = 14.0;
/// Base line height in pixels at zoom offset 0.
pub const BASE_LINE_HEIGHT_PX: f32 = 16.0;
/// Width of the editable prefix area in characters (e.g. "DD    ").
pub const PREFIX_COLS: usize = 6;
/// Pixel width of the prefix area (PREFIX_COLS chars * 8 px + 4 px separator gap).
pub const PREFIX_WIDTH: f32 = (PREFIX_COLS as f32) * 8.0 + 4.0;
/// Legacy alias kept so existing tests that reference GUTTER_CHAR_WIDTH still compile.
pub const GUTTER_CHAR_WIDTH: f32 = PREFIX_WIDTH;

/// A single row in the editor display list.
///
/// Either a normal document line or a placeholder for an exclusion block.
#[derive(Debug, Clone)]
pub enum DisplayRow {
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
/// * `top_line` -- first 1-based document line in the viewport
/// * `end_line` -- last 1-based document line in the viewport (inclusive)
/// * `doc_line_count` -- total lines in the document
/// * `lines` -- pre-fetched line content indexed from `top_line` (index 0 = top_line)
/// * `blocks` -- exclusion blocks for this tab (0-based doc lines)
///
/// Validates: Requirement 6.1, 6.2, 6.3, 6.8 -- placeholder display model
pub fn build_display_list(
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

/// Apply a scroll-amount-aware page scroll to the viewport.
///
/// Translates the active `ScrollAmount` into the correct `ViewportModel` call.
/// `MAX` scrolls to the document bottom; `CSR` scrolls by 1 line.
///
/// Validates: Requirement 14.1, 14.2, 14.3, 14.4, 14.5, 14.6
pub fn scroll_by_amount(
    viewport: &mut ff_viewport_scrolling::ViewportModel,
    cursor: &mut ff_viewport_scrolling::CursorModel,
    amount: &ScrollAmount,
    down: bool,
) {
    let page = viewport.visible_count();
    let lines = match amount {
        ScrollAmount::Page | ScrollAmount::Data => page,
        ScrollAmount::Half => (page / 2).max(1),
        ScrollAmount::Csr => 1,
        ScrollAmount::Max => u64::MAX,
        ScrollAmount::Lines(n) => *n,
    };
    if lines >= page {
        // Use the existing page scroll helpers for full-page and larger amounts.
        if down {
            // For MAX, scroll_to_bottom; otherwise scroll by `lines` pages.
            if lines == u64::MAX {
                viewport.scroll_to_bottom(cursor);
            } else {
                // Scroll by `lines` lines: call scroll_page_down repeatedly
                // would be expensive; instead set top_line directly via scroll_to_line.
                let target = viewport
                    .top_line()
                    .saturating_add(lines)
                    .min(viewport.max_top_line());
                viewport.scroll_to_line(target, cursor);
                cursor.set_position(target, cursor.cursor_column());
            }
        } else if lines == u64::MAX {
            viewport.scroll_to_top(cursor);
        } else {
            let target = viewport.top_line().saturating_sub(lines).max(1);
            viewport.scroll_to_line(target, cursor);
            cursor.set_position(target, cursor.cursor_column());
        }
    } else {
        // Sub-page scroll: use scroll_to_line for precision.
        if down {
            let target = viewport
                .top_line()
                .saturating_add(lines)
                .min(viewport.max_top_line());
            viewport.scroll_to_line(target, cursor);
            cursor.set_position(target, cursor.cursor_column());
        } else {
            let target = viewport.top_line().saturating_sub(lines).max(1);
            viewport.scroll_to_line(target, cursor);
            cursor.set_position(target, cursor.cursor_column());
        }
    }
}

/// Return the character count of a document line (0-based index).
///
/// Used to supply line-length information to cursor movement methods.
pub fn line_char_count(doc: &ff_document_model::Document, line_idx: u64) -> u64 {
    let start = doc.line_start(LineNumber(line_idx));
    let end = doc.line_end(LineNumber(line_idx));
    end.0.saturating_sub(start.0)
}

/// Extract the text covered by a canvas selection from the document.
///
/// `sel` is `(anchor_line, anchor_col, end_line, end_col)` in 1-based coordinates.
/// Lines are joined with `\n`. Returns an empty string when the selection is empty.
///
/// Validates: Requirement 20.1, 20.4 (clipboard-operations)
pub fn extract_selected_text(
    doc: &ff_document_model::Document,
    sel: (u64, u64, u64, u64),
) -> String {
    let (al, ac, el, ec) = sel;
    let (start_line, start_col, end_line, end_col) = if (al, ac) <= (el, ec) {
        (al, ac, el, ec)
    } else {
        (el, ec, al, ac)
    };
    if start_line == end_line && start_col == end_col {
        return String::new();
    }
    let mut result = String::new();
    for ln in start_line..=end_line {
        let line_idx = ln.saturating_sub(1);
        let line_start = doc.line_start(LineNumber(line_idx));
        let line_end = doc.line_end(LineNumber(line_idx));
        let line_len = line_end.0.saturating_sub(line_start.0);
        let col_from = if ln == start_line {
            start_col.saturating_sub(1)
        } else {
            0
        };
        let col_to = if ln == end_line {
            end_col.saturating_sub(1).min(line_len)
        } else {
            line_len
        };
        if col_from < col_to {
            let byte_start = ff_document_model::BytePosition(line_start.0 + col_from);
            let byte_len = col_to - col_from;
            if let Some(bytes) = doc.get_range(byte_start, byte_len) {
                result.push_str(&String::from_utf8_lossy(&bytes));
            }
        }
        if ln < end_line {
            result.push('\n');
        }
    }
    result
}

/// Return the normalised selection endpoints `(start_line, start_col, end_line, end_col)`.
///
/// Ensures start <= end in document order.
///
/// Validates: Requirement 13.4 (caret-and-selection)
pub fn normalise_selection(
    anchor_line: u64,
    anchor_col: u64,
    end_line: u64,
    end_col: u64,
) -> (u64, u64, u64, u64) {
    if (anchor_line, anchor_col) <= (end_line, end_col) {
        (anchor_line, anchor_col, end_line, end_col)
    } else {
        (end_line, end_col, anchor_line, anchor_col)
    }
}

/// Convert a 1-based cursor (line, column) to a `BytePosition` in the document.
///
/// Column 1 maps to the first byte of the line. Columns beyond the line end
/// clamp to the line end position.
pub fn cursor_byte_position(
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

#[cfg(test)]
mod tests {
    use super::{
        build_display_list, cursor_byte_position, extract_selected_text, line_char_count,
        normalise_selection, scroll_by_amount, DisplayRow,
    };
    use ff_document_model::{new_document, BytePosition};
    use ff_exclude_show_filter::ExclusionBlock;
    use ff_scroll_amount::ScrollAmount;
    use ff_viewport_scrolling::{CursorModel, ViewportModel};
    use tokio::runtime::Runtime;

    // === display list / placeholder tests =============================

    /// Validates: Requirement 6.8 -- no exclusions produce display list equal to all lines in order.
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

    /// Validates: Requirement 6.1, 6.2 -- single exclusion block produces one placeholder row.
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

    /// Validates: Requirement 6.1 -- two separate blocks produce two placeholder rows.
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

    /// Validates: Requirement 6.2 -- placeholder text contains correct line count.
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

    /// Validates: Requirement 6.3 -- excluded lines do not appear as Line rows.
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

    // === normalise_selection tests ====================================

    /// Validates: Requirement 13.4 (caret-and-selection) -- normalise_selection orders endpoints.
    #[test]
    fn normalise_selection_orders_start_before_end() {
        // Validates: caret-and-selection Requirement 13.4
        let (sl, sc, el, ec) = normalise_selection(3, 10, 1, 2);
        assert_eq!((sl, sc), (1, 2));
        assert_eq!((el, ec), (3, 10));
    }

    /// Validates: Requirement 13.4 -- normalise_selection is a no-op when already ordered.
    #[test]
    fn normalise_selection_noop_when_already_ordered() {
        // Validates: caret-and-selection Requirement 13.4
        let result = normalise_selection(1, 3, 2, 7);
        assert_eq!(result, (1, 3, 2, 7));
    }

    /// Validates: Requirement 13.4 -- normalise_selection handles same-line reversed columns.
    #[test]
    fn normalise_selection_same_line_reversed_columns() {
        // Validates: caret-and-selection Requirement 13.4
        let (sl, sc, el, ec) = normalise_selection(2, 8, 2, 3);
        assert_eq!((sl, sc, el, ec), (2, 3, 2, 8));
    }

    // === extract_selected_text tests ==================================

    /// Validates: Requirement 20.1 (clipboard-operations) -- extract_selected_text single line.
    #[test]
    fn extract_selected_text_single_line() {
        // Validates: clipboard-operations Requirement 20.1
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello world");
        });
        let text = runtime.block_on(async {
            let doc = document.read().await;
            extract_selected_text(&doc, (1, 1, 1, 6))
        });
        assert_eq!(text, "hello");
    }

    /// Validates: Requirement 20.4 (clipboard-operations) -- extract_selected_text multi-line.
    #[test]
    fn extract_selected_text_multi_line_joins_with_newline() {
        // Validates: clipboard-operations Requirement 20.4
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello\nworld");
        });
        let text = runtime.block_on(async {
            let doc = document.read().await;
            extract_selected_text(&doc, (1, 1, 2, 6))
        });
        assert_eq!(text, "hello\nworld");
    }

    /// Validates: Requirement 20.3 (clipboard-operations) -- empty selection returns empty string.
    #[test]
    fn extract_selected_text_empty_selection_returns_empty() {
        // Validates: clipboard-operations Requirement 20.3
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello");
        });
        let text = runtime.block_on(async {
            let doc = document.read().await;
            extract_selected_text(&doc, (1, 3, 1, 3))
        });
        assert_eq!(text, "");
    }

    /// Validates: Requirement 20.1 -- extract_selected_text handles reversed anchor/end.
    #[test]
    fn extract_selected_text_reversed_anchor_normalises() {
        // Validates: clipboard-operations Requirement 20.1
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"abcde");
        });
        let text = runtime.block_on(async {
            let doc = document.read().await;
            // anchor after end -- should still extract "bcd"
            extract_selected_text(&doc, (1, 5, 1, 2))
        });
        assert_eq!(text, "bcd");
    }

    // === cursor_byte_position / line_char_count tests =================

    /// Validates: Requirement 13.4 -- cursor_byte_position maps column 1 to the line start.
    #[test]
    fn cursor_byte_position_column_one_is_line_start() {
        // Validates: caret-and-selection Requirement 13.4
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello\nworld");
        });
        let (first, second) = runtime.block_on(async {
            let doc = document.read().await;
            (
                cursor_byte_position(&doc, 1, 1),
                cursor_byte_position(&doc, 2, 1),
            )
        });
        assert_eq!(first, BytePosition(0));
        assert_eq!(second, BytePosition(6)); // after "hello\n"
    }

    /// Validates: Requirement 13.4 -- line_char_count returns the line length in bytes.
    #[test]
    fn line_char_count_returns_line_length() {
        // Validates: caret-and-selection Requirement 13.4
        let runtime = Runtime::new().expect("runtime");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello\nworld!");
        });
        let (l0, l1) = runtime.block_on(async {
            let doc = document.read().await;
            (line_char_count(&doc, 0), line_char_count(&doc, 1))
        });
        assert_eq!(l0, 5); // "hello"
        assert_eq!(l1, 6); // "world!"
    }

    // === scroll_by_amount tests (Req 14.1-14.8) =======================

    /// Validates: Requirement 14.1 -- PAGE scrolls by visible_count lines.
    #[test]
    fn scroll_by_amount_page_down_advances_by_visible_count() {
        // Validates: viewport-and-scrolling Requirement 14.1
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        scroll_by_amount(&mut viewport, &mut cursor, &ScrollAmount::Page, true);
        assert_eq!(viewport.top_line(), 21);
    }

    /// Validates: Requirement 14.1 -- PAGE scrolls up by visible_count lines.
    #[test]
    fn scroll_by_amount_page_up_retreats_by_visible_count() {
        // Validates: viewport-and-scrolling Requirement 14.1
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        scroll_by_amount(&mut viewport, &mut cursor, &ScrollAmount::Page, true);
        scroll_by_amount(&mut viewport, &mut cursor, &ScrollAmount::Page, false);
        assert_eq!(viewport.top_line(), 1);
    }

    /// Validates: Requirement 14.2 -- HALF scrolls by half the visible_count.
    #[test]
    fn scroll_by_amount_half_down_advances_by_half_page() {
        // Validates: viewport-and-scrolling Requirement 14.2
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        scroll_by_amount(&mut viewport, &mut cursor, &ScrollAmount::Half, true);
        assert_eq!(viewport.top_line(), 11);
    }

    /// Validates: Requirement 14.2 -- HALF with odd visible_count rounds down, min 1.
    #[test]
    fn scroll_by_amount_half_odd_visible_count_rounds_down() {
        // Validates: viewport-and-scrolling Requirement 14.2
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(5);
        let mut cursor = CursorModel::new();
        scroll_by_amount(&mut viewport, &mut cursor, &ScrollAmount::Half, true);
        assert_eq!(viewport.top_line(), 3); // 5/2 = 2
    }

    /// Validates: Requirement 14.3 -- CSR scrolls by exactly 1 line.
    #[test]
    fn scroll_by_amount_csr_down_advances_by_one_line() {
        // Validates: viewport-and-scrolling Requirement 14.3
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        scroll_by_amount(&mut viewport, &mut cursor, &ScrollAmount::Csr, true);
        assert_eq!(viewport.top_line(), 2);
    }

    /// Validates: Requirement 14.4 -- MAX scrolls to the document bottom.
    #[test]
    fn scroll_by_amount_max_down_scrolls_to_bottom() {
        // Validates: viewport-and-scrolling Requirement 14.4
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        scroll_by_amount(&mut viewport, &mut cursor, &ScrollAmount::Max, true);
        assert_eq!(viewport.top_line(), viewport.max_top_line());
    }

    /// Validates: Requirement 14.4 -- MAX scrolls to the document top.
    #[test]
    fn scroll_by_amount_max_up_scrolls_to_top() {
        // Validates: viewport-and-scrolling Requirement 14.4
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        scroll_by_amount(&mut viewport, &mut cursor, &ScrollAmount::Max, true);
        scroll_by_amount(&mut viewport, &mut cursor, &ScrollAmount::Max, false);
        assert_eq!(viewport.top_line(), 1);
    }

    /// Validates: Requirement 14.5 -- DATA behaves identically to PAGE.
    #[test]
    fn scroll_by_amount_data_behaves_like_page() {
        // Validates: viewport-and-scrolling Requirement 14.5
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        scroll_by_amount(&mut viewport, &mut cursor, &ScrollAmount::Data, true);
        assert_eq!(viewport.top_line(), 21);
    }

    /// Validates: Requirement 14.6 -- Lines(n) scrolls by exactly n lines.
    #[test]
    fn scroll_by_amount_lines_n_advances_by_n() {
        // Validates: viewport-and-scrolling Requirement 14.6
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        scroll_by_amount(&mut viewport, &mut cursor, &ScrollAmount::Lines(7), true);
        assert_eq!(viewport.top_line(), 8);
    }
}
