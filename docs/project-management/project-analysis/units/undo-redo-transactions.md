# Analysis Record: undo-redo-transactions

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-undo-redo`
- **Spec folder**: `docs/specs/undo-redo-transactions/`
- **Analysed**: Wave 1, task W1.1 (CR-NR-057/058 re-baseline)
- **Verdict**: COMPLETE (159/159 tasks `[x]`, TCR PASS). SPLIT CANDIDATE (19 reqs,
  553 req lines). One logging gap (Req 3.5 orphaned-transaction WARNING missing --
  PA-LOG-006), one 400-cap refactor (manager.rs 600 -- PA-STD-007), one
  crate-name-drift doc note (PA-DOC-003).
- **CR-NR-057/058 impact**: NONE (undo-redo specs untouched). Re-verified.

---

## 1. Scope summary

`ff-undo-redo` bridges command-framework (produces Undo_Records) and
document-model (receives reversed/re-applied edits). 19 requirements: undo/redo
stacks, transaction boundaries (nesting/rollback/orphan detection), execution
(UNDO/REDO n), save-point + dirty flag + detach point, coalescing, bulk
transactions (Rule vs Index), recovery files, selection history, non-undoable
operations, per-document undo, tentative actions (IME), container actions,
logical record identity, UNDO/REDO command integration, history validation,
scrap-stack text storage, crate API + GUI independence, SETUNDO/RECOVERY (P2).

553 req lines, 19 requirements, single backing crate.

## 2. Split candidacy -- CANDIDATE

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 553 lines AND 19 reqs | YES (both) |
| 3+ distinct responsibilities | core stacks/exec; save-point/dirty; coalescing; bulk+record-identity; recovery; selection history; tentative/container; scrap; command integration | YES |
| 2+ crates | one crate | No |
| low-cohesion clusters | recovery (Req 8) and command integration (15,19) most separable | Weak-Yes |
| file-size pressure | `manager.rs` 600 non-test over cap | Yes |

3+ criteria. **SPLIT CANDIDATE.** Recorded PA-SPLIT-004 (proposal only). Crate
split NOT recommended (stacks/coalescing/scrap/save-point interlock through the
manager); a SPEC split for traceability (core vs recovery/tentative/container;
command integration into command-semantics/function-keys) is the useful action.
Owner-gated.

## 3. Consistency / conflict

- Public types owned here (Transaction, EditOperation, UndoStack, RedoStack,
  SavePoint, ScrapStack, UndoConfig, UndoManager, RecoveryPayload, SelectionState,
  LogicalRecordId, TentativeAction, ContainerAction) -- sole owner. Added to
  consistency-matrix.
- **PA-DOC-003 (crate-name drift)**: spec Introduction + Req 18.1 say "`ff-undo`
  crate"; the actual crate is `ff-undo-redo`. Reconcile the spec.
- Consumer/contract relationships correct: command-framework Req 4 (this crate
  owns the stacks its Undo_Records push onto); document-model (edit ops applied/
  reversed on the gap buffer); edit-operations (op types); configuration-system
  (`editor.undo.*`, `editor.recovery.*`); file-operations (save-point + recovery
  cleanup). No conflict here.
- **PA-CONFLICT-002 (carried, resolves at W1.5)**: undo-redo defines
  `Transaction`/`EditOperation`/`ScrapStack` (Scintilla model) while
  edit-operations defines `EditorTransaction` (line-snapshot model). Verify at
  edit-operations (W1.5) whether these are bridged or a genuine duplication.
- Recovery I/O (Req 8): recovery.rs serialises to bytes + CRC32 and computes paths
  but delegates the actual file write to the caller -- GUI/IO-independent (no
  FFW-ARCH-001 issue, unlike ff-workflow). Good design (to re-confirm at W1.5).
- SelectionState (Req 9) is a caret/anchor snapshot for restore; verify vs
  edit-operations Selection ownership at W1.5 (PA-WATCH-006).

## 4. Completeness

- Tasks: 159 `[x]`, 0 `[ ]`. TCR `ff-undo-redo` PASS (recording, stacks, recovery);
  Phase CE rows PASS (Req 19 SETUNDO/RECOVERY).
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- `ff-logging` declared; used in config.rs (WARN on `editor.undo.*` clamp /
  negative max_levels -- Req 1.6). That is the only logging.
- **Req 3.5 UNMET (partial gap)**: orphaned-transaction force-close
  (transaction.rs `force_close` :137 / its manager.rs caller) commits the orphaned
  transaction SILENTLY. Req 3.5 mandates "force-close ... WITH A WARNING logged
  via the logging subsystem". The WARN is missing.
- CR-NR-058 relevance: the Req 3.5 WARN is a Retained_Level (always compiled).
  Additional undo/redo dev diagnostics (transaction begin/commit/coalesce) would
  be good `log_debug!` candidates under the new `dev-logging` gate.
- No println!/eprintln!. GUI-independence upheld.
- Logging verdict: **mostly adequate; Req 3.5 orphaned-transaction WARNING
  missing.** PA-LOG-006.

## 6. Findings logged

- **PA-SPLIT-004** (SPLIT PROPOSAL): 19 reqs / 553 lines / 3+ responsibilities.
  SPEC split (core vs recovery/tentative/container; command integration ->
  command-semantics/function-keys). Crate split NOT recommended. Owner-gated.
- **PA-LOG-006** (LOGGING-GAP + Req 3.5 partial violation -- code fix, no gate):
  orphaned-transaction force-close does not emit the mandated WARNING. Add a
  `log_warn!` naming the force-closed transaction at the force-close site.
- **PA-STD-007** (REFACTOR -- 400-line cap): `manager.rs` 600 non-test lines. Split
  by concern (manager_exec / manager_txn; keep manager.rs coordinator). No gate.
- **PA-DOC-003** (TRACKING-FIX): spec says "`ff-undo` crate"; actual is
  `ff-undo-redo`. Reconcile the spec. Doc-only.
- **PA-CONFLICT-002** (carried): two transaction models (this crate vs
  edit-operations `EditorTransaction`); resolve at W1.5.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-undo-redo contributes matches
  (doc em-dashes, separators). Rolled into project-wide PA-LOG-001.
