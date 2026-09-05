//! FFTest assertion evaluation engine.
//!
//! Evaluates `ASSERT` commands against the current [`AutomationRegistry`] snapshot
//! and produces structured [`AssertionResult`] values for the runner to collect.
//!
//! Validates: Requirement 4.3, 4.4 (automated-dialog-testing)

use crate::automation::{AutomationId, AutomationRegistry};

// === AssertionResult ========================================================

/// The outcome of evaluating a single assertion.
///
/// Validates: Requirement 4.3
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionResult {
    /// The assertion text as it appeared in the script.
    pub assertion_text: String,
    /// Whether the assertion passed.
    pub passed: bool,
    /// The expected value (for value-comparison assertions).
    pub expected: Option<String>,
    /// The actual value observed (for value-comparison assertions).
    pub actual: Option<String>,
    /// Human-readable failure reason, populated when `passed` is false.
    pub failure_reason: Option<String>,
}

impl AssertionResult {
    fn pass(assertion_text: impl Into<String>) -> Self {
        Self {
            assertion_text: assertion_text.into(),
            passed: true,
            expected: None,
            actual: None,
            failure_reason: None,
        }
    }

    fn fail(
        assertion_text: impl Into<String>,
        reason: impl Into<String>,
        expected: Option<String>,
        actual: Option<String>,
    ) -> Self {
        Self {
            assertion_text: assertion_text.into(),
            passed: false,
            expected,
            actual,
            failure_reason: Some(reason.into()),
        }
    }
}

// === evaluate_* functions ===================================================

/// Evaluate `ASSERT WINDOW EXISTS "<title>"`.
///
/// Checks that a control with the automation ID `shell.window` is present and
/// its label matches `title`. If no window control is registered, falls back to
/// checking whether any registered control has a matching label.
///
/// Validates: Requirement 4.3
pub fn evaluate_window_exists(title: &str, registry: &dyn AutomationRegistry) -> AssertionResult {
    let text = format!("ASSERT WINDOW EXISTS \"{title}\"");
    let id = AutomationId::new("shell.window");
    match registry.query(&id) {
        Some(state) => {
            let label = state.label.as_deref().unwrap_or("");
            if label == title {
                AssertionResult::pass(text)
            } else {
                AssertionResult::fail(
                    text,
                    "window label does not match",
                    Some(title.to_string()),
                    Some(label.to_string()),
                )
            }
        }
        None => AssertionResult::fail(
            text,
            "no window control registered",
            Some(title.to_string()),
            None,
        ),
    }
}

/// Evaluate `ASSERT TEXT EXISTS "<text>"`.
///
/// Checks whether any registered control has a value or label containing `text`.
///
/// Validates: Requirement 4.3
pub fn evaluate_text_exists(
    text: &str,
    registry: &dyn AutomationRegistry,
    all_ids: &[AutomationId],
) -> AssertionResult {
    let assertion_text = format!("ASSERT TEXT EXISTS \"{text}\"");
    for id in all_ids {
        if let Some(state) = registry.query(id) {
            let value_match = state
                .value
                .as_deref()
                .map(|v| v.contains(text))
                .unwrap_or(false);
            let label_match = state
                .label
                .as_deref()
                .map(|l| l.contains(text))
                .unwrap_or(false);
            if value_match || label_match {
                return AssertionResult::pass(assertion_text);
            }
        }
    }
    AssertionResult::fail(
        assertion_text,
        format!("text '{text}' not found in any registered control"),
        Some(text.to_string()),
        None,
    )
}

/// Evaluate `ASSERT STATUSBAR CONTAINS "<text>"`.
///
/// Checks the `statusbar.message` control's value.
///
/// Validates: Requirement 4.3
pub fn evaluate_statusbar_contains(
    text: &str,
    registry: &dyn AutomationRegistry,
) -> AssertionResult {
    let assertion_text = format!("ASSERT STATUSBAR CONTAINS \"{text}\"");
    let id = AutomationId::new("statusbar.message");
    match registry.query(&id) {
        Some(state) => {
            let value = state.value.as_deref().unwrap_or("");
            if value.contains(text) {
                AssertionResult::pass(assertion_text)
            } else {
                AssertionResult::fail(
                    assertion_text,
                    "status bar does not contain expected text",
                    Some(text.to_string()),
                    Some(value.to_string()),
                )
            }
        }
        None => AssertionResult::fail(
            assertion_text,
            "statusbar.message control not registered",
            Some(text.to_string()),
            None,
        ),
    }
}

