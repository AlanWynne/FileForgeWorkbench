//! Plugin Registry -- tracks plugin states, metadata, and instances.
//!
//! The central registry that manages plugin lifecycle, discovery,
//! loading, and unloading.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::capability::Capability;
use crate::capability_registry::CapabilityRegistry;
use crate::context::{PlatformServices, PluginContext};
use crate::error::PluginError;
use crate::lifecycle::PluginState;
use crate::metadata::PluginMetadata;
use crate::traits::FileForgePlugin;

/// Internal registry entry tracking a single plugin's runtime state.
pub(crate) struct PluginEntry {
    /// The plugin instance (None after Shutdown).
    pub instance: Option<Box<dyn FileForgePlugin>>,
    /// Current lifecycle state.
    pub state: PluginState,
    /// Plugin metadata (cached for post-shutdown queries).
    pub metadata: PluginMetadata,
    /// Capabilities currently registered by this plugin.
    pub registered_capabilities: Vec<Capability>,
    /// Context provided to this plugin.
    pub context: Option<Arc<PluginContext>>,
}

/// Result of attempting to load a single plugin.
#[derive(Debug)]
pub struct PluginLoadResult {
    /// Plugin name.
    pub name: String,
    /// Whether loading succeeded.
    pub success: bool,
    /// Error if loading failed.
    pub error: Option<PluginError>,
    /// Final state after the load attempt.
    pub state: PluginState,
}

/// The central plugin registry managing all plugin instances and their lifecycle.
///
/// Thread-safe via `RwLock`. Provides methods for discovery, loading,
/// unloading, and querying plugin state.
pub struct PluginRegistry {
    /// All known plugins indexed by name.
    pub(super) plugins: RwLock<HashMap<String, PluginEntry>>,
    /// Directory to scan for plugins.
    pub(super) plugin_directory: PathBuf,
    /// Platform services for creating plugin contexts.
    pub(super) services: Arc<PlatformServices>,
    /// Capability registry for managing plugin capabilities.
    pub(crate) capability_registry: Arc<CapabilityRegistry>,
}

impl PluginRegistry {
    /// Creates a new empty plugin registry.
    pub fn new(plugin_directory: PathBuf, services: PlatformServices) -> Self {
        let capability_registry = Arc::new(CapabilityRegistry::new());
        Self {
            plugins: RwLock::new(HashMap::new()),
            plugin_directory,
            services: Arc::new(services),
            capability_registry,
        }
    }

    /// Creates a new plugin registry with a shared capability registry.
    pub fn with_capability_registry(
        plugin_directory: PathBuf,
        services: PlatformServices,
        capability_registry: Arc<CapabilityRegistry>,
    ) -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            plugin_directory,
            services: Arc::new(services),
            capability_registry,
        }
    }

    /// Returns the plugin directory path.
    pub fn plugin_directory(&self) -> &PathBuf {
        &self.plugin_directory
    }

    /// Query the current state of a plugin by name.
    pub fn plugin_state(&self, name: &str) -> Option<PluginState> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name).map(|e| e.state)
    }

    /// Get metadata for a plugin by name (available even after shutdown).
    pub fn plugin_metadata(&self, name: &str) -> Option<PluginMetadata> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name).map(|e| e.metadata.clone())
    }

    /// List all registered plugin names with their current states.
    pub fn list_plugins(&self) -> Vec<(String, PluginState)> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .iter()
            .map(|(name, entry)| (name.clone(), entry.state))
            .collect()
    }

    /// Register a plugin instance directly (for testing or programmatic loading).
    pub fn register_plugin(&self, plugin: Box<dyn FileForgePlugin>) {
        let meta = plugin.metadata().clone();
        let name = meta.name.clone();
        let entry = PluginEntry {
            instance: Some(plugin),
            state: PluginState::Discovered,
            metadata: meta,
            registered_capabilities: Vec::new(),
            context: None,
        };
        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(name, entry);
    }

    /// Discover plugins by scanning the plugin directory.
    ///
    /// Each subdirectory containing a `plugin.toml` is treated as a plugin.
    /// Creates entries in the Discovered state.
    pub fn discover_plugins(&self) -> Result<Vec<String>, PluginError> {
        let mut discovered = Vec::new();

        if !self.plugin_directory.exists() {
            return Ok(discovered);
        }

        let entries = std::fs::read_dir(&self.plugin_directory).map_err(|e| {
            PluginError::InitializationFailed {
                plugin: "registry".to_string(),
                description: format!("failed to read plugin directory: {e}"),
            }
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.toml");
            if !manifest_path.exists() {
                continue;
            }

            match std::fs::read_to_string(&manifest_path) {
                Ok(content) => match crate::metadata::parse_manifest(&content) {
                    Ok(meta) => {
                        let name = meta.name.clone();
                        ff_logging::log(
                            ff_logging::LogLevel::Info,
                            "plugin_registry",
                            &format!("discovered plugin: {} v{}", name, meta.version),
                        );
                        let plugin_entry = PluginEntry {
                            instance: None,
                            state: PluginState::Discovered,
                            metadata: meta,
                            registered_capabilities: Vec::new(),
                            context: None,
                        };
                        let mut plugins = self.plugins.write().unwrap();
                        plugins.insert(name.clone(), plugin_entry);
                        discovered.push(name);
                    }
                    Err(e) => {
                        ff_logging::log(
                            ff_logging::LogLevel::Warn,
                            "plugin_registry",
                            &format!(
                                "skipping malformed manifest at {}: {e}",
                                manifest_path.display()
                            ),
                        );
                    }
                },
                Err(e) => {
                    ff_logging::log(
                        ff_logging::LogLevel::Warn,
                        "plugin_registry",
                        &format!("cannot read manifest at {}: {e}", manifest_path.display()),
                    );
                }
            }
        }

        Ok(discovered)
    }
}

