# Gate Summary -- Mainframe Dataset Stack Rationalisation (CR-CH-059)

Status: DOCS-ONLY requirements-gate package, conformance-reviewed and APPROVED
(zero HIGH/MEDIUM/NIT findings). AWAITING OWNER APPROVAL before any code task
begins. Every touched spec is PENDING GATE / IN PROGRESS until the owner approves.

Workspace root: `c:\workspace\VSC\FileForgeWorkbench`.
Authoritative plan: `.agents/tasks/dataset-vision-fit/report.md` (sections C/D/E)
plus the retirement procedure in `.agents/tasks/dscatalog-duplicate/report.md`.
Conformance review: `.agents/tasks/dataset-rationalisation-gate/gate-review.md`.

---

## 1. Allocated CR id

- **CR-CH-059** -- single, unprefixed MAINLINE change-request id (next free after
  CR-CH-058; no collision). Used consistently across all spec Source lines,
  tasks, TCR headers, project-master, and the change-log.
- **No additional CR-NR was logged.** The new `DatasetAccess` access contract is
  a genuinely new capability, but it was DELIBERATELY kept inside this single
  CR-CH (not split into a separate CR-NR), per the conservative-single-change-
  request gate rule: it is the keystone of the SAME consolidation (reconciling
  the three drifted service traits plus the allocator's physical-path return into
  one handle-based interface), not an independent feature. Its criteria live in
  `dataset-catalog` Requirements 33-34.

## 2. Sub-projects touched -- new requirement numbers and criterion ranges

| Sub-project | New Requirement(s) | Criterion range | Scope |
|---|---|---|---|
| `dataset-ownership-model` | Req 22 | 22.1 - 22.7 | ADR-001 corrected to name `ff-dscatalog` as the catalog authority; `ff-dataset-catalog` + `ff-vsam-services` deprecated-for-merge; acyclic DAG with `ff-volume` owning Volume; ADR-002 (catalogs never own bytes) preserved; fitness-function update. |
| `virtual-file-system` | Req 13, Req 14 | 13.1 - 13.5, 14.1 - 14.4 | Single physical `ff-vfs::StorageProvider` seam (duplicate deleted); single `posix` registrant; record codecs stay in `ff-dscatalog`; hybrid storage unchanged. |
| `dataset-catalog` | Req 33, Req 34, Req 35 | 33.1 - 33.7, 34.1 - 34.8, 35.1 - 35.5 | Single authority + reconciled `CatalogService`/`VsamService`; the `DatasetAccess` contract; one `Dsorg`/`Recfm`; VSAM as a cluster entity; additive-first retirement sequence; the terminology map lives in this one design.md. |
| `dataset-allocator` | Req 19 | 19.1 - 19.6 | Retarget `catalog_bridge` to `ff-dscatalog`; allocation returns a `DatasetHandle` (not a raw `physical_path`); preserves CR-NR-105/CR-CH-057 Volume flow (Req 17-18). |
| `idcams-emulator` | Req 28 | 28.1 - 28.7 | Repoint `ff-idcams` private traits at the reconciled `ff-dscatalog` traits; DEFINE/REPRO/DELETE map onto `DatasetAccess`; composes with Req 27 (DEFINE VOLUME). |
| `jes-emulator` | Req 19 | 19.1 - 19.5 | Future JCL executor depends ONLY on `ff-dsalloc` + `DatasetAccess`; crate-name corrections (`ff-dataset-catalog` -> `ff-dscatalog`, `ff-dataset-allocator` -> `ff-dsalloc`); JES stays DEFERRED (no new build obligation). |
| `volume-model` | Req 12 | 12.1 - 12.4 | `DatasetAccess` resolves location via Dataset -> DatasetVolume -> Volume -> locator; schema v4 `storage_path` -> DatasetVolume locator (referenced, owned by CR-NR-105/CR-CH-057, not duplicated); SPACE charging preserved. |

