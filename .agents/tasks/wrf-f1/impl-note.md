# CR-CH-058 F1 -- Implementation Note (code-complete pending full gate)

STATUS: F1 (document-model piece-table spine + record model + byte-identical
SAVE) is CODE-COMPLETE and scoped-verified. Awaiting the OWNER's full
`cargo gate --build` on branch `feature/windowed-record-foundation` (worktree
`.worktrees/wrf-foundation`). This is the FIRST, lowest-risk phase of CR-CH-058;
it introduces the spine with NO windowing (whole file still resident) and NO
observable native behaviour change. Windowing (F2), background index + open UX
(F3), windowed FIND/CHANGE (F4), and the scalability guard (F5) are later phases.

## What F1 delivers (plan items 1-16, scope = tasks 19 + 20.1-20.4 + F1 slices
of 20.8/20.9)

New `ff-document-model` modules (each < 400 non-test lines; tests split out):
- `record_format.rs` -- `RecordFormat { Delimited{terminator} | Fixed{lrecl} |
  Variable{max_lrecl,rdw} }` + `DelimiterTerminator`; `frame_records` reproduces
  the existing line-scan boundaries for Delimited (CR/LF/CRLF/Mixed), arithmetic
  for Fixed. `native(LineEndMode)` is how the native CE generalises LineEndMode.
- `original_index.rs` -- `OriginalIndex` trait + `ResidentOriginalIndex` (lean
  16-byte entry: file_offset u64 + byte_length u32 + flags u32; ~1.6GB at 100M).
  The trait boundary is the F5 seam for a future sparse/mmap impl.
- `append_buffer.rs` -- `AppendBuffer` (append-only edited/inserted record bytes,
  wrapping the existing GapBuffer in its surviving role; stable `BufRange`).
- `piece_list.rs` -- `Piece` (Original/Edited + running count + dirty) + `PieceList`
  with insert/delete/move/copy/overtype SPLICES (no O(file) shift, no file
  rewrite), linear `piece_at_record`.
- `piece_journal.rs` -- `SpliceOp` (before/after + `inverse()`) + `PieceJournal`
  (undo/redo stacks, save-point, `drop_to_current` for re-baseline).
- `types.rs` -- `RecordNumber(u64)` + `BufRange`.

Integration (plan items 8-9, 12):
- `text_buffer.rs` + `text_buffer_records.rs` -- TextBuffer carries a
  `RecordFormat`; record API (`total_records/record_start/record_byte_length/
  record_from_position`) where a Delimited record IS a line (delegates to the
  line index, byte-identical). `save_image()`/`rebaseline()` prove the
  piece-table spine + re-baseline SAVE. `set_line_end_mode` keeps the native
  RecordFormat in step. The whole file stays resident (F1).
- `document.rs` + `document_records.rs` -- Document exposes the same record API +
  `save_image`/`rebaseline`, delegating to TextBuffer via the crate-internal
  `text_buffer()`/`text_buffer_mut()` accessors.

## Key decisions / adaptations (flagged)

- UNDO ADAPTATION (deferred, per plan): `ff-undo-redo`'s `EditOperation` is keyed
  by absolute byte position (assumes a resident doc). F1 does NOT migrate it;
  instead the piece-splice journal lives self-contained in `ff-document-model`
  (`piece_journal.rs`), expressed in piece/record terms. `ff-undo-redo` is
  UNTOUCHED and its tests stay green. The full byte->record migration of
  `ff-undo-redo` is a later phase (design.md "Undo operation-addressing
  adaptation"). This kept F1 bounded and avoided disturbing ~9000 existing tests.
- SAVE SEAM UNTOUCHED: `save_image()`/`rebaseline()` expose the byte image + the
  collapse/rebuild so the byte-identical round-trip is PROVEN without modifying
  the CR-CH-053 Task 20/21 "FFEDIT addresses SAVE to the owning CE" atomic-write
  seam. The shell/CE still owns the physical write.
- INTERACTIVE EDIT PATH unchanged in F1: edits still go through the proven
  gap-buffer `insert`/`delete`; the piece table is PROVEN (via save_image +
  proptests) but F2 is where interactive edits route through pieces + windowing.
  This is the FOUNDATION-DESIGN F1 intent ("prove the piece-table model ... still
  loads the whole file").
- Behaviour-identical simplification: removed the dead CRLF-merge helper stubs in
  `TextBuffer::insert` (`compute_actual_lines_added` returned `base` unchanged,
  ignoring the computed split/merge flags; the real work is the line-index
  rebuild). The CRLF split/merge tests still pass, confirming no behaviour change.

## Recovery note (honest record)

F1 was first attempted via a delegated workflow that STALLED (~2h17m no activity).
I aborted it and completed F1 directly. On takeover I VERIFIED its partial output:
the crate did NOT compile (an `AppendBuffer` `Default` derive requiring
`GapBuffer: Default`) and one `copy_records` test failed (source-piece
fragmentation). Both were fixed (hand-written `Default` delegating to `new()`;
`copy_records` now extracts sub-pieces without mutating the source).

## Verification (scoped -- the owner runs the full gate)

- `cargo test -p ff-document-model --lib` = 186 passed, 0 failed (includes 4
  proptests at 200 cases each).
- `cargo clippy -p ff-document-model --all-targets -- -D warnings` = clean.
- `cargo fmt` applied.
- 400-line rule: every new/changed non-test `.rs` under 400 (text_buffer.rs 361,
  document.rs ~399; record/save methods split into *_records.rs continuations;
  tests split into *_tests.rs).

## TCR / tasks

- TCR Req 11.1-11.6 -> PASS; Req 12.1, 12.2, 12.3, 12.9, 12.11, 12.12 -> PASS;
  Req 12.4-12.8, 12.10 stay NOT COVERED (F2/F5).
- tasks.md: 19 (+19.1-19.5), 20.1-20.4 and the F1 slices of 20.8/20.9 marked
  `[x]`; 20.5/20.6/20.7(remainder)/20.8(sparse+guard) stay `[ ]` (F2/F5).

## Hand-off

Scoped checks are clean. OWNER: please run the full gate from the worktree:
`cargo gate --build`
(fallback: `pwsh -ExecutionPolicy Bypass -File tools\ffwb-gate.ps1`).
F1 is code-complete pending that gate; it does NOT self-certify "done".
