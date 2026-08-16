//! FFJCL (FileForge Job Control Language) parser.
//!
//! Parses job definitions into a structured AST, validates them,
//! and produces meaningful error messages for invalid input.

use serde::{Deserialize, Serialize};

use crate::error::JesError;

// ─── AST Types ───────────────────────────────────────────────────────────────

/// A parsed FFJCL job definition.
///
/// Validates: Requirement 2 AC 1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FfjclDefinition {
    /// Job name from the JOB statement.
    pub job_name: String,
    /// Optional owner override.
    pub owner: Option<String>,
    /// Priority override (0 = default).
    pub priority: Option<u32>,
    /// Job class (optional).
    pub class: Option<String>,
    /// Execution steps in order.
    pub steps: Vec<FfjclStep>,
    /// Raw source text (for log display).
    pub source: String,
}

/// A single execution step within an FFJCL job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FfjclStep {
    /// Step name.
    pub name: String,
    /// Program or script to execute.
    pub program: String,
    /// Arguments to pass to the program.
    pub args: Vec<String>,
    /// DD statements for this step.
    pub dds: Vec<FfjclDd>,
    /// Optional step condition (COND parameter equivalent).
    pub condition: Option<StepCondition>,
}

/// A DD statement within an FFJCL step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FfjclDd {
    /// DD name (1-8 alphanumeric characters).
    pub ddname: String,
    /// Dataset name reference.
    pub dsn: Option<String>,
    /// Disposition string (NEW, OLD, SHR, MOD).
    pub disp: Option<String>,
    /// Whether this is SYSOUT.
    pub sysout: bool,
    /// Whether this is DUMMY.
    pub dummy: bool,
    /// Inline data content.
    pub inline_data: Option<String>,
}

/// Condition code check for step execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepCondition {
    /// Conditions: (code, operator) pairs — if ANY is true, step is bypassed.
    pub conditions: Vec<(i32, CondOperator)>,
}

/// Comparison operators for COND parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CondOperator {
    Gt,
    Ge,
    Eq,
    Lt,
    Le,
    Ne,
}

impl CondOperator {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GT" => Some(Self::Gt),
            "GE" => Some(Self::Ge),
            "EQ" => Some(Self::Eq),
            "LT" => Some(Self::Lt),
            "LE" => Some(Self::Le),
            "NE" => Some(Self::Ne),
            _ => None,
        }
    }

    /// Evaluates the condition: returns true if the step should be bypassed.
    pub fn evaluate(self, step_rc: i32, threshold: i32) -> bool {
        match self {
            Self::Gt => step_rc > threshold,
            Self::Ge => step_rc >= threshold,
            Self::Eq => step_rc == threshold,
            Self::Lt => step_rc < threshold,
            Self::Le => step_rc <= threshold,
            Self::Ne => step_rc != threshold,
        }
    }
}

// ─── Parser ──────────────────────────────────────────────────────────────────

