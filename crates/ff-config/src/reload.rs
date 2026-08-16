//! Hot-reload coordination.
//!
//! Orchestrates the reload pipeline: re-read changed file → re-merge layers →
//! diff effective values → invoke callbacks for changed keys. Ensures atomic
//! application of changes from a single file.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use crate::callback::CallbackRegistry;
use crate::error::ConfigError;
use crate::layer::ConfigLayer;
use crate::loader::{load_toml_file, LayerData};
use crate::merger::merge_layers;
use crate::schema::SchemaRegistry;
use crate::store::EffectiveStore;
use crate::watcher::ConfigWatcher;

/// Event describing what changed during a reload.
///
/// Contains the set of changed keys, the layer that was reloaded, and
/// the timestamp of the reload event.
#[derive(Debug, Clone)]
pub struct ReloadEvent {
    /// Keys whose effective value changed.
    pub changed_keys: Vec<String>,
    /// The layer that was reloaded.
    pub source_layer: ConfigLayer,
    /// When the reload occurred.
    pub timestamp: SystemTime,
}

/// Manages the hot-reload lifecycle for configuration files.
///
/// Holds all loaded layer data, the current effective store, and the schema
/// registry. Provides methods to reload individual files or all files,
/// computing diffs and returning events for changed keys.
pub struct ReloadManager {
    /// All loaded layer data (mutable for re-loading individual layers).
    layers: Vec<LayerData>,
    /// The current effective store.
    current_store: EffectiveStore,
    /// Schema registry for validation and default fallback.
    schema: SchemaRegistry,
    /// Callback registry for invoking reload callbacks on changed keys.
    callbacks: Arc<CallbackRegistry>,
    /// Optional file watcher for hot-reload monitoring.
    watcher: Option<ConfigWatcher>,
}

impl ReloadManager {
    /// Create a new ReloadManager with initial layers and schema.
    ///
    /// Immediately merges the provided layers against the schema to produce
    /// the initial effective store.
    pub fn new(layers: Vec<LayerData>, schema: SchemaRegistry) -> Self {
        let current_store = merge_layers(&layers, &schema);
        Self {
            layers,
            current_store,
            schema,
            callbacks: Arc::new(CallbackRegistry::new()),
            watcher: None,
        }
    }

    /// Create a new ReloadManager with initial layers, schema, and a shared callback registry.
    ///
    /// Use this constructor when you need to share the callback registry with
    /// other components (e.g., for registering callbacks externally).
    pub fn with_callbacks(
        layers: Vec<LayerData>,
        schema: SchemaRegistry,
        callbacks: Arc<CallbackRegistry>,
    ) -> Self {
        let current_store = merge_layers(&layers, &schema);
        Self {
            layers,
            current_store,
            schema,
            callbacks,
            watcher: None,
        }
    }

    /// Get a reference to the current effective store.
    pub fn store(&self) -> &EffectiveStore {
        &self.current_store
    }

    /// Get a reference to the schema registry.
    pub fn schema(&self) -> &SchemaRegistry {
        &self.schema
    }

    /// Get a mutable reference to the schema registry.
    pub fn schema_mut(&mut self) -> &mut SchemaRegistry {
        &mut self.schema
    }

    /// Get a reference to the shared callback registry.
    ///
    /// Use this to register or deregister reload callbacks externally.
    pub fn callbacks(&self) -> &Arc<CallbackRegistry> {
        &self.callbacks
    }

    /// Set the file watcher for hot-reload monitoring.
    ///
    /// When a watcher is set, `load_project` and `unload_project` will
    /// automatically register/unregister the project config file path
    /// with the watcher for change detection.
    pub fn set_watcher(&mut self, watcher: ConfigWatcher) {
        self.watcher = Some(watcher);
    }

    /// Returns a mutable reference to the file watcher, if one is set.
    pub fn watcher_mut(&mut self) -> Option<&mut ConfigWatcher> {
        self.watcher.as_mut()
    }

    /// Returns a reference to the file watcher, if one is set.
    pub fn watcher(&self) -> Option<&ConfigWatcher> {
        self.watcher.as_ref()
    }

    /// Take ownership of the file watcher, removing it from the manager.
    ///
    /// Returns the watcher if one was set, or `None` if no watcher was active.
    /// After this call, the manager has no watcher.
    pub fn take_watcher(&mut self) -> Option<ConfigWatcher> {
        self.watcher.take()
    }

