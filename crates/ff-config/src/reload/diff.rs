use crate::store::EffectiveStore;

/// Flatten a nested ConfigTable into dot-path keys.
pub(super) fn flatten_config_table(
    table: &crate::value::ConfigTable,
    prefix: &str,
) -> crate::value::ConfigTable {
    let mut result = crate::value::ConfigTable::new();
    for (k, v) in table {
        let full_key = if prefix.is_empty() {
            k.clone()
        } else {
            format!("{prefix}.{k}")
        };
        match v {
            crate::value::ConfigValue::Table(sub) => {
                let sub_flat = flatten_config_table(sub, &full_key);
                result.extend(sub_flat);
            }
            other => {
                result.insert(full_key, other.clone());
            }
        }
    }
    result
}

/// Compute the set of keys whose values differ between two stores.
///
/// Detects added, changed, and removed keys by comparing old and new stores.
pub(super) fn compute_diff(old: &EffectiveStore, new: &EffectiveStore) -> Vec<String> {
    let mut changed = Vec::new();

    // Check all keys in the new store for additions or changes
    for key in new.keys() {
        match (old.get_value(key), new.get_value(key)) {
            (Some(old_val), Some(new_val)) if old_val != new_val => {
                changed.push(key.clone());
            }
            (None, Some(_)) => {
                changed.push(key.clone()); // New key appeared
            }
            _ => {}
        }
    }

    // Check for keys removed (in old but not in new)
    for key in old.keys() {
        if new.get_value(key).is_none() {
            changed.push(key.clone());
        }
    }

    changed
}
