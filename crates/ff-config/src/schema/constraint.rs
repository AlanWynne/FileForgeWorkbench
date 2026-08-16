//! Schema constraints.
//!
//! Defines validation constraints that can be attached to schema entries:
//! minimum/maximum values, allowed enum values, and regex patterns.

use crate::value::ConfigValue;

/// Constraints on a configuration value.
///
/// Each field is optional; `None` means that constraint is not applied.
/// Multiple constraints can be active simultaneously (e.g., both min and max
/// for a numeric range).
///
/// # Applicable Types
///
/// - `min` / `max`: Integer and Float values
/// - `allowed_values`: String and Integer enumerated types
/// - `pattern`: String values (validated against a regex)
#[derive(Debug, Clone, Default)]
pub struct Constraints {
    /// Minimum value (for Integer and Float types).
    pub min: Option<f64>,
    /// Maximum value (for Integer and Float types).
    pub max: Option<f64>,
    /// Allowed values (for String and Integer enumerated types).
    pub allowed_values: Option<Vec<ConfigValue>>,
    /// Regex pattern (for String validation).
    pub pattern: Option<String>,
}
