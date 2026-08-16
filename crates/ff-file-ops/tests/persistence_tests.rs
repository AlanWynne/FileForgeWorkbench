//! Integration tests for persistence strategies using mock VFS provider.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ff_vfs::{
    CreateOptions, DeleteOptions, OpenOptions, ResourceUri, VfsCapabilities, VfsEntry, VfsError,
    VfsFile, VfsMetadata, VfsProvider,
};

use ff_file_ops::persistence::{
    AtomicWriteStrategy, DeleteFirstStrategy, DirectWriteStrategy, PersistenceStrategy,
};

/// A mock VFS provider that stores files in memory.
struct MockVfsProvider {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    capabilities: VfsCapabilities,
    /// If set, operations will fail with this error.
    fail_on: Arc<Mutex<Option<FailOn>>>,
}

#[derive(Clone)]
enum FailOn {
    Write,
    Rename,
    Delete,
}

struct MockVfsFile {
    path: String,
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

#[async_trait]
impl VfsFile for MockVfsFile {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, VfsError> {
        let files = self.files.lock().unwrap();
        if let Some(data) = files.get(&self.path) {
            let len = buf.len().min(data.len());
            buf[..len].copy_from_slice(&data[..len]);
            Ok(len)
        } else {
            Ok(0)
        }
    }

    async fn write(&mut self, data: &[u8]) -> Result<usize, VfsError> {
        let mut files = self.files.lock().unwrap();
        files.insert(self.path.clone(), data.to_vec());
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

impl MockVfsProvider {
    fn new(capabilities: VfsCapabilities) -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            capabilities,
            fail_on: Arc::new(Mutex::new(None)),
        }
    }

    fn with_file(self, path: &str, content: &[u8]) -> Self {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), content.to_vec());
        self
    }

    fn set_fail_on(&self, fail_on: FailOn) {
        *self.fail_on.lock().unwrap() = Some(fail_on);
    }

    fn clear_fail(&self) {
        *self.fail_on.lock().unwrap() = None;
    }

    fn get_file(&self, path: &str) -> Option<Vec<u8>> {
        self.files.lock().unwrap().get(path).cloned()
    }

    fn file_exists(&self, path: &str) -> bool {
        self.files.lock().unwrap().contains_key(path)
    }
}

#[async_trait]
impl VfsProvider for MockVfsProvider {
    fn scheme(&self) -> &str {
        "mock"
    }

    fn capabilities(&self) -> VfsCapabilities {
        self.capabilities.clone()
    }

