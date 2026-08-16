//! Provider trait and file handle trait for the VFS abstraction layer.
//!
//! Defines `VfsProvider` — the core trait all storage backends implement — and
//! `VfsFile` — the trait for open file handles supporting async read/write.
//!
//! Both traits are object-safe and require `Send + Sync` for use in concurrent,
//! multi-threaded Tokio environments.

use std::pin::Pin;

use async_trait::async_trait;
use futures_core::Stream;
use tokio::io::AsyncRead;

use crate::error::VfsError;
use crate::search::{SearchOptions, SearchQuery, VfsSearchResult};
use crate::types::{
    CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsMetadata, WatchOptions,
};
use crate::watch::WatchHandle;

/// The core trait that all storage backend implementations must implement.
///
/// Object-safe for dynamic dispatch via `dyn VfsProvider`. All async methods are
/// compatible with the Tokio runtime. Providers register with the `ProviderRegistry`
/// using their `scheme()` identifier.
///
/// Addresses: Requirement 4, criteria 1–10
#[async_trait]
pub trait VfsProvider: Send + Sync {
    /// Returns the unique scheme identifier for this provider (e.g., "local", "catalog").
    ///
    /// Addresses: Requirement 4 AC 6
    fn scheme(&self) -> &str;

    /// Returns the capabilities this provider supports.
    ///
    /// Addresses: Requirement 4 AC 4
    fn capabilities(&self) -> VfsCapabilities;

    /// Open a resource for reading and/or writing.
    ///
    /// Addresses: Requirement 4 AC 2
    async fn open(&self, path: &str, options: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError>;

    /// Read entire resource content into memory.
    ///
    /// Addresses: Requirement 4 AC 2
    async fn read(&self, path: &str) -> Result<Vec<u8>, VfsError>;

    /// Read resource content as an async byte stream.
    ///
    /// Addresses: Requirement 4 AC 2
    async fn read_stream(&self, path: &str) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError>;

    /// Write data to a resource (create or overwrite based on provider semantics).
    ///
    /// Addresses: Requirement 4 AC 2
    async fn write(&self, path: &str, data: &[u8]) -> Result<(), VfsError>;

    /// Create a new resource or container.
    ///
    /// Addresses: Requirement 4 AC 2
    async fn create(&self, path: &str, options: CreateOptions) -> Result<(), VfsError>;

    /// Delete a resource or container.
    ///
    /// Addresses: Requirement 4 AC 2
    async fn delete(&self, path: &str, options: DeleteOptions) -> Result<(), VfsError>;

    /// Rename/move a resource within this provider's namespace.
    ///
    /// Addresses: Requirement 4 AC 2
    async fn rename(&self, old_path: &str, new_path: &str) -> Result<(), VfsError>;

    /// List directory/container contents.
    ///
    /// Addresses: Requirement 4 AC 2
    async fn list(&self, path: &str) -> Result<Vec<VfsEntry>, VfsError>;

    /// Get resource metadata.
    ///
    /// Addresses: Requirement 4 AC 2
    async fn stat(&self, path: &str) -> Result<VfsMetadata, VfsError>;

    /// Check if a resource exists.
    ///
    /// Addresses: Requirement 4 AC 2
    async fn exists(&self, path: &str) -> Result<bool, VfsError>;

    /// Watch a resource or directory for changes.
    ///
    /// Default returns `VfsError::UnsupportedOperation` for providers that don't
    /// support file watching.
    ///
    /// Addresses: Requirement 4 AC 9
    async fn watch(&self, _path: &str, _options: WatchOptions) -> Result<WatchHandle, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "watch".to_string(),
            provider: self.scheme().to_string(),
        })
    }

    /// Search within this provider's scope.
    ///
    /// Default returns `VfsError::UnsupportedOperation` for providers without
    /// native search capability.
    ///
    /// Addresses: Requirement 4 AC 10
    async fn search(
        &self,
        _path: &str,
        _query: &SearchQuery,
        _options: &SearchOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = VfsSearchResult> + Send>>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "search".to_string(),
            provider: self.scheme().to_string(),
        })
    }
}

/// A handle to an open resource. Supports async read and write.
///
/// Returned by `VfsProvider::open()`. Implementations must be `Send + Sync`
/// for use across Tokio tasks.
///
/// Addresses: Requirement 5, criteria 1–3
#[async_trait]
pub trait VfsFile: Send + Sync {
    /// Read bytes from the file into the buffer. Returns the number of bytes read.
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, VfsError>;

    /// Write bytes to the file. Returns the number of bytes written.
    async fn write(&mut self, data: &[u8]) -> Result<usize, VfsError>;

    /// Flush all buffers to the underlying storage.
    async fn flush(&mut self) -> Result<(), VfsError>;

    /// Sync all data and metadata to durable storage (fsync equivalent).
    ///
    /// Addresses: Requirement 5 AC 3
    async fn sync_all(&mut self) -> Result<(), VfsError>;

    /// Close the file handle, releasing resources.
    async fn close(self: Box<Self>) -> Result<(), VfsError>;
}

