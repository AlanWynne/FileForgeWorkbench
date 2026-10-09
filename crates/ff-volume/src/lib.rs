//! Mainframe Volume model for FileForgeWorkbench.
//!
//! `ff-volume` is a pure in-memory model crate that introduces the first-class
//! mainframe Volume layer beneath the dataset catalog (CR-NR-105 / CR-CH-057,
//! volume-model Requirements 1-11). It owns the Volume entity, VOLSER identity,
//! volume status/access-mode behaviour, the emulated geometry profile, the SPACE
//! allocation-unit model, extent accounting with the two distinct space failures
//! (dataset x37-style Space_Abend and Volume_Full), volume-capacity counters,
//! derived VTOC reporting, and the DatasetVolume association.
//!
//! # Ownership boundary (acyclic DAG)
//!
//! `ff-volume` depends ONLY on `ff-vfs` (the StorageProvider URI seam, ADR-001),
//! `thiserror`, `serde`, and `chrono`. It MUST NOT depend on `ff-dscatalog`,
//! `ff-dsalloc`, or `ff-idcams` -- those crates depend on `ff-volume`, keeping
//! the dependency direction `ff-dscatalog -> ff-volume -> ff-vfs` acyclic
//! (dataset-ownership-model Requirement 21.3).
//!
//! # Scope
//!
//! This crate is model + behaviour only. SQLite persistence of the `volumes`
//! and `dataset_volumes` rows is owned by `ff-dscatalog`; the `DatasetAccess`
//! trait, the command-dispatch wiring, and any `WorkspaceContext` are built in
//! later phases. This crate exposes the SERVICE API and the command CONTRACT
//! (types + a documented method set) only.

#![forbid(unsafe_code)]

pub mod dataset_volume;
pub mod error;
pub mod extent;
pub mod geometry;
pub mod migration;
pub mod reporting;
pub mod service;
pub mod space;
pub mod volume;

pub use dataset_volume::{DatasetVolume, DatasetVolumeSet, ResolvedVolume};
pub use error::VolumeError;
pub use extent::{Extent, ExtentSet, SpaceAbendReason, DEFAULT_MAX_EXTENTS};
pub use geometry::{GeometryProfile, DEFAULT_BYTES_PER_TRACK, DEFAULT_TRACKS_PER_CYLINDER};
pub use migration::{seed_dataset_volumes, volume_over_repository};
pub use reporting::{DatasetSpaceReport, VolumeSpaceReport, VtocView};
pub use service::{
    DefineVolumeParams, DefineVolumeRequest, VolumeListingParams, VolumeService, VtocParams,
    DEFINE_VOLUME_COMMAND, VOLUME_LISTING_COMMAND, VTOC_COMMAND,
};
pub use space::{AllocationUnit, UnitKind};
pub use volume::{AccessMode, DatasetId, Volser, Volume, VolumeId, VolumeRegistry, VolumeStatus};
