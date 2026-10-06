# Authoring Plan: Volume Model + Dataset-Catalog Spec Changes

Status: PLAN ONLY. This file is the step-by-step instruction set for the
documentation-authoring agent. It writes NO deliverable docs itself.

Scope reminder (hard constraints for the author):
- DOCUMENTATION ONLY. NOTHING under `crates/` may be created or modified.
- ONLY files under `docs/` plus the ONE steering list file
  `.kiro/steering/specs.md` may be written.
- This passes through the project requirements gate (workflow.md). It is TWO
  gated items run together: a NEW REQUIREMENT (the new `volume-model`
  sub-project) and a CHANGE REQUEST (edits to the existing `dataset-catalog`
  spec, plus a governance note in `dataset-ownership-model`).
- At the end the author presents a summary and STOPS. No code implementation.
- The owner has ALREADY approved the direction (split, not rename) and
  scoping decisions 1-8 in the brief. The gate's step 7 CONFIRM is therefore a
  summary-and-proceed, not a fresh approval request, because approval is on
  record ("Approved, proceed.").

---

## 0. Source-of-truth inputs (read in this order before writing)

1. `.agents/tasks/volume-model-recommendation/recommendation.md` (the approved
   recommendation; sections 2, 4, 5.1, 5.2, 5.3, 6 are the backbone).
