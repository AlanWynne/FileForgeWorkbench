//! Plain-text renderer with Unicode box-drawing framing and an ASCII fallback.
//!
//! Validates: screen-snapshot-scrm Requirement 4.2, 5.1, 5.2, 5.4.

use crate::model::ScreenModel;
use crate::render::RenderOptions;

/// Box-drawing glyphs, either Unicode or ASCII fallback.
pub(super) struct Glyphs {
    pub(super) top_left: char,
    pub(super) top_right: char,
    pub(super) bottom_left: char,
    pub(super) bottom_right: char,
    pub(super) horizontal: char,
    pub(super) vertical: char,
    pub(super) tee_left: char,
    pub(super) tee_right: char,
    /// Junction where a cell divider continues DOWN into the row below only
    /// (ASCII fallback: `+`). Used on a rule directly above a cell band.
    pub(super) tee_down: char,
    /// Junction where a cell divider continues UP into the row above only
    /// (ASCII fallback: `+`). Used on a rule directly below a cell band.
    pub(super) tee_up: char,
    /// Junction where a cell divider continues on BOTH sides of the rule
    /// (ASCII fallback: `+`).
    pub(super) cross: char,
}

impl Glyphs {
    pub(super) fn for_options(options: RenderOptions) -> Self {
        if options.unicode_box {
            Self {
                top_left: '\u{250C}',
                top_right: '\u{2510}',
                bottom_left: '\u{2514}',
                bottom_right: '\u{2518}',
                horizontal: '\u{2500}',
                vertical: '\u{2502}',
                tee_left: '\u{251C}',
                tee_right: '\u{2524}',
                tee_down: '\u{252C}',
                tee_up: '\u{2534}',
                cross: '\u{253C}',
            }
        } else {
            Self {
                top_left: '+',
                top_right: '+',
                bottom_left: '+',
                bottom_right: '+',
                horizontal: '-',
                vertical: '|',
                tee_left: '+',
                tee_right: '+',
                tee_down: '+',
                tee_up: '+',
                cross: '+',
            }
        }
    }
}

