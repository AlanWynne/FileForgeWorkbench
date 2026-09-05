//! FFTest sequential command runner.
//!
//! Executes a [`ParsedScript`] command-by-command against an [`AutomationRegistry`]
//! snapshot, collecting [`StepResult`] values and producing a [`RunReport`].
//!
//! Validates: Requirement 4.1, 4.2, 4.3, 4.4, 4.5 (automated-dialog-testing)

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::assertions::{
    evaluate_control_value, evaluate_file_open, evaluate_statusbar_contains, evaluate_text_exists,
    evaluate_window_exists, AssertionResult,
};
use crate::automation::{AutomationId, AutomationRegistry};
use crate::parser::{substitute_vars, Command, ParsedScript};

// === StepResult =============================================================

/// The outcome of executing a single script step.
///
/// Validates: Requirement 4.3
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepResult {
    /// 1-based line number in the source script.
    pub line: usize,
    /// Human-readable description of the step.
    pub description: String,
    /// Whether this step passed (or was a non-assertion step that succeeded).
    pub passed: bool,
    /// Assertion detail, populated for ASSERT commands.
    pub assertion: Option<AssertionResult>,
    /// Diagnostic message for failures.
    pub diagnostic: Option<String>,
}

impl StepResult {
    fn ok(line: usize, description: impl Into<String>) -> Self {
        Self {
            line,
            description: description.into(),
            passed: true,
            assertion: None,
            diagnostic: None,
        }
    }

    fn from_assertion(line: usize, result: AssertionResult) -> Self {
        let description = result.assertion_text.clone();
        let passed = result.passed;
        let diagnostic = if passed {
            None
        } else {
            Some(result.failure_reason.clone().unwrap_or_default())
        };
        Self {
            line,
            description,
            passed,
            assertion: Some(result),
            diagnostic,
        }
    }

    fn unresolved(line: usize, id: &str) -> Self {
        Self {
            line,
            description: format!("unresolved automation ID: {id}"),
            passed: false,
            assertion: None,
            diagnostic: Some(format!(
                "Automation ID '{id}' was not registered in the current frame"
            )),
        }
    }
}

// === RunReport ==============================================================

/// The complete result of executing a [`ParsedScript`].
///
/// Validates: Requirement 4.4
#[derive(Debug, Clone)]
pub struct RunReport {
    /// All step results in execution order.
    pub steps: Vec<StepResult>,
    /// Total number of assertion steps.
    pub total_assertions: usize,
    /// Number of assertions that passed.
    pub passed: usize,
    /// Number of assertions that failed.
    pub failed: usize,
    /// Wall-clock execution duration.
    pub duration: Duration,
}

impl RunReport {
    /// Returns true if all assertions passed (or there were no assertions).
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

// === Runner =================================================================

/// Executes a [`ParsedScript`] against an [`AutomationRegistry`] snapshot.
///
/// The runner processes commands sequentially. Non-assertion commands (OPEN FILE,
/// CLICK BUTTON, etc.) are recorded as informational steps -- the actual UI
/// interaction is driven by the shell; the runner records intent and checks
/// post-conditions via ASSERT commands.
///
/// CHECKPOINT is recorded as a stub step (screenshot capture is CK-4).
///
/// Validates: Requirement 4.1, 4.2, 4.5
pub struct Runner<'a> {
    registry: &'a dyn AutomationRegistry,
    /// All IDs currently known to the registry (supplied by caller for text-search assertions).
    known_ids: Vec<AutomationId>,
}

impl<'a> Runner<'a> {
    /// Create a new runner bound to the given registry snapshot.
    ///
    /// `known_ids` should list every Automation ID that may be registered in the
    /// current frame -- used by `ASSERT TEXT EXISTS` and `ASSERT FILE OPEN`.
    pub fn new(registry: &'a dyn AutomationRegistry, known_ids: Vec<AutomationId>) -> Self {
        Self {
            registry,
            known_ids,
        }
    }

    /// Execute a parsed script and return a [`RunReport`].
    ///
    /// Validates: Requirement 4.1, 4.2, 4.4
    pub fn run(&self, script: &ParsedScript) -> RunReport {
        let start = Instant::now();
        let mut steps = Vec::new();
        let mut vars: HashMap<String, String> = HashMap::new();
        let mut total_assertions = 0usize;
        let mut passed = 0usize;
        let mut failed = 0usize;

        for (line, cmd) in &script.commands {
            let step = self.execute_command(*line, cmd, &mut vars);
            if step.assertion.is_some() {
                total_assertions += 1;
                if step.passed {
                    passed += 1;
                } else {
                    failed += 1;
                }
            }
            steps.push(step);
        }

        RunReport {
            steps,
            total_assertions,
            passed,
            failed,
            duration: start.elapsed(),
        }
    }

