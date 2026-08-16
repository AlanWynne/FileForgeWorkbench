//! VFS provider implementation for the dataset catalog.
//!
//! Registers under scheme `"catalog"` and translates VFS operations to
//! catalog operations.

use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use tokio::io::AsyncRead;

use ff_vfs::{
    CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsEntryType, VfsError,
    VfsFile, VfsMetadata, VfsProvider,
};

use crate::catalog_registry::CatalogRegistry;
use crate::dataset::Dsorg;
use crate::dsn::Dsn;

/// VFS provider implementation for the dataset catalog.
///
/// Translates VFS operations (open, read, write, list, stat, etc.) into
/// catalog operations on mounted catalogs.
pub struct CatalogVfsProvider {
    /// The catalog registry managing mounted catalogs.
    /// Uses std::sync::RwLock since rusqlite Connection is not Send.
    registry: Arc<RwLock<CatalogRegistry>>,
}

impl CatalogVfsProvider {
    /// Create a new catalog VFS provider wrapping the given registry.
    pub fn new(registry: Arc<RwLock<CatalogRegistry>>) -> Self {
        Self { registry }
    }

    /// Helper: resolve a DSN to physical path without holding lock across await.
    fn resolve_path(&self, path: &str) -> Result<PathBuf, VfsError> {
        let registry = self.registry.read().map_err(|_| VfsError::Io {
            uri: path.to_string(),
            operation: "lock".to_string(),
            source: std::io::Error::other("lock poisoned"),
        })?;
        let dsn = Dsn::parse(path).map_err(VfsError::from)?;
        let result = registry.resolve(&dsn).map_err(VfsError::from)?;
        Ok(result.physical_path)
    }

    /// Helper: resolve DSN(MEMBER) to physical path.
    fn resolve_member_path(&self, path: &str) -> Result<PathBuf, VfsError> {
        let registry = self.registry.read().map_err(|_| VfsError::Io {
            uri: path.to_string(),
            operation: "lock".to_string(),
            source: std::io::Error::other("lock poisoned"),
        })?;
        let (dsn, member) = Dsn::parse_member_ref(path).map_err(VfsError::from)?;
        let result = registry.resolve(&dsn).map_err(VfsError::from)?;
        if let Some(m) = member {
            Ok(result.physical_path.join(m.as_str()))
        } else {
            Ok(result.physical_path)
        }
    }
}

/// A simple in-memory VFS file handle for dataset content.
struct CatalogFile {
    data: Vec<u8>,
    position: usize,
}

#[async_trait]
impl VfsFile for CatalogFile {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, VfsError> {
        let remaining = &self.data[self.position..];
        let to_read = buf.len().min(remaining.len());
        buf[..to_read].copy_from_slice(&remaining[..to_read]);
        self.position += to_read;
        Ok(to_read)
    }

    async fn write(&mut self, data: &[u8]) -> Result<usize, VfsError> {
        self.data.extend_from_slice(data);
        Ok(data.len())
    }

    async fn flush(&mut self) -> Result<(), VfsError> {
        Ok(())
    }

    async fn sync_all(&mut self) -> Result<(), VfsError> {
        Ok(())
    }

    async fn close(self: Box<Self>) -> Result<(), VfsError> {
        Ok(())
    }
}

// SAFETY: CatalogVfsProvider uses std::sync::RwLock which doesn't need Send on the inner type.
// The lock is never held across await points so this is safe.
unsafe impl Send for CatalogVfsProvider {}
unsafe impl Sync for CatalogVfsProvider {}

#[async_trait]
impl VfsProvider for CatalogVfsProvider {
    fn scheme(&self) -> &str {
        "catalog"
    }

    fn capabilities(&self) -> VfsCapabilities {
        VfsCapabilities {
            read: true,
            write: true,
            watch: false,
            search: false,
            random_access: false,
            append: false,
            rename: true,
            delete: true,
            list: true,
            create_directory: true,
        }
    }

    async fn open(&self, path: &str, _options: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError> {
        let physical_path = self.resolve_path(path)?;
        let data = tokio::fs::read(&physical_path)
            .await
            .map_err(|e| VfsError::Io {
                uri: path.to_string(),
                operation: "open".to_string(),
                source: e,
            })?;
        Ok(Box::new(CatalogFile { data, position: 0 }))
    }

    async fn read(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let physical_path = self.resolve_member_path(path)?;
        tokio::fs::read(&physical_path)
            .await
            .map_err(|e| VfsError::Io {
                uri: path.to_string(),
                operation: "read".to_string(),
                source: e,
            })
    }

    async fn read_stream(&self, path: &str) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
        let physical_path = self.resolve_path(path)?;
        let file = tokio::fs::File::open(&physical_path)
            .await
            .map_err(|e| VfsError::Io {
                uri: path.to_string(),
                operation: "read_stream".to_string(),
                source: e,
            })?;
        Ok(Box::pin(file))
    }

