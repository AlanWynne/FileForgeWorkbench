# Implementation Plan: Configuration System (`ff-config`)

## Overview

This plan covers the complete implementation of the `ff-config` crate — the central settings management layer for FileForgeWorkbench. It provides TOML-based configuration files, a six-layer override model (Defaults → System → User → Profile → Project → Workspace), hot-reload with debounced file watching, named user profiles, per-project overrides, EditorConfig integration, a typed access API with compile-time key definitions, plugin namespace scoping, and runtime-queryable schema validation.

This is a **Wave 2 (Platform Architecture)** sub-project depending on `ff-logging` (Wave 0).

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-config/Cargo.toml` with dependencies (toml, notify, dirs, regex, thiserror, serde, serde_derive, glob, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-config/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `value.rs`, `layer.rs`, `loader.rs`, `merger.rs`, `store.rs`, `access.rs`, `provenance.rs`, `watcher.rs`, `reload.rs`, `callback.rs`, `profile.rs`, `namespace.rs`, `plugin_handle.rs`, `paths.rs`, `keys.rs`, `init.rs`, `error.rs`
  - [x] 1.4 Create schema submodule files: `schema/mod.rs`, `schema/registry.rs`, `schema/entry.rs`, `schema/constraint.rs`
  - [x] 1.5 Create editorconfig submodule files: `editorconfig/mod.rs`, `editorconfig/parser.rs`, `editorconfig/resolver.rs`
  - [x] 1.6 Add `ff-config` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Core value types and layer enumeration
  - [x] 2.1 Define `ConfigValue` enum (String, Integer, Float, Boolean, Array, Table) with `#[non_exhaustive]`
  - [x] 2.2 Define `ConfigTable` type alias (`BTreeMap<String, ConfigValue>`)
  - [x] 2.3 Define `ConfigLayer` enum with fixed ordering (Defaults=0, System=1, User=2, Profile=3, Project=4, Workspace=5) implementing `Ord`
  - [x] 2.4 Define `Provenance` struct with `layer` and `source_file` fields
  - [x] 2.5 Define `EffectiveValue` struct combining `ConfigValue` and `Provenance`
  - [x] 2.6 Write unit tests for layer ordering and value type equality
  - Covers: Requirement 1 (AC 1.4), Requirement 2 (AC 2.1, 2.3)

- [x] 3. Error types
  - [x] 3.1 Define `ConfigError` enum with variants: ParseError, UndefinedKey, TypeMismatch, ValidationFailed, NamespaceViolation, InvalidPluginName, ReservedNamespace, ProfileNotFound, WatcherError, Io, SchemaConflict, EditorConfigParseError
  - [x] 3.2 Implement `thiserror::Error` with formatted messages following `[config] operation: description` pattern
  - [x] 3.3 Define `ValueType` enum (String, Integer, Float, Boolean, Array, Table)
  - [x] 3.4 Write unit tests for error message format and Display implementations
  - Covers: Requirement 1 (AC 1.6), Requirement 2 (AC 2.6), Requirement 7 (AC 7.5, 7.6), Requirement 8 (AC 8.1, 8.3, 8.7)

- [x] 4. Platform-specific path resolution
  - [x] 4.1 Implement system config path resolution (Linux: `/etc/ffworkbench/config.toml`, Windows: `%PROGRAMDATA%\FFWorkbench\config.toml`, macOS: `/Library/Application Support/FFWorkbench/config.toml`)
  - [x] 4.2 Implement user config path resolution (Linux: `$XDG_CONFIG_HOME/ffworkbench/config.toml`, Windows: `%APPDATA%\FFWorkbench\config.toml`, macOS: `~/Library/Application Support/FFWorkbench/config.toml`)
  - [x] 4.3 Implement user profiles directory resolution (`profiles/` subdirectory of user config dir)
  - [x] 4.4 Implement languages directory resolution (`languages/` subdirectory of user config dir)
  - [x] 4.5 Implement project-layer path detection (`.ffworkbench/config.toml` in project root)
  - [x] 4.6 Write unit tests for path resolution on current platform
  - Covers: Requirement 1 (AC 1.2, 1.5), Requirement 4 (AC 4.1), Requirement 5 (AC 5.1)

- [x] 5. TOML loader
  - [x] 5.1 Implement `load_toml_file(path) → Result<ConfigTable, ConfigError>` that parses a TOML file into a ConfigTable
  - [x] 5.2 Implement syntax error handling: reject entire file, return `ConfigError::ParseError` with file path and error location
  - [x] 5.3 Implement I/O error handling for unreadable files (permission errors, missing files)
  - [x] 5.4 Implement `LayerData` struct holding layer identity, source path, and parsed values
  - [x] 5.5 Implement namespace hierarchy validation (settings organized into TOML tables: `[logging]`, `[editor]`, `[theme]`, `[plugins]`, `[vfs]`)
  - [x] 5.6 Write unit tests for valid TOML parsing, syntax error rejection, and I/O error paths
  - Covers: Requirement 1 (AC 1.1, 1.3, 1.6), Requirement 5 (AC 5.7)

