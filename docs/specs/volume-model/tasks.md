# Implementation Plan: Volume Model (`ff-volume`)

## Overview

This plan delivers the first-class Volume layer as a new thin crate `ff-volume` on which `ff-dscatalog` depends. It implements the Volume entity, emulated geometry, space/extent accounting with the two distinct failure points (dataset x37-style and Volume_Full), reporting, the DatasetVolume association with multivolume and uncataloged resolution, and the `DEFINE VOLUME` command contract.

These are FUTURE tasks. They are NOT performed by the authoring gate. All tasks are `[ ]` (pending). Each task cross-references the volume-model acceptance criteria it satisfies. Follow TDD per `testing.md`: write the failing test first, then the minimum implementation.

> **Crate ownership:** `ff-volume` depends only on narrow model crates (and `ff-vfs` for the StorageProvider URI seam); it SHALL NEVER depend on `ff-dscatalog`, `ff-dsalloc`, or `ff-idcams`. See design.md "Crate Ownership" and dataset-ownership-model Requirement 21.

## Tasks

- [ ] 1. Scaffold `ff-volume` crate and the Volume entity
  - [ ] 1.1 Create `crates/ff-volume/Cargo.toml` (narrow deps only; add `ff-volume` to the workspace and to `ff-dscatalog`'s `Cargo.toml`) and `src/lib.rs` with crate docs.
  - [ ] 1.2 Define the `Volume` struct (volume_id, VOLSER, display name, storage_uri, status, access_mode, capacity counters) and `VolumeStatus` (Online/Offline) + `AccessMode` (ReadWrite/ReadOnly) enums.
  - [ ] 1.3 Implement VOLSER uniqueness validation within a storage system.
    - Validates: Requirement 1.1, 1.2, 1.5

- [ ] 2. Volume status and access mode behaviour
  - [ ] 2.1 Implement set-online/set-offline and mount/unmount transitions.
  - [ ] 2.2 Reject new allocation on an Offline Volume and write/delete/extend on a ReadOnly Volume, each with a distinct reported error.
  - [ ] 2.3 Verify required Volumes are Online during resolution; report the first unavailable Volume.
    - Validates: Requirement 2.1, 2.2, 2.3, 2.4

- [ ] 3. Geometry profile and byte/track/cylinder conversions
  - [ ] 3.1 Define `GeometryProfile { bytes_per_track, tracks_per_cylinder }` with documented defaults (3390-style bytes_per_track; tracks_per_cylinder = 15).
  - [ ] 3.2 Implement deterministic byte->track, track->cylinder, and cylinder->byte conversions (ceil rounding); document them.
    - Validates: Requirement 3.1, 3.2, 3.3, 3.4, 3.5; Requirement 11.1

- [ ] 4. Space-allocation unit model (SPACE TRK/CYL/block)
  - [ ] 4.1 Define the allocation-unit model (unit kind TRK/CYL/block-avg-record + primary + secondary quantities) that the JCL SPACE forms map to.
  - [ ] 4.2 Map block/average-record requests to allocation units via the Geometry_Profile.
    - Note: the JCL SPACE keyword PARSE is owned by `ff-dsalloc`; this task is the unit model `ff-volume` exposes for it to consume.
    - Validates: Requirement 4.1, 4.2, 4.3, 4.4, 4.5

- [ ] 5. Extent model, max-extents, and the dataset x37-style failure
  - [ ] 5.1 Define the `Extent` metadata type (dataset_id, volume_id, sequence_number, allocated_units, used_bytes, opaque locator).
  - [ ] 5.2 Implement allocated-space accounting (primary + acquired secondary extents) and the overflow decision (fit / acquire secondary / fail).
  - [ ] 5.3 Implement secondary-extent acquisition up to a configurable Max_Extents (default 16) and the reported x37-style Space_Abend (B37/D37/E37 analogue) on exhaustion, leaving prior content intact.
    - Validates: Requirement 5.1, 5.2, 5.3, 5.4, 5.5; Requirement 6.1, 6.2, 6.3, 6.4

- [ ] 6. Volume capacity and the Volume_Full failure
  - [ ] 6.1 Track Volume total/used/free capacity derived from allocated extents.
  - [ ] 6.2 Implement the Volume_Full failure, distinct from the dataset x37-style failure, not consuming a dataset extent when free space is lacking.
    - Validates: Requirement 7.1, 7.2, 7.3, 7.4

- [ ] 7. Reporting counters and the VTOC_View
  - [ ] 7.1 Implement derived reporting for a dataset (extents used/remaining, tracks/cylinders in use) and a Volume (total/used/free tracks/cylinders, extents).
  - [ ] 7.2 Implement the VTOC_View (datasets + extents on a Volume + usage counters) as a derived read view.
    - Validates: Requirement 8.1, 8.2, 8.3, 8.4; Requirement 11.2

- [ ] 8. DatasetVolume association, multivolume, and uncataloged resolution
  - [ ] 8.1 Define the `DatasetVolume` association (sequence_number, is_primary, opaque locator) replacing the raw storage_path.
  - [ ] 8.2 Permit a dataset on multiple Volumes (ordered sequence) and datasets from different catalogs sharing a Volume.
  - [ ] 8.3 Implement uncataloged resolution by explicit VOL=SER + UNIT without a catalog row.
    - Validates: Requirement 9.1, 9.2, 9.3, 9.4, 9.5, 9.6

- [ ] 9. DEFINE VOLUME command contract and volume listing surface
  - [ ] 9.1 Expose the `ff-volume` service API for defining a Volume (name/VOLSER, path, capacity, status) with duplicate-VOLSER rejection.
  - [ ] 9.2 Define the `DEFINE VOLUME` command contract and the volume-listing/VTOC_View command, both resolved through the single command-dispatch path (wiring lands in `ff-desktop`/`ff-dscatalog` at implementation).
    - Note: a VTOC listing surfaced as a Context is a `WorkspaceContext` dispatched via `render_workspace_context` with a mandatory full-shell first-Tab focus test (see workspace-conformance.md).
    - Validates: Requirement 10.1, 10.2, 10.3, 10.4, 10.5

- [ ] 10. Metadata-only migration seam
  - [ ] 10.1 Expose the seam that lets `ff-dscatalog` define a Volume over an existing Repository root without moving any dataset bytes (metadata-only).
    - Validates: Requirement 11.3

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
