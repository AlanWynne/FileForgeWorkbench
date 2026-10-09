//! In-crate `VsamBackend` abstraction over the concrete RC.A VSAM backends
//! (CR-CH-059, dataset-catalog Requirement 34.1/34.2/35.3).
//!
//! The single physical seam `ff_vfs::StorageProvider` is byte-oriented and
//! cannot carry keyed/relative record ops, so `CatalogDatasetAccess` cannot
//! reach the typed KSDS/ESDS/RRDS APIs through `Arc<dyn StorageProvider>`.
//! `VsamBackend` resolves this by OWNING the concrete providers directly. It is
//! legal because it lives in `ff-dscatalog` alongside the backends, it adds no
//! new backend, it does not widen any public trait, and it preserves the
//! acyclic DAG `ff-dscatalog -> ff-volume -> ff-vfs`. VSAM remains a cluster
//! entity (`VsamType`), never a `Dsorg` variant.
//!
//! Validates: dataset-catalog Requirement 34.1, 34.2, 35.3.

use std::path::Path;

use uuid::Uuid;

use crate::error::CatalogError;
use crate::storage::{
    AlternateIndex, KsdsKeyDefinition, NativeEsdsProvider, RrdsSlot, SqliteRecordProvider,
    SqliteRrdsProvider,
};
use crate::vsam_service::{BrowseDirection, Record, VsamError, VsamParams, VsamType};

// === VsamBackend =============================================================

/// A concrete VSAM record backend selected by `VsamType`.
///
/// Each variant owns the RC.A provider for that organization. The thin typed
/// methods below are the only surface `DatasetAccess` / `CatalogVsamService`
/// use; they map the backend's `CatalogError` onto `VsamError` locally.
#[derive(Debug)]
pub(crate) enum VsamBackend {
    /// Key-Sequenced Data Set (keyed access).
    Ksds(SqliteRecordProvider),
    /// Entry-Sequenced Data Set (append with stable address).
    Esds(NativeEsdsProvider),
    /// Relative Record Data Set (positioning by RRN).
    Rrds(SqliteRrdsProvider),
}

impl VsamBackend {
    /// Provision a backend for `vsam_type` under `root` keyed by `dataset_id`.
    ///
    /// KSDS derives its key field from `params.key_offset` / `params.key_length`
    /// via `KsdsKeyDefinition::new`. LDS has no concrete backend yet and returns
    /// `VsamError::UnsupportedOperation` with a documented reason.
    pub(crate) fn open_for(
        root: &Path,
        dataset_id: Uuid,
        vsam_type: VsamType,
        params: &VsamParams,
    ) -> Result<Self, VsamError> {
        match vsam_type {
            VsamType::Ksds => {
                let offset = u32::from(params.key_offset.unwrap_or(0));
                let length = u32::from(params.key_length.unwrap_or(1)).max(1);
                let definition = KsdsKeyDefinition::new(offset, length);
                let provider =
                    SqliteRecordProvider::open(root, dataset_id, definition).map_err(map_err)?;
                Ok(Self::Ksds(provider))
            }
            VsamType::Esds => {
                let provider = NativeEsdsProvider::open(root, dataset_id).map_err(map_err)?;
                Ok(Self::Esds(provider))
            }
            VsamType::Rrds => {
                let provider = SqliteRrdsProvider::open(root, dataset_id).map_err(map_err)?;
                Ok(Self::Rrds(provider))
            }
            VsamType::Lds => Err(VsamError::UnsupportedOperation {
                reason: "LDS has no concrete record backend yet (byte-stream only)".to_string(),
            }),
        }
    }

    /// The VSAM type this backend serves.
    pub(crate) fn vsam_type(&self) -> VsamType {
        match self {
            Self::Ksds(_) => VsamType::Ksds,
            Self::Esds(_) => VsamType::Esds,
            Self::Rrds(_) => VsamType::Rrds,
        }
    }

    // === Keyed (KSDS) ===================================================

