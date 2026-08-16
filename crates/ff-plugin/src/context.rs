//! PluginContext — the sandboxed gateway to platform services.
//!
//! Each plugin receives an `Arc<PluginContext>` during initialization.
//! The context delegates to platform services while enforcing scoping
//! and security boundaries.

use std::sync::Arc;

use crate::capability::Capability;
use crate::error::PluginError;
use crate::event::{EventHandler, PlatformEvent, SubscriptionId};
use crate::traits::{
    CapabilityRegistrar, CommandRegistration, PluginCommand, PluginConfigAccess, PluginEventBus,
    PluginVfsAccess,
};
use crate::version::{Version, PLUGIN_API_VERSION};
use ff_logging::PluginLogHandle;

/// The sandboxed gateway through which plugins access platform services.
///
/// Provided to each plugin during `initialize`. Thread-safe (`Send + Sync`).
/// All operations are scoped to the owning plugin's namespace.
///
/// # Security
///
/// - Configuration access is limited to `[plugins.{plugin_name}]`
/// - Capability registrations are stamped with the plugin's identity
/// - No cross-plugin state access is possible through this interface
pub struct PluginContext {
    /// The identity of the owning plugin.
    plugin_name: String,
    /// Logging service handle (prefixes records with `[plugin:{name}]`).
    log_handle: Box<dyn PluginLogHandle>,
    /// Command registration service.
    command_service: Arc<dyn CommandRegistration>,
    /// Configuration access (scoped to plugin namespace).
    config_service: Arc<dyn PluginConfigAccess>,
    /// VFS access (read/write through virtual file system).
    vfs_service: Arc<dyn PluginVfsAccess>,
    /// Event subscription and emission.
    event_service: Arc<dyn PluginEventBus>,
    /// Capability registration.
    capability_service: Arc<dyn CapabilityRegistrar>,
    /// Current plugin API version.
    api_version: Version,
    /// Whether this plugin has declared NetworkAccess capability.
    has_network_access: bool,
}

/// Services injected by the platform into `PluginContext`.
///
/// The platform-core crate constructs this bundle and passes it to
/// the plugin registry, which uses it to build per-plugin contexts.
pub struct PlatformServices {
    /// Command registration service implementation.
    pub command_service: Arc<dyn CommandRegistration>,
    /// Configuration access service implementation.
    pub config_service: Arc<dyn PluginConfigAccess>,
    /// VFS access service implementation.
    pub vfs_service: Arc<dyn PluginVfsAccess>,
    /// Event bus service implementation.
    pub event_service: Arc<dyn PluginEventBus>,
    /// Capability registration service implementation.
    pub capability_service: Arc<dyn CapabilityRegistrar>,
}

impl PluginContext {
    /// Creates a new plugin context for the given plugin.
    ///
    /// The context scopes all operations to the plugin's identity and
    /// delegates to the provided platform services.
    pub fn new(plugin_name: &str, services: &PlatformServices) -> Self {
        let log_handle = ff_logging::create_plugin_handle(plugin_name);
        Self {
            plugin_name: plugin_name.to_string(),
            log_handle,
            command_service: Arc::clone(&services.command_service),
            config_service: Arc::clone(&services.config_service),
            vfs_service: Arc::clone(&services.vfs_service),
            event_service: Arc::clone(&services.event_service),
            capability_service: Arc::clone(&services.capability_service),
            api_version: PLUGIN_API_VERSION,
            has_network_access: false,
        }
    }

    /// Creates a new plugin context with network access capability.
    pub fn with_network_access(
        plugin_name: &str,
        services: &PlatformServices,
        has_network_access: bool,
    ) -> Self {
        let log_handle = ff_logging::create_plugin_handle(plugin_name);
        Self {
            plugin_name: plugin_name.to_string(),
            log_handle,
            command_service: Arc::clone(&services.command_service),
            config_service: Arc::clone(&services.config_service),
            vfs_service: Arc::clone(&services.vfs_service),
            event_service: Arc::clone(&services.event_service),
            capability_service: Arc::clone(&services.capability_service),
            api_version: PLUGIN_API_VERSION,
            has_network_access,
        }
    }

    /// Returns the plugin's name.
    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    /// Returns a reference to the logging handle for this plugin.
    ///
    /// Records are prefixed with `[plugin:{name}]`.
    pub fn log(&self) -> &dyn PluginLogHandle {
        self.log_handle.as_ref()
    }

    /// Register a command with the platform's command framework.
    ///
    /// # Errors
    ///
    /// Returns `PluginError` if registration fails.
    pub fn register_command(&self, command: PluginCommand) -> Result<(), PluginError> {
        self.command_service.register(&self.plugin_name, command)
    }

    /// Read a configuration value scoped to this plugin's namespace.
    ///
    /// Only keys under `[plugins.{plugin_name}]` are accessible.
    /// Access to keys outside this namespace returns `ConfigAccessDenied`.
    ///
    /// # Errors
    ///
    /// Returns `PluginError::ConfigAccessDenied` if key is outside the plugin's namespace.
    pub fn config_get(&self, key: &str) -> Result<Option<toml::Value>, PluginError> {
        self.validate_config_key(key)?;
        self.config_service.get(&self.plugin_name, key)
    }

