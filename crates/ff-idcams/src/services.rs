//! Downstream service trait definitions and dependency injection container.
//!
//! All downstream operations are accessed through these trait interfaces.
//! ff-idcams never depends on concrete implementations — only these traits.

use std::sync::Arc;

use crate::error::{AllocatorError, CatalogError, VsamError};
use crate::parser::ast::{
    DatasetName, DeleteEntryType, DisplayLevel, ListcatFilter, SpaceUnit, VsamOrganization,
};

// ─── Catalog Service Parameters ─────────────────────────────────────────────

/// Parameters for creating a dataset in the catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateDatasetParams {
    /// The dataset name.
    pub name: DatasetName,
    /// The dataset organization (KSDS, ESDS, RRDS, LDS).
    pub dsorg: VsamOrganization,
    /// Volume serials.
    pub volumes: Vec<String>,
    /// Space allocation.
    pub space: Option<SpaceUnit>,
    /// Record size (average, maximum).
    pub recordsize: Option<(u32, u32)>,
    /// Key definition (length, offset).
    pub keys: Option<(u16, u32)>,
    /// Free space (CI percent, CA percent).
    pub freespace: Option<(u8, u8)>,
    /// Share options (cross-region, cross-system).
    pub shareoptions: Option<(u8, u8)>,
    /// Buffer space in bytes.
    pub bufferspace: Option<u32>,
    /// Whether the dataset is reusable.
    pub reuse: bool,
}

/// Parameters for creating a GDG base.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateGdgParams {
    /// The GDG base name.
    pub name: DatasetName,
    /// Maximum generations (1-255).
    pub limit: u8,
    /// Whether rolled-off generations are physically deleted.
    pub scratch: bool,
    /// Whether all generations are rolled off when limit exceeded.
    pub empty: bool,
    /// Whether FIFO ordering is used (false = LIFO).
    pub fifo: bool,
}

/// Attributes for updating a dataset.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UpdateAttrs {
    /// New free space settings.
    pub freespace: Option<(u8, u8)>,
    /// New share options.
    pub shareoptions: Option<(u8, u8)>,
    /// New buffer space.
    pub bufferspace: Option<u32>,
    /// New record size.
    pub recordsize: Option<(u32, u32)>,
    /// New keys.
    pub keys: Option<(u16, u32)>,
    /// Volumes to add.
    pub add_volumes: Vec<String>,
    /// Volumes to remove.
    pub remove_volumes: Vec<String>,
    /// Attributes to nullify/reset.
    pub nullify: Vec<String>,
}

/// Filter for listing catalog entries.
#[derive(Debug, Clone, PartialEq)]
pub struct ListFilter {
    /// The filter pattern.
    pub filter: ListcatFilter,
    /// Entry type filter.
    pub entry_type: Option<DeleteEntryType>,
    /// Display level.
    pub display_level: DisplayLevel,
}

/// A catalog entry returned by list operations.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogEntry {
    /// The entry name.
    pub name: DatasetName,
    /// The entry type.
    pub entry_type: String,
    /// Organization.
    pub dsorg: Option<VsamOrganization>,
}

/// Full dataset attributes for detailed display.
#[derive(Debug, Clone, PartialEq)]
pub struct DatasetAttributes {
    /// The entry name.
    pub name: DatasetName,
    /// The entry type.
    pub entry_type: String,
    /// Organization.
    pub dsorg: Option<VsamOrganization>,
    /// Record count.
    pub record_count: u64,
    /// Creation date.
    pub creation_date: Option<String>,
    /// Last access date.
    pub last_access_date: Option<String>,
    /// Key length.
    pub key_length: Option<u16>,
    /// Key offset (RKP).
    pub key_offset: Option<u32>,
    /// Average record length.
    pub avg_record_length: Option<u32>,
    /// Maximum record length.
    pub max_record_length: Option<u32>,
    /// Volumes.
    pub volumes: Vec<String>,
}

/// Export operation parameters.
#[derive(Debug, Clone, PartialEq)]
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

/// Export result.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportResult {
    /// Number of records exported.
    pub record_count: u64,
    /// Total bytes exported.
    pub byte_count: u64,
}

