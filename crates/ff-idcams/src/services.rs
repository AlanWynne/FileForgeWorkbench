//! Downstream service wiring and dependency-injection container.
//!
//! CR-CH-059 (Requirement 28): `ff-idcams` no longer carries its OWN private
//! `CatalogService` / `VsamService` trait copies. It now depends on the SINGLE
//! reconciled traits exposed by `ff-dscatalog` (`DynCatalogService`,
//! `VsamService`) and the record-aware `DatasetAccess` contract. `ff-idcams`
//! stays a THIN orchestrator (Requirement 21): it parses commands and routes
//! the dataset work onto those reconciled traits -- it defines no catalog / VSAM
//! trait of its own.
//!
//! A small set of IDCAMS-specific command concepts have no reconciled
//! `ff-dscatalog` trait method (DEFINE PATH, VERIFY, EXPORT, IMPORT). Those stay
//! as orchestration-local param helpers here; the executor performs them as an
//! orchestration-only path and NEVER re-adds a private catalog / VSAM trait.
//! The DD-name resolution step (`AllocatorService`) likewise stays local -- it
//! is the IDCAMS DD->DSN orchestration step, distinct from the catalog
//! authority.

use std::sync::Arc;

use ff_dscatalog::{CatalogDatasetAccess, DatasetAccess, DynCatalogService, VsamService};

use crate::error::AllocatorError;
use crate::parser::ast::DatasetName;

// === Orchestration-local parameter helpers ==================================
//
// These describe IDCAMS command concepts that `ff-dscatalog` does NOT model as
// a trait method. They are plain data the executor consumes while performing
// the operation as an orchestration-only path (Requirement 28, Requirement 21
// thin-orchestrator boundary). They are NOT a re-added private service trait.

/// Parameters for the IDCAMS DEFINE PATH orchestration step.
///
/// `ff-dscatalog` models no PATH trait method; `ff-idcams` performs PATH as an
/// orchestration-only action over the reconciled traits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinePathParams {
    /// Path name.
    pub path_name: DatasetName,
    /// AIX name.
    pub aix_name: DatasetName,
    /// Whether base cluster updates trigger AIX maintenance via this path.
    pub update: bool,
}

/// Result of the IDCAMS VERIFY orchestration step.
///
/// `ff-dscatalog` models no verify-integrity trait method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyResult {
    /// Whether the dataset was consistent.
    pub is_consistent: bool,
    /// Whether corrections were applied.
    pub corrections_applied: bool,
}

/// Parameters for the IDCAMS EXPORT orchestration step.
///
/// `ff-dscatalog` models no export trait method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportParams {
    /// Source dataset name.
    pub source: DatasetName,
    /// Destination path or dataset.
    pub destination: String,
    /// Whether the export is temporary.
    pub temporary: bool,
    /// Whether to inhibit access to source after export.
    pub inhibit_source: bool,
}

/// Result of the IDCAMS EXPORT orchestration step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportResult {
    /// Number of records exported.
    pub record_count: u64,
    /// Total bytes exported.
    pub byte_count: u64,
}

/// Parameters for the IDCAMS IMPORT orchestration step.
///
/// `ff-dscatalog` models no import trait method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportParams {
    /// Input source path.
    pub source: String,
    /// Target dataset name.
    pub target: DatasetName,
    /// Catalog to register in.
    pub catalog: Option<DatasetName>,
}

/// Result of the IDCAMS IMPORT orchestration step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportResult {
    /// Number of records imported.
    pub record_count: u64,
}

// === AllocatorService (orchestration-local) =================================

/// Trait for DD/dataset-name resolution.
///
/// This stays local to `ff-idcams`: it is the IDCAMS DD->DSN orchestration
/// step, not a catalog-authority responsibility. `ff-dscatalog` models dataset
/// resolution via `DatasetAccess::resolve`; the DD-name indirection is IDCAMS's.
pub trait AllocatorService: Send + Sync {
    /// Resolves a DD name to a dataset name.
    fn resolve_dd(&self, ddname: &str) -> Result<DatasetName, AllocatorError>;
}

// === Dependency-injection container =========================================

/// Dependency-injection container for the reconciled downstream services.
///
/// Holds the reconciled `ff-dscatalog` traits (`DynCatalogService`,
/// `VsamService`), the record-aware `DatasetAccess` contract, and the
/// orchestration-local `AllocatorService`. All are `Arc` for safe sharing.
#[derive(Clone)]
pub struct IdcamsServices {
    /// Reconciled catalog service (object-safe `ff-dscatalog` wrapper).
    pub catalog: Arc<dyn DynCatalogService>,
    /// Reconciled VSAM service from `ff-dscatalog`.
    pub vsam: Arc<dyn VsamService>,
    /// Record-aware dataset-access contract from `ff-dscatalog`.
    pub access: Arc<dyn DatasetAccess>,
    /// Orchestration-local allocator for DD resolution.
    pub allocator: Arc<dyn AllocatorService>,
}

impl IdcamsServices {
    /// Creates a new `IdcamsServices` from the reconciled trait implementations.
    pub fn new(
        catalog: Arc<dyn DynCatalogService>,
        vsam: Arc<dyn VsamService>,
        access: Arc<dyn DatasetAccess>,
        allocator: Arc<dyn AllocatorService>,
    ) -> Self {
        Self {
            catalog,
            vsam,
            access,
            allocator,
        }
    }
}

/// Build a default record-aware `DatasetAccess` (an in-memory
/// `CatalogDatasetAccess`) for wiring points that have not supplied one.
///
/// This is the production-shaped record-aware contract from `ff-dscatalog`; it
/// keeps `ff-idcams` free of any direct storage engine (Requirement 21).
pub fn default_dataset_access() -> Arc<dyn DatasetAccess> {
    let dir = std::env::temp_dir().join(format!(
        "ffwb-idcams-access-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::create_dir_all(&dir);
    Arc::new(CatalogDatasetAccess::in_memory(dir))
}

/// Mock implementations of the reconciled services for testing.
pub mod mocks;