2. `docs/source-documents/dataset-catalog/FFWB_Storage_and_Catalog_Data_Model.md`
   (the project's own design doc). CITE this doc. The load-bearing citations:
   - ADR-001 (sec 23): volumes are first-class domain objects mapped to
     storage-provider URIs.
   - ADR-002 (sec 23): catalogs hold metadata references only, NEVER dataset
     bytes.
   - ADR-005 (sec 23): tracks/cylinders are EMULATED units via a geometry
     profile (not physical CKD).
   - sec 7.2 Volume, 7.3 Volume Group, 7.9 AllocationDefinition, 7.10
     DatasetVolume, 7.11 Extent, 7.12 PhysicalObject.
   - sec 14 Volume Capacity + `GeometryProfile { bytes_per_track,
     tracks_per_cylinder }`; sec 15 Multivolume; sec 18.2 Volume validation
     rules; sec 20 Error Model (`StorageError::{VolumeNotFound, VolumeOffline,
     VolumeReadOnly, InsufficientSpace, ...}`); sec 22 phases; sec 11.1 DEFINE
     VOLUME.

---

## 1. ID / NUMBER ALLOCATIONS (determined from the live docs -- use EXACTLY these)

| Allocation | Value | How determined |
|---|---|---|
| New-requirement CR id (volume-model) | **CR-NR-105** | Highest existing `### CR-NR-###` in `docs/status/change-log.md` is CR-NR-104; +1. |
| Change-request CR id (dataset-catalog + ownership note) | **CR-CH-057** | Highest existing `### CR-CH-###` is CR-CH-056; +1. |
| volume-model Requirements | **Requirement 1 .. Requirement 10** (new file) | New sub-project; numbering starts at 1. Criteria `1.1..`, `2.1..`, etc. |
| dataset-catalog next NEW Requirement | **Requirement 32** (criteria 32.1 ..) | Highest existing `### Requirement N` in `docs/specs/dataset-catalog/requirements.md` is Requirement 31 (criteria 31.1-31.9); +1. |
| dataset-catalog next task number | **Task 33** (33.1 ..) | Highest existing top-level task is `- [ ]/[x] 32.` (CR-NR-017 block); +1. Note tasks are NOT in strict numeric order in the file -- 32 is the max, so continue at 33. Append a new `## Tasks Added by CR-CH-057 -- Volume Model Integration` section. |
| dataset-ownership-model next Requirement | **Requirement 21** (criteria 21.1 ..) | Highest existing is Requirement 20; +1. |
| project-master new phase | **Phase (volume-model)** with rows **VM.1 .. VM.n** | project-master `tasks.md` groups by `### Phase (<label>)` with `- [ ] <PREFIX>.<n>` rows + a trailing Status/Count table. Latest phases use letter-prefix ids (e.g. NLS.*, L10N.*, TER.*). Use prefix `VM`. |
| volume-model entry in steering list | insert `volume-model` line | `.kiro/steering/specs.md` alphabetical list; insert AFTER `virtual-file-system` and BEFORE `whitespace-and-guides` (lines ~95-96). |

TCR rows: `docs/quality/TCR.md`. Two things to do:
- Add a NEW crate line in the Wave summary table near the existing
  `| `ff-dscatalog` | ... |` / `| `ff-dsalloc` | ... |` rows (around line 137)
  for the planned `ff-volume` crate -- but since no tests exist yet, this is a
  per-criterion NOT COVERED block, see below.
- Add one NOT COVERED row PER NEW CRITERION in the detailed per-criterion
  `ff-dscatalog` section (the big block starting ~line 1393). Use the EXACT
  row format already in that block:
  `| `<crate>` | 🔴 | -- | Req X.Y: <one-line description> |`
  NOTE: the steering `testing.md` NOT COVERED marker is the RED CIRCLE emoji
  (🔴). The TCR file ALSO contains non-standard 🟢/🟡 marks on some older rows;
  DO NOT copy those -- new rows are 🔴 only.

---

## 2. CHARACTER-SET + TASK-MARKER CHECKLIST (apply to EVERY file written)

From `.kiro/steering/documentation.md` and `specs.md`:

- [ ] Plain ASCII in all PROSE. Substitute: em-dash -> `--`; en-dash -> `-`;
      curly quotes -> straight `' '` / `" "`; ellipsis -> `...`; arrows -> `->`
      / `<->`; `<=` `>=` `!=`; logic words `AND`/`OR`/`NOT`; `for all`,
      `exists`, `in`.
- [ ] Box-drawing (U+2500-U+257F) ONLY inside fenced code blocks / ASCII-art
      diagrams, never in matchable prose. Markdown tables use `|` and `-` only.
- [ ] TCR status emoji (checkmark / cross / white square / RED circle) are
      permitted ONLY in `docs/quality/TCR.md` tables. New TCR rows use 🔴.
- [ ] No BOM; no stray a-circumflex artefacts.
- [ ] Task files: ONLY `[ ]` and `[x]` checkbox markers. NEVER `[~]`, `[-]`,
      `[/]`. (Note: project-master's key table documents `[~]` as a legend but
      do NOT introduce new `[~]` task lines -- author only `[ ]`.)
- [ ] EVERY task line has descriptive title text immediately after the number.
- [ ] All drafted tasks are `[ ]` only -- never pre-mark `[x]`.
- [ ] Status notes (BLOCKED etc.) go on an indented sub-bullet, never inside
      the brackets.
- [ ] EARS format for every acceptance criterion:
      `WHEN ... THE ... SHALL ...` (or `THE ... SHALL ...` for invariants);
      each criterion independently testable (testing.md).
- [ ] Run the enforcement greps from documentation.md over every new/edited
      file before declaring done:
      `rg "[^\x00-\x7F\u2500-\u257F]" <file>` (md) must be clean except TCR
      emoji; `.rs` is irrelevant here (no code touched).

---

## 3. FRAMEWORK-CONFORMANCE NOTES THE AUTHOR MUST BAKE IN

Per `framework-conformance.md`, the criteria MUST build ON existing seams, not
invent parallel mechanisms. Thread these through the volume-model design.md and
the dataset-catalog design delta:

- A Volume maps to a VFS StorageProvider URI (ADR-001). Volume is a storage
  locator target, NOT a new navigation stack or dispatch path.
- `DEFINE VOLUME`, volume listing/VTOC, mount/vary, and VOL=SER allocation are
  COMMANDS resolved through the single command-dispatch path (CommandTarget /
  the existing `handle_command` ladder until the verb table lands). Note the
  wiring-standard caveat: register verbs on the existing ladder, do not create
  a second dispatcher.
- Any Volume Workspace/VTOC listing UI (if specified) is a `WorkspaceContext`
  dispatched via `render_workspace_context` returning `InteriorFocus` -- NOT a
  bespoke panel. Keep this as a design note; the UI criteria live mostly in
  virtual-catalog-manager (out of THIS gate's write scope -- see sec 8).
- Session persistence of a Volume view uses `WorkspaceDescriptor`
  (`CustomWorkspace { kind, params }`), not a new persistence format.
- Crate ownership (owner decision 5): the Volume entity lives in a NEW thin
  crate `ff-volume` that `ff-dscatalog` depends on. State this ownership in
  volume-model/design.md AND the dataset-ownership-model Requirement 21 note.
  DO NOT create the crate. This keeps the dependency DAG acyclic and ADR-002
  enforceable (catalog depends on volume; volume never depends on catalog).

---

## 4. DELIVERABLE A -- NEW sub-project `docs/specs/volume-model/` (CR-NR-105)

Create the folder and THREE files. This is the NEW REQUIREMENT half of the gate.

### 4.1 `docs/specs/volume-model/requirements.md`

Structure: `# Requirements Document` -> `## Introduction` -> `## Glossary` ->
`## Requirements` (Requirement 1..10) -> `## Non-Functional Requirements`.

Introduction MUST: state this owns the first-class Volume layer that the split
(not rename) introduces; cite `FFWB_Storage_and_Catalog_Data_Model.md`
ADR-001/ADR-002/ADR-005; name the sibling specs it coordinates with
(dataset-catalog, dataset-allocator, idcams-emulator, virtual-catalog-manager,
dataset-ownership-model); state that tracks/cylinders/extents are METADATA ONLY
(no physical disk geometry simulated).

Glossary terms to define (ASCII prose): Volume, VOLSER, Volume_Status
(Online/Offline), Access_Mode (ReadWrite/ReadOnly), Volume_Group,
Geometry_Profile, Track, Cylinder, Extent, Primary_Extent, Secondary_Extent,
Max_Extents, Space_Abend (x37-style), VTOC_View, DatasetVolume association,
Uncataloged_Dataset, VOL=SER + UNIT direct access, Multivolume_Dataset.

Author Requirements 1-10 as follows (each `#### Acceptance Criteria` numbered
`N.1`, `N.2`, ...). The "Traces to" column tells the author which owner
decision (1-8) and/or source ADR each requirement anchors to -- put a
`**Source:**` line under each requirement pointing at the ADR/section and the
owner decision number.

| Req | Title | Criteria to write (summaries -- author in EARS) | Traces to |
|---|---|---|---|
| 1 | Volume Entity and VOLSER Identity | Volume has volume_id, VOLSER (unique within storage system), display name, storage_uri (the promoted Repository root), status, access_mode, capacity counters. VOLSER uniqueness enforced. A Volume owns the physical `storage/ pds/ gdg/ temp/` (or objects/) layout. | rec 4, 5.1; ADR-001; src 7.2, 18.2 |
| 2 | Volume Status and Access Mode | WHEN a Volume is Offline THE system SHALL reject new allocation; WHEN ReadOnly THE system SHALL reject write/delete/extend; mount/unmount + set-online/set-offline transitions; online check during resolution. | rec 5.1; src 7.2, 9 step 8, 10.1 |
| 3 | Emulated Geometry (Tracks, Cylinders, Byte Conversions) | Geometry_Profile with configurable bytes_per_track (default a 3390-style value) and tracks_per_cylinder = 15; deterministic, documented byte<->track<->cylinder conversions; conversions are ACCOUNTING ONLY, no physical layout. | decision 2; ADR-005; src 14 |
| 4 | Space Allocation Units and JCL SPACE Syntax | Support SPACE in TRK, CYL, and block/avg-record units: `SPACE=(TRK,(primary,secondary))`, `SPACE=(CYL,(primary,secondary))`, `SPACE=(avgreclen,(primary,secondary))` with `AVGREC`; map each to allocation units via the geometry profile. | decision 2; src 7.9 AllocationUnit |
| 5 | Dataset Space Capacity Constraint (x37-style failure) | A dataset has allocated space = primary extent + up to max secondary extents; WHEN a write/append would exceed the dataset's allocatable capacity OR exceed max-extent count THE system SHALL FAIL with a clean reported space-abend-style error (B37/D37/E37 analogue), NOT a crash. | decisions 1a, 3; src 20 InsufficientSpace |
| 6 | Secondary Extents and Max-Extent Limit | Primary extent + up to a CONFIGURABLE max secondary extents (default 16); running out of secondary extents OR exceeding max-extent count is a space-abend-style failure; separate, distinct from volume-full. | decision 3; src 7.11, 7.9 max_extents |
| 7 | Volume Capacity and Volume-Full Failure | A Volume has its own total capacity in tracks/cylinders; WHEN a Volume has insufficient free space for a requested allocation THE system SHALL FAIL with a volume-full error that is DISTINCT from the dataset x37 failure (two separate reported failure points). | decision 4; src 14, 20 |
| 8 | Reporting -- Tracks/Cylinders/Extents In Use and Remaining | THE system SHALL report, for a dataset: extents used + remaining, tracks + cylinders in use; for a Volume: total/used/free tracks + cylinders and allocated/available space. (VTOC-style volume listing is the surface.) | decision 1b; src 14 |
| 9 | Multivolume + Uncataloged + Cardinality | Model permits: a dataset residing on one OR more Volumes (DatasetVolume sequence); many catalogs registering datasets on one shared Volume; an UNCATALOGED dataset on a Volume with NO catalog entry, resolvable by explicit VOL=SER + UNIT. Break the 1:1 catalog<->repository binding at the model level (UI MAY default to one-volume-per-catalog). | decisions 6, 7; rec 3.3, 5.2; src 5, 7.10, 15 |
| 10 | DEFINE VOLUME + Volume Visibility (advanced concept) | `DEFINE VOLUME` FFWB-extension command registers a host directory as an emulated Volume (name, path, capacity, status) through the single command path; Volume is a VISIBLE-but-ADVANCED concept -- surfaced during catalog/dataset creation and in a Volume report/VTOC listing, not hidden. | decision 8; rec 6; src 11.1 |

Non-Functional section: deterministic geometry conversions (same inputs ->
same units every run); reporting counters are derived metadata, O(1) or O(n
extents); no dataset bytes move when a Volume is defined over an existing
Repository (migration is metadata-only, ADR-003).

### 4.2 `docs/specs/volume-model/design.md`

Sections to write:
- Introduction / scope.
- Entity model: Volume, Geometry_Profile, Extent (logical), DatasetVolume
  association, VTOC_View (derived, not necessarily a stored table initially).
  Reproduce the ER view from recommendation.md section 5.2 inside a fenced code
  block (box-drawing allowed there).
- Crate ownership decision (owner decision 5): NEW thin crate `ff-volume`;
  `ff-dscatalog` depends on `ff-volume`; `ff-volume` depends only on narrow
  model crates, NEVER on `ff-dscatalog` (keeps DAG acyclic, enforces ADR-002).
  State: "DO NOT create the crate in this gate; owner confirms again at
  implementation."
- Geometry convention: default bytes_per_track value, cylinder = 15 tracks,
  the conversion formulae; worked example (bytes -> tracks -> cylinders).
- Failure model: TWO distinct failure points (dataset x37 vs volume-full),
  mapped onto `StorageError` variants from src sec 20.
- Framework-conformance statement (sec 3 above): VFS StorageProvider URI
  mapping, single command dispatch for DEFINE VOLUME / listing, WorkspaceContext
  for any VTOC UI, WorkspaceDescriptor persistence. Explicitly: no new dispatch
  path, no second navigation stack, no new persistence format.
- Migration approach (from recommendation.md 5.3): schema v4 adds `volumes` +
  `dataset_volumes`; existing Repository becomes a Volume whose storage_uri is
  the current root; dual-read `storage_path` during transition; extents/
  multivolume phased later (src sec 22 Phase 1 vs Phase 4).
- A "Deferred / later phase" note: full extent-object mapping and multivolume
  spanning are Phase 4; phase-1 Volume is a named directory + VOLSER + status +
  capacity counters.

### 4.3 `docs/specs/volume-model/tasks.md`

`# Implementation Plan: Volume Model (ff-volume)` -> `## Overview` ->
`## Tasks`. Author `[ ]`-only tasks, numbered from 1, each cross-referencing
the volume-model criteria it satisfies (`Validates: Requirement X.Y`). Suggested
groupings (author may refine, keep each independently completable):
1. `ff-volume` crate scaffold + Volume entity + VOLSER validation (Req 1, 2).
2. Geometry_Profile + byte/track/cylinder conversions (Req 3).
3. SPACE syntax units parsing hand-off model (Req 4) -- note the JCL SPACE
   PARSE itself is ff-dsalloc's; this task is the unit model `ff-volume`
   exposes.
4. Extent model + max-extents + dataset capacity constraint failure (Req 5, 6).
5. Volume capacity + volume-full failure (Req 7).
6. Reporting counters for dataset + volume (Req 8).
7. DatasetVolume association + multivolume + uncataloged resolution model
   (Req 9).
8. DEFINE VOLUME command + volume listing/VTOC surface (Req 10) -- note command
   WIRING lands in ff-desktop/ff-dscatalog at implementation; here it is the
   ff-volume service API + the command contract.
Add an `## Acceptance Criteria Coverage` table mapping each Requirement.criterion
to the task(s), mirroring the dataset-catalog tasks.md style.

---

## 5. DELIVERABLE B -- EDIT `docs/specs/dataset-catalog/` (CR-CH-057)

This is the CHANGE REQUEST half. THREE files touched.

### 5.1 `docs/specs/dataset-catalog/requirements.md` -- edits + Requirement 32

EDIT IN PLACE (note each change inline, do not silently rewrite):

1. Glossary: REDEFINE `Catalog`, `Catalog_Database`, `Repository`,
   `Dataset_Resolution` to the split model. New wording:
   - Catalog: a metadata locator that maps a Dataset_Name to a Dataset entry
     and its Volume(s) + opaque locator. It does NOT physically contain dataset
     bytes (ADR-002).
   - Repository: the physical directory layout (`storage/ pds/ gdg/ temp/` or
     the UUID `datasets/objects/` layout) that is now OWNED BY A VOLUME, not by
     the Catalog.
   - Dataset_Resolution: look up a DSN -> Dataset entry -> DatasetVolume
     sequence -> Volume -> locator; confirm required Volumes are online.
   - Add glossary terms: Volume, VOLSER, DatasetVolume, Uncataloged_Dataset.
   Add a one-line note at each edited definition: "(Amended by CR-CH-057 --
   Volume split; was: <old gist>.)"
2. Requirement 1 (SQLite Catalog Database): add a note that schema gains
   `volumes` and `dataset_volumes` tables at schema v4 and that `storage_path`
   is superseded by the DatasetVolume (volume_id, locator) pair; do NOT delete
   the existing criteria -- add the superseding note like Requirement 4 already
   does for Requirement 20. (Edit-in-place note, not a renumber.)
3. Requirement 7 (Dataset Create/Delete/Rename/Allocate): add a criterion-level
   note (or a sub-note under the existing Ownership Clarification block) that
   allocation now records the target Volume via DatasetVolume and that physical
   storage belongs to the resolved Volume. Do NOT renumber existing criteria.

ADD a brand-new `### Requirement 32: Volume Binding and Catalog-as-Locator`
with criteria 32.1 .. (EARS). Criteria to write and their traces:

| Crit | Statement (author in EARS) | Traces to |
|---|---|---|
| 32.1 | THE catalog SHALL store a `volumes` table (volume_id, volser UNIQUE, storage_uri, status, access_mode, capacity counters) at schema v4. | decision 5/6; ADR-001; rec 5.3 |
| 32.2 | THE catalog SHALL store a `dataset_volumes` table (dataset_id, volume_id, sequence_number, is_primary, locator) replacing the dataset `storage_path` as the authoritative location. | decision 6; src 7.10; rec 5.3 |
| 32.3 | WHEN resolving a DSN THE catalog SHALL go Dataset -> DatasetVolume (by sequence) -> Volume -> locator, and SHALL verify each required Volume is Online. | ADR-002; src 9 |
| 32.4 | THE catalog SHALL NOT physically contain dataset bytes -- the bytes belong to the Volume's storage (ADR-002); the catalog persists metadata + locators only. | ADR-002 |
| 32.5 | THE schema v4 migration SHALL, for each existing dataset, create a Volume over the current Repository root and insert a dataset_volumes row with locator = existing storage_path, preserving `storage_path` for dual-read during transition; NO dataset bytes SHALL move. | ADR-003; rec 5.3 |
| 32.6 | THE catalog SHALL permit many catalogs to register datasets on one shared Volume AND one dataset to reside on multiple Volumes (multivolume) at the schema level. | decision 6; src 15 |
| 32.7 | THE catalog SHALL support an UNCATALOGED dataset on a Volume with no catalog entry, resolvable by explicit VOL=SER + UNIT (a resolution path that does not require a catalog row). | decision 7; rec 3.3 |
| 32.8 | THE catalog's Volume entity SHALL be owned by the `ff-volume` crate on which `ff-dscatalog` depends; the catalog SHALL NOT redefine the Volume type. | decision 5; ownership Req 21 |

Add a `**Source:**` line to Requirement 32 citing
`FFWB_Storage_and_Catalog_Data_Model.md` ADR-001/ADR-002 + CR-CH-057.

### 5.2 `docs/specs/dataset-catalog/design.md` -- append CR-CH-057 section

Read first. Append a section `## Volume Split (CR-CH-057)` covering: catalog
becomes a pure locator; `volumes` + `dataset_volumes` schema v4 tables and
columns; resolution path change (DatasetVolume indirection); dual-read migration
of `storage_path`; `ff-volume` crate dependency (catalog depends on volume;
never the reverse); uncataloged (VOL=SER+UNIT) resolution path; framework seams
(VFS StorageProvider URI per Volume, single command dispatch). If design.md has
no contradicting prior decision, state so; never silently contradict an existing
decision -- if one is touched, call it out.

### 5.3 `docs/specs/dataset-catalog/tasks.md` -- append CR-CH-057 tasks (from 33)

Append `## Tasks Added by CR-CH-057 -- Volume Model Integration` then `[ ]`
tasks numbered from **33**. Each cross-references Requirement 32 criteria and/or
the glossary/Req 1/Req 7 edits. Suggested tasks (author refines; keep each
independently completable, `[ ]` only):
- 33. Schema v4 -- add `volumes` + `dataset_volumes` tables and the forward
  migration (Validates: Req 32.1, 32.2, 32.5).
- 34. Resolution via DatasetVolume indirection + Volume online check
  (Validates: Req 32.3, 32.4).
- 35. Depend on `ff-volume`; replace internal Volume notions with the ff-volume
  type (Validates: Req 32.8).
- 36. Multivolume + shared-volume + uncataloged (VOL=SER+UNIT) resolution path
  (Validates: Req 32.6, 32.7).
- 37. Tests: migration dual-read, resolution via volume, uncataloged resolve,
  shared-volume registration (Validates: Req 32.1-32.7).
Add rows to the file's existing `## Acceptance Criteria Coverage` table (or a
new CR-CH-057 coverage sub-table) mapping Req 32.1-32.8 to the tasks above.