/// Import operation parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportParams {
    /// Input source path.
    pub source: String,
    /// Target dataset name.
    pub target: DatasetName,
    /// Catalog to register in.
    pub catalog: Option<DatasetName>,
}

/// Import result.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportResult {
    /// Number of records imported.
    pub record_count: u64,
}

// ─── VSAM Service Parameters ────────────────────────────────────────────────

/// VSAM type for initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VsamType {
    /// Key Sequenced Data Set.
    Ksds,
    /// Entry Sequenced Data Set.
    Esds,
    /// Relative Record Data Set.
    Rrds,
    /// Linear Data Set.
    Lds,
}

/// Parameters for VSAM dataset initialization.
#[derive(Debug, Clone, PartialEq)]
pub struct VsamInitParams {
    /// Key definition (length, offset) — required for KSDS.
    pub keys: Option<(u16, u32)>,
    /// Record size (average, maximum).
    pub recordsize: Option<(u32, u32)>,
    /// Control interval size.
    pub ci_size: Option<u32>,
}

/// Parameters for defining an alternate index.
#[derive(Debug, Clone, PartialEq)]
pub struct DefineAixParams {
    /// AIX dataset name.
    pub aix_name: DatasetName,
    /// Base cluster name.
    pub base_cluster: DatasetName,
    /// Key definition (length, offset).
    pub keys: (u16, u32),
    /// Whether keys must be unique.
    pub unique_key: bool,
    /// Whether AIX is auto-maintained on updates.
    pub upgrade: bool,
    /// Record size for AIX.
    pub recordsize: Option<(u32, u32)>,
}

/// Parameters for defining a path.
#[derive(Debug, Clone, PartialEq)]
pub struct DefinePathParams {
    /// Path name.
    pub path_name: DatasetName,
    /// AIX name.
    pub aix_name: DatasetName,
    /// Whether base cluster updates trigger AIX maintenance via this path.
    pub update: bool,
}

/// Dataset handle for open datasets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetHandle {
    /// Internal identifier.
    pub id: String,
}

/// Open mode for datasets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    /// Read-only access.
    Input,
    /// Write access.
    Output,
}

/// Browse position specification.
#[derive(Debug, Clone, PartialEq)]
pub enum BrowsePosition {
    /// Start from the beginning.
    Start,
    /// Start from a specific key.
    Key(String),
    /// Start from a specific RBA.
    Address(u64),
    /// Start from a specific record number.
    RecordNumber(u64),
}

/// A browse cursor for sequential record access.
#[derive(Debug, Clone, PartialEq)]
pub struct BrowseCursor {
    /// Internal state.
    pub position: u64,
    /// Whether more records are available.
    pub has_more: bool,
}

/// A record from a VSAM dataset.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// The record data bytes.
    pub data: Vec<u8>,
    /// The record key (if KSDS).
    pub key: Option<Vec<u8>>,
}

/// Result of a verify operation.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifyResult {
    /// Whether the dataset was consistent.
    pub is_consistent: bool,
    /// Whether corrections were applied.
    pub corrections_applied: bool,
}

/// Result of a build index operation.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildIndexResult {
    /// Number of index entries created.
    pub entries_created: u64,
    /// Number of duplicate keys found (for UNIQUEKEY AIX).
    pub duplicates_found: u64,
}

// ─── Trait Definitions ──────────────────────────────────────────────────────

/// Trait for catalog operations — implemented by ff-dataset-catalog.
///
/// ff-idcams depends on this trait only, never on the concrete implementation.
pub trait CatalogService: Send + Sync {
    /// Creates a new dataset entry in the catalog.
    fn create_dataset(&self, params: CreateDatasetParams) -> Result<(), CatalogError>;

    /// Deletes a dataset entry from the catalog.
    fn delete_dataset(&self, dsn: &DatasetName) -> Result<(), CatalogError>;

    /// Updates attributes of an existing dataset.
    fn update_dataset(&self, dsn: &DatasetName, attrs: UpdateAttrs) -> Result<(), CatalogError>;

    /// Renames a dataset in the catalog.
    fn rename_dataset(&self, old: &DatasetName, new: &DatasetName) -> Result<(), CatalogError>;

    /// Lists datasets matching the given filter.
    fn list_datasets(&self, filter: &ListFilter) -> Result<Vec<CatalogEntry>, CatalogError>;

