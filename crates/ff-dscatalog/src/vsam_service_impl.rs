//! Concrete `VsamService` over `VsamBackend` (CR-CH-059, dataset-catalog
//! Requirement 33.3/34.1/34.2/35.3).
//!
//! `CatalogVsamService` is the reconciled `ff-dscatalog` implementation of the
//! `VsamService` trait. It provisions a `VsamBackend` per cluster, routes
//! keyed/relative/append record ops through the backend, and iterates browse
//! handles over the backend's sequential / browse order. Record boundaries come
//! from the backend, never from host text lines. It adds no new backend and no
//! second dispatcher -- all physical record I/O flows through the RC.A backends.
//!
//! Validates: dataset-catalog Requirement 33.3, 34.1, 34.2, 35.3.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use uuid::Uuid;

use crate::vsam_backend::VsamBackend;
use crate::vsam_service::{
    AccessMode, BrowseDirection, BrowseHandle, KeyField, Record, VsamError, VsamHandle, VsamParams,
    VsamService, VsamType,
};

// === State ===================================================================

/// A provisioned (but not necessarily open) cluster definition.
struct ClusterDef {
    dataset_id: Uuid,
    vsam_type: VsamType,
    params: VsamParams,
}

/// An open browse: the materialised record list plus the current cursor.
struct BrowseState {
    records: Vec<Record>,
    cursor: usize,
}

/// Interior-mutable state behind the `&self` trait surface.
#[derive(Default)]
struct ServiceState {
    clusters: HashMap<String, ClusterDef>,
    handles: HashMap<u64, String>,
    browses: HashMap<u64, BrowseState>,
    next_handle: u64,
    next_browse: u64,
}

// === CatalogVsamService ===================================================

/// The reconciled concrete `VsamService` backed by `VsamBackend`.
///
/// Clusters live under `root`; each is addressed by a UUID derived at creation,
/// never by its DSN. The service is `Send + Sync` behind a `Mutex`.
pub struct CatalogVsamService {
    root: PathBuf,
    state: Mutex<ServiceState>,
}

impl std::fmt::Debug for CatalogVsamService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CatalogVsamService")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl CatalogVsamService {
    /// Create a service whose clusters are stored under `root`.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            state: Mutex::new(ServiceState::default()),
        }
    }

    fn key(dsn: &str) -> String {
        dsn.trim().to_uppercase()
    }

    /// Open a transient backend for the cluster named `dsn`.
    fn backend_for(&self, dsn: &str) -> Result<VsamBackend, VsamError> {
        let state = self.state.lock().expect("vsam service state poisoned");
        let cluster =
            state
                .clusters
                .get(&Self::key(dsn))
                .ok_or_else(|| VsamError::DatasetNotFound {
                    dsn: dsn.to_string(),
                })?;
        VsamBackend::open_for(
            &self.root,
            cluster.dataset_id,
            cluster.vsam_type,
            &cluster.params,
        )
    }

    /// Open a backend for an existing handle.
    fn backend_for_handle(&self, handle: &VsamHandle) -> Result<VsamBackend, VsamError> {
        let dsn = {
            let state = self.state.lock().expect("vsam service state poisoned");
            state
                .handles
                .get(&handle.0)
                .cloned()
                .ok_or(VsamError::InvalidHandle)?
        };
        self.backend_for(&dsn)
    }

    fn define(&self, dsn: &str, vsam_type: VsamType, params: VsamParams) -> Result<(), VsamError> {
        let key = Self::key(dsn);
        let dataset_id = Uuid::new_v4();
        // Provision the physical backend eagerly so the cluster exists on disk.
        VsamBackend::open_for(&self.root, dataset_id, vsam_type, &params)?;
        let mut state = self.state.lock().expect("vsam service state poisoned");
        if state.clusters.contains_key(&key) {
            return Err(VsamError::DatasetAlreadyExists {
                dsn: dsn.to_string(),
            });
        }
        state.clusters.insert(
            key,
            ClusterDef {
                dataset_id,
                vsam_type,
                params,
            },
        );
        Ok(())
    }
}