    async fn open(&self, path: &str, _options: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError> {
        Ok(Box::new(MockVfsFile {
            path: path.to_string(),
            files: Arc::clone(&self.files),
        }))
    }

    async fn read(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let files = self.files.lock().unwrap();
        files.get(path).cloned().ok_or_else(|| VfsError::NotFound {
            uri: path.to_string(),
            operation: "read".to_string(),
        })
    }

    async fn read_stream(
        &self,
        _path: &str,
    ) -> Result<Pin<Box<dyn tokio::io::AsyncRead + Send>>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "read_stream".to_string(),
            provider: "mock".to_string(),
        })
    }

    async fn write(&self, path: &str, data: &[u8]) -> Result<(), VfsError> {
        if let Some(FailOn::Write) = &*self.fail_on.lock().unwrap() {
            return Err(VfsError::Io {
                uri: path.to_string(),
                operation: "write".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::Other, "simulated write failure"),
            });
        }
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), data.to_vec());
        Ok(())
    }

    async fn create(&self, path: &str, _options: CreateOptions) -> Result<(), VfsError> {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), Vec::new());
        Ok(())
    }

    async fn delete(&self, path: &str, _options: DeleteOptions) -> Result<(), VfsError> {
        if let Some(FailOn::Delete) = &*self.fail_on.lock().unwrap() {
            return Err(VfsError::Io {
                uri: path.to_string(),
                operation: "delete".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::Other, "simulated delete failure"),
            });
        }
        self.files.lock().unwrap().remove(path);
        Ok(())
    }

    async fn rename(&self, old_path: &str, new_path: &str) -> Result<(), VfsError> {
        if let Some(FailOn::Rename) = &*self.fail_on.lock().unwrap() {
            return Err(VfsError::Io {
                uri: old_path.to_string(),
                operation: "rename".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::Other, "simulated rename failure"),
            });
        }
        if !self.capabilities.rename {
            return Err(VfsError::UnsupportedOperation {
                operation: "rename".to_string(),
                provider: "mock".to_string(),
            });
        }
        let mut files = self.files.lock().unwrap();
        if let Some(data) = files.remove(old_path) {
            files.insert(new_path.to_string(), data);
            Ok(())
        } else {
            Err(VfsError::NotFound {
                uri: old_path.to_string(),
                operation: "rename".to_string(),
            })
        }
    }

    async fn list(&self, path: &str) -> Result<Vec<VfsEntry>, VfsError> {
        let files = self.files.lock().unwrap();
        let prefix = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{path}/")
        };
        let entries: Vec<VfsEntry> = files
            .keys()
            .filter(|k| k.starts_with(&prefix) && !k[prefix.len()..].contains('/'))
            .map(|k| {
                let name = k[prefix.len()..].to_string();
                VfsEntry {
                    name,
                    entry_type: ff_vfs::VfsEntryType::File,
                    size: Some(files.get(k).map_or(0, |v| v.len() as u64)),
                    modified: None,
                }
            })
            .collect();
        Ok(entries)
    }

    async fn stat(&self, path: &str) -> Result<VfsMetadata, VfsError> {
        let files = self.files.lock().unwrap();
        if let Some(data) = files.get(path) {
            Ok(VfsMetadata {
                size: Some(data.len() as u64),
                modified: Some(std::time::SystemTime::now()),
                entry_type: ff_vfs::VfsEntryType::File,
                extra: HashMap::new(),
            })
        } else {
            Err(VfsError::NotFound {
                uri: path.to_string(),
                operation: "stat".to_string(),
            })
        }
    }

    async fn exists(&self, path: &str) -> Result<bool, VfsError> {
        Ok(self.files.lock().unwrap().contains_key(path))
    }
}

// --- Tests ---

#[tokio::test]
async fn atomic_write_creates_temp_then_renames() {
    // Validates: Requirement 7 AC 7.1
    let provider = MockVfsProvider::new(VfsCapabilities::all())
        .with_file("/docs/file.txt", b"original content");

    let uri = ResourceUri::new("mock", "/docs/file.txt");
    let strategy = AtomicWriteStrategy;
    let new_content = b"new content here";

    strategy.write(&provider, &uri, new_content).await.unwrap();

    // Target should have new content
    assert_eq!(provider.get_file("/docs/file.txt").unwrap(), new_content);
    // Temp file should be cleaned up (renamed away)
    assert!(!provider.file_exists("/docs/file.txt.tmp"));
}

#[tokio::test]
async fn atomic_write_falls_back_to_direct_when_rename_unsupported() {
    // Validates: Requirement 7 AC 7.2
    let mut caps = VfsCapabilities::all();
    caps.rename = false;

    let provider = MockVfsProvider::new(caps).with_file("/docs/file.txt", b"original content");

    let uri = ResourceUri::new("mock", "/docs/file.txt");
    let strategy = AtomicWriteStrategy;
    let new_content = b"fallback content";

    strategy.write(&provider, &uri, new_content).await.unwrap();

    // Target should have new content (via fallback direct write)
    assert_eq!(provider.get_file("/docs/file.txt").unwrap(), new_content);
}

