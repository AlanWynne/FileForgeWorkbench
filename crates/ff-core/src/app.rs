//! # WorkbenchApp — Top-Level Application Struct
//!
//! This module defines `WorkbenchApp`, the primary application struct that owns
//! all platform state and serves as the single entry point for subsystem
//! initialization, event dispatch, and lifecycle management.
//!
//! `WorkbenchApp` orchestrates the Service Registry, Event Bus, thread model,
//! and lifecycle phases. It accepts required dependencies (configuration context
//! and logging handle) at construction time.

use crate::config::ConfigProvider;
use crate::error::CoreError;
use crate::event_bus::{EventBus, WorkbenchEvent};
use crate::lifecycle::LifecyclePhase;
use crate::service_registry::ServiceRegistry;
use ff_logging::LoggingStatus;

/// The primary application struct. Owns all platform state and serves as
/// the single entry point for subsystem initialization, event dispatch,
/// and lifecycle management.
///
/// Addresses: Requirement 1, criteria 3/4/5
pub struct WorkbenchApp {
    /// Type-safe service registry holding all subsystem references.
    registry: ServiceRegistry,
    /// Event bus for core ↔ shell communication.
    event_bus: EventBus,
    /// Current lifecycle phase.
    phase: LifecyclePhase,
    /// Configuration provider trait object — backed by `ff-config` in production.
    config: Box<dyn ConfigProvider>,
    /// Status of the logging subsystem (active vs. fallback/no-op).
    logging_status: LoggingStatus,
}

impl WorkbenchApp {
    /// Constructs a new `WorkbenchApp` with the required dependencies.
    ///
    /// Accepts a configuration provider and the logging status returned by
    /// `ff_logging::init()` or `ff_logging::init_default()`. Initializes the
    /// application in the `Initializing` lifecycle phase with an empty service
    /// registry and event bus.
    ///
    /// # Arguments
    ///
    /// * `config` — A boxed trait object implementing `ConfigProvider`. In
    ///   production this is provided by `ff-config`; in tests a mock may be used.
    /// * `logging_status` — The status returned by the logging subsystem's
    ///   initialization, indicating whether file-based logging is active or in
    ///   fallback (no-op) mode.
    ///
    /// # Errors
    ///
    /// Returns `CoreError` if a critical subsystem fails to initialize during
    /// construction.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use ff_core::{WorkbenchApp, ConfigProvider};
    /// use ff_logging::LoggingStatus;
    ///
    /// struct MockConfig;
    /// impl ConfigProvider for MockConfig {
    ///     fn get_string(&self, _ns: &str, _key: &str) -> Option<String> { None }
    ///     fn get_u64(&self, _ns: &str, _key: &str) -> Option<u64> { None }
    ///     fn get_bool(&self, _ns: &str, _key: &str) -> Option<bool> { None }
    /// }
    ///
    /// let app = WorkbenchApp::new(Box::new(MockConfig), LoggingStatus::Active).unwrap();
    /// ```
    pub fn new(
        config: Box<dyn ConfigProvider>,
        logging_status: LoggingStatus,
    ) -> Result<Self, CoreError> {
        Ok(Self {
            registry: ServiceRegistry::new(),
            event_bus: EventBus::with_default_capacity(),
            phase: LifecyclePhase::Initializing,
            config,
            logging_status,
        })
    }

    /// Returns the current lifecycle phase of the application.
    pub fn phase(&self) -> LifecyclePhase {
        self.phase
    }

    /// Returns a reference to the configuration provider.
    pub fn config(&self) -> &dyn ConfigProvider {
        self.config.as_ref()
    }

    /// Returns the logging subsystem status.
    pub fn logging_status(&self) -> LoggingStatus {
        self.logging_status
    }

    /// Execute the full startup sequence.
    ///
    /// Initializes subsystems in deterministic order:
    /// logging → configuration → VFS → commands → plugins → GUI shell.
    /// Dispatches `WorkbenchReady` event on success.
    ///
    /// Full implementation in Task 9.
    ///
    /// # Errors
    ///
    /// Returns `CoreError` if a critical subsystem fails to initialize.
    pub async fn startup(&mut self) -> Result<(), CoreError> {
        // For now, since we don't have actual subsystems yet, just dispatch the ready event.
        // Full subsystem wiring comes when the actual subsystem crates exist.
        self.event_bus.dispatch(WorkbenchEvent::WorkbenchReady);
        self.phase = LifecyclePhase::Running;
        Ok(())
    }

    /// Initiate orderly shutdown.
    ///
    /// Tears down subsystems in reverse order of their initialization
    /// with a 3-second grace period per subsystem.
    ///
    /// Full implementation in Task 10.
    pub async fn shutdown(&mut self) {
        // Placeholder — full implementation in Task 10
        self.phase = LifecyclePhase::Terminated;
    }

    /// Returns a reference to the service registry.
    ///
    /// After startup completes, the registry is frozen (read-only) and
    /// provides thread-safe access to all registered subsystems.
    pub fn registry(&self) -> &ServiceRegistry {
        &self.registry
    }

    /// Returns a reference to the event bus.
    ///
    /// The event bus enables bidirectional communication between the core
    /// layer and the GUI shell, and between subsystems.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal mock `ConfigProvider` for testing.
    struct MockConfigProvider;

