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

use super::block_apply::{resolve_blocks, BlockMarker};
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
    pub(super) fn push(&mut self, notice: impl Into<String>) {
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
    let mut blocks: Vec<BlockMarker> = Vec::new();
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
        match classify(&parsed.kind) {
            LineCommandCategory::Target => {
                if set.copy || set.move_ {
                    targets.push((index, parsed.kind));
                } else {
                    result.push(format!("ignored line command '{act}' (not enabled)"));
                    clear.push(index);
                }
            }
            // A doubled block marker (RR/DD/CC/MM). When the host opts in to
            // blocks it is collected for the block pass; otherwise it is cleared
            // and ignored with a non-blocking notice (Requirement 9.5, 4.5).
            LineCommandCategory::Block if set.blocks && block_enabled(&parsed.kind) => {
                blocks.push(BlockMarker {
                    index,
                    kind: parsed.kind,
                });
            }
            LineCommandCategory::Block => {
                result.push(format!("ignored block line command '{act}' (not enabled)"));
                clear.push(index);
            }
            LineCommandCategory::Immediate | LineCommandCategory::Source => {
                if command_enabled(&parsed.kind, &set) {
                    entries.push(Entry {
                        index,
                        kind: parsed.kind,
                    });
                } else {
                    result.push(format!("ignored line command '{act}' (not enabled)"));
                    clear.push(index);
                }
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

    // --- 2b. Resolve DOUBLED block markers (RR/DD/CC/MM) --------------------
    // Blocks are paired via the `ff-line-commands` engine; CC/MM consume the
    // same A/B targets as a single C/M (so `consumed_targets` is shared). A
    // malformed/overlapping block is retained with a non-blocking notice and
    // does not abort the rest of the batch (Requirement 9.3, 9.4).
    let block_plan = resolve_blocks::<H>(
        rows,
        &set,
        &blocks,
        &targets,
        &mut consumed_targets,
        &mut result,
    );

    // A target with no source is a harmless no-op: clear it with a notice.
    for (t_index, _) in &targets {
        if !consumed_targets.contains(t_index) {
            result.push("ignored target (A/B) with no copy/move source".to_string());
            host.clear_act(&mut rows[*t_index]);
        }
    }

    // --- 3. Reconcile every structural op against STABLE original indices ---
    // Every structural edit is expressed relative to an ORIGINAL row index as a
    // per-row plan: rows to insert BEFORE it, rows to insert AFTER it, and whether
    // the row itself is removed. The output list is then rebuilt by walking the
    // original rows IN ORDER and emitting [before] + [row?] + [after] for each.
    // This rebuild-by-walk model is immune to index shifting, so a C/A batched
    // with an I, D, or R lands on the right rows regardless of their relative
    // position (ispf-table-editor Requirement 4.3; command-configurator Req 7.6).
    // Insert payloads (copy clones, repeat clones, blank rows) are snapshotted
    // NOW, while the original indices are still valid.
    let len = rows.len();
    let mut before: Vec<Vec<H::Row>> = (0..len).map(|_| Vec::new()).collect();
    let mut after: Vec<Vec<H::Row>> = (0..len).map(|_| Vec::new()).collect();
    let mut removed: Vec<bool> = vec![false; len];

    for op in &copy_ops {
        let mut new_row = host.clone_row(&rows[op.source]);
        host.clear_act(&mut new_row);
        if op.before {
            before[op.target].push(new_row);
        } else {
            after[op.target].push(new_row);
        }
        host.clear_act(&mut rows[op.source]);
        host.clear_act(&mut rows[op.target]);
        if op.is_move {
            removed[op.source] = true;
        }
    }

    // Fold the resolved block plan into the same before/after/removed model. The
    // clones are snapshotted NOW while the original indices are valid, exactly
    // like the single-command copy/repeat path (Requirement 9.1, 9.3).
    for insert in &block_plan.inserts {
        for &src in &insert.rows {
            let mut clone = host.clone_row(&rows[src]);
            host.clear_act(&mut clone);
            if insert.before {
                before[insert.target].push(clone);
            } else {
                after[insert.target].push(clone);
            }
        }
    }
    for &i in &block_plan.removed {
        removed[i] = true;
    }
    for &i in &block_plan.clear {
        host.clear_act(&mut rows[i]);
    }

    for entry in entries
        .iter()
        .filter(|e| classify(&e.kind) == LineCommandCategory::Immediate)
    {
        match &entry.kind {
            LineCommandKind::Insert => {
                host.clear_act(&mut rows[entry.index]);
                after[entry.index].push(host.blank_row());
            }
            LineCommandKind::Delete => {
                host.clear_act(&mut rows[entry.index]);
                removed[entry.index] = true;
            }
            // `R` / `Rn` repeat: clone the row `count` times immediately after it
            // (a single-row Copy alias, Req 7.6). The clone carries the field
            // values with the Act cell cleared, like a copy.
            LineCommandKind::Repeat | LineCommandKind::RepeatCount(_) => {
                let count = match &entry.kind {
                    LineCommandKind::RepeatCount(n) => (*n).max(1) as usize,
                    _ => 1,
                };
                let mut clone = host.clone_row(&rows[entry.index]);
                host.clear_act(&mut clone);
                host.clear_act(&mut rows[entry.index]);
                for _ in 0..count {
                    after[entry.index].push(host.clone_row(&clone));
                }
            }
            _ => {}
        }
    }

    // Rebuild the row list by walking the originals in order.
    let originals = std::mem::take(rows);
    for (index, row) in originals.into_iter().enumerate() {
        for b in std::mem::take(&mut before[index]) {
            rows.push(b);
        }
        if !removed[index] {
            rows.push(row);
        }
        for a in std::mem::take(&mut after[index]) {
            rows.push(a);
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
        // `R` / `Rn` repeat: a clone-after Copy alias (command-configurator
        // Req 7.6), enabled by the host's `repeat` flag.
        LineCommandKind::Repeat | LineCommandKind::RepeatCount(_) => set.repeat,
        LineCommandKind::Copy => set.copy,
        LineCommandKind::Move => set.move_,
        // Targets are enabled whenever a source command that needs them is.
        LineCommandKind::After | LineCommandKind::Before => set.copy || set.move_,
        _ => false,
    }
}

/// Whether a parsed BLOCK kind is one the table editor's block set permits. The
/// enabled block set is EXACTLY RR / DD / CC / MM (ispf-table-editor Requirement
/// 9.1); every other `*Block` marker (XX / TT / shift / ... ) is rejected even
/// when a host opts in to blocks. `MM` is additionally gated on the host's
/// `move_` flag inside the block pass (Requirement 7.12), so it is admitted here
/// and dropped-with-notice there when Move is disabled.
fn block_enabled(kind: &LineCommandKind) -> bool {
    matches!(
        kind,
        LineCommandKind::DeleteBlock
            | LineCommandKind::RepeatBlock
            | LineCommandKind::CopyBlock
            | LineCommandKind::MoveBlock
    )
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

    /// A stub host enabling I / D / R / C / SORT (the Alias set) over StubRow.
    /// `move_enabled` adds `M` / `MM` so the CC/MM block tests can exercise the
    /// move-block path a move-enabled host (not Alias) would have.
    struct StubHost {
        columns: Vec<ColumnSpec>,
        move_enabled: bool,
    }

    impl StubHost {
        fn new() -> Self {
            Self {
                columns: vec![
                    ColumnSpec::display("act", "Act"),
                    ColumnSpec::filterable("value", "Value", CompareMode::ContainsIgnoreCase)
                        .with_sortable(),
                ],
                move_enabled: false,
            }
        }

        /// A blocks-enabled AND move-enabled host (blocks on, move_ on), used by
        /// the CC/MM block tests.
        fn with_move() -> Self {
            Self {
                move_enabled: true,
                ..Self::new()
            }
        }
    }

    impl IspfTableHost for StubHost {
        type Row = StubRow;
        fn columns(&self) -> &[ColumnSpec] {
            &self.columns
        }
        fn line_command_set(&self) -> LineCommandSet {
            LineCommandSet {
                move_: self.move_enabled,
                ..LineCommandSet::alias()
            }
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

    // Validates: command-configurator Requirement 7.4, 7.6 -- a single `R`
    // clones the marked row and places the clone AFTER it (a single-row Copy
    // alias), clearing both Act cells. Before the fix, R was not in the enabled
    // set so it was cleared with a "not enabled" notice and NOTHING was cloned.
    #[test]
    fn repeat_clones_row_after_on_non_empty_store() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "R"), r("B", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert!(res.notices.is_empty(), "clean repeat: {:?}", res.notices);
        assert_eq!(
            rows.iter().map(|x| x.value.clone()).collect::<Vec<_>>(),
            vec!["A".to_string(), "A".to_string(), "B".to_string()],
            "the clone lands immediately after the source row"
        );
        assert_eq!(rows[0].act, "", "the R cell is cleared");
        assert_eq!(rows[1].act, "", "the clone's Act cell is clear");
    }

    // Validates: command-configurator Requirement 7.4, 7.6 -- a single `R` on a
    // one-row (empty-store scratch row) store clones that row after itself.
    #[test]
    fn repeat_clones_row_after_on_single_row_store() {
        let host = StubHost::new();
        let mut rows = vec![r("ONLY", "R")];
        apply_line_commands(&host, &mut rows);
        assert_eq!(
            rows.iter().map(|x| x.value.clone()).collect::<Vec<_>>(),
            vec!["ONLY".to_string(), "ONLY".to_string()]
        );
    }

    // Validates: command-configurator Requirement 7.6 -- `Rn` clones the row n
    // times after it.
    #[test]
    fn repeat_count_clones_row_n_times() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "R3"), r("B", "")];
        apply_line_commands(&host, &mut rows);
        assert_eq!(
            rows.iter().map(|x| x.value.clone()).collect::<Vec<_>>(),
            vec![
                "A".to_string(),
                "A".to_string(),
                "A".to_string(),
                "A".to_string(),
                "B".to_string()
            ]
        );
    }

    // Validates: ispf-table-editor Requirement 4.3 -- a batch mixing a copy
    // (C/A) with an immediate `I` resolves indices against STABLE original rows:
    // the I inserts after its own row, not after a row shifted by the copy insert.
    // Before the fix, the copy insert grew the list first and the I was applied
    // against the pre-insert index, landing on the wrong row.
    #[test]
    fn batch_copy_after_plus_insert_resolves_indices() {
        let host = StubHost::new();
        // Row 0 copies after row 1; row 2 inserts a blank after itself.
        let mut rows = vec![r("SRC", "C"), r("T", "A"), r("INS", "I"), r("Z", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert!(res.notices.is_empty(), "clean batch: {:?}", res.notices);
        assert_eq!(
            rows.iter().map(|x| x.value.clone()).collect::<Vec<_>>(),
            vec![
                "SRC".to_string(),
                "T".to_string(),
                "SRC".to_string(), // copy of SRC after T
                "INS".to_string(),
                "".to_string(), // blank inserted after INS
                "Z".to_string(),
            ],
            "the I lands after INS and the copy after T, both on stable indices"
        );
    }

    // Validates: ispf-table-editor Requirement 4.3 -- a batch mixing a copy
    // (C/A) with an immediate `D` deletes the intended row, not a shifted one.
    #[test]
    fn batch_copy_after_plus_delete_resolves_indices() {
        let host = StubHost::new();
        // Copy row 0 after row 1; delete row 2.
        let mut rows = vec![r("SRC", "C"), r("T", "A"), r("DEL", "D"), r("Z", "")];
        apply_line_commands(&host, &mut rows);
        assert_eq!(
            rows.iter().map(|x| x.value.clone()).collect::<Vec<_>>(),
            vec![
                "SRC".to_string(),
                "T".to_string(),
                "SRC".to_string(),
                "Z".to_string(),
            ],
            "DEL is removed and the copy lands after T; Z is untouched"
        );
    }

    // Validates: ispf-table-editor Requirement 4.3, command-configurator Req 7.6
    // -- a batch mixing a copy (C/B) with an `R` resolves correctly: the repeat
    // clones its own row after itself while the copy lands before its target.
    #[test]
    fn batch_copy_before_plus_repeat_resolves_indices() {
        let host = StubHost::new();
        // Row 0 repeats; row 2 copies before row 1 (its target).
        let mut rows = vec![r("REP", "R"), r("T", "B"), r("SRC", "C")];
        apply_line_commands(&host, &mut rows);
        assert_eq!(
            rows.iter().map(|x| x.value.clone()).collect::<Vec<_>>(),
            vec![
                "REP".to_string(),
                "REP".to_string(), // clone after REP
                "SRC".to_string(), // copy before T
                "T".to_string(),
                "SRC".to_string(),
            ]
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

    // === CR-CH-076 block line commands (RR/DD/CC/MM) ======================

    fn values(rows: &[StubRow]) -> Vec<String> {
        rows.iter().map(|x| x.value.clone()).collect()
    }

    // Validates: ispf-table-editor Requirement 9.1 -- a matched DD/DD pair
    // deletes the WHOLE inclusive range (both endpoints and every row between).
    #[test]
    fn block_dd_deletes_inclusive_range() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "DD"), r("B", ""), r("C", "DD"), r("D", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert!(res.notices.is_empty(), "clean DD block: {:?}", res.notices);
        assert_eq!(values(&rows), vec!["D".to_string()]);
    }

    // Validates: ispf-table-editor Requirement 9.1 -- a matched RR/RR pair
    // duplicates EVERY row in the inclusive range, immediately after the block.
    #[test]
    fn block_rr_duplicates_inclusive_range() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "RR"), r("B", ""), r("C", "RR"), r("Z", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert!(res.notices.is_empty(), "clean RR block: {:?}", res.notices);
        assert_eq!(
            values(&rows),
            vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "Z".to_string(),
            ],
            "the whole range is duplicated after the block"
        );
        assert!(rows[0].act.is_empty(), "the RR markers are cleared");
        assert!(rows[2].act.is_empty());
    }

    // Validates: ispf-table-editor Requirement 9.3 -- a CC/CC block + an `A`
    // target copies the whole block AFTER the target row.
    #[test]
    fn block_cc_copies_range_after_target() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "CC"), r("B", "CC"), r("T", "A"), r("Z", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert!(res.notices.is_empty(), "clean CC block: {:?}", res.notices);
        assert_eq!(
            values(&rows),
            vec![
                "A".to_string(),
                "B".to_string(),
                "T".to_string(),
                "A".to_string(),
                "B".to_string(),
                "Z".to_string(),
            ],
            "the block is copied after the target"
        );
    }

    // Validates: ispf-table-editor Requirement 9.3 -- a CC/CC block + a `B`
    // target copies the whole block BEFORE the target row.
    #[test]
    fn block_cc_copies_range_before_target() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "CC"), r("B", "CC"), r("T", "B")];
        apply_line_commands(&host, &mut rows);
        assert_eq!(
            values(&rows),
            vec![
                "A".to_string(),
                "B".to_string(),
                "A".to_string(),
                "B".to_string(),
                "T".to_string(),
            ],
            "the block is copied before the target"
        );
    }

    // Validates: ispf-table-editor Requirement 9.3 -- an MM/MM block + an `A`
    // target MOVES the whole block (removes the source range, re-inserts after
    // the target) on a move-ENABLED host.
    #[test]
    fn block_mm_moves_range_after_target() {
        let host = StubHost::with_move();
        let mut rows = vec![r("A", "MM"), r("B", "MM"), r("T", "A"), r("Z", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert!(res.notices.is_empty(), "clean MM block: {:?}", res.notices);
        assert_eq!(
            values(&rows),
            vec![
                "T".to_string(),
                "A".to_string(),
                "B".to_string(),
                "Z".to_string(),
            ],
            "the block is moved after the target (source range removed)"
        );
    }

    // Validates: ispf-table-editor Requirement 9.1 -- the two markers may be
    // typed in REVERSED order and still form the same inclusive range (the
    // smaller and larger backing index normalise to start/end).
    #[test]
    fn block_dd_reversed_markers_same_range() {
        let host = StubHost::new();
        // The second-typed marker is on the earlier row; order must not matter.
        let mut rows = vec![
            r("A", ""),
            r("B", "DD"),
            r("C", ""),
            r("D", "DD"),
            r("E", ""),
        ];
        apply_line_commands(&host, &mut rows);
        assert_eq!(
            values(&rows),
            vec!["A".to_string(), "E".to_string()],
            "rows B..=D are deleted regardless of marker typing order"
        );
    }

    // Validates: ispf-table-editor Requirement 9.4 -- a LONE DD marker makes NO
    // structural change, retains the marker (act stays 'DD'), and surfaces a
    // non-blocking notice, while a well-formed I in the SAME batch still applies.
    #[test]
    fn block_lone_dd_retained_notice_batch_continues() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "DD"), r("B", "I"), r("C", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert_eq!(
            values(&rows),
            vec![
                "A".to_string(),
                "B".to_string(),
                "".to_string(),
                "C".to_string(),
            ],
            "the lone DD makes no change; the I still inserts after B"
        );
        assert_eq!(rows[0].act, "DD", "the lone DD marker is retained");
        assert!(
            res.notices.iter().any(|n| n.contains("not closed")),
            "a non-blocking 'block not closed' notice is surfaced: {:?}",
            res.notices
        );
    }

    // Validates: ispf-table-editor Requirement 9.4 -- THREE DD markers are
    // malformed (too many): no structural change, the markers are retained, and
    // a non-blocking notice is surfaced.
    #[test]
    fn block_too_many_dd_markers_no_change() {
        let host = StubHost::new();
        let mut rows = vec![r("A", "DD"), r("B", "DD"), r("C", "DD"), r("D", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert_eq!(
            values(&rows),
            vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
            ],
            "too many markers -> no structural change"
        );
        assert_eq!(rows[0].act, "DD", "the markers are retained");
        assert_eq!(rows[2].act, "DD");
        assert!(
            res.notices.iter().any(|n| n.contains("not closed")),
            "a non-blocking notice is surfaced: {:?}",
            res.notices
        );
    }

    // Validates: ispf-table-editor Requirement 9.4 -- overlapping DD and RR
    // ranges are rejected with a notice and NEITHER is applied.
    #[test]
    fn block_overlapping_dd_and_rr_rejected() {
        let host = StubHost::new();
        // DD spans rows 0..=2, RR spans rows 1..=3 -> overlap.
        let mut rows = vec![
            r("A", "DD"),
            r("B", "RR"),
            r("C", "DD"),
            r("D", "RR"),
            r("E", ""),
        ];
        let res = apply_line_commands(&host, &mut rows);
        assert_eq!(
            values(&rows),
            vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
                "E".to_string(),
            ],
            "overlapping blocks -> neither applied"
        );
        assert!(
            res.notices.iter().any(|n| n.contains("overlapping")),
            "an overlap notice is surfaced: {:?}",
            res.notices
        );
    }

    // Validates: ispf-table-editor Requirement 9.2 -- the single `R` vs the
    // doubled `RR` distinction is preserved: a single R repeats exactly ONE row
    // while a matched RR/RR pair repeats the whole range, in the SAME batch.
    #[test]
    fn single_r_vs_block_rr_distinct() {
        let host = StubHost::new();
        // Row 0: single R (repeat one). Rows 2..=3: RR block (repeat range).
        let mut rows = vec![r("A", "R"), r("B", ""), r("C", "RR"), r("D", "RR")];
        let res = apply_line_commands(&host, &mut rows);
        assert!(
            res.notices.is_empty(),
            "clean mixed batch: {:?}",
            res.notices
        );
        assert_eq!(
            values(&rows),
            vec![
                "A".to_string(),
                "A".to_string(), // single R clone of A
                "B".to_string(),
                "C".to_string(),
                "D".to_string(),
                "C".to_string(), // RR range duplicated after the block
                "D".to_string(),
            ],
            "single R repeats one row; RR/RR repeats the range"
        );
    }

    // Validates: ispf-table-editor Requirement 9.5 -- a host that does NOT opt in
    // to blocks clears and ignores a doubled marker with a non-blocking notice
    // (unchanged Requirement 4.5 behaviour).
    #[test]
    fn block_marker_ignored_when_host_opts_out() {
        struct NoBlockHost {
            columns: Vec<ColumnSpec>,
        }
        impl IspfTableHost for NoBlockHost {
            type Row = StubRow;
            fn columns(&self) -> &[ColumnSpec] {
                &self.columns
            }
            fn line_command_set(&self) -> LineCommandSet {
                LineCommandSet {
                    blocks: false,
                    ..LineCommandSet::alias()
                }
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
            fn sort_key(&self, row: &StubRow, _column: super::super::state::ColumnId) -> String {
                row.value.to_uppercase()
            }
        }
        let host = NoBlockHost {
            columns: vec![ColumnSpec::display("act", "Act")],
        };
        let mut rows = vec![r("A", "DD"), r("B", "DD")];
        let res = apply_line_commands(&host, &mut rows);
        assert_eq!(rows.len(), 2, "blocks off -> no structural change");
        assert!(rows[0].act.is_empty(), "the doubled marker is cleared");
        assert!(
            res.notices.iter().any(|n| n.contains("not enabled")),
            "a not-enabled notice is surfaced: {:?}",
            res.notices
        );
    }

    // Validates: ispf-table-editor Requirement 7.12 -- an MM block on a
    // move-DISABLED host (the Alias set: blocks on, move_ off) is dropped with a
    // not-enabled notice; its markers are retained and no row moves.
    #[test]
    fn block_mm_disabled_when_move_off() {
        let host = StubHost::new(); // move_enabled = false (the Alias shape)
        let mut rows = vec![r("A", "MM"), r("B", "MM"), r("T", "A"), r("Z", "")];
        let res = apply_line_commands(&host, &mut rows);
        assert_eq!(
            values(&rows),
            vec![
                "A".to_string(),
                "B".to_string(),
                "T".to_string(),
                "Z".to_string(),
            ],
            "MM disabled -> no structural change"
        );
        assert!(
            res.notices.iter().any(|n| n.contains("not enabled")),
            "a not-enabled notice is surfaced: {:?}",
            res.notices
        );
    }
}
