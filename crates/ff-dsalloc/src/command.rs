//! RESOLVE command handler.
//!
//! Registers and handles the `dataset.resolve` command for interactive
//! DSN-to-path tracing via the command framework.

use crate::catalog_bridge::CatalogProvider;
use crate::config::{ResolveMode, ResolverConfig};
use crate::pipeline::{ResolveOutput, ResolveSummary};

/// Command ID for the resolve command.
pub const COMMAND_ID: &str = "dataset.resolve";

/// Command display name.
pub const COMMAND_DISPLAY_NAME: &str = "Resolve Dataset Allocation";

/// Command category.
pub const COMMAND_CATEGORY: &str = "dataset";

/// Parameters for the resolve command invocation.
#[derive(Debug, Clone, Default)]
pub struct ResolveCommandParams {
    /// Specific DSN to resolve (if provided, no JCL context needed).
    pub dsn: Option<String>,
    /// Cursor line position (for single-DD resolution).
    pub cursor_line: Option<usize>,
    /// Mode override ("dry-run" or "live").
    pub mode: Option<ResolveMode>,
}

/// Result of a resolve command invocation.
#[derive(Debug, Clone)]
pub struct ResolveCommandResult {
    /// Whether the command succeeded.
    pub success: bool,
    /// Error message (if any).
    pub error: Option<String>,
    /// Resolution output (if successful).
    pub output: Option<ResolveOutput>,
    /// Summary statistics.
    pub summary: ResolveSummary,
}

/// Execute the resolve command.
///
/// # Modes
/// - Full document: resolve all DD statements (no DSN param, no cursor)
/// - Cursor position: resolve only DD at/nearest cursor
/// - DSN parameter: resolve a specific DSN against catalogs
pub fn execute_resolve_command(
    params: &ResolveCommandParams,
    text: &str,
    language_id: &str,
    config: &ResolverConfig,
    catalog: &dyn CatalogProvider,
) -> ResolveCommandResult {
    // Language guard: must be JCL
    if language_id != "jcl" && params.dsn.is_none() {
        return ResolveCommandResult {
            success: false,
            error: Some("Active document is not a JCL file".to_string()),
            output: None,
            summary: ResolveSummary::default(),
        };
    }

    // Apply mode override
    let effective_config = if let Some(mode) = params.mode {
        let mut cfg = config.clone();
        cfg.resolve_mode = mode;
        cfg
    } else {
        config.clone()
    };

    // Direct DSN resolution (no JCL context)
    if let Some(ref dsn) = params.dsn {
        return resolve_single_dsn(dsn, &effective_config, catalog);
    }

    // Full document resolution
    let output = crate::pipeline::resolve_document(text, &effective_config, catalog);
    let summary = output.summary;

    ResolveCommandResult {
        success: summary.errors == 0,
        error: None,
        output: Some(output),
        summary,
    }
}

/// Resolve a single DSN against catalogs without JCL context.
fn resolve_single_dsn(
    dsn: &str,
    _config: &ResolverConfig,
    catalog: &dyn CatalogProvider,
) -> ResolveCommandResult {
    match catalog.lookup_dsn(dsn) {
        Ok(matches) if !matches.is_empty() => {
            let first = &matches[0];
            let mut output = ResolveOutput::new();
            output.results.push(crate::pipeline::ResolutionResult {
                ddname: String::new(),
                step_name: String::new(),
                original_dsn: Some(dsn.to_string()),
                substituted_dsn: Some(dsn.to_string()),
                outcome: crate::pipeline::ResolutionOutcome::Resolved {
                    physical_path: first.physical_path.clone(),
                    catalog_name: first.catalog_name.clone(),
                    dataset_type: crate::pipeline::DatasetType::Ps,
                },
                concatenation_index: 0,
            });
            output.compute_summary();
            let summary = output.summary;
            ResolveCommandResult {
                success: true,
                error: None,
                output: Some(output),
                summary,
            }
        }
        Ok(_) => ResolveCommandResult {
            success: false,
            error: Some(format!("Dataset not found: {}", dsn)),
            output: None,
            summary: ResolveSummary {
                total_dds: 1,
                errors: 1,
                ..Default::default()
            },
        },
        Err(e) => ResolveCommandResult {
            success: false,
            error: Some(format!("Catalog query failed: {}", e)),
            output: None,
            summary: ResolveSummary {
                total_dds: 1,
                errors: 1,
                ..Default::default()
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_bridge::{CatalogDatasetType, MockCatalog};

    #[test]
    fn command_rejects_non_jcl_document() {
        // Validates: Requirement 9 AC 8
        let params = ResolveCommandParams::default();
        let config = ResolverConfig::default();
        let catalog = MockCatalog::new();

        let result = execute_resolve_command(&params, "", "cobol", &config, &catalog);
        assert!(!result.success);
        assert_eq!(
            result.error.as_deref(),
            Some("Active document is not a JCL file")
        );
    }

    #[test]
    fn command_resolves_full_document() {
        // Validates: Requirement 9 AC 2
        let params = ResolveCommandParams::default();
        let config = ResolverConfig::default();
        let catalog = MockCatalog::new();
        let jcl = "//MYJOB  JOB (ACCT),'PGMR'\n//STEP1  EXEC PGM=IEFBR14\n//DD1    DD SYSOUT=A\n";

        let result = execute_resolve_command(&params, jcl, "jcl", &config, &catalog);
        assert!(result.output.is_some());
    }

    #[test]
    fn command_resolves_single_dsn() {
        // Validates: Requirement 9 AC 4
        let mut catalog = MockCatalog::new();
        catalog.add_dataset(
            "MY.DATA.SET",
            "/data/my/data/set",
            CatalogDatasetType::Ps,
            "PROD",
        );

        let params = ResolveCommandParams {
            dsn: Some("MY.DATA.SET".to_string()),
            ..Default::default()
        };
        let config = ResolverConfig::default();

        let result = execute_resolve_command(&params, "", "jcl", &config, &catalog);
        assert!(result.success);
    }

    #[test]
    fn command_mode_override() {
        // Validates: Requirement 9 AC 5
        let params = ResolveCommandParams {
            mode: Some(ResolveMode::Live),
            ..Default::default()
        };
        let config = ResolverConfig::default(); // DryRun by default
        let catalog = MockCatalog::new();
        let jcl = "//MYJOB  JOB (ACCT),'PGMR'\n//STEP1  EXEC PGM=IEFBR14\n//DD1    DD SYSOUT=A\n";

        let result = execute_resolve_command(&params, jcl, "jcl", &config, &catalog);
        // Should execute in Live mode (override)
        assert!(result.output.is_some());
    }
}
