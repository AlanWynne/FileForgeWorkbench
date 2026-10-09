//! VSAM record path for `CatalogDatasetAccess` (CR-CH-059, dataset-catalog
//! Requirement 34.1/34.2/35.3).
//!
//! The byte-only `ff_vfs::StorageProvider` seam cannot carry keyed/relative
//! record ops, so VSAM clusters are backed by an in-crate `VsamBackend` rooted
//! at `CatalogDatasetAccess::vsam_root`. `open_vsam` reuses the existing locator
//! resolution for the Volume Online check, then selects the backend from the
//! cluster's `VsamType`. `point` establishes a key/RRN position; `get`/`put`
//! then operate at that position over the backend. Record boundaries come from
//! the backend, never a host text line.
//!
//! Validates: dataset-catalog Requirement 34.1, 34.2, 35.3.

use std::sync::Arc;

use uuid::Uuid;

use super::error::DatasetError;
use super::impl_access::CatalogDatasetAccess;
use super::types::{AccessIntent, DatasetHandle, OpenDataset, Positioner, VsamOpen};
use crate::vsam_backend::VsamBackend;
use crate::vsam_service::{VsamParams, VsamType};
use crate::Record;

impl CatalogDatasetAccess {
    /// Open a VSAM cluster for keyed/relative record access (Requirement 34.1,
    /// 35.3).
    ///
    /// Reuses the existing locator resolution for the Volume Online check (the
    /// cluster must be allocated, like any dataset), then provisions the
    /// concrete `VsamBackend` for the cluster's `VsamType` under `vsam_root`.
    ///
    /// # Errors
    /// `DatasetError::NotFound` if the handle no longer resolves; a mapped
    /// `DatasetError` if the backend cannot be provisioned.
    pub fn open_vsam(
        &self,
        handle: &DatasetHandle,
        vsam_type: VsamType,
        params: &VsamParams,
        intent: AccessIntent,
    ) -> Result<OpenDataset, DatasetError> {
        // Reuse the Volume Online check (the cluster is an allocated dataset).
        let _locator = self.resolve_locator(handle)?;
        let dataset_id = cluster_id(handle.dsn());
        let backend = VsamBackend::open_for(&self.vsam_root, dataset_id, vsam_type, params)
            .map_err(map_vsam_err)?;
        Ok(OpenDataset {
            handle: handle.clone(),
            intent,
            records: Vec::new(),
            cursor: 0,
            pending: Vec::new(),
            dirty: false,
            vsam: Some(VsamOpen {
                vsam_type,
                backend: Arc::new(backend),
                position: None,
            }),
        })
    }

    /// Establish a VSAM position from a keyed/relative positioner (Requirement
    /// 34.1).
    pub(crate) fn point_vsam(
        &self,
        open: &mut OpenDataset,
        positioner: &Positioner,
    ) -> Result<(), DatasetError> {
        let vsam = open
            .vsam
            .as_mut()
            .ok_or_else(|| bad_positioner("dataset is not open as a VSAM cluster"))?;
        match (vsam.vsam_type, positioner) {
            (VsamType::Ksds, Positioner::Key(_)) => {}
            (VsamType::Rrds | VsamType::Esds, Positioner::Rrn(rrn)) => {
                if *rrn == 0 {
                    return Err(bad_positioner("relative record number 0 is invalid"));
                }
            }
            (vsam_type, _) => {
                return Err(bad_positioner(&format!(
                    "positioner does not match {vsam_type:?} cluster"
                )));
            }
        }
        vsam.position = Some(positioner.clone());
        Ok(())
    }

    /// Read the record at the current VSAM position (Requirement 34.2).
    pub(crate) fn get_vsam(&self, open: &mut OpenDataset) -> Result<Option<Record>, DatasetError> {
        let vsam = open
            .vsam
            .as_ref()
            .ok_or_else(|| bad_positioner("dataset is not open as a VSAM cluster"))?;
        let position = vsam
            .position
            .as_ref()
            .ok_or_else(|| bad_positioner("get requires a prior point on a VSAM cluster"))?;
        match position {
            Positioner::Key(key) => vsam.backend.get_keyed(key).map_err(map_vsam_err),
            Positioner::Rrn(rrn) => match vsam.vsam_type {
                VsamType::Esds => vsam.backend.get_address(*rrn).map_err(map_vsam_err),
                _ => vsam.backend.get_relative(*rrn).map_err(map_vsam_err),
            },
        }
    }

    /// Write the record at the current VSAM position (Requirement 34.2).
    ///
    /// For ESDS the position's address is advisory: an append always lands at a
    /// fresh stable address (append-stable-address semantics).
    pub(crate) fn put_vsam(
        &self,
        open: &mut OpenDataset,
        record: &Record,
    ) -> Result<(), DatasetError> {
        if open.intent == AccessIntent::Read {
            return Err(DatasetError::InvalidIntent {
                reason: "cannot put on a dataset opened for read".to_string(),
            });
        }
        let vsam = open
            .vsam
            .as_ref()
            .ok_or_else(|| bad_positioner("dataset is not open as a VSAM cluster"))?;
        let position = vsam
            .position
            .as_ref()
            .ok_or_else(|| bad_positioner("put requires a prior point on a VSAM cluster"))?;
        match position {
            Positioner::Key(key) => vsam
                .backend
                .put_keyed(&Record {
                    key: key.clone(),
                    data: record.data.clone(),
                })
                .map_err(map_vsam_err),
            Positioner::Rrn(rrn) => match vsam.vsam_type {
                VsamType::Esds => vsam
                    .backend
                    .append_entry(&record.data)
                    .map(|_| ())
                    .map_err(map_vsam_err),
                _ => vsam
                    .backend
                    .put_relative(*rrn, &record.data)
                    .map_err(map_vsam_err),
            },
        }
    }
}

/// Derive the stable backend UUID for a cluster from its DSN.
///
/// A deterministic 128-bit FNV-1a fold over the uppercased DSN bytes keeps the
/// same cluster mapped to the same backend across repeated opens, without
/// requiring the `uuid` crate's `v5` feature.
fn cluster_id(dsn: &str) -> Uuid {
    const FNV_OFFSET: u128 = 0x6c62272e07bb014262b821756295c58d;
    const FNV_PRIME: u128 = 0x0000000001000000000000000000013b;
    let mut hash = FNV_OFFSET;
    for byte in dsn.to_uppercase().as_bytes() {
        hash ^= u128::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    Uuid::from_u128(hash)
}

fn bad_positioner(reason: &str) -> DatasetError {
    DatasetError::BadPositioner {
        reason: reason.to_string(),
    }
}

/// Map a `VsamError` onto the `DatasetError` taxonomy.
fn map_vsam_err(source: crate::vsam_service::VsamError) -> DatasetError {
    use crate::vsam_service::VsamError;
    match source {
        VsamError::DatasetNotFound { dsn } => DatasetError::NotFound { dsn },
        VsamError::DatasetAlreadyExists { dsn } => DatasetError::AlreadyExists { dsn },
        VsamError::RecordNotFound { dsn } => DatasetError::NotFound { dsn },
        VsamError::UnsupportedOperation { reason } => DatasetError::BadPositioner { reason },
        other => DatasetError::BadPositioner {
            reason: other.to_string(),
        },
    }
}
