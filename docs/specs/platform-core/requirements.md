# Requirements Document

## Introduction

This feature specifies the GUI-independent workbench core for FileForgeWorkbench — the `ff-core` crate. The platform-core is the **central orchestration layer** of the entire workbench platform. It owns all application state, manages the lifecycle of every subsystem, and defines the strict boundary between business logic and the replaceable GUI rendering shell.

The platform-core operates with **zero GUI framework dependencies** (no egui, winit, wgpu, or any rendering library). All business logic — commands, file operations, document management, undo/redo, workflows, plugins — executes within this GUI-independent layer and communicates with the GUI shell through a defined event/messaging interface. This architecture ensures that the rendering shell can be replaced without rewriting or recompiling any business logic.

The platform-core also defines the **crate structure and layer rules** for the entire workspace: which crates belong to which layer, the dependency direction between layers, and the strict prohibition on reverse dependencies.

**Source references:**
- **WB** = Workbench Platform Architecture Brief §3–§6 (primary source)
- **FFE** = FileForgeEditor specifications (startup/shutdown patterns adapted)

## Glossary

- **Platform_Core**: The `ff-core` crate — the GUI-independent central orchestration layer that owns all application state and manages all subsystem lifecycles. [WB]
- **WorkbenchApp**: The primary struct/trait within `ff-core` that owns all platform state and provides the entry point for subsystem initialization, event dispatch, and lifecycle management. [WB]
- **Service_Registry**: The component within Platform_Core that holds references to all registered subsystems (services), providing type-safe access and startup ordering guarantees. [WB]
- **Event_Bus**: The internal event/message system that connects Platform_Core to the GUI shell and between subsystems, enabling bidirectional communication without tight coupling. [WB]
- **GUI_Shell**: The replaceable rendering layer (e.g., `ff-desktop` using egui) that depends on Platform_Core but is never depended upon by it. Responsible only for rendering state and forwarding user input as events. [WB]
- **Foundation_Layer**: The lowest crate layer containing `ff-logging` — no dependencies on other `ff-*` crates. [WB]
- **Core_Layer**: The layer containing `ff-core`, `ff-config`, `ff-command`, `ff-plugin`, `ff-workflow`, `ff-vfs` — depends only on Foundation_Layer. [WB]
- **Editor_Layer**: The layer containing `ff-document`, `ff-edit`, `ff-undo`, `ff-viewport`, `ff-display-lines` — depends on Core_Layer and Foundation_Layer. [WB]
- **Feature_Layer**: The layer containing `ff-find`, `ff-line-commands`, `ff-exclude`, `ff-nav`, and other feature crates — depends on Editor_Layer and below. [WB]
- **Shell_Layer**: The topmost layer containing `ff-desktop` (egui GUI shell) — depends on all lower layers; nothing depends on it. [WB]
- **Subsystem**: Any registered component within Platform_Core that provides a distinct service (e.g., logging, configuration, VFS, commands, plugins). [WB]
- **Startup_Sequence**: The ordered sequence in which subsystems are initialized: logging → configuration → VFS → commands → plugins → GUI shell. [WB]
- **Shutdown_Sequence**: The reverse-ordered sequence in which subsystems are terminated with a grace period for cleanup. [WB]
- **Tokio_Runtime**: The async runtime used for background I/O workers (file operations, network, background tasks) within the workbench. [WB]

## Requirements

### Requirement 1: Core Layer Architecture

**User Story:** As a workbench architect, I want the `ff-core` crate to be the central orchestration layer with zero GUI dependencies, so that all business logic can execute independently of any rendering framework and the GUI shell is replaceable.

**Source:** WB Architecture Brief §3 Principle 1, §4 Crate Architecture. [WB]

#### Acceptance Criteria

1. THE `ff-core` crate SHALL have zero direct or transitive dependencies on any GUI framework library (egui, winit, wgpu, or any future rendering library) in its `Cargo.toml` dependency tree.
2. THE `ff-core` crate SHALL define the boundary between business logic and the rendering shell by exposing a public messaging/event interface through which the GUI shell communicates, without requiring the GUI shell crate to be present at compile time.
3. THE `ff-core` crate SHALL provide the `WorkbenchApp` struct that owns all platform state and serves as the single entry point for subsystem initialization, event dispatch, and lifecycle management.
4. WHEN the `WorkbenchApp` is constructed, THE `WorkbenchApp` SHALL accept a configuration context (from `ff-config`) and a logging handle (from `ff-logging`) as its only required dependencies.
5. THE `ff-core` crate SHALL expose all business logic APIs through well-defined trait boundaries, enabling any GUI shell implementation to drive the workbench without knowledge of internal state representation.
6. IF the GUI shell crate is absent from the workspace (e.g., in a headless testing or CLI scenario), THEN THE `ff-core` crate SHALL compile, link, and execute all non-rendering functionality without error.