    /// Write a configuration value scoped to this plugin's namespace.
    ///
    /// # Errors
    ///
    /// Returns `PluginError::ConfigAccessDenied` if key is outside the plugin's namespace.
    pub fn config_set(&self, key: &str, value: toml::Value) -> Result<(), PluginError> {
        self.validate_config_key(key)?;
        self.config_service.set(&self.plugin_name, key, value)
    }

    /// Access the VFS for file operations.
    pub fn vfs(&self) -> &dyn PluginVfsAccess {
        self.vfs_service.as_ref()
    }

    /// Subscribe to a platform event.
    pub fn subscribe_event(&self, event_type: &str, handler: EventHandler) -> SubscriptionId {
        self.event_service
            .subscribe(&self.plugin_name, event_type, handler)
    }

    /// Unsubscribe from a previously subscribed event.
    pub fn unsubscribe_event(&self, id: SubscriptionId) {
        self.event_service.unsubscribe(id);
    }

    /// Emit a platform event.
    pub fn emit_event(&self, event: PlatformEvent) {
        self.event_service.emit(event);
    }

    /// Register a capability with the platform's Capability Registry.
    ///
    /// The registration is stamped with this plugin's identity — ownership
    /// cannot be forged.
    ///
    /// # Errors
    ///
    /// Returns `PluginError` if registration fails (e.g., duplicate conflict).
    pub fn register_capability(&self, capability: Capability) -> Result<(), PluginError> {
        self.capability_service
            .register(&self.plugin_name, capability)
    }

    /// Query the current Plugin API version.
    pub fn api_version(&self) -> &Version {
        &self.api_version
    }

    /// Check whether this plugin has declared network access capability.
    ///
    /// # Errors
    ///
    /// Returns `PluginError::NetworkAccessDenied` if the plugin has not declared
    /// the NetworkAccess capability in its manifest.
    pub fn check_network_access(&self) -> Result<(), PluginError> {
        if self.has_network_access {
            Ok(())
        } else {
            Err(PluginError::NetworkAccessDenied {
                plugin: self.plugin_name.clone(),
            })
        }
    }

    /// Validates that a configuration key is within this plugin's namespace.
    ///
    /// Keys must not contain path traversal attempts or reference other
    /// plugin namespaces.
    fn validate_config_key(&self, key: &str) -> Result<(), PluginError> {
        // Reject keys that attempt path traversal
        if key.contains("..") || key.starts_with('/') || key.starts_with('\\') {
            return Err(PluginError::ConfigAccessDenied {
                plugin: self.plugin_name.clone(),
                key: key.to_string(),
            });
        }

        // Reject keys that reference other plugin namespaces
        // A key like "plugins.other-plugin.setting" would be a namespace violation
        if let Some(after_prefix) = key.strip_prefix("plugins.") {
            // Check if it references our own namespace
            if !after_prefix.starts_with(&self.plugin_name) {
                return Err(PluginError::ConfigAccessDenied {
                    plugin: self.plugin_name.clone(),
                    key: key.to_string(),
                });
            }
        }

        Ok(())
    }
}

