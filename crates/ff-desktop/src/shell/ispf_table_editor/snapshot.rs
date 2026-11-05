//! Shared ISPF Table Editor -- the PURE, egui-free SNAPSHOT cell conventions and
//! table builders (CR-CH-075, CR-NR-116, ispf-table-editor Requirement 12).
//!
//! A snapshot of a table editor must look like the live grid. These helpers
//! implement the cell conventions ONCE so every table editor (Alias first, then
//! Menu / Kinds / Keys / Theme) gets the same form from one place instead of
//! re-deriving it (Requirement 12.7):
//!   - [`combo_cell`] -- a dropdown cell `[value v]` (empty -> `[ v]`) padded to
//!     the field width.
//!   - [`text_cell`] -- a text cell: the value padded to the width, or a run of
//!     `_` filling the width when empty.
//!   - [`sanitise_cell`] -- replaces tab / CR / LF with a space so a cell never
//!     breaks the grid.
//!   - [`snapshot_table`] / [`raw_table`] -- turn a host's displayed rows into a
//!     [`ff_screen_model::Table`] with (snapshot) or without (markdown) the
//!     trailing scratch row.
//!
//! Nothing here depends on egui; placeholder HINT TEXT a live widget shows for an
//! empty field is NOT cell content, so none of these helpers takes a hint-text
//! parameter (Requirement 12.4). A cell longer than the width is NOT truncated
//! (the renderer widens the column, Requirement 12.2 / screen-snapshot-scrm
//! Requirement 29.6).

// The cell helpers and table builders are the shared snapshot core; their sole
// consumer (the Alias host's `snapshot_cells` wiring) lands in FEAT-003, and the
// Menu / Kinds hosts follow. Keep the module-level allow until a host consumes
// them, mirroring the sibling `state` module's precedent.
#![allow(dead_code)]

use super::state::{CellKind, ColumnSpec};

/// Replace tab / CR / LF in `value` with a single space so a cell never breaks
/// the snapshot grid (ispf-table-editor Requirement 12.2). No hint-text
/// parameter (Requirement 12.4).
pub fn sanitise_cell(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            '\t' | '\r' | '\n' => ' ',
            other => other,
        })
        .collect()
}

/// A combo (dropdown) snapshot cell: `[value v]`, or `[ v]` when empty, padded
/// to `width` (ispf-table-editor Requirement 12.2). The value is sanitised; a
/// cell longer than `width` is NOT truncated -- it widens the column.
pub fn combo_cell(value: &str, width: usize) -> String {
    let inner = sanitise_cell(value);
    let cell = if inner.is_empty() {
        "[ v]".to_string()
    } else {
        format!("[{inner} v]")
    };
    pad_to(&cell, width)
}

/// A text snapshot cell: `value` padded to `width`, or a run of `_` filling the
/// width when empty (ispf-table-editor Requirement 12.2). The value is
/// sanitised; a cell longer than `width` is NOT truncated -- it widens the
/// column.
pub fn text_cell(value: &str, width: usize) -> String {
    let inner = sanitise_cell(value);
    if inner.is_empty() {
        "_".repeat(width)
    } else {
        pad_to(&inner, width)
    }
}

/// Pad `cell` with trailing spaces to at least `width` characters; never
/// truncates (a longer cell is returned unchanged so the column widens).
fn pad_to(cell: &str, width: usize) -> String {
    let len = cell.chars().count();
    if len >= width {
        cell.to_string()
    } else {
        let mut out = String::from(cell);
        out.extend(std::iter::repeat_n(' ', width - len));
        out
    }
}

/// Render one raw-value row into its per-column snapshot cells, applying each
/// column's [`CellKind`] and snapshot width (ispf-table-editor Requirement 12.3).
fn row_cells(columns: &[ColumnSpec], raw: &[String]) -> Vec<String> {
    columns
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let value = raw.get(i).map(String::as_str).unwrap_or("");
            match col.cell {
                CellKind::Text => text_cell(value, col.snapshot_width),
                CellKind::Combo => combo_cell(value, col.snapshot_width),
            }
        })
        .collect()
}

/// Build the SNAPSHOT table: headers from `columns`, one table row per displayed
/// host row with each column's cell kind + width applied, THEN the always-present
/// trailing scratch row (ispf-table-editor Requirement 12.3, 12.5). `rows` and
/// `scratch_cells` are RAW per-column values (no markers, no padding).
pub fn snapshot_table(
    columns: &[ColumnSpec],
    rows: &[Vec<String>],
    scratch_cells: &[String],
) -> ff_screen_model::Table {
    let headers = columns.iter().map(|c| c.header.to_string()).collect();
    let mut out: Vec<Vec<String>> = rows.iter().map(|raw| row_cells(columns, raw)).collect();
    // The scratch row is the blank row in the same conventions (Requirement 12.3).
    out.push(row_cells(columns, scratch_cells));
    ff_screen_model::Table {
        caption: None,
        headers,
        rows: out,
    }
}