/// Parses FFJCL text into a structured definition.
///
/// FFJCL format:
/// ```text
/// //JOBNAME  JOB  [PRIORITY=n] [CLASS=c] [OWNER=user]
/// //STEPNAME EXEC PGM=program[,ARGS='arg1 arg2']
/// //DDNAME   DD   DSN=dataset.name,DISP=SHR
/// //DDNAME   DD   SYSOUT=*
/// //DDNAME   DD   DUMMY
/// ```
///
/// Lines starting with `*` or `//` followed by `*` are comments.
/// Lines ending with `,` continue on the next line.
///
/// Validates: Requirement 2 AC 1, AC 7
pub fn parse_ffjcl(input: &str) -> Result<FfjclDefinition, JesError> {
    let source = input.to_string();
    let lines = preprocess_lines(input);

    if lines.is_empty() {
        return Err(JesError::FfjclParseError {
            line: 0,
            message: "empty job definition".to_string(),
        });
    }

    let mut job_name = None;
    let mut owner = None;
    let mut priority = None;
    let mut class = None;
    let mut steps: Vec<FfjclStep> = Vec::new();
    let mut current_step: Option<FfjclStep> = None;

    for (line_num, line) in &lines {
        let line = line.trim();

        // Skip blank lines and comments
        if line.is_empty() || line.starts_with("//*") || line.starts_with("*") {
            continue;
        }

        if !line.starts_with("//") {
            // Inline data for current DD — skip for now
            continue;
        }

        let rest = &line[2..];
        if rest.trim().is_empty() {
            continue;
        }

        // Parse: //NAME  KEYWORD  params
        // Split on whitespace runs to handle multiple spaces
        let tokens: Vec<&str> = rest.split_whitespace().collect();
        let name_field = tokens.first().copied().unwrap_or("");
        let keyword = tokens.get(1).copied().unwrap_or("").to_uppercase();
        // params: rejoin everything from index 2 onward
        let params_str = if tokens.len() > 2 {
            tokens[2..].join(" ")
        } else {
            String::new()
        };
        let params_str = params_str.as_str();

        match keyword.as_str() {
            "JOB" => {
                if name_field.is_empty() {
                    return Err(JesError::FfjclParseError {
                        line: *line_num,
                        message: "JOB statement requires a job name".to_string(),
                    });
                }
                job_name = Some(name_field.to_string());
                parse_job_params(params_str, &mut owner, &mut priority, &mut class);
            }
            "EXEC" => {
                // Save previous step
                if let Some(step) = current_step.take() {
                    steps.push(step);
                }
                let step = parse_exec_statement(name_field, params_str, *line_num)?;
                current_step = Some(step);
            }
            "DD" => {
                let dd = parse_dd_statement(name_field, params_str, *line_num)?;
                if let Some(ref mut step) = current_step {
                    step.dds.push(dd);
                }
                // DD before any EXEC is silently ignored
            }
            _ => {
                // Unknown keyword — skip with no error (forward compatibility)
            }
        }
    }

    // Save last step
    if let Some(step) = current_step {
        steps.push(step);
    }

    let job_name = job_name.ok_or_else(|| JesError::FfjclParseError {
        line: 0,
        message: "missing JOB statement".to_string(),
    })?;

    Ok(FfjclDefinition {
        job_name,
        owner,
        priority,
        class,
        steps,
        source,
    })
}

/// Preprocesses FFJCL lines: joins continuation lines, strips sequence numbers.
fn preprocess_lines(input: &str) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_num = 0usize;
    let mut in_continuation = false;

    for (i, line) in input.lines().enumerate() {
        let line_num = i + 1;

        // Strip sequence numbers (columns 73-80 if line is 80 chars)
        let line = if line.len() >= 80 {
            let seq = &line[72..80];
            if seq.chars().all(|c| c.is_ascii_digit() || c == ' ') {
                &line[..72]
            } else {
                line
            }
        } else {
            line
        };

        if in_continuation {
            // Continuation: skip leading // and whitespace
            let trimmed = line.trim_start_matches('/').trim_start();
            current_line.push_str(trimmed);
            in_continuation = false;
        } else {
            if !current_line.is_empty() {
                result.push((current_num, current_line.clone()));
                current_line.clear();
            }
            current_num = line_num;
            current_line = line.to_string();
        }

        // Check for continuation (line ends with comma, not in comment)
        let trimmed = current_line.trim_end();
        if trimmed.ends_with(',') && !trimmed.starts_with("//*") {
            in_continuation = true;
        }
    }

    if !current_line.is_empty() {
        result.push((current_num, current_line));
    }

    result
}

fn parse_job_params(
    params: &str,
    owner: &mut Option<String>,
    priority: &mut Option<u32>,
    class: &mut Option<String>,
) {
    for param in params.split(',') {
        let param = param.trim();
        if let Some(val) = param.strip_prefix("PRIORITY=") {
            *priority = val.trim().parse().ok();
        } else if let Some(val) = param.strip_prefix("CLASS=") {
            *class = Some(val.trim().to_string());
        } else if let Some(val) = param.strip_prefix("OWNER=") {
            *owner = Some(val.trim().to_string());
        }
    }
}

fn parse_exec_statement(name: &str, params: &str, line_num: usize) -> Result<FfjclStep, JesError> {
    if name.is_empty() {
        return Err(JesError::FfjclParseError {
            line: line_num,
            message: "EXEC statement requires a step name".to_string(),
        });
    }

    let mut program = String::new();
    let mut args = Vec::new();
    let mut condition = None;

    for param in params.split(',') {
        let param = param.trim();
        if let Some(pgm) = param.strip_prefix("PGM=") {
            program = pgm.trim().to_string();
        } else if let Some(arg_str) = param.strip_prefix("ARGS=") {
            let arg_str = arg_str.trim().trim_matches('\'');
            args = arg_str.split_whitespace().map(str::to_string).collect();
        } else if let Some(cond_str) = param.strip_prefix("COND=") {
            condition = parse_cond(cond_str.trim());
        }
    }

    if program.is_empty() {
        return Err(JesError::FfjclParseError {
            line: line_num,
            message: format!("EXEC statement for step '{}' missing PGM= parameter", name),
        });
    }

    Ok(FfjclStep {
        name: name.to_string(),
        program,
        args,
        dds: Vec::new(),
        condition,
    })
}

