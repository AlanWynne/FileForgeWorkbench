use std::time::{Duration, Instant};

use crate::dependency::DependencyGraph;
use crate::error::PluginError;
use crate::lifecycle::PluginState;
use crate::metadata::PluginMetadata;

use super::{extract_panic_message, PluginRegistry};

impl PluginRegistry {
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

    /// Deactivate a single plugin (transition Active -> Deactivating -> Shutdown).
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
    /// Cycles: Active -> Deactivating -> Shutdown -> Discovered -> Loaded -> Initialized -> Active
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
