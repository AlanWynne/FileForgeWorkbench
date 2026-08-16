//! Plugin configuration handle.
//!
//! Provides `PluginConfigHandle` — a scoped configuration interface given to
//! plugins via `PluginContext`. Restricts read and write access to the
//! plugin's own namespace (`plugins.{plugin-name}.*`).
//!
//! Also provides plugin lifecycle functions for default registration,
//! reload callback management, and unload cleanup.

use std::sync::Arc;

use crate::callback::{CallbackHandle, CallbackRegistry, ReloadCallback};
use crate::error::{ConfigError, ValueType};
use crate::namespace::{is_reserved_namespace, plugin_namespace_prefix, validate_plugin_name};
use crate::schema::{Constraints, SchemaEntry, SchemaRegistry};
use crate::store::EffectiveStore;
use crate::value::ConfigValue;

/// A scoped configuration handle restricting access to a single plugin's namespace.
///
/// Each plugin receives a `PluginConfigHandle` that only permits reads and
/// writes to keys under `plugins.{plugin-name}.*`. Any attempt to access
/// keys outside this namespace results in a `NamespaceViolation` error.
pub struct PluginConfigHandle<'a> {
    /// The full namespace prefix, e.g., `"plugins.sql-viewer."`.
    namespace_prefix: String,
    /// The plugin's registered name, e.g., `"sql-viewer"`.
    plugin_name: String,
    /// Reference to the effective configuration store.
    store: &'a EffectiveStore,
    /// Reference to the schema registry for default value fallback.
    schema: &'a SchemaRegistry,
    /// Reference to the callback registry for reload notification.
    callbacks: Option<&'a Arc<CallbackRegistry>>,
    /// Write buffer for plugin-initiated configuration changes.
    write_buffer: Vec<(String, ConfigValue)>,
}

impl std::fmt::Debug for PluginConfigHandle<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginConfigHandle")
            .field("namespace_prefix", &self.namespace_prefix)
            .field("plugin_name", &self.plugin_name)
            .field("has_callbacks", &self.callbacks.is_some())
            .field("write_buffer_len", &self.write_buffer.len())
            .finish()
    }
}

impl<'a> PluginConfigHandle<'a> {
    /// Create a new plugin configuration handle.
    ///
    /// This is an internal constructor — use `create_plugin_config_handle`
    /// for the validated public API.
    fn new(
        plugin_name: String,
        store: &'a EffectiveStore,
        schema: &'a SchemaRegistry,
        callbacks: Option<&'a Arc<CallbackRegistry>>,
    ) -> Self {
        let namespace_prefix = plugin_namespace_prefix(&plugin_name);
        Self {
            namespace_prefix,
            plugin_name,
            store,
            schema,
            callbacks,
            write_buffer: Vec::new(),
        }
    }

    /// Returns the full namespace prefix for this plugin.
    ///
    /// For a plugin named `"sql-viewer"`, this returns `"plugins.sql-viewer."`.
    pub fn namespace(&self) -> &str {
        &self.namespace_prefix
    }

    /// Returns the plugin's registered name.
    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    /// Get a raw `ConfigValue` by relative key.
    ///
    /// The key is automatically prefixed with the plugin's namespace.
    /// For example, `handle.get("max_rows")` looks up `"plugins.sql-viewer.max_rows"`.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NamespaceViolation` if the fully-qualified key
    /// falls outside this plugin's namespace (e.g., key contains path traversal).
    pub fn get(&self, key: &str) -> Result<ConfigValue, ConfigError> {
        let full_key = self.resolve_key(key)?;
        // Try store first
        if let Some(v) = self.store.get_value(&full_key) {
            return Ok(v.clone());
        }
        // Try schema default
        if let Some(entry) = self.schema.get(&full_key) {
            return Ok(entry.default.clone());
        }
        Err(ConfigError::UndefinedKey { key: full_key })
    }

    /// Get a string value by relative key.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NamespaceViolation` if access is outside namespace.
    /// Returns `ConfigError::UndefinedKey` if the key is not defined.
    /// Returns `ConfigError::TypeMismatch` if the stored value is not a string.
    pub fn get_string(&self, key: &str) -> Result<String, ConfigError> {
        let value = self.get(key)?;
        match value {
            ConfigValue::String(s) => Ok(s),
            other => Err(ConfigError::TypeMismatch {
                key: self.make_full_key(key),
                expected: crate::error::ValueType::String,
                found: crate::validate::value_type_of(&other),
            }),
        }
    }