    /// Gets full attributes for a dataset.
    fn get_dataset_attributes(&self, dsn: &DatasetName) -> Result<DatasetAttributes, CatalogError>;

    /// Creates a GDG base entry.
    fn create_gdg_base(&self, params: CreateGdgParams) -> Result<(), CatalogError>;

    /// Deletes a GDG base entry.
    fn delete_gdg_base(&self, dsn: &DatasetName, force: bool) -> Result<(), CatalogError>;

    /// Exports a dataset to a portable format.
    fn export_dataset(&self, params: ExportParams) -> Result<ExportResult, CatalogError>;

    /// Imports a dataset from a portable format.
    fn import_dataset(&self, params: ImportParams) -> Result<ImportResult, CatalogError>;
}

/// Trait for VSAM operations — implemented by ff-vsam-services.
///
/// ff-idcams depends on this trait only, never on the concrete implementation.
pub trait VsamService: Send + Sync {
    /// Initializes a new VSAM dataset.
    fn initialize_dataset(
        &self,
        dsn: &DatasetName,
        vtype: VsamType,
        params: VsamInitParams,
    ) -> Result<(), VsamError>;

    /// Destroys a VSAM dataset and all its structures.
    fn destroy_dataset(&self, dsn: &DatasetName) -> Result<(), VsamError>;

    /// Defines an alternate index.
    fn define_aix(&self, params: DefineAixParams) -> Result<(), VsamError>;

    /// Defines a path connecting an AIX to its base cluster.
    fn define_path(&self, params: DefinePathParams) -> Result<(), VsamError>;

    /// Deletes a path.
    fn delete_path(&self, path_name: &DatasetName) -> Result<(), VsamError>;

    /// Verifies dataset integrity.
    fn verify_integrity(&self, dsn: &DatasetName) -> Result<VerifyResult, VsamError>;

    /// Builds or rebuilds an alternate index from base cluster records.
    fn build_index(
        &self,
        base_dsn: &DatasetName,
        aix_dsn: &DatasetName,
    ) -> Result<BuildIndexResult, VsamError>;

    /// Opens a dataset for reading or writing.
    fn open(&self, dsn: &DatasetName, mode: OpenMode) -> Result<DatasetHandle, VsamError>;

    /// Starts a browse operation from the given position.
    fn start_browse(
        &self,
        handle: &DatasetHandle,
        position: BrowsePosition,
    ) -> Result<BrowseCursor, VsamError>;

    /// Retrieves the next record from a browse cursor.
    fn next_record(&self, cursor: &mut BrowseCursor) -> Result<Option<Record>, VsamError>;

    /// Writes a record to a dataset.
    fn put(&self, handle: &DatasetHandle, record: &Record) -> Result<(), VsamError>;
}

/// Trait for DD/dataset allocation resolution — implemented by ff-dataset-allocator.
pub trait AllocatorService: Send + Sync {
    /// Resolves a DD name to a dataset name.
    fn resolve_dd(&self, ddname: &str) -> Result<DatasetName, AllocatorError>;
}

// ─── Dependency Injection Container ─────────────────────────────────────────

/// Dependency injection container for all downstream services.
///
/// Holds trait objects for catalog, VSAM, and allocator services.
/// All services are wrapped in `Arc` for safe sharing across threads.
pub struct IdcamsServices {
    /// Catalog service for dataset management.
    pub catalog: Arc<dyn CatalogService>,
    /// VSAM service for VSAM operations.
    pub vsam: Arc<dyn VsamService>,
    /// Allocator service for DD resolution.
    pub allocator: Arc<dyn AllocatorService>,
}

impl IdcamsServices {
    /// Creates a new `IdcamsServices` with the given trait implementations.
    pub fn new(
        catalog: Arc<dyn CatalogService>,
        vsam: Arc<dyn VsamService>,
        allocator: Arc<dyn AllocatorService>,
    ) -> Self {
        Self {
            catalog,
            vsam,
            allocator,
        }
    }
}

// ─── Test Helpers ───────────────────────────────────────────────────────────

/// Mock implementations for testing.
pub mod mocks {
    use super::*;
    use std::sync::Mutex;

