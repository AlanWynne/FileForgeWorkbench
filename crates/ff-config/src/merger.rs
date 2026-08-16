//! Layer merging logic.
//!
//! Implements key-by-key recursive merging of configuration tables from
//! multiple layers, producing the effective configuration store.

use std::path::PathBuf;

use crate::layer::ConfigLayer;
use crate::loader::LayerData;
use crate::provenance::{EffectiveValue, Provenance};
use crate::schema::SchemaRegistry;
use crate::store::EffectiveStore;
use crate::value::{ConfigTable, ConfigValue};

/// Merge multiple layers into an EffectiveStore.
///
/// Layers are processed in priority order (lowest to highest).
/// For each key:
/// - Tables are recursively merged (keys from both sides preserved)
/// - Scalar values from higher-priority layers override lower ones
/// - Provenance records which layer provided each final value
///
/// After merging all layers, schema defaults are applied for keys
/// defined in the schema but not present in any layer.
pub fn merge_layers(layers: &[LayerData], schema: &SchemaRegistry) -> EffectiveStore {
    let mut store = EffectiveStore::new();

    // Sort layers by priority (lowest first, so higher layers overwrite)
    let mut sorted_layers: Vec<&LayerData> = layers.iter().collect();
    sorted_layers.sort_by_key(|l| l.layer);

    // Merge each layer's values into the store
    for layer_data in &sorted_layers {
        flatten_and_merge(
            &layer_data.values,
            "",
            layer_data.layer,
            &layer_data.source_path,
            &mut store,
        );
    }

    // Apply schema defaults for keys not present in any layer
    for entry in schema.list_all() {
        if store.get(&entry.key).is_none() {
            store.insert(
                entry.key.clone(),
                EffectiveValue {
                    value: entry.default.clone(),
                    provenance: Provenance {
                        layer: ConfigLayer::Defaults,
                        source_file: None,
                    },
                },
            );
        }
    }

    store
}

