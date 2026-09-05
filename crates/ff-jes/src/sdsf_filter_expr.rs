//! FILTER command expression parser.
//!
//! Validates: Requirement 17.2, 17.12, 17.13, 17.14

use crate::model::Job;
use crate::sdsf_filter::SdsfColumn;

// === Comparison Operator ====================================================

/// Comparison operators supported in FILTER expressions.
///
/// Validates: Requirement 17.12
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
}

impl CmpOp {
    fn parse_prefix(s: &str) -> Option<(Self, usize)> {
        if s.starts_with(">=") {
            return Some((Self::Ge, 2));
        }
        if s.starts_with("<=") {
            return Some((Self::Le, 2));
        }
        if s.starts_with("!=") {
            return Some((Self::Ne, 2));
        }
        if s.starts_with('>') {
            return Some((Self::Gt, 1));
        }
        if s.starts_with('<') {
            return Some((Self::Lt, 1));
        }
        if s.starts_with('=') {
            return Some((Self::Eq, 1));
        }
        None
    }
}

// === Wildcard match =========================================================

fn wildcard_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return pattern.eq_ignore_ascii_case(value);
    }
    let val_up = value.to_uppercase();
    let pat_up = pattern.to_uppercase();
    let parts: Vec<&str> = pat_up.split('*').collect();
    let mut pos = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !val_up.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else {
            match val_up[pos..].find(part) {
                Some(idx) => pos += idx + part.len(),
                None => return false,
            }
        }
    }
    true
}

fn compare_str(field: &str, op: CmpOp, pattern: &str) -> bool {
    // For ordering operators, attempt numeric comparison first.
    if matches!(op, CmpOp::Gt | CmpOp::Lt | CmpOp::Ge | CmpOp::Le) {
        if let (Ok(fv), Ok(pv)) = (field.parse::<i64>(), pattern.parse::<i64>()) {
            return match op {
                CmpOp::Gt => fv > pv,
                CmpOp::Lt => fv < pv,
                CmpOp::Ge => fv >= pv,
                CmpOp::Le => fv <= pv,
                _ => unreachable!(),
            };
        }
    }
    match op {
        CmpOp::Eq => wildcard_match(pattern, field),
        CmpOp::Ne => !wildcard_match(pattern, field),
        CmpOp::Gt => field.to_uppercase() > pattern.to_uppercase(),
        CmpOp::Lt => field.to_uppercase() < pattern.to_uppercase(),
        CmpOp::Ge => field.to_uppercase() >= pattern.to_uppercase(),
        CmpOp::Le => field.to_uppercase() <= pattern.to_uppercase(),
    }
}

fn job_field_str(job: &Job, col: SdsfColumn) -> String {
    match col {
        SdsfColumn::JobName => job.name.clone(),
        SdsfColumn::JobId => job.id.to_string(),
        SdsfColumn::Owner => job.owner.clone(),
        SdsfColumn::Status => job.status.to_string(),
        SdsfColumn::Priority => job.priority.to_string(),
        SdsfColumn::ReturnCode => job.return_code.map(|r| r.to_string()).unwrap_or_default(),
        SdsfColumn::Start => job.start_time.map(|t| t.to_string()).unwrap_or_default(),
        SdsfColumn::End => job.end_time.map(|t| t.to_string()).unwrap_or_default(),
        SdsfColumn::Class | SdsfColumn::Queue | SdsfColumn::StepName | SdsfColumn::ProcStep => {
            String::new()
        }
    }
}

// === Filter Predicate =======================================================

/// A single field comparison predicate.
#[derive(Debug, Clone)]
pub struct FilterPredicate {
    pub column: SdsfColumn,
    pub op: CmpOp,
    pub value: String,
}

impl FilterPredicate {
    pub fn matches(&self, job: &Job) -> bool {
        compare_str(&job_field_str(job, self.column), self.op, &self.value)
    }
}

// === Filter Expression ======================================================

