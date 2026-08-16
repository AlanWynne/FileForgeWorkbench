# Implementation Plan: Plugin Architecture (`ff-plugin`)

## Overview

This plan covers the complete implementation of the `ff-plugin` crate — the plugin extensibility framework for FileForgeWorkbench. It defines how optional features are packaged, discovered, loaded, and managed through well-defined lifecycle states. Every optional feature (viewers, language services, connectors, macro engines, the database tool) is implemented as a plugin that interacts with the core exclusively through traits and a context object defined here.

This is a **Wave 2 (Platform Architecture)** sub-project, depending only on `ff-logging` (Wave 0).

---

## Tasks

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-plugin/Cargo.toml` with dependencies (semver, toml, thiserror, proptest dev-dep)
  - [ ] 1.2 Create `crates/ff-plugin/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `traits.rs`, `context.rs`, `metadata.rs`, `capability.rs`, `registry.rs`, `capability_registry.rs`, `loader.rs`, `dependency.rs`, `lifecycle.rs`, `version.rs`, `security.rs`, `error.rs`, `event.rs`
  - [ ] 1.4 Add `ff-plugin` to workspace `Cargo.toml` members list
  - [ ] 1.5 Add `ff-logging` as a dependency in `Cargo.toml`
  - Covers: Structural foundation for all requirements

- [ ] 2. Error types and version types
  - [ ] 2.1 Define `PluginError` enum with all variants: InitializationFailed, ActivationFailed, DeactivationFailed, ShutdownFailed, DependencyNotSatisfied, IncompatibleApiVersion, PluginNotFound, InvalidStateTransition, CircularDependency, ConfigAccessDenied, VfsError, Panicked, CapabilityConflict, NetworkAccessDenied
  - [ ] 2.2 Implement `Display` via `thiserror` with `[plugin:{name}] operation: description` format
  - [ ] 2.3 Define `Version` struct (major, minor, patch) with `PartialOrd`, `Ord`, `Display`, `FromStr`
  - [ ] 2.4 Define `VersionReq` struct with `minimum` and `same_major` fields and `matches(&self, version: &Version) -> bool` method
  - [ ] 2.5 Define `PLUGIN_API_VERSION` constant as `Version { major: 1, minor: 0, patch: 0 }`
  - [ ] 2.6 Write unit tests for error formatting, version comparison, and version requirement matching
  - Covers: Requirement 1 (AC 4, 5), Requirement 6 (AC 1)

- [ ] 3. Plugin metadata and dependency declarations
  - [ ] 3.1 Define `PluginMetadata` struct with name, version, author, description, dependencies, required_api_version fields
  - [ ] 3.2 Define `PluginDependency` struct with name and version_req fields
  - [ ] 3.3 Implement manifest parsing from TOML (`plugin.toml` format) into `PluginMetadata`
  - [ ] 3.4 Write unit tests for metadata construction and TOML manifest parsing
  - Covers: Requirement 1 (AC 2), Requirement 6 (AC 2)

- [ ] 4. Capability types and descriptors
  - [ ] 4.1 Define `Capability` enum with variants: Commands, Viewers, Providers, LanguageSupport, ThemeContribution
  - [ ] 4.2 Define capability metadata structs: `CommandsCapability`, `ViewersCapability`, `ProvidersCapability`, `LanguageSupportCapability`, `ThemeCapability` — each with a `version` field
  - [ ] 4.3 Define `CapabilityType` enum for type-based queries
  - [ ] 4.4 Define `CapabilityDescriptor` struct with capability, owner_plugin, and registration_order fields
  - [ ] 4.5 Define `CapabilityFilter` struct with optional type, mime_type, category, language_id, and owner fields
  - [ ] 4.6 Implement `Capability::cap_type() -> CapabilityType` helper method
  - [ ] 4.7 Write unit tests for capability construction and type classification
  - Covers: Requirement 4 (AC 1), Requirement 6 (AC 6)