---

### Requirement 2: Service Registry

**User Story:** As a workbench developer, I want a service registry that provides type-safe access to all subsystems, so that any component can obtain the services it needs without tight coupling to concrete implementations.

**Source:** WB Architecture Brief §5 Service Architecture. [WB]

#### Acceptance Criteria

1. THE Platform_Core SHALL provide a Service_Registry that allows subsystems to register themselves during the startup sequence.
2. THE Service_Registry SHALL support type-safe service access via a generic method (`get_service::<T>()`) that returns an `Option<&T>` or equivalent reference to the requested service type, without requiring downcasting from a trait object by the caller.
3. THE Service_Registry SHALL enforce service ordering guarantees: services registered earlier in the Startup_Sequence SHALL be available to services registered later, but not vice versa during initialization.
4. THE Service_Registry SHALL guarantee that the initialization order is deterministic and follows the defined Startup_Sequence: logging → configuration → VFS → commands → plugins.
5. THE Service_Registry SHALL provide thread-safe read access to registered services from any thread (including Tokio worker threads) without requiring the caller to acquire an external lock, using interior mutability or shared references as appropriate.
6. IF a service is requested via `get_service::<T>()` and no service of that type has been registered, THEN THE Service_Registry SHALL return `None` (or equivalent absence indicator) without panicking.
7. THE Service_Registry SHALL NOT allow a service to be registered more than once for the same type; attempting to register a duplicate SHALL return an error and write a WARN-level log record.
8. WHEN all services have been registered, THE Service_Registry SHALL transition to a frozen/read-only state where no further registrations are accepted, preventing mutation of the service set after startup completes.

---

### Requirement 3: Event Bus and Message Passing

**User Story:** As a workbench developer, I want an internal event/message system that connects the core to the GUI shell and between subsystems, so that business logic and rendering are fully decoupled and events can flow bidirectionally.

**Source:** WB Architecture Brief §6 Event Architecture. [WB]

#### Acceptance Criteria

1. THE Platform_Core SHALL provide an Event_Bus that enables bidirectional event flow: user input events flow from the GUI shell to Platform_Core, and state-change events flow from Platform_Core to the GUI shell.
2. THE Event_Bus SHALL support the following event categories: commands (user-initiated operations), notifications (informational messages to the GUI), state-change signals (model updates that require re-rendering), and progress updates (long-running operation status).
3. THE Event_Bus SHALL support event dispatch from Tokio async worker threads without blocking the caller, using an async-capable channel or equivalent mechanism.
4. THE Event_Bus SHALL support event subscription: subsystems and the GUI shell SHALL be able to register interest in specific event types and receive only those events.
5. WHEN an event is dispatched to the Event_Bus, THE Event_Bus SHALL deliver it to all registered subscribers for that event type within the same application tick or frame cycle (no unbounded delivery delay).
6. THE Event_Bus SHALL NOT require the GUI shell to be present for event dispatch to succeed; events dispatched when no GUI subscriber is registered SHALL be silently discarded for GUI-targeted events, or processed by core subscribers as appropriate.
7. IF the Event_Bus internal buffer reaches capacity (defined as 10,000 pending events), THEN THE Event_Bus SHALL drop the oldest undelivered events and write a WARN-level log record indicating the number of events dropped.
8. THE Event_Bus SHALL be safe to use from any thread without external synchronization, maintaining the same thread-safety guarantees as the Service_Registry.

---

### Requirement 4: Crate Structure and Layer Rules

**User Story:** As a workbench developer, I want strict layering rules that prevent reverse dependencies, so that the codebase remains modular, independently testable, and the GUI shell remains replaceable.

**Source:** WB Architecture Brief §4 Crate Structure. [WB]

#### Acceptance Criteria

1. THE workspace SHALL define exactly five layers with the following membership:
   - **Foundation Layer**: `ff-logging` (no dependencies on other `ff-*` crates)
   - **Core Layer**: `ff-core`, `ff-config`, `ff-command`, `ff-plugin`, `ff-workflow`, `ff-vfs`
   - **Editor Layer**: `ff-document`, `ff-edit`, `ff-undo`, `ff-viewport`, `ff-display-lines`
   - **Feature Layer**: `ff-find`, `ff-line-commands`, `ff-exclude`, `ff-nav`, and other feature crates
   - **Shell Layer**: `ff-desktop` (egui GUI shell)
