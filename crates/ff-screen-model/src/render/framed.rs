//! Plain-text renderer for a COMMON-FRAME model (CR-NR-115).
//!
//! A model that carries `header_lines` (the window title bar, menu bar and tab
//! bar composed by the shell's common-frame helper) is rendered as ONE box made
//! of horizontal RULED BANDS, top to bottom:
//!
//! 1. each `header_lines` entry as a band (title bar, menu bar, tab bar)
//! 2. the workspace title, CENTERED, as a single-cell band
//! 3. the `Command ===>` line (underscores fill to the right border)
//! 4. the frame-owned boundary RULE between the Command line and the content
//!    (merged with the content's own grid top rule when it begins with one, and
//!    likewise at the content bottom: never two consecutive rules)
//! 5. the CONTENT (body lines, fields, table grids, buttons)
//! 6. a rule DIRECTLY after the last content row (no blank row)
//! 7. the message band, only when a message is present
//! 8. the status bar band
//! 9. the function-key band (`footer_lines`), the LAST band, only when present,
//!    drawn as ONE cell (its ` | ` separators are plain text, no junctions)
//!
//! BAND CELLS. A band line whose cells are joined by ` | ` is drawn as separate
//! cells divided by the vertical glyph. Where a divider meets a rule above or
//! below, the rule carries the matching junction glyph (tee-down, tee-up, or
//! cross). In the LAST cell of a band the [`RIGHT_ALIGN`] marker splits the
//! cell text from a tail that is right-aligned against the border (the window
//! controls of the title bar). No public model type changes: the cells are
//! carried inside the existing `header_lines` / `status_bar` / `footer_lines`
//! strings.
//!
//! The legacy layout in `text::render_plain` is kept for models without
//! `header_lines`.
//!
//! Validates: screen-snapshot-scrm Requirement 30.1-30.11, 30.15, 30.16,
//! 30.18-30.24, 30.29, 30.30; workspace-framework Requirement 7.1-7.10,
//! 7.30-7.32.

use crate::model::ScreenModel;
use crate::render::text::{push_content, render_table_grid, Glyphs};
use crate::render::RenderOptions;

/// Marker inside the LAST cell of a band: the text after it is right-aligned
/// against the right border (the title bar's `_ [] X` window controls).
pub const RIGHT_ALIGN: char = '\t';

/// The text between two cells of a band line (and the visible divider's pads).
const CELL_SEPARATOR: &str = " | ";
/// Width of a drawn divider: pad, vertical glyph, pad.
const DIVIDER_WIDTH: usize = 3;
/// Smallest gap between a band's left text and its right-aligned tail.
const RIGHT_ALIGN_MIN_GAP: usize = 4;
/// The `Command ===> ` prompt that precedes the input field.
const COMMAND_PROMPT: &str = "Command ===> ";
/// Fewest underscores the Command input shows (sets the natural width only).
const COMMAND_MIN_FILL: usize = 4;

/// One logical row of the framed box before widths are known.
enum Row {
    /// A left-aligned content row.
    Text(String),
    /// A row centered within the inner width.
    Centered(String),
    /// The Command line: the typed text; underscores fill to the border.
    Command(String),
    /// A band of cells divided by vertical glyphs.
    Band(Vec<String>),
    /// A horizontal rule between bands.
    Rule,
    /// A frame rule MERGED with the Content_Area's own grid TOP rule: carries
    /// the grid's column junctions (line columns) as tee-down (Req 7.32).
    GridTop(Vec<usize>),
    /// A frame rule MERGED with the Content_Area's own grid BOTTOM rule: carries
    /// the grid's column junctions (line columns) as tee-up (Req 7.32).
    GridBottom(Vec<usize>),
}

/// Line column at which band/content text starts: `vertical`, then a pad space.
const CONTENT_START: usize = 2;

/// Whether `row` is any kind of horizontal rule.
fn is_rule_row(row: &Row) -> bool {
    matches!(row, Row::Rule | Row::GridTop(_) | Row::GridBottom(_))
}

