//! StorageProvider abstraction layer.
//!
//! Separates physical access from catalogue resolution so that alternative
//! storage backends can be added without changing dataset editors or catalogue
//! consumers.
//!
//! Validates: Requirement 19.1, 19.2, 19.3, 19.4

mod native;
mod sqlite_record;

pub use native::NativeFileProvider;
pub use sqlite_record::{
    KeyCollation, KeyDefinition, KeyType, KsdsKeyDefinition, KsdsRecord, PrimaryKeyDefinition,
    SqliteRecordProvider,
};

use std::path::PathBuf;

use crate::error::CatalogError;

/// A stable UUID identifying a physical storage object.
pub type ObjectId = uuid::Uuid;

/// Metadata returned by `StorageProvider::stat`.
#[derive(Debug, Clone)]
pub struct ObjectStat {
    /// Size in bytes of the physical object.
    pub size: u64,
    /// Whether this object is a container (directory / library).
    pub is_container: bool,
    /// Provider-specific opaque locator string (not for UI display).
    pub locator: String,
}

/// Capabilities a provider may advertise.
///
/// Callers check capabilities rather than inferring them from dataset type.
/// Validates: Requirement 19.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderCapability {
    StreamRead,
    StreamWrite,
    RecordRead,
    RecordWrite,
    KeyedAccess,
    RelativeAccess,
    AppendOnly,
    MemberOperations,
    AtomicRename,
}

/// Abstraction over physical storage for dataset objects.
///
/// Implementations must map to `CatalogError` variants for the common error
/// taxonomy. Provider-specific locators are opaque outside the provider and
/// catalogue services -- UI code must not construct or parse them.
///
/// Validates: Requirement 19.1, 19.3, 19.4
pub trait StorageProvider: Send + Sync {
    /// The set of capabilities this provider supports.
    fn capabilities(&self) -> &[ProviderCapability];

    /// Allocate a new physical object, returning its stable UUID and locator.
    ///
    /// The UUID is assigned here and stored in the catalogue.
    fn allocate(
        &self,
        workspace_root: &std::path::Path,
        is_container: bool,
    ) -> Result<(ObjectId, String), CatalogError>;

    /// Open a physical object for reading, returning its resolved path.
    fn open(
        &self,
        workspace_root: &std::path::Path,
        locator: &str,
    ) -> Result<PathBuf, CatalogError>;

    /// Return metadata for a physical object.
    fn stat(
        &self,
        workspace_root: &std::path::Path,
        locator: &str,
    ) -> Result<ObjectStat, CatalogError>;

    /// Rename a physical object's locator entry (catalogue-only for UUID layout).
    ///
    /// For UUID-based providers this is a no-op on the filesystem -- only the
    /// catalogue entry changes. Validates: Requirement 20.6
    fn rename(
        &self,
        workspace_root: &std::path::Path,
        locator: &str,
        new_locator: &str,
    ) -> Result<(), CatalogError>;

    /// Delete a physical object.
    fn delete(&self, workspace_root: &std::path::Path, locator: &str) -> Result<(), CatalogError>;

    /// List child locators for a container object (e.g. PDS members).
    fn list(
        &self,
        workspace_root: &std::path::Path,
        locator: &str,
    ) -> Result<Vec<String>, CatalogError>;

    /// Compare catalogue entries with physical objects and report discrepancies.
    ///
    /// Returns a list of human-readable discrepancy descriptions.
    /// Does not auto-apply corrections. Validates: Requirement 27.1, 27.2, 27.3
    fn reconcile(
        &self,
        workspace_root: &std::path::Path,
        known_locators: &[String],
    ) -> Result<Vec<String>, CatalogError>;
}
