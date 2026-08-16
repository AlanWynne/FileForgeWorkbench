//! Plugin trait definitions.
//!
//! Defines the `FileForgePlugin` trait (the primary contract for all plugins)
//! and service traits that platform-core implements and injects via `PluginContext`.

use std::sync::Arc;

use crate::capability::Capability;
use crate::error::PluginError;
use crate::event::{EventHandler, PlatformEvent, SubscriptionId};
use crate::metadata::PluginMetadata;

// Forward declare PluginContext to avoid circular dependency
use crate::context::PluginContext;

/// The primary trait that all plugins must implement.
///
/// Object-safe — the core stores plugins as `Box<dyn FileForgePlugin>`.
/// All lifecycle methods return `Result<(), PluginError>` and must not panic.
///
/// # Lifecycle
///
/// 1. `initialize` — called with the context; store it for later use
/// 2. `activate` — register capabilities, start background work
/// 3. `deactivate` — unregister capabilities, stop background work
/// 4. `shutdown` — release all resources, final cleanup
///
/// # Thread Safety
///
/// The trait requires `Send + Sync` to allow plugins to be managed
/// from the registry's management thread.
pub trait FileForgePlugin: Send + Sync {
    /// Returns an immutable reference to the plugin's metadata.
    fn metadata(&self) -> &PluginMetadata;

    /// Returns the list of capabilities this plugin provides to the platform.
    ///
    /// Named `plugin_capabilities` to avoid collision with other trait methods.
    fn plugin_capabilities(&self) -> &[Capability];

    /// Initialize the plugin with the provided context.
    ///
    /// Plugins receive an `Arc<PluginContext>` and should store it for use
    /// throughout their lifetime. Called after dependencies are active.
    ///
    /// # Errors
    ///
    /// Returns `PluginError::InitializationFailed` if initialization cannot complete.
    fn initialize(&mut self, context: Arc<PluginContext>) -> Result<(), PluginError>;

    /// Activate the plugin — register capabilities, start background work.
    ///
    /// # Errors
    ///
    /// Returns `PluginError::ActivationFailed` if activation cannot complete.
    fn activate(&mut self) -> Result<(), PluginError>;

    /// Deactivate the plugin — unregister capabilities, stop background work.
    ///
    /// # Errors
    ///
    /// Returns `PluginError::DeactivationFailed` if deactivation encounters issues.
    fn deactivate(&mut self) -> Result<(), PluginError>;

    /// Shutdown the plugin — release all resources, final cleanup.
    ///
    /// # Errors
    ///
    /// Returns `PluginError::ShutdownFailed` if shutdown encounters issues.
    fn shutdown(&mut self) -> Result<(), PluginError>;

    /// Whether this plugin supports hot-reload.
    ///
    /// Defaults to `false`. Plugins that support hot-reload override this
    /// to return `true`, enabling the Active → Shutdown → Discovered cycle.
    fn supports_hot_reload(&self) -> bool {
        false
    }
}

// ─── Service Traits ─────────────────────────────────────────────────────────
// These traits are DEFINED here but IMPLEMENTED by downstream crates.

/// Command registration service.
///
/// Implemented by the command framework crate. Injected into `PluginContext`
/// via `PlatformServices`.
pub trait CommandRegistration: Send + Sync {
    /// Register a command for a plugin.
    fn register(&self, owner: &str, command: PluginCommand) -> Result<(), PluginError>;
    /// Unregister a specific command owned by a plugin.
    fn unregister(&self, owner: &str, command_id: &str) -> Result<(), PluginError>;
}

/// Scoped configuration access.
///
/// Implemented by the configuration system crate. Access is scoped to
/// the plugin's namespace: `[plugins.{plugin_name}]`.
pub trait PluginConfigAccess: Send + Sync {
    /// Read a configuration value for a plugin.
    fn get(&self, plugin_name: &str, key: &str) -> Result<Option<toml::Value>, PluginError>;
    /// Write a configuration value for a plugin.
    fn set(&self, plugin_name: &str, key: &str, value: toml::Value) -> Result<(), PluginError>;
}

/// VFS access for plugins.
///
/// Implemented by the virtual file system crate. Plugins access files
/// through VFS URIs rather than direct `std::fs` calls.
pub trait PluginVfsAccess: Send + Sync {
    /// Read file contents at the given URI.
    fn read(&self, uri: &str) -> Result<Vec<u8>, PluginError>;
    /// Write data to the given URI.
    fn write(&self, uri: &str, data: &[u8]) -> Result<(), PluginError>;
    /// Check whether a URI exists.
    fn exists(&self, uri: &str) -> Result<bool, PluginError>;
    /// List entries in a directory URI.
    fn list_directory(&self, uri: &str) -> Result<Vec<String>, PluginError>;
}