    /// A configurable mock catalog service for testing.
    pub struct MockCatalogService {
        /// Responses to return for create_dataset calls.
        pub create_responses: Mutex<Vec<Result<(), CatalogError>>>,
        /// Responses to return for delete_dataset calls.
        pub delete_responses: Mutex<Vec<Result<(), CatalogError>>>,
        /// Responses to return for update_dataset calls.
        pub update_responses: Mutex<Vec<Result<(), CatalogError>>>,
        /// Responses to return for rename_dataset calls.
        pub rename_responses: Mutex<Vec<Result<(), CatalogError>>>,
        /// Responses to return for list_datasets calls.
        pub list_responses: Mutex<Vec<Result<Vec<CatalogEntry>, CatalogError>>>,
        /// Responses to return for get_dataset_attributes calls.
        pub attr_responses: Mutex<Vec<Result<DatasetAttributes, CatalogError>>>,
        /// Responses for create_gdg_base.
        pub create_gdg_responses: Mutex<Vec<Result<(), CatalogError>>>,
        /// Responses for delete_gdg_base.
        pub delete_gdg_responses: Mutex<Vec<Result<(), CatalogError>>>,
        /// Responses for export_dataset.
        pub export_responses: Mutex<Vec<Result<ExportResult, CatalogError>>>,
        /// Responses for import_dataset.
        pub import_responses: Mutex<Vec<Result<ImportResult, CatalogError>>>,
    }

    impl MockCatalogService {
        /// Creates a new mock that returns success for everything.
        pub fn new_success() -> Self {
            Self {
                create_responses: Mutex::new(vec![Ok(())]),
                delete_responses: Mutex::new(vec![Ok(())]),
                update_responses: Mutex::new(vec![Ok(())]),
                rename_responses: Mutex::new(vec![Ok(())]),
                list_responses: Mutex::new(vec![Ok(vec![])]),
                attr_responses: Mutex::new(vec![]),
                create_gdg_responses: Mutex::new(vec![Ok(())]),
                delete_gdg_responses: Mutex::new(vec![Ok(())]),
                export_responses: Mutex::new(vec![Ok(ExportResult {
                    record_count: 0,
                    byte_count: 0,
                })]),
                import_responses: Mutex::new(vec![Ok(ImportResult { record_count: 0 })]),
            }
        }

        fn pop_or_default<T: Clone>(responses: &Mutex<Vec<T>>, default: T) -> T {
            let mut guard = responses.lock().unwrap();
            if guard.is_empty() {
                default
            } else {
                guard.remove(0)
            }
        }
    }

    impl CatalogService for MockCatalogService {
        fn create_dataset(&self, _params: CreateDatasetParams) -> Result<(), CatalogError> {
            Self::pop_or_default(&self.create_responses, Ok(()))
        }

        fn delete_dataset(&self, _dsn: &DatasetName) -> Result<(), CatalogError> {
            Self::pop_or_default(&self.delete_responses, Ok(()))
        }

        fn update_dataset(
            &self,
            _dsn: &DatasetName,
            _attrs: UpdateAttrs,
        ) -> Result<(), CatalogError> {
            Self::pop_or_default(&self.update_responses, Ok(()))
        }

        fn rename_dataset(
            &self,
            _old: &DatasetName,
            _new: &DatasetName,
        ) -> Result<(), CatalogError> {
            Self::pop_or_default(&self.rename_responses, Ok(()))
        }

        fn list_datasets(&self, _filter: &ListFilter) -> Result<Vec<CatalogEntry>, CatalogError> {
            Self::pop_or_default(&self.list_responses, Ok(vec![]))
        }

        fn get_dataset_attributes(
            &self,
            _dsn: &DatasetName,
        ) -> Result<DatasetAttributes, CatalogError> {
            Self::pop_or_default(
                &self.attr_responses,
                Err(CatalogError::NotFound("mock".to_string())),
            )
        }

        fn create_gdg_base(&self, _params: CreateGdgParams) -> Result<(), CatalogError> {
            Self::pop_or_default(&self.create_gdg_responses, Ok(()))
        }

        fn delete_gdg_base(&self, _dsn: &DatasetName, _force: bool) -> Result<(), CatalogError> {
            Self::pop_or_default(&self.delete_gdg_responses, Ok(()))
        }

