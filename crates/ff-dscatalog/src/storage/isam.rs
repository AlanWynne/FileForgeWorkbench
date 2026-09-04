//! ISAM (Indexed Sequential Access Method) storage provider.
//!
//! ISAM shares the indexed-record interface with KSDS by wrapping
//! `SqliteRecordProvider`.  Primary access is by key; secondary access is
//! provided through alternate indexes registered on the same SQLite database.
//! All ISAM implementation details are encapsulated behind `StorageProvider`
//! so callers never depend on ISAM-specific types.
//!
//! Validates: Requirement 24.1, 24.2, 24.3

use std::path::{Path, PathBuf};
use std::sync::Arc;

use uuid::Uuid;

use crate::error::CatalogError;

use super::{
    AlternateIndex, KsdsKeyDefinition, KsdsRecord, ObjectId, ObjectStat, ProviderCapability,
    SqliteRecordProvider, StorageProvider,
};

/// ISAM provider backed by `SqliteRecordProvider`.
///
/// Exposes the same keyed-record interface as KSDS.  Secondary access paths
/// are registered as alternate indexes on the underlying SQLite database.
/// No ISAM-specific types are visible to callers -- all interaction goes
/// through `StorageProvider` or the shared record-access methods.
///
/// Validates: Requirement 24.1, 24.2, 24.3
pub struct IsamProvider {
    inner: Arc<SqliteRecordProvider>,
}

impl std::fmt::Debug for IsamProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IsamProvider")
            .field("inner", &self.inner)
            .finish()
    }
}

impl IsamProvider {
    /// Open or create an ISAM database for `dataset_id`.
    ///
    /// The physical file is stored at
    /// `<repository_root>/indexed/<uuid>.sqlite`, identical to KSDS layout,
    /// because ISAM shares the same indexed-record infrastructure.
    pub fn open(
        repository_root: impl AsRef<Path>,
        dataset_id: Uuid,
        key_definition: KsdsKeyDefinition,
    ) -> Result<Self, CatalogError> {
        let inner = SqliteRecordProvider::open(repository_root, dataset_id, key_definition)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Insert a record by explicit key.
    pub fn insert(&self, key: &str, data: &[u8]) -> Result<(), CatalogError> {
        self.inner.insert(key, data)
    }

    /// Read a record by primary key.
    pub fn read(&self, key: &str) -> Result<Option<KsdsRecord>, CatalogError> {
        self.inner.read(key)
    }

    /// Update the payload for an existing key.
    pub fn update(&self, key: &str, data: &[u8]) -> Result<bool, CatalogError> {
        self.inner.update(key, data)
    }

    /// Delete a record by primary key.
    pub fn delete_record(&self, key: &str) -> Result<bool, CatalogError> {
        self.inner.delete(key)
    }

    /// Read all records in primary-key order.
    pub fn sequential_read(&self) -> Result<Vec<KsdsRecord>, CatalogError> {
        self.inner.sequential_read()
    }

    /// Register a secondary access path (alternate index).
    ///
    /// Validates: Requirement 24.2
    pub fn add_secondary_index(&self, definition: &AlternateIndex) -> Result<(), CatalogError> {
        self.inner.add_alternate_index(definition)
    }

    /// Populate a secondary index from the current record set.
    pub fn rebuild_secondary_index(&self, name: &str) -> Result<(), CatalogError> {
        self.inner.rebuild_alternate_index(name)
    }

    /// Look up primary keys via a secondary index.
    ///
    /// Validates: Requirement 24.2
    pub fn lookup_by_secondary_key(
        &self,
        index_name: &str,
        key: &str,
    ) -> Result<Vec<String>, CatalogError> {
        self.inner.lookup_by_alternate_key(index_name, key)
    }
}

impl StorageProvider for IsamProvider {
    fn capabilities(&self) -> &[ProviderCapability] {
        static CAPABILITIES: [ProviderCapability; 3] = [
            ProviderCapability::RecordRead,
            ProviderCapability::RecordWrite,
            ProviderCapability::KeyedAccess,
        ];
        &CAPABILITIES
    }

    fn allocate(
        &self,
        workspace_root: &Path,
        is_container: bool,
    ) -> Result<(ObjectId, String), CatalogError> {
        self.inner.allocate(workspace_root, is_container)
    }

    fn open(&self, workspace_root: &Path, locator: &str) -> Result<PathBuf, CatalogError> {
        self.inner.open(workspace_root, locator)
    }

    fn stat(&self, workspace_root: &Path, locator: &str) -> Result<ObjectStat, CatalogError> {
        self.inner.stat(workspace_root, locator)
    }

    fn rename(
        &self,
        workspace_root: &Path,
        locator: &str,
        new_locator: &str,
    ) -> Result<(), CatalogError> {
        self.inner.rename(workspace_root, locator, new_locator)
    }

    fn delete(&self, workspace_root: &Path, locator: &str) -> Result<(), CatalogError> {
        StorageProvider::delete(self.inner.as_ref(), workspace_root, locator)
    }

    fn list(&self, workspace_root: &Path, locator: &str) -> Result<Vec<String>, CatalogError> {
        self.inner.list(workspace_root, locator)
    }