    /// Read a record by key (KSDS). `None` when no record has that key.
    pub(crate) fn get_keyed(&self, key: &[u8]) -> Result<Option<Record>, VsamError> {
        let ksds = self.as_ksds("get_keyed")?;
        let key_text = key_to_text(key)?;
        let found = ksds.read(&key_text).map_err(map_err)?;
        Ok(found.map(|record| Record {
            key: record.key.into_bytes(),
            data: record.data,
        }))
    }

    /// Insert or update a keyed record (KSDS). Insert first, update on conflict.
    pub(crate) fn put_keyed(&self, record: &Record) -> Result<(), VsamError> {
        let ksds = self.as_ksds("put_keyed")?;
        let key_text = key_to_text(&record.key)?;
        match ksds.insert(&key_text, &record.data) {
            Ok(()) => Ok(()),
            Err(_) => {
                ksds.update(&key_text, &record.data).map_err(map_err)?;
                Ok(())
            }
        }
    }

    /// Delete a keyed record (KSDS). Returns whether a record was removed.
    pub(crate) fn delete_keyed(&self, key: &[u8]) -> Result<bool, VsamError> {
        let ksds = self.as_ksds("delete_keyed")?;
        let key_text = key_to_text(key)?;
        ksds.delete(&key_text).map_err(map_err)
    }

    /// Browse records from `start` in `direction`, in primary-key order (KSDS).
    pub(crate) fn browse_from(
        &self,
        start: &[u8],
        direction: BrowseDirection,
    ) -> Result<Vec<Record>, VsamError> {
        let ksds = self.as_ksds("browse_from")?;
        let start_text = key_to_text(start)?;
        let mut records: Vec<Record> = ksds
            .sequential_read()
            .map_err(map_err)?
            .into_iter()
            .filter(|record| record.key.as_str() >= start_text.as_str())
            .map(|record| Record {
                key: record.key.into_bytes(),
                data: record.data,
            })
            .collect();
        if direction == BrowseDirection::Backward {
            records.reverse();
        }
        Ok(records)
    }

    /// Look up primary keys via an alternate index (KSDS).
    ///
    /// Part of the backend's keyed-access surface and exercised by the
    /// alternate-index round-trip test; the `VsamService` trait exposes only
    /// define/build, so no non-test caller references it yet.
    #[allow(dead_code)]
    pub(crate) fn lookup_alternate(
        &self,
        index_name: &str,
        alt_key: &[u8],
    ) -> Result<Vec<Vec<u8>>, VsamError> {
        let ksds = self.as_ksds("lookup_alternate")?;
        let alt_text = key_to_text(alt_key)?;
        let keys = ksds
            .lookup_by_alternate_key(index_name, &alt_text)
            .map_err(map_err)?;
        Ok(keys.into_iter().map(String::into_bytes).collect())
    }

    /// Define an alternate index over this KSDS.
    pub(crate) fn add_alternate_index(&self, definition: &AlternateIndex) -> Result<(), VsamError> {
        let ksds = self.as_ksds("add_alternate_index")?;
        ksds.add_alternate_index(definition).map_err(map_err)
    }

    /// Rebuild an alternate index from the current record set (KSDS).
    pub(crate) fn rebuild_alternate_index(&self, name: &str) -> Result<(), VsamError> {
        let ksds = self.as_ksds("rebuild_alternate_index")?;
        ksds.rebuild_alternate_index(name).map_err(map_err)
    }

    // === Relative (RRDS) ===================================================

    /// Read a record by relative record number (RRDS). RRN 0 is rejected by the
    /// backend; `None` when the slot is unallocated.
    pub(crate) fn get_relative(&self, rrn: u64) -> Result<Option<Record>, VsamError> {
        let rrds = self.as_rrds("get_relative")?;
        match rrds.read(rrn).map_err(map_err)? {
            RrdsSlot::Unallocated => Ok(None),
            RrdsSlot::Allocated(data) => Ok(Some(Record {
                key: rrn.to_be_bytes().to_vec(),
                data,
            })),
        }
    }

