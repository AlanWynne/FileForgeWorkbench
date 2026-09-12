use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::error::ConfigError;
use crate::layer::ConfigLayer;
use crate::loader::{load_toml_file, LayerData};
use crate::merger::merge_layers;
use crate::value::ConfigTable;

use super::diff::compute_diff;
use super::diff::flatten_config_table;
use super::{ReloadEvent, ReloadManager};

impl ReloadManager {
    /// Reload a single file that belongs to a specific layer.
    ///
    /// Pipeline: re-read -> parse TOML -> re-merge all layers -> diff -> return event.
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
                // Reject reload -- retain previous values, emit WARN log
                ff_logging::log_warn!(
                    "[config] reload: file '{}' has invalid TOML: {} -- retaining previous values",
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

        // Step 5: Atomic swap -- replace old store with new
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
    /// - If `.ffworkbench/config.toml` does not exist -> no-op (empty event)
    /// - If the file exists but contains invalid TOML -> logs WARN, skips project layer
    /// - If the file exists but cannot be read (I/O error) -> logs WARN, skips project layer
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
                    "[config] open_project: project config '{}' has invalid TOML: {} -- skipping project layer",
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
                    "[config] open_project: cannot read project config: {} -- skipping project layer",
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
                    "[config] open_project: unexpected error loading project config: {} -- skipping project layer",
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

    /// Collect all values from a specific layer as a flat ConfigTable.
    ///
    /// Returns the raw (pre-merge) values from the first layer matching
    /// `layer`. Used by export_settings(UserLayer/ProjectLayer).
    pub fn layer_values(&self, layer: ConfigLayer) -> ConfigTable {
        self.layers
            .iter()
            .find(|l| l.layer == layer)
            .map(|l| flatten_config_table(&l.values, ""))
            .unwrap_or_default()
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