---

## 6. DELIVERABLE C -- governance note in `docs/specs/dataset-ownership-model/`

Per owner decision 5 and recommendation.md section 6, the ownership model needs
an ADR-amendment-style note (the file itself defines this process in Req 1.4 and
Req 20).

### 6.1 `docs/specs/dataset-ownership-model/requirements.md` -- add Requirement 21

ADD `### Requirement 21: ff-volume Ownership Boundary (CR-CH-057)` with criteria
21.1 .. (EARS). Content:
- 21.1 THE `ff-volume` crate SHALL own the Volume entity, VOLSER identity,
  volume status/access-mode, geometry profile, extent accounting, volume
  capacity counters, and the DatasetVolume association model.
- 21.2 THE `ff-dataset-catalog` (ff-dscatalog) crate SHALL depend on `ff-volume`
  and SHALL obtain Volume data through its API; it SHALL NOT redefine the
  Volume type.
- 21.3 THE `ff-volume` crate SHALL NOT depend on `ff-dataset-catalog`,
  `ff-dataset-allocator`, or `ff-idcams` (acyclic DAG; ADR-002 enforceable).
- 21.4 THE catalog SHALL NOT physically contain dataset bytes; physical storage
  belongs to the Volume (restates ADR-002 at the ownership layer).
- 21.5 Uncataloged datasets (VOL=SER + UNIT) are owned by the Volume layer for
  physical existence; the catalog layer owns only cataloged resolution.