impl VsamService for CatalogVsamService {
    fn create_ksds(
        &self,
        dsn: &str,
        key_length: u16,
        key_offset: u16,
        record_length: u32,
    ) -> Result<(), VsamError> {
        self.define(
            dsn,
            VsamType::Ksds,
            VsamParams {
                key_length: Some(key_length),
                key_offset: Some(key_offset),
                record_length: Some(record_length),
                slot_size: None,
            },
        )
    }

    fn create_esds(&self, dsn: &str, record_length: u32) -> Result<(), VsamError> {
        self.define(
            dsn,
            VsamType::Esds,
            VsamParams {
                record_length: Some(record_length),
                ..VsamParams::default()
            },
        )
    }

    fn create_rrds(&self, dsn: &str, slot_size: u32) -> Result<(), VsamError> {
        self.define(
            dsn,
            VsamType::Rrds,
            VsamParams {
                slot_size: Some(slot_size),
                ..VsamParams::default()
            },
        )
    }

    fn create_lds(&self, dsn: &str) -> Result<(), VsamError> {
        self.define(dsn, VsamType::Lds, VsamParams::default())
    }

    fn destroy_dataset(&self, dsn: &str) -> Result<(), VsamError> {
        let mut state = self.state.lock().expect("vsam service state poisoned");
        if state.clusters.remove(&Self::key(dsn)).is_none() {
            return Err(VsamError::DatasetNotFound {
                dsn: dsn.to_string(),
            });
        }
        Ok(())
    }

    fn initialize_dataset(
        &self,
        dsn: &str,
        vsam_type: VsamType,
        params: VsamParams,
    ) -> Result<(), VsamError> {
        self.define(dsn, vsam_type, params)
    }

    fn open(&self, dsn: &str, _mode: AccessMode) -> Result<VsamHandle, VsamError> {
        // Confirm the cluster exists (and is provisionable) before handing out a
        // handle.
        self.backend_for(dsn)?;
        let mut state = self.state.lock().expect("vsam service state poisoned");
        state.next_handle += 1;
        let id = state.next_handle;
        state.handles.insert(id, Self::key(dsn));
        Ok(VsamHandle(id))
    }

    fn get(&self, handle: &VsamHandle, key: &[u8]) -> Result<Record, VsamError> {
        let backend = self.backend_for_handle(handle)?;
        let dsn = self.dsn_for(handle)?;
        match backend.vsam_type() {
            VsamType::Rrds => backend
                .get_relative(rrn_from_key(key)?)?
                .ok_or(VsamError::RecordNotFound { dsn }),
            VsamType::Esds => backend
                .get_address(rrn_from_key(key)?)?
                .ok_or(VsamError::RecordNotFound { dsn }),
            _ => backend
                .get_keyed(key)?
                .ok_or(VsamError::RecordNotFound { dsn }),
        }
    }

    fn put(&self, handle: &VsamHandle, record: &Record) -> Result<(), VsamError> {
        let backend = self.backend_for_handle(handle)?;
        match backend.vsam_type() {
            VsamType::Rrds => backend.put_relative(rrn_from_key(&record.key)?, &record.data),
            VsamType::Esds => backend.append_entry(&record.data).map(|_| ()),
            _ => backend.put_keyed(record),
        }
    }

    fn delete(&self, handle: &VsamHandle, key: &[u8]) -> Result<(), VsamError> {
        let backend = self.backend_for_handle(handle)?;
        let dsn = self.dsn_for(handle)?;
        let removed = match backend.vsam_type() {
            VsamType::Rrds => backend.delete_relative(rrn_from_key(key)?)?,
            VsamType::Esds => {
                return Err(VsamError::UnsupportedOperation {
                    reason: "ESDS records are deleted by address, not key".to_string(),
                })
            }
            _ => backend.delete_keyed(key)?,
        };
        if removed {
            Ok(())
        } else {
            Err(VsamError::RecordNotFound { dsn })
        }
    }

    fn close(&self, handle: VsamHandle) -> Result<(), VsamError> {
        let mut state = self.state.lock().expect("vsam service state poisoned");
        if state.handles.remove(&handle.0).is_none() {
            return Err(VsamError::InvalidHandle);
        }
        Ok(())
    }

