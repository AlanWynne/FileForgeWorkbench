//! Catalog resolution bridge -- trait abstraction and mock implementation.
//!
//! The `CatalogProvider` trait abstracts catalog access for testability.
//! Production delegates to `ff-dscatalog` (through its reconciled
//! `CatalogService` trait, dataset-catalog Requirement 33); tests use
//! `MockCatalog`. The `DatasetAllocator` trait wraps
//! `ff_dscatalog::DatasetAccess::allocate` so a live allocation returns an
//! opaque `ff_dscatalog::DatasetHandle` rather than a raw physical path
//! (dataset-allocator Requirement 19.1, 19.2, 19.3). The allocator keeps NO
//! direct SQLite access -- it depends on the catalog only through these mockable
//! traits (Requirement 19.4).

use ff_dscatalog::DatasetHandle;

use crate::operands::{DcbAttributes, SpaceAllocation};

/// Errors from catalog operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CatalogError {
    /// Catalog query failed.
    #[error("catalog '{catalog}' query failed: {detail}")]
    QueryFailed {
        /// Name of the catalog that failed.
        catalog: String,
        /// Detail of the failure.
        detail: String,
    },

    /// Catalog is not mounted.
    #[error("catalog '{catalog}' is not mounted")]
    NotMounted {
        /// Name of the catalog.
        catalog: String,
    },

    /// Allocation failed.
    #[error("allocation failed in catalog '{catalog}': {detail}")]
    AllocationFailed {
        /// Name of the catalog.
        catalog: String,
        /// Detail of the failure.
        detail: String,
    },
}

/// Dataset type returned from catalog resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogDatasetType {
    /// Physical sequential.
    Ps,
    /// Partitioned (PDS/PDSE).
    Po,
    /// Generation Data Group.
    Gdg,
}

/// A single match from a catalog lookup.
#[derive(Debug, Clone)]
pub struct CatalogMatch {
    /// Name of the catalog that contains this dataset.
    pub catalog_name: String,
    /// Physical file path.
    pub physical_path: String,
    /// Dataset type.
    pub dataset_type: CatalogDatasetType,
}

/// GDG information from catalog.
#[derive(Debug, Clone)]
pub struct GdgInfo {
    /// GDG base name.
    pub base_name: String,
    /// Maximum number of generations.
    pub limit: u32,
    /// Active generations (ordered newest-first).
    pub generations: Vec<GdgGeneration>,
}

/// A single GDG generation entry.
#[derive(Debug, Clone)]
pub struct GdgGeneration {
    /// Absolute generation number.
    pub number: u32,
    /// Full generation dataset name.
    pub dsn: String,
    /// Physical path.
    pub physical_path: String,
}

/// Trait abstracting catalog access for testability.
///
/// Production implementation delegates to `ff-dscatalog`.
/// Test implementations can provide canned responses.
pub trait CatalogProvider: Send + Sync {
    /// Look up a DSN in mounted catalogs.
    /// Returns all matches across catalogs (caller applies search order).
    fn lookup_dsn(&self, dsn: &str) -> Result<Vec<CatalogMatch>, CatalogError>;

    /// Verify that a PDS member exists within a resolved PDS.
    fn verify_member(&self, pds_dsn: &str, member: &str) -> Result<bool, CatalogError>;

    /// Query GDG state for a base name.
    /// Returns generation list (ordered newest-first).
    fn query_gdg(&self, base_name: &str) -> Result<Option<GdgInfo>, CatalogError>;

    /// Allocate a new dataset in the catalog (live mode only).
    fn allocate_dataset(
        &self,
        dsn: &str,
        attributes: &DcbAttributes,
        space: Option<&SpaceAllocation>,
    ) -> Result<String, CatalogError>;

    /// Check if a DSN already exists in any mounted catalog.
    fn dataset_exists(&self, dsn: &str) -> Result<bool, CatalogError>;
}

/// A resolved allocation request handed to a `DatasetAllocator`.
///
/// The allocator owns the JCL DISP/SPACE/DCB parse; this request is the
/// already-parsed form it passes to the `DatasetAccess` seam. It is a thin
/// local DTO so `ff-dsalloc` need not reconstruct `ff_dscatalog::DdRequest`
/// (which carries Volume-layer types) at the trait boundary.
#[derive(Debug, Clone)]
pub struct DatasetAllocationRequest {
    /// The dataset name to allocate.
    pub dsn: String,
    /// DCB attributes (RECFM/LRECL/BLKSIZE).
    pub attributes: DcbAttributes,
    /// The SPACE request, if any.
    pub space: Option<SpaceAllocation>,
}

