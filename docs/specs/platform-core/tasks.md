# Implementation Plan: Platform Core (`ff-core`)

## Overview

This plan covers the complete implementation of the `ff-core` crate — the GUI-independent central orchestration layer for FileForgeWorkbench. The platform-core owns all application state, manages the lifecycle of every subsystem, defines the event bus for decoupled communication, enforces strict layer separation, and provides panic handling and recovery.

This is a **Wave 2 (Platform Architecture)** sub-project with an upstream dependency on `ff-logging` (Wave 0).

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-core/Cargo.toml` with dependencies (ff-logging, tokio, thiserror, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-core/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `app.rs`, `service_registry.rs`, `event_bus.rs`, `lifecycle.rs`, `shutdown.rs`, `panic_hook.rs`, `hot_restart.rs`, `thread_model.rs`, `layer_rules.rs`, `error.rs`
  - [x] 1.4 Add `ff-core` to workspace `Cargo.toml` members list
  - [x] 1.5 Verify `cargo check -p ff-core` compiles with zero GUI dependencies
  - Covers: Requirement 1 (AC 1.1, 1.6), structural foundation

- [x] 2. WorkbenchApp struct and core trait boundaries
  - [x] 2.1 Define `WorkbenchApp` struct that owns all platform state
  - [x] 2.2 Implement `WorkbenchApp::new()` constructor accepting a configuration context and a logging handle as required dependencies
  - [x] 2.3 Define public trait boundaries for business logic APIs (e.g., `WorkbenchService`, `LifecycleManaged`)
  - [x] 2.4 Ensure WorkbenchApp is the single entry point for subsystem initialization, event dispatch, and lifecycle management
  - [x] 2.5 Write unit tests for construction with valid dependencies and trait exposure
  - Covers: Requirement 1 (AC 1.3, 1.4, 1.5)

- [x] 3. Event and messaging interface definition
  - [x] 3.1 Define the public messaging/event interface types (event enums, message traits) for GUI shell communication
  - [x] 3.2 Ensure the interface is defined without requiring the GUI shell crate at compile time
  - [x] 3.3 Define event categories: commands, notifications, state-change signals, progress updates
  - [x] 3.4 Write unit tests verifying interface compiles independently of any GUI crate
  - Covers: Requirement 1 (AC 1.2), Requirement 3 (AC 3.2)

- [x] 4. Service Registry — core implementation
  - [x] 4.1 Implement `ServiceRegistry` struct with type-safe storage using `TypeId`-keyed map
  - [x] 4.2 Implement `register_service::<T>()` method for service registration during startup
  - [x] 4.3 Implement `get_service::<T>()` returning `Option<&T>` without requiring caller downcasting
  - [x] 4.4 Implement duplicate registration detection — return error and write WARN-level log
  - [x] 4.5 Write unit tests for registration, retrieval, duplicate rejection, and absence case
  - Covers: Requirement 2 (AC 2.1, 2.2, 2.6, 2.7)

- [x] 5. Service Registry — ordering and thread safety
  - [x] 5.1 Implement initialization order tracking — services registered earlier are available to later registrants
  - [x] 5.2 Enforce deterministic startup sequence: logging → configuration → VFS → commands → plugins
  - [x] 5.3 Implement frozen/read-only state transition after all services are registered
  - [x] 5.4 Implement thread-safe read access using `Arc` and interior mutability (no external lock required by caller)
  - [x] 5.5 Write unit tests for ordering guarantees, freeze behavior, and concurrent read access
  - Covers: Requirement 2 (AC 2.3, 2.4, 2.5, 2.8)

- [x] 6. Event Bus — core dispatch mechanism
  - [x] 6.1 Implement `EventBus` struct with internal async-capable bounded channel (capacity: 10,000 events)
  - [x] 6.2 Implement bidirectional event flow: input events (GUI→Core) and state-change events (Core→GUI)
  - [x] 6.3 Implement non-blocking event dispatch from any thread (including Tokio worker threads)
  - [x] 6.4 Implement thread-safe access without external synchronization
  - [x] 6.5 Write unit tests for dispatch, bidirectional flow, and thread safety
  - Covers: Requirement 3 (AC 3.1, 3.3, 3.8)

- [x] 7. Event Bus — subscription and delivery
  - [x] 7.1 Implement event subscription: subsystems and GUI shell register interest in specific event types
  - [x] 7.2 Implement filtered delivery — subscribers receive only events matching their registered interest
  - [x] 7.3 Implement delivery guarantee: events delivered to all subscribers within the same tick/frame cycle
  - [x] 7.4 Implement GUI-absent handling: GUI-targeted events silently discarded when no GUI subscriber present
  - [x] 7.5 Write unit tests for subscription, filtered delivery, tick-bound delivery, and GUI absence
  - Covers: Requirement 3 (AC 3.4, 3.5, 3.6)

- [x] 8. Event Bus — overflow and backpressure
  - [x] 8.1 Implement buffer capacity enforcement at 10,000 pending events
  - [x] 8.2 Implement oldest-event-drop policy when buffer is full
  - [x] 8.3 Implement WARN-level log record on overflow with count of dropped events
  - [x] 8.4 Write unit tests for overflow behavior, drop counting, and warn logging
  - Covers: Requirement 3 (AC 3.7)

- [x] 9. Startup sequence
  - [x] 9.1 Implement deterministic initialization order: logging → configuration → VFS → commands → plugins → GUI shell
  - [x] 9.2 Implement per-subsystem INFO-level log record on successful initialization (subsystem name + duration in ms)
  - [x] 9.3 Implement non-critical subsystem failure handling: log ERROR, continue with reduced functionality (plugins, GUI shell)
  - [x] 9.4 Implement critical subsystem failure handling: log ERROR, orderly shutdown of initialized subsystems, non-zero exit code (logging, configuration, VFS, commands)
  - [x] 9.5 Implement 5-second startup timeout with progress feedback to GUI shell via Event_Bus
  - [x] 9.6 Implement `WorkbenchReady` event dispatch on successful startup completion
  - [x] 9.7 Write unit tests for ordering, logging, failure handling, timeout, and ready event
  - Covers: Requirement 5 (AC 5.1, 5.2, 5.3, 5.4, 5.5, 5.6)

- [x] 10. Shutdown sequence
  - [x] 10.1 Implement reverse-order shutdown: GUI shell → plugins → commands → VFS → configuration → logging
  - [x] 10.2 Implement per-subsystem 3-second grace period for cleanup operations
  - [x] 10.3 Implement grace period timeout handling: log WARN, forcibly terminate, proceed to next subsystem
  - [x] 10.4 Implement final INFO-level log record ("Application shutdown complete"), flush logging, exit code 0
  - [x] 10.5 Implement panic-during-shutdown resilience: catch panic, log ERROR, continue with remaining subsystems
  - [x] 10.6 Implement OS signal-triggered shutdown: SIGTERM/SIGINT on Unix, WM_CLOSE/CTRL_CLOSE_EVENT on Windows
  - [x] 10.7 Write unit tests for reverse ordering, timeouts, panic resilience, and signal handling
  - Covers: Requirement 6 (AC 6.1, 6.2, 6.3, 6.4, 6.5, 6.6)

- [x] 11. Panic handling and recovery
  - [x] 11.1 Implement custom panic hook installation at startup (before any other subsystem initializes)
  - [x] 11.2 Implement background thread panic capture: log ERROR with panic details and thread name, continue main thread operation
  - [x] 11.3 Implement main thread panic response: persist unsaved state (if available), log panic details, initiate orderly shutdown
  - [x] 11.4 Implement unrecoverable panic detection: terminate with non-zero exit code rather than continue in undefined state
  - [x] 11.5 Ensure panic hook never panics itself — silently abandon logging on failure
  - [x] 11.6 Write unit tests for background thread panic recovery, main thread panic behavior, and hook robustness
  - Covers: Requirement 7 (AC 7.1, 7.2, 7.3, 7.4, 7.5)

- [x] 12. Hot-restart capability
  - [x] 12.1 Implement individual plugin hot-restart sequence: deactivate → shutdown → load new → initialize → activate
  - [x] 12.2 Implement state preservation during hot-restart: documents, undo history, configuration, VFS mounts unchanged
  - [x] 12.3 Implement failed hot-restart recovery: log ERROR, discard failed load, leave plugin unloaded
  - [x] 12.4 Implement `PluginReloaded { plugin_name }` event dispatch via Event_Bus after successful hot-restart
  - [x] 12.5 Write unit tests for full restart cycle, state preservation, failure recovery, and event dispatch
  - Covers: Requirement 8 (AC 8.1, 8.2, 8.3, 8.4, 8.5)

- [x] 13. Thread model and Tokio runtime management
  - [x] 13.1 Define three thread contexts: main thread (GUI/event loop), core thread (optional dedicated), Tokio runtime (multi-threaded async I/O)
  - [x] 13.2 Implement Tokio runtime initialization during startup (after logging, before VFS)
  - [x] 13.3 Implement Tokio runtime shutdown during teardown (after VFS, before configuration)
  - [x] 13.4 Implement channel-based inter-thread communication (mpsc, broadcast, oneshot as appropriate)
  - [x] 13.5 Enforce GUI thread non-blocking rule: no blocking I/O operations on the GUI shell thread
  - [x] 13.6 Implement async result communication: workers send results via Event_Bus or response channels, never by mutating GUI state directly
  - [x] 13.7 Implement spawned thread/task tracking and join/cancel during shutdown (prevent resource leaks)
  - [x] 13.8 Implement Tokio fatal error handling: log ERROR and initiate orderly shutdown
  - [x] 13.9 Write unit tests for runtime lifecycle, channel communication, task tracking, and fatal error behavior
  - Covers: Requirement 9 (AC 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7)

- [x] 14. Layer rule enforcement and documentation
  - [x] 14.1 Document five-layer structure in crate-level documentation: Foundation, Core, Editor, Feature, Shell
  - [x] 14.2 Configure `Cargo.toml` dependency declarations to enforce downward-only dependency direction
  - [x] 14.3 Verify Foundation Layer (`ff-logging`) has zero `ff-*` dependencies
  - [x] 14.4 Verify Shell Layer dependencies: `ff-desktop` depends on lower layers, no other layer depends on Shell
  - [x] 14.5 Verify each crate compiles independently via `cargo check -p ff-{name}` without Shell Layer
  - [x] 14.6 Document layer rule enforcement mechanism: `cargo check` fails on violation due to Cargo.toml declarations
  - [x] 14.7 Write integration tests verifying `ff-core` compiles without any GUI/Shell layer crates present
  - Covers: Requirement 4 (AC 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7)

- [x] 15. Property-based tests
  - [x] 15.1 Write PBT: Service Registry type-safety invariant
  - [x] 15.2 Write PBT: Event Bus delivery completeness
  - [x] 15.3 Write PBT: Event Bus overflow monotonicity
  - [x] 15.4 Write PBT: Startup sequence ordering determinism
  - [x] 15.5 Write PBT: Shutdown reverse-order invariant
  - [x] 15.6 Write PBT: Service Registry freeze immutability
  - Covers: Requirements 2, 3, 5, 6 (see Property-Based Test Definitions below)

---

## Property-Based Test Definitions

### Property 1: Service Registry Type-Safety Invariant

**Validates: Requirement 2.2, 2.6**

- **Statement:** For any set of distinct service types registered in any order, `get_service::<T>()` SHALL return `Some` for every registered type and `None` for every unregistered type. No type confusion or erroneous `None` shall occur.
- **Strategy:** Generate:
  - Number of distinct service types: [1, 20] (simulated via wrapper newtypes around u64)
  - Registration order: random permutation of types
  - Query set: mix of registered and unregistered type queries
- **Invariant:** For all types `T` in registered set: `get_service::<T>() == Some(_)`; for all types `U` not in registered set: `get_service::<U>() == None`

### Property 2: Event Bus Delivery Completeness

**Validates: Requirement 3.4, 3.5**

- **Statement:** For any set of subscribers each with a filter and any sequence of dispatched events, every subscriber SHALL receive exactly the events matching its registered filter — no missed deliveries and no spurious deliveries.
- **Strategy:** Generate:
  - Number of subscribers: [1, 10]
  - Filter per subscriber: random subset of event categories {Command, Notification, StateChange, Progress}
  - Event sequence: [10, 500] events with random categories
- **Invariant:** For each subscriber, received events == dispatched events whose category is in that subscriber's filter set

### Property 3: Event Bus Overflow Monotonicity

**Validates: Requirement 3.7**

- **Statement:** The dropped event counter on the Event_Bus is monotonically non-decreasing. For any sequence of dispatches (some succeeding, some overflowing), the counter at time T₂ >= counter at time T₁ for T₂ > T₁.
- **Strategy:** Generate:
  - Sequence of [100, 20000] dispatch attempts
  - Channel capacity: fixed at 10,000
  - Consumer drain rate: [0, 5000] events per tick (simulating variable consumption)
- **Invariant:** Counter value never decreases across consecutive observations

### Property 4: Startup Sequence Ordering Determinism

**Validates: Requirement 5.1, Requirement 2.4**

- **Statement:** For any configuration context, the startup sequence SHALL always produce the same initialization order: logging → configuration → VFS → commands → plugins → GUI shell. No reordering shall occur regardless of timing or system load.
- **Strategy:** Generate:
  - Configuration variants: random valid config values
  - Simulated subsystem initialization durations: [0, 100] ms per subsystem
  - Number of repeated startup attempts: [5, 20]
- **Invariant:** Recorded initialization order is identical across all attempts and matches the defined sequence

### Property 5: Shutdown Reverse-Order Invariant

**Validates: Requirement 6.1**

- **Statement:** For any set of successfully initialized subsystems, the shutdown sequence SHALL visit them in the exact reverse order of their initialization. No subsystem shall be shut down before a subsystem that was initialized after it.
- **Strategy:** Generate:
  - Subset of subsystems that successfully initialized: random subsets of the full sequence (simulating partial startup due to failures)
  - Simulated shutdown durations: [0, 3500] ms per subsystem (some exceeding grace period)
- **Invariant:** Shutdown visitation order == reverse(initialization order for the subset that was initialized)

### Property 6: Service Registry Freeze Immutability

**Validates: Requirement 2.8**

- **Statement:** After the Service_Registry transitions to frozen state, any registration attempt SHALL fail with an error. The set of registered services SHALL remain unchanged for all subsequent `get_service` calls.
- **Strategy:** Generate:
  - Initial registration set: [1, 15] services
  - Post-freeze registration attempts: [1, 10] additional services
  - Post-freeze query set: mix of initially registered and new types
- **Invariant:** All post-freeze registrations return `Err`; all post-freeze queries return the same results as pre-freeze queries

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and Interfaces", "tasks": ["2", "3"], "dependsOn": [0] },
    { "id": 2, "label": "Service Registry", "tasks": ["4", "5"], "dependsOn": [1] },
    { "id": 3, "label": "Event Bus", "tasks": ["6", "7", "8"], "dependsOn": [1] },
    { "id": 4, "label": "Lifecycle Management", "tasks": ["9", "10", "11"], "dependsOn": [2, 3] },
    { "id": 5, "label": "Advanced Features", "tasks": ["12", "13"], "dependsOn": [4] },
    { "id": 6, "label": "Layer Enforcement", "tasks": ["14"], "dependsOn": [0] },
    { "id": 7, "label": "Validation and PBT", "tasks": ["15"], "dependsOn": [2, 3, 4, 5] }
  ]
}
```

