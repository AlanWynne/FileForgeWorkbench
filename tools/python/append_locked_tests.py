"""Append locked-key unit tests to config_handle.rs (Task 32.8)."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\append_locked_tests.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

TARGET = r"C:\workspace\VSC\FileForgeWorkbench\crates\ff-config\src\config_handle.rs"

ANCHOR = b"        // editor table may be absent \xe2\x80\x94 also acceptable\n    }\n}"

TESTS = b"""
        // editor table may be absent \xe2\x80\x94 also acceptable
    }

    // Validates: Requirement 18.5 -- is_locked returns true for locked key
    #[test]
    fn is_locked_returns_true_for_locked_key() {
        let schema = crate::schema::SchemaRegistry::new();
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);

        {
            let mut system = handle.inner.write().unwrap();
            system.locked_keys.insert("editor.tab_size".to_string());
        }

        assert!(handle.is_locked("editor.tab_size"));
        assert!(!handle.is_locked("editor.word_wrap"));
    }

    // Validates: Requirement 18.5 -- is_locked returns false for unlocked key
    #[test]
    fn is_locked_returns_false_for_unlocked_key() {
        let schema = crate::schema::SchemaRegistry::new();
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);
        assert!(!handle.is_locked("editor.tab_size"));
    }

    // Validates: Requirement 18.3 -- set_user_value returns KeyLocked for locked key
    #[test]
    fn set_user_value_locked_key_returns_error() {
        let schema = crate::schema::SchemaRegistry::new();
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);

        {
            let mut system = handle.inner.write().unwrap();
            system.locked_keys.insert("editor.tab_size".to_string());
        }

        let result =
            handle.set_user_value("editor.tab_size", crate::value::ConfigValue::Integer(8));
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::ConfigError::KeyLocked { key } => {
                assert_eq!(key, "editor.tab_size");
            }
            other => panic!("Expected KeyLocked, got: {:?}", other),
        }
    }

    // Validates: Requirement 18.1 -- extract_locked_keys parses [_locked].locked_keys
    #[test]
    fn extract_locked_keys_parses_system_layer_table() {
        let mut locked_table = crate::value::ConfigTable::new();
        locked_table.insert(
            "locked_keys".to_string(),
            crate::value::ConfigValue::Array(vec![
                crate::value::ConfigValue::String("editor.tab_size".to_string()),
                crate::value::ConfigValue::String("logging.level".to_string()),
            ]),
        );
        let mut values = crate::value::ConfigTable::new();
        values.insert(
            "_locked".to_string(),
            crate::value::ConfigValue::Table(locked_table),
        );

        let locked = extract_locked_keys(&values);
        assert!(locked.contains("editor.tab_size"));
        assert!(locked.contains("logging.level"));
        assert_eq!(locked.len(), 2);
    }

    // Validates: Requirement 18.1 -- extract_locked_keys returns empty set when absent
    #[test]
    fn extract_locked_keys_returns_empty_when_absent() {
        let values = crate::value::ConfigTable::new();
        let locked = extract_locked_keys(&values);
        assert!(locked.is_empty());
    }

    // Validates: Requirement 18.8 -- update_locked_keys replaces the locked set
    #[test]
    fn hot_reload_locked_keys_list_recomputes_effective_values() {
        let schema = crate::schema::SchemaRegistry::new();
        let manager = ReloadManager::new(Vec::new(), schema);
        let handle = ConfigHandle::new(manager);

        assert!(!handle.is_locked("editor.tab_size"));

        let mut locked_table = crate::value::ConfigTable::new();
        locked_table.insert(
            "locked_keys".to_string(),
            crate::value::ConfigValue::Array(vec![
                crate::value::ConfigValue::String("editor.tab_size".to_string()),
            ]),
        );
        let mut sys_values = crate::value::ConfigTable::new();
        sys_values.insert(
            "_locked".to_string(),
            crate::value::ConfigValue::Table(locked_table),
        );

        handle.update_locked_keys(&sys_values);
        assert!(handle.is_locked("editor.tab_size"));

        handle.update_locked_keys(&crate::value::ConfigTable::new());
        assert!(!handle.is_locked("editor.tab_size"));
    }
}"""

with open(TARGET, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

if ANCHOR in data:
    log("Anchor found")
    data = data.replace(ANCHOR, TESTS, 1)
    with open(TARGET, "wb") as f:
        f.write(data)
    log("Tests appended successfully")
else:
    log("ERROR: anchor not found -- no change made")
    sys.exit(1)
