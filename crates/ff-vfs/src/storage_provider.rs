//! StorageProvider trait and capability advertisement for the VFS abstraction layer.
//!
//! Defines `StorageProvider` -- the physical storage backend interface that sits below
//! `VfsProvider`. All physical storage backends implement this trait. The separation
//! keeps physical storage concerns out of VFS routing logic.
//!
//! Addresses: Requirement 9, criteria 9.1-9.5

use std::collections::HashSet;

use crate::error::VfsError;

// === StorageCapability ===================================================

/// Capabilities that a `StorageProvider` may declare.
///
/// Providers declare which operations they support. Callers check capabilities
/// before invoking operations rather than inferring support from dataset type.
///
/// Addresses: Requirement 9 AC 3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StorageCapability {
    /// Provider supports streaming reads.
    StreamRead,
    /// Provider supports streaming writes.
    StreamWrite,
    /// Provider supports record-oriented reads.
    RecordRead,
    /// Provider supports record-oriented writes.
    RecordWrite,
    /// Provider supports keyed (random) access by record key.
    KeyedAccess,
    /// Provider supports relative-record access by record number.
    RelativeAccess,
    /// Provider is append-only; existing records cannot be overwritten.
    AppendOnly,
    /// Provider supports member-level operations (PDS/PDSE).
    MemberOperations,
    /// Provider supports atomic rename without physical relocation.
    AtomicRename,
    /// Provider supports advisory or mandatory record locking.
    Locking,
    /// Provider supports point-in-time snapshots.
    Snapshotting,
    /// Provider can emit watch notifications for content changes.
    WatchNotifications,
}

// === StorageLocator ===================================================

/// An opaque locator that identifies a physical object within a provider.
///
/// The internal representation is intentionally opaque: no UI or editor code
/// constructs or parses raw locator strings directly.
///
/// Addresses: Requirement 9 AC 5
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageLocator(pub(crate) String);

impl StorageLocator {
    /// Creates a new locator from a provider-internal string.
    ///
    /// Only provider and catalogue service code should call this.
    pub fn new(inner: impl Into<String>) -> Self {
        Self(inner.into())
    }

    /// Returns the inner string for use within provider code only.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// === StorageStat ===================================================

/// Metadata returned by `StorageProvider::stat`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageStat {
    /// Logical dataset name or member name.
    pub name: String,
    /// Size in bytes, if known.
    pub size_bytes: Option<u64>,
    /// Whether the object is a container (PDS, directory, etc.).
    pub is_container: bool,
    /// Provider-specific attributes as key-value pairs.
    pub attributes: Vec<(String, String)>,
}

// === StorageProvider trait ===================================================

/// Physical storage backend interface, separate from `VfsProvider`.
///
/// All physical storage backends implement this trait. The separation keeps
/// physical storage concerns (UUID layout, record codecs, SQLite schemas) out
/// of VFS routing logic.
///
/// Object-safe for dynamic dispatch via `dyn StorageProvider + Send + Sync`.
///
/// Addresses: Requirement 9 AC 1, 9.2
pub trait StorageProvider: Send + Sync {
    /// Returns the set of capabilities this provider supports.
    ///
    /// Callers check capabilities before invoking operations.
    ///
    /// Addresses: Requirement 9 AC 3
    fn capabilities(&self) -> HashSet<StorageCapability>;

    /// Returns `true` if this provider supports the given capability.
    fn supports(&self, cap: StorageCapability) -> bool {
        self.capabilities().contains(&cap)
    }

    /// Allocates a new physical object and returns its opaque locator.
    ///
    /// Addresses: Requirement 9 AC 2
    fn allocate(&self, name: &str) -> Result<StorageLocator, VfsError>;

    /// Opens an existing object for reading, returning its raw bytes.
    ///
    /// Addresses: Requirement 9 AC 2
    fn open(&self, locator: &StorageLocator) -> Result<Vec<u8>, VfsError>;

    /// Returns metadata for the object identified by `locator`.
    ///
    /// Addresses: Requirement 9 AC 2
    fn stat(&self, locator: &StorageLocator) -> Result<StorageStat, VfsError>;

