# FFEDIT File-Interface Design Proposal

Re-evaluating how FFEDIT (the editor Command Environment) works with files of
different internal mechanisms (host NTFS/POSIX, mainframe datasets, VSAM), aimed
at a simpler-but-complete interface, with forward-consideration of the JES
emulator.

STATUS: DESIGN PROPOSAL FOR HUMAN REVIEW. No code changes made. Every
architectural claim is cited to a file/trait/function actually read. "Implemented
today" and "Proposed" are kept strictly separate.

Plain ASCII only (per `documentation-charset.md`).

---

## Summary answer (read this first)

**The user's hypothesis is correct and largely already latent in the codebase --
it just is not finished.** FFEDIT should NOT be platform-aware, and in the SAVE
direction it already is not: the save path resolves the owning environment by a
tab ATTRIBUTE (`TabState.owning_environment`) and dispatches through one seam, so
the editor never branches on "mainframe vs POSIX". The destination identity
already travels WITH the opened file, exactly as the user proposes.

Three things, however, fall short of the user's clean picture:

1. **The OPEN direction is NOT symmetric with SAVE.** Mainframe datasets are
   opened by materialising them to a HOST PATH and reading raw host bytes through
   `ff-connector-local-fs::LocalFsProvider` -- the record-aware `DatasetAccess`
   read path (`open`/`get` over a RECFM codec) that ALREADY EXISTS is bypassed on
   open. So the attributes (RECFM/LRECL/DSN/catalog) are pushed in from OUTSIDE
   the editor as command parameters, rather than returned BY the controller that
   owns the file.
2. **There IS residual platform special-casing in the shell save code**, but not
   inside FFEDIT's verb body -- it is in `WorkbenchShell::mainframe_backend_for_active_tab`
   (`shell/dispatch_ffedit.rs`), which peeks at `store_identity` to build a
   per-save MAINFRAME backend. This is the one real "leak" the user wants removed.
3. **The attribute set is split across two parallel channels** (`owning_environment`
   + `store_identity` on `TabState`, and `RecordFormat` on the `Document`), set by
   TWO separate shell steps (`shell_open_file_with_env` and
   `apply_pending_open_identity`). A single "file attributes" struct returned by
   the controller at open would collapse both.

**The good news for the JES forward-look:** the shared contract the user is
reaching for ALREADY EXISTS and is ALREADY designed to serve both the editor and
the JES executor. It is `ff_dscatalog::DatasetAccess` (CR-CH-059, dataset-catalog
Req 34), whose own doc-comment says: "the JCL executor, the editor's future
MAINFRAME BackendEnvironment SAVE path, and IDCAMS all perform record-level I/O
through it" (`crates/ff-dscatalog/src/dataset_access/trait_def.rs`). The JES
design delta (CR-CH-059, `docs/specs/jes-emulator/design.md`) independently
commits the future JCL executor to depend ONLY on `ff-dsalloc` (DD/DISP ->
`DatasetHandle`) and `DatasetAccess` (record I/O). So the editor and JES are
ALREADY aimed at one dataset/record contract; this proposal mostly asks that the
editor OPEN path be routed through it too, and that the save-side backend
selection stop peeking at tab internals.

**Recommended minimal shape:** keep the two load-bearing ff-vfs seams
(`VfsProvider` async scheme-routed access, and `BackendEnvironment` sync store
contract) exactly as they are -- do NOT reshape them (that is owner-confirmation
territory, framework-conformance.md). Instead introduce ONE small ff-vfs-local
plain-data struct, `FileAttributes`, and change the shell OPEN orchestration so
the file controller RETURNS `(FileAttributes, windowed content)` as one unit
instead of the shell pushing four command params in through `file.open`. Move the
per-save backend selection off `store_identity`-peeking and onto the
`owning_environment` name + the attributes carried on the handle. This removes the
leak, unifies the attribute channel, and leaves both ff-vfs contracts and the
crate DAG untouched.

---

## A. Current mechanism (as-built)

### A.1 The two ff-vfs seams (verified)

FFWB has TWO distinct storage seams in `crates/ff-vfs/`, confirmed by reading the
crate:

**Seam 1 -- `VfsProvider` (async, scheme-routed): OPEN / READ / LIST / WATCH /
SEARCH.** `crates/ff-vfs/src/provider.rs`. Object-safe `#[async_trait]` trait.
Methods: `scheme`, `capabilities`, `open` (-> `Box<dyn VfsFile>`), `read`,
`read_stream`, `write`, `create`, `delete`, `rename`, `list`, `stat`, `exists`,
`watch` (default Unsupported), `search` (default Unsupported). Open handles are
`VfsFile` (`read`/`write`/`flush`/`sync_all`/`close`). Providers register in
`ProviderRegistry` (`crates/ff-vfs/src/registry.rs`) keyed by `scheme()`. Supporting
types in `crates/ff-vfs/src/types.rs` (`OpenOptions`, `VfsEntry`, `VfsMetadata`,
`VfsCapabilities`, `WriteMode`, ...).

