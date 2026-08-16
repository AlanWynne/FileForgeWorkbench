//! ViewerRegistry — central registry of FileViewer implementations.
//!
//! Thread-safe registry mapping ViewerKeys to `Box<dyn FileViewer>` instances.
//! Populated at startup with built-in viewers, extended at runtime by plugins.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::ViewerError;
use crate::key::ViewerKey;
use crate::trait_def::FileViewer;

/// Identifies the origin of a registered viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerSource {
    /// Compiled into the ff-viewers crate.
    BuiltIn,
    /// Contributed by a plugin at runtime.
    Plugin,
}

/// Summary information for a registered viewer (used in LIST output).
#[derive(Debug, Clone)]
pub struct ViewerInfo {
    /// The viewer's unique key.
    pub key: ViewerKey,
    /// Human-readable display name.
    pub display_name: String,
    /// Brief description of what the viewer renders.
    pub description: String,
    /// File extensions the viewer handles.
    pub extensions: Vec<String>,
    /// MIME types the viewer handles.
    pub mime_types: Vec<String>,
    /// Whether this is a built-in or plugin viewer.
    pub source: ViewerSource,
}

/// Internal entry in the viewer registry.
struct ViewerEntry {
    /// The viewer implementation.
    viewer: Box<dyn FileViewer>,
    /// Whether this is a built-in or plugin-contributed viewer.
    source: ViewerSource,
}

/// Thread-safe registry mapping ViewerKeys to FileViewer implementations.
///
/// The registry supports concurrent reads from any thread. Writes (registration
/// and deregistration) acquire an exclusive lock but are expected to be infrequent
/// (startup + plugin lifecycle events).
pub struct ViewerRegistry {
    viewers: Arc<RwLock<HashMap<String, ViewerEntry>>>,
}

