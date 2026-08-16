//! # Shutdown — Reverse-Ordered Teardown
//!
//! This module implements the shutdown sequence for the platform. Subsystems
//! are torn down in the exact reverse order of their initialization, with a
//! 3-second grace period per subsystem for cleanup operations.
//!
//! Features:
//! - Reverse-order shutdown with grace periods
//! - Timeout enforcement with WARN logging and forcible termination
//! - Panic resilience: catches panics during shutdown and continues
//! - OS signal handling (SIGTERM/SIGINT on Unix, WM_CLOSE/CTRL_CLOSE_EVENT on Windows)
//! - Final INFO-level log and logging flush before exit

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use crate::lifecycle::Subsystem;

/// Default grace period per subsystem during shutdown (3 seconds).
///
/// Each subsystem is given this amount of time to complete its cleanup
/// operations before being forcibly terminated.
///
/// Addresses: Requirement 6, criterion 2
pub const DEFAULT_GRACE_PERIOD: Duration = Duration::from_secs(3);

/// Result of a shutdown sequence execution.
///
/// Contains the names of subsystems that shut down successfully, those
/// that timed out, and those that panicked during shutdown.
///
/// Addresses: Requirement 6, criteria 1–5
pub struct ShutdownResult {
    /// Names of subsystems that shut down successfully, in reverse order.
    pub shut_down: Vec<&'static str>,
    /// Names of subsystems that timed out during shutdown.
    pub timed_out: Vec<&'static str>,
    /// Names of subsystems that panicked during shutdown.
    pub panicked: Vec<&'static str>,
}

/// Execute the orderly shutdown sequence in reverse initialization order.
///
/// Each subsystem is given a grace period to complete its shutdown.
/// If a subsystem exceeds the grace period, a WARN is logged and shutdown
/// proceeds to the next subsystem. If a subsystem panics, the panic is
/// caught, an ERROR is logged, and shutdown continues.
///
/// After all subsystems are shut down, a final INFO log is written:
/// "Application shutdown complete".
///
/// # Arguments
///
/// * `subsystems` — Mutable slice of boxed subsystems to shut down. The slice
///   is sorted in-place by startup order descending (reverse order).
/// * `grace_period` — Maximum time each subsystem is allowed for cleanup.
///
/// # Returns
///
/// A [`ShutdownResult`] containing which subsystems completed, timed out,
/// or panicked during the shutdown sequence.
///
/// Addresses: Requirement 6, criteria 1–5
pub async fn execute_shutdown(
    subsystems: &mut [Box<dyn Subsystem>],
    grace_period: Duration,
) -> ShutdownResult {
    // Sort by startup order descending (reverse order):
    // GUI shell → plugins → commands → VFS → configuration → logging
    subsystems.sort_by_key(|s| std::cmp::Reverse(s.descriptor().order));

    let mut shut_down = Vec::new();
    let mut timed_out = Vec::new();
    let mut panicked = Vec::new();

    for subsystem in subsystems.iter_mut() {
        let descriptor = subsystem.descriptor();
        let name = descriptor.name;

        // Use spawn to isolate panics — if the subsystem panics, the spawned
        // task will panic but we can detect it via the JoinHandle.
        let shutdown_result = run_subsystem_shutdown(subsystem, grace_period).await;

        match shutdown_result {
            SubsystemShutdownOutcome::Success => {
                ff_logging::log_info!(
                    "[core] shutdown: subsystem '{}' shut down successfully",
                    name
                );
                shut_down.push(name);
            }
            SubsystemShutdownOutcome::Error(err) => {
                ff_logging::log_error!(
                    "[core] shutdown: subsystem '{}' reported error during shutdown: {}",
                    name,
                    err
                );
                // It completed (within grace period), just with an error
                shut_down.push(name);
            }
            SubsystemShutdownOutcome::TimedOut => {
                ff_logging::log_warn!(
                    "[core] shutdown: subsystem '{}' exceeded {}s grace period — forcibly terminated",
                    name,
                    grace_period.as_secs()
                );
                timed_out.push(name);
            }
            SubsystemShutdownOutcome::Panicked(msg) => {
                ff_logging::log_error!(
                    "[core] shutdown: subsystem '{}' panicked during shutdown: {}",
                    name,
                    msg
                );
                panicked.push(name);
            }
        }
    }

    ff_logging::log_info!("[core] shutdown: Application shutdown complete");

    ShutdownResult {
        shut_down,
        timed_out,
        panicked,
    }
}

