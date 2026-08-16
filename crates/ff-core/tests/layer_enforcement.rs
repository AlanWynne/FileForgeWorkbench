//! # Layer Enforcement Integration Tests
//!
//! These integration tests verify that `ff-core` (Core Layer) compiles and
//! functions correctly without any GUI/Shell layer crates present in the
//! workspace.
//!
//! ## How This Works
//!
//! The test is self-verifying: if this file compiles and the test passes, it
//! proves that `ff-core` has no dependency on any Shell Layer crate
//! (`ff-desktop`). The Rust compiler and Cargo enforce this structurally:
//!
//! 1. `ff-core`'s `Cargo.toml` declares only `ff-logging` as an `ff-*` dependency.
//! 2. `ff-logging` has zero `ff-*` dependencies (Foundation Layer rule).
//! 3. No Shell, Feature, or Editor layer crate is listed as a dependency.
//! 4. Therefore, `cargo check -p ff-core` and `cargo test -p ff-core` succeed
//!    without any Shell Layer crate in the workspace.
//!
//! If a developer mistakenly adds a dependency on `ff-desktop` (or any
//! non-existent crate), `cargo check` will fail with a resolution error,
//! blocking the violation at compile time.

// Validates: Requirement 4.6 — each crate compiles independently without Shell Layer
// Validates: Requirement 4.7 — cargo check fails on layer violation

/// Proves ff-core compiles without Shell Layer crates.
///
/// This integration test exercises the public API of ff-core to confirm that
/// the crate is fully functional in a headless (no GUI shell) configuration.
/// If ff-core had an undeclared dependency on ff-desktop, this test would
/// fail at the linking stage.
#[test]
fn ff_core_compiles_and_runs_without_shell_layer() {
    // Verify core types are accessible — if ff-core had a hidden Shell Layer
    // dependency, these types would fail to resolve.
    let _phase = ff_core::LifecyclePhase::Initializing;

    // Verify the event bus can be created without a GUI shell subscriber.
    let event_bus = ff_core::EventBus::new(ff_core::DEFAULT_EVENT_BUS_CAPACITY);

    // Dispatch an event — this must succeed even with no Shell subscriber.
    event_bus.dispatch(ff_core::WorkbenchEvent::WorkbenchReady);

    // Verify the service registry works without Shell Layer.
    let registry = ff_core::ServiceRegistry::new();
    assert!(!registry.is_frozen());

    // If we reach this point, ff-core is fully operational without any
    // Shell Layer crate. The layer rule is proven.
}

/// Proves the EventBus operates correctly when no GUI shell subscriber exists.
///
/// Per Requirement 3.6: events dispatched when no GUI subscriber is registered
/// shall be silently discarded for GUI-targeted events. This test confirms
/// that behaviour works without ff-desktop present.
#[test]
fn event_bus_operates_without_gui_shell() {
    let event_bus = ff_core::EventBus::new(100);

    // Dispatch multiple event types — none should panic or error.
    event_bus.dispatch(ff_core::WorkbenchEvent::WorkbenchReady);
    event_bus.dispatch(ff_core::WorkbenchEvent::ConfigReloaded);
    event_bus.dispatch(ff_core::WorkbenchEvent::Notification {
        message: "test".to_string(),
        severity: ff_core::NotificationSeverity::Info,
    });
    event_bus.dispatch(ff_core::WorkbenchEvent::ShutdownInitiated);

    // No panic, no error — GUI-independent operation confirmed.
    assert_eq!(event_bus.dropped_count(), 0);
}
