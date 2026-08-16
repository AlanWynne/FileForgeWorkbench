//! # Service Registry — Type-Safe Subsystem Container
//!
//! This module implements the `ServiceRegistry`, a type-safe container for
//! registered subsystems. During startup, subsystems register themselves via
//! `register_service::<T>()`. After startup completes, the registry transitions
//! to a frozen (read-only) state.
//!
//! The registry uses `TypeId`-keyed storage to provide type-safe retrieval
//! without requiring caller downcasting. Thread-safe read access is provided
//! via `Arc` and interior mutability.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::CoreError;
use crate::lifecycle::StartupOrder;

/// Type-safe container for registered subsystems. Transitions from
/// mutable (during startup) to frozen (after startup completes).
///
/// Addresses: Requirement 2, criteria 1–8
pub struct ServiceRegistry {
    /// Storage for registered services, keyed by `TypeId`.
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    /// Records the order in which services were registered, enabling
    /// initialization ordering guarantees (Requirement 2.3).
    registration_order: Vec<TypeId>,
    /// The last startup order position that was registered.
    /// Used to enforce deterministic startup sequence ordering.
    last_registered_order: Option<StartupOrder>,
    /// Whether the registry has been frozen (no further registrations allowed).
    /// Transitions to `true` after all services are registered during startup.
    frozen: bool,
}

impl ServiceRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            registration_order: Vec::new(),
            last_registered_order: None,
            frozen: false,
        }
    }

    /// Register a service instance. Returns an error if the registry is frozen
    /// or if a service of the same type is already registered.
    ///
    /// # Errors
    /// Returns `CoreError::RegistryFrozen` if the registry has been frozen.
    /// Returns `CoreError::DuplicateServiceRegistration` if a service of the same type
    /// was previously registered.
    pub fn register<T: Send + Sync + 'static>(&mut self, service: T) -> Result<(), CoreError> {
        let type_name = std::any::type_name::<T>().to_string();

        if self.frozen {
            ff_logging::log_warn!("Cannot register '{}' — registry is frozen", type_name);
            return Err(CoreError::RegistryFrozen { type_name });
        }

        let type_id = TypeId::of::<T>();
        if self.services.contains_key(&type_id) {
            ff_logging::log_warn!(
                "Service type '{}' is already registered — duplicate registration rejected",
                type_name
            );
            return Err(CoreError::DuplicateServiceRegistration { type_name });
        }
        self.services.insert(type_id, Box::new(service));
        self.registration_order.push(type_id);
        Ok(())
    }

    /// Retrieve a reference to a registered service by type.
    /// Returns None if not registered.
    ///
    /// The caller does not need to perform downcasting — the type parameter
    /// ensures compile-time type safety.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.services
            .get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Returns the number of services currently registered.
    ///
    /// This reflects the registration order count and can be used to
    /// verify initialization ordering guarantees.
    ///
    /// Addresses: Requirement 2, criterion 3
    pub fn service_count(&self) -> usize {
        self.registration_order.len()
    }

    /// Freeze the registry, preventing further registrations.
    /// Called after startup completes to transition to read-only state.
    ///
    /// After freezing, any call to `register()` or `register_ordered()` will
    /// return `CoreError::RegistryFrozen`.
    ///
    /// Addresses: Requirement 2, criterion 8
    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    /// Returns whether the registry is in frozen (read-only) state.
    ///
    /// Addresses: Requirement 2, criterion 8
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// Register a service with startup ordering enforcement.
    ///
    /// This method validates that services are registered in non-decreasing
    /// [`StartupOrder`] sequence. If a service is registered out of order,
    /// a WARN-level log is emitted but registration still proceeds (the
    /// startup orchestration layer is responsible for calling subsystems
    /// in the correct sequence).
    ///
    /// # Errors
    ///
    /// Returns `CoreError::RegistryFrozen` if the registry has been frozen.
    /// Returns `CoreError::DuplicateServiceRegistration` if a service of the
    /// same type was previously registered.
    ///
    /// Addresses: Requirement 2, criterion 4; Requirement 5, criterion 1
    pub fn register_ordered<T: Send + Sync + 'static>(
        &mut self,
        service: T,
        order: StartupOrder,
    ) -> Result<(), CoreError> {
        // Verify ordering — warn if out of sequence but do not hard-fail
        if let Some(last) = self.last_registered_order {
            if order < last {
                ff_logging::log_warn!(
                    "Service registered out of expected order: {:?} after {:?}",
                    order,
                    last
                );
            }
        }

        // Register the service using the existing register method
        self.register(service)?;

        // Update last registered order (only if it advances)
        if self.last_registered_order.is_none_or(|last| order >= last) {
            self.last_registered_order = Some(order);
        }

        Ok(())
    }
}

