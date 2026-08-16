//! Core evaluation logic: orchestrates comparison and logical combination.
//!
//! The `CriteriaEvaluator` applies a `CriteriaSet` to a record's field values,
//! returning whether the record matches the filter expression.

use std::collections::HashMap;

use crate::comparison::ComparisonEngine;
use crate::logical::{LogicalCombiner, LogicalRow};
use crate::model::{ComparisonMode, CriteriaResult, CriteriaSet, RowResult};
use crate::types::{FieldDataType, RecordFields};
use crate::validator::ValidationIssue;

/// The criteria evaluator applies a CriteriaSet to a record's field values,
/// returning whether the record matches the filter expression.
///
/// This is the single entry point for all criteria evaluation.
pub struct CriteriaEvaluator {
    comparison: ComparisonEngine,
}

impl CriteriaEvaluator {
    /// Create a new evaluator.
    pub fn new() -> Self {
        Self {
            comparison: ComparisonEngine::new(),
        }
    }

    /// Check if all criteria rows are disabled or the set is empty.
    /// When true, filtering is skipped entirely.
    ///
    /// Addresses: Requirement 1 AC 4
    pub fn is_passthrough(criteria: &CriteriaSet) -> bool {
        criteria.criteria.is_empty() || criteria.criteria.iter().all(|c| !c.enabled)
    }

    /// Evaluate a CriteriaSet against a record's extracted field values.
    ///
    /// Returns a `CriteriaResult` indicating match/no-match with per-row details.
    ///
    /// `field_values` is a map of field name → extracted string value.
    /// `field_types` provides the data type for each field (for comparison mode selection).
    ///
    /// Addresses: Requirement 1 AC 3, 4, 5; Requirement 7 AC 1
    pub fn evaluate(
        &self,
        criteria: &CriteriaSet,
        field_values: &HashMap<String, String>,
        field_types: &HashMap<String, FieldDataType>,
    ) -> CriteriaResult {
        // Passthrough: empty or all disabled
        if Self::is_passthrough(criteria) {
            let row_results: Vec<RowResult> = criteria
                .criteria
                .iter()
                .enumerate()
                .map(|(i, _)| RowResult {
                    row_index: i,
                    matched: false,
                    skipped: true,
                    issue: None,
                })
                .collect();
            return CriteriaResult {
                matches: true,
                row_results,
            };
        }

        let mut row_results = Vec::new();
        let mut logical_rows = Vec::new();

        for (i, criterion) in criteria.criteria.iter().enumerate() {
            if !criterion.enabled {
                row_results.push(RowResult {
                    row_index: i,
                    matched: false,
                    skipped: true,
                    issue: None,
                });
                continue;
            }

            // Get field value
            let field_value = field_values.get(&criterion.field);
            if field_value.is_none() {
                // Unknown field — treat as non-matching
                row_results.push(RowResult {
                    row_index: i,
                    matched: false,
                    skipped: false,
                    issue: Some(ValidationIssue::UnknownField {
                        field: criterion.field.clone(),
                    }),
                });
                logical_rows.push(LogicalRow {
                    result: false,
                    connector: criterion.connector,
                    group_open: criterion.group_open,
                    group_close: criterion.group_close,
                });
                continue;
            }

            let field_value = field_value.unwrap();

            // Determine comparison mode
            let mode = field_types
                .get(&criterion.field)
                .map(ComparisonEngine::determine_mode)
                .unwrap_or(ComparisonMode::String);

            // Perform comparison
            let result = self.comparison.compare(
                field_value,
                &criterion.value,
                criterion.operator,
                mode,
                criteria.case_sensitive,
            );

            let (matched, issue) = match result {
                Ok(m) => (m, None),
                Err(_) => (false, None), // Comparison errors → non-matching
            };

            row_results.push(RowResult {
                row_index: i,
                matched,
                skipped: false,
                issue,
            });
            logical_rows.push(LogicalRow {
                result: matched,
                connector: criterion.connector,
                group_open: criterion.group_open,
                group_close: criterion.group_close,
            });
        }

        // Combine logical results
        let final_match = LogicalCombiner::combine(&logical_rows);

        CriteriaResult {
            matches: final_match,
            row_results,
        }
    }

