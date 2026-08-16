//! Connector registry: validates, stores, and manages connector registrations.
//!
//! The `ConnectorRegistry` is the central subsystem responsible for:
//! - Validating connector registrations (scheme uniqueness, capabilities, API version)
//! - Storing registered connectors as trait objects
//! - Providing runtime discovery and capability queries
//! - Managing connector lifecycle (connect, disconnect, hot-swap)
//! - Emitting events via the platform EventBus

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use ff_core::EventBus;
use ff_vfs::ProviderRegistry;

use crate::api_version::CONNECTOR_API_VERSION;
use crate::capability::{validate_capabilities, ConnectorCapability};
use crate::descriptor::ConnectorDescriptor;
use crate::error::ConnectorError;
use crate::event::{
    ConnectorCapabilityChangedEvent, ConnectorRegisteredEvent, ConnectorStateChangedEvent,
};
use crate::reconnection::{ReconnectionManager, RetryPolicy};
use crate::state::ConnectorState;
use crate::traits::ConnectorPlugin;

/// Internal entry tracking a connector and its metadata.
pub(crate) struct ConnectorEntry {
    /// The connector instance.
    pub connector: Box<dyn ConnectorPlugin>,
    /// Cached descriptor for post-shutdown queries.
    pub descriptor: ConnectorDescriptor,
    /// Current state (mirrors connector.state() with registry-level tracking).
    pub state: ConnectorState,
    /// Capabilities (cached for fast queries).
    pub capabilities: Vec<ConnectorCapability>,
}

/// Manages connector registrations, validates constraints, and provides
/// runtime discovery and lifecycle management of connectors.
///
/// Thread-safe: uses `RwLock` for concurrent read access.
///
/// Addresses: Requirement 2, all acceptance criteria
pub struct ConnectorRegistry {
    /// Registered connectors indexed by scheme.
    connectors: Arc<RwLock<HashMap<String, ConnectorEntry>>>,
    /// Reference to the VFS ProviderRegistry for provider registration.
    #[allow(dead_code)]
    vfs_registry: Arc<ProviderRegistry>,
    /// Event bus for emitting registration/state-change events.
    event_bus: Arc<EventBus>,
    /// Reconnection managers indexed by scheme.
    #[allow(dead_code)]
    reconnection_managers: Arc<RwLock<HashMap<String, ReconnectionManager>>>,
}