- [ ] 5. FileForgePlugin trait definition
  - [ ] 5.1 Define `FileForgePlugin` trait with lifecycle methods: `initialize`, `activate`, `deactivate`, `shutdown`
  - [ ] 5.2 Add `metadata(&self) -> &PluginMetadata` and `capabilities(&self) -> &[Capability]` accessors
  - [ ] 5.3 Add default method `supports_hot_reload(&self) -> bool` returning false
  - [ ] 5.4 Ensure trait is object-safe (`Send + Sync` supertrait bounds, no generics)
  - [ ] 5.5 Verify `Box<dyn FileForgePlugin>` compiles and can be stored in collections
  - [ ] 5.6 Write unit tests demonstrating object-safety with a mock plugin implementation
  - Covers: Requirement 1 (AC 1, 2, 3, 4, 6), Requirement 3 (AC 6)

- [ ] 6. Service traits for PluginContext
  - [ ] 6.1 Define `PluginLogHandle` trait (re-export from ff-logging or define locally with delegation)
  - [ ] 6.2 Define `CommandRegistration` trait with `register` and `unregister` methods
  - [ ] 6.3 Define `PluginConfigAccess` trait with scoped `get` and `set` methods
  - [ ] 6.4 Define `PluginVfsAccess` trait with `read`, `write`, `exists`, `list_directory` methods
  - [ ] 6.5 Define `PluginEventBus` trait with `subscribe`, `unsubscribe`, `emit` methods
  - [ ] 6.6 Define `CapabilityRegistrar` trait with `register` and `unregister` methods
  - [ ] 6.7 Ensure all service traits have `Send + Sync` bounds
  - [ ] 6.8 Write unit tests with mock implementations verifying trait definitions compile
  - Covers: Requirement 2 (AC 2, 3), Requirement 7 (AC 2)

- [ ] 7. PluginContext implementation
  - [ ] 7.1 Define `PluginContext` struct holding plugin_name, service trait object references (`Arc<dyn Trait>`), and api_version
  - [ ] 7.2 Implement `PluginContext::new()` constructor accepting `PlatformServices` and plugin name
  - [ ] 7.3 Implement logging delegation: `log()` returns handle that prefixes records with `[plugin:{name}]`
  - [ ] 7.4 Implement command registration delegation: `register_command()` delegates to `CommandRegistration` trait
  - [ ] 7.5 Implement scoped configuration access: `config_get()` and `config_set()` enforce namespace scoping to `[plugins.{name}]`
  - [ ] 7.6 Implement VFS access delegation: `vfs()` returns reference to `PluginVfsAccess`
  - [ ] 7.7 Implement event methods: `subscribe_event()`, `unsubscribe_event()`, `emit_event()`
  - [ ] 7.8 Implement capability registration: `register_capability()` delegates to `CapabilityRegistrar`
  - [ ] 7.9 Implement `api_version()` returning the host `PLUGIN_API_VERSION`
  - [ ] 7.10 Verify `PluginContext` is `Send + Sync` (compile-time assertion)
  - [ ] 7.11 Write unit tests for namespace scoping, service delegation, and thread-safety
  - Covers: Requirement 2 (AC 1, 2, 3, 4, 5, 6, 7), Requirement 6 (AC 7), Requirement 7 (AC 4, 5)

- [ ] 8. Plugin lifecycle state machine
  - [ ] 8.1 Define `PluginState` enum: Discovered, Loaded, Initialized, Active, Deactivating, Shutdown
  - [ ] 8.2 Implement state transition validation function that enforces valid forward-only transitions
  - [ ] 8.3 Implement hot-reload transition exception: Active → Deactivating → Shutdown → Discovered cycle
  - [ ] 8.4 Return `PluginError::InvalidStateTransition` on invalid transitions
  - [ ] 8.5 Write unit tests for all valid transitions and rejection of invalid ones
  - Covers: Requirement 5 (AC 1)

- [ ] 9. Plugin_Registry core structure
  - [ ] 9.1 Define `PluginEntry` struct holding instance, state, metadata, registered_capabilities, and context
  - [ ] 9.2 Define `PluginRegistry` struct with `RwLock<HashMap<String, PluginEntry>>`, plugin_directory path, and platform services
  - [ ] 9.3 Implement `PluginRegistry::new()` constructor
  - [ ] 9.4 Implement `plugin_state(&self, name: &str) -> Option<PluginState>` query method
  - [ ] 9.5 Implement `plugin_metadata(&self, name: &str) -> Option<&PluginMetadata>` query method
  - [ ] 9.6 Implement `list_plugins(&self) -> Vec<(&str, PluginState)>` listing method
  - [ ] 9.7 Write unit tests for registry construction and state queries
  - Covers: Requirement 5 (AC 7), Requirement 3 (AC 1)