**Seam 2 -- `BackendEnvironment` (sync, name-addressed): the SAVE "store
contract".** `crates/ff-vfs/src/backend_environment.rs` (CR-CH-053 Task 21 /
CR-CH-060). Object-safe `trait BackendEnvironment: Send + Sync`. Methods:
- `name(&self) -> &str` -- the stable addressing name ("NTFS"/"POSIX"/"HOSTFS"/"MAINFRAME").
- `is_case_sensitive(&self) -> bool`.
- `save(&self, path: &Path, bytes: &[u8]) -> io::Result<()>` -- the RETAINED byte
  store (host CEs; native SAVE byte-identical).
- `save_records(&self, target: &StoreTarget, records: &mut dyn RecordSource, attrs: &RecordAttrs) -> RecordStoreOutcome`
  -- the ADDITIVE record-aware store (CR-CH-060 Shape 2); provided default returns
  `NotRecordCapable`.
- `record_capable(&self) -> bool` -- provided default `false`.

Supporting ff-vfs-local plain-data types (all in the same file):
- `StoreTarget { dsn: String, owning_env: String, catalog_id: Option<String> }`
  -- dataset identity, NOT a host path.
- `RecordFormatKind { Fixed, Variable, Undefined }` (`#[non_exhaustive]`).
- `RecordAttrs { recfm: RecordFormatKind, lrecl: u32, encoding: String }`.
- `RecordSource { fn next_record(&mut self) -> Option<&[u8]> }` -- object-safe
  pull stream over the editor's framed records.
- `RecordStoreOutcome { Stored { rc: i32 }, NotRecordCapable }`.
- `BackendOutcome { Handled { rc }, NotHandled }` (sibling, used by addressing).

Deliberate DAG note (verified in `crates/ff-vfs/Cargo.toml`): ff-vfs depends ONLY
on `ff-core` + `ff-logging` (+ tokio/async-trait/thiserror). It does NOT depend on
`ff-dscatalog` or `ff-volume`. This is WHY `StoreTarget`/`RecordAttrs`/
`RecordFormatKind` are ff-vfs-local plain data rather than reusing
`ff_dscatalog::Recfm` etc. The acyclic DAG is real and must be preserved.

### A.2 The OPEN path (as-built)

```
Files Panel (mainframe dataset) OR menu/command (host file)
  |
  |  resolve_dataset_path(repo, dsn)  [ff-files-panel/src/resolve.rs]
  |    maps DSN qualifiers -> host subpath; creates the file if absent
  v
file.open command  [FileOpenHandler, shell/handlers.rs]
  params: path (ALWAYS a host path), owning_env?, dsn?, catalog?, recfm?, lrecl?
  -> stores PendingOpenReq { path, owning_env, identity, recfm_lrecl }
     [shell/state.rs]
  v
WorkbenchShell::update() drains pending_open  [shell/update.rs]
  -> shell_open_file_with_env(path, owning_env)   [shell/actions.rs]
  |     -> TabManager::open_file(path)            [tab_manager.rs]
  |          reads bytes via LocalFsProvider::with_defaults().read(path)
  |          builds Document (new_document + insert bytes)
  |          TabState::for_file(...) -> owning_environment = "HOSTFS" (default)
  |     -> if owning_env.is_some(): active_tab.owning_environment = env
  -> apply_pending_open_identity(identity, recfm_lrecl)  [shell/actions.rs]
        if identity: active_tab.store_identity = Some(StoreIdentity { dsn, catalog })
        if recfm_lrecl: document.set_record_format(Fixed/Variable { ... })
```

Key as-built facts (cited):
- The editor ALWAYS reads raw host bytes, even for a mainframe dataset. The
  dataset is pre-materialised to a host file by `resolve_dataset_path`
  (`crates/ff-files-panel/src/resolve.rs`, mapping `PAYROLL.EMPLOYEE` ->
  `<repo>/PAYROLL/EMPLOYEE`), and `TabManager::open_file` reads it with
  `ff_connector_local_fs::LocalFsProvider` (`crates/ff-desktop/src/tab_manager.rs`,
  `open_file`). NOTE this bypasses the `Vfs`/`ProviderRegistry` facade AND the
  record-aware `DatasetAccess::open`/`get` read path.
- The attributes are PUSHED INTO the editor from outside as `file.open` command
  params (`owning_env`, `dsn`, `catalog`, `recfm`, `lrecl` -- parsed in
  `FileOpenHandler::execute`, `shell/handlers.rs`). The editor does not learn them
  from the controller that owns the file.
- The record read path that WOULD return framed records + attributes already
  exists but is unused on open: `CatalogDatasetAccess::open_impl` reads bytes via
  the `StorageProvider` seam and decodes to records via the RECFM codec
  (`crates/ff-dscatalog/src/dataset_access/impl_io.rs`, `codec_for` ->
  Fixed/Variable/Binary).

### A.3 The SAVE path (as-built)

