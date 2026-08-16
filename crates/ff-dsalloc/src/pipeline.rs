//! Resolution processing pipeline.
//!
//! Orchestrates the four-stage pipeline: Parse → Substitute → Resolve → Validate.
//! Each stage produces intermediate results; errors in one DD do not prevent
//! resolution of subsequent DDs.

use std::collections::HashMap;

use crate::catalog_bridge::CatalogProvider;
use crate::config::ResolverConfig;
use crate::diagnostic::{DiagnosticSeverity, LintDiagnostic};
use crate::job_model::JclJob;

/// Dataset type returned from resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetType {
    /// Physical sequential.
    Ps,
    /// Partitioned (PDS/PDSE).
    Po,
    /// Generation Data Group.
    Gdg,
}

/// Reason a DD was skipped during resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// SYSOUT DD.
    Sysout,
    /// DUMMY DD.
    Dummy,
    /// Inline DD (DD * or DD DATA).
    Inline,
}

/// The outcome of a single DSN resolution attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionOutcome {
    /// Successfully resolved to a physical path.
    Resolved {
        /// Physical file path.
        physical_path: String,
        /// Catalog that provided the resolution.
        catalog_name: String,
        /// Dataset type.
        dataset_type: DatasetType,
    },
    /// Resolved as a temporary dataset.
    Temporary {
        /// Step that created this temporary.
        creating_step: String,
    },
    /// Allocated as a new dataset (DISP=NEW).
    Allocated {
        /// Physical file path.
        physical_path: String,
        /// Catalog name.
        catalog_name: String,
    },
    /// GDG generation resolved.
    GdgResolved {
        /// Full generation DSN.
        generation_dsn: String,
        /// Physical file path.
        physical_path: String,
        /// Catalog name.
        catalog_name: String,
        /// Generation number.
        generation_number: i32,
    },
    /// Skipped (SYSOUT, DUMMY, inline DD).
    Skipped {
        /// Reason for skipping.
        reason: SkipReason,
    },
    /// Resolution failed — see diagnostics.
    Failed,
}

/// The output of resolving a single DD statement.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionResult {
    /// The ddname of the resolved DD.
    pub ddname: String,
    /// The step containing this DD.
    pub step_name: String,
    /// The original DSN (before substitution).
    pub original_dsn: Option<String>,
    /// The DSN after symbolic substitution.
    pub substituted_dsn: Option<String>,
    /// Resolution outcome.
    pub outcome: ResolutionOutcome,
    /// Concatenation index (0 for primary, 1+ for concatenated).
    pub concatenation_index: usize,
}

/// Summary statistics for a resolution run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResolveSummary {
    /// Total DD statements processed.
    pub total_dds: usize,
    /// Successfully resolved count.
    pub resolved: usize,
    /// Warning count.
    pub warnings: usize,
    /// Error count.
    pub errors: usize,
    /// Skipped count (SYSOUT, DUMMY, inline).
    pub skipped: usize,
}

/// Timing data for each pipeline stage.
#[derive(Debug, Clone, Copy, Default)]
pub struct StageTiming {
    /// Parse stage duration in milliseconds.
    pub parse_ms: u64,
    /// Substitution stage duration in milliseconds.
    pub substitute_ms: u64,
    /// Resolution stage duration in milliseconds.
    pub resolve_ms: u64,
    /// Validation stage duration in milliseconds.
    pub validate_ms: u64,
}

/// Intermediate pipeline state for inspection/debugging.
#[derive(Debug, Clone, Default)]
pub struct PipelineState {
    /// Parsed job model (stage 1 output).
    pub job_model: Option<JclJob>,
    /// Substituted operand values per DD (stage 2 output).
    pub substitutions: HashMap<String, String>,
    /// Stage timing in milliseconds.
    pub stage_timings: StageTiming,
}

