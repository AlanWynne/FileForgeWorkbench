//! Field-type-aware comparison engine.
//!
//! Performs comparisons between field values and criterion values using
//! the appropriate semantics for the field's data type (string, numeric,
//! or packed-decimal).

use crate::error::CriteriaError;
use crate::model::{ComparisonMode, CriteriaOperator};
use crate::types::FieldDataType;
use crate::wildcard::WildcardMatcher;

/// Performs field-type-aware comparisons between field values and criterion values.
pub struct ComparisonEngine;

impl ComparisonEngine {
    /// Create a new comparison engine.
    pub fn new() -> Self {
        Self
    }

    /// Determine the comparison mode for a given field data type.
    ///
    /// Addresses: Requirement 3 AC 1, 2, 3
    pub fn determine_mode(field_type: &FieldDataType) -> ComparisonMode {
        match field_type {
            FieldDataType::Int | FieldDataType::Float => ComparisonMode::Numeric,
            FieldDataType::Packed => ComparisonMode::PackedDecimal,
            FieldDataType::Str | FieldDataType::Bool | FieldDataType::Ebcdic => {
                ComparisonMode::String
            }
        }
    }

    /// Compare a field value against a criterion value using the specified operator.
    ///
    /// Addresses: Requirement 2, Requirement 3
    pub fn compare(
        &self,
        field_value: &str,
        criterion_value: &str,
        operator: CriteriaOperator,
        mode: ComparisonMode,
        case_sensitive: bool,
    ) -> Result<bool, CriteriaError> {
        match operator {
            CriteriaOperator::Contains => {
                Ok(self.compare_contains(field_value, criterion_value, case_sensitive))
            }
            CriteriaOperator::StartsWith => {
                Ok(self.compare_starts_with(field_value, criterion_value, case_sensitive))
            }
            CriteriaOperator::EndsWith => {
                Ok(self.compare_ends_with(field_value, criterion_value, case_sensitive))
            }
            CriteriaOperator::MatchesRegex => {
                self.compare_regex(field_value, criterion_value, case_sensitive)
            }
            _ => match mode {
                ComparisonMode::Numeric | ComparisonMode::PackedDecimal => {
                    self.compare_numeric(field_value, criterion_value, operator)
                }
                ComparisonMode::String => {
                    Ok(self.compare_string(field_value, criterion_value, operator, case_sensitive))
                }
            },
        }
    }

    /// Numeric comparison: parse both values as f64, compare with operator.
    fn compare_numeric(
        &self,
        field_value: &str,
        criterion_value: &str,
        operator: CriteriaOperator,
    ) -> Result<bool, CriteriaError> {
        let fv: f64 =
            field_value
                .trim()
                .parse()
                .map_err(|_| CriteriaError::NumericParseFailed {
                    field: String::from("field"),
                    value: field_value.to_string(),
                })?;
        let cv: f64 =
            criterion_value
                .trim()
                .parse()
                .map_err(|_| CriteriaError::NumericParseFailed {
                    field: String::from("criterion"),
                    value: criterion_value.to_string(),
                })?;

        Ok(match operator {
            CriteriaOperator::Eq => (fv - cv).abs() < f64::EPSILON,
            CriteriaOperator::Ne => (fv - cv).abs() >= f64::EPSILON,
            CriteriaOperator::Gt => fv > cv,
            CriteriaOperator::Ge => fv >= cv,
            CriteriaOperator::Lt => fv < cv,
            CriteriaOperator::Le => fv <= cv,
            _ => false,
        })
    }

    /// String comparison: equality, ordering, or wildcard-enhanced equality.
    fn compare_string(
        &self,
        field_value: &str,
        criterion_value: &str,
        operator: CriteriaOperator,
        case_sensitive: bool,
    ) -> bool {
        // For EQ/NE with wildcards, delegate to WildcardMatcher
        if matches!(operator, CriteriaOperator::Eq | CriteriaOperator::Ne)
            && WildcardMatcher::has_wildcards(criterion_value)
        {
            let wildcard_match =
                WildcardMatcher::matches(field_value, criterion_value, case_sensitive);
            return match operator {
                CriteriaOperator::Eq => wildcard_match,
                CriteriaOperator::Ne => !wildcard_match,
                _ => unreachable!(),
            };
        }

        let (fv, cv) = if case_sensitive {
            (field_value.to_string(), criterion_value.to_string())
        } else {
            (field_value.to_lowercase(), criterion_value.to_lowercase())
        };

        match operator {
            CriteriaOperator::Eq => fv == cv,
            CriteriaOperator::Ne => fv != cv,
            CriteriaOperator::Gt => fv > cv,
            CriteriaOperator::Ge => fv >= cv,
            CriteriaOperator::Lt => fv < cv,
            CriteriaOperator::Le => fv <= cv,
            _ => false,
        }
    }

