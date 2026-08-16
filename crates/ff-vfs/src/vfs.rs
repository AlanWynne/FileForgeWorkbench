//! Top-level VFS facade providing async file and directory operations.
//!
//! The [`Vfs`] struct wraps a [`ProviderRegistry`] and dispatches operations
//! to the appropriate provider based on the URI scheme. It implements the
//! routing pattern described in the design: acquire read lock, clone Arc,
//! release lock, then call async method.
//!
//! Addresses: Requirement 1 AC 2, AC 3; Requirement 5; Requirement 6

use std::pin::Pin;
use std::sync::Arc;

use futures_core::Stream;
use tokio::io::AsyncRead;
use tokio_util::sync::CancellationToken;

use crate::error::VfsError;
use crate::provider::{VfsFile, VfsProvider};
use crate::registry::ProviderRegistry;
use crate::search::{fallback_search, SearchOptions, SearchQuery, VfsSearchResult};
use crate::types::{
    CreateOptions, DeleteOptions, OpenOptions, VfsEntry, VfsMetadata, WatchOptions,
};
use crate::uri::ResourceUri;
use crate::watch::WatchHandle;

/// Top-level facade for all VFS operations.
///
/// Routes operations to the correct provider based on the URI scheme.
/// Thread-safe and cheaply cloneable (inner state is `Arc`-wrapped).
///
/// Addresses: Requirement 1 AC 2, AC 3
#[derive(Clone)]
pub struct Vfs {
    registry: Arc<ProviderRegistry>,
}

impl Vfs {
    /// Creates a new `Vfs` with an empty [`ProviderRegistry`].
    pub fn new() -> Self {
        Self {
            registry: Arc::new(ProviderRegistry::new()),
        }
    }

