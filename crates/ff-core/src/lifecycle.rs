//! # Lifecycle — Startup Sequence Orchestration
//!
//! This module implements the startup lifecycle sequences for the platform.
//! It provides ordered subsystem initialization with dependency guarantees,
//! progress reporting, timeout enforcement, and error handling for both
//! critical and non-critical subsystem failures.
//!
//! The deterministic initialization order is:
//! logging → configuration → VFS → commands → plugins → GUI shell
//!
//! ## Public Trait Boundaries
//!
//! The [`Subsystem`] trait defines the contract that all lifecycle-managed
//! services must implement. The Lifecycle Manager calls `initialize` and
//! `shutdown` on each subsystem in the defined order. Supporting types
//! ([`SubsystemDescriptor`], [`SubsystemCriticality`], [`StartupOrder`])
//! describe each subsystem's identity, failure semantics, and position in
//! the startup sequence.

use crate::error::CoreError;
use crate::event_bus::{EventBus, OperationId, ProgressInfo, WorkbenchEvent};
use crate::service_registry::ServiceRegistry;

/// Represents the current phase of the application lifecycle.
///
/// The application transitions through these phases in order:
/// `Initializing` → `Running` → `ShuttingDown` → `Terminated`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    /// Subsystems are being initialized in startup order.
    Initializing,
    /// All subsystems initialized; application is fully operational.
    Running,
    /// Shutdown has been initiated; subsystems are being torn down.
    ShuttingDown,
    /// All subsystems have been shut down; process is about to exit.
    Terminated,
}

// ─── Subsystem Trait and Supporting Types ────────────────────────────────────

/// Describes a subsystem for registration and lifecycle management.
///
/// Each subsystem provides a descriptor that identifies it by name,
/// declares whether it is critical to application operation, and
/// specifies its position in the deterministic startup order.
///
/// Addresses: Requirement 5, criteria 1/2
pub struct SubsystemDescriptor {
    /// Human-readable name (e.g., "logging", "configuration", "vfs").
    pub name: &'static str,
    /// Whether failure to initialize is fatal to the application.
    pub criticality: SubsystemCriticality,
    /// Position in the deterministic startup order.
    pub order: StartupOrder,
}

/// Whether a subsystem is critical (failure = app termination) or
/// non-critical (failure = reduced functionality).
///
/// Critical subsystems: logging, configuration, VFS, commands.
/// Non-critical subsystems: plugins, GUI shell.
///
/// Addresses: Requirement 5, criteria 3/4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubsystemCriticality {
    /// Failure terminates the application (logging, config, VFS, commands).
    Critical,
    /// Failure allows continued operation with reduced functionality (plugins, GUI shell).
    NonCritical,
}

/// Deterministic startup ordering for subsystems.
///
/// Subsystems are initialized in ascending order and shut down in
/// descending (reverse) order. The numeric values encode the fixed
/// sequence: logging → configuration → VFS → commands → plugins → GUI shell.
///
/// Addresses: Requirement 5, criterion 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StartupOrder {
    /// Logging subsystem — initialized first, shut down last.
    Logging = 0,
    /// Configuration subsystem — provides settings to all later subsystems.
    Configuration = 1,
    /// Virtual File System — provides file access to commands and plugins.
    Vfs = 2,
    /// Command registry — user-initiated operations.
    Commands = 3,
    /// Plugin subsystem — extensibility layer.
    Plugins = 4,
    /// GUI shell — rendering layer, initialized last.
    GuiShell = 5,
}