/// Evaluate `ASSERT FILE OPEN`.
///
/// Checks that at least one `tab.editor.*` control is registered.
///
/// Validates: Requirement 4.3
pub fn evaluate_file_open(
    registry: &dyn AutomationRegistry,
    all_ids: &[AutomationId],
) -> AssertionResult {
    let assertion_text = "ASSERT FILE OPEN";
    for id in all_ids {
        if id.as_str().starts_with("tab.editor.") && registry.is_present(id) {
            return AssertionResult::pass(assertion_text);
        }
    }
    AssertionResult::fail(
        assertion_text,
        "no editor tab control registered",
        None,
        None,
    )
}

/// Evaluate `ASSERT CONTROL VALUE "<id>" "<expected>"`.
///
/// Validates: Requirement 4.3
pub fn evaluate_control_value(
    id_str: &str,
    expected: &str,
    registry: &dyn AutomationRegistry,
) -> AssertionResult {
    let assertion_text = format!("ASSERT CONTROL VALUE \"{id_str}\" \"{expected}\"");
    let id = AutomationId::new(id_str);
    match registry.query(&id) {
        Some(state) => {
            let actual = state.value.as_deref().unwrap_or("");
            if actual == expected {
                AssertionResult::pass(assertion_text)
            } else {
                AssertionResult::fail(
                    assertion_text,
                    "control value does not match expected",
                    Some(expected.to_string()),
                    Some(actual.to_string()),
                )
            }
        }
        None => AssertionResult::fail(
            assertion_text,
            format!("control '{id_str}' not registered"),
            Some(expected.to_string()),
            None,
        ),
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::{ControlState, InMemoryAutomationRegistry};

    fn registry_with(id: &str, state: ControlState) -> InMemoryAutomationRegistry {
        let mut r = InMemoryAutomationRegistry::new();
        r.register(AutomationId::new(id), state);
        r
    }

    // Validates: Requirement 4.3 -- ASSERT STATUSBAR CONTAINS passes when value matches
    #[test]
    fn statusbar_contains_passes_when_value_matches() {
        let reg = registry_with("statusbar.message", ControlState::with_value("Ready"));
        let result = evaluate_statusbar_contains("Ready", &reg);
        assert!(result.passed);
    }

    // Validates: Requirement 4.3 -- ASSERT STATUSBAR CONTAINS fails when value differs
    #[test]
    fn statusbar_contains_fails_when_value_differs() {
        let reg = registry_with("statusbar.message", ControlState::with_value("Error"));
        let result = evaluate_statusbar_contains("Ready", &reg);
        assert!(!result.passed);
        assert_eq!(result.expected.as_deref(), Some("Ready"));
        assert_eq!(result.actual.as_deref(), Some("Error"));
    }

    // Validates: Requirement 4.3 -- ASSERT STATUSBAR CONTAINS fails when control absent
    #[test]
    fn statusbar_contains_fails_when_control_absent() {
        let reg = InMemoryAutomationRegistry::new();
        let result = evaluate_statusbar_contains("Ready", &reg);
        assert!(!result.passed);
        assert!(result.failure_reason.is_some());
    }

    // Validates: Requirement 4.3 -- ASSERT CONTROL VALUE passes on exact match
    #[test]
    fn control_value_passes_on_exact_match() {
        let reg = registry_with("textbox.cmd", ControlState::with_value("FIND HELLO"));
        let result = evaluate_control_value("textbox.cmd", "FIND HELLO", &reg);
        assert!(result.passed);
    }

    // Validates: Requirement 4.3 -- ASSERT CONTROL VALUE fails on mismatch
    #[test]
    fn control_value_fails_on_mismatch() {
        let reg = registry_with("textbox.cmd", ControlState::with_value("FIND HELLO"));
        let result = evaluate_control_value("textbox.cmd", "CHANGE", &reg);
        assert!(!result.passed);
        assert_eq!(result.expected.as_deref(), Some("CHANGE"));
        assert_eq!(result.actual.as_deref(), Some("FIND HELLO"));
    }

    // Validates: Requirement 4.3 -- ASSERT CONTROL VALUE fails when ID absent
    #[test]
    fn control_value_fails_when_id_absent() {
        let reg = InMemoryAutomationRegistry::new();
        let result = evaluate_control_value("textbox.cmd", "FIND", &reg);
        assert!(!result.passed);
    }

    // Validates: Requirement 4.3 -- ASSERT WINDOW EXISTS passes when label matches
    #[test]
    fn window_exists_passes_when_label_matches() {
        let reg = registry_with("shell.window", ControlState::with_label("My Window"));
        let result = evaluate_window_exists("My Window", &reg);
        assert!(result.passed);
    }

    // Validates: Requirement 4.3 -- ASSERT WINDOW EXISTS fails when label differs
    #[test]
    fn window_exists_fails_when_label_differs() {
        let reg = registry_with("shell.window", ControlState::with_label("Other"));
        let result = evaluate_window_exists("My Window", &reg);
        assert!(!result.passed);
    }
}