impl ViewerRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            viewers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a built-in viewer.
    ///
    /// Called during crate initialization. Validates viewer_key format and
    /// uniqueness before inserting.
    ///
    /// # Errors
    ///
    /// Returns `ViewerError::InvalidKeyFormat` if the key is malformed.
    /// Returns `ViewerError::DuplicateKey` if the key already exists.
    pub fn register_builtin(&self, viewer: Box<dyn FileViewer>) -> Result<(), ViewerError> {
        self.register_internal(viewer, ViewerSource::BuiltIn)
    }

    /// Register a plugin-provided viewer.
    ///
    /// Called from the plugin bridge during a plugin's `initialize` phase.
    /// Validates viewer_key format and uniqueness before inserting.
    ///
    /// # Errors
    ///
    /// Returns `ViewerError::InvalidKeyFormat` if the key is malformed.
    /// Returns `ViewerError::DuplicateKey` if the key already exists.
    pub fn register_plugin(&self, viewer: Box<dyn FileViewer>) -> Result<(), ViewerError> {
        self.register_internal(viewer, ViewerSource::Plugin)
    }

    /// Deregister a viewer by key. Returns `true` if the viewer existed and
    /// was removed, `false` if the key was not found.
    pub fn deregister(&self, key: &ViewerKey) -> bool {
        let mut map = self.viewers.write().expect("registry lock poisoned");
        map.remove(key.as_str()).is_some()
    }

    /// Look up a viewer by key. Returns a clone of the viewer's key and
    /// rendered output capability check result, or `None` if not found.
    ///
    /// Since we cannot return a reference through an `RwLock`, this method
    /// executes a callback with the viewer reference while holding the read lock.
    pub fn with_viewer<F, R>(&self, key: &ViewerKey, f: F) -> Option<R>
    where
        F: FnOnce(&dyn FileViewer) -> R,
    {
        let map = self.viewers.read().expect("registry lock poisoned");
        map.get(key.as_str()).map(|entry| f(entry.viewer.as_ref()))
    }

    /// Returns whether a viewer key is currently registered.
    pub fn contains(&self, key: &ViewerKey) -> bool {
        let map = self.viewers.read().expect("registry lock poisoned");
        map.contains_key(key.as_str())
    }

    /// Returns the number of registered viewers.
    pub fn viewer_count(&self) -> usize {
        let map = self.viewers.read().expect("registry lock poisoned");
        map.len()
    }

    /// List all registered viewers with metadata.
    ///
    /// Returns a vector of `ViewerInfo` structs containing each viewer's key,
    /// display name, description, supported extensions, MIME types, and source.
    pub fn list_viewers(&self) -> Vec<ViewerInfo> {
        let map = self.viewers.read().expect("registry lock poisoned");
        map.iter()
            .map(|(key_str, entry)| {
                let viewer = entry.viewer.as_ref();
                ViewerInfo {
                    key: ViewerKey::new(key_str).expect("registered key should always be valid"),
                    display_name: viewer.display_name().to_string(),
                    description: viewer.description().to_string(),
                    extensions: viewer
                        .supported_extensions()
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    mime_types: viewer
                        .supported_mime_types()
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    source: entry.source,
                }
            })
            .collect()
    }

    /// Returns all viewer keys whose `supported_extensions` include the given extension.
    pub fn viewers_for_extension(&self, ext: &str) -> Vec<ViewerKey> {
        let ext_lower = ext.to_lowercase();
        let map = self.viewers.read().expect("registry lock poisoned");
        map.iter()
            .filter(|(_, entry)| {
                entry
                    .viewer
                    .supported_extensions()
                    .iter()
                    .any(|e| e.to_lowercase() == ext_lower)
            })
            .filter_map(|(key_str, _)| ViewerKey::new(key_str).ok())
            .collect()
    }

    /// Returns all viewer keys whose `supported_mime_types` include the given MIME type.
    pub fn viewers_for_mime(&self, mime: &str) -> Vec<ViewerKey> {
        let mime_lower = mime.to_lowercase();
        let map = self.viewers.read().expect("registry lock poisoned");
        map.iter()
            .filter(|(_, entry)| {
                entry
                    .viewer
                    .supported_mime_types()
                    .iter()
                    .any(|m| m.to_lowercase() == mime_lower)
            })
            .filter_map(|(key_str, _)| ViewerKey::new(key_str).ok())
            .collect()
    }

    /// Returns the source (BuiltIn or Plugin) for a given viewer key.
    pub fn viewer_source(&self, key: &ViewerKey) -> Option<ViewerSource> {
        let map = self.viewers.read().expect("registry lock poisoned");
        map.get(key.as_str()).map(|entry| entry.source)
    }

    /// Deregister all plugin-provided viewers. Returns keys that were removed.
    pub fn deregister_all_plugin_viewers(&self) -> Vec<ViewerKey> {
        let mut map = self.viewers.write().expect("registry lock poisoned");
        let plugin_keys: Vec<String> = map
            .iter()
            .filter(|(_, entry)| entry.source == ViewerSource::Plugin)
            .map(|(key, _)| key.clone())
            .collect();

        let mut removed = Vec::new();
        for key_str in plugin_keys {
            map.remove(&key_str);
            if let Ok(key) = ViewerKey::new(&key_str) {
                removed.push(key);
            }
        }
        removed
    }

    /// Internal registration implementation shared by built-in and plugin paths.
    fn register_internal(
        &self,
        viewer: Box<dyn FileViewer>,
        source: ViewerSource,
    ) -> Result<(), ViewerError> {
        let key_str = viewer.viewer_key().to_string();

        // Validate key format
        let _key = ViewerKey::new(&key_str)?;

        let mut map = self.viewers.write().expect("registry lock poisoned");

        if map.contains_key(&key_str) {
            return Err(ViewerError::DuplicateKey { key: key_str });
        }

        map.insert(key_str, ViewerEntry { viewer, source });
        Ok(())
    }
}

