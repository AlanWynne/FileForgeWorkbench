//! Referback resolution.
//!
//! Resolves referback DSN references (`*.stepname.ddname`) by following the chain
//! to the ultimate DSN, respecting the configurable depth limit.

use crate::diagnostic::{DiagnosticCode, LintDiagnostic};
use crate::dsn::DsnReference;
use crate::job_model::JclJob;

/// Resolve a referback reference against the job model.
///
/// Follows referback chains recursively up to the configured depth limit.
/// Returns the ultimate DSN reference (non-referback) or a diagnostic on failure.
pub fn resolve_referback(
    referback: &DsnReference,
    job: &JclJob,
    line: usize,
    max_depth: usize,
) -> Result<DsnReference, LintDiagnostic> {
    resolve_referback_recursive(referback, job, line, max_depth, 0)
}

fn resolve_referback_recursive(
    referback: &DsnReference,
    job: &JclJob,
    line: usize,
    max_depth: usize,
    current_depth: usize,
) -> Result<DsnReference, LintDiagnostic> {
    if current_depth > max_depth {
        return Err(LintDiagnostic::new(
            DiagnosticCode::ReferbackChainTooDeep,
            line,
            (0, 0),
            format!("Referback chain too deep (limit: {})", max_depth),
        ));
    }

    match referback {
        DsnReference::Referback {
            step_name,
            proc_step,
            ddname,
        } => {
            // Find the target step
            let step = job.find_step(step_name).ok_or_else(|| {
                LintDiagnostic::new(
                    DiagnosticCode::ReferbackNotFound,
                    line,
                    (0, 0),
                    format!("Referback target step not found: {}", step_name),
                )
            })?;

            // If proc_step is specified, we'd look into procedure expansion
            // For now, handle simple case
            if proc_step.is_some() {
                // TODO: proc step handling - for now just look in the step's DDs
            }

            // Find the target DD
            let target_dd = step
                .dd_statements
                .iter()
                .find(|dd| dd.ddname.eq_ignore_ascii_case(ddname))
                .ok_or_else(|| {
                    LintDiagnostic::new(
                        DiagnosticCode::ReferbackNotFound,
                        line,
                        (0, 0),
                        format!(
                            "Referback target DD not found: {} in step {}",
                            ddname, step_name
                        ),
                    )
                })?;

            // Get the target's DSN
            match &target_dd.dsn {
                Some(dsn_ref) => {
                    // If the target is itself a referback, follow the chain
                    if dsn_ref.is_referback() {
                        resolve_referback_recursive(
                            dsn_ref,
                            job,
                            line,
                            max_depth,
                            current_depth + 1,
                        )
                    } else {
                        Ok(dsn_ref.clone())
                    }
                }
                None => Err(LintDiagnostic::new(
                    DiagnosticCode::ReferbackNotFound,
                    line,
                    (0, 0),
                    format!(
                        "Referback target DD {} in step {} has no DSN",
                        ddname, step_name
                    ),
                )),
            }
        }
        // Not a referback — return as-is
        other => Ok(other.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dd_statement::{DdKind, DdStatement};
    use crate::job_model::{ExecTarget, JclJob, JclStep};
    use std::collections::HashMap;

    fn make_job_with_steps(steps: Vec<(&str, Vec<(&str, DsnReference)>)>) -> JclJob {
        let mut job = JclJob::new("TESTJOB", 1);
        for (step_name, dds) in steps {
            let dd_statements: Vec<DdStatement> = dds
                .into_iter()
                .map(|(ddname, dsn)| DdStatement {
                    ddname: ddname.to_string(),
                    line_number: 1,
                    column_range: (0, 40),
                    step_name: step_name.to_string(),
                    dsn: Some(dsn),
                    disp: None,
                    dcb: None,
                    space: None,
                    kind: DdKind::Dataset,
                    concatenation_index: 0,
                    raw_operands: String::new(),
                })
                .collect();
            job.steps.push(JclStep {
                name: step_name.to_string(),
                line_number: 1,
                exec_target: ExecTarget::Program("PGM".to_string()),
                dd_statements,
                symbol_overrides: HashMap::new(),
            });
        }
        job
    }

    #[test]
    fn resolve_simple_referback() {
        // Validates: Requirement 7 AC 2
        let job = make_job_with_steps(vec![
            (
                "STEP1",
                vec![(
                    "SYSUT1",
                    DsnReference::Simple {
                        dsn: "MY.DATA.SET".to_string(),
                    },
                )],
            ),
            ("STEP2", vec![]),
        ]);

        let referback = DsnReference::Referback {
            step_name: "STEP1".to_string(),
            proc_step: None,
            ddname: "SYSUT1".to_string(),
        };

        let result = resolve_referback(&referback, &job, 10, 10);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            DsnReference::Simple {
                dsn: "MY.DATA.SET".to_string()
            }
        );
    }

    #[test]
    fn resolve_referback_step_not_found() {
        // Validates: Requirement 7 AC 4
        let job = make_job_with_steps(vec![(
            "STEP1",
            vec![(
                "SYSUT1",
                DsnReference::Simple {
                    dsn: "A.B".to_string(),
                },
            )],
        )]);

        let referback = DsnReference::Referback {
            step_name: "NOSUCH".to_string(),
            proc_step: None,
            ddname: "SYSUT1".to_string(),
        };

        let result = resolve_referback(&referback, &job, 5, 10);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("step not found"));
    }

    #[test]
    fn resolve_referback_dd_not_found() {
        // Validates: Requirement 7 AC 5
        let job = make_job_with_steps(vec![(
            "STEP1",
            vec![(
                "SYSUT1",
                DsnReference::Simple {
                    dsn: "A.B".to_string(),
                },
            )],
        )]);

        let referback = DsnReference::Referback {
            step_name: "STEP1".to_string(),
            proc_step: None,
            ddname: "NOSUCHDD".to_string(),
        };

        let result = resolve_referback(&referback, &job, 5, 10);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("DD not found"));
    }

    #[test]
    fn resolve_referback_chain() {
        // Validates: Requirement 7 AC 6
        let job = make_job_with_steps(vec![
            (
                "STEP1",
                vec![(
                    "DD1",
                    DsnReference::Simple {
                        dsn: "FINAL.DATA".to_string(),
                    },
                )],
            ),
            (
                "STEP2",
                vec![(
                    "DD2",
                    DsnReference::Referback {
                        step_name: "STEP1".to_string(),
                        proc_step: None,
                        ddname: "DD1".to_string(),
                    },
                )],
            ),
        ]);

        let referback = DsnReference::Referback {
            step_name: "STEP2".to_string(),
            proc_step: None,
            ddname: "DD2".to_string(),
        };

        let result = resolve_referback(&referback, &job, 10, 10);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            DsnReference::Simple {
                dsn: "FINAL.DATA".to_string()
            }
        );
    }

    #[test]
    fn resolve_referback_chain_depth_exceeded() {
        // Validates: Requirement 7 AC 6
        // Create a circular reference (which will hit depth limit)
        let job = make_job_with_steps(vec![(
            "STEP1",
            vec![(
                "DD1",
                DsnReference::Referback {
                    step_name: "STEP1".to_string(),
                    proc_step: None,
                    ddname: "DD1".to_string(),
                },
            )],
        )]);

        let referback = DsnReference::Referback {
            step_name: "STEP1".to_string(),
            proc_step: None,
            ddname: "DD1".to_string(),
        };

        let result = resolve_referback(&referback, &job, 10, 3);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("too deep"));
    }
}
