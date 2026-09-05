//! FFTest report generation -- JSON and HTML output.
//!
//! Validates: Requirement 7.1, 7.2, 7.3, 7.4, 7.6 (automated-dialog-testing)

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::assertions::AssertionResult;
use crate::runner::{RunReport, StepResult};

// === JSON report types ======================================================

/// Per-assertion record in the JSON report.
///
/// Validates: Requirement 7.3
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonAssertion {
    pub text: String,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

/// Per-test record in the JSON report.
///
/// Validates: Requirement 7.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTestResult {
    pub name: String,
    pub status: String,
    pub duration_ms: u64,
    pub assertions: Vec<JsonAssertion>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<String>,
}

/// Top-level JSON report.
///
/// Validates: Requirement 7.1, 7.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonReport {
    pub suite: String,
    pub timestamp: String,
    pub duration_ms: u64,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub tests: Vec<JsonTestResult>,
}

// === build_json_report =======================================================

/// Build a [`JsonReport`] from a [`RunReport`].
///
/// `suite_name` is the logical name of the test suite (e.g. the script file stem).
/// `timestamp` should be an ISO-8601 string; pass `""` in tests.
///
/// Validates: Requirement 7.1, 7.3
pub fn build_json_report(suite_name: &str, timestamp: &str, report: &RunReport) -> JsonReport {
    let assertions: Vec<JsonAssertion> = report
        .steps
        .iter()
        .filter_map(|s| s.assertion.as_ref().map(json_assertion))
        .collect();

    let errors: Vec<String> = report
        .steps
        .iter()
        .filter(|s| !s.passed && s.assertion.is_none())
        .filter_map(|s| s.diagnostic.clone())
        .collect();

    let status = if report.all_passed() { "PASS" } else { "FAIL" }.to_string();

    let test = JsonTestResult {
        name: suite_name.to_string(),
        status,
        duration_ms: duration_ms(report.duration),
        assertions,
        errors,
    };

    JsonReport {
        suite: suite_name.to_string(),
        timestamp: timestamp.to_string(),
        duration_ms: duration_ms(report.duration),
        total: report.total_assertions,
        passed: report.passed,
        failed: report.failed,
        tests: vec![test],
    }
}

fn json_assertion(a: &AssertionResult) -> JsonAssertion {
    JsonAssertion {
        text: a.assertion_text.clone(),
        passed: a.passed,
        expected: a.expected.clone(),
        actual: a.actual.clone(),
        failure_reason: a.failure_reason.clone(),
    }
}

fn duration_ms(d: Duration) -> u64 {
    d.as_millis() as u64
}

// === serialise_json ==========================================================

