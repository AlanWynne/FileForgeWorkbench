# Implementation Plan -- CR-CH-058 Phase F1 (ff-document-model)

Universal windowed record-oriented document model, PHASE F1 ONLY: RecordFormat +
Immutable_Original_Index + Piece_List + Append_Buffer, STILL FULLY RESIDENT on
open (no windowing -- that is F2). Proves the piece-table spine, the undo
piece-splice journal, and the NON-NEGOTIABLE byte-identical native (Delimited)
open/edit/save round-trip (AC 12.12).

Covers tasks 19 (all), 20.1-20.4, 20.8 PARTIAL (trait only), 20.9 PARTIAL
(F1-relevant proptests). Everything else in task 20 is a LATER phase and MUST NOT
be built now.

## Ground rules (apply to every item)

- Worktree (do NOT create a new one): `c:/workspace/VSC/FileForgeWorkbench/.worktrees/wrf-foundation`
  on branch `feature/windowed-record-foundation`. Step agents' cwd may differ, so
  use absolute paths everywhere.
- Git: `git -C 'c:/workspace/VSC/FileForgeWorkbench/.worktrees/wrf-foundation' ...`.
  Commit locally per logical item; never push.
- Cargo: run with the worktree as the working directory, e.g.
  `cargo test -p ff-document-model --manifest-path c:/workspace/VSC/FileForgeWorkbench/.worktrees/wrf-foundation/Cargo.toml`.
- Shell: ALWAYS the clean wrapper
  `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "<one command>"`.
  One command per call; never `;`-chain; never pipe to Format-Table; never glue a
  kill. Use read/grep/glob tools for inspection, not Get-Content/Select-String.
- TDD MANDATORY: write the test FIRST, run it, confirm it FAILS (red) for the
  right reason, THEN implement to green. Every test carries a
  `// Validates: Requirement X.Y` comment. Do not pre-write implementation.
- 400-line rule: NO non-test `.rs` file exceeds 400 lines. Split test modules
  into a sibling `_tests.rs` (via `#[path = "..._tests.rs"] mod tests;`) before
  the inline `#[cfg(test)]` module approaches ~200 lines.
- Framework conformance: this is an ff-document-model INTERNAL rework. Its public
  surface MAY change (owner-confirmed foundational change). Do NOT touch shell
  command-dispatch, navigation-stack, focus-latch, or WorkspaceDescriptor
  persistence. SAVE rides the existing CR-CH-053 Task 20/21 seam -- do NOT change
  that seam.
- Do NOT break existing ff-undo-redo tests, and do NOT break existing
  ff-document-model tests except where this plan explicitly reframes an API
  (keep behaviour byte-identical for Delimited).
- Documentation/comments: plain ASCII only in `.rs` files.

## Design decisions made for F1 (recorded here, not relitigated)

These are already fixed by the approved specs/FOUNDATION-DESIGN; restated so the
implementer does not re-decide:

1. `RecordFormat` lives in `ff-document-model`; the CE supplies the value; native
   = `Delimited` (generalises `LineEndMode`). (AC 11.1, 11.2)
2. `u64` record numbers everywhere. A new `RecordNumber(u64)` newtype is added in
   `types.rs` mirroring `LineNumber`; the Delimited line API treats record N ==
   line N so `LineNumber` and `RecordNumber` are interchangeable for Delimited.
   DECISION: introduce `RecordNumber` but make the line API delegate to the
   record API (not vice-versa), so "a Delimited record IS a line" holds and the
   existing `LineNumber`-based public methods stay byte-identical.
3. F1 keeps the WHOLE FILE resident. The `OriginalIndex` resident `Vec` impl is
   built from the fully-resident buffer bytes on open; original record bytes are
   read from that resident buffer via index offsets. No ranged VFS reads, no
   eviction, no `max_resident_records` guard (all F2/F5).