        fn export_dataset(&self, _params: ExportParams) -> Result<ExportResult, CatalogError> {
            Self::pop_or_default(
                &self.export_responses,
                Ok(ExportResult {
                    record_count: 0,
                    byte_count: 0,
                }),
            )
        }

        fn import_dataset(&self, _params: ImportParams) -> Result<ImportResult, CatalogError> {
            Self::pop_or_default(&self.import_responses, Ok(ImportResult { record_count: 0 }))
        }
    }

    /// A configurable mock VSAM service for testing.
    pub struct MockVsamService {
        /// Responses for initialize_dataset.
        pub init_responses: Mutex<Vec<Result<(), VsamError>>>,
        /// Responses for destroy_dataset.
        pub destroy_responses: Mutex<Vec<Result<(), VsamError>>>,
        /// Responses for define_aix.
        pub define_aix_responses: Mutex<Vec<Result<(), VsamError>>>,
        /// Responses for define_path.
        pub define_path_responses: Mutex<Vec<Result<(), VsamError>>>,
        /// Responses for delete_path.
        pub delete_path_responses: Mutex<Vec<Result<(), VsamError>>>,
        /// Responses for verify_integrity.
        pub verify_responses: Mutex<Vec<Result<VerifyResult, VsamError>>>,
        /// Responses for build_index.
        pub build_index_responses: Mutex<Vec<Result<BuildIndexResult, VsamError>>>,
        /// Responses for open.
        pub open_responses: Mutex<Vec<Result<DatasetHandle, VsamError>>>,
        /// Responses for start_browse.
        pub browse_responses: Mutex<Vec<Result<BrowseCursor, VsamError>>>,
        /// Records to return from next_record.
        pub records: Mutex<Vec<Option<Record>>>,
        /// Responses for put.
        pub put_responses: Mutex<Vec<Result<(), VsamError>>>,
    }

    impl MockVsamService {
        /// Creates a new mock that returns success for everything.
        pub fn new_success() -> Self {
            Self {
                init_responses: Mutex::new(vec![Ok(())]),
                destroy_responses: Mutex::new(vec![Ok(())]),
                define_aix_responses: Mutex::new(vec![Ok(())]),
                define_path_responses: Mutex::new(vec![Ok(())]),
                delete_path_responses: Mutex::new(vec![Ok(())]),
                verify_responses: Mutex::new(vec![Ok(VerifyResult {
                    is_consistent: true,
                    corrections_applied: false,
                })]),
                build_index_responses: Mutex::new(vec![Ok(BuildIndexResult {
                    entries_created: 0,
                    duplicates_found: 0,
                })]),
                open_responses: Mutex::new(vec![Ok(DatasetHandle {
                    id: "mock".to_string(),
                })]),
                browse_responses: Mutex::new(vec![Ok(BrowseCursor {
                    position: 0,
                    has_more: false,
                })]),
                records: Mutex::new(vec![None]),
                put_responses: Mutex::new(vec![Ok(())]),
            }
        }

        fn pop_or_default<T: Clone>(responses: &Mutex<Vec<T>>, default: T) -> T {
            let mut guard = responses.lock().unwrap();
            if guard.is_empty() {
                default
            } else {
                guard.remove(0)
            }
        }
    }

    impl VsamService for MockVsamService {
        fn initialize_dataset(
            &self,
            _dsn: &DatasetName,
            _vtype: VsamType,
            _params: VsamInitParams,
        ) -> Result<(), VsamError> {
            Self::pop_or_default(&self.init_responses, Ok(()))
        }

        fn destroy_dataset(&self, _dsn: &DatasetName) -> Result<(), VsamError> {
            Self::pop_or_default(&self.destroy_responses, Ok(()))
        }

        fn define_aix(&self, _params: DefineAixParams) -> Result<(), VsamError> {
            Self::pop_or_default(&self.define_aix_responses, Ok(()))
        }

        fn define_path(&self, _params: DefinePathParams) -> Result<(), VsamError> {
            Self::pop_or_default(&self.define_path_responses, Ok(()))
        }

        fn delete_path(&self, _path_name: &DatasetName) -> Result<(), VsamError> {
            Self::pop_or_default(&self.delete_path_responses, Ok(()))
        }