#[tokio::test]
async fn atomic_write_preserves_original_on_write_failure() {
    // Validates: Requirement 7 AC 7.1 — crash safety property
    let provider = MockVfsProvider::new(VfsCapabilities::all())
        .with_file("/docs/file.txt", b"original content");

    provider.set_fail_on(FailOn::Write);

    let uri = ResourceUri::new("mock", "/docs/file.txt");
    let strategy = AtomicWriteStrategy;

    let result = strategy.write(&provider, &uri, b"new content").await;

    assert!(result.is_err());
    // Original should be preserved
    assert_eq!(
        provider.get_file("/docs/file.txt").unwrap(),
        b"original content"
    );
}

#[tokio::test]
async fn atomic_write_cleans_temp_on_rename_failure() {
    // Validates: Requirement 7 AC 7.9
    let provider = MockVfsProvider::new(VfsCapabilities::all())
        .with_file("/docs/file.txt", b"original content");

    provider.set_fail_on(FailOn::Rename);

    let uri = ResourceUri::new("mock", "/docs/file.txt");
    let strategy = AtomicWriteStrategy;

    let result = strategy.write(&provider, &uri, b"new content").await;

    assert!(result.is_err());
    // Temp file should be cleaned up
    assert!(!provider.file_exists("/docs/file.txt.tmp"));
}

#[tokio::test]
async fn delete_first_strategy_deletes_then_writes() {
    // Validates: Requirement 7 AC 7.6
    let provider =
        MockVfsProvider::new(VfsCapabilities::all()).with_file("/docs/file.txt", b"old content");

    let uri = ResourceUri::new("mock", "/docs/file.txt");
    let strategy = DeleteFirstStrategy;
    let new_content = b"replaced content";

    strategy.write(&provider, &uri, new_content).await.unwrap();

    assert_eq!(provider.get_file("/docs/file.txt").unwrap(), new_content);
}

#[tokio::test]
async fn delete_first_strategy_works_when_target_missing() {
    // Validates: Requirement 7 AC 7.6 — creating a new file via delete_first
    let provider = MockVfsProvider::new(VfsCapabilities::all());

    let uri = ResourceUri::new("mock", "/docs/new_file.txt");
    let strategy = DeleteFirstStrategy;
    let content = b"brand new content";

    strategy.write(&provider, &uri, content).await.unwrap();

    assert_eq!(provider.get_file("/docs/new_file.txt").unwrap(), content);
}

#[tokio::test]
async fn direct_strategy_overwrites_in_place() {
    // Validates: Requirement 7 AC 7.7
    let provider =
        MockVfsProvider::new(VfsCapabilities::all()).with_file("/docs/file.txt", b"old content");

    let uri = ResourceUri::new("mock", "/docs/file.txt");
    let strategy = DirectWriteStrategy;
    let new_content = b"direct overwrite";

    strategy.write(&provider, &uri, new_content).await.unwrap();

    assert_eq!(provider.get_file("/docs/file.txt").unwrap(), new_content);
}

