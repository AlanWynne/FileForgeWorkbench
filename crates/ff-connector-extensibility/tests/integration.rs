//! Integration tests for the connector extensibility framework.
//!
//! Tests end-to-end flows: register → connect → operations → disconnect → deregister,
//! error cases, and hot-swap scenarios.

use std::pin::Pin;
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use ff_connector_extensibility::{
    ApiVersion, ConnectorCapability, ConnectorDescriptor, ConnectorError, ConnectorPlugin,
    ConnectorRegistry, ConnectorState, CredentialStore, RetryPolicy, CONNECTOR_API_VERSION,
};
use ff_core::EventBus;
use ff_plugin::context::PluginContext;
use ff_plugin::{Capability, FileForgePlugin, PluginError, PluginMetadata, Version};
use ff_vfs::provider::{VfsFile, VfsProvider};
use ff_vfs::types::{
    CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsMetadata,
};
use ff_vfs::{ProviderRegistry, VfsError};
use tokio::io::AsyncRead;

/// A mock connector implementing the full ConnectorPlugin trait for integration testing.
struct MockConnector {
    descriptor: ConnectorDescriptor,
    capabilities: Vec<ConnectorCapability>,
    api_version: ApiVersion,
    state: ConnectorState,
    policy: RetryPolicy,
}

impl MockConnector {
    fn new(scheme: &str) -> Self {
        Self {
            descriptor: ConnectorDescriptor {
                scheme: scheme.to_string(),
                display_name: format!("{scheme} Mock Connector"),
                description: format!("A mock {scheme} connector for testing"),
                icon: None,
                version: Version::new(1, 0, 0),
            },
            capabilities: vec![
                ConnectorCapability::Read,
                ConnectorCapability::Write,
                ConnectorCapability::List,
                ConnectorCapability::Metadata,
            ],
            api_version: CONNECTOR_API_VERSION,
            state: ConnectorState::Registered,
            policy: RetryPolicy::default(),
        }
    }

    fn with_capabilities(mut self, caps: Vec<ConnectorCapability>) -> Self {
        self.capabilities = caps;
        self
    }

    fn with_api_version(mut self, version: ApiVersion) -> Self {
        self.api_version = version;
        self
    }
}

#[async_trait]
impl VfsProvider for MockConnector {
    fn scheme(&self) -> &str {
        &self.descriptor.scheme
    }
    fn capabilities(&self) -> VfsCapabilities {
        VfsCapabilities::none()
    }
    async fn open(&self, _: &str, _: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "open".to_string(),
            provider: self.descriptor.scheme.clone(),
        })
    }
    async fn read(&self, _: &str) -> Result<Vec<u8>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "read".to_string(),
            provider: self.descriptor.scheme.clone(),
        })
    }
    async fn read_stream(&self, _: &str) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "read_stream".to_string(),
            provider: self.descriptor.scheme.clone(),
        })
    }
    async fn write(&self, _: &str, _: &[u8]) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "write".to_string(),
            provider: self.descriptor.scheme.clone(),
        })
    }
    async fn create(&self, _: &str, _: CreateOptions) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "create".to_string(),
            provider: self.descriptor.scheme.clone(),
        })
    }
    async fn delete(&self, _: &str, _: DeleteOptions) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "delete".to_string(),
            provider: self.descriptor.scheme.clone(),
        })
    }
    async fn rename(&self, _: &str, _: &str) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "rename".to_string(),
            provider: self.descriptor.scheme.clone(),
        })
    }
    async fn list(&self, _: &str) -> Result<Vec<VfsEntry>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "list".to_string(),
            provider: self.descriptor.scheme.clone(),
        })
    }
    async fn stat(&self, _: &str) -> Result<VfsMetadata, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "stat".to_string(),
            provider: self.descriptor.scheme.clone(),
        })
    }
    async fn exists(&self, _: &str) -> Result<bool, VfsError> {
        Ok(false)
    }
}

