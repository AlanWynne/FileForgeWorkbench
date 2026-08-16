//! JCL parser — DD statement parsing, continuation line handling, operand extraction.
//!
//! Parses JCL text into structured DD statements with all operands extracted.

use crate::dd_statement::{DdKind, DdStatement};
use crate::diagnostic::{DiagnosticCode, LintDiagnostic};
use crate::dsn::parse_dsn_reference;
use crate::operands::{DcbAttributes, DispParameter, SpaceAllocation};

/// Result of parsing JCL text.
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Parsed DD statements.
    pub dd_statements: Vec<DdStatement>,
    /// Diagnostics produced during parsing.
    pub diagnostics: Vec<LintDiagnostic>,
}

/// Parse all DD statements from JCL text.
///
/// Handles continuation lines (column 72 non-blank + next `// ` line),
/// concatenation detection, and all DD operand extraction.
pub fn parse_jcl_statements(text: &str) -> ParseResult {
    let lines: Vec<&str> = text.lines().collect();
    let mut dd_statements = Vec::new();
    let mut diagnostics = Vec::new();
    let mut current_step = String::new();
    let mut last_ddname = String::new();
    let mut concat_index: usize = 0;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let line_number = i + 1; // 1-based

        // Skip non-JCL lines (comments, JES2/JES3, blank)
        if !line.starts_with("//") || line.starts_with("//*") {
            i += 1;
            continue;
        }

        // Join continuation lines
        let full_statement = join_continuations(&lines, &mut i);

        // Determine statement type
        let stmt_body = &full_statement[2..]; // skip "//"

        // EXEC statement — update current step
        if let Some(step_info) = parse_exec_statement(stmt_body) {
            current_step = step_info;
            concat_index = 0;
            last_ddname.clear();
            i += 1;
            continue;
        }

        // JOB statement — skip
        if is_job_statement(stmt_body) {
            i += 1;
            continue;
        }

        // DD statement detection
        if let Some(dd_result) = try_parse_dd(
            stmt_body,
            line_number,
            &current_step,
            &last_ddname,
            concat_index,
        ) {
            match dd_result {
                Ok((dd, new_ddname, new_concat)) => {
                    last_ddname = new_ddname;
                    concat_index = new_concat;
                    dd_statements.push(dd);
                }
                Err(diag) => {
                    diagnostics.push(diag);
                }
            }
        }

        i += 1;
    }

    ParseResult {
        dd_statements,
        diagnostics,
    }
}

/// Parse a single DD statement line (for incremental resolution).
pub fn parse_single_dd(line: &str, line_number: usize) -> Result<DdStatement, LintDiagnostic> {
    let body = line.strip_prefix("//").unwrap_or(line);

    match try_parse_dd(body, line_number, "", "", 0) {
        Some(Ok((dd, _, _))) => Ok(dd),
        Some(Err(diag)) => Err(diag),
        None => Err(LintDiagnostic::new(
            DiagnosticCode::SyntaxError,
            line_number,
            (0, line.len()),
            "Not a DD statement",
        )),
    }
}

