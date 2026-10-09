//! Reconciled VsamService trait and VsamCluster entity (CR-CH-059).
//!
//! Hosts the SINGLE reconciled `VsamService` trait in `ff-dscatalog`, backed
//! conceptually by the existing KSDS (`SqliteRecordProvider`), ESDS
//! (`NativeEsdsProvider`), and RRDS (`SqliteRrdsProvider`) backends. VSAM is
//! modelled as a CLUSTER ENTITY (`VsamCluster` carrying a `VsamType`), NOT as a
//! `Dsorg` variant -- the removal of the `Vsam` / `Da` `Dsorg` variants is
//! deliberate (dataset-catalog Requirement 33.5).
//!
//! In RC.A this trait is a SURFACE: a mock implements it for the governance
//! test. A concrete `ff-dscatalog` impl against the live backends is RC.B.7 and
//! is intentionally not wired here.
//!
//! Validates: dataset-catalog Requirement 33.3, 33.5.

// === Error type =============================================================

/// Error type for VSAM operations surfaced through `VsamService`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum VsamError {
    /// The dataset was not found.
    #[error("VSAM dataset not found: {dsn}")]
    DatasetNotFound {
        /// Offending DSN.
        dsn: String,
    },
    /// The dataset already exists.
    #[error("VSAM dataset already exists: {dsn}")]
    DatasetAlreadyExists {
        /// Offending DSN.
        dsn: String,
    },
    /// A duplicate key was detected during insertion.
    #[error("duplicate key in dataset '{dsn}'")]
    DuplicateKey {
        /// Offending DSN.
        dsn: String,
    },
    /// The requested record was not found.
    #[error("record not found for key in dataset '{dsn}'")]
    RecordNotFound {
        /// Offending DSN.
        dsn: String,
    },
    /// The dataset is not open or the handle is invalid.
    #[error("invalid VSAM handle")]
    InvalidHandle,
    /// The browse handle is invalid or exhausted.
    #[error("invalid browse handle")]
    InvalidBrowseHandle,
    /// The operation is not supported for this dataset type.
    #[error("operation not supported for this VSAM type: {reason}")]
    UnsupportedOperation {
        /// Reason the operation is unsupported.
        reason: String,
    },
    /// The functionality is not yet implemented.
    #[error("VSAM operation not implemented: {operation}")]
    NotImplemented {
        /// Operation name.
        operation: String,
    },
    /// An I/O or storage error occurred.
    #[error("VSAM storage error: {0}")]
    StorageError(String),
    /// An internal error occurred.
    #[error("internal VSAM error: {0}")]
    Internal(String),
}

// === Data types =============================================================

/// Opaque handle to an open VSAM dataset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VsamHandle(pub u64);

/// Opaque handle to an active browse operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BrowseHandle(pub u64);

/// A VSAM record (key + data).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// Record key (for KSDS/RRDS); empty for ESDS.
    pub key: Vec<u8>,
    /// Record data.
    pub data: Vec<u8>,
}

/// VSAM dataset type carried by a `VsamCluster`.
///
/// VSAM is a cluster entity, NOT a `Dsorg` variant (Requirement 33.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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

/// A VSAM cluster entity.
///
/// Models a VSAM dataset as a cluster with a `VsamType`, keeping VSAM out of the
/// `Dsorg` enum (Requirement 33.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VsamCluster {
    /// Cluster dataset name.
    pub dsn: String,
    /// VSAM type (KSDS/ESDS/RRDS/LDS).
    pub vsam_type: VsamType,
    /// Initialization parameters.
    pub params: VsamParams,
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
#[non_exhaustive]
pub enum AccessMode {
    /// Read-only access.
    Read,
    /// Read-write access.
    ReadWrite,
    /// Write-only (bulk loading).
    Write,
}

/// Browse direction for sequential traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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

// === VsamService trait ======================================================