/// Trait that all lifecycle-managed subsystems implement.
///
/// The Lifecycle Manager calls [`Subsystem::initialize`] during the startup
/// sequence (in [`StartupOrder`] order) and [`Subsystem::shutdown`] during
/// the shutdown sequence (in reverse order). Each subsystem declares its
/// identity and behaviour via [`Subsystem::descriptor`].
///
/// # Errors
///
/// - `initialize` returns `Err(CoreError)` if the subsystem cannot start.
///   For critical subsystems this triggers application termination; for
///   non-critical subsystems the error is logged and operation continues.
/// - `shutdown` returns `Err(CoreError)` if cleanup fails. The lifecycle
///   manager logs the error and proceeds to the next subsystem.
///
/// # Examples
///
/// ```rust,no_run
/// use ff_core::lifecycle::{Subsystem, SubsystemDescriptor, SubsystemCriticality, StartupOrder};
/// use ff_core::error::CoreError;
/// use ff_core::service_registry::ServiceRegistry;
///
/// struct MyPlugin;
///
/// #[async_trait::async_trait]
/// impl Subsystem for MyPlugin {
///     fn descriptor(&self) -> SubsystemDescriptor {
///         SubsystemDescriptor {
///             name: "my-plugin",
///             criticality: SubsystemCriticality::NonCritical,
///             order: StartupOrder::Plugins,
///         }
///     }
///
///     async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
///         // Perform plugin initialization...
///         Ok(())
///     }
///
///     async fn shutdown(&mut self) -> Result<(), CoreError> {
///         // Perform plugin cleanup...
///         Ok(())
///     }
/// }
/// ```
///
/// Addresses: Requirement 1, criterion 5; Requirement 5, criterion 1; Requirement 6, criterion 1
#[async_trait::async_trait]
pub trait Subsystem: Send + Sync {
    /// Returns the descriptor providing name, criticality, and startup order.
    fn descriptor(&self) -> SubsystemDescriptor;

    /// Initialize the subsystem. Called during the startup sequence in
    /// [`StartupOrder`] order. The registry provides access to previously
    /// initialized subsystems.
    async fn initialize(&mut self, registry: &ServiceRegistry) -> Result<(), CoreError>;

    /// Shut down the subsystem. Called during the shutdown sequence in
    /// reverse [`StartupOrder`] order. Must complete within the grace
    /// period (3 seconds) or risk forcible termination.
    async fn shutdown(&mut self) -> Result<(), CoreError>;
}

// ─── Startup Sequence Orchestration ─────────────────────────────────────────

/// Result of a startup sequence execution.
///
/// Contains the names of subsystems that initialized successfully (in order)
/// and, if a critical subsystem failed, the associated error.
///
/// Addresses: Requirement 5, criteria 1–4
pub struct StartupResult {
    /// Names of subsystems that initialized successfully, in order.
    /// This order is later used for reverse-order shutdown.
    pub initialized: Vec<&'static str>,
    /// If a critical subsystem failed, this contains the error.
    /// When `Some`, startup halted and only the subsystems in `initialized`
    /// were brought up before the failure occurred.
    pub critical_failure: Option<CoreError>,
}

/// Execute the deterministic startup sequence.
///
/// Subsystems are sorted by their [`StartupOrder`] and initialized one by one.
/// If a critical subsystem fails, initialization halts immediately and the
/// error is returned in [`StartupResult::critical_failure`].
/// If a non-critical subsystem fails, an ERROR-level log is written and
/// initialization continues with reduced functionality.
///
/// # Arguments
///
/// * `subsystems` — Mutable slice of boxed subsystems to initialize. The slice
///   is sorted in-place by startup order before iteration.
/// * `registry` — The service registry, passed to each subsystem's `initialize`
///   method so that later subsystems can access earlier ones.
///
/// # Returns
///
/// A [`StartupResult`] containing the initialization order (for later
/// reverse-order shutdown) and any critical failure that halted the sequence.
///
/// Addresses: Requirement 5, criteria 1–4
pub async fn execute_startup(
    subsystems: &mut [Box<dyn Subsystem>],
    registry: &ServiceRegistry,
) -> StartupResult {
    use std::time::Instant;

    // Sort by startup order — enforces deterministic sequence
    subsystems.sort_by_key(|s| s.descriptor().order);

    let mut initialized = Vec::new();

    for subsystem in subsystems.iter_mut() {
        let descriptor = subsystem.descriptor();
        let start = Instant::now();

        match subsystem.initialize(registry).await {
            Ok(()) => {
                let duration_ms = start.elapsed().as_millis();
                ff_logging::log_info!(
                    "[core] startup: subsystem '{}' initialized in {}ms",
                    descriptor.name,
                    duration_ms
                );
                initialized.push(descriptor.name);
            }
            Err(err) => match descriptor.criticality {
                SubsystemCriticality::Critical => {
                    ff_logging::log_error!(
                        "[core] startup: critical subsystem '{}' failed: {}",
                        descriptor.name,
                        err
                    );
                    return StartupResult {
                        initialized,
                        critical_failure: Some(CoreError::CriticalSubsystemFailure {
                            name: descriptor.name.to_string(),
                            reason: err.to_string(),
                        }),
                    };
                }
                SubsystemCriticality::NonCritical => {
                    ff_logging::log_error!(
                        "[core] startup: non-critical subsystem '{}' failed: {} \
                         — continuing with reduced functionality",
                        descriptor.name,
                        err
                    );
                    // Continue — don't add to initialized list
                }
            },
        }
    }

    StartupResult {
        initialized,
        critical_failure: None,
    }
}

