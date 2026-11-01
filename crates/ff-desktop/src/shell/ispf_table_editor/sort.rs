//! Shared ISPF Table Editor -- the PURE, egui-free Sort core (CR-CH-075,
//! ispf-table-editor Requirement 5).
//!
//! `SORT <column> [ASC|DESC]` reorders the host's BACKING row list by a
//! host-declared sortable key column, ascending by default. The sort is STABLE
//! (`slice::sort_by` is stable) so rows equal on the key retain their relative
//! order (Requirement 5.2). A host with NO sortable column, or an unknown column
//! argument, produces a non-blocking notice and leaves the order unchanged
//! (Requirement 5.3).

use super::state::{IspfTableHost, SortDirection, SortRequest};

/// The non-blocking outcome of a sort attempt.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SortResult {
    /// True when the backing list was reordered.
    pub reordered: bool,
    /// A non-blocking notice when nothing was sorted (no sortable column / an
    /// unknown column argument), else `None`.
    pub notice: Option<String>,
}

/// Parse a `SORT <column> [ASC|DESC]` command line into a [`SortRequest`]. The
/// leading `SORT` verb is matched case-insensitively; a missing column means
/// "the default sortable column" (`column = None`); a missing direction defaults
/// to ascending. Returns `None` when the line is not a SORT command.
///
/// Validates: ispf-table-editor Requirement 5.1
pub fn parse_sort_command(line: &str) -> Option<SortRequest> {
    let mut parts = line.split_whitespace();
    let verb = parts.next()?;
    if !verb.eq_ignore_ascii_case("SORT") {
        return None;
    }
    let column = parts.next().map(|c| c.to_string());
    let direction = match parts.next() {
        Some(d) if d.eq_ignore_ascii_case("DESC") => SortDirection::Descending,
        _ => SortDirection::Ascending,
    };
    Some(SortRequest { column, direction })
}

