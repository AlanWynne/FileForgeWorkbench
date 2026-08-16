//! Property-based tests for ff-select using proptest.
//!
//! Each property test runs a minimum of 256 cases and validates
//! universal invariants that must hold for all valid inputs.

use proptest::prelude::*;
use std::collections::HashMap;

use ff_select::comparison::ComparisonEngine;
use ff_select::filter_state::FilterState;
use ff_select::logical::{LogicalCombiner, LogicalRow};
use ff_select::model::{
    ComparisonMode, CriteriaConnector, CriteriaOperator, CriteriaSet, Criterion,
};
use ff_select::scope::CriteriaScope;
use ff_select::types::FieldDataType;
use ff_select::wildcard::WildcardMatcher;
use ff_select::CriteriaEvaluator;

// ─── Strategies ─────────────────────────────────────────────────────────────

fn arb_operator() -> impl Strategy<Value = CriteriaOperator> {
    prop_oneof![
        Just(CriteriaOperator::Eq),
        Just(CriteriaOperator::Ne),
        Just(CriteriaOperator::Gt),
        Just(CriteriaOperator::Ge),
        Just(CriteriaOperator::Lt),
        Just(CriteriaOperator::Le),
        Just(CriteriaOperator::Contains),
        Just(CriteriaOperator::StartsWith),
        Just(CriteriaOperator::EndsWith),
        Just(CriteriaOperator::MatchesRegex),
    ]
}

fn arb_connector() -> impl Strategy<Value = CriteriaConnector> {
    prop_oneof![Just(CriteriaConnector::And), Just(CriteriaConnector::Or),]
}

fn arb_field_name() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9_]{0,9}".prop_map(|s| s)
}

fn arb_string_value() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{0,30}".prop_map(|s| s)
}

fn arb_criterion(is_last: bool) -> impl Strategy<Value = Criterion> {
    (
        any::<bool>(),
        arb_field_name(),
        arb_operator(),
        arb_string_value(),
        if is_last {
            Just(None).boxed()
        } else {
            arb_connector().prop_map(Some).boxed()
        },
    )
        .prop_map(|(enabled, field, operator, value, connector)| Criterion {
            enabled,
            field,
            operator,
            value,
            value2: None,
            connector,
            group_open: false,
            group_close: false,
        })
}

fn arb_criteria_set() -> impl Strategy<Value = CriteriaSet> {
    (
        proptest::option::of("[a-z_]{1,15}"),
        proptest::option::of("[A-Z_]{1,15}"),
        proptest::option::of("[A-Z_]{1,10}"),
        any::<bool>(),
        prop::collection::vec(arb_criterion(false), 0..5),
    )
        .prop_map(
            |(name, structure_association, record_type_scope, case_sensitive, mut criteria)| {
                // Fix the last criterion's connector to None
                if let Some(last) = criteria.last_mut() {
                    last.connector = None;
                }
                CriteriaSet {
                    name,
                    structure_association,
                    record_type_scope,
                    case_sensitive,
                    criteria,
                }
            },
        )
}

// ─── Property 1: Empty/All-Disabled Criteria Passthrough ────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 1.4, 1.5**
    ///
    /// When a CriteriaSet is empty or all criteria rows are disabled,
    /// evaluation returns matches: true for every record.
    // Feature: record-selection-criteria, Property 1: Empty/All-Disabled Criteria Passthrough
    #[test]
    fn prop_empty_or_all_disabled_passthrough(
        num_rows in 0usize..10,
        num_fields in 1usize..5,
    ) {
        // Generate a criteria set with all rows disabled
        let criteria: Vec<Criterion> = (0..num_rows)
            .map(|i| Criterion {
                enabled: false,
                field: format!("F{i}"),
                operator: CriteriaOperator::Eq,
                value: "x".to_string(),
                value2: None,
                connector: if i < num_rows - 1 { Some(CriteriaConnector::And) } else { None },
                group_open: false,
                group_close: false,
            })
            .collect();

        let cs = CriteriaSet {
            criteria,
            ..CriteriaSet::empty()
        };

        // Generate arbitrary field values
        let field_values: HashMap<String, String> = (0..num_fields)
            .map(|i| (format!("F{i}"), format!("val{i}")))
            .collect();
        let field_types: HashMap<String, FieldDataType> = (0..num_fields)
            .map(|i| (format!("F{i}"), FieldDataType::Str))
            .collect();

        let evaluator = CriteriaEvaluator::new();
        let result = evaluator.evaluate(&cs, &field_values, &field_types);
        prop_assert!(result.matches, "Passthrough criteria should always match");
    }
}

