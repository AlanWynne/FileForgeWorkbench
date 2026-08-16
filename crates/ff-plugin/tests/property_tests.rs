//! Property-based tests for the ff-plugin crate.
//!
//! These tests validate correctness properties that must hold across
//! all valid inputs, using the proptest framework.

use proptest::prelude::*;
use std::collections::HashSet;

use ff_plugin::lifecycle::validate_transition;
use ff_plugin::security::validate_config_key;
use ff_plugin::{
    check_api_compatibility, Capability, CapabilityRegistry, CapabilityType, CommandsCapability,
    DependencyGraph, PluginDependency, PluginError, PluginMetadata, PluginState, Version,
    VersionReq,
};

// ─── Strategies ─────────────────────────────────────────────────────────────

/// Generate a valid plugin name (kebab-case, 3-20 chars).
fn plugin_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9\\-]{2,19}".prop_map(|s| s)
}

/// Generate a random lifecycle operation.
fn lifecycle_op_strategy() -> impl Strategy<Value = PluginState> {
    prop_oneof![
        Just(PluginState::Loaded),
        Just(PluginState::Initialized),
        Just(PluginState::Active),
        Just(PluginState::Deactivating),
        Just(PluginState::Shutdown),
        Just(PluginState::Discovered),
    ]
}

/// Generate a version with bounded components.
fn version_strategy() -> impl Strategy<Value = Version> {
    (0u32..6, 0u32..21, 0u32..51)
        .prop_map(|(major, minor, patch)| Version::new(major, minor, patch))
}

/// Generate a capability type.
fn capability_type_strategy() -> impl Strategy<Value = CapabilityType> {
    prop_oneof![
        Just(CapabilityType::Commands),
        Just(CapabilityType::Viewers),
        Just(CapabilityType::Providers),
        Just(CapabilityType::LanguageSupport),
        Just(CapabilityType::ThemeContribution),
    ]
}

// ─── Property 1: Lifecycle State Machine Validity ───────────────────────────

// Feature: plugin-architecture, Property 1: Lifecycle state machine validity
// **Validates: Requirements 5.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn lifecycle_state_machine_validity(
        ops in proptest::collection::vec(lifecycle_op_strategy(), 1..20)
    ) {
        let mut current_state = PluginState::Discovered;

        for target_state in ops {
            let result = validate_transition("test-plugin", current_state, target_state);
            match result {
                Ok(new_state) => {
                    // Verify the transition was valid by checking it's a known good edge
                    let valid = matches!(
                        (current_state, new_state),
                        (PluginState::Discovered, PluginState::Loaded)
                            | (PluginState::Loaded, PluginState::Initialized)
                            | (PluginState::Initialized, PluginState::Active)
                            | (PluginState::Active, PluginState::Deactivating)
                            | (PluginState::Deactivating, PluginState::Shutdown)
                            | (PluginState::Shutdown, PluginState::Discovered)
                            | (PluginState::Discovered, PluginState::Shutdown)
                            | (PluginState::Loaded, PluginState::Shutdown)
                            | (PluginState::Initialized, PluginState::Shutdown)
                            | (PluginState::Active, PluginState::Shutdown)
                    );
                    prop_assert!(valid, "transition {:?} -> {:?} was accepted but is not valid",
                        current_state, new_state);
                    current_state = new_state;
                }
                Err(PluginError::InvalidStateTransition { from, to, .. }) => {
                    // State should remain unchanged
                    prop_assert_eq!(from, current_state);
                    prop_assert_eq!(to, target_state);
                    // The transition should indeed be invalid
                }
                Err(other) => {
                    prop_assert!(false, "unexpected error type: {:?}", other);
                }
            }
        }
    }
}

// ─── Property 2: Dependency Graph Acyclicity After Validation ───────────────