---

## Notes

- This is a Wave 2 (Platform Architecture) crate depending only on `ff-logging` (Wave 0 / Foundation Layer)
- The `ff-config` crate does not exist yet at this wave; `WorkbenchApp` accepts configuration via a trait or struct that will later be backed by the configuration-system
- Plugin-related hot-restart (Task 12) defines the interface and protocol; concrete plugin types come from `ff-plugin` (a separate Wave 2 crate)
- The Event Bus implementation should use `tokio::sync::broadcast` or `tokio::sync::mpsc` for async-capable channels
- Thread model (Task 13) must support both configurations: GUI shell on main thread with Core on dedicated thread, and non-GUI mode where Core owns the main thread
- Layer rule enforcement (Task 14) is primarily about documentation and Cargo.toml configuration — it does not require runtime code, but integration tests can verify compilation independence
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- OS signal handling (Task 10.6) uses `tokio::signal` for cross-platform support
- The `WorkbenchReady` event (Task 9.6) is the contract between platform-core and the GUI shell — the shell must not render the main interface until it receives this event

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Core Layer Architecture | AC 1.1 (zero GUI deps) | Task 1 |
| | AC 1.2 (public messaging interface) | Task 3 |
| | AC 1.3 (WorkbenchApp struct) | Task 2 |
| | AC 1.4 (constructor deps: config + logging) | Task 2 |
| | AC 1.5 (trait boundaries for APIs) | Task 2 |
| | AC 1.6 (compiles without GUI shell) | Tasks 1, 14 |
| Req 2: Service Registry | AC 2.1 (registration during startup) | Task 4 |
| | AC 2.2 (type-safe get_service) | Task 4 |
| | AC 2.3 (ordering guarantees) | Task 5 |
| | AC 2.4 (deterministic startup order) | Task 5 |
| | AC 2.5 (thread-safe read access) | Task 5 |
| | AC 2.6 (None for missing service) | Task 4 |
| | AC 2.7 (duplicate registration error + WARN) | Task 4 |
| | AC 2.8 (frozen/read-only after startup) | Task 5 |
| Req 3: Event Bus | AC 3.1 (bidirectional event flow) | Task 6 |
| | AC 3.2 (event categories) | Tasks 3, 6 |
| | AC 3.3 (async dispatch from Tokio threads) | Task 6 |
| | AC 3.4 (event subscription) | Task 7 |
| | AC 3.5 (same-tick delivery) | Task 7 |
| | AC 3.6 (GUI-absent handling) | Task 7 |
| | AC 3.7 (overflow: drop oldest + WARN log) | Task 8 |
| | AC 3.8 (thread safety) | Task 6 |
| Req 4: Layer Rules | AC 4.1 (five layers defined) | Task 14 |
| | AC 4.2 (downward-only deps) | Task 14 |
| | AC 4.3 (same-layer inter-deps documented) | Task 14 |
| | AC 4.4 (Foundation zero ff-* deps) | Task 14 |
| | AC 4.5 (Shell deps downward, nothing depends on Shell) | Task 14 |
| | AC 4.6 (independent cargo check per crate) | Tasks 1, 14 |
| | AC 4.7 (cargo check fails on violation) | Task 14 |
| Req 5: Startup | AC 5.1 (deterministic order) | Task 9 |
| | AC 5.2 (INFO log per subsystem + duration) | Task 9 |
| | AC 5.3 (non-critical failure: log + continue) | Task 9 |
| | AC 5.4 (critical failure: log + shutdown + exit) | Task 9 |
| | AC 5.5 (5-second timeout + progress feedback) | Task 9 |
| | AC 5.6 (WorkbenchReady event) | Task 9 |
| Req 6: Shutdown | AC 6.1 (reverse order) | Task 10 |
| | AC 6.2 (3-second grace period) | Task 10 |
| | AC 6.3 (timeout: WARN + forcible termination) | Task 10 |
| | AC 6.4 (final INFO + flush + exit code 0) | Task 10 |
| | AC 6.5 (panic resilience during shutdown) | Task 10 |
| | AC 6.6 (OS signal-triggered shutdown) | Task 10 |
| Req 7: Panic Handling | AC 7.1 (custom panic hook at startup) | Task 11 |
| | AC 7.2 (background thread panic: log + continue) | Task 11 |
| | AC 7.3 (main thread panic: persist + shutdown) | Task 11 |
| | AC 7.4 (unrecoverable: terminate with non-zero) | Task 11 |
| | AC 7.5 (hook never panics) | Task 11 |
| Req 8: Hot-Restart | AC 8.1 (individual plugin hot-restart) | Task 12 |
| | AC 8.2 (deactivate → shutdown → load → init → activate) | Task 12 |
| | AC 8.3 (non-plugin state preserved) | Task 12 |
| | AC 8.4 (failed load: log ERROR, leave unloaded) | Task 12 |
| | AC 8.5 (PluginReloaded event) | Task 12 |
| Req 9: Thread Model | AC 9.1 (three thread contexts) | Task 13 |
| | AC 9.2 (Tokio runtime init timing) | Task 13 |
| | AC 9.3 (channel-based communication) | Task 13 |
| | AC 9.4 (GUI thread non-blocking) | Task 13 |
| | AC 9.5 (async result via Event_Bus/channels) | Task 13 |
| | AC 9.6 (thread/task tracking + join/cancel) | Task 13 |
| | AC 9.7 (Tokio fatal error: shutdown) | Task 13 |