// ─── Property 3: EQ/NE Symmetry ────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 2.2, 2.3**
    ///
    /// For any field value and criterion value, EQ returns the logical
    /// negation of NE (and vice versa).
    // Feature: record-selection-criteria, Property 3: Operator Correctness — EQ Symmetry with NE
    #[test]
    fn prop_eq_ne_symmetry_string(
        field_value in "[a-zA-Z0-9]{0,20}",
        criterion_value in "[a-zA-Z0-9]{0,20}",
        case_sensitive in any::<bool>(),
    ) {
        let engine = ComparisonEngine::new();
        let eq_result = engine.compare(
            &field_value, &criterion_value,
            CriteriaOperator::Eq, ComparisonMode::String, case_sensitive,
        ).unwrap();
        let ne_result = engine.compare(
            &field_value, &criterion_value,
            CriteriaOperator::Ne, ComparisonMode::String, case_sensitive,
        ).unwrap();

        prop_assert_eq!(eq_result, !ne_result,
            "EQ and NE must be logical negations: field='{}', criterion='{}', eq={}, ne={}",
            field_value, criterion_value, eq_result, ne_result);
    }

    /// **Validates: Requirements 2.2, 2.3**
    // Feature: record-selection-criteria, Property 3: Operator Correctness — EQ Symmetry with NE (numeric)
    #[test]
    fn prop_eq_ne_symmetry_numeric(
        fv in -1000i64..1000,
        cv in -1000i64..1000,
    ) {
        let engine = ComparisonEngine::new();
        let fv_str = fv.to_string();
        let cv_str = cv.to_string();
        let eq_result = engine.compare(
            &fv_str, &cv_str,
            CriteriaOperator::Eq, ComparisonMode::Numeric, true,
        ).unwrap();
        let ne_result = engine.compare(
            &fv_str, &cv_str,
            CriteriaOperator::Ne, ComparisonMode::Numeric, true,
        ).unwrap();

        prop_assert_eq!(eq_result, !ne_result,
            "Numeric EQ and NE must be logical negations");
    }
}

// ─── Property 4: Ordering Consistency ───────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 2.4**
    ///
    /// The ordering operators form a consistent total order.
    // Feature: record-selection-criteria, Property 4: Ordering Consistency (GT/GE/LT/LE)
    #[test]
    fn prop_ordering_consistency_numeric(
        fv in -1000i64..1000,
        cv in -1000i64..1000,
    ) {
        let engine = ComparisonEngine::new();
        let fv_str = fv.to_string();
        let cv_str = cv.to_string();

        let eq = engine.compare(&fv_str, &cv_str, CriteriaOperator::Eq, ComparisonMode::Numeric, true).unwrap();
        let gt = engine.compare(&fv_str, &cv_str, CriteriaOperator::Gt, ComparisonMode::Numeric, true).unwrap();
        let lt = engine.compare(&fv_str, &cv_str, CriteriaOperator::Lt, ComparisonMode::Numeric, true).unwrap();
        let ge = engine.compare(&fv_str, &cv_str, CriteriaOperator::Ge, ComparisonMode::Numeric, true).unwrap();
        let le = engine.compare(&fv_str, &cv_str, CriteriaOperator::Le, ComparisonMode::Numeric, true).unwrap();

        // Exactly one of EQ, GT, LT is true
        let sum = eq as u8 + gt as u8 + lt as u8;
        prop_assert_eq!(sum, 1, "Exactly one of EQ/GT/LT must hold: eq={}, gt={}, lt={}, fv={}, cv={}", eq, gt, lt, fv, cv);

        // GE ≡ GT ∨ EQ
        prop_assert_eq!(ge, gt || eq, "GE must equal GT||EQ");
        // LE ≡ LT ∨ EQ
        prop_assert_eq!(le, lt || eq, "LE must equal LT||EQ");
    }

    /// **Validates: Requirements 2.4**
    // Feature: record-selection-criteria, Property 4: Ordering Consistency (GT/GE/LT/LE) (string)
    #[test]
    fn prop_ordering_consistency_string(
        fv in "[a-z]{1,10}",
        cv in "[a-z]{1,10}",
    ) {
        let engine = ComparisonEngine::new();

        let eq = engine.compare(&fv, &cv, CriteriaOperator::Eq, ComparisonMode::String, true).unwrap();
        let gt = engine.compare(&fv, &cv, CriteriaOperator::Gt, ComparisonMode::String, true).unwrap();
        let lt = engine.compare(&fv, &cv, CriteriaOperator::Lt, ComparisonMode::String, true).unwrap();
        let ge = engine.compare(&fv, &cv, CriteriaOperator::Ge, ComparisonMode::String, true).unwrap();
        let le = engine.compare(&fv, &cv, CriteriaOperator::Le, ComparisonMode::String, true).unwrap();

        let sum = eq as u8 + gt as u8 + lt as u8;
        prop_assert_eq!(sum, 1, "Exactly one of EQ/GT/LT must hold for strings");
        prop_assert_eq!(ge, gt || eq);
        prop_assert_eq!(le, lt || eq);
    }
}