```
Ctrl+S or typed "SAVE"  [shell/update.rs -> handle_command("SAVE")]
  v
single command front door -> active environment claim
  FFEDIT claims SAVE and ADDRESSes it to the owning backend environment
  [shell/commands_environment.rs: HostFsPlaceholder arm AND MainframePlaceholder
   arm both claim ONLY "SAVE"]
  v
WorkbenchShell::host_fs_save()  [shell/dispatch_ffedit.rs]   <-- THE SAVE SEAM
  |  1. if !is_modified: no-op (dirty-aware contract, Req 10.1)
  |  2. per_save_backend = self.mainframe_backend_for_active_tab()   <-- THE LEAK
  |       reads active_tab.store_identity; if Some, resolves catalog path from
  |       files_panel.registry and builds a per-save MAINFRAME backend
  |       [shell/mainframe_backend.rs: build_mainframe_backend_for_repo]
  |  3. backend = per_save_backend OR environments.backend_for(owning_environment)
  |       [shell/environment_registry.rs: backend_for -> MAINFRAME CE or host_fs]
  v
TabManager::save_active_tab_via_backend(backend)  [tab_manager.rs]
  |  reads Document::record_format() (OPEN-supplied; save does NOT re-derive)
  |  record_attrs_for(format) -> Some((kind,lrecl)) for Fixed/Variable, None for Delimited
  |  use_record_path = mapping.is_some() && backend.record_capable()
  |  if record path:
  |     build StoreTarget from store_identity (real DSN+catalog) OR host-path placeholder
  |     build RecordImageSource over the document's F1 framed records
  |     backend.save_records(&target, &mut source, &attrs) -> rc
  |  else (BYTE path, Delimited/host or non-record-capable backend):
  |     backend.save(path, bytes)   <-- byte-identical native save
  v
MAINFRAME CE: MainframeEnvironment::save_records  [ff-idcams/src/mainframe_env.rs]
  access.resolve(dsn, Write) -> open(Write) -> put(each record) -> close
  over Arc<dyn ff_dscatalog::DatasetAccess>; maps DatasetError -> rc (x37=37, etc.)
```

Key as-built facts (cited):
- FFEDIT's SAVE verb body does NOT branch on platform. It addresses SAVE to "the
  owning environment", and the backend is resolved by
  `EnvironmentRegistry::backend_for(owning_environment_name)`
  (`shell/environment_registry.rs`). MAINFRAME -> the record-capable
  `ff_idcams::MainframeEnvironment`; HOSTFS/unknown -> the host backend
  (`ff-ce-host-fs` decider -> NTFS/POSIX). This half already matches the user's
  vision.
- The record-vs-byte SELECTION is driven by the DOCUMENT'S `RecordFormat` (set at
  open) plus `backend.record_capable()` -- NOT by any platform check
  (`tab_manager.rs`, `save_active_tab_via_backend`).
