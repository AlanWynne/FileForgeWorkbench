"""Append locked-key merger tests to merger.rs (Task 32.8)."""
import sys

TARGET = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-config\src\merger.rs"

# The last test ends with this exact byte sequence (LF line endings)
ANCHOR = b"        assert_eq!(effective.provenance.source_file, None);\n    }\n}"

REPLACEMENT = b"""        assert_eq!(effective.provenance.source_file, None);
    }

    // Validates: Requirement 18.2 -- locked key uses system-layer value despite user override
    #[test]
    fn locked_key_uses_system_value_despite_user_override() {
        let schema = SchemaRegistry::new();

        let mut sys_editor = ConfigTable::new();
        sys_editor.insert("tab_size".to_string(), ConfigValue::Integer(4));
        let mut sys_values = ConfigTable::new();
        sys_values.insert("editor".to_string(), ConfigValue::Table(sys_editor));

        let mut user_editor = ConfigTable::new();
        user_editor.insert("tab_size".to_string(), ConfigValue::Integer(8));
        let mut user_values = ConfigTable::new();
        user_values.insert("editor".to_string(), ConfigValue::Table(user_editor));

        let layers = vec![
            make_layer(ConfigLayer::System, "/etc/config.toml", sys_values),
            make_layer(ConfigLayer::User, "/home/user/config.toml", user_values),
        ];

        let mut locked = HashSet::new();
        locked.insert("editor.tab_size".to_string());

        let store = merge_layers_with_locked(&layers, &schema, &locked);

        assert_eq!(
            store.get_value("editor.tab_size"),
            Some(&ConfigValue::Integer(4)),
            "Locked key must use system-layer value"
        );
        assert_eq!(
            store.get("editor.tab_size").unwrap().provenance.layer,
            ConfigLayer::System
        );
    }

    // Validates: Requirement 18.4 -- higher-layer value silently ignored for locked key
    #[test]
    fn higher_layer_value_silently_ignored_for_locked_key() {
        let schema = SchemaRegistry::new();

        let mut sys_log = ConfigTable::new();
        sys_log.insert("level".to_string(), ConfigValue::String("warn".to_string()));
        let mut sys_values = ConfigTable::new();
        sys_values.insert("logging".to_string(), ConfigValue::Table(sys_log));

        let mut proj_log = ConfigTable::new();
        proj_log.insert("level".to_string(), ConfigValue::String("debug".to_string()));
        let mut proj_values = ConfigTable::new();
        proj_values.insert("logging".to_string(), ConfigValue::Table(proj_log));

        let layers = vec![
            make_layer(ConfigLayer::System, "/etc/config.toml", sys_values),
            make_layer(ConfigLayer::Project, "/proj/config.toml", proj_values),
        ];

        let mut locked = HashSet::new();
        locked.insert("logging.level".to_string());

        let store = merge_layers_with_locked(&layers, &schema, &locked);

        assert_eq!(
            store.get_value("logging.level"),
            Some(&ConfigValue::String("warn".to_string())),
            "Project override must be suppressed for locked key"
        );
    }

    // Validates: Requirement 18.2 -- unlocked keys are unaffected by locked set
    #[test]
    fn unlocked_keys_unaffected_by_locked_set() {
        let schema = SchemaRegistry::new();

        let mut sys_editor = ConfigTable::new();
        sys_editor.insert("tab_size".to_string(), ConfigValue::Integer(4));
        sys_editor.insert("word_wrap".to_string(), ConfigValue::Boolean(false));
        let mut sys_values = ConfigTable::new();
        sys_values.insert("editor".to_string(), ConfigValue::Table(sys_editor));

        let mut user_editor = ConfigTable::new();
        user_editor.insert("tab_size".to_string(), ConfigValue::Integer(8));
        user_editor.insert("word_wrap".to_string(), ConfigValue::Boolean(true));
        let mut user_values = ConfigTable::new();
        user_values.insert("editor".to_string(), ConfigValue::Table(user_editor));

        let layers = vec![
            make_layer(ConfigLayer::System, "/etc/config.toml", sys_values),
            make_layer(ConfigLayer::User, "/home/user/config.toml", user_values),
        ];

        let mut locked = HashSet::new();
        locked.insert("editor.tab_size".to_string());

        let store = merge_layers_with_locked(&layers, &schema, &locked);

        assert_eq!(store.get_value("editor.tab_size"), Some(&ConfigValue::Integer(4)));
        assert_eq!(store.get_value("editor.word_wrap"), Some(&ConfigValue::Boolean(true)));
    }

    // Validates: Requirement 18.2 -- empty locked set behaves identically to merge_layers
    #[test]
    fn empty_locked_set_behaves_like_normal_merge() {
        let schema = SchemaRegistry::new();

        let mut user_editor = ConfigTable::new();
        user_editor.insert("tab_size".to_string(), ConfigValue::Integer(8));
        let mut user_values = ConfigTable::new();
        user_values.insert("editor".to_string(), ConfigValue::Table(user_editor));

        let layers = vec![make_layer(
            ConfigLayer::User,
            "/home/user/config.toml",
            user_values,
        )];

        let normal = merge_layers(&layers, &schema);
        let with_empty = merge_layers_with_locked(&layers, &schema, &HashSet::new());

        assert_eq!(
            normal.get_value("editor.tab_size"),
            with_empty.get_value("editor.tab_size")
        );
    }
}"""

with open(TARGET, "rb") as f:
    data = f.read()

print(f"File size: {len(data)} bytes", flush=True)

if ANCHOR in data:
    print("Anchor found", flush=True)
    data = data.replace(ANCHOR, REPLACEMENT, 1)
    with open(TARGET, "wb") as f:
        f.write(data)
    print("Merger tests appended successfully", flush=True)
else:
    print("ERROR: anchor not found", flush=True)
    sys.exit(1)