/// Apply `request` over `rows` via `host` (ispf-table-editor Requirement 5). The
/// key column is the requested column (matched case-insensitively against the
/// host's SORTABLE `ColumnSpec` ids) or, when none is named, the FIRST sortable
/// column. A stable sort by `host.sort_key` reorders the backing list.
///
/// Validates: ispf-table-editor Requirement 5.2, 5.3, 5.4
pub fn apply_sort<H: IspfTableHost>(
    host: &H,
    rows: &mut [H::Row],
    request: &SortRequest,
) -> SortResult {
    let sortable: Vec<&super::state::ColumnSpec> =
        host.columns().iter().filter(|c| c.sortable).collect();
    if sortable.is_empty() {
        return SortResult {
            reordered: false,
            notice: Some("nothing to sort (no sortable column)".to_string()),
        };
    }
    // Resolve the key column: the named one (if sortable) or the first sortable.
    let column = match &request.column {
        Some(name) => match sortable.iter().find(|c| c.id.eq_ignore_ascii_case(name)) {
            Some(c) => c.id,
            None => {
                return SortResult {
                    reordered: false,
                    notice: Some(format!("unknown sort column '{name}'")),
                };
            }
        },
        None => sortable[0].id,
    };

    // A stable sort by the host-declared key string. `sort_by_key` is stable, so
    // equal rows keep their relative order (Requirement 5.2). For DESC, keying on
    // `Reverse(key)` reverses the key order while STILL being a stable sort, so
    // ties keep their original relative order.
    match request.direction {
        SortDirection::Ascending => rows.sort_by_key(|row| host.sort_key(row, column)),
        SortDirection::Descending => {
            rows.sort_by_key(|row| std::cmp::Reverse(host.sort_key(row, column)))
        }
    }
    SortResult {
        reordered: true,
        notice: None,
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::ispf_table_editor::state::{
        ColumnId, ColumnSpec, CompareMode, FilterState, IspfTableHost, LineCommandSet,
        SortDirection,
    };

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    struct StubRow {
        key: String,
        tag: String,
        act: String,
    }

    fn r(key: &str, tag: &str) -> StubRow {
        StubRow {
            key: key.to_string(),
            tag: tag.to_string(),
            act: String::new(),
        }
    }

    /// Host with one sortable column `key`.
    struct SortableHost {
        columns: Vec<ColumnSpec>,
    }
    impl SortableHost {
        fn new() -> Self {
            Self {
                columns: vec![
                    ColumnSpec::display("act", "Act"),
                    ColumnSpec::filterable("key", "Key", CompareMode::ContainsIgnoreCase)
                        .with_sortable(),
                ],
            }
        }
    }
    impl IspfTableHost for SortableHost {
        type Row = StubRow;
        fn columns(&self) -> &[ColumnSpec] {
            &self.columns
        }
        fn line_command_set(&self) -> LineCommandSet {
            LineCommandSet::alias()
        }
        fn blank_row(&self) -> StubRow {
            StubRow::default()
        }
        fn clone_row(&self, row: &StubRow) -> StubRow {
            row.clone()
        }
        fn act_of<'a>(&self, row: &'a StubRow) -> &'a str {
            &row.act
        }
        fn clear_act(&self, row: &mut StubRow) {
            row.act.clear();
        }
        fn filter_matches(&self, _row: &StubRow, _filter: &FilterState) -> bool {
            true
        }
        fn sort_key(&self, row: &StubRow, _column: ColumnId) -> String {
            row.key.to_uppercase()
        }
    }

    /// Host with NO sortable column.
    struct UnsortedHost {
        columns: Vec<ColumnSpec>,
    }
    impl IspfTableHost for UnsortedHost {
        type Row = StubRow;
        fn columns(&self) -> &[ColumnSpec] {
            &self.columns
        }
        fn line_command_set(&self) -> LineCommandSet {
            LineCommandSet::alias()
        }
        fn blank_row(&self) -> StubRow {
            StubRow::default()
        }
        fn clone_row(&self, row: &StubRow) -> StubRow {
            row.clone()
        }
        fn act_of<'a>(&self, row: &'a StubRow) -> &'a str {
            &row.act
        }
        fn clear_act(&self, row: &mut StubRow) {
            row.act.clear();
        }
        fn filter_matches(&self, _row: &StubRow, _filter: &FilterState) -> bool {
            true
        }
        fn sort_key(&self, _row: &StubRow, _column: ColumnId) -> String {
            String::new()
        }
    }

    fn keys(rows: &[StubRow]) -> Vec<String> {
        rows.iter().map(|r| r.key.clone()).collect()
    }

    // Validates: ispf-table-editor Requirement 5.1 -- the SORT command parses the
    // column and direction, case-insensitively, with ASC default.
    #[test]
    fn parse_sort_command_forms() {
        assert_eq!(parse_sort_command("stuff"), None);
        let a = parse_sort_command("SORT key").expect("parses");
        assert_eq!(a.column.as_deref(), Some("key"));
        assert_eq!(a.direction, SortDirection::Ascending);
        let d = parse_sort_command("sort KEY desc").expect("parses");
        assert_eq!(d.direction, SortDirection::Descending);
        let bare = parse_sort_command("SORT").expect("parses");
        assert_eq!(bare.column, None);
    }

    // Validates: ispf-table-editor Requirement 5.2 -- a stable ASC sort reorders
    // the backing list and keeps equal-key rows in their relative order.
    #[test]
    fn stable_ascending_sort_reorders_and_keeps_ties_stable() {
        let host = SortableHost::new();
        let mut rows = vec![r("B", "1"), r("A", "2"), r("B", "3"), r("A", "4")];
        let req = SortRequest {
            column: Some("key".to_string()),
            direction: SortDirection::Ascending,
        };
        let res = apply_sort(&host, &mut rows, &req);
        assert!(res.reordered);
        assert_eq!(keys(&rows), vec!["A", "A", "B", "B"]);
        // Ties preserve original relative order: A(2) before A(4), B(1) before B(3).
        assert_eq!(rows[0].tag, "2");
        assert_eq!(rows[1].tag, "4");
        assert_eq!(rows[2].tag, "1");
        assert_eq!(rows[3].tag, "3");
    }

    // Validates: ispf-table-editor Requirement 5.1 -- a DESC sort reverses the
    // key order while keeping equal-key rows stable.
    #[test]
    fn descending_sort_reverses_and_keeps_ties_stable() {
        let host = SortableHost::new();
        let mut rows = vec![r("A", "1"), r("B", "2"), r("A", "3")];
        let req = SortRequest {
            column: None,
            direction: SortDirection::Descending,
        };
        apply_sort(&host, &mut rows, &req);
        assert_eq!(keys(&rows), vec!["B", "A", "A"]);
        // Ties (the two A rows) keep original order 1 before 3.
        assert_eq!(rows[1].tag, "1");
        assert_eq!(rows[2].tag, "3");
    }

    // Validates: ispf-table-editor Requirement 5.3 -- a host with no sortable
    // column leaves the order unchanged and returns a non-blocking notice.
    #[test]
    fn no_sortable_column_is_a_noop_with_notice() {
        let host = UnsortedHost {
            columns: vec![ColumnSpec::display("a", "A")],
        };
        let mut rows = vec![r("B", "1"), r("A", "2")];
        let req = SortRequest {
            column: None,
            direction: SortDirection::Ascending,
        };
        let res = apply_sort(&host, &mut rows, &req);
        assert!(!res.reordered, "order is unchanged");
        assert_eq!(keys(&rows), vec!["B", "A"]);
        assert!(res.notice.is_some(), "a non-blocking notice is returned");
    }

    // Validates: ispf-table-editor Requirement 5.1 -- an unknown sort column is a
    // no-op with a notice.
    #[test]
    fn unknown_column_is_a_noop_with_notice() {
        let host = SortableHost::new();
        let mut rows = vec![r("B", "1"), r("A", "2")];
        let req = SortRequest {
            column: Some("nope".to_string()),
            direction: SortDirection::Ascending,
        };
        let res = apply_sort(&host, &mut rows, &req);
        assert!(!res.reordered);
        assert_eq!(keys(&rows), vec!["B", "A"]);
        assert!(res.notice.is_some());
    }
}