In-place edits to pre-existing criteria are limited to crate-name corrections in
`jes-emulator` (Introduction, Req 1.7, Req 11.1/11.3/11.5/12.5) and the ADR-001
read-as remap in ownership Req 22.1 + its design delta; each carries a
"(Crate names corrected ... by CR-CH-059)" / change note.

## 3. Design decisions encoded

### The `DatasetAccess` trait (dataset-catalog Req 34 + design.md)

The single mainframe-faithful access contract a JES/JCL executor needs. Shape
(seven methods, all present and correctly scoped):

```
trait DatasetAccess {
    fn allocate(&self, dd: &DdRequest) -> Result<DatasetHandle, DatasetError>;      // DISP/SPACE/DCB/VOL=SER
    fn open(&self, h: &DatasetHandle, intent: AccessIntent) -> Result<OpenDataset, DatasetError>;
    fn get(&self, od: &mut OpenDataset) -> Result<Option<Record>, DatasetError>;     // RECFM-aware
    fn put(&self, od: &mut OpenDataset, rec: &Record) -> Result<(), DatasetError>;
    fn point(&self, od: &mut OpenDataset, key_or_rrn: &Positioner) -> Result<(), DatasetError>; // VSAM
    fn close(&self, od: OpenDataset) -> Result<(), DatasetError>;
    fn dispose(&self, h: DatasetHandle, outcome: StepOutcome) -> Result<(), DatasetError>; // KEEP/CATLG/...
}
```

Key properties encoded:
- **Owned by `ff-dscatalog`** (Req 34.1/34.6), the single catalog authority.
- Keyed by **DD + DISP + DSN with VOL=SER** (Req 34.1); the `DatasetHandle` is
  opaque -- consumers never parse its internals (Req 34.5).
- **RECFM-aware** get/put (fixed / variable-with-RDW / U / keyed); record
  boundaries come from the codecs, NEVER from a host text-line delimiter (34.2).
- **Resolves physical location through `ff-volume`** (Req 34.3; volume-model
  Req 12.1) and performs all physical I/O over the **single
  `ff-vfs::StorageProvider` seam** backed by the record codecs; no raw
  `storage_path` / SQLite leaks into the surface (34.3/34.7).
- It is the ONE record-I/O contract called by the JCL executor, the editor's
  future MAINFRAME `BackendEnvironment` SAVE, and IDCAMS (34.6).
- **Object-safe** (or an object-safe companion) for `dyn` + mocks (34.8).

### Terminology map (Hercules-informed; lives in dataset-catalog design.md)

- **DASD / Volume / VOLSER / VTOC** is the explicit physical-layer anchor (the
  `ff-volume` crate) -- the single biggest terminology gap being closed.
- **One `Dsorg {PS,PO,GDG}`** and **one `Recfm {F,FB,V,VB,U}`** (both
  `#[non_exhaustive]`, owned by `ff-dscatalog`); the divergent
  `ff-dataset-catalog` `Dsorg {Ps,Po,Da,Vsam}` / `Recfm {F,Fb,V,Vb,U}` set is NOT
  carried forward.
- **VSAM is a cluster entity (`VsamCluster`), NOT a `Dsorg` variant** -- the
  removal of the `Vsam`/`Da` `Dsorg` variants is deliberate.
- Seam vocabulary: **`StorageProvider` = physical**, **`VfsProvider` = interface
  / routing**; the duplicate catalog-local `StorageProvider` is deleted.

### Target dependency DAG (asserted acyclic)

`ff-idcams -> ff-dsalloc -> ff-dscatalog -> ff-volume -> storage providers`, with
`ff-vfs` as universal infrastructure. Catalogs hold metadata + locator only
(ADR-002 preserved).

## 4. Full ordered task list (RC.A / RC.B / RC.C)

### Phase RC.A -- cheap / safe-now docs + additive gated consolidation

