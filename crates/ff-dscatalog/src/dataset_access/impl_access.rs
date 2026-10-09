//! Concrete `CatalogDatasetAccess` -- the working non-VSAM `DatasetAccess`
//! implementation (CR-CH-059, Requirement 34).
//!
//! This impl composes the existing seams: it resolves a dataset's physical
//! location through `ff-volume` (`DatasetVolumeSet` -> `ResolvedVolume` ->
//! locator) honouring the Volume Online check, charges SPACE against the Volume
//! through `ExtentSet`, and performs all physical I/O through the single
//! `ff_vfs::StorageProvider` seam. The PS/PO sequential + member paths are wired
//! end-to-end; the VSAM keyed/relative record ops are DEFINED with their seam
//! reachable but return `DatasetError::NotYetWired` (RC.B.7 owns the concrete
//! VSAM wiring).
//!
//! Validates: dataset-catalog Requirement 34.1, 34.3, 34.4, 34.5, 34.6;
//! volume-model Requirement 12.1, 12.2, 12.3, 12.4.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use ff_vfs::{StorageLocator, StorageProvider};
use ff_volume::{
    DatasetId, DatasetVolume, DatasetVolumeSet, ExtentSet, GeometryProfile, VolumeId,
    VolumeRegistry,
};

use super::error::DatasetError;
use super::types::{AccessIntent, DatasetHandle, DdRequest, HandleInner, StepOutcome};

// === Per-dataset state ===================================================

/// The Volume-binding and extent state the access layer tracks for one dataset.
pub(crate) struct DatasetState {
    pub(crate) dataset_id: DatasetId,
    pub(crate) volumes: DatasetVolumeSet,
    pub(crate) extents: ExtentSet,
    pub(crate) geometry: GeometryProfile,
    pub(crate) alloc: ff_volume::AllocationUnit,
    pub(crate) max_extents: u32,
    pub(crate) catalogued: bool,
    /// Record format snapshot, so an opaque handle can be reconstructed on
    /// `resolve` without re-reading a DD request.
    pub(crate) recfm: crate::dataset::Recfm,
    /// Logical record length snapshot.
    pub(crate) lrecl: u32,
    /// Dataset organization snapshot.
    pub(crate) dsorg: crate::dataset::Dsorg,
}

/// Interior-mutable state behind the `&self` trait surface.
pub(crate) struct AccessState {
    pub(crate) registry: VolumeRegistry,
    pub(crate) datasets: HashMap<String, DatasetState>,
    pub(crate) next_dataset_id: u64,
}

// === CatalogDatasetAccess ===================================================

/// A concrete `DatasetAccess` backed by `ff-volume` + the record codecs + the
/// single `ff_vfs::StorageProvider` physical seam.
///
/// Construct it with a storage provider and a seeded `VolumeRegistry`; it owns
/// the DatasetVolume / extent accounting for datasets it allocates. The
/// provider is the ONE physical seam (Requirement 34.3) -- no raw `storage_path`
/// or SQLite handle appears in the surface.
pub struct CatalogDatasetAccess {
    pub(crate) provider: Arc<dyn StorageProvider>,
    pub(crate) state: Mutex<AccessState>,
    /// Root directory under which the VSAM record backends (KSDS/ESDS/RRDS)
    /// store their per-cluster physical objects. VSAM record I/O cannot flow
    /// through the byte-only `StorageProvider` seam, so the keyed/relative
    /// backends are rooted here (Requirement 35.3).
    pub(crate) vsam_root: PathBuf,
}

impl CatalogDatasetAccess {
    /// Create an access layer over `provider` and `registry`.
    ///
    /// The VSAM record backends are rooted in a process-unique directory under
    /// the system temp dir; use [`Self::with_vsam_root`] to pin a specific root.
    pub fn new(provider: Arc<dyn StorageProvider>, registry: VolumeRegistry) -> Self {
        let vsam_root = std::env::temp_dir().join(format!("ffwb-vsam-{}", uuid::Uuid::new_v4()));
        Self::with_vsam_root(provider, registry, vsam_root)
    }

    /// Create an access layer with an explicit VSAM backend root.
    pub fn with_vsam_root(
        provider: Arc<dyn StorageProvider>,
        registry: VolumeRegistry,
        vsam_root: PathBuf,
    ) -> Self {
        Self {
            provider,
            state: Mutex::new(AccessState {
                registry,
                datasets: HashMap::new(),
                next_dataset_id: 1,
            }),
            vsam_root,
        }
    }