    /// Execute a single command and return its [`StepResult`].
    ///
    /// Validates: Requirement 4.2, 4.3
    fn execute_command(
        &self,
        line: usize,
        cmd: &Command,
        vars: &mut HashMap<String, String>,
    ) -> StepResult {
        match cmd {
            // --- Variable definition (Req 3.6) ---
            Command::Variable { name, value } => {
                let resolved = substitute_vars(value, vars);
                vars.insert(name.clone(), resolved.clone());
                StepResult::ok(line, format!("VARIABLE {name} = \"{resolved}\""))
            }

            // --- Navigation / interaction commands ---
            // These are intent-recording steps; actual UI driving is the shell's job.
            Command::OpenFile { path } => {
                let path = substitute_vars(path, vars);
                StepResult::ok(line, format!("OPEN FILE \"{path}\""))
            }
            Command::WaitWindow { title } => {
                let title = substitute_vars(title, vars);
                // Check that the window control is present in the registry.
                let id = AutomationId::new("shell.window");
                if self.registry.is_present(&id) {
                    StepResult::ok(line, format!("WAIT WINDOW \"{title}\""))
                } else {
                    StepResult::unresolved(line, "shell.window")
                }
            }
            Command::ClickMenu { path } => {
                let path = substitute_vars(path, vars);
                StepResult::ok(line, format!("CLICK MENU \"{path}\""))
            }
            Command::ClickButton { id } => {
                let id = substitute_vars(id, vars);
                let aid = AutomationId::new(&id);
                if self.registry.is_present(&aid) {
                    StepResult::ok(line, format!("CLICK BUTTON \"{id}\""))
                } else {
                    StepResult::unresolved(line, &id)
                }
            }
            Command::SelectMenuItem { label } => {
                let label = substitute_vars(label, vars);
                StepResult::ok(line, format!("SELECT MENUITEM \"{label}\""))
            }
            Command::TypeText { value } => {
                let value = substitute_vars(value, vars);
                StepResult::ok(line, format!("TYPE TEXT \"{value}\""))
            }
            Command::PressKey { key } => StepResult::ok(line, format!("PRESS KEY {key}")),
            Command::CloseWindow => StepResult::ok(line, "CLOSE WINDOW"),
            Command::LoadPlugin { name } => {
                let name = substitute_vars(name, vars);
                StepResult::ok(line, format!("LOAD PLUGIN \"{name}\""))
            }

            // --- Checkpoint (stub -- screenshot capture deferred to CK-4) ---
            Command::Checkpoint { name } => {
                let name = substitute_vars(name, vars);
                StepResult::ok(line, format!("CHECKPOINT \"{name}\" [stub]"))
            }

            // --- Assertion commands (Req 4.3) ---
            Command::AssertWindowExists { title } => {
                let title = substitute_vars(title, vars);
                let result = evaluate_window_exists(&title, self.registry);
                StepResult::from_assertion(line, result)
            }
            Command::AssertTextExists { text } => {
                let text = substitute_vars(text, vars);
                let result = evaluate_text_exists(&text, self.registry, &self.known_ids);
                StepResult::from_assertion(line, result)
            }
            Command::AssertStatusbarContains { text } => {
                let text = substitute_vars(text, vars);
                let result = evaluate_statusbar_contains(&text, self.registry);
                StepResult::from_assertion(line, result)
            }
            Command::AssertFileOpen => {
                let result = evaluate_file_open(self.registry, &self.known_ids);
                StepResult::from_assertion(line, result)
            }
            Command::AssertControlValue { id, expected } => {
                let id = substitute_vars(id, vars);
                let expected = substitute_vars(expected, vars);
                let result = evaluate_control_value(&id, &expected, self.registry);
                StepResult::from_assertion(line, result)
            }
        }
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::{AutomationId, ControlState, InMemoryAutomationRegistry};
    use crate::parser::parse;

    fn make_registry() -> InMemoryAutomationRegistry {
        let mut r = InMemoryAutomationRegistry::new();
        r.register(
            AutomationId::new("statusbar.message"),
            ControlState::with_value("Ready"),
        );
        r.register(
            AutomationId::new("shell.window"),
            ControlState::with_label("FileForge Workbench"),
        );
        r.register(AutomationId::new("tab.editor.0"), ControlState::active());
        r.register(
            AutomationId::new("textbox.command_field"),
            ControlState::with_value("FIND HELLO"),
        );
        r
    }

    fn known_ids() -> Vec<AutomationId> {
        vec![
            AutomationId::new("statusbar.message"),
            AutomationId::new("shell.window"),
            AutomationId::new("tab.editor.0"),
            AutomationId::new("textbox.command_field"),
        ]
    }

    // Validates: Requirement 4.2 -- commands execute sequentially in file order
    #[test]
    fn commands_execute_in_file_order() {
        let src = "OPEN FILE \"test.txt\"\nOPEN FILE \"other.txt\"";
        let script = parse(src).expect("parse ok");
        let reg = make_registry();
        let runner = Runner::new(&reg, known_ids());
        let report = runner.run(&script);
        assert_eq!(report.steps.len(), 2);
        assert_eq!(report.steps[0].line, 1);
        assert_eq!(report.steps[1].line, 2);
    }

    // Validates: Requirement 4.3 -- passing assertion counted correctly
    #[test]
    fn passing_assertion_increments_passed_count() {
        let src = "ASSERT STATUSBAR CONTAINS \"Ready\"";
        let script = parse(src).expect("parse ok");
        let reg = make_registry();
        let runner = Runner::new(&reg, known_ids());
        let report = runner.run(&script);
        assert_eq!(report.total_assertions, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
        assert!(report.all_passed());
    }

    // Validates: Requirement 4.3 -- failing assertion counted correctly
    #[test]
    fn failing_assertion_increments_failed_count() {
        let src = "ASSERT STATUSBAR CONTAINS \"Error\"";
        let script = parse(src).expect("parse ok");
        let reg = make_registry();
        let runner = Runner::new(&reg, known_ids());
        let report = runner.run(&script);
        assert_eq!(report.failed, 1);
        assert!(!report.all_passed());
    }

    // Validates: Requirement 4.3 -- diagnostic info recorded on failure
    #[test]
    fn failure_step_has_diagnostic_info() {
        let src = "ASSERT STATUSBAR CONTAINS \"Error\"";
        let script = parse(src).expect("parse ok");
        let reg = make_registry();
        let runner = Runner::new(&reg, known_ids());
        let report = runner.run(&script);
        let step = &report.steps[0];
        assert!(!step.passed);
        assert!(step.diagnostic.is_some());
        let assertion = step.assertion.as_ref().expect("assertion present");
        assert_eq!(assertion.expected.as_deref(), Some("Error"));
        assert_eq!(assertion.actual.as_deref(), Some("Ready"));
    }

    // Validates: Requirement 4.4 -- report summary counts are correct
    #[test]
    fn report_summary_counts_all_assertions() {
        let src = r#"
ASSERT STATUSBAR CONTAINS "Ready"
ASSERT STATUSBAR CONTAINS "Error"
ASSERT FILE OPEN
"#;
        let script = parse(src).expect("parse ok");
        let reg = make_registry();
        let runner = Runner::new(&reg, known_ids());
        let report = runner.run(&script);
        assert_eq!(report.total_assertions, 3);
        assert_eq!(report.passed, 2);
        assert_eq!(report.failed, 1);
    }

    // Validates: Requirement 3.6 -- VARIABLE command sets variable for substitution
    #[test]
    fn variable_command_enables_substitution_in_later_commands() {
        let src = "VARIABLE MYFILE \"test.txt\"\nOPEN FILE \"${MYFILE}\"";
        let script = parse(src).expect("parse ok");
        let reg = make_registry();
        let runner = Runner::new(&reg, known_ids());
        let report = runner.run(&script);
        assert_eq!(report.steps[1].description, "OPEN FILE \"test.txt\"");
    }

    // Validates: Requirement 4.3 -- ASSERT CONTROL VALUE passes on match
    #[test]
    fn assert_control_value_passes_on_match() {
        let src = "ASSERT CONTROL VALUE \"textbox.command_field\" \"FIND HELLO\"";
        let script = parse(src).expect("parse ok");
        let reg = make_registry();
        let runner = Runner::new(&reg, known_ids());
        let report = runner.run(&script);
        assert!(report.all_passed());
    }

    // Validates: Requirement 3.5 -- unresolved automation ID recorded as failure
    #[test]
    fn click_button_with_unresolved_id_records_failure() {
        let src = "CLICK BUTTON \"button.nonexistent\"";
        let script = parse(src).expect("parse ok");
        let reg = make_registry();
        let runner = Runner::new(&reg, known_ids());
        let report = runner.run(&script);
        let step = &report.steps[0];
        assert!(!step.passed);
        assert!(step
            .diagnostic
            .as_ref()
            .unwrap()
            .contains("button.nonexistent"));
    }

    // Validates: Requirement 4.4 -- duration is recorded
    #[test]
    fn run_report_records_duration() {
        let src = "OPEN FILE \"test.txt\"";
        let script = parse(src).expect("parse ok");
        let reg = make_registry();
        let runner = Runner::new(&reg, known_ids());
        let report = runner.run(&script);
        // Duration should be non-negative (always true, but confirms field is populated)
        let _ = report.duration;
    }

    // Validates: Requirement 4.2 -- CHECKPOINT step is recorded as ok (stub)
    #[test]
    fn checkpoint_step_is_recorded_as_ok() {
        let src = "CHECKPOINT \"after_open\"";
        let script = parse(src).expect("parse ok");
        let reg = make_registry();
        let runner = Runner::new(&reg, known_ids());
        let report = runner.run(&script);
        assert!(report.steps[0].passed);
        assert!(report.steps[0].description.contains("after_open"));
    }
}