- [ ] 10. Plugin discovery and manifest loading
  - [ ] 10.1 Implement `discover_plugins()` — scan plugin_directory for subdirectories containing `plugin.toml`
  - [ ] 10.2 Parse each `plugin.toml` into `PluginMetadata` and create `PluginEntry` in Discovered state
  - [ ] 10.3 Log INFO record for each discovered plugin with name and version
  - [ ] 10.4 Handle missing or malformed manifests gracefully — log WARN and skip
  - [ ] 10.5 Write unit tests using tempdir with mock plugin directories
  - Covers: Requirement 3 (AC 1)

- [ ] 11. Dependency graph and topological sort
  - [ ] 11.1 Implement `DependencyGraph` struct with adjacency list representation
  - [ ] 11.2 Implement `build_graph()` from a set of `PluginMetadata` entries and their declared dependencies
  - [ ] 11.3 Implement cycle detection using Kahn's algorithm or DFS-based approach
  - [ ] 11.4 Implement topological sort returning load order (Vec of plugin names)
  - [ ] 11.5 When cycle detected, return `PluginError::CircularDependency` listing all plugins in the cycle
  - [ ] 11.6 Log ERROR-level record identifying circular dependencies
  - [ ] 11.7 When dependency not found or version-incompatible, skip dependent plugin and log ERROR
  - [ ] 11.8 Write unit tests for DAG construction, topological ordering, cycle detection, and missing dependency handling
  - Covers: Requirement 3 (AC 3, 4, 7)

- [ ] 12. Version compatibility checking
  - [ ] 12.1 Implement `check_api_compatibility(required: &Version, available: &Version) -> Result<(), PluginError>`
  - [ ] 12.2 Reject plugin if `required.major != available.major` — log ERROR with version incompatibility
  - [ ] 12.3 Reject plugin if `required.minor > available.minor` (same major) — log WARN indicating newer API needed
  - [ ] 12.4 Accept plugin if `required.major == available.major && required.minor <= available.minor`
  - [ ] 12.5 Write unit tests for all version comparison scenarios
  - Covers: Requirement 6 (AC 3, 4, 5)

- [ ] 13. Plugin loading sequence
  - [ ] 13.1 Implement `load_all()` — discover → build dependency graph → topological sort → load each in order
  - [ ] 13.2 For each plugin in load order: validate API version → initialize → activate
  - [ ] 13.3 Transition states correctly: Discovered → Loaded → Initialized → Active
  - [ ] 13.4 If any step fails, transition plugin to Shutdown, log error, continue with remaining plugins
  - [ ] 13.5 Return `Vec<PluginLoadResult>` summarizing success/failure for each plugin
  - [ ] 13.6 Implement `load_plugin(name)` for loading a single plugin (dependencies must already be active)
  - [ ] 13.7 Write unit tests for successful load sequences and partial failure scenarios
  - Covers: Requirement 3 (AC 2, 3, 7)

- [ ] 14. Capability_Registry implementation
  - [ ] 14.1 Define `CapabilityRegistry` struct with `RwLock<Vec<CapabilityDescriptor>>` and registration counter
  - [ ] 14.2 Implement `register()` — add capability with owner identity and increment registration_order
  - [ ] 14.3 Implement duplicate detection: if same type+identifier already exists, emit WARN log and use first-registered as default
  - [ ] 14.4 Implement `unregister_all(owner)` — remove all capabilities for a given plugin
  - [ ] 14.5 Implement `query_by_type(cap_type)` — return all descriptors matching the capability type
  - [ ] 14.6 Implement `query_by_attribute(filter)` — filter by mime_type, category, language_id, or owner
  - [ ] 14.7 Implement `has_capability(cap_type, id)` — check existence
  - [ ] 14.8 Emit `CapabilityChanged` platform event on register and unregister
  - [ ] 14.9 Ensure capabilities become queryable immediately on registration and removed immediately on unregistration
  - [ ] 14.10 Write unit tests for registration, querying, duplicate handling, and event emission
  - Covers: Requirement 4 (AC 1, 2, 3, 5, 6), Requirement 3 (AC 5)

