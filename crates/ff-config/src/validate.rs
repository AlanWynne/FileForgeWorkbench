//! Schema validation engine.
//!
//! Validates configuration values against their schema entries, applying
//! type checks, numeric range constraints, enum validation, and regex
//! pattern matching.
//!
//! Addresses: Requirement 7 (AC 7.4, 7.5, 7.6), Requirement 9 (AC 9.4, 9.6)

use regex::Regex;

use crate::error::ValueType;
use crate::schema::{Constraints, SchemaEntry, SchemaRegistry};
use crate::value::{ConfigTable, ConfigValue};

/// Result of validating a single value against its schema entry.
#[derive(Debug)]
pub enum ValidationResult {
    /// Value passed all validations.
    Valid(ConfigValue),
    /// Value failed validation; the schema default should be used instead.
    /// Contains the reason for the failure.
    DefaultApplied {
        /// Human-readable description of why validation failed.
        reason: String,
        /// The schema-defined default value to use instead.
        default: ConfigValue,
    },
}

/// Validate a single value against a schema entry.
///
/// Checks: type match → numeric range → allowed values → regex pattern.
/// If any check fails, returns `DefaultApplied` with the schema's default.
///
/// # Arguments
///
/// * `value` - The configuration value to validate.
/// * `entry` - The schema entry defining expected type and constraints.
pub fn validate_value(value: &ConfigValue, entry: &SchemaEntry) -> ValidationResult {
    // Step 1: Type validation
    let actual_type = value_type_of(value);
    if actual_type != entry.value_type {
        return ValidationResult::DefaultApplied {
            reason: format!(
                "type mismatch: expected {}, found {}",
                entry.value_type, actual_type
            ),
            default: entry.default.clone(),
        };
    }

    // Step 2: Constraint validation (if constraints exist)
    if let Some(ref constraints) = entry.constraints {
        if let Some(reason) = check_constraints(value, constraints) {
            return ValidationResult::DefaultApplied {
                reason,
                default: entry.default.clone(),
            };
        }
    }

    ValidationResult::Valid(value.clone())
}

/// Determine the `ValueType` of a `ConfigValue`.
pub fn value_type_of(value: &ConfigValue) -> ValueType {
    match value {
        ConfigValue::String(_) => ValueType::String,
        ConfigValue::Integer(_) => ValueType::Integer,
        ConfigValue::Float(_) => ValueType::Float,
        ConfigValue::Boolean(_) => ValueType::Boolean,
        ConfigValue::Array(_) => ValueType::Array,
        ConfigValue::Table(_) => ValueType::Table,
    }
}

/// Check constraints. Returns `Some(reason)` if a constraint is violated.
fn check_constraints(value: &ConfigValue, constraints: &Constraints) -> Option<String> {
    // Numeric range: min
    if let Some(min) = constraints.min {
        match value {
            ConfigValue::Integer(i) => {
                if (*i as f64) < min {
                    return Some(format!("value {} is below minimum {}", i, min));
                }
            }
            ConfigValue::Float(f) if *f < min => {
                return Some(format!("value {} is below minimum {}", f, min));
            }
            _ => {}
        }
    }

    // Numeric range: max
    if let Some(max) = constraints.max {
        match value {
            ConfigValue::Integer(i) => {
                if (*i as f64) > max {
                    return Some(format!("value {} exceeds maximum {}", i, max));
                }
            }
            ConfigValue::Float(f) if *f > max => {
                return Some(format!("value {} exceeds maximum {}", f, max));
            }
            _ => {}
        }
    }

    // Allowed values (enum validation)
    if let Some(ref allowed) = constraints.allowed_values {
        if !allowed.contains(value) {
            return Some("value is not in allowed set".to_string());
        }
    }

    // Regex pattern (for strings)
    if let Some(ref pattern) = constraints.pattern {
        if let ConfigValue::String(s) = value {
            match Regex::new(pattern) {
                Ok(re) => {
                    if !re.is_match(s) {
                        return Some(format!(
                            "value '{}' does not match pattern '{}'",
                            s, pattern
                        ));
                    }
                }
                Err(_) => {
                    return Some(format!("invalid regex pattern: '{}'", pattern));
                }
            }
        }
    }

    None
}

