//! Configurable mock implementations of the reconciled `ff-dscatalog` services.
//!
//! CR-CH-059 (Requirement 28.2): the test doubles implement the RECONCILED
//! `ff-dscatalog` traits (`CatalogService`, `VsamService`) over `ff-dscatalog`'s
//! own types (`&str` DSNs, `DatasetAttributes`, `VsamHandle`, `Record`, ...).
//! They never reintroduce the divergent local types that were removed from
//! `ff-idcams`.
//!
//! Each mock exposes response queues so a test can inject a specific
//! success/error outcome per call; an empty queue falls back to a sensible
//! default (success, or `NotImplemented`-free success for the record paths).

use std::sync::{Arc, Mutex};

use ff_dscatalog::{
    AccessMode, BrowseDirection, BrowseHandle, CatalogError, CatalogService, DatasetAttributes,
    DatasetEntry, DatasetFilter, DatasetId, DsnValidationError, DynCatalogService, GenerationInfo,
    KeyField, Record, ResolutionResult, VsamError, VsamHandle, VsamParams, VsamService, VsamType,
};
use ff_dscatalog::{Dsorg, Recfm};

use super::{default_dataset_access, AllocatorService, IdcamsServices};
use crate::error::AllocatorError;
use crate::parser::ast::DatasetName;

/// Pop the next queued response, or produce a default via `make_default`.
///
/// Takes ownership from the queue (the reconciled `CatalogError` / `VsamError`
/// are not `Clone`), so each queued response is consumed exactly once.
fn pop_or<T>(responses: &Mutex<Vec<T>>, make_default: impl FnOnce() -> T) -> T {
    let mut guard = responses.lock().expect("mock response lock poisoned");
    if guard.is_empty() {
        make_default()
    } else {
        guard.remove(0)
    }
}

// === MockCatalogService =====================================================

/// A configurable mock of the reconciled `CatalogService`.
pub struct MockCatalogService {
    /// Responses for `create_dataset`.
    pub create_responses: Mutex<Vec<Result<DatasetId, CatalogError>>>,
    /// Responses for `delete_dataset`.
    pub delete_responses: Mutex<Vec<Result<(), CatalogError>>>,
    /// Responses for `update_dataset`.
    pub update_responses: Mutex<Vec<Result<(), CatalogError>>>,
    /// Responses for `rename_dataset`.
    pub rename_responses: Mutex<Vec<Result<(), CatalogError>>>,
    /// Responses for `list_datasets`.
    pub list_responses: Mutex<Vec<Result<Vec<DatasetEntry>, CatalogError>>>,
    /// Responses for `get_dataset_attributes`.
    pub attr_responses: Mutex<Vec<Result<DatasetAttributes, CatalogError>>>,
    /// Responses for `create_gdg_base`.
    pub create_gdg_responses: Mutex<Vec<Result<(), CatalogError>>>,
}

impl MockCatalogService {
    /// Creates a mock that returns success for everything.
    pub fn new_success() -> Self {
        Self {
            create_responses: Mutex::new(vec![]),
            delete_responses: Mutex::new(vec![]),
            update_responses: Mutex::new(vec![]),
            rename_responses: Mutex::new(vec![]),
            list_responses: Mutex::new(vec![]),
            attr_responses: Mutex::new(vec![]),
            create_gdg_responses: Mutex::new(vec![]),
        }
    }
}

impl CatalogService for MockCatalogService {
    fn create_dataset(
        &self,
        _dsn: &str,
        _attrs: DatasetAttributes,
    ) -> Result<DatasetId, CatalogError> {
        pop_or(&self.create_responses, || {
            Ok(DatasetId("MOCK-001".to_string()))
        })
    }

    fn delete_dataset(&self, _dsn: &str) -> Result<(), CatalogError> {
        pop_or(&self.delete_responses, || Ok(()))
    }

    fn update_dataset(&self, _dsn: &str, _attrs: DatasetAttributes) -> Result<(), CatalogError> {
        pop_or(&self.update_responses, || Ok(()))
    }

    fn rename_dataset(&self, _old_dsn: &str, _new_dsn: &str) -> Result<(), CatalogError> {
        pop_or(&self.rename_responses, || Ok(()))
    }

