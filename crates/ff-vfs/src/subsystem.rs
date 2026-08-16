//! VFS Subsystem integration with ff-core lifecycle management.
//!
//! The [`VfsSubsystem`] implements the ff-core [`Subsystem`] trait to participate
//! in the deterministic startup and shutdown sequence. During initialization, it
//! creates a [`ProviderRegistry`] and [`Vfs`] instance. During shutdown, it
//! deregisters all providers and releases resources.
//!
//! Addresses: Cross-cutting Req 6 AC 1; Integration with ff-core

use std::sync::Arc;

use async_trait::async_trait;
use ff_core::error::CoreError;
use ff_core::lifecycle::{StartupOrder, Subsystem, SubsystemCriticality, SubsystemDescriptor};
use ff_core::service_registry::ServiceRegistry;

use crate::registry::ProviderRegistry;
use crate::vfs::Vfs;

/// VFS subsystem for lifecycle management integration with ff-core.
///
/// Manages the creation and teardown of the VFS abstraction layer as part of
/// the platform's deterministic startup sequence. The VFS is the third subsystem
/// to initialize (after logging and configuration).
///
/// # Lifecycle
///
/// 1. `initialize` — creates a [`ProviderRegistry`] and [`Vfs`], stores the
///    `Arc<Vfs>` internally for access by later subsystems.
/// 2. `shutdown` — deregisters all providers and releases the `Vfs` reference.
///
/// # Accessing the Vfs
///
/// After initialization, the `Vfs` instance is available via [`VfsSubsystem::vfs()`].
/// The startup orchestrator is responsible for registering the `Arc<Vfs>` with the
/// [`ServiceRegistry`] if other subsystems need to look it up by type.
pub struct VfsSubsystem {
    /// The Vfs instance created during initialization.
    vfs: Option<Arc<Vfs>>,
}

impl VfsSubsystem {
    /// Creates a new uninitialized VFS subsystem.
    pub fn new() -> Self {
        Self { vfs: None }
    }

    /// Returns a reference to the `Vfs` instance, if initialized.
    ///
    /// Returns `None` before `initialize()` is called or after `shutdown()`.
    pub fn vfs(&self) -> Option<&Arc<Vfs>> {
        self.vfs.as_ref()
    }
}

impl Default for VfsSubsystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subsystem for VfsSubsystem {
    /// Returns the subsystem descriptor: name "vfs", criticality Critical,
    /// order Vfs (third in startup after logging and configuration).
    fn descriptor(&self) -> SubsystemDescriptor {
        SubsystemDescriptor {
            name: "vfs",
            criticality: SubsystemCriticality::Critical,
            order: StartupOrder::Vfs,
        }
    }