fn parse_cond(s: &str) -> Option<StepCondition> {
    // COND=(code,op) or COND=((code,op),(code,op),...)
    let s = s.trim_matches(|c| c == '(' || c == ')');
    let mut conditions = Vec::new();

    for part in s.split("),(") {
        let part = part.trim_matches(|c| c == '(' || c == ')');
        let mut iter = part.splitn(2, ',');
        let code: i32 = iter.next()?.trim().parse().ok()?;
        let op = CondOperator::from_str(iter.next()?.trim())?;
        conditions.push((code, op));
    }

    if conditions.is_empty() {
        None
    } else {
        Some(StepCondition { conditions })
    }
}

fn parse_dd_statement(name: &str, params: &str, line_num: usize) -> Result<FfjclDd, JesError> {
    if name.is_empty() {
        return Err(JesError::FfjclParseError {
            line: line_num,
            message: "DD statement requires a DD name".to_string(),
        });
    }

    let mut dsn = None;
    let mut disp = None;
    let mut sysout = false;
    let mut dummy = false;
    let mut inline_data = None;

    let params_upper = params.to_uppercase();

    if params_upper.trim() == "DUMMY" {
        dummy = true;
    } else if params_upper.trim().starts_with("SYSOUT=") {
        sysout = true;
    } else if params_upper.trim().starts_with("DATA") || params_upper.trim().starts_with("*") {
        inline_data = Some(String::new()); // placeholder
    } else {
        for param in params.split(',') {
            let param = param.trim();
            let param_upper = param.to_uppercase();
            if let Some(val) = param_upper.strip_prefix("DSN=") {
                dsn = Some(val.trim().to_string());
            } else if let Some(val) = param_upper.strip_prefix("DISP=") {
                disp = Some(
                    val.trim()
                        .trim_matches(|c| c == '(' || c == ')')
                        .to_string(),
                );
            }
        }
    }

    Ok(FfjclDd {
        ddname: name.to_string(),
        dsn,
        disp,
        sysout,
        dummy,
        inline_data,
    })
}

// ─── Validator ───────────────────────────────────────────────────────────────

/// A validation issue found in an FFJCL definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationIssue {
    /// Line number (0 if not applicable).
    pub line: usize,
    /// Description of the issue.
    pub message: String,
}