Add `**Source:** ADR-001 amendment via CR-CH-057` and cite
`FFWB_Storage_and_Catalog_Data_Model.md` ADR-001/ADR-002.
ALSO edit Requirement 7 (Dependency Direction Enforcement) in place: add
`ff-volume` to the permitted dependency chain as the layer `ff-dataset-catalog`
depends on (e.g. `... -> ff-dataset-catalog -> ff-volume -> storage providers`),
with an inline "(Added by CR-CH-057)" note. Do NOT renumber Req 7 criteria;
append a new criterion (7.7) or extend 7.1's wording with the note -- prefer a
new criterion 7.7 to keep existing criteria stable.

### 6.2 `docs/specs/dataset-ownership-model/design.md`

Read first. If it carries dependency diagrams, add `ff-volume` to them with a
CR-CH-057 note; otherwise append a short "No structural change beyond the
ff-volume layer added by CR-CH-057" note. (tasks.md for ownership-model is a
governance doc -- only add a task if the file's convention expects one; if it
has a tasks.md with real tasks, add a single `[ ]` doc-alignment task, else
leave it.)

---

## 7. CROSS-CUTTING REGISTRATION FILES (gate steps 1, 5, 6 + change-log)

### 7.1 `docs/status/change-log.md` -- TWO entries

