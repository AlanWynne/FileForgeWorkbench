//! Property-Based Tests for ff-core
//!
//! These tests verify critical invariants of the platform core using
//! randomized input generation via proptest.

use proptest::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;

use ff_core::error::CoreError;
use ff_core::event_bus::{
    CommandOutcome, CommandParams, DocumentId, EventBus, EventCategory, EventFilter,
    EventSubscription, NotificationSeverity, OperationId, ProgressInfo, WorkbenchEvent,
};
use ff_core::lifecycle::{
    execute_startup, StartupOrder, Subsystem, SubsystemCriticality, SubsystemDescriptor,
};
use ff_core::service_registry::ServiceRegistry;
use ff_core::shutdown::execute_shutdown;

// ═══════════════════════════════════════════════════════════════════════════════
// Property 1: Service Registry Type-Safety Invariant
// ═══════════════════════════════════════════════════════════════════════════════

// Feature: platform-core, Property 1: For any set of distinct service types
// registered in any order, `get::<T>()` SHALL return `Some` for every registered
// type and `None` for every unregistered type.

// Newtype wrappers to simulate distinct service types
struct S0(u64);
struct S1(u64);
struct S2(u64);
struct S3(u64);
struct S4(u64);
struct S5(u64);
struct S6(u64);
struct S7(u64);
struct S8(u64);
struct S9(u64);

