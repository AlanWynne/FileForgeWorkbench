//! `CatalogRepositoryAccess` -- a catalog-backed `DatasetAccess` implementor
//! that resolves and stores against the ON-DISK SQLite catalog of record
//! (CR-CH-061, Requirement 36).
//!
//! Unlike the in-memory [`super::impl_access::CatalogDatasetAccess`] (whose
//! catalog is its own in-process `datasets` HashMap), this type resolves a DSN
//! by opening a SHORT-LIVED [`crate::Catalog`] against the catalog repository
//! path(s) it was constructed with, looking the DSN up, and returning the
//! dataset's physical locator via the catalog + `ff-volume` indirection
//! (Requirement 32). The connection is dropped before `resolve` returns, so the
//! type holds only `Send + Sync` state and satisfies the `DatasetAccess:
//! Send + Sync` bound (Requirement 36.3) -- a `rusqlite::Connection` is NOT
//! `Sync` and is never held.
//!
//! Physical reads/writes go through the single `ff-vfs::StorageProvider` seam
//! (Requirement 36.4): the resolved physical path is the opaque locator the
//! provider reads/writes directly. Records are framed by the existing RECFM
//! codecs (Requirement 16/17), identical record semantics to the in-memory
//! access -- only the resolve SOURCE differs (Requirement 36.5).
//!
//! Validates: dataset-catalog Requirement 36.1, 36.2, 36.3, 36.4, 36.5, 36.6,
//! 36.7, 36.8.

use std::path::PathBuf;
use std::sync::Arc;

use ff_vfs::{PosixNativeProvider, StorageLocator, StorageProvider};

use super::error::DatasetError;
use super::trait_def::DatasetAccess;
use super::types::{
    AccessIntent, DatasetHandle, DdRequest, HandleInner, OpenDataset, Positioner, StepOutcome,
};
use crate::catalog::Catalog;
use crate::codecs::{BinaryCodec, FixedCodec, RecordCodec, VariableCodec};
use crate::dataset::{Dsorg, Recfm};
use crate::dsn::Dsn;
use crate::Record;

/// Default priority for a short-lived catalog mount (single-catalog lookups do
/// not depend on priority ordering).
const MOUNT_PRIORITY: u32 = 1;

// === CatalogRepositoryAccess ===================================================

/// A `DatasetAccess` backed by the on-disk SQLite catalog(s) + physical files of
/// one or more catalog repositories (CR-CH-061, Requirement 36).
///
/// Construct it with the catalog repository path(s) it should resolve DSNs
/// against. All physical I/O flows through an internal non-read-only
/// `ff_vfs::PosixNativeProvider` (the resolved physical path is used as the
/// opaque locator, so the provider reads/writes it directly, Requirement 36.4).
///
/// The type holds ONLY `Send + Sync` state (the repository paths + an
/// `Arc<dyn StorageProvider>`); it opens a short-lived `Catalog` per `resolve`
/// call and drops it before returning, so the shared `Arc<dyn DatasetAccess>`
/// stays `Send + Sync` (Requirement 36.3).
pub struct CatalogRepositoryAccess {
    /// The catalog repository root path(s) to resolve DSNs against, in search
    /// order. A `resolve` tries each until one catalogs the DSN.
    repository_paths: Vec<PathBuf>,
    /// The single physical seam (Requirement 36.4). A non-read-only native
    /// provider; `open`/`write` use the absolute physical locator directly, so
    /// the provider root is irrelevant to resolution.
    provider: Arc<dyn StorageProvider>,
}

impl CatalogRepositoryAccess {
    /// Create a catalog-backed access over the given repository path(s).
    ///
    /// The physical seam is an internal non-read-only `PosixNativeProvider`;
    /// resolved physical paths are absolute, so the provider root only anchors
    /// relative operations that this access never performs.
    pub fn new(repository_paths: Vec<PathBuf>) -> Self {
        // The native provider's root is unused for absolute-locator reads/writes;
        // anchor it at the first repository path (or the current dir) so the
        // provider is well-formed.
        let root = repository_paths
            .first()
            .cloned()
            .unwrap_or_else(|| PathBuf::from("."));
        let provider: Arc<dyn StorageProvider> = Arc::new(PosixNativeProvider::new(root, false));
        Self {
            repository_paths,
            provider,
        }
    }

    /// Create a catalog-backed access over a single repository path.
    pub fn single(repository_path: impl Into<PathBuf>) -> Self {
        Self::new(vec![repository_path.into()])
    }

    /// Create a catalog-backed access with an explicit storage provider (the
    /// physical seam). Lets a consumer or test inject a provider; the default
    /// [`Self::new`] constructs a native one.
    pub fn with_provider(
        repository_paths: Vec<PathBuf>,
        provider: Arc<dyn StorageProvider>,
    ) -> Self {
        Self {
            repository_paths,
            provider,
        }
    }

    /// Select the RECFM codec for a handle (Requirement 36.5). Record boundaries
    /// always come from the codec, never a host text line -- identical to the
    /// in-memory access's selection.
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