- `MainframeEnvironment::save` (the byte entry) deliberately errors ("MAINFRAME is
  record-only; use save_records"), so a mainframe dataset can only be stored via
  the record path (`ff-idcams/src/mainframe_env.rs`).

### A.4 WHERE platform/environment knowledge currently lives (the crux)

Enumerated precisely, from most to least defensible:

1. **DEFENSIBLE (attribute-driven, already clean):**
   `TabState.owning_environment: String` and `TabState.store_identity:
   Option<StoreIdentity>` (`crates/ff-desktop/src/tab_state.rs`). The destination
   identity already travels with the opened file as an attribute. SAVE selection
   by `Document::record_format()` + `backend.record_capable()` is attribute-driven,
   not platform-driven.

2. **THE REAL LEAK (shell save code peeks at tab internals):**
   `WorkbenchShell::mainframe_backend_for_active_tab()`
   (`crates/ff-desktop/src/shell/dispatch_ffedit.rs`). It inspects
   `active_tab().store_identity`, looks the catalog up in
   `files_panel.registry`, and builds a MAINFRAME-specific backend per save. This
   is mainframe-specific branching sitting in the shell SAVE seam. It exists only
   because the per-save backend must be pointed at the right on-disk catalog
   (CR-CH-061 Req 19.3), and because `backend_for("MAINFRAME")` on the registry
   returns a backend built with a FIXED set of repository paths at startup.

3. **ASYMMETRY (open pushes attributes in from outside):**
   `FileOpenHandler` parses `owning_env`/`dsn`/`catalog`/`recfm`/`lrecl` from
   command params (`shell/handlers.rs`); `apply_pending_open_identity` applies them
   in a SECOND step (`shell/actions.rs`). The CALLER (Files Panel) must know the
   dataset's mainframe attributes and encode them as params; the editor/controller
   does not discover them. The record-aware read path (`DatasetAccess::open`/`get`)
   is bypassed entirely on open.

4. **SPLIT ATTRIBUTE CHANNEL:** record format lives on the `Document`
   (`ff-document-model`, `RecordFormat` enum: `Delimited { terminator }` /
   `Fixed { lrecl }` / `Variable { max_lrecl, rdw }`), while DSN/catalog/owning-env
   live on `TabState`. Two homes, two setters.

Everything else (encoding label, case-sensitivity) is already plain advisory data.

---

## B. Evaluation of the user's hypothesis, point by point

> "FFEDIT should be PLATFORM-AGNOSTIC: it should not need to know mainframe vs
> POSIX. It needs the file ATTRIBUTES and the windowed content, not the platform
> identity."

**ENDORSED, and already ~70% true.** The SAVE direction proves it is achievable:
FFEDIT addresses SAVE to a named owning environment and never names a platform.
The editor's buffer, cursor, undo, profile, find/change, exclude -- none of it
reads a platform flag. The only thing FFEDIT genuinely needs from the file is:
(a) the record framing (so the save walk frames records correctly), which is a
document ATTRIBUTE (`RecordFormat`), and (b) the owning-environment NAME to
address on save. Both are attributes, neither is "platform identity". Confirmed by
`save_active_tab_via_backend` selecting the path purely on `record_format()` +
`record_capable()` (`tab_manager.rs`).

> "On OPEN/READ, FFEDIT reads through the VFS / a file controller, receiving the
> file attributes and the windowed content."

**ENDORSED as the right target; NOT how it works today.** Today the editor reads
raw host bytes via `LocalFsProvider` and the attributes are injected from outside
as command params. The clean version -- controller returns
`(attributes, content)` -- is better AND is already physically possible, because
`DatasetAccess::open` + `get` already produce framed records and the handle
already snapshots RECFM/LRECL/DSORG (`DatasetHandle`/`HandleInner` in
`dataset_access/types.rs`). The gap is purely that the open orchestration does not
call it.

> "On SAVE, FFEDIT provides the changed data and tells the VFS to save; the
> VFS/controller -- which knows the destination -- routes to the appropriate tool."

**ENDORSED and already implemented**, via `BackendEnvironment::save` /
`save_records` dispatched by `backend_for(owning_environment)`. The "appropriate
tool" for mainframe is `MainframeEnvironment` over `DatasetAccess`; for host it is
`ff-ce-ntfs`/`ff-ce-posix`. FFEDIT hands over a `RecordSource` + `StoreTarget` +
`RecordAttrs` and does not persist anything itself.

> "Perhaps one of the file's ATTRIBUTES is the command environment to be addressed
> on save or close."

**ENDORSED -- this is exactly `TabState.owning_environment` and it already exists**
(`tab_state.rs`, `DEFAULT_OWNING_ENVIRONMENT = "HOSTFS"`). The user's instinct
matches the built design precisely. The refinement this proposal makes is: that
attribute, plus the dataset identity and record attributes, should all arrive
TOGETHER from the controller at open as ONE attributes value, instead of being set
by two separate shell methods from command params.

### What breaks / what simplifies

**Simplifies:**
- One attributes struct returned at open replaces the `owning_env` + `dsn` +
  `catalog` + `recfm` + `lrecl` param-passing (`FileOpenHandler`) AND the
  two-step `shell_open_file_with_env` + `apply_pending_open_identity`.
- The `mainframe_backend_for_active_tab` leak disappears if the opened handle
  carries enough identity for the owning environment to resolve its own catalog
  (see C.4), so the shell save seam stops being mainframe-aware.
- OPEN and SAVE become symmetric: both go through the owning environment /
  controller, both speak records+attributes.

**What must be preserved (does NOT break, by construction):**
- Native host SAVE must stay byte-identical (document-model Req 12.12). The byte
  `save` entry is retained; a Delimited document on a host backend still takes the
  byte path. Proposal keeps this untouched.
- The acyclic DAG: ff-vfs must not learn about ff-dscatalog. Proposal keeps all
  new types ff-vfs-local plain data.
- `BackendEnvironment` is a load-bearing core type (framework-conformance.md).
  Proposal does NOT reshape it (see E for the one additive method that WOULD need
  owner confirmation, offered as optional).

**Genuine risk:** today the editor reads a mainframe dataset as the host file the
catalog materialised. If OPEN is re-routed through `DatasetAccess::open`/`get`, the
editor would receive DECODED RECORDS, not raw host bytes, and the windowing /
large-file story (`ff-large-file-performance`) would have to work against a record
stream for datasets. That is a real design decision, not a mechanical change (see
E risks and F open questions).

---

## C. Proposed design (clean, minimal interface)

Guiding rule: reuse existing contracts; add the SMALLEST new surface; change
orchestration, not the load-bearing traits.

### C.1 One attributes value: `FileAttributes` (NEW, ff-vfs-local plain data)

Add a plain-data struct in `ff-vfs` (same home and same dependency-free style as
`StoreTarget`/`RecordAttrs`). It is the single thing a controller returns
alongside content at open, and the single thing the save seam reads to build its
store call.

```
// PROPOSED, crates/ff-vfs/src/backend_environment.rs (or a new attributes.rs)
pub struct FileAttributes {
    /// Name of the owning BackendEnvironment to address on SAVE/CLOSE.
    /// Replaces TabState.owning_environment as the single source.
    pub owning_env: String,
    /// Dataset identity when the resource is a dataset; None for a plain host file.
    /// Carries what StoreIdentity carries today (dsn + catalog), as ff-vfs-local data.
    pub store_target: Option<StoreTarget>,
    /// Record framing + LRECL + encoding. For a host text file this is the
    /// Delimited-equivalent (see note); for a dataset it is the RECFM/LRECL.
    pub record_attrs: Option<RecordAttrs>,
    /// Advisory host facets (case sensitivity, read-only, etc.) as needed.
    pub case_sensitive: bool,
}
```

Notes:
- `RecordAttrs` already exists and is reused. The ONE representational mismatch:
  `RecordFormatKind` has no `Delimited` variant (it is `Fixed`/`Variable`/
  `Undefined`), whereas the editor's `ff_document_model::RecordFormat` has
  `Delimited { terminator }`. Keep the mapping where it already is
  (`record_attrs_for` in `tab_manager.rs`): `record_attrs = None` means "native /
  Delimited -> byte path", exactly the current signal. No new enum variant needed,
  so `RecordFormatKind` is NOT reshaped.
- `FileAttributes` is plain data -- no ff-dscatalog types, DAG preserved.

### C.2 OPEN: controller returns `(FileAttributes, content)` as one unit

Change the OPEN ORCHESTRATION (not a core trait) so the thing that resolves a
resource returns its attributes with its content, instead of the shell pushing
params.

Concretely, collapse the current two shell steps into one:
- Replace `PendingOpenReq { path, owning_env, identity, recfm_lrecl }` with
  `PendingOpenReq { path, attributes: Option<FileAttributes> }` where `None` means
  "plain host open, derive defaults" (behaviour-preserving for every existing host
  open) and `Some(attrs)` is what a dataset open supplies.
- `TabManager::open_file` keeps reading content; `TabState` stores the attributes
  as a single field (`attributes: FileAttributes`) instead of the separate
  `owning_environment` + `store_identity` + document-side record format being set
  by two methods. `shell_open_file_with_env` + `apply_pending_open_identity`
  collapse into one `apply_open_attributes`.

This is an orchestration refactor inside `ff-desktop` -- it touches
`shell/handlers.rs`, `shell/state.rs`, `shell/actions.rs`, `shell/update.rs`,
`tab_state.rs`, `tab_manager.rs`, but NO ff-vfs trait and NO cross-crate contract.
It is framework-conformant (no new dispatch path, no new persistence format).

OPTIONAL stronger version (owner decision, see E/F): make the Files Panel resolve
a dataset through `DatasetAccess::open`/`get` and return records-as-content +
`FileAttributes` derived from the `DatasetHandle` (RECFM/LRECL/DSORG). This is what
truly makes "the controller returns attributes + content" real and closes the
open/save asymmetry, but it changes how dataset content enters the editor (records
vs host bytes) and intersects large-file windowing -- hence owner-gated.

### C.3 SAVE: unchanged seam, reads attributes off the handle

`host_fs_save` -> `save_active_tab_via_backend` stays the single seam. It reads
`FileAttributes.record_attrs` (record vs byte selection) and
`FileAttributes.owning_env` (which backend), builds `StoreTarget` from
`FileAttributes.store_target`. This is what it does today, only sourcing from one
struct instead of three fields. No change to `BackendEnvironment`.

### C.4 Eliminating the `mainframe_backend_for_active_tab` leak

The leak exists because `backend_for("MAINFRAME")` returns a backend wired to a
FIXED set of catalog repository paths chosen at startup, so SAVE has to build a
per-save backend pointed at the active tab's catalog
(`shell/dispatch_ffedit.rs` + `shell/mainframe_backend.rs`). Two framework-conformant
ways to remove the shell-side peek, in order of preference:

**Option 1 (preferred, no core-type change): make the MAINFRAME backend catalog-aware
from the `StoreTarget` it already receives.** `save_records` already gets
`StoreTarget { dsn, owning_env, catalog_id }`. `catalog_id` is the catalog name.
Give `MainframeEnvironment` (in ff-idcams) a `CatalogRepositoryAccess` capable of
resolving the catalog repository path FROM the catalog name at call time (it
already owns an `Arc<dyn DatasetAccess>` built over `CatalogRepositoryAccess`,
`crates/ff-desktop/src/shell/mainframe_backend.rs`). Then the registry's single
`backend_for("MAINFRAME")` backend serves ALL catalogs, and
`mainframe_backend_for_active_tab` is deleted. The shell save seam becomes
platform-agnostic again. This is a change WITHIN ff-idcams + how the catalog
registry resolves a name->path; it does not touch `BackendEnvironment` or ff-vfs.

**Option 2 (if Option 1's name->path resolution is not available at save time):**
carry the resolved catalog repository path as part of the opened `FileAttributes`
(an extra optional field), so the save seam passes it through to the backend via
`StoreTarget`/an additive attr rather than re-deriving it from
`files_panel.registry`. Still no `BackendEnvironment` reshape, but it widens
`StoreTarget` (a core-adjacent ff-vfs type) -- see E for the owner-confirmation
note on `StoreTarget`.

Either way the GOAL is: the shell save seam reads attributes off the handle and
addresses the owning environment; it never builds a mainframe-specific backend.

### C.5 The save/close addressing rule (stated plainly)

> On SAVE (and later CLOSE-with-save), FFEDIT addresses the verb to
> `FileAttributes.owning_env`. The resolved `BackendEnvironment` persists the
> changed data: records via `save_records(StoreTarget, RecordSource, RecordAttrs)`
> when `record_attrs` is `Some` and the backend is `record_capable()`, else bytes
> via `save(path, bytes)`. FFEDIT performs no persistence and names no platform.

This is the current rule, made explicit and sourced from one attributes value.

### C.6 Object-safety and DAG (preserved)

- `FileAttributes` is plain data (no methods needed for dispatch); `StoreTarget`,
  `RecordAttrs`, `RecordSource`, `BackendEnvironment`, `DatasetAccess` are all
  already object-safe (verified: each has an `_assert_object_safe` guard or a
  boxed-dyn test).
- ff-vfs gains no dependency. ff-idcams/ff-dscatalog continue to depend on ff-vfs,
  never the reverse. DAG unchanged.

---

## D. JES forward-fit: one dataset/record contract for editor AND batch

**The shared abstraction already exists and is already designated for both
consumers: `ff_dscatalog::DatasetAccess` (CR-CH-059).** Its own module doc
(`crates/ff-dscatalog/src/dataset_access/trait_def.rs`) states it is THE single
record-aware contract for "the JCL executor, the editor's future MAINFRAME
BackendEnvironment SAVE path, and IDCAMS". Method set (object-safe):
`allocate(&DdRequest)`, `resolve(dsn, intent)`, `open(handle, intent)`,
`get(&mut open) -> Option<Record>`, `put(&mut open, &Record)`,
`point(&mut open, &Positioner)` (KSDS key / RRDS-ESDS RRN), `close(open)`,
`dispose(handle, StepOutcome)`.

How the two clients share it:

| Concern | Interactive editor (FFEDIT) | Batch program (JES executor) |
|---------|-----------------------------|------------------------------|
| Identity in | `StoreTarget { dsn, catalog_id }` from `FileAttributes` | `DdRequest` (DD/DISP/SPACE) from `ff-dsalloc` -> `DatasetHandle` |
| Open | `resolve(dsn, Write)` -> `open` (today, save side) | `allocate`/`resolve` -> `open(handle, intent)` |
| Records out | `get` loop decodes via RECFM codec (proposed for open) | `get` loop per DD (INPUT) |
| Records in | `put` loop from editor `RecordSource` -> `close` | `put` loop from program output -> `close` |
| RC / abend | `RecordStoreOutcome::Stored { rc }` (x37=37, etc.) | `DatasetError` -> step RC / abend |
| Disposition | n/a (edit-in-place) | `dispose(handle, KEEP/CATLG/DELETE/PASS/UNCATLG)` |

The JES design INDEPENDENTLY commits to this (CR-CH-059 "Design Delta: JCL
Executor Depends Only on ff-dsalloc + DatasetAccess",
`docs/specs/jes-emulator/design.md`): the future executor depends ONLY on
`ff-dsalloc` (DD/DISP allocation producing a `DatasetHandle`) and `DatasetAccess`
(record I/O) -- never on SQLite, a raw `storage_path`, or `physical_path`. So:

**The interface FFEDIT should consume on open/save is a THIN editor-facing adapter
over the SAME `DatasetAccess` the JES executor will consume.** The editor's
`MainframeEnvironment` (ff-idcams) is ALREADY exactly that adapter on the save
side (`save_records` -> `resolve`/`open`/`put`/`close` over `DatasetAccess`,
`crates/ff-idcams/src/mainframe_env.rs`). The proposal's open-side symmetry
(C.2 optional) would make the editor consume `DatasetAccess::open`/`get` too,
through the controller, so editor and batch read the SAME way.

What the JES side ADDITIONALLY needs, and whether it fits:
- **DD allocation + DISP + concatenation + GDG:** lives in `ff-dsalloc`
  (`dd_statement.rs`, `concatenation.rs`, `gdg_resolver.rs`, `job_model.rs`;
  `AllocatorService` trait in `traits.rs`). It produces a `DdRequest` / resolves a
  DSN; `DatasetAccess::allocate(&DdRequest)` consumes it. This is a BATCH concern
  the editor does not need, and it FITS as a PRE-STEP that produces the same
  `DatasetHandle` the editor's `resolve` produces. No change to the shared contract.
- **SYSIN/SYSOUT and spool:** JES stores job logs/SYSOUT via the `VfsProvider`
  seam (jes-emulator design: `vfs://local/.ffwb/spool/...`). SYSOUT is a spool
  resource, not a catalogued dataset; it uses Seam 1, not `DatasetAccess`. A SYSIN
  DD pointing at a real dataset uses `DatasetAccess` like any other DD. This fits
  the two-seam model as-is.
- **Status: ff-jes today is job management only.** `crates/ff-jes/` implements
  queue/scheduler/initiator/SDSF/FFJCL-parse (`provider.rs`, `model.rs`,
  `ffjcl.rs`); it has NO `DatasetAccess` wiring yet and NO program-execution
  record I/O. The `JobExecutor`/`DatasetApi` in the design
  (`docs/specs/jes-emulator/design.md`) are DEFERRED (ROADMAP PLUGIN phase 6). So
  the forward-fit is a DESIGN alignment, not an integration that can be wired now.

Conclusion: the proposed editor interface and the future JES executor share ONE
dataset/record access contract (`DatasetAccess`) plus ONE identity-production path
(`ff-dsalloc` for batch, the catalog resolve for the editor). The editor needs a
strict SUBSET of what JES needs (no DD/DISP/GDG, no dispose), so designing the
editor interface as a thin adapter over `DatasetAccess` does not constrain JES; it
pre-aligns with it.

---

## E. Trade-offs, risks, and migration sketch

### E.1 Changes that need OWNER CONFIRMATION before any implementation

Per `framework-conformance.md`, flag every touch of a load-bearing core type or
the crate DAG:

1. **ANY reshape of `BackendEnvironment`** (its byte `save` / record `save_records`
   store contract) -- load-bearing core type, CR-CH-060 owner-confirmed as it
   stands. THIS PROPOSAL'S CORE PATH DOES NOT RESHAPE IT. If the owner later wants
   a symmetric record-aware LOAD entry on `BackendEnvironment` (e.g.
   `open_records(&StoreTarget) -> RecordSource`) so OPEN is routed through the same
   addressed backend instead of through `DatasetAccess` directly, THAT is an
   additive reshape of the store contract and REQUIRES owner confirmation.
   (Offered as optional; not required for B/C/D above.)
2. **Widening `StoreTarget`** (adding e.g. a resolved catalog repository path for
   C.4 Option 2). `StoreTarget` is an ff-vfs core-adjacent type threaded through
   the save seam; widening it is additive but still a core-type change -- CONFIRM.
   C.4 Option 1 avoids this.
3. **Routing editor OPEN through `DatasetAccess` for datasets (C.2 optional):**
   not a type reshape, but it changes HOW dataset content enters the editor
   (decoded records vs materialised host bytes) and intersects the large-file
   windowing design. Because it changes observable open behaviour for datasets,
   treat it as a gated change request, not a silent refactor.
4. **No DAG change is proposed.** If any future step required ff-vfs to know
   ff-dscatalog types, that WOULD be a DAG violation needing confirmation; this
   proposal explicitly avoids it by keeping `FileAttributes` plain data.

### E.2 Changes that are framework-conformant and need NO core-type confirmation

- Introducing `FileAttributes` as ff-vfs-local plain data (new type, not a reshape
  of an existing core type) and collapsing the two open-orchestration steps into
  one. This is shell-internal orchestration + one plain struct.
- Deleting `mainframe_backend_for_active_tab` by making `MainframeEnvironment`
  resolve its catalog from the `StoreTarget.catalog_id` it already receives
  (C.4 Option 1) -- a change inside ff-idcams + catalog-registry name resolution.
- Unifying `TabState.owning_environment` + `store_identity` into one
  `attributes: FileAttributes` field.

### E.3 Migration sketch (incremental, each step leaves a green build)

1. **Add `FileAttributes` (plain data) to ff-vfs.** Reuses `StoreTarget` /
   `RecordAttrs`. No behaviour change; nothing consumes it yet.
2. **Collapse the open orchestration (shell-only).** Replace the `PendingOpenReq`
   quad-of-options + two-step apply with `FileAttributes`; keep `None` =>
   host-default so every existing host open is byte-identical. Update the
   `file.open` handler to build `FileAttributes`. Full-shell open/save tests
   (`shell/tests_command.rs`, `tests_crch061.rs`) must still pass.
3. **Remove the save-seam leak (C.4 Option 1).** Make `MainframeEnvironment`
   catalog-name aware; delete `mainframe_backend_for_active_tab`; registry's single
   `backend_for("MAINFRAME")` serves all catalogs. Re-run the CR-CH-061 catalog
   save tests.
4. **(Owner-gated) Symmetric OPEN via `DatasetAccess`.** Only if the owner approves
   E.1.3: route dataset open through `DatasetAccess::open`/`get`, deriving
   `FileAttributes` from the `DatasetHandle`, reconciled with large-file windowing.
5. **(Owner-gated, optional) `open_records` on `BackendEnvironment`.** Only if the
   owner wants OPEN addressed through the owning environment symmetrically with
   SAVE (E.1.1).

Steps 1-3 deliver the user's goal (FFEDIT platform-agnostic, destination identity
on the handle, leak removed, one attribute channel) WITHOUT touching a
load-bearing contract or the DAG. Steps 4-5 are the deeper symmetry, owner-gated.

### E.4 Residual risks

- **Large-file windowing for datasets (step 4).** The editor's "windowed content"
  story (`ff-large-file-performance`, `ff-viewport-scrolling`) is built around a
  byte/line document. Feeding it a record stream from `DatasetAccess::get` needs a
  record-addressed window, not a byte window. Non-trivial; do not bundle into the
  orchestration refactor.
- **Encoding.** `RecordAttrs.encoding` is an opaque string ("cp037"/"utf-8"); the
  save path currently hardcodes "utf-8" in `save_active_tab_via_backend`
  (`tab_manager.rs`). If datasets are EBCDIC, encoding must flow from
  `FileAttributes` into the store call rather than being hardcoded. `ff-dscatalog`
  has an `encoding.rs` + codecs; wire the real label through `FileAttributes`.
- **Byte-identical host SAVE is sacrosanct** (document-model Req 12.12). Every step
  must keep the Delimited/host path on the byte `save` entry.

---

## F. Open questions for the owner

1. **Open symmetry depth.** Do you want the editor to OPEN mainframe datasets
   through `DatasetAccess::open`/`get` (true controller-returns-records), or keep
   opening the materialised host file and only treat records specially on SAVE (as
   today)? This is the single biggest decision; it drives whether step 4 happens
   and how large-file windowing must change.

2. **`open_records` on `BackendEnvironment`?** Should OPEN be addressed to the
   owning environment symmetrically with SAVE (an additive record-aware LOAD entry
   on the store contract), or is it acceptable for OPEN to consume `DatasetAccess`
   directly via the controller while SAVE goes through `BackendEnvironment`? The
   former is cleaner conceptually but is a confirmed reshape of a load-bearing type.

3. **Catalog resolution at save (C.4).** Can `MainframeEnvironment` resolve a
   catalog name -> repository path at call time (Option 1, no core-type change), or
   must the resolved path be carried on the handle (Option 2, widens `StoreTarget`)?
   This decides whether removing the leak touches a core-adjacent type.

4. **Encoding model.** Should `FileAttributes` carry a first-class encoding (EBCDIC
   code pages for datasets) that flows into both read decode and write encode,
   replacing the hardcoded "utf-8" in the save path?

5. **Scope now vs later.** Steps 1-3 are safe and deliver the user's stated goal.
   Do you want this slice gated and implemented now, with steps 4-5 deferred to
   when the JES executor / large-file dataset windowing are actually built?

---

## Appendix: evidence index (files/symbols read)

- Seams: `crates/ff-vfs/src/{lib.rs, provider.rs, backend_environment.rs, Cargo.toml}`.
- Editor open: `crates/ff-desktop/src/tab_manager.rs` (`open_file`),
  `crates/ff-files-panel/src/resolve.rs` (`resolve_dataset_path`,
  `resolve_and_open_dataset`), `crates/ff-desktop/src/shell/{handlers.rs
  (FileOpenHandler), state.rs (PendingOpenReq), update.rs, actions.rs
  (shell_open_file_with_env, apply_pending_open_identity)}`.
- Editor save: `crates/ff-desktop/src/shell/dispatch_ffedit.rs` (`host_fs_save`,
  `mainframe_backend_for_active_tab`), `tab_manager.rs`
  (`save_active_tab_via_backend`, `record_attrs_for`, `RecordImageSource`),
  `crates/ff-desktop/src/shell/{environment_registry.rs (backend_for),
  mainframe_backend.rs, commands_environment.rs}`.
- Attributes: `crates/ff-desktop/src/tab_state.rs` (`TabState`, `StoreIdentity`,
  `DEFAULT_OWNING_ENVIRONMENT`), `crates/ff-document-model/src/record_format.rs`
  (`RecordFormat`, `DelimiterTerminator`), `text_buffer.rs`
  (`record_format`/`set_record_format`).
- Mainframe/VSAM: `crates/ff-idcams/src/mainframe_env.rs` (`MainframeEnvironment`),
  `crates/ff-dscatalog/src/dataset_access/{trait_def.rs (DatasetAccess), types.rs
  (DdRequest, DatasetHandle, OpenDataset, AccessIntent, Positioner, StepOutcome,
  VsamOpen), impl_io.rs (codec_for, open_impl)}`, VSAM in
  `crates/ff-dscatalog/src/{vsam_backend.rs, vsam_service.rs, storage/{esds,isam,rrds}.rs}`,
  `crates/ff-dsalloc/src/traits.rs` (`AllocatorService`).
- JES: `crates/ff-jes/src/{lib.rs, provider.rs, ffjcl.rs (FfjclDd), model.rs}`,
  `docs/specs/jes-emulator/design.md` (CR-CH-059 Design Delta; ff-dsalloc +
  DatasetAccess dependency; SYSOUT via VFS).
- Vision/constraints: `docs/specs/command-environments/environments-vision.md`
  (FFEDIT/FFAMS/FFJES/FFJOB), `.kiro/steering/{framework-conformance.md,
  wiring-standard.md}`.

Content was rephrased for compliance with licensing restrictions where external
mainframe terminology is summarised.
