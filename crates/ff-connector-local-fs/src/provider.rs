//! LocalFsProvider — implements VfsProvider for the host OS filesystem.
//!
//! This is the primary VFS provider for FileForgeWorkbench. All local filesystem
//! operations are performed asynchronously via Tokio.
//!
//! Addresses: Requirement 1, all acceptance criteria

use std::pin::Pin;

use async_trait::async_trait;
use futures_core::Stream;
use tokio::io::AsyncRead;

use ff_vfs::search::{SearchOptions, SearchQuery, VfsSearchResult};
use ff_vfs::types::{
    CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsEntryType,
    VfsMetadata, WatchOptions,
};
use ff_vfs::watch::WatchHandle;
use ff_vfs::{VfsError, VfsFile, VfsProvider};

use crate::config::LocalFsConfig;
use crate::error::map_io_error;
use crate::metadata;
use crate::path::{NativePath, PathResolver};
use crate::streaming::{AtomicWriter, ChunkedReader, StreamingManager};
use crate::watcher::FileWatcher;

/// The primary VFS provider for the host operating system's native filesystem.
///
/// Registered with the ProviderRegistry under scheme `"local"`.
///
/// Addresses: Requirement 1, all acceptance criteria
pub struct LocalFsProvider {
    /// Path resolver for URI ↔ native path conversion.
    path_resolver: PathResolver,
    /// File watcher manager for OS-native change notifications.
    file_watcher: FileWatcher,
    /// Streaming I/O manager for large file support.
    streaming_manager: StreamingManager,
    /// Configuration for this provider.
    config: LocalFsConfig,
}

impl LocalFsProvider {
    /// Construct a new `LocalFsProvider` with the given configuration.
    ///
    /// Validates: Requirement 1, criterion 1
    pub fn new(config: LocalFsConfig) -> Result<Self, VfsError> {
        let path_resolver = PathResolver::new()?;
        let debounce = config.debounce_duration();
        let file_watcher = FileWatcher::new(debounce)?;
        let streaming_manager =
            StreamingManager::new(config.effective_chunk_size(), config.enable_mmap);

        Ok(Self {
            path_resolver,
            file_watcher,
            streaming_manager,
            config,
        })
    }

    /// Construct with default configuration.
    pub fn with_defaults() -> Result<Self, VfsError> {
        Self::new(LocalFsConfig::default())
    }

    /// Returns a reference to the path resolver.
    pub fn path_resolver(&self) -> &PathResolver {
        &self.path_resolver
    }

    /// Returns a reference to the file watcher.
    pub fn file_watcher(&self) -> &FileWatcher {
        &self.file_watcher
    }

    /// Returns a reference to the streaming manager.
    pub fn streaming_manager(&self) -> &StreamingManager {
        &self.streaming_manager
    }

    /// Returns the configuration.
    pub fn config(&self) -> &LocalFsConfig {
        &self.config
    }

    /// Resolve a VFS path to a native path, building the URI string.
    fn resolve_path(&self, path: &str) -> Result<(NativePath, String), VfsError> {
        let native = self.path_resolver.resolve(path)?;
        let uri = format!("vfs://local{}", PathResolver::native_to_uri_path(&native));
        Ok((native, uri))
    }
}

#[async_trait]
impl VfsProvider for LocalFsProvider {
    /// Returns `"local"`.
    ///
    /// Validates: Requirement 1 AC 2
    fn scheme(&self) -> &str {
        "local"
    }

    /// Full capabilities: read, write, watch, search, random_access,
    /// append, rename, delete, list, create_directory.
    fn capabilities(&self) -> VfsCapabilities {
        VfsCapabilities::all()
    }