- [x] 6. Schema registry and entry definitions
  - [x] 6.1 Define `SchemaEntry` struct with key, value_type, default, description, and optional constraints
  - [x] 6.2 Define `Constraints` struct with optional min, max, allowed_values, and pattern fields
  - [x] 6.3 Implement `SchemaRegistry` with methods: `register(entry)`, `get(key)`, `list_all()`, `deregister(prefix)`
  - [x] 6.4 Implement duplicate key detection — reject re-registration with different type via `SchemaConflict` error
  - [x] 6.5 Implement runtime schema growth — allow registration of new keys during plugin initialization
  - [x] 6.6 Write unit tests for registration, lookup, listing, deregistration, and conflict detection
  - Covers: Requirement 9 (AC 9.1, 9.2, 9.3, 9.5, 9.7)

- [x] 7. Schema validation engine
  - [x] 7.1 Implement type validation: check loaded value type against schema-declared ValueType
  - [x] 7.2 Implement numeric range validation (min/max constraints for Integer and Float)
  - [x] 7.3 Implement enum validation (allowed_values constraint for String and Integer)
  - [x] 7.4 Implement regex pattern validation for String values
  - [x] 7.5 Implement validation failure handling: discard invalid value, apply schema default, emit WARN log
  - [x] 7.6 Implement unknown key handling: ignore key, emit DEBUG log, no error or WARN
  - [x] 7.7 Write unit tests for each constraint type, failure handling, and unknown key behavior
  - Covers: Requirement 7 (AC 7.4, 7.5, 7.6), Requirement 9 (AC 9.4, 9.6)

- [x] 8. Layer merger — recursive key-by-key merge
  - [x] 8.1 Implement recursive table merge: when two layers define the same TOML table, merge their keys rather than replacing
  - [x] 8.2 Implement key-by-key conflict resolution: highest-priority layer wins for scalar values
  - [x] 8.3 Implement full six-layer merge producing `EffectiveStore` with provenance for every key
  - [x] 8.4 Implement provenance tracking: record which layer and source file provided each effective value
  - [x] 8.5 Implement schema default fallback: keys not defined in any layer use schema-declared default
  - [x] 8.6 Implement undefined key detection: return `UndefinedKey` error for keys with no schema and no layer value
  - [x] 8.7 Write unit tests for recursive merge, priority resolution, provenance accuracy, and undefined key error
  - Covers: Requirement 2 (AC 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7)

- [x] 9. Typed access API
  - [x] 9.1 Implement `get_string(key) → Result<String, ConfigError>` with type checking and validation
  - [x] 9.2 Implement `get_int(key) → Result<i64, ConfigError>` with type checking and validation
  - [x] 9.3 Implement `get_float(key) → Result<f64, ConfigError>` with type checking and validation
  - [x] 9.4 Implement `get_bool(key) → Result<bool, ConfigError>` with type checking and validation
  - [x] 9.5 Implement `get_array(key) → Result<Vec<ConfigValue>, ConfigError>`
  - [x] 9.6 Implement `get_table(key) → Result<ConfigTable, ConfigError>`
  - [x] 9.7 Implement `get(key) → Result<ConfigValue, ConfigError>` generic getter without type coercion
  - [x] 9.8 Implement `get_with_provenance(key) → Result<EffectiveValue, ConfigError>`
  - [x] 9.9 Implement type mismatch fallback: return schema default and emit WARN log on wrong type
  - [x] 9.10 Implement validation failure fallback: return schema default and emit WARN log on constraint violation
  - [x] 9.11 Write unit tests for all getter methods, type mismatch fallback, and validation fallback
  - Covers: Requirement 7 (AC 7.1, 7.2, 7.3, 7.5, 7.6, 7.7), Requirement 2 (AC 2.3)

- [x] 10. Compile-time key definitions
  - [x] 10.1 Define const key definitions in `keys.rs` for editor settings (tab_size, indent_style, line_endings, trim_trailing_whitespace, insert_final_newline)
  - [x] 10.2 Define const key definitions for logging settings (level, directory, max_file_size_mb, max_retained_files)
  - [x] 10.3 Define const key definitions for theme settings (active, font_size)
  - [x] 10.4 Define const key definitions for VFS settings (default_provider)
  - [x] 10.5 Write unit tests verifying key constants are valid dot-separated paths
  - Covers: Requirement 7 (AC 7.2)

- [x] 11. File watcher with debounce
  - [x] 11.1 Implement OS-native file watcher using `notify` crate (inotify on Linux, ReadDirectoryChangesW on Windows, FSEvents on macOS)
  - [x] 11.2 Implement watch registration for all loaded configuration file paths
  - [x] 11.3 Implement debounce logic: coalesce multiple events for the same file within a 500ms window into a single reload
  - [x] 11.4 Implement change detection within 2 seconds of file modification
  - [x] 11.5 Implement watcher initialization error handling via `ConfigError::WatcherError`
  - [x] 11.6 Write unit tests for debounce coalescing, watch registration, and error handling
  - Covers: Requirement 3 (AC 3.1, 3.2, 3.7)