// ─── Property 5: Case Sensitivity Toggle ────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 2.10, 2.11**
    ///
    /// When case_sensitive is false, string comparison results are identical
    /// regardless of the case of the input values.
    // Feature: record-selection-criteria, Property 5: Case Sensitivity Toggle
    #[test]
    fn prop_case_insensitive_equivalence(
        fv in "[a-zA-Z]{1,15}",
        cv in "[a-zA-Z]{1,15}",
        op in prop_oneof![
            Just(CriteriaOperator::Eq),
            Just(CriteriaOperator::Ne),
            Just(CriteriaOperator::Contains),
            Just(CriteriaOperator::StartsWith),
            Just(CriteriaOperator::EndsWith),
        ],
    ) {
        let engine = ComparisonEngine::new();

        let insensitive = engine.compare(&fv, &cv, op, ComparisonMode::String, false).unwrap();
        let lowered = engine.compare(
            &fv.to_lowercase(), &cv.to_lowercase(),
            op, ComparisonMode::String, true,
        ).unwrap();

        prop_assert_eq!(insensitive, lowered,
            "Case-insensitive comparison should equal comparison of lowered values");
    }
}

// ─── Property 6: Wildcard No-Op Without Pattern Characters ──────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 4.4**
    ///
    /// When a criterion value contains no wildcard characters,
    /// EQ with that value produces the same result as exact equality.
    // Feature: record-selection-criteria, Property 6: Wildcard No-Op Without Pattern Characters
    #[test]
    fn prop_wildcard_noop_without_pattern_chars(
        fv in "[a-zA-Z0-9]{0,20}",
        cv in "[a-zA-Z0-9]{0,20}",
        case_sensitive in any::<bool>(),
    ) {
        // Both values guaranteed to have no wildcards (alphanumeric only)
        prop_assert!(!WildcardMatcher::has_wildcards(&cv));

        let match_result = WildcardMatcher::matches(&fv, &cv, case_sensitive);
        let expected = if case_sensitive {
            fv == cv
        } else {
            fv.to_lowercase() == cv.to_lowercase()
        };

        prop_assert_eq!(match_result, expected,
            "Without wildcards, matching should equal exact equality");
    }
}

// ─── Property 7: Logical AND Strictness ─────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 5.1**
    ///
    /// Combining two criterion results with AND produces true only
    /// when both individual results are true.
    // Feature: record-selection-criteria, Property 7: Logical AND Strictness
    #[test]
    fn prop_and_strictness(a in any::<bool>(), b in any::<bool>()) {
        let rows = vec![
            LogicalRow { result: a, connector: Some(CriteriaConnector::And), group_open: false, group_close: false },
            LogicalRow { result: b, connector: None, group_open: false, group_close: false },
        ];
        let combined = LogicalCombiner::combine(&rows);
        prop_assert_eq!(combined, a && b, "AND must be strict conjunction");
    }
}

// ─── Property 8: Logical OR Leniency ───────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 5.1**
    ///
    /// Combining two criterion results with OR produces true when
    /// at least one individual result is true.
    // Feature: record-selection-criteria, Property 8: Logical OR Leniency
    #[test]
    fn prop_or_leniency(a in any::<bool>(), b in any::<bool>()) {
        let rows = vec![
            LogicalRow { result: a, connector: Some(CriteriaConnector::Or), group_open: false, group_close: false },
            LogicalRow { result: b, connector: None, group_open: false, group_close: false },
        ];
        let combined = LogicalCombiner::combine(&rows);
        prop_assert_eq!(combined, a || b, "OR must be lenient disjunction");
    }
}

// ─── Property 9: Group Override Precedence ──────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 5.2, 5.3**
    ///
    /// Parenthesised groups override default AND/OR precedence.
    /// A OR (B AND C) evaluates the group first.
    // Feature: record-selection-criteria, Property 9: Group Override Precedence
    #[test]
    fn prop_group_override_precedence(a in any::<bool>(), b in any::<bool>(), c in any::<bool>()) {
        let rows = vec![
            LogicalRow { result: a, connector: Some(CriteriaConnector::Or), group_open: false, group_close: false },
            LogicalRow { result: b, connector: Some(CriteriaConnector::And), group_open: true, group_close: false },
            LogicalRow { result: c, connector: None, group_open: false, group_close: true },
        ];
        let combined = LogicalCombiner::combine(&rows);
        prop_assert_eq!(combined, a || (b && c),
            "Group should override precedence: a={}, b={}, c={}, result={}", a, b, c, combined);
    }
}