/// The reconciled interface for VSAM record-level operations.
///
/// Object-safe for dynamic dispatch (`Box<dyn VsamService>`). `ff-idcams`
/// depends on this trait for all VSAM operations.
///
/// # Errors
///
/// All fallible methods return `Result<T, VsamError>`.
///
/// Validates: dataset-catalog Requirement 33.3.
pub trait VsamService: Send + Sync {
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

// === StubVsamService ========================================================

/// A no-op stub returning `VsamError::NotImplemented` for all methods.
///
/// Enables dependent crates to compile and test against the trait before the
/// concrete `ff-dscatalog` VSAM impl lands (RC.B.7).
pub struct StubVsamService;

impl VsamService for StubVsamService {
    fn create_ksds(
        &self,
        _dsn: &str,
        _key_length: u16,
        _key_offset: u16,
        _record_length: u32,
    ) -> Result<(), VsamError> {
        Err(not_impl("create_ksds"))
    }

    fn create_esds(&self, _dsn: &str, _record_length: u32) -> Result<(), VsamError> {
        Err(not_impl("create_esds"))
    }

    fn create_rrds(&self, _dsn: &str, _slot_size: u32) -> Result<(), VsamError> {
        Err(not_impl("create_rrds"))
    }

    fn create_lds(&self, _dsn: &str) -> Result<(), VsamError> {
        Err(not_impl("create_lds"))
    }

    fn destroy_dataset(&self, _dsn: &str) -> Result<(), VsamError> {
        Err(not_impl("destroy_dataset"))
    }

    fn initialize_dataset(
        &self,
        _dsn: &str,
        _vsam_type: VsamType,
        _params: VsamParams,
    ) -> Result<(), VsamError> {
        Err(not_impl("initialize_dataset"))
    }

    fn open(&self, _dsn: &str, _mode: AccessMode) -> Result<VsamHandle, VsamError> {
        Err(not_impl("open"))
    }

    fn get(&self, _handle: &VsamHandle, _key: &[u8]) -> Result<Record, VsamError> {
        Err(not_impl("get"))
    }

    fn put(&self, _handle: &VsamHandle, _record: &Record) -> Result<(), VsamError> {
        Err(not_impl("put"))
    }

    fn delete(&self, _handle: &VsamHandle, _key: &[u8]) -> Result<(), VsamError> {
        Err(not_impl("delete"))
    }

    fn close(&self, _handle: VsamHandle) -> Result<(), VsamError> {
        Err(not_impl("close"))
    }

    fn start_browse(
        &self,
        _handle: &VsamHandle,
        _start_key: &[u8],
        _direction: BrowseDirection,
    ) -> Result<BrowseHandle, VsamError> {
        Err(not_impl("start_browse"))
    }

    fn next_record(&self, _browse: &BrowseHandle) -> Result<Option<Record>, VsamError> {
        Err(not_impl("next_record"))
    }

    fn end_browse(&self, _browse: BrowseHandle) -> Result<(), VsamError> {
        Err(not_impl("end_browse"))
    }

    fn define_aix(
        &self,
        _base_dsn: &str,
        _aix_dsn: &str,
        _key_field: KeyField,
    ) -> Result<(), VsamError> {
        Err(not_impl("define_aix"))
    }

