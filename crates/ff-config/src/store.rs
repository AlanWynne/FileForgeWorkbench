//! Effective configuration store.
//!
//! Holds the merged, validated set of effective configuration values along
//! with their provenance metadata. Provides the backing data for the typed
//! access API.

use std::collections::BTreeMap;

use crate::provenance::EffectiveValue;
use crate::value::ConfigValue;

/// The effective configuration store after all layers have been merged.
/// Maps flattened dot-path keys to their effective values with provenance.
#[derive(Debug, Clone)]
pub struct EffectiveStore {
    /// All effective values keyed by their full dot-path (e.g., "editor.tab_size").
    entries: BTreeMap<String, EffectiveValue>,
}

impl EffectiveStore {
    /// Create a new, empty effective store.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Get the effective value for a key.
    pub fn get(&self, key: &str) -> Option<&EffectiveValue> {
        self.entries.get(key)
    }

    /// Get just the value (without provenance).
    pub fn get_value(&self, key: &str) -> Option<&ConfigValue> {
        self.entries.get(key).map(|e| &e.value)
    }

    /// Insert or overwrite an effective value.
    pub fn insert(&mut self, key: String, effective: EffectiveValue) {
        self.entries.insert(key, effective);
    }

    /// Returns the number of effective entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns all keys in the store.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }
}

impl Default for EffectiveStore {
    fn default() -> Self {
        Self::new()
    }
}
