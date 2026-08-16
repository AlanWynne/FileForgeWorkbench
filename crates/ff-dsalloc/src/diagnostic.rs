//! Lint diagnostic types and codes for the JCL resolver.
//!
//! Defines `LintDiagnostic`, `DiagnosticSeverity`, and `DiagnosticCode` —
//! the structured validation outputs produced by each pipeline stage.

use std::fmt;

use serde::Deserialize;

/// Diagnostic severity levels.
///
/// Ordered from least to most severe for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    /// Informational observation.
    Info,
    /// Potential problem that may or may not cause failure.
    Warning,
    /// Resolution failure — prevents successful execution.
    Error,
}

impl fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Unique diagnostic codes for each class of JCL problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    /// JCL001 — Syntax error in DD statement.
    SyntaxError,
    /// JCL002 — Unresolved DSN (not found in catalogs).
    UnresolvedDsn,
    /// JCL003 — Unresolved symbolic parameter.
    UnresolvedSymbolic,
    /// JCL004 — DISP conflict (NEW on existing, OLD on non-existent).
    DispConflict,
    /// JCL005 — Referback target not found.
    ReferbackNotFound,
    /// JCL006 — GDG base or generation not found.
    GdgNotFound,
    /// JCL007 — Concatenation error (max exceeded, attribute mismatch).
    ConcatenationError,
    /// JCL008 — Invalid DSN syntax.
    InvalidDsnSyntax,
    /// JCL009 — Temporary dataset not created in prior step.
    TemporaryNotFound,
    /// JCL010 — Duplicate ddname in step.
    DuplicateDdname,
    /// JCL011 — Missing well-known DD (SYSIN, SYSPRINT, etc.).
    MissingWellKnownDd,
    /// JCL012 — Invalid symbolic parameter name.
    InvalidSymbolicName,
    /// JCL013 — Catalog query failure.
    CatalogQueryFailed,
    /// JCL014 — GDG roll-off notification.
    GdgRollOff,
    /// JCL015 — Multiple forward GDG generations.
    MultipleForwardGdg,
    /// JCL016 — Member not found in PDS.
    MemberNotFound,
    /// JCL017 — Ambiguous DSN (found in multiple catalogs).
    AmbiguousDsn,
    /// JCL018 — Referback chain too deep.
    ReferbackChainTooDeep,
}

impl DiagnosticCode {
    /// Returns the string representation (e.g., "JCL001").
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SyntaxError => "JCL001",
            Self::UnresolvedDsn => "JCL002",
            Self::UnresolvedSymbolic => "JCL003",
            Self::DispConflict => "JCL004",
            Self::ReferbackNotFound => "JCL005",
            Self::GdgNotFound => "JCL006",
            Self::ConcatenationError => "JCL007",
            Self::InvalidDsnSyntax => "JCL008",
            Self::TemporaryNotFound => "JCL009",
            Self::DuplicateDdname => "JCL010",
            Self::MissingWellKnownDd => "JCL011",
            Self::InvalidSymbolicName => "JCL012",
            Self::CatalogQueryFailed => "JCL013",
            Self::GdgRollOff => "JCL014",
            Self::MultipleForwardGdg => "JCL015",
            Self::MemberNotFound => "JCL016",
            Self::AmbiguousDsn => "JCL017",
            Self::ReferbackChainTooDeep => "JCL018",
        }
    }

    /// Returns the default severity for this diagnostic code.
    pub fn default_severity(&self) -> DiagnosticSeverity {
        match self {
            Self::GdgRollOff => DiagnosticSeverity::Info,
            Self::MultipleForwardGdg
            | Self::MemberNotFound
            | Self::AmbiguousDsn
            | Self::MissingWellKnownDd
            | Self::ConcatenationError => DiagnosticSeverity::Warning,
            _ => DiagnosticSeverity::Error,
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A validation diagnostic produced by the resolver.
///
/// All diagnostics include location information (line, column range),
/// a unique code, and a human-readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintDiagnostic {
    /// Severity level.
    pub severity: DiagnosticSeverity,
    /// Line number in JCL source (1-based).
    pub line: usize,
    /// Column range (start, end) for highlighting.
    pub column_range: (usize, usize),
    /// Unique diagnostic code.
    pub code: DiagnosticCode,
    /// Human-readable message.
    pub message: String,
    /// Optional ddname context.
    pub ddname: Option<String>,
}

impl LintDiagnostic {
    /// Create a new diagnostic with the given parameters.
    pub fn new(
        code: DiagnosticCode,
        line: usize,
        column_range: (usize, usize),
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: code.default_severity(),
            line,
            column_range,
            code,
            message: message.into(),
            ddname: None,
        }
    }

    /// Set the severity (overriding the default for the code).
    pub fn with_severity(mut self, severity: DiagnosticSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the ddname context.
    pub fn with_ddname(mut self, ddname: impl Into<String>) -> Self {
        self.ddname = Some(ddname.into());
        self
    }
}

impl fmt::Display for LintDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} (line {}): {}",
            self.severity, self.code, self.line, self.message
        )
    }
}