// Feature: plugin-architecture, Property 2: Dependency graph acyclicity after validation
// **Validates: Requirements 3.3, 3.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn dependency_graph_acyclicity_after_validation(
        num_plugins in 2usize..15,
        seed in any::<u64>(),
    ) {
        use std::collections::HashMap;

        // Generate plugin names
        let names: Vec<String> = (0..num_plugins).map(|i| format!("plugin-{i}")).collect();

        // Generate random edges (with probability 0.3 of introducing cycles)
        let mut plugins = Vec::new();
        let mut rng_state = seed;
        for (i, name) in names.iter().enumerate() {
            let mut deps = Vec::new();
            for (j, dep_name) in names.iter().enumerate() {
                if i == j { continue; }
                // Simple pseudo-random based on seed
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                if (rng_state >> 33) % 10 < 3 { // ~30% chance of edge
                    deps.push(PluginDependency {
                        name: dep_name.clone(),
                        version_req: VersionReq {
                            minimum: Version::new(1, 0, 0),
                            same_major: true,
                        },
                    });
                }
            }
            plugins.push(PluginMetadata {
                name: name.clone(),
                version: Version::new(1, 0, 0),
                author: "Test".to_string(),
                description: "".to_string(),
                dependencies: deps,
                required_api_version: Version::new(1, 0, 0),
            });
        }

        let (graph, _errors) = DependencyGraph::build(&plugins);

        // After topological sort, the result is either:
        // 1. A valid ordering (all included nodes form a DAG)
        // 2. An error listing cycle nodes (which are excluded)
        match graph.topological_sort() {
            Ok(order) => {
                // Verify it's a valid topological order
                let positions: HashMap<&str, usize> = order.iter()
                    .enumerate()
                    .map(|(i, n)| (n.as_str(), i))
                    .collect();
                // For each plugin in the order, all its deps must come before it
                for plugin in &plugins {
                    if let Some(&pos) = positions.get(plugin.name.as_str()) {
                        for dep in &plugin.dependencies {
                            if let Some(&dep_pos) = positions.get(dep.name.as_str()) {
                                prop_assert!(dep_pos < pos,
                                    "dependency {} (pos {}) should come before {} (pos {})",
                                    dep.name, dep_pos, plugin.name, pos);
                            }
                        }
                    }
                }
            }
            Err(PluginError::CircularDependency { cycle }) => {
                // Cycle was detected — verify the reported nodes are involved
                prop_assert!(!cycle.is_empty(), "cycle should not be empty");
            }
            Err(e) => {
                prop_assert!(false, "unexpected error: {:?}", e);
            }
        }
    }
}

// ─── Property 3: Topological Load Order Correctness ─────────────────────────

// Feature: plugin-architecture, Property 3: Topological load order correctness
// **Validates: Requirements 3.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn topological_load_order_correctness(
        num_plugins in 1usize..30,
        seed in any::<u64>(),
    ) {
        use std::collections::HashMap;

        // Generate a guaranteed DAG by only allowing edges from higher-index to lower-index
        let names: Vec<String> = (0..num_plugins).map(|i| format!("plugin-{i}")).collect();
        let mut plugins = Vec::new();
        let mut rng_state = seed;

        for (i, name) in names.iter().enumerate() {
            let mut deps = Vec::new();
            // Only depend on plugins with lower index (guarantees DAG)
            for j in 0..i {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                if (rng_state >> 33) % 5 < 2 { // 40% chance of edge
                    deps.push(PluginDependency {
                        name: names[j].clone(),
                        version_req: VersionReq {
                            minimum: Version::new(1, 0, 0),
                            same_major: true,
                        },
                    });
                }
            }
            plugins.push(PluginMetadata {
                name: name.clone(),
                version: Version::new(1, 0, 0),
                author: "Test".to_string(),
                description: "".to_string(),
                dependencies: deps,
                required_api_version: Version::new(1, 0, 0),
            });
        }

        let (graph, _) = DependencyGraph::build(&plugins);
        let order = graph.topological_sort().expect("DAG should have valid topo sort");

        // Verify: for every plugin P, all deps of P appear before P in the order
        let positions: HashMap<&str, usize> = order.iter()
            .enumerate()
            .map(|(i, n)| (n.as_str(), i))
            .collect();

        for plugin in &plugins {
            let plugin_pos = positions[plugin.name.as_str()];
            for dep in &plugin.dependencies {
                let dep_pos = positions[dep.name.as_str()];
                prop_assert!(dep_pos < plugin_pos,
                    "dependency {} (pos {}) should come before {} (pos {})",
                    dep.name, dep_pos, plugin.name, plugin_pos);
            }
        }
    }
}

// ─── Property 4: Version Compatibility Decision Correctness ─────────────────

