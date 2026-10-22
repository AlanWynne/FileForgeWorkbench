# FFEDIT File-Interface Design -- FINALIZED

Finalized design for how FFEDIT (the editor Command Environment) works with files
of different internal mechanisms (host NTFS/POSIX, mainframe datasets, VSAM),
with forward-fit for the JES emulator. This supersedes `design-proposal.md` by
folding in the OWNER's answers to the open questions and the two NEW constraints
(cross-platform save-as, universal catalog). The proposal is left intact for
reference; this file is the decision-of-record.

STATUS: FINALIZED DESIGN FOR HUMAN REVIEW. No code changes made. BUILD DEFERRED
until the core stabilises (owner decision 7). Every architectural claim is cited
to a file/trait/function actually read. "Implemented today" and "Proposed/
Finalized" are kept strictly separate.

Plain ASCII only (per `documentation-charset.md`).

---

## Summary answer (read this first)

The owner has settled the single biggest decision: OPEN is now **symmetric with
SAVE** -- it is always addressed to the owning environment, which returns the
correct file attributes and a uniform record/line stream, rather than the shell
pushing raw host bytes + attributes in as command params. This commits the design
to a controller-returns-(attributes, records) model on BOTH ends.

Two code-facts were verified against the current tree and they make the finalized
design substantially EASIER than the proposal feared:

1. **The platform-agnostic record window the owner wants ALREADY EXISTS in the
   document model.** `ff_document_model::window_band::{WindowBand, RecordRange}`
   (CR-CH-058 Req 12.4-12.8) windows over **RECORD numbers, not bytes**, keeps
   `total_records` (the scrollbar/navigation authority) strictly separate from the
   resident 3-page band, and resolves far jumps in one shift. It is a pure
   document-model primitive with no egui/shell/platform dependency. The windowing
   tool therefore ALREADY does not know the platform -- it pages over records. The
   gap is only that the band's byte read/evict is still fed from a whole-buffer
   byte load (`streaming.rs`), not from a record source. (Section 5.)

2. **The catalog model is UNIVERSAL in shape, mainframe-only in the record-access
   path.** `ff_catalog_registry::CatalogType` has `Mainframe`, `Posix`, and
   `Native` variants and `VirtualCatalog` represents all three
   (`crates/ff-catalog-registry/src/lib.rs`); the File Explorer spec renders three
   section headers "Mainframe Catalogs / POSIX Catalogs / Native Catalogs"
   (`docs/specs/file-tree-panel/requirements.md` Req 14/17). So EVERY file already
   belongs to a catalog -- the owner's original intent is intact in the model. BUT
   the SQLite/record delegation (`CatalogRegistry::allocate/list_datasets/
   resolve_dsn`, `open_dscatalog`) filters to `CatalogType::Mainframe`, and
   `DatasetAccess`/`CatalogVfsProvider` are mainframe-dataset constructs. So
   catalog RESOLUTION is universal; catalog RECORD ACCESS is mainframe-only today.
   This lets us design the clean universal-resolution target while keeping a
   workable near-term path. (Section 7.)

The shared record contract the design leans on -- `ff_dscatalog::DatasetAccess`
(CR-CH-059) -- is unchanged and already designated for the editor, IDCAMS, and the
future JCL executor, so the JES forward-fit holds (Section 8).

Net: the finalized design adds ONE single, immutable per-open attribute value
(`FileAttributes`, the source environment's read attributes -- NO read/write
split), models SAVE as a trivial same-environment write and SAVE-AS as a distinct
EXPORT+TRANSFORM pipeline that owns the cross-platform concern, adds ONE additive
record-aware LOAD entry on `BackendEnvironment` (`open_records`, owner-authorized,
object-safe, provided default -- symmetric with the existing `save_records`),
wires a uniform record/line stream into the EXISTING `WindowBand`, flows a
first-class encoding attribute through decode/encode, and moves catalog resolution
toward the universal model that dissolves the `mainframe_backend_for_active_tab`
leak.

---

## 1. Decisions locked (settled design axioms)

These are no longer open. They are the axioms the rest of the document builds on.

1. **AX-OPEN-SYMMETRY.** OPEN is ALWAYS addressed to the owning environment. The
   environment/controller returns the file attributes (authoritative, from the
   source of truth for that file) and the content as a uniform record/line stream.
   The shell never pushes raw host bytes + attributes in as `file.open` params.

2. **AX-OPEN-RECORDS.** `BackendEnvironment` gains an additive, object-safe,
   record-aware LOAD entry (`open_records`) symmetric with `save_records`, with a
   provided default so host CEs are unchanged. This is a confirmed, owner-
   authorized reshape of a load-bearing core type (CR-CH-060 extended). ff-vfs
   stays dependency-free (plain-data types only; no `ff-dscatalog` edge).

3. **AX-SAVE-VS-SAVEAS (owner refinement).** SAVE and SAVE-AS are TWO DISTINCT
   operations, not one parameterized save.
   - **SAVE** writes back to the SAME environment the file was opened from, using
     the file's existing (read) attributes unchanged. Source env == target env,
     ALWAYS. No user prompting, no new attributes.
   - **SAVE-AS** is an EXPORT + TRANSFORM operation: it takes the in-memory
     record/line content, TRANSFORMS it to a destination environment's format
     (re-frame records to target RECFM/LRECL, re-encode to target encoding, apply
     target EOL/case rules), and writes it to the destination under a NEW identity.
     The user supplies the destination attributes because they are choosing a new
     target.
   CONSEQUENCE: the opened file's `FileAttributes` is SINGLE and IMMUTABLE for the
   life of the open -- it is the source/owning environment's read attributes,
   period. There is NO read/write attribute split on the handle and NO mutable
   "target" carried on the tab. The cross-platform (source != target) concern lives
   ENTIRELY inside the SAVE-AS / export-transform operation, which builds its OWN
   destination `StoreTarget` + write attributes locally and leaves the open
   document's attributes untouched.

4. **AX-WINDOW-PLATFORM-AGNOSTIC.** The windowing tool does NOT know the platform.
   It pages over a UNIFORM record/line abstraction that the owning environment
   produces by decoding platform specifics (EBCDIC, RECFM framing, host EOL). This
   is realized on the EXISTING `WindowBand` record-range primitive.

