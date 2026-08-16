//! Configuration value types.
//!
//! Defines the core `ConfigValue` enum representing all possible configuration
//! value types and the `ConfigTable` alias for nested key-value maps.

use std::collections::BTreeMap;

/// A single configuration value.
///
/// Represents one of the TOML-compatible types supported by the configuration
/// system. Marked `#[non_exhaustive]` to allow future extension without
/// breaking downstream consumers.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    /// A UTF-8 string value.
    String(std::string::String),
    /// A signed 64-bit integer value.
    Integer(i64),
    /// A 64-bit floating-point value.
    Float(f64),
    /// A boolean value.
    Boolean(bool),
    /// An ordered list of configuration values.
    Array(Vec<ConfigValue>),
    /// A nested table of key-value pairs.
    Table(ConfigTable),
}

/// A mapping of string keys to configuration values, ordered lexicographically.
pub type ConfigTable = BTreeMap<String, ConfigValue>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_value_string_equality() {
        // Validates: Requirement 1.4 — String values compare by content
        let a = ConfigValue::String("hello".to_string());
        let b = ConfigValue::String("hello".to_string());
        let c = ConfigValue::String("world".to_string());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn config_value_integer_equality() {
        // Validates: Requirement 1.4 — Integer values compare numerically
        let a = ConfigValue::Integer(42);
        let b = ConfigValue::Integer(42);
        let c = ConfigValue::Integer(99);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn config_value_float_equality() {
        // Validates: Requirement 1.4 — Float values compare numerically
        let a = ConfigValue::Float(3.14);
        let b = ConfigValue::Float(3.14);
        let c = ConfigValue::Float(2.71);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn config_value_boolean_equality() {
        // Validates: Requirement 1.4 — Boolean values compare by truth value
        let a = ConfigValue::Boolean(true);
        let b = ConfigValue::Boolean(true);
        let c = ConfigValue::Boolean(false);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn config_value_array_equality() {
        // Validates: Requirement 1.4 — Array values compare element-by-element
        let a = ConfigValue::Array(vec![
            ConfigValue::Integer(1),
            ConfigValue::String("two".to_string()),
        ]);
        let b = ConfigValue::Array(vec![
            ConfigValue::Integer(1),
            ConfigValue::String("two".to_string()),
        ]);
        let c = ConfigValue::Array(vec![ConfigValue::Integer(1)]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn config_value_table_equality() {
        // Validates: Requirement 1.4 — Table values compare by key-value pairs
        let mut table_a = ConfigTable::new();
        table_a.insert("key".to_string(), ConfigValue::Boolean(true));
        let mut table_b = ConfigTable::new();
        table_b.insert("key".to_string(), ConfigValue::Boolean(true));
        let table_c = ConfigTable::new();

        assert_eq!(
            ConfigValue::Table(table_a.clone()),
            ConfigValue::Table(table_b)
        );
        assert_ne!(ConfigValue::Table(table_a), ConfigValue::Table(table_c));
    }

    #[test]
    fn config_value_different_variants_not_equal() {
        // Validates: Requirement 1.4 — Different variant types are never equal
        let string_val = ConfigValue::String("42".to_string());
        let int_val = ConfigValue::Integer(42);
        let float_val = ConfigValue::Float(42.0);
        let bool_val = ConfigValue::Boolean(true);
        let array_val = ConfigValue::Array(vec![]);
        let table_val = ConfigValue::Table(ConfigTable::new());

        assert_ne!(string_val, int_val);
        assert_ne!(int_val, float_val);
        assert_ne!(float_val, bool_val);
        assert_ne!(bool_val, array_val);
        assert_ne!(array_val, table_val);
        assert_ne!(table_val, string_val);
    }

    #[test]
    fn config_value_nested_table_equality() {
        // Validates: Requirement 1.4 — Nested tables compare recursively
        let mut inner = ConfigTable::new();
        inner.insert("nested".to_string(), ConfigValue::Integer(10));

        let mut outer_a = ConfigTable::new();
        outer_a.insert("sub".to_string(), ConfigValue::Table(inner.clone()));

        let mut outer_b = ConfigTable::new();
        outer_b.insert("sub".to_string(), ConfigValue::Table(inner));

        assert_eq!(ConfigValue::Table(outer_a), ConfigValue::Table(outer_b));
    }

    #[test]
    fn config_table_is_ordered_map() {
        // Validates: Requirement 1.3 — ConfigTable preserves lexicographic key order
        let mut table = ConfigTable::new();
        table.insert("zebra".to_string(), ConfigValue::Integer(1));
        table.insert("alpha".to_string(), ConfigValue::Integer(2));
        table.insert("middle".to_string(), ConfigValue::Integer(3));

        let keys: Vec<&String> = table.keys().collect();
        assert_eq!(keys, vec!["alpha", "middle", "zebra"]);
    }
}
