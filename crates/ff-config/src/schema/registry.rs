//! Schema registry.
//!
//! Maintains the set of all known configuration keys and their schema entries.
//! Supports registration at startup (core) and at runtime (plugins), and
//! provides query access for settings UI generation.

use std::collections::BTreeMap;

use super::entry::SchemaEntry;
use crate::error::ConfigError;

/// The central registry of all configuration schema entries.
///
/// The registry supports:
/// - Registration of entries at startup by core subsystems
/// - Runtime registration by plugins during initialization (schema growth)
/// - Idempotent re-registration of the same key with the same type
/// - Duplicate key detection (different type triggers `SchemaConflict` error)
/// - Prefix-based deregistration for plugin unload
/// - Querying by key and full enumeration for settings UI
#[derive(Debug, Default)]
pub struct SchemaRegistry {
    entries: BTreeMap<String, SchemaEntry>,
}

impl SchemaRegistry {
    /// Create a new, empty schema registry.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Register a schema entry.
    ///
    /// If the key is already registered with a different type, returns
    /// `ConfigError::SchemaConflict`. Re-registration with the same type
    /// is idempotent and updates the entry in place (no error).
    ///
    /// This method supports runtime schema growth — new keys can be
    /// registered at any time (e.g., during plugin initialization).
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::SchemaConflict` if a key is already registered
    /// with a different `value_type`.
    pub fn register(&mut self, entry: SchemaEntry) -> Result<(), ConfigError> {
        if let Some(existing) = self.entries.get(&entry.key) {
            if existing.value_type != entry.value_type {
                return Err(ConfigError::SchemaConflict {
                    key: entry.key.clone(),
                    details: format!(
                        "already registered as {}, cannot re-register as {}",
                        existing.value_type, entry.value_type
                    ),
                });
            }
            // Same type re-registration is allowed (idempotent update)
        }
        self.entries.insert(entry.key.clone(), entry);
        Ok(())
    }

    /// Look up a schema entry by key.
    ///
    /// Returns `None` if the key has no registered schema entry.
    pub fn get(&self, key: &str) -> Option<&SchemaEntry> {
        self.entries.get(key)
    }

    /// List all registered schema entries.
    ///
    /// Returns entries in lexicographic key order (BTreeMap ordering).
    /// Used by the settings UI for auto-generation.
    pub fn list_all(&self) -> Vec<&SchemaEntry> {
        self.entries.values().collect()
    }

    /// Deregister all entries whose key starts with the given prefix.
    ///
    /// Used during plugin unload to remove the plugin's schema entries.
    /// Returns the number of entries removed.
    pub fn deregister(&mut self, prefix: &str) -> usize {
        let keys_to_remove: Vec<String> = self
            .entries
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.entries.remove(&key);
        }
        count
    }

    /// Returns the number of registered entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::super::constraint::Constraints;
    use super::*;
    use crate::error::ValueType;
    use crate::value::ConfigValue;

    /// Helper: create a minimal schema entry for testing.
    fn make_entry(key: &str, value_type: ValueType, default: ConfigValue) -> SchemaEntry {
        SchemaEntry {
            key: key.to_string(),
            value_type,
            default,
            description: format!("Test entry for {key}"),
            constraints: None,
        }
    }

    // Validates: Requirement 9.1 — Schema registration succeeds
    #[test]
    fn register_entry_succeeds() {
        let mut registry = SchemaRegistry::new();
        let entry = make_entry(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
        );
        let result = registry.register(entry);
        assert!(result.is_ok());
        assert_eq!(registry.len(), 1);
    }

    // Validates: Requirement 9.5 — Schema is queryable; lookup by key returns entry
    #[test]
    fn get_returns_registered_entry() {
        let mut registry = SchemaRegistry::new();
        let entry = make_entry(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
        );
        registry.register(entry).unwrap();

        let retrieved = registry.get("editor.tab_size");
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.key, "editor.tab_size");
        assert_eq!(retrieved.value_type, ValueType::Integer);
        assert_eq!(retrieved.default, ConfigValue::Integer(4));
    }

    // Validates: Requirement 9.5 — Schema is queryable; get returns None for unknown key
    #[test]
    fn get_returns_none_for_unregistered_key() {
        let registry = SchemaRegistry::new();
        assert!(registry.get("nonexistent.key").is_none());
    }

    // Validates: Requirement 9.5 — list_all returns all registered entries
    #[test]
    fn list_all_returns_all_entries() {
        let mut registry = SchemaRegistry::new();
        registry
            .register(make_entry(
                "editor.tab_size",
                ValueType::Integer,
                ConfigValue::Integer(4),
            ))
            .unwrap();
        registry
            .register(make_entry(
                "theme.active",
                ValueType::String,
                ConfigValue::String("dark".to_string()),
            ))
            .unwrap();
        registry
            .register(make_entry(
                "logging.level",
                ValueType::String,
                ConfigValue::String("info".to_string()),
            ))
            .unwrap();

        let all = registry.list_all();
        assert_eq!(all.len(), 3);
        // BTreeMap ordering: editor < logging < theme
        assert_eq!(all[0].key, "editor.tab_size");
        assert_eq!(all[1].key, "logging.level");
        assert_eq!(all[2].key, "theme.active");
    }

    // Validates: Requirement 8.6 — Deregistration by prefix removes matching entries
    #[test]
    fn deregister_removes_entries_by_prefix() {
        let mut registry = SchemaRegistry::new();
        registry
            .register(make_entry(
                "plugins.sql-viewer.max_rows",
                ValueType::Integer,
                ConfigValue::Integer(100),
            ))
            .unwrap();
        registry
            .register(make_entry(
                "plugins.sql-viewer.timeout",
                ValueType::Integer,
                ConfigValue::Integer(30),
            ))
            .unwrap();
        registry
            .register(make_entry(
                "plugins.git-helper.enabled",
                ValueType::Boolean,
                ConfigValue::Boolean(true),
            ))
            .unwrap();
        registry
            .register(make_entry(
                "editor.tab_size",
                ValueType::Integer,
                ConfigValue::Integer(4),
            ))
            .unwrap();

        let removed = registry.deregister("plugins.sql-viewer");
        assert_eq!(removed, 2);
        assert_eq!(registry.len(), 2);
        assert!(registry.get("plugins.sql-viewer.max_rows").is_none());
        assert!(registry.get("plugins.sql-viewer.timeout").is_none());
        assert!(registry.get("plugins.git-helper.enabled").is_some());
        assert!(registry.get("editor.tab_size").is_some());
    }

    // Validates: Requirement 9.1 — Duplicate key with different type returns SchemaConflict
    #[test]
    fn duplicate_key_different_type_returns_schema_conflict() {
        let mut registry = SchemaRegistry::new();
        registry
            .register(make_entry(
                "editor.tab_size",
                ValueType::Integer,
                ConfigValue::Integer(4),
            ))
            .unwrap();

        let conflict_entry = make_entry(
            "editor.tab_size",
            ValueType::String,
            ConfigValue::String("four".to_string()),
        );
        let result = registry.register(conflict_entry);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("schema conflict"),
            "Error should mention schema conflict, got: {msg}"
        );
        assert!(
            msg.contains("editor.tab_size"),
            "Error should mention the key, got: {msg}"
        );
    }

    // Validates: Requirement 9.1 — Duplicate key with same type is idempotent (no error)
    #[test]
    fn duplicate_key_same_type_is_idempotent() {
        let mut registry = SchemaRegistry::new();
        registry
            .register(make_entry(
                "editor.tab_size",
                ValueType::Integer,
                ConfigValue::Integer(4),
            ))
            .unwrap();

        // Re-register with same type but different default
        let updated_entry = make_entry(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(2),
        );
        let result = registry.register(updated_entry);
        assert!(result.is_ok());

        // Value should be updated to the new registration
        let entry = registry.get("editor.tab_size").unwrap();
        assert_eq!(entry.default, ConfigValue::Integer(2));
    }

    // Validates: Requirement 9.1 — Registry starts empty
    #[test]
    fn registry_starts_empty() {
        let registry = SchemaRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.list_all().is_empty());
    }

    // Validates: Requirement 9.7 — Runtime growth: register entries after initial setup
    #[test]
    fn runtime_schema_growth_allows_new_registrations_after_initial_setup() {
        let mut registry = SchemaRegistry::new();

        // Simulate core startup registrations
        registry
            .register(make_entry(
                "editor.tab_size",
                ValueType::Integer,
                ConfigValue::Integer(4),
            ))
            .unwrap();
        registry
            .register(make_entry(
                "logging.level",
                ValueType::String,
                ConfigValue::String("info".to_string()),
            ))
            .unwrap();

        assert_eq!(registry.len(), 2);

        // Simulate plugin initialization adding new keys at runtime
        registry
            .register(make_entry(
                "plugins.sql-viewer.max_rows",
                ValueType::Integer,
                ConfigValue::Integer(1000),
            ))
            .unwrap();
        registry
            .register(make_entry(
                "plugins.sql-viewer.timeout",
                ValueType::Float,
                ConfigValue::Float(30.0),
            ))
            .unwrap();

        assert_eq!(registry.len(), 4);
        assert!(registry.get("plugins.sql-viewer.max_rows").is_some());
        assert!(registry.get("plugins.sql-viewer.timeout").is_some());
    }

    // Validates: Requirement 9.3 — Schema entries can have constraints
    #[test]
    fn entry_with_constraints_is_stored_correctly() {
        let mut registry = SchemaRegistry::new();
        let entry = SchemaEntry {
            key: "editor.tab_size".to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(4),
            description: "Number of spaces per tab".to_string(),
            constraints: Some(Constraints {
                min: Some(1.0),
                max: Some(16.0),
                allowed_values: None,
                pattern: None,
            }),
        };
        registry.register(entry).unwrap();

        let retrieved = registry.get("editor.tab_size").unwrap();
        let constraints = retrieved.constraints.as_ref().unwrap();
        assert_eq!(constraints.min, Some(1.0));
        assert_eq!(constraints.max, Some(16.0));
        assert!(constraints.allowed_values.is_none());
        assert!(constraints.pattern.is_none());
    }

    // Validates: Requirement 8.6 — Deregister with no matches returns zero
    #[test]
    fn deregister_with_no_matches_returns_zero() {
        let mut registry = SchemaRegistry::new();
        registry
            .register(make_entry(
                "editor.tab_size",
                ValueType::Integer,
                ConfigValue::Integer(4),
            ))
            .unwrap();

        let removed = registry.deregister("plugins.nonexistent");
        assert_eq!(removed, 0);
        assert_eq!(registry.len(), 1);
    }
}
