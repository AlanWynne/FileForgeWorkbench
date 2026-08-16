//! `CommandParams` typed key-value map for passing parameters to commands.
//!
//! Supports string, integer, float, boolean, and nested map value types.

use std::collections::HashMap;

/// A single parameter value within `CommandParams`.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    /// A string value.
    String(String),
    /// A 64-bit signed integer value.
    Integer(i64),
    /// A 64-bit floating-point value.
    Float(f64),
    /// A boolean value.
    Boolean(bool),
    /// A nested key-value map.
    Map(HashMap<String, ParamValue>),
}

impl From<String> for ParamValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for ParamValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for ParamValue {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}

impl From<f64> for ParamValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<bool> for ParamValue {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}

impl From<HashMap<String, ParamValue>> for ParamValue {
    fn from(m: HashMap<String, ParamValue>) -> Self {
        Self::Map(m)
    }
}

/// A typed key-value map of parameters passed to a command at execution time.
///
/// Supports string, integer, float, boolean, and nested map value types.
///
/// # Examples
///
/// ```
/// use ff_command::CommandParams;
///
/// let mut params = CommandParams::new();
/// params.insert("path", "/tmp/file.txt");
/// params.insert("line", 42i64);
/// assert_eq!(params.get_string("path"), Some("/tmp/file.txt"));
/// assert_eq!(params.get_integer("line"), Some(42));
/// ```
#[derive(Debug, Clone, Default)]
pub struct CommandParams {
    inner: HashMap<String, ParamValue>,
}

impl CommandParams {
    /// Creates a new empty parameter map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<ParamValue>) {
        self.inner.insert(key.into(), value.into());
    }

    /// Returns a builder-style method for chaining insertions.
    pub fn with(mut self, key: impl Into<String>, value: impl Into<ParamValue>) -> Self {
        self.insert(key, value);
        self
    }

    /// Retrieves a value by key.
    pub fn get(&self, key: &str) -> Option<&ParamValue> {
        self.inner.get(key)
    }

    /// Retrieves a string value by key.
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.inner.get(key) {
            Some(ParamValue::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Retrieves an integer value by key.
    pub fn get_integer(&self, key: &str) -> Option<i64> {
        match self.inner.get(key) {
            Some(ParamValue::Integer(v)) => Some(*v),
            _ => None,
        }
    }

    /// Retrieves a float value by key.
    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.inner.get(key) {
            Some(ParamValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    /// Retrieves a boolean value by key.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.inner.get(key) {
            Some(ParamValue::Boolean(v)) => Some(*v),
            _ => None,
        }
    }

    /// Retrieves a nested map value by key.
    pub fn get_map(&self, key: &str) -> Option<&HashMap<String, ParamValue>> {
        match self.inner.get(key) {
            Some(ParamValue::Map(m)) => Some(m),
            _ => None,
        }
    }

    /// Returns true if the parameter map is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of parameters.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns an iterator over the key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ParamValue)> {
        self.inner.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.8
    #[test]
    fn empty_params_reports_empty() {
        let params = CommandParams::new();
        assert!(params.is_empty());
        assert_eq!(params.len(), 0);
    }

    // Validates: Requirement 2.8
    #[test]
    fn insert_and_retrieve_string() {
        let mut params = CommandParams::new();
        params.insert("path", "/tmp/file.txt");
        assert_eq!(params.get_string("path"), Some("/tmp/file.txt"));
    }

    // Validates: Requirement 2.8
    #[test]
    fn insert_and_retrieve_integer() {
        let mut params = CommandParams::new();
        params.insert("line", 42i64);
        assert_eq!(params.get_integer("line"), Some(42));
    }

    // Validates: Requirement 2.8
    #[test]
    fn insert_and_retrieve_float() {
        let mut params = CommandParams::new();
        params.insert("scale", 1.5f64);
        assert_eq!(params.get_float("scale"), Some(1.5));
    }

    // Validates: Requirement 2.8
    #[test]
    fn insert_and_retrieve_boolean() {
        let mut params = CommandParams::new();
        params.insert("force", true);
        assert_eq!(params.get_bool("force"), Some(true));
    }

    // Validates: Requirement 2.8
    #[test]
    fn type_mismatch_returns_none() {
        let mut params = CommandParams::new();
        params.insert("value", 42i64);
        assert_eq!(params.get_string("value"), None);
        assert_eq!(params.get_float("value"), None);
        assert_eq!(params.get_bool("value"), None);
    }

    // Validates: Requirement 2.8
    #[test]
    fn missing_key_returns_none() {
        let params = CommandParams::new();
        assert_eq!(params.get_string("nonexistent"), None);
        assert_eq!(params.get_integer("nonexistent"), None);
    }

    // Validates: Requirement 2.8
    #[test]
    fn builder_style_with_method() {
        let params = CommandParams::new()
            .with("path", "/tmp/file.txt")
            .with("line", 10i64)
            .with("force", true);
        assert_eq!(params.len(), 3);
        assert_eq!(params.get_string("path"), Some("/tmp/file.txt"));
        assert_eq!(params.get_integer("line"), Some(10));
        assert_eq!(params.get_bool("force"), Some(true));
    }

    // Validates: Requirement 2.8
    #[test]
    fn nested_map_value() {
        let mut nested = HashMap::new();
        nested.insert("key".to_string(), ParamValue::String("value".to_string()));

        let mut params = CommandParams::new();
        params.insert("options", ParamValue::Map(nested));

        let map = params.get_map("options").unwrap();
        assert_eq!(
            map.get("key"),
            Some(&ParamValue::String("value".to_string()))
        );
    }
}