    fn resolve_dsn(&self, _dsn: &str) -> Result<ResolutionResult, CatalogError> {
        Ok(ResolutionResult::default())
    }

    fn dataset_exists(&self, _dsn: &str) -> Result<bool, CatalogError> {
        Ok(true)
    }

    fn get_dataset_attributes(&self, dsn: &str) -> Result<DatasetAttributes, CatalogError> {
        pop_or(&self.attr_responses, || {
            Err(CatalogError::DatasetNotFound {
                dsn: dsn.to_string(),
                operation: "get_dataset_attributes".to_string(),
            })
        })
    }

    fn list_datasets(&self, _filter: &DatasetFilter) -> Result<Vec<DatasetEntry>, CatalogError> {
        pop_or(&self.list_responses, || Ok(Vec::new()))
    }

    fn validate_dsn(&self, dsn: &str) -> Result<(), DsnValidationError> {
        ff_dscatalog::validate_dsn_string(dsn)
    }

    fn create_gdg_base(&self, _dsn: &str, _limit: u8, _scratch: bool) -> Result<(), CatalogError> {
        pop_or(&self.create_gdg_responses, || Ok(()))
    }

    fn create_generation(
        &self,
        base_dsn: &str,
        _attrs: DatasetAttributes,
    ) -> Result<GenerationInfo, CatalogError> {
        Ok(GenerationInfo {
            generation_name: "G0001V00".to_string(),
            dsn: format!("{base_dsn}.G0001V00"),
            path: std::path::PathBuf::from("/mock/gdg/gen1"),
            relative_offset: 0,
        })
    }

    fn resolve_generation(
        &self,
        base_dsn: &str,
        offset: i32,
    ) -> Result<GenerationInfo, CatalogError> {
        Ok(GenerationInfo {
            generation_name: "G0001V00".to_string(),
            dsn: format!("{base_dsn}.G0001V00"),
            path: std::path::PathBuf::from("/mock/gdg/gen1"),
            relative_offset: offset,
        })
    }

    fn list_generations(&self, _base_dsn: &str) -> Result<Vec<GenerationInfo>, CatalogError> {
        Ok(Vec::new())
    }

    fn get_allocation_defaults(&self, dsorg: Dsorg) -> DatasetAttributes {
        DatasetAttributes {
            recfm: Some(Recfm::FB),
            dsorg: Some(dsorg),
            ..DatasetAttributes::default()
        }
    }
}

// === MockVsamService ========================================================

/// A configurable mock of the reconciled `VsamService`.
pub struct MockVsamService {
    /// Responses for `initialize_dataset` and the per-type `create_*` calls.
    pub init_responses: Mutex<Vec<Result<(), VsamError>>>,
    /// Responses for `destroy_dataset`.
    pub destroy_responses: Mutex<Vec<Result<(), VsamError>>>,
    /// Responses for `define_aix`.
    pub define_aix_responses: Mutex<Vec<Result<(), VsamError>>>,
    /// Responses for `build_index`.
    pub build_index_responses: Mutex<Vec<Result<(), VsamError>>>,
    /// Responses for `open`.
    pub open_responses: Mutex<Vec<Result<VsamHandle, VsamError>>>,
    /// Responses for `start_browse`.
    pub browse_responses: Mutex<Vec<Result<BrowseHandle, VsamError>>>,
    /// Records returned sequentially from `next_record`.
    pub records: Mutex<Vec<Option<Record>>>,
    /// Responses for `put`.
    pub put_responses: Mutex<Vec<Result<(), VsamError>>>,
    /// Count of `put` calls actually made (record-copy observability).
    pub put_calls: Mutex<u64>,
}

impl MockVsamService {
    /// Creates a mock that returns success for everything.
    pub fn new_success() -> Self {
        Self {
            init_responses: Mutex::new(vec![]),
            destroy_responses: Mutex::new(vec![]),
            define_aix_responses: Mutex::new(vec![]),
            build_index_responses: Mutex::new(vec![]),
            open_responses: Mutex::new(vec![]),
            browse_responses: Mutex::new(vec![]),
            records: Mutex::new(vec![]),
            put_responses: Mutex::new(vec![]),
            put_calls: Mutex::new(0),
        }
    }
}