/// Compile-time assertion that `VfsProvider` and `VfsFile` are object-safe
/// with `Send + Sync` bounds.
///
/// Addresses: Requirement 4 AC 8
fn _assert_object_safety() {
    fn _provider(_: &dyn VfsProvider) {}
    fn _file(_: &dyn VfsFile) {}
    fn _provider_send(_: Box<dyn VfsProvider + Send + Sync>) {}
    fn _file_send(_: Box<dyn VfsFile + Send + Sync>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::sync::Arc;

    use tokio::io::AsyncRead;

    /// A minimal mock provider that returns errors for all required methods.
    /// Used to test trait object construction and default method behaviour.
    struct MockProvider;

    #[async_trait]
    impl VfsProvider for MockProvider {
        fn scheme(&self) -> &str {
            "mock"
        }

        fn capabilities(&self) -> VfsCapabilities {
            VfsCapabilities::none()
        }

        async fn open(
            &self,
            path: &str,
            _options: OpenOptions,
        ) -> Result<Box<dyn VfsFile>, VfsError> {
            Err(VfsError::NotFound {
                uri: format!("vfs://mock/{path}"),
                operation: "open".to_string(),
            })
        }

        async fn read(&self, path: &str) -> Result<Vec<u8>, VfsError> {
            Err(VfsError::NotFound {
                uri: format!("vfs://mock/{path}"),
                operation: "read".to_string(),
            })
        }

        async fn read_stream(
            &self,
            path: &str,
        ) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
            Err(VfsError::NotFound {
                uri: format!("vfs://mock/{path}"),
                operation: "read_stream".to_string(),
            })
        }

        async fn write(&self, path: &str, _data: &[u8]) -> Result<(), VfsError> {
            Err(VfsError::NotFound {
                uri: format!("vfs://mock/{path}"),
                operation: "write".to_string(),
            })
        }

        async fn create(&self, path: &str, _options: CreateOptions) -> Result<(), VfsError> {
            Err(VfsError::NotFound {
                uri: format!("vfs://mock/{path}"),
                operation: "create".to_string(),
            })
        }

        async fn delete(&self, path: &str, _options: DeleteOptions) -> Result<(), VfsError> {
            Err(VfsError::NotFound {
                uri: format!("vfs://mock/{path}"),
                operation: "delete".to_string(),
            })
        }

        async fn rename(&self, old_path: &str, _new_path: &str) -> Result<(), VfsError> {
            Err(VfsError::NotFound {
                uri: format!("vfs://mock/{old_path}"),
                operation: "rename".to_string(),
            })
        }

        async fn list(&self, path: &str) -> Result<Vec<VfsEntry>, VfsError> {
            Err(VfsError::NotFound {
                uri: format!("vfs://mock/{path}"),
                operation: "list".to_string(),
            })
        }

        async fn stat(&self, path: &str) -> Result<VfsMetadata, VfsError> {
            Err(VfsError::NotFound {
                uri: format!("vfs://mock/{path}"),
                operation: "stat".to_string(),
            })
        }

        async fn exists(&self, _path: &str) -> Result<bool, VfsError> {
            Ok(false)
        }
    }

    // Validates: Requirement 4 AC 8
    #[test]
    fn trait_objects_are_object_safe() {
        // This test verifies that VfsProvider and VfsFile can be used as trait objects.
        // If these lines compile, the traits are object-safe.
        fn _accept_provider(_: &dyn VfsProvider) {}
        fn _accept_file(_: &dyn VfsFile) {}
        fn _accept_provider_boxed(_: Box<dyn VfsProvider + Send + Sync>) {}
        fn _accept_file_boxed(_: Box<dyn VfsFile + Send + Sync>) {}
    }

    // Validates: Requirement 4 AC 8
    #[test]
    fn mock_provider_can_be_stored_as_arc_dyn() {
        let provider: Arc<dyn VfsProvider> = Arc::new(MockProvider);
        assert_eq!(provider.scheme(), "mock");
        assert_eq!(provider.capabilities(), VfsCapabilities::none());
    }

    // Validates: Requirement 4 AC 9
    #[tokio::test]
    async fn watch_default_returns_unsupported_operation() {
        let provider: Arc<dyn VfsProvider> = Arc::new(MockProvider);
        let result = provider.watch("/some/path", WatchOptions::default()).await;

        match result {
            Err(VfsError::UnsupportedOperation {
                operation,
                provider: prov,
            }) => {
                assert_eq!(operation, "watch");
                assert_eq!(prov, "mock");
            }
            Err(other) => panic!("expected UnsupportedOperation, got: {other:?}"),
            Ok(_) => panic!("expected UnsupportedOperation error, got Ok"),
        }
    }

    // Validates: Requirement 4 AC 10
    #[tokio::test]
    async fn search_default_returns_unsupported_operation() {
        let provider: Arc<dyn VfsProvider> = Arc::new(MockProvider);
        let query = SearchQuery::Content("hello".to_string());
        let options = SearchOptions::default();
        let result = provider.search("/some/path", &query, &options).await;

        match result {
            Err(VfsError::UnsupportedOperation {
                operation,
                provider: prov,
            }) => {
                assert_eq!(operation, "search");
                assert_eq!(prov, "mock");
            }
            Err(other) => panic!("expected UnsupportedOperation, got: {other:?}"),
            Ok(_) => panic!("expected UnsupportedOperation error, got Ok"),
        }
    }

    // Validates: Requirement 4 AC 4
    #[test]
    fn mock_provider_capabilities_return_none() {
        let provider = MockProvider;
        let caps = provider.capabilities();
        assert!(!caps.read);
        assert!(!caps.write);
        assert!(!caps.watch);
        assert!(!caps.search);
    }

    // Validates: Requirement 4 AC 6
    #[test]
    fn mock_provider_scheme_returns_identifier() {
        let provider = MockProvider;
        assert_eq!(provider.scheme(), "mock");
    }
}
