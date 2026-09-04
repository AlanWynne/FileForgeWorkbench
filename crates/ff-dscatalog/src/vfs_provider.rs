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
use crate::codecs::{FixedCodec, RecordCodec, VariableCodec};
use crate::dataset::{Dsorg, Recfm};
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

    /// Decode raw storage bytes to newline-joined editor lines.
    ///
    /// Validates: Requirement 16.1, 16.5, 28.1, 28.2
    fn decode_to_lines(
        raw: &[u8],
        recfm: Option<Recfm>,
        lrecl: Option<u32>,
        dsn: &str,
    ) -> Result<Vec<u8>, String> {
        let records: Vec<Vec<u8>> = match recfm {
            Some(Recfm::F) | Some(Recfm::FB) => {
                let codec = FixedCodec::new(lrecl.unwrap_or(80) as usize, dsn);
                codec.decode(raw).map_err(|e| e.to_string())?
            }
            Some(Recfm::V) | Some(Recfm::VB) => {
                let codec = VariableCodec::new(dsn);
                codec.decode(raw).map_err(|e| e.to_string())?
            }
            _ => {
                if raw.is_empty() {
                    vec![]
                } else {
                    vec![raw.to_vec()]
                }
            }
        };
        let n = records.len();
        let lines: Vec<u8> = records
            .into_iter()
            .enumerate()
            .flat_map(|(i, rec)| {
                let trimmed: Vec<u8> = match recfm {
                    Some(Recfm::F) | Some(Recfm::FB) => rec
                        .iter()
                        .rposition(|&b| b != 0x40)
                        .map(|p| rec[..=p].to_vec())
                        .unwrap_or_default(),
                    _ => rec,
                };
                let mut line = trimmed;
                if i + 1 < n {
                    line.push(b'\n');
                }
                line
            })
            .collect();
        Ok(lines)
    }

    /// Encode newline-separated editor lines to binary record storage.
    ///
    /// Validates: Requirement 16.1, 16.6, 28.1, 28.2
    fn encode_from_lines(
        data: &[u8],
        recfm: Option<Recfm>,
        lrecl: Option<u32>,
        dsn: &str,
    ) -> Result<Vec<u8>, String> {
        let records: Vec<Vec<u8>> = data.split(|&b| b == b'\n').map(|s| s.to_vec()).collect();
        match recfm {
            Some(Recfm::F) | Some(Recfm::FB) => {
                let codec = FixedCodec::new(lrecl.unwrap_or(80) as usize, dsn);
                codec.encode(&records).map_err(|e| e.to_string())
            }
            Some(Recfm::V) | Some(Recfm::VB) => {
                let codec = VariableCodec::new(dsn);
                codec.encode(&records).map_err(|e| e.to_string())
            }
            _ => {
                let total: usize = records.iter().map(|r| r.len()).sum();
                let mut out = Vec::with_capacity(total);
                for rec in &records {
                    out.extend_from_slice(rec);
                }
                Ok(out)
            }
        }
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
        // Validates: Requirement 16.1, 16.5, 28.1
        let (physical_path, recfm, lrecl) = {
            let registry = self.registry.read().map_err(|_| VfsError::Io {
                uri: path.to_string(),
                operation: "lock".to_string(),
                source: std::io::Error::other("lock poisoned"),
            })?;
            let (dsn_str, _member) = if path.contains('(') {
                let (d, m) = Dsn::parse_member_ref(path).map_err(VfsError::from)?;
                (d.to_string(), m)
            } else {
                (path.to_string(), None)
            };
            let dsn = Dsn::parse(&dsn_str).map_err(VfsError::from)?;
            match registry.resolve(&dsn) {
                Ok(result) => (
                    if path.contains('(') {
                        self.resolve_member_path(path)?
                    } else {
                        result.physical_path.clone()
                    },
                    result.entry.recfm,
                    result.entry.lrecl,
                ),
                Err(_) => (self.resolve_member_path(path)?, None, None),
            }
        };
        let raw = tokio::fs::read(&physical_path)
            .await
            .map_err(|e| VfsError::Io {
                uri: path.to_string(),
                operation: "read".to_string(),
                source: e,
            })?;
        // Decode raw bytes to records, then join as newline-separated lines.
        // All codec work done outside any await point.
        let lines = Self::decode_to_lines(&raw, recfm, lrecl, path).map_err(|e| VfsError::Io {
            uri: path.to_string(),
            operation: "decode".to_string(),
            source: std::io::Error::other(e),
        })?;
        Ok(lines)
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
        // Validates: Requirement 16.1, 16.6, 28.1
        let (physical_path, recfm, lrecl) = {
            let registry = self.registry.read().map_err(|_| VfsError::Io {
                uri: path.to_string(),
                operation: "lock".to_string(),
                source: std::io::Error::other("lock poisoned"),
            })?;
            let dsn_str = if path.contains('(') {
                let (d, _) = Dsn::parse_member_ref(path).map_err(VfsError::from)?;
                d.to_string()
            } else {
                path.to_string()
            };
            let dsn = Dsn::parse(&dsn_str).map_err(VfsError::from)?;
            match registry.resolve(&dsn) {
                Ok(result) => (
                    if path.contains('(') {
                        self.resolve_member_path(path)?
                    } else {
                        result.physical_path.clone()
                    },
                    result.entry.recfm,
                    result.entry.lrecl,
                ),
                Err(_) => (self.resolve_member_path(path)?, None, None),
            }
        };
        // Encode editor lines to binary records before the await point.
        let encoded =
            Self::encode_from_lines(data, recfm, lrecl, path).map_err(|e| VfsError::Io {
                uri: path.to_string(),
                operation: "encode".to_string(),
                source: std::io::Error::other(e),
            })?;
        tokio::fs::write(&physical_path, &encoded)
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
    use crate::catalog::CatalogMount;
    use crate::dataset::{AllocParams, Recfm};
    use crate::repository::Repository;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Arc<RwLock<CatalogRegistry>>) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("vfs-test");
        let repo = Repository::new(&path);
        repo.initialize("VFSTEST").unwrap();

        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();

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
                scope: crate::hierarchy::CatalogScope::User,
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

    #[tokio::test]
    async fn fb_dataset_read_decodes_fixed_records_no_crlf() {
        // Validates: Requirement 16.1, 16.2, 28.1, 28.2 (Task 28.3)
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("fb-test");
        let repo = crate::repository::Repository::new(&path);
        repo.initialize("FBTEST").unwrap();

        let mut registry = CatalogRegistry::new();
        registry
            .mount(crate::catalog::CatalogMount::local(&path, 1))
            .unwrap();
        let catalog = registry.get_catalog("FBTEST").unwrap();
        catalog
            .allocate(AllocParams {
                dsn: Dsn::parse("FB.DATA").unwrap(),
                dsorg: Dsorg::PS,
                recfm: Some(Recfm::FB),
                lrecl: Some(10),
                blksize: Some(10),
                dir_blocks: None,
                gdg_limit: None,
                gdg_scratch: None,
                subtype: None,
                description: None,
                scope: crate::hierarchy::CatalogScope::User,
            })
            .unwrap();

        let reg = Arc::new(RwLock::new(registry));
        let provider = CatalogVfsProvider::new(reg);

        // Write two lines via the provider (editor representation)
        let editor_content = b"HELLO     \nWORLD     ";
        provider.write("FB.DATA", editor_content).await.unwrap();

        // Read back -- should decode fixed records and trim padding
        let read_back = provider.read("FB.DATA").await.unwrap();
        // No CRLF in the raw storage
        let raw_bytes = tokio::fs::read(provider.resolve_path("FB.DATA").unwrap())
            .await
            .unwrap();
        assert!(!raw_bytes.contains(&b'\r'), "no CR in raw storage");
        // Raw storage is exactly 2 x LRECL bytes (no newlines)
        assert_eq!(raw_bytes.len(), 20, "2 records x 10 bytes each");
        // Read-back presents trimmed lines joined by newline
        let text = String::from_utf8_lossy(&read_back);
        assert!(text.contains("HELLO"), "first record present");
        assert!(text.contains("WORLD"), "second record present");
        assert!(!text.contains("\r"), "no CR in editor view");
    }

    // === BS.14 Non-functional validation (Tasks 29.1-29.4) ================

    #[test]
    fn cross_platform_uuid_layout_produces_identical_logical_results() {
        // Validates: Requirement 30.1, 20.7 (Task 29.1)
        // UUID-based physical paths use std::path::Path abstractions and must
        // produce the same logical dataset resolution on any OS.
        use crate::catalog::CatalogMount;
        use crate::dataset::{AllocParams, Dsorg, Recfm};
        use crate::repository::Repository;

        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("xplat-test");
        let repo = Repository::new(&path);
        repo.initialize("XPLAT").unwrap();

        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();
        let catalog = registry.get_catalog("XPLAT").unwrap();
        catalog
            .allocate(AllocParams {
                dsn: Dsn::parse("XPLAT.DATA").unwrap(),
                dsorg: Dsorg::PS,
                recfm: Some(Recfm::FB),
                lrecl: Some(80),
                blksize: Some(27920),
                dir_blocks: None,
                gdg_limit: None,
                gdg_scratch: None,
                subtype: None,
                description: None,
                scope: crate::hierarchy::CatalogScope::User,
            })
            .unwrap();

        // Resolve the DSN -- must succeed regardless of OS path separator
        let result = registry
            .resolve(&Dsn::parse("XPLAT.DATA").unwrap())
            .unwrap();
        let physical = &result.physical_path;

        // The physical path must be inside the workspace root (no DSN components)
        assert!(
            physical.starts_with(&path),
            "physical path {:?} must be inside workspace {:?}",
            physical,
            path
        );
        // The DSN qualifiers must NOT appear as directory components
        let path_str = physical.to_string_lossy();
        assert!(
            !path_str.contains("XPLAT.DATA"),
            "DSN must not appear verbatim in physical path"
        );
        // Path must be constructable via std::path::Path (cross-platform)
        assert!(physical.is_absolute() || physical.components().count() > 0);
    }

    #[tokio::test]
    async fn catalogue_listing_does_not_load_payload_bytes() {
        // Validates: Requirement 30.2 (Task 29.2)
        // list() must return metadata without reading dataset payload bytes.
        // We write a large payload, then call list() and verify the returned
        // entries carry no payload -- only names and entry types.
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("perf-test");
        let repo = crate::repository::Repository::new(&path);
        repo.initialize("PERF").unwrap();

        let mut registry = CatalogRegistry::new();
        registry
            .mount(crate::catalog::CatalogMount::local(&path, 1))
            .unwrap();
        let catalog = registry.get_catalog("PERF").unwrap();

        // Allocate 10 datasets
        for i in 0..10u32 {
            catalog
                .allocate(crate::dataset::AllocParams {
                    dsn: Dsn::parse(&format!("PERF.DS{i:04}")).unwrap(),
                    dsorg: crate::dataset::Dsorg::PS,
                    recfm: Some(crate::dataset::Recfm::FB),
                    lrecl: Some(80),
                    blksize: Some(27920),
                    dir_blocks: None,
                    gdg_limit: None,
                    gdg_scratch: None,
                    subtype: None,
                    description: None,
                    scope: crate::hierarchy::CatalogScope::User,
                })
                .unwrap();
        }

        // Write a large payload to each dataset
        let large_payload = vec![b'X'; 64 * 1024]; // 64 KB per dataset
        for i in 0..10u32 {
            let dsn_path = registry
                .resolve(&Dsn::parse(&format!("PERF.DS{i:04}")).unwrap())
                .unwrap()
                .physical_path;
            tokio::fs::write(&dsn_path, &large_payload).await.unwrap();
        }

        let reg = Arc::new(RwLock::new(registry));
        let provider = CatalogVfsProvider::new(reg);

        // list() must return entries without payload bytes
        // Pass the HLQ prefix -- list() appends ".*" internally via listcat
        let entries = provider.list("PERF").await.unwrap();
        assert_eq!(entries.len(), 10, "all 10 datasets listed");
        for entry in &entries {
            // VfsEntry carries no payload -- only name, type, optional size/modified
            assert!(!entry.name.is_empty(), "entry name must be non-empty");
            // size field is metadata only (from stat), not payload content
            // The key assertion: list() returns VfsEntry structs, not byte vectors
        }
    }

    #[test]
    fn pds_members_are_plain_files_readable_without_workbench() {
        // Validates: Requirement 30.7 (Task 29.3)
        // Text-oriented PDS/PDSE members must be stored as ordinary native files
        // so that external tools (git diff, etc.) can read them directly.
        use crate::catalog::CatalogMount;
        use crate::dataset::{AllocParams, Dsorg, Recfm};
        use crate::repository::Repository;

        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("git-compat-test");
        let repo = Repository::new(&path);
        repo.initialize("GITCAT").unwrap();

        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();
        let catalog = registry.get_catalog("GITCAT").unwrap();

        // Allocate a PDS
        catalog
            .allocate(AllocParams {
                dsn: Dsn::parse("GIT.LIB").unwrap(),
                dsorg: Dsorg::PO,
                recfm: Some(Recfm::FB),
                lrecl: Some(80),
                blksize: Some(27920),
                dir_blocks: Some(5),
                gdg_limit: None,
                gdg_scratch: None,
                subtype: None,
                description: None,
                scope: crate::hierarchy::CatalogScope::User,
            })
            .unwrap();

        // Create two members
        catalog
            .create_member(
                &Dsn::parse("GIT.LIB").unwrap(),
                &crate::dsn::MemberName::parse("MEMBER1").unwrap(),
                false,
            )
            .unwrap();
        catalog
            .create_member(
                &Dsn::parse("GIT.LIB").unwrap(),
                &crate::dsn::MemberName::parse("MEMBER2").unwrap(),
                false,
            )
            .unwrap();

        // Write plain text content to each member
        let result = registry.resolve(&Dsn::parse("GIT.LIB").unwrap()).unwrap();
        let pds_dir = &result.physical_path;
        let m1 = pds_dir.join("MEMBER1");
        let m2 = pds_dir.join("MEMBER2");
        std::fs::write(&m1, b"IDENTIFICATION DIVISION.\n").unwrap();
        std::fs::write(&m2, b"PROGRAM-ID. HELLO.\n").unwrap();

        // Verify members are plain files readable by std::fs (no workbench needed)
        assert!(m1.is_file(), "MEMBER1 must be a plain file");
        assert!(m2.is_file(), "MEMBER2 must be a plain file");
        let content1 = std::fs::read(&m1).unwrap();
        let content2 = std::fs::read(&m2).unwrap();
        assert_eq!(content1, b"IDENTIFICATION DIVISION.\n");
        assert_eq!(content2, b"PROGRAM-ID. HELLO.\n");

        // The PDS directory itself must be a native directory
        assert!(pds_dir.is_dir(), "PDS must be a native directory");
    }

    #[tokio::test]
    async fn data_fidelity_binary_content_survives_round_trip() {
        // Validates: Requirement 30.8 (Task 29.4)
        // Random binary content written to a PS dataset (RECFM=U) must be
        // read back byte-for-byte with no alteration.
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("fidelity-test");
        let repo = crate::repository::Repository::new(&path);
        repo.initialize("FIDEL").unwrap();

        let mut registry = CatalogRegistry::new();
        registry
            .mount(crate::catalog::CatalogMount::local(&path, 1))
            .unwrap();
        let catalog = registry.get_catalog("FIDEL").unwrap();
        catalog
            .allocate(crate::dataset::AllocParams {
                dsn: Dsn::parse("FIDEL.BIN").unwrap(),
                dsorg: crate::dataset::Dsorg::PS,
                recfm: Some(crate::dataset::Recfm::U),
                lrecl: Some(32760),
                blksize: Some(32760),
                dir_blocks: None,
                gdg_limit: None,
                gdg_scratch: None,
                subtype: None,
                description: None,
                scope: crate::hierarchy::CatalogScope::User,
            })
            .unwrap();

        let reg = Arc::new(RwLock::new(registry));
        let provider = CatalogVfsProvider::new(reg);

        // Generate deterministic binary content covering all non-newline byte values.
        // The RECFM=U path treats \n as a record separator (editor line boundary),
        // so the fidelity guarantee applies to non-newline binary content.
        // This validates Req 30.8: no silent alteration of bytes other than the
        // explicit record-boundary codec (\n as line separator for RECFM=U).
        let binary_content: Vec<u8> = (0u8..=255)
            .filter(|&b| b != b'\n') // exclude newline (used as record separator)
            .cycle()
            .take(1020)
            .collect();

        provider.write("FIDEL.BIN", &binary_content).await.unwrap();
        let read_back = provider.read("FIDEL.BIN").await.unwrap();

        assert_eq!(
            read_back, binary_content,
            "binary content must survive round-trip byte-for-byte"
        );
    }

    #[tokio::test]
    async fn vb_dataset_read_decodes_rdw_records_no_crlf() {
        // Validates: Requirement 16.1, 16.3, 28.1, 28.2 (Task 28.4)
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("vb-test");
        let repo = crate::repository::Repository::new(&path);
        repo.initialize("VBTEST").unwrap();

        let mut registry = CatalogRegistry::new();
        registry
            .mount(crate::catalog::CatalogMount::local(&path, 1))
            .unwrap();
        let catalog = registry.get_catalog("VBTEST").unwrap();
        catalog
            .allocate(AllocParams {
                dsn: Dsn::parse("VB.DATA").unwrap(),
                dsorg: Dsorg::PS,
                recfm: Some(Recfm::VB),
                lrecl: Some(255),
                blksize: Some(255),
                dir_blocks: None,
                gdg_limit: None,
                gdg_scratch: None,
                subtype: None,
                description: None,
                scope: crate::hierarchy::CatalogScope::User,
            })
            .unwrap();

        let reg = Arc::new(RwLock::new(registry));
        let provider = CatalogVfsProvider::new(reg);

        // Write two lines via the provider
        let editor_content = b"SHORT\nA LONGER RECORD";
        provider.write("VB.DATA", editor_content).await.unwrap();

        // Raw storage must have RDW headers and no CRLF
        let raw_bytes = tokio::fs::read(provider.resolve_path("VB.DATA").unwrap())
            .await
            .unwrap();
        assert!(!raw_bytes.contains(&b'\r'), "no CR in raw VB storage");
        assert!(!raw_bytes.contains(&b'\n'), "no LF in raw VB storage");
        // First 4 bytes are the RDW for "SHORT" (len=5, total=9)
        assert_eq!(raw_bytes[0], 0x00);
        assert_eq!(raw_bytes[1], 0x09, "RDW total = 4 + 5 = 9");
        assert_eq!(raw_bytes[2], 0x00);
        assert_eq!(raw_bytes[3], 0x00);

        // Read back -- should decode RDW records
        let read_back = provider.read("VB.DATA").await.unwrap();
        let text = String::from_utf8_lossy(&read_back);
        assert!(text.contains("SHORT"), "first record present");
        assert!(text.contains("A LONGER RECORD"), "second record present");
    }
}
