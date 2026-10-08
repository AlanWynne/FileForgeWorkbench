# Foundation Design -- Windowed, Record-Oriented Document Model (piece-table spine)

STATUS: DESIGN DOCUMENT FOR OWNER REVIEW. This is NOT a gated spec. It captures
the architecture converged on in conversation (owner-approved process) so it can
be reviewed as a whole before being turned into formal gated
requirements/design/tasks per `.kiro/steering/workflow.md` and `specs.md`.

Plain ASCII only (`documentation.md`). This is a MAINLINE core change (it reworks
the document model's public surface), NOT `V`-stream plugin work; the mainframe
dataset work (`V`) builds ON this foundation.

Grounded in the read-only investigation `windowed-record-foundation/findings.md`
(what is built / unwired / must-be-revised) -- all "current state" claims trace
there.

---

## 1. Purpose and the problem being solved

Deliver the owner's long-standing intent: **every file, on every platform, loads
WINDOWED** -- only the visible window (plus overscan) and the user's edits live in
memory, never the whole file -- and the editor edits **records** (a native text
file's CRLF/LF/CR-delimited lines are records just as much as a mainframe FB
dataset's fixed-length records). Large files are not a mainframe-only concern (a
native log can be multi-GB), so windowing + records are a UNIVERSAL foundation;
the mainframe dataset work is a specialisation on top.

Problems in the current build this fixes (from findings.md):
- `tab_manager::open_file` reads the WHOLE file (`provider.read` ->
  `doc.insert(0, all)`); the gap buffer holds the entire file resident.
- The scrollbar / `max_top_line` size from `line_count` of the RESIDENT buffer,
  so a windowed buffer collapses the scrollbar to the window (the owner-observed
  bug).
- `SparseLineIndex` exists but is a transient scaffold: `finalize` rescans the
  whole buffer into a dense `LineIndex` and the sparse index is discarded.
- "Record" is not first-class; the mainframe provider FLATTENS records to
  `\n`-lines before the editor sees them, so FB fixed-length structure cannot be
  kept "real".

---

## 2. The spine: an immutable original index + a piece list of changes (piece-table)

The document is represented by THREE layers, cleanly separated so navigation,
editing, undo, and memory each have the structure they need:

```
(A) Immutable Original Index   original record K -> (file_offset, byte_length)
    - one lean entry per original record; NEVER mutated after the open scan
    - fast O(1) "where is original record K"; mmap/spill-friendly (immutable)

(B) Piece List (the "changes")  ordered pieces presenting the CURRENT document:
      Piece::Original { first_record, count }   -> resolves via (A)
      Piece::Edited   { buf_range, count }      -> bytes in the Append Buffer
    - every edit (insert/delete/move/copy/overtype) is a piece-list SPLICE
    - unedited spans stay as ONE Original piece, so the list is small
      (one entry per contiguous unedited run + one per edit), never 100M
    - each piece carries a running record-count so "current line N -> piece"
      is a quick search (linear now; order-statistic tree over PIECES later if
      a pathologically fragmented file ever needs O(log pieces))

(C) Append Buffer                bytes of inserted/edited records only
    - small; grows with edits, not file size
```

Underneath, the ORIGINAL record bytes are NOT resident: an Original piece means
"read these records from the file on demand via (A)'s offsets." Physical byte
residency is the Window (section 4).

Why this shape (owner's synthesis, refined):
- **Fast navigation** -- (A) is O(1) random access; the piece list is tiny.
- **Cheap edits anywhere** -- move/insert/copy/delete are piece splices, O(pieces),
  never an O(file) array shift and never a file rewrite. The immutable index is
  never shifted.
- **Undo/redo nearly free** -- because (A) and the file are immutable and (C) is
  append-only, undo = restore the previous piece-list arrangement. The piece list
  is small, so snapshot/journal of it is cheap (section 6).
- **Immutable (A) is paging-friendly** -- the one large structure (~1.6 GB at
  100M records) never mutates, so it can be memory-mapped from an on-disk cache
  or spilled with zero coherence risk (section 5 budget/degradation).

This is the classic piece-table pattern (immutable original + append-only change
record + ordered piece view). It is well-proven; we adopt it as the spine.

---

## 3. The universal Record abstraction

The editable unit is a RECORD, not a raw byte line. A record's FRAMING is
supplied by the owning Command Environment (CR-CH-053 D4/D4a):

```
RecordFormat (from the owning CE):
  Delimited { terminator: Crlf | Lf | Cr | Mixed }   // native NTFS/POSIX/mac; variable length
  Fixed     { lrecl }                                 // mainframe FB; position-terminated, no delimiter
  Variable  { max_lrecl, rdw }                        // mainframe VB; length-prefixed
```

- The native CE returns `Delimited` (generalising today's `LineEndMode`), so
  native editing is byte-identical to today in observable result.
- The mainframe CE returns `Fixed`/`Variable` (generalising
  `ff-dscatalog::Recfm` + LRECL), so FB structure is kept REAL and record-aware
  SAVE re-pads to LRECL. This REPLACES the provider's current flatten-to-`\n`.

The open-time index scan (section 5) frames the original file into records USING
the owning CE's `RecordFormat`: delimiter scan for `Delimited`, pure arithmetic
(`offset = K * lrecl`) for `Fixed` (no scan at all), RDW walk for `Variable`.

OPEN QUESTION for the gate: does `RecordFormat` live in `ff-document-model` (so
the editor is record-native and the model owns the type) or in `ff-vfs`
(alongside the provider seam)? Leaning `ff-document-model` with the CE SUPPLYING
a value, since the editor must be record-native end-to-end. (Reconcile with the
D4 EditChrome query -- the CE answers both "how are records framed" and "how
should the editor present/constrain them".)

---

## 4. Windowed byte residency (physical), decoupled from the index (logical)

The piece list + index are the LOGICAL document (whole file). The WINDOW is the
PHYSICAL byte cache (a slice of records' bytes actually in RAM).

Rules (each a gate criterion):
1. **Scrollbar / navigation extents size on the INDEX total record count, NEVER
   on the resident window.** (Fixes the owner-observed bug.) Two distinct counts
   must never be fused: `total_records` (whole file, from the index) vs
   `resident_window` (what bytes are loaded).
2. **The window is sized in RECORDS, generously, and is INDEPENDENT of zoom and
   viewport height.** It covers the smallest-plausible-font viewport x a factor +
   overscan, so zoom-out still fits.
3. **ZOOM NEVER LOADS.** Ctrl+wheel zoom changes only how many resident records
   are RENDERED, never which records are LOADED. (Prevents the owner-flagged
   zoom thrashing.) The paint path is already window-shaped (findings Q4:
   `build_display_list` + `paint.rs` fetch only `top_line..=end_line`).
4. **Load/evict is SCROLL-driven with HYSTERESIS + overscan**, debounced so a
   fast jump (`down 9999`) does ONE window load, not a cascade of boundary loads.
5. **Dirty (edited) pieces are PINNED** -- never evicted until saved, regardless
   of scroll. (They live in the Append Buffer anyway.)

`down 9999` / `up 9999`: resolve the target record via the index (O(1)/O(log
pieces)), load that window, paint. No intermediate records are read (large-file
-performance Req 5.3 already specifies this jump semantics).

---

## 5. Lifecycle, open behaviour, sizing, and the scalability guard

### Build on open, drop on close
The index (A) + piece list (B) are built on open and dropped on close (owner
directive). Rebuilt each open. No persisted index in the first build (a cached
on-disk index is a later optimisation, enabled by (A) being immutable).

### Open behaviour
- **Fixed-format (FB mainframe):** index is ARITHMETIC (`offset = K * lrecl`) --
  exact and INSTANT, no scan, any size. The largest datasets are the EASY case.
- **Delimited (native/VB):** OPEN INSTANTLY on the first window (first chunk
  only); run the delimiter-scan index build in the BACKGROUND via
  `ff-idle-processing`; show an ESTIMATED scrollbar (file_size / sampled avg
  record length) with a visible "indexing... N%" indicator (large-file
  -performance Req 6.2 "counting..." placeholder); SNAP to exact total when the
  scan completes. The scan is sequential delimiter-only (no UTF-8 decode, no
  per-line allocation), so it is far cheaper than loading-into-RAM, but at ~100M
  records it is several seconds -- hence the indicator is load-bearing UX.

### Sizing (target ceiling ~100M records; u64 numbers)
- **Record numbers: `u64`** everywhere (already `LineNumber`'s type). The TYPE
  has no ceiling; ~100M is a MEMORY-BUDGET target, not a type limit -- this is the
  "scalable, not unscalable" guarantee at zero cost.
- **Immutable index node: LEAN ~16 bytes** (file_offset u64 + byte_length u32 +
  flags u32). At 100M records that is ~1.6 GB resident. Fat tree-nodes-per-record
  (~48 B, ~4.8 GB) are RULED OUT as the default; the piece list (small) is where
  tree-like structure may appear, over PIECES not records.

| Records | Lean index (16 B/rec) |
|---------|-----------------------|
| 10,000,000 (10M, real-world seen) | ~160 MB |
| 99,999,999 (~100M, target ceiling) | ~1.6 GB |

### The scalability guard (above budget)
- Configurable `max_resident_records` (default ~150M, so 100M sits inside with
  headroom). Above it, DO NOT crash/hang: degrade to a reserved
  **sparse/windowed-index mode** (resident sparse checkpoints + per-window dense
  detail re-scanned on jump) -- navigate + reduced-cost edit. This mode is
  DESIGNED-FOR (the index is behind an interface that admits it) but BUILT LATER;
  the first build ships the lean fully-resident index only. Because (A) is
  immutable, the above-budget path can alternatively mmap/spill (A) rather than
  switch modes -- decided when the budget is actually hit.
- Configurable budgets (the knobs, matching the existing config pattern): window
  size (records), overscan, checkpoint interval, `max_resident_records`, spill
  threshold. NOT a configurable number type.

### The window as a 3-PAGE band (owner decision, resolves #6)
The resident byte window is a **3-page band** around the viewport:
- A **page** = the maximum number of records that fit a full screen at the
  SMALLEST zoom (most records on screen), so zoom-in never needs more than the
  resident band.
- The band = **current page + 1 page above + 1 page below**, the above/below
  pages PREFETCHED in the background via `ff-idle-processing`. This gives
  almost-instantaneous one-page Up/Down paging.
- Sustained scrolling (holding Page-Down, or a long drag) streams further pages
  on demand -- "slightly slower" as the file is read/buffered ahead, which is the
  acceptable cost past the prefetched band.
- This band IS the hysteresis/overscan of section 4: load/evict happens at the
  band edges, not the viewport edge, so small jitter near a boundary never
  thrashes. `down 9999` jumps the band to the target page (index lookup), paints,
  and prefetches its neighbours -- no intermediate pages read.

---

## 5a. Compaction vs re-baseline (two distinct mechanisms; resolves #5)

- **Compaction (mid-session, undo-PRESERVING):** over a long edit session the
  piece list fragments (many small Edited pieces). A background pass (via
  `ff-idle-processing`) MERGES adjacent compatible pieces to reclaim memory
  WITHOUT changing the document or dropping undo. Later phase; not day one.
- **Re-baseline (on SAVE, undo-DROPPING):** the owner's save policy (section 6
  SAVE) -- the just-saved file becomes the new immutable original, the piece list
  collapses, the index rebuilds, undo is dropped to that point. Default on save
  (bounded memory); a keep-undo-across-save alternative may be offered for small
  files.

These are NOT the same operation: compaction keeps undo and runs during editing;
re-baseline drops undo and runs at save. The design keeps them separate.

---

## 6. How each operation maps onto the spine

- **Navigation / scroll / goto / down-up N**: resolve current line N -> piece ->
  index -> file offset; load window if needed. Scrollbar from index total.
- **Line move / insert / copy / delete** (line commands): piece-list splices.
  Delete trims/splits pieces; insert/copy adds Edited (or Original-referencing)
  pieces; move re-orders pieces. No array shift, no file rewrite.
- **In-record text edit / overtype**: the edited record becomes an Edited piece
  pointing at new Append-Buffer bytes; original bytes untouched.
- **Undo / redo**: a journal of inverse piece-list splices (NOT whole-index
  copies -- that would blow memory). "Versions of the index" = snapshots/deltas of
  the SMALL piece list. Reconcile with the existing `ff-undo-redo` crate: if it
  is already operation-based, piece-splice ops slot in; confirm at the gate.
- **FIND / CHANGE across the whole file**: today `find_manager` snapshots the
  ENTIRE document (findings Q4). Rework to scan by RECORD RANGE on demand via the
  piece list + index + windowed byte reads. The find engine is already behind the
  `CharacterIndexer` trait, so a windowed indexer slots in without rewriting the
  engine. CHANGE edits produce Edited pieces, not a whole-buffer rewrite.
- **SAVE**: walk the piece list in order; emit each piece's bytes (Original via
  the file + index; Edited via the Append Buffer), RE-FRAMED per the owning CE's
  `RecordFormat` (re-pad to LRECL for Fixed, re-emit delimiter for Delimited,
  RDW for Variable). Atomic write; original file untouched until commit. The
  CR-CH-053 Task 20/21 SAVE-addressing-to-the-owning-CE path is the seam this
  rides (record-aware save lives in the owning CE). SAVE policy = RE-BASELINE
  (owner decision, resolves #5): after a successful save, the just-saved file
  BECOMES the new immutable original -- the piece list collapses to a single
  `Original{0..N}`, the index rebuilds, and undo history is dropped to that
  point. This keeps memory bounded (the whole point for large files). A
  configurable alternative (keep piece list + undo across save; memory grows)
  may be offered for small files where it is free; default is re-baseline.
  NOTE: re-baseline (save-time, undo-dropping) is DISTINCT from compaction
  (section 5a, mid-session, undo-preserving).

---

## 6a. Undo capacity, destructive-scale operations, and macro semantics (resolves #4)

Three DISTINCT problems, three distinct answers (do not conflate them):

### P1 -- Cumulative undo-memory pressure (slow accumulation over a session)
Many normal edits grow the piece list / undo journal over time.
- **Soft threshold:** a NON-BLOCKING advisory (status-bar note / gentle prompt):
  "Large amount of unsaved changes; consider saving. Undo history may be
  trimmed." Never blocks; never fires in a macro (headless).
- **Hard threshold:** begin TRIMMING the oldest undo entries (bounded undo
  depth). Edits remain; only deep undo is lost. Standard editor behaviour; keeps
  memory bounded without blocking. Macros: trim silently per policy.

### P2 -- A single destructive-SCALE operation (e.g. CHANGE ALL on a huge file)
One command can produce millions of changes at once (`CHANGE ALL 'a' 'b'` over
100M records). Two mechanisms, used together:
- **(i) Prefer a streaming transform where possible.** A global substitution
  need NOT materialise every record as an Edited piece: represent it as a pending
  TRANSFORM piece-op applied as bytes stream to disk at SAVE. In-memory cost
  near-zero; undoable (as one op) until save. Works for transforms expressible as
  a streaming rule.
- **(ii) When an op MUST materialise beyond the undo budget, it is a
  "destructive-scale" operation** -- it cannot be held in undo, so proceeding
  means undo is lost past that point. This is modelled as a CR-CH-053
  **Confirmable_Command** (REUSE the existing mechanism -- command-framework Req
  16 / the CANCEL/RETURN `-Y`/`-N` pattern -- do NOT invent new flag semantics):

  | Context | Behaviour |
  |---------|-----------|
  | Interactive, no switch | CONFIRM popup stating the consequence: "This will change ~N records and cannot be undone. Continue? [Confirm] [Cancel]" |
  | Interactive, `-Y`/`--yes` | Proceed immediately, no popup; undo dropped past this point |
  | Macro / headless, no switch | Does NOT hang on a popup. DEFAULT = REFUSE with a clear RC ("needs -Y: destructive-scale, undo would be lost"), so a macro cannot silently destroy undo by omission |
  | Macro / headless, `-Y` | Proceed, no prompt, undo dropped -- the macro explicitly opted in |

  The confirmation REASON is "exceeds undo budget / destructive-scale"; otherwise
  it is an ordinary Confirmable_Command. The undo-loss CONSEQUENCE is inherent in
  proceeding -- RECOMMENDATION: a SINGLE `-Y` whose prompt text states the
  consequence, rather than a dual `--force --confirm` (less friction; the loss is
  not a separate toggle). OWNER DECISION: **single `-Y`** (consequence stated in
  the prompt); NOT the dual `--force --confirm`.

### P3 -- Interactive vs macro/headless (how the popup is suppressed)
A macro runs HEADLESS, so by the Confirmable_Command rule it NEVER shows a popup
-- it carries `-Y` (proceed) or omits it (refuse-with-RC per P2). The macro author
decides per command. Optionally a macro-scope `ADDRESS <env> ASSUME YES` block
directive (REXX ADDRESS semantics the CE model already uses) sets the default for
a block so `-Y` need not be repeated -- opt-in and explicit, so a macro cannot
accidentally become destructive. OWNER DECISION: macro-no-flag =
**refuse-with-RC** (a macro cannot silently destroy undo by omission).

---

## 7. Crates: reused vs reworked (from findings)

REUSE / leverage (already built):
- Paint path (`ff-editor-panel::build_display_list`, `editor_panel/paint.rs`) --
  already window-shaped (fetches only the visible range). Keep; it feeds off the
  piece/window instead of the resident `LineIndex`.
- `CharacterIndexer` trait (`ff-find-and-replace`) -- windowed indexer slots in.
- `SparseLineIndex` (`ff-document-model`) -- repurpose as the RESIDENT checkpoint
  layer / seed for the above-budget sparse mode (stop discarding it in finalize).
- `ff-idle-processing` -- background index scan + overscan pre-fetch scheduler.
- `ff-background-io` -- large-file detection, cancellation, progress; its spec
  already reserves a `MAY` seek-based partial-load + record-format pass-through.
- `ff-dscatalog` Recfm/LRECL codec -- generalise into `RecordFormat::Fixed/Variable`
  (stop flattening to `\n` at the provider).
- `ff-large-file-performance` -- the render/measurement cache; wire it in for
  layout caching over the window (it is currently UNWIRED).

REWORK (the full-residency couplings to break, findings section 3):
- `Document` / `GapBuffer` / `TextBuffer`: introduce the piece-list +
  immutable-index + append-buffer + windowed-byte-cache model. The gap buffer
  survives as the Append Buffer for edited pieces. `Document`'s PUBLIC surface
  changes -- flag explicitly (framework-conformance: a foundational change).
- `StreamingFileReader`: from progressive-APPEND to ranged/windowed read +
  index-build (no full residency).
- `tab_manager::open_file` / `save_active_tab_via_backend`: windowed open; piece
  -list merge save.
- `find_manager`: windowed/ranged FIND/CHANGE (via `CharacterIndexer`).
- `viewport` / `max_top_line`: size from the index total (with
  estimated/"indexing..." total while a delimited scan is incomplete).

---

## 8. Existing requirements that must be REVISED (not just extended)

(From findings Q8 -- these currently ASSUME full residency and CONTRADICT
windowed loading; the gate must revise them, which is why this is owner-reviewed.)
- `document-model` Req 4 "Streaming File Loading" (esp. 4.5 "finalize into a
  COMPLETE index") + the fully-resident gap-buffer glossary -> revise to
  windowed/evict-off-window + the index as the surviving authority.
- `viewport-and-scrolling` Req 1.10 / Req 2 (`max_top_line` from a KNOWN total
  line count) -> revise to allow an estimated/"still-counting" total during a
  background delimited scan.
- `large-file-performance` Req 7 / 7.6 (range access "IF the document model
  supports range access") -> becomes MANDATORY under this directive.
- Introduce `record` as a first-class concept alongside `line` across
  document-model / viewport / display-line-mapping, with `RecordFormat`.

---

## 9. Safety rule and phasing

SAFETY RULE (non-negotiable, every phase): **native editing stays behaviourally
identical in observable result.** A native text file opened, edited, and saved
must produce byte-identical output to today (same bytes, same line endings, same
dirty/save-point behaviour), windowing and records notwithstanding. This is the
regression guard the whole rework is measured against (CR-CH-053 kept native SAVE
byte-identical; this continues that discipline).

Proposed phasing (MAINLINE; each phase keeps FFWB building + the safety rule):
- **F1 Record abstraction + immutable index + piece list (no windowing yet):**
  introduce `RecordFormat`, the immutable original index, the piece list, and the
  append buffer -- but still load the whole file into the index on open (prove the
  piece-table model with native files, byte-identical SAVE, undo via piece
  journal). Lowest-risk first step; no behaviour change.
- **F2 Windowed byte residency:** load record bytes on demand per the window;
  scrollbar sizes from the index total; zoom-never-loads; scroll hysteresis;
  dirty pinned. (The owner's scrollbar bug fixed here.)
- **F3 Background index build + open-time UX:** instant open + background
  delimited scan + estimated->exact scrollbar + "indexing..." indicator;
  arithmetic/instant for Fixed.
- **F4 Windowed FIND/CHANGE** via the `CharacterIndexer` windowed indexer.
- **F5 Scalability guard** (`max_resident_records` + reserved sparse/mmap
  above-budget mode).
Then the `V`-stream mainframe CE supplies `RecordFormat::Fixed/Variable` +
record-aware SAVE on this foundation (separate gate).

---

## 10. Open questions -- RESOLVED (owner, this review) + remaining gate-time checks

RESOLVED by owner:
1. `RecordFormat` home -> **`ff-document-model`** (the editor is record-native;
   the CE SUPPLIES the value).
3. Piece-list search -> **start LINEAR over pieces**; add an order-statistic tree
   over PIECES only if fragmentation profiling demands it.
5. Compaction vs re-baseline -> **SAVE = RE-BASELINE** (save the file as it
   stands, drop the index, rebuild; undo lost to that point) is the save policy
   (section 6 SAVE). Mid-session COMPACTION (merge adjacent pieces, undo
   -preserving, backgrounded via idle) is a SEPARATE, later-phase mechanism
   (section 5a). Default: background compaction during the session; re-baseline
   on save.
6. Window sizing -> **3-PAGE band** (current + above + below, prefetched), page =
   max records at smallest zoom (section 5 "3-page band"). `max_resident_records`
   default ~150M (headroom over the 100M target).
7. Delimited open: estimated-then-exact scrollbar -> **ACCEPTED**.

RESOLVED by owner (this review) -- #4:
4a. Destructive-scale confirm flag -> **single `-Y`** (consequence stated in the
    prompt); NOT the dual `--force --confirm`.
4b. Macro hitting a destructive-scale op WITHOUT `-Y` -> **refuse-with-RC** (a
    macro cannot silently destroy undo by omission).
    (The above-budget INDEX strategy -- sparse mode vs mmap/spill the immutable
    index -- remains a deferred IMPLEMENTATION choice decided when the budget is
    actually approached; the design admits both. Not blocking the gate.)

VERIFIED (read the crate, this review):
2. `ff-undo-redo` is OPERATION-based (good -- no snapshot rewrite needed): it
   exports `EditOperation` (Insert/Delete/Replace, each with an `inverse()`),
   `UndoableState`, `DocumentUndoManager` with transaction boundaries +
   coalescing, byte data held in a `ScrapStack`. SUBTLETY the gate MUST address:
   `EditOperation` is keyed by absolute byte `position: u64`, assuming a
   byte-addressable, fully-resident document. Under the piece-table/windowed
   model, edits are naturally in RECORD/PIECE terms (positions shift as pieces
   splice; absolute bytes are not stable when the file is not resident). So the
   operation model needs ADAPTATION from byte-position to record/piece
   addressing -- piece-splices fit the op+inverse shape, but the addressing
   changes. This is a design item for the gate, not a trivial slot-in.

---

## 11. Decisions already made (owner-confirmed in conversation)

- Universal windowed loading, all platforms, from the start (small files = a
  window covering the whole file).
- Records are universal; the owning CE supplies `RecordFormat`; mainframe builds
  on this foundation (no bespoke large-file machinery).
- Piece-table spine: immutable lean flat ORIGINAL index + a piece list of
  changes + append buffer; navigation fast, edits cheap, undo via piece versions.
- Target ceiling ~100M records; `u64` record numbers (no type ceiling);
  scalable-not-unscalable via budget + reserved above-budget degradation.
- Scrollbar sizes on the index total, NEVER the resident buffer; zoom never
  loads; scroll load/evict with hysteresis; dirty pieces pinned.
- Index + pieces built on open, dropped on close.
- Native editing stays behaviourally identical (the safety rule).
- `RecordFormat` lives in `ff-document-model`; the CE supplies the value (#1).
- Window = 3-page band (current + prefetched above/below); page = max records at
  smallest zoom; `max_resident_records` default ~150M (#6).
- SAVE = re-baseline (drop/rebuild index, undo lost to that point); mid-session
  compaction is separate and undo-preserving (#5).
- Destructive-SCALE ops (exceed undo budget) are CR-CH-053 Confirmable_Commands;
  prefer streaming-transform-at-save where possible; cumulative undo pressure
  handled by soft advisory + bounded-undo trim (#4). Confirm flag = single `-Y`
  (consequence in the prompt); macro-no-flag = refuse-with-RC (#4, owner-decided).
- Piece-list search starts linear; order-statistic tree over pieces only if
  needed (#3). Estimated-then-exact scrollbar for delimited open accepted (#7).