/// The junction line columns of a content row that is a GRID rule (made only of
/// the horizontal glyph and at least one tee/cross junction), else `None`.
fn grid_rule_cuts(text: &str, g: &Glyphs) -> Option<Vec<usize>> {
    let is_junction = |c: char| c == g.tee_down || c == g.tee_up || c == g.cross;
    if text.is_empty() || !text.chars().all(|c| c == g.horizontal || is_junction(c)) {
        return None;
    }
    // A frame-ruled grid rule has NO corner of its own: the frame border is its
    // edge, so it starts and ends with the horizontal glyph. A self-boxed grid
    // (corner at either end, `+` in the ASCII fallback) is ordinary content.
    if !text.starts_with(g.horizontal) || !text.ends_with(g.horizontal) {
        return None;
    }
    let cuts: Vec<usize> = text
        .chars()
        .enumerate()
        .filter(|(_, c)| is_junction(*c))
        .map(|(i, _)| i + CONTENT_START)
        .collect();
    (!cuts.is_empty()).then_some(cuts)
}

/// Merge a Content_Area that BEGINS with a grid top rule into the frame rule
/// pushed just before it (`start` is the index of the first content row).
fn merge_top_rule(rows: &mut Vec<Row>, start: usize, g: &Glyphs) {
    let Some(Row::Text(first)) = rows.get(start) else {
        return;
    };
    let Some(cuts) = grid_rule_cuts(first, g) else {
        return;
    };
    if start > 0 && matches!(rows.get(start - 1), Some(Row::Rule)) {
        rows[start - 1] = Row::GridTop(cuts);
        rows.remove(start);
    }
}

/// Split a band line into its cells.
fn split_cells(line: &str) -> Vec<String> {
    line.split(CELL_SEPARATOR).map(str::to_string).collect()
}

/// Collect the rows of the framed layout, in display order.
fn collect_rows(model: &ScreenModel, g: &Glyphs) -> Vec<Row> {
    let mut rows = Vec::new();
    for h in &model.header_lines {
        rows.push(Row::Band(split_cells(h)));
        rows.push(Row::Rule);
    }
    rows.push(Row::Centered(model.title.clone()));
    rows.push(Row::Rule);
    if !model.command_line.is_empty() {
        // The renderer owns the underscore fill, so strip any the caller added.
        rows.push(Row::Command(
            model.command_line.trim_end_matches('_').to_string(),
        ));
    }
    // The frame-owned boundary rule between the shell area and the Content_Area
    // (Req 7.30), never doubled against a rule that is already last.
    if !rows.last().is_some_and(is_rule_row) {
        rows.push(Row::Rule);
    }
    let content_start = rows.len();
    rows.extend(model.body_lines.iter().cloned().map(Row::Text));
    if !model.fields.is_empty() {
        rows.push(Row::Text(String::new()));
        for f in &model.fields {
            rows.push(Row::Text(if f.label.is_empty() {
                f.value.clone()
            } else {
                format!("{} . . . : {}", f.label, f.value)
            }));
        }
    }
    for table in &model.tables {
        rows.push(Row::Text(String::new()));
        if let Some(cap) = &table.caption {
            rows.push(Row::Text(cap.clone()));
        }
        let mut grid = String::new();
        render_table_grid(&mut grid, table, g);
        rows.extend(grid.lines().map(|l| Row::Text(l.to_string())));
    }
    if !model.buttons.is_empty() {
        let line = model.buttons.iter().map(|b| format!("[{b}]"));
        rows.push(Row::Text(String::new()));
        rows.push(Row::Text(line.collect::<Vec<_>>().join(" ")));
    }
    merge_top_rule(&mut rows, content_start, g);
    push_bottom_bands(model, &mut rows, g);
    rows
}

/// Bottom chrome: a rule DIRECTLY after the content, then the message band
/// (when present), the status band, and the key-line band (when present).
/// A Content_Area ending in a grid bottom rule MERGES with that rule (Req 7.32).
fn push_bottom_bands(model: &ScreenModel, rows: &mut Vec<Row>, g: &Glyphs) {
    let has_messages = !model.messages.is_empty();
    let has_status = !model.status_bar.text.is_empty();
    let has_footer = !model.footer_lines.is_empty();
    if !(has_messages || has_status || has_footer) {
        return;
    }
    let grid_cuts = match rows.last() {
        Some(Row::Text(t)) => grid_rule_cuts(t, g),
        _ => None,
    };
    if grid_cuts.is_some() {
        rows.pop();
    }
    // Never two consecutive rules at the content bottom.
    if !rows.last().is_some_and(is_rule_row) {
        rows.push(grid_cuts.map_or(Row::Rule, Row::GridBottom));
    }
    rows.extend(model.messages.iter().cloned().map(Row::Text));
    if has_messages && (has_status || has_footer) {
        rows.push(Row::Rule);
    }
    if has_status {
        rows.push(Row::Band(split_cells(&model.status_bar.text)));
        if has_footer {
            rows.push(Row::Rule);
        }
    }
    // The key line is ONE cell: its ` | ` separators stay plain text, so the
    // rules around it carry no key-line junctions (Req 30.25).
    rows.extend(model.footer_lines.iter().cloned().map(Row::Text));
}

