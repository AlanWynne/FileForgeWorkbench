//! Expression validation for criteria sets.
//!
//! Validates criteria expressions for correctness before evaluation:
//! unknown fields, unmatched groups, invalid regex patterns, type mismatches,
//! nesting depth limits, and row count limits.

use crate::model::{CriteriaOperator, CriteriaSet};
use crate::types::FieldDataType;

use std::collections::HashMap;

/// A validation issue detected in a criteria expression.
///
/// Addresses: Requirement 5 AC 4, Requirement 2 AC 9, 12
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValidationIssue {
    /// A referenced field name does not exist in the active Record_Structure.
    UnknownField {
        /// The field name that was not found.
        field: String,
    },
    /// Group open/close flags are unmatched.
    UnmatchedGroup {
        /// Row index where the mismatch was detected.
        row_index: usize,
        /// Description of the mismatch.
        detail: String,
    },
    /// The regex pattern in a MATCHES_REGEX criterion is invalid.
    InvalidRegex {
        /// Row index containing the invalid regex.
        row_index: usize,
        /// The invalid pattern string.
        pattern: String,
        /// Description of the regex error.
        error: String,
    },
    /// The criterion value cannot be parsed as the expected numeric type.
    TypeMismatch {
        /// Row index with the type mismatch.
        row_index: usize,
        /// The field name.
        field: String,
        /// The expected type.
        expected: String,
        /// The value that failed to parse.
        value: String,
    },
    /// Maximum nesting depth exceeded (>8 levels).
    NestingDepthExceeded {
        /// Row index where depth was exceeded.
        row_index: usize,
        /// The depth that was reached.
        depth: usize,
    },
    /// Maximum criteria rows exceeded.
    MaxRowsExceeded {
        /// Actual row count.
        count: usize,
        /// Maximum allowed.
        max: usize,
    },
}

/// Validates a CriteriaSet for correctness before evaluation.
///
/// Addresses: Requirement 5 AC 4, Requirement 10 AC 14
pub struct CriteriaValidator;

impl CriteriaValidator {
    /// Validate a CriteriaSet against the current field definitions.
    ///
    /// Returns a list of validation issues (empty = valid).
    pub fn validate(
        criteria: &CriteriaSet,
        available_fields: &[String],
        field_types: &HashMap<String, FieldDataType>,
        max_rows: usize,
    ) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check max rows
        if criteria.criteria.len() > max_rows {
            issues.push(ValidationIssue::MaxRowsExceeded {
                count: criteria.criteria.len(),
                max: max_rows,
            });
        }

        // Check unknown fields
        for criterion in &criteria.criteria {
            if !criterion.enabled {
                continue;
            }
            if !available_fields.contains(&criterion.field) {
                issues.push(ValidationIssue::UnknownField {
                    field: criterion.field.clone(),
                });
            }
        }

        // Check groups
        issues.extend(Self::validate_groups(criteria));

        // Check regex patterns
        issues.extend(Self::validate_regex_patterns(criteria));

        // Check type mismatches
        issues.extend(Self::validate_types(criteria, field_types));