/// **Validates: Requirements 2.2, 2.6**
///
/// Registers a random subset of 10 distinct service types in a random order,
/// then verifies that `get::<T>()` returns `Some` for registered types and
/// `None` for unregistered types.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn service_registry_type_safety_invariant(
        // Which of the 10 service slots to register (as a bitmask)
        registered_mask in 1u16..1024u16,
        // Values to store in each service
        values in prop::array::uniform10(any::<u64>()),
        // Shuffle order for registration (indices 0-9 shuffled)
        shuffle_seed in any::<u64>(),
    ) {
        let mut registry = ServiceRegistry::new();

        // Determine which indices are registered
        let mut indices_to_register: Vec<usize> = (0..10)
            .filter(|i| registered_mask & (1 << i) != 0)
            .collect();

        // Shuffle the registration order deterministically based on seed
        let len = indices_to_register.len();
        if len > 1 {
            for i in (1..len).rev() {
                let j = (shuffle_seed.wrapping_mul(i as u64 + 1).wrapping_add(7)) as usize % (i + 1);
                indices_to_register.swap(i, j);
            }
        }

        // Register services in shuffled order
        for &idx in &indices_to_register {
            match idx {
                0 => { registry.register(S0(values[0])).unwrap(); }
                1 => { registry.register(S1(values[1])).unwrap(); }
                2 => { registry.register(S2(values[2])).unwrap(); }
                3 => { registry.register(S3(values[3])).unwrap(); }
                4 => { registry.register(S4(values[4])).unwrap(); }
                5 => { registry.register(S5(values[5])).unwrap(); }
                6 => { registry.register(S6(values[6])).unwrap(); }
                7 => { registry.register(S7(values[7])).unwrap(); }
                8 => { registry.register(S8(values[8])).unwrap(); }
                9 => { registry.register(S9(values[9])).unwrap(); }
                _ => unreachable!(),
            }
        }

        let registered_set: HashSet<usize> = indices_to_register.into_iter().collect();

        // Verify: registered types return Some with correct value
        // Verify: unregistered types return None
        for idx in 0..10 {
            if registered_set.contains(&idx) {
                match idx {
                    0 => prop_assert_eq!(registry.get::<S0>().map(|s| s.0), Some(values[0])),
                    1 => prop_assert_eq!(registry.get::<S1>().map(|s| s.0), Some(values[1])),
                    2 => prop_assert_eq!(registry.get::<S2>().map(|s| s.0), Some(values[2])),
                    3 => prop_assert_eq!(registry.get::<S3>().map(|s| s.0), Some(values[3])),
                    4 => prop_assert_eq!(registry.get::<S4>().map(|s| s.0), Some(values[4])),
                    5 => prop_assert_eq!(registry.get::<S5>().map(|s| s.0), Some(values[5])),
                    6 => prop_assert_eq!(registry.get::<S6>().map(|s| s.0), Some(values[6])),
                    7 => prop_assert_eq!(registry.get::<S7>().map(|s| s.0), Some(values[7])),
                    8 => prop_assert_eq!(registry.get::<S8>().map(|s| s.0), Some(values[8])),
                    9 => prop_assert_eq!(registry.get::<S9>().map(|s| s.0), Some(values[9])),
                    _ => unreachable!(),
                }
            } else {
                match idx {
                    0 => prop_assert!(registry.get::<S0>().is_none()),
                    1 => prop_assert!(registry.get::<S1>().is_none()),
                    2 => prop_assert!(registry.get::<S2>().is_none()),
                    3 => prop_assert!(registry.get::<S3>().is_none()),
                    4 => prop_assert!(registry.get::<S4>().is_none()),
                    5 => prop_assert!(registry.get::<S5>().is_none()),
                    6 => prop_assert!(registry.get::<S6>().is_none()),
                    7 => prop_assert!(registry.get::<S7>().is_none()),
                    8 => prop_assert!(registry.get::<S8>().is_none()),
                    9 => prop_assert!(registry.get::<S9>().is_none()),
                    _ => unreachable!(),
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Property 2: Event Bus Delivery Completeness
// ═══════════════════════════════════════════════════════════════════════════════

// Feature: platform-core, Property 2: For any set of subscribers each with a
// filter and any sequence of dispatched events, every subscriber SHALL receive
// exactly the events matching its registered filter.

/// Maps a u8 index (0-4) to an EventCategory.
fn category_from_index(idx: u8) -> EventCategory {
    match idx % 5 {
        0 => EventCategory::Command,
        1 => EventCategory::Notification,
        2 => EventCategory::StateChange,
        3 => EventCategory::Progress,
        _ => EventCategory::Lifecycle,
    }
}

/// Creates a WorkbenchEvent with the given category.
fn event_with_category(cat: EventCategory) -> WorkbenchEvent {
    match cat {
        EventCategory::Command => WorkbenchEvent::CommandDispatched {
            command_id: "test".to_string(),
            params: CommandParams::default(),
        },
        EventCategory::Notification => WorkbenchEvent::Notification {
            message: "test".to_string(),
            severity: NotificationSeverity::Info,
        },
        EventCategory::StateChange => WorkbenchEvent::DocumentChanged {
            document_id: DocumentId(1),
        },
        EventCategory::Progress => WorkbenchEvent::Progress {
            operation_id: OperationId(1),
            progress: ProgressInfo {
                label: "test".to_string(),
                fraction: Some(0.5),
                cancellable: false,
            },
        },
        EventCategory::Lifecycle => WorkbenchEvent::WorkbenchReady,
    }
}

/// **Validates: Requirements 3.4, 3.5**
///
/// For any set of subscribers each with a filter and any sequence of dispatched
/// events, every subscriber SHALL receive exactly the events matching its
/// registered filter — no missed deliveries and no spurious deliveries.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn event_bus_delivery_completeness(
        // Number of subscribers (1-8)
        num_subscribers in 1usize..=8,
        // Filter bitmasks for each subscriber (5 categories, bits 0-4)
        filter_masks in prop::collection::vec(1u8..32u8, 1..=8),
        // Sequence of event categories to dispatch (10-100 events)
        event_categories in prop::collection::vec(0u8..5u8, 10..=100),
    ) {
        let bus = EventBus::with_default_capacity();
        let actual_num_subscribers = num_subscribers.min(filter_masks.len());

        // Create subscribers with their filters
        let mut subscriptions: Vec<EventSubscription> = Vec::new();
        let mut subscriber_filters: Vec<HashSet<EventCategory>> = Vec::new();

        for i in 0..actual_num_subscribers {
            let mask = filter_masks[i];
            let mut cats = Vec::new();
            let mut cat_set = HashSet::new();
            for bit in 0..5u8 {
                if mask & (1 << bit) != 0 {
                    let cat = category_from_index(bit);
                    cats.push(cat);
                    cat_set.insert(cat);
                }
            }
            let filter = EventFilter::Categories(cats);
            subscriptions.push(bus.subscribe_filtered(filter));
            subscriber_filters.push(cat_set);
        }

        // Dispatch events
        let dispatched_categories: Vec<EventCategory> = event_categories
            .iter()
            .map(|&idx| category_from_index(idx))
            .collect();

        for &cat in &dispatched_categories {
            bus.dispatch(event_with_category(cat));
        }

        // For each subscriber, verify they received exactly the matching events
        for (i, sub) in subscriptions.iter_mut().enumerate() {
            let filter_set = &subscriber_filters[i];

            // Count expected events for this subscriber
            let expected_count = dispatched_categories
                .iter()
                .filter(|cat| filter_set.contains(cat))
                .count();

            // Count received events
            let mut received_count = 0;
            while let Some(event) = sub.try_recv() {
                // Every received event must match the filter
                prop_assert!(
                    filter_set.contains(&event.category()),
                    "Subscriber {} received event with category {:?} not in filter {:?}",
                    i, event.category(), filter_set
                );
                received_count += 1;
            }

            prop_assert_eq!(
                received_count, expected_count,
                "Subscriber {} expected {} events but received {}",
                i, expected_count, received_count
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Property 3: Event Bus Overflow Monotonicity
// ═══════════════════════════════════════════════════════════════════════════════

// Feature: platform-core, Property 3: The dropped event counter on the EventBus
// is monotonically non-decreasing.

/// **Validates: Requirement 3.7**
///
/// For any sequence of dispatch operations, the dropped_count is monotonically
/// non-decreasing. We use a small capacity to trigger overflow.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn event_bus_overflow_monotonicity(
        // Number of events to dispatch (100-2000)
        num_events in 100usize..=2000,
        // Small capacity to trigger overflow
        capacity in 5usize..=50,
    ) {
        // Use a small capacity bus to trigger overflows
        let bus = EventBus::new(capacity);

        // Subscribe to create a receiver that won't drain (simulating slow consumer)
        let _sub = bus.subscribe_filtered(EventFilter::All);

        let mut prev_dropped = bus.dropped_count();

        for _ in 0..num_events {
            bus.dispatch(WorkbenchEvent::WorkbenchReady);
            let current_dropped = bus.dropped_count();

            // Monotonicity: current >= previous
            prop_assert!(
                current_dropped >= prev_dropped,
                "Dropped count decreased from {} to {}",
                prev_dropped, current_dropped
            );
            prev_dropped = current_dropped;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Property 4: Startup Sequence Ordering Determinism
// ═══════════════════════════════════════════════════════════════════════════════

// Feature: platform-core, Property 4: For any configuration context, the startup
// sequence SHALL always produce the same initialization order.

/// A simple mock subsystem for property testing startup ordering.
struct MockSubsystem {
    name: &'static str,
    criticality: SubsystemCriticality,
    order: StartupOrder,
}

#[async_trait::async_trait]
impl Subsystem for MockSubsystem {
    fn descriptor(&self) -> SubsystemDescriptor {
        SubsystemDescriptor {
            name: self.name,
            criticality: self.criticality,
            order: self.order,
        }
    }

    async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), CoreError> {
        Ok(())
    }
}

/// Maps an index to a StartupOrder variant.
fn order_from_index(idx: u8) -> StartupOrder {
    match idx % 6 {
        0 => StartupOrder::Logging,
        1 => StartupOrder::Configuration,
        2 => StartupOrder::Vfs,
        3 => StartupOrder::Commands,
        4 => StartupOrder::Plugins,
        _ => StartupOrder::GuiShell,
    }
}

/// Maps an index to a subsystem name.
fn name_from_index(idx: u8) -> &'static str {
    match idx % 6 {
        0 => "logging",
        1 => "configuration",
        2 => "vfs",
        3 => "commands",
        4 => "plugins",
        _ => "gui-shell",
    }
}

/// **Validates: Requirements 5.1, 2.4**
///
/// For any subset of subsystems provided in any input order, repeated calls to
/// execute_startup produce the same initialization order every time.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn startup_sequence_ordering_determinism(
        // Which subsystems to include (bitmask over 6 slots)
        subsystem_mask in 1u8..64u8,
        // Input order permutation seed
        shuffle_seed in any::<u64>(),
        // Number of repeated attempts
        num_attempts in 2usize..=5,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let registry = ServiceRegistry::new();

            // Determine which subsystems to include
            let mut indices: Vec<u8> = (0u8..6)
                .filter(|i| subsystem_mask & (1 << i) != 0)
                .collect();

            // Shuffle input order using seed
            let len = indices.len();
            if len > 1 {
                for i in (1..len).rev() {
                    let j = (shuffle_seed.wrapping_mul(i as u64 + 1).wrapping_add(13)) as usize % (i + 1);
                    indices.swap(i, j);
                }
            }

            // Run startup multiple times and collect results
            let mut results: Vec<Vec<&'static str>> = Vec::new();

            for _ in 0..num_attempts {
                let mut subsystems: Vec<Box<dyn Subsystem>> = indices
                    .iter()
                    .map(|&idx| -> Box<dyn Subsystem> {
                        Box::new(MockSubsystem {
                            name: name_from_index(idx),
                            criticality: SubsystemCriticality::Critical,
                            order: order_from_index(idx),
                        })
                    })
                    .collect();

                let result = execute_startup(&mut subsystems, &registry).await;
                results.push(result.initialized);
            }

            // All attempts must produce the same order
            let first = &results[0];
            for (attempt_idx, result) in results.iter().enumerate().skip(1) {
                prop_assert_eq!(
                    first, result,
                    "Startup order differed between attempt 0 and attempt {}: {:?} vs {:?}",
                    attempt_idx, first, result
                );
            }

            Ok(())
        })?;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Property 5: Shutdown Reverse-Order Invariant
// ═══════════════════════════════════════════════════════════════════════════════

// Feature: platform-core, Property 5: For any set of successfully initialized
// subsystems, the shutdown sequence SHALL visit them in the exact reverse order
// of their initialization.

/// A mock subsystem that records its shutdown order.
struct ShutdownTrackingSubsystem {
    name: &'static str,
    order: StartupOrder,
    shutdown_log: Arc<std::sync::Mutex<Vec<&'static str>>>,
}

#[async_trait::async_trait]
impl Subsystem for ShutdownTrackingSubsystem {
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
        self.shutdown_log.lock().unwrap().push(self.name);
        Ok(())
    }
}

/// **Validates: Requirement 6.1**
///
/// For any set of successfully initialized subsystems, the shutdown sequence
/// SHALL visit them in the exact reverse order of their initialization.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn shutdown_reverse_order_invariant(
        // Which subsystems to include (bitmask over 6 slots, at least 1)
        subsystem_mask in 1u8..64u8,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let registry = ServiceRegistry::new();
            let shutdown_log = Arc::new(std::sync::Mutex::new(Vec::new()));

            // Determine which subsystems to include
            let indices: Vec<u8> = (0u8..6)
                .filter(|i| subsystem_mask & (1 << i) != 0)
                .collect();

            // First, run startup to get the initialization order
            let mut startup_subsystems: Vec<Box<dyn Subsystem>> = indices
                .iter()
                .map(|&idx| -> Box<dyn Subsystem> {
                    Box::new(MockSubsystem {
                        name: name_from_index(idx),
                        criticality: SubsystemCriticality::Critical,
                        order: order_from_index(idx),
                    })
                })
                .collect();

            let startup_result = execute_startup(&mut startup_subsystems, &registry).await;
            let init_order = startup_result.initialized.clone();

            // Now create shutdown subsystems with tracking
            let mut shutdown_subsystems: Vec<Box<dyn Subsystem>> = indices
                .iter()
                .map(|&idx| -> Box<dyn Subsystem> {
                    Box::new(ShutdownTrackingSubsystem {
                        name: name_from_index(idx),
                        order: order_from_index(idx),
                        shutdown_log: Arc::clone(&shutdown_log),
                    })
                })
                .collect();

            let grace_period = std::time::Duration::from_millis(100);
            execute_shutdown(&mut shutdown_subsystems, grace_period).await;

            // Verify shutdown order is exact reverse of initialization order
            let actual_shutdown_order = shutdown_log.lock().unwrap().clone();
            let expected_shutdown_order: Vec<&str> = init_order.iter().rev().copied().collect();

            prop_assert_eq!(
                &actual_shutdown_order, &expected_shutdown_order,
                "Shutdown order {:?} is not the reverse of init order {:?}",
                actual_shutdown_order, init_order
            );

            Ok(())
        })?;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Property 6: Service Registry Freeze Immutability