    /// Open a local file for read/write.
    ///
    /// Validates: Requirement 1, criterion 3
    async fn open(&self, path: &str, options: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError> {
        let (native, uri) = self.resolve_path(path)?;

        let mut open_opts = tokio::fs::OpenOptions::new();
        open_opts
            .read(options.read)
            .write(options.write)
            .create(options.create)
            .truncate(options.truncate)
            .append(options.append);

        let file = open_opts
            .open(native.as_path())
            .await
            .map_err(|e| map_io_error(e, "open", &uri))?;

        Ok(Box::new(LocalFile { file }))
    }

    /// Read entire file content into memory.
    ///
    /// Validates: Requirement 1, criterion 3
    async fn read(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let (native, uri) = self.resolve_path(path)?;
        tokio::fs::read(native.as_path())
            .await
            .map_err(|e| map_io_error(e, "read", &uri))
    }

    /// Read file as an async byte stream (chunked).
    ///
    /// Validates: Requirement 6, criteria 1–2
    async fn read_stream(&self, path: &str) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
        let (native, _uri) = self.resolve_path(path)?;
        let reader = self.streaming_manager.open_reader(&native, None).await?;
        Ok(Box::pin(reader))
    }

    /// Write content to a file using atomic write (temp + rename).
    ///
    /// Validates: Requirement 1, criterion 4
    async fn write(&self, path: &str, data: &[u8]) -> Result<(), VfsError> {
        let (native, _uri) = self.resolve_path(path)?;
        let writer = AtomicWriter::new(&native).await?;
        writer.write_all(data).await
    }

    /// Create a file or directory with parent directory creation.
    ///
    /// Validates: Requirement 1, criteria 5–6
    async fn create(&self, path: &str, options: CreateOptions) -> Result<(), VfsError> {
        let (native, uri) = self.resolve_path(path)?;

        if options.is_directory {
            if options.create_parents {
                tokio::fs::create_dir_all(native.as_path())
                    .await
                    .map_err(|e| map_io_error(e, "create", &uri))?;
            } else {
                tokio::fs::create_dir(native.as_path())
                    .await
                    .map_err(|e| map_io_error(e, "create", &uri))?;
            }
        } else {
            // Create parent directories if requested
            if options.create_parents {
                if let Some(parent) = native.as_path().parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(|e| map_io_error(e, "create", &uri))?;
                }
            }
            // Create the file (fail if exists by using create_new semantics)
            tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(native.as_path())
                .await
                .map_err(|e| map_io_error(e, "create", &uri))?;
        }

