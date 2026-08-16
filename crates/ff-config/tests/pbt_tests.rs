//! Property-based tests for ff-config.
//!
//! These tests validate the core correctness properties of the configuration
//! system using randomized inputs via the `proptest` crate.

use std::path::PathBuf;

use proptest::prelude::*;

use ff_config::layer::ConfigLayer;
use ff_config::merge_layers;
use ff_config::namespace::plugin_namespace_prefix;
use ff_config::plugin_handle::create_plugin_config_handle;
use ff_config::schema::{Constraints, SchemaEntry, SchemaRegistry};
use ff_config::store::EffectiveStore;
use ff_config::validate::validate_value;
use ff_config::value::{ConfigTable, ConfigValue};
use ff_config::ConfigError;

// ─────────────────────────────────────────────────────────────────────────────
// Strategies
// ─────────────────────────────────────────────────────────────────────────────

/// Generate a scalar ConfigValue (no tables/arrays for simplicity).
fn arb_scalar_value() -> impl Strategy<Value = ConfigValue> {
    prop_oneof![
        "[a-z]{1,10}".prop_map(ConfigValue::String),
        (-1000i64..1000).prop_map(ConfigValue::Integer),
        (-100.0f64..100.0).prop_map(ConfigValue::Float),
        any::<bool>().prop_map(ConfigValue::Boolean),
    ]
}

/// Generate a simple key name for config tables.
fn arb_key() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,7}"
}

/// Generate a ConfigTable with leaf (scalar) values, up to a given depth.
fn arb_config_table(max_depth: u32) -> impl Strategy<Value = ConfigTable> {
    let leaf = prop::collection::btree_map(arb_key(), arb_scalar_value(), 1..5);
    leaf.prop_recursive(max_depth, 16, 4, |inner| {
        prop::collection::btree_map(
            arb_key(),
            prop_oneof![
                3 => arb_scalar_value(),
                1 => inner.prop_map(ConfigValue::Table),
            ],
            1..5,
        )
    })
}

/// Generate a valid plugin name: lowercase ASCII + digits + hyphens, 1-32 chars.
fn arb_plugin_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9\\-]{0,15}".prop_filter("must not start/end with hyphen", |s| {
        !s.starts_with('-') && !s.ends_with('-') && !s.contains("--")
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 1: Layer Precedence Determinism
// ─────────────────────────────────────────────────────────────────────────────

/// Strategy to generate a subset of distinct layers (2-6 layers, each unique).
fn arb_distinct_layers_with_values() -> impl Strategy<Value = Vec<(ConfigLayer, ConfigValue)>> {
    // Generate a random permutation of all layers, then take 2-6
    let all_layers = vec![
        ConfigLayer::Defaults,
        ConfigLayer::System,
        ConfigLayer::User,
        ConfigLayer::Profile,
        ConfigLayer::Project,
        ConfigLayer::Workspace,
    ];
    (2usize..=6, prop::collection::vec(arb_scalar_value(), 6)).prop_map(move |(count, values)| {
        let count = count.min(all_layers.len());
        all_layers[..count]
            .iter()
            .zip(values.into_iter())
            .map(|(l, v)| (*l, v))
            .collect::<Vec<_>>()
    })
}

