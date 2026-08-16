//! Typed access API.
//!
//! Provides type-safe getter methods (`get_string`, `get_int`, `get_float`,
//! `get_bool`, `get_array`, `get_table`) for reading effective configuration
//! values with automatic type checking and schema-default fallback.
//!
//! Addresses: Requirement 9 (AC 9.1–9.11)

use std::path::Path;

use crate::editorconfig::parser::{
    Charset, EditorConfigProperties, EndOfLine, IndentSize, IndentStyle,
};
use crate::editorconfig::resolver::resolve_editorconfig as resolve_editorconfig_for_path;
use crate::error::{ConfigError, ValueType};
use crate::provenance::EffectiveValue;
use crate::schema::SchemaRegistry;
use crate::store::EffectiveStore;
use crate::validate::{validate_value, value_type_of, ValidationResult};
use crate::value::{ConfigTable, ConfigValue};

/// Provides typed access to the effective configuration store.
///
/// Each getter looks up the key in the `EffectiveStore`, checks the type,
/// and returns the value or falls back to the schema default on type mismatch
/// or validation failure.
pub struct ConfigAccess<'a> {
    store: &'a EffectiveStore,
    schema: &'a SchemaRegistry,
}

impl<'a> ConfigAccess<'a> {
    /// Create a new `ConfigAccess` wrapping a store and schema.
    pub fn new(store: &'a EffectiveStore, schema: &'a SchemaRegistry) -> Self {
        Self { store, schema }
    }

    /// Get a raw `ConfigValue` by key. Returns `UndefinedKey` if not found.
    pub fn get(&self, key: &str) -> Result<ConfigValue, ConfigError> {
        if let Some(v) = self.store.get_value(key) {
            return Ok(v.clone());
        }
        // Fall back to schema default
        if let Some(entry) = self.schema.get(key) {
            return Ok(entry.default.clone());
        }
        Err(ConfigError::UndefinedKey {
            key: key.to_string(),
        })
    }

    /// Get with provenance information.
    ///
    /// Returns the effective value along with metadata about which layer
    /// and source file provided it. Falls back to the schema default with
    /// `ConfigLayer::Defaults` provenance when the key is not in any file
    /// layer. Returns `UndefinedKey` only if neither store nor schema has
    /// the key.
    pub fn get_with_provenance(&self, key: &str) -> Result<EffectiveValue, ConfigError> {
        if let Some(ev) = self.store.get(key) {
            return Ok(ev.clone());
        }
        // Fall back to schema default with Defaults-layer provenance.
        if let Some(entry) = self.schema.get(key) {
            return Ok(crate::provenance::EffectiveValue {
                value: entry.default.clone(),
                provenance: crate::provenance::Provenance {
                    layer: crate::layer::ConfigLayer::Defaults,
                    source_file: None,
                },
            });
        }
        Err(ConfigError::UndefinedKey {
            key: key.to_string(),
        })
    }

    /// Get a string value.
    ///
    /// Falls back to schema default on type mismatch or validation failure.
    pub fn get_string(&self, key: &str) -> Result<String, ConfigError> {
        let value = self.resolve_value(key)?;
        match value {
            ConfigValue::String(s) => {
                // Validate constraints if schema entry exists
                if let Some(validated) =
                    self.validate_if_schema_exists(key, &ConfigValue::String(s.clone()))
                {
                    match validated {
                        ConfigValue::String(vs) => return Ok(vs),
                        _ => unreachable!(),
                    }
                }
                Ok(s)
            }
            other => {
                // Type mismatch — try schema default
                let found_type = value_type_of(&other);
                if let Some(entry) = self.schema.get(key) {
                    if let ConfigValue::String(ref default_s) = entry.default {
                        ff_logging::log_warn!(
                            "[config] type mismatch: key \"{}\" expected string, found {} — applying default",
                            key,
                            found_type
                        );
                        return Ok(default_s.clone());
                    }
                }
                Err(ConfigError::TypeMismatch {
                    key: key.to_string(),
                    expected: ValueType::String,
                    found: found_type,
                })
            }
        }
    }

    /// Get an integer value.
    ///
    /// Falls back to schema default on type mismatch or validation failure.
    pub fn get_int(&self, key: &str) -> Result<i64, ConfigError> {
        let value = self.resolve_value(key)?;
        match value {
            ConfigValue::Integer(i) => {
                if let Some(validated) =
                    self.validate_if_schema_exists(key, &ConfigValue::Integer(i))
                {
                    match validated {
                        ConfigValue::Integer(vi) => return Ok(vi),
                        _ => unreachable!(),
                    }
                }
                Ok(i)
            }
            other => {
                let found_type = value_type_of(&other);
                if let Some(entry) = self.schema.get(key) {
                    if let ConfigValue::Integer(ref default_i) = entry.default {
                        ff_logging::log_warn!(
                            "[config] type mismatch: key \"{}\" expected integer, found {} — applying default",
                            key,
                            found_type
                        );
                        return Ok(*default_i);
                    }
                }
                Err(ConfigError::TypeMismatch {
                    key: key.to_string(),
                    expected: ValueType::Integer,
                    found: found_type,
                })
            }
        }
    }