/// Validate an entire `ConfigTable` against the schema registry.
///
/// For each key in the table:
/// - If the key has a schema entry: validate the value
/// - If the key has no schema entry: emit DEBUG log (unknown key), pass through
///
/// Returns the validated table with invalid values replaced by defaults.
///
/// # Arguments
///
/// * `table` - The configuration table to validate.
/// * `schema` - The schema registry containing known key definitions.
/// * `namespace_prefix` - Dot-separated prefix for building fully-qualified keys.
pub fn validate_table(
    table: &ConfigTable,
    schema: &SchemaRegistry,
    namespace_prefix: &str,
) -> ConfigTable {
    let mut result = ConfigTable::new();

    for (key, value) in table {
        let full_key = if namespace_prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", namespace_prefix, key)
        };

        match value {
            ConfigValue::Table(sub_table) => {
                // Recurse into sub-tables
                let validated_sub = validate_table(sub_table, schema, &full_key);
                if !validated_sub.is_empty() {
                    result.insert(key.clone(), ConfigValue::Table(validated_sub));
                }
            }
            _ => {
                if let Some(entry) = schema.get(&full_key) {
                    match validate_value(value, entry) {
                        ValidationResult::Valid(v) => {
                            result.insert(key.clone(), v);
                        }
                        ValidationResult::DefaultApplied { reason, default } => {
                            ff_logging::log_warn!(
                                "[config] validation: key \"{}\": {} — applying default",
                                full_key,
                                reason
                            );
                            result.insert(key.clone(), default);
                        }
                    }
                } else {
                    // Unknown key — emit DEBUG and include as-is
                    ff_logging::log_debug!(
                        "[config] unknown key \"{}\" — no schema entry, passing through",
                        full_key
                    );
                    result.insert(key.clone(), value.clone());
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SchemaEntry;

    /// Helper: create a schema entry with no constraints.
    fn entry_no_constraints(key: &str, value_type: ValueType, default: ConfigValue) -> SchemaEntry {
        SchemaEntry {
            key: key.to_string(),
            value_type,
            default,
            description: format!("Test entry for {key}"),
            constraints: None,
        }
    }

    /// Helper: create a schema entry with constraints.
    fn entry_with_constraints(
        key: &str,
        value_type: ValueType,
        default: ConfigValue,
        constraints: Constraints,
    ) -> SchemaEntry {
        SchemaEntry {
            key: key.to_string(),
            value_type,
            default,
            description: format!("Test entry for {key}"),
            constraints: Some(constraints),
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 7.1: Type validation
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.4 — type mismatch detected
    #[test]
    fn type_mismatch_string_expected_integer_returns_default() {
        let entry = entry_no_constraints(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
        );
        let value = ConfigValue::String("not a number".to_string());

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::DefaultApplied { reason, default } => {
                assert!(reason.contains("type mismatch"));
                assert!(reason.contains("integer"));
                assert!(reason.contains("string"));
                assert_eq!(default, ConfigValue::Integer(4));
            }
            ValidationResult::Valid(_) => panic!("expected DefaultApplied"),
        }
    }

    // Validates: Requirement 9.4 — correct type passes validation
    #[test]
    fn correct_type_passes_validation() {
        let entry = entry_no_constraints(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
        );
        let value = ConfigValue::Integer(8);

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::Valid(v) => assert_eq!(v, ConfigValue::Integer(8)),
            ValidationResult::DefaultApplied { .. } => panic!("expected Valid"),
        }
    }

    // Validates: Requirement 9.4 — boolean where string expected
    #[test]
    fn type_mismatch_boolean_expected_string_returns_default() {
        let entry = entry_no_constraints(
            "theme.active",
            ValueType::String,
            ConfigValue::String("dark".to_string()),
        );
        let value = ConfigValue::Boolean(true);

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::DefaultApplied { reason, default } => {
                assert!(reason.contains("type mismatch"));
                assert_eq!(default, ConfigValue::String("dark".to_string()));
            }
            ValidationResult::Valid(_) => panic!("expected DefaultApplied"),
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 7.2: Numeric range validation (min/max)
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 7.4, 9.4 — integer below minimum
    #[test]
    fn integer_below_minimum_returns_default() {
        let entry = entry_with_constraints(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
            Constraints {
                min: Some(1.0),
                max: Some(16.0),
                allowed_values: None,
                pattern: None,
            },
        );
        let value = ConfigValue::Integer(0);

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::DefaultApplied { reason, default } => {
                assert!(reason.contains("below minimum"));
                assert_eq!(default, ConfigValue::Integer(4));
            }
            ValidationResult::Valid(_) => panic!("expected DefaultApplied"),
        }
    }

    // Validates: Requirement 7.4, 9.4 — integer above maximum
    #[test]
    fn integer_above_maximum_returns_default() {
        let entry = entry_with_constraints(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
            Constraints {
                min: Some(1.0),
                max: Some(16.0),
                allowed_values: None,
                pattern: None,
            },
        );
        let value = ConfigValue::Integer(32);

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::DefaultApplied { reason, default } => {
                assert!(reason.contains("exceeds maximum"));
                assert_eq!(default, ConfigValue::Integer(4));
            }
            ValidationResult::Valid(_) => panic!("expected DefaultApplied"),
        }
    }

    // Validates: Requirement 7.4 — integer within range passes
    #[test]
    fn integer_within_range_passes() {
        let entry = entry_with_constraints(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
            Constraints {
                min: Some(1.0),
                max: Some(16.0),
                allowed_values: None,
                pattern: None,
            },
        );
        let value = ConfigValue::Integer(8);

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::Valid(v) => assert_eq!(v, ConfigValue::Integer(8)),
            ValidationResult::DefaultApplied { .. } => panic!("expected Valid"),
        }
    }

    // Validates: Requirement 7.4 — float below minimum
    #[test]
    fn float_below_minimum_returns_default() {
        let entry = entry_with_constraints(
            "theme.font_size",
            ValueType::Float,
            ConfigValue::Float(12.0),
            Constraints {
                min: Some(6.0),
                max: Some(72.0),
                allowed_values: None,
                pattern: None,
            },
        );
        let value = ConfigValue::Float(3.5);

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::DefaultApplied { reason, default } => {
                assert!(reason.contains("below minimum"));
                assert_eq!(default, ConfigValue::Float(12.0));
            }
            ValidationResult::Valid(_) => panic!("expected DefaultApplied"),
        }
    }

    // Validates: Requirement 7.4 — float above maximum
    #[test]
    fn float_above_maximum_returns_default() {
        let entry = entry_with_constraints(
            "theme.font_size",
            ValueType::Float,
            ConfigValue::Float(12.0),
            Constraints {
                min: Some(6.0),
                max: Some(72.0),
                allowed_values: None,
                pattern: None,
            },
        );
        let value = ConfigValue::Float(100.5);

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::DefaultApplied { reason, default } => {
                assert!(reason.contains("exceeds maximum"));
                assert_eq!(default, ConfigValue::Float(12.0));
            }
            ValidationResult::Valid(_) => panic!("expected DefaultApplied"),
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 7.3: Enum validation (allowed_values)
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 7.4, 9.4 — string not in allowed set
    #[test]
    fn string_not_in_allowed_values_returns_default() {
        let entry = entry_with_constraints(
            "editor.indent_style",
            ValueType::String,
            ConfigValue::String("space".to_string()),
            Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("space".to_string()),
                    ConfigValue::String("tab".to_string()),
                ]),
                pattern: None,
            },
        );
        let value = ConfigValue::String("mixed".to_string());

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::DefaultApplied { reason, default } => {
                assert!(reason.contains("not in allowed set"));
                assert_eq!(default, ConfigValue::String("space".to_string()));
            }
            ValidationResult::Valid(_) => panic!("expected DefaultApplied"),
        }
    }

    // Validates: Requirement 7.4 — string in allowed set passes
    #[test]
    fn string_in_allowed_values_passes() {
        let entry = entry_with_constraints(
            "editor.indent_style",
            ValueType::String,
            ConfigValue::String("space".to_string()),
            Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("space".to_string()),
                    ConfigValue::String("tab".to_string()),
                ]),
                pattern: None,
            },
        );
        let value = ConfigValue::String("tab".to_string());

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::Valid(v) => {
                assert_eq!(v, ConfigValue::String("tab".to_string()));
            }
            ValidationResult::DefaultApplied { .. } => panic!("expected Valid"),
        }
    }

    // Validates: Requirement 7.4 — integer not in allowed set
    #[test]
    fn integer_not_in_allowed_values_returns_default() {
        let entry = entry_with_constraints(
            "logging.max_retained_files",
            ValueType::Integer,
            ConfigValue::Integer(5),
            Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::Integer(1),
                    ConfigValue::Integer(5),
                    ConfigValue::Integer(10),
                ]),
                pattern: None,
            },
        );
        let value = ConfigValue::Integer(7);

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::DefaultApplied { reason, default } => {
                assert!(reason.contains("not in allowed set"));
                assert_eq!(default, ConfigValue::Integer(5));
            }
            ValidationResult::Valid(_) => panic!("expected DefaultApplied"),
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 7.4: Regex pattern validation
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 7.4 — string fails regex
    #[test]
    fn string_failing_regex_returns_default() {
        let entry = entry_with_constraints(
            "logging.level",
            ValueType::String,
            ConfigValue::String("info".to_string()),
            Constraints {
                min: None,
                max: None,
                allowed_values: None,
                pattern: Some("^(trace|debug|info|warn|error)$".to_string()),
            },
        );
        let value = ConfigValue::String("invalid_level".to_string());

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::DefaultApplied { reason, default } => {
                assert!(reason.contains("does not match pattern"));
                assert_eq!(default, ConfigValue::String("info".to_string()));
            }
            ValidationResult::Valid(_) => panic!("expected DefaultApplied"),
        }
    }

    // Validates: Requirement 7.4 — string passes regex
    #[test]
    fn string_matching_regex_passes() {
        let entry = entry_with_constraints(
            "logging.level",
            ValueType::String,
            ConfigValue::String("info".to_string()),
            Constraints {
                min: None,
                max: None,
                allowed_values: None,
                pattern: Some("^(trace|debug|info|warn|error)$".to_string()),
            },
        );
        let value = ConfigValue::String("warn".to_string());

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::Valid(v) => {
                assert_eq!(v, ConfigValue::String("warn".to_string()));
            }
            ValidationResult::DefaultApplied { .. } => panic!("expected Valid"),
        }
    }

    // Validates: Requirement 7.4 — invalid regex pattern returns default
    #[test]
    fn invalid_regex_pattern_returns_default() {
        let entry = entry_with_constraints(
            "custom.pattern_key",
            ValueType::String,
            ConfigValue::String("fallback".to_string()),
            Constraints {
                min: None,
                max: None,
                allowed_values: None,
                pattern: Some("[invalid(regex".to_string()),
            },
        );
        let value = ConfigValue::String("anything".to_string());

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::DefaultApplied { reason, default } => {
                assert!(reason.contains("invalid regex pattern"));
                assert_eq!(default, ConfigValue::String("fallback".to_string()));
            }
            ValidationResult::Valid(_) => panic!("expected DefaultApplied"),
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 7.5: Validation failure handling
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.4 — invalid value discarded, default applied
    #[test]
    fn validation_failure_applies_schema_default() {
        let entry = entry_with_constraints(
            "editor.tab_size",
            ValueType::Integer,
            ConfigValue::Integer(4),
            Constraints {
                min: Some(1.0),
                max: Some(16.0),
                allowed_values: None,
                pattern: None,
            },
        );
        // Value violates max constraint
        let value = ConfigValue::Integer(100);

        let result = validate_value(&value, &entry);
        match result {
            ValidationResult::DefaultApplied { default, .. } => {
                assert_eq!(default, ConfigValue::Integer(4));
            }
            ValidationResult::Valid(_) => {
                panic!("expected DefaultApplied for invalid value")
            }
        }
    }

    // Validates: Requirement 7.5 — validate_table replaces invalid with default
    #[test]
    fn validate_table_replaces_invalid_value_with_default() {
        let mut schema = SchemaRegistry::new();
        schema
            .register(entry_with_constraints(
                "editor.tab_size",
                ValueType::Integer,
                ConfigValue::Integer(4),
                Constraints {
                    min: Some(1.0),
                    max: Some(16.0),
                    allowed_values: None,
                    pattern: None,
                },
            ))
            .unwrap();

        let mut editor_table = ConfigTable::new();
        editor_table.insert("tab_size".to_string(), ConfigValue::Integer(999));

        let mut table = ConfigTable::new();
        table.insert("editor".to_string(), ConfigValue::Table(editor_table));

        let validated = validate_table(&table, &schema, "");

        // The invalid value should be replaced by the default
        let editor = match validated.get("editor") {
            Some(ConfigValue::Table(t)) => t,
            _ => panic!("expected editor table"),
        };
        assert_eq!(editor.get("tab_size"), Some(&ConfigValue::Integer(4)));
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 7.6: Unknown key handling
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.6 — unknown key passed through without error
    #[test]
    fn unknown_key_passed_through_in_validate_table() {
        let schema = SchemaRegistry::new(); // empty schema

        let mut table = ConfigTable::new();
        table.insert(
            "unknown_key".to_string(),
            ConfigValue::String("mystery".to_string()),
        );
        table.insert("another_unknown".to_string(), ConfigValue::Integer(42));

        let validated = validate_table(&table, &schema, "");

        // Unknown keys should be preserved as-is
        assert_eq!(
            validated.get("unknown_key"),
            Some(&ConfigValue::String("mystery".to_string()))
        );
        assert_eq!(
            validated.get("another_unknown"),
            Some(&ConfigValue::Integer(42))
        );
    }

    // Validates: Requirement 9.6 — unknown key does not cause error
    #[test]
    fn unknown_key_with_known_keys_both_handled_correctly() {
        let mut schema = SchemaRegistry::new();
        schema
            .register(entry_no_constraints(
                "logging.level",
                ValueType::String,
                ConfigValue::String("info".to_string()),
            ))
            .unwrap();

        let mut logging_table = ConfigTable::new();
        // Known key with valid value
        logging_table.insert(
            "level".to_string(),
            ConfigValue::String("debug".to_string()),
        );
        // Unknown key
        logging_table.insert("custom_flag".to_string(), ConfigValue::Boolean(true));

        let mut table = ConfigTable::new();
        table.insert("logging".to_string(), ConfigValue::Table(logging_table));

        let validated = validate_table(&table, &schema, "");

        let logging = match validated.get("logging") {
            Some(ConfigValue::Table(t)) => t,
            _ => panic!("expected logging table"),
        };
        // Known key passes validation
        assert_eq!(
            logging.get("level"),
            Some(&ConfigValue::String("debug".to_string()))
        );
        // Unknown key is passed through
        assert_eq!(
            logging.get("custom_flag"),
            Some(&ConfigValue::Boolean(true))
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Task 7.7: Additional unit tests
    // ──────────────────────────────────────────────────────────────────

    // Validates: Requirement 9.4 — value_type_of maps correctly
    #[test]
    fn value_type_of_all_variants() {
        assert_eq!(
            value_type_of(&ConfigValue::String("hi".to_string())),
            ValueType::String
        );
        assert_eq!(value_type_of(&ConfigValue::Integer(1)), ValueType::Integer);
        assert_eq!(value_type_of(&ConfigValue::Float(1.0)), ValueType::Float);
        assert_eq!(
            value_type_of(&ConfigValue::Boolean(true)),
            ValueType::Boolean
        );
        assert_eq!(value_type_of(&ConfigValue::Array(vec![])), ValueType::Array);
        assert_eq!(
            value_type_of(&ConfigValue::Table(ConfigTable::new())),
            ValueType::Table
        );
    }

    // Validates: Requirement 7.5 — valid value passes through unchanged
    #[test]
    fn valid_value_passes_through_unchanged_in_validate_table() {
        let mut schema = SchemaRegistry::new();
        schema
            .register(entry_with_constraints(
                "editor.tab_size",
                ValueType::Integer,
                ConfigValue::Integer(4),
                Constraints {
                    min: Some(1.0),
                    max: Some(16.0),
                    allowed_values: None,
                    pattern: None,
                },
            ))
            .unwrap();

        let mut editor_table = ConfigTable::new();
        editor_table.insert("tab_size".to_string(), ConfigValue::Integer(8));

        let mut table = ConfigTable::new();
        table.insert("editor".to_string(), ConfigValue::Table(editor_table));

        let validated = validate_table(&table, &schema, "");

        let editor = match validated.get("editor") {
            Some(ConfigValue::Table(t)) => t,
            _ => panic!("expected editor table"),
        };
        assert_eq!(editor.get("tab_size"), Some(&ConfigValue::Integer(8)));
    }
}
