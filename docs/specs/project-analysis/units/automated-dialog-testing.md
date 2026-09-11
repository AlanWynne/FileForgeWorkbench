# Analysis Record: automated-dialog-testing (W5.19 / row 77)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-fftest` (FFTest Automated Dialog Testing Framework -- native
  UI-testing subsystem: automation IDs, scripting language, execution, record/playback,
  headless run, reporting, visual regression, plugin testing, bug-report generation)
- **Spec files**: requirements.md (392 lines, 13 requirements), tasks.md
  (50 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy + cap (PA-STD-073)

13 reqs / 392 lines, 8 files. Not a crate-split candidate (cohesive test framework). One
cap file: `parser.rs` = 406 non-test (the FFTest scripting-language parser -- Req 3, just
over the 400 cap). Recorded PA-STD-073 (MEDIUM -- cap): split parser.rs (lexer / grammar /
AST) if it grows; borderline, low urgency. REFACTOR.

---

## 2. Cross-unit consistency -- CLEAN + POSITIVE (egui-decoupled test framework)

### WIRED (NOT an orphan) + egui-INDEPENDENT design

`ff-fftest` is referenced 37x outside the crate (the same FFTest driven by the
batch-execution `fftest_cli.rs` runner, W5.6 -- confirming the integration; + governance
tests). NOT an orphan.

POSITIVE architecture: despite being a UI-testing framework, ff-fftest has NO egui dep
(deps ff-core + serde only). It inspects/drives the UI through an AUTOMATION-ID model
(Req 2): `AutomationId::new("shell.window")`, `AutomationId::new("statusbar.message")`,
assertions over `&[AutomationId]`. So the test framework is DECOUPLED from egui -- the
shell exposes named automation IDs, and FFTest asserts against them. This is a clean,
testable design (the framework does not reach into egui internals) and mirrors the
clean-seam pattern seen across the project.

### Legitimate test-infrastructure fs (not a VFS violation)

7 std::fs usages, all in bug_report.rs (bug-report generation, Req 12) + capture.rs
(visual-regression screenshot artifacts, Req 8: create_dir_all / write / read / read_dir /
remove_file). Writing test reports + screenshot artifacts to disk is exactly what a test
framework does -- LEGITIMATE (like the toolchain's test fs, W5.5). Test artifacts are NOT
workbench documents, so no VFS mediation is expected. Clean (contrast JES data-store
raw-fs PA-CONFLICT-015).

### No duplication

FFTest scripting, automation IDs, record/playback, visual regression, bug-report gen --
sole-owned by ff-fftest. No overlap with other units. (It is the project's UI-test
harness; batch-execution's fftest_cli is a thin CLI over it -- reuse, not duplication.)

### Public types and ownership

- AutomationId, FFTest script parser/executor, capture/visual-regression, reporting,
  bug-report generation -- sole-owned. Consumed by ff-desktop (automation IDs) + the CLI
  runner + governance tests.

### Cross-reference integrity

Cross-refs (ff-core, ff-desktop automation IDs, batch fftest_cli) resolve + are wired.

---

## 3. Completeness

Tracking: all 50 sub-tasks `[x]`. Implementation present across all 13 reqs (foundation,
automation IDs, scripting, execution, record/playback, headless, reporting, visual
regression, plugin testing, test-repo structure, workspace inspection, bug-report gen,
script suite) + WIRED. Genuinely complete. No PA-INCOMPLETE.

### TCR -- EXEMPLARY (62 rows)

TCR.md has 62 rows for ff-fftest across 13 reqs -- the SECOND-BEST coverage in the analysis
(after JES's 74). Fitting for a test framework. Positive; no PA-TCR gap.

---

## 4. Logging audit

- `ff_logging` / `log_*!`: 0; `ff-logging` Cargo dep: ABSENT (not dead).
- `std::fs`: 7 (legitimate test-artifact I/O).

A test framework has its OWN Reporting (Req 7) + bug-report (Req 12) mechanisms -- its
"logging" is the structured test report, not ff-logging. So the ff-logging absence is
largely defensible. Recorded PA-LOG-052 (LOW / optional): dev-logging on script-execution
steps + assertion failures could aid debugging FFTest itself, but the reporting layer
already covers the primary need. Low priority.

---

## 5. Task revision proposals

- **PA-STD-073 (MEDIUM -- cap, low urgency)**: parser.rs 406 -- split (lexer/grammar/AST)
  if it grows. REFACTOR.
- **PA-LOG-052 (LOW / optional)**: optional dev-logging on script execution + assertion
  failures (reporting layer already covers the main need).

No PA-CONFLICT (no duplication, wired, egui-decoupled). No PA-TCR gap (62 rows --
exemplary). No orphan. No raw-fs violation (legitimate test-artifact fs). No ASCII item
(0 non-comment). No PA-DOC (crate name ff-fftest matches usage).

---

## Summary

automated-dialog-testing (`ff-fftest`) is a strong, well-tested, well-integrated unit -- a
Wave-5 positive. The FFTest UI-testing framework (automation IDs, scripting language,
record/playback, headless, reporting, visual regression, plugin testing, bug-report
generation; 13 reqs, 50/50) is WIRED (37 refs -- the same FFTest driven by batch-execution's
fftest_cli, W5.6) and has EXEMPLARY coverage (62 TCR rows, second only to JES). Notable
POSITIVE design: it is egui-DECOUPLED -- it drives/inspects the UI through an AUTOMATION-ID
model (AutomationId::new("shell.window"), etc.) rather than reaching into egui, keeping the
harness clean + testable. Its 7 std::fs usages are LEGITIMATE test-infrastructure I/O
(bug reports + visual-regression screenshots), not a VFS violation. Findings are minimal:
parser.rs at 406 (PA-STD-073, cap, low urgency) and optional dev-logging (PA-LOG-052, LOW --
the framework's own Reporting/bug-report layer already covers the primary need). No
conflict, no duplication, no orphan, no incompleteness.