    /// Get a float value.
    ///
    /// Falls back to schema default on type mismatch or validation failure.
    pub fn get_float(&self, key: &str) -> Result<f64, ConfigError> {
        let value = self.resolve_value(key)?;
        match value {
            ConfigValue::Float(f) => {
                if let Some(validated) = self.validate_if_schema_exists(key, &ConfigValue::Float(f))
                {
                    match validated {
                        ConfigValue::Float(vf) => return Ok(vf),
                        _ => unreachable!(),
                    }
                }
                Ok(f)
            }
            other => {
                let found_type = value_type_of(&other);
                if let Some(entry) = self.schema.get(key) {
                    if let ConfigValue::Float(ref default_f) = entry.default {
                        ff_logging::log_warn!(
                            "[config] type mismatch: key \"{}\" expected float, found {} — applying default",
                            key,
                            found_type
                        );
                        return Ok(*default_f);
                    }
                }
                Err(ConfigError::TypeMismatch {
                    key: key.to_string(),
                    expected: ValueType::Float,
                    found: found_type,
                })
            }
        }
    }

    /// Get a boolean value.
    ///
    /// Falls back to schema default on type mismatch or validation failure.
    pub fn get_bool(&self, key: &str) -> Result<bool, ConfigError> {
        let value = self.resolve_value(key)?;
        match value {
            ConfigValue::Boolean(b) => {
                if let Some(validated) =
                    self.validate_if_schema_exists(key, &ConfigValue::Boolean(b))
                {
                    match validated {
                        ConfigValue::Boolean(vb) => return Ok(vb),
                        _ => unreachable!(),
                    }
                }
                Ok(b)
            }
            other => {
                let found_type = value_type_of(&other);
                if let Some(entry) = self.schema.get(key) {
                    if let ConfigValue::Boolean(ref default_b) = entry.default {
                        ff_logging::log_warn!(
                            "[config] type mismatch: key \"{}\" expected boolean, found {} — applying default",
                            key,
                            found_type
                        );
                        return Ok(*default_b);
                    }
                }
                Err(ConfigError::TypeMismatch {
                    key: key.to_string(),
                    expected: ValueType::Boolean,
                    found: found_type,
                })
            }
        }
    }

    /// Get an array value.
    ///
    /// Falls back to schema default on type mismatch.
    pub fn get_array(&self, key: &str) -> Result<Vec<ConfigValue>, ConfigError> {
        let value = self.resolve_value(key)?;
        match value {
            ConfigValue::Array(a) => Ok(a),
            other => {
                let found_type = value_type_of(&other);
                if let Some(entry) = self.schema.get(key) {
                    if let ConfigValue::Array(ref default_a) = entry.default {
                        ff_logging::log_warn!(
                            "[config] type mismatch: key \"{}\" expected array, found {} — applying default",
                            key,
                            found_type
                        );
                        return Ok(default_a.clone());
                    }
                }
                Err(ConfigError::TypeMismatch {
                    key: key.to_string(),
                    expected: ValueType::Array,
                    found: found_type,
                })
            }
        }
    }

    /// Get a table value.
    ///
    /// Falls back to schema default on type mismatch.
    pub fn get_table(&self, key: &str) -> Result<ConfigTable, ConfigError> {
        let value = self.resolve_value(key)?;
        match value {
            ConfigValue::Table(t) => Ok(t),
            other => {
                let found_type = value_type_of(&other);
                if let Some(entry) = self.schema.get(key) {
                    if let ConfigValue::Table(ref default_t) = entry.default {
                        ff_logging::log_warn!(
                            "[config] type mismatch: key \"{}\" expected table, found {} — applying default",
                            key,
                            found_type
                        );
                        return Ok(default_t.clone());
                    }
                }
                Err(ConfigError::TypeMismatch {
                    key: key.to_string(),
                    expected: ValueType::Table,
                    found: found_type,
                })
            }
        }
    }

    /// Internal: resolve a value from store or schema default.
    fn resolve_value(&self, key: &str) -> Result<ConfigValue, ConfigError> {
        if let Some(v) = self.store.get_value(key) {
            return Ok(v.clone());
        }
        // Try schema default
        if let Some(entry) = self.schema.get(key) {
            return Ok(entry.default.clone());
        }
        Err(ConfigError::UndefinedKey {
            key: key.to_string(),
        })
    }

    /// Internal: validate a value against schema constraints.
    ///
    /// Returns `Some(value)` with either the original value (if valid) or
    /// the schema default (if validation failed, with WARN log).
    /// Returns `None` if no schema entry exists for this key.
    fn validate_if_schema_exists(&self, key: &str, value: &ConfigValue) -> Option<ConfigValue> {
        let entry = self.schema.get(key)?;
        match validate_value(value, entry) {
            ValidationResult::Valid(v) => Some(v),
            ValidationResult::DefaultApplied { reason, default } => {
                ff_logging::log_warn!(
                    "[config] validation: key \"{}\": {} — applying default",
                    key,
                    reason
                );
                Some(default)
            }
        }
    }

    /// Resolve EditorConfig properties for a given file path.
    ///
    /// Delegates to the EditorConfig resolver, which traverses the directory
    /// hierarchy looking for `.editorconfig` files and merges matching sections.
    ///
    /// # Arguments
    ///
    /// * `file_path` — The absolute path of the file to resolve properties for.
    ///
    /// # Returns
    ///
    /// The merged `EditorConfigProperties` for the given file path.
    pub fn resolve_editorconfig(&self, file_path: &Path) -> EditorConfigProperties {
        resolve_editorconfig_for_path(file_path)
    }