impl FileForgePlugin for MockConnector {
    fn metadata(&self) -> &PluginMetadata {
        static META: OnceLock<PluginMetadata> = OnceLock::new();
        META.get_or_init(|| PluginMetadata {
            name: "mock-connector".to_string(),
            version: Version::new(1, 0, 0),
            author: "Test".to_string(),
            description: "Mock connector for integration tests".to_string(),
            dependencies: vec![],
            required_api_version: Version::new(1, 0, 0),
        })
    }
    fn plugin_capabilities(&self) -> &[Capability] {
        &[]
    }
    fn initialize(&mut self, _: Arc<PluginContext>) -> Result<(), PluginError> {
        Ok(())
    }
    fn activate(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    fn deactivate(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait]
impl ConnectorPlugin for MockConnector {
    fn descriptor(&self) -> &ConnectorDescriptor {
        &self.descriptor
    }
    fn connector_capabilities(&self) -> &[ConnectorCapability] {
        &self.capabilities
    }
    fn api_version(&self) -> ApiVersion {
        self.api_version
    }
    fn state(&self) -> ConnectorState {
        self.state.clone()
    }
    async fn connect(&mut self) -> Result<(), ConnectorError> {
        self.state = ConnectorState::Connected;
        Ok(())
    }
    async fn disconnect(&mut self) -> Result<(), ConnectorError> {
        self.state = ConnectorState::Disconnected;
        Ok(())
    }
    async fn authenticate(&mut self, _: &dyn CredentialStore) -> Result<(), ConnectorError> {
        Ok(())
    }
    fn retry_policy(&self) -> &RetryPolicy {
        &self.policy
    }
    fn map_error(&self, source: Box<dyn std::error::Error + Send + Sync>) -> ConnectorError {
        ConnectorError::Internal {
            scheme: self.descriptor.scheme.clone(),
            message: source.to_string(),
        }
    }
}

fn make_registry() -> ConnectorRegistry {
    ConnectorRegistry::new(
        Arc::new(ProviderRegistry::new()),
        Arc::new(EventBus::with_default_capacity()),
    )
}

// ─── End-to-End Flow Tests ──────────────────────────────────────────────────

// Validates: Requirement 1 AC 1–6, Requirement 2 AC 1
#[tokio::test]
async fn end_to_end_register_connect_disconnect_deregister() {
    let registry = make_registry();

    // Register
    let connector = Box::new(MockConnector::new("ftp"));
    registry.register(connector).await.unwrap();

    // Verify registered
    let state = registry.get_connector_state("ftp").unwrap();
    assert_eq!(state, ConnectorState::Registered);

    // Connect
    registry.connect("ftp").await.unwrap();
    let state = registry.get_connector_state("ftp").unwrap();
    assert_eq!(state, ConnectorState::Connected);

    // Disconnect
    registry.disconnect("ftp").await.unwrap();
    let state = registry.get_connector_state("ftp").unwrap();
    assert_eq!(state, ConnectorState::Disconnected);

    // Deregister
    registry.deregister("ftp").await.unwrap();
    assert!(registry.get_connector_state("ftp").is_none());
}

// Validates: Requirement 2 AC 2a
#[tokio::test]
async fn duplicate_registration_fails() {
    let registry = make_registry();
    registry
        .register(Box::new(MockConnector::new("sftp")))
        .await
        .unwrap();

    let result = registry
        .register(Box::new(MockConnector::new("sftp")))
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("already registered"),
        "unexpected error: {err}"
    );
}

// Validates: Requirement 2 AC 2b, Requirement 3 AC 2
#[tokio::test]
async fn missing_capabilities_registration_fails() {
    let registry = make_registry();
    let connector = MockConnector::new("bad")
        .with_capabilities(vec![ConnectorCapability::Read, ConnectorCapability::Write]);

    let result = registry.register(Box::new(connector)).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("missing required capabilities"),
        "unexpected error: {err}"
    );
}

