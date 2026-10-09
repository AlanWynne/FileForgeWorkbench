//! Reconciled CatalogService trait over ff-dscatalog's own types (CR-CH-059).
//!
//! This module hosts the SINGLE reconciled `CatalogService` trait (and the
//! object-safe `DynCatalogService` wrapper) that external consumers (`ff-dsalloc`,
//! `ff-idcams`) depend on. It uses `ff-dscatalog`'s own `Dsn`, `Dsorg`, and
//! `Recfm` types -- the divergent enums of the retired `ff-dataset-catalog`
//! trait crate (`Dsorg {Ps,Po,Da,Vsam}`, `Recfm {F,Fb,V,Vb,U}`) are NOT carried
//! forward. VSAM is modelled as a cluster entity (see `vsam_service`), NOT a
//! `Dsorg` variant.
//!
//! Validates: dataset-catalog Requirement 33.1, 33.2, 33.4; dataset-ownership-model
//! Requirement 22.2, 22.4, 22.5.

use std::path::PathBuf;

use crate::dataset::{Dsorg, Recfm};
use crate::dsn::Dsn;
use crate::error::CatalogError;

// === DSN validation error ===================================================

/// Error type for DSN validation failures surfaced through `CatalogService`.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum DsnValidationError {
    /// DSN is empty.
    #[error("DSN must not be empty")]
    Empty,
    /// DSN exceeds the 44-character maximum.
    #[error("DSN exceeds maximum length of 44 characters: length={length}")]
    TooLong {
        /// Offending length.
        length: usize,
    },
    /// A qualifier exceeds the 8-character maximum.
    #[error("qualifier '{qualifier}' exceeds 8 characters")]
    QualifierTooLong {
        /// Offending qualifier.
        qualifier: String,
    },
    /// A qualifier contains invalid characters.
    #[error("qualifier '{qualifier}' contains invalid characters")]
    InvalidCharacters {
        /// Offending qualifier.
        qualifier: String,
    },
    /// DSN has no qualifiers.
    #[error("DSN must have at least one qualifier")]
    NoQualifiers,
}

// === Reconciled DTOs ========================================================

/// Opaque identifier for a dataset in the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatasetId(pub String);

/// Dataset attributes exchanged across the `CatalogService` boundary.
///
/// Uses `ff-dscatalog`'s own `Dsorg` / `Recfm` enums (CR-CH-059).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatasetAttributes {
    /// Record format.
    pub recfm: Option<Recfm>,
    /// Logical record length.
    pub lrecl: Option<u32>,
    /// Block size.
    pub blksize: Option<u32>,
    /// Dataset organization.
    pub dsorg: Option<Dsorg>,
    /// Volume serial (if applicable).
    pub volser: Option<String>,
}

/// Result of resolving a DSN to its physical location and catalog metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionResult {
    /// Resolved physical path.
    pub path: PathBuf,
    /// Catalog that contained the entry.
    pub catalog_name: String,
    /// Attributes from the catalog entry.
    pub attributes: DatasetAttributes,
}

impl Default for ResolutionResult {
    fn default() -> Self {
        Self {
            path: PathBuf::from("/default"),
            catalog_name: String::from("MASTER"),
            attributes: DatasetAttributes::default(),
        }
    }
}

/// A dataset entry returned from catalog queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetEntry {
    /// Fully-qualified dataset name.
    pub dsn: String,
    /// Dataset attributes.
    pub attributes: DatasetAttributes,
    /// Catalog containing this entry.
    pub catalog_name: String,
}

/// Filter criteria for listing datasets.
#[derive(Debug, Clone, Default)]
pub struct DatasetFilter {
    /// DSN pattern (supports wildcards).
    pub pattern: Option<String>,
    /// Filter by dataset organization.
    pub dsorg: Option<Dsorg>,
    /// Filter by catalog name.
    pub catalog_name: Option<String>,
}

/// Information about a GDG generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationInfo {
    /// Absolute generation name (e.g. G0001V00).
    pub generation_name: String,
    /// Fully-qualified DSN of this generation.
    pub dsn: String,
    /// Physical path.
    pub path: PathBuf,
    /// Relative offset from current (0 = current, -1 = previous).
    pub relative_offset: i32,
}

// === CatalogService trait ===================================================

/// The reconciled catalog-operations interface consumers depend on.
///
/// `ff-dsalloc` and `ff-idcams` code against this trait (and `DynCatalogService`
/// for dynamic dispatch / mocks) rather than concrete `ff-dscatalog` types.
///
/// # Errors
///
/// All fallible methods return `Result<T, CatalogError>` (DSN validation returns
/// `DsnValidationError`).
///
/// Validates: dataset-catalog Requirement 33.1, 33.2.
pub trait CatalogService: Send + Sync {
    /// Create a new dataset entry in the catalog.
    fn create_dataset(
        &self,
        dsn: &str,
        attrs: DatasetAttributes,
    ) -> Result<DatasetId, CatalogError>;