4a. **AX-ALWAYS-WINDOWED (owner decision 4).** The editor is ALWAYS windowed, for
   EVERY file including host text, from the beginning. There is NO non-windowed
   browse/edit path -- windowed record/line access is a FOUNDATIONAL property of
   the editor, not a dataset-first capability with host following later. A host
   line IS a Delimited record, so the host `RecordCursor` is a first-class, EARLY
   piece. CONSEQUENCE: the whole-file `StreamingFileReader::load` path
   (`crates/ff-document-model/src/streaming.rs`) is REPLACED by band-driven
   `seek_record`/`next_record` loading for ALL files, not just datasets; this is
   not deferred as a late dataset-only step (see Section 10 reordering).

5. **AX-UNIVERSAL-CATALOG.** Every file belongs to a catalog (Mainframe, POSIX, or
   Native). Catalog resolution is the universal mechanism by which a file resolves
   its owning environment + path + attributes. The target state routes host files
   through their Native/POSIX catalog exactly as datasets route through Mainframe
   catalogs, which dissolves the `mainframe_backend_for_active_tab` leak.

6. **AX-ENCODING-FIRST-CLASS.** Encoding is a first-class file attribute (EBCDIC
   code pages e.g. cp037 for datasets; utf-8/other for host), carried on the open
   `FileAttributes.record` (read/decode) and on the transient `ExportTarget.record`
   for a save-as (write/encode), replacing the hardcoded `"utf-8"` in
   `save_active_tab_via_backend` (`crates/ff-desktop/src/tab_manager.rs:1351`).

7. **AX-DESIGN-NOW-BUILD-LATER.** The FULL design is finalized now (symmetric open,
   open_records, cross-platform save-as, universal catalog, encoding). BUILD is
   DEFERRED until the core stabilises. Section 10 gives the dependency-ordered
   implementation sequence for when build resumes.

---

## 2. As-built delta (what changed in understanding since the proposal)

The proposal's as-built analysis (two ff-vfs seams, the open/save paths, the
`mainframe_backend_for_active_tab` leak, the split attribute channel, the
`DatasetAccess` forward-fit) is CONFIRMED and not repeated. Two findings are NEW
or sharpened and they reshape the design.

### 2.1 NEW: the record window already exists (and is already platform-agnostic)

