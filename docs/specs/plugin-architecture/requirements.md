# Requirements Document

## Introduction

This feature specifies the plugin architecture for FileForgeWorkbench (`ff-plugin` crate). The plugin architecture defines how optional features are packaged, discovered, loaded, and managed throughout their lifecycle. It is a **foundational platform crate** — all optional features (viewers, language services, connectors, macro engines, the database tool) are implemented as plugins that interact with the core exclusively through traits and a context object defined here.

The plugin architecture implements a trait-based extensibility model where the core remains minimal and stable while plugins independently provide capabilities. Plugins are discovered at startup, validated for compatibility, initialized with a sandboxed context, and managed through well-defined lifecycle states. The architecture supports capability advertisement, dependency resolution, semantic versioning, and failure isolation — a misbehaving plugin cannot crash the host application.

**Source references:**
- **WB** = Workbench Architecture Brief §10 (plugin model, trait-based extensibility)
- **FFW** = FileForgeWorkbench cross-cutting Requirement 3 (Plugin Architecture Principle)

## Glossary

- **FileForgePlugin**: The primary trait that all plugins must implement, defining lifecycle methods (initialize, activate, deactivate, shutdown) and metadata accessors. [WB]
- **PluginContext**: An opaque context object provided to plugins during initialization, serving as the ONLY interface between a plugin and the core platform services. [WB]
- **Plugin_Registry**: The core-owned registry that tracks all discovered, loaded, and active plugins, their states, capabilities, and metadata. [WB]
- **Plugin_Directory**: The filesystem directory scanned at startup to discover available plugins. [WB]
- **Plugin_Manifest**: Metadata associated with a plugin including name, version, author, description, dependencies, required API version, and declared capabilities. [WB]
- **Capability**: A typed service or feature that a plugin provides to the platform (e.g., Commands, Viewers, Providers, LanguageSupport, ThemeContribution). [WB]
- **Capability_Registry**: A dynamic registry of all capabilities currently available, updated as plugins load and unload. [WB]
- **Plugin_State**: The current lifecycle state of a plugin: Discovered, Loaded, Initialized, Active, Deactivating, or Shutdown. [WB]
- **Plugin_API_Version**: The semantic version of the core plugin API that a plugin targets, used for compatibility checking at load time. [WB]
- **Dependency_Graph**: A directed acyclic graph representing the dependency relationships between plugins, used to determine load order. [WB]

## Requirements

### Requirement 1: Plugin Trait Definition

**User Story:** As a plugin developer, I want a well-defined trait with clear lifecycle methods and metadata accessors, so that I can implement a plugin without needing to understand the core's internal architecture.

**Source:** WB Architecture Brief §10, FFW cross-cutting Req 3 AC 2. [WB]

#### Acceptance Criteria

1. THE `ff-plugin` crate SHALL define a `FileForgePlugin` trait with the following lifecycle methods: `initialize(&mut self, context: Arc<PluginContext>) -> Result<(), PluginError>`, `activate(&mut self) -> Result<(), PluginError>`, `deactivate(&mut self) -> Result<(), PluginError>`, and `shutdown(&mut self) -> Result<(), PluginError>`.
2. THE `FileForgePlugin` trait SHALL define a `metadata(&self) -> &PluginMetadata` method that returns an immutable reference to the plugin's metadata, where `PluginMetadata` contains: name (String), version (semver-compatible Version), author (String), description (String), and dependencies (Vec of dependency declarations).
3. THE `FileForgePlugin` trait SHALL define a `plugin_capabilities(&self) -> &[Capability]` method that returns the list of capabilities the plugin provides to the platform.
4. ALL lifecycle methods on `FileForgePlugin` SHALL return `Result<(), PluginError>`, where `PluginError` is a structured error type defined by the `ff-plugin` crate — a lifecycle method failure SHALL NOT panic or propagate a panic to the host.
5. THE `PluginError` type SHALL include variants for: initialization failure, activation failure, deactivation failure, shutdown failure, dependency not satisfied, and incompatible API version, each carrying a human-readable description string.
6. THE `FileForgePlugin` trait SHALL be object-safe, allowing the core to store plugins as trait objects (`Box<dyn FileForgePlugin>`).

---

### Requirement 2: Plugin Context

**User Story:** As a plugin developer, I want a context object that provides access to platform services (logging, commands, configuration, VFS, events), so that I can register capabilities and interact with the workbench without importing internal crate APIs.

**Source:** WB Architecture Brief §10, FFW cross-cutting Req 3 AC 3. [WB]

#### Acceptance Criteria