    /// CONTAINS operator: substring check.
    fn compare_contains(
        &self,
        field_value: &str,
        criterion_value: &str,
        case_sensitive: bool,
    ) -> bool {
        if case_sensitive {
            field_value.contains(criterion_value)
        } else {
            field_value
                .to_lowercase()
                .contains(&criterion_value.to_lowercase())
        }
    }

    /// STARTS_WITH operator: prefix check.
    fn compare_starts_with(
        &self,
        field_value: &str,
        criterion_value: &str,
        case_sensitive: bool,
    ) -> bool {
        if case_sensitive {
            field_value.starts_with(criterion_value)
        } else {
            field_value
                .to_lowercase()
                .starts_with(&criterion_value.to_lowercase())
        }
    }

    /// ENDS_WITH operator: suffix check.
    fn compare_ends_with(
        &self,
        field_value: &str,
        criterion_value: &str,
        case_sensitive: bool,
    ) -> bool {
        if case_sensitive {
            field_value.ends_with(criterion_value)
        } else {
            field_value
                .to_lowercase()
                .ends_with(&criterion_value.to_lowercase())
        }
    }

    /// MATCHES_REGEX operator: regex pattern match.
    fn compare_regex(
        &self,
        field_value: &str,
        pattern: &str,
        case_sensitive: bool,
    ) -> Result<bool, CriteriaError> {
        let regex_pattern = if case_sensitive {
            pattern.to_string()
        } else {
            format!("(?i){pattern}")
        };

        let re = regex::Regex::new(&regex_pattern).map_err(|e| CriteriaError::InvalidRegex {
            row: 0,
            pattern: pattern.to_string(),
            detail: e.to_string(),
        })?;

        Ok(re.is_match(field_value))
    }
}