        issues
    }

    /// Validate group structure only (matched open/close flags).
    ///
    /// Addresses: Requirement 5 AC 4
    pub fn validate_groups(criteria: &CriteriaSet) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let mut depth: i32 = 0;
        let mut max_depth: usize = 0;

        for (i, criterion) in criteria.criteria.iter().enumerate() {
            if criterion.group_open {
                depth += 1;
                if depth as usize > max_depth {
                    max_depth = depth as usize;
                }
                if max_depth > 8 {
                    issues.push(ValidationIssue::NestingDepthExceeded {
                        row_index: i,
                        depth: max_depth,
                    });
                }
            }
            if criterion.group_close {
                depth -= 1;
                if depth < 0 {
                    issues.push(ValidationIssue::UnmatchedGroup {
                        row_index: i,
                        detail: String::from("group_close without matching group_open"),
                    });
                }
            }
        }

        if depth > 0 {
            issues.push(ValidationIssue::UnmatchedGroup {
                row_index: criteria.criteria.len().saturating_sub(1),
                detail: format!("{depth} unclosed group(s) at end of expression"),
            });
        }

        issues
    }

    /// Validate regex patterns in MATCHES_REGEX criteria.
    ///
    /// Addresses: Requirement 2 AC 9
    pub fn validate_regex_patterns(criteria: &CriteriaSet) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (i, criterion) in criteria.criteria.iter().enumerate() {
            if !criterion.enabled {
                continue;
            }
            if criterion.operator == CriteriaOperator::MatchesRegex {
                if let Err(e) = regex::Regex::new(&criterion.value) {
                    issues.push(ValidationIssue::InvalidRegex {
                        row_index: i,
                        pattern: criterion.value.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        issues
    }

    /// Validate type compatibility of criterion values with field types.
    fn validate_types(
        criteria: &CriteriaSet,
        field_types: &HashMap<String, FieldDataType>,
    ) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (i, criterion) in criteria.criteria.iter().enumerate() {
            if !criterion.enabled {
                continue;
            }
            if let Some(field_type) = field_types.get(&criterion.field) {
                let needs_numeric = matches!(
                    field_type,
                    FieldDataType::Int | FieldDataType::Float | FieldDataType::Packed
                );
                let is_ordering_op = matches!(
                    criterion.operator,
                    CriteriaOperator::Eq
                        | CriteriaOperator::Ne
                        | CriteriaOperator::Gt
                        | CriteriaOperator::Ge
                        | CriteriaOperator::Lt
                        | CriteriaOperator::Le
                );

                if needs_numeric && is_ordering_op && criterion.value.trim().parse::<f64>().is_err()
                {
                    issues.push(ValidationIssue::TypeMismatch {
                        row_index: i,
                        field: criterion.field.clone(),
                        expected: String::from("numeric"),
                        value: criterion.value.clone(),
                    });
                }
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CriteriaConnector, Criterion};

    fn make_criterion(field: &str, operator: CriteriaOperator, value: &str) -> Criterion {
        Criterion {
            enabled: true,
            field: field.to_string(),
            operator,
            value: value.to_string(),
            value2: None,
            connector: None,
            group_open: false,
            group_close: false,
        }
    }

    #[test]
    fn validate_empty_criteria_returns_no_issues() {
        let cs = CriteriaSet::empty();
        let issues = CriteriaValidator::validate(&cs, &[], &HashMap::new(), 50);
        assert!(issues.is_empty());
    }

    #[test]
    fn validate_unknown_field_reported() {
        let cs = CriteriaSet::single("UNKNOWN", CriteriaOperator::Eq, "x");
        let available = vec!["NAME".to_string(), "AGE".to_string()];
        let issues = CriteriaValidator::validate(&cs, &available, &HashMap::new(), 50);
        assert_eq!(issues.len(), 1);
        assert!(
            matches!(&issues[0], ValidationIssue::UnknownField { field } if field == "UNKNOWN")
        );
    }

    #[test]
    fn validate_disabled_row_unknown_field_not_reported() {
        let mut cs = CriteriaSet::single("UNKNOWN", CriteriaOperator::Eq, "x");
        cs.criteria[0].enabled = false;
        let available = vec!["NAME".to_string()];
        let issues = CriteriaValidator::validate(&cs, &available, &HashMap::new(), 50);
        assert!(issues.is_empty());
    }

    #[test]
    fn validate_groups_matched_no_issues() {
        let cs = CriteriaSet {
            criteria: vec![
                Criterion {
                    group_open: true,
                    group_close: false,
                    ..make_criterion("A", CriteriaOperator::Eq, "1")
                },
                Criterion {
                    group_open: false,
                    group_close: true,
                    connector: None,
                    ..make_criterion("B", CriteriaOperator::Eq, "2")
                },
            ],
            ..CriteriaSet::empty()
        };
        let issues = CriteriaValidator::validate_groups(&cs);
        assert!(issues.is_empty());
    }

    #[test]
    fn validate_groups_unclosed_group_reported() {
        let cs = CriteriaSet {
            criteria: vec![Criterion {
                group_open: true,
                group_close: false,
                ..make_criterion("A", CriteriaOperator::Eq, "1")
            }],
            ..CriteriaSet::empty()
        };
        let issues = CriteriaValidator::validate_groups(&cs);
        assert_eq!(issues.len(), 1);
        assert!(matches!(&issues[0], ValidationIssue::UnmatchedGroup { .. }));
    }

    #[test]
    fn validate_groups_extra_close_reported() {
        let cs = CriteriaSet {
            criteria: vec![Criterion {
                group_open: false,
                group_close: true,
                ..make_criterion("A", CriteriaOperator::Eq, "1")
            }],
            ..CriteriaSet::empty()
        };
        let issues = CriteriaValidator::validate_groups(&cs);
        assert_eq!(issues.len(), 1);
        assert!(matches!(
            &issues[0],
            ValidationIssue::UnmatchedGroup { detail, .. } if detail.contains("without matching")
        ));
    }

    #[test]
    fn validate_groups_nesting_depth_exceeded() {
        let mut criteria = Vec::new();
        for _ in 0..9 {
            criteria.push(Criterion {
                group_open: true,
                group_close: false,
                connector: Some(CriteriaConnector::And),
                ..make_criterion("A", CriteriaOperator::Eq, "1")
            });
        }
        criteria.push(Criterion {
            group_open: false,
            group_close: false,
            connector: None,
            ..make_criterion("A", CriteriaOperator::Eq, "1")
        });
        // Close all groups
        for _ in 0..9 {
            criteria.push(Criterion {
                group_open: false,
                group_close: true,
                connector: None,
                ..make_criterion("A", CriteriaOperator::Eq, "1")
            });
        }
        let cs = CriteriaSet {
            criteria,
            ..CriteriaSet::empty()
        };
        let issues = CriteriaValidator::validate_groups(&cs);
        assert!(issues
            .iter()
            .any(|i| matches!(i, ValidationIssue::NestingDepthExceeded { .. })));
    }

    #[test]
    fn validate_invalid_regex_reported() {
        let cs = CriteriaSet::single("NAME", CriteriaOperator::MatchesRegex, "[invalid");
        let issues = CriteriaValidator::validate_regex_patterns(&cs);
        assert_eq!(issues.len(), 1);
        assert!(matches!(&issues[0], ValidationIssue::InvalidRegex { .. }));
    }

    #[test]
    fn validate_valid_regex_no_issues() {
        let cs = CriteriaSet::single("NAME", CriteriaOperator::MatchesRegex, r"\d+");
        let issues = CriteriaValidator::validate_regex_patterns(&cs);
        assert!(issues.is_empty());
    }

    #[test]
    fn validate_type_mismatch_numeric_field_non_numeric_value() {
        let cs = CriteriaSet::single("AGE", CriteriaOperator::Gt, "abc");
        let mut field_types = HashMap::new();
        field_types.insert("AGE".to_string(), FieldDataType::Int);
        let available = vec!["AGE".to_string()];
        let issues = CriteriaValidator::validate(&cs, &available, &field_types, 50);
        assert!(issues
            .iter()
            .any(|i| matches!(i, ValidationIssue::TypeMismatch { .. })));
    }

    #[test]
    fn validate_max_rows_exceeded() {
        let criteria: Vec<Criterion> = (0..51)
            .map(|i| make_criterion(&format!("F{i}"), CriteriaOperator::Eq, "x"))
            .collect();
        let cs = CriteriaSet {
            criteria,
            ..CriteriaSet::empty()
        };
        let issues = CriteriaValidator::validate(&cs, &[], &HashMap::new(), 50);
        assert!(issues
            .iter()
            .any(|i| matches!(i, ValidationIssue::MaxRowsExceeded { .. })));
    }
}
