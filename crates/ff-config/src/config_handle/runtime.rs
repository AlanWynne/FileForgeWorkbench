use std::path::Path;
use std::sync::Arc;

use crate::audit::{AuditEntry, AuditFilter};
use crate::error::ConfigError;
use crate::layer::ConfigLayer;
use crate::reload::ReloadEvent;

use super::toml_io::{extract_locked_keys, remove_key_from_toml_file, write_key_to_toml_file};
use super::{ConfigHandle, ConfigSystem};

impl ConfigHandle {
    // ====================================================================
    // Write access (Task 20.3): mutations acquire write lock briefly
    // ====================================================================

    /// Reload all configuration layer files.
    ///
    /// Acquires a write lock, re-reads all layer files from disk, re-merges,
    /// computes diffs, and invokes callbacks for changed keys. The write lock
    /// is held for the duration of the reload (including callback invocation).
    ///
    /// Returns results for each layer reload attempt.
    pub fn reload(&self) -> Vec<Result<Option<ReloadEvent>, ConfigError>> {
        // Recompute+swap every layer under the write lock, DROP the guard, then
        // invoke callbacks for each changed layer. Invoking after the guard is
        // released is what prevents a callback that reads this handle from
        // self-deadlocking on the non-reentrant RwLock (CR-NR-103 Fix 1).
        let (results, events) = {
            let mut system = self.inner.write().unwrap();
            let results = system.manager.reload_all_compute();
            let events: Vec<ReloadEvent> = results
                .iter()
                .filter_map(|r| r.as_ref().ok().and_then(|opt| opt.clone()))
                .collect();
            (results, events)
        };
        if !events.is_empty() {
            let callbacks = self.callbacks();
            for event in &events {
                callbacks.invoke(event);
            }
        }
        results
    }

    /// Set the active user profile, triggering a re-merge.
    ///
    /// When `name` is `Some`, activates the named profile (loads its TOML file
    /// into the Profile layer). When `name` is `None`, deactivates the current
    /// profile (removes the Profile layer).
    ///
    /// Acquires a write lock for the duration of the profile switch and re-merge.
    /// Returns a `ReloadEvent` with the keys that changed.
    pub fn set_active_profile(&self, name: Option<&str>) -> Result<ReloadEvent, ConfigError> {
        let mut system = self.inner.write().unwrap();

        let profile_manager =
            system
                .profile_manager
                .as_mut()
                .ok_or_else(|| ConfigError::ProfileNotFound {
                    name: name.unwrap_or("<none>").to_string(),
                })?;

        match name {
            Some(profile_name) => {
                let layer_data = profile_manager.set_active_profile(profile_name)?;

                // Replace or add the Profile layer in the manager's layers
                Self::upsert_layer(&mut system.manager, layer_data);

                // Rebuild the effective store
                let event = Self::rebuild_store(&mut system.manager, ConfigLayer::Profile);
                Ok(event)
            }
            None => {
                profile_manager.deactivate_profile();

                // Remove the Profile layer
                Self::remove_layer(&mut system.manager, ConfigLayer::Profile);

                // Rebuild the effective store
                let event = Self::rebuild_store(&mut system.manager, ConfigLayer::Profile);
                Ok(event)
            }
        }
    }

    /// Load project-layer configuration from the given project root.
    ///
    /// Acquires a write lock, loads the project config file, re-merges layers,
    /// and invokes callbacks for changed keys.
    pub fn load_project(&self, root: &Path) -> Result<ReloadEvent, ConfigError> {
        // Compute+swap under the write lock, drop the guard, then invoke
        // callbacks -- keeping invocation off the lock (CR-NR-103 Fix 1).
        let event = {
            let mut system = self.inner.write().unwrap();
            system.manager.load_project_compute(root)?
        };
        if !event.changed_keys.is_empty() {
            self.callbacks().invoke(&event);
        }
        Ok(event)
    }

    /// Unload the project-layer configuration.
    ///
    /// Acquires a write lock, removes the project layer, re-merges remaining
    /// layers, and invokes callbacks for changed keys.
    pub fn unload_project(&self) -> ReloadEvent {
        // Compute+swap under the write lock, drop the guard, then invoke
        // callbacks -- keeping invocation off the lock (CR-NR-103 Fix 1).
        let event = {
            let mut system = self.inner.write().unwrap();
            system.manager.unload_project_compute()
        };
        if !event.changed_keys.is_empty() {
            self.callbacks().invoke(&event);
        }
        event
    }

    /// Get a reference to the shared callback registry.
    ///
    /// The `CallbackRegistry` is `Arc`-wrapped and can be cloned out for
    /// registering callbacks without holding the `ConfigHandle`'s lock.
    pub fn callbacks(&self) -> Arc<crate::callback::CallbackRegistry> {
        let system = self.inner.read().unwrap();
        Arc::clone(system.manager.callbacks())
    }