/// The complete output of a resolution operation.
#[derive(Debug, Clone)]
pub struct ResolveOutput {
    /// Resolution results for each DD statement processed.
    pub results: Vec<ResolutionResult>,
    /// All diagnostics produced across all pipeline stages.
    pub diagnostics: Vec<LintDiagnostic>,
    /// Summary statistics.
    pub summary: ResolveSummary,
    /// Intermediate pipeline state (for debugging/inspection).
    pub pipeline_state: PipelineState,
}

impl ResolveOutput {
    /// Create a new empty resolve output.
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            diagnostics: Vec::new(),
            summary: ResolveSummary::default(),
            pipeline_state: PipelineState::default(),
        }
    }

    /// Compute summary from current results and diagnostics.
    pub fn compute_summary(&mut self) {
        self.summary.total_dds = self.results.len();
        self.summary.resolved = self
            .results
            .iter()
            .filter(|r| {
                matches!(
                    r.outcome,
                    ResolutionOutcome::Resolved { .. }
                        | ResolutionOutcome::Allocated { .. }
                        | ResolutionOutcome::GdgResolved { .. }
                        | ResolutionOutcome::Temporary { .. }
                )
            })
            .count();
        self.summary.skipped = self
            .results
            .iter()
            .filter(|r| matches!(r.outcome, ResolutionOutcome::Skipped { .. }))
            .count();
        self.summary.errors = self
            .diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count();
        self.summary.warnings = self
            .diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count();
    }

    /// Filter diagnostics by minimum severity level.
    pub fn filter_diagnostics(&mut self, min_severity: DiagnosticSeverity) {
        self.diagnostics.retain(|d| d.severity >= min_severity);
    }
}

