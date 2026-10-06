# RESUME -- Volume Model (CR-NR-105 / CR-CH-057)

Saved: 2026-10-04. Pick up here next session.

## One-line status

Requirements gate for the first-class Volume layer is AUTHORED and the direction
is owner-APPROVED. No code written yet. Next real work is a SECOND gate for the
Volume/catalog UI (virtual-catalog-manager + dataset-allocator), then the code
build (project-master phase VM.2 onward).

## What this work is

Owner-approved "split, not rename": today a FFWB "catalog" is a SQLite DB bound
1:1 to a physical Repository directory that BOTH locates AND physically stores
datasets -- conflating the mainframe Volume (physical container, VOLSER) and
Catalog (name->location locator) roles. Fix: promote the Repository into a
first-class Volume (new thin `ff-volume` crate), reduce the Catalog to a pure
DSN -> (Volume, locator) metadata locator. Realigns code/specs with the project's
own source doc `docs/source-documents/dataset-catalog/FFWB_Storage_and_Catalog_Data_Model.md`
(ADR-001 volumes first-class; ADR-002 catalogs never own bytes; ADR-005 tracks/
cylinders are emulated units).

## Owner decisions already settled (do NOT re-litigate)

1. Tracks / cylinders / extents are METADATA ONLY -- not physical layout. Two
   uses: (a) capacity CONSTRAINT -- a dataset growing past its allocated capacity
   / max extents fails with a reported x37-style space abend (B37/D37/E37), never
   a crash; (b) REPORTING -- tracks/cylinders/extents in use + remaining.
2. SPACE in TRK, CYL, and block/avg-record forms; deterministic geometry profile,
   configurable bytes_per_track (3390-style default), cylinder = 15 tracks.
3. Primary + up to configurable max secondary extents (default 16); exhaustion =
   x37-style failure.
4. Volume has its own capacity; Volume_Full is a DISTINCT, separately reported
   failure from the dataset x37 failure.
5. New thin `ff-volume` crate owns the Volume entity; `ff-dscatalog` depends on it,
   never the reverse (makes ADR-002 enforceable by construction). Crate NOT created
   yet -- confirm again at implementation.
6. Break the 1:1 catalog<->volume binding in the schema: many catalogs per volume,
   many volumes per dataset, uncataloged datasets all permitted. UI MAY default to
   one-volume-per-catalog initially.
7. Uncataloged datasets supported (exist on a volume, resolved by VOL=SER + UNIT).
8. Volume is VISIBLE but ADVANCED in the UI -- surfaced at catalog/dataset creation
   and in a VTOC/volume report, not hidden.

## Deliverables ALREADY authored (gate complete, all docs-only, nothing in crates/)

- NEW `docs/specs/volume-model/requirements.md` -- Req 1-11 (EARS), glossary, NFRs.
- NEW `docs/specs/volume-model/design.md` -- entity model + ER diagram, geometry/
  extent accounting algorithm, two failure points, ff-volume ownership, VFS/command
  conformance, dual-read migration.
- NEW `docs/specs/volume-model/tasks.md` -- Tasks 1-10 (all [ ]).
- EDITED `docs/specs/dataset-catalog/requirements.md` -- glossary + Req 1/Req 7
  superseding notes; NEW Requirement 32 (32.1-32.8).
- EDITED `docs/specs/dataset-catalog/design.md` -- "Volume Split (CR-CH-057)"
  section (schema v4, resolution indirection, dual-read migration, ff-volume dep).
- EDITED `docs/specs/dataset-catalog/tasks.md` -- NEW tasks 33-37 + coverage table.
- EDITED `docs/specs/dataset-ownership-model/` -- Req 21 (ff-volume ownership) +
  Req 7.7 (dependency chain), design DAG, task 12.
- `docs/status/change-log.md` -- CR-NR-105 + CR-CH-057, both IN PROGRESS.
- `docs/project-management/project-master/tasks.md` -- new "Phase (volume-model)"
  with VM.1-VM.5 + status/count row.