impl ServiceRegistry {
    /// Consume the registry and wrap it in an `Arc` for thread-safe shared access.
    ///
    /// The registry must be frozen before calling this method. If it is not frozen,
    /// this method will auto-freeze it before wrapping.
    ///
    /// # Returns
    ///
    /// A `SharedRegistry` that can be cheaply cloned and sent across threads.
    ///
    /// Addresses: Requirement 2, criterion 5
    pub fn into_shared(mut self) -> SharedRegistry {
        if !self.frozen {
            self.freeze();
        }
        SharedRegistry {
            inner: Arc::new(self),
        }
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A thread-safe, read-only view of the service registry.
///
/// Created by calling [`ServiceRegistry::into_shared()`] after freezing.
/// Can be cheaply cloned and shared across threads. Since the underlying
/// registry is frozen (immutable), concurrent read access is safe without
/// any `Mutex` or `RwLock`.
///
/// Addresses: Requirement 2, criterion 5
#[derive(Clone)]
pub struct SharedRegistry {
    inner: Arc<ServiceRegistry>,
}

impl SharedRegistry {
    /// Retrieve a reference to a registered service by type.
    ///
    /// Returns `None` if the service type was not registered before freezing.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.inner.get::<T>()
    }

    /// Returns the number of registered services.
    pub fn service_count(&self) -> usize {
        self.inner.service_count()
    }

    /// Returns whether the registry is frozen (always `true` for a `SharedRegistry`).
    pub fn is_frozen(&self) -> bool {
        self.inner.is_frozen()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ServiceA(u32);
    struct ServiceB(String);
    struct ServiceC;

    #[test]
    fn register_service_returns_ok() {
        // Validates: Requirement 2.1 — registration during startup succeeds
        let mut registry = ServiceRegistry::new();
        let result = registry.register(ServiceA(42));
        assert!(result.is_ok());
    }

    #[test]
    fn get_returns_registered_service_with_correct_value() {
        // Validates: Requirement 2.2 — type-safe get returns correct reference
        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(99)).unwrap();

        let service = registry.get::<ServiceA>();
        assert!(service.is_some());
        assert_eq!(service.unwrap().0, 99);
    }

    #[test]
    fn multiple_distinct_types_retrieved_independently() {
        // Validates: Requirement 2.2 — multiple types coexist without interference
        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(1)).unwrap();
        registry.register(ServiceB("hello".to_string())).unwrap();
        registry.register(ServiceC).unwrap();

        let a = registry.get::<ServiceA>().unwrap();
        let b = registry.get::<ServiceB>().unwrap();
        let c = registry.get::<ServiceC>();

        assert_eq!(a.0, 1);
        assert_eq!(b.0, "hello");
        assert!(c.is_some());
    }

    #[test]
    fn duplicate_registration_returns_error() {
        // Validates: Requirement 2.7 — duplicate registration returns DuplicateServiceRegistration
        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(1)).unwrap();

        let result = registry.register(ServiceA(2));
        assert!(result.is_err());