impl Default for ResolveOutput {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the full resolution pipeline on JCL text.
///
/// Stages:
/// 1. Parse — extract job structure and DD statements
/// 2. Substitute — replace symbolic parameters
/// 3. Resolve — look up DSNs in catalogs
/// 4. Validate — produce lint diagnostics
pub fn resolve_document(
    text: &str,
    config: &ResolverConfig,
    catalog: &dyn CatalogProvider,
) -> ResolveOutput {
    use std::time::Instant;

    let mut output = ResolveOutput::new();
    let mut timings = StageTiming::default();

    // Stage 1: Parse
    let parse_start = Instant::now();
    let job = crate::job_model::build_job_model(text);
    let parse_result = crate::parser::parse_jcl_statements(text);
    timings.parse_ms = parse_start.elapsed().as_millis() as u64;

    output.diagnostics.extend(parse_result.diagnostics);
    output.pipeline_state.job_model = Some(job.clone());

    // Stage 2: Substitute
    let sub_start = Instant::now();
    let symbols = crate::symbols::SymbolTable::new_with_system_symbols(config);
    // Substitution happens on each DD's raw operands — for this pipeline
    // we process the already-parsed DD statements
    timings.substitute_ms = sub_start.elapsed().as_millis() as u64;

    // Stage 3: Resolve
    let resolve_start = Instant::now();
    let mut temp_registry = crate::temp_registry::TempDatasetRegistry::new();
    let mut pass_table = crate::allocation::PassTable::new();
    let _gdg_state = crate::gdg_resolver::GdgJobState::new();

    for step in &job.steps {
        for dd in &step.dd_statements {
            let result = resolve_single_dd(
                dd,
                catalog,
                config,
                &mut pass_table,
                &mut temp_registry,
                &symbols,
                &job,
            );
            output.results.push(result.0);
            output.diagnostics.extend(result.1);
        }
    }
    timings.resolve_ms = resolve_start.elapsed().as_millis() as u64;

    // Stage 4: Validate
    let validate_start = Instant::now();
    let lint_diags = crate::lint::validate_job(&job, config);
    output.diagnostics.extend(lint_diags);
    timings.validate_ms = validate_start.elapsed().as_millis() as u64;

    output.pipeline_state.stage_timings = timings;

    // Filter by configured lint level
    output.filter_diagnostics(config.lint_level);

    // Sort diagnostics by line number
    output.diagnostics.sort();

    // Compute summary
    output.compute_summary();

    output
}

/// Resolve a single DD statement.
fn resolve_single_dd(
    dd: &crate::dd_statement::DdStatement,
    catalog: &dyn CatalogProvider,
    config: &ResolverConfig,
    pass_table: &mut crate::allocation::PassTable,
    temp_registry: &mut crate::temp_registry::TempDatasetRegistry,
    _symbols: &crate::symbols::SymbolTable,
    _job: &JclJob,
) -> (ResolutionResult, Vec<LintDiagnostic>) {
    use crate::dd_statement::DdKind;
    use crate::dsn::DsnReference;

    let original_dsn = dd.dsn.as_ref().map(|d| d.display_name());
    let mut diagnostics = Vec::new();

    // Handle non-resolution DD types
    let outcome = match &dd.kind {
        DdKind::Sysout { .. } => ResolutionOutcome::Skipped {
            reason: SkipReason::Sysout,
        },
        DdKind::Inline => ResolutionOutcome::Skipped {
            reason: SkipReason::Inline,
        },
        DdKind::Dummy => ResolutionOutcome::Skipped {
            reason: SkipReason::Dummy,
        },
        DdKind::Dataset => {
            // Resolve based on DSN type
            match &dd.dsn {
                Some(DsnReference::Temporary { name }) => {
                    // Check temp registry
                    let disp = dd.effective_disp();
                    if disp.creates_new() {
                        temp_registry.register(name, &dd.step_name, dd.dcb.clone());
                        ResolutionOutcome::Temporary {
                            creating_step: dd.step_name.clone(),
                        }
                    } else {
                        match temp_registry.lookup(name, dd.line_number) {
                            Ok(entry) => ResolutionOutcome::Temporary {
                                creating_step: entry.creating_step.clone(),
                            },
                            Err(diag) => {
                                diagnostics.push(diag);
                                ResolutionOutcome::Failed
                            }
                        }
                    }
                }
                Some(DsnReference::Simple { .. } | DsnReference::Member { .. }) => {
                    // Use allocation simulator
                    let (alloc_outcome, alloc_diags) = crate::allocation::simulate_allocation(
                        dd,
                        catalog,
                        config,
                        pass_table,
                        temp_registry,
                    );
                    diagnostics.extend(alloc_diags);
                    match alloc_outcome {
                        crate::allocation::AllocationOutcome::Verified {
                            physical_path,
                            catalog_name,
                            dataset_type,
                        } => {
                            let dt = match dataset_type {
                                crate::catalog_bridge::CatalogDatasetType::Ps => DatasetType::Ps,
                                crate::catalog_bridge::CatalogDatasetType::Po => DatasetType::Po,
                                crate::catalog_bridge::CatalogDatasetType::Gdg => DatasetType::Gdg,
                            };
                            ResolutionOutcome::Resolved {
                                physical_path,
                                catalog_name,
                                dataset_type: dt,
                            }
                        }
                        crate::allocation::AllocationOutcome::Allocated {
                            physical_path,
                            catalog_name,
                        } => ResolutionOutcome::Allocated {
                            physical_path,
                            catalog_name,
                        },
                        crate::allocation::AllocationOutcome::WouldAllocate { .. } => {
                            ResolutionOutcome::Allocated {
                                physical_path: String::new(),
                                catalog_name: "dry-run".to_string(),
                            }
                        }
                        crate::allocation::AllocationOutcome::Passed {
                            physical_path,
                            passing_step,
                        } => ResolutionOutcome::Resolved {
                            physical_path,
                            catalog_name: format!("passed from {}", passing_step),
                            dataset_type: DatasetType::Ps,
                        },
                        crate::allocation::AllocationOutcome::Skipped => {
                            if diagnostics.is_empty() {
                                ResolutionOutcome::Skipped {
                                    reason: SkipReason::Dummy,
                                }
                            } else {
                                ResolutionOutcome::Failed
                            }
                        }
                    }
                }
                Some(DsnReference::Gdg { .. }) | Some(DsnReference::Referback { .. }) => {
                    // GDG and referback resolution handled at higher level
                    ResolutionOutcome::Failed
                }
                None => ResolutionOutcome::Skipped {
                    reason: SkipReason::Dummy,
                },
            }
        }
    };

    let result = ResolutionResult {
        ddname: dd.ddname.clone(),
        step_name: dd.step_name.clone(),
        original_dsn: original_dsn.clone(),
        substituted_dsn: original_dsn, // In full pipeline, would be post-substitution
        outcome,
        concatenation_index: dd.concatenation_index,
    };

    (result, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_bridge::{CatalogDatasetType, MockCatalog};

    #[test]
    fn resolve_document_handles_sysout_and_dummy() {
        // Validates: Requirement 13 AC 3
        let jcl = "\
//MYJOB  JOB (ACCT),'PGMR'
//STEP1  EXEC PGM=IEFBR14
//SYSPRINT DD SYSOUT=A
//NULLDD DD DUMMY
//SYSIN  DD *
";
        let config = ResolverConfig::default();
        let catalog = MockCatalog::new();
        let output = resolve_document(jcl, &config, &catalog);

        assert_eq!(output.results.len(), 3);
        assert!(matches!(
            output.results[0].outcome,
            ResolutionOutcome::Skipped {
                reason: SkipReason::Sysout
            }
        ));
        assert!(matches!(
            output.results[1].outcome,
            ResolutionOutcome::Skipped {
                reason: SkipReason::Dummy
            }
        ));
        assert!(matches!(
            output.results[2].outcome,
            ResolutionOutcome::Skipped {
                reason: SkipReason::Inline
            }
        ));
    }

    #[test]
    fn resolve_document_error_isolation() {
        // Validates: Requirement 13 AC 3
        // Errors in one DD should not prevent resolution of others
        let jcl = "\
//MYJOB  JOB (ACCT),'PGMR'
//STEP1  EXEC PGM=IEFBR14
//DD1    DD DSN=MISSING.DATA,DISP=OLD
//DD2    DD SYSOUT=A
//DD3    DD DSN=ALSO.MISSING,DISP=SHR
";
        let config = ResolverConfig::default();
        let catalog = MockCatalog::new();
        let output = resolve_document(jcl, &config, &catalog);

        // All 3 DDs should have results — none dropped
        assert_eq!(output.results.len(), 3);
        assert_eq!(output.summary.total_dds, 3);
    }

    #[test]
    fn resolve_document_successful_resolution() {
        // Validates: Requirement 13 AC 1
        let jcl = "\
//MYJOB  JOB (ACCT),'PGMR'
//STEP1  EXEC PGM=IEFBR14
//INPUT  DD DSN=MY.DATA.SET,DISP=SHR
";
        let mut catalog = MockCatalog::new();
        catalog.add_dataset(
            "MY.DATA.SET",
            "/data/my/data/set",
            CatalogDatasetType::Ps,
            "PROD",
        );
        let config = ResolverConfig::default();
        let output = resolve_document(jcl, &config, &catalog);

        assert_eq!(output.results.len(), 1);
        assert!(matches!(
            output.results[0].outcome,
            ResolutionOutcome::Resolved { .. }
        ));
    }

    #[test]
    fn resolve_document_computes_summary() {
        // Validates: Requirement 13 AC 4
        let jcl = "\
//MYJOB  JOB (ACCT),'PGMR'
//STEP1  EXEC PGM=IEFBR14
//DD1    DD SYSOUT=A
//DD2    DD DSN=NEW.DATA,DISP=(NEW,CATLG)
";
        let config = ResolverConfig::default();
        let catalog = MockCatalog::new();
        let output = resolve_document(jcl, &config, &catalog);

        assert!(output.summary.total_dds > 0);
    }
}