    /// Delete a dataset entry from the catalog.
    fn delete_dataset(&self, dsn: &str) -> Result<(), CatalogError>;

    /// Update attributes of an existing dataset.
    fn update_dataset(&self, dsn: &str, attrs: DatasetAttributes) -> Result<(), CatalogError>;

    /// Rename a dataset.
    fn rename_dataset(&self, old_dsn: &str, new_dsn: &str) -> Result<(), CatalogError>;

    /// Resolve a DSN to its physical path.
    fn resolve_dsn(&self, dsn: &str) -> Result<ResolutionResult, CatalogError>;

    /// Check whether a dataset exists in any mounted catalog.
    fn dataset_exists(&self, dsn: &str) -> Result<bool, CatalogError>;

    /// Retrieve the attributes of an existing dataset.
    fn get_dataset_attributes(&self, dsn: &str) -> Result<DatasetAttributes, CatalogError>;

    /// List datasets matching the given filter criteria.
    fn list_datasets(&self, filter: &DatasetFilter) -> Result<Vec<DatasetEntry>, CatalogError>;

    /// Validate a DSN string against naming rules.
    fn validate_dsn(&self, dsn: &str) -> Result<(), DsnValidationError>;

    /// Create a GDG base definition.
    fn create_gdg_base(&self, dsn: &str, limit: u8, scratch: bool) -> Result<(), CatalogError>;

    /// Create a new generation under an existing GDG base.
    fn create_generation(
        &self,
        base_dsn: &str,
        attrs: DatasetAttributes,
    ) -> Result<GenerationInfo, CatalogError>;

    /// Resolve a relative generation reference to its generation info.
    fn resolve_generation(
        &self,
        base_dsn: &str,
        offset: i32,
    ) -> Result<GenerationInfo, CatalogError>;

    /// List all generations under a GDG base.
    fn list_generations(&self, base_dsn: &str) -> Result<Vec<GenerationInfo>, CatalogError>;

    /// Retrieve the configured default attributes for a given dataset organization.
    fn get_allocation_defaults(&self, dsorg: Dsorg) -> DatasetAttributes;
}

// === DynCatalogService (object-safe wrapper) ================================

/// Object-safe wrapper enabling `Box<dyn DynCatalogService>` for dynamic
/// dispatch and mock injection.
///
/// Validates: dataset-catalog Requirement 33.1; dataset-ownership-model Requirement 22.5.
pub trait DynCatalogService: Send + Sync {
    /// Create a new dataset entry in the catalog.
    fn create_dataset(
        &self,
        dsn: &str,
        attrs: DatasetAttributes,
    ) -> Result<DatasetId, CatalogError>;

    /// Delete a dataset entry from the catalog.
    fn delete_dataset(&self, dsn: &str) -> Result<(), CatalogError>;

    /// Update attributes of an existing dataset.
    fn update_dataset(&self, dsn: &str, attrs: DatasetAttributes) -> Result<(), CatalogError>;

    /// Rename a dataset.
    fn rename_dataset(&self, old_dsn: &str, new_dsn: &str) -> Result<(), CatalogError>;

    /// Resolve a DSN to its physical path.
    fn resolve_dsn(&self, dsn: &str) -> Result<ResolutionResult, CatalogError>;

    /// Check whether a dataset exists in any mounted catalog.
    fn dataset_exists(&self, dsn: &str) -> Result<bool, CatalogError>;

    /// Retrieve the attributes of an existing dataset.
    fn get_dataset_attributes(&self, dsn: &str) -> Result<DatasetAttributes, CatalogError>;

    /// List datasets matching the given filter criteria.
    fn list_datasets(&self, filter: &DatasetFilter) -> Result<Vec<DatasetEntry>, CatalogError>;

    /// Validate a DSN string against naming rules.
    fn validate_dsn(&self, dsn: &str) -> Result<(), DsnValidationError>;

    /// Create a GDG base definition.
    fn create_gdg_base(&self, dsn: &str, limit: u8, scratch: bool) -> Result<(), CatalogError>;

    /// Create a new generation under an existing GDG base.
    fn create_generation(
        &self,
        base_dsn: &str,
        attrs: DatasetAttributes,
    ) -> Result<GenerationInfo, CatalogError>;

    /// Resolve a relative generation reference to its generation info.
    fn resolve_generation(
        &self,
        base_dsn: &str,
        offset: i32,
    ) -> Result<GenerationInfo, CatalogError>;

    /// List all generations under a GDG base.
    fn list_generations(&self, base_dsn: &str) -> Result<Vec<GenerationInfo>, CatalogError>;