The proposal treated dataset windowing as an unsolved risk ("feeding a record
stream to a byte/line document ... non-trivial", proposal E.4). That risk is
smaller than stated, because a record-addressed window primitive is ALREADY
BUILT:

- `ff_document_model::window_band::WindowBand` + `RecordRange`
  (`crates/ff-document-model/src/window_band.rs`, CR-CH-058 F2). The band is
  expressed in RECORD numbers; `total_records()` (the navigation authority) is
  deliberately NEVER fused with the resident band span (Req 12.5, documented as the
  fix for the "scrollbar collapses to the window buffer" bug). It supports
  `scroll_to_record` with hysteresis (Req 12.7), one-shift `jump_to_record`
  (Req 12.8), and dirty-record pinning for eviction (`evictable`).
- It is a PURE primitive: its doc-comment states "nothing here depends on egui or
  the shell" and "the actual byte read/evict ... live in the shell/viewport
  layer". So it already knows nothing about mainframe vs host -- it pages records.
- The record model on the document is also present:
  `Document::{total_records, record_start, record_byte_length, record_from_position}`
  (`crates/ff-document-model/src/document_records.rs`, CR-CH-058 Req 11/12), and
  `RecordFormat` has `Delimited { terminator } / Fixed { lrecl } / Variable { .. }`
  (`crates/ff-document-model/src/record_format.rs`). A Delimited record IS a line
  (Req 11.3), so host text and datasets already share one record abstraction.

What is NOT yet wired (the real gap): `streaming.rs`
(`StreamingFileReader::load`) reads the WHOLE file in 64 KB byte chunks over
`Vfs::read_stream` into a `GapBuffer` to EOF -- it is a whole-buffer byte loader,
not a record-windowed loader. And `CatalogVfsProvider::read` loads the entire
dataset file and decodes ALL records at once (`random_access: false` in its
`capabilities()`, `crates/ff-dscatalog/src/vfs_provider.rs`). So the band exists
but is fed by a whole-file load. The finalized windowing design (Section 5) is
therefore "feed the EXISTING band from a record source", not "invent windowing".

### 2.2 NEW: the catalog model is universal in shape, mainframe-only in access

- Universal shape (CONFIRMED): `CatalogType::{Mainframe, Posix, Native}` and
  `VirtualCatalog { name, catalog_type, path, default_hlq, mount_point, read_only,
  .. }` (`crates/ff-catalog-registry/src/lib.rs`). `section_label()` yields
  "Mainframe Catalogs / POSIX Catalogs / Native Catalogs". File Explorer spec
  requires all three sections and generic `NodeType::CatalogRoot` modelling
  (`docs/specs/file-tree-panel/{requirements.md,tasks.md}`: Req 14/17, Task 26.10
  "Model Mainframe/POSIX catalog roots generically ... POSIX/Native roots list
  their host directory"). So every file already conceptually belongs to a catalog.
- Mainframe-only access (CONFIRMED): the record/SQLite delegation
  `CatalogRegistry::{allocate, list_datasets, resolve_dsn}` all go through
  `open_dscatalog`, which filters `c.catalog_type == CatalogType::Mainframe`
  (`crates/ff-catalog-registry/src/lib.rs`). `DatasetAccess` and
  `CatalogVfsProvider` (scheme `"catalog"`) are dataset constructs that resolve a
  DSN through `ff-volume` indirection (`catalog_resolve.rs::resolve_locator`).
  Native/POSIX catalogs today resolve to a host directory and open through
  `LocalFsProvider`, NOT through a record-access contract.

Consequence for the design: the universal-catalog TARGET (AX-UNIVERSAL-CATALOG) is
"every catalog type can answer 'resolve this file to (owning_env, locator,
attributes)' and 'open it as a record/line stream'". The Native/POSIX side already
resolves to a path; it needs a record/line adapter (trivial: a host line IS a
Delimited record). The mainframe side already has the record adapter
(`DatasetAccess`); it needs its catalog name->path resolution reachable at
save-time (removing the leak). Near-term vs target handled in Section 7.

### 2.3 CONFIRMED: encoding infrastructure exists and is unused on the save path

- `ff-encoding` is a full encoding subsystem (`crates/ff-encoding/src/lib.rs`):
  `Encoding { name, code_page, family, display_name, aliases }`, `EncodingFamily`
  whose `SingleByte` doc explicitly lists EBCDIC, bidirectional
  `convert_to_utf8` / `convert_from_utf8`, `StreamDecoder` / `StreamEncoder`,
  detection. Its Cargo manifest depends only on `thiserror` (no heavy deps, no
  `ff-vfs` edge) -- `crates/ff-encoding/Cargo.toml`.
- The editor already tracks an `encoding_label()` on the tab (defaults "UTF-8",
  `tab_manager.rs` test `untitled_tab_encoding_label_is_utf8`).
- BUT the save path hardcodes `encoding: "utf-8".to_string()` when building
  `RecordAttrs` (`tab_manager.rs:1351`). `RecordAttrs.encoding` is already a
  `String` field on the ff-vfs store contract; it simply is not being populated
  from a real attribute. AX-ENCODING-FIRST-CLASS closes exactly this gap.

---

## 3. The finalized attribute model

### 3.1 One single, immutable per-open `FileAttributes` (AX-SAVE-VS-SAVEAS)

Because SAVE always targets the source environment and SAVE-AS is a separate
export-transform operation (AX-SAVE-VS-SAVEAS), the opened file needs only ONE
attribute value, describing the source/owning environment, fixed for the life of
the open. There is NO read/write split on the handle: a plain SAVE reuses these
attributes unchanged, and a cross-platform save-as builds its own destination
attributes locally inside the export operation (Section 6.2) without mutating
these.

```
// FINALIZED -- crates/ff-vfs/src/file_attributes.rs (NEW), ff-vfs-local plain data.

/// The authoritative attributes of an opened file, reported by the owning
/// (source) environment at OPEN and IMMUTABLE for the life of the open.
/// Replaces the split TabState.owning_environment + store_identity +
/// document-side record format being set by two separate shell methods.
pub struct FileAttributes {
    /// The owning environment the file was opened from and is SAVED back to
    /// ("NTFS" / "POSIX" / "MAINFRAME"). SAVE addresses this environment.
    pub owning_env: String,
    /// Dataset identity when the source is a dataset; None for a plain host file.
    pub store_target: Option<StoreTarget>,
    /// Record framing + LRECL + encoding as read. None == native/Delimited
    /// (host text -> byte save path). RecordAttrs already carries encoding.
    pub record: Option<RecordAttrs>,
    /// Advisory source facets.
    pub case_sensitive: bool,
    pub read_only: bool,
}
```

The destination side of a cross-platform save-as is NOT a field here. It is a
transient value the export-transform operation constructs for the chosen target:

```
// FINALIZED -- the DESTINATION descriptor for a SAVE-AS export, built transiently
// by the export operation, NOT carried on the open handle.
pub struct ExportTarget {
    /// Target environment to address ("NTFS" / "POSIX" / "MAINFRAME").
    pub target_env: String,
    /// Target dataset identity when exporting to a dataset; None for a host file.
    pub store_target: Option<StoreTarget>,
    /// Record framing + LRECL + encoding to TRANSFORM to. None == Delimited/byte.
    pub record: Option<RecordAttrs>,
}
```

Notes and rationale:
- `RecordAttrs` ALREADY carries `{ recfm: RecordFormatKind, lrecl: u32, encoding:
  String }` (`crates/ff-vfs/src/backend_environment.rs`). Reusing it for both the
  open `FileAttributes.record` and the transient `ExportTarget.record` means
  encoding is carried for free -- AX-ENCODING-FIRST-CLASS needs no new type, only
  real population instead of the `"utf-8"` literal.
- `StoreTarget { dsn, owning_env, catalog_id }` already exists and is reused
  verbatim; `ExportTarget.store_target` lets a save-as carry a NEW dataset identity
  distinct from the (untouched) source.
- `record: None` keeps meaning "native/Delimited -> byte path", preserving the
  existing selection signal (`record_attrs_for` in `tab_manager.rs`). No new
  `RecordFormatKind` variant, so that enum is NOT reshaped.
- Plain SAVE reads `FileAttributes` directly; nothing is mutated per save. SAVE-AS
  constructs an `ExportTarget`, runs the transform pipeline (Section 6.2), and
  leaves `FileAttributes` untouched (tab-adoption after save-as is a separate UI
  policy, open question 5).

### 3.2 Where it lives, object-safety, DAG

- Lives in `ff-vfs` as plain data (same home/style as `StoreTarget`/`RecordAttrs`).
  No methods needed for dispatch, so object-safety is not even in question for
  `FileAttributes`/`ExportTarget` themselves; the traits that CONSUME them
  (`BackendEnvironment`, `RecordSource`) stay object-safe (each already has an
  `_assert_object_safe` guard).
- Encoding stays a `String` label in ff-vfs (matching `RecordAttrs.encoding`). It
  is resolved to `ff_encoding::Encoding` at decode/encode time INSIDE the owning
  environment (ff-idcams for datasets, ff-ce-host-fs for host), NOT in ff-vfs. This
  preserves the ff-vfs dependency set (`ff-core` + `ff-logging` only,
  `crates/ff-vfs/Cargo.toml`) -- NO new DAG edge. (`ff-encoding` depends only on
  `thiserror`, so an ff-vfs -> ff-encoding edge would be acyclic IF ever wanted;
  the design deliberately does not take it, to keep ff-vfs a pure contract crate.)
- DAG preserved: ff-idcams/ff-dscatalog continue to depend on ff-vfs, never the
  reverse.

---

## 4. OPEN design (symmetric, addressed to the owning environment)

### 4.1 The additive record-aware LOAD entry (AX-OPEN-RECORDS)

Symmetric with the existing `save_records`. Added to `BackendEnvironment`
(`crates/ff-vfs/src/backend_environment.rs`), object-safe, with a provided default
so every host CE that does not override it is unchanged.

```
// FINALIZED -- additive method on trait BackendEnvironment (CR-CH-060 extended).

/// Open a resource for record-aware READ, returning its authoritative
/// attributes and a pull stream of framed records. Symmetric with save_records.
/// The owning environment decodes platform specifics (EBCDIC, RECFM framing,
/// host EOL) into the uniform record stream; the caller never sees raw bytes.
///
/// Provided default returns NotRecordCapable so host/byte CEs are unchanged
/// until they opt in.
fn open_records(&self, source: &StoreTarget)
    -> RecordLoadOutcome
{
    let _ = source;
    RecordLoadOutcome::NotRecordCapable
}
```

Supporting ff-vfs-local plain-data types (new, same file):

```
pub enum RecordLoadOutcome {
    /// Loaded: the attributes read from the source + a boxed record stream.
    Loaded {
        attrs: RecordAttrs,                 // recfm/lrecl/encoding as read
        records: Box<dyn RecordCursor>,     // object-safe pull/seek stream (below)
    },
    /// This environment is not record-capable (host byte CEs); caller byte-opens.
    NotRecordCapable,
    /// A real load error (mapped from DatasetError etc.).
    Failed { rc: i32, message: String },
}
```

Why a `RecordCursor` rather than reusing `RecordSource`: the existing
`RecordSource { fn next_record(&mut self) -> Option<&[u8]> }` is a forward-only
pull used by SAVE. OPEN windowing needs POSITIONING for backward scroll and far
jumps (Section 5). So OPEN returns a slightly richer object-safe cursor:

```
pub trait RecordCursor: Send {
    /// Total records if known up-front (Fixed/indexed); None if not yet known
    /// (streaming Variable/VSAM) -- see Section 5 on unknown length.
    fn known_len(&self) -> Option<u64>;
    /// Position to record k (0-based). For a sequential backend this may be a
    /// re-read from the start to k; for an indexed/Fixed backend it is O(1).
    fn seek_record(&mut self, k: u64) -> std::io::Result<()>;
    /// Next record's bytes (already decoded to the record's own bytes), or None
    /// at end of data.
    fn next_record(&mut self) -> std::io::Result<Option<Vec<u8>>>;
}

fn _assert_cursor_object_safe(_: &dyn RecordCursor) {}
```

Object-safety: `&self`/`&mut self` receivers only, no generics, no `Self` by
value -- `dyn RecordCursor` and `dyn BackendEnvironment` both remain usable
(guarded by compile-time `_assert_object_safe` references, matching the existing
pattern in the crate).

### 4.2 The symmetric open flow (finalized)

```
Files Panel selects a file (any catalog type)
  v
resolve via the file's CATALOG (Section 7): (owning_env, locator, read attrs)
  v
file.open addressed to owning_env's BackendEnvironment
  v
backend.open_records(&source_target):
    MAINFRAME CE -> DatasetAccess::resolve(dsn, Read) -> open(Read) -> a
       RecordCursor that calls get() and decodes via the RECFM codec; attrs from
       the DatasetHandle (RECFM/LRECL/DSORG) + encoding (cp037 default for EBCDIC)
    HOST CE (NTFS/POSIX) -> either NotRecordCapable (caller byte-opens, below) or
       a Delimited RecordCursor that frames host lines by the host EOL and reports
       encoding from detection (ff-encoding)
  v
RecordLoadOutcome::Loaded { attrs, records } OR NotRecordCapable
  v
TabManager builds the Document, sets RecordFormat from attrs, builds the single
  immutable FileAttributes { owning_env, store_target, record: Some(attrs),
  case_sensitive, read_only }, feeds `records` into the WindowBand.
  NotRecordCapable -> existing byte open via LocalFsProvider (Delimited), attrs
  derived host-side; behaviour-identical to today for plain host text.
```

This collapses the current two shell steps (`shell_open_file_with_env` +
`apply_pending_open_identity`) into ONE `apply_open_attributes`, and removes the
`FileOpenHandler` param-pushing of `owning_env/dsn/catalog/recfm/lrecl`
(`crates/ff-desktop/src/shell/handlers.rs`). The attributes now come FROM the
controller, satisfying AX-OPEN-SYMMETRY.

---

## 5. WINDOWING design (the centerpiece -- platform-agnostic record window)

### 5.1 The abstraction (recommended)

The windowing tool pages over the EXISTING `WindowBand` (record numbers), fed by a
`RecordCursor` that the owning environment produced in `open_records`. Concretely:

- **Navigation authority = `WindowBand::total_records()`** -- the scrollbar and
  `down N`/`up N` size on this, never on the resident band (Req 12.5, already
  enforced). This is platform-independent: a record count is a record count
  whether the records are EBCDIC FB/80 or host UTF-8 lines.
- **Resident band = 3 pages of records** (`WindowBand::resident_range()`), shifted
  with hysteresis on scroll (`scroll_to_record`) or in one shot on a far jump
  (`jump_to_record`). When the band shifts, the shell's byte/record cache asks the
  `RecordCursor` to `seek_record(new_band.start)` and pulls `next_record()` for the
  band span, evicting the non-pinned records that left the band
  (`WindowBand::evictable`, dirty records pinned).
- **The windowing layer never inspects platform.** It sees record numbers and a
  `RecordCursor`. The owning environment already decoded EBCDIC/RECFM/host-EOL
  INSIDE `open_records`, so the records in the band are uniform (the record's own
  bytes, to be rendered via the document's encoding attribute). This is exactly
  AX-WINDOW-PLATFORM-AGNOSTIC.

So the recommended uniform abstraction is: **(total_records authority) + (3-page
resident RecordRange) + (object-safe RecordCursor with seek_record/next_record/
known_len)**. `WindowBand` already provides the first two; `RecordCursor`
(Section 4.1) provides the third.

### 5.2 Honest treatment of what is hard

- **Sequential vs random access.** `DatasetAccess` is sequential for PS
  (`get` walks forward; `CatalogVfsProvider` advertises `random_access: false`).
  `seek_record(k)` on a purely sequential backend means "re-open and read forward
  to k". For FORWARD paging and far-forward jumps that is acceptable (one pass).
  For a Fixed-record dataset (FB/LRECL known) record k starts at `k*lrecl`, so a
  Fixed backend CAN implement O(1) `seek_record` by seeking the physical offset --
  recommended as the first optimization. Variable (VB/RDW) and VSAM are not O(1)
  by record number.
- **Backward scrolling.** With a forward-only stream, scrolling UP past the band
  requires re-seeking to an earlier record. For Fixed records this is O(1). For
  Variable/sequential it requires either (a) a record-offset index built on first
  pass (store each record's physical offset as records are first read -- this is
  what `SparseLineIndex`/`sparse_line_index.rs` already does for lines and what
  `Document::record_start` provides for the resident buffer), or (b) re-read from
  start. RECOMMENDATION: build an incremental record-offset index (sparse, one
  entry per N records) during forward reads, so backward jumps seek to the nearest
  indexed anchor then read forward a bounded amount. This mirrors the existing
  sparse line index strategy and keeps memory O(total_records / N).
- **Record count not known upfront.** For Variable/VSAM the total is not known
  until EOF. `RecordCursor::known_len()` returns `None` in that case.
  `WindowBand::set_total_records` is DESIGNED for exactly this: start with an
  ESTIMATE (e.g. file_size / average_lrecl, or "unknown -> large sentinel"), and
  refine via `set_total_records` when a background full scan (or incremental index
  build) learns the true count. The band and the authority are already decoupled
  (Req 12.5), so a refined total does not disturb the resident band. The scrollbar
  shows an estimate that firms up -- a standard large-file behaviour.
- **VSAM keyed access.** KSDS is keyed, not record-numbered;
  `DatasetAccess::point(&Positioner)` establishes a key/RRN position, then
  `get`/`put` operate there (`trait_def.rs`). A record-NUMBER window is a poor fit
  for a keyed cluster. RECOMMENDATION: for the FFEDIT browse/edit window, present
  KSDS/RRDS in **physical/entry sequence** (ESDS-like) via a sequential cursor over
  the cluster, and treat keyed navigation (find-by-key) as a SEPARATE positioning
  command that calls `point` then recentres the window via `jump_to_record` on the
  resulting record position. Full keyed random editing in a windowed view is the
  genuinely hard case; it is OUT of the first windowing scope and flagged as an
  open risk (Section 11). Sequential browse of a VSAM cluster is in scope.

### 5.3 What must change vs what already works

- Already works: `WindowBand`, `RecordRange`, `Document` record model, the
  separation of total vs band, hysteresis, far-jump, dirty pinning.
- Must be built (build phase): the `RecordCursor` feed from `open_records`; the
  band's load/evict calling `seek_record`/`next_record` instead of the whole-file
  `StreamingFileReader::load`; the incremental record-offset index for
  Variable/VSAM (and large host files) backward scroll; the Fixed O(1)
  `seek_record` fast path. Per AX-ALWAYS-WINDOWED (owner decision 4) this is NOT a
  deferred dataset-only add-on: it is the editor's FOUNDATIONAL data path for every
  file, built early, co-evolving with the large-file subsystem rather than waiting
  on it (Section 10, step 6).

---

## 6. SAVE / SAVE-AS design

### 6.1 Plain SAVE (target == source) -- unchanged seam

`host_fs_save` -> `save_active_tab_via_backend` stays the single SAVE seam
(`crates/ff-desktop/src/shell/dispatch_ffedit.rs`, `tab_manager.rs`). It reads the
single immutable `FileAttributes` (no write facet -- SAVE targets the source):
- address `FileAttributes.owning_env`'s backend (via the registry / Section 7
  resolution),
- if `FileAttributes.record.is_some()` AND `backend.record_capable()` -> build
  `StoreTarget` from `FileAttributes.store_target`, build a `RecordSource` over the
  document's framed records, call `save_records(target, source, record.encoding...)`,
- else -> byte `save(path, bytes)` (byte-identical host save, Delimited).

The ONLY change vs today: the record/env/target come from the single
`FileAttributes` value instead of from `owning_environment` + `store_identity` +
document record format read separately, and `RecordAttrs.encoding` is
`FileAttributes.record.encoding` instead of the hardcoded `"utf-8"`
(AX-ENCODING-FIRST-CLASS, `tab_manager.rs:1351`). Nothing on the handle is mutated
by a SAVE.

### 6.2 SAVE-AS as EXPORT + TRANSFORM (AX-SAVE-VS-SAVEAS)

SAVE-AS is a DISTINCT operation from SAVE, modelled as an EXPORT + TRANSFORM
pipeline. It does NOT mutate the open file's `FileAttributes`; it reads the
in-memory records, transforms them to a chosen destination format, and writes them
to the destination under a NEW identity. Flow when the user saves an open NTFS
file AS a mainframe dataset (or vice versa):

1. User invokes SAVE-AS and selects the destination (a target catalog + a new DSN
   for a mainframe target, or a host path for a host target). The destination
   selection identifies the TARGET ENVIRONMENT.
2. If the target is a MAINFRAME dataset and the source was host (so no RECFM/LRECL/
   encoding exists yet), the UI MUST collect the destination record attributes:
   RECFM (F/FB/V/VB), LRECL, and encoding (default cp037 for EBCDIC). This is the
   one genuinely new UI surface -- see the open risk in Section 11. For a host
   target, destination attributes are "Delimited + chosen encoding + host EOL".
3. Build a TRANSIENT `ExportTarget { target_env, store_target: Some(new
   StoreTarget{dsn, owning_env=target_env, catalog_id}), record: Some(RecordAttrs{
   recfm, lrecl, encoding}) }` LOCAL to the export operation. The open document's
   `FileAttributes` is NOT touched.
4. Run the EXPORT-TRANSFORM pipeline:
   - SOURCE: the in-memory records/lines (already decoded to the document's record
     model).
   - TRANSFORM: re-frame to the target RECFM/LRECL and RE-ENCODE to the target
     encoding (host-utf8 lines -> EBCDIC FB/80 via the RECFM codec + `ff_encoding::
     convert_from_utf8`; the converse frames each dataset record as a Delimited
     line and re-encodes to the chosen host encoding).
   - SINK: address `ExportTarget.target_env`'s backend and store -- records via
     `save_records(store_target, transformed_source, record)` when
     `record.is_some()` AND `backend.record_capable()`, else byte `save`. This
     reuses the SAME attribute-driven backend selection as SAVE (Section 6.1), so
     no platform branch is introduced; the transform is the only addition.
5. After a successful save-as the tab may ADOPT the target as its new source
   (re-open semantics: a fresh OPEN of the new destination yields a new immutable
   `FileAttributes`), so subsequent plain SAVEs target the dataset. This is a UI
   policy choice (open question 5), not a contract change, and it is the only way
   the open handle's attributes ever change -- by re-opening, never by mutation.

Byte-identical host SAVE is preserved: a host-source, host-target, Delimited
export (and every plain SAVE) still takes the byte `save` path with its encoding;
nothing forces a round-trip through records.

`StoreTarget` is NOT widened for save-as -- `ExportTarget.store_target` is just
another `StoreTarget` value with the destination's dsn/catalog, constructed
transiently. Section 9 confirms no core widening is required.

Note (forward-fit): this export-transform pipeline (source records -> reframe/
re-encode -> sink to a target environment) is conceptually the SAME machinery a
JES utility such as IEBGENER or an IDCAMS REPRO-style copy needs. Designing
SAVE-AS as an explicit transform between environments pre-seeds that batch
capability rather than hiding the conversion inside the save path.

---

## 7. CATALOG design (universal resolution; the leak dissolves)

### 7.1 Target state (AX-UNIVERSAL-CATALOG)

Every file resolves through its catalog. Define a single resolution step the shell
uses on open AND on save-as target selection:

```
resolve_in_catalog(catalog_name, file_key)
  -> (owning_env, locator, FileAttributes-or-host-defaults)
```

- Mainframe catalog -> `owning_env = "MAINFRAME"`, locator via the existing
  volume-indirected `resolve_locator` (`catalog_resolve.rs`), attrs = RECFM/LRECL/
  DSORG/encoding from the `DatasetRecord`. Record access via `DatasetAccess`.
- Native/POSIX catalog -> `owning_env = "NTFS"`/`"POSIX"`, locator = the host path
  under the catalog's `path`/`mount_point`, attrs = Delimited + detected encoding +
  host EOL. Record access via a Delimited `RecordCursor` over `LocalFsProvider`.

With this, the save seam addresses `FileAttributes.owning_env` (and SAVE-AS
addresses `ExportTarget.target_env`) and the target backend resolves its own
locator FROM the catalog identity carried on `StoreTarget`
(`catalog_id`). The MAINFRAME backend resolves catalog_name -> repository path at
call time from the `StoreTarget.catalog_id` it already receives (the proposal's
C.4 Option 1), so the registry's single `backend_for("MAINFRAME")` serves ALL
catalogs and `WorkbenchShell::mainframe_backend_for_active_tab`
(`crates/ff-desktop/src/shell/dispatch_ffedit.rs`, confirmed present, peeks
`store_identity` and `files_panel.registry.get_by_name`) is DELETED. The shell
save seam becomes platform-agnostic -- the leak dissolves because catalog
resolution, not a shell-side mainframe peek, supplies the backend's target.

### 7.2 Near-term reality and the bridge

Today: catalog resolution is universal in the registry MODEL but the record-access
path is mainframe-only; Native/POSIX catalogs resolve to a host directory browsed
via `LocalFsProvider` and have no `DatasetAccess`-style record contract (they do
not need one -- a host line is a Delimited record). So the near-term workable path
is:

- Keep the proposal's C.4 Option 1 as the concrete first move: make
  `MainframeEnvironment` catalog-name aware from `StoreTarget.catalog_id` and
  delete `mainframe_backend_for_active_tab`. This removes the leak WITHOUT needing
  the full universal record abstraction on host catalogs.
- For host files, the "catalog" is the Native/POSIX catalog that already resolves a
  path; `open_records` on the host CE can start as `NotRecordCapable` (byte open,
  Delimited framing done document-side) and later gain a real Delimited
  `RecordCursor` when host windowing is wired. Either way the owning_env + locator
  came from the catalog, so the resolution is already universal in shape.

RECOMMENDATION: treat "every file resolves through its catalog" as the design
target (Section 1 AX-UNIVERSAL-CATALOG) and implement the mainframe
catalog-name-aware resolution first (removes the leak now); the host-catalog
record cursor is additive and lands with host windowing. No core-type change is
needed for either move.

---

## 8. JES forward-fit

Unchanged and confirmed. The shared record contract `ff_dscatalog::DatasetAccess`
(`crates/ff-dscatalog/src/dataset_access/trait_def.rs`) is explicitly designated
for "the JCL executor, the editor's future MAINFRAME BackendEnvironment SAVE path,
and IDCAMS" (Req 34.6). Method set (object-safe): `allocate`, `resolve`, `open`,
`get`, `put`, `point`, `close`, `dispose`.

The finalized editor interface consumes a THIN adapter over this SAME contract on
BOTH ends:
- OPEN: the MAINFRAME CE's `open_records` drives `resolve(dsn, Read)` -> `open` ->
  `get`-loop into a `RecordCursor` (new, Section 4) -- the read symmetry the
  proposal left optional.
- SAVE: the MAINFRAME CE's `save_records` already drives `resolve` -> `open` ->
  `put` -> `close` (`crates/ff-idcams/src/mainframe_env.rs`).

What JES ADDS and why it fits (per proposal Section D, re-confirmed):
- DD allocation + DISP + concatenation + GDG live in `ff-dsalloc` and produce a
  `DatasetHandle` / `DdRequest` consumed by `DatasetAccess::allocate` -- a BATCH
  pre-step the editor does not need; it fits as an additional producer of the same
  handle the editor's `resolve` produces. No shared-contract change.
- Disposition (`dispose(handle, KEEP/CATLG/DELETE/PASS/UNCATLG)`) is a batch-only
  method the editor never calls; it is already on the trait.
- SYSIN/SYSOUT spool uses the `VfsProvider` seam (`vfs://local/.ffwb/spool/...`,
  jes-emulator design), not `DatasetAccess`; unchanged.
- `ff-jes` today is job-management only (queue/scheduler/SDSF/FFJCL parse); the
  executor + record I/O are DEFERRED (ROADMAP PLUGIN phase 6). So this remains a
  DESIGN alignment, not a wireable integration now -- consistent with
  AX-DESIGN-NOW-BUILD-LATER.

The editor consumes a strict SUBSET of what JES needs (no DD/DISP/GDG/dispose), so
designing the editor interface as a thin `DatasetAccess` adapter plus the uniform
`RecordCursor` does not constrain JES; it pre-aligns with it. The `RecordCursor`
is editor-facing (windowing); the executor reads records via `get` directly and
does not need a window, so `RecordCursor` is additive and JES-neutral.

---

## 9. Load-bearing / DAG impact

Per `framework-conformance.md`, every core-type touch is listed. "BackendEnvironment
is a load-bearing core type"; the owner has EXPLICITLY AUTHORIZED the additive
record-aware reshape (CR-CH-060 extended).

1. **`BackendEnvironment::open_records` (NEW, additive, OWNER-AUTHORIZED).**
   Object-safe, provided default `NotRecordCapable` so host CEs are unchanged.
   Symmetric with `save_records`. This is the one authorized reshape of the store
   contract (AX-OPEN-RECORDS). `crates/ff-vfs/src/backend_environment.rs`.
2. **New ff-vfs-local plain-data types:** `FileAttributes` (single, immutable
   per-open), the transient `ExportTarget` (save-as destination, not carried on the
   handle), `RecordLoadOutcome`, and the `RecordCursor` trait. Additive, plain
   data / object-safe trait; no existing type reshaped. `RecordFormatKind`
   is NOT changed (no `Delimited` variant added; `record: None` keeps meaning
   native/byte).
3. **`StoreTarget` is NOT widened.** Save-as reuses `StoreTarget` with the
   destination dsn/catalog; catalog resolution (Section 7, Option 1) means the
   MAINFRAME backend resolves catalog_name -> path from `StoreTarget.catalog_id`,
   so no resolved-path field is added. (If, at build time, Option 1's name->path
   resolution proves unavailable at save, the proposal's Option 2 would widen
   `StoreTarget` -- that WOULD be a core-adjacent change needing confirmation; the
   design chooses Option 1 to avoid it.)
4. **NO DAG change.** ff-vfs keeps its `ff-core` + `ff-logging` dependency set.
   Encoding stays a `String` label in ff-vfs, resolved to `ff_encoding::Encoding`
   inside the owning environments (ff-idcams/ff-ce-host-fs), which already may
   depend on ff-encoding. No ff-vfs -> ff-dscatalog and no ff-vfs -> ff-encoding
   edge is introduced.
5. **Shell-internal refactors (no core-type, framework-conformant):** collapse
   `PendingOpenReq` + the two apply steps into one `apply_open_attributes`; unify
   `TabState.owning_environment` + `store_identity` into one `attributes:
   FileAttributes`; delete `mainframe_backend_for_active_tab`. These touch
   `shell/{handlers.rs,state.rs,actions.rs,update.rs,dispatch_ffedit.rs}`,
   `tab_state.rs`, `tab_manager.rs` -- no trait, no persistence format, no dispatch
   path change.

---

## 10. Implementation ordering for WHEN BUILD RESUMES (deferred)

Dependency-ordered; each step leaves a green build. Steps explicitly gated on
core subsystems still under development are marked [CORE-GATED].

1. **Add the attribute model to ff-vfs** (`FileAttributes`, single immutable;
   plain data). Nothing consumes it yet. Green trivially.
2. **Collapse the open orchestration (shell-only).** Replace `PendingOpenReq`
   quad-of-options + two-step apply with one `apply_open_attributes` building the
   single `FileAttributes`. Keep "no attrs -> host default" so every existing host
   open is byte-identical. Re-run full-shell open/save tests
   (`shell/tests_command.rs`, `tests_crch061.rs`).
3. **Flow real encoding into SAVE.** Populate `RecordAttrs.encoding` from
   `FileAttributes.record.encoding` instead of the `"utf-8"` literal
   (`tab_manager.rs`); resolve via `ff_encoding` inside the owning env. Round-trip
   tests for host (utf-8 unchanged) and a cp037 dataset.
4. **Remove the save-seam leak (catalog-name-aware mainframe backend,
   Section 7/C.4 Option 1).** Make `MainframeEnvironment` resolve catalog_id ->
   repo path at call time; delete `mainframe_backend_for_active_tab`. Re-run
   CR-CH-061 catalog save tests.
5. **Add `open_records` + `RecordLoadOutcome` + `RecordCursor` to ff-vfs**
   (owner-authorized, additive, provided default). Implement it FIRST on the HOST
   CE as a Delimited `RecordCursor` (a host line = a Delimited record) AND on the
   MAINFRAME CE over `DatasetAccess::resolve/open/get`. Per AX-ALWAYS-WINDOWED the
   host cursor is NOT deferred -- it is foundational, because every file is opened
   windowed.
6. **Wire the record window as the ONE AND ONLY browse/edit path (AX-ALWAYS-
   WINDOWED).** Feed the EXISTING `WindowBand` from a `RecordCursor` for ALL files
   (host text AND datasets); REPLACE the whole-file `StreamingFileReader::load`
   path entirely with band load/evict via `seek_record`/`next_record`. Add the
   Fixed O(1) `seek_record` fast path and the incremental record-offset index for
   Variable/VSAM (and large host files) backward scroll. Unknown record count ->
   estimated total refined by a background scan (`WindowBand::set_total_records`),
   per owner decision 3. This is the editor's foundational data path, built early,
   not a late dataset-only add-on.
7. **Implement SAVE-AS as the export-transform pipeline.** Target selection +
   destination attribute collection via the reused allocation dialog + encoding
   picker (owner decision 1), build a transient `ExportTarget`, run the
   reframe/re-encode transform, then store via the existing attribute-driven
   backend selection. Open `FileAttributes` is never mutated; on success the tab
   re-opens the destination so it adopts the target as new source (owner
   decision 5). Tests: host->dataset and dataset->host round-trips.
8. **[CORE-GATED on JES executor] JES consumption.** No editor change; the executor
   consumes the SAME `DatasetAccess` + `ff-dsalloc`. Verify the editor adapter did
   not constrain the batch path. Depends on the deferred JobExecutor.

Steps 1-7 deliver the owner's stated interface (platform-agnostic FFEDIT, always
windowed browse/edit for every file, symmetric open addressed to the owning env,
cross-platform save-as, encoding first-class, leak removed) using only additive/
owner-authorized changes. Step 8 is gated on the JES executor still under
development, consistent with AX-DESIGN-NOW-BUILD-LATER. NOTE: because windowing is
now foundational (AX-ALWAYS-WINDOWED), the window wiring (step 6) is a FIRST-CLASS
early step, not a core-gated late one -- it co-evolves with the large-file
subsystem rather than waiting on it.

