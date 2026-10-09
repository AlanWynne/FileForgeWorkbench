//! Supporting value types for the `DatasetAccess` contract (CR-CH-059).
//!
//! These types are the opaque handle, the open-dataset cursor, the access
//! intent, the keyed/relative positioner, the step-disposition outcome, and the
//! DD allocation request that `DatasetAccess::allocate` consumes. They compose
//! the existing seams: SPACE is expressed as an `ff_volume::AllocationUnit`
//! (the JCL SPACE parse stays in `ff-dsalloc`), the record DTO is the existing
//! `crate::Record`, and the handle carries a resolved `ff_volume` locator -- it
//! never exposes a raw `storage_path` (Requirement 34.5).
//!
//! Validates: dataset-catalog Requirement 34.1, 34.5; dataset-allocator
//! Requirement 19.3.

use std::sync::Arc;

use ff_volume::{AllocationUnit, GeometryProfile, VolumeId};

use crate::dataset::{Dsorg, Recfm};
use crate::vsam_backend::VsamBackend;
use crate::vsam_service::VsamType;

// === AccessIntent ===================================================

/// The access intent an `open` establishes for a dataset (Requirement 34.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AccessIntent {
    /// Open for sequential read (INPUT).
    Read,
    /// Open for sequential write, replacing content (OUTPUT).
    Write,
    /// Open for update in place (I-O / UPDATE).
    Update,
}

// === Positioner ===================================================

/// A record positioner for keyed or relative access (Requirement 34.1).
///
/// `Key` positions a KSDS browse/point by record key; `Rrn` positions an
/// RRDS/ESDS access by relative record number / RBA. The concrete VSAM record
/// operation these drive is RC.B.7; this run defines the positioner and proves
/// the backend seam is reachable.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Positioner {
    /// Position by record key (KSDS).
    Key(Vec<u8>),
    /// Position by relative record number (RRDS) or byte address (ESDS).
    Rrn(u64),
}

// === StepOutcome ===================================================

/// The job-step disposition `dispose` applies to a handle (Requirement 34.1).
///
/// Mirrors the normal DISP dispositions. `dispose` applies the outcome:
/// KEEP/CATLG retain the dataset, UNCATLG drops the catalog association while
/// keeping the bytes, DELETE removes the physical object and releases extents,
/// PASS retains the dataset for the remaining job scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StepOutcome {
    /// Retain the dataset and its catalog entry (DISP KEEP).
    Keep,
    /// Retain and (re)catalog the dataset (DISP CATLG).
    Catlg,
    /// Keep the bytes but drop the catalog association (DISP UNCATLG).
    Uncatlg,
    /// Delete the physical object and release its extents (DISP DELETE).
    Delete,
    /// Retain the dataset for the remaining job scope (DISP PASS).
    Pass,
}

// === DdRequest ===================================================

/// A resolved DD allocation request handed to `DatasetAccess::allocate`
/// (Requirement 34.1, 34.4).
///
/// The JCL SPACE / DISP / DCB parse is owned by `ff-dsalloc`; this request is
/// the already-parsed, Volume-ready form. SPACE is an `ff_volume::AllocationUnit`
/// plus geometry so allocation charges the Volume without re-parsing JCL
/// (dataset-allocator Requirement 17, preserved additively).
#[derive(Debug, Clone)]
pub struct DdRequest {
    /// The dataset name to allocate.
    pub dsn: String,
    /// Dataset organization.
    pub dsorg: Dsorg,
    /// Record format (drives the codec selection on open).
    pub recfm: Recfm,
    /// Logical record length.
    pub lrecl: u32,
    /// Block size (metadata only in the local emulation).
    pub blksize: u32,
    /// The SPACE request to charge against the Volume.
    pub space: AllocationUnit,
    /// The emulated geometry the SPACE request is sized against.
    pub geometry: GeometryProfile,
    /// The target Volume the dataset is allocated on.
    pub volume_id: VolumeId,
    /// Maximum extents the dataset may hold.
    pub max_extents: u32,
}

// === DatasetHandle ===================================================

/// An opaque handle identifying an allocated or resolved dataset for subsequent
/// `open` / `dispose` (Requirement 34.5).
///
/// Consumers (the allocator, a future JES executor) hold this opaquely. It
/// exposes NO raw `storage_path`, SQLite row, or provider-internal string; the
/// only observable facets are the DSN and dataset shape needed to drive I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetHandle {
    pub(crate) inner: HandleInner,
}

/// Private handle payload. Not part of the public surface (Requirement 34.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HandleInner {
    /// The dataset name.
    pub(crate) dsn: String,
    /// The Volume the dataset resides on.
    pub(crate) volume_id: VolumeId,
    /// The opaque per-volume locator (an `ff_vfs::StorageLocator` string).
    pub(crate) locator: String,
    /// Record format snapshot (drives codec selection).
    pub(crate) recfm: Recfm,
    /// Logical record length snapshot.
    pub(crate) lrecl: u32,
    /// Dataset organization snapshot.
    pub(crate) dsorg: Dsorg,
}

impl DatasetHandle {
    /// Returns the dataset name this handle identifies.
    ///
    /// This is the only DSN-level facet exposed; the physical locator stays
    /// opaque (Requirement 34.5).
    pub fn dsn(&self) -> &str {
        &self.inner.dsn
    }

    /// Returns the dataset organization snapshot captured at allocate/resolve.
    pub fn dsorg(&self) -> Dsorg {
        self.inner.dsorg
    }
}

// === OpenDataset ===================================================

/// An open dataset carrying the access intent and the record cursor
/// (Requirement 34.1).
///
/// For the sequential (PS/PO) paths this holds the decoded record vector and a
/// read cursor plus a write buffer; `close` flushes the buffer through the
/// codec. The handle is retained so `get`/`put`/`point` can reach the resolved
/// locator and the codec selection without re-resolving.
#[derive(Debug, Clone)]
pub struct OpenDataset {
    pub(crate) handle: DatasetHandle,
    pub(crate) intent: AccessIntent,
    /// Decoded records available for sequential `get`.
    pub(crate) records: Vec<Vec<u8>>,
    /// Read cursor into `records`.
    pub(crate) cursor: usize,
    /// Records accumulated by `put` pending flush on `close`.
    pub(crate) pending: Vec<Vec<u8>>,
    /// Whether `pending` holds unflushed writes.
    pub(crate) dirty: bool,
    /// VSAM positioning state for a keyed/relative open. `None` for the
    /// sequential (PS/PO) path, which does not use a `VsamBackend`.
    pub(crate) vsam: Option<VsamOpen>,
}

// === VsamOpen ===================================================

/// The VSAM positioning state carried by a keyed/relative `OpenDataset`
/// (Requirement 34.1, 35.3).
///
/// `point` sets `position`; `get`/`put` then operate at that position over the
/// shared `VsamBackend`. The backend lives behind an `Arc` so `OpenDataset`
/// stays `Clone` without duplicating the physical connections.
#[derive(Debug, Clone)]
pub(crate) struct VsamOpen {
    /// The VSAM type selected from the cluster.
    pub(crate) vsam_type: VsamType,
    /// The concrete record backend for the open cluster.
    pub(crate) backend: Arc<VsamBackend>,
    /// The current position established by `point`, if any.
    pub(crate) position: Option<Positioner>,
}

impl OpenDataset {
    /// Returns the access intent this dataset was opened with.
    pub fn intent(&self) -> AccessIntent {
        self.intent
    }

    /// Returns the handle backing this open dataset.
    pub fn handle(&self) -> &DatasetHandle {
        &self.handle
    }
}