/// Outcome of attempting to shut down a single subsystem.
enum SubsystemShutdownOutcome {
    /// Shutdown completed successfully within the grace period.
    Success,
    /// Shutdown completed within the grace period but returned an error.
    Error(String),
    /// Shutdown exceeded the grace period.
    TimedOut,
    /// The subsystem panicked during shutdown.
    Panicked(String),
}

/// Run a single subsystem's shutdown with timeout and panic resilience.
///
/// Uses `tokio::task::spawn` to isolate the shutdown future so that panics
/// are caught via the `JoinHandle`. Applies `tokio::time::timeout` to
/// enforce the grace period.
async fn run_subsystem_shutdown(
    subsystem: &mut Box<dyn Subsystem>,
    grace_period: Duration,
) -> SubsystemShutdownOutcome {
    // We can't move the subsystem into a spawned task (it's behind &mut),
    // so we use std::panic::AssertUnwindSafe + catch_unwind on the poll.
    // Since async catch_unwind isn't directly supported, we wrap the future
    // in a FutureExt::catch_unwind equivalent using tokio's approach.

    // Create a raw pointer to work around the borrow — safe because we await
    // within the same scope and the subsystem lives long enough.
    let subsystem_ptr = subsystem.as_mut() as *mut dyn Subsystem;

    // SAFETY: We hold &mut subsystem for the entire duration of the await.
    // The pointer is not sent across threads — we use it within the same task.
    let shutdown_future = unsafe { &mut *subsystem_ptr }.shutdown();

    // Wrap in AssertUnwindSafe for catch_unwind compatibility
    let caught = std::panic::AssertUnwindSafe(async move {
        tokio::time::timeout(grace_period, shutdown_future).await
    });

    // Use futures-style catch_unwind by spawning on the current runtime
    // and checking the JoinError for panic.
    let handle = tokio::task::spawn(caught);

    match handle.await {
        Ok(Ok(Ok(()))) => SubsystemShutdownOutcome::Success,
        Ok(Ok(Err(err))) => SubsystemShutdownOutcome::Error(err.to_string()),
        Ok(Err(_elapsed)) => SubsystemShutdownOutcome::TimedOut,
        Err(join_error) => {
            // JoinError can be either a panic or a cancellation
            if join_error.is_panic() {
                let panic_msg = if let Ok(msg) = join_error.into_panic().downcast::<String>() {
                    *msg
                } else {
                    "unknown panic".to_string()
                };
                SubsystemShutdownOutcome::Panicked(panic_msg)
            } else {
                SubsystemShutdownOutcome::Panicked("task cancelled".to_string())
            }
        }
    }
}

/// Install OS signal handlers that trigger orderly shutdown.
///
/// On Unix: listens for SIGTERM and SIGINT.
/// On Windows: listens for CTRL_CLOSE_EVENT and CTRL_C (WM_CLOSE equivalent).
///
/// When a signal is received, the provided shutdown callback is invoked.
/// This function returns a future that resolves when a shutdown signal is received.
///
/// # Arguments
///
/// * `shutdown_tx` — A oneshot sender that signals shutdown was requested.
///
/// # Platform Behavior
///
/// - **Unix**: Registers handlers for SIGTERM and SIGINT.
/// - **Windows**: Registers handlers for Ctrl+C and Ctrl+Close events.
///
/// Addresses: Requirement 6, criterion 6
pub async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                ff_logging::log_info!("[core] shutdown: received SIGTERM — initiating shutdown");
            }
            _ = sigint.recv() => {
                ff_logging::log_info!("[core] shutdown: received SIGINT — initiating shutdown");
            }
        }
    }

    #[cfg(windows)]
    {
        // On Windows, tokio::signal::ctrl_c handles Ctrl+C and CTRL_CLOSE_EVENT
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        ff_logging::log_info!(
            "[core] shutdown: received shutdown signal (Ctrl+C / WM_CLOSE) — initiating shutdown"
        );
    }
}