- [x] 12. Hot-reload orchestration
  - [x] 12.1 Implement reload pipeline: re-read file → parse TOML → validate schema → re-merge layers → compute diff → notify callbacks
  - [x] 12.2 Implement atomic change application: all changed values from a single file applied together or none applied
  - [x] 12.3 Implement reload failure handling: on invalid TOML or schema failure, reject reload, retain previous values, emit WARN log
  - [x] 12.4 Implement `ReloadEvent` struct with changed_keys, source_layer, and timestamp
  - [x] 12.5 Implement `reload()` method for manual forced reload of all configuration files
  - [x] 12.6 Write unit tests for successful reload, atomic apply, and failure rejection
  - Covers: Requirement 3 (AC 3.2, 3.3, 3.5, 3.6), Requirement 5 (AC 5.5)

- [x] 13. Reload callback management
  - [x] 13.1 Define `ReloadCallback` type (`Box<dyn Fn(&ReloadEvent) + Send + Sync>`)
  - [x] 13.2 Define `CallbackHandle` for deregistration
  - [x] 13.3 Implement `on_reload(keys, callback) → CallbackHandle` registration method
  - [x] 13.4 Implement `remove_callback(handle)` deregistration method
  - [x] 13.5 Implement callback invocation: invoke only callbacks registered for keys whose effective value changed
  - [x] 13.6 Implement callback invocation ordering: callbacks run after state update, no lock held during invocation
  - [x] 13.7 Write unit tests for registration, deregistration, selective invocation, and thread safety
  - Covers: Requirement 3 (AC 3.3, 3.4)

- [x] 14. User profile management
  - [x] 14.1 Implement profile discovery: scan profiles directory for TOML files, return `Vec<UserProfile>` with name and path
  - [x] 14.2 Implement `set_active_profile(name)`: load profile file, insert as Profile layer, recompute effective values, invoke callbacks
  - [x] 14.3 Implement profile deactivation: pass `None` to remove profile layer and recompute
  - [x] 14.4 Implement single-activation invariant: activating a new profile automatically deactivates the previous one
  - [x] 14.5 Implement active profile persistence: store selection in user-layer `[_session].active_profile` key
  - [x] 14.6 Implement profile auto-activation on startup: read persisted profile selection and activate if available
  - [x] 14.7 Implement missing profile handling: emit WARN log, deactivate profile, continue operating
  - [x] 14.8 Write unit tests for discovery, activation, deactivation, persistence, and missing profile handling
  - Covers: Requirement 4 (AC 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7)

- [x] 15. Per-project configuration management
  - [x] 15.1 Implement `load_project(project_root)`: detect `.ffworkbench/config.toml`, load and merge at Project layer priority
  - [x] 15.2 Implement `unload_project()`: remove project layer, recompute effective values, invoke callbacks for changed keys
  - [x] 15.3 Implement automatic project config detection when project is opened
  - [x] 15.4 Implement project-layer hot-reload: monitor project config file for changes per Requirement 3
  - [x] 15.5 Implement project config load failure handling: emit WARN, skip project layer, continue operating
  - [x] 15.6 Write unit tests for project load, unload, hot-reload integration, and failure handling
  - Covers: Requirement 5 (AC 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7)

- [x] 16. EditorConfig parser
  - [x] 16.1 Implement `.editorconfig` file parser supporting all standard properties: indent_style, indent_size, tab_width, end_of_line, charset, trim_trailing_whitespace, insert_final_newline
  - [x] 16.2 Define `EditorConfigProperties` struct with optional fields for each property
  - [x] 16.3 Define enums: `IndentStyle` (Space, Tab), `EndOfLine` (Lf, CrLf, Cr), `Charset` (Utf8, Utf8Bom, Latin1, Utf16Be, Utf16Le)
  - [x] 16.4 Implement glob pattern matching for EditorConfig section headers
  - [x] 16.5 Implement parse error handling: skip invalid file, emit WARN log, continue resolution
  - [x] 16.6 Write unit tests for property parsing, glob matching, and error handling
  - Covers: Requirement 6 (AC 6.1, 6.2, 6.6)

- [x] 17. EditorConfig resolver
  - [x] 17.1 Implement path traversal resolution: walk from file's directory up to `root = true` file or filesystem root
  - [x] 17.2 Implement multi-file merge: closer (deeper) files take priority over farther (shallower) files
  - [x] 17.3 Implement `resolve_editorconfig(file_path) → EditorConfigProperties` on ConfigHandle
  - [x] 17.4 Implement EditorConfig precedence: EditorConfig properties override ALL configuration layers for their specific properties on a per-file basis
  - [x] 17.5 Implement scope restriction: EditorConfig only applies to file-specific editor settings, not to logging/theme/plugin/vfs settings
  - [x] 17.6 Write unit tests for path traversal, multi-file merge, precedence over layers, and scope restriction
  - Covers: Requirement 6 (AC 6.3, 6.4, 6.5, 6.7)

