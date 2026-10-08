# Findings: Universal Windowed-Loading + Record-Abstraction Foundation

READ-ONLY investigation. No code was changed. Worktree:
`c:/workspace/VSC/FileForgeWorkbench/.worktrees/env-registry` (branch
`feature/environment-registry`). All citations are from that worktree.

Content was rephrased for compliance with licensing restrictions where source
docs are quoted; all quotes are <= 30 words.

---

## 1. Summary answer (for the requirements-gate author)

The owner directive -- **every file on every platform loads WINDOWED (only the
visible window + overscan resident), records are UNIVERSAL, the editor edits
records, and mainframe builds on this foundation** -- is **"build missing
pieces", NOT "wire existing crates".**

The decisive distinction the brief asked for:

> **Nothing in the stack today does RANGED/WINDOWED access.** Every loading path
> that exists is PROGRESSIVE-APPEND: it streams the whole file in chunks and the
> whole file ends up resident in one in-memory buffer. There is no concept of
> "load records N..M and evict off-window".

Secondary conclusion: **"record" is not a first-class concept** anywhere in the
document model. The editor edits *lines* derived from a fully-resident byte
buffer; the only `Recfm`/LRECL type lives in the mainframe catalog crate and is
converted to `\n`-delimited bytes by the mainframe VFS provider *before* the
editor ever sees it. So there is no universal `RecordFormat` abstraction to
build on -- it must be created.

Status by area:

| Area | Verdict | One-line reason |
|------|---------|-----------------|
| `ff-large-file-performance` | **BUILT but UNWIRED, and WRONG LAYER** | Real render/measurement cache with tests; nothing consumes it; it is NOT a loader |
| Document-model streaming (`StreamingFileReader`) | **BUILT but PROGRESSIVE-APPEND + UNWIRED** | Loops `buffer.insert(len, chunk)` to EOF; whole file resident; not used by the editor |
| `SparseLineIndex` | **BUILT but requires full buffer + UNWIRED** | `finalize` rescans the whole buffer; cannot index a window |
| Gap buffer / `Document` | **BUILT, fully-resident by design** | One `Vec<u8>` holds all content; no paged/partial backing store |
| Editor RENDER path | **PARTIAL (render-windowed over resident doc)** | Paint reads only `top_line..=end_line`, but off a fully-resident `Document` |
| Editor FIND/CHANGE path | **MISSING (whole-document resident)** | `find_manager` snapshots the ENTIRE document to a `SliceIndexer` every op |
| `tab_manager` open/save | **MISSING (whole-file)** | `open_file` reads all bytes then `doc.insert`; save writes `contiguous_view` |
| Universal `RecordFormat` | **MISSING** | Only `LineEndMode` (delimiters) + mainframe-only `Recfm`; no shared abstraction |
| `ff-background-io` | **BUILT but PROGRESSIVE + UNWIRED** | Streams to EOF via callback; `MAY` seek is spec-only, unimplemented; no dep from app |
| `ff-idle-processing` | **BUILT but UNWIRED** | Cooperative idle scheduler; no crate depends on it |

---

## 2. Evidence by question

### Q1 -- `ff-large-file-performance`: REAL but UNWIRED, and it is the wrong layer

**REAL.** `crates/ff-large-file-performance/src/lib.rs` exports eleven modules
(`position_cache`, `line_layout_cache`, `line_layout`, `chunk_renderer`,
`invalidation`, `scroll_predictor`, `surface`, `status`, `config`, `types`,
`error`). Each has substantive logic and inline `#[cfg(test)] #[test]` units
(e.g. `line_layout_cache.rs` ~11 KB with 10+ tests; `position_cache.rs`,
`invalidation.rs`, `config.rs` similarly). No `todo!`/`unimplemented!` stubs
were found. The full gate reported 9645 tests passing (user message index 8),
so these compile and pass.

**WRONG LAYER for this directive.** It is a *rendering/measurement* cache, not a
file loader. `line_layout_cache.rs:16` keys entries by **document line number**
(`entries: HashMap<u64, LineLayout>`); `compute_max_entries` only sizes a cache
(`Viewport`/`Page`/`Document` levels). Its only dependency is `thiserror`
(`Cargo.toml`) -- it does NOT depend on `ff-document-model` and never reads a
file. It presumes some *other* layer supplies line content.

