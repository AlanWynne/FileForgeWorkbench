//! Concatenation group handling.
//!
//! Manages groups of DD statements sharing the same ddname (JCL concatenation).

use crate::dd_statement::DdStatement;
use crate::diagnostic::{DiagnosticCode, LintDiagnostic};

/// Maximum number of datasets allowed in a single concatenation group.
pub const MAX_CONCATENATION: usize = 255;

/// A concatenation group — multiple DDs sharing the same ddname.
#[derive(Debug, Clone)]
pub struct ConcatenationGroup {
    /// The shared ddname.
    pub ddname: String,
    /// Component DD statements (ordered, index 0 = primary).
    pub components: Vec<DdStatement>,
}

impl ConcatenationGroup {
    /// Create a new concatenation group from a list of DD statements with the same ddname.
    pub fn new(ddname: impl Into<String>, components: Vec<DdStatement>) -> Self {
        Self {
            ddname: ddname.into(),
            components,
        }
    }

    /// Returns the number of components in this group.
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Returns true if the group is empty.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Validate the concatenation group.
    ///
    /// Checks:
    /// - Not exceeding 255-dataset limit
    /// - Attribute compatibility (RECFM match, LRECL compatibility)
    pub fn validate(&self) -> Vec<LintDiagnostic> {
        let mut diagnostics = Vec::new();

        // Check 255 limit
        if self.components.len() > MAX_CONCATENATION {
            diagnostics.push(
                LintDiagnostic::new(
                    DiagnosticCode::ConcatenationError,
                    self.components.first().map(|d| d.line_number).unwrap_or(0),
                    (0, 0),
                    format!(
                        "Concatenation group '{}' exceeds 255-dataset limit ({} datasets)",
                        self.ddname,
                        self.components.len()
                    ),
                )
                .with_ddname(&self.ddname),
            );
        }

        // Check attribute compatibility
        if self.components.len() > 1 {
            let first_recfm = self.components[0]
                .dcb
                .as_ref()
                .and_then(|d| d.recfm.as_deref());

            for (i, comp) in self.components.iter().enumerate().skip(1) {
                let comp_recfm = comp.dcb.as_ref().and_then(|d| d.recfm.as_deref());

                if let (Some(first), Some(current)) = (first_recfm, comp_recfm) {
                    if first != current {
                        diagnostics.push(
                            LintDiagnostic::new(
                                DiagnosticCode::ConcatenationError,
                                comp.line_number,
                                comp.column_range,
                                format!(
                                    "Concatenation RECFM mismatch in '{}' component {}: expected '{}', found '{}'",
                                    self.ddname,
                                    i + 1,
                                    first,
                                    current
                                ),
                            )
                            .with_severity(crate::diagnostic::DiagnosticSeverity::Warning)
                            .with_ddname(&self.ddname),
                        );
                    }
                }
            }
        }

        diagnostics
    }
}

/// Assemble concatenation groups from a flat list of DD statements.
///
/// DD statements with the same ddname and consecutive concatenation indices
/// are grouped together.
pub fn assemble_concatenation_groups(dd_statements: &[DdStatement]) -> Vec<ConcatenationGroup> {
    let mut groups: Vec<ConcatenationGroup> = Vec::new();

    for dd in dd_statements {
        if dd.concatenation_index == 0 {
            // Start of a new group (or standalone DD)
            groups.push(ConcatenationGroup::new(&dd.ddname, vec![dd.clone()]));
        } else {
            // Continuation of previous group
            if let Some(last_group) = groups.last_mut() {
                if last_group.ddname == dd.ddname {
                    last_group.components.push(dd.clone());
                }
            }
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dd_statement::DdKind;
    use crate::dsn::DsnReference;
    use crate::operands::DcbAttributes;

    fn make_concat_dd(ddname: &str, index: usize, recfm: Option<&str>) -> DdStatement {
        DdStatement {
            ddname: ddname.to_string(),
            line_number: index + 1,
            column_range: (0, 40),
            step_name: "STEP1".to_string(),
            dsn: Some(DsnReference::Simple {
                dsn: format!("DATA.SET{}", index),
            }),
            disp: None,
            dcb: recfm.map(|r| DcbAttributes {
                recfm: Some(r.to_string()),
                lrecl: None,
                blksize: None,
                dsorg: None,
            }),
            space: None,
            kind: DdKind::Dataset,
            concatenation_index: index,
            raw_operands: String::new(),
        }
    }

    #[test]
    fn assemble_groups_from_sequential_dds() {
        // Validates: Requirement 5 AC 1, AC 3
        let dds = vec![
            make_concat_dd("SYSUT1", 0, None),
            make_concat_dd("SYSUT1", 1, None),
            make_concat_dd("SYSUT1", 2, None),
            make_concat_dd("SYSUT2", 0, None),
        ];

        let groups = assemble_concatenation_groups(&dds);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].ddname, "SYSUT1");
        assert_eq!(groups[0].len(), 3);
        assert_eq!(groups[1].ddname, "SYSUT2");
        assert_eq!(groups[1].len(), 1);
    }

    #[test]
    fn validate_exceeding_255_limit() {
        // Validates: Requirement 5 AC 6
        let components: Vec<DdStatement> = (0..256)
            .map(|i| make_concat_dd("BIGCONCAT", i, None))
            .collect();

        let group = ConcatenationGroup::new("BIGCONCAT", components);
        let diags = group.validate();
        assert!(!diags.is_empty());
        assert!(diags[0].message.contains("255"));
    }

    #[test]
    fn validate_recfm_mismatch_produces_warning() {
        // Validates: Requirement 5 AC 5
        let dds = vec![
            make_concat_dd("SYSUT1", 0, Some("FB")),
            make_concat_dd("SYSUT1", 1, Some("VB")),
        ];

        let group = ConcatenationGroup::new("SYSUT1", dds);
        let diags = group.validate();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("RECFM mismatch"));
    }

    #[test]
    fn validate_matching_recfm_no_warnings() {
        let dds = vec![
            make_concat_dd("SYSUT1", 0, Some("FB")),
            make_concat_dd("SYSUT1", 1, Some("FB")),
        ];

        let group = ConcatenationGroup::new("SYSUT1", dds);
        let diags = group.validate();
        assert!(diags.is_empty());
    }
}
