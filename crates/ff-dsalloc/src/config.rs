//! Configuration model for the JCL resolver.
//!
//! Defines `ResolverConfig` with all settings read from the `[jcl]` TOML table.

use std::collections::HashMap;

use serde::Deserialize;

use crate::diagnostic::DiagnosticSeverity;

/// Resolution execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResolveMode {
    /// Report what allocations would occur without creating datasets.
    #[default]
    DryRun,
    /// Perform actual catalog allocations for DISP=NEW.
    Live,
}

/// Configuration for the JCL resolver, read from the `[jcl]` config table.
///
/// # Defaults
///
/// - `resolve_mode`: DryRun
/// - `default_hlq`: None
/// - `catalog_search_order`: None (uses mount order)
/// - `lint_level`: Info (show all diagnostics)
/// - `max_referback_depth`: 10
/// - `symbols`: empty
/// - `auto_resolve`: false
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ResolverConfig {
    /// Resolution mode: "dry-run" (report only) or "live" (perform allocations).
    pub resolve_mode: ResolveMode,

    /// Default high-level qualifier prepended to unqualified DSNs.
    pub default_hlq: Option<String>,

    /// Explicit catalog search order (overrides mount order).
    pub catalog_search_order: Option<Vec<String>>,

    /// Minimum diagnostic severity to report.
    pub lint_level: DiagnosticSeverity,

    /// Maximum referback chain depth before producing an error.
    pub max_referback_depth: usize,

    /// Persistent user-defined symbols from `[jcl.symbols]`.
    pub symbols: HashMap<String, String>,

    /// Auto-resolve on document save.
    pub auto_resolve: bool,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            resolve_mode: ResolveMode::DryRun,
            default_hlq: None,
            catalog_search_order: None,
            lint_level: DiagnosticSeverity::Info,
            max_referback_depth: 10,
            symbols: HashMap::new(),
            auto_resolve: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        // Validates: Requirement 14 AC 1, AC 5
        let config = ResolverConfig::default();
        assert_eq!(config.resolve_mode, ResolveMode::DryRun);
        assert_eq!(config.default_hlq, None);
        assert_eq!(config.catalog_search_order, None);
        assert_eq!(config.lint_level, DiagnosticSeverity::Info);
        assert_eq!(config.max_referback_depth, 10);
        assert!(config.symbols.is_empty());
        assert!(!config.auto_resolve);
    }

    #[test]
    fn config_deserializes_from_toml() {
        // Validates: Requirement 14 AC 1, AC 2
        let toml_str = r#"
            resolve_mode = "live"
            default_hlq = "USER"
            catalog_search_order = ["PROD.CATALOG", "TEST.CATALOG"]
            lint_level = "warning"
            max_referback_depth = 5
            auto_resolve = true

            [symbols]
            SYSPARM = "PROD"
            ENV = "TEST"
        "#;

        let config: ResolverConfig = toml::from_str(toml_str).expect("should parse");
        assert_eq!(config.resolve_mode, ResolveMode::Live);
        assert_eq!(config.default_hlq.as_deref(), Some("USER"));
        assert_eq!(
            config.catalog_search_order,
            Some(vec!["PROD.CATALOG".to_string(), "TEST.CATALOG".to_string()])
        );
        assert_eq!(config.lint_level, DiagnosticSeverity::Warning);
        assert_eq!(config.max_referback_depth, 5);
        assert_eq!(
            config.symbols.get("SYSPARM").map(String::as_str),
            Some("PROD")
        );
        assert_eq!(config.symbols.get("ENV").map(String::as_str), Some("TEST"));
        assert!(config.auto_resolve);
    }

    #[test]
    fn config_partial_toml_uses_defaults_for_missing_fields() {
        // Validates: Requirement 14 AC 1
        let toml_str = r#"
            resolve_mode = "dry-run"
        "#;

        let config: ResolverConfig = toml::from_str(toml_str).expect("should parse");
        assert_eq!(config.resolve_mode, ResolveMode::DryRun);
        assert_eq!(config.max_referback_depth, 10);
        assert!(!config.auto_resolve);
    }
}