    /// Create a self-contained in-memory access layer backed by a transient
    /// native storage provider under `storage_dir` and a single large Online
    /// ReadWrite Volume (id 1).
    ///
    /// This spares consumers (e.g. `ff-dsalloc`, which depends only on
    /// `ff-dscatalog`) from naming the `ff-volume` / `ff-vfs` types to obtain an
    /// opaque `DatasetHandle`, keeping the dependency DAG `ff-dsalloc ->
    /// ff-dscatalog` intact.
    pub fn in_memory(storage_dir: impl AsRef<std::path::Path>) -> Self {
        let provider = Arc::new(ff_vfs::PosixNativeProvider::new(
            storage_dir.as_ref().to_path_buf(),
            false,
        ));
        let mut registry = VolumeRegistry::new();
        let volume = ff_volume::Volume::new(
            VolumeId(1),
            ff_volume::Volser::try_new("MEM001").expect("in-memory volser"),
            "in-memory-storage-uri",
            100_000,
        );
        registry.define(volume).expect("define in-memory volume");
        Self::new(provider, registry)
    }

    /// The default Volume id used by `in_memory`.
    pub fn in_memory_volume_id() -> VolumeId {
        VolumeId(1)
    }

    /// Convenience allocate for a sequential (PS) dataset expressed in
    /// primitive terms, targeting the `in_memory` default Volume. Lets a
    /// consumer that depends only on `ff-dscatalog` acquire an opaque
    /// `DatasetHandle` without naming the `ff-volume` SPACE/geometry types.
    ///
    /// `primary_tracks` / `secondary_tracks` are a TRK SPACE request.
    ///
    /// # Errors
    /// As `DatasetAccess::allocate`.
    // The wide primitive signature is deliberate: it lets a caller that depends
    // only on ff-dscatalog build an allocation without naming the ff-volume
    // SPACE/geometry types, which is the whole point of this convenience.
    #[allow(clippy::too_many_arguments)]
    pub fn allocate_sequential(
        &self,
        dsn: &str,
        recfm: crate::dataset::Recfm,
        lrecl: u32,
        blksize: u32,
        primary_tracks: u64,
        secondary_tracks: u64,
        max_extents: u32,
    ) -> Result<DatasetHandle, DatasetError> {
        let dd = DdRequest {
            dsn: dsn.to_string(),
            dsorg: crate::dataset::Dsorg::PS,
            recfm,
            lrecl,
            blksize,
            space: ff_volume::AllocationUnit::new(
                ff_volume::UnitKind::Trk,
                primary_tracks,
                secondary_tracks,
            ),
            geometry: GeometryProfile::default(),
            volume_id: VolumeId(1),
            max_extents,
        };
        self.allocate_impl(&dd)
    }

    /// Resolve an existing dataset by name to an opaque `DatasetHandle`
    /// (dataset-allocator Requirement 19.3). A public convenience so a consumer
    /// that depends only on `ff-dscatalog` (e.g. `ff-dsalloc`) can resolve a
    /// DSN without naming the `ff-volume` locator types or importing the
    /// `DatasetAccess` trait.
    ///
    /// # Errors
    /// As `DatasetAccess::resolve`.
    pub fn resolve_existing(
        &self,
        dsn: &str,
        intent: AccessIntent,
    ) -> Result<DatasetHandle, DatasetError> {
        self.resolve_impl(dsn, intent)
    }

    /// Normalise a DSN to the catalog's uppercase form.
    pub(crate) fn key(dsn: &str) -> String {
        dsn.trim().to_uppercase()
    }

    /// Build an opaque handle snapshot from a DD request and resolved locator.
    pub(crate) fn handle_from(dd: &DdRequest, locator: String) -> DatasetHandle {
        DatasetHandle {
            inner: HandleInner {
                dsn: Self::key(&dd.dsn),
                volume_id: dd.volume_id,
                locator,
                recfm: dd.recfm,
                lrecl: dd.lrecl,
                dsorg: dd.dsorg,
            },
        }
    }

    /// Resolve an existing dataset by name to an opaque `DatasetHandle`,
    /// honouring the Volume Online check (dataset-allocator Requirement 19.3,
    /// dataset-catalog Requirement 34.5). The `intent` is accepted for parity
    /// with `open`/`allocate`; resolution exposes no raw path.
    pub(crate) fn resolve_impl(
        &self,
        dsn: &str,
        _intent: AccessIntent,
    ) -> Result<DatasetHandle, DatasetError> {
        let key = Self::key(dsn);
        let state = self.state.lock().expect("access state poisoned");
        let ds = state
            .datasets
            .get(&key)
            .ok_or_else(|| DatasetError::NotFound { dsn: key.clone() })?;
        // Resolve through ff-volume, honouring the Online check (volume-model
        // 2.4) -- the SAME seam open/allocate use. A failed resolve (e.g. an
        // Offline Volume) surfaces as the mapped DatasetError.
        let resolved = ds.volumes.resolve(&state.registry)?;
        let primary = resolved
            .into_iter()
            .find(|r| r.sequence_number == 1)
            .ok_or_else(|| DatasetError::NotFound { dsn: key.clone() })?;
        Ok(DatasetHandle {
            inner: HandleInner {
                dsn: key,
                volume_id: primary.volume.volume_id(),
                locator: primary.locator,
                recfm: ds.recfm,
                lrecl: ds.lrecl,
                dsorg: ds.dsorg,
            },
        })
    }