// ─── Property 10: JSON Round-Trip Preservation ──────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 1.6**
    ///
    /// Serialising a CriteriaSet to JSON and deserialising back
    /// produces an identical CriteriaSet.
    // Feature: record-selection-criteria, Property 10: JSON Round-Trip Preservation
    #[test]
    fn prop_json_round_trip(cs in arb_criteria_set()) {
        let json = cs.to_json().unwrap();
        let restored = CriteriaSet::from_json(&json).unwrap();
        prop_assert_eq!(&cs, &restored, "JSON round-trip must preserve CriteriaSet");
    }
}

// ─── Property 11: Filter State Indicator Consistency ────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 13.1, 13.2**
    ///
    /// format_indicator() returns Some(...) if and only if a CriteriaSet is active.
    // Feature: record-selection-criteria, Property 11: Filter State Indicator Consistency
    #[test]
    fn prop_filter_state_indicator_consistency(
        is_active in any::<bool>(),
        visible in 0usize..1000,
        total in 0usize..10000,
    ) {
        let mut fs = FilterState::inactive();
        if is_active {
            let cs = CriteriaSet::single("F", CriteriaOperator::Eq, "v");
            fs.apply(cs, visible, total);
        }

        prop_assert_eq!(
            fs.format_indicator().is_some(),
            fs.is_active(),
            "format_indicator().is_some() must equal is_active()"
        );
    }
}

// ─── Property 12: Criteria Scope Record Containment ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 8.1, 8.6**
    ///
    /// A CriteriaScope constructed from matching record indices correctly
    /// reports containment for exactly those indices and no others.
    // Feature: record-selection-criteria, Property 12: Criteria Scope Record Containment
    #[test]
    fn prop_scope_record_containment(
        indices in prop::collection::vec(0usize..1000, 0..50),
        query in 0usize..1000,
    ) {
        let scope = CriteriaScope::new(indices.clone());
        let expected = indices.contains(&query);
        let actual = scope.contains_record(query);
        prop_assert_eq!(actual, expected,
            "Scope containment must match vec::contains for query={}", query);
    }
}

// ─── Property 2: Disabled Row Skip Equivalence ──────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// **Validates: Requirements 1.5**
    ///
    /// Evaluating a CriteriaSet with a disabled row produces the same result
    /// as evaluating the CriteriaSet with that row removed entirely.
    // Feature: record-selection-criteria, Property 2: Disabled Row Skip Equivalence
    #[test]
    fn prop_disabled_row_skip_equivalence(
        field_value in "[a-z]{1,10}",
        criterion_value in "[a-z]{1,10}",
        disable_index in 0usize..3,
    ) {
        let field_name = "FIELD";
        let num_rows = 3;
        let disable_idx = disable_index % num_rows;

        // Build a criteria set with 3 rows (all EQ for simplicity)
        let mut criteria_with_disabled: Vec<Criterion> = (0..num_rows)
            .map(|i| Criterion {
                enabled: true,
                field: field_name.to_string(),
                operator: CriteriaOperator::Eq,
                value: criterion_value.clone(),
                value2: None,
                connector: if i < num_rows - 1 { Some(CriteriaConnector::And) } else { None },
                group_open: false,
                group_close: false,
            })
            .collect();

        // Disable one row
        criteria_with_disabled[disable_idx].enabled = false;

        // Build the equivalent set without the disabled row
        let mut criteria_without: Vec<Criterion> = criteria_with_disabled
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != disable_idx)
            .map(|(_, c)| c.clone())
            .collect();
        // Fix connectors: last row must have None
        if let Some(last) = criteria_without.last_mut() {
            last.connector = None;
        }

        let cs_with = CriteriaSet { criteria: criteria_with_disabled, ..CriteriaSet::empty() };
        let cs_without = CriteriaSet { criteria: criteria_without, ..CriteriaSet::empty() };

        let mut field_values = HashMap::new();
        field_values.insert(field_name.to_string(), field_value.clone());
        let mut field_types = HashMap::new();
        field_types.insert(field_name.to_string(), FieldDataType::Str);

        let evaluator = CriteriaEvaluator::new();
        let result_with = evaluator.evaluate(&cs_with, &field_values, &field_types);
        let result_without = evaluator.evaluate(&cs_without, &field_values, &field_types);

        prop_assert_eq!(result_with.matches, result_without.matches,
            "Disabled row should produce same result as removed row");
    }
}