/// A parsed FILTER expression tree.
///
/// Validates: Requirement 17.2, 17.13
#[derive(Debug, Clone)]
pub enum FilterExpr {
    Predicate(FilterPredicate),
    And(Box<FilterExpr>, Box<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
}

impl FilterExpr {
    pub fn matches(&self, job: &Job) -> bool {
        match self {
            Self::Predicate(p) => p.matches(job),
            Self::And(a, b) => a.matches(job) && b.matches(job),
            Self::Or(a, b) => a.matches(job) || b.matches(job),
        }
    }

    /// Parses a FILTER expression. Returns Ok(None) for empty input (clears filter).
    pub fn parse(input: &str) -> Result<Option<Self>, String> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(None);
        }
        let tokens = tokenise(input)?;
        if tokens.is_empty() {
            return Ok(None);
        }
        let (expr, _) = parse_or(&tokens, 0)?;
        Ok(Some(expr))
    }
}

// === Tokeniser ==============================================================

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Word(String),
    Op(CmpOp),
    And,
    Or,
}

fn tokenise(input: &str) -> Result<Vec<Tok>, String> {
    let mut tokens = Vec::new();
    let mut i = 0;
    let bytes = input.as_bytes();
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let rest = &input[i..];
        if let Some((op, len)) = CmpOp::parse_prefix(rest) {
            tokens.push(Tok::Op(op));
            i += len;
        } else {
            let end = rest
                .find(|c: char| c.is_whitespace() || ">=<!".contains(c))
                .unwrap_or(rest.len());
            let word = &rest[..end];
            match word.to_uppercase().as_str() {
                "AND" => tokens.push(Tok::And),
                "OR" => tokens.push(Tok::Or),
                _ => tokens.push(Tok::Word(word.to_string())),
            }
            i += end;
        }
    }
    Ok(tokens)
}

fn parse_or(tokens: &[Tok], pos: usize) -> Result<(FilterExpr, usize), String> {
    let (mut left, mut pos) = parse_and(tokens, pos)?;
    while pos < tokens.len() && tokens[pos] == Tok::Or {
        let (right, np) = parse_and(tokens, pos + 1)?;
        left = FilterExpr::Or(Box::new(left), Box::new(right));
        pos = np;
    }
    Ok((left, pos))
}

fn parse_and(tokens: &[Tok], pos: usize) -> Result<(FilterExpr, usize), String> {
    let (mut left, mut pos) = parse_pred(tokens, pos)?;
    while pos < tokens.len() && tokens[pos] == Tok::And {
        let (right, np) = parse_pred(tokens, pos + 1)?;
        left = FilterExpr::And(Box::new(left), Box::new(right));
        pos = np;
    }
    Ok((left, pos))
}

fn parse_pred(tokens: &[Tok], pos: usize) -> Result<(FilterExpr, usize), String> {
    if pos + 3 > tokens.len() {
        return Err(format!("incomplete predicate at position {pos}"));
    }
    let col_str = match &tokens[pos] {
        Tok::Word(w) => w.clone(),
        other => return Err(format!("expected column name, got {other:?}")),
    };
    let col = SdsfColumn::parse(&col_str).ok_or_else(|| format!("unknown column: {col_str}"))?;
    let op = match tokens[pos + 1] {
        Tok::Op(o) => o,
        ref other => return Err(format!("expected operator, got {other:?}")),
    };
    let value = match &tokens[pos + 2] {
        Tok::Word(w) => w.clone(),
        other => return Err(format!("expected value, got {other:?}")),
    };
    Ok((
        FilterExpr::Predicate(FilterPredicate {
            column: col,
            op,
            value,
        }),
        pos + 3,
    ))
}

// === Active Filter State ====================================================

/// Holds the active FILTER expression.
///
/// Validates: Requirement 17.2
#[derive(Debug, Clone, Default)]
pub struct ActiveFilter {
    pub expr: Option<FilterExpr>,
    pub raw: String,
}

impl ActiveFilter {
    pub fn set(&mut self, input: &str) -> Result<(), String> {
        self.raw = input.to_string();
        self.expr = FilterExpr::parse(input)?;
        Ok(())
    }

    pub fn matches(&self, job: &Job) -> bool {
        match &self.expr {
            None => true,
            Some(expr) => expr.matches(job),
        }
    }