- **RC.A.1** ADR-001 correction + crate-name doc fixes -- name `ff-dscatalog` as
  the catalog authority, mark `ff-dataset-catalog`/`ff-vsam-services`
  deprecated-for-merge, update the fitness function, fix the wrong crate names in
  jes-emulator. Delivers dataset-ownership-model Req 22 (task 13); jes-emulator
  Req 19 (task 35).
- **RC.A.2** Reconciled `CatalogService`/`VsamService` in `ff-dscatalog` + single
  `Dsorg`/`Recfm` + repoint `ff-governance-tests` -- additive; collapse the
  divergent enums; VSAM as a cluster entity. Delivers dataset-catalog Req 33
  (task 38).
- **RC.A.3** Unify onto `ff-vfs::StorageProvider` + delete the duplicate seam --
  the five backends implement `ff-vfs::StorageProvider`; delete
  `ff-dscatalog::storage::StorageProvider`. Delivers virtual-file-system Req 13 +
  dataset-catalog Req 35.1b (dataset-catalog task 39).
- **RC.A.4** Collapse the duplicate posix provider -- one `posix` registrant
  (`ff-vfs::PosixNativeProvider`). Delivers virtual-file-system Req 14 (task 17).

### Phase RC.B -- next, when PLUGIN phase 2 opens (gated features, in order)

- **RC.B.5** Land `ff-volume` + schema v4 + `storage_path` -> DatasetVolume
  dual-read migration -- prerequisite already gated under CR-CH-057
  (project-master Phase (volume-model) VM.2-VM.5). Referenced here, not
  re-authored.
- **RC.B.6** Define `DatasetAccess` in `ff-dscatalog` + retarget `ff-dsalloc` to
  return a handle -- the single access contract; the allocator returns a
  `DatasetHandle`, not a `physical_path`. Delivers dataset-catalog Req 34
  (task 40); dataset-allocator Req 19 (task 21); volume-model Req 12 (task 11).
- **RC.B.7** Wire VSAM under `DatasetAccess` then retire `ff-vsam-services` --
  concrete `VsamService` over the KSDS/ESDS/RRDS backends via `DatasetAccess`;
  remove `ff-vsam-services` + its member line. Delivers dataset-catalog Req 35.3
  (task 41). **(THIS is where the redirected `vsam-wiring` stream's work
  belongs.)**
- **RC.B.8** Repoint `ff-idcams` private traits + MAINFRAME record-aware SAVE
  path -- repoint `ff-idcams` at the reconciled traits; DEFINE/REPRO/DELETE map
  onto `DatasetAccess`; the editor's MAINFRAME `BackendEnvironment` SAVE routes
  record-aware writes through `DatasetAccess` (composes CR-CH-053 Task 20/21).
  Delivers idcams-emulator Req 28 (task 29). Then delete `ff-dataset-catalog`
  (dataset-catalog task 42).

### Phase RC.C -- defer / owner-blocked

- **RC.C.9** JES/JCL executor on `DatasetAccess` (ROADMAP PLUGIN phase 6) --
  contract/dependency criteria only; the executor depends ONLY on `ff-dsalloc` +
  `DatasetAccess`. Delivers jes-emulator Req 19 (task 35, docs/contract); build
  deferred.
- **RC.C.10** Volume UI after `ff-volume` + multivolume/extents/master-catalog --
  Volume management `WorkspaceContext` + picker (virtual-catalog-manager
  Req 17-18, already gated under CR-CH-057 VM.6); multivolume / extents /
  master-catalog + alias routing remain owner-blocked. Deferred.
- **RC.C.11** DO NOT build before consolidation (standing guard) -- do not wire
  VSAM against `ff-vsam-services`'s dead trait or
  `ff-dscatalog::storage::StorageProvider`; do not build JES / Volume UI against
  `storage_path` or `physical_path`; do not add a MAINFRAME CE that calls the
  catalog's SQLite path directly; do not extend `ff-idcams`'s private traits.
  Recorded in dataset-catalog design.md + Req 35.5.

