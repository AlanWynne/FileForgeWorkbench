//! Data model: CriteriaSet, Criterion, operators, connectors.
//!
//! Defines the core data structures for record selection criteria.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::CriteriaError;

/// Comparison operators available for criteria evaluation.
///
/// Addresses: Requirement 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CriteriaOperator {
    /// Equals — exact match (or wildcard match if value contains `*` or `?`).
    #[serde(rename = "EQ")]
    Eq,
    /// Not equals — inverse of Eq.
    #[serde(rename = "NE")]
    Ne,
    /// Greater than — ordered comparison.
    #[serde(rename = "GT")]
    Gt,
    /// Greater than or equal — ordered comparison.
    #[serde(rename = "GE")]
    Ge,
    /// Less than — ordered comparison.
    #[serde(rename = "LT")]
    Lt,
    /// Less than or equal — ordered comparison.
    #[serde(rename = "LE")]
    Le,
    /// Contains — substring match.
    #[serde(rename = "CONTAINS")]
    Contains,
    /// Starts with — prefix match.
    #[serde(rename = "STARTS_WITH")]
    StartsWith,
    /// Ends with — suffix match.
    #[serde(rename = "ENDS_WITH")]
    EndsWith,
    /// Matches regex — regular expression pattern match.
    #[serde(rename = "MATCHES_REGEX")]
    MatchesRegex,
}

impl fmt::Display for CriteriaOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eq => write!(f, "EQ"),
            Self::Ne => write!(f, "NE"),
            Self::Gt => write!(f, "GT"),
            Self::Ge => write!(f, "GE"),
            Self::Lt => write!(f, "LT"),
            Self::Le => write!(f, "LE"),
            Self::Contains => write!(f, "CONTAINS"),
            Self::StartsWith => write!(f, "STARTS_WITH"),
            Self::EndsWith => write!(f, "ENDS_WITH"),
            Self::MatchesRegex => write!(f, "MATCHES_REGEX"),
        }
    }
}

/// Logical connectors joining adjacent criterion rows.
///
/// Addresses: Requirement 5
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CriteriaConnector {
    /// Logical AND — both sides must be true. Binds tighter than OR.
    #[serde(rename = "AND")]
    And,
    /// Logical OR — either side must be true.
    #[serde(rename = "OR")]
    Or,
}

impl fmt::Display for CriteriaConnector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
        }
    }
}

/// The comparison mode determined by the field's data type.
///
/// Addresses: Requirement 3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonMode {
    /// Lexicographic string comparison (field types: str, bool, ebcdic).
    String,
    /// Numeric comparison after parsing to decimal (field types: int, float).
    Numeric,
    /// Packed-decimal comparison via COMP-3 decoding (field type: packed).
    PackedDecimal,
}

/// A single filter rule within a CriteriaSet.
///
/// Addresses: Requirement 1 AC 2
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Criterion {
    /// Whether this criterion row is active in evaluation.
    pub enabled: bool,
    /// The field name referencing a field in the active Record_Structure.
    pub field: String,
    /// The comparison operator to apply.
    pub operator: CriteriaOperator,
    /// The primary comparison value (as a string; parsed per field type).
    pub value: String,
    /// Secondary value for range operators (reserved for future use).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value2: Option<String>,
    /// Logical connector to the next row. None on the last row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connector: Option<CriteriaConnector>,
    /// Whether this row opens a parenthesised group.
    #[serde(default)]
    pub group_open: bool,
    /// Whether this row closes a parenthesised group.
    #[serde(default)]
    pub group_close: bool,
}

/// A complete filter expression: an ordered list of criteria rows with metadata.
///
/// Addresses: Requirement 1 AC 1, 6, 7
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CriteriaSet {
    /// The user-assigned name (if saved to catalog). None for unsaved expressions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional structure association for auto-suggestion matching.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structure_association: Option<String>,
    /// Optional record type scope. None means ALL TYPES.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_type_scope: Option<String>,
    /// Case sensitivity for string comparisons. Default: false.
    #[serde(default)]
    pub case_sensitive: bool,
    /// The ordered list of criterion rows forming the filter expression.
    pub criteria: Vec<Criterion>,
}

