//! Schema entry definition.
//!
//! Defines `SchemaEntry` — the structured definition for a single
//! configuration key including its type, default, constraints, and
//! human-readable description.

use super::constraint::Constraints;
use crate::error::ValueType;
use crate::value::ConfigValue;

/// A schema definition for a single configuration key.
///
/// Every known configuration key has a corresponding `SchemaEntry` that
/// declares its type, provides a default value, describes its purpose,
/// and optionally specifies validation constraints.
///
/// Schema entries are registered by core subsystems at startup and by
/// plugins during their initialization phase.
#[derive(Debug, Clone)]
pub struct SchemaEntry {
    /// The fully-qualified key path (e.g., `"editor.tab_size"`).
    pub key: String,
    /// The expected value type for this key.
    pub value_type: ValueType,
    /// The default value applied when no layer provides this key.
    pub default: ConfigValue,
    /// Human-readable description of the setting's purpose (for settings UI).
    pub description: String,
    /// Optional validation constraints (min, max, allowed values, pattern).
    pub constraints: Option<Constraints>,
}
