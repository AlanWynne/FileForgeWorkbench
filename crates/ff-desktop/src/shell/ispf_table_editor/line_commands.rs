//! Shared ISPF Table Editor -- the PURE, egui-free line-command apply pass
//! (CR-CH-075, ispf-table-editor Requirement 4).
//!
//! This is the one deterministic batch pass that structures a host's backing
//! `Vec<Row>` from the transient Act letters typed into the rows. It REUSES the
//! GUI-free `ff-line-commands` engine for PARSING and CLASSIFICATION (no new
//! parser): [`ff_line_commands::LineCommandParser::parse`] and
//! [`ff_line_commands::command::classify`] decide whether a letter is an
//! Immediate (I/D), a Source marker (C/M), or a Target marker (A/B). The
//! adapter here maps that classified result onto the generic row vector
//! (insert / remove / clone) via the [`super::state::IspfTableHost`] trait.
//!
//! Determinism rules (ispf-table-editor Requirement 4.3):
//!   - Immediate commands are applied by DESCENDING backing index so an
//!     insert/delete never shifts an unprocessed row's index.
//!   - A `C` / `M` source is PENDING until an `A` (after) / `B` (before) target
//!     is supplied; an unresolved source is RETAINED (its Act cell is NOT
//!     cleared) and a non-blocking notice is returned (Requirement 4.4).
//!   - An Act letter that is unrecognised or not in the host's enabled
//!     Line_Command_Set is CLEARED and ignored, with a notice (Requirement 4.5).

use ff_line_commands::command::{classify, LineCommandCategory, LineCommandKind};
use ff_line_commands::LineCommandParser;

use super::state::{IspfTableHost, LineCommandSet};

/// The non-blocking outcome of one apply pass (ispf-table-editor Requirement
/// 4.4, 4.5). An empty `notices` list means every Act cell applied cleanly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApplyResult {
    /// Human-readable, non-blocking notices (e.g. an unresolved copy source, an
    /// ignored unrecognised letter). Never an error that aborts the pass.
    pub notices: Vec<String>,
}

impl ApplyResult {
    fn push(&mut self, notice: impl Into<String>) {
        self.notices.push(notice.into());
    }
}

/// One classified Act cell, keyed by its BACKING index into the full row list.
struct Entry {
    index: usize,
    kind: LineCommandKind,
}

