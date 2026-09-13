# Analysis Record: edit-operations

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-edit-operations`
- **Spec folder**: `docs/specs/edit-operations/`
- **Analysed**: Wave 1, task W1.5 (CR-NR-057/058 re-baseline)
- **Verdict**: COMPLETE (280/280 tasks `[x]`, TCR PASS). SPLIT CANDIDATE (17 reqs,
  553 req lines). PA-WATCH-006 RESOLVED (selection-model ownership clean).
  PA-CONFLICT-002 confirmed (two unbridged transaction models; EditorTransaction
  de-facto canonical). Zero-log defensible.
- **CR-NR-057/058 impact**: NONE (edit-operations specs untouched). Re-verified.

---

## 1. Scope summary

`ff-edit-operations` owns the LOGICAL editing model: character insert/overstrike/
delete, the selection model (SelectionPosition/SelectionRange/Selection),
multi-caret coordination, rectangular selection, BOUNDS, line manipulation,
transaction UNITS, clipboard edit-side semantics, CAPS/profile. 17 requirements.
It explicitly scopes OUT (to other crates): undo/redo stack mechanics
(undo-redo-transactions), caret visuals (caret-and-selection), navigation
(navigation-commands), buffer (document-model), clipboard system access
(clipboard-operations).

553 req lines, 17 requirements, single backing crate. Files well-sized (none over
the 400 cap: profile.rs 275, selection.rs 266 largest non-test).

## 2. Split candidacy -- CANDIDATE

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 553 lines AND 17 reqs | YES (both) |
| 3+ distinct responsibilities | text mutation (Req 1-5); selection model (6-7); multi-caret (8); rectangular (9); clipboard edit-side (10); transaction units (11); CAPS/profile (16-17) | YES |
| 2+ crates | one crate | No |
| low-cohesion clusters | selection (6-9) vs text mutation (1-5) vs profile/CAPS (16-17) separable | Weak-Yes |
| file-size pressure | none over cap | No |

3 criteria (>12 reqs AND >350 lines; 3+ responsibilities). **SPLIT CANDIDATE**
(PA-SPLIT-005, proposal only). SPEC split for traceability (text-mutation core;
selection/multi-caret/rectangular; profile/CAPS). Crate split lower value (files
well-sized). Owner-gated.

## 3. Consistency / conflict -- PA-WATCH-006 RESOLVED, PA-CONFLICT-002 confirmed

Ownership across the three Wave-1 crates that touch selection/transactions:

| Concept | Owner | Notes |
|---------|-------|-------|
| `SelectionPosition` / `SelectionRange` / `Selection` / multi-caret | edit-operations (HERE) | VERIFIED defined here (position/range/selection.rs). |
| Visual caret/selection rendering | caret-and-selection (W1.3) | consumes the model; no duplication (W1.3 confirmed). |
| Undo/redo stack mechanics, scrap, save-point | undo-redo-transactions (W1.1) | owns Transaction/EditOperation/ScrapStack/SelectionState. |

- **PA-WATCH-006 RESOLVED**: edit-operations is the SOLE owner of the logical
  selection model; caret-and-selection does not duplicate it (W1.3); undo-redo
  stores its own decoupled SelectionState snapshot. Selection ownership is CLEAN.
- **PA-CONFLICT-002 (confirmed)**: TWO independent, unbridged transaction-unit
  models:
  1. edit-operations `EditorTransaction` (transaction.rs) -- before/after
     LineSnapshot pairs (FFE-MVP-3). transaction.rs:4 claims "the actual
     TransactionStack mechanics are in ff-undo-redo-transactions".
  2. undo-redo `Transaction` + `EditOperation` + `ScrapStack` (Scintilla model).
  VERIFIED: `ff-edit-operations` does NOT depend on `ff-undo-redo`; `ff-undo-redo`
  has ZERO references to `EditorTransaction`/`ff_edit_operations`; the stated
  delegation is NOT wired. DECISIVE EVIDENCE (W1.6-equivalent): ff-line-commands
  produces `EditorTransaction` (execute_delete -> `Result<EditorTransaction>`) for
  all undoable commands and NEVER uses its declared `ff-undo-redo` dep (grep 0).
  CONCLUSION: `EditorTransaction` is the DE-FACTO canonical undo unit at the
  command layer; undo-redo-transactions' parallel model is unwired. Owner
  decision: declare `EditorTransaction` canonical and reconcile/retire undo-redo's
  parallel model, OR wire undo-redo to consume EditorTransaction. Confirm shell
  undo dispatch at Wave 4 (file-operations/shell).
- Other consumers correct: document-model (buffer), encoding-and-characters (word
  classification, Req 6.15), clipboard-operations (Req 10), command-framework.
- SelectionState (undo Req 9) vs Selection (here): undo-redo's SelectionState is a
  caret/anchor snapshot; it does not import edit-operations' Selection -- consistent
  with PA-CONFLICT-002 (the two crates are decoupled). No additional conflict.

## 4. Completeness

- Tasks: 280 `[x]`, 0 `[ ]`. TCR PASS (edit bounds, region operations, batch
  edits; Req 16 CAPS profile rows PASS). Well-factored at file level.
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` declared (wired, unused). DEFENSIBLE: logical
  editing model; edits succeed, no-op (Req 5.8), or return an error to the caller
  (bounds violations via EditBounds, protected ranges). No requirement mandates a
  log. Same class as the other Wave-1 pure models.
- CR-NR-058 relevance: edit/selection operations would be good `log_trace!`/
  `log_debug!` candidates under the new `dev-logging` gate for testing/debugging;
  optional, folds into the PA-LOG-002 re-scope.
- No println!/eprintln!. GUI-independence upheld.
- Logging verdict: **adequate (defensible zero-log for a logical model)**.

## 6. Findings logged

- **PA-CONFLICT-002** (CONSISTENCY -- architectural; PA-WATCH-006 resolved):
  edit-operations `EditorTransaction` vs undo-redo `Transaction`/`ScrapStack` are
  two independent, unbridged undo-unit models; the editing layer (ff-line-commands)
  produces `EditorTransaction` and never uses ff-undo-redo. Owner decision: unify
  (undo-redo consumes EditorTransaction) OR declare EditorTransaction canonical +
  reconcile/retire undo-redo's parallel model. Confirm at Wave 4.
- **PA-SPLIT-005** (SPLIT PROPOSAL): 17 reqs / 553 lines / 3+ responsibilities.
  SPEC split (text-mutation Req 1-5; selection/multi-caret/rectangular Req 6-9;
  profile/CAPS Req 16-17). Crate split lower value. Owner-gated.
- **PA-WATCH-006 RESOLVED**: selection-model ownership clean (edit-operations sole
  owner; caret-and-selection consumes; undo-redo snapshots separately). The
  transaction-model concern is carried as PA-CONFLICT-002.
- **PA-DEP-001** (unused dependency -- carried from prior pass, re-confirmed):
  ff-line-commands declares `ff-undo-redo` but never uses it. Remove the dep (or
  wire it if undo-redo becomes canonical per PA-CONFLICT-002). No gate. [Owning
  unit line-commands, W1.6.]
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-edit-operations contributes
  matches (doc-comment arrows/em-dashes, separators). Rolled into PA-LOG-001.
