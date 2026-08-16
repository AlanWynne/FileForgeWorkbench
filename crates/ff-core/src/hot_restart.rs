//! # Hot Restart — Plugin Hot-Restart Orchestration
//!
//! This module implements the plugin hot-restart protocol that allows individual
//! plugins to be reloaded without requiring a full application restart.
//!
//! The hot-restart sequence for a single plugin is:
//! 1. Deactivate the plugin (stop receiving events)
//! 2. Shutdown the plugin (release resources)
//! 3. Initialize the new plugin (allocate resources with fresh state)
//! 4. Activate the new plugin (begin receiving events)
//!
//! Non-plugin state (documents, undo history, configuration, VFS mounts) is
//! preserved across hot-restarts. Failed loads result in the plugin being left
//! unloaded with an ERROR-level log.

use crate::error::CoreError;
use crate::event_bus::{EventBus, WorkbenchEvent};
use crate::service_registry::ServiceRegistry;

/// Trait that plugins implement to support hot-restart.
///
/// The platform core orchestrates the hot-restart sequence through these methods.
/// Each method corresponds to one phase of the restart lifecycle. Implementations
/// must be `Send + Sync` to support cross-thread orchestration.
///
/// # Sequence
///
/// The hot-restart orchestrator calls methods in this order:
/// 1. [`deactivate`](Self::deactivate) — stop processing, release active resources
/// 2. [`shutdown`](Self::shutdown) — release all resources
/// 3. [`initialize`](Self::initialize) — re-initialize with fresh code/configuration
/// 4. [`activate`](Self::activate) — resume processing
#[async_trait::async_trait]
pub trait HotRestartable: Send + Sync {
    /// The plugin's registered name.
    fn name(&self) -> &str;

    /// Deactivate the plugin (stop processing, release resources that will be reloaded).
    ///
    /// # Errors
    ///
    /// Returns `CoreError` if deactivation fails.
    async fn deactivate(&mut self) -> Result<(), CoreError>;

    /// Shut down the plugin (release all resources).
    ///
    /// # Errors
    ///
    /// Returns `CoreError` if shutdown fails.
    async fn shutdown(&mut self) -> Result<(), CoreError>;

    /// Re-initialize the plugin with fresh code/configuration.
    ///
    /// The `ServiceRegistry` is provided so the plugin can look up
    /// dependencies it needs during initialization.
    ///
    /// # Errors
    ///
    /// Returns `CoreError` if re-initialization fails.
    async fn initialize(&mut self, registry: &ServiceRegistry) -> Result<(), CoreError>;

    /// Activate the plugin (resume processing).
    ///
    /// # Errors
    ///
    /// Returns `CoreError` if activation fails.
    async fn activate(&mut self) -> Result<(), CoreError>;
}

/// Result of a hot-restart attempt.
#[derive(Debug)]
pub enum HotRestartResult {
    /// Plugin was successfully hot-restarted.
    Success {
        /// The name of the plugin that was restarted.
        plugin_name: String,
    },
    /// Hot-restart failed. Plugin is left in unloaded state.
    Failed {
        /// The name of the plugin that failed.
        plugin_name: String,
        /// The reason for the failure.
        reason: String,
    },
}