        fn verify_integrity(&self, _dsn: &DatasetName) -> Result<VerifyResult, VsamError> {
            Self::pop_or_default(
                &self.verify_responses,
                Ok(VerifyResult {
                    is_consistent: true,
                    corrections_applied: false,
                }),
            )
        }

        fn build_index(
            &self,
            _base_dsn: &DatasetName,
            _aix_dsn: &DatasetName,
        ) -> Result<BuildIndexResult, VsamError> {
            Self::pop_or_default(
                &self.build_index_responses,
                Ok(BuildIndexResult {
                    entries_created: 0,
                    duplicates_found: 0,
                }),
            )
        }

        fn open(&self, _dsn: &DatasetName, _mode: OpenMode) -> Result<DatasetHandle, VsamError> {
            Self::pop_or_default(
                &self.open_responses,
                Ok(DatasetHandle {
                    id: "mock".to_string(),
                }),
            )
        }

        fn start_browse(
            &self,
            _handle: &DatasetHandle,
            _position: BrowsePosition,
        ) -> Result<BrowseCursor, VsamError> {
            Self::pop_or_default(
                &self.browse_responses,
                Ok(BrowseCursor {
                    position: 0,
                    has_more: false,
                }),
            )
        }

        fn next_record(&self, _cursor: &mut BrowseCursor) -> Result<Option<Record>, VsamError> {
            let mut guard = self.records.lock().unwrap();
            if guard.is_empty() {
                Ok(None)
            } else {
                Ok(guard.remove(0))
            }
        }

        fn put(&self, _handle: &DatasetHandle, _record: &Record) -> Result<(), VsamError> {
            Self::pop_or_default(&self.put_responses, Ok(()))
        }
    }

    /// A mock allocator service for testing.
    pub struct MockAllocatorService {
        /// Responses for resolve_dd.
        pub resolve_responses: Mutex<Vec<Result<DatasetName, AllocatorError>>>,
    }

    impl MockAllocatorService {
        /// Creates a mock that always fails with DD not found.
        pub fn new_not_found() -> Self {
            Self {
                resolve_responses: Mutex::new(vec![]),
            }
        }
    }

    impl AllocatorService for MockAllocatorService {
        fn resolve_dd(&self, ddname: &str) -> Result<DatasetName, AllocatorError> {
            let mut guard = self.resolve_responses.lock().unwrap();
            if guard.is_empty() {
                Err(AllocatorError::DdNotFound(ddname.to_string()))
            } else {
                guard.remove(0)
            }
        }
    }

    /// Builder for constructing test `IdcamsServices` with mock implementations.
    pub struct TestServicesBuilder {
        catalog: Option<Arc<dyn CatalogService>>,
        vsam: Option<Arc<dyn VsamService>>,
        allocator: Option<Arc<dyn AllocatorService>>,
    }

    impl TestServicesBuilder {
        /// Creates a new builder.
        pub fn new() -> Self {
            Self {
                catalog: None,
                vsam: None,
                allocator: None,
            }
        }

        /// Sets the catalog service.
        pub fn with_catalog(mut self, catalog: impl CatalogService + 'static) -> Self {
            self.catalog = Some(Arc::new(catalog));
            self
        }

        /// Sets the VSAM service.
        pub fn with_vsam(mut self, vsam: impl VsamService + 'static) -> Self {
            self.vsam = Some(Arc::new(vsam));
            self
        }

        /// Sets the allocator service.
        pub fn with_allocator(mut self, allocator: impl AllocatorService + 'static) -> Self {
            self.allocator = Some(Arc::new(allocator));
            self
        }

        /// Builds the services, using default success mocks for unset services.
        pub fn build(self) -> IdcamsServices {
            IdcamsServices {
                catalog: self
                    .catalog
                    .unwrap_or_else(|| Arc::new(MockCatalogService::new_success())),
                vsam: self
                    .vsam
                    .unwrap_or_else(|| Arc::new(MockVsamService::new_success())),
                allocator: self
                    .allocator
                    .unwrap_or_else(|| Arc::new(MockAllocatorService::new_not_found())),
            }
        }
    }

    impl Default for TestServicesBuilder {
        fn default() -> Self {
            Self::new()
        }
    }
}
