//! Provenance tracking for configuration values.
//!
//! Records which layer and source file provided an effective value,
//! enabling users and subsystems to understand where a setting came from.

use std::path::PathBuf;

use crate::layer::ConfigLayer;
use crate::value::ConfigValue;

/// Metadata indicating the origin of an effective configuration value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// The configuration layer that provided the value.
    pub layer: ConfigLayer,
    /// The filesystem path of the source file, or `None` for hardcoded defaults.
    pub source_file: Option<PathBuf>,
}

/// A configuration value together with its provenance information.
///
/// Returned by `get_with_provenance()` to provide both the effective value
/// and metadata about which layer and file supplied it.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveValue {
    /// The effective value after layer merging.
    pub value: ConfigValue,
    /// Where this value came from.
    pub provenance: Provenance,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_construction_with_source_file() {
        // Validates: Requirement 2.3 — Provenance records layer and source file
        let provenance = Provenance {
            layer: ConfigLayer::User,
            source_file: Some(PathBuf::from("/home/user/.config/ffworkbench/config.toml")),
        };
        assert_eq!(provenance.layer, ConfigLayer::User);
        assert_eq!(
            provenance.source_file,
            Some(PathBuf::from("/home/user/.config/ffworkbench/config.toml"))
        );
    }

    #[test]
    fn provenance_construction_defaults_layer_no_source_file() {
        // Validates: Requirement 2.3 — Defaults layer has no source file
        let provenance = Provenance {
            layer: ConfigLayer::Defaults,
            source_file: None,
        };
        assert_eq!(provenance.layer, ConfigLayer::Defaults);
        assert_eq!(provenance.source_file, None);
    }

    #[test]
    fn effective_value_construction() {
        // Validates: Requirement 2.3 — EffectiveValue combines value and provenance
        let effective = EffectiveValue {
            value: ConfigValue::Integer(42),
            provenance: Provenance {
                layer: ConfigLayer::Project,
                source_file: Some(PathBuf::from(".ffworkbench/config.toml")),
            },
        };
        assert_eq!(effective.value, ConfigValue::Integer(42));
        assert_eq!(effective.provenance.layer, ConfigLayer::Project);
    }

    #[test]
    fn effective_value_equality() {
        // Validates: Requirement 2.3 — EffectiveValues with same content are equal
        let a = EffectiveValue {
            value: ConfigValue::Boolean(true),
            provenance: Provenance {
                layer: ConfigLayer::User,
                source_file: None,
            },
        };
        let b = EffectiveValue {
            value: ConfigValue::Boolean(true),
            provenance: Provenance {
                layer: ConfigLayer::User,
                source_file: None,
            },
        };
        assert_eq!(a, b);
    }
}
