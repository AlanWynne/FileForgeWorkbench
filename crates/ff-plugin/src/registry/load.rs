use std::sync::Arc;

use crate::context::PluginContext;
use crate::dependency::DependencyGraph;
use crate::error::PluginError;
use crate::lifecycle::PluginState;
use crate::metadata::PluginMetadata;
use crate::version::{is_api_compatible, PLUGIN_API_VERSION};

use super::{extract_panic_message, PluginLoadResult, PluginRegistry};

impl PluginRegistry {
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
    pub(super) fn load_single_plugin(&self, name: &str) -> PluginLoadResult {
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
                    // No instance -- just metadata from discovery, can't initialize
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
}