/// Apply the pending Act line commands over `rows` via `host`, in one
/// deterministic batch (ispf-table-editor Requirement 4). Returns the
/// non-blocking [`ApplyResult`].
///
/// The pass:
/// 1. Collects `(backing_index, parsed_kind)` for every non-empty Act cell,
///    clearing unrecognised / not-enabled letters with a notice.
/// 2. Pairs each Source marker (C/M) with the FIRST Target marker (A/B) by
///    backing-index order; an unresolved source is retained with a notice.
/// 3. Applies Copy/Move for resolved pairs, then the Immediate I/D set by
///    DESCENDING index, and clears every consumed Act cell.
///
/// Validates: ispf-table-editor Requirement 4.2, 4.3, 4.4, 4.5, 4.7
pub fn apply_line_commands<H: IspfTableHost>(host: &H, rows: &mut Vec<H::Row>) -> ApplyResult {
    let set = host.line_command_set();
    let mut result = ApplyResult::default();

    // --- 1. Parse + classify every non-empty Act cell -----------------------
    let mut entries: Vec<Entry> = Vec::new();
    let mut targets: Vec<(usize, LineCommandKind)> = Vec::new();
    let mut clear: Vec<usize> = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let act = host.act_of(row).trim();
        if act.is_empty() {
            continue;
        }
        let Ok(parsed) = LineCommandParser::parse(act, index as u64) else {
            result.push(format!("ignored unrecognised line command '{act}'"));
            clear.push(index);
            continue;
        };
        if !command_enabled(&parsed.kind, &set) {
            result.push(format!("ignored line command '{act}' (not enabled)"));
            clear.push(index);
            continue;
        }
        match classify(&parsed.kind) {
            LineCommandCategory::Target => targets.push((index, parsed.kind)),
            LineCommandCategory::Immediate | LineCommandCategory::Source => entries.push(Entry {
                index,
                kind: parsed.kind,
            }),
            // Block markers are not enabled by any current host set; treat as
            // unrecognised and clear (defensive -- command_enabled already
            // rejects them for the alias set).
            LineCommandCategory::Block => {
                result.push(format!("ignored block line command '{act}' (not enabled)"));
                clear.push(index);
            }
        }
    }

    // Clear the ignored cells up front (they take no part in the batch).
    for &i in &clear {
        host.clear_act(&mut rows[i]);
    }

    // --- 2. Resolve Copy/Move sources against the first target --------------
    // A single target serves the pending sources in ISPF; here we pair each
    // source with the first available target by index. Unresolved sources are
    // retained with a notice (Requirement 4.4).
    let mut copy_ops: Vec<CopyMove> = Vec::new();
    let mut consumed_targets: Vec<usize> = Vec::new();
    let mut unresolved_sources: Vec<usize> = Vec::new();
    for entry in &entries {
        if classify(&entry.kind) != LineCommandCategory::Source {
            continue;
        }
        if let Some(&(t_index, ref t_kind)) = targets
            .iter()
            .find(|(ti, _)| !consumed_targets.contains(ti))
        {
            consumed_targets.push(t_index);
            let before = matches!(t_kind, LineCommandKind::Before);
            let is_move = matches!(entry.kind, LineCommandKind::Move);
            copy_ops.push(CopyMove {
                source: entry.index,
                target: t_index,
                before,
                is_move,
            });
        } else {
            unresolved_sources.push(entry.index);
        }
    }
    if !unresolved_sources.is_empty() {
        result.push("copy/move source pending a target (A/B)".to_string());
    }
    // A target with no source is a harmless no-op: clear it with a notice.
    for (t_index, _) in &targets {
        if !consumed_targets.contains(t_index) {
            result.push("ignored target (A/B) with no copy/move source".to_string());
            host.clear_act(&mut rows[*t_index]);
        }
    }

    // --- 3a. Apply Copy/Move ------------------------------------------------
    // Build the new-row payloads BEFORE any structural change so indices are
    // valid, then insert (and, for Move, mark the source for removal).
    //
    // Apply inserts by DESCENDING insertion point so earlier insertions do not
    // shift later target indices.
    let mut inserts: Vec<(usize, H::Row)> = Vec::new();
    let mut move_removals: Vec<usize> = Vec::new();
    for op in &copy_ops {
        let mut new_row = host.clone_row(&rows[op.source]);
        host.clear_act(&mut new_row);
        let at = if op.before { op.target } else { op.target + 1 };
        inserts.push((at, new_row));
        host.clear_act(&mut rows[op.source]);
        host.clear_act(&mut rows[op.target]);
        if op.is_move {
            move_removals.push(op.source);
        }
    }
    inserts.sort_by_key(|a| std::cmp::Reverse(a.0));
    for (at, new_row) in inserts {
        let at = at.min(rows.len());
        rows.insert(at, new_row);
        // A move-removal index past the insertion point shifts by one.
        for r in move_removals.iter_mut() {
            if *r >= at {
                *r += 1;
            }
        }
    }
    move_removals.sort_unstable();
    move_removals.dedup();
    for &r in move_removals.iter().rev() {
        if r < rows.len() {
            rows.remove(r);
        }
    }

    // --- 3b. Apply Immediate I/D by DESCENDING index ------------------------
    let mut immediates: Vec<&Entry> = entries
        .iter()
        .filter(|e| classify(&e.kind) == LineCommandCategory::Immediate)
        .collect();
    immediates.sort_by_key(|a| std::cmp::Reverse(a.index));
    for entry in immediates {
        // The index is still valid: immediates were collected against the
        // ORIGINAL list and copy/move already cleared their own cells; a C/M
        // never overlaps an I/D cell (one Act letter per row).
        if entry.index >= rows.len() {
            continue;
        }
        match &entry.kind {
            LineCommandKind::Insert => {
                host.clear_act(&mut rows[entry.index]);
                let blank = host.blank_row();
                rows.insert(entry.index + 1, blank);
            }
            LineCommandKind::Delete => {
                rows.remove(entry.index);
            }
            _ => {}
        }
    }

    result
}

/// A resolved copy/move operation (source row -> target position).
struct CopyMove {
    source: usize,
    target: usize,
    before: bool,
    is_move: bool,
}

