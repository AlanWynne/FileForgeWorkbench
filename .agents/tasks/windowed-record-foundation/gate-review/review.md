# Gate Review -- CR-CH-058 Universal Windowed Record-Oriented Document Model

Reviewer: design-review subagent (fresh context). This is a DOCUMENTATION gate
(spec docs only); no builds or test suites were run. The gate was judged against
the authoritative inputs (`FOUNDATION-DESIGN.md`, `findings.md`), the author's
`manifest.md` + `verification-note.md`, and the project workflow / specs /
documentation steering rules.

## Verdict

APPROVED. No HIGH or MEDIUM findings. One NIT (non-blocking) is recorded below.

The gate is a faithful, complete, and internally consistent encoding of the
owner-approved FOUNDATION-DESIGN. Every verification area in the brief was
checked against the actual authored files (not the author's summary) and passed.

---

## What was verified (area by area)

### 1. Conformance to FOUNDATION-DESIGN.md -- PASS

- Piece-table spine encoded: document-model Req 12.1 names the three layers
  (Immutable_Original_Index ~16-byte entries, ordered Piece_List of
  `Original`/`Edited` with running record-count, append-only Append_Buffer).
  Glossary terms for all three added. Design delta (document-model/design.md
  section "Design Delta ... (CR-CH-058)") restates the (A)/(B)/(C) model.
- Universal RecordFormat in ff-document-model, SUPPLIED by the owning CE:
  Req 11.1 defines the type with the exact three variants
  (`Delimited`/`Fixed`/`Variable`); Req 11.2 has the CE supply it; design delta
  "RecordFormat (lives in ff-document-model, CE supplies the value)" matches
  owner decision #1. Native CE returns Delimited (generalises LineEndMode).
- 3-page Window_Band: Req 12.4 and large-file-performance Req 10.3; glossary
  Window_Band term; design delta "Windowed byte residency".
- Scrollbar/extents sized on index Total_Records, never the resident window:
  document-model Req 12.5, viewport Req 15.1, and the revised viewport Req 1.10.
  `total_records` vs resident window are explicitly stated as never fused.
- Zoom-never-loads: document-model Req 12.6, viewport Req 15.5,
  large-file-performance Req 10.2.
- SAVE-as-rebaseline distinct from mid-session compaction: document-model design
  delta "SAVE = re-baseline vs mid-session compaction (distinct)"; change-log
  entry states both and their distinction. SAVE rides CR-CH-053 Task 20/21 seam.
- Undo as inverse piece-list splices WITH the byte->record/piece addressing
  ADAPTATION called out as a real design item: undo Req 20.1 (address in
  record/piece terms), 20.2 (inverse splices, not snapshots), 20.3 (byte-identical
  round-trip); the REVISION NOTE explicitly flags that `EditOperation`'s absolute
  `position: u64` assumption no longer holds and must be adapted. This matches
  FOUNDATION-DESIGN open-question #2 (verified at the crate).
- Destructive-scale ops mapped onto CR-CH-053 Confirmable_Commands with SINGLE
  -Y (NOT dual --force --confirm): edit-operations Req 18.4/18.5,
  find-and-replace Req 21.4/21.5, undo Req 20.4. Macro-no-switch = refuse-with-RC
  encoded in all three. No dual-flag semantics anywhere.
- Mainframe Fixed/Variable CE deferred to a later V-stream: stated in
  document-model Req 11.2, Req 11.5 ("mainframe save itself is OUT OF SCOPE"),
  and the change-log Affects line ("the mainframe ... record-aware save is a
  LATER V-stream gate built ON this foundation").
- No owner-confirmed decision is relitigated or contradicted. The
  verification-note's owner-fidelity list (single -Y, refuse-with-RC, re-baseline
  vs compaction, 3-page band, scrollbar-from-index, zoom-never-loads, u64 with
  ~100M target / ~150M max_resident_records, estimated-then-exact accepted,
  RecordFormat home, piece search linear-first) was spot-checked against the
  authored criteria and holds.

### 2. The SAFETY RULE present and testable in every relevant sub-project -- PASS

Byte-identical native (Delimited) open/edit/save appears as an explicit,
testable criterion in each relevant sub-project:
- document-model Req 12.12 (byte-identical open/edit/save, "non-negotiable
  safety rule"), with task 20.4 calling for a "byte-identical native round-trip
  on Delimited fixtures" test (tests-first).
- edit-operations Req 18.3 (byte-identical observable result of every edit).
- find-and-replace Req 21.6 (CHANGE bytes identical to pre-CR-CH-058).
- undo-redo Req 20.3 (UNDO/REDO byte-identical to original state).
The master phase and WRF entries restate the safety rule per phase.

### 3. Existing residency-assuming criteria REVISED IN PLACE -- PASS

Each named revision was confirmed in the actual file, not merely appended-around:
- document-model Req 4 reworked (header REVISION NOTE withdrawing full-residency;
  AC 4.1 windowed ranged read, 4.5 explicitly replaces the old "finalize into a
  COMPLETE index" and states the model SHALL NOT rescan into a dense resident
  index). Glossary GapBuffer / TextBuffer / SparseLineIndex / StreamingFileReader
  entries each carry an inline "REVISED (CR-CH-058)" annotation describing the
  change.
- viewport Req 1.10 revised in place (total MAY be estimated/still-counting,
  from index Total_Records never the window). Req 2 carries a REVISION NOTE that
  clamping is against the possibly-estimated total and re-clamps on exact publish.
- large-file-performance Req 7 header REVISION NOTE + AC 7.1 and AC 7.6 reworked
  so range access is MANDATORY ("no longer conditional"), "line" access is a
  record access.

### 4. EARS format, numbering, record/RecordFormat as first-class -- PASS

- New ACs use valid EARS: event-driven `WHEN ... THE ... SHALL ...` (e.g. 11.2,
  12.7, 12.8, 15.2, 15.3, 18.4, 20.4, 21.2) and the valid ubiquitous
  `THE ... SHALL ...` form (e.g. 11.1, 12.1, 12.3, 15.1, 20.1). No malformed
  criteria found.
- Numbering continues sequentially after existing criteria: document-model Req
  11, 12 (existing ended at 10); viewport Req 15 (existing 14); large-file Req 10
  (existing 9); display-line-mapping Req 11 (existing 10); edit-operations Req 18
  (existing 17); undo Req 20 (existing 19); find-and-replace Req 21 (existing 20).
- `record` / `RecordFormat` introduced as first-class alongside `line`:
  document-model glossary (Record, RecordFormat) + Req 11; display-line-mapping
  Document_Line glossary annotated "a Document_Line IS a document RECORD";
  "line" retained as a Delimited synonym so native behaviour is byte-identical.

### 5. Task phasing matches F1..F5; task hygiene -- PASS

- document-model tasks 19 (F1 RecordFormat) and 20 (F1 spine / F2 window / F5
  guard) with sub-tasks tagged (F1)/(F2)/(F5); viewport 17 (F2/F3); large-file 17
  (F2/F4); display-line-mapping 14; edit-operations 40 (F1 edits / F4 guard); undo
  21 (F1 journal / F4 destructive-scale); find-and-replace 23. Master WRF.1-WRF.6
  map WRF.2=F1, WRF.3=F2, WRF.4=F3, WRF.5=F4, WRF.6=F5.
- Every phase keeps FFWB building and honours the safety rule (stated in the F1
  task notes, master phase blockquote, and change-log). F1 is explicitly the
  no-behaviour-change first step.
- Tasks are independently completable, cross-reference criteria via
  `// Validates: Requirement X.Y`, use only `[ ]` markers with descriptive titles
  and `Covers:` lines; new tasks are `[ ]` (never pre-marked `[x]`). Edit-ops
  continues 38 -> 39 -> 40 with no gap.
- 400-line rule respected in planning: document-model task 19.1/20.1 explicitly
  split by concern (`record_format.rs`, `piece_list.rs`, `original_index.rs`,
  `append_buffer.rs`) "each under 400 lines".

### 6. Gate mechanics completeness -- PASS

- project-master/tasks.md: new `## Phase (windowed-record-foundation) -- CR-CH-058`
  section with WRF.1-WRF.6 and a Status/Count table row, matching the file's
  existing phase+count pattern.
- TCR.md: new `### Phase (windowed-record-foundation)` section with one NOT
  COVERED red-circle row per NEW criterion in the correct crate column
  (ff-document-model Req 11.x/12.x spot-checked).
- change-log.md: CR-CH-058 under `## Change Requests`, the next MAINLINE
  unprefixed id after CR-CH-057, Status PENDING GATE, with a dedicated
  Framework-change note and the REVISED-in-place criteria called out.

### 7. Framework conformance -- PASS

- ff-document-model public-surface rework flagged explicitly as an
  owner-confirmed foundational change in document-model Req 12 (FRAMEWORK-TOUCHING
  NOTE), the design delta, the change-log Framework-change note, and the master
  phase blockquote.
- Every new requirement carries a terminal criterion stating it does NOT change
  the shell command-dispatch / navigation-stack / focus-latch /
  WorkspaceDescriptor-persistence seams (document-model 11.6, edit-ops 18.6, undo
  20.6, find-and-replace 21.7; large-file/display-line/viewport state the same in
  prose). SAVE rides the CR-CH-053 Task 20/21 seam (document-model design delta +
  change-log).

### 8. Documentation rules -- PASS

- The authored CR-CH-058 content (document-model Req 11/12, revised Req 4
  glossary; viewport Req 15 + revisions; large-file Req 10 + revisions;
  display-line Req 11; edit-ops Req 18; undo Req 20; find Req 21; all design
  deltas; master phase; TCR rows; change-log) is plain ASCII, using `--`, `->`,
  `<=`, `>=`. A targeted grep for the prohibited set (em/en dash, curly quotes,
  ellipsis, math arrows/comparisons) over the authored regions returned no hits.
- The only non-ASCII matches in the touched files are PRE-EXISTING en dashes in
  unrelated Introduction source-reference lines and the long-standing Requirement
  8 ACs of document-model (not edited by this gate), plus the TCR red-circle
  status emoji (allowed in TCR tables per documentation.md). These are not gate
  defects.
- Task-format rules from specs.md obeyed (only `[ ]`/`[x]`, titles present,
  status/phase notes on sub-bullets).

---

## Findings

### NIT-1 -- document-model tasks.md "Acceptance Criteria Coverage Matrix" not extended to Req 11/12

File: `docs/specs/document-model/tasks.md`, the trailing
"Acceptance Criteria Coverage Matrix" table.

The matrix still lists only Req 1-10 and does not add rows for the new Req 11
(Universal Record Abstraction) and Req 12 (Piece-Table Spine) mapping to the new
tasks 19 and 20. The tasks themselves correctly carry `Covers:` and
`// Validates:` annotations, so traceability is not actually lost; this is a
documentation-tidiness gap in a summary table, not a missing linkage.

Severity: NIT (does not block). Concrete fix: append two rows to the matrix:
`| Req 11: Universal Record Abstraction and RecordFormat | AC 11.1-11.6 | Task 19 |`
and
`| Req 12: Piece-Table Spine and Windowed Byte Residency | AC 12.1-12.12 | Task 20 |`.
Optionally also update the Req 4 matrix row label from "Streaming File Loading"
to "Windowed File Loading and Index Build" to match the revised requirement
title. Not required for approval.

---

## Verified assumptions

- The author's claim that document-model Req 4.5 was reworked away from "finalize
  into a COMPLETE index" -- VERIFIED by reading AC 4.5 (now explicitly withdraws
  the dense-resident-index finalize and forbids a whole-buffer rescan).
- The claim that viewport Req 1.10 / Req 2 were revised in place for an
  estimated/still-counting total -- VERIFIED (Req 1.10 REVISED clause; Req 2
  REVISION NOTE).
- The claim that large-file-performance Req 7/7.6 make range access MANDATORY --
  VERIFIED (Req 7 header note + AC 7.1/7.6 wording "no longer conditional").
- The claim that the undo model is operation-based and already carries
  Logical_Record_ID / Rule_Transaction / Index_Transaction, so Req 20 adapts
  rather than replaces -- VERIFIED against the undo-redo requirements/tasks (Req 7
  bulk transactions, Req 14 logical record IDs, tasks 17-20 present) and the
  Req 20 REVISION NOTE.
- The claim that find-and-replace is already behind a `CharacterIndexer` trait so
  Req 21 slots a windowed indexer in -- CONSISTENT with the authored Req 21 and
  design delta and with findings.md Q4; taken as stated (the trait's existence is
  a findings.md current-state claim, not re-derived here).
- CR-CH-058 is the correct next mainline unprefixed id -- VERIFIED (CR-CH-057 is
  the prior highest in change-log.md; mainline worktree uses no prefix).
- All seven design.md files received an appended CR-CH-058 design delta --
  VERIFIED by grep (each has a "Design Delta ... (CR-CH-058)" section).
- Single-`-Y` (not dual-flag) destructive-scale semantics -- VERIFIED in
  edit-ops 18.5, find 21.5, and the design.

## Unverified / wrong assumptions

- None found. No criterion was observed to contradict an owner-confirmed decision
  or another section. (Current-state claims about existing crate internals -- e.g.
  the exact `CharacterIndexer` API shape -- originate in findings.md and were not
  independently re-derived against source, as this is a docs-only gate; they are
  internally consistent with the authored specs.)
