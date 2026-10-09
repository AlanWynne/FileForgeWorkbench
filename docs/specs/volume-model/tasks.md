# Implementation Plan: Volume Model (`ff-volume`)

## Overview

This plan delivers the first-class Volume layer as a new thin crate `ff-volume` on which `ff-dscatalog` depends. It implements the Volume entity, emulated geometry, space/extent accounting with the two distinct failure points (dataset x37-style and Volume_Full), reporting, the DatasetVolume association with multivolume and uncataloged resolution, and the `DEFINE VOLUME` command contract.

These are FUTURE tasks. They are NOT performed by the authoring gate. All tasks are `[ ]` (pending). Each task cross-references the volume-model acceptance criteria it satisfies. Follow TDD per `testing.md`: write the failing test first, then the minimum implementation.

> **Crate ownership:** `ff-volume` depends only on narrow model crates (and `ff-vfs` for the StorageProvider URI seam); it SHALL NEVER depend on `ff-dscatalog`, `ff-dsalloc`, or `ff-idcams`. See design.md "Crate Ownership" and dataset-ownership-model Requirement 21.

## Tasks

- [x] 1. Scaffold `ff-volume` crate and the Volume entity
  - [x] 1.1 Create `crates/ff-volume/Cargo.toml` (narrow deps only; add `ff-volume` to the workspace and to `ff-dscatalog`'s `Cargo.toml`) and `src/lib.rs` with crate docs.
  - [x] 1.2 Define the `Volume` struct (volume_id, VOLSER, display name, storage_uri, status, access_mode, capacity counters) and `VolumeStatus` (Online/Offline) + `AccessMode` (ReadWrite/ReadOnly) enums.
  - [x] 1.3 Implement VOLSER uniqueness validation within a storage system.
    - Validates: Requirement 1.1, 1.2, 1.5

- [x] 2. Volume status and access mode behaviour
  - [x] 2.1 Implement set-online/set-offline and mount/unmount transitions.
  - [x] 2.2 Reject new allocation on an Offline Volume and write/delete/extend on a ReadOnly Volume, each with a distinct reported error.
  - [x] 2.3 Verify required Volumes are Online during resolution; report the first unavailable Volume.
    - Validates: Requirement 2.1, 2.2, 2.3, 2.4

- [x] 3. Geometry profile and byte/track/cylinder conversions
  - [x] 3.1 Define `GeometryProfile { bytes_per_track, tracks_per_cylinder }` with documented defaults (3390-style bytes_per_track; tracks_per_cylinder = 15).
  - [x] 3.2 Implement deterministic byte->track, track->cylinder, and cylinder->byte conversions (ceil rounding); document them.
    - Validates: Requirement 3.1, 3.2, 3.3, 3.4, 3.5; Requirement 11.1

- [x] 4. Space-allocation unit model (SPACE TRK/CYL/block)
  - [x] 4.1 Define the allocation-unit model (unit kind TRK/CYL/block-avg-record + primary + secondary quantities) that the JCL SPACE forms map to.
  - [x] 4.2 Map block/average-record requests to allocation units via the Geometry_Profile.
    - Note: the JCL SPACE keyword PARSE is owned by `ff-dsalloc`; this task is the unit model `ff-volume` exposes for it to consume.
    - Validates: Requirement 4.1, 4.2, 4.3, 4.4, 4.5

- [x] 5. Extent model, max-extents, and the dataset x37-style failure
  - [x] 5.1 Define the `Extent` metadata type (dataset_id, volume_id, sequence_number, allocated_units, used_bytes, opaque locator).
  - [x] 5.2 Implement allocated-space accounting (primary + acquired secondary extents) and the overflow decision (fit / acquire secondary / fail).
  - [x] 5.3 Implement secondary-extent acquisition up to a configurable Max_Extents (default 16) and the reported x37-style Space_Abend (B37/D37/E37 analogue) on exhaustion, leaving prior content intact.
    - Validates: Requirement 5.1, 5.2, 5.3, 5.4, 5.5; Requirement 6.1, 6.2, 6.3, 6.4

- [x] 6. Volume capacity and the Volume_Full failure
  - [x] 6.1 Track Volume total/used/free capacity derived from allocated extents.
  - [x] 6.2 Implement the Volume_Full failure, distinct from the dataset x37-style failure, not consuming a dataset extent when free space is lacking.
    - Validates: Requirement 7.1, 7.2, 7.3, 7.4

- [x] 7. Reporting counters and the VTOC_View
  - [x] 7.1 Implement derived reporting for a dataset (extents used/remaining, tracks/cylinders in use) and a Volume (total/used/free tracks/cylinders, extents).
  - [x] 7.2 Implement the VTOC_View (datasets + extents on a Volume + usage counters) as a derived read view.
    - Validates: Requirement 8.1, 8.2, 8.3, 8.4; Requirement 11.2

- [x] 8. DatasetVolume association, multivolume, and uncataloged resolution
  - [x] 8.1 Define the `DatasetVolume` association (sequence_number, is_primary, opaque locator) replacing the raw storage_path.
  - [x] 8.2 Permit a dataset on multiple Volumes (ordered sequence) and datasets from different catalogs sharing a Volume.
  - [x] 8.3 Implement uncataloged resolution by explicit VOL=SER + UNIT without a catalog row.
    - Validates: Requirement 9.1, 9.2, 9.3, 9.4, 9.5, 9.6

- [x] 9. DEFINE VOLUME command contract and volume listing surface
  - [x] 9.1 Expose the `ff-volume` service API for defining a Volume (name/VOLSER, path, capacity, status) with duplicate-VOLSER rejection.
  - [x] 9.2 Define the `DEFINE VOLUME` command contract and the volume-listing/VTOC_View command, both resolved through the single command-dispatch path (wiring lands in `ff-desktop`/`ff-dscatalog` at implementation).
    - Note: a VTOC listing surfaced as a Context is a `WorkspaceContext` dispatched via `render_workspace_context` with a mandatory full-shell first-Tab focus test (see workspace-conformance.md).
    - Validates: Requirement 10.1, 10.2, 10.3, 10.4, 10.5

- [x] 10. Metadata-only migration seam
  - [x] 10.1 Expose the seam that lets `ff-dscatalog` define a Volume over an existing Repository root without moving any dataset bytes (metadata-only).
    - Validates: Requirement 11.3

---

## Mainframe Dataset Stack Rationalisation (CR-CH-059)

> Phase RC.B. Reference-only: the `DatasetAccess` contract (owned by `ff-dscatalog`) resolves location through the existing `ff-volume` structures. This adds NO new Volume behaviour; the DatasetAccess trait and its tests live in dataset-catalog Task 40. All tasks `[ ]`.

- [x] 11. Confirm the ff-volume resolution seam for DatasetAccess
  - [x] 11.1 Confirm `ff-volume` exposes the resolution seam (Dataset -> DatasetVolume -> Volume -> locator with Online check) that `DatasetAccess` (dataset-catalog Requirement 34) consumes, and the SPACE-charge/status API (Tasks 2, 4-7) it drives; no new Volume type or failure mode is added. (Confirmed: `DatasetAccess` resolves via the ff-volume seam -- test `resolves_via_volume_datasetvolume_locator`; SPACE charged via the ff-volume extent API -- `allocate_charges_space_against_volume`, `put_growth_surfaces_x37_space_abend`.)
    - Validates: Requirement 12.1, 12.4
  - [x] 11.2 Confirm `DatasetAccess` reads the `DatasetVolume` locator (not `storage_path`) and that `ff-volume` does not depend on `ff-dscatalog`/`ff-dsalloc`/`ff-idcams` (acyclic DAG); record in TCR Req 12.1-12.4. (Confirmed: `physical_io_through_storage_provider_seam` + the DatasetVolume-locator resolution; ff-volume deps remain ff-vfs + thiserror/serde/chrono only, builds without ff-dscatalog.)
    - Validates: Requirement 12.2, 12.3

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Volume Entity | 1.1-1.5 | 1.1, 1.2, 1.3 |
| Req 2: Status/Access | 2.1-2.5 | 2.1, 2.2, 2.3; 9.2 (2.5) |
| Req 3: Geometry | 3.1-3.5 | 3.1, 3.2 |
| Req 4: SPACE units | 4.1-4.5 | 4.1, 4.2 |
| Req 5: x37 failure | 5.1-5.5 | 5.2, 5.3 |
| Req 6: Secondary extents | 6.1-6.4 | 5.3 |
| Req 7: Volume_Full | 7.1-7.4 | 6.1, 6.2 |
| Req 8: Reporting/VTOC | 8.1-8.5 | 7.1, 7.2; 9.2 (8.5) |
| Req 9: Multivol/Uncat | 9.1-9.6 | 8.1, 8.2, 8.3 |
| Req 10: DEFINE VOLUME | 10.1-10.5 | 9.1, 9.2 |
| Req 11: NFR determinism/migration | 11.1-11.3 | 3.2 (11.1), 7.1 (11.2), 10.1 (11.3) |
| Req 12: DatasetAccess resolves via ff-volume (CR-CH-059) | 12.1-12.4 | 11.1, 11.2 |
