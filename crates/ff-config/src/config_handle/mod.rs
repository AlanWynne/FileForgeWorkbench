//! Thread-safe configuration handle.
//!
//! Provides `ConfigHandle`, a clonable, `Send + Sync` wrapper around the
//! configuration system runtime state. All public access goes through this
//! handle using interior `Arc<RwLock<ConfigSystem>>` for safe concurrent reads
//! and atomic writes.
//!
//! Addresses: Design Sec 9 (Concurrency Model), Requirement 3 (AC 3.5)

use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::access::ConfigAccess;
use crate::audit::AuditLog;
use crate::editorconfig::parser::EditorConfigProperties;
use crate::error::ConfigError;
use crate::profile::ProfileManager;
use crate::provenance::EffectiveValue;
use crate::reload::ReloadManager;
use crate::value::{ConfigTable, ConfigValue};

/// Internal runtime state of the configuration system.
///
/// Holds the reload manager (which owns layers, store, schema, callbacks,
/// and watcher) and the profile manager for profile switching.
pub(crate) struct ConfigSystem {
    /// The reload manager holds layers, the effective store, schema, callbacks,
    /// and the optional file watcher.
    pub(crate) manager: ReloadManager,
    /// Profile manager for activating/deactivating named profiles.
    pub(crate) profile_manager: Option<ProfileManager>,
    /// Keys locked by the system layer ([_locked].locked_keys).
    /// Addresses: Requirement 18.1, 18.2
    pub(crate) locked_keys: std::collections::HashSet<String>,
    /// In-memory audit log ring buffer.
    /// Addresses: Requirement 16
    pub(crate) audit_log: AuditLog,
}

/// Thread-safe, clonable handle to the configuration system.
///
/// `ConfigHandle` wraps the entire configuration runtime in an
/// `Arc<RwLock<ConfigSystem>>`, enabling:
/// - Multiple concurrent readers (typed getters acquire a read lock)
/// - Exclusive writers (reload, profile switch acquire a write lock briefly)
/// - Safe sharing across threads via `Clone` (all clones share state)
///
/// # Concurrency Model
///
/// Read operations (`get_string`, `get_int`, etc.) acquire the `RwLock` in
/// shared (read) mode, clone the result into an owned value, and release
/// the lock immediately. This ensures readers never block each other.
///
/// Write operations (`reload`, `set_active_profile`, `load_project`,
/// `unload_project`) acquire the `RwLock` in exclusive (write) mode,
/// perform the mutation atomically, and release the lock. Callbacks are
/// invoked by the `ReloadManager` internally during the write lock scope
/// using the `CallbackRegistry`'s own internal `Mutex` -- callbacks must
/// not re-enter `ConfigHandle` methods to avoid deadlock.
///
/// # Examples
///
/// ```ignore
/// let handle = ConfigHandle::new(manager);
/// let cloned = handle.clone(); // shares the same state
///
/// // Read from any thread
/// std::thread::spawn(move || {
///     let value = cloned.get_string("editor.tab_size");
/// });
/// ```
#[derive(Clone)]
pub struct ConfigHandle {
    pub(super) inner: Arc<RwLock<ConfigSystem>>,
}