/// Event bus for plugins.
///
/// Implemented by the platform event bus. Provides subscription, unsubscription,
/// and emission of platform events.
pub trait PluginEventBus: Send + Sync {
    /// Subscribe to events of a given type.
    fn subscribe(&self, owner: &str, event_type: &str, handler: EventHandler) -> SubscriptionId;
    /// Unsubscribe from a previously subscribed event.
    fn unsubscribe(&self, id: SubscriptionId);
    /// Emit a platform event to all subscribers.
    fn emit(&self, event: PlatformEvent);
}

/// Capability registration service.
///
/// Delegates to the Capability_Registry. Stamps registrations with the
/// owning plugin's identity.
pub trait CapabilityRegistrar: Send + Sync {
    /// Register a capability for a plugin.
    fn register(&self, owner: &str, capability: Capability) -> Result<(), PluginError>;
    /// Unregister a specific capability owned by a plugin.
    fn unregister(&self, owner: &str, capability_id: &str) -> Result<(), PluginError>;
}

/// A command definition provided by a plugin for registration with the command framework.
pub struct PluginCommand {
    /// Unique command identifier (e.g., "my-plugin.do-something").
    pub id: String,
    /// Human-readable display name shown in the command palette.
    pub display_name: String,
    /// Category for grouping in command listings.
    pub category: String,
    /// Optional default keyboard shortcut.
    pub default_shortcut: Option<String>,
    /// The handler invoked when the command is executed.
    pub handler: Box<dyn Fn() -> Result<(), String> + Send + Sync>,
}

impl std::fmt::Debug for PluginCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginCommand")
            .field("id", &self.id)
            .field("display_name", &self.display_name)
            .field("category", &self.category)
            .field("default_shortcut", &self.default_shortcut)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;

    /// A minimal mock plugin for testing trait object-safety.
    struct MockPlugin {
        meta: PluginMetadata,
        caps: Vec<Capability>,
    }

    impl MockPlugin {
        fn new(name: &str) -> Self {
            Self {
                meta: PluginMetadata {
                    name: name.to_string(),
                    version: Version::new(1, 0, 0),
                    author: "Test".to_string(),
                    description: "Mock".to_string(),
                    dependencies: vec![],
                    required_api_version: Version::new(1, 0, 0),
                },
                caps: vec![],
            }
        }
    }

    impl FileForgePlugin for MockPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.meta
        }

        fn plugin_capabilities(&self) -> &[Capability] {
            &self.caps
        }

        fn initialize(&mut self, _context: Arc<PluginContext>) -> Result<(), PluginError> {
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

    #[test]
    fn file_forge_plugin_is_object_safe() {
        // Validates: Requirement 1.6
        let plugin: Box<dyn FileForgePlugin> = Box::new(MockPlugin::new("test"));
        assert_eq!(plugin.metadata().name, "test");
    }

    #[test]
    fn box_dyn_plugin_can_be_stored_in_vec() {
        // Validates: Requirement 1.6
        let mut plugins: Vec<Box<dyn FileForgePlugin>> = Vec::new();
        plugins.push(Box::new(MockPlugin::new("a")));
        plugins.push(Box::new(MockPlugin::new("b")));
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins[0].metadata().name, "a");
        assert_eq!(plugins[1].metadata().name, "b");
    }

    #[test]
    fn default_supports_hot_reload_is_false() {
        // Validates: Requirement 3.6
        let plugin = MockPlugin::new("test");
        assert!(!plugin.supports_hot_reload());
    }

    #[test]
    fn plugin_lifecycle_methods_return_result() {
        // Validates: Requirement 1.4
        let mut plugin = MockPlugin::new("test");
        // All methods should return Ok
        assert!(plugin.activate().is_ok());
        assert!(plugin.deactivate().is_ok());
        assert!(plugin.shutdown().is_ok());
    }

    #[test]
    fn plugin_capabilities_returns_empty_slice_by_default() {
        // Validates: Requirement 1.3
        let plugin = MockPlugin::new("test");
        assert!(plugin.plugin_capabilities().is_empty());
    }

    #[test]
    fn mock_service_traits_compile() {
        // Validates: Requirement 2.2 — service traits are definable
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn CommandRegistration>>();
        assert_send_sync::<Box<dyn PluginConfigAccess>>();
        assert_send_sync::<Box<dyn PluginVfsAccess>>();
        assert_send_sync::<Box<dyn PluginEventBus>>();
        assert_send_sync::<Box<dyn CapabilityRegistrar>>();
    }
}
