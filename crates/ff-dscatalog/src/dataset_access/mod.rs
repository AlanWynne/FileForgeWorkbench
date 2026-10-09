//! The `DatasetAccess` contract -- the single mainframe-faithful dataset-access
//! interface owned by `ff-dscatalog` (CR-CH-059, Requirement 34).
//!
//! `DatasetAccess` composes the existing seams: it resolves physical location
//! through `ff-volume` (Dataset -> DatasetVolume -> Volume -> locator), performs
//! all physical I/O through the single `ff_vfs::StorageProvider` seam, and
//! encodes/decodes records with the RECFM codecs. It introduces no second
//! dispatcher, navigation stack, or persistence format.
//!
//! The concrete `CatalogDatasetAccess` wires the NON-VSAM (PS/PO sequential +
//! member) paths end-to-end. The VSAM keyed/relative record ops are DEFINED
//! with their backend seam reachable but return `DatasetError::NotYetWired`
//! (RC.B.7 owns the concrete VSAM wiring; this run does not enter that scope).

mod error;
mod impl_access;
mod impl_io;
mod trait_def;
mod types;

#[cfg(test)]
mod impl_access_tests;
#[cfg(test)]
mod impl_io_tests;

pub use error::DatasetError;
pub use impl_access::CatalogDatasetAccess;
pub use trait_def::DatasetAccess;
pub use types::{AccessIntent, DatasetHandle, DdRequest, OpenDataset, Positioner, StepOutcome};
