//! Language property configuration with layered override support.

use std::collections::HashMap;

use crate::definition::LanguageId;

/// Stores per-language properties with layered lookup:
/// user/project overrides → definition built-in properties.
#[derive(Debug, Clone)]
pub struct PropertyStore {
    /// Overrides from user/project configuration.
    /// Key: (language_id string, property key) → value.
    overrides: HashMap<String, HashMap<String, String>>,
    /// Built-in properties from language definitions.
    /// Key: language_id string → properties map.
    builtins: HashMap<String, HashMap<String, String>>,
}

impl PropertyStore {
    /// Create a new empty property store.
    pub fn new() -> Self {
        Self {
            overrides: HashMap::new(),
            builtins: HashMap::new(),
        }
    }

    /// Register built-in properties from a language definition.
    pub fn register_builtins(
        &mut self,
        language_id: &LanguageId,
        properties: HashMap<String, String>,
    ) {
        self.builtins
            .insert(language_id.as_str().to_string(), properties);
    }

    /// Remove built-in properties for a language.
    pub fn remove_builtins(&mut self, language_id: &LanguageId) {
        self.builtins.remove(language_id.as_str());
    }

    /// Set an override property value.
    pub fn set_override(&mut self, language_id: &LanguageId, key: &str, value: String) {
        self.overrides
            .entry(language_id.as_str().to_string())
            .or_default()
            .insert(key.to_string(), value);
    }

    /// Remove all overrides for a language.
    pub fn clear_overrides(&mut self, language_id: &LanguageId) {
        self.overrides.remove(language_id.as_str());
    }

    /// Get a property value, checking overrides first then built-in properties.
    pub fn get_property(&self, language_id: &LanguageId, key: &str) -> Option<String> {
        // Check overrides first
        if let Some(overrides) = self.overrides.get(language_id.as_str()) {
            if let Some(value) = overrides.get(key) {
                return Some(value.clone());
            }
        }
        // Fall back to built-in properties
        self.builtins
            .get(language_id.as_str())
            .and_then(|props| props.get(key))
            .cloned()
    }

    /// Get a property as an integer with a default fallback.
    ///
    /// Returns `default` if the key is absent or the value is not a valid integer.
    pub fn get_property_int(&self, language_id: &LanguageId, key: &str, default: i64) -> i64 {
        self.get_property(language_id, key)
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(default)
    }

    /// Get a property as a boolean with a default fallback.
    ///
    /// Recognized true values: "1", "true", "yes" (case-insensitive).
    /// Recognized false values: "0", "false", "no" (case-insensitive).
    /// Returns `default` for any other value or absent key.
    pub fn get_property_bool(&self, language_id: &LanguageId, key: &str, default: bool) -> bool {
        match self.get_property(language_id, key) {
            Some(v) => match v.to_lowercase().as_str() {
                "1" | "true" | "yes" => true,
                "0" | "false" | "no" => false,
                _ => default,
            },
            None => default,
        }
    }
}

