# Analysis Record: undo-redo-transactions

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-undo-redo`
- **Spec folder**: `docs/specs/undo-redo-transactions/`
- **Analysed**: Wave 1, task W1.1
- **Verdict**: COMPLETE (159/159 tasks `[x]`, TCR PASS). SPLIT CANDIDATE (19 reqs,
  553 req lines). One logging gap (Req 3.5 orphaned-transaction WARNING missing),
  one 400-cap refactor (manager.rs 600), one crate-name-drift doc note.

---

## 1. Scope summary

`ff-undo-redo` is the undo/redo transaction system bridging command-framework
(produces Undo_Records) and document-model (receives reversed/re-applied edits).
19 requirements:

- Req 1 undo stack; Req 2 redo stack; Req 3 transaction boundaries (nesting,
  rollback, orphan detection); Req 4 undo/redo execution (UNDO/REDO n);
  Req 5 save point + dirty flag + detach point; Req 6 coalescing; Req 7 bulk
  transactions (Rule vs Index); Req 8 recovery files; Req 9 selection history;
  Req 10 non-undoable operations; Req 11 per-document undo; Req 12 tentative
  actions (IME); Req 13 container actions (plugin state); Req 14 logical record
  identity; Req 15 UNDO/REDO command integration; Req 16 history validation/
  integrity; Req 17 scrap stack text storage; Req 18 crate API + GUI
  independence; Req 19 SETUNDO/RECOVERY commands (P2).

553 req lines, 19 requirements, single backing crate.

## 2. Split candidacy -- CANDIDATE

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 553 lines AND 19 reqs | YES (both) |
| 3+ distinct responsibilities | core stacks/exec (1-4,11); save-point/dirty (5); coalescing (6); bulk+record-identity (7,14); recovery (8); selection history (9); tentative/container (12,13); scrap storage (17); command integration (15,19) | YES |
| 2+ crates | one crate `ff-undo-redo` | No |
| low-cohesion clusters | recovery (8) and command integration (15,19) are the most separable; core stacks/coalescing/scrap are tightly coupled | Weak-Yes |
| file-size pressure | `manager.rs` 600 non-test over cap | Yes |

3+ criteria (>12 reqs AND >350 lines; 3+ responsibilities; file-size).
**SPLIT CANDIDATE.** Recorded as PA-SPLIT-004 (proposal only).

### Proposed SPEC split (proposal, owner-gated)

The engine is cohesive; a CRATE split is NOT recommended (stacks, coalescing,
scrap, save-point all interlock through the manager). But the 19-req SPEC could
be split for traceability:
- `undo-redo-transactions` (core): Req 1-7, 9-11, 14, 16-18 (stacks, boundaries,
  execution, save-point, coalescing, bulk, selection, non-undoable, per-doc,
  record identity, validation, scrap, API).
- `undo-recovery` (new): Req 8 recovery files + Req 12 tentative + Req 13
  container (the "peripheral persistence/extension" band) -- OR keep 12/13 in
  core and split only Req 8.
- Req 15/19 (command integration, SETUNDO/RECOVERY) could fold into
  command-semantics / function-keys specs.
Proposal only; owner approval + own gate required.

## 3. Consistency / conflict

- Public types owned here (Transaction, EditOperation, UndoStack, RedoStack,
  SavePoint, ScrapStack, UndoConfig, UndoManager, UndoManagerTrait, RecoveryPayload,
  SelectionState, LogicalRecordId, TentativeAction, ContainerAction) -- sole
  owner `ff-undo-redo`. Added to consistency-matrix.
- **Crate-name drift (PA-DOC-003)**: the spec Introduction says "`ff-undo` crate"
  and design principles reference `ff-undo`, but the actual crate is
  `ff-undo-redo`. Cosmetic doc drift (same class as PA-DOC-001). Reconcile the
  spec to `ff-undo-redo`.
- Consumer/contract relationships (correct direction): command-framework Req 4
  (undo/redo integration -- this crate owns the stacks its Undo_Records push
  onto; consistent with command-framework W0.7 UndoRecord bridge); document-model
  (edit ops applied/reversed on the gap buffer -- W0.9); edit-operations (op
  types); configuration-system (`editor.undo.*` + `editor.recovery.*` keys);
  file-operations (save-point + recovery cleanup); encoding-and-characters (n/a).
  No conflict.
- SelectionState here vs caret-and-selection (Wave 1, not yet analysed): Req 9
  stores caret/anchor/virtual-space for restore. WATCH when caret-and-selection
  is analysed (W1.3) that SelectionState ownership is not duplicated -- record
  PA-WATCH-006.

## 4. Completeness

- Tasks: 159 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-undo-redo` row PASS (transaction recording, undo/redo stacks, recovery
  files); Phase CE rows PASS (Req 19 SETUNDO/RECOVERY).