// Validates: Requirement 2 AC 2c
#[tokio::test]
async fn incompatible_api_version_registration_fails() {
    let registry = make_registry();
    let connector = MockConnector::new("v2").with_api_version(ApiVersion::new(2, 0, 0));

    let result = registry.register(Box::new(connector)).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("incompatible API version"),
        "unexpected error: {err}"
    );
}

// Validates: Requirement 2 AC 5
#[tokio::test]
async fn hot_swap_replaces_old_connector_with_new_version() {
    let registry = make_registry();

    // Register v1
    let v1 = MockConnector::new("cloud");
    registry.register(Box::new(v1)).await.unwrap();
    assert!(!registry.supports("cloud", ConnectorCapability::Search));

    // Hot swap with v2 that has additional capabilities
    let v2 = MockConnector::new("cloud").with_capabilities(vec![
        ConnectorCapability::Read,
        ConnectorCapability::List,
        ConnectorCapability::Metadata,
        ConnectorCapability::Search,
    ]);
    registry.hot_swap(Box::new(v2)).await.unwrap();

    // Verify new capabilities
    assert!(registry.supports("cloud", ConnectorCapability::Search));
    assert!(registry.supports("cloud", ConnectorCapability::Read));
}

// Validates: Requirement 3 AC 3, AC 5
#[tokio::test]
async fn capability_queries_match_declared_capabilities() {
    let registry = make_registry();
    let connector = MockConnector::new("test").with_capabilities(vec![
        ConnectorCapability::Read,
        ConnectorCapability::List,
        ConnectorCapability::Metadata,
        ConnectorCapability::Write,
        ConnectorCapability::Delete,
    ]);
    registry.register(Box::new(connector)).await.unwrap();

    // Positive checks
    assert!(registry.supports("test", ConnectorCapability::Read));
    assert!(registry.supports("test", ConnectorCapability::Write));
    assert!(registry.supports("test", ConnectorCapability::Delete));

    // Negative checks
    assert!(!registry.supports("test", ConnectorCapability::Watch));
    assert!(!registry.supports("test", ConnectorCapability::Search));
    assert!(!registry.supports("test", ConnectorCapability::Copy));

    // capabilities_for returns exact set
    let caps = registry.capabilities_for("test").unwrap();
    assert_eq!(caps.len(), 5);
}

// Validates: Requirement 4 AC 6
#[tokio::test]
async fn shutdown_all_disconnects_connected_connectors() {
    let registry = make_registry();
    registry
        .register(Box::new(MockConnector::new("a")))
        .await
        .unwrap();
    registry
        .register(Box::new(MockConnector::new("b")))
        .await
        .unwrap();

    // Connect both
    registry.connect("a").await.unwrap();
    registry.connect("b").await.unwrap();

    // Shutdown all
    registry
        .shutdown_all(std::time::Duration::from_secs(5))
        .await;

    // Both should be disconnected
    let state_a = registry.get_connector_state("a").unwrap();
    let state_b = registry.get_connector_state("b").unwrap();
    assert_eq!(state_a, ConnectorState::Disconnected);
    assert_eq!(state_b, ConnectorState::Disconnected);
}

// Validates: Requirement 3 AC 6
#[tokio::test]
async fn refresh_capabilities_updates_capability_set() {
    let registry = make_registry();
    registry
        .register(Box::new(MockConnector::new("dynamic")))
        .await
        .unwrap();

    // Initially has Read, Write, List, Metadata
    assert!(registry.supports("dynamic", ConnectorCapability::Write));
    assert!(!registry.supports("dynamic", ConnectorCapability::Search));

    // Refresh with different set
    registry
        .refresh_capabilities(
            "dynamic",
            vec![
                ConnectorCapability::Read,
                ConnectorCapability::List,
                ConnectorCapability::Metadata,
                ConnectorCapability::Search,
            ],
        )
        .unwrap();

    // Write is gone, Search is present
    assert!(!registry.supports("dynamic", ConnectorCapability::Write));
    assert!(registry.supports("dynamic", ConnectorCapability::Search));
}