- [ ] 15. Lifecycle management — panic catching and error handling
  - [ ] 15.1 Wrap all lifecycle method calls in `std::panic::catch_unwind`
  - [ ] 15.2 On panic: transition plugin to Shutdown, log ERROR with plugin name and panic message
  - [ ] 15.3 On error return: log WARN with plugin name, lifecycle phase, and error description; transition to Shutdown
  - [ ] 15.4 Ensure panics do NOT propagate to the host application under any circumstances
  - [ ] 15.5 Write unit tests with deliberately panicking mock plugins verifying isolation
  - Covers: Requirement 5 (AC 3, 4)

- [ ] 16. Lifecycle management — deactivation and resource cleanup
  - [ ] 16.1 Implement `unload_plugin(name)` — deactivate dependents first (reverse order), then deactivate target
  - [ ] 16.2 On deactivation: remove plugin's capabilities from Capability_Registry
  - [ ] 16.3 On deactivation: cancel plugin's event subscriptions
  - [ ] 16.4 After shutdown: release all references the platform holds to the plugin (set instance to None)
  - [ ] 16.5 Guarantee: after Shutdown state, no capabilities, subscriptions, or references remain
  - [ ] 16.6 Write unit tests verifying complete resource cleanup after deactivation
  - Covers: Requirement 5 (AC 2, 6)

- [ ] 17. Lifecycle management — application shutdown
  - [ ] 17.1 Implement `shutdown_all(timeout: Duration)` — compute reverse dependency order
  - [ ] 17.2 Call `deactivate()` then `shutdown()` on each active plugin in reverse dependency order
  - [ ] 17.3 Wrap each call in `catch_unwind` for panic isolation
  - [ ] 17.4 Track elapsed time — if total exceeds timeout (default 5 seconds), forcibly drop remaining instances
  - [ ] 17.5 Log summary of shutdown results (successful, timed-out, panicked)
  - [ ] 17.6 Write unit tests for orderly shutdown, timeout enforcement, and panic during shutdown
  - Covers: Requirement 5 (AC 5)

- [ ] 18. Hot-reload support
  - [ ] 18.1 Implement `hot_reload(name)` — check `supports_hot_reload()` on the plugin
  - [ ] 18.2 Cycle: Active → Deactivating → Shutdown → Discovered → Loaded → Initialized → Active
  - [ ] 18.3 Remove old capabilities, load new instance from disk, re-initialize, re-activate
  - [ ] 18.4 If plugin does not support hot-reload, return error
  - [ ] 18.5 Write unit tests for hot-reload happy path and unsupported rejection
  - Covers: Requirement 3 (AC 6), Requirement 5 (AC 1)

- [ ] 19. Security and sandboxing enforcement
  - [ ] 19.1 Implement configuration namespace enforcement in `PluginContext` — reject access outside `[plugins.{name}]`
  - [ ] 19.2 Return `PluginError::ConfigAccessDenied` on namespace violation and log WARN
  - [ ] 19.3 Implement network access control — check manifest for `NetworkAccess` capability declaration
  - [ ] 19.4 Return `PluginError::NetworkAccessDenied` if plugin lacks NetworkAccess capability
  - [ ] 19.5 Ensure VFS-only file access — `PluginContext` exposes only VFS methods, no `std::fs` primitives
  - [ ] 19.6 Ensure capability registrations are stamped with the calling plugin's identity (cannot be forged)
  - [ ] 19.7 Log WARN on any sandboxing violation with plugin name and violation description
  - [ ] 19.8 Write unit tests for namespace violations, network access denial, and capability ownership stamps
  - Covers: Requirement 7 (AC 1, 2, 3, 4, 5, 6, 7)

- [ ] 20. Type-safe capability querying
  - [ ] 20.1 Implement generic query method `query::<T: CapabilityProvider>() -> Vec<&dyn T>` on Capability_Registry
  - [ ] 20.2 Define `CapabilityProvider` trait for compile-time type verification of capability usage
  - [ ] 20.3 Write unit tests demonstrating type-safe queries with mock capability providers
  - Covers: Requirement 4 (AC 4)

- [ ] 21. Platform event integration
  - [ ] 21.1 Define `PlatformEvent` struct for plugin lifecycle events (plugin loaded, unloaded, capability changed)
  - [ ] 21.2 Define `SubscriptionId` type and `EventHandler` type alias
  - [ ] 21.3 Emit events when capabilities are added or removed (including plugin identity and capability type)
  - [ ] 21.4 Write unit tests verifying event emission on capability change
  - Covers: Requirement 4 (AC 6)