Append under `## New Requirements`:
```
### CR-NR-105 -- New volume-model sub-project: first-class mainframe Volume layer
- **Date/Phase**: Phase (volume-model) -- PENDING GATE
- **Prompt**: "<first 80 chars of the owner's message 1/2>"
- **Description**: <1-2 sentences: new volume-model spec owning the first-class Volume (VOLSER, status, capacity, emulated geometry, extents-as-metadata, x37 + volume-full failures, DEFINE VOLUME). Split not rename. ff-volume crate.>
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/volume-model/requirements.md`
```
Append under `## Change Requests`:
```
### CR-CH-057 -- Dataset-catalog becomes a pure metadata locator; Volume owns physical storage
- **Date/Phase**: Phase (volume-model) -- PENDING GATE
- **Prompt**: "<first 80 chars>"
- **Description**: <1-2 sentences: split the 1:1 catalog<->repository binding; add volumes + dataset_volumes schema v4; catalog resolves DSN -> Volume + locator; ownership-model Req 21 adds ff-volume.>
- **Affects**: dataset-catalog, dataset-ownership-model (and design-only cross-refs to dataset-allocator, idcams-emulator, virtual-catalog-manager)
- **Status**: PENDING GATE
```
(Status may be set to IN PROGRESS when the author starts writing, per the gate
convention, since the owner already approved.)