### The "DO NOT build before consolidation" waste note

Captured in three places (dataset-catalog design.md, Req 35.5, project-master
RC.C.11): any VSAM wiring, JES executor, Volume UI, or MAINFRAME CE built on the
pre-consolidation surface (the dead `ff-vsam-services` trait, the duplicate
`ff-dscatalog::storage::StorageProvider`, raw `storage_path`/`physical_path`, or
`ff-idcams`'s private traits) will have to be REDONE after consolidation and must
not be started. The sequence is additive-first: the reconciled trait and unified
seam land (RC.A) before anything is deleted (RC.B/RC.C), so FFWB builds at every
step.

## 5. Project-master Phase rows and TCR count

- **project-master/tasks.md**: a new **Phase (dataset-stack-rationalisation)**
  block (rows RC.A.1-RC.A.4 / RC.B.5-RC.B.8 / RC.C.9-RC.C.11) with a Summary
  count row for CR-CH-059. The existing Phase (volume-model) Summary line was
  updated to reflect VM.1-VM.8 (the CR-CH-057 second gate). All gate-added task
  lines use `[ ]`; no pre-marked `[x]`.
- **TCR.md**: **58 NOT COVERED (red-circle) rows** added, one per new criterion,
  in the correct crate sections -- ownership 22.1-22.7 (7), vfs 13.1-13.5 /
  14.1-14.4 (9), dataset-catalog 33.1-33.7 / 34.1-34.8 / 35.1-35.5 (20),
  dataset-allocator 19.1-19.6 (6), idcams 28.1-28.7 (7), jes 19.1-19.5 (5),
  volume-model 12.1-12.4 (4).

## 6. EXPLICIT conflict flag -- the `vsam-wiring` worktree (OWNER DECISION)

An ACTIVE parallel stream is doing work this consolidation makes throwaway:

- **Stream prefix `V`**, worktree `.worktrees/vsam-wiring`, branch
  `feature/vsam-service-wiring` (registered in `docs/status/stream-prefixes.md`,
  row set to "ON HOLD -- REDIRECT").
- Its stated job: **wire `ff-vsam-services::VsamService` to the existing
  `ff-dscatalog` storage backends (KSDS/ESDS/RRDS)**.

The vision report (section E) states EXPLICITLY that this is work that MUST NOT be
done before consolidation: `ff-vsam-services` is slated to RETIRE, its trait is
disconnected from the real backends, and VSAM should be wired UNDER the ONE
`DatasetAccess` trait on the unified `ff-vfs::StorageProvider` seam -- sequenced
as Phase RC.B.7, after RC.A.2/RC.A.3 land the reconciled trait + unified seam.
Wiring VSAM against `ff-vsam-services` now will be REDONE after consolidation.

**RECOMMENDATION (owner to decide):** PAUSE / REDIRECT the `vsam-wiring` stream to
target the reconciled `ff-dscatalog` `VsamService` under `DatasetAccess` on the
single physical seam (as RC.B.7), OR explicitly accept its current output as
THROWAWAY. This conflict is flagged, not silently ignored; it is an OWNER
decision and MUST be resolved before the `V` stream continues.

## 7. Approval required

**OWNER APPROVAL is required before any code task begins.** No source outside
`docs/` has been or may be touched under this gate. All touched specs
(dataset-ownership-model, virtual-file-system, dataset-catalog, dataset-allocator,
idcams-emulator, jes-emulator, volume-model), the change-log CR-CH-059 entry, the
TCR rows, and the project-master Phase rows are **PENDING GATE / IN PROGRESS**
until the owner approves. The package is reviewed and APPROVED for conformance,
but it is NOT self-approved for implementation -- the gate PAUSES here for the
owner's go-ahead and for the owner's decision on the `vsam-wiring` conflict.