/// Trait wrapping `ff_dscatalog::DatasetAccess::allocate` for the allocator
/// (dataset-allocator Requirement 19.2, 19.3).
///
/// Production wires this to a concrete `DatasetAccess`; tests substitute
/// `MockDatasetAllocator`. The returned handle is opaque -- the allocator and
/// its downstream consumers (the JES executor) hold it instead of a physical
/// path (Requirement 19.3).
pub trait DatasetAllocator: Send + Sync {
    /// Acquire a `DatasetHandle` for a live (DISP=NEW) allocation.
    fn allocate(&self, request: &DatasetAllocationRequest) -> Result<DatasetHandle, CatalogError>;
}

/// Mock allocator returning a canned opaque handle for tests.
///
/// It drives the real `ff_dscatalog::DatasetAccess` over an in-memory Volume
/// and a native storage provider so the handle it yields is a genuine opaque
/// `DatasetHandle`, proving the seam without a live catalog database.
pub struct MockDatasetAllocator {
    access: ff_dscatalog::CatalogDatasetAccess,
}

impl Default for MockDatasetAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for MockDatasetAllocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockDatasetAllocator").finish()
    }
}

impl MockDatasetAllocator {
    /// Create a mock allocator backed by an in-memory Volume and a transient
    /// native storage provider.
    pub fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "ffwb-dsalloc-mock-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        Self {
            access: ff_dscatalog::CatalogDatasetAccess::in_memory(dir),
        }
    }
}

impl DatasetAllocator for MockDatasetAllocator {
    fn allocate(&self, request: &DatasetAllocationRequest) -> Result<DatasetHandle, CatalogError> {
        self.access
            .allocate_sequential(
                &request.dsn,
                ff_dscatalog::Recfm::FB,
                request.attributes.lrecl.unwrap_or(80),
                request.attributes.blksize.unwrap_or(0),
                1,
                1,
                16,
            )
            .map_err(|e| CatalogError::AllocationFailed {
                catalog: "mock".to_string(),
                detail: e.to_string(),
            })
    }
}

/// Mock catalog for testing -- provides canned responses.
#[derive(Debug, Clone, Default)]
pub struct MockCatalog {
    /// Datasets available in the mock catalog.
    pub datasets: Vec<CatalogMatch>,
    /// Members available in PDS datasets.
    pub members: std::collections::HashMap<String, Vec<String>>,
    /// GDG definitions.
    pub gdgs: std::collections::HashMap<String, GdgInfo>,
    /// Allocated datasets (DSN -> path).
    pub allocated: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl CatalogProvider for MockCatalog {
    fn lookup_dsn(&self, dsn: &str) -> Result<Vec<CatalogMatch>, CatalogError> {
        let matches: Vec<CatalogMatch> = self
            .datasets
            .iter()
            .filter(|_m| self.has_dsn_entry(dsn))
            .cloned()
            .collect();

        Ok(matches)
    }

    fn verify_member(&self, pds_dsn: &str, member: &str) -> Result<bool, CatalogError> {
        let upper_dsn = pds_dsn.to_uppercase();
        let upper_member = member.to_uppercase();
        Ok(self
            .members
            .get(&upper_dsn)
            .map(|members| members.iter().any(|m| m.to_uppercase() == upper_member))
            .unwrap_or(false))
    }

    fn query_gdg(&self, base_name: &str) -> Result<Option<GdgInfo>, CatalogError> {
        Ok(self.gdgs.get(&base_name.to_uppercase()).cloned())
    }

    fn allocate_dataset(
        &self,
        dsn: &str,
        _attributes: &DcbAttributes,
        _space: Option<&SpaceAllocation>,
    ) -> Result<String, CatalogError> {
        let path = format!("/data/{}", dsn.to_lowercase().replace('.', "/"));
        self.allocated.lock().unwrap().push(dsn.to_string());
        Ok(path)
    }

    fn dataset_exists(&self, dsn: &str) -> Result<bool, CatalogError> {
        Ok(self.has_dsn_entry(dsn))
    }
}

impl MockCatalog {
    /// Create a new empty mock catalog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dataset entry to the mock catalog.
    pub fn add_dataset(&mut self, dsn: &str, path: &str, dtype: CatalogDatasetType, catalog: &str) {
        self.datasets.push(CatalogMatch {
            catalog_name: catalog.to_string(),
            physical_path: path.to_string(),
            dataset_type: dtype,
        });
        // Also store by DSN for lookup
        self.members
            .entry(format!("__DSN__{}", dsn.to_uppercase()))
            .or_default();
    }

