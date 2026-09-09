# Analysis Record: edit-operations

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-edit-operations`
- **Spec folder**: `docs/specs/edit-operations/`
- **Analysed**: Wave 1, task W1.5
- **Verdict**: COMPLETE (280/280 tasks `[x]`, TCR PASS). SPLIT CANDIDATE
  (17 reqs, 553 req lines). PA-WATCH-006 RESOLVED and upgraded to a real
  architectural conflict: TWO independent, unbridged transaction models
  (PA-CONFLICT-002). Zero-log defensible.

---

## 1. Scope summary

`ff-edit-operations` owns the LOGICAL editing model: character insert/overstrike/
delete, the selection model (SelectionPosition/SelectionRange/Selection
container), multi-caret coordination, rectangular selection, BOUNDS, line
manipulation, transaction UNITS, clipboard edit-side semantics, CAPS/profile.
17 requirements (Req 1-17; incl. Req 16 CAPS profile, Req 17 edit profile
persistence).

553 req lines, 17 requirements, single backing crate. It explicitly scopes OUT
(to other crates): undo/redo stack mechanics (undo-redo-transactions), caret
visuals (caret-and-selection), navigation (navigation-commands), buffer
(document-model), clipboard system access (clipboard-operations).

## 2. Split candidacy -- CANDIDATE

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 553 lines AND 17 reqs | YES (both) |
| 3+ distinct responsibilities | text mutation (insert/overstrike/delete, Req 1-5); selection model (Req 6-7); multi-caret (Req 8); rectangular (Req 9); clipboard edit-side (Req 10); transaction units (Req 11); CAPS/profile (Req 16-17) | YES |
| 2+ crates | one crate `ff-edit-operations` | No |
| low-cohesion clusters | selection model (6-9) vs text mutation (1-5) vs profile/CAPS (16-17) are separable bands | Weak-Yes |
| file-size pressure | largest non-test profile.rs 275, selection.rs 266 -- ALL under 400 cap | No |

3 criteria (>12 reqs AND >350 lines; 3+ responsibilities). **SPLIT CANDIDATE.**
Recorded as PA-SPLIT-005 (proposal only). Note: the crate is well-factored at
the FILE level (no oversized files), so a crate split is lower-value than the
configuration-system/undo-redo cases; a SPEC split for traceability is the more
likely useful action. Owner-gated.

## 3. Consistency / conflict -- PA-WATCH-006 RESOLVED -> PA-CONFLICT-002

Ownership of the selection/transaction concepts is now fully mapped across the
three Wave-1 crates that touch them:

| Concept | Owner | Notes |
|---------|-------|-------|
| `SelectionPosition` / `SelectionRange` / `Selection` container / multi-caret | edit-operations (HERE) | LOGICAL model. VERIFIED defined here (position.rs/range.rs/selection.rs). |
| Visual caret/selection rendering | caret-and-selection (W1.3) | consumes the logical model; no duplication (confirmed W1.3). |
| Undo/redo stack mechanics, coalescing, save-point, scrap | undo-redo-transactions (W1.1) | owns `Transaction`/`EditOperation`/`ScrapStack`/`SelectionState`. |

- **Selection model ownership: CLEAN.** caret-and-selection does not duplicate it
  (W1.3); edit-operations is the sole owner. PA-WATCH-006's selection half is
  RESOLVED with no conflict.
- **PA-CONFLICT-002 (NEW -- transaction-model duplication):** there are TWO
  independent transaction-unit representations that are NOT bridged:
  1. edit-operations `EditorTransaction` (transaction.rs) -- before/after
     LineSnapshot pairs (FFE-MVP-3 model). Its doc-comment says "the actual
     TransactionStack mechanics are in ff-undo-redo-transactions".
  2. undo-redo `Transaction` + `EditOperation` + `ScrapStack` (Scintilla model).
  VERIFIED: `ff-edit-operations` Cargo.toml does NOT depend on `ff-undo-redo`,
  and `ff-undo-redo` src has ZERO references to `ff_edit_operations` /
  `EditorTransaction`. So edit-operations' EditorTransaction is NOT fed into
  undo-redo's stack -- the stated delegation ("mechanics are in ff-undo-redo")
  is not wired. Two parallel undo-unit models coexist. edit-operations Req 11
  even describes UNDO/REDO on its own TransactionStack, overlapping
  undo-redo-transactions Req 1-4. Owner decision: (a) make undo-redo consume
  `EditorTransaction` (single model), or (b) declare one model canonical and
  mark the other as the crate-internal/vestigial one, documenting the boundary.
  Likely resolved by whichever the ff-desktop shell actually uses at runtime --
  check during Wave 4 (file-operations / shell). Surface for owner.
- Consumer relationships otherwise correct: document-model (buffer),
  encoding-and-characters (word classification for double-click / word ops,
  Req 6.15), clipboard-operations (Req 10 system access), command-framework
  (edit commands). No other conflict.
- SelectionState (undo-redo Req 9) vs Selection (here): undo-redo's SelectionState
  is a caret/anchor SNAPSHOT for restore; it does not import edit-operations'
  Selection (grep 0) -- it stores its own snapshot representation. Consistent with
  PA-CONFLICT-002 (the two crates are decoupled); no additional conflict beyond it.

## 4. Completeness

- Tasks: 280 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-edit-operations` row PASS (edit bounds, region operations, batch edits;
  Req 16 CAPS profile rows PASS).
- Well-factored at file level (no file over the 400 cap).
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` IS a declared dependency (wired, unused).
- Defensible: logical editing model; edits either succeed, no-op (e.g. transpose
  on first line, Req 5.8), or return an error to the caller (bounds violations
  via `EditBounds`, protected ranges). No requirement mandates a log record here.
  Same class as the other Wave-1 pure models.
- No `println!`/`eprintln!`. GUI-independence upheld.
- Logging verdict: **adequate (defensible zero-log for a logical model)**.

## 6. Findings logged

- **PA-CONFLICT-002** (CONSISTENCY -- architectural; PA-WATCH-006 resolved):
  edit-operations `EditorTransaction` (line-snapshot model) and
  undo-redo-transactions `Transaction`/`EditOperation` (scrap-stack model) are
  two independent, unbridged undo-unit representations -- no dependency either
  way, no reference either way -- despite edit-operations' claim that "the actual
  TransactionStack mechanics are in ff-undo-redo-transactions" and its own Req 11
  UNDO/REDO. Owner decision: unify (undo-redo consumes EditorTransaction) OR
  declare one canonical and document the other. Confirm which the shell uses at
  Wave 4. Surface for owner; no gate for the record.
- **PA-SPLIT-005** (SPLIT PROPOSAL): 17 reqs / 553 lines / 3+ responsibilities.
  Proposal: SPEC split (text-mutation Req 1-5 core; selection/multi-caret/
  rectangular Req 6-9; profile/CAPS Req 16-17). Crate split lower value (files are
  well-sized). Owner approval + own gate.
- **PA-WATCH-006 RESOLVED**: selection-model ownership is clean (edit-operations
  sole owner; caret-and-selection consumes; undo-redo snapshots separately). The
  transaction-model concern is carried forward as PA-CONFLICT-002.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-edit-operations contributes
  40 matches (doc-comment arrows/em-dashes e.g. "uppercase -> lowercase" prose,
  separators). Rolled into project-wide PA-LOG-001.
