# Verification Note -- CR-CH-058 Requirements Gate

This note records what I verified against the REAL spec/code state in the
worktree `c:/workspace/VSC/FileForgeWorkbench/.worktrees/env-registry` (branch
`feature/environment-registry`) so the reviewer can check the evidence without
re-reading every spec. Primary inputs read in full: FOUNDATION-DESIGN.md and
findings.md (both under `.agents/tasks/windowed-record-foundation/`).

## Iteration

No `gate-review/review.json` existed -- this is the FIRST iteration (authored from
scratch).

## Sub-projects touched (all existing folders under docs/specs/ were read first)

document-model, viewport-and-scrolling, large-file-performance,
display-line-mapping, edit-operations, undo-redo-transactions, find-and-replace.

## Existing requirements READ and the highest existing numbers (basis for numbering)

- document-model: Req 1-10 read in full. New: Req 11, 12. Revised in place: Req 4
  (was "Streaming File Loading", AC 4.1-4.9 -> windowed, AC 4.1-4.11 with 4.5
  reworked away from "finalize into a COMPLETE index"); glossary GapBuffer,
  TextBuffer, SparseLineIndex, StreamingFileReader annotated REVISED and new
  record/piece/window/total glossary terms added.
- viewport-and-scrolling: Req 1-14 read. New: Req 15. Revised in place: Req 1.10
  (max_top_line now allows estimated/still-counting total from index, not window)
  and a Req 2 revision note (clamping against the possibly-estimated total).
- large-file-performance: Req 1-9 read. New: Req 10. Revised in place: Req 7
  header note + Req 7.1 and Req 7.6 (range access now MANDATORY, "line" is a
  record access).
- display-line-mapping: Req 1-10 read. New: Req 11. Document_Line glossary term
  annotated (a Document_Line is a record; total may be estimated).
- edit-operations: Req 1-17 read (last task was 39). New: Req 18.
- undo-redo-transactions: Req 1-19 read. New: Req 20. Verified at the spec that
  the model is OPERATION-based (EditOperation.inverse(), UndoableState,
  DocumentUndoManager, ScrapStack) and already carries Logical_Record_ID,
  Rule_Transaction, Index_Transaction (Req 7, 14) -- the design delta builds on
  these, matching FOUNDATION-DESIGN open-question #2.
- find-and-replace: Req 1-20 read. New: Req 21. Verified the engine is already
  behind the `CharacterIndexer` trait (Req 18) so only the fed indexer + CHANGE
  write path change (matches findings.md Q4).

## Existing TASK numbering verified (continued, never pre-marked [x])

document-model last 18 -> added 19, 20. viewport last 16 -> 17. large-file last
16 -> 17. display-line-mapping last 13 -> 14. edit-operations last 39 -> 40.
undo last 20 -> 21. find-and-replace last 22 -> 23. All new tasks are `[ ]`.

## Cross-cutting docs

- change-log.md: highest MAINLINE unprefixed CR-CH id found was CR-CH-057 (grep
  of `^### CR-CH-\d+`); new id = CR-CH-058, logged under `## Change Requests`
  (newest-first), Status PENDING GATE, with the framework-change note and the
  REVISED-in-place criteria called out.
- project-management/project-master/tasks.md: new `## Phase
  (windowed-record-foundation)` section with WRF.1-WRF.6 (ordered to F1..F5) and
  a Status/Count row, matching the file's existing phase-section + count-table
  pattern.
- quality/TCR.md: new `### Phase (windowed-record-foundation)` section with one
  NOT COVERED (red-circle) row per NEW criterion (document-model 11.1-11.6 +
  12.1-12.12; viewport 15.1-15.6; large-file 10.1-10.5; display-line-mapping
  11.1-11.5; edit-operations 18.1-18.6; undo 20.1-20.6; find-and-replace
  21.1-21.7), placed in the correct crate column.

## Owner-decision fidelity (no relitigation)

Single `-Y` for destructive-scale (not dual --force --confirm); macro-no-switch
refuses with RC; SAVE = re-baseline distinct from compaction; 3-page Window_Band;
scrollbar from index Total_Records never the window; zoom-never-loads; u64 record
numbers with ~100M target + ~150M max_resident_records default; estimated-then
-exact scrollbar accepted; RecordFormat lives in ff-document-model, CE supplies
value; piece search linear-first. All encoded verbatim from FOUNDATION-DESIGN
sections 10-11.

## Safety rule and framework-conformance encoded

Byte-identical native (Delimited) open/edit/save appears as an explicit criterion
in every relevant sub-project (document-model 12.12, edit-operations 18.3,
find-and-replace 21.6, undo 20.3) and the master phase. The ff-document-model
public-surface rework is flagged framework-touching in document-model Req 12, the
design delta, the change-log, and the master phase; no shell dispatch / nav stack
/ focus latch / WorkspaceDescriptor change is introduced.

## Character compliance

All authored content is plain ASCII (`--`, `->`). A grep of the prohibited set
over my payloads returned no matches; the only non-ASCII in the touched files are
PRE-EXISTING en dashes in unrelated glossary/intro ranges that I did not edit, and
the TCR red-circle status emoji (allowed in TCR tables).

## No source code written

No file outside docs/ was modified except the two gate-review notes under
`.agents/tasks/windowed-record-foundation/gate-review/` and a reusable helper
`tools/python/append_text.py` (plus ephemeral `tools/logs/` payloads, git-ignored).