    /// Initialize the VFS subsystem.
    ///
    /// Creates a fresh [`ProviderRegistry`] and wraps it in a [`Vfs`] facade.
    /// The resulting `Arc<Vfs>` is stored internally and can be retrieved via
    /// [`VfsSubsystem::vfs()`].
    ///
    /// # Errors
    ///
    /// Currently infallible — returns `Ok(())` on success.
    async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
        let provider_registry = ProviderRegistry::new();
        let vfs = Arc::new(Vfs::with_registry(provider_registry));
        self.vfs = Some(vfs);
        eprintln!("[vfs] subsystem initialized");
        Ok(())
    }

    /// Shut down the VFS subsystem.
    ///
    /// Deregisters all providers from the registry and releases the `Vfs`
    /// reference. After shutdown, [`VfsSubsystem::vfs()`] returns `None`.
    ///
    /// # Errors
    ///
    /// Currently infallible — returns `Ok(())` on success.
    async fn shutdown(&mut self) -> Result<(), CoreError> {
        if let Some(vfs) = self.vfs.take() {
            let schemes = vfs.registry().list_schemes();
            for scheme in &schemes {
                let _ = vfs.registry().deregister(scheme);
            }
            eprintln!(
                "[vfs] subsystem shut down ({} providers deregistered)",
                schemes.len()
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Cross-cutting Req 6 AC 1
    #[test]
    fn vfs_subsystem_descriptor_returns_correct_metadata() {
        let sub = VfsSubsystem::new();
        let desc = sub.descriptor();
        assert_eq!(desc.name, "vfs");
        assert_eq!(desc.criticality, SubsystemCriticality::Critical);
        assert_eq!(desc.order, StartupOrder::Vfs);
    }

    // Validates: Cross-cutting Req 6 AC 1
    #[test]
    fn vfs_subsystem_new_has_no_vfs() {
        let sub = VfsSubsystem::new();
        assert!(sub.vfs().is_none());
    }

    // Validates: Cross-cutting Req 6 AC 1
    #[tokio::test]
    async fn vfs_subsystem_initialize_creates_vfs() {
        let mut sub = VfsSubsystem::new();
        let registry = ServiceRegistry::new();
        let result = sub.initialize(&registry).await;
        assert!(result.is_ok());
        assert!(sub.vfs().is_some());
    }

    // Validates: Cross-cutting Req 6 AC 1
    #[tokio::test]
    async fn vfs_subsystem_initialize_creates_empty_registry() {
        let mut sub = VfsSubsystem::new();
        let registry = ServiceRegistry::new();
        sub.initialize(&registry).await.unwrap();
        let vfs = sub.vfs().unwrap();
        assert!(vfs.registry().list_schemes().is_empty());
    }

    // Validates: Cross-cutting Req 6 AC 1
    #[tokio::test]
    async fn vfs_subsystem_shutdown_releases_vfs() {
        let mut sub = VfsSubsystem::new();
        let registry = ServiceRegistry::new();
        sub.initialize(&registry).await.unwrap();
        assert!(sub.vfs().is_some());

        let result = sub.shutdown().await;
        assert!(result.is_ok());
        assert!(sub.vfs().is_none());
    }

    // Validates: Cross-cutting Req 6 AC 1
    #[tokio::test]
    async fn vfs_subsystem_shutdown_deregisters_providers() {
        use crate::provider::VfsFile;
        use crate::provider::VfsProvider;
        use crate::types::{
            CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsMetadata,
        };
        use std::pin::Pin;
        use tokio::io::AsyncRead;

        /// Minimal mock provider for shutdown testing.
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
                _path: &str,
                _options: OpenOptions,
            ) -> Result<Box<dyn VfsFile>, crate::VfsError> {
                unimplemented!()
            }

            async fn read(&self, _path: &str) -> Result<Vec<u8>, crate::VfsError> {
                unimplemented!()
            }

            async fn read_stream(
                &self,
                _path: &str,
            ) -> Result<Pin<Box<dyn AsyncRead + Send>>, crate::VfsError> {
                unimplemented!()
            }

            async fn write(&self, _path: &str, _data: &[u8]) -> Result<(), crate::VfsError> {
                unimplemented!()
            }

            async fn create(
                &self,
                _path: &str,
                _options: CreateOptions,
            ) -> Result<(), crate::VfsError> {
                unimplemented!()
            }

            async fn delete(
                &self,
                _path: &str,
                _options: DeleteOptions,
            ) -> Result<(), crate::VfsError> {
                unimplemented!()
            }

            async fn rename(
                &self,
                _old_path: &str,
                _new_path: &str,
            ) -> Result<(), crate::VfsError> {
                unimplemented!()
            }

            async fn list(&self, _path: &str) -> Result<Vec<VfsEntry>, crate::VfsError> {
                unimplemented!()
            }

            async fn stat(&self, _path: &str) -> Result<VfsMetadata, crate::VfsError> {
                unimplemented!()
            }

            async fn exists(&self, _path: &str) -> Result<bool, crate::VfsError> {
                Ok(false)
            }
        }

        let mut sub = VfsSubsystem::new();
        let service_registry = ServiceRegistry::new();
        sub.initialize(&service_registry).await.unwrap();

        // Register a mock provider
        let vfs = sub.vfs().unwrap();
        vfs.registry()
            .register(Arc::new(MockProvider) as Arc<dyn VfsProvider>)
            .unwrap();
        assert_eq!(vfs.registry().list_schemes().len(), 1);

        // Shutdown should deregister the provider
        sub.shutdown().await.unwrap();
        assert!(sub.vfs().is_none());
    }

    // Validates: Cross-cutting Req 6 AC 1
    #[tokio::test]
    async fn vfs_subsystem_shutdown_without_initialize_is_noop() {
        let mut sub = VfsSubsystem::new();
        let result = sub.shutdown().await;
        assert!(result.is_ok());
        assert!(sub.vfs().is_none());
    }
}