impl Default for PropertyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lang_id(id: &str) -> LanguageId {
        LanguageId::new(id).unwrap()
    }

    #[test]
    fn get_property_returns_builtin_value() {
        // Validates: Requirement 8.2
        let mut store = PropertyStore::new();
        let lang = make_lang_id("rust");
        let mut props = HashMap::new();
        props.insert("tab.size".to_string(), "4".to_string());
        store.register_builtins(&lang, props);

        assert_eq!(store.get_property(&lang, "tab.size"), Some("4".to_string()));
    }

    #[test]
    fn get_property_override_takes_precedence() {
        // Validates: Requirement 8.2, 8.6
        let mut store = PropertyStore::new();
        let lang = make_lang_id("rust");
        let mut props = HashMap::new();
        props.insert("tab.size".to_string(), "4".to_string());
        store.register_builtins(&lang, props);
        store.set_override(&lang, "tab.size", "2".to_string());

        assert_eq!(store.get_property(&lang, "tab.size"), Some("2".to_string()));
    }

    #[test]
    fn get_property_returns_none_for_absent_key() {
        // Validates: Requirement 8.2
        let store = PropertyStore::new();
        let lang = make_lang_id("rust");
        assert_eq!(store.get_property(&lang, "nonexistent"), None);
    }

    #[test]
    fn get_property_int_parses_valid_integer() {
        // Validates: Requirement 8.3
        let mut store = PropertyStore::new();
        let lang = make_lang_id("rust");
        let mut props = HashMap::new();
        props.insert("tab.size".to_string(), "4".to_string());
        store.register_builtins(&lang, props);

        assert_eq!(store.get_property_int(&lang, "tab.size", 8), 4);
    }

    #[test]
    fn get_property_int_returns_default_for_invalid_value() {
        // Validates: Requirement 8.3
        let mut store = PropertyStore::new();
        let lang = make_lang_id("rust");
        let mut props = HashMap::new();
        props.insert("tab.size".to_string(), "not_a_number".to_string());
        store.register_builtins(&lang, props);

        assert_eq!(store.get_property_int(&lang, "tab.size", 8), 8);
    }

    #[test]
    fn get_property_int_returns_default_for_absent_key() {
        // Validates: Requirement 8.3
        let store = PropertyStore::new();
        let lang = make_lang_id("rust");
        assert_eq!(store.get_property_int(&lang, "missing", 42), 42);
    }

    #[test]
    fn get_property_bool_parses_true_values() {
        // Validates: Requirement 8.4
        let mut store = PropertyStore::new();
        let lang = make_lang_id("rust");
        let mut props = HashMap::new();
        props.insert("a".to_string(), "1".to_string());
        props.insert("b".to_string(), "true".to_string());
        props.insert("c".to_string(), "yes".to_string());
        props.insert("d".to_string(), "TRUE".to_string());
        props.insert("e".to_string(), "Yes".to_string());
        store.register_builtins(&lang, props);

        assert!(store.get_property_bool(&lang, "a", false));
        assert!(store.get_property_bool(&lang, "b", false));
        assert!(store.get_property_bool(&lang, "c", false));
        assert!(store.get_property_bool(&lang, "d", false));
        assert!(store.get_property_bool(&lang, "e", false));
    }

    #[test]
    fn get_property_bool_parses_false_values() {
        // Validates: Requirement 8.4
        let mut store = PropertyStore::new();
        let lang = make_lang_id("rust");
        let mut props = HashMap::new();
        props.insert("a".to_string(), "0".to_string());
        props.insert("b".to_string(), "false".to_string());
        props.insert("c".to_string(), "no".to_string());
        props.insert("d".to_string(), "FALSE".to_string());
        props.insert("e".to_string(), "No".to_string());
        store.register_builtins(&lang, props);

        assert!(!store.get_property_bool(&lang, "a", true));
        assert!(!store.get_property_bool(&lang, "b", true));
        assert!(!store.get_property_bool(&lang, "c", true));
        assert!(!store.get_property_bool(&lang, "d", true));
        assert!(!store.get_property_bool(&lang, "e", true));
    }

    #[test]
    fn get_property_bool_returns_default_for_unknown_values() {
        // Validates: Requirement 8.4
        let mut store = PropertyStore::new();
        let lang = make_lang_id("rust");
        let mut props = HashMap::new();
        props.insert("weird".to_string(), "maybe".to_string());
        store.register_builtins(&lang, props);

        assert!(store.get_property_bool(&lang, "weird", true));
        assert!(!store.get_property_bool(&lang, "weird", false));
    }

    #[test]
    fn get_property_bool_returns_default_for_absent_key() {
        // Validates: Requirement 8.4
        let store = PropertyStore::new();
        let lang = make_lang_id("rust");
        assert!(store.get_property_bool(&lang, "missing", true));
        assert!(!store.get_property_bool(&lang, "missing", false));
    }

    #[test]
    fn clear_overrides_removes_all_overrides_for_language() {
        // Validates: Requirement 8.5
        let mut store = PropertyStore::new();
        let lang = make_lang_id("rust");
        let mut props = HashMap::new();
        props.insert("tab.size".to_string(), "4".to_string());
        store.register_builtins(&lang, props);
        store.set_override(&lang, "tab.size", "2".to_string());

        store.clear_overrides(&lang);
        assert_eq!(store.get_property(&lang, "tab.size"), Some("4".to_string()));
    }
}