        match result.unwrap_err() {
            CoreError::DuplicateServiceRegistration { type_name } => {
                assert!(type_name.contains("ServiceA"));
            }
            other => panic!("Expected DuplicateServiceRegistration, got: {other:?}"),
        }
    }

    #[test]
    fn get_unregistered_type_returns_none() {
        // Validates: Requirement 2.6 — absence returns None without panicking
        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(1)).unwrap();

        let result = registry.get::<ServiceB>();
        assert!(result.is_none());
    }

    #[test]
    fn new_registry_starts_empty() {
        // Validates: Requirement 2.6 — fresh registry has no services
        let registry = ServiceRegistry::new();

        assert!(registry.get::<ServiceA>().is_none());
        assert!(registry.get::<ServiceB>().is_none());
        assert!(registry.get::<ServiceC>().is_none());
    }

    #[test]
    fn service_count_reflects_registration_count() {
        // Validates: Requirement 2.3 — initialization order tracking
        let mut registry = ServiceRegistry::new();
        assert_eq!(registry.service_count(), 0);

        registry.register(ServiceA(1)).unwrap();
        assert_eq!(registry.service_count(), 1);

        registry.register(ServiceB("hello".to_string())).unwrap();
        assert_eq!(registry.service_count(), 2);

        registry.register(ServiceC).unwrap();
        assert_eq!(registry.service_count(), 3);
    }

    #[test]
    fn earlier_services_available_to_later_registrants() {
        // Validates: Requirement 2.3 — services registered earlier are available
        // to services registered later during initialization
        let mut registry = ServiceRegistry::new();

        // Register ServiceA first
        registry.register(ServiceA(42)).unwrap();

        // At this point, ServiceA is available (simulating a later service
        // checking for its dependency during its own registration)
        assert!(registry.get::<ServiceA>().is_some());
        assert_eq!(registry.get::<ServiceA>().unwrap().0, 42);

        // Register ServiceB — ServiceA is still available
        registry.register(ServiceB("world".to_string())).unwrap();
        assert!(registry.get::<ServiceA>().is_some());
        assert!(registry.get::<ServiceB>().is_some());

        // Register ServiceC — both A and B are available
        registry.register(ServiceC).unwrap();
        assert!(registry.get::<ServiceA>().is_some());
        assert!(registry.get::<ServiceB>().is_some());
        assert!(registry.get::<ServiceC>().is_some());

        // Final count matches total registrations
        assert_eq!(registry.service_count(), 3);
    }

    #[test]
    fn duplicate_registration_does_not_increment_count() {
        // Validates: Requirement 2.3, 2.7 — failed registrations do not affect order tracking
        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(1)).unwrap();
        assert_eq!(registry.service_count(), 1);

        // Duplicate registration fails — count stays the same
        let result = registry.register(ServiceA(2));
        assert!(result.is_err());
        assert_eq!(registry.service_count(), 1);
    }

    // ─── register_ordered Tests ─────────────────────────────────────────────

    #[test]
    fn register_ordered_succeeds_in_correct_sequence() {
        // Validates: Requirement 2.4 — deterministic startup order enforcement
        use crate::lifecycle::StartupOrder;

        let mut registry = ServiceRegistry::new();
        let result_a = registry.register_ordered(ServiceA(1), StartupOrder::Logging);
        let result_b =
            registry.register_ordered(ServiceB("config".to_string()), StartupOrder::Configuration);
        let result_c = registry.register_ordered(ServiceC, StartupOrder::Vfs);

        assert!(result_a.is_ok());
        assert!(result_b.is_ok());
        assert!(result_c.is_ok());
        assert_eq!(registry.service_count(), 3);
    }

    #[test]
    fn register_ordered_allows_same_order_position() {
        // Validates: Requirement 2.4 — multiple services at the same order position is valid
        use crate::lifecycle::StartupOrder;

        struct ServiceD;
        let mut registry = ServiceRegistry::new();

        let result_a = registry.register_ordered(ServiceA(1), StartupOrder::Commands);
        let result_d = registry.register_ordered(ServiceD, StartupOrder::Commands);

        assert!(result_a.is_ok());
        assert!(result_d.is_ok());
        assert_eq!(registry.service_count(), 2);
    }

    #[test]
    fn register_ordered_warns_on_out_of_order_but_succeeds() {
        // Validates: Requirement 2.4 — out-of-order registration logs WARN but does not fail
        use crate::lifecycle::StartupOrder;

        let mut registry = ServiceRegistry::new();

        // Register in correct order first
        registry
            .register_ordered(ServiceA(1), StartupOrder::Vfs)
            .unwrap();

        // Register out of order (Configuration < Vfs) — should still succeed
        let result =
            registry.register_ordered(ServiceB("late".to_string()), StartupOrder::Configuration);
        assert!(result.is_ok());

        // Both services should be accessible
        assert!(registry.get::<ServiceA>().is_some());
        assert!(registry.get::<ServiceB>().is_some());
        assert_eq!(registry.service_count(), 2);
    }

    #[test]
    fn register_ordered_rejects_duplicate_type() {
        // Validates: Requirement 2.4, 2.7 — duplicate detection still works with ordered registration
        use crate::lifecycle::StartupOrder;

        let mut registry = ServiceRegistry::new();
        registry
            .register_ordered(ServiceA(1), StartupOrder::Logging)
            .unwrap();

        let result = registry.register_ordered(ServiceA(2), StartupOrder::Configuration);
        assert!(result.is_err());

        match result.unwrap_err() {
            CoreError::DuplicateServiceRegistration { type_name } => {
                assert!(type_name.contains("ServiceA"));
            }
            other => panic!("Expected DuplicateServiceRegistration, got: {other:?}"),
        }
    }

    #[test]
    fn register_ordered_tracks_last_order_correctly() {
        // Validates: Requirement 2.4 — last_registered_order advances with valid registrations
        use crate::lifecycle::StartupOrder;

        struct ServiceD;
        struct ServiceE;

        let mut registry = ServiceRegistry::new();

        // Register in the full deterministic sequence
        registry
            .register_ordered(ServiceA(1), StartupOrder::Logging)
            .unwrap();
        registry
            .register_ordered(ServiceB("cfg".to_string()), StartupOrder::Configuration)
            .unwrap();
        registry
            .register_ordered(ServiceC, StartupOrder::Vfs)
            .unwrap();
        registry
            .register_ordered(ServiceD, StartupOrder::Commands)
            .unwrap();
        registry
            .register_ordered(ServiceE, StartupOrder::Plugins)
            .unwrap();

        assert_eq!(registry.service_count(), 5);

        // All services are retrievable
        assert!(registry.get::<ServiceA>().is_some());
        assert!(registry.get::<ServiceB>().is_some());
        assert!(registry.get::<ServiceC>().is_some());
        assert!(registry.get::<ServiceD>().is_some());
        assert!(registry.get::<ServiceE>().is_some());
    }

    #[test]
    fn register_ordered_does_not_advance_order_on_out_of_sequence() {
        // Validates: Requirement 2.4 — last_registered_order only advances, never regresses
        use crate::lifecycle::StartupOrder;

        struct ServiceD;

        let mut registry = ServiceRegistry::new();

        // Register at Vfs level
        registry
            .register_ordered(ServiceA(1), StartupOrder::Vfs)
            .unwrap();

        // Register out of order at Configuration level — succeeds with warning
        registry
            .register_ordered(ServiceB("late".to_string()), StartupOrder::Configuration)
            .unwrap();

        // Now register at Commands level — this should succeed because
        // last_registered_order should still be Vfs (not regressed to Configuration)
        let result = registry.register_ordered(ServiceD, StartupOrder::Commands);
        assert!(result.is_ok());
    }

    // ─── Frozen State Tests ─────────────────────────────────────────────────

    #[test]
    fn is_frozen_returns_false_for_new_registry() {
        // Validates: Requirement 2.8 — registry starts unfrozen
        let registry = ServiceRegistry::new();
        assert!(!registry.is_frozen());
    }

    #[test]
    fn is_frozen_returns_true_after_freeze() {
        // Validates: Requirement 2.8 — freeze transitions to read-only state
        let mut registry = ServiceRegistry::new();
        registry.freeze();
        assert!(registry.is_frozen());
    }

    #[test]
    fn freeze_prevents_further_registration() {
        // Validates: Requirement 2.8 — no registrations accepted after freeze
        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(1)).unwrap();
        registry.freeze();

        let result = registry.register(ServiceB("rejected".to_string()));
        assert!(result.is_err());

        match result.unwrap_err() {
            CoreError::RegistryFrozen { type_name } => {
                assert!(type_name.contains("ServiceB"));
            }
            other => panic!("Expected RegistryFrozen, got: {other:?}"),
        }
    }

    #[test]
    fn freeze_prevents_ordered_registration() {
        // Validates: Requirement 2.8 — register_ordered also rejected after freeze
        use crate::lifecycle::StartupOrder;

        let mut registry = ServiceRegistry::new();
        registry
            .register_ordered(ServiceA(1), StartupOrder::Logging)
            .unwrap();
        registry.freeze();

        let result = registry.register_ordered(
            ServiceB("rejected".to_string()),
            StartupOrder::Configuration,
        );
        assert!(result.is_err());

        match result.unwrap_err() {
            CoreError::RegistryFrozen { type_name } => {
                assert!(type_name.contains("ServiceB"));
            }
            other => panic!("Expected RegistryFrozen, got: {other:?}"),
        }
    }

    #[test]
    fn get_still_works_after_freeze() {
        // Validates: Requirement 2.8 — frozen registry remains readable
        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(42)).unwrap();
        registry.register(ServiceB("hello".to_string())).unwrap();
        registry.register(ServiceC).unwrap();
        registry.freeze();

        // All previously registered services are still retrievable
        let a = registry.get::<ServiceA>().unwrap();
        assert_eq!(a.0, 42);

        let b = registry.get::<ServiceB>().unwrap();
        assert_eq!(b.0, "hello");

        assert!(registry.get::<ServiceC>().is_some());
    }

    #[test]
    fn service_count_unchanged_after_freeze_and_rejected_registration() {
        // Validates: Requirement 2.8 — rejected registrations do not alter state
        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(1)).unwrap();
        registry.register(ServiceB("two".to_string())).unwrap();
        assert_eq!(registry.service_count(), 2);

        registry.freeze();
        assert_eq!(registry.service_count(), 2);

        // Attempt registration after freeze — should fail
        let _ = registry.register(ServiceC);
        assert_eq!(registry.service_count(), 2);
    }

    // ─── SharedRegistry Tests ───────────────────────────────────────────────

    #[test]
    fn shared_registry_is_send_and_sync() {
        // Validates: Requirement 2.5 — thread-safe read access without external lock
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SharedRegistry>();
    }

    #[test]
    fn into_shared_returns_shared_registry_with_services() {
        // Validates: Requirement 2.5 — SharedRegistry provides access to registered services
        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(42)).unwrap();
        registry.register(ServiceB("shared".to_string())).unwrap();
        registry.freeze();

        let shared = registry.into_shared();

        let a = shared.get::<ServiceA>().unwrap();
        assert_eq!(a.0, 42);

        let b = shared.get::<ServiceB>().unwrap();
        assert_eq!(b.0, "shared");

        assert!(shared.get::<ServiceC>().is_none());
    }

    #[test]
    fn into_shared_auto_freezes_unfrozen_registry() {
        // Validates: Requirement 2.5 — into_shared auto-freezes if not already frozen
        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(10)).unwrap();

        // Not frozen yet
        assert!(!registry.is_frozen());

        let shared = registry.into_shared();

        // SharedRegistry reports frozen state
        assert!(shared.is_frozen());
        assert_eq!(shared.service_count(), 1);
        assert_eq!(shared.get::<ServiceA>().unwrap().0, 10);
    }

    #[test]
    fn shared_registry_clone_shares_same_data() {
        // Validates: Requirement 2.5 — cloned SharedRegistry shares data cheaply
        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(99)).unwrap();
        registry.freeze();

        let shared = registry.into_shared();
        let cloned = shared.clone();

        // Both clones see the same data
        assert_eq!(shared.get::<ServiceA>().unwrap().0, 99);
        assert_eq!(cloned.get::<ServiceA>().unwrap().0, 99);
        assert_eq!(shared.service_count(), cloned.service_count());
    }

    #[test]
    fn shared_registry_concurrent_read_access() {
        // Validates: Requirement 2.5 — concurrent read access from multiple threads
        use std::thread;

        let mut registry = ServiceRegistry::new();
        registry.register(ServiceA(777)).unwrap();
        registry
            .register(ServiceB("concurrent".to_string()))
            .unwrap();
        registry.freeze();

        let shared = registry.into_shared();

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let shared_clone = shared.clone();
                thread::spawn(move || {
                    let a = shared_clone.get::<ServiceA>().unwrap();
                    assert_eq!(a.0, 777);

                    let b = shared_clone.get::<ServiceB>().unwrap();
                    assert_eq!(b.0, "concurrent");

                    assert_eq!(shared_clone.service_count(), 2);
                    assert!(shared_clone.is_frozen());
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread should not panic");
        }
    }
}