/// Render `model` as plain framed text.
///
/// A model that carries common-frame `header_lines` (composed by the shell's
/// `apply_common_frame`, CR-NR-115) is rendered by the dedicated
/// [`super::framed::render_framed`] layout, which places every element INSIDE
/// one box in the required top-to-bottom order (screen-snapshot-scrm
/// Requirement 30.1-30.11). A model without `header_lines` keeps the legacy
/// layout below unchanged.
pub fn render_plain(model: &ScreenModel, options: RenderOptions) -> String {
    if !model.header_lines.is_empty() {
        return super::framed::render_framed(model, options);
    }
    let g = Glyphs::for_options(options);
    // Inner width: fit the widest content line, with sane bounds.
    let mut lines: Vec<String> = Vec::new();
    lines.push(model.title.clone());

    // Pre-composed verbatim body lines (the layout-owning menu/POM path) sit
    // DIRECTLY under the title border with NO leading blank and NO decoration:
    // the Context has already folded its command line / two-column body / PF
    // footer into these exact lines (screen-snapshot-scrm Req 3.8-3.13).
    for b in &model.body_lines {
        lines.push(b.clone());
    }

    // Header chrome (window title bar, menu bar, tab header) directly under the
    // title row (Requirement 3.9).
    if !model.header_lines.is_empty() {
        lines.push(String::new());
        for h in &model.header_lines {
            lines.push(h.clone());
        }
    }

    if !model.fields.is_empty() {
        lines.push(String::new());
        for f in &model.fields {
            if f.label.is_empty() {
                lines.push(f.value.clone());
            } else {
                lines.push(format!("{} . . . : {}", f.label, f.value));
            }
        }
    }

    // Tables are rendered as box-drawing GRIDS below the main frame (like the
    // command line / status bar sit outside it), so the grid's own ruled borders
    // and vertical cell separators are drawn with box-drawing glyphs rather than
    // being double-wrapped by the outer frame. See `render_table_grid`.
    // Validates: screen-snapshot-scrm Requirement 30.12, 30.13, 30.14.

    if !model.messages.is_empty() {
        lines.push(String::new());
        for m in &model.messages {
            lines.push(m.clone());
        }
    }

    if !model.buttons.is_empty() {
        lines.push(String::new());
        lines.push(
            model
                .buttons
                .iter()
                .map(|b| format!("[{b}]"))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }

    // Footer chrome (calendar, PF-key footer) below the body, still inside the
    // frame (Requirement 3.9, 3.10).
    if !model.footer_lines.is_empty() {
        lines.push(String::new());
        for f in &model.footer_lines {
            lines.push(f.clone());
        }
    }

    let inner_width = lines
        .iter()
        .map(|l| l.chars().count())
        .chain(std::iter::once(model.title.chars().count()))
        .max()
        .unwrap_or(0)
        .max(20);

    let mut out = String::new();
    // Top border.
    push_border(&mut out, g.top_left, g.horizontal, g.top_right, inner_width);
    // Title row, then a separator.
    push_content(&mut out, g.vertical, &model.title, inner_width);
    push_border(&mut out, g.tee_left, g.horizontal, g.tee_right, inner_width);
    // Body rows (skip the title we already emitted at index 0).
    for line in lines.iter().skip(1) {
        push_content(&mut out, g.vertical, line, inner_width);
    }
    // Bottom border.
    push_border(
        &mut out,
        g.bottom_left,
        g.horizontal,
        g.bottom_right,
        inner_width,
    );

    // Box-drawing table grids below the main frame (Requirement 30.12-30.14).
    for table in &model.tables {
        out.push('\n');
        if let Some(cap) = &table.caption {
            out.push_str(cap);
            out.push('\n');
        }
        render_table_grid(&mut out, table, &g);
    }

    // Command line and status bar sit OUTSIDE the frame, like ISPF.
    if !model.command_line.is_empty() {
        out.push('\n');
        out.push_str(&format!("Command ===> {}\n", model.command_line));
    }
    if !model.status_bar.text.is_empty() {
        out.push_str(&model.status_bar.text);
        out.push('\n');
    }
    out
}

/// Render a [`crate::model::Table`] as a box-drawing grid: a top rule, the header
/// row, a header/data separator rule, one line per data row, and a bottom rule.
/// Every column has a FIXED width -- the widest of its header and all its data
/// cells -- so a cell that OVERFLOWS the header widens the whole column and keeps
/// the grid aligned (Requirement 30.14). The provider cell conventions
/// (`[value v]`, `[ v]`, `____`, padded values) are preserved verbatim; this
/// function only lays them out inside box cells. `glyphs` selects Unicode
/// box-drawing or the ASCII `+ - |` fallback.
///
/// Validates: screen-snapshot-scrm Requirement 30.12, 30.13, 30.14.
pub(super) fn render_table_grid(out: &mut String, table: &crate::model::Table, glyphs: &Glyphs) {
    let cols = table
        .headers
        .len()
        .max(table.rows.iter().map(|r| r.len()).max().unwrap_or(0));
    if cols == 0 {
        return;
    }
    // Fixed per-column widths: widest of header + every data cell in that column.
    let mut widths = vec![0usize; cols];
    for (c, w) in widths.iter_mut().enumerate() {
        let header_len = table.headers.get(c).map(|h| h.chars().count()).unwrap_or(0);
        let data_len = table
            .rows
            .iter()
            .map(|r| r.get(c).map(|cell| cell.chars().count()).unwrap_or(0))
            .max()
            .unwrap_or(0);
        *w = header_len.max(data_len).max(1);
    }

    // Top rule, header row, header/data separator, data rows, bottom rule.
    push_grid_rule(out, &widths, glyphs, GridRule::Top);
    if !table.headers.is_empty() {
        push_grid_row(out, &table.headers, &widths, glyphs);
        push_grid_rule(out, &widths, glyphs, GridRule::Mid);
    }
    for row in &table.rows {
        push_grid_row(out, row, &widths, glyphs);
    }
    push_grid_rule(out, &widths, glyphs, GridRule::Bottom);
}

/// Which box-drawing corners/tees a grid rule uses.
enum GridRule {
    /// Top border (`top_left` / `tee-down` / `top_right`).
    Top,
    /// Interior header/data separator (`tee_left` / cross / `tee_right`).
    Mid,
    /// Bottom border (`bottom_left` / `tee-up` / `bottom_right`).
    Bottom,
}

/// Push one horizontal grid rule spanning every column, with the left/junction/
/// right glyphs chosen by `rule`. Each column segment is `width + 2` horizontal
/// glyphs (one pad space either side of the cell content).
fn push_grid_rule(out: &mut String, widths: &[usize], g: &Glyphs, rule: GridRule) {
    let (left, junction, right) = match rule {
        // The ASCII fallback uses '+' at every junction; Unicode uses the proper
        // down/cross/up tees so the grid reads as a true box-drawing table.
        GridRule::Top => (
            g.top_left,
            if g.horizontal == '-' { '+' } else { '\u{252C}' },
            g.top_right,
        ),
        GridRule::Mid => (
            g.tee_left,
            if g.horizontal == '-' { '+' } else { '\u{253C}' },
            g.tee_right,
        ),
        GridRule::Bottom => (
            g.bottom_left,
            if g.horizontal == '-' { '+' } else { '\u{2534}' },
            g.bottom_right,
        ),
    };
    out.push(left);
    for (i, w) in widths.iter().enumerate() {
        for _ in 0..w + 2 {
            out.push(g.horizontal);
        }
        if i + 1 < widths.len() {
            out.push(junction);
        }
    }
    out.push(right);
    out.push('\n');
}

/// Push one grid content row: each cell is wrapped in `vertical` separators and
/// padded to its fixed column width, with a single pad space either side.
fn push_grid_row(out: &mut String, cells: &[String], widths: &[usize], g: &Glyphs) {
    out.push(g.vertical);
    for (i, w) in widths.iter().enumerate() {
        let empty = String::new();
        let cell = cells.get(i).unwrap_or(&empty);
        let pad = w.saturating_sub(cell.chars().count());
        out.push(' ');
        out.push_str(cell);
        for _ in 0..pad {
            out.push(' ');
        }
        out.push(' ');
        out.push(g.vertical);
    }
    out.push('\n');
}

pub(super) fn push_border(
    out: &mut String,
    left: char,
    fill: char,
    right: char,
    inner_width: usize,
) {
    out.push(left);
    for _ in 0..inner_width + 2 {
        out.push(fill);
    }
    out.push(right);
    out.push('\n');
}

pub(super) fn push_content(out: &mut String, vertical: char, text: &str, inner_width: usize) {
    let len = text.chars().count();
    let pad = inner_width.saturating_sub(len);
    out.push(vertical);
    out.push(' ');
    out.push_str(text);
    for _ in 0..pad {
        out.push(' ');
    }
    out.push(' ');
    out.push(vertical);
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Field;

    // Validates: Requirement 4.2 -- values and command line appear.
    #[test]
    fn plain_includes_command_line_outside_frame() {
        let mut m = ScreenModel::new("T").with_field(Field::new("A", "b"));
        m.command_line = "SNAPSHOT".to_string();
        let out = render_plain(&m, RenderOptions::default());
        assert!(out.contains("Command ===> SNAPSHOT"));
        assert!(out.contains('b'));
    }

    use crate::model::Table;

    /// A small Alias-like table mirroring the owner's mock, using the provider
    /// cell conventions: a `[value v]` populated dropdown, a `[ v]` empty
    /// dropdown, and a `____` empty editable field.
    fn alias_like_table() -> Table {
        Table {
            caption: None,
            headers: vec![
                "Act".to_string(),
                "CE".to_string(),
                "Command".to_string(),
                "Alias".to_string(),
            ],
            rows: vec![
                vec![
                    "____".to_string(),
                    "[FFCMD v]".to_string(),
                    "SNAPSHOT".to_string(),
                    "SS".to_string(),
                ],
                vec![
                    "____".to_string(),
                    "[ v]".to_string(),
                    "________".to_string(),
                    "________".to_string(),
                ],
            ],
        }
    }

    // Validates: screen-snapshot-scrm Requirement 30.12, 30.13 -- a table is
    // rendered as a BOX-DRAWING grid: a header rule above the headers, a
    // header/data separator rule between the headers and the data rows, a bottom
    // rule, and box-drawing VERTICAL separators between every cell. The provider
    // cell conventions `[value v]`/`[ v]`/`____` are preserved verbatim.
    #[test]
    fn box_draw_table_grid_has_ruled_borders_and_cell_separators() {
        let mut m = ScreenModel::new("Alias Editor");
        m.tables.push(alias_like_table());
        let out = render_plain(&m, RenderOptions::default());

        // The provider cell conventions survive verbatim.
        assert!(
            out.contains("[FFCMD v]"),
            "populated dropdown cell preserved:\n{out}"
        );
        assert!(
            out.contains("[ v]"),
            "empty dropdown cell preserved:\n{out}"
        );
        assert!(
            out.contains("____"),
            "empty editable field cell preserved:\n{out}"
        );
        assert!(
            out.contains("SNAPSHOT"),
            "a real value cell is present:\n{out}"
        );

        // A header-to-data separator rule exists: a line made only of Unicode
        // horizontal + cross glyphs (the grid's internal rule, distinct from the
        // frame's own tee-based separators).
        let has_cross_rule = out.lines().any(|l| {
            l.contains('\u{253C}') // cross '+'
                && l.chars().all(|c| {
                    c == '\u{2500}' // horizontal
                        || c == '\u{253C}' // cross
                        || c == '\u{251C}' // tee-left
                        || c == '\u{2524}' // tee-right
                        || c == ' '
                })
        });
        assert!(
            has_cross_rule,
            "the grid must draw a ruled separator with box-drawing crosses:\n{out}"
        );

        // Each data row carries interior vertical cell separators.
        let data_row = out
            .lines()
            .find(|l| l.contains("SNAPSHOT"))
            .expect("a data row");
        let interior_verticals = data_row.matches('\u{2502}').count();
        assert!(
            interior_verticals >= 5,
            "a 4-column data row has >=5 vertical separators (frame + interior); got {interior_verticals} in {data_row:?}"
        );
    }

    // Validates: screen-snapshot-scrm Requirement 30.12, 30.14 -- the box-drawing
    // grid has a FIXED column width per column (headers + data share one width per
    // column) and WIDENS a column when a cell overflows the header width. Every
    // grid line therefore has the SAME total character width.
    #[test]
    fn box_draw_table_columns_fixed_width_and_widen_on_overflow() {
        let mut m = ScreenModel::new("T");
        m.tables.push(Table {
            caption: None,
            headers: vec!["A".to_string(), "B".to_string()],
            rows: vec![
                vec!["x".to_string(), "a-very-long-overflowing-value".to_string()],
                vec!["y".to_string(), "z".to_string()],
            ],
        });
        let out = render_plain(&m, RenderOptions::default());
        assert!(
            out.contains("a-very-long-overflowing-value"),
            "the overflowing value must fit (column widened):\n{out}"
        );
        // Isolate the GRID block: from its top rule (a line with the tee-down
        // junction, unique to the grid) through its bottom rule (tee-up). Every
        // line in that block must share the same width (fixed columns). The outer
        // frame uses only corners + tee-left/right, so it is excluded.
        let lines: Vec<&str> = out.lines().collect();
        let top = lines
            .iter()
            .position(|l| l.contains('\u{252C}'))
            .expect("grid top rule");
        let bottom = lines
            .iter()
            .rposition(|l| l.contains('\u{2534}'))
            .expect("grid bottom rule");
        let grid_widths: Vec<usize> = lines[top..=bottom]
            .iter()
            .map(|l| l.chars().count())
            .collect();
        let first = grid_widths[0];
        assert!(
            grid_widths.iter().all(|w| *w == first),
            "every grid line shares the same width (fixed columns); got {grid_widths:?}\n{out}"
        );
    }

    // Validates: screen-snapshot-scrm Requirement 30.12 -- the ASCII fallback
    // (unicode_box = false) draws the grid with `+ - |` instead of box-drawing
    // glyphs and still preserves the cell conventions.
    #[test]
    fn box_draw_table_grid_ascii_fallback() {
        let mut m = ScreenModel::new("T");
        m.tables.push(alias_like_table());
        let out = render_plain(&m, RenderOptions { unicode_box: false });
        assert!(out.contains("[FFCMD v]"));
        assert!(out.contains("____"));
        assert!(out.contains('+'), "ASCII grid uses '+' crosses:\n{out}");
        assert!(out.contains('|'), "ASCII grid uses '|' separators:\n{out}");
        // No box-drawing glyphs leak into the ASCII fallback.
        assert!(
            !out.contains('\u{2502}'),
            "no box vertical in ASCII mode:\n{out}"
        );
        assert!(
            !out.contains('\u{253C}'),
            "no box cross in ASCII mode:\n{out}"
        );
    }

    // Validates: screen-snapshot-scrm Req 3.8-3.13 -- body_lines render
    // VERBATIM inside the frame, the FIRST body row sits directly under the
    // title separator (no leading blank), and no generic chrome is injected.
    #[test]
    fn body_lines_render_verbatim_as_first_body_rows() {
        let m = ScreenModel::new("Primary Option Menu")
            .with_body_line("Command ===> ____")
            .with_body_line("0 Settings : FFWB Settings (Settings)   October 2026");
        let out = render_plain(&m, RenderOptions::default());
        let rows: Vec<&str> = out.lines().collect();
        // rows: [top border, title, separator, first body row, ...]
        assert!(
            rows[3].contains("Command ===> ____"),
            "first body row must be the command line (no leading blank):\n{out}"
        );
        assert!(rows[4].contains("0 Settings : FFWB Settings (Settings)   October 2026"));
        // No command-line echo OUTSIDE the frame when command_line is empty.
        assert!(!out.contains("\nCommand ===> \n"));
    }
}
