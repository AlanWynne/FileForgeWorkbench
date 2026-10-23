//! Project-layer reload operations for [`ReloadManager`].
//!
//! Split out of `ops.rs` (which keeps the single-file and all-layer reload)
//! purely to stay under the 400-line per-file limit; no behaviour changed. Each
//! public method has a `_compute` sibling that performs the recompute+swap
//! WITHOUT invoking callbacks, so a caller holding the `ConfigSystem` write lock
//! can drop the guard before invoking (CR-NR-103 Fix 1).

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::error::ConfigError;
use crate::layer::ConfigLayer;
use crate::loader::{load_toml_file, LayerData};
use crate::merger::merge_layers;

use super::diff::compute_diff;
use super::{ReloadEvent, ReloadManager};

impl ReloadManager {
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
        let event = self.load_project_compute(project_root)?;
        if !event.changed_keys.is_empty() {
            self.callbacks.invoke(&event);
        }
        Ok(event)
    }

    /// Load the project layer WITHOUT invoking reload callbacks.
    ///
    /// Same pipeline as [`load_project`](Self::load_project) but returns the
    /// event instead of invoking callbacks, so a caller holding the
    /// `ConfigSystem` write lock can drop the guard and invoke afterwards
    /// (CR-NR-103 Fix 1 -- keep callback invocation off the write lock).
    pub fn load_project_compute(
        &mut self,
        project_root: &Path,
    ) -> Result<ReloadEvent, ConfigError> {
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
        let event = self.unload_project_compute();
        if !event.changed_keys.is_empty() {
            self.callbacks.invoke(&event);
        }
        event
    }

    /// Unload the project layer WITHOUT invoking reload callbacks.
    ///
    /// Same pipeline as [`unload_project`](Self::unload_project) but returns the
    /// event instead of invoking callbacks, so a caller holding the
    /// `ConfigSystem` write lock can drop the guard and invoke afterwards
    /// (CR-NR-103 Fix 1).
    pub fn unload_project_compute(&mut self) -> ReloadEvent {
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

        ReloadEvent {
            changed_keys,
            source_layer: ConfigLayer::Project,
            timestamp: SystemTime::now(),
        }
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
}
