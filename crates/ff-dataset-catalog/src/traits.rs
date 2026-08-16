//! Trait interface definitions for the Dataset Catalog service.
//!
//! The `CatalogService` trait defines the complete public API contract that
//! dependent crates (ff-dsalloc, ff-idcams) code against. This enables
//! trait-based coupling and mock implementations for testing.

use std::fmt;
use std::path::PathBuf;

// ─── Error Types ────────────────────────────────────────────────────────────

/// Error type for catalog operations.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    /// The specified dataset was not found in any mounted catalog.
    #[error("dataset not found: {dsn}")]
    DatasetNotFound { dsn: String },

    /// The dataset already exists in the catalog.
    #[error("dataset already exists: {dsn}")]
    DatasetAlreadyExists { dsn: String },

    /// The DSN failed validation.
    #[error("invalid DSN: {0}")]
    InvalidDsn(DsnValidationError),

    /// A GDG-specific error occurred.
    #[error("GDG error for base '{base_dsn}': {reason}")]
    GdgError { base_dsn: String, reason: String },

    /// An I/O or storage error occurred.
    #[error("catalog storage error: {0}")]
    StorageError(String),

    /// An internal error occurred.
    #[error("internal catalog error: {0}")]
    Internal(String),
}

/// Error type for DSN validation failures.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DsnValidationError {
    /// DSN is empty.
    #[error("DSN must not be empty")]
    Empty,

    /// DSN exceeds maximum length (44 characters).
    #[error("DSN exceeds maximum length of 44 characters: length={length}")]
    TooLong { length: usize },

    /// A qualifier exceeds maximum length (8 characters).
    #[error("qualifier '{qualifier}' exceeds 8 characters")]
    QualifierTooLong { qualifier: String },

    /// A qualifier contains invalid characters.
    #[error("qualifier '{qualifier}' contains invalid characters")]
    InvalidCharacters { qualifier: String },

    /// DSN has no qualifiers.
    #[error("DSN must have at least one qualifier")]
    NoQualifiers,
}

// ─── Data Types ─────────────────────────────────────────────────────────────

/// Unique identifier for a dataset in the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatasetId(pub String);

/// Dataset organization type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dsorg {
    /// Physical Sequential.
    Ps,
    /// Partitioned (PDS).
    Po,
    /// Direct Access.
    Da,
    /// VSAM (sub-types handled by ff-vsam-services).
    Vsam,
}

/// Record format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recfm {
    /// Fixed-length records.
    F,
    /// Fixed-length blocked records.
    Fb,
    /// Variable-length records.
    V,
    /// Variable-length blocked records.
    Vb,
    /// Undefined-length records.
    U,
}

/// Dataset attributes describing the physical characteristics of a dataset.
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

/// Result of resolving a DSN to a physical path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionResult {
    /// The resolved physical path.
    pub path: PathBuf,
    /// The catalog that contained the entry.
    pub catalog_name: String,
    /// The dataset attributes from the catalog entry.
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
    /// The fully-qualified dataset name.
    pub dsn: String,
    /// The dataset attributes.
    pub attributes: DatasetAttributes,
    /// The catalog containing this entry.
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
    /// The absolute generation number (e.g., G0001V00).
    pub generation_name: String,
    /// The fully-qualified DSN of this generation.
    pub dsn: String,
    /// The physical path.
    pub path: PathBuf,
    /// The relative generation number from current (0 = current, -1 = previous).
    pub relative_offset: i32,
}

// ─── CatalogService Trait ───────────────────────────────────────────────────

/// The primary interface for catalog operations.
///
/// This trait defines the complete set of operations available to external
/// consumers (ff-dsalloc, ff-idcams). Dependent crates depend on this trait
/// rather than concrete implementation types, enabling mock implementations
/// for unit testing.
///
/// # Errors
///
/// All fallible methods return `Result<T, CatalogError>`.
pub trait CatalogService: Send + Sync {
    // ── Dataset CRUD ──

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

    // ── Resolution ──

    /// Resolve a DSN to its physical path.
    fn resolve_dsn(&self, dsn: &str) -> Result<ResolutionResult, CatalogError>;

    /// Check whether a dataset exists in any mounted catalog.
    fn dataset_exists(&self, dsn: &str) -> Result<bool, CatalogError>;

    /// Retrieve the attributes of an existing dataset.
    fn get_dataset_attributes(&self, dsn: &str) -> Result<DatasetAttributes, CatalogError>;

    // ── Query ──

    /// List datasets matching the given filter criteria.
    fn list_datasets(&self, filter: &DatasetFilter) -> Result<Vec<DatasetEntry>, CatalogError>;

    /// Validate a DSN string against naming rules.
    fn validate_dsn(&self, dsn: &str) -> Result<(), DsnValidationError>;

    // ── GDG Operations ──

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

    // ── Defaults ──

    /// Retrieve the configured default attributes for a given dataset organization.
    fn get_allocation_defaults(&self, dsorg: Dsorg) -> DatasetAttributes;
}

// ─── DynCatalogService (Object-Safe Wrapper) ────────────────────────────────

/// Object-safe wrapper trait for dynamic dispatch and mock injection.
///
/// This trait uses concrete `CatalogError` instead of associated types,
/// enabling `Box<dyn DynCatalogService>` for runtime polymorphism.
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

/// Blanket implementation: any `CatalogService + Send + Sync` auto-implements `DynCatalogService`.
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

impl fmt::Display for Dsorg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dsorg::Ps => write!(f, "PS"),
            Dsorg::Po => write!(f, "PO"),
            Dsorg::Da => write!(f, "DA"),
            Dsorg::Vsam => write!(f, "VSAM"),
        }
    }
}

impl fmt::Display for Recfm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Recfm::F => write!(f, "F"),
            Recfm::Fb => write!(f, "FB"),
            Recfm::V => write!(f, "V"),
            Recfm::Vb => write!(f, "VB"),
            Recfm::U => write!(f, "U"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 15 AC 7 — DynCatalogService is object-safe
    #[test]
    fn dyn_catalog_service_is_object_safe() {
        // This test verifies that Box<dyn DynCatalogService> compiles,
        // proving object safety of the trait.
        fn _assert_object_safe(_: Box<dyn DynCatalogService>) {}
    }
}