2. CRATES in an upper layer SHALL be permitted to depend on crates in any lower layer; crates SHALL NEVER depend on crates in a higher layer (strict downward-only dependency direction).
3. CRATES within the same layer SHALL be permitted to have inter-dependencies only where explicitly documented in the sub-project dependency graph; circular dependencies within a layer are forbidden.
4. THE Foundation Layer crate (`ff-logging`) SHALL have zero dependencies on any other `ff-*` crate in the workspace.
5. THE Shell Layer crate (`ff-desktop`) SHALL depend on Core Layer and Editor Layer crates as needed, but NO crate in any other layer SHALL depend on `ff-desktop` or any Shell Layer crate.
6. EACH crate in the workspace SHALL be independently compilable via `cargo check -p ff-{crate-name}` without requiring Shell Layer crates to be present.
7. IF a developer introduces a dependency that violates the layer rules (e.g., a Core Layer crate depending on an Editor Layer crate), THEN `cargo check` SHALL fail due to the resulting circular or missing dependency, enforced by the Cargo.toml dependency declarations.

---

### Requirement 5: Application Lifecycle — Startup

**User Story:** As a workbench developer, I want a well-defined startup sequence that initializes subsystems in dependency order, so that each subsystem can rely on its dependencies being available when it initializes.

**Source:** WB Architecture Brief §3 Lifecycle, §5 Startup Order. [WB]

#### Acceptance Criteria

1. THE WorkbenchApp SHALL initialize subsystems in the following deterministic order: logging → configuration → VFS → commands → plugins → GUI shell (when present).
2. WHEN each subsystem initializes successfully, THE WorkbenchApp SHALL write an INFO-level log record containing the subsystem name and initialization duration in milliseconds.
3. IF a non-critical subsystem (plugins, GUI shell) fails to initialize, THEN THE WorkbenchApp SHALL log an ERROR-level record describing the failure and continue operating with reduced functionality — the application SHALL NOT terminate.
4. IF a critical subsystem (logging, configuration, VFS, commands) fails to initialize, THEN THE WorkbenchApp SHALL log an ERROR-level record (if logging is available), attempt an orderly shutdown of any already-initialized subsystems, and terminate the application with a non-zero exit code.
5. THE WorkbenchApp SHALL complete the full startup sequence (all subsystems initialized) within 5 seconds on a system meeting minimum hardware requirements; if startup takes longer, THE WorkbenchApp SHALL provide progress feedback to the GUI shell (if connected) via the Event_Bus.
6. WHEN the startup sequence completes successfully, THE WorkbenchApp SHALL dispatch a `WorkbenchReady` event via the Event_Bus, signalling to the GUI shell that it may begin rendering the main interface.

---

### Requirement 6: Application Lifecycle — Shutdown

**User Story:** As a workbench developer, I want a well-defined shutdown sequence that tears down subsystems in reverse order with a grace period, so that all subsystems can persist state and release resources cleanly.

**Source:** WB Architecture Brief §3 Lifecycle, §6 Shutdown. [WB]

#### Acceptance Criteria

1. WHEN a shutdown is initiated (via user action, window close, or OS signal), THE WorkbenchApp SHALL shut down subsystems in the reverse order of their initialization: GUI shell → plugins → commands → VFS → configuration → logging.
2. EACH subsystem SHALL be given a grace period of up to 3 seconds to complete its shutdown operations (flushing buffers, persisting state, releasing resources) before THE WorkbenchApp proceeds to the next subsystem.
3. IF a subsystem's shutdown exceeds the 3-second grace period, THEN THE WorkbenchApp SHALL log a WARN-level record, forcibly terminate that subsystem's operations, and proceed to shut down the next subsystem.
4. WHEN all subsystems have been shut down, THE WorkbenchApp SHALL write a final INFO-level log record ("Application shutdown complete"), flush the logging subsystem, and exit the process with exit code 0.
5. IF a panic occurs during shutdown, THEN THE WorkbenchApp SHALL catch the panic (where possible), log an ERROR-level record with the panic message, and continue shutting down remaining subsystems — a panic in one subsystem SHALL NOT prevent shutdown of others.
6. THE WorkbenchApp SHALL support graceful shutdown triggered by OS signals: SIGTERM/SIGINT on Unix, WM_CLOSE/CTRL_CLOSE_EVENT on Windows.