    /// Reload a single file that belongs to a specific layer.
    ///
    /// Pipeline: re-read → parse TOML → re-merge all layers → diff → return event.
    /// If parsing fails, the reload is rejected and previous values are retained.
    ///
    /// Returns `Ok(Some(ReloadEvent))` on success with changed keys.
    /// Returns `Ok(None)` if no keys changed or if the file had invalid TOML
    /// (reload rejected, previous values retained, WARN log emitted).
    /// Returns `Err` on I/O errors that prevent reading the file.
    pub fn reload_file(
        &mut self,
        path: &PathBuf,
        layer: ConfigLayer,
    ) -> Result<Option<ReloadEvent>, ConfigError> {
        // Step 1: Re-read and parse the file
        let new_values = match load_toml_file(path) {
            Ok(table) => table,
            Err(ConfigError::ParseError { path: p, details }) => {
                // Reject reload — retain previous values, emit WARN log
                ff_logging::log_warn!(
                    "[config] reload: file '{}' has invalid TOML: {} — retaining previous values",
                    p.display(),
                    details
                );
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        // Step 2: Build new layer data (atomic: replace the whole layer at once)
        let new_layer_data = LayerData {
            layer,
            source_path: path.clone(),
            values: new_values,
        };

        // Find and replace the existing layer data, or add new
        if let Some(existing) = self
            .layers
            .iter_mut()
            .find(|l| l.layer == layer && l.source_path == *path)
        {
            *existing = new_layer_data;
        } else {
            self.layers.push(new_layer_data);
        }

        // Step 3: Re-merge all layers to produce new effective store
        let new_store = merge_layers(&self.layers, &self.schema);

        // Step 4: Compute diff (keys that changed)
        let changed_keys = compute_diff(&self.current_store, &new_store);

        if changed_keys.is_empty() {
            return Ok(None);
        }

        // Step 5: Atomic swap — replace old store with new
        self.current_store = new_store;

        // Step 6: Build ReloadEvent and invoke callbacks
        let event = ReloadEvent {
            changed_keys,
            source_layer: layer,
            timestamp: SystemTime::now(),
        };

        self.callbacks.invoke(&event);

        Ok(Some(event))
    }

    /// Load project-layer configuration from the given project root.
    ///
    /// Detects `.ffworkbench/config.toml` in `project_root`, loads and parses
    /// it, inserts or replaces the Project layer in the layer stack, re-merges
    /// all layers, and returns a `ReloadEvent` with the set of keys that changed.
    ///
    /// If the project config file does not exist, returns `Ok(ReloadEvent)` with
    /// an empty `changed_keys` (no-op, project layer not added).
    ///
    /// # Errors
    ///
    /// - `ConfigError::Io` if the file exists but cannot be read.
    /// - `ConfigError::ParseError` if the file contains invalid TOML syntax.
    ///
    /// Addresses: Requirement 5, criteria 1/2
    pub fn load_project(&mut self, project_root: &Path) -> Result<ReloadEvent, ConfigError> {
        let config_path = crate::paths::project_config_path(project_root);

        if !config_path.exists() {
            return Ok(ReloadEvent {
                changed_keys: Vec::new(),
                source_layer: ConfigLayer::Project,
                timestamp: SystemTime::now(),
            });
        }

        let values = load_toml_file(&config_path)?;

        let new_layer_data = LayerData {
            layer: ConfigLayer::Project,
            source_path: config_path.clone(),
            values,
        };

        // Replace existing Project layer if present, otherwise add new
        if let Some(existing) = self
            .layers
            .iter_mut()
            .find(|l| l.layer == ConfigLayer::Project)
        {
            *existing = new_layer_data;
        } else {
            self.layers.push(new_layer_data);
        }

        // Re-merge all layers to produce new effective store
        let new_store = merge_layers(&self.layers, &self.schema);

        // Compute diff
        let changed_keys = compute_diff(&self.current_store, &new_store);

        // Atomic swap
        self.current_store = new_store;

        // Register the project config file with the watcher for hot-reload monitoring
        if let Some(ref mut watcher) = self.watcher {
            if let Err(e) = watcher.watch(&config_path) {
                ff_logging::log_warn!(
                    "[config] load_project: failed to watch project config '{}': {}",
                    config_path.display(),
                    e
                );
            }
        }

        let event = ReloadEvent {
            changed_keys,
            source_layer: ConfigLayer::Project,
            timestamp: SystemTime::now(),
        };

        // Invoke registered callbacks for changed keys
        if !event.changed_keys.is_empty() {
            self.callbacks.invoke(&event);
        }

        Ok(event)
    }

    /// Unload the project-layer configuration.
    ///
    /// Removes the Project layer from the layer stack, re-merges remaining
    /// layers, computes the set of keys whose effective value changed, invokes
    /// registered Reload_Callbacks for those keys, and returns a `ReloadEvent`.
    ///
    /// If no Project layer is currently loaded, returns an event with empty
    /// `changed_keys`.
    ///
    /// Addresses: Requirement 5, criterion 6
    pub fn unload_project(&mut self) -> ReloadEvent {
        let had_project = self.layers.iter().any(|l| l.layer == ConfigLayer::Project);

        // Capture the project config path before removing the layer (for unwatching)
        let project_config_path: Option<PathBuf> = self
            .layers
            .iter()
            .find(|l| l.layer == ConfigLayer::Project)
            .map(|l| l.source_path.clone());

        // Remove Project layer(s)
        self.layers.retain(|l| l.layer != ConfigLayer::Project);

        if !had_project {
            return ReloadEvent {
                changed_keys: Vec::new(),
                source_layer: ConfigLayer::Project,
                timestamp: SystemTime::now(),
            };
        }

        // Unregister the project config file from the watcher
        if let (Some(ref mut watcher), Some(ref path)) = (&mut self.watcher, &project_config_path) {
            if let Err(e) = watcher.unwatch(path) {
                ff_logging::log_warn!(
                    "[config] unload_project: failed to unwatch project config '{}': {}",
                    path.display(),
                    e
                );
            }
        }

        // Re-merge remaining layers
        let new_store = merge_layers(&self.layers, &self.schema);

        // Compute diff
        let changed_keys = compute_diff(&self.current_store, &new_store);

        // Atomic swap
        self.current_store = new_store;

        let event = ReloadEvent {
            changed_keys,
            source_layer: ConfigLayer::Project,
            timestamp: SystemTime::now(),
        };

        // Invoke registered callbacks for changed keys
        if !event.changed_keys.is_empty() {
            self.callbacks.invoke(&event);
        }

        event
    }

    /// Automatically detect and load project-layer configuration.
    ///
    /// This is the "automatic detection" entry point intended for use during
    /// initialization or when a project is opened at runtime. Unlike
    /// `load_project()`, this method handles errors gracefully:
    ///
    /// - If `.ffworkbench/config.toml` does not exist → no-op (empty event)
    /// - If the file exists but contains invalid TOML → logs WARN, skips project layer
    /// - If the file exists but cannot be read (I/O error) → logs WARN, skips project layer
    ///
    /// Callers simply say "I opened this project" and the config system handles
    /// detection and loading seamlessly without requiring explicit error handling.
    ///
    /// Returns a `ReloadEvent` with the set of changed keys (empty if no config
    /// was loaded or if detection found no config file).
    ///
    /// Addresses: Requirement 5, criterion 2 (automatic detection and load)
    /// Addresses: Requirement 5, criterion 7 (graceful failure handling)
    pub fn open_project(&mut self, project_root: &Path) -> ReloadEvent {
        match self.load_project(project_root) {
            Ok(event) => event,
            Err(ConfigError::ParseError { path, details }) => {
                ff_logging::log_warn!(
                    "[config] open_project: project config '{}' has invalid TOML: {} — skipping project layer",
                    path.display(),
                    details
                );
                ReloadEvent {
                    changed_keys: Vec::new(),
                    source_layer: ConfigLayer::Project,
                    timestamp: SystemTime::now(),
                }
            }
            Err(ConfigError::Io(io_err)) => {
                ff_logging::log_warn!(
                    "[config] open_project: cannot read project config: {} — skipping project layer",
                    io_err
                );
                ReloadEvent {
                    changed_keys: Vec::new(),
                    source_layer: ConfigLayer::Project,
                    timestamp: SystemTime::now(),
                }
            }
            Err(other) => {
                ff_logging::log_warn!(
                    "[config] open_project: unexpected error loading project config: {} — skipping project layer",
                    other
                );
                ReloadEvent {
                    changed_keys: Vec::new(),
                    source_layer: ConfigLayer::Project,
                    timestamp: SystemTime::now(),
                }
            }
        }
    }

    /// Returns whether a project layer is currently loaded.
    pub fn has_project_layer(&self) -> bool {
        self.layers.iter().any(|l| l.layer == ConfigLayer::Project)
    }

    /// Returns the source path of the currently loaded project layer, if any.
    pub fn project_source_path(&self) -> Option<&Path> {
        self.layers
            .iter()
            .find(|l| l.layer == ConfigLayer::Project)
            .map(|l| l.source_path.as_path())
    }

    /// Reload all layer files. Returns events for each layer that had changes.
    ///
    /// Iterates over all currently loaded layers and re-reads each file.
    /// Each layer is reloaded independently; failures in one layer do not
    /// prevent reloading others.
    pub fn reload_all(&mut self) -> Vec<Result<Option<ReloadEvent>, ConfigError>> {
        let paths_and_layers: Vec<(PathBuf, ConfigLayer)> = self
            .layers
            .iter()
            .map(|l| (l.source_path.clone(), l.layer))
            .collect();

        let mut results = Vec::new();
        for (path, layer) in paths_and_layers {
            results.push(self.reload_file(&path, layer));
        }
        results
    }
}

/// Compute the set of keys whose values differ between two stores.
///
/// Detects added, changed, and removed keys by comparing old and new stores.
fn compute_diff(old: &EffectiveStore, new: &EffectiveStore) -> Vec<String> {
    let mut changed = Vec::new();

    // Check all keys in the new store for additions or changes
    for key in new.keys() {
        match (old.get_value(key), new.get_value(key)) {
            (Some(old_val), Some(new_val)) if old_val != new_val => {
                changed.push(key.clone());
            }
            (None, Some(_)) => {
                changed.push(key.clone()); // New key appeared
            }
            _ => {}
        }
    }

    // Check for keys removed (in old but not in new)
    for key in old.keys() {
        if new.get_value(key).is_none() {
            changed.push(key.clone());
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ConfigValue;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper: write TOML content to a temp file and return path.
    fn write_toml_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    // ========================================================================
    // 12.1 — Reload pipeline: re-read → parse → re-merge → diff → event
    // ========================================================================

    // Validates: Requirement 3.2 — Successful reload changes effective values
    #[test]
    fn reload_file_updates_effective_values_and_returns_event() {
        let dir = TempDir::new().unwrap();
        let path = write_toml_file(&dir, "user.toml", "[editor]\ntab_size = 4\n");

        // Build initial layer data by loading the file
        let initial_values = load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path.clone(),
            values: initial_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Verify initial state
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(4))
        );

        // Modify the file on disk
        std::fs::write(&path, "[editor]\ntab_size = 2\n").unwrap();

        // Reload the file
        let result = manager.reload_file(&path, ConfigLayer::User);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert!(event.is_some(), "Should return event when values change");

        let event = event.unwrap();
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
        assert_eq!(event.source_layer, ConfigLayer::User);

        // Verify effective value is updated
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2))
        );
    }

    // Validates: Requirement 3.3 — ReloadEvent contains correct changed_keys
    #[test]
    fn reload_event_has_correct_changed_keys() {
        let dir = TempDir::new().unwrap();
        let path = write_toml_file(
            &dir,
            "project.toml",
            "[editor]\ntab_size = 4\nword_wrap = true\n",
        );

        let initial_values = load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::Project,
            source_path: path.clone(),
            values: initial_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Change tab_size and add a new key, keep word_wrap the same
        std::fs::write(
            &path,
            "[editor]\ntab_size = 8\nword_wrap = true\nfont_size = 14\n",
        )
        .unwrap();

        let event = manager
            .reload_file(&path, ConfigLayer::Project)
            .unwrap()
            .unwrap();

        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
        assert!(event.changed_keys.contains(&"editor.font_size".to_string()));
        // word_wrap didn't change, should NOT be in changed_keys
        assert!(!event.changed_keys.contains(&"editor.word_wrap".to_string()));
    }

    // ========================================================================
    // 12.2 — Atomic change application
    // ========================================================================

    // Validates: Requirement 3.5 — All changed values applied together
    #[test]
    fn atomic_apply_all_values_visible_together_after_reload() {
        let dir = TempDir::new().unwrap();
        let path = write_toml_file(
            &dir,
            "workspace.toml",
            "[editor]\ntab_size = 4\nindent_style = \"tab\"\n[logging]\nlevel = \"info\"\n",
        );

        let initial_values = load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::Workspace,
            source_path: path.clone(),
            values: initial_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Change multiple values at once
        std::fs::write(
            &path,
            "[editor]\ntab_size = 2\nindent_style = \"space\"\n[logging]\nlevel = \"debug\"\n",
        )
        .unwrap();

        let event = manager
            .reload_file(&path, ConfigLayer::Workspace)
            .unwrap()
            .unwrap();

        // All three changed values should be visible in the store simultaneously
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2))
        );
        assert_eq!(
            manager.store().get_value("editor.indent_style"),
            Some(&ConfigValue::String("space".to_string()))
        );
        assert_eq!(
            manager.store().get_value("logging.level"),
            Some(&ConfigValue::String("debug".to_string()))
        );

        // Event should record all three changes
        assert_eq!(event.changed_keys.len(), 3);
    }

    // ========================================================================
    // 12.3 — Reload failure handling
    // ========================================================================

    // Validates: Requirement 3.6 — Invalid TOML: reject reload, retain previous values
    #[test]
    fn reload_with_invalid_toml_retains_previous_values() {
        let dir = TempDir::new().unwrap();
        let path = write_toml_file(&dir, "user.toml", "[editor]\ntab_size = 4\n");

        let initial_values = load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path.clone(),
            values: initial_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Write invalid TOML to the file
        std::fs::write(&path, "this is not valid TOML [[[").unwrap();

        // Reload should return Ok(None) — rejected, no changes
        let result = manager.reload_file(&path, ConfigLayer::User);
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_none(),
            "Invalid TOML should be rejected with Ok(None)"
        );

        // Previous values should still be intact
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(4))
        );
    }

    // Validates: Requirement 5.7 — I/O error propagated for missing file
    #[test]
    fn reload_with_missing_file_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = write_toml_file(&dir, "user.toml", "[editor]\ntab_size = 4\n");

        let initial_values = load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path.clone(),
            values: initial_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Delete the file
        std::fs::remove_file(&path).unwrap();

        // Reload should return an I/O error
        let result = manager.reload_file(&path, ConfigLayer::User);
        assert!(result.is_err());
    }

    // ========================================================================
    // 12.4 — ReloadEvent struct
    // ========================================================================

    // Validates: Requirement 3.3 — ReloadEvent has changed_keys, source_layer, timestamp
    #[test]
    fn reload_event_struct_has_required_fields() {
        let event = ReloadEvent {
            changed_keys: vec!["editor.tab_size".to_string(), "logging.level".to_string()],
            source_layer: ConfigLayer::User,
            timestamp: SystemTime::now(),
        };

        assert_eq!(event.changed_keys.len(), 2);
        assert_eq!(event.source_layer, ConfigLayer::User);
        // Timestamp should be reasonably recent
        assert!(event.timestamp.elapsed().unwrap().as_secs() < 5);
    }

    // Validates: Requirement 3.5 — ReloadEvent is cloneable
    #[test]
    fn reload_event_is_cloneable() {
        let event = ReloadEvent {
            changed_keys: vec!["editor.tab_size".to_string()],
            source_layer: ConfigLayer::Project,
            timestamp: SystemTime::now(),
        };

        let cloned = event.clone();
        assert_eq!(cloned.changed_keys, event.changed_keys);
        assert_eq!(cloned.source_layer, event.source_layer);
    }

    // ========================================================================
    // 12.5 — reload_all() method
    // ========================================================================

    // Validates: Requirement 3.2 — reload_all processes all layers
    #[test]
    fn reload_all_processes_all_layers() {
        let dir = TempDir::new().unwrap();
        let user_path = write_toml_file(&dir, "user.toml", "[editor]\ntab_size = 4\n");
        let project_path = write_toml_file(&dir, "project.toml", "[logging]\nlevel = \"info\"\n");

        let user_values = load_toml_file(&user_path).unwrap();
        let project_values = load_toml_file(&project_path).unwrap();

        let layers = vec![
            LayerData {
                layer: ConfigLayer::User,
                source_path: user_path.clone(),
                values: user_values,
            },
            LayerData {
                layer: ConfigLayer::Project,
                source_path: project_path.clone(),
                values: project_values,
            },
        ];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Modify both files
        std::fs::write(&user_path, "[editor]\ntab_size = 2\n").unwrap();
        std::fs::write(&project_path, "[logging]\nlevel = \"debug\"\n").unwrap();

        let results = manager.reload_all();
        assert_eq!(results.len(), 2);

        // Both should succeed with events
        for result in &results {
            assert!(result.is_ok());
        }

        // Verify both stores updated
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2))
        );
        assert_eq!(
            manager.store().get_value("logging.level"),
            Some(&ConfigValue::String("debug".to_string()))
        );
    }

    // ========================================================================
    // 12.6 — Additional edge case tests
    // ========================================================================

    // Validates: Requirement 3.2 — No-change reload returns Ok(None)
    #[test]
    fn no_change_reload_returns_none() {
        let dir = TempDir::new().unwrap();
        let path = write_toml_file(&dir, "user.toml", "[editor]\ntab_size = 4\n");

        let initial_values = load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path.clone(),
            values: initial_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Reload without modifying the file — content is the same
        let result = manager.reload_file(&path, ConfigLayer::User);
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_none(),
            "Reload with no changes should return None"
        );
    }

    // Validates: Requirement 3.5 — compute_diff detects added, changed, and removed keys
    #[test]
    fn compute_diff_detects_added_changed_and_removed_keys() {
        let dir = TempDir::new().unwrap();
        let path = write_toml_file(
            &dir,
            "config.toml",
            "[editor]\ntab_size = 4\nword_wrap = true\n",
        );

        let initial_values = load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path.clone(),
            values: initial_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // New file: tab_size changed, word_wrap removed, font_size added
        std::fs::write(&path, "[editor]\ntab_size = 8\nfont_size = 14\n").unwrap();

        let event = manager
            .reload_file(&path, ConfigLayer::User)
            .unwrap()
            .unwrap();

        // tab_size changed (4 → 8)
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
        // word_wrap removed
        assert!(event.changed_keys.contains(&"editor.word_wrap".to_string()));
        // font_size added
        assert!(event.changed_keys.contains(&"editor.font_size".to_string()));
    }

    // Validates: Requirement 3.5 — compute_diff correctly compares stores
    #[test]
    fn compute_diff_empty_stores_produces_no_changes() {
        let old = EffectiveStore::new();
        let new = EffectiveStore::new();
        let diff = compute_diff(&old, &new);
        assert!(diff.is_empty());
    }

    // Validates: Requirement 3.5 — compute_diff detects new keys
    #[test]
    fn compute_diff_detects_new_keys_in_new_store() {
        use crate::provenance::{EffectiveValue, Provenance};

        let old = EffectiveStore::new();
        let mut new = EffectiveStore::new();
        new.insert(
            "editor.tab_size".to_string(),
            EffectiveValue {
                value: ConfigValue::Integer(4),
                provenance: Provenance {
                    layer: ConfigLayer::User,
                    source_file: None,
                },
            },
        );

        let diff = compute_diff(&old, &new);
        assert_eq!(diff, vec!["editor.tab_size".to_string()]);
    }

    // Validates: Requirement 3.5 — compute_diff detects removed keys
    #[test]
    fn compute_diff_detects_removed_keys() {
        use crate::provenance::{EffectiveValue, Provenance};

        let mut old = EffectiveStore::new();
        old.insert(
            "editor.tab_size".to_string(),
            EffectiveValue {
                value: ConfigValue::Integer(4),
                provenance: Provenance {
                    layer: ConfigLayer::User,
                    source_file: None,
                },
            },
        );

        let new = EffectiveStore::new();
        let diff = compute_diff(&old, &new);
        assert_eq!(diff, vec!["editor.tab_size".to_string()]);
    }

    // Validates: Requirement 3.6 — Reload after failure still works
    #[test]
    fn reload_recovers_after_previous_failure() {
        let dir = TempDir::new().unwrap();
        let path = write_toml_file(&dir, "user.toml", "[editor]\ntab_size = 4\n");

        let initial_values = load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path.clone(),
            values: initial_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // First: write invalid TOML — reload is rejected
        std::fs::write(&path, "invalid [[[").unwrap();
        let result = manager.reload_file(&path, ConfigLayer::User);
        assert!(result.unwrap().is_none());
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(4))
        );

        // Second: write valid TOML — reload should succeed
        std::fs::write(&path, "[editor]\ntab_size = 8\n").unwrap();
        let result = manager.reload_file(&path, ConfigLayer::User);
        let event = result.unwrap().unwrap();
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(8))
        );
    }

    // ========================================================================
    // 15.1 — load_project: detect .ffworkbench/config.toml, load at Project layer
    // ========================================================================

    // Validates: Requirement 5.1 — Recognizes .ffworkbench/config.toml as project config
    #[test]
    fn load_project_detects_and_loads_config_from_project_root() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\nindent_style = \"space\"\n",
        )
        .unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let event = manager.load_project(dir.path()).unwrap();

        // Should have changed keys for the loaded project config
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
        assert!(event
            .changed_keys
            .contains(&"editor.indent_style".to_string()));
        assert_eq!(event.source_layer, ConfigLayer::Project);

        // Values should be in the effective store
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2))
        );
        assert_eq!(
            manager.store().get_value("editor.indent_style"),
            Some(&ConfigValue::String("space".to_string()))
        );
    }

    // Validates: Requirement 5.2 — Merges project settings at Project priority level
    #[test]
    fn load_project_merges_at_project_priority_overriding_user_layer() {
        let dir = TempDir::new().unwrap();

        // Set up a User layer with tab_size = 4
        let user_path = write_toml_file(&dir, "user.toml", "[editor]\ntab_size = 4\n");
        let user_values = load_toml_file(&user_path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: user_path,
            values: user_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Verify User layer value
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(4))
        );

        // Set up project config with tab_size = 2
        let project_root = dir.path().join("my-project");
        let ffworkbench_dir = project_root.join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();

        let event = manager.load_project(&project_root).unwrap();

        // Project (priority 4) should override User (priority 2)
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2))
        );
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
    }

    // Validates: Requirement 5.1, 5.2 — Returns empty event when no config file exists
    #[test]
    fn load_project_returns_empty_event_when_no_config_exists() {
        let dir = TempDir::new().unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let event = manager.load_project(dir.path()).unwrap();

        assert!(
            event.changed_keys.is_empty(),
            "Should return empty changed_keys when no project config exists"
        );
        assert_eq!(event.source_layer, ConfigLayer::Project);
    }

    // Validates: Requirement 5.7 — Returns error for invalid TOML in project config
    #[test]
    fn load_project_returns_error_for_invalid_toml() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "this is not [valid toml\nbroken",
        )
        .unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let result = manager.load_project(dir.path());
        assert!(result.is_err(), "Should return error for invalid TOML");

        match result.unwrap_err() {
            ConfigError::ParseError { path, details } => {
                assert!(
                    path.ends_with("config.toml"),
                    "ParseError path should reference config.toml"
                );
                assert!(!details.is_empty());
            }
            other => panic!("Expected ParseError, got: {:?}", other),
        }
    }

    // Validates: Requirement 5.2 — Replaces existing Project layer on second load
    #[test]
    fn load_project_replaces_existing_project_layer() {
        let dir = TempDir::new().unwrap();

        // First project
        let project1 = dir.path().join("project1");
        let ffworkbench1 = project1.join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench1).unwrap();
        std::fs::write(ffworkbench1.join("config.toml"), "[editor]\ntab_size = 2\n").unwrap();

        // Second project
        let project2 = dir.path().join("project2");
        let ffworkbench2 = project2.join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench2).unwrap();
        std::fs::write(ffworkbench2.join("config.toml"), "[editor]\ntab_size = 8\n").unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        // Load first project
        manager.load_project(&project1).unwrap();
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2))
        );

        // Load second project — should replace the first
        let event = manager.load_project(&project2).unwrap();
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(8))
        );
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
    }

    // Validates: Requirement 5.1 — has_project_layer reports correctly
    #[test]
    fn has_project_layer_reports_correctly_after_load() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        assert!(!manager.has_project_layer());

        manager.load_project(dir.path()).unwrap();

        assert!(manager.has_project_layer());
    }

    // Validates: Requirement 5.1 — project_source_path returns correct path
    #[test]
    fn project_source_path_returns_loaded_config_path() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        let config_path = ffworkbench_dir.join("config.toml");
        std::fs::write(&config_path, "[editor]\ntab_size = 2\n").unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        assert!(manager.project_source_path().is_none());

        manager.load_project(dir.path()).unwrap();

        assert_eq!(manager.project_source_path(), Some(config_path.as_path()));
    }

    // Validates: Requirement 5.6 — unload_project removes project layer and reverts values
    #[test]
    fn unload_project_removes_layer_and_reverts_values() {
        let dir = TempDir::new().unwrap();

        // User layer
        let user_path = write_toml_file(&dir, "user.toml", "[editor]\ntab_size = 4\n");
        let user_values = load_toml_file(&user_path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: user_path,
            values: user_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Load project that overrides tab_size
        let project_root = dir.path().join("project");
        let ffworkbench_dir = project_root.join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();
        manager.load_project(&project_root).unwrap();
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2))
        );

        // Unload project — should revert to User layer value
        let event = manager.unload_project();
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(4))
        );
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
        assert!(!manager.has_project_layer());
    }

    // Validates: Requirement 5.6 — unload when no project is loaded is a no-op
    #[test]
    fn unload_project_with_no_loaded_project_returns_empty_event() {
        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let event = manager.unload_project();
        assert!(event.changed_keys.is_empty());
        assert_eq!(event.source_layer, ConfigLayer::Project);
    }

    // ========================================================================
    // 15.2 — unload_project invokes callbacks for changed keys
    // ========================================================================

    // Validates: Requirement 5.6 — unload_project invokes Reload_Callbacks for changed keys
    #[test]
    fn unload_project_invokes_callbacks_for_changed_keys() {
        use crate::callback::CallbackRegistry;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let dir = TempDir::new().unwrap();

        // User layer with tab_size = 4
        let user_path = write_toml_file(&dir, "user.toml", "[editor]\ntab_size = 4\n");
        let user_values = load_toml_file(&user_path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: user_path,
            values: user_values,
        }];

        let schema = SchemaRegistry::new();
        let callbacks = Arc::new(CallbackRegistry::new());

        let mut manager = ReloadManager::with_callbacks(layers, schema, Arc::clone(&callbacks));

        // Register a callback watching editor.tab_size
        let invocation_count = Arc::new(AtomicU32::new(0));
        let count_clone = Arc::clone(&invocation_count);
        let _handle = callbacks.on_reload(
            &["editor.tab_size"],
            Box::new(move |_event| {
                count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        );

        // Load project that overrides tab_size
        let project_root = dir.path().join("project");
        let ffworkbench_dir = project_root.join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();
        manager.load_project(&project_root).unwrap();

        // load_project also invokes callbacks — reset the count
        invocation_count.store(0, Ordering::SeqCst);

        // Unload project — should invoke callback because tab_size reverts from 2 to 4
        let event = manager.unload_project();
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
        assert_eq!(invocation_count.load(Ordering::SeqCst), 1);
    }

    // Validates: Requirement 5.6 — unload_project does NOT invoke callbacks when no keys changed
    #[test]
    fn unload_project_does_not_invoke_callbacks_when_no_keys_changed() {
        use crate::callback::CallbackRegistry;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let schema = SchemaRegistry::new();
        let callbacks = Arc::new(CallbackRegistry::new());

        let mut manager = ReloadManager::with_callbacks(Vec::new(), schema, Arc::clone(&callbacks));

        // Register a callback
        let invocation_count = Arc::new(AtomicU32::new(0));
        let count_clone = Arc::clone(&invocation_count);
        let _handle = callbacks.on_reload(
            &["editor.tab_size"],
            Box::new(move |_event| {
                count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        );

        // Unload with no project loaded — no keys change, no callback
        let event = manager.unload_project();
        assert!(event.changed_keys.is_empty());
        assert_eq!(invocation_count.load(Ordering::SeqCst), 0);
    }

    // Validates: Requirement 5.6 — unload_project only invokes callbacks for keys that actually changed
    #[test]
    fn unload_project_invokes_only_matching_callbacks() {
        use crate::callback::CallbackRegistry;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let dir = TempDir::new().unwrap();

        // User layer with tab_size = 4
        let user_path = write_toml_file(&dir, "user.toml", "[editor]\ntab_size = 4\n");
        let user_values = load_toml_file(&user_path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: user_path,
            values: user_values,
        }];

        let schema = SchemaRegistry::new();
        let callbacks = Arc::new(CallbackRegistry::new());

        let mut manager = ReloadManager::with_callbacks(layers, schema, Arc::clone(&callbacks));

        // Register callbacks for different keys
        let tab_size_count = Arc::new(AtomicU32::new(0));
        let theme_count = Arc::new(AtomicU32::new(0));

        let tab_clone = Arc::clone(&tab_size_count);
        let _h1 = callbacks.on_reload(
            &["editor.tab_size"],
            Box::new(move |_event| {
                tab_clone.fetch_add(1, Ordering::SeqCst);
            }),
        );

        let theme_clone = Arc::clone(&theme_count);
        let _h2 = callbacks.on_reload(
            &["theme.active"],
            Box::new(move |_event| {
                theme_clone.fetch_add(1, Ordering::SeqCst);
            }),
        );

        // Load project that only overrides tab_size (not theme)
        let project_root = dir.path().join("project");
        let ffworkbench_dir = project_root.join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();
        manager.load_project(&project_root).unwrap();

        // Reset counts after load_project callback invocations
        tab_size_count.store(0, Ordering::SeqCst);
        theme_count.store(0, Ordering::SeqCst);

        // Unload — only editor.tab_size should change, not theme.active
        manager.unload_project();

        assert_eq!(tab_size_count.load(Ordering::SeqCst), 1);
        assert_eq!(theme_count.load(Ordering::SeqCst), 0);
    }

    // ========================================================================
    // 15.4 — Project-layer hot-reload: monitor project config file for changes
    // ========================================================================

    // Validates: Requirement 5.5, 3.1 — load_project registers config file with watcher
    #[test]
    fn load_project_registers_config_file_with_watcher() {
        use crate::watcher::ConfigWatcher;

        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        let config_path = ffworkbench_dir.join("config.toml");
        std::fs::write(&config_path, "[editor]\ntab_size = 2\n").unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        // Set up a watcher
        let watcher = ConfigWatcher::new().unwrap();
        manager.set_watcher(watcher);

        // Verify watcher has no watched paths initially
        assert!(manager.watcher().unwrap().watched_paths().is_empty());

        // Load project — should register the config file with the watcher
        manager.load_project(dir.path()).unwrap();

        // Verify the project config file is now in the watcher's watch list
        let watched = manager.watcher().unwrap().watched_paths();
        assert_eq!(watched.len(), 1);
        assert_eq!(watched[0], config_path);
    }

    // Validates: Requirement 5.5, 5.6 — unload_project unregisters config file from watcher
    #[test]
    fn unload_project_unregisters_config_file_from_watcher() {
        use crate::watcher::ConfigWatcher;

        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        let config_path = ffworkbench_dir.join("config.toml");
        std::fs::write(&config_path, "[editor]\ntab_size = 2\n").unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let watcher = ConfigWatcher::new().unwrap();
        manager.set_watcher(watcher);

        // Load project — registers with watcher
        manager.load_project(dir.path()).unwrap();
        assert_eq!(manager.watcher().unwrap().watched_paths().len(), 1);

        // Unload project — should unregister from watcher
        manager.unload_project();
        assert!(
            manager.watcher().unwrap().watched_paths().is_empty(),
            "Watcher should have no watched paths after unload_project"
        );
    }

    // Validates: Requirement 5.5, 3.1 — load_project without watcher still works (graceful)
    #[test]
    fn load_project_without_watcher_does_not_panic() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        // No watcher set — load_project should still work
        let event = manager.load_project(dir.path()).unwrap();
        assert!(!event.changed_keys.is_empty());
    }

    // Validates: Requirement 5.5, 3.2 — watcher detects project config change and reload_file applies it
    #[test]
    fn watcher_detects_project_config_change_and_reload_applies_it() {
        use crate::watcher::ConfigWatcher;
        use std::thread;
        use std::time::{Duration, Instant};

        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        let config_path = ffworkbench_dir.join("config.toml");
        std::fs::write(&config_path, "[editor]\ntab_size = 2\n").unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let watcher = ConfigWatcher::with_debounce(Duration::from_millis(50)).unwrap();
        manager.set_watcher(watcher);

        // Load project — registers with watcher
        manager.load_project(dir.path()).unwrap();
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2))
        );

        // Allow the watcher to fully initialize
        thread::sleep(Duration::from_millis(100));

        // Modify the project config file on disk
        std::fs::write(&config_path, "[editor]\ntab_size = 8\n").unwrap();

        // Poll the watcher until the change is detected (within 2 seconds per Req 3.2)
        let start = Instant::now();
        let mut detected = false;
        while start.elapsed() < Duration::from_secs(2) {
            thread::sleep(Duration::from_millis(70));
            let changes = manager.watcher_mut().unwrap().poll_changes();
            if changes.iter().any(|c| c.path == config_path) {
                detected = true;
                break;
            }
        }

        assert!(detected, "File change should be detected within 2 seconds");

        // Now trigger reload via reload_file (the same call the system would make)
        let event = manager
            .reload_file(&config_path, ConfigLayer::Project)
            .unwrap();
        assert!(event.is_some());
        let event = event.unwrap();
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));

        // Verify effective value is updated
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(8))
        );
    }

    // Validates: Requirement 5.5, 5.1 — open_project also registers with watcher
    #[test]
    fn open_project_registers_config_file_with_watcher() {
        use crate::watcher::ConfigWatcher;

        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        let config_path = ffworkbench_dir.join("config.toml");
        std::fs::write(&config_path, "[editor]\ntab_size = 2\n").unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let watcher = ConfigWatcher::new().unwrap();
        manager.set_watcher(watcher);

        // open_project (the graceful variant) should also register with watcher
        manager.open_project(dir.path());

        let watched = manager.watcher().unwrap().watched_paths();
        assert_eq!(watched.len(), 1);
        assert_eq!(watched[0], config_path);
    }

    // Validates: Requirement 5.5 — load_project with no config file does not register with watcher
    #[test]
    fn load_project_no_config_file_does_not_register_watcher() {
        use crate::watcher::ConfigWatcher;

        let dir = TempDir::new().unwrap();
        // No .ffworkbench directory

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let watcher = ConfigWatcher::new().unwrap();
        manager.set_watcher(watcher);

        manager.load_project(dir.path()).unwrap();

        // No config file exists, so nothing should be watched
        assert!(manager.watcher().unwrap().watched_paths().is_empty());
    }

    // ========================================================================
    // 15.5 — Project config load failure handling (open_project graceful path)
    // ========================================================================

    // Validates: Requirement 5.7 — Invalid TOML: open_project emits WARN, skips project layer, continues
    #[test]
    fn open_project_with_invalid_toml_skips_project_layer_and_returns_empty_event() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "this is [[[not valid TOML content\nbroken = = =",
        )
        .unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        // open_project should NOT panic or propagate error
        let event = manager.open_project(dir.path());

        // Should return empty event (no changes applied)
        assert!(
            event.changed_keys.is_empty(),
            "Invalid TOML should result in empty changed_keys, got: {:?}",
            event.changed_keys
        );
        assert_eq!(event.source_layer, ConfigLayer::Project);

        // Project layer should NOT be loaded
        assert!(
            !manager.has_project_layer(),
            "Project layer must not be loaded after invalid TOML"
        );
    }

    // Validates: Requirement 5.7 — I/O error (missing file path simulated): open_project emits WARN, skips project layer
    #[test]
    fn open_project_with_io_error_skips_project_layer_and_returns_empty_event() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        let config_path = ffworkbench_dir.join("config.toml");
        std::fs::write(&config_path, "[editor]\ntab_size = 4\n").unwrap();

        // Make the file unreadable by replacing it with a directory (causes I/O error)
        std::fs::remove_file(&config_path).unwrap();
        std::fs::create_dir_all(&config_path).unwrap();

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        // open_project should NOT panic or propagate error
        let event = manager.open_project(dir.path());

        // Should return empty event
        assert!(
            event.changed_keys.is_empty(),
            "I/O error should result in empty changed_keys"
        );
        assert_eq!(event.source_layer, ConfigLayer::Project);

        // Project layer should NOT be loaded
        assert!(
            !manager.has_project_layer(),
            "Project layer must not be loaded after I/O error"
        );
    }

    // Validates: Requirement 5.7 — After project config failure, other layers remain accessible
    #[test]
    fn open_project_failure_does_not_affect_other_layers() {
        let dir = TempDir::new().unwrap();

        // Set up a valid User layer first
        let user_path = write_toml_file(
            &dir,
            "user.toml",
            "[editor]\ntab_size = 4\nword_wrap = true\n",
        );
        let user_values = load_toml_file(&user_path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: user_path,
            values: user_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Verify User layer values are accessible before project open attempt
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(4))
        );
        assert_eq!(
            manager.store().get_value("editor.word_wrap"),
            Some(&ConfigValue::Boolean(true))
        );

        // Create a project with invalid TOML
        let project_root = dir.path().join("my-project");
        let ffworkbench_dir = project_root.join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "invalid TOML [[[content",
        )
        .unwrap();

        // open_project with broken config
        let event = manager.open_project(&project_root);
        assert!(event.changed_keys.is_empty());

        // User layer values must still be fully accessible after project load failure
        assert_eq!(
            manager.store().get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(4)),
            "User layer values must remain accessible after project config failure"
        );
        assert_eq!(
            manager.store().get_value("editor.word_wrap"),
            Some(&ConfigValue::Boolean(true)),
            "All user layer values must remain unchanged"
        );
    }

    // Validates: Requirement 5.7 — open_project returns ReloadEvent (not Result), no error propagation
    #[test]
    fn open_project_signature_never_propagates_errors() {
        let dir = TempDir::new().unwrap();

        // Case 1: No config file at all — returns normally
        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);
        let event = manager.open_project(dir.path());
        assert!(event.changed_keys.is_empty());

        // Case 2: Invalid TOML — returns normally (no Result, no panic)
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(ffworkbench_dir.join("config.toml"), "broken [= toml\n{{{").unwrap();
        let event = manager.open_project(dir.path());
        assert!(event.changed_keys.is_empty());

        // Case 3: Config exists, then is corrupted — same path, no error propagation
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();
        let event = manager.open_project(dir.path());
        assert!(!event.changed_keys.is_empty()); // valid config loads successfully

        // Now corrupt it and reload via open_project on a fresh manager
        std::fs::write(ffworkbench_dir.join("config.toml"), "[[[").unwrap();
        let schema2 = SchemaRegistry::new();
        let mut manager2 = ReloadManager::new(Vec::new(), schema2);
        let event = manager2.open_project(dir.path());
        assert!(event.changed_keys.is_empty()); // graceful handling
    }

    // Validates: Requirement 3.7 — Debounce coalesces rapid project config changes
    #[test]
    fn debounce_coalesces_rapid_project_config_changes() {
        use crate::watcher::ConfigWatcher;
        use std::thread;
        use std::time::Duration;

        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        let config_path = ffworkbench_dir.join("config.toml");
        std::fs::write(&config_path, "[editor]\ntab_size = 2\n").unwrap();

        let debounce_ms = 50;
        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(Vec::new(), schema);

        let watcher = ConfigWatcher::with_debounce(Duration::from_millis(debounce_ms)).unwrap();
        manager.set_watcher(watcher);

        manager.load_project(dir.path()).unwrap();

        // Allow watcher to initialize
        thread::sleep(Duration::from_millis(100));

        // Write to the file multiple times rapidly
        for i in 1..=5 {
            std::fs::write(&config_path, format!("[editor]\ntab_size = {}\n", i)).unwrap();
            thread::sleep(Duration::from_millis(10));
        }

        // Wait for debounce window to elapse
        thread::sleep(Duration::from_millis((debounce_ms + 100) as u64));

        // Poll — should get at most one coalesced event
        let changes = manager.watcher_mut().unwrap().poll_changes();
        let config_changes: Vec<_> = changes.iter().filter(|c| c.path == config_path).collect();
        assert!(
            config_changes.len() <= 1,
            "Expected at most 1 coalesced event, got {}",
            config_changes.len()
        );
    }
}
