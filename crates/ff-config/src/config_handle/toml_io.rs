use crate::layer::ConfigLayer;
use crate::value::ConfigTable;

/// Collect all effective values from the store as a flat ConfigTable.
pub(super) fn collect_all_effective_values(
    access: &crate::access::ConfigAccess,
    _schema: &crate::schema::SchemaRegistry,
) -> ConfigTable {
    access.all_values()
}

/// Collect values from a specific layer in the manager's layer stack.
pub(super) fn collect_layer_values(
    manager: &crate::reload::ReloadManager,
    layer: ConfigLayer,
) -> ConfigTable {
    manager.layer_values(layer)
}

/// Extract the locked_keys set from a system-layer ConfigTable.
///
/// Reads `[_locked].locked_keys` as an array of strings.
/// Returns an empty set if the table or key is absent.
///
/// Validates: Requirement 18.1
pub(super) fn extract_locked_keys(
    values: &crate::value::ConfigTable,
) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    if let Some(crate::value::ConfigValue::Table(locked_table)) = values.get("_locked") {
        if let Some(crate::value::ConfigValue::Array(arr)) = locked_table.get("locked_keys") {
            for item in arr {
                if let crate::value::ConfigValue::String(k) = item {
                    set.insert(k.clone());
                }
            }
        }
    }
    set
}

/// Write a single dot-separated key to a TOML file.
///
/// Reads the existing file (or starts with an empty table if missing),
/// sets the key at the given dot-separated path, and writes the result back.
/// The write is atomic: the file is written to a temp path then renamed.
pub(super) fn write_key_to_toml_file(
    path: &std::path::Path,
    key: &str,
    value: crate::value::ConfigValue,
) -> Result<(), crate::error::ConfigError> {
    let mut root = read_toml_as_value(path)?;
    set_dotted_key(&mut root, key, config_value_to_toml(value));
    write_toml_value(path, &root)
}

/// Remove a single dot-separated key from a TOML file.
///
/// Reads the existing file (or returns Ok if missing), removes the key,
/// and writes the result back. No-op if the key does not exist.
pub(super) fn remove_key_from_toml_file(
    path: &std::path::Path,
    key: &str,
) -> Result<(), crate::error::ConfigError> {
    if !path.exists() {
        return Ok(());
    }
    let mut root = read_toml_as_value(path)?;
    remove_dotted_key(&mut root, key);
    write_toml_value(path, &root)
}

/// Read a TOML file into a `toml::Value::Table`, or return an empty table.
fn read_toml_as_value(path: &std::path::Path) -> Result<toml::Value, crate::error::ConfigError> {
    if !path.exists() {
        return Ok(toml::Value::Table(toml::map::Map::new()));
    }
    let content = std::fs::read_to_string(path).map_err(crate::error::ConfigError::Io)?;
    content
        .parse::<toml::Value>()
        .map_err(|e| crate::error::ConfigError::ParseError {
            path: path.to_path_buf(),
            details: e.to_string(),
        })
}

/// Write a `toml::Value` back to a file, creating parent directories as needed.
fn write_toml_value(
    path: &std::path::Path,
    value: &toml::Value,
) -> Result<(), crate::error::ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(crate::error::ConfigError::Io)?;
    }
    let content = toml::to_string_pretty(value)
        .map_err(|e| crate::error::ConfigError::Io(std::io::Error::other(e.to_string())))?;
    std::fs::write(path, content).map_err(crate::error::ConfigError::Io)
}

/// Set a dot-separated key path in a `toml::Value::Table`, creating intermediate tables.
pub(super) fn set_dotted_key(root: &mut toml::Value, key: &str, value: toml::Value) {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = root;
    for part in &parts[..parts.len() - 1] {
        if let toml::Value::Table(ref mut map) = current {
            let entry = map
                .entry((*part).to_string())
                .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
            current = entry;
        } else {
            return;
        }
    }
    if let toml::Value::Table(ref mut map) = current {
        map.insert(parts[parts.len() - 1].to_string(), value);
    }
}

/// Remove a dot-separated key path from a `toml::Value::Table`.
pub(super) fn remove_dotted_key(root: &mut toml::Value, key: &str) {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = root;
    for part in &parts[..parts.len() - 1] {
        if let toml::Value::Table(ref mut map) = current {
            if let Some(next) = map.get_mut(*part) {
                current = next;
            } else {
                return;
            }
        } else {
            return;
        }
    }
    if let toml::Value::Table(ref mut map) = current {
        map.remove(parts[parts.len() - 1]);
    }
}

/// Convert a `ConfigValue` to a `toml::Value`.
fn config_value_to_toml(value: crate::value::ConfigValue) -> toml::Value {
    match value {
        crate::value::ConfigValue::String(s) => toml::Value::String(s),
        crate::value::ConfigValue::Integer(i) => toml::Value::Integer(i),
        crate::value::ConfigValue::Float(f) => toml::Value::Float(f),
        crate::value::ConfigValue::Boolean(b) => toml::Value::Boolean(b),
        crate::value::ConfigValue::Array(arr) => {
            toml::Value::Array(arr.into_iter().map(config_value_to_toml).collect())
        }
        crate::value::ConfigValue::Table(t) => {
            let mut map = toml::map::Map::new();
            for (k, v) in t {
                map.insert(k, config_value_to_toml(v));
            }
            toml::Value::Table(map)
        }
    }
}