        Ok(())
    }

    /// Delete a file or directory.
    ///
    /// Validates: Requirement 1, criterion 7
    async fn delete(&self, path: &str, options: DeleteOptions) -> Result<(), VfsError> {
        let (native, uri) = self.resolve_path(path)?;

        let meta = tokio::fs::metadata(native.as_path())
            .await
            .map_err(|e| map_io_error(e, "delete", &uri))?;

        if meta.is_dir() {
            if options.recursive {
                tokio::fs::remove_dir_all(native.as_path())
                    .await
                    .map_err(|e| map_io_error(e, "delete", &uri))?;
            } else {
                tokio::fs::remove_dir(native.as_path())
                    .await
                    .map_err(|e| map_io_error(e, "delete", &uri))?;
            }
        } else {
            tokio::fs::remove_file(native.as_path())
                .await
                .map_err(|e| map_io_error(e, "delete", &uri))?;
        }

        Ok(())
    }

    /// Rename/move a resource using native OS rename.
    ///
    /// Validates: Requirement 1, criterion 8
    async fn rename(&self, old_path: &str, new_path: &str) -> Result<(), VfsError> {
        let (old_native, old_uri) = self.resolve_path(old_path)?;
        let (new_native, _new_uri) = self.resolve_path(new_path)?;

        tokio::fs::rename(old_native.as_path(), new_native.as_path())
            .await
            .map_err(|e| map_io_error(e, "rename", &old_uri))
    }

    /// List directory entries.
    ///
    /// Validates: Requirement 1, criterion 9
    async fn list(&self, path: &str) -> Result<Vec<VfsEntry>, VfsError> {
        let (native, uri) = self.resolve_path(path)?;

        let mut entries = Vec::new();
        let mut read_dir = tokio::fs::read_dir(native.as_path())
            .await
            .map_err(|e| map_io_error(e, "list", &uri))?;

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| map_io_error(e, "list", &uri))?
        {
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = entry
                .file_type()
                .await
                .map_err(|e| map_io_error(e, "list", &uri))?;

            let entry_type = if file_type.is_dir() {
                VfsEntryType::Directory
            } else if file_type.is_symlink() {
                VfsEntryType::Symlink
            } else if file_type.is_file() {
                VfsEntryType::File
            } else {
                VfsEntryType::Other
            };

            let metadata = entry.metadata().await.ok();
            let size = metadata.as_ref().map(|m| m.len());
            let modified = metadata.and_then(|m| m.modified().ok());

            entries.push(VfsEntry {
                name,
                entry_type,
                size,
                modified,
            });
        }

        Ok(entries)
    }

    /// Get file/directory metadata.
    ///
    /// Validates: Requirement 5, all criteria
    async fn stat(&self, path: &str) -> Result<VfsMetadata, VfsError> {
        let (native, _uri) = self.resolve_path(path)?;

        let meta = metadata::stat(native.as_path(), true).await?;

        let entry_type = match meta.resource_type {
            metadata::ResourceType::RegularFile => VfsEntryType::File,
            metadata::ResourceType::Directory => VfsEntryType::Directory,
            metadata::ResourceType::Symlink => VfsEntryType::Symlink,
            metadata::ResourceType::Other => VfsEntryType::Other,
        };

        Ok(VfsMetadata {
            size: Some(meta.size),
            modified: meta.modified,
            entry_type,
            extra: std::collections::HashMap::new(),
        })
    }

    /// Check if a path exists.
    async fn exists(&self, path: &str) -> Result<bool, VfsError> {
        let (native, _uri) = self.resolve_path(path)?;
        Ok(tokio::fs::try_exists(native.as_path())
            .await
            .unwrap_or(false))
    }

    /// Register a file/directory watch.
    ///
    /// Validates: Requirement 3, all criteria
    async fn watch(&self, path: &str, options: WatchOptions) -> Result<WatchHandle, VfsError> {
        let (native, _uri) = self.resolve_path(path)?;

        let (_watch_id, rx) = self.file_watcher.watch(&native, options.recursive).await?;

        let cancel_token = tokio_util::sync::CancellationToken::new();
        Ok(WatchHandle::new(rx, cancel_token))
    }

    /// Search file content within a directory tree.
    async fn search(
        &self,
        path: &str,
        query: &SearchQuery,
        options: &SearchOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = VfsSearchResult> + Send>>, VfsError> {
        let (native, _uri) = self.resolve_path(path)?;
        let native_str = native.to_string_lossy().to_string();

        // Use the fallback_search from ff-vfs which uses list + read_stream
        let provider: std::sync::Arc<dyn VfsProvider> = std::sync::Arc::new(SearchProxy {
            path_resolver: PathResolver::with_dirs(
                self.path_resolver.working_directory().to_path_buf(),
                self.path_resolver.home_directory().to_path_buf(),
            ),
            chunk_size: self.streaming_manager.chunk_size(),
        });

        let cancel = tokio_util::sync::CancellationToken::new();
        let stream =
            ff_vfs::fallback_search(provider, &native_str, "local", query, options, cancel).await;

        Ok(stream)
    }
}

/// A minimal proxy provider used by fallback_search that delegates to local FS ops.
struct SearchProxy {
    path_resolver: PathResolver,
    chunk_size: usize,
}

#[async_trait]
impl VfsProvider for SearchProxy {
    fn scheme(&self) -> &str {
        "local"
    }

    fn capabilities(&self) -> VfsCapabilities {
        VfsCapabilities::all()
    }

