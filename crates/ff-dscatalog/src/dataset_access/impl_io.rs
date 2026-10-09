//! Record I/O for `CatalogDatasetAccess`: open / get / put / point / close, and
//! the `DatasetAccess` trait impl (CR-CH-059, Requirement 34).
//!
//! The sequential (PS/PO) paths are wired end-to-end over the record codecs and
//! the single `ff_vfs::StorageProvider` seam. The VSAM keyed/relative paths are
//! wired over the concrete `VsamBackend` (KSDS/ESDS/RRDS): `get`/`put`/`point`
//! delegate to the VSAM record path (see `impl_vsam.rs`) when the open dataset
//! carries VSAM positioning state.
//!
//! Validates: dataset-catalog Requirement 34.1, 34.2, 34.3, 35.3; volume-model
//! Requirement 12.1, 12.2, 12.4.

use ff_volume::DatasetId;

use super::error::DatasetError;
use super::impl_access::CatalogDatasetAccess;
use super::trait_def::DatasetAccess;
use super::types::{AccessIntent, DatasetHandle, DdRequest, OpenDataset, Positioner, StepOutcome};
use crate::codecs::{BinaryCodec, FixedCodec, RecordCodec, VariableCodec};
use crate::dataset::{Dsorg, Recfm};
use crate::Record;

impl CatalogDatasetAccess {
    /// Select the RECFM codec for a handle (Requirement 34.2). Record
    /// boundaries always come from the codec, never a host text line.
    fn codec_for(handle: &DatasetHandle) -> Box<dyn RecordCodec> {
        match handle.inner.recfm {
            Recfm::F | Recfm::FB => Box::new(FixedCodec::new(
                handle.inner.lrecl as usize,
                handle.inner.dsn.clone(),
            )),
            Recfm::V | Recfm::VB => Box::new(VariableCodec::new(handle.inner.dsn.clone())),
            Recfm::U => Box::new(BinaryCodec),
        }
    }

    /// True when a positioner is keyed/relative (VSAM). VSAM is NOT a `Dsorg`
    /// variant; the keyed/relative record op is driven by `point` over the
    /// cluster's `VsamBackend`.
    fn is_vsam_positioner(positioner: &Positioner) -> bool {
        matches!(positioner, Positioner::Key(_) | Positioner::Rrn(_))
    }

    /// Shared `open` implementation for the sequential paths (Requirement 34.1,
    /// 34.3).
    fn open_impl(
        &self,
        handle: &DatasetHandle,
        intent: AccessIntent,
    ) -> Result<OpenDataset, DatasetError> {
        // Resolve the physical locator through ff-volume (Online-checked).
        let locator = self.resolve_locator(handle)?;

        // Read current bytes through the single StorageProvider seam and decode
        // to records via the RECFM codec (Requirement 34.2, 34.3). A fresh
        // (empty) object decodes to zero records.
        let bytes = match self.provider.open(&locator) {
            Ok(b) => b,
            Err(e) => return Err(DatasetError::from(e)),
        };
        let codec = Self::codec_for(handle);
        let records = codec.decode(&bytes)?;

        Ok(OpenDataset {
            handle: handle.clone(),
            intent,
            records,
            cursor: 0,
            pending: Vec::new(),
            dirty: false,
            vsam: None,
        })
    }

    /// Shared `get` implementation (Requirement 34.2).
    fn get_impl(&self, open: &mut OpenDataset) -> Result<Option<Record>, DatasetError> {
        if open.vsam.is_some() {
            return self.get_vsam(open);
        }
        if open.handle.inner.dsorg == Dsorg::GDG {
            return Err(DatasetError::BadPositioner {
                reason: "GDG base has no sequential records".to_string(),
            });
        }
        if open.cursor >= open.records.len() {
            return Ok(None);
        }
        let data = open.records[open.cursor].clone();
        open.cursor += 1;
        // Non-keyed records carry an empty key (consistent with ESDS Record).
        Ok(Some(Record {
            key: Vec::new(),
            data,
        }))
    }

    /// Shared `put` implementation (Requirement 34.2).
    fn put_impl(&self, open: &mut OpenDataset, record: &Record) -> Result<(), DatasetError> {
        if open.vsam.is_some() {
            return self.put_vsam(open, record);
        }
        if open.intent == AccessIntent::Read {
            return Err(DatasetError::InvalidIntent {
                reason: "cannot put on a dataset opened for read".to_string(),
            });
        }
        open.pending.push(record.data.clone());
        open.dirty = true;
        Ok(())
    }