**UNWIRED.** Grep for `ff-large-file-performance` / `ff_large_file_performance`
across the workspace (`tools/logs/lfp-usage.txt`) returns ONLY: the workspace
member list (`Cargo.toml:62`), its own `lib.rs`/`error.rs`, and its own
`Cargo.toml`. **No crate depends on it; nothing consumes it.** It is absent from
`crates/ff-desktop/Cargo.toml`.

### Q2 -- Document-model streaming + indexing: PROGRESSIVE-APPEND, not windowed

`crates/ff-document-model/src/streaming.rs`, `StreamingFileReader::load`:

```rust
loop {
    ... let n = stream.read(&mut chunk_buf).await ...; // Ok(0) => break
    buffer.insert(buffer.length(), chunk);  // APPEND every chunk
    sparse_index.process_chunk(chunk, mode);
}
```

This appends each chunk at `buffer.length()` until EOF. **The whole file ends
resident in the `GapBuffer`.** There is no `N..M` range parameter, no eviction,
no off-window discard. This is the exact "progressive-append" the brief warns is
NOT windowing.

`crates/ff-document-model/src/sparse_line_index.rs`:
- `process_chunk` records `(line_number, byte_position)` checkpoints as it scans
  chunks -- fine for progressive display.
- `finalize(self, buffer: &mut GapBuffer, mode)` calls
  `index.rebuild_from_buffer(buffer, mode)` -- it **rescans the FULL buffer** to
  build the real `LineIndex`. So the completed index requires the whole file
  resident.
- `approximate_line(position)` is a checkpoint binary-search for a *byte
  position already known*; it cannot map display-line -> byte-offset for a
  window that was never loaded.

Key signatures (quoted exactly):
- `pub async fn load(&self, vfs, uri, buffer: &mut GapBuffer, sparse_index: &mut SparseLineIndex, mode: LineEndMode) -> Result<LoadingProgress, DocumentError>`
- `pub fn finalize(self, buffer: &mut GapBuffer, mode: LineEndMode) -> LineIndex`
- `pub fn approximate_line(&self, position: BytePosition) -> Option<LineNumber>`

Both `StreamingFileReader` and `SparseLineIndex` are **UNWIRED**: grep outside
`ff-document-model` (`tools/logs/stream-usage.txt`) finds zero references to
`StreamingFileReader` or `SparseLineIndex`. (The `read_stream` hits are the VFS
trait method, used by `ff-vfs/src/search.rs` and `ff-background-io`, not by the
editor loader.)

### Q3 -- Gap buffer / Document: fully resident, no paged backing store

`crates/ff-document-model/src/gap_buffer.rs`: `GapBuffer { storage: Vec<u8>,
gap_start, gap_end, growth_factor }`. One contiguous allocation holds all
content plus the gap. `length()` = storage minus gap. There is **no partial /
windowed / paged variant** anywhere in the crate.

`crates/ff-document-model/src/document.rs`: `Document { buffer: TextBuffer, ...
viewport: Viewport ... }`. All text access (`char_at`, `get_range`,
`contiguous_view`, `line_start`, `line_end`) indexes the single resident buffer.
`contiguous_view(&mut self)` compacts the gap and returns `&[u8]` over the whole
content. **`Document` has NO `load`/`save` method** -- the file read/insert is
done by the caller (`tab_manager`, Q5). The load API the editor actually uses is
whole-file `insert(BytePosition(0), &bytes)`.

### Q4 -- Editor render is window-oriented; FIND/CHANGE is whole-document

**RENDER: already windowed over a resident doc (PARTIAL win).**
`crates/ff-editor-panel/src/lib.rs::build_display_list(top_line, end_line,
doc_line_count, lines: &[String], blocks)` iterates only `top_line..=actual_end`
and consumes a **pre-fetched** `lines` slice. The caller
`crates/ff-desktop/src/editor_panel/paint.rs` (around line 41) computes
`end_line = top_line + visible_lines - 1` and fetches ONLY that range:

```rust
let actual_end = end_line.min(line_count);
(top_line..=actual_end).map(|ln| { let start = doc.line_start(..); ... doc.get_range(start, len) ... }).collect()
```

So paint touches only the visible window (satisfying large-file-performance Req
4.1/4.4 "do not iterate all lines during paint"). **BUT** `line_start`/
`line_end`/`get_range` all resolve against the fully-resident `LineIndex` +
`GapBuffer`. It is "render-windowed over a fully-resident document", not a
windowed loader.

