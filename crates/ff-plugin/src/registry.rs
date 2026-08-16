//! Plugin Registry — tracks plugin states, metadata, and instances.
//!
//! The central registry that manages plugin lifecycle, discovery,
//! loading, and unloading.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::capability::Capability;
use crate::capability_registry::CapabilityRegistry;
use crate::context::{PlatformServices, PluginContext};
use crate::dependency::DependencyGraph;
use crate::error::PluginError;
use crate::lifecycle::PluginState;
use crate::metadata::PluginMetadata;
use crate::traits::FileForgePlugin;
use crate::version::{is_api_compatible, PLUGIN_API_VERSION};

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
    plugins: RwLock<HashMap<String, PluginEntry>>,
    /// Directory to scan for plugins.
    plugin_directory: PathBuf,
    /// Platform services for creating plugin contexts.
    services: Arc<PlatformServices>,
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

    /// Load all discovered plugins in dependency order.
    ///
    /// Constructs the dependency graph, performs topological sort,
    /// validates API versions, initializes, and activates each plugin.
    pub fn load_all(&self) -> Vec<PluginLoadResult> {
        let mut results = Vec::new();
        let plugins = self.plugins.read().unwrap();
        let metadata: Vec<&PluginMetadata> = plugins.values().map(|e| &e.metadata).collect();

        let graph = DependencyGraph::build_from_refs(&metadata);
        let load_order = match graph.topological_sort() {
            Ok(order) => order,
            Err(PluginError::CircularDependency { cycle }) => {
                ff_logging::log(
                    ff_logging::LogLevel::Error,
                    "plugin_registry",
                    &format!("circular dependency detected: {:?}", cycle),
                );
                for name in &cycle {
                    results.push(PluginLoadResult {
                        name: name.clone(),
                        success: false,
                        error: Some(PluginError::CircularDependency {
                            cycle: cycle.clone(),
                        }),
                        state: PluginState::Shutdown,
                    });
                }
                // Return results for non-cyclic plugins
                let non_cyclic: Vec<String> = plugins
                    .keys()
                    .filter(|k| !cycle.contains(k))
                    .cloned()
                    .collect();
                drop(plugins);
                for name in non_cyclic {
                    let result = self.load_single_plugin(&name);
                    results.push(result);
                }
                return results;
            }
            Err(e) => {
                results.push(PluginLoadResult {
                    name: "unknown".to_string(),
                    success: false,
                    error: Some(e),
                    state: PluginState::Shutdown,
                });
                return results;
            }
        };
        drop(plugins);

        for name in load_order {
            let result = self.load_single_plugin(&name);
            results.push(result);
        }

        results
    }

    /// Load a single plugin by name (dependencies must already be active).
    pub fn load_plugin(&self, name: &str) -> Result<(), PluginError> {
        let result = self.load_single_plugin(name);
        if result.success {
            Ok(())
        } else {
            Err(result.error.unwrap_or(PluginError::PluginNotFound {
                name: name.to_string(),
            }))
        }
    }

    /// Internal: load, initialize, and activate a single plugin.
    fn load_single_plugin(&self, name: &str) -> PluginLoadResult {
        // Check API version compatibility
        {
            let plugins = self.plugins.read().unwrap();
            if let Some(entry) = plugins.get(name) {
                if !is_api_compatible(&entry.metadata.required_api_version, &PLUGIN_API_VERSION) {
                    let err = PluginError::IncompatibleApiVersion {
                        plugin: name.to_string(),
                        required: entry.metadata.required_api_version.clone(),
                        available: PLUGIN_API_VERSION,
                    };
                    ff_logging::log(
                        ff_logging::LogLevel::Error,
                        "plugin_registry",
                        &err.to_string(),
                    );
                    return PluginLoadResult {
                        name: name.to_string(),
                        success: false,
                        error: Some(err),
                        state: PluginState::Shutdown,
                    };
                }
            } else {
                return PluginLoadResult {
                    name: name.to_string(),
                    success: false,
                    error: Some(PluginError::PluginNotFound {
                        name: name.to_string(),
                    }),
                    state: PluginState::Shutdown,
                };
            }
        }

        // Transition to Loaded
        {
            let mut plugins = self.plugins.write().unwrap();
            if let Some(entry) = plugins.get_mut(name) {
                entry.state = PluginState::Loaded;
            }
        }

        // Create context and initialize
        let context = Arc::new(PluginContext::new(name, &self.services));

        // Initialize (with panic catching)
        let init_result = {
            let mut plugins = self.plugins.write().unwrap();
            if let Some(entry) = plugins.get_mut(name) {
                if let Some(ref mut instance) = entry.instance {
                    let ctx = Arc::clone(&context);
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        instance.initialize(ctx)
                    }));
                    match result {
                        Ok(Ok(())) => {
                            entry.state = PluginState::Initialized;
                            entry.context = Some(context.clone());
                            Ok(())
                        }
                        Ok(Err(e)) => {
                            entry.state = PluginState::Shutdown;
                            Err(e)
                        }
                        Err(panic_payload) => {
                            entry.state = PluginState::Shutdown;
                            let msg = extract_panic_message(&panic_payload);
                            Err(PluginError::Panicked {
                                plugin: name.to_string(),
                                phase: "initialize".to_string(),
                                message: msg,
                            })
                        }
                    }
                } else {
                    // No instance — just metadata from discovery, can't initialize
                    entry.state = PluginState::Initialized;
                    entry.context = Some(context.clone());
                    Ok(())
                }
            } else {
                Err(PluginError::PluginNotFound {
                    name: name.to_string(),
                })
            }
        };

        if let Err(e) = init_result {
            ff_logging::log(
                ff_logging::LogLevel::Warn,
                "plugin_registry",
                &format!("[plugin:{name}] initialization failed: {e}"),
            );
            return PluginLoadResult {
                name: name.to_string(),
                success: false,
                error: Some(e),
                state: PluginState::Shutdown,
            };
        }

        // Activate (with panic catching)
        let activate_result = {
            let mut plugins = self.plugins.write().unwrap();
            if let Some(entry) = plugins.get_mut(name) {
                if let Some(ref mut instance) = entry.instance {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        instance.activate()
                    }));
                    match result {
                        Ok(Ok(())) => {
                            entry.state = PluginState::Active;
                            Ok(())
                        }
                        Ok(Err(e)) => {
                            entry.state = PluginState::Shutdown;
                            Err(e)
                        }
                        Err(panic_payload) => {
                            entry.state = PluginState::Shutdown;
                            let msg = extract_panic_message(&panic_payload);
                            Err(PluginError::Panicked {
                                plugin: name.to_string(),
                                phase: "activate".to_string(),
                                message: msg,
                            })
                        }
                    }
                } else {
                    entry.state = PluginState::Active;
                    Ok(())
                }
            } else {
                Err(PluginError::PluginNotFound {
                    name: name.to_string(),
                })
            }
        };

        match activate_result {
            Ok(()) => PluginLoadResult {
                name: name.to_string(),
                success: true,
                error: None,
                state: PluginState::Active,
            },
            Err(e) => {
                ff_logging::log(
                    ff_logging::LogLevel::Warn,
                    "plugin_registry",
                    &format!("[plugin:{name}] activation failed: {e}"),
                );
                PluginLoadResult {
                    name: name.to_string(),
                    success: false,
                    error: Some(e),
                    state: PluginState::Shutdown,
                }
            }
        }
    }

    /// Deactivate and unload a single plugin.
    ///
    /// Plugins that depend on it will be deactivated first (reverse order).
    pub fn unload_plugin(&self, name: &str) -> Result<(), PluginError> {
        // First deactivate dependents
        let dependents = {
            let plugins = self.plugins.read().unwrap();
            let metadata: Vec<&PluginMetadata> = plugins.values().map(|e| &e.metadata).collect();
            let graph = DependencyGraph::build_from_refs(&metadata);
            graph.dependents_of(name)
        };

        for dependent in &dependents {
            let state = self.plugin_state(dependent);
            if state == Some(PluginState::Active) {
                self.deactivate_plugin(dependent)?;
            }
        }

        self.deactivate_plugin(name)?;
        self.shutdown_plugin(name)?;
        Ok(())
    }

    /// Deactivate a single plugin (transition Active → Deactivating → Shutdown).
    fn deactivate_plugin(&self, name: &str) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(entry) = plugins.get_mut(name) {
            if entry.state != PluginState::Active {
                return Ok(()); // Already deactivated
            }

            entry.state = PluginState::Deactivating;

            if let Some(ref mut instance) = entry.instance {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    instance.deactivate()
                }));
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        ff_logging::log(
                            ff_logging::LogLevel::Warn,
                            "plugin_registry",
                            &format!("[plugin:{name}] deactivation error: {e}"),
                        );
                    }
                    Err(panic_payload) => {
                        let msg = extract_panic_message(&panic_payload);
                        ff_logging::log(
                            ff_logging::LogLevel::Error,
                            "plugin_registry",
                            &format!("[plugin:{name}] panicked during deactivate: {msg}"),
                        );
                    }
                }
            }

            // Remove capabilities
            self.capability_registry.unregister_all(name);
            entry.registered_capabilities.clear();
            entry.state = PluginState::Shutdown;
            Ok(())
        } else {
            Err(PluginError::PluginNotFound {
                name: name.to_string(),
            })
        }
    }

    /// Shutdown a single plugin.
    fn shutdown_plugin(&self, name: &str) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(entry) = plugins.get_mut(name) {
            if entry.state == PluginState::Shutdown {
                // Already shut down, just cleanup references
                if let Some(ref mut instance) = entry.instance {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        instance.shutdown()
                    }));
                }
                entry.instance = None;
                entry.context = None;
                return Ok(());
            }

            if let Some(ref mut instance) = entry.instance {
                let _ =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| instance.shutdown()));
            }
            entry.instance = None;
            entry.context = None;
            entry.state = PluginState::Shutdown;
            Ok(())
        } else {
            Err(PluginError::PluginNotFound {
                name: name.to_string(),
            })
        }
    }

    /// Shut down all plugins in reverse dependency order.
    ///
    /// Waits up to `timeout` for all plugins to complete shutdown.
    /// After timeout, forcibly drops remaining plugin instances.
    pub fn shutdown_all(&self, timeout: Duration) {
        let start = Instant::now();

        // Compute reverse dependency order
        let shutdown_order = {
            let plugins = self.plugins.read().unwrap();
            let metadata: Vec<&PluginMetadata> = plugins.values().map(|e| &e.metadata).collect();
            let graph = DependencyGraph::build_from_refs(&metadata);
            match graph.topological_sort() {
                Ok(order) => {
                    let mut reversed = order;
                    reversed.reverse();
                    reversed
                }
                Err(_) => {
                    // If we can't sort, just use arbitrary order
                    plugins.keys().cloned().collect()
                }
            }
        };

        let mut successful = 0;
        let mut timed_out = 0;
        let mut panicked = 0;

        for name in &shutdown_order {
            if start.elapsed() >= timeout {
                timed_out += 1;
                // Forcibly drop remaining
                let mut plugins = self.plugins.write().unwrap();
                if let Some(entry) = plugins.get_mut(name) {
                    entry.instance = None;
                    entry.context = None;
                    entry.state = PluginState::Shutdown;
                    self.capability_registry.unregister_all(name);
                }
                continue;
            }

            let state = self.plugin_state(name);
            if state != Some(PluginState::Active) && state != Some(PluginState::Initialized) {
                continue;
            }

            let mut plugins = self.plugins.write().unwrap();
            if let Some(entry) = plugins.get_mut(name) {
                if entry.state == PluginState::Active {
                    entry.state = PluginState::Deactivating;
                    if let Some(ref mut instance) = entry.instance {
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            instance.deactivate()
                        }));
                        if result.is_err() {
                            panicked += 1;
                        }
                    }
                    self.capability_registry.unregister_all(name);
                }

                if let Some(ref mut instance) = entry.instance {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        instance.shutdown()
                    }));
                    if result.is_err() {
                        panicked += 1;
                    } else {
                        successful += 1;
                    }
                } else {
                    successful += 1;
                }

                entry.instance = None;
                entry.context = None;
                entry.state = PluginState::Shutdown;
            }
        }

        ff_logging::log(
            ff_logging::LogLevel::Info,
            "plugin_registry",
            &format!(
                "shutdown complete: {successful} successful, {timed_out} timed out, {panicked} panicked"
            ),
        );
    }

    /// Attempt hot-reload of a plugin that supports it.
    ///
    /// Cycles: Active → Deactivating → Shutdown → Discovered → Loaded → Initialized → Active
    pub fn hot_reload(&self, name: &str) -> Result<(), PluginError> {
        // Check if plugin supports hot-reload
        {
            let plugins = self.plugins.read().unwrap();
            if let Some(entry) = plugins.get(name) {
                if let Some(ref instance) = entry.instance {
                    if !instance.supports_hot_reload() {
                        return Err(PluginError::ActivationFailed {
                            plugin: name.to_string(),
                            description: "plugin does not support hot-reload".to_string(),
                        });
                    }
                }
            } else {
                return Err(PluginError::PluginNotFound {
                    name: name.to_string(),
                });
            }
        }

        // Deactivate and shutdown
        self.deactivate_plugin(name)?;

        // Transition back to Discovered for re-loading
        {
            let mut plugins = self.plugins.write().unwrap();
            if let Some(entry) = plugins.get_mut(name) {
                entry.state = PluginState::Discovered;
                entry.instance = None;
                entry.context = None;
            }
        }

        // Re-load
        let result = self.load_single_plugin(name);
        if result.success {
            Ok(())
        } else {
            Err(result.error.unwrap_or(PluginError::ActivationFailed {
                plugin: name.to_string(),
                description: "hot-reload failed".to_string(),
            }))
        }
    }
}

/// Extracts a human-readable message from a panic payload.
fn extract_panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
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

    // ─── Mock Services ──────────────────────────────────────────────────────

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

    // ─── Mock Plugin ────────────────────────────────────────────────────────

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

    // ─── Tests ──────────────────────────────────────────────────────────────

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