- [ ] 22. Integration tests
  - [ ] 22.1 Write end-to-end test: discover → load → activate → query capabilities → deactivate → shutdown
  - [ ] 22.2 Write integration test with multiple plugins demonstrating dependency ordering
  - [ ] 22.3 Write integration test demonstrating plugin failure isolation (one plugin panics, others continue)
  - [ ] 22.4 Write integration test for configuration scoping across multiple plugins
  - [ ] 22.5 Write integration test for capability registration and query lifecycle
  - Covers: All requirements (integration verification)

- [ ] 23. Property-based tests
  - [ ] 23.1 Write PBT: lifecycle state machine validity property
  - [ ] 23.2 Write PBT: dependency graph acyclicity after validation property
  - [ ] 23.3 Write PBT: topological load order correctness property
  - [ ] 23.4 Write PBT: version compatibility decision correctness property
  - [ ] 23.5 Write PBT: capability registry consistency property
  - [ ] 23.6 Write PBT: configuration scoping enforcement property
  - [ ] 23.7 Write PBT: panic isolation property
  - [ ] 23.8 Write PBT: capability ownership identity property
  - [ ] 23.9 Write PBT: shutdown reverse dependency order property
  - [ ] 23.10 Write PBT: duplicate capability resolution property
  - Covers: All correctness properties from design.md §10

---

## Property-Based Test Definitions

### Property 1: Lifecycle State Machine Validity

**Validates: Requirement 5.1**

- **Statement:** For any sequence of lifecycle operations on a plugin, the plugin's state transitions always follow the valid state machine. No operation can produce an invalid transition (e.g., Active → Discovered).
- **Strategy:** Generate arbitrary sequences of lifecycle operations:
  - Operations: uniform selection from {initialize, activate, deactivate, shutdown}
  - Sequence length: [1, 20] operations
  - Starting state: Discovered
- **Invariant:** After each operation, the resulting state is reachable from the previous state via exactly one valid transition edge. Invalid transitions produce `PluginError::InvalidStateTransition` and leave state unchanged.

### Property 2: Dependency Graph Acyclicity After Validation

**Validates: Requirement 3.3, 3.4**

- **Statement:** For any set of plugins with arbitrary dependency declarations, after the registry's validation phase, the resolved dependency graph (excluding rejected plugins) is always a DAG — it contains no cycles.
- **Strategy:** Generate:
  - Number of plugins: [2, 30]
  - Dependency edges: random subset of possible edges, including intentional cycles with probability 0.3
  - Plugin names: generated strings matching `[a-z][a-z0-9-]*` (3–20 chars)
- **Invariant:** After `resolve()`, the accepted subset forms a valid DAG (topological sort succeeds, no back-edges). All plugins in detected cycles are rejected.

### Property 3: Topological Load Order Correctness

**Validates: Requirement 3.3**

- **Statement:** For any valid (acyclic) dependency graph, the load order produced by the registry ensures that every plugin is loaded only after all of its dependencies have been loaded and activated.
- **Strategy:** Generate:
  - Random DAGs of [1, 50] plugins with dependency edges
  - Ensure generated graphs are acyclic by only allowing edges from higher-index to lower-index nodes
- **Invariant:** For every plugin P in the load order, all dependencies of P appear at earlier indices in the sequence.

### Property 4: Version Compatibility Decision Correctness

**Validates: Requirement 6.3, 6.4, 6.5**

- **Statement:** For any pair of (plugin `required_api_version`, host `PLUGIN_API_VERSION`), the compatibility decision follows semantic versioning rules: different major → reject; same major, plugin minor > host minor → reject; same major, plugin minor <= host minor → accept.
- **Strategy:** Generate:
  - Required version: major in [0, 5], minor in [0, 20], patch in [0, 50]
  - Available version: major in [0, 5], minor in [0, 20], patch in [0, 50]
- **Invariant:** `is_compatible(required, available)` ⟺ `required.major == available.major ∧ required.minor <= available.minor`

### Property 5: Capability Registry Consistency

**Validates: Requirement 4.2, 4.3**

- **Statement:** After any sequence of register/unregister operations, querying by type returns exactly the set of capabilities that have been registered and not yet unregistered — no phantom entries, no missing entries.
- **Strategy:** Generate:
  - Sequences of [10, 100] operations: mix of `register(owner, capability)` and `unregister_all(owner)`
  - Plugin names: [3, 10] unique names
  - Capability types: uniform from {Commands, Viewers, Providers, LanguageSupport, ThemeContribution}
