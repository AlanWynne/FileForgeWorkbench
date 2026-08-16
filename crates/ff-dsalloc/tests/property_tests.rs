//! Property-based tests for the ff-dsalloc crate.
//!
//! These tests verify universal properties hold across all inputs using proptest.

use proptest::prelude::*;

use ff_dsalloc::dsn::DatasetName;
use ff_dsalloc::operands::DispParameter;
use ff_dsalloc::symbols::SymbolTable;

// ─── Property 1: DSN Validation ──────────────────────────────────────────────

/// **Validates: Requirement 10 AC 7**
///
/// Feature: ff-dsalloc, Property 1: DSN validation accepts only valid names
/// per z/OS rules (≤44 chars, qualifiers ≤8, alpha/national start, no consecutive dots).
mod dsn_validation {
    use super::*;

    /// Strategy generating valid DSN qualifiers (1-8 chars, alpha start).
    fn valid_qualifier() -> impl Strategy<Value = String> {
        let first_char = prop_oneof![
            prop::char::ranges(std::borrow::Cow::Borrowed(&[('A'..='Z')])),
            Just('@'),
            Just('#'),
            Just('$'),
        ];
        let rest_chars = prop::collection::vec(
            prop_oneof![
                prop::char::ranges(std::borrow::Cow::Borrowed(&[('A'..='Z')])),
                prop::char::ranges(std::borrow::Cow::Borrowed(&[('0'..='9')])),
                Just('@'),
                Just('#'),
                Just('$'),
            ],
            0..7, // 0-7 more chars (total 1-8)
        );

        (first_char, rest_chars).prop_map(|(first, rest)| {
            let mut s = String::with_capacity(8);
            s.push(first);
            for ch in rest {
                s.push(ch);
            }
            s
        })
    }

    /// Strategy generating valid DSNs (1-4 qualifiers, dot-separated).
    fn valid_dsn_strategy() -> impl Strategy<Value = String> {
        prop::collection::vec(valid_qualifier(), 1..=4)
            .prop_map(|quals| quals.join("."))
            .prop_filter("DSN must be ≤44 chars", |dsn| dsn.len() <= 44)
    }

    proptest! {
        #[test]
        fn valid_dsns_always_parse_successfully(dsn in valid_dsn_strategy()) {
            let result = DatasetName::parse(&dsn, 1, 0);
            prop_assert!(result.is_ok(), "Valid DSN '{}' should parse successfully, got: {:?}", dsn, result.err());
        }

        #[test]
        fn dsns_exceeding_44_chars_always_rejected(
            base in "[A-Z]{8}",
            count in 6..=8usize,
        ) {
            // Build a DSN with many qualifiers to exceed 44 chars
            let dsn: String = (0..count).map(|_| base.as_str()).collect::<Vec<_>>().join(".");
            if dsn.len() > 44 {
                let result = DatasetName::parse(&dsn, 1, 0);
                prop_assert!(result.is_err(), "DSN '{}' (len {}) should be rejected", dsn, dsn.len());
            }
        }

        #[test]
        fn qualifier_starting_with_digit_always_rejected(
            digit in prop::char::ranges(std::borrow::Cow::Borrowed(&[('0'..='9')])),
            rest in "[A-Z]{1,7}",
        ) {
            let dsn = format!("VALID.{}{}", digit, rest);
            let result = DatasetName::parse(&dsn, 1, 0);
            prop_assert!(result.is_err(), "DSN '{}' with digit-start qualifier should be rejected", dsn);
        }
    }
}

// ─── Property 4: Substitution Idempotence ────────────────────────────────────

/// **Validates: Requirement 3 AC 1, AC 9**
///
/// Feature: ff-dsalloc, Property 4: substitution idempotence —
/// after substituting known symbols, no `&known_symbol` remains;
/// substituting again produces the same output.
mod substitution_idempotence {
    use super::*;

    proptest! {
        #[test]
        fn substitution_is_idempotent_for_known_symbols(
            sym_name in "[A-Z]{1,4}",
            sym_value in "[A-Z0-9]{1,8}",
            prefix in "[A-Z.]{0,10}",
            suffix in "[A-Z.]{0,10}",
        ) {
            let mut table = SymbolTable::new();
            table.define(&sym_name, &sym_value);

            let input = format!("{}&{}.{}", prefix, sym_name, suffix);
            let first_pass = ff_dsalloc::symbols::substitute_symbols(&input, &table, 1);

            // After first pass, the known symbol should be gone
            let marker = format!("&{}", sym_name);
            if first_pass.diagnostics.is_empty() {
                prop_assert!(
                    !first_pass.text.contains(&marker),
                    "After substitution, '{}' should not contain '{}'. Got: '{}'",
                    input, marker, first_pass.text
                );

                // Second pass should produce identical output (idempotent)
                let second_pass = ff_dsalloc::symbols::substitute_symbols(&first_pass.text, &table, 1);
                prop_assert_eq!(
                    &first_pass.text, &second_pass.text,
                    "Substitution should be idempotent"
                );
            }
        }
    }
}

// ─── Property 5: Dot Terminator Correctness ──────────────────────────────────

/// **Validates: Requirement 3 AC 6**
///
/// Feature: ff-dsalloc, Property 5: dot terminator correctness —
/// `&SYM.suffix` always produces `value(SYM) + suffix` with dot consumed.
mod dot_terminator {
    use super::*;

    proptest! {
        #[test]
        fn dot_terminator_always_consumed(
            sym_value in "[A-Z]{1,8}",
            suffix in "[A-Z0-9]{1,8}",
        ) {
            let mut table = SymbolTable::new();
            table.define("SYM", &sym_value);

            let input = format!("&SYM.{}", suffix);
            let result = ff_dsalloc::symbols::substitute_symbols(&input, &table, 1);

            let expected = format!("{}{}", sym_value, suffix);
            prop_assert_eq!(
                result.text, expected.clone(),
                "Input '{}' should produce '{}'",
                input, expected
            );
        }
    }
}

// ─── Property 7: DISP Default Application ───────────────────────────────────

/// **Validates: Requirement 4 AC 7**
///
/// Feature: ff-dsalloc, Property 7: DISP default application —
/// DD statements without explicit DISP always default to (NEW, DELETE).
mod disp_defaults {
    use super::*;
    use ff_dsalloc::operands::{DispAction, DispStatus};

    proptest! {
        #[test]
        fn default_disp_is_always_new_delete(_ignored in 0..100u32) {
            let disp = DispParameter::default_disp();
            prop_assert_eq!(disp.status, DispStatus::New);
            prop_assert_eq!(disp.normal_disp, Some(DispAction::Delete));
            prop_assert_eq!(disp.abnormal_disp, None);
        }
    }
}