/// The cell text with any right-align marker flattened to a space.
fn flat(cell: &str) -> String {
    cell.replace(RIGHT_ALIGN, " ")
}

/// Natural width of a band: its cells, the dividers, and (for a right-aligned
/// tail) the minimum gap.
fn band_width(cells: &[String]) -> usize {
    let dividers = DIVIDER_WIDTH * cells.len().saturating_sub(1);
    let last = cells.len().saturating_sub(1);
    let text: usize = cells
        .iter()
        .enumerate()
        .map(|(i, c)| match c.split_once(RIGHT_ALIGN) {
            Some((left, right)) if i == last => {
                left.chars().count() + RIGHT_ALIGN_MIN_GAP + right.chars().count()
            }
            _ => flat(c).chars().count(),
        })
        .sum();
    dividers + text
}

/// Lay a band out as one content string plus the content offsets of its
/// dividers. The last cell's right-aligned tail ends at `inner_width`.
fn compose_band(cells: &[String], vertical: char, inner_width: usize) -> (String, Vec<usize>) {
    let mut text = String::new();
    let mut cuts = Vec::new();
    let mut len = 0usize;
    let last = cells.len().saturating_sub(1);
    for (i, cell) in cells.iter().enumerate() {
        if i > 0 {
            text.push(' ');
            cuts.push(len + 1);
            text.push(vertical);
            text.push(' ');
            len += DIVIDER_WIDTH;
        }
        match cell.split_once(RIGHT_ALIGN) {
            Some((left, right)) if i == last => {
                let tail = right.chars().count();
                let used = len + left.chars().count() + tail;
                text.push_str(left);
                text.push_str(&" ".repeat(inner_width.saturating_sub(used).max(1)));
                text.push_str(right);
            }
            _ => {
                let flattened = flat(cell);
                len += flattened.chars().count();
                text.push_str(&flattened);
            }
        }
    }
    (text, cuts)
}

/// The width a row needs, or 0 for a rule.
fn natural_width(row: &Row) -> usize {
    match row {
        Row::Text(t) | Row::Centered(t) => t.chars().count(),
        Row::Command(t) => COMMAND_PROMPT.len() + t.chars().count() + COMMAND_MIN_FILL,
        Row::Band(cells) => band_width(cells),
        Row::Rule | Row::GridTop(_) | Row::GridBottom(_) => 0,
    }
}

/// Lay one non-rule row out as a full line plus the LINE columns of its cell
/// dividers (empty for a plain row). Returns `None` for a rule.
fn lay_row(row: &Row, g: &Glyphs, inner_width: usize) -> Option<(String, Vec<usize>)> {
    let mut out = String::new();
    let mut cuts = Vec::new();
    match row {
        Row::Rule | Row::GridTop(_) | Row::GridBottom(_) => return None,
        Row::Text(t) => push_content(&mut out, g.vertical, t, inner_width),
        Row::Centered(t) => {
            let left = inner_width.saturating_sub(t.chars().count()) / 2;
            let shifted = format!("{}{t}", " ".repeat(left));
            push_content(&mut out, g.vertical, &shifted, inner_width);
        }
        Row::Command(typed) => {
            let used = COMMAND_PROMPT.len() + typed.chars().count();
            let fill = "_".repeat(inner_width.saturating_sub(used));
            let line = format!("{COMMAND_PROMPT}{typed}{fill}");
            push_content(&mut out, g.vertical, &line, inner_width);
        }
        Row::Band(cells) => {
            let (text, offsets) = compose_band(cells, g.vertical, inner_width);
            cuts = offsets.iter().map(|o| o + CONTENT_START).collect();
            push_content(&mut out, g.vertical, &text, inner_width);
        }
    }
    Some((out, cuts))
}

