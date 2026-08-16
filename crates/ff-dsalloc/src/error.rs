//! Error types for the dataset allocator.
//!
//! Defines `JclResolverError` — the primary error enum for all allocator operations.
//! Each variant carries sufficient context (line number, ddname, DSN, reason) for
//! diagnostics and error reporting.

/// Errors produced by the dataset allocator.
///
/// Each variant includes enough context to produce a meaningful diagnostic
/// message identifying the location and nature of the problem.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum JclResolverError {
    /// JCL syntax error during parsing.
    #[error("[jcl] parse: syntax error at line {line}: {description}")]
    SyntaxError {
        /// Line number in JCL source (1-based).
        line: usize,
        /// DD name if identifiable.
        ddname: Option<String>,
        /// Description of the syntax problem.
        description: String,
    },

    /// DSN not found in any mounted catalog.
    #[error("[jcl] resolve: dataset not found: {dsn} (line {line})")]
    DatasetNotFound {
        /// The dataset name that could not be resolved.
        dsn: String,
        /// Line number in JCL source.
        line: usize,
        /// DD name containing the reference.
        ddname: String,
    },

    /// Unresolved symbolic parameter.
    #[error("[jcl] substitute: unresolved symbolic &{symbol} at line {line}")]
    UnresolvedSymbolic {
        /// The symbolic name that could not be resolved.
        symbol: String,
        /// Line number in JCL source.
        line: usize,
    },

    /// DISP conflict (NEW on existing or OLD on non-existent).
    #[error("[jcl] allocate: DISP conflict for {dsn} at line {line}: {description}")]
    DispConflict {
        /// The dataset name involved in the conflict.
        dsn: String,
        /// Line number in JCL source.
        line: usize,
        /// Description of the conflict.
        description: String,
    },

    /// Referback target not found.
    #[error("[jcl] referback: target not found — {description} (line {line})")]
    ReferbackNotFound {
        /// Line number in JCL source.
        line: usize,
        /// Description of the missing target.
        description: String,
    },

    /// Referback chain exceeded depth limit.
    #[error("[jcl] referback: chain too deep at line {line} (limit: {limit})")]
    ReferbackChainTooDeep {
        /// Line number in JCL source.
        line: usize,
        /// Maximum allowed depth.
        limit: usize,
    },

    /// GDG base not defined or generation not available.
    #[error("[jcl] gdg: {description} (line {line})")]
    GdgError {
        /// Line number in JCL source.
        line: usize,
        /// GDG base name.
        base_name: String,
        /// Description of the GDG problem.
        description: String,
    },

    /// Temporary dataset not created in prior step.
    #[error("[jcl] temporary: &&{name} not created in prior step (line {line})")]
    TemporaryNotFound {
        /// Temporary dataset name (without && prefix).
        name: String,
        /// Line number in JCL source.
        line: usize,
    },

    /// Catalog query failed (database/I/O error).
    #[error("[jcl] catalog: query failed for {catalog_name}: {detail}")]
    CatalogQueryFailed {
        /// Name of the catalog that failed.
        catalog_name: String,
        /// Detail of the failure.
        detail: String,
    },

    /// Invalid DSN syntax.
    #[error("[jcl] validate: invalid DSN syntax: {dsn} — {reason}")]
    InvalidDsnSyntax {
        /// The invalid dataset name.
        dsn: String,
        /// Reason the name is invalid.
        reason: String,
    },

    /// Configuration error.
    #[error("[jcl] config: {description}")]
    ConfigError {
        /// Description of the configuration problem.
        description: String,
    },

    /// Active document is not a JCL file.
    #[error("[jcl] resolve: active document is not a JCL file")]
    NotJclFile,

    /// Internal error (should not occur in normal operation).
    #[error("[jcl] internal: {description}")]
    InternalError {
        /// Description of the internal problem.
        description: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_error_display_includes_line_and_description() {
        // Validates: Requirement 15 AC 1
        let err = JclResolverError::SyntaxError {
            line: 5,
            ddname: Some("SYSUT1".to_string()),
            description: "unbalanced parentheses".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("line 5"));
        assert!(msg.contains("unbalanced parentheses"));
    }

    #[test]
    fn dataset_not_found_display_includes_dsn_and_line() {
        // Validates: Requirement 15 AC 1
        let err = JclResolverError::DatasetNotFound {
            dsn: "MY.DATA.SET".to_string(),
            line: 10,
            ddname: "INPUT".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("MY.DATA.SET"));
        assert!(msg.contains("line 10"));
    }

    #[test]
    fn unresolved_symbolic_display_includes_symbol_name() {
        // Validates: Requirement 15 AC 1
        let err = JclResolverError::UnresolvedSymbolic {
            symbol: "SYSPARM".to_string(),
            line: 3,
        };
        let msg = err.to_string();
        assert!(msg.contains("&SYSPARM"));
        assert!(msg.contains("line 3"));
    }

    #[test]
    fn all_error_variants_carry_context() {
        // Validates: Requirement 15 AC 1, AC 6
        let errors: Vec<JclResolverError> = vec![
            JclResolverError::SyntaxError {
                line: 1,
                ddname: None,
                description: "test".into(),
            },
            JclResolverError::DatasetNotFound {
                dsn: "A.B".into(),
                line: 2,
                ddname: "DD1".into(),
            },
            JclResolverError::UnresolvedSymbolic {
                symbol: "X".into(),
                line: 3,
            },
            JclResolverError::DispConflict {
                dsn: "A.B".into(),
                line: 4,
                description: "dup".into(),
            },
            JclResolverError::ReferbackNotFound {
                line: 5,
                description: "step".into(),
            },
            JclResolverError::ReferbackChainTooDeep { line: 6, limit: 10 },
            JclResolverError::GdgError {
                line: 7,
                base_name: "G.B".into(),
                description: "gen".into(),
            },
            JclResolverError::TemporaryNotFound {
                name: "TEMP".into(),
                line: 8,
            },
            JclResolverError::CatalogQueryFailed {
                catalog_name: "CAT1".into(),
                detail: "io".into(),
            },
            JclResolverError::InvalidDsnSyntax {
                dsn: "BAD".into(),
                reason: "short".into(),
            },
            JclResolverError::ConfigError {
                description: "bad key".into(),
            },
            JclResolverError::NotJclFile,
            JclResolverError::InternalError {
                description: "unexpected".into(),
            },
        ];

        for err in &errors {
            let msg = err.to_string();
            assert!(!msg.is_empty(), "Error display must not be empty");
            assert!(
                msg.starts_with("[jcl]"),
                "Error must start with [jcl] prefix: {msg}"
            );
        }
    }
}