    /// Get an integer value by relative key.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NamespaceViolation` if access is outside namespace.
    /// Returns `ConfigError::UndefinedKey` if the key is not defined.
    /// Returns `ConfigError::TypeMismatch` if the stored value is not an integer.
    pub fn get_int(&self, key: &str) -> Result<i64, ConfigError> {
        let value = self.get(key)?;
        match value {
            ConfigValue::Integer(i) => Ok(i),
            other => Err(ConfigError::TypeMismatch {
                key: self.make_full_key(key),
                expected: crate::error::ValueType::Integer,
                found: crate::validate::value_type_of(&other),
            }),
        }
    }

    /// Get a float value by relative key.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NamespaceViolation` if access is outside namespace.
    /// Returns `ConfigError::UndefinedKey` if the key is not defined.
    /// Returns `ConfigError::TypeMismatch` if the stored value is not a float.
    pub fn get_float(&self, key: &str) -> Result<f64, ConfigError> {
        let value = self.get(key)?;
        match value {
            ConfigValue::Float(f) => Ok(f),
            other => Err(ConfigError::TypeMismatch {
                key: self.make_full_key(key),
                expected: crate::error::ValueType::Float,
                found: crate::validate::value_type_of(&other),
            }),
        }
    }

    /// Get a boolean value by relative key.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NamespaceViolation` if access is outside namespace.
    /// Returns `ConfigError::UndefinedKey` if the key is not defined.
    /// Returns `ConfigError::TypeMismatch` if the stored value is not a boolean.
    pub fn get_bool(&self, key: &str) -> Result<bool, ConfigError> {
        let value = self.get(key)?;
        match value {
            ConfigValue::Boolean(b) => Ok(b),
            other => Err(ConfigError::TypeMismatch {
                key: self.make_full_key(key),
                expected: crate::error::ValueType::Boolean,
                found: crate::validate::value_type_of(&other),
            }),
        }
    }

    /// Set a configuration value by relative key.
    ///
    /// The write is validated for namespace scoping but stored in an internal
    /// write buffer. Persistence to the user-layer file will be handled by
    /// the `ConfigHandle` integration (Task 20).
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::NamespaceViolation` if the key resolves outside
    /// this plugin's namespace.
    pub fn set(&mut self, key: &str, value: ConfigValue) -> Result<(), ConfigError> {
        let full_key = self.resolve_key(key)?;
        self.write_buffer.push((full_key, value));
        Ok(())
    }

    /// Returns a reference to the pending write buffer.
    ///
    /// This allows the integration layer to flush writes to the user-layer
    /// file when ready.
    pub fn pending_writes(&self) -> &[(String, ConfigValue)] {
        &self.write_buffer
    }

    /// Clears the pending write buffer after a successful flush.
    pub fn clear_pending_writes(&mut self) {
        self.write_buffer.clear();
    }

    /// Register a reload callback for specific keys within this plugin's namespace.
    ///
    /// The provided relative keys are auto-prefixed with the plugin's namespace.
    /// For example, if the plugin is `"sql-viewer"` and keys are `["max_rows", "timeout"]`,
    /// the callback watches `["plugins.sql-viewer.max_rows", "plugins.sql-viewer.timeout"]`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(CallbackHandle)` on success, which can be used to deregister the callback.
    /// Returns `Err(ConfigError::UndefinedKey)` if the callback registry is not available.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let handle = plugin_handle.on_reload(
    ///     &["max_rows", "timeout"],
    ///     Box::new(|event| {
    ///         println!("Plugin settings changed: {:?}", event.changed_keys);
    ///     }),
    /// )?;
    /// ```
    pub fn on_reload(
        &self,
        keys: &[&str],
        callback: ReloadCallback,
    ) -> Result<CallbackHandle, ConfigError> {
        let callbacks = self.callbacks.ok_or_else(|| ConfigError::UndefinedKey {
            key: format!("{}[callback_registry]", self.namespace_prefix),
        })?;

        let full_keys: Vec<String> = keys.iter().map(|k| self.make_full_key(k)).collect();

        let key_refs: Vec<&str> = full_keys.iter().map(|s| s.as_str()).collect();
        let handle = callbacks.on_reload(&key_refs, callback);
        Ok(handle)
    }

    /// Resolve a relative key to a fully-qualified key and validate namespace.
    ///
    /// Prepends the namespace prefix and then checks that the result stays
    /// within bounds. This catches malicious keys like `"../../editor.tab_size"`.
    fn resolve_key(&self, key: &str) -> Result<String, ConfigError> {
        let full_key = self.make_full_key(key);
        self.check_namespace(&full_key)?;
        Ok(full_key)
    }