    /// Retrieve the configured default attributes for a given dataset organization.
    fn get_allocation_defaults(&self, dsorg: Dsorg) -> DatasetAttributes;
}

/// Blanket impl: any `CatalogService` auto-implements `DynCatalogService`.
impl<T: CatalogService> DynCatalogService for T {
    fn create_dataset(
        &self,
        dsn: &str,
        attrs: DatasetAttributes,
    ) -> Result<DatasetId, CatalogError> {
        CatalogService::create_dataset(self, dsn, attrs)
    }

    fn delete_dataset(&self, dsn: &str) -> Result<(), CatalogError> {
        CatalogService::delete_dataset(self, dsn)
    }

    fn update_dataset(&self, dsn: &str, attrs: DatasetAttributes) -> Result<(), CatalogError> {
        CatalogService::update_dataset(self, dsn, attrs)
    }

    fn rename_dataset(&self, old_dsn: &str, new_dsn: &str) -> Result<(), CatalogError> {
        CatalogService::rename_dataset(self, old_dsn, new_dsn)
    }

    fn resolve_dsn(&self, dsn: &str) -> Result<ResolutionResult, CatalogError> {
        CatalogService::resolve_dsn(self, dsn)
    }

    fn dataset_exists(&self, dsn: &str) -> Result<bool, CatalogError> {
        CatalogService::dataset_exists(self, dsn)
    }

    fn get_dataset_attributes(&self, dsn: &str) -> Result<DatasetAttributes, CatalogError> {
        CatalogService::get_dataset_attributes(self, dsn)
    }

    fn list_datasets(&self, filter: &DatasetFilter) -> Result<Vec<DatasetEntry>, CatalogError> {
        CatalogService::list_datasets(self, filter)
    }

    fn validate_dsn(&self, dsn: &str) -> Result<(), DsnValidationError> {
        CatalogService::validate_dsn(self, dsn)
    }

    fn create_gdg_base(&self, dsn: &str, limit: u8, scratch: bool) -> Result<(), CatalogError> {
        CatalogService::create_gdg_base(self, dsn, limit, scratch)
    }

    fn create_generation(
        &self,
        base_dsn: &str,
        attrs: DatasetAttributes,
    ) -> Result<GenerationInfo, CatalogError> {
        CatalogService::create_generation(self, base_dsn, attrs)
    }

    fn resolve_generation(
        &self,
        base_dsn: &str,
        offset: i32,
    ) -> Result<GenerationInfo, CatalogError> {
        CatalogService::resolve_generation(self, base_dsn, offset)
    }

    fn list_generations(&self, base_dsn: &str) -> Result<Vec<GenerationInfo>, CatalogError> {
        CatalogService::list_generations(self, base_dsn)
    }

    fn get_allocation_defaults(&self, dsorg: Dsorg) -> DatasetAttributes {
        CatalogService::get_allocation_defaults(self, dsorg)
    }
}