/// Build the RAW table: headers from `columns` and one row per host row holding
/// the UNPADDED raw values, with NO scratch row -- for a Context's
/// `markdown_sections` (ispf-table-editor Requirement 12.3, screen-snapshot-scrm
/// Requirement 30.40).
pub fn raw_table(columns: &[ColumnSpec], rows: &[Vec<String>]) -> ff_screen_model::Table {
    let headers = columns.iter().map(|c| c.header.to_string()).collect();
    let out: Vec<Vec<String>> = rows
        .iter()
        .map(|raw| {
            columns
                .iter()
                .enumerate()
                .map(|(i, _)| raw.get(i).cloned().unwrap_or_default())
                .collect()
        })
        .collect();
    ff_screen_model::Table {
        caption: None,
        headers,
        rows: out,
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::ispf_table_editor::state::{CellKind, ColumnSpec, CompareMode};

    // Validates: ispf-table-editor Requirement 12.2 -- combo_cell wraps the
    // value as `[value v]` and pads to the width; an empty value is `[ v]`.
    #[test]
    fn combo_cell_wraps_and_pads() {
        // `[FFCMD v]` is 9 chars; padded to 10 adds one trailing space.
        assert_eq!(combo_cell("FFCMD", 10), "[FFCMD v] ");
        assert_eq!(combo_cell("", 6), "[ v]  ", "empty combo is [ v] padded");
    }

    // Validates: ispf-table-editor Requirement 12.2 -- a combo value longer than
    // the width is NOT truncated (the column widens).
    #[test]
    fn combo_cell_wider_than_width_is_not_truncated() {
        let cell = combo_cell("VERYLONGVALUE", 4);
        assert_eq!(cell, "[VERYLONGVALUE v]");
        assert!(cell.chars().count() > 4, "the cell widens past the width");
    }

    // Validates: ispf-table-editor Requirement 12.2 -- text_cell pads the value
    // to the width, and an empty value becomes a run of `_` filling the width.
    #[test]
    fn text_cell_pads_and_empty_is_underscores() {
        assert_eq!(text_cell("hi", 5), "hi   ");
        assert_eq!(text_cell("", 4), "____", "an empty text cell is _ fill");
    }

    // Validates: ispf-table-editor Requirement 12.2 -- a text value longer than
    // the width is NOT truncated (the column widens).
    #[test]
    fn text_cell_wider_than_width_is_not_truncated() {
        let cell = text_cell("abcdefgh", 3);
        assert_eq!(cell, "abcdefgh");
        assert!(cell.chars().count() > 3, "the cell widens past the width");
    }

    // Validates: ispf-table-editor Requirement 12.2 -- the sanitiser replaces
    // tab / CR / LF with a single space.
    #[test]
    fn sanitise_cell_replaces_control_chars_with_space() {
        assert_eq!(sanitise_cell("a\tb\rc\nd"), "a b c d");
    }

    // Validates: ispf-table-editor Requirement 12.4 -- the helpers have NO
    // hint-text parameter; an empty cell shows the underscore/`[ v]` convention,
    // never a hint.
    #[test]
    fn empty_cells_use_the_convention_not_a_hint() {
        // text_cell / combo_cell take only (value, width): no hint arg exists.
        assert_eq!(text_cell("", 3), "___");
        assert_eq!(combo_cell("", 4), "[ v]");
    }

    fn cols() -> Vec<ColumnSpec> {
        vec![
            ColumnSpec::display("act", "Act").with_snapshot(CellKind::Text, 3),
            ColumnSpec::filterable("ce", "CE", CompareMode::EqualsIgnoreCase)
                .with_snapshot(CellKind::Combo, 8),
        ]
    }

    // Validates: ispf-table-editor Requirement 12.3, 12.5 -- snapshot_table emits
    // headers from the columns, one cell-rendered row per host row, THEN the
    // trailing scratch row (blank in the same conventions).
    #[test]
    fn snapshot_table_renders_rows_then_scratch() {
        let columns = cols();
        let rows = vec![
            vec!["".to_string(), "FFCMD".to_string()],
            vec!["".to_string(), "FFEDIT".to_string()],
        ];
        let scratch = vec![String::new(), String::new()];
        let table = snapshot_table(&columns, &rows, &scratch);

        assert_eq!(table.headers, vec!["Act".to_string(), "CE".to_string()]);
        assert_eq!(table.rows.len(), 3, "two data rows plus the scratch row");
        assert_eq!(
            table.rows[0],
            vec!["___".to_string(), "[FFCMD v]".to_string()]
        );
        assert_eq!(
            table.rows[1],
            vec!["___".to_string(), "[FFEDIT v]".to_string()]
        );
        // The scratch row is the blank row in the same conventions.
        assert_eq!(
            table.rows[2],
            vec!["___".to_string(), "[ v]    ".to_string()],
            "the trailing scratch row is the blank row rendered by convention"
        );
    }

    // Validates: ispf-table-editor Requirement 12.3 -- raw_table emits the
    // headers and the UNPADDED raw values with NO scratch row.
    #[test]
    fn raw_table_is_unpadded_with_no_scratch() {
        let columns = cols();
        let rows = vec![vec!["".to_string(), "FFCMD".to_string()]];
        let table = raw_table(&columns, &rows);

        assert_eq!(table.headers, vec!["Act".to_string(), "CE".to_string()]);
        assert_eq!(table.rows.len(), 1, "no scratch row is appended");
        assert_eq!(
            table.rows[0],
            vec!["".to_string(), "FFCMD".to_string()],
            "raw values are unpadded and uncombo-wrapped"
        );
    }
}