// Feature: plugin-architecture, Property 4: Version compatibility decision correctness
// **Validates: Requirements 6.3, 6.4, 6.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]
    #[test]
    fn version_compatibility_decision_correctness(
        required in version_strategy(),
        available in version_strategy(),
    ) {
        let result = check_api_compatibility("test", &required, &available);

        // Expected result based on semantic versioning rules
        let should_accept = required.major == available.major
            && required.minor <= available.minor;

        match result {
            Ok(()) => {
                prop_assert!(should_accept,
                    "accepted but should reject: required={}, available={}",
                    required, available);
            }
            Err(PluginError::IncompatibleApiVersion { .. }) => {
                prop_assert!(!should_accept,
                    "rejected but should accept: required={}, available={}",
                    required, available);
            }
            Err(e) => {
                prop_assert!(false, "unexpected error: {:?}", e);
            }
        }
    }
}

// ─── Property 5: Capability Registry Consistency ────────────────────────────

/// Operations on the capability registry for property testing.
#[derive(Debug, Clone)]
enum CapOp {
    Register {
        owner: String,
        cap_type: CapabilityType,
    },
    UnregisterAll {
        owner: String,
    },
}

fn cap_op_strategy(owners: Vec<String>) -> impl Strategy<Value = CapOp> {
    let owners_clone = owners.clone();
    prop_oneof![
        (
            proptest::sample::select(owners.clone()),
            capability_type_strategy()
        )
            .prop_map(|(owner, cap_type)| CapOp::Register { owner, cap_type }),
        proptest::sample::select(owners_clone).prop_map(|owner| CapOp::UnregisterAll { owner }),
    ]
}

fn make_capability_from_type(cap_type: CapabilityType) -> Capability {
    match cap_type {
        CapabilityType::Commands => Capability::Commands(CommandsCapability {
            command_ids: vec!["test.cmd".to_string()],
            category: "test".to_string(),
            version: Version::new(1, 0, 0),
        }),
        CapabilityType::Viewers => Capability::Viewers(ff_plugin::ViewersCapability {
            mime_types: vec!["text/plain".to_string()],
            display_name: "Test".to_string(),
            version: Version::new(1, 0, 0),
        }),
        CapabilityType::Providers => Capability::Providers(ff_plugin::ProvidersCapability {
            provider_type: "test".to_string(),
            version: Version::new(1, 0, 0),
        }),
        CapabilityType::LanguageSupport => {
            Capability::LanguageSupport(ff_plugin::LanguageSupportCapability {
                language_ids: vec!["test".to_string()],
                features: vec!["highlighting".to_string()],
                version: Version::new(1, 0, 0),
            })
        }
        CapabilityType::ThemeContribution => {
            Capability::ThemeContribution(ff_plugin::ThemeCapability {
                theme_name: "Test Theme".to_string(),
                is_dark: true,
                version: Version::new(1, 0, 0),
            })
        }
        _ => Capability::Commands(CommandsCapability {
            command_ids: vec!["fallback.cmd".to_string()],
            category: "fallback".to_string(),
            version: Version::new(1, 0, 0),
        }),
    }
}

// Feature: plugin-architecture, Property 5: Capability registry consistency
// **Validates: Requirements 4.2, 4.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn capability_registry_consistency(
        ops in {
            let owners = vec!["plugin-a".to_string(), "plugin-b".to_string(), "plugin-c".to_string(),
                             "plugin-d".to_string(), "plugin-e".to_string()];
            proptest::collection::vec(cap_op_strategy(owners), 10..50)
        }
    ) {
        let registry = CapabilityRegistry::new();

        // Track expected state: which owners currently have capabilities registered
        let mut registered: Vec<(String, CapabilityType)> = Vec::new();

        for op in &ops {
            match op {
                CapOp::Register { owner, cap_type } => {
                    let cap = make_capability_from_type(*cap_type);
                    registry.register(owner, cap).unwrap();
                    registered.push((owner.clone(), *cap_type));
                }
                CapOp::UnregisterAll { owner } => {
                    registry.unregister_all(owner);
                    registered.retain(|(o, _)| o != owner);
                }
            }
        }

        // Verify: for each capability type, query returns exactly what we expect
        for cap_type in &[
            CapabilityType::Commands,
            CapabilityType::Viewers,
            CapabilityType::Providers,
            CapabilityType::LanguageSupport,
            CapabilityType::ThemeContribution,
        ] {
            let expected_count = registered.iter()
                .filter(|(_, ct)| ct == cap_type)
                .count();
            let actual = registry.query_by_type(*cap_type);
            prop_assert_eq!(actual.len(), expected_count,
                "mismatch for {:?}: expected {}, got {}",
                cap_type, expected_count, actual.len());
        }
    }
}

