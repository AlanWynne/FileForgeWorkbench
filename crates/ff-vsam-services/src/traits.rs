//! Trait interface definitions for VSAM Services.
//!
//! The `VsamService` trait defines the complete public API contract for VSAM
//! record-level operations. Dependent crates (ff-idcams) code against this
//! trait rather than concrete implementations.

// ─── Error Types ────────────────────────────────────────────────────────────

/// Error type for VSAM operations.
#[derive(Debug, thiserror::Error)]
pub enum VsamError {
    /// The dataset was not found.
    #[error("VSAM dataset not found: {dsn}")]
    DatasetNotFound { dsn: String },

    /// The dataset already exists.
    #[error("VSAM dataset already exists: {dsn}")]
    DatasetAlreadyExists { dsn: String },

    /// A duplicate key was detected during insertion.
    #[error("duplicate key in dataset '{dsn}'")]
    DuplicateKey { dsn: String },

    /// The requested record was not found.
    #[error("record not found for key in dataset '{dsn}'")]
    RecordNotFound { dsn: String },

    /// The dataset is not open or the handle is invalid.
    #[error("invalid VSAM handle")]
    InvalidHandle,

    /// The browse handle is invalid or exhausted.
    #[error("invalid browse handle")]
    InvalidBrowseHandle,

    /// The operation is not supported for this dataset type.
    #[error("operation not supported for this VSAM type: {reason}")]
    UnsupportedOperation { reason: String },

    /// The functionality is not yet implemented.
    #[error("VSAM operation not implemented: {operation}")]
    NotImplemented { operation: String },

    /// An I/O or storage error occurred.
    #[error("VSAM storage error: {0}")]
    StorageError(String),

    /// An internal error occurred.
    #[error("internal VSAM error: {0}")]
    Internal(String),
}

// ─── Data Types ─────────────────────────────────────────────────────────────

/// Opaque handle to an open VSAM dataset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VsamHandle(pub u64);

/// Opaque handle to an active browse operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BrowseHandle(pub u64);

/// A VSAM record (key + data).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// The record key (for KSDS/RRDS), empty for ESDS.
    pub key: Vec<u8>,
    /// The record data.
    pub data: Vec<u8>,
}

/// VSAM dataset type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VsamType {
    /// Key-Sequenced Data Set.
    Ksds,
    /// Entry-Sequenced Data Set.
    Esds,
    /// Relative Record Data Set.
    Rrds,
    /// Linear Data Set.
    Lds,
}

/// Parameters for VSAM dataset initialization.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VsamParams {
    /// Key length (KSDS only).
    pub key_length: Option<u16>,
    /// Key offset within record (KSDS only).
    pub key_offset: Option<u16>,
    /// Maximum record length.
    pub record_length: Option<u32>,
    /// Slot size (RRDS only).
    pub slot_size: Option<u32>,
}

/// Access mode for opening a VSAM dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    /// Read-only access.
    Read,
    /// Read-write access.
    ReadWrite,
    /// Write-only (for bulk loading).
    Write,
}

/// Browse direction for sequential traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowseDirection {
    /// Forward (ascending key order).
    Forward,
    /// Backward (descending key order).
    Backward,
}

/// Key field definition for alternate indexes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyField {
    /// Offset of the key within the record.
    pub offset: u16,
    /// Length of the key field.
    pub length: u16,
    /// Whether duplicate keys are allowed.
    pub unique: bool,
}

// ─── VsamService Trait ──────────────────────────────────────────────────────

/// The primary interface for VSAM record-level operations.
///
/// This trait is object-safe to enable dynamic dispatch and mock implementations.
/// Dependent crates (ff-idcams) depend on this trait for all VSAM operations.
///
/// # Errors
///
/// All fallible methods return `Result<T, VsamError>`.
pub trait VsamService: Send + Sync {
    // ── Dataset Lifecycle ──

    /// Create a new KSDS (Key-Sequenced Data Set).
    fn create_ksds(
        &self,
        dsn: &str,
        key_length: u16,
        key_offset: u16,
        record_length: u32,
    ) -> Result<(), VsamError>;

    /// Create a new ESDS (Entry-Sequenced Data Set).
    fn create_esds(&self, dsn: &str, record_length: u32) -> Result<(), VsamError>;

    /// Create a new RRDS (Relative Record Data Set).
    fn create_rrds(&self, dsn: &str, slot_size: u32) -> Result<(), VsamError>;

    /// Create a new LDS (Linear Data Set).
    fn create_lds(&self, dsn: &str) -> Result<(), VsamError>;

    /// Destroy an existing VSAM dataset and clean up storage structures.
    fn destroy_dataset(&self, dsn: &str) -> Result<(), VsamError>;

    /// Initialize a VSAM dataset with the given type and parameters.
    fn initialize_dataset(
        &self,
        dsn: &str,
        vsam_type: VsamType,
        params: VsamParams,
    ) -> Result<(), VsamError>;

    // ── Record-Level Operations ──

    /// Open a VSAM dataset for record-level access.
    fn open(&self, dsn: &str, mode: AccessMode) -> Result<VsamHandle, VsamError>;