    /// Get a configuration value for a specific file, applying EditorConfig
    /// precedence for editor-scoped keys.
    ///
    /// For keys in the `editor.*` namespace, this method first resolves
    /// EditorConfig properties for the given file path. If EditorConfig
    /// provides a value for the corresponding property, that value is returned
    /// (EditorConfig overrides ALL configuration layers for editor keys).
    ///
    /// For keys outside the `editor.*` namespace (e.g., `logging.*`, `theme.*`,
    /// `plugins.*`, `vfs.*`), EditorConfig is never consulted and the normal
    /// layered resolution applies.
    ///
    /// # Arguments
    ///
    /// * `key` — The configuration key to look up (e.g., `"editor.indent_style"`).
    /// * `file_path` — The absolute path of the file being edited.
    ///
    /// # Returns
    ///
    /// The resolved `ConfigValue`, or a `ConfigError` if the key is undefined.
    pub fn get_for_file(&self, key: &str, file_path: &Path) -> Result<ConfigValue, ConfigError> {
        // Only editor-scoped keys consult EditorConfig (Task 17.5)
        if is_editor_key(key) {
            let ec_props = self.resolve_editorconfig(file_path);
            if let Some(value) = editorconfig_value_for_key(key, &ec_props) {
                return Ok(value);
            }
        }
        // Fall back to normal layered resolution
        self.get(key)
    }

    /// Get a string value for a specific file, applying EditorConfig precedence.
    ///
    /// Behaves like `get_string`, but for editor-scoped keys, EditorConfig
    /// values take priority over all configuration layers.
    pub fn get_string_for_file(&self, key: &str, file_path: &Path) -> Result<String, ConfigError> {
        if is_editor_key(key) {
            let ec_props = self.resolve_editorconfig(file_path);
            if let Some(ConfigValue::String(s)) = editorconfig_value_for_key(key, &ec_props) {
                return Ok(s);
            }
        }
        self.get_string(key)
    }

    /// Get an integer value for a specific file, applying EditorConfig precedence.
    ///
    /// Behaves like `get_int`, but for editor-scoped keys, EditorConfig
    /// values take priority over all configuration layers.
    pub fn get_int_for_file(&self, key: &str, file_path: &Path) -> Result<i64, ConfigError> {
        if is_editor_key(key) {
            let ec_props = self.resolve_editorconfig(file_path);
            if let Some(ConfigValue::Integer(i)) = editorconfig_value_for_key(key, &ec_props) {
                return Ok(i);
            }
        }
        self.get_int(key)
    }

    /// Get a boolean value for a specific file, applying EditorConfig precedence.
    ///
    /// Behaves like `get_bool`, but for editor-scoped keys, EditorConfig
    /// values take priority over all configuration layers.
    pub fn get_bool_for_file(&self, key: &str, file_path: &Path) -> Result<bool, ConfigError> {
        if is_editor_key(key) {
            let ec_props = self.resolve_editorconfig(file_path);
            if let Some(ConfigValue::Boolean(b)) = editorconfig_value_for_key(key, &ec_props) {
                return Ok(b);
            }
        }
        self.get_bool(key)
    }
}

/// Check whether a configuration key is in the `editor.*` namespace.
///
/// Only editor-scoped keys are eligible for EditorConfig override.
/// Keys in other namespaces (logging, theme, plugins, vfs, etc.) are
/// never affected by EditorConfig settings.
fn is_editor_key(key: &str) -> bool {
    key.starts_with("editor.")
}