    /// Add a PDS with members.
    pub fn add_pds_with_members(
        &mut self,
        pds_dsn: &str,
        members: &[&str],
        path: &str,
        catalog: &str,
    ) {
        self.add_dataset(pds_dsn, path, CatalogDatasetType::Po, catalog);
        self.members.insert(
            pds_dsn.to_uppercase(),
            members.iter().map(|m| m.to_uppercase()).collect(),
        );
    }

    /// Check if a DSN entry exists.
    fn has_dsn_entry(&self, dsn: &str) -> bool {
        let key = format!("__DSN__{}", dsn.to_uppercase());
        self.members.contains_key(&key)
    }
}

/// Helper to extract a DSN-like string from a path (not used in production).
#[allow(dead_code)]
fn extract_dsn_from_path(path: &str) -> String {
    path.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_catalog_lookup_finds_existing_dataset() {
        // Validates: Requirement 2 AC 1, AC 2
        let mut catalog = MockCatalog::new();
        catalog.add_dataset(
            "MY.DATA.SET",
            "/data/my/data/set",
            CatalogDatasetType::Ps,
            "PROD.CATALOG",
        );

        assert!(catalog.dataset_exists("MY.DATA.SET").unwrap());
        assert!(!catalog.dataset_exists("NO.SUCH.SET").unwrap());
    }

    #[test]
    fn mock_catalog_member_verification() {
        // Validates: Requirement 2 AC 5, AC 6
        let mut catalog = MockCatalog::new();
        catalog.add_pds_with_members(
            "MY.PDS",
            &["MEMBER1", "MEMBER2"],
            "/data/my/pds",
            "CATALOG1",
        );

        assert!(catalog.verify_member("MY.PDS", "MEMBER1").unwrap());
        assert!(!catalog.verify_member("MY.PDS", "NOTHERE").unwrap());
    }

    #[test]
    fn mock_catalog_gdg_query() {
        // Validates: Requirement 8 AC 2
        let mut catalog = MockCatalog::new();
        catalog.gdgs.insert(
            "MY.GDG.BASE".to_string(),
            GdgInfo {
                base_name: "MY.GDG.BASE".to_string(),
                limit: 5,
                generations: vec![
                    GdgGeneration {
                        number: 3,
                        dsn: "MY.GDG.BASE.G0003V00".to_string(),
                        physical_path: "/data/g3".to_string(),
                    },
                    GdgGeneration {
                        number: 2,
                        dsn: "MY.GDG.BASE.G0002V00".to_string(),
                        physical_path: "/data/g2".to_string(),
                    },
                ],
            },
        );

        let info = catalog.query_gdg("MY.GDG.BASE").unwrap().unwrap();
        assert_eq!(info.generations.len(), 2);
        assert_eq!(info.generations[0].number, 3);
    }

    #[test]
    fn mock_catalog_allocation() {
        // Validates: Requirement 4 AC 1
        let catalog = MockCatalog::new();
        let attrs = DcbAttributes::hardcoded_defaults();
        let path = catalog
            .allocate_dataset("NEW.DATA.SET", &attrs, None)
            .unwrap();
        assert!(!path.is_empty());
        assert_eq!(catalog.allocated.lock().unwrap().len(), 1);
    }

    #[test]
    fn dataset_allocator_yields_opaque_handle_via_dataset_access() {
        // Validates: Requirement 19.2, 19.3 -- the allocator seam returns an
        // opaque ff_dscatalog::DatasetHandle, not a physical path.
        let allocator = MockDatasetAllocator::new();
        let request = DatasetAllocationRequest {
            dsn: "NEW.HANDLE.DS".to_string(),
            attributes: DcbAttributes::hardcoded_defaults(),
            space: None,
        };
        let handle = allocator
            .allocate(&request)
            .expect("allocate via DatasetAccess");
        // The only observable facet is the DSN; no raw path is exposed.
        assert_eq!(handle.dsn(), "NEW.HANDLE.DS");
    }

    #[test]
    fn dataset_allocator_is_object_safe_as_dyn() {
        // Validates: Requirement 19.4 -- the allocator is mockable behind a dyn
        // trait (no direct SQLite in ff-dsalloc).
        let allocator: Box<dyn DatasetAllocator> = Box::new(MockDatasetAllocator::new());
        let request = DatasetAllocationRequest {
            dsn: "DYN.ALLOC.DS".to_string(),
            attributes: DcbAttributes::hardcoded_defaults(),
            space: None,
        };
        assert_eq!(
            allocator.allocate(&request).expect("dyn allocate").dsn(),
            "DYN.ALLOC.DS"
        );
    }
}