/// Join continuation lines starting at position `i`.
/// Advances `i` past any continuation lines consumed.
fn join_continuations(lines: &[&str], i: &mut usize) -> String {
    let mut result = lines[*i].to_string();

    // Check if line has continuation (column 72 non-blank, line at least 72 chars)
    while result.len() >= 72 && !result[71..72].trim().is_empty() {
        let next = *i + 1;
        if next >= lines.len() {
            break;
        }
        let next_line = lines[next];
        // Continuation line must start with "// " (slashes + spaces then operands)
        if next_line.starts_with("//") && next_line.len() > 2 {
            let cont_part = &next_line[2..];
            if cont_part.starts_with(' ') {
                // Trim the continuation marker from column 72 of previous line
                result.truncate(71);
                result.push_str(cont_part.trim_start());
                *i = next;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    result
}

/// Check if a statement body is a JOB statement.
fn is_job_statement(body: &str) -> bool {
    let upper = body.to_uppercase();
    // Format: name JOB ...
    let parts: Vec<&str> = upper.split_whitespace().collect();
    parts.len() >= 2 && parts[1] == "JOB"
}

/// Parse EXEC statement, returning the step name.
fn parse_exec_statement(body: &str) -> Option<String> {
    let parts: Vec<&str> = body.split_whitespace().collect();
    if parts.len() >= 2 && parts[1].to_uppercase() == "EXEC" {
        Some(parts[0].to_uppercase())
    } else {
        None
    }
}

/// Attempt to parse a DD statement from the body (after "//").
/// Returns None if this is not a DD statement.
/// Returns Some(Ok(...)) on success, Some(Err(...)) on syntax error.
fn try_parse_dd(
    body: &str,
    line_number: usize,
    current_step: &str,
    last_ddname: &str,
    current_concat_index: usize,
) -> Option<Result<(DdStatement, String, usize), LintDiagnostic>> {
    // Split into name field and rest
    let parts: Vec<&str> = body.splitn(2, char::is_whitespace).collect();

    if parts.is_empty() {
        return None;
    }

    let name_field = parts[0].trim().to_uppercase();
    let rest = parts.get(1).map(|s| s.trim()).unwrap_or("");

    // Check if this line contains " DD " keyword
    let rest_parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();

    // Concatenation: blank name field, follows a DD
    if name_field.is_empty() && !last_ddname.is_empty() {
        // This might be a concatenation DD
        if let Some(keyword) = rest_parts.first() {
            if keyword.to_uppercase() == "DD" {
                let operands = rest_parts.get(1).map(|s| s.trim()).unwrap_or("");
                let new_concat = current_concat_index + 1;
                return Some(
                    parse_dd_operands(last_ddname, operands, line_number, current_step, new_concat)
                        .map(|dd| (dd, last_ddname.to_string(), new_concat)),
                );
            }
        }
        return None;
    }

    // Standard DD: name DD operands
    if rest_parts.first().map(|s| s.to_uppercase()) == Some("DD".to_string()) {
        let operands = rest_parts.get(1).map(|s| s.trim()).unwrap_or("");
        return Some(
            parse_dd_operands(&name_field, operands, line_number, current_step, 0)
                .map(|dd| (dd, name_field.clone(), 0)),
        );
    }

    None
}

/// Parse the operand field of a DD statement.
fn parse_dd_operands(
    ddname: &str,
    operands: &str,
    line_number: usize,
    step_name: &str,
    concat_index: usize,
) -> Result<DdStatement, LintDiagnostic> {
    let operands_upper = operands.to_uppercase();
    let raw_operands = operands.to_string();

    // Check for special DD types first
    if operands_upper.starts_with('*') || operands_upper.starts_with("DATA") {
        return Ok(DdStatement {
            ddname: ddname.to_string(),
            line_number,
            column_range: (0, operands.len()),
            step_name: step_name.to_string(),
            dsn: None,
            disp: None,
            dcb: None,
            space: None,
            kind: DdKind::Inline,
            concatenation_index: concat_index,
            raw_operands,
        });
    }

    if operands_upper == "DUMMY"
        || operands_upper.starts_with("DUMMY,")
        || operands_upper.starts_with("DUMMY ")
    {
        return Ok(DdStatement {
            ddname: ddname.to_string(),
            line_number,
            column_range: (0, operands.len()),
            step_name: step_name.to_string(),
            dsn: None,
            disp: None,
            dcb: None,
            space: None,
            kind: DdKind::Dummy,
            concatenation_index: concat_index,
            raw_operands,
        });
    }

    // Parse operand key=value pairs
    let tokens = tokenize_operands(operands);

    let mut dsn: Option<String> = None;
    let mut disp: Option<DispParameter> = None;
    let mut dcb: Option<DcbAttributes> = None;
    let mut space: Option<SpaceAllocation> = None;
    let mut sysout_class: Option<char> = None;
    let mut kind = DdKind::Dataset;

    for token in &tokens {
        let token_upper = token.to_uppercase();

        if let Some((key, value)) = token_upper.split_once('=') {
            match key.trim() {
                "DSN" | "DSNAME" => {
                    // Strip quotes if present
                    let v = value.trim().trim_matches('\'');
                    dsn = Some(v.to_string());
                }
                "DISP" => {
                    disp = DispParameter::parse(value.trim());
                }
                "DCB" => {
                    let dcb_text = value.trim();
                    let inner = dcb_text
                        .strip_prefix('(')
                        .and_then(|s| s.strip_suffix(')'))
                        .unwrap_or(dcb_text);
                    dcb = Some(DcbAttributes::parse(inner));
                }
                "SPACE" => {
                    let space_val = value.trim();
                    // The value after SPACE= might be "(TRK,(100,50,5))" or "TRK,(100,50,5))"
                    // Wrap in parens if not already wrapped
                    if space_val.starts_with('(') {
                        space = SpaceAllocation::parse(space_val);
                    } else {
                        space = SpaceAllocation::parse(&format!("({})", space_val));
                    }
                }
                "SYSOUT" => {
                    let c = value.trim().chars().next().unwrap_or('A');
                    sysout_class = Some(c);
                    kind = DdKind::Sysout { class: c };
                }
                _ => {} // ignore unknown operands
            }
        } else if token_upper == "DUMMY" {
            kind = DdKind::Dummy;
        }
    }

    // Determine DSN reference
    let dsn_ref = dsn.map(|d| parse_dsn_reference(&d));

    // If SYSOUT was detected, override kind
    if sysout_class.is_some() {
        // kind already set above
    }

    // Validate parentheses balance in raw operands
    let open_count = operands.chars().filter(|c| *c == '(').count();
    let close_count = operands.chars().filter(|c| *c == ')').count();
    if open_count != close_count {
        return Err(LintDiagnostic::new(
            DiagnosticCode::SyntaxError,
            line_number,
            (0, operands.len()),
            format!(
                "Unbalanced parentheses in DD statement (opened: {}, closed: {})",
                open_count, close_count
            ),
        )
        .with_ddname(ddname.to_string()));
    }

    Ok(DdStatement {
        ddname: ddname.to_string(),
        line_number,
        column_range: (0, operands.len()),
        step_name: step_name.to_string(),
        dsn: dsn_ref,
        disp,
        dcb,
        space,
        kind,
        concatenation_index: concat_index,
        raw_operands,
    })
}

/// Tokenize operand field into individual operands, respecting parentheses nesting.
fn tokenize_operands(operands: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_quote = false;

    for ch in operands.chars() {
        match ch {
            '\'' if !in_quote => {
                in_quote = true;
                current.push(ch);
            }
            '\'' if in_quote => {
                in_quote = false;
                current.push(ch);
            }
            '(' if !in_quote => {
                depth += 1;
                current.push(ch);
            }
            ')' if !in_quote => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 && !in_quote => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    tokens.push(trimmed);
                }
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        tokens.push(trimmed);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsn::DsnReference;
    use crate::operands::{DispAction, DispStatus, SpaceUnit};

    #[test]
    fn parse_basic_dd_statement() {
        // Validates: Requirement 1 AC 1
        let result = parse_single_dd("//SYSUT1  DD DSN=MY.DATA.SET,DISP=SHR", 1);
        let dd = result.unwrap();
        assert_eq!(dd.ddname, "SYSUT1");
        assert_eq!(
            dd.dsn,
            Some(DsnReference::Simple {
                dsn: "MY.DATA.SET".to_string()
            })
        );
    }

    #[test]
    fn parse_dd_with_quoted_dsn() {
        // Validates: Requirement 1 AC 2
        let result = parse_single_dd("//INPUT   DD DSN='MY.QUOTED.SET',DISP=OLD", 1);
        let dd = result.unwrap();
        assert_eq!(
            dd.dsn,
            Some(DsnReference::Simple {
                dsn: "MY.QUOTED.SET".to_string()
            })
        );
    }

    #[test]
    fn parse_dd_with_member_reference() {
        // Validates: Requirement 1 AC 3
        let result = parse_single_dd("//SYSIN   DD DSN=MY.PDS(MYPROG),DISP=SHR", 1);
        let dd = result.unwrap();
        assert_eq!(
            dd.dsn,
            Some(DsnReference::Member {
                pds_dsn: "MY.PDS".to_string(),
                member: "MYPROG".to_string(),
            })
        );
    }

    #[test]
    fn parse_dd_with_full_disp() {
        // Validates: Requirement 1 AC 4
        let result = parse_single_dd("//OUTPUT  DD DSN=MY.OUT,DISP=(NEW,CATLG,DELETE)", 1);
        let dd = result.unwrap();
        let disp = dd.disp.unwrap();
        assert_eq!(disp.status, DispStatus::New);
        assert_eq!(disp.normal_disp, Some(DispAction::Catlg));
        assert_eq!(disp.abnormal_disp, Some(DispAction::Delete));
    }

    #[test]
    fn parse_dd_with_dcb() {
        // Validates: Requirement 1 AC 5
        let result = parse_single_dd(
            "//OUTPUT  DD DSN=MY.OUT,DISP=(NEW,CATLG),DCB=(RECFM=FB,LRECL=80,BLKSIZE=27920)",
            1,
        );
        let dd = result.unwrap();
        let dcb = dd.dcb.unwrap();
        assert_eq!(dcb.recfm.as_deref(), Some("FB"));
        assert_eq!(dcb.lrecl, Some(80));
        assert_eq!(dcb.blksize, Some(27920));
    }

    #[test]
    fn parse_dd_with_space() {
        // Validates: Requirement 1 AC 6
        let result = parse_single_dd(
            "//OUTPUT  DD DSN=MY.OUT,DISP=(NEW,CATLG),SPACE=(TRK,(100,50,5))",
            1,
        );
        let dd = result.unwrap();
        let space = dd.space.unwrap();
        assert_eq!(space.unit, SpaceUnit::Trk);
        assert_eq!(space.primary, 100);
        assert_eq!(space.secondary, Some(50));
        assert_eq!(space.directory, Some(5));
    }

    #[test]
    fn parse_sysout_dd() {
        // Validates: Requirement 1 AC 8
        let result = parse_single_dd("//SYSPRINT DD SYSOUT=A", 1);
        let dd = result.unwrap();
        assert_eq!(dd.kind, DdKind::Sysout { class: 'A' });
        assert!(!dd.requires_resolution());
    }

    #[test]
    fn parse_inline_dd_star() {
        // Validates: Requirement 1 AC 9
        let result = parse_single_dd("//SYSIN   DD *", 1);
        let dd = result.unwrap();
        assert_eq!(dd.kind, DdKind::Inline);
        assert!(!dd.requires_resolution());
    }

    #[test]
    fn parse_dummy_dd() {
        // Validates: Requirement 1 AC 10
        let result = parse_single_dd("//NULLDD  DD DUMMY", 1);
        let dd = result.unwrap();
        assert_eq!(dd.kind, DdKind::Dummy);
        assert!(!dd.requires_resolution());
    }

    #[test]
    fn parse_detects_unbalanced_parentheses() {
        // Validates: Requirement 1 AC 11
        let result = parse_single_dd("//BAD     DD DSN=MY.DATA,DISP=(NEW,CATLG", 1);
        assert!(result.is_err());
        let diag = result.unwrap_err();
        assert_eq!(diag.code, DiagnosticCode::SyntaxError);
        assert!(diag.message.contains("Unbalanced parentheses"));
    }

    #[test]
    fn parse_jcl_multi_step_with_concatenation() {
        // Validates: Requirement 5 AC 1
        let jcl = "\
//MYJOB   JOB (ACCT),'PGMR'
//STEP1   EXEC PGM=IEFBR14
//SYSUT1  DD DSN=MY.FIRST.DATA,DISP=SHR
//        DD DSN=MY.SECOND.DATA,DISP=SHR
//SYSPRINT DD SYSOUT=A
";
        let result = parse_jcl_statements(jcl);
        assert_eq!(result.dd_statements.len(), 3);
        assert_eq!(result.dd_statements[0].ddname, "SYSUT1");
        assert_eq!(result.dd_statements[0].concatenation_index, 0);
        assert_eq!(result.dd_statements[1].ddname, "SYSUT1");
        assert_eq!(result.dd_statements[1].concatenation_index, 1);
        assert_eq!(result.dd_statements[2].ddname, "SYSPRINT");
    }

    #[test]
    fn parse_jcl_updates_step_name() {
        let jcl = "\
//MYJOB  JOB (ACCT),'PGMR'
//STEP1  EXEC PGM=IEFBR14
//DD1    DD DSN=A.B,DISP=SHR
//STEP2  EXEC PGM=IEBGENER
//DD2    DD DSN=C.D,DISP=OLD
";
        let result = parse_jcl_statements(jcl);
        assert_eq!(result.dd_statements[0].step_name, "STEP1");
        assert_eq!(result.dd_statements[1].step_name, "STEP2");
    }
}
