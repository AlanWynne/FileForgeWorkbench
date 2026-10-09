//! The `DatasetAccess` contract -- the single mainframe-faithful dataset-access
//! interface owned by `ff-dscatalog` (CR-CH-059, Requirement 34).
//!
//! `DatasetAccess` composes the existing seams: it resolves physical location
//! through `ff-volume` (Dataset -> DatasetVolume -> Volume -> locator), performs
//! all physical I/O through the single `ff_vfs::StorageProvider` seam, and
//! encodes/decodes records with the RECFM codecs. It introduces no second
//! dispatcher, navigation stack, or persistence format.
//!
//! The concrete `CatalogDatasetAccess` wires both the sequential (PS/PO +
//! member) paths and the VSAM keyed/relative record ops end-to-end: VSAM
//! clusters open over an in-crate `VsamBackend` (KSDS/ESDS/RRDS) and `point` /
//! `get` / `put` operate at the keyed/relative position (see `impl_vsam.rs`).

mod error;
mod impl_access;
mod impl_io;
mod impl_vsam;
mod trait_def;
mod types;

#[cfg(test)]
mod impl_access_tests;
#[cfg(test)]
mod impl_io_tests;
#[cfg(test)]
mod impl_vsam_tests;

pub use error::DatasetError;
pub use impl_access::CatalogDatasetAccess;
pub use trait_def::DatasetAccess;
pub use types::{AccessIntent, DatasetHandle, DdRequest, OpenDataset, Positioner, StepOutcome};
