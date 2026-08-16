//! Job structure model (JclJob, JclStep, ExecTarget).
//!
//! Represents the hierarchical structure of a JCL job: job → steps → DD statements.

use std::collections::HashMap;

use crate::dd_statement::DdStatement;

/// What an EXEC statement invokes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecTarget {
    /// PGM=program_name.
    Program(String),
    /// Procedure invocation (catalogued or in-stream).
    Proc(String),
}

/// A single execution step within a job.
#[derive(Debug, Clone)]
pub struct JclStep {
    /// Step name from EXEC statement.
    pub name: String,
    /// Line number of the EXEC statement.
    pub line_number: usize,
    /// Program or procedure being executed.
    pub exec_target: ExecTarget,
    /// DD statements in this step (including overrides).
    pub dd_statements: Vec<DdStatement>,
    /// Symbolic overrides from EXEC statement.
    pub symbol_overrides: HashMap<String, String>,
}

/// A parsed JCL job structure.
#[derive(Debug, Clone)]
pub struct JclJob {
    /// Job name from JOB statement (or "NOJOB" for fragments).
    pub name: String,
    /// Line number of the JOB statement.
    pub job_line: usize,
    /// Ordered list of execution steps.
    pub steps: Vec<JclStep>,
}

impl JclJob {
    /// Create a new job with the given name.
    pub fn new(name: impl Into<String>, job_line: usize) -> Self {
        Self {
            name: name.into(),
            job_line,
            steps: Vec::new(),
        }
    }

    /// Find a step by name.
    pub fn find_step(&self, step_name: &str) -> Option<&JclStep> {
        self.steps
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(step_name))
    }

    /// Find a DD statement by step name and ddname.
    pub fn find_dd(&self, step_name: &str, ddname: &str) -> Option<&DdStatement> {
        self.find_step(step_name)?
            .dd_statements
            .iter()
            .find(|dd| dd.ddname.eq_ignore_ascii_case(ddname))
    }

    /// Returns all DD statements across all steps (flattened).
    pub fn all_dd_statements(&self) -> Vec<&DdStatement> {
        self.steps
            .iter()
            .flat_map(|s| s.dd_statements.iter())
            .collect()
    }
}

/// Build a `JclJob` from parsed JCL text.
///
/// This is a lightweight parser that extracts job structure (JOB/EXEC statements)
/// and assigns DD statements to their respective steps.
pub fn build_job_model(text: &str) -> JclJob {
    let lines: Vec<&str> = text.lines().collect();
    let mut job_name = "NOJOB".to_string();
    let mut job_line = 0;
    let mut steps: Vec<JclStep> = Vec::new();
    let mut current_step: Option<JclStep> = None;
    let mut last_ddname = String::new();
    let mut concat_index: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_number = i + 1;

        if !line.starts_with("//") || line.starts_with("//*") {
            continue;
        }

        let body = &line[2..];
        let parts: Vec<&str> = body.splitn(2, char::is_whitespace).collect();
        if parts.is_empty() {
            continue;
        }

        let name_field = parts[0].trim().to_uppercase();
        let rest = parts.get(1).map(|s| s.trim_start()).unwrap_or("");
        let rest_parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
        let keyword = rest_parts
            .first()
            .map(|s| s.to_uppercase())
            .unwrap_or_default();

        // JOB statement
        if keyword == "JOB" {
            job_name = name_field.clone();
            job_line = line_number;
            continue;
        }

        // EXEC statement
        if keyword == "EXEC" {
            // Save current step
            if let Some(step) = current_step.take() {
                steps.push(step);
            }

            let exec_ops = rest_parts.get(1).copied().unwrap_or("");
            let exec_target = parse_exec_target(exec_ops);
            let overrides = parse_symbol_overrides(exec_ops);

            current_step = Some(JclStep {
                name: name_field,
                line_number,
                exec_target,
                dd_statements: Vec::new(),
                symbol_overrides: overrides,
            });
            last_ddname.clear();
            concat_index = 0;
            continue;
        }

        // DD statement
        if keyword == "DD" {
            let _operands = rest_parts.get(1).copied().unwrap_or("");

            if let Some(ref mut step) = current_step {
                if name_field.is_empty() && !last_ddname.is_empty() {
                    // Concatenation
                    concat_index += 1;
                } else {
                    last_ddname = name_field.clone();
                    concat_index = 0;
                }

                let dd = crate::parser::parse_single_dd(line, line_number);
                if let Ok(mut dd) = dd {
                    dd.step_name = step.name.clone();
                    dd.concatenation_index = concat_index;
                    if dd.ddname.is_empty() {
                        dd.ddname = last_ddname.clone();
                    }
                    step.dd_statements.push(dd);
                }
            }
            continue;
        }
    }

    // Save last step
    if let Some(step) = current_step {
        steps.push(step);
    }

    JclJob {
        name: job_name,
        job_line,
        steps,
    }
}