    fn reconcile(
        &self,
        workspace_root: &Path,
        known_locators: &[String],
    ) -> Result<Vec<String>, CatalogError> {
        self.inner.reconcile(workspace_root, known_locators)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::KeyCollation;
    use tempfile::TempDir;

    fn provider() -> (TempDir, IsamProvider) {
        let dir = tempfile::tempdir().expect("tempdir");
        let id = Uuid::new_v4();
        let provider =
            IsamProvider::open(dir.path(), id, KsdsKeyDefinition::new(0, 3)).expect("open");
        (dir, provider)
    }

    // === Requirement 24.1 -- shared indexed-record interface with KSDS ===

    #[test]
    fn isam_primary_key_insert_and_read() {
        // Validates: Requirement 24.1
        let (_dir, provider) = provider();
        provider.insert("AAA", b"AAAdata").unwrap();
        let record = provider.read("AAA").unwrap().unwrap();
        assert_eq!(record.key, "AAA");
        assert_eq!(record.data, b"AAAdata");
    }

    #[test]
    fn isam_primary_key_uniqueness_enforced() {
        // Validates: Requirement 24.1
        let (_dir, provider) = provider();
        provider.insert("KEY", b"first").unwrap();
        let err = provider.insert("KEY", b"second").unwrap_err();
        assert!(matches!(err, CatalogError::SqliteError { .. }));
        assert_eq!(provider.read("KEY").unwrap().unwrap().data, b"first");
    }

    #[test]
    fn isam_sequential_read_returns_records_in_key_order() {
        // Validates: Requirement 24.1
        let (_dir, provider) = provider();
        provider.insert("CCC", b"CCCdata").unwrap();
        provider.insert("AAA", b"AAAdata").unwrap();
        provider.insert("BBB", b"BBBdata").unwrap();
        let keys: Vec<_> = provider
            .sequential_read()
            .unwrap()
            .into_iter()
            .map(|r| r.key)
            .collect();
        assert_eq!(keys, ["AAA", "BBB", "CCC"]);
    }

    #[test]
    fn isam_update_and_delete() {
        // Validates: Requirement 24.1
        let (_dir, provider) = provider();
        provider.insert("KEY", b"original").unwrap();
        assert!(provider.update("KEY", b"updated").unwrap());
        assert_eq!(provider.read("KEY").unwrap().unwrap().data, b"updated");
        assert!(provider.delete_record("KEY").unwrap());
        assert!(provider.read("KEY").unwrap().is_none());
    }

    // === Requirement 24.2 -- SQLite indexes for secondary access paths ===

    #[test]
    fn isam_secondary_index_lookup_returns_matching_primary_keys() {
        // Validates: Requirement 24.2
        let (_dir, provider) = provider();
        // Record layout: key(3) + dept(3) + rest
        provider.insert("A01", b"A01ENG payload").unwrap();
        provider.insert("B02", b"B02ENG payload").unwrap();
        provider.insert("C03", b"C03MKT payload").unwrap();
        let secondary = AlternateIndex {
            name: "BY_DEPT".to_string(),
            offset: 3,
            length: 3,
            unique: false,
            collation: KeyCollation::Binary,
        };
        provider.add_secondary_index(&secondary).unwrap();
        provider.rebuild_secondary_index("BY_DEPT").unwrap();
        let mut eng_keys = provider.lookup_by_secondary_key("BY_DEPT", "ENG").unwrap();
        eng_keys.sort();
        assert_eq!(eng_keys, ["A01", "B02"]);
        let mkt_keys = provider.lookup_by_secondary_key("BY_DEPT", "MKT").unwrap();
        assert_eq!(mkt_keys, ["C03"]);
    }

    #[test]
    fn isam_multiple_secondary_indexes_coexist() {
        // Validates: Requirement 24.2
        let (_dir, provider) = provider();
        // Record layout: key(3) + dept(3) + grade(2)
        provider.insert("A01", b"A01ENG01").unwrap();
        provider.insert("B02", b"B02MKT02").unwrap();
        provider.insert("C03", b"C03ENG02").unwrap();
        provider
            .add_secondary_index(&AlternateIndex {
                name: "BY_DEPT".to_string(),
                offset: 3,
                length: 3,
                unique: false,
                collation: KeyCollation::Binary,
            })
            .unwrap();
        provider
            .add_secondary_index(&AlternateIndex {
                name: "BY_GRADE".to_string(),
                offset: 6,
                length: 2,
                unique: false,
                collation: KeyCollation::Binary,
            })
            .unwrap();
        provider.rebuild_secondary_index("BY_DEPT").unwrap();
        provider.rebuild_secondary_index("BY_GRADE").unwrap();
        let mut eng = provider.lookup_by_secondary_key("BY_DEPT", "ENG").unwrap();
        eng.sort();
        assert_eq!(eng, ["A01", "C03"]);
        let mut grade02 = provider.lookup_by_secondary_key("BY_GRADE", "02").unwrap();
        grade02.sort();
        assert_eq!(grade02, ["B02", "C03"]);
    }

    // === Requirement 24.3 -- encapsulated behind StorageProvider ===

    #[test]
    fn isam_provider_implements_storage_provider_trait() {
        // Validates: Requirement 24.3
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let provider: Box<dyn StorageProvider> =
            Box::new(IsamProvider::open(dir.path(), id, KsdsKeyDefinition::new(0, 3)).unwrap());
        assert!(provider
            .capabilities()
            .contains(&ProviderCapability::KeyedAccess));
    }

    #[test]
    fn isam_storage_provider_allocate_and_stat() {
        // Validates: Requirement 24.3
        let dir = tempfile::tempdir().unwrap();
        let id = Uuid::new_v4();
        let provider = IsamProvider::open(dir.path(), id, KsdsKeyDefinition::new(0, 3)).unwrap();
        let (new_id, locator) = provider.allocate(dir.path(), false).unwrap();
        assert_ne!(new_id, id);
        let stat = provider.stat(dir.path(), &locator).unwrap();
        assert!(!stat.is_container);
    }
}