/// Execute the hot-restart sequence for a single plugin.
///
/// Sequence: deactivate → shutdown → initialize → activate
///
/// On success, dispatches `WorkbenchEvent::PluginReloaded` via the event bus.
/// On failure at any step, logs ERROR and leaves the plugin unloaded.
///
/// # State Preservation
///
/// This function ONLY touches the plugin being restarted.
/// Documents, undo history, configuration, and VFS mounts are NOT modified —
/// they are owned by other subsystems that remain running throughout.
///
/// # Arguments
///
/// * `plugin` — The plugin to hot-restart (must implement `HotRestartable`).
/// * `registry` — The service registry for plugin re-initialization.
/// * `event_bus` — The event bus for dispatching `PluginReloaded` on success.
///
/// # Returns
///
/// A `HotRestartResult` indicating whether the restart succeeded or failed.
pub async fn hot_restart_plugin(
    plugin: &mut dyn HotRestartable,
    registry: &ServiceRegistry,
    event_bus: &EventBus,
) -> HotRestartResult {
    let plugin_name = plugin.name().to_string();

    // Step 1: Deactivate
    if let Err(err) = plugin.deactivate().await {
        ff_logging::log_error!(
            "[core] hot-restart: failed to deactivate plugin '{}': {}",
            plugin_name,
            err
        );
        return HotRestartResult::Failed {
            plugin_name,
            reason: format!("deactivation failed: {err}"),
        };
    }

    // Step 2: Shutdown
    if let Err(err) = plugin.shutdown().await {
        ff_logging::log_error!(
            "[core] hot-restart: failed to shut down plugin '{}': {}",
            plugin_name,
            err
        );
        return HotRestartResult::Failed {
            plugin_name,
            reason: format!("shutdown failed: {err}"),
        };
    }

    // Step 3: Initialize (with fresh state)
    if let Err(err) = plugin.initialize(registry).await {
        ff_logging::log_error!(
            "[core] hot-restart: failed to re-initialize plugin '{}': {} — leaving unloaded",
            plugin_name,
            err
        );
        return HotRestartResult::Failed {
            plugin_name,
            reason: format!("re-initialization failed: {err}"),
        };
    }

    // Step 4: Activate
    if let Err(err) = plugin.activate().await {
        ff_logging::log_error!(
            "[core] hot-restart: failed to activate plugin '{}': {} — leaving unloaded",
            plugin_name,
            err
        );
        return HotRestartResult::Failed {
            plugin_name,
            reason: format!("activation failed: {err}"),
        };
    }

    // Success — dispatch event
    ff_logging::log_info!(
        "[core] hot-restart: plugin '{}' successfully reloaded",
        plugin_name
    );
    event_bus.dispatch(WorkbenchEvent::PluginReloaded {
        plugin_name: plugin_name.clone(),
    });

    HotRestartResult::Success { plugin_name }
}