impl CriteriaSet {
    /// Create an empty CriteriaSet with default settings.
    pub fn empty() -> Self {
        Self {
            name: None,
            structure_association: None,
            record_type_scope: None,
            case_sensitive: false,
            criteria: Vec::new(),
        }
    }

    /// Create a CriteriaSet with a single criterion row.
    pub fn single(field: &str, operator: CriteriaOperator, value: &str) -> Self {
        Self {
            name: None,
            structure_association: None,
            record_type_scope: None,
            case_sensitive: false,
            criteria: vec![Criterion {
                enabled: true,
                field: field.to_string(),
                operator,
                value: value.to_string(),
                value2: None,
                connector: None,
                group_open: false,
                group_close: false,
            }],
        }
    }

    /// Get only the enabled criteria rows.
    pub fn enabled_criteria(&self) -> Vec<&Criterion> {
        self.criteria.iter().filter(|c| c.enabled).collect()
    }

    /// Format the criteria expression as a displayable string.
    ///
    /// Example: `FIELD1 EQ 'ABC' AND FIELD2 GT '100'`
    ///
    /// Addresses: Requirement 1 AC 7
    pub fn to_expression_string(&self) -> String {
        let enabled: Vec<&Criterion> = self.enabled_criteria();
        if enabled.is_empty() {
            return String::from("(no criteria)");
        }

        let mut parts = Vec::new();
        for criterion in &enabled {
            let mut part = String::new();
            if criterion.group_open {
                part.push('(');
            }
            part.push_str(&format!(
                "{} {} '{}'",
                criterion.field, criterion.operator, criterion.value
            ));
            if criterion.group_close {
                part.push(')');
            }
            if let Some(conn) = &criterion.connector {
                part.push_str(&format!(" {conn}"));
            }
            parts.push(part);
        }
        parts.join(" ")
    }

    /// Sanitise a name for use as a filename.
    ///
    /// Replaces non-alphanumeric characters (except hyphens and underscores)
    /// with underscores.
    ///
    /// Addresses: Requirement 11 AC 9
    pub fn sanitise_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    /// Deserialise a CriteriaSet from a JSON string.
    ///
    /// Addresses: Requirement 1 AC 6
    pub fn from_json(json: &str) -> Result<Self, CriteriaError> {
        serde_json::from_str(json).map_err(|e| CriteriaError::ParseFailed {
            path: String::from("<json>"),
            detail: e.to_string(),
        })
    }

    /// Serialise the CriteriaSet to a JSON string.
    ///
    /// Addresses: Requirement 1 AC 6
    pub fn to_json(&self) -> Result<String, CriteriaError> {
        serde_json::to_string_pretty(self).map_err(|e| CriteriaError::ParseFailed {
            path: String::from("<json>"),
            detail: e.to_string(),
        })
    }
}

/// The result of evaluating a CriteriaSet against a single record.
///
/// Addresses: Requirement 7 AC 1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriteriaResult {
    /// Whether the record satisfies the criteria expression.
    pub matches: bool,
    /// Per-row evaluation details (for UI highlighting in the panel).
    pub row_results: Vec<RowResult>,
}

