//! Integration tests for VfsSubsystem lifecycle.
//!
//! Validates: Cross-cutting Req 6 AC 1 — VFS integrates with ff-core lifecycle

use std::sync::Arc;

use async_trait::async_trait;
use ff_core::lifecycle::{StartupOrder, Subsystem, SubsystemCriticality};
use ff_core::service_registry::ServiceRegistry;
use ff_vfs::provider::{VfsFile, VfsProvider};
use ff_vfs::types::{
    CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsMetadata,
};
use ff_vfs::{VfsError, VfsSubsystem};
use std::pin::Pin;
use tokio::io::AsyncRead;

/// A minimal mock provider for integration testing.
struct TestProvider {
    scheme_name: String,
}

impl TestProvider {
    fn new(scheme: &str) -> Self {
        Self {
            scheme_name: scheme.to_string(),
        }
    }
}

#[async_trait]
impl VfsProvider for TestProvider {
    fn scheme(&self) -> &str {
        &self.scheme_name
    }

    fn capabilities(&self) -> VfsCapabilities {
        VfsCapabilities::none()
    }

    async fn open(&self, _path: &str, _options: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "open".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn read(&self, _path: &str) -> Result<Vec<u8>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "read".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn read_stream(&self, _path: &str) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "read_stream".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn write(&self, _path: &str, _data: &[u8]) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "write".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn create(&self, _path: &str, _options: CreateOptions) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "create".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn delete(&self, _path: &str, _options: DeleteOptions) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "delete".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn rename(&self, _old_path: &str, _new_path: &str) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "rename".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn list(&self, _path: &str) -> Result<Vec<VfsEntry>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "list".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn stat(&self, _path: &str) -> Result<VfsMetadata, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "stat".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn exists(&self, _path: &str) -> Result<bool, VfsError> {
        Ok(false)
    }
}

/// Validates: Cross-cutting Req 6 AC 1
/// Full lifecycle: initialize → use → shutdown
#[tokio::test]
async fn vfs_subsystem_lifecycle() {
    let mut sub = VfsSubsystem::new();

    // Verify descriptor metadata
    let desc = sub.descriptor();
    assert_eq!(desc.name, "vfs");
    assert_eq!(desc.criticality, SubsystemCriticality::Critical);
    assert_eq!(desc.order, StartupOrder::Vfs);

    // Before initialization, vfs() returns None
    assert!(sub.vfs().is_none());

    // Initialize
    let registry = ServiceRegistry::new();
    let result = sub.initialize(&registry).await;
    assert!(result.is_ok());

    // After initialization, vfs() returns Some with empty registry
    let vfs = sub.vfs().unwrap();
    assert!(vfs.registry().list_schemes().is_empty());

    // Shutdown
    let result = sub.shutdown().await;
    assert!(result.is_ok());

    // After shutdown, vfs() returns None
    assert!(sub.vfs().is_none());
}

/// Validates: Cross-cutting Req 6 AC 1
/// Shutdown deregisters all providers that were registered.
#[tokio::test]
async fn vfs_subsystem_shutdown_deregisters_all_providers() {
    let mut sub = VfsSubsystem::new();
    let service_registry = ServiceRegistry::new();
    sub.initialize(&service_registry).await.unwrap();

    // Register multiple providers
    let vfs = sub.vfs().unwrap();
    vfs.registry()
        .register(Arc::new(TestProvider::new("local")) as Arc<dyn VfsProvider>)
        .unwrap();
    vfs.registry()
        .register(Arc::new(TestProvider::new("remote")) as Arc<dyn VfsProvider>)
        .unwrap();
    assert_eq!(vfs.registry().list_schemes().len(), 2);

    // Shutdown should deregister all
    sub.shutdown().await.unwrap();
    assert!(sub.vfs().is_none());
}

/// Validates: Cross-cutting Req 6 AC 1
/// Shutdown is safe to call multiple times.
#[tokio::test]
async fn vfs_subsystem_double_shutdown_is_safe() {
    let mut sub = VfsSubsystem::new();
    let registry = ServiceRegistry::new();
    sub.initialize(&registry).await.unwrap();

    sub.shutdown().await.unwrap();
    // Second shutdown is a no-op
    let result = sub.shutdown().await;
    assert!(result.is_ok());
}

/// Validates: Cross-cutting Req 6 AC 1
/// The subsystem can be re-initialized after shutdown.
#[tokio::test]
async fn vfs_subsystem_reinitialize_after_shutdown() {
    let mut sub = VfsSubsystem::new();
    let registry = ServiceRegistry::new();

    // First lifecycle
    sub.initialize(&registry).await.unwrap();
    assert!(sub.vfs().is_some());
    sub.shutdown().await.unwrap();
    assert!(sub.vfs().is_none());

    // Second lifecycle
    sub.initialize(&registry).await.unwrap();
    assert!(sub.vfs().is_some());
    sub.shutdown().await.unwrap();
    assert!(sub.vfs().is_none());
}