/// Recursively flatten a ConfigTable into dot-path keys and merge into the store.
fn flatten_and_merge(
    table: &ConfigTable,
    prefix: &str,
    layer: ConfigLayer,
    source_path: &PathBuf,
    store: &mut EffectiveStore,
) {
    for (key, value) in table {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            ConfigValue::Table(sub_table) => {
                // Recursive merge for nested tables
                flatten_and_merge(sub_table, &full_key, layer, source_path, store);
            }
            _ => {
                // Scalar value — higher priority layer overwrites
                store.insert(
                    full_key,
                    EffectiveValue {
                        value: value.clone(),
                        provenance: Provenance {
                            layer,
                            source_file: Some(source_path.clone()),
                        },
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ValueType;
    use crate::schema::SchemaEntry;

    /// Helper: create a LayerData with given layer, path, and values.
    fn make_layer(layer: ConfigLayer, path: &str, values: ConfigTable) -> LayerData {
        LayerData {
            layer,
            source_path: PathBuf::from(path),
            values,
        }
    }

    /// Helper: create a minimal schema entry.
    fn make_schema_entry(key: &str, default: ConfigValue) -> SchemaEntry {
        let value_type = match &default {
            ConfigValue::String(_) => ValueType::String,
            ConfigValue::Integer(_) => ValueType::Integer,
            ConfigValue::Float(_) => ValueType::Float,
            ConfigValue::Boolean(_) => ValueType::Boolean,
            ConfigValue::Array(_) => ValueType::Array,
            ConfigValue::Table(_) => ValueType::Table,
        };
        SchemaEntry {
            key: key.to_string(),
            value_type,
            default,
            description: format!("Test entry for {key}"),
            constraints: None,
        }
    }

    // Validates: Requirement 2.2 — Two layers with the same table key merge keys (not replace)
    #[test]
    fn recursive_table_merge_preserves_keys_from_both_layers() {
        let schema = SchemaRegistry::new();

        // User layer defines editor.tab_size
        let mut user_editor = ConfigTable::new();
        user_editor.insert("tab_size".to_string(), ConfigValue::Integer(4));
        let mut user_values = ConfigTable::new();
        user_values.insert("editor".to_string(), ConfigValue::Table(user_editor));

        // Project layer defines editor.indent_style
        let mut project_editor = ConfigTable::new();
        project_editor.insert(
            "indent_style".to_string(),
            ConfigValue::String("space".to_string()),
        );
        let mut project_values = ConfigTable::new();
        project_values.insert("editor".to_string(), ConfigValue::Table(project_editor));

        let layers = vec![
            make_layer(ConfigLayer::User, "/home/user/config.toml", user_values),
            make_layer(
                ConfigLayer::Project,
                "/project/.ffworkbench/config.toml",
                project_values,
            ),
        ];

        let store = merge_layers(&layers, &schema);

        // Both keys should be present — tables merged, not replaced
        assert_eq!(
            store.get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(4)),
            "editor.tab_size from User layer should be preserved"
        );
        assert_eq!(
            store.get_value("editor.indent_style"),
            Some(&ConfigValue::String("space".to_string())),
            "editor.indent_style from Project layer should be preserved"
        );
    }

    // Validates: Requirement 2.4 — Higher priority layer wins for same scalar key
    #[test]
    fn higher_priority_layer_wins_for_same_scalar_key() {
        let schema = SchemaRegistry::new();

        // User layer sets editor.tab_size = 4
        let mut user_editor = ConfigTable::new();
        user_editor.insert("tab_size".to_string(), ConfigValue::Integer(4));
        let mut user_values = ConfigTable::new();
        user_values.insert("editor".to_string(), ConfigValue::Table(user_editor));

        // Project layer sets editor.tab_size = 2
        let mut project_editor = ConfigTable::new();
        project_editor.insert("tab_size".to_string(), ConfigValue::Integer(2));
        let mut project_values = ConfigTable::new();
        project_values.insert("editor".to_string(), ConfigValue::Table(project_editor));

        let layers = vec![
            make_layer(ConfigLayer::User, "/home/user/config.toml", user_values),
            make_layer(
                ConfigLayer::Project,
                "/project/.ffworkbench/config.toml",
                project_values,
            ),
        ];

        let store = merge_layers(&layers, &schema);

        // Project (priority 4) > User (priority 2), so Project wins
        assert_eq!(
            store.get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2)),
            "Project layer (higher priority) should override User layer"
        );
    }

    // Validates: Requirement 2.3 — Provenance correctly identifies which layer provided a value
    #[test]
    fn provenance_tracks_source_layer_and_file() {
        let schema = SchemaRegistry::new();

        let mut user_editor = ConfigTable::new();
        user_editor.insert("tab_size".to_string(), ConfigValue::Integer(4));
        let mut user_values = ConfigTable::new();
        user_values.insert("editor".to_string(), ConfigValue::Table(user_editor));

        let mut project_editor = ConfigTable::new();
        project_editor.insert("tab_size".to_string(), ConfigValue::Integer(2));
        let mut project_values = ConfigTable::new();
        project_values.insert("editor".to_string(), ConfigValue::Table(project_editor));

        let layers = vec![
            make_layer(ConfigLayer::User, "/home/user/config.toml", user_values),
            make_layer(
                ConfigLayer::Project,
                "/project/.ffworkbench/config.toml",
                project_values,
            ),
        ];

        let store = merge_layers(&layers, &schema);

        let effective = store.get("editor.tab_size").expect("key should exist");
        assert_eq!(
            effective.provenance.layer,
            ConfigLayer::Project,
            "Provenance should record Project as the winning layer"
        );
        assert_eq!(
            effective.provenance.source_file,
            Some(PathBuf::from("/project/.ffworkbench/config.toml")),
            "Provenance should record the source file path"
        );
    }

    // Validates: Requirement 2.5 — Schema defaults applied for keys not in any layer
    #[test]
    fn schema_defaults_applied_for_missing_keys() {
        let mut schema = SchemaRegistry::new();
        schema
            .register(make_schema_entry(
                "editor.tab_size",
                ConfigValue::Integer(4),
            ))
            .unwrap();
        schema
            .register(make_schema_entry(
                "editor.word_wrap",
                ConfigValue::Boolean(false),
            ))
            .unwrap();

        // Only provide editor.tab_size in a layer
        let mut editor = ConfigTable::new();
        editor.insert("tab_size".to_string(), ConfigValue::Integer(8));
        let mut values = ConfigTable::new();
        values.insert("editor".to_string(), ConfigValue::Table(editor));

        let layers = vec![make_layer(
            ConfigLayer::User,
            "/home/user/config.toml",
            values,
        )];

        let store = merge_layers(&layers, &schema);

        // editor.tab_size comes from the layer
        assert_eq!(
            store.get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(8))
        );

        // editor.word_wrap falls back to schema default
        let word_wrap = store.get("editor.word_wrap").expect("default should apply");
        assert_eq!(word_wrap.value, ConfigValue::Boolean(false));
        assert_eq!(word_wrap.provenance.layer, ConfigLayer::Defaults);
        assert_eq!(word_wrap.provenance.source_file, None);
    }

    // Validates: Requirement 2.1 — Empty layers produce empty store (except schema defaults)
    #[test]
    fn empty_layers_produce_store_with_only_schema_defaults() {
        let mut schema = SchemaRegistry::new();
        schema
            .register(make_schema_entry(
                "logging.level",
                ConfigValue::String("info".to_string()),
            ))
            .unwrap();

        let layers: Vec<LayerData> = vec![];
        let store = merge_layers(&layers, &schema);

        // Only schema default should be present
        assert_eq!(store.len(), 1);
        assert_eq!(
            store.get_value("logging.level"),
            Some(&ConfigValue::String("info".to_string()))
        );
    }

    // Validates: Requirement 2.2 — Three-layer merge with partial overlaps
    #[test]
    fn three_layer_merge_with_partial_overlaps() {
        let schema = SchemaRegistry::new();

        // System layer: editor.tab_size=8, logging.level="warn"
        let mut sys_editor = ConfigTable::new();
        sys_editor.insert("tab_size".to_string(), ConfigValue::Integer(8));
        let mut sys_logging = ConfigTable::new();
        sys_logging.insert("level".to_string(), ConfigValue::String("warn".to_string()));
        let mut sys_values = ConfigTable::new();
        sys_values.insert("editor".to_string(), ConfigValue::Table(sys_editor));
        sys_values.insert("logging".to_string(), ConfigValue::Table(sys_logging));

        // User layer: editor.tab_size=4, editor.word_wrap=true
        let mut user_editor = ConfigTable::new();
        user_editor.insert("tab_size".to_string(), ConfigValue::Integer(4));
        user_editor.insert("word_wrap".to_string(), ConfigValue::Boolean(true));
        let mut user_values = ConfigTable::new();
        user_values.insert("editor".to_string(), ConfigValue::Table(user_editor));

        // Workspace layer: editor.tab_size=2, theme.active="dark"
        let mut ws_editor = ConfigTable::new();
        ws_editor.insert("tab_size".to_string(), ConfigValue::Integer(2));
        let mut ws_theme = ConfigTable::new();
        ws_theme.insert(
            "active".to_string(),
            ConfigValue::String("dark".to_string()),
        );
        let mut ws_values = ConfigTable::new();
        ws_values.insert("editor".to_string(), ConfigValue::Table(ws_editor));
        ws_values.insert("theme".to_string(), ConfigValue::Table(ws_theme));

        let layers = vec![
            make_layer(
                ConfigLayer::System,
                "/etc/ffworkbench/config.toml",
                sys_values,
            ),
            make_layer(ConfigLayer::User, "/home/user/config.toml", user_values),
            make_layer(
                ConfigLayer::Workspace,
                "/project/.ffworkbench/workspace.toml",
                ws_values,
            ),
        ];

        let store = merge_layers(&layers, &schema);

        // editor.tab_size: Workspace wins (priority 5)
        assert_eq!(
            store.get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(2))
        );
        assert_eq!(
            store.get("editor.tab_size").unwrap().provenance.layer,
            ConfigLayer::Workspace
        );

        // editor.word_wrap: Only in User layer
        assert_eq!(
            store.get_value("editor.word_wrap"),
            Some(&ConfigValue::Boolean(true))
        );
        assert_eq!(
            store.get("editor.word_wrap").unwrap().provenance.layer,
            ConfigLayer::User
        );

        // logging.level: Only in System layer
        assert_eq!(
            store.get_value("logging.level"),
            Some(&ConfigValue::String("warn".to_string()))
        );
        assert_eq!(
            store.get("logging.level").unwrap().provenance.layer,
            ConfigLayer::System
        );

        // theme.active: Only in Workspace layer
        assert_eq!(
            store.get_value("theme.active"),
            Some(&ConfigValue::String("dark".to_string()))
        );
        assert_eq!(
            store.get("theme.active").unwrap().provenance.layer,
            ConfigLayer::Workspace
        );
    }

    // Validates: Requirement 2.2 — Deeply nested tables merge correctly
    #[test]
    fn deeply_nested_tables_merge_recursively() {
        let schema = SchemaRegistry::new();

        // User: plugins.sql-viewer.connection.timeout = 30
        let mut user_conn = ConfigTable::new();
        user_conn.insert("timeout".to_string(), ConfigValue::Integer(30));
        let mut user_sql = ConfigTable::new();
        user_sql.insert("connection".to_string(), ConfigValue::Table(user_conn));
        let mut user_plugins = ConfigTable::new();
        user_plugins.insert("sql-viewer".to_string(), ConfigValue::Table(user_sql));
        let mut user_values = ConfigTable::new();
        user_values.insert("plugins".to_string(), ConfigValue::Table(user_plugins));

        // Project: plugins.sql-viewer.connection.host = "localhost"
        let mut proj_conn = ConfigTable::new();
        proj_conn.insert(
            "host".to_string(),
            ConfigValue::String("localhost".to_string()),
        );
        let mut proj_sql = ConfigTable::new();
        proj_sql.insert("connection".to_string(), ConfigValue::Table(proj_conn));
        let mut proj_plugins = ConfigTable::new();
        proj_plugins.insert("sql-viewer".to_string(), ConfigValue::Table(proj_sql));
        let mut proj_values = ConfigTable::new();
        proj_values.insert("plugins".to_string(), ConfigValue::Table(proj_plugins));

        let layers = vec![
            make_layer(ConfigLayer::User, "/home/user/config.toml", user_values),
            make_layer(
                ConfigLayer::Project,
                "/project/.ffworkbench/config.toml",
                proj_values,
            ),
        ];

        let store = merge_layers(&layers, &schema);

        // Both nested keys should be present
        assert_eq!(
            store.get_value("plugins.sql-viewer.connection.timeout"),
            Some(&ConfigValue::Integer(30))
        );
        assert_eq!(
            store.get_value("plugins.sql-viewer.connection.host"),
            Some(&ConfigValue::String("localhost".to_string()))
        );
    }

    // Validates: Requirement 2.1 — Full six-layer merge producing EffectiveStore
    #[test]
    fn full_six_layer_merge_highest_priority_wins() {
        let schema = SchemaRegistry::new();

        let make_editor_layer = |layer: ConfigLayer, path: &str, tab_size: i64| -> LayerData {
            let mut editor = ConfigTable::new();
            editor.insert("tab_size".to_string(), ConfigValue::Integer(tab_size));
            let mut values = ConfigTable::new();
            values.insert("editor".to_string(), ConfigValue::Table(editor));
            make_layer(layer, path, values)
        };

        // All six layers define editor.tab_size with different values
        let layers = vec![
            make_editor_layer(ConfigLayer::Workspace, "/ws/config.toml", 1),
            make_editor_layer(ConfigLayer::Defaults, "/defaults", 8),
            make_editor_layer(ConfigLayer::Project, "/proj/config.toml", 2),
            make_editor_layer(ConfigLayer::System, "/etc/config.toml", 6),
            make_editor_layer(ConfigLayer::Profile, "/profile/config.toml", 3),
            make_editor_layer(ConfigLayer::User, "/user/config.toml", 4),
        ];

        let store = merge_layers(&layers, &schema);

        // Workspace (priority 5) is highest, should win regardless of input order
        assert_eq!(
            store.get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(1))
        );
        assert_eq!(
            store.get("editor.tab_size").unwrap().provenance.layer,
            ConfigLayer::Workspace
        );
    }

    // Validates: Requirement 2.3 — Provenance for keys only defined in one layer
    #[test]
    fn provenance_for_key_from_single_layer() {
        let schema = SchemaRegistry::new();

        let mut logging = ConfigTable::new();
        logging.insert(
            "level".to_string(),
            ConfigValue::String("debug".to_string()),
        );
        let mut values = ConfigTable::new();
        values.insert("logging".to_string(), ConfigValue::Table(logging));

        let layers = vec![make_layer(
            ConfigLayer::System,
            "/etc/ffworkbench/config.toml",
            values,
        )];

        let store = merge_layers(&layers, &schema);

        let effective = store.get("logging.level").expect("key should exist");
        assert_eq!(effective.provenance.layer, ConfigLayer::System);
        assert_eq!(
            effective.provenance.source_file,
            Some(PathBuf::from("/etc/ffworkbench/config.toml"))
        );
    }

    // Validates: Requirement 2.5 — Schema default provenance has no source file
    #[test]
    fn schema_default_provenance_has_no_source_file() {
        let mut schema = SchemaRegistry::new();
        schema
            .register(make_schema_entry(
                "editor.font_size",
                ConfigValue::Integer(14),
            ))
            .unwrap();

        let layers: Vec<LayerData> = vec![];
        let store = merge_layers(&layers, &schema);

        let effective = store.get("editor.font_size").expect("default should apply");
        assert_eq!(effective.provenance.layer, ConfigLayer::Defaults);
        assert_eq!(effective.provenance.source_file, None);
    }
}
