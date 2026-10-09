# CR-CH-058 (windowed record-oriented document model) and RC.B.8 Part 2 (record-aware MAINFRAME editor SAVE): overlap, reconciliation, and integration plan

READ-ONLY investigation. No code or docs were changed. Design-input only; no gate was run.
Workspace root: `c:\workspace\VSC\FileForgeWorkbench`.

---

## Executive summary

**One-line verdict:** The two efforts are not rivals and do not conflict -- they are
two halves of ONE record-aware-save story that the designs already deliberately
split: **CR-CH-058 builds the editor-side record model and the piece-list SAVE walk
that RE-FRAMES per `RecordFormat`, and it is explicitly designed to "ride" the exact
same SAVE-addressing seam (CR-CH-053 Task 20/21 -> owning Command Environment) that
RC.B.8 Part 2 is blocked on; RC.B.8 Part 2 is the mainframe specialisation that lands
on top of CR-CH-058's foundation.** The owner's recollection -- "CR-CH-058 is also
trying to align with the mainframe vision" -- is **CONFIRMED, in writing, in both
specs and the design doc.**

Consequences for sequencing:

- **CR-CH-058 should land first (or at least its F1 record foundation).** RC.B.8 Part 2
  needs the editor to HAVE first-class records framed by a `RecordFormat` before a
  mainframe SAVE has anything RECFM/LRECL-correct to pack. Today the editor's save
  produces one flat byte buffer (`doc.contiguous_view().to_vec()`); there are no
  record boundaries to hand to `DatasetAccess::put`.
- **The framework change can and should be designed ONCE.** CR-CH-058's SAVE walk and
  RC.B.8 Part 2 prerequisite (a) both need the same thing: a SAVE path that carries
  records + dataset identity to the owning CE instead of `(path, bytes)`. Designing
  `BackendEnvironment`'s record-aware store contract for both at the same time avoids
  doing the framework change twice.
- **CR-CH-058 is already partly built** in the `.worktrees/wrf-foundation` worktree
  (branch `feature/windowed-record-foundation`): F1 (record abstraction + piece-table
  spine + byte-identical native SAVE) and the document-model-local half of F2
  (`WindowBand`) are committed and scoped-green (201 tests), but NOT merged to `main`.
  RC.B.8 Part 2's prerequisites (a)-(e) remain entirely unbuilt.

Biggest risk if they are built independently: the SAVE seam is reshaped twice (first a
line/byte-oriented reshape for CR-CH-058, then a second reshape for mainframe records),
and CR-CH-058's piece-list re-framing logic is duplicated by a separate mainframe
packer. Both are wasted effort that the integration plan (section F) avoids.

---

## A. Scope and status of CR-CH-058