    async fn open(&self, _path: &str, _options: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "open".to_string(),
            provider: "search_proxy".to_string(),
        })
    }

    async fn read(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let native = self.path_resolver.resolve(path)?;
        let uri = format!("vfs://local{}", PathResolver::native_to_uri_path(&native));
        tokio::fs::read(native.as_path())
            .await
            .map_err(|e| map_io_error(e, "read", &uri))
    }

    async fn read_stream(&self, path: &str) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
        let native = self.path_resolver.resolve(path)?;
        let reader = ChunkedReader::open(&native, self.chunk_size, None).await?;
        Ok(Box::pin(reader))
    }

    async fn write(&self, _path: &str, _data: &[u8]) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "write".to_string(),
            provider: "search_proxy".to_string(),
        })
    }

    async fn create(&self, _path: &str, _options: CreateOptions) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "create".to_string(),
            provider: "search_proxy".to_string(),
        })
    }

    async fn delete(&self, _path: &str, _options: DeleteOptions) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "delete".to_string(),
            provider: "search_proxy".to_string(),
        })
    }

    async fn rename(&self, _old_path: &str, _new_path: &str) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "rename".to_string(),
            provider: "search_proxy".to_string(),
        })
    }

    async fn list(&self, path: &str) -> Result<Vec<VfsEntry>, VfsError> {
        let native = self.path_resolver.resolve(path)?;
        let uri = format!("vfs://local{}", PathResolver::native_to_uri_path(&native));

        let mut entries = Vec::new();
        let mut read_dir = tokio::fs::read_dir(native.as_path())
            .await
            .map_err(|e| map_io_error(e, "list", &uri))?;

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| map_io_error(e, "list", &uri))?
        {
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = entry.file_type().await.ok();

            let entry_type = match file_type {
                Some(ft) if ft.is_dir() => VfsEntryType::Directory,
                Some(ft) if ft.is_symlink() => VfsEntryType::Symlink,
                Some(ft) if ft.is_file() => VfsEntryType::File,
                _ => VfsEntryType::Other,
            };

            entries.push(VfsEntry {
                name,
                entry_type,
                size: None,
                modified: None,
            });
        }

        Ok(entries)
    }

    async fn stat(&self, path: &str) -> Result<VfsMetadata, VfsError> {
        let native = self.path_resolver.resolve(path)?;
        let uri = format!("vfs://local{}", PathResolver::native_to_uri_path(&native));
        let meta = tokio::fs::metadata(native.as_path())
            .await
            .map_err(|e| map_io_error(e, "stat", &uri))?;

        let entry_type = if meta.is_dir() {
            VfsEntryType::Directory
        } else if meta.is_file() {
            VfsEntryType::File
        } else {
            VfsEntryType::Other
        };

        Ok(VfsMetadata {
            size: Some(meta.len()),
            modified: meta.modified().ok(),
            entry_type,
            extra: std::collections::HashMap::new(),
        })
    }

    async fn exists(&self, path: &str) -> Result<bool, VfsError> {
        let native = self.path_resolver.resolve(path)?;
        Ok(tokio::fs::try_exists(native.as_path())
            .await
            .unwrap_or(false))
    }
}

/// A local file handle implementing VfsFile.
struct LocalFile {
    file: tokio::fs::File,
}

#[async_trait]
impl VfsFile for LocalFile {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, VfsError> {
        use tokio::io::AsyncReadExt;
        self.file.read(buf).await.map_err(|e| VfsError::Io {
            uri: String::new(),
            operation: "read".to_string(),
            source: e,
        })
    }

    async fn write(&mut self, data: &[u8]) -> Result<usize, VfsError> {
        use tokio::io::AsyncWriteExt;
        self.file.write(data).await.map_err(|e| VfsError::Io {
            uri: String::new(),
            operation: "write".to_string(),
            source: e,
        })
    }

    async fn flush(&mut self) -> Result<(), VfsError> {
        use tokio::io::AsyncWriteExt;
        self.file.flush().await.map_err(|e| VfsError::Io {
            uri: String::new(),
            operation: "flush".to_string(),
            source: e,
        })
    }