- [x] 18. Plugin namespace scoping
  - [x] 18.1 Implement namespace validation: plugin names must be lowercase ASCII, hyphens, and digits only
  - [x] 18.2 Implement namespace scoping: plugin config under `[plugins.{plugin-name}]` TOML table
  - [x] 18.3 Implement `PluginConfigHandle` struct with namespace prefix and inner ConfigHandle reference
  - [x] 18.4 Implement scoped getters on `PluginConfigHandle`: relative keys auto-prefixed with `plugins.{plugin-name}.`
  - [x] 18.5 Implement scoped `set()` method for plugin writes (persists to user-layer file)
  - [x] 18.6 Implement namespace violation detection: reject read/write outside plugin's namespace with `NamespaceViolation` error
  - [x] 18.7 Implement reserved namespace enforcement: reject plugin registration under `logging`, `editor`, `theme`, `vfs`, `commands`, `layout`, `core`, `_session`
  - [x] 18.8 Implement `create_plugin_config_handle(config, plugin_name) → Result<PluginConfigHandle, ConfigError>`
  - [x] 18.9 Write unit tests for namespace validation, scoped access, violation detection, and reserved namespace enforcement
  - Covers: Requirement 8 (AC 8.1, 8.2, 8.3, 8.7)

- [x] 19. Plugin configuration lifecycle
  - [x] 19.1 Implement plugin default registration: plugins declare defaults in manifest, registered as Defaults layer for plugin namespace
  - [x] 19.2 Implement plugin reload callback registration via `PluginConfigHandle::on_reload()`
  - [x] 19.3 Implement plugin hot-reload: changes to plugin namespace in config files invoke plugin's reload callbacks
  - [x] 19.4 Implement plugin unload cleanup: deregister callbacks, remove schema entries; persisted values retained in files
  - [x] 19.5 Write unit tests for plugin default registration, reload notification, and unload cleanup
  - Covers: Requirement 8 (AC 8.4, 8.5, 8.6)

- [x] 20. ConfigHandle and thread safety
  - [x] 20.1 Implement `ConfigHandle` as `Arc<RwLock<ConfigSystem>>` — thread-safe, clonable, shareable
  - [x] 20.2 Implement read access pattern: typed getters acquire read lock, return owned values (cloned)
  - [x] 20.3 Implement write access pattern: reload and profile switch acquire write lock briefly for atomic swap
  - [x] 20.4 Implement callback invocation after releasing write lock (no lock held during callbacks)
  - [x] 20.5 Write unit tests for concurrent read access and thread safety (`Send + Sync` verification)
  - Covers: Design §9 (Concurrency Model), Requirement 3 (AC 3.5)

- [x] 21. Initialization and shutdown
  - [x] 21.1 Implement `init(options: ConfigInitOptions) → Result<ConfigHandle, ConfigError>` initialization sequence
  - [x] 21.2 Implement initialization ordering: load schema defaults → load system file → load user file → auto-activate persisted profile → detect and load project file → detect and load workspace file → start file watcher
  - [x] 21.3 Implement `ConfigInitOptions` struct with optional project_root, workspace_root, and enable_hot_reload fields
  - [x] 21.4 Implement `shutdown(handle)`: stop file watching, deregister all callbacks
  - [x] 21.5 Implement graceful handling of missing layer files (skip missing files silently, load available layers)
  - [x] 21.6 Write unit tests for successful init, partial layer availability, and shutdown cleanup
  - Covers: Requirement 1 (AC 1.1, 1.2), Requirement 2 (AC 2.1), Requirement 4 (AC 4.5)

- [x] 22. ConfigProvider trait implementation
  - [x] 22.1 Implement `ff_core::ConfigProvider` trait on `ConfigHandle` with `get<T>(namespace, key)` and `get_namespace(namespace)` methods
  - [x] 22.2 Implement serde deserialization bridge for typed access via ConfigProvider
  - [x] 22.3 Write unit tests for ConfigProvider trait integration
  - Covers: Design §5 (Integration with ff-core)

- [x] 23. Integration tests
  - [x] 23.1 Write end-to-end test: full initialization with all layers, query effective values, verify provenance
  - [x] 23.2 Write end-to-end test: hot-reload cycle — modify file on disk, verify callback invocation with correct changed keys
  - [x] 23.3 Write end-to-end test: profile switch — activate profile, verify effective values change, switch back
  - [x] 23.4 Write end-to-end test: project load/unload — open project, verify overrides, close project, verify revert
  - [x] 23.5 Write end-to-end test: EditorConfig resolution — create .editorconfig hierarchy, verify per-file resolution
  - [x] 23.6 Write end-to-end test: plugin scoped access — create handle, verify isolation, verify namespace violation
  - [x] 23.7 Write end-to-end test: schema validation at load time — invalid values replaced by defaults
  - Covers: All requirements (integration validation)

- [x] 24. Property-based tests
  - [x] 24.1 Write PBT: layer precedence determinism property
  - [x] 24.2 Write PBT: recursive table merge property
  - [x] 24.3 Write PBT: schema validation fallback property
  - [x] 24.4 Write PBT: namespace isolation property
  - [x] 24.5 Write PBT: hot-reload atomicity property
  - [x] 24.6 Write PBT: debounce coalescing property
  - [x] 24.7 Write PBT: profile layer placement property
  - [x] 24.8 Write PBT: EditorConfig precedence property
  - [x] 24.9 Write PBT: unknown key tolerance property
  - [x] 24.10 Write PBT: provenance accuracy property
  - [x] 24.11 Write PBT: reserved namespace enforcement property
  - [x] 24.12 Write PBT: profile single-activation invariant property
  - Covers: Design §10 (Correctness Properties 1–12)