### 7.2 `.kiro/steering/specs.md` -- register the sub-project

Insert a new list line `- volume-model (CR-NR-105: first-class Volume layer --
VOLSER/status/capacity/emulated geometry/extents-as-metadata; split not rename;
new ff-volume crate; gate authored)` alphabetically BETWEEN
`- virtual-file-system` (line ~95) and `- whitespace-and-guides` (line ~96).
This is the ONE steering file writable in this task.

### 7.3 `docs/project-management/project-master/tasks.md` -- new phase

Append a `### Phase (volume-model) -- CR-NR-105 + CR-CH-057 ...` section with a
short `>` blockquote summary, then `- [ ] VM.1 ...` rows, then the trailing
`| Status | Count |` table row, matching the format of the latest phases
(Phase (localization), Phase (theme-egui-rework)). Suggested rows:
- `- [ ] VM.1 Requirements gate -- volume-model requirements/design/tasks,
  dataset-catalog Req 32 + edits, dataset-ownership-model Req 21 + Req 7 note,
  this master phase, TCR NOT COVERED rows, change-log CR-NR-105/CR-CH-057,
  specs.md registration. (Authored; owner already approved direction.)`
- `- [ ] VM.2 ff-volume crate -- Volume entity, VOLSER, status/access-mode,
  geometry profile, byte/track/cylinder conversions. Delivers volume-model
  Req 1-3.`