    async fn sync_all(&mut self) -> Result<(), VfsError> {
        self.file.sync_all().await.map_err(|e| VfsError::Io {
            uri: String::new(),
            operation: "sync_all".to_string(),
            source: e,
        })
    }

    async fn close(self: Box<Self>) -> Result<(), VfsError> {
        // Dropping the file handle closes it
        drop(self.file);
        Ok(())
    }
}

// Ensure Send + Sync
fn _assert_send_sync() {
    fn _assert<T: Send + Sync>() {}
    _assert::<LocalFsProvider>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn scheme_returns_local() {
        let provider = LocalFsProvider::with_defaults().unwrap();
        assert_eq!(provider.scheme(), "local");
    }

    #[tokio::test]
    async fn capabilities_returns_all() {
        let provider = LocalFsProvider::with_defaults().unwrap();
        let caps = provider.capabilities();
        assert!(caps.read);
        assert!(caps.write);
        assert!(caps.watch);
        assert!(caps.search);
        assert!(caps.rename);
        assert!(caps.delete);
        assert!(caps.list);
        assert!(caps.create_directory);
    }

    #[tokio::test]
    async fn read_write_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let path_str = file_path.to_string_lossy().to_string();

        let provider = LocalFsProvider::with_defaults().unwrap();

        // Write
        provider.write(&path_str, b"hello world").await.unwrap();

        // Read
        let content = provider.read(&path_str).await.unwrap();
        assert_eq!(content, b"hello world");
    }

    #[tokio::test]
    async fn create_and_delete_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("new_file.txt");
        let path_str = file_path.to_string_lossy().to_string();

        let provider = LocalFsProvider::with_defaults().unwrap();

        provider
            .create(&path_str, CreateOptions::default())
            .await
            .unwrap();
        assert!(provider.exists(&path_str).await.unwrap());

        provider
            .delete(&path_str, DeleteOptions::default())
            .await
            .unwrap();
        assert!(!provider.exists(&path_str).await.unwrap());
    }

    #[tokio::test]
    async fn create_and_list_directory() {
        let dir = tempfile::tempdir().unwrap();
        let sub_dir = dir.path().join("subdir");
        let dir_str = sub_dir.to_string_lossy().to_string();

        let provider = LocalFsProvider::with_defaults().unwrap();

        provider
            .create(
                &dir_str,
                CreateOptions {
                    is_directory: true,
                    create_parents: true,
                },
            )
            .await
            .unwrap();

        // Create a file inside
        let file_path = sub_dir.join("inner.txt");
        tokio::fs::write(&file_path, b"content").await.unwrap();

        let entries = provider.list(&dir_str).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "inner.txt");
        assert_eq!(entries[0].entry_type, VfsEntryType::File);
    }

    #[tokio::test]
    async fn stat_returns_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("stat_test.txt");
        tokio::fs::write(&file_path, b"12345").await.unwrap();
        let path_str = file_path.to_string_lossy().to_string();

        let provider = LocalFsProvider::with_defaults().unwrap();
        let meta = provider.stat(&path_str).await.unwrap();

        assert_eq!(meta.size, Some(5));
        assert_eq!(meta.entry_type, VfsEntryType::File);
        assert!(meta.modified.is_some());
    }

    #[tokio::test]
    async fn rename_moves_file() {
        let dir = tempfile::tempdir().unwrap();
        let old_path = dir.path().join("old.txt");
        let new_path = dir.path().join("new.txt");
        tokio::fs::write(&old_path, b"content").await.unwrap();

        let old_str = old_path.to_string_lossy().to_string();
        let new_str = new_path.to_string_lossy().to_string();

        let provider = LocalFsProvider::with_defaults().unwrap();
        provider.rename(&old_str, &new_str).await.unwrap();

        assert!(!provider.exists(&old_str).await.unwrap());
        assert!(provider.exists(&new_str).await.unwrap());
    }
}