    /// Evaluate a CriteriaSet against all records, returning indices of matching records.
    ///
    /// Used for bulk filtering in grid display.
    ///
    /// Addresses: Requirement 7 AC 1, 2
    pub fn evaluate_all(
        &self,
        criteria: &CriteriaSet,
        records: &[RecordFields],
        field_types: &HashMap<String, FieldDataType>,
    ) -> Vec<usize> {
        if Self::is_passthrough(criteria) {
            return (0..records.len()).collect();
        }

        records
            .iter()
            .enumerate()
            .filter_map(|(i, record)| {
                let result = self.evaluate(criteria, &record.values, field_types);
                if result.matches {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for CriteriaEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CriteriaConnector, CriteriaOperator, Criterion};

    fn field_values(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn field_types(pairs: &[(&str, FieldDataType)]) -> HashMap<String, FieldDataType> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn passthrough_empty_criteria() {
        let cs = CriteriaSet::empty();
        assert!(CriteriaEvaluator::is_passthrough(&cs));

        let evaluator = CriteriaEvaluator::new();
        let result = evaluator.evaluate(&cs, &HashMap::new(), &HashMap::new());
        assert!(result.matches);
    }

    #[test]
    fn passthrough_all_disabled() {
        let cs = CriteriaSet {
            criteria: vec![Criterion {
                enabled: false,
                field: "A".to_string(),
                operator: CriteriaOperator::Eq,
                value: "1".to_string(),
                value2: None,
                connector: None,
                group_open: false,
                group_close: false,
            }],
            ..CriteriaSet::empty()
        };
        assert!(CriteriaEvaluator::is_passthrough(&cs));

        let evaluator = CriteriaEvaluator::new();
        let result = evaluator.evaluate(&cs, &HashMap::new(), &HashMap::new());
        assert!(result.matches);
    }

    #[test]
    fn single_criterion_matches() {
        let cs = CriteriaSet::single("NAME", CriteriaOperator::Eq, "Alice");
        let fv = field_values(&[("NAME", "Alice")]);
        let ft = field_types(&[("NAME", FieldDataType::Str)]);

        let evaluator = CriteriaEvaluator::new();
        let result = evaluator.evaluate(&cs, &fv, &ft);
        assert!(result.matches);
    }

    #[test]
    fn single_criterion_does_not_match() {
        let cs = CriteriaSet::single("NAME", CriteriaOperator::Eq, "Alice");
        let fv = field_values(&[("NAME", "Bob")]);
        let ft = field_types(&[("NAME", FieldDataType::Str)]);

        let evaluator = CriteriaEvaluator::new();
        let result = evaluator.evaluate(&cs, &fv, &ft);
        assert!(!result.matches);
    }

    #[test]
    fn case_insensitive_match() {
        let mut cs = CriteriaSet::single("NAME", CriteriaOperator::Eq, "alice");
        cs.case_sensitive = false;
        let fv = field_values(&[("NAME", "ALICE")]);
        let ft = field_types(&[("NAME", FieldDataType::Str)]);

        let evaluator = CriteriaEvaluator::new();
        let result = evaluator.evaluate(&cs, &fv, &ft);
        assert!(result.matches);
    }

    #[test]
    fn unknown_field_treated_as_non_matching() {
        let cs = CriteriaSet::single("MISSING", CriteriaOperator::Eq, "x");
        let fv = field_values(&[("NAME", "Alice")]);
        let ft = HashMap::new();

        let evaluator = CriteriaEvaluator::new();
        let result = evaluator.evaluate(&cs, &fv, &ft);
        assert!(!result.matches);
        assert!(result.row_results[0].issue.is_some());
    }

    #[test]
    fn multi_criteria_and_both_match() {
        let cs = CriteriaSet {
            criteria: vec![
                Criterion {
                    enabled: true,
                    field: "NAME".to_string(),
                    operator: CriteriaOperator::Eq,
                    value: "Alice".to_string(),
                    value2: None,
                    connector: Some(CriteriaConnector::And),
                    group_open: false,
                    group_close: false,
                },
                Criterion {
                    enabled: true,
                    field: "AGE".to_string(),
                    operator: CriteriaOperator::Gt,
                    value: "20".to_string(),
                    value2: None,
                    connector: None,
                    group_open: false,
                    group_close: false,
                },
            ],
            ..CriteriaSet::empty()
        };
        let fv = field_values(&[("NAME", "Alice"), ("AGE", "30")]);
        let ft = field_types(&[("NAME", FieldDataType::Str), ("AGE", FieldDataType::Int)]);

        let evaluator = CriteriaEvaluator::new();
        let result = evaluator.evaluate(&cs, &fv, &ft);
        assert!(result.matches);
    }

    #[test]
    fn multi_criteria_and_one_fails() {
        let cs = CriteriaSet {
            criteria: vec![
                Criterion {
                    enabled: true,
                    field: "NAME".to_string(),
                    operator: CriteriaOperator::Eq,
                    value: "Alice".to_string(),
                    value2: None,
                    connector: Some(CriteriaConnector::And),
                    group_open: false,
                    group_close: false,
                },
                Criterion {
                    enabled: true,
                    field: "AGE".to_string(),
                    operator: CriteriaOperator::Gt,
                    value: "50".to_string(),
                    value2: None,
                    connector: None,
                    group_open: false,
                    group_close: false,
                },
            ],
            ..CriteriaSet::empty()
        };
        let fv = field_values(&[("NAME", "Alice"), ("AGE", "30")]);
        let ft = field_types(&[("NAME", FieldDataType::Str), ("AGE", FieldDataType::Int)]);

        let evaluator = CriteriaEvaluator::new();
        let result = evaluator.evaluate(&cs, &fv, &ft);
        assert!(!result.matches);
    }

    #[test]
    fn disabled_row_skipped_in_evaluation() {
        let cs = CriteriaSet {
            criteria: vec![
                Criterion {
                    enabled: true,
                    field: "NAME".to_string(),
                    operator: CriteriaOperator::Eq,
                    value: "Alice".to_string(),
                    value2: None,
                    connector: Some(CriteriaConnector::And),
                    group_open: false,
                    group_close: false,
                },
                Criterion {
                    enabled: false,
                    field: "AGE".to_string(),
                    operator: CriteriaOperator::Gt,
                    value: "999".to_string(),
                    value2: None,
                    connector: None,
                    group_open: false,
                    group_close: false,
                },
            ],
            ..CriteriaSet::empty()
        };
        let fv = field_values(&[("NAME", "Alice"), ("AGE", "30")]);
        let ft = field_types(&[("NAME", FieldDataType::Str), ("AGE", FieldDataType::Int)]);

        let evaluator = CriteriaEvaluator::new();
        let result = evaluator.evaluate(&cs, &fv, &ft);
        // Only the first (enabled) criterion matters — it matches
        assert!(result.matches);
    }

    #[test]
    fn evaluate_all_returns_matching_indices() {
        let cs = CriteriaSet::single("STATUS", CriteriaOperator::Eq, "active");
        let records = vec![
            RecordFields {
                values: field_values(&[("STATUS", "active")]),
            },
            RecordFields {
                values: field_values(&[("STATUS", "inactive")]),
            },
            RecordFields {
                values: field_values(&[("STATUS", "active")]),
            },
        ];
        let ft = field_types(&[("STATUS", FieldDataType::Str)]);

        let evaluator = CriteriaEvaluator::new();
        let indices = evaluator.evaluate_all(&cs, &records, &ft);
        assert_eq!(indices, vec![0, 2]);
    }

    #[test]
    fn evaluate_all_passthrough_returns_all_indices() {
        let cs = CriteriaSet::empty();
        let records = vec![
            RecordFields {
                values: HashMap::new(),
            },
            RecordFields {
                values: HashMap::new(),
            },
        ];

        let evaluator = CriteriaEvaluator::new();
        let indices = evaluator.evaluate_all(&cs, &records, &HashMap::new());
        assert_eq!(indices, vec![0, 1]);
    }
}