    /// Renames the logical name of an object without moving physical content.
    ///
    /// Addresses: Requirement 9 AC 2
    fn rename(&self, locator: &StorageLocator, new_name: &str) -> Result<(), VfsError>;

    /// Deletes the physical object identified by `locator`.
    ///
    /// Addresses: Requirement 9 AC 2
    fn delete(&self, locator: &StorageLocator) -> Result<(), VfsError>;

    /// Lists all objects managed by this provider, returning their locators and names.
    ///
    /// Addresses: Requirement 9 AC 2
    fn list(&self) -> Result<Vec<(StorageLocator, String)>, VfsError>;

    /// Compares provider state with catalogue state and reports discrepancies.
    ///
    /// Returns a list of human-readable discrepancy descriptions. Does not
    /// automatically apply corrections.
    ///
    /// Addresses: Requirement 9 AC 2
    fn reconcile(&self, catalogue_names: &[String]) -> Result<Vec<String>, VfsError>;

    /// Writes data to an existing object identified by `locator`.
    ///
    /// Default returns `VfsError::UnsupportedOperation` for providers that
    /// do not support writes (e.g. append-only or read-only providers).
    ///
    /// Addresses: Requirement 9 AC 3
    fn write(&self, locator: &StorageLocator, _data: &[u8]) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "write".to_string(),
            provider: format!("{locator:?}"),
        })
    }
}

// === Compile-time object-safety assertion ===================================================

fn _assert_object_safety(_: &dyn StorageProvider) {}