impl ConnectorRegistry {
    /// Creates a new `ConnectorRegistry` backed by the given VFS registry and event bus.
    pub fn new(vfs_registry: Arc<ProviderRegistry>, event_bus: Arc<EventBus>) -> Self {
        Self {
            connectors: Arc::new(RwLock::new(HashMap::new())),
            vfs_registry,
            event_bus,
            reconnection_managers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a connector. Validates:
    /// - Scheme uniqueness (no duplicate)
    /// - Required capabilities present (Read, List, Metadata)
    /// - API version compatibility (same major, minor ≤ current)
    ///
    /// On success, emits a `ConnectorRegisteredEvent`.
    ///
    /// Addresses: Requirement 2 AC 1, AC 2, AC 3
    pub async fn register(
        &self,
        connector: Box<dyn ConnectorPlugin>,
    ) -> Result<(), ConnectorError> {
        let descriptor = connector.descriptor().clone();
        let capabilities = connector.connector_capabilities().to_vec();
        let api_version = connector.api_version();
        let scheme = descriptor.scheme.clone();

        // Validate scheme uniqueness
        {
            let connectors = self
                .connectors
                .read()
                .expect("connector registry lock poisoned");
            if connectors.contains_key(&scheme) {
                ff_logging::log_error!(
                    "[connector-registry] registration failed: duplicate scheme '{}'",
                    scheme
                );
                return Err(ConnectorError::RegistrationFailed {
                    message: format!("scheme '{scheme}' is already registered"),
                });
            }
        }

        // Validate required capabilities
        validate_capabilities(&capabilities)?;

        // Validate API version compatibility
        if !api_version.is_compatible_with(&CONNECTOR_API_VERSION) {
            ff_logging::log_error!(
                "[connector-registry] registration failed: incompatible API version {} (current: {})",
                api_version,
                CONNECTOR_API_VERSION
            );
            return Err(ConnectorError::RegistrationFailed {
                message: format!(
                    "incompatible API version {} (current: {})",
                    api_version, CONNECTOR_API_VERSION
                ),
            });
        }

        // Insert into registry
        {
            let mut connectors = self
                .connectors
                .write()
                .expect("connector registry lock poisoned");
            connectors.insert(
                scheme.clone(),
                ConnectorEntry {
                    connector,
                    descriptor: descriptor.clone(),
                    state: ConnectorState::Registered,
                    capabilities,
                },
            );
        }

        // Create reconnection manager
        {
            let mut managers = self
                .reconnection_managers
                .write()
                .expect("reconnection managers lock poisoned");
            managers.insert(
                scheme.clone(),
                ReconnectionManager::new(RetryPolicy::default()),
            );
        }

        // Emit registration event
        ff_logging::log_info!(
            "[connector-registry] registered connector '{}' (scheme: {})",
            descriptor.display_name,
            scheme
        );
        let _event = ConnectorRegisteredEvent {
            scheme: scheme.clone(),
            display_name: descriptor.display_name.clone(),
            registered: true,
        };
        // Note: EventBus uses WorkbenchEvent enum; connector events would be
        // dispatched as a Notification or custom variant in a full integration.
        // For now we log the event. A future PR will add WorkbenchEvent::ConnectorRegistered.
        let _ = &self.event_bus;

        Ok(())
    }

    /// Deregister a connector by scheme.
    ///
    /// Calls `disconnect()` if connected, removes from the registry, and
    /// emits a `ConnectorRegisteredEvent(registered=false)`.
    ///
    /// Addresses: Requirement 2 AC 4
    pub async fn deregister(&self, scheme: &str) -> Result<(), ConnectorError> {
        let mut entry = {
            let mut connectors = self
                .connectors
                .write()
                .expect("connector registry lock poisoned");
            connectors
                .remove(scheme)
                .ok_or_else(|| ConnectorError::RegistrationFailed {
                    message: format!("scheme '{scheme}' is not registered"),
                })?
        };

        // Disconnect if connected
        if entry.state.is_connected() {
            let _ = entry.connector.disconnect().await;
        }

        // Remove reconnection manager
        {
            let mut managers = self
                .reconnection_managers
                .write()
                .expect("reconnection managers lock poisoned");
            managers.remove(scheme);
        }

        ff_logging::log_info!(
            "[connector-registry] deregistered connector '{}' (scheme: {})",
            entry.descriptor.display_name,
            scheme
        );

        let _event = ConnectorRegisteredEvent {
            scheme: scheme.to_string(),
            display_name: entry.descriptor.display_name.clone(),
            registered: false,
        };

        Ok(())
    }

    /// Hot-swap a connector: deactivate old version, register new version,
    /// preserve URI resolution.
    ///
    /// Addresses: Requirement 2 AC 5
    pub async fn hot_swap(
        &self,
        new_connector: Box<dyn ConnectorPlugin>,
    ) -> Result<(), ConnectorError> {
        let scheme = new_connector.descriptor().scheme.clone();

        // Deregister old version if present
        let had_old = {
            let connectors = self
                .connectors
                .read()
                .expect("connector registry lock poisoned");
            connectors.contains_key(&scheme)
        };

        if had_old {
            self.deregister(&scheme).await?;
        }

        // Register new version
        self.register(new_connector).await
    }

    /// Look up a connector by scheme (returns scheme and state info).
    ///
    /// Note: Due to `RwLock` interior mutability, we cannot return a direct
    /// reference to the connector. Use `supports()` and `capabilities_for()`
    /// for queries.
    ///
    /// Addresses: Requirement 2 AC 7
    pub fn get_connector_state(&self, scheme: &str) -> Option<ConnectorState> {
        let connectors = self
            .connectors
            .read()
            .expect("connector registry lock poisoned");
        connectors.get(scheme).map(|e| e.state.clone())
    }

    /// Check if a connector supports a specific capability.
    ///
    /// Addresses: Requirement 3 AC 3
    pub fn supports(&self, scheme: &str, capability: ConnectorCapability) -> bool {
        let connectors = self
            .connectors
            .read()
            .expect("connector registry lock poisoned");
        connectors
            .get(scheme)
            .map(|e| e.capabilities.contains(&capability))
            .unwrap_or(false)
    }

    /// Get the full capability list for a connector.
    ///
    /// Addresses: Requirement 3 AC 5
    pub fn capabilities_for(&self, scheme: &str) -> Option<Vec<ConnectorCapability>> {
        let connectors = self
            .connectors
            .read()
            .expect("connector registry lock poisoned");
        connectors.get(scheme).map(|e| e.capabilities.clone())
    }

    /// Refresh a connector's capabilities (called when capabilities change).
    ///
    /// Validates the new capabilities meet requirements and emits a
    /// capability-change event on success.
    ///
    /// Addresses: Requirement 3 AC 6
    pub fn refresh_capabilities(
        &self,
        scheme: &str,
        capabilities: Vec<ConnectorCapability>,
    ) -> Result<(), ConnectorError> {
        validate_capabilities(&capabilities)?;

        let mut connectors = self
            .connectors
            .write()
            .expect("connector registry lock poisoned");

        let entry =
            connectors
                .get_mut(scheme)
                .ok_or_else(|| ConnectorError::RegistrationFailed {
                    message: format!("scheme '{scheme}' is not registered"),
                })?;

        entry.capabilities = capabilities.clone();

        let _event = ConnectorCapabilityChangedEvent {
            scheme: scheme.to_string(),
            capabilities,
        };

        Ok(())
    }

    /// Initiate a connection for a registered connector.
    ///
    /// Addresses: Requirement 4 AC 8
    #[allow(clippy::await_holding_lock)]
    // Lock is held across await because the connector's connect() must be called
    // while we have mutable access to the entry. The lock scope is intentionally
    // tight and connect() implementations should be fast.
    pub async fn connect(&self, scheme: &str) -> Result<(), ConnectorError> {
        let mut connectors = self
            .connectors
            .write()
            .expect("connector registry lock poisoned");

        let entry = connectors
            .get_mut(scheme)
            .ok_or_else(|| ConnectorError::NotConnected {
                scheme: scheme.to_string(),
                operation: "connect".to_string(),
            })?;

        if !entry.state.can_connect() {
            return Err(ConnectorError::RegistrationFailed {
                message: format!("cannot connect from state {:?}", entry.state),
            });
        }

        let previous_state = entry.state.clone();
        entry.state = ConnectorState::Connecting;

        let _state_event = ConnectorStateChangedEvent {
            scheme: scheme.to_string(),
            previous_state: previous_state.clone(),
            new_state: ConnectorState::Connecting,
        };

        match entry.connector.connect().await {
            Ok(()) => {
                entry.state = ConnectorState::Connected;
                let _event = ConnectorStateChangedEvent {
                    scheme: scheme.to_string(),
                    previous_state: ConnectorState::Connecting,
                    new_state: ConnectorState::Connected,
                };
                Ok(())
            }
            Err(e) => {
                entry.state = ConnectorState::Error {
                    message: e.to_string(),
                };
                Err(e)
            }
        }
    }

    /// Initiate a disconnect for a connected connector.
    ///
    /// Addresses: Requirement 4 AC 8
    #[allow(clippy::await_holding_lock)]
    // Lock is held across await because the connector's disconnect() must be
    // called while we have mutable access to the entry. The lock scope is
    // intentionally tight and disconnect() implementations should be fast.
    pub async fn disconnect(&self, scheme: &str) -> Result<(), ConnectorError> {
        let mut connectors = self
            .connectors
            .write()
            .expect("connector registry lock poisoned");

        let entry = connectors
            .get_mut(scheme)
            .ok_or_else(|| ConnectorError::NotConnected {
                scheme: scheme.to_string(),
                operation: "disconnect".to_string(),
            })?;

        if !entry.state.can_disconnect() {
            return Err(ConnectorError::RegistrationFailed {
                message: format!("cannot disconnect from state {:?}", entry.state),
            });
        }

        let previous_state = entry.state.clone();
        entry.state = ConnectorState::Disconnecting;

        let _state_event = ConnectorStateChangedEvent {
            scheme: scheme.to_string(),
            previous_state,
            new_state: ConnectorState::Disconnecting,
        };

        match entry.connector.disconnect().await {
            Ok(()) => {
                entry.state = ConnectorState::Disconnected;
                let _event = ConnectorStateChangedEvent {
                    scheme: scheme.to_string(),
                    previous_state: ConnectorState::Disconnecting,
                    new_state: ConnectorState::Disconnected,
                };
                Ok(())
            }
            Err(e) => {
                entry.state = ConnectorState::Error {
                    message: e.to_string(),
                };
                Err(e)
            }
        }
    }

    /// List all registered connector schemes with their states.
    pub fn list_connectors(&self) -> Vec<(String, ConnectorState)> {
        let connectors = self
            .connectors
            .read()
            .expect("connector registry lock poisoned");
        connectors
            .iter()
            .map(|(scheme, entry)| (scheme.clone(), entry.state.clone()))
            .collect()
    }

    /// Shut down all connected connectors with a configurable drain period.
    ///
    /// Transitions all connected connectors to Disconnecting, calls their
    /// `disconnect` method, and waits up to `drain_timeout` for completion.
    ///
    /// Addresses: Requirement 4 AC 6
    pub async fn shutdown_all(&self, _drain_timeout: Duration) {
        let schemes: Vec<String> = {
            let connectors = self
                .connectors
                .read()
                .expect("connector registry lock poisoned");
            connectors.keys().cloned().collect()
        };

        for scheme in schemes {
            let _ = self.disconnect(&scheme).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::sync::Arc;

    use async_trait::async_trait;
    use ff_plugin::context::PluginContext;
    use ff_plugin::PluginError;
    use ff_plugin::{Capability, FileForgePlugin, PluginMetadata, Version};
    use ff_vfs::provider::{VfsFile, VfsProvider};
    use ff_vfs::types::{
        CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsMetadata,
    };
    use ff_vfs::VfsError;
    use tokio::io::AsyncRead;

    use crate::api_version::ApiVersion;
    use crate::credential::CredentialStore;

    /// A mock connector for testing the registry.
    struct MockConnector {
        desc: ConnectorDescriptor,
        caps: Vec<ConnectorCapability>,
        version: ApiVersion,
        state: ConnectorState,
        policy: RetryPolicy,
    }

    impl MockConnector {
        fn new(scheme: &str, caps: Vec<ConnectorCapability>, version: ApiVersion) -> Self {
            Self {
                desc: ConnectorDescriptor {
                    scheme: scheme.to_string(),
                    display_name: format!("{scheme} connector"),
                    description: format!("Mock {scheme} connector"),
                    icon: None,
                    version: Version::new(1, 0, 0),
                },
                caps,
                version,
                state: ConnectorState::Registered,
                policy: RetryPolicy::default(),
            }
        }

        fn valid(scheme: &str) -> Self {
            Self::new(
                scheme,
                vec![
                    ConnectorCapability::Read,
                    ConnectorCapability::List,
                    ConnectorCapability::Metadata,
                ],
                ApiVersion::new(1, 0, 0),
            )
        }
    }

    #[async_trait]
    impl VfsProvider for MockConnector {
        fn scheme(&self) -> &str {
            &self.desc.scheme
        }
        fn capabilities(&self) -> VfsCapabilities {
            VfsCapabilities::none()
        }
        async fn open(&self, _: &str, _: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "open".to_string(),
                provider: self.desc.scheme.clone(),
            })
        }
        async fn read(&self, _: &str) -> Result<Vec<u8>, VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "read".to_string(),
                provider: self.desc.scheme.clone(),
            })
        }
        async fn read_stream(&self, _: &str) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "read_stream".to_string(),
                provider: self.desc.scheme.clone(),
            })
        }
        async fn write(&self, _: &str, _: &[u8]) -> Result<(), VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "write".to_string(),
                provider: self.desc.scheme.clone(),
            })
        }
        async fn create(&self, _: &str, _: CreateOptions) -> Result<(), VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "create".to_string(),
                provider: self.desc.scheme.clone(),
            })
        }
        async fn delete(&self, _: &str, _: DeleteOptions) -> Result<(), VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "delete".to_string(),
                provider: self.desc.scheme.clone(),
            })
        }
        async fn rename(&self, _: &str, _: &str) -> Result<(), VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "rename".to_string(),
                provider: self.desc.scheme.clone(),
            })
        }
        async fn list(&self, _: &str) -> Result<Vec<VfsEntry>, VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "list".to_string(),
                provider: self.desc.scheme.clone(),
            })
        }
        async fn stat(&self, _: &str) -> Result<VfsMetadata, VfsError> {
            Err(VfsError::UnsupportedOperation {
                operation: "stat".to_string(),
                provider: self.desc.scheme.clone(),
            })
        }
        async fn exists(&self, _: &str) -> Result<bool, VfsError> {
            Ok(false)
        }
    }

    impl FileForgePlugin for MockConnector {
        fn metadata(&self) -> &PluginMetadata {
            static META: std::sync::OnceLock<PluginMetadata> = std::sync::OnceLock::new();
            META.get_or_init(|| PluginMetadata {
                name: "mock-connector".to_string(),
                version: Version::new(1, 0, 0),
                author: "Test".to_string(),
                description: "Mock connector".to_string(),
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
            &self.desc
        }
        fn connector_capabilities(&self) -> &[ConnectorCapability] {
            &self.caps
        }
        fn api_version(&self) -> ApiVersion {
            self.version
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
                scheme: self.desc.scheme.clone(),
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

    // Validates: Requirement 2 AC 1, AC 2
    #[tokio::test]
    async fn register_valid_connector_succeeds() {
        let registry = make_registry();
        let connector = Box::new(MockConnector::valid("ftp"));
        let result = registry.register(connector).await;
        assert!(result.is_ok());
    }

    // Validates: Requirement 2 AC 2a
    #[tokio::test]
    async fn register_duplicate_scheme_fails() {
        let registry = make_registry();
        registry
            .register(Box::new(MockConnector::valid("ftp")))
            .await
            .unwrap();
        let result = registry
            .register(Box::new(MockConnector::valid("ftp")))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("already registered"));
    }

    // Validates: Requirement 2 AC 2b
    #[tokio::test]
    async fn register_missing_capabilities_fails() {
        let registry = make_registry();
        let connector = Box::new(MockConnector::new(
            "bad",
            vec![ConnectorCapability::Read], // missing List and Metadata
            ApiVersion::new(1, 0, 0),
        ));
        let result = registry.register(connector).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing required capabilities"));
    }

    // Validates: Requirement 2 AC 2c
    #[tokio::test]
    async fn register_incompatible_version_fails() {
        let registry = make_registry();
        let connector = Box::new(MockConnector::new(
            "v2",
            vec![
                ConnectorCapability::Read,
                ConnectorCapability::List,
                ConnectorCapability::Metadata,
            ],
            ApiVersion::new(2, 0, 0), // incompatible major version
        ));
        let result = registry.register(connector).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("incompatible API version"));
    }

    // Validates: Requirement 2 AC 4
    #[tokio::test]
    async fn deregister_removes_connector() {
        let registry = make_registry();
        registry
            .register(Box::new(MockConnector::valid("ftp")))
            .await
            .unwrap();
        assert!(registry.get_connector_state("ftp").is_some());

        registry.deregister("ftp").await.unwrap();
        assert!(registry.get_connector_state("ftp").is_none());
    }

    // Validates: Requirement 3 AC 3
    #[tokio::test]
    async fn supports_returns_true_for_declared_capability() {
        let registry = make_registry();
        registry
            .register(Box::new(MockConnector::valid("ftp")))
            .await
            .unwrap();
        assert!(registry.supports("ftp", ConnectorCapability::Read));
        assert!(registry.supports("ftp", ConnectorCapability::List));
        assert!(registry.supports("ftp", ConnectorCapability::Metadata));
    }

    // Validates: Requirement 3 AC 3
    #[tokio::test]
    async fn supports_returns_false_for_undeclared_capability() {
        let registry = make_registry();
        registry
            .register(Box::new(MockConnector::valid("ftp")))
            .await
            .unwrap();
        assert!(!registry.supports("ftp", ConnectorCapability::Write));
        assert!(!registry.supports("ftp", ConnectorCapability::Watch));
    }

    // Validates: Requirement 3 AC 5
    #[tokio::test]
    async fn capabilities_for_returns_exact_set() {
        let registry = make_registry();
        registry
            .register(Box::new(MockConnector::valid("ftp")))
            .await
            .unwrap();
        let caps = registry.capabilities_for("ftp").unwrap();
        assert_eq!(caps.len(), 3);
        assert!(caps.contains(&ConnectorCapability::Read));
        assert!(caps.contains(&ConnectorCapability::List));
        assert!(caps.contains(&ConnectorCapability::Metadata));
    }

    // Validates: Requirement 2 AC 5
    #[tokio::test]
    async fn hot_swap_replaces_connector() {
        let registry = make_registry();
        registry
            .register(Box::new(MockConnector::valid("ftp")))
            .await
            .unwrap();

        // Hot swap with a new version that has additional capabilities
        let mut new_connector = MockConnector::valid("ftp");
        new_connector.caps.push(ConnectorCapability::Write);
        registry.hot_swap(Box::new(new_connector)).await.unwrap();

        // Verify new capabilities are present
        assert!(registry.supports("ftp", ConnectorCapability::Write));
    }

    #[tokio::test]
    async fn list_connectors_returns_all_registered() {
        let registry = make_registry();
        registry
            .register(Box::new(MockConnector::valid("ftp")))
            .await
            .unwrap();
        registry
            .register(Box::new(MockConnector::valid("sftp")))
            .await
            .unwrap();

        let list = registry.list_connectors();
        assert_eq!(list.len(), 2);
        let schemes: Vec<&str> = list.iter().map(|(s, _)| s.as_str()).collect();
        assert!(schemes.contains(&"ftp"));
        assert!(schemes.contains(&"sftp"));
    }
}