- **Invariant:** `query_by_type(T)` returns exactly `{ c | c was registered with type T and c.owner has not been subsequently unregistered }`

### Property 6: Configuration Scoping Enforcement

**Validates: Requirement 2.7, Requirement 7.5**

- **Statement:** For any plugin with name N, configuration access through its `PluginContext` can only read/write keys within the `[plugins.N]` namespace. Access to keys outside this namespace always returns `ConfigAccessDenied`.
- **Strategy:** Generate:
  - Plugin names: strings matching `[a-z][a-z0-9-]*` (3–20 chars)
  - Key attempts: mix of valid keys (within namespace) and invalid keys (other namespaces, absolute paths, traversal attempts)
- **Invariant:** `config_get(valid_key)` succeeds; `config_get(invalid_key)` returns `Err(ConfigAccessDenied)` for all keys not in `[plugins.{name}]`

### Property 7: Panic Isolation

**Validates: Requirement 5.3**

- **Statement:** For any plugin lifecycle method that panics, the panic is caught and does NOT propagate to the host. The plugin transitions to Shutdown state, and the registry remains operational for all other plugins.
- **Strategy:** Generate:
  - Number of plugins: [2, 15]
  - For each plugin, randomly designate which lifecycle methods panic (0–4 methods)
  - Generate interleaved lifecycle operations across all plugins
- **Invariant:** After all operations, non-panicking plugins reach their expected state, panicking plugins reach Shutdown state, and no host-level panic occurs (test completes successfully).

### Property 8: Capability Ownership Identity

**Validates: Requirement 7.6**

- **Statement:** For any capability registered by plugin P, the `CapabilityDescriptor.owner_plugin` always equals P's name. No plugin can register a capability that appears to be owned by a different plugin.
- **Strategy:** Generate:
  - Number of plugins: [2, 10]
  - Each plugin registers [1, 5] capabilities
  - Registration goes through `PluginContext.register_capability()` which stamps ownership
- **Invariant:** For all descriptors `d` in the registry, `d.owner_plugin` equals the name of the plugin whose context was used to register `d`.

### Property 9: Shutdown Reverse Dependency Order

**Validates: Requirement 5.5**

- **Statement:** For any dependency graph, the shutdown order is the exact reverse of the load order. A plugin that depends on another plugin is always shut down before its dependency.
- **Strategy:** Generate:
  - Random acyclic dependency graphs of [1, 50] plugins
  - Compute load order via topological sort
- **Invariant:** `shutdown_order == reverse(load_order)`. Equivalently: for every edge A→B (A depends on B), A appears before B in the shutdown sequence.

### Property 10: Duplicate Capability Resolution

**Validates: Requirement 3.5**

- **Statement:** When two or more plugins register the same capability type with the same identifier, the first-registered provider becomes the default (lowest `registration_order`), but all alternatives remain queryable.
- **Strategy:** Generate:
  - Number of plugins: [2, 8] all registering overlapping capabilities
  - Capabilities with shared identifiers across multiple plugins
  - Random registration order
