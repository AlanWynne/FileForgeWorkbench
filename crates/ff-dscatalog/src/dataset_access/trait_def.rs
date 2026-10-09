//! The object-safe `DatasetAccess` trait (CR-CH-059, Requirement 34).
//!
//! `DatasetAccess` is the SINGLE record-aware dataset-access contract: the JCL
//! executor, the editor's future MAINFRAME BackendEnvironment SAVE path, and
//! IDCAMS all perform record-level I/O through it (Requirement 34.6). It
//! resolves physical location through `ff-volume` and does all physical I/O
//! through the single `ff_vfs::StorageProvider` seam; errors are the mapped
//! `DatasetError` (Requirement 34.3, 34.7). The trait is object-safe so a
//! future executor and tests can hold `dyn DatasetAccess` and substitute a mock
//! (Requirement 34.8).
//!
//! Validates: dataset-catalog Requirement 34.1, 34.6, 34.8.

use super::error::DatasetError;
use super::types::{AccessIntent, DatasetHandle, DdRequest, OpenDataset, Positioner, StepOutcome};
use crate::Record;

/// The single mainframe-faithful dataset-access contract (Requirement 34.1).
///
/// Method set: `allocate` / `open` / `get` / `put` / `point` / `close` /
/// `dispose`. All signatures are object-safe (`&self`, no generics, no `Self`
/// by value) so `dyn DatasetAccess` is usable (Requirement 34.8).
pub trait DatasetAccess: Send + Sync {
    /// Allocate a new dataset per the resolved DD request, honouring Volume
    /// status/access mode and charging SPACE against the Volume, returning an
    /// opaque `DatasetHandle` (Requirement 34.1, 34.4, 34.5).
    ///
    /// # Errors
    /// `DatasetError::VolumeUnavailable` / `ReadOnlyVolume` for a rejected
    /// Volume; `VolumeFull` / `SpaceAbend` for a space failure; `AlreadyExists`
    /// for a duplicate DSN.
    fn allocate(&self, dd: &DdRequest) -> Result<DatasetHandle, DatasetError>;

    /// Resolve an existing dataset by name to an opaque `DatasetHandle`,
    /// honouring the SAME Volume Online check as `open` / `allocate`
    /// (dataset-allocator Requirement 19.3, dataset-catalog Requirement 34.5).
    ///
    /// This closes the deferred resolve-by-DSN capability: a consumer holding
    /// only a DSN (e.g. the JCL allocator verifying a DISP=OLD/SHR dataset or
    /// passing one between steps) obtains the opaque handle without ever
    /// naming a raw physical path. The `intent` selects the access the handle
    /// will be used for; resolution itself exposes no path.
    ///
    /// # Errors
    /// `DatasetError::NotFound` if the DSN is not catalogued;
    /// `DatasetError::VolumeUnavailable` if the resolved Volume is Offline.
    fn resolve(&self, dsn: &str, intent: AccessIntent) -> Result<DatasetHandle, DatasetError>;

    /// Open a previously allocated/resolved dataset for the given intent,
    /// selecting the RECFM codec and resolving the physical locator
    /// (Requirement 34.1, 34.3).
    ///
    /// # Errors
    /// `DatasetError::NotFound` if the handle no longer resolves;
    /// `DatasetError::Vfs` on a physical read failure.
    fn open(
        &self,
        handle: &DatasetHandle,
        intent: AccessIntent,
    ) -> Result<OpenDataset, DatasetError>;

    /// Return the next record in sequence, or `None` at end of data. Record
    /// boundaries come from the RECFM codec, never a host text line
    /// (Requirement 34.2).
    ///
    /// # Errors
    /// `DatasetError::BadPositioner` for a VSAM open with no prior `point`;
    /// `DatasetError::Codec` on a malformed record stream.
    fn get(&self, open: &mut OpenDataset) -> Result<Option<Record>, DatasetError>;

    /// Append a record, encoding it through the RECFM codec on `close`
    /// (Requirement 34.2).
    ///
    /// # Errors
    /// `DatasetError::InvalidIntent` if the dataset was opened read-only;
    /// `DatasetError::BadPositioner` for a VSAM put with no prior `point`.
    fn put(&self, open: &mut OpenDataset, record: &Record) -> Result<(), DatasetError>;

    /// Position the open dataset for keyed (KSDS) or relative (RRDS/ESDS)
    /// access (Requirement 34.1). For a VSAM cluster open this establishes the
    /// key/RRN position over the backend; `get`/`put` then operate there.
    ///
    /// # Errors
    /// `DatasetError::BadPositioner` if the positioner does not match the
    /// dataset organization (e.g. a keyed/relative positioner on a sequential
    /// dataset, a mismatched VSAM type, or relative record number 0).
    fn point(&self, open: &mut OpenDataset, positioner: &Positioner) -> Result<(), DatasetError>;

    /// Flush any pending writes through the codec and release the open dataset
    /// (Requirement 34.1).
    ///
    /// # Errors
    /// `DatasetError::Vfs` on a physical write failure; `DatasetError::Codec`
    /// on an encode failure; a Volume space failure if the flush grows the
    /// dataset past its capacity.
    fn close(&self, open: OpenDataset) -> Result<(), DatasetError>;

    /// Apply the step disposition to the handle (Requirement 34.1).
    ///
    /// # Errors
    /// `DatasetError::Vfs` if a DELETE physical remove fails.
    fn dispose(&self, handle: DatasetHandle, outcome: StepOutcome) -> Result<(), DatasetError>;
}

/// Compile-time guard that `DatasetAccess` is object-safe (Requirement 34.8).
/// A non-object-safe signature would fail to compile this reference.
#[allow(dead_code)]
fn _assert_object_safe(_: &dyn DatasetAccess) {}