// ═══════════════════════════════════════════════════════════════════════════════

// Feature: platform-core, Property 6: After the Service_Registry transitions to
// frozen state, any registration attempt SHALL fail with an error.

// Additional newtype wrappers for post-freeze registration attempts
struct P0(u64);
struct P1(u64);
struct P2(u64);
struct P3(u64);
struct P4(u64);
struct P5(u64);
struct P6(u64);
struct P7(u64);
struct P8(u64);
struct P9(u64);

/// **Validates: Requirement 2.8**
///
/// After the Service_Registry transitions to frozen state, any registration
/// attempt SHALL fail with an error. The set of registered services SHALL
/// remain unchanged for all subsequent get calls.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn service_registry_freeze_immutability(
        // Which of S0-S9 to register before freeze (bitmask, at least 1)
        pre_freeze_mask in 1u16..1024u16,
        // Which of P0-P9 to attempt registering after freeze (bitmask)
        post_freeze_mask in 1u16..1024u16,
        // Values for pre-freeze services
        pre_values in prop::array::uniform10(any::<u64>()),
        // Values for post-freeze attempts
        post_values in prop::array::uniform10(any::<u64>()),
    ) {
        let mut registry = ServiceRegistry::new();

        // Register services before freeze
        let pre_indices: Vec<usize> = (0..10)
            .filter(|i| pre_freeze_mask & (1 << i) != 0)
            .collect();

        for &idx in &pre_indices {
            match idx {
                0 => { registry.register(S0(pre_values[0])).unwrap(); }
                1 => { registry.register(S1(pre_values[1])).unwrap(); }
                2 => { registry.register(S2(pre_values[2])).unwrap(); }
                3 => { registry.register(S3(pre_values[3])).unwrap(); }
                4 => { registry.register(S4(pre_values[4])).unwrap(); }
                5 => { registry.register(S5(pre_values[5])).unwrap(); }
                6 => { registry.register(S6(pre_values[6])).unwrap(); }
                7 => { registry.register(S7(pre_values[7])).unwrap(); }
                8 => { registry.register(S8(pre_values[8])).unwrap(); }
                9 => { registry.register(S9(pre_values[9])).unwrap(); }
                _ => unreachable!(),
            }
        }

        // Snapshot service count before freeze
        let count_before_freeze = registry.service_count();

        // Freeze the registry
        registry.freeze();
        prop_assert!(registry.is_frozen());

        // Attempt post-freeze registrations — ALL must fail
        let post_indices: Vec<usize> = (0..10)
            .filter(|i| post_freeze_mask & (1 << i) != 0)
            .collect();

        for &idx in &post_indices {
            let result = match idx {
                0 => registry.register(P0(post_values[0])),
                1 => registry.register(P1(post_values[1])),
                2 => registry.register(P2(post_values[2])),
                3 => registry.register(P3(post_values[3])),
                4 => registry.register(P4(post_values[4])),
                5 => registry.register(P5(post_values[5])),
                6 => registry.register(P6(post_values[6])),
                7 => registry.register(P7(post_values[7])),
                8 => registry.register(P8(post_values[8])),
                9 => registry.register(P9(post_values[9])),
                _ => unreachable!(),
            };
            prop_assert!(
                result.is_err(),
                "Post-freeze registration of P{} should have failed but succeeded", idx
            );
            // Verify the error is specifically RegistryFrozen
            match result.unwrap_err() {
                CoreError::RegistryFrozen { .. } => {} // Expected
                other => prop_assert!(false, "Expected RegistryFrozen error, got: {:?}", other),
            }
        }

        // Verify service count unchanged after failed post-freeze registrations
        prop_assert_eq!(
            registry.service_count(), count_before_freeze,
            "Service count changed after freeze: was {}, now {}",
            count_before_freeze, registry.service_count()
        );

        // Verify all pre-freeze services are still accessible with correct values
        let pre_set: HashSet<usize> = pre_indices.into_iter().collect();
        for idx in 0..10 {
            if pre_set.contains(&idx) {
                match idx {
                    0 => prop_assert_eq!(registry.get::<S0>().map(|s| s.0), Some(pre_values[0])),
                    1 => prop_assert_eq!(registry.get::<S1>().map(|s| s.0), Some(pre_values[1])),
                    2 => prop_assert_eq!(registry.get::<S2>().map(|s| s.0), Some(pre_values[2])),
                    3 => prop_assert_eq!(registry.get::<S3>().map(|s| s.0), Some(pre_values[3])),
                    4 => prop_assert_eq!(registry.get::<S4>().map(|s| s.0), Some(pre_values[4])),
                    5 => prop_assert_eq!(registry.get::<S5>().map(|s| s.0), Some(pre_values[5])),
                    6 => prop_assert_eq!(registry.get::<S6>().map(|s| s.0), Some(pre_values[6])),
                    7 => prop_assert_eq!(registry.get::<S7>().map(|s| s.0), Some(pre_values[7])),
                    8 => prop_assert_eq!(registry.get::<S8>().map(|s| s.0), Some(pre_values[8])),
                    9 => prop_assert_eq!(registry.get::<S9>().map(|s| s.0), Some(pre_values[9])),
                    _ => unreachable!(),
                }
            } else {
                match idx {
                    0 => prop_assert!(registry.get::<S0>().is_none()),
                    1 => prop_assert!(registry.get::<S1>().is_none()),
                    2 => prop_assert!(registry.get::<S2>().is_none()),
                    3 => prop_assert!(registry.get::<S3>().is_none()),
                    4 => prop_assert!(registry.get::<S4>().is_none()),
                    5 => prop_assert!(registry.get::<S5>().is_none()),
                    6 => prop_assert!(registry.get::<S6>().is_none()),
                    7 => prop_assert!(registry.get::<S7>().is_none()),
                    8 => prop_assert!(registry.get::<S8>().is_none()),
                    9 => prop_assert!(registry.get::<S9>().is_none()),
                    _ => unreachable!(),
                }
            }
        }
    }
}
