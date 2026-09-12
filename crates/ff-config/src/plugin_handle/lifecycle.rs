use std::sync::Arc;

use crate::callback::{CallbackHandle, CallbackRegistry};
use crate::error::{ConfigError, ValueType};
use crate::namespace::{is_reserved_namespace, plugin_namespace_prefix, validate_plugin_name};
use crate::schema::{Constraints, SchemaEntry, SchemaRegistry};
use crate::store::EffectiveStore;
use crate::value::ConfigValue;

use super::PluginConfigHandle;

/// Create a validated plugin configuration handle.
///
/// Validates the plugin name against naming rules and checks that it
/// is not a reserved core namespace.
///
/// # Errors
///
/// Returns `ConfigError::InvalidPluginName` if the name fails validation.
/// Returns `ConfigError::ReservedNamespace` if the name matches a reserved namespace.
pub fn create_plugin_config_handle<'a>(
    store: &'a EffectiveStore,
    schema: &'a SchemaRegistry,
    plugin_name: &str,
) -> Result<PluginConfigHandle<'a>, ConfigError> {
    validate_plugin_name(plugin_name)?;

    if is_reserved_namespace(plugin_name) {
        return Err(ConfigError::ReservedNamespace {
            plugin: plugin_name.to_string(),
            namespace: plugin_name.to_string(),
        });
    }

    Ok(PluginConfigHandle::new(
        plugin_name.to_string(),
        store,
        schema,
        None,
    ))
}

/// Create a validated plugin configuration handle with callback registry access.
///
/// Same as `create_plugin_config_handle` but also provides access to the
/// callback registry, enabling the plugin to register reload callbacks via
/// [`PluginConfigHandle::on_reload`].
///
/// # Errors
///
/// Returns `ConfigError::InvalidPluginName` if the name fails validation.
/// Returns `ConfigError::ReservedNamespace` if the name matches a reserved namespace.
pub fn create_plugin_config_handle_with_callbacks<'a>(
    store: &'a EffectiveStore,
    schema: &'a SchemaRegistry,
    callbacks: &'a Arc<CallbackRegistry>,
    plugin_name: &str,
) -> Result<PluginConfigHandle<'a>, ConfigError> {
    validate_plugin_name(plugin_name)?;

    if is_reserved_namespace(plugin_name) {
        return Err(ConfigError::ReservedNamespace {
            plugin: plugin_name.to_string(),
            namespace: plugin_name.to_string(),
        });
    }

    Ok(PluginConfigHandle::new(
        plugin_name.to_string(),
        store,
        schema,
        Some(callbacks),
    ))
}

/// A default value declared by a plugin for one of its configuration keys.
///
/// Plugins declare their defaults in their manifest. During plugin initialization,
/// these defaults are registered as the Defaults layer for the plugin's namespace.
#[derive(Debug, Clone)]
pub struct PluginDefault {
    /// The relative key within the plugin's namespace (e.g., `"max_rows"`).
    pub key: String,
    /// The expected value type for this key.
    pub value_type: ValueType,
    /// The default value applied when no layer provides this key.
    pub default: ConfigValue,
    /// Human-readable description of the setting's purpose.
    pub description: String,
    /// Optional validation constraints.
    pub constraints: Option<Constraints>,
}

/// Register plugin default configuration values in the schema registry.
///
/// Each default entry's key is auto-prefixed with the plugin's namespace
/// (e.g., `"max_rows"` -> `"plugins.sql-viewer.max_rows"`) and registered
/// as a `SchemaEntry`. This makes the defaults available through the
/// schema's default fallback mechanism.
///
/// # Arguments
///
/// * `schema` -- The schema registry to register defaults in.
/// * `plugin_name` -- The plugin's registered name (must be pre-validated).
/// * `defaults` -- The list of default declarations from the plugin's manifest.
///
/// # Errors
///
/// Returns `ConfigError::SchemaConflict` if a key is already registered
/// with a different type. Otherwise returns `Ok(())`.
pub fn register_plugin_defaults(
    schema: &mut SchemaRegistry,
    plugin_name: &str,
    defaults: Vec<PluginDefault>,
) -> Result<(), ConfigError> {
    let prefix = plugin_namespace_prefix(plugin_name);

    for plugin_default in defaults {
        let full_key = format!("{}{}", prefix, plugin_default.key);
        let entry = SchemaEntry {
            key: full_key,
            value_type: plugin_default.value_type,
            default: plugin_default.default,
            description: plugin_default.description,
            constraints: plugin_default.constraints,
        };
        schema.register(entry)?;
    }

    Ok(())
}

/// Unload a plugin from the configuration system.
///
/// Performs the following cleanup:
/// 1. Removes all schema entries with keys prefixed by `plugins.{plugin_name}.`
/// 2. Deregisters all callback handles provided
///
/// Previously persisted configuration values are NOT removed from config files
/// -- they are retained on disk but no longer actively served.
///
/// # Arguments
///
/// * `plugin_name` -- The plugin's registered name.
/// * `schema` -- The schema registry to deregister entries from.
/// * `callbacks` -- The callback registry to deregister callbacks from.
/// * `handles` -- The callback handles to deregister.
///
/// # Returns
///
/// The number of schema entries that were removed.
pub fn unload_plugin(
    plugin_name: &str,
    schema: &mut SchemaRegistry,
    callbacks: &CallbackRegistry,
    handles: Vec<CallbackHandle>,
) -> usize {
    let prefix = plugin_namespace_prefix(plugin_name);

    // Deregister all callback handles
    for handle in handles {
        callbacks.remove_callback(handle);
    }

    // Remove all schema entries for the plugin namespace
    schema.deregister(&prefix)
}