---

## 11. Open questions -- ALL RESOLVED (owner decisions)

The five questions are settled. Recorded here as decisions of record.

1. **Save-as destination-attribute UI. RESOLVED: reuse the dataset-allocation
   dialog's RECFM/LRECL fields plus an encoding picker** (`ff-dataset-alloc-dialog`).
   No dedicated new dialog. Collected at SAVE-AS time for a host->dataset export.
2. **VSAM keyed windowing. RESOLVED: sequential (entry-order) browse of a VSAM
   cluster is IN scope; full keyed random editing in a windowed view is OUT of
   first scope.** Keyed navigation (find-by-key via `DatasetAccess::point`) that
   recentres the sequential window is allowed; keyed random edit is a later,
   separately-scoped capability.
3. **Unknown record count UX. RESOLVED: an estimated scrollbar that refines is
   acceptable.** The total starts as an estimate and firms up via a background
   scan (`WindowBand::set_total_records`); no full pre-scan on open is required
   (which would defeat windowing for huge files).
4. **Host windowing scope. RESOLVED (STRONGER than proposed): the editor is ALWAYS
   windowed for EVERY file from the beginning** (AX-ALWAYS-WINDOWED). There is no
   non-windowed path; the whole-file `StreamingFileReader::load` is replaced by
   band-driven loading for host text as well as datasets. The host `RecordCursor`
   is foundational and early (ordering step 5/6), NOT a deferred follow-on.