mod lifecycle;
mod load;

/// Extracts a human-readable message from a panic payload.
pub(super) fn extract_panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

// Compile-time assertion that PluginRegistry is Send + Sync
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PluginRegistry>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Capability;
    use crate::context::PlatformServices;
    use crate::event::{EventHandler, PlatformEvent, SubscriptionId};
    use crate::traits::*;
    use crate::version::Version;
    use std::time::Duration;

    // === Mock Services ======================================================

    struct MockCommandService;
    impl CommandRegistration for MockCommandService {
        fn register(&self, _o: &str, _c: PluginCommand) -> Result<(), PluginError> {
            Ok(())
        }
        fn unregister(&self, _o: &str, _id: &str) -> Result<(), PluginError> {
            Ok(())
        }
    }

    struct MockConfigService;
    impl PluginConfigAccess for MockConfigService {
        fn get(&self, _p: &str, _k: &str) -> Result<Option<toml::Value>, PluginError> {
            Ok(None)
        }
        fn set(&self, _p: &str, _k: &str, _v: toml::Value) -> Result<(), PluginError> {
            Ok(())
        }
    }

    struct MockVfsService;
    impl PluginVfsAccess for MockVfsService {
        fn read(&self, _u: &str) -> Result<Vec<u8>, PluginError> {
            Ok(vec![])
        }
        fn write(&self, _u: &str, _d: &[u8]) -> Result<(), PluginError> {
            Ok(())
        }
        fn exists(&self, _u: &str) -> Result<bool, PluginError> {
            Ok(false)
        }
        fn list_directory(&self, _u: &str) -> Result<Vec<String>, PluginError> {
            Ok(vec![])
        }
    }

    struct MockEventBus;
    impl PluginEventBus for MockEventBus {
        fn subscribe(&self, _o: &str, _t: &str, _h: EventHandler) -> SubscriptionId {
            SubscriptionId::new(1)
        }
        fn unsubscribe(&self, _id: SubscriptionId) {}
        fn emit(&self, _event: PlatformEvent) {}
    }

    struct MockCapabilityRegistrar;
    impl CapabilityRegistrar for MockCapabilityRegistrar {
        fn register(&self, _o: &str, _c: Capability) -> Result<(), PluginError> {
            Ok(())
        }
        fn unregister(&self, _o: &str, _id: &str) -> Result<(), PluginError> {
            Ok(())
        }
    }

    fn make_services() -> PlatformServices {
        PlatformServices {
            command_service: Arc::new(MockCommandService),
            config_service: Arc::new(MockConfigService),
            vfs_service: Arc::new(MockVfsService),
            event_service: Arc::new(MockEventBus),
            capability_service: Arc::new(MockCapabilityRegistrar),
        }
    }

    // === Mock Plugin ========================================================

    struct TestPlugin {
        meta: PluginMetadata,
        caps: Vec<Capability>,
        activated: bool,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                meta: PluginMetadata {
                    name: name.to_string(),
                    version: Version::new(1, 0, 0),
                    author: "Test".to_string(),
                    description: "".to_string(),
                    dependencies: vec![],
                    required_api_version: Version::new(1, 0, 0),
                },
                caps: vec![],
                activated: false,
            }
        }
    }

    impl FileForgePlugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.meta
        }
        fn plugin_capabilities(&self) -> &[Capability] {
            &self.caps
        }
        fn initialize(&mut self, _ctx: Arc<PluginContext>) -> Result<(), PluginError> {
            Ok(())
        }
        fn activate(&mut self) -> Result<(), PluginError> {
            self.activated = true;
            Ok(())
        }
        fn deactivate(&mut self) -> Result<(), PluginError> {
            self.activated = false;
            Ok(())
        }
        fn shutdown(&mut self) -> Result<(), PluginError> {
            Ok(())
        }
    }

    fn make_registry() -> PluginRegistry {
        let dir = std::env::temp_dir().join("ff-plugin-test-registry");
        let services = make_services();
        PluginRegistry::new(dir, services)
    }

    // === Tests ==============================================================

    #[test]
    fn empty_registry_has_no_plugins() {
        // Validates: Requirement 5.7
        let reg = make_registry();
        assert!(reg.list_plugins().is_empty());
    }

    #[test]
    fn register_and_query_plugin_state() {
        // Validates: Requirement 5.7
        let reg = make_registry();
        reg.register_plugin(Box::new(TestPlugin::new("alpha")));
        assert_eq!(reg.plugin_state("alpha"), Some(PluginState::Discovered));
    }

    #[test]
    fn load_plugin_transitions_to_active() {
        // Validates: Requirement 3.2
        let reg = make_registry();
        reg.register_plugin(Box::new(TestPlugin::new("beta")));
        let result = reg.load_plugin("beta");
        assert!(result.is_ok());
        assert_eq!(reg.plugin_state("beta"), Some(PluginState::Active));
    }

    #[test]
    fn load_nonexistent_plugin_returns_error() {
        // Validates: Requirement 3.2
        let reg = make_registry();
        let result = reg.load_plugin("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn unload_plugin_transitions_to_shutdown() {
        // Validates: Requirement 5.2
        let reg = make_registry();
        reg.register_plugin(Box::new(TestPlugin::new("gamma")));
        reg.load_plugin("gamma").unwrap();
        reg.unload_plugin("gamma").unwrap();
        assert_eq!(reg.plugin_state("gamma"), Some(PluginState::Shutdown));
    }

    #[test]
    fn shutdown_all_shuts_down_all_active_plugins() {
        // Validates: Requirement 5.5
        let reg = make_registry();
        reg.register_plugin(Box::new(TestPlugin::new("p1")));
        reg.register_plugin(Box::new(TestPlugin::new("p2")));
        reg.load_plugin("p1").unwrap();
        reg.load_plugin("p2").unwrap();
        reg.shutdown_all(Duration::from_secs(5));
        assert_eq!(reg.plugin_state("p1"), Some(PluginState::Shutdown));
        assert_eq!(reg.plugin_state("p2"), Some(PluginState::Shutdown));
    }

    #[test]
    fn panicking_plugin_does_not_crash_registry() {
        // Validates: Requirement 5.3
        struct PanickingPlugin {
            meta: PluginMetadata,
        }
        impl PanickingPlugin {
            fn new() -> Self {
                Self {
                    meta: PluginMetadata {
                        name: "panicker".to_string(),
                        version: Version::new(1, 0, 0),
                        author: "".to_string(),
                        description: "".to_string(),
                        dependencies: vec![],
                        required_api_version: Version::new(1, 0, 0),
                    },
                }
            }
        }
        impl FileForgePlugin for PanickingPlugin {
            fn metadata(&self) -> &PluginMetadata {
                &self.meta
            }
            fn plugin_capabilities(&self) -> &[Capability] {
                &[]
            }
            fn initialize(&mut self, _ctx: Arc<PluginContext>) -> Result<(), PluginError> {
                panic!("deliberate panic in initialize");
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

        let reg = make_registry();
        reg.register_plugin(Box::new(PanickingPlugin::new()));
        let result = reg.load_plugin("panicker");
        assert!(result.is_err());
        assert_eq!(reg.plugin_state("panicker"), Some(PluginState::Shutdown));
        // Registry is still operational
        reg.register_plugin(Box::new(TestPlugin::new("healthy")));
        assert!(reg.load_plugin("healthy").is_ok());
    }

    #[test]
    fn plugin_metadata_available_after_registration() {
        // Validates: Requirement 1.2
        let reg = make_registry();
        reg.register_plugin(Box::new(TestPlugin::new("meta-test")));
        let meta = reg.plugin_metadata("meta-test").unwrap();
        assert_eq!(meta.name, "meta-test");
    }
}