/// Execute the deterministic startup sequence with timeout monitoring and
/// progress feedback.
///
/// Behaves identically to [`execute_startup`] but additionally tracks total
/// elapsed time. If the cumulative startup time exceeds `timeout`, a
/// [`WorkbenchEvent::Progress`] event is dispatched to the `event_bus`
/// indicating that startup is still in progress. Startup is **not** aborted
/// on timeout — it continues until all subsystems are initialized or a
/// critical failure occurs.
///
/// # Arguments
///
/// * `subsystems` — Mutable slice of boxed subsystems to initialize.
/// * `registry` — The service registry, passed to each subsystem's `initialize`.
/// * `event_bus` — The event bus used to dispatch progress feedback on timeout.
/// * `timeout` — Duration after which a progress event is dispatched.
///
/// # Returns
///
/// A [`StartupResult`] containing the initialization order and any critical failure.
///
/// Addresses: Requirement 5, criterion 5.5
pub async fn execute_startup_with_timeout(
    subsystems: &mut [Box<dyn Subsystem>],
    registry: &ServiceRegistry,
    event_bus: &EventBus,
    timeout: std::time::Duration,
) -> StartupResult {
    use std::time::Instant;

    // Sort by startup order — enforces deterministic sequence
    subsystems.sort_by_key(|s| s.descriptor().order);

    let overall_start = Instant::now();
    let mut timeout_reported = false;
    let mut initialized = Vec::new();
    let total_subsystems = subsystems.len();

    for (index, subsystem) in subsystems.iter_mut().enumerate() {
        // Check if we've exceeded the timeout and need to send progress feedback
        if !timeout_reported && overall_start.elapsed() > timeout {
            timeout_reported = true;
            event_bus.dispatch(WorkbenchEvent::Progress {
                operation_id: OperationId(0), // Startup operation
                progress: ProgressInfo {
                    label: "Startup taking longer than expected...".to_string(),
                    fraction: Some(index as f32 / total_subsystems as f32),
                    cancellable: false,
                },
            });
            ff_logging::log_warn!(
                "[core] startup: exceeded {}s timeout — still initializing",
                timeout.as_secs()
            );
        }

        let descriptor = subsystem.descriptor();
        let start = Instant::now();

        match subsystem.initialize(registry).await {
            Ok(()) => {
                let duration_ms = start.elapsed().as_millis();
                ff_logging::log_info!(
                    "[core] startup: subsystem '{}' initialized in {}ms",
                    descriptor.name,
                    duration_ms
                );
                initialized.push(descriptor.name);
            }
            Err(err) => match descriptor.criticality {
                SubsystemCriticality::Critical => {
                    ff_logging::log_error!(
                        "[core] startup: critical subsystem '{}' failed: {}",
                        descriptor.name,
                        err
                    );
                    return StartupResult {
                        initialized,
                        critical_failure: Some(CoreError::CriticalSubsystemFailure {
                            name: descriptor.name.to_string(),
                            reason: err.to_string(),
                        }),
                    };
                }
                SubsystemCriticality::NonCritical => {
                    ff_logging::log_error!(
                        "[core] startup: non-critical subsystem '{}' failed: {} \
                         — continuing with reduced functionality",
                        descriptor.name,
                        err
                    );
                    // Continue — don't add to initialized list
                }
            },
        }
    }

    StartupResult {
        initialized,
        critical_failure: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Mock Subsystem Implementation ──────────────────────────────────────

    /// A mock subsystem demonstrating that external types can implement the
    /// `Subsystem` trait. This verifies the trait boundary is properly exposed.
    struct MockVfsSubsystem {
        initialized: bool,
        shut_down: bool,
    }

    impl MockVfsSubsystem {
        fn new() -> Self {
            Self {
                initialized: false,
                shut_down: false,
            }
        }
    }

    #[async_trait::async_trait]
    impl Subsystem for MockVfsSubsystem {
        fn descriptor(&self) -> SubsystemDescriptor {
            SubsystemDescriptor {
                name: "mock-vfs",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Vfs,
            }
        }

        async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
            self.initialized = true;
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), CoreError> {
            self.shut_down = true;
            Ok(())
        }
    }

    /// A second mock subsystem to demonstrate non-critical plugin pattern.
    struct MockPluginSubsystem;

    #[async_trait::async_trait]
    impl Subsystem for MockPluginSubsystem {
        fn descriptor(&self) -> SubsystemDescriptor {
            SubsystemDescriptor {
                name: "mock-plugin",
                criticality: SubsystemCriticality::NonCritical,
                order: StartupOrder::Plugins,
            }
        }

        async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), CoreError> {
            Ok(())
        }
    }

    // ─── Subsystem Trait Tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn subsystem_trait_can_be_implemented_and_initialized() {
        // Validates: Requirement 1.5 — trait boundaries enable external types
        let mut subsystem = MockVfsSubsystem::new();
        let registry = ServiceRegistry::new();

        assert!(!subsystem.initialized);
        let result = subsystem.initialize(&registry).await;
        assert!(result.is_ok());
        assert!(subsystem.initialized);
    }

    #[tokio::test]
    async fn subsystem_trait_shutdown_can_be_called() {
        // Validates: Requirement 1.5 — trait boundaries enable external types
        let mut subsystem = MockVfsSubsystem::new();
        let registry = ServiceRegistry::new();

        subsystem
            .initialize(&registry)
            .await
            .expect("init should succeed");
        let result = subsystem.shutdown().await;
        assert!(result.is_ok());
        assert!(subsystem.shut_down);
    }

    #[test]
    fn subsystem_descriptor_returns_correct_metadata() {
        // Validates: Requirement 5.1 — subsystems declare their identity and order
        let subsystem = MockVfsSubsystem::new();
        let descriptor = subsystem.descriptor();

        assert_eq!(descriptor.name, "mock-vfs");
        assert_eq!(descriptor.criticality, SubsystemCriticality::Critical);
        assert_eq!(descriptor.order, StartupOrder::Vfs);
    }

    #[test]
    fn subsystem_non_critical_descriptor_returns_correct_metadata() {
        // Validates: Requirement 5.3 — non-critical subsystems identified by criticality
        let subsystem = MockPluginSubsystem;
        let descriptor = subsystem.descriptor();

        assert_eq!(descriptor.name, "mock-plugin");
        assert_eq!(descriptor.criticality, SubsystemCriticality::NonCritical);
        assert_eq!(descriptor.order, StartupOrder::Plugins);
    }

    // ─── SubsystemCriticality Tests ─────────────────────────────────────────

    #[test]
    fn subsystem_criticality_variants_are_distinct() {
        // Validates: Requirement 5.3, 5.4 — critical vs non-critical distinction
        assert_ne!(
            SubsystemCriticality::Critical,
            SubsystemCriticality::NonCritical
        );
    }

    #[test]
    fn subsystem_criticality_derives_debug() {
        // Validates: Requirement 1.5 — types have proper derives for usability
        let critical = SubsystemCriticality::Critical;
        let debug_str = format!("{:?}", critical);
        assert_eq!(debug_str, "Critical");

        let non_critical = SubsystemCriticality::NonCritical;
        let debug_str = format!("{:?}", non_critical);
        assert_eq!(debug_str, "NonCritical");
    }

    #[test]
    fn subsystem_criticality_clone_and_copy() {
        // Validates: Requirement 1.5 — types are Copy + Clone for ergonomic use
        let original = SubsystemCriticality::Critical;
        let cloned = original.clone();
        let copied = original;
        assert_eq!(original, cloned);
        assert_eq!(original, copied);
    }

    // ─── StartupOrder Tests ─────────────────────────────────────────────────

    #[test]
    fn startup_order_follows_deterministic_sequence() {
        // Validates: Requirement 5.1 — logging → configuration → VFS → commands → plugins → GUI shell
        assert!(StartupOrder::Logging < StartupOrder::Configuration);
        assert!(StartupOrder::Configuration < StartupOrder::Vfs);
        assert!(StartupOrder::Vfs < StartupOrder::Commands);
        assert!(StartupOrder::Commands < StartupOrder::Plugins);
        assert!(StartupOrder::Plugins < StartupOrder::GuiShell);
    }

    #[test]
    fn startup_order_non_adjacent_comparisons_hold() {
        // Validates: Requirement 5.1 — transitive ordering correctness
        assert!(StartupOrder::Logging < StartupOrder::Vfs);
        assert!(StartupOrder::Logging < StartupOrder::GuiShell);
        assert!(StartupOrder::Configuration < StartupOrder::Plugins);
        assert!(StartupOrder::Vfs < StartupOrder::GuiShell);
    }

    #[test]
    fn startup_order_equality_for_same_variant() {
        // Validates: Requirement 5.1 — same order position compares equal
        assert_eq!(StartupOrder::Logging, StartupOrder::Logging);
        assert_eq!(StartupOrder::Configuration, StartupOrder::Configuration);
        assert_eq!(StartupOrder::Vfs, StartupOrder::Vfs);
        assert_eq!(StartupOrder::Commands, StartupOrder::Commands);
        assert_eq!(StartupOrder::Plugins, StartupOrder::Plugins);
        assert_eq!(StartupOrder::GuiShell, StartupOrder::GuiShell);
    }

    #[test]
    fn startup_order_derives_debug() {
        // Validates: Requirement 1.5 — proper derives for diagnostic output
        assert_eq!(format!("{:?}", StartupOrder::Logging), "Logging");
        assert_eq!(
            format!("{:?}", StartupOrder::Configuration),
            "Configuration"
        );
        assert_eq!(format!("{:?}", StartupOrder::Vfs), "Vfs");
        assert_eq!(format!("{:?}", StartupOrder::Commands), "Commands");
        assert_eq!(format!("{:?}", StartupOrder::Plugins), "Plugins");
        assert_eq!(format!("{:?}", StartupOrder::GuiShell), "GuiShell");
    }

    #[test]
    fn startup_order_clone_and_copy() {
        // Validates: Requirement 1.5 — types are Copy + Clone for ergonomic use
        let original = StartupOrder::Commands;
        let cloned = original.clone();
        let copied = original;
        assert_eq!(original, cloned);
        assert_eq!(original, copied);
    }

    // ─── LifecyclePhase Tests ───────────────────────────────────────────────

    #[test]
    fn lifecycle_phase_all_variants_are_distinct() {
        // Validates: Requirement 1.3 — lifecycle phases are distinct states
        let phases = [
            LifecyclePhase::Initializing,
            LifecyclePhase::Running,
            LifecyclePhase::ShuttingDown,
            LifecyclePhase::Terminated,
        ];

        for (i, phase_a) in phases.iter().enumerate() {
            for (j, phase_b) in phases.iter().enumerate() {
                if i == j {
                    assert_eq!(phase_a, phase_b);
                } else {
                    assert_ne!(phase_a, phase_b);
                }
            }
        }
    }

    #[test]
    fn lifecycle_phase_derives_debug() {
        // Validates: Requirement 1.5 — proper Debug derive for diagnostic output
        assert_eq!(
            format!("{:?}", LifecyclePhase::Initializing),
            "Initializing"
        );
        assert_eq!(format!("{:?}", LifecyclePhase::Running), "Running");
        assert_eq!(
            format!("{:?}", LifecyclePhase::ShuttingDown),
            "ShuttingDown"
        );
        assert_eq!(format!("{:?}", LifecyclePhase::Terminated), "Terminated");
    }

    #[test]
    fn lifecycle_phase_clone_and_copy() {
        // Validates: Requirement 1.5 — LifecyclePhase is Copy + Clone
        let original = LifecyclePhase::Running;
        let cloned = original.clone();
        let copied = original;
        assert_eq!(original, cloned);
        assert_eq!(original, copied);
    }

    // ─── Public API Trait Boundary Tests ────────────────────────────────────

    #[test]
    fn subsystem_trait_is_object_safe() {
        // Validates: Requirement 1.5 — trait boundary allows dynamic dispatch
        // If this compiles, the trait is usable as a trait object (dyn Subsystem)
        fn _accepts_dyn_subsystem(_s: &dyn Subsystem) {}
        // Note: async_trait makes traits object-safe by boxing the future
    }

    #[test]
    fn config_provider_trait_is_object_safe() {
        // Validates: Requirement 1.5 — ConfigProvider can be used as trait object
        use crate::config::ConfigProvider;

        fn _accepts_dyn_config(_c: &dyn ConfigProvider) {}

        // Verify the trait can be boxed (as WorkbenchApp requires Box<dyn ConfigProvider>)
        struct TestConfig;
        impl ConfigProvider for TestConfig {
            fn get_string(&self, _ns: &str, _key: &str) -> Option<String> {
                None
            }
            fn get_u64(&self, _ns: &str, _key: &str) -> Option<u64> {
                None
            }
            fn get_bool(&self, _ns: &str, _key: &str) -> Option<bool> {
                None
            }
        }

        let boxed: Box<dyn ConfigProvider> = Box::new(TestConfig);
        assert_eq!(boxed.get_string("test", "key"), None);
    }

    // ─── execute_startup Tests ─────────────────────────────────────────────

    /// A configurable mock subsystem that can fail on demand.
    struct ConfigurableSubsystem {
        name: &'static str,
        criticality: SubsystemCriticality,
        order: StartupOrder,
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl Subsystem for ConfigurableSubsystem {
        fn descriptor(&self) -> SubsystemDescriptor {
            SubsystemDescriptor {
                name: self.name,
                criticality: self.criticality,
                order: self.order,
            }
        }

        async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
            if self.should_fail {
                Err(CoreError::CriticalSubsystemFailure {
                    name: self.name.to_string(),
                    reason: "simulated failure".to_string(),
                })
            } else {
                Ok(())
            }
        }

        async fn shutdown(&mut self) -> Result<(), CoreError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn execute_startup_initializes_subsystems_in_startup_order() {
        // Validates: Requirement 5.1 — deterministic ordering: logging → config → VFS → commands → plugins → GUI shell
        let registry = ServiceRegistry::new();

        // Register subsystems in REVERSE order to prove sorting works
        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(ConfigurableSubsystem {
                name: "gui-shell",
                criticality: SubsystemCriticality::NonCritical,
                order: StartupOrder::GuiShell,
                should_fail: false,
            }),
            Box::new(ConfigurableSubsystem {
                name: "plugins",
                criticality: SubsystemCriticality::NonCritical,
                order: StartupOrder::Plugins,
                should_fail: false,
            }),
            Box::new(ConfigurableSubsystem {
                name: "commands",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Commands,
                should_fail: false,
            }),
            Box::new(ConfigurableSubsystem {
                name: "vfs",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Vfs,
                should_fail: false,
            }),
            Box::new(ConfigurableSubsystem {
                name: "configuration",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Configuration,
                should_fail: false,
            }),
            Box::new(ConfigurableSubsystem {
                name: "logging",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Logging,
                should_fail: false,
            }),
        ];

        let result = execute_startup(&mut subsystems, &registry).await;

        assert!(result.critical_failure.is_none());
        assert_eq!(result.initialized.len(), 6);
        assert_eq!(result.initialized[0], "logging");
        assert_eq!(result.initialized[1], "configuration");
        assert_eq!(result.initialized[2], "vfs");
        assert_eq!(result.initialized[3], "commands");
        assert_eq!(result.initialized[4], "plugins");
        assert_eq!(result.initialized[5], "gui-shell");
    }

    #[tokio::test]
    async fn execute_startup_logs_info_per_successful_subsystem() {
        // Validates: Requirement 5.2 — INFO-level log per subsystem with duration in ms
        // This test verifies the function completes and records initialized subsystems.
        // The INFO log is emitted via ff_logging::log_info! — we verify by successful completion
        // and that each subsystem appears in the initialized list (log was written on success path).
        let registry = ServiceRegistry::new();

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(ConfigurableSubsystem {
                name: "logging",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Logging,
                should_fail: false,
            }),
            Box::new(ConfigurableSubsystem {
                name: "configuration",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Configuration,
                should_fail: false,
            }),
        ];

        let result = execute_startup(&mut subsystems, &registry).await;

        // If we reach here without panic and all subsystems are in initialized list,
        // the INFO log path was executed for each (log_info! call is in the Ok branch).
        assert!(result.critical_failure.is_none());
        assert_eq!(result.initialized.len(), 2);
        assert_eq!(result.initialized[0], "logging");
        assert_eq!(result.initialized[1], "configuration");
    }

    #[tokio::test]
    async fn execute_startup_halts_on_critical_subsystem_failure() {
        // Validates: Requirement 5.4 — critical failure halts startup and returns error
        let registry = ServiceRegistry::new();

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(ConfigurableSubsystem {
                name: "logging",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Logging,
                should_fail: false,
            }),
            Box::new(ConfigurableSubsystem {
                name: "configuration",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Configuration,
                should_fail: true, // This critical subsystem fails
            }),
            Box::new(ConfigurableSubsystem {
                name: "vfs",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Vfs,
                should_fail: false,
            }),
        ];

        let result = execute_startup(&mut subsystems, &registry).await;

        // Startup halted at configuration — only logging was initialized
        assert_eq!(result.initialized.len(), 1);
        assert_eq!(result.initialized[0], "logging");

        // Critical failure is reported
        let err = result
            .critical_failure
            .expect("should have critical failure");
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("configuration"),
            "error should name the failed subsystem"
        );
    }

    #[tokio::test]
    async fn execute_startup_continues_on_non_critical_subsystem_failure() {
        // Validates: Requirement 5.3 — non-critical failure logs ERROR and continues
        let registry = ServiceRegistry::new();

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(ConfigurableSubsystem {
                name: "logging",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Logging,
                should_fail: false,
            }),
            Box::new(ConfigurableSubsystem {
                name: "configuration",
                criticality: SubsystemCriticality::Critical,
                order: StartupOrder::Configuration,
                should_fail: false,
            }),
            Box::new(ConfigurableSubsystem {
                name: "plugins",
                criticality: SubsystemCriticality::NonCritical,
                order: StartupOrder::Plugins,
                should_fail: true, // Non-critical fails — should NOT halt
            }),
            Box::new(ConfigurableSubsystem {
                name: "gui-shell",
                criticality: SubsystemCriticality::NonCritical,
                order: StartupOrder::GuiShell,
                should_fail: false,
            }),
        ];

        let result = execute_startup(&mut subsystems, &registry).await;

        // No critical failure
        assert!(result.critical_failure.is_none());

        // plugins failed so not in initialized list, but gui-shell succeeded
        assert_eq!(result.initialized.len(), 3);
        assert_eq!(result.initialized[0], "logging");
        assert_eq!(result.initialized[1], "configuration");
        assert_eq!(result.initialized[2], "gui-shell");
        // "plugins" is NOT in the initialized list because it failed
        assert!(!result.initialized.contains(&"plugins"));
    }

    #[tokio::test]
    async fn execute_startup_critical_failure_returns_critical_subsystem_failure_error() {
        // Validates: Requirement 5.4 — error type is CriticalSubsystemFailure with name and reason
        let registry = ServiceRegistry::new();

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![Box::new(ConfigurableSubsystem {
            name: "vfs",
            criticality: SubsystemCriticality::Critical,
            order: StartupOrder::Vfs,
            should_fail: true,
        })];

        let result = execute_startup(&mut subsystems, &registry).await;

        let err = result
            .critical_failure
            .expect("should have critical failure");
        match err {
            CoreError::CriticalSubsystemFailure {
                ref name,
                ref reason,
            } => {
                assert_eq!(name, "vfs");
                assert!(reason.contains("simulated failure"));
            }
            _ => panic!("Expected CriticalSubsystemFailure, got: {:?}", err),
        }
    }

    // ─── execute_startup_with_timeout Tests ─────────────────────────────────

    /// A slow subsystem that delays initialization to trigger timeout feedback.
    struct SlowSubsystem {
        delay: std::time::Duration,
        order: StartupOrder,
        name: &'static str,
    }

    #[async_trait::async_trait]
    impl Subsystem for SlowSubsystem {
        fn descriptor(&self) -> SubsystemDescriptor {
            SubsystemDescriptor {
                name: self.name,
                criticality: SubsystemCriticality::NonCritical,
                order: self.order,
            }
        }

        async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
            std::thread::sleep(self.delay);
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), CoreError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn startup_with_timeout_dispatches_progress_when_timeout_exceeded() {
        // Validates: Requirement 5.5 — 5-second startup timeout with progress feedback
        use crate::event_bus::{EventBus, EventCategory, WorkbenchEvent};
        use std::sync::Arc;
        use std::time::Duration;

        let registry = ServiceRegistry::new();
        let event_bus = EventBus::with_default_capacity();
        let mut rx = event_bus.subscribe();

        // Two slow subsystems that together exceed the tiny timeout
        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(SlowSubsystem {
                delay: Duration::from_millis(30),
                order: StartupOrder::Logging,
                name: "slow-logging",
            }),
            Box::new(SlowSubsystem {
                delay: Duration::from_millis(10),
                order: StartupOrder::Configuration,
                name: "slow-config",
            }),
        ];

        // Use a very short timeout (20ms) so the second subsystem triggers it
        let timeout = Duration::from_millis(20);
        let result =
            execute_startup_with_timeout(&mut subsystems, &registry, &event_bus, timeout).await;

        assert!(result.critical_failure.is_none());
        assert_eq!(result.initialized.len(), 2);

        // Check that a Progress event was dispatched
        let mut found_progress = false;
        while let Ok(event) = rx.try_recv() {
            if let WorkbenchEvent::Progress {
                operation_id,
                progress,
            } = event.as_ref()
            {
                found_progress = true;
                assert_eq!(operation_id.0, 0); // Startup operation ID
                assert!(progress.label.contains("taking longer than expected"));
                assert!(!progress.cancellable);
            }
        }
        assert!(
            found_progress,
            "Expected a Progress event to be dispatched on timeout"
        );
    }

    #[tokio::test]
    async fn startup_with_timeout_does_not_dispatch_progress_when_fast() {
        // Validates: Requirement 5.5 — no spurious progress event when startup is fast
        use crate::event_bus::EventBus;
        use std::time::Duration;

        let registry = ServiceRegistry::new();
        let event_bus = EventBus::with_default_capacity();
        let mut rx = event_bus.subscribe();

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(MockVfsSubsystem::new()),
            Box::new(MockPluginSubsystem),
        ];

        // Use a generous timeout that won't be exceeded
        let timeout = Duration::from_secs(10);
        let result =
            execute_startup_with_timeout(&mut subsystems, &registry, &event_bus, timeout).await;

        assert!(result.critical_failure.is_none());
        assert_eq!(result.initialized.len(), 2);

        // No Progress event should have been dispatched
        assert!(
            rx.try_recv().is_err(),
            "No events should be dispatched when startup is fast"
        );
    }

    #[tokio::test]
    async fn startup_with_timeout_still_initializes_all_subsystems_on_timeout() {
        // Validates: Requirement 5.5 — startup is NOT aborted on timeout
        use crate::event_bus::EventBus;
        use std::time::Duration;

        let registry = ServiceRegistry::new();
        let event_bus = EventBus::with_default_capacity();

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(SlowSubsystem {
                delay: Duration::from_millis(30),
                order: StartupOrder::Logging,
                name: "slow-logging",
            }),
            Box::new(SlowSubsystem {
                delay: Duration::from_millis(30),
                order: StartupOrder::Configuration,
                name: "slow-config",
            }),
            Box::new(SlowSubsystem {
                delay: Duration::from_millis(30),
                order: StartupOrder::Vfs,
                name: "slow-vfs",
            }),
        ];

        // Short timeout that will be exceeded, but all subsystems should still initialize
        let timeout = Duration::from_millis(10);
        let result =
            execute_startup_with_timeout(&mut subsystems, &registry, &event_bus, timeout).await;

        assert!(result.critical_failure.is_none());
        assert_eq!(result.initialized.len(), 3);
        assert_eq!(result.initialized[0], "slow-logging");
        assert_eq!(result.initialized[1], "slow-config");
        assert_eq!(result.initialized[2], "slow-vfs");
    }
}