5. **Save-as adopting the target as new source. RESOLVED: yes, adopt the target
   via re-open semantics.** After a host->dataset save-as, the tab re-opens the
   destination, yielding a fresh immutable `FileAttributes`, so subsequent plain
   SAVEs target the dataset.

---

## Appendix: evidence index (files/symbols read for the finalized delta)

- Catalog universality: `crates/ff-catalog-registry/src/lib.rs`
  (`CatalogType::{Mainframe,Posix,Native}`, `VirtualCatalog`, `section_label`,
  `open_dscatalog` mainframe filter, `allocate`/`list_datasets`/`resolve_dsn`);
  `crates/ff-dscatalog/src/vfs_provider.rs` (`CatalogVfsProvider`, scheme
  "catalog", `capabilities` random_access:false, `read` whole-file decode,
  `decode_to_lines`/`encode_from_lines`); `crates/ff-dscatalog/src/catalog_resolve.rs`
  (`resolve_locator` volume indirection, `resolve_uncataloged`);
  `docs/specs/file-tree-panel/{requirements.md,tasks.md}` (three catalog sections,
  Req 14/15/17/18, Task 26.10 generic CatalogRoot).
- Windowing: `crates/ff-document-model/src/window_band.rs` (`WindowBand`,
  `RecordRange`, `total_records` vs band, `scroll_to_record`, `jump_to_record`,
  `evictable`); `crates/ff-document-model/src/document_records.rs` (`total_records`,
  `record_start`, `record_byte_length`, `save_image`, `rebaseline`);
  `crates/ff-document-model/src/streaming.rs` (`StreamingFileReader::load`
  whole-file byte chunks); `crates/ff-document-model/src/viewport.rs` (`Viewport`
  line-based scroll); `crates/ff-document-model/src/lib.rs` (`RecordFormat`,
  `RecordNumber`, `WindowBand`/`RecordRange` re-exports);
  `crates/ff-large-file-performance/src/lib.rs` (render/layout caches, not data
  windowing).