impl Default for ComparisonEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> ComparisonEngine {
        ComparisonEngine::new()
    }

    // --- determine_mode tests ---

    #[test]
    fn determine_mode_int_is_numeric() {
        assert_eq!(
            ComparisonEngine::determine_mode(&FieldDataType::Int),
            ComparisonMode::Numeric
        );
    }

    #[test]
    fn determine_mode_float_is_numeric() {
        assert_eq!(
            ComparisonEngine::determine_mode(&FieldDataType::Float),
            ComparisonMode::Numeric
        );
    }

    #[test]
    fn determine_mode_packed_is_packed_decimal() {
        assert_eq!(
            ComparisonEngine::determine_mode(&FieldDataType::Packed),
            ComparisonMode::PackedDecimal
        );
    }

    #[test]
    fn determine_mode_str_is_string() {
        assert_eq!(
            ComparisonEngine::determine_mode(&FieldDataType::Str),
            ComparisonMode::String
        );
    }

    // --- EQ/NE string tests ---

    #[test]
    fn eq_string_exact_match() {
        let e = engine();
        assert!(e
            .compare(
                "hello",
                "hello",
                CriteriaOperator::Eq,
                ComparisonMode::String,
                true
            )
            .unwrap());
    }

    #[test]
    fn eq_string_case_insensitive() {
        let e = engine();
        assert!(e
            .compare(
                "Hello",
                "hello",
                CriteriaOperator::Eq,
                ComparisonMode::String,
                false
            )
            .unwrap());
    }

    #[test]
    fn ne_string_returns_opposite_of_eq() {
        let e = engine();
        let eq = e
            .compare(
                "abc",
                "def",
                CriteriaOperator::Eq,
                ComparisonMode::String,
                true,
            )
            .unwrap();
        let ne = e
            .compare(
                "abc",
                "def",
                CriteriaOperator::Ne,
                ComparisonMode::String,
                true,
            )
            .unwrap();
        assert_eq!(eq, !ne);
    }

    // --- Ordering tests ---

    #[test]
    fn gt_string_lexicographic() {
        let e = engine();
        assert!(e
            .compare("b", "a", CriteriaOperator::Gt, ComparisonMode::String, true)
            .unwrap());
        assert!(!e
            .compare("a", "b", CriteriaOperator::Gt, ComparisonMode::String, true)
            .unwrap());
    }

    #[test]
    fn le_string_includes_equal() {
        let e = engine();
        assert!(e
            .compare("a", "a", CriteriaOperator::Le, ComparisonMode::String, true)
            .unwrap());
        assert!(e
            .compare("a", "b", CriteriaOperator::Le, ComparisonMode::String, true)
            .unwrap());
    }

    // --- Numeric tests ---

    #[test]
    fn eq_numeric_match() {
        let e = engine();
        assert!(e
            .compare(
                "42",
                "42",
                CriteriaOperator::Eq,
                ComparisonMode::Numeric,
                true
            )
            .unwrap());
    }

    #[test]
    fn gt_numeric_comparison() {
        let e = engine();
        assert!(e
            .compare(
                "100",
                "50",
                CriteriaOperator::Gt,
                ComparisonMode::Numeric,
                true
            )
            .unwrap());
        assert!(!e
            .compare(
                "50",
                "100",
                CriteriaOperator::Gt,
                ComparisonMode::Numeric,
                true
            )
            .unwrap());
    }

    #[test]
    fn numeric_parse_failure_returns_error() {
        let e = engine();
        let result = e.compare(
            "abc",
            "42",
            CriteriaOperator::Eq,
            ComparisonMode::Numeric,
            true,
        );
        assert!(result.is_err());
    }

    // --- CONTAINS/STARTS_WITH/ENDS_WITH tests ---

    #[test]
    fn contains_substring_match() {
        let e = engine();
        assert!(e
            .compare(
                "hello world",
                "lo wo",
                CriteriaOperator::Contains,
                ComparisonMode::String,
                true
            )
            .unwrap());
    }

    #[test]
    fn starts_with_prefix_match() {
        let e = engine();
        assert!(e
            .compare(
                "hello world",
                "hello",
                CriteriaOperator::StartsWith,
                ComparisonMode::String,
                true
            )
            .unwrap());
        assert!(!e
            .compare(
                "hello world",
                "world",
                CriteriaOperator::StartsWith,
                ComparisonMode::String,
                true
            )
            .unwrap());
    }

    #[test]
    fn ends_with_suffix_match() {
        let e = engine();
        assert!(e
            .compare(
                "hello world",
                "world",
                CriteriaOperator::EndsWith,
                ComparisonMode::String,
                true
            )
            .unwrap());
        assert!(!e
            .compare(
                "hello world",
                "hello",
                CriteriaOperator::EndsWith,
                ComparisonMode::String,
                true
            )
            .unwrap());
    }

    // --- MATCHES_REGEX tests ---

    #[test]
    fn regex_partial_match() {
        let e = engine();
        assert!(e
            .compare(
                "hello123world",
                r"\d+",
                CriteriaOperator::MatchesRegex,
                ComparisonMode::String,
                true
            )
            .unwrap());
    }

    #[test]
    fn regex_invalid_pattern_returns_error() {
        let e = engine();
        let result = e.compare(
            "hello",
            "[invalid",
            CriteriaOperator::MatchesRegex,
            ComparisonMode::String,
            true,
        );
        assert!(result.is_err());
    }

    #[test]
    fn regex_case_insensitive() {
        let e = engine();
        assert!(e
            .compare(
                "HELLO",
                "hello",
                CriteriaOperator::MatchesRegex,
                ComparisonMode::String,
                false
            )
            .unwrap());
    }

    // --- Wildcard integration tests ---

    #[test]
    fn eq_with_wildcards_delegates_to_wildcard_matcher() {
        let e = engine();
        assert!(e
            .compare(
                "hello world",
                "hello*",
                CriteriaOperator::Eq,
                ComparisonMode::String,
                true
            )
            .unwrap());
        assert!(!e
            .compare(
                "goodbye",
                "hello*",
                CriteriaOperator::Eq,
                ComparisonMode::String,
                true
            )
            .unwrap());
    }

    #[test]
    fn ne_with_wildcards_is_negated_wildcard_match() {
        let e = engine();
        assert!(!e
            .compare(
                "hello world",
                "hello*",
                CriteriaOperator::Ne,
                ComparisonMode::String,
                true
            )
            .unwrap());
        assert!(e
            .compare(
                "goodbye",
                "hello*",
                CriteriaOperator::Ne,
                ComparisonMode::String,
                true
            )
            .unwrap());
    }
}