/// Evaluation result for a single criterion row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowResult {
    /// Index of the criterion row in the CriteriaSet.
    pub row_index: usize,
    /// Whether this individual row matched.
    pub matched: bool,
    /// Whether this row was skipped (disabled).
    pub skipped: bool,
    /// Validation issue if any (e.g., unknown field, type mismatch).
    pub issue: Option<crate::validator::ValidationIssue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_criteria_set_has_no_rows() {
        let cs = CriteriaSet::empty();
        assert!(cs.criteria.is_empty());
        assert!(!cs.case_sensitive);
        assert!(cs.name.is_none());
    }

    #[test]
    fn single_criteria_set_has_one_enabled_row() {
        let cs = CriteriaSet::single("AMOUNT", CriteriaOperator::Gt, "100");
        assert_eq!(cs.criteria.len(), 1);
        assert!(cs.criteria[0].enabled);
        assert_eq!(cs.criteria[0].field, "AMOUNT");
        assert_eq!(cs.criteria[0].operator, CriteriaOperator::Gt);
        assert_eq!(cs.criteria[0].value, "100");
        assert!(cs.criteria[0].connector.is_none());
    }

    #[test]
    fn enabled_criteria_filters_disabled_rows() {
        let cs = CriteriaSet {
            name: None,
            structure_association: None,
            record_type_scope: None,
            case_sensitive: false,
            criteria: vec![
                Criterion {
                    enabled: true,
                    field: "A".to_string(),
                    operator: CriteriaOperator::Eq,
                    value: "1".to_string(),
                    value2: None,
                    connector: Some(CriteriaConnector::And),
                    group_open: false,
                    group_close: false,
                },
                Criterion {
                    enabled: false,
                    field: "B".to_string(),
                    operator: CriteriaOperator::Eq,
                    value: "2".to_string(),
                    value2: None,
                    connector: None,
                    group_open: false,
                    group_close: false,
                },
            ],
        };
        let enabled = cs.enabled_criteria();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].field, "A");
    }

    #[test]
    fn expression_string_formats_correctly() {
        let cs = CriteriaSet {
            name: None,
            structure_association: None,
            record_type_scope: None,
            case_sensitive: false,
            criteria: vec![
                Criterion {
                    enabled: true,
                    field: "NAME".to_string(),
                    operator: CriteriaOperator::Eq,
                    value: "ABC".to_string(),
                    value2: None,
                    connector: Some(CriteriaConnector::And),
                    group_open: false,
                    group_close: false,
                },
                Criterion {
                    enabled: true,
                    field: "AGE".to_string(),
                    operator: CriteriaOperator::Gt,
                    value: "30".to_string(),
                    value2: None,
                    connector: None,
                    group_open: false,
                    group_close: false,
                },
            ],
        };
        assert_eq!(cs.to_expression_string(), "NAME EQ 'ABC' AND AGE GT '30'");
    }

    #[test]
    fn expression_string_empty_returns_no_criteria() {
        let cs = CriteriaSet::empty();
        assert_eq!(cs.to_expression_string(), "(no criteria)");
    }

    #[test]
    fn sanitise_name_replaces_special_chars() {
        assert_eq!(CriteriaSet::sanitise_name("My Criteria!"), "My_Criteria_");
        assert_eq!(CriteriaSet::sanitise_name("test-set_01"), "test-set_01");
        assert_eq!(CriteriaSet::sanitise_name("a/b\\c.d"), "a_b_c_d");
    }

    #[test]
    fn json_round_trip_preserves_criteria_set() {
        let cs = CriteriaSet {
            name: Some("test_set".to_string()),
            structure_association: Some("MY_STRUCT".to_string()),
            record_type_scope: Some("DETAIL".to_string()),
            case_sensitive: true,
            criteria: vec![Criterion {
                enabled: true,
                field: "FIELD1".to_string(),
                operator: CriteriaOperator::Contains,
                value: "hello".to_string(),
                value2: None,
                connector: None,
                group_open: false,
                group_close: false,
            }],
        };
        let json = cs.to_json().unwrap();
        let restored = CriteriaSet::from_json(&json).unwrap();
        assert_eq!(cs, restored);
    }

    #[test]
    fn operator_display_formats_correctly() {
        assert_eq!(format!("{}", CriteriaOperator::Eq), "EQ");
        assert_eq!(format!("{}", CriteriaOperator::Ne), "NE");
        assert_eq!(format!("{}", CriteriaOperator::Contains), "CONTAINS");
        assert_eq!(
            format!("{}", CriteriaOperator::MatchesRegex),
            "MATCHES_REGEX"
        );
    }

    #[test]
    fn connector_display_formats_correctly() {
        assert_eq!(format!("{}", CriteriaConnector::And), "AND");
        assert_eq!(format!("{}", CriteriaConnector::Or), "OR");
    }
}