/// Whether a parsed line-command kind is in the host's enabled set. Only the
/// subset the shared editors use (I / D / C / M + their A/B targets) is honoured;
/// anything else is treated as not enabled.
fn command_enabled(kind: &LineCommandKind, set: &LineCommandSet) -> bool {
    match kind {
        LineCommandKind::Insert => set.insert,
        LineCommandKind::Delete => set.delete,
        LineCommandKind::Copy => set.copy,
        LineCommandKind::Move => set.move_,
        // Targets are enabled whenever a source command that needs them is.
        LineCommandKind::After | LineCommandKind::Before => set.copy || set.move_,
        _ => false,
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::ispf_table_editor::state::{
        ColumnSpec, CompareMode, FilterState, IspfTableHost, LineCommandSet,
    };

    /// A minimal stub row: a value plus a transient Act cell.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    struct StubRow {
        value: String,
        act: String,
    }

    fn r(value: &str, act: &str) -> StubRow {
        StubRow {
            value: value.to_string(),
            act: act.to_string(),
        }
    }

    /// A stub host enabling I / D / C / SORT (the Alias set) over StubRow.
    struct StubHost {
        columns: Vec<ColumnSpec>,
    }

    impl StubHost {
        fn new() -> Self {
            Self {
                columns: vec![
                    ColumnSpec::display("act", "Act"),
                    ColumnSpec::filterable("value", "Value", CompareMode::ContainsIgnoreCase)
                        .with_sortable(),
                ],
            }
        }
    }

    impl IspfTableHost for StubHost {
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
        fn filter_matches(&self, row: &StubRow, filter: &FilterState) -> bool {
            match filter.get("value") {
                Some(v) => row.value.to_uppercase().contains(&v.to_uppercase()),
                None => true,
            }
        }
        fn sort_key(&self, row: &StubRow, _column: super::super::state::ColumnId) -> String {
            row.value.to_uppercase()
        }
    }

    // Validates: ispf-table-editor Requirement 4.2 -- `I` inserts a blank row
    // AFTER the row and clears the Act cell.
    #[test]
    fn insert_adds_blank_after_and_clears_act() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "I"), r("B", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert!(res.notices.is_empty(), "clean apply: {:?}", res.notices);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].value, "A");
        assert_eq!(rows[1].value, "", "a blank row is inserted after A");
        assert_eq!(rows[2].value, "B");
        assert_eq!(rows[0].act, "", "the I cell is cleared");
    }

    // Validates: ispf-table-editor Requirement 4.2 -- `D` removes the row.
    #[test]
    fn delete_removes_row() {
        let host = StubHost::new();
        let mut rows = vec![r("A", ""), r("B", "D"), r("C", "")];
        apply_line_commands(&host, &mut rows);
        assert_eq!(
            rows.iter().map(|x| x.value.clone()).collect::<Vec<_>>(),
            vec!["A".to_string(), "C".to_string()]
        );
    }

    // Validates: ispf-table-editor Requirement 4.3 -- a mixed I/D batch keeps
    // index integrity (processed by descending index): deleting row 0 and
    // inserting after row 2 both land correctly.
    #[test]
    fn mixed_insert_delete_keeps_index_integrity() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "D"), r("B", ""), r("C", "I")];
        apply_line_commands(&host, &mut rows);
        // D removes A; I inserts a blank after C.
        assert_eq!(
            rows.iter().map(|x| x.value.clone()).collect::<Vec<_>>(),
            vec!["B".to_string(), "C".to_string(), "".to_string()]
        );
    }

    // Validates: ispf-table-editor Requirement 4.2, 4.4 -- `C` + `A` copies the
    // source row AFTER the target.
    #[test]
    fn copy_after_target() {
        let host = StubHost::new();
        let mut rows = vec![r("SRC", "C"), r("T", "A"), r("Z", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert!(res.notices.is_empty(), "clean copy: {:?}", res.notices);
        assert_eq!(
            rows.iter().map(|x| x.value.clone()).collect::<Vec<_>>(),
            vec![
                "SRC".to_string(),
                "T".to_string(),
                "SRC".to_string(),
                "Z".to_string()
            ],
            "the copy lands after the target"
        );
    }

    // Validates: ispf-table-editor Requirement 4.2, 4.4 -- `C` + `B` copies the
    // source row BEFORE the target.
    #[test]
    fn copy_before_target() {
        let host = StubHost::new();
        let mut rows = vec![r("SRC", "C"), r("T", "B")];
        apply_line_commands(&host, &mut rows);
        assert_eq!(
            rows.iter().map(|x| x.value.clone()).collect::<Vec<_>>(),
            vec!["SRC".to_string(), "SRC".to_string(), "T".to_string()],
            "the copy lands before the target"
        );
    }

    // Validates: ispf-table-editor Requirement 4.4 -- an unresolved `C` source
    // (no A/B target) is RETAINED (Act cell not cleared) with a notice.
    #[test]
    fn unresolved_copy_source_is_retained() {
        let host = StubHost::new();
        let mut rows = vec![r("SRC", "C"), r("T", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert_eq!(rows.len(), 2, "no row added for an unresolved source");
        assert_eq!(rows[0].act, "C", "the unresolved source is retained");
        assert!(
            res.notices.iter().any(|n| n.contains("pending a target")),
            "a non-blocking notice is surfaced: {:?}",
            res.notices
        );
    }

    // Validates: ispf-table-editor Requirement 4.5 -- an unrecognised Act letter
    // is cleared and ignored with a notice; the row list is unchanged.
    #[test]
    fn unrecognised_letter_is_cleared_and_ignored() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "Z"), r("B", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert_eq!(rows.len(), 2, "the row list is unchanged");
        assert_eq!(rows[0].act, "", "the stray letter is cleared");
        assert!(
            res.notices.iter().any(|n| n.contains("unrecognised")),
            "a notice is surfaced: {:?}",
            res.notices
        );
    }

    // Validates: command-configurator Requirement 7.4 -- `M` (Move) is NOT in
    // the alias set, so a typed `M` is cleared and ignored (not enabled).
    #[test]
    fn move_not_enabled_is_ignored() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "M"), r("B", "A")];
        let res = apply_line_commands(&host, &mut rows);
        assert_eq!(rows.len(), 2, "Move disabled -> no structural change");
        assert_eq!(rows[0].act, "", "the disabled M is cleared");
        assert!(
            res.notices.iter().any(|n| n.contains("not enabled")),
            "a not-enabled notice is surfaced: {:?}",
            res.notices
        );
    }
}