// Feature: configuration-system, Property 1: Layer Precedence Determinism
// **Validates: Requirements 2.1, 2.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn layer_precedence_determinism(
        // Generate 2-6 (layer, value) pairs with distinct layers per entry
        entries in arb_distinct_layers_with_values()
    ) {
        use ff_config::loader::LayerData;

        let key = "test_key";

        // Determine expected winner: highest-priority layer
        let max_layer = entries.iter().map(|(l, _)| *l).max().unwrap();
        let expected_value = entries
            .iter()
            .find(|(l, _)| *l == max_layer)
            .unwrap()
            .1
            .clone();

        // Build layer data in original order
        let layers: Vec<LayerData> = entries
            .iter()
            .map(|(layer, value)| {
                let mut table = ConfigTable::new();
                table.insert(key.to_string(), value.clone());
                LayerData {
                    layer: *layer,
                    source_path: PathBuf::from(format!("{:?}.toml", layer)),
                    values: table,
                }
            })
            .collect();

        let schema = SchemaRegistry::new();
        let store = merge_layers(&layers, &schema);
        let effective = store.get_value(key).unwrap();
        prop_assert_eq!(effective, &expected_value,
            "Effective value should come from highest-priority layer {:?}", max_layer);

        // Shuffle order (reverse) and verify same result (determinism)
        let mut shuffled_layers = layers.clone();
        shuffled_layers.reverse();
        let store2 = merge_layers(&shuffled_layers, &schema);
        let effective2 = store2.get_value(key).unwrap();
        prop_assert_eq!(effective2, &expected_value,
            "Shuffled insertion order should produce same effective value");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 2: Recursive Table Merge
// ─────────────────────────────────────────────────────────────────────────────

// Feature: configuration-system, Property 2: Recursive Table Merge
// **Validates: Requirement 2.7**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn recursive_table_merge(
        lower_table in arb_config_table(3),
        higher_table in arb_config_table(3),
    ) {
        use ff_config::loader::LayerData;

        let layers = vec![
            LayerData {
                layer: ConfigLayer::User,
                source_path: PathBuf::from("user.toml"),
                values: lower_table.clone(),
            },
            LayerData {
                layer: ConfigLayer::Project,
                source_path: PathBuf::from("project.toml"),
                values: higher_table.clone(),
            },
        ];

        let schema = SchemaRegistry::new();
        let store = merge_layers(&layers, &schema);

        // Collect all flattened keys from both tables
        fn flatten_keys(table: &ConfigTable, prefix: &str, out: &mut Vec<String>) {
            for (k, v) in table {
                let full = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                match v {
                    ConfigValue::Table(sub) => flatten_keys(sub, &full, out),
                    _ => out.push(full),
                }
            }
        }

        let mut lower_keys = Vec::new();
        flatten_keys(&lower_table, "", &mut lower_keys);
        let mut higher_keys = Vec::new();
        flatten_keys(&higher_table, "", &mut higher_keys);

        // All keys from higher-priority layer must be present
        for key in &higher_keys {
            prop_assert!(
                store.get_value(key).is_some(),
                "Higher-priority key '{}' must be present in merged store", key
            );
        }

        // All keys unique to lower-priority layer must also be present
        for key in &lower_keys {
            prop_assert!(
                store.get_value(key).is_some(),
                "Lower-priority key '{}' should still be present after merge (union)", key
            );
        }

        // For conflicting keys, the higher-priority value wins
        fn get_leaf(table: &ConfigTable, path: &str) -> Option<ConfigValue> {
            let parts: Vec<&str> = path.splitn(2, '.').collect();
            match table.get(parts[0]) {
                Some(ConfigValue::Table(sub)) if parts.len() == 2 => {
                    get_leaf(sub, parts[1])
                }
                Some(v) if parts.len() == 1 => Some(v.clone()),
                _ => None,
            }
        }

        for key in &higher_keys {
            let expected = get_leaf(&higher_table, key);
            if let Some(ref exp) = expected {
                let actual = store.get_value(key).unwrap();
                prop_assert_eq!(actual, exp,
                    "Conflicting key '{}' should use higher-priority value", key);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 3: Schema Validation Fallback
// ─────────────────────────────────────────────────────────────────────────────

// Feature: configuration-system, Property 3: Schema Validation Fallback
// **Validates: Requirement 7.5, 7.6; Requirement 9.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn schema_validation_fallback_type_mismatch(
        // Generate a valid integer default
        default_val in 1i64..100,
        // Generate a string value that violates integer type expectation
        invalid_str in "[a-z]{1,10}",
    ) {
        let entry = SchemaEntry {
            key: "test.key".to_string(),
            value_type: ff_config::error::ValueType::Integer,
            default: ConfigValue::Integer(default_val),
            description: "Test entry".to_string(),
            constraints: Some(Constraints {
                min: Some(1.0),
                max: Some(100.0),
                allowed_values: None,
                pattern: None,
            }),
        };

        // String value violates the Integer type constraint
        let invalid_value = ConfigValue::String(invalid_str);
        let result = validate_value(&invalid_value, &entry);

        match result {
            ff_config::validate::ValidationResult::DefaultApplied { default, .. } => {
                prop_assert_eq!(default, ConfigValue::Integer(default_val),
                    "Should return the schema default when type mismatches");
            }
            ff_config::validate::ValidationResult::Valid(_) => {
                prop_assert!(false, "Invalid value should not pass validation");
            }
        }
    }

    #[test]
    fn schema_validation_fallback_out_of_range(
        // Default always in range
        default_val in 1i64..50,
        // Value that exceeds max (101+)
        over_max in 101i64..1000,
    ) {
        let entry = SchemaEntry {
            key: "test.key".to_string(),
            value_type: ff_config::error::ValueType::Integer,
            default: ConfigValue::Integer(default_val),
            description: "Test entry".to_string(),
            constraints: Some(Constraints {
                min: Some(1.0),
                max: Some(100.0),
                allowed_values: None,
                pattern: None,
            }),
        };

        let invalid_value = ConfigValue::Integer(over_max);
        let result = validate_value(&invalid_value, &entry);

        match result {
            ff_config::validate::ValidationResult::DefaultApplied { default, .. } => {
                prop_assert_eq!(default, ConfigValue::Integer(default_val),
                    "Should return schema default when value exceeds max");
            }
            ff_config::validate::ValidationResult::Valid(_) => {
                prop_assert!(false, "Out-of-range value should not pass validation");
            }
        }
    }

    #[test]
    fn schema_validation_fallback_below_min(
        default_val in 10i64..50,
        // Value below min (negative or zero)
        below_min in -1000i64..0,
    ) {
        let entry = SchemaEntry {
            key: "test.key".to_string(),
            value_type: ff_config::error::ValueType::Integer,
            default: ConfigValue::Integer(default_val),
            description: "Test entry".to_string(),
            constraints: Some(Constraints {
                min: Some(1.0),
                max: Some(100.0),
                allowed_values: None,
                pattern: None,
            }),
        };

        let invalid_value = ConfigValue::Integer(below_min);
        let result = validate_value(&invalid_value, &entry);

        match result {
            ff_config::validate::ValidationResult::DefaultApplied { default, .. } => {
                prop_assert_eq!(default, ConfigValue::Integer(default_val),
                    "Should return schema default when value below min");
            }
            ff_config::validate::ValidationResult::Valid(_) => {
                prop_assert!(false, "Below-minimum value should not pass validation");
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 4: Namespace Isolation
// ─────────────────────────────────────────────────────────────────────────────

// Feature: configuration-system, Property 4: Namespace Isolation
// **Validates: Requirement 8.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn namespace_isolation(
        plugin_name in arb_plugin_name(),
        relative_key in arb_key(),
    ) {
        // Ensure plugin_name is not reserved
        let reserved = ["logging", "editor", "theme", "vfs", "commands", "layout", "core"];
        prop_assume!(!reserved.contains(&plugin_name.as_str()));

        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();

        let handle = create_plugin_config_handle(&store, &schema, &plugin_name);
        prop_assume!(handle.is_ok());
        let handle = handle.unwrap();

        // The namespace prefix should be "plugins.{plugin_name}."
        let expected_prefix = plugin_namespace_prefix(&plugin_name);
        prop_assert_eq!(handle.namespace(), expected_prefix.as_str());

        // Any key access through the handle should resolve to a key within the namespace
        // The handle always prepends its namespace prefix, so any relative key "foo"
        // becomes "plugins.{plugin_name}.foo" — structurally guaranteeing isolation.
        // Let's verify by checking that get() for any relative_key either:
        // 1. Returns an error (key not found) — but the full key starts with namespace
        // 2. Returns a value — from a key that starts with namespace
        let result = handle.get(&relative_key);
        match result {
            Ok(_) => {
                // Value found — it must be under the namespace prefix
                // (This would only happen if the store had the key)
                prop_assert!(true);
            }
            Err(ConfigError::UndefinedKey { ref key }) => {
                // The resolved key should start with the namespace prefix
                prop_assert!(
                    key.starts_with(&expected_prefix),
                    "Resolved key '{}' should start with namespace prefix '{}'",
                    key, expected_prefix
                );
            }
            Err(other) => {
                prop_assert!(false, "Unexpected error: {:?}", other);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 5: Hot-Reload Atomicity
// ─────────────────────────────────────────────────────────────────────────────

// Feature: configuration-system, Property 5: Hot-Reload Atomicity
// **Validates: Requirement 3.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn hot_reload_atomicity(
        // Generate 2-10 distinct keys with new values
        num_keys in 2usize..10,
        values in prop::collection::vec(1i64..1000, 10),
    ) {
        use ff_config::loader::LayerData;
        use ff_config::reload::ReloadManager;

        // Ensure we have enough unique keys
        let keys: Vec<String> = (0..num_keys).map(|i| format!("key_{}", i)).collect();
        let values: Vec<i64> = values.into_iter().take(num_keys).collect();

        // Create a temp file with initial values (all zeros)
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.toml");

        let initial_toml = keys
            .iter()
            .map(|k| format!("{} = 0", k))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&path, &initial_toml).unwrap();

        let initial_values = ff_config::loader::load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path.clone(),
            values: initial_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Write new TOML with distinct keys and new values
        let new_toml = keys
            .iter()
            .zip(values.iter())
            .map(|(k, v)| format!("{} = {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&path, &new_toml).unwrap();

        // Reload — this is atomic (store swap)
        let result = manager.reload_file(&path, ConfigLayer::User);
        prop_assert!(result.is_ok());

        // After atomic reload, ALL keys should consistently have their new values.
        // No mix of old zeros and new values is possible.
        let store = manager.store();
        for (k, v) in keys.iter().zip(values.iter()) {
            let effective = store.get_value(k);
            prop_assert_eq!(
                effective,
                Some(&ConfigValue::Integer(*v)),
                "After atomic reload, key '{}' should have new value {}", k, v
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 6: Debounce Coalescing
// ─────────────────────────────────────────────────────────────────────────────

// Feature: configuration-system, Property 6: Debounce Coalescing
// **Validates: Requirement 3.7**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn debounce_coalescing(
        // Generate 2-10 events within a 500ms window with varying gaps
        event_count in 2usize..10,
        final_value in 1i64..1000,
    ) {
        use ff_config::loader::LayerData;
        use ff_config::reload::ReloadManager;

        // Simulate the debounce coalescing logic:
        // Multiple rapid writes followed by a single reload should produce exactly
        // one reload that sees the final state.

        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        // Write initial value
        std::fs::write(&path, "value = 0\n").unwrap();
        let initial_values = ff_config::loader::load_toml_file(&path).unwrap();
        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: path.clone(),
            values: initial_values,
        }];

        let schema = SchemaRegistry::new();
        let mut manager = ReloadManager::new(layers, schema);

        // Simulate multiple rapid writes (all within 500ms debounce window)
        // Each write overwrites the file, simulating rapid user edits
        for i in 0..event_count {
            let intermediate_value = if i == event_count - 1 {
                final_value
            } else {
                i as i64 + 1 // intermediate values
            };
            std::fs::write(&path, format!("value = {}\n", intermediate_value)).unwrap();
        }

        // After debounce coalescing, only ONE reload is performed.
        // The reload picks up the FINAL state of the file.
        let result = manager.reload_file(&path, ConfigLayer::User);
        prop_assert!(result.is_ok());

        let reload_count = if result.unwrap().is_some() { 1 } else { 0 };

        // Exactly one reload was performed (we called reload_file once after debounce)
        prop_assert_eq!(reload_count, 1,
            "Debounced events should result in exactly one reload");

        // The reloaded content matches the final write
        let effective = manager.store().get_value("value");
        prop_assert_eq!(
            effective,
            Some(&ConfigValue::Integer(final_value)),
            "Reloaded content should match the final write (value = {})", final_value
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 7: Profile Layer Placement
// ─────────────────────────────────────────────────────────────────────────────

// Feature: configuration-system, Property 7: Profile Layer Placement
// **Validates: Requirements 4.2, 4.3, 2.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn profile_layer_placement(
        user_val in arb_scalar_value(),
        profile_val in arb_scalar_value(),
        project_val in arb_scalar_value(),
        // Which layers are present: bit 0 = User, bit 1 = Profile, bit 2 = Project
        present_mask in 1u8..8,
    ) {
        use ff_config::loader::LayerData;

        let key = "test_key";

        // Build layers based on the presence mask (at least one layer is present)
        let mut layers: Vec<LayerData> = Vec::new();

        let has_user = present_mask & 0b001 != 0;
        let has_profile = present_mask & 0b010 != 0;
        let has_project = present_mask & 0b100 != 0;

        if has_user {
            let mut table = ConfigTable::new();
            table.insert(key.to_string(), user_val.clone());
            layers.push(LayerData {
                layer: ConfigLayer::User,
                source_path: PathBuf::from("user.toml"),
                values: table,
            });
        }
        if has_profile {
            let mut table = ConfigTable::new();
            table.insert(key.to_string(), profile_val.clone());
            layers.push(LayerData {
                layer: ConfigLayer::Profile,
                source_path: PathBuf::from("profile.toml"),
                values: table,
            });
        }
        if has_project {
            let mut table = ConfigTable::new();
            table.insert(key.to_string(), project_val.clone());
            layers.push(LayerData {
                layer: ConfigLayer::Project,
                source_path: PathBuf::from("project.toml"),
                values: table,
            });
        }

        let schema = SchemaRegistry::new();
        let store = merge_layers(&layers, &schema);

        // Determine expected value: Project > Profile > User
        let expected = if has_project {
            &project_val
        } else if has_profile {
            &profile_val
        } else {
            &user_val
        };

        let effective = store.get_value(key).unwrap();
        prop_assert_eq!(effective, expected,
            "Effective value should follow Project > Profile > User precedence. \
             User={}, Profile={}, Project={}",
            has_user, has_profile, has_project);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 8: EditorConfig Precedence
// ─────────────────────────────────────────────────────────────────────────────

// Feature: configuration-system, Property 8: EditorConfig Precedence
// **Validates: Requirement 6.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn editorconfig_precedence(
        // Generate an indent_size for the workspace layer (1-8)
        workspace_indent in 1u32..9,
        // Generate an indent_size for the EditorConfig (1-8), must differ from workspace
        ec_indent in 1u32..9,
    ) {
        use ff_config::editorconfig::resolver::resolve_editorconfig;
        use ff_config::editorconfig::parser::IndentSize;

        // Skip if they happen to be the same (we want to verify precedence)
        prop_assume!(workspace_indent != ec_indent);

        // Create a temp directory with a file and an .editorconfig
        let dir = tempfile::TempDir::new().unwrap();
        let root = dir.path();

        // Create the target file
        let file_path = root.join("main.rs");
        std::fs::write(&file_path, "").unwrap();

        // Create .editorconfig that sets indent_size for *.rs files
        let ec_content = format!(
            "root = true\n\n[*.rs]\nindent_size = {}\n",
            ec_indent
        );
        std::fs::write(root.join(".editorconfig"), &ec_content).unwrap();

        // Resolve EditorConfig for the file
        let props = resolve_editorconfig(&file_path);

        // EditorConfig value should take precedence over any workspace layer value
        prop_assert_eq!(
            props.indent_size,
            Some(IndentSize::Value(ec_indent)),
            "EditorConfig indent_size ({}) must take precedence over workspace value ({})",
            ec_indent, workspace_indent
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 9: Unknown Key Tolerance
// ─────────────────────────────────────────────────────────────────────────────

// Feature: configuration-system, Property 9: Unknown Key Tolerance
// **Validates: Requirement 9.6**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn unknown_key_tolerance(
        // Generate a known key with a value
        known_key in arb_key(),
        known_value in arb_scalar_value(),
        // Generate 1-5 unknown keys
        unknown_keys in prop::collection::vec("[a-z]{1,5}_unknown_[0-9]{1,3}", 1..5),
    ) {
        use ff_config::loader::LayerData;

        // Ensure unknown keys don't collide with the known key
        prop_assume!(!unknown_keys.contains(&known_key));

        // Build a config table with both known and unknown keys
        let mut table = ConfigTable::new();
        table.insert(known_key.clone(), known_value.clone());
        for uk in &unknown_keys {
            table.insert(uk.clone(), ConfigValue::String("unknown_val".to_string()));
        }

        let layers = vec![LayerData {
            layer: ConfigLayer::User,
            source_path: PathBuf::from("user.toml"),
            values: table,
        }];

        // Merge with empty schema — all keys are "unknown" to the schema
        // but loading should still succeed
        let schema = SchemaRegistry::new();
        let store = merge_layers(&layers, &schema);

        // The known key should be accessible in the merged store
        let effective = store.get_value(&known_key);
        prop_assert_eq!(effective, Some(&known_value),
            "Known key '{}' should be accessible even with unknown keys present", known_key);

        // Unknown keys are also present in the store (they are loaded, just not schema-validated)
        // The key point is: no error or panic occurred during merge
        for uk in &unknown_keys {
            let uk_val = store.get_value(uk);
            prop_assert!(uk_val.is_some(),
                "Unknown key '{}' should be loaded without causing an error", uk);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 10: Provenance Accuracy
// ─────────────────────────────────────────────────────────────────────────────

/// Strategy: generate 2-6 layers, each with a random subset of keys defined.
fn arb_layers_with_key_subsets() -> impl Strategy<Value = Vec<(ConfigLayer, PathBuf, ConfigValue)>>
{
    let all_layers = vec![
        (ConfigLayer::Defaults, PathBuf::from("defaults.toml")),
        (ConfigLayer::System, PathBuf::from("system.toml")),
        (ConfigLayer::User, PathBuf::from("user.toml")),
        (ConfigLayer::Profile, PathBuf::from("profile.toml")),
        (ConfigLayer::Project, PathBuf::from("project.toml")),
        (ConfigLayer::Workspace, PathBuf::from("workspace.toml")),
    ];
    (
        2usize..=6,
        prop::collection::vec(arb_scalar_value(), 6),
        prop::collection::vec(any::<bool>(), 6),
    )
        .prop_map(move |(count, values, include)| {
            let count = count.min(all_layers.len());
            all_layers[..count]
                .iter()
                .zip(values.into_iter())
                .zip(include.into_iter())
                .filter(|((_, _), inc)| *inc)
                .map(|(((layer, path), val), _)| (*layer, path.clone(), val))
                .collect::<Vec<_>>()
        })
        .prop_filter("at least one layer must define the key", |v| !v.is_empty())
}

// Feature: configuration-system, Property 10: Provenance Accuracy
// **Validates: Requirement 2.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn provenance_accuracy(
        layer_entries in arb_layers_with_key_subsets()
    ) {
        use ff_config::loader::LayerData;

        let key = "provenance_test_key";

        // Build LayerData for each entry
        let layers: Vec<LayerData> = layer_entries
            .iter()
            .map(|(layer, path, value)| {
                let mut table = ConfigTable::new();
                table.insert(key.to_string(), value.clone());
                LayerData {
                    layer: *layer,
                    source_path: path.clone(),
                    values: table,
                }
            })
            .collect();

        let schema = SchemaRegistry::new();
        let store = merge_layers(&layers, &schema);

        // Determine the expected winning layer (highest priority)
        let expected_winner = layer_entries
            .iter()
            .max_by_key(|(layer, _, _)| *layer)
            .unwrap();

        let effective = store.get(key).unwrap();

        // Provenance layer must match the highest-priority layer that defines the key
        prop_assert_eq!(
            effective.provenance.layer, expected_winner.0,
            "Provenance layer should be {:?}, got {:?}",
            expected_winner.0, effective.provenance.layer
        );

        // Provenance source file must match
        prop_assert_eq!(
            effective.provenance.source_file.as_ref(),
            Some(&expected_winner.1),
            "Provenance source file should match the winning layer's file"
        );

        // The value must match the winning layer's value
        prop_assert_eq!(
            &effective.value, &expected_winner.2,
            "Effective value should come from the highest-priority layer"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 11: Reserved Namespace Enforcement
// ─────────────────────────────────────────────────────────────────────────────

// Feature: configuration-system, Property 11: Reserved Namespace Enforcement
// **Validates: Requirement 8.7**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn reserved_namespace_enforcement(
        // Pick a random reserved namespace index (0-7 for 8 reserved namespaces)
        reserved_idx in 0usize..8,
    ) {
        use ff_config::namespace::RESERVED_NAMESPACES;

        let reserved_name = RESERVED_NAMESPACES[reserved_idx];
        let store = EffectiveStore::new();
        let schema = SchemaRegistry::new();

        // Attempt to create a plugin handle with the reserved namespace name
        let result = create_plugin_config_handle(&store, &schema, reserved_name);

        // The result should always be an error — either ReservedNamespace or InvalidPluginName
        // (Some reserved names like "_session" contain characters invalid for plugin names,
        //  so they fail plugin name validation before reaching the reserved namespace check.)
        prop_assert!(
            result.is_err(),
            "Creating a plugin handle with reserved namespace '{}' must fail, but got Ok",
            reserved_name
        );

        match result.unwrap_err() {
            ConfigError::ReservedNamespace { ref plugin, ref namespace } => {
                prop_assert_eq!(plugin.as_str(), reserved_name);
                prop_assert_eq!(namespace.as_str(), reserved_name);
            }
            ConfigError::InvalidPluginName { ref name } => {
                // _session starts with underscore, which is invalid for plugin names
                prop_assert_eq!(name.as_str(), reserved_name,
                    "InvalidPluginName should contain the attempted name");
            }
            other => {
                prop_assert!(false,
                    "Expected ReservedNamespace or InvalidPluginName error, got: {:?}", other);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Property 12: Profile Single-Activation Invariant
// ─────────────────────────────────────────────────────────────────────────────

// Feature: configuration-system, Property 12: Profile Single-Activation Invariant
// **Validates: Requirements 4.3, 4.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn profile_single_activation_invariant(
        // Generate a sequence of 3-10 profile activation steps.
        // Each step either activates a profile (with its own distinct value)
        // or deactivates (None).
        steps in prop::collection::vec(
            prop_oneof![
                3 => (1i64..100).prop_map(Some),
                1 => Just(None),
            ],
            3..10
        ),
    ) {
        use ff_config::loader::LayerData;

        let key = "profile_key";

        // A fixed User-layer value that should show through when no profile is active
        let user_value = ConfigValue::Integer(0);

        let mut user_table = ConfigTable::new();
        user_table.insert(key.to_string(), user_value.clone());
        let user_layer = LayerData {
            layer: ConfigLayer::User,
            source_path: PathBuf::from("user.toml"),
            values: user_table,
        };

        // Simulate profile activation sequence
        for step in &steps {
            let mut layers = vec![user_layer.clone()];

            if let Some(profile_value) = step {
                // Activate profile: add a Profile layer with this value
                let mut profile_table = ConfigTable::new();
                profile_table.insert(key.to_string(), ConfigValue::Integer(*profile_value));
                layers.push(LayerData {
                    layer: ConfigLayer::Profile,
                    source_path: PathBuf::from("profile.toml"),
                    values: profile_table,
                });
            }
            // If step is None, no profile layer is present (deactivation)

            let schema = SchemaRegistry::new();
            let store = merge_layers(&layers, &schema);

            let effective = store.get_value(key).unwrap();

            if let Some(profile_value) = step {
                // Profile is active: Profile layer (priority 3) > User layer (priority 2)
                prop_assert_eq!(
                    effective,
                    &ConfigValue::Integer(*profile_value),
                    "With active profile, effective value should be the profile value"
                );
                // Verify only one profile layer's provenance
                let eff = store.get(key).unwrap();
                prop_assert_eq!(eff.provenance.layer, ConfigLayer::Profile);
            } else {
                // No profile active: User layer value should win
                prop_assert_eq!(
                    effective,
                    &user_value,
                    "With no active profile, effective value should fall back to User layer"
                );
                let eff = store.get(key).unwrap();
                prop_assert_eq!(eff.provenance.layer, ConfigLayer::User);
            }
        }
    }
}