**FIND/CHANGE/EXCLUDE: whole-document resident (MISSING).**
`crates/ff-desktop/src/find_manager.rs` -- the module comment states it
"Snapshots the active document into a `SliceIndexer`":
- `snapshot_bytes` (lines ~150): `doc.get_range(BytePosition(0), doc.length())`
  -- reads the **ENTIRE document** into a `Vec<u8>` on every FIND/RFIND/CHANGE.
- `find`/`rfind`: `let indexer = SliceIndexer::new(&bytes);` over all bytes.
- `change`/`rchange`: builds `MutableSliceIndexer` from the whole string, then
  `write_bytes` **deletes the whole document and re-inserts** the new buffer
  (`doc.delete(0, len)` then `doc.insert(0, bytes)`).

The underlying engine (`crates/ff-find-and-replace/src/indexer.rs`) is actually
abstracted behind a `CharacterIndexer` trait (`line_count()`, `content() ->
&[u8]`, range finders like `find_all_in_range`), so the *engine* could in
principle run against a windowed indexer -- but the only concrete indexers
(`SliceIndexer`, `MutableSliceIndexer`) wrap a full in-memory slice, and the
shell always feeds them the whole document.

Navigation/line commands (`ff-navigation-commands`) mostly operate on the
viewport model (`ViewportModel::with_line_count`, `top_line` arithmetic) and on
small `lines: &[&str]` slices (`word.rs`, `paragraph.rs`, `sort.rs`), so they
are already content-sliceable; the coupling is that the shell sources those
slices from the resident document.

### Q5 -- `tab_manager`: whole-file open and whole-buffer save (MISSING)

`crates/ff-desktop/src/tab_manager.rs::open_file` (around line 1219):
```rust
let bytes = runtime.block_on(async { provider.read(path).await ... })?;  // WHOLE file
let document = new_document();
... doc.insert(BytePosition(0), &bytes); ...                              // insert ALL bytes
let (line_count, line_end_mode) = ...(doc.line_count(), doc.line_end_mode());
```
It uses `LocalFsProvider::read` (whole-file), **not** `StreamingFileReader`.

`save_active_tab_via_backend` (line 1269): `let bytes = doc.contiguous_view()
.to_vec();` then `backend.save(&path, &bytes)` -- writes the **whole buffer**.

Whole-document-resident assumptions in this file:
- **open**: `provider.read` -> `doc.insert(0, all)`; `line_count` from full doc.
- **save**: `contiguous_view()` of the entire buffer.
- **duplicate detection**: by `t.path.as_deref() == Some(path)` (fine, path-based).
- **line_count**: cached on `TabState` from the fully-indexed document.
- **scroll**: viewport arithmetic uses the full `line_count`.

### Q6 -- Delimiters are "lines", not "records"; `Recfm` is mainframe-only

`crates/ff-document-model/src/line_end.rs`: `LineEndMode { Default, Unicode }`.
`Default` recognises CR / LF / CRLF; `Unicode` adds NEL/LS/PS. So **native
delimiters ARE detected** -- but the model's concept is a *line* (a delimiter
scan over the byte buffer), produced by `LineIndex` (`line_index.rs`:
`line_starts: Vec<u64>` over the resident buffer). The word "record" never
denotes a first-class type in `ff-document-model` -- it appears only as "line
records" / "log records" (spec grep `tools/logs/bgrecord.txt`).

A record-format type DOES exist, but only for mainframe catalog metadata:
`crates/ff-dscatalog/src/dataset.rs:56` `pub enum Recfm { F, FB, V, VB, U }`,
with `lrecl`/`blksize` on `AllocParams`/`DatasetRecord`. Critically, the
mainframe VFS provider already **decodes records into `\n`-delimited bytes at
read time** and re-encodes on write: `crates/ff-dscatalog/src/vfs_provider.rs`
(around lines 865-905) treats `\n` as the record separator for RECFM=U and
decodes RDW records for VB "no CRLF". So by the time the editor sees mainframe
content, records have been flattened to lines -- records are NOT first-class in
the editor today.

A universal `RecordFormat` abstraction (native = delimiter-terminated variable;
FB = fixed position-terminated; VB = length-prefixed) could *generalise* `Recfm`
+ `LineEndMode`, but it does not exist and would be new.

### Q7 -- `ff-background-io` + `ff-idle-processing`: BUILT, PROGRESSIVE, UNWIRED