/// Push one rule line with junction glyphs where a divider meets it from
/// `above` and/or `below` (line columns).
fn push_rule(
    out: &mut String,
    ends: (char, char),
    g: &Glyphs,
    junctions: (&[usize], &[usize]),
    inner_width: usize,
) {
    let (above, below) = junctions;
    out.push(ends.0);
    for col in 1..inner_width + 3 {
        out.push(match (above.contains(&col), below.contains(&col)) {
            (true, true) => g.cross,
            (true, false) => g.tee_up,
            (false, true) => g.tee_down,
            (false, false) => g.horizontal,
        });
    }
    out.push(ends.1);
    out.push('\n');
}

/// Render `model` as ONE framed box with every common-frame element inside it.
pub(super) fn render_framed(model: &ScreenModel, options: RenderOptions) -> String {
    let g = Glyphs::for_options(options);
    let rows = collect_rows(model, &g);
    let inner_width = rows.iter().map(natural_width).max().unwrap_or(0).max(20);
    let laid: Vec<Option<(String, Vec<usize>)>> =
        rows.iter().map(|r| lay_row(r, &g, inner_width)).collect();
    let cuts_at = |i: Option<usize>| -> &[usize] {
        match i.and_then(|i| laid.get(i)) {
            Some(Some((_, cuts))) => cuts,
            _ => &[],
        }
    };

    let mut out = String::new();
    let top_ends = (g.top_left, g.top_right);
    push_rule(&mut out, top_ends, &g, (&[], cuts_at(Some(0))), inner_width);
    for (i, line) in laid.iter().enumerate() {
        match line {
            Some((text, _)) => out.push_str(text),
            None => {
                // Junctions from the neighbouring bands, plus a merged grid
                // rule's own column junctions on the side it faces.
                let (mut above, mut below) = (
                    cuts_at(i.checked_sub(1)).to_vec(),
                    cuts_at(Some(i + 1)).to_vec(),
                );
                match &rows[i] {
                    Row::GridTop(c) => below.extend(c),
                    Row::GridBottom(c) => above.extend(c),
                    _ => {}
                }
                let ends = (g.tee_left, g.tee_right);
                push_rule(&mut out, ends, &g, (&above, &below), inner_width);
            }
        }
    }
    let bottom_ends = (g.bottom_left, g.bottom_right);
    let above = cuts_at(laid.len().checked_sub(1));
    push_rule(&mut out, bottom_ends, &g, (above, &[]), inner_width);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn framed_model() -> ScreenModel {
        let mut m = ScreenModel::new("Configuration");
        m.header_lines = vec!["WINDOW".into(), "MENU".into(), "TABS".into()];
        m.command_line = "____".into();
        m.body_lines = vec!["content line".into()];
        m.messages = vec!["a message".into()];
        m.status_bar.text = "Last: 00:00".into();
        m.footer_lines = vec!["F1 Help".into()];
        m
    }

    // Validates: screen-snapshot-scrm Requirement 30.1-30.11 -- the framed
    // renderer emits header chrome, centered title, Command line, content,
    // message, status and key line in order, all inside ONE box.
    #[test]
    fn framed_layout_orders_rows_and_keeps_everything_inside_the_box() {
        let out = render_framed(&framed_model(), RenderOptions { unicode_box: false });
        let rows: Vec<&str> = out.lines().collect();
        let at = |needle: &str| rows.iter().position(|r| r.contains(needle)).expect(needle);
        assert!(at("WINDOW") < at("MENU"));
        assert!(at("MENU") < at("TABS"));
        assert!(at("TABS") < at("Configuration"));
        assert!(at("Configuration") < at("Command ===>"));
        assert!(at("Command ===>") < at("content line"));
        assert!(at("content line") < at("a message"));
        assert!(at("a message") < at("Last: 00:00"));
        assert!(at("Last: 00:00") < at("F1 Help"));
        assert!(rows
            .iter()
            .all(|r| r.starts_with('+') || r.starts_with('|')));
        assert!(rows.last().expect("rows").starts_with('+'), "nothing below");
        assert_eq!(out.matches("Configuration").count(), 1, "title once");
    }

    // Validates: screen-snapshot-scrm Requirement 30.9, 30.11 -- no message
    // rows and no key line when none are present.
    #[test]
    fn framed_layout_omits_message_and_key_rows_when_absent() {
        let mut m = framed_model();
        m.messages.clear();
        m.footer_lines.clear();
        let out = render_framed(&m, RenderOptions { unicode_box: false });
        assert!(!out.contains("a message") && !out.contains("F1 Help"));
        assert!(out.contains("Last: 00:00"));
    }
}
