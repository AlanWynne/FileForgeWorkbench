//! DISP interpretation and allocation simulation.
//!
//! Interprets DISP parameters and simulates dataset allocation,
//! supporting both dry-run and live modes.

use crate::catalog_bridge::{CatalogDatasetType, CatalogProvider};
use crate::config::{ResolveMode, ResolverConfig};
use crate::dd_statement::DdStatement;
use crate::diagnostic::{DiagnosticCode, LintDiagnostic};
use crate::operands::{DcbAttributes, DispAction, DispStatus};
use crate::temp_registry::TempDatasetRegistry;

use std::collections::HashMap;

/// Job-scoped tracking of datasets passed between steps (DISP=PASS).
#[derive(Debug, Clone, Default)]
pub struct PassTable {
    /// Maps DSN → PassEntry.
    entries: HashMap<String, PassEntry>,
}

/// A single passed dataset entry.
#[derive(Debug, Clone)]
pub struct PassEntry {
    /// DSN of the passed dataset.
    pub dsn: String,
    /// Step that passed this dataset.
    pub passing_step: String,
    /// Resolved physical path.
    pub physical_path: String,
}

impl PassTable {
    /// Create a new empty pass table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a dataset as passed.
    pub fn record_pass(&mut self, dsn: &str, step: &str, path: &str) {
        self.entries.insert(
            dsn.to_uppercase(),
            PassEntry {
                dsn: dsn.to_uppercase(),
                passing_step: step.to_string(),
                physical_path: path.to_string(),
            },
        );
    }

    /// Look up a passed dataset.
    pub fn lookup(&self, dsn: &str) -> Option<&PassEntry> {
        self.entries.get(&dsn.to_uppercase())
    }
}

/// Allocation outcome for a single DD statement.
#[derive(Debug, Clone, PartialEq)]
pub enum AllocationOutcome {
    /// Dataset exists and was verified (DISP=OLD/SHR).
    Verified {
        physical_path: String,
        catalog_name: String,
        dataset_type: CatalogDatasetType,
    },
    /// New dataset allocated (DISP=NEW, live mode).
    Allocated {
        physical_path: String,
        catalog_name: String,
    },
    /// New dataset would be allocated (DISP=NEW, dry-run mode).
    WouldAllocate { dsn: String },
    /// Dataset passed from a prior step.
    Passed {
        physical_path: String,
        passing_step: String,
    },
    /// No allocation needed (SYSOUT, DUMMY, inline).
    Skipped,
}