// Compile-time assertion that PluginContext is Send + Sync
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PluginContext>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // ─── Mock Implementations ───────────────────────────────────────────────

    struct MockCommandService;
    impl CommandRegistration for MockCommandService {
        fn register(&self, _owner: &str, _command: PluginCommand) -> Result<(), PluginError> {
            Ok(())
        }
        fn unregister(&self, _owner: &str, _command_id: &str) -> Result<(), PluginError> {
            Ok(())
        }
    }

    struct MockConfigService {
        store: Mutex<std::collections::HashMap<String, toml::Value>>,
    }
    impl MockConfigService {
        fn new() -> Self {
            Self {
                store: Mutex::new(std::collections::HashMap::new()),
            }
        }
    }
    impl PluginConfigAccess for MockConfigService {
        fn get(&self, _plugin_name: &str, key: &str) -> Result<Option<toml::Value>, PluginError> {
            let store = self.store.lock().unwrap();
            Ok(store.get(key).cloned())
        }
        fn set(
            &self,
            _plugin_name: &str,
            key: &str,
            value: toml::Value,
        ) -> Result<(), PluginError> {
            let mut store = self.store.lock().unwrap();
            store.insert(key.to_string(), value);
            Ok(())
        }
    }

    struct MockVfsService;
    impl PluginVfsAccess for MockVfsService {
        fn read(&self, _uri: &str) -> Result<Vec<u8>, PluginError> {
            Ok(vec![])
        }
        fn write(&self, _uri: &str, _data: &[u8]) -> Result<(), PluginError> {
            Ok(())
        }
        fn exists(&self, _uri: &str) -> Result<bool, PluginError> {
            Ok(false)
        }
        fn list_directory(&self, _uri: &str) -> Result<Vec<String>, PluginError> {
            Ok(vec![])
        }
    }

    struct MockEventBus;
    impl PluginEventBus for MockEventBus {
        fn subscribe(
            &self,
            _owner: &str,
            _event_type: &str,
            _handler: EventHandler,
        ) -> SubscriptionId {
            SubscriptionId::new(1)
        }
        fn unsubscribe(&self, _id: SubscriptionId) {}
        fn emit(&self, _event: PlatformEvent) {}
    }

    struct MockCapabilityRegistrar;
    impl CapabilityRegistrar for MockCapabilityRegistrar {
        fn register(&self, _owner: &str, _capability: Capability) -> Result<(), PluginError> {
            Ok(())
        }
        fn unregister(&self, _owner: &str, _capability_id: &str) -> Result<(), PluginError> {
            Ok(())
        }
    }

    fn create_test_services() -> PlatformServices {
        PlatformServices {
            command_service: Arc::new(MockCommandService),
            config_service: Arc::new(MockConfigService::new()),
            vfs_service: Arc::new(MockVfsService),
            event_service: Arc::new(MockEventBus),
            capability_service: Arc::new(MockCapabilityRegistrar),
        }
    }

    fn create_test_context(name: &str) -> PluginContext {
        let services = create_test_services();
        PluginContext::new(name, &services)
    }

    // ─── Tests ──────────────────────────────────────────────────────────────

    #[test]
    fn plugin_context_returns_plugin_name() {
        // Validates: Requirement 2.1
        let ctx = create_test_context("my-plugin");
        assert_eq!(ctx.plugin_name(), "my-plugin");
    }

    #[test]
    fn plugin_context_api_version_returns_constant() {
        // Validates: Requirement 6.7
        let ctx = create_test_context("test");
        assert_eq!(ctx.api_version(), &PLUGIN_API_VERSION);
    }

    #[test]
    fn plugin_context_is_send_and_sync() {
        // Validates: Requirement 2.5
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PluginContext>();
    }

    #[test]
    fn config_get_rejects_path_traversal() {
        // Validates: Requirement 7.5
        let ctx = create_test_context("my-plugin");
        let result = ctx.config_get("../other/secret");
        assert!(result.is_err());
        match result.unwrap_err() {
            PluginError::ConfigAccessDenied { plugin, key } => {
                assert_eq!(plugin, "my-plugin");
                assert_eq!(key, "../other/secret");
            }
            _ => panic!("expected ConfigAccessDenied"),
        }
    }

    #[test]
    fn config_get_rejects_other_plugin_namespace() {
        // Validates: Requirement 7.5
        let ctx = create_test_context("my-plugin");
        let result = ctx.config_get("plugins.other-plugin.secret");
        assert!(result.is_err());
    }

    #[test]
    fn config_get_allows_own_namespace() {
        // Validates: Requirement 2.7
        let ctx = create_test_context("my-plugin");
        let result = ctx.config_get("plugins.my-plugin.setting");
        assert!(result.is_ok());
    }

    #[test]
    fn config_get_allows_simple_keys() {
        // Validates: Requirement 2.7
        let ctx = create_test_context("my-plugin");
        let result = ctx.config_get("timeout_ms");
        assert!(result.is_ok());
    }

    #[test]
    fn config_set_rejects_absolute_path() {
        // Validates: Requirement 7.5
        let ctx = create_test_context("my-plugin");
        let result = ctx.config_set("/etc/passwd", toml::Value::String("hack".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn network_access_denied_when_not_declared() {
        // Validates: Requirement 7.3
        let ctx = create_test_context("no-net-plugin");
        let result = ctx.check_network_access();
        assert!(result.is_err());
        match result.unwrap_err() {
            PluginError::NetworkAccessDenied { plugin } => {
                assert_eq!(plugin, "no-net-plugin");
            }
            _ => panic!("expected NetworkAccessDenied"),
        }
    }

    #[test]
    fn network_access_allowed_when_declared() {
        // Validates: Requirement 7.3
        let services = create_test_services();
        let ctx = PluginContext::with_network_access("net-plugin", &services, true);
        assert!(ctx.check_network_access().is_ok());
    }

    #[test]
    fn vfs_access_returns_service_reference() {
        // Validates: Requirement 7.2
        let ctx = create_test_context("vfs-test");
        // Just verify we can call vfs() and it works
        let result = ctx.vfs().exists("vfs://test/file.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn register_capability_delegates_to_service() {
        // Validates: Requirement 2.6
        use crate::capability::{Capability, CommandsCapability};
        use crate::version::Version;
        let ctx = create_test_context("cap-test");
        let cap = Capability::Commands(CommandsCapability {
            command_ids: vec!["test.cmd".to_string()],
            category: "test".to_string(),
            version: Version::new(1, 0, 0),
        });
        assert!(ctx.register_capability(cap).is_ok());
    }

    #[test]
    fn subscribe_event_returns_subscription_id() {
        // Validates: Requirement 2.2
        let ctx = create_test_context("event-test");
        let handler: EventHandler = Box::new(|_| {});
        let id = ctx.subscribe_event("config_changed", handler);
        assert_eq!(id.as_u64(), 1);
    }
}