1. THE `ff-plugin` crate SHALL define a `PluginContext` struct that is provided to each plugin's `initialize` method as the sole interface to platform services.
2. THE `PluginContext` SHALL provide access to the following services: logging (emit log records), command registration (register new commands), configuration (read plugin-scoped settings), VFS access (read/write files through the virtual file system), and event subscription (subscribe to and emit platform events).
3. WHEN a plugin calls methods on `PluginContext`, THE context SHALL delegate to the appropriate platform-core service without exposing internal implementation types — the plugin depends only on the `ff-plugin` crate's public API.
4. THE `PluginContext` SHALL be the ONLY interface between a plugin and the core platform — plugins SHALL NOT have access to any internal platform-core APIs, service implementations, or other plugins' internal state.
5. THE `PluginContext` SHALL be thread-safe (`Send + Sync`), allowing plugins to use the context from threads they spawn without additional synchronization by the plugin developer.
6. THE `PluginContext` SHALL provide a method to register capabilities (commands, viewers, providers, language support contributions, theme contributions) with the Capability_Registry.
7. THE `PluginContext` SHALL scope configuration access to the plugin's own namespace — a plugin with name "my-plugin" SHALL only read and write keys under the `[plugins.my-plugin]` configuration section.

---

### Requirement 3: Plugin Registration and Loading

**User Story:** As a platform developer, I want the system to automatically discover, validate, and load plugins at startup in the correct dependency order, so that plugins are available without manual intervention and dependency conflicts are resolved gracefully.

**Source:** WB Architecture Brief §10. [WB]

#### Acceptance Criteria

1. WHEN the application starts, THE Plugin_Registry SHALL scan the Plugin_Directory for available plugins and transition each discovered plugin to the Discovered state.
2. THE plugin loading sequence SHALL follow this order for each plugin: discover → load → validate → initialize → activate, where each step must succeed before the next begins.
3. WHEN plugins declare dependencies on other plugins, THE Plugin_Registry SHALL construct a Dependency_Graph and load plugins in topological order — a plugin's dependencies SHALL be initialized and active before the dependent plugin's `initialize` method is called.
4. IF a circular dependency is detected in the Dependency_Graph, THEN THE Plugin_Registry SHALL reject all plugins in the cycle, log an ERROR-level record identifying the cycle, and continue loading unaffected plugins.
5. IF two or more plugins declare they provide the same capability type with the same identifier, THEN THE Plugin_Registry SHALL emit a WARN-level log record identifying the duplicate, load both plugins, and use the first-registered provider as the default while making alternatives queryable.
6. THE Plugin_Registry SHALL support optional hot-reload: when a plugin's binary or manifest changes on disk while the application is running, the platform MAY deactivate the old instance, unload it, load the new version, and re-activate it — this is an optional capability that plugins can opt into by implementing a `supports_hot_reload(&self) -> bool` method on the trait.
7. IF a plugin's declared dependencies cannot be satisfied (dependency not found or incompatible version), THEN THE Plugin_Registry SHALL skip that plugin, log an ERROR-level record identifying the unmet dependency, and continue loading other plugins.

---

### Requirement 4: Capability Discovery

**User Story:** As a platform developer, I want to query at runtime what capabilities are available (viewers, commands, language services, themes), so that the UI and other subsystems can dynamically adapt to the set of loaded plugins.

**Source:** WB Architecture Brief §10, FFW cross-cutting Req 3 AC 4. [WB]

#### Acceptance Criteria

1. PLUGINS SHALL declare their capabilities using a typed `Capability` enum with the following variants: `Commands`, `Viewers`, `Providers`, `LanguageSupport`, and `ThemeContribution`, each carrying metadata specific to that capability type.
2. THE Capability_Registry SHALL provide a runtime query interface that allows any subsystem to ask "what capabilities of type X are currently registered?" and receive a list of matching capability descriptors with their owning plugin identifiers.
3. THE Capability_Registry SHALL be dynamic — when a plugin is activated, its capabilities SHALL become queryable immediately; when a plugin is deactivated or unloaded, its capabilities SHALL be removed from the registry immediately.
4. THE Capability_Registry SHALL support type-safe querying where possible, using generic methods with trait bounds (e.g., `query::<T: CapabilityProvider>() -> Vec<&dyn T>`) to enable compile-time verification of capability usage.
5. THE Capability_Registry SHALL support querying by capability metadata attributes (e.g., "all viewers that handle MIME type text/plain", "all commands in category 'file'") in addition to querying by capability type.
6. WHEN the set of available capabilities changes (plugin loaded or unloaded), THE Capability_Registry SHALL emit a platform event notifying subscribers of the change, including the capability type and the plugin identifier involved.

---

### Requirement 5: Plugin Lifecycle Management

**User Story:** As a platform developer, I want plugins to transition through well-defined lifecycle states with graceful cleanup guarantees, so that plugin failures are isolated and resources are properly released.

**Source:** WB Architecture Brief §10, FFW cross-cutting Req 3 AC 5. [WB]

#### Acceptance Criteria

