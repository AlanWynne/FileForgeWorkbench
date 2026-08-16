//! # ff-core — GUI-Independent Central Orchestration Layer
//!
//! This crate is the **central orchestration layer** for the FileForgeWorkbench
//! platform. It owns all application state, manages the lifecycle of every
//! subsystem, and defines the strict boundary between business logic and the
//! replaceable GUI rendering shell.
//!
//! ## Responsibilities
//!
//! - Application lifecycle (startup sequence, shutdown sequence, OS signal handling)
//! - Service Registry (type-safe subsystem registration and lookup)
//! - Event Bus (typed event dispatch, subscription, async-safe channels)
//! - Thread model (main thread, core thread, Tokio runtime management)
//! - Panic handling (custom hook, recovery, graceful degradation)
//! - Plugin hot-restart orchestration (deactivate → reload → activate)
//!
//! ## GUI Independence
//!
//! `ff-core` has **zero** direct or transitive dependencies on any GUI framework
//! library (egui, winit, wgpu). All business logic executes within this layer and
//! communicates with the GUI shell through a defined event/messaging interface.
//! The rendering shell can be replaced without rewriting or recompiling any
//! business logic.
//!
//! ## Layer Position
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              Shell Layer: ff-desktop (egui)                   │
//! │         Depends on ff-core; ff-core NEVER depends on this    │
//! ├─────────────────────────────────────────────────────────────┤
//! │              Core Layer: ff-core (THIS CRATE)                 │
//! │  Also: ff-config, ff-command, ff-plugin, ff-workflow, ff-vfs │
//! ├─────────────────────────────────────────────────────────────┤
//! │              Foundation Layer: ff-logging                     │
//! │              No dependencies on other ff-* crates            │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ff_core::WorkbenchApp;
//! // WorkbenchApp is the single entry point for subsystem initialization,
//! // event dispatch, and lifecycle management.
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// `WorkbenchApp` struct — the top-level owner of all platform state and the
/// single entry point for initialization, event dispatch, and lifecycle management.
pub mod app;

/// `ServiceRegistry` — type-safe container for subsystem registration and lookup.
pub mod service_registry;

/// `EventBus` — async-safe typed event dispatch and subscription system for
/// bidirectional communication between core and shell.
pub mod event_bus;

/// Startup lifecycle sequences — ordered subsystem initialization with
/// dependency guarantees and progress reporting.
pub mod lifecycle;

/// Shutdown logic — reverse-ordered teardown with grace periods, timeout
/// enforcement, and OS signal handling.
pub mod shutdown;

/// Custom panic hook — captures panic information, logs details, and
/// coordinates recovery or graceful degradation.
pub mod panic_hook;

/// Plugin hot-restart orchestration — deactivate, unload, reload, and
/// re-activate plugins without full application restart.
pub mod hot_restart;

/// Thread model — Tokio runtime creation, task tracking, and join-on-shutdown
/// for all async I/O workers.
pub mod thread_model;

/// Thread context enum identifying which execution context code runs in.
pub use thread_model::ThreadContext;

/// Wrapper around the Tokio multi-threaded runtime with task tracking.
pub use thread_model::TokioRuntime;

/// A tracked async task with a name and cancellation token.
pub use thread_model::TrackedTask;

/// Layer rules documentation — defines the five workspace layers and their
/// dependency direction constraints.
pub mod layer_rules;

/// `ConfigProvider` trait — defines the configuration interface accepted by
/// `WorkbenchApp`, decoupling ff-core from the configuration crate's internals.
pub mod config;

/// `CoreError` enum — unified error type for the ff-core crate.
pub mod error;

// ─── Public API Re-exports ──────────────────────────────────────────────────

/// The primary application struct that owns all platform state.
pub use app::WorkbenchApp;

/// Type-safe service registry for subsystem access.
pub use service_registry::ServiceRegistry;

/// Thread-safe, read-only shared view of the service registry.
pub use service_registry::SharedRegistry;

/// Async-safe event bus for typed message dispatch.
pub use event_bus::EventBus;

/// Default capacity for the event bus bounded channel (10,000 events).
pub use event_bus::DEFAULT_EVENT_BUS_CAPACITY;

/// All events that flow through the Event Bus.
pub use event_bus::WorkbenchEvent;

/// Event categories for subscription filtering.
pub use event_bus::EventCategory;

/// Opaque document identifier.
pub use event_bus::DocumentId;

/// Opaque operation identifier for progress tracking.
pub use event_bus::OperationId;

/// Severity level for GUI notifications.
pub use event_bus::NotificationSeverity;

/// Progress information for long-running operations.
pub use event_bus::ProgressInfo;

/// Parameters passed to a command at dispatch time.
pub use event_bus::CommandParams;

/// Outcome of a dispatched command.
pub use event_bus::CommandOutcome;

/// A single parameter value within CommandParams.
pub use event_bus::ParamValue;

/// Filter for event subscriptions.
pub use event_bus::EventFilter;

/// A subscription handle for receiving filtered events from the EventBus.
pub use event_bus::EventSubscription;

/// Unified error type for the ff-core crate.
pub use error::CoreError;

/// Application lifecycle phase tracking.
pub use lifecycle::LifecyclePhase;

/// Trait boundary for lifecycle-managed subsystems.
pub use lifecycle::Subsystem;

/// Descriptor providing name, criticality, and startup order for a subsystem.
pub use lifecycle::SubsystemDescriptor;

/// Whether a subsystem failure is fatal or non-fatal.
pub use lifecycle::SubsystemCriticality;

/// Deterministic startup ordering for subsystems.
pub use lifecycle::StartupOrder;

/// Result of a startup sequence execution.
pub use lifecycle::StartupResult;

/// Execute the deterministic startup sequence.
pub use lifecycle::execute_startup;
/// Execute the startup sequence with timeout monitoring and progress feedback.
pub use lifecycle::execute_startup_with_timeout;

/// Configuration provider trait for dependency injection.
pub use config::ConfigProvider;

/// Default grace period per subsystem during shutdown (3 seconds).
pub use shutdown::DEFAULT_GRACE_PERIOD;

/// Result of a shutdown sequence execution.
pub use shutdown::ShutdownResult;

/// Execute the orderly shutdown sequence in reverse initialization order.
pub use shutdown::execute_shutdown;

/// Awaitable future that resolves when an OS shutdown signal is received.
pub use shutdown::shutdown_signal;

/// Trait that plugins implement to support hot-restart.
pub use hot_restart::HotRestartable;

/// Result of a hot-restart attempt.
pub use hot_restart::HotRestartResult;

/// Execute the hot-restart sequence for a single plugin.
pub use hot_restart::hot_restart_plugin;