/// Simulate allocation for a DD statement based on its DISP parameter.
///
/// Returns the allocation outcome and any diagnostics produced.
#[allow(clippy::needless_return)]
pub fn simulate_allocation(
    dd: &DdStatement,
    catalog: &dyn CatalogProvider,
    config: &ResolverConfig,
    pass_table: &mut PassTable,
    _temp_registry: &TempDatasetRegistry,
) -> (AllocationOutcome, Vec<LintDiagnostic>) {
    let mut diagnostics = Vec::new();
    let disp = dd.effective_disp();

    // Get the DSN string for catalog operations
    let dsn_str = match &dd.dsn {
        Some(crate::dsn::DsnReference::Simple { dsn }) => dsn.clone(),
        Some(crate::dsn::DsnReference::Member { pds_dsn, .. }) => pds_dsn.clone(),
        Some(crate::dsn::DsnReference::Temporary { .. }) => {
            return (AllocationOutcome::Skipped, diagnostics);
        }
        Some(crate::dsn::DsnReference::Referback { .. }) => {
            return (AllocationOutcome::Skipped, diagnostics);
        }
        Some(crate::dsn::DsnReference::Gdg { .. }) => {
            return (AllocationOutcome::Skipped, diagnostics);
        }
        None => {
            return (AllocationOutcome::Skipped, diagnostics);
        }
    };

    match disp.status {
        DispStatus::New => {
            // Check if dataset already exists
            match catalog.dataset_exists(&dsn_str) {
                Ok(true) => {
                    diagnostics.push(
                        LintDiagnostic::new(
                            DiagnosticCode::DispConflict,
                            dd.line_number,
                            dd.column_range,
                            format!(
                                "Dataset already exists: {} (DISP=NEW requires non-existent dataset)",
                                dsn_str
                            ),
                        )
                        .with_ddname(&dd.ddname),
                    );
                    return (AllocationOutcome::Skipped, diagnostics);
                }
                Ok(false) => {
                    // Allocate based on mode
                    match config.resolve_mode {
                        ResolveMode::Live => {
                            let attrs = dd
                                .dcb
                                .clone()
                                .unwrap_or_else(DcbAttributes::hardcoded_defaults);
                            match catalog.allocate_dataset(&dsn_str, &attrs, dd.space.as_ref()) {
                                Ok(path) => {
                                    // Record PASS if applicable
                                    if disp.normal_disp == Some(DispAction::Pass) {
                                        pass_table.record_pass(&dsn_str, &dd.step_name, &path);
                                    }
                                    return (
                                        AllocationOutcome::Allocated {
                                            physical_path: path,
                                            catalog_name: "default".to_string(),
                                        },
                                        diagnostics,
                                    );
                                }
                                Err(e) => {
                                    diagnostics.push(
                                        LintDiagnostic::new(
                                            DiagnosticCode::CatalogQueryFailed,
                                            dd.line_number,
                                            dd.column_range,
                                            format!("Allocation failed: {}", e),
                                        )
                                        .with_ddname(&dd.ddname),
                                    );
                                    return (AllocationOutcome::Skipped, diagnostics);
                                }
                            }
                        }
                        ResolveMode::DryRun => {
                            if disp.normal_disp == Some(DispAction::Pass) {
                                let path =
                                    format!("/data/{}", dsn_str.to_lowercase().replace('.', "/"));
                                pass_table.record_pass(&dsn_str, &dd.step_name, &path);
                            }
                            return (
                                AllocationOutcome::WouldAllocate { dsn: dsn_str },
                                diagnostics,
                            );
                        }
                    }
                }
                Err(e) => {
                    diagnostics.push(
                        LintDiagnostic::new(
                            DiagnosticCode::CatalogQueryFailed,
                            dd.line_number,
                            dd.column_range,
                            format!("Catalog query failed: {}", e),
                        )
                        .with_ddname(&dd.ddname),
                    );
                    return (AllocationOutcome::Skipped, diagnostics);
                }
            }
        }

        DispStatus::Old | DispStatus::Shr => {
            // Check pass table first
            if let Some(pass_entry) = pass_table.lookup(&dsn_str) {
                return (
                    AllocationOutcome::Passed {
                        physical_path: pass_entry.physical_path.clone(),
                        passing_step: pass_entry.passing_step.clone(),
                    },
                    diagnostics,
                );
            }

            // Verify dataset exists in catalog
            match catalog.dataset_exists(&dsn_str) {
                Ok(true) => {
                    match catalog.lookup_dsn(&dsn_str) {
                        Ok(matches) if !matches.is_empty() => {
                            let first = &matches[0];
                            if disp.normal_disp == Some(DispAction::Pass) {
                                pass_table.record_pass(
                                    &dsn_str,
                                    &dd.step_name,
                                    &first.physical_path,
                                );
                            }
                            return (
                                AllocationOutcome::Verified {
                                    physical_path: first.physical_path.clone(),
                                    catalog_name: first.catalog_name.clone(),
                                    dataset_type: first.dataset_type,
                                },
                                diagnostics,
                            );
                        }
                        _ => {
                            // Has entry but lookup failed — still report as verified
                            return (
                                AllocationOutcome::Verified {
                                    physical_path: String::new(),
                                    catalog_name: String::new(),
                                    dataset_type: CatalogDatasetType::Ps,
                                },
                                diagnostics,
                            );
                        }
                    }
                }
                Ok(false) => {
                    diagnostics.push(
                        LintDiagnostic::new(
                            DiagnosticCode::DispConflict,
                            dd.line_number,
                            dd.column_range,
                            format!(
                                "Dataset not found: {} (DISP={} requires existing dataset)",
                                dsn_str, disp.status
                            ),
                        )
                        .with_ddname(&dd.ddname),
                    );
                    return (AllocationOutcome::Skipped, diagnostics);
                }
                Err(e) => {
                    diagnostics.push(
                        LintDiagnostic::new(
                            DiagnosticCode::CatalogQueryFailed,
                            dd.line_number,
                            dd.column_range,
                            format!("Catalog query failed: {}", e),
                        )
                        .with_ddname(&dd.ddname),
                    );
                    return (AllocationOutcome::Skipped, diagnostics);
                }
            }
        }

        DispStatus::Mod => {
            // MOD: verify existence for append; if not found AND SPACE provided, treat as NEW
            match catalog.dataset_exists(&dsn_str) {
                Ok(true) => {
                    return (
                        AllocationOutcome::Verified {
                            physical_path: String::new(),
                            catalog_name: String::new(),
                            dataset_type: CatalogDatasetType::Ps,
                        },
                        diagnostics,
                    );
                }
                Ok(false) => {
                    if dd.space.is_some() {
                        // Treat as NEW
                        match config.resolve_mode {
                            ResolveMode::Live => {
                                let attrs = dd
                                    .dcb
                                    .clone()
                                    .unwrap_or_else(DcbAttributes::hardcoded_defaults);
                                match catalog.allocate_dataset(&dsn_str, &attrs, dd.space.as_ref())
                                {
                                    Ok(path) => {
                                        return (
                                            AllocationOutcome::Allocated {
                                                physical_path: path,
                                                catalog_name: "default".to_string(),
                                            },
                                            diagnostics,
                                        );
                                    }
                                    Err(e) => {
                                        diagnostics.push(
                                            LintDiagnostic::new(
                                                DiagnosticCode::CatalogQueryFailed,
                                                dd.line_number,
                                                dd.column_range,
                                                format!("Allocation failed: {}", e),
                                            )
                                            .with_ddname(&dd.ddname),
                                        );
                                    }
                                }
                            }
                            ResolveMode::DryRun => {
                                return (
                                    AllocationOutcome::WouldAllocate { dsn: dsn_str },
                                    diagnostics,
                                );
                            }
                        }
                    } else {
                        diagnostics.push(
                            LintDiagnostic::new(
                                DiagnosticCode::DispConflict,
                                dd.line_number,
                                dd.column_range,
                                format!(
                                    "Dataset not found: {} (DISP=MOD without SPACE cannot create)",
                                    dsn_str
                                ),
                            )
                            .with_ddname(&dd.ddname),
                        );
                    }
                    return (AllocationOutcome::Skipped, diagnostics);
                }
                Err(e) => {
                    diagnostics.push(
                        LintDiagnostic::new(
                            DiagnosticCode::CatalogQueryFailed,
                            dd.line_number,
                            dd.column_range,
                            format!("Catalog query failed: {}", e),
                        )
                        .with_ddname(&dd.ddname),
                    );
                    return (AllocationOutcome::Skipped, diagnostics);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_bridge::MockCatalog;
    use crate::dd_statement::DdKind;
    use crate::dsn::DsnReference;
    use crate::operands::{DispAction, DispParameter, DispStatus, SpaceAllocation, SpaceUnit};

    fn make_dd(dsn: &str, disp: DispParameter) -> DdStatement {
        DdStatement {
            ddname: "TEST".to_string(),
            line_number: 1,
            column_range: (0, 40),
            step_name: "STEP1".to_string(),
            dsn: Some(DsnReference::Simple {
                dsn: dsn.to_string(),
            }),
            disp: Some(disp),
            dcb: None,
            space: None,
            kind: DdKind::Dataset,
            concatenation_index: 0,
            raw_operands: String::new(),
        }
    }

    #[test]
    fn new_dataset_dry_run_reports_would_allocate() {
        // Validates: Requirement 4 AC 9
        let catalog = MockCatalog::new();
        let config = ResolverConfig::default(); // DryRun
        let mut pass_table = PassTable::new();
        let temp_registry = TempDatasetRegistry::new();
        let dd = make_dd(
            "NEW.DATA",
            DispParameter {
                status: DispStatus::New,
                normal_disp: Some(DispAction::Catlg),
                abnormal_disp: None,
            },
        );

        let (outcome, diags) =
            simulate_allocation(&dd, &catalog, &config, &mut pass_table, &temp_registry);
        assert!(matches!(outcome, AllocationOutcome::WouldAllocate { .. }));
        assert!(diags.is_empty());
    }

    #[test]
    fn new_dataset_already_exists_produces_error() {
        // Validates: Requirement 4 AC 3
        let mut catalog = MockCatalog::new();
        catalog.add_dataset("EXISTS.DATA", "/path", CatalogDatasetType::Ps, "CAT1");
        let config = ResolverConfig::default();
        let mut pass_table = PassTable::new();
        let temp_registry = TempDatasetRegistry::new();
        let dd = make_dd(
            "EXISTS.DATA",
            DispParameter {
                status: DispStatus::New,
                normal_disp: Some(DispAction::Catlg),
                abnormal_disp: None,
            },
        );

        let (_, diags) =
            simulate_allocation(&dd, &catalog, &config, &mut pass_table, &temp_registry);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, DiagnosticCode::DispConflict);
    }

    #[test]
    fn old_dataset_not_found_produces_error() {
        // Validates: Requirement 4 AC 4
        let catalog = MockCatalog::new();
        let config = ResolverConfig::default();
        let mut pass_table = PassTable::new();
        let temp_registry = TempDatasetRegistry::new();
        let dd = make_dd(
            "MISSING.DATA",
            DispParameter {
                status: DispStatus::Old,
                normal_disp: Some(DispAction::Keep),
                abnormal_disp: None,
            },
        );

        let (_, diags) =
            simulate_allocation(&dd, &catalog, &config, &mut pass_table, &temp_registry);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("not found"));
    }

    #[test]
    fn mod_without_space_and_not_found_produces_error() {
        // Validates: Requirement 4 AC 6
        let catalog = MockCatalog::new();
        let config = ResolverConfig::default();
        let mut pass_table = PassTable::new();
        let temp_registry = TempDatasetRegistry::new();
        let dd = make_dd(
            "MISSING.DATA",
            DispParameter {
                status: DispStatus::Mod,
                normal_disp: None,
                abnormal_disp: None,
            },
        );

        let (_, diags) =
            simulate_allocation(&dd, &catalog, &config, &mut pass_table, &temp_registry);
        assert!(!diags.is_empty());
    }

    #[test]
    fn mod_with_space_and_not_found_treats_as_new() {
        // Validates: Requirement 4 AC 6
        let catalog = MockCatalog::new();
        let config = ResolverConfig::default();
        let mut pass_table = PassTable::new();
        let temp_registry = TempDatasetRegistry::new();
        let mut dd = make_dd(
            "MISSING.DATA",
            DispParameter {
                status: DispStatus::Mod,
                normal_disp: None,
                abnormal_disp: None,
            },
        );
        dd.space = Some(SpaceAllocation {
            unit: SpaceUnit::Trk,
            primary: 10,
            secondary: None,
            directory: None,
        });

        let (outcome, diags) =
            simulate_allocation(&dd, &catalog, &config, &mut pass_table, &temp_registry);
        assert!(matches!(outcome, AllocationOutcome::WouldAllocate { .. }));
        assert!(diags.is_empty());
    }

    #[test]
    fn pass_disposition_records_in_pass_table() {
        // Validates: Requirement 4 AC 8
        let catalog = MockCatalog::new();
        let config = ResolverConfig::default();
        let mut pass_table = PassTable::new();
        let temp_registry = TempDatasetRegistry::new();
        let dd = make_dd(
            "PASS.DATA",
            DispParameter {
                status: DispStatus::New,
                normal_disp: Some(DispAction::Pass),
                abnormal_disp: None,
            },
        );

        let _ = simulate_allocation(&dd, &catalog, &config, &mut pass_table, &temp_registry);
        assert!(pass_table.lookup("PASS.DATA").is_some());
    }

    #[test]
    fn default_disp_applied_when_none() {
        // Validates: Requirement 4 AC 7
        let catalog = MockCatalog::new();
        let config = ResolverConfig::default();
        let mut pass_table = PassTable::new();
        let temp_registry = TempDatasetRegistry::new();
        let dd = DdStatement {
            ddname: "DD1".to_string(),
            line_number: 1,
            column_range: (0, 20),
            step_name: "STEP1".to_string(),
            dsn: Some(DsnReference::Simple {
                dsn: "TEST.DATA".to_string(),
            }),
            disp: None, // no DISP — defaults to (NEW,DELETE)
            dcb: None,
            space: None,
            kind: DdKind::Dataset,
            concatenation_index: 0,
            raw_operands: String::new(),
        };

        let (outcome, _) =
            simulate_allocation(&dd, &catalog, &config, &mut pass_table, &temp_registry);
        // Should attempt NEW allocation (dry-run)
        assert!(matches!(outcome, AllocationOutcome::WouldAllocate { .. }));
    }
}