    /// Shared `close` implementation: flush pending writes through the codec and
    /// charge any growth against the Volume (Requirement 34.2, 34.4).
    fn close_impl(&self, open: OpenDataset) -> Result<(), DatasetError> {
        if !open.dirty {
            return Ok(());
        }
        let handle = &open.handle;
        let locator = self.resolve_locator(handle)?;

        // For OUTPUT (Write) the pending records replace content; for Update we
        // append pending to the already-read records.
        let mut records = if open.intent == AccessIntent::Write {
            Vec::new()
        } else {
            open.records.clone()
        };
        records.extend(open.pending.iter().cloned());

        let codec = Self::codec_for(handle);
        let bytes = codec.encode(&records)?;

        // Charge the resulting size against the Volume so growth surfaces the
        // Volume failures unchanged (Requirement 34.4, volume-model 12.4).
        {
            let mut state = self.state.lock().expect("access state poisoned");
            // Disjoint field borrows: the dataset map and the registry.
            let state = &mut *state;
            let dataset = state.datasets.get_mut(&handle.inner.dsn).ok_or_else(|| {
                DatasetError::NotFound {
                    dsn: handle.inner.dsn.clone(),
                }
            })?;
            let geom = dataset.geometry;
            let alloc = dataset.alloc;
            let max_extents = dataset.max_extents;
            let dataset_id: DatasetId = dataset.dataset_id;
            let volume_id = dataset
                .volumes
                .rows()
                .first()
                .map(|r| r.volume_id)
                .ok_or_else(|| DatasetError::NotFound {
                    dsn: handle.inner.dsn.clone(),
                })?;
            let volume = state.registry.find_by_id_mut(volume_id).ok_or_else(|| {
                DatasetError::VolumeUnavailable {
                    volser: format!("volume_id {}", volume_id.0),
                }
            })?;
            dataset.extents.accommodate(
                dataset_id,
                bytes.len() as u64,
                &alloc,
                &geom,
                volume,
                max_extents,
            )?;
        }

        // Persist the bytes through the single StorageProvider seam.
        self.provider
            .write(&locator, &bytes)
            .map_err(DatasetError::from)?;
        Ok(())
    }

    /// Shared `point` implementation. For a VSAM open the keyed/relative
    /// positioner is established over the cluster's backend (Requirement 34.1,
    /// 35.3); a positioner on a sequential dataset is rejected.
    fn point_impl(
        &self,
        open: &mut OpenDataset,
        positioner: &Positioner,
    ) -> Result<(), DatasetError> {
        if open.vsam.is_some() {
            return self.point_vsam(open, positioner);
        }
        if Self::is_vsam_positioner(positioner) {
            return Err(DatasetError::BadPositioner {
                reason: "keyed/relative positioner requires a VSAM cluster open".to_string(),
            });
        }
        Err(DatasetError::BadPositioner {
            reason: "positioner not supported for sequential dataset".to_string(),
        })
    }
}

// === DatasetAccess trait impl ===================================================

impl DatasetAccess for CatalogDatasetAccess {
    fn allocate(&self, dd: &DdRequest) -> Result<DatasetHandle, DatasetError> {
        self.allocate_impl(dd)
    }

    fn resolve(&self, dsn: &str, intent: AccessIntent) -> Result<DatasetHandle, DatasetError> {
        self.resolve_impl(dsn, intent)
    }

    fn open(
        &self,
        handle: &DatasetHandle,
        intent: AccessIntent,
    ) -> Result<OpenDataset, DatasetError> {
        self.open_impl(handle, intent)
    }

    fn get(&self, open: &mut OpenDataset) -> Result<Option<Record>, DatasetError> {
        self.get_impl(open)
    }

    fn put(&self, open: &mut OpenDataset, record: &Record) -> Result<(), DatasetError> {
        self.put_impl(open, record)
    }

    fn point(&self, open: &mut OpenDataset, positioner: &Positioner) -> Result<(), DatasetError> {
        self.point_impl(open, positioner)
    }

    fn close(&self, open: OpenDataset) -> Result<(), DatasetError> {
        self.close_impl(open)
    }

    fn dispose(&self, handle: DatasetHandle, outcome: StepOutcome) -> Result<(), DatasetError> {
        self.dispose_impl(handle, outcome)
    }
}