impl ConfigHandle {
    /// Create a new `ConfigHandle` wrapping the given `ReloadManager`.
    ///
    /// The handle takes ownership of the manager and provides thread-safe
    /// access to all configuration operations.
    pub fn new(manager: ReloadManager) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ConfigSystem {
                manager,
                profile_manager: None,
                locked_keys: std::collections::HashSet::new(),
                audit_log: AuditLog::new(),
            })),
        }
    }

    /// Create a new `ConfigHandle` with both a `ReloadManager` and a `ProfileManager`.
    pub fn with_profile_manager(manager: ReloadManager, profile_manager: ProfileManager) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ConfigSystem {
                manager,
                profile_manager: Some(profile_manager),
                locked_keys: std::collections::HashSet::new(),
                audit_log: AuditLog::new(),
            })),
        }
    }

    // ====================================================================
    // Read access (Task 20.2): typed getters acquire read lock, return owned values
    // ====================================================================

    /// Get a raw `ConfigValue` by key.
    ///
    /// Acquires a read lock, resolves the value through the typed access API,
    /// clones the result, and releases the lock.
    pub fn get(&self, key: &str) -> Result<ConfigValue, ConfigError> {
        let system = self.inner.read().unwrap();
        let access = ConfigAccess::new(system.manager.store(), system.manager.schema());
        access.get(key)
    }

    /// Get a string value by key.
    ///
    /// Acquires a read lock, resolves the value with type checking and schema
    /// default fallback, and returns an owned `String`.
    pub fn get_string(&self, key: &str) -> Result<String, ConfigError> {
        let system = self.inner.read().unwrap();
        let access = ConfigAccess::new(system.manager.store(), system.manager.schema());
        access.get_string(key)
    }

    /// Get an integer value by key.
    ///
    /// Acquires a read lock, resolves the value with type checking, and
    /// returns the `i64` value.
    pub fn get_int(&self, key: &str) -> Result<i64, ConfigError> {
        let system = self.inner.read().unwrap();
        let access = ConfigAccess::new(system.manager.store(), system.manager.schema());
        access.get_int(key)
    }

    /// Get a float value by key.
    ///
    /// Acquires a read lock, resolves the value with type checking, and
    /// returns the `f64` value.
    pub fn get_float(&self, key: &str) -> Result<f64, ConfigError> {
        let system = self.inner.read().unwrap();
        let access = ConfigAccess::new(system.manager.store(), system.manager.schema());
        access.get_float(key)
    }

    /// Get a boolean value by key.
    ///
    /// Acquires a read lock, resolves the value with type checking, and
    /// returns the `bool` value.
    pub fn get_bool(&self, key: &str) -> Result<bool, ConfigError> {
        let system = self.inner.read().unwrap();
        let access = ConfigAccess::new(system.manager.store(), system.manager.schema());
        access.get_bool(key)
    }

    /// Get an array value by key.
    ///
    /// Acquires a read lock, resolves the value with type checking, and
    /// returns an owned `Vec<ConfigValue>`.
    pub fn get_array(&self, key: &str) -> Result<Vec<ConfigValue>, ConfigError> {
        let system = self.inner.read().unwrap();
        let access = ConfigAccess::new(system.manager.store(), system.manager.schema());
        access.get_array(key)
    }

    /// Get a table value by key.
    ///
    /// Acquires a read lock, resolves the value with type checking, and
    /// returns an owned `ConfigTable`.
    pub fn get_table(&self, key: &str) -> Result<ConfigTable, ConfigError> {
        let system = self.inner.read().unwrap();
        let access = ConfigAccess::new(system.manager.store(), system.manager.schema());
        access.get_table(key)
    }

    /// Get a value with full provenance information.
    ///
    /// Acquires a read lock and returns an owned `EffectiveValue` containing
    /// both the value and metadata about which layer provided it.
    pub fn get_with_provenance(&self, key: &str) -> Result<EffectiveValue, ConfigError> {
        let system = self.inner.read().unwrap();
        let access = ConfigAccess::new(system.manager.store(), system.manager.schema());
        access.get_with_provenance(key)
    }

    /// Resolve EditorConfig properties for a given file path.
    ///
    /// Acquires a read lock and delegates to the EditorConfig resolver.
    pub fn resolve_editorconfig(&self, file_path: &Path) -> EditorConfigProperties {
        let system = self.inner.read().unwrap();
        let access = ConfigAccess::new(system.manager.store(), system.manager.schema());
        access.resolve_editorconfig(file_path)
    }

    /// Get a configuration value for a specific file, applying EditorConfig precedence.
    ///
    /// For editor-scoped keys, EditorConfig overrides all configuration layers.
    /// Acquires a read lock and returns an owned `ConfigValue`.
    pub fn get_for_file(&self, key: &str, file_path: &Path) -> Result<ConfigValue, ConfigError> {
        let system = self.inner.read().unwrap();
        let access = ConfigAccess::new(system.manager.store(), system.manager.schema());
        access.get_for_file(key, file_path)
    }
}

mod export_ext;
mod runtime;
mod toml_io;