/// Validates a parsed FFJCL definition.
///
/// Returns Ok(()) if valid, or Err with the first validation error.
///
/// Validates: Requirement 2 AC 7
pub fn validate_definition(def: &FfjclDefinition) -> Result<(), JesError> {
    // Job name must be present and 1-8 chars
    if def.job_name.is_empty() {
        return Err(JesError::ValidationError {
            line: 0,
            message: "job name is required".to_string(),
        });
    }
    if def.job_name.len() > 8 {
        return Err(JesError::ValidationError {
            line: 0,
            message: format!("job name '{}' exceeds 8 characters", def.job_name),
        });
    }

    // At least one step required
    if def.steps.is_empty() {
        return Err(JesError::ValidationError {
            line: 0,
            message: "job must have at least one EXEC step".to_string(),
        });
    }

    // Validate each step
    for step in &def.steps {
        if step.name.is_empty() {
            return Err(JesError::ValidationError {
                line: 0,
                message: "step name is required".to_string(),
            });
        }
        if step.program.is_empty() {
            return Err(JesError::ValidationError {
                line: 0,
                message: format!("step '{}' has no program (PGM=)", step.name),
            });
        }

        // DD names must be unique within a step
        let mut seen_dds = std::collections::HashSet::new();
        for dd in &step.dds {
            if !seen_dds.insert(dd.ddname.to_uppercase()) {
                return Err(JesError::ValidationError {
                    line: 0,
                    message: format!("duplicate DD name '{}' in step '{}'", dd.ddname, step.name),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_valid_job() {
        // Validates: Requirement 2 AC 1
        let input = "//MYJOB   JOB\n//STEP1   EXEC PGM=IEFBR14\n";
        let def = parse_ffjcl(input).expect("should parse");
        assert_eq!(def.job_name, "MYJOB");
        assert_eq!(def.steps.len(), 1);
        assert_eq!(def.steps[0].name, "STEP1");
        assert_eq!(def.steps[0].program, "IEFBR14");
    }

    #[test]
    fn parse_job_with_priority_and_class() {
        let input = "//MYJOB   JOB  PRIORITY=5,CLASS=A\n//STEP1   EXEC PGM=PROG1\n";
        let def = parse_ffjcl(input).expect("should parse");
        assert_eq!(def.priority, Some(5));
        assert_eq!(def.class, Some("A".to_string()));
    }

    #[test]
    fn parse_multi_step_job() {
        let input = concat!(
            "//MYJOB   JOB\n",
            "//STEP1   EXEC PGM=PROG1\n",
            "//SYSOUT  DD   SYSOUT=*\n",
            "//STEP2   EXEC PGM=PROG2\n",
            "//INPUT   DD   DSN=MY.DATA,DISP=SHR\n",
        );
        let def = parse_ffjcl(input).expect("should parse");
        assert_eq!(def.steps.len(), 2);
        assert_eq!(def.steps[0].dds.len(), 1);
        assert!(def.steps[0].dds[0].sysout);
        assert_eq!(def.steps[1].dds.len(), 1);
        assert_eq!(def.steps[1].dds[0].dsn, Some("MY.DATA".to_string()));
    }

    #[test]
    fn parse_missing_job_statement_returns_error() {
        // Validates: Requirement 2 AC 7
        let input = "//STEP1   EXEC PGM=PROG1\n";
        let result = parse_ffjcl(input);
        assert!(result.is_err());
        match result.unwrap_err() {
            JesError::FfjclParseError { message, .. } => {
                assert!(message.contains("JOB statement"));
            }
            e => panic!("unexpected error: {e}"),
        }
    }

    #[test]
    fn parse_empty_input_returns_error() {
        let result = parse_ffjcl("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_comments_are_ignored() {
        let input = concat!(
            "//* This is a comment\n",
            "//MYJOB   JOB\n",
            "//* Another comment\n",
            "//STEP1   EXEC PGM=PROG1\n",
        );
        let def = parse_ffjcl(input).expect("should parse");
        assert_eq!(def.job_name, "MYJOB");
        assert_eq!(def.steps.len(), 1);
    }

    #[test]
    fn parse_exec_missing_pgm_returns_error() {
        let input = "//MYJOB   JOB\n//STEP1   EXEC\n";
        let result = parse_ffjcl(input);
        assert!(result.is_err());
    }

    #[test]
    fn validate_rejects_empty_job_name() {
        // Validates: Requirement 2 AC 7
        let def = FfjclDefinition {
            job_name: String::new(),
            owner: None,
            priority: None,
            class: None,
            steps: vec![FfjclStep {
                name: "STEP1".to_string(),
                program: "PROG1".to_string(),
                args: vec![],
                dds: vec![],
                condition: None,
            }],
            source: String::new(),
        };
        assert!(validate_definition(&def).is_err());
    }

    #[test]
    fn validate_rejects_no_steps() {
        // Validates: Requirement 2 AC 7
        let def = FfjclDefinition {
            job_name: "MYJOB".to_string(),
            owner: None,
            priority: None,
            class: None,
            steps: vec![],
            source: String::new(),
        };
        let err = validate_definition(&def).unwrap_err();
        match err {
            JesError::ValidationError { message, .. } => {
                assert!(message.contains("at least one"));
            }
            e => panic!("unexpected: {e}"),
        }
    }

    #[test]
    fn validate_rejects_duplicate_dd_names() {
        // Validates: Requirement 2 AC 7
        let def = FfjclDefinition {
            job_name: "MYJOB".to_string(),
            owner: None,
            priority: None,
            class: None,
            steps: vec![FfjclStep {
                name: "STEP1".to_string(),
                program: "PROG1".to_string(),
                args: vec![],
                dds: vec![
                    FfjclDd {
                        ddname: "SYSOUT".to_string(),
                        dsn: None,
                        disp: None,
                        sysout: true,
                        dummy: false,
                        inline_data: None,
                    },
                    FfjclDd {
                        ddname: "SYSOUT".to_string(),
                        dsn: None,
                        disp: None,
                        sysout: true,
                        dummy: false,
                        inline_data: None,
                    },
                ],
                condition: None,
            }],
            source: String::new(),
        };
        assert!(validate_definition(&def).is_err());
    }

    #[test]
    fn cond_operator_evaluate() {
        assert!(CondOperator::Gt.evaluate(5, 4));
        assert!(!CondOperator::Gt.evaluate(4, 4));
        assert!(CondOperator::Eq.evaluate(0, 0));
        assert!(CondOperator::Ne.evaluate(4, 0));
        assert!(CondOperator::Le.evaluate(4, 4));
        assert!(CondOperator::Lt.evaluate(3, 4));
    }
}