    /// Retrieve a record by key.
    fn get(&self, handle: &VsamHandle, key: &[u8]) -> Result<Record, VsamError>;

    /// Insert or update a record.
    fn put(&self, handle: &VsamHandle, record: &Record) -> Result<(), VsamError>;

    /// Delete a record by key.
    fn delete(&self, handle: &VsamHandle, key: &[u8]) -> Result<(), VsamError>;

    /// Close an open VSAM dataset handle.
    fn close(&self, handle: VsamHandle) -> Result<(), VsamError>;

    // ── Browse Operations ──

    /// Start a browse (sequential traversal) from a given key position.
    fn start_browse(
        &self,
        handle: &VsamHandle,
        start_key: &[u8],
        direction: BrowseDirection,
    ) -> Result<BrowseHandle, VsamError>;

    /// Retrieve the next record in a browse operation.
    fn next_record(&self, browse: &BrowseHandle) -> Result<Option<Record>, VsamError>;

    /// End a browse operation and release resources.
    fn end_browse(&self, browse: BrowseHandle) -> Result<(), VsamError>;

    // ── Alternate Index Operations ──

    /// Define an alternate index over a base dataset.
    fn define_aix(
        &self,
        base_dsn: &str,
        aix_dsn: &str,
        key_field: KeyField,
    ) -> Result<(), VsamError>;

    /// Build (or rebuild) an alternate index.
    fn build_index(&self, aix_dsn: &str) -> Result<(), VsamError>;
}

// ─── StubVsamService ────────────────────────────────────────────────────────

/// A no-op stub implementation that returns `VsamError::NotImplemented` for all methods.
///
/// This enables dependent crates to compile and test against the trait interface
/// before the full VSAM implementation is available.
pub struct StubVsamService;

impl VsamService for StubVsamService {
    fn create_ksds(
        &self,
        _dsn: &str,
        _key_length: u16,
        _key_offset: u16,
        _record_length: u32,
    ) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "create_ksds".to_string(),
        })
    }

    fn create_esds(&self, _dsn: &str, _record_length: u32) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "create_esds".to_string(),
        })
    }

    fn create_rrds(&self, _dsn: &str, _slot_size: u32) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "create_rrds".to_string(),
        })
    }

    fn create_lds(&self, _dsn: &str) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "create_lds".to_string(),
        })
    }

    fn destroy_dataset(&self, _dsn: &str) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "destroy_dataset".to_string(),
        })
    }

    fn initialize_dataset(
        &self,
        _dsn: &str,
        _vsam_type: VsamType,
        _params: VsamParams,
    ) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "initialize_dataset".to_string(),
        })
    }

    fn open(&self, _dsn: &str, _mode: AccessMode) -> Result<VsamHandle, VsamError> {
        Err(VsamError::NotImplemented {
            operation: "open".to_string(),
        })
    }

    fn get(&self, _handle: &VsamHandle, _key: &[u8]) -> Result<Record, VsamError> {
        Err(VsamError::NotImplemented {
            operation: "get".to_string(),
        })
    }

    fn put(&self, _handle: &VsamHandle, _record: &Record) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "put".to_string(),
        })
    }

    fn delete(&self, _handle: &VsamHandle, _key: &[u8]) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "delete".to_string(),
        })
    }

    fn close(&self, _handle: VsamHandle) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "close".to_string(),
        })
    }

    fn start_browse(
        &self,
        _handle: &VsamHandle,
        _start_key: &[u8],
        _direction: BrowseDirection,
    ) -> Result<BrowseHandle, VsamError> {
        Err(VsamError::NotImplemented {
            operation: "start_browse".to_string(),
        })
    }

    fn next_record(&self, _browse: &BrowseHandle) -> Result<Option<Record>, VsamError> {
        Err(VsamError::NotImplemented {
            operation: "next_record".to_string(),
        })
    }

    fn end_browse(&self, _browse: BrowseHandle) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "end_browse".to_string(),
        })
    }

    fn define_aix(
        &self,
        _base_dsn: &str,
        _aix_dsn: &str,
        _key_field: KeyField,
    ) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "define_aix".to_string(),
        })
    }

    fn build_index(&self, _aix_dsn: &str) -> Result<(), VsamError> {
        Err(VsamError::NotImplemented {
            operation: "build_index".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 16 AC 6 — VsamService is object-safe
    #[test]
    fn vsam_service_is_object_safe() {
        // This test verifies that Box<dyn VsamService> compiles,
        // proving object safety of the trait.
        fn _assert_object_safe(_: Box<dyn VsamService>) {}
    }

    // Validates: Requirement 16 AC 7 — StubVsamService enables compilation
    #[test]
    fn stub_vsam_service_returns_not_implemented() {
        let stub = StubVsamService;
        let result = stub.create_ksds("TEST.DATASET", 8, 0, 100);
        assert!(result.is_err());
        match result.unwrap_err() {
            VsamError::NotImplemented { operation } => {
                assert_eq!(operation, "create_ksds");
            }
            other => panic!("expected NotImplemented, got: {other:?}"),
        }
    }
}
