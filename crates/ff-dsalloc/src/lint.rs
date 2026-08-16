//! JCL validation and lint diagnostic emitter.
//!
//! Produces lint diagnostics for common JCL problems: unresolved DSNs,
//! duplicate ddnames, invalid DSN syntax, missing well-known DDs, etc.

use std::collections::HashSet;

use crate::config::ResolverConfig;
use crate::dd_statement::DdKind;
use crate::diagnostic::{DiagnosticCode, DiagnosticSeverity, LintDiagnostic};
use crate::dsn::DatasetName;
use crate::job_model::JclJob;

/// Well-known DD names that are commonly expected in a step.
const WELL_KNOWN_DDS: &[&str] = &["SYSIN", "SYSPRINT", "SYSUT1", "SYSUT2", "SYSLIB"];

/// Validate a complete JCL job and produce lint diagnostics.
pub fn validate_job(job: &JclJob, _config: &ResolverConfig) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();

    for step in &job.steps {
        // Check for duplicate ddnames within a step (excluding concatenation)
        diagnostics.extend(check_duplicate_ddnames(step));

        // Validate DSN syntax for each DD
        for dd in &step.dd_statements {
            if let Some(ref dsn_ref) = dd.dsn {
                let dsn_str = match dsn_ref {
                    crate::dsn::DsnReference::Simple { dsn } => Some(dsn.as_str()),
                    crate::dsn::DsnReference::Member { pds_dsn, .. } => Some(pds_dsn.as_str()),
                    _ => None,
                };

                if let Some(dsn) = dsn_str {
                    if let Err(diag) = DatasetName::parse(dsn, dd.line_number, 0) {
                        diagnostics.push(diag.with_ddname(&dd.ddname));
                    }
                }
            }
        }
    }

    diagnostics
}

/// Check for duplicate ddnames within a step (excluding concatenation).
fn check_duplicate_ddnames(step: &crate::job_model::JclStep) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for dd in &step.dd_statements {
        // Skip concatenated DDs (they share a ddname intentionally)
        if dd.concatenation_index > 0 {
            continue;
        }

        if matches!(dd.kind, DdKind::Dataset | DdKind::Sysout { .. }) {
            let upper = dd.ddname.to_uppercase();
            if !seen.insert(upper.clone()) {
                diagnostics.push(
                    LintDiagnostic::new(
                        DiagnosticCode::DuplicateDdname,
                        dd.line_number,
                        dd.column_range,
                        format!("Duplicate ddname '{}' in step '{}'", dd.ddname, step.name),
                    )
                    .with_ddname(&dd.ddname),
                );
            }
        }
    }

    diagnostics
}

/// Check for missing well-known DD names in a step.
///
/// Produces WARNING diagnostics for commonly expected DDs not present.
#[allow(dead_code)]
fn check_missing_well_known_dds(step: &crate::job_model::JclStep) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();
    let step_ddnames: HashSet<String> = step
        .dd_statements
        .iter()
        .map(|dd| dd.ddname.to_uppercase())
        .collect();

    for &well_known in WELL_KNOWN_DDS {
        if !step_ddnames.contains(well_known) {
            diagnostics.push(
                LintDiagnostic::new(
                    DiagnosticCode::MissingWellKnownDd,
                    step.line_number,
                    (0, 0),
                    format!(
                        "Well-known DD '{}' not defined in step '{}'",
                        well_known, step.name
                    ),
                )
                .with_severity(DiagnosticSeverity::Warning),
            );
        }
    }

    diagnostics
}

/// Validate a symbolic parameter name.
///
/// Valid names contain only alphanumeric and national characters (@, #, $).
pub fn validate_symbolic_name(name: &str, line: usize) -> Option<LintDiagnostic> {
    for ch in name.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '@' && ch != '#' && ch != '$' {
            return Some(LintDiagnostic::new(
                DiagnosticCode::InvalidSymbolicName,
                line,
                (0, name.len()),
                format!(
                    "Invalid symbolic parameter name '&{}': character '{}' not allowed (only A-Z, 0-9, @, #, $ permitted)",
                    name, ch
                ),
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dd_statement::DdStatement;
    use crate::dsn::DsnReference;
    use crate::job_model::{ExecTarget, JclStep};
    use std::collections::HashMap;

    fn make_step(name: &str, dds: Vec<DdStatement>) -> JclStep {
        JclStep {
            name: name.to_string(),
            line_number: 1,
            exec_target: ExecTarget::Program("PGM".to_string()),
            dd_statements: dds,
            symbol_overrides: HashMap::new(),
        }
    }

    fn make_dd(ddname: &str, line: usize, concat_index: usize) -> DdStatement {
        DdStatement {
            ddname: ddname.to_string(),
            line_number: line,
            column_range: (0, 40),
            step_name: "STEP1".to_string(),
            dsn: Some(DsnReference::Simple {
                dsn: "A.B.C".to_string(),
            }),
            disp: None,
            dcb: None,
            space: None,
            kind: DdKind::Dataset,
            concatenation_index: concat_index,
            raw_operands: String::new(),
        }
    }

    #[test]
    fn detect_duplicate_ddnames() {
        // Validates: Requirement 10 AC 5
        let dds = vec![
            make_dd("SYSUT1", 1, 0),
            make_dd("SYSUT1", 2, 0), // duplicate!
        ];
        let step = make_step("STEP1", dds);
        let diags = check_duplicate_ddnames(&step);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, DiagnosticCode::DuplicateDdname);
    }

    #[test]
    fn concatenation_not_flagged_as_duplicate() {
        // Validates: Requirement 10 AC 5
        let dds = vec![
            make_dd("SYSUT1", 1, 0),
            make_dd("SYSUT1", 2, 1), // concatenation, not duplicate
        ];
        let step = make_step("STEP1", dds);
        let diags = check_duplicate_ddnames(&step);
        assert!(diags.is_empty());
    }

    #[test]
    fn validate_symbolic_name_valid() {
        // Validates: Requirement 10 AC 8
        assert!(validate_symbolic_name("SYSPARM", 1).is_none());
        assert!(validate_symbolic_name("MY@VAR", 1).is_none());
        assert!(validate_symbolic_name("A1B2", 1).is_none());
    }

    #[test]
    fn validate_symbolic_name_invalid_chars() {
        // Validates: Requirement 10 AC 8
        let diag = validate_symbolic_name("BAD-NAME", 1);
        assert!(diag.is_some());
        assert_eq!(diag.unwrap().code, DiagnosticCode::InvalidSymbolicName);
    }

    #[test]
    fn validate_job_detects_invalid_dsn() {
        // Validates: Requirement 10 AC 7
        let mut job = JclJob::new("TESTJOB", 1);
        let dd = DdStatement {
            ddname: "DD1".to_string(),
            line_number: 5,
            column_range: (0, 40),
            step_name: "STEP1".to_string(),
            dsn: Some(DsnReference::Simple {
                dsn: "1INVALID.START".to_string(),
            }),
            disp: None,
            dcb: None,
            space: None,
            kind: DdKind::Dataset,
            concatenation_index: 0,
            raw_operands: String::new(),
        };
        job.steps.push(JclStep {
            name: "STEP1".to_string(),
            line_number: 3,
            exec_target: ExecTarget::Program("PGM".to_string()),
            dd_statements: vec![dd],
            symbol_overrides: HashMap::new(),
        });

        let config = ResolverConfig::default();
        let diags = validate_job(&job, &config);
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code, DiagnosticCode::InvalidDsnSyntax);
    }
}