    /// Write a record at a relative record number (RRDS). RRN 0 is rejected.
    pub(crate) fn put_relative(&self, rrn: u64, data: &[u8]) -> Result<(), VsamError> {
        let rrds = self.as_rrds("put_relative")?;
        rrds.write(rrn, data).map_err(map_err)
    }

    /// Delete the slot at a relative record number (RRDS).
    pub(crate) fn delete_relative(&self, rrn: u64) -> Result<bool, VsamError> {
        let rrds = self.as_rrds("delete_relative")?;
        rrds.delete_record(rrn).map_err(map_err)
    }

    // === Append (ESDS) ===================================================

    /// Append a record (ESDS), returning its stable byte-offset address.
    pub(crate) fn append_entry(&self, data: &[u8]) -> Result<u64, VsamError> {
        let esds = self.as_esds("append_entry")?;
        esds.append(data).map_err(map_err)
    }

    /// Read an ESDS record by its stable address. `None` when absent/deleted.
    pub(crate) fn get_address(&self, address: u64) -> Result<Option<Record>, VsamError> {
        let esds = self.as_esds("get_address")?;
        Ok(esds.read(address).map_err(map_err)?.map(|record| Record {
            key: record.address.to_be_bytes().to_vec(),
            data: record.data,
        }))
    }

    // === Sequential (all types) ===================================================

    /// Read every active record in the backend's natural order.
    pub(crate) fn sequential(&self) -> Result<Vec<Record>, VsamError> {
        match self {
            Self::Ksds(ksds) => Ok(ksds
                .sequential_read()
                .map_err(map_err)?
                .into_iter()
                .map(|record| Record {
                    key: record.key.into_bytes(),
                    data: record.data,
                })
                .collect()),
            Self::Esds(esds) => Ok(esds
                .sequential_read()
                .map_err(map_err)?
                .into_iter()
                .map(|record| Record {
                    key: record.address.to_be_bytes().to_vec(),
                    data: record.data,
                })
                .collect()),
            Self::Rrds(rrds) => Ok(rrds
                .sequential_read()
                .map_err(map_err)?
                .into_iter()
                .map(|record| Record {
                    key: record.record_number.to_be_bytes().to_vec(),
                    data: record.data,
                })
                .collect()),
        }
    }

    // === Variant guards ===================================================

    fn as_ksds(&self, operation: &str) -> Result<&SqliteRecordProvider, VsamError> {
        match self {
            Self::Ksds(ksds) => Ok(ksds),
            _ => Err(unsupported(operation, self.vsam_type())),
        }
    }

    fn as_esds(&self, operation: &str) -> Result<&NativeEsdsProvider, VsamError> {
        match self {
            Self::Esds(esds) => Ok(esds),
            _ => Err(unsupported(operation, self.vsam_type())),
        }
    }

    fn as_rrds(&self, operation: &str) -> Result<&SqliteRrdsProvider, VsamError> {
        match self {
            Self::Rrds(rrds) => Ok(rrds),
            _ => Err(unsupported(operation, self.vsam_type())),
        }
    }
}

/// Decode a byte key into the backend's text key representation (KSDS keys are
/// text in the underlying provider).
fn key_to_text(key: &[u8]) -> Result<String, VsamError> {
    String::from_utf8(key.to_vec()).map_err(|_| VsamError::UnsupportedOperation {
        reason: "KSDS keys must be valid UTF-8 text".to_string(),
    })
}

fn unsupported(operation: &str, vsam_type: VsamType) -> VsamError {
    VsamError::UnsupportedOperation {
        reason: format!("{operation} not supported for {vsam_type:?}"),
    }
}

/// Map a backend `CatalogError` onto the `VsamService` taxonomy.
fn map_err(source: CatalogError) -> VsamError {
    match source {
        CatalogError::DatasetNotFound { dsn, .. } => VsamError::DatasetNotFound { dsn },
        CatalogError::DuplicateDataset { dsn, .. } => VsamError::DatasetAlreadyExists { dsn },
        other => VsamError::StorageError(other.to_string()),
    }
}

#[cfg(test)]
#[path = "vsam_backend_tests.rs"]
mod tests;
