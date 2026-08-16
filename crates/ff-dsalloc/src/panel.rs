//! Resolution output panel model.
//!
//! Defines the data model for the resolution results panel UI,
//! including row formatting, sorting, filtering, and navigation.

use crate::pipeline::{ResolutionOutcome, ResolutionResult, ResolveOutput, SkipReason};

/// Panel ID for the resolution output panel.
pub const PANEL_ID: &str = "jcl.resolution";

/// Status displayed in the panel for each DD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelStatus {
    /// Successfully resolved.
    Resolved,
    /// Resolution produced an error.
    Error,
    /// Resolution produced a warning.
    Warning,
    /// DD was skipped (SYSOUT, DUMMY, inline).
    Skipped,
}

impl std::fmt::Display for PanelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolved => write!(f, "Resolved"),
            Self::Error => write!(f, "Error"),
            Self::Warning => write!(f, "Warning"),
            Self::Skipped => write!(f, "Skipped"),
        }
    }
}

/// A single row in the resolution panel.
#[derive(Debug, Clone)]
pub struct PanelRow {
    /// Step name.
    pub step_name: String,
    /// DD name.
    pub dd_name: String,
    /// DSN after substitution (for display).
    pub dsn_display: String,
    /// Resolution status.
    pub status: PanelStatus,
    /// Physical path or error message.
    pub path_or_message: String,
    /// Catalog name (if resolved).
    pub catalog_name: String,
    /// Line number in source (for navigation).
    pub source_line: usize,
    /// Concatenation children (for expandable groups).
    pub concatenation_children: Vec<PanelRow>,
}

/// The resolution panel model.
#[derive(Debug, Clone)]
pub struct ResolutionPanelModel {
    /// Panel identifier.
    pub panel_id: String,
    /// All rows in the panel.
    pub rows: Vec<PanelRow>,
    /// Summary statistics.
    pub total: usize,
    /// Resolved count.
    pub resolved: usize,
    /// Warning count.
    pub warnings: usize,
    /// Error count.
    pub errors: usize,
}

impl ResolutionPanelModel {
    /// Create a panel model from resolution output.
    pub fn from_resolve_output(output: &ResolveOutput) -> Self {
        let rows: Vec<PanelRow> = output
            .results
            .iter()
            .filter(|r| r.concatenation_index == 0) // Only primary DDs as top-level rows
            .map(|r| {
                // Collect concatenation children
                let children: Vec<PanelRow> = output
                    .results
                    .iter()
                    .filter(|c| {
                        c.ddname == r.ddname
                            && c.step_name == r.step_name
                            && c.concatenation_index > 0
                    })
                    .map(|c| result_to_row(c, 0))
                    .collect();

                let mut row = result_to_row(r, 0);
                row.concatenation_children = children;
                row
            })
            .collect();

        let resolved = rows
            .iter()
            .filter(|r| r.status == PanelStatus::Resolved)
            .count();
        let errors = rows
            .iter()
            .filter(|r| r.status == PanelStatus::Error)
            .count();
        let warnings = rows
            .iter()
            .filter(|r| r.status == PanelStatus::Warning)
            .count();

        Self {
            panel_id: PANEL_ID.to_string(),
            rows,
            total: output.summary.total_dds,
            resolved,
            warnings,
            errors,
        }
    }

    /// Filter rows by status.
    pub fn filter_by_status(&self, status: PanelStatus) -> Vec<&PanelRow> {
        self.rows.iter().filter(|r| r.status == status).collect()
    }

    /// Sort rows by step name.
    pub fn sort_by_step(&mut self) {
        self.rows.sort_by(|a, b| a.step_name.cmp(&b.step_name));
    }

    /// Sort rows by DD name.
    pub fn sort_by_dd_name(&mut self) {
        self.rows.sort_by(|a, b| a.dd_name.cmp(&b.dd_name));
    }

    /// Sort rows by status (errors first).
    pub fn sort_by_status(&mut self) {
        self.rows.sort_by_key(|r| match r.status {
            PanelStatus::Error => 0,
            PanelStatus::Warning => 1,
            PanelStatus::Resolved => 2,
            PanelStatus::Skipped => 3,
        });
    }
}