// === Tests ===================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // === Minimal mock provider ===================================================

    struct MockStorageProvider {
        caps: HashSet<StorageCapability>,
    }

    impl MockStorageProvider {
        fn with_caps(caps: impl IntoIterator<Item = StorageCapability>) -> Self {
            Self {
                caps: caps.into_iter().collect(),
            }
        }

        fn none() -> Self {
            Self::with_caps([])
        }

        fn read_write() -> Self {
            Self::with_caps([
                StorageCapability::StreamRead,
                StorageCapability::StreamWrite,
            ])
        }
    }

    impl StorageProvider for MockStorageProvider {
        fn capabilities(&self) -> HashSet<StorageCapability> {
            self.caps.clone()
        }

        fn allocate(&self, name: &str) -> Result<StorageLocator, VfsError> {
            Ok(StorageLocator::new(format!("mock::{name}")))
        }

        fn open(&self, locator: &StorageLocator) -> Result<Vec<u8>, VfsError> {
            if self.supports(StorageCapability::StreamRead) {
                Ok(format!("data::{}", locator.as_str()).into_bytes())
            } else {
                Err(VfsError::UnsupportedOperation {
                    operation: "open".to_string(),
                    provider: "mock".to_string(),
                })
            }
        }

        fn stat(&self, locator: &StorageLocator) -> Result<StorageStat, VfsError> {
            Ok(StorageStat {
                name: locator.as_str().to_string(),
                size_bytes: Some(0),
                is_container: false,
                attributes: vec![],
            })
        }

        fn rename(&self, _locator: &StorageLocator, _new_name: &str) -> Result<(), VfsError> {
            Ok(())
        }

        fn delete(&self, _locator: &StorageLocator) -> Result<(), VfsError> {
            Ok(())
        }

        fn list(&self) -> Result<Vec<(StorageLocator, String)>, VfsError> {
            Ok(vec![])
        }

        fn reconcile(&self, _catalogue_names: &[String]) -> Result<Vec<String>, VfsError> {
            Ok(vec![])
        }
    }

    // Validates: Requirement 9.1 -- StorageProvider trait defined separate from VfsProvider
    #[test]
    fn storage_provider_trait_object_is_object_safe() {
        fn _accept(_: &dyn StorageProvider) {}
        fn _accept_boxed(_: Box<dyn StorageProvider + Send + Sync>) {}
        fn _accept_arc(_: Arc<dyn StorageProvider + Send + Sync>) {}
    }

    // Validates: Requirement 9.1 -- mock can be stored as Arc<dyn StorageProvider>
    #[test]
    fn mock_provider_stored_as_arc_dyn() {
        let p: Arc<dyn StorageProvider> = Arc::new(MockStorageProvider::none());
        assert!(p.capabilities().is_empty());
    }

    // Validates: Requirement 9.3 -- providers declare capabilities
    #[test]
    fn capability_advertisement_stream_read_write() {
        let p = MockStorageProvider::read_write();
        assert!(p.supports(StorageCapability::StreamRead));
        assert!(p.supports(StorageCapability::StreamWrite));
        assert!(!p.supports(StorageCapability::KeyedAccess));
    }

    // Validates: Requirement 9.3 -- provider with no capabilities returns empty set
    #[test]
    fn capability_advertisement_none() {
        let p = MockStorageProvider::none();
        assert!(!p.supports(StorageCapability::StreamRead));
        assert!(!p.supports(StorageCapability::RecordRead));
    }

    // Validates: Requirement 9.3 -- default write() returns UnsupportedOperation
    #[test]
    fn default_write_returns_unsupported_operation() {
        let p = MockStorageProvider::none();
        let locator = StorageLocator::new("test");
        let result = p.write(&locator, b"data");
        match result {
            Err(VfsError::UnsupportedOperation { operation, .. }) => {
                assert_eq!(operation, "write");
            }
            other => panic!("expected UnsupportedOperation, got: {other:?}"),
        }
    }

    // Validates: Requirement 9.2 -- allocate returns opaque locator
    #[test]
    fn allocate_returns_locator() {
        let p = MockStorageProvider::none();
        let locator = p.allocate("MY.DATASET").expect("allocate failed");
        assert!(locator.as_str().contains("MY.DATASET"));
    }

    // Validates: Requirement 9.2 -- open delegates to provider
    #[test]
    fn open_with_stream_read_capability_returns_data() {
        let p = MockStorageProvider::read_write();
        let locator = StorageLocator::new("mock::MY.DATASET");
        let data = p.open(&locator).expect("open failed");
        assert!(!data.is_empty());
    }

    // Validates: Requirement 9.2 -- open without capability returns UnsupportedOperation
    #[test]
    fn open_without_stream_read_returns_unsupported() {
        let p = MockStorageProvider::none();
        let locator = StorageLocator::new("mock::MY.DATASET");
        let result = p.open(&locator);
        assert!(matches!(result, Err(VfsError::UnsupportedOperation { .. })));
    }

    // Validates: Requirement 9.2 -- stat returns metadata
    #[test]
    fn stat_returns_storage_stat() {
        let p = MockStorageProvider::none();
        let locator = StorageLocator::new("mock::MY.DATASET");
        let stat = p.stat(&locator).expect("stat failed");
        assert_eq!(stat.name, "mock::MY.DATASET");
        assert!(!stat.is_container);
    }

    // Validates: Requirement 9.2 -- list returns empty vec for mock
    #[test]
    fn list_returns_empty_for_mock() {
        let p = MockStorageProvider::none();
        let entries = p.list().expect("list failed");
        assert!(entries.is_empty());
    }

    // Validates: Requirement 9.2 -- reconcile returns empty discrepancies for mock
    #[test]
    fn reconcile_returns_no_discrepancies_for_mock() {
        let p = MockStorageProvider::none();
        let discrepancies = p
            .reconcile(&["MY.DATASET".to_string()])
            .expect("reconcile failed");
        assert!(discrepancies.is_empty());
    }

    // Validates: Requirement 9.5 -- locator is opaque (no public field access)
    #[test]
    fn storage_locator_opaque_via_as_str() {
        let loc = StorageLocator::new("uuid::abc123");
        assert_eq!(loc.as_str(), "uuid::abc123");
    }

    // Validates: Requirement 9.3 -- all StorageCapability variants are distinct
    #[test]
    fn all_capability_variants_are_distinct() {
        let all = [
            StorageCapability::StreamRead,
            StorageCapability::StreamWrite,
            StorageCapability::RecordRead,
            StorageCapability::RecordWrite,
            StorageCapability::KeyedAccess,
            StorageCapability::RelativeAccess,
            StorageCapability::AppendOnly,
            StorageCapability::MemberOperations,
            StorageCapability::AtomicRename,
            StorageCapability::Locking,
            StorageCapability::Snapshotting,
            StorageCapability::WatchNotifications,
        ];
        let set: HashSet<_> = all.iter().collect();
        assert_eq!(
            set.len(),
            all.len(),
            "duplicate capability variant detected"
        );
    }
}