4. Undo adaptation (see item 9): F1 adds a piece-splice journal WITH `inverse()`
   INSIDE `ff-document-model` (new `piece_journal.rs`), expressed in record/piece
   terms. The existing `ff-undo-redo` crate is NOT reworked in F1 (its
   byte-position `EditOperation` model is left intact and its tests stay green);
   the document's own journal is what the F1 byte-identical round-trip test
   drives. DEFERRAL FLAGGED: full migration of `ff-undo-redo` from byte-position
   to record/piece addressing is explicitly deferred beyond F1 (design.md "Undo
   operation-addressing adaptation"); F1 ships the document-model journal as the
   correct, self-contained undo mechanism for the piece table and leaves the
   `ff-undo-redo` integration seam untouched. This keeps F1 bounded and avoids
   breaking the 9000+ existing undo tests.
5. SAVE = re-baseline (item 8): walk pieces, emit re-framed bytes, atomic write,
   collapse to one Original piece, rebuild index, drop journal to that point.

## File-by-file budget (which concern -> which <400-line file)

New files under `crates/ff-document-model/src/`:
- `record_format.rs` -- `RecordFormat` enum + `DelimiterTerminator` enum + record
  framing helpers (frame-scan a byte slice into record boundaries per format;
  re-emit a record's delimiter). Target < 250 lines; tests in
  `record_format_tests.rs` if they exceed ~150 lines.
- `original_index.rs` -- `OriginalIndexEntry` (`file_offset: u64`,
  `byte_length: u32`, `flags: u32`), the `OriginalIndex` trait, and
  `ResidentOriginalIndex` (Vec-backed) impl. Build-from-bytes constructor per
  RecordFormat. Target < 300 lines.
- `append_buffer.rs` -- `AppendBuffer` newtype wrapping the existing `GapBuffer`
  in the Append_Buffer role (append-only edited/inserted record bytes; return
  a `buf_range` for appended bytes). Thin; target < 200 lines.
- `piece_list.rs` -- `Piece` enum (`Original { first_record, count }` /
  `Edited { buf_range, count }`), `PieceList` with running record-count per
  piece, the splice operations, and record<->piece<->position resolution.
  This is the largest; keep CORE logic < 400 lines and move tests to
  `piece_list_tests.rs`.
- `piece_journal.rs` -- `SpliceOp` (the inverse-carrying journal entry) +
  `PieceJournal` (undo/redo stacks of `SpliceOp`, save-point marker, drop-to
  on re-baseline). Target < 300 lines.

Modified files:
- `types.rs` -- add `RecordNumber(u64)` newtype (+ tests).
- `text_buffer.rs` -- reframe to coordinate the three layers; keep the public
  method signatures. WATCH the 400-line budget: the current file is ~470 lines
  WITH tests (~250 non-test). Move its `#[cfg(test)] mod tests` to a new
  `text_buffer_tests.rs` as part of this work so new coordination code fits.
- `document.rs` -- thread `RecordFormat` through open; add record-API passthrough
  methods; keep the line API byte-identical. If non-test lines approach 400,
  split record-API methods into a `document_records.rs` (via `impl Document`
  continued in a second file) -- decide at implementation time.
- `lib.rs` -- declare the new modules and re-export the new public types.
- `Cargo.toml` -- no new deps expected (proptest/pretty_assertions already dev-deps).

## Verification commands (SCOPED only -- Kiro never runs the full gate)

Run these from the worktree (as cwd or via `--manifest-path`):
- Red/green per step: `cargo test -p ff-document-model --manifest-path c:/workspace/VSC/FileForgeWorkbench/.worktrees/wrf-foundation/Cargo.toml <test_name>`
- Full crate tests: `cargo test -p ff-document-model --manifest-path c:/workspace/VSC/FileForgeWorkbench/.worktrees/wrf-foundation/Cargo.toml`
- Undo crate stays green: `cargo test -p ff-undo-redo --manifest-path c:/workspace/VSC/FileForgeWorkbench/.worktrees/wrf-foundation/Cargo.toml`
- Lint: `cargo clippy -p ff-document-model --manifest-path c:/workspace/VSC/FileForgeWorkbench/.worktrees/wrf-foundation/Cargo.toml -- -D warnings`
- Format: `cargo fmt --manifest-path c:/workspace/VSC/FileForgeWorkbench/.worktrees/wrf-foundation/Cargo.toml`
- 400-line check (PowerShell, read tool for counts preferred): confirm each new
  non-test `.rs` is under 400 lines.
Then HAND OFF: the owner runs the full `cargo gate --build` outside Kiro. Do NOT
run `cargo gate` or any `--workspace` build/test.

---

# Ordered Implementation Plan

- [ ] 1. Add the `RecordNumber(u64)` newtype and declare F1 module stubs.
      Add `RecordNumber` to `types.rs` (mirror `LineNumber`: `ZERO`, `value()`,
      `From<u64>`/`Into<u64>`, arithmetic, `Default`, `Display`). Add empty module
      files `record_format.rs`, `original_index.rs`, `append_buffer.rs`,
      `piece_list.rs`, `piece_journal.rs` and declare them `pub mod` in `lib.rs`
      (no re-exports yet). This keeps the crate compiling as later items fill the
      modules.
      Files: types.rs, lib.rs, record_format.rs, original_index.rs,
      append_buffer.rs, piece_list.rs, piece_journal.rs
      Verify: `cargo test -p ff-document-model --manifest-path <worktree>/Cargo.toml`
      compiles and all existing tests pass; new `RecordNumber` unit tests pass.

- [ ] 2. (TDD) RecordFormat type + Delimited framing -- tests first.
      Write failing tests in `record_format.rs` (or `record_format_tests.rs`):
      (a) `RecordFormat::Delimited { terminator }` with
      `DelimiterTerminator { Crlf, Lf, Cr, Mixed }`, plus `Fixed { lrecl: u32 }`
      and `Variable { max_lrecl: u32, rdw: bool }` variants EXIST and derive
      Debug/Clone/PartialEq; (b) a framing helper `frame_records(bytes, format)`
      that, for `Delimited`, returns record boundaries byte-IDENTICAL to the
      existing `line_end::line_ending_length_at` scan for CRLF/LF/CR/Mixed
      fixtures (compare against `LineIndex::rebuild` line_starts for the same
      bytes); (c) a `reemit_terminator(terminator)` helper returning the exact
      delimiter bytes. Confirm RED. Then implement the enum + helpers. Fixed and
      Variable framing may be MINIMAL stubs (arithmetic for Fixed; return an error
      or `todo`-free "unsupported in F1" path for Variable) -- F1 only EXERCISES
      Delimited, but the variants must exist for AC 11.1.
      // Validates: Requirement 11.1, 11.4
      Files: record_format.rs (+ record_format_tests.rs if large)
      Verify: targeted tests go red before impl, green after; full crate tests pass.

- [ ] 3. (TDD) OriginalIndex trait + ResidentOriginalIndex -- tests first.
      Write failing tests in `original_index.rs`: (a) `OriginalIndexEntry` has
      `file_offset: u64`, `byte_length: u32`, `flags: u32` and is ~16 bytes
      (`assert_eq!(size_of::<OriginalIndexEntry>(), 16)`); (b) the `OriginalIndex`
      trait exposes `total_records() -> u64`, `entry(k: RecordNumber) ->
      Option<OriginalIndexEntry>`, `record_start(k) -> u64`,
      `record_byte_length(k) -> u32`, `record_from_offset(pos: u64) ->
      Option<RecordNumber>`; (c) `ResidentOriginalIndex::from_bytes(bytes,
      RecordFormat::Delimited {..})` produces one entry per delimiter-terminated
      record with correct offsets/lengths (delimiter bytes counted in the record's
      byte_length so re-emit is byte-identical); (d) round-trip:
      `record_from_offset(record_start(k)) == Some(k)` for all k; (e) the empty
      document case = a single record of length 0 (AC 4.8 shape). Confirm RED.
      Implement the trait + resident Vec impl. Build-from-bytes reuses the
      `record_format::frame_records` helper.
      // Validates: Requirement 11.4, 12.1
      Files: original_index.rs (+ original_index_tests.rs if large)
      Verify: targeted tests red-before/green-after; full crate tests pass.

- [ ] 4. (TDD) AppendBuffer role scoping -- tests first.
      Write failing tests in `append_buffer.rs`: `AppendBuffer` wraps a
      `GapBuffer`, `append(bytes) -> buf_range` returns a `(start,len)` range of
      the appended bytes, `bytes(range) -> Vec<u8>` reads them back, and appends
      are stable (an earlier `buf_range` still reads the same bytes after later
      appends). Confirm RED. Implement `AppendBuffer` delegating to the existing
      `GapBuffer` (reuse `gap_buffer.rs`; do NOT delete or rewrite it -- it keeps
      its current API and tests). Add a `BufRange { start: u64, len: u64 }` type
      (in append_buffer.rs or types.rs).
      // Validates: Requirement 12.1
      Files: append_buffer.rs, (types.rs if BufRange lives there)
      Verify: targeted tests red-before/green-after; full crate tests pass;
      existing gap_buffer tests still pass.

- [ ] 5. (TDD) PieceList construction + record/position resolution -- tests first.
      Write failing tests in `piece_list.rs` (or `piece_list_tests.rs`):
      (a) `Piece::Original { first_record, count }` and
      `Piece::Edited { buf_range, count }` exist; (b) `PieceList::from_original(
      total_records)` yields a single Original piece spanning the whole doc with
      a running record-count equal to total_records; (c) `total_records()` sums
      piece counts; (d) `piece_at_record(n) -> (piece_idx, local_record)` by
      LINEAR scan over pieces using running counts (AC 12.9); (e) the
      running-record-count invariant holds (sum of counts == total_records after
      construction). Confirm RED. Implement `PieceList`, `Piece`, running-count
      bookkeeping, and the linear search. No splicing yet.
      // Validates: Requirement 12.1, 12.9
      Files: piece_list.rs (+ piece_list_tests.rs)
      Verify: targeted tests red-before/green-after; full crate tests pass.

- [ ] 6. (TDD) PieceList SPLICES: insert/delete/move/copy/overtype -- tests first.
      Write failing tests covering each splice as a piece-list operation (NOT an
      O(file) byte shift): inserting a record splits an Original piece into
      Original + Edited + Original; deleting records trims/splits pieces; move
      re-orders pieces; copy adds a piece referencing existing bytes; overtype
      replaces a record's piece with an Edited piece. After EACH operation assert
      the running-record-count invariant and that `total_records()` matches a
      naive `Vec<record>` reference model. Mark dirty pieces: `Edited` pieces (and
      any piece touched by a splice) carry a `dirty` flag tracked for later
      pinning (F2) -- F1 only SETS/queries the flag, no eviction. Confirm RED.
      Implement the splice methods and dirty tracking. Edited record bytes go
      through the `AppendBuffer` (item 4).
      // Validates: Requirement 12.2
      Files: piece_list.rs (+ piece_list_tests.rs)
      Verify: targeted tests red-before/green-after; full crate tests pass.

- [ ] 7. (TDD) Piece-splice undo journal with inverse() -- tests first.
      Write failing tests in `piece_journal.rs`: `SpliceOp` records a piece-list
      splice and its `inverse()` (insert-splice <-> delete-splice; move/overtype
      swap); `PieceJournal` has `record(op)`, `undo()` and `redo()` that apply the
      (inverse) splice to a `PieceList`, `can_undo`/`can_redo`, a save-point
      marker (`set_save_point`/`is_at_save_point`), and `drop_to_current()` for
      re-baseline (item 8). Assert: record -> undo restores the exact prior
      PieceList arrangement (same pieces, same running counts); redo re-applies;
      round-trip over an arbitrary sequence of splices returns to the original.
      Confirm RED. Implement. This is self-contained in ff-document-model and does
      NOT call into ff-undo-redo (deferral per decision 4).
      // Validates: Requirement 12.2 (reversibility), 12.12 (undo behaviour)
      Files: piece_journal.rs (+ piece_journal_tests.rs if large)
      Verify: targeted tests red-before/green-after; full crate tests pass;
      ff-undo-redo tests UNCHANGED and still green.

- [ ] 8. (TDD) TextBuffer reframed as the 3-layer coordinator -- tests first.
      First, as a pure refactor, move the existing `text_buffer.rs`
      `#[cfg(test)] mod tests` into `text_buffer_tests.rs` so the file has budget
      headroom (no behaviour change; existing tests must still pass unchanged).
      Then write failing tests asserting TextBuffer now owns
      `OriginalIndex` + `PieceList` + `AppendBuffer` + `PieceJournal` and a
      `RecordFormat`, while its EXISTING public line/record methods
      (`insert`, `delete`, `line_start`, `line_end`, `line_from_position`,
      `line_count`, `length`, `get_range`, `char_at`, `contiguous_view`,
      `split_view`) remain byte-identical for a Delimited document built from
      bytes. Add record-API methods `total_records()`, `record_start(k)`,
      `record_byte_length(k)`, `record_from_position(pos)` and make the Delimited
      line API DELEGATE to them (line N == record N). Confirm RED. Implement:
      `TextBuffer::from_bytes(bytes, RecordFormat)` builds the resident index +
      single Original piece + empty append buffer; `insert`/`delete` become piece
      splices recorded in the journal; reads resolve record bytes from the
      resident buffer via index/piece. KEEP the whole file resident (F1). Watch
      the 400-line budget -- split coordination helpers if needed.
      // Validates: Requirement 11.3, 11.4, 12.1, 12.2
      Files: text_buffer.rs, text_buffer_tests.rs, (helper split file if needed)
      Verify: targeted tests red-before/green-after; the FULL existing
      text_buffer/document/line_index test suite passes byte-identically.

- [ ] 9. (TDD) Thread RecordFormat into Document open + record-API passthrough --
      tests first.
      Write failing tests in `document.rs`/`document_tests`: `Document` can be
      opened/constructed with an explicit `RecordFormat` (a new
      `Document::from_bytes_with_format(bytes, RecordFormat)` or a
      `set_record_format` + load path); the native/default path supplies
      `RecordFormat::Delimited { terminator }` derived from the current
      `LineEndMode` so existing `Document::new()` + `insert` behaviour is
      byte-identical (generalising LineEndMode, NOT replacing the observable line
      behaviour); `Document` exposes `total_records()`, `record_start`,
      `record_byte_length`, `record_from_position` delegating to TextBuffer;
      Fixed/Variable are NOT flattened to newline (assert a Fixed document keeps
      record framing -- a minimal constructor-level assertion is enough for F1
      since mainframe save is out of scope). Confirm RED. Implement the open
      threading and passthrough. Do NOT change watcher/viewport/save-point public
      behaviour.
      // Validates: Requirement 11.2, 11.3, 11.5, 11.6
      Files: document.rs (+ document_records.rs / document_tests.rs if budget needs)
      Verify: targeted tests red-before/green-after; full crate tests pass.

- [ ] 10. (TDD) Delimited record == line equivalence + record<->position
      round-trips + CRLF/LF/CR/Mixed framing (Task 19.4) -- tests first.
      Add a dedicated test module (e.g. `tests/record_equivalence.rs` integration
      test or a focused unit test file) asserting, over CRLF / LF / CR / Mixed
      fixtures: (a) for every k, `record_start(k) == line_start(LineNumber(k))`
      and `record_byte_length(k)` matches the line's content+terminator length;
      (b) `total_records() == line_count()`; (c)
      `record_from_position(record_start(k)) == k` and
      `line_from_position(line_start(k)) == LineNumber(k)` agree; (d) framing is
      correct for each terminator (a lone CR, a lone LF, a CRLF, and a Mixed file
      each produce the expected record boundaries). Confirm these are RED where
      functionality is still missing, then make green (most should pass if items
      2-9 are correct; any failure is a real bug to fix).
      // Validates: Requirement 11.1, 11.2, 11.3, 11.4, 11.5
      Files: record_equivalence tests (new), fixes to earlier modules as needed
      Verify: `cargo test -p ff-document-model --manifest-path <worktree>/Cargo.toml`
      all green.

- [ ] 11. (TDD) Splice correctness vs naive reference model + running-count
      invariant (Task 20.4 parts 1-2) -- tests first.
      Add tests driving an arbitrary scripted sequence of
      insert/delete/move/copy/overtype splices through TextBuffer and comparing
      the resulting document bytes against a naive `String`/`Vec<Line>` reference
      model that applies the same operations; after every operation assert the
      running-record-count invariant (sum of piece counts == total_records ==
      reference record count). Confirm RED (write at least one case that would
      fail a broken splice), then green.
      // Validates: Requirement 12.1, 12.2
      Files: new splice-correctness test file
      Verify: full crate tests green.

- [ ] 12. (TDD) Byte-identical native SAVE = re-baseline (Task 20.3 + 20.4
      part 3, AC 12.12) -- tests FIRST. THIS IS THE NON-NEGOTIABLE SAFETY TEST.
      Write failing tests FIRST: open a Delimited fixture (one per CRLF/LF/CR/
      Mixed), perform a representative edit sequence (insert line, delete line,
      overtype within a record), then SAVE and assert the emitted bytes are
      BYTE-IDENTICAL to the reference bytes produced by applying the same edits to
      a naive model AND, for the no-edit case, byte-identical to the ORIGINAL
      input (same bytes, same line endings). Also assert dirty/save-point
      behaviour: dirty after an edit, clean (`is_at_save_point`) after save, and
      the journal is dropped to the save point (re-baseline: can_undo == false
      past the save). Confirm RED. Then implement SAVE as re-baseline in
      TextBuffer/Document: walk the PieceList in order, emit each piece's bytes
      (Original via the resident buffer + index offsets; Edited via AppendBuffer)
      RE-FRAMED per `RecordFormat` (for Delimited re-emit the exact terminator so
      output is byte-identical), produce the full byte image, then collapse the
      PieceList to a single `Original{0..N}`, rebuild the OriginalIndex from the
      saved bytes, and `drop_to_current()` on the journal. The ATOMIC WRITE itself
      (VFS temp-file + rename) is the shell/CE's job via the CR-CH-053 Task 20/21
      seam -- F1 provides the byte image and the collapse/rebuild; expose a
      `rebaseline(saved_bytes)` entry point and a `save_image() -> Vec<u8>` so the
      round-trip is testable WITHOUT touching the save seam. Do NOT change the
      save seam.
      // Validates: Requirement 12.12, 12.1, 12.2
      Files: text_buffer.rs / document.rs (save_image + rebaseline), new
      byte_identical_roundtrip test file
      Verify: the byte-identical tests are RED before impl, GREEN after; full
      crate tests pass; ff-undo-redo tests still green.

- [ ] 13. OriginalIndex trait boundary slice for F5 (Task 20.8 PARTIAL).
      Confirm (and add a test asserting) that all TextBuffer/Document access to
      the original index goes through the `OriginalIndex` TRAIT, not the concrete
      `ResidentOriginalIndex` type -- so a future sparse/mmap impl (F5) can replace
      it without touching callers. Ship ONLY the resident Vec impl. Do NOT build
      sparse mode, mmap/spill, or the `max_resident_records` guard. Add a
      `// F5: above-budget sparse/mmap impl replaces ResidentOriginalIndex behind
      this trait` comment at the trait definition. If any caller references the
      concrete type directly, refactor it to the trait.
      // Validates: Requirement 12.11 (design-for only; impl deferred to F5)
      Files: original_index.rs, text_buffer.rs (trait-object or generic bound)
      Verify: a test constructs TextBuffer over a `Box<dyn OriginalIndex>` (or
      generic) and reads records; full crate tests pass.

- [ ] 14. (proptest, >=100 iters) Splice/undo round-trips + record addressing
      stable across edits (Task 20.9 PARTIAL) -- tests first.
      Add proptests (minimum 100 iterations, `// Feature: document-model,
      Property N:` comment) asserting: (a) for an arbitrary sequence of splice ops,
      applying then undoing them all returns the document to byte-identical
      original content and total_records; (b) record addressing is stable across
      intervening edits -- a record identified before an unrelated edit resolves to
      the correct bytes after it (where the edit does not target that record);
      (c) the running-record-count invariant holds after every op in the sequence.
      SKIP the estimated->exact Total_Records monotonicity proptest (that is
      F2/F3 -- windowing/background scan is not built in F1). Confirm RED on a
      deliberately broken shrink case if practical, then green.
      // Validates: Requirement 12.1, 12.2, 12.12
      Files: new proptest file (e.g. tests/piece_table_proptests.rs)
      Verify: `cargo test -p ff-document-model --manifest-path <worktree>/Cargo.toml`
      proptests pass (>=100 cases each).

- [ ] 15. Re-exports, lib docs, TCR update, and final scoped verification.
      Re-export the new public types from `lib.rs` (`RecordFormat`,
      `DelimiterTerminator`, `RecordNumber`, `OriginalIndex` + entry type,
      `Piece`/`PieceList` as needed by consumers, `BufRange`). Update the crate
      module doc comment to describe the three-layer model. Update
      `docs/quality/TCR.md`: set the rows for Req 11 (11.1-11.6) and the F1 slice
      of Req 12 (12.1, 12.2, 12.9, 12.11 design-for, 12.12) to PASS where an
      automated test now exists; leave F2-F5 criteria (12.3-12.8, 12.10) NOT
      COVERED. Then run the full SCOPED verification suite and fix anything red.
      Files: lib.rs, docs/quality/TCR.md
      Verify (all must pass):
        - `cargo fmt --manifest-path <worktree>/Cargo.toml`
        - `cargo clippy -p ff-document-model --manifest-path <worktree>/Cargo.toml -- -D warnings`
        - `cargo test -p ff-document-model --manifest-path <worktree>/Cargo.toml`
        - `cargo test -p ff-undo-redo --manifest-path <worktree>/Cargo.toml` (unchanged, green)
        - 400-line check on every new/changed non-test `.rs` file.

- [ ] 16. Mark F1 tasks done and HAND OFF for the full gate.
      In `docs/specs/document-model/tasks.md`, check off 19.1-19.5, 20.1-20.4,
      and the F1 portions of 20.8/20.9 that this plan delivered (leave F2-F5
      subtasks unchecked). Commit all work locally on
      `feature/windowed-record-foundation` (never push). State explicitly which
      scoped commands were run and their results, note the DEFERRAL (ff-undo-redo
      byte->record migration is post-F1; F1 ships the document-model piece-splice
      journal), and PROMPT the owner to run the full gate manually outside Kiro:
      `cargo gate --build` (fallback `pwsh -ExecutionPolicy Bypass -File tools\ffwb-gate.ps1`).
      Do NOT run the full gate or claim "done" on scoped checks alone -- this is
      "code-complete pending full gate".
      Files: docs/specs/document-model/tasks.md
      Verify: `git -C <worktree> status` shows only intended changes; scoped
      suite green; hand-off message posted.

---

## Deferrals and assumptions (flagged)

- DEFERRED to post-F1: migrating `ff-undo-redo`'s `EditOperation` from absolute
  byte-position addressing to record/piece addressing. F1 implements the
  piece-splice journal inside `ff-document-model` (item 7) as the correct,
  self-contained undo mechanism for the piece table and does NOT modify
  `ff-undo-redo` (its tests stay green). This matches design.md's "ADAPTS the
  operation addressing" note while keeping F1 bounded; the full crate migration is
  a later phase item.
- DEFERRED to F2+: all windowing (Window_Band, load/evict, scroll hysteresis),
  scrollbar-from-index authority, zoom-never-loads, background index scan +
  estimated->exact Total_Records, configurable budgets, `max_resident_records`
  guard, sparse/mmap OriginalIndex impl. F1 is fully resident on open by design.
- ASSUMPTION: Fixed/Variable framing is only STUBBED in F1 (variants exist for
  AC 11.1 and are not flattened to newline per AC 11.5), because mainframe open
  and save are out of scope of this gate (V-stream). Only Delimited is exercised
  end-to-end. If a reviewer reads AC 11.5 as requiring working Fixed re-padding,
  that is a mainframe-save concern explicitly out of F1 scope -- flag rather than
  build it.
- ASSUMPTION: SAVE's atomic VFS write stays on the CR-CH-053 Task 20/21 seam
  (shell/CE owned); F1 exposes `save_image()`/`rebaseline()` so the byte-identical
  round-trip is proven without touching that seam. If the implementer finds the
  round-trip cannot be proven without the seam, STOP and surface it rather than
  modifying the save seam.