    fn start_browse(
        &self,
        handle: &VsamHandle,
        start_key: &[u8],
        direction: BrowseDirection,
    ) -> Result<BrowseHandle, VsamError> {
        let backend = self.backend_for_handle(handle)?;
        let records = if backend.vsam_type() == VsamType::Ksds && !start_key.is_empty() {
            backend.browse_from(start_key, direction)?
        } else {
            let mut records = backend.sequential()?;
            if direction == BrowseDirection::Backward {
                records.reverse();
            }
            records
        };
        let mut state = self.state.lock().expect("vsam service state poisoned");
        state.next_browse += 1;
        let id = state.next_browse;
        state.browses.insert(id, BrowseState { records, cursor: 0 });
        Ok(BrowseHandle(id))
    }

    fn next_record(&self, browse: &BrowseHandle) -> Result<Option<Record>, VsamError> {
        let mut state = self.state.lock().expect("vsam service state poisoned");
        let browse_state = state
            .browses
            .get_mut(&browse.0)
            .ok_or(VsamError::InvalidBrowseHandle)?;
        if browse_state.cursor >= browse_state.records.len() {
            return Ok(None);
        }
        let record = browse_state.records[browse_state.cursor].clone();
        browse_state.cursor += 1;
        Ok(Some(record))
    }

    fn end_browse(&self, browse: BrowseHandle) -> Result<(), VsamError> {
        let mut state = self.state.lock().expect("vsam service state poisoned");
        if state.browses.remove(&browse.0).is_none() {
            return Err(VsamError::InvalidBrowseHandle);
        }
        Ok(())
    }

    fn define_aix(
        &self,
        base_dsn: &str,
        aix_dsn: &str,
        key_field: KeyField,
    ) -> Result<(), VsamError> {
        let backend = self.backend_for(base_dsn)?;
        if backend.vsam_type() != VsamType::Ksds {
            return Err(VsamError::UnsupportedOperation {
                reason: "alternate indexes are only supported for KSDS".to_string(),
            });
        }
        backend.add_alternate_index(&crate::storage::AlternateIndex {
            name: aix_index_name(aix_dsn),
            offset: u32::from(key_field.offset),
            length: u32::from(key_field.length),
            unique: key_field.unique,
            collation: crate::storage::KeyCollation::Binary,
        })
    }

    fn build_index(&self, aix_dsn: &str) -> Result<(), VsamError> {
        // The alternate index is defined on the base KSDS; `aix_dsn` names the
        // index. Rebuild it on whichever base cluster carries it.
        let name = aix_index_name(aix_dsn);
        let bases: Vec<String> = {
            let state = self.state.lock().expect("vsam service state poisoned");
            state
                .clusters
                .iter()
                .filter(|(_, def)| def.vsam_type == VsamType::Ksds)
                .map(|(dsn, _)| dsn.clone())
                .collect()
        };
        for base in bases {
            let backend = self.backend_for(&base)?;
            if backend.rebuild_alternate_index(&name).is_ok() {
                return Ok(());
            }
        }
        Err(VsamError::UnsupportedOperation {
            reason: format!("alternate index '{name}' is not defined on any KSDS"),
        })
    }
}

impl CatalogVsamService {
    fn dsn_for(&self, handle: &VsamHandle) -> Result<String, VsamError> {
        let state = self.state.lock().expect("vsam service state poisoned");
        state
            .handles
            .get(&handle.0)
            .cloned()
            .ok_or(VsamError::InvalidHandle)
    }
}

/// Decode a key as a big-endian relative record / byte address.
fn rrn_from_key(key: &[u8]) -> Result<u64, VsamError> {
    if key.len() == 8 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(key);
        return Ok(u64::from_be_bytes(bytes));
    }
    std::str::from_utf8(key)
        .ok()
        .and_then(|text| text.trim().parse::<u64>().ok())
        .ok_or_else(|| VsamError::UnsupportedOperation {
            reason: "relative key must be an 8-byte RRN or a decimal string".to_string(),
        })
}

/// Derive the alternate-index registry name from its DSN (last qualifier,
/// uppercased, non-alphanumeric mapped to underscore).
fn aix_index_name(aix_dsn: &str) -> String {
    let tail = aix_dsn.rsplit('.').next().unwrap_or(aix_dsn);
    tail.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "vsam_service_impl_tests.rs"]
mod tests;