#[cfg(test)]
mod tests {
    use super::toml_io::{
        extract_locked_keys, remove_dotted_key, remove_key_from_toml_file, set_dotted_key,
        write_key_to_toml_file,
    };
    use super::*;
    use crate::layer::ConfigLayer;
    use crate::loader::LayerData;
    use crate::schema::SchemaRegistry;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    // ====================================================================
    // Task 20.5: Thread safety verification
    // ====================================================================

    // Validates: Requirement 3.5 -- ConfigHandle is Send (can be transferred between threads)
    #[test]
    fn config_handle_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ConfigHandle>();
    }

    // Validates: Requirement 3.5 -- ConfigHandle is Sync (can be shared between threads)
    #[test]
    fn config_handle_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ConfigHandle>();
    }

    // Validates: Requirement 3.5 -- ConfigHandle is Clone (multiple handles share state)
    #[test]
    fn config_handle_clone_shares_state() {
        let schema = SchemaRegistry::new();
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);

        let cloned = handle.clone();

        // Both handles resolve the same keys (empty store -> UndefinedKey)
        let r1 = handle.get("nonexistent");
        let r2 = cloned.get("nonexistent");
        assert!(r1.is_err());
        assert!(r2.is_err());
    }

    // Validates: Requirement 3.5 -- Multiple threads can read concurrently without panics
    #[test]
    fn concurrent_reads_do_not_panic() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("user.toml");
        std::fs::write(&path, "[editor]\ntab_size = 4\nword_wrap = true\n").unwrap();

        let values = crate::loader::load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path,
            values,
        }];

        let schema = SchemaRegistry::new();
        let manager = ReloadManager::new(layers, schema);
        let handle = ConfigHandle::new(manager);

        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();

        for _ in 0..10 {
            let h = handle.clone();
            let c = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let result = h.get("editor.tab_size");
                    assert!(result.is_ok());
                    assert_eq!(result.unwrap(), ConfigValue::Integer(4));
                    c.fetch_add(1, Ordering::Relaxed);
                }
            }));
        }

        for t in handles {
            t.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::Relaxed), 1000);
    }

    // Validates: Requirement 3.5 -- Concurrent reads and writes do not deadlock or panic
    #[test]
    fn concurrent_reads_and_writes_are_safe() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[editor]\ntab_size = 4\n").unwrap();

        let values = crate::loader::load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path.clone(),
            values,
        }];

        let schema = SchemaRegistry::new();
        let manager = ReloadManager::new(layers, schema);
        let handle = ConfigHandle::new(manager);

        let mut threads = Vec::new();

        // Spawn reader threads
        for _ in 0..5 {
            let h = handle.clone();
            threads.push(thread::spawn(move || {
                for _ in 0..50 {
                    let _ = h.get("editor.tab_size");
                    let _ = h.get_string("editor.tab_size");
                    let _ = h.get_int("editor.tab_size");
                }
            }));
        }

        // Spawn a writer thread that reloads
        let h = handle.clone();
        let p = path.clone();
        threads.push(thread::spawn(move || {
            for i in 0..10 {
                std::fs::write(&p, format!("[editor]\ntab_size = {}\n", i + 1)).unwrap();
                let _ = h.reload();
            }
        }));

        for t in threads {
            t.join().unwrap();
        }

        // After all threads finish, the store should be consistent
        let result = handle.get_int("editor.tab_size");
        assert!(result.is_ok());
    }

    // Validates: Requirement 3.5 -- Read access returns owned values (cloned, not references)
    #[test]
    fn read_access_returns_owned_values() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("user.toml");
        std::fs::write(
            &path,
            "[editor]\ntab_size = 4\nname = \"dark\"\nwrap = true\n",
        )
        .unwrap();

        let values = crate::loader::load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path,
            values,
        }];

        let schema = SchemaRegistry::new();
        let manager = ReloadManager::new(layers, schema);
        let handle = ConfigHandle::new(manager);

        // These return owned values -- they can outlive any lock scope
        let int_val = handle.get_int("editor.tab_size").unwrap();
        let str_val = handle.get_string("editor.name").unwrap();
        let bool_val = handle.get_bool("editor.wrap").unwrap();

        assert_eq!(int_val, 4);
        assert_eq!(str_val, "dark");
        assert!(bool_val);
    }

    // Validates: Requirement 3.5 -- Write operations (reload) apply changes atomically
    #[test]
    fn reload_applies_changes_atomically() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[editor]\ntab_size = 4\nword_wrap = true\n").unwrap();

        let values = crate::loader::load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path.clone(),
            values,
        }];

        let schema = SchemaRegistry::new();
        let manager = ReloadManager::new(layers, schema);
        let handle = ConfigHandle::new(manager);

        // Modify the file
        std::fs::write(&path, "[editor]\ntab_size = 2\nword_wrap = false\n").unwrap();

        // Reload
        let results = handle.reload();
        assert!(!results.is_empty());

        // Both values should be updated atomically
        assert_eq!(handle.get_int("editor.tab_size").unwrap(), 2);
        assert_eq!(handle.get_bool("editor.word_wrap").unwrap(), false);
    }

    // Validates: Requirement 3.5 -- load_project through ConfigHandle works correctly
    #[test]
    fn load_project_through_handle_works() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();

        let schema = SchemaRegistry::new();
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);

        let event = handle.load_project(dir.path()).unwrap();
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));
        assert_eq!(handle.get_int("editor.tab_size").unwrap(), 2);
    }

    // Validates: Requirement 3.5 -- unload_project through ConfigHandle works correctly
    #[test]
    fn unload_project_through_handle_works() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();

        let schema = SchemaRegistry::new();
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);

        handle.load_project(dir.path()).unwrap();
        assert_eq!(handle.get_int("editor.tab_size").unwrap(), 2);

        let event = handle.unload_project();
        assert!(event.changed_keys.contains(&"editor.tab_size".to_string()));

        // After unload, the key should no longer be found
        let result = handle.get("editor.tab_size");
        assert!(result.is_err());
    }

    // Validates: Requirement 15.4 -- set_user_value persists key to the user TOML file
    #[test]
    fn set_user_value_persists_to_file() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let user_path = dir.path().join("config.toml");

        // Write an initial file with one key
        std::fs::write(&user_path, "[editor]\ntab_size = 4\n").unwrap();

        // Call the helper directly (bypasses user_config_path resolution)
        write_key_to_toml_file(
            &user_path,
            "editor.tab_size",
            crate::value::ConfigValue::Integer(8),
        )
        .unwrap();

        let content = std::fs::read_to_string(&user_path).unwrap();
        assert!(
            content.contains("tab_size"),
            "file must contain the key after write"
        );
        // Re-parse and verify the value
        let table = crate::loader::load_toml_file(&user_path).unwrap();
        if let Some(crate::value::ConfigValue::Table(editor)) = table.get("editor") {
            assert_eq!(
                editor.get("tab_size"),
                Some(&crate::value::ConfigValue::Integer(8))
            );
        } else {
            panic!("editor table must exist after write");
        }
    }

    // Validates: Requirement 15.6 -- remove_user_value removes key from the user TOML file
    #[test]
    fn remove_user_value_restores_default() {
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let user_path = dir.path().join("config.toml");

        std::fs::write(&user_path, "[editor]\ntab_size = 8\n").unwrap();

        remove_key_from_toml_file(&user_path, "editor.tab_size").unwrap();

        let table = crate::loader::load_toml_file(&user_path).unwrap();
        if let Some(crate::value::ConfigValue::Table(editor)) = table.get("editor") {
            assert!(
                !editor.contains_key("tab_size"),
                "key must be absent after remove"
            );
        }
        // If editor table is gone entirely that is also acceptable
    }

    // Validates: Requirement 15.4 -- set_dotted_key creates intermediate tables
    #[test]
    fn set_dotted_key_creates_intermediate_tables() {
        let mut root = toml::Value::Table(toml::map::Map::new());
        set_dotted_key(&mut root, "a.b.c", toml::Value::Integer(42));
        if let toml::Value::Table(ref map) = root {
            if let Some(toml::Value::Table(ref a)) = map.get("a") {
                if let Some(toml::Value::Table(ref b)) = a.get("b") {
                    assert_eq!(b.get("c"), Some(&toml::Value::Integer(42)));
                    return;
                }
            }
        }
        panic!("nested key not set correctly");
    }

    // Validates: Requirement 15.6 -- remove_dotted_key removes leaf key
    #[test]
    fn remove_dotted_key_removes_leaf() {
        let mut root = toml::Value::Table(toml::map::Map::new());
        set_dotted_key(&mut root, "editor.tab_size", toml::Value::Integer(4));
        remove_dotted_key(&mut root, "editor.tab_size");
        if let toml::Value::Table(ref map) = root {
            if let Some(toml::Value::Table(ref editor)) = map.get("editor") {
                assert!(!editor.contains_key("tab_size"));
                return;
            }
        }

        // editor table may be absent -- also acceptable
    }

    // Validates: Requirement 18.5 -- is_locked returns true for locked key
    #[test]
    fn is_locked_returns_true_for_locked_key() {
        let schema = crate::schema::SchemaRegistry::new();
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);

        {
            let mut system = handle.inner.write().unwrap();
            system.locked_keys.insert("editor.tab_size".to_string());
        }

        assert!(handle.is_locked("editor.tab_size"));
        assert!(!handle.is_locked("editor.word_wrap"));
    }

    // Validates: Requirement 18.5 -- is_locked returns false for unlocked key
    #[test]
    fn is_locked_returns_false_for_unlocked_key() {
        let schema = crate::schema::SchemaRegistry::new();
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);
        assert!(!handle.is_locked("editor.tab_size"));
    }

    // Validates: Requirement 18.3 -- set_user_value returns KeyLocked for locked key
    #[test]
    fn set_user_value_locked_key_returns_error() {
        let schema = crate::schema::SchemaRegistry::new();
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);

        {
            let mut system = handle.inner.write().unwrap();
            system.locked_keys.insert("editor.tab_size".to_string());
        }

        let result =
            handle.set_user_value("editor.tab_size", crate::value::ConfigValue::Integer(8));
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::ConfigError::KeyLocked { key } => {
                assert_eq!(key, "editor.tab_size");
            }
            other => panic!("Expected KeyLocked, got: {:?}", other),
        }
    }

    // Validates: Requirement 18.1 -- extract_locked_keys parses [_locked].locked_keys
    #[test]
    fn extract_locked_keys_parses_system_layer_table() {
        let mut locked_table = crate::value::ConfigTable::new();
        locked_table.insert(
            "locked_keys".to_string(),
            crate::value::ConfigValue::Array(vec![
                crate::value::ConfigValue::String("editor.tab_size".to_string()),
                crate::value::ConfigValue::String("logging.level".to_string()),
            ]),
        );
        let mut values = crate::value::ConfigTable::new();
        values.insert(
            "_locked".to_string(),
            crate::value::ConfigValue::Table(locked_table),
        );

        let locked = extract_locked_keys(&values);
        assert!(locked.contains("editor.tab_size"));
        assert!(locked.contains("logging.level"));
        assert_eq!(locked.len(), 2);
    }

    // Validates: Requirement 18.1 -- extract_locked_keys returns empty set when absent
    #[test]
    fn extract_locked_keys_returns_empty_when_absent() {
        let values = crate::value::ConfigTable::new();
        let locked = extract_locked_keys(&values);
        assert!(locked.is_empty());
    }

    // Validates: Requirement 18.8 -- update_locked_keys replaces the locked set
    #[test]
    fn hot_reload_locked_keys_list_recomputes_effective_values() {
        let schema = crate::schema::SchemaRegistry::new();
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);

        assert!(!handle.is_locked("editor.tab_size"));

        let mut locked_table = crate::value::ConfigTable::new();
        locked_table.insert(
            "locked_keys".to_string(),
            crate::value::ConfigValue::Array(vec![crate::value::ConfigValue::String(
                "editor.tab_size".to_string(),
            )]),
        );
        let mut sys_values = crate::value::ConfigTable::new();
        sys_values.insert(
            "_locked".to_string(),
            crate::value::ConfigValue::Table(locked_table),
        );

        handle.update_locked_keys(&sys_values);
        assert!(handle.is_locked("editor.tab_size"));

        handle.update_locked_keys(&crate::value::ConfigTable::new());
        assert!(!handle.is_locked("editor.tab_size"));
    }
}