---

## Property-Based Test Definitions

### Property 1: Layer Precedence Determinism

**Validates: Requirement 2.1, 2.2**

- **Statement:** For any set of layer values for the same key, the effective value is always the value from the highest-priority layer that defines the key. The result is deterministic and independent of insertion order.
- **Strategy:** Generate `Vec<(ConfigLayer, ConfigValue)>` pairs for the same key, with random subset of layers defining the key and random insertion orderings.
- **Invariant:** `effective_value == value_from_max_priority_layer_that_defines_key`; re-running merge with shuffled insertion order produces identical result.

### Property 2: Recursive Table Merge

**Validates: Requirement 2.7**

- **Statement:** For any two TOML tables at different layers defining overlapping keys within a nested table, the merge produces a table containing all keys from both layers, with higher-priority values winning on conflict — recursively for nested tables.
- **Strategy:** Generate two `ConfigTable` values with a mix of overlapping and disjoint keys, including nested tables up to 3 levels deep.
- **Invariant:** Merged table contains union of all keys; conflicting leaf values use higher-layer value; nested tables are merged recursively (not replaced wholesale).

### Property 3: Schema Validation Fallback

**Validates: Requirement 7.5, 7.6; Requirement 9.4**

- **Statement:** For any schema entry with a default value, and any stored value that violates the schema constraints (wrong type, out of range, not in enum set, fails regex), the typed getter returns the schema default — never the invalid value.
- **Strategy:** Generate `SchemaEntry` with random constraints (min/max for numerics, allowed_values for enums, regex for strings); generate `ConfigValue` that deliberately violates at least one constraint.
- **Invariant:** `get_typed(key) == schema_entry.default`; a WARN-level log is emitted with key name and violation details.

### Property 4: Namespace Isolation

**Validates: Requirement 8.3**

- **Statement:** For any plugin name P and any key K where K does not start with `"plugins.{P}."`, accessing K through a PluginConfigHandle for P always returns a NamespaceViolation error.
- **Strategy:** Generate valid `plugin_name` (lowercase ASCII + hyphens + digits, 1–32 chars); generate `key` that either belongs to a different plugin namespace, a core namespace, or an arbitrary path not under `plugins.{plugin_name}.`.
- **Invariant:** `handle.get(key) == Err(ConfigError::NamespaceViolation { plugin: P, key: K, namespace: "plugins.{P}" })`

### Property 5: Hot-Reload Atomicity

**Validates: Requirement 3.5**

- **Statement:** For any reload event affecting N keys, either all N keys are updated to their new effective values simultaneously, or none are updated. There is no observable intermediate state where some keys reflect new values and others reflect old values from the same file.
- **Strategy:** Generate a set of 2–20 key-value changes for a single layer file. Snapshot effective values before and after reload on a concurrent reader thread.
- **Invariant:** The reader thread either sees all old values or all new values for the set of changed keys — never a mix.

### Property 6: Debounce Coalescing

**Validates: Requirement 3.7**

- **Statement:** For any sequence of file modification events for the same file arriving within a 500ms window, exactly one reload operation is performed. The reload uses the final file state.
- **Strategy:** Generate sequence of 2–10 `(file_path, timestamp)` events within 500ms window with varying inter-event gaps.
- **Invariant:** `reload_count == 1` for that file within the window; the reloaded content matches the final write.

### Property 7: Profile Layer Placement

**Validates: Requirement 4.2, 4.3; Requirement 2.1**

- **Statement:** For any active profile defining keys that are also defined in the User layer and the Project layer, the effective value for each key follows strictly: Project > Profile > User.
- **Strategy:** Generate values for the same key at User, Profile, and Project layers with distinct values. Test all combinations of which layers define the key.
- **Invariant:** `effective == Project_value` if Project defines it; else `effective == Profile_value` if Profile defines it; else `effective == User_value`.

### Property 8: EditorConfig Precedence

**Validates: Requirement 6.3**

- **Statement:** For any file path where EditorConfig defines a property (indent_style, tab_width, etc.), the EditorConfig value takes precedence over ALL configuration layers for that specific property for that specific file.
- **Strategy:** Generate a configuration value at Workspace layer (highest file-based priority) and an EditorConfig value for the same property for a matching file path.
- **Invariant:** Resolved value for that file == EditorConfig value (not Workspace value), for all seven supported properties.

### Property 9: Unknown Key Tolerance

**Validates: Requirement 9.6**

- **Statement:** For any TOML file containing keys that have no schema entry, loading succeeds without error. Unknown keys are silently ignored (DEBUG log only). All known keys in the file are loaded normally.
- **Strategy:** Generate a TOML table mixing 1–5 known schema keys (with valid values) and 1–10 arbitrary unknown keys (random dot-separated paths not in schema).
- **Invariant:** Load succeeds; all known keys accessible with correct values; no WARN-level log emitted for unknown keys; DEBUG log emitted for each unknown key.

### Property 10: Provenance Accuracy

**Validates: Requirement 2.3**