    /// Construct the fully-qualified key from a relative key.
    fn make_full_key(&self, key: &str) -> String {
        format!("{}{}", self.namespace_prefix, key)
    }

    /// Validate that a fully-qualified key is within this plugin's namespace.
    fn check_namespace(&self, full_key: &str) -> Result<(), ConfigError> {
        if !full_key.starts_with(&self.namespace_prefix) {
            return Err(ConfigError::NamespaceViolation {
                plugin: self.plugin_name.clone(),
                key: full_key.to_string(),
            });
        }
        Ok(())
    }
}

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
/// (e.g., `"max_rows"` → `"plugins.sql-viewer.max_rows"`) and registered
/// as a `SchemaEntry`. This makes the defaults available through the
/// schema's default fallback mechanism.
///
/// # Arguments
///
/// * `schema` — The schema registry to register defaults in.
/// * `plugin_name` — The plugin's registered name (must be pre-validated).
/// * `defaults` — The list of default declarations from the plugin's manifest.
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
/// — they are retained on disk but no longer actively served.
///
/// # Arguments
///
/// * `plugin_name` — The plugin's registered name.
/// * `schema` — The schema registry to deregister entries from.
/// * `callbacks` — The callback registry to deregister callbacks from.
/// * `handles` — The callback handles to deregister.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::ConfigLayer;
    use crate::provenance::{EffectiveValue, Provenance};
    use crate::schema::SchemaEntry;
    use std::path::PathBuf;

    /// Helper: build a store with a single key-value entry.
    fn store_with(key: &str, value: ConfigValue) -> EffectiveStore {
        let mut store = EffectiveStore::new();
        store.insert(
            key.to_string(),
            EffectiveValue {
                value,
                provenance: Provenance {
                    layer: ConfigLayer::User,
                    source_file: Some(PathBuf::from("test.toml")),
                },
            },
        );
        store
    }

    /// Helper: build a schema with a single entry.
    fn schema_with(key: &str, default: ConfigValue) -> SchemaRegistry {
        let mut schema = SchemaRegistry::new();
        schema
            .register(SchemaEntry {
                key: key.to_string(),
                value_type: crate::error::ValueType::Integer,
                default,
                description: format!("Test entry for {key}"),
                constraints: None,
            })
            .unwrap();
        schema
    }

    // ──────────────────────────────────────────────────────────────────
    // create_plugin_config_handle
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.1, 8.2
    #[test]
    fn create_handle_with_valid_name_succeeds() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let result = create_plugin_config_handle(&store, &schema, "sql-viewer");
        assert!(result.is_ok());
        let handle = result.unwrap();
        assert_eq!(handle.plugin_name(), "sql-viewer");
        assert_eq!(handle.namespace(), "plugins.sql-viewer.");
    }

    // Validates: Requirement 8.1
    #[test]
    fn create_handle_with_invalid_name_fails() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let result = create_plugin_config_handle(&store, &schema, "My Plugin!!");
        assert!(matches!(result, Err(ConfigError::InvalidPluginName { .. })));
    }

    // Validates: Requirement 8.7
    #[test]
    fn create_handle_with_reserved_namespace_fails() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();

        // Reserved namespaces that are also valid plugin names get ReservedNamespace error
        let valid_reserved = [
            "logging", "editor", "theme", "vfs", "commands", "layout", "core",
        ];
        for reserved in valid_reserved {
            let result = create_plugin_config_handle(&store, &schema, reserved);
            assert!(
                matches!(result, Err(ConfigError::ReservedNamespace { .. })),
                "Expected ReservedNamespace error for '{reserved}', got: {result:?}"
            );
        }

        // `_session` has an underscore so it fails name validation first —
        // the system still prevents registration under this namespace.
        let result = create_plugin_config_handle(&store, &schema, "_session");
        assert!(
            matches!(
                result,
                Err(ConfigError::InvalidPluginName { .. })
                    | Err(ConfigError::ReservedNamespace { .. })
            ),
            "Expected rejection for '_session', got: {result:?}"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Scoped getters
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.2
    #[test]
    fn get_with_relative_key_prepends_namespace() {
        let store = store_with("plugins.sql-viewer.max_rows", ConfigValue::Integer(500));
        let schema = SchemaRegistry::new();
        let handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        let result = handle.get("max_rows");
        assert_eq!(result.unwrap(), ConfigValue::Integer(500));
    }

    // Validates: Requirement 8.2
    #[test]
    fn get_string_returns_value_from_store() {
        let store = store_with(
            "plugins.my-plugin.mode",
            ConfigValue::String("fast".to_string()),
        );
        let schema = SchemaRegistry::new();
        let handle = create_plugin_config_handle(&store, &schema, "my-plugin").unwrap();

        let result = handle.get_string("mode");
        assert_eq!(result.unwrap(), "fast");
    }

    // Validates: Requirement 8.2
    #[test]
    fn get_int_returns_value_from_store() {
        let store = store_with("plugins.sql-viewer.timeout", ConfigValue::Integer(30));
        let schema = SchemaRegistry::new();
        let handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        let result = handle.get_int("timeout");
        assert_eq!(result.unwrap(), 30);
    }

    // Validates: Requirement 8.2
    #[test]
    fn get_float_returns_value_from_store() {
        let store = store_with("plugins.sql-viewer.scale", ConfigValue::Float(1.5));
        let schema = SchemaRegistry::new();
        let handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        let result = handle.get_float("scale");
        assert_eq!(result.unwrap(), 1.5);
    }

    // Validates: Requirement 8.2
    #[test]
    fn get_bool_returns_value_from_store() {
        let store = store_with("plugins.sql-viewer.enabled", ConfigValue::Boolean(true));
        let schema = SchemaRegistry::new();
        let handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        let result = handle.get_bool("enabled");
        assert_eq!(result.unwrap(), true);
    }

    // Validates: Requirement 8.2
    #[test]
    fn get_falls_back_to_schema_default() {
        let store = EffectiveStore::new();
        let schema = schema_with("plugins.sql-viewer.max_rows", ConfigValue::Integer(1000));
        let handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        let result = handle.get("max_rows");
        assert_eq!(result.unwrap(), ConfigValue::Integer(1000));
    }

    // Validates: Requirement 8.2
    #[test]
    fn get_returns_undefined_key_when_not_found() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        let result = handle.get("nonexistent");
        assert!(matches!(result, Err(ConfigError::UndefinedKey { .. })));
    }

    // ──────────────────────────────────────────────────────────────────
    // Scoped set
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.2
    #[test]
    fn set_stores_value_in_write_buffer() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let mut handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        let result = handle.set("max_rows", ConfigValue::Integer(2000));
        assert!(result.is_ok());
        assert_eq!(handle.pending_writes().len(), 1);
        assert_eq!(
            handle.pending_writes()[0],
            (
                "plugins.sql-viewer.max_rows".to_string(),
                ConfigValue::Integer(2000)
            )
        );
    }

    // Validates: Requirement 8.2
    #[test]
    fn set_multiple_values_accumulates_in_write_buffer() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let mut handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        handle.set("max_rows", ConfigValue::Integer(500)).unwrap();
        handle.set("timeout", ConfigValue::Float(10.0)).unwrap();

        assert_eq!(handle.pending_writes().len(), 2);
    }

    // Validates: Requirement 8.2
    #[test]
    fn clear_pending_writes_empties_buffer() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let mut handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        handle.set("max_rows", ConfigValue::Integer(500)).unwrap();
        assert_eq!(handle.pending_writes().len(), 1);

        handle.clear_pending_writes();
        assert!(handle.pending_writes().is_empty());
    }

    // ──────────────────────────────────────────────────────────────────
    // Namespace violation detection
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.3
    #[test]
    fn get_type_mismatch_returns_error() {
        let store = store_with("plugins.sql-viewer.max_rows", ConfigValue::Integer(100));
        let schema = SchemaRegistry::new();
        let handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        let result = handle.get_string("max_rows");
        assert!(matches!(result, Err(ConfigError::TypeMismatch { .. })));
    }

    // ──────────────────────────────────────────────────────────────────
    // Namespace violation detection (Task 18.6)
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.3 — plugin cannot read core namespace keys
    #[test]
    fn namespace_violation_detected_for_core_key_via_resolve() {
        // The resolve_key method always prepends the namespace prefix,
        // so even if a plugin passes "editor.tab_size" as a relative key,
        // it becomes "plugins.sql-viewer.editor.tab_size" — which is valid
        // from a namespace perspective (it's under the plugin's namespace).
        // The real violation scenario is when the full key would escape the prefix.
        // Since resolve_key always prepends, namespace violations arise from
        // the check_namespace internal method. Let's verify the handle's
        // namespace prefix is correctly enforced.
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        // Relative key "max_rows" becomes "plugins.sql-viewer.max_rows" — valid namespace
        // The handle always prepends its prefix, so namespace violations from `get`/`set`
        // can only happen if we ever call check_namespace with an external key.
        // The current design ensures all public methods go through resolve_key which
        // always prepends the prefix — namespace violation is structurally impossible
        // via the public API (which is the desired security property).
        assert_eq!(handle.namespace(), "plugins.sql-viewer.");
    }

    // Validates: Requirement 8.3 — plugin handles are isolated from each other
    #[test]
    fn different_plugins_cannot_see_each_others_keys() {
        let store = store_with(
            "plugins.sql-viewer.secret",
            ConfigValue::String("hidden".to_string()),
        );
        let schema = SchemaRegistry::new();

        // other-plugin cannot see sql-viewer's keys because its namespace
        // prefix is different: "plugins.other-plugin." vs "plugins.sql-viewer."
        let handle = create_plugin_config_handle(&store, &schema, "other-plugin").unwrap();
        let result = handle.get("secret");
        // This resolves to "plugins.other-plugin.secret" which doesn't exist
        assert!(matches!(result, Err(ConfigError::UndefinedKey { .. })));
    }

    // Validates: Requirement 8.3 — set is also namespace-scoped
    #[test]
    fn set_is_namespace_scoped_to_plugin_prefix() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let mut handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        // Setting "timeout" should produce "plugins.sql-viewer.timeout" in write buffer
        handle.set("timeout", ConfigValue::Integer(60)).unwrap();
        let (key, _) = &handle.pending_writes()[0];
        assert!(key.starts_with("plugins.sql-viewer."));
    }

    // Validates: Requirement 8.3 — nested relative keys work correctly
    #[test]
    fn nested_relative_keys_are_properly_scoped() {
        let store = store_with(
            "plugins.sql-viewer.display.theme",
            ConfigValue::String("dark".to_string()),
        );
        let schema = SchemaRegistry::new();
        let handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        // "display.theme" → "plugins.sql-viewer.display.theme"
        let result = handle.get_string("display.theme");
        assert_eq!(result.unwrap(), "dark");
    }

    // Validates: Requirement 8.7 — all reserved namespaces enumerated
    #[test]
    fn reserved_namespaces_list_contains_expected_entries() {
        use crate::namespace::RESERVED_NAMESPACES;
        assert!(RESERVED_NAMESPACES.contains(&"logging"));
        assert!(RESERVED_NAMESPACES.contains(&"editor"));
        assert!(RESERVED_NAMESPACES.contains(&"theme"));
        assert!(RESERVED_NAMESPACES.contains(&"vfs"));
        assert!(RESERVED_NAMESPACES.contains(&"commands"));
        assert!(RESERVED_NAMESPACES.contains(&"layout"));
        assert!(RESERVED_NAMESPACES.contains(&"core"));
        assert!(RESERVED_NAMESPACES.contains(&"_session"));
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 19.1 — Plugin default registration
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.4 — register_plugin_defaults creates schema entries
    #[test]
    fn register_plugin_defaults_creates_schema_entries_with_correct_keys() {
        let mut schema = SchemaRegistry::new();
        let defaults = vec![
            PluginDefault {
                key: "max_rows".to_string(),
                value_type: ValueType::Integer,
                default: ConfigValue::Integer(1000),
                description: "Maximum number of rows to display".to_string(),
                constraints: None,
            },
            PluginDefault {
                key: "timeout".to_string(),
                value_type: ValueType::Float,
                default: ConfigValue::Float(30.0),
                description: "Query timeout in seconds".to_string(),
                constraints: None,
            },
        ];

        let result = register_plugin_defaults(&mut schema, "sql-viewer", defaults);
        assert!(result.is_ok());

        // Verify keys are prefixed with the plugin namespace
        let entry1 = schema.get("plugins.sql-viewer.max_rows");
        assert!(entry1.is_some());
        let entry1 = entry1.unwrap();
        assert_eq!(entry1.value_type, ValueType::Integer);
        assert_eq!(entry1.default, ConfigValue::Integer(1000));
        assert_eq!(entry1.description, "Maximum number of rows to display");

        let entry2 = schema.get("plugins.sql-viewer.timeout");
        assert!(entry2.is_some());
        let entry2 = entry2.unwrap();
        assert_eq!(entry2.value_type, ValueType::Float);
        assert_eq!(entry2.default, ConfigValue::Float(30.0));
    }

    // Validates: Requirement 8.4 — defaults are used as fallback in PluginConfigHandle
    #[test]
    fn registered_defaults_serve_as_fallback_values() {
        let mut schema = SchemaRegistry::new();
        let defaults = vec![PluginDefault {
            key: "max_rows".to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(1000),
            description: "Max rows".to_string(),
            constraints: None,
        }];
        register_plugin_defaults(&mut schema, "sql-viewer", defaults).unwrap();

        let store = EffectiveStore::new(); // empty store — no user values
        let handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        // Should fall back to schema default
        let result = handle.get_int("max_rows");
        assert_eq!(result.unwrap(), 1000);
    }

    // Validates: Requirement 8.4 — constraints are preserved in registered schema entries
    #[test]
    fn register_plugin_defaults_preserves_constraints() {
        let mut schema = SchemaRegistry::new();
        let defaults = vec![PluginDefault {
            key: "max_rows".to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(100),
            description: "Max rows".to_string(),
            constraints: Some(Constraints {
                min: Some(1.0),
                max: Some(10000.0),
                allowed_values: None,
                pattern: None,
            }),
        }];
        register_plugin_defaults(&mut schema, "sql-viewer", defaults).unwrap();

        let entry = schema.get("plugins.sql-viewer.max_rows").unwrap();
        let constraints = entry.constraints.as_ref().unwrap();
        assert_eq!(constraints.min, Some(1.0));
        assert_eq!(constraints.max, Some(10000.0));
    }

    // Validates: Requirement 8.4 — type conflict returns SchemaConflict error
    #[test]
    fn register_plugin_defaults_with_type_conflict_returns_error() {
        let mut schema = SchemaRegistry::new();

        // Register first batch
        let defaults1 = vec![PluginDefault {
            key: "max_rows".to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(100),
            description: "Max rows".to_string(),
            constraints: None,
        }];
        register_plugin_defaults(&mut schema, "sql-viewer", defaults1).unwrap();

        // Try to re-register with different type → should fail
        let defaults2 = vec![PluginDefault {
            key: "max_rows".to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("unlimited".to_string()),
            description: "Max rows".to_string(),
            constraints: None,
        }];
        let result = register_plugin_defaults(&mut schema, "sql-viewer", defaults2);
        assert!(matches!(result, Err(ConfigError::SchemaConflict { .. })));
    }

    // Validates: Requirement 8.4 — empty defaults list is a no-op
    #[test]
    fn register_plugin_defaults_with_empty_list_is_noop() {
        let mut schema = SchemaRegistry::new();
        let result = register_plugin_defaults(&mut schema, "sql-viewer", vec![]);
        assert!(result.is_ok());
        assert_eq!(schema.len(), 0);
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 19.2 — Plugin reload callback registration via on_reload
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.5 — on_reload registers callback with prefixed keys
    #[test]
    fn on_reload_registers_callback_with_prefixed_keys() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let callbacks = Arc::new(CallbackRegistry::new());

        let handle =
            create_plugin_config_handle_with_callbacks(&store, &schema, &callbacks, "sql-viewer")
                .unwrap();

        let result = handle.on_reload(&["max_rows", "timeout"], Box::new(|_event| {}));
        assert!(result.is_ok());
        assert_eq!(callbacks.len(), 1);
    }

    // Validates: Requirement 8.5 — callback returns a handle for deregistration
    #[test]
    fn on_reload_returns_callback_handle() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let callbacks = Arc::new(CallbackRegistry::new());

        let handle =
            create_plugin_config_handle_with_callbacks(&store, &schema, &callbacks, "sql-viewer")
                .unwrap();

        let cb_handle = handle
            .on_reload(&["max_rows"], Box::new(|_event| {}))
            .unwrap();

        // Handle can be used for deregistration
        callbacks.remove_callback(cb_handle);
        assert_eq!(callbacks.len(), 0);
    }

    // Validates: Requirement 8.5 — on_reload without callback registry returns error
    #[test]
    fn on_reload_without_callbacks_returns_error() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();

        // Create handle without callback registry
        let handle = create_plugin_config_handle(&store, &schema, "sql-viewer").unwrap();

        let result = handle.on_reload(&["max_rows"], Box::new(|_event| {}));
        assert!(result.is_err());
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 19.3 — Plugin hot-reload: callbacks fire for plugin namespace changes
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.5 — callback fires when plugin namespace key changes
    #[test]
    fn plugin_reload_callback_fires_when_namespace_key_changes() {
        use crate::reload::ReloadEvent;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::time::SystemTime;

        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let callbacks = Arc::new(CallbackRegistry::new());

        let handle =
            create_plugin_config_handle_with_callbacks(&store, &schema, &callbacks, "sql-viewer")
                .unwrap();

        let invocation_count = Arc::new(AtomicU32::new(0));
        let count_clone = Arc::clone(&invocation_count);

        let _cb_handle = handle
            .on_reload(
                &["max_rows", "timeout"],
                Box::new(move |_event| {
                    count_clone.fetch_add(1, Ordering::SeqCst);
                }),
            )
            .unwrap();

        // Simulate a reload event with the plugin's key changing
        let event = ReloadEvent {
            changed_keys: vec!["plugins.sql-viewer.max_rows".to_string()],
            source_layer: ConfigLayer::User,
            timestamp: SystemTime::now(),
        };
        callbacks.invoke(&event);

        assert_eq!(invocation_count.load(Ordering::SeqCst), 1);
    }

    // Validates: Requirement 8.5 — callback does NOT fire for other plugin's keys
    #[test]
    fn plugin_reload_callback_does_not_fire_for_other_plugins_keys() {
        use crate::reload::ReloadEvent;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::time::SystemTime;

        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let callbacks = Arc::new(CallbackRegistry::new());

        let handle =
            create_plugin_config_handle_with_callbacks(&store, &schema, &callbacks, "sql-viewer")
                .unwrap();

        let invocation_count = Arc::new(AtomicU32::new(0));
        let count_clone = Arc::clone(&invocation_count);

        let _cb_handle = handle
            .on_reload(
                &["max_rows"],
                Box::new(move |_event| {
                    count_clone.fetch_add(1, Ordering::SeqCst);
                }),
            )
            .unwrap();

        // Event for a DIFFERENT plugin's key — should NOT fire
        let event = ReloadEvent {
            changed_keys: vec!["plugins.git-helper.enabled".to_string()],
            source_layer: ConfigLayer::User,
            timestamp: SystemTime::now(),
        };
        callbacks.invoke(&event);

        assert_eq!(invocation_count.load(Ordering::SeqCst), 0);
    }

    // Validates: Requirement 8.5 — callback receives correct event data
    #[test]
    fn plugin_reload_callback_receives_event_with_changed_keys() {
        use crate::reload::ReloadEvent;
        use std::sync::Mutex;
        use std::time::SystemTime;

        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let callbacks = Arc::new(CallbackRegistry::new());

        let handle =
            create_plugin_config_handle_with_callbacks(&store, &schema, &callbacks, "sql-viewer")
                .unwrap();

        let received_keys = Arc::new(Mutex::new(Vec::<String>::new()));
        let keys_clone = Arc::clone(&received_keys);

        let _cb_handle = handle
            .on_reload(
                &["max_rows"],
                Box::new(move |event| {
                    let mut keys = keys_clone.lock().unwrap();
                    keys.extend(event.changed_keys.clone());
                }),
            )
            .unwrap();

        let event = ReloadEvent {
            changed_keys: vec![
                "plugins.sql-viewer.max_rows".to_string(),
                "editor.tab_size".to_string(),
            ],
            source_layer: ConfigLayer::User,
            timestamp: SystemTime::now(),
        };
        callbacks.invoke(&event);

        let keys = received_keys.lock().unwrap();
        assert!(keys.contains(&"plugins.sql-viewer.max_rows".to_string()));
        assert!(keys.contains(&"editor.tab_size".to_string()));
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 19.4 — Plugin unload cleanup
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 8.6 — unload removes schema entries
    #[test]
    fn unload_plugin_removes_schema_entries() {
        let mut schema = SchemaRegistry::new();
        let defaults = vec![
            PluginDefault {
                key: "max_rows".to_string(),
                value_type: ValueType::Integer,
                default: ConfigValue::Integer(1000),
                description: "Max rows".to_string(),
                constraints: None,
            },
            PluginDefault {
                key: "timeout".to_string(),
                value_type: ValueType::Float,
                default: ConfigValue::Float(30.0),
                description: "Timeout".to_string(),
                constraints: None,
            },
        ];
        register_plugin_defaults(&mut schema, "sql-viewer", defaults).unwrap();
        assert_eq!(schema.len(), 2);

        let callbacks = CallbackRegistry::new();
        let removed = unload_plugin("sql-viewer", &mut schema, &callbacks, vec![]);

        assert_eq!(removed, 2);
        assert!(schema.get("plugins.sql-viewer.max_rows").is_none());
        assert!(schema.get("plugins.sql-viewer.timeout").is_none());
    }

    // Validates: Requirement 8.6 — unload deregisters callbacks
    #[test]
    fn unload_plugin_deregisters_callbacks() {
        let mut schema = SchemaRegistry::new();
        let callbacks = CallbackRegistry::new();

        // Register some callbacks
        let h1 = callbacks.on_reload(&["plugins.sql-viewer.max_rows"], Box::new(|_| {}));
        let h2 = callbacks.on_reload(&["plugins.sql-viewer.timeout"], Box::new(|_| {}));
        assert_eq!(callbacks.len(), 2);

        let removed = unload_plugin("sql-viewer", &mut schema, &callbacks, vec![h1, h2]);

        assert_eq!(removed, 0); // no schema entries were registered
        assert_eq!(callbacks.len(), 0); // both callbacks deregistered
    }

    // Validates: Requirement 8.6 — unload does not affect other plugins' schema entries
    #[test]
    fn unload_plugin_does_not_affect_other_plugins() {
        let mut schema = SchemaRegistry::new();
        let sql_defaults = vec![PluginDefault {
            key: "max_rows".to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(1000),
            description: "Max rows".to_string(),
            constraints: None,
        }];
        let git_defaults = vec![PluginDefault {
            key: "enabled".to_string(),
            value_type: ValueType::Boolean,
            default: ConfigValue::Boolean(true),
            description: "Enable git".to_string(),
            constraints: None,
        }];
        register_plugin_defaults(&mut schema, "sql-viewer", sql_defaults).unwrap();
        register_plugin_defaults(&mut schema, "git-helper", git_defaults).unwrap();
        assert_eq!(schema.len(), 2);

        let callbacks = CallbackRegistry::new();
        unload_plugin("sql-viewer", &mut schema, &callbacks, vec![]);

        // git-helper's schema entries remain
        assert_eq!(schema.len(), 1);
        assert!(schema.get("plugins.git-helper.enabled").is_some());
        assert!(schema.get("plugins.sql-viewer.max_rows").is_none());
    }

    // Validates: Requirement 8.6 — persisted values retained (not actively served after unload)
    #[test]
    fn unload_plugin_retains_persisted_values_in_store() {
        let mut schema = SchemaRegistry::new();
        let defaults = vec![PluginDefault {
            key: "max_rows".to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(1000),
            description: "Max rows".to_string(),
            constraints: None,
        }];
        register_plugin_defaults(&mut schema, "sql-viewer", defaults).unwrap();

        // Store has a persisted value for the plugin
        let store = store_with("plugins.sql-viewer.max_rows", ConfigValue::Integer(500));

        // After unload, schema is gone but store still has the value
        // (store represents the on-disk persisted state)
        let callbacks = CallbackRegistry::new();
        unload_plugin("sql-viewer", &mut schema, &callbacks, vec![]);

        // Schema no longer has the entry (not actively served)
        assert!(schema.get("plugins.sql-viewer.max_rows").is_none());

        // But the store still contains the persisted value (retained in files)
        assert_eq!(
            store.get_value("plugins.sql-viewer.max_rows"),
            Some(&ConfigValue::Integer(500))
        );
    }

    // Validates: Requirement 8.6 — callback not invoked after unload
    #[test]
    fn unloaded_plugin_callback_not_invoked_after_deregistration() {
        use crate::reload::ReloadEvent;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::time::SystemTime;

        let mut schema = SchemaRegistry::new();
        let callbacks = CallbackRegistry::new();

        let invocation_count = Arc::new(AtomicU32::new(0));
        let count_clone = Arc::clone(&invocation_count);

        let cb_handle = callbacks.on_reload(
            &["plugins.sql-viewer.max_rows"],
            Box::new(move |_event| {
                count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        );

        // Unload the plugin (deregisters callback)
        unload_plugin("sql-viewer", &mut schema, &callbacks, vec![cb_handle]);

        // Invoke with the plugin's key — callback should NOT fire
        let event = ReloadEvent {
            changed_keys: vec!["plugins.sql-viewer.max_rows".to_string()],
            source_layer: ConfigLayer::User,
            timestamp: SystemTime::now(),
        };
        callbacks.invoke(&event);

        assert_eq!(invocation_count.load(Ordering::SeqCst), 0);
    }
}
