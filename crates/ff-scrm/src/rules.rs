//! Capture rules and masking rules.
//!
//! A capture rule decides whether a screen SHOULD be captured based on
//! conditions (screen name, program, message class, transaction id, dataset
//! name, user id). A masking rule decides which field values are hidden on
//! export.
//!
//! Validates: screen-snapshot-scrm Requirement 9.7, 9.8, 9.9, 13.1-13.4.

use ff_screen_model::ScreenModel;
use serde::{Deserialize, Serialize};

/// The context a capture rule is evaluated against.
///
/// Validates: Requirement 9.8 (the supported condition fields).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CaptureContext {
    /// The logical screen name.
    pub screen_name: Option<String>,
    /// The program/command name.
    pub program_name: Option<String>,
    /// The message class.
    pub message_class: Option<String>,
    /// The transaction id.
    pub transaction_id: Option<String>,
    /// The dataset name.
    pub dataset_name: Option<String>,
    /// The user id.
    pub user_id: Option<String>,
}

/// A single condition on one field of the [`CaptureContext`].
///
/// Validates: Requirement 9.8.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "field", content = "equals", rename_all = "snake_case")]
pub enum Condition {
    /// Screen name equals (case-insensitive).
    ScreenName(String),
    /// Program name equals.
    ProgramName(String),
    /// Message class equals.
    MessageClass(String),
    /// Transaction id equals.
    TransactionId(String),
    /// Dataset name equals.
    DatasetName(String),
    /// User id equals.
    UserId(String),
}

impl Condition {
    /// Evaluate this condition against a context.
    fn matches(&self, ctx: &CaptureContext) -> bool {
        fn eq(actual: &Option<String>, expected: &str) -> bool {
            actual
                .as_deref()
                .is_some_and(|a| a.eq_ignore_ascii_case(expected))
        }
        match self {
            Condition::ScreenName(v) => eq(&ctx.screen_name, v),
            Condition::ProgramName(v) => eq(&ctx.program_name, v),
            Condition::MessageClass(v) => eq(&ctx.message_class, v),
            Condition::TransactionId(v) => eq(&ctx.transaction_id, v),
            Condition::DatasetName(v) => eq(&ctx.dataset_name, v),
            Condition::UserId(v) => eq(&ctx.user_id, v),
        }
    }
}

/// A capture rule: all its conditions must hold (AND) for the rule to fire.
///
/// Validates: Requirement 9.7.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureRule {
    /// A human-readable rule name.
    pub name: String,
    /// The conditions; empty means "always matches".
    pub conditions: Vec<Condition>,
}

impl CaptureRule {
    /// Whether this rule fires for the given context.
    pub fn matches(&self, ctx: &CaptureContext) -> bool {
        self.conditions.iter().all(|c| c.matches(ctx))
    }
}

/// A set of capture rules, any of which firing means "capture".
///
/// Validates: Requirement 9.9 (multiple active rules).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureRuleSet {
    /// The active rules.
    pub rules: Vec<CaptureRule>,
}

impl CaptureRuleSet {
    /// Should a screen with this context be captured? True if ANY rule fires.
    /// With no rules, returns false (rules-based capture is opt-in).
    ///
    /// Validates: Requirement 9.7, 9.9.
    pub fn should_capture(&self, ctx: &CaptureContext) -> bool {
        self.rules.iter().any(|r| r.matches(ctx))
    }
}

/// How a masked field value is rendered on export.
const MASK: &str = "********";

/// A masking rule set. Any field marked sensitive on the model is masked; a
/// rule can additionally mask by label match.
///
/// Validates: Requirement 13.1, 13.2.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaskingRules {
    /// Field labels (case-insensitive) whose values are masked.
    pub mask_labels: Vec<String>,
}

impl MaskingRules {
    /// Return a copy of `model` with sensitive/masked field values replaced by
    /// the mask token. A field is masked if its `attrs.sensitive` is set OR its
    /// label matches a `mask_labels` entry (Requirement 13.1).
    ///
    /// Validates: Requirement 13.1, 13.2.
    pub fn apply(&self, model: &ScreenModel) -> ScreenModel {
        let mut out = model.clone();
        for field in &mut out.fields {
            let label_match = self
                .mask_labels
                .iter()
                .any(|l| l.eq_ignore_ascii_case(&field.label));
            if field.attrs.sensitive || label_match {
                field.value = MASK.to_string();
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_screen_model::{Field, FieldAttributes};

    // Validates: Requirement 9.7 -- a rule fires when its condition matches.
    #[test]
    fn rule_matches_on_screen_name_case_insensitive() {
        let rule = CaptureRule {
            name: "edit screens".to_string(),
            conditions: vec![Condition::ScreenName("EDIT".to_string())],
        };
        let ctx = CaptureContext {
            screen_name: Some("edit".to_string()),
            ..Default::default()
        };
        assert!(rule.matches(&ctx));
    }

    // Validates: Requirement 9.9 -- any of several rules firing means capture.
    #[test]
    fn ruleset_captures_if_any_rule_fires() {
        let set = CaptureRuleSet {
            rules: vec![
                CaptureRule {
                    name: "a".to_string(),
                    conditions: vec![Condition::UserId("ADMIN".to_string())],
                },
                CaptureRule {
                    name: "b".to_string(),
                    conditions: vec![Condition::DatasetName("PAYROLL".to_string())],
                },
            ],
        };
        let ctx = CaptureContext {
            dataset_name: Some("payroll".to_string()),
            ..Default::default()
        };
        assert!(set.should_capture(&ctx));
        assert!(!set.should_capture(&CaptureContext::default()));
    }

    // Validates: Requirement 13.1 -- sensitive field is masked on apply.
    #[test]
    fn masking_hides_sensitive_field() {
        let m = ScreenModel::new("Login").with_field(
            Field::new("Password", "hunter2").with_attrs(FieldAttributes::plain().sensitive()),
        );
        let masked = MaskingRules::default().apply(&m);
        assert_eq!(masked.fields[0].value, MASK);
    }

    // Validates: Requirement 13.2 -- label-based masking rule.
    #[test]
    fn masking_by_label_rule() {
        let m = ScreenModel::new("Login").with_field(Field::new("PIN", "1234"));
        let rules = MaskingRules {
            mask_labels: vec!["pin".to_string()],
        };
        let masked = rules.apply(&m);
        assert_eq!(masked.fields[0].value, MASK);
    }
}