    pub fn clear(&mut self) {
        self.expr = None;
        self.raw.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffjcl::{FfjclDefinition, FfjclStep};
    use crate::model::{Job, JobId};

    fn make_job(id: u64, name: &str, owner: &str) -> Job {
        let def = FfjclDefinition {
            job_name: name.to_string(),
            owner: None,
            priority: None,
            class: None,
            steps: vec![FfjclStep {
                name: "S1".to_string(),
                program: "PGM".to_string(),
                args: vec![],
                dds: vec![],
                condition: None,
            }],
            source: String::new(),
        };
        let mut job = Job::new(JobId::new(id), def, owner);
        job.name = name.to_string();
        job
    }

    #[test]
    fn filter_eq_jobname() {
        // Validates: Requirement 17.2, 17.12
        let expr = FilterExpr::parse("JOBNAME=PAYROLL").unwrap().unwrap();
        assert!(expr.matches(&make_job(1, "PAYROLL", "u")));
        assert!(!expr.matches(&make_job(2, "BILLING", "u")));
    }

    #[test]
    fn filter_wildcard_prefix() {
        // Validates: Requirement 17.12
        let expr = FilterExpr::parse("JOBNAME=PAY*").unwrap().unwrap();
        assert!(expr.matches(&make_job(1, "PAYROLL", "u")));
        assert!(!expr.matches(&make_job(2, "BILLING", "u")));
    }

    #[test]
    fn filter_ne_operator() {
        // Validates: Requirement 17.12
        let expr = FilterExpr::parse("JOBNAME!=PAYROLL").unwrap().unwrap();
        assert!(!expr.matches(&make_job(1, "PAYROLL", "u")));
        assert!(expr.matches(&make_job(2, "BILLING", "u")));
    }

    #[test]
    fn filter_and_operator() {
        // Validates: Requirement 17.13
        let expr = FilterExpr::parse("JOBNAME=PAY* AND OWNER=ALICE")
            .unwrap()
            .unwrap();
        let mut j1 = make_job(1, "PAYROLL", "alice");
        j1.owner = "alice".to_string();
        let mut j2 = make_job(2, "PAYROLL", "bob");
        j2.owner = "bob".to_string();
        assert!(expr.matches(&j1));
        assert!(!expr.matches(&j2));
    }

    #[test]
    fn filter_or_operator() {
        // Validates: Requirement 17.13
        let expr = FilterExpr::parse("JOBNAME=PAY* OR JOBNAME=BILL*")
            .unwrap()
            .unwrap();
        assert!(expr.matches(&make_job(1, "PAYROLL", "u")));
        assert!(expr.matches(&make_job(2, "BILLING", "u")));
        assert!(!expr.matches(&make_job(3, "OTHER", "u")));
    }

    #[test]
    fn filter_empty_returns_none() {
        // Validates: Requirement 17.2
        assert!(FilterExpr::parse("").unwrap().is_none());
    }

    #[test]
    fn filter_unknown_column_errors() {
        assert!(FilterExpr::parse("BOGUS=X").is_err());
    }

    #[test]
    fn active_filter_set_and_clear() {
        // Validates: Requirement 17.2
        let mut af = ActiveFilter::default();
        af.set("JOBNAME=PAY*").unwrap();
        assert!(af.expr.is_some());
        af.clear();
        assert!(af.expr.is_none());
    }

    #[test]
    fn active_filter_no_filter_passes_all() {
        let af = ActiveFilter::default();
        assert!(af.matches(&make_job(1, "ANY", "u")));
    }

    #[test]
    fn filter_status_active() {
        // Validates: Requirement 17.1 (ST panel shows all statuses)
        let expr = FilterExpr::parse("STATUS=QUEUED").unwrap().unwrap();
        let j = make_job(1, "JOB1", "u"); // new jobs are Queued
        assert!(expr.matches(&j));
    }

    #[test]
    fn filter_ge_operator() {
        // Validates: Requirement 17.12
        let expr = FilterExpr::parse("PRTY>=5").unwrap().unwrap();
        let mut j_high = make_job(1, "J1", "u");
        j_high.priority = 10;
        let mut j_low = make_job(2, "J2", "u");
        j_low.priority = 3;
        assert!(expr.matches(&j_high));
        assert!(!expr.matches(&j_low));
    }
}