impl PartialOrd for LintDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LintDiagnostic {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.line
            .cmp(&other.line)
            .then(self.severity.cmp(&other.severity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_codes_have_unique_string_representations() {
        // Validates: Requirement 10 AC 9
        use std::collections::HashSet;
        let codes = [
            DiagnosticCode::SyntaxError,
            DiagnosticCode::UnresolvedDsn,
            DiagnosticCode::UnresolvedSymbolic,
            DiagnosticCode::DispConflict,
            DiagnosticCode::ReferbackNotFound,
            DiagnosticCode::GdgNotFound,
            DiagnosticCode::ConcatenationError,
            DiagnosticCode::InvalidDsnSyntax,
            DiagnosticCode::TemporaryNotFound,
            DiagnosticCode::DuplicateDdname,
            DiagnosticCode::MissingWellKnownDd,
            DiagnosticCode::InvalidSymbolicName,
            DiagnosticCode::CatalogQueryFailed,
            DiagnosticCode::GdgRollOff,
            DiagnosticCode::MultipleForwardGdg,
            DiagnosticCode::MemberNotFound,
            DiagnosticCode::AmbiguousDsn,
            DiagnosticCode::ReferbackChainTooDeep,
        ];
        let strings: HashSet<&str> = codes.iter().map(|c| c.as_str()).collect();
        assert_eq!(
            strings.len(),
            codes.len(),
            "All diagnostic codes must be unique"
        );
    }

    #[test]
    fn lint_diagnostic_display_includes_all_fields() {
        // Validates: Requirement 10 AC 1, AC 9
        let diag = LintDiagnostic::new(
            DiagnosticCode::UnresolvedDsn,
            10,
            (3, 20),
            "Dataset not found: MY.DATA.SET",
        );
        let display = diag.to_string();
        assert!(display.contains("ERROR"));
        assert!(display.contains("JCL002"));
        assert!(display.contains("line 10"));
        assert!(display.contains("Dataset not found: MY.DATA.SET"));
    }

    #[test]
    fn lint_diagnostics_sort_by_line_then_severity() {
        // Validates: Requirement 10 AC 9
        let d1 = LintDiagnostic::new(DiagnosticCode::UnresolvedDsn, 5, (0, 10), "a");
        let d2 = LintDiagnostic::new(DiagnosticCode::GdgRollOff, 5, (0, 10), "b");
        let d3 = LintDiagnostic::new(DiagnosticCode::UnresolvedDsn, 10, (0, 10), "c");

        let mut diags = vec![d3.clone(), d1.clone(), d2.clone()];
        diags.sort();
        assert_eq!(diags[0].line, 5);
        assert_eq!(diags[0].severity, DiagnosticSeverity::Info); // GdgRollOff is Info
        assert_eq!(diags[1].line, 5);
        assert_eq!(diags[1].severity, DiagnosticSeverity::Error);
        assert_eq!(diags[2].line, 10);
    }
}