    /// Creates a new `Vfs` wrapping the given [`ProviderRegistry`].
    pub fn with_registry(registry: ProviderRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
        }
    }

    /// Returns a reference to the underlying [`ProviderRegistry`].
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    /// Resolve the provider for the given URI's scheme.
    ///
    /// Returns `VfsError::ProviderUnavailable` if no provider is registered
    /// for the URI's scheme.
    fn resolve_provider(&self, uri: &ResourceUri) -> Result<Arc<dyn VfsProvider>, VfsError> {
        self.registry
            .get(uri.scheme())
            .ok_or_else(|| VfsError::ProviderUnavailable {
                scheme: uri.scheme().to_string(),
            })
    }

    /// Open a resource for reading and/or writing.
    ///
    /// Addresses: Requirement 5 AC 1
    pub async fn open(
        &self,
        uri: &ResourceUri,
        options: OpenOptions,
    ) -> Result<Box<dyn VfsFile>, VfsError> {
        let provider = self.resolve_provider(uri)?;
        provider.open(uri.path(), options).await
    }

    /// Read entire resource content into memory.
    ///
    /// Addresses: Requirement 5 AC 2
    pub async fn read(&self, uri: &ResourceUri) -> Result<Vec<u8>, VfsError> {
        let provider = self.resolve_provider(uri)?;
        provider.read(uri.path()).await
    }

    /// Read resource content as an async byte stream.
    ///
    /// Addresses: Requirement 5 AC 2
    pub async fn read_stream(
        &self,
        uri: &ResourceUri,
    ) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
        let provider = self.resolve_provider(uri)?;
        provider.read_stream(uri.path()).await
    }

    /// Write data to a resource.
    ///
    /// Addresses: Requirement 5 AC 2
    pub async fn write(&self, uri: &ResourceUri, data: &[u8]) -> Result<(), VfsError> {
        let provider = self.resolve_provider(uri)?;
        provider.write(uri.path(), data).await
    }

    /// Delete a resource or container.
    ///
    /// Addresses: Requirement 5 AC 4
    pub async fn delete(&self, uri: &ResourceUri, options: DeleteOptions) -> Result<(), VfsError> {
        let provider = self.resolve_provider(uri)?;
        provider.delete(uri.path(), options).await
    }

    /// Rename/move a resource within the same provider.
    ///
    /// Cross-provider rename is not supported and returns
    /// `VfsError::UnsupportedOperation`.
    ///
    /// Addresses: Requirement 5 AC 6
    pub async fn rename(
        &self,
        old_uri: &ResourceUri,
        new_uri: &ResourceUri,
    ) -> Result<(), VfsError> {
        if old_uri.scheme() != new_uri.scheme() {
            return Err(VfsError::UnsupportedOperation {
                operation: "rename".to_string(),
                provider: format!(
                    "cross-provider: {} -> {}",
                    old_uri.scheme(),
                    new_uri.scheme()
                ),
            });
        }
        let provider = self.resolve_provider(old_uri)?;
        provider.rename(old_uri.path(), new_uri.path()).await
    }

    /// Copy a resource from source to destination.
    ///
    /// Supports cross-provider copy by reading from source and writing to
    /// destination. Both providers must be registered.
    ///
    /// Addresses: Requirement 5 AC 7
    pub async fn copy(&self, src: &ResourceUri, dst: &ResourceUri) -> Result<(), VfsError> {
        let src_provider = self.resolve_provider(src)?;
        let dst_provider = self.resolve_provider(dst)?;

        // Read all content from source
        let data = src_provider.read(src.path()).await?;
        // Write to destination
        dst_provider.write(dst.path(), &data).await?;
        Ok(())
    }

    /// List directory/container contents.
    ///
    /// Addresses: Requirement 6 AC 1
    pub async fn list(&self, uri: &ResourceUri) -> Result<Vec<VfsEntry>, VfsError> {
        let provider = self.resolve_provider(uri)?;
        provider.list(uri.path()).await
    }

    /// Create a directory/container.
    ///
    /// Addresses: Requirement 6 AC 2
    pub async fn create_dir(
        &self,
        uri: &ResourceUri,
        options: CreateOptions,
    ) -> Result<(), VfsError> {
        let provider = self.resolve_provider(uri)?;
        let opts = CreateOptions {
            is_directory: true,
            create_parents: options.create_parents,
        };
        provider.create(uri.path(), opts).await
    }

    /// Get resource metadata.
    ///
    /// Addresses: Requirement 6 AC 4
    pub async fn stat(&self, uri: &ResourceUri) -> Result<VfsMetadata, VfsError> {
        let provider = self.resolve_provider(uri)?;
        provider.stat(uri.path()).await
    }

    /// Check if a resource exists.
    ///
    /// Addresses: Requirement 6 AC 5
    pub async fn exists(&self, uri: &ResourceUri) -> Result<bool, VfsError> {
        let provider = self.resolve_provider(uri)?;
        provider.exists(uri.path()).await
    }

    /// Watch a resource or directory for changes.
    ///
    /// Returns `VfsError::UnsupportedOperation` if the provider does not
    /// support file watching (i.e., `capabilities().watch` is `false`).
    ///
    /// Addresses: Requirement 7 AC 1, AC 3
    pub async fn watch(
        &self,
        uri: &ResourceUri,
        options: WatchOptions,
    ) -> Result<WatchHandle, VfsError> {
        let provider = self.resolve_provider(uri)?;
        if !provider.capabilities().watch {
            return Err(VfsError::UnsupportedOperation {
                operation: "watch".to_string(),
                provider: uri.scheme().to_string(),
            });
        }
        provider.watch(uri.path(), options).await
    }

    /// Search for content or filenames within a resource tree.
    ///
    /// Delegates to the provider's native search if the provider advertises
    /// search capability. Otherwise falls back to the generic
    /// [`fallback_search`] implementation that enumerates via `list` and
    /// matches content line-by-line via `read_stream`.
    ///
    /// Addresses: Requirement 8 AC 1–5
    pub async fn search(
        &self,
        root: &ResourceUri,
        query: &SearchQuery,
        options: &SearchOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = VfsSearchResult> + Send>>, VfsError> {
        let provider = self.resolve_provider(root)?;
        if provider.capabilities().search {
            // Provider has native search — delegate
            provider.search(root.path(), query, options).await
        } else {
            // Use fallback search
            let cancel_token = CancellationToken::new();
            Ok(fallback_search(
                provider,
                root.path(),
                root.scheme(),
                query,
                options,
                cancel_token,
            )
            .await)
        }
    }

    /// Search for content or filenames with a caller-provided cancellation token.
    ///
    /// Like [`search`](Self::search) but allows the caller to cancel the search
    /// by triggering the supplied `CancellationToken`.
    ///
    /// Addresses: Requirement 8 AC 6
    pub async fn search_with_cancel(
        &self,
        root: &ResourceUri,
        query: &SearchQuery,
        options: &SearchOptions,
        cancel_token: CancellationToken,
    ) -> Result<Pin<Box<dyn Stream<Item = VfsSearchResult> + Send>>, VfsError> {
        let provider = self.resolve_provider(root)?;
        if provider.capabilities().search {
            provider.search(root.path(), query, options).await
        } else {
            Ok(fallback_search(
                provider,
                root.path(),
                root.scheme(),
                query,
                options,
                cancel_token,
            )
            .await)
        }
    }
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{VfsCapabilities, VfsEntryType};

    use std::collections::HashMap;
    use std::io::Cursor;
    use std::sync::Mutex;

    use async_trait::async_trait;

    /// In-memory mock provider that stores data in a `HashMap<String, Vec<u8>>`.
    struct InMemoryProvider {
        scheme_name: String,
        store: Arc<Mutex<HashMap<String, Vec<u8>>>>,
        /// Tracks directories that have been explicitly created.
        dirs: Arc<Mutex<Vec<String>>>,
    }

    impl InMemoryProvider {
        fn new(scheme: &str) -> Self {
            Self {
                scheme_name: scheme.to_string(),
                store: Arc::new(Mutex::new(HashMap::new())),
                dirs: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl VfsProvider for InMemoryProvider {
        fn scheme(&self) -> &str {
            &self.scheme_name
        }

        fn capabilities(&self) -> VfsCapabilities {
            VfsCapabilities {
                read: true,
                write: true,
                watch: false,
                search: false,
                random_access: false,
                append: true,
                rename: true,
                delete: true,
                list: true,
                create_directory: true,
            }
        }

        async fn open(
            &self,
            _path: &str,
            _options: OpenOptions,
        ) -> Result<Box<dyn VfsFile>, VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "open".to_string(),
                provider: self.scheme_name.clone(),
            })
        }

        async fn read(&self, path: &str) -> Result<Vec<u8>, VfsError> {
            let store = self.store.lock().expect("store lock poisoned");
            store.get(path).cloned().ok_or_else(|| VfsError::NotFound {
                uri: format!("vfs://{}{}", self.scheme_name, path),
                operation: "read".to_string(),
            })
        }

        async fn read_stream(
            &self,
            path: &str,
        ) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
            let store = self.store.lock().expect("store lock poisoned");
            let data = store.get(path).cloned().ok_or_else(|| VfsError::NotFound {
                uri: format!("vfs://{}{}", self.scheme_name, path),
                operation: "read_stream".to_string(),
            })?;
            Ok(Box::pin(Cursor::new(data)))
        }

        async fn write(&self, path: &str, data: &[u8]) -> Result<(), VfsError> {
            let mut store = self.store.lock().expect("store lock poisoned");
            store.insert(path.to_string(), data.to_vec());
            Ok(())
        }

        async fn create(&self, path: &str, options: CreateOptions) -> Result<(), VfsError> {
            if options.is_directory {
                let mut dirs = self.dirs.lock().expect("dirs lock poisoned");
                if options.create_parents {
                    // Create all parent directories
                    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
                    let mut current = String::new();
                    for part in &parts {
                        current.push('/');
                        current.push_str(part);
                        if !dirs.contains(&current) {
                            dirs.push(current.clone());
                        }
                    }
                } else {
                    dirs.push(path.to_string());
                }
            }
            Ok(())
        }

        async fn delete(&self, path: &str, _options: DeleteOptions) -> Result<(), VfsError> {
            let mut store = self.store.lock().expect("store lock poisoned");
            store
                .remove(path)
                .map(|_| ())
                .ok_or_else(|| VfsError::NotFound {
                    uri: format!("vfs://{}{}", self.scheme_name, path),
                    operation: "delete".to_string(),
                })
        }

        async fn rename(&self, old_path: &str, new_path: &str) -> Result<(), VfsError> {
            let mut store = self.store.lock().expect("store lock poisoned");
            let data = store.remove(old_path).ok_or_else(|| VfsError::NotFound {
                uri: format!("vfs://{}{}", self.scheme_name, old_path),
                operation: "rename".to_string(),
            })?;
            store.insert(new_path.to_string(), data);
            Ok(())
        }

        async fn list(&self, path: &str) -> Result<Vec<VfsEntry>, VfsError> {
            let dirs = self.dirs.lock().expect("dirs lock poisoned");
            let store = self.store.lock().expect("store lock poisoned");

            // Check if path is a known directory or root
            let is_dir = path == "/" || dirs.contains(&path.to_string());

            if !is_dir {
                // Check if it's a file — then return NotADirectory
                if store.contains_key(path) {
                    return Err(VfsError::NotADirectory {
                        uri: format!("vfs://{}{}", self.scheme_name, path),
                        operation: "list".to_string(),
                    });
                }
                return Err(VfsError::NotFound {
                    uri: format!("vfs://{}{}", self.scheme_name, path),
                    operation: "list".to_string(),
                });
            }

            let prefix = if path.ends_with('/') {
                path.to_string()
            } else {
                format!("{}/", path)
            };

            let mut entries = Vec::new();
            // Find files directly under this path
            for (key, value) in store.iter() {
                if let Some(rest) = key.strip_prefix(&prefix) {
                    if !rest.contains('/') {
                        entries.push(VfsEntry {
                            name: rest.to_string(),
                            entry_type: VfsEntryType::File,
                            size: Some(value.len() as u64),
                            modified: None,
                        });
                    }
                }
            }
            // Find subdirectories directly under this path
            for dir in dirs.iter() {
                if let Some(rest) = dir.strip_prefix(&prefix) {
                    if !rest.contains('/') && !rest.is_empty() {
                        entries.push(VfsEntry {
                            name: rest.to_string(),
                            entry_type: VfsEntryType::Directory,
                            size: None,
                            modified: None,
                        });
                    }
                }
            }
            entries.sort_by(|a, b| a.name.cmp(&b.name));
            Ok(entries)
        }

        async fn stat(&self, path: &str) -> Result<VfsMetadata, VfsError> {
            let store = self.store.lock().expect("store lock poisoned");
            if let Some(data) = store.get(path) {
                return Ok(VfsMetadata {
                    size: Some(data.len() as u64),
                    modified: None,
                    entry_type: VfsEntryType::File,
                    extra: HashMap::new(),
                });
            }
            drop(store);

            let dirs = self.dirs.lock().expect("dirs lock poisoned");
            if dirs.contains(&path.to_string()) {
                return Ok(VfsMetadata {
                    size: None,
                    modified: None,
                    entry_type: VfsEntryType::Directory,
                    extra: HashMap::new(),
                });
            }

            Err(VfsError::NotFound {
                uri: format!("vfs://{}{}", self.scheme_name, path),
                operation: "stat".to_string(),
            })
        }

        async fn exists(&self, path: &str) -> Result<bool, VfsError> {
            let store = self.store.lock().expect("store lock poisoned");
            if store.contains_key(path) {
                return Ok(true);
            }
            drop(store);

            let dirs = self.dirs.lock().expect("dirs lock poisoned");
            Ok(dirs.contains(&path.to_string()))
        }
    }

    /// Helper: create a Vfs with a single in-memory provider registered.
    fn vfs_with_provider(scheme: &str) -> (Vfs, Arc<InMemoryProvider>) {
        let provider = Arc::new(InMemoryProvider::new(scheme));
        let registry = ProviderRegistry::new();
        registry
            .register(provider.clone() as Arc<dyn VfsProvider>)
            .unwrap();
        let vfs = Vfs::with_registry(registry);
        (vfs, provider)
    }

    // ===== Task 6.1 Tests =====

    // Validates: Requirement 5 AC 1
    #[test]
    fn vfs_new_creates_instance_with_empty_registry() {
        let vfs = Vfs::new();
        assert!(vfs.registry().list_schemes().is_empty());
    }

    // Validates: Requirement 5 AC 1
    #[test]
    fn vfs_with_registry_exposes_registry_accessor() {
        let registry = ProviderRegistry::new();
        let provider: Arc<dyn VfsProvider> = Arc::new(InMemoryProvider::new("test"));
        registry.register(provider).unwrap();
        let vfs = Vfs::with_registry(registry);
        assert_eq!(vfs.registry().list_schemes(), vec!["test"]);
    }

    // ===== Task 6.2 Tests =====

    // Validates: Requirement 5 AC 3
    #[tokio::test]
    async fn routing_returns_provider_unavailable_for_missing_scheme() {
        let vfs = Vfs::new();
        let uri = ResourceUri::new("nonexistent", "/file.txt");
        let result = vfs.read(&uri).await;
        match result {
            Err(VfsError::ProviderUnavailable { scheme }) => {
                assert_eq!(scheme, "nonexistent");
            }
            other => panic!("expected ProviderUnavailable, got: {other:?}"),
        }
    }

    // ===== Task 6.3 Tests =====

    // Validates: Requirement 5 AC 2
    #[tokio::test]
    async fn write_and_read_round_trip() {
        let (vfs, _provider) = vfs_with_provider("mem");
        let uri = ResourceUri::new("mem", "/hello.txt");

        vfs.write(&uri, b"hello world").await.unwrap();
        let data = vfs.read(&uri).await.unwrap();
        assert_eq!(data, b"hello world");
    }

    // Validates: Requirement 5 AC 2
    #[tokio::test]
    async fn read_stream_returns_content() {
        use tokio::io::AsyncReadExt;

        let (vfs, _provider) = vfs_with_provider("mem");
        let uri = ResourceUri::new("mem", "/stream.txt");
        vfs.write(&uri, b"stream data").await.unwrap();

        let mut reader = vfs.read_stream(&uri).await.unwrap();
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, b"stream data");
    }

    // Validates: Requirement 5 AC 4
    #[tokio::test]
    async fn delete_removes_resource() {
        let (vfs, _provider) = vfs_with_provider("mem");
        let uri = ResourceUri::new("mem", "/to_delete.txt");
        vfs.write(&uri, b"data").await.unwrap();

        vfs.delete(&uri, DeleteOptions::default()).await.unwrap();
        let result = vfs.read(&uri).await;
        assert!(matches!(result, Err(VfsError::NotFound { .. })));
    }

    // Validates: Requirement 5 AC 4
    #[tokio::test]
    async fn delete_nonexistent_returns_not_found() {
        let (vfs, _provider) = vfs_with_provider("mem");
        let uri = ResourceUri::new("mem", "/nope.txt");
        let result = vfs.delete(&uri, DeleteOptions::default()).await;
        assert!(matches!(result, Err(VfsError::NotFound { .. })));
    }

    // ===== Task 6.4 Tests =====

    // Validates: Requirement 5 AC 5
    #[tokio::test]
    async fn rename_same_provider_moves_resource() {
        let (vfs, _provider) = vfs_with_provider("mem");
        let old_uri = ResourceUri::new("mem", "/old.txt");
        let new_uri = ResourceUri::new("mem", "/new.txt");
        vfs.write(&old_uri, b"content").await.unwrap();

        vfs.rename(&old_uri, &new_uri).await.unwrap();

        let result = vfs.read(&old_uri).await;
        assert!(matches!(result, Err(VfsError::NotFound { .. })));
        let data = vfs.read(&new_uri).await.unwrap();
        assert_eq!(data, b"content");
    }

    // Validates: Requirement 5 AC 6
    #[tokio::test]
    async fn rename_cross_provider_returns_unsupported_operation() {
        let registry = ProviderRegistry::new();
        let p1: Arc<dyn VfsProvider> = Arc::new(InMemoryProvider::new("alpha"));
        let p2: Arc<dyn VfsProvider> = Arc::new(InMemoryProvider::new("beta"));
        registry.register(p1).unwrap();
        registry.register(p2).unwrap();
        let vfs = Vfs::with_registry(registry);

        let old_uri = ResourceUri::new("alpha", "/file.txt");
        let new_uri = ResourceUri::new("beta", "/file.txt");
        let result = vfs.rename(&old_uri, &new_uri).await;

        match result {
            Err(VfsError::UnsupportedOperation {
                operation,
                provider,
            }) => {
                assert_eq!(operation, "rename");
                assert!(provider.contains("alpha"));
                assert!(provider.contains("beta"));
            }
            other => panic!("expected UnsupportedOperation, got: {other:?}"),
        }
    }

    // ===== Task 6.5 Tests =====

    // Validates: Requirement 5 AC 7
    #[tokio::test]
    async fn copy_same_provider_duplicates_content() {
        let (vfs, _provider) = vfs_with_provider("mem");
        let src = ResourceUri::new("mem", "/source.txt");
        let dst = ResourceUri::new("mem", "/dest.txt");
        vfs.write(&src, b"copy me").await.unwrap();

        vfs.copy(&src, &dst).await.unwrap();

        let src_data = vfs.read(&src).await.unwrap();
        let dst_data = vfs.read(&dst).await.unwrap();
        assert_eq!(src_data, b"copy me");
        assert_eq!(dst_data, b"copy me");
    }

    // Validates: Requirement 5 AC 7
    #[tokio::test]
    async fn copy_cross_provider_reads_from_source_writes_to_dest() {
        let registry = ProviderRegistry::new();
        let p1 = Arc::new(InMemoryProvider::new("src_prov"));
        let p2 = Arc::new(InMemoryProvider::new("dst_prov"));

        // Pre-populate source
        {
            let mut store = p1.store.lock().unwrap();
            store.insert("/data.bin".to_string(), b"binary data".to_vec());
        }

        registry
            .register(p1.clone() as Arc<dyn VfsProvider>)
            .unwrap();
        registry
            .register(p2.clone() as Arc<dyn VfsProvider>)
            .unwrap();
        let vfs = Vfs::with_registry(registry);

        let src = ResourceUri::new("src_prov", "/data.bin");
        let dst = ResourceUri::new("dst_prov", "/data.bin");
        vfs.copy(&src, &dst).await.unwrap();

        // Verify destination has the data
        let dst_data = vfs.read(&dst).await.unwrap();
        assert_eq!(dst_data, b"binary data");
    }

    // ===== Task 7.1 & 7.2 Tests =====

    // Validates: Requirement 6 AC 1
    #[tokio::test]
    async fn list_returns_entries_for_directory() {
        let (vfs, provider) = vfs_with_provider("mem");

        // Create directory and files
        {
            let mut dirs = provider.dirs.lock().unwrap();
            dirs.push("/docs".to_string());
        }
        let uri_a = ResourceUri::new("mem", "/docs/a.txt");
        let uri_b = ResourceUri::new("mem", "/docs/b.txt");
        vfs.write(&uri_a, b"aaa").await.unwrap();
        vfs.write(&uri_b, b"bbb").await.unwrap();

        let dir_uri = ResourceUri::new("mem", "/docs");
        let entries = vfs.list(&dir_uri).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "a.txt");
        assert_eq!(entries[0].entry_type, VfsEntryType::File);
        assert_eq!(entries[0].size, Some(3));
        assert_eq!(entries[1].name, "b.txt");
    }

    // Validates: Requirement 6 AC 2
    #[tokio::test]
    async fn create_dir_with_parents_creates_hierarchy() {
        let (vfs, provider) = vfs_with_provider("mem");
        let uri = ResourceUri::new("mem", "/a/b/c");
        let opts = CreateOptions {
            create_parents: true,
            is_directory: false, // create_dir forces is_directory=true
        };
        vfs.create_dir(&uri, opts).await.unwrap();

        let dirs = provider.dirs.lock().unwrap();
        assert!(dirs.contains(&"/a".to_string()));
        assert!(dirs.contains(&"/a/b".to_string()));
        assert!(dirs.contains(&"/a/b/c".to_string()));
    }

    // Validates: Requirement 6 AC 4
    #[tokio::test]
    async fn stat_returns_metadata_for_file() {
        let (vfs, _provider) = vfs_with_provider("mem");
        let uri = ResourceUri::new("mem", "/info.txt");
        vfs.write(&uri, b"metadata test").await.unwrap();

        let meta = vfs.stat(&uri).await.unwrap();
        assert_eq!(meta.entry_type, VfsEntryType::File);
        assert_eq!(meta.size, Some(13));
    }

    // Validates: Requirement 6 AC 4
    #[tokio::test]
    async fn stat_returns_metadata_for_directory() {
        let (vfs, provider) = vfs_with_provider("mem");
        {
            let mut dirs = provider.dirs.lock().unwrap();
            dirs.push("/mydir".to_string());
        }
        let uri = ResourceUri::new("mem", "/mydir");
        let meta = vfs.stat(&uri).await.unwrap();
        assert_eq!(meta.entry_type, VfsEntryType::Directory);
        assert_eq!(meta.size, None);
    }

    // Validates: Requirement 6 AC 5
    #[tokio::test]
    async fn exists_returns_true_for_existing_file() {
        let (vfs, _provider) = vfs_with_provider("mem");
        let uri = ResourceUri::new("mem", "/exists.txt");
        vfs.write(&uri, b"x").await.unwrap();
        assert!(vfs.exists(&uri).await.unwrap());
    }

    // Validates: Requirement 6 AC 5
    #[tokio::test]
    async fn exists_returns_false_for_missing_resource() {
        let (vfs, _provider) = vfs_with_provider("mem");
        let uri = ResourceUri::new("mem", "/nope.txt");
        assert!(!vfs.exists(&uri).await.unwrap());
    }

    // Validates: Requirement 6 AC 7
    #[tokio::test]
    async fn list_on_non_directory_returns_not_a_directory() {
        let (vfs, _provider) = vfs_with_provider("mem");
        let uri = ResourceUri::new("mem", "/file.txt");
        vfs.write(&uri, b"data").await.unwrap();

        let result = vfs.list(&uri).await;
        assert!(matches!(result, Err(VfsError::NotADirectory { .. })));
    }

    // Validates: Requirement 6 AC 8
    #[tokio::test]
    async fn list_on_missing_path_returns_not_found() {
        let (vfs, _provider) = vfs_with_provider("mem");
        let uri = ResourceUri::new("mem", "/ghost");
        let result = vfs.list(&uri).await;
        assert!(matches!(result, Err(VfsError::NotFound { .. })));
    }

    // ===== Task 8.4 & 8.5 Tests =====

    /// A mock provider that does NOT support watch (capabilities.watch = false).
    struct NoWatchProvider;

    #[async_trait]
    impl VfsProvider for NoWatchProvider {
        fn scheme(&self) -> &str {
            "nowatch"
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

        async fn open(
            &self,
            _path: &str,
            _options: OpenOptions,
        ) -> Result<Box<dyn crate::provider::VfsFile>, VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "open".to_string(),
                provider: "nowatch".to_string(),
            })
        }

        async fn read(&self, _path: &str) -> Result<Vec<u8>, VfsError> {
            Ok(Vec::new())
        }

        async fn read_stream(
            &self,
            _path: &str,
        ) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
            Ok(Box::pin(std::io::Cursor::new(Vec::new())))
        }

        async fn write(&self, _path: &str, _data: &[u8]) -> Result<(), VfsError> {
            Ok(())
        }

        async fn create(&self, _path: &str, _options: CreateOptions) -> Result<(), VfsError> {
            Ok(())
        }

        async fn delete(&self, _path: &str, _options: DeleteOptions) -> Result<(), VfsError> {
            Ok(())
        }

        async fn rename(&self, _old_path: &str, _new_path: &str) -> Result<(), VfsError> {
            Ok(())
        }

        async fn list(&self, _path: &str) -> Result<Vec<VfsEntry>, VfsError> {
            Ok(Vec::new())
        }

        async fn stat(&self, _path: &str) -> Result<VfsMetadata, VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "stat".to_string(),
                provider: "nowatch".to_string(),
            })
        }

        async fn exists(&self, _path: &str) -> Result<bool, VfsError> {
            Ok(false)
        }
    }

    // Validates: Requirement 7 AC 1
    #[tokio::test]
    async fn watch_event_delivery_via_channel() {
        use crate::watch::{WatchEvent, WatchHandle};
        use tokio::sync::mpsc;
        use tokio_util::sync::CancellationToken;

        let (tx, rx) = mpsc::channel(16);
        let token = CancellationToken::new();
        let mut handle = WatchHandle::new(rx, token);

        let uri = ResourceUri::new("mem", "/watched.txt");
        tx.send(WatchEvent::Created(uri.clone())).await.unwrap();
        tx.send(WatchEvent::Modified(uri.clone())).await.unwrap();

        let event1 = handle.recv().await.unwrap();
        assert_eq!(event1, WatchEvent::Created(uri.clone()));

        let event2 = handle.recv().await.unwrap();
        assert_eq!(event2, WatchEvent::Modified(uri));
    }

    // Validates: Requirement 7 AC 4
    #[tokio::test]
    async fn watch_cancel_stops_delivery() {
        use crate::watch::WatchHandle;
        use tokio::sync::mpsc;
        use tokio_util::sync::CancellationToken;

        let (tx, rx) = mpsc::channel(16);
        let token = CancellationToken::new();
        let mut handle = WatchHandle::new(rx, token.clone());

        // Cancel and drop the sender
        handle.cancel();
        drop(tx);

        // After cancel + sender drop, recv should return None
        let event = handle.recv().await;
        assert_eq!(event, None);
    }

    // Validates: Requirement 7 AC 3
    #[tokio::test]
    async fn watch_unsupported_provider_returns_error() {
        let registry = ProviderRegistry::new();
        let provider: Arc<dyn VfsProvider> = Arc::new(NoWatchProvider);
        registry.register(provider).unwrap();
        let vfs = Vfs::with_registry(registry);

        let uri = ResourceUri::new("nowatch", "/some/path");
        let result = vfs.watch(&uri, WatchOptions::default()).await;

        match result {
            Err(VfsError::UnsupportedOperation {
                operation,
                provider,
            }) => {
                assert_eq!(operation, "watch");
                assert_eq!(provider, "nowatch");
            }
            other => panic!("expected UnsupportedOperation, got: {other:?}"),
        }
    }

    // Validates: Requirement 7 AC 5
    #[tokio::test]
    async fn watch_options_custom_debounce_accepted() {
        use crate::watch::{WatchEvent, WatchHandle};
        use std::time::Duration;
        use tokio::sync::mpsc;
        use tokio_util::sync::CancellationToken;

        // Verify WatchOptions can be constructed with custom debounce
        let options = WatchOptions {
            debounce: Duration::from_millis(500),
            recursive: true,
        };
        assert_eq!(options.debounce, Duration::from_millis(500));
        assert!(options.recursive);

        // And the handle still delivers events regardless of debounce config
        let (tx, rx) = mpsc::channel(16);
        let token = CancellationToken::new();
        let mut handle = WatchHandle::new(rx, token);

        let uri = ResourceUri::new("mem", "/debounced.txt");
        tx.send(WatchEvent::Deleted(uri.clone())).await.unwrap();

        let event = handle.recv().await.unwrap();
        assert_eq!(event, WatchEvent::Deleted(uri));
    }

    // ===== Task 9.3 & 9.4 Tests =====

    use crate::search::{SearchOptions, SearchQuery, VfsSearchResult};
    use tokio_stream::StreamExt;

    /// A mock provider that has native search capability and returns
    /// canned results. Used to verify delegation.
    struct NativeSearchProvider {
        results: Vec<VfsSearchResult>,
    }

    impl NativeSearchProvider {
        fn new(results: Vec<VfsSearchResult>) -> Self {
            Self { results }
        }
    }

    #[async_trait]
    impl VfsProvider for NativeSearchProvider {
        fn scheme(&self) -> &str {
            "native"
        }

        fn capabilities(&self) -> VfsCapabilities {
            VfsCapabilities {
                read: true,
                write: true,
                watch: false,
                search: true, // native search supported
                random_access: false,
                append: false,
                rename: true,
                delete: true,
                list: true,
                create_directory: true,
            }
        }

        async fn open(
            &self,
            _path: &str,
            _options: OpenOptions,
        ) -> Result<Box<dyn crate::provider::VfsFile>, VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "open".to_string(),
                provider: "native".to_string(),
            })
        }

        async fn read(&self, _path: &str) -> Result<Vec<u8>, VfsError> {
            Ok(Vec::new())
        }

        async fn read_stream(
            &self,
            _path: &str,
        ) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
            Ok(Box::pin(std::io::Cursor::new(Vec::new())))
        }

        async fn write(&self, _path: &str, _data: &[u8]) -> Result<(), VfsError> {
            Ok(())
        }

        async fn create(&self, _path: &str, _options: CreateOptions) -> Result<(), VfsError> {
            Ok(())
        }

        async fn delete(&self, _path: &str, _options: DeleteOptions) -> Result<(), VfsError> {
            Ok(())
        }

        async fn rename(&self, _old_path: &str, _new_path: &str) -> Result<(), VfsError> {
            Ok(())
        }

        async fn list(&self, _path: &str) -> Result<Vec<VfsEntry>, VfsError> {
            Ok(Vec::new())
        }

        async fn stat(&self, _path: &str) -> Result<VfsMetadata, VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "stat".to_string(),
                provider: "native".to_string(),
            })
        }

        async fn exists(&self, _path: &str) -> Result<bool, VfsError> {
            Ok(false)
        }

        async fn search(
            &self,
            _path: &str,
            _query: &SearchQuery,
            _options: &SearchOptions,
        ) -> Result<Pin<Box<dyn Stream<Item = VfsSearchResult> + Send>>, VfsError> {
            let results = self.results.clone();
            let (tx, rx) = tokio::sync::mpsc::channel(64);
            tokio::spawn(async move {
                for r in results {
                    let _ = tx.send(r).await;
                }
            });
            Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
        }
    }

    // Validates: Requirement 8 AC 1
    #[tokio::test]
    async fn search_delegates_to_native_provider_when_capable() {
        let expected_results = vec![
            VfsSearchResult {
                uri: ResourceUri::new("native", "/file.txt"),
                line: Some(1),
                column: Some(0),
                preview: "hello world".to_string(),
            },
            VfsSearchResult {
                uri: ResourceUri::new("native", "/other.txt"),
                line: Some(5),
                column: Some(3),
                preview: "say hello".to_string(),
            },
        ];

        let provider = Arc::new(NativeSearchProvider::new(expected_results.clone()));
        let registry = ProviderRegistry::new();
        registry.register(provider as Arc<dyn VfsProvider>).unwrap();
        let vfs = Vfs::with_registry(registry);

        let root = ResourceUri::new("native", "/");
        let query = SearchQuery::Content("hello".to_string());
        let options = SearchOptions::default();

        let mut stream = vfs.search(&root, &query, &options).await.unwrap();
        let mut results = Vec::new();
        while let Some(r) = stream.next().await {
            results.push(r);
        }

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].preview, "hello world");
        assert_eq!(results[1].preview, "say hello");
    }

    // Validates: Requirement 8 AC 4, AC 5
    #[tokio::test]
    async fn search_fallback_enumerates_and_finds_content() {
        let (vfs, provider) = vfs_with_provider("mem");

        // Set up directory structure
        {
            let mut dirs = provider.dirs.lock().unwrap();
            dirs.push("/project".to_string());
        }
        vfs.write(
            &ResourceUri::new("mem", "/project/hello.txt"),
            b"line one\nhello world\nline three",
        )
        .await
        .unwrap();
        vfs.write(
            &ResourceUri::new("mem", "/project/other.txt"),
            b"no match here\nstill nothing",
        )
        .await
        .unwrap();

        let root = ResourceUri::new("mem", "/project");
        let query = SearchQuery::Content("hello".to_string());
        let options = SearchOptions::default();

        let mut stream = vfs.search(&root, &query, &options).await.unwrap();
        let mut results = Vec::new();
        while let Some(r) = stream.next().await {
            results.push(r);
        }

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line, Some(2));
        assert_eq!(results[0].preview, "hello world");
    }

    // Validates: Requirement 8 AC 2
    #[tokio::test]
    async fn search_fallback_case_insensitive_matches() {
        let (vfs, provider) = vfs_with_provider("mem");

        {
            let mut dirs = provider.dirs.lock().unwrap();
            dirs.push("/src".to_string());
        }
        vfs.write(
            &ResourceUri::new("mem", "/src/code.rs"),
            b"fn main() {\n    println!(\"HELLO\");\n}\n",
        )
        .await
        .unwrap();

        let root = ResourceUri::new("mem", "/src");
        let query = SearchQuery::Content("hello".to_string());
        let options = SearchOptions {
            case_sensitive: false,
            ..Default::default()
        };

        let mut stream = vfs.search(&root, &query, &options).await.unwrap();
        let mut results = Vec::new();
        while let Some(r) = stream.next().await {
            results.push(r);
        }

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line, Some(2));
    }

    // Validates: Requirement 8 AC 2
    #[tokio::test]
    async fn search_fallback_case_sensitive_respects_case() {
        let (vfs, provider) = vfs_with_provider("mem");

        {
            let mut dirs = provider.dirs.lock().unwrap();
            dirs.push("/src".to_string());
        }
        vfs.write(
            &ResourceUri::new("mem", "/src/code.rs"),
            b"fn main() {\n    println!(\"HELLO\");\n}\n",
        )
        .await
        .unwrap();

        let root = ResourceUri::new("mem", "/src");
        let query = SearchQuery::Content("hello".to_string());
        let options = SearchOptions {
            case_sensitive: true,
            ..Default::default()
        };

        let mut stream = vfs.search(&root, &query, &options).await.unwrap();
        let mut results = Vec::new();
        while let Some(r) = stream.next().await {
            results.push(r);
        }

        // "hello" (lowercase) should NOT match "HELLO" in case-sensitive mode
        assert_eq!(results.len(), 0);
    }

    // Validates: Requirement 8 AC 2
    #[tokio::test]
    async fn search_fallback_whole_word_matches_only_complete_words() {
        let (vfs, provider) = vfs_with_provider("mem");

        {
            let mut dirs = provider.dirs.lock().unwrap();
            dirs.push("/docs".to_string());
        }
        vfs.write(
            &ResourceUri::new("mem", "/docs/text.txt"),
            b"cat\ncatalog\nthe cat sat\nconcatenate",
        )
        .await
        .unwrap();

        let root = ResourceUri::new("mem", "/docs");
        let query = SearchQuery::Content("cat".to_string());
        let options = SearchOptions {
            whole_word: true,
            case_sensitive: true,
            ..Default::default()
        };

        let mut stream = vfs.search(&root, &query, &options).await.unwrap();
        let mut results = Vec::new();
        while let Some(r) = stream.next().await {
            results.push(r);
        }

        // "cat" alone and "the cat sat" match; "catalog" and "concatenate" do not
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|r| r.preview == "cat"));
        assert!(results.iter().any(|r| r.preview == "the cat sat"));
    }

    // Validates: Requirement 8 AC 3
    #[tokio::test]
    async fn search_fallback_max_results_limits_output() {
        let (vfs, provider) = vfs_with_provider("mem");

        {
            let mut dirs = provider.dirs.lock().unwrap();
            dirs.push("/many".to_string());
        }
        // Create a file with many matching lines
        let content: String = (0..20)
            .map(|i| format!("match line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        vfs.write(
            &ResourceUri::new("mem", "/many/data.txt"),
            content.as_bytes(),
        )
        .await
        .unwrap();

        let root = ResourceUri::new("mem", "/many");
        let query = SearchQuery::Content("match".to_string());
        let options = SearchOptions {
            max_results: 5,
            ..Default::default()
        };

        let mut stream = vfs.search(&root, &query, &options).await.unwrap();
        let mut results = Vec::new();
        while let Some(r) = stream.next().await {
            results.push(r);
        }

        assert_eq!(results.len(), 5);
    }

    // Validates: Requirement 8 AC 6
    #[tokio::test]
    async fn search_cancellation_stops_results() {
        use tokio_util::sync::CancellationToken;

        let (vfs, provider) = vfs_with_provider("mem");

        {
            let mut dirs = provider.dirs.lock().unwrap();
            dirs.push("/cancel".to_string());
        }
        // Create multiple files
        for i in 0..10 {
            let path = format!("/cancel/file{}.txt", i);
            let content = format!("searchable content in file {}", i);
            vfs.write(&ResourceUri::new("mem", &path), content.as_bytes())
                .await
                .unwrap();
        }

        let root = ResourceUri::new("mem", "/cancel");
        let query = SearchQuery::Content("searchable".to_string());
        let options = SearchOptions::default();
        let cancel_token = CancellationToken::new();

        // Cancel immediately before consuming results
        cancel_token.cancel();

        let mut stream = vfs
            .search_with_cancel(&root, &query, &options, cancel_token)
            .await
            .unwrap();

        let mut results = Vec::new();
        while let Some(r) = stream.next().await {
            results.push(r);
        }

        // With immediate cancellation, we should get fewer results than the 10 files
        assert!(
            results.len() < 10,
            "expected fewer than 10 results due to cancellation, got {}",
            results.len()
        );
    }

    // Validates: Requirement 8 AC 1
    #[tokio::test]
    async fn search_fallback_filename_search_matches() {
        let (vfs, provider) = vfs_with_provider("mem");

        {
            let mut dirs = provider.dirs.lock().unwrap();
            dirs.push("/root".to_string());
        }
        vfs.write(&ResourceUri::new("mem", "/root/readme.md"), b"# Hello")
            .await
            .unwrap();
        vfs.write(&ResourceUri::new("mem", "/root/config.json"), b"{}")
            .await
            .unwrap();
        vfs.write(&ResourceUri::new("mem", "/root/readme.txt"), b"text")
            .await
            .unwrap();

        let root = ResourceUri::new("mem", "/root");
        let query = SearchQuery::Filename("readme".to_string());
        let options = SearchOptions::default();

        let mut stream = vfs.search(&root, &query, &options).await.unwrap();
        let mut results = Vec::new();
        while let Some(r) = stream.next().await {
            results.push(r);
        }

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.line.is_none()));
        let previews: Vec<&str> = results.iter().map(|r| r.preview.as_str()).collect();
        assert!(previews.contains(&"readme.md"));
        assert!(previews.contains(&"readme.txt"));
    }
}
