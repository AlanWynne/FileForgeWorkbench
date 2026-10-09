//! The single `VolumeError` enum for the Volume model.
//!
//! The two space failures are DISTINCT variants: `SpaceAbend` is the
//! dataset-level x37-style failure (Requirements 5, 6), and `VolumeFull` is the
//! volume-capacity failure (Requirement 7). A caller can always tell which
//! cause occurred (Requirements 5.5, 7.3).

use crate::extent::SpaceAbendReason;

// === VolumeError ===================================================

/// Errors reported by the Volume model.
///
/// `#[non_exhaustive]` because the source error model lists further failure
/// states (recovering, device error) that may be modelled later.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum VolumeError {
    /// A dataset write/append/extend exceeded its allocatable capacity because
    /// no further secondary extent could be acquired (maximum extents reached).
    /// Emulates the z/OS x37 abend family (B37/D37/E37). Distinct from
    /// `VolumeFull` (Requirement 5.5).
    #[error("x37-style space abend on dataset {dataset_id}: {reason}")]
    SpaceAbend {
        /// The dataset that could not grow.
        dataset_id: u64,
        /// Why the dataset could not grow.
        reason: SpaceAbendReason,
    },

    /// An allocation or extent acquisition required more free space than the
    /// target Volume had. Distinct from `SpaceAbend` (Requirement 7.3).
    #[error("volume {volser} is full")]
    VolumeFull {
        /// The VOLSER of the Volume that lacked free space.
        volser: String,
    },

    /// An operation targeted an Offline Volume (Requirement 2.1, 2.4).
    #[error("volume {volser} is offline")]
    VolumeOffline {
        /// The VOLSER of the offline Volume.
        volser: String,
    },

    /// A write/delete/extend targeted a ReadOnly Volume (Requirement 2.2).
    #[error("volume {volser} is read-only")]
    VolumeReadOnly {
        /// The VOLSER of the read-only Volume.
        volser: String,
    },

    /// A referenced Volume (by id or VOLSER) was not found in the registry.
    #[error("volume not found: {reference}")]
    VolumeNotFound {
        /// The id or VOLSER that could not be resolved.
        reference: String,
    },

    /// A `DEFINE VOLUME` / registry insert used a VOLSER that already exists
    /// (Requirements 1.2, 10.3).
    #[error("duplicate VOLSER: {volser}")]
    DuplicateVolser {
        /// The VOLSER that was already registered.
        volser: String,
    },
}