// ─── Unit Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Tracks which lifecycle methods were called and in what order.
    #[derive(Debug, Clone, Default)]
    struct CallLog {
        calls: Vec<String>,
    }

    /// A mock plugin that records all lifecycle calls and can be configured
    /// to fail at specific steps.
    struct MockPlugin {
        plugin_name: String,
        log: Arc<Mutex<CallLog>>,
        fail_at: Option<FailAt>,
    }

    /// Which step the mock plugin should fail at.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum FailAt {
        Deactivate,
        Shutdown,
        Initialize,
        Activate,
    }

    impl MockPlugin {
        fn new(name: &str, log: Arc<Mutex<CallLog>>, fail_at: Option<FailAt>) -> Self {
            Self {
                plugin_name: name.to_string(),
                log,
                fail_at,
            }
        }
    }

    #[async_trait::async_trait]
    impl HotRestartable for MockPlugin {
        fn name(&self) -> &str {
            &self.plugin_name
        }

        async fn deactivate(&mut self) -> Result<(), CoreError> {
            self.log.lock().await.calls.push("deactivate".to_string());
            if self.fail_at == Some(FailAt::Deactivate) {
                return Err(CoreError::HotRestartFailed {
                    plugin_name: self.plugin_name.clone(),
                    reason: "mock deactivation failure".to_string(),
                });
            }
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), CoreError> {
            self.log.lock().await.calls.push("shutdown".to_string());
            if self.fail_at == Some(FailAt::Shutdown) {
                return Err(CoreError::HotRestartFailed {
                    plugin_name: self.plugin_name.clone(),
                    reason: "mock shutdown failure".to_string(),
                });
            }
            Ok(())
        }

        async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
            self.log.lock().await.calls.push("initialize".to_string());
            if self.fail_at == Some(FailAt::Initialize) {
                return Err(CoreError::HotRestartFailed {
                    plugin_name: self.plugin_name.clone(),
                    reason: "mock initialization failure".to_string(),
                });
            }
            Ok(())
        }

        async fn activate(&mut self) -> Result<(), CoreError> {
            self.log.lock().await.calls.push("activate".to_string());
            if self.fail_at == Some(FailAt::Activate) {
                return Err(CoreError::HotRestartFailed {
                    plugin_name: self.plugin_name.clone(),
                    reason: "mock activation failure".to_string(),
                });
            }
            Ok(())
        }
    }

    // ─── Test: Full successful restart cycle ────────────────────────────────

    // Validates: Requirement 8.1 — hot-restart sequence: deactivate → shutdown → initialize → activate
    #[tokio::test]
    async fn hot_restart_successful_cycle_calls_all_steps_in_order() {
        let log = Arc::new(Mutex::new(CallLog::default()));
        let mut plugin = MockPlugin::new("test-plugin", Arc::clone(&log), None);
        let registry = ServiceRegistry::new();
        let event_bus = EventBus::with_default_capacity();

        let result = hot_restart_plugin(&mut plugin, &registry, &event_bus).await;

        assert!(matches!(result, HotRestartResult::Success { .. }));
        if let HotRestartResult::Success { plugin_name } = &result {
            assert_eq!(plugin_name, "test-plugin");
        }

        let calls = log.lock().await;
        assert_eq!(
            calls.calls,
            vec!["deactivate", "shutdown", "initialize", "activate"]
        );
    }

    // ─── Test: Failure at deactivation step ─────────────────────────────────

    // Validates: Requirement 8.3 — failed hot-restart logs ERROR, leaves plugin unloaded
    #[tokio::test]
    async fn hot_restart_failure_at_deactivate_returns_failed_and_stops() {
        let log = Arc::new(Mutex::new(CallLog::default()));
        let mut plugin =
            MockPlugin::new("failing-plugin", Arc::clone(&log), Some(FailAt::Deactivate));
        let registry = ServiceRegistry::new();
        let event_bus = EventBus::with_default_capacity();

        let result = hot_restart_plugin(&mut plugin, &registry, &event_bus).await;

        assert!(matches!(result, HotRestartResult::Failed { .. }));
        if let HotRestartResult::Failed {
            plugin_name,
            reason,
        } = &result
        {
            assert_eq!(plugin_name, "failing-plugin");
            assert!(reason.contains("deactivation failed"));
        }

        // Only deactivate was called — subsequent steps were skipped
        let calls = log.lock().await;
        assert_eq!(calls.calls, vec!["deactivate"]);
    }

    // ─── Test: Failure at shutdown step ─────────────────────────────────────

    // Validates: Requirement 8.3 — failed hot-restart logs ERROR, leaves plugin unloaded
    #[tokio::test]
    async fn hot_restart_failure_at_shutdown_returns_failed_and_stops() {
        let log = Arc::new(Mutex::new(CallLog::default()));
        let mut plugin =
            MockPlugin::new("failing-plugin", Arc::clone(&log), Some(FailAt::Shutdown));
        let registry = ServiceRegistry::new();
        let event_bus = EventBus::with_default_capacity();

        let result = hot_restart_plugin(&mut plugin, &registry, &event_bus).await;

        assert!(matches!(result, HotRestartResult::Failed { .. }));
        if let HotRestartResult::Failed {
            plugin_name,
            reason,
        } = &result
        {
            assert_eq!(plugin_name, "failing-plugin");
            assert!(reason.contains("shutdown failed"));
        }

        let calls = log.lock().await;
        assert_eq!(calls.calls, vec!["deactivate", "shutdown"]);
    }

    // ─── Test: Failure at initialization step ───────────────────────────────

    // Validates: Requirement 8.3 — failed hot-restart logs ERROR, leaves plugin unloaded
    #[tokio::test]
    async fn hot_restart_failure_at_initialize_returns_failed_and_stops() {
        let log = Arc::new(Mutex::new(CallLog::default()));
        let mut plugin =
            MockPlugin::new("failing-plugin", Arc::clone(&log), Some(FailAt::Initialize));
        let registry = ServiceRegistry::new();
        let event_bus = EventBus::with_default_capacity();

        let result = hot_restart_plugin(&mut plugin, &registry, &event_bus).await;

        assert!(matches!(result, HotRestartResult::Failed { .. }));
        if let HotRestartResult::Failed {
            plugin_name,
            reason,
        } = &result
        {
            assert_eq!(plugin_name, "failing-plugin");
            assert!(reason.contains("re-initialization failed"));
        }

        let calls = log.lock().await;
        assert_eq!(calls.calls, vec!["deactivate", "shutdown", "initialize"]);
    }

    // ─── Test: Failure at activation step ───────────────────────────────────

    // Validates: Requirement 8.3 — failed hot-restart logs ERROR, leaves plugin unloaded
    #[tokio::test]
    async fn hot_restart_failure_at_activate_returns_failed_and_stops() {
        let log = Arc::new(Mutex::new(CallLog::default()));
        let mut plugin =
            MockPlugin::new("failing-plugin", Arc::clone(&log), Some(FailAt::Activate));
        let registry = ServiceRegistry::new();
        let event_bus = EventBus::with_default_capacity();

        let result = hot_restart_plugin(&mut plugin, &registry, &event_bus).await;

        assert!(matches!(result, HotRestartResult::Failed { .. }));
        if let HotRestartResult::Failed {
            plugin_name,
            reason,
        } = &result
        {
            assert_eq!(plugin_name, "failing-plugin");
            assert!(reason.contains("activation failed"));
        }

        let calls = log.lock().await;
        assert_eq!(
            calls.calls,
            vec!["deactivate", "shutdown", "initialize", "activate"]
        );
    }

    // ─── Test: PluginReloaded event dispatched on success ───────────────────

    // Validates: Requirement 8.4 — PluginReloaded event dispatched via Event_Bus after success
    #[tokio::test]
    async fn hot_restart_success_dispatches_plugin_reloaded_event() {
        let log = Arc::new(Mutex::new(CallLog::default()));
        let mut plugin = MockPlugin::new("event-plugin", Arc::clone(&log), None);
        let registry = ServiceRegistry::new();
        let event_bus = EventBus::with_default_capacity();

        // Subscribe before dispatch so we can receive the event
        let mut receiver = event_bus.subscribe();

        let result = hot_restart_plugin(&mut plugin, &registry, &event_bus).await;
        assert!(matches!(result, HotRestartResult::Success { .. }));

        // Verify the PluginReloaded event was dispatched
        let received = receiver
            .try_recv()
            .expect("should receive PluginReloaded event");
        match received.as_ref() {
            WorkbenchEvent::PluginReloaded { plugin_name } => {
                assert_eq!(plugin_name, "event-plugin");
            }
            other => panic!("Expected PluginReloaded event, got: {other:?}"),
        }
    }

    // ─── Test: No event dispatched on failure ───────────────────────────────

    // Validates: Requirement 8.4 — no event dispatched when hot-restart fails
    #[tokio::test]
    async fn hot_restart_failure_does_not_dispatch_event() {
        let log = Arc::new(Mutex::new(CallLog::default()));
        let mut plugin = MockPlugin::new("no-event", Arc::clone(&log), Some(FailAt::Initialize));
        let registry = ServiceRegistry::new();
        let event_bus = EventBus::with_default_capacity();

        let mut receiver = event_bus.subscribe();

        let result = hot_restart_plugin(&mut plugin, &registry, &event_bus).await;
        assert!(matches!(result, HotRestartResult::Failed { .. }));

        // No event should have been dispatched
        let try_recv = receiver.try_recv();
        assert!(
            try_recv.is_err(),
            "No event should be dispatched on failure"
        );
    }

    // ─── Test: State preservation — services in registry persist ────────────

    // Validates: Requirement 8.2 — documents, undo history, configuration, VFS mounts unchanged
    #[tokio::test]
    async fn hot_restart_preserves_service_registry_state() {
        // Register services in the registry to simulate subsystem state
        let mut registry = ServiceRegistry::new();

        struct DocumentService(String);
        struct ConfigService(u32);

        registry
            .register(DocumentService("my-document-state".to_string()))
            .expect("register document service");
        registry
            .register(ConfigService(42))
            .expect("register config service");

        let log = Arc::new(Mutex::new(CallLog::default()));
        let mut plugin = MockPlugin::new("stateful-plugin", Arc::clone(&log), None);
        let event_bus = EventBus::with_default_capacity();

        // Perform hot-restart
        let result = hot_restart_plugin(&mut plugin, &registry, &event_bus).await;
        assert!(matches!(result, HotRestartResult::Success { .. }));

        // Verify that registry state is completely untouched
        let doc = registry
            .get::<DocumentService>()
            .expect("document service should persist");
        assert_eq!(doc.0, "my-document-state");

        let cfg = registry
            .get::<ConfigService>()
            .expect("config service should persist");
        assert_eq!(cfg.0, 42);

        assert_eq!(registry.service_count(), 2);
    }

    // ─── Test: Plugin name is correctly reported in result ───────────────────

    // Validates: Requirement 8.1 — plugin name correctly propagated in result
    #[tokio::test]
    async fn hot_restart_result_contains_correct_plugin_name() {
        let log = Arc::new(Mutex::new(CallLog::default()));
        let mut plugin = MockPlugin::new("my-special-plugin", Arc::clone(&log), None);
        let registry = ServiceRegistry::new();
        let event_bus = EventBus::with_default_capacity();

        let result = hot_restart_plugin(&mut plugin, &registry, &event_bus).await;

        match result {
            HotRestartResult::Success { plugin_name } => {
                assert_eq!(plugin_name, "my-special-plugin");
            }
            _ => panic!("Expected Success"),
        }
    }
}
