//! Workflow context — typed key-value store shared between steps.
//!
//! The `WorkflowContext` carries state between workflow steps. Values are
//! stored as `ContextValue` enum variants, all of which are serializable
//! for checkpoint/resume support.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Supported value types in the workflow context.
///
/// Used in step declarations to describe expected input/output types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ContextValueType {
    /// A UTF-8 string.
    String,
    /// A 64-bit signed integer.
    Integer,
    /// A 64-bit floating-point number.
    Float,
    /// A boolean.
    Boolean,
    /// Raw bytes.
    Bytes,
    /// A list of strings.
    StringList,
    /// A nested key-value map.
    Map,
    /// Opaque serializable type identified by name.
    Custom(String),
}

/// A value stored in the workflow context.
///
/// All variants are serializable to support checkpoint persistence.
/// Addresses: Requirement 7, criterion 8
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ContextValue {
    /// A UTF-8 string value.
    String(String),
    /// A 64-bit signed integer value.
    Integer(i64),
    /// A 64-bit floating-point value.
    Float(f64),
    /// A boolean value.
    Boolean(bool),
    /// Raw byte data.
    Bytes(Vec<u8>),
    /// A list of strings.
    StringList(Vec<String>),
    /// A nested key-value map.
    Map(HashMap<String, ContextValue>),
    /// An explicit null/absent value.
    Null,
}

impl ContextValue {
    /// Returns the type of this value.
    pub fn value_type(&self) -> ContextValueType {
        match self {
            Self::String(_) => ContextValueType::String,
            Self::Integer(_) => ContextValueType::Integer,
            Self::Float(_) => ContextValueType::Float,
            Self::Boolean(_) => ContextValueType::Boolean,
            Self::Bytes(_) => ContextValueType::Bytes,
            Self::StringList(_) => ContextValueType::StringList,
            Self::Map(_) => ContextValueType::Map,
            Self::Null => ContextValueType::String, // Null is compatible with any type
        }
    }
}

/// A typed key-value store carrying state between workflow steps.
///
/// Steps read inputs from and write outputs to the context. All values
/// are serializable to support checkpoint persistence for long-running
/// workflows.
///
/// Addresses: Requirement 2, criterion 2; Requirement 7, criterion 8
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowContext {
    values: HashMap<String, ContextValue>,
}

impl Default for WorkflowContext {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowContext {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Inserts a value into the context, overwriting any existing value for the key.
    pub fn set(&mut self, key: impl Into<String>, value: ContextValue) {
        self.values.insert(key.into(), value);
    }

    /// Gets a value by key.
    pub fn get(&self, key: &str) -> Option<&ContextValue> {
        self.values.get(key)
    }