`ff-background-io` (`crates/ff-background-io/src/load.rs::execute_load`): queries
`vfs.stat` for size, flags `is_large_file` against a threshold, opens
`vfs.read_stream`, then **loops reading chunks and delivering each via
`chunk_callback` to EOF** (`if bytes_read == 0 { break }`). It has large-file
*detection*, progress throttling, and cooperative cancellation -- but it is
**progressive-append**: it has no "read records N..M then stop", no eviction.
(The only partial-load notion is spec-level: background-io Req says a provider
with random-access reads **MAY** use seek-based partial loading "rather than
streaming the entire file" -- that `MAY` is NOT implemented.)

`ff-idle-processing` (`crates/ff-idle-processing/src/lib.rs`): a cooperative
idle-time scheduler (`IdleScheduler`, `IdleWorkSource`, `WorkPriority`,
time-slicing, cancel-on-input). It is exactly what a windowed loader would use
to pre-compute/pre-fetch overscan during idle -- but it is pure infrastructure.

**Both UNWIRED.** Grep of all `Cargo.toml` (`tools/logs/idlebg.txt`) shows
`ff-background-io` and `ff-idle-processing` appear ONLY in their own manifests.
`crates/ff-desktop/Cargo.toml` depends on NEITHER (nor on
`ff-large-file-performance`). The app does not use any of the three
"performance/large-file" crates.

### Q8 -- Specs: windowing is specified for RENDER, loading is PROGRESSIVE, "record" absent

**Specifies windowed RENDER (not loading):** `large-file-performance/
requirements.md` is thorough on viewport/overscan rendering: Req 4.1 compute/
paint only `top_line..top_line+visible_count-1`; Req 4.4 "SHALL NOT iterate over
all document lines during a paint cycle ... O(visible_count)"; Req 5.1 do not
measure outside viewport+overscan; Req 3.2 cache levels `Viewport`/`Page`/
`Document`. These are satisfied by the paint path (Q4) but are about *layout
caching*, not file residency.

**Loading is specified as PROGRESSIVE, not ranged:** `document-model/
requirements.md` Req 4 "Streaming File Loading": Req 4.1 load in 64 KB chunks;
Req 4.3 build `SparseLineIndex` incrementally; Req 4.5 on completion "finalize
the LineIndex from the sparse checkpoints into a complete index". The glossary
calls `StreamingFileReader` the reader "enabling progressive display before the
full file is indexed". **Nothing requires evicting off-window content**; the
model assumes the file becomes fully resident and fully indexed.

**One optional ranged hook exists:** `background-io/requirements.md` Req (line
183) -- a provider that declares random-access reads **MAY** do seek-based
partial loading "rather than streaming the entire file". And Req (line 186)
passes through provider "record format, line-ending type" metadata to the
document model untouched. These are hooks a universal design could lean on, but
they are permissive (`MAY`) and unimplemented.

**"Record" vs "line":** across document-model/viewport/display-line-mapping/
large-file-performance, the editable unit is always a *line* (document-line vs
display-line). "Record" never names an editable unit. (`bgrecord.txt`,
`specgrep.txt`.)

**Direct contradictions with universal windowed loading:**
- `large-file-performance/requirements.md` Req 7 ("Memory_Efficient_Storage")
  only *hopes* the document model offers range access: "SHALL obtain line
  content ... without requiring a contiguous copy of the entire file" and Req
  7.6 "request only the needed sub-range ... **if the document model supports
  range access**". Today it does not (gap buffer is fully resident), so this
  criterion is written against a capability that must still be built.
- `document-model/requirements.md` Req 4.5 (finalize into a *complete* index)
  and the Glossary (whole-file resident gap buffer) assume full residency, which
  conflicts with "never hold the whole file; evict off-window". These criteria
  must be revised, not merely extended.
- `viewport-and-scrolling/requirements.md` Req 1.10 / Req 2 compute
  `max_top_line` from `line_count` / `total_display_lines` -- i.e. the TOTAL
  line count must be known. Under true windowed loading of a not-fully-indexed
  file, total line count is not yet known, so scrollbar range (Req 4) and
  `max_top_line` clamping need an "indexing in progress / estimated total" story
  (the specs already hint at a "counting..." placeholder in
  large-file-performance Req 6.2, so this is reconcilable but currently
  under-specified).

---

## 3. Whole-document-resident couplings universal windowed loading must break

Concrete code sites that assume the full document is in memory:

1. **`StreamingFileReader::load`** (`ff-document-model/src/streaming.rs`) --
   `buffer.insert(buffer.length(), chunk)` to EOF. Needs a ranged read
   (`records/bytes N..M`) + an evictable backing store.
2. **`SparseLineIndex::finalize`** (`.../sparse_line_index.rs`) --
   `rebuild_from_buffer` rescans the whole buffer. Needs a window-local index
   that maps display-line -> byte-offset without full residency.
3. **`GapBuffer` / `TextBuffer` / `Document`** (`gap_buffer.rs`, `text_buffer.rs`,
   `document.rs`) -- single resident `Vec<u8>`; `contiguous_view` compacts the
   whole thing. Needs a paged/windowed backing store (or a buffer-over-window
   with a page cache + dirty-page writeback).
4. **`tab_manager::open_file`** -- `provider.read(path)` (whole file) then
   `doc.insert(0, all)`. Needs windowed open via the streaming/ranged path.
5. **`tab_manager::save_active_tab_via_backend`** -- `contiguous_view().to_vec()`
   whole-buffer write. Needs windowed/dirty-region save (and, for FB/VB, a
   record-format-aware encoder; the mainframe provider already does record
   codec, so the pattern exists to generalise).
6. **`find_manager::snapshot_bytes` + `write_bytes`** -- reads the entire
   document per FIND/CHANGE and rewrites the whole buffer on CHANGE. Needs a
   windowed/streaming search that scans ranges on demand (and the `CharacterIndexer`
   trait already allows a non-slice implementation).
7. **`viewport`/`max_top_line` total-line dependency** (`viewport.rs`,
   `ff-viewport-scrolling`) -- needs an "estimated / still-counting" total-line
   source while indexing is incomplete.
8. **Record flattening in `ff-dscatalog/src/vfs_provider.rs`** -- mainframe
   records are decoded to `\n`-delimited bytes before the editor sees them, so
   the editor cannot preserve FB fixed-length structure (the owner's "keep the
   Fixed record length structure real" requirement). A universal `RecordFormat`
   owned by the Command Environment must replace this flatten-at-provider
   approach so the editor edits real records.

---

## 4. Conclusions and recommendations (gate author)

1. **This is a build, not a wiring job.** The three "performance" crates
   (`ff-large-file-performance`, `ff-background-io`, `ff-idle-processing`) are
   real and UNWIRED, but none of them is a windowed/ranged *loader* -- they are a
   render cache, a progressive streamer, and an idle scheduler. The missing
   pieces are: (a) a windowed/paged document backing store with eviction, (b) a
   window-local line/record index, (c) a universal `RecordFormat` abstraction
   supplied by the owning Command Environment, (d) windowed open/save, and (e)
   windowed FIND/CHANGE.

2. **Leverage what exists.** The RENDER path is already window-shaped
   (`build_display_list` + `paint.rs` fetch only `top_line..=end_line`); the find
   engine is already behind a `CharacterIndexer` trait; mainframe RECFM decode/
   encode already exists in the dscatalog provider; background-io already carries
   large-file detection, cancellation, and progress; idle-processing is ready for
   overscan pre-fetch; background-io's spec already reserves a `MAY` seek-based
   partial-load path and a record-format metadata pass-through. A universal
   design should generalise these rather than add parallel machinery.

3. **Record abstraction belongs to the Command Environment, per the owner.** The
   native CE supplies a delimiter-terminated variable `RecordFormat` (generalising
   `LineEndMode`); the mainframe CE supplies fixed (FB) / length-prefixed (VB)
   formats (generalising `ff-dscatalog::Recfm` + LRECL). The editor should edit
   `RecordFormat`-typed records uniformly; the mainframe path then "builds on"
   the universal foundation instead of flattening records to lines at the
   provider.

4. **Expect to REVISE, not just extend, several criteria.** `document-model` Req
   4.5 (finalize to a complete index) and the fully-resident gap-buffer glossary,
   plus `viewport` `max_top_line`'s dependence on a known total line count,
   directly assume full residency and must be reworked for evict-off-window
   behaviour and incremental/estimated totals. `large-file-performance` Req 7/7.6
   are already written conditionally ("if the document model supports range
   access") and become mandatory under this directive.

5. **Framework-conformance note (`framework-conformance.md`).** A windowed
   `RecordFormat`-aware document model and record-aware save are a document-model/
   platform change, not a shell dispatch/navigation/focus change. The editor and
   FIND/CHANGE rework should route through the existing seams (editor paint
   already window-shaped; `CharacterIndexer` trait; CE/backend `save`). Flag the
   backing-store replacement explicitly to the owner as a foundational change (it
   touches the public `Document`/`GapBuffer` surface).

No code, tests, or docs were modified. Scratch grep logs were written under
`tools/logs/` (git-ignored, ephemeral per `tooling.md`).
