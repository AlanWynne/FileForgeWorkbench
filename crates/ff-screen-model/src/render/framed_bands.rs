//! Table-band layout for the framed (common-frame) renderer (CR-NR-116,
//! screen-snapshot-scrm Requirement 30.32-30.38).
//!
//! A `tables` entry in a framed model is NOT drawn as a nested box. It becomes
//! ordinary frame BANDS: a header band and one band per data row, each cell
//! padded to its column width so the dividers align across the header and every
//! data row (30.33). The frame's own rules above and below the grid carry the
//! tee-down / cross / tee-up junctions via the existing band/junction machinery
//! in `framed.rs` (30.34), so this module introduces NO second junction
//! mechanism: it only turns a [`Table`] into padded cell vectors that `framed`
//! wraps in `Row::Band`.
//!
//! Each cell is carried as a DISCRETE value: a ` | ` inside a cell stays in the
//! cell (it is never re-split on the band separator) and control characters
//! (tab, CR, LF) are flattened to a single space so a tab is never read as the
//! right-align marker (30.36).

use crate::model::Table;

/// A table reduced to padded frame-band cell vectors, column-aligned.
pub(super) struct TableBands {
    /// The header band's padded cells, or `None` when the table has no headers.
    pub(super) header: Option<Vec<String>>,
    /// One padded-cell vector per data row.
    pub(super) rows: Vec<Vec<String>>,
}

/// Reduce `table` to column-aligned [`TableBands`], or `None` when the table has
/// no columns (it is omitted entirely, 30.37). Column width is the widest of the
/// header and every data cell in that column (30.33); each cell is sanitised
/// (control chars -> space, 30.36) and padded to its column width.
pub(super) fn table_bands(table: &Table) -> Option<TableBands> {
    let cols = table
        .headers
        .len()
        .max(table.rows.iter().map(|r| r.len()).max().unwrap_or(0));
    if cols == 0 {
        return None;
    }
    let widths = column_widths(table, cols);
    let header = (!table.headers.is_empty()).then(|| pad_row(&table.headers, &widths));
    let rows = table.rows.iter().map(|r| pad_row(r, &widths)).collect();
    Some(TableBands { header, rows })
}

/// The fixed per-column width: the widest sanitised header or data cell in that
/// column, at least 1.
fn column_widths(table: &Table, cols: usize) -> Vec<usize> {
    (0..cols)
        .map(|c| {
            let header = table.headers.get(c).map(|h| sanitise(h).chars().count());
            let data = table
                .rows
                .iter()
                .filter_map(|r| r.get(c).map(|cell| sanitise(cell).chars().count()));
            header.into_iter().chain(data).max().unwrap_or(0).max(1)
        })
        .collect()
}

/// Sanitise and pad each cell of `cells` to its column width (missing trailing
/// cells become empty padded cells so every band has the same cell count).
fn pad_row(cells: &[String], widths: &[usize]) -> Vec<String> {
    widths
        .iter()
        .enumerate()
        .map(|(c, w)| {
            let text = cells.get(c).map(|s| sanitise(s)).unwrap_or_default();
            let pad = w.saturating_sub(text.chars().count());
            format!("{text}{}", " ".repeat(pad))
        })
        .collect()
}

/// Flatten the control characters tab, CR and LF to a single space so a cell is
/// one display line and a tab is never read as the band right-align marker
/// (30.36). All other characters, including an in-cell ` | `, are preserved.
fn sanitise(cell: &str) -> String {
    cell.chars()
        .map(|c| {
            if matches!(c, '\t' | '\r' | '\n') {
                ' '
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(headers: &[&str], rows: &[&[&str]]) -> Table {
        Table {
            caption: None,
            headers: headers.iter().map(|s| s.to_string()).collect(),
            rows: rows
                .iter()
                .map(|r| r.iter().map(|s| s.to_string()).collect())
                .collect(),
        }
    }

    // Validates: screen-snapshot-scrm Requirement 30.37 -- a table with no
    // columns yields no bands.
    #[test]
    fn no_columns_yields_none() {
        assert!(table_bands(&table(&[], &[])).is_none());
    }

    // Validates: screen-snapshot-scrm Requirement 30.33 -- each column is padded
    // to the widest of its header and all cells, so every band cell shares the
    // column width.
    #[test]
    fn cells_padded_to_widest_of_header_and_data() {
        let b = table_bands(&table(&["A", "BB"], &[&["xxxx", "y"]])).expect("bands");
        let header = b.header.expect("header");
        assert_eq!(header[0].chars().count(), 4, "col 0 widened to 'xxxx'");
        assert_eq!(header[1].chars().count(), 2, "col 1 is header 'BB'");
        assert_eq!(b.rows[0][0], "xxxx");
        assert_eq!(b.rows[0][1], "y ");
    }

    // Validates: screen-snapshot-scrm Requirement 30.37 -- a short data row is
    // padded out to the full column count.
    #[test]
    fn short_row_is_padded_to_column_count() {
        let b = table_bands(&table(&["A", "B", "C"], &[&["x"]])).expect("bands");
        assert_eq!(b.rows[0].len(), 3);
        assert_eq!(b.rows[0][1], " ");
    }

    // Validates: screen-snapshot-scrm Requirement 30.36 -- an in-cell ` | ` is
    // kept and control chars become a single space.
    #[test]
    fn cell_keeps_pipe_and_flattens_control_chars() {
        let b = table_bands(&table(&["H"], &[&["a | b\tc\rd\ne"]])).expect("bands");
        assert_eq!(b.rows[0][0], "a | b c d e");
    }

    // Validates: screen-snapshot-scrm Requirement 30.37 -- a table with no rows
    // still yields a header band.
    #[test]
    fn no_rows_still_yields_header() {
        let b = table_bands(&table(&["A"], &[])).expect("bands");
        assert!(b.header.is_some());
        assert!(b.rows.is_empty());
    }
}