    /// Resolve a DSN against the on-disk catalog(s) to a physical locator +
    /// record shape, opening a SHORT-LIVED `Catalog` per repository and dropping
    /// it before returning (Requirement 36.2, 36.3).
    ///
    /// Returns `DatasetError::NotFound` when no repository catalogs the DSN,
    /// preserving a true miss (Requirement 36.6).
    fn resolve_impl(&self, dsn: &str) -> Result<DatasetHandle, DatasetError> {
        let parsed = Dsn::parse(dsn).map_err(DatasetError::from)?;
        let key = parsed.as_str().to_string();

        for path in &self.repository_paths {
            // Open a short-lived catalog; a repository that fails to mount is
            // skipped (the DSN may live in another repository).
            let catalog = match Catalog::mount(path, MOUNT_PRIORITY) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let record = match catalog.lookup(&parsed) {
                Ok(r) => r,
                Err(_) => continue,
            };
            // Resolve the physical path via the catalog + ff-volume indirection
            // (Requirement 36.4). The returned path is the opaque locator.
            let physical = catalog
                .resolve_locator(&record)
                .map_err(DatasetError::from)?;
            let locator = physical.to_string_lossy().into_owned();
            let (recfm, lrecl) = record_shape(&record);
            // Catalog + connection drop here as `catalog` leaves scope.
            return Ok(DatasetHandle {
                inner: HandleInner {
                    dsn: key,
                    // No ff-volume VolumeId is tracked by this access; the
                    // locator is self-sufficient for physical I/O.
                    volume_id: ff_volume::VolumeId(0),
                    locator,
                    recfm,
                    lrecl,
                    dsorg: record.dsorg,
                },
            });
        }

        Err(DatasetError::NotFound { dsn: key })
    }

    /// Open a resolved dataset for the given intent: read current bytes through
    /// the single provider seam and decode via the RECFM codec (Requirement
    /// 36.4, 36.5). A fresh (empty) object decodes to zero records.
    fn open_impl(
        &self,
        handle: &DatasetHandle,
        intent: AccessIntent,
    ) -> Result<OpenDataset, DatasetError> {
        let locator = StorageLocator::new(handle.inner.locator.clone());
        let bytes = self.provider.open(&locator).map_err(DatasetError::from)?;
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

    /// Flush pending writes through the codec and persist via the single
    /// provider seam (Requirement 36.4, 36.5). The stored bytes are the
    /// RECFM-framed record image written to the dataset's catalog physical file.
    fn close_impl(&self, open: OpenDataset) -> Result<(), DatasetError> {
        if !open.dirty {
            return Ok(());
        }
        let handle = &open.handle;
        let mut records = if open.intent == AccessIntent::Write {
            Vec::new()
        } else {
            open.records.clone()
        };
        records.extend(open.pending.iter().cloned());

        let codec = Self::codec_for(handle);
        let bytes = codec.encode(&records)?;
        let locator = StorageLocator::new(handle.inner.locator.clone());
        self.provider
            .write(&locator, &bytes)
            .map_err(DatasetError::from)
    }
}

/// Map a `DatasetRecord`'s record attributes to the codec shape the handle
/// carries. A catalog PS dataset defaults to FB/80 when attributes are omitted
/// (mirroring `AllocParams::with_defaults`).
fn record_shape(record: &crate::dataset::DatasetRecord) -> (Recfm, u32) {
    let recfm = record.recfm.unwrap_or(Recfm::FB);
    let lrecl = record.lrecl.unwrap_or(80);
    (recfm, lrecl)
}

// === DatasetAccess trait impl ===================================================

impl DatasetAccess for CatalogRepositoryAccess {
    /// Allocation is the Files Panel / allocator flow's responsibility; the
    /// catalog-backed access resolves and stores an already-catalogued dataset.
    fn allocate(&self, dd: &DdRequest) -> Result<DatasetHandle, DatasetError> {
        Err(DatasetError::NotYetWired {
            operation: format!("allocate {} (use the catalog allocate flow)", dd.dsn),
        })
    }

    fn resolve(&self, dsn: &str, _intent: AccessIntent) -> Result<DatasetHandle, DatasetError> {
        self.resolve_impl(dsn)
    }

    fn open(
        &self,
        handle: &DatasetHandle,
        intent: AccessIntent,
    ) -> Result<OpenDataset, DatasetError> {
        self.open_impl(handle, intent)
    }

    fn get(&self, open: &mut OpenDataset) -> Result<Option<Record>, DatasetError> {
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
        Ok(Some(Record {
            key: Vec::new(),
            data,
        }))
    }

    fn put(&self, open: &mut OpenDataset, record: &Record) -> Result<(), DatasetError> {
        if open.intent == AccessIntent::Read {
            return Err(DatasetError::InvalidIntent {
                reason: "cannot put on a dataset opened for read".to_string(),
            });
        }
        open.pending.push(record.data.clone());
        open.dirty = true;
        Ok(())
    }

    fn point(&self, _open: &mut OpenDataset, positioner: &Positioner) -> Result<(), DatasetError> {
        // The catalog-backed access serves sequential (PS) datasets; a
        // keyed/relative positioner is not supported through this access.
        let _ = positioner;
        Err(DatasetError::BadPositioner {
            reason: "positioner not supported for catalog-backed sequential access".to_string(),
        })
    }

    fn close(&self, open: OpenDataset) -> Result<(), DatasetError> {
        self.close_impl(open)
    }

    fn dispose(&self, handle: DatasetHandle, outcome: StepOutcome) -> Result<(), DatasetError> {
        // Disposition is a catalog-metadata operation owned by the catalog
        // layer; the record-aware SAVE path does not dispose.
        let _ = (handle, outcome);
        Ok(())
    }
}

// === Compile-time Send + Sync assertion (Requirement 36.3) ===================

/// Compile-time guard that `CatalogRepositoryAccess` is `Send + Sync` so it can
/// back the mainframe CE's `Arc<dyn DatasetAccess>` (Requirement 36.3). A field
/// that was not `Send + Sync` (e.g. a held `rusqlite::Connection`) would fail to
/// compile this reference.
#[allow(dead_code)]
fn _assert_send_sync<T: Send + Sync>() {}

#[allow(dead_code)]
fn _assert_catalog_access_send_sync() {
    _assert_send_sync::<CatalogRepositoryAccess>();
}