    fn build_index(&self, _aix_dsn: &str) -> Result<(), VsamError> {
        Err(not_impl("build_index"))
    }
}

fn not_impl(operation: &str) -> VsamError {
    VsamError::NotImplemented {
        operation: operation.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::Dsorg;

    struct MockVsamService;

    impl VsamService for MockVsamService {
        fn create_ksds(
            &self,
            _dsn: &str,
            _key_length: u16,
            _key_offset: u16,
            _record_length: u32,
        ) -> Result<(), VsamError> {
            Ok(())
        }

        fn create_esds(&self, _dsn: &str, _record_length: u32) -> Result<(), VsamError> {
            Ok(())
        }

        fn create_rrds(&self, _dsn: &str, _slot_size: u32) -> Result<(), VsamError> {
            Ok(())
        }

        fn create_lds(&self, _dsn: &str) -> Result<(), VsamError> {
            Ok(())
        }

        fn destroy_dataset(&self, _dsn: &str) -> Result<(), VsamError> {
            Ok(())
        }

        fn initialize_dataset(
            &self,
            _dsn: &str,
            _vsam_type: VsamType,
            _params: VsamParams,
        ) -> Result<(), VsamError> {
            Ok(())
        }

        fn open(&self, _dsn: &str, _mode: AccessMode) -> Result<VsamHandle, VsamError> {
            Ok(VsamHandle(1))
        }

        fn get(&self, _handle: &VsamHandle, _key: &[u8]) -> Result<Record, VsamError> {
            Ok(Record {
                key: vec![1, 2, 3],
                data: vec![10, 20, 30],
            })
        }

        fn put(&self, _handle: &VsamHandle, _record: &Record) -> Result<(), VsamError> {
            Ok(())
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
            Ok(BrowseHandle(1))
        }

        fn next_record(&self, _browse: &BrowseHandle) -> Result<Option<Record>, VsamError> {
            Ok(None)
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
            Ok(())
        }

        fn build_index(&self, _aix_dsn: &str) -> Result<(), VsamError> {
            Ok(())
        }
    }

    #[test]
    fn mock_vsam_service_is_object_safe_as_dyn() {
        // Validates: Requirement 33.3 -- object-safe VsamService.
        let svc: Box<dyn VsamService> = Box::new(MockVsamService);
        assert!(svc.create_ksds("TEST.KSDS", 8, 0, 256).is_ok());
        assert_eq!(
            svc.open("TEST.KSDS", AccessMode::Read).unwrap(),
            VsamHandle(1)
        );
    }

    #[test]
    fn vsam_service_covers_requirement_16_operation_set() {
        // Validates: Requirement 33.3 -- create per type, destroy/init, record
        // ops, browse, alternate indexes.
        let svc = MockVsamService;
        assert!(svc.create_ksds("K", 8, 0, 256).is_ok());
        assert!(svc.create_esds("E", 256).is_ok());
        assert!(svc.create_rrds("R", 80).is_ok());
        assert!(svc.create_lds("L").is_ok());
        assert!(svc.destroy_dataset("K").is_ok());
        assert!(svc
            .initialize_dataset("K", VsamType::Ksds, VsamParams::default())
            .is_ok());
        let handle = svc.open("K", AccessMode::ReadWrite).unwrap();
        assert!(svc.get(&handle, b"key").is_ok());
        assert!(svc
            .put(
                &handle,
                &Record {
                    key: vec![1],
                    data: vec![2]
                }
            )
            .is_ok());
        assert!(svc.delete(&handle, b"key").is_ok());
        let browse = svc
            .start_browse(&handle, b"", BrowseDirection::Forward)
            .unwrap();
        assert!(svc.next_record(&browse).is_ok());
        assert!(svc.end_browse(browse).is_ok());
        assert!(svc.close(handle).is_ok());
        assert!(svc
            .define_aix(
                "K",
                "K.AIX",
                KeyField {
                    offset: 0,
                    length: 4,
                    unique: false
                }
            )
            .is_ok());
        assert!(svc.build_index("K.AIX").is_ok());
    }

    #[test]
    fn vsam_cluster_carries_type_and_is_not_a_dsorg_variant() {
        // Validates: Requirement 33.5 -- VSAM is a cluster entity; Dsorg has
        // exactly PS, PO, GDG (no Vsam / Da variant).
        let cluster = VsamCluster {
            dsn: "MY.VSAM".to_string(),
            vsam_type: VsamType::Ksds,
            params: VsamParams::default(),
        };
        assert_eq!(cluster.vsam_type, VsamType::Ksds);

        // Exhaustive match over Dsorg proves no VSAM/DA variant exists.
        for dsorg in [Dsorg::PS, Dsorg::PO, Dsorg::GDG] {
            match dsorg {
                Dsorg::PS | Dsorg::PO | Dsorg::GDG => {}
            }
        }
    }

    #[test]
    fn stub_returns_not_implemented() {
        // Validates: Requirement 33.3 -- stub enables compilation.
        let stub = StubVsamService;
        match stub.create_ksds("X", 8, 0, 256).unwrap_err() {
            VsamError::NotImplemented { operation } => assert_eq!(operation, "create_ksds"),
            other => panic!("expected NotImplemented, got {other:?}"),
        }
    }
}