- `docs/quality/TCR.md` -- NOT COVERED (red-circle) rows for every new criterion.
- `.kiro/steering/specs.md` -- volume-model registered alphabetically.

Prior artifacts (context):
- `.agents/tasks/volume-model-recommendation/recommendation.md` -- the original
  split-not-rename analysis.
- `.agents/tasks/volume-model-recommendation/authoring-plan.md` -- the id/number
  allocation plan.

## OPEN QUESTIONS raised but NOT yet answered (this is where we stopped)

The last conversation was about the Volume/catalog UI. These are NOT yet decided
and were deliberately left to a future gate (virtual-catalog-manager was marked
out of scope in the volume-model gate):

A. Dedicated Volume dialog/context vs one shared dialog with catalog creation?
   Kiro RECOMMENDED: a dedicated Volume management context (a WorkspaceContext
   that lists volumes = the VTOC/volume report, and hosts DEFINE VOLUME / vary
   online-offline / set RW-RO), with the catalog-creation dialog getting a VOLUME
   PICKER (pick existing OR "Define new volume..." that returns), NOT an inline
   volume editor. Single-user default can auto-select/create one volume so casual
   users never see it. OWNER HAS NOT CONFIRMED.

B. How volume allocation happens -- Kiro clarified there are TWO allocations:
   - Defining a VOLUME = `DEFINE VOLUME` admin act: name/VOLSER + host path
     (becomes storage_uri) + capacity (tracks/cyls) + status; creates the
     storage/ pds/ gdg/ temp/ layout; VOLSER must be unique. (volume-model Req 10)
   - Allocating SPACE ON a volume = automatic, driven by dataset SPACE= requests,
     charged against volume free capacity as extents. (volume-model Req 4-7)

C. Volume capacity hard cap vs elastic? Kiro RECOMMENDED: hard cap, fixed at
   DEFINE time, resizable later via an alter command, no over-commit (Volume_Full
   is a real failure). OWNER HAS NOT CONFIRMED.

## NEXT ACTIONS (in order)

1. Get owner answers to open questions A, B(confirm), and C.
2. Run a SECOND requirements gate (docs-only) for the UI/flow, scope:
   - `docs/specs/virtual-catalog-manager/` -- dedicated Volume management
     WorkspaceContext + catalog dialog volume-picker (per decision A).
   - `docs/specs/dataset-allocator/` -- SPACE-against-volume allocation flow,
     VOL=SER + UNIT uncataloged allocation.
   - Possibly `idcams-emulator` (DEFINE VOLUME / DEFINE CLUSTER VOLUMES()).
   Follow the gate in .kiro/steering/workflow.md; delegate via run_workflow
   workflowPrompt (same pattern as the volume-model gate). ASCII-only docs.
3. THEN start the code build via project-master phases:
   - VM.2 `ff-volume` crate (Volume entity, VOLSER uniqueness, status/access,
     geometry, byte/track/cyl conversions) -- volume-model Req 1-3.
   - VM.3 space + extents + both failures -- Req 4-7.
   - VM.4 reporting + DatasetVolume + uncataloged + DEFINE VOLUME -- Req 8-11.
   - VM.5 dataset-catalog integration (schema v4, resolution indirection,
     dual-read migration, depend on ff-volume) -- dataset-catalog Req 32.
   Before VM.2: confirm the default bytes_per_track constant (design uses ~56664
   in the worked example) and re-confirm the ff-volume crate per the ownership ADR.
   TDD per testing.md; Kiro runs SCOPED `-p ff-volume` / `-p ff-dscatalog` checks
   only, then hands off the full ffwb-gate.ps1 to the owner.

## Guardrails reminder

Code build is a SEPARATE step -- no code until an explicit TASK/IMPLEMENTATION
instruction. Everything above is docs-only and reversible. Delegate substantive
work to workflows; keep the orchestrator light.
