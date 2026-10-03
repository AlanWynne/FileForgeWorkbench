//! Value commit + constraint validation for the Config View
//! (configuration-system Requirement 15.4, 15.5). Split out of
//! `config_panel/render.rs` for the 400-line rule; behaviour unchanged.

use ff_config::value::ConfigValue;
use ff_config::ConfigHandle;

use super::ConfigPanelState;

/// Commit a new value: validate against schema constraints, then write to user layer.
///
/// Validates: Requirement 15.4, 15.5
pub(super) fn commit_value(
    state: &mut ConfigPanelState,
    config: &ConfigHandle,
    key: &str,
    value: ConfigValue,
) {
    // Validate against schema constraints if available.
    if let Some(entry) = config
        .list_schema_entries()
        .into_iter()
        .find(|e| e.key == key)
    {
        if let Some(ref c) = entry.constraints {
            if let Some(err) = validate_against_constraints(&value, c) {
                state.errors.insert(key.to_string(), err);
                return;
            }
        }
    }
    state.errors.remove(key);
    let _ = config.set_user_value(key, value);
}

/// Validate a value against schema constraints. Returns Some(error message) on failure.
pub(super) fn validate_against_constraints(
    value: &ConfigValue,
    constraints: &ff_config::schema::Constraints,
) -> Option<String> {
    match value {
        ConfigValue::Integer(i) => {
            if let Some(min) = constraints.min {
                if (*i as f64) < min {
                    return Some(format!("Must be >= {min}"));
                }
            }
            if let Some(max) = constraints.max {
                if (*i as f64) > max {
                    return Some(format!("Must be <= {max}"));
                }
            }
            if let Some(ref allowed) = constraints.allowed_values {
                let ok = allowed
                    .iter()
                    .any(|v| matches!(v, ConfigValue::Integer(n) if n == i));
                if !ok {
                    return Some("Value not in allowed set".to_string());
                }
            }
        }
        ConfigValue::Float(f) => {
            if let Some(min) = constraints.min {
                if *f < min {
                    return Some(format!("Must be >= {min}"));
                }
            }
            if let Some(max) = constraints.max {
                if *f > max {
                    return Some(format!("Must be <= {max}"));
                }
            }
        }
        ConfigValue::String(s) => {
            if let Some(ref allowed) = constraints.allowed_values {
                let ok = allowed
                    .iter()
                    .any(|v| matches!(v, ConfigValue::String(a) if a == s));
                if !ok {
                    return Some("Value not in allowed set".to_string());
                }
            }
            if let Some(ref pattern) = constraints.pattern {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if !re.is_match(s) {
                        return Some(format!("Must match pattern: {pattern}"));
                    }
                }
            }
        }
        _ => {}
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 15.4 -- valid value calls set_user_value (constraint check passes)
    #[test]
    fn valid_value_passes_constraint_check() {
        use ff_config::schema::Constraints;
        use ff_config::value::ConfigValue;
        let constraints = Constraints {
            min: Some(1.0),
            max: Some(16.0),
            allowed_values: None,
            pattern: None,
        };
        let result = validate_against_constraints(&ConfigValue::Integer(8), &constraints);
        assert!(result.is_none(), "valid value should pass constraints");
    }

    // Validates: Requirement 15.5 -- invalid value shows error (constraint check fails)
    #[test]
    fn invalid_value_shows_error() {
        use ff_config::schema::Constraints;
        use ff_config::value::ConfigValue;
        let constraints = Constraints {
            min: Some(1.0),
            max: Some(16.0),
            allowed_values: None,
            pattern: None,
        };
        let result = validate_against_constraints(&ConfigValue::Integer(99), &constraints);
        assert!(
            result.is_some(),
            "out-of-range value should fail constraints"
        );
        assert!(result.unwrap().contains("<="));
    }

    // Validates: Requirement 15.5 -- string pattern validation fails correctly
    #[test]
    fn string_pattern_validation_fails_for_non_matching_value() {
        use ff_config::schema::Constraints;
        use ff_config::value::ConfigValue;
        let constraints = Constraints {
            min: None,
            max: None,
            allowed_values: None,
            pattern: Some("^(info|warn|error|debug)$".to_string()),
        };
        let result =
            validate_against_constraints(&ConfigValue::String("invalid".to_string()), &constraints);
        assert!(result.is_some(), "non-matching pattern should fail");
    }

    // Validates: Requirement 15.5 -- string pattern validation passes for matching value
    #[test]
    fn string_pattern_validation_passes_for_matching_value() {
        use ff_config::schema::Constraints;
        use ff_config::value::ConfigValue;
        let constraints = Constraints {
            min: None,
            max: None,
            allowed_values: None,
            pattern: Some("^(info|warn|error|debug)$".to_string()),
        };
        let result =
            validate_against_constraints(&ConfigValue::String("info".to_string()), &constraints);
        assert!(result.is_none(), "matching pattern should pass");
    }
}