    async fn write(&self, path: &str, data: &[u8]) -> Result<(), VfsError> {
        let physical_path = self.resolve_member_path(path)?;
        tokio::fs::write(&physical_path, data)
            .await
            .map_err(|e| VfsError::Io {
                uri: path.to_string(),
                operation: "write".to_string(),
                source: e,
            })
    }

    async fn create(&self, _path: &str, _options: CreateOptions) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "create".to_string(),
            provider: "catalog".to_string(),
        })
    }

    async fn delete(&self, path: &str, _options: DeleteOptions) -> Result<(), VfsError> {
        let registry = self.registry.read().map_err(|_| VfsError::Io {
            uri: path.to_string(),
            operation: "delete".to_string(),
            source: std::io::Error::other("lock poisoned"),
        })?;
        let dsn = Dsn::parse(path).map_err(VfsError::from)?;
        for catalog in registry.catalogs() {
            if catalog.exists(&dsn).unwrap_or(false) {
                catalog.delete(&dsn).map_err(VfsError::from)?;
                return Ok(());
            }
        }
        Err(VfsError::NotFound {
            uri: path.to_string(),
            operation: "delete".to_string(),
        })
    }

    async fn rename(&self, old_path: &str, new_path: &str) -> Result<(), VfsError> {
        let registry = self.registry.read().map_err(|_| VfsError::Io {
            uri: old_path.to_string(),
            operation: "rename".to_string(),
            source: std::io::Error::other("lock poisoned"),
        })?;
        let old_dsn = Dsn::parse(old_path).map_err(VfsError::from)?;
        let new_dsn = Dsn::parse(new_path).map_err(VfsError::from)?;
        for catalog in registry.catalogs() {
            if catalog.exists(&old_dsn).unwrap_or(false) {
                catalog.rename(&old_dsn, &new_dsn).map_err(VfsError::from)?;
                return Ok(());
            }
        }
        Err(VfsError::NotFound {
            uri: old_path.to_string(),
            operation: "rename".to_string(),
        })
    }

    async fn list(&self, path: &str) -> Result<Vec<VfsEntry>, VfsError> {
        let registry = self.registry.read().map_err(|_| VfsError::Io {
            uri: path.to_string(),
            operation: "list".to_string(),
            source: std::io::Error::other("lock poisoned"),
        })?;

        if path.is_empty() || path == "/" {
            let mounted = registry.list_mounted();
            return Ok(mounted
                .iter()
                .map(|(name, _, _)| VfsEntry {
                    name: name.to_string(),
                    entry_type: VfsEntryType::Directory,
                    size: None,
                    modified: None,
                })
                .collect());
        }

        if let Ok(dsn) = Dsn::parse(path) {
            if let Ok(result) = registry.resolve(&dsn) {
                if result.entry.dsorg == Dsorg::PO {
                    for catalog in registry.catalogs() {
                        if let Ok(members) = catalog.list_members(&dsn) {
                            return Ok(members
                                .iter()
                                .map(|m| VfsEntry {
                                    name: m.name.as_str().to_string(),
                                    entry_type: VfsEntryType::File,
                                    size: Some(m.size),
                                    modified: None,
                                })
                                .collect());
                        }
                    }
                }
            }
        }

        let results = registry
            .listcat(&format!("{path}.*"), None, None)
            .map_err(VfsError::from)?;
        Ok(results
            .iter()
            .map(|entry| {
                let entry_type = match entry.dsorg {
                    Dsorg::PO => VfsEntryType::Directory,
                    _ => VfsEntryType::File,
                };
                VfsEntry {
                    name: entry.dsn.clone(),
                    entry_type,
                    size: None,
                    modified: None,
                }
            })
            .collect())
    }

    async fn stat(&self, path: &str) -> Result<VfsMetadata, VfsError> {
        let (physical_path, entry_type, extra) = {
            let registry = self.registry.read().map_err(|_| VfsError::Io {
                uri: path.to_string(),
                operation: "stat".to_string(),
                source: std::io::Error::other("lock poisoned"),
            })?;
            let dsn = Dsn::parse(path).map_err(VfsError::from)?;
            let result = registry.resolve(&dsn).map_err(VfsError::from)?;

            let entry_type = match result.entry.dsorg {
                Dsorg::PO => VfsEntryType::Directory,
                _ => VfsEntryType::File,
            };

            let mut extra = HashMap::new();
            extra.insert("dsorg".to_string(), result.entry.dsorg.to_string());
            if let Some(recfm) = result.entry.recfm {
                extra.insert("recfm".to_string(), recfm.to_string());
            }
            if let Some(lrecl) = result.entry.lrecl {
                extra.insert("lrecl".to_string(), lrecl.to_string());
            }
            if let Some(blksize) = result.entry.blksize {
                extra.insert("blksize".to_string(), blksize.to_string());
            }

            (result.physical_path, entry_type, extra)
        };

        let size = tokio::fs::metadata(&physical_path)
            .await
            .ok()
            .map(|m| m.len());

        Ok(VfsMetadata {
            size,
            modified: None,
            entry_type,
            extra,
        })
    }

    async fn exists(&self, path: &str) -> Result<bool, VfsError> {
        let registry = self.registry.read().map_err(|_| VfsError::Io {
            uri: path.to_string(),
            operation: "exists".to_string(),
            source: std::io::Error::other("lock poisoned"),
        })?;
        match Dsn::parse(path) {
            Ok(dsn) => Ok(registry.exists(&dsn)),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{AllocParams, Recfm};
    use crate::repository::Repository;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Arc<RwLock<CatalogRegistry>>) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("vfs-test");
        let repo = Repository::new(&path);
        repo.initialize("VFSTEST").unwrap();

        let mut registry = CatalogRegistry::new();
        registry.mount(&path, 1).unwrap();

        let catalog = registry.get_catalog("VFSTEST").unwrap();
        catalog
            .allocate(AllocParams {
                dsn: Dsn::parse("TEST.FILE").unwrap(),
                dsorg: Dsorg::PS,
                recfm: Some(Recfm::FB),
                lrecl: Some(80),
                blksize: Some(27920),
                dir_blocks: None,
                gdg_limit: None,
                gdg_scratch: None,
                subtype: None,
                description: None,
            })
            .unwrap();

        (tmp, Arc::new(RwLock::new(registry)))
    }

    #[test]
    fn scheme_returns_catalog() {
        // Validates: Requirement 10 AC 1
        let (_tmp, reg) = setup();
        let provider = CatalogVfsProvider::new(reg);
        assert_eq!(provider.scheme(), "catalog");
    }

    #[test]
    fn capabilities_correct() {
        // Validates: Requirement 10 AC 11
        let (_tmp, reg) = setup();
        let provider = CatalogVfsProvider::new(reg);
        let caps = provider.capabilities();
        assert!(caps.read);
        assert!(caps.write);
        assert!(caps.list);
        assert!(caps.delete);
        assert!(caps.rename);
        assert!(!caps.watch);
        assert!(!caps.search);
    }

    #[tokio::test]
    async fn exists_returns_true_for_known_dataset() {
        // Validates: Requirement 10 AC 10
        let (_tmp, reg) = setup();
        let provider = CatalogVfsProvider::new(reg);
        assert!(provider.exists("TEST.FILE").await.unwrap());
        assert!(!provider.exists("NO.SUCH.DS").await.unwrap());
    }

    #[tokio::test]
    async fn stat_returns_metadata() {
        // Validates: Requirement 10 AC 4
        let (_tmp, reg) = setup();
        let provider = CatalogVfsProvider::new(reg);
        let meta = provider.stat("TEST.FILE").await.unwrap();
        assert_eq!(meta.entry_type, VfsEntryType::File);
        assert_eq!(meta.extra.get("dsorg").unwrap(), "PS");
        assert_eq!(meta.extra.get("recfm").unwrap(), "FB");
    }

    #[tokio::test]
    async fn list_root_shows_catalogs() {
        // Validates: Requirement 10 AC 3
        let (_tmp, reg) = setup();
        let provider = CatalogVfsProvider::new(reg);
        let entries = provider.list("").await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "VFSTEST");
    }

    #[tokio::test]
    async fn read_write_round_trip() {
        // Validates: Requirement 10 AC 6
        let (_tmp, reg) = setup();
        let provider = CatalogVfsProvider::new(reg);

        let data = b"Hello, dataset!";
        provider.write("TEST.FILE", data).await.unwrap();
        let read_back = provider.read("TEST.FILE").await.unwrap();
        assert_eq!(read_back, data);
    }
}