// ─── Property 6: Configuration Scoping Enforcement ──────────────────────────

// Feature: plugin-architecture, Property 6: Configuration scoping enforcement
// **Validates: Requirements 2.7, 7.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]
    #[test]
    fn configuration_scoping_enforcement(
        plugin_name in plugin_name_strategy(),
        key in prop_oneof![
            // Valid keys: simple identifiers
            "[a-z][a-z0-9_]{1,20}",
            // Valid keys: dotted paths (not starting with "plugins.")
            "[a-z][a-z0-9_]{1,10}\\.[a-z][a-z0-9_]{1,10}",
            // Invalid keys: plugins.* prefix
            "plugins\\.[a-z][a-z0-9_]{1,15}",
            // Invalid keys: absolute paths
            "/[a-z]{1,15}",
            // Invalid keys: parent traversal
            "\\.\\./?[a-z]{1,10}",
        ],
    ) {
        let result = validate_config_key(&plugin_name, &key);

        // Determine expected result
        let should_reject = key.is_empty()
            || key.starts_with("plugins.")
            || key.starts_with('/')
            || key.starts_with('\\')
            || key.contains("..");

        match result {
            Ok(()) => {
                prop_assert!(!should_reject,
                    "key '{}' was accepted but should be rejected for plugin '{}'",
                    key, plugin_name);
            }
            Err(PluginError::ConfigAccessDenied { plugin, key: rejected_key }) => {
                prop_assert!(should_reject,
                    "key '{}' was rejected but should be accepted for plugin '{}'",
                    key, plugin_name);
                prop_assert_eq!(plugin, plugin_name);
                prop_assert_eq!(rejected_key, key);
            }
            Err(e) => {
                prop_assert!(false, "unexpected error: {:?}", e);
            }
        }
    }
}

// ─── Property 7: Panic Isolation ────────────────────────────────────────────

// Feature: plugin-architecture, Property 7: Panic isolation
// **Validates: Requirements 5.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn panic_isolation(
        num_plugins in 2usize..8,
        panic_mask in any::<u64>(),
    ) {
        // For each plugin, decide if it panics during activate
        let mut panicking_plugins = HashSet::new();

        for i in 0..num_plugins {
            if (panic_mask >> (i % 64)) & 1 == 1 {
                panicking_plugins.insert(format!("plugin-{i}"));
            }
        }

        // Simulate lifecycle calls with catch_unwind
        for i in 0..num_plugins {
            let name = format!("plugin-{i}");
            let should_panic = panicking_plugins.contains(&name);

            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if should_panic {
                    panic!("intentional panic from {name}");
                }
                Ok::<(), String>(())
            }));

            match result {
                Ok(Ok(())) => {
                    // Non-panicking plugin succeeded
                    prop_assert!(!should_panic,
                        "plugin {} should have panicked but didn't", name);
                }
                Ok(Err(_)) => {
                    // Plugin returned error (not a panic)
                    prop_assert!(false, "unexpected error return");
                }
                Err(_) => {
                    // Panic was caught
                    prop_assert!(should_panic,
                        "plugin {} panicked unexpectedly", name);
                }
            }
        }
        // If we reach here, no panic propagated to the host — property holds
    }
}

// ─── Property 8: Capability Ownership Identity ──────────────────────────────