- Recovery I/O (Req 8) design VERIFIED CORRECT: recovery.rs serializes state to
  `Vec<u8>` with a magic header + version + CRC32 and computes the file path
  (`recovery_path_for`, `unsaved_recovery_path`), but does NOT perform the file
  write itself -- persistence is delegated to the caller (file-operations /
  background-io via VFS). This preserves the crate's GUI/IO-independence (Req 18)
  and avoids the FFW-ARCH-001 issue seen in workflow-engine (PA-WATCH-004). No
  std::fs/tokio::fs in the crate. Good design.
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- log call sites: `ff-logging` declared; used in config.rs (2 WARN sites).
  - Req 1.6 (negative max_levels -> default + config warning) SATISFIED
    (config.rs:76, LogLevel::Warn; `UndoConfig::new` takes i32 precisely to
    detect negatives).
  - coalesce_timeout clamp -> WARN (config.rs:94) -- good extra diagnostic.
- **Req 3.5 UNMET (partial logging gap)**: orphaned-transaction force-close.
  Req 3.5 mandates "force-close the transaction WITH A WARNING logged via the
  logging subsystem". The path `TransactionBuilder::force_close` (transaction.rs)
  and its caller (manager.rs:155) commit the orphaned transaction SILENTLY -- no
  `log_warn!`. The warning half of Req 3.5 is missing.
- No `println!`/`eprintln!`. GUI-independence upheld.
- Logging verdict: **mostly adequate** -- config warnings present; the Req 3.5
  orphaned-transaction WARNING is missing. Logged PA-LOG-006.

## 6. Findings logged

- **PA-SPLIT-004** (SPLIT PROPOSAL): 19 reqs / 553 lines / 3+ responsibilities.
  Proposal to split the SPEC (core vs recovery/tentative/container; command
  integration into command-semantics/function-keys). Crate split NOT recommended.
  Owner approval + own gate.
- **PA-LOG-006** (LOGGING-GAP + Req 3.5 partial violation -- code fix, no gate):
  orphaned-transaction force-close (transaction.rs `force_close` / manager.rs:155)
  does not emit the WARNING mandated by Req 3.5. Add a `log_warn!` naming the
  force-closed transaction at the force-close site. Criterion exists -- code-mode.
- **PA-STD-005** (REFACTOR -- 400-line cap): `manager.rs` is 600 non-test lines,
  over the cap. Split by concern (e.g. `manager_exec.rs` undo/redo execution,
  `manager_txn.rs` transaction lifecycle; keep `manager.rs` as coordinator).
  REFACTOR, no gate.
- **PA-DOC-003** (TRACKING-FIX): spec Introduction / design principles say
  "`ff-undo` crate" but the actual crate is `ff-undo-redo`. Reconcile the spec to
  the real crate name. Doc-only, no gate.
- **PA-WATCH-006** (consistency watch): SelectionState (Req 9) restore semantics
  vs caret-and-selection (W1.3). Verify no duplicate SelectionState ownership
  when caret-and-selection is analysed.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-undo-redo contributes 60
  matches (doc-comment em-dashes, separators). Rolled into project-wide PA-LOG-001.