1. THE Plugin_Registry SHALL track each plugin through the following states in order: Discovered → Loaded → Initialized → Active → Deactivating → Shutdown, where transitions only move forward except for hot-reload scenarios which cycle Active → Deactivating → Shutdown → Discovered → Loaded → Initialized → Active.
2. WHEN a plugin's `deactivate` method is called, THE plugin SHALL release all resources it holds (unregister capabilities, cancel background tasks, close file handles) and THE Plugin_Registry SHALL remove the plugin's capabilities from the Capability_Registry before transitioning to the Shutdown state.
3. IF a plugin's lifecycle method panics, THEN THE Plugin_Registry SHALL catch the panic (using `std::panic::catch_unwind`), transition the plugin to the Shutdown state, log an ERROR-level record containing the plugin name and panic message, and continue operating — the panic SHALL NOT propagate to the host application.
4. IF a plugin's lifecycle method returns an error, THEN THE Plugin_Registry SHALL log a WARN-level record containing the plugin name, the lifecycle phase, and the error description, transition the plugin to the Shutdown state, and continue operating with reduced functionality.
5. WHEN the application is shutting down, THE Plugin_Registry SHALL call `deactivate` followed by `shutdown` on all active plugins in reverse dependency order, waiting up to 5 seconds total for all plugins to complete their shutdown before forcibly dropping remaining plugin instances.
6. THE Plugin_Registry SHALL guarantee resource cleanup: after a plugin transitions to the Shutdown state, all capabilities it registered SHALL be removed, all event subscriptions it held SHALL be cancelled, and all references the platform holds to the plugin SHALL be released.
7. THE Plugin_Registry SHALL provide a method to query the current state of any plugin by name, returning the `Plugin_State` enum value.

---

### Requirement 6: Versioning and Compatibility

**User Story:** As a platform developer, I want semantic versioning for the plugin API with compatibility checks at load time, so that plugins built against an older API version continue to work after minor core updates and incompatible plugins are rejected before they can cause failures.

**Source:** WB Architecture Brief §10. [WB]

#### Acceptance Criteria

1. THE `ff-plugin` crate SHALL define a `PLUGIN_API_VERSION` constant using semantic versioning (major.minor.patch) that represents the current version of the plugin API contract.
2. EACH plugin's `PluginMetadata` SHALL include a `required_api_version` field specifying the minimum Plugin_API_Version the plugin requires to function.
3. WHEN loading a plugin, THE Plugin_Registry SHALL compare the plugin's `required_api_version` against the current `PLUGIN_API_VERSION` — IF the plugin requires a major version different from the current major version, THEN THE Plugin_Registry SHALL reject the plugin and log an ERROR-level record stating the version incompatibility.
4. WHEN the current `PLUGIN_API_VERSION` has the same major version as a plugin's `required_api_version` but a higher or equal minor version, THE Plugin_Registry SHALL accept the plugin (forward compatibility: minor version bumps do not break existing plugins).
5. IF a plugin's `required_api_version` has a minor version higher than the current `PLUGIN_API_VERSION`'s minor version (same major), THEN THE Plugin_Registry SHALL reject the plugin and log a WARN-level record indicating the plugin requires a newer API version than currently available.
6. EACH `Capability` declared by a plugin SHALL include its own version (separate from the plugin version), allowing capability consumers to negotiate based on capability version rather than plugin version.
7. THE Plugin_API_Version SHALL be queryable at runtime by plugins through the `PluginContext`, allowing a plugin to adapt its behaviour based on the host API version if needed.

---

### Requirement 7: Plugin Security and Sandboxing

**User Story:** As a platform developer, I want plugins to operate within controlled boundaries — file access through VFS, scoped configuration, no cross-plugin state access — so that a misbehaving plugin cannot compromise data integrity or access resources it should not.

**Source:** WB Architecture Brief §10. [WB]

#### Acceptance Criteria

1. PLUGINS SHALL execute within the host process (same address space) — the plugin architecture does NOT provide process-level isolation, but enforces API-level boundaries through the `PluginContext` interface.
2. WHEN a plugin needs file system access, THE plugin SHALL use the VFS methods provided by `PluginContext` — plugins SHALL NOT use `std::fs` or direct file system APIs, and the platform's public API SHALL NOT expose direct file system primitives to plugins.
3. THE `PluginContext` SHALL support capability-based access control for network operations — a plugin that does not declare the `NetworkAccess` capability in its manifest SHALL NOT be granted network-related methods on its context.
4. PLUGINS SHALL NOT be able to access other plugins' internal state, configuration namespace, or registered service implementations — the `PluginContext` provides no mechanism to reference or inspect another plugin's internals.
5. THE `PluginContext` SHALL scope all configuration access to the plugin's own namespace (as defined in Requirement 2 AC 7) — attempts to read or write keys outside the plugin's namespace SHALL return an error.
6. WHEN a plugin registers a capability, THE Plugin_Registry SHALL associate it with the plugin's identity — capability registrations cannot be forged to appear as if they come from a different plugin.
7. IF a plugin violates a sandboxing constraint (e.g., attempts unauthorized configuration access), THEN THE platform SHALL return an error to the plugin without terminating it, and SHALL log a WARN-level record describing the violation and the plugin responsible.