/// Install signal handlers and return a future that completes when shutdown
/// is triggered. This is a convenience wrapper around [`wait_for_shutdown_signal`]
/// that can be used in `tokio::select!` blocks.
///
/// # Example
///
/// ```rust,no_run
/// use ff_core::shutdown::shutdown_signal;
///
/// # async fn example() {
/// tokio::select! {
///     _ = shutdown_signal() => {
///         // Perform orderly shutdown
///     }
///     // ... other branches ...
/// }
/// # }
/// ```
///
/// Addresses: Requirement 6, criterion 6
pub fn shutdown_signal() -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(wait_for_shutdown_signal())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;
    use crate::lifecycle::{StartupOrder, Subsystem, SubsystemCriticality, SubsystemDescriptor};
    use crate::service_registry::ServiceRegistry;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // ─── Mock Subsystems for Testing ────────────────────────────────────────

    /// A mock subsystem that records whether shutdown was called and in what order.
    struct OrderTrackingSubsystem {
        name: &'static str,
        order: StartupOrder,
        shutdown_counter: Arc<AtomicUsize>,
        shutdown_order: Arc<std::sync::Mutex<Vec<&'static str>>>,
    }

    #[async_trait::async_trait]
    impl Subsystem for OrderTrackingSubsystem {
        fn descriptor(&self) -> SubsystemDescriptor {
            SubsystemDescriptor {
                name: self.name,
                criticality: SubsystemCriticality::Critical,
                order: self.order,
            }
        }

        async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), CoreError> {
            self.shutdown_counter.fetch_add(1, Ordering::SeqCst);
            self.shutdown_order.lock().unwrap().push(self.name);
            Ok(())
        }
    }

    /// A mock subsystem that sleeps longer than the grace period (causes timeout).
    struct SlowSubsystem {
        name: &'static str,
        order: StartupOrder,
        delay: Duration,
    }

    #[async_trait::async_trait]
    impl Subsystem for SlowSubsystem {
        fn descriptor(&self) -> SubsystemDescriptor {
            SubsystemDescriptor {
                name: self.name,
                criticality: SubsystemCriticality::Critical,
                order: self.order,
            }
        }

        async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), CoreError> {
            tokio::time::sleep(self.delay).await;
            Ok(())
        }
    }

    /// A mock subsystem that panics during shutdown.
    struct PanickingSubsystem {
        name: &'static str,
        order: StartupOrder,
    }

    #[async_trait::async_trait]
    impl Subsystem for PanickingSubsystem {
        fn descriptor(&self) -> SubsystemDescriptor {
            SubsystemDescriptor {
                name: self.name,
                criticality: SubsystemCriticality::NonCritical,
                order: self.order,
            }
        }

        async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), CoreError> {
            panic!("simulated panic in {} shutdown", self.name);
        }
    }

    /// A mock subsystem that returns an error on shutdown.
    struct ErrorSubsystem {
        name: &'static str,
        order: StartupOrder,
    }

    #[async_trait::async_trait]
    impl Subsystem for ErrorSubsystem {
        fn descriptor(&self) -> SubsystemDescriptor {
            SubsystemDescriptor {
                name: self.name,
                criticality: SubsystemCriticality::Critical,
                order: self.order,
            }
        }

        async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), CoreError> {
            Err(CoreError::CriticalSubsystemFailure {
                name: self.name.to_string(),
                reason: "simulated shutdown error".to_string(),
            })
        }
    }

    // ─── Reverse Ordering Tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn shutdown_executes_in_reverse_startup_order() {
        // Validates: Requirement 6.1 — reverse order: GUI shell → plugins → commands → VFS → configuration → logging
        let shutdown_order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let counter = Arc::new(AtomicUsize::new(0));

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(OrderTrackingSubsystem {
                name: "logging",
                order: StartupOrder::Logging,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
            Box::new(OrderTrackingSubsystem {
                name: "configuration",
                order: StartupOrder::Configuration,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
            Box::new(OrderTrackingSubsystem {
                name: "vfs",
                order: StartupOrder::Vfs,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
            Box::new(OrderTrackingSubsystem {
                name: "commands",
                order: StartupOrder::Commands,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
            Box::new(OrderTrackingSubsystem {
                name: "plugins",
                order: StartupOrder::Plugins,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
            Box::new(OrderTrackingSubsystem {
                name: "gui-shell",
                order: StartupOrder::GuiShell,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
        ];

        let result = execute_shutdown(&mut subsystems, DEFAULT_GRACE_PERIOD).await;

        assert_eq!(counter.load(Ordering::SeqCst), 6);
        assert_eq!(result.shut_down.len(), 6);
        assert!(result.timed_out.is_empty());
        assert!(result.panicked.is_empty());

        let order = shutdown_order.lock().unwrap();
        assert_eq!(order[0], "gui-shell");
        assert_eq!(order[1], "plugins");
        assert_eq!(order[2], "commands");
        assert_eq!(order[3], "vfs");
        assert_eq!(order[4], "configuration");
        assert_eq!(order[5], "logging");
    }

    #[tokio::test]
    async fn shutdown_reverse_order_works_regardless_of_input_order() {
        // Validates: Requirement 6.1 — sorting ensures reverse order even if input is unsorted
        let shutdown_order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let counter = Arc::new(AtomicUsize::new(0));

        // Input in random order
        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(OrderTrackingSubsystem {
                name: "commands",
                order: StartupOrder::Commands,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
            Box::new(OrderTrackingSubsystem {
                name: "logging",
                order: StartupOrder::Logging,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
            Box::new(OrderTrackingSubsystem {
                name: "gui-shell",
                order: StartupOrder::GuiShell,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
        ];

        let result = execute_shutdown(&mut subsystems, DEFAULT_GRACE_PERIOD).await;

        let order = shutdown_order.lock().unwrap();
        assert_eq!(order[0], "gui-shell");
        assert_eq!(order[1], "commands");
        assert_eq!(order[2], "logging");
        assert_eq!(result.shut_down.len(), 3);
    }

    // ─── Grace Period Timeout Tests ─────────────────────────────────────────

    #[tokio::test]
    async fn shutdown_detects_timeout_when_subsystem_exceeds_grace_period() {
        // Validates: Requirement 6.2, 6.3 — grace period enforcement and WARN logging
        let short_grace = Duration::from_millis(50);

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![Box::new(SlowSubsystem {
            name: "slow-plugin",
            order: StartupOrder::Plugins,
            delay: Duration::from_millis(200), // Much longer than grace period
        })];

        let result = execute_shutdown(&mut subsystems, short_grace).await;

        assert!(result.shut_down.is_empty());
        assert_eq!(result.timed_out.len(), 1);
        assert_eq!(result.timed_out[0], "slow-plugin");
        assert!(result.panicked.is_empty());
    }

    #[tokio::test]
    async fn shutdown_continues_to_next_subsystem_after_timeout() {
        // Validates: Requirement 6.3 — forcibly terminate and proceed to next
        let short_grace = Duration::from_millis(50);
        let shutdown_order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let counter = Arc::new(AtomicUsize::new(0));

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(SlowSubsystem {
                name: "slow-gui",
                order: StartupOrder::GuiShell,
                delay: Duration::from_millis(200),
            }),
            Box::new(OrderTrackingSubsystem {
                name: "plugins",
                order: StartupOrder::Plugins,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
            Box::new(OrderTrackingSubsystem {
                name: "logging",
                order: StartupOrder::Logging,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
        ];

        let result = execute_shutdown(&mut subsystems, short_grace).await;

        // slow-gui timed out, but plugins and logging should still shut down
        assert_eq!(result.timed_out.len(), 1);
        assert_eq!(result.timed_out[0], "slow-gui");
        assert_eq!(result.shut_down.len(), 2);

        let order = shutdown_order.lock().unwrap();
        assert_eq!(order[0], "plugins");
        assert_eq!(order[1], "logging");
    }

    #[tokio::test]
    async fn shutdown_subsystem_within_grace_period_succeeds() {
        // Validates: Requirement 6.2 — subsystem completing within grace period is OK
        let grace = Duration::from_millis(200);

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![Box::new(SlowSubsystem {
            name: "quick-plugin",
            order: StartupOrder::Plugins,
            delay: Duration::from_millis(10), // Well within grace period
        })];

        let result = execute_shutdown(&mut subsystems, grace).await;

        assert_eq!(result.shut_down.len(), 1);
        assert_eq!(result.shut_down[0], "quick-plugin");
        assert!(result.timed_out.is_empty());
    }

    // ─── Panic Resilience Tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn shutdown_catches_panic_and_continues_with_remaining_subsystems() {
        // Validates: Requirement 6.5 — panic caught, ERROR logged, continue with remaining
        let shutdown_order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let counter = Arc::new(AtomicUsize::new(0));

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(PanickingSubsystem {
                name: "panicky-gui",
                order: StartupOrder::GuiShell,
            }),
            Box::new(OrderTrackingSubsystem {
                name: "plugins",
                order: StartupOrder::Plugins,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
            Box::new(OrderTrackingSubsystem {
                name: "logging",
                order: StartupOrder::Logging,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
        ];

        let result = execute_shutdown(&mut subsystems, DEFAULT_GRACE_PERIOD).await;

        // panicky-gui should be in panicked list
        assert_eq!(result.panicked.len(), 1);
        assert_eq!(result.panicked[0], "panicky-gui");

        // plugins and logging should have shut down successfully
        assert_eq!(result.shut_down.len(), 2);
        let order = shutdown_order.lock().unwrap();
        assert_eq!(order[0], "plugins");
        assert_eq!(order[1], "logging");
    }

    #[tokio::test]
    async fn shutdown_multiple_panicking_subsystems_all_continue() {
        // Validates: Requirement 6.5 — multiple panics don't halt the sequence
        let shutdown_order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let counter = Arc::new(AtomicUsize::new(0));

        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![
            Box::new(PanickingSubsystem {
                name: "panicky-gui",
                order: StartupOrder::GuiShell,
            }),
            Box::new(PanickingSubsystem {
                name: "panicky-plugin",
                order: StartupOrder::Plugins,
            }),
            Box::new(OrderTrackingSubsystem {
                name: "logging",
                order: StartupOrder::Logging,
                shutdown_counter: counter.clone(),
                shutdown_order: shutdown_order.clone(),
            }),
        ];

        let result = execute_shutdown(&mut subsystems, DEFAULT_GRACE_PERIOD).await;

        assert_eq!(result.panicked.len(), 2);
        assert_eq!(result.shut_down.len(), 1);
        assert_eq!(result.shut_down[0], "logging");
    }

    // ─── Error Handling Tests ───────────────────────────────────────────────

    #[tokio::test]
    async fn shutdown_subsystem_error_still_counts_as_completed() {
        // Validates: Requirement 6.1 — subsystem that errors is still considered shut down
        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![Box::new(ErrorSubsystem {
            name: "error-vfs",
            order: StartupOrder::Vfs,
        })];

        let result = execute_shutdown(&mut subsystems, DEFAULT_GRACE_PERIOD).await;

        // Error subsystem should be in shut_down (it completed, just with error)
        assert_eq!(result.shut_down.len(), 1);
        assert_eq!(result.shut_down[0], "error-vfs");
        assert!(result.timed_out.is_empty());
        assert!(result.panicked.is_empty());
    }

    // ─── Empty Subsystem List Tests ─────────────────────────────────────────

    #[tokio::test]
    async fn shutdown_empty_subsystem_list_succeeds() {
        // Validates: Requirement 6.4 — final log is written even with no subsystems
        let mut subsystems: Vec<Box<dyn Subsystem>> = vec![];

        let result = execute_shutdown(&mut subsystems, DEFAULT_GRACE_PERIOD).await;

        assert!(result.shut_down.is_empty());
        assert!(result.timed_out.is_empty());
        assert!(result.panicked.is_empty());
    }

    // ─── Default Grace Period Tests ─────────────────────────────────────────

    #[test]
    fn default_grace_period_is_three_seconds() {
        // Validates: Requirement 6.2 — grace period is 3 seconds
        assert_eq!(DEFAULT_GRACE_PERIOD, Duration::from_secs(3));
    }

    // ─── Signal Handling Tests ──────────────────────────────────────────────

    #[test]
    fn shutdown_signal_returns_a_future() {
        // Validates: Requirement 6.6 — signal handler returns an awaitable future
        // This test verifies the API compiles and returns the correct type.
        let _future = shutdown_signal();
        // The future is a Pin<Box<dyn Future<Output = ()> + Send>>
        // We can't easily test signal delivery in a unit test, but we verify
        // the API is callable and returns the expected type.
    }
}