/// Validate a DSN using `ff-dscatalog`'s own `Dsn` parser, mapping to the
/// `CatalogService` validation-error taxonomy.
pub fn validate_dsn_string(dsn: &str) -> Result<(), DsnValidationError> {
    if dsn.is_empty() {
        return Err(DsnValidationError::Empty);
    }
    match Dsn::parse(dsn) {
        Ok(_) => Ok(()),
        Err(_) => {
            if dsn.len() > 44 {
                Err(DsnValidationError::TooLong { length: dsn.len() })
            } else if dsn.split('.').any(|q| q.len() > 8) {
                Err(DsnValidationError::QualifierTooLong {
                    qualifier: dsn
                        .split('.')
                        .find(|q| q.len() > 8)
                        .unwrap_or_default()
                        .to_string(),
                })
            } else {
                Err(DsnValidationError::InvalidCharacters {
                    qualifier: dsn.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal in-memory mock proving the reconciled trait is implementable
    /// and object-safe over ff-dscatalog's own types.
    struct MockCatalogService;

    impl CatalogService for MockCatalogService {
        fn create_dataset(
            &self,
            _dsn: &str,
            _attrs: DatasetAttributes,
        ) -> Result<DatasetId, CatalogError> {
            Ok(DatasetId("MOCK-001".to_string()))
        }

        fn delete_dataset(&self, _dsn: &str) -> Result<(), CatalogError> {
            Ok(())
        }

        fn update_dataset(
            &self,
            _dsn: &str,
            _attrs: DatasetAttributes,
        ) -> Result<(), CatalogError> {
            Ok(())
        }

        fn rename_dataset(&self, _old_dsn: &str, _new_dsn: &str) -> Result<(), CatalogError> {
            Ok(())
        }

        fn resolve_dsn(&self, _dsn: &str) -> Result<ResolutionResult, CatalogError> {
            Ok(ResolutionResult::default())
        }

        fn dataset_exists(&self, _dsn: &str) -> Result<bool, CatalogError> {
            Ok(true)
        }

        fn get_dataset_attributes(&self, _dsn: &str) -> Result<DatasetAttributes, CatalogError> {
            Ok(DatasetAttributes::default())
        }

        fn list_datasets(
            &self,
            _filter: &DatasetFilter,
        ) -> Result<Vec<DatasetEntry>, CatalogError> {
            Ok(Vec::new())
        }

        fn validate_dsn(&self, dsn: &str) -> Result<(), DsnValidationError> {
            validate_dsn_string(dsn)
        }

        fn create_gdg_base(
            &self,
            _dsn: &str,
            _limit: u8,
            _scratch: bool,
        ) -> Result<(), CatalogError> {
            Ok(())
        }

        fn create_generation(
            &self,
            _base_dsn: &str,
            _attrs: DatasetAttributes,
        ) -> Result<GenerationInfo, CatalogError> {
            Ok(GenerationInfo {
                generation_name: "G0001V00".to_string(),
                dsn: "MOCK.BASE.G0001V00".to_string(),
                path: PathBuf::from("/mock/gdg/gen1"),
                relative_offset: 0,
            })
        }

        fn resolve_generation(
            &self,
            _base_dsn: &str,
            _offset: i32,
        ) -> Result<GenerationInfo, CatalogError> {
            Ok(GenerationInfo {
                generation_name: "G0001V00".to_string(),
                dsn: "MOCK.BASE.G0001V00".to_string(),
                path: PathBuf::from("/mock/gdg/gen1"),
                relative_offset: 0,
            })
        }

        fn list_generations(&self, _base_dsn: &str) -> Result<Vec<GenerationInfo>, CatalogError> {
            Ok(Vec::new())
        }

        fn get_allocation_defaults(&self, dsorg: Dsorg) -> DatasetAttributes {
            // Round-trips the ff-dscatalog Dsorg enum, proving single-enum usage.
            let recfm = match dsorg {
                Dsorg::PS | Dsorg::PO => Some(Recfm::FB),
                Dsorg::GDG => None,
            };
            DatasetAttributes {
                recfm,
                dsorg: Some(dsorg),
                ..DatasetAttributes::default()
            }
        }
    }

    #[test]
    fn mock_catalog_service_is_object_safe_as_dyn() {
        // Validates: Requirement 33.1 -- object-safe DynCatalogService wrapper.
        let svc: Box<dyn DynCatalogService> = Box::new(MockCatalogService);
        assert!(svc.dataset_exists("ANY.DSN").unwrap());
        assert_eq!(
            svc.create_dataset("NEW.DSN", DatasetAttributes::default())
                .unwrap(),
            DatasetId("MOCK-001".to_string())
        );
    }

    #[test]
    fn reconciled_trait_covers_requirement_15_operation_set() {
        // Validates: Requirement 33.2 -- CRUD, resolution, query, GDG, defaults.
        let svc = MockCatalogService;
        assert!(CatalogService::create_dataset(&svc, "A.B", DatasetAttributes::default()).is_ok());
        assert!(CatalogService::delete_dataset(&svc, "A.B").is_ok());
        assert!(CatalogService::update_dataset(&svc, "A.B", DatasetAttributes::default()).is_ok());
        assert!(CatalogService::rename_dataset(&svc, "A.B", "C.D").is_ok());
        assert!(CatalogService::resolve_dsn(&svc, "A.B").is_ok());
        assert!(CatalogService::get_dataset_attributes(&svc, "A.B").is_ok());
        assert!(CatalogService::list_datasets(&svc, &DatasetFilter::default()).is_ok());
        assert!(CatalogService::validate_dsn(&svc, "VALID.DSN").is_ok());
        assert!(CatalogService::create_gdg_base(&svc, "A.GDG", 5, true).is_ok());
        assert!(
            CatalogService::create_generation(&svc, "A.GDG", DatasetAttributes::default()).is_ok()
        );
        assert!(CatalogService::resolve_generation(&svc, "A.GDG", -1).is_ok());
        assert!(CatalogService::list_generations(&svc, "A.GDG").is_ok());
    }

    #[test]
    fn get_allocation_defaults_round_trips_ff_dscatalog_dsorg() {
        // Validates: Requirement 33.4 -- single Dsorg/Recfm enum pair is used.
        let svc = MockCatalogService;
        let defaults = CatalogService::get_allocation_defaults(&svc, Dsorg::PS);
        assert_eq!(defaults.dsorg, Some(Dsorg::PS));
        assert_eq!(defaults.recfm, Some(Recfm::FB));
        let gdg = CatalogService::get_allocation_defaults(&svc, Dsorg::GDG);
        assert_eq!(gdg.dsorg, Some(Dsorg::GDG));
        assert_eq!(gdg.recfm, None);
    }
}