- **Invariant:** `query_by_type(T)` returns all registered instances; the result list is ordered by `registration_order` (ascending); the first entry is the first-registered provider.

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types", "tasks": ["2", "3", "4", "8"], "dependsOn": [0] },
    { "id": 2, "label": "Trait Definitions", "tasks": ["5", "6", "21"], "dependsOn": [1] },
    { "id": 3, "label": "Context and Registry Core", "tasks": ["7", "9"], "dependsOn": [2] },
    { "id": 4, "label": "Discovery and Dependency Resolution", "tasks": ["10", "11", "12"], "dependsOn": [3] },
    { "id": 5, "label": "Loading and Capability Registry", "tasks": ["13", "14", "20"], "dependsOn": [4] },
    { "id": 6, "label": "Lifecycle Management", "tasks": ["15", "16", "17", "18"], "dependsOn": [5] },
    { "id": 7, "label": "Security and Sandboxing", "tasks": ["19"], "dependsOn": [3] },
    { "id": 8, "label": "Integration and PBT", "tasks": ["22", "23"], "dependsOn": [6, 7] }
  ]
}
```

---

## Notes

- This is a Wave 2 (Platform Architecture) crate depending only on `ff-logging` (Wave 0)
- All service traits (CommandRegistration, PluginConfigAccess, PluginVfsAccess, PluginEventBus) are defined in `ff-plugin` but IMPLEMENTED by downstream crates (command-framework, configuration-system, virtual-file-system)
- `ff-plugin` does NOT depend on platform-core; the dependency is inverted — platform-core depends on ff-plugin
- The `PluginLogHandle` trait is defined in or re-exported from `ff-logging` to avoid circular dependencies
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Thread-safety is enforced via `Send + Sync` bounds on all public types and service traits
- Hot-reload (Task 18) is an optional capability — plugins opt in via `supports_hot_reload()` method
- The type-safe capability query (Task 20) uses Rust generics with trait bounds for compile-time verification
- Plugin manifest files use TOML format (`plugin.toml`) as specified in design.md §8

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Plugin Trait Definition | AC 1.1 (lifecycle methods) | Task 5 |
| | AC 1.2 (metadata accessor) | Tasks 3, 5 |
| | AC 1.3 (capabilities accessor) | Tasks 4, 5 |
| | AC 1.4 (Result return type) | Tasks 2, 5 |
| | AC 1.5 (PluginError variants) | Task 2 |
| | AC 1.6 (object-safe) | Task 5 |
| Req 2: Plugin Context | AC 2.1 (PluginContext struct) | Task 7 |
| | AC 2.2 (service access) | Tasks 6, 7 |
| | AC 2.3 (delegation without exposing internals) | Tasks 6, 7 |
| | AC 2.4 (sole interface) | Tasks 7, 19 |
| | AC 2.5 (Send + Sync) | Task 7 |
| | AC 2.6 (capability registration) | Tasks 7, 14 |
| | AC 2.7 (scoped config) | Tasks 7, 19 |
| Req 3: Registration and Loading | AC 3.1 (discover at startup) | Tasks 9, 10 |
| | AC 3.2 (loading sequence) | Task 13 |
| | AC 3.3 (dependency order) | Tasks 11, 13 |
| | AC 3.4 (circular dependency rejection) | Task 11 |
| | AC 3.5 (duplicate capability handling) | Task 14 |
| | AC 3.6 (hot-reload) | Task 18 |
| | AC 3.7 (unsatisfied dependency skip) | Tasks 11, 13 |
| Req 4: Capability Discovery | AC 4.1 (typed Capability enum) | Task 4 |
| | AC 4.2 (runtime query interface) | Task 14 |
| | AC 4.3 (dynamic register/unregister) | Task 14 |
| | AC 4.4 (type-safe querying) | Task 20 |
| | AC 4.5 (attribute-based querying) | Task 14 |
| | AC 4.6 (capability change events) | Tasks 14, 21 |
| Req 5: Lifecycle Management | AC 5.1 (state transitions) | Task 8 |
| | AC 5.2 (deactivation cleanup) | Task 16 |
| | AC 5.3 (panic catching) | Task 15 |
| | AC 5.4 (error handling) | Task 15 |
| | AC 5.5 (shutdown in reverse order) | Task 17 |
| | AC 5.6 (resource cleanup guarantee) | Task 16 |
| | AC 5.7 (state query method) | Task 9 |
| Req 6: Versioning and Compatibility | AC 6.1 (PLUGIN_API_VERSION constant) | Task 2 |
| | AC 6.2 (required_api_version field) | Task 3 |
| | AC 6.3 (major version rejection) | Task 12 |
| | AC 6.4 (minor version acceptance) | Task 12 |
| | AC 6.5 (newer minor rejection) | Task 12 |
| | AC 6.6 (capability versioning) | Task 4 |
| | AC 6.7 (runtime API version query) | Task 7 |
| Req 7: Security and Sandboxing | AC 7.1 (same process, API boundaries) | Task 19 |
| | AC 7.2 (VFS-only file access) | Tasks 6, 19 |
| | AC 7.3 (network access control) | Task 19 |
| | AC 7.4 (no cross-plugin state access) | Tasks 7, 19 |
| | AC 7.5 (scoped configuration) | Tasks 7, 19 |
| | AC 7.6 (capability ownership stamp) | Task 19 |
| | AC 7.7 (violation error + WARN log) | Task 19 |
