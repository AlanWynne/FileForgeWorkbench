//! # ConfigProvider Trait Implementation
//!
//! Implements `ff_core::ConfigProvider` on `ConfigHandle`, bridging the
//! configuration system to the core layer.
//!
//! The `ConfigProvider` trait is defined in `ff-core` and uses namespace-scoped
//! lookups (`(namespace, key)` → full key `"{namespace}.{key}"`). This module
//! maps those calls to the internal typed access API and handles type
//! conversions (the "serde deserialization bridge" for typed access).
//!
//! Addresses: Design §5 (Integration with ff-core), Task 22

use ff_core::ConfigProvider;

use crate::config_handle::ConfigHandle;
use crate::value::ConfigValue;

impl ConfigProvider for ConfigHandle {
    /// Retrieve a string value by namespace and key.
    ///
    /// Constructs the full dotted key (`"{namespace}.{key}"`) and delegates
    /// to `ConfigHandle::get`. Returns `Some(String)` if the value is a
    /// `ConfigValue::String`, `None` otherwise (key missing, wrong type, or
    /// lookup error).
    fn get_string(&self, namespace: &str, key: &str) -> Option<String> {
        let full_key = format!("{namespace}.{key}");
        match self.get(&full_key) {
            Ok(ConfigValue::String(s)) => Some(s),
            _ => None,
        }
    }

    /// Retrieve an unsigned 64-bit integer value by namespace and key.
    ///
    /// Constructs the full dotted key and delegates to `ConfigHandle::get`.
    /// Returns `Some(u64)` if the value is a `ConfigValue::Integer` with a
    /// non-negative value. Returns `None` if the key is missing, the value
    /// is not an integer, or the integer is negative (cannot be represented
    /// as `u64`).
    fn get_u64(&self, namespace: &str, key: &str) -> Option<u64> {
        let full_key = format!("{namespace}.{key}");
        match self.get(&full_key) {
            Ok(ConfigValue::Integer(i)) => u64::try_from(i).ok(),
            _ => None,
        }
    }

    /// Retrieve a boolean value by namespace and key.
    ///
    /// Constructs the full dotted key and delegates to `ConfigHandle::get`.
    /// Returns `Some(bool)` if the value is a `ConfigValue::Boolean`, `None`
    /// otherwise.
    fn get_bool(&self, namespace: &str, key: &str) -> Option<bool> {
        let full_key = format!("{namespace}.{key}");
        match self.get(&full_key) {
            Ok(ConfigValue::Boolean(b)) => Some(b),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::ConfigLayer;
    use crate::loader::LayerData;
    use crate::reload::ReloadManager;
    use crate::schema::SchemaRegistry;
    use std::sync::Arc;

    /// Helper: create a ConfigHandle pre-loaded with a TOML layer from a temp file.
    fn handle_with_toml(content: &str) -> (ConfigHandle, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("user.toml");
        std::fs::write(&path, content).unwrap();

        let values = crate::loader::load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path,
            values,
        }];

        let schema = SchemaRegistry::new();
        let manager = ReloadManager::new(layers, schema);
        (ConfigHandle::new(manager), dir)
    }

    // ────────────────────────────────────────────────────────────────────
    // Task 22.3: Unit tests for ConfigProvider trait integration
    // ────────────────────────────────────────────────────────────────────

    // Validates: Design §5 — get_string returns correct values via namespace+key
    #[test]
    fn config_provider_get_string_returns_value() {
        let (handle, _dir) =
            handle_with_toml("[editor]\ntheme = \"dark\"\n[logging]\nlevel = \"info\"\n");

        let provider: &dyn ConfigProvider = &handle;
        assert_eq!(
            provider.get_string("editor", "theme"),
            Some("dark".to_string())
        );
        assert_eq!(
            provider.get_string("logging", "level"),
            Some("info".to_string())
        );
    }

    // Validates: Design §5 — get_u64 returns correct values via namespace+key
    #[test]
    fn config_provider_get_u64_returns_value() {
        let (handle, _dir) = handle_with_toml("[editor]\ntab_size = 4\nindent = 8\n");

        let provider: &dyn ConfigProvider = &handle;
        assert_eq!(provider.get_u64("editor", "tab_size"), Some(4));
        assert_eq!(provider.get_u64("editor", "indent"), Some(8));
    }

    // Validates: Design §5 — get_u64 returns None for negative integers
    #[test]
    fn config_provider_get_u64_returns_none_for_negative() {
        let (handle, _dir) = handle_with_toml("[editor]\noffset = -5\n");

        let provider: &dyn ConfigProvider = &handle;
        assert_eq!(provider.get_u64("editor", "offset"), None);
    }

    // Validates: Design §5 — get_bool returns correct values via namespace+key
    #[test]
    fn config_provider_get_bool_returns_value() {
        let (handle, _dir) = handle_with_toml("[editor]\nword_wrap = true\nauto_save = false\n");

        let provider: &dyn ConfigProvider = &handle;
        assert_eq!(provider.get_bool("editor", "word_wrap"), Some(true));
        assert_eq!(provider.get_bool("editor", "auto_save"), Some(false));
    }

    // Validates: Design §5 — all methods return None for undefined keys
    #[test]
    fn config_provider_returns_none_for_undefined_keys() {
        let (handle, _dir) = handle_with_toml("[editor]\ntab_size = 4\n");

        let provider: &dyn ConfigProvider = &handle;
        assert_eq!(provider.get_string("editor", "nonexistent"), None);
        assert_eq!(provider.get_u64("editor", "nonexistent"), None);
        assert_eq!(provider.get_bool("editor", "nonexistent"), None);
        // Completely unknown namespace
        assert_eq!(provider.get_string("unknown", "key"), None);
        assert_eq!(provider.get_u64("unknown", "key"), None);
        assert_eq!(provider.get_bool("unknown", "key"), None);
    }

    // Validates: Design §5 — type mismatches return None
    #[test]
    fn config_provider_returns_none_on_type_mismatch() {
        let (handle, _dir) =
            handle_with_toml("[editor]\ntab_size = 4\ntheme = \"dark\"\nwrap = true\n");

        let provider: &dyn ConfigProvider = &handle;
        // Integer asked as string → None
        assert_eq!(provider.get_string("editor", "tab_size"), None);
        // String asked as u64 → None
        assert_eq!(provider.get_u64("editor", "theme"), None);
        // Boolean asked as string → None
        assert_eq!(provider.get_string("editor", "wrap"), None);
        // String asked as bool → None
        assert_eq!(provider.get_bool("editor", "theme"), None);
    }

    // Validates: Design §5 — ConfigHandle can be used as Box<dyn ConfigProvider> (trait object safety)
    #[test]
    fn config_provider_is_trait_object_safe() {
        let (handle, _dir) = handle_with_toml("[editor]\ntab_size = 4\n");

        let boxed: Box<dyn ConfigProvider> = Box::new(handle);
        assert_eq!(boxed.get_u64("editor", "tab_size"), Some(4));
    }

    // Validates: Design §5 — ConfigHandle as Arc<dyn ConfigProvider> works for shared ownership
    #[test]
    fn config_provider_works_with_arc() {
        let (handle, _dir) = handle_with_toml("[editor]\ntab_size = 4\ntheme = \"dark\"\n");

        let provider: Arc<dyn ConfigProvider> = Arc::new(handle);
        let provider_clone = Arc::clone(&provider);

        // Can be used from multiple owners
        assert_eq!(provider.get_u64("editor", "tab_size"), Some(4));
        assert_eq!(
            provider_clone.get_string("editor", "theme"),
            Some("dark".to_string())
        );
    }
}