/// Convert a ResolutionResult to a PanelRow.
fn result_to_row(result: &ResolutionResult, _line_offset: usize) -> PanelRow {
    let (status, path_or_message, catalog_name) = match &result.outcome {
        ResolutionOutcome::Resolved {
            physical_path,
            catalog_name,
            ..
        } => (
            PanelStatus::Resolved,
            physical_path.clone(),
            catalog_name.clone(),
        ),
        ResolutionOutcome::Allocated {
            physical_path,
            catalog_name,
        } => (
            PanelStatus::Resolved,
            format!("[NEW] {}", physical_path),
            catalog_name.clone(),
        ),
        ResolutionOutcome::GdgResolved {
            generation_dsn,
            physical_path,
            catalog_name,
            ..
        } => (
            PanelStatus::Resolved,
            format!("{} → {}", generation_dsn, physical_path),
            catalog_name.clone(),
        ),
        ResolutionOutcome::Temporary { creating_step } => (
            PanelStatus::Resolved,
            format!("[TEMP from {}]", creating_step),
            String::new(),
        ),
        ResolutionOutcome::Skipped { reason } => {
            let msg = match reason {
                SkipReason::Sysout => "SYSOUT",
                SkipReason::Dummy => "DUMMY",
                SkipReason::Inline => "INLINE",
            };
            (PanelStatus::Skipped, msg.to_string(), String::new())
        }
        ResolutionOutcome::Failed => (
            PanelStatus::Error,
            "Resolution failed".to_string(),
            String::new(),
        ),
    };

    PanelRow {
        step_name: result.step_name.clone(),
        dd_name: result.ddname.clone(),
        dsn_display: result.substituted_dsn.clone().unwrap_or_default(),
        status,
        path_or_message,
        catalog_name,
        source_line: 0, // Would be populated from DD statement
        concatenation_children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{DatasetType, PipelineState, ResolveSummary};

    fn make_output(results: Vec<ResolutionResult>) -> ResolveOutput {
        let total = results.len();
        ResolveOutput {
            results,
            diagnostics: Vec::new(),
            summary: ResolveSummary {
                total_dds: total,
                ..Default::default()
            },
            pipeline_state: PipelineState::default(),
        }
    }

    #[test]
    fn panel_model_from_resolve_output() {
        // Validates: Requirement 11 AC 1, AC 2
        let output = make_output(vec![
            ResolutionResult {
                ddname: "SYSUT1".to_string(),
                step_name: "STEP1".to_string(),
                original_dsn: Some("MY.DATA".to_string()),
                substituted_dsn: Some("MY.DATA".to_string()),
                outcome: ResolutionOutcome::Resolved {
                    physical_path: "/data/my/data".to_string(),
                    catalog_name: "PROD".to_string(),
                    dataset_type: DatasetType::Ps,
                },
                concatenation_index: 0,
            },
            ResolutionResult {
                ddname: "SYSPRINT".to_string(),
                step_name: "STEP1".to_string(),
                original_dsn: None,
                substituted_dsn: None,
                outcome: ResolutionOutcome::Skipped {
                    reason: SkipReason::Sysout,
                },
                concatenation_index: 0,
            },
        ]);

        let panel = ResolutionPanelModel::from_resolve_output(&output);
        assert_eq!(panel.panel_id, "jcl.resolution");
        assert_eq!(panel.rows.len(), 2);
        assert_eq!(panel.rows[0].status, PanelStatus::Resolved);
        assert_eq!(panel.rows[1].status, PanelStatus::Skipped);
    }

    #[test]
    fn panel_filter_by_status() {
        // Validates: Requirement 11 AC 7
        let output = make_output(vec![
            ResolutionResult {
                ddname: "DD1".to_string(),
                step_name: "STEP1".to_string(),
                original_dsn: None,
                substituted_dsn: None,
                outcome: ResolutionOutcome::Resolved {
                    physical_path: "/p".to_string(),
                    catalog_name: "C".to_string(),
                    dataset_type: DatasetType::Ps,
                },
                concatenation_index: 0,
            },
            ResolutionResult {
                ddname: "DD2".to_string(),
                step_name: "STEP1".to_string(),
                original_dsn: None,
                substituted_dsn: None,
                outcome: ResolutionOutcome::Failed,
                concatenation_index: 0,
            },
        ]);

        let panel = ResolutionPanelModel::from_resolve_output(&output);
        let errors = panel.filter_by_status(PanelStatus::Error);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].dd_name, "DD2");
    }

    #[test]
    fn panel_sort_by_status() {
        // Validates: Requirement 11 AC 7
        let output = make_output(vec![
            ResolutionResult {
                ddname: "DD1".to_string(),
                step_name: "S".to_string(),
                original_dsn: None,
                substituted_dsn: None,
                outcome: ResolutionOutcome::Skipped {
                    reason: SkipReason::Sysout,
                },
                concatenation_index: 0,
            },
            ResolutionResult {
                ddname: "DD2".to_string(),
                step_name: "S".to_string(),
                original_dsn: None,
                substituted_dsn: None,
                outcome: ResolutionOutcome::Failed,
                concatenation_index: 0,
            },
        ]);

        let mut panel = ResolutionPanelModel::from_resolve_output(&output);
        panel.sort_by_status();
        assert_eq!(panel.rows[0].status, PanelStatus::Error);
        assert_eq!(panel.rows[1].status, PanelStatus::Skipped);
    }
}