---

### Requirement 7: Panic Handling and Recovery

**User Story:** As a user, I want the workbench to recover gracefully from panics where possible, so that a bug in one subsystem does not crash the entire application and lose my unsaved work.

**Source:** WB Architecture Brief §6 Resilience. [WB]

#### Acceptance Criteria

1. THE WorkbenchApp SHALL install a custom panic hook at startup (before any other subsystem initializes) that captures panic information (message, location, backtrace).
2. WHEN a panic occurs on a background thread (Tokio worker, plugin thread), THE panic hook SHALL log an ERROR-level record containing the panic details and the thread name, and THE WorkbenchApp SHALL continue operating on the main thread without terminating.
3. WHEN a panic occurs on the main thread during normal operation (after startup), THE WorkbenchApp SHALL attempt to persist unsaved state (via the document model's auto-save mechanism if available), log the panic details, and initiate an orderly shutdown sequence.
4. IF recovery from a panic is not possible (e.g., the panic corrupted shared state), THEN THE WorkbenchApp SHALL terminate the process with a non-zero exit code after logging the panic details, rather than continuing in an undefined state.
5. THE panic hook SHALL NOT panic itself; if writing the panic log record fails, THE panic hook SHALL silently abandon logging and allow the default panic behaviour to proceed.

---

### Requirement 8: Hot-Restart Capability

**User Story:** As a developer, I want plugins to be reloadable without a full application restart, so that I can iterate on plugin development rapidly and users can update plugins without closing their work.

**Source:** WB Architecture Brief §10 Plugin Lifecycle. [WB]

#### Acceptance Criteria

1. THE WorkbenchApp SHALL support hot-restart of individual plugins: deactivating a plugin, unloading it, loading the updated version, and re-activating it, without shutting down Platform_Core or other subsystems.
2. WHEN a plugin is hot-restarted, THE WorkbenchApp SHALL first call the plugin's `deactivate` method, then `shutdown`, then load the new plugin binary/module, then call `initialize` and `activate` on the new instance.
3. DURING a plugin hot-restart, THE WorkbenchApp SHALL maintain all non-plugin state (documents, undo history, configuration, VFS mounts) unchanged.
4. IF the new plugin version fails to initialize during hot-restart, THEN THE WorkbenchApp SHALL log an ERROR-level record, discard the failed load, and leave the plugin in an unloaded state — the application SHALL continue operating without that plugin.
5. THE WorkbenchApp SHALL dispatch a `PluginReloaded { plugin_name }` event via the Event_Bus after a successful hot-restart, allowing other subsystems and the GUI shell to update any plugin-dependent UI.

---

### Requirement 9: Thread Model

**User Story:** As a workbench developer, I want a clearly defined thread model, so that I know which operations run on which threads and how threads communicate safely.

**Source:** WB Architecture Brief §9 Concurrency Model. [WB]

#### Acceptance Criteria

1. THE WorkbenchApp SHALL define three distinct thread contexts:
   - **Main thread**: GUI rendering loop (when GUI shell is active); on non-GUI configurations, this thread runs the Platform_Core event loop.
   - **Core thread**: If the GUI shell is present and owns the main thread, Platform_Core MAY run its event loop on a dedicated core thread; the architecture SHALL support both same-thread and separate-thread configurations.
   - **Tokio runtime**: A multi-threaded Tokio runtime for async I/O workers (file operations, network, background tasks).
2. THE WorkbenchApp SHALL initialize the Tokio runtime during startup (after logging, before VFS initialization) and shut it down during the shutdown sequence (after VFS, before configuration shutdown).
3. ALL communication between threads SHALL use channels (mpsc, broadcast, oneshot as appropriate) or atomic operations — shared mutable state protected by locks SHALL be minimized and documented where used.
4. THE GUI shell thread SHALL NOT perform blocking I/O operations (file reads, network requests, database queries); all such operations SHALL be dispatched to the Tokio runtime via the Event_Bus or direct channel communication.
5. WHEN a Tokio worker completes an async operation, THE worker SHALL communicate the result back to the requesting thread via the Event_Bus or a response channel — never by directly mutating GUI state.
6. THE WorkbenchApp SHALL ensure that all spawned threads and Tokio tasks are tracked and joined/cancelled during the shutdown sequence, preventing resource leaks or orphaned background work.
7. IF the Tokio runtime encounters a fatal error (e.g., all worker threads panicked), THEN THE WorkbenchApp SHALL log an ERROR-level record and initiate an orderly shutdown, as async I/O capability is considered critical.