impl VsamService for MockVsamService {
    fn create_ksds(
        &self,
        _dsn: &str,
        _key_length: u16,
        _key_offset: u16,
        _record_length: u32,
    ) -> Result<(), VsamError> {
        pop_or(&self.init_responses, || Ok(()))
    }

    fn create_esds(&self, _dsn: &str, _record_length: u32) -> Result<(), VsamError> {
        pop_or(&self.init_responses, || Ok(()))
    }

    fn create_rrds(&self, _dsn: &str, _slot_size: u32) -> Result<(), VsamError> {
        pop_or(&self.init_responses, || Ok(()))
    }

    fn create_lds(&self, _dsn: &str) -> Result<(), VsamError> {
        pop_or(&self.init_responses, || Ok(()))
    }

    fn destroy_dataset(&self, _dsn: &str) -> Result<(), VsamError> {
        pop_or(&self.destroy_responses, || Ok(()))
    }

    fn initialize_dataset(
        &self,
        _dsn: &str,
        _vsam_type: VsamType,
        _params: VsamParams,
    ) -> Result<(), VsamError> {
        pop_or(&self.init_responses, || Ok(()))
    }

    fn open(&self, _dsn: &str, _mode: AccessMode) -> Result<VsamHandle, VsamError> {
        pop_or(&self.open_responses, || Ok(VsamHandle(1)))
    }

    fn get(&self, _handle: &VsamHandle, key: &[u8]) -> Result<Record, VsamError> {
        Ok(Record {
            key: key.to_vec(),
            data: Vec::new(),
        })
    }

    fn put(&self, _handle: &VsamHandle, _record: &Record) -> Result<(), VsamError> {
        let result = pop_or(&self.put_responses, || Ok(()));
        if result.is_ok() {
            *self.put_calls.lock().expect("put_calls lock") += 1;
        }
        result
    }

    fn delete(&self, _handle: &VsamHandle, _key: &[u8]) -> Result<(), VsamError> {
        Ok(())
    }

    fn close(&self, _handle: VsamHandle) -> Result<(), VsamError> {
        Ok(())
    }

    fn start_browse(
        &self,
        _handle: &VsamHandle,
        _start_key: &[u8],
        _direction: BrowseDirection,
    ) -> Result<BrowseHandle, VsamError> {
        pop_or(&self.browse_responses, || Ok(BrowseHandle(1)))
    }

    fn next_record(&self, _browse: &BrowseHandle) -> Result<Option<Record>, VsamError> {
        let mut guard = self.records.lock().expect("records lock poisoned");
        if guard.is_empty() {
            Ok(None)
        } else {
            Ok(guard.remove(0))
        }
    }

    fn end_browse(&self, _browse: BrowseHandle) -> Result<(), VsamError> {
        Ok(())
    }

    fn define_aix(
        &self,
        _base_dsn: &str,
        _aix_dsn: &str,
        _key_field: KeyField,
    ) -> Result<(), VsamError> {
        pop_or(&self.define_aix_responses, || Ok(()))
    }

    fn build_index(&self, _aix_dsn: &str) -> Result<(), VsamError> {
        pop_or(&self.build_index_responses, || Ok(()))
    }
}

// === MockAllocatorService ===================================================

/// A mock allocator service for testing.
pub struct MockAllocatorService {
    /// Responses for `resolve_dd`.
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
        let mut guard = self
            .resolve_responses
            .lock()
            .expect("resolve lock poisoned");
        if guard.is_empty() {
            Err(AllocatorError::DdNotFound(ddname.to_string()))
        } else {
            guard.remove(0)
        }
    }
}

// === TestServicesBuilder ====================================================

/// Builder for constructing test `IdcamsServices` with mock implementations.
pub struct TestServicesBuilder {
    catalog: Option<Arc<dyn DynCatalogService>>,
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

    /// Sets the catalog service (any reconciled `CatalogService`).
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
            access: default_dataset_access(),
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