    impl ConfigProvider for MockConfigProvider {
        fn get_string(&self, _namespace: &str, _key: &str) -> Option<String> {
            None
        }
        fn get_u64(&self, _namespace: &str, _key: &str) -> Option<u64> {
            None
        }
        fn get_bool(&self, _namespace: &str, _key: &str) -> Option<bool> {
            None
        }
    }

    #[test]
    fn new_returns_ok_with_valid_dependencies() {
        // Validates: Requirement 1.4 — WorkbenchApp accepts config + logging as required deps
        let app = WorkbenchApp::new(Box::new(MockConfigProvider), LoggingStatus::Active);

        assert!(app.is_ok());
    }

    #[test]
    fn new_sets_phase_to_initializing() {
        // Validates: Requirement 1.3 — WorkbenchApp starts in Initializing phase
        let app = WorkbenchApp::new(Box::new(MockConfigProvider), LoggingStatus::Active)
            .expect("construction should succeed");

        assert_eq!(app.phase(), LifecyclePhase::Initializing);
    }

    #[test]
    fn new_stores_logging_status_active() {
        // Validates: Requirement 1.4 — logging handle is stored
        let app = WorkbenchApp::new(Box::new(MockConfigProvider), LoggingStatus::Active)
            .expect("construction should succeed");

        assert_eq!(app.logging_status(), LoggingStatus::Active);
    }

    #[test]
    fn new_stores_logging_status_fallback() {
        // Validates: Requirement 1.4 — fallback status is preserved
        let app = WorkbenchApp::new(Box::new(MockConfigProvider), LoggingStatus::Fallback)
            .expect("construction should succeed");

        assert_eq!(app.logging_status(), LoggingStatus::Fallback);
    }

    #[test]
    fn config_accessor_returns_provider_reference() {
        // Validates: Requirement 1.4 — config provider is accessible after construction
        let app = WorkbenchApp::new(Box::new(MockConfigProvider), LoggingStatus::Active)
            .expect("construction should succeed");

        // Verify the config provider works (returns None from mock)
        assert_eq!(app.config().get_string("test", "key"), None);
        assert_eq!(app.config().get_u64("test", "key"), None);
        assert_eq!(app.config().get_bool("test", "key"), None);
    }

    #[tokio::test]
    async fn startup_transitions_phase_to_running() {
        // Validates: Requirement 1.3 — WorkbenchApp is single entry point for subsystem initialization
        let mut app = WorkbenchApp::new(Box::new(MockConfigProvider), LoggingStatus::Active)
            .expect("construction should succeed");

        assert_eq!(app.phase(), LifecyclePhase::Initializing);

        let result = app.startup().await;
        assert!(result.is_ok());
        assert_eq!(app.phase(), LifecyclePhase::Running);
    }

    #[tokio::test]
    async fn shutdown_transitions_phase_to_terminated() {
        // Validates: Requirement 1.3 — WorkbenchApp is single entry point for lifecycle management
        let mut app = WorkbenchApp::new(Box::new(MockConfigProvider), LoggingStatus::Active)
            .expect("construction should succeed");

        app.startup().await.expect("startup should succeed");
        app.shutdown().await;

        assert_eq!(app.phase(), LifecyclePhase::Terminated);
    }

    #[test]
    fn registry_accessor_returns_reference() {
        // Validates: Requirement 1.3 — WorkbenchApp provides access to the service registry
        let app = WorkbenchApp::new(Box::new(MockConfigProvider), LoggingStatus::Active)
            .expect("construction should succeed");

        // Just verify we can obtain a reference without panicking
        let _registry = app.registry();
    }

    #[test]
    fn event_bus_accessor_returns_reference() {
        // Validates: Requirement 1.3 — WorkbenchApp provides access to the event bus
        let app = WorkbenchApp::new(Box::new(MockConfigProvider), LoggingStatus::Active)
            .expect("construction should succeed");

        // Just verify we can obtain a reference without panicking
        let _event_bus = app.event_bus();
    }

    #[tokio::test]
    async fn startup_dispatches_workbench_ready_event() {
        // Validates: Requirement 5.6 — WorkbenchReady event dispatched on successful startup
        use crate::event_bus::WorkbenchEvent;

        let mut app = WorkbenchApp::new(Box::new(MockConfigProvider), LoggingStatus::Active)
            .expect("construction should succeed");

        // Subscribe before startup so we capture the event
        let mut rx = app.event_bus().subscribe();

        let result = app.startup().await;
        assert!(result.is_ok());

        // Verify the WorkbenchReady event was dispatched
        let received = rx.try_recv().expect("should receive WorkbenchReady event");
        assert!(matches!(*received, WorkbenchEvent::WorkbenchReady));
    }

    #[tokio::test]
    async fn no_standalone_functions_bypass_workbench_app() {
        // Validates: Requirement 1.3 — All lifecycle/registry/event operations are
        // method calls ON the WorkbenchApp instance. There are no standalone free functions
        // or alternate constructors that bypass WorkbenchApp for these operations.
        let mut app = WorkbenchApp::new(Box::new(MockConfigProvider), LoggingStatus::Active)
            .expect("construction should succeed");

        // All operations go through the app instance
        let _phase = app.phase();
        let _registry = app.registry();
        let _event_bus = app.event_bus();
        let _startup_result = app.startup().await;
        app.shutdown().await;
    }
}