#[tokio::test]
async fn direct_strategy_fails_propagates_error() {
    // Validates: Requirement 7 AC 7.7 — error propagation
    let provider = MockVfsProvider::new(VfsCapabilities::all());
    provider.set_fail_on(FailOn::Write);

    let uri = ResourceUri::new("mock", "/docs/file.txt");
    let strategy = DirectWriteStrategy;

    let result = strategy.write(&provider, &uri, b"content").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn cleanup_temp_files_removes_tmp_files() {
    // Validates: Requirement 7 AC 7.8
    let provider = MockVfsProvider::new(VfsCapabilities::all())
        .with_file("/docs/file.txt", b"normal file")
        .with_file("/docs/file.txt.tmp", b"leftover temp")
        .with_file("/docs/other.tmp", b"another leftover");

    let cleaned = ff_file_ops::cleanup_temp_files(&provider, "/docs").await;

    assert_eq!(cleaned.len(), 2);
    assert!(!provider.file_exists("/docs/file.txt.tmp"));
    assert!(!provider.file_exists("/docs/other.tmp"));
    // Normal file preserved
    assert!(provider.file_exists("/docs/file.txt"));
}

// --- Backup Tests ---

use ff_file_ops::backup::{create_backup, BackupConfig, BackupLocation};

#[tokio::test]
async fn backup_creates_copy_alongside_with_suffix() {
    // Validates: Requirement 7 AC 7.3, 7.4
    let provider = MockVfsProvider::new(VfsCapabilities::all())
        .with_file("/docs/file.txt", b"important content");

    let uri = ResourceUri::new("mock", "/docs/file.txt");
    let config = BackupConfig {
        enabled: true,
        location: BackupLocation::Alongside,
        suffix: ".bak".to_string(),
    };

    create_backup(&provider, &uri, &config).await.unwrap();

    assert_eq!(
        provider.get_file("/docs/file.txt.bak").unwrap(),
        b"important content"
    );
    // Original still intact
    assert_eq!(
        provider.get_file("/docs/file.txt").unwrap(),
        b"important content"
    );
}

#[tokio::test]
async fn backup_creates_copy_in_directory() {
    // Validates: Requirement 7 AC 7.4
    let provider = MockVfsProvider::new(VfsCapabilities::all())
        .with_file("/docs/file.txt", b"directory backup content");

    let uri = ResourceUri::new("mock", "/docs/file.txt");
    let config = BackupConfig {
        enabled: true,
        location: BackupLocation::Directory("/backups".to_string()),
        suffix: ".bak".to_string(),
    };

    create_backup(&provider, &uri, &config).await.unwrap();

    assert_eq!(
        provider.get_file("/backups/file.txt.bak").unwrap(),
        b"directory backup content"
    );
}

#[tokio::test]
async fn backup_disabled_does_nothing() {
    // Validates: Requirement 7 AC 7.3
    let provider =
        MockVfsProvider::new(VfsCapabilities::all()).with_file("/docs/file.txt", b"content");

    let uri = ResourceUri::new("mock", "/docs/file.txt");
    let config = BackupConfig::default(); // disabled

    create_backup(&provider, &uri, &config).await.unwrap();

    // No backup file created
    assert!(!provider.file_exists("/docs/file.txt.bak"));
}

#[tokio::test]
async fn backup_failure_returns_error_but_does_not_panic() {
    // Validates: Requirement 7 AC 7.5
    let provider = MockVfsProvider::new(VfsCapabilities::all());
    // File doesn't exist — read will fail

    let uri = ResourceUri::new("mock", "/docs/missing.txt");
    let config = BackupConfig {
        enabled: true,
        location: BackupLocation::Alongside,
        suffix: ".bak".to_string(),
    };

    let result = create_backup(&provider, &uri, &config).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ff_file_ops::FileOpsError::BackupFailed { .. } => {}
        other => panic!("Expected BackupFailed, got: {other:?}"),
    }
}

#[tokio::test]
async fn backup_write_failure_returns_backup_failed_error() {
    // Validates: Requirement 7 AC 7.5
    let provider =
        MockVfsProvider::new(VfsCapabilities::all()).with_file("/docs/file.txt", b"content");

    provider.set_fail_on(FailOn::Write);

    let uri = ResourceUri::new("mock", "/docs/file.txt");
    let config = BackupConfig {
        enabled: true,
        location: BackupLocation::Alongside,
        suffix: ".bak".to_string(),
    };

    // The read succeeds (fail_on only triggers on write), but writing backup fails
    // Actually we need to be more careful — the read also calls provider.read which is separate
    // Let's clear the fail and set it between read and write... but our mock doesn't support that.
    // Instead, let's just verify the error case conceptually.
    provider.clear_fail();
    // Re-set to fail only on write
    provider.set_fail_on(FailOn::Write);

    let result = create_backup(&provider, &uri, &config).await;
    // The read of original content uses provider.read() which doesn't check fail_on for Write
    // Actually our mock's read() doesn't check fail_on at all. Let me check...
    // Looking at the mock: read() only returns from files HashMap, doesn't check fail_on.
    // write() does check fail_on. So this test should work.
    assert!(result.is_err());
}
