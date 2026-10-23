use std::path::PathBuf;
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
    ///
    /// This recomputes and swaps the effective store AND invokes the matching
    /// reload callbacks in a single call. Callers that hold the `ConfigSystem`
    /// write lock must NOT use this method: invoking callbacks under the lock
    /// can self-deadlock if a callback re-enters the handle for a read (the
    /// `std::sync::RwLock` is not reentrant). Those callers use
    /// [`reload_file_compute`](Self::reload_file_compute) to recompute+swap
    /// under the lock, drop the guard, then invoke via
    /// [`callbacks`](super::ReloadManager::callbacks). See CR-NR-103 Fix 1.
    pub fn reload_file(
        &mut self,
        path: &PathBuf,
        layer: ConfigLayer,
    ) -> Result<Option<ReloadEvent>, ConfigError> {
        let event = self.reload_file_compute(path, layer)?;
        if let Some(ref ev) = event {
            self.callbacks.invoke(ev);
        }
        Ok(event)
    }

    /// Recompute and swap the effective store for a single file WITHOUT
    /// invoking reload callbacks.
    ///
    /// Identical pipeline to [`reload_file`](Self::reload_file) (re-read ->
    /// parse -> re-merge -> diff -> atomic swap) but it returns the
    /// `ReloadEvent` instead of invoking callbacks. The caller is responsible
    /// for invoking `callbacks().invoke(&event)` AFTER releasing any held
    /// `ConfigSystem` lock, which is what keeps callback invocation off the
    /// write lock and avoids the reentrant-lock self-deadlock (CR-NR-103 Fix 1).
    pub fn reload_file_compute(
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

        // Step 6: Build and return the ReloadEvent (callbacks NOT invoked here)
        let event = ReloadEvent {
            changed_keys,
            source_layer: layer,
            timestamp: SystemTime::now(),
        };

        Ok(Some(event))
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
    /// prevent reloading others. Callbacks are invoked per changed layer.
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

    /// Reload all layer files WITHOUT invoking reload callbacks.
    ///
    /// Same iteration as [`reload_all`](Self::reload_all) but recomputes+swaps
    /// each layer without invoking callbacks, returning the changed events so a
    /// caller holding the `ConfigSystem` write lock can drop the guard and
    /// invoke them afterwards (CR-NR-103 Fix 1). Preserves the per-layer
    /// independent-failure behaviour: a failed layer yields its `Err` in the
    /// result vector and does not stop the others.
    pub fn reload_all_compute(&mut self) -> Vec<Result<Option<ReloadEvent>, ConfigError>> {
        let paths_and_layers: Vec<(PathBuf, ConfigLayer)> = self
            .layers
            .iter()
            .map(|l| (l.source_path.clone(), l.layer))
            .collect();

        let mut results = Vec::new();
        for (path, layer) in paths_and_layers {
            results.push(self.reload_file_compute(&path, layer));
        }
        results
    }
}