- **Statement:** For any effective value returned by `get_with_provenance()`, the reported provenance layer and source file accurately reflect which layer and file provided the winning value.
- **Strategy:** Generate 2–6 layers with various subsets of keys defined. Query effective values and their provenance.
- **Invariant:** `provenance.layer == highest_priority_layer_that_defines_key`; `provenance.source_file` matches the file for that layer (or `None` for Defaults layer).

### Property 11: Reserved Namespace Enforcement

**Validates: Requirement 8.7**

- **Statement:** For any plugin attempting to register schema keys under a reserved core namespace (`logging`, `editor`, `theme`, `vfs`, `commands`, `layout`, `core`, `_session`), the registration fails with a ReservedNamespace error.
- **Strategy:** Generate valid `plugin_name`; generate a `SchemaEntry` with key path starting with one of the 8 reserved prefixes.
- **Invariant:** `register_schema(entry)` returns `Err(ConfigError::ReservedNamespace { plugin, namespace })` for every reserved prefix.

### Property 12: Profile Single-Activation Invariant

**Validates: Requirement 4.3, 4.4**

- **Statement:** At any point in time, at most one profile is active. Activating a new profile automatically deactivates the previous one. After deactivation, no profile-layer values influence effective values.
- **Strategy:** Generate a sequence of 3–10 `set_active_profile` calls with random profile names (some valid, some None for deactivation).
- **Invariant:** After each call, `active_profile() == last_set_profile`; only one profile's values are present in the effective store at the Profile layer; after `set_active_profile(None)`, no Profile-layer values exist in effective store.

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and Errors", "tasks": ["2", "3", "4"], "dependsOn": [0] },
    { "id": 2, "label": "Loader and Schema", "tasks": ["5", "6", "7", "10"], "dependsOn": [1] },
    { "id": 3, "label": "Merge and Access", "tasks": ["8", "9"], "dependsOn": [2] },
    { "id": 4, "label": "File Watching and Reload", "tasks": ["11", "12", "13"], "dependsOn": [3] },
    { "id": 5, "label": "Profiles and Projects", "tasks": ["14", "15"], "dependsOn": [4] },
    { "id": 6, "label": "EditorConfig", "tasks": ["16", "17"], "dependsOn": [3] },
    { "id": 7, "label": "Plugin Scoping", "tasks": ["18", "19"], "dependsOn": [4] },
    { "id": 8, "label": "System Assembly", "tasks": ["20", "21", "22"], "dependsOn": [5, 6, 7] },
    { "id": 9, "label": "Validation and PBT", "tasks": ["23", "24"], "dependsOn": [8] }
  ]
}
```

---

## Notes

- This is a Wave 2 (Platform Architecture) crate depending only on `ff-logging` (Wave 0)
- All other workspace crates consume `ff-config` — the public API surface must be stable before downstream work begins
- The `notify` crate provides cross-platform file watching; debounce logic is implemented within `ff-config` (not using notify's built-in debouncer)
- EditorConfig resolution is per-file and does not use the layered model; it is a separate resolution path that overrides layers for specific properties
- Plugin configuration isolation is enforced at the API level — plugins receive a `PluginConfigHandle` that only permits access to their namespace
- The `[_session]` table in user config is reserved for internal persistence (active profile); it is not exposed to plugins or schema queries
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Thread-safety uses `std::sync::RwLock` and `Arc` — no async runtime dependency
- Configuration files are read via direct filesystem access (not VFS) since ff-config initializes before VFS is available (FFW-ARCH-001)
- Language profile files (`languages/*.toml`) are loaded as part of the User layer but stored in separate files per Requirement 1 AC 1.5

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: TOML Format | AC 1.1–1.6 | Tasks 5, 4, 10, 21 |
| Req 2: Layered Model | AC 2.1–2.7 | Tasks 2, 8, 9, 24 (PBT 1, 2, 10) |
| Req 3: Hot-Reload | AC 3.1–3.7 | Tasks 11, 12, 13, 20, 24 (PBT 5, 6) |
| Req 4: User Profiles | AC 4.1–4.7 | Tasks 14, 24 (PBT 7, 12) |
| Req 5: Per-Project | AC 5.1–5.7 | Tasks 4, 15 |
| Req 6: EditorConfig | AC 6.1–6.7 | Tasks 16, 17, 24 (PBT 8) |
| Req 7: Typed Access API | AC 7.1–7.7 | Tasks 9, 10, 7, 24 (PBT 3) |
| Req 8: Plugin Scoping | AC 8.1–8.7 | Tasks 18, 19, 24 (PBT 4, 11) |
| Req 9: Schema & Validation | AC 9.1–9.7 | Tasks 6, 7, 24 (PBT 9) |

---

## Settings Panel Tasks (Requirement 15)

- [x] 25. `set_user_value` and `remove_user_value` on `ConfigHandle`
  - [x] 25.1 Add `set_user_value(key: &str, value: ConfigValue) -> Result<(), ConfigError>` to
          `ConfigHandle` — writes key to user-layer TOML file atomically, triggers hot-reload
    - Validates: Requirement 15.4
  - [x] 25.2 Add `remove_user_value(key: &str) -> Result<(), ConfigError>` to `ConfigHandle` —
          removes key from user-layer TOML file, triggers hot-reload
    - Validates: Requirement 15.6
  - [x] 25.3 Write unit tests: `set_user_value_persists_to_file`, `remove_user_value_restores_default`
    - Validates: Requirement 15.4, 15.6
  - [x] 25.4 Run `cargo test -p ff-config` — confirm green

- [x] 26. `TabKind::SettingsPanel` and shell routing
  - [x] 26.1 Add `SettingsPanel` variant to `TabKind` enum in `tab_state.rs`
    - Validates: Requirement 15.1, 15.9
  - [x] 26.2 Update `handle_command()` in `shell.rs`: route `"0"`, `"SETTINGS"`, and `"=0"` to
          open/activate a `SettingsPanel` tab
    - Validates: Requirement 15.1
  - [x] 26.3 Update `render_central_panel()` to dispatch `TabKind::SettingsPanel` →
          `settings_panel::render(ui, state, config_handle)`
    - Validates: Requirement 15.1
  - [x] 26.4 Update POM option 0 button to route to Settings panel (same as typing `0`)
    - Validates: Requirement 15.11
  - [x] 26.5 Update session persistence to save/restore `SettingsPanel` tab kind
    - Validates: Requirement 15.9
  - [x] 26.6 Write unit tests: `settings_panel_tab_kind_exists`, `command_0_routes_to_settings`,
          `command_settings_routes_to_settings`, `command_equals_0_routes_to_settings`
    - Validates: Requirement 15.1
  - [x] 26.7 Run `cargo test -p ff-desktop` — confirm green

- [x] 27. `SettingsPanelState` and render skeleton
  - [x] 27.1 Create `crates/ff-desktop/src/settings_panel.rs` with `SettingsPanelState` struct
          (filter text, section collapse state map, pending edits map, validation errors map)
    - Validates: Requirement 15.2, 15.7
  - [x] 27.2 Implement namespace grouping: collect schema entries, group by first dot-segment,
          sort groups and keys within groups
    - Validates: Requirement 15.2
  - [x] 27.3 Implement collapsible section headers per namespace group
    - Validates: Requirement 15.2
  - [x] 27.4 Implement filter input — case-insensitive substring match on key path and description
    - Validates: Requirement 15.7
  - [x] 27.5 Implement source file path footer (read from `UserDataDir`)
    - Validates: Requirement 15.8
  - [x] 27.6 Implement F3/END command to return tab to POM view
    - Validates: Requirement 15.10
  - [x] 27.7 Write unit tests: `namespace_grouping_correct`, `filter_hides_non_matching_keys`,
          `f3_returns_to_pom`
    - Validates: Requirement 15.2, 15.7, 15.10
  - [x] 27.8 Run `cargo test -p ff-desktop` — confirm green

- [x] 28. Per-key value widgets and provenance display
  - [x] 28.1 Implement Boolean widget: `egui::Checkbox` bound to effective bool value
    - Validates: Requirement 15.3
  - [x] 28.2 Implement Integer/Float with min+max: `egui::Slider`
    - Validates: Requirement 15.3
  - [x] 28.3 Implement Integer/Float without constraints: numeric `egui::TextEdit`
    - Validates: Requirement 15.3
  - [x] 28.4 Implement String with `allowed_values`: `egui::ComboBox` drop-down
    - Validates: Requirement 15.3
  - [x] 28.5 Implement String without constraints: `egui::TextEdit` single-line
    - Validates: Requirement 15.3
  - [x] 28.6 Implement provenance badge label (Default / User / System / Profile / Project /
          Workspace) derived from `get_with_provenance()`
    - Validates: Requirement 15.3
  - [x] 28.7 Write unit tests: `widget_type_selected_for_bool`, `widget_type_selected_for_enum_string`,
          `widget_type_selected_for_bounded_int`, `provenance_badge_shows_correct_layer`
    - Validates: Requirement 15.3
  - [x] 28.8 Run `cargo test -p ff-desktop` — confirm green

- [x] 29. Write path, validation, and Reset to Default
  - [x] 29.1 Implement on-change handler: validate new value against schema constraints;
          on success call `config_handle.set_user_value(key, value)`; on failure set inline error
    - Validates: Requirement 15.4, 15.5
  - [x] 29.2 Implement inline validation error display adjacent to the offending field
    - Validates: Requirement 15.5
  - [x] 29.3 Implement `Reset to Default` button — visible only when provenance != Default;
          calls `config_handle.remove_user_value(key)`
    - Validates: Requirement 15.6
  - [x] 29.4 Write unit tests: `valid_value_calls_set_user_value`, `invalid_value_shows_error`,
          `reset_to_default_calls_remove_user_value`, `reset_button_hidden_when_at_default`
    - Validates: Requirement 15.4, 15.5, 15.6
  - [x] 29.5 Run `cargo test --workspace` — confirm all tests green
  - [x] 29.6 Run `cargo clippy -p ff-desktop -- -D warnings` — confirm clean
  - [x] 29.7 Update `docs/quality/TCR.md` — mark all Req 15 rows ✅ or 🔲
  - [x] 29.8 Update `docs/specs/project-master/tasks.md` — mark Phase AH complete

---

## Phase CQ Tasks

- [ ] 30. Audit Logging (Requirement 16)
  - [ ] 30.1 Define `AuditEntry` struct (timestamp, key, old_value, new_value, layer, actor)
    - Validates: Requirement 16.5
  - [ ] 30.2 Define `AuditFilter` struct (key_prefix, layer, since, until, actor -- all Option)
    - Validates: Requirement 16.3
  - [ ] 30.3 Implement `AuditLog` ring buffer (max 10,000 entries) with `record()` and `query(filter)` methods
    - Validates: Requirement 16.2, 16.3
  - [ ] 30.4 Wire `AuditLog` into `ConfigSystem`; call `audit.record()` in `set_user_value`, hot-reload diff, profile switch, project load/unload
    - Validates: Requirement 16.1
  - [ ] 30.5 Implement file persistence: append entries to `<user-config-dir>/audit.log` on background thread; handle write failures with WARN log
    - Validates: Requirement 16.2, 16.4
  - [ ] 30.6 Add `query_audit_log(filter)` and `clear_audit_log()` to `ConfigHandle`
    - Validates: Requirement 16.3, 16.6
  - [ ] 30.7 Write unit tests: `audit_entry_recorded_on_set_user_value`, `audit_filter_by_key_prefix`, `audit_filter_by_layer`, `audit_filter_by_time_range`, `audit_write_failure_does_not_block_config_change`, `clear_audit_log_empties_buffer`
    - Validates: Requirement 16.1, 16.3, 16.4, 16.6
  - [ ] 30.8 Run `cargo test -p ff-config` -- confirm green

- [ ] 31. Settings Export and Import (Requirement 17)
  - [ ] 31.1 Define `ExportScope` enum (AllLayers, UserLayer, ProjectLayer) and `ImportTarget` enum (UserLayer, ProjectLayer)
    - Validates: Requirement 17.2, 17.5
  - [ ] 31.2 Define `ImportSummary` struct (imported_count, skipped_count, skipped_keys)
    - Validates: Requirement 17.7
  - [ ] 31.3 Implement `export_settings(scope, path)` -- collect values for scope, write TOML with `[_export_meta]` header
    - Validates: Requirement 17.1, 17.3
  - [ ] 31.4 Implement `import_settings(path, target)` -- read file, validate each key against schema, write valid keys to target layer, collect skipped keys in ImportSummary
    - Validates: Requirement 17.4, 17.6, 17.7, 17.8
  - [ ] 31.5 Wire post-import hot-reload cycle so callbacks are notified of changed keys
    - Validates: Requirement 17.9
  - [ ] 31.6 Write unit tests: `export_user_layer_produces_valid_toml`, `export_includes_meta_header`, `import_valid_file_updates_layer`, `import_invalid_values_skipped_in_summary`, `import_bad_toml_returns_parse_error`, `import_triggers_reload_callbacks`
    - Validates: Requirement 17.1, 17.3, 17.4, 17.6, 17.8, 17.9
  - [ ] 31.7 Run `cargo test -p ff-config` -- confirm green

- [ ] 32. Locked Configuration Keys (Requirement 18)
  - [ ] 32.1 Add `KeyLocked { key: String }` variant to `ConfigError` with message `"[config] lock: key '{key}' is locked by system policy and cannot be modified"`
    - Validates: Requirement 18.7
  - [ ] 32.2 Parse `[_locked].locked_keys` from system-layer TOML into `HashSet<String>` on `ConfigSystem`
    - Validates: Requirement 18.1
  - [ ] 32.3 Enforce locked keys in merger: after computing winning layer, override with system-layer value for locked keys; emit DEBUG log when a higher-priority layer value is suppressed
    - Validates: Requirement 18.2, 18.4
  - [ ] 32.4 Guard `set_user_value()`: return `ConfigError::KeyLocked` if key is in locked set
    - Validates: Requirement 18.3
  - [ ] 32.5 Add `is_locked(key: &str) -> bool` to `ConfigHandle`
    - Validates: Requirement 18.5
  - [ ] 32.6 Update Settings panel in `ff-desktop`: call `is_locked()` per key; disable widget and Reset button; show "LOCKED" badge when true
    - Validates: Requirement 18.6
  - [ ] 32.7 Wire hot-reload of system layer to recompute locked set and re-merge affected keys
    - Validates: Requirement 18.8
  - [ ] 32.8 Write unit tests: `locked_key_uses_system_value_despite_user_override`, `set_user_value_locked_key_returns_error`, `is_locked_returns_true_for_locked_key`, `is_locked_returns_false_for_unlocked_key`, `hot_reload_locked_keys_list_recomputes_effective_values`, `higher_layer_value_silently_ignored_for_locked_key`
    - Validates: Requirement 18.1, 18.2, 18.3, 18.4, 18.5, 18.8
  - [ ] 32.9 Run `cargo test --workspace` -- confirm green
  - [ ] 32.10 Run `cargo clippy -- -D warnings` -- confirm clean
  - [ ] 32.11 Update `docs/quality/TCR.md` -- add rows for Req 16, 17, 18
  - [ ] 32.12 Update `docs/specs/project-master/tasks.md` -- add Phase CQ section