    /// Gets a string value by key. Returns `None` if the key is missing or not a string.
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.values.get(key) {
            Some(ContextValue::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Gets an integer value by key. Returns `None` if the key is missing or not an integer.
    pub fn get_integer(&self, key: &str) -> Option<i64> {
        match self.values.get(key) {
            Some(ContextValue::Integer(i)) => Some(*i),
            _ => None,
        }
    }

    /// Gets a boolean value by key. Returns `None` if the key is missing or not a boolean.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.values.get(key) {
            Some(ContextValue::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    /// Gets a float value by key. Returns `None` if the key is missing or not a float.
    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.values.get(key) {
            Some(ContextValue::Float(f)) => Some(*f),
            _ => None,
        }
    }

    /// Checks if a key exists in the context.
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Removes a key and returns its value, or `None` if the key was not present.
    pub fn remove(&mut self, key: &str) -> Option<ContextValue> {
        self.values.remove(key)
    }

    /// Returns all keys in the context.
    pub fn keys(&self) -> Vec<&str> {
        self.values.keys().map(|k| k.as_str()).collect()
    }

    /// Returns the number of entries in the context.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if the context has no entries.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Merges another context into this one. Values from `other` overwrite
    /// on key conflict.
    pub fn merge(&mut self, other: WorkflowContext) {
        self.values.extend(other.values);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.2 — WorkflowContext typed get/set accessors

    #[test]
    fn new_context_is_empty() {
        let ctx = WorkflowContext::new();
        assert!(ctx.is_empty());
        assert_eq!(ctx.len(), 0);
    }

    #[test]
    fn set_and_get_string_value() {
        let mut ctx = WorkflowContext::new();
        ctx.set("name", ContextValue::String("hello".to_string()));
        assert_eq!(ctx.get_string("name"), Some("hello"));
    }

    #[test]
    fn set_and_get_integer_value() {
        let mut ctx = WorkflowContext::new();
        ctx.set("count", ContextValue::Integer(42));
        assert_eq!(ctx.get_integer("count"), Some(42));
    }

    #[test]
    fn set_and_get_boolean_value() {
        let mut ctx = WorkflowContext::new();
        ctx.set("flag", ContextValue::Boolean(true));
        assert_eq!(ctx.get_bool("flag"), Some(true));
    }

    #[test]
    fn set_and_get_float_value() {
        let mut ctx = WorkflowContext::new();
        ctx.set("ratio", ContextValue::Float(3.14));
        assert_eq!(ctx.get_float("ratio"), Some(3.14));
    }

    #[test]
    fn get_missing_key_returns_none() {
        let ctx = WorkflowContext::new();
        assert_eq!(ctx.get("missing"), None);
        assert_eq!(ctx.get_string("missing"), None);
        assert_eq!(ctx.get_integer("missing"), None);
        assert_eq!(ctx.get_bool("missing"), None);
    }

    #[test]
    fn get_wrong_type_returns_none() {
        let mut ctx = WorkflowContext::new();
        ctx.set("key", ContextValue::Integer(10));
        assert_eq!(ctx.get_string("key"), None);
        assert_eq!(ctx.get_bool("key"), None);
    }

    #[test]
    fn overwrite_replaces_value() {
        let mut ctx = WorkflowContext::new();
        ctx.set("key", ContextValue::Integer(1));
        ctx.set("key", ContextValue::Integer(2));
        assert_eq!(ctx.get_integer("key"), Some(2));
    }

    #[test]
    fn contains_key_works() {
        let mut ctx = WorkflowContext::new();
        assert!(!ctx.contains_key("x"));
        ctx.set("x", ContextValue::Null);
        assert!(ctx.contains_key("x"));
    }

    #[test]
    fn remove_returns_value_and_deletes_key() {
        let mut ctx = WorkflowContext::new();
        ctx.set("key", ContextValue::Integer(5));
        let removed = ctx.remove("key");
        assert_eq!(removed, Some(ContextValue::Integer(5)));
        assert!(!ctx.contains_key("key"));
    }

    #[test]
    fn remove_missing_key_returns_none() {
        let mut ctx = WorkflowContext::new();
        assert_eq!(ctx.remove("missing"), None);
    }

    #[test]
    fn merge_combines_contexts_with_other_winning() {
        let mut ctx1 = WorkflowContext::new();
        ctx1.set("a", ContextValue::Integer(1));
        ctx1.set("b", ContextValue::Integer(2));

        let mut ctx2 = WorkflowContext::new();
        ctx2.set("b", ContextValue::Integer(99));
        ctx2.set("c", ContextValue::Integer(3));

        ctx1.merge(ctx2);
        assert_eq!(ctx1.get_integer("a"), Some(1));
        assert_eq!(ctx1.get_integer("b"), Some(99));
        assert_eq!(ctx1.get_integer("c"), Some(3));
    }

    #[test]
    fn keys_returns_all_keys() {
        let mut ctx = WorkflowContext::new();
        ctx.set("alpha", ContextValue::Null);
        ctx.set("beta", ContextValue::Null);
        let mut keys = ctx.keys();
        keys.sort();
        assert_eq!(keys, vec!["alpha", "beta"]);
    }

    #[test]
    fn context_serialization_round_trip() {
        let mut ctx = WorkflowContext::new();
        ctx.set("name", ContextValue::String("test".to_string()));
        ctx.set("count", ContextValue::Integer(42));
        ctx.set("flag", ContextValue::Boolean(false));

        let json = serde_json::to_string(&ctx).expect("serialize");
        let restored: WorkflowContext = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(ctx, restored);
    }
}