- Record contract / forward-fit: `crates/ff-dscatalog/src/dataset_access/trait_def.rs`
  (`DatasetAccess` full method set, object-safety guard).
- Attributes / save seam / leak / encoding: `crates/ff-vfs/src/backend_environment.rs`
  (`BackendEnvironment`, `StoreTarget`, `RecordAttrs`, `RecordSource`,
  `RecordFormatKind`, `save_records` provided default) -- via proposal + cited
  signatures; `crates/ff-desktop/src/shell/dispatch_ffedit.rs`
  (`mainframe_backend_for_active_tab` peeks `store_identity` +
  `files_panel.registry.get_by_name`); `crates/ff-desktop/src/tab_manager.rs:1351`
  (`encoding: "utf-8"` hardcode in `save_active_tab_via_backend`;
  `encoding_label` default UTF-8).
- Encoding infrastructure: `crates/ff-encoding/src/lib.rs`,
  `crates/ff-encoding/src/encoding.rs` (`Encoding`, `EncodingFamily` with EBCDIC in
  SingleByte, `convert_to_utf8`/`convert_from_utf8`, `StreamDecoder`/`StreamEncoder`);
  `crates/ff-encoding/Cargo.toml` (thiserror-only, no ff-vfs edge);
  `crates/ff-vfs/Cargo.toml` (ff-core + ff-logging only -- DAG authority).
- Prior context: `.agents/tasks/ffedit-file-interface-design/design-proposal.md`
  (as-built analysis, two seams, open/save paths, C.4, JES section D, migration).
- Steering honored: `.kiro/steering/{framework-conformance.md (BackendEnvironment
  load-bearing, CR-CH-060 authorized record-aware reshape), wiring-standard.md,
  rust-standards.md (400-line, object-safety), documentation-charset.md (ASCII)}`.

Content was rephrased for compliance with licensing restrictions where external
mainframe terminology is summarised.