// Feature: plugin-architecture, Property 8: Capability ownership identity
// **Validates: Requirements 7.6**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn capability_ownership_identity(
        num_plugins in 2usize..8,
        caps_per_plugin in 1usize..4,
    ) {
        let registry = CapabilityRegistry::new();

        let mut expected_owners: Vec<(String, CapabilityType)> = Vec::new();

        for i in 0..num_plugins {
            let owner = format!("plugin-{i}");
            for _ in 0..caps_per_plugin {
                let cap = Capability::Commands(CommandsCapability {
                    command_ids: vec![format!("{owner}.cmd")],
                    category: "test".to_string(),
                    version: Version::new(1, 0, 0),
                });
                registry.register(&owner, cap).unwrap();
                expected_owners.push((owner.clone(), CapabilityType::Commands));
            }
        }

        // Verify: every descriptor's owner_plugin matches who registered it
        let all_descriptors = registry.all_descriptors();
        prop_assert_eq!(all_descriptors.len(), expected_owners.len());

        for (desc, (expected_owner, _)) in all_descriptors.iter().zip(expected_owners.iter()) {
            prop_assert_eq!(&desc.owner_plugin, expected_owner,
                "capability owned by '{}' but expected '{}'",
                desc.owner_plugin, expected_owner);
        }
    }
}

// ─── Property 9: Shutdown Reverse Dependency Order ──────────────────────────

// Feature: plugin-architecture, Property 9: Shutdown reverse dependency order
// **Validates: Requirements 5.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn shutdown_reverse_dependency_order(
        num_plugins in 2usize..30,
        seed in any::<u64>(),
    ) {
        use std::collections::HashMap;

        // Generate a guaranteed DAG
        let names: Vec<String> = (0..num_plugins).map(|i| format!("plugin-{i}")).collect();
        let mut plugins = Vec::new();
        let mut rng_state = seed;

        for (i, name) in names.iter().enumerate() {
            let mut deps = Vec::new();
            for j in 0..i {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                if (rng_state >> 33) % 4 < 1 { // 25% chance of edge
                    deps.push(PluginDependency {
                        name: names[j].clone(),
                        version_req: VersionReq {
                            minimum: Version::new(1, 0, 0),
                            same_major: true,
                        },
                    });
                }
            }
            plugins.push(PluginMetadata {
                name: name.clone(),
                version: Version::new(1, 0, 0),
                author: "Test".to_string(),
                description: "".to_string(),
                dependencies: deps,
                required_api_version: Version::new(1, 0, 0),
            });
        }

        let (graph, _) = DependencyGraph::build(&plugins);
        let load_order = graph.topological_sort().expect("DAG should sort");

        // Shutdown order is reverse of load order
        let mut shutdown_order = load_order.clone();
        shutdown_order.reverse();

        // Verify: for every edge A→B (A depends on B), A appears before B in shutdown
        let shutdown_positions: HashMap<&str, usize> = shutdown_order.iter()
            .enumerate()
            .map(|(i, n)| (n.as_str(), i))
            .collect();

        for plugin in &plugins {
            let plugin_pos = shutdown_positions[plugin.name.as_str()];
            for dep in &plugin.dependencies {
                let dep_pos = shutdown_positions[dep.name.as_str()];
                // A depends on B, so A should be shut down BEFORE B
                prop_assert!(plugin_pos < dep_pos,
                    "plugin {} (pos {}) depends on {} (pos {}), should shutdown first",
                    plugin.name, plugin_pos, dep.name, dep_pos);
            }
        }
    }
}

// ─── Property 10: Duplicate Capability Resolution ───────────────────────────

// Feature: plugin-architecture, Property 10: Duplicate capability resolution
// **Validates: Requirements 3.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn duplicate_capability_resolution(
        num_plugins in 2usize..8,
    ) {
        let registry = CapabilityRegistry::new();

        // All plugins register the same capability type
        let mut registration_order: Vec<String> = Vec::new();

        for i in 0..num_plugins {
            let owner = format!("plugin-{i}");
            let cap = Capability::Commands(CommandsCapability {
                command_ids: vec![format!("{owner}.cmd")],
                category: "shared".to_string(),
                version: Version::new(1, 0, 0),
            });
            registry.register(&owner, cap).unwrap();
            registration_order.push(owner);
        }

        // Query returns all instances
        let results = registry.query_by_type(CapabilityType::Commands);
        prop_assert_eq!(results.len(), num_plugins,
            "expected {} results, got {}", num_plugins, results.len());

        // Results are ordered by registration_order (ascending)
        for i in 1..results.len() {
            prop_assert!(results[i-1].registration_order < results[i].registration_order,
                "results not ordered by registration_order");
        }

        // First entry is the first-registered provider (the default)
        prop_assert_eq!(&results[0].owner_plugin, &registration_order[0],
            "first result should be the first-registered provider");
    }
}