/// Serialise a [`JsonReport`] to a JSON string.
///
/// Validates: Requirement 7.1
pub fn serialise_json(report: &JsonReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

// === build_html_report =======================================================

/// Build a self-contained HTML report string from a [`RunReport`].
///
/// Produces:
/// - Summary table at the top (suite, total/passed/failed, duration)
/// - Per-step expandable `<details>` sections
/// - Failure details with expected vs actual
///
/// Validates: Requirement 7.2, 7.4
pub fn build_html_report(suite_name: &str, timestamp: &str, report: &RunReport) -> String {
    let status_label = if report.all_passed() { "PASS" } else { "FAIL" };
    let status_colour = if report.all_passed() { "#2a2" } else { "#c22" };
    let duration = duration_ms(report.duration);

    let mut rows = String::new();
    for step in &report.steps {
        let icon = if step.passed { "&#10003;" } else { "&#10007;" };
        let colour = if step.passed { "#2a2" } else { "#c22" };
        let detail = build_step_detail(step);
        rows.push_str(&format!(
            "<details><summary><span style='color:{colour}'>{icon}</span> \
             L{line}: {desc}</summary>{detail}</details>\n",
            colour = colour,
            icon = icon,
            line = step.line,
            desc = html_escape(&step.description),
            detail = detail,
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><title>FFTest Report: {suite}</title>
<style>
body{{font-family:monospace;background:#1e1e1e;color:#d4d4d4;padding:1em}}
table{{border-collapse:collapse;margin-bottom:1em}}
th,td{{border:1px solid #444;padding:4px 8px}}
th{{background:#333}}
details{{margin:2px 0;padding:2px 4px;background:#252525;border-radius:3px}}
summary{{cursor:pointer}}
.fail{{color:#c22}} .pass{{color:#2a2}}
</style>
</head>
<body>
<h2>FFTest Report: {suite}</h2>
<table>
<tr><th>Suite</th><td>{suite}</td></tr>
<tr><th>Timestamp</th><td>{ts}</td></tr>
<tr><th>Status</th><td style="color:{sc}">{sl}</td></tr>
<tr><th>Total assertions</th><td>{total}</td></tr>
<tr><th>Passed</th><td class="pass">{passed}</td></tr>
<tr><th>Failed</th><td class="fail">{failed}</td></tr>
<tr><th>Duration</th><td>{dur}ms</td></tr>
</table>
<h3>Steps</h3>
{rows}
</body>
</html>"#,
        suite = html_escape(suite_name),
        ts = html_escape(timestamp),
        sc = status_colour,
        sl = status_label,
        total = report.total_assertions,
        passed = report.passed,
        failed = report.failed,
        dur = duration,
        rows = rows,
    )
}

fn build_step_detail(step: &StepResult) -> String {
    if step.passed {
        return String::new();
    }
    let mut out = String::from("<div style='padding:4px;color:#c22'>");
    if let Some(diag) = &step.diagnostic {
        out.push_str(&format!("<p>Diagnostic: {}</p>", html_escape(diag)));
    }
    if let Some(a) = &step.assertion {
        if let Some(exp) = &a.expected {
            out.push_str(&format!(
                "<p>Expected: <code>{}</code></p>",
                html_escape(exp)
            ));
        }
        if let Some(act) = &a.actual {
            out.push_str(&format!("<p>Actual: <code>{}</code></p>", html_escape(act)));
        }
    }
    out.push_str("</div>");
    out
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::{
        AutomationId, AutomationRegistry as _, ControlState, InMemoryAutomationRegistry,
    };
    use crate::parser::parse;
    use crate::runner::Runner;

    fn make_report(src: &str) -> RunReport {
        let mut reg = InMemoryAutomationRegistry::new();
        reg.register(
            AutomationId::new("statusbar.message"),
            ControlState::with_value("Ready"),
        );
        let script = parse(src).expect("parse ok");
        let ids = vec![AutomationId::new("statusbar.message")];
        Runner::new(&reg, ids).run(&script)
    }

    // Validates: Requirement 7.1 -- JSON report serialises without error
    #[test]
    fn json_report_serialises_without_error() {
        let report = make_report("ASSERT STATUSBAR CONTAINS \"Ready\"");
        let json_rep = build_json_report("test_suite", "2025-01-01T00:00:00Z", &report);
        let json = serialise_json(&json_rep).expect("serialise ok");
        assert!(json.contains("\"suite\""));
        assert!(json.contains("test_suite"));
    }

    // Validates: Requirement 7.3 -- JSON report includes suite, timestamp, duration, per-test status
    #[test]
    fn json_report_includes_required_fields() {
        let report = make_report("ASSERT STATUSBAR CONTAINS \"Ready\"");
        let json_rep = build_json_report("my_suite", "2025-06-01T12:00:00Z", &report);
        assert_eq!(json_rep.suite, "my_suite");
        assert_eq!(json_rep.timestamp, "2025-06-01T12:00:00Z");
        assert_eq!(json_rep.total, 1);
        assert_eq!(json_rep.passed, 1);
        assert_eq!(json_rep.failed, 0);
        assert_eq!(json_rep.tests[0].status, "PASS");
    }

    // Validates: Requirement 7.3 -- failing assertion captured in JSON with expected/actual
    #[test]
    fn json_report_captures_failing_assertion_details() {
        let report = make_report("ASSERT STATUSBAR CONTAINS \"Error\"");
        let json_rep = build_json_report("fail_suite", "", &report);
        assert_eq!(json_rep.failed, 1);
        let assertion = &json_rep.tests[0].assertions[0];
        assert!(!assertion.passed);
        assert_eq!(assertion.expected.as_deref(), Some("Error"));
        assert_eq!(assertion.actual.as_deref(), Some("Ready"));
    }

    // Validates: Requirement 7.2 -- HTML report contains required structural elements
    #[test]
    fn html_report_contains_summary_table_and_steps() {
        let report = make_report("ASSERT STATUSBAR CONTAINS \"Ready\"");
        let html = build_html_report("my_suite", "2025-01-01", &report);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("FFTest Report: my_suite"));
        assert!(html.contains("<table>"));
        assert!(html.contains("<details>"));
    }

    // Validates: Requirement 7.4 -- HTML report shows failure details with expected/actual
    #[test]
    fn html_report_shows_failure_details() {
        let report = make_report("ASSERT STATUSBAR CONTAINS \"Error\"");
        let html = build_html_report("fail_suite", "", &report);
        assert!(html.contains("Expected:"));
        assert!(html.contains("Error"));
        assert!(html.contains("Actual:"));
        assert!(html.contains("Ready"));
    }

    // Validates: Requirement 7.3 -- JSON round-trips through serde
    #[test]
    fn json_report_round_trips_through_serde() {
        let report = make_report("ASSERT STATUSBAR CONTAINS \"Ready\"");
        let json_rep = build_json_report("rt_suite", "2025-01-01T00:00:00Z", &report);
        let json = serialise_json(&json_rep).expect("serialise ok");
        let decoded: JsonReport = serde_json::from_str(&json).expect("deserialise ok");
        assert_eq!(decoded.suite, json_rep.suite);
        assert_eq!(decoded.total, json_rep.total);
        assert_eq!(decoded.passed, json_rep.passed);
    }
}
