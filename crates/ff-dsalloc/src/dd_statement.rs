//! DD statement model.
//!
//! The `DdStatement` struct represents a parsed JCL DD statement with all
//! extracted operands ready for resolution.

use crate::dsn::DsnReference;
use crate::operands::{DcbAttributes, DispParameter, SpaceAllocation};

/// Classification of a DD statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DdKind {
    /// Normal dataset reference (requires resolution).
    Dataset,
    /// SYSOUT=class (output-directed, no resolution).
    Sysout { class: char },
    /// DD * or DD DATA (inline data, no resolution).
    Inline,
    /// DUMMY (null dataset, no resolution).
    Dummy,
}

/// A parsed JCL DD statement with all extracted operands.
#[derive(Debug, Clone, PartialEq)]
pub struct DdStatement {
    /// The ddname (columns 3–10, trimmed).
    pub ddname: String,
    /// Line number in the JCL source (1-based).
    pub line_number: usize,
    /// Column range in source (start, end) for diagnostic highlighting.
    pub column_range: (usize, usize),
    /// The step this DD belongs to.
    pub step_name: String,
    /// DSN reference (if present — not present for SYSOUT, DUMMY, DD *).
    pub dsn: Option<DsnReference>,
    /// DISP operand (parsed sub-parameters).
    pub disp: Option<DispParameter>,
    /// DCB operand (dataset attributes).
    pub dcb: Option<DcbAttributes>,
    /// SPACE operand (allocation size).
    pub space: Option<SpaceAllocation>,
    /// DD classification.
    pub kind: DdKind,
    /// Concatenation index (0 = primary, 1+ = concatenated).
    pub concatenation_index: usize,
    /// Raw operand text (before substitution) for display.
    pub raw_operands: String,
}

impl DdStatement {
    /// Returns true if this DD requires DSN resolution.
    pub fn requires_resolution(&self) -> bool {
        matches!(self.kind, DdKind::Dataset)
    }

    /// Returns the effective DISP, applying defaults if not specified.
    ///
    /// Default DISP when not specified: (NEW, DELETE).
    pub fn effective_disp(&self) -> DispParameter {
        self.disp
            .clone()
            .unwrap_or_else(DispParameter::default_disp)
    }

    /// Returns the effective DCB attributes, using hardcoded defaults if not specified.
    pub fn effective_dcb(&self) -> DcbAttributes {
        self.dcb
            .clone()
            .unwrap_or_else(DcbAttributes::hardcoded_defaults)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operands::{DispAction, DispStatus};

    #[test]
    fn dd_statement_requires_resolution_for_dataset_kind() {
        // Validates: Requirement 1 AC 1
        let dd = DdStatement {
            ddname: "SYSUT1".to_string(),
            line_number: 5,
            column_range: (0, 40),
            step_name: "STEP1".to_string(),
            dsn: Some(DsnReference::Simple {
                dsn: "MY.DATA".to_string(),
            }),
            disp: Some(DispParameter {
                status: DispStatus::Old,
                normal_disp: None,
                abnormal_disp: None,
            }),
            dcb: None,
            space: None,
            kind: DdKind::Dataset,
            concatenation_index: 0,
            raw_operands: "DSN=MY.DATA,DISP=OLD".to_string(),
        };
        assert!(dd.requires_resolution());
    }

    #[test]
    fn dd_statement_does_not_require_resolution_for_sysout() {
        // Validates: Requirement 1 AC 8
        let dd = DdStatement {
            ddname: "SYSPRINT".to_string(),
            line_number: 6,
            column_range: (0, 30),
            step_name: "STEP1".to_string(),
            dsn: None,
            disp: None,
            dcb: None,
            space: None,
            kind: DdKind::Sysout { class: 'A' },
            concatenation_index: 0,
            raw_operands: "SYSOUT=A".to_string(),
        };
        assert!(!dd.requires_resolution());
    }

    #[test]
    fn effective_disp_returns_default_when_none() {
        // Validates: Requirement 4 AC 7
        let dd = DdStatement {
            ddname: "DD1".to_string(),
            line_number: 1,
            column_range: (0, 10),
            step_name: "STEP1".to_string(),
            dsn: None,
            disp: None,
            dcb: None,
            space: None,
            kind: DdKind::Dataset,
            concatenation_index: 0,
            raw_operands: String::new(),
        };
        let eff = dd.effective_disp();
        assert_eq!(eff.status, DispStatus::New);
        assert_eq!(eff.normal_disp, Some(DispAction::Delete));
    }
}