- `- [ ] VM.3 Space + extents + failures -- SPACE unit model, extents + max-
  extents, dataset x37 constraint, volume-full. Delivers Req 4-7.`
- `- [ ] VM.4 Reporting + DatasetVolume + uncataloged + DEFINE VOLUME. Delivers
  Req 8-10.`
- `- [ ] VM.5 dataset-catalog integration -- schema v4 volumes/dataset_volumes,
  resolution indirection, dual-read migration, depend on ff-volume. Delivers
  dataset-catalog Req 32.`
Keep all rows `[ ]`. Update any workspace Summary counts the file maintains.

### 7.4 `docs/quality/TCR.md` -- NOT COVERED rows (one per new criterion)

In the DETAILED per-criterion area (the block that already lists
`| `ff-dscatalog` | ... | Req N.Y: ... |` rows ~line 1393+), append NOT COVERED
rows. Use crate `ff-volume` for the volume-model criteria and `ff-dscatalog`
for the dataset-catalog Req 32 criteria. One row per criterion:
```
| `ff-volume` | 🔴 | -- | Req 1.1: <desc> |
...through all volume-model criteria 1.1..10.n...
| `ff-dscatalog` | 🔴 | -- | Req 32.1: <desc> |
...through 32.8...
```
Also add `ff-dataset-catalog` (ownership-model) Req 21.1-21.5 rows if the TCR
tracks ownership-model criteria (check for an existing ownership-model section;
if none, add them under a clearly labelled ownership-model note using 🔴). Use
ONLY the 🔴 marker for new rows (not 🟢/🟡).