impl Default for ViewerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple test viewer for registry tests.
    struct StubViewer {
        key: String,
        name: String,
        extensions: Vec<&'static str>,
        mime_types: Vec<&'static str>,
    }

    impl StubViewer {
        fn new(key: &str) -> Self {
            Self {
                key: key.to_string(),
                name: format!("{key} Viewer"),
                extensions: vec![],
                mime_types: vec![],
            }
        }

        fn with_extensions(mut self, exts: Vec<&'static str>) -> Self {
            self.extensions = exts;
            self
        }

        fn with_mime_types(mut self, mimes: Vec<&'static str>) -> Self {
            self.mime_types = mimes;
            self
        }
    }

    impl FileViewer for StubViewer {
        fn viewer_key(&self) -> &str {
            &self.key
        }

        fn display_name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "A stub viewer for testing"
        }

        fn supported_extensions(&self) -> &[&str] {
            &self.extensions
        }

        fn supported_mime_types(&self) -> &[&str] {
            &self.mime_types
        }

        fn can_render(&self, _uri: &str, _content_sample: &[u8]) -> bool {
            false
        }

        fn render(&self, content: &[u8]) -> String {
            String::from_utf8_lossy(content).to_string()
        }

        fn on_content_changed(&mut self, _new_content: &[u8]) {}
    }

    #[test]
    fn register_and_lookup_builtin_viewer() {
        // Validates: Requirement 1 AC 1, AC 3
        let registry = ViewerRegistry::new();
        let viewer = Box::new(StubViewer::new("hex"));
        registry.register_builtin(viewer).unwrap();

        let key = ViewerKey::new("hex").unwrap();
        assert!(registry.contains(&key));
        assert_eq!(registry.viewer_count(), 1);
    }

    #[test]
    fn register_plugin_viewer() {
        // Validates: Requirement 1 AC 4
        let registry = ViewerRegistry::new();
        let viewer = Box::new(StubViewer::new("custom-viewer"));
        registry.register_plugin(viewer).unwrap();

        let key = ViewerKey::new("custom-viewer").unwrap();
        assert!(registry.contains(&key));
        assert_eq!(registry.viewer_source(&key), Some(ViewerSource::Plugin));
    }

    #[test]
    fn duplicate_key_rejected() {
        // Validates: Requirement 1 AC 6
        let registry = ViewerRegistry::new();
        registry
            .register_builtin(Box::new(StubViewer::new("hex")))
            .unwrap();

        let result = registry.register_builtin(Box::new(StubViewer::new("hex")));
        assert!(result.is_err());
        match result.unwrap_err() {
            ViewerError::DuplicateKey { key } => assert_eq!(key, "hex"),
            other => panic!("Expected DuplicateKey, got: {other:?}"),
        }
    }

    #[test]
    fn invalid_key_format_rejected_on_register() {
        // Validates: Requirement 1 AC 1
        let registry = ViewerRegistry::new();

        struct BadViewer;
        impl FileViewer for BadViewer {
            fn viewer_key(&self) -> &str {
                "INVALID KEY"
            }
            fn display_name(&self) -> &str {
                "Bad"
            }
            fn description(&self) -> &str {
                "bad"
            }
            fn supported_extensions(&self) -> &[&str] {
                &[]
            }
            fn supported_mime_types(&self) -> &[&str] {
                &[]
            }
            fn can_render(&self, _: &str, _: &[u8]) -> bool {
                false
            }
            fn render(&self, _: &[u8]) -> String {
                String::new()
            }
            fn on_content_changed(&mut self, _: &[u8]) {}
        }

        let result = registry.register_builtin(Box::new(BadViewer));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ViewerError::InvalidKeyFormat { .. }
        ));
    }

    #[test]
    fn deregister_removes_viewer() {
        // Validates: Requirement 1 AC 5
        let registry = ViewerRegistry::new();
        registry
            .register_builtin(Box::new(StubViewer::new("hex")))
            .unwrap();

        let key = ViewerKey::new("hex").unwrap();
        assert!(registry.deregister(&key));
        assert!(!registry.contains(&key));
        assert_eq!(registry.viewer_count(), 0);
    }

    #[test]
    fn deregister_nonexistent_returns_false() {
        let registry = ViewerRegistry::new();
        let key = ViewerKey::new("nonexistent").unwrap();
        assert!(!registry.deregister(&key));
    }

    #[test]
    fn list_viewers_returns_all_registered() {
        // Validates: Requirement 1 AC 7
        let registry = ViewerRegistry::new();
        registry
            .register_builtin(Box::new(
                StubViewer::new("hex").with_extensions(vec!["bin"]),
            ))
            .unwrap();
        registry
            .register_builtin(Box::new(
                StubViewer::new("csv-table").with_extensions(vec!["csv", "tsv"]),
            ))
            .unwrap();

        let list = registry.list_viewers();
        assert_eq!(list.len(), 2);

        let keys: Vec<&str> = list.iter().map(|info| info.key.as_str()).collect();
        assert!(keys.contains(&"hex"));
        assert!(keys.contains(&"csv-table"));
    }

    #[test]
    fn viewers_for_extension_finds_matching() {
        let registry = ViewerRegistry::new();
        registry
            .register_builtin(Box::new(
                StubViewer::new("csv-table").with_extensions(vec!["csv", "tsv"]),
            ))
            .unwrap();
        registry
            .register_builtin(Box::new(
                StubViewer::new("hex").with_extensions(vec!["bin"]),
            ))
            .unwrap();

        let matches = registry.viewers_for_extension("csv");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].as_str(), "csv-table");
    }

    #[test]
    fn viewers_for_mime_finds_matching() {
        let registry = ViewerRegistry::new();
        registry
            .register_builtin(Box::new(
                StubViewer::new("csv-table").with_mime_types(vec!["text/csv"]),
            ))
            .unwrap();

        let matches = registry.viewers_for_mime("text/csv");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].as_str(), "csv-table");
    }

    #[test]
    fn with_viewer_executes_callback() {
        let registry = ViewerRegistry::new();
        registry
            .register_builtin(Box::new(StubViewer::new("hex")))
            .unwrap();

        let key = ViewerKey::new("hex").unwrap();
        let name = registry.with_viewer(&key, |v| v.display_name().to_string());
        assert_eq!(name, Some("hex Viewer".to_string()));
    }

    #[test]
    fn with_viewer_returns_none_for_missing_key() {
        let registry = ViewerRegistry::new();
        let key = ViewerKey::new("missing").unwrap();
        let result = registry.with_viewer(&key, |v| v.display_name().to_string());
        assert_eq!(result, None);
    }

    #[test]
    fn deregister_all_plugin_viewers_removes_only_plugins() {
        let registry = ViewerRegistry::new();
        registry
            .register_builtin(Box::new(StubViewer::new("hex")))
            .unwrap();
        registry
            .register_plugin(Box::new(StubViewer::new("custom-a")))
            .unwrap();
        registry
            .register_plugin(Box::new(StubViewer::new("custom-b")))
            .unwrap();

        let removed = registry.deregister_all_plugin_viewers();
        assert_eq!(removed.len(), 2);
        assert_eq!(registry.viewer_count(), 1);

        let hex_key = ViewerKey::new("hex").unwrap();
        assert!(registry.contains(&hex_key));
    }

    #[test]
    fn thread_safety_concurrent_reads() {
        // Validates: Requirement 1 AC 2
        let registry = Arc::new(ViewerRegistry::new());
        registry
            .register_builtin(Box::new(StubViewer::new("hex")))
            .unwrap();

        let key = ViewerKey::new("hex").unwrap();
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let reg = Arc::clone(&registry);
                let k = key.clone();
                std::thread::spawn(move || reg.contains(&k))
            })
            .collect();

        for handle in handles {
            assert!(handle.join().unwrap());
        }
    }

    #[test]
    fn viewer_source_returns_correct_source() {
        let registry = ViewerRegistry::new();
        registry
            .register_builtin(Box::new(StubViewer::new("hex")))
            .unwrap();
        registry
            .register_plugin(Box::new(StubViewer::new("custom")))
            .unwrap();

        let hex_key = ViewerKey::new("hex").unwrap();
        let custom_key = ViewerKey::new("custom").unwrap();

        assert_eq!(
            registry.viewer_source(&hex_key),
            Some(ViewerSource::BuiltIn)
        );
        assert_eq!(
            registry.viewer_source(&custom_key),
            Some(ViewerSource::Plugin)
        );
    }
}