/// Parse the EXEC target (PGM= or procedure name).
fn parse_exec_target(operands: &str) -> ExecTarget {
    let upper = operands.to_uppercase();
    if let Some(pgm) = upper.strip_prefix("PGM=") {
        let name = pgm.split(',').next().unwrap_or("").trim().to_string();
        ExecTarget::Program(name)
    } else {
        let name = upper.split(',').next().unwrap_or("").trim().to_string();
        ExecTarget::Proc(name)
    }
}

/// Parse symbolic overrides from EXEC operands.
fn parse_symbol_overrides(operands: &str) -> HashMap<String, String> {
    let mut overrides = HashMap::new();
    let upper = operands.to_uppercase();

    // Skip the first operand (PGM= or proc name), process remaining key=value pairs
    for part in upper.split(',').skip(1) {
        if let Some((key, value)) = part.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('\'');
            // Only include non-keyword overrides (not standard EXEC keywords)
            if !matches!(
                key,
                "PGM"
                    | "PROC"
                    | "PARM"
                    | "COND"
                    | "REGION"
                    | "TIME"
                    | "ACCT"
                    | "ADDRSPC"
                    | "DYNAMNBR"
            ) {
                overrides.insert(key.to_string(), value.to_string());
            }
        }
    }

    overrides
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_job_model_basic_job() {
        // Validates: Requirement 12 AC 1, AC 2, AC 3
        let jcl = "\
//MYJOB   JOB (ACCT),'PGMR'
//STEP1   EXEC PGM=IEFBR14
//DD1     DD DSN=MY.DATA,DISP=SHR
//STEP2   EXEC PGM=IEBGENER
//SYSUT1  DD DSN=INPUT.DATA,DISP=SHR
//SYSUT2  DD DSN=OUTPUT.DATA,DISP=(NEW,CATLG)
";
        let job = build_job_model(jcl);
        assert_eq!(job.name, "MYJOB");
        assert_eq!(job.steps.len(), 2);
        assert_eq!(job.steps[0].name, "STEP1");
        assert_eq!(
            job.steps[0].exec_target,
            ExecTarget::Program("IEFBR14".to_string())
        );
        assert_eq!(job.steps[0].dd_statements.len(), 1);
        assert_eq!(job.steps[1].name, "STEP2");
        assert_eq!(job.steps[1].dd_statements.len(), 2);
    }

    #[test]
    fn build_job_model_no_job_statement() {
        // Validates: Requirement 12 AC 8
        let jcl = "\
//STEP1   EXEC PGM=IEFBR14
//DD1     DD DSN=MY.DATA,DISP=SHR
";
        let job = build_job_model(jcl);
        assert_eq!(job.name, "NOJOB");
    }

    #[test]
    fn find_step_and_dd() {
        let jcl = "\
//MYJOB  JOB (ACCT),'PGMR'
//STEP1  EXEC PGM=IEFBR14
//INPUT  DD DSN=MY.DATA,DISP=SHR
";
        let job = build_job_model(jcl);
        assert!(job.find_step("STEP1").is_some());
        assert!(job.find_step("STEP2").is_none());
        assert!(job.find_dd("STEP1", "INPUT").is_some());
        assert!(job.find_dd("STEP1", "OUTPUT").is_none());
    }

    #[test]
    fn exec_with_proc_invocation() {
        // Validates: Requirement 12 AC 2
        let jcl = "\
//MYJOB  JOB (ACCT),'PGMR'
//STEP1  EXEC MYPROC,PARM='TEST'
//DD1    DD DSN=MY.DATA,DISP=SHR
";
        let job = build_job_model(jcl);
        assert_eq!(
            job.steps[0].exec_target,
            ExecTarget::Proc("MYPROC".to_string())
        );
    }
}