/// Map a configuration key to the corresponding EditorConfig property value.
///
/// Returns `Some(ConfigValue)` if the EditorConfig properties have a value
/// for the property that maps to the given key. Returns `None` if the
/// EditorConfig has no value for this key.
fn editorconfig_value_for_key(key: &str, props: &EditorConfigProperties) -> Option<ConfigValue> {
    match key {
        "editor.indent_style" => props.indent_style.map(|v| {
            ConfigValue::String(
                match v {
                    IndentStyle::Space => "space",
                    IndentStyle::Tab => "tab",
                }
                .to_string(),
            )
        }),
        "editor.indent_size" | "editor.tab_size" => props.indent_size.map(|v| match v {
            IndentSize::Value(n) => ConfigValue::Integer(i64::from(n)),
            IndentSize::Tab => ConfigValue::String("tab".to_string()),
        }),
        "editor.tab_width" => props.tab_width.map(|v| ConfigValue::Integer(i64::from(v))),
        "editor.end_of_line" | "editor.line_endings" => props.end_of_line.map(|v| {
            ConfigValue::String(
                match v {
                    EndOfLine::Lf => "lf",
                    EndOfLine::CrLf => "crlf",
                    EndOfLine::Cr => "cr",
                }
                .to_string(),
            )
        }),
        "editor.charset" => props.charset.map(|v| {
            ConfigValue::String(
                match v {
                    Charset::Utf8 => "utf-8",
                    Charset::Utf8Bom => "utf-8-bom",
                    Charset::Latin1 => "latin1",
                    Charset::Utf16Be => "utf-16be",
                    Charset::Utf16Le => "utf-16le",
                }
                .to_string(),
            )
        }),
        "editor.trim_trailing_whitespace" => {
            props.trim_trailing_whitespace.map(ConfigValue::Boolean)
        }
        "editor.insert_final_newline" => props.insert_final_newline.map(ConfigValue::Boolean),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::ConfigLayer;
    use crate::provenance::Provenance;
    use crate::schema::{Constraints, SchemaEntry};
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

    /// Helper: build a schema with a single entry (no constraints).
    fn schema_with(key: &str, value_type: ValueType, default: ConfigValue) -> SchemaRegistry {
        let mut schema = SchemaRegistry::new();
        schema
            .register(SchemaEntry {
                key: key.to_string(),
                value_type,
                default,
                description: format!("Test entry for {key}"),
                constraints: None,
            })
            .unwrap();
        schema
    }

    /// Helper: build a schema with constraints.
    fn schema_with_constraints(
        key: &str,
        value_type: ValueType,
        default: ConfigValue,
        constraints: Constraints,
    ) -> SchemaRegistry {
        let mut schema = SchemaRegistry::new();
        schema
            .register(SchemaEntry {
                key: key.to_string(),
                value_type,
                default,
                description: format!("Test entry for {key}"),
                constraints: Some(constraints),
            })
            .unwrap();
        schema
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 9.1: get_string
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.1 — get_string returns string value
    #[test]
    fn get_string_returns_value_when_type_matches() {
        let store = store_with("theme.name", ConfigValue::String("dark".to_string()));
        let schema = schema_with(
            "theme.name",
            ValueType::String,
            ConfigValue::String("light".to_string()),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_string("theme.name");
        assert_eq!(result.unwrap(), "dark");
    }

    // Validates: Requirement 9.1 — get_string returns UndefinedKey for missing key
    #[test]
    fn get_string_returns_undefined_key_when_not_found() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_string("nonexistent");
        assert!(matches!(result, Err(ConfigError::UndefinedKey { .. })));
    }

    // Validates: Requirement 9.1 — get_string returns schema default when key absent from store
    #[test]
    fn get_string_returns_schema_default_when_key_absent_from_store() {
        let store = EffectiveStore::new();
        let schema = schema_with(
            "theme.name",
            ValueType::String,
            ConfigValue::String("light".to_string()),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_string("theme.name");
        assert_eq!(result.unwrap(), "light");
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 9.2: get_int
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.2 — get_int returns integer value
    #[test]
    fn get_int_returns_value_when_type_matches() {
        let store = store_with("editor.tab_size", ConfigValue::Integer(8));
        let schema = schema_with(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_int("editor.tab_size");
        assert_eq!(result.unwrap(), 8);
    }

    // Validates: Requirement 9.2 — get_int returns UndefinedKey for missing key
    #[test]
    fn get_int_returns_undefined_key_when_not_found() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_int("nonexistent");
        assert!(matches!(result, Err(ConfigError::UndefinedKey { .. })));
    }

    // Validates: Requirement 9.2 — get_int returns schema default when key absent from store
    #[test]
    fn get_int_returns_schema_default_when_key_absent_from_store() {
        let store = EffectiveStore::new();
        let schema = schema_with(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_int("editor.tab_size");
        assert_eq!(result.unwrap(), 4);
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 9.3: get_float
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.3 — get_float returns float value
    #[test]
    fn get_float_returns_value_when_type_matches() {
        let store = store_with("editor.font_size", ConfigValue::Float(14.5));
        let schema = schema_with(
            "editor.font_size",
            ValueType::Float,
            ConfigValue::Float(12.0),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_float("editor.font_size");
        assert!((result.unwrap() - 14.5).abs() < f64::EPSILON);
    }

    // Validates: Requirement 9.3 — get_float returns UndefinedKey for missing key
    #[test]
    fn get_float_returns_undefined_key_when_not_found() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_float("nonexistent");
        assert!(matches!(result, Err(ConfigError::UndefinedKey { .. })));
    }

    // Validates: Requirement 9.3 — get_float returns schema default when key absent
    #[test]
    fn get_float_returns_schema_default_when_key_absent_from_store() {
        let store = EffectiveStore::new();
        let schema = schema_with(
            "editor.font_size",
            ValueType::Float,
            ConfigValue::Float(12.0),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_float("editor.font_size");
        assert!((result.unwrap() - 12.0).abs() < f64::EPSILON);
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 9.4: get_bool
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.4 — get_bool returns boolean value
    #[test]
    fn get_bool_returns_value_when_type_matches() {
        let store = store_with("editor.word_wrap", ConfigValue::Boolean(true));
        let schema = schema_with(
            "editor.word_wrap",
            ValueType::Boolean,
            ConfigValue::Boolean(false),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_bool("editor.word_wrap");
        assert_eq!(result.unwrap(), true);
    }

    // Validates: Requirement 9.4 — get_bool returns UndefinedKey for missing key
    #[test]
    fn get_bool_returns_undefined_key_when_not_found() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_bool("nonexistent");
        assert!(matches!(result, Err(ConfigError::UndefinedKey { .. })));
    }

    // Validates: Requirement 9.4 — get_bool returns schema default when key absent
    #[test]
    fn get_bool_returns_schema_default_when_key_absent_from_store() {
        let store = EffectiveStore::new();
        let schema = schema_with(
            "editor.word_wrap",
            ValueType::Boolean,
            ConfigValue::Boolean(false),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_bool("editor.word_wrap");
        assert_eq!(result.unwrap(), false);
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 9.5: get_array
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.5 — get_array returns array value
    #[test]
    fn get_array_returns_value_when_type_matches() {
        let arr = vec![
            ConfigValue::String("a".to_string()),
            ConfigValue::Integer(1),
        ];
        let store = store_with("editor.rulers", ConfigValue::Array(arr.clone()));
        let schema = schema_with(
            "editor.rulers",
            ValueType::Array,
            ConfigValue::Array(vec![]),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_array("editor.rulers");
        assert_eq!(result.unwrap(), arr);
    }

    // Validates: Requirement 9.5 — get_array returns UndefinedKey for missing key
    #[test]
    fn get_array_returns_undefined_key_when_not_found() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_array("nonexistent");
        assert!(matches!(result, Err(ConfigError::UndefinedKey { .. })));
    }

    // Validates: Requirement 9.5 — get_array returns schema default when key absent
    #[test]
    fn get_array_returns_schema_default_when_key_absent_from_store() {
        let store = EffectiveStore::new();
        let default_arr = vec![ConfigValue::Integer(80)];
        let schema = schema_with(
            "editor.rulers",
            ValueType::Array,
            ConfigValue::Array(default_arr.clone()),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_array("editor.rulers");
        assert_eq!(result.unwrap(), default_arr);
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 9.6: get_table
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.6 — get_table returns table value
    #[test]
    fn get_table_returns_value_when_type_matches() {
        let mut table = ConfigTable::new();
        table.insert("nested_key".to_string(), ConfigValue::Boolean(true));
        let store = store_with("editor.settings", ConfigValue::Table(table.clone()));
        let schema = schema_with(
            "editor.settings",
            ValueType::Table,
            ConfigValue::Table(ConfigTable::new()),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_table("editor.settings");
        assert_eq!(result.unwrap(), table);
    }

    // Validates: Requirement 9.6 — get_table returns UndefinedKey for missing key
    #[test]
    fn get_table_returns_undefined_key_when_not_found() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_table("nonexistent");
        assert!(matches!(result, Err(ConfigError::UndefinedKey { .. })));
    }

    // Validates: Requirement 9.6 — get_table returns schema default when key absent
    #[test]
    fn get_table_returns_schema_default_when_key_absent_from_store() {
        let store = EffectiveStore::new();
        let mut default_table = ConfigTable::new();
        default_table.insert("default_key".to_string(), ConfigValue::Integer(1));
        let schema = schema_with(
            "editor.settings",
            ValueType::Table,
            ConfigValue::Table(default_table.clone()),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_table("editor.settings");
        assert_eq!(result.unwrap(), default_table);
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 9.7: get (generic getter)
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.7 — get returns raw ConfigValue
    #[test]
    fn get_returns_raw_value_from_store() {
        let store = store_with("editor.tab_size", ConfigValue::Integer(8));
        let schema = schema_with(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get("editor.tab_size");
        assert_eq!(result.unwrap(), ConfigValue::Integer(8));
    }

    // Validates: Requirement 9.7 — get falls back to schema default
    #[test]
    fn get_returns_schema_default_when_key_absent_from_store() {
        let store = EffectiveStore::new();
        let schema = schema_with(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get("editor.tab_size");
        assert_eq!(result.unwrap(), ConfigValue::Integer(4));
    }

    // Validates: Requirement 9.7 — get returns UndefinedKey when neither store nor schema has key
    #[test]
    fn get_returns_undefined_key_when_not_found_anywhere() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get("nonexistent");
        assert!(matches!(result, Err(ConfigError::UndefinedKey { .. })));
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 9.8: get_with_provenance
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.8 — get_with_provenance returns EffectiveValue
    #[test]
    fn get_with_provenance_returns_effective_value() {
        let store = store_with("editor.tab_size", ConfigValue::Integer(8));
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_with_provenance("editor.tab_size").unwrap();
        assert_eq!(result.value, ConfigValue::Integer(8));
        assert_eq!(result.provenance.layer, ConfigLayer::User);
        assert_eq!(
            result.provenance.source_file,
            Some(PathBuf::from("test.toml"))
        );
    }

    // Validates: Requirement 9.8 — get_with_provenance returns UndefinedKey when neither store nor schema has key
    #[test]
    fn get_with_provenance_returns_undefined_key_when_not_found() {
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_with_provenance("nonexistent");
        assert!(matches!(result, Err(ConfigError::UndefinedKey { .. })));
    }

    // Validates: Requirement 2.5 — get_with_provenance returns schema default with Defaults provenance
    #[test]
    fn get_with_provenance_returns_schema_default_with_defaults_provenance() {
        let store = EffectiveStore::new();
        let schema = schema_with(
            "logging.level",
            ValueType::String,
            ConfigValue::String("info".to_string()),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_with_provenance("logging.level").unwrap();
        assert_eq!(result.value, ConfigValue::String("info".to_string()));
        assert_eq!(result.provenance.layer, ConfigLayer::Defaults);
        assert!(result.provenance.source_file.is_none());
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 9.9: Type mismatch fallback
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.9 — get_string type mismatch falls back to schema default
    #[test]
    fn get_string_type_mismatch_falls_back_to_schema_default() {
        let store = store_with("theme.name", ConfigValue::Integer(42));
        let schema = schema_with(
            "theme.name",
            ValueType::String,
            ConfigValue::String("dark".to_string()),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_string("theme.name");
        assert_eq!(result.unwrap(), "dark");
    }

    // Validates: Requirement 9.9 — get_int type mismatch falls back to schema default
    #[test]
    fn get_int_type_mismatch_falls_back_to_schema_default() {
        let store = store_with("editor.tab_size", ConfigValue::String("four".to_string()));
        let schema = schema_with(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_int("editor.tab_size");
        assert_eq!(result.unwrap(), 4);
    }

    // Validates: Requirement 9.9 — get_float type mismatch falls back to schema default
    #[test]
    fn get_float_type_mismatch_falls_back_to_schema_default() {
        let store = store_with("editor.font_size", ConfigValue::Boolean(true));
        let schema = schema_with(
            "editor.font_size",
            ValueType::Float,
            ConfigValue::Float(12.0),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_float("editor.font_size");
        assert!((result.unwrap() - 12.0).abs() < f64::EPSILON);
    }

    // Validates: Requirement 9.9 — get_bool type mismatch falls back to schema default
    #[test]
    fn get_bool_type_mismatch_falls_back_to_schema_default() {
        let store = store_with("editor.word_wrap", ConfigValue::Integer(1));
        let schema = schema_with(
            "editor.word_wrap",
            ValueType::Boolean,
            ConfigValue::Boolean(true),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_bool("editor.word_wrap");
        assert_eq!(result.unwrap(), true);
    }

    // Validates: Requirement 9.9 — get_array type mismatch falls back to schema default
    #[test]
    fn get_array_type_mismatch_falls_back_to_schema_default() {
        let store = store_with("editor.rulers", ConfigValue::String("80".to_string()));
        let default_arr = vec![ConfigValue::Integer(80)];
        let schema = schema_with(
            "editor.rulers",
            ValueType::Array,
            ConfigValue::Array(default_arr.clone()),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_array("editor.rulers");
        assert_eq!(result.unwrap(), default_arr);
    }

    // Validates: Requirement 9.9 — get_table type mismatch falls back to schema default
    #[test]
    fn get_table_type_mismatch_falls_back_to_schema_default() {
        let store = store_with("editor.settings", ConfigValue::Integer(99));
        let mut default_table = ConfigTable::new();
        default_table.insert("key".to_string(), ConfigValue::Boolean(false));
        let schema = schema_with(
            "editor.settings",
            ValueType::Table,
            ConfigValue::Table(default_table.clone()),
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_table("editor.settings");
        assert_eq!(result.unwrap(), default_table);
    }

    // Validates: Requirement 9.9 — type mismatch without schema returns TypeMismatch error
    #[test]
    fn get_string_type_mismatch_without_schema_returns_error() {
        let store = store_with("unknown.key", ConfigValue::Integer(42));
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_string("unknown.key");
        assert!(matches!(
            result,
            Err(ConfigError::TypeMismatch {
                expected: ValueType::String,
                found: ValueType::Integer,
                ..
            })
        ));
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 9.10: Validation failure fallback
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.10 — validation failure falls back to schema default (integer range)
    #[test]
    fn get_int_validation_failure_falls_back_to_schema_default() {
        let store = store_with("editor.tab_size", ConfigValue::Integer(999));
        let schema = schema_with_constraints(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
            Constraints {
                min: Some(1.0),
                max: Some(16.0),
                allowed_values: None,
                pattern: None,
            },
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_int("editor.tab_size");
        assert_eq!(result.unwrap(), 4);
    }

    // Validates: Requirement 9.10 — validation failure falls back to schema default (string pattern)
    #[test]
    fn get_string_validation_failure_falls_back_to_schema_default() {
        let store = store_with(
            "logging.level",
            ConfigValue::String("invalid_level".to_string()),
        );
        let schema = schema_with_constraints(
            "logging.level",
            ValueType::String,
            ConfigValue::String("info".to_string()),
            Constraints {
                min: None,
                max: None,
                allowed_values: None,
                pattern: Some("^(trace|debug|info|warn|error)$".to_string()),
            },
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_string("logging.level");
        assert_eq!(result.unwrap(), "info");
    }

    // Validates: Requirement 9.10 — validation failure with enum constraint falls back
    #[test]
    fn get_string_enum_validation_failure_falls_back_to_schema_default() {
        let store = store_with(
            "editor.indent_style",
            ConfigValue::String("mixed".to_string()),
        );
        let schema = schema_with_constraints(
            "editor.indent_style",
            ValueType::String,
            ConfigValue::String("space".to_string()),
            Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("space".to_string()),
                    ConfigValue::String("tab".to_string()),
                ]),
                pattern: None,
            },
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_string("editor.indent_style");
        assert_eq!(result.unwrap(), "space");
    }

    // Validates: Requirement 9.10 — float validation failure falls back to schema default
    #[test]
    fn get_float_validation_failure_falls_back_to_schema_default() {
        let store = store_with("editor.font_size", ConfigValue::Float(200.0));
        let schema = schema_with_constraints(
            "editor.font_size",
            ValueType::Float,
            ConfigValue::Float(12.0),
            Constraints {
                min: Some(6.0),
                max: Some(72.0),
                allowed_values: None,
                pattern: None,
            },
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_float("editor.font_size");
        assert!((result.unwrap() - 12.0).abs() < f64::EPSILON);
    }

    // Validates: Requirement 9.10 — valid value passes through without fallback
    #[test]
    fn get_int_valid_value_passes_through_without_fallback() {
        let store = store_with("editor.tab_size", ConfigValue::Integer(8));
        let schema = schema_with_constraints(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
            Constraints {
                min: Some(1.0),
                max: Some(16.0),
                allowed_values: None,
                pattern: None,
            },
        );
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_int("editor.tab_size");
        assert_eq!(result.unwrap(), 8);
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 17.3: resolve_editorconfig on ConfigAccess
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 6 AC 6.3 — resolve_editorconfig delegates correctly
    #[test]
    fn resolve_editorconfig_delegates_to_resolver() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Create .editorconfig with indent_style = space
        fs::write(
            root.join(".editorconfig"),
            "root = true\n\n[*]\nindent_style = space\nindent_size = 2\n",
        )
        .unwrap();

        let file_path = root.join("main.rs");
        fs::write(&file_path, "").unwrap();

        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let props = access.resolve_editorconfig(&file_path);
        assert_eq!(
            props.indent_style,
            Some(crate::editorconfig::parser::IndentStyle::Space)
        );
        assert_eq!(
            props.indent_size,
            Some(crate::editorconfig::parser::IndentSize::Value(2))
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 17.4: EditorConfig precedence over all layers
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 6 AC 6.3 — EditorConfig overrides workspace-layer value
    #[test]
    fn get_for_file_editorconfig_overrides_workspace_layer_for_editor_key() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // EditorConfig says indent_size = 2
        fs::write(
            root.join(".editorconfig"),
            "root = true\n\n[*]\nindent_size = 2\n",
        )
        .unwrap();

        let file_path = root.join("main.rs");
        fs::write(&file_path, "").unwrap();

        // Workspace layer has indent_size = 8
        let store = store_with("editor.indent_size", ConfigValue::Integer(8));
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        // EditorConfig (indent_size = 2) should override workspace (indent_size = 8)
        let result = access
            .get_for_file("editor.indent_size", &file_path)
            .unwrap();
        assert_eq!(result, ConfigValue::Integer(2));
    }

    // Validates: Requirement 6 AC 6.3 — EditorConfig overrides for indent_style
    #[test]
    fn get_string_for_file_editorconfig_overrides_for_indent_style() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // EditorConfig says indent_style = tab
        fs::write(
            root.join(".editorconfig"),
            "root = true\n\n[*]\nindent_style = tab\n",
        )
        .unwrap();

        let file_path = root.join("main.rs");
        fs::write(&file_path, "").unwrap();

        // Store has indent_style = space (from config layers)
        let store = store_with(
            "editor.indent_style",
            ConfigValue::String("space".to_string()),
        );
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access
            .get_string_for_file("editor.indent_style", &file_path)
            .unwrap();
        assert_eq!(result, "tab");
    }

    // Validates: Requirement 6 AC 6.3 — EditorConfig overrides for boolean properties
    #[test]
    fn get_bool_for_file_editorconfig_overrides_trim_trailing_whitespace() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // EditorConfig says trim_trailing_whitespace = true
        fs::write(
            root.join(".editorconfig"),
            "root = true\n\n[*]\ntrim_trailing_whitespace = true\n",
        )
        .unwrap();

        let file_path = root.join("main.rs");
        fs::write(&file_path, "").unwrap();

        // Store has trim_trailing_whitespace = false
        let store = store_with(
            "editor.trim_trailing_whitespace",
            ConfigValue::Boolean(false),
        );
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access
            .get_bool_for_file("editor.trim_trailing_whitespace", &file_path)
            .unwrap();
        assert!(result);
    }

    // Validates: Requirement 6 AC 6.3 — falls back to layered value when EditorConfig has no value
    #[test]
    fn get_for_file_falls_back_to_store_when_editorconfig_has_no_value() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // EditorConfig only defines indent_style, not tab_width
        fs::write(
            root.join(".editorconfig"),
            "root = true\n\n[*]\nindent_style = space\n",
        )
        .unwrap();

        let file_path = root.join("main.rs");
        fs::write(&file_path, "").unwrap();

        // Store has tab_width = 4
        let store = store_with("editor.tab_width", ConfigValue::Integer(4));
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        // tab_width not in EditorConfig, so falls back to store
        let result = access.get_for_file("editor.tab_width", &file_path).unwrap();
        assert_eq!(result, ConfigValue::Integer(4));
    }

    // Validates: Requirement 6 AC 6.5 — multiple .editorconfig files merge correctly
    #[test]
    fn get_for_file_merges_multiple_editorconfig_files_closer_wins() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Root level: indent_size = 4
        fs::write(
            root.join(".editorconfig"),
            "root = true\n\n[*]\nindent_size = 4\ncharset = utf-8\n",
        )
        .unwrap();

        // Sub directory: indent_size = 2 (overrides root)
        let sub = root.join("src");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join(".editorconfig"), "[*]\nindent_size = 2\n").unwrap();

        let file_path = sub.join("lib.rs");
        fs::write(&file_path, "").unwrap();

        let store = EffectiveStore::new();
        let schema = schema_with(
            "editor.indent_size",
            ValueType::Integer,
            ConfigValue::Integer(8),
        );
        let access = ConfigAccess::new(&store, &schema);

        // Closer .editorconfig (src/) wins: indent_size = 2
        let result = access
            .get_for_file("editor.indent_size", &file_path)
            .unwrap();
        assert_eq!(result, ConfigValue::Integer(2));

        // charset comes from root .editorconfig: utf-8
        let result = access.get_for_file("editor.charset", &file_path).unwrap();
        assert_eq!(result, ConfigValue::String("utf-8".to_string()));
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 17.5: Scope restriction — EditorConfig only applies to editor keys
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 6 AC 6.7 — logging keys are not affected by EditorConfig
    #[test]
    fn get_for_file_does_not_apply_editorconfig_to_logging_keys() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::write(
            root.join(".editorconfig"),
            "root = true\n\n[*]\nindent_style = space\n",
        )
        .unwrap();

        let file_path = root.join("main.rs");
        fs::write(&file_path, "").unwrap();

        let store = store_with("logging.level", ConfigValue::String("debug".to_string()));
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        // logging.level should come from store, unaffected by EditorConfig
        let result = access.get_for_file("logging.level", &file_path).unwrap();
        assert_eq!(result, ConfigValue::String("debug".to_string()));
    }

    // Validates: Requirement 6 AC 6.7 — theme keys are not affected by EditorConfig
    #[test]
    fn get_for_file_does_not_apply_editorconfig_to_theme_keys() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::write(
            root.join(".editorconfig"),
            "root = true\n\n[*]\nindent_style = tab\n",
        )
        .unwrap();

        let file_path = root.join("main.rs");
        fs::write(&file_path, "").unwrap();

        let store = store_with("theme.active", ConfigValue::String("dark".to_string()));
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access.get_for_file("theme.active", &file_path).unwrap();
        assert_eq!(result, ConfigValue::String("dark".to_string()));
    }

    // Validates: Requirement 6 AC 6.7 — plugin keys are not affected by EditorConfig
    #[test]
    fn get_for_file_does_not_apply_editorconfig_to_plugin_keys() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::write(
            root.join(".editorconfig"),
            "root = true\n\n[*]\nindent_size = 2\n",
        )
        .unwrap();

        let file_path = root.join("main.rs");
        fs::write(&file_path, "").unwrap();

        let store = store_with("plugins.my-plugin.enabled", ConfigValue::Boolean(true));
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access
            .get_for_file("plugins.my-plugin.enabled", &file_path)
            .unwrap();
        assert_eq!(result, ConfigValue::Boolean(true));
    }

    // Validates: Requirement 6 AC 6.7 — vfs keys are not affected by EditorConfig
    #[test]
    fn get_for_file_does_not_apply_editorconfig_to_vfs_keys() {
        use std::fs;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::write(
            root.join(".editorconfig"),
            "root = true\n\n[*]\nindent_style = space\n",
        )
        .unwrap();

        let file_path = root.join("main.rs");
        fs::write(&file_path, "").unwrap();

        let store = store_with(
            "vfs.default_provider",
            ConfigValue::String("local".to_string()),
        );
        let schema = SchemaRegistry::new();
        let access = ConfigAccess::new(&store, &schema);

        let result = access
            .get_for_file("vfs.default_provider", &file_path)
            .unwrap();
        assert_eq!(result, ConfigValue::String("local".to_string()));
    }

    // ──────────────────────────────────────────────────────────────────
    // Unit tests for helper functions
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 6 AC 6.7 — is_editor_key correctly identifies editor namespace
    #[test]
    fn is_editor_key_identifies_editor_namespace() {
        assert!(super::is_editor_key("editor.tab_size"));
        assert!(super::is_editor_key("editor.indent_style"));
        assert!(super::is_editor_key("editor.charset"));
        assert!(!super::is_editor_key("logging.level"));
        assert!(!super::is_editor_key("theme.active"));
        assert!(!super::is_editor_key("plugins.foo.bar"));
        assert!(!super::is_editor_key("vfs.default_provider"));
    }

    // Validates: Requirement 6 AC 6.3 — editorconfig_value_for_key maps correctly
    #[test]
    fn editorconfig_value_for_key_maps_all_properties() {
        use crate::editorconfig::parser::{Charset, EndOfLine, IndentSize, IndentStyle};

        let props = EditorConfigProperties {
            indent_style: Some(IndentStyle::Space),
            indent_size: Some(IndentSize::Value(4)),
            tab_width: Some(8),
            end_of_line: Some(EndOfLine::Lf),
            charset: Some(Charset::Utf8),
            trim_trailing_whitespace: Some(true),
            insert_final_newline: Some(false),
        };

        assert_eq!(
            super::editorconfig_value_for_key("editor.indent_style", &props),
            Some(ConfigValue::String("space".to_string()))
        );
        assert_eq!(
            super::editorconfig_value_for_key("editor.indent_size", &props),
            Some(ConfigValue::Integer(4))
        );
        assert_eq!(
            super::editorconfig_value_for_key("editor.tab_size", &props),
            Some(ConfigValue::Integer(4))
        );
        assert_eq!(
            super::editorconfig_value_for_key("editor.tab_width", &props),
            Some(ConfigValue::Integer(8))
        );
        assert_eq!(
            super::editorconfig_value_for_key("editor.end_of_line", &props),
            Some(ConfigValue::String("lf".to_string()))
        );
        assert_eq!(
            super::editorconfig_value_for_key("editor.line_endings", &props),
            Some(ConfigValue::String("lf".to_string()))
        );
        assert_eq!(
            super::editorconfig_value_for_key("editor.charset", &props),
            Some(ConfigValue::String("utf-8".to_string()))
        );
        assert_eq!(
            super::editorconfig_value_for_key("editor.trim_trailing_whitespace", &props),
            Some(ConfigValue::Boolean(true))
        );
        assert_eq!(
            super::editorconfig_value_for_key("editor.insert_final_newline", &props),
            Some(ConfigValue::Boolean(false))
        );
    }

    // Validates: Requirement 6 AC 6.7 — unmapped editor keys return None
    #[test]
    fn editorconfig_value_for_key_returns_none_for_unmapped_editor_keys() {
        let props = EditorConfigProperties {
            indent_style: Some(IndentStyle::Space),
            ..Default::default()
        };

        // A hypothetical editor key that has no EditorConfig mapping
        assert_eq!(
            super::editorconfig_value_for_key("editor.word_wrap", &props),
            None
        );
    }

    // Validates: Requirement 6 AC 6.3 — None properties return None from mapping
    #[test]
    fn editorconfig_value_for_key_returns_none_when_property_not_set() {
        let props = EditorConfigProperties::default();

        assert_eq!(
            super::editorconfig_value_for_key("editor.indent_style", &props),
            None
        );
        assert_eq!(
            super::editorconfig_value_for_key("editor.indent_size", &props),
            None
        );
        assert_eq!(
            super::editorconfig_value_for_key("editor.tab_width", &props),
            None
        );
    }
}