**What it is (one paragraph).** CR-CH-058 reworks `ff-document-model` from a single
fully-resident gap buffer into a **piece-table spine with windowed byte residency**:
an immutable lean flat ORIGINAL index (one ~16-byte entry per original record), a small
ordered PIECE LIST of edits (Original/Edited pieces), and an Append_Buffer (the
surviving gap-buffer role). Crucially it makes the editable unit a **universal RECORD**
whose framing is a `RecordFormat` supplied by the owning Command Environment:
`Delimited { terminator }` (native, generalising `LineEndMode`), `Fixed { lrecl }`
(mainframe FB), `Variable { max_lrecl, rdw }` (mainframe VB). Scroll/extents size on the
index `Total_Records` (fixing the owner's scrollbar-collapse bug), zoom never loads, and
native (Delimited) editing stays BYTE-IDENTICAL.

**Specs it drives** (change-log `CR-CH-058` entry, `docs/status/change-log.md`):
`document-model` (Req 11-12 + revised Req 4/glossary), `viewport-and-scrolling` (Req 15 +
revised 1.10/2), `large-file-performance` (Req 7/7.6 -> mandatory range access),
`display-line-mapping` (Req 11), `edit-operations` (Req 18), `undo-redo-transactions`
(Req 20), `find-and-replace` (Req 21). Project-master Phase (windowed-record-foundation)
with WRF.1-WRF.6.

**Crates:** `ff-document-model` (spine), `ff-viewport-scrolling`, `ff-display-line-mapping`,
`ff-edit-operations`, `ff-undo-redo`, `ff-find-and-replace`, `ff-large-file-performance`
(consuming), `ff-idle-processing` (background scan/prefetch).

**Status: PARTIALLY LANDED in a worktree, not on main.**
- Change-log status line: `Phase (windowed-record-foundation) -- PENDING GATE`.
- Git (`git worktree list`): worktree `.worktrees/wrf-foundation` on branch
  `feature/windowed-record-foundation`, HEAD `74a0414`, **clean working tree**,
  **9 commits ahead of / 2 behind `main`**.
- Commit history on that branch (newest first) shows the real build state:
  - `74a0414 CR-CH-058 F2 (document-model slice): Window_Band + Total_Records authority`
  - `b63f8c0 CR-CH-058 F1 (part 2b): piece-table proptests + TCR/tasks + impl note (F1 complete)`
  - `cb856f3 CR-CH-058 F1 (part 2a): record API + byte-identical SAVE wired into TextBuffer/Document`
  - `3aeb293 CR-CH-058 F1 (part 1/2): piece-table data-structure layer`
  - `cdb0389 CR-CH-058: requirements gate ... (docs only)`
  - and three `docs(wrf-f1)` commits recording that the **F1 merge to main is held**
    pending the owner committing in-progress dataset/volume work.
- `.agents/tasks/wrf-foundation/F2-doc-model-slice-note.md`: F2 delivered only the
  `WindowBand` primitive in `ff-document-model`; it DELIBERATELY does not touch the
  shell/viewport/scrollbar wiring "because the other chat is actively editing
  `ff-desktop/shell/render_*.rs` (CR-CH-059)" -- an explicit, recorded awareness that
  the two efforts share the shell and must not collide. Scoped tests: 201 passed.

**Key cross-dependency visible in git:** the `wrf-foundation` branch is built ON TOP of
CR-CH-059's RC.A commit (`216c3cd CR-CH-059 RC.A+RC.B.5+RC.B.6: consolidate dataset
stack, add ff-volume and the DatasetAccess contract`). So on this one branch the
`DatasetAccess` contract AND the piece-table record model already coexist -- the
integration surface is literally already assembled in that worktree, un-merged.

**TCR:** `docs/quality/TCR.md` Phase (windowed-record-foundation) rows for Req 11.1-11.5
are present (status mostly red/building).

---

## B. Overlap -- where they touch, and conflict vs complement

They meet at exactly four places, and in every one they COMPLEMENT (one supplies what
the other needs); none is a design conflict.

1. **The editor document model / the notion of a "record".**
   - CR-CH-058 introduces `RecordFormat` + `Record` as the universal editable unit in
     `ff-document-model` (`document-model` Req 11.1-11.5).
   - RC.B.8 Part 2 needs `Record` boundaries to pack via `DatasetAccess::put`
     (`part2-mainframe-save-stop.md` evidence point 1).
   - COMPLEMENT: CR-CH-058 is the only thing that gives the editor records at all.
     Without it there is nothing to pack. (See section C for the reconciliation.)

2. **The SAVE path.**
   - Today: `tab_manager.rs::save_active_tab_via_backend` reads
     `doc.contiguous_view().to_vec()` (one flat byte buffer) and calls
     `backend.save(&path, &bytes)` on a `dyn ff_vfs::BackendEnvironment`
     (`shell/dispatch_ffedit.rs::host_fs_save` -> `save_active_tab_via_backend`).
   - CR-CH-058's FOUNDATION-DESIGN section 6 redefines SAVE as: walk the piece list,
     emit each piece's bytes RE-FRAMED per the owning CE's `RecordFormat`
     (re-pad LRECL for Fixed, delimiter for Delimited, RDW for Variable), atomic write.
   - RC.B.8 Part 2 is blocked on the SAME path becoming record-aware (prereq a).
   - COMPLEMENT / shared seam: both ride the CR-CH-053 Task 20/21 "FFEDIT addresses SAVE
     to the owning CE" seam -- stated verbatim in FOUNDATION-DESIGN section 6
     ("The CR-CH-053 Task 20/21 SAVE-addressing-to-the-owning-CE path is the seam this
     rides (record-aware save lives in the owning CE)") and in `document-model`
     requirements Cross-References ("command-environments ... is the SAVE-addressing seam
     (CR-CH-053 Task 20/21)").

3. **The `ff-vfs::BackendEnvironment` contract.**
   - RC.B.8 Part 2 prereq (a): `save(&self, path: &Path, bytes: &[u8])` cannot carry a
     DSN/RECFM/LRECL/`Record` boundaries; reshaping it is a FRAMEWORK CHANGE.
   - CR-CH-058's record-aware SAVE walk needs the SAME richer store call to deliver
     re-framed records to the owning CE rather than a flat byte slice.
   - COMPLEMENT: one contract reshape serves both (section E).

4. **`owning_env` binding.**
   - CR-CH-053 already binds every tab to an `owning_environment` (default `"HOSTFS"`,
     `tab_state.rs::DEFAULT_OWNING_ENVIRONMENT`); FFEDIT addresses SAVE there.
   - RC.B.8 Part 2 prereq (c): `open_mainframe_dsn` must pass `owning_env="MAINFRAME"`
     so a mainframe dataset tab binds to the mainframe CE instead of HOSTFS. Today it
     passes only `path` into `file.open` (evidence point 3), so the tab defaults to
     HOSTFS and (correctly) does a host byte write.
   - COMPLEMENT: CR-CH-058 is agnostic to WHICH env owns the tab; it just asks the
     owning CE for the `RecordFormat` (`document-model` Req 11.2). RC.B.8 supplies the
     mainframe binding + the mainframe CE that returns `Fixed`/`Variable`.

**No conflicts found.** The two designs were authored aware of each other: CR-CH-058's
change-log entry ends "the mainframe Fixed/Variable RecordFormat + record-aware save is
a LATER V-stream gate built ON this foundation," and RC.B.8 Part 2's spec defers the
mainframe CE to "a later phase." They point at each other.

---

## C. The record reconciliation (the key question)

**Do CR-CH-058's windowed records and the dataset's RECFM records align, and where do
they meet on SAVE?** -- They align by DESIGN; they are the same byte-level notion modelled
in two layers, and they meet in the owning CE's record-aware store call.

Two `Record` notions exist, intentionally:

| Layer | Type | Shape | Role |
|-------|------|-------|------|
| Editor (CR-CH-058) | `ff_document_model::Record` + `RecordFormat` | logical editable unit, framing = Delimited/Fixed/Variable | what the user edits; the piece list splices these |
| Dataset (CR-CH-059) | `ff_dscatalog::Record { key: Vec<u8>, data: Vec<u8> }` + `RecordCodec` | raw record bytes; boundaries from the RECFM codec | what `DatasetAccess::put`/`get` move to/from the SQLite/host store |

These are NOT a conflict -- they are the two ends of the same pipe:

- `ff-dscatalog` already ships the RECFM codecs that frame bytes:
  `codecs::{FixedCodec, VariableCodec, BinaryCodec, TextCodec}` all impl `RecordCodec`
  (`encode(&[Vec<u8>]) -> bytes` / decode), selected by RECFM in
  `dataset_access/impl_io.rs::codec_for` (`Recfm::F|FB -> FixedCodec`, `V|VB ->
  VariableCodec`, `U -> BinaryCodec`). `DatasetAccess::put(open, record)` appends a
  `Record` and the codec frames it on `close` (trait doc + `impl_io.rs::put_impl`).
  Test `get_put_use_codec_boundaries_not_crlf` (Req 34.2) proves record boundaries come
  from the codec, never CRLF.
- CR-CH-058's `RecordFormat::Fixed { lrecl }` / `Variable { max_lrecl, rdw }` are the
  SAME framing one layer up -- the FOUNDATION-DESIGN says so explicitly
  (`Fixed`/`Variable` "generalising `ff-dscatalog::Recfm` + LRECL") and calls the move
  "This REPLACES the provider's current flatten-to-`\n`." `document-model` Req 11.5
  states the editor must NOT flatten Fixed/Variable records "so a later Fixed-format save
  can re-pad to LRECL."

**Where they meet on SAVE:** the owning (mainframe) CE. CR-CH-058's SAVE walk produces
re-framed records from the piece list; the mainframe CE converts those into
`ff_dscatalog::Record`s and drives `DatasetAccess::put` (then `close`, which runs the
RECFM codec). This is prereq (e): "the mainframe CE ... implements the record-aware
contract from (a) over `ff_dscatalog::DatasetAccess` (allocate/open/get/put/.../dispose)."

**What has to line up for an edited mainframe dataset to save correctly:**
- **Record boundaries:** the editor's `RecordFormat` must match the dataset's RECFM, so
  CR-CH-058 Req 11.2 says the CE SUPPLIES the format at open (single source of truth; no
  double-definition).
- **RECFM/LRECL:** supplied by the dataset's catalog attributes (`ff_dscatalog::Recfm` +
  LRECL), fed into both the editor `RecordFormat` (open) and the codec (save).
- **Encoding:** `TextCodec` already models an encoding profile (host text <-> record
  bytes) -- the encoding seam exists on the dataset side; CR-CH-058 keeps the editor
  encoding-aware (`Document` wraps encoding).
- **x37/space:** `DatasetAccess::put` already surfaces x37 space-full abends
  (`put_growth_surfaces_x37_space_abend`, Req 34.4), so a record-aware save gets correct
  mainframe failure semantics for free.

Net: the abstractions were deliberately designed to meet. The only missing glue is the
contract that carries records from the editor SAVE walk into the mainframe CE
(section E) and the mainframe CE itself (prereq e).

---

## D. Dependency direction and ordering

**CR-CH-058 is the foundation; RC.B.8 Part 2 is the specialisation on top.** Stated
directly in FOUNDATION-DESIGN section 7 phasing: F1-F5 build the universal model, "Then
the `V`-stream mainframe CE supplies `RecordFormat::Fixed/Variable` + record-aware SAVE
on this foundation (separate gate)."

**Does record-aware SAVE need CR-CH-058's document model first?** For the EDITOR half,
yes -- there must be records to pack. Today SAVE flattens to one byte buffer
(`contiguous_view().to_vec()`), so there is nothing RECFM-correct to hand to
`DatasetAccess::put`. The editor cannot even open a mainframe-owned editable tab yet
(evidence point 3: nav "rejects mainframe dataset editing as 'available in a later
update'"), which is why RC.B.8 Part 2 notes the first-fail test "cannot be written
today -- no MAINFRAME-owned tab exists to assert against."

**Can the non-editor prerequisites proceed independently/in parallel?** Yes -- (b), (c),
(d) are shell/registry/provider wiring that do not need the piece-table model:
- (b) open named-backend registry + `dispatch_to_environment` arm (`environment_registry.rs`,
  `commands_environment.rs`) -- today a closed enum `RegisteredEnv { FfCmdBase, FfEdit,
  HostFsPlaceholder }` with a single boxed `host_fs` backend.
- (c) `open_mainframe_dsn` passing `owning_env="MAINFRAME"` (`render_body.rs`,
  `render_body_arms.rs`, `render_nav.rs`).
- (d) register the mainframe VFS provider live in `construct_provider.rs::build_live_provider_registry`
  (today seeds only the host-FS `local` provider; Req 17.4).

**Interim option (important for ordering):** an INTERIM record-aware save with a simple
line-based packing (split current flat bytes on the delimiter, re-pad to LRECL via the
existing `FixedCodec`) could be wired through (a)+(b)+(c)+(d)+(e) WITHOUT the full
CR-CH-058 piece table -- the dataset-side codecs already exist. But this interim packer
is throwaway once CR-CH-058's `RecordFormat`-aware SAVE walk lands, and it cannot keep FB
structure "real" for records longer/shorter than a delimited line. Recommendation: do NOT
build the interim packer unless there is urgent standalone demand; prefer sequencing
CR-CH-058 F1 first so the real record SAVE is built once.

**True blockers each way:**
- RC.B.8 editor-correctness blocker -> CR-CH-058 F1 (records exist, SAVE walk re-frames).
- RC.B.8 reachability blockers -> (c) tab binding + (d) provider registration + (e)
  mainframe CE in `ff-idcams` + the editor-nav gate that currently refuses mainframe edit.
- Shared blocker for BOTH record-aware SAVE paths -> the (a) framework contract reshape.

---

## E. The framework change -- design it once

**Prereq (a) restated:** `ff-vfs::BackendEnvironment::save(&self, path: &Path,
bytes: &[u8]) -> io::Result<()>` cannot express a record-aware store: it carries no DSN,
no RECFM/LRECL/catalog identity, no `Record` boundaries. The fix is to EITHER change the
`save` signature to a record-aware form OR add a record-aware addressing variant
alongside the byte `save`. Because `BackendEnvironment` is a load-bearing core framework
type, this is a FRAMEWORK CHANGE requiring express owner confirmation
(framework-conformance.md). (`part2-mainframe-save-stop.md` prereq a; evidence point 1 --
the trait's own doc comment already acknowledges a mainframe backend "would instead pack
records per RECFM/LRECL" but the signature does not allow it.)

**Does CR-CH-058 already imply the same change?** Yes. CR-CH-058's SAVE walk needs to
deliver RE-FRAMED records (not a flat byte slice) to the owning CE for Fixed/Variable
documents; for Delimited it still produces bytes, but the shape of the call it wants is
"hand the owning CE the records (or a record stream) + the dataset identity and let the CE
write." That is the identical reshape (a) describes. CR-CH-058's change-log framework-note
says SAVE "rides the existing CR-CH-053 Task 20/21 'FFEDIT addresses SAVE to the owning
CE' seam" -- i.e. it expects the owning-CE store call to be the record-aware one.

**Design-once recommendation:** author ONE framework-change gate for the
`BackendEnvironment` store contract that satisfies BOTH:
- keep the byte `save(path, bytes)` for the light host-FS CEs (native byte-identical,
  Req 16.5 -- unchanged),
- ADD a record-aware store entry (e.g. `save_records(target, records, attrs)` where
  `target` carries DSN/catalog identity and `attrs` carries RECFM/LRECL/encoding) that
  the mainframe CE implements over `DatasetAccess::put`,
- have CR-CH-058's SAVE walk call the byte form for Delimited/host and the record form
  for a CE that advertises a Fixed/Variable `RecordFormat`.
Doing this once avoids reshaping the seam twice and avoids CR-CH-058 and the mainframe
packer each inventing a different "records -> store" call.

**Note:** CR-CH-099 (cited in the brief as "wire the live editor onto the existing
editor-aspect crates") was NOT found in `docs/status/change-log.md` (grep returned no
match). The nearest live editor-refactor item is CR-NR-107 (WAL/WBL via the external
Walrus project) which is unrelated to the SAVE seam. If the owner has a CR-CH-099 in
mind, it is not recorded in the change-log read here; treat the editor-wiring concern as
covered by CR-CH-058 F-phase wiring (`tab_manager::save_active_tab_via_backend` is named
as a wiring target in FOUNDATION-DESIGN section "integration points").

---

## F. Recommended integration plan

Goal: move BOTH forward together, build the record model and the SAVE seam ONCE, and keep
the two efforts off each other's toes on the shell SAVE path.

**Owner decisions required (not mechanical):**
1. Confirm the (a) FRAMEWORK CHANGE to `ff-vfs::BackendEnvironment` and direct that it be
   designed once to serve both CR-CH-058 and RC.B.8 Part 2 (section E). This is the single
   gate that unblocks both save paths.
2. Choose the merge order for the in-flight `wrf-foundation` worktree: the branch holds F1
   + F2-doc-slice ahead of main and is held pending the owner committing in-progress
   dataset/volume (CR-CH-059) work. Decide whether CR-CH-059 RC.A lands on main first and
   `wrf-foundation` rebases, or `wrf-foundation` merges and RC.B continues from there.
   (The branch is already built on RC.A, so either order is coherent; this is purely a
   merge-sequencing call.)
3. Confirm RC.B.8 Part 2 stays DEFERRED until CR-CH-058 F1 is on main (recommended), vs.
   authorising an interim line-based mainframe packer now (NOT recommended -- throwaway,
   section D).

**Sequenced proposal:**

- **Step 0 (now, docs-only):** keep RC.B.8 Part 2 as the recorded DEFERRAL it already is.
  No code. Accept Part 1 (the repoint) as satisfying Req 28; Part 2 composes later.
- **Step 1 (gate, owner-confirmed framework change):** author ONE framework-change gate
  for the record-aware `BackendEnvironment` store contract (section E), shared by both
  CRs. This is the keystone and is owner-decision #1.
- **Step 2 (land CR-CH-058 foundation):** merge/settle the `wrf-foundation` F1 (record
  model + piece-table + byte-identical native SAVE) to main, then F2 shell slice, F3
  (background scan/open UX), F4 (windowed FIND/CHANGE), F5 (scalability guard). F1 is the
  hard dependency for RC.B.8; F2-F5 can proceed in parallel with Step 3's non-editor
  wiring because they do not touch the mainframe CE.
- **Step 3 (RC.B.8 non-editor wiring, parallelisable with Step 2 F2-F5):** build
  prerequisites (b) open named-backend registry + dispatch arm, (c)
  `open_mainframe_dsn` owning_env="MAINFRAME", (d) register the mainframe VFS provider
  live. These are shell/registry/provider edits independent of the piece table.
- **Step 4 (RC.B.8 editor join, needs Step 1 + Step 2-F1 + Step 3):** implement the
  mainframe CE in `ff-idcams` (prereq e) implementing the Step 1 record-aware contract
  over `DatasetAccess::put`; have CR-CH-058's SAVE walk call the record form when the
  owning CE advertises a Fixed/Variable `RecordFormat`; lift the editor-nav gate that
  refuses mainframe editing. Now a MAINFRAME-owned editable tab exists and the first-fail
  SAVE test (`DatasetAccess::put` driven) becomes writable.

**What is parallel:** Step 2 (F2-F5) and Step 3 can run concurrently once Step 1's
contract shape is agreed; they touch different code (document-model/viewport vs
shell registry/provider). The one serialisation point is the shell SAVE path
(`save_active_tab_via_backend` / `host_fs_save`): CR-CH-058 reshapes HOW bytes/records
are produced; RC.B.8 reshapes WHERE they are addressed. The F2-doc-slice note already
shows the teams coordinating on `shell/render_*.rs`; the same discipline applies to the
SAVE seam -- land CR-CH-058's SAVE-walk refactor first, then RC.B.8 adds the mainframe
record form through the Step 1 contract.

**Where the worktrees fit:** `.worktrees/wrf-foundation` is CR-CH-058 (keep; merge per
owner decision #2). `.worktrees/vsam-wiring` (branch `feature/vsam-service-wiring`) is
ON HOLD -- REDIRECT per the CR-CH-059 change-log resolution (its `ff-vsam-services` work
is accepted as throwaway; VSAM re-wires under `DatasetAccess` at RC.B.7). RC.B.8 Part 2's
non-editor wiring (Step 3) belongs with the RC.B dataset stream (same stream as the
`DatasetAccess`/mainframe CE work), NOT the editor worktree.

**Wasted effort to avoid if built independently:**
- Reshaping the `BackendEnvironment` SAVE seam twice (byte-reshape for CR-CH-058, then
  record-reshape for mainframe). Design (a) once.
- Writing a separate mainframe "records -> store" packer that duplicates CR-CH-058's
  piece-list re-framing SAVE walk. Let the walk produce the records; the CE just maps to
  `ff_dscatalog::Record` + `DatasetAccess::put`.
- A throwaway interim line-based mainframe packer (section D) if CR-CH-058 F1 is close.
- Defining `RecordFormat`/RECFM twice: the CE supplies ONE format at open consumed by
  both the editor (open/index) and the save (re-frame) -- do not let the mainframe save
  re-derive framing independently of the editor's `RecordFormat`.

---

## Evidence index

Specs / change-log / design (docs):
- `docs/status/change-log.md` -- `CR-CH-058` entry (piece-table rework; "mainframe
  Fixed/Variable RecordFormat + record-aware save is a LATER V-stream gate built ON this
  foundation"); `CR-CH-059` entry (DatasetAccess contract, single StorageProvider seam,
  vsam-wiring REDIRECT resolution). CR-CH-099 NOT present (grep: no match).
- `docs/specs/document-model/requirements.md` -- Glossary `RecordFormat`/`Record`/
  `Immutable_Original_Index`/`Piece_List`/`Append_Buffer`/`Window_Band`/`Total_Records`;
  Req 11.1-11.6 (universal record + CE supplies format + 11.5 no-flatten/re-pad-to-LRECL +
  "mainframe save OUT OF SCOPE of this gate"); Req 12.x (piece-table + windowing + 12.12
  byte-identical native); Cross-References ("command-environments ... SAVE-addressing seam
  (CR-CH-053 Task 20/21)").
- `docs/specs/command-environments/requirements.md` -- Req 16 (ff-ce-* family; mainframe
  CE housed in `ff-idcams`, "built in a later phase"); Req 17 (live provider-registry
  prerequisite; non-host Req 14-16 depend on it; host path does not).
- `.agents/tasks/rcb8-dataset-rationalisation/part2-mainframe-save-stop.md` -- the
  deferral + prerequisites (a)-(e) + five code-grounded evidence points.
- `.agents/tasks/windowed-record-foundation/FOUNDATION-DESIGN.md` -- section 3
  (`RecordFormat` generalises `ff-dscatalog::Recfm`+LRECL, "REPLACES the provider's
  current flatten-to-`\n`"); section 6 SAVE ("re-framed per RecordFormat", "The CR-CH-053
  Task 20/21 SAVE-addressing-to-the-owning-CE path is the seam this rides (record-aware
  save lives in the owning CE)"); section 7 phasing ("Then the `V`-stream mainframe CE
  supplies `RecordFormat::Fixed/Variable` + record-aware SAVE on this foundation").
- `.agents/tasks/wrf-foundation/F2-doc-model-slice-note.md` -- F1 complete + F2 doc-slice
  (WindowBand) delivered, shell slice deferred to avoid colliding with CR-CH-059 render
  work; 201 scoped tests.

Code (symbols):
- `crates/ff-desktop/src/tab_manager.rs::save_active_tab_via_backend` -- reads
  `doc.contiguous_view().to_vec()` (flat bytes) and calls `backend.save(&path, &bytes)`.
- `crates/ff-desktop/src/shell/dispatch_ffedit.rs::host_fs_save` -- dirty-aware FFEDIT
  SAVE delegating to the owning host-FS backend's `save`.
- `crates/ff-vfs/src/backend_environment.rs` -- `fn save(&self, path, bytes)` (the
  byte-only contract; prereq (a) target).
- `crates/ff-desktop/src/tab_state.rs::DEFAULT_OWNING_ENVIRONMENT = "HOSTFS"`;
  `shell/actions.rs::active_owning_environment` / `shell_open_file_with_env` (owning_env
  binding).
- `crates/ff-desktop/src/shell/construct_provider.rs::build_live_provider_registry` --
  seeds only the host-FS `local` provider (prereq (d)).
- `crates/ff-dscatalog/src/dataset_access/trait_def.rs` -- `trait DatasetAccess`
  (`allocate/open/get/put/point/close/dispose`); `impl_io.rs::{codec_for, put_impl,
  get_impl}`; `vsam_service.rs::Record { key, data }`; `dataset.rs::Recfm {F,FB,V,VB,U}`;
  `codecs/{fixed,variable,binary,text}.rs` impl `RecordCodec`; tests
  `get_put_use_codec_boundaries_not_crlf` (Req 34.2), `put_growth_surfaces_x37_space_abend`
  (Req 34.4).

Git (read-only, via `git worktree list` / `git log` / `git rev-list`):
- Worktrees: `main` (`b9921f3`), `.worktrees/env-registry` (`feature/environment-registry`),
  `.worktrees/vsam-wiring` (`feature/vsam-service-wiring`), `.worktrees/wrf-foundation`
  (`feature/windowed-record-foundation`, HEAD `74a0414`).
- `wrf-foundation`: clean tree; 2 behind / 9 ahead of `main`; built on CR-CH-059 RC.A
  commit `216c3cd`; F1 merge to main explicitly held pending owner's in-progress
  dataset/volume work.