    /// Resolve a handle's locator as a `StorageLocator` through the Volume
    /// Online check (Requirement 34.3, volume-model 12.1).
    pub(crate) fn resolve_locator(
        &self,
        handle: &DatasetHandle,
    ) -> Result<StorageLocator, DatasetError> {
        let state = self.state.lock().expect("access state poisoned");
        let ds = state
            .datasets
            .get(&handle.inner.dsn)
            .ok_or_else(|| DatasetError::NotFound {
                dsn: handle.inner.dsn.clone(),
            })?;
        // Resolve through ff-volume, honouring the Online check (volume-model 2.4).
        let resolved = ds.volumes.resolve(&state.registry)?;
        let primary = resolved
            .into_iter()
            .find(|r| r.sequence_number == 1)
            .ok_or_else(|| DatasetError::NotFound {
                dsn: handle.inner.dsn.clone(),
            })?;
        Ok(StorageLocator::new(primary.locator))
    }
}

// === allocate / dispose ===================================================

impl CatalogDatasetAccess {
    /// Shared implementation of `DatasetAccess::allocate`.
    pub(crate) fn allocate_impl(&self, dd: &DdRequest) -> Result<DatasetHandle, DatasetError> {
        let key = Self::key(&dd.dsn);
        let mut state = self.state.lock().expect("access state poisoned");
        if state.datasets.contains_key(&key) {
            return Err(DatasetError::AlreadyExists { dsn: key });
        }

        // Allocate the physical object through the single StorageProvider seam
        // (Requirement 34.3). The provider returns an opaque locator.
        let locator = self.provider.allocate(&key).map_err(DatasetError::from)?;

        // Charge SPACE against the Volume through the extent model
        // (Requirement 34.4, volume-model 12.4). This surfaces VolumeFull /
        // VolumeReadOnly / VolumeOffline unchanged via the From<VolumeError>.
        let dataset_id = DatasetId(state.next_dataset_id);
        state.next_dataset_id += 1;

        let mut extents = ExtentSet::new();
        {
            let volume = state.registry.find_by_id_mut(dd.volume_id).ok_or_else(|| {
                DatasetError::VolumeUnavailable {
                    volser: format!("volume_id {}", dd.volume_id.0),
                }
            })?;
            extents.allocate_primary(dataset_id, &dd.space, &dd.geometry, volume)?;
        }

        let mut volumes = DatasetVolumeSet::new();
        volumes.add(DatasetVolume {
            dataset_id,
            volume_id: dd.volume_id,
            sequence_number: 1,
            is_primary: true,
            locator: locator.as_str().to_string(),
        });

        state.datasets.insert(
            key,
            DatasetState {
                dataset_id,
                volumes,
                extents,
                geometry: dd.geometry,
                alloc: dd.space,
                max_extents: dd.max_extents,
                catalogued: true,
                recfm: dd.recfm,
                lrecl: dd.lrecl,
                dsorg: dd.dsorg,
            },
        );

        Ok(Self::handle_from(dd, locator.as_str().to_string()))
    }

    /// Shared implementation of `DatasetAccess::dispose`.
    pub(crate) fn dispose_impl(
        &self,
        handle: DatasetHandle,
        outcome: StepOutcome,
    ) -> Result<(), DatasetError> {
        let key = handle.inner.dsn.clone();
        match outcome {
            // KEEP / CATLG / PASS retain the dataset and its bytes.
            StepOutcome::Keep | StepOutcome::Catlg | StepOutcome::Pass => {
                let mut state = self.state.lock().expect("access state poisoned");
                if let Some(ds) = state.datasets.get_mut(&key) {
                    ds.catalogued = true;
                }
                Ok(())
            }
            // UNCATLG drops the catalog association but keeps the bytes.
            StepOutcome::Uncatlg => {
                let mut state = self.state.lock().expect("access state poisoned");
                if let Some(ds) = state.datasets.get_mut(&key) {
                    ds.catalogued = false;
                }
                Ok(())
            }
            // DELETE removes the physical object and releases the extents.
            StepOutcome::Delete => {
                let locator = self.resolve_locator(&handle)?;
                self.provider.delete(&locator).map_err(DatasetError::from)?;
                let mut state = self.state.lock().expect("access state poisoned");
                if let Some(ds) = state.datasets.remove(&key) {
                    release_extents(&mut state.registry, &ds);
                }
                Ok(())
            }
        }
    }
}

/// Release a dataset's charged tracks back to its Volume on DELETE.
fn release_extents(registry: &mut VolumeRegistry, ds: &DatasetState) {
    let tracks = ds.extents.allocated_tracks();
    if tracks == 0 {
        return;
    }
    let volume_id: VolumeId = ds
        .volumes
        .rows()
        .first()
        .map(|r| r.volume_id)
        .unwrap_or(VolumeId(0));
    if let Some(volume) = registry.find_by_id_mut(volume_id) {
        volume.release(tracks);
    }
}