---

## 8. OUT OF SCOPE FOR THIS GATE (note, do NOT write)

Recommendation.md section 6 lists other specs that WILL change
(virtual-catalog-manager, dataset-allocator, idcams-emulator,
virtual-file-system, jes-emulator, jcl-resolver). This gate authors ONLY
volume-model + dataset-catalog + the dataset-ownership-model governance note.
In volume-model/design.md, add a short "Coordinates with (future gates)" note
naming those specs and the dependency, but do NOT edit their requirements.md in
this task. If the author believes a cross-spec edit is unavoidable to keep
volume-model non-contradictory, STOP and surface it (do not expand scope
silently) -- but the expectation is these stay future gates.

---

## 9. GATE SEQUENCE THE AUTHOR FOLLOWS (workflow.md section 2/3)

1. IDENTIFY: volume-model (new) + dataset-catalog + dataset-ownership-model.
2. REQUIREMENTS: write volume-model/requirements.md; edit
   dataset-catalog/requirements.md (Req 32 + glossary/Req1/Req7 notes);
   add dataset-ownership-model Req 21 + Req 7 note.
3. DESIGN: volume-model/design.md; dataset-catalog design CR-CH-057 section;
   ownership-model design note.
4. TASKS: volume-model/tasks.md; dataset-catalog tasks from 33.
5. MASTER: project-master Phase (volume-model) VM.1-VM.5.
6. TCR: NOT COVERED rows per new criterion.
7. CONFIRM: summarise all doc changes + ids/numbers; owner approval is already
   on record ("Approved, proceed."), so present the summary and proceed to
   finalise the docs (do NOT write any source).
8. CODE: NONE. This task ends after the docs + summary. STOP.

Also update change-log.md (CR-NR-105 + CR-CH-057) and specs.md registration as
part of step 2/5. Run the documentation.md ASCII grep over every written file
before declaring done.

---

## 10. FINAL SUMMARY THE AUTHOR MUST PRODUCE

State: files created/edited; the id/number allocations actually used
(CR-NR-105, CR-CH-057, volume-model Req 1-10, dataset-catalog Req 32 /
criteria 32.1-32.8 / tasks from 33, dataset-ownership-model Req 21 + Req 7.7,
project-master VM.1-VM.5, TCR 🔴 rows added, specs.md line added); confirm
plain-ASCII + task-marker checks passed; confirm NOTHING under crates/ was
touched; then STOP.