    // ====================================================================
    // Initialization / shutdown helpers (Task 21)
    // ====================================================================

    /// Acquire a write lock on the internal system state.
    ///
    /// Used by the initialization sequence to load workspace layers directly.
    pub(crate) fn inner_write(&self) -> std::sync::RwLockWriteGuard<'_, ConfigSystem> {
        self.inner.write().unwrap()
    }

    /// Set the file watcher on the underlying ReloadManager.
    ///
    /// Acquires a write lock and installs the watcher for hot-reload monitoring.
    pub fn set_watcher(&self, watcher: crate::watcher::ConfigWatcher) {
        let mut system = self.inner.write().unwrap();
        system.manager.set_watcher(watcher);
    }

    /// Stop the file watcher if one is active.
    ///
    /// Acquires a write lock, takes ownership of the watcher, and stops it.
    /// After this call, no further file change events will be generated.
    pub fn stop_watcher(&self) {
        let mut system = self.inner.write().unwrap();
        if let Some(watcher) = system.manager.take_watcher() {
            watcher.stop();
        }
    }

    /// Deregister all reload callbacks.
    ///
    /// Used during shutdown to ensure no callbacks are invoked after
    /// the system is shut down.
    pub fn clear_callbacks(&self) {
        let system = self.inner.read().unwrap();
        system.manager.callbacks().clear_all();
    }

    /// Register a single schema entry.
    ///
    /// Acquires a write lock and registers the entry in the schema registry.
    /// Idempotent for same-type re-registration; returns `ConfigError::SchemaConflict`
    /// if the key is already registered with a different type.
    pub fn register_schema_entry(
        &self,
        entry: crate::schema::SchemaEntry,
    ) -> Result<(), crate::error::ConfigError> {
        let mut system = self.inner.write().unwrap();
        system.manager.schema_mut().register(entry)
    }

    /// Write a value to the user-layer configuration file.
    ///
    /// Reads the current user-layer TOML file (if it exists), sets the key
    /// using dot-separated path notation, writes the file back atomically,
    /// then triggers a reload so the effective store reflects the change.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::KeyLocked` if the key is locked by system policy.
    /// Returns `ConfigError::Io` if the file cannot be read or written.
    /// Returns `ConfigError::ParseError` if the existing file is invalid TOML.
    ///
    /// Validates: Requirement 15.4, Requirement 18.3
    pub fn set_user_value(
        &self,
        key: &str,
        value: crate::value::ConfigValue,
    ) -> Result<(), crate::error::ConfigError> {
        // Guard: reject writes to locked keys (Req 18.3)
        {
            let system = self.inner.read().unwrap();
            if system.locked_keys.contains(key) {
                return Err(crate::error::ConfigError::KeyLocked {
                    key: key.to_string(),
                });
            }
        }
        let user_path = self.user_config_path_resolved().ok_or_else(|| {
            crate::error::ConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "user config directory not available",
            ))
        })?;
        write_key_to_toml_file(&user_path, key, value)?;
        // Trigger reload so the effective store picks up the change. Recompute
        // and swap the store UNDER the write lock, then DROP the guard before
        // invoking reload callbacks: a callback (e.g. the ui.locale catalogue
        // swap) may re-read this handle, and the non-reentrant RwLock would
        // self-deadlock if invocation happened while the write lock is held
        // (CR-NR-103 Fix 1).
        self.reload_user_layer_and_notify(&user_path);
        Ok(())
    }

    /// Recompute+swap the User layer for `user_path` under the write lock, then
    /// release the lock and invoke the matching reload callbacks.
    ///
    /// This is the shared helper for [`set_user_value`](Self::set_user_value)
    /// and [`remove_user_value`](Self::remove_user_value). Keeping callback
    /// invocation strictly AFTER the write guard is dropped is what prevents the
    /// reentrant-`RwLock` self-deadlock when a callback reads the handle back
    /// (CR-NR-103 Fix 1). The catalogue swap performed inside a callback takes
    /// `ff_i18n`'s own independent lock, which is never nested under the config
    /// lock here.
    fn reload_user_layer_and_notify(&self, user_path: &Path) {
        let event = {
            let mut system = self.inner.write().unwrap();
            match system
                .manager
                .reload_file_compute(&user_path.to_path_buf(), crate::layer::ConfigLayer::User)
            {
                Ok(event) => event,
                Err(e) => {
                    ff_logging::log_warn!(
                        "[config] reload after user-layer write failed: {} -- \
                         effective store retains previous values",
                        e
                    );
                    None
                }
            }
            // Write guard dropped here, before any callback runs.
        };
        if let Some(event) = event {
            self.callbacks().invoke(&event);
        }
    }

    /// Remove a user-layer override for a key, restoring the schema default.
    ///
    /// Reads the current user-layer TOML file, removes the key at the given
    /// dot-separated path, writes the file back, then triggers a reload.
    ///
    /// If the key is not present in the user file, this is a no-op (succeeds).
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::Io` if the file cannot be read or written.
    /// Returns `ConfigError::ParseError` if the existing file is invalid TOML.
    ///
    /// Validates: Requirement 15.6
    pub fn remove_user_value(&self, key: &str) -> Result<(), crate::error::ConfigError> {
        let user_path = self.user_config_path_resolved().ok_or_else(|| {
            crate::error::ConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "user config directory not available",
            ))
        })?;
        remove_key_from_toml_file(&user_path, key)?;
        // Same lock discipline as set_user_value: recompute+swap under the
        // write lock, then invoke callbacks after the guard is dropped so a
        // callback that reads the handle cannot self-deadlock (CR-NR-103 Fix 1).
        self.reload_user_layer_and_notify(&user_path);
        Ok(())
    }

    // === Locked Key API (Requirement 18) ===================================

    /// Returns true if the given key is locked by system policy.
    ///
    /// Locked keys cannot be overridden by User, Profile, Project, or Workspace
    /// layers. The Settings panel uses this to disable widgets and show a
    /// "LOCKED" badge.
    ///
    /// Validates: Requirement 18.5
    pub fn is_locked(&self, key: &str) -> bool {
        let system = self.inner.read().unwrap();
        system.locked_keys.contains(key)
    }

    /// Replace the locked-keys set from a freshly loaded system-layer table.
    ///
    /// Called during initialisation and on system-layer hot-reload.
    /// Parses `[_locked].locked_keys` from the system layer values.
    ///
    /// Validates: Requirement 18.1, 18.8
    pub fn update_locked_keys(&self, system_layer_values: &crate::value::ConfigTable) {
        let new_locked = extract_locked_keys(system_layer_values);
        let mut system = self.inner.write().unwrap();
        system.locked_keys = new_locked;
    }

    // === Schema query helpers (for Settings Panel) ==========================

    /// List all registered schema entries.
    ///
    /// Returns a snapshot of all schema entries currently registered.
    /// Used by the Settings Panel to enumerate keys for display.
    pub fn list_schema_entries(&self) -> Vec<crate::schema::SchemaEntry> {
        let system = self.inner.read().unwrap();
        system
            .manager
            .schema()
            .list_all()
            .into_iter()
            .cloned()
            .collect()
    }

    // === Audit Log API (Requirement 16) ====================================

    /// Query the in-memory audit log with the given filter.
    ///
    /// Returns matching entries in chronological order.
    /// Validates: Requirement 16.3
    pub fn query_audit_log(&self, filter: &AuditFilter) -> Vec<AuditEntry> {
        let system = self.inner.read().unwrap();
        system.audit_log.query(filter)
    }

    /// Clear the in-memory audit log and truncate the on-disk file.
    ///
    /// Validates: Requirement 16.6
    pub fn clear_audit_log(&self) {
        let dir = self.resolved_audit_dir();
        let mut system = self.inner.write().unwrap();
        system.audit_log.clear();
        if let Some(dir) = dir {
            crate::audit::truncate_log_file(&dir.join("audit.log"));
        }
    }

    /// Resolve the directory the audit log (`audit.log`) lives in.
    ///
    /// When an explicit user-config path is injected (B048 test seam), derive
    /// the audit dir from that file's parent so audit writes stay co-located
    /// with the injected config. Otherwise fall back to
    /// `crate::paths::user_config_dir()` (prior behaviour, production unchanged).
    fn resolved_audit_dir(&self) -> Option<std::path::PathBuf> {
        if let Some(path) = self.user_config_path_injected() {
            return path.parent().map(Path::to_path_buf);
        }
        crate::paths::user_config_dir()
    }

    /// The explicitly injected user-config path, if any (without the fallback to
    /// `crate::paths::user_config_path()`). Used by the audit-dir resolver so a
    /// `None` injection falls back to `user_config_dir()`, not a derived parent.
    fn user_config_path_injected(&self) -> Option<std::path::PathBuf> {
        let system = self.inner.read().unwrap();
        system.user_config_path.clone()
    }

    /// Append an audit entry to the in-memory log and persist it to disk.
    ///
    /// Write failures are logged at WARN level and do not block the caller.
    /// Validates: Requirement 16.1, 16.4
    pub fn record_audit_entry(&self, entry: AuditEntry) {
        let audit_path = self.resolved_audit_dir().map(|d| d.join("audit.log"));
        let mut system = self.inner.write().unwrap();
        if let Some(ref path) = audit_path {
            crate::audit::persist_entry(path, &entry);
        }
        system.audit_log.record(entry);
    }
}
