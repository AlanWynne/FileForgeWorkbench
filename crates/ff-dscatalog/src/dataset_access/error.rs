//! The `DatasetError` taxonomy for the `DatasetAccess` contract (CR-CH-059).
//!
//! `DatasetError` is a thin enum that maps onto the existing taxonomies
//! (`CatalogError`, `ff_volume::VolumeError`, `crate::codecs::CodecError`) and,
//! where it crosses the physical seam, onto `ff_vfs::VfsError` (Requirement
//! 34.7). No `rusqlite` error and no raw `storage_path` / `StorageLocator`
//! string leaks through this surface.
//!
//! Validates: dataset-catalog Requirement 34.7.

use ff_vfs::VfsError;
use ff_volume::VolumeError;

use crate::codecs::CodecError;
use crate::error::CatalogError;

// === DatasetError ===================================================

/// Errors surfaced by the `DatasetAccess` contract.
///
/// `#[non_exhaustive]` because the contract may grow further variants (e.g.
/// concrete VSAM failures) as RC.B.7 lands.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DatasetError {
    /// The dataset was not found during open/resolve.
    #[error("dataset access: not found: {dsn}")]
    NotFound {
        /// The DSN that was not found.
        dsn: String,
    },

    /// A dataset with the requested DSN already exists.
    #[error("dataset access: already exists: {dsn}")]
    AlreadyExists {
        /// The duplicate DSN.
        dsn: String,
    },

    /// The target Volume is unavailable (Offline or missing).
    #[error("dataset access: volume unavailable: {volser}")]
    VolumeUnavailable {
        /// The VOLSER (or reference) of the unavailable Volume.
        volser: String,
    },

    /// The target Volume is ReadOnly and rejected a write/allocate/extend.
    #[error("dataset access: volume read-only: {volser}")]
    ReadOnlyVolume {
        /// The VOLSER of the ReadOnly Volume.
        volser: String,
    },

    /// The target Volume lacked free space (Volume_Full, Requirement 7).
    #[error("dataset access: volume full: {volser}")]
    VolumeFull {
        /// The VOLSER of the full Volume.
        volser: String,
    },

    /// The dataset exceeded its allocatable capacity / maximum extents
    /// (x37-style space abend, Requirements 5-6).
    #[error("dataset access: x37 space abend on dataset {dataset_id}")]
    SpaceAbend {
        /// The dataset that could not grow.
        dataset_id: u64,
    },

    /// A record codec rejected the bytes or records.
    #[error("dataset access: codec error: {source}")]
    Codec {
        /// The underlying codec error.
        #[source]
        source: CodecError,
    },

    /// A physical read/write through the `ff_vfs::StorageProvider` seam failed.
    #[error("dataset access: vfs error: {source}")]
    Vfs {
        /// The underlying VFS error.
        #[source]
        source: VfsError,
    },

    /// A catalog-metadata operation failed.
    #[error("dataset access: catalog error: {source}")]
    Catalog {
        /// The underlying catalog error.
        #[source]
        source: CatalogError,
    },

    /// The requested access intent was invalid for the operation.
    #[error("dataset access: invalid intent: {reason}")]
    InvalidIntent {
        /// Why the intent was rejected.
        reason: String,
    },

    /// The positioner did not match the dataset organization.
    #[error("dataset access: bad positioner: {reason}")]
    BadPositioner {
        /// Why the positioner was rejected.
        reason: String,
    },

    /// A VSAM record operation is defined but its concrete wiring is deferred to
    /// RC.B.7 (Requirement 35.3). The seam is reachable; the record op is not
    /// yet implemented.
    #[error("dataset access: operation not yet wired (RC.B.7): {operation}")]
    NotYetWired {
        /// The operation whose concrete wiring is deferred.
        operation: String,
    },
}

// === Mapping From impls (Requirement 34.7) ===================================================

impl From<CodecError> for DatasetError {
    fn from(source: CodecError) -> Self {
        DatasetError::Codec { source }
    }
}

impl From<VfsError> for DatasetError {
    fn from(source: VfsError) -> Self {
        // Preserve the mainframe-faithful NotFound / AlreadyExists shape where
        // the VFS layer already distinguishes them.
        match &source {
            VfsError::NotFound { uri, .. } => DatasetError::NotFound { dsn: uri.clone() },
            VfsError::AlreadyExists { uri, .. } => DatasetError::AlreadyExists { dsn: uri.clone() },
            _ => DatasetError::Vfs { source },
        }
    }
}

impl From<CatalogError> for DatasetError {
    fn from(source: CatalogError) -> Self {
        match &source {
            CatalogError::DatasetNotFound { dsn, .. } => {
                DatasetError::NotFound { dsn: dsn.clone() }
            }
            CatalogError::DuplicateDataset { dsn, .. } => {
                DatasetError::AlreadyExists { dsn: dsn.clone() }
            }
            CatalogError::VolumeUnavailable { volser, .. } => DatasetError::VolumeUnavailable {
                volser: volser.clone(),
            },
            _ => DatasetError::Catalog { source },
        }
    }
}

impl From<VolumeError> for DatasetError {
    fn from(source: VolumeError) -> Self {
        match source {
            VolumeError::VolumeOffline { volser }
            | VolumeError::VolumeNotFound { reference: volser } => {
                DatasetError::VolumeUnavailable { volser }
            }
            VolumeError::VolumeReadOnly { volser } => DatasetError::ReadOnlyVolume { volser },
            VolumeError::VolumeFull { volser } => DatasetError::VolumeFull { volser },
            VolumeError::SpaceAbend { dataset_id, .. } => DatasetError::SpaceAbend { dataset_id },
            VolumeError::DuplicateVolser { volser } => DatasetError::VolumeUnavailable { volser },
            // `VolumeError` is #[non_exhaustive]; any future variant maps to the
            // generic unavailable surface until a dedicated mapping is added.
            other => DatasetError::VolumeUnavailable {
                volser: other.to_string(),
            },
        }
    }
}

/// Map a `DatasetError` back onto `VfsError` where it crosses the physical seam
/// (Requirement 34.7). A `Vfs`-wrapped error round-trips unchanged.
impl From<DatasetError> for VfsError {
    fn from(err: DatasetError) -> VfsError {
        match err {
            DatasetError::Vfs { source } => source,
            DatasetError::NotFound { dsn } => VfsError::NotFound {
                uri: dsn,
                operation: "dataset_access".to_string(),
            },
            DatasetError::AlreadyExists { dsn } => VfsError::AlreadyExists {
                uri: dsn,
                operation: "dataset_access".to_string(),
            },
            DatasetError::ReadOnlyVolume { volser } => VfsError::PermissionDenied {
                uri: volser,
                operation: "dataset_access".to_string(),
            },
            other => VfsError::Io {
                uri: String::new(),
                operation: "dataset_access".to_string(),
                source: std::io::Error::other(other.to_string()),
            },
        }
    }
}
